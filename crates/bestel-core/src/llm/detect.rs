use anyhow::{anyhow, Result};

use super::anthropic::AnthropicClient;
use super::models::{ModelProfile, ProviderKind};
use super::ollama::{self, OllamaClient};
use super::Provider;

#[derive(Debug, Clone)]
pub struct Probe {
    pub name: &'static str,
    pub installed: bool,
    pub version: Option<String>,
    pub note: Option<String>,
}

pub struct Detection {
    pub provider: Option<Provider>,
    pub probes: Vec<Probe>,
}

pub async fn detect_provider() -> Detection {
    let mut probes: Vec<Probe> = Vec::new();
    let mut chosen: Option<Provider> = None;

    let key_set = std::env::var("ANTHROPIC_API_KEY").is_ok();
    probes.push(Probe {
        name: "anthropic api key",
        installed: key_set,
        version: None,
        note: if key_set {
            None
        } else {
            Some("ANTHROPIC_API_KEY not set".into())
        },
    });
    if chosen.is_none() && key_set {
        if let Ok(c) = AnthropicClient::from_env() {
            chosen = Some(Provider::Anthropic(c));
        }
    }

    let deepseek_key_set = std::env::var("DEEPSEEK_API_KEY").is_ok();
    probes.push(Probe {
        name: "deepseek api key",
        installed: deepseek_key_set,
        version: None,
        note: if deepseek_key_set {
            Some("DeepSeek V3.2 available via Anthropic-compat endpoint".into())
        } else {
            Some("DEEPSEEK_API_KEY not set — pick DeepSeek profile to enable".into())
        },
    });

    // Ollama is the free / local fallback. We surface it as a probe even when
    // a paid provider is already chosen so the user can see local mode is
    // available and switch to it via the model picker. If nothing else
    // succeeded above and Ollama is reachable, default to it.
    let ollama_host = std::env::var("OLLAMA_HOST")
        .ok()
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "http://localhost:11434".to_string());
    let ollama_models = ollama::probe_models(Some(&ollama_host)).await;
    match &ollama_models {
        Some(list) if !list.is_empty() => {
            let preview = list
                .iter()
                .take(3)
                .cloned()
                .collect::<Vec<_>>()
                .join(", ");
            let extra = if list.len() > 3 {
                format!(" (+{} more)", list.len() - 3)
            } else {
                String::new()
            };
            probes.push(Probe {
                name: "ollama",
                installed: true,
                version: Some(format!("{} models{}", list.len(), extra)),
                note: Some(format!("e.g. {preview}")),
            });
            if chosen.is_none() {
                if let Ok(c) = OllamaClient::from_env() {
                    chosen = Some(Provider::Ollama(c));
                }
            }
        }
        Some(_) => {
            probes.push(Probe {
                name: "ollama",
                installed: true,
                version: None,
                note: Some(format!(
                    "daemon reachable at {ollama_host} but no model installed — run `ollama pull llama3.1:8b`"
                )),
            });
        }
        None => {
            probes.push(Probe {
                name: "ollama",
                installed: false,
                version: None,
                note: Some(format!("no daemon at {ollama_host} — install from https://ollama.com")),
            });
        }
    }

    Detection {
        provider: chosen,
        probes,
    }
}

/// Build a fresh `Provider` for the given profile. Used by the model
/// picker to hot-swap providers without restarting Bestel.
pub async fn build_provider_for_profile(
    profile: &ModelProfile,
) -> Result<Provider> {
    match profile.provider {
        ProviderKind::Anthropic => {
            // Always build via `from_endpoint` so the profile's `model_id`
            // is honored — `from_env` falls back to whatever the persisted
            // active_profile() says, which races with the actual profile we
            // were asked to build (e.g. battery runner switching between
            // DeepSeek and Haiku without changing model.json).
            let base_url = profile
                .base_url
                .as_deref()
                .unwrap_or("https://api.anthropic.com");
            let env_var = profile
                .api_key_env
                .as_deref()
                .unwrap_or("ANTHROPIC_API_KEY");
            let c = AnthropicClient::from_endpoint(
                base_url,
                env_var,
                profile.model_id.clone(),
                profile.cost_per_mtok,
            )
            .map_err(|e| anyhow!("{env_var} missing or endpoint unreachable: {e}"))?;
            Ok(Provider::Anthropic(c))
        }
        ProviderKind::Ollama => {
            let host = std::env::var("OLLAMA_HOST")
                .ok()
                .filter(|s| !s.is_empty())
                .unwrap_or_else(|| "http://localhost:11434".to_string());
            let models = ollama::probe_models(Some(&host))
                .await
                .ok_or_else(|| {
                    anyhow!(
                        "Ollama daemon not reachable at {host}. Install from https://ollama.com and run `ollama serve`."
                    )
                })?;
            // Verify the requested model is installed locally; otherwise the
            // first call would fail with a confusing 500 from the daemon.
            if !profile.model_id.is_empty()
                && !models.iter().any(|m| {
                    m == &profile.model_id || m.starts_with(&format!("{}:", profile.model_id))
                })
            {
                return Err(anyhow!(
                    "Ollama model `{}` not found locally. Run `ollama pull {}` first.",
                    profile.model_id,
                    profile.model_id
                ));
            }
            Ok(Provider::Ollama(OllamaClient::with(
                host,
                profile.model_id.clone(),
            )))
        }
    }
}

pub fn render_probes(probes: &[Probe]) -> String {
    let mut out = String::new();
    for p in probes {
        let mark = if p.installed { "✓" } else { "·" };
        let v = p
            .version
            .as_deref()
            .or(p.note.as_deref())
            .unwrap_or("non installé");
        out.push_str(&format!("{} {}: {}\n", mark, p.name, v));
    }
    out
}

