# Brainstorm — Build Sheets ("Bestel learns the build")

> **Status:** in-progress (Phase 1 UI scaffolding + Phase 2 backend foundations shipped 2026-05-09).
> **Date parked:** 2026-05-09
> **Authored from:** UX-1 follow-up conversation, after ~$0.74 / 658k-token chat exposed the unsustainable per-turn cost of replaying full PoB JSON.

## TL;DR

Today every chat replays the full PoB import (28 items × 26 gems → ~30 KB of structured JSON, plus tool outputs that re-cite item names) on every single turn. This pushes input tokens past 600 K for routine sessions, costs ~€0.70 per échange on Sonnet, and still leaves the model derived re-deriving build context from scratch each time.

**Proposal**: build a one-time, validated "Build Sheet" — Bestel and the user co-author a curated dossier of the player's build, save it locally, and reuse it as the canonical context in every future chat about that build. The full PoB stays attached as fallback; the sheet becomes the primary context surface.

Outcome target:
- **Cost**: ~€0.70 → ~€0.10 per échange after the sheet exists (~85% reduction)
- **Accuracy**: model stops re-guessing scaling tags, defining items, defensive layers — it reads the sheet's stipulations
- **Trust**: the user has explicitly validated the sheet, so disagreements about "Bestel misread my build" become much rarer

## Why now

Sprint UX-1 (see `plans/wondrous-puzzling-forest.md`) shipped the chat-surface polish layer. The cost ceiling is now the dominant blocker to shipping to first testers. Even with truncation tricks (Sprint UX-2 candidate: tool-output truncation, tool-storm cap, Haiku-routing), the long pole remains: **the PoB itself is heavy and replayed verbatim**. A curated sheet sidesteps that long pole rather than incrementally chipping at it.

## Naming

Working name throughout this doc: **Build Sheet**. Alternatives considered:

| Name | Verdict |
|---|---|
| Build Sheet | Clear, neutral, scannable. **Pick.** |
| Build Dossier | Evocative, matches the chronicler vibe. Second choice. |
| Build Profile | Sounds like settings; reject. |
| Build Card | Confuses with Identity Card (already a thing). Reject. |
| Build Brief / Notebook | Unspecific. Reject. |

The user-facing string can still bend toward "dossier" if design wants warmth. Code identifier: `BuildSheet`.

## Problem (current state)

1. **Cost**: ~600 K input tokens for a multi-tool session because the full PoB is re-attached on every turn AND tool outputs (wiki pages ~30 K each, repeated 5-6 times in history) compound it. `crates/bestel-core/src/llm/anthropic.rs` does standard caching but the PoB sits inside a frequently-modified prompt block that doesn't cache as cleanly as a static system prompt.
2. **Accuracy drift**: the model re-derives "is this a crit build?", "what's the main scaling tag?", "is `Wand Ascendant` the engine here?" from raw item lines every turn. It often makes the right call but occasionally gets it wrong on edge cases (hybrid builds, conversion chains, niche uniques).
3. **No memory**: even when the model nailed the build's identity in turn 1, turn 7 still has to re-derive it from raw PoB. There is no place to write down "we already decided X about this build."
4. **Cold-start friction**: a fresh chat starts from zero. The user effectively re-onboards Bestel every time.

## Vision / north star

Bestel should treat each player's build as a **persistent character in its world** — known once, remembered forever (until the build changes). The first conversation about a build is a brief, structured **interview**: Bestel asks targeted questions, drafts a sheet section by section, the user validates or corrects, the sheet is saved. Every subsequent chat about that build loads the sheet (compact, ~2-4 K tokens) instead of the full PoB.

The interview is itself a feature, not a tax: the user gets a chance to confirm Bestel "understands" their build before stakes-rising questions ("should I drop Mageblood?"). Friction is converted into trust.

## UX paths considered

### Path A — Two distinct chat types

Separate "Analyzer" chat from "Generic" chat. Analyzer chat is a special mode whose only purpose is to produce a sheet.

- Pros: clear mental model; the analyzer has its own UI affordances (progress bar, section preview, validate button).
- Cons: forces the user to context-switch; awkward when a quick question kicks off the realization that no sheet exists yet; introduces a "chat type" concept that complicates the chat picker.

### Path B — Single chat, in-line discovery (RECOMMENDED)

One chat type. When the user attaches a PoB and asks anything substantive, Bestel detects the absence of a sheet for this PoB and pivots:

> *"Before I answer that, give me a few minutes to make sure I understand your build correctly — it'll save us churn now and skip this step in every future conversation. Three quick questions: …"*

