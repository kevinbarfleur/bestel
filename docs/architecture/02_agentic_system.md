# 02 — Agentic system

The brain of Bestel. How a single user message becomes an answer:
provider selection, streaming, the tool loop, verification, and what
gets written to disk along the way.

This is the document the user asked for first — read it before any of
the others if your goal is to *change agent behaviour*.

---

## Chat flow end-to-end

```
User types message
   │
   ▼
ChatComposer.vue ──invoke──► chat_start (commands.rs)
                                  │
                                  ├─ resolve active model profile (model.json)
                                  ├─ build provider (anthropic | ollama)
                                  ├─ compose system blocks (3 cached + 1 dynamic)
                                  ├─ allocate (delta_tx, delta_rx) + oneshot cancel
                                  ├─ store cancel_tx in AppState
                                  ├─ create Recorder, stash in pump
                                  ├─ spawn provider.run(history, ctx, delta_tx)
                                  ├─ spawn pump_deltas(delta_rx, cancel, recorder)
                                  └─ return new session_id (u64)
                                            │
            ┌───────────────────────────────┴─────────────────────────────┐
            ▼                                                               ▼
provider.run (tokio task)                                pump_deltas (tokio task)
   │                                                          │
   ├── loop {                                                  │
   │     POST messages.create (SSE / NDJSON)                   │
   │     decode chunks → LlmDelta on delta_tx ───────────────► │ ─── feed Recorder
   │     if stop_reason == "tool_use":                         │     │
   │       run BuildContext::dispatch_tool                     │     ▼
   │       append tool_use + tool_result blocks                │ window.emit("llm:delta", DeltaEvent)
   │       iterate                                             │
   │     else: break                                           │
   │   } (cap: MAX_AGENT_ITERATIONS = 16, then 1 wrap-up turn) │
   │                                                            │
   ├── if should_verify(final_text): verify_factored() ─────► Verifier delta
   ├── emit Usage(UsageStats)                                  │
   ├── emit MessageEnd                                         │
   └── close delta_tx                                          │
                                                          recorder.finalize()
                                                          write ~/.bestel/runtime/debug-chats/<id>.json
```

The frontend `chat.ts` store has been listening for `llm:delta` since
mount; it switches on `DeltaEvent` variant and patches the streaming
message in place.

---

## Provider abstraction

`crates/bestel-core/src/llm/mod.rs` exposes:

```rust
pub enum Provider {
    Anthropic(AnthropicClient),
    Ollama(OllamaClient),
}
```

Two enum variants only. **DeepSeek goes through `AnthropicClient`** —
DeepSeek exposes an Anthropic-compatible endpoint at
`https://api.deepseek.com/anthropic`, so we just point the existing
client at a different `base_url` and use the `DEEPSEEK_API_KEY` env var.
The Codex CLI provider mentioned in older code has been removed.

Each provider has its own native event schema (Anthropic structured
thinking blocks, Ollama NDJSON tool-call accumulation, etc.). Its
parser translates events into the shared [`LlmDelta`](#llmdelta)
enum — every downstream consumer (recorder, pump, UI) deals with
`LlmDelta` only. Swapping a provider, adding a local model, or
experimenting with another HTTP API touches one parser only.

### `Provider::run`

```rust
pub async fn run(
    &self,
    history: Vec<ChatMessage>,
    ctx: BuildContext,
    deltas: mpsc::UnboundedSender<LlmDelta>,
) -> Result<String>
```

- `history` is the full conversation so far (user / assistant
  alternation). Attachments are inlined per message — images as
  base64 content blocks, text/PDFs as text blocks with a header
  preamble.
- `ctx` is the `BuildContext` (`Arc<RwLock<Option<PobBuild>>>`) shared
  with the watcher and tools.
- `deltas` is the half-duplex channel the provider writes into. The
  pump owns the receiver.

The returned `String` is the assistant's final text (also emitted via
`LlmDelta::TextDelta` chunks during streaming).

---

## LlmDelta

The central streaming enum. **17 variants** in `crates/bestel-core/src/llm/mod.rs`:

