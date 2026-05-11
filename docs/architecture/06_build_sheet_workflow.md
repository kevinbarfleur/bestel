# 06 — Build Sheet workflow

A **Build Sheet** is a validated, persistent description of a player's
build authored *with* the agent. It survives across chat sessions:
once a sheet exists for a `(ascendancy, main_skill, sorted_uniques)`
fingerprint, any future chat that attaches the same PoB sees the
sheet hydrated and can cite the user's confirmed identity / archetype
/ intent without re-interviewing.

Shipped 2026-05 across commits `530512c` (UX polish, persistent
interview, rehydration) and `17617a8` (engine-aware swap rule,
persistent submission panel).

---

## Why it exists

Generic chat repeatedly forced the agent to re-derive the build's
identity from the PoB on every turn (and to re-ask the user any
ambiguous purpose questions every chat). For multi-turn build coaching
the cost — both token-wise and conversationally — was high.

The Build Sheet captures the user-confirmed answer once:

- **6 fixed sections** (identity, archetype, damage, defense, items,
  intent) drafted by the agent and edited / confirmed by the user.
- **Defining items with roles** (`engine | defining | amplifier |
  enabler`) and per-item purpose strings (this is the Aul's Uprising
  reservation reduction, this is the Mageblood for flask uptime, …).
- **Intent constraints** (`SSF only`, `no Mageblood`, `must keep CI`,
  `cannot use chaos res rares`).
- **Known gaps** (`weapon swap missing`, `no eldritch implicits yet`).

Future chats read this back via `get_active_build_sheet` and respect
the constraints instead of suggesting impossible upgrades.

---

## Phase machine

```
                    ┌──────────────┐
                    │  phase=none  │  (no PoB or no sheet activity)
                    └──────┬───────┘
                           │ user attaches a PoB
                           ▼
              ┌──────────────────────────────┐
              │  phase=invite                │  agent calls
              │  (BSInviteSidebarCard)       │  sheet_propose_section
              │                              │  or sheet_open_interview
              └──────────────┬───────────────┘
                             │
              ┌──────────────┴──────────────┐
              │                              │
              ▼                              ▼
   phase=interview              phase=interview
   (incremental: per-section)   (one-shot: full BSInterviewPanel)
              │                              │
              │  sheet_propose_section *N    │  sheet_open_interview
              │  sheet_ask *N                │  user fills form → submit
              │                              │
              └──────────────┬──────────────┘
                             │ all sections confirmed
                             ▼
                    sheet_finalize_request
                             │
                             ▼
              ┌─────────────────────────────┐
              │  phase=finalized            │  SheetFinalized banner +
              │  (BSLinkedSheetCard)        │  SheetLoaded → sidebar card
              └─────────────┬───────────────┘
                            │
                            │  PoB changes (hash drift)
                            ▼
              ┌─────────────────────────────┐
              │  phase=stale                │  agent surfaces "drifted
              │  (BSLinkedSheetCard, stale) │  since authoring" warning
              └─────────────────────────────┘
```

Source of truth: `crates/bestel/ui/src/stores/sheet.ts` (Pinia).

```typescript
type SheetPhase = 'none' | 'invite' | 'interview' | 'edit' | 'finalized' | 'stale';

interface SheetState {
  phase: SheetPhase;
  draftSections: Map<string, SheetDraftSection>;
  pendingAsk: PendingAsk | null;
  activeSheet: ActiveSheet | null;
  activeInterview: ActiveInterview | null;
}
```

The `edit` phase is transient: the user is reworking one section's
prose before re-confirming. The other phases drive routed UI.

---

## Two interview modes

The store supports both an **incremental** and a **one-shot** flow.
The agent picks one based on how much PoB analysis it has done.

### Incremental — `sheet_propose_section` + `sheet_ask`

For lightweight builds or follow-up edits. The agent drafts ONE
section, the user Confirms / Edits, agent moves to the next section.
Optional `sheet_ask` cards interject for high-leverage purpose
questions.

Per turn: one section, then `end_turn`. The user's reply drives the
next iteration.

### One-shot — `sheet_open_interview` (Sprint UX-2)

For deep audits. The agent runs `get_active_build` + `pob_calc` + at
least 2-3 of `{wiki_parse, kb_search, read_internal_reference}` on
defining items, then emits ONE big interview delta carrying:

- All 6 section drafts pre-populated
- 3-7 leverage-purpose questions across sections
- A free-form `notes_prompt`

The frontend renders a single `BSInterviewPanel`; the user fills
everything in one round and clicks Submit.