The chat continues normally; Bestel asks questions, user validates, a "Save Build Sheet" affordance appears in the gutter when Bestel is satisfied. Once saved, Bestel pivots back to the user's original question and answers it, now using the sheet.

- Pros: zero context-switch; the interview is a natural prelude to the answer; chat history remains a single timeline; the user can interrupt and accept a "best-guess" answer if they're in a hurry.
- Cons: blurs "Bestel is interviewing me" with "Bestel is answering me" — needs a clear visual treatment for interview turns; risks frustrating users who want a one-shot answer ("just tell me, I don't want to be quizzed").

### Path C — Auto-generated sheet, user-validated post-hoc

Bestel silently drafts a sheet from the PoB on first attach (no questions asked), then surfaces it for review next to the chat. User can accept-as-is, edit, or discard.

- Pros: zero friction in the chat itself; gives the user something concrete to react to.
- Cons: the auto-sheet's quality is exactly the thing we don't trust today (model makes mistakes reading raw PoB); without the interview, the user might rubber-stamp a wrong sheet; loses the "Bestel asks, you answer" trust loop.

### Recommendation

**Path B is the lead candidate**, with two affordances borrowed from C:

1. **Skip-and-answer escape hatch**: a small "Just answer my question — best guess, no sheet" button visible during the interview. If clicked, Bestel proceeds without a sheet, current behavior preserved.
2. **Auto-draft as a starting point**: Bestel does generate a draft sheet from the raw PoB before asking questions. The interview is then a *correction loop* on the draft, not a from-zero quiz. Each question is "I think X, am I right?" — much less tedious than "tell me what your defense layers are."

## What goes on a sheet

A sheet is a compact, structured artifact (~2-4 K tokens). It is **not** a copy of the PoB; it is the *interpretation* of the PoB plus user intent.

Proposed schema (loose; will be refined during implementation):

```yaml
# build_sheet@v1
identity:
  defense: [armour, suppression]
  hit_model: spell crit
  mechanic: [conversion, ailment-immune]

archetype: "Spell Caster — Crit Cold Conversion"
ascendancy: "Inquisitor"
skill: "Ice Nova of Frostbolts"

damage:
  scaling_priority: [spell damage, cold damage, crit multi, projectile]
  conversion: "Cold → 0% (no further conversion)"
  key_multipliers:
    - "Inquisitor +crit on consecrated ground (~70% uptime)"
    - "Frostbolt projectiles → Ice Nova radius scaling"
  notes: "Scales mostly off gem levels and clusters; rare item slots are flexible."

defense:
  primary: armour (base ~24K, with flask ~38K)
  secondary: spell suppression (capped at 100%)
  ailment: "elemental ailment immunity via Brass Dome"
  weak_layer: "low life pool (~4.2K, no Mind Over Matter); vulnerable to chaos burst"

defining_items:
  - { name: "Brass Dome", role: defining, why: "AoE suppression + crit denial" }
  - { name: "Cospri's Will", role: amplifier, why: "ailment delivery without crit chance investment" }

gem_chains:
  main:
    - Ice Nova of Frostbolts — 6L
    - links: [Frostbolt, Hypothermia, Bonechill, Inspiration, Awakened Spell Cascade]
    - purpose: "primary damage; cascade triples projectile count for clear"
  utility:
    - Vortex on Cast — 4L  → "ground freeze + chill on bossing"

intent:
  goals: ["pinnacle bossing", "T17 farming"]
  constraints: ["budget ≤ 50 div", "no Mageblood"]
  rejected_pivots: ["Aegis Aurora swap suggested by streamer X — user said no, prefers offense"]

known_gaps:
  - "no chaos res investment yet (currently -23%)"
  - "movement speed at 30%, low for blight"

provenance:
  pob_fingerprint: "ice-nova-of-frostbolts/inquisitor/v3-28/cospri-brass"
  pob_hash: "sha256:abc123..."
  authored_at: "2026-05-09T14:22:00Z"
  authored_in_chat: "chat-20260509-…"
  validated_by_user: true
```

### Why this shape

- **`identity`** mirrors the existing Identity card grammar — zero re-design, the lint already enforces it.
- **`damage` / `defense`** are short prose stanzas, not exhaustive tables. The model needs to *infer*, not parrot. Keeping them prose lets the user edit naturally.
- **`intent`** is the highest-leverage section — it captures things that simply do not exist in the PoB (goals, budget, rejected pivots). This alone justifies the feature.
- **`known_gaps`** lets the user pre-flag things they already know are weak, so Bestel doesn't waste turns "discovering" them.
- **`provenance`** anchors the sheet to a specific PoB version, enabling drift detection.

