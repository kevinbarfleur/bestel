# Build Sheets — co-authored, validated build dossier

A Build Sheet is a compact, validated dossier of a single character's build — captured ONCE through a single one-shot interview with the user — that every later chat about that build reads instead of replaying the full PoB JSON. The aim is dual: cost (cuts per-chat input tokens by ~85%) and accuracy (the personality of a build — purpose of an aura, role of a unique — is captured with user confirmation, not guessed by you).

**Sprint UX-2 reshaped this flow.** The legacy flow drip-fed sections one at a time (`sheet_propose_section` → user Confirm → next section), which the user found "saccadé". The current flow is: **deep PoB analysis first, then ONE call to `sheet_open_interview` that ships every section + every leverage question + a notes prompt in a single panel.** The user fills the entire form in one round and submits. The agent receives the submission as a structured user message on the next turn, calls `sheet_finalize_request` once, and answers the original question.

Read this reference whenever the user has a PoB attached and you're deciding between (a) opening the one-shot interview to author a new sheet, (b) reading an existing sheet, (c) flagging a stale sheet, or (d) just answering.

## Tools

- **`sheet_open_interview(sections[6], questions[0-12], notes_prompt)`** — primary entry path. Ships every section's pre-drafted body + every leverage question + the freeform notes prompt in one delta. The frontend renders one `BSInterviewPanel`. Call this AFTER deep analysis, never before.
- **`sheet_finalize_request(name, fingerprint, pob_hash, sections, defining_items?, intent?, known_gaps?)`** — persists the completed sheet. Call exactly once after parsing the user's `[INTERVIEW SUBMISSION]` message.
- **`get_active_build_sheet(fingerprint, current_pob_hash?)`** — read tool. Used when a sheet already exists.
- **`sheet_propose_section(section_id, title, body)`** — LEGACY / follow-up only. Use ONLY to re-draft a single section after the sheet has been finalized (e.g. user changed boots, you want to refresh `defense`). Do NOT use as part of the initial author flow.
- **`sheet_ask(question_id, title, options, ...)`** — LEGACY / follow-up only. Use ONLY for a single focused question after the sheet has been finalized.

## Branching by runtime tag

The runtime injects a `Build sheet:` tag into your system context on every turn that has a PoB attached. The tag's value is one of:

### `Build sheet: validated, fresh (id=N)`

