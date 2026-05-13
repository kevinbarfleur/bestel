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
                // SSE streaming for long agentic turns can exceed 5 min with
                // DeepSeek-V4-Pro's reasoning + deep analysis chains (we saw
                // 190s of pure analysis before the model emits
                // sheet_open_interview). The previous 120s overall timeout
                // killed those streams mid-response → reqwest
                // "error decoding response body" on the next chunk. Bumping
                // to 30 min covers worst-case multi-tool agentic loops
                // without leaving the user hanging forever if the provider
                // truly gets stuck. Connect timeout stays short (10s) so
                // dead endpoints fail fast.
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(1800))
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
                // SSE streaming for long agentic turns can exceed 5 min with
                // DeepSeek-V4-Pro's reasoning + deep analysis chains (we saw
                // 190s of pure analysis before the model emits
                // sheet_open_interview). The previous 120s overall timeout
                // killed those streams mid-response → reqwest
                // "error decoding response body" on the next chunk. Bumping
                // to 30 min covers worst-case multi-tool agentic loops
                // without leaving the user hanging forever if the provider
                // truly gets stuck. Connect timeout stays short (10s) so
                // dead endpoints fail fast.
                .connect_timeout(std::time::Duration::from_secs(10))
                .timeout(std::time::Duration::from_secs(1800))
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
                if !m.content.trim().is_empty() {
                    blocks.push(json!({"type": "text", "text": m.content}));
                } else if blocks.is_empty() {
                    // 2026-05-09 — Anthropic rejects messages whose only
                    // content block is `{type: "text", text: ""}` AND
                    // also rejects whitespace-only text blocks
                    // ("text content blocks must contain non-whitespace
                    // text"). This happens when an assistant turn ended
                    // on a forced tool_use (e.g. sheet_open_interview)
                    // without emitting any final text — the history then
                    // stored `content: ""`. The placeholder must be
                    // non-whitespace; an ellipsis is short, semantically
                    // neutral, and the model treats it as a no-op
                    // continuation cue. Whitespace-only `m.content`
                    // values from upstream go through the same trim,
                    // so they hit this branch too.
                    blocks.push(json!({"type": "text", "text": "…"}));
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
        // Pre-compute the Build Sheet status so the model sees it AT TURN
        // START rather than having to remember to call `get_active_build_sheet`
        // and branch on the result. The 2026-05-09 audit showed the
        // skill-text/system-prompt approach failing — the model loaded the
        // build-review skill *after* doing wiki_parse, so the STEP 0 pivot
        // never fired. Surfacing the status as a runtime tag means the rules
        // can reference observable state instead of relying on the model to
        // probe.
        //
        // Format:
        //   "Build sheet: absent (fingerprint=…)"            — no validated sheet
        //   "Build sheet: validated, fresh (id=N)"           — sheet found, hash matches
        //   "Build sheet: stale-since-Xd (id=N)"             — sheet found, hash drift
        //   "" (empty)                                       — no build / no DB
        //
        // `sheet_status_kind` is used downstream to gate the iter-2 forced
        // tool_choice (see force_pivot_to_sheet below).
        #[derive(Clone, Copy, PartialEq, Eq)]
        enum SheetStatusKind {
            Absent,
            Fresh,
            Stale,
            Unknown,
        }
        // Pre-compute fingerprint + canonical hash + sheet lookup once per
        // turn. Results feed both the runtime system tag (so the agent
        // sees them without recomputing) and the iter > 8 deferred-force
        // logic below. `sheet_pob_hash_for_prompt` is the CURRENT PoB
        // hash — the agent passes it to `sheet_finalize_request` (and to
        // `get_active_build_sheet` when checking staleness manually).
        let (
            build_sheet_line,
            sheet_fingerprint_for_prompt,
            sheet_pob_hash_for_prompt,
            sheet_status_kind,
        ) = match ctx.get() {
            Some(b) => match crate::sheets::compute_fingerprint_from_pob(&b) {
                Some(fp) => {
                    let current_hash = crate::sheets::compute_pob_hash_from_build(&b);
                    match crate::persistence::global_db() {
                        Some(db) => match crate::sheets::store::find_by_fingerprint(&db, &fp) {
                            Ok(Some(row)) => {
                                if current_hash == row.pob_hash {
                                    (
                                        format!("Build sheet: validated, fresh (id={})", row.id),
                                        Some(fp),
                                        Some(current_hash),
                                        SheetStatusKind::Fresh,
                                    )
                                } else {
                                    (
                                        format!(
                                            "Build sheet: stale (id={}, hash drift since authoring)",
                                            row.id
                                        ),
                                        Some(fp),
                                        Some(current_hash),
                                        SheetStatusKind::Stale,
                                    )
                                }
                            }
                            Ok(None) => (
                                format!("Build sheet: absent (fingerprint={fp})"),
                                Some(fp),
                                Some(current_hash),
                                SheetStatusKind::Absent,
                            ),
                            Err(e) => {
                                tracing::warn!(error = ?e, "find_by_fingerprint failed");
                                (String::new(), None, None, SheetStatusKind::Unknown)
                            }
                        },
                        None => (String::new(), None, None, SheetStatusKind::Unknown),
                    }
                }
                None => (String::new(), None, None, SheetStatusKind::Unknown),
            },
            None => (String::new(), None, None, SheetStatusKind::Unknown),
        };

        // Detect the structured `[INTERVIEW SUBMISSION ...]` cue emitted
        // by the frontend when the user clicks Submit on a one-shot
        // interview panel. Used downstream to (a) inject a runtime
        // directive that overrides the `Build sheet: absent` flow into
        // a finalize-and-answer flow, and (b) force tool_choice to
        // `sheet_finalize_request` at iter 2 of that turn so the agent
        // cannot re-launch the interview by mistake. Without this, the
        // model loops the interview each time the user submits — the
        // 2026-05-09 audit reproduced this exact failure twice.
        let last_user_was_interview_submission = history
            .iter()
            .rev()
            .find(|m| matches!(m.role, super::Role::User))
            .map(|m| m.content.contains("[INTERVIEW SUBMISSION"))
            .unwrap_or(false);
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
        // Sprint value-purge — snapshot the last user message in lowercase
        // so the per-iteration mechanics-directive heuristic can read it
        // without re-borrowing from the consumed `last_user_question`.
        let last_user_lower: String = last_user_question
            .as_deref()
            .map(|s| s.to_ascii_lowercase())
            .unwrap_or_default();
        // Sprint v3 — deterministic turn-mode classifier. Runs once per
        // turn BEFORE the first LLM call so the system prompt receives a
        // `[Mode: ...]` runtime tag and the frontend renders a ModeChip
        // for non-default modes. Routes brief mechanical questions away
        // from forced interviews, escalates explicit "skip the sheet"
        // phrasing to the legacy diagnostic flow.
        let has_active_sheet = matches!(
            sheet_status_kind,
            SheetStatusKind::Fresh | SheetStatusKind::Stale
        );
        let turn_mode = super::turn_classifier::classify_turn(
            &last_user_lower,
            has_active_build,
            has_active_sheet,
        );
        if turn_mode.surfaces_to_user() {
            let _ = deltas.send(LlmDelta::ModeAssigned {
                mode: turn_mode.as_str().to_string(),
            });
        }
        let mut tool_ctx = ToolCtx::new(ctx).context("build ToolCtx")?;
        tool_ctx = tool_ctx.with_deltas(deltas.clone());
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
            let mut state_block = format!("[{build_state_line}]");
            // Sprint v3 — surface the deterministic mode classification as
            // a runtime tag so the system-prompt `<answer_mode_router>`
            // block can branch on it. Always emit (even default) so the
            // router has a known value to dispatch on.
            state_block.push_str(&format!("\n[Mode: {}]", turn_mode.as_str()));
            // Sprint M — anti-hallucination anchor. The 2026-05-10 quality
            // batch caught Haiku 4.5 fabricating build stats: Q1 invented
            // a "Ranger Kinetic Blast" PoB that didn't match the loaded
            // Inquisitor, and Q3 invented life/ES/evasion numbers. The
            // model called `get_active_build` correctly but treated the
            // JSON as decorative rather than authoritative. We surface an
            // explicit "cite ONLY this JSON" directive as a runtime tag
            // BEFORE `get_active_build` runs — same attentional weight as
            // the build state tag, zero UI side-effects (runtime tags live
            // in the system prompt, never rendered to the exile).
            //
            // Put inside the same dynamic block as `Build state:` so it
            // sits AFTER the cached blocks (no cache_control here) — the
            // build identity can change turn-to-turn but the directive is
            // always identical when a build is loaded, so the model sees
            // both in the same uncached chunk.
            if has_active_build {
                // Sprint v3 anti-duplication: if `get_active_build` was already
                // called earlier in this chat (its result is in the messages
                // array above as a tool_result block), tell the model to read
                // from that prior payload instead of calling the tool again.
                // The build can change turn-to-turn (PoB swap, watcher hot-
                // reload), but the runtime would flip `Build state:` and clear
                // the cached fingerprint hash before getting here, so a stale
                // tool_result is not a concern — every call returns the
                // current snapshot.
                let build_already_fetched =
                    messages_contain_tool_use(&messages, super::tools::GET_ACTIVE_BUILD);
                if build_already_fetched {
                    state_block.push_str(
                        "\n[Build data directive: get_active_build has ALREADY been called \
                         earlier in this conversation — its full JSON payload is in the prior \
                         tool_result blocks above. Do NOT call get_active_build again this turn; \
                         read every value (life, ES, resistances, ascendancy, main skill, \
                         defining uniques, stat IDs, gem levels, tree allocations) from that \
                         existing payload. The build snapshot is your source of truth: cite ONLY \
                         values present there, never extrapolate or recall from training. \
                         Numbers reported MUST match the JSON byte-for-byte or be flagged as \
                         derived via pob_calc.]",
                    );
                } else {
                    state_block.push_str(
                        "\n[Build data directive: when get_active_build returns, the JSON it carries \
                         is the AUTHORITATIVE snapshot of the exile's build. Cite ONLY values present \
                         in that payload — life, ES, resistances, ascendancy, main skill, defining \
                         uniques, stat IDs, gem levels, tree allocations. Do NOT extrapolate, infer, \
                         or recall from training data — if a field is absent or null, say so plainly. \
                         Numbers reported MUST match the JSON byte-for-byte or be flagged as derived \
                         via pob_calc.]",
                    );
                }
            }
            // Sprint value-purge — mechanics directive. The 2026-05-12 audit
            // found ~60% of hardcoded numeric values in references were stale
            // or wrong (MoM = 30% in refs vs 40% live since patch 3.19,
            // Trinity 50% pen vs 16% live, Bone Helmet +30% vs 15-20% live,
            // Flame Dash 10s vs 3.5s live). Root cause: a value in context
            // discourages the model from fetching the live wiki value at
            // answer-time. We strip values from references AND surface this
            // directive so the model is forced to fetch when the question is
            // mechanic-coded. Fires when no build is loaded (generalist mode
            // = no engine ground truth) OR when the question matches the
            // mechanic heuristic. Build-loaded turns without mechanic intent
            // are already covered by the [Build data directive] above.
            let mechanic_coded = !has_active_build || is_mechanic_question(&last_user_lower);
            if mechanic_coded {
                state_block.push_str(
                    "\n[Mechanics directive: any numerical claim in this turn's answer \
                     — percent, cooldown, charge count, socket count, magnitude, mod tier, \
                     drop level, base stat, breakpoint — MUST be backed by a same-turn call \
                     to wiki_parse, wiki_cargo, repoe_lookup, pob_calc, or web_fetch. \
                     Bundled references (`prompts/references/...`) carry methodology only; \
                     their values are by design stripped, stale, or community-heuristic and \
                     NEVER ground a numerical claim. Exceptions: fields from get_active_build \
                     / get_active_build_sheet (already engine-grounded) and the `thresholds/` \
                     files (must be cited as community-aggregate, never as wiki fact). If a \
                     value is needed and no tool call is appropriate, say plainly: \"the old \
                     chronicles are silent on the current value, exile — fetch the wiki.\"]",
                );
            }
            system_blocks.push(json!({
                "type": "text",
                "text": state_block,
            }));
            if !build_sheet_line.is_empty() {
                let mut tag = format!("[{build_sheet_line}]");
                // Embed the lookup fingerprint right next to the status so
                // the model never has to recompute it (and risk a mismatch
                // with the deterministic server-side computation). `sheet_*`
                // tools accept `fingerprint=…` verbatim — copying this string
                // is the cheapest possible path.
                if let Some(fp) = &sheet_fingerprint_for_prompt {
                    tag.push_str(&format!("\n[Build fingerprint: {fp}]"));
                }
                if let Some(hash) = &sheet_pob_hash_for_prompt {
                    tag.push_str(&format!("\n[Build pob_hash: {hash}]"));
                }
                if last_user_was_interview_submission {
                    // Highest-priority directive: when the previous user
                    // turn was an interview submission, the only valid
                    // next move is to finalize. Surfaced before any
                    // status-specific directive so the model can't get
                    // confused by `Build sheet: absent` and re-launch the
                    // interview.
                    tag.push_str(
                        "\n[Submission directive: the user has just SUBMITTED a one-shot Build \
                         Sheet interview (their last message starts with `[INTERVIEW \
                         SUBMISSION ...]`). Your VERY NEXT tool call (after the iter-1 forced \
                         get_active_build) MUST be `sheet_finalize_request`. Pass the canonical \
                         hash from the `Build pob_hash:` tag above as the `pob_hash` field, the \
                         fingerprint from `Build fingerprint:` as the `fingerprint` field, and \
                         the section bodies from the `## Sections` block of the user message as \
                         the `sections[]` array (each entry: id + label + body + confirmed=true). \
                         Derive `defining_items[]`, `intent[]`, and `known_gaps[]` from the user's \
                         answers + your earlier analysis. After the call returns, answer the \
                         user's ORIGINAL question (the one they asked before the interview pivot) \
                         citing the persisted sheet — `read_from_sheet · key1 · key2` italic at \
                         the end. Do NOT call sheet_open_interview again under any circumstance \
                         in this turn — the user already submitted, looping is the bug we're \
                         fixing.]",
                    );
                } else if matches!(sheet_status_kind, SheetStatusKind::Absent) {
                    tag.push_str(
                        "\n[Sheet directive: a build is loaded but has no validated Build Sheet. \
                         If the user's question is build-specific (review / why-do-I-die / defence \
                         audit / itemisation / next-upgrade / content-feasibility), enter the \
                         one-shot interview flow: (1) deep analysis cap ~7 tool calls — \
                         pob_calc(defence), pob_calc(offence), read_internal_reference(\"thresholds/<tier>.md\"), \
                         and 2-3 wiki_parse / kb_search on the main skill + defining uniques; then \
                         (2) ONE call to sheet_open_interview with all 6 sections pre-drafted + 3-7 \
                         leverage questions across sections + a notes_prompt; then (3) end your \
                         turn silently — no prose. The user fills the panel in one round and the \
                         submission arrives next turn as a `[INTERVIEW SUBMISSION]` user message; \
                         parse it, call sheet_finalize_request once, then answer the original \
                         question citing the sheet. Do NOT use sheet_propose_section or sheet_ask \
                         for authoring — those are follow-up-edit tools only. Override only if the \
                         user explicitly says 'skip the sheet' or 'audit me from scratch'. See \
                         ref 32 for the full procedure.]",
                    );
                } else if matches!(sheet_status_kind, SheetStatusKind::Fresh) {
                    // Sprint v3 anti-duplication: when the sheet was already
                    // fetched earlier in this chat AND nothing has changed
                    // since (Fresh status), re-fetching would surface an
                    // identical payload. Detect prior tool_use and route the
                    // directive accordingly.
                    let sheet_already_fetched = messages_contain_tool_use(
                        &messages,
                        super::sheet_tools::GET_ACTIVE_BUILD_SHEET,
                    );
                    let build_already_fetched =
                        messages_contain_tool_use(&messages, super::tools::GET_ACTIVE_BUILD);
                    if sheet_already_fetched {
                        tag.push_str(
                            "\n[Sheet directive: a validated, fresh Build Sheet exists for this \
                             build, and it has ALREADY been fetched earlier in this conversation \
                             — its payload is in the prior tool_result blocks above. Do NOT call \
                             get_active_build_sheet again this turn; read from the existing \
                             payload in your context. Same for get_active_build (already called \
                             above) — the JSON snapshot is your source of truth. Skip pob_calc \
                             and threshold lookups unless the sheet does not cover the question. \
                             End the answer with `read_from_sheet · key1 · key2` in italic.]",
                        );
                        let _ = build_already_fetched;
                    } else {
                        tag.push_str(
                            "\n[Sheet directive: a validated, fresh Build Sheet exists for this \
                             build. For build-specific questions, call get_active_build_sheet \
                             once with the fingerprint above (copy verbatim) to fetch the body, \
                             then read from the sheet's sections (identity / archetype / damage \
                             / defense / items / intent) as the source of truth. Skip pob_calc \
                             and threshold lookups unless the sheet does not cover the question. \
                             End the answer with `read_from_sheet · key1 · key2` in italic.]",
                        );
                    }
                } else if matches!(sheet_status_kind, SheetStatusKind::Stale) {
                    let sheet_already_fetched = messages_contain_tool_use(
                        &messages,
                        super::sheet_tools::GET_ACTIVE_BUILD_SHEET,
                    );
                    if sheet_already_fetched {
                        tag.push_str(
                            "\n[Sheet directive: a Build Sheet exists but the PoB hash differs \
                             since authoring (gear / gem swaps). The sheet has ALREADY been \
                             fetched earlier in this conversation — its payload is in the prior \
                             tool_result blocks above. Do NOT call get_active_build_sheet again. \
                             Re-read intent / archetype / defining items / known gaps from that \
                             prior payload. For numerical claims (DPS, EHP, max-hit, resists), DO \
                             NOT use the sheet's stored numbers — call pob_calc fresh against \
                             the live build (which is also already in messages above as the \
                             get_active_build result). End with `read_from_sheet · keys` italic \
                             caption listing the sections you cited.]",
                        );
                    } else {
                        tag.push_str(
                            "\n[Sheet directive: a Build Sheet exists but the PoB hash differs since \
                             authoring (gear / gem swaps). Call get_active_build_sheet anyway with the \
                             fingerprint above. Intent / archetype / defining items / known gaps are \
                             authored decisions that don't go stale when an item swaps — only the \
                             numbers age. Open the answer with one short sentence acknowledging the \
                             drift, then cite the sheet's intent / archetype / items sections for \
                             context. For numerical claims (DPS, EHP, max-hit, resists, regen), DO \
                             NOT use the sheet's stored numbers — re-derive from the live PoB via \
                             pob_calc and present those. End with `read_from_sheet · keys` italic \
                             caption listing the sections you cited (intent / archetype / items, \
                             never numbers). Do NOT call sheet_open_interview to refresh — the user \
                             has a refresh button in the UI and triggers it themselves.]",
                        );
                    }
                }
                system_blocks.push(json!({
                    "type": "text",
                    "text": tag,
                }));
            }

            // Sprint D — force `get_active_build` on iteration 1 whenever a
            // build is loaded. Eliminates the "model decides not to call the
            // build context" failure class that the 2026-05-08 audit found in
            // 23/24 build runs.
            //
            // Sprint v3 anti-duplication: only force when the messages array
            // does NOT already contain a prior `get_active_build` tool_use.
            // The chat history carries every prior turn's tool calls + results,
            // so once the build context has been fetched ONCE in this chat,
            // every later turn already has the JSON payload in scope. Forcing
            // a fresh call on each turn (which the original logic did) just
            // burns tokens and surfaces an identical payload — the 2026-05-12
            // Juggernaut chat showed `get_active_build` re-called on every
            // user message, and the agent re-analysing what it already knew.
            let force_build_context = iterations == 1
                && !force_finalize
                && has_active_build
                && !messages_contain_tool_use(&messages, super::tools::GET_ACTIVE_BUILD);
            // Sprint UX-2 — deferred force on `sheet_open_interview`. The
            // legacy iter-2 force on `sheet_propose_section` was dropped
            // because the new flow requires the model to do deep analysis
            // FIRST (pob_calc, threshold lookup, wiki_parse on uniques)
            // before opening the one-shot interview. Forcing the interview
            // tool too early would skip that analysis and the panel's
            // pre-drafted bodies would be generic guesses. So we let the
            // model run analysis freely and only escalate if it stalls past
            // iter 8 without ever calling `sheet_open_interview`. The
            // `sheet_open_interview_emitted_already` scan walks the
            // messages array to detect a prior tool_use of that name so the
            // force fires at most once and never after a successful call.
            let sheet_open_interview_emitted_already =
                messages_contain_tool_use(&messages, super::sheet_tools::SHEET_OPEN_INTERVIEW);
            // Sprint v3 — only pivot to the sheet interview when the
            // deterministic classifier flagged the turn as DeepAudit. Brief
            // mechanic questions ("what's my fire res") and explicit
            // skip-the-sheet overrides bypass the interview entirely.
            let mode_allows_pivot = matches!(
                turn_mode,
                super::turn_classifier::TurnMode::DeepAudit
                    | super::turn_classifier::TurnMode::Default
            );
            let force_pivot_to_sheet = iterations > 8
                && !force_finalize
                && has_active_build
                && matches!(sheet_status_kind, SheetStatusKind::Absent)
                && !sheet_open_interview_emitted_already
                && !last_user_was_interview_submission
                && mode_allows_pivot;
            // 2026-05-09 — when the previous user turn was an interview
            // submission and the agent has just received the build context
            // (iter 2 of the post-submission turn), force
            // `sheet_finalize_request` directly. The bug we're patching:
            // even with a strong runtime directive in the system prompt,
            // the model occasionally re-launched `sheet_open_interview`
            // because `Build sheet: absent` was still true (the sheet
            // wasn't persisted yet). tool_choice closes that loophole.
            let sheet_finalize_emitted_already =
                messages_contain_tool_use(&messages, super::sheet_tools::SHEET_FINALIZE_REQUEST);
            let force_finalize_after_submission = iterations == 2
                && !force_finalize
                && has_active_build
                && last_user_was_interview_submission
                && !sheet_finalize_emitted_already;
            // Sprint v6 Reco 6 — force `pob_calc` on iter 2 when the user
            // asks an explicit quantitative mechanic question about their
            // build ("how much fire res", "what's my EHP", "am i capped").
            // Iter 1 already forces `get_active_build`, so by iter 2 the
            // model has the full build payload in scope; this second force
            // guarantees an authoritative numeric source (computed stats
            // + Calcs config) before the model commits to a number in
            // user-visible prose. The classifier sub-flag is the only
            // gate — non-quantitative BriefMechanic falls through to the
            // legacy free-choice path.
            //
            // Priority chain (top wins, all checked in the if/else below):
            //   force_build_context              (iter 1, foundational)
            //   force_finalize_after_submission  (iter 2 post-submission)
            //   force_mechanic_fetch             (iter 2 quantitative)
            //   force_pivot_to_sheet             (iter >8, deep audit)
            let pob_calc_emitted_already =
                messages_contain_tool_use(&messages, super::tools::POB_CALC);
            let force_mechanic_fetch = iterations == 2
                && !force_finalize
                && has_active_build
                && matches!(
                    turn_mode,
                    super::turn_classifier::TurnMode::BriefMechanic { quantitative: true }
                )
                && !pob_calc_emitted_already
                && !last_user_was_interview_submission;
            // 16384 leaves headroom for both Anthropic extended-thinking
            // and DeepSeek-reasoner, which can burn 2-3K tokens before
            // the user-visible draft starts. Whisperer audit (2026-05-09)
            // hit truncation mid-sentence at 8192 with thinking on.
            let mut body = json!({
                "model": self.model,
                "max_tokens": 16384,
                "system": system_blocks,
                "tools": if force_finalize { json!([]) } else { json!(schemas) },
                "stream": true,
                "messages": messages,
            });
            // Anthropic-compatible third-party endpoints (DeepSeek's
            // `api.deepseek.com/anthropic`, Z.ai, etc.) frequently route the
            // request to a backend that rejects the typed `tool_choice` field
            // ("deepseek-reasoner does not support this tool_choice"). Skip
            // the force on non-native endpoints; the system-prompt directives
            // (e.g. always call `get_active_build` before commenting on the
            // build) still steer the agent toward the right call, just
            // without the hard guarantee.
            let supports_forced_tool_choice = self.api_url == API_URL;
            if supports_forced_tool_choice {
                if force_build_context {
                    body["tool_choice"] = json!({
                        "type": "tool",
                        "name": super::tools::GET_ACTIVE_BUILD,
                    });
                } else if force_finalize_after_submission {
                    body["tool_choice"] = json!({
                        "type": "tool",
                        "name": crate::llm::sheet_tools::SHEET_FINALIZE_REQUEST,
                    });
                } else if force_mechanic_fetch {
                    // Sprint v6 Reco 6 — see priority chain above.
                    body["tool_choice"] = json!({
                        "type": "tool",
                        "name": super::tools::POB_CALC,
                    });
                } else if force_pivot_to_sheet {
                    body["tool_choice"] = json!({
                        "type": "tool",
                        "name": crate::llm::sheet_tools::SHEET_OPEN_INTERVIEW,
                    });
                }
            }

            let resp = self.post_with_retry(&body, &deltas).await?;

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
            // Thinking blocks must round-trip — DeepSeek's reasoner backend
            // (which `deepseek-v4-flash` maps onto via the Anthropic-compat
            // endpoint) and Anthropic's extended thinking both reject the
            // next iteration if the prior assistant turn's thinking content
            // isn't passed back verbatim. We capture text + signature here
            // so the replay block is byte-for-byte identical.
            let mut current_thinking_index: std::collections::BTreeMap<usize, ThinkingCapture> =
                std::collections::BTreeMap::new();
            let mut thinking_open: bool = false;
            let mut stop_reason: Option<String> = None;

            while let Some(chunk) = stream.next().await {
                let bytes = match chunk {
                    Ok(b) => b,
                    Err(e) => {
                        // Stream broke mid-flight — typical when DeepSeek's
                        // Anthropic-compat endpoint truncates the response on
                        // a large tool_use payload (sheet_open_interview with
                        // 6 sections + 3-7 questions can push the SSE body
                        // past their internal limit). Without explicit cleanup,
                        // any `current_tool_index` entry whose tool_use_start
                        // arrived but whose tool_use_end never came stays in
                        // `running` status forever in the chat UI, blocking
                        // the next turn. Emit ToolEnd(Failed) for each pending
                        // tool so the frontend resolves the orphans, then
                        // propagate the error.
                        let msg = format!("stream interrupted: {e}");
                        for (_idx, pending) in &current_tool_index {
                            let _ = deltas.send(LlmDelta::ToolEnd {
                                id: pending.id.clone(),
                                status: ToolStatus::Failed,
                                summary: Some(
                                    "tool call truncated — provider stream interrupted mid-tool"
                                        .to_string(),
                                ),
                            });
                        }
                        let _ = deltas.send(LlmDelta::Error(msg.clone()));
                        return Err(anyhow!(msg));
                    }
                };
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
                                data_line = Some(format!("{}{}", data_line.unwrap_or_default(), d));
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
                            let idx = v.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                            let block = v.get("content_block").cloned().unwrap_or(Value::Null);
                            let btype = block.get("type").and_then(|t| t.as_str()).unwrap_or("");
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
                            } else if btype == "thinking" {
                                if !thinking_open {
                                    let _ = deltas.send(LlmDelta::ReasoningBegin);
                                    thinking_open = true;
                                }
                                current_thinking_index.entry(idx).or_insert(
                                    ThinkingCapture::Open {
                                        text: String::new(),
                                        signature: String::new(),
                                    },
                                );
                            } else if btype == "redacted_thinking" {
                                if !thinking_open {
                                    let _ = deltas.send(LlmDelta::ReasoningBegin);
                                    thinking_open = true;
                                }
                                let data = block
                                    .get("data")
                                    .and_then(|d| d.as_str())
                                    .unwrap_or("")
                                    .to_string();
                                current_thinking_index
                                    .insert(idx, ThinkingCapture::Redacted { data });
                            }
                        }
                        "content_block_delta" => {
                            let idx = v.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                            if let Some(d) = v.get("delta") {
                                let dtype = d.get("type").and_then(|t| t.as_str()).unwrap_or("");
                                match dtype {
                                    "text_delta" => {
                                        if let Some(text) = d.get("text").and_then(|t| t.as_str()) {
                                            current_text_index_text
                                                .entry(idx)
                                                .or_default()
                                                .push_str(text);
                                            let _ =
                                                deltas.send(LlmDelta::TextDelta(text.to_string()));
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
                                            if let Some(ThinkingCapture::Open {
                                                text: buf, ..
                                            }) = current_thinking_index.get_mut(&idx)
                                            {
                                                buf.push_str(text);
                                            }
                                            let _ = deltas
                                                .send(LlmDelta::ReasoningDelta(text.to_string()));
                                        }
                                    }
                                    "signature_delta" => {
                                        if let Some(sig) =
                                            d.get("signature").and_then(|s| s.as_str())
                                        {
                                            if let Some(ThinkingCapture::Open {
                                                signature, ..
                                            }) = current_thinking_index.get_mut(&idx)
                                            {
                                                signature.push_str(sig);
                                            }
                                        }
                                    }
                                    "input_json_delta" => {
                                        if let Some(part) =
                                            d.get("partial_json").and_then(|p| p.as_str())
                                        {
                                            if let Some(t) = current_tool_index.get_mut(&idx) {
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
                            let idx = v.get("index").and_then(|i| i.as_u64()).unwrap_or(0) as usize;
                            if let Some(t) = current_tool_index.remove(&idx) {
                                if let Ok(input_value) =
                                    serde_json::from_str::<Value>(&t.input_json)
                                {
                                    if let Some(summary_input) =
                                        super::util::summarize_tool_args(&t.name, &input_value)
                                    {
                                        let _ = deltas.send(LlmDelta::ToolDetailUpdate {
                                            id: t.id.clone(),
                                            summary_input,
                                        });
                                    }
                                }
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
            // Thinking blocks must come first in the replay — Anthropic's
            // extended thinking spec and DeepSeek's reasoner protocol both
            // expect them before text/tool_use blocks. Iterating the BTreeMap
            // by ascending index preserves the original ordering when the
            // model returns several thinking blocks in a single turn.
            for (_idx, capture) in &current_thinking_index {
                let block = match capture {
                    ThinkingCapture::Open { text, signature } => {
                        let mut obj = json!({
                            "type": "thinking",
                            "thinking": text,
                        });
                        if !signature.is_empty() {
                            obj["signature"] = json!(signature);
                        }
                        obj
                    }
                    ThinkingCapture::Redacted { data } => json!({
                        "type": "redacted_thinking",
                        "data": data,
                    }),
                };
                assistant_content.push(block);
            }
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
                    let (result, status) = match dispatch(&tu.name, &parsed_input, &tool_ctx).await
                    {
                        Ok(s) => (s, ToolStatus::Done),
                        Err(e) => (
                            // `{:#}` walks the anyhow context chain (outermost
                            // → innermost, separated by `: `) so the LLM sees
                            // the *real* underlying message — e.g. the GGG
                            // trade API's `Unknown stat provided: …`. Without
                            // this, `to_string()` returns only the outermost
                            // `.context(...)` label and silently drops every
                            // useful diagnostic the agent could act on.
                            json!({"error": format!("{:#}", e)}).to_string(),
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

        // Sprint v6 Phase 6 — provider-agnostic post-stream pipeline.
        // Counts fetch tool calls (for the verifier gate), runs the live
        // response-lint and emits `LintFindings`, parses the tool
        // transcript for the verifier. Centralised in `post_stream.rs` so
        // `ollama.rs` can reuse it without duplicating the logic.
        let outcome = super::post_stream::run_shared_post_stream(
            &messages,
            &full_assistant,
            &deltas,
            "anthropic",
        );

        // Sprint G — conditional verifier. Runs only if `should_verify` matches
        // (numbers / sources / cached / banned phrases / identity / patch /
        // tier). Latency budget: ~2s typical; never propagates errors back to
        // the user (a failing verifier is a `pass_with_note`, not a failure).
        let verified_text = self
            .run_verifier_pass(
                &history,
                &full_assistant,
                outcome.fetch_tool_calls,
                &outcome.transcript,
                &deltas,
            )
            .await;
        let _ = deltas.send(LlmDelta::MessageEnd);
        Ok(verified_text)
    }

    /// Sprint v6 Phase 6 — promoted from `async fn` private to `pub(crate)
    /// async fn` so `ollama.rs` can re-use the Anthropic-backed CoVe
    /// verifier when `ANTHROPIC_API_KEY` is set in the environment.
    pub(crate) async fn run_verifier_pass(
        &self,
        history: &[ChatMessage],
        draft: &str,
        fetch_tool_calls: usize,
        transcript: &super::verifier::ToolTranscript,
        deltas: &mpsc::UnboundedSender<LlmDelta>,
    ) -> String {
        // Cheap heuristic gate first — most drafts skip the API call entirely.
        if !super::verifier::should_verify_with_context(draft, fetch_tool_calls) {
            return draft.to_string();
        }
        // User toggle: when off, the CoVe pipeline is skipped wholesale and
        // the draft ships as-is. The setting is read fresh on every turn so
        // toggling in the UI takes effect without a restart.
        if !crate::settings::is_verify_enabled() {
            tracing::debug!(
                target: "bestel.verifier",
                "verifier disabled by user setting"
            );
            return draft.to_string();
        }
        let last_user = history
            .iter()
            .rev()
            .find(|m| matches!(m.role, super::Role::User))
            .map(|m| m.content.as_str())
            .unwrap_or("");
        let verdict = super::verifier::verify_factored(
            &self.http,
            &self.api_url,
            &self.api_key,
            API_VERSION,
            &self.model,
            last_user,
            draft,
            fetch_tool_calls,
            Some(transcript),
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
                // Verdict surfaces only via `LlmDelta::Verifier` (consumed
                // by the dev panel badge). Appending a `[verifier: …]`
                // TextDelta would leak into the visible chat reply.
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
                // Same reasoning as the Revise arm: dev panel learns
                // about the rejection through `LlmDelta::Verifier`; we
                // do not emit a `[verifier: rejected — …]` block into
                // the visible reply.
                format!(
                    "The chronicler must withhold that answer, exile — the verifier flagged it ({summary}). Ask again with a narrower question, or attach a fresh PoB so the engine can ground the numbers."
                )
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
            let retryable =
                status == reqwest::StatusCode::TOO_MANY_REQUESTS || status.is_server_error();
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

// Sprint v6 Phase 6 — `count_fetch_tool_calls`, `segments_from_messages`,
// and `build_partial_lint_run` were moved to `crates/bestel-core/src/llm/
// post_stream.rs` so `ollama.rs` can reuse the live response-lint and
// `ToolTranscript` construction without duplicating logic.

/// Walk the assembled `messages` array and return true when ANY assistant
/// message has already produced a `tool_use` block whose `name` equals the
/// supplied tool name. Used by the Sprint UX-2 deferred-force logic in the
/// agent loop: we only force `sheet_open_interview` when the model has not
/// already shipped one, so the escalation is a safety net rather than a
/// repeat-call on every iteration past the cap.
///
/// The shape we match is Anthropic's request body convention:
/// `messages[i].role == "assistant"` AND `messages[i].content` is an array
/// of content blocks AND at least one block has `type == "tool_use"` AND
/// `name == <target>`.
fn messages_contain_tool_use(messages: &[Value], tool_name: &str) -> bool {
    for m in messages {
        if m.get("role").and_then(|v| v.as_str()) != Some("assistant") {
            continue;
        }
        let Some(content) = m.get("content").and_then(|v| v.as_array()) else {
            continue;
        };
        for block in content {
            let is_tool_use = block.get("type").and_then(|v| v.as_str()) == Some("tool_use");
            if !is_tool_use {
                continue;
            }
            if block.get("name").and_then(|v| v.as_str()) == Some(tool_name) {
                return true;
            }
        }
    }
    false
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

/// Captures a `thinking` or `redacted_thinking` content block during
/// streaming so it can be replayed verbatim in the next iteration's
/// assistant message — required by Anthropic extended thinking and by
/// DeepSeek's reasoner backend.
#[derive(Debug, Clone)]
enum ThinkingCapture {
    /// Plain thinking with optional signature accumulated from
    /// `signature_delta` events.
    Open { text: String, signature: String },
    /// Safety-redacted thinking — opaque `data` payload that must be
    /// echoed back as-is.
    Redacted { data: String },
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

/// Heuristic for the Sprint `value-purge` mechanics directive: returns true
/// when the user's question looks "mechanic-coded" — i.e. asks for a value
/// or names a known value-bearing entity. False positives over-trigger the
/// directive (forcing an extra fetch, acceptable failure mode); false
/// negatives let through bare mechanic answers, caught by the verifier's
/// `has_unsourced_numeric_claim` heuristic post-draft.
///
/// Input is expected to already be lowercased.
fn is_mechanic_question(lower: &str) -> bool {
    const VALUE_QUESTION_CUES: &[&str] = &[
        "how much",
        "how many",
        "what's the",
        "what is the",
        "what does",
        "how long",
        "how often",
        "%",
        "percent",
        "cooldown",
        "charges",
        "socket",
        "duration",
        "drop level",
        " tier",
        "implicit",
        "base stat",
        "breakpoint",
        "cap on",
        "max ",
        "regen",
    ];
    // Known value-bearing entities — names that historically pull the model
    // into reciting a stale stat (sourced from the 2026-05-12 audit's hit
    // list). When any of these appear in the question, we always surface
    // the directive even if the cue list misses.
    const VALUE_ENTITIES: &[&str] = &[
        "soul of solaris",
        "mind over matter",
        "spell suppression",
        "spine bow",
        "bone helmet",
        "marble amulet",
        "trinity",
        "flame dash",
        "elemental overload",
        "cast on crit",
        "cast on critical",
        "righteous fire",
        "voices",
        "cluster jewel",
        "eldritch implicit",
        "watcher's eye",
        "tabula rasa",
        "lineage support",
        "resolute technique",
        "pain attunement",
        "fracturing orb",
        "awakener's orb",
    ];
    VALUE_QUESTION_CUES.iter().any(|c| lower.contains(c))
        || VALUE_ENTITIES.iter().any(|e| lower.contains(e))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn mechanic_question_detects_value_cues() {
        assert!(is_mechanic_question(
            "how much does mind over matter actually mitigate"
        ));
        assert!(is_mechanic_question("what's the cap on spell suppression"));
        assert!(is_mechanic_question(
            "is spine bow really 1.5 attacks per second"
        ));
        assert!(is_mechanic_question(
            "how many flame dash charges and what's the cd"
        ));
        assert!(is_mechanic_question(
            "what does trinity support actually give"
        ));
    }

    #[test]
    fn mechanic_question_ignores_pure_narrative() {
        assert!(!is_mechanic_question("my build feels bad"));
        assert!(!is_mechanic_question("should i go pathfinder or trickster"));
        assert!(!is_mechanic_question("how was your day exile"));
        assert!(!is_mechanic_question(""));
    }

    #[test]
    fn mechanic_question_detects_value_entities() {
        assert!(is_mechanic_question("tell me about bone helmet"));
        assert!(is_mechanic_question("voices unique jewel"));
        assert!(is_mechanic_question("i'm running righteous fire chieftain"));
    }
}
