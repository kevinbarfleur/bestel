pub mod dict;
pub mod locator;
pub mod parser;
pub mod semantic;
pub mod watcher;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::SystemTime;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum PoeVersion {
    Poe1,
    Poe2,
}

impl PoeVersion {
    pub fn label(self) -> &'static str {
        match self {
            PoeVersion::Poe1 => "PoE1",
            PoeVersion::Poe2 => "PoE2",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobItem {
    pub id: String,
    pub rarity: Option<String>,
    pub name: Option<String>,
    pub base: Option<String>,
    pub raw: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PobSkillGem {
    pub name: String,
    pub level: Option<u32>,
    pub quality: Option<u32>,
    pub enabled: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub skill_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variant_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gem_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stat_set_index: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub is_minion: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobSkillGroup {
    pub label: String,
    pub is_main: bool,
    pub gems: Vec<PobSkillGem>,
    /// Gear slot the skill is socketed in: `Helmet`, `Body Armour`, `Gloves`,
    /// `Boots`, `Weapon 1`, `Weapon 2`, `Ring 1`, `Amulet`, etc. Set from
    /// `<Skill slot="...">`. Some skills (e.g. granted by an item, or the
    /// implicit "shared" group) have no slot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub slot: Option<String>,
}

/// Pantheon + bandit choice (PoE1 only). Empty on PoE2.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PoePantheon {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub major: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub minor: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bandit: Option<String>,
}

impl PoePantheon {
    pub fn is_empty(&self) -> bool {
        self.major.is_none() && self.minor.is_none() && self.bandit.is_none()
    }
}

/// PoE1 tattoo override — replaces a passive node with a Karui tattoo effect.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobTattoo {
    pub node_id: u32,
    pub display_name: String,
    pub body: String,
}

/// One mastery effect picked. PoE1 stores `{nodeId,effectId}` pairs;
/// PoE2 stores just effect ids — we expose both shapes via the variant.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "lowercase")]
pub enum MasteryPick {
    /// PoE1 — node where the mastery sits + chosen effect id.
    Poe1 { node_id: u32, effect_id: u32 },
    /// PoE2 — only the effect id is stored in the XML.
    Poe2 { effect_id: u32 },
}

/// A jewel item placed in a tree socket.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JewelPlacement {
    pub node_id: u32,
    pub item_id: u32,
}

/// Single live charge pool (current vs max).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct Charge {
    pub current: u32,
    pub max: u32,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PobCharges {
    pub power: Charge,
    pub frenzy: Charge,
    pub endurance: Charge,
}

impl PobCharges {
    pub fn is_empty(&self) -> bool {
        self.power.max == 0 && self.frenzy.max == 0 && self.endurance.max == 0
    }
}

/// Active buffs/curses listed in the `<Buffs combatList=... buffList=... curseList=.../>` element.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PobBuffs {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub combat: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub buffs: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub curses: Vec<String>,
}

impl PobBuffs {
    pub fn is_empty(&self) -> bool {
        self.combat.is_empty() && self.buffs.is_empty() && self.curses.is_empty()
    }
}

/// Config set values (boss profile, enemy resists, flask uptimes, custom mods).
/// Stored as strings — typed values would force JSON tagging that hurts LLM legibility.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PobConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub active_set_id: Option<String>,
    pub inputs: BTreeMap<String, String>,
    pub placeholders: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub custom_mods: Vec<String>,
}

impl PobConfig {
    pub fn is_empty(&self) -> bool {
        self.inputs.is_empty() && self.placeholders.is_empty() && self.custom_mods.is_empty()
    }
}

/// Passive tree metadata (counts only — actual node IDs stay in the URL).
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PobTree {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascend_class_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub class_internal_id: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ascendancy_internal_id: Option<String>,
    pub node_count: u32,
    pub mastery_count: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon_set_1_node_count: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub weapon_set_2_node_count: Option<u32>,
}

