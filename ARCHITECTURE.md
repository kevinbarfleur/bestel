# Bestel — Architecture

A reference for anyone (human or agent) walking into this codebase for the
first time. Goal: understand the runtime, the data flow, and where to look
when something needs changing — without spelunking.

---

## 1. What is Bestel

Bestel is a Windows-first desktop AI companion for **Path of Exile 1 and 2**.
The persona is **Bestel, chronicler of Lioneye's Watch** — a grim PoE1 NPC
who answers in-character, but only after running concrete tool calls
against the user's loaded Path of Building file, the wikis, the trade
catalogue, and a curated knowledge layer. The audience is PoE veterans, not
new players: answers should land at creator-guide quality (Maxroll /
Mobalytics / Pohx), with named items, exact numbers, and citations.

The product runs as a Tauri 2 desktop app (one chat window plus two utility
windows) with a Rust backend, a Vue 3 + TypeScript frontend, and a vendored
LuaJIT subprocess that wraps the upstream Path of Building engine.

---

## 2. Stack at a glance

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 (custom titlebar, multi-window) |
| Frontend | Vue 3 + TypeScript + Vite + Pinia + markdown-it |
| Backend | Rust 2021 (`tokio`, `reqwest` with rustls, `quick-xml`, `notify`, `rmcp`, `chrono`, `serde`, `dirs`) |
| LLM providers | Anthropic API (HTTP SSE), DeepSeek (Anthropic-compatible endpoint), Ollama (local HTTP) |
| PoB engine | LuaJIT 2.1 ROLLING (built with MSVC) wrapping `PathOfBuildingCommunity` and `PathOfBuilding-PoE2` (vendored as git submodules) |
| Persistence | Plain JSON / Markdown files under `~/.bestel/` (no SQLite yet) |
| Typography | EB Garamond (body / display), Kalam (script), JetBrains Mono (code) |

The product is intentionally Windows-first: paths use `dirs::document_dir()`
where appropriate, the LuaJIT child is spawned with `CREATE_NO_WINDOW` to
avoid console flicker, and PoB file watching defaults to the canonical
`Documents/Path of Building` and `Documents/Path of Building (PoE2)` trees.

---

## 3. Workspace structure

The workspace is a Cargo workspace with four crates rooted at `Cargo.toml`.

### 3.1 `crates/bestel-core/` — domain library

The brain of the system, depended on by every other crate. Holds zero UI
code.

- `src/llm/` — provider abstraction, streaming, tool dispatch, recorder.
  - `mod.rs` — `Provider` enum, `ChatMessage`, `LlmDelta` (the central
    streaming enum), `UsageStats`, `Role`.
  - `anthropic.rs` — Anthropic SSE streaming client. Used both for
    Anthropic and for DeepSeek (DeepSeek exposes an Anthropic-compatible
    endpoint at `https://api.deepseek.com/anthropic`).
  - `ollama.rs` — Ollama JSON-lines streaming client.
  - `tools.rs` — `BuildContext`, `tool_schemas()` JSON, dispatch entry
    point `BuildContext::dispatch_tool` and renderer
    `render_tool_result`.
  - `recorder.rs` — `Recorder` accumulates `LlmDelta` events into a
    `PersistedRun` written to `~/.bestel/runtime/debug-chats/<id>.json`.
  - `models.rs` — `ProviderProfile` definitions (the four shipped
    profiles), `find_profile`, `list_profiles_with_local`, active-profile
    persistence.
  - `keys.rs` — API key storage at `~/.bestel/runtime/keys.json`,
    whitelist `ANTHROPIC_API_KEY` / `DEEPSEEK_API_KEY`, `apply_to_env()`.
  - `detect.rs` — `build_provider_for_profile`, Ollama probing, key
    presence checks.
- `src/pob/` — Path of Building parsing.
  - `parser.rs` — `parse_file` / `parse_bytes` → `PobBuild`. Pure XML
    deserialization, sync, ~10–30 ms per file.
  - `mod.rs` — public types (`PobBuild`, `PobItem`, `PobSkillGroup`,
    `PobSkillGem`, `PobDefenses`, `PobCharges`, `PobBuffs`, `PobConfig`,
    `PobTree`, `JewelPlacement`, …).
  - `semantic.rs` — Sprint 3 layer: `BuildIdentity { archetype,
    defining_uniques, conversion_chain }`. Pure heuristics over the
    parser output, ~1–5 ms.
  - `dict.rs` — vendored skill / gem / item lookup tables.
  - `locator.rs` — find PoB Builds folder (PoE1 + PoE2).
  - `watcher.rs` — `notify` file-system watcher fed into the Tauri app.
- `src/sources/` — external knowledge clients (see § 10).
- `src/prompts.rs` — bundled `SYSTEM_PROMPT.md`, `CORE_KNOWLEDGE.md`,
  `BUNDLED_REFERENCES` array (~65 entries) loaded at compile time via
  `include_str!`.
- `src/test_runner/` — TOML scenario loader (`scenario.rs`,
  `real_prompts.rs`, `mod.rs`), shared between the Tauri side and the
  headless `bestel run-battery` subcommand.
- `src/mcp/` — MCP server (`rmcp`) so other agents can call Bestel's
  tools over stdio. Exposed via `bestel mcp-serve`.

### 3.2 `crates/bestel/` — Tauri binary + Vue UI

The shipped product. Depends on `bestel-core`. Holds the Tauri commands,
the event pump, the custom titlebar, and the Vue frontend under `ui/`.

- `src/main.rs` — Tauri builder, invoke handler, `bestel mcp-serve` and
  `bestel run-battery` CLI subcommands.
- `src/commands.rs` — every `#[tauri::command]`. ~36 commands grouped by
  domain (chat, build, model, debug, prompts, dev-panel, link-modal).
