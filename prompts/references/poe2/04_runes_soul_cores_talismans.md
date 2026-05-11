---
description: PoE2 itemization extras — runes (socketed-modifier items in rune sockets), soul cores (rune-tier upgrades from boss / trial sources), talismans (shapeshift weapon class with charm-charge mechanic). Plus 0.5's Verisium / Runic Ward / Alloys placeholder.
fetch_when: User asks about runes, soul cores, talismans, or 0.5 league mechanics. Always apply to PoE2 only.
---

# PoE2 — Runes, Soul Cores, Talismans

> Always check `00_version_pinning.md` for current PoE2 version. The 0.5 launch will introduce new content here — fill `06_runes_of_aldur.md` on launch.

This doc covers PoE2's "extra modifier" itemization layer — features that don't directly map to PoE1. **No numeric values are stored here by design.** For specific magnitudes, fetch `wiki_parse` or `repoe_lookup` against the named entity.

## Runes

Runes are **rune-stones socketed into rune sockets on PoE2 gear**. They grant a flat or scaling modifier to the equipment.

### Mechanic

- Drop / craft / vendor / quest as item types.
- Socketed into a base's rune sockets (varies by base — see `20_item_basetype_identity.md`).
- Removability rules vary by patch (early PoE2 patches destroyed runes on removal; later patches may have a removal currency). **Fetch `wiki_parse https://www.poe2wiki.net/wiki/Rune` for the current rule.**
- **A rune is the modifier**, not a skill gem. The base "wears" the rune.

### Rune family taxonomy

Runes are organised into families by **the kind of effect they apply** — damage-typed families, defensive families, hybrid families. The set of families and the specific magnitudes per rune are **patch-volatile** (the audit on 2026-05-12 found the bundled list of nine family names was already stale vs the live wiki).

**For the current family list, the magnitudes, and the rune-name spellings, always fetch `wiki_parse https://www.poe2wiki.net/wiki/Rune` or `repoe_lookup category=mods name=<rune-name>` before quoting.**

### When to use what rune

- **Slot type matters**: weapon sockets favour damage-typed runes; armour sockets favour defensive runes.
- **ilvl / rarity gates**: higher-ilvl runes have better numerical floors — exact gates live in `repoe_lookup`.
- **Build alignment**: an ignite-build wants fire-typed runes; a phys-attack build wants physical-typed runes; an evasion-stacker wants evasion-typed runes. The right specific rune-name per family is wiki-current.

### Pitfalls

- Treating runes as skill gems — they're not, they're stat modifiers.
- Forgetting socket coloring isn't a thing on rune sockets — runes don't color, they fit any rune socket.
- Confusing rune slots with regular gem sockets — different concept.

## Soul Cores

Soul Cores are **higher-tier rune-equivalents** — significantly stronger modifiers than baseline runes, bound to specific monster / trial sources.

### Mechanic

- Drop primarily from trial / boss / sanctum tiers in PoE2.
- Socketed similarly to runes (occupy rune sockets).
- Per-stat budget is meaningfully higher than baseline runes — frequently build-defining at endgame.
- The patch in which Soul Cores were introduced and their current acquisition rules live on the wiki — **fetch `wiki_parse https://www.poe2wiki.net/wiki/Soul_core` before quoting how / when / where they drop**.

### Pitfalls

- Treating soul cores as commodity items — they're build-defining at endgame.
- Confusing soul cores with the regular rune family.
- Forgetting their source-bound nature; SSF players have to find them themselves.

## Talismans

Talismans in PoE2 are **a two-handed weapon class** used by shapeshift-archetype builds (Druid, plus 0.5 Spirit Walker/Martial Artist additions). They are NOT amulet-slot items. The PoE1 Talisman league mechanic is a completely separate, unrelated system that shares only the name.

### Mechanic (categorical — no magnitudes here)

