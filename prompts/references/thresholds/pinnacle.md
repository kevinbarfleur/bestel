---
description: Numerical thresholds for "ready for sub-uber pinnacle bosses" (Maven, Sirus, Eater, Exarch, Shaper, Elder, Atziri in PoE1; Arbiter of Ash, King in the Mists in PoE2 0.4). DPS, EHP, max-hit, recovery — separately for hit-based and DoT, with per-pinnacle quirks.
fetch_when: User asks about pinnacle readiness, "can I do Maven / Sirus / Eater / Exarch?", or evaluates a build for a specific pinnacle fight. Always cross-check the PoE1/PoE2 distinction and the patch version. Cross-reference `red_maps.md` for the floor and `uber_pinnacle.md` for the ceiling.
---

# Thresholds — Pinnacle bosses

> ⚠ **COMMUNITY HEURISTICS — NOT FACTS.**
>
> The numbers in this file are aggregated from boss-kill videos, Reddit "can my build do X?" threads, Maxroll boss-fight guides, and Pohx / Mathil / Ben_ pinnacle attempts. They are **NOT engine-derived, NOT official, NOT patch-stable**. Pinnacle damage scaling is rebalanced regularly; every league cycle shifts the bar.
>
> **When citing any threshold from this file:**
>
> 1. State it as a directional bar, not a hard line. Phrase: *"rough community floor"*, *"ballpark target"*, *"creator-aggregate"*, *"around N"*.
> 2. Tag the source as "community-aggregate" in your reply; **never present these as wiki facts**.
> 3. Sub-uber tier — assume **deathless or near-deathless kill**, not "rolled for it 50 times".
> 4. When in doubt, run `pob_calc` against the user's actual build instead of citing a static bar — engine output supersedes heuristic.
> 5. Re-baseline against the current Maxroll boss-fight guides + Pohx / Mathil / Ben_ pinnacle videos if the league changed since last calibration.
>
> The numbers below are the calibration as of 2026-05-08 (last audit). They are NOT current-patch authoritative.
>
> **Trade SC assumed.** HC bars are higher across the board. SSF bars depend on found gear.

## Universal floor (any pinnacle)

