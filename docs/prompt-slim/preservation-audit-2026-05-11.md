# Prompt-slim preservation audit — Rule 3 (Build Sheet sequence)

This document tracks **every instruction** present in the current Rule 3 of `SYSTEM_PROMPT.md` (line 28) and proves that none disappears during the slim sprint — they are either kept in place, delegated to the already-injected runtime tag, or to ref 32.

## Source A — current Rule 3 (SYSTEM_PROMPT.md line 28)

650 words. Single monolithic paragraph covering: header rule, branching on runtime tag, fresh case, stale case, absent case (interview flow), override case, exemption, pointer.

## Source B — runtime tag injected by `crates/bestel-core/src/llm/anthropic.rs:339-428`

Three conditional directives injected at every turn that has a build attached. Each branches on the server-computed `SheetStatusKind`:

| Condition | Text injected |
|---|---|
| `last_user_was_interview_submission == true` | `[Submission directive: …]` — finalize + answer original question (lines 363-378) |
| `SheetStatusKind::Absent` | `[Sheet directive: a build is loaded but has no validated Build Sheet … one-shot interview flow … See ref 32]` (lines 381-396) |
| `SheetStatusKind::Fresh` | `[Sheet directive: a validated, fresh Build Sheet exists … call get_active_build_sheet … read from sections as source of truth … end with read_from_sheet · key1 · key2 italic]` (lines 399-406) |
| `SheetStatusKind::Stale` | `[Sheet directive: a Build Sheet exists but PoB hash differs … intent/archetype/items don't age, numbers do … re-derive numbers via pob_calc … drift acknowledgement … read_from_sheet caption listing sections cited (intent/archetype/items, never numbers) … don't call sheet_open_interview to refresh]` (lines 409-422) |

## Source C — ref 32 `32_build_sheets.md`

147 lines. Full procedural spec covering:
- Tools list (lines 9-15)
- Branching by runtime tag (lines 17-37) — fresh / stale / absent
- One-shot interview procedure (lines 41-124) — Phase 1 analysis, Phase 2 sheet_open_interview, Phase 3 submission parsing, Phase 4 finalize, Phase 5 answer
- What NOT to do (lines 126-133)
- Fingerprint format (lines 135-137)
- Override exception (lines 139-141)
- Cross-references (lines 143-147)

## Audit phrase-par-phrase

Each row : the instruction extracted from current Rule 3 → covered by (A) new Rule 3, (B) runtime tag, (C) ref 32 → verdict.

