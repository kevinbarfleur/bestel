---
description: Numerical thresholds for "ready for red maps" — the bar at PoE1 T14-16 / PoE2 T14-15 mapping. DPS, EHP, max-hit, recovery — separately for hit-based and DoT, with per-archetype anchors.
fetch_when: User asks "am I ready for red maps?", "what's the bar for tier X?", or you're evaluating a PoB against a mapping bar. Always cross-check the PoE1/PoE2 distinction and the patch version (thresholds drift across patches). Cross-reference `17_build_archetype_taxonomy.md` for archetype-specific scaling.
---

# Thresholds — Red maps

> ⚠ **COMMUNITY HEURISTICS — NOT FACTS.**
>
> The numbers in this file are aggregated from creator videos, Reddit "did I make it?" megathreads, and historical Maxroll / Pohx / Ben_ guides. They are **NOT engine-derived, NOT official, NOT patch-stable**. They drift every league, sometimes within a league when a major mechanic changes.
>
> **When citing any threshold from this file:**
>
> 1. State it as a directional bar, not a hard line. Phrase: *"rough community floor"*, *"ballpark target"*, *"creator-aggregate"*, *"around N"*.
> 2. Tag the source as "community-aggregate" in your reply; **never present these as wiki facts**.
> 3. When in doubt, run `pob_calc` against the user's actual build instead of citing a static bar — engine output supersedes heuristic.
> 4. Re-baseline against the current Maxroll / Pohx / Ben_ league guides if the league changed since last calibration.
>
> The numbers below are the calibration as of 2026-05-08 (last audit). They are NOT current-patch authoritative.
>
> **Trade SC assumed.** HC bars are higher across the board. SSF bars depend on what you've found.

## Universal floor (any archetype)

These are non-negotiable for T14-16 mapping in PoE1 / equivalent tier in PoE2.

| Stat | Bar | Why it matters |
|---|---|---|
| Max resists | 75% all (78%+ chaos preferred) | Below 75% any = the modifier "Reduced Maximum Resistances" map mod chunks you. |
| Spell Suppression OR Block | 90-100% / 60%+ | Spell damage is the #1 killer at red-map tier without one of these. |
| Movement speed | 130%+ | Below = runs feel slow but workable. |
| Phys mitigation (armour or guard) | 30%+ | Phys slams from rares hurt without it. |
| Ailment immunity (chill / freeze / shock at minimum) | yes | Pantheon (PoE1), Charms (PoE2), Stormshroud, Brine King's Soul. |

## Hit-based builds

| Stat | Bar | Notes |
|---|---|---|
| Total DPS (boss profile, single target) | 1.5M+ | Below 1M = bossing struggle in T14-16. |
| EHP (HP+ES+MoM-equivalent) | 7-8k | Below 6k = one-shot risk against rare slams. |
| Max hit by element | 7k+ per type | Rares can pop this on ele weakness map mods. |
| Recovery (life regen / leech / recoup) | 1.5-2k life/sec | Sustained between hits. |

### Per-archetype hit-builds

- **Hit / crit (Lightning Arrow Deadeye, Tornado Shot Deadeye, Frostblades Trickster)**
  - 95%+ effective crit chance; crit multi 350%+.
  - Penetration via Trinity / Wise Oak / curse if hitting capped resists.
  - Mageblood-tier "endgame" version exists at 5M+ DPS / 12k+ EHP — that's pinnacle-grade, not red-map.
- **Hit / non-crit EO (Penance Brand-of-Dissipation Hierophant, Storm Brand Trickster)**
  - Elemental Overload uptime confirmed (some hit must crit per 8s).
  - More multipliers stacked: 4 layers minimum (Hypothermia + Conc Effect + Trinity + ascendancy).
  - MoM stack typical for survivability.
