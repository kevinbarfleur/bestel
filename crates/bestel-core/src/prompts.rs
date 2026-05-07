//! User-editable prompts under `~/.bestel/prompts/`.
//!
//! Bestel ships three system prompts baked into the binary
//! (`BUNDLED_SYSTEM_PROMPT`, `BUNDLED_CORE_KNOWLEDGE`, `BUNDLED_LOCAL_ADDENDUM`).
//! On first launch these are seeded into `~/.bestel/prompts/` so the user can
//! tweak voice, hard rules, source allowlist, etc. without rebuilding. Every
//! chat turn re-reads from disk; any IO error falls back to the bundled
//! version so the agent stays usable even if the user wipes the folder.
//!
//! Per-provider and per-model overrides are appended to the base prompt when
//! the matching file exists:
//!
//! - `~/.bestel/prompts/providers/<provider>.md` — applies to every model of
//!   that provider (e.g. `providers/anthropic.md` for the Anthropic API,
//!   `providers/ollama.md` for any local Ollama model).
//! - `~/.bestel/prompts/models/<sanitized_model_id>.md` — narrowest scope,
//!   only fires for that exact model id.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Mutex;

use anyhow::{anyhow, Context, Result};
use serde::Serialize;

pub const BUNDLED_SYSTEM_PROMPT: &str = include_str!("../../../SYSTEM_PROMPT.md");
pub const BUNDLED_CORE_KNOWLEDGE: &str = include_str!("../CORE_KNOWLEDGE.md");
pub const BUNDLED_LOCAL_ADDENDUM: &str = include_str!("./llm/local_addendum.md");

const FILE_SYSTEM_PROMPT: &str = "SYSTEM_PROMPT.md";
const FILE_CORE_KNOWLEDGE: &str = "CORE_KNOWLEDGE.md";
const FILE_LOCAL_ADDENDUM: &str = "local_addendum.md";

/// The reference library — 17 numbered conceptual docs + 5 maxroll catalogues.
/// Bundled into the binary so the agent has a complete fallback even when the
/// user has wiped `~/.bestel/prompts/references/`. The agent fetches these on
/// demand via the `read_internal_reference` tool — they are NOT concatenated
/// into the system prompt.
pub const BUNDLED_REFERENCES: &[(&str, &str)] = &[
    ("00_README.md", include_str!("../../../docs/references/00_README.md")),
    ("01_source_policy.md", include_str!("../../../docs/references/01_source_policy.md")),
    ("02_arpg_foundations.md", include_str!("../../../docs/references/02_arpg_foundations.md")),
    ("03_ggg_design_philosophy.md", include_str!("../../../docs/references/03_ggg_design_philosophy.md")),
    ("04_game_model_poe1_poe2.md", include_str!("../../../docs/references/04_game_model_poe1_poe2.md")),
    ("05_build_reasoning_framework.md", include_str!("../../../docs/references/05_build_reasoning_framework.md")),
    ("06_character_stats_and_mechanics.md", include_str!("../../../docs/references/06_character_stats_and_mechanics.md")),
    ("07_offence_damage_scaling.md", include_str!("../../../docs/references/07_offence_damage_scaling.md")),
    ("08_defence_recovery_survivability.md", include_str!("../../../docs/references/08_defence_recovery_survivability.md")),
    ("09_itemisation_crafting.md", include_str!("../../../docs/references/09_itemisation_crafting.md")),
    ("10_skills_gems_passives_ascendancies.md", include_str!("../../../docs/references/10_skills_gems_passives_ascendancies.md")),
    ("11_endgame_economy_trade_leagues.md", include_str!("../../../docs/references/11_endgame_economy_trade_leagues.md")),
    ("12_vocabulary_glossary.md", include_str!("../../../docs/references/12_vocabulary_glossary.md")),
    ("13_retrieval_playbooks.md", include_str!("../../../docs/references/13_retrieval_playbooks.md")),
    ("14_validation_and_failure_modes.md", include_str!("../../../docs/references/14_validation_and_failure_modes.md")),
    ("15_source_registry.md", include_str!("../../../docs/references/15_source_registry.md")),
    ("16_build_methodology_and_creators.md", include_str!("../../../docs/references/16_build_methodology_and_creators.md")),
    ("maxroll/00_README.md", include_str!("../../../docs/references/maxroll/00_README.md")),
    ("maxroll/poe1_bosses.md", include_str!("../../../docs/references/maxroll/poe1_bosses.md")),
    ("maxroll/poe1_crafting.md", include_str!("../../../docs/references/maxroll/poe1_crafting.md")),
    ("maxroll/poe1_currency.md", include_str!("../../../docs/references/maxroll/poe1_currency.md")),
    ("maxroll/poe1_getting_started.md", include_str!("../../../docs/references/maxroll/poe1_getting_started.md")),
];

