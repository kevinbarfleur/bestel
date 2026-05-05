# 02 — Action RPG foundations

This document explains **the conventions of the action-RPG genre that PoE inherits and bends**. It is not PoE-specific; it is the genre grammar PoE players already know intuitively, and that the agent should hold as background when reasoning about a build.

PoE makes radical choices, but those choices only make sense against the backdrop of the genre's invariants. When a player asks "is this a strong build?" they are silently invoking the assumptions below. The agent must do the same.

## 1. The aRPG contract

Action RPGs are built on a tight loop: **kill monsters → loot drops → upgrade character → kill harder monsters**. Every system in the genre exists to serve that loop. The four pillars:

- **Power escalation.** A character's effective power grows by orders of magnitude across a single playthrough. End-game content assumes a fully built character; campaign content assumes minimal investment.
- **Gear-driven identity.** Most of a character's power lives on items, not on the level-up screen. Skills define style; gear defines magnitude.
- **Build identity.** A class is not a role — it is a starting point on a customisation surface. Players expect to express identity via build choices (skill, ascendancy, items, passives, supports).
- **Replayability through ladder/economy resets.** New seasons or leagues reset progress, refreshing the chase. Players accept that progress is ephemeral.

A player asking advice is asking: "given this loop, am I optimising correctly for the stage I'm in?" The agent's job is to map back to that loop.

## 2. The Diablo 2 template

PoE inherits explicitly from Diablo 2 (Brevik / Schaefer / etc.) more than from Diablo 3 or Diablo 4. The agent should recognise the D2 patterns:

- **Skill trees with prerequisites and synergies.** D2 1.10 introduced *skill synergies* so low-tier skills feed high-tier skills. This pattern (build a tree, choose nodes, scale via synergy) is foundational. PoE's passive tree is a direct descendant.
- **Equivalent skill systems across classes.** Casters, melee, ranged all use the same skill mechanics (point allocation, mana cost). No "spells vs abilities" split. This made every class equally complex and equally interesting in multiplayer. PoE preserves this rigorously.
- **Mana as the universal cost vector.** D2 made mana the primary restriction on skill use, eliminating "rest to recover spells" mechanics. PoE uses mana, life, Spirit, and reservation, but the philosophy remains: cost is uniform across class.
- **Items as currency / runewords.** D2 introduced runewords (recipes that turn socketed white items into uniques). PoE replaced this with currency-as-craft (orbs that modify items). Both reject "gold" as the primary unit.
- **Difficulty rebooted on second playthrough.** D2 Nightmare and Hell raise enemy stats and add resistance penalties. PoE inherits this pattern: a post-campaign resistance penalty kicks in when the player transitions from acts to maps, forcing the player to *re-cap* resistances on gear before progressing. Verify the exact penalty value on the wiki — it has changed across patches.
- **Hardcore mode as a permadeath alternative.** D2 invented this; PoE preserves it.

When a question mentions a concept that "feels classic," the agent should mentally cross-check D2 first. If the user is a D2 veteran who switched to PoE, they often expect D2 mechanics to apply, and the agent must either confirm or correct.

## 3. Damage delivery taxonomy

Every skill in any aRPG falls into a **delivery type**. The delivery type determines what scales the skill, what mitigation applies, and what supports work. Confusing the delivery type is the most common reasoning error.

| Delivery | What it is | Scales from | Mitigated by |
|---|---|---|---|
| **Hit** | Single damage instance applied on contact. | Base damage + added + increased + more, weapon DPS, gem level. | Resistances, armour (phys), evasion (attack), block. |
| **DoT (Damage over Time)** | Damage per second over a duration. | Base damage + DoT multipliers. | Resistances. Cannot be evaded, dodged, blocked. |
| **Ailment** | Effect triggered by a hit, follows its own rules. | Hit damage at the moment of application + ailment-specific multipliers. | Resistances, ailment-avoidance, immunity. |
| **Minion / pet** | Independent entity that does its own hits. | Minion damage + minion-tagged supports + auras + the player's gear that affects minions. | Targets the minions' hit profile, not the player's. |
| **Trigger / meta** | Skill cast in response to a condition (on hit, on crit, on damage taken). | The triggered skill's normal scaling. | Same as the underlying skill. |
| **Aura / curse / herald** | Persistent effect on player or enemies. | Aura/herald-specific multipliers. | Curse resistance, aura nullifiers. |

