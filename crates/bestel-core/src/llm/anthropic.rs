use anyhow::{anyhow, Context, Result};
use base64::engine::general_purpose::STANDARD as B64;
use base64::Engine;
use futures_util::StreamExt;
use serde_json::{json, Value};
use tokio::sync::mpsc;

use super::tools::{dispatch, tool_schemas, BuildContext, ToolCtx};
use super::{ChatMessage, LlmDelta, Role, ToolStatus};
use crate::devlog;
use crate::prompts;

fn base64_decode_to_text(s: &str) -> Option<String> {
    let bytes = B64.decode(s).ok()?;
    String::from_utf8(bytes).ok()
}

const API_URL: &str = "https://api.anthropic.com/v1/messages";
const API_VERSION: &str = "2023-06-01";
const DEFAULT_API_KEY_ENV: &str = "ANTHROPIC_API_KEY";

/// Hard cap on tool-use iterations within a single agent run. DeepSeek and
/// other tool-eager models can chain 8-12 wiki lookups before answering;
/// 16 leaves room for thoroughness without runaway loops. When this is
/// reached we still run ONE more pass with `tools: []` and a wrap-up nudge
/// so the user always gets a final answer instead of a bare error.
const MAX_AGENT_ITERATIONS: usize = 16;
const WRAP_UP_NUDGE: &str =
    "[system: tool budget reached. Give your final answer now using what you've already gathered. Do not request more tools.]";

pub struct AnthropicClient {
    api_key: String,
    model: String,
    api_url: String,
    /// Optional `(input_rate_usd_per_mtok, output_rate_usd_per_mtok)` used
    /// to compute live cost telemetry. `None` skips the cost field of the
    /// emitted `Usage` delta but token counts are still surfaced.
    cost_per_mtok: Option<(f32, f32)>,
    http: reqwest::Client,
}

impl AnthropicClient {
    pub fn from_env() -> Result<Self> {
        let api_key = std::env::var(DEFAULT_API_KEY_ENV)
            .context("ANTHROPIC_API_KEY environment variable not set")?;
        let active = super::models::active_profile();
        let model = if active.provider == super::models::ProviderKind::Anthropic
            && !active.model_id.is_empty()
        {
            active.model_id.clone()
        } else {
            "claude-sonnet-4-5-20250929".to_string()
        };
        Ok(Self {
            api_key,
            model,
            api_url: API_URL.to_string(),
            cost_per_mtok: active.cost_per_mtok,
            http: reqwest::Client::builder()
                .timeout(std::time::Duration::from_secs(120))
                .build()?,
        })
    }