| Variant | Purpose |
|---|---|
| `TextDelta(String)` | Streaming token chunk for the visible answer |
| `ReasoningBegin` | Anthropic extended-thinking / DeepSeek reasoning block start |
| `ReasoningDelta(String)` | Reasoning text chunk (collapsible in UI) |
| `ReasoningEnd` | End of reasoning block |
| `ToolBegin { id, name, detail }` | Tool call requested by the model |
| `ToolDetailUpdate { id, summary_input }` | Late-arriving formatted view of the input args (Anthropic streams input JSON in pieces — finalised at `content_block_stop`) |
| `ToolOutput { id, chunk }` | Streamed tool output chunk |
| `ToolEnd { id, status, summary }` | Tool finished — status is `Running | Done | Failed`, summary is one-line |
| `Usage(UsageStats)` | Final token + cost accounting for the full agent run |
| `Verifier(VerifierVerdict)` | CoVe verdict emitted after streaming, before MessageEnd |
| `SheetDraftUpdate { section_id, title, body, confirmed }` | Build Sheets — agent drafted / re-drafted a section |
| `SheetAskUser { question_id, title, subtitle, options, multi, has_other }` | Build Sheets — agent surfaces a picker question |
| `SheetInterviewOpen { payload }` | Build Sheets — agent opens the full one-shot interview panel |
| `SheetFinalized { sheet_id, name }` | Build Sheets — sheet persisted to SQLite |
| `SheetLoaded { sheet_id, fingerprint, name, pob_hash, stale, authored_at, updated_at, schema_version, payload }` | Build Sheets — active sheet hydrated for the current chat |
| `MessageEnd` | Run complete |
| `Error(String)` | Fatal error (network, parse, cancelled) |

`UsageStats` carries `input_tokens`, `cached_input_tokens`,
`cache_creation_tokens`, `output_tokens`, and `cost_usd`. Cache fields
are populated only by Anthropic / DeepSeek (they report them); Ollama
leaves them at zero.

### DeltaEvent on the wire

The Tauri layer wraps `LlmDelta` into `crate::dto::DeltaEvent` — a
variant enum serialized as discriminated JSON — before emitting the
`llm:delta` window event. Extra UI-only variants exist:

- `Cancelled { session_id }` — emitted by the pump when the cancel
  oneshot fires
- `Completed { session_id }` — emitted by the pump after `MessageEnd`

So the frontend sees 17 + 2 = 19 distinct event shapes. The chat store
dispatches on `kind` and patches the streaming message in place.

---

## Tool loop

`AnthropicClient::run` is the canonical implementation
(`crates/bestel-core/src/llm/anthropic.rs`). Ollama's loop mirrors the
same structure with NDJSON instead of SSE.

```rust
const MAX_AGENT_ITERATIONS: usize = 16;
const WRAP_UP_NUDGE: &str =
    "[system: tool budget reached. Give your final answer now \
     using what you've already gathered. Do not request more tools.]";
```

Why 16: DeepSeek and other tool-eager models routinely chain 8–12
`wiki_parse` + `repoe_lookup` + `read_internal_reference` calls before
answering. 16 leaves headroom for thoroughness without runaway loops.

Per-turn structure (one full LLM iteration):

1. Build `system` array: three cached blocks (BP2 persona+contract, BP3
   tool policy + answer-mode router + contracts + failure policy + per
   provider/model overrides, BP4 CORE_KNOWLEDGE + skill descriptions)
   each with `cache_control: ephemeral, ttl: 1h` for prompt-caching.
2. Append a dynamic state block (no cache_control) carrying:
   - `Build state: loaded — <game> <class> lvl <n>` or
     `Build state: detached`
   - When a build is loaded: a runtime directive forcing the agent to
     cite ONLY values present in the `get_active_build` JSON (the
     "anti-hallucination anchor" from Sprint M, 2026-05-10)
   - `Build sheet: validated, fresh (id=N)` / `stale (id=N, hash drift)`
     / `absent (fingerprint=…)` — pre-computed so the agent doesn't
     have to call `get_active_build_sheet` to know the status
   - When the most recent user message contains `[INTERVIEW
     SUBMISSION`, a directive forcing `sheet_finalize_request` instead
     of relaunching the interview
