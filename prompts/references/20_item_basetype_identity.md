---
description: Why specific item base types matter — implicit pool, ilvl-gated mod tiers, attribute requirements, mod-weight differences. PoE1 highlights (Spine Bow, Eternal Burgonet, Sorcerer Boots, Convoking Wand, Stellar Amulet). PoE2 Expert bases + base-implicit identity.
fetch_when: User asks "which base should I craft?", "why this base over that one?", or you're recommending a craft target / vendor recipe / SSF base-find priority. Always pair with `repoe_lookup category=base_items` (Sprint 1) for current numerical implicits.
---

# 20 — Item basetype identity

This doc is about **why a base is the right base**, not the raw numbers. For implicit values, mod-tier ranges, and ilvl gates, fetch `repoe_lookup` (Sprint 1+) or wiki. The map below is for **reasoning about base selection**.

The single most useful question this doc answers: *"Why this base over a similar one?"*

## What makes a base "the right base"

A base type isn't picked for one reason — it's the intersection of:

1. **Implicit modifier** — every base has 0-2 implicit mods (visible at top of item). Some implicits are *defining* (Stellar Amulet's hybrid attributes; Two-Toned Boots' resistances).
2. **Attribute requirements** — strength / dex / int gating. A "Sorcerer Boots" needs int; an "Eternal Burgonet" needs strength. Builds without that attribute pool can't equip the base, full stop.
3. **Defensive type** — armour / evasion / ES / armour-evasion / armour-ES / evasion-ES / pure ES / pure armour / pure evasion. Determines which defensive scaling applies.
4. **ilvl-gated mod tiers** — high-tier mods require base ilvl. ilvl 86 is the baseline for endgame crafting (Eldritch implicits, T1 mods).
5. **Mod weight asymmetries** — some bases have higher mod weight on certain affix types. e.g., a wand is biased toward spell mods; a rune dagger toward attack speed.
6. **Slot-specific quirks** — gloves on certain bases support Eldritch implicits, others don't. Rings have base-specific ilvl gates.
7. **Build-defining synergy** — a base might be "the" base for an archetype (Spine Bow for evasion-attack builds, Convoking Wand for caster crafts).

A "best base" recommendation must explain at least one of these reasons.

---

## PoE1 — base type identity

### Bows

| Base | Why it matters |
|---|---|
| Spine Bow | High base attack speed (1.50), evasion-based. The default for evasion attack-bow builds (Lightning Arrow Deadeye etc.). |
| Citadel Bow | Strength-stack focus. Base has implicit "+X to Strength". Niche — pure-str-stack bows. |
| Maraketh Bow (Maraketh-tagged bases) | Built for league-launch crafting (Maraketh league legacy). Some have +1-2 socketed gem implicits. |
| Crude Bow | Earliest leveling bow; cheap. |
| Imperial Bow | Higher base damage, slower attack speed. Niche; Lioneye's Glare unique base. |
| Short Bow / Composite Bow | Leveling tier; cheap re-rolls. |

**Build-determining principle**: bow choice scales with whether you want raw damage (Imperial Bow) or attack speed (Spine Bow). Most modern endgame bow builds favour Spine Bow for the speed.

### Helmets

| Base | Why it matters |
|---|---|
| Eternal Burgonet | Strength-armour. Highest evasion-evasion-ES craft for str-priority slots; common Eldritch helm. |
| Lion Pelt | Dex-evasion. The default for evasion-side helmets. Eldritch helm capable. |
| Hubris Circlet | Int-ES. The endgame ES helmet for caster builds. |
| Solaris Circlet | Lower-tier ES. Niche; ilvl-gated. |
| Hubris Circlet vs Vaal Mask | Hubris is the "clean" ES base; Vaal Mask has corrupted-only implicit support. |
| Bone Helmet | Minion-specific implicit "+30% Minion Damage". Necromancer / Animated Guardian. |

**Build-determining principle**: pick the base by attribute requirement and defensive type. Eldritch implicits (3.17+) make most endgame helmets either Lion Pelt / Eternal Burgonet / Hubris Circlet.

