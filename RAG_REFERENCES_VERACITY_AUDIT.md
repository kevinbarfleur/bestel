# RAG References Veracity Audit — Request for Deep Research

**Audit date:** 2026-05-11
**Project:** Bestel (Path of Exile AI build-analysis companion)
**Scope of audit:** 43 reference markdown files used as bundled knowledge layer for the LLM
**Audit method:** structural read by 7 parallel internal-agent passes against the Bestel repo state at 2026-05-11

---

## 1. Read this first

You are a **Deep Research agent** receiving a verification request. You **do not have access** to the Bestel repository or to the reference markdown files themselves — only to this single document. Every claim you need to verify is **fully quoted inline below**.

### Your job

For each claim in §§ 4–14, return one of:

- **Verified** — claim matches current authoritative source.
- **Stale** — claim was correct historically but no longer matches current patch.
- **Wrong** — claim is factually incorrect.
- **Inconclusive** — cannot determine from available sources; explain why.

When marking Stale or Wrong, **state the correct fact and cite a current authoritative source** (URL + retrieval date).

### What is in scope

- Path of Exile 1 (PoE1) game mechanics, items, currency, keystones, ascendancies, league mechanics, endgame bosses, patch history.
- Path of Exile 2 (PoE2) Early Access (currently 0.4) mechanics, with explicit acknowledgement that 0.5 ships on **2026-05-29 (18 days from audit date)**.
- GGG (Grinding Gear Games) company history, design philosophy, monetisation policy.
- Diablo 2 / aRPG genre history (for foundational claims).
- URL allowlist (do all these resources still exist and serve the claimed content?).
- Community trade norms and scam patterns.
- Numerical thresholds for build-readiness tiers (red maps / pinnacle / uber pinnacle).

### What is OUT of scope (do not investigate)

- Bestel's Rust + TypeScript codebase. Any claim of the form "Bestel's runtime does X" or "this tool accepts Y schema" is **internal verification** — I will handle it separately. These are collected in § 15 for completeness but you should not attempt to verify them.
- The contents of `prompts/references/creators_registry/**` (per-creator profiles, excluded by user).
- The contents of `prompts/references/maxroll/**` (Maxroll catalogs, excluded by user).
- Stylistic / pedagogical critiques of the reference files (these are noted internally but not for you to verify).

### Preferred sources, by tier

1. **Tier 1 — Canonical:** Official GGG patch notes (pathofexile.com/forum/view-forum/patch-notes), Developer Docs (pathofexile.com/developer/docs), official trade API endpoints.
2. **Tier 2 — Authoritative reference:** Official PoE Wiki (poewiki.net), PoE2 Wiki (poe2wiki.net).
3. **Tier 3 — Datamined:** poedb.tw, poe2db.tw, repoe-fork on GitHub.
4. **Tier 4 — Calculator / economy:** Path of Building (PoB) community fork, Craft of Exile, poe.ninja.
5. **Tier 5 — Trusted community guides:** Maxroll, Mobalytics, Pohx, PoE Vault, dated forum/Reddit threads from known creators.
6. **Blocked (do not cite):** pathofexile.fandom.com (legacy, abandoned), fextralife.com, RMT/SEO sites (aoeah, mmogah, iggm, ggwtb, boostmatch, sportskeeda, gamewatcher, switchbladegaming, dotesports, gamerant).

For each verdict, **also note which patch** the verification is anchored to (e.g., "verified against PoE1 patch 3.25.1 retrieved 2026-05-11").

---

## 2. Critical timeline

**Today (audit date):** 2026-05-11
**Active PoE2 patch:** 0.4
**PoE2 0.5 "Return of the Ancients" launch:** 2026-05-29 (**18 days from audit**)

The PoE2 0.5 patch is announced to overhaul the endgame mapping system, introduce a new league mechanic ("Runes of Aldur"), add new currency types (Verisium, Runic Ward, Alloys), expand the Atlas with ocean tiles, ship a ~400-node guided fortress passive tree, and introduce new pinnacle bosses.

**Implications for verification:**
- All PoE2 0.4 claims will become stale on 2026-05-29 unless 0.5 explicitly preserves the mechanic.
- All "pre-launch datamining" claims about 0.5 (Verisium, Runic Ward, Alloys, 100+ runes, 400-node tree) need to be classified: **confirmed by GGG official statement** vs **datamined only** vs **community speculation**.

---

## 3. How findings are organised

§§ 4–14 group all verifiable claims by topic. Each entry is:

```
Claim ID — short label
Source file (for traceability only; you don't need to open it)
Exact quote (verbatim from the file)
Verification source hint
Notes (if any)
```

Where multiple reference files state the same thing, only one entry is shown unless there is a discrepancy (cross-file inconsistencies are in § 13).

§ 15 lists codebase-drift items that I will verify internally — they are listed here only so you can ignore them.

---

## 4. PoE2 0.5 pre-launch claims (2026-05-29)

The reference files contain several specific claims about PoE2 0.5 content. Verify which are **confirmed by GGG** (patch notes, livestreams, forum announcements) vs **datamining only** vs **community speculation**.

### A1 — 0.5 launch date

> As of 2026-05-08, PoE2 0.4 is current. PoE2 0.5 "Return of the Ancients" releases 2026-05-29.

**Verify:** GGG official announcement / launcher / news posts. Is the firm launch date 2026-05-29? Has it slipped or accelerated?

### A2 — 0.5 Atlas restructure

> Pre-launch expectations: Atlas tree restructured into guided fortress + sub-trees per league mechanic. ~400-node tree. New pinnacle bosses. Expanded Atlas to ocean tiles. "Return of the Ancients" theme.

> The 0.5 mapping rework end of May 2026 is the next major change.

**Verify:** Are these features (guided fortress tree, ~400 nodes, ocean tiles, "Return of the Ancients" theme) confirmed by GGG announcement? Or datamined / speculative?

### A3 — 0.5 league mechanic "Runes of Aldur"

> Pre-launch expectations: new league mechanic (in-map encounter), drops Verisium, Runic Ward, Alloys, 100+ runes, integrates with 0.5 Atlas sub-tree.

**Verify:** Has GGG officially named the league mechanic "Runes of Aldur"? Are Verisium, Runic Ward, and Alloys confirmed currency / item names? Is "100+ runes" official?

### A4 — Verisium currency type

> Verisium: new currency tied to "Runes of Aldur" league mechanic. Likely augments runes / creates rune variants. [Datamining-based pre-launch]

**Verify:** Is Verisium named in any GGG official announcement, or is it datamining-only?

### A5 — Runic Ward defensive buff

> Runic Ward: defensive buff with rune-derived effects. [Datamining-based pre-launch]

**Verify:** Is Runic Ward named or described in GGG materials?

### A6 — Alloys crafting modifier

> Alloys: crafting modifier (datamining suggests). Likely augments runes or creates rune variants.

**Verify:** Is "Alloys" a confirmed PoE2 0.5 item / currency name?

### A7 — Empty stubs

The reference files include two empty stubs awaiting 0.5 content:

> Empty stub. Fill on 2026-05-29 launch day.

(One for Atlas mechanics in 0.5, one for Runes of Aldur details.)

**Verify:** Confirm 0.5 is indeed shipping with this content so the stubs can be filled at launch.

---

## 5. PoE2 0.4 current-patch claims

These are the operational claims about how PoE2 0.4 works today. Each will become stale on 2026-05-29 unless 0.5 preserves the mechanic.

### B1 — Skill gem sockets

> Skill gems live in the Skills Panel; support sockets are on the gems themselves … skill gems can have 2 to 5 sockets, expandable via Jeweller's Orbs.

**Verify on:** PoE2 Wiki Gem socket page (poe2wiki.net/wiki/Gem_socket). Confirm range 2–5 and that Jeweller's Orbs is the expansion currency. Is this still true at 0.4? Will 0.5 change it?

### B2 — Spirit as separate reservation resource

> PoE2 gives `Spirit` a central role: the PoE2 Wiki page defines it as a reserve used to activate and sustain skills with persistent effects.

> Spirit is a separate resource pool from mana. Spirit reserves auras/heralds; mana costs active skills. In PoE1, auras reduced max mana; in PoE2, they reserve separate Spirit.

**Verify on:** PoE2 Wiki Spirit page. Confirm Spirit is dedicated reservation pool, separate from mana, used for auras / heralds / persistent buffs / permanent minions / trigger gems.

### B3 — Spirit endgame total

> Endgame characters typically reach 200-350 Spirit total — this is a moving target and shifts each patch.

**Verify:** Is the 200–350 range accurate for PoE2 0.4 endgame characters? Source: Maxroll PoE2 guides, recent build videos.

### B4 — Spirit aura breakpoints

> Breakpoints table: 80–120 Spirit = 1–2 auras; 150–200 = 3 auras + 1 herald; 200–280 = 4–5 auras + 2 heralds + buff; 300+ = full stack.

**Verify:** Are these breakpoints accurate for PoE2 0.4?

### B5 — PoE2 ascendancy roster

The reference files mention these PoE2 ascendancies:

> Invoker (Quarterstaff conversion + Spirit), Stormweaver (lightning / Archmage), Witchhunter (crossbow / grenades), Tactician (Mercenary spirit-stacking with reservation reduction), Blood Mage (life-cost mechanics), Pathfinder (flasks / poison), Amazon (accuracy / crit), Ritualist (rings / lifeforce), Disciple of Varashta (plants), Druid (PoE2 0.4+).

