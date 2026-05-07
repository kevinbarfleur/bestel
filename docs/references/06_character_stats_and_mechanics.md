---
description: 6-axis stat classification, local vs global, more vs increased, source/type/delivery, charges/buffs/uptime, ailments.
fetch_when: When parsing any unfamiliar mod text, debugging stat interactions, or reasoning about ailments/charges/buffs.
---

# 06 — Player, character stats, and mechanical vocabulary

## The stat axes

Most analysis errors come from mis-classifying a stat. Before computing, the agent must classify:

| Axis | Questions to ask |
|---|---|
| Source | Does the stat come from an item, passive, support, gem level, buff, flask, ascendancy, map mod, enemy debuff? |
| Scope | Local to the item, or global to the character? |
| Type | Increased / reduced, more / less, added, converted, taken as, penetration, resistance reduction, chance, cap, multiplier? |
| Target | Does it affect the player, minions, allies, enemies, the skill, the item, the hit, the ailment, or the DoT? |
| Condition | Always on, on hit, while stationary, if you've crit recently, against rare / unique, with a charge, during flask effect? |
| Time scale | Instant, duration, cooldown, uptime, ramping, snapshot / non-snapshot? |

## Local vs global

A local stat usually modifies the values of the item it sits on. Typical example: `% increased Physical Damage` on a weapon modifies that weapon's physical damage. A global stat modifies the character or a category of skills. The practical issue: two visually similar mod texts can have different effects depending on where they sit.

Agent rule: if the recommendation depends on local vs global, verify the wiki page or PoEDB. Never infer from wording alone.

## Increased / reduced vs more / less

General principle:

- `increased` and `reduced` add together within their applicable pool;
- `more` and `less` are multiplicative against any other applicable modifiers;
- a conditional stat only exists when its condition is true;
- a stat that doesn't apply to the right delivery type is wasted.

Reflex: when a player asks "does this support scale my skill?", verify compatibility and tags, but also the exact text of the support gem. The PoE1 `Support gem` page reminds that tags alone don't determine supportability.

## Damage source, damage type, delivery type

Three categories that are often conflated:

- **Damage source** — attack, spell, secondary, minion, totem, trap / mine, damage over time.
- **Damage type** — physical, fire, cold, lightning, chaos.
- **Delivery type** — hit, ailment, ground DoT, projectile, melee, AoE, channelled, trigger.

A mod can apply to a type but not to a source, or to a source but not to a delivery type. Conceptual example: `increased Attack Damage` does not mean "everything from an attack" in all cases; for ailments and DoTs, verify the exact mechanic.

## Tags

Tags are useful for searching, but not always sufficient. Use them to:

- identify candidate supports;
- find related clusters / passives / items;
- search PoEDB / PoE2DB;
- understand likely scalings.

But always verify actual compatibility. The exact gem / support text and the explicit restrictions take priority over tags.

## Resources

### PoE1

Frequent resources and constraints: mana cost, life cost, reservation, flask charges, charges, rage, energy shield, cooldown, trigger cost. Auras and reservations can consume as much defensive / offensive headroom as an item slot.

### PoE2

PoE2 gives `Spirit` a central role: the PoE2 Wiki page defines it as a reserve used to activate and sustain skills with persistent effects. It also describes interaction with weapon sets, sceptres, persistent skills, buffs, permanent minions, and meta / trigger gems. Searching `Spirit` should be a reflex for any PoE2 build with auras, minions, triggers, or persistent buffs.

## Charges, buffs, and uptime

Charges and buffs are decision multipliers. Distinguish:

- `current` vs `max` charges in the build;
- reliable generation vs theoretical generation;
- boss uptime vs mapping uptime;
- charge on kill vs charge on hit / crit;
- permanent vs temporary buffs;
- PoB conditions toggled manually.

A DPS recommendation that assumes Onslaught, full rage, strong shock, exposure, curses, flasks, and permanent charges should be flagged as config-dependent.

## Ailments

Ailments are effects tied to damage types and to specific hits or conditions. Never assume a "fire" build ignites correctly, or that a "cold" build freezes correctly. Verify:

- chance to apply;
- hit damage or threshold;
- ailment effect;
- ailment duration;
- enemy ailment threshold;
- applicable scaling;
- immunities / reductions of the content target.

## Practical stat checklist

Before answering a mechanical question:

```text
1. What is the exact stat text?
2. Where does the stat appear? item, support, passive, buff, enemy debuff?
3. Local or global?
4. Which type / source / delivery does it affect?
5. Is there a condition?
6. Does the player's skill satisfy that condition?
7. Does the PoB / config show the condition active?
8. Does the stat exist in PoE1, PoE2, or both?
```

## Useful sources

- [PoE1 Support gem](https://www.poewiki.net/wiki/Support_gem)
- [PoE2 Gem](https://www.poe2wiki.net/wiki/Gem)
- [PoE2 Spirit](https://www.poe2wiki.net/wiki/Spirit)
- [PoE1 Resistance](https://www.poewiki.net/wiki/Resistance)
- [PoE2 Modifier](https://www.poe2wiki.net/wiki/Modifier)
- [PoEDB modifiers](https://poedb.tw/us/Modifiers)
- [PoE2DB modifiers](https://poe2db.tw/us/Modifiers)
