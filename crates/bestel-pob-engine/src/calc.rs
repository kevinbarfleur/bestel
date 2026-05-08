//! High-level `calc(...)` orchestration on top of the lifecycle module.
//!
//! Flow:
//! 1. Acquire the per-game slot mutex.
//! 2. Ensure a live LuaJIT process (lazy spawn / restart on death).
//! 3. Re-load the build XML if its hash changed since the last call.
//! 4. Apply Calcs config overrides via `set_config`.
//! 5. Optionally `set_main_selection` for non-default skill groups.
//! 6. Issue `get_stats` and return the canonical PoB output table.
//! 7. Always echo the effective Calcs config in the response.

use std::collections::BTreeMap;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::error::PobEngineError;
use crate::lifecycle::{Game, PobEngineHandle};
use crate::protocol::Cmd;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Category {
    Offence,
    Defence,
    Charges,
    Reservation,
    Ailments,
    All,
}

impl Category {
    pub fn as_str(self) -> &'static str {
        match self {
            Category::Offence => "offence",
            Category::Defence => "defence",
            Category::Charges => "charges",
            Category::Reservation => "reservation",
            Category::Ailments => "ailments",
            Category::All => "all",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "offence" => Some(Category::Offence),
            "defence" => Some(Category::Defence),
            "charges" => Some(Category::Charges),
            "reservation" => Some(Category::Reservation),
            "ailments" => Some(Category::Ailments),
            "all" => Some(Category::All),
            _ => None,
        }
    }
}

/// Subset of PoB Calcs config that the agent can toggle through `pob_calc`.
/// Mirrors the keys exposed by the forked ianderse harness.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct EngineCalcs {
    pub enemy_is_boss: Option<bool>,
    pub use_power_charges: Option<bool>,
    pub use_frenzy_charges: Option<bool>,
    pub use_endurance_charges: Option<bool>,
    pub force_buff_onslaught: Option<bool>,
    pub multiplier_impale_stacks: Option<i32>,
    pub use_flask1: Option<bool>,
    pub use_flask2: Option<bool>,
    pub use_flask3: Option<bool>,
    pub use_flask4: Option<bool>,
    pub use_flask5: Option<bool>,
}

impl EngineCalcs {
    fn into_payload(&self) -> BTreeMap<String, Value> {
        let mut m = BTreeMap::new();
        if let Some(v) = self.enemy_is_boss {
            m.insert("enemyIsBoss".into(), Value::Bool(v));
        }
        if let Some(v) = self.use_power_charges {
            m.insert("usePowerCharges".into(), Value::Bool(v));
        }
        if let Some(v) = self.use_frenzy_charges {
            m.insert("useFrenzyCharges".into(), Value::Bool(v));
        }
        if let Some(v) = self.use_endurance_charges {
            m.insert("useEnduranceCharges".into(), Value::Bool(v));
        }
        if let Some(v) = self.force_buff_onslaught {
            m.insert("forceBuffOnslaught".into(), Value::Bool(v));
        }
        if let Some(v) = self.multiplier_impale_stacks {
            m.insert("multiplierImpaleStacks".into(), Value::Number(v.into()));
        }
        for (i, slot) in [
            self.use_flask1,
            self.use_flask2,
            self.use_flask3,
            self.use_flask4,
            self.use_flask5,
        ]
        .iter()
        .enumerate()
        {
            if let Some(v) = slot {
                m.insert(format!("useFlask{}", i + 1), Value::Bool(*v));
            }
        }
        m
    }
}

