---
description: Why specific item base types matter — implicit pool, ilvl-gated mod tiers, attribute requirements, mod-weight differences. Categorical / role-based identity only; magnitudes and specific implicit values are fetched live.
fetch_when: User asks "which base should I craft?", "why this base over that one?", or you're recommending a craft target / vendor recipe / SSF base-find priority. Always pair with `repoe_lookup category=base_items` for current numerical implicits.
---

# 20 — Item basetype identity

> **No specific implicit values, attack speeds, or magnitudes are stored in this file by design.** The 2026-05-12 audit found that 60%+ of the numbers previously cached here were stale or wrong (Spine Bow APS, Bone Helmet minion damage, Two-Toned Boots resist range, Onyx all-attribute implicit, Imbued Wand implicit type). For any quantitative claim, fetch `repoe_lookup category=base_items name=<base>` or `wiki_parse https://www.poewiki.net/wiki/<Base>` before quoting.

This doc is about **why a base is the right base**, not the raw numbers. The map below is for **reasoning about base selection**.

The single most useful question this doc answers: *"Why this base over a similar one?"*

## What makes a base "the right base"

A base type isn't picked for one reason — it's the intersection of:

1. **Implicit modifier** — every base has 0-2 implicit mods (visible at top of item). Some implicits are *defining* (the base wouldn't be picked without that specific implicit).
2. **Attribute requirements** — strength / dex / int gating. A "Sorcerer Boots" needs int; an "Eternal Burgonet" needs strength. Builds without that attribute pool can't equip the base, full stop.
3. **Defensive type** — armour / evasion / ES / armour-evasion / armour-ES / evasion-ES / pure ES / pure armour / pure evasion. Determines which defensive scaling applies.
4. **ilvl-gated mod tiers** — high-tier mods require base ilvl. Endgame craft targets sit at the top of this gate — fetch the current ilvl floor from PoEDB / wiki.
5. **Mod weight asymmetries** — some bases have higher mod weight on certain affix types. e.g., a wand is biased toward spell mods; a rune dagger toward attack speed.
6. **Slot-specific quirks** — gloves on certain bases support Eldritch implicits, others don't. Rings have base-specific ilvl gates.
7. **Build-defining synergy** — a base might be "the" base for an archetype (Spine Bow for evasion-attack builds, Convoking Wand for caster crafts).

A "best base" recommendation must explain at least one of these reasons.

---

## PoE1 — base type identity

For each base below, the description is **role-only**: which build archetype it serves, which defensive type it has, what kind of implicit it carries. For the actual magnitude of any implicit or base stat (APS, base damage, ES floor, attribute requirement value), **fetch via `repoe_lookup` or `wiki_parse`**.

### Bows

| Base | Role |
|---|---|
| Spine Bow | High-attack-speed evasion bow — the default for evasion attack-bow builds (Lightning Arrow Deadeye etc.). APS and crit chance: `repoe_lookup` or `wiki_parse https://www.poewiki.net/wiki/Spine_Bow`. |
| Citadel Bow | Strength-stacking bow. Implicit pool: `wiki_parse https://www.poewiki.net/wiki/Citadel_Bow`. |
| Maraketh Bow (Maraketh-tagged bases) | Maraketh-league lineage. Implicit specifics (including any +socketed-gem-level) vary by patch — `wiki_parse`. |
| Crude Bow | Earliest leveling bow; cheap. |
| Imperial Bow | High-damage, slow-APS bow. Base for Lioneye's Glare. |
| Short Bow / Composite Bow | Leveling tier; cheap re-rolls. |

**Build-determining principle**: bow choice trades raw damage (Imperial Bow) against attack speed (Spine Bow). Modern endgame bow builds typically favour Spine Bow for the speed; verify the current APS / damage spread before committing.

### Helmets

| Base | Role |
|---|---|
| Eternal Burgonet | Strength-armour helmet base; common Eldritch helm target. |
| Lion Pelt | Dex-evasion helmet — default for evasion-side. Eldritch helm capable. |
| Hubris Circlet | Int-ES — the endgame ES helmet for caster builds. |
| Solaris Circlet | Lower-tier ES; ilvl-gated. |
| Vaal Mask | Corrupted-only implicit support — niche vs Hubris Circlet (clean ES). |
| Bone Helmet | Minion-specific implicit (minion-damage modifier). Necromancer / Animated Guardian default. Current implicit magnitude: `wiki_parse https://www.poewiki.net/wiki/Bone_Helmet` (was nerfed in 3.17 — reciting an old value is the audit's #1 hallucination case). |

**Build-determining principle**: pick the base by attribute requirement and defensive type. Eldritch implicits (introduced 3.17, Siege of the Atlas) make most endgame helmets either Lion Pelt / Eternal Burgonet / Hubris Circlet.

### Body armours

| Base | Role |
|---|---|
| Astral Plate | Pure armour with an all-elemental-resistance implicit — common pure-armour endgame craft. |
| Vaal Regalia | Pure ES — the endgame ES chest. |
| Saintly Garb | Pure ES (lower tier). |
| Carnal Armour | Hybrid ES-armour-evasion. Niche. |
| Cloak of Flame | Unique chest historically used in RF-Chieftain archetypes. Whether it's the current meta pick varies per league — `wiki_parse https://www.poewiki.net/wiki/Cloak_of_Flame` for current mods. |

The body-armour base is often less critical than the helmet / gloves / boots — most builds put 6L mods + 1 unique-base requirement, then pick whichever fits.

### Gloves

| Base | Role |
|---|---|
| Spiked Gloves | Attack-side gloves — a classic craft target for attack speed + life + resists. Defensive type: `wiki_parse`. |
| Slink Gloves | Pure evasion — default for evasion-side. |
| Sorcerer Gloves | Pure ES — caster default. |
| Fingerless Silk Gloves | Spell-damage implicit — default base for spell-builds layering Eldritch implicits. |

Gloves are Eldritch-implicit-capable since 3.17 and are typically *crafted around* the Eldritch implicit.

### Boots

| Base | Role |
|---|---|
| Sorcerer Boots | Pure ES — default for ES casters. |
| Two-Toned Boots | Hybrid (armour-evasion / armour-ES / evasion-ES subtypes) with a dual-elemental-resistance implicit — the default for resist-flexible builds. Specific implicit range and subtype-by-subtype defensive stats: `wiki_parse https://www.poewiki.net/wiki/Two-Toned_Boots`. |
| Stealth Boots | Evasion + ES. Niche; specific archetypes. |
| Slink Boots | Pure evasion — default for evasion-only chassis. |

**Build-determining principle**: Two-Toned Boots' dual-resist implicit is strong enough that they're "the" base for builds without specific implicit requirements. Sorcerer Boots dominate ES.

### Wands and rune daggers

| Base | Role |
|---|---|
| Convoking Wand | Minion-wand with minion-damage implicit — fetch the current implicit and mod pool via `wiki_parse https://www.poewiki.net/wiki/Convoking_Wand`. Often a 6-link replacement for caster builds when body 6L is too expensive. |
| Imbued Wand | Spell-damage-implicit wand (the audit corrected a previous "spell crit chance" claim — verify the current implicit type via `wiki_parse https://www.poewiki.net/wiki/Imbued_Wand`). |
| Crystal Wand | Hybrid dex/int. |
| Scorpion / Sai (rune dagger) | Spell-crit / attack-speed rune dagger. Niche dagger casters. |
| Karui Maul (mace) | Niche melee strength stack. |

### Amulets

| Base | Role |
|---|---|
| Onyx Amulet | All-attribute implicit (the audit corrected a previous "Strength + Intelligence" claim — Onyx grants all three attributes, fetch `wiki_parse https://www.poewiki.net/wiki/Onyx_Amulet` for current range). The universal-default amulet for PoE1 attribute-flexible builds. |
| Stellar Amulet | **PoE2 base — NOT PoE1.** A previous version of this doc misattributed it to PoE1; Stellar Amulet is the all-attribute amulet in PoE2 only. PoE1 uses Onyx for the same role. |
| Lapis Amulet | Intelligence + evasion-side hybrid. |
| Marble Amulet | Strength + life-regeneration implicit. Used in RF / regen builds. Current life-regen implicit range: `wiki_parse https://www.poewiki.net/wiki/Marble_Amulet`. |
| Citrine Amulet | Strength + dexterity. |
| Jade Amulet / Coral Amulet / Lapis Amulet (single-attribute) | Single-attribute implicits — niche when a single attribute is over-prioritized. |

**Build-determining principle**: Onyx is the universal PoE1 default; pick a more specific base only if a single implicit attribute matters more than flexibility.

### Belts

| Base | Role |
|---|---|
| Stygian Vise | Abyssal-jewel-socket belt — default endgame belt for builds that benefit from Abyssal Jewel mods. |
| Heavy Belt | Strength stack. Niche. |
| Chain Belt | Stun-recovery implicit. |
| Leather Belt | Life implicit. Niche but cheap craft target for league-start. |

### Rings

| Base | Role |
|---|---|
| Two-Stone (Topaz/Sapphire/Ruby) | Hybrid resist implicit (3 sub-variants pairing the elemental resistances). Default for resist-flexible builds. |
| Diamond Ring | Crit-chance-related implicit. Caster crit. |
| Vermillion Ring | Life implicit. Niche health-stack rings. |
| Coral Ring | Life implicit (lower tier). |
| Topaz Ring / Sapphire Ring / Ruby Ring | Single-resist implicit. Cheap builds. |
| Iron Ring | Adds physical damage to attacks (implicit). Niche attack-build. |
| Unset Ring | Skill gem socket — powerful for trigger setups. |

### Quivers

| Base | Role |
|---|---|
| Heavy Quiver | High-implicit physical-damage quiver. |
| Spike-Point Arrow Quiver | Crit-multiplier implicit. Niche bow crit. |
| Penetrating Arrow Quiver | Life-leech-on-hit. Niche life-sustain. |
| Broadhead Arrow Quiver | Bleed chance + duration. Bleed bows. |

### Shields

| Base | Role |
|---|---|
| Champion Kite Shield | Pure armour. Common defensive shield. |
| Ezomyte Spiked Shield | Pure armour with attack-side use. |
| Pinnacle Tower Shield | Highest base armour. Niche pure-armour build. |
| Titanium Spirit Shield | Pure ES — caster default. |
| Mosaic Kite Shield | Hybrid armour/evasion. |

---

## PoE2 — base type identity

> **PoE2 base names, tier structure, and implicit magnitudes are version-pinned and likely to shift at the 0.5 launch on 2026-05-29.** Always verify against `repoe_lookup category=base_items` or `poe2db.tw/us/Items` before recommending a specific base.

PoE2 is more **base-implicit-driven**. The Expert base tier is dramatically more powerful than Advanced and Basic, and each base often has a defining build-relevant implicit.

### Expert bases

PoE2 introduces an "Expert" tier of bases — late-game, ilvl-gated, with significantly higher implicit ranges than Advanced. The single biggest driver of late-game gear quality.

| Tier | Implicit range |
|---|---|
| Basic | Lowest. |
| Advanced | Mid. |
| Expert | Highest; endgame-only. |

A craft target should always be on an Expert base for late-game characters. The exact ilvl floor at which Expert bases start appearing: `wiki_parse https://www.poe2wiki.net/wiki/Item` or `poe2db.tw/us/Items`.

### Highlights (categorical only — verify current names)

| Base family | Role |
|---|---|
| Expert Robe / equivalent | Highest ES base for casters. |
| Expert Plate Vest / equivalent | Highest armour base. |
| Expert Buckler / Tower Shield | Block / armour focus. |
| Expert War Quiver / Hunter's Quiver | Attack-skill quivers with varying implicits. |
| Expert Sceptre / Wand | Caster spell-damage weapons (Sceptres also grant Spirit). |
| Expert Crossbow | Mercenary-class core base. |
| Expert Spear / Quarterstaff | Huntress / Monk weapon archetype defaults. |
| Talisman (two-handed) | Druid / Spirit Walker / Martial Artist shapeshift weapon class — NOT an amulet. |

PoE2 base names are evolving across patches. **For any specific base recommendation, fetch the current name + implicit range via `repoe_lookup category=base_items` or `wiki_parse` against the live wiki page** — reciting a remembered name (especially from pre-0.4 datamines) is unreliable.

### Runes-as-base-feature

PoE2 bases distinguish on **how many rune sockets** they support. A base with more rune sockets effectively grants more free-ish modifiers via socketed runes; higher-socket bases dramatically increase a base's worth. Exact socket counts per base are version-pinned.

### Base implicits as identity

In PoE2, base implicits often *define* the base's role:

- A weapon with an attack-speed implicit becomes an attack-speed weapon.
- A ring with an all-resistance implicit becomes the universal resist ring.

Treat the implicit as the primary identifier — base name secondary, magnitude fetched live.

---

## How the agent should reason about base selection

1. **What's the build's attribute split?** — pick a base that matches the build's primary attribute.
2. **What's the build's defensive type?** — match armour / evasion / ES / hybrid base.
3. **What ilvl?** — endgame craft target sits at the ilvl floor for T1 mods; verify the current floor on `poedb.tw/us/Modifiers` or `wiki_parse`.
4. **What's the implicit doing?** — is it adding meaningful build power (Onyx all-attribute, Two-Toned dual-resist), or just baseline?
5. **What Eldritch / craft-bench / influenced-mod / Hateforge etc.?** — does this base support the modifiers the build needs?
6. **What's the alternative?** — there's almost always a tier-1 and a tier-2 viable base. Mention both.

The mistake to avoid: recommending a "best base" without stating *which build*. Bases are build-relative, not absolute.

## Cross-references

- `09_itemisation_crafting.md` — broader crafting principles, mod system, ilvl mechanics.
- `21_currency_and_barter_taxonomy.md` — orbs used to craft on these bases.
- `13_retrieval_playbooks.md` — base / mod search recipes.
- `26_validation_and_self_correction.md` Rule 4 — PoE1↔PoE2 disambiguation when discussing bases.
- `repoe_lookup` `category=base_items` — current implicit values + ilvl gates.
- `poe2/00_version_pinning.md` — current PoE2 version anchor for Expert-tier specifics.
