---
description: PoE2 Spirit budget concept, mana sustain interaction. Spirit reserves auras and persistent skills (heralds, persistent buffs); mana still costs skill activations. PoE2-only mechanic; never apply to PoE1.
fetch_when: User asks about Spirit (PoE2 context), "how do I afford another aura?", a PoE2 reservation puzzle, or compares with PoE1's reservation system. Always apply to PoE2 questions only — never to PoE1.
---

# PoE2 — Spirit economy

> **No numeric values are stored in this file by design.** Spirit budgets, breakpoints, item magnitudes, and reservation costs are patch-volatile. For any quantitative claim about Spirit, fetch `wiki_parse https://www.poe2wiki.net/wiki/Spirit` or `repoe_lookup` for items granting Spirit. Always check `00_version_pinning.md` for the active PoE2 version before answering.

This doc is about **Spirit as a separate budget from mana**, the most distinctive resource mechanic in PoE2 vs PoE1. Conflating Spirit with mana / reservation is one of the most common LLM hallucinations on PoE2.

## Concept

In PoE2, **Spirit is a separate resource pool from mana**:

| Resource | What it does | How it changes |
|---|---|---|
| **Spirit** | **Reserves** auras and persistent skills (heralds, persistent buffs). | Built into the character at creation; gear / passives / quest rewards add more. |
| **Mana** | **Costs** active skill activations + channelling tick costs. | Drops when you use a skill; regenerates back. |
| **Life** (rare) | Some "blood magic"-style nodes substitute for mana. | Niche. |

The conceptual delta from PoE1: in PoE1, auras *reduced max mana* (later: *reserved %*); in PoE2, auras instead reserve a separate Spirit pool. Mana sustain (regen / leech / cost reduction) is therefore **independent** of aura budget — solving one doesn't help the other.

## Sources of Spirit (categorical)

| Source family | Where to fetch the actual magnitude |
|---|---|
| Class base + level scaling | poe2wiki.net/wiki/Spirit (base table) |
| Quest rewards (Acts) | Per-Act quest pages on the PoE2 wiki |
| Atlas tree nodes | `wiki_parse` on the current Atlas Tree page |
| Items (amulets, body armours, sceptres) | `repoe_lookup category=base_items` + `category=mods` for Spirit affixes |
| Unique items | `repoe_lookup category=uniques` — Spirit-granting uniques live here |
| Ascendancy paths | The ascendancy's own wiki page |
| Specific gem effects | The gem's wiki page |

**Endgame Spirit totals shift each patch.** When a user asks "is X auras feasible at level N?", always fetch the current calibration from a recent Maxroll PoE2 build guide rather than reciting a remembered figure.

## Reservation efficiency

Modifiers like **"Reduced Spirit Reservation"** function similarly to PoE1's reservation efficiency — direct percent-decrease on the reservation cost of a specific aura/skill or all auras.

- **Per-aura efficiency**: aura-specific reservation reductions (gem quality, support gem mods, item affixes naming the aura).
- **Global efficiency**: tree-wide or item-wide reservation modifiers (passive notables, cluster jewels, generic gear affixes).
- **Stacking**: efficiency sources stack with the same multiplicative-to-base pattern as PoE1.

For the exact percent values per source, fetch the wiki page of the specific modifier or use `repoe_lookup` against the mod name.

## Mana sustain still matters separately

Spirit reservations don't replace mana costs. A heavy-Spirit caster may still struggle with mana sustain on a high-cost spell:

- **Mana leech**: % of damage as mana, similar to PoE1.
- **Mana regen on tree / gear**: scales mana/sec.
- **"Skills cost X less mana"**: direct cost reduction.
- **Mana flask (PoE2)**: still in the game; restores a chunk on use.

A character with a full aura/herald stack may still be unable to spam a high-mana-cost skill at high cast speed. **Ask both questions when diagnosing**: "Is your Spirit budget sufficient?" *and* "Is your mana sustain matching your skill use rate?"

## Aura archetypes in PoE2 (categorical)

| Aura family | Spirit-cost class | Effect |
|---|---|---|
| Heralds | Medium | On-hit damage explosions (Herald of Thunder, Herald of Ice, etc.). |
| Persistent buffs | Variable | Discipline (ES recovery), Determination (armour), Vitality (life regen). |
| Specific class auras | Variable | Class-/Ascendancy-specific (e.g., Tactician Mercenary heralds). |
| Companion-tied | Build-class | Druid/Witch summon-pets reserve Spirit per minion. |

Build-defining example: a Tactician Mercenary build is fundamentally a *Spirit budget allocation problem*, with heralds chosen by what damage type the build wants to layer.

## Common reservation puzzles (and how to think)

### Puzzle 1: "I want one more aura but can't fit it"

- Solution path: increase Spirit (gear / passives / amulet) OR reduce reservation efficiency (cluster jewel notable, gear mods).
- **Wrong path**: try to reduce mana cost — different budget.

### Puzzle 2: "My heralds drop during combat"

- Diagnostic: are heralds being killed (companion-class) or de-buffed (curse stacks)?
- Solution: verify Spirit isn't being temporarily reduced by enemy-applied curse.

### Puzzle 3: "Switching weapons drops auras"

- Diagnostic: weapons can grant +Spirit; switching may drop Spirit below aura threshold.
- Solution: model both Weapon Set 1 and Weapon Set 2 Spirit totals, allocate auras to fit minimum.

### Puzzle 4: "I can run more heralds in PoB than in-game"

- Diagnostic: PoB may not model character-current Spirit correctly. Verify base + items.
- Solution: build to actual Spirit, not theoretical max.

## How to reason about a Spirit-budget question

1. **Compute current Spirit** from base + level + items + passives — fetch each source magnitude live.
2. **Sum reservation costs** of allocated auras + heralds + persistent buffs — fetch each cost live.
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
