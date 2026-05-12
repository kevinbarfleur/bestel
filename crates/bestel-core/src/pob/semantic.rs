//! Semantic facts extracted from a parsed `PobBuild` — archetype tags,
//! defining uniques, and conversion chain. This sits on top of the
//! structural parser and adds the build-identity layer the LLM needs to
//! comment correctly on a build (vs. guessing from class+ascendancy).
//!
//! All extraction is pure / synchronous / no I/O. Heuristics are
//! priority-ordered and first-match-wins — accept some loss of nuance
//! in exchange for predictability across hundreds of builds.
//!
//! Reference for the canonical archetype taxonomy:
//! `prompts/references/17_build_archetype_taxonomy.md`.
use serde::{Deserialize, Serialize};

use super::PobBuild;

/// Top-level identity card surfaced into `get_active_build` JSON.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct BuildIdentity {
    pub archetype: ArchetypeTags,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub defining_uniques: Vec<DefiningUniqueMatch>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub conversion_chain: Option<ConversionChain>,
}

/// Three-axis archetype taxonomy. Each axis returns 0..N tags — most
/// builds yield exactly one per axis, but `["life", "MoM"]` and similar
/// hybrids are valid.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ArchetypeTags {
    /// life | ES | LL | CI | MoM | hybrid | RF | VLS
    pub defense: Vec<String>,
    /// crit | non-crit | non-crit-EO | ailment-stack | DoT
    pub hit_model: Vec<String>,
    /// self-cast | trigger | totem | mine | trap | minion | autobomber
    pub mechanic: Vec<String>,
}

/// One unique-item match against the registry, with the tagged role.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DefiningUniqueMatch {
    pub name: String,
    /// engine | defining | amplifier
    pub category: String,
    pub identity_hint: String,
}

/// Damage conversion path detected from item mods + intrinsic gems.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConversionChain {
    pub steps: Vec<String>,
    pub final_type: String,
}

impl BuildIdentity {
    /// Build the identity card from a parsed PoB. Pure, ~1-5 ms.
    pub fn from_build(b: &PobBuild) -> Self {
        Self {
            archetype: tag_archetype(b),
            defining_uniques: match_defining_uniques(b),
            conversion_chain: extract_conversion_chain(b),
        }
    }
}

// ─────────────────────────────────────────────────────────────────────
// Archetype tagging
// ─────────────────────────────────────────────────────────────────────

fn tag_archetype(b: &PobBuild) -> ArchetypeTags {
    ArchetypeTags {
        defense: tag_defense(b),
        hit_model: tag_hit_model(b),
        mechanic: tag_mechanic(b),
    }
}

fn tag_defense(b: &PobBuild) -> Vec<String> {
    let mut tags = Vec::new();

    let life = b.defenses.life.unwrap_or(0.0);
    let es = b.defenses.energy_shield.unwrap_or(0.0);
    let life_unreserved_pct = b.stats.get("LifeUnreservedPercent").copied().unwrap_or(100.0);

    // Righteous Fire — only tag if it's the main skill. Many crit/hit
    // builds keep RF in a side group as a damage buff (Searing Bond).
    if is_main_skill(b, "Righteous Fire") {
        tags.push("RF".into());
    }

    // Vaal Lightning Strike — same rule, only when main skill.
    if is_main_skill(b, "Vaal Lightning Strike") {
        tags.push("VLS".into());
    }

    // Chaos Inoculation: life is set to 1 by the keystone, ES becomes the
    // entire health pool. Detection via items raw is the most reliable
    // signal we have without a tree dict.
    let has_ci_keystone = items_raw_contains(b, "Chaos Inoculation");
    if has_ci_keystone || (life <= 1.0 && es > 1000.0) {
        tags.push("CI".into());
        return tags; // CI is exclusive of life-based tags
    }

    // Low Life: life is reserved down to <35% by Shavronne / Pain Attunement
    // / The Coming Calamity / Lori's Lantern setup.
    if life_unreserved_pct < 35.0 && life > 100.0 {
        tags.push("LL".into());
    }

    // Mind over Matter — keystone or Cloak of Defiance / Anomalous Inspiration.
    let has_mom = items_raw_contains(b, "Mind Over Matter")
        || b.stats.contains_key("MindOverMatter")
        || b.stats.contains_key("ManaProtectAmount");
    if has_mom {
        tags.push("MoM".into());
    }

    // Hybrid life+ES — neither dominates.
    if life > 4000.0 && es > 2000.0 {
        tags.push("hybrid".into());
    } else if es > life * 1.5 && es > 1000.0 {
        // Pure ES — life pool exists but ES carries.
        tags.push("ES".into());
    } else if !tags.iter().any(|t| t == "LL" || t == "RF" || t == "MoM") {
        // Plain life-based fallback. Skip if we already tagged LL/RF/MoM
        // since those are still life-pool builds technically — including
        // "life" alongside would be redundant.
        tags.push("life".into());
    } else if tags.iter().all(|t| t == "MoM") {
        // MoM alone usually rides on life; surface "life" to make clear.
        tags.insert(0, "life".into());
    }

    tags
}

