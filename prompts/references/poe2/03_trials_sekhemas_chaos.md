---
description: PoE2 Trial of the Sekhemas + Trial of Chaos — the two parallel ascendancy progression systems replacing PoE1's Labyrinth. Mechanic differences, ascendancy point thresholds, when to pick which.
fetch_when: User asks how to get / upgrade an ascendancy in PoE2, mentions "Sekhemas" or "Chaos" trial, or compares to PoE1 Lab. Always apply to PoE2 only.
---

# PoE2 — Trials of the Sekhemas + Chaos

> Always check `00_version_pinning.md` for current PoE2 version. Mechanic specifics drift across patches.

PoE2 deliberately rejected the Labyrinth model. Instead, **two parallel trial systems** gate ascendancy progression — players pick whichever fits their build/skill better.

The single most useful question this doc answers: *"Which trial should this character do for ascendancy points?"*

## Concept

Each character earns ascendancy points by completing trials. Both trial systems gate the same ascendancy points (no ascendancy is locked to one trial). Player chooses by preference.

| Trial | Format | Run shape | Skill check |
|---|---|---|---|
| Trial of the Sekhemas | Multi-room dungeon, run-based, relic-modified. | Honor-resource management; persistent state per run. | Methodical, low-twitch, RNG-tolerant. |
| Trial of Chaos | Combat-arena format, escalating wave difficulty. | Wave-based combat; defensive-bar-checked. | Direct combat skill, DPS-bar gated. |

**Ascendancy points across both**: a fixed total per character, distributed across trial tiers and gated by progression milestones. The exact point count and level/quest gates are patch-volatile — fetch `wiki_parse https://www.poe2wiki.net/wiki/Ascendancy_class` for the current calibration.

## Trial of the Sekhemas characteristics

### Run mechanics

- Player enters a multi-room dungeon, navigates rooms in any order.
- Each room has an obstacle (combat, puzzle, hazard).
- **Honor** is the resource: starts full, decreases when hit by hazards. Honor 0 = run fail.
- **Sacred Fluid** / charges scale defensive options (relic upgrades, doors, healings).
- **Relics** modify the run — equip a relic loadout that fits.

### Difficulty escalation

- 1st trial (entry): manageable, designed for first ascendancy.
- 2nd trial (mid-game): scaled.
- 3rd / 4th trial (endgame): pinnacle-tier difficulty.

### When Sekhemas is the right choice

- **Squishy build** (caster with low max-hit): Honor-resource management is more forgiving than wave-DPS-check.
- **Methodical player**: rewards careful play.
- **HC player**: Sekhemas can be paused / reset more flexibly.
- **Builds with heavy gear-prerequisites**: Sekhemas relics let weak gear scale into trial completion.

### Sekhemas pitfalls

- Honor management is critical; ignore-the-mechanic players often fail.
- Relic management is a meta-game in itself.
- Time-cost per trial run is meaningfully higher than Chaos; the gap is build- and player-dependent.

## Trial of Chaos characteristics

### Wave mechanics

- Player enters an arena.
- Survive waves of escalating enemies.
- Direct combat — kill the wave to progress.
- Each wave is more difficult than the last (more / tougher / faster monsters).
- **Mod system**: each wave applies a randomly-selected modifier — element-resist-down, monster-buff, etc.
- DPS / defensive bar are the limiting factors.

### Difficulty escalation

- 1st-3rd trial: each tier is its own wave-set.
- Higher trials face more punishing modifiers.

### When Chaos is the right choice

- **Tanky build** (high max-hit, decent DPS): direct combat is straightforward.
- **DPS-confident player**: rewards clear-speed play.
- **Build with proven DPS bar**: if your build hits red maps, Chaos is usually doable.
- **Time-efficient runs**: faster per attempt than Sekhemas.

### Chaos pitfalls

- Modifier RNG can produce unwinnable rolls (multi-curse + ele-weakness on a build that can't handle).
- Defensive holes punished hard.
- Less RNG buffer than Sekhemas.

## When to pick which

| Player profile | Pick |
|---|---|
| New to PoE2, mid-budget gear | Sekhemas (more forgiving). |
| Veteran PoE2, gear-strong build | Chaos (faster). |
| HC ascending | Sekhemas (relics let you tune defensive bar). |
| Squishy build (caster, ES no Discipline yet) | Sekhemas. |
| Tanky build (block + armour or high regen) | Chaos. |
| Build with one defensive layer missing | Sekhemas (relic compensates). |
| Build that scales with map mods | Chaos (you can leverage modifier RNG). |
| HC, deathful build attempts | Sekhemas (less harsh failure). |

## Ascendancy progression schedule

Ascendancy points accrue across multiple trial completions, with each tier (entry / mid / endgame) granting a fixed bundle. The exact per-tier bundles and the total are patch-volatile — **fetch `wiki_parse https://www.poe2wiki.net/wiki/Ascendancy_class` before quoting a number**.

Some builds save the weak-tier ascendancy nodes until after they have stronger gear; others rush every available point early. Both are valid strategies.

## Pitfalls (common LLM hallucinations)

1. **Treating Sekhemas like Lab** — it's not. Lab was a single-shot dungeon run; Sekhemas is multi-trial honor-management.
2. **Treating Chaos like a normal map** — modifiers compound; bad RNG can be unwinnable.
3. **Telling a player to "skip ascendancy"** — never. Ascendancy points are always worth pursuing.
4. **Recommending the same trial for all builds** — choose by build profile, not personal preference.

## Cross-references

- `08_defence_recovery_survivability.md` — defensive bars (Sekhemas Honor, Chaos waves).
- `17_build_archetype_taxonomy.md` — archetype matching to trial pick.
- `23_hardcore_softcore_ssf_mode_differences.md` — HC trial considerations.
- `26_validation_and_self_correction.md` Rule 4 — PoE1 Lab vs PoE2 Trials disambiguation.
- `00_version_pinning.md` — version anchor.
- `04_runes_soul_cores_talismans.md` — items found in trials.
