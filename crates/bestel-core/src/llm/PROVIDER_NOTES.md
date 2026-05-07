# Provider parsing notes

Each LLM provider streams a different event schema. Bestel normalises them
into the shared `LlmDelta` enum, but the *parser* per provider has to know
the native shape. This file collects what we've learned so future-you (or
future-us) doesn't re-discover it.

> Rule of thumb: **the only file that should know about a provider's wire
> format is `crates/bestel-core/src/llm/<provider>.rs`**. Everything else
> consumes `LlmDelta`.

---

## Tool dispatch — both shipped providers

Both shipped providers route tool calls through `tools::dispatch(name,
input, ctx)` in `tools.rs`. Tools: `get_active_build`, `wiki_search`,
`wiki_parse`, `wiki_synergies`, `wiki_cargo`, `trade_resolve_stats`,
`trade_search_url`, `web_fetch`. The `web_fetch` allowlist (tier-1–7) is
enforced in `tools.rs` regardless of caller, so Bestel never fetches
Fandom / Fextralife / RMT pages even if a model tries.

- **Anthropic API (`anthropic.rs`)**: full tool loop in `run()` —
  agentic, up to 16 iterations.
- **Ollama (`ollama.rs`)**: full tool loop, same dispatch — but reduced
  schema for small (≤8B) models to fit a 8K context window.

---

## Anthropic API (`anthropic.rs`)

- Stable, versioned via the `anthropic-version: 2023-06-01` header.
- SSE stream with documented event types: `message_start`, `content_block_start`,
  `content_block_delta` (`text_delta`, `thinking_delta`, `input_json_delta`),
  `content_block_stop`, `message_delta`, `message_stop`.
- Tools are typed in the request schema; their `input` arrives as a
  parsed JSON object (e.g. `{"query":"..."}` or `{"url":"..."}`).
- Thinking blocks come as `thinking` content blocks, possibly with
  `redacted_thinking` for safety-redacted ones.
- Cache control is set on the final tool schema entry (ephemeral, 5-min
  TTL) so the system prompt + tool schemas hit the cache between turns.

## Ollama (`ollama.rs`)

- HTTP POST to `/api/chat` with `stream: true`. Server emits NDJSON.
- Each line is a `ChatResponse` with optional `message.content` (text
  delta) and optional `message.tool_calls[]` (the model decided to call
  a tool — input is already parsed JSON).
- Tool results are appended back into the message history as
  `role: "tool"` entries before the next iteration.
- No reasoning channel — if the model thinks, it emits text. We don't
  attempt to split.
- Context window capped at 8K (`num_ctx`). Tool schema is reduced for
  smaller models to leave room for the system prompt + history.

---

## TODO when adding more providers

- **OpenAI Responses API** (direct, no CLI subprocess): event types like
  `response.output_text.delta`, `response.tool_call.delta`. Closer to
  Anthropic SSE in spirit. Would route through the same `tools.rs`
  dispatch.

- **Per-provider schema versioning**: consider stamping each parser with
  the provider version it was tested against. When users report a
  regression, asking for the `BESTEL_DEV_LOG` is the fastest way to
  confirm a schema drift.

- **Live tool input streaming for Anthropic** — `input_json_delta`
  arrives chunk-by-chunk. On each chunk, do a tolerant partial-JSON
  parse and emit a new `LlmDelta` variant (e.g. `ToolDetailUpdate { id,
  detail }`) so the URL/query appears inside the tool card *while it is
  being built*. Mirrors the ChatGPT/Claude.ai UX where the search URL
  is visible at start, not only at the end. See the TODO marker in
  `anthropic.rs::run`.
