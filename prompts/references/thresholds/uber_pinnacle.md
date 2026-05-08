---
description: Numerical thresholds for "ready for uber pinnacle bosses" — the absolute high-end of the game (Uber Maven, Uber Sirus, Uber Eater, Uber Exarch, Uber Atziri / current PoE2 ubers). DPS, EHP, max-hit, recovery — separately for hit-based and DoT, with per-uber quirks.
fetch_when: User asks about uber pinnacle readiness, mentions "uber Sirus / Maven / Eater / Exarch", or evaluates a build claiming uber capability. Always cross-check against `pinnacle.md` for the sub-uber baseline and PoE1/PoE2 distinction. Re-verify against current league because uber bars drift more than sub-uber bars across patches.
---

# Thresholds — Uber pinnacle bosses

> The top tier of the game. Bars below are conservative; "carry-attempt" builds can run lower DPS but expect deaths and consumable burn.
>
> **Trade SC assumed.** HC ubers require +50% across the board with 3+ defensive layer redundancy (any single layer failure = death).
>
> **Numbers drift hard across patches.** Uber thresholds shift on every major patch — re-verify against current Pohx / Mathil / Ben_ uber-attempt videos for the active league.

## What makes uber different from sub-uber

Three structural differences:

1. **No defensive holes tolerated** — sub-uber forgives one missing layer (e.g., 95% suppression instead of 100%). Uber does not. One missing layer = one-shot death.
2. **DPS bar 2-4× higher** — uber HP pools and phase pressures require burst windows.
3. **Phase mechanics punish slow builds** — uber bosses gain extra phases, accelerated mechanics, and on-death effects. Animation cancelling fluency becomes mandatory.

## Universal floor (any uber)

| Stat | Bar | Why it matters |
|---|---|---|
| Max resists | 80%+ all (chaos overcap mandatory) | Uber bosses apply ele weakness + curse stacks. |
| Spell Suppression | 100% AND Block 75%+ Spell Block | Uber spells regularly chunk through one defensive layer. |
| Phys mitigation | 60%+ effective (armour + endurance + fortify + Determination) | Uber phys slams hit 11-14k. |
| Movement / dash fluency | dash + flame dash + Fortify-on-dash + leap | Phase dodging is mandatory, not optional. |
| Ailment immunity | yes, no exceptions (Pantheon + charm/flask + immunity stacking) | One frozen state in phase = death. |
| Ailment immunity layers | 2+ (e.g., Pantheon + Charm in PoE2 / two flasks in PoE1) | Uber adds + bosses both apply ailments simultaneously. |
| Recovery between phases | 5k+ life/sec sustained | Phase intervals 20-30 sec; recovery dictates re-engagement window. |

## Hit-based builds

| Stat | Bar | Notes |
|---|---|---|
| Total DPS (boss profile, single target) | 12-20M+ | Below 10M = struggle / multiple deaths to phase pressure. |
| EHP (HP+ES+MoM-equivalent) | 12-15k+ | Uber slams hit 11-14k; redundant defences mandatory. |
| Max hit by element | 14k+ per type | Uber Eater deathwave / Uber Exarch beam. |
| Recovery rate | 5k+ life/sec | Between-hit recovery is the difference between life and death. |
| Movement speed | 140%+ | Phase dodging matters; below 130% = slow phase rotation. |
| Effective HP per hit | 7k+ life leech burst capacity | Sustained uber phase 5+ multi-hit windows. |

### Per-archetype hit-builds at uber tier

