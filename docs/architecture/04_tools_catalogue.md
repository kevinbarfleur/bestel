# 04 — Tools catalogue

The 18 tools exposed to the agent. Source of truth:
`crates/bestel-core/src/llm/tools.rs::tool_schemas()` and
`crates/bestel-core/src/llm/sheet_tools.rs::schemas()`. The latter is
spliced in via `schemas.extend(sheet_tools::schemas())` at the end of
`tool_schemas()`.

13 general-purpose tools + 5 Build Sheet tools = **18 tools**.

The MCP server (`bestel mcp-serve`) exposes the 13 general-purpose
tools only — Build Sheet tools need a UI to be useful.

---

## Quick reference

| # | Const name (Rust) | Wire name | Category | Per-turn cap |
|---|---|---|---|---|
| 1 | `GET_ACTIVE_BUILD` | `get_active_build` | PoB | — |
| 2 | `WIKI_SEARCH` | `wiki_search` | Wiki | — |
| 3 | `WIKI_PARSE` | `wiki_parse` | Wiki | — |
| 4 | `WIKI_SYNERGIES` | `wiki_synergies` | Wiki | — |
| 5 | `WIKI_CARGO` | `wiki_cargo` | Wiki | — |
| 6 | `TRADE_RESOLVE_STATS` | `trade_resolve_stats` | Trade | — |
| 7 | `TRADE_SEARCH_URL` | `trade_search_url` | Trade | — |
| 8 | `WEB_FETCH` | `web_fetch` | Web | — |
| 9 | `REPOE_LOOKUP` | `repoe_lookup` | Bundled snapshot | — |
| 10 | `POB_CALC` | `pob_calc` | PoB engine | — |
| 11 | `READ_INTERNAL_REFERENCE` | `read_internal_reference` | Knowledge | — |
| 12 | `LOAD_SKILL` | `load_skill` | Knowledge | 2 |
| 13 | `KB_SEARCH` | `kb_search` | Knowledge (LanceDB) | 3 |
| 14 | `SHEET_PROPOSE_SECTION` | `sheet_propose_section` | Sheet | — |
| 15 | `SHEET_ASK` | `sheet_ask` | Sheet | — |
| 16 | `SHEET_OPEN_INTERVIEW` | `sheet_open_interview` | Sheet | — |
| 17 | `SHEET_FINALIZE_REQUEST` | `sheet_finalize_request` | Sheet | — |
| 18 | `GET_ACTIVE_BUILD_SHEET` | `get_active_build_sheet` | Sheet | — |

Caps live in `tools.rs` (`KB_SEARCH_TURN_CAP = 3`, `LOAD_SKILL_TURN_CAP = 2`),
counters on `ToolCtx` as `Arc<AtomicU32>`.

---

## Dispatch path

```
provider receives stop_reason="tool_use"
   │
   ▼
collect (name, id, input_json) per tool block
   │
   ▼
BuildContext::dispatch_tool(name, args, ctx)
   │ (match by name)
   ▼
handler (sync or async) returns String / streamed chunks
   │
   ▼
emit ToolOutput chunks + ToolEnd(Done|Failed)
   │
   ▼
render_tool_result(name, result) → JSON envelope shape the provider expects
   │
   ▼
append assistant tool_use block + user tool_result block to history
   │
   ▼
re-call API
```

Pure tools (`get_active_build`, `read_internal_reference`) run sync.
Network / engine tools (`wiki_*`, `web_fetch`, `pob_calc`, `kb_search`,
`load_skill`) run async on the tokio runtime.

`render_tool_result` is mostly identity for Anthropic-shaped providers
(they accept a string in `tool_result.content`); Ollama has a slightly
different envelope handled in `ollama.rs`.

### `ToolCtx` — what each handler can reach