/// Per-path mutex so concurrent `prompts_write` calls (autosave races, etc.)
/// don't tear writes against each other. Granularity is the relative path
/// inside `prompts_dir()`.
fn write_lock(rel: &str) -> &'static Mutex<()> {
    use std::sync::OnceLock;
    static MAP: OnceLock<Mutex<HashMap<String, &'static Mutex<()>>>> = OnceLock::new();
    let map = MAP.get_or_init(|| Mutex::new(HashMap::new()));
    let mut guard = map.lock().expect("prompts write_lock poisoned");
    if let Some(m) = guard.get(rel) {
        return m;
    }
    let m: &'static Mutex<()> = Box::leak(Box::new(Mutex::new(())));
    guard.insert(rel.to_string(), m);
    m
}

/// `~/.bestel/prompts/`. Creates the directory if missing. Returns `None` if
/// the home directory cannot be resolved (extremely rare on Windows / Linux).
pub fn prompts_dir() -> Option<PathBuf> {
    let dir = dirs::home_dir()?.join(".bestel").join("prompts");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    Some(dir)
}

/// `~/.bestel/prompts/references/` — the reference library root. Subset of
/// `prompts_dir()` reserved for the agent's fetch-on-demand knowledge base.
pub fn references_dir() -> Option<PathBuf> {
    let dir = prompts_dir()?.join("references");
    if !dir.exists() {
        let _ = fs::create_dir_all(&dir);
    }
    Some(dir)
}

/// Validate a relative path against traversal / absolute / rooted segments
/// and join it onto a known-good root. Same rejection rules whether the root
/// is `prompts_dir()` or `references_dir()`.
fn safe_join_under(root: &Path, rel: &str) -> Result<PathBuf> {
    let p = PathBuf::from(rel);
    if p.is_absolute() {
        return Err(anyhow!("absolute paths are not allowed: {rel}"));
    }
    for comp in p.components() {
        match comp {
            std::path::Component::ParentDir => {
                return Err(anyhow!("path traversal rejected: {rel}"));
            }
            std::path::Component::Prefix(_) | std::path::Component::RootDir => {
                return Err(anyhow!("rooted paths are not allowed: {rel}"));
            }
            _ => {}
        }
    }
    Ok(root.join(&p))
}

/// Resolve a relative path under `prompts_dir()`. Returned path is not
/// canonicalized — callers that want to create a file should ensure the
/// parent exists.
fn safe_join(rel: &str) -> Result<PathBuf> {
    let root = prompts_dir().context("home directory not resolvable")?;
    safe_join_under(&root, rel)
}

/// Resolve a relative path under `references_dir()`. Same traversal rules.
fn safe_join_reference(rel: &str) -> Result<PathBuf> {
    let root = references_dir().context("home directory not resolvable")?;
    safe_join_under(&root, rel)
}

/// Lowercase + replace `/`, `:`, `\\` with `-` so model ids like
/// `anthropic/claude-haiku-4-5` map to a single safe filename.
pub fn sanitize_model_id(model_id: &str) -> String {
    let mut s = String::with_capacity(model_id.len());
    for ch in model_id.chars() {
        let lower = ch.to_ascii_lowercase();
        if matches!(lower, '/' | ':' | '\\') {
            s.push('-');
        } else {
            s.push(lower);
        }
    }
    s
}