### Body armours

| Base | Why it matters |
|---|---|
| Astral Plate | Pure armour, all-resistance implicit. Common pure-armour endgame craft. |
| Vaal Regalia | Pure ES. The endgame ES chest. |
| Saintly Garb | Pure ES (lower tier). |
| Carnal Armour | Hybrid ES-armour-evasion. Niche. |
| Cloak of Flame (unique-only base concept) | Specific build pressure (RF Chieftain Pohx archetype). |

The body-armour base is often less critical than the helmet/gloves/boots — most builds put 6L mods + 1 unique-base requirement, then pick whichever fits.

### Gloves

| Base | Why it matters |
|---|---|
| Spiked Gloves | Pure armour, attack-side gloves. Crafting "Spiked Gloves with attack speed + life + resists" is a classic. |
| Slink Gloves | Pure evasion. Default for evasion-side. |
| Sorcerer Gloves | Pure ES. Caster default. |
| Fingerless Silk Gloves | Spell damage implicit. The default base for spell-builds wanting Eldritch + spell-damage implicit. |

Gloves are Eldritch-implicit-capable since 3.17 and are typically *crafted around* the Eldritch implicit (e.g., Exarch "Trigger socketed spell on attack").

### Boots

| Base | Why it matters |
|---|---|
| Sorcerer Boots | Pure ES. Default for ES casters. |
| Two-Toned Boots | Hybrid (armour-evasion / armour-ES / evasion-ES depending on sub-type) with **+X% to two random elemental resistances** as implicit. The default for resist-flexible builds. |
| Stealth Boots | Evasion + ES. Niche; specific archetypes. |
| Slink Boots | Pure evasion. Default for evasion-only chassis. |
| Bone Trinkets / Skinning Knife (legacy) | Niche. |

**Build-determining principle**: Two-Toned Boots' implicit is so strong that they're "the" base for builds without specific implicit requirements. Sorcerer Boots dominate ES.

### Wands and rune daggers

| Base | Why it matters |
|---|---|
| Convoking Wand | Caster default. High base spell damage; common 6-link replacement when body 6L too expensive. |
| Imbued Wand | Higher spell crit chance implicit. |
| Crystal Wand | Hybrid dex/int. |
| Scorpion / Sai (rune dagger) | High spell crit chance + attack speed. Niche dagger casters. |
| Karui Maul (mace) | Niche melee strength stack. |

### Amulets

| Base | Why it matters |
|---|---|
| Stellar Amulet | All-attribute (str/dex/int balance). Most flexible base for any build. |
| Onyx Amulet | Strength + intelligence implicit. Niche when pure str-int build. |
| Lapis Amulet | Intelligence + evasion-side build. |
| Marble Amulet | Strength + life regeneration. Niche RF / regen builds. |
| Citrine Amulet | Strength + dexterity. |

**Build-determining principle**: Stellar Amulet is the universal default; pick a more specific base only if the implicit attribute matters more than flexibility.

### Belts

| Base | Why it matters |
|---|---|
| Stygian Vise | One abyssal jewel socket. Default endgame belt — the abyssal jewel mod pool is too valuable to skip. |
| Heavy Belt | Strength stack. Niche. |
| Chain Belt | Stun recovery implicit. |
| Leather Belt | Life implicit. Niche but cheap craft target for league-start. |

### Rings

| Base | Why it matters |
|---|---|
| Two-Stone (Topaz/Sapphire/Ruby) | Hybrid resist implicit. Two-Stone Ring (Lightning + Cold) is common. |
| Diamond Ring | Crit chance implicit. Caster crit. |
| Vermillion Ring | Life implicit. Niche health-stack rings. |
| Coral Ring | Life implicit (lower tier). |
| Topaz Ring / Sapphire Ring / Ruby Ring | Single-resist implicit. Cheap builds. |
| Iron Ring | "Adds X-Y physical damage to attacks" implicit. Niche attack-build. |
| Unset Ring | Skill gem socket. Niche but powerful for trigger setups. |