- `src/events.rs` — `pump_deltas` consumes `mpsc::UnboundedReceiver<LlmDelta>`
  and emits `llm:delta` events to the active window. Defines the event
  name constants (`LLM_DELTA`, `POB_BUILD`, `POB_CLEARED`, `PROVIDER_STATUS`).
- `src/dto.rs` — types serialized over IPC (`PobBuildDto`,
  `ModelProfileDto`, `LlmDeltaEvent`, etc.).
- `src/state.rs` — Tauri-managed `AppState` (build context, current
  session, provider profile cache, cancellation handles).
- `src/titlebar.rs` — `window_minimize`, `window_toggle_maximize`,
  `window_close`. Window-aware so the same handlers serve every window.
- `ui/` — Vue 3 frontend (see § 12).

### 3.3 `crates/bestel-pob-engine/` — headless PoB sidecar

A LuaJIT-based subprocess wrapper that lets the agent re-run PoB
calculations with a custom Calcs config (boss profile, charges, flask
toggles).

- `src/lifecycle.rs` — spawn / kill / reuse the LuaJIT child. Spawn uses
  `creation_flags(CREATE_NO_WINDOW)` on Windows so the GUI parent does
  not pop a console for each calc.
- `src/protocol.rs` — ndJSON request / response frames (`LoadBuildXml`,
  `GetStats`, `SetConfig`, `SetMainSelection`, `Reply`).
- `src/calc.rs` — high-level `CalcRequest` builder, response decoder,
  `CalcResponse` (includes `active_skill` echo so the agent can verify
  the engine targeted the right skill).
- `vendor/api-stdio-bestel/api-stdio.lua` — forked PoB harness. Adds
  `inert_control()` stubs for UI primitives, `sanitise()` cycle-breaker
  for dkjson encode, `active_skill_meta()` returning
  `{main_socket_group, main_active_skill, skill_part, active_skill_label,
  active_skill_gem}`, and the bug-fix `set_main_selection` rewrite that
  resolved the silent wrong-skill DPS issue.
- `vendor/PathOfBuildingCommunity/` — git submodule, pinned to v2.65.0
  (PoE1 engine).
- `vendor/PathOfBuilding-PoE2/` — git submodule, pinned to v2.49.3 (PoE2
  engine).

### 3.4 `crates/bestel-test/` — TOML regression scenarios

Thin crate that re-exports the scenario types from `bestel-core::test_runner`
and provides the integration-test harness. The shipped scenarios live at
`tests/scenarios/` (see § 15).

---

## 4. Multi-window topology

Three windows declared in `crates/bestel/tauri.conf.json`:

| Label | URL | Default | Theme | Purpose |
|---|---|---|---|---|
| `main` (no explicit label) | `index.html` | visible | dark `#0a0a0c` | The chat window. Composer + transcript + side panel + topbar |
| `prompt-editor` | `prompt-editor.html` | hidden | light `#ece7dc` | CodeMirror editor over the user-disk copy of `~/.bestel/prompts/`. Lets the user tweak system prompt / core knowledge / references without rebuilding |
| `dev-panel` | `dev-panel.html` | hidden | dynamic (paper / dark per main app theme) | Four tabs: Runs, Scenarios, Real prompts, Live test. See § 15 |

Each window has its own Vite entry point and root Vue component; there is
no shared router. `crates/bestel/ui/vite.config.ts` declares the three
inputs in `rollupOptions.input`.

| Window | HTML | TS entry | Root component |
|---|---|---|---|
| `main` | `crates/bestel/ui/index.html` | `src/main.ts` | `App.vue` (router-driven) |
| `prompt-editor` | `crates/bestel/ui/prompt-editor.html` | `src/prompt-editor-main.ts` | `views/PromptEditor/PromptEditorRoot.vue` |
| `dev-panel` | `crates/bestel/ui/dev-panel.html` | `src/dev-panel-main.ts` | `views/DevPanel/DevPanelRoot.vue` |

**Cross-window theme sync.** When the user toggles the theme in Settings,
the main window emits a `theme:changed` event with `{ theme: 'light' | 'dark' }`.
The other two windows listen via `listen('theme:changed', …)` in their
TS entry and toggle the `theme-dark` class on `document.documentElement`.

**Opening utility windows.** The `prompts_open_editor` and `dev_panel_open`
IPC commands (`crates/bestel/src/commands.rs`) follow a show-or-build
pattern: try `app.get_webview_window(label)`, if found `show + focus`, else
build with the config from `tauri.conf.json`. `Ctrl+Shift+D` from the main
window invokes `dev_panel_open` directly.

---

## 5. Chat flow end to end

This is the most important section — it ties together IPC, providers, the
tool loop, and the Vue store.

```
User types → ChatComposer.vue → invoke chat_start
                                       │
                              [crates/bestel/src/commands.rs::chat_start]
                                       │
                          ┌────────────┴────────────┐
                          │                         │
                   build_provider_for_profile   pump_deltas spawned
                          │                         │
                  spawn provider.run        listen mpsc::UnboundedReceiver
                          │                         │
                  emit LlmDelta on chan       emit "llm:delta" via app.emit
                          │                         │
                          ▼                         ▼
                Anthropic / DeepSeek /         Window receives event
                 Ollama streaming                   │
                          │                         │
                  on tool_use → dispatch_tool       │
                          │                         │
                  inject ToolResult, re-call API    │
                          │                         │
                  loop until end_turn               │
                          │                         │
                  emit MessageEnd                   ▼
                                              chat store mutation
                                              Vue render
```

### 5.1 The exact path on the user-typed-a-message side

1. User submits a message in `ChatComposer.vue`. The composer calls
   `chatStart(text, attachments)` exposed by `crates/bestel/ui/src/api/tauri.ts`.
