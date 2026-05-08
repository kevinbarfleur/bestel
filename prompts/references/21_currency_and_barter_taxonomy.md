---
description: Conceptual map of PoE currency by *role* (rolling, quality, socket, link, high-end, fragments, splinters, scarabs, tablets). PoE2 narrower currency tree, runes, soul cores, Verisium (0.5).
fetch_when: User asks about a specific orb's role, "what's X used for?", crafting cost, trade-economy questions, or you're recommending a craft action. Always cross-check live exchange rates via `web_fetch poe.ninja` for actual values — this doc is the *taxonomy*, not the *price list*.
---

# 21 — Currency and barter taxonomy

This doc is about **role and structure**. For current exchange rates, fetch poe.ninja. For per-orb rules-of-text, fetch the wiki. The map below is for **reasoning about which orb solves which problem** — not for citing prices.

The single most useful question this doc answers: *"Which currency type is the right tool for this crafting / economic step?"*

## Why currency in PoE is different

Three properties make PoE's currency different from typical RPG gold:

1. **Currency is barter** — every orb has a crafting use *and* a trade value. There is no "gold". Chaos Orbs, Divine Orbs, etc. function as both.
2. **Each currency has a specific crafting purpose** — "Chaos Orb re-rolls a rare", "Divine Orb re-rolls numerical values within mod ranges". You cannot substitute one for another.
3. **Inflation/deflation is league-driven** — currency value drifts dramatically across leagues. A "Chaos pricing" guide from last league is structurally rotted.

The result: knowing the **role** of a currency is a stable skill; knowing the **price** is league-pinned and stale within weeks.

---

## PoE1 — currency by role

### Rolling currencies (modify mods on items)

The bread-and-butter of crafting. Each one solves a different sub-problem.

| Currency | Role |
|---|---|
| Orb of Transmutation | Normal → Magic (1-2 mods). Cheap; bulk-craft starter step. |
| Orb of Augmentation | Magic with 1 mod → Magic with 2 mods. Adds, not re-rolls. |
| Orb of Alteration | Magic re-roll. Spam-crafting (e.g., Pathfinder beacon-of-life on jewels). |
| Orb of Alchemy | Normal → Rare (4 random mods). Map crafting; cheaper than Chaos for that role. |
| Regal Orb | Magic → Rare (adds 1 mod, keeping existing). Crafting via Alt+Regal recipe. |
| Chaos Orb | Rare re-roll (full re-roll of mods). The "currency unit" of trade. |
| Orb of Annulment | Rare → remove 1 random mod. Used in finishing slams. |
| Orb of Scouring | Item → Normal. Strip a craft and start over. |
| Orb of Chance | Normal → Magic / Rare / Unique with weighted probability. Niche; targeted Headhunter chance recipes (Leather Belt) etc. |
| Vaal Orb | Item → Corrupted (random outcome: re-roll / +1 socket / no-change / brick). Permanent change; cannot un-corrupt. |
| Exalted Orb | Rare with 4-5 mods → +1 mod (preserves existing). High-end finishing currency in PoE1; less central since Settlers shifted economy. |
| Divine Orb | Re-roll *numerical values* within current mod ranges. Does not change which mods exist. The dominant high-end currency in modern PoE1 since Settlers economy shift. |
| Mirror of Kalandra | Duplicates an item once. Rarest currency tier; mirror-tier items are showcase pieces. |

**Key conceptual distinction**: Chaos *re-rolls which mods exist*; Divine *re-rolls the values within mod ranges that already exist*. Mixing these up is one of the most common LLM hallucinations on PoE.

### Quality currencies (improve item quality, separate from mods)

| Currency | Slot |
|---|---|
| Whetstone | Weapon quality (more physical damage). |
| Armourer's Scrap | Armour quality (more armour/evasion/ES). |
| Glassblower's Bauble | Flask quality. |
| Catalyst (Turbulent / Tempering / etc., 7 types) | Jewellery quality with stat-tag specialisation (more X-tagged mods). |
| Gemcutter's Prism | Skill gem quality. |

Quality matters because it scales the local property by quality % — a 20%-quality bow rolls higher base damage than a 0%-quality one.

### Socket / link / colour currencies

