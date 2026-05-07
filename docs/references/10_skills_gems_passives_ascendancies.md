---
description: Reasoning order (skill->delivery->supports->tree->ascendancy->items), PoE1 vs PoE2 gem systems, keystone evaluation, synergy sweep.
fetch_when: For skill choice, gem-link questions, passive-tree planning, ascendancy comparison, keystone/synergy queries.
---

# 10 — Skills, gems, passives, and ascendancies

## Skill first, class second

In PoE the class is not the build. The class gives a starting point, attributes, and an ascendancy, but the skill and its mechanics define most of the constraints.

Reasoning order:

```text
Skill -> delivery type -> scaling -> support package -> resource cost -> passive tree -> ascendancy -> items
```

Don't start with "this ascendancy is good" without knowing what the skill wants.

## PoE1 gems

PoE1 uses skill gems socketed in equipment. Supports must be linked to skills. Important questions:

- is the skill an attack, spell, minion, projectile, AoE, melee, duration, or channelled skill?
- is the support compatible by exact text, not only by tag?
- does the build have a clear setup and a boss setup?
- is the body / weapon 6-link a progression constraint?
- does the gem quality / transfigured / vaal / awakened version change the skill's identity?

## PoE2 gems

PoE2 reasons differently: skill gems live in the Skills Panel and the sockets sit on the gem itself. The `Gem socket` page states that skill gems can have 2 to 5 sockets, expandable via Jeweller's Orbs. The `Gem` page states that gems can grant a skill or modify a skill, and includes skill gems, support gems, meta gems, and spirit gems.

For the agent:

- avoid PoE1 terminology like "6-link chest" unless explicitly comparing;
- analyse the sockets on the skill itself;
- check support restrictions and tier / lineage supports;
- check `Spirit` for persistent buffs, permanent minions, and trigger / meta gems;
- check weapon-set behavior if the skill depends on a weapon or on a buff tied to a set.

## Passive tree

The PoE1 `Passive skill` page describes a large network where classes start at different points. The PoE2 `Passive Skill Tree` page also describes a central tree with small passives, notables, jewel sockets, keystones, and attribute zones.

The agent must understand that the passive tree is used to:

- connect the build to useful clusters;
- pick up efficient stats;
- solve attributes / resistances / resources where needed;
- access keystones;
- place jewels;
- optimise pathing.

A tree is bad if its pathing costs more than it gains, even if every individual node is "good" in isolation.

## Keystones

A keystone changes a rule of the game in exchange for a cost. Every keystone recommendation must include:

1. exact text;
2. benefit;
3. drawback;
4. condition for the benefit to outweigh the drawback;
5. acquisition / pathing;
6. synergies;
7. anti-synergies;
8. PoB test.

Example reasoning: a keystone that converts incoming damage or alters a resource can only be strong if defensive layers and recovery have been built around it.

## Ascendancies

An ascendancy isn't only "more power." It creates mechanical promises:

- source of damage scaling;
- source of sustain;
- source of mitigation;
- resource change;
- trigger or special skill;
- unique interaction with a mechanic;
- economic pathing into specific clusters.

For each proposed ascendancy node, the agent must verify:

```text
Does it solve the user's current problem?
Does the build meet the condition?
Is uptime realistic in bossing?
Does it conflict with an item / passive / support?
What gets cut to allocate it?
```

## Synergy sweep

The value of a specialised agent often comes from connecting sources. After identifying a mechanic, the agent must look for entities that interact with it:

- wiki page backlinks / `Special:WhatLinksHere`;
- PoEDB tags;
- items granting the mechanic;
- passives / notables / cluster jewels;
- support gems;
- uniques with "damage taken as", "counts as", "gain as", "while affected by", "per charge", "reservation", "maximum resistance".

Don't surface every synergy. Surface the 2 to 4 that are mechanically relevant for the build and the budget.

## Passive / gem search templates

```text
site:poewiki.net/wiki "<skill name>"
site:poewiki.net/wiki "<support name>"
site:poe2wiki.net/wiki "<skill name>"
site:poe2wiki.net/wiki "<support name>"
site:poedb.tw/us "<skill name>" "Level"
site:poe2db.tw/us/Gem "<skill name>"
site:poewiki.net/wiki "<keystone name>"
site:poe2wiki.net/wiki "<keystone name>"
```

## Useful sources

- [PoE1 Support gem](https://www.poewiki.net/wiki/Support_gem)
- [PoE1 Passive skill](https://www.poewiki.net/wiki/Passive_skill)
- [PoE2 Gem](https://www.poe2wiki.net/wiki/Gem)
- [PoE2 Gem socket](https://www.poe2wiki.net/wiki/Gem_socket)
- [PoE2 Spirit](https://www.poe2wiki.net/wiki/Spirit)
- [PoE2 Passive Skill Tree](https://www.poe2wiki.net/wiki/Passive_skill_tree)
- [PoE2 Ascendancy class](https://www.poe2wiki.net/wiki/Ascendancy_class)
- [PoE2DB Gems](https://poe2db.tw/us/Gem)
