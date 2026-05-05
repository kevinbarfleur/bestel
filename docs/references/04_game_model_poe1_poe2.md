# 04 — Game model: PoE1 and PoE2

## Shared model

PoE1 and PoE2 are action RPGs built around the same conceptual core:

- a character with a class, attributes, resources, and defences;
- skills that define the way of killing;
- supports that modify skills;
- a passive tree that grants stats, notables, and keystones;
- an ascendancy that gives strong mechanical identity;
- items that carry most of the build's power;
- a replayable endgame;
- a trade economy where availability and prices shift with the league.

The mistake to avoid: treating PoE as a list of items. A build is a **system of constraints** — damage, survival, recovery, resources, mobility, content target, budget, patch.

## Structural differences

| Dimension | PoE1 | PoE2 |
|---|---|---|
| Status | Mature game, league patches, systems stacked over many years. | Early Access; systems are less stable and can change rapidly. |
| Skills / supports | Skill gems socketed in equipment; support gems linked through item sockets / links. The `Support gem` page states that supports modify any compatible linked skill. | Skill gems live in the Skills Panel; support sockets are on the gems themselves. The `Gem socket` page states that skill gems can have 2 to 5 sockets, expandable via Jeweller's Orbs. |
| Reservation resource | Mana / life reservation historically central, with auras and reservations. | `Spirit` is a dedicated resource for persistent skills, buffs, permanent minions, and trigger / meta gems. |
| Passive tree | Shared tree where classes start at different points. | Tree is also large, but with weapon specialisation, configurable attribute travel nodes, and explicit interaction with weapon sets. |
| Ascendancy | Unlocked via Labyrinth and tied to the base class. | Unlocked via Ascension Trials; the PoE2 Wiki page mentions multiple trial tiers up to a maximum number of ascendancy points. |
| Itemisation | Very mature: influences, essences, fossils, Harvest, eldritch implicits, metamods, fractures, synthesis, corruptions, bench. | Newer: currency / crafting, runes, soul cores, desecrated modifiers, Vaal / cultivated systems depending on patch — verify per case. |
| Endgame | Maps as items, Atlas of Worlds, atlas tree, bosses, fragments, scarabs, league mechanics. | Infinite Atlas, waystones, map nodes, towers, precursor tablets, citadels, PoE2 league mechanics. |
| Economy | Heavily documented through official trade, poe.ninja, community tooling. | Trade2 / official site and emerging tooling; high variability due to Early Access. |

## PoE1 mental model

PoE1 is a game of **historical layers**. When a user asks a PoE1 question, the agent should ask itself:

1. Does this mechanic come from the core game, an integrated league, a specific crafting system, or an older patch?
2. Is the item league-specific, boss drop, drop-restricted, legacy, or permanent-league-only?
3. Does the build hinge on a tag, a threshold, a breakpoint, a conversion interaction, a unique, or a keystone?
4. Is the PoB configured correctly?

PoE1 rewards precise answers. *"Get more life"* is almost always insufficient. The slot, the stat, the tier, the crafting route, or the trade filter must be named.

## PoE2 mental model

Treat PoE2 as a sibling game, not a PoE1 reskin. Common concepts exist, but exact systems can differ: gem sockets, Spirit, Atlas, item bases, ascendancies, support restrictions, defences, skill balance.

For PoE2, the agent must be stricter:

- check the PoE2 Wiki page, not the PoE1 page;
- check Early Access patch notes;
- check PoE2DB for raw data;
- avoid importing PoE1 conclusions without confirmation;
- flag pages that are stubs, recent, or potentially incomplete.

## Mandatory pre-question framing

Before reasoning, the agent must mentally fill:

```text
Game: PoE1 / PoE2 / unknown
Patch / league: known / unknown / must verify
Player stage: campaign / early maps / red maps / bosses / farming / SSF / trade
Core entity: skill / item / mechanic / build slot / endgame system
Decision needed: explanation / diagnosis / upgrade / craft / trade / route / comparison
```

If `Game` is unknown and the wrong choice would change the answer, the agent must ask, or explicitly present both branches.

## Cross-game traps (most common confusions)

A handful of words exist in both games but mean different things. Resolve game first, then map the term:

| Word | PoE1 | PoE2 |
|---|---|---|
| **Chaos Orb** | Full reroll of all explicit mods on a Rare. | Removes one random mod and adds one new mod. |
| **Exalted Orb** | Adds one mod; high-rarity currency. | Adds one mod; common-tier currency (the role Chaos played in PoE1). |
| **Map** | Endgame consumable map item. | Conceptually the "Waystone" — verify the user is not on PoE2. |
| **Witch / Ranger / Templar / Marauder** | PoE1 class with PoE1 ascendancies. | PoE2 class with different ascendancies, starting position, mechanics. |
| **Atlas** | Fixed quadrant structure with discrete map nodes. | Procedural infinite endgame (subject to 0.5 rework). |
| **6-link** | Endgame goal on chest / 2H. | Does not exist; PoE2 supports go inside the gem itself. |
| **Spirit** | Does not exist. | Central reservation resource. |
| **Weapon Swap** | Existed but rarely used in practice. | Core mechanic, dual-spec. |

## Useful sources

- [PoE Wiki main page](https://www.poewiki.net/wiki/Path_of_Exile_Wiki)
- [PoE2 Wiki main page](https://www.poe2wiki.net/wiki/Path_of_Exile_2_Wiki)
- [PoE2 Gem socket](https://www.poe2wiki.net/wiki/Gem_socket)
- [PoE2 Spirit](https://www.poe2wiki.net/wiki/Spirit)
- [PoE2 Passive Skill Tree](https://www.poe2wiki.net/wiki/Passive_skill_tree)
- [PoE2 Ascendancy class](https://www.poe2wiki.net/wiki/Ascendancy_class)
- [PoE1 Support gem](https://www.poewiki.net/wiki/Support_gem)
- [PoE1 Atlas of Worlds](https://www.poewiki.net/wiki/Atlas_of_Worlds)
- [PoE2 Atlas](https://www.poe2wiki.net/wiki/Atlas)