3. POST to `https://api.anthropic.com/v1/messages` (or the DeepSeek
   compat URL). Streaming SSE.
4. Parse SSE events. Emit `LlmDelta` on the channel:
   - `content_block_start (type: thinking)` → `ReasoningBegin`
   - `content_block_delta (thinking_delta)` → `ReasoningDelta`
   - `content_block_delta (text_delta)` → `TextDelta`
   - `content_block_start (type: tool_use)` → `ToolBegin`
   - `input_json_delta` → accumulate, emit `ToolDetailUpdate` at
     `content_block_stop` once the JSON is complete
   - `message_delta (stop_reason)` → record stop reason
5. If `stop_reason == "tool_use"`:
   - For each accumulated tool block, run
     `BuildContext::dispatch_tool(name, args)`. The dispatch is async
     for network tools (`wiki_*`, `web_fetch`, `pob_calc`, `kb_search`)
     and sync for pure local ones (`get_active_build`,
     `read_internal_reference`).
   - Stream `ToolOutput` chunks, end with `ToolEnd` (Done / Failed).
   - Append an assistant `tool_use` block + a user `tool_result` block
     to the history.
   - Loop.
6. Else (text or `end_turn`): exit the loop.
7. After loop: emit `Usage(UsageStats)`, optionally run the
   [verifier](#verifier-cove), emit `MessageEnd`, close the channel.

### Forced wrap-up

If the loop hits `MAX_AGENT_ITERATIONS + 1`:

- The provider injects a `user` message with `WRAP_UP_NUDGE`.
- It calls the API one more time with `tools: []` so the model
  *cannot* request another tool.
- Whatever final text comes back ships to the user.

This guarantees a final answer instead of a bare error in the rare
case the model genuinely cannot stop.

### Per-turn caps inside tools

Two tools enforce additional caps inside the loop:

- `kb_search` — capped at `KB_SEARCH_TURN_CAP = 3` invocations per
  turn. Beyond that, dispatch returns the structural fallback
  `"tool_storm_kb_search"` so the model gets a clean signal to stop.
- `load_skill` — capped at `LOAD_SKILL_TURN_CAP = 2` per turn.
  Guards against skill-load thrash; the model can ask for two only
  when the user explicitly requested a full multi-skill audit.

Counters are `Arc<AtomicU32>` on `ToolCtx`, reset implicitly because a
new `ToolCtx` is constructed per LLM iteration.

---

## Cache breakpoints

`crates/bestel-core/src/prompts.rs::load_anthropic_blocks` splits the
on-disk `SYSTEM_PROMPT.md` on the literal token `<tool_policy>`:

| Block | Contents | Why it's its own breakpoint |
|---|---|---|
| BP2 | Persona, hard rules, runtime contract (everything before `<tool_policy>`) | Rarely changes; full session reuse |
| BP3 | Tool policy + answer-mode router + output contracts + failure policy + per-provider override + per-model override | Changes when tools or contracts evolve. Per-provider / per-model overrides ride along here so they share the same cache entry (overrides are stable for a given `(provider, model)` pair) |
| BP4 | `CORE_KNOWLEDGE.md` + bundled skill-descriptions block | Changes when the reference library is re-indexed |

All three blocks get `cache_control: { type: ephemeral, ttl: "1h" }`.
A 1h TTL covers desktop sessions that span more than five minutes (the
default Anthropic cache TTL); shorter sessions still get the benefit on
the same chat thread.

The dynamic state block is appended **after** the cached blocks with
no `cache_control`, so flipping a build between turns does not
invalidate any cache entry.

---

## Cancellation

Two layers:

- **Pump-side.** `chat_cancel` consumes the
  `oneshot::Sender<()>` stored in `AppState.inner.cancel_tx`. The pump's
  `tokio::select { biased; … }` checks the cancel receiver before each
  channel read, emits `DeltaEvent::Cancelled`, and returns. The recorder
  is `finalize`d as cancelled. The provider task is left to wind down
  on its own (its writes to a closed channel become no-ops).
- **Provider-side.** Each provider also accepts its own cancel channel
  in newer code; it checks between SSE chunks. If triggered it emits
  `LlmDelta::Error("cancelled")` followed by `MessageEnd` and returns.
  The pump path is the user-observable one; the provider path matters
  for cleanly releasing the HTTP request.

The HTTP client's overall timeout is **1800 s** (30 min) — long
agentic turns with DeepSeek-V4-Pro have been observed to exceed 190 s
of reasoning before the model emits `sheet_open_interview`. The
connect timeout stays at **10 s** so dead endpoints fail fast.

---

## Verifier (CoVe)

`crates/bestel-core/src/llm/verifier.rs` implements **Chain-of-Verification**
(Meta, ACL 2024, arxiv 2309.11495). It replaces Sprint G's single-pass
consistency check with a 3-step retrieval-grounded pipeline plus an
optional 4th revise pass.

### When it fires

Two-tier gate, both byte-scan heuristics (no API call):

- `should_verify(draft)` — original Sprint G triggers:
  - Contains `Sources:` block
  - `(cached)` marker
  - Banned phrases (`real DPS`, `actual DPS`, `live engine result`,
    `verified DPS`)
  - Numerical claim with PoE units (DPS, EHP, max-hit, life, ES, …)
  - Tier marker (T1–T17, tier 1/2/3)
  - Patch version + context word (patch / league / poe1 / poe2 /
    Settlers / Affliction / Kalandra / Ancestor / Necropolis)
  - `Identity:` line (build-grounded answer)
- `should_verify_with_context(draft, fetch_tool_calls)` — Sprint
  `value-purge` extension: *additionally* fires when the draft makes a
  numeric mechanic claim AND the agent did NOT call any fetch tool
  (`wiki_parse`, `wiki_cargo`, `repoe_lookup`, `pob_calc`, `web_fetch`)
  this turn. The bundled references no longer carry hardcoded values,
  so an unsourced number is recalled from training data — exactly the
  failure mode this catches.

Brief, off-topic, or pure-narrative answers short-circuit to
`VerifierVerdict::pass_empty()` with zero API cost.

### Pipeline

```
draft, fetch_tool_calls
   │
   ▼
should_verify_with_context? ── no ─► pass_empty()
   │ yes
   ▼
extract_claims (1 LLM call, ≤ 7 atomic claims, MAX_CLAIMS = 7)
   │
   ▼
gather_evidence (KB local, deterministic, no LLM)
   per claim: kb.search(topic, KB_TOP_K_PER_CLAIM = 3)
   evidence truncated to MAX_EVIDENCE_PER_CLAIM_CHARS = 500
   │
   ▼
judge_batch (1 LLM call, all claims+evidence in one prompt)
   per claim: ok | wrong | unverified (+ correction sentence)
   │
   ▼
any "wrong"? ── no ─► VerdictStatus::Pass with claims_checked
   │ yes
   ▼
revise_draft (1 LLM call, splice corrections into draft verbatim)
   │
   ▼
VerdictStatus::Revise with minimal_rewrite
```

Total: **2–4 sub-calls per verified turn**, each 500–2000 tokens.
Total overhead ~10–20% vs the original ~5% Sprint G cost.

### Hard guardrails

- `HARD_TIMEOUT_SECS = 12` whole-pipeline timeout — if any sub-call
  stalls, fall back to `pass_with_note` so the user-facing reply
  ships.
- Every failure path collapses into `pass_with_note(error_msg)`. We
  never promote a failed verifier into a false `revise`.
- The model used for verifier sub-calls is the *same* model running
  the user-facing run. No cross-model judging.

### Verdict shape

```rust
pub struct VerifierVerdict {
    pub status: VerdictStatus,                  // Pass | Revise | Fail
    pub findings: Vec<VerifierFinding>,         // category, issue, evidence
    pub minimal_rewrite: String,                // populated on Revise
    pub claims_checked: Vec<VerifiedClaim>,     // audit trail for UI
    pub corrections_count: usize,
}

pub struct VerifiedClaim {
    pub statement: String,
    pub topic: String,                          // search query used for KB
    pub status: ClaimStatus,                    // Ok | Wrong | Unverified
    pub evidence_excerpt: String,               // truncated KB hit
    pub correction: Option<String>,             // single-sentence fix
}
```

### Surfacing

- `LlmDelta::Verifier(verdict)` is emitted on the channel right before
  `MessageEnd`.
- The chat store renders a `VerifyClaimsCard.vue` with status pill and
  per-claim audit rows.
- The `Recorder` stores the verdict in `ChatStats.verifier_verdict` so
  the dev panel can show it on past runs.
- The user can disable verification via Settings → Developer pane,
  which calls `settings_set_verify_enabled(false)` and persists to
  `~/.bestel/runtime/settings.json`.

---

## Recorder

`crates/bestel-core/src/llm/recorder.rs` mirrors every `LlmDelta` into
a structured `PersistedRun`:

```rust
pub struct PersistedRun {
    pub id: String,
    pub started_at: String,
    pub ended_at: Option<String>,
    pub source: String,                              // "ui" | "smoke" | "battery"
    pub provider: String,
    pub model_id: String,
    pub model_display_name: String,
    pub user_text: String,
    pub user_attachments: Vec<PersistedAttachment>,
    pub assistant_segments: Vec<PersistedSegment>,   // Text | Reasoning | Tool
    pub final_text: String,
    pub stats: ChatStats,
    pub error: Option<String>,
}

pub struct ChatStats {
    pub elapsed_ms: u64,
    pub text_bytes: usize,
    pub reasoning_bytes: usize,
    pub tool_calls: usize,
    pub tool_done: usize,
    pub tool_failed: usize,
    pub input_tokens: u64,
    pub cached_input_tokens: u64,
    pub cache_creation_tokens: u64,
    pub output_tokens: u64,
    pub cost_usd: Option<f64>,
    pub verifier_verdict: Option<VerifierVerdict>,
}
```

Path: `~/.bestel/runtime/debug-chats/<id>.json`. Write is atomic
(`tempfile + rename`) so a crash mid-write never produces a torn JSON.

The dev panel reads this directory; `bestel run-battery` and
`bestel eval-judge` use the same `Recorder` so headless runs and UI
runs are indistinguishable in the inbox.

---

## Provider profiles

`crates/bestel-core/src/llm/models.rs` defines five static cloud
profiles. Ollama profiles are built dynamically from `/api/tags` so the
picker only ever shows models the user actually has.

| id | model_id | provider | base_url | cost $/Mtok (in / out) | Vision |
|---|---|---|---|---|---|
| `deepseek-v4-flash` | `deepseek-v4-flash` | Anthropic-compat | `https://api.deepseek.com/anthropic` | 0.14 / 0.28 | no |
| `deepseek-v4-pro` | `deepseek-v4-pro` | Anthropic-compat | `https://api.deepseek.com/anthropic` | 1.74 / 3.48 | no |
| `anthropic-haiku-4-5` | `claude-haiku-4-5-20251001` | Anthropic | (default) | 1.0 / 5.0 | yes |
| `anthropic-sonnet-4-6` | `claude-sonnet-4-6` | Anthropic | (default) | 3.0 / 15.0 | yes |
| `anthropic-opus-4-7` | `claude-opus-4-7` | Anthropic | (default) | 5.0 / 25.0 | yes |
| `ollama:<tag>` | `<tag>` | Ollama | `http://127.0.0.1:11434` | free | family-detected |

Default = `anthropic-haiku-4-5` (~90% of Sonnet quality at 1/3 the
price). DeepSeek is cheaper but requires its own key.

Persistence: `~/.bestel/runtime/model.json` stores `{ profile_id: "…" }`.

Picker list construction: `list_profiles_with_local()` returns
`cloud_profiles()` concatenated with Ollama probe results. If the
Ollama daemon is unreachable, only cloud profiles are returned (no
fictional `ollama pull X` ghosts).

`resolved_model_for(provider)` lets `BESTEL_MODEL` override the
selection for one-off testing.

---

## API keys

`crates/bestel-core/src/llm/keys.rs`:

- Strict whitelist: `ANTHROPIC_API_KEY`, `DEEPSEEK_API_KEY`.
- Persisted to `~/.bestel/runtime/keys.json` in plaintext.
- `apply_to_env()` injects them into `std::env::set_var` at startup,
  **before** any subcommand runs, so the Tauri app, the battery
  runner, and `mcp-serve` all see them.
- `list_api_keys` returns `KeyStatusDto { env_name, set,
  masked_preview }`. `masked_preview` is `Some("sk-ant-…4f2c")` for
  user-set keys, `Some("(from environment)")` for OS-env-only keys
  (we never echo the raw OS value), `None` when unset.

`set_api_key` / `delete_api_key` write through to disk and re-apply to
env.

---

## Detect providers

`crates/bestel-core/src/llm/detect.rs`:

- Pings Ollama (`GET /api/tags`).
- Checks `ANTHROPIC_API_KEY` presence.
- Checks `DEEPSEEK_API_KEY` presence.
- Emits `provider:status` (constant `PROVIDER_STATUS`) carrying
  `ProviderStatusEvent { detection: { anthropic, deepseek, ollama: { reachable, model_count } } }`.

The topbar greys out unavailable providers; the settings model picker
disables their rows.

`detect_providers` runs once at startup and again on any key change.

---

## Override hierarchy

Three system-prompt scopes, applied in order:

1. **Global** (`~/.bestel/prompts/SYSTEM_PROMPT.md` + `CORE_KNOWLEDGE.md`)
   — always loaded.
2. **Per-provider** (`~/.bestel/prompts/providers/<provider>.md`) —
   loaded when the chosen profile's provider matches (`anthropic.md`,
   `ollama.md`). Anthropic-compat profiles like DeepSeek match
   `anthropic.md` because the underlying provider enum is `Anthropic`.