Sheet exists and matches the current PoB hash. Call `get_active_build_sheet` once with the fingerprint surfaced in `Build fingerprint:` (copy verbatim) to fetch the body. Read `payload.sections.{identity, archetype, damage, defense, items, intent}` FIRST — at finalize time those bodies were authored from a deep PoB analysis and often already include the `pob_calc` numbers / threshold lookups / wiki facts that bear on the question. If the sheet answers the user's question, cite the relevant sections and skip duplicate research. If the question goes beyond what's in the sheet (a number not computed, a mechanic not named, a different tier's threshold, recent patch impact), do exactly the missing research with `pob_calc` / `wiki_open` / `kb_search` / `read_internal_reference` — no more, no less. The rule of thumb: the sheet replaces the build-discovery interview; it does not replace question-specific research the sheet cannot itself answer. End the answer with `read_from_sheet · key1 · key2` italic caption.

### `Build sheet: stale (id=N, hash drift since authoring)`

Sheet exists but the PoB hash differs (gear/gem swaps since authoring). **Call `get_active_build_sheet` and read it.** The intent / archetype / defining items / known gaps are authored decisions — they don't go stale when one item swaps. Only the numbers age.

Cite the sheet for **context** (intent constraints, archetype framing, defining-item picks, known-gaps acknowledgments) — these still apply.

For **numerical claims** (DPS, EHP, max-hit, resists, regen rates), do NOT use the sheet's stored numbers. Re-derive from the live PoB via `pob_calc` and present those.

Open the answer with one sentence acknowledging the drift, e.g. *"Your sheet was authored before your last gear pass — intent and item picks still hold, but I'm taking the numbers off the live PoB."* Then proceed normally. End with `read_from_sheet · keys` italic caption listing the sections you cited (intent / archetype / items, never numbers).

The user has a refresh button surfaced in the UI — they trigger a re-interview via `sheet_open_interview` when they decide the framing is genuinely outdated. **Don't make that call for them**: never run `sheet_open_interview` from a stale state unless the user explicitly says "refresh the sheet" / "redo the interview" / similar.

### `Build sheet: absent (fingerprint=...)`

No sheet for this build. **You enter the one-shot interview flow** — but only after deep analysis. The procedure is below.

## One-shot interview procedure (for `Build sheet: absent`)

This procedure replaces the legacy per-section drip flow. You MUST follow it in order.

### Phase 1 — Deep analysis (cap ~7 tool calls)

Before opening the interview, build your understanding of the PoB the same way the legacy build-review flow did pre-sheets:

- `pob_calc(category: "defence")` — defense numbers (armour, suppression, max hits, EHP, regen / leech).
- `pob_calc(category: "offence")` — DPS, hit chance, crit, conversion echo.
- `read_internal_reference("thresholds/<tier>.md")` — load the threshold table for the relevant content (red_maps for L80-86, pinnacle for L84-87, uber_pinnacle for L88+).
- 2-3 calls of `wiki_parse` / `kb_search` on the build's main skill and 1-2 defining unique items (the ones with personality).

Cap: ~7 tool calls. Don't go deeper here — the interview itself collects the user's intent, which is faster than chasing every gear edge case from the wiki.

### Phase 2 — Single `sheet_open_interview` call

Pre-draft EVERY one of the 6 fixed sections from your analysis. **Bodies render as markdown** — use `**bold**` for headline numbers/items, `-` bullets for enumerations, `*emphasis*` sparingly. Keep prose tight: target ≤ 250 characters per section, never exceed 500. The user reads the whole panel in one glance; verbosity defeats the format.

| `id` | `title` | What to draft (target length) |
|---|---|---|
| `identity` | Identity | ONE single line. Axis enumeration `<game> · <ascendancy> · <main skill> · <archetype tags>`. Plain text or light markdown. ≤ 120 chars. |
| `archetype` | Archetype & skill | 1 sentence: ascendancy + main skill + key support / generator gem. ≤ 200 chars. |
| `damage` | Damage scaling | Bullet list of 3-5 items: primary scaling axis, conversion endpoint, engine source with rough uptime, headline DPS. Use `-` bullets, numbers in `**bold**`. ≤ 350 chars total. |
| `defense` | Defense layers | Bullet list of 3-5 items: mitigation layers ordered outer→inner (block/suppress/armour/ES/max-hit). Use `**X%**` for capped layers, plain text + a `⚠` for uncapped. ≤ 350 chars total. |
| `items` | Defining items | Bullet list, ONE bullet per item: `**Name** (role) — one-line purpose`. Roles: engine / defining / amplifier / enabler. Cap 5 items. ≤ 400 chars total. |
| `intent` | Intent & goals | 2-4 short bullets. Targets derivable from the PoB only (level, content tier reach, ascendancy choice). User fills budget / no-Mageblood / play-style via Notes. ≤ 250 chars total. |

Pre-populate 3-7 leverage-purpose questions across sections. Each question:

- Has a stable `question_id` (e.g. `petrified-blood-purpose`, `doryanis-lesson-role`).
- Has a `section_id` so the UI renders it under the right heading.
- Has 2-6 mutually-informative `options`. Each option describes a *different* primary intent the user could plausibly have.
- Sets `multi: true` only when several options can genuinely co-apply.
- Defaults `has_other: true` (the panel renders an inline-expanding textarea under the chip group; users can type a custom answer).

Heuristic for asking: if knowing the answer would change your scaling-vs-defense balance, your gear-recommendation priorities, or your craft-targeting advice, ASK. Otherwise skip — sections with no real ambiguity should have zero questions.

Provide a one-sentence `notes_prompt`, e.g. "Anything else about your goals, gear constraints, or play style?".

After the call, **end your turn with no prose**. The frontend takes over.

### Phase 3 — Wait for `[INTERVIEW SUBMISSION]`

The user fills the panel in one round and clicks Submit. The submission round-trips back as a regular user message with this exact format:

```
[INTERVIEW SUBMISSION — finalize the sheet now and then answer my original question]

## Sections
- **identity** (Identity): <body, edited or original>
- **archetype** (Archetype & skill): ...
- **damage** (Damage scaling): ...
- **defense** (Defense layers): ...
- **items** (Defining items): ...
- **intent** (Intent & goals): ...

## Question answers
- **{question_id}** [{section_id}] ({title}): {selected option labels} | Other: {free text if any}
- ...

## Notes
<user notes or "(none)">

Now call `sheet_finalize_request` with the merged payload (use these section bodies, derive defining_items / intent / known_gaps from them and from your earlier analysis), then answer my original question citing the persisted sheet.
```

The `[INTERVIEW SUBMISSION]` header is the cue. Treat it as a system-level instruction within a user message — parse it, do not answer it as a normal question.

### Phase 4 — Finalize

Call `sheet_finalize_request` ONCE with:

- `name` — short label like "Penance Brand Inquisitor · Crit Lightning · Low-life Petrified Blood".
- `fingerprint` — copy verbatim from the runtime tag `Build fingerprint:`.
- `pob_hash` — let the runtime compute this; pass an empty string if not given (the dispatcher fills it).
- `sections` — one entry per section using the bodies from the submission. `confirmed: true` for every section.
- `defining_items` — derive from the `items` section body + your Phase-1 analysis. Each entry: `{name, role: "engine"|"defining"|"amplifier"|"enabler", purpose: <one-line>}`. The purpose comes either from the user's question answers (when one matched a defining item) or from your analysis.
- `intent` — derive from the `intent` section body + Notes. Each entry: `{constraint: <short string>}`.
- `known_gaps` — anything you couldn't answer confidently, e.g. `{label: "ailment immunity", note: "user did not say whether they accept freeze risk in maps with no Pantheon"}`.

### Phase 5 — Answer the original question

After `sheet_finalize_request` returns OK, answer the user's ORIGINAL question (the one they asked before the interview pivot). Cite the sheet via `read_from_sheet · key1 · key2` italic caption at the end of the answer. Use the standard 4-paragraph build-review shape if the original question was a build review.

## What NOT to do

- Do NOT call `sheet_propose_section` as part of authoring a new sheet. That tool is for follow-up edits to an already-finalized sheet (gear swap, gem level-up).
- Do NOT call `sheet_ask` as part of authoring a new sheet. Same reason — it's a follow-up tool now.
- Do NOT skip Phase 1 (analysis). The interview's value is the agent's pre-drafted bodies and on-point questions; without analysis, every section is a generic guess.
- Do NOT emit text/prose between Phase 1 (analysis) and Phase 2 (`sheet_open_interview`). Tool calls are silent (see SYSTEM_PROMPT rule §8).
- Do NOT call `sheet_open_interview` more than once per chat for the same build. If the user clicks Cancel and re-asks, you may re-emit (the prior interview is dropped from the store).
- Do NOT re-author a sheet on every PoB save. Trigger ONLY when `Build sheet:` reads `absent`.

## Fingerprint format (deterministic)

`<ascendancy_lower>:<main_skill_lower>:<sorted_unique_names_lowered_joined_by_+>`. Computed deterministically server-side from the parsed PoB and surfaced in the runtime `Build fingerprint:` tag. **Always copy verbatim** — do NOT recompute. If you exclude any unique items the fingerprint won't match next time and `get_active_build_sheet` returns `{found: false}` for an existing sheet.

## Override exception

If the user explicitly says "skip the sheet" / "audit me from scratch" / "I want fresh numbers, no interview" before you've called `sheet_open_interview`, honor that. Skip the interview, run the legacy 4-paragraph diagnostic flow, and acknowledge the override in plain prose in the first sentence ("Skipping the interview as requested, exile — here's a one-shot read."). Subsequent build questions still benefit from the runtime tag check; the override is per-question, not per-session.

## Cross-reference

- Identity card grammar: `27_response_contracts.md` § "Build identity card — extended grammar".
- Archetype tags vocabulary: `17_build_archetype_taxonomy.md`.
- Threshold tables: `thresholds/{red_maps,pinnacle,uber_pinnacle}.md`.