## Sheet ↔ PoB binding

The hard question. Two extremes:

- **Strict hash binding**: tiny PoB tweak invalidates the sheet → constant re-interviewing. Not viable.
- **Loose user-managed binding**: user picks "use sheet X for this chat" → too manual, kills the magic.

**Proposed hybrid**: store *both* a strict hash AND a loose fingerprint.

- **Fingerprint** (loose, deterministic from PoB content):
  - ascendancy + main skill (auto-detected) + sorted list of "defining" uniques (top ~3-5 items by rarity score)
  - Format example: `inquisitor/ice-nova-of-frostbolts/cospri-brass-doryani`
  - Used for **auto-detection on attach**: "I have a sheet that looks like this build."
- **Hash** (strict, sha256 of canonical PoB JSON):
  - Used for **drift detection**: "I have a sheet for an earlier version — your gear changed since then. Want me to update it?"

On PoB attach in a fresh chat:
1. Compute fingerprint + hash.
2. Look up sheets table: `SELECT * FROM build_sheets WHERE fingerprint = ?`
3. If match found:
   - Hash equal → silently load sheet, proceed.
   - Hash differs → diff prompt: "Your build changed. {summary of diff}. Use existing sheet anyway, or refresh?"
4. If no match → no sheet, behavior falls back to current (full PoB, possibly + interview offer).

### Versioning

Keep a flat history per fingerprint: `build_sheets` rows with `version` column auto-incrementing per fingerprint. Latest version is "active"; older versions retained for "look at how my build evolved" use cases (low priority for v1).

Probably overkill for v1. Defer until users ask.

## Storage

New SQLite table (existing infra in `crates/bestel-core/src/db/`):

```sql
CREATE TABLE build_sheets (
  id TEXT PRIMARY KEY,
  fingerprint TEXT NOT NULL,
  pob_hash TEXT NOT NULL,
  pob_snapshot_path TEXT,         -- relative path to a frozen copy of the PoB at sheeting time
  name TEXT,                       -- user-editable label, default = ascendancy + skill
  schema_version INTEGER NOT NULL, -- bump if sheet shape evolves
  payload TEXT NOT NULL,           -- JSON
  authored_at TEXT NOT NULL,
  updated_at TEXT NOT NULL,
  authored_in_chat TEXT,
  validated INTEGER NOT NULL DEFAULT 0
);
CREATE INDEX idx_sheets_fingerprint ON build_sheets(fingerprint);
```

Sheets live alongside chat history in the user's `~/.bestel/` data dir. Local-only for v1. No sync.

## How the sheet is consumed

When loading context for a chat that has a matching sheet:

1. **Replace** the full PoB import block in the prompt with a compact "Active Build Sheet" block (~2-4 K tokens).
2. **Keep the raw PoB** in a tool-fetchable form (`get_full_pob_details(area: "items"|"gems"|"tree"|"…")` — already partially exists). Bestel reaches for it only when the sheet is insufficient.
3. **Tag the sheet block as cacheable** (Anthropic prompt cache). Since the sheet rarely changes, this should hit the high-cache-rate path much more cleanly than raw PoB.

This is the actual cost win: the system prompt + sheet stays cache-warm across turns, while raw PoB only loads on demand.

## Cost economics (working numbers)

Current per-échange cost (UX-1 evidence): **~$0.74** (~€0.70).

| Item | Tokens | Cost contribution |
|---|---|---|
| System + tools + raw PoB (replayed every turn, partly cached) | ~600 K input | ~$0.52 |
| Tool outputs (wikis, etc., compounded across turns) | embedded above | — |
| Output (answer + verifier 2nd pass) | ~3 K | ~$0.05 |
| Cache writes | ~$0.18 | |

Estimated post-sheet per-échange cost (rough):

| Item | Tokens | Cost contribution |
|---|---|---|
| System + tools + sheet (small, well-cached) | ~70 K input, ~95% cached | ~$0.07 |
| On-demand PoB fetch (1-2 calls per session, scoped) | ~10-30 K | ~$0.03 |
| Output | ~3 K | ~$0.05 |
| **Total** | | **~$0.15** |

**Sheet authoring cost** (one-time per build):
- Bestel drafts + 5-7 question-answer pairs + final validation = ~5-8 turns
- Per turn cost during interview is comparable to today (~$0.30-0.50, full PoB still loaded since no sheet exists yet)
- One-time spend: **~$2-4 per build**

**Break-even**: after ~3-5 follow-up chats per build, the analyzer has paid for itself. For power users (10+ chats per build per league), the lifetime savings are large.

## Risks and tradeoffs

