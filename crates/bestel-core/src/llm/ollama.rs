//! Ollama provider — connects to a locally running Ollama daemon for fully
//! offline / free inference. Uses the `/api/chat` endpoint with NDJSON
//! streaming and OpenAI-style `tools` arrays.
//!
//! Tool support depends on the model: `llama3.1`, `llama3.2`, `qwen2.5`,
//! `mistral-nemo`, `mixtral`, `command-r`, etc. ship with native tool
//! calling. Models without tool calling will simply answer without
//! invoking `get_active_build` / `find_synergies` — to keep parity with
//! the CLI providers, we **also inline the active build into the system
//! prompt**, so even tool-blind models see the exile's character.

use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use super::tools::{dispatch, tool_schemas, BuildContext, ToolCtx};
use super::{ChatMessage, LlmDelta, Role, ToolStatus};
use crate::devlog;
use crate::prompt::SYSTEM_PROMPT_COMPOSED;

const DEFAULT_HOST: &str = "http://localhost:11434";

pub struct OllamaClient {
    host: String,
    model: String,
    http: reqwest::Client,
}

impl OllamaClient {
    pub fn from_env() -> Result<Self> {
        let host = std::env::var("OLLAMA_HOST")
            .ok()
            .filter(|s| !s.is_empty())
            .unwrap_or_else(|| DEFAULT_HOST.to_string());
        let model = super::models::resolved_model_for(super::models::ProviderKind::Ollama)
            .unwrap_or_else(|| "llama3.1:8b".to_string());
        Ok(Self {
            host,
            model,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()?,
        })
    }

    pub fn with(host: String, model: String) -> Self {
        Self {
            host,
            model,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(300))
                .build()
                .expect("reqwest client"),
        }
    }

    pub fn host(&self) -> &str {
        &self.host
    }

    pub fn model(&self) -> &str {
        &self.model
    }

    pub async fn run(
        &self,
        history: Vec<ChatMessage>,
        ctx: BuildContext,
        deltas: mpsc::UnboundedSender<LlmDelta>,
    ) -> Result<String> {
        let tool_ctx = ToolCtx::new(ctx.clone()).context("build ToolCtx")?;
        let tools = anthropic_tools_to_openai(tool_schemas());

        // Inline the active build inside the system prompt for the benefit
        // of tool-blind local models. Tool-capable models will call
        // `get_active_build` and overwrite this with the same data, so the
        // overhead is small. We also append a short instruction to lean on
        // tool calls when available.
        let build_block = ctx.render_tool_result();
        let system_prompt = format!(
            "{persona}\n\n[CURRENT PATH OF BUILDING DATA]\n{build}\n\n\
             [Tool guidance] Two helper tools are available: `get_active_build` (returns the build above as JSON) and `find_synergies(topic)` (queries the wiki reverse-link index). Call them when useful, but do not block on tool calls if your model does not support them.",
            persona = SYSTEM_PROMPT_COMPOSED,
            build = build_block,
        );

        let mut messages: Vec<Value> = Vec::new();
        messages.push(json!({"role": "system", "content": system_prompt}));
        for m in &history {
            let role = match m.role {
                Role::User => "user",
                Role::Assistant => "assistant",
            };
            // Ollama vision models accept an `images` array of base64 strings
            // alongside `content`. Non-vision models silently ignore it.
            let mut images: Vec<Value> = Vec::new();
            for att in &m.attachments {
                if att.is_image() {
                    images.push(Value::String(att.data_base64.clone()));
                }
            }
            let mut msg = serde_json::Map::new();
            msg.insert("role".into(), json!(role));
            msg.insert("content".into(), json!(m.content));
            if !images.is_empty() {
                msg.insert("images".into(), Value::Array(images));
            }
            messages.push(Value::Object(msg));
        }

        let mut full_assistant = String::new();
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > 8 {
                let _ = deltas.send(LlmDelta::Error("agent loop limit reached".into()));
                break;
            }

            let body = json!({
                "model": self.model,
                "messages": messages,
                "tools": tools,
                "stream": true,
                "options": {
                    "num_ctx": 16384,
                }
            });

            let url = format!("{}/api/chat", self.host.trim_end_matches('/'));
            let resp = self
                .http
                .post(&url)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await
                .with_context(|| format!("POST {url}"))?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                let msg = format!("Ollama HTTP {status}: {text}");
                let _ = deltas.send(LlmDelta::Error(msg.clone()));
                return Err(anyhow!(msg));
            }

            let mut stream = resp.bytes_stream();
            let mut buf = String::new();
            let mut assistant_text = String::new();
            let mut accumulated_tool_calls: Vec<Value> = Vec::new();
            let mut done_reason: Option<String> = None;

            while let Some(chunk) = stream.next().await {
                let bytes = chunk?;
                buf.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(pos) = buf.find('\n') {
                    let raw_line = buf[..pos].to_string();
                    buf.drain(..pos + 1);
                    let line = raw_line.trim();
                    if line.is_empty() {
                        continue;
                    }

                    devlog::log_provider_raw("ollama", line);

                    let v: Value = match serde_json::from_str(line) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };

                    if let Some(err) = v.get("error").and_then(|e| e.as_str()) {
                        let msg = format!("Ollama error: {err}");
                        let _ = deltas.send(LlmDelta::Error(msg.clone()));
                        return Err(anyhow!(msg));
                    }

                    if let Some(message) = v.get("message") {
                        if let Some(text) = message.get("content").and_then(|c| c.as_str()) {
                            if !text.is_empty() {
                                assistant_text.push_str(text);
                                let _ = deltas.send(LlmDelta::TextDelta(text.to_string()));
                            }
                        }
                        if let Some(calls) =
                            message.get("tool_calls").and_then(|c| c.as_array())
                        {
                            for c in calls {
                                accumulated_tool_calls.push(c.clone());
                            }
                        }
                    }

                    if v.get("done").and_then(|d| d.as_bool()).unwrap_or(false) {
                        done_reason = v
                            .get("done_reason")
                            .and_then(|s| s.as_str())
                            .map(|s| s.to_string());
                    }
                }
            }

            full_assistant.push_str(&assistant_text);

            // Build the assistant message we will append for the next turn.
            let mut assistant_msg = serde_json::Map::new();
            assistant_msg.insert("role".into(), json!("assistant"));
            assistant_msg.insert("content".into(), json!(assistant_text));
            if !accumulated_tool_calls.is_empty() {
                assistant_msg.insert(
                    "tool_calls".into(),
                    Value::Array(accumulated_tool_calls.clone()),
                );
            }
            messages.push(Value::Object(assistant_msg));

            if accumulated_tool_calls.is_empty() {
                // Either the model is done, or it produced text without
                // calling tools. Either way: stop the loop.
                let _ = done_reason;
                break;
            }

            // Dispatch each tool call and append the result as a tool message.
            for call in &accumulated_tool_calls {
                let function = call.get("function").cloned().unwrap_or(Value::Null);
                let name = function
                    .get("name")
                    .and_then(|s| s.as_str())
                    .unwrap_or("")
                    .to_string();
                // Ollama returns `arguments` as an object directly (not as a
                // JSON-encoded string the way OpenAI Chat does). Handle both.
                let arguments = match function.get("arguments") {
                    Some(Value::Object(_)) => function.get("arguments").cloned().unwrap_or(json!({})),
                    Some(Value::String(s)) => serde_json::from_str(s).unwrap_or(json!({})),
                    _ => json!({}),
                };

                // Synthesise a stable id so the UI tool card can pair begin/end.
                let id = call
                    .get("id")
                    .and_then(|s| s.as_str())
                    .map(|s| s.to_string())
                    .unwrap_or_else(|| format!("ollama-{}-{}", name, full_assistant.len()));
                let _ = deltas.send(LlmDelta::ToolBegin {
                    id: id.clone(),
                    name: name.clone(),
                    detail: None,
                });

                let (result, status) = match dispatch(&name, &arguments, &tool_ctx).await {
                    Ok(s) => (s, ToolStatus::Done),
                    Err(e) => (
                        json!({"error": e.to_string()}).to_string(),
                        ToolStatus::Failed,
                    ),
                };
                crate::devlog::log_value(
                    "tool_call",
                    json!({
                        "name": name,
                        "input": arguments,
                        "status": match status {
                            ToolStatus::Done => "done",
                            ToolStatus::Failed => "failed",
                            ToolStatus::Running => "running",
                        },
                        "result_bytes": result.len(),
                    }),
                );
                let _ = deltas.send(LlmDelta::ToolEnd {
                    id: id.clone(),
                    status,
                    summary: None,
                });

                messages.push(json!({
                    "role": "tool",
                    "name": name,
                    "content": result,
                }));
            }

            // Continue the loop: the model now sees the tool results and
            // produces the final answer (or another round of tool calls).
        }

        let _ = deltas.send(LlmDelta::MessageEnd);
        Ok(full_assistant)
    }
}

