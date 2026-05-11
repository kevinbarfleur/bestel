---
description: Cross-game / PoE1-only / PoE2-only vocabulary partitioned to prevent leakage. Acronyms, mechanics, ailments, items, modes.
fetch_when: When the user uses jargon (CWDT, CoC, MoM, PF, RF, IIQ, Sekhemas, Realmgate, Spirit, Voidstone, Cluster Jewel, etc).
---

# 12 — Vocabulary glossary (PoE1 + PoE2)

This is the agent's quick lookup for jargon and acronyms. Concept-only: meanings, not current values. Whenever a value (cap, multiplier, level, price) might be relevant, the agent must verify on the wiki for the current patch.

The glossary is split into three layers:
1. **Cross-game** — terms with shared meaning in PoE1 and PoE2.
2. **PoE1-specific** — terms that only exist in PoE1, or whose meaning would mislead a PoE2 user.
3. **PoE2-specific** — terms introduced or redefined by PoE2.

Whenever a user uses an ambiguous term that diverges between the two games (Chaos Orb, Map, Witch, Atlas, …), the agent must resolve the game first and then map the term to the correct definition.

## 1. Cross-game vocabulary

### Damage and scaling

| Term | Meaning |
|---|---|
| **Hit** | Single direct damage instance. Triggers on-hit effects. |
| **DoT (Damage over Time)** | Damage per second over a duration. Cannot be evaded; scaled by DoT multipliers. |
| **Ailment** | Effect triggered by a hit, follows its own scaling rules. Damaging ailments (Ignite, Poison, Bleed) and non-damaging ailments (Chill, Freeze, Shock + variants by game). |
| **Debuff** | Negative effect without an associated damage type (Withered, Hinder, Maim, exposure, …). |
| **Curse / Mark** | Subtypes of debuffs cast by curse skills. Marks are usually limited to one target. |
| **Added** | `+X` flat damage applied before increased/more, modified by skill damage effectiveness. |
| **Increased / Reduced** | Additive bucket — all `increased` and `reduced` modifiers sum together first, then multiply once. |
| **More / Less** | Multiplicative independent multiplier. Stacks compositely. The single most impactful damage scaler in the engine. |
| **Penetration** | Ignores a portion of the enemy's resistance during the hit. |
| **Reduction** | Lowers the enemy's resistance value. |
| **Conversion** | Transforms a portion of one damage type into another, propagating through scaling. |
| **Gain as Extra** | Adds an additional layer of damage on top, without removing the original. |

### Combat tags / keywords

`Aura`, `Herald`, `Banner`, `Curse`, `Mark`, `Vaal`, `Trigger`, `Channelling`, `Slam`, `Strike`, `Melee`, `Projectile`, `Bow`, `AoE`, `Movement`, `Travel`, `Minion`, `Trap`, `Mine`, `Totem`, `Brand`, `Spell`, `Attack`, `Hit`, `DoT`. **Tags determine which modifiers apply** to a skill (e.g. `+X% increased projectile damage` only affects gems with the Projectile tag). Always read the gem's tags before assuming a support or stat applies.

### Items and rarity

| Term | Meaning |
|---|---|
| **Normal / Magic / Rare / Unique** | Rarity tiers in both games. |
| **Implicit** | Mod intrinsic to the base item type. |
| **Explicit** | Random-rolled mod. Prefixes vs suffixes count separately. |
| **Prefix / Suffix** | Two affix categories. Prefixes typically host damage/defenses; suffixes host resistances/attributes/utility. |
| **Item Level (ilvl)** | Caps the tier quality of mods that can roll on the item. |
| **Tier (T1, T2, …)** | Mod rolls ranked from best (T1) to worst. Each tier has its own ilvl threshold. |
| **Quality** | Improves the item's primary stat. Cap varies by item type and patch — verify on wiki. |
| **Catalyst** | Adds typed quality to jewelry. |
| **Build-defining item** | Unique whose effect transforms the build's mechanic, distinct from "best-in-slot" which only means statistically superior. |
| **Influence (PoE1)** | Shaper / Elder / Conqueror (Crusader, Hunter, Redeemer, Warlord) — adds exclusive mod pools. |
| **Eldritch implicit (PoE1)** | Searing Exarch (red) and Eater of Worlds (blue) implicits on selected armour slots. |
| **Corrupted** | Vaal Orb result; irreversible final modification. |

### Game modes