1. **Interview fatigue**: 5-7 questions before answering the first question is a lot. Mitigation: skip-and-answer escape hatch + auto-draft so the interview is correction-only, not exhaustive.
2. **Sheet rot**: PoB evolves, sheet drifts. Mitigation: hash-based diff detection; on diff, offer "refresh" via a short delta interview rather than full re-authoring.
3. **Wrong sheet validated**: user rubber-stamps a sheet, then realizes mid-conversation it was wrong. Mitigation: any chat can append corrections to the sheet via a "update sheet with this correction" affordance.
4. **State explosion**: paths multiply (no PoB / PoB but no sheet / matching sheet / stale sheet / sheet authoring in progress). Each needs clear UI. This is the design agent's biggest task.
5. **Sheets as silent context = harder debugging**: if the model says something weird, it might be from the sheet, the PoB, or a tool output. Mitigation: dev panel shows "this turn loaded sheet vN of build X" when a sheet was active.
6. **Multi-character users**: an alt with the same ascendancy + skill but different gear lands on the same fingerprint. Mitigation: allow user to "fork" a sheet at attach time ("this is a different character — start a fresh sheet").
7. **Sheet schema lock-in**: once we ship, evolving the schema is a migration problem. Mitigation: `schema_version` column, lazy migrations on read.
8. **Privacy / shareability**: sheets contain intent ("I won't drop Mageblood") and effective spend / progression. Local-only for v1; do not export, do not embed in error reports.

## Phasing

### v1 — Manual sheet authoring + reuse (MVP)

- New table + storage layer
- Auto-draft sheet from raw PoB on user request ("Bestel, study this build")
- Inline interview flow (Path B) with auto-draft pre-fill
- "Save Build Sheet" affordance in chat gutter
- Sheet list in left sidebar above the build details
- On future PoB attach: fingerprint match → load sheet, replace raw PoB in context
- Hash-based stale detection: warn user, no auto-refresh

Estimated scope: ~3-4 weeks (back + front + design).

### v2 — Drift handling + sheet edit UI

- "Sheet diff" visualization when PoB changes after authoring
- Mini-interview to refresh stale sections only
- Direct sheet editing (not via chat) — a structured form view
- Sheet versioning with rollback

Estimated scope: ~1-2 weeks on top of v1.

### v3 — Continuous refinement

- Any chat turn can append a correction to the active sheet ("save this correction to my sheet")
- Auto-suggest sheet updates when Bestel detects a contradiction in the conversation ("I think I had your secondary skill wrong — update the sheet?")
- Multi-build "character profiles" grouping sheets together (alts, league switches, etc.)

Estimated scope: open-ended.

## Sidebar requirements (forward to design)

Today the left sidebar is the PoB build panel (`crates/bestel/ui/src/components/sidebar/BuildPanel.vue`). Proposed additions:

1. **Sheet header section** at the top of BuildPanel:
   - If a sheet exists for the current PoB: badge ("Build Sheet · v3 · ✓ validated"), one-click summary expansion, "Edit" / "View" / "Refresh" actions.
   - If no sheet: "No Build Sheet yet — Bestel doesn't know this build yet" + "Start interview" button.
   - If stale (PoB changed since sheet authored): warning chip "Build changed — sheet may be out of date" + "Refresh sheet" CTA.

2. **Sheet preview** (collapsed by default, expandable):
   - Renders the sheet's identity, archetype, intent, defining items in a read-friendly way.
   - "Show full sheet" → modal with the full structured view.

3. **Interview-in-progress affordance** (during Path B flow):
   - Progress bar / section checklist visible somewhere (sidebar or chat header).
   - Visual differentiation of interview turns vs. answer turns in the chat (icon? color tag?).

4. **Save / cancel buttons** — needs to be findable but not distracting; ideally in the chat composer area near the input, not buried in a settings dialog.

Open question for design: where does the "interview" live visually? Three candidates:

- **Inline in the chat** (current chat thread, with interview turns visually distinct)
- **Side panel** (chat continues normally, interview happens in a stacked panel)
- **Modal** (chat is paused, interview is a focused full-screen flow)

Path B as recommended above implies inline-in-chat. Modal feels heavyweight. Side panel is a middle ground worth prototyping.

## Tools / wire format additions

New tools the model needs (to be implemented in `crates/bestel-core/src/llm/tools.rs`):

- `sheet_propose_section(section_name: str, content: str)` — Bestel writes a draft section into the active sheet draft. UI shows the section being filled in the sidebar preview in real time.
- `sheet_ask(question: str, options?: [str])` — Bestel asks a structured question. UI surfaces the question with optional quick-reply chips.
- `sheet_finalize_request()` — Bestel signals "I think the sheet is ready." UI shows the "Save Build Sheet" CTA prominently.
- (Read-side) `get_active_build_sheet()` — Bestel pulls the active sheet's payload during a normal chat. Returns null if none.

