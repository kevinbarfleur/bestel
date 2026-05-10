//! User-tunable runtime settings persistence.
//!
//! Stores app-level toggles at `~/.bestel/runtime/settings.json` as plaintext
//! JSON, mirroring the on-disk pattern used by `keys.json`, `model.json`, and
//! `claude-mcp.json`. The schema is versioned so we can evolve it without
//! breaking older installs — unknown fields fall back to defaults.
//!
//! Currently exposes a single toggle:
//!
//! - `verify_enabled` — on by default; gates the Chain-of-Verification
//!   factored verifier (`llm/verifier.rs`). When false, the post-draft
//!   verification pipeline short-circuits and the draft ships as-is.
//!
//! Failsafe: a missing or corrupt file always returns `Settings::default()`
//! (verify on). Boot must never fail because the user hasn't created the
//! file yet, and a half-written file from a crash should not lock the user
//! out of verification.

use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

/// Persisted settings shape. Add new fields with `#[serde(default)]` so
/// older `settings.json` files keep parsing.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct Settings {
    #[serde(default = "default_schema_version")]
    pub schema_version: u32,
    #[serde(default = "default_verify_enabled")]
    pub verify_enabled: bool,
}

fn default_schema_version() -> u32 {
    1
}

fn default_verify_enabled() -> bool {
    true
}

impl Default for Settings {
    fn default() -> Self {
        Self {
            schema_version: default_schema_version(),
            verify_enabled: default_verify_enabled(),
        }
    }
}

/// `~/.bestel/runtime/settings.json` — `None` if the home directory cannot
/// be resolved (rare; treated as "no persistence available").
pub fn settings_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".bestel").join("runtime").join("settings.json"))
}

/// Reads the settings file. Missing file or parse failures return
/// `Settings::default()` — boot must never fail and a corrupt file must
/// not lock features off.
pub fn load() -> Settings {
    let Some(path) = settings_path() else {
        return Settings::default();
    };
    load_at(&path)
}

fn load_at(path: &Path) -> Settings {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return Settings::default(),
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

/// Persists the full settings object, atomically (via tempfile rename).
pub fn save(settings: &Settings) -> Result<()> {
    let path = settings_path().ok_or_else(|| anyhow!("no home dir"))?;
    save_at(&path, settings)
}

fn save_at(path: &Path, settings: &Settings) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let body = serde_json::to_vec_pretty(settings).context("serialize settings")?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, body).with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

/// Cheap accessor for hot-path checks (`run_verifier_pass` calls this on
/// every turn). Reads the file fresh each time so a UI toggle takes effect
/// without process restart. The file is small (< 100 bytes); the I/O cost
/// is negligible vs the LLM call that follows.
pub fn is_verify_enabled() -> bool {
    load().verify_enabled
}

/// Updates the `verify_enabled` flag and persists. The next call to
/// `is_verify_enabled` picks it up immediately.
pub fn set_verify_enabled(enabled: bool) -> Result<()> {
    let mut current = load();
    current.verify_enabled = enabled;
    save(&current)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_settings_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!(
            "bestel-test-settings-{}-{}.json",
            std::process::id(),
            nanos
        ));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn default_is_verify_on() {
        let s = Settings::default();
        assert!(s.verify_enabled);
        assert_eq!(s.schema_version, 1);
    }

    #[test]
    fn missing_file_returns_default() {
        let path = tmp_settings_path();
        let loaded = load_at(&path);
        assert_eq!(loaded, Settings::default());
    }

    #[test]
    fn roundtrip_save_then_load() {
        let path = tmp_settings_path();
        let s = Settings {
            schema_version: 1,
            verify_enabled: false,
        };
        save_at(&path, &s).unwrap();
        let loaded = load_at(&path);
        assert_eq!(loaded, s);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn corrupt_file_returns_default() {
        let path = tmp_settings_path();
        std::fs::write(&path, "{ not valid json").unwrap();
        let loaded = load_at(&path);
        assert_eq!(loaded, Settings::default());
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn missing_field_uses_default() {
        let path = tmp_settings_path();
        // Write a file missing `verify_enabled`; serde defaults must kick in.
        std::fs::write(&path, r#"{"schema_version": 1}"#).unwrap();
        let loaded = load_at(&path);
        assert!(loaded.verify_enabled);
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn unknown_field_is_ignored() {
        let path = tmp_settings_path();
        std::fs::write(
            &path,
            r#"{"schema_version": 1, "verify_enabled": false, "future_flag": 42}"#,
        )
        .unwrap();
        let loaded = load_at(&path);
        assert!(!loaded.verify_enabled);
        let _ = std::fs::remove_file(&path);
    }
}