Submission round-trips back as a structured `[INTERVIEW SUBMISSION
…]` user message. The agent recognises the marker (a runtime
directive in `anthropic.rs` *forces* `sheet_finalize_request` at
`iter 2` of the resulting turn so the agent cannot relaunch the
interview by mistake — the 2026-05-09 audit caught this exact bug
twice).

---

## Sheet tools (5)

Defined in `crates/bestel-core/src/llm/sheet_tools.rs`. Full schemas
documented in [04 — Tools catalogue](./04_tools_catalogue.md).

| Tool | Emits delta | Trigger |
|---|---|---|
| `sheet_propose_section` | `SheetDraftUpdate { section_id, title, body, confirmed }` | One section drafted; user works through one at a time |
| `sheet_ask` | `SheetAskUser { question_id, title, subtitle, options, multi, has_other }` | Leverage-based purpose question (after a section is drafted) |
| `sheet_open_interview` | `SheetInterviewOpen { payload }` | One-shot interview after deep analysis |
| `sheet_finalize_request` | `SheetFinalized { sheet_id, name }` + `SheetLoaded { … }` | Once, after all sections confirmed |
| `get_active_build_sheet` | `SheetLoaded { … }` | Lookup by fingerprint when the agent wants to read sheet fields |

Tools 1-2 are the incremental path; tool 3 is the one-shot path;
tools 4-5 are shared.

---

## Vue components (10)

Under `crates/bestel/ui/src/components/build-sheet/`:

| Component | Role |
|---|---|
| `BSInviteSidebarCard.vue` | "PoB attached — would you like a Build Sheet?" entry card when `phase=invite` |
| `BSInterviewSidebarCard.vue` | Progress card during incremental interview — shows how many sections drafted, current pending ask |
| `BSInterviewPanel.vue` | The one-shot interview panel — 6 sections + N questions + notes, rendered as a single scrollable form. Lives in the main message timeline |
| `BSDraftedCard.vue` | One drafted-section card with Confirm / Edit buttons. Renders inline in the message stream |
| `BSAskCard.vue` | `questions_v2` chip picker for `sheet_ask`. Multi / single / has-other modes |
| `BSSectionHead.vue` | Section title + stepper for the BSInterviewPanel |
| `BSSheetSavedBanner.vue` | "Your Build Sheet was saved" one-liner in the chat timeline after `sheet_finalize_request` |
| `BSLinkedSheetCard.vue` | Sidebar card showing the active sheet (name, fingerprint short-hash, fresh / stale badge). Lives in the build panel |
| `BSSheetFullModal.vue` | Modal wrapper for the full sheet view |
| `BSSheetFullView.vue` | Full sheet read-only renderer (sections, defining items, intent, gaps) |

Plus `ArtInterviewSubmission.vue` under
`components/chat/artifacts/` — renders the user's submitted answers
as a structured chat artefact in the timeline so the round-trip is
visible.

---

## Submission flow

When the user clicks Submit in `BSInterviewPanel`:

```typescript
// sheet.ts (simplified)
function submitInterview() {
  const interview = activeInterview.value!;
  const payload = serialiseInterviewToMarkdown(interview);
  chatStore.send(`[INTERVIEW SUBMISSION]\n${payload}`);
  interview.submitted = true;
  phase.value = 'interview';                 // stays in interview until finalize
}
```

The submission is a *regular user message* with a recognisable header:

```
[INTERVIEW SUBMISSION]
## Identity
<user's edit or kept draft>
## Archetype & skill
…
## Questions
- cospri-purpose: <selected option> | other: <free text>
…
## Notes
<user's free notes>
```

On the next turn, the provider:

1. Detects `[INTERVIEW SUBMISSION` in `last_user_message.content`.
2. Injects a runtime directive into the dynamic state block: "force
   `sheet_finalize_request` at `iter 2`, do NOT relaunch interview".
3. Optionally forces `tool_choice: sheet_finalize_request` at iter 2
   so the agent literally cannot pick another tool.
4. The agent calls `sheet_finalize_request` with the merged payload
   (its original drafts + user edits + answered questions).
5. Sheet persists; banner drops; sidebar card populates.

---

## Persistence

### SQLite table `build_sheets`

`crates/bestel-core/src/sheets/store.rs` reads / writes the table.
Schema (simplified):

```sql
CREATE TABLE build_sheets (
    id              TEXT PRIMARY KEY,        -- UUID
    fingerprint     TEXT NOT NULL,           -- <ascend>:<main_skill>:<sorted_uniques>
    pob_hash        TEXT NOT NULL,           -- BLAKE3 of canonical JSON at authoring
    name            TEXT NOT NULL,
    schema_version  INTEGER NOT NULL,
    payload_json    TEXT NOT NULL,           -- full BuildSheet serialised
    authored_at     TEXT NOT NULL,           -- ISO-8601
    updated_at      TEXT NOT NULL,
    authored_in_chat TEXT,                   -- session id where it was authored
    validated       INTEGER NOT NULL DEFAULT 1
);
CREATE INDEX idx_build_sheets_fingerprint ON build_sheets(fingerprint);
```

