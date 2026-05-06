//! Curated model registry + persistent active-profile selection.
//!
//! Cloud profiles (Anthropic API, Codex CLI, Claude CLI) are hand-picked
//! and shipped statically. Ollama profiles are built dynamically from the
//! daemon's `/api/tags` response so the picker only ever shows models the
//! user has actually pulled — no fictional "run `ollama pull X` first"
//! ghosts.
//!
//! Persistence: `~/.bestel/runtime/model.json` stores the chosen `id`.
//! Cloud ids are stable strings (`anthropic-sonnet-4-5` etc.). Ollama
//! dynamic ids embed the tag literally (`ollama:qwen3:8b`) so a sync
//! `resolved_model_for` call can recover the model tag without probing.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

use super::ollama::OllamaModelInfo;

/// Which client implementation will run the request.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ProviderKind {
    /// `AnthropicClient` — direct API, requires `ANTHROPIC_API_KEY`.
    Anthropic,
    /// `CodexCliClient` — spawns the `codex` subprocess. Subscription billing.
    CodexCli,
    /// `ClaudeCliClient` — spawns the `claude` subprocess. Subscription billing.
    ClaudeCli,
    /// `OllamaClient` — connects to a local Ollama daemon. Free, offline-capable.
    Ollama,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum SpeedTier {
    Fast,
    Balanced,
    Heavy,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CostTier {
    Free,
    Cheap,
    Mid,
    Premium,
    Subscription,
}

/// A user-selectable (provider, model) profile. Owned strings everywhere
/// so cloud profiles (built once from constants) and Ollama profiles
/// (built dynamically from `/api/tags`) live in the same vector.
#[derive(Debug, Clone)]
pub struct ModelProfile {
    pub id: String,
    pub provider: ProviderKind,
    /// Model id passed to the API / CLI (`-m` for codex, `model` field for
    /// Anthropic API, `model` for Ollama). Empty when the provider does
    /// not accept a model override (Claude Code CLI uses whatever the
    /// subscription gives).
    pub model_id: String,
    pub display_name: String,
    pub description: String,
    pub speed: SpeedTier,
    pub cost: CostTier,
    /// Approximate cost in USD per million tokens (input, output) — for
    /// API providers only. `None` for subscription / CLI / local flows.
    pub cost_per_mtok: Option<(f32, f32)>,
}

const DYNAMIC_OLLAMA_PREFIX: &str = "ollama:";

/// All cloud profiles (Anthropic + Codex + Claude CLI). Pure data, sync.
/// Ollama profiles are NOT here — they come from [`list_profiles_with_local`].
pub fn cloud_profiles() -> Vec<ModelProfile> {
    vec![
        ModelProfile {
            id: "anthropic-sonnet-4-5".into(),
            provider: ProviderKind::Anthropic,
            model_id: "claude-sonnet-4-5-20250929".into(),
            display_name: "Claude Sonnet 4.5".into(),
            description:
                "Anthropic API, balanced research model. Strong tool-calling and synthesis. Default for the Anthropic path.".into(),
            speed: SpeedTier::Balanced,
            cost: CostTier::Mid,
            cost_per_mtok: Some((3.0, 15.0)),
        },
        ModelProfile {
            id: "anthropic-haiku-4-5".into(),
            provider: ProviderKind::Anthropic,
            model_id: "claude-haiku-4-5-20251001".into(),
            display_name: "Claude Haiku 4.5".into(),
            description:
                "Cheapest Anthropic option, ~3x faster than Sonnet. Lower task completion on multi-step research; good for follow-ups, weak on cold synthesis.".into(),
            speed: SpeedTier::Fast,
            cost: CostTier::Cheap,
            cost_per_mtok: Some((1.0, 5.0)),
        },
        ModelProfile {
            id: "anthropic-opus-4-7".into(),
            provider: ProviderKind::Anthropic,
            model_id: "claude-opus-4-7".into(),
            display_name: "Claude Opus 4.7".into(),
            description:
                "Anthropic flagship. Most thorough research, slowest, most expensive. Use for deep dives.".into(),
            speed: SpeedTier::Heavy,
            cost: CostTier::Premium,
            cost_per_mtok: Some((5.0, 25.0)),
        },
        ModelProfile {
            id: "codex-default".into(),
            provider: ProviderKind::CodexCli,
            model_id: "".into(),
            display_name: "Codex CLI (default)".into(),
            description:
                "Whatever model your Codex CLI subscription resolves to (gpt-5-codex on ChatGPT accounts). The safe pick — Bestel passes no `-m` flag and lets codex choose.".into(),
            speed: SpeedTier::Balanced,
            cost: CostTier::Subscription,
            cost_per_mtok: None,
        },
        ModelProfile {
            id: "codex-gpt-5-codex".into(),
            provider: ProviderKind::CodexCli,
            model_id: "gpt-5-codex".into(),
            display_name: "Codex CLI · GPT-5 Codex".into(),
            description:
                "Explicit gpt-5-codex via `-m`. Code-specialised; works on ChatGPT accounts. Same model as the default for most subscriptions.".into(),
            speed: SpeedTier::Balanced,
            cost: CostTier::Subscription,
            cost_per_mtok: None,
        },
        ModelProfile {
            id: "claude-cli".into(),
            provider: ProviderKind::ClaudeCli,
            model_id: "".into(),
            display_name: "Claude Code CLI".into(),
            description:
                "Whatever model the user's Claude subscription provides. No model override possible — the CLI picks.".into(),
            speed: SpeedTier::Balanced,
            cost: CostTier::Subscription,
            cost_per_mtok: None,
        },
    ]
}

/// Returns the first cloud profile (Anthropic Sonnet 4.5). Used as a sane
/// fallback when the persisted profile id can't be resolved.
pub fn default_profile() -> ModelProfile {
    cloud_profiles().into_iter().next().expect("cloud profiles non-empty")
}

/// Lookup a profile by id without probing Ollama. For dynamic Ollama ids
/// (`ollama:<tag>`), this builds a minimal stub from the tag alone — the
/// daemon may be offline, but we still need a valid `model_id` to feed
/// the provider. The picker enriches dynamic profiles separately via
/// [`list_profiles_with_local`].
pub fn find_profile(id: &str) -> Option<ModelProfile> {
    if let Some(tag) = id.strip_prefix(DYNAMIC_OLLAMA_PREFIX) {
        if tag.is_empty() {
            return None;
        }
        return Some(stub_ollama_profile(tag));
    }
    cloud_profiles().into_iter().find(|p| p.id == id)
}

/// Build a minimal Ollama profile from a bare tag — used when the daemon
/// is unreachable but we know the user's last selection. The tag becomes
/// the `model_id`; display name is humanised; speed/cost are defaults.
fn stub_ollama_profile(tag: &str) -> ModelProfile {
    ModelProfile {
        id: format!("{DYNAMIC_OLLAMA_PREFIX}{tag}"),
        provider: ProviderKind::Ollama,
        model_id: tag.to_string(),
        display_name: humanize_ollama_tag(tag, None),
        description: format!("Local model via Ollama. Tag: `{tag}`."),
        speed: SpeedTier::Balanced,
        cost: CostTier::Free,
        cost_per_mtok: None,
    }
}

/// Build a rich Ollama profile from `/api/tags` metadata — used by the
/// picker so the user sees parameter size + speed hints, not just the
/// raw tag.
pub fn build_ollama_profile(info: &OllamaModelInfo) -> ModelProfile {
    let tag = info.name.clone();
    let display_name = humanize_ollama_tag(&tag, Some(info));
    let description = describe_ollama_info(info);
    let speed = classify_ollama_speed(info);
    ModelProfile {
        id: format!("{DYNAMIC_OLLAMA_PREFIX}{tag}"),
        provider: ProviderKind::Ollama,
        model_id: tag,
        display_name,
        description,
        speed,
        cost: CostTier::Free,
        cost_per_mtok: None,
    }
}

fn humanize_ollama_tag(tag: &str, info: Option<&OllamaModelInfo>) -> String {
    let (family_part, size_part) = tag.split_once(':').unwrap_or((tag, ""));
    let family_pretty = humanize_family(family_part);
    let size_suffix = if size_part.is_empty() || size_part == "latest" {
        info.and_then(|i| i.parameter_size.clone())
            .map(|p| format!(" {p}"))
            .unwrap_or_default()
    } else {
        format!(" {}", size_part.to_uppercase())
    };
    format!("Ollama · {family_pretty}{size_suffix}")
}

fn humanize_family(family: &str) -> String {
    let mut out = String::with_capacity(family.len());
    let mut prev_alpha = false;
    for ch in family.chars() {
        if ch == '-' || ch == '_' {
            out.push(' ');
            prev_alpha = false;
            continue;
        }
        if !prev_alpha && ch.is_alphabetic() {
            out.extend(ch.to_uppercase());
        } else {
            out.push(ch);
        }
        prev_alpha = ch.is_alphanumeric();
    }
    if let Some((head, tail)) = out.split_once(char::is_numeric) {
        let mut rebuilt = String::with_capacity(out.len() + 1);
        rebuilt.push_str(head.trim_end());
        rebuilt.push(' ');
        let first = out[head.len()..head.len() + 1].chars().next().unwrap_or(' ');
        rebuilt.push(first);
        rebuilt.push_str(tail);
        rebuilt
    } else {
        out
    }
}

fn describe_ollama_info(info: &OllamaModelInfo) -> String {
    let mut bits: Vec<String> = Vec::new();
    bits.push("Local model via Ollama. Free, offline.".to_string());
    if let Some(p) = &info.parameter_size {
        bits.push(format!("Parameters: {p}."));
    }
    if let Some(b) = info.size_bytes {
        bits.push(format!("Size on disk: {}.", format_bytes(b)));
    }
    bits.join(" ")
}

fn format_bytes(b: u64) -> String {
    const GB: f64 = 1024.0 * 1024.0 * 1024.0;
    const MB: f64 = 1024.0 * 1024.0;
    let f = b as f64;
    if f >= GB {
        format!("{:.1} GB", f / GB)
    } else if f >= MB {
        format!("{:.0} MB", f / MB)
    } else {
        format!("{b} B")
    }
}

fn classify_ollama_speed(info: &OllamaModelInfo) -> SpeedTier {
    if let Some(b) = info.size_bytes {
        const FAST_LIMIT: u64 = 6 * 1024 * 1024 * 1024;
        const HEAVY_LIMIT: u64 = 18 * 1024 * 1024 * 1024;
        if b <= FAST_LIMIT {
            return SpeedTier::Fast;
        }
        if b >= HEAVY_LIMIT {
            return SpeedTier::Heavy;
        }
        return SpeedTier::Balanced;
    }
    SpeedTier::Balanced
}

/// Build the picker list: cloud profiles + every Ollama model the user
/// has currently pulled. If the daemon is unreachable, only cloud
/// profiles are returned (no fictional Ollama entries).
pub async fn list_profiles_with_local() -> Vec<ModelProfile> {
    let mut out = cloud_profiles();
    let host = std::env::var("OLLAMA_HOST")
        .ok()
        .filter(|s| !s.is_empty());
    if let Some(infos) = super::ollama::probe_models_detailed(host.as_deref()).await {
        for info in infos {
            out.push(build_ollama_profile(&info));
        }
    }
    out
}

/// Path of the persisted active-profile pointer. Lives next to the other
/// runtime artefacts under `~/.bestel/runtime/`.
fn active_profile_path() -> Option<PathBuf> {
    dirs::home_dir().map(|h| h.join(".bestel").join("runtime").join("model.json"))
}

#[derive(Debug, Serialize, Deserialize)]
struct Persisted {
    profile_id: String,
}

/// Read the persisted active profile, or fall back to the default. Bad /
/// missing file is not an error — we silently default. Unknown ids fall
/// back too. This keeps the app from refusing to start when the registry
/// gets pruned between releases or the user uninstalls the Ollama tag
/// they were using.
pub fn active_profile() -> ModelProfile {
    let Some(path) = active_profile_path() else {
        return default_profile();
    };
    let raw = match std::fs::read_to_string(&path) {
        Ok(s) => s,
        Err(_) => return default_profile(),
    };
    let parsed: Persisted = match serde_json::from_str(&raw) {
        Ok(p) => p,
        Err(_) => return default_profile(),
    };
    find_profile(&parsed.profile_id).unwrap_or_else(default_profile)
}

/// Persist the picker's choice. Cloud ids must match the curated list.
/// Dynamic Ollama ids (prefixed `ollama:`) are accepted without probe —
/// the picker is the source of truth for what's installed; offline
/// re-launch should still recover the user's last choice.
pub fn save_active_profile(id: &str) -> Result<()> {
    let known = id.starts_with(DYNAMIC_OLLAMA_PREFIX)
        || cloud_profiles().iter().any(|p| p.id == id);
    if !known {
        return Err(anyhow!("unknown profile id '{id}'"));
    }
    let path = active_profile_path().ok_or_else(|| anyhow!("no home dir"))?;
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).ok();
    }
    let body = serde_json::to_vec_pretty(&Persisted {
        profile_id: id.to_string(),
    })
    .context("serialize profile")?;
    std::fs::write(&path, body).with_context(|| format!("write {}", path.display()))?;
    Ok(())
}