| Currency | Role |
|---|---|
| Jeweller's Orb | Re-roll *number* of sockets. |
| Orb of Fusing | Re-roll *links* between sockets. |
| Chromatic Orb | Re-roll *colours* of sockets. |
| Vaal Orb (corruption variant) | Can corrupt to 6L; risky. |
| Tainted Chromatic / Fusing / Jeweller's | Corrupted-only socket/link/colour orbs from Heist. Can break the item. |

Most builds need a 6L body armour, often 5L weapon. Socket-coloring on off-stat bases (Sorcerer Boots with red sockets for str-str-int) is a classic crafting puzzle.

### Fragments and pinnacle keys

| Item | Function |
|---|---|
| Sacrifice Fragments (Dawn / Dusk / Midnight / Noon) | Atziri / Uber Atziri access. |
| Mortal Fragments (Hope / Grief / Rage / Ignorance) | Uber Atziri. |
| Fragment of Constriction / Enslavement / Eradication / Purification | Shaper / Elder. |
| Pure Breachstone (per breach lord, 4 tiers) | Domain bosses; XP-farming via Pure Esh's. |
| Maven's Writ | Uber Maven. |
| Eldritch Fragment (Crux of Fire / Hunger of Worlds / etc.) | Eater / Exarch ubers. |

Fragments are the bottleneck for pinnacle access — supply is rate-limited by atlas tree allocation.

### Splinters and shards

Currency that comes in fractional units; combine N splinters into 1 base currency.

| Splinter type | Combine to |
|---|---|
| Splinter of Chayula / Esh / Tul / Uul-Netol / Xoph | Breachstone. |
| Timeless Splinter (Ahkeli / Lukaru / Maraketh / etc.) | Timeless Emblem. |
| Legion Splinter | Timeless Splinter (multi-step). |
| Simulacrum Splinter | Simulacrum (Delirium endgame). |

Splinters drop in stacks during the matching mechanic, gradually building up to a boss-key item.

### Scarabs (atlas modifiers)

Pre-3.25: scarabs apply to the map device, modifying a single map run. Post-3.25 Settlers: scarabs remain, expanded into ~5 per league mechanic with Awakened/Greater/Polished tiers.

| Scarab family (examples) | Effect domain |
|---|---|
| Cartography Scarabs | Map drop modifiers. |
| Breach Scarabs | Breach spawn modifiers. |
| Legion Scarabs | Legion encounter modifiers. |
| Beyond Scarabs | Beyond mob spawning. |
| Anarchy Scarabs | Rogue exiles. |

Scarabs are a major lever in farming-strategy optimisation — pick scarabs that match your atlas tree allocation.

### Tablets (post-3.25 Settlers)

| Tablet type | Effect |
|---|---|
| Map Tablet | Applied to towers; radiates effects to maps in range. |
| Dispatch Tablet | Used at dock for currency exchange. |

Tablets replaced Sextants in 3.25. The tablet → tower → map-radius pattern is the modern equivalent of "applying sextants to your watchstones".

### Currency Exchange (Settlers)

A 1:N bulk-trade infrastructure tied to the Settlers Kingsmarch system. Send a worker with a stack of Currency A; they return with Currency B at a market rate. The most reliable bulk-conversion tool in modern PoE1, **inside the league mechanic** rather than via player trade.

---

## PoE2 — currency by role (current as of 0.4)

PoE2 deliberately narrows the currency tree. The conceptual deltas:

### Rolling currencies — tier system

PoE2 uses a **Lesser → Greater → Perfect** tier hierarchy on rolling orbs (crystallised in 0.2):

| PoE2 currency | Role |
|---|---|
| Orb of Transmutation | Normal → Magic (1 mod). |
| Orb of Augmentation | Magic 1-mod → 2-mod. |
| Regal Orb | Magic → Rare (adds 1 mod, keeping existing). |
| Greater Orb of Transmutation/Augmentation | Higher tier rolling orb (better mod-tier weighting). |
| Perfect Orb of Transmutation/Augmentation | Top tier, T1 mod weighting. |
| Orb of Alchemy | Normal → Rare (4 mods); much more expensive in PoE2 vs PoE1. |
| Chaos Orb | Re-roll *one mod* on a Rare (NOT full re-roll). Different from PoE1 Chaos. |
| Exalted Orb | Add a mod to a Rare (similar to PoE1 Exalt; cheaper relatively). |
| Divine Orb | Re-roll numerical values within mod ranges (same as PoE1). |
| Vaal Orb | Corrupt (same role as PoE1). |
| Mirror of Kalandra | Duplicate (same role as PoE1). |

