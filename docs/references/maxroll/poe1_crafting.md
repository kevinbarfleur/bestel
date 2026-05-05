# Maxroll — Crafting Catalog (Path of Exile 1)

Source: <https://maxroll.gg/poe/crafting>

This catalog covers Maxroll's crafting articles, divided into two groups:

1. **Crafting Basics** (Foundational guides) — explanations of each crafting method as a standalone system. Read these before any applied recipe.
2. **Applied Recipes** — step-by-step crafts for specific items used in named build guides. Useful when a player wants to actually make a particular item rather than learn theory.

The conceptual framework for crafting (modifier types, prefix/suffix, item levels, fractured items, the meta of currency-as-tools) is in `poe1/05_build_crafting_methodology.md`. Use this catalog when the player needs **applied** advice — either learning a method or following a step-by-step recipe.

---

## Crafting Basics — Foundational Guides

These articles each explain a single crafting system. They build on each other — many advanced techniques combine multiple methods (e.g., Essence + Eldritch + Veiled + Beastcraft for an endgame item). The agent should reference the relevant Basics article when the player asks "how does X crafting work" before pointing to applied recipes.

### 1. Crafting Basics: Alterations, Augments & Regals

**URL:** <https://maxroll.gg/poe/crafting/alterations-augments-regals>

**What it covers:** The cheapest crafting method using Orb of Alteration (re-rolls magic items), Orb of Augmentation (adds a mod to a magic item with an open prefix or suffix), and Regal Orb (upgrades magic to rare, adding one mod). Used for low-mod-target crafting like flasks (target prefix on charge gain), jewels (target two specific mods), and early-league items. Strategy: alt-spam to hit a desired modifier, augment for a second, regal for a third (potentially), exalt for a fourth on rares. This is the foundation of leaguestart crafting.

**When to consult:** Player asks how to craft flasks, asks what alt-spam is, or wants to start crafting on a budget. Also for "how to craft cobalt jewels" and similar low-investment items.

---

### 2. Crafting Basics: Fossils and Resonators

**URL:** <https://maxroll.gg/poe/crafting/fossils-resonators>

**What it covers:** Targeted-tag reforging via fossils socketed into Resonators. Each fossil affects mod weights (e.g., Pristine Fossil multiplies Life mods by 10x and blocks high-weight mods). Resonator types determine fossil count (Primitive Chaotic = 1 socket, Potent = 2, Prime = 3, Powerful = 4). Higher fossil counts enable mod-blocking through combination. Critical reference: poewiki.net for fossil mod-pool details, and Craft of Exile for the full combination calculator. Combined fossil setups can create extreme outcomes like +1 to Level of all Spell Skill Gems on a wand by blocking nearly every other mod via 4-fossil Prime Chaotic setups.

**When to consult:** Player asks how fossils work, asks what a Resonator does, or wants to target a specific tag like Life or Caster mods.

---

### 3. Crafting Basics: Essences (Essence Crafting Guide)

**URL:** <https://maxroll.gg/poe/leagues/essence-league-guide>

**Note:** This article lives under `/leagues/` rather than `/crafting/` but is the canonical Essence reference.

**What it covers:** Essence crafting fundamentals. Essences upgrade normal items to rare with a guaranteed modifier (depending on Essence type and item type). Tier scaling 1–7 (Whispering / Muttering / Weeping / Wailing / Screaming / Shrieking / Deafening), with three lower tiers combining via vendor or Essence Stash Tab into one of the next tier. Tier 8 Corrupted Essences come from corrupting a Tier-7 Essence monster with a Remnant of Corruption (purple Essences only — MEDS: Misery, Envy, Dread, Scorn). Used at every level of the game — leaguestart bows, endgame crafting bases, Solo Self-Found gear upgrades. Cannot be used on items with metacrafted "cannot be changed" mods.

