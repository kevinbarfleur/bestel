use anyhow::{anyhow, Result};
use serde_json::Value;
use std::collections::HashMap;
use std::process::Stdio;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::sync::mpsc;

use super::spawn::cli_command;
use super::tools::BuildContext;
use super::{ChatMessage, LlmDelta, Role, ToolStatus};
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
        let user_prompt = build_user_prompt(&history);
        let system_addon = format!(
            "{}\n\n[CURRENT PATH OF BUILDING DATA]\n{}\n\n[INSTRUCTIONS]\n\
             Stay in character as Bestel. Do not modify files. Do not run shell commands. \
             Answer in the user's language (French by default). \
             Quote concrete numbers from the build above when relevant.",
            SYSTEM_PROMPT,
            ctx.render_tool_result()
        );

        let mut cmd = cli_command("claude");
        cmd.arg("-p")
            .arg("--bare")
            .arg("--output-format")
            .arg("stream-json")
            .arg("--include-partial-messages")
            .arg("--verbose")
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

        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| anyhow!("claude stdout missing"))?;
        let stderr = child
            .stderr
            .take()
            .ok_or_else(|| anyhow!("claude stderr missing"))?;

        let mut reader = BufReader::new(stdout).lines();
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

        let mut state = ClaudeState::default();

        while let Some(line) = reader.next_line().await? {
            let trimmed = line.trim();
            if trimmed.is_empty() {
                continue;
            }
            let v: Value = match serde_json::from_str(trimmed) {
                Ok(v) => v,
                Err(_) => continue,
            };
            handle_claude_event(&v, &deltas, &mut state);
        }

        if state.reasoning_open {
            let _ = deltas.send(LlmDelta::ReasoningEnd);
            state.reasoning_open = false;
        }
        for id in state.running_tools.keys().cloned().collect::<Vec<_>>() {
            let _ = deltas.send(LlmDelta::ToolEnd {
                id,
                status: ToolStatus::Done,
                summary: None,
            });
        }
        state.running_tools.clear();

        let err_buf = err_task.await.unwrap_or_default();
        let status = child.wait().await?;

        if state.full_text.is_empty() && !status.success() {
            let lower = err_buf.to_lowercase();
            let hint = if lower.contains("auth") || lower.contains("login") {
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

        let _ = deltas.send(LlmDelta::MessageEnd);
        Ok(state.full_text)
    }
}

#[derive(Default)]
struct ClaudeState {
    reasoning_open: bool,
    running_tools: HashMap<String, String>,
    full_text: String,
}

fn handle_claude_event(
    v: &Value,
    deltas: &mpsc::UnboundedSender<LlmDelta>,
    state: &mut ClaudeState,
) {
    let outer = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

    match outer {
        "stream_event" => {
            // Inner SSE events from streaming partial messages
            if let Some(ev) = v.get("event") {
                let etype = ev.get("type").and_then(|t| t.as_str()).unwrap_or("");
                handle_inner_stream_event(etype, ev, deltas, state);
            }
        }
        "assistant" => {
            // Final assembled assistant message — we may already have streamed all of it.
            // Used to detect tool_use blocks and guarantee text is captured if streaming was missed.
            if let Some(msg) = v.get("message") {
                if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                    for block in content {
                        let btype = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                        match btype {
                            "tool_use" => {
                                let id = block
                                    .get("id")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let name = block
                                    .get("name")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("tool")
                                    .to_string();
                                if !state.running_tools.contains_key(&id) {
                                    state.running_tools.insert(id.clone(), name.clone());
                                    let _ = deltas.send(LlmDelta::ToolBegin {
                                        id,
                                        name,
                                        detail: None,
                                    });
                                }
                            }
                            "text" => {
                                if let Some(text) =
                                    block.get("text").and_then(|t| t.as_str())
                                {
                                    if state.full_text.is_empty() && !text.is_empty() {
                                        state.full_text.push_str(text);
                                        let _ = deltas
                                            .send(LlmDelta::TextDelta(text.to_string()));
                                    }
                                }
                            }
                            _ => {}
                        }
                    }
                }
            }
        }
        "user" => {
            // tool_result back to model — surface as tool end if matching id
            if let Some(msg) = v.get("message") {
                if let Some(content) = msg.get("content").and_then(|c| c.as_array()) {
                    for block in content {
                        if block.get("type").and_then(|t| t.as_str()) == Some("tool_result") {
                            let id = block
                                .get("tool_use_id")
                                .and_then(|s| s.as_str())
                                .unwrap_or("")
                                .to_string();
                            let is_error = block
                                .get("is_error")
                                .and_then(|b| b.as_bool())
                                .unwrap_or(false);
                            let summary = extract_tool_result_summary(block);
                            state.running_tools.remove(&id);
                            let _ = deltas.send(LlmDelta::ToolEnd {
                                id,
                                status: if is_error {
                                    ToolStatus::Failed
                                } else {
                                    ToolStatus::Done
                                },
                                summary,
                            });
                        }
                    }
                }
            }
        }
        "result" => {
            // Final result event — full result text (if not streamed already).
            if let Some(r) = v.get("result").and_then(|r| r.as_str()) {
                if state.full_text.is_empty() && !r.is_empty() {
                    state.full_text.push_str(r);
                    let _ = deltas.send(LlmDelta::TextDelta(r.to_string()));
                }
            }
        }
        "error" | "system" => {
            // Surface system errors only.
            if outer == "error" {
                let msg = v
                    .get("message")
                    .and_then(|m| m.as_str())
                    .unwrap_or("claude error")
                    .to_string();
                let _ = deltas.send(LlmDelta::Error(msg));
            }
        }
        _ => {}
    }
}

