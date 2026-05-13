//! User-supplied API key persistence.
//!
//! Stores keys at `~/.bestel/runtime/keys.json` as plaintext JSON, mirroring
//! the same on-disk pattern as `model.json` and `claude-mcp.json`. Keys are
//! merged into the process environment at boot via [`apply_to_env`] so the
//! existing env-var consumers (`AnthropicClient::from_env`, etc.) keep
//! working unchanged.
//!
//! The set of saveable env vars is whitelisted in [`ALLOWED_KEYS`]. Anything
//! outside the list is rejected — we never want a UI bug to write `PATH`
//! or `HOME` into the user's process environment.
//!
//! Security note: keys are stored as plaintext on disk. The UI surfaces this
//! explicitly via the `ApiKeyField` disclaimer; users who want OS-level
//! encryption should use system env vars instead.
//!
//! Live updates: `set_api_key` / `delete_api_key` call `std::env::set_var` /
//! `remove_var` so the running process picks up the change without restart.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use anyhow::{anyhow, Context, Result};

/// Whitelist of env vars the UI is allowed to save.
pub const ALLOWED_KEYS: &[&str] = &["ANTHROPIC_API_KEY", "DEEPSEEK_API_KEY"];

/// `~/.bestel/runtime/keys.json` — `None` if the home directory cannot be
/// resolved (rare; treated as "no persistence available").
pub fn keys_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".bestel").join("runtime").join("keys.json"))
}

/// Reads the keys file. Missing file or parse failures return an empty map —
/// boot must never fail because the user hasn't created the file yet.
pub fn load_keys() -> BTreeMap<String, String> {
    let Some(path) = keys_path() else {
        return BTreeMap::new();
    };
    load_keys_at(&path)
}

/// Same as [`load_keys`] but reads from an explicit path. Used by tests.
fn load_keys_at(path: &Path) -> BTreeMap<String, String> {
    let raw = match std::fs::read_to_string(path) {
        Ok(s) => s,
        Err(_) => return BTreeMap::new(),
    };
    serde_json::from_str(&raw).unwrap_or_default()
}

/// Persists `value` under `env_name`. Validates against [`ALLOWED_KEYS`],
/// trims whitespace, rejects empty / overly-short values, and writes the
/// whole file atomically (via tempfile rename).
pub fn save_key(env_name: &str, value: &str) -> Result<()> {
    let path = keys_path().ok_or_else(|| anyhow!("no home dir"))?;
    save_key_at(&path, env_name, value)
}

fn save_key_at(path: &Path, env_name: &str, value: &str) -> Result<()> {
    if !ALLOWED_KEYS.contains(&env_name) {
        return Err(anyhow!(
            "env var '{env_name}' is not on Bestel's allowlist: {:?}",
            ALLOWED_KEYS
        ));
    }
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(anyhow!("API key value is empty"));
    }
    if trimmed.len() < 10 {
        return Err(anyhow!("API key value is suspiciously short"));
    }
    let mut keys = load_keys_at(path);
    keys.insert(env_name.to_string(), trimmed.to_string());
    write_keys_at(path, &keys)
}

/// Removes `env_name` from the persisted map. No-op if not present.
pub fn delete_key(env_name: &str) -> Result<()> {
    let path = keys_path().ok_or_else(|| anyhow!("no home dir"))?;
    delete_key_at(&path, env_name)
}

fn delete_key_at(path: &Path, env_name: &str) -> Result<()> {
    let mut keys = load_keys_at(path);
    keys.remove(env_name);
    write_keys_at(path, &keys)
}

/// Pushes every persisted key into the current process's environment, so
/// downstream `std::env::var(...)` calls see the user-supplied values. Does
/// NOT overwrite values already set in the OS environment — env wins, since
/// the user explicitly set it externally and probably wants it that way.
pub fn apply_to_env() {
    for (k, v) in load_keys() {
        if std::env::var(&k).is_err() {
            std::env::set_var(&k, &v);
        }
    }
}

/// Reveals the last-4 of a key as `prefix...suffix` (e.g. `sk-ant-...4f2c`)
/// for the picker UI. Returns `None` when the value is too short to mask
/// without leaking too much.
pub fn mask_preview(value: &str) -> Option<String> {
    if value.len() < 12 {
        return None;
    }
    let prefix: String = value.chars().take(7).collect();
    let suffix: String = value.chars().skip(value.chars().count() - 4).collect();
    Some(format!("{prefix}...{suffix}"))
}

fn write_keys_at(path: &Path, keys: &BTreeMap<String, String>) -> Result<()> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let body = serde_json::to_vec_pretty(keys).context("serialize keys")?;
    let tmp = path.with_extension("json.tmp");
    std::fs::write(&tmp, body).with_context(|| format!("write {}", tmp.display()))?;
    std::fs::rename(&tmp, path)
        .with_context(|| format!("rename {} -> {}", tmp.display(), path.display()))?;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn tmp_keys_path() -> PathBuf {
        let mut p = std::env::temp_dir();
        let nanos = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        p.push(format!(
            "bestel-test-keys-{}-{}.json",
            std::process::id(),
            nanos
        ));
        let _ = std::fs::remove_file(&p);
        p
    }

    #[test]
    fn roundtrip_save_then_load() {
        let path = tmp_keys_path();
        save_key_at(&path, "ANTHROPIC_API_KEY", "sk-ant-test-0000000000").unwrap();
        let keys = load_keys_at(&path);
        assert_eq!(
            keys.get("ANTHROPIC_API_KEY").map(String::as_str),
            Some("sk-ant-test-0000000000")
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn delete_removes_key() {
        let path = tmp_keys_path();
        save_key_at(&path, "DEEPSEEK_API_KEY", "sk-deepseek-test-1111").unwrap();
        delete_key_at(&path, "DEEPSEEK_API_KEY").unwrap();
        assert!(!load_keys_at(&path).contains_key("DEEPSEEK_API_KEY"));
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn rejects_unallowed_key() {
        let path = tmp_keys_path();
        let res = save_key_at(&path, "PATH", "anything-malicious-value");
        assert!(res.is_err());
        assert!(res.unwrap_err().to_string().contains("allowlist"));
    }

    #[test]
    fn rejects_empty_value() {
        let path = tmp_keys_path();
        assert!(save_key_at(&path, "ANTHROPIC_API_KEY", "   ").is_err());
    }

    #[test]
    fn rejects_short_value() {
        let path = tmp_keys_path();
        assert!(save_key_at(&path, "ANTHROPIC_API_KEY", "abc").is_err());
    }

    #[test]
    fn missing_file_returns_empty_map() {
        let path = tmp_keys_path();
        assert!(load_keys_at(&path).is_empty());
    }

    #[test]
    fn save_overwrites_existing_value() {
        let path = tmp_keys_path();
        save_key_at(&path, "ANTHROPIC_API_KEY", "sk-ant-old-0000000000").unwrap();
        save_key_at(&path, "ANTHROPIC_API_KEY", "sk-ant-new-1111111111").unwrap();
        assert_eq!(
            load_keys_at(&path)
                .get("ANTHROPIC_API_KEY")
                .map(String::as_str),
            Some("sk-ant-new-1111111111")
        );
        let _ = std::fs::remove_file(&path);
    }

    #[test]
    fn mask_preview_short_returns_none() {
        assert_eq!(mask_preview("short"), None);
    }

    #[test]
    fn mask_preview_long_keeps_head_and_tail() {
        let p = mask_preview("sk-ant-1234567890abcdef").unwrap();
        assert!(p.starts_with("sk-ant-"));
        assert!(p.ends_with("cdef"));
        assert!(p.contains("..."));
    }
}