```rust
pub struct ToolCtx {
    pub build: BuildContext,                 // Arc<RwLock<Option<PobBuild>>>
    pub wiki_poe1: WikiClient,
    pub wiki_poe2: WikiClient,
    pub trade_poe1: TradeClient,
    pub trade_poe2: TradeClient,
    pub http: PoeHttpClient,                 // shared reqwest client + rate limiter
    pub pob_engine: Option<Arc<PobEngineHandle>>,  // None in MCP-serve
    pub kb: Option<SharedKbEngine>,          // None until LanceDB boot finishes
    pub kb_calls_this_turn: Arc<AtomicU32>,
    pub skill_calls_this_turn: Arc<AtomicU32>,
    pub user_question: Option<String>,       // for server-side kb_search pre-attach
    pub deltas: Option<UnboundedSender<LlmDelta>>,  // for sheet tools
}
```

A fresh `ToolCtx` is constructed per LLM iteration so the counters
reset implicitly.

---

## 1 · `get_active_build`

**Purpose.** Returns the exile's currently loaded PoB build as JSON.
The primary build-grounding tool — **the agent must call this before
making any claim about the exile's character**.

**Args.** None.

**Returns.** Big JSON object:

- Structural: game (PoE1 / PoE2), class, ascendancy, level, main skill,
  every skill group with linked gems, every item with full raw text,
  passive tree summary (class id, ascendancy id, version, node count,
  mastery count, weapon-set node split)
- Defensive stats: life, mana, ES, EHP, armour, evasion, suppression,
  block, dodge, per-element resists + max-hit values, charges
  (power / frenzy / endurance current+max), active buffs / curses,
  config (boss profile, enemy resists, flask uptimes, custom mods)