**When to consult:** Player asks how Essences work, what tier they should use, what corrupted Essences are, or how to combine lower-tier Essences. Also for "how do I leaguestart-craft a bow" and similar early-league questions.

---

### 4. Crafting Basics: Beastcrafting

**URL:** <https://maxroll.gg/poe/crafting/beastcrafting>

**What it covers:** Niche but important crafting via beasts captured with Bestiary nets and sacrificed in the Menagerie. Key recipes:

- **Craicic Chimeral** — creates an Imprint of a magic item (reset point for crafting). Imprints store a copy that can be restored after the item is modified. Multiple orbs can be created at a time. Often used to bank a base before risky operations.
- **Fenumal Plagued Arachnid (FPA)** — creates 2 pseudo-copies of an item, similar to a Fractured Fossil. Restrictions: no influenced/fractured/synthesized/unique/corrupted/enchanted/already-split items. Used to duplicate high-quality 6-link body armours.
- **Farric Lynx Alpha** — attempts to remove a prefix and slam a suffix.
- **Farric Wolf Alpha** — attempts to remove a suffix and slam a prefix.
- **Craicic Maw** — adds a modifier to an influenced item.
- **Aspect skills** — Black Mórrigan / Saqawal / Farric / Craicic spirit beasts grant Aspect skill suffixes (auras attached to gear).
- **Wild Bristle Matron + Black Mórrigan** — added in 3.26, more powerful alternatives to existing menagerie crafts.

**When to consult:** Player asks how to capture beasts, what beast crafts do, or wants to use specific recipes. Also for "how do I make 6-link copies" or "what's an Aspect skill."

---

### 5. Crafting Basics: Veiled Crafting

**URL:** <https://maxroll.gg/poe/crafting/veiled-crafting>

**What it covers:** Crafting using Veiled Modifiers (unique to the Betrayal mechanic). Veiled mods drop on items from Betrayal encounters or are added via Veiled Orb (drops from Catarina, the Mastermind). The Veiled Orb removes one mod at random and adds a Veiled mod in exchange. Once a Veiled mod is taken to Jun, it's unveiled and unlocked at the Crafting Bench (as a lesser version). Article walks through a typical use case: get item to ideal state with 3 suffixes, benchcraft Suffixes Cannot Be Changed, use Veiled Orb (50% chance the metamod is removed instead of a real mod). Also covers blocking high-weight veiled mods at the bench before unveiling to bias the unveil pool. Use Craft of Exile to see possible unveils.

**When to consult:** Player asks what a Veiled mod is, asks how to unlock crafting bench mods, or asks about Aisling slamming. Also for "how do I add a hybrid mod to a body armor."

---

### 6. Crafting Basics: Eldritch Crafting

**URL:** <https://maxroll.gg/poe/crafting/eldritch-crafting>

**What it covers:** Crafting introduced with Siege of the Atlas (3.17). Eldritch Currency are modified versions of Basic Currency that work on non-influenced helmets/gloves/boots/body armours. The two Implicit families (Searing Exarch / Eater of Worlds) determine which side (prefix / suffix) the Eldritch Currency targets. Tier hierarchy: Lesser (cheap, small mod pool) → Greater → Grand → Exceptional (max tier). Dominance — the higher-tier Implicit determines current Dominant side. Eldritch Exalted Orb adds a mod to the Dominant side. Eldritch Orb of Annulment removes from Dominant side. Eldritch Chaos Orb reforges Dominant side. Used heavily in high-end armour crafting because it allows deterministic targeting of half the affixes.

**When to consult:** Player asks how Eldritch crafting works, asks what Embers/Ichors do, asks about Dominance, or asks how to mod-block a body armor.

---

### 7. Crafting Basics: Eldritch Implicits

**URL:** <https://maxroll.gg/poe/crafting/eldritch-implicits>

