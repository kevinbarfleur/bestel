# Bestel — Intelligence Roadmap (v15+)

> Living roadmap. Update the **Status** section as sprints progress.
> Last revision: 2026-05-08 (post-pivot).

> **2026-05-08 architecture pivot.** This roadmap was substantially restructured after the 2026-05-08 run export (67 runs across Haiku 4.5 / Sonnet 4.5 / DeepSeek V3.x, 22 distinct prompts) showed that the original "more knowledge" trajectory was not closing the quality gap on small models. Two external research efforts (an internal quality audit of the export, and a deep-research review of single-agent vs multi-agent architectures) converged on a single-agent reliability-first plan: ship deterministic output guards, segmented prompt caching, an embedded hybrid RAG layer, a forced first-call context tool, and progressive-disclosure skills — in that order. The original Sprints 0–3 (knowledge / data / engine / semantic) are kept as Tier 1 with statuses brought up to date; Sprints 4 and 5 are reframed and renumbered into the new Tier 2 (sprints A–H). Anti-patterns explicitly avoided are documented in Tier 3. See § *Architecture pivot 2026-05-08* below for the source-grounded rationale, and `ARCHITECTURE.md` at the repo root (commit `eac759e`) for the current end-to-end snapshot.

## Legend

- ✅ done · 🟡 in progress · ☐ pending · ⏸ deferred · 🔁 triggered-only

## Context

Bestel is a Tauri 2 + Vue 3 + Rust desktop AI companion for Path of Exile 1 and 2. Persona: chronicler of Lioneye's Watch. The agent runs single-window for chat, with a separate prompt-editor window and a dev-panel window. Today it has 11 tools (`get_active_build`, `wiki_search`, `wiki_parse`, `wiki_synergies`, `wiki_cargo`, `trade_resolve_stats`, `trade_search_url`, `web_fetch`, `read_internal_reference`, `repoe_lookup`, `pob_calc`) backed by 27 numbered reference docs plus 4 subfolders (`creators_registry/`, `glossary/`, `poe2/`, `thresholds/`, `maxroll/`) — all bundled via `BUNDLED_REFERENCES` in `crates/bestel-core/src/prompts.rs`. It can read a PoB XML file structurally, derive a semantic build identity (Sprint 3), fetch wiki pages, resolve trade stats, and call a headless LuaJIT PoB engine for live DPS/EHP calculations.

The 2026-05-08 research report originally identified ten high-leverage improvements grouped into six sprints (Tier 1 below). After Sprints 0/1/2 shipped and Sprint 3 landed (commit `ab003ce`), the run-export revealed that knowledge depth is not the bottleneck anymore — output discipline, context delivery, and tool-failure handling are. The pivot below entrenches that diagnosis as the next four months of work.

## Architecture pivot — 2026-05-08

### Why now

The 2026-05-08 export, recorded automatically via the Recorder (`crates/bestel-core/src/llm/recorder.rs`) during a battery run on three models, surfaced these objective measurements:

- 56 of 336 tool calls failed (~16.7%).
- 19 of 29 `pob_calc` calls failed, almost all with `Classes/DropDownControl.lua:147: attempt to index a nil value`.
- 5 runs leaked raw `<thinking>` / `</thinking>` tags into the user-facing answer.
- ~1 of 24 build-loaded runs followed the prescribed *Identity card* contract (`Identity: defense=..., hit_model=..., mechanic=...`).
- 3 of 13 panel-using runs placed the `⟦panel-data⟧` sidecar at the start of the message as required.
- Several runs cited URLs that were never fetched in the same turn.
- Multi-turn drift: a build-diagnosis sequence identified physical mitigation as the bottleneck on turn 1, then proposed an elemental-resistance belt on turn 2, then asked the user to clarify which mod they meant on turn 3.
- DeepSeek frequently fell back to the PoB cache `CombinedDPS` after `pob_calc` failed and presented it as "real DPS" — a textbook engine-trust violation already documented in `prompts/references/26_validation_and_self_correction.md`.

These are not failures of *what Bestel knows*. They are failures of *how Bestel delivers and validates output*. Adding more reference docs would not move any of these numbers.

### Three load-bearing verdicts

1. **Stay single-agent, reinforced.** Cognition (Walden Yan, "Don't Build Multi-Agents", June 2025) and Anthropic Applied AI (*Effective Context Engineering for AI Agents*, September 2025) are aligned despite contrary headlines: multi-agent only wins on breadth-first parallelisable read tasks (research-style web exploration). For a coherent conversational domain expert like Bestel, multi-agent introduces fragility, latent-state divergence, and roughly 15× the token cost of a comparable single-agent chat. Cognition's rule, internalised here: *"if you can get away with building a single, linear system using LLMs, always prefer that over a mix of agents."* No permanent thematic sub-agents (Bestel-craft, Bestel-mapper) — those needs are addressed by Skills (Sprint F).

2. **The #1 quality lever is retrieval + caching + forced context, not more prompt.** The current `SYSTEM_PROMPT.md` (640 lines) and `CORE_KNOWLEDGE.md` (206 lines) are already large. Adding more text will not stop a model from emitting `<thinking>` or fabricating a Sources URL. What will:
   - Anthropic prompt caching segmented properly (TTFT down to ~15% of cold, cached input tokens at 0.1× of base price, TTL 1h via the `extended-cache-ttl-2025-04-11` header on Haiku 4.5 and Sonnet 4.5/4.6).
   - An embedded hybrid RAG (vector + BM25 + RRF) replacing manual `read_internal_reference` selection so the model stops guessing filenames.
   - A first-tool `build_context` that composes the Identity card *server-side* and bundles the top-N relevant KB chunks, eliminating the "model could have known but didn't retrieve" failure class.
   - Deterministic output lints that fail closed on `<thinking>` leaks, missing Identity card, "real DPS" after a failed engine, and mismatched panel sidecars.

3. **Quality gate first, RAG second.** If LanceDB ships before the response linter, the model can still leak `<thinking>`, fabricate citations, or claim "real DPS" after a failed `pob_calc`. RAG improves retrieval precision, not output grammar. Both layers are necessary, but the linter must come first to convert observed defects into deterministic test failures.

### Pointer

`ARCHITECTURE.md` at the repo root (commit `eac759e`) snapshots the current end-to-end design. This roadmap describes the *delta* on top of that snapshot.

## Architecture decisions

### Original (locked 2026-05-08, Sprints 0–5)

- **D1 — Doc surface**: every reference, glossary, threshold table, creator profile, validation rule lives under `prompts/references/` and is bundled via `BUNDLED_REFERENCES` in `crates/bestel-core/src/prompts.rs`. End-users edit copies under `~/.bestel/prompts/references/` via the existing prompt-editor window.
- **D2 — Tooling expansion**: every new tool follows the existing pattern in `crates/bestel-core/src/llm/tools.rs` — constant + schema in `tool_schemas()` + match arm in `dispatch()` + 25–30 KB truncation. Local-friendly subset selectively gated (≤ 8 B models stay simple).
- **D3 — Data layer strategy**: hybrid `include_bytes!` snapshot (zstd-19) + background nightly refresh task on startup. Atomic rename on disk; the agent never sees a half-written file. Bundled snapshot is the offline-first fallback.
- **D4 — PoB engine strategy**: LuaJIT sidecar binary bundled via `tauri.conf.json` `bundle.externalBin`. Vendored PathOfBuildingCommunity + PathOfBuilding-PoE2 sources as git submodules pinned to release tags. Lazy spawn on first `pob_calc` call. In-process `mlua` rejected (panic isolation, MSVC LuaJIT pain on Windows).
- **D5 — PoE2 0.5 (2026-05-29)**: scaffold `references/poe2/` subdirectory in Sprint 0 with stub files + `00_version_pinning.md`. Fill content on launch day.
- **D6 — Source-quality enforcement**: `web_fetch` gains a hard tier-4 ban list. Tier-1/2 only. Multilingual: agent reasons in English internally, translates to the user's language at output. Glossary FR↔EN bundled.

### Pivot additions (locked 2026-05-08, Sprints A–H)

- **D7 — Single-agent.** No permanent thematic sub-agents. Workflow specialisation goes into Skills/Recipes (Sprint F), not separate agent personas. Source: Cognition June 2025 + Anthropic September 2025. Multi-agent costs ~15× tokens for our conversational use case.