**Rot trigger for AI**: PoE2 Chaos Orb is *not* a full re-roll. It re-rolls a single mod. Conflating PoE1 Chaos and PoE2 Chaos is a common error — see `26_validation_and_self_correction.md` Rule 4.

### Runes (socket-fed gems on items)

PoE2 introduces **Runes** — items socketed into bases that grant a passive effect. Conceptually a "currency you find" rather than "currency you craft with":

- Iron Rune: + flat phys damage on weapons.
- Body Rune: + life on armours.
- Many more, scaling by ilvl and rarity.

Runes are not crafting currencies — they're modifiers applied to bases via specific sockets the base provides.

### Soul Cores (corrupted-base modifier system)

Late-game currency that imbues corrupted items with additional modifiers. Niche compared to runes; rare and high-impact.

### Talismans (PoE2 neck-slot mechanic)

Talismans are amulets that gain charges while in play; charges can be spent for buffs / curse-immunity / etc. They are an **item slot** with currency-like charge accumulation, not an orb.

### Gold (PoE2 crafting bench currency)

PoE2 introduced **gold** as a crafting-bench currency. Used for re-roll / crafting recipes at NPCs. Conceptually closer to a typical RPG gold than the orb economy.

### 0.5 additions (2026-05-29)

⚠️ Pre-launch from GGG announcement materials — verify on launch day:

- **Verisium**: new currency tied to "Runes of Aldur" league mechanic.
- **Runic Ward**: defensive buff with rune-derived effects.
- **Alloys**: crafting modifier (datamining suggests). Likely augments runes or creates rune variants.

Fill `poe2/06_runes_of_aldur.md` with authoritative content on launch.

---

## Cross-currency taxonomy table (both games)

A unified view by **role**, not by game. Useful when reasoning across the games.

| Role | PoE1 | PoE2 |
|---|---|---|
| Re-roll all mods on Rare | Chaos Orb | (no direct equivalent — closest is full Scour + Alch chain) |
| Re-roll one mod on Rare | (no direct equivalent — closest is Annul + Exalt) | Chaos Orb |
| Add a mod to Rare | Exalted Orb | Exalted Orb |
| Re-roll numerical values | Divine Orb | Divine Orb |
| Strip mods (Item → Normal) | Orb of Scouring | (no direct equivalent — see Vaal/full re-craft) |
| Top-tier duplication | Mirror of Kalandra | Mirror of Kalandra |
| Quality on weapons | Whetstone | Whetstone-equivalent (PoE2 specifics in 0.x notes) |

---

## How the agent should reason about currency

1. **Identify the goal**: is this a crafting step or a trade step?
2. **Identify the right tool by role** — not by familiar name. Don't recommend "Chaos Orb" without checking which game's Chaos.
3. **Reason about substitutes** — is there a cheaper alternative? Does Settlers Currency Exchange handle this in bulk?
4. **State the lever, not the price** — "Use Divine to perfect roll values; this is roughly N divines mid-league" beats citing a stale price. For prices, fetch poe.ninja.
5. **Always cite which game** when describing currency behavior (PoE1 Chaos behaves differently from PoE2 Chaos).

## Cross-references

- `15_source_registry.md` — poe.ninja and trade-API for live prices.
- `09_itemisation_crafting.md` — broader crafting principles.
- `11_endgame_economy_trade_leagues.md` — economy mechanics across leagues.
- `13_retrieval_playbooks.md` — currency-pricing search recipes.
- `26_validation_and_self_correction.md` Rule 4 — PoE1↔PoE2 disambiguation.
- `poe2/00_version_pinning.md` — current PoE2 version anchor.
- `poe2/06_runes_of_aldur.md` — fills 2026-05-29 with 0.5 currency additions.
