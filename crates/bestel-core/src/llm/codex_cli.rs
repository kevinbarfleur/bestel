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
use crate::devlog;
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
            .arg(format!(
                "model_reasoning_summary={}",
                std::env::var("BESTEL_REASONING_SUMMARY")
                    .unwrap_or_else(|_| "detailed".to_string())
            ));

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
            devlog::log_provider_raw("codex", trimmed);
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

    // Catch-all: anything that looks like a tool/function/search/exec call
    // gets surfaced even if we don't recognize the exact item type, so the
    // user always sees what Bestel is doing.
    let looks_like_tool = item_type.contains("call")
        || item_type.contains("execution")
        || item_type.contains("command")
        || item_type.contains("search")
        || item_type.contains("fetch")
        || item_type.contains("browse")
        || item_type.contains("navigate")
        || item_type.contains("tool")
        || item_type.contains("function");

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
            let cmd_str = extract_first_str(
                item,
                &["command", "cmd", "script", "name", "args"],
            )
            .unwrap_or_else(|| "commande".into());
            handle_tool_lifecycle(
                outer,
                &item_id,
                "shell",
                Some(first_line_summary(&cmd_str, 80)),
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
            let detail = extract_first_str(
                item,
                &["arguments", "args", "input", "params", "query", "url"],
            );
            handle_tool_lifecycle(outer, &item_id, &name, detail, item, deltas, state);
        }
        "web_search" | "web_fetch" => {
            let label = if item_type == "web_fetch" {
                "fetch"
            } else {
                "web"
            };

            if outer == "item.completed" {
                // Split a batched search ({queries: [q1, q2, q3]}) into one
                // card per query so the activity stream matches what the
                // model actually queried, instead of showing '+N' on a
                // single card. The first query reuses the original tool id
                // (which was opened on item.started); subsequent queries
                // get synthetic ids derived from it.
                let queries = extract_all_queries(item);
                if !queries.is_empty() {
                    state.running_tools.remove(&item_id);
                    let _ = deltas.send(LlmDelta::ToolEnd {
                        id: item_id.clone(),
                        status: ToolStatus::Done,
                        summary: Some(queries[0].clone()),
                    });
                    for (i, q) in queries.iter().skip(1).enumerate() {
                        let synth_id = format!("{}#q{}", item_id, i + 2);
                        let _ = deltas.send(LlmDelta::ToolBegin {
                            id: synth_id.clone(),
                            name: label.to_string(),
                            detail: Some(q.clone()),
                        });
                        // Stagger so the cards appear one after another in
                        // the chat instead of as an instantaneous burst.
                        tokio::time::sleep(Duration::from_millis(120)).await;
                        let _ = deltas.send(LlmDelta::ToolEnd {
                            id: synth_id,
                            status: ToolStatus::Done,
                            summary: None,
                        });
                    }
                    return;
                }
                // No queries[] array — fall through to standard flow.
            }

            let detail = extract_web_search_detail(item).or_else(|| {
                if outer == "item.started" {
                    Some("(searching…)".into())
                } else {
                    None
                }
            });
            handle_tool_lifecycle(outer, &item_id, label, detail, item, deltas, state);
        }
        _ if looks_like_tool => {
            let label_owned = derive_tool_label(item_type, item);
            let detail = extract_first_str(
                item,
                &[
                    "query",
                    "url",
                    "target_url",
                    "search_query",
                    "command",
                    "cmd",
                    "script",
                    "input",
                    "args",
                    "arguments",
                    "params",
                    "text",
                    "name",
                ],
            )
            .or_else(|| extract_url_from_results(item))
            .or_else(|| dump_item_for_detail(item));
            handle_tool_lifecycle(outer, &item_id, &label_owned, detail, item, deltas, state);
        }
        _ => {}
    }
}

fn derive_tool_label(item_type: &str, item: &Value) -> String {
    if let Some(name) = item.get("name").and_then(|n| n.as_str()) {
        if !name.is_empty() {
            return name.to_string();
        }
    }
    let lower = item_type.to_lowercase();
    if lower.contains("web") || lower.contains("search") {
        return "web".into();
    }
    if lower.contains("fetch") || lower.contains("browse") || lower.contains("navigate") {
        return "fetch".into();
    }
    if lower.contains("shell") || lower.contains("command") || lower.contains("execution") {
        return "shell".into();
    }
    if lower.contains("function") || lower.contains("tool") {
        return "tool".into();
    }
    item_type.to_string()
}

fn dump_item_for_detail(item: &Value) -> Option<String> {
    let obj = item.as_object()?;
    let mut compact = serde_json::Map::new();
    for (k, v) in obj {
        // Skip noisy / boilerplate keys.
        if matches!(
            k.as_str(),
            "id" | "type" | "status" | "created" | "ended_at" | "started_at"
        ) {
            continue;
        }
        if v.is_null() {
            continue;
        }
        compact.insert(k.clone(), v.clone());
    }
    if compact.is_empty() {
        return None;
    }
    let s = serde_json::Value::Object(compact).to_string();
    Some(first_line_summary(&s, 120))
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
            // Build a summary that's worth showing in the tool header. Order:
            //   item.summary → item.output → web-search query → JSON dump
            let summary = item
                .get("summary")
                .and_then(|s| s.as_str())
                .map(|s| s.to_string())
                .or_else(|| {
                    item.get("output")
                        .and_then(|s| s.as_str())
                        .map(|s| s.to_string())
                })
                .or_else(|| extract_web_search_detail(item))
                .or_else(|| dump_item_for_detail(item))
                .map(|s| first_line_summary(&s, 140));
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
    // ~120 chars/sec, chunked at word boundaries when possible.
    let chunk_size = 12usize;
    let chars: Vec<char> = text.chars().collect();
    let mut i = 0;
    while i < chars.len() {
        let mut end = (i + chunk_size).min(chars.len());
        // Try to extend to next whitespace to avoid splitting words.
        while end < chars.len() && !chars[end].is_whitespace() && end - i < chunk_size + 8 {
            end += 1;
        }
        let chunk: String = chars[i..end].iter().collect();
        if deltas.send(LlmDelta::TextDelta(chunk)).is_err() {
            return;
        }
        i = end;
        tokio::time::sleep(Duration::from_millis(80)).await;
    }
}