/// Write the bundled defaults into `~/.bestel/prompts/` only when the
/// destination file does not exist. Idempotent — running twice has no
/// effect; never overwrites user edits. Also seeds the reference library.
pub fn seed_defaults_if_missing() -> Result<()> {
    let root = prompts_dir().context("home directory not resolvable")?;
    fs::create_dir_all(&root)?;
    fs::create_dir_all(root.join("providers"))?;
    fs::create_dir_all(root.join("models"))?;

    let bundled: [(&str, &str); 3] = [
        (FILE_SYSTEM_PROMPT, BUNDLED_SYSTEM_PROMPT),
        (FILE_CORE_KNOWLEDGE, BUNDLED_CORE_KNOWLEDGE),
        (FILE_LOCAL_ADDENDUM, BUNDLED_LOCAL_ADDENDUM),
    ];
    for (name, content) in bundled {
        let path = root.join(name);
        if !path.exists() {
            fs::write(&path, content)?;
        }
    }

    seed_references_if_missing()?;
    Ok(())
}

/// Write the bundled reference library into `~/.bestel/prompts/references/`
/// only when each destination file is missing. Creates the `references/`
/// and `references/maxroll/` subdirectories as needed.
pub fn seed_references_if_missing() -> Result<()> {
    let root = references_dir().context("home directory not resolvable")?;
    fs::create_dir_all(&root)?;
    fs::create_dir_all(root.join("maxroll"))?;
    for (rel, content) in BUNDLED_REFERENCES {
        let path = root.join(rel);
        if !path.exists() {
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, content)?;
        }
    }
    Ok(())
}

/// Read a reference file from `~/.bestel/prompts/references/<rel_path>`. Falls
/// back to the bundled content if the disk read fails. Path is validated to
/// reject traversal and absolute paths.
pub fn read_reference(rel_path: &str) -> Result<String> {
    let path = safe_join_reference(rel_path)?;
    if let Ok(s) = fs::read_to_string(&path) {
        return Ok(s);
    }
    for (k, v) in BUNDLED_REFERENCES {
        if *k == rel_path {
            return Ok((*v).to_string());
        }
    }
    Err(anyhow!("{rel_path} not found in reference library"))
}

/// Parse the YAML frontmatter (`description:` + `fetch_when:` keys) from a
/// markdown file's leading `---` block. Returns `None` if the file has no
/// frontmatter or the keys are absent.
pub fn parse_frontmatter(content: &str) -> Option<(String, String)> {
    let body = content.strip_prefix("---\n").or_else(|| content.strip_prefix("---\r\n"))?;
    let end = body
        .find("\n---\n")
        .or_else(|| body.find("\r\n---\r\n"))
        .or_else(|| body.find("\n---"))?;
    let yaml = &body[..end];
    let mut description = String::new();
    let mut fetch_when = String::new();
    for raw in yaml.lines() {
        let line = raw.trim();
        if let Some(rest) = line.strip_prefix("description:") {
            description = strip_yaml_quotes(rest.trim()).to_string();
        } else if let Some(rest) = line.strip_prefix("fetch_when:") {
            fetch_when = strip_yaml_quotes(rest.trim()).to_string();
        }
    }
    if description.is_empty() && fetch_when.is_empty() {
        None
    } else {
        Some((description, fetch_when))
    }
}

fn strip_yaml_quotes(s: &str) -> &str {
    let s = s.trim();
    if (s.starts_with('"') && s.ends_with('"')) || (s.starts_with('\'') && s.ends_with('\'')) {
        if s.len() >= 2 {
            return &s[1..s.len() - 1];
        }
    }
    s
}

/// Best-effort one-line description for a file: frontmatter `description:` if
/// present, else first non-blank, non-heading line under the H1, else empty.
pub fn extract_description(content: &str) -> String {
    if let Some((desc, _)) = parse_frontmatter(content) {
        if !desc.is_empty() {
            return desc;
        }
    }
    let mut after_h1 = false;
    for raw in content.lines() {
        let line = raw.trim();
        if line.starts_with("# ") {
            after_h1 = true;
            continue;
        }
        if after_h1 && !line.is_empty() && !line.starts_with('#') && !line.starts_with("---") {
            return line.chars().take(140).collect();
        }
    }
    String::new()
}

