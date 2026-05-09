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

pub const BUNDLED_SYSTEM_PROMPT: &str = include_str!("../../../prompts/SYSTEM_PROMPT.md");
pub const BUNDLED_CORE_KNOWLEDGE: &str = include_str!("../../../prompts/CORE_KNOWLEDGE.md");
pub const BUNDLED_LOCAL_ADDENDUM: &str = include_str!("./llm/local_addendum.md");

const FILE_SYSTEM_PROMPT: &str = "SYSTEM_PROMPT.md";
const FILE_CORE_KNOWLEDGE: &str = "CORE_KNOWLEDGE.md";
const FILE_LOCAL_ADDENDUM: &str = "local_addendum.md";

/// The reference library — 27 numbered conceptual docs + creators registry +
/// PoE2 scaffolds + glossary + threshold tables + 5 maxroll catalogues.
/// Bundled into the binary so the agent has a complete fallback even when the
/// user has wiped `~/.bestel/prompts/references/`. The agent fetches these on
/// demand via the `read_internal_reference` tool — they are NOT concatenated
/// into the system prompt.
pub const BUNDLED_REFERENCES: &[(&str, &str)] = &[
    // Core numbered conceptual docs.
    ("00_README.md", include_str!("../../../prompts/references/00_README.md")),
    ("01_source_policy.md", include_str!("../../../prompts/references/01_source_policy.md")),
    ("02_arpg_foundations.md", include_str!("../../../prompts/references/02_arpg_foundations.md")),
    ("03_ggg_design_philosophy.md", include_str!("../../../prompts/references/03_ggg_design_philosophy.md")),
    ("04_game_model_poe1_poe2.md", include_str!("../../../prompts/references/04_game_model_poe1_poe2.md")),
    ("05_build_reasoning_framework.md", include_str!("../../../prompts/references/05_build_reasoning_framework.md")),
    ("06_character_stats_and_mechanics.md", include_str!("../../../prompts/references/06_character_stats_and_mechanics.md")),
    ("07_offence_damage_scaling.md", include_str!("../../../prompts/references/07_offence_damage_scaling.md")),
    ("08_defence_recovery_survivability.md", include_str!("../../../prompts/references/08_defence_recovery_survivability.md")),
    ("09_itemisation_crafting.md", include_str!("../../../prompts/references/09_itemisation_crafting.md")),
    ("10_skills_gems_passives_ascendancies.md", include_str!("../../../prompts/references/10_skills_gems_passives_ascendancies.md")),
    ("11_endgame_economy_trade_leagues.md", include_str!("../../../prompts/references/11_endgame_economy_trade_leagues.md")),
    ("12_vocabulary_glossary.md", include_str!("../../../prompts/references/12_vocabulary_glossary.md")),
    ("13_retrieval_playbooks.md", include_str!("../../../prompts/references/13_retrieval_playbooks.md")),
    ("14_validation_and_failure_modes.md", include_str!("../../../prompts/references/14_validation_and_failure_modes.md")),
    ("15_source_registry.md", include_str!("../../../prompts/references/15_source_registry.md")),
    ("16_build_methodology_and_creators.md", include_str!("../../../prompts/references/16_build_methodology_and_creators.md")),
    ("17_build_archetype_taxonomy.md", include_str!("../../../prompts/references/17_build_archetype_taxonomy.md")),
    ("18_atlas_and_endgame_mechanics.md", include_str!("../../../prompts/references/18_atlas_and_endgame_mechanics.md")),
    ("19_combat_movement_animation.md", include_str!("../../../prompts/references/19_combat_movement_animation.md")),
    ("20_item_basetype_identity.md", include_str!("../../../prompts/references/20_item_basetype_identity.md")),
    ("21_currency_and_barter_taxonomy.md", include_str!("../../../prompts/references/21_currency_and_barter_taxonomy.md")),
    ("22_trade_etiquette_and_scams.md", include_str!("../../../prompts/references/22_trade_etiquette_and_scams.md")),
    ("23_hardcore_softcore_ssf_mode_differences.md", include_str!("../../../prompts/references/23_hardcore_softcore_ssf_mode_differences.md")),
    ("24_patch_history_meta.md", include_str!("../../../prompts/references/24_patch_history_meta.md")),
    ("25_pob_engine_integration.md", include_str!("../../../prompts/references/25_pob_engine_integration.md")),
    ("26_validation_and_self_correction.md", include_str!("../../../prompts/references/26_validation_and_self_correction.md")),
    // Sprint B runtime extension — output contracts, failure policy, tripwires, panel grammar, mode examples.
    ("27_response_contracts.md", include_str!("../../../prompts/references/27_response_contracts.md")),
    ("28_tool_failure_policy.md", include_str!("../../../prompts/references/28_tool_failure_policy.md")),
    ("29_known_mechanics_tripwires.md", include_str!("../../../prompts/references/29_known_mechanics_tripwires.md")),
    ("30_panel_marker_grammar.md", include_str!("../../../prompts/references/30_panel_marker_grammar.md")),
    ("31_answer_mode_examples.md", include_str!("../../../prompts/references/31_answer_mode_examples.md")),
    // Creators registry — per-creator profiles, indexed by 00_README.
    ("creators_registry/00_README.md", include_str!("../../../prompts/references/creators_registry/00_README.md")),
    ("creators_registry/ben_.md", include_str!("../../../prompts/references/creators_registry/ben_.md")),
    ("creators_registry/coconutmage.md", include_str!("../../../prompts/references/creators_registry/coconutmage.md")),
    ("creators_registry/darthmicrotransaction.md", include_str!("../../../prompts/references/creators_registry/darthmicrotransaction.md")),
    ("creators_registry/dslily.md", include_str!("../../../prompts/references/creators_registry/dslily.md")),
    ("creators_registry/fubgun.md", include_str!("../../../prompts/references/creators_registry/fubgun.md")),
    ("creators_registry/furty.md", include_str!("../../../prompts/references/creators_registry/furty.md")),
    ("creators_registry/ghazzy.md", include_str!("../../../prompts/references/creators_registry/ghazzy.md")),
    ("creators_registry/goratha.md", include_str!("../../../prompts/references/creators_registry/goratha.md")),
    ("creators_registry/kobeblaubeere.md", include_str!("../../../prompts/references/creators_registry/kobeblaubeere.md")),
    ("creators_registry/kripparrian.md", include_str!("../../../prompts/references/creators_registry/kripparrian.md")),
    ("creators_registry/mathil.md", include_str!("../../../prompts/references/creators_registry/mathil.md")),
    ("creators_registry/octoxy.md", include_str!("../../../prompts/references/creators_registry/octoxy.md")),
    ("creators_registry/palsteron.md", include_str!("../../../prompts/references/creators_registry/palsteron.md")),
    ("creators_registry/pohx.md", include_str!("../../../prompts/references/creators_registry/pohx.md")),
    ("creators_registry/quin69.md", include_str!("../../../prompts/references/creators_registry/quin69.md")),
    ("creators_registry/ruetoo.md", include_str!("../../../prompts/references/creators_registry/ruetoo.md")),
    ("creators_registry/subtractem.md", include_str!("../../../prompts/references/creators_registry/subtractem.md")),
    ("creators_registry/tripolarbear.md", include_str!("../../../prompts/references/creators_registry/tripolarbear.md")),
    ("creators_registry/zizaran.md", include_str!("../../../prompts/references/creators_registry/zizaran.md")),
    ("creators_registry/ziggyd.md", include_str!("../../../prompts/references/creators_registry/ziggyd.md")),
    // PoE2 scaffolds — version-pinned mechanics. 0.5 stubs filled at launch.
    ("poe2/00_version_pinning.md", include_str!("../../../prompts/references/poe2/00_version_pinning.md")),
    ("poe2/01_spirit_economy.md", include_str!("../../../prompts/references/poe2/01_spirit_economy.md")),
    ("poe2/02_weapon_sets.md", include_str!("../../../prompts/references/poe2/02_weapon_sets.md")),
    ("poe2/03_trials_sekhemas_chaos.md", include_str!("../../../prompts/references/poe2/03_trials_sekhemas_chaos.md")),
    ("poe2/04_runes_soul_cores_talismans.md", include_str!("../../../prompts/references/poe2/04_runes_soul_cores_talismans.md")),
    ("poe2/05_atlas_mechanics_05.md", include_str!("../../../prompts/references/poe2/05_atlas_mechanics_05.md")),
    ("poe2/06_runes_of_aldur.md", include_str!("../../../prompts/references/poe2/06_runes_of_aldur.md")),
    // Thresholds — version-pinned numerical bars by content tier.
    ("thresholds/red_maps.md", include_str!("../../../prompts/references/thresholds/red_maps.md")),
    ("thresholds/pinnacle.md", include_str!("../../../prompts/references/thresholds/pinnacle.md")),
    ("thresholds/uber_pinnacle.md", include_str!("../../../prompts/references/thresholds/uber_pinnacle.md")),
    // Maxroll catalogues — author-curated article index.
    ("maxroll/00_README.md", include_str!("../../../prompts/references/maxroll/00_README.md")),
    ("maxroll/poe1_bosses.md", include_str!("../../../prompts/references/maxroll/poe1_bosses.md")),
    ("maxroll/poe1_crafting.md", include_str!("../../../prompts/references/maxroll/poe1_crafting.md")),
    ("maxroll/poe1_currency.md", include_str!("../../../prompts/references/maxroll/poe1_currency.md")),
    ("maxroll/poe1_getting_started.md", include_str!("../../../prompts/references/maxroll/poe1_getting_started.md")),
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
///
/// On miss, the error message lists the closest matches (token-overlap rank)
/// from the bundle so a retrying agent can self-correct without consulting
/// CORE_KNOWLEDGE again. The agent regression that triggered this:
/// Haiku invented `09_build_archetypes.md` when it wanted
/// `17_build_archetype_taxonomy.md` — wrong number AND plural. The error
/// must surface the canonical filename within the agent's loop.
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
    let suggestions = suggest_close_paths(rel_path, 5);
    Err(anyhow!(
        "'{rel_path}' is not in the reference library. Did you mean one of: {}? Full list lives in the `read_internal_reference` tool's `rel_path` enum.",
        suggestions.join(", ")
    ))
}