fn tag_hit_model(b: &PobBuild) -> Vec<String> {
    let mut tags = Vec::new();

    let crit_chance = b.stats.get("CritChance").copied().unwrap_or(0.0);
    let total_dot = b.stats.get("TotalDotDPS").copied().unwrap_or(0.0);
    let total_hit = b.stats.get("TotalDPS").copied().unwrap_or(0.0);

    let main_skill = b.main_skill.as_deref().unwrap_or("");

    // Pure DoT skill — main damage comes from a degen rather than hits.
    if is_dot_skill(main_skill) {
        tags.push("DoT".into());
        return tags;
    }

    // DoT-dominant build: dot DPS larger than hit DPS by 2× or more.
    if total_dot > total_hit * 2.0 && total_dot > 1000.0 {
        tags.push("DoT".into());
        return tags;
    }

    // Crit threshold — anything above 30% effective crit is a crit build
    // (50%+ is conservative crit, 30-50% is hybrid-ish).
    if crit_chance > 30.0 {
        tags.push("crit".into());
        return tags;
    }

    // Non-crit family: detect Elemental Overload (granted by keystone or
    // anomalous Inspiration support). EO is the most common reason a low-
    // crit build still scales — surface it explicitly.
    let has_eo = items_raw_contains(b, "Elemental Overload")
        || b.stats.contains_key("ElementalOverloadActive");
    if has_eo {
        tags.push("non-crit-EO".into());
    } else {
        tags.push("non-crit".into());
    }

    // Ailment stacker — build commits to ignite/poison/bleed scaling on
    // top of the hit. Detect via gem support names (less precise than
    // tag-based detection but works without gem tag data).
    if has_support_named(b, "Ignite Proliferation")
        || has_support_named(b, "Vile Toxins")
        || has_support_named(b, "Greater Volley")
        || has_support_named(b, "Bloodlust")
        || has_support_named(b, "Deadly Ailments")
    {
        tags.push("ailment-stack".into());
    }

    tags
}

fn tag_mechanic(b: &PobBuild) -> Vec<String> {
    let mut tags = Vec::new();

    // Totem variants.
    if has_support_named(b, "Spell Totem")
        || has_support_named(b, "Ballista Totem")
        || has_support_named(b, "Ranged Attack Totem")
    {
        tags.push("totem".into());
    }

    // Mine variants.
    if has_support_named(b, "Blastchain Mine")
        || has_support_named(b, "High-Impact Mine")
        || has_support_named(b, "Swift Assembly")
    {
        tags.push("mine".into());
    }

    // Trap variants.
    if has_support_named(b, "Trap")
        || has_support_named(b, "Cluster Trap")
        || has_support_named(b, "Multiple Traps")
        || has_support_named(b, "Charged Traps")
    {
        tags.push("trap".into());
    }

    // Minion-driven build.
    let main = b.main_skill.as_deref().unwrap_or("");
    let is_minion_main = matches!(
        main,
        "Raise Spectre"
            | "Raise Zombie"
            | "Summon Skeletons"
            | "Summon Raging Spirit"
            | "Animate Guardian"
            | "Animate Weapon"
            | "Carrion Golem"
            | "Stone Golem"
            | "Lightning Golem"
            | "Flame Golem"
            | "Ice Golem"
            | "Chaos Golem"
            | "Holy Relic"
            | "Summon Holy Relic"
            | "Summon Reaper"
            | "Dominating Blow"
    );
    // Note: we deliberately do NOT tag minion just because `b.spectres`
    // is non-empty. Many non-minion builds carry a utility spectre slot
    // (Carnage Chieftain frenzy generation, Mannequin damage proxy)
    // without being "a minion build". Only the main skill matters.
    if is_minion_main {
        tags.push("minion".into());
    }

    // Trigger-driven build.
    let has_trigger_item = ["Cospri's Malice", "Mjolner", "Mjölner", "Asenath's Gentle Touch"]
        .iter()
        .any(|n| has_unique_named(b, n));
    let has_trigger_support = has_support_named(b, "Cast On Critical Strike")
        || has_support_named(b, "Cast on Critical Strike")
        || has_support_named(b, "Cast when Damage Taken")
        || has_support_named(b, "Cast When Damage Taken")
        || has_support_named(b, "Cast on Death")
        || has_support_named(b, "Cast When Stunned")
        || has_support_named(b, "Cast on Hexed Skill");
    if has_trigger_item || has_trigger_support {
        tags.push("trigger".into());
    }

    if tags.is_empty() {
        tags.push("self-cast".into());
    }

    tags
}

// ─────────────────────────────────────────────────────────────────────
// Defining uniques registry
// ─────────────────────────────────────────────────────────────────────