fn handle_inner_stream_event(
    etype: &str,
    ev: &Value,
    deltas: &mpsc::UnboundedSender<LlmDelta>,
    state: &mut ClaudeState,
) {
    match etype {
        "content_block_start" => {
            if let Some(block) = ev.get("content_block") {
                let btype = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
                if btype == "thinking" || btype == "redacted_thinking" {
                    if !state.reasoning_open {
                        let _ = deltas.send(LlmDelta::ReasoningBegin);
                        state.reasoning_open = true;
                    }
                } else if btype == "tool_use" {
                    let id = block
                        .get("id")
                        .and_then(|s| s.as_str())
                        .unwrap_or("")
                        .to_string();
                    let name = block
                        .get("name")
                        .and_then(|s| s.as_str())
                        .unwrap_or("tool")
                        .to_string();
                    if !state.running_tools.contains_key(&id) {
                        state.running_tools.insert(id.clone(), name.clone());
                        let _ = deltas.send(LlmDelta::ToolBegin {
                            id,
                            name,
                            detail: None,
                        });
                    }
                }
            }
        }
        "content_block_delta" => {
            if let Some(d) = ev.get("delta") {
                let dtype = d.get("type").and_then(|t| t.as_str()).unwrap_or("");
                match dtype {
                    "text_delta" => {
                        if let Some(text) = d.get("text").and_then(|t| t.as_str()) {
                            if !text.is_empty() {
                                state.full_text.push_str(text);
                                let _ =
                                    deltas.send(LlmDelta::TextDelta(text.to_string()));
                            }
                        }
                    }
                    "thinking_delta" => {
                        if let Some(text) = d.get("thinking").and_then(|t| t.as_str()) {
                            if !text.is_empty() {
                                if !state.reasoning_open {
                                    let _ = deltas.send(LlmDelta::ReasoningBegin);
                                    state.reasoning_open = true;
                                }
                                let _ = deltas
                                    .send(LlmDelta::ReasoningDelta(text.to_string()));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
        "content_block_stop" => {
            if state.reasoning_open {
                let _ = deltas.send(LlmDelta::ReasoningEnd);
                state.reasoning_open = false;
            }
        }
        _ => {}
    }
}

fn extract_tool_result_summary(block: &Value) -> Option<String> {
    if let Some(s) = block.get("content").and_then(|c| c.as_str()) {
        return Some(first_line_summary(s, 100));
    }
    if let Some(arr) = block.get("content").and_then(|c| c.as_array()) {
        for entry in arr {
            if let Some(s) = entry.get("text").and_then(|t| t.as_str()) {
                return Some(first_line_summary(s, 100));
            }
        }
    }
    None
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

fn build_user_prompt(history: &[ChatMessage]) -> String {
    let mut s = String::new();
    let take = history.len().saturating_sub(8);
    for m in &history[take..] {
        let speaker = match m.role {
            Role::User => "Exile",
            Role::Assistant => "Bestel",
        };
        s.push_str(&format!("{}: {}\n", speaker, m.content));
    }
    if !s.trim_end().ends_with(':') {
        s.push_str("Bestel:");
    }
    s
}