| # | Instruction in current Rule 3 | A: new Rule 3 | B: runtime tag | C: ref 32 | Verdict |
|---|---|---|---|---|---|
| 1 | Header "Build-Sheet sequence is mandatory and ordered" | ✅ kept (renamed slightly) | — | — | **KEEP in Rule 3** |
| 2 | "When Build state: loaded and the question is build-specific, branch on the runtime `Build sheet:` tag" | ✅ kept (compressed) | ✅ tag exists | ✅ § Branching | **KEEP in Rule 3** |
| 3 | "A sheet that exists is always read — fresh OR stale" | ✅ kept verbatim | ✅ implied in both Fresh + Stale directives | ✅ § Branching | **KEEP in Rule 3** (load-bearing invariant) |
| 4 | Reason: "intent / archetype / defining items / known gaps are authored choices that don't go stale when gear shifts. Only the numbers age." | ✅ kept (1 phrase) | ✅ in Stale directive lines 412-414 | ✅ § Stale | **KEEP in Rule 3** (the *why*) |
| 5a | Fresh: call `get_active_build_sheet` once with fingerprint from `Build fingerprint:` tag | — | ✅ Fresh directive lines 401-402 | ✅ § Fresh | **DELEGATE** (runtime tag is authoritative) |
| 5b | Fresh: read payload sections FIRST, then answer | — | ✅ Fresh directive lines 402-404 | ✅ § Fresh | **DELEGATE** |
| 5c | Fresh: sheet often contains pob_calc/threshold/wiki facts | — | ✅ Fresh directive line 403 | ✅ § Fresh | **DELEGATE** |
| 5d | Fresh: cite them and skip duplicate research | — | ✅ Fresh directive line 404 | ✅ § Fresh | **DELEGATE** |
| 5e | Fresh: if question goes beyond sheet, do the missing research (pob_calc / wiki_open / kb_search / read_internal_reference) | — | ✅ Fresh directive lines 404-405 ("Skip pob_calc … unless the sheet does not cover the question") | ✅ § Fresh | **DELEGATE** |
| 5f | Fresh: "sheet replaces the build-discovery interview, never the question-specific research it cannot itself answer" | — | ✅ implied in Fresh directive | ✅ § Fresh | **DELEGATE** |
| 5g | Fresh: end with `read_from_sheet · keys` italic caption | — | ✅ Fresh directive line 406 | ✅ § Fresh | **DELEGATE** |
| 6a | Stale: call `get_active_build_sheet` ALSO | — | ✅ Stale directive line 411 | ✅ § Stale | **DELEGATE** |
| 6b | Stale: read sections for context (intent, archetype, defining items, known gaps) | — | ✅ Stale directive lines 412-414 | ✅ § Stale | **DELEGATE** |
| 6c | Stale: DO NOT cite sheet's stored numbers — re-derive from live PoB via pob_calc | — | ✅ Stale directive lines 415-417 | ✅ § Stale | **DELEGATE** |
| 6d | Stale: open answer with drift acknowledgement (example given: "Your sheet was authored before your last gear pass — the intent and item picks still hold, but I'm taking the numbers off the live PoB") | — | ✅ Stale directive line 414 | ✅ § Stale (with example) | **DELEGATE** (example preserved in both B + C) |
| 6e | Stale: end with `read_from_sheet · keys` italic caption listing sections cited (intent / archetype / items, never numbers) | — | ✅ Stale directive lines 418-420 | ✅ § Stale | **DELEGATE** |
| 6f | Stale: "the user has a refresh button — they'll trigger a re-interview when ready; you don't decide that for them" | — | ✅ Stale directive lines 420-421 | ✅ § Stale (line 35) | **DELEGATE** |
| 7a | Absent: enter one-shot interview flow | — | ✅ Absent directive lines 381-396 | ✅ § One-shot interview procedure | **DELEGATE** |
| 7b | Absent: deep analysis cap ~7 tool calls (pob_calc defence + offence, threshold ref, 2-3 wiki_parse / kb_search) | — | ✅ Absent directive lines 385-387 | ✅ § Phase 1 | **DELEGATE** |
| 7c | Absent: ONE call to sheet_open_interview with 6 sections pre-drafted + 3-7 leverage questions + notes_prompt | — | ✅ Absent directive lines 388-389 | ✅ § Phase 2 | **DELEGATE** |
| 7d | Absent: end turn silently (no prose) | — | ✅ Absent directive lines 389-390 | ✅ § Phase 2 line 81 | **DELEGATE** |
| 7e | Absent: on next turn parse [INTERVIEW SUBMISSION], call sheet_finalize_request once, answer original question | — | ✅ Submission directive lines 363-378 | ✅ § Phase 3-5 | **DELEGATE** |
| 7f | Absent: do NOT use sheet_propose_section or sheet_ask for authoring — those are follow-up-edit tools only | — | ✅ Absent directive lines 393-394 | ✅ § What NOT to do (line 128) | **DELEGATE** |
| 8a | Override: only when user explicitly says "skip the sheet" / "audit me from scratch" / "I want fresh numbers, no interview" | — | ✅ Absent directive line 394-395 ("Override only if the user explicitly says 'skip the sheet' or 'audit me from scratch'") | ✅ § Override exception (lines 139-141) | **DELEGATE** |
| 8b | Override: acknowledge the override in plain prose | — | ⚠️ implied only | ✅ § Override exception (line 141 with example) | **DELEGATE** (ref 32 has the example) |
| 8c | Override: run legacy 4-paragraph diagnostic flow without authoring | — | ⚠️ implied only | ✅ § Override exception (line 141) | **DELEGATE** |
| 9 | "Trivial mechanic questions (no character context required) are exempt entirely" | ✅ kept (one-liner) | ⚠️ not explicit — Stale directive doesn't have this gate | ⚠️ not explicit | **KEEP in Rule 3** — only place this is documented |
| 10 | "See ref 32 for the full procedure" | ✅ kept | — | — | **KEEP in Rule 3** (pointer) |

## Special concerns

### Concern 1 — `Build sheet:` tag absent (DB failure mode)

`anthropic.rs:244-249` : when `find_by_fingerprint` errors, `build_sheet_line` is empty and the entire tag block is **not injected**.