A separate file states:

> 6 ascendancies (3 base classes × 2 ascendancies each in EA) [PoE2 0.1 launch].

**Verify:**
- What is the actual current PoE2 0.4 ascendancy roster (full list)?
- Were there 6 ascendancies in 0.1 and how many in 0.4?
- Are the descriptive tags above (Quarterstaff conversion + Spirit for Invoker, etc.) accurate?

### B6 — Weapon Set system

> PoE2 characters carry Weapon Set 1 and Set 2 with independent weapons, independent socketed gems, and per-set passive points via Book of Specialization (introduced 0.3).

> Nodes flagged as Set 1 / Set 2 are visually distinct in the passive tree UI. A node allocated to Set 1 contributes zero when Set 2 is active.

> The Book of Specialization grants a **fixed number of 'marker' points** (typically lower than your total tree allocation).

**Verify on:** PoE2 Wiki / 0.3 patch notes. Confirm Weapon Set system was introduced in 0.3 and works as described.

### B7 — Weapon swap timing

> Animation cost: ~250ms swap animation (verify per current version).

**Verify:** Is weapon-set swap animation ~250ms in PoE2 0.4? Source: in-game testing / Maxroll combat guides.

### B8 — Dodge Roll i-frames

> Dodge Roll i-frames (PoE2's universal active mitigation, ~80% of animation).

**Verify on:** PoE2 Wiki Dodge Roll page or community testing. Is dodge roll i-frame window ~80% of animation duration in 0.4?

### B9 — Trial of the Sekhemas

> Sekhemas mechanic: multi-room dungeon, Honor resource (starts full, decreases on hazard hits, 0 = run fail), Sacred Fluid / charges, Relics modify the run.

**Verify on:** PoE2 Wiki Trial of the Sekhemas page. Confirm Honor mechanic, Sacred Fluid, Relics.

### B10 — Trial of Chaos

> Chaos mechanic: arena format, escalating waves, DPS/defensive bar gated, modifiers per wave (ele-resist-down, monster-buff, etc.).

**Verify on:** PoE2 Wiki Trial of Chaos page.

### B11 — Ascendancy points total

> Typically 8 total ascendancy points per character (2 from level 30, 4 from level 60, etc. — verify current per-version).

**Verify:** Is 8 the total ascendancy points in PoE2 0.4? Are the level breakpoints (30, 60, etc.) correct?

### B12 — Combo primer/executor window

> Window between primer apply and executor fire: 1-3 seconds typical (verify per skill / current patch).

**Verify:** Is the primer-to-executor window 1–3 seconds for typical Monk / Huntress combo skills in 0.4?

### B13 — Rune families and effects

The reference files list these PoE2 rune types with associated effects:

> Iron Rune: + flat phys damage on weapons. Body Rune: + life on armours. Many more, scaling by ilvl and rarity.

> Rune families: Iron (phys damage), Body (life), Ember (fire), Glacial (cold), Storm (lightning), Blood (leech/regen), Stone (armour), Vine (evasion), Soul (mana/spirit).

**Verify on:** PoE2 Wiki Rune page or poe2db.tw. Confirm these rune families exist with the listed effects in 0.4.

### B14 — Soul Cores

> Soul Cores (corrupted-base modifier system): Late-game currency that imbues corrupted items with additional modifiers. Niche compared to runes; rare and high-impact.

> Soul Cores [0.2+] are higher-tier rune equivalents with dramatically higher stat budgets. Drop from boss/Sanctum/league mechanics.

**Verify:** Were Soul Cores introduced in PoE2 0.2? Are they "rune-like socketables" with higher stat budgets dropping from bosses/Sanctum/league mechanic?

### B15 — Talismans

> Talismans (PoE2 neck-slot mechanic): Talismans are amulets that gain charges while in play; charges can be spent for buffs / curse-immunity / etc. They are an **item slot** with currency-like charge accumulation, not an orb.

**Verify on:** PoE2 Wiki Talisman page. Confirm amulet-slot mechanic with charge accumulation.

### B16 — Gold as crafting bench currency

> Gold (PoE2 crafting bench currency): PoE2 introduced **gold** as a crafting-bench currency. Used for re-roll / crafting recipes at NPCs. Conceptually closer to a typical RPG gold than the orb economy.

**Verify:** Is gold the PoE2 crafting-bench currency in 0.4?

### B17 — PoE2 Atlas (legacy pre-0.5)

> Atlas (current as of 0.4): Infinite-atlas hex map, expanding outward from spawn point. Each hex is a Waystone-tier map.

> Pre-0.5: Waystones open maps; tier determines monster level; Precursor Tablets add map mechanics through the Map Device.

**Verify:** Confirm the PoE2 0.4 endgame structure (Waystones, Precursor Tablets, hex-grid Atlas).

### B18 — PoE2 pinnacle bosses (0.4)

> Arbiter of Ash: high max fire res (overcap), max-hit fire 7k+, bullet-hell + ground-fire dodge.
>
> King in the Mists: chill / freeze immunity, high max cold res, ice-storm dodge, fog-arena visibility constraints.

**Verify on:** PoE2 Wiki / 0.4 endgame guides. Confirm Arbiter of Ash and King in the Mists are the named pinnacle bosses in 0.4 and that the mechanics described are accurate.

### B19 — Chaos damage vs ES doubling (PoE2)

> In PoE2, the Energy Shield page states that ES acts as additional hit points that recharge, but also that bleeding / poison bypass ES and that chaos damage deals twice as much damage against ES.

**Verify on:** PoE2 Wiki Energy Shield page. Confirm: (a) bleed/poison bypass ES, (b) chaos damage does 2× to ES. Is this still true at 0.4?

### B20 — Charms vs Flasks (PoE2)

> Charms in PoE2 are NOT renamed flasks. Flasks in PoE2 are exclusively for instant life / mana recovery (no utility flasks). Charms are slotted on the belt and provide passive effects with on-demand triggers (different cadence, different scaling).

**Verify on:** PoE2 Wiki Charm and Flask pages. Confirm flasks are life/mana-only in PoE2, and charms occupy belt slots with charge-spend mechanics.

### B21 — PoE2 Pantheon does not exist

> The Pantheon system is PoE1-only. PoE2 does not currently ship Soul-of-Solaris-style passives.

**Verify:** Is there no Pantheon equivalent in PoE2 0.4?

### B22 — PoE2 cluster jewels do not exist

> Cluster jewels (Large / Medium / Small) are PoE1-only. PoE2 has its own jewel system (Time-Lost Jewels and others) but **no cluster jewel hierarchy** and no cluster-jewel-style notables.

**Verify:** Are there no PoE2 cluster jewels in 0.4? Does PoE2 have Time-Lost Jewels?

### B23 — Pinnacle boss life is engine-derived

> Pinnacle boss life in PoE2 0.5 is engine-derived; do not memorize. The bundled `pob_calc` engine surfaces the live target HP via the `enemyIsBoss=true` setting.

**Verify:** Is the boss life value variable / patch-dependent in PoE2? (This is a soft verification — just confirm boss life is not fixed.)

### B24 — Companion mechanics

> Companion: Tamed minions (Tame Beast on Huntress).

**Verify on:** PoE2 Wiki Companion page. Confirm companion system exists in 0.4 and Huntress uses "Tame Beast".

### B25 — Realmgate hub

> Realmgate: Hub for non-Atlas pinnacles (Breach, Expedition, Delirium, Ritual, …).

**Verify:** Is Realmgate the hub for non-Atlas pinnacle content in PoE2 0.4?

### B26 — Heavy Stun / Daze / Electrocute

> Heavy Stun / Daze / Electrocute: reinforced control layers; Electrocute immobilises.

**Verify:** Confirm Heavy Stun, Daze, and Electrocute exist as control mechanics in PoE2 0.4 and Electrocute immobilises.

### B27 — Magic / Rare affix budget

> PoE2: the Modifier page states that magic items can have up to two modifiers (one prefix, one suffix), and rare items up to six modifiers (three prefixes, three suffixes).

**Verify on:** PoE2 Wiki Modifier page. Confirm magic = max 2 mods (1P+1S), rare = max 6 mods (3P+3S) in PoE2 0.4.

### B28 — Tier hierarchy on rolling orbs

> PoE2 uses a **Lesser → Greater → Perfect** tier hierarchy on rolling orbs (crystallised in 0.2):
> | Greater Orb of Transmutation/Augmentation | Higher tier rolling orb (better mod-tier weighting).
> | Perfect Orb of Transmutation/Augmentation | Top tier, T1 mod weighting.

**Verify:** Was the Lesser/Greater/Perfect rolling-orb hierarchy crystallised in PoE2 0.2? Are these the correct tier names in 0.4?

### B29 — Chaos Orb behavior (PoE2)

> | Chaos Orb | Re-roll *one mod* on a Rare (NOT full re-roll). Different from PoE1 Chaos. |

> PoE2 Chaos Orb is *not* a full re-roll. It re-rolls a single mod. Conflating PoE1 Chaos and PoE2 Chaos is a common error.

**Verify on:** PoE2 Wiki Chaos Orb page. Confirm PoE2 Chaos Orb re-rolls a SINGLE mod on a rare (vs PoE1 Chaos = full re-roll).

### B30 — Orb of Alchemy cost (PoE2)

> Orb of Alchemy | Normal → Rare (4 mods); much more expensive in PoE2 vs PoE1.

**Verify:** Does PoE2 Orb of Alchemy produce a rare with 4 mods? Is it significantly more expensive / rarer than the PoE1 equivalent?

### B31 — PoE2 socket coloring

> Socket coloring doesn't exist (runes fit any rune socket).

**Verify:** Are PoE2 rune sockets color-agnostic (any rune fits any socket)?

---

## 6. PoE1 core mechanics

### C1 — Resistance calculation order

> The PoE1 Resistance page states that resistance reduces or amplifies damage of the matching type, and is calculated after avoidance and before damage reduction.

**Verify on:** poewiki.net/wiki/Resistance. Confirm: resistance applies AFTER avoidance (evasion, spell dodge), BEFORE damage reduction (armour, less multipliers).

### C2 — Resistance cap

> Verify the current resistance cap on the wiki for the relevant game and patch — caps have moved across patches.

**Verify on:** poewiki.net/wiki/Resistance. What is the current PoE1 maximum resistance cap (base 75% elemental, 75% chaos with overcap)? Has this changed in recent patches?

### C3 — Armour mechanics and cap

> Armour must never be summarised as '% physical reduction' without context. The PoE1 and PoE2 Armour pages state the same critical principle: armour is more effective against small hits than against large hits, and does not apply to DoTs by default. The PoE2 page also states that mitigation depends on the size of the hit relative to the armour value and is capped — verify the current cap on the wiki.

**Verify on:** poewiki.net/wiki/Armour and poe2wiki.net/wiki/Armour. Confirm:
- Armour formula favors small hits.
- Armour does NOT apply to DoT by default.
- What is the current armour reduction cap (PoE1 typically 90%)?

### C4 — Evasion vs accuracy + cap

> The PoE2 Wiki indicates that evasion uses an evasion-rating-against-accuracy system, and that chance to evade has a hard cap (verify the current cap on the wiki).

**Verify on:** poewiki.net/wiki/Evasion and poe2wiki.net/wiki/Evasion. Confirm evasion-vs-accuracy system and the current cap (PoE1 typically 95%).

### C5 — Spell Suppression cap and value

> The Spell suppression page states that characters have no base chance, that suppression stacks up to its cap, and that the spell damage prevented at the cap is fixed (verify the current value).

> **Cap is 100%.** Reaching 100% chance to suppress means every spell hit triggers the suppression check; the damage reduction itself is fixed at 50%.

**Verify on:** poewiki.net/wiki/Spell_suppression. Confirm:
- Base spell suppression chance is 0%.
- Cap is 100%.
- At cap, suppressed hits prevent 50% spell damage.
- Does PoE2 have spell suppression? (Audit unclear.)

### C6 — Block mechanics

> Block usually prevents the damage of a blocked hit, except for specific exceptions. But block is RNG without a guarantee mechanic.

**Verify on:** poewiki.net/wiki/Block. Confirm block prevents full damage of blocked hit and is RNG-based.

### C7 — Energy Shield (PoE2 specifics)

> In PoE2, the Energy Shield page states that ES acts as additional hit points that recharge, but also that bleeding / poison bypass ES and that chaos damage deals twice as much damage against ES.

**Verify on:** poe2wiki.net/wiki/Energy_Shield. (Same as B19; flagged here under PoE1/PoE2 comparison.)

### C8 — Conversion order of operations

> Conversion is sequential, not parallel. If gear converts 35% physical → lightning, then a support adds 100% physical → fire, the lightning conversion fires first on its 35% slice, leaving 65% physical to be reconverted by the support. Final: 35% lightning, 65% fire.

**Verify on:** poewiki.net/wiki/Damage_conversion. Confirm sequential application (gear before skill gem before support gem) and that the specific example yields 35% lightning / 65% fire.

### C9 — Increased vs More multipliers

> increased and reduced add together within their applicable pool; more and less are multiplicative against any other applicable modifiers

**Verify on:** poewiki.net/wiki/Damage_modifier (or equivalent). Confirm "increased/reduced" stacks additively within a pool, "more/less" stacks multiplicatively across pools.

### C10 — Ailment definitions

PoE1 ailments listed:

> Ignite, Chill, Freeze, Shock, Bleed, Poison, plus Scorch, Brittle, Sap.

**Verify on:** poewiki.net/wiki/Ailment. Confirm all 9 ailments exist in current PoE1 and Scorch/Brittle/Sap are the "alternate" ailments added in recent patches.

### C11 — Mind over Matter

> Mind Over Matter (30% of damage taken from Mana before Life — Hierophant signature)

**Verify on:** poewiki.net/wiki/Mind_over_Matter. Confirm MoM diverts 30% of damage taken to mana.

### C12 — Elemental Overload mechanics

> Elemental Overload (40% more damage on a crit-trigger every 8s) is low-floor and works on near-zero crit chance, often the better choice on spells with low base crit. *They do not stack* — Elemental Overload disables crit damage scaling.

> Elemental Overload requires *a crit in the last 8s*.

**Verify on:** poewiki.net/wiki/Elemental_Overload. Confirm:
- 40% more elemental damage.
- 8-second window after a crit.
- Disables crit damage multiplier scaling.

### C13 — Resolute Technique

> Resolute Technique disables crit on every attack and spell from your character. It is total — there is no partial Resolute Technique.

**Verify on:** poewiki.net/wiki/Resolute_Technique. Confirm RT disables ALL crits (no partial mechanic).

### C14 — Soul of Solaris

> Soul of Solaris reduces physical damage from hits, not from ailments. The flat 6% additional physical damage reduction applies to incoming hits only.

**Verify on:** poewiki.net/wiki/Soul_of_Solaris. Confirm:
- 6% additional phys damage reduction from hits.
- Does NOT reduce phys DoT (bleed, etc.).

### C15 — Charges (PoE1)

> Charge generation / consumption. Stacks earned by some action, consumed by another (Power/Frenzy/Endurance Charges in PoE).

> Endurance (physical/elemental damage reduction), Frenzy (attack/cast speed + more damage), Power (crit chance + spell damage).

**Verify on:** poewiki.net/wiki/Endurance_Charge, /Frenzy_Charge, /Power_Charge. Confirm the effect of each charge type.

### C16 — Trinity Support

> Trinity support (50% pen at 3 stacks).

**Verify on:** poewiki.net/wiki/Trinity_Support. Confirm Trinity grants 50% elemental penetration at 3 element stacks (or equivalent — confirm exact mechanic).

### C17 — Cast on Critical Strike cooldown

> CoC at 0.15s server cooldown caps trigger rate.

**Verify on:** poewiki.net/wiki/Cast_on_Critical_Strike_Support. Confirm 0.15-second server-side cooldown.

### C18 — Totem cap

> most totem skills cap at 4+ active totems; need ascendancy + tree to enable.

**Verify on:** poewiki.net/wiki/Totem. Confirm base totem cap (typically 1) and that 4+ requires Hierophant ascendancy / specific passives.

### C19 — Cluster jewel hierarchy

> A Medium cluster jewel hosts exactly one Small cluster jewel socket. Not two, not three. The hierarchy is: Large hosts up to 2 Mediums, each Medium hosts 1 Small.

**Verify on:** poewiki.net/wiki/Cluster_Jewel. Confirm Large → 2 Medium → 1 Small per Medium hierarchy.

### C20 — Voices unique

> Voices replaces a single jewel socket with three Small cluster jewel sockets. Each Small holds 2 notable passives.

**Verify on:** poewiki.net/wiki/Voices. Confirm:
- Replaces one passive jewel socket with 3 Small cluster sockets.
- Each Small slot accommodates 2 notable passives (verify the "2 notables" claim specifically).

### C21 — Eldritch implicit slots

> Eldritch implicits roll only on body armour, helmet, gloves, and boots. They do NOT roll on weapons, amulets, rings, or belts.

**Verify on:** poewiki.net/wiki/Eldritch_modifier. Confirm exactly 4 slot types (body, helm, gloves, boots) can receive Eldritch implicits.

### C22 — Watcher's Eye randomness

> Watcher's Eye rolls are entirely random. You cannot pre-pick which auras roll.

**Verify on:** poewiki.net/wiki/Watcher%27s_Eye. Confirm aura selection on Watcher's Eye drops is uniform random.

### C23 — Crafting bench suffix cap

> Maximum one bench-crafted suffix on a single item. The crafting bench enforces this hard cap regardless of the prefix slot count.

**Verify on:** poewiki.net/wiki/Crafting_bench. Confirm max 1 crafted suffix (and max 1 crafted prefix?) per item.

### C24 — Fracturing Orb

> A Fracturing Orb fractures one random mod on a rare item with at least four mods. You do not choose which mod gets fractured; the result is uniform random across the item's existing mods.

**Verify on:** poewiki.net/wiki/Fracturing_Orb. Confirm: requires 4+ mods, random target, fractures one mod.

### C25 — Synthesised implicits

> A synthesised implicit is a special implicit added by the Synthesis crafting process. It is mechanically distinct from the base implicit of the item and shows in a separate row on the in-game item display.

**Verify on:** poewiki.net/wiki/Synthesis_(mechanic). Confirm synthesised implicits are a distinct row.

### C26 — Heist contract modes

> Heist contracts have job tags (Lockpicking, Perception, etc.) and area levels, but there are no named 'modes' like `Apex Mode` or `Nadir Mode`. Those names are fabrications.

**Verify on:** poewiki.net/wiki/Heist. Confirm Heist contracts have NO named "modes" like Apex/Nadir.

### C27 — Maven Forgotten invitation reward

> The Forgotten Maven invitation drops uber pinnacle invitation set fragments and limited unique items, NOT Cortex maps. Cortex is a unique map associated with the Synthesis boss…

**Verify on:** poewiki.net/wiki/Maven and Cortex pages. Confirm Forgotten Maven invitation does NOT drop Cortex, and Cortex is Synthesis-related.

### C28 — Ritual deferral

> The deferral cost formula and limits change with patches and are easy to misremember. Specifically: do NOT assert a specific defer fee (10%, 5%, 15%) or hold count (50 items) without a wiki fetch confirming the current value.

**Verify on:** poewiki.net/wiki/Ritual_altar. State the **current** deferral fee and item-hold count.

### C29 — Trade pseudo stats scope

> Pseudo stats on the trade site aggregate within a single listed item, not across the build. A `pseudo.pseudo_total_life` filter on a chest searches chests where the implicit + explicit + crafted life rolls on **that one chest** sum to your minimum.

**Verify on:** pathofexile.com/api/trade/data/stats or trade documentation. Confirm pseudo-stat aggregation is per-item, not per-character.

### C30 — Flame Dash mechanics

> 3 charges, 10s recovery; Vaal Flame Dash for emergency.

**Verify on:** poewiki.net/wiki/Flame_Dash. Confirm 3 charges, 10-second recovery, Vaal variant exists.

### C31 — Action speed as master stat

> Action speed is the multiplier on **everything you do**: attack speed, cast speed, movement speed, animation speed.

**Verify on:** poewiki.net/wiki/Action_speed. Confirm action speed multiplies attack/cast/movement/animation.

### C32 — Tailwind sources

> Sources: Tailwind (Charm of Tailwind boots Eldritch implicit, Pathfinder ascendancy node, Quicksilver flask), Onslaught (item mods, flasks, gems), specific uniques.

**Verify on:** Wiki Tailwind / Onslaught pages. Confirm Tailwind comes from Charm of Tailwind boots Eldritch implicit + Pathfinder + Quicksilver. (Note: "Charm of Tailwind" boots may have a different name; verify.)

### C33 — Onslaught buff

> Frenzy (attack/cast speed + more damage)

> Onslaught (item mods, flasks, gems)

**Verify on:** poewiki.net/wiki/Onslaught. Confirm Onslaught grants attack/cast/move speed bonuses (typically 20%).

### C34 — Spectre +gem-level scaling

> spectres' damage scales with gem level. +levels from items (Mon'tregul's Grasp, Vis Mortis, +2 minion gem helmets) compound.

**Verify on:** poewiki.net/wiki/Raise_Spectre and the named uniques. Confirm:
- Mon'tregul's Grasp and Vis Mortis are PoE1 uniques that boost minion/spectre stats.
- +2 to socketed minion gem helmet implicit exists.

---

## 7. PoE1 currency and crafting mechanics

### E1 — Currency tier table (PoE1)

The reference files list these currencies with roles:

> Orb of Transmutation | Normal → Magic (1-2 mods). Cheap; bulk-craft starter step.
> Orb of Alteration | Magic re-roll. Spam-crafting (e.g., Pathfinder beacon-of-life on jewels).
> Regal Orb | Magic → Rare (adds 1 mod, keeping existing). Crafting via Alt+Regal recipe.
> Chaos Orb | Rare re-roll (full re-roll of mods). The "currency unit" of trade.
> Exalted Orb | Rare with 4-5 mods → +1 mod (preserves existing). High-end finishing currency in PoE1; less central since Settlers shifted economy.
> Divine Orb | Re-roll *numerical values* within current mod ranges. Does not change which mods exist. The dominant high-end currency in modern PoE1 since Settlers economy shift.
> Mirror of Kalandra | Duplicates an item once. Rarest currency tier; mirror-tier items are showcase pieces.

**Verify on:** poewiki.net pages for each orb. Confirm each role is accurate in the current PoE1 patch. **Of particular interest:**
- Has the Settlers (3.25) economy shift held? Is Divine currently dominant over Exalted?
- Is Chaos still "the currency unit of trade"?

### E2 — Chaos vs Divine distinction

> Chaos *re-rolls which mods exist*; Divine *re-rolls the values within mod ranges that already exist*. Mixing these up is one of the most common LLM hallucinations on PoE.

**Verify on:** poewiki.net Chaos Orb and Divine Orb pages. Confirm the distinction.

### E3 — Catalyst types

> | Catalyst (Turbulent / Tempering / etc., 7 types) | Jewellery quality with stat-tag specialisation (more X-tagged mods). |

**Verify on:** poewiki.net/wiki/Catalyst. Confirm:
- Total number of Catalyst types (the file claims 7).
- Catalyst names including Turbulent and Tempering.

### E4 — Tainted currencies

> | Tainted Chromatic / Fusing / Jeweller's | Corrupted-only socket/link/colour orbs from Heist. Can break the item. |

**Verify on:** poewiki.net Tainted Chromatic / Tainted Fusing / Tainted Jeweller's pages. Confirm Heist origin and failure-chance mechanic.

### E5 — Awakener's Orb

> Awakener's Orb fuses two influenced items, keeping one mod from each

**Verify on:** poewiki.net/wiki/Awakener%27s_Orb.

### E6 — Sacrifice Fragments (Atziri)

> Sacrifice Fragments (Dawn / Dusk / Midnight / Noon) | Atziri / Uber Atziri access.

**Verify on:** poewiki.net/wiki/Atziri. Confirm the 4 Sacrifice Fragment names and Atziri access mechanic.

### E7 — Scarabs in Settlers

> Pre-3.25: scarabs apply to the map device, modifying a single map run. Post-3.25 Settlers: scarabs remain, expanded into ~5 per league mechanic with Awakened/Greater/Polished tiers.

**Verify on:** PoE1 3.25 patch notes / poewiki.net Scarab page. Confirm:
- The Settlers scarab rework.
- "~5 per league mechanic" count.
- Awakened / Greater / Polished tier hierarchy.

### E8 — Settlers Tablets system

> | Map Tablet | Applied to towers; radiates effects to maps in range. |
> | Dispatch Tablet | Used at dock for currency exchange. |

**Verify on:** PoE1 3.25 patch notes. Confirm Tablets-on-Towers system and Dispatch Tablet for currency exchange.

### E9 — Settlers Currency Exchange

> A 1:N bulk-trade infrastructure tied to the Settlers Kingsmarch system. Send a worker with a stack of Currency A; they return with Currency B at a market rate. The most reliable bulk-conversion tool in modern PoE1, **inside the league mechanic** rather than via player trade.

**Verify:** Has the Settlers Currency Exchange persisted into the current PoE1 league? Or was it rotated out / core-integrated / removed?

### E10 — Recombinator

> Recombinator combines two items into a third with statistical heritage

**Verify on:** poewiki.net/wiki/Recombinator. Confirm the mechanic. Note that one reference file claims it is "core in PoE2 since 0.2" — verify.

### E11 — Mob Mentality cluster notable

> Mob Mentality is a Cluster Jewel notable that grants Minion Damage scaling. It rolls on cluster jewels with the `Minion Damage` enchantment, not on amulets or rings.

**Verify on:** poewiki.net/wiki/Mob_Mentality. Confirm this is a Cluster Jewel notable and not an amulet/ring mod.

### E12 — Pathfinder "beacon-of-life" jewel

The reference files cite this jewel as an example of Orb of Alteration spam target:

> Spam-crafting (e.g., Pathfinder beacon-of-life on jewels).

**Verify:** Does a Pathfinder-related jewel called "beacon-of-life" (or "Pathfinder's Beacon of Life" or similar) exist in current PoE1? Is it commonly crafted via Alteration spam?

---

## 8. PoE1 named items / uniques / bases

### F1 — Spine Bow

> | Spine Bow | High base attack speed (1.50), evasion-based. The default for evasion attack-bow builds (Lightning Arrow Deadeye etc.). |

**Verify on:** poewiki.net/wiki/Spine_Bow. Confirm 1.50 base attack speed.

### F2 — Citadel Bow

> | Citadel Bow | Strength-stack focus. Base has implicit "+X to Strength". Niche — pure-str-stack bows. |

**Verify on:** poewiki.net/wiki/Citadel_Bow. Confirm the +Strength implicit.

### F3 — Maraketh Bow

> | Maraketh Bow (Maraketh-tagged bases) | Built for league-launch crafting (Maraketh league legacy). Some have +1-2 socketed gem implicits. |

**Verify on:** poewiki.net Maraketh Bow page. Confirm the "+1–2 socketed gem level" implicit.

### F4 — Imperial Bow → Lioneye's Glare

> | Imperial Bow | Higher base damage, slower attack speed. Niche; Lioneye's Glare unique base. |

**Verify:** Does Lioneye's Glare unique still exist in current PoE1? Is its base type still Imperial Bow?

### F5 — Eternal Burgonet

> | Eternal Burgonet | Strength-armour. Highest evasion-evasion-ES craft for str-priority slots; common Eldritch helm. |

(Note: "highest evasion-evasion-ES" appears to be a typo for "highest armour-armour-ES" or similar.)

**Verify on:** poewiki.net/wiki/Eternal_Burgonet. Confirm: strength-based armour helmet, late-game base for Eldritch crafting.

### F6 — Lion Pelt

> | Lion Pelt | Dex-evasion. The default for evasion-side helmets. Eldritch helm capable. |

**Verify on:** poewiki.net/wiki/Lion_Pelt.

### F7 — Hubris Circlet

> | Hubris Circlet | Int-ES. The endgame ES helmet for caster builds. |

**Verify on:** poewiki.net/wiki/Hubris_Circlet.

### F8 — Bone Helmet implicit

> | Bone Helmet | Minion-specific implicit "+30% Minion Damage". Necromancer / Animated Guardian. |

**Verify on:** poewiki.net/wiki/Bone_Helmet. Confirm the exact value of the minion-damage implicit (+30%?).

### F9 — Astral Plate

> | Astral Plate | Pure armour, all-resistance implicit. Common pure-armour endgame craft. |

**Verify on:** poewiki.net/wiki/Astral_Plate. Confirm "all-resistance" implicit.

### F10 — Vaal Regalia

> | Vaal Regalia | Pure ES. The endgame ES chest. |

**Verify on:** poewiki.net/wiki/Vaal_Regalia.

### F11 — Cloak of Flame

> | Cloak of Flame (unique-only base concept) | Specific build pressure (RF Chieftain Pohx archetype). |

**Verify on:** poewiki.net/wiki/Cloak_of_Flame. Confirm the unique still exists and is used in RF builds (per Pohx).

### F12 — Spiked Gloves

> | Spiked Gloves | Pure armour, attack-side gloves. Crafting "Spiked Gloves with attack speed + life + resists" is a classic. |

**Verify on:** poewiki.net/wiki/Spiked_Gloves.

### F13 — Fingerless Silk Gloves

> | Fingerless Silk Gloves | Spell damage implicit. The default base for spell-builds wanting Eldritch + spell-damage implicit. |

**Verify on:** poewiki.net/wiki/Fingerless_Silk_Gloves. Confirm spell-damage implicit.

### F14 — Sorcerer Boots

> | Sorcerer Boots | Pure ES. Default for ES casters. |

**Verify on:** poewiki.net/wiki/Sorcerer_Boots.

### F15 — Two-Toned Boots

> | Two-Toned Boots | Hybrid (armour-evasion / armour-ES / evasion-ES depending on sub-type) with **+X% to two random elemental resistances** as implicit. The default for resist-flexible builds. |

**Verify on:** poewiki.net/wiki/Two-Toned_Boots. Confirm:
- 3 sub-types (armour-evasion, armour-ES, evasion-ES).
- "+X% to two random elemental resistances" implicit.

### F16 — Convoking Wand

> | Convoking Wand | Caster default. High base spell damage; common 6-link replacement when body 6L too expensive. |

**Verify on:** poewiki.net/wiki/Convoking_Wand.

### F17 — Imbued Wand

> | Imbued Wand | Higher spell crit chance implicit. |

**Verify on:** poewiki.net/wiki/Imbued_Wand. Confirm spell crit chance implicit.

### F18 — Stellar Amulet

> | Stellar Amulet | All-attribute (str/dex/int balance). Most flexible base for any build. |

**Verify on:** poewiki.net/wiki/Stellar_Amulet. Confirm all-attribute implicit.

### F19 — Onyx Amulet

> | Onyx Amulet | Strength + intelligence implicit. Niche when pure str-int build. |

**Verify on:** poewiki.net/wiki/Onyx_Amulet. (Note: "Onyx Amulet" in current PoE1 typically gives all attributes, not str+int specifically. Verify.)

### F20 — Marble Amulet

> Marble Amulet (PoE1) is the canonical chaos-res + life-regen base. The amulet rolls life regeneration as an implicit (`+(1.0–1.4)% to maximum Life per second`) and has full access to the standard amulet explicit pool…

(Note: the same reference file in another section gives a different range: `(1.2-1.6)%`. Internal inconsistency.)

**Verify on:** poewiki.net/wiki/Marble_Amulet. Confirm:
- Marble Amulet exists.
- Has life-regen implicit.
- **What is the exact range?** (1.0–1.4)% or (1.2–1.6)%?

### F21 — Stygian Vise

> | Stygian Vise | One abyssal jewel socket. Default endgame belt — the abyssal jewel mod pool is too valuable to skip. |

**Verify on:** poewiki.net/wiki/Stygian_Vise. Confirm 1 abyssal jewel socket.

### F22 — Two-Stone Ring

> | Two-Stone (Topaz/Sapphire/Ruby) | Hybrid resist implicit. Two-Stone Ring (Lightning + Cold) is common. |

**Verify on:** poewiki.net/wiki/Two-Stone_Ring. Confirm 3 sub-variants and hybrid-resist implicit.

### F23 — Diamond Ring

> | Diamond Ring | Crit chance implicit. Caster crit. |

**Verify on:** poewiki.net/wiki/Diamond_Ring. Confirm crit chance implicit.

### F24 — Heavy Quiver

> | Heavy Quiver | High implicit physical damage. |

**Verify on:** poewiki.net/wiki/Heavy_Quiver.

### F25 — Spike-Point Arrow Quiver

> | Spike-Point Arrow Quiver | Crit-multiplier implicit. Niche bow crit. |

**Verify on:** poewiki.net/wiki/Spike-Point_Arrow_Quiver. Confirm crit-multi implicit.

### F26 — Titanium Spirit Shield

> | Titanium Spirit Shield | Pure ES caster default. |

**Verify on:** poewiki.net/wiki/Titanium_Spirit_Shield.

### F27 — Tabula Rasa drop level

> Tabula Rasa drop level is 1. It is the single notable exception to 'drop level == base level' for many uniques and is intentional so brand-new characters can chase one.

**Verify on:** poewiki.net/wiki/Tabula_Rasa. Confirm drop level 1.

### F28 — Build-defining uniques

> Build-defining uniques. Single items whose mechanic transforms the build (Mageblood, Headhunter, Voidforge in PoE; Enigma, Infinity in D2).

**Verify on:** poewiki.net pages for Mageblood, Headhunter, Voidforge. Confirm they exist and are build-transformative in current PoE1.

### F29 — Timeless Jewels

> Timeless Jewels: Glorious Vanity, Lethal Pride, Brutal Restraint, Elegant Hubris, Militant Faith.

**Verify on:** poewiki.net/wiki/Timeless_Jewel. Confirm the 5 named Timeless Jewels.

---

## 9. PoE1 endgame mechanics

### G1 — Voidstones

> Voidstones (4 total) raise tier baseline from T11 → T16.

**Verify on:** poewiki.net/wiki/Voidstone. Confirm 4 Voidstones exist and they elevate map tier baseline.

### G2 — Maven mechanic

> Maven witnesses pinnacle bosses; collect 10 witnesses for The Maven encounter. Memory Game phase requires concentration — failure means no rewards.

**Verify on:** poewiki.net/wiki/The_Maven. Confirm:
- 10 witnesses required.
- Memory Game phase exists.

### G3 — Sanctum mechanic and Original Sin

> Repeatable run mechanic: enter Sanctum, navigate rooms, collect Resolve / Aureus. Relics modify run state; equippable across runs. Drops chase uniques: Original Sin, Sanctum-exclusive uniques.

**Verify on:** poewiki.net/wiki/Forbidden_Sanctum and Original_Sin pages. Confirm:
- Sanctum is repeatable with Resolve / Aureus resources.
- Original Sin is a Sanctum drop.

### G4 — Heist Replica uniques

> Heist-specific Replica unique pool. League-mechanic loot only available via Heist (e.g., Replica Soul Tether).

**Verify on:** poewiki.net/wiki/Heist and Replica_unique pages. Confirm:
- Replica uniques are Heist-exclusive.
- Replica Soul Tether exists and is a Heist drop.

### G5 — Delve mechanics

> Infinite linear depth; Sulphite-gated. Niko's mechanic; depth scales monster level.

**Verify on:** poewiki.net/wiki/Delve. Confirm infinite depth, Sulphite gating, NPC Niko, depth-monster-level scaling.

### G6 — Ritual mechanics

> Ritual altars in maps; defer rewards across multiple altars per map. Tribute-based reroll economy.

**Verify on:** poewiki.net/wiki/Ritual.

### G7 — Expedition factions

> Logbook explorer faction grind: Druids of the Broken Circle, Order of the Chayula, Knights of the Sun, Black Scythe Mercenaries.

**Verify on:** poewiki.net/wiki/Expedition. Confirm the exact 4 faction names.

### G8 — Pinnacle boss roster

> Sirus, Maven, Exarch/Eater (3.17 split), Shaper/Elder (pre-3.17, legacy), Catarina (Mastermind), Brine King / Black Star (now folded into Conquerors).

**Verify on:** poewiki.net wiki. Confirm:
- Sirus, Maven, Searing Exarch + Eater of Worlds, Mastermind (Catarina) exist as PoE1 pinnacle bosses.
- Searing Exarch + Eater of Worlds were introduced in 3.17 (Siege of the Atlas).
- Shaper / Elder are still accessible / legacy fights.
- Brine King and Black Star references — are these correct PoE1 boss names?

### G9 — Atlas Passive Tree introduction

> 3.20 (Forbidden Sanctum): Atlas Passive Tree introduced, sub-trees per league mechanic.

**Verify on:** PoE1 3.20 patch notes. Confirm Atlas Passive Tree shipped in 3.20.

### G10 — Eldritch implicits introduction

> 3.17 (Siege of the Atlas): Eater of Worlds + Searing Exarch as endgame split, atlas-wide voidstones.

> Eldritch implicit: Searing Exarch (red) and Eater of Worlds (blue) on selected armour slots.

**Verify on:** PoE1 3.17 patch notes. Confirm Eldritch implicit system introduced in 3.17.

### G11 — Cluster jewel introduction

> Cluster jewels introduced 3.10 (Delirium): Large (8-12 nodes), Medium (2-3 nodes), Small (1 node).

**Verify on:** PoE1 3.10 patch notes. Confirm cluster jewels shipped in Delirium league with these node counts.

### G12 — Aura reservation rework

> Auras pre-3.10: cost flat mana to cast, reduced max mana.

> 3.10 (Delirium) replaced auras with reservation %.

> 3.18 (Sentinel) replaced "Reduced Mana Reserved" with "Aura Effect" framing.

**Verify on:** PoE1 3.10 and 3.18 patch notes. Confirm:
- Pre-3.10 auras reduced max mana.
- 3.10 introduced percentage reservation.
- 3.18 reworked reservation efficiency into Aura Effect multipliers.

### G13 — Flask rework (Ritual + Settlers)

> 3.13 (Ritual): Pathfinder rework + flask passives consolidated.

> 3.25 (Settlers): utility flasks made permanent; passive buff once equipped, no more flask piano.

**Verify on:** PoE1 3.13 and 3.25 patch notes. Confirm:
- 3.13 reworked Pathfinder + flasks.
- 3.25 (Settlers) made utility flasks permanent / removed flask piano.

### G14 — Recoup introduction

> 3.20 (Forbidden Sanctum): Recoup mechanic introduced

**Verify on:** PoE1 3.20 patch notes.

### G15 — Spell Suppression introduction

> 3.16 (Scourge): Spell Suppression introduced as Dexterity-side cap mechanic

**Verify on:** PoE1 3.16 patch notes.

### G16 — Archnemesis rework

> 3.19 (Lake of Kalandra): Archnemesis-rare-mod overhaul affected on-death effects + rare drops

**Verify on:** PoE1 3.19 patch notes.

### G17 — Armour rework

> 3.21 (Crucible): Armour rework — armour formula tuned, Armour-vs-large-hits effectiveness reduced

**Verify on:** PoE1 3.21 patch notes. Confirm an armour rework happened in Crucible.

### G18 — Harvest rework history

> 3.11 (Harvest): Harvest crafting introduced with reforge/augment/remove recipes

> 3.13 → 3.18: progressive nerfs to Harvest "remove non-X / add X" recipes

> 3.20+: Harvest exists but reduced; Tier 17 maps + Beastcraft + essences fill the gap

**Verify on:** PoE1 3.11, 3.13–3.18 patch notes. Confirm Harvest introduction in 3.11 and progressive nerfs.

### G19 — Tier 17 maps

> 3.25+ Tier 17 maps exist as endgame tier above T16 in current PoE1.

**Verify:** Are T17 maps still present in current PoE1? When were they introduced?

### G20 — League integration timeline examples

> League content folded into core over 1-3 leagues: examples Sanctum (3.20 → core 3.21), Heist (3.12 → core 3.13), Delve (3.4 → core 3.5), Ritual (3.13 → core 3.14)

**Verify on:** PoE1 patch notes. Confirm each league → core integration version.

### G21 — Sextants → Tablets transition

> Atlas pre-3.13: sextants applied to watchstone slots

> 3.13 → 3.16: region atlas + watchstones + voidstones progression

> 3.25 (Settlers): Tablets replaced sextants, Towers as tablet-application points

**Verify on:** PoE1 patch notes. Confirm Tablet system replaced Sextants in 3.25.

### G22 — Ascendancy rework history

> Ascendancy reworks: 3.20 (Inquisitor, Pathfinder), 3.21 (Trickster, Champion), 3.22 (Hierophant, Slayer), 3.23 (Necromancer), 3.24 (Berserker), 3.25 (Pathfinder).

**Verify on:** PoE1 3.20–3.25 patch notes. Confirm which ascendancies were reworked in which patch.

### G23 — Current PoE1 league

The reference files anchor several economy claims to "Settlers of Kalguur" (3.25). What is the **current active PoE1 league as of 2026-05-11**? Is it still 3.25 or has a newer league shipped?

---

## 10. GGG, D2, and aRPG history claims

### H1 — GGG founders

> GGG (Auckland, NZ) was founded by Chris Wilson, Jonathan Rogers, Erik Olofsson, and others.

**Verify:** GGG official company history. Confirm the three named founders and approximate founding date.

### H2 — GGG monetisation policy

> GGG monetises **only via cosmetics, stash tabs, and supporter packs**. There is no XP boost, no power-up, no gameplay shortcut sold for money. This is non-negotiable for the studio.

> Wilson and Rogers, 2014 Polygon/Gamasutra quotes on monetisation.

**Verify:** Polygon and Gamasutra interviews circa 2014 with Wilson/Rogers stating pay-no-advantage policy.

### H3 — Trade Manifesto

> Trade Manifesto (Wilson, 2017)

**Verify:** Does the forum thread `pathofexile.com/forum/view-thread/2025870` contain the Wilson 2017 Trade Manifesto?

### H4 — Ruthless mode

> Ruthless dropped fewer items, fewer currencies, slower XP, no trade.

> No alteration / regal / chaos orbs; crafting via vendor recipe, fossil, essence.

**Verify on:** poewiki.net/wiki/Ruthless. Confirm Ruthless is a variant with reduced drops, restricted (not removed) alt/regal/chaos.

### H5 — PoE2 as parallel game

> PoE2 is **not a sequel that replaces PoE1**. GGG explicitly positioned it as a parallel game.

**Verify:** GGG public statements / interviews. Confirm PoE2 is officially positioned as parallel (not replacement) to PoE1.

### H6 — D2 1.10 skill synergies

> D2 1.10 introduced *skill synergies* so low-tier skills feed high-tier skills.

**Verify on:** D2 community sources / PureDiablo / Diablo Wiki. Confirm skill synergies were added in D2 patch 1.10 (released mid-2003).

### H7 — D2 difficulty tiers

> D2 Nightmare and Hell raise enemy stats and add resistance penalties.

**Verify on:** D2 community sources. Confirm Nightmare and Hell add resistance penalties (e.g., -40% and -100% resists respectively).

### H8 — D2 Hardcore origin

> D2 invented this; PoE preserves it.

**Verify:** Did D2 introduce Hardcore mode (no D1 hardcore)? Was it added in a specific D2 patch (e.g., 1.09 or 1.10)?

### H9 — D2 build-defining uniques

> Enigma, Infinity in D2 as build-defining uniques.

**Verify on:** D2 Wiki / PureDiablo. Confirm:
- Enigma is a D2 runeword granting Teleport across classes.
- Infinity is a D2 runeword enabling phys-conversion melee/minion builds.

### H10 — D2/D3 rarity tier comparison

> Rarity tiers (white/blue/yellow/orange in PoE; equivalent in D2/D3/D4/Last Epoch).

**Likely wrong.** Audit flagged: D2 rarity tiers are actually white/blue (magic) / yellow (rare) / green (set) / gold (unique). D3/D4 use different schemes. **Verify exact D2 rarity tier names** and the comparison claim. (PoE rarities: Normal/Magic/Rare/Unique — typically rendered white/blue/yellow/orange/brown.)

### H11 — D3 RoS auction house

> Auction-house games (D3 RoS-era, D2:R) were abandoned for design reasons.

**Verify:**
- Did D3 remove its auction house in Reaper of Souls era (2014)?
- Does D2:R have / not have an auction house? (Should not have.)

### H12 — PoE2 patch cadence

> PoE2 patches roughly every 4 months. Major content patches: 0.1 (early access launch), 0.2, 0.3, 0.4, 0.5 (next).

**Possibly wrong.** The implied timeline (0.1 December 2024 → 0.2 → 0.3 → 0.4 May 2025) suggests ~6–8 week cadence, not 4-month.

**Verify:** What are the actual launch dates of PoE2 0.1, 0.2, 0.3, 0.4? What is the average cadence?

### H13 — PoE2 EA launch date

> PoE2 0.1 (Early Access launch, December 2024)

**Verify:** Was the PoE2 0.1 EA launch on 2024-12-06?

### H14 — PoE2 1.0 timeline

> PoE2 1.0 target December 2026, post-ExileCon

**Verify:** Has GGG publicly committed to a December 2026 PoE2 1.0 launch?

---

## 11. URL allowlist verification

For each URL below, confirm:
- It resolves (HTTP 200).
- It hosts the content the reference files claim.
- It is still the canonical source as of 2026-05-11.

### J1 — Tier 1 (Canonical)

- `https://www.pathofexile.com/forum/view-forum/patch-notes` — official patch notes forum.
- `https://www.pathofexile.com/forum/view-forum/2212` — claimed as PoE2 patch notes section. **Verify** this forum ID is the PoE2 section.
- `https://www.pathofexile.com/developer/docs` — Developer Docs.
- `https://www.pathofexile.com/api/trade/data/static` — trade static data endpoint.
- `https://www.pathofexile.com/api/trade/data/stats` — trade stats endpoint.

### J2 — Tier 2 (Wikis)

- `https://www.poewiki.net/wiki/Path_of_Exile_Wiki` — PoE1 wiki main.
- `https://www.poe2wiki.net/wiki/Path_of_Exile_2_Wiki` — PoE2 wiki main.

### J3 — PoE1 specific wiki pages cited

- `https://www.poewiki.net/wiki/Resistance`
- `https://www.poewiki.net/wiki/Armour`
- `https://www.poewiki.net/wiki/Spell_suppression`
- `https://www.poewiki.net/wiki/Passive_skill`
- `https://www.poewiki.net/wiki/Support_gem`
- `https://www.poewiki.net/wiki/Crafting`
- `https://www.poewiki.net/wiki/Rarity`
- `https://www.poewiki.net/wiki/Item_level`
- `https://www.poewiki.net/wiki/Metamod`
- `https://www.poewiki.net/wiki/Essence`
- `https://www.poewiki.net/wiki/Fossil`
- `https://www.poewiki.net/wiki/Map`
- `https://www.poewiki.net/wiki/Atlas_of_Worlds`

### J4 — PoE2 specific wiki pages cited

- `https://www.poe2wiki.net/wiki/Gem`
- `https://www.poe2wiki.net/wiki/Gem_socket`
- `https://www.poe2wiki.net/wiki/Spirit`
- `https://www.poe2wiki.net/wiki/Passive_skill_tree`
- `https://www.poe2wiki.net/wiki/Ascendancy_class`
- `https://www.poe2wiki.net/wiki/Modifier`
- `https://www.poe2wiki.net/wiki/Item`
- `https://www.poe2wiki.net/wiki/Crafting`
- `https://www.poe2wiki.net/wiki/Armour`
- `https://www.poe2wiki.net/wiki/Evasion`
- `https://www.poe2wiki.net/wiki/Atlas`
- `https://www.poe2wiki.net/wiki/Precursor_tablet`

### J5 — Tier 3 (Datamined)

- `https://poedb.tw/us/Modifiers`
- `https://poedb.tw/us/Craft_Modifiers`
- `https://poe2db.tw/us/Modifiers`
- `https://poe2db.tw/us/Items`
- `https://poe2db.tw/us/Gem`
- `https://github.com/repoe-fork/repoe`
- `https://repoe-fork.github.io/poe2/`

### J6 — Tier 4 (Calculators / Economy)

- `https://www.craftofexile.com/`
- `https://poe.ninja/`

### J7 — Tier 5 (Trusted creator guides)

- `https://maxroll.gg/poe`
- `https://maxroll.gg/poe2`
- `https://mobalytics.gg/poe`
- `https://mobalytics.gg/poe-2`
- `https://www.pohx.net/`

### J8 — Blocked sources

The reference files list these as RMT / SEO-blog domains that should be blocked from `web_fetch`:

> | aoeah.com | RMT-adjacent SEO blog. |
> | mmogah.com | RMT/boosting SEO. |
> | iggm.com | RMT-adjacent SEO. |
> | ggwtb.com | RMT/SEO. |
> | boostmatch.com | Boost service SEO. |
> | sportskeeda.com | Generic gaming-news SEO; PoE coverage shallow and frequently wrong. |
> | gamewatcher.com | Generic gaming-news SEO. |
> | switchbladegaming.com | Low-quality SEO. |
> | dotesports.com | News-flavoured SEO; PoE coverage frequently wrong on mechanics. |
> | gamerant.com | Generic gaming SEO. |

The reference files also claim:

> Blocked hosts (`fandom.com`, `*.fextralife.com`, RMT, AI-aggregator pages) are **never** valid citations.

> `pathofexile.fandom.com` — the legacy wiki ranks high in search but is largely abandoned and outdated.

**Verify:**
- Is `pathofexile.fandom.com` still abandoned (or has it been revived)?
- Are there additional RMT / SEO domains targeting PoE that should be added (e.g., new gold-selling sites)?
- Is fextralife still a low-quality source for PoE?

---

## 12. Community norms (trade etiquette, scams)

### K1 — Trade whisper auto-template

> Hi, I would like to buy your *Shaper's Touch* listed for *3 divine* in Settlers (stash tab "PriceLT"; position: left N, top M)

**Verify on:** pathofexile.com/trade. Confirm this is the current auto-whisper format.

### K2 — Courtesy windows

> Live-search reply window: ≤1 minute.
> Recent listing reply window: &lt;1 hour.
> Cold-but-valid listing: 1–24 hours.
> Dead listing: 1+ days.
> Hideout AFK is convention.

**Verify:** Are these courtesy windows the current PoE community norm? (Source: reddit r/pathofexile, official trade FAQ, community guides.)

### K3 — Item-swap is most common scam

> Item-swap is "the most common scam in PoE". Veteran players READ the mods *every accept*, especially after any seller modification.

**Verify:** Is item-swap (seller modifies item in trade window after buyer agrees) widely reported as the most common scam pattern? (Source: r/pathofexile, GGG forum reports.)

### K4 — RMT-flagged items risk

> Accepting RMT-flagged items carries account-action risk

**Verify on:** GGG Terms of Service / forum policy posts. Confirm GGG policy on accepting RMT-flagged items.

### K5 — GGG communication policy

> GGG never DMs in-game. Anything asking for credentials is a scam.

**Verify on:** GGG support documentation. Confirm GGG does not initiate in-game DMs.

### K6 — Report flow

> Use the in-game `/report &lt;name&gt;` chat command + GGG ticket. Recovery is slow but possible.

**Verify:** Is `/report` the correct in-game command? Is GGG ticket-based recovery available?

### K7 — Currency Exchange eliminates off-rate scam

> Settlers Currency Exchange (PoE1) and PoE2 Currency Exchange formalize rates, reducing off-rate scam risk.

> Both systems eliminate certain scam vectors — the 'off-rate currency trade' scam doesn't apply when the rate is posted.

**Verify:** Are both systems still active (PoE1 Settlers Currency Exchange + PoE2 Currency Exchange in 0.4)? Do they formalise rates as described?

---

## 13. Numerical threshold tables (DPS / EHP / max-hit)

These are community-consensus "build readiness" floors for PoE1 content tiers. They are by nature soft and patch-volatile. Verify against current Maxroll guides / Pohx HC bars / community VODs to confirm they remain reasonable for the **current PoE1 league**.

### L1 — Red maps universal floor (T14–16)

> Max resists: 75% all (78%+ chaos preferred).
> Spell Suppression OR Block: 90–100% / 60%+.
> Movement speed: 130%+.
> Phys mitigation (armour or guard): 30%+.
> Ailment immunity (chill / freeze / shock at minimum): yes.

**Verify:** Are these the current community floors for red mapping?

### L2 — Red maps hit-based builds

> Total DPS (boss profile, single target): 1.5M+.
> EHP (HP+ES+MoM-equivalent): 7–8k.
> Max hit by element: 7k+ per type.
> Recovery (life regen / leech / recoup): 1.5–2k life/sec.

### L3 — Red maps DoT builds

> Total DoT DPS (against pinnacle): 800k+.
> EHP: 6–7k.
> Max hit by element: 8k+ per type.
> Recovery rate: 2k+ life/sec.

### L4 — Red maps ailment-stack builds

> Hit damage: 200k–500k per hit.
> Ailment DPS (effective): 1M+.
> Ailment chance: 100%.
> EHP: 7–8k.

### L5 — Red maps minion builds

> Total minion DPS (boss profile): 1.5M+.
> Minion EHP: 4–6k per minion (spectres); low for SRS, high for spectres / golems.
> Player EHP: 6–7k.
> Minion supports: 5+ effective links.

### L6 — Red maps PoE2 (0.4) baseline

> Total DPS (single target): 200k–500k (PoE2 baseline DPS ~3–4× lower than PoE1 due to slower combat).
> EHP: 4–6k (PoE2 EHP scales differently — Spirit-fed defensive auras, Grim Feast for ES).
> Max resists: 75%+ (chaos overcap critical for ES builds; chaos damage deals **double** to ES in PoE2).
> Movement / dodge fluency: yes (PoE2 dodge-roll i-frame fluency replaces some passive defence).
> Recovery: 800–1500 life/sec (PoE2 recovery is harder to scale; Spirit-aura recovery (Vitality I/II) is the main lever).

**Verify:** Are PoE2 0.4 thresholds in this range per current Maxroll PoE2 guides?

### L7 — Sub-uber pinnacle universal floor

> Max resists: 75–80% all (chaos overcap mandatory for Sirus / Maven).
> Spell Suppression OR Block + Spell Block: 100% / 75% spell block.
> Phys mitigation: 50%+ effective (armour + endurance + fortify).
> Movement / dash fluency: dash + flame dash + leap.
> Ailment immunity: yes (Pantheon, charm, Brine King's Soul, etc.).
> Recovery between phases: sufficient flask charges or auto-regen (30–60 sec phase intervals).

### L8 — Sub-uber pinnacle hit-based

> Total DPS (boss profile, single target): 5M+.
> EHP (HP+ES+MoM-equivalent): 9–12k.
> Max hit by element: 10k+ per type.
> Recovery: 3k+ life/sec.

### L9 — Per-pinnacle PoE1 quirks

> Sirus: chaos res overcap mandatory, phase 4-5 die-beam dodge tight, 5M+ DPS comfortable.
> Maven: 10k+ EHP, Memory Game phase requires concentration, 5M+ for phase-skip range.
> Eater of Worlds: max ele resists (fire, has cold-shock-degen), bullet-hell projectile-dodge phase, 4–7M DPS, low-mobility hard counter.
> Searing Exarch: max fire res (overcap), ball-and-chain bullet-hell phase, ground fire to dodge, 4–7M DPS.
> Shaper / Elder: 9–10k EHP, max ele resists, chaos overcap, 4M+ Shaper / 5M+ Elder.

**Verify each:** Are these mechanic descriptions and damage ranges still accurate for the current PoE1 patch?

### L10 — Per-pinnacle PoE2 (0.4) quirks

> Arbiter of Ash: high max fire res (overcap), max-hit fire 7k+, bullet-hell + ground-fire dodge, 500k–1M DPS.
> King in the Mists: chill / freeze immunity, high max cold res, ice-storm dodge, fog-arena visibility constraints, 500k–1M DPS.

**Verify:** Are these the correct PoE2 0.4 pinnacle bosses and bars?

### L11 — Uber pinnacle universal floor

> Max resists: 80%+ all (chaos overcap mandatory).
> Spell Suppression 100% AND Block 75%+ Spell Block.
> Phys mitigation: 60%+ effective.
> Movement / dash fluency: dash + flame dash + Fortify-on-dash + leap.
> Ailment immunity: yes, no exceptions (Pantheon + charm/flask + immunity stacking).
> Recovery between phases: 5k+ life/sec sustained.

### L12 — Uber hit-based

> Total DPS (boss profile, single target): 12–20M+.
> EHP (HP+ES+MoM-equivalent): 12–15k+.
> Max hit by element: 14k+ per type (Uber Eater deathwave / Uber Exarch beam).
> Recovery rate: 5k+ life/sec.
> Movement speed: 140%+.
> Effective HP per hit: 7k+ life leech burst capacity.

### L13 — Uber DoT

> Total DoT DPS: 3–5M+.
> EHP: 11–13k.
> Max hit by element: 14k+ per type.
> Recovery rate: 5k+ life/sec.

### L14 — Uber pinnacle damage ratios

> Uber HP / phase scaling 2-4× higher than sub-uber.

**Verify:** Is the 2–4× difference between sub-uber and uber pinnacle still accurate?

### L15 — HC adjustments

> HC: bars +30–40% across the board; redundant defences preferred; phase-skip strategies prioritised.

> HC uber bars: +50% across the board, defensive redundancy 3+ layers, phase-skip prioritised.

### L16 — Mageblood / Headhunter / Original Sin require trade

> SSF: Mageblood / Headhunter / Original Sin are unattainable without trade.

**Verify:** Are these uniques effectively unattainable in SSF in the current PoE1 patch?

### L17 — RF Chieftain regen claim

> RF Chieftain: 12–15% effective life regen (ascendancy + Beacon of Madness + Watcher's Eye + tree).

**Verify on:** Pohx RF guides. Is 12–15% effective life regen achievable on RF Chieftain in current PoE1?

### L18 — RF Maven hard limit

> RF struggles on Maven Memory Game and pinnacle adds. Sirus / Maven attempts often do require a swap to Vaal RF or trigger-spell DPS.

**Verify:** Is this RF limitation widely acknowledged in current Pohx / community RF guides?

---

## 14. Internal cross-file inconsistencies

These are places where two or more reference files state different things. Resolve by determining which is correct.

### M1 — Staleness threshold (7 days vs 30 days)

One reference file:
> If a fact relies on data older than 7 days (cached wiki query, repoe-fork snapshot, trade index), say so.

Another reference file (different section):
> RePoE `fetched_at > 30 days` triggers warning

**Resolve:** Should the universal staleness flag be 7 days, 30 days, or different per source type? (Note: 7d for general queries, 30d for RePoE snapshots may both be intentional.)

### M2 — Marble Amulet implicit range

Section 1 of a reference file:
> The amulet rolls life regeneration as an implicit (`+(1.0–1.4)% to maximum Life per second`)

Section 2 of the same file:
> exact value (e.g. `(1.2-1.6)%`) ... must be confirmed

**Resolve:** What is the **correct current** Marble Amulet life-regen implicit range? (Already in F20.)

### M3 — D2 vs PoE rarity tier claim

> Rarity tiers (white/blue/yellow/orange in PoE; equivalent in D2/D3/D4/Last Epoch).

(D2 actually has different rarity names. See H10.)

### M4 — Rule of Three illustration

> GGG ships in threes: Three-way endgame interactions (Shaper / Elder / Conqueror, Searing Exarch / Eater of Worlds, …)

**Issue:** Shaper + Elder = 2; Conqueror = 4 (Crusader, Hunter, Redeemer, Warlord); Searing Exarch + Eater of Worlds = 2. None of these form a "Rule of Three" group of 3.

**Verify:** Is the "Rule of Three" actually a GGG-stated design principle? If so, what are the correct illustrative examples? (Three primary attributes STR/DEX/INT, three damage types fire/cold/lightning, three ascendancies per class in PoE1 — these would all match better.)

### M5 — Low Life / Righteous Fire as keystones

> LL keystone: Low Life
> RF keystone: Righteous Fire

**Issue:** Low Life is a STATE (typically achieved via Pain Attunement keystone, Shavronne's Wrappings unique, or similar). Righteous Fire is a SKILL gem.

**Verify:** Confirm:
- "Low Life" is not itself a keystone (Pain Attunement is the keystone associated with LL).
- "Righteous Fire" is a skill gem, not a keystone.

### M6 — PoE2 ascendancy count

One reference file claims PoE2 0.1 launched with **6 ascendancies (3 base classes × 2 each in EA)**. Another lists **10 PoE2 ascendancies** without version pinning (Invoker, Stormweaver, Witchhunter, Tactician, Blood Mage, Pathfinder, Amazon, Ritualist, Disciple of Varashta, Druid 0.4+).

**Resolve:** What is the correct PoE2 0.4 ascendancy count and roster? (Already in B5.)

### M7 — PoE2 patch cadence

One reference file claims PoE2 patches "roughly every 4 months". Another timeline (0.1 Dec 2024 → 0.2 → 0.3 → 0.4 May 2025) implies ~6–8 weeks.

**Resolve:** What is the actual PoE2 patch cadence based on real launch dates? (Already in H12 / H13.)

---

## 15. OUT OF SCOPE — internal codebase verification (do NOT investigate)

These are claims the audit found about Bestel's Rust + TypeScript codebase. They will be verified internally by the primary Claude agent. Listed here for completeness only.

- Mechanic token "autobomber" in `27_response_contracts.md` line 35: claimed to exist but not found in `crates/bestel-core/src/pob/semantic.rs`.
- Defining-unique category "enabler" in `27_response_contracts.md` line 39 and `32_build_sheets.md` line 66: appears in sheet finalize schema but not in build identity extraction.
- Blocklist `fandom.com` and `*.fextralife.com` in `27_response_contracts.md` line 99: not present in `crates/bestel-core/src/llm/tools.rs` FETCH_BLOCKLIST.
- Tool storm cap of 3 retrieval calls per turn in `28_tool_failure_policy.md`: code shows only KB_SEARCH-specific cap.
- `pob_calc` timeout claimed as 8 seconds in `25_pob_engine_integration.md`: code shows 60 seconds.
- RePoE daily refresh task claimed in `13_retrieval_playbooks.md`: not verified.
- Tool output schemas for `pob_calc`, `repoe_lookup`, `trade_resolve_stats`: described in prose, depend on external engine + datamined Rust struct definitions.
- Panel marker regex syntax in `30_panel_marker_grammar.md`: claims to match `crates/bestel/ui/src/api/markdown.ts` regex.
- Build sheet fingerprint format `<asc>:<skill>:<sorted_uniques>` in `32_build_sheets.md`: depends on `crates/bestel-core/src/sheets/fingerprint.rs`.
- PoB submodule pins (PathOfBuildingCommunity v2.65.0, PathOfBuilding-PoE2 v2.49.3) in `25_pob_engine_integration.md`.
- Runtime tag injection for `Build sheet:` claimed in `32_build_sheets.md`.
- Interview submission format parsing claimed in `32_build_sheets.md`.

These are tracked separately. **Skip them in your verification work.**

---

## 16. Meta-questions you should answer

Beyond the per-claim verification, please answer the following high-level questions:

1. **Current PoE1 league.** What is the active PoE1 league as of 2026-05-11? (Critical for resolving all Settlers / 3.25-anchored economy claims.)
2. **PoE2 0.5 confirmation.** Has GGG officially confirmed 2026-05-29 as the firm launch date? If not, what is the latest official communication?
3. **PoE2 0.5 feature reality.** Of the announced 0.5 features (guided fortress Atlas, ~400 nodes, ocean tiles, Runes of Aldur league, Verisium, Runic Ward, Alloys, 100+ runes, new pinnacles), how many are **confirmed by GGG official statement** vs **datamining only** vs **community speculation**?
4. **Path of Building / PoB Community / PoB-PoE2 forks.** What are the current public versions / release cadences? Is the PoE2 fork keeping pace with GGG patches?
5. **Source availability.** Of the URL allowlist in § 11, are any sources currently offline, paywalled, or in transition?
6. **Surprising drift.** What did you find that is NOT in this audit document — i.e., reference-file claims you'd flag as suspect that I missed?

---

## 17. Output format expected

Please return:

1. A verdict table (Claim ID + Verified/Stale/Wrong/Inconclusive + 1-line note).
2. For Stale or Wrong verdicts, a full correction with:
   - Correct fact.
   - Source URL.
   - Retrieval date.
3. Answers to the meta-questions in § 16.
4. Any additional drift findings under "Surprising drift".

There is no length limit on your response. Be exhaustive.

---

*End of audit request. ~290 individual claims indexed across 14 thematic sections.*
