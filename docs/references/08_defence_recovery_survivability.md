# 08 — Defence, recovery, and survivability

## Principle

Survival in PoE is not a single EHP value. It is a stack of layers:

1. **Hit pool** — life, energy shield, mana-as-buffer, ward-like systems, guard buffers.
2. **Mitigation** — resistances, armour, physical damage reduction, max resist, less damage taken.
3. **Avoidance / prevention** — evasion, block, spell suppression, dodge-like mechanics, ailment avoidance.
4. **Recovery** — leech, regeneration, recoup, flasks, recharge, life gained on hit, recovery on block.
5. **Uptime and failure modes** — flask downtime, ES recharge interruption, block RNG, suppression not capped, map mods, curses, shock, exposure, reduced recovery.

The agent must look for the weakest layer, not stack generic defensive advice.

## Resistance

The PoE1 `Resistance` page states that resistance reduces or amplifies damage of the matching type, and is calculated after avoidance and before damage reduction. Implications:

- resistances are a floor, not a bonus;
- an uncapped resistance is often a bigger problem than missing life;
- chaos resistance must be analysed separately;
- max resistance changes the ceiling, not just the current value;
- enemy penetration and exposure can erode the real mitigation.

Verify the current resistance cap on the wiki for the relevant game and patch — caps have moved across patches.

## Armour

Armour must never be summarised as "% physical reduction" without context. The PoE1 and PoE2 `Armour` pages state the same critical principle: armour is more effective against small hits than against large hits, and does not apply to DoTs by default. The PoE2 page also states that mitigation depends on the size of the hit relative to the armour value and is capped — verify the current cap on the wiki.

Diagnosis: if the build has a lot of armour but a weak physical max hit, look for:

- endurance charges / additional physical damage reduction;
- physical taken as elemental / chaos;
- guard skills;
- real max hit against large hits;
- armour break / overwhelm / enemy mods if the user is on PoE2 or specific content.

## Evasion

Evasion prevents hits but does not guarantee continuous mitigation. The PoE2 Wiki indicates that evasion uses an evasion-rating-against-accuracy system, and that chance to evade has a hard cap (verify the current cap on the wiki). For reasoning:

- evasion is strong against many small avoidable hits;
- it does not solve hits that pass through;
- it must be paired with hit pool, suppression / block, or mitigation;
- it is not a linear EHP equivalent.

## Spell suppression (PoE1)

The `Spell suppression` page states that characters have no base chance, that suppression stacks up to its cap, and that the spell damage prevented at the cap is fixed (verify the current value). So:

- partial suppression is not "almost the same" as capped suppression for a build that depends on the guarantee;
- an evasion build without suppression can die to spell hits;
- `Prevent +#% of Suppressed Spell Damage` changes the quality of the layer;
- damaging ailments coming from spell hits may also be affected per the suppression rules in the wiki.

## Block

Block usually prevents the damage of a blocked hit, except for specific exceptions. But block is RNG without a guarantee mechanic. The diagnosis must look at:

- attack block vs spell block;
- current block chance and cap;
- recovery on block;
- glancing / partial block mechanics if present;
- non-blockable hits or DoTs.

## Energy Shield

Energy Shield can be a primary, secondary, or useless hit pool depending on the mechanics. In PoE2, the `Energy Shield` page states that ES acts as additional hit points that recharge, but also that bleeding / poison bypass ES and that chaos damage deals twice as much damage against ES. For PoE1, the chaos / ES rules differ depending on mechanics and keystones; verify the game before advising.

## Recovery

A build can have a good max hit and still die from missing recovery. Read:

- life regen;
- leech rate and leech cap;
- life / mana gained on hit;
- recoup;
- flask uptime;
- ES recharge start / delay;
- recovery disabled or reduced by map mods;
- recovery conditional on kill vs on hit.

## Ailment and debuff defence

Always verify:

- freeze / chill;
- shock;
- ignite;
- bleed / corrupted blood;
- poison;
- stun;
- curse / exposure;
- maim / hinder / slow;
- armour break or other PoE2 debuffs;
- crit mitigation if relevant.

A build with capped resistances can die very fast under shock, reduced recovery, corrupted blood, or freeze.

## Build death diagnostic checklist

```text
1. What killed the player? hit / DoT / ailment / degen ground / burst / boss mechanic / map mod.
2. Which damage type is likely? physical / fire / cold / lightning / chaos / mixed.
3. Current max hit by type?
4. Current recovery under boss conditions?
5. Current avoidance reliability? evasion / block / suppression / dodge.
6. Ailment and debuff coverage?
7. Map / content modifiers that invalidate a layer?
8. Is the user in HC / SSF / trade? Risk tolerance changes advice.
```

## Upgrade priority for weak defence

1. Cap elemental resistances.
2. Fix chaos resistance if chaos max hit is the lowest, or if content applies chaos / poison.
3. Raise hit pool on weak slots.
4. Add one reliable mitigation / avoidance layer appropriate to the archetype.
5. Add ailment / stun / corrupted blood protection.
6. Improve recovery for boss uptime, not only mapping.
7. Only then chase luxury layers like max-res stacking or rare uniques.

## Useful sources

- [PoE1 Resistance](https://www.poewiki.net/wiki/Resistance)
- [PoE1 Armour](https://www.poewiki.net/wiki/Armour)
- [PoE1 Spell suppression](https://www.poewiki.net/wiki/Spell_suppression)
- [PoE2 Armour](https://www.poe2wiki.net/wiki/Armour)
- [PoE2 Evasion](https://www.poe2wiki.net/wiki/Evasion)
- [PoE2 Spirit](https://www.poe2wiki.net/wiki/Spirit)
- [PoE2 Wiki mechanics index](https://www.poe2wiki.net/wiki/Path_of_Exile_2_Wiki)
