use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::tools::BuildContext;
use super::{ChatMessage, LlmDelta, Role};
use crate::prompt::SYSTEM_PROMPT;

pub struct ClaudeCliClient {
    version: String,
}

impl ClaudeCliClient {
    pub fn new(version: String) -> Self {
        Self { version }
    }

    pub fn version_label(&self) -> &str {
        if self.version.is_empty() {
            "subscription"
        } else {
            &self.version
        }
    }

    pub async fn run(
        &self,
        history: Vec<ChatMessage>,
        ctx: BuildContext,
        deltas: mpsc::UnboundedSender<LlmDelta>,
    ) -> Result<String> {
        let user_prompt = build_user_prompt(&history, &ctx);
        let system_addon = format!(
            "{}\n\n[CURRENT PATH OF BUILDING DATA]\n{}\n\n[INSTRUCTIONS]\n\
             Stay in character. Do not modify files. Do not run shell commands. Answer in the user's language.",
            SYSTEM_PROMPT,
            ctx.render_tool_result()
        );

        let mut cmd = Command::new("claude");
        cmd.arg("-p")
            .arg("--bare")
            .arg("--output-format")
            .arg("text")
            .arg("--append-system-prompt")
            .arg(&system_addon)
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow!("failed to spawn claude CLI: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(user_prompt.as_bytes()).await?;
            stdin.shutdown().await.ok();
            drop(stdin);
        }

        let mut stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("claude stdout missing"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("claude stderr missing"))?;

        let mut err_reader = BufReader::new(stderr).lines();
        let err_task = tokio::spawn(async move {
            let mut acc = String::new();
            while let Ok(Some(line)) = err_reader.next_line().await {
                acc.push_str(&line);
                acc.push('\n');
                if acc.len() > 16_000 {
                    break;
                }
            }
            acc
        });

        let mut full = String::new();
        let mut buf = [0u8; 1024];
        loop {
            let n = stdout.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            let chunk = String::from_utf8_lossy(&buf[..n]).to_string();
            full.push_str(&chunk);
            let _ = deltas.send(LlmDelta::Text(chunk));
        }

        let err_buf = err_task.await.unwrap_or_default();
        let status = child.wait().await?;

        if !status.success() && full.trim().is_empty() {
            let hint = if err_buf.to_lowercase().contains("auth")
                || err_buf.to_lowercase().contains("login")
            {
                " — connecte-toi avec `claude auth login` puis relance Bestel."
            } else {
                ""
            };
            let msg = format!(
                "claude -p a échoué (code {}){}\n{}",
                status.code().unwrap_or(-1),
                hint,
                err_buf.trim()
            );
            let _ = deltas.send(LlmDelta::Error(msg.clone()));
            return Err(anyhow!(msg));
        }

        let _ = deltas.send(LlmDelta::End);
        Ok(full)
    }
}

fn build_user_prompt(history: &[ChatMessage], _ctx: &BuildContext) -> String {
    let mut s = String::new();
    let take = history.len().saturating_sub(8);
    for m in &history[take..] {
        let speaker = match m.role {
            Role::User => "Exile",
            Role::Assistant => "Bestel",
        };
        s.push_str(&format!("{}: {}\n", speaker, m.content));
    }
    if !s.contains("Bestel:") || !s.trim_end().ends_with(':') {
        s.push_str("Bestel:");
    }
    s
}

#[allow(dead_code)]
fn parse_claude_json_event(line: &str) -> Option<String> {
    let v: Value = serde_json::from_str(line).ok()?;
    let t = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if t == "assistant" {
        let content = v.get("message")?.get("content")?.as_array()?;
        let mut out = String::new();
        for block in content {
            if block.get("type").and_then(|t| t.as_str()) == Some("text") {
                if let Some(s) = block.get("text").and_then(|t| t.as_str()) {
                    out.push_str(s);
                }
            }
        }
        if !out.is_empty() {
            return Some(out);
        }
    }
    if t == "result" {
        if let Some(r) = v.get("result").and_then(|r| r.as_str()) {
            return Some(r.to_string());
        }
    }
    None
}