| Term | Meaning |
|---|---|
| **Standard / Hardcore** | Permanent leagues. Hardcore deletes characters on death (PoE2 reverts them to Standard equivalent — verify per patch). |
| **SSF (Solo Self-Found)** | No trading, no party. Variant available in both leagues. |
| **Ruthless (PoE1)** | Variant with drops and stats heavily reduced. Never confuse with the standard mode. |
| **Challenge league** | Temporary league rotated periodically; mechanics may be integrated into core, removed, or rotated. |

### Acronyms (commonly used cross-game)

| Acronym | Expansion |
|---|---|
| `PoB` | Path of Building (build planner; PoE2 has its own fork) |
| `EHP` | Effective Hit Pool |
| `DPS` | Damage Per Second |
| `eDPS` | Effective DPS (after enemy mitigation) |
| `IIQ / IIR` | Item Quantity / Item Rarity |
| `HC` | Hardcore |
| `SC` | Softcore |
| `EA` | Early Access (PoE2 status) |

## 2. PoE1-specific vocabulary

### Mechanics

| Term | Meaning |
|---|---|
| **Six-link (6L)** | The historical "fully-linked chest or 2H" goal in PoE1 endgame. Exact socket / link counts evolve per patch — fetch `wiki_parse https://www.poewiki.net/wiki/Item_socket` for current rules. |
| **Tabula Rasa** | Unique chest providing a fully-linked socket setup with no other stats — the canonical leveling shortcut. Drop level and exact socket layout: fetch `wiki_parse https://www.poewiki.net/wiki/Tabula_Rasa`. |
| **Lord's Labyrinth** | Trial used to grant Ascendancy points. Multiple difficulties per playthrough. |
| **Hideout** | Personal instance for crafting bench, master interactions, social hub. |
| **Pantheon** | Set of god powers captured via map bosses, swappable in town. |
| **Atlas of Worlds** | Endgame structure with fixed map nodes connected in quadrants. |
| **Voidstone** | Atlas item earned from pinnacle bosses; raises minimum map tier of a quadrant. |
| **Conquerors of the Atlas** | The four influence holders (Crusader, Hunter, Redeemer, Warlord) that imprint mods on items. |
| **Awakener / Sirus** | Pinnacle of the Conqueror chain. |
| **Maven Invitations** | Multi-boss gauntlets unlocked by witnessed bosses. |
| **Eldritch Currency** | Eldritch Annul/Chaos/Exalted that target prefix or suffix according to Exarch/Eater dominance. |
| **Awakener's Orb** | Fuses two influenced items, keeping one mod from each. |
| **Recombinator** | Combines two items into a third with statistical heritage. Rotated through specific leagues. |
| **Cluster Jewels** | Inserted in cluster sockets, graft a sub-tree of specialised nodes. |
| **Timeless Jewels** | Radically transform notables in a tree zone (Glorious Vanity, Lethal Pride, Brutal Restraint, Elegant Hubris, Militant Faith). |
| **Mastery** | Conditional second-layer choice unlocked once a notable in the cluster is allocated. |

### PoE1 ailments

`Ignite`, `Chill`, `Freeze`, `Shock`, `Bleed`, `Poison`, plus alternates: `Scorch` (-resistance), `Brittle` (+crit chance), `Sap` (-damage).

### Common keystone abbreviations

| Abbreviation | Keystone / mechanic |
|---|---|
| `CI` | Chaos Inoculation (keystone) |
| `MoM` | Mind over Matter (keystone) |
| `EB` | Eldritch Battery (keystone) |
| `LL` | "Low Life" — a CHARACTER STATE (not a keystone), typically reached via Pain Attunement keystone or Shavronne's Wrappings unique to weaponise it. |
| `RF` | "Righteous Fire" — a SKILL GEM (not a keystone). Build archetype around the persistent burn aura. |

### Build/skill abbreviations

| Acronym | Meaning |
|---|---|
| `CWDT` | Cast When Damage Taken |
| `CoC` | Cast on Critical Strike |
| `PF` | Pathfinder ascendancy |
| `TR` | Toxic Rain |
| `HH` | Headhunter |
| `GH` | Goldrim (or Great Helm depending on context) |

## 3. PoE2-specific vocabulary

### Mechanics