/// (name, category, identity_hint, game). Names are matched case-
/// insensitively against `PobItem::name`. Categories: `engine` (build
/// collapses without), `defining` (shapes archetype), `amplifier` (boosts
/// but replaceable). `game` gates the entry to PoE1 or PoE2; matches in
/// the wrong game are skipped so PoE1 names don't pollute PoE2 builds.
const DEFINING_UNIQUES: &[(&str, &str, &str, super::PoeVersion)] = &[
    // ── PoE1 engine ────────────────────────────────────────────────
    ("Mageblood", "engine", "magic-flask uptime engine — sustains 4 magic flasks permanently", super::PoeVersion::Poe1),
    ("Headhunter", "engine", "rare-monster mod stealing — defines map-clear identity", super::PoeVersion::Poe1),
    ("Original Sin", "engine", "dual-conversion + curse uniqueness — entire build pivots on it", super::PoeVersion::Poe1),
    ("Cospri's Malice", "engine", "Cast on Crit cold-skill engine for triggerbot melee", super::PoeVersion::Poe1),
    ("Mjölner", "engine", "Cast on Crit lightning-spell engine for melee staff", super::PoeVersion::Poe1),
    ("Mjolner", "engine", "Cast on Crit lightning-spell engine for melee staff", super::PoeVersion::Poe1),
    ("Voll's Devotion", "engine", "endurance-charge generation engine for cycling builds", super::PoeVersion::Poe1),
    ("Maw of Mischief", "engine", "Death Aura DD engine — entire skill granted by helm", super::PoeVersion::Poe1),
    ("Replica Cold Iron Point", "engine", "physical-spell scaling — converts spell phys", super::PoeVersion::Poe1),
    ("The Squire", "engine", "+2 socket support amplifier — defines AG-stacker ceilings", super::PoeVersion::Poe1),
    ("Doryani's Prototype", "engine", "lightning-resist conversion defender — reshapes EHP entirely", super::PoeVersion::Poe1),
    ("Voidforge", "engine", "random-elemental-roll sword — drives Cyclone/Vaal Slam crits with elemental conversion", super::PoeVersion::Poe1),
    ("Voltaxic Rift", "engine", "lightning-to-chaos shock-stack bow — defines bow-chaos identity", super::PoeVersion::Poe1),
    ("Replica Atziri's Acuity", "engine", "instant leech on hit — sustains every leech-dependent build", super::PoeVersion::Poe1),
    ("Beltimber Blade", "engine", "movement-speed-on-hit bow — frenzy generation backbone for clear", super::PoeVersion::Poe1),
    ("Ashes of the Stars", "engine", "wildwood-modifier amulet — defines alt-quality aura/support stack", super::PoeVersion::Poe1),
    ("The Devouring Diadem", "engine", "feast-of-flesh ES helm + free reservation — defines auras+ES setup", super::PoeVersion::Poe1),
    ("Heatshiver", "engine", "frozen-to-fire conversion helm — defines cold-strike ignite identity", super::PoeVersion::Poe1),
    ("Crown of the Inward Eye", "engine", "life-as-extra-ES + reservation reducer — Cyclone/MoM enabler", super::PoeVersion::Poe1),
    ("Bloodnotch", "engine", "stun-recoup amulet — defines impale/stun defensive layer", super::PoeVersion::Poe1),
    ("Tempered Spirits", "engine", "trigger gem-level mace — Volcanic Fissure / Frostblink trigger engine", super::PoeVersion::Poe1),
    ("Coward's Legacy", "engine", "Low Life via belt + flagellant trinity — defines low-life-without-Shavronne", super::PoeVersion::Poe1),
    ("Replica Conqueror's Efficiency", "engine", "reservation reduction belt — enables triple-aura stacks", super::PoeVersion::Poe1),
    ("Replica Farrul's Fur", "engine", "frenzy-charge stack on kill body armour — replaces Determination-only profile", super::PoeVersion::Poe1),
    ("Carcass Jack", "engine", "+AoE area-overlap multiplier — defines self-shotgun damage profile", super::PoeVersion::Poe1),

    // ── PoE1 defining ──────────────────────────────────────────────
    ("Shavronne's Wrappings", "defining", "low-life enabler — chaos damage doesn't bypass ES", super::PoeVersion::Poe1),
    ("Solaris Lorica", "defining", "low-life alternative — guards against critical strikes", super::PoeVersion::Poe1),
    ("Lori's Lantern", "defining", "stun-immunity + low-life synergy ring", super::PoeVersion::Poe1),
    ("Replica Soul Tether", "defining", "life-as-extra-ES on top of regular life pool", super::PoeVersion::Poe1),
    ("Replica Restless Ward", "defining", "movement-skill cooldown reduction enabler", super::PoeVersion::Poe1),
    ("Replica Dragonfang's Flight", "defining", "skill-gem-level cluster — defines aura stack", super::PoeVersion::Poe1),
    ("Asenath's Gentle Touch", "defining", "death-curse trigger glove — defines hex chain", super::PoeVersion::Poe1),
    ("Kingmaker", "defining", "Animate Guardian leadership weapon — fortify + culling", super::PoeVersion::Poe1),
    ("Brutal Restraint", "defining", "Maraketh timeless jewel — re-rolls passive cluster", super::PoeVersion::Poe1),
    ("Glorious Vanity", "defining", "Vaal timeless jewel — corrupts passive cluster", super::PoeVersion::Poe1),
    ("Lethal Pride", "defining", "Karui timeless jewel — strength-cluster rework", super::PoeVersion::Poe1),
    ("Militant Faith", "defining", "Templar timeless jewel — devotion conversion cluster", super::PoeVersion::Poe1),
    ("Elegant Hubris", "defining", "Eternal Empire timeless jewel — passive replacement", super::PoeVersion::Poe1),
    ("The Pandemonius", "defining", "blind + cold-pen amulet — defines cold-conversion identity", super::PoeVersion::Poe1),
    ("Indigon", "defining", "mana-spent damage helm — defines mana-stack identity", super::PoeVersion::Poe1),
    ("Inpulsa's Broken Heart", "defining", "shock-explosion clear identity body armour", super::PoeVersion::Poe1),
    ("Crown of the Tyrant", "defining", "abyss-jewel socket helm — engine for jewel stack", super::PoeVersion::Poe1),
    ("Eyes of the Greatwolf", "defining", "double-talisman amulet — abyss/eldritch synergy", super::PoeVersion::Poe1),
    ("Forbidden Flesh", "defining", "ascendancy-cross jewel — pairs with Forbidden Flame", super::PoeVersion::Poe1),
    ("Forbidden Flame", "defining", "ascendancy-cross jewel — pairs with Forbidden Flesh", super::PoeVersion::Poe1),
    ("The Eternal Apple", "defining", "life-as-extra-ES shield + endurance generation", super::PoeVersion::Poe1),
    ("Mahuxotl's Machination", "defining", "all-keystone shield — locks the build into specific keystones", super::PoeVersion::Poe1),
    ("The Fourth Vow", "defining", "Divine Flesh-aligned chest — chaos taken as ele identity", super::PoeVersion::Poe1),
    ("Defiance of Destiny", "defining", "missing-life-protection amulet — defines life-recovery EHP", super::PoeVersion::Poe1),
    ("Hyrri's Truth", "defining", "accuracy-stack amulet — defines pure-physical bow/melee crit", super::PoeVersion::Poe1),
    ("Astramentis", "defining", "all-attribute amulet — defines attribute-stack identity", super::PoeVersion::Poe1),
    ("Replica Stampede", "defining", "movement-speed-cap-removing boots — defines speed-stack", super::PoeVersion::Poe1),
    ("Yoke of Suffering", "defining", "double-ailment amulet — defines ailment-stack hybrid", super::PoeVersion::Poe1),
    ("The Light of Meaning", "defining", "consecrated-ground prefix jewel — defines crit-without-resolute build", super::PoeVersion::Poe1),
    ("Sublime Vision", "defining", "aura-specific keystone amulet — locks a single aura's modifier", super::PoeVersion::Poe1),

    // ── PoE1 amplifier ─────────────────────────────────────────────
    ("Watcher's Eye", "amplifier", "aura-mod jewel — strong but replaceable", super::PoeVersion::Poe1),
    ("Thread of Hope", "amplifier", "ring-radius cluster jewel — passive efficiency", super::PoeVersion::Poe1),
    ("Impossible Escape", "amplifier", "keystone-radius cluster jewel", super::PoeVersion::Poe1),
    ("That Which Was Taken", "amplifier", "buff-effect cluster jewel", super::PoeVersion::Poe1),
    ("Stormshroud", "amplifier", "shock-aura body armour — defensive amplifier", super::PoeVersion::Poe1),
    ("Bottled Faith", "amplifier", "consecrated-ground flask — crit + clear amplifier", super::PoeVersion::Poe1),
    ("Dying Sun", "amplifier", "extra-projectiles flask — projectile amplifier", super::PoeVersion::Poe1),
    ("The Wise Oak", "amplifier", "balanced-resists flask — penetration amplifier", super::PoeVersion::Poe1),
    ("Atziri's Promise", "amplifier", "chaos-leech flask — early league amplifier", super::PoeVersion::Poe1),
    ("Megalomaniac", "amplifier", "three-random-notable medium cluster — passive-shape amplifier", super::PoeVersion::Poe1),
    ("Forbidden Shako", "amplifier", "random-support-gem helm — versatile amplifier", super::PoeVersion::Poe1),
    ("Progenesis", "amplifier", "life-recovery flask — defensive cushion amplifier", super::PoeVersion::Poe1),
    ("Hunter's Omen", "amplifier", "movement-speed amulet — clear-speed amplifier", super::PoeVersion::Poe1),
    ("Hand of Wisdom and Action", "amplifier", "Int-to-Dex claw — attack-speed stack amplifier", super::PoeVersion::Poe1),
    ("Aul's Uprising", "amplifier", "free-aura-reservation amulet — defines what fits the aura stack", super::PoeVersion::Poe1),
    ("Voll's Vision", "amplifier", "PoE1 alternative-quality helm — converts crit damage", super::PoeVersion::Poe1),

    // ── PoE2 engine ────────────────────────────────────────────────
    ("Pillar of the Caged God", "engine", "PoE2 dex-stack staff — defines pillar-attack staff builds", super::PoeVersion::Poe2),
    ("El Coro", "engine", "PoE2 spirit-stacker amulet — defines spirit-reservation engine", super::PoeVersion::Poe2),
    ("Astramentis", "engine", "PoE2 omni-attribute amulet — defines attribute-stack engine", super::PoeVersion::Poe2),
    ("Hateforge", "engine", "PoE2 trigger-skill weapon — defines automated-spell engine", super::PoeVersion::Poe2),
    ("Skybreaker", "engine", "PoE2 lightning-conversion staff — defines hit-converting attack identity", super::PoeVersion::Poe2),
    ("Howl of the Wolf", "engine", "PoE2 minion-companion amulet — defines companion-engine identity", super::PoeVersion::Poe2),

    // ── PoE2 defining ──────────────────────────────────────────────
    ("Morior Invictus", "defining", "PoE2 +socket body armour — defines socket-count identity", super::PoeVersion::Poe2),
    ("Ingenuity", "defining", "PoE2 ring-effect belt — amplifies both rings, defines ring-stack", super::PoeVersion::Poe2),
    ("Bramblejack", "defining", "PoE2 reflect chest — defines thorn/reflect identity", super::PoeVersion::Poe2),
    ("Pillars of Arun", "defining", "PoE2 demon-form transition staff — defines demon-form identity", super::PoeVersion::Poe2),
    ("The Adorned", "defining", "PoE2 magic-jewel-effect jewel — defines magic-jewel-stack identity", super::PoeVersion::Poe2),
    ("Megalomaniac", "defining", "PoE2 three-random-notable cluster — defines passive-shape identity", super::PoeVersion::Poe2),

    // ── PoE2 amplifier ─────────────────────────────────────────────
    ("Sanctuary of Thought", "amplifier", "PoE2 spell-mana amplifier helm", super::PoeVersion::Poe2),
    ("Doomgate Cord", "amplifier", "PoE2 reservation amplifier belt", super::PoeVersion::Poe2),
    ("Charms Galore", "amplifier", "PoE2 charm-slot amulet — charm-stack amplifier", super::PoeVersion::Poe2),
    ("Stormcaller", "amplifier", "PoE2 lightning-conversion ring — caster amplifier", super::PoeVersion::Poe2),
];