fn read_or_bundled(name: &str, bundled: &str) -> String {
    if let Ok(p) = safe_join(name) {
        if let Ok(content) = fs::read_to_string(&p) {
            return content;
        }
    }
    bundled.to_string()
}

/// Composed base system prompt = `SYSTEM_PROMPT.md` + separator +
/// `CORE_KNOWLEDGE.md`, both read from disk per turn with bundled fallback.
pub fn load_composed() -> String {
    let system = read_or_bundled(FILE_SYSTEM_PROMPT, BUNDLED_SYSTEM_PROMPT);
    let core = read_or_bundled(FILE_CORE_KNOWLEDGE, BUNDLED_CORE_KNOWLEDGE);
    format!("{system}\n\n---\n\n{core}")
}

/// Local-model addendum read from disk, with bundled fallback.
pub fn load_local_addendum() -> String {
    read_or_bundled(FILE_LOCAL_ADDENDUM, BUNDLED_LOCAL_ADDENDUM)
}

/// Provider-wide override: `~/.bestel/prompts/providers/<provider>.md`.
/// Returns `None` if the file doesn't exist, is unreadable, or is empty
/// (whitespace-only). The caller is responsible for appending it to the base
/// prompt with a `\n\n---\n\n` separator.
pub fn load_provider_override(provider: &str) -> Option<String> {
    let rel = format!("providers/{provider}.md");
    let path = safe_join(&rel).ok()?;
    let content = fs::read_to_string(&path).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    Some(content)
}

/// Per-model override: `~/.bestel/prompts/models/<sanitized>.md`. Same null
/// semantics as `load_provider_override`.
pub fn load_model_override(model_id: &str) -> Option<String> {
    let rel = format!("models/{}.md", sanitize_model_id(model_id));
    let path = safe_join(&rel).ok()?;
    let content = fs::read_to_string(&path).ok()?;
    if content.trim().is_empty() {
        return None;
    }
    Some(content)
}

