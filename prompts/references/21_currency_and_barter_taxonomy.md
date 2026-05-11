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
| Exalted Orb | Rare → adds 1 mod (preserves existing). The exact mod-count gate and economic weight have shifted across patches — fetch the current rules from `wiki_parse https://www.poewiki.net/wiki/Exalted_Orb` and live pricing from poe.ninja. |
| Divine Orb | Re-roll *numerical values* within current mod ranges. Does not change which mods exist. Economic weight vs Chaos / Exalted shifts each league — check poe.ninja before quoting "the dominant currency". |
| Mirror of Kalandra | Duplicates an item once. Rarest currency tier; mirror-tier items are showcase pieces. |

**Key conceptual distinction**: Chaos *re-rolls which mods exist*; Divine *re-rolls the values within mod ranges that already exist*. Mixing these up is one of the most common LLM hallucinations on PoE.

### Quality currencies (improve item quality, separate from mods)

| Currency | Slot |
|---|---|
| Whetstone | Weapon quality (more physical damage). |
| Armourer's Scrap | Armour quality (more armour/evasion/ES). |
| Glassblower's Bauble | Flask quality. |
| Catalyst (Turbulent / Tempering / Imbued / Abrasive / Fertile / Prismatic / Intrinsic / Sinistral / Dextral / Tainted / etc.) | Jewellery quality with stat-tag specialisation (more X-tagged mods). The complete current list of catalyst names + their tag biases lives at `wiki_parse https://www.poewiki.net/wiki/Catalyst`. |
| Gemcutter's Prism | Skill gem quality. |

Quality matters because it scales the local property by quality % — a 20%-quality bow rolls higher base damage than a 0%-quality one.

### Socket / link / colour currencies

| Currency | Role |
|---|---|
| Jeweller's Orb | Re-roll *number* of sockets. |
| Orb of Fusing | Re-roll *links* between sockets. |
| Chromatic Orb | Re-roll *colours* of sockets. |
| Vaal Orb (corruption variant) | Can corrupt to 6L; risky. |
| Tainted Chromatic / Fusing / Jeweller's | Corrupted-only socket/link/colour orbs originating in Scourge-era tainted-currency mechanics (not Heist). Can break the item. Specific success/failure rules per orb: `wiki_parse https://www.poewiki.net/wiki/Tainted_Chromatic_Orb`. |

Most builds need a 6L body armour, often 5L weapon. Socket-coloring on off-stat bases (Sorcerer Boots with red sockets for str-str-int) is a classic crafting puzzle.

### Fragments and pinnacle keys

| Item | Function |
|---|---|
| Sacrifice Fragments (Dawn / Dusk / Midnight / Noon) | Atziri (Apex of Sacrifice) access. |
| Mortal Fragments (Hope / Grief / Rage / Ignorance) | Uber Atziri (Alluring Abyss). |
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

Scarabs are atlas modifiers applied to the map device, modifying a single map run. The scarab system has been reworked multiple times (notably the major 3.24-era restructure that retired the Rusted/Polished/Gilded/Winged tier ladder). **For the current scarab families, tier structure, and per-mechanic counts, fetch `wiki_parse https://www.poewiki.net/wiki/Scarab`** — citing a remembered tier name (Awakened / Greater / Polished etc.) is the audit's classic stale-claim pattern.

| Scarab family (examples) | Effect domain |
|---|---|
| Cartography Scarabs | Map drop modifiers. |
| Breach Scarabs | Breach spawn modifiers. |
| Legion Scarabs | Legion encounter modifiers. |
| Beyond Scarabs | Beyond mob spawning. |
| Anarchy Scarabs | Rogue exiles. |

Scarabs are a major lever in farming-strategy optimisation — pick scarabs that match your atlas tree allocation.

### Atlas-region modifiers (Sextants / Tablets / Astrolabes — patch-volatile)

The mechanic that applies a modifier to a region of the Atlas has been reworked multiple times — Sextants on watchstones (pre-Settlers), Tablets on towers (Settlers 3.25-era), and successor systems in subsequent leagues. **Always fetch `wiki_parse https://www.poewiki.net/wiki/Atlas_of_Worlds` or the current patch notes before describing which system is live** — the audit found that hardcoding "Tablets replaced Sextants in 3.25" mislabels the current state if a newer rework has shipped.