**What it covers:** The Eldritch Implicit slot system on non-influenced armour. Two Implicits per item — one Searing Exarch, one Eater of Worlds. Tier hierarchy: Lesser → Greater → Grand → Exceptional. Lesser/Greater Embers and Lesser/Greater Ichors (cheap, used to roll specific implicits). Exceptional Embers/Ichors drop from Pinnacle bosses and apply max-tier implicits. Orb of Conflict (drops from Maven) upgrades one implicit while downgrading the other — heavily weighted toward upgrading the lower-tier implicit. Standard endgame chase — roll Lesser to hit desired implicit, apply Exceptional opposite, use Orbs of Conflict to upgrade.

**When to consult:** Player asks what an Eldritch Implicit is, what an Orb of Conflict does, or how to upgrade Tier 6 → Tier 1 implicits.

---

### 8. Crafting Basics: Metacrafting

**URL:** <https://maxroll.gg/poe/crafting/metacrafting>

**What it covers:** High-end crafting using expensive crafting-bench Metamods. Key Metamods:

- **Suffixes Cannot Be Changed / Prefixes Cannot Be Changed** (2 Divine Orb cost each) — protect one side of the item during a reforge.
- **Cannot Roll Attack Modifiers / Cannot Roll Caster Modifiers** (1 Divine Orb each) — block one tag from being added or removed (allows targeted Annulment).
- **Can Have up to 3 Crafted Modifiers** (2 Divine Orb cost, unlocked in The Putrid Cloister) — allows 2 benchcrafted mods alongside this one.

Restrictions: Essences and Fossils cannot be applied to items with Metamods. Awakener's Orb ignores Metamods. Orb of Dominance respects them. Used in workflows where prefixes and suffixes are crafted separately due to expensive key mods.

**When to consult:** Player asks what Metacrafting is, asks how to protect one side during reforge, or asks how to force high-tier mods via mod-blocking. Also for any complex craft that mentions "lock prefixes" or "block attack mods."

---

### 9. Harvest Crafting Guide

**URL:** <https://maxroll.gg/poe/crafting/harvest-crafting-guide>

**What it covers:** Harvest Bench (Horticrafting Station in hideout) crafts using Lifeforce. Three Lifeforce types — Vivid (Cold/Speed), Primal (Fire/Defence), Wild (Lightning/Resistances) — each unlocks specific crafts. Common high-value crafts:

- **Reforge with X tag** (e.g., Reforge Caster, Reforge Speed) — re-roll the rare item keeping at least one mod with the chosen tag.
- **Augment with X tag** — add a mod with the chosen tag (requires open affix).
- **Remove non-X mod, add X-tagged mod** — surgical exchange.
- **Resistance swap** (500 Lifeforce) — swap one ele resistance type to another. Works on Cluster Jewels and regular Jewels too.
- **Quality enchant override** — override the standard 20% Quality bonus on weapons with bonuses like local increased ele damage (sacrifices Quality for higher damage scaling).

Used heavily in Cluster Jewel crafting (force specific notables via Reforge Caster on Projectile clusters, etc.) and almost every endgame item recipe.

**When to consult:** Player asks how Harvest crafting works, asks what Lifeforce does, asks how to swap resistances, or asks about Cluster Jewel crafting. Also for "what does Reforge Cold mean."

---

### 10. Double Influence Crafting

**URL:** <https://maxroll.gg/poe/crafting/double-influence-crafting>

