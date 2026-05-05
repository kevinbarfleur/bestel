# 16 — Build creators and build-crafting methodology

This document is **not a catalog of builds**. Builds are ephemeral — they get nerfed, reworked, replaced from one league to the next. What does *not* change is the **reasoning structure** that good builds share, and the **set of creators** whose work is worth consulting when the agent needs to ground its thinking in current expert practice.

The document has three jobs:

1. **Provide URLs** to trusted creator profiles so the agent can always go check the live state of any build before quoting numbers, items, or trees.
2. **Map the anatomy** of a high-quality build guide so the agent knows what sections to expect and what each section actually *means*.
3. **Articulate the build-crafting decision framework** so the agent can reason about a new build, evaluate a player's existing build, or understand why a creator made a specific choice — independently of any single guide.

The agent should treat everything below as a **mental scaffold**, not as rules. PoE1 and PoE2 evolve constantly. Specific items, skills, ascendancies, and meta priorities can shift in any patch. The framework is durable; the examples illustrating it are not.

---

## Part 1 — Creator profiles

These are the creators whose work the agent should consult when asked about builds, league starts, or the current meta. Each profile lists the creator's URL, archetype focus, and signature philosophy. **The agent should always open the profile URL to retrieve the current build list before quoting specifics**, because builds get archived per-league and updated per-patch.

### Goratha — Maxroll

- **PoE1 profile:** <https://maxroll.gg/@goratha?game=poe>
- **PoE2 profile:** <https://maxroll.gg/@goratha?game=poe2>

Goratha is a Maxroll content creator known for SSF-friendly, well-rounded league starters that smooth out the early-to-late game transition. His writing emphasises that a build should be *playable from level 1 with no twink gear*, scale gracefully through the campaign, and remain viable through Atlas progression without requiring a chase unique to function. His PoE1 signature archetypes have included Penance Brand Ignite Elementalist, Rolling Magma Mines Saboteur, Shock Nova of Procession Archmage Hierophant, and Eviscerate Bleed Gladiator. In PoE2 (0.4.0+) he has covered Disciple of Varashta plant builds, Fireball Blood Mage, and Bleed Rake Amazon. The agent should reach for Goratha when a player asks "what should I league-start with as a beginner" or "I want a build I can theoretically finish on SSF."

### Zizaran — Maxroll

- **PoE1 profile:** <https://maxroll.gg/@zizaran?game=poe>
- **PoE2 profile:** <https://maxroll.gg/@zizaran?game=poe2>

Zizaran has been streaming PoE since 2015 and runs the Gauntlet, the largest hardcore community event. His builds skew toward *hardcore-viable* — meaning defensive layers are non-negotiable, and the build is expected to survive bad map mods, on-death effects, and lag spikes. Signature PoE1 builds include Boneshatter Juggernaut (slow immunity, regen, Trauma stacking), EA Ballista Champion (Fortify-based defenses on totems), Lacerate Gladiator (Block-stacking bleed). In PoE2 he has covered Ice Strike Invoker. He is also an unofficial community spokesperson — he interviews GGG developers, surfaces community concerns, and his guides usually include an honest difficulty/safety rating. The agent should reach for Zizaran when a player asks about hardcore play, when defense matters more than damage, or when the player is a beginner who values clear and stable instructions.

### Palsteron — Maxroll

- **PoE1 profile:** <https://maxroll.gg/@palsteron>
- **PoE2 profile:** <https://maxroll.gg/@palsteron?game=poe2>
- Twitch: <https://www.twitch.tv/palsteron>

Palsteron specialises in *bow and totem* builds — Explosive Arrow Ballista (Champion and Elementalist variants), Toxic Rain Ballista Pathfinder, Caustic Arrow Poison Ballista Trickster, Kinetic Fusillade Ballista Hierophant, Archmage Ball Lightning Hierophant. His writing is unusually clear about itemisation philosophy: *"The Item Progression in Path of Exile is not as rigid as in most other games... flexibility is key. The game won't always give you exactly the item you are looking for, so you need to adapt. Even if small upgrades seem insignificant, the sum of them decides whether your character fails or succeeds."* The agent should reach for Palsteron when a player wants a bow or totem build, when crafting itemisation needs explanation, or when the player asks about scaling Caustic Arrow / Toxic Rain / Explosive Arrow specifically.

### Pohx — Mobalytics

- **PoE1 profile:** <https://mobalytics.gg/poe/profile/pohx>
- External reference site (his own): the "Pohx Wiki" — searchable as a PoE community institution dedicated to Righteous Fire

