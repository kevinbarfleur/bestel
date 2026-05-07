---
description: Damage delivery first, scaling axes table, enemy mitigation, conversion warnings, DPS-low diagnosis playbook.
fetch_when: For DPS issues, "why is my damage low", scaling questions, support gem choice, or conversion mechanics.
---

# 07 — Offence and damage scaling

## Principle

A viable offensive build answers three questions:

1. **How is damage delivered?** hit, DoT, ailment, minion, trigger, totem, projectile, melee, AoE.
2. **What actually scales the damage?** base damage, added damage, gem level, weapon DPS, more multipliers, attack / cast speed, crit, DoT multiplier, conversion, penetration, exposure, curses.
3. **What limits real DPS?** uptime, accuracy, mana, cooldown, animation, range, boss movement, ailment threshold, enemy resistance, defensive map mods.

## Damage delivery first

The first error to avoid: hunting "good supports" without knowing what deals the damage. Examples:

- a hit attack scaling weapon DPS has different needs from a spell scaling gem level;
- an ailment build can be limited by hit size, ailment chance, ailment effect, or duration;
- a DoT build typically wants gem level, DoT multiplier, duration / uptime, and enemy resistance reduction;
- a minion build depends on a different entity from the player;
- a trigger build depends on cost, cooldown, chance / proc rate, and skill order.

## Skill package

For each main skill, the agent must extract:

```text
Skill name:
Game:
Gem level / quality:
Tags:
Damage source:
Damage types:
Hit or DoT:
Base damage source: weapon / gem / minion / corpse / other
Supports:
Single-target support setup:
Clear support setup:
Resource cost:
Cooldown / trigger / reservation:
```

## Supports

In PoE1, supports must be linked to the skill via the equipment. The `Support gem` page states that a support modifies any compatible skill gem it is linked to, and that tags alone don't determine supportability.

In PoE2, sockets sit on the skill gems themselves: the `Gem socket` page states that skill gems can host support gems, with 2 to 5 sockets expandable via Jeweller's Orbs. The agent must therefore not talk about "6-link chest" PoE2 the way it would in PoE1 without clarifying the context.

## Scaling axes

| Axis | To check | Sources |
|---|---|---|
| Base damage | Weapon DPS, gem level, added damage, minion base, corpse life, skill effectiveness. | Wiki skill, PoEDB / PoE2DB gem. |
| Multipliers | Supports, more / less, ascendancy, keystone, conditional buffs. | Wiki, PoB. |
| Speed | Attack / cast speed, cooldown recovery, trigger rate, animation lock. | Wiki skill / support, PoB calc. |
| Crit | Base crit, increased crit, crit multi, lucky, brittle / ailments, uptime. | Wiki mechanics, PoB. |
| Conversion | Damage type conversion order and final damage type. | Wiki, GGG developer posts only if still relevant and cross-checked. |
| Enemy mitigation | Resistance, exposure, curse, penetration, armour, overwhelm, armour break. | Wiki resistance / armour, PoB enemy config. |
| Ailments | Chance, effect, threshold, duration, source hit. | Wiki ailment pages, skill pages. |

## Enemy resistance and mitigation

Enemy resistance handling is a core offensive layer. The player may have high tooltip DPS but poor real damage if the build lacks penetration, exposure, curse, resistance reduction, or the correct enemy config in PoB.

Agent checklist:

```text
1. What damage type is actually dealt after conversion?
2. What enemy resistance is assumed in PoB?
3. Does the build apply exposure / curse / penetration reliably?
4. Does the content target resist or reduce that type?
5. Are map mods disabling curse effect or adding resistance?
```

## Conversion and "damage taken as"

Conversion and "damage taken as" are frequent sources of wrong answers. Treat them as high-risk mechanics. Always verify exact wording and order. Search the official wiki first; if a specific historical forum explanation is used, verify it has not been superseded.

Do not collapse:

- damage conversion by the attacker;
- damage taken as by the defender;
- gain as extra;
- added as;
- penetration;
- resistance reduction;
- armour applying to non-physical damage.

## Offence diagnosis playbook

When a user says "my DPS is low":

1. Read main skill, level, quality, supports, weapon / base / gem level.
2. Read PoB config and active buffs.
3. Identify whether the skill is hit, DoT, ailment, or minion.
4. Check whether supports actually apply.
5. Check weapon / base / gem level against the player's stage.
6. Check enemy mitigation layers.
7. Find the first upgrade with the largest multiplier per cost.

## Example outputs

Bad:

> Use better supports and get more damage.

Good:

> Your problem isn't generic damage; it's base damage. This attack scales from weapon DPS, and your weapon has no high local physical / elemental roll. Before changing the tree, search a weapon base with a high local `% increased Physical Damage`, flat added damage, and attack speed. Then re-check accuracy and mana cost, because a faster weapon can expose sustain problems.

## Useful sources

- [PoE1 Support gem](https://www.poewiki.net/wiki/Support_gem)
- [PoE2 Gem](https://www.poe2wiki.net/wiki/Gem)
- [PoE2 Gem socket](https://www.poe2wiki.net/wiki/Gem_socket)
- [PoE1 Resistance](https://www.poewiki.net/wiki/Resistance)
- [PoE1 Armour](https://www.poewiki.net/wiki/Armour)
- [PoE2 Armour](https://www.poe2wiki.net/wiki/Armour)
- [PoEDB modifiers](https://poedb.tw/us/Modifiers)
- [PoE2DB gems](https://poe2db.tw/us/Gem)