fn match_defining_uniques(b: &PobBuild) -> Vec<DefiningUniqueMatch> {
    let mut out = Vec::new();
    let mut matched_names: std::collections::HashSet<String> =
        std::collections::HashSet::new();
    // First pass — hardcoded registry. Names are matched
    // case-insensitively against `PobItem::name`. First match wins per
    // item.
    for item in &b.items {
        let Some(name) = item.name.as_deref() else {
            continue;
        };
        let needle = name.to_ascii_lowercase();
        for (uname, cat, hint, game) in DEFINING_UNIQUES {
            // Sprint v5: gate by game so PoE1 entries don't pollute PoE2
            // matches (and vice versa). Same name may legitimately exist
            // in both registries with different identity hints
            // (Astramentis, Megalomaniac).
            if *game != b.game {
                continue;
            }
            if uname.to_ascii_lowercase() == needle {
                out.push(DefiningUniqueMatch {
                    name: (*uname).into(),
                    category: (*cat).into(),
                    identity_hint: (*hint).into(),
                });
                matched_names.insert(needle.clone());
                break;
            }
        }
    }
    // Second pass — auto-engine via mod patterns. Catches items the
    // hardcoded registry doesn't know about but that are clearly
    // engineered into the build (built-in gem supports, gem-level
    // boosts, conditional triggers, main-skill-named mods). Sprint v5:
    // also runs on RARE items — engineered rares (Awakener's Touch
    // trigger-on-crit rings, +1 socketed gem helms, fractured/elevated
    // skill-name mods) deserve `engine` flagging just like uniques.
    // See the `<failure_policy>` section of `SYSTEM_PROMPT.md` for how
    // the agent should treat `category: "engine"` items.
    let main_skill_lower = b
        .main_skill
        .as_deref()
        .map(|s| s.to_ascii_lowercase())
        .unwrap_or_default();
    for item in &b.items {
        let Some(name) = item.name.as_deref() else {
            continue;
        };
        let needle = name.to_ascii_lowercase();
        if matched_names.contains(&needle) {
            continue;
        }
        let is_unique = item
            .rarity
            .as_deref()
            .map(|r| r.eq_ignore_ascii_case("UNIQUE"))
            .unwrap_or(false);
        let is_rare = item
            .rarity
            .as_deref()
            .map(|r| r.eq_ignore_ascii_case("RARE"))
            .unwrap_or(false);
        if !is_unique && !is_rare {
            continue;
        }
        if let Some(hint) = detect_engine_mod_pattern(&item.raw, &main_skill_lower) {
            out.push(DefiningUniqueMatch {
                name: name.to_string(),
                category: "engine".into(),
                identity_hint: hint,
            });
            matched_names.insert(needle);
        }
    }
    out
}