Pohx is *the* Righteous Fire reference. His career has been built on making the RF Chieftain accessible to new players: low APM, tanky, very forgiving, scales naturally with monster density via Hinekora's Death's Fury proc. He maintains both a vanilla league-start version and a Life Stack RF endgame variant (Kaom's Heart, Adorned Jewels, Foulborn Rathpith). Anyone asking "I'm new to PoE, give me something I can't fail with" or "how does Righteous Fire work" should be routed to Pohx. The agent should also know his archetype's hard limit: RF is **DoT-cap-bound** at the high end, single-target on Maven and similar pinnacle bosses is often weak, and the build is restricted to Marauder (Chieftain ascendancy is the canonical home).

### Ruetoo — Mobalytics

- **PoE1 profile:** <https://mobalytics.gg/poe/profile/ruetoo>
- **PoE2 profile:** <https://mobalytics.gg/poe-2/ruetoo>
- Twitch: <https://www.twitch.tv/ruetoo>

Ruetoo is a partnered creator who covers both games. PoE1: Slayer (Elemental Hit), Elementalist (Phys DoT — often co-authored with imexile), and other off-meta optimisation work. PoE2: Mercenary Tactician with Stormblast Bolts + Trinity stacking, Pathfinder with grenades, Ranger Lightning Spear / Explosive Shot, Huntress Rapid Shot Amazon (Fresh Clip mechanic). His guide style is unusual in that he ships **multiple progression tabs in one guide** (Live, Take 2, Rip variants — explicitly tracking his own character iterations), which is useful when a player wants to see how a build evolves rather than only its endgame state. The agent should reach for Ruetoo when a player wants high-skill optimisation, off-meta exploration, or to see the *iteration process* behind a build rather than a static recipe.

### Fubgun — Mobalytics

- **PoE1 profile:** <https://mobalytics.gg/poe/profile/fubgun>
- **PoE2 profile:** <https://mobalytics.gg/poe-2/fubgun>
- Twitch: <https://www.twitch.tv/fubgun>

Fubgun is the clearspeed maximalist. PoE1: Lightning Arrow / Elemental Hit / Tornado Shot Deadeye — bow archetypes optimised for mapping throughput. PoE2: Lightning Arrow Deadeye (one of the most popular meta builds), Lightning Spear Deadeye (pre-nerf reference). His guide format is unusually explicit about progression tiers — he ships variants labelled `Starter`, `Crit swap`, `Mid game`, `Inspired setup`, `Endgame no HH`, `Headhunter`, `Warden(Optional) Uber Endgame`. He also includes vendor regex strings to filter shop UIs and gear-acquisition shortcuts (Hyrri's Bite vendor recipe, Prismweave as cheap early unique). The agent should reach for Fubgun when a player explicitly wants *speed* — fast mapping, fast farming, fast clear — or wants to understand how a build's gearing scales tier by tier from league start to mirror-tier.

---

## Part 2 — Anatomy of a high-quality build guide

Both Maxroll and Mobalytics build guides converge on a similar information architecture, even if the section names differ. Knowing what a "good" guide contains lets the agent *evaluate* whether a guide is complete, and lets it route a player's question to the correct section of the right guide.

### Maxroll PoE1 — the Build Progression Tool

Maxroll's PoE1 guides have been migrated to an interactive *Build Progression Tool* with a slider that advances character level and shows the corresponding tree, gem links, gear, and tips at each level. The skeleton of every guide is:

- **Overview** — Build Rating (a per-content score: campaign / mapping / bossing / SSF / hardcore), embedded video, gear and ascendancy summary, Farming Strategies recommended for the build, links to Crafting Guides for any custom-crafted gear.
- **Build Information** — the *why*. Explains the key mechanics that make the build tick, why each chosen unique is in the build, why each support gem is in the link, and how the skill interacts with the chosen ascendancy.
- **Leveling** — passive points by milestone, where to obtain each gem (vendor, quest reward, level), early-act tips, gear breakpoints, and how to "swap" to the endgame version when ready (e.g., RF swap at level 18 once Fire Resistance is sufficient).
- **Path of Building (PoB) export** — the canonical artifact every serious build ships with. Imports into Path of Building Community for full DPS / EHP calculation.
- **Changelog** — patch-by-patch updates with the league tag (e.g., "Mirage 3.28").

### Maxroll PoE2 — section-driven layout

PoE2 guides on Maxroll use an explicit section sidebar:

- **Skills** (the gem links and rotation),
- **Ascendancy** (which path and node order),
- **Passives** (tree progression, leveling vs endgame),
- **Stat Priorities** (the gear-shopping cheatsheet — what to look for first, second, third),
- **Gearing** (slot-by-slot recommendations),
- **FAQ** (anticipated questions: "do I need Trinity?" "Sekhemas or Chaos for points 5–6?"),
- **Video** (the streamed companion),
- **Summary** (the elevator pitch),
- **Changelog** (patch history).

The PoE2 *Stat Priorities* section deserves particular attention — it's a flat ranked list of what stats to look for on each gear slot, which is the most decision-useful artifact of any guide for a player buying or crafting upgrades.

### Mobalytics — variant-driven layout

Mobalytics guides put **build variants** at the centre. A single guide page can contain `Starter / Mid game / Endgame / Headhunter / Uber` tabs, each with its own complete tree, gear, and gem links. The skeleton is:

- **Path of Building Code** — top of page, copy-paste into PoB Community.
- **Build Overview** — the elevator pitch and what makes the build distinctive.
- **Strengths and Weaknesses** — explicit pro/con list (e.g., "20K life / very tanky / can kill Ubers / very expensive / limited by DoT cap").
- **Build Variants** — the progression tabs.
- **Equipment** — slot-by-slot stats, often with example items.
- **Passive Tree** — tree image with notable callouts.
- **Skill Gems** — main link, secondary links, auras, movement.
- **How it Plays** — the moment-to-moment rotation (e.g., "Shield Charge into pack, Punishment + Fire Trap on rares, reposition with Frostblink").
- **How it Works** — the deeper mechanic (e.g., "Hinekora's Death's Fury procs scale with monster density, Cloak of Flame scales the proc's ignite").

### What the agent should look for

Across all three layouts, a *complete* build guide answers five questions:

1. **What does the build do** — clear pitch in one paragraph.
2. **Why does it work** — the mechanical engine (e.g., Trauma stacking, Spirit reservation budget, conversion order, ailment proliferation).
3. **What does it need** — the gear / gems / passives required for the engine to function vs nice-to-have.
4. **How does it play** — the moment-to-moment button presses.
5. **How does it scale** — what to upgrade first, what's the next breakpoint, where does it cap out.

If a guide skips any of these, it's incomplete. If a player asks the agent for help evaluating an unfamiliar guide, mapping that guide's content against these five questions is a good diagnostic.

---

## Part 3 — The build-crafting decision framework

When designing or evaluating a build, the same set of decisions recurs. They are loosely sequential — earlier decisions constrain later ones — but in practice good builders iterate across all of them at once, often using Path of Building to test a hypothesis numerically before committing.

### 3.1 Define the goal

The single most important step, repeatedly emphasised across the community, is to **decide what the build is for** before choosing skill or class. The questions are:

- **What content** is the priority? Mapping / boss farming / Delve / Heist / Sanctum / SSF self-found grind / hardcore survival / racing? Each has different demands. A Delve build prioritises movement speed and life regen against ground degens; a boss farmer prioritises burst single-target; a mapper prioritises clear-speed AoE.
- **What budget** is realistic? A league-start build assumes Chaos Orbs only; a mid-game build assumes a few Divine Orbs; an endgame build may assume Mirror-tier gear. *The same skill can be a totally different build at each budget tier* — this is why Fubgun's guides ship 5+ tabs.
- **What level cap** is plausible? Most players plateau at 90–95, not 100. A build planned around level 100 has unrealistic passive count.
- **What playstyle** does the player enjoy? Spin2win Cyclone, autobomber chain explode, channeled DoT, totem placement, mine throwing, summoner pet management. The mechanically optimal build a player won't enjoy playing is worse than a slightly weaker one they will.

A build without an articulated goal scales nothing well because every decision becomes ambiguous.

### 3.2 Choose a skill identity

The main skill is the build's identity. It determines:

- **Damage type and tags** — physical / cold / fire / lightning / chaos; attack vs spell; projectile vs AoE; melee vs ranged. Tags are the levers that passive nodes, supports, and gear modifiers attach to.
- **Damage model** — hit-based or DoT-based or both. RF, Bleed, Ignite, Poison, Caustic Arrow, Corrupting Fever, Toxic Rain are DoT. Lightning Arrow, Spark, Boneshatter, Cyclone are hit. Some skills do both (Penance Brand Ignite, Explosive Arrow's totem hit + ignite).
- **Single-target vs clear vs both** — many builds use a clear skill plus a separate single-target skill (Lightning Arrow + Lightning Rod, Rolling Magma + Holy Flame Totem, EA Ballista's clear + boss-burst phases).
- **Transfigured / awakened variants** (PoE1) — many top builds use a transfigured gem from the Lord's Labyrinth Divine Font (Boneshatter of Complex Trauma, Penance Brand of Dissipation, Shock Nova of Procession, Boneshatter Slam, Ice Shot of Penetration). The transfigured variant is often the *real* build identity, not the base skill.
- **PoE2 skill differences** — PoE2 skills are far fewer in number and often have explicit *combo* requirements (Stormblast Bolts requires a detonator skill, Living Bombs require a Fireball trigger, Frost Wall enables shotgun setups). Build identity in PoE2 is often a *skill pair* rather than a single skill.

A common pitfall: trying to scale **three or more damage vectors at once** (e.g., scaling physical damage, impale, bleed, *and* elemental conversion). Most successful builds focus on one or two. The combinatorial cost of supporting multiple scaling vectors usually means none of them get enough investment.

### 3.3 Choose class and ascendancy

Class determines starting tree position and class-specific masteries; ascendancy determines the build's signature mechanic. The pairing follows the skill, not the other way around:

- **Damage and ailment specialists** — Elementalist (Shaper of Storms / Flames, Heralds), Inquisitor (Consecrated Ground crits, ailment immunity), Trickster (Polymath, Patient Reaper).
- **Defense specialists** — Juggernaut (slow immunity, endurance generation, regen), Champion (Fortify uptime, taunt), Gladiator (Block, Versatile Combatant).
- **Skill-specific homes** — Chieftain for RF, Necromancer for minions, Pathfinder for bow / flask uptime, Saboteur for mines, Hierophant for totems and Mind Over Matter, Deadeye for projectiles.
- **PoE2 ascendancies** — Invoker (Quarterstaff conversion + Spirit), Stormweaver (lightning / Archmage), Witchhunter (crossbow / grenades), Tactician (Mercenary spirit-stacking with reservation reduction), Blood Mage (life-cost mechanics), Pathfinder (flasks / poison), Amazon (accuracy / crit), Ritualist (rings / lifeforce), Disciple of Varashta (plants), Druid (PoE2 0.4+).

A useful diagnostic: if a guide can swap skill while keeping the same ascendancy with only minor tree shifts, the ascendancy is the real engine. If the ascendancy is non-essential, the build is skill-driven and the ascendancy choice is more about defense or quality of life than identity.

### 3.4 Choose a damage scaling vector

Damage in PoE follows the formula **base × (1 + sum of increased) × (1 + each more) × penetration vs resistances × crit**, plus added flat damage applied at base. The levers are:

- **Increased modifiers** (sum together) — cheap and abundant on the tree and gear. Stacking too many runs into diminishing returns relative to More.
- **More multipliers** (multiplicative with each other) — rarer and more impactful. Comes from supports (Concentrated Effect, Awakened gems), ascendancies (Vaal Pact's Slayer leech is a *more*), and uniques.
- **Added flat damage** — hugely important early because it scales with the skill's base damage multiplier. Goldrim-tier rings with "adds X to Y physical to attacks" carry leveling phases.
- **Penetration** — flat percentage subtracted from the enemy's resistance, after Exposure debuffs. Often the highest-leverage lategame damage stat against capped pinnacle bosses.
- **Crit chance × crit multi** vs **Elemental Overload** — crit-stacking is high-ceiling; Elemental Overload (40% more damage on a crit-trigger every 8s) is low-floor and works on near-zero crit chance, often the better choice on spells with low base crit. *They do not stack* — Elemental Overload disables crit damage scaling.
- **Conversion** — converting one damage type to another lets you scale both the source and destination types. Order of operations matters: in PoE1, modifiers to the source type apply first, then conversion, then modifiers to the destination type also apply. PoE2 reverses some of this — *all conversion happens before bonuses are applied*. The agent must verify the live mechanic when explaining conversion in either game.
- **Triggers** — Cast on Crit (CoC), Cast When Damage Taken (CWDT), Cast When Stunned (CWS), Spellslinger, Mines, Totems, Ballistas. Triggers shift the action economy from "cast button" to "auto-fire," and unlock combos that wouldn't be possible with self-cast (e.g., Cospri's Malice + Ice Spear, Mjolner + Discharge).
- **Autobombers** — chain-explode setups where killing a monster triggers an explosion that kills its neighbours. Drivers include Herald of Ice, Herald of Thunder shock, Obliteration wand, Crusader chest's "Enemies you Kill have a chance to Explode," Inpulsa, Asenath's Gentle Touch. The damage doesn't have to overkill — it just has to seed a chain reaction.

### 3.5 Architect the defensive layers

Defense in PoE is a **stack of independent layers**, not a single number. A 7K life pool is meaningless if a single hit deals 30K. The layers, roughly from most to least universal:

- **Resistances** — the 75% cap is the floor, not a target. -10% per Act in PoE1's campaign is real. PoE2 has the same penalty per Act with the additional warning that **Chaos damage deals double to Energy Shield**, making chaos res mandatory for ES builds. Max-res mods on uniques (Dawnbreaker, Kaom's Heart, Aegis Aurora, +1% all max from Loreweave) push the cap to 80–90%.
- **Life pool / ES pool / hybrid** — PoE1 floors are 4–5K life campaign, 6–7K early maps, 8K+ red maps, 10K+ pinnacle. ES builds aim higher because the recharge replaces flask reliance. PoE2 numbers shift each patch.
- **Mitigation type** — Armor (effective vs many small hits, weak vs big hits — see formula on poewiki.net), Evasion (chance-based, attacks/projectiles only, does not avoid AoE without Acrobatics), Energy Shield (recharges after no-damage period, vulnerable to Chaos). PoE2 adds Block (Raise Shield) as an active stagger-avoid layer.
- **Avoidance** — Spell Suppression (PoE1, Dexterity-side, caps at 100%, halves spell damage when triggered), Block (chance-based), Dodge Roll i-frames (PoE2's universal active mitigation, ~80% of animation).
- **Recovery** — Life Regen (constant), Leech (over time, capped, instant with Vaal Pact at the cost of flasks), Recoup (delayed Recover from damage taken), Life on Hit (steady on attack builds), Eldritch Battery (mana absorbs damage before life), Mind Over Matter (30% of damage taken from Mana before Life — Hierophant signature).
- **Avoidance of effects** — ailment immunity (Pantheon, Charms PoE2, Brine King's Soul, Stormshroud, Yugul reflect), Bleed avoid (Glorious Plate, Ralakesh's Pantheon), Stun threshold, Slow / Hinder / Maim / Curse immunity.
- **On-death effects** — physical AoE corpse explosions, Volatiles, blood pools. Mitigation via Determination, Molten Shell / Steelskin guards, Cyberion's Cloak, etc.

A defensive plan in a guide should name **at least three layers** and explain how they combine. Layers should be uncorrelated — stacking two redundant Armor sources is much weaker than Armor + Block + Spell Suppression.

### 3.6 Aura and reservation budget

Auras and Heralds reserve a percentage of mana (PoE1) or Spirit (PoE2). The reservation budget caps the number of buffs running simultaneously and is one of the hardest constraints in build planning:

- **PoE1 mana reservation** — Enlighten support reduces it; Aspect of the Spider, Determination, Grace, Discipline, Wrath, Anger, Hatred, Pride, Skitterbots are the canonical auras. Mind Over Matter wants minimal reservation; Eldritch Battery wants enough mana to absorb damage. Watcher's Eye + Aura is a major endgame power source.
- **PoE2 Spirit reservation** — Spirit comes from passives, Act-quest rewards (campaign Permanent Stats — Beira, Ignagduk, Lythara give 100 total), gear (Spirit on amulet/body), and ascendancy nodes. Builds that stack Persistent Buffs (Tactician's *A Solid Plan* notable cuts reservation 50%, enabling Trinity + Berserk + Vitality I/II + Painter's Servant simultaneously) live or die by their Spirit budget. The agent should treat Spirit as a hard scarce resource in PoE2 build planning.

### 3.7 Mana sustain (PoE1)

Often overlooked. Even the best damage tree fails if you can't cast. Sustain options include Clarity, Mana Leech, Mana on Kill, Eldritch Battery + ES recharge, Indigon stacking, Archmage + mana flask use. The Comprehensive Leveling Guide and most early-tree planning should account for this — running out of mana mid-pack is a death sentence.

### 3.8 Item philosophy: uniques vs rares

Uniques are *enabling* items: they unlock a build mechanic (Inpulsa for Shock-explode, Mjolner for Discharge trigger, The Squire for 12-link, Mageblood for 4 utility flask uptime, Headhunter for steroid-on-rare-kill, Kaom's Heart for the life pool, Stormshroud for Avoid Shock). The build is designed *around* these. Rares fill the gaps — life, resistance, attribute requirements, slot-specific affixes the build needs.

A widely shared rule of thumb: a build that uses *more than 4–5 uniques* will struggle to cap resistances, hit life thresholds, or have flexibility. Mageblood and aura-stacking builds break this rule, but they're niche endgame archetypes, not general advice. Palsteron's quoted philosophy applies: every small rare upgrade is multiplicative across slots, even when each one looks insignificant.

In PoE2 specifically: Runes (socketable into gear) and Greater/Perfect rune progression replace some of PoE1's mod density. Iron, Glacial, Storm, Desert runes for resistances and stats — covered in Maxroll's Comprehensive Leveling Guide. The agent should account for runes as a separate gear axis when discussing PoE2 itemisation.

### 3.9 Crafting strategy

For high-end gear, the build needs a crafting plan. The strategies (covered in detail in `maxroll/poe1_crafting.md`) include Essence spam, Fossils with mod-blocking, Veiled Crafting / Aisling slamming, Eldritch Implicits with Conflict orbs, Harvest reforges and resistance swaps, Beastcrafting (Imprints via Craicic Chimeral, FPA duplicates), Metacrafting (Suffixes/Prefixes Cannot Be Changed, Can Have 3 Crafted Mods), Double Influence with Awakener's Orbs.

The key conceptual point: **crafting is an alternative to buying**. A build's gearing section should specify whether each piece is bought from trade or crafted, because the two paths have very different currency profiles. Pure-trade builds need divines; pure-craft builds need access to raw essences/fossils/lifeforce.

PoE2 crafting is fundamentally different (and also less mature) — Reforging Bench, Salvage Bench, Greater Jeweller's Orbs, Artificer's Orbs, Runes, Essences. The agent should not transfer PoE1 crafting reasoning wholesale to PoE2.

### 3.10 Leveling path

A complete build guide includes a *leveling section*, not just an endgame state. The campaign in PoE1 (10 acts, ~6–10 hours speedrunning, 15–25 hours casual) and PoE2 (4 acts + 3 interludes in EA, 6 acts at full release) is fundamentally different content from mapping, and the build has to function during it. The patterns:

- **Leveling skill** is often *not* the endgame skill. Rolling Magma carries Acts 1–10 for many fire builds before swapping to Penance Brand or Armageddon Brand at level 28. EA Ballista builds level with The Porcupine 6-link and Spectral Throw or Splitting Steel before transitioning. Spear builds in PoE2 use Rake all the way; ranger builds use Bow Shot and Wind Dancer.
- **Leveling uniques** are a separate ecosystem — Goldrim, Wanderlust, Tabula Rasa, The Pariah, Karui Ward, Quill Rain, Asenath's Mark, Lochtonial Caress, Lifesprig, Praxis. They drop in early Acts or are cheap on trade. Maxroll's "Best Leveling Uniques" articles cover this for both games.
- **Gear breakpoints** — movement speed boots at 10/15/25/35%, flask tiers at levels 4/10/16/23/30/40/50/60, ring/amulet upgrades.
- **Bandits and Pantheon** (PoE1) — Help Alira (resistance, mana regen, crit multi) for early-league flexibility, Help Kraityn for movement speed in race contexts, Kill All for the 2 passive points endgame (vendor recipe with Onyx + 20 Regrets).
- **PoE2 permanent buffs** — listed in the Permanent Stats from Campaign article. The 24 passives + 100 Spirit + +10% all elemental res + life/mana buffs should all be claimed during leveling.

A guide that says "level with the build's endgame skill from level 1" is usually wrong — the endgame skill almost certainly isn't available at level 1, and even if it is, leveling damage is often dominated by flat-damage scaling that the endgame build deprioritises.

### 3.11 Atlas / endgame strategy

The build's farming target informs Atlas tree allocation. A bow Deadeye optimised for clear-speed Legion farming has a different tree from a Chieftain RF farming Essences in Crystal Resonance Memories. Build guides should at minimum recommend:

- Initial Atlas point allocation (Kirac, easy-mechanic wheels first — Bestiary, Ambush, Harbinger, Heist, Essence — before going deep on Eater/Exarch).
- One or two compatible farming strategies (cross-reference `maxroll/poe1_currency.md`).
- Map mods to avoid for the build (Reflect for hit-based, no regen / no leech for life-recovery dependent, reduced flask charges for flask-piano, etc.).

### 3.12 Endgame progression tiers

A mature build guide ships *multiple budget tiers* not because the author is being thorough, but because a single endgame target is unhelpful for a player at any other budget. Fubgun's `Starter / Crit swap / Mid game / Inspired setup / Endgame no HH / Headhunter / Warden Uber Endgame` is the gold standard of progression visibility. Each tier should answer: what's the *next* upgrade I should target with my current currency? The agent, when helping a player evaluate a build, should pinpoint which tier the player is in and direct attention to the *next* upgrade, not the final endgame state.

---

## Part 4 — Vocabulary and patterns shared across builds

The terms below recur in nearly every high-quality build guide. Recognising them lets the agent parse a guide, route a player's question correctly, and spot when a player is using a term inconsistently.

**DoT cap.** Damage over Time has a per-skill cap in PoE1, scaling with level. RF, Caustic Arrow, Corrupting Fever, Bleed all hit it at sufficient investment. Once a build is at the DoT cap, *more increased damage does nothing* — only More multipliers and ailment magnitude scaling continue to help. A guide that doesn't acknowledge the DoT cap on a DoT build is incomplete.

**Conversion order.** PoE1: source-type modifiers apply, then conversion happens, then destination-type modifiers also apply, meaning both ends of the conversion benefit. PoE2 (per the Goratha Fireball Blood Mage guide): all conversion occurs *before* bonuses are applied. The agent must verify against current patch notes.

**Trigger setups.** Cast on Crit, Cast When Damage Taken, Cast When Stunned, Spellslinger, Mines, Totems, Ballistas, Cospri's Malice / Mjolner / Indigon. Each has a different action-economy footprint and damage cap mechanism (cooldowns, attack rate, stun frequency).

**Autobomber.** A clear style where killing seeds chain explosions. Drivers: Inpulsa Shock, Herald of Ice freeze-shatter explode, Crusader chest explode, Obliteration wand chaos explode, Asenath's gloves freeze + corpse-explode chain. The defining property is that the player doesn't need to hit every enemy — they hit one and the rest die. Penance Brand Ignite is a thematic example using ignite proliferation as the chain.

**Spirit cost.** PoE2's analog to mana reservation. Persistent Buffs, Heralds, Companions all cost Spirit. The total Spirit pool is the hard cap on simultaneous buffs. Tactician's reservation reduction, ascendancy Spirit grants, and gear Spirit rolls expand the budget.

**Charges.** Endurance (physical/elemental damage reduction), Frenzy (attack/cast speed + more damage), Power (crit chance + spell damage). Generation: Frenzy on Hit / on Kill, Endurance on hit taken or via Warcries, Power via specific gloves or kills. A build that uses charges as a damage source needs reliable charge generation; a build that uses them defensively (Endurance + Determination) needs stacks to stay up between hits.

**Persistent buffs vs temporary buffs.** Persistent: aura-style, always on (Vitality, Determination, Wrath, Hatred). Temporary: timed or stack-consuming (Berserk, Adrenaline, Vaal Haste, Vaal Skills, Soul Eater). A build's damage profile *with all temporaries up* and *without them* can differ by 2x — guides should specify which is being measured.

**Curses and Marks.** Curse: AoE debuff, default limit 1 per enemy, stack via passives or gear (Whispers of Doom). Marks: single-target debuff, separate from curse limit. Blasphemy: socket a curse to convert it to an aura at Spirit/mana cost. Sniper's Mark, Voltaic Mark, Despair, Vulnerability, Flammability, Frostbite, Conductivity are the bread-and-butter.

**Crit vs Elemental Overload.** Crit-scaling: stack base crit chance + crit multi for high-ceiling damage. Elemental Overload: 40% more elemental damage if you crit in the last 8 seconds, but disables crit damage scaling. Most spells (low base crit) prefer EO; most attack builds (high base crit on weapons) prefer crit-stacking. Mutually exclusive.

**Ailments.** Ignite (fire DoT), Shock (more damage taken — % varies), Freeze (status), Chill (action speed reduction), Bleed (physical DoT, scales with movement), Poison (chaos DoT, stacks). PoE2 also has Frostbite stacks, Fire Exposure. Each ailment has scaling parameters — magnitude (intensity of effect), duration, application chance, proliferation (does it spread on death/kill).

**Defensive keystones (PoE1 highlights).** *Mind Over Matter* (30% damage from Mana before Life, mana-pool dependent), *Eldritch Battery* (mana absorbs damage as ES, swaps the layers), *Acrobatics* (chance to dodge attacks, halves block/spell-block), *Iron Reflexes* (Evasion → Armour, simplifies to Armour-only), *Glancing Blows* (double block chance, but blocked hits deal 65% damage), *Vaal Pact* (instant leech, disables flask life-recovery), *Eternal Youth* (Life and ES swap recovery behavior), *Pain Attunement* (more spell damage at low life), *Resolute Technique* (no crits, no misses), *Avatar of Fire* (all damage is fire), *Elemental Equilibrium* (hitting with one element gives -25% to that, +25% to others), *Necromantic Aegis* (shield bonuses to minions), *Ghost Reaver* (leech recharges ES instead of life). Each is build-defining or build-breaking depending on context.

**Defensive keystones (PoE2).** *Bulwark*, *Heartstopper*, *Eternal Youth*, *Vaal Pact*, *Oasis* — covered in the Defence Guide. The PoE2 keystone pool is smaller but each has comparable build-warping effect.

**Gem links anatomy.** A 6-link in PoE1 is *Main skill + 5 supports*, where supports are chosen for multiplicative effect. The canonical ranking heuristic: any *More* support beats any *Increased* support of the same color, *unless* the More disables a key tag. Awakened gem variants give 1–2% More per level beyond the base. PoE2 gem links work differently — the main skill has *up to 5 socketed supports*, but the supports themselves have a global cap (each support gem can only be used a limited number of times across all skills), forcing tradeoffs.

**Cluster jewels (PoE1).** Large clusters give 8–12 passive points carrying 1–3 notables aimed at a specific tag. Medium clusters add 2 notables on a small fork. Small clusters add 1 notable. Crafting clusters via Harvest Reforge with the relevant tag is a low-investment way to acquire build-defining notables (Smoking Remains, Burning Bright, Crusader of Magic, Quick Getaway). PoE2 has no equivalent system at time of writing — verify on the wiki for the current patch.

**Influence mods and Eldritch implicits (PoE1).** Body armours, helmets, gloves, boots, rings, belts, amulets can carry Shaper / Elder / Conqueror influence mods (Hunter, Warlord, Crusader, Redeemer) with specific mod pools. Non-influenced armour pieces also carry Eldritch Implicits — Searing Exarch and Eater of Worlds, capped at Exceptional tier. Orbs of Conflict elevate them. Many top-tier builds depend on a specific influenced mod (e.g., Crusader chest's Explode mod, Hunter's "Adds Phys to Bow Attacks").

---

## Part 5 — Evaluating a build's quality

When the agent is presented with a guide, a PoB, or a player's character and asked "is this good," a structured assessment is more useful than vague approval. Diagnostic questions:

- **Is the goal articulated?** A build with no stated content target can't be evaluated against any target. If the player wants to do Uber Maven and the build is mapping-optimised, the answer is "wrong tool" not "bad build."
- **Are layers stacked or redundant?** Three Armor sources without Block, Suppression, or Evasion is redundant. Mixed layers compound multiplicatively.
- **Does the build account for the DoT cap, mana sustain, Spirit budget, and resistance cap?** Missing any of these is a significant gap.
- **Does the gear plan separate "essential uniques" from "filler rares"?** A guide that says "you need these 6 uniques" without explaining which is the build's *engine* and which is *quality of life* is hard to budget against.
- **Is there a leveling section?** A guide without one assumes the player has access to either a leveling guide elsewhere or twink gear from another character.
- **Is there a Path of Building file?** Anything without PoB is hard to verify and probably not worth following for serious play.
- **Is the changelog active?** A build last updated three leagues ago is usually unsafe — items may have been nerfed, modifiers reworked, ascendancies adjusted. The build may still work but the agent cannot vouch for the numbers.
- **Does the author engage with weaknesses?** Pohx writes that RF is weak on Maven; Fubgun writes that bow builds are squishy before Headhunter. Honest weakness disclosure is a quality signal.

When the agent is helping a player improve their *own* build (not a guide), the same framework applies. Walk through goal → skill → ascendancy → scaling → defense → recovery → reservation → gear → crafting → leveling completion → Atlas → progression tier. Identify which axis is weakest and direct the next upgrade there.

---

## Part 6 — What evolves and what doesn't

This document is durable in its structure and ephemeral in its specifics. Concretely:

- **Stable across patches:** the decision framework, the section anatomy of guides, the role of PoB, the existence of conversion / triggers / ailments / keystones, the importance of articulating a goal, the trade-off between uniques and rares, the necessity of layered defenses.
- **Volatile across patches:** every numerical value, every named skill's relative strength, every transfigured gem (added or removed each league), the meta tier list, the specific items that make a build viable, the specific Atlas tree priorities, the specific scarab / astrolabe / ore synergies. Ascendancies can be partially or fully reworked between major patches.
- **Volatile across games:** PoE1 and PoE2 share a vocabulary but differ in mechanic implementation. Conversion order, mana vs Spirit reservation, gem socket model, Trial of Ascendancy structure, crafting orb pool, Atlas mechanic, are all different. The agent should never transfer a PoE1 specific recipe to PoE2 without verification.

When the agent quotes a creator's build, it should:

1. Open the creator's profile URL to confirm the build still exists and is current-league.
2. Note the patch tag on the guide and warn the player if it's older than the current patch.
3. Treat the PoB file as the primary source of truth for numbers; treat the prose as the *reasoning* behind those numbers.
4. Cross-reference with the conceptual core (`05_build_reasoning_framework.md`, `09_itemisation_crafting.md`, `10_skills_gems_passives_ascendancies.md`) when the player asks "why does this work?"

The creators listed here are reliable for being *up-to-date* and *internally consistent*. They are not infallible. When two creators disagree about the strongest ascendancy or the best skill, the disagreement itself is information — usually it means both are viable and the choice depends on the player's goal (Section 3.1).
