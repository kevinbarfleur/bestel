use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use super::spawn::cli_command;
use super::tools::BuildContext;
use super::{ChatMessage, LlmDelta, Role, ToolStatus};
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

        let mut state = CodexState::default();

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
            handle_codex_event(&v, &deltas, &mut state).await;
        }

        // close any reasoning still open
        if state.reasoning_open {
            let _ = deltas.send(LlmDelta::ReasoningEnd);
            state.reasoning_open = false;
        }
        // close any tool still running
        for id in state.running_tools.keys().cloned().collect::<Vec<_>>() {
            let _ = deltas.send(LlmDelta::ToolEnd {
                id,
                status: ToolStatus::Failed,
                summary: None,
            });
        }
        state.running_tools.clear();

        let err_buf = err_task.await.unwrap_or_default();
        let status = child.wait().await?;

        if state.full_text.is_empty() && !status.success() {
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

        let _ = deltas.send(LlmDelta::MessageEnd);
        Ok(state.full_text)
    }
}

#[derive(Default)]
struct CodexState {
    reasoning_open: bool,
    running_tools: HashMap<String, String>, // id → name
    last_reasoning_text: HashMap<String, String>, // item_id → text already sent
    full_text: String,
}

async fn handle_codex_event(
    v: &Value,
    deltas: &mpsc::UnboundedSender<LlmDelta>,
    state: &mut CodexState,
) {
    let outer = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match outer {
        "thread.started" | "turn.started" | "turn.completed" => return,
        _ => {}
    }

    if !matches!(outer, "item.started" | "item.updated" | "item.completed") {
        return;
    }

    let item = match v.get("item") {
        Some(i) => i,
        None => return,
    };
    let item_type = item.get("type").and_then(|t| t.as_str()).unwrap_or("");
    let item_id = item
        .get("id")
        .and_then(|t| t.as_str())
        .unwrap_or("")
        .to_string();

    match item_type {
        "agent_message" => {
            if outer == "item.completed" {
                if let Some(text) = item.get("text").and_then(|t| t.as_str()) {
                    if !text.is_empty() {
                        state.full_text.push_str(text);
                        pseudo_stream(deltas, text).await;
                    }
                }
            } else if outer == "item.updated" {
                if let Some(delta) = item.get("delta").and_then(|d| d.as_str()) {
                    if !delta.is_empty() {
                        state.full_text.push_str(delta);
                        let _ = deltas.send(LlmDelta::TextDelta(delta.to_string()));
                    }
                }
            }
        }
        "reasoning" => {
            if outer == "item.started" {
                if !state.reasoning_open {
                    let _ = deltas.send(LlmDelta::ReasoningBegin);
                    state.reasoning_open = true;
                }
            }
            if outer == "item.updated" || outer == "item.completed" {
                let text = item
                    .get("text")
                    .and_then(|t| t.as_str())
                    .or_else(|| item.get("summary").and_then(|t| t.as_str()))
                    .unwrap_or("");
                if !text.is_empty() {
                    if !state.reasoning_open {
                        let _ = deltas.send(LlmDelta::ReasoningBegin);
                        state.reasoning_open = true;
                    }
                    let already = state
                        .last_reasoning_text
                        .entry(item_id.clone())
                        .or_default();
                    if text.starts_with(already.as_str()) {
                        let new_part: String = text[already.len()..].to_string();
                        if !new_part.is_empty() {
                            let _ = deltas.send(LlmDelta::ReasoningDelta(new_part));
                            *already = text.to_string();
                        }
                    } else {
                        // text was rewritten — emit full as new chunk
                        let _ = deltas.send(LlmDelta::ReasoningDelta(text.to_string()));
                        *already = text.to_string();
                    }
                }
            }
            if outer == "item.completed" && state.reasoning_open {
                let _ = deltas.send(LlmDelta::ReasoningEnd);
                state.reasoning_open = false;
                state.last_reasoning_text.remove(&item_id);
            }
        }
        "command_execution" | "command_executed" | "shell_command" => {
            let cmd_str = item
                .get("command")
                .and_then(|c| c.as_str())
                .or_else(|| item.get("name").and_then(|c| c.as_str()))
                .unwrap_or("commande");
            handle_tool_lifecycle(
                outer,
                &item_id,
                "shell",
                Some(first_line_summary(cmd_str, 80)),
                item,
                deltas,
                state,
            );
        }
        "function_call" | "tool_call" => {
            let name = item
                .get("name")
                .and_then(|c| c.as_str())
                .unwrap_or("tool")
                .to_string();
            handle_tool_lifecycle(outer, &item_id, &name, None, item, deltas, state);
        }
        "web_search" | "web_fetch" => {
            let detail = item
                .get("query")
                .and_then(|c| c.as_str())
                .or_else(|| item.get("url").and_then(|c| c.as_str()))
                .map(|s| first_line_summary(s, 80));
            handle_tool_lifecycle(outer, &item_id, "web", detail, item, deltas, state);
        }
        _ => {}
    }
}

fn handle_tool_lifecycle(
    outer: &str,
    id: &str,
    name: &str,
    detail: Option<String>,
    item: &Value,
    deltas: &mpsc::UnboundedSender<LlmDelta>,
    state: &mut CodexState,
) {
    match outer {
        "item.started" => {
            state
                .running_tools
                .insert(id.to_string(), name.to_string());
            let _ = deltas.send(LlmDelta::ToolBegin {
                id: id.to_string(),
                name: name.to_string(),
                detail,
            });
        }
        "item.updated" => {
            if let Some(out) = item.get("output").and_then(|o| o.as_str()) {
                let snippet = first_line_summary(out, 200);
                if !snippet.is_empty() {
                    let _ = deltas.send(LlmDelta::ToolOutput {
                        id: id.to_string(),
                        chunk: snippet,
                    });
                }
            }
        }
        "item.completed" => {
            let status = item
                .get("status")
                .and_then(|s| s.as_str())
                .map(|s| {
                    if s.eq_ignore_ascii_case("failed") || s.eq_ignore_ascii_case("error") {
                        ToolStatus::Failed
                    } else {
                        ToolStatus::Done
                    }
                })
                .unwrap_or(ToolStatus::Done);
            let summary = item
                .get("summary")
                .and_then(|s| s.as_str())
                .or_else(|| item.get("output").and_then(|s| s.as_str()))
                .map(|s| first_line_summary(s, 100));
            state.running_tools.remove(id);
            let _ = deltas.send(LlmDelta::ToolEnd {
                id: id.to_string(),
                status,
                summary,
            });
        }
        _ => {}
    }
}

async fn pseudo_stream(deltas: &mpsc::UnboundedSender<LlmDelta>, text: &str) {
    // Simulate token streaming when codex hands us the full message at once.
    // ~80 chars / sec → chunk of 4 chars every 50ms.
    let chunk_size = 4usize;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let end = (i + chunk_size).min(chars.len());
        let chunk: String = chars[i..end].iter().collect();
        if deltas.send(LlmDelta::TextDelta(chunk)).is_err() {
            return;
        }
        i = end;
        tokio::time::sleep(Duration::from_millis(50)).await;
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
