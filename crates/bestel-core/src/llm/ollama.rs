//! Ollama provider — connects to a locally running Ollama daemon for fully
//! offline / free inference. Uses the `/api/chat` endpoint with NDJSON
//! streaming and OpenAI-style `tools` arrays.
//!
//! Tool support depends on the model: `llama3.1`, `llama3.2`, `qwen2.5`,
//! `mistral-nemo`, `mixtral`, `command-r`, etc. ship with native tool
//! calling. Models without tool calling will simply answer without
//! invoking the helpers — to keep parity with the CLI providers, we
//! **also inline the active build into the system prompt**, so even
//! tool-blind models see the exile's character.
//!
//! With Phase G the in-app tool surface (the same one Anthropic API
//! uses) gives Ollama models real wiki / trade / web research:
//! `get_active_build`, `wiki_search`, `wiki_parse`, `wiki_cargo`,
//! `wiki_synergies`, `trade_resolve_stats`, `trade_search_url`, and
//! `web_fetch` (allowlisted to tier-1–7 hosts). The inline guidance
//! below nudges the model toward calling them aggressively.

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
        // Local 7-8B models drown in 8 tools — drop the niche ones (cargo,
        // trade pair, find_synergies legacy alias) to reduce selection
        // confusion. Anthropic / MCP keep the full surface untouched.
        let tools = anthropic_tools_to_openai(local_friendly_tools(tool_schemas()));

        // Inline the active build inside the system prompt for the benefit
        // of tool-blind local models. Tool-capable models will call
        // `get_active_build` and overwrite this with the same data.
        let build_block = ctx.render_tool_result();
        let system_prompt = format!(
            "{persona}\n\n[CURRENT PATH OF BUILDING DATA]\n{build}\n\n{LOCAL_ADDENDUM}",
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
        let mut wrap_up_done = false;

        loop {
            iterations += 1;
            // Same wrap-up policy as Anthropic: 16 tool rounds, then one
            // final pass with tools empty + a nudge so the local model has
            // to produce a final answer from what it gathered. See
            // anthropic.rs::MAX_AGENT_ITERATIONS for rationale.
            let force_finalize = iterations > 16;
            if force_finalize {
                if wrap_up_done {
                    break;
                }
                wrap_up_done = true;
                messages.push(json!({
                    "role": "user",
                    "content": "[system: tool budget reached. Give your final answer now using what you've already gathered. Do not request more tools.]",
                }));
            }

            let body = json!({
                "model": self.model,
                "messages": messages,
                "tools": if force_finalize { json!([]) } else { json!(tools) },
                "stream": true,
                "options": {
                    // 8K context: balance between fitting SYSTEM_PROMPT +
                    // CORE_KNOWLEDGE + a few turns of history, and keeping the
                    // KV cache small enough that Ollama's allocator (notably on
                    // Windows + qwen3 family) can secure a contiguous chunk.
                    // 16K and above triggers `memory layout cannot be allocated`
                    // failures on Ollama 0.23.x for qwen3:8b on a 32 GB machine
                    // — even with plenty of free RAM.
                    "num_ctx": 8192,
                    // 4K output cap so the model can write a thorough
                    // multi-paragraph answer without being truncated mid-thought.
                    "num_predict": 4096,
                    // Low-but-not-zero temperature: factual analysis benefits
                    // from determinism; pure 0 can produce repetitive / stuck
                    // outputs on small models. 0.3 is the sweet spot for
                    // tool-using research agents.
                    "temperature": 0.3,
                    // Tight nucleus sampling — discourages the model from
                    // wandering into rare-token hallucinations (the typical
                    // source of invented item / mod names).
                    "top_p": 0.9,
                    // Mild repetition penalty — small models love to repeat
                    // bullet headers and recap the same items twice.
                    "repeat_penalty": 1.1,
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
                // Surface the tool args as the delta detail so the UI can
                // show "wiki_parse(title=Resolute Technique)" instead of
                // just a bare badge. Mirrors the visibility Anthropic
                // already gives via its native tool_use blocks.
                let detail = summarize_tool_args(&arguments);
                let _ = deltas.send(LlmDelta::ToolBegin {
                    id: id.clone(),
                    name: name.clone(),
                    detail,
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
                let summary = summarize_tool_result(&result, status == ToolStatus::Failed);
                let _ = deltas.send(LlmDelta::ToolEnd {
                    id: id.clone(),
                    status,
                    summary,
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

/// Format the tool call arguments as a compact human-readable detail
/// string for the delta stream. Used to give the UI parity with
/// Anthropic which surfaces `tool_use.input` natively.
fn summarize_tool_args(args: &Value) -> Option<String> {
    let obj = args.as_object()?;
    if obj.is_empty() {
        return None;
    }
    let mut parts: Vec<String> = Vec::new();
    for (k, v) in obj.iter() {
        let val_str = match v {
            Value::String(s) => {
                let s = s.trim();
                if s.len() > 60 {
                    format!("\"{}…\"", &s[..59])
                } else {
                    format!("\"{s}\"")
                }
            }
            Value::Number(n) => n.to_string(),
            Value::Bool(b) => b.to_string(),
            Value::Null => "null".to_string(),
            other => {
                let s = other.to_string();
                if s.len() > 60 {
                    format!("{}…", &s[..59])
                } else {
                    s
                }
            }
        };
        parts.push(format!("{k}={val_str}"));
        if parts.len() == 3 {
            break;
        }
    }
    Some(parts.join(", "))
}

/// Compress a tool dispatch result into a short summary line for the
/// delta stream. Errors are passed through verbatim (they're already
/// short JSON); successes are truncated to ~200 chars of the first
/// non-empty line. The full result still flows into the conversation
/// history — this is purely a UI hint.
fn summarize_tool_result(result: &str, is_error: bool) -> Option<String> {
    if result.is_empty() {
        return None;
    }
    if is_error {
        let trimmed = result.trim();
        if trimmed.len() > 200 {
            return Some(format!("{}…", &trimmed[..199]));
        }
        return Some(trimmed.to_string());
    }
    // Try to extract a single readable line. JSON results look like
    // `{"title":"...","sections":[...]}` — strip braces and quotes for a
    // cleaner one-liner.
    let trimmed = result.trim();
    let first_line = trimmed.lines().next().unwrap_or(trimmed);
    let cleaned: String = first_line
        .chars()
        .map(|c| if c == '\n' || c == '\t' { ' ' } else { c })
        .collect();
    let cleaned = cleaned.trim();
    if cleaned.is_empty() {
        return None;
    }
    if cleaned.len() > 200 {
        Some(format!("{}…", &cleaned[..199]))
    } else {
        Some(cleaned.to_string())
    }
}

/// Tools we expose to local 7-8B models. SLM literature is unanimous
/// (cf. arxiv 2510.03847): fewer focused tools beat broader surfaces on
/// small models because tool-selection accuracy collapses with similar-
/// sounding alternatives. Anthropic and the MCP server still see the
/// full `tool_schemas()` surface — this filter is local-only.
const LOCAL_TOOL_ALLOWLIST: &[&str] = &[
    "get_active_build",
    "wiki_search",
    "wiki_parse",
    "wiki_synergies",
    "web_fetch",
];

/// Drop tools not in [`LOCAL_TOOL_ALLOWLIST`] from the schema list. The
/// dispatch path (`crate::llm::tools::dispatch`) still recognises every
/// tool — we just don't advertise the niche ones to the local model.
fn local_friendly_tools(schemas: Vec<Value>) -> Vec<Value> {
    schemas
        .into_iter()
        .filter(|s| {
            s.get("name")
                .and_then(|n| n.as_str())
                .map(|n| LOCAL_TOOL_ALLOWLIST.contains(&n))
                .unwrap_or(false)
        })
        .collect()
}

/// Local-model addendum to the base SYSTEM_PROMPT_COMPOSED. Strengthens
/// the 5 things small 7-8B models slip on relative to Sonnet 4.5:
/// first-person voice, in-character refusal, escape hatch on
/// get_active_build when inline data is sufficient, PoE2 verification
/// reflex, and clarification gate on vague prompts. Final `/no_think`
/// flag is for qwen3 family — issue QwenLM/Qwen3#1817 documents that
/// thinking mode + tools = ~60% fabricated tool calls; non-thinking
/// mode → 100% execution.
const LOCAL_ADDENDUM: &str = include_str!("./local_addendum.md");

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
    probe_models_detailed(host)
        .await
        .map(|infos| infos.into_iter().map(|i| i.name).collect())
}

/// Richer view of an installed Ollama model. Populated from the same
/// `/api/tags` response, with the optional `details.parameter_size` /
/// `details.family` fields surfaced when present.
#[derive(Debug, Clone)]
pub struct OllamaModelInfo {
    /// Full tag, e.g. `qwen3:8b`. Pass directly to `/api/chat` `model`.
    pub name: String,
    /// On-disk size in bytes (uncompressed weights).
    pub size_bytes: Option<u64>,
    /// Parameter size hint, e.g. `"8.0B"`, `"30B"`. Source: `details.parameter_size`.
    pub parameter_size: Option<String>,
    /// Family name, e.g. `"qwen3"`, `"llama"`. Source: `details.family`.
    pub family: Option<String>,
}

/// Detailed variant of [`probe_models`]. Returns each installed model with
/// the metadata Ollama exposes via `/api/tags` so the model picker can
/// build human-readable labels and rough speed/heaviness hints without a
/// second roundtrip.
pub async fn probe_models_detailed(host: Option<&str>) -> Option<Vec<OllamaModelInfo>> {
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
    let mut out: Vec<OllamaModelInfo> = Vec::new();
    for m in arr {
        let Some(name) = m.get("name").and_then(|s| s.as_str()) else {
            continue;
        };
        let size_bytes = m.get("size").and_then(|v| v.as_u64());
        let details = m.get("details");
        let parameter_size = details
            .and_then(|d| d.get("parameter_size"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        let family = details
            .and_then(|d| d.get("family"))
            .and_then(|s| s.as_str())
            .map(|s| s.to_string());
        out.push(OllamaModelInfo {
            name: name.to_string(),
            size_bytes,
            parameter_size,
            family,
        });
    }
    Some(out)
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