- **D8 — System prompt as cacheable blocks.** The Anthropic request `system` field is structured as ordered blocks with explicit `cache_control: ephemeral` on stable units (persona, tool policy, core knowledge, skill descriptions). All dynamic content (build state, user message, session notes summary) is injected *after* the last `cache_control` boundary. Tool definitions are versioned to keep the system cache stable. Header `extended-cache-ttl-2025-04-11` is enabled for Haiku 4.5 and Sonnet 4.5/4.6 to extend TTL to 1h, paying the 2× write multiplier once per stable session. Caveat: any byte change inside a cacheable block invalidates everything that follows it; this is now part of the contract.

- **D9 — Embedded hybrid RAG (vector + BM25 + RRF) via LanceDB**, behind a thin `bestel-rag::KbIndex` trait abstraction. LanceDB chosen because Rust-native (`lancedb` crate), in-process like SQLite, FTS via Tantivy integrated, RRF supported via `nearest_to() + text()` combination, no dynamic loader, Lance file format versioned. `read_internal_reference` (the current enum-locked tool) becomes a fallback for known critical files. New `kb_search` tool becomes the primary discovery path. Storage at `~/.bestel/index/kb.lance/`. Embeddings: OpenAI `text-embedding-3-small` (online default, $0.02/M tokens, 1536 dim, Matryoshka-reducible) with BGE-small/M3 via Ollama as offline fallback. Re-embedding is incremental on content-hash, never full at boot. Why not sqlite-vec, Qdrant, Chroma, Weaviate, Tantivy-only: documented in Sprint E rationale.

- **D10 — Skills/recipes as the only extension primitive.** Format: open Anthropic spec (December 2025), adopted by OpenAI (Codex CLI/ChatGPT). One `SKILL.md` per skill with frontmatter (`name`, `description`, `tools`, `answer_contract`) plus optional `templates/` and `scripts/` subfolders. Frontmatter description (~100 words) is always visible in the system prompt; the body is loaded on demand via a single `load_skill(name)` tool. Bestel implements the format directly — no Anthropic SDK dependency — to stay portable across model pivots. Caveat: the spec is recent; we lock our own minimal implementation against the published format.

## Tier 1 — Foundational sprints (knowledge / data / engine / semantic)

### Sprint 0 — Knowledge base + validation hardening ✅ (~2–3 weeks)

**Status**: ✅ done 2026-05-08.

Pure text + small `web_fetch` change. No new tools, no new crates, no network code. Highest agent-quality lift per dev-day.

**Notes**: Day-1 kickoff done 2026-05-08 (ROADMAP, 30+ stubs, `list_files()` expansion, `FETCH_BLOCKLIST`, build clean, `host_allowed` tests pass). Content fill complete: docs `14`/`15`/`16`/`17`/`18`/`19`/`20`/`21`/`22`/`23`/`24`/`26` + `thresholds/{red_maps, pinnacle, uber_pinnacle}` + `poe2/{00,01,02,03,04}` + `glossary/fr_en` + `creators_registry/*` (20 profiles). Doc 25 stays conceptual stub until Sprint 2 ships `pob_calc`. PoE2 `05` (atlas) + `06` (Runes of Aldur) stay empty stubs until 2026-05-29. New `web_fetch` blocklist: `aoeah, mmogah, iggm, ggwtb, boostmatch, sportskeeda, gamewatcher, switchbladegaming, dotesports`.

### Sprint 1 — Data layer (repoe-fork + trade-stats) ✅ (~1–2 weeks)

**Status**: ✅ done 2026-05-08.

**Notes**: New crate-internal modules `sources/{repoe, repoe_refresh, trade_catalogue}.rs`, snapshot bundle under `src/sources/snapshots/{poe1,poe2,trade_stats}/` (2.7 MB compressed total — well under the 12 MB cap), zstd-19 + `include_bytes!` + `OnceLock<RwLock<HashMap>>` lazy load. New tool `repoe_lookup` (8 categories × 2 games, id + fuzzy-name modes, NOT in `LOCAL_TOOL_ALLOWLIST`). `TradeClient::raw_stats()` refactored to consume the catalogue (existing trade tests pass). Daily refresh spawned from `bootstrap_runtime::boot_data_refresh`. `build.rs` auto-creates stub blobs so first build works without script run. `examples/refresh_snapshots.rs` + `scripts/refresh-snapshots.ps1` populate real data. PoE2 currently has 3/8 categories on repoe-fork (mods/base_items/uniques); `Category::available_for` matrix returns explicit "not available" error for the rest. 68 tests pass.

(S1-1) Resolved: `version.txt` does not exist on `repoe-fork.github.io` — refresh task uses plain GET.
(S1-2) Resolved: parity encoded in `available_for`.

### Sprint 2 — PoB engine headless ✅ (~3–4 weeks)

**Status**: ✅ done 2026-05-08. **Known bug**: `Classes/DropDownControl.lua:147: attempt to index a nil value` causes ~65% of `pob_calc` calls to fail under battery — addressed by **Sprint C2** in Tier 2.

**Notes**: New crate `bestel-pob-engine` with `protocol.rs` (ndJSON `Cmd`/`Reply`), `lifecycle.rs` (per-game process, 8 s command timeout, 10 min idle, 3-restart-window circuit-breaker), `calc.rs` (high-level orchestration: `load_build_xml` → `set_config` → `set_main_selection` → `get_stats` → echo). Vendored `PathOfBuildingCommunity` (v2.65.0) and `PathOfBuilding-PoE2` (v2.49.3) as git submodules under `crates/bestel-pob-engine/vendor/`. Forked the ianderse `api-stdio` harness from scratch under `vendor/api-stdio-bestel/api-stdio.lua` so `set_config` exposes the full Calcs key set (`enemyIsBoss`/charges/flasks/impale/onslaught). Built LuaJIT 2.1 ROLLING from upstream MIT source via MSVC; vendored Windows binary at `external/luajit/windows-x86_64/luajit.exe` (~290 KB) plus `lua51.dll`. SHA256 in `scripts/luajit.sha256.txt`, `vendor-luajit.ps1` regenerates from source. `pob_calc` tool advertised in `tools.rs` schemas (length 10 → 11), dispatch arm reads the active `PobBuild`, pipes XML to the engine, surfaces the Calcs echo in the response. `pob_calc` IS in `LOCAL_TOOL_ALLOWLIST` (simple I/O, high-signal output suits 7-8 B models). `crates/bestel-core/src/llm/pob_engine.rs` provides the lazy global handle.

(S2-1) Resolved: PoB1 v2.65.0, PoB2 v2.49.3.
(S2-2) Resolved: ianderse `set_config` exposed only `bandit`/`pantheon`/`enemyLevel`; we forked and extended.
(S2-3) Deferred: code-signing — ships unsigned with SmartScreen disclaimer.
(S2-4) Out of scope: cross-platform LuaJIT build deferred (Windows-only).

**Sub-sprint deferred**: Tauri `bundle.externalBin/resources` wiring is non-trivial because the vendored PoB tree is ~600 MB (`TreeData/` alone is 524 MB). Engine works in dev (`cargo test`, `cargo tauri dev`) via workspace-relative path resolution. Production installer integration becomes a Sprint 2.1 (figure out which PoB asset subset is load-bearing for headless calc, ship the trimmed bundle).

### Sprint Tooling — knowledge layer relocation + dev panel + battery CLI ✅

**Status**: ✅ done 2026-05-08.

**Notes**: Pause on the main Intelligence Roadmap to fix layout debt + ship a self-service dev/test stack. Knowledge layer moved to top-level `prompts/` (`SYSTEM_PROMPT.md` + `CORE_KNOWLEDGE.md` + `references/`), mirroring `~/.bestel/prompts/` exactly. New separate Tauri window `dev-panel` (mirrors `prompt-editor` pattern) with four tabs: Runs (migrated from old `/debug` route), Scenarios (loads `tests/scenarios/*.toml`), Real prompts (loads `docs/test_prompts/real_user_prompts.toml`, 31 entries, filterable by category), Live test (model picker + PoB fixture picker + free prompt + side-by-side rendered markdown / raw stream events / stats). New Settings section "Developer" with "Open dev panel" CTA. Global shortcut `Ctrl+Shift+D` opens the panel from any window. Old `/debug` route deleted. Scenario parser moved to `bestel-core::test_runner` so the CLI and dev panel share it. New CLI subcommand `bestel run-battery <dir> [--model <id>] [--out <dir>] [--filter-name <substr>] [--pob-fixture <name|path>]` runs scenarios in-process against live providers, persists `PersistedRun` JSON per scenario, exits non-zero on errors. Two known UX bugs (`wiki.La` auto-link from markdown-it linkify, narration-leak around tool calls) carry over to a future UX sprint — the dev panel makes both reproducible.

