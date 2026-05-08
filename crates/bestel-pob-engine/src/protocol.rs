//! ndJSON wire protocol between the Rust supervisor and the forked
//! ianderse `api-stdio-bestel/api-stdio.lua` harness.
//!
//! Each command is a single-line JSON object containing an `action` field
//! plus action-specific keys. Each reply is a single-line JSON object
//! containing `ok` (bool), an optional `error` string, and action-specific
//! result keys. Newline-terminated.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize)]
#[serde(tag = "action", rename_all = "snake_case")]
pub enum Cmd {
    Ping,
    Version,
    Quit,
    LoadBuildXml { xml: String },
    GetStats { category: String },
    SetConfig {
        #[serde(flatten)]
        config: BTreeMap<String, Value>,
    },
    GetConfig,
    SetMainSelection {
        #[serde(rename = "mainSocketGroup", skip_serializing_if = "Option::is_none")]
        main_socket_group: Option<u32>,
        #[serde(rename = "mainActiveSkill", skip_serializing_if = "Option::is_none")]
        main_active_skill: Option<u32>,
        #[serde(rename = "skillPart", skip_serializing_if = "Option::is_none")]
        skill_part: Option<u32>,
    },
}

#[derive(Debug, Clone, Deserialize)]
pub struct Reply {
    pub ok: bool,
    #[serde(default)]
    pub error: Option<String>,
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub game: Option<String>,
    #[serde(default)]
    pub stats: Option<Value>,
    #[serde(default)]
    pub config: Option<Value>,
    /// Active-skill metadata: which skill group the engine actually used.
    /// Always present in `get_stats` replies from the forked harness.
    /// The agent must verify this matches the build's main_skill before
    /// quoting any DPS / EHP number — silent wrong-skill fallback was a
    /// real bug observed in the 2026-05-08 battery.
    #[serde(default)]
    pub active_skill: Option<Value>,
}

pub fn encode(cmd: &Cmd) -> Result<String, serde_json::Error> {
    let mut buf = serde_json::to_string(cmd)?;
    buf.push('\n');
    Ok(buf)
}

pub fn decode(line: &str) -> Result<Reply, serde_json::Error> {
    serde_json::from_str(line.trim())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ping_serialises_to_single_line() {
        let s = encode(&Cmd::Ping).unwrap();
        assert_eq!(s, "{\"action\":\"ping\"}\n");
    }

    #[test]
    fn load_build_xml_includes_xml_field() {
        let s = encode(&Cmd::LoadBuildXml {
            xml: "<PathOfBuilding/>".into(),
        })
        .unwrap();
        assert!(s.contains("\"action\":\"load_build_xml\""));
        assert!(s.contains("\"xml\":\"<PathOfBuilding/>\""));
    }

    #[test]
    fn set_config_flattens_keys() {
        let mut config = BTreeMap::new();
        config.insert("enemyIsBoss".to_string(), Value::Bool(true));
        config.insert("enemyLevel".to_string(), Value::Number(84.into()));
        let s = encode(&Cmd::SetConfig { config }).unwrap();
        assert!(s.contains("\"action\":\"set_config\""));
        assert!(s.contains("\"enemyIsBoss\":true"));
        assert!(s.contains("\"enemyLevel\":84"));
    }

    #[test]
    fn ok_reply_decodes() {
        let r = decode("{\"ok\":true,\"version\":\"v0.15.0\"}").unwrap();
        assert!(r.ok);
        assert_eq!(r.version.as_deref(), Some("v0.15.0"));
    }

    #[test]
    fn error_reply_decodes() {
        let r = decode("{\"ok\":false,\"error\":\"bad xml\"}").unwrap();
        assert!(!r.ok);
        assert_eq!(r.error.as_deref(), Some("bad xml"));
    }
}