/// Append provider + model overrides to a base system prompt. Each override
/// is separated by `\n\n---\n\n` so the LLM sees clear scope boundaries.
pub fn append_overrides(base: &str, provider: &str, model_id: &str) -> String {
    let mut out = String::with_capacity(base.len() + 1024);
    out.push_str(base);
    if let Some(po) = load_provider_override(provider) {
        out.push_str("\n\n---\n\n");
        out.push_str(&po);
    }
    if let Some(mo) = load_model_override(model_id) {
        out.push_str("\n\n---\n\n");
        out.push_str(&mo);
    }
    out
}

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum PromptKind {
    Shipped,
    Custom,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptFileMeta {
    /// Path relative to `prompts_dir()`. Always uses forward slashes.
    pub rel_path: String,
    pub kind: PromptKind,
    pub modified_vs_bundled: bool,
    pub byte_size: u64,
    pub line_count: u32,
    /// One-line purpose extracted from the file's YAML frontmatter
    /// (`description:` field) or the first non-blank line under the H1.
    pub description: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptGroup {
    pub label: String,
    /// Section header sentence that explains the group's role to the user
    /// editing prompts. Renders directly in the editor's tree.
    pub description: String,
    pub items: Vec<PromptFileMeta>,
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptTree {
    pub groups: Vec<PromptGroup>,
}

fn bundled_for(rel_path: &str) -> Option<&'static str> {
    match rel_path {
        FILE_SYSTEM_PROMPT => Some(BUNDLED_SYSTEM_PROMPT),
        FILE_CORE_KNOWLEDGE => Some(BUNDLED_CORE_KNOWLEDGE),
        FILE_LOCAL_ADDENDUM => Some(BUNDLED_LOCAL_ADDENDUM),
        _ => {
            for (k, v) in BUNDLED_REFERENCES {
                if *k == rel_path.trim_start_matches("references/") {
                    return Some(*v);
                }
            }
            None
        }
    }
}

fn meta_for(rel_path: &str, abs: &Path) -> PromptFileMeta {
    let bytes = fs::metadata(abs).map(|m| m.len()).unwrap_or(0);
    let content = fs::read_to_string(abs).unwrap_or_default();
    let line_count = if content.is_empty() {
        0
    } else {
        content.lines().count() as u32
    };
    let bundled = bundled_for(rel_path);
    let kind = if bundled.is_some() {
        PromptKind::Shipped
    } else {
        PromptKind::Custom
    };
    let modified_vs_bundled = match bundled {
        Some(b) => b != content,
        None => false,
    };
    let description = extract_description(&content);
    PromptFileMeta {
        rel_path: rel_path.to_string(),
        kind,
        modified_vs_bundled,
        byte_size: bytes,
        line_count,
        description,
    }
}

/// Build a structured tree of every `.md` file under `prompts_dir()`. The
/// tree groups files into Core / Providers / Per-model overrides / Custom.
pub fn list_files() -> Result<PromptTree> {
    let root = prompts_dir().context("home directory not resolvable")?;
    let mut always_loaded: Vec<PromptFileMeta> = Vec::new();
    let mut references_top: Vec<PromptFileMeta> = Vec::new();
    let mut references_maxroll: Vec<PromptFileMeta> = Vec::new();
    let mut providers: Vec<PromptFileMeta> = Vec::new();
    let mut models: Vec<PromptFileMeta> = Vec::new();
    let mut custom: Vec<PromptFileMeta> = Vec::new();

    for name in [FILE_SYSTEM_PROMPT, FILE_CORE_KNOWLEDGE, FILE_LOCAL_ADDENDUM] {
        let p = root.join(name);
        if p.exists() {
            always_loaded.push(meta_for(name, &p));
        }
    }

    let refs_root = root.join("references");
    if let Ok(rd) = fs::read_dir(&refs_root) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    let rel = format!("references/{file_name}");
                    references_top.push(meta_for(&rel, &path));
                }
            }
        }
    }

    let maxroll_root = refs_root.join("maxroll");
    if let Ok(rd) = fs::read_dir(&maxroll_root) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    let rel = format!("references/maxroll/{file_name}");
                    references_maxroll.push(meta_for(&rel, &path));
                }
            }
        }
    }

    let providers_dir = root.join("providers");
    if let Ok(rd) = fs::read_dir(&providers_dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    let rel = format!("providers/{file_name}");
                    providers.push(meta_for(&rel, &path));
                }
            }
        }
    }

    let models_dir = root.join("models");
    if let Ok(rd) = fs::read_dir(&models_dir) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    let rel = format!("models/{file_name}");
                    models.push(meta_for(&rel, &path));
                }
            }
        }
    }

    if let Ok(rd) = fs::read_dir(&root) {
        for entry in rd.flatten() {
            let path = entry.path();
            if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                    if !matches!(
                        file_name,
                        FILE_SYSTEM_PROMPT | FILE_CORE_KNOWLEDGE | FILE_LOCAL_ADDENDUM
                    ) {
                        custom.push(meta_for(file_name, &path));
                    }
                }
            }
        }
    }

    let sort_by_name = |v: &mut Vec<PromptFileMeta>| {
        v.sort_by(|a, b| a.rel_path.cmp(&b.rel_path));
    };
    sort_by_name(&mut always_loaded);
    sort_by_name(&mut references_top);
    sort_by_name(&mut references_maxroll);
    sort_by_name(&mut providers);
    sort_by_name(&mut models);
    sort_by_name(&mut custom);

    let mut groups = Vec::new();
    if !always_loaded.is_empty() {
        groups.push(PromptGroup {
            label: "Always loaded".to_string(),
            description: "Sent to the agent on every turn — voice, hard rules, tool inventory.".to_string(),
            items: always_loaded,
        });
    }
    if !references_top.is_empty() {
        groups.push(PromptGroup {
            label: "Reference library".to_string(),
            description: "Conceptual knowledge base. The agent fetches these on demand via the read_internal_reference tool.".to_string(),
            items: references_top,
        });
    }
    if !references_maxroll.is_empty() {
        groups.push(PromptGroup {
            label: "Reference library — Maxroll catalogue".to_string(),
            description: "Curated URLs to applied Maxroll articles (bosses, crafting, currency farming, getting started). Fetched the same way.".to_string(),
            items: references_maxroll,
        });
    }
    if !providers.is_empty() {
        groups.push(PromptGroup {
            label: "Provider overrides".to_string(),
            description: "Optional. Append extra rules for one provider (anthropic.md, ollama.md). Loaded only when the matching provider runs.".to_string(),
            items: providers,
        });
    }
    if !models.is_empty() {
        groups.push(PromptGroup {
            label: "Per-model overrides".to_string(),
            description: "Optional. Narrowest scope — fires only for the exact model id. Filename is the sanitized model id.".to_string(),
            items: models,
        });
    }
    if !custom.is_empty() {
        groups.push(PromptGroup {
            label: "Custom".to_string(),
            description: "User-added .md files at the prompts root. Not loaded automatically by the agent.".to_string(),
            items: custom,
        });
    }

    Ok(PromptTree { groups })
}

