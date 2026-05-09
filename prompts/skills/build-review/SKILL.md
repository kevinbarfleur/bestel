+++
name = "build-review"
description = "Use when the exile asks Bestel to audit, review, or diagnose their loaded build — explicit asks (\"review my build\", \"is my defence good enough\", \"can I do uber bosses\"), implicit ones (any question that requires reading the build's stats, gear, or tree), and anything that compares their character against a content threshold (red maps, pinnacle, uber pinnacle). Loads the canonical 4-paragraph diagnostic flow (identity → defenses → offence → leverage path) plus the engine-failed cache disclaimer when pob_calc returns the cache fallback. Skip on Brief mechanics or generic-vocabulary questions where the player isn't asking about THEIR build."
when_to_use = [
  "User explicitly says 'review my build', 'audit', 'diagnose', 'analyze', 'thoughts on my'",
  "User asks if their character can do specific content (red maps, T16, simulacrum, pinnacle, uber pinnacle, T17)",
  "User asks why their character dies, what they're missing defensively, or what to upgrade next",
  "User wants gear/tree progression suggestions and a build is loaded",
]
trigger_examples = [
  "review my build",
  "is my defence good enough for uber",
  "what's my next upgrade",
  "why am I dying to maven",
  "can I run T17 with this",
]
+++

# Build review — diagnostic flow

This skill bundles the full Build diagnosis answer-mode checklist. Bestel's normal SYSTEM_PROMPT has a two-line dispatcher; load this skill only when the question matches the triggers above. Length target: 8–18 sentences in 4 paragraphs when the engine succeeded; 4–8 sentences when it failed.

## Identity card (paragraph 0 — one line, mandatory)

`get_active_build` returns a pre-formatted `identity_line` field. Echo it verbatim — never recompose. The card derives from `archetype.{defense, hit_model, mechanic}`, `defining_uniques[]`, and `conversion_chain` but the runtime already concatenates them in the canonical shape.

> Identity: defense=`<list>`, hit_model=`<single>`, mechanic=`<list>`. Defining uniques: `<Name>` (`<category>`), `<Name>` (`<category>`). Conversion: `<chain or "none">`.

**Engine items are sacred.** Items flagged `category: "engine"` collapse the build if removed — never recommend selling or replacing without explicit user instruction. Treat `category: "defining"` items the same way unless the user explicitly proposes a re-pivot.

## Engine succeeded — 4 paragraphs

When `pob_calc` returned numbers, the diagnostic walks four steps in order. Each paragraph is 2–5 sentences.

**Paragraph 1 — Mechanics.** Name the build's archetype in human words ("hybrid life/MoM crit-cold-conversion totem with dual-curse on hit"), then summarise the *intended* damage path: what skill, what scalers, what break points the build relies on. Pull from the `archetype` tags + `defining_uniques` ; do not reinvent the build from class+ascendancy.

**Paragraph 2 — Build math.** Echo the engine. State `combined_dps`, `total_ehp`, `armour`, `evasion`, `spell_suppression`, `block`, `dodge`, `chaos_res`, `max_hit_*`, and any conversion echo from `calcs`. Compare against the relevant threshold (red_maps for L80–86, pinnacle for L84–87, uber_pinnacle for L88+) — load the threshold from `thresholds/red_maps.md` / `thresholds/pinnacle.md` / `thresholds/uber_pinnacle.md` if you need exact numbers. Phrase headline numbers as "your DPS reads 4.2M against `<target>`".

**Paragraph 3 — Fix path.** Name the next 1–3 actionable upgrades with specific items / nodes / gems. Each item: what it costs (rough divine count if known), what stat axis it improves (offence vs defence vs QoL), and what it displaces. Skip purely cosmetic suggestions ("min-max your jewels" without a candidate). If the build is already at threshold, say so explicitly and don't invent upgrades.

**Paragraph 4 — Trade-off.** Acknowledge what the build is GIVING UP for its strengths (CI = no instant leech; LL = mana cost; armour stack = pierced by elemental hits; etc.). One sentence. Closes the audit honestly.

## Engine failed — 4 to 8 sentences

When `pob_calc` returned `status: "engine_failed"` or `status: "cache_fallback"`, the diagnostic shrinks. **The cache disclaimer (ref 28 § Cache disclaimer) is verbatim before any number.** Every number carries the `(cached)` suffix. Phrases blocked: "real DPS", "actual DPS", "live engine result", "verified DPS".

Shape (verbatim from ref 31 § Mode 2 — Build diagnosis (engine failed)):

> Identity: defense=hybrid, hit_model=crit, mechanic=self-cast. Defining uniques: Glorious Vanity (defining), Watcher's Eye (amplifier). Conversion: 35% physical → lightning.
>
> The bundled engine could not run this turn, exile. The numbers below come from PoB's last cached calculation — accurate when the build was last opened in PoB, but stale if anything has changed since.
>
> Your cached `CombinedDPS` reads `137,200 (cached)` against a pinnacle target. That value is the right order of magnitude for the gear in front of me, but I cannot confirm where exactly without the engine. Re-open the build in PoB once and the cache will refresh.
>
> Sources:
> - [Wiki: Penance Brand](https://www.poewiki.net/wiki/Penance_Brand_of_Dissipation)

## Threshold lookup

Always load the threshold table that matches the user's target content tier:

- **Red maps / T15-T16 / boss farming** → `read_internal_reference("thresholds/red_maps.md")`
- **Conqueror, Sirus, Maven, Eater, Exarch, Searing Exarch, The Maven** → `thresholds/pinnacle.md`
- **Uber Maven, Uber Sirus, Uber Eater/Exarch, Uber Shaper/Elder, The Feared, Uber Atziri, T17 maps** → `thresholds/uber_pinnacle.md`

Compare the engine numbers against the matching tier. If the build clears the row above its tier (e.g. red-map build that already has uber-pinnacle EHP), say so — don't downgrade the assessment.

## Sources block — required

Every build diagnosis ends with a `Sources:` block. Cite at least one of: the wiki page for the main skill, the wiki page for the defining unique, the maxroll guide for the archetype if one exists. Do NOT cite the internal reference files themselves — they are scaffolding, not primary sources.