**Current behavior** : Rule 3 says "branch on the runtime `Build sheet:` tag" — if the tag isn't there, current behavior is undefined.

**Proposed behavior** : same. The new Rule 3 also points at the tag. If the tag doesn't fire (DB failure), the agent has no sheet-related guidance, falls back to legacy flow. Acceptable degradation — DB failure is rare and the user will see no sheet effects.

**No regression introduced.** ✅

### Concern 2 — Stale directive doesn't gate on "build-specific" question

The Absent directive opens with "If the user's question is build-specific" — gating the interview flow.
The Fresh directive opens with "For build-specific questions, call get_active_build_sheet" — gating.
The Stale directive opens with **"Call get_active_build_sheet anyway"** — **no gate**.

This means for a stale-sheet build, a trivial mechanic question would technically trigger a sheet read. This is a **pre-existing behavior** the current Rule 3 ALSO has via instruction #9 (the "trivial questions exempt entirely" line).

**Mitigation** : the new Rule 3 explicitly preserves instruction #9 — it tells the agent that trivial mechanic questions skip the whole flow regardless of sheet status. This **adds robustness** to a gap in the runtime tag.

**No regression introduced.** ✅

### Concern 3 — Drift acknowledgement example phrasing

The current Rule 3 gives the verbatim example *"Your sheet was authored before your last gear pass — the intent and item picks still hold, but I'm taking the numbers off the live PoB."*

Test outputs show this phrasing being adopted by V4-Flash and V4-Pro (see baseline Q1, Q4, Q5 answers). The example is load-bearing for natural prose.

**Verification** : the same example is preserved both in the runtime Stale directive (lines 410-414 paraphrase) and in ref 32 § Stale (line 33 verbatim).

**No regression introduced.** ✅

### Concern 4 — `read_from_sheet · keys` italic caption discipline

Currently dictated in 2 places in Rule 3 (Fresh case 5g + Stale case 6e). Test outputs show 100 % conservation across V4-Flash and V4-Pro sessions where sheet is read.

**Verification** : the runtime Fresh + Stale directives both reiterate this caption requirement. Ref 32 § Fresh + § Stale also.

**No regression introduced.** ✅

## Proposed new Rule 3

```markdown
3. **Build-Sheet sequence (when build is loaded).** Branch on the runtime `Build sheet:` tag — the `[Sheet directive: …]` block injected right after the tag spells out the exact procedure for fresh / stale / absent. Two invariants apply regardless of branch: (a) **a sheet that exists is always read** — fresh or stale — because intent / archetype / defining items / known gaps are authored choices that don't go stale when gear shifts (only the numbers age); (b) trivial mechanic questions (no character context required) are exempt from the sheet flow entirely. See ref 32 for the full procedure.
```

**Word count** : current Rule 3 = ~650 words → new Rule 3 = ~95 words. **Net gain : ~555 words preserved as either runtime injection (3 directives covering 22 instructions) or ref 32 (every detail).**

## Verification checklist

Before merging, on the new SYSTEM_PROMPT.md verify:

- [ ] New Rule 3 still names the runtime tag (`Build sheet:` and `[Sheet directive: …]`)
- [ ] Invariant (a) "sheet that exists is always read — fresh or stale" present verbatim
- [ ] Reason "intent / archetype / defining items / known gaps don't go stale" present
- [ ] Invariant (b) "trivial mechanic questions exempt" present
- [ ] Pointer "See ref 32" present
- [ ] Runtime tag (anthropic.rs) UNCHANGED — `Submission`, `Fresh`, `Stale`, `Absent` directives still inject
- [ ] Ref 32 UNCHANGED — full procedure preserved

After the quality-batch replay:

- [ ] Sheet read at position 2 across 100 % of sessions where sheet exists (binary, must match baseline)
- [ ] Drift acknowledgement sentence appears in 100 % of Stale-sheet sessions (must match baseline)
- [ ] `read_from_sheet · keys` italic caption appears in 100 % of sheet-read sessions (must match baseline)
- [ ] Absent flow: deep analysis → sheet_open_interview → silence → submission → sheet_finalize_request still works end-to-end
- [ ] Override path ("skip the sheet" / "audit me from scratch") still falls back to legacy flow
- [ ] Trivial mechanic questions (e.g. "what is `Resolute Technique`") still skip the sheet