#[derive(Debug, Clone, Serialize)]
pub struct PromptFile {
    pub rel_path: String,
    pub content: String,
    pub kind: PromptKind,
    /// Bundled default for shipped files — `None` for custom files.
    pub bundled: Option<String>,
}

pub fn read_file(rel: &str) -> Result<PromptFile> {
    let path = safe_join(rel)?;
    let content = fs::read_to_string(&path).with_context(|| format!("read {rel}"))?;
    let bundled = bundled_for(rel).map(|s| s.to_string());
    let kind = if bundled.is_some() {
        PromptKind::Shipped
    } else {
        PromptKind::Custom
    };
    Ok(PromptFile {
        rel_path: rel.to_string(),
        content,
        kind,
        bundled,
    })
}

pub fn write_file(rel: &str, content: &str) -> Result<()> {
    let path = safe_join(rel)?;
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let _g = write_lock(rel).lock().expect("write_lock poisoned");
    fs::write(&path, content).with_context(|| format!("write {rel}"))?;
    Ok(())
}

pub fn reset_file(rel: &str) -> Result<()> {
    let bundled =
        bundled_for(rel).ok_or_else(|| anyhow!("{rel} is not a shipped file; nothing to reset"))?;
    write_file(rel, bundled)
}

pub fn reset_all() -> Result<()> {
    write_file(FILE_SYSTEM_PROMPT, BUNDLED_SYSTEM_PROMPT)?;
    write_file(FILE_CORE_KNOWLEDGE, BUNDLED_CORE_KNOWLEDGE)?;
    write_file(FILE_LOCAL_ADDENDUM, BUNDLED_LOCAL_ADDENDUM)?;
    Ok(())
}

pub fn delete_file(rel: &str) -> Result<()> {
    if bundled_for(rel).is_some() {
        return Err(anyhow!(
            "{rel} is shipped; use reset_file to restore the bundled default"
        ));
    }
    let path = safe_join(rel)?;
    if path.exists() {
        fs::remove_file(&path).with_context(|| format!("delete {rel}"))?;
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sanitize_handles_provider_slash() {
        assert_eq!(
            sanitize_model_id("anthropic/Claude-Haiku-4-5"),
            "anthropic-claude-haiku-4-5"
        );
        assert_eq!(
            sanitize_model_id("openai:gpt-5"),
            "openai-gpt-5"
        );
    }

    #[test]
    fn safe_join_rejects_traversal() {
        assert!(safe_join("../runtime/keys.json").is_err());
        assert!(safe_join("/etc/passwd").is_err());
        assert!(safe_join("nested/../escape.md").is_err());
    }

    #[test]
    fn safe_join_accepts_subpaths() {
        assert!(safe_join("SYSTEM_PROMPT.md").is_ok());
        assert!(safe_join("providers/anthropic.md").is_ok());
        assert!(safe_join("models/claude-haiku-4-5.md").is_ok());
    }
}
