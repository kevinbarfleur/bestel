---
description: PoE2 Spirit budget, breakpoints, mana sustain interaction. Spirit reserves auras and persistent skills (heralds, persistent buffs); mana still costs skill activations. PoE2-only mechanic; never apply to PoE1.
fetch_when: User asks about Spirit (PoE2 context), "how do I afford another aura?", a PoE2 reservation puzzle, or compares with PoE1's reservation system. Always apply to PoE2 questions only — never to PoE1.
---

# PoE2 — Spirit economy

> Always check `00_version_pinning.md` for current PoE2 version before quoting numbers below.

This doc is about **Spirit as a separate budget from mana**, the most distinctive resource mechanic in PoE2 vs PoE1. Conflating Spirit with mana / reservation is one of the most common LLM hallucinations on PoE2.

## Concept

In PoE2, **Spirit is a separate resource pool from mana**:

| Resource | What it does | How it changes |
|---|---|---|
| **Spirit** | **Reserves** auras and persistent skills (heralds, persistent buffs). | Builds-up at character creation; gear / passives / quest-rewards add more. |
| **Mana** | **Costs** active skill activations + channelling tick costs. | Drops when you use a skill; regenerates back. |
| **Life** (rare) | Some "blood magic"-style nodes substitute for mana. | Niche. |

The conceptual delta from PoE1: in PoE1, auras *reduced max mana* (later: *reserved %*); in PoE2, auras instead reserve a separate Spirit pool. Mana sustain (regen / leech / cost reduction) is therefore **independent** of aura budget — solving one doesn't help the other.

## Sources of Spirit (PoE2 0.4 — verify per version)

| Source | Approx. magnitude |
|---|---|
| Class base + level scaling | 60-100 base; ~+1-2 per level (varies). |
| Quest rewards | Specific Acts grant Spirit on quest completion. |
| Atlas / Atlas-tree | Small bonus per Atlas Tree node + Pinnacle drops. |
| Items (amulets, body armours) | "+X Spirit" affix on specific bases. |
| Unique items | Some uniques grant large Spirit (e.g., Astramentis-style). |
| Ascendancy nodes | Several ascendancies have Spirit-bonus paths. |
| Specific gem-effects | Some skill gems grant Spirit when used. |

**Endgame characters typically reach 200-350 Spirit total** — this is a moving target and shifts each patch. Verify against current Maxroll PoE2 build guides.

## Reservation efficiency

Modifiers like **"Reduced Spirit Reservation"** function similarly to PoE1's reservation efficiency — direct percent-decrease on the reservation cost of a specific aura/skill or all auras.

- Per-aura efficiency: e.g., "Discipline reserves 20% less Spirit".
- Global efficiency: e.g., "Reservation Efficiency of all your skills 8% increased" via passive notable.
- Stacking: efficiency sources stack in PoE2 with the same multiplicative-to-base pattern as PoE1.

## Mana sustain still matters separately

Spirit reservations don't replace mana costs. A heavy-spirit caster may still struggle with mana sustain on a high-cost spell:

- **Mana leech**: % of damage as mana, similar to PoE1.
- **Mana regen on tree / gear**: scales mana/sec.
- **"Skills cost X less mana"**: direct cost reduction.
- **Mana flask (PoE2)**: still in the game; restores chunk on use.

A character at 350 Spirit (5 auras + heralds running smoothly) may still be unable to spam a 60-mana-cost skill at high cast speed. **Ask both questions when diagnosing**: "Is your Spirit budget sufficient?" *and* "Is your mana sustain matching your skill use rate?"

## Breakpoints and aura stacks (current as of 0.4 — verify)

These are conceptual placeholders. Re-check at every patch boundary.

| Spirit budget | Typical use |
|---|---|
| 80-120 | Low budget — 1-2 auras (e.g., Vitality I + 1 Herald). |
| 150-200 | Comfortable mid-game — 3 auras + 1 herald, modest specialisation. |
| 200-280 | Endgame — 4-5 auras + 2 heralds + persistent buff. |
| 300+ | High investment — full aura/herald stack, Spirit-prerequisite uniques. |

## Aura archetypes in PoE2 (selected)

| Aura family | Spirit-cost class | Effect |
|---|---|---|
| Heralds | Medium | On-hit damage explosions (Herald of Thunder, Herald of Ice, etc.). |
| Persistent buffs | Variable | Discipline (ES recovery), Determination (armour), Vitality (life regen). |
| Specific class auras | Variable | Class-/Ascendancy-specific (e.g., Tactician Mercenary heralds). |
| Companion-tied | Build-class | Druid/Witch summon-pets reserve Spirit per minion. |

Build-defining: a Tactician Mercenary build is fundamentally a *Spirit budget allocation problem*, with heralds chosen by what damage type the build wants to layer.

## Common reservation puzzles (and how to think)

### Puzzle 1: "I want one more aura but can't fit it"

- Solution path: increase Spirit (gear / passives / amulet) OR reduce reservation efficiency (cluster jewel notable, gear mods).
- **Wrong path**: Try to reduce mana cost — different budget.

### Puzzle 2: "My heralds drop during combat"

- Diagnostic: are heralds being killed (companion-class) or de-buffed (curse stacks)?
- Solution: verify Spirit isn't being temporarily reduced by enemy-applied curse.

### Puzzle 3: "Switching weapons drops auras"

- Diagnostic: weapons can grant +Spirit; switching may drop Spirit below aura threshold.
- Solution: model both Weapon Set 1 and Weapon Set 2 Spirit totals, allocate auras to fit minimum.

### Puzzle 4: "I can run 4 heralds in PoB but only 3 in-game"

- Diagnostic: PoB may not model character-current Spirit correctly. Verify base + items.
- Solution: build to actual Spirit, not theoretical max.

## How to reason about a Spirit-budget question

1. **Compute current Spirit** from base + level + items + passives.
2. **Sum reservation costs** of allocated auras + heralds + persistent buffs.
3. **Apply reservation efficiency** modifiers (most aura-specific, some global).
4. **Compare**: if (sum after efficiency) < (current Spirit), it fits.
5. If 4 fails: either add Spirit, increase efficiency, or drop an aura.

## Cross-references

- `06_character_resources_action_speed.md` — broader resource theory.
- `07_offence_damage_scaling.md` — how heralds scale damage.
- `08_defence_recovery_survivability.md` — how persistent buffs (Determination, Vitality, Discipline) feed defense.
- `21_currency_and_barter_taxonomy.md` — Spirit-affecting affixes on items.
- `26_validation_and_self_correction.md` Rule 4 — PoE1 reservation vs PoE2 Spirit disambiguation.
- `00_version_pinning.md` — version-specific Spirit values.
- `02_weapon_sets.md` — weapon-set Spirit considerations.
