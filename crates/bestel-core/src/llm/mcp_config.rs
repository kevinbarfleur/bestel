//! Helpers to wire CLI providers to Bestel's MCP server.
//!
//! Each provider has its own config format. We isolate user state by writing
//! per-Bestel configs under `~/.bestel/runtime/`, never touching the user's
//! global Codex / Claude / Gemini configs.
//!
//! Opt-out: `BESTEL_DISABLE_MCP=1` returns `None` from each helper, which
//! makes the spawn code skip MCP injection — useful for debugging the
//! pre-Phase-4 behaviour.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};

/// Strip MCP server-namespace prefixes that some hosts add to tool names so
/// the display label stays readable. Examples:
///   `bestel.wiki_search`        → `wiki_search`
///   `mcp__bestel__wiki_search`  → `wiki_search`
///   `bestel/wiki_search`        → `wiki_search`
pub fn strip_mcp_prefix(name: &str) -> &str {
    for prefix in ["mcp__bestel__", "bestel.", "bestel/", "bestel:", "bestel-"] {
        if let Some(rest) = name.strip_prefix(prefix) {
            return rest;
        }
    }
    name
}

pub fn mcp_disabled() -> bool {
    std::env::var("BESTEL_DISABLE_MCP")
        .map(|v| !v.is_empty() && v != "0" && v.to_lowercase() != "false")
        .unwrap_or(false)
}

pub fn current_exe_for_subprocess() -> Result<PathBuf> {
    std::env::current_exe().context("std::env::current_exe failed")
}

/// Forward slashes — TOML and JSON tolerate them on Windows and they sidestep
/// the backslash-escaping nightmare in inline `-c` flags.
pub fn exe_path_str() -> Result<String> {
    let p = current_exe_for_subprocess()?;
    Ok(p.to_string_lossy().replace('\\', "/"))
}

/// Ensure Codex CLI's persistent config (`~/.codex/config.toml`) registers
/// `bestel-mcp` as an MCP server. Idempotent: only writes if our marker
/// block is missing or points to a different exe path. Returns `Ok(true)`
/// when MCP was successfully wired (either already present or just added).
///
/// We write to the user's real config file rather than passing inline
/// `-c mcp_servers.bestel.command="..."` overrides because cmd.exe (which
/// Windows runs implicitly under any `.cmd` shim) re-parses argv and
/// strips inner double quotes, which corrupts the TOML value. The config
/// file path bypasses all quoting layers.
///
/// User can remove our block manually any time, or set
/// `BESTEL_DISABLE_MCP=1` to skip the write.
pub async fn ensure_codex_mcp_registered() -> Result<bool> {
    if mcp_disabled() {
        return Ok(false);
    }
    let exe = exe_path_str()?;
    let codex_home = std::env::var("CODEX_HOME")
        .ok()
        .map(PathBuf::from)
        .or_else(|| dirs::home_dir().map(|h| h.join(".codex")))
        .ok_or_else(|| anyhow!("could not determine codex home"))?;
    tokio::fs::create_dir_all(&codex_home).await.ok();
    let cfg_path = codex_home.join("config.toml");
    let existing = tokio::fs::read_to_string(&cfg_path)
        .await
        .unwrap_or_default();

    let begin = "# >>> bestel-mcp managed block >>>";
    let end = "# <<< bestel-mcp managed block <<<";
    let block = format!(
        "{begin}\n[mcp_servers.bestel]\ncommand = \"{exe}\"\nargs = [\"mcp-serve\"]\n{end}\n",
    );

    if let (Some(b_idx), Some(e_idx)) = (existing.find(begin), existing.find(end)) {
        // Block exists — replace it only if the path changed.
        let current = &existing[b_idx..e_idx + end.len()];
        let expected_inner = format!("[mcp_servers.bestel]\ncommand = \"{exe}\"");
        if current.contains(&expected_inner) {
            return Ok(true);
        }
        let mut new_cfg = String::new();
        new_cfg.push_str(&existing[..b_idx]);
        new_cfg.push_str(&block);
        new_cfg.push_str(&existing[e_idx + end.len()..]);
        tokio::fs::write(&cfg_path, new_cfg)
            .await
            .with_context(|| format!("update {}", cfg_path.display()))?;
        return Ok(true);
    }

    // No block yet — append.
    let mut new_cfg = existing;
    if !new_cfg.is_empty() && !new_cfg.ends_with('\n') {
        new_cfg.push('\n');
    }
    new_cfg.push('\n');
    new_cfg.push_str(&block);
    tokio::fs::write(&cfg_path, new_cfg)
        .await
        .with_context(|| format!("write {}", cfg_path.display()))?;
    Ok(true)
}

/// Writes a Claude-CLI-compatible MCP config JSON to a stable runtime path
/// and returns it. Caller passes it via `--mcp-config <path>`.
pub async fn claude_mcp_config_path() -> Result<Option<PathBuf>> {
    if mcp_disabled() {
        return Ok(None);
    }
    let exe = exe_path_str()?;
    let runtime_dir = dirs::home_dir()
        .ok_or_else(|| anyhow!("no home dir"))?
        .join(".bestel")
        .join("runtime");
    tokio::fs::create_dir_all(&runtime_dir).await.ok();
    let path = runtime_dir.join("claude-mcp.json");
    let body = serde_json::json!({
        "mcpServers": {
            "bestel": {
                "command": exe,
                "args": ["mcp-serve"]
            }
        }
    });
    tokio::fs::write(&path, serde_json::to_vec_pretty(&body)?)
        .await
        .with_context(|| format!("write {}", path.display()))?;
    Ok(Some(path))
}

/// Writes a Gemini-CLI-compatible MCP server settings file. Gemini CLI reads
/// `~/.gemini/settings.json` with an `mcpServers` block; rather than mutate
/// the user's settings, we spawn Gemini with `GEMINI_HOME` set to a Bestel
/// runtime dir that has only this MCP server registered.
///
/// Returns the runtime dir path so the spawn site can set `GEMINI_HOME` to it.
pub async fn gemini_runtime_home() -> Result<Option<PathBuf>> {
    if mcp_disabled() {
        return Ok(None);
    }
    let exe = exe_path_str()?;
    let home = dirs::home_dir()
        .ok_or_else(|| anyhow!("no home dir"))?
        .join(".bestel")
        .join("runtime")
        .join("gemini");
    tokio::fs::create_dir_all(&home).await.ok();
    let settings = home.join("settings.json");
    let body = serde_json::json!({
        "mcpServers": {
            "bestel": {
                "command": exe,
                "args": ["mcp-serve"]
            }
        }
    });
    tokio::fs::write(&settings, serde_json::to_vec_pretty(&body)?)
        .await
        .with_context(|| format!("write {}", settings.display()))?;
    Ok(Some(home))
}
