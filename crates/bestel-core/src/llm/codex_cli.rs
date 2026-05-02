use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::process::Command;
use tokio::sync::mpsc;

use super::tools::BuildContext;
use super::{ChatMessage, LlmDelta, Role};
use crate::prompt::SYSTEM_PROMPT;

pub struct CodexCliClient {
    version: String,
}

impl CodexCliClient {
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
        let prompt = build_full_prompt(&history, &ctx);

        let mut cmd = Command::new("codex");
        cmd.arg("exec")
            .arg("--json")
            .arg("--skip-git-repo-check")
            .arg("--sandbox")
            .arg("read-only")
            .arg("--color")
            .arg("never")
            .arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let mut child = cmd
            .spawn()
            .map_err(|e| anyhow!("failed to spawn codex CLI: {}", e))?;

        if let Some(mut stdin) = child.stdin.take() {
            stdin.write_all(prompt.as_bytes()).await?;
            stdin.shutdown().await.ok();
            drop(stdin);
        }

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("codex stdout missing"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("codex stderr missing"))?;

        let mut reader = BufReader::new(stdout).lines();
        let mut err_reader = BufReader::new(stderr).lines();

        let mut full = String::new();
        let mut pushed_any = false;

        let mut err_buf = String::new();
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

        while let Some(line) = reader.next_line().await? {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let v: Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => continue,
            };

            if let Some(msg) = extract_codex_text(&v) {
                if !msg.is_empty() {
                    pushed_any = true;
                    full.push_str(&msg);
                    let _ = deltas.send(LlmDelta::Text(msg));
                }
            }
            if let Some(name) = extract_codex_tool_call(&v) {
                let _ = deltas.send(LlmDelta::ToolCall { name });
            }
            if is_codex_error(&v) {
                let msg = v
                    .get("msg")
                    .and_then(|m| m.get("message"))
                    .and_then(|m| m.as_str())
                    .or_else(|| v.get("message").and_then(|m| m.as_str()))
                    .unwrap_or("codex error")
                    .to_string();
                let _ = deltas.send(LlmDelta::Error(msg.clone()));
            }
        }

        if let Ok(s) = err_task.await {
            err_buf = s;
        }

        let status = child.wait().await?;

        if !pushed_any && !status.success() {
            let hint = if err_buf.contains("not authenticated")
                || err_buf.contains("login")
                || err_buf.to_lowercase().contains("auth")
            {
                " — connecte-toi avec `codex login` puis relance Bestel."
            } else {
                ""
            };
            let msg = format!(
                "codex exec a échoué (code {}){}\n{}",
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

fn extract_codex_text(v: &Value) -> Option<String> {
    let outer_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    if outer_type == "item.completed" || outer_type == "item.updated" {
        if let Some(item) = v.get("item") {
            let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
            if item_type == "agent_message" {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    if !text.is_empty() {
                        return Some(text.to_string());
                    }
                }
            }
        }
    }

    let body = v.get("msg").or_else(|| v.get("event")).unwrap_or(v);
    let kind = body
        .get("type")
        .and_then(|t| t.as_str())
        .or(Some(outer_type))
        .unwrap_or("");

    match kind {
        "agent_message_delta" | "message_delta" | "assistant_delta" | "delta" => body
            .get("delta")
            .and_then(|d| d.as_str())
            .or_else(|| body.get("text").and_then(|d| d.as_str()))
            .map(|s| s.to_string()),
        "agent_message" | "message" => body
            .get("text")
            .and_then(|d| d.as_str())
            .or_else(|| body.get("message").and_then(|d| d.as_str()))
            .or_else(|| body.get("content").and_then(|d| d.as_str()))
            .map(|s| s.to_string()),
        _ => None,
    }
}

fn extract_codex_tool_call(v: &Value) -> Option<String> {
    let body = v.get("msg").or_else(|| v.get("event")).unwrap_or(v);
    let kind = body
        .get("type")
        .and_then(|t| t.as_str())
        .or_else(|| v.get("type").and_then(|t| t.as_str()))
        .unwrap_or("");
    if kind.contains("tool") && (kind.contains("call") || kind.contains("started")) {
        let name = body
            .get("name")
            .or_else(|| body.get("tool"))
            .or_else(|| body.get("command"))
            .and_then(|n| n.as_str())
            .unwrap_or("tool");
        return Some(name.to_string());
    }
    None
}

fn is_codex_error(v: &Value) -> bool {
    let outer = v.get("type").and_then(|t| t.as_str()).unwrap_or("");
    if outer == "error" || outer.ends_with(".error") || outer.ends_with("_error") {
        return true;
    }
    let body = v.get("msg").or_else(|| v.get("event")).unwrap_or(v);
    let kind = body.get("type").and_then(|t| t.as_str()).unwrap_or("");
    kind == "error" || kind.ends_with("_error")
}

fn build_full_prompt(history: &[ChatMessage], ctx: &BuildContext) -> String {
    let mut s = String::new();
    s.push_str("[BESTEL — PERSONA INSTRUCTIONS]\n");
    s.push_str(SYSTEM_PROMPT);
    s.push_str("\n\n[CURRENT PATH OF BUILDING DATA]\n");
    s.push_str(
        "Below is the live build the user has open in Path of Building. Use it as ground truth. \
         You do NOT need any tool to access this — it is already provided.\n\n",
    );
    s.push_str(&ctx.render_tool_result());
    s.push_str("\n\n[INSTRUCTIONS]\n");
    s.push_str(
        "- Stay in character as Bestel.\n\
         - Answer in the user's language (French by default).\n\
         - Do not modify any files. Do not run shell commands. Just answer.\n\
         - Quote concrete numbers from the build above when relevant.\n\n",
    );
    s.push_str("[CONVERSATION]\n");
    let take = history.len().saturating_sub(8);
    for m in &history[take..] {
        let speaker = match m.role {
            Role::User => "Exile",
            Role::Assistant => "Bestel",
        };
        s.push_str(&format!("{}: {}\n", speaker, m.content));
    }
    s.push_str("Bestel:");
    s
}