The agent's first move on a "is X strong?" question is: **identify the delivery type** of the skill. Everything else flows from that.

## 4. Defensive layers

aRPGs treat survival as a stack of independent layers. Reading "EHP" as a single number is a beginner mistake. The veteran reads:

1. **Avoidance.** Don't get hit. Sources: dodge, evasion, spell suppression, block, dodge roll.
2. **Mitigation.** When hit, take less. Sources: armour (phys), resistances (elemental, chaos), max-resistance modifiers, "% damage reduction," "% damage taken as another type."
3. **Recovery.** After hitting, restore. Sources: life regen, leech, recoup, ES recharge, flasks, life-on-hit.
4. **Hit pool.** What absorbs the residual. Sources: life, ES, mana (with MoM), Ward.
5. **Ailment / debuff coverage.** Don't get killed by status, slow, curse, freeze. Sources: ailment immunity, freeze threshold, curse resistance.

A defensive build is **layered**, not maximised on a single dimension. A player who has 10k life but 0% block, 0% suppression, and 0% chaos resistance is one chaos slam away from a corpse. The agent must read the layers, find the weakest, and recommend filling that gap — not "get more life."

## 5. Scaling axes

aRPGs use multiplicative compound scaling. The buckets are universal:

| Axis | Behaviour | Notes |
|---|---|---|
| **Base damage** | The starting flat number on the skill / weapon. | Multiplied by everything downstream. |
| **Added damage** | `+X` flat damage from gear / passives / auras. | Modified by the skill's "damage effectiveness" coefficient. |
| **Increased / reduced** | Additive bucket. All `increased/reduced` of one type sum into a single multiplier. | "Increased" stacking is sub-linear past a threshold. |
| **More / less** | Multiplicative independent. Stacks compositively across sources. | The single most impactful scaler. Compound `more` multipliers dominate. |
| **Conversion** | Transforms a portion of one damage type into another. Inherits scaling from both source and destination types. | Build-defining effect, not a small bonus. |
| **Penetration / reduction** | Lowers the enemy's effective resistance. | The "more damage" of late-game. |
| **Crit chance × crit multiplier** | Probabilistic damage spike. | Flat "more" if crit chance is capped or near-capped. |
| **Action speed** | Cast / attack / movement. | Multiplies hits-per-second, indirectly multiplies DPS. |

The classic mistake: stacking 600% increased damage when adding one more `more` source would have doubled the output. The agent must always check: **is the next upgrade in the right scaling bucket?**

## 6. Action economy

aRPG combat is a real-time ballet of discrete actions. Each action costs something:

- **Cooldown.** Per-skill or shared. Forces rotation rather than spam.
- **Resource cost.** Mana, life, Spirit, charges, soul-resources.
- **Cast / attack time.** Locks the character. Animation-cancel is a recurring discussion.
- **Channelling.** Sustained but interruptible.
- **Charge generation / consumption.** Stacks earned by some action, consumed by another (Power/Frenzy/Endurance Charges in PoE).
- **Reservation.** Permanent commitment of a resource pool to maintain a buff (auras in PoE1 mana, Spirit in PoE2).

A build's "feel" is a function of its action economy. A perma-Frenzy build runs differently from a CWDT trigger build. When a user complains "it feels clunky," the agent should look at action economy first — not damage numbers.

## 7. Itemisation grammar

The genre converged on a shared vocabulary for items:

- **Rarity tiers** (white/blue/yellow/orange in PoE; equivalent in D2/D3/D4/Last Epoch). More mods, scarcer drops as rarity climbs.
- **Affixes** (prefixes vs suffixes). Two affix categories balance offence vs defence on the same item.
- **Item level** caps the tiers an item can roll. Older items can never roll modern T1 mods.
- **Sockets and links** (D2/PoE1) vs **gem-as-item** (PoE2/D3-ish). Socket systems trade complexity for flexibility.
- **Implicit modifiers** intrinsic to the base. Modify the base's identity, not its rolls.
- **Set bonuses / runewords / influence** as alternate paths to powerful items. Each game picks one or two of these patterns; PoE inherits runeword spirit via influence + currency-as-craft.
- **Build-defining uniques.** Single items whose mechanic transforms the build (Mageblood, Headhunter, Voidforge in PoE; Enigma, Infinity in D2). Distinct from "best-in-slot" — they enable a build, not optimise it.

When the agent suggests an item, it must distinguish "build-defining" (changes the play) from "best-in-slot" (improves the math).

## 8. Trade economy archetypes

aRPGs split into two economic philosophies:

- **Trade-friction games** (PoE1, PoE2, D2 with stash mules). No auction house, manual whisper, party invite, friction by design. Trades are slower; items keep value longer; drop rates can be lower because trade amplifies effective acquisition.
- **Auction-house games** (D3 RoS-era, briefly D2:R). Trades are instant; items lose value rapidly; drop rates must be lower to compensate; "perfect rolls" become commodities. Often abandoned for design reasons.
- **SSF (solo self-found).** No trading at all. Drop rates feel different because there is no trade leverage; the player must self-craft or self-find. PoE supports this as a first-class mode.

The "currency-as-craft" pattern (PoE) eliminates gold by making every crafting orb a barter unit. Items are valued by what they enable a buyer to craft, not just what they are.

The agent must recognise which economic context a question lives in. "What's a fair price?" makes no sense without league + trade-vs-SSF context.

## 9. The campaign-vs-endgame split

aRPGs typically split into two distinct phases:

- **Campaign.** Linear narrative, fixed encounters, gear acquisition through scripted drops + low-tier crafting. Tuned for ~6–15 hours. The campaign is a tutorial for the genre and the build.
- **Endgame.** Open-ended, randomised, infinitely scalable. Atlas / Greater Rifts / Monoliths / etc. The endgame is where 90%+ of build-relevant decisions are tested.

A build that "works in campaign" tells you almost nothing about endgame viability. The agent must always ask: **what stage of the loop is the player in?** Advice changes drastically between act 4 of the campaign and a juiced T16 map.

## 10. Build-thinking discipline (transferable from other aRPGs)

Veteran aRPG players think with a transferable mental model:

1. **Pick a delivery type.** What kills mobs? What kills bosses?
2. **Pick a scaling vector.** Which bucket multiplies the damage by 10×, not 1.1×?
3. **Pick a defensive paradigm.** Life-stacking, ES, hybrid, Ward, MoM, block-cap, suppress-cap.
4. **Pick a content target.** Clear-speed farmer, single-target boss, league mechanic specialist, hardcore-safe, SSF-craftable.
5. **Verify the budget.** What does the build need at minimum to function (uniques, ascendancy, levels)?
6. **Plan the path.** Which order do upgrades happen in? What's the minimum-viable version?

A user asking "is this a good build?" is asking step 6, but the agent must be ready to back-track to step 1 if the foundation is wrong. The most common failure mode is optimising step 6 on a broken step 1.

## Sources informing this document

- The Game Design Forum, *Reverse Design: Diablo 2*: `https://thegamedesignforum.com/features/RD_D2_4.html`
- Unwinnable, *How Diablo II Reshaped RPGs*: `https://unwinnable.com/2015/06/30/diablo-ii/`
- Game Developer (Bycer), *The Devil Is in the Details of Action RPGs — Part Two: Leveling Up*
- Path of Exile forums, *Trade Manifesto*: `https://www.pathofexile.com/forum/view-thread/2025870`
- GameRant, *How Diablo 4 and Path of Exile 2 Both Inherit the Legacy of Diablo 2*

The above are background reading for the agent's own reasoning. When citing facts to a user, always link to current PoE / PoE2 wiki and patch notes, not to these articles.
