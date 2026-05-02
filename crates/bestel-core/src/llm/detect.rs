use std::process::Stdio;

use super::anthropic::AnthropicClient;
use super::claude_cli::ClaudeCliClient;
use super::codex_cli::CodexCliClient;
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
            Some("ANTHROPIC_API_KEY non définie".into())
        },
    });
    if chosen.is_none() && key_set {
        if let Ok(c) = AnthropicClient::from_env() {
            chosen = Some(Provider::Anthropic(c));
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