- **Trigger (CoC Cospri's, CWC Soulrend, CWDT Steelskin)**
  - Trigger source uptime 90%+ during boss fights.
  - Mana sustain on triggered spell modeled (flask, leech, Indigon).
  - DPS bar same as hit-crit; trigger latency makes the *number* look smaller in PoB without reflecting actual feel.

## DoT builds

| Stat | Bar | Notes |
|---|---|---|
| Total DoT DPS (against pinnacle) | 800k+ | DoT-cap-bound; raw "DPS number" misleading without ailment context. |
| EHP | 6-7k | DoT chassis typically softer; max-hit matters more. |
| Max hit by element | 8k+ per type | T14-16 bosses can hit 10k+. |
| Recovery rate | 2k+ life/sec | Sustained damage stress on DoT chassis. |

### Per-archetype DoT-builds

- **RF Chieftain (Pohx archetype)**
  - 8% life regen baseline (ascendancy + tree + items).
  - Cloak of Flame for phys mitigation.
  - DoT cap typically reached at level 90-94 with mid-budget gear.
  - **Archetype hard limit**: Maven / Sirus single-target is weak — the bar above assumes mapping.
- **ED-Contagion Occultist**
  - Spell Suppression cap mandatory.
  - Caustic Cloud + Chaos DoT scaling via Despair curse.
  - ES-recharge-based recovery (Energy from Within etc.).
- **Bane / Soulrend Totem Hierophant**
  - Withering Touch stacks for chaos res shred.
  - Totem placement speed > cast speed.
  - MoM stack with Indigon + Inspired Learning + a lot of mana.

## Ailment-stack builds

Ailment-stack builds bridge hit and DoT — front-load hit damage, DoT-cap on the ailment.

| Stat | Bar | Notes |
|---|---|---|
| Hit damage | 200k-500k per hit | The ailment magnitude scales from this. |
| Ailment DPS (effective) | 1M+ | Cap is per-skill; check PoB Calcs ailment magnitude. |
| Ailment chance | 100% | Sub-100% kills ailment uptime. |
| Application rate | adequate for proliferation | 5+ poison stacks on bosses, ignite-prolif uptime, bleed stacks during phase. |
| EHP | 7-8k | Hybrid life + suppression typical. |

### Per-archetype ailment-stack

- **Poison-stack Pathfinder (Cospri's Will, Toxic Rain)**
  - 100% poison chance from gear + tree.
  - Master Surgeon flask uptime.
  - Proliferation via Cospri's Will + Wildfire jewel.
- **Bleed-explode Gladiator (Lacerate, Boneshatter)**
  - 100% bleed chance.
  - Gratuitous Violence ascendancy = bleed-explode chain on kill.
  - High block + Versatile Combatant.
- **Ignite-prolif Elementalist (Penance Brand Ignite)**
  - Single-ignite-cap aware (ignite is per-monster max, doesn't stack — magnitude is the lever).
  - Master of Pain mastery for ignite proliferation on death.

## Minion builds

Two health bars to track — minion EHP and player EHP — with different bars.

| Stat | Bar | Notes |
|---|---|---|
| Total minion DPS (boss profile) | 1.5M+ | Aggregate across all minions. |
| Minion EHP | 4-6k per minion (spectres) | Low for SRS, high for spectres / golems. |
| Player EHP | 6-7k | Player-side often deprioritised but T14+ requires this minimum. |
| Minion supports | 5+ effective links | +gem-level + Awakened Minion Damage etc. |

### Per-archetype minion

- **Spectre Necromancer**
  - +gems on helmet (`+2 to Socketed Minion Gems`).
  - Spectre choice depends on current league — Maxroll spectre tier list authoritative.
  - AG safety mandatory at 50ex+ gear.
- **SRS Necromancer (Specter of Phaaryl etc.)**
  - SRS skill duration scales total damage output.
  - Carrion Lord + Ascendancy Mistress of Sacrifice.
- **Golem builds (Holy Relic, Stone Golem, Carrion Golem)**
  - Build-defining unique: Anima Stone, +Golems gear.
  - Niche in current meta but viable.

---

## PoE2 specifics (current as of `poe2/00_version_pinning.md`)

⚠️ Numbers below are conceptual placeholders — verify against current PoE2 patch + Maxroll PoE2 guides. Re-baseline after every major patch.

| Stat | Bar | Notes |
|---|---|---|
| Total DPS (single target) | 200k-500k | PoE2 baseline DPS is ~3-4x lower than PoE1 due to slower combat. |
| EHP | 4-6k | PoE2 EHP scales differently — Spirit-fed defensive auras, Grim Feast for ES. |
| Max resists | 75%+ (chaos overcap critical for ES builds) | Chaos damage deals **double** to ES in PoE2. |
| Movement / dodge fluency | yes | PoE2 dodge-roll i-frame fluency replaces some passive defence. |
| Recovery | 800-1500 life/sec | PoE2 recovery is harder to scale; Spirit-aura recovery (Vitality I/II) is the main lever. |

### PoE2 per-archetype

- **Combo-builder (Monk, Huntress)**
  - Primer status uptime against bosses.
  - Executor "more damage" applies *only* during primer window.
  - Bar is not raw DPS — it's the executor-multiplied figure during primer.
- **Companion-pet (Druid, Witch)**
  - Companion EHP matters as much as player EHP.
  - Re-summon cooldown vs phase length.
- **Herald-stack (Tactician Mercenary)**
  - Spirit budget allows 3-5 heralds.
  - Damage scaling via heralds, not main skill.

---

## Beyond red maps

For pinnacle bars (Maven / Sirus / Eater / Exarch / Shaper / current PoE2 pinnacles), see `pinnacle.md`.

For uber bars (Uber Maven / Uber Sirus / current PoE2 ubers), see `uber_pinnacle.md`.

## Cross-references

- `17_build_archetype_taxonomy.md` — per-archetype scaling levers + diagnostic checklists.
- `08_defence_recovery_survivability.md` — defensive layer mechanics.
- `07_offence_damage_scaling.md` — DPS scaling math.
- `14_validation_and_failure_modes.md` Rule 1 — engine-trust before quoting numbers.
- `25_pob_engine_integration.md` — `pob_calc` will verify these directly once Sprint 2 ships.
- `maxroll/poe1_bosses.md` / `maxroll/poe1_getting_started.md` — applied progression articles for current league.