/// Token-overlap fuzzy match against the bundled paths. Returns up to `limit`
/// suggestions sorted by overlap descending. Used by the not-found error.
fn suggest_close_paths(needle: &str, limit: usize) -> Vec<String> {
    let needle_tokens = path_tokens(needle);
    let mut scored: Vec<(usize, &'static str)> = Vec::new();
    for (k, _) in BUNDLED_REFERENCES {
        let hay = path_tokens(k);
        let overlap = needle_tokens
            .iter()
            .filter(|t| hay.contains(t))
            .count();
        if overlap > 0 {
            scored.push((overlap, *k));
        }
    }
    scored.sort_by(|a, b| b.0.cmp(&a.0).then_with(|| a.1.cmp(b.1)));
    scored
        .into_iter()
        .take(limit)
        .map(|(_, p)| p.to_string())
        .collect()
}

fn path_tokens(s: &str) -> Vec<String> {
    s.to_lowercase()
        .split(|c: char| !c.is_alphanumeric())
        .filter(|t| !t.is_empty())
        .map(|t| t.to_string())
        .collect()
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

/// Three system-prompt blocks designed for Anthropic prompt-cache breakpoints
/// (Sprint C). Splits `SYSTEM_PROMPT.md` on the opening `<tool_policy>` tag —
/// everything before is block 1 (persona + runtime contract), everything from
/// that tag onward is block 2 (tool policy + answer mode router + output
/// contracts + failure policy + auxiliaries). `CORE_KNOWLEDGE.md` is block 3.
/// Provider + model overrides are appended to block 2 so they share its cache
/// entry (overrides are stable per (provider, model) pair).
///
/// Returned tuple: `(block_1, block_2_with_overrides, block_3_core_knowledge)`.
/// All three are intended to receive `cache_control: ephemeral` with TTL 1h.
pub fn load_anthropic_blocks(provider: &str, model_id: &str) -> (String, String, String) {
    let system = read_or_bundled(FILE_SYSTEM_PROMPT, BUNDLED_SYSTEM_PROMPT);
    let core = read_or_bundled(FILE_CORE_KNOWLEDGE, BUNDLED_CORE_KNOWLEDGE);

    let split_marker = "<tool_policy>";
    let (block_1, block_2_base) = match system.find(split_marker) {
        Some(pos) => (system[..pos].to_string(), system[pos..].to_string()),
        // SYSTEM_PROMPT.md was edited and lost the `<tool_policy>` marker.
        // Fall back to a single block so caching still works at coarser
        // granularity instead of breaking the request entirely.
        None => (String::new(), system.clone()),
    };

    let mut block_2 = block_2_base;
    if let Some(po) = load_provider_override(provider) {
        block_2.push_str("\n\n---\n\n");
        block_2.push_str(&po);
    }
    if let Some(mo) = load_model_override(model_id) {
        block_2.push_str("\n\n---\n\n");
        block_2.push_str(&mo);
    }

    // Sprint F — append the skill descriptions block to the cached BP4
    // (CORE_KNOWLEDGE) section. The model sees skill names + when-to-use
    // hints alongside CORE_KNOWLEDGE; skill BODIES still load on demand
    // via the `load_skill` tool.
    let mut core_with_skills = core;
    let skills_block = crate::skills::descriptions_block();
    if !skills_block.is_empty() {
        if !core_with_skills.is_empty() {
            core_with_skills.push_str("\n\n---\n\n");
        }
        core_with_skills.push_str(&skills_block);
    }

    (block_1, block_2, core_with_skills)
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
    let mut references_creators: Vec<PromptFileMeta> = Vec::new();
    let mut references_poe2: Vec<PromptFileMeta> = Vec::new();
    let mut references_thresholds: Vec<PromptFileMeta> = Vec::new();
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

    let scan_subdir = |subdir: &str, bucket: &mut Vec<PromptFileMeta>| {
        let dir = refs_root.join(subdir);
        if let Ok(rd) = fs::read_dir(&dir) {
            for entry in rd.flatten() {
                let path = entry.path();
                if path.is_file() && path.extension().and_then(|s| s.to_str()) == Some("md") {
                    if let Some(file_name) = path.file_name().and_then(|s| s.to_str()) {
                        let rel = format!("references/{subdir}/{file_name}");
                        bucket.push(meta_for(&rel, &path));
                    }
                }
            }
        }
    };
    scan_subdir("creators_registry", &mut references_creators);
    scan_subdir("poe2", &mut references_poe2);
    scan_subdir("thresholds", &mut references_thresholds);
    scan_subdir("maxroll", &mut references_maxroll);

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
    sort_by_name(&mut references_creators);
    sort_by_name(&mut references_poe2);
    sort_by_name(&mut references_thresholds);
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
    if !references_creators.is_empty() {
        groups.push(PromptGroup {
            label: "Reference library — Creator profiles".to_string(),
            description: "Per-creator profiles (build creators on Maxroll, Mobalytics, Twitch). Each profile lists archetypes, signature quirks, biases, where to verify.".to_string(),
            items: references_creators,
        });
    }
    if !references_poe2.is_empty() {
        groups.push(PromptGroup {
            label: "Reference library — PoE2 mechanics".to_string(),
            description: "Version-pinned PoE2-specific mechanics. Always check 00_version_pinning.md before applying claims here.".to_string(),
            items: references_poe2,
        });
    }
    if !references_thresholds.is_empty() {
        groups.push(PromptGroup {
            label: "Reference library — Threshold tables".to_string(),
            description: "Version-pinned numerical bars per content tier (red maps, pinnacle, uber pinnacle). DPS, EHP, max-hit, recovery floors.".to_string(),
            items: references_thresholds,
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