### Sprint 3 — PoB semantic-facts extractor ✅ (~1–2 weeks)

**Status**: ✅ done 2026-05-08, commit `ab003ce`. **Known limitation**: data is injected into the `get_active_build` payload but the Identity card surfacing is inconsistent across models — addressed by **Sprint D** (`build_context`) in Tier 2.

**Notes**: New module `crates/bestel-core/src/pob/semantic.rs` (~533 lines) exposing `BuildIdentity::from_build(b: &PobBuild) -> Self` with three axes: `defense` (life | ES | LL | CI | MoM | hybrid | RF | VLS), `hit_model` (crit | non-crit | non-crit-EO | DoT), `mechanic` (self-cast | trigger | totem | mine | trap | minion | autobomber). Static `DEFINING_UNIQUES: &[(&str, &str, &str)]` registry — name, category (`engine` | `defining` | `amplifier`), identity_hint. Conversion chain extractor with manual line parser. 5 unit tests against `poe1_inquisitor.xml` + `poe2_druid.xml` fixtures. Wired into `render_build_for_llm()` in `tools.rs::~879` so `get_active_build` JSON now carries `archetype`, `defining_uniques`, and `conversion_chain` summary fields. `prompts/CORE_KNOWLEDGE.md` updated with a "Build identity (read it, do not guess it)" section under §5; `SYSTEM_PROMPT.md` has the build identity card requirement at top.

### Sprint 4 — Wiki SQLite mirror ⏸ REFRAMED

**Status**: ⏸ deferred and reframed. Original scope (a SQLite cache mirror for poewiki/poe2wiki Cargo queries) is largely subsumed by Sprint E (LanceDB embedded RAG). Structured persistence for runs/sessions/builds is reframed as Sprint G2 in Tier 2.

### Sprint 5 — PoE2 0.5 absorption ⏸ REFRAMED

**Status**: ⏸ deferred and reframed as **Sprint H** in Tier 2. Re-positioned to run *after* Sprint E so it can leverage the RAG `applies_to=poe2` metadata filter for game-aware retrieval. Calendar target (2026-05-29) preserved; only ordering changes.

## Tier 2 — Reliability pivot sprints

### Quality findings from 2026-05-08 export

| Measure | Value | Sprint that addresses |
|---|---|---|
| Total runs | 67 | — |
| Distinct prompts | 22 | — |
| Total tool calls | 336 | — |
| Tool failures | 56 (16.7%) | C2 (engine) + B (DeepSeek bloat) |
| `pob_calc` failures | 19/29 | C2 |
| Runs with `<thinking>` leak | 5 | A |
| Build-loaded runs with conformant Identity card | ~1/24 | C, D |
| Panel sidecar correctly placed | 3/13 | A, B |
| Cited URL not actually fetched | not measured (qualitative present) | A, F |
| Multi-turn diagnosis drift | 1/8 multi-turn scenarios | G |

Nine recurrent failure modes:

1. **Reasoning leakage**: raw `<thinking>` tags emitted to the user.
2. **Missing Identity card**: build-loaded answers do not start with the prescribed `Identity:` line.
3. **Engine-trust violation**: after `pob_calc` fails, the model presents the cached PoB headline DPS as "real DPS".
4. **Citation fabrication**: a Sources URL appears that was never fetched in the same turn.
5. **Panel sidecar misplacement**: `⟦panel-data⟧` JSON appears mid-message instead of at the top.
6. **Mod-pool fabrication**: after `repoe_lookup` or `wiki_cargo` fails, the model invents an exact tier/ilvl table.
7. **Multi-turn drift**: a chosen bottleneck on turn 1 is silently replaced by an unrelated fix on turn 2.
8. **PoE1↔PoE2 contamination**: PoE1 mechanics (e.g., cluster jewels) cited for a PoE2 question.
9. **Internal docs presented as live source**: `prompts/references/*.md` cited as evidence for current patch state.

### Sprint A — Quality Gate + Baseline ☐ (1 week)

**Goal**: convert observed failure modes into deterministic runtime + battery assertions, and freeze a measurable baseline before any other change.

**Rationale (from studies)**:
- Audit 2026-05-08 chiffrage above: without a linter, RAG and skills cannot fix output-grammar bugs because they are not retrieval bugs.
- Deep research § Recommendations Sprint 1: *"Construire un eval set de 40 questions (10 par catégorie : mechanic, craft, build, mapping). LLM-as-judge avec Sonnet 4.5+. Baseline figée."* This is the only objective measure for the sprints that follow.
- Deep research § Caveats: *"the only way to be serious is to build your own eval set"*. Headline numbers like "90.2% Anthropic Research" do not transfer to our use case.