- **Semantic facts** (see [05 § Semantic layer](./05_pob_integration.md#semantic-layer-buildidentity)):
  `archetype` (defense / hit_model / mechanic tag vectors),
  `defining_uniques` (each tagged `engine | defining | amplifier`),
  `conversion_chain` (verbatim damage-conversion steps)
- **Server-side KB pre-attachment**: when `ToolCtx::user_question` is
  populated, `dispatch` runs a `kb.search(user_question, top_k=3)` and
  attaches the top hits as `relevant_kb` in the response (Sprint E P7).

**Trigger rules.** Always first. Refuses to comment on a build before
calling this. Surface `archetype` tags FIRST in the answer (do not
guess the archetype from class+ascendancy alone). Never recommend
selling an item flagged `category: "engine"` without explicit user
instruction.

---

## 2 · `wiki_search`

**Purpose.** Free-text search against the MediaWiki API for `poewiki.net`
(PoE1) or `poe2wiki.net` (PoE2).

**Args.** `query: string`, `game: "poe1" | "poe2"` (default poe1),
`limit: 1..20` (default 6).

**Returns.** Up to `limit` hits with title, snippet, canonical URL.

**Trigger.** When the agent does not know the canonical page title.
Follow up with `wiki_parse(title=…)` on the most relevant hit.

---

## 3 · `wiki_parse`

**Purpose.** Fetch the full text of a specific wiki page by exact
title. Strips infoboxes / metadata; returns title, URL, section
anchors, plain text body. Cached 12h per page.

**Args.** `title: string`, `game: "poe1" | "poe2"` (default poe1).

**Trigger.** Primary research tool for mechanics. Read
Mechanics / Caps / Interactions sections rather than relying on
memory.

---

## 4 · `wiki_synergies`

**Purpose.** Reverse-link sweep via `Special:WhatLinksHere`. Returns
the set of pages that link TO the topic, filtered to drop meta pages.
Surfaces synergies the exile didn't name (e.g.
`wiki_synergies("Divine Flesh")` reveals The Fourth Vow, Mahuxotl's
Machination, Born of Chaos).

**Args.** `topic: string`, `game`, `limit: 1..200` (default 80).

**Trigger.** After understanding a core mechanic — use this to
surface uniques / keystones / cluster jewel notables / gems that
matter for a creator-grade answer.

---

## 5 · `wiki_cargo`

**Purpose.** Structured-table queries against the wiki's Cargo
(Semantic MediaWiki) tables. Niche but exact for mod tier weights,
item base stats, gem tags, version history.

**Args.** `table: string` (`items`, `item_stats`, `mods`, `skill`,
`versions`), `fields: string[]`, `where: string` (SQL-like), `game`,
`limit: 1..500` (default 50).

**Trigger.** Only when the structured data is needed and `repoe_lookup`
doesn't carry it. Read `Special:CargoTables` for schemas before
calling.

---

## 6 · `trade_resolve_stats`

**Purpose.** Map a stat phrase → trade stat ID. The trade site filters
by IDs (`pseudo.pseudo_total_life`,
`explicit.stat_3299347043`), not free text. Required precursor for any
trade search.

**Args.** `phrase: string`, `game`, `prefer_pseudo: bool` (default
true), `limit: 1..20` (default 6).

**Returns.** Ranked matches with `id`, `type`, `text`, `min`/`max`/`option`
where applicable.

**Trigger.** Before constructing any trade search.

---

## 7 · `trade_search_url`

**Purpose.** Build a shareable `pathofexile.com/trade` URL from a
query body. **Bestel does NOT execute the search live** to avoid
hitting GGG rate limits — the exile clicks the URL.

**Args.** `league: string`, `query_body: object` (full trade-API JSON
including `query.stats[].filters`), `game`.

**Returns.** The URL string the exile can open.

**Trigger.** When the user wants a price check or item search. Pair
with `trade_resolve_stats` to populate `query_body`.

---

## 8 · `web_fetch`

**Purpose.** HTTP GET against the source allowlist. Strips HTML,
truncates large bodies, surfaces explicit errors on blocklisted hosts.

**Args.** `url: string` (full http(s) URL).

**Allowlist** (21 hosts, 7 tiers): see
[03 § Source allowlist and blocklist](./03_knowledge_layer_and_rag.md#source-allowlist-and-blocklist).

**Blocklist** (10 hosts): aoeah, mmogah, iggm, ggwtb, boostmatch,
sportskeeda, gamewatcher, switchbladegaming, dotesports, gamerant.

**Trigger.** Patch notes (`pathofexile.com/forum/view-thread/…`),
PoEDB pages, Maxroll / Pohx / Mobalytics articles. Off-allowlist hosts
return an explicit error — retry with an allowlisted source.

---

## 9 · `repoe_lookup`

**Purpose.** Query the bundled `repoe-fork` snapshot. **ALWAYS try
this BEFORE `wiki_search` / `wiki_parse` / `wiki_cargo` / `web_fetch`**
when the question is about a base item, unique, mod pool, gem, cluster
jewel notable, essence, fossil, or stat translation. < 10 ms cold, no
network round-trip.

**Args.** `game: "poe1" | "poe2"`, `category: "mods" | "base_items" |
"gems" | "uniques" | "cluster_jewels" | "essences" | "fossils" |
"stat_translations"`, `id: string` (exact metadata path) OR
`name: string` (fuzzy token-overlap), `limit: 1..20` (default 5).

**Returns.** Matching JSON object(s) plus snapshot metadata (`source:
"bundled" | "refreshed"`, `fetched_at`).

**PoE2 caveat.** Only `mods`, `base_items`, `uniques` are shipped;
`gems` and `stat_translations` placeholder 11-byte files exist (the
2026-05-08 upstream 404). Other categories return an explicit "not
available for poe2" error.

**Trigger.** Itemisation, mod tier, gem-tag lookups. Falls back to
wiki only when 0 entries match the name.

---

## 10 · `pob_calc`

**Purpose.** Run the bundled headless PoB engine against the active
build and return canonical output stats.

**Args.**
- `category: "offence" | "defence" | "charges" | "reservation" | "ailments" | "all"`
- `skill_index: int >= 0` (optional — select a non-default skill group)
- `calcs: { enemyIsBoss, usePowerCharges, useFrenzyCharges,
  useEnduranceCharges, forceBuffOnslaught, multiplierImpaleStacks,
  useFlask1..5 }` (optional Calcs config overrides)

**Returns.**
- Numeric stats for the requested category
- **Effective Calcs config** (echoes whatever the engine ended up using)
- **`active_skill` metadata** (`main_socket_group`, `main_active_skill`,
  `skill_part`, `active_skill_label`, `active_skill_gem`)
- `warnings: string[]`

**Critical verification step.** Before quoting any number, the agent
must compare `active_skill.active_skill_label` (or
`active_skill_gem`) with the build's `main_skill` from
`get_active_build`. If they don't match, the engine fell back to the
wrong skill — DO NOT quote the number; report the cached `<PlayerStat>`
with an explicit staleness note OR retry `pob_calc` with an explicit
`skill_index`.

**Calcs disclosure.** Two PoBs with identical gear can show 10× DPS
swings purely from Calcs config. The agent must surface assumptions
(`enemyIsBoss=Pinnacle`, `useFlask3=true`, …) — never quote a number
without naming the assumptions.

**Availability.** `ToolCtx.pob_engine` is `None` in the MCP-serve
entry point (no Tauri AppHandle, no bundled-resource path), so
`pob_calc` returns "engine not configured" there.

See [05 § Engine sidecar](./05_pob_integration.md#engine-sidecar) for
the full protocol.

---

## 11 · `read_internal_reference`

**Purpose.** Read one of the 69 bundled reference files by exact
relative path.

**Args.** `rel_path: string` — **strict enum** of every key in
`BUNDLED_REFERENCES`. The Anthropic API rejects unknown values before
the model can submit them.

**Returns.** Full markdown text (truncated at 25 KB).

**Trigger.** Only when the exact filename is already known. For
semantic queries, prefer `kb_search`. Common hallucination caught by
the enum: agents trying `09_build_archetypes.md` (plural, wrong number)
when the file is `17_build_archetype_taxonomy.md`. On a miss the error
returns top-5 token-overlap suggestions so the agent can self-correct.

---

## 12 · `load_skill`

**Purpose.** Load the full body of one progressive-disclosure **skill**
(repeatable workflow: build review, craft audit, mapping strategy)
from `crates/bestel-core/src/skills/`. The agent sees skill
descriptions + triggers in BP4 of the system prompt; this tool
materialises the full SKILL.md body when the user's question matches
a skill.

**Args.** `name: string` — strict enum of `bundled_skill_names()`.

**Returns.** SKILL.md body + names of templates under that skill.

**Cap.** `LOAD_SKILL_TURN_CAP = 2` (default 1, raised to 2 only when
the user explicitly asks for a full multi-skill audit).

**Trigger.** When the user's question matches a skill's triggers AND
the agent needs the full structure. Skills are NOT a substitute for
tool calls — `pob_calc` / `wiki_parse` / `kb_search` still run for
live data; the skill provides STRUCTURE for the answer.

---

## 13 · `kb_search`

**Purpose.** Hybrid (vector + BM25) search over the
`~/.bestel/prompts/references/**` corpus indexed in LanceDB.

**Args.** `query: string` (natural-language), `game: "poe1" | "poe2"`
(optional, filters passages), `top_k: 1..10` (default 5),
`tags: string[]` (reserved for future shards).

**Returns.** Up to `top_k` passages with parent file path
(`~/.bestel/prompts/references/<rel_path>`), heading path, body
excerpt, score.

**Cap.** `KB_SEARCH_TURN_CAP = 3` per turn. Beyond that, dispatch
returns the structural fallback `"tool_storm_kb_search"` so the model
stops digging.

**Trigger.** Any question where the agent doesn't already know the
exact source file. `read_internal_reference` is for known filenames;
`kb_search` is for semantic discovery.

**Availability.** `ToolCtx.kb` is `None` until LanceDB boot completes
(no embedding backend → KB disabled). Returns `"kb_not_ready"` in
that case.

See [03 § LanceDB KB](./03_knowledge_layer_and_rag.md#lancedb-kb).

---

## 14 · `sheet_propose_section`

**Purpose.** Build Sheets feature. Surface a draft for ONE section of
the active Build Sheet. The UI renders a card with the agent's prose
and a Confirm / Edit choice; the user works through sections one at a
time.

**Args.** `section_id: "identity" | "archetype" | "damage" | "defense"
| "items" | "intent"`, `title: string`, `body: string`.

**Side effects.** Emits `LlmDelta::SheetDraftUpdate { section_id,
title, body, confirmed }`. Tool result tells the agent to end its
turn — the user's reply is the next input.

**Trigger.** When the agent has read the PoB enough to propose prose
for one of the 6 fixed sections.

See [06](./06_build_sheet_workflow.md) for the full FSM.

---

## 15 · `sheet_ask`

**Purpose.** Build Sheets. Surface a `questions_v2`-style chip picker
for a leverage-based purpose question (purpose of an aura, role of a
unique).

**Args.** `question_id: string`, `title: string`, `subtitle: string?`,
`options: string[2..6]`, `multi: bool` (default false),
`has_other: bool` (default true).

**Side effects.** Emits `LlmDelta::SheetAskUser { … }`. The user's
answer comes back as a regular `ChatMessage`.

**Trigger.** AFTER drafting the corresponding section but BEFORE
finalising — when the leverage of getting it right is high (item that
defines the build's identity).

---

## 16 · `sheet_open_interview`

**Purpose.** Build Sheets. Emit a **complete one-shot interview** —
every section's draft body PLUS every leverage-purpose question across
sections PLUS a notes prompt, in a single delta.

**Args.** `sections: [{ id, title, draft_body }]` (exactly 6, fixed
ids), `questions: [{ … }]` (optional), `notes_prompt: string?`.

**Side effects.** Emits `LlmDelta::SheetInterviewOpen { payload }`.
The UI renders a single `BSInterviewPanel` and the user fills the
entire form in one round.

**Trigger.** AFTER `get_active_build`, `pob_calc`, AND at least 2-3 of
`{wiki_parse | kb_search | read_internal_reference}` on the build's
defining items + main skill. Never before deep analysis.

After this tool returns, the agent must end its turn. The user's
submission comes back as a structured `[INTERVIEW SUBMISSION …]` user
message; on the next turn, the agent must call
`sheet_finalize_request` directly — NOT re-emit the interview.

---

## 17 · `sheet_finalize_request`

**Purpose.** Build Sheets. Persist the completed sheet to the
`build_sheets` SQLite table.

**Args.** `name: string`, `fingerprint: string` (build fingerprint —
ascendancy:main_skill:sorted_unique_names), `pob_hash: string`,
`sections: [{ id, label, body, confirmed }]`, `defining_items: [{
name, role: "engine"|"defining"|"amplifier"|"enabler", purpose }]`
(optional), `intent: [{ constraint }]` (optional), `known_gaps: [{
label, note }]` (optional), `authored_in_chat: string?`.

**Side effects.** Writes the row. Emits `LlmDelta::SheetFinalized
{ sheet_id, name }` (the banner) AND `LlmDelta::SheetLoaded { … }`
(the sidebar card).

**Trigger.** Once, after every section has been confirmed and any
purpose questions answered.

---

## 18 · `get_active_build_sheet`

**Purpose.** Build Sheets. Return the validated sheet for the current
PoB's fingerprint (if any) AND whether it is stale vs the current PoB
hash.

**Args.** `fingerprint: string`, `pob_hash: string` (current). Optional
override; defaults to the current build's fingerprint and hash.

**Returns.** `{ id, name, payload, stale: bool, authored_at,
updated_at }` or null.

**Side effects.** Emits `LlmDelta::SheetLoaded` when a sheet is
returned, so the sidebar populates the same turn.

**Trigger.** At chat start, after the agent reads the runtime
`Build sheet:` tag (`fresh | stale | absent`). The agent only calls
this when it wants to read sheet fields directly (e.g. to cite the
user-confirmed intent in its answer).

---

## See also

- [02 — Agentic system](./02_agentic_system.md) — how the tool loop
  works and where the caps are enforced.
- [05 — PoB integration](./05_pob_integration.md) — semantic
  enrichment of `get_active_build` and `pob_calc` engine protocol.
- [06 — Build Sheet workflow](./06_build_sheet_workflow.md) — the
  FSM that the five sheet tools drive on the UI.
- [03 § Source allowlist](./03_knowledge_layer_and_rag.md#source-allowlist-and-blocklist)
  — full host lists for `web_fetch`.