/// Convenience: model id to use for a given provider, taking the active
/// profile into account but allowing the env var to win (for one-off
/// overrides during testing).
pub fn resolved_model_for(provider: ProviderKind) -> Option<String> {
    if let Ok(env) = std::env::var("BESTEL_MODEL") {
        if !env.is_empty() {
            return Some(env);
        }
    }
    let p = active_profile();
    if p.provider == provider && !p.model_id.is_empty() {
        Some(p.model_id)
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_first_cloud_profile() {
        assert_eq!(default_profile().id, cloud_profiles()[0].id);
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(find_profile("nope").is_none());
    }

    #[test]
    fn lookup_cloud_returns_some() {
        assert!(find_profile("anthropic-sonnet-4-5").is_some());
        assert!(find_profile("codex-gpt-5-codex").is_some());
    }

    #[test]
    fn lookup_dynamic_ollama_builds_stub() {
        let p = find_profile("ollama:qwen3:8b").expect("dynamic ollama profile");
        assert_eq!(p.provider, ProviderKind::Ollama);
        assert_eq!(p.model_id, "qwen3:8b");
        assert!(p.display_name.contains("Qwen"));
    }

    #[test]
    fn lookup_dynamic_ollama_empty_tag_rejected() {
        assert!(find_profile("ollama:").is_none());
    }

    #[test]
    fn humanize_handles_common_tags() {
        assert!(humanize_ollama_tag("qwen3:8b", None).contains("Qwen 3 8B"));
        assert!(humanize_ollama_tag("llama3.1:8b", None).contains("Llama 3.1 8B"));
        assert!(humanize_ollama_tag("mistral-nemo:latest", None).contains("Mistral Nemo"));
    }
}
