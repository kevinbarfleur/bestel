use anyhow::{anyhow, Context, Result};
use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use super::tools::{tool_schema, BuildContext, GET_ACTIVE_BUILD};
use super::{ChatMessage, LlmDelta, Role};
use crate::prompt::SYSTEM_PROMPT;

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";

pub struct AnthropicClient {
    api_key: String,
    model: String,
    http: reqwest::Client,
}

impl AnthropicClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var("ANTHROPIC_API_KEY")
            .context("ANTHROPIC_API_KEY environment variable not set")?;
        let model = std::env::var("BESTEL_MODEL")
            .unwrap_or_else(|_| "claude-sonnet-4-5-20250929".to_string());
        Ok(Self {
            api_key,
            model,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()?,
        })
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
        let mut messages: Vec<Value> = history
            .iter()
            .map(|m| {
                let role = match m.role {
                    Role::User => "user",
                    Role::Assistant => "assistant",
                };
                json!({
                    "role": role,
                    "content": [{"type": "text", "text": m.content}]
                })
            })
            .collect();

        let mut full_assistant = String::new();
        let mut iterations = 0;

        loop {
            iterations += 1;
            if iterations > 6 {
                let _ = deltas.send(LlmDelta::Error("agent loop limit reached".into()));
                break;
            }

            let body = json!({
                "model": self.model,
                "max_tokens": 2048,
                "system": SYSTEM_PROMPT,
                "tools": [tool_schema()],
                "stream": true,
                "messages": messages,
            });

            let resp = self
                .http
                .post(API_URL)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", API_VERSION)
                .header("content-type", "application/json")
                .json(&body)
                .send()
                .await?;

            if !resp.status().is_success() {
                let status = resp.status();
                let text = resp.text().await.unwrap_or_default();
                let msg = format!("Anthropic HTTP {}: {}", status, text);
                let _ = deltas.send(LlmDelta::Error(msg.clone()));
                return Err(anyhow!(msg));
            }

            let mut stream = resp.bytes_stream();
            let mut buf = String::new();

            let mut tool_uses: Vec<ToolUse> = Vec::new();
            let mut current_text_index_text = std::collections::BTreeMap::<usize, String>::new();
            let mut current_tool_index: std::collections::BTreeMap<usize, ToolUse> =
                std::collections::BTreeMap::new();
            let mut stop_reason: Option<String> = None;

            while let Some(chunk) = stream.next().await {
                let bytes = chunk?;
                buf.push_str(&String::from_utf8_lossy(&bytes));

                while let Some(pos) = buf.find("\n\n") {
                    let raw_event = buf[..pos].to_string();
                    buf.drain(..pos + 2);
                    let mut data_line: Option<String> = None;
                    for line in raw_event.lines() {
                        if let Some(d) = line.strip_prefix("data:") {
                            let d = d.trim_start();
                            if data_line.is_none() {
                                data_line = Some(d.to_string());
                            } else {
                                data_line = Some(format!(
                                    "{}{}",
                                    data_line.unwrap_or_default(),
                                    d
                                ));
                            }
                        }
                    }

                    let Some(data) = data_line else { continue };
                    if data == "[DONE]" {
                        continue;
                    }

                    let v: Value = match serde_json::from_str(&data) {
                        Ok(v) => v,
                        Err(_) => continue,
                    };
                    let event_type = v.get("type").and_then(|t| t.as_str()).unwrap_or("");

                    match event_type {
                        "content_block_start" => {
                            let idx = v
                                .get("index")
                                .and_then(|i| i.as_u64())
                                .unwrap_or(0) as usize;
                            let block = v.get("content_block").cloned().unwrap_or(Value::Null);
                            let btype = block
                                .get("type")
                                .and_then(|t| t.as_str())
                                .unwrap_or("");
                            if btype == "tool_use" {
                                let name = block
                                    .get("name")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                let id = block
                                    .get("id")
                                    .and_then(|s| s.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                current_tool_index.insert(
                                    idx,
                                    ToolUse {
                                        id,
                                        name: name.clone(),
                                        input_json: String::new(),
                                    },
                                );
                                let _ = deltas.send(LlmDelta::ToolCall { name });
                            } else if btype == "text" {
                                current_text_index_text.entry(idx).or_default();
                            }
                        }
                        "content_block_delta" => {
                            let idx = v
                                .get("index")
                                .and_then(|i| i.as_u64())
                                .unwrap_or(0) as usize;
                            if let Some(d) = v.get("delta") {
                                let dtype =
                                    d.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                match dtype {
                                    "text_delta" => {
                                        if let Some(text) =
                                            d.get("text").and_then(|t| t.as_str())
                                        {
                                            current_text_index_text
                                                .entry(idx)
                                                .or_default()
                                                .push_str(text);
                                            let _ = deltas
                                                .send(LlmDelta::Text(text.to_string()));
                                        }
                                    }
                                    "input_json_delta" => {
                                        if let Some(part) = d
                                            .get("partial_json")
                                            .and_then(|p| p.as_str())
                                        {
                                            if let Some(t) =
                                                current_tool_index.get_mut(&idx)
                                            {
                                                t.input_json.push_str(part);
                                            }
                                        }
                                    }
                                    _ => {}
                                }
                            }
                        }
                        "content_block_stop" => {
                            let idx = v
                                .get("index")
                                .and_then(|i| i.as_u64())
                                .unwrap_or(0) as usize;
                            if let Some(t) = current_tool_index.remove(&idx) {
                                tool_uses.push(t);
                            }
                        }
                        "message_delta" => {
                            if let Some(reason) = v
                                .get("delta")
                                .and_then(|d| d.get("stop_reason"))
                                .and_then(|s| s.as_str())
                            {
                                stop_reason = Some(reason.to_string());
                            }
                        }
                        "message_stop" => {}
                        "error" => {
                            let msg = v
                                .get("error")
                                .and_then(|e| e.get("message"))
                                .and_then(|m| m.as_str())
                                .unwrap_or("anthropic stream error")
                                .to_string();
                            let _ = deltas.send(LlmDelta::Error(msg.clone()));
                            return Err(anyhow!(msg));
                        }
                        _ => {}
                    }
                }
            }

            let assistant_text: String = current_text_index_text
                .values()
                .cloned()
                .collect::<Vec<_>>()
                .join("");

            full_assistant.push_str(&assistant_text);

            let mut assistant_content: Vec<Value> = Vec::new();
            if !assistant_text.is_empty() {
                assistant_content.push(json!({"type":"text","text":assistant_text}));
            }
            for tu in &tool_uses {
                let parsed_input: Value = if tu.input_json.trim().is_empty() {
                    json!({})
                } else {
                    serde_json::from_str(&tu.input_json).unwrap_or(json!({}))
                };
                assistant_content.push(json!({
                    "type": "tool_use",
                    "id": tu.id,
                    "name": tu.name,
                    "input": parsed_input,
                }));
            }

            if assistant_content.is_empty() {
                break;
            }

            messages.push(json!({
                "role": "assistant",
                "content": assistant_content,
            }));

            if stop_reason.as_deref() == Some("tool_use") && !tool_uses.is_empty() {
                let mut user_content: Vec<Value> = Vec::new();
                for tu in &tool_uses {
                    let result = match tu.name.as_str() {
                        GET_ACTIVE_BUILD => ctx.render_tool_result(),
                        other => json!({"error": format!("unknown tool {}", other)}).to_string(),
                    };
                    let _ = deltas.send(LlmDelta::ToolResult {
                        name: tu.name.clone(),
                    });
                    user_content.push(json!({
                        "type": "tool_result",
                        "tool_use_id": tu.id,
                        "content": result,
                    }));
                }
                messages.push(json!({
                    "role": "user",
                    "content": user_content,
                }));
                continue;
            }

            break;
        }

        let _ = deltas.send(LlmDelta::End);
        Ok(full_assistant)
    }
}

#[derive(Debug, Clone)]
struct ToolUse {
    id: String,
    name: String,
    input_json: String,
}