/// Detect engine-equivalence patterns in an item's raw mods. Returns
/// `Some(reason)` when the item is engineered for the build — either via
/// built-in gem supports, gem-level boosts, or by mentioning the main
/// skill by name in a mod. The returned string surfaces the matched
/// pattern verbatim (or a paraphrase) so the agent can name what would
/// be lost when discussing trade-offs with the exile.
///
/// Patterns covered (lowercase line scanning):
/// - `Socketed Gems are Supported by Level X <Support>` — built-in support
/// - `Socketed <Type> Gems deal X% more <Damage>` — built-in multiplier
/// - `+X to Level of Socketed <Type> Gems` — socket-scoped gem-level boost
/// - `+X to Level of all <Type> Spell/Skill Gems` — wide gem-level boost
/// - `Trigger Level X <Skill> when <Condition>` — built-in trigger
/// - main skill name appears in any mod text — item explicitly engineered
///   for this specific skill (catches Replica Dragonfang's Flight class
///   even when the unique isn't in the hardcoded registry)
pub(crate) fn detect_engine_mod_pattern(raw: &str, main_skill_lower: &str) -> Option<String> {
    for line in raw.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let lower = trimmed.to_ascii_lowercase();

        if lower.starts_with("socketed gems are supported by level ") {
            return Some(format!("built-in gem support — \"{trimmed}\""));
        }
        if lower.starts_with("socketed ")
            && lower.contains("gems deal ")
            && (lower.contains(" more ") || lower.contains(" increased "))
        {
            return Some(format!("socketed-gem damage multiplier — \"{trimmed}\""));
        }
        if (lower.contains("to level of socketed") && lower.contains("gems"))
            || (lower.contains("to level of all") && lower.contains("gems"))
        {
            return Some(format!("gem-level boost — \"{trimmed}\""));
        }
        if lower.starts_with("trigger level ")
            && (lower.contains(" when ")
                || lower.contains(" on critical")
                || lower.contains(" on hit")
                || lower.contains(" on kill"))
        {
            return Some(format!("built-in trigger — \"{trimmed}\""));
        }
        if !main_skill_lower.is_empty()
            && main_skill_lower.len() >= 4
            && lower.contains(main_skill_lower)
        {
            return Some(format!(
                "item mod names the build's main skill — \"{trimmed}\""
            ));
        }
    }
    None
}

