use std::sync::{Arc, RwLock};

use serde_json::json;

use crate::pob::PobBuild;

pub const GET_ACTIVE_BUILD: &str = "get_active_build";

#[derive(Clone, Default)]
pub struct BuildContext {
    inner: Arc<RwLock<Option<PobBuild>>>,
}

impl BuildContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn set(&self, build: PobBuild) {
        if let Ok(mut g) = self.inner.write() {
            *g = Some(build);
        }
    }

    pub fn get(&self) -> Option<PobBuild> {
        self.inner.read().ok().and_then(|g| g.clone())
    }

    pub fn render_tool_result(&self) -> String {
        match self.get() {
            None => json!({
                "status": "no_build",
                "message": "No Path of Building build is currently loaded. Ask the exile to save a build in PoB so the chronicler can read it."
            })
            .to_string(),
            Some(b) => {
                let mut summary = serde_json::Map::new();
                summary.insert("game".into(), json!(b.game.label()));
                summary.insert("class".into(), json!(b.class));
                summary.insert("ascendancy".into(), json!(b.ascendancy));
                summary.insert("level".into(), json!(b.level));
                summary.insert("target_version".into(), json!(b.target_version));
                summary.insert("notes".into(), json!(b.notes));
                summary.insert("main_skill".into(), json!(b.main_skill));
                summary.insert("source_file".into(), json!(b.source_file.display().to_string()));

                let key_stats = [
                    "Life",
                    "Mana",
                    "EnergyShield",
                    "TotalEHP",
                    "CombinedDPS",
                    "TotalDPS",
                    "FullDPS",
                    "FireResist",
                    "ColdResist",
                    "LightningResist",
                    "ChaosResist",
                    "Armour",
                    "Evasion",
                    "PhysicalDamageReduction",
                    "EffectiveSpellSuppressionChance",
                    "EffectiveBlockChance",
                    "AttackDodgeChance",
                    "SpellDodgeChance",
                    "Str",
                    "Dex",
                    "Int",
                ];
                let mut stats = serde_json::Map::new();
                for k in key_stats {
                    if let Some(v) = b.stats.get(k) {
                        stats.insert(k.to_string(), json!(v));
                    }
                }
                summary.insert("stats".into(), serde_json::Value::Object(stats));

                let skill_groups: Vec<_> = b
                    .skill_groups
                    .iter()
                    .map(|g| {
                        json!({
                            "label": g.label,
                            "is_main": g.is_main,
                            "gems": g.gems.iter().map(|gem| json!({
                                "name": gem.name,
                                "level": gem.level,
                                "quality": gem.quality,
                                "enabled": gem.enabled,
                            })).collect::<Vec<_>>(),
                        })
                    })
                    .collect();
                summary.insert("skill_groups".into(), json!(skill_groups));

                let items: Vec<_> = b
                    .items
                    .iter()
                    .map(|it| {
                        json!({
                            "id": it.id,
                            "rarity": it.rarity,
                            "name": it.name,
                            "base": it.base,
                            "raw": truncate(&it.raw, 1500),
                        })
                    })
                    .collect();
                summary.insert("items".into(), json!(items));

                let value = serde_json::Value::Object(summary);
                let s = serde_json::to_string(&value).unwrap_or_default();
                truncate(&s, 60_000)
            }
        }
    }
}

fn truncate(s: &str, max: usize) -> String {
    if s.len() <= max {
        s.to_string()
    } else {
        let mut end = max;
        while !s.is_char_boundary(end) && end > 0 {
            end -= 1;
        }
        format!("{}…[truncated]", &s[..end])
    }
}

pub fn tool_schema() -> serde_json::Value {
    json!({
        "name": GET_ACTIVE_BUILD,
        "description": "Returns the currently loaded Path of Building build for this exile, including class, ascendancy, level, key stats (life, mana, ES, EHP, DPS, resistances), main skill, skill groups, items, and notes. Always call this tool before making any claim about the user's character.",
        "input_schema": {
            "type": "object",
            "properties": {},
            "additionalProperties": false
        }
    })
}