**Tasks**:
1. New module `crates/bestel-core/src/test_runner/response_lint.rs` with `enum FindingSeverity { Pass, Warn, Fail }` and `struct LintFinding { id, severity, message, evidence }`. Initial rule set:
   - `NO_THINKING_TAGS` (FAIL)
   - `NO_PROCESS_NARRATION` ("Let me fetch", "I'll search", "Je vais chercher" — WARN/FAIL)
   - `BUILD_IDENTITY_REQUIRED` (FAIL when `get_active_build` succeeded)
   - `POB_CALC_FAILURE_NO_REAL_NUMBER` (FAIL on "real DPS" / "actual DPS" / "true DPS" without cache disclaimer after `pob_calc` failed)
   - `PANEL_DATA_FIRST` (FAIL if `⟦panel:` appears anywhere and message does not start with `⟦panel-data⟧`)
   - `PANEL_MARKER_PAYLOAD_MATCH` (FAIL if marker has no matching sidecar key)
   - `SOURCES_FETCHED_ONLY` (FAIL on URL in Sources: not present in this run's tool fetches)
   - `NO_INTERNAL_DOC_AS_FINAL_SOURCE` (FAIL on `prompts/references/*` cited as live game evidence)
   - `NO_BLOCKED_SOURCE` (FAIL on Fandom/RMT/SEO domains)
   - `FAILED_RAW_DATA_NO_TABLE` (FAIL on exact tier/ilvl table after `repoe_lookup`/`wiki_cargo` failed without uncertainty marker)
   - `CALCS_ECHO_REQUIRED` (FAIL when `pob_calc` succeeded but `enemyIsBoss`/charges/flasks/active skill not echoed)
2. Extend `Expectation` (`crates/bestel-core/src/test_runner/scenario.rs:127`) with seven new specialised booleans: `must_have_identity_card_if_build`, `must_not_expose_reasoning`, `must_have_panel_data_first`, `must_cite_only_fetched_urls`, `must_surface_calcs_echo_if_pob_calc`, `must_not_claim_real_number_if_pob_calc_failed`, `must_not_fabricate_mod_table_after_raw_source_failure`.
3. Wire into `bestel run-battery` (`crates/bestel/src/main.rs::run_battery`) with an opt-in `--strict` flag. Default mode is *warn-only* during the A→H transition to avoid blocking CI before the underlying defects are fixed.
4. Dev panel: add a "Score / Findings" column to the Runs tab (`crates/bestel/ui/src/views/DevPanel/tabs/RunsTab.vue`).
5. **Baseline measurement** (deep research Sprint 1 §3): build a 40-question eval set (10 per category: mechanic, craft, build, mapping). LLM-as-judge with Sonnet 4.5+ via a prompt rubric. Persist the frozen baseline at `tests/eval/baseline-2026-05-XX.json`. Re-run after every subsequent sprint to measure delta. The baseline expands to 60 in Sprint H.

**New files**: `response_lint.rs`, `tests/eval/baseline-*.json`, `tests/eval/judge_prompt.md`.

**Exit criteria**:
- The 67 export runs replay through the linter and produce a deterministic findings report.
- Every `<thinking>`-leaking run FAILs.
- Every build-loaded answer without an Identity card FAILs.
- Baseline eval is frozen with category × model scores.
- `--strict` documented for future CI gating once Tier 2 is complete.

**Risks**: rules too strict invalidate good answers → the WARN/FAIL split mitigates. Warn-only default during transition prevents CI blockage.

**Duration**: ~6 days.

### Sprint B — Prompt cleanup + small-model discipline ☐ (3–5 days)

**Goal**: eliminate prompt contradictions, add an explicit answer-mode router, neutralise dangerous wording, and harden Haiku/DeepSeek behaviour.

**Rationale (from studies)**:
- `SYSTEM_PROMPT.md` (640 lines) contains an internal contradiction: it asks for "concise — three sentences beat ten" *and* mandates a 15–25 sentence pre-flight checkpoint for substantial answers. The export shows models writing essays even on short questions.
- The prompt currently includes wording resembling "run this in your reasoning/thinking block". Five export runs literally emitted `<thinking>...</thinking>`. This is a prompt bug.
- Deep research § Key Finding 6: *"DeepSeek-R1 — Anthropic-style long system prompts degrade performance. The model prefers concise instructions in the user prompt, no few-shot. DeepSeek-V3.x supports a system prompt but stays sensitive to bloat past Sonnet."*
- Deep research § 2(d) "Anti-laziness for small models": strict output schema (JSON contract) on tool results returning calculations; 2–3 canonical few-shots, *not* an exhaustive edge-case list; cap system prompt to 4–6k tokens for DeepSeek-V3.x.
- Anthropic *Effective Context Engineering*: *"the goal is not to be shorter, but to find the right altitude"* — explicit XML/Markdown structure beats dense prose.

**Tasks**:
1. Restructure `prompts/SYSTEM_PROMPT.md` into explicit sections: `<runtime_contract>`, `<persona>`, `<tool_policy>`, `<answer_mode_router>`, `<output_contracts>`, `<failure_policy>`. Strip every "thinking/reasoning block" mention; replace with "Run this silently before writing the user-facing answer. Never expose hidden checklists, scratchpad text, reasoning tags, or tool-planning narration."
2. Resolve concise-vs-essay contradiction with a 6-mode router:
   - **A — Brief mechanics**: 2–6 sentences.
   - **B — Build diagnosis**: Identity card + 2–5 paragraphs + numbers + one bottleneck + one fix.
   - **C — Craft/mod lookup**: identify base/ilvl/tiers from raw source; if source fails, no exact table.
   - **D — Entity deep-dive**: panel + 8–18 sentences.
   - **E — Patch/current/economy**: official source required this turn or do not claim current state.
   - **F — Off-topic refusal**: 1–2 sentences, in character.
3. New numbered references (slots 27–29 confirmed free):
   - `prompts/references/27_response_contracts.md` — the answer-mode router formalised.
   - `prompts/references/28_tool_failure_policy.md` — what to do after each tool fails (covered explicitly: `pob_calc`, `wiki_parse`, `repoe_lookup`, `wiki_cargo`, `web_fetch`, official patch notes).
   - `prompts/references/29_known_mechanics_tripwires.md` — short invariants that the export shows the model gets wrong (cluster jewels, Voices/Luminous Trove, trade pseudo-stats aggregation, eldritch implicits scope, conversion chain rules, drop levels).
4. `CORE_KNOWLEDGE.md` (206 lines): add §10 pointing to the three new references. Remove redundancy with `SYSTEM_PROMPT.md`.
5. Regenerate `BUNDLED_REFERENCES` array in `crates/bestel-core/src/prompts.rs`.
6. **Small-model discipline** (deep research § 2(d)):
   - Strict JSON output schema on tools returning calculations (`pob_calc`, `repoe_lookup`, `trade_resolve_stats`).
   - 2–3 canonical few-shots in `<output_contracts>`, not an exhaustive edge-case list.
   - DeepSeek-R1 (if added later): no long system prompt — user-prompt templates via `~/.bestel/prompts/models/deepseek-r1.md`. DeepSeek-V3.x: system prompt OK but capped at 4–6k tokens.
   - Document target token sizes per model in `27_response_contracts.md`.

**Exit criteria**: Sprint A linter passes ≥ 80% on a battery replay (with no code changes); tool-selection accuracy stable on Haiku/DeepSeek; estimated system prompt size < 4–6k tokens for DeepSeek-V3.x after cleanup.

**Risks**: invalidates the existing prompt cache (Sprint C is the right moment to rebuild it).

**Duration**: ~5 days.

### Sprint C — Cache segmentation + telemetry ☐ (3–5 days)

**Goal**: real cache hit rate > 80% on multi-turn sessions; dynamic content (build state, user message) injected *after* the last `cache_control` boundary.

**Rationale (from studies)**:
- Deep research § Key Finding 2: Anthropic prompt caching is under-utilised in most assistants. Cached read tokens at 0.1× of base price; TTFT down to ~15% of cold; the 1h TTL header `extended-cache-ttl-2025-04-11` covers desktop sessions with 2× cache-write multiplier paid once.
- Inventory of `crates/bestel-core/src/llm/anthropic.rs`: line 192 wraps the *full composed* system prompt in a single `cache_control: ephemeral` block (too coarse — any prompt edit invalidates everything). Lines 651–654 mark only the *last* tool schema with `cache_control`. We need to verify empirically that this still caches the full tools array, and rewrite if it does not.
- Deep research § Caveats: *"Changing any single byte in a cacheable block invalidates everything that follows. Design your blocks as stable units; put dynamic elements (current build, question) after the last `cache_control`. Likewise, changing the tools list breaks the system cache. Version your tool definitions."*
- Deep research § 2(b) prescribes four explicit cache breakpoints: tools, system block 1 (persona/runtime), system block 2 (tool policy / answer mode / output contracts / failure policy), system block 3 (CORE_KNOWLEDGE + skill descriptions).

**Tasks**:
1. Four explicit cache breakpoints:
   - **BP1 — Tools array**: `cache_control: ephemeral` covering the full schemas list. Verify empirically by counting `cache_read_input_tokens` before/after.
   - **BP2 — System block 1**: `<persona>` + `<runtime_contract>` — `cache_control: ephemeral`.
   - **BP3 — System block 2**: `<tool_policy>` + `<answer_mode_router>` + `<output_contracts>` + `<failure_policy>` — `cache_control: ephemeral`.
   - **BP4 — System block 3**: `CORE_KNOWLEDGE` + skill descriptions (~100 words × 3) — `cache_control: ephemeral`.
   - Dynamic: build state, session-notes summary, user message — *after* the last `cache_control`, never cached.
2. `crates/bestel-core/src/llm/anthropic.rs:192`: replace single-block `system` with the array conforming to BP2/BP3/BP4.
3. `crates/bestel-core/src/llm/anthropic.rs:651-654`: confirm or correct that `cache_control` on the last tool schema actually caches the entire tools list (Anthropic API contract: `cache_control` on the last element caches everything before it — confirm with a usage-counter test).
4. Enable header `extended-cache-ttl-2025-04-11` (1h TTL) for Haiku 4.5 + Sonnet 4.5/4.6.
5. **Tool-definition versioning**: any change to the tools list invalidates the system cache → introduce a `tools_version` constant for traceability and group tool changes into batched releases rather than ad-hoc additions.
6. Extend `ChatStats` (`crates/bestel-core/src/llm/recorder.rs:44-51`) with: `cache_read_input_tokens`, `cache_creation_input_tokens`, `output_tokens`, `estimated_cost_usd`, `cache_hit_ratio`. Persist into `PersistedRun`.
7. Dev panel: surface cache stats per run + aggregated per session.

**Exit criteria**: multi-turn Anthropic sessions show `cache_read > 0` from turn 2 onward; switching the active build does not invalidate the stable system cache; cache hit rate > 80% on multi-turn sessions (deep research Sprint 1 exit).

**Risks**: reordering blocks invalidates hot cache once — document in dev CHANGELOG. DeepSeek's implicit context cache is provider-specific; verify per provider, otherwise assume no-cache for DeepSeek.

**Duration**: ~5 days.

### Sprint C2 — `pob_calc` hardening ☐ (4–6 days, parallel to A/B/C)

**Goal**: drop `pob_calc` failure rate below 5%.

**Rationale (from studies)**:
- Audit 2026-05-08: 19/29 `pob_calc` failures, almost all `Classes/DropDownControl.lua:147: attempt to index a nil value`. This is the most impactful technical bug because `pob_calc` is the canonical source for calculated DPS/EHP/max-hit (`prompts/references/25_pob_engine_integration.md`).
- Worse: when `pob_calc` fails, some models (DeepSeek primarily) fall back confidently on the cached PoB `CombinedDPS` and present it as "real DPS". On the same build, models that succeeded in calling the engine report ~137k DPS while DeepSeek (engine failed, cache fallback) announces 14M DPS. Direct violation of the engine-trust rule in `26_validation_and_self_correction.md`.
- The fix has two halves: **technical** (DropDownControl + typed errors + sanitiser) here in C2; **behavioural** (linter `POB_CALC_FAILURE_NO_REAL_NUMBER`) in Sprint A. Both are necessary.

**Tasks**:
1. Reproduce `DropDownControl.lua:147` with the export fixtures that failed.
2. Log the exact ndJSON envelope sent to the sidecar: build game, skill index, category, calcs overrides, active skill group, dropdown selection.
3. Add a config sanitiser in the `pob_calc` handler before sending to the engine: reject unknown keys, normalise `enemyIsBoss` (`true`/`false` vs `"Pinnacle"`/`"Boss"`/`"None"`), refuse known incompatible combinations.
4. Single retry without non-essential overrides on first failure.
5. Typed error returned to the model:
   ```json
   {
     "error_kind": "pob_engine_sidecar_protocol",
     "recoverable": false,
     "message": "DropDownControl.lua:147 ...",
     "safe_fallback": "cache_only"
   }
   ```
6. Hook into Sprint A `POB_CALC_FAILURE_NO_REAL_NUMBER` lint.

**Exit criteria**: zero "real DPS" answers after engine failure on battery replay; `pob_calc` failure rate < 5%; 100% of successful DPS answers cite `active_skill_gem` + `TotalDPS`/`CombinedDPS` + calcs echo.

**Risks**: the Lua bug may require upstream PoB reverse-engineering — timebox to 2 days, otherwise log + structural fallback.

**Duration**: ~5 days.

### Sprint D — `build_context` tool ☐ (1 week)

**Goal**: eliminate the "model ignored available context" failure class and guarantee the Identity card server-side.

**Rationale (from studies)**:
- Deep research § 2(c): *"Documented LangGraph pattern: `tool_choice = {"type": "tool", "name": "build_context"}` on the first turn. The agent receives a rich payload at turn 1, before any streaming response, eliminating the 'mediocre answer despite available context' error class."*
- Audit 2026-05-08: ~1/24 build-loaded runs follow the Identity card contract, even though the data is already extracted by Sprint 3 (`BuildIdentity::from_build`). The model has the material but forgets to format it. Solution: compose the card *server-side* in the `build_context` payload, not as a model-formatting task.
- Anthropic *Effective Context Engineering* (Sept 2025) recommends "just-in-time retrieval" against context rot (Chroma Research July 2025, 18 LLMs: non-linear precision degradation with length, even on trivial tasks). `build_context` is the Bestel embodiment of this pattern.
- Cognition June 2025: *"share full context, prefer linear"* — `build_context` consolidates context *before* the model starts reasoning, instead of letting the model discover it piecewise via 4–5 sequential tool calls.

**Tasks**:
1. Add tool constant `BUILD_CONTEXT` in `crates/bestel-core/src/llm/tools.rs` (becomes tool #12).
2. Schema:
   ```
   build_context(question: string, mode: enum["auto", "build_diagnosis", "craft", "mechanic", "patch", "trade", "entity"], top_k: int = 5)
   ```
   Returns XML payload:
   ```
   <build_context>
     <game>...</game>
     <identity>Identity: defense=..., hit_model=..., mechanic=...</identity>
     <build_numbers>...</build_numbers>
     <active_skill>...</active_skill>
     <defining_uniques>...</defining_uniques>
     <conversion_chain>...</conversion_chain>
     <relevant_kb>... (top-N chunks from kb_search) ...</relevant_kb>
     <answer_contract>mode=..., required=identity_card,calcs_echo,sources</answer_contract>
   </build_context>
   ```
3. Server-side composition of the Identity card via `BuildIdentity::from_build` (Sprint 3 module).
4. Force first call: `tool_choice: {type: "tool", name: "build_context"}` in `crates/bestel-core/src/llm/anthropic.rs` when (a) build is loaded, *and* (b) request mode classifier (cheap regex + heuristics) does not flag off-topic / trivial-no-build.
5. Read session notes (Sprint G) at the start of `build_context` to preserve multi-turn diagnosis state.

**Exit criteria**: 100% of build-loaded scenarios start with `build_context`; Sprint A `BUILD_IDENTITY_REQUIRED` lint passes 100%.

**Risks**: forcing too broadly = latency cost on trivial questions → cheap router (regex + build-loaded heuristic) gates the force.

**Duration**: ~6 days.

### Sprint E — Hybrid embedded KB search ☐ (2–3 weeks)

**Goal**: replace the `read_internal_reference` enum with semantic + lexical retrieval.

**Rationale (from studies)**:
- Inventory: `read_internal_reference` (tool #9 at `crates/bestel-core/src/llm/tools.rs:19`) uses a strict JSON `enum` schema built from `BUNDLED_REFERENCES`, which prevents path hallucination but still requires the model to *guess* which file to open. On Haiku/DeepSeek this is the biggest blind spot: wrong file picked, or 3–4 sequential reads.
- Deep research § Key Finding 4: LanceDB is the dominant option for Tauri/Rust — Rust-native, in-process like SQLite, FTS via Tantivy integrated, hybrid search via `nearest_to() + text()` combination (Rust API), no dynamic loader, Lance file format versioned.
- **Why not sqlite-vec**: dynamic loader (`sqlite3_auto_extension()`) complicates Tauri builds (Windows MSI, macOS notarisation). No FTS5 + KNN in a single query without manual glue. Bench (Alex Garcia): 1536 dim @ 100k vectors → 105–214 ms/query, vs LanceDB brute-force < 50 ms at this scale (and IVF-PQ beyond). If we want a SQLite mirror later (Sprint G2), that is for *structured* data (recorder, builds) — vector stays on LanceDB. Both coexist.
- **Why not Qdrant / Chroma / Weaviate**: server process or Python dependency. Incompatible with a seamless Tauri binary.
- **Why not Tantivy alone (BM25-only)**: valid degraded option if zero embedding/inference is required. But losing semantic search on "how do Pantheon interact with duration buffs" vs "Pantheon + buff effect" costs more than the simplicity gain.
- **Why hybrid + RRF k=60**: LiveRAG 2025 + Snowflake finance RAG converge on RRF k=60 as robust to vector/sparse scale asymmetry; LanceDB default.
- **Why skip rerank in v1**: deep research § Key Finding 5: *"For ~60 Markdown docs, cross-encoder rerank is over-engineering. Benchmarks (LiveRAG 2025, OptyxStack, ZeroEntropy) show rerank gain +20–40% MAP only on corpora ≥ 100k chunks. On < 5k chunks with hybrid BM25 + vector + RRF, top-5 raw already MRR > 0.85. bge-reranker-v2-m3 cost ~130 ms / batch of 16 on CPU — visible user latency."* Add only if Sprint H eval reveals a plateau (Tier 3 trigger).
- **Why text-embedding-3-small + BGE/Ollama fallback**: $0.02/M tokens, 1536 dim, 8k context, top-tier price/quality. BGE-small-en-v1.5 or BGE-M3 via Ollama for offline-first / privacy. Voyage-3-large is ~3 pts MTEB stronger on code/tech but $0.18/M — reserved for Pro users if introduced.
- **Anti-pattern to avoid here**: re-embedding the entire knowledge layer at every boot. Index incrementally on file content-hash or boot latency dies.

**Tasks**:
1. New crate `crates/bestel-rag` (5th workspace member; add to root `Cargo.toml`).
2. Trait `KbIndex` abstraction (isolates LanceDB API to ease a future switch). Backend: LanceDB (`lancedb` crate, version pinned — official docs flag "expect breaking changes" mid-2025). FTS via Tantivy. Hybrid search with RRF k=60.
3. Markdown-header-aware chunker: target 400–800 tokens per chunk, 15% overlap. Metadata: `doc_path`, `heading_path: Vec<String>`, `tags`, `applies_to: [poe1|poe2]`, `last_updated`, `content_hash`, `source_kind`. Snowflake/Databricks 2025: +5–10 pts vs fixed-size on structured docs.
4. `EmbeddingProvider` trait: OpenAI `text-embedding-3-small` online (default), Ollama `bge-small-en-v1.5` or `bge-m3` (offline / privacy). Incremental via content-hash.
5. Index `prompts/references/**` at boot (incremental). Storage: `~/.bestel/index/kb.lance/`. Document explicit UX degradation if offline + no Ollama.
6. New tool `kb_search(query, game, top_k, tags)` (#13). `read_internal_reference` stays as fallback (description updated: "use only when a specific known file is required").
7. Wire into `build_context` (Sprint D): `relevant_kb` filled by `kb_search(question, top_k=5)`.
8. **Tool storm cap** (deep research § Triggers): limit `kb_search` to **3 calls per turn**. Beyond that, structural fallback: "I don't know in current knowledge base." Implement in `BuildContext::dispatch_tool`. Antipattern documented in Anthropic *Writing tools for agents*.

**Exit criteria**: top-5 ≥ 90% accuracy on 30 curated queries; zero fabricated citations on Sprint A linter over a 40-question replay; `read_internal_reference` calls drop ≥ 60%; Sprint A eval delta of +15–25 pts for Haiku/DeepSeek, +5–10 pts for Sonnet.

**Risks**: LanceDB breaking changes → `KbIndex` abstraction (R11). Online embeddings → Ollama fallback documented (R14). Tool storm → hard 3-call cap.

**Duration**: ~12–15 days.

### Sprint F — Skills / recipes ☐ (1–2 weeks)

**Goal**: extract repeatable workflows out of the permanent `SYSTEM_PROMPT.md` into progressive-disclosure skills.

**Rationale (from studies)**:
- Deep research § Key Finding 7: *"Skills/Recipes (Anthropic, December 2025) are the right extension primitive. Open format (`SKILL.md` + frontmatter + resources), adopted by OpenAI (Codex CLI/ChatGPT). This is deterministic context engineering (progressive disclosure: ~100-word metadata always visible, content loaded on demand), not multi-agent."*
- **Why not sub-agents**: deep research § 4 + § 7: *"Do not create thematic sub-agents (Bestel-craft, Bestel-mapper, Bestel-PoB). This is the antipattern Cognition describes: sub-agents that lose latent state, make implicitly incompatible decisions, and cost 15× more."* Bestel "modes" (build review, craft audit, mapping strategy) are **UX**, not architecture.
- The frontmatter `description` (~100 words) stays in the system prompt — the agent decides when to invoke. The body loads only when invoked → progressive disclosure. This is what makes Haiku/DeepSeek reliable without paying the cost of a dense permanent knowledge layer.
- Inventory: 11 tools today, below the 15–20 threshold where tool-selection accuracy drops < 80% (deep research § Key Finding 6). Adding ad-hoc tools per workflow would saturate. Skills add only one tool (`load_skill`).
- Caveat (R15): the spec is recent (December 2025). Implement the pattern (folders + `SKILL.md` + progressive disclosure) without the official SDK to stay portable.

**Tasks**:
1. New folder `prompts/skills/` (bundled via the same `BUNDLED_REFERENCES` pattern; seeded to `~/.bestel/skills/` for user override).
2. Layout per skill (deep research § 4):
   ```
   prompts/skills/
     build-review/
       SKILL.md          # frontmatter + when-to-use
       templates/        # parameterised prompts (quick_audit.md, full_audit.md)
       scripts/          # deterministic Rust binaries called via tool, optional
     craft-audit/
       SKILL.md
       templates/
     mapping-strategy/
       SKILL.md
   ```
3. Three initial skills: `build-review/`, `craft-audit/`, `mapping-strategy/`. Future candidates (deferred): `price-check/`, `poe2-build-review/`.
4. New tool `load_skill(name)` (#14, reaches 14/20 — still below the threshold). Cap: 1 skill per turn by default, 2 for full audit.
5. Inject `description` (~100 words × 3 = ~300 tokens) into the cacheable BP4 system block (Sprint C). No increase in dynamic tokens.
6. Migrate verbose instructions from `SYSTEM_PROMPT.md` (extended build review, craft audit, mapping checklist) into the corresponding skills. The system prompt must shrink.
7. Implement `crates/bestel-core/src/skills.rs`: parse `SKILL.md` frontmatter, expose `list_skills()` and `load_skill_body(name)`. Bundle via `include_str!` like prompts.

**Exit criteria**: `SYSTEM_PROMPT.md` shrinks ≥ 20% in line count; build-diagnosis runs invoke `build-review` 100% of the time; tool-selection accuracy stable on Haiku/DeepSeek despite adding tool #14.

**Risks**: skills not used because frontmatter too vague → test per model. "1 skill / turn" cap prevents skill-load thrash.

**Duration**: ~8 days.

### Sprint G — Conditional verifier + session notes ☐ (1 week)

**Goal**: catch unsourced numerical/named claims without paying verification latency on every turn; stabilise multi-turn diagnosis.

**Rationale (from studies)**:
- Deep research § 5: *"No general Reflexion (latency × 2–3, low ROI on conversation). Conditional validation: trigger only when output contains numbers (DPS, EHP, prices), claims about named uniques, or skill references. The validator is a second pass with the same agent, not a sub-agent."*
- *"Plan-and-Execute is over-engineered for a conversational assistant; ReAct (the current implicit pattern) is correct as soon as retrieval is good. Plan-and-Execute makes sense only for an 8+ step audit, encapsulated as a Skill."* Consistent with Sprint F.
- Audit 2026-05-08 multi-turn case: user asks "weakest defensive layer → cheapest gear-only fix → item base/ilvl". Turn 1 = correct diagnosis (physical mitigation). Turn 2 = pivots to belt elemental res, then to Granite Flask. Turn 3 = Bestel asks the user which mod they meant. Multi-turn drift: turn-1 diagnosis not preserved across turns.
- Anthropic *Effective Context Engineering* September 2025: three patterns against context rot — **compaction**, **structured note-taking**, **just-in-time retrieval**. `build_context` (Sprint D) addresses just-in-time. The verifier + session notes here address structured note-taking.
- Anti-pattern to avoid: *"Vectorising user conversation for long-term memory. Bestel is session-scoped (PoB analysis, conversation about a build) — a Markdown notes file per session (`~/.bestel/notes/<build_id>.md`) suffices, exactly the Anthropic 'structured note-taking' pattern."*

**Tasks**:
1. New module `crates/bestel-core/src/llm/verifier.rs`. Trigger only if output contains: game/build numbers, DPS/EHP/max-hit, item/mod/craft tiers, patch claims, prices, named uniques, or a Sources: section.
2. Verifier = same model, strict JSON-only prompt: `{status: pass|revise|fail, findings: [...], minimal_rewrite: ...}`.
3. Wire pre-emit in the streaming pump (`crates/bestel/src/events.rs::pump_deltas`).
4. Session notes: `~/.bestel/notes/<session_id>.json` structured (`current_diagnosis: {primary_bottleneck, evidence, chosen_fix, constraints}`). Read at the start of `build_context` (Sprint D).
5. Persist verifier output in `PersistedRun` + dev panel display.

**Exit criteria**: zero "real DPS" after engine failure; multi-turn "single cheapest fix" scenarios remain coherent over 3 turns.

**Risks**: verifier too sensitive blocks good answers → `revise` (minor rewrite) vs `fail` (refusal) tiering.

**Duration**: ~6 days.

### Sprint G2 — Structured persistence (formerly Sprint 4 reframed) ✅ (~1 day)

**Goal**: SQLite mirror for structured data (sessions, builds, recorder, lint findings, verifier results, kb_versions). Coexists with LanceDB.

**Rationale (from studies)**:
- Deep research § Sprint 4: *"SQLite mirror for structured data (recorder, sessions, build snapshots) via `rusqlite` or `sqlx`. Do not touch LanceDB here — both backends coexist. Migrate `~/.bestel/*.json` flat files to SQLite with schema versioning. Add a `kb_versions` table tracking the hash of each indexed doc."*
- Inventory: `PersistedRun` (`crates/bestel-core/src/llm/recorder.rs:54-72`) is currently flat JSON under `~/.bestel/runtime/debug-chats/<id>.json`. With 67+ runs already accumulated and battery generating hundreds, querying "all Haiku runs that FAILed Identity card" is impossible without an index.
- Why reframed vs original Sprint 4: original was a wiki SQLite cache. Sprint E's LanceDB hybrid + content-hash incremental subsumes that as a special case of the RAG cache. Reframing G2 toward structured persistence of runs/sessions/lints/verifier output is more useful: it powers the dev panel and Sprint H eval.

**Tasks**:
1. Schema versioned via `rusqlite` or `sqlx`.
2. Tables: `sessions`, `runs`, `builds_snapshot`, `kb_versions` (hash per indexed doc), `lint_findings`, `verifier_results`.
3. Migrate `~/.bestel/runtime/debug-chats/*.json` flat files → SQLite. Idempotent script + backup.
4. Keep JSON export for dev-panel compatibility (Sprint A linter input).

**Exit criteria**: query "all Haiku runs that FAIL Identity card" responds < 100 ms over 1000+ runs.

**Risks**: migration breaks existing runs → idempotent script + backup before migration.

**Duration**: ~10 days.

### Sprint H — Eval set + PoE2 0.5 (formerly Sprint 5 reframed) ✅ (code-level; eval-run + 0.5 day-of pending)

**Goal**: 60 strict scenarios + PoE2 0.5 absorption with `applies_to=poe2` filter on the RAG layer.

**Rationale (from studies)**:
- Deep research § Sprint 5: *"Re-index with the `applies_to: poe2` filter separated. The hybrid RAG now allows answering 'this rule changed in PoE2' cleanly without contamination."*
- Inventory: `prompts/references/poe2/` (6 docs) and `00_version_pinning.md`. PoE2 is volatile Early Access. The audit shows PoE1↔PoE2 contamination (e.g., citing cluster jewels for a PoE2 question).
- *"Conditional rerank decision: if between Sprint 3 and Sprint 5 you have plateaued at < 90% precision on the eval, integrate `bge-reranker-v2-m3` via `ort` (ONNX Runtime Rust) or Candle. Otherwise, skip."* This is where we decide.
- *"Skip rerank is not final. If you eventually accumulate > 200 docs, dense homonyms (two skills with similar names), or chunks where BM25+vector score nearly equally, a cross-encoder (bge-reranker-v2-m3, ~130 ms CPU) becomes justifiable. The retest must be metric-driven, not instinct."*
- Inventory: `tests/scenarios/` = 30 TOML (20 single-prompt + 10 multi-turn battery). Sprint H target = 60 strict scenarios with `Expectation` extended in Sprint A. Source: convert `docs/test_prompts/real_user_prompts.toml` (30 prompts categorised vague / beginner / mechanics / diagnosis / build_choice / external_link / crafting / currency_chase / boss / poe2 / comparison_lore / off_topic_refusal).

**Tasks**:
1. Convert 30+ prompts from `docs/test_prompts/real_user_prompts.toml` into strict scenarios in `tests/scenarios/` with the Sprint A extended `Expectation`. Target: 60 (~8 per category).
2. Re-index `prompts/references/poe2/**` with `applies_to=poe2` separated.
3. Tune RRF if a vector/BM25 imbalance shows up in eval (default k=60 rarely needs tuning).
4. **Conditional rerank decision**: if plateau < 90% accuracy after Sprint H → integrate `bge-reranker-v2-m3` via `ort`/Candle. Otherwise skip (Tier 3).
5. Update `prompts/references/poe2/00_version_pinning.md` with the current 0.5 patch version.
6. PoE2 day-of work (formerly Sprint 5): re-pull `/api/trade2/data/stats` + `/data/static`; force-refresh repoe-fork PoE2 snapshot; fill `references/poe2/05_atlas_mechanics_05.md` (Atlas tree rework, Verisium, Runic Ward, Alloys); fill `references/poe2/06_runes_of_aldur.md` (league mechanic); update `references/24_patch_history_meta.md`; verify PoB2 engine still runs against new XML schema; possibly bump submodule pin.

**Exit criteria**: eval set ≥ 90% pass rate Sonnet 4.5, ≥ 75% Haiku 4.5, ≥ 65% DeepSeek V3.x; zero PoE1↔PoE2 contamination on 8 PoE2 scenarios.

**Risks**: LLM-as-judge bias → cross-judge with Opus and Sonnet, keep strict regex assertions.

**Duration**: ~10 days.

## Expected gains — qualitative

> Caveat (deep research § Caveats): all gain estimates are qualitative. Headline benchmark numbers (90.2% Anthropic Research, 92% Plan-and-Execute) come from non-comparable workloads. The Sprint A baseline + Sprint H eval set are the only numbers that count.

| Model | Current state (audit 2026-05-08) | After A+B+C (caching + prompt clean + linter) | After D+E (build_context + RAG) | After F+G+H (skills + verifier + eval) |
|---|---|---|---|---|
| **Claude Haiku 4.5** | Correct but forgets / fabricates. Identity card missed. Citations imagined. | +10–15 pts quality, visibly better latency (cache hit), cost −60–80% | ~85–90% of current Sonnet 4 quality (deep research § 6) | ~90% Sonnet quality, robust on domain rules |
| **DeepSeek V3.x** | Average, ignores some rules, dangerous fallback after engine failure | Significantly better instruction-following thanks to restructured prompt | RAG replaces fragile instruction-following with hard context | Plateau reached; still below Haiku on long coherence. **Best effort** target, not optimised PPCM. |
| **Claude Sonnet 4.5/4.6** | Very good | −50–70% cost via cache, baseline preserved | Excellent | Domain state of the art for Bestel |
| **Claude Opus 4.x** (deep review only) | Excellent | Excellent; latency less critical (low usage) | Excellent | Marginally better than Sonnet for Bestel; reserved for "deep review" sessions |

## Triggers that change the plan

| Trigger | Action | Why |
|---|---|---|
| Sprint A baseline (post-A+B+C) already +20 pts on Haiku | Sprint E (RAG) stays priority | Caching does not fix citation fabrication — only RAG does. |
| Sprint E does not produce +15 pts on Haiku | Before any other change: revisit (a) chunk size (probably too large), (b) embedding (try voyage-3-large on a sample) | If retrieval fails, skills/verifier cannot compensate. |
| > 20% of sessions exceed 30k tokens in context | Add Anthropic-style compaction (auto-summary + restart with `NOTES.md`). Not before. | Context rot — Chroma Research July 2025. |
| Tool storms observed (5+ `kb_search` consecutive without convergence) | Hard cap 3 retrieval calls per turn + "I don't know in current knowledge base" fallback | Anthropic *Writing tools for agents* "retrieval thrash" antipattern. Implemented in Sprint E task 8. |
| Sprint H eval plateau < 90% | Conditional rerank: integrate `bge-reranker-v2-m3` via `ort` or Candle | Otherwise skip (Tier 3). |
| Corpus exceeds 200 docs OR dense homonyms appear | Rerank becomes justifiable | < 5k chunks suffice with hybrid + RRF. |
| Anthropic publishes a new hybrid skills + sub-agent pattern | Re-evaluate D7 in Tier 3 | The multi-agent debate is recent (June 2025). The Cognition principle "share full context, prefer linear" remains structurally robust. |

## Tier 3 — Deferred / triggered-only / never

> These items are explicitly DEFERRED or NEVER. Documenting them here protects against future temptation to add them "to be safe". Sources: deep research § 7 (Anti-patterns) + § Triggers + § Caveats + 2026-05-08 audit.

| Item | Trigger to activate | Justification |
|---|---|---|
| **Cross-encoder reranker** (`bge-reranker-v2-m3`) | Plateau < 90% accuracy after Sprint E + H, OR corpus > 200 docs, OR dense homonyms (skills with near-identical names), OR chunks where BM25 + vector score near-identically | Deep research § Key Finding 5: *"On < 5k chunks with hybrid + RRF, top-5 raw already MRR > 0.85. ~130 ms CPU = visible user latency."* Metric-driven, not instinct. |
| **GraphRAG** | Never except explicit user demand | Enterprise scale only (Pinecone/Snowflake 2025). False friend on < 10k chunks. |
| **Multi-vector ColBERT** | Never before 50k+ chunks | Same as GraphRAG. Bestel has ~60 docs. |
| **Permanent thematic sub-agents** ("Bestel-craft", "Bestel-mapper") | **Never** (Cognition antipattern, June 2025) | *"Sub-agents that lose latent state, make incompatible implicit decisions, and cost 15× more."* If considering this: it is UX, not architecture → skill router in frontend. Covered by Sprint F. |
| **Global Plan-and-Execute** | Never except for 8+ step workflow encapsulated as a Skill | Deep research § 5: *"Latency × 2–3, low ROI on conversation. ReAct (the current implicit pattern) is correct once retrieval is good."* |
| **Vectorising user conversation** for long-term memory | Never (session-scoped suffices) | Deep research § 7: *"Bestel is session-scoped. A Markdown notes file per session (`~/.bestel/notes/<build_id>.md`) suffices — exactly the Anthropic 'structured note-taking' pattern."* Covered by Sprint G. |
| **Re-injecting the full transcript into sub-prompts** "to share context" | Never | Cognition June 2025 explicit: *"a clumsy palliative that ends up exploding cost and still losing latent state."* Correct pattern: `build_context` (Sprint D) + session notes (Sprint G). |
| **Re-embedding the knowledge layer at every boot** | Never | Incremental indexing on file content-hash, otherwise boot latency. Covered Sprint E task 4. |
| **LLM-as-reranker** (Anthropic/GPT as reranker) | Never | Cost × N turns. Local cross-encoder (ONNX Runtime) if absolutely required some day. |
| **Auto compaction / context summary** | > 20% of sessions exceed 30k tokens (deep research § Triggers) | Not yet observed in Bestel. If/when: Anthropic-style compaction with restart on `NOTES.md`. |
| **Counterfactual engine-vs-filler XML round-trip** | If Sprint H eval reveals need | Sprint 3.5 stretch. PoB engine has no remove-item API → high dev cost for uncertain ROI. |
| **Tree-dict keystone by node_id** | If Sprint H eval reveals need | Sprint 3.5 stretch. Partially covered by Sprint 3 archetype tagger. |
| **Qwen3:8b tool loop fix** | If Qwen becomes a focus provider | Out of scope for current focus models (Haiku 4.5, Sonnet 4.5/4.6, DeepSeek V3.x). |
| **macOS / Linux build** | When user asks (cf `docs/TODO.md`) | Windows-first. Memory rule: subprocess spawns must `CREATE_NO_WINDOW` (already implemented). |
| **Voyage-3-large embeddings** | Pro users with code/tech retrieval need | $0.18/M vs $0.02/M for `text-embedding-3-small`. ~3 pts MTEB on code, marginal on PoE Markdown. |

## Risks

1. **Bundle size** — Sprint 1 ~10 MB, Sprint 2 ~80 MB.
2. **CI complexity** — multi-target LuaJIT build, weekly PoB submodule pumps, snapshot refresh job.
3. **Antivirus false positives** — unsigned `luajit.exe`. Mitigate with code-signing.
4. **PoB patch lag** — 48 h SLA for submodule bumps.
5. **PoE2 0.5 datamining lag** — wikis lag 1–3 days; prefer repoe-fork for first 72 h.
6. **Prompt drift from KB expansion** — `read_internal_reference` is fetch-on-demand; `LOCAL_TOOL_ALLOWLIST` keeps Ollama on a curated subset.
7. **SEO blog ban false positives** — per-domain blocklist, easy to override.
8. **PoB2 engine completeness** — `pob_calc` returns `warnings` array.
9. **License compliance** — PoB MIT, PoB2 MIT, LuaJIT MIT. Aggregated in `LICENSE-third-party.md`.
10. **GGG ToS** — no scraping of undocumented endpoints.
11. **LanceDB API breaking changes** — official docs flag "expect breaking changes" mid-2025. Mitigation: `KbIndex` abstraction isolates the LanceDB API to ease a future switch (qdrant embedded or sqlite-vec mature).
12. **Anthropic cache invalidation per byte** — *"changing any byte in a cacheable block invalidates everything that follows. Design blocks as stable units; put dynamic elements (current build, question) AFTER the last `cache_control`. Likewise: changing the tools list breaks the system cache. Version your tool definitions."*
13. **DeepSeek prompt sensitivity** — R1 and V3.x have distinct prompting behaviours; Flash favours concision. Document which variant Bestel targets and adapt templates. If DeepSeek usage stays minor, do not optimise for the lowest common denominator — Anthropic-first, DeepSeek best-effort.
14. **Online embeddings = network dependency** at boot and indexing. BGE local via Ollama is a must as fallback for "works offline" desktop posture. Document explicit UX degradation if user is offline and lacks Ollama.
15. **Skills format spec is recent** (December 2025). Adoption outside the Claude ecosystem is partial. Implement the pattern (folders + `SKILL.md` + progressive disclosure) without the official SDK; do not adopt the SDK except for clear benefit.

## Out of scope

- ❌ OAuth integration (private stash, private characters).
- ❌ Public stash tab stream consumer.
- ❌ Awakened-PoE-Trade `apt-data` integration.
- ❌ poe.ninja API integration (dedicated tool deferred).
- ❌ Live markdown preview pane in prompt-editor.
- ❌ PoE1 3.X league absorption sprint.
- ❌ Vector index over wiki prose (LanceDB indexes the *internal* knowledge layer, not the wiki).
- ❌ Build comparison ("which of my 3 PoBs is best for Maven?").
- ❌ Permanent multi-agent (D7).
- ❌ Vectorising user conversation.
- ❌ Premature reranker.
- ❌ "Broad research" parallel mode.
- ❌ Full-transcript re-injection into sub-prompts.
- ❌ Re-embedding KB at every boot.
- ❌ LLM-as-reranker.
- ❌ Auto-compaction before > 20% sessions exceed 30k tokens trigger.

## Status

> ☐ pending · 🟡 in progress · ✅ done · ⏸ deferred · 🔁 triggered-only

### Tier 1
- ✅ **Sprint 0** — Knowledge base + validation hardening (2026-05-08)
- ✅ **Sprint 1** — Data layer (repoe + trade-stats) (2026-05-08)
- ✅ **Sprint 2** — PoB engine headless (2026-05-08, known DropDownControl bug → Sprint C2)
- ✅ **Sprint Tooling** — knowledge layer relocation + dev panel + battery CLI (2026-05-08)
- ✅ **Sprint 3** — PoB semantic-facts extractor (2026-05-08, commit `ab003ce`)
- ⏸ **Sprint 4** — Wiki SQLite mirror — REFRAMED into Sprint G2
- ⏸ **Sprint 5** — PoE2 0.5 absorption — REFRAMED into Sprint H (target 2026-05-29 preserved)

### Tier 2 (reliability pivot)
- ☐ **Sprint A** — Quality Gate + Baseline
- ☐ **Sprint B** — Prompt cleanup + small-model discipline
- ☐ **Sprint C** — Cache segmentation + telemetry
- ☐ **Sprint C2** — `pob_calc` hardening (parallel)
- ☐ **Sprint D** — `build_context` tool
- ☐ **Sprint E** — Hybrid embedded KB search (LanceDB)
- ☐ **Sprint F** — Skills / recipes
- ☐ **Sprint G** — Conditional verifier + session notes
- ☐ **Sprint G2** — Structured persistence (SQLite mirror, formerly Sprint 4)
- ☐ **Sprint H** — Eval set + PoE2 0.5 (formerly Sprint 5)

## Unresolved questions

- (S1-1) ✅ Resolved 2026-05-08. No `version.txt` on `repoe-fork.github.io`; refresh task uses plain GET.
- (S1-2) ✅ Resolved 2026-05-08. PoE2 currently exposes 3/8 categories on repoe-fork.
- (S2-1) Resolved: PoE2 PoB fork pinned at v2.49.3.
- (S2-2) Resolved: ianderse `set_config` covered only `bandit`/`pantheon`/`enemyLevel`; we forked.
- (S2-3) Deferred: Windows EV cert for code-signing — ships unsigned with SmartScreen disclaimer.
- (S2-4) Out of scope: cross-platform LuaJIT — Windows-only this sprint.
- (E-1) Online vs offline default for embeddings: `text-embedding-3-small` (online) by default with Ollama BGE fallback, or invert to honour "Bestel works without internet"?
- (E-2) Skills bundling strategy: bundle via `include_str!` (like prompts) AND seed to `~/.bestel/skills/` for user override — confirm parity with prompts.
- (E-3) LLM-as-judge for the eval set: Sonnet 4.5 (consistent with Bestel's providers) vs Opus 4.x (better judge but outside the current stack), vs cross-validation across two models?
- (A-1) `--strict` flag on `bestel run-battery`: keep warn-only as default during the A→H transition, switch to fail-on-critical only at end of Sprint H?
