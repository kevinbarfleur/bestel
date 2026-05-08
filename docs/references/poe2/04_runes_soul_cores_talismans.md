---
description: PoE2 itemization extras — runes (socketed-gem-like modifiers on bases), soul cores (rune-tier upgrades from boss/sanctum sources), talismans (neck slot variant). Plus 0.5's Verisium / Runic Ward / Alloys placeholder.
fetch_when: User asks about runes, soul cores, talismans, or 0.5 league mechanics. Always apply to PoE2 only.
---

# PoE2 — Runes, Soul Cores, Talismans

> Always check `00_version_pinning.md` for current PoE2 version. The 0.5 (2026-05-29) launch will introduce new content here — fill `06_runes_of_aldur.md` on launch.

This doc covers PoE2's "extra modifier" itemization layer — features that don't directly map to PoE1.

## Runes

Runes are **rune-stones socketed into equipment sockets** on PoE2 bases. They grant a flat or scaling modifier to the equipment.

### Mechanic

- Drop / craft / vendor / quest as item types.
- Socketed into a base's rune sockets (varies by base — see `20_item_basetype_identity.md`).
- Once socketed: cannot be removed without consuming the rune (early patches) or with a specific currency (later patches — verify).
- **A rune is the modifier**, not the gem. The base "wears" the rune; the rune isn't a separate skill.

### Common rune families

| Rune family | Effect domain |
|---|---|
| Iron Rune | Flat physical damage on weapons. |
| Body Rune | Flat life on armours. |
| Ember Rune | Flat fire damage. |
| Glacial Rune | Flat cold damage. |
| Storm Rune | Flat lightning damage. |
| Blood Rune | Life leech / regen. |
| Stone Rune | Armour. |
| Vine Rune | Evasion. |
| Soul Rune | Mana / spirit related. |

Names and effects evolve per patch; verify against current wiki / `repoe_lookup` (Sprint 1+).

### When to use what rune

- **Slot type matters**: weapon sockets favour damage runes; armour sockets favour defensive runes.
- **ilvl / rarity gates**: higher-ilvl runes have better numerical floors.
- **Build alignment**: an ignite-build wants Ember Runes; a phys-attack build wants Iron Runes.

### Pitfalls

- Treating runes as "skill gems" — they're not, they're stat modifiers.
- Forgetting socket coloring isn't a thing — runes don't color, they fit any rune socket.
- Confusing rune slots with regular gem sockets — different concept.

## Soul Cores [0.2+]

Soul Cores are **higher-tier rune-equivalents** — significantly stronger modifiers than baseline runes, bound to specific monster sources.

### Mechanic

- Drop / quest from boss-tier sources (Sanctum relics, atlas mechanic drops, league mechanics).
- Socketed similarly to runes (occupy rune sockets).
- Dramatically higher per-stat budget than baseline runes.
- Often build-defining for high-end gear — a soul-core-enabled gear slot is meaningfully more powerful.

### Acquisition

- **Boss drops**: pinnacle / atlas-mechanic boss kills.
- **Sanctum relics**: high-tier Sanctum relics drop soul cores.
- **League mechanic**: 0.4 had a specific league-mechanic soul-core source — verify per current league.

### Pitfalls

- Treating soul cores as commodity items — they're build-defining.
- Confusing soul cores with the regular rune family.
- Forgetting their source-bound nature; SSF players have to find them themselves.

## Talismans [0.1+]

Talismans are an **amulet-slot variant** with charge / curse interactions — distinct mod pool from regular amulets.

### Mechanic

- Equip in the amulet slot (talismans replace regular amulets in this slot).
- Talismans have a **charge** mechanic: gain charges over combat / per condition, spend charges for buffs / curse-immunity / damage-modifiers.
- Cannot be modified by certain craft mods that affect regular amulets.
- Have unique mod pools — some mods only roll on talismans.

### Common talisman archetypes

- **Curse-immunity charge**: spend charge to ignore next curse.
- **Buff-charge**: spend charge for "Berserk-like" temporary buff.
- **Defensive charge**: spend charge for damage reduction window.
- **Build-defining-unique talismans**: rare and specific.

### Pitfalls

- Treating a talisman as a regular amulet — different mod pool, different scaling.
- Forgetting talisman charge mechanic — many talisman effects are passive only when charges available.

## 0.5 additions [2026-05-29]

⚠️ Pre-launch from GGG announcement materials — fill `06_runes_of_aldur.md` with authoritative content on launch day.

### Verisium

- New currency type, bound to "Runes of Aldur" league mechanic.
- Likely augments runes or creates rune variants.
- Datamining-based pre-launch.

### Runic Ward

- Defence-flavoured rune-derived buff.
- Likely a passive defensive layer triggered by runes.

### Alloys

- Crafting modifier — datamining suggests it's a rune/currency hybrid.
- Likely modifies how runes apply to bases.

### When 0.5 ships

1. Update `00_version_pinning.md`.
2. Fill `06_runes_of_aldur.md` with authoritative mechanics.
3. Re-section this doc — distinguish "0.4 runes" from "0.5 Verisium-augmented runes".

## How the agent should reason about rune / soul-core questions

1. **Identify whether the user is asking about runes, soul cores, or talismans** — they're three distinct mechanics.
2. **Match the build's needs** — a phys build wants Iron Runes; a caster wants spell-tagged runes.
3. **Consider socket count on the base** — higher socket count = more rune slots = more total modifier value.
4. **Differentiate between PoE2 rune (modifier) and PoE1 catalyst (jewellery quality)** — different concepts despite naming overlap.
5. **For soul cores: ask the build's tier**. Endgame builds prioritise soul cores; mid-game builds use baseline runes.

## Pitfalls (common LLM hallucinations)

1. **Calling runes "skill gems"** — they're not. Runes are stat modifiers; gems are skills.
2. **Treating runes like PoE1 Catalysts** — Catalysts grant quality with stat-tag bias on jewellery; runes grant a stat directly. Different mechanic.
3. **Treating talismans like PoE1 Talismans (legacy league)** — PoE1 had a Talisman league mechanic in 2.X. PoE2 talismans are a different concept.
4. **Telling user to "remove a rune" without checking the version** — early-patch runes were destroyed on removal; later patches may have a removal currency.

## Cross-references

- `09_itemisation_crafting.md` — broader itemisation principles.
- `20_item_basetype_identity.md` — bases that support rune sockets.
- `21_currency_and_barter_taxonomy.md` — currency-side overview.
- `26_validation_and_self_correction.md` Rule 4 — PoE1 vs PoE2 itemisation disambiguation.
- `00_version_pinning.md` — version anchor.
- `06_runes_of_aldur.md` — fills 2026-05-29 with 0.5 mechanics.