| Term | Meaning |
|---|---|
| **Spirit** | Dedicated reservation resource for persistent buffs, permanent minions and trigger/meta gems. Replaces PoE1's mana reservation paradigm. |
| **Reservation Efficiency** | Multiplicative modifier reducing Spirit cost. |
| **Uncut Skill Gem** | Raw item used to engrave a skill gem at the desired level. |
| **Uncut Spirit Gem** | Variant for persistent buff skills. |
| **Uncut Support Gem** | Variant for support gems. |
| **Lineage Support Gem** | Endgame tier of support, boss-only drop, with a per-skill cap. Current cap: fetch `wiki_parse https://www.poe2wiki.net/wiki/Lineage_Support_Gem`. |
| **Meta Gem** | Trigger gems (Cast on Crit, Cast on Shock, Cast on Ignite, …). Often reserve Spirit. |
| **Weapon Set 1 / 2** | Two interchangeable weapon loadouts; skills can be assigned to one set or both. |
| **Weapon Set Points** | Passive points allocated specifically to one weapon set, enabling parallel sub-builds. |
| **Dodge Roll** | Active evasion with i-frames; not all AoE is avoidable by default. |
| **Heavy Stun / Daze / Electrocute** | Reinforced control layers. Electrocute immobilises. |
| **Companion** | Tamed minions (e.g. Tame Beast on Huntress). |
| **Realmgate** | Hub for non-Atlas pinnacles (Breach, Expedition, Delirium, Ritual, …). |

### Trial system

| Term | Meaning |
|---|---|
| **Trial of the Sekhemas** | Honour-based ascension trial (Sanctum-like). |
| **Trial of Chaos** | Trial with chosen Tribulations (Ultimatum-like). |
| **Honour** | Resource consumed in Trial of the Sekhemas. |
| **Tribulation** | Negative mod chosen in Trial of Chaos. |
| **Djinn Barya** | Sekhemas key. |
| **Inscribed Ultimatum** | Chaos key. |
| **Zarokh, the Temporal** | Pinnacle of Trial of the Sekhemas. |
| **The Trialmaster** | Pinnacle of Trial of Chaos. |

### Endgame (subject to imminent rework)

> ⚠️ **PoE2 0.5 (end of May 2026) overhauls the mapping system.** All terms below are likely to evolve or be replaced. Always verify against current patch notes before describing the system.

| Term | Meaning (legacy / pre-0.5) |
|---|---|
| **Waystone** | Map item (T1–Tn). Equivalent of "Map" in PoE1. |
| **Atlas (PoE2)** | Procedural infinite endgame structure. |
| **Tower / Precursor Tablet** | Tower nodes accept Tablets that modify surrounding zones. |
| **Citadel** | Pinnacle gate (Iron, Copper, Stone variants); drops Crisis Fragments. |
| **Crisis Fragment** | Citadel drop; opens Burning Monolith. |
| **Burning Monolith** | Access point to Arbiter of Ash. |
| **Arbiter of Ash** | Final pinnacle (legacy 0.4 designation). |

### Itemisation differences

| Term | Meaning |
|---|---|
| **Rune slot** | PoE2 socket on equipment for Runes / Soul Cores via Artificer's Orb. Replaces PoE1's RGB skill-gem socket. |
| **Soul Core** | Rune-like socketable. |
| **Desecrated mod** | Endgame mod system introduced in PoE2 0.3 — conceptual successor to PoE1 Influence. |
| **Recombinator (PoE2)** | Core crafting tool since 0.2 (Dawn of the Hunt), unlike PoE1 where it is rotated through leagues. |
| **Omen (PoE2)** | Modifies the next orb's effect; a more user-friendly meta-craft compared to PoE1 bench meta-mods. |

### Acronyms

| Acronym | Meaning |
|---|---|
| `WS1 / WS2` | Weapon Set 1 / 2 |
| `EA` | Early Access |

## Cross-game traps

A handful of words exist in both games but mean different things. Resolve game first.

| Word | PoE1 | PoE2 |
|---|---|---|
| **Chaos Orb** | Full reroll of all explicit mods on a Rare. | Removes one random mod and adds one new mod. |
| **Exalted Orb** | Adds one mod; high-rarity currency. | Adds one mod; common-tier currency (the role Chaos played in PoE1). |
| **Map** | Endgame consumable map item. | Conceptually the "Waystone" — verify the user is not on PoE2. |
| **Witch / Ranger / Templar / Marauder** | PoE1 class with PoE1 ascendancies. | PoE2 class with different ascendancies, different starting position, different mechanics. |
| **Atlas** | Fixed quadrant structure with discrete map nodes. | Procedural infinite endgame (subject to 0.5 rework). |
| **6-link** | Endgame goal on chest/2H. | Does not exist; PoE2 supports go inside the gem itself. |

## Reference links

- PoE1 Glossary: `https://www.poewiki.net/wiki/Glossary`
- PoE2 Glossary: `https://www.poe2wiki.net/wiki/Glossary`
- PoE1 Mods reference: `https://poedb.tw/us/Mods`
- PoE2 Mods reference: `https://poe2db.tw/us/`