fn extract_first_str(item: &Value, keys: &[&str]) -> Option<String> {
    // First pass: try to find a primitive string in top-level fields.
    for k in keys {
        if let Some(v) = item.get(*k) {
            if let Some(s) = v.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    // If it's a JSON string carrying structured args (common for
                    // OpenAI function calls — arguments = "{\"query\":\"...\"}"),
                    // try to dig into known fields.
                    if let Ok(parsed) = serde_json::from_str::<Value>(trimmed) {
                        if let Some(inner) = first_meaningful_field(&parsed) {
                            return Some(inner);
                        }
                    }
                    return Some(trimmed.to_string());
                }
            } else if v.is_object() {
                if let Some(inner) = first_meaningful_field(v) {
                    return Some(inner);
                }
                let s = v.to_string();
                if !s.is_empty() && s != "null" {
                    return Some(s);
                }
            } else if !v.is_null() {
                let s = v.to_string();
                if !s.is_empty() && s != "null" {
                    return Some(s);
                }
            }
        }
    }
    None
}

fn first_meaningful_field(v: &Value) -> Option<String> {
    let candidates = [
        "url",
        "target_url",
        "query",
        "search_query",
        "q",
        "input",
        "text",
        "command",
        "cmd",
        "script",
    ];
    for k in candidates {
        if let Some(s) = v.get(k).and_then(|x| x.as_str()) {
            let t = s.trim();
            if !t.is_empty() {
                return Some(t.to_string());
            }
        }
    }
    None
}

fn extract_url_from_results(item: &Value) -> Option<String> {
    let results = item.get("results").and_then(|v| v.as_array())?;
    let first = results.first()?;
    if let Some(url) = first.get("url").and_then(|u| u.as_str()) {
        return Some(url.to_string());
    }
    None
}

/// Pull every query out of a Codex web_search item — the `queries[]`
/// array if available (each entry is shown as its own card), else fall
/// back to the singular `action.query` / `query` / `url` fields.
fn extract_all_queries(item: &Value) -> Vec<String> {
    if let Some(arr) = item
        .get("action")
        .and_then(|a| a.get("queries"))
        .and_then(|q| q.as_array())
    {
        let cleaned: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if !cleaned.is_empty() {
            return cleaned;
        }
    }
    if let Some(s) = extract_web_search_detail(item) {
        return vec![s];
    }
    Vec::new()
}

/// Codex 0.128 web_search payload shape:
///   item.query                                  → primary search string
///   item.action = { type, query, queries: [...] }
/// On item.started both query and action.query are typically empty;
/// they are populated on item.completed. We surface the most useful
/// representation: the joined queries (with a "+N" suffix when there
/// are several) so the user always sees what Bestel actually searched.
///
/// IMPORTANT — batched semantics.
/// One `web_search` item = one BATCH of N parallel queries. So a single
/// visible tool card may correspond to several search-engine round-trips.
/// And `web_search` only returns SNIPPETS — the model does not navigate
/// each result URL, it cites them from snippet metadata. URLs that appear
/// in the assistant's `Sources:` list will not necessarily each have a
/// matching tool card; only the queries do.
///
/// TODO: render the queries[] array as expandable body lines so users
///       see all N queries inside a batch, not just the first + count.
/// TODO: when the response payload eventually exposes result URLs (some
///       provider versions include `results[]`), attach them as inline
///       citation children of the search card.
fn extract_web_search_detail(item: &Value) -> Option<String> {
    let action = item.get("action");

    // Prefer the queries array (often holds multiple parallel searches).
    if let Some(arr) = action
        .and_then(|a| a.get("queries"))
        .and_then(|q| q.as_array())
    {
        let cleaned: Vec<String> = arr
            .iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.trim().to_string())
            .filter(|s| !s.is_empty())
            .collect();
        if let Some(first) = cleaned.first() {
            let detail = if cleaned.len() > 1 {
                format!("{} · +{}", first, cleaned.len() - 1)
            } else {
                first.clone()
            };
            return Some(first_line_summary(&detail, 140));
        }
    }

    // Single query field on action or item.
    let getters: [fn(&Value) -> Option<&Value>; 4] = [
        |i| i.get("action").and_then(|a| a.get("query")),
        |i| i.get("query"),
        |i| i.get("url"),
        |i| i.get("action").and_then(|a| a.get("url")),
    ];
    for getter in getters {
        if let Some(v) = getter(item) {
            if let Some(s) = v.as_str() {
                let trimmed = s.trim();
                if !trimmed.is_empty() {
                    return Some(first_line_summary(trimmed, 140));
                }
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
