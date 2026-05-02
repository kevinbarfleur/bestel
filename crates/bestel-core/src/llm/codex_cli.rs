use anyhow::{anyhow, Result};
use serde_json::Value;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use super::spawn::cli_command;
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

        let reasoning =
            std::env::var("BESTEL_REASONING").unwrap_or_else(|_| "low".to_string());

        let mut cmd = cli_command("codex");
        cmd.arg("exec")
            .arg("--json")
            .arg("--skip-git-repo-check")
            .arg("--sandbox")
            .arg("read-only")
            .arg("--color")
            .arg("never")
            .arg("-c")
            .arg(format!("model_reasoning_effort={}", reasoning))
            .arg("-c")
            .arg("model_reasoning_summary=auto");

        if let Ok(model) = std::env::var("BESTEL_MODEL") {
            cmd.arg("-m").arg(model);
        }

        cmd.arg("-")
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .kill_on_drop(true);

        let _ = deltas.send(LlmDelta::Activity("démarre la session…".into()));

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

            handle_codex_event(&v, &deltas, &mut full, &mut pushed_any);
        }

        let err_buf = err_task.await.unwrap_or_default();
        let status = child.wait().await?;

        if !pushed_any && !status.success() {
            let lower = err_buf.to_lowercase();
            let hint = if lower.contains("not authenticated")
                || lower.contains("login")
                || lower.contains("auth")
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

fn handle_codex_event(
    v: &Value,
    deltas: &mpsc::UnboundedSender<LlmDelta>,
    full: &mut String,
    pushed_any: &mut bool,
) {
    let outer = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match outer {
        "thread.started" => {
            let _ = deltas.send(LlmDelta::Activity("session ouverte".into()));
            return;
        }
        "turn.started" => {
            let _ = deltas.send(LlmDelta::Activity("Bestel délie ses pensées".into()));
            return;
        }
        "turn.completed" => {
            return;
        }
        _ => {}
    }

    if matches!(outer, "item.started" | "item.updated" | "item.completed") {
        if let Some(item) = v.get("item") {
            handle_codex_item(outer, item, deltas, full, pushed_any);
        }
        return;
    }
}

fn handle_codex_item(
    event: &str,
    item: &Value,
    deltas: &mpsc::UnboundedSender<LlmDelta>,
    full: &mut String,
    pushed_any: &mut bool,
) {
    let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match item_type {
        "agent_message" => {
            if event == "item.completed" {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    if !text.is_empty() {
                        *pushed_any = true;
                        full.push_str(text);
                        let _ = deltas.send(LlmDelta::Text(text.to_string()));
                    }
                }
            } else if event == "item.updated" {
                if let Some(delta) = item
                    .get("delta")
                    .and_then(|d| d.as_str())
                    .or_else(|| item.get("text").and_then(|d| d.as_str()))
                {
                    if !delta.is_empty() {
                        *pushed_any = true;
                        full.push_str(delta);
                        let _ = deltas.send(LlmDelta::Text(delta.to_string()));
                    }
                }
            }
        }
        "reasoning" => {
            if event == "item.completed" || event == "item.updated" {
                if let Some(text) = item
                    .get("text")
                    .and_then(|t| t.as_str())
                    .or_else(|| item.get("summary").and_then(|t| t.as_str()))
                {
                    let trimmed = text.trim();
                    if !trimmed.is_empty() {
                        let snippet = first_line_summary(trimmed, 80);
                        let _ = deltas.send(LlmDelta::Reasoning(snippet));
                    }
                }
            } else if event == "item.started" {
                let _ = deltas.send(LlmDelta::Activity("Bestel médite".into()));
            }
        }
        "command_execution" | "command_executed" | "shell_command" => {
            let cmd = item
                .get("command")
                .and_then(|c| c.as_str())
                .or_else(|| item.get("name").and_then(|c| c.as_str()))
                .unwrap_or("commande");
            if event == "item.started" {
                let _ = deltas.send(LlmDelta::ToolCall {
                    name: "shell".into(),
                    detail: Some(first_line_summary(cmd, 80)),
                });
            } else if event == "item.completed" {
                let _ = deltas.send(LlmDelta::ToolResult {
                    name: "shell".into(),
                    detail: Some(first_line_summary(cmd, 80)),
                });
            }
        }
        "function_call" | "tool_call" => {
            let name = item
                .get("name")
                .and_then(|c| c.as_str())
                .unwrap_or("tool")
                .to_string();
            if event == "item.started" {
                let _ = deltas.send(LlmDelta::ToolCall {
                    name,
                    detail: None,
                });
            } else if event == "item.completed" {
                let _ = deltas.send(LlmDelta::ToolResult {
                    name,
                    detail: None,
                });
            }
        }
        "web_search" | "web_fetch" => {
            let query = item
                .get("query")
                .and_then(|c| c.as_str())
                .or_else(|| item.get("url").and_then(|c| c.as_str()))
                .unwrap_or("");
            if event == "item.started" {
                let _ = deltas.send(LlmDelta::ToolCall {
                    name: "web".into(),
                    detail: if query.is_empty() {
                        None
                    } else {
                        Some(first_line_summary(query, 80))
                    },
                });
            }
        }
        _ => {}
    }
}

fn first_line_summary(s: &str, max: usize) -> String {
    let line = s.lines().next().unwrap_or(s).trim();
    if line.chars().count() <= max {
        line.to_string()
    } else {
        let mut out: String = line.chars().take(max).collect();
        out.push('…');
        out
    }
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
         - Quote concrete numbers from the build above when relevant.\n\
         - Be concise. A chronicler weighs his words.\n\n",
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