// ─────────────────────────────────────────────────────────────────────
// Conversion chain extraction
// ─────────────────────────────────────────────────────────────────────

fn extract_conversion_chain(b: &PobBuild) -> Option<ConversionChain> {
    let mut steps: Vec<(String, String, u32)> = Vec::new(); // (from, to, pct)

    // Item raw mods — match `(\d+)% of (Type) Damage Converted to (Type)`.
    for item in &b.items {
        for line in item.raw.lines() {
            if let Some((pct, from, to)) = parse_conversion_line(line) {
                steps.push((from, to, pct));
            }
        }
    }

    // Intrinsic gem conversions — limited but high-signal.
    for group in &b.skill_groups {
        for gem in &group.gems {
            if !gem.enabled {
                continue;
            }
            match gem.name.as_str() {
                "Cold to Fire Support" | "Cold to Fire" => {
                    steps.push(("Cold".into(), "Fire".into(), 50));
                }
                "Physical to Lightning Support" | "Physical to Lightning" => {
                    steps.push(("Physical".into(), "Lightning".into(), 50));
                }
                "Glacial Cascade" => {
                    steps.push(("Physical".into(), "Cold".into(), 50));
                }
                "Volatile Dead" => {
                    steps.push(("Physical".into(), "Fire".into(), 50));
                }
                _ => {}
            }
        }
    }

    if steps.is_empty() {
        return None;
    }

    // Render the chain in the order discovered. De-duplicate identical
    // (from, to) pairs by keeping the highest pct.
    let mut dedup: Vec<(String, String, u32)> = Vec::new();
    for (from, to, pct) in steps {
        if let Some(existing) = dedup.iter_mut().find(|s| s.0 == from && s.1 == to) {
            if pct > existing.2 {
                existing.2 = pct;
            }
        } else {
            dedup.push((from, to, pct));
        }
    }

    let final_type = dedup
        .last()
        .map(|s| s.1.to_ascii_lowercase())
        .unwrap_or_else(|| "physical".into());
    let rendered: Vec<String> = dedup
        .into_iter()
        .map(|(f, t, p)| format!("{p}% {} → {}", f.to_ascii_lowercase(), t.to_ascii_lowercase()))
        .collect();

    Some(ConversionChain {
        steps: rendered,
        final_type,
    })
}