    /// Build a client against an Anthropic-compatible endpoint (e.g.
    /// DeepSeek's `https://api.deepseek.com/anthropic`). `base_url` is the
    /// prefix without `/v1/messages`. `env_var` names the env variable
    /// holding the API key. `model` is the model id to send. `cost_per_mtok`
    /// is the `(input, output)` USD/Mtok pair from the active profile —
    /// pass `None` to omit cost telemetry.
    pub fn from_endpoint(
        base_url: &str,
        env_var: &str,
        model: String,
        cost_per_mtok: Option<(f32, f32)>,
    ) -> Result<Self> {
        let api_key = std::env::var(env_var)
            .with_context(|| format!("{env_var} environment variable not set"))?;
        let trimmed = base_url.trim_end_matches('/');
        let api_url = format!("{trimmed}/v1/messages");
        Ok(Self {
            api_key,
            model,
            api_url,
            cost_per_mtok,
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
                let mut blocks: Vec<Value> = Vec::new();
                for att in &m.attachments {
                    if att.is_image() {
                        blocks.push(json!({
                            "type": "image",
                            "source": {
                                "type": "base64",
                                "media_type": att.mime,
                                "data": att.data_base64,
                            }
                        }));
                    } else if att.is_text_doc() {
                        // Decode + inline as a text block. We don't try to render
                        // PDFs here — just include a header so the model knows
                        // a document was attached. Anthropic's PDF support is a
                        // separate beta we don't ship in v0.1.
                        let preview = base64_decode_to_text(&att.data_base64)
                            .unwrap_or_else(|| "<binary content>".to_string());
                        let trimmed: String = preview.chars().take(20_000).collect();
                        blocks.push(json!({
                            "type": "text",
                            "text": format!(
                                "[Attached file: {} · {}]\n\n{}",
                                att.name, att.mime, trimmed
                            )
                        }));
                    }
                }
                if !m.content.is_empty() || blocks.is_empty() {
                    blocks.push(json!({"type": "text", "text": m.content}));
                }
                json!({"role": role, "content": blocks})
            })
            .collect();

        let mut full_assistant = String::new();
        let mut iterations = 0;
        // Build a one-line state hint kept OUT of the cached system blocks so
        // detaching / attaching a build doesn't invalidate the cache. The
        // string is appended as a final, non-cached system block on every
        // request. Convention documented in SYSTEM_PROMPT.md ("Build
        // awareness").
        let has_active_build = ctx.get().is_some();
        let build_state_line = match ctx.get() {
            Some(b) => format!(
                "Build state: loaded — {} {} lvl {}",
                b.game.label(),
                b.class,
                b.level.unwrap_or(0)
            ),
            None => "Build state: detached".to_string(),
        };
        // Sprint C cache breakpoints: split SYSTEM_PROMPT on `<tool_policy>`
        // so the persona + runtime contract live in BP2 (rarely changes), the
        // tool policy + mode router + contracts + failure policy live in BP3
        // (changes when tools or contracts evolve), and CORE_KNOWLEDGE lives
        // in BP4 (changes with reference library re-indexing).
        let (system_block_1, system_block_2, core_knowledge_block) =
            prompts::load_anthropic_blocks("anthropic", &self.model);
        let last_user_question: Option<String> = history
            .iter()
            .rev()
            .find(|m| matches!(m.role, super::Role::User))
            .map(|m| m.content.clone())
            .filter(|s| !s.trim().is_empty());
        let mut tool_ctx = ToolCtx::new(ctx).context("build ToolCtx")?;
        if let Some(q) = last_user_question {
            tool_ctx = tool_ctx.with_user_question(q);
        }
        let schemas = cached_tool_schemas();
        let mut wrap_up_done = false;
        let mut total_usage = super::UsageStats::default();

        loop {
            iterations += 1;
            // After MAX_AGENT_ITERATIONS regular passes, run exactly one more
            // turn with tools disabled and a wrap-up nudge so the model
            // produces a final answer instead of erroring on the user.
            let force_finalize = iterations > MAX_AGENT_ITERATIONS;
            if force_finalize {
                if wrap_up_done {
                    break;
                }
                wrap_up_done = true;
                messages.push(json!({
                    "role": "user",
                    "content": [{ "type": "text", "text": WRAP_UP_NUDGE }],
                }));
            }

            // Build the system array. The first three blocks each carry
            // `cache_control: ephemeral` with TTL 1h so a desktop session
            // spanning more than 5 min still benefits from the cache. The
            // fourth block (build_state) is dynamic — no cache_control, sits
            // AFTER the cached blocks so flipping a build between turns
            // doesn't invalidate any cache entry.
            let mut system_blocks: Vec<Value> = Vec::with_capacity(4);
            if !system_block_1.is_empty() {
                system_blocks.push(json!({
                    "type": "text",
                    "text": system_block_1,
                    "cache_control": { "type": "ephemeral", "ttl": "1h" }
                }));
            }
            if !system_block_2.is_empty() {
                system_blocks.push(json!({
                    "type": "text",
                    "text": system_block_2,
                    "cache_control": { "type": "ephemeral", "ttl": "1h" }
                }));
            }
            if !core_knowledge_block.is_empty() {
                system_blocks.push(json!({
                    "type": "text",
                    "text": core_knowledge_block,
                    "cache_control": { "type": "ephemeral", "ttl": "1h" }
                }));
            }
            system_blocks.push(json!({
                "type": "text",
                "text": format!("[{}]", build_state_line),
            }));

            // Sprint D — force `get_active_build` on iteration 1 whenever a
            // build is loaded. Eliminates the "model decides not to call the
            // build context" failure class that the 2026-05-08 audit found in
            // 23/24 build runs. After this single forced call, the result is
            // in the messages array for every subsequent iteration, and we
            // fall back to `tool_choice: auto` so the model picks freely.
            let force_build_context =
                iterations == 1 && !force_finalize && has_active_build;
            let mut body = json!({
                "model": self.model,
                "max_tokens": 8192,
                "system": system_blocks,
                "tools": if force_finalize { json!([]) } else { json!(schemas) },
                "stream": true,
                "messages": messages,
            });
            if force_build_context {
                body["tool_choice"] = json!({
                    "type": "tool",
                    "name": super::tools::GET_ACTIVE_BUILD,
                });
            }

            let resp = self
                .post_with_retry(&body, &deltas)
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
            let mut thinking_open: bool = false;
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

                    devlog::log_provider_raw("anthropic", &data);

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
                                        id: id.clone(),
                                        name: name.clone(),
                                        input_json: String::new(),
                                    },
                                );
                                let _ = deltas.send(LlmDelta::ToolBegin {
                                    id,
                                    name,
                                    detail: None,
                                });
                            } else if btype == "text" {
                                current_text_index_text.entry(idx).or_default();
                            } else if btype == "thinking" || btype == "redacted_thinking" {
                                if !thinking_open {
                                    let _ = deltas.send(LlmDelta::ReasoningBegin);
                                    thinking_open = true;
                                }
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
                                                .send(LlmDelta::TextDelta(text.to_string()));
                                        }
                                    }
                                    "thinking_delta" => {
                                        if let Some(text) =
                                            d.get("thinking").and_then(|t| t.as_str())
                                        {
                                            if !thinking_open {
                                                let _ = deltas.send(LlmDelta::ReasoningBegin);
                                                thinking_open = true;
                                            }
                                            let _ = deltas
                                                .send(LlmDelta::ReasoningDelta(text.to_string()));
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
                                                // TODO (live tool input streaming):
                                                // every few chunks, attempt a
                                                // best-effort partial JSON parse on
                                                // `t.input_json` and emit a
                                                // `ToolDetailUpdate` (new variant)
                                                // so the user sees the URL/query
                                                // grow inside the tool card *while
                                                // it is being constructed*. This
                                                // matches ChatGPT/Claude.ai's UX
                                                // where the URL appears at the
                                                // start of a search rather than at
                                                // the end. Codex CLI cannot do
                                                // this — see PROVIDER_NOTES.md.
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
                            if thinking_open {
                                let _ = deltas.send(LlmDelta::ReasoningEnd);
                                thinking_open = false;
                            }
                        }
                        "message_start" => {
                            // Initial usage block: input + cache stats are
                            // settled by the time this fires; only output
                            // tokens grow during the stream.
                            if let Some(usage) = v.pointer("/message/usage") {
                                accumulate_usage(&mut total_usage, usage);
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
                            // Final output token count for this iteration.
                            // Anthropic only re-reports `output_tokens` here,
                            // not the cache fields — but DeepSeek may include
                            // them so we accumulate defensively.
                            if let Some(usage) = v.get("usage") {
                                accumulate_usage(&mut total_usage, usage);
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

            // Separate text from this iteration's assistant turn from the
            // previous iteration's text. Without this the "narrating"
            // prose around tool calls gets glued: e.g.
            // "...the boss:Here is your real DPS..." (no break). Inserting
            // a paragraph break makes the persisted final_text + the chat
            // UI render naturally and matches what the user actually saw
            // streaming in.
            if !full_assistant.is_empty() && !assistant_text.is_empty() {
                if !full_assistant.ends_with('\n') {
                    full_assistant.push('\n');
                }
                full_assistant.push('\n');
            }
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
                    let parsed_input: Value = if tu.input_json.trim().is_empty() {
                        json!({})
                    } else {
                        serde_json::from_str(&tu.input_json).unwrap_or(json!({}))
                    };
                    let (result, status) = match dispatch(&tu.name, &parsed_input, &tool_ctx).await {
                        Ok(s) => (s, ToolStatus::Done),
                        Err(e) => (
                            json!({"error": e.to_string()}).to_string(),
                            ToolStatus::Failed,
                        ),
                    };
                    crate::devlog::log_value(
                        "tool_call",
                        json!({
                            "name": tu.name,
                            "input": parsed_input,
                            "status": match status {
                                ToolStatus::Done => "done",
                                ToolStatus::Failed => "failed",
                                ToolStatus::Running => "running",
                            },
                            "result_bytes": result.len(),
                        }),
                    );
                    // Forward the full result as a single ToolOutput chunk so
                    // the frontend can populate `seg.output` and parse rich
                    // payloads (show_in_panel, future structured tools).
                    // Aligned with the Ollama provider's behavior.
                    let _ = deltas.send(LlmDelta::ToolOutput {
                        id: tu.id.clone(),
                        chunk: result.clone(),
                    });
                    let _ = deltas.send(LlmDelta::ToolEnd {
                        id: tu.id.clone(),
                        status,
                        summary: None,
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

        if total_usage.input_tokens
            + total_usage.cached_input_tokens
            + total_usage.cache_creation_tokens
            + total_usage.output_tokens
            > 0
        {
            total_usage.cost_usd = compute_cost(self.cost_per_mtok, &total_usage);
            let _ = deltas.send(LlmDelta::Usage(total_usage));
        }

        // Sprint G — conditional verifier. Runs only if `should_verify` matches
        // (numbers / sources / cached / banned phrases / identity / patch /
        // tier). Latency budget: ~2s typical; never propagates errors back to
        // the user (a failing verifier is a `pass_with_note`, not a failure).
        let verified_text = self
            .run_verifier_pass(&history, &full_assistant, &deltas)
            .await;
        let _ = deltas.send(LlmDelta::MessageEnd);
        Ok(verified_text)
    }

    async fn run_verifier_pass(
        &self,
        history: &[ChatMessage],
        draft: &str,
        deltas: &mpsc::UnboundedSender<LlmDelta>,
    ) -> String {
        if !super::verifier::should_verify(draft) {
            return draft.to_string();
        }
        let last_user = history
            .iter()
            .rev()
            .find(|m| matches!(m.role, super::Role::User))
            .map(|m| m.content.as_str())
            .unwrap_or("");
        let verdict = super::verifier::verify(
            &self.http,
            &self.api_url,
            &self.api_key,
            API_VERSION,
            &self.model,
            last_user,
            draft,
        )
        .await;
        let result = match verdict.status {
            super::verifier::VerdictStatus::Pass => {
                tracing::info!(
                    target: "bestel.verifier",
                    findings = verdict.findings.len(),
                    "verifier pass"
                );
                draft.to_string()
            }
            super::verifier::VerdictStatus::Revise => {
                let rewrite = if verdict.minimal_rewrite.trim().is_empty() {
                    draft.to_string()
                } else {
                    verdict.minimal_rewrite.clone()
                };
                tracing::info!(
                    target: "bestel.verifier",
                    findings = verdict.findings.len(),
                    "verifier revise — swapping in minimal_rewrite"
                );
                let _ = deltas.send(LlmDelta::TextDelta(format!(
                    "\n\n[verifier: revised — {} finding(s)]\n",
                    verdict.findings.len()
                )));
                rewrite
            }
            super::verifier::VerdictStatus::Fail => {
                let summary = if verdict.findings.is_empty() {
                    "no specific findings".to_string()
                } else {
                    verdict
                        .findings
                        .iter()
                        .map(|f| format!("{}: {}", f.category, f.issue))
                        .collect::<Vec<_>>()
                        .join("; ")
                };
                tracing::warn!(
                    target: "bestel.verifier",
                    findings = verdict.findings.len(),
                    summary = %summary,
                    "verifier FAIL — emitting fallback"
                );
                let fallback = format!(
                    "The chronicler must withhold that answer, exile — the verifier flagged it ({summary}). Ask again with a narrower question, or attach a fresh PoB so the engine can ground the numbers."
                );
                let _ = deltas.send(LlmDelta::TextDelta(format!(
                    "\n\n[verifier: rejected — {summary}]\n"
                )));
                fallback
            }
        };
        let _ = deltas.send(LlmDelta::Verifier(verdict));
        result
    }
}

impl AnthropicClient {
    /// POST with automatic retry on 429 / 5xx. Parses `retry-after` header
    /// (seconds) when present, falls back to an exponential schedule. Caps
    /// total attempts at 4 (1 initial + 3 retries) so a permanently-down
    /// endpoint still surfaces the error eventually.
    async fn post_with_retry(
        &self,
        body: &Value,
        deltas: &mpsc::UnboundedSender<LlmDelta>,
    ) -> Result<reqwest::Response> {
        const MAX_ATTEMPTS: u32 = 4;
        const FALLBACK_5XX_BASE: u64 = 5;
        let mut attempt: u32 = 0;
        loop {
            attempt += 1;
            let resp = self
                .http
                .post(&self.api_url)
                .header("x-api-key", &self.api_key)
                .header("anthropic-version", API_VERSION)
                // Sprint C: enable 1h prompt cache TTL (default is 5 min).
                // Cache writes are billed at 2× input rate when TTL = 1h, but
                // Bestel desktop sessions span 30+ min so the per-turn
                // cache_read savings amortize the write cost across the
                // session. DeepSeek via Anthropic-compat silently ignores
                // unknown beta headers.
                .header("anthropic-beta", "extended-cache-ttl-2025-04-11")
                .header("content-type", "application/json")
                .json(body)
                .send()
                .await?;
            let status = resp.status();
            let retryable = status == reqwest::StatusCode::TOO_MANY_REQUESTS
                || status.is_server_error();
            if !retryable || attempt >= MAX_ATTEMPTS {
                return Ok(resp);
            }
            let header_wait = resp
                .headers()
                .get(reqwest::header::RETRY_AFTER)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.trim().parse::<u64>().ok());
            let wait = header_wait.unwrap_or_else(|| {
                if status == reqwest::StatusCode::TOO_MANY_REQUESTS {
                    30
                } else {
                    FALLBACK_5XX_BASE * (1u64 << (attempt - 1))
                }
            });
            // Drop the response body so the connection can be reused.
            let _ = resp.text().await;
            crate::devlog::log_value(
                "anthropic_retry",
                json!({
                    "status": status.as_u16(),
                    "attempt": attempt,
                    "max_attempts": MAX_ATTEMPTS,
                    "wait_secs": wait,
                    "header_wait": header_wait,
                }),
            );
            let _ = deltas.send(LlmDelta::TextDelta(String::new()));
            tokio::time::sleep(std::time::Duration::from_secs(wait)).await;
        }
    }
}

/// Read the `usage` JSON object Anthropic sends in `message_start` /
/// `message_delta` events and add its counters into our running total.
/// Anthropic returns these as cumulative-since-start, so we replace
/// instead of summing — except for `output_tokens` which is reported
/// per iteration in `message_delta` (each iteration starts at 0 again).
///
/// Across multiple agent loop iterations, totals are summed.
fn accumulate_usage(total: &mut super::UsageStats, usage: &Value) {
    if let Some(v) = usage.get("input_tokens").and_then(|x| x.as_u64()) {
        total.input_tokens += v;
    }
    if let Some(v) = usage
        .get("cache_read_input_tokens")
        .and_then(|x| x.as_u64())
    {
        total.cached_input_tokens += v;
    }
    if let Some(v) = usage
        .get("cache_creation_input_tokens")
        .and_then(|x| x.as_u64())
    {
        total.cache_creation_tokens += v;
    }
    if let Some(v) = usage.get("output_tokens").and_then(|x| x.as_u64()) {
        total.output_tokens += v;
    }
}

/// Compute USD cost using Anthropic's standard 5m-cache pricing recipe.
/// Cache reads ≈ 10% of base input rate; cache writes ≈ 125%. Rates passed
/// in are USD per million tokens. DeepSeek doesn't actually cache via the
/// `/anthropic` endpoint — `cached_input_tokens` and `cache_creation_tokens`
/// will be zero there, so the formula collapses to plain input × output.
fn compute_cost(rates: Option<(f32, f32)>, u: &super::UsageStats) -> Option<f64> {
    let (input_rate, output_rate) = rates?;
    let input_rate = input_rate as f64;
    let output_rate = output_rate as f64;
    let cost = (u.input_tokens as f64 * input_rate
        + u.cached_input_tokens as f64 * input_rate * 0.10
        + u.cache_creation_tokens as f64 * input_rate * 1.25
        + u.output_tokens as f64 * output_rate)
        / 1_000_000.0;
    Some(cost)
}

#[derive(Debug, Clone)]
struct ToolUse {
    id: String,
    name: String,
    input_json: String,
}

fn cached_tool_schemas() -> Vec<Value> {
    let mut schemas = tool_schemas();
    if let Some(last) = schemas.last_mut() {
        if let Some(obj) = last.as_object_mut() {
            // `cache_control` on the LAST tool schema caches the entire tools
            // array per Anthropic API semantics. TTL 1h matches the system
            // blocks (Sprint C) so desktop sessions longer than 5 min still
            // hit cache.
            obj.insert(
                "cache_control".to_string(),
                json!({ "type": "ephemeral", "ttl": "1h" }),
            );
        }
    }
    schemas
}