/// Defense breakdown lifted out of the flat `stats` map for easy navigation.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PobDefenses {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub life: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mana: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub energy_shield: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spirit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub armour: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evasion: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_dr: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spell_suppression: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub block_chance: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attack_dodge: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub spell_dodge: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fire_resist: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cold_resist: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightning_resist: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chaos_resist: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fire_max_hit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cold_max_hit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lightning_max_hit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub physical_max_hit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub chaos_max_hit: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub total_ehp: Option<f64>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NotesSection {
    pub heading: String,
    pub body: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobBuild {
    pub source_file: PathBuf,
    pub game: PoeVersion,
    pub class: String,
    pub ascendancy: Option<String>,
    pub level: Option<u32>,
    pub target_version: Option<String>,
    pub stats: BTreeMap<String, f64>,
    pub notes: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub notes_sections: Vec<NotesSection>,
    pub main_skill: Option<String>,
    pub skill_groups: Vec<PobSkillGroup>,
    pub items: Vec<PobItem>,
    pub passive_tree_url: Option<String>,
    #[serde(default, skip_serializing_if = "PobCharges::is_empty")]
    pub charges: PobCharges,
    #[serde(default, skip_serializing_if = "PobBuffs::is_empty")]
    pub buffs: PobBuffs,
    #[serde(default, skip_serializing_if = "PobConfig::is_empty")]
    pub config: PobConfig,
    #[serde(default)]
    pub tree: PobTree,
    #[serde(default)]
    pub defenses: PobDefenses,
    /// Slot name → item id mapping from the active `<ItemSet>`. Examples of
    /// keys : `Helmet`, `Body Armour`, `Ring 1`, `Weapon 1 Swap`, `Charm 1`,
    /// `Flask 2`. Empty when items haven't been laid out yet.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub slot_map: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "PoePantheon::is_empty")]
    pub pantheon: PoePantheon,
    /// PoE1 tattoo overrides on passive nodes (Karui tattoos).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tattoos: Vec<PobTattoo>,
    /// Full list of allocated passive node ids (the `nodes="..."` attr).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allocated_nodes: Vec<u32>,
    /// Mastery effects chosen, exact format depending on the game.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mastery_picks: Vec<MasteryPick>,
    /// Jewel items placed in tree sockets.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub jewel_placements: Vec<JewelPlacement>,
    /// Active spectres (PoE1 raise-spectre minions). One element per spectre.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub spectres: Vec<String>,
    /// pobb.in publish URL if present. **Never holds account/character hashes.**
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub import_link: Option<String>,
}

impl PobBuild {
    pub fn summary_line(&self) -> String {
        let asc = self
            .ascendancy
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("—");
        let lvl = self
            .level
            .map(|l| l.to_string())
            .unwrap_or_else(|| "?".to_string());
        format!(
            "[{}] {} / {} · lvl {}",
            self.game.label(),
            self.class,
            asc,
            lvl
        )
    }

    pub fn stat(&self, key: &str) -> Option<f64> {
        self.stats.get(key).copied()
    }

    pub fn life(&self) -> Option<f64> {
        self.stat("Life")
    }
    pub fn mana(&self) -> Option<f64> {
        self.stat("Mana")
    }
    pub fn energy_shield(&self) -> Option<f64> {
        self.stat("EnergyShield")
    }
    pub fn ehp(&self) -> Option<f64> {
        self.stat("TotalEHP")
    }
    pub fn dps(&self) -> Option<f64> {
        self.stat("CombinedDPS").or_else(|| self.stat("TotalDPS"))
    }

    pub fn resistances(&self) -> [(&'static str, Option<f64>); 4] {
        [
            ("fire", self.stat("FireResist")),
            ("cold", self.stat("ColdResist")),
            ("lightning", self.stat("LightningResist")),
            ("chaos", self.stat("ChaosResist")),
        ]
    }
}

/// Lightweight summary used by the build picker before full parsing.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobBuildSummary {
    pub path: PathBuf,
    pub game: PoeVersion,
    pub class: String,
    pub ascendancy: Option<String>,
    pub level: Option<u32>,
    pub main_skill_hint: Option<String>,
    #[serde(skip)]
    pub mtime: Option<SystemTime>,
}

impl PobBuildSummary {
    pub fn header(&self) -> String {
        let asc = self
            .ascendancy
            .as_deref()
            .filter(|s| !s.is_empty())
            .unwrap_or("—");
        let lvl = self
            .level
            .map(|l| l.to_string())
            .unwrap_or_else(|| "?".to_string());
        format!(
            "[{}] {} / {} · lvl {}",
            self.game.label(),
            self.class,
            asc,
            lvl
        )
    }
}
