//! Curated model registry + persistent active-profile selection.
//!
//! Bestel does not benefit from an exhaustive model catalogue — text
//! research is the only workload — so this file ships a *short*
//! hand-picked list. Each entry maps to a (provider, model_id) pair plus
//! enough metadata for the picker UI to label it.
//!
//! Persistence: `~/.bestel/runtime/model.json` stores the chosen `id`.
//! The TUI loads the active profile at startup and writes through the
//! picker. CLIs read the same file via `active_profile()`.

use std::path::PathBuf;

use anyhow::{anyhow, Context, Result};
use serde::{Deserialize, Serialize};

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

/// A user-selectable (provider, model) profile.
#[derive(Debug, Clone)]
pub struct ModelProfile {
    pub id: &'static str,
    pub provider: ProviderKind,
    /// Model id passed to the API / CLI (`-m` for codex, `model` field for
    /// Anthropic API). Empty when the provider does not accept a model
    /// override (Claude Code CLI uses whatever the subscription gives).
    pub model_id: &'static str,
    pub display_name: &'static str,
    pub description: &'static str,
    pub speed: SpeedTier,
    pub cost: CostTier,
    /// Approximate cost in USD per million tokens (input, output) — for
    /// API providers only. `None` for subscription / CLI flows.
    pub cost_per_mtok: Option<(f32, f32)>,
}

/// Curated catalogue. Order matters: it is the order shown in the picker.
pub fn all_profiles() -> &'static [ModelProfile] {
    &PROFILES
}