#[derive(Debug, Clone)]
pub struct CalcRequest {
    pub game: Game,
    pub build_xml: String,
    pub category: Category,
    pub skill_index: Option<u32>,
    pub calcs: EngineCalcs,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalcResponse {
    pub stats: Value,
    pub calcs: Value,
    /// Identifies the skill group the engine actually used to produce
    /// `stats`. Mismatch with the build's main_skill is the canonical
    /// signal that the calc is unreliable.
    #[serde(default)]
    pub active_skill: Value,
    #[serde(default)]
    pub warnings: Vec<String>,
}

impl PobEngineHandle {
    pub async fn calc(&self, req: CalcRequest) -> Result<CalcResponse, PobEngineError> {
        let slot_arc = self.slot(req.game);
        let mut slot = slot_arc.lock().await;
        let proc = self.ensure_alive(req.game, &mut slot).await?;

        let mut hasher = DefaultHasher::new();
        req.build_xml.hash(&mut hasher);
        let xml_hash = hasher.finish();

        if proc.loaded_xml_hash != Some(xml_hash) {
            let load = self
                .send_on(
                    proc,
                    &Cmd::LoadBuildXml {
                        xml: req.build_xml.clone(),
                    },
                )
                .await?;
            if !load.ok {
                let msg = load.error.unwrap_or_else(|| "load_build_xml failed".into());
                return Err(PobEngineError::BuildLoadFailed(msg));
            }
            proc.loaded_xml_hash = Some(xml_hash);
        }

        let cfg = req.calcs.into_payload();
        if !cfg.is_empty() {
            let reply = self
                .send_on(proc, &Cmd::SetConfig { config: cfg })
                .await?;
            if !reply.ok {
                let msg = reply
                    .error
                    .unwrap_or_else(|| "set_config rejected".into());
                return Err(PobEngineError::ConfigRejected(msg));
            }
        }

        if let Some(idx) = req.skill_index {
            let _ = self
                .send_on(
                    proc,
                    &Cmd::SetMainSelection {
                        main_socket_group: Some(idx),
                        main_active_skill: None,
                        skill_part: None,
                    },
                )
                .await?;
        }

        let stats_reply = self
            .send_on(
                proc,
                &Cmd::GetStats {
                    category: req.category.as_str().into(),
                },
            )
            .await?;
        let stats = stats_reply
            .stats
            .ok_or_else(|| PobEngineError::ProtocolBroken("missing stats payload".into()))?;
        let active_skill = stats_reply.active_skill.unwrap_or(Value::Null);

        let cfg_reply = self.send_on(proc, &Cmd::GetConfig).await?;
        let calcs_echo = cfg_reply.config.unwrap_or(Value::Null);

        let mut warnings = Vec::new();
        if active_skill.is_null() {
            warnings.push(
                "engine did not return active_skill metadata — numbers may be unreliable"
                    .to_string(),
            );
        } else if let Some(label) = active_skill.get("active_skill_label") {
            if label.is_null() {
                warnings.push(
                    "active skill group has no label — verify the build's main skill is selected"
                        .to_string(),
                );
            }
        }

        Ok(CalcResponse {
            stats,
            calcs: calcs_echo,
            active_skill,
            warnings,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn category_round_trip() {
        for cat in [
            Category::Offence,
            Category::Defence,
            Category::Charges,
            Category::Reservation,
            Category::Ailments,
            Category::All,
        ] {
            assert_eq!(Category::parse(cat.as_str()), Some(cat));
        }
    }

    #[test]
    fn empty_calcs_is_empty_payload() {
        assert!(EngineCalcs::default().into_payload().is_empty());
    }

    #[test]
    fn calcs_payload_includes_set_keys() {
        let calcs = EngineCalcs {
            enemy_is_boss: Some(true),
            multiplier_impale_stacks: Some(5),
            use_flask3: Some(true),
            ..Default::default()
        };
        let payload = calcs.into_payload();
        assert_eq!(payload.get("enemyIsBoss"), Some(&Value::Bool(true)));
        assert_eq!(
            payload.get("multiplierImpaleStacks"),
            Some(&Value::Number(5.into()))
        );
        assert_eq!(payload.get("useFlask3"), Some(&Value::Bool(true)));
        assert_eq!(payload.len(), 3);
    }
}