2. The IPC handler is `crates/bestel/src/commands.rs::chat_start`. It:
   - Reads the active model profile from `AppState` (mirror of
     `~/.bestel/runtime/model.json`).
   - Reads the API key for that provider via `bestel_core::llm::keys`.
   - Constructs the system prompt by concatenating the on-disk
     `SYSTEM_PROMPT.md` + `CORE_KNOWLEDGE.md` + provider-specific addendum
     + model-specific addendum (see § 8).
   - Reads the active build (if any) from `BuildContext`, embeds a small
     `[Build state: …]` line at the top of the system prompt.
   - Builds a `Provider` via `bestel_core::llm::detect::build_provider_for_profile`.
   - Creates an `mpsc::UnboundedChannel<LlmDelta>` and a
     `oneshot::Sender<()>` cancellation handle (kept in `AppState`).
   - Spawns `provider.run(history, BuildContext, delta_tx, cancel_rx)` on
     the tokio runtime.
   - Spawns `pump_deltas(delta_rx, app_handle)` on the tokio runtime.
   - Returns the new session id (a `u64`) immediately to the frontend.

### 5.2 The provider side

Inside `provider.run`, the code path lives in
`crates/bestel-core/src/llm/anthropic.rs::AnthropicProvider::run` (or
`ollama.rs::OllamaProvider::run`). The shape is the same:

1. POST to the streaming endpoint, parse SSE / NDJSON lines.
2. Translate provider-native events into `LlmDelta` (see § 14).
3. On `tool_use` events: collect args, call
   `BuildContext::dispatch_tool(name, args)` synchronously when the tool is
   pure (e.g. `get_active_build`) or asynchronously for the network ones
   (`wiki_parse`, `web_fetch`, `pob_calc`). Stream the tool output as
   `ToolBegin → ToolOutput* → ToolEnd` deltas.
4. After the tool returns, append an assistant `tool_use` block + a user
   `tool_result` block to the history and re-call the API.
5. Loop until the API returns `stop_reason: end_turn` or hits the safety
   cutoff.
6. On the final iteration: emit `Usage(UsageStats)` and `MessageEnd`,
   close the channel.

### 5.3 The pump and the frontend

`crates/bestel/src/events.rs::pump_deltas` reads the channel and emits
`llm:delta` events to the window with a serialized `LlmDeltaEvent` payload
(`crates/bestel/src/dto.rs`). The frontend store
`crates/bestel/ui/src/stores/chat.ts` listens once per session and
dispatches by `kind`:

- `text` → append to the open `Text` segment of the streaming message
- `reasoning_begin` / `reasoning_delta` / `reasoning_end` → fold into a
  collapsible `Reasoning` segment
- `tool_begin` → push a `Tool` segment with status `running`
- `tool_output` → append to the matching tool's output buffer
- `tool_end` → flip status to `done` / `failed`, persist summary
- `usage` → store input / output / cost on the message
- `error` → render the message as failed
- `message_end` / `cancelled` / `completed` → flip the message to final
  state

Markdown rendering is done by `crates/bestel/ui/src/api/markdown.ts`
(markdown-it instance with `linkify: false` so accidental URL-like fragments
in prose do not get auto-linked — the bug that surfaced with `wiki.La`).

### 5.4 Recorder

In parallel, the `Recorder` (`crates/bestel-core/src/llm/recorder.rs`)
mirrors every `LlmDelta` into a `PersistedRun` that is written atomically
to `~/.bestel/runtime/debug-chats/<id>.json` when the run completes. The
dev panel reads from that directory; the `bestel run-battery` CLI uses the
same recorder.

### 5.5 Cancellation

`chat_cancel` IPC sends on the `oneshot::Sender<()>` stored in `AppState`.
The provider checks the cancellation receiver between API chunks; on
trigger it emits `LlmDelta::Error("cancelled")` followed by `MessageEnd`
and returns.

---

## 6. Provider system

Five profiles ship today (`crates/bestel-core/src/llm/models.rs`):

| Profile id | Model id | Provider | Endpoint |
|---|---|---|---|
| `anthropic-haiku-4-5` | `claude-haiku-4-5-20251001` | Anthropic | api.anthropic.com |
| `anthropic-sonnet-4-6` | `claude-sonnet-4-6` | Anthropic | api.anthropic.com |
| `anthropic-opus-4-7` | `claude-opus-4-7` | Anthropic | api.anthropic.com |
| `deepseek-v4-flash` | `deepseek-v4-flash` | Anthropic-compat | api.deepseek.com/anthropic |
| (any local Ollama tag) | (resolved at runtime) | Ollama | http://127.0.0.1:11434 |

Profile selection is persisted at `~/.bestel/runtime/model.json` and
mutated by the IPC `set_active_model`. `list_models` returns
`list_profiles_with_local()` which probes Ollama and merges discovered
local models into the static profile list.

`detect_providers` runs a fast availability check at startup: it pings
Ollama, checks for an Anthropic key, checks for a DeepSeek key, and emits a
`PROVIDER_STATUS` event so the topbar can grey out unavailable providers.

API keys live in `~/.bestel/runtime/keys.json` with a strict whitelist
(`ANTHROPIC_API_KEY`, `DEEPSEEK_API_KEY`). `apply_to_env()` injects them
into `std::env::set_var` at startup so any provider that reads from env
picks them up.

---

## 7. Tool system

The agent has **eleven** tools registered in
`crates/bestel-core/src/llm/tools.rs::tool_schemas()`. Const names are:

| Const | Purpose | Trigger |
|---|---|---|
| `GET_ACTIVE_BUILD` | Returns the loaded PoB as JSON: structural fields + Sprint 3 semantic facts (`archetype`, `defining_uniques`, `conversion_chain`) | Always before commenting on the exile's character |
| `WIKI_SEARCH` | Free-text search on the official MediaWiki API (poewiki / poe2wiki) | When the agent does not know the canonical page title |
| `WIKI_PARSE` | Fetch a wiki page by title and extract the prose sections (Mechanics / Caps / Interactions). The primary research tool | After `wiki_search` or when the title is known |
| `WIKI_SYNERGIES` | Reverse-link sweep (`Special:WhatLinksHere`) on a topic, filtered to uniques / keystones / cluster jewels / gems | After every keystone / mechanic / unique / skill question |
| `WIKI_CARGO` | Structured Cargo (Semantic MediaWiki) query for mod tiers, item bases, version history | Niche — when the wiki structured tables are needed |
| `TRADE_RESOLVE_STATS` | Map a stat phrase → trade stat ID using the bundled trade-stats catalogue | Required before any trade search |
| `TRADE_SEARCH_URL` | Build a shareable `pathofexile.com/trade` URL from a query body | When the user wants a price check |
| `WEB_FETCH` | HTTP GET against the source allowlist (tier 1–7), explicit error on tier-8 blocked hosts | Patch notes, PoEDB pages, Maxroll / Pohx / Mobalytics articles |
| `READ_INTERNAL_REFERENCE` | Load one of the bundled reference docs from `prompts/references/` by relative path. Schema is a strict JSON `enum` of every known path so the agent cannot invent unknown filenames | Conceptual grounding (defence layering, retrieval playbooks, archetype taxonomy, etc.) |
| `REPOE_LOOKUP` | Query the bundled repoe-fork snapshot for base items, mods, gems with their tags / weights / item-level minimums | Before `wiki_parse` for cheap structural lookups |
| `POB_CALC` | Run the headless PoB engine with a Calcs config override and return canonical output stats. Echoes the Calcs config and `active_skill` metadata so the agent can verify the engine targeted the right skill | When real DPS / EHP under specific conditions is needed |

Dispatch path: provider → `BuildContext::dispatch_tool(name, args)` →
`match name { GET_ACTIVE_BUILD => …, WIKI_PARSE => …, … }` → handler
returns a string (or streamed chunks for long responses) →
`render_tool_result` adapts to the JSON envelope each provider expects.