- **Hit / crit (Lightning Arrow Deadeye, Lightning Conduit Inquisitor, COC Cospri's)**
  - Comfortable DPS bar: 15-25M.
  - 100% effective crit chance + 400%+ crit multi.
  - Penetration via Trinity + Wise Oak + curse stacking + Eldritch Battery (rare).
  - Defensive: 100% spell suppression, 14k+ EHP, dual-curse via Hexproof.
- **Hit / non-crit EO (Penance Brand Hierophant, Cyclone CWC Soulrend)**
  - Comfortable DPS bar: 12-18M.
  - Multiplier stack at 6+ layers.
  - MoM stack + 10k mana for chunky absorption.
  - Defensive: 12-14k effective EHP.
- **Trigger / CWC (CoC Cospri's, Cast on Crit DD)**
  - Comfortable DPS bar: 15-30M (trigger-latency hides peak in PoB).
  - Trigger source uptime 95%+ during boss phases.
  - Mana sustain modeled (Indigon + Cinderswallow + leech).
- **Heralds-stack (Voidforge Slayer / Heralds + Mageblood)**
  - Comfortable DPS bar: 20M+ (heralds proliferate damage).
  - Defensive: Mageblood permanent flask uptime; 12k+ EHP.

## DoT builds

| Stat | Bar | Notes |
|---|---|---|
| Total DoT DPS | 3-5M+ | Phase uptime makes raw DPS misleading; sustained DPS over 30s windows matters. |
| EHP | 11-13k | DoT chassis softer; redundant defences mandatory. |
| Max hit by element | 14k+ per type | Uber slams. |
| Recovery rate | 5k+ life/sec | Sustained chassis health. |
| Phase-skip cooldown | sub-30s on high-pressure phases | DoT builds need a way to skip lethal-add windows. |

### Per-archetype DoT-builds at uber tier

- **RF Chieftain (Pohx archetype)**
  - **Uber Maven hard limit**: RF struggles. Pohx-style RF clears T16 maps comfortably but uber-pinnacle requires specific Vaal RF + trigger-spell DPS to handle phase-skip. Most uber-RF videos use Vaal RF burst windows.
  - 12-15% effective life regen with stacked sources (ascendancy + Beacon of Madness + Watcher's Eye + tree).
- **Cold DoT Trickster (Cold Snap Vortex, Coldsnap of Power)**
  - Comfortable DoT DPS bar: 4-6M.
  - Pathfinder flask uptime + Doryani's Prototype optional.
- **Bane Hierophant (chaos DoT totem stack)**
  - Comfortable DoT DPS bar: 3-5M.
  - 3 totems + Despair + Withering Touch.
- **Poison Pathfinder (Toxic Rain, Cospri's Will)**
  - Hybrid hit + DoT bridge; treat as ailment-stack.

## Per-uber quirks (PoE1)

### Uber Maven

- **Mechanic ask**: Memory Game in P3, multi-phase boss-summoning in P4-5.
- **Defensive ask**: 13-14k EHP, chaos overcap, spell suppression cap, dual curse via Hexproof.
- **DPS ask**: 15M+ comfortable; 10M is "rolling for it".
- **Phase-skip strategy**: build that can DPS-skip Memory phase via burst is the dominant comp.
- **Hard counter**: any build without dash/flame dash + Memory-burst struggles.

### Uber Sirus

- **Mechanic ask**: P4-5 die-beam, P6 sphere-orbit phase requires phase-rotation.
- **Defensive ask**: chaos overcap MANDATORY (his degen is chaos), 14k+ EHP, spell suppression cap.
- **DPS ask**: 12M+ phase-skip range; 8M = struggle.
- **Hard counter**: stationary builds fail P6 sphere phase.

### Uber Eater of Worlds

- **Mechanic ask**: bullet-hell projectile-dodge phase 4 (intensified vs sub-uber); accelerated bullet-hell pace.
- **Defensive ask**: max-hit cold + lightning 14k+.
- **DPS ask**: 12-18M; bullet-hell phase rewards burst windows.
- **Hard counter**: low-mobility builds fail phase 4.

### Uber Searing Exarch

- **Mechanic ask**: ball-and-chain phase with accelerated cycle, ground fire stacking.
- **Defensive ask**: max fire res 80%+ overcap, max-hit fire 14k+.
- **DPS ask**: 12-18M.
- **Hard counter**: stationary builds and slow-totem builds.

### Uber Shaper / Elder

- **Mechanic ask**: phase compounded with degen ground; Shaper slams come faster.
- **Defensive ask**: 12k+ EHP, chaos overcap (Elder's tendrils).
- **DPS ask**: 10M+.
- **Hard counter**: caster builds with low mobility.

### Uber Atziri (Apex of Sacrifice / Alluring Abyss)

- **Mechanic ask**: triple-Vaal phase with simultaneous spell + slam pressures.
- **Defensive ask**: max ele resists 80%+.
- **DPS ask**: 6-10M (lower than other ubers as Atziri is a legacy uber).

### Uber Catarina

- **Mechanic ask**: pillar-identification phase, intensified.
- **Defensive ask**: chaos res, spell suppression.
- **DPS ask**: 8-12M.

## Per-uber quirks (PoE2)

⚠️ PoE2 uber pinnacles are **less-defined as of 2026-05-08** (0.4 era). 0.5 launch (2026-05-29) introduces new pinnacles + likely uber variants.

### Uber Arbiter of Ash (PoE2 0.4)

- **Defensive ask**: max fire res 80%+ overcap, max-hit fire 8-10k.
- **DPS ask**: 1.5-3M (PoE2 baseline DPS lower).

### Uber King in the Mists (PoE2 0.4)

- **Defensive ask**: chill / freeze immunity, max cold res 80%+.
- **DPS ask**: 1.5-3M.

### 0.5 ubers (post-2026-05-29)

Fill-in-place once 0.5 ships and pinnacles are documented.

## Build-defining requirements (typical of uber-tier characters)

- **No defensive holes** — one missing layer (no suppression, no block, low chaos res) = uber failure.
- **Animation cancelling fluency** — dash / flame dash / leap-cancel through phases is mandatory.
- **Build-defining uniques typical**: Mageblood (PoE1), Original Sin, Headhunter (mapping), Voidforge (high-roll attack-builds), Replica Restless Ward, perfect Awakened gems.
- **Endgame jewels typical**: Forbidden Flame + Forbidden Flesh combo for cross-ascendancy (PoE1).
- **Cluster jewel layer**: 4+ Large Clusters with hand-picked notables.

## SSF / HC

- **SSF uber bars**: same DPS / EHP, but mod ratchets less perfect — expect 70-80% of trade equivalents and longer-tier farming.
- **HC uber bars**: +50% across the board, defensive redundancy at 3+ layers, phase-skip strategies prioritised over DPS races.

## Cross-references

- `red_maps.md` — the floor.
- `pinnacle.md` — the sub-uber baseline.
- `17_build_archetype_taxonomy.md` — archetype-specific scaling levers.
- `08_defence_recovery_survivability.md` — defensive mechanics.
- `14_validation_and_failure_modes.md` Rule 1 — engine-trust before quoting numbers.
- `25_pob_engine_integration.md` — `pob_calc` will verify these directly once Sprint 2 ships.
- `creators_registry/pohx.md` / `creators_registry/mathil.md` / `creators_registry/ben_.md` — uber-attempt video sources.
- `poe2/00_version_pinning.md` — current PoE2 version anchor.