/// Convert Anthropic-style tool schemas to OpenAI / Ollama format.
/// Anthropic ships `{name, description, input_schema}`; OpenAI / Ollama
/// expect `{type: "function", function: {name, description, parameters}}`.
fn anthropic_tools_to_openai(schemas: Vec<Value>) -> Vec<Value> {
    schemas
        .into_iter()
        .map(|s| {
            let name = s.get("name").cloned().unwrap_or(Value::Null);
            let description = s.get("description").cloned().unwrap_or(Value::Null);
            let parameters = s
                .get("input_schema")
                .cloned()
                .unwrap_or_else(|| json!({"type": "object", "properties": {}}));
            json!({
                "type": "function",
                "function": {
                    "name": name,
                    "description": description,
                    "parameters": parameters,
                }
            })
        })
        .collect()
}

/// Probe an Ollama daemon by hitting `/api/tags`. Returns the list of
/// installed model names if reachable, `None` otherwise. Times out fast so
/// the startup probe doesn't block the UI when nothing is listening.
pub async fn probe_models(host: Option<&str>) -> Option<Vec<String>> {
    let host = host.unwrap_or(DEFAULT_HOST);
    let url = format!("{}/api/tags", host.trim_end_matches('/'));
    let client = reqwest::Client::builder()
        .timeout(std::time::Duration::from_millis(1500))
        .build()
        .ok()?;
    let resp = client.get(&url).send().await.ok()?;
    if !resp.status().is_success() {
        return None;
    }
    let body: Value = resp.json().await.ok()?;
    let arr = body.get("models")?.as_array()?;
    let mut names: Vec<String> = Vec::new();
    for m in arr {
        if let Some(n) = m.get("name").and_then(|s| s.as_str()) {
            names.push(n.to_string());
        }
    }
    Some(names)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn schema_conversion_wraps_input_schema() {
        let raw = vec![json!({
            "name": "noop",
            "description": "test",
            "input_schema": {"type": "object", "properties": {}}
        })];
        let openai = anthropic_tools_to_openai(raw);
        assert_eq!(openai.len(), 1);
        assert_eq!(openai[0]["type"], "function");
        assert_eq!(openai[0]["function"]["name"], "noop");
        assert_eq!(openai[0]["function"]["parameters"]["type"], "object");
    }
}
