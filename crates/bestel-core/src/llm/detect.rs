use std::process::Stdio;

use anyhow::{anyhow, Result};

use super::anthropic::AnthropicClient;
use super::claude_cli::ClaudeCliClient;
use super::codex_cli::CodexCliClient;
use super::models::{ModelProfile, ProviderKind};
use super::ollama::{self, OllamaClient};
use super::spawn::cli_command;
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

    let codex = probe_cli("codex").await;
    probes.push(codex.clone());
    if codex.installed && chosen.is_none() {
        chosen = Some(Provider::CodexCli(CodexCliClient::new(
            codex.version.clone().unwrap_or_default(),
        )));
    }

    let claude = probe_cli("claude").await;
    probes.push(claude.clone());
    if claude.installed && chosen.is_none() {
        chosen = Some(Provider::ClaudeCli(ClaudeCliClient::new(
            claude.version.clone().unwrap_or_default(),
        )));
    }

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

async fn probe_cli(bin: &str) -> Probe {
    let child = cli_command(bin)
        .arg("--version")
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::null())
        .kill_on_drop(true)
        .spawn();

    let probe = Probe {
        name: cli_label(bin),
        installed: false,
        version: None,
        note: None,
    };

    let mut child = match child {
        Ok(c) => c,
        Err(_) => return probe,
    };

    let timeout = tokio::time::sleep(std::time::Duration::from_secs(3));
    tokio::pin!(timeout);

    let mut buf = String::new();
    let stdout = child.stdout.take();
    let read = async move {
        if let Some(mut s) = stdout {
            use tokio::io::AsyncReadExt;
            let _ = s.read_to_string(&mut buf).await;
        }
        buf
    };

    tokio::select! {
        v = read => {
            let _ = child.wait().await;
            let trimmed = v.trim().to_string();
            Probe {
                name: cli_label(bin),
                installed: !trimmed.is_empty(),
                version: if trimmed.is_empty() { None } else { Some(trimmed) },
                note: None,
            }
        }
        _ = &mut timeout => {
            let _ = child.kill().await;
            probe
        }
    }
}

fn cli_label(bin: &str) -> &'static str {
    match bin {
        "codex" => "codex cli",
        "claude" => "claude code",
        "gemini" => "gemini cli",
        _ => "cli",
    }
}

/// Build a fresh `Provider` for the given profile, reusing the existing
/// detection results to know whether the underlying CLI is installed
/// (codex/claude) or whether the API key is set (Anthropic). Used by the
/// model picker to hot-swap providers without restarting Bestel.
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
        ProviderKind::CodexCli => {
            let p = probe_cli("codex").await;
            if !p.installed {
                return Err(anyhow!("codex CLI not detected on PATH"));
            }
            Ok(Provider::CodexCli(CodexCliClient::new(
                p.version.unwrap_or_default(),
            )))
        }
        ProviderKind::ClaudeCli => {
            let p = probe_cli("claude").await;
            if !p.installed {
                return Err(anyhow!("claude CLI not detected on PATH"));
            }
            Ok(Provider::ClaudeCli(ClaudeCliClient::new(
                p.version.unwrap_or_default(),
            )))
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

