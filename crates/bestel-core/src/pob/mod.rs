pub mod locator;
pub mod parser;
pub mod watcher;

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::path::PathBuf;

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobSkillGem {
    pub name: String,
    pub level: Option<u32>,
    pub quality: Option<u32>,
    pub enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PobSkillGroup {
    pub label: String,
    pub is_main: bool,
    pub gems: Vec<PobSkillGem>,
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
    pub main_skill: Option<String>,
    pub skill_groups: Vec<PobSkillGroup>,
    pub items: Vec<PobItem>,
    pub passive_tree_url: Option<String>,
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