**What it covers:** Crafting items with two Influences. Workflow: buy a single-influence base with desired tier-1 influence mod (filtered via "Maximum Influence: 1" trade filter), or create one yourself by adding influence to a non-influenced base via influenced Exalted Orbs (e.g., Hunter's Exalted Orb on Stygian Vise). Use a Regal Orb / Reforge Attack to add a second influence mod, or use an Awakener's Orb (combines two single-influence items into one double-influence item with one mod kept and one randomly added). Orb of Dominance elevates an Influence mod tier (and deletes another at random — requires 2+ Influence mods present). Elevation upgrades tier-1 mods to "elevated tier" (numerical or new bonuses). Use Imprints (Craicic Chimeral) to bank between attempts.

**When to consult:** Player asks about elevated mods, asks how an Awakener's Orb works, or wants to craft a double-influence body armor / amulet.

---

## Applied Recipes — Step-by-Step Crafts

These are walkthroughs for crafting specific items used in named build guides. They combine multiple Crafting Basics methods. Each links back to the Crafting Basics it depends on.

### 11. Archmage Hierophant Crafting Guide

**URL:** <https://maxroll.gg/poe/crafting/archmage-hierophant-crafting-guide>

**What it covers:** Endgame items for an Archmage Hierophant build. Wand crafting via Essence (target Cast Speed or Mana, then bench Lightning-as-Chaos), Ring crafting via Essence (Sorrow / Zeal until 3 desired mods), Energy Shield gear via Essence of Greed (target Maximum Mana). Notes: items can be bought from Rog crafters cheaply during the league.

**Dependencies:** Essence + Veiled + Metacrafting basics.

**When to consult:** Player runs an Archmage build, asks how to craft an ES wand, or asks about Mana-related Essence targeting.

---

### 12. Crafting a Chest for Omni Attack Builds (Nimis Kinetic Blast)

**URL:** <https://maxroll.gg/poe/crafting/omni-nimis-kinetic-blast-chest>

**What it covers:** Endgame chest for Omniscience Attack builds. Use a fractured T1 Chaos Resistance base (Triumphant Lamellar, General's Brigandine, Full Dragonscale). Perfect Fossils for quality. Exarch Dominance via Lesser Eldritch Ember to target prefixes. Then Eldritch Exalt or regular Exalt for a 1/7 chance at T5+ max Life.

**Dependencies:** Essence + Veiled + Eldritch + Metacrafting + Eldritch Implicits.

**When to consult:** Player runs an Omni Kinetic Blast build, asks how to craft a chase chest, or asks about Eldritch Exalt slamming.

---

### 13. Crafting a Ring for Omni Attack Builds

**URL:** <https://maxroll.gg/poe/crafting/omni-nimis-kinetic-blast-ring>

**What it covers:** Ring crafting for Omni Attack builds. Fractured Prefix base (~1/21 chance). Use Shrieking Essence of Scorn (budget — 500+ may be required) to roll Tier 2+ Attribute. Annul to set up open suffix paired with Attribute and Crit Multi.

**Dependencies:** Essence + Beastcrafting (Lynx Alpha).

**When to consult:** Player asks for an Omni Attack ring craft.

---

### 14. Crafting an Omni Venom Gyre Helmet (Blizzard Crown)

**URL:** <https://maxroll.gg/poe/crafting/omni-venom-gyre-helmet>

**What it covers:** Endgame helmet for Omniscience builds. Item Level 78+ Blizzard Crown. Suffixes: 9–10% increased Mana Reservation Efficiency of Skills (Loathing), Dexterity or Intelligence (1/23). Uses Annul cycling and Beast slams.

**Dependencies:** Essence + Veiled + Beastcrafting + Eldritch.

**When to consult:** Player runs Omni Venom Gyre, asks how to craft mana reservation gear, or asks about Loathing essence.

---

### 15. Crafting a Belt for Omni Venom Gyre

**URL:** <https://maxroll.gg/poe/crafting/omni-venom-gyre-belt>

**What it covers:** Belt for Omni Venom Gyre. Shrieking Essence of Sorrow (700+ avg). Add third suffix via Exalted Orb slam or Farric Lynx Alpha.

**Dependencies:** Essence + Beastcrafting.

**When to consult:** Player asks for an Omni Venom Gyre belt craft.

---

### 16. Crafting a Shield for Explosive Trap Saboteur

**URL:** <https://maxroll.gg/poe/crafting/explosive-trap-saboteur-shield>

**What it covers:** Item Level 84+ Spirit Shield with Fractured +2% to all max Resistances. Best bases: Chiming / Harmonic / Vaal Spirit Shield. Perfect Fossils for quality. 70–109% increased Fire/Spell Damage (1/190 chance). Farric Lynx Alpha to remove extra Prefix and add Suffix. Cannot Roll Caster Mods + Annul to clean.

**Dependencies:** Essence + Veiled + Metacrafting + Beastcrafting.

**When to consult:** Player runs Explosive Trap, asks about +2% max res shields, or asks about Fire DoT shields.

---

### 17. Crafting a Shield for Hexblast Mines Saboteur

**URL:** <https://maxroll.gg/poe/crafting/hexblast-mines-saboteur-shield>

**What it covers:** Item Level 76+ Shield with Fractured 70–109% increased Spell Damage. Best bases (best to worst): Harmonic, Vaal, Lacewood, Titanium Spirit Shield. Deafening Essence of Greed until 80–109% increased Spell Crit Chance. Exalted Orb slam for ele resists.

**Dependencies:** Essence only (relatively simple).

**When to consult:** Player runs Hexblast Mines and asks about budget shield crafting.

---

### 18. Crafting Elemental Avoidance Boots

**URL:** <https://maxroll.gg/poe/crafting/elemental-avoidance-boots>

**What it covers:** Boots with Movement Speed + 100% chance to Avoid being Chilled / Onslaught on Kill / etc. Uses Deafening Essences of Loathing (Mana Res Efficiency suffix). Exarch Dominance + Eldritch Annul to make room. Suffixes Cannot Be Changed + Veiled Orb. Blessed Orb to up implicit values.

**Dependencies:** Essence + Veiled + Eldritch + Metacrafting + Eldritch Implicits.

**When to consult:** Player asks for boot crafting with elemental avoidance + movement speed combo.

---

### 19. Crafting Gloves for Boneshatter Slayer

**URL:** <https://maxroll.gg/poe/crafting/crafting-gloves-for-boneshatter-builds>

**What it covers:** Gloves for Boneshatter (Slayer ascendancy). Deafening Essence of Greed to T1 ele resist (Harvest swap later). Eater Dominance via Eldritch Annul cycling. Veiled Chaos Orb to add Veiled Suffix (target 45–50% Attack/Cast Speed while Focused). Strike Skills target +N nearby enemies via Greater/Grand Embers + Exceptional Ichor for tier disparity. Spell Suppression via Eater Implicit.

**Dependencies:** Essence + Veiled + Eldritch + Eldritch Implicits + Harvest.

**When to consult:** Player runs Boneshatter, asks how to craft Strike-target gloves, or asks about Spell Suppress + Strike Range gloves.

---

### 20. Crafting Boots for Ignite Builds

**URL:** <https://maxroll.gg/poe/crafting/crafting-boots-for-ignite-builds>

**What it covers:** Item Level 85+ pair of Fractured T2+ Spell Suppress DEX/STR boots. Essence of Greed for max Life. Eater Dominance + Eldritch Annul. Suffixes Cannot Be Changed + Veiled Orb. Implicit options: Drops Scorched Ground while moving (Exarch), Ignites deal damage 5–6% faster (Eater).

**Dependencies:** Essence + Veiled + Eldritch + Metacrafting + Eldritch Implicits.

**When to consult:** Player runs an Ignite build (Detonate Dead, RF, Volatile Dead, etc.) and asks for boot crafting.

---

### 21. Crafting an Endgame Wand for Corrupting Fever Champion

**URL:** <https://maxroll.gg/poe/crafting/corrupting-fever-champion-endgame-wand>

**What it covers:** Endgame wand. +24–26% Damage over Time Multiplier off-trade base. Engraved Wand (low INT requirement) recommended. Resonator + fossil combo for +1 Spell Skill Gems (1/2 chance). Need 1–2 open Prefixes + 2 open Suffixes before continuing.

**Dependencies:** Fossils + Veiled + Metacrafting.

**When to consult:** Player runs Corrupting Fever Champion and wants the chase wand.

---

### 22. Crafting a Wand for Corrupting Fever Champion (Budget)

**URL:** <https://maxroll.gg/poe/crafting/corrupting-fever-wand>

**What it covers:** Budget version of the CF wand. Buy a level 4 Driftwood Wand from Nessa, level 2 character. Buy +1 Spell Skill Gem fractured wand off-trade. Deafening Essences of Woe to T2+ Damage over Time Multiplier (1/133 chance). Bench-craft Can Have 3 Crafted Mods + DoT Multiplier + DoT increased.

**Dependencies:** Essence + Veiled + Metacrafting.

**When to consult:** Player runs Corrupting Fever and wants the leaguestart wand.

---

### 23. Crafting a +2 DoT Amulet for Corrupting Fever Champion

**URL:** <https://maxroll.gg/poe/crafting/corrupting-fever-amulet>

**What it covers:** +2 DoT amulet for Corrupting Fever. Item Level 84+ Turquoise or Lapis Amulet base. Alteration spam, Annul down to 2 mods, Fenumal Plagued Arachnid to create magic base.

**Dependencies:** Alterations + Beastcrafting (FPA) + Metacrafting.

**When to consult:** Player asks for the chase amulet for CF Champion.

---

### 24. Crafting a Detonate Dead Oscillating Sceptre

**URL:** <https://maxroll.gg/poe/crafting/crafting-a-detonate-dead-oscillating-sceptre>

**What it covers:** Sceptre for Detonate Dead Elementalist. Oscillating Sceptre (gives Elemental Overload keystone via implicit, no need to allocate). Item Level 82+. Need 1 open Suffix + 2 open Prefixes. Essence + Annul to clean. Bench-craft Gain 7–8% Fire as Extra Chaos.

**Dependencies:** Essence + Veiled + Metacrafting.

**When to consult:** Player runs Detonate Dead Elementalist or asks about Oscillating Sceptre.

---

### 25. Crafting Poison Spark Sceptre

**URL:** <https://maxroll.gg/poe/crafting/poison-spark-sceptre>

**What it covers:** Endgame Poison Spark sceptre. Buy lowest-ilvl-but-84+ Oscillating Sceptre. Perfect Fossils for 30%+ Quality (for the future enchant). Deafening Essences of Torment.

**Dependencies:** Fossils + Essence + Veiled + Metacrafting.

**When to consult:** Player runs Poison Spark and asks for the sceptre craft.

---

### 26. Crafting Poison Spark Boots (Stormshroud)

**URL:** <https://maxroll.gg/poe/crafting/poison-spark-boots>

**What it covers:** Poison Spark boots. Deafening Essences of Torment to second T1 Resistance (1/35 chance). Chaos Resistance is alternative if Stormshroud handles shock avoidance elsewhere. Need 59%+ Chance to Avoid Being Shocked if boots are the sole shock-avoid source. Lesser Eldritch Ember for Exarch Dominance + Eldritch Annul.

**Dependencies:** Essence + Veiled + Eldritch.

**When to consult:** Player runs Poison Spark and asks about Stormshroud boot prep.

---

### 27. Crafting a Wand for KB Nimis Fireball Deadeye

**URL:** <https://maxroll.gg/poe/crafting/fireball-deadeye-wand>

**What it covers:** CoC Nimis KB Deadeye wand. Fractured +1 to Level of all Fire Skill Gems base recommended (cheaper than +1 Spell). 35–38% Crit Chance prefix (~1/120 with fractured prefix).

**Dependencies:** Essence + Fossils + Veiled + Metacrafting.

**When to consult:** Player runs CoC Kinetic Blast Deadeye or asks about +1 Fire Skill wands.

---

### 28. Crafting a Bow for Ice Shot Deadeye

**URL:** <https://maxroll.gg/poe/crafting/ice-shot-deadeye-bow>

**What it covers:** Ice Shot Deadeye bow. T4 increased Physical Damage fractured base (Spine, Imperial, Maraketh). Adds # to # Cold Damage prefix (1/46). 25–27% Crit Chance suffix.

**Dependencies:** Essence + Fossils + Beastcrafting + Veiled + Metacrafting.

**When to consult:** Player runs Ice Shot or wants a generic ele bow craft.

---

### 29. Crafting an Elemental Bow for Lightning Arrow / Tornado Shot

**URL:** <https://maxroll.gg/poe/crafting/lightning-arrow-builds-elemental-bow>

**What it covers:** Endgame bow for LA/TS. Item Level 82+ Bow with Fractured Flat Elemental Damage T2+. Best bases: Spine, Imperial, Maraketh. Essence of Hatred / Anger / Wrath cycling. Add open prefix/suffix structure, then Harvest Augment Cold/Fire/Lightning targeted to the missing element (~7 attempts on average). Veiled Orb for desired veiled suffix. 6-link bench costs 1500 Fusing.

**Dependencies:** Essence + Harvest + Veiled + Metacrafting + Fossils.

**When to consult:** Player runs Lightning Arrow / Tornado Shot / Ice Shot Deadeye and wants the chase ele bow.

---

### 30. Crafting a +2 Arrow Bow for Cold Convert Tornado Shot

**URL:** <https://maxroll.gg/poe/crafting/omni-ts-bow>

**What it covers:** +2 Arrow Bow with Fractured Spine (or other) base. Use Perfect Fossils for 30% quality. Aisling cannot add Hybrid Physical/Resistance — must use Essence spam to get hybrid. Annul down to open prefix + open suffix. Prefixes Cannot Be Changed + Veiled Orb to add veiled prefix. After unveil, Prefixes Cannot Be Changed + Orb of Scouring to retry if needed.

**Dependencies:** Essence + Aisling (Veiled Suffix slamming) + Metacrafting + Fossils.

**When to consult:** Player runs Cold Convert Tornado Shot or wants a +2 Arrow chase bow.

---

### 31. Crafting an Elemental Quiver for Lightning Arrow / Tornado Shot

**URL:** <https://maxroll.gg/poe/crafting/lightning-arrow-deadeye-quiver>

**What it covers:** Bow Attacks fire +1 additional Arrows fractured suffix base (1/147). Shrieking Essence of Torment. Annul down to clean. Cannot Roll Attack Modifiers + Exalt-slam cycle for high-tier increased Damage with Bow Skills.

**Dependencies:** Essence + Metacrafting.

**When to consult:** Player asks for ele quiver craft (LA, TS, KB, etc.).

---

### 32. Crafting a +1 Arrow Quiver for Cold Convert Tornado Shot

**URL:** <https://maxroll.gg/poe/crafting/cold-convert-tornado-shot-quiver>

**What it covers:** +1 Arrow Quiver. T1 Crit Multi with Bows fractured suffix (1/38). Annul to setup open suffix with no other Attack tags besides Adds Phys + +35–38% Crit Multi.

**Dependencies:** Essence + Metacrafting.

**When to consult:** Player asks for a Cold-Convert Tornado Shot quiver.

---

### 33. Crafting a Bow for Explosive Arrow Ballista Elementalist

**URL:** <https://maxroll.gg/poe/crafting/crafting-a-bow-for-explosive-arrow-ballista-elementalist>

**What it covers:** Two versions — leaguestart (6-link from The Porcupine div card set) and endgame (Item Level 82+ Fractured base). Veiled +18–20% Fire DoT Multi prefix (unlocked from veiled suffixes on weapons). Beware: Fire Damage to Attacks prevents Elemental Equilibrium — annul or sell if slammed.

**Dependencies:** Essence + Veiled + Metacrafting + Fossils + Harvest.

**When to consult:** Player runs Explosive Arrow Ballista Elementalist.

---

### 34. Crafting a Leaguestart Bow for Toxic Rain Pathfinder

**URL:** <https://maxroll.gg/poe/crafting/toxic-rain-pathfinder-leaguestart-bow>

**What it covers:** Leaguestart Bow for TR Pathfinder. The Porcupine div card set (6 cards) → 6-Link Item Level 50 Short Bow. Cheap acquisition for new league. Annul down to open prefix + suffix.

**Dependencies:** Divination cards + Essence + Veiled + Metacrafting.

**When to consult:** Player asks for cheap leaguestart bow for TR.

---

### 35. Crafting a Bow for EK Ignite Elementalist

**URL:** <https://maxroll.gg/poe/crafting/ek-ignite-elementalist-bow>

**What it covers:** Ethereal Knives Ignite Elementalist Bow. Enchant: Enemies you Kill have 25% Chance to Explode (Physical 10% maxLife). Annul to 2-mod base. Suffixes Cannot Be Changed + Orb of Scouring. Cannot Roll Attack Mods + Exalt cycle. Quality Enchant: Grants 1% increased Elemental Damage per 2% Quality (5000 Primal Crystallised Lifeforce via Harvest).

**Dependencies:** Veiled + Metacrafting + Harvest + Eldritch.

**When to consult:** Player runs EK Ignite Elementalist (a fairly niche build).

---

## Recap — Routing matrix

| Player intent | Article(s) to consult |
|---|---|
| "How does X crafting method work?" | Relevant Crafting Basics (Alterations / Fossils / Essences / Beastcrafting / Veiled / Eldritch / Eldritch Implicits / Metacrafting / Harvest / Double Influence) |
| "How do I leaguestart-craft cheap items?" | Alterations Augments Regals + Essences + Toxic Rain Leaguestart Bow |
| "How do I craft an endgame body armour?" | Eldritch Crafting + Omni Nimis Chest example |
| "How do I make a +2 Arrow Bow?" | Omni TS Bow recipe |
| "How do I make an ele bow?" | LA/TS Elemental Bow recipe |
| "What does Reforge X mean?" | Harvest Crafting Guide |
| "What's an Imprint?" | Beastcrafting (Craicic Chimeral) |
| "How do I do Aisling?" | Veiled Crafting + relevant recipe (Aisling adds veiled mods) |
| "What's an Awakener's Orb?" | Double Influence Crafting |
| "How do I 6-link?" | Bench craft (1500 Fusings) — mentioned in many recipes |
| Player names a build → wants the gear | Find the matching Applied Recipe by build name |

## Cross-reference notes

- **Many recipes assume mastery of multiple Basics** — point the player to the relevant Basics first if they don't understand a step.
- **Recipes are usually tied to a specific patch's mod weights** — values like "1/133 chance" or "1/46 chance" assume current mod pool. After a patch with mod-pool changes (3.27, 3.28 added new mods), some chances shift slightly. Use these as ballpark estimates.
- **For absolute mod-weight references, use Craft of Exile** (<https://www.craftofexile.com>) — third-party calculator with the full mod pool. Maxroll references it explicitly in the Veiled Crafting and Fossils & Resonators articles.
- **Awakener's Orb**, **Orb of Dominance**, **Orb of Conflict** — these are the three "luxury" crafting orbs. Each has a dedicated section in the relevant Basics article (Awakener's Orb in Double Influence, Orb of Dominance in Double Influence, Orb of Conflict in Eldritch Implicits).
- **Aisling slamming** = using a Tier 4 Aisling betrayal reward to add a random Veiled mod to an item (and remove a random mod). Critical for endgame rare bows and weapons. See Veiled Crafting basics + Betrayal Farming Guide (in `maxroll_currency.md`).
- **The Putrid Cloister** (Museum unique map) — required to unlock Can Have up to 3 Crafted Modifiers metamod. Mentioned in Metacrafting basics.