New `LlmDelta` variants:
- `SheetDraftUpdate { section: String, payload_json: String }` — incremental sheet draft updates streamed to UI for live preview.
- `SheetAskUser { question: String, options: Option<Vec<String>> }` — same as `sheet_ask` tool but surfaces directly to the chat for special rendering.

These are deferred sketches; locked in during implementation.

## Locked product decisions (2026-05-09)

After design round-trip with Claude Design (handoff bundle `1q4MLngVYvv8pH4VOrlo-w`):

1. **Purpose-question cap**: no cap. Bestel decides per build, leverage-based — items / auras with the strongest impact on scaling or defense get a `sheet_ask` first, then we stop when remaining items are low-leverage. Sidebar stepper does *not* surface a "1 of N" counter.
2. **Mixed input during interview**: when free-text is entered AND chips are selected, **block submit** with an inline warning. User must clear one mode before sending. Cleaner than silent override and avoids accidental answers.
3. **Section ordering**: fixed order in v1 (`Identity → Archetype → Damage → Defense → Items → Intent`). Dynamic ordering keyed off the user's question category is v2.
4. **States covered in v1**: empty / invite / match / interview / edit / finalized / stale (the 7 states from the design).
5. **No `purpose questions · 1 of max 4` counter** in the stepper sidebar (consequence of #1).

## Open questions

1. **Auto-draft vs. blank-start interview**: the recommendation here is auto-draft from raw PoB on first interview entry. But the auto-draft itself burns tokens (one big "draft me a sheet from this PoB" call). Worth it? Or start blank and ask the user to fill?
2. **Sheet schema rigidity**: locked schema (predictable) vs. free-form sections (Bestel adds whatever sections seem useful per build)? Lean toward locked for v1, free-form notes section as escape valve.
3. **PoB attached but no fingerprint match — auto-prompt interview, or wait for the user to ask a substantive question?** Auto-prompting on attach feels intrusive; waiting until the user asks something is more polite but means the interview always interrupts a real question.
4. **Multi-language**: per project rules, persisted artifacts (sheets) are English-only. Confirmed — no I18n on the sheet content.
5. **Provider compatibility**: Anthropic and Ollama paths both need the new sheet tools. Ollama path is much weaker at structured tool use; v1 may gate "Bestel-led interview" to Anthropic only and leave Ollama users with manual sheet editing only.
6. **What counts as a "different build"?** If user copies a friend's PoB to compare, that triggers a sheet flow they don't want. Mitigation: a "compare-only mode" attach option that explicitly bypasses sheet matching.
7. **Sheet preview rendering**: do we reuse the existing markdown pipeline (`api/markdown.ts`) or build a dedicated sheet renderer? Probably a dedicated component for richer chip / chip-row layouts, similar to the Identity Card.
8. **Telemetry**: do we want anonymous opt-in telemetry on sheet quality (was the sheet edited post-save? did the user end up correcting Bestel anyway?) — useful for v2/v3 tuning. Defer the decision.

## Hand-off checklist for design agent

The design agent should produce mockups and interaction specs for:

- [ ] Sidebar "Sheet header" section in BuildPanel (no-sheet / has-sheet / stale states).
- [ ] Sheet preview component (collapsed and expanded form).
- [ ] Full sheet modal (read mode + edit mode).
- [ ] Interview-in-chat visual treatment: how does an interview turn look different from a normal answer turn? Section progress indicator?
- [ ] "Skip and answer" escape hatch — placement, copy, confirmation behavior.
- [ ] "Save Build Sheet" CTA — placement (chat gutter? side panel? composer area?) and confirmation flow.
- [ ] Stale sheet warning chip + "Refresh" mini-interview entry.
- [ ] Sheet matching prompt on PoB attach — "I think this is build X, use that sheet?" / "different character" affordance.
- [ ] Empty state when no PoB attached: "Build Sheets are tied to a PoB import. Drop one in to get started."

Constraints to respect (already established in the codebase):
- Tauri 2 + Vue 3 + Pinia.
- Existing typography: `--hand`, `--hand-display`, `--script`. Color tokens: `--paper`, `--ink`, `--ink-soft`, `--ink-faint`, `--amber`, `--red`, `--sky`, `--paper-line`.
- Identity Card chip styling already exists in `markdown.css` — reuse for sheet axis chips.
- `RunicModal.vue` for any modal flows.