### Currency Exchange (bulk-trade infrastructure)

PoE1 has a 1:N bulk-trade system originally introduced in the Settlers Kingsmarch league (3.25) and since persisted via Faustus / asynchronous trade / Trade Market in subsequent patches. Send a worker (or place an order) with a stack of Currency A; they return with Currency B at a market rate. The most reliable bulk-conversion tool in modern PoE1. **Fetch current patch notes** to confirm which form the system takes in the active league.

---

## PoE2 — currency by role (current as of 0.4)

PoE2 deliberately narrows the currency tree. The conceptual deltas:

### Rolling currencies — tier system

PoE2 uses a Lesser → Greater → Perfect tier hierarchy on rolling orbs, systematised across the 0.2-0.3 patch cycle:

| PoE2 currency | Role |
|---|---|
| Orb of Transmutation | Normal → Magic. Mod count: `wiki_parse https://www.poe2wiki.net/wiki/Orb_of_Transmutation`. |
| Orb of Augmentation | Magic upgrade. |
| Regal Orb | Magic → Rare (adds 1 mod, keeping existing). |
| Greater Orb of Transmutation/Augmentation | Higher tier rolling orb (better mod-tier weighting). |
| Perfect Orb of Transmutation/Augmentation | Top tier (T1 mod weighting bias). |
| Orb of Alchemy | Normal → Rare. Mod count and rarity vs PoE1: `wiki_parse https://www.poe2wiki.net/wiki/Orb_of_Alchemy`. |
| Chaos Orb | Re-roll *one mod* on a Rare (NOT full re-roll). Different from PoE1 Chaos. |
| Exalted Orb | Add a mod to a Rare (similar to PoE1 Exalt; relative economic weight differs). |
| Divine Orb | Re-roll numerical values within mod ranges (same as PoE1). |
| Vaal Orb | Corrupt (same role as PoE1). |
| Mirror of Kalandra | Duplicate (same role as PoE1). |

**Rot trigger for AI**: PoE2 Chaos Orb is *not* a full re-roll. It re-rolls a single mod. Conflating PoE1 Chaos and PoE2 Chaos is a common error — see `26_validation_and_self_correction.md` Rule 4.

### Runes (socketable modifier items)

PoE2 has **Runes** — items socketed into rune sockets on bases that grant a passive effect. Conceptually a "currency you find" rather than "currency you craft with". The current rune family list, naming, and per-rune magnitudes are version-pinned and have shifted across patches — see `poe2/04_runes_soul_cores_talismans.md` and `wiki_parse https://www.poe2wiki.net/wiki/Rune`. **Do not recite specific rune names or magnitudes from memory.**

Runes are not crafting currencies — they're modifiers applied to bases via specific sockets the base provides.

### Soul Cores

Higher-tier rune-equivalents primarily acquired from trial / boss / sanctum tiers in PoE2. See `poe2/04_runes_soul_cores_talismans.md` for the categorical detail and `wiki_parse https://www.poe2wiki.net/wiki/Soul_core` for current acquisition / behaviour.

### Talismans

In PoE2 Talismans are a **two-handed melee weapon class** used by shapeshift archetypes — NOT amulet-slot charge items. The previous "neck-slot charge accumulation" description in this file was wrong. See `poe2/04_runes_soul_cores_talismans.md` and `wiki_parse https://www.poe2wiki.net/wiki/Talisman`.

### Gold (PoE2 account-bound currency)

PoE2 has **gold** as an account-bound currency used at vendors, currency exchange, passive respec, and asynchronous trade. The exact list of use cases shifts with patches — `wiki_parse https://www.poe2wiki.net/wiki/Gold`. Conceptually closer to a typical RPG gold than the orb economy.

### 0.5 additions — PLACEHOLDER

⚠️ **DO NOT QUOTE as facts.** Pre-launch datamining + GGG announcement materials. Authoritative content goes in `poe2/06_runes_of_aldur.md` once the 0.5 patch notes ship.

Expected entities (subject to GGG confirmation): a league mechanic named **Runes of Aldur**, a currency named **Verisium**, a defensive layer named **Runic Ward**, a crafting modifier referred to as **Alloys**. Mechanics are not verified — refuse to give specifics until the placeholder is filled.

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