const PROFILES: &[ModelProfile] = &[
    ModelProfile {
        id: "anthropic-sonnet-4-5",
        provider: ProviderKind::Anthropic,
        model_id: "claude-sonnet-4-5-20250929",
        display_name: "Claude Sonnet 4.5",
        description:
            "Anthropic API, balanced research model. Strong tool-calling and synthesis. Default for the Anthropic path.",
        speed: SpeedTier::Balanced,
        cost: CostTier::Mid,
        cost_per_mtok: Some((3.0, 15.0)),
    },
    ModelProfile {
        id: "anthropic-haiku-4-5",
        provider: ProviderKind::Anthropic,
        model_id: "claude-haiku-4-5-20251001",
        display_name: "Claude Haiku 4.5",
        description:
            "Cheapest Anthropic option, ~3x faster than Sonnet. Lower task completion on multi-step research; good for follow-ups, weak on cold synthesis.",
        speed: SpeedTier::Fast,
        cost: CostTier::Cheap,
        cost_per_mtok: Some((1.0, 5.0)),
    },
    ModelProfile {
        id: "anthropic-opus-4-7",
        provider: ProviderKind::Anthropic,
        model_id: "claude-opus-4-7",
        display_name: "Claude Opus 4.7",
        description:
            "Anthropic flagship. Most thorough research, slowest, most expensive. Use for deep dives.",
        speed: SpeedTier::Heavy,
        cost: CostTier::Premium,
        cost_per_mtok: Some((5.0, 25.0)),
    },
    ModelProfile {
        id: "codex-default",
        provider: ProviderKind::CodexCli,
        model_id: "",
        display_name: "Codex CLI (default)",
        description:
            "Whatever model your Codex CLI subscription resolves to (gpt-5-codex on ChatGPT accounts). The safe pick — Bestel passes no `-m` flag and lets codex choose.",
        speed: SpeedTier::Balanced,
        cost: CostTier::Subscription,
        cost_per_mtok: None,
    },
    ModelProfile {
        id: "codex-gpt-5-codex",
        provider: ProviderKind::CodexCli,
        model_id: "gpt-5-codex",
        display_name: "Codex CLI · GPT-5 Codex",
        description:
            "Explicit gpt-5-codex via `-m`. Code-specialised; works on ChatGPT accounts. Same model as the default for most subscriptions.",
        speed: SpeedTier::Balanced,
        cost: CostTier::Subscription,
        cost_per_mtok: None,
    },
    ModelProfile {
        id: "claude-cli",
        provider: ProviderKind::ClaudeCli,
        model_id: "",
        display_name: "Claude Code CLI",
        description:
            "Whatever model the user's Claude subscription provides. No model override possible — the CLI picks.",
        speed: SpeedTier::Balanced,
        cost: CostTier::Subscription,
        cost_per_mtok: None,
    },
    // Local / free models via Ollama. Free, offline, no API key. Tool
    // calling depends on the model — the entries below all support
    // function calling. Model availability depends on which models the
    // user has pulled (`ollama pull <name>`); the picker filters this
    // list against the live `/api/tags` probe at startup.
    ModelProfile {
        id: "ollama-llama3.1-8b",
        provider: ProviderKind::Ollama,
        model_id: "llama3.1:8b",
        display_name: "Ollama · Llama 3.1 8B",
        description:
            "Local Llama 3.1 8B via Ollama. Free, fast on consumer GPUs (8 GB VRAM), supports tool calling. Run `ollama pull llama3.1:8b` first.",
        speed: SpeedTier::Fast,
        cost: CostTier::Free,
        cost_per_mtok: None,
    },
    ModelProfile {
        id: "ollama-llama3.1-70b",
        provider: ProviderKind::Ollama,
        model_id: "llama3.1:70b",
        display_name: "Ollama · Llama 3.1 70B",
        description:
            "Local Llama 3.1 70B via Ollama. Heavy: needs ~40 GB VRAM or strong CPU offload. Stronger reasoning than the 8B; supports tool calling.",
        speed: SpeedTier::Heavy,
        cost: CostTier::Free,
        cost_per_mtok: None,
    },
    ModelProfile {
        id: "ollama-qwen2.5-7b",
        provider: ProviderKind::Ollama,
        model_id: "qwen2.5:7b",
        display_name: "Ollama · Qwen 2.5 7B",
        description:
            "Local Qwen 2.5 7B via Ollama. Free, strong on tool calling and structured output. Good default for low-VRAM machines.",
        speed: SpeedTier::Fast,
        cost: CostTier::Free,
        cost_per_mtok: None,
    },
    ModelProfile {
        id: "ollama-mistral-nemo",
        provider: ProviderKind::Ollama,
        model_id: "mistral-nemo",
        display_name: "Ollama · Mistral Nemo 12B",
        description:
            "Local Mistral Nemo 12B via Ollama. Free, strong multilingual support, native tool calling. Run `ollama pull mistral-nemo` first.",
        speed: SpeedTier::Balanced,
        cost: CostTier::Free,
        cost_per_mtok: None,
    },
    ModelProfile {
        id: "ollama-llama3.2-vision",
        provider: ProviderKind::Ollama,
        model_id: "llama3.2-vision",
        display_name: "Ollama · Llama 3.2 Vision",
        description:
            "Local Llama 3.2 Vision via Ollama. Free, accepts image attachments. Lighter on tool calling than text-only models — use it when you want to drop a screenshot.",
        speed: SpeedTier::Balanced,
        cost: CostTier::Free,
        cost_per_mtok: None,
    },
];

pub fn default_profile() -> &'static ModelProfile {
    &PROFILES[0]
}

pub fn find_profile(id: &str) -> Option<&'static ModelProfile> {
    PROFILES.iter().find(|p| p.id == id)
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
/// back too. This keeps the TUI from refusing to start when the registry
/// gets pruned between releases.
pub fn active_profile() -> &'static ModelProfile {
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
    find_profile(&parsed.profile_id).unwrap_or(default_profile())
}

pub fn save_active_profile(id: &str) -> Result<()> {
    if find_profile(id).is_none() {
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
        Some(p.model_id.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn default_is_first_profile() {
        assert_eq!(default_profile().id, PROFILES[0].id);
    }

    #[test]
    fn lookup_unknown_returns_none() {
        assert!(find_profile("nope").is_none());
    }

    #[test]
    fn lookup_known_returns_some() {
        assert!(find_profile("anthropic-sonnet-4-5").is_some());
        assert!(find_profile("codex-gpt-5-codex").is_some());
        assert!(find_profile("ollama-llama3.1-8b").is_some());
    }
}
