//! Persistence of the attached debug port between CLI invocations.
//!
//! `bestel-driver attach --port N` stores `N` here so subsequent
//! `bestel-driver screenshot` / `eval` / etc. can reconnect without the
//! user passing `--port` every time. CDP WebSocket sessions themselves
//! are NOT persisted — every command spins up a fresh connection.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Session {
    pub port: u16,
    /// Wall-clock when the attach happened; helps the user spot stale sessions.
    pub attached_at: String,
}

pub fn session_path() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow!("no home dir"))?;
    Ok(home.join(".bestel").join("runtime").join("driver-session.json"))
}

pub fn save(session: &Session) -> Result<()> {
    let path = session_path()?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let body = serde_json::to_vec_pretty(session).context("serialize session")?;
    std::fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

pub fn load() -> Result<Session> {
    let path = session_path()?;
    let raw = std::fs::read_to_string(&path)
        .with_context(|| format!("no session — run `bestel-driver attach` first ({})", path.display()))?;
    serde_json::from_str(&raw).with_context(|| format!("parse {}", path.display()))
}

#[allow(dead_code)]
pub fn clear() -> Result<()> {
    let path = session_path()?;
    if path.exists() {
        std::fs::remove_file(&path).ok();
    }
    Ok(())
}