One row per validated sheet. Lookup is by `fingerprint`; staleness is
detected by comparing the stored `pob_hash` to the current build's
hash.

### Fingerprint

`crates/bestel-core/src/sheets/fingerprint.rs::compute_fingerprint_from_pob`
builds a deterministic string from the parsed `PobBuild`:

```
<ascendancy_lower>:<main_skill_lower>:<sorted_unique_names_lowered_joined_by_+>
```

E.g. `"hierophant:archmage:bottled faith+mageblood+watcher's eye"`.

Critical: the fingerprint must include **every** unique-rarity item
the build carries, sorted alphabetically. The tool description for
`sheet_finalize_request` includes a hard-coded warning against
omitting items, because the next chat won't find the sheet otherwise.

### PoB hash

`compute_pob_hash(canonical_json)` is `blake3` of the
`serde_json::to_string(&PobBuild)` output. Stable across re-parses of
the same file (the parser is deterministic).

---

## Rehydration

On every chat turn, `anthropic.rs::run` pre-computes the sheet status
*before* the LLM sees the system prompt:

```rust
let (build_sheet_line, fp, pob_hash, status_kind) = match ctx.get() {
    Some(b) => match compute_fingerprint_from_pob(&b) {
        Some(fp) => match find_by_fingerprint(&db, &fp) {
            Ok(Some(row)) =>
              if current_hash == row.pob_hash { Fresh }
              else { Stale }
            Ok(None) => Absent
        }
    }
    None => Unknown
};
```

The result is appended to the dynamic system block as one line:

- `Build sheet: validated, fresh (id=N)`
- `Build sheet: stale (id=N, hash drift since authoring)`
- `Build sheet: absent (fingerprint=…)`
- *(empty)* — no build / no DB

The agent reads this *as a runtime tag* rather than having to call
`get_active_build_sheet` and branch on the result. The 2026-05-09
audit showed the latter pattern failing — the agent loaded the
build-review skill *after* doing wiki_parse, so the STEP 0 pivot
never fired.

### UI rehydration

The Tauri command `get_active_build_sheet_for_ui` runs the same
fingerprint + lookup + hash compare and returns
`ActiveBuildSheetDto { …, pob_hash_match: bool }` to the build store.
The store sets `phase = 'finalized'` when `pob_hash_match` is true,
`phase = 'stale'` when false, `phase = 'invite'` when the lookup
returns `None`. `BSLinkedSheetCard` renders accordingly.

---

## Staleness

When `pob_hash` drift is detected:

- Sidebar card shows a *stale* badge.
- The agent's runtime tag reads `Build sheet: stale (id=N, hash drift
  since authoring)`.
- The agent is instructed (via `prompts/references/32_build_sheets.md`)
  to surface the staleness to the user *and* offer either:
  - Quick path: update one section (`sheet_propose_section` for the
    affected section, then `sheet_finalize_request` to update the
    same row).
  - Full re-author: launch a fresh interview.

Schema version migrations are also possible (`schema_version`
column), but only one version exists today.

---

## Engine-aware swap rule

When a sheet is present *and* its `defining_items` carry roles, every
upgrade suggestion the agent makes goes through the swap rule:

- If the swapped-out item has `role: "engine"` OR `category: "engine"`
  on the live `BuildIdentity` OR matches `detect_engine_mod_pattern`,
  the agent MUST ask for explicit user confirmation before
  recommending the swap.
- Sheet roles take precedence over BuildIdentity flags when both
  exist — the user-confirmed role is more authoritative.

The rule is encoded in `prompts/SYSTEM_PROMPT.md` § *Swap discipline*
and reinforced in `prompts/references/32_build_sheets.md` §
*Post-finalize discipline*.

---

## See also

- [04 — Tools catalogue](./04_tools_catalogue.md) — full schemas for
  the five sheet tools.
- [02 — Agentic system](./02_agentic_system.md) — the dynamic state
  block injection and the `[INTERVIEW SUBMISSION` force.
- [05 — PoB integration](./05_pob_integration.md) — `BuildIdentity`
  categories the sheet's `defining_items.role` overlaps with.
- `prompts/references/32_build_sheets.md` — the agent-side playbook
  for sheets (when to enter, when to finalize, sheet maintenance
  discipline).
