# 09 — Itemisation, crafting, and upgrade logic

## Item = base + rarity + item level + affixes + implicits + constraints

A PoE item must be analysed as a structure:

```text
Base type:
Item class:
Item level:
Rarity:
Implicit(s):
Explicit prefixes:
Explicit suffixes:
Quality:
Influence / eldritch / fracture / synth / corruption / enchant / rune / soul core / desecrated:
Socket / gem implications:
Required attributes:
Build-critical mods:
Replaceable mods:
```

The agent must not say "change your helmet" without first reading what the slot already solves: resistances, attributes, life / ES, suppression, eldritch implicit, reservation, gem levels, enchant, unique power.

## Rarity and affix budget

PoE1: the `Rarity` page states that rares have at most six affixes, up to three prefixes and three suffixes. The `Item level` page states that item level mainly determines which affixes are available.

PoE2: the `Modifier` page states that magic items can have up to two modifiers (one prefix, one suffix), and rare items up to six modifiers (three prefixes, three suffixes). The `Item` page also covers the role of rarity in item generation and crafting actions.

Agent rule: as soon as a craft or trade route depends on a stat, verify whether it is prefix / suffix, its tier, its required item level, its tags, and its restrictions.

## Slot pressure

Each slot has typical roles:

| Slot | Common roles | Questions |
|---|---|---|
| Weapon | Base damage, attack / cast speed, gem levels, crit, trigger, minion stats. | Does the skill scale weapon DPS or gem level? |
| Shield / off-hand / quiver | Defence, block, spell suppression, damage, reservation / Spirit, special mechanics. | Is the slot free or build-defining? |
| Helmet | Life / ES, res, reservation, ailment, eldritch implicit, gem levels / minion. | Can we lose the enchant / implicit / unique? |
| Body armour | Hit pool, armour / evasion / ES, suppression, max res, unique keystone mechanics, links in PoE1. | Is the body core or just stats? |
| Gloves | Damage conversion, attack / cast speed, suppression, exposure / implicit, life / res. | Is the glove implicit important? |
| Boots | Movement speed, res, ailment avoidance, suppression, action speed. | Can we sacrifice movement speed? |
| Rings | Resistances, chaos res, attributes, life, damage, curse, mana cost, min charges. | Which ring is the least loaded? |
| Amulet | Gem levels, attributes, life / ES, crit, DoT multi, anoint / instill-like systems. | Does the amulet carry the main breakpoint? |
| Belt | Life, res, flask mods, strength, cooldown, abyss jewel, utility. | Does the belt solve flasks / recovery? |
| Jewels | Efficient stats, corruptions, cluster / timeless / radius effects. | Is the jewel generic or build-defining? |

## Rare vs unique

A unique is not "better" by default. It is correct if its unique mechanic compensates for the loss of rare-affix budget. The agent must ask:

1. What problem does this unique solve that rares can't?
2. What slot pressure does it create?
3. What resistances / attributes / life / ES does the player lose?
4. Does it exist in the right game and current league?
5. Is it available at a realistic price?

## Crafting mental model

Crafting = shrinking the space of possibilities until you reach the required mods at acceptable cost.

Workflow:

1. Define the minimum-viable target item.
2. Choose the base and item level.
3. Identify mandatory prefixes / suffixes.
4. Identify "nice to have" mods.
5. Verify the mod pools on PoEDB / PoE2DB.
6. Choose a method: essence, fossil, harvest, bench / metamod, eldritch, fracture, alteration / regal, chaos spam, trade.
7. Simulate if the cost is non-trivial.
8. Compare against trade prices.

## PoE1 crafting routes

PoE1 has many layers:

- `Essence` — guarantees an affix on upgrade / reroll depending on type and tier.
- `Fossil` — uses resonators and biases mod weights / tags.
- `Harvest crafting` — can guarantee certain tags depending on the available craft.
- `Metamod` — changes the rules of crafting (e.g. *Prefixes Cannot Be Changed*, multimod).
- `Eldritch implicit` — adds / replaces implicits on certain item types.
- `Fractured`, `Synthesised`, `Influenced`, `Corrupted` — each state strongly constrains what is possible.

Never propose a route without verifying the restrictions. Examples: fossils and metamods can have exclusions; eldritch implicits don't behave like traditional influences; corruption limits modifications.

## PoE2 crafting routes

PoE2 crafting must be treated separately. The PoE2 `Crafting` page explains that crafting modifies items / modifiers through currencies or mechanics, with exceptions for corrupted / mirrored items. The PoE2DB `Modifiers` page lists current currencies and mods. For PoE2, always verify:

- does the currency still exist and behave this way at the current patch?
- is the mod prefix or suffix?
- is the base compatible?
- does rune / soul core / desecrated / cultivated / corrupted status block the route?
- could the item simply be bought cheaper?

## Upgrade logic

To recommend an item upgrade:

```text
1. Identify the weakest slot by opportunity cost.
2. Preserve build-enabling stats first.
3. Add the missing defensive floor: life / ES, res, chaos, suppression / block, movement speed.
4. Add an offensive breakpoint: weapon DPS, gem levels, DoT multi, crit, attack / cast speed.
5. Check attributes and resource cost.
6. Give a trade search or craft route.
```

## Trade vs craft decision

| Situation | Prefer trade | Prefer craft |
|---|---|---|
| Common stats | Yes, especially early / mid league. | Only if SSF or extremely low budget. |
| Specific fractured base | Trade the base, then craft. | If the base is farmable or SSF. |
| Multi-mod rare with exact tiers | Often trade if available. | Craft if a semi-deterministic method exists. |
| League start | Trade may be expensive / scarce. | Simple essence / bench crafts can win out. |
| SSF | Trade impossible. | Farm + craft route is mandatory. |
| PoE2 Early Access | Trade availability is variable. | Verify currencies and mod pools at the current patch. |

## Useful sources

- [PoE1 Rarity](https://www.poewiki.net/wiki/Rarity)
- [PoE1 Item level](https://www.poewiki.net/wiki/Item_level)
- [PoE1 Crafting](https://www.poewiki.net/wiki/Crafting)
- [PoE1 Metamod](https://www.poewiki.net/wiki/Metamod)
- [PoE1 Essence](https://www.poewiki.net/wiki/Essence)
- [PoE1 Fossil](https://www.poewiki.net/wiki/Fossil)
- [PoE2 Modifier](https://www.poe2wiki.net/wiki/Modifier)
- [PoE2 Item](https://www.poe2wiki.net/wiki/Item)
- [PoE2 Crafting](https://www.poe2wiki.net/wiki/Crafting)
- [PoEDB modifiers](https://poedb.tw/us/Modifiers)
- [PoE2DB modifiers](https://poe2db.tw/us/Modifiers)
- [Craft of Exile](https://www.craftofexile.com/)