| Stat | Bar | Why it matters |
|---|---|---|
| Max resists | 75-80% all (chaos overcap mandatory for Sirus / Maven) | Pinnacle bosses apply ele weakness / curse-on-hit. |
| Spell Suppression OR Block + Spell Block | 100% / 75% spell block | Pinnacle spell damage is the single biggest killer. |
| Phys mitigation | 50%+ effective (armour + endurance + fortify) | Pinnacle phys slams hit 8-10k. |
| Movement / dash fluency | dash + flame dash + leap | Phase-skipping is half the fight. |
| Ailment immunity | yes (Pantheon, charm, Brine King's Soul, etc.) | One frozen state during a phase = death. |
| Recovery between phases | sufficient flask charges or auto-regen | 30-60 sec phase intervals require recovery. |

## Hit-based builds

| Stat | Bar | Notes |
|---|---|---|
| Total DPS (boss profile, single target) | 5M+ | 3M is feasible but slow / requires RNG; 7M+ comfortable. |
| EHP (HP+ES+MoM-equivalent) | 9-12k | Pinnacle slams 8-10k; under 9k = one-shot risk. |
| Max hit by element | 10k+ per type | Eater deathwave / Exarch ball-fire deal heavy ele damage. |
| Recovery (life regen / leech / recoup) | 3k+ life/sec | Sustained between hits during phase transitions. |

### Per-archetype hit-builds at pinnacle

- **Hit / crit (Lightning Arrow Deadeye, Tornado Shot Deadeye)**
  - Comfortable DPS bar: 5-10M.
  - 100% effective crit chance + 380%+ crit multi.
  - Penetration via Trinity / Wise Oak / curse stacking.
  - Defensive: 100% spell suppression, 12k+ EHP.
- **Hit / non-crit EO (Penance Brand-of-Dissipation Hierophant)**
  - Comfortable DPS bar: 5-7M (brands ramp during phases).
  - Multiplier stack at 5+ layers (Hypothermia + Conc Effect + Trinity + ascendancy + Inevitable Judgement).
  - Defensive: MoM stack + spell suppression.
- **Trigger (CoC Cospri's, CWC Soulrend, Cast on Crit DD)**
  - Comfortable DPS bar: 7-15M (trigger latency sustains during phases).
  - Trigger source uptime 90%+ during boss phases.
  - Mana sustain modeled (flask + leech + Indigon).
  - Defensive: at least 9k EHP.

## DoT builds

| Stat | Bar | Notes |
|---|---|---|
| Total DoT DPS | 1.5-2M+ | Most pinnacle bosses have phases; uptime matters more than raw cap. |
| EHP | 8-10k | DoT chassis softer; max-hit stacks matter. |
| Max hit by element | 10k+ per type | Pinnacle slams. |
| Recovery rate | 3k+ life/sec | Critical for DoT survivability between hits. |

### Per-archetype DoT-builds at pinnacle

- **RF Chieftain (Pohx archetype)**
  - DoT cap reached at level 92-95 with mid-budget gear.
  - **Maven hard limit**: RF struggles on Maven Memory Game and pinnacle adds. Sirus / Maven attempts often do require a swap to Vaal RF or trigger-spell DPS.
  - 8% life regen baseline (ascendancy + tree + Beacon of Madness + items).
  - Cloak of Flame + Watcher's Eye for phys mitigation.
- **ED-Contagion Occultist**
  - Spell Suppression cap mandatory.
  - Caustic Cloud + Chaos DoT scaling via Despair curse.
  - ES-recharge-based recovery (Discipline + Energy from Within).
- **Bane / Soulrend Totem Hierophant**
  - Withering Touch stacks for chaos res shred.
  - Chaos DoT cap; reasonable kill speed only with Despair + 3 totems.
  - Defensive: MoM + Indigon mana stack.

## Per-pinnacle quirks (PoE1)

### Sirus (Awakener of Worlds)

- **Defensive ask**: chaos res overcap (his beam degen is chaos), spell suppression cap.
- **Movement ask**: phase 4-5 die-beam dodge windows are tight; flame dash uptime matters.
- **DPS ask**: 5M+ comfortable; phases 1-3 forgiving, 4-5 punishing.
- **Hard counter**: any build without dash/flame dash struggles in phase 5.

### Maven (atlas memory + 10-witness)

- **Defensive ask**: 10k+ EHP (Maven minor phys hits 8-9k); spell suppression.
- **Mechanic ask**: Memory Game phase requires concentration — failure = no rewards. Practice in low-tier first.
- **DPS ask**: 5M+ phase-skip range to skip add waves.
- **Phase-skip strategy**: build that can DPS-skip "Memory" phase via burst is preferred.

### Eater of Worlds

- **Defensive ask**: max ele resists (Eater deals fire, has cold-shock-degen); spell suppression for cast-spell phase.
- **Mechanic ask**: bullet-hell projectile-dodge phase — movement speed + dodge fluency required.
- **DPS ask**: 4-7M; relatively forgiving DPS-wise.
- **Hard counter**: low-mobility builds fail phase 3.

### Searing Exarch

- **Defensive ask**: max fire res (overcap), high max-hit fire (12k+).
- **Mechanic ask**: ball-and-chain bullet-hell phase; ground fire to dodge.
- **DPS ask**: 4-7M.
- **Hard counter**: stationary builds (totems with placement constraints) struggle on movement phase.

### Shaper / Elder (legacy pinnacles)

- **Defensive ask**: 9-10k EHP, max ele resists, chaos overcap (Elder's tendrils).
- **Mechanic ask**: Shaper slam phases require dodge timing. Elder's Slam covers 80% of arena.
- **DPS ask**: 4M+ Shaper, 5M+ Elder — both relatively forgiving in their tier.

### Catarina (Mastermind of Discord)

- **Defensive ask**: chaos res, spell suppression.
- **Mechanic ask**: phase pillars, identification.
- **DPS ask**: 4M+.

### Atziri (legacy)

- **Defensive ask**: high max ele resists.
- **DPS ask**: 2-4M.
- **Niche**: Lioneye's Glare uniques drop here; mostly farmed by alts.

## Per-pinnacle quirks (PoE2 0.4)

⚠️ Numbers below are conceptual placeholders — verify against current PoE2 patch + Maxroll PoE2 boss guides + creator videos. Re-baseline after every major patch (especially 0.5).

### Arbiter of Ash (PoE2 0.4 fire-pinnacle)

- **Defensive ask**: high max fire res (overcap), max-hit fire 7k+.
- **Mechanic ask**: bullet-hell + ground-fire dodge.
- **DPS ask**: 500k-1M (PoE2 baseline DPS is lower).

### King in the Mists (PoE2 0.4 ice-pinnacle)

- **Defensive ask**: chill / freeze immunity, high max cold res.
- **Mechanic ask**: ice-storm dodge, fog-arena visibility constraints.
- **DPS ask**: 500k-1M.

## Build-specific notes

- **Trigger / CWC builds** — uptime of trigger source matters more than peak DPS. PoB "Total DPS" undersells trigger-build pinnacle viability.
- **Minion builds** — minion EHP is a separate axis; AG safety is mandatory at pinnacle (AG death = build-defining unique loss).
- **Totem / mine / trap builds** — placement speed + reposition fluency is the real bar. Phase changes that move the boss reset placement.
- **Self-cast builds** — cast speed bar 3+ casts/sec for sustained DPS during boss windows.

## SSF / HC

- **SSF**: same DPS/EHP, but mod ratchets less perfect; expect 70-80% of trade equivalents.
- **HC**: bars +30-40% across the board; redundant defences (block + suppression + armour) preferred. Phase-skip strategies prioritised over DPS races.

## Cross-references

- `red_maps.md` — the floor before pinnacle.
- `uber_pinnacle.md` — the ceiling above pinnacle (10-20M+ DPS, 12-15k EHP).
- `17_build_archetype_taxonomy.md` — archetype-specific scaling levers.
- `08_defence_recovery_survivability.md` — defensive mechanics (suppression, block, armour).
- `07_offence_damage_scaling.md` — DPS scaling math.
- `14_validation_and_failure_modes.md` Rule 1 — engine-trust before quoting numbers.
- `25_pob_engine_integration.md` — `pob_calc` will verify these directly once Sprint 2 ships.
- `maxroll/poe1_bosses.md` — applied per-pinnacle progression articles.
- `poe2/00_version_pinning.md` — current PoE2 version anchor.