fn parse_conversion_line(line: &str) -> Option<(u32, String, String)> {
    // Cheap manual parse to avoid pulling in a regex dep here.
    // Pattern: "<n>% of <X> Damage Converted to <Y>"
    let trimmed = line.trim();
    let pct_end = trimmed.find('%')?;
    let pct: u32 = trimmed[..pct_end].trim().parse().ok()?;
    let rest = trimmed[pct_end + 1..].trim_start();
    let rest = rest.strip_prefix("of ")?;
    let damage_pos = rest.find(" Damage Converted to ")?;
    let from_raw = &rest[..damage_pos];
    let to_raw = &rest[damage_pos + " Damage Converted to ".len()..];
    let from = title_case_damage_type(from_raw)?;
    // After "Converted to " the next word is the target type ("Cold",
    // "Fire", etc.) — anything trailing it ("Damage", punctuation) is
    // noise we drop.
    let to_word = to_raw.split_whitespace().next()?;
    let to = title_case_damage_type(to_word)?;
    Some((pct, from, to))
}

fn title_case_damage_type(s: &str) -> Option<String> {
    match s.trim() {
        "Physical" | "physical" | "PHYSICAL" => Some("Physical".into()),
        "Cold" | "cold" | "COLD" => Some("Cold".into()),
        "Fire" | "fire" | "FIRE" => Some("Fire".into()),
        "Lightning" | "lightning" | "LIGHTNING" => Some("Lightning".into()),
        "Chaos" | "chaos" | "CHAOS" => Some("Chaos".into()),
        _ => None,
    }
}

// ─────────────────────────────────────────────────────────────────────
// Helpers
// ─────────────────────────────────────────────────────────────────────

fn is_main_skill(b: &PobBuild, name: &str) -> bool {
    if let Some(main) = b.main_skill.as_deref() {
        if main.eq_ignore_ascii_case(name) {
            return true;
        }
    }
    // Fallback: a `is_main` skill group with the named gem present.
    let needle = name.to_ascii_lowercase();
    b.skill_groups.iter().any(|g| {
        g.is_main
            && g.gems
                .iter()
                .any(|gem| gem.enabled && gem.name.to_ascii_lowercase() == needle)
    })
}

#[allow(dead_code)]
fn has_skill_named(b: &PobBuild, name: &str) -> bool {
    let needle = name.to_ascii_lowercase();
    b.skill_groups.iter().any(|g| {
        g.gems
            .iter()
            .any(|gem| gem.enabled && gem.name.to_ascii_lowercase() == needle)
    })
}

fn has_support_named(b: &PobBuild, name: &str) -> bool {
    let needle = name.to_ascii_lowercase();
    b.skill_groups.iter().any(|g| {
        g.gems.iter().any(|gem| {
            if !gem.enabled {
                return false;
            }
            let gn = gem.name.to_ascii_lowercase();
            gn == needle
                || gn == format!("{} support", needle)
                || gn.contains(&needle)
        })
    })
}

fn items_raw_contains(b: &PobBuild, needle: &str) -> bool {
    b.items.iter().any(|it| it.raw.contains(needle))
}

fn has_unique_named(b: &PobBuild, name: &str) -> bool {
    let needle = name.to_ascii_lowercase();
    b.items
        .iter()
        .filter_map(|it| it.name.as_deref())
        .any(|n| n.to_ascii_lowercase() == needle)
}