- Equipped as a two-handed martial weapon (taking both weapon slots).
- Require Strength and Intelligence to wield.
- Allow Shapeshifting skills associated with an animal form (Bear / Werewolf / Wyvern, etc. — set per talisman).
- Grant a default attack appropriate to the chosen form.
- Persistent and trigger skills continue to function while shapeshifted.

For the current list of talismans, the form options each one supports, the granted-attack patterns, and the unique-talisman roster, **always fetch `wiki_parse https://www.poe2wiki.net/wiki/Talisman` or `repoe_lookup category=base_items name=<talisman>`**.

### Pitfalls

- **Treating PoE2 Talismans like PoE1 Talismans (Talisman league, 2.X)** — completely different system. PoE1 Talismans were corruption-tier amulets with bestiary heritage; PoE2 Talismans are shapeshift weapons. Naming overlap only.
- **Treating a Talisman as an amulet** — wrong slot, wrong category, wrong stat pool.
- **Forgetting the shapeshift requirement** — Talisman-equipped builds spend most of their time in animal form, which restricts skill availability.

## 0.5 additions — PLACEHOLDER

⚠️ **DO NOT QUOTE these as facts.** This block is pre-launch datamining + GGG announcement materials. The official patch notes drop with the 0.5 launch and `06_runes_of_aldur.md` will be filled with authoritative content. Until that file is filled, refuse to give specifics:

> *"The chronicles of the next age are not yet written, exile — the 0.5 league mechanic is still pre-launch. Wait for the official patch notes."*

The user-facing entities expected in 0.5 (subject to GGG confirmation) include a league mechanic named **Runes of Aldur**, a currency named **Verisium**, a defensive layer named **Runic Ward**, and a crafting modifier referred to as **Alloys**. None of these has a verified mechanic in this file — fetch the official forum thread once `06_runes_of_aldur.md` is updated.

### When 0.5 ships

1. Update `00_version_pinning.md`.
2. Fill `06_runes_of_aldur.md` with authoritative mechanics fetched from `pathofexile.com/forum/...` patch-notes thread.
3. Re-section this doc — distinguish "pre-0.5 runes" from "0.5+ runes" where the mechanic diverges.

## How the agent should reason about rune / soul-core / talisman questions

1. **Identify whether the user is asking about runes, soul cores, or talismans** — three distinct mechanics with overlapping vocabulary.
2. **Match the build's needs** — fetch the current rune family aligned with the build's damage type or defensive layer; never recite a remembered "Iron Rune = X% phys" magnitude.
3. **Consider rune socket count on the base** — more sockets = more modifier value. The base-specific socket counts live in `repoe_lookup category=base_items`.
4. **Differentiate between PoE2 rune (modifier) and PoE1 catalyst (jewellery quality)** — different concepts despite naming overlap.
5. **For soul cores: ask the build's tier**. Endgame builds prioritise soul cores; mid-game builds use baseline runes. Always fetch the current source list before recommending where to farm them.
6. **For Talismans: confirm the user is on PoE2 and is talking about the weapon class, not the PoE1 league item**. If unclear, ask.

## Pitfalls (common LLM hallucinations)

1. **Calling runes "skill gems"** — they're not. Runes are stat modifiers; gems are skills.
2. **Treating runes like PoE1 Catalysts** — Catalysts grant quality with stat-tag bias on jewellery; runes grant a stat directly. Different mechanic.
3. **Treating PoE2 Talismans like PoE1 Talismans** — already noted above; load-bearing distinction.
4. **Quoting a specific rune-family magnitude from memory** — the audit found these are already stale. Always fetch.

## Cross-references

- `09_itemisation_crafting.md` — broader itemisation principles.
- `20_item_basetype_identity.md` — bases that support rune sockets.
- `21_currency_and_barter_taxonomy.md` — currency-side overview.
- `26_validation_and_self_correction.md` Rule 4 — PoE1 vs PoE2 itemisation disambiguation.
- `00_version_pinning.md` — version anchor.
- `06_runes_of_aldur.md` — fills on 0.5 launch with the authoritative new mechanics.