3. **Per-model** (`~/.bestel/prompts/models/<sanitized_model_id>.md`)
   — narrowest scope. Filename is the sanitized model id (lowercase,
   `/`/`:`/`\` replaced with `-`).

For Anthropic-shaped requests, provider + model overrides are
**appended to BP3** so they ride the same cache breakpoint as the tool
policy (overrides are stable per `(provider, model)` pair).

A **baked-in addendum** also exists:
`crates/bestel-core/src/llm/local_addendum.md` — Ollama-specific tool
restrictions, included via `include_str!` and seeded to
`~/.bestel/prompts/local_addendum.md` on first launch.

---

## MCP server

`crates/bestel-core/src/mcp/` exposes Bestel's tools to other agents
via JSON-RPC over stdio (`rmcp` crate). Entry point:
`bestel mcp-serve`.

- Methods: `initialize`, `notifications/initialized`, `tools/list`,
  `tools/call`, `ping`, `shutdown`, `exit`.
- Tools exposed: all 13 tools from `tools.rs` (see
  [04 — Tools catalogue](./04_tools_catalogue.md)). Sheet tools are
  *not* exposed (they need a UI to be useful).
- No `pob_engine` available in this path (no Tauri AppHandle, no
  bundled-resource path resolution), so `pob_calc` returns a clear
  "engine not configured" error.

Used by external orchestrators that want to drive Bestel's research
tools without depending on the desktop app.

---

## See also

- [04 — Tools catalogue](./04_tools_catalogue.md) — the 18 tools the
  agent can invoke during the loop.
- [03 — Knowledge layer and RAG](./03_knowledge_layer_and_rag.md) —
  the prompts, references, and the LanceDB KB the verifier uses.
- [06 — Build Sheet workflow](./06_build_sheet_workflow.md) — what the
  five `SheetXxx` deltas drive on the UI.
- [07 — Persistence](./07_persistence_and_storage.md) — where
  `PersistedRun` lands and what else is on disk.