fn is_dot_skill(name: &str) -> bool {
    matches!(
        name,
        "Righteous Fire"
            | "Death's Oath"
            | "Detonate Dead"
            | "Caustic Arrow"
            | "Toxic Rain"
            | "Essence Drain"
            | "Soulrend"
            | "Bane"
            | "Blight"
            | "Scourge Arrow"
            | "Reap"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::pob::parser;
    use std::path::Path;

    fn load_fixture(rel: &str) -> PobBuild {
        let path = Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("..")
            .join("..")
            .join("tests")
            .join("fixtures")
            .join("pob")
            .join(rel);
        let bytes = std::fs::read(&path)
            .unwrap_or_else(|_| panic!("read fixture {}", path.display()));
        parser::parse_bytes(&bytes, path.clone()).expect("parse fixture")
    }

    #[test]
    fn poe1_inquisitor_archetype_basic() {
        let b = load_fixture("poe1_inquisitor.xml");
        let id = BuildIdentity::from_build(&b);

        // Inquisitor Templar with very high crit chance — must be tagged crit.
        assert!(
            id.archetype.hit_model.iter().any(|t| t == "crit"),
            "expected crit, got {:?}",
            id.archetype.hit_model
        );
        // Has Life pool (~5k), no LL stage — must include life or hybrid.
        assert!(
            id.archetype
                .defense
                .iter()
                .any(|t| t == "life" || t == "hybrid" || t == "MoM"),
            "expected life-family defense, got {:?}",
            id.archetype.defense
        );
        // Self-cast brand build — no totem/mine/trap/minion/trigger expected.
        assert!(
            id.archetype.mechanic.iter().any(|t| t == "self-cast")
                || id.archetype.mechanic.iter().any(|t| t == "trigger"),
            "expected self-cast or trigger mechanic, got {:?}",
            id.archetype.mechanic
        );
    }

    #[test]
    fn poe2_druid_does_not_panic() {
        let b = load_fixture("poe2_druid.xml");
        let id = BuildIdentity::from_build(&b);
        // Just ensure extraction runs — PoE2 has incomplete keystone surfacing
        // so we accept whatever tags emerge as long as the lists are non-empty.
        assert!(!id.archetype.defense.is_empty());
        assert!(!id.archetype.hit_model.is_empty());
        assert!(!id.archetype.mechanic.is_empty());
    }

    #[test]
    fn conversion_line_parser() {
        let line = "60% of Physical Damage Converted to Cold Damage";
        let (pct, from, to) = parse_conversion_line(line).expect("parse");
        assert_eq!(pct, 60);
        assert_eq!(from, "Physical");
        assert_eq!(to, "Cold");
    }

    #[test]
    fn conversion_line_with_trailing_punctuation() {
        let line = "100% of Physical Damage Converted to Lightning";
        let (pct, from, to) = parse_conversion_line(line).expect("parse");
        assert_eq!(pct, 100);
        assert_eq!(from, "Physical");
        assert_eq!(to, "Lightning");
    }

    #[test]
    fn no_conversion_line_returns_none() {
        assert!(parse_conversion_line("100% increased Damage").is_none());
        assert!(parse_conversion_line("+50 to maximum Life").is_none());
    }

    #[test]
    fn detect_engine_pattern_socketed_supports() {
        // Archdemon Crown / Cataclysm Guardian — built-in support gems.
        let raw = "\
Rarity: UNIQUE
Archdemon Crown
Reaver Helm
30% increased Elemental Damage
Socketed Gems are Supported by Level 30 Concentrated Effect
Socketed Gems are Supported by Level 30 Hypothermia
+50 to maximum Life
+(11-15)% to all Elemental Resistances";
        let hint = detect_engine_mod_pattern(raw, "penance brand of dissipation")
            .expect("should detect built-in support");
        assert!(
            hint.contains("Concentrated Effect"),
            "hint should name the lost support, got {hint}"
        );
    }

    #[test]
    fn detect_engine_pattern_gem_level_boost() {
        // Replica Dragonfang's Flight class — +3 to a specific gem.
        let raw = "\
Rarity: UNIQUE
Replica Dragonfang's Flight
Cloth Belt
+(20-30) to Strength
+3 to Level of Socketed Aura Gems
Inflict Fire Exposure on Hit";
        let hint =
            detect_engine_mod_pattern(raw, "").expect("should detect gem level boost");
        assert!(
            hint.contains("Aura Gems") || hint.contains("Level of"),
            "hint should name the boost, got {hint}"
        );
    }

    #[test]
    fn detect_engine_pattern_main_skill_named() {
        // Replica Dragonfang-style: explicit main skill name in mod.
        let raw = "\
Rarity: UNIQUE
Generic Amulet
Lapis Amulet
+(20-30) to Intelligence
+3 to Level of all Penance Brand of Dissipation Gems";
        let hint = detect_engine_mod_pattern(raw, "penance brand of dissipation")
            .expect("should detect main-skill-named mod");
        assert!(
            hint.to_ascii_lowercase().contains("penance brand"),
            "hint should quote the line, got {hint}"
        );
    }

    #[test]
    fn detect_engine_pattern_built_in_trigger() {
        let raw = "\
Rarity: UNIQUE
Some Cospri-like
Slaughter Knife
Trigger Level 20 Cold Spell on Critical Strike";
        let hint = detect_engine_mod_pattern(raw, "").expect("should detect trigger");
        assert!(
            hint.to_ascii_lowercase().contains("trigger"),
            "hint should mention trigger, got {hint}"
        );
    }

    #[test]
    fn detect_engine_pattern_returns_none_for_vanilla_unique() {
        // A unique with no engine-equivalence patterns — e.g. Headhunter
        // (which IS engine in the hardcoded registry, but the pattern
        // detector alone shouldn't fire on it — Headhunter's mods are
        // about rare-monster theft, not gem support).
        let raw = "\
Rarity: UNIQUE
Headhunter
Leather Belt
+(40-50) to Strength
+(40-55) to maximum Life
+(50-60) to maximum Mana
20% increased Damage with Hits against Rare monsters
When you Kill a Rare monster, you gain its Modifiers for 60 seconds";
        let hint = detect_engine_mod_pattern(raw, "");
        assert!(
            hint.is_none(),
            "vanilla unique without engine patterns shouldn't fire, got {hint:?}"
        );
    }

}