### Quivers

| Base | Why it matters |
|---|---|
| Heavy Quiver | High implicit physical damage. |
| Spike-Point Arrow Quiver | Crit-multiplier implicit. Niche bow crit. |
| Penetrating Arrow Quiver | Life-leech-on-hit. Niche life-sustain. |
| Broadhead Arrow Quiver | Bleed chance + duration. Bleed bows. |

### Shields

| Base | Why it matters |
|---|---|
| Champion Kite Shield | Pure armour. Common defensive shield. |
| Ezomyte Spiked Shield | Pure armour with attack-side use. |
| Pinnacle Tower Shield | Highest base armour. Niche pure-armour build. |
| Titanium Spirit Shield | Pure ES caster default. |
| Mosaic Kite Shield | Hybrid armour/evasion. |

---

## PoE2 — base type identity (current as of 0.4)

PoE2 is more **base-implicit-driven**. The Expert base tier is dramatically more powerful, and each base often has a defining build-relevant implicit.

### Expert bases

PoE2 introduces an "Expert" tier of bases — late-game, ilvl-gated, with significantly higher implicit ranges than Advanced. The single biggest driver of late-game gear quality.

| Tier | Implicit range |
|---|---|
| Basic | Lowest. |
| Advanced | Mid. |
| Expert | Highest; endgame-only. |

A craft target should always be on an Expert base for ilvl 80+ characters.

### Highlights (PoE2 0.4)

| Base | Why it matters |
|---|---|
| Expert Robe | Highest ES base (caster). |
| Expert Plate Vest | Highest armour base. |
| Expert Buckler / Tower Shield | Block/armour focus. |
| Expert War Quiver / Hunter's Quiver | Attack-skill quivers, varying implicit. |
| Expert Sceptre / Wand | Caster spell damage. |
| Expert Crossbow | Mercenary-class core base. |
| Expert Spear / Quarterstaff | Huntress / Monk weapon archetype defaults. |

PoE2 names are evolving across patches — verify current names against `repoe_lookup category=base_items` (Sprint 1+) or wiki.

### Runes-as-base-feature

PoE2 bases distinguish on **how many rune sockets** they support. A 2-socket base lets you slot 2 runes (effectively 2 free-ish modifiers via socketed runes); higher-socket bases dramatically increase a base's worth.

### Base implicits as identity

In PoE2, base implicits often *define* the base's role:

- A weapon with "+X% to attack speed" implicit becomes an attack-speed weapon.
- A ring with "+X to all elemental resistances" implicit becomes the universal resist ring.

Treat the implicit as the primary identifier — base name secondary.

---

## How the agent should reason about base selection

1. **What's the build's attribute split?** — pick a base that matches the build's primary attribute.
2. **What's the build's defensive type?** — match armour / evasion / ES / hybrid base.
3. **What ilvl?** — endgame target = ilvl 86 (PoE1) / current Expert tier (PoE2).
4. **What's the implicit doing?** — is it adding meaningful build power (Stellar Amulet, Two-Toned Boots), or just baseline?
5. **What Eldritch / craft-bench / Hateforge etc.?** — does this base support the modifiers the build needs?
6. **What's the alternative?** — there's almost always a tier-1 and a tier-2 viable base. Mention both.

The mistake to avoid: recommending a "best base" without stating *which build*. Bases are build-relative, not absolute.

## Cross-references

- `09_itemisation_crafting.md` — broader crafting principles, mod system, ilvl mechanics.
- `21_currency_and_barter_taxonomy.md` — orbs used to craft on these bases.
- `13_retrieval_playbooks.md` — base / mod search recipes.
- `26_validation_and_self_correction.md` Rule 4 — PoE1↔PoE2 disambiguation when discussing bases.
- `repoe_lookup` (Sprint 1+) `category=base_items` — current implicit values + ilvl gates.
- `poe2/00_version_pinning.md` — current PoE2 version anchor for Expert-tier specifics.