The tool descriptions in `tool_schemas()` carry **agent-side discipline**:
required calling order ("ALWAYS try `repoe_lookup` BEFORE `wiki_parse` for
items / mods"), verification rules ("compare `active_skill.active_skill_label`
with the build's `main_skill` BEFORE quoting any `pob_calc` number"), and
hard prohibitions ("never recommend selling a `category: \"engine\"` item
without explicit user instruction"). These mirror the guidance in the
system prompt and are repeated in the schema so the agent encounters them
the moment it considers the tool.

---

## 8. Knowledge layer (`prompts/`)

The `prompts/` directory at the repo root holds the agent's load-bearing
knowledge. It is bundled into the binary at compile time via
`include_str!` and seeded onto the user disk at `~/.bestel/prompts/` on
first launch so the user can override anything without rebuilding.

### 8.1 Layout

```
prompts/
├── SYSTEM_PROMPT.md          # voice + hard rules + identity card requirement
├── CORE_KNOWLEDGE.md         # invariants + tool inventory + reference index
└── references/               # 27 numbered docs + 4 sub-folders
    ├── 00_README.md
    ├── 01_source_policy.md
    ├── 02_arpg_foundations.md
    ├── …
    ├── 26_validation_and_self_correction.md
    ├── creators_registry/    # per-creator profiles (Pohx, Quin, Mathil, …)
    ├── poe2/                 # version-pinned PoE2 mechanics
    ├── thresholds/           # red maps / pinnacle / uber numerical bars
    ├── glossary/             # FR ↔ EN proper nouns
    └── maxroll/              # URL catalogue per archetype
```

### 8.2 Bundling

`crates/bestel-core/src/prompts.rs` declares:

```rust
pub const BUNDLED_SYSTEM_PROMPT: &str = include_str!("../../../prompts/SYSTEM_PROMPT.md");
pub const BUNDLED_CORE_KNOWLEDGE: &str = include_str!("../../../prompts/CORE_KNOWLEDGE.md");
pub const BUNDLED_REFERENCES: &[(&str, &str)] = &[
    ("00_README.md", include_str!("../../../prompts/references/00_README.md")),
    ("01_source_policy.md", include_str!("../../../prompts/references/01_source_policy.md")),
    // … ~65 entries total
];
```

### 8.3 Resolution at runtime

Every chat turn re-reads `SYSTEM_PROMPT.md` and `CORE_KNOWLEDGE.md` from
disk; on any I/O error the bundled string wins, so wiping
`~/.bestel/prompts/` cannot brick the agent. References are NOT
concatenated into the system prompt — they are fetched on demand by the
`read_internal_reference` tool. The agent only ever pays the token cost of
references it actually needs.

### 8.4 Override hierarchy

| Scope | Path | Concatenated when |
|---|---|---|
| Global | `~/.bestel/prompts/SYSTEM_PROMPT.md` + `CORE_KNOWLEDGE.md` | Always |
| Per-provider | `~/.bestel/prompts/providers/<provider>.md` (e.g. `anthropic.md`, `ollama.md`) | The chosen profile matches |
| Per-model | `~/.bestel/prompts/models/<sanitized_model_id>.md` | The chosen model id matches exactly |

This lets the user inject Ollama-specific tool restrictions
(`local_addendum.md` shipped baked into the binary) or a per-model nudge
without disturbing the global prompt.

### 8.5 The `read_internal_reference` enum

The schema for this tool's `rel_path` argument is a JSON `enum` of every
key in `BUNDLED_REFERENCES`. The Anthropic API rejects any value not in
the enum before the model even sees its own attempt — eliminating an
entire class of hallucination where the agent invents filenames like
`09_build_archetypes.md`.

---

## 9. PoB integration

Three layers, called in order of cost.

### 9.1 Parser (cheap, always available)

`crates/bestel-core/src/pob/parser.rs::parse_file` deserializes the PoB
XML into `PobBuild`. The struct carries everything the file declares:
class, ascendancy, level, items (raw mod text preserved), skill groups
with linked gems, the full passive node-id list, defenses, charges, buffs,
config inputs, mastery picks, jewel placements, spectres, and
`pobb.in` import link. Sync, no I/O after the read, ~10–30 ms per file.

`PobBuild` is the canonical shape. It is wrapped in
`Arc<RwLock<Option<PobBuild>>>` inside `BuildContext`
(`crates/bestel-core/src/llm/tools.rs`) so commands and the provider
share the same view without copying.

### 9.2 Semantic layer (Sprint 3)

`crates/bestel-core/src/pob/semantic.rs::BuildIdentity::from_build(&PobBuild)`
adds three derived facts per build:

- `archetype: ArchetypeTags { defense, hit_model, mechanic }` — short tag
  vectors. Heuristics use stats, gem names, items raw text. Priority-
  ordered, first-match wins. RF / VLS only fire when the matching gem is
  the `main_skill`, never when it's a side group.
- `defining_uniques: Vec<DefiningUniqueMatch>` — match against a curated
  ~40-entry registry. Each entry is tagged `engine` (build collapses
  without it: Mageblood, Headhunter, Cospri's, Mjölner, Original Sin),
  `defining` (shapes archetype: Replica Soul Tether, Asenath's, Indigon,
  timeless jewels), or `amplifier` (replaceable boost: Watcher's Eye,
  Bottled Faith, Thread of Hope).
- `conversion_chain: Option<ConversionChain>` — manual line-by-line scan
  of item raw mods for `(\d+)% of <Type> Damage Converted to <Type>` plus
  intrinsic gem conversions (Cold to Fire support, Glacial Cascade,
  Volatile Dead). Returns `None` if no conversion is found.

`BuildIdentity::from_build` is called at the top of `render_build_for_llm`
in `tools.rs`, before serialization, so every `get_active_build` response
carries these keys. Cost ~1–5 ms.

### 9.3 Headless engine (expensive, on-demand)

`crates/bestel-pob-engine/` wraps a long-lived LuaJIT subprocess that
loads the upstream PoB Lua sources (`vendor/PathOfBuildingCommunity/` for
PoE1, `vendor/PathOfBuilding-PoE2/` for PoE2) through a forked harness
(`vendor/api-stdio-bestel/api-stdio.lua`).

- **Lifecycle.** `lifecycle.rs::PobEngineHandle::ensure` spawns the child
  on first use and reuses it. Spawn applies `creation_flags(0x0800_0000)`
  (CREATE_NO_WINDOW) on Windows so no console pops up.
- **Protocol.** `protocol.rs` defines an ndJSON request / reply pair:
  - `LoadBuildXml { xml }` — load the build text
  - `SetMainSelection { main_socket_group, main_active_skill, skill_part }`
    — pick which skill to calc against
  - `SetConfig { config }` — Calcs panel inputs (`enemyIsBoss`,
    `usePowerCharges`, `useFlask1`, `multiplierImpaleStacks`, …)
  - `GetStats { category }` — return a sub-block of the engine's stats
    table (`offence`, `defence`, `charges`, `reservation`, `ailments`,
    `all`)
- **Calcs orchestration.** `calc.rs::CalcRequest` builds the multi-step
  request the `pob_calc` tool needs (load → set selection → set config
  → get stats), with the response always echoing the effective config and
  `active_skill` metadata so the agent can sanity-check the engine
  targeted the intended skill.

The harness fork is meaningful: upstream `api-stdio.lua` assumes a desktop
PoB context, so the fork stubs `inert_control()` for the UI primitives
(`NewDropDown`, `NewEditControl`, `NewButtonControl`, …), patches
`set_main_selection` to fix a silent wrong-skill fallback, exposes
`active_skill_meta()`, and sanitizes recursive tables before dkjson encode.

---

## 10. External data sources

`crates/bestel-core/src/sources/` holds the network and bundled-snapshot
clients. All are async on top of the shared `reqwest` client in `http.rs`
(rustls, timeout, custom user-agent).

| Module | Backs which tool | Notes |
|---|---|---|
| `wiki.rs` | `wiki_search`, `wiki_parse`, `wiki_synergies`, `wiki_cargo` | MediaWiki API on poewiki.net (PoE1) and poe2wiki.net (PoE2). Parses the wikitext / HTML and strips infoboxes |
| `trade.rs` | `trade_resolve_stats`, `trade_search_url` | Hits `pathofexile.com/api/trade/data/stats` and builds search URLs |
| `trade_catalogue.rs` | `trade_resolve_stats` | Bundled snapshot of the stat catalogue (PoE1 + PoE2) so the agent has a fast offline path |
| `repoe.rs` | `repoe_lookup` | Bundled snapshot of repoe-fork (base items, mods with tiers, gems with tags) |
| `http.rs` | All HTTP tools | Shared client. Enforces user-agent, blocks tier-8 hosts before sending |
| `cache.rs` | All HTTP tools | Disk cache under `~/.bestel/cache/<source>/` keyed by URL hash |

### 10.1 Source allowlist

`tools.rs` declares a `FETCH_BLOCKLIST` (fandom, fextralife, RMT sites,
generic SEO blogs) that blocks `web_fetch` outright before any HTTP call.
The trusted whitelist is encoded in `SYSTEM_PROMPT.md` as the seven-tier
table the agent must respect:

| Tier | Domains | Use for |
|---|---|---|
| 1 — Canonical | pathofexile.com, pathofexile2.com | Patch notes, official rules, trade |
| 2 — Wikis | poewiki.net, poe2wiki.net | Mechanics, items, skills, league mechanics |
| 3 — Datamined | poedb.tw, poe2db.tw, repoe-fork.github.io | Mod data, tags, spawn weights |
| 4 — Calculators | pathofbuilding.community, craftofexile.com | Build math, crafting odds |
| 5 — Economy | poe.ninja | Price trends, popular builds |
| 6 — Filters | filterblade.xyz | Loot filters |
| 7 — Trusted creators | maxroll.gg, mobalytics.gg, pohx.net, poe-vault.com, poeplanner.com, pathofpathing.com, poelab.com | Secondary, only when patch number / author / date is shown |

---

## 11. Persistence layout

Everything Bestel writes lives under `~/.bestel/`. There is no SQLite, no
shared state mutation server — just JSON / Markdown files.

| Path | Producer | Contents |
|---|---|---|
| `runtime/keys.json` | `keys.rs` | API keys (whitelist `ANTHROPIC_API_KEY`, `DEEPSEEK_API_KEY`) |
| `runtime/model.json` | `models.rs` | Active model id + provider |
| `runtime/debug-chats/<id>.json` | `recorder.rs` | One `PersistedRun` per chat or battery scenario |
| `prompts/SYSTEM_PROMPT.md` + `CORE_KNOWLEDGE.md` | seeded from bundle | User-editable copies |
| `prompts/references/**` | seeded from bundle | User-editable reference library |
| `prompts/providers/<provider>.md` | user / prompt-editor | Per-provider system-prompt addendum |
| `prompts/models/<sanitized_id>.md` | user / prompt-editor | Per-model addendum |
| `cache/repoe/`, `cache/trade_stats/`, `cache/wiki/` | `cache.rs` | HTTP response snapshots keyed by URL hash |
| `exports/runs-export-<stamp>.json` | `dev_export_all_runs` IPC | Full corpus dump for external review |

`debug_recorder::debug_runs_dir()` (in `recorder.rs`) is the canonical
reader for the debug-chats directory.

---

## 12. Frontend state

### 12.1 Pinia stores

Every store under `crates/bestel/ui/src/stores/`.

| Store | Scope |
|---|---|
| `chat.ts` | Streaming message segments, current session id, send / cancel actions, history of the current conversation |
| `build.ts` | Active build (mirror of backend `BuildContext`), parsed metadata for the UI (chip values, defenses) |
| `chatHistory.ts` | List + paginate saved chats (legacy persistence layer, distinct from `debug-chats/`) |
| `prompts.ts` | Prompt-editor state — file tree, open file content, dirty tracking, save / reset actions |
| `settings.ts` | Theme, density, palette, active model, API keys (proxied), feature flags |
| `toasts.ts` | Notification queue (info / error variants) |
| `ui.ts` | Side-panel open state, picker visibility, transient UI flags |

Pinia stores live in the main window only. The prompt-editor and dev-panel
windows each create their own fresh Pinia instance in their `*-main.ts`
entry — store state is **not** shared across windows; everything that must
be shared goes through Tauri events.

### 12.2 Component domains

Under `crates/bestel/ui/src/components/`:

| Folder | Signature components | Role |
|---|---|---|
| `chat/` | `ChatComposer.vue`, `ChatMessage.vue`, `ChatStream.vue` | Composer with attachments, message renderer with tool cards, transcript |
| `build/` | `BuildPanel.vue`, `BuildPicker.vue`, `BuildItemCard.vue`, `ModelPicker.vue` | PoB load / display / item hover |
| `settings/` | `SettingsPicker.vue`, `SettingsField.vue`, `SettingsModelRow.vue`, `SettingsDeveloperPane.vue` | Settings rail with eight panes |
| `pickers/` | Legacy modal pickers (chat, build, model) | Pre-v2 surface |
| `pickers-v2/` | `PickerModal.vue`, `PickerListItem.vue`, `PickerSearchInput.vue`, `PickerStatusDot.vue`, `PickerLeader.vue`, … | Phase 1 design primitives shipped — used by the dev panel re-skin |
| `runic/` | `GemTooltip.vue`, `ItemTooltip.vue`, `NodeTooltip.vue`, `RunicModal.vue`, `RunicSelect.vue` | Hover / popover tooltips for entity backticks |
| `prompt-editor/` | `PromptEditorRoot.vue` view + CodeMirror 6 wrapper | Editor window content |
| `link-viewer/` | Sub-webview modal for clicked external links | Stays inside the app, no OS browser pop |
| `titlebar/` | Custom Tauri-aware titlebar component | Used by all three windows |
| `panel/` | Side panel host + panel artifacts (item-card, gem-detail, mechanic, markdown) | Driven by `⟦panel:type:name⟧` markers in agent prose |
| `topbar/` | Model trigger, theme toggle, prompt-editor button | Row above the chat |

### 12.3 Style tokens

`crates/bestel/ui/src/styles/tokens.css` defines ~98 CSS custom properties
with semantic naming (`--paper`, `--paper-shade`, `--paper-line`, `--ink`,
`--ink-soft`, `--ink-faint`, `--ink-ghost`, `--amber`, `--good`, `--bad`,
`--note`, `--el-fire`, `--el-cold`, `--el-lit`, `--el-chaos`, `--el-phys`,
…). Light + dark themes are sister token sets activated via `theme-dark`
class on `<html>`. Density is configurable via `data-density='compact' |
'standard' | 'airy'` on the same root.

Fonts loaded in `crates/bestel/ui/src/styles/fonts.css`: EB Garamond
(weights 400 / 500 / 600 / 700), Kalam (script), Caveat (annotation),
JetBrains Mono.

---

## 13. IPC surface

All commands live in `crates/bestel/src/commands.rs` (and `titlebar.rs`).
Registered in `main.rs::invoke_handler`.

### 13.1 Commands by domain

| Domain | Commands |
|---|---|
| Chat | `chat_start`, `chat_cancel`, `chat_reset` |
| Build | `list_builds`, `get_active_build`, `set_active_build`, `clear_active_build`, `preview_build` |
| Models | `list_models`, `get_active_model`, `set_active_model`, `detect_providers` |
| Debug runs | `list_debug_runs`, `get_debug_run`, `delete_debug_run`, `delete_all_debug_runs`, `dev_export_all_runs` |
| Prompts | `prompts_list`, `prompts_read`, `prompts_write`, `prompts_reset`, `prompts_reset_all`, `prompts_open_editor`, `prompts_reveal_in_finder` |
| Dev panel | `dev_panel_open`, `dev_load_scenarios`, `dev_load_real_prompts` |
| API keys | `list_api_keys`, `set_api_key`, `delete_api_key` |
| Link modal | `open_link_modal`, `update_link_modal_bounds`, `close_link_modal` |
| External | `open_external` |
| Titlebar | `window_minimize`, `window_toggle_maximize`, `window_close` |

### 13.2 Events emitted by backend

| Event | Payload | Producer |
|---|---|---|
| `llm:delta` | `LlmDeltaEvent` | `pump_deltas` per active session |
| `pob:build` | `BuildEvent` (parsed `PobBuildDto`) | `set_active_build` and the file watcher |
| `pob:cleared` | empty | `clear_active_build`, file deletion |
| `provider:status` | `ProviderStatusEvent` | startup `detect_providers`, key changes |
| `theme:changed` | `{ theme: 'light' \| 'dark' }` | `settings.ts` on toggle |

Names are constants in `crates/bestel/src/events.rs`.

---

## 14. Streaming pipeline detail

The whole streaming subsystem rotates around one enum:
`crates/bestel-core/src/llm/mod.rs::LlmDelta`.

```rust
pub enum LlmDelta {
    TextDelta(String),
    ReasoningBegin,
    ReasoningDelta(String),
    ReasoningEnd,
    ToolBegin   { id: String, name: String, detail: Option<String> },
    ToolOutput  { id: String, chunk: String },
    ToolEnd     { id: String, status: ToolStatus, summary: Option<String> },
    Usage(UsageStats),
    MessageEnd,
    Error(String),
}
```

Every provider — Anthropic SSE, Ollama JSON-lines, anything we add later
— translates its native event format into this enum. The frontend never
sees provider-specific shapes; it sees `LlmDelta` only. This lets us swap
providers, add Codex CLI, or experiment with a local model without
touching the Vue store.

`UsageStats` (same module) carries `input_tokens`, `cached_input_tokens`,
`cache_creation_tokens`, `output_tokens`, and `cost_usd`. Cache fields are
populated only by providers that report them (Anthropic + DeepSeek);
Ollama leaves them at zero.

`pump_deltas` (`crates/bestel/src/events.rs`) is the only bridge between
the Rust async world and Vue. It owns no state — it just reads the
channel, serializes through `dto::LlmDeltaEvent`, and `app.emit`s.

---

## 15. Test infrastructure

Bestel has no traditional unit-test-only stance for agent behaviour. The
real verification surface is a **scenario battery** of actual chat runs
captured to JSON.

### 15.1 Scenario format

TOML files under `tests/scenarios/`. Two layouts share the same loader:

- Top-level `tests/scenarios/*.toml` — 12 broad scenarios (mechanics,
  refusal discipline, source discipline, French proper nouns, trade
  queries, build identity, etc.).
- `tests/scenarios/battery-2026-05-08/*.toml` — 10 scenarios pinned to
  real PoB builds on the user's disk: 7 single-turn (mod lookup,
  Mageblood, Penance Brand DPS, Archmage Hierophant survival, Molten
  Strike Zenith, PoE2 Shapeshift, PoB perso 3.26 diagnose) + 3 multi-turn
  (`08_followup_correction_dps`, `09_followup_drill_down`,
  `10_followup_lookup_then_apply`) using `[[turn]]` blocks.

The schema lives in `crates/bestel-core/src/test_runner/scenario.rs`. It
supports both the single-prompt shape (`prompt = "…"`) and the multi-turn
shape (`[[turn]] user = "…"`). Per-scenario expectations
(`must_match`, `must_not_match`, `must_cite_domain`, `must_call_tool`,
`forbid_tool`, `min_final_text_len`, `min_tool_calls`) gate assertion
results.

### 15.2 Battery CLI

```
bestel run-battery <scenarios-dir>
       [--model <profile-id>]
       [--filter-name <scenario-name>]
       [--out <dir>]
       [--pob-fixture <abs-path>]
```

Implementation: `crates/bestel/src/main.rs::run_battery_inner`. The CLI
boots the same provider stack as the Tauri app, drives each scenario's
turns through `provider.run`, and writes the same `PersistedRun` JSON
files to `~/.bestel/runtime/debug-chats/`. Multi-turn scenarios reuse the
same provider across turns and accumulate history between calls.

### 15.3 Dev panel

`Ctrl+Shift+D` opens the `dev-panel` window. Four tabs:

| Tab | Purpose |
|---|---|
| Runs | Lists persisted runs from `~/.bestel/runtime/debug-chats/`. Filterable by model. Click a row to view the full transcript with tool segments. Footer has `clear all` + `export all` |
| Scenarios | Loads `tests/scenarios/*.toml` via the `dev_load_scenarios` IPC. Renders one card per scenario with provider / fixture / expectations. Run button planned, currently logs |
| Real prompts | Loads `docs/test_prompts/real_user_prompts.toml` via `dev_load_real_prompts`. 31 prompts mined from real Reddit / forum / Maxroll comments, voice-preserved, filterable by category |
| Live test | Free chat with model picker, PoB fixture picker, prompt textarea. Right pane has Rendered / Raw events / Stats sub-tabs |

### 15.4 Export

The Runs tab's `export all` button invokes `dev_export_all_runs` which
writes a single envelope to `~/.bestel/exports/runs-export-<stamp>.json`.
The envelope reshapes each `PersistedRun` into a review-friendly entry
(`question`, `answer`, `reasoning`, `tools[]` with output_excerpt capped
at 4 KB, `stats`, `error`). Designed to be paste-able into a separate
expert-agent conversation without bringing the full repo.

---

## 16. Build / dev workflow

| Command | Effect |
|---|---|
| `cargo build --release -p bestel -j 1` | Release binary at `target/release/bestel.exe`. The `-j 1` is intentional — Tauri's macro pipeline trips on parallel build of the auto-generated TS bindings |
| `cd crates/bestel && cargo tauri dev` | Vite dev server + Rust hot reload. Use this for UI iteration |
| `npm --prefix crates/bestel/ui run build` | Standalone UI build into `crates/bestel/ui/dist/`. Required before the release build because the binary embeds `dist/` via the `custom-protocol` feature, and `cargo build --release` does NOT trigger Vite |
| `bestel run-battery <dir>` | Headless test runner |
| `bestel mcp-serve` | Expose Bestel's tools over stdio MCP for other agents |

Always kill the running `bestel.exe` before re-running `cargo build` —
Windows file locks otherwise prevent the `.exe` from being overwritten.

---

## 17. Where to look first

| Question | Read this |
|---|---|
| How does the chat actually stream from API to Vue? | § 5, then `crates/bestel/src/commands.rs::chat_start` and `crates/bestel/src/events.rs::pump_deltas` |
| What tools does the agent have? | § 7, then `crates/bestel-core/src/llm/tools.rs::tool_schemas` |
| Where do system prompts come from? | § 8, then `crates/bestel-core/src/prompts.rs` |
| How is the PoB engine called? | § 9.3, then `crates/bestel-pob-engine/src/calc.rs` |
| What does `get_active_build` return now (Sprint 3)? | § 9.2, then `crates/bestel-core/src/pob/semantic.rs` |
| Where is the chat state? | `crates/bestel/ui/src/stores/chat.ts` |
| How does theme sync across windows? | § 4, then `crates/bestel/ui/src/dev-panel-main.ts` and `prompt-editor-main.ts` |
| How do I add a new IPC command? | `crates/bestel/src/commands.rs` (`#[tauri::command]`) + register in `main.rs::invoke_handler` + add a wrapper in `crates/bestel/ui/src/api/tauri.ts` |
| Where do persisted chats live on disk? | `~/.bestel/runtime/debug-chats/<id>.json` |
| How do I run a regression battery? | `bestel run-battery tests/scenarios --model anthropic-haiku-4-5` |
| How does the agent decide which wiki to use? | `prompts/SYSTEM_PROMPT.md` § Trusted source whitelist |
| Where is the design token palette defined? | `crates/bestel/ui/src/styles/tokens.css` |
| Where is Bestel's voice defined? | `prompts/SYSTEM_PROMPT.md` § Voice |
| How do I add a new bundled reference doc? | Drop the file under `prompts/references/`, then add it to `BUNDLED_REFERENCES` in `crates/bestel-core/src/prompts.rs` (also makes it available in the `read_internal_reference` enum) |

---

## 18. Deferred and known gaps

Honest status, so an external reviewer is not surprised when grepping for
features that "should" exist.

- **Sprint 4 (Wiki SQLite mirror).** Paused. `wiki_parse` / `wiki_search`
  hit MediaWiki live; an offline-first SQLite mirror under
  `~/.bestel/cache/wiki.sqlite` is in the roadmap. No code yet.
- **Sprint 5 (PoE2 0.5 absorption, target 2026-05-29).** Patch-day work:
  re-pull trade stats, force-refresh repoe-fork, fill the PoE2 reference
  scaffolds (`05_atlas_mechanics_05.md`, `06_runes_of_aldur.md`), bump
  PoB2 submodule. Not started.
- **Counterfactual engine-vs-filler classifier** (Sprint 3 stretch). The
  PoB engine has no native "remove item" API; an XML round-trip would be
  required (parse → drop slot → reload → re-calc → diff DPS / EHP).
  Deferred — the compile-time tags in `defining_uniques` deliver most of
  the value at zero runtime cost.
- **Tree-dict keystone detection** (Sprint 3.5). Currently the semantic
  layer detects keystones via items-raw text or stat indicators. A proper
  node-id → keystone-name dict (cross-referenced with the passive tree)
  would make the detection robust on builds that allocate keystones from
  the tree directly.
- **Identity card verbatim quoting** (Sprint 3 follow-up). The data is
  injected into every `get_active_build` response and the SYSTEM_PROMPT
  carries the template, but Haiku 4.5 and Sonnet 4.5 still prefer to lead
  with class+ascendancy+level rather than the abstract tags. Stronger
  enforcement options: pre-render an `identity_summary` string, or
  introduce a dedicated `build_summary` tool the agent must call last.
- **Qwen3:8b tool loop.** Diagnosed — the model loops on certain prompts.
  Not yet fixed. Local Ollama models stay on a curated allowlist
  (`local_addendum.md`).
- **Rendered HTML capture per persisted run.** Would let external
  reviewers reproduce UI bugs (auto-link, panel sidecar parsing).
  Deferred.
- **Replay an existing PersistedRun.** Re-run a past prompt on a
  different model. UI in dev panel exists for design but no IPC yet.
- **Live markdown preview pane in prompt-editor.** Deferred. CodeMirror
  side only.
- **Public stash tab consumer / OAuth integration.** Out of scope for
  now.

---

## End

If anything in this doc feels stale by the time you read it, the source of
truth is always the code. Start at `crates/bestel/src/main.rs` and follow
the imports.
