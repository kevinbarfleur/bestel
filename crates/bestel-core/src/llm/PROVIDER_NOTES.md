# Provider parsing notes

Each LLM provider streams a different event schema. Bestel normalises them
into the shared `LlmDelta` enum, but the *parser* per provider has to know
the native shape. This file collects what we've learned so future-you (or
future-us) doesn't re-discover it.

> Rule of thumb: **the only file that should know about a provider's wire
> format is `crates/bestel-core/src/llm/<provider>.rs`**. Everything else
> consumes `LlmDelta`.

---

## Anthropic API (`anthropic.rs`)

- **Stable**, versioned via the `anthropic-version: 2023-06-01` header.
- SSE stream with documented event types: `message_start`, `content_block_start`,
  `content_block_delta` (`text_delta`, `thinking_delta`, `input_json_delta`),
  `content_block_stop`, `message_delta`, `message_stop`.
- Tools are typed: `web_search_20250305`, `web_fetch_20250910`,
  `code_execution_20250522`, etc. Their `input` is a parsed JSON object
  (e.g. `{"query":"..."}` or `{"url":"..."}`).
- Thinking blocks come as `thinking` content blocks, possibly with
  `redacted_thinking` for safety-redacted ones.

## Claude Code CLI (`claude_cli.rs`)

- Wraps the Anthropic API. With `--output-format stream-json
  --include-partial-messages --verbose` we get JSONL where each line wraps
  one of the SSE events from the API.
- Outer types: `system`, `assistant`, `user`, `result`, `stream_event`,
  `error`. `stream_event.event` carries the raw API events (same shapes as
  Anthropic API).
- Tool use blocks arrive in `assistant.content[]` with `type: "tool_use"`,
  `name`, and parsed `input` JSON.
- Tool results come back in subsequent `user.content[]` blocks with
  `type: "tool_result"`, `tool_use_id`, `is_error`, `content`.

## Codex CLI (`codex_cli.rs`)

- **Schema changes between minor versions** — verified between 0.111 → 0.128.
  Always cross-check the dev log (`BESTEL_DEV_LOG=1`) when tooling looks off.
- JSONL of `thread.started`, `turn.started`, `item.started`,
  `item.updated`, `item.completed`, `turn.completed`.
- `item.type` covers: `agent_message`, `reasoning`, `web_search`,
  `web_fetch`, `command_execution`, `function_call`, `tool_call`.
  Newer versions may invent more — `codex_cli.rs` has a catch-all matcher
  on type substrings (`call`/`execution`/`search`/`fetch`/...).

### Quirks that bit us

- **`agent_message` is NOT streamed token-by-token.** Codex emits the full
  text in one `item.completed`. We simulate streaming locally
  (`pseudo_stream`).
- **`web_search` query is empty on `item.started`** — only filled on
  `item.completed`. We display `(searching…)` until then and overwrite the
  detail once we know the real query.
- **`web_search` is BATCHED.** A single `item` carries
  `action: { type:"search", query:"<primary>", queries:["q1","q2","q3"] }`.
  One visible tool card = N parallel queries. The header shows
  `<first> · +N` to make this obvious.
- **`web_search` does NOT navigate URLs.** It returns search-engine snippets,
  not page contents. The model cites URLs from snippets without per-URL
  fetches. If a model cites 5 URLs in `Sources:`, you won't necessarily see
  5 fetch tool cards — only the search batches that surfaced those URLs.
  To see per-URL navigation, the model has to invoke `web_fetch` (Codex
  rarely does this automatically on PoE-style chat prompts).
- **`reasoning` items can repeat-emit** the full text on update events, not
  just deltas. We track `last_reasoning_text` per item id and only push the
  diff to avoid double-rendering.

---

## TODO when adding more providers

- **Gemini CLI**: schema unknown until first wiring. Will need its own
  `gemini_cli.rs` mapping native events → `LlmDelta`. Expect:
  - Different reasoning representation (Gemini "thinking" tokens vs
    Anthropic's structured `thinking` blocks).
  - Different tool naming (Gemini's `googleSearch`, `urlContext`,
    `codeExecution`).
  - Possibly proto-style or raw JSON output mode flag (`--mode=json` etc.).

- **OpenAI Responses API** (direct, without Codex CLI): event types like
  `response.output_text.delta`, `response.tool_call.delta`. Closer to
  Anthropic SSE in spirit.

- **Shared extractor module** (deferred): when we hit ≥4 providers, extract
  helpers like `extract_first_str`, `first_meaningful_field`,
  `dump_item_for_detail`, `first_line_summary` into
  `crates/bestel-core/src/llm/extractors.rs`. Today they live duplicated
  inside `codex_cli.rs` (and a couple in `claude_cli.rs`); not worth the
  abstraction yet for 3 parsers.

- **Per-provider schema versioning**: consider stamping each parser with
  the provider version it was tested against (`// verified against
  codex-cli 0.128`). When users report a regression, asking for the
  `BESTEL_DEV_LOG` is the fastest way to confirm a schema drift.

- **Tool transparency upgrades** (cross-provider, deferred):
  - Surface the full `queries[]` array as expandable lines in the tool
    card body, not just `· +N` in the header.
  - Detect `web_fetch` results that include URL+title+excerpt and render
    them as inline citation cards.
  - When a provider exposes search result URLs in the response payload,
    attach them as clickable items below the search card.
