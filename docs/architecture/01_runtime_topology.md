# 01 — Runtime topology

The Tauri shell, the three windows, the IPC surface, the events, the
frontend tree, and the CLI subcommands. This document is the *map* of the
runtime; the *behaviour* of each subsystem lives in `02`–`07`.

---

## Workspace

```
Cargo.toml                              # workspace root
crates/bestel-core/                     # library — no UI
crates/bestel/                          # Tauri binary + Vue UI under ui/
crates/bestel-pob-engine/               # LuaJIT sidecar (PoB engine)
crates/bestel-test/                     # TOML regression harness
```

Plus an internal `bestel-rag` crate consumed by `bestel-core` for the
LanceDB hybrid KB (see [03 § LanceDB KB](./03_knowledge_layer_and_rag.md#lancedb-kb)).

---

## Stack

| Layer | Technology |
|---|---|
| Desktop shell | Tauri 2 (custom titlebar, multi-window) |
| Frontend | Vue 3.5 + TypeScript + Vite + Pinia 2 + Vue Router 4 + markdown-it 14 + highlight.js 11 + DOMPurify 3 + CodeMirror 6 |
| Backend | Rust 2021 — `tokio`, `reqwest` (rustls + gzip + brotli + stream), `quick-xml`, `notify`, `rmcp`, `chrono`, `serde`, `dirs`, `rusqlite` (bundled), `lancedb` 0.27 with `arrow-array` / `arrow-schema` 57.2, `blake3`, `tiktoken-rs` |
| LLM providers | Anthropic API (SSE), DeepSeek (Anthropic-compatible endpoint), Ollama (NDJSON) |
| PoB engine | LuaJIT 2.1 ROLLING wrapping `PathOfBuildingCommunity` (PoE1) and `PathOfBuilding-PoE2` (PoE2), both vendored as git submodules |
| Persistence | JSON + Markdown + SQLite + LanceDB under `~/.bestel/` |
| Typography | EB Garamond (body / display), Kalam (script), Caveat (annotation), JetBrains Mono (code) |

The product is intentionally Windows-first: `dirs::document_dir()` for
PoB folder discovery, `CREATE_NO_WINDOW` flag on the LuaJIT child so no
console pops up, and tray / installer targets `nsis` + `msi`.

---

## Multi-window topology

Three windows declared in `crates/bestel/tauri.conf.json`:

| Label | URL | Visible at start | Background | Purpose |
|---|---|---|---|---|
| *(main, no label)* | `index.html` | yes | `#0a0a0c` | Chat composer + transcript + side panel + topbar |
| `prompt-editor` | `prompt-editor.html` | hidden | `#ece7dc` | CodeMirror over `~/.bestel/prompts/` |
| `dev-panel` | `dev-panel.html` | hidden | `#0a0a0c` | Runs, Scenarios, Real prompts, Live test |

Each window has its own Vite entry; there is no shared router.
`crates/bestel/ui/vite.config.ts` declares the three inputs in
`rollupOptions.input`:

```ts
input: {
  index: resolve(__dirname, 'index.html'),
  'prompt-editor': resolve(__dirname, 'prompt-editor.html'),
  'dev-panel': resolve(__dirname, 'dev-panel.html'),
}
```

| Window | HTML | TS entry | Root component |
|---|---|---|---|
| main | `ui/index.html` | `ui/src/main.ts` | `ui/src/App.vue` (router-driven) |
| prompt-editor | `ui/prompt-editor.html` | `ui/src/prompt-editor-main.ts` | `ui/src/components/prompt-editor/PromptEditorRoot.vue` |
| dev-panel | `ui/dev-panel.html` | `ui/src/dev-panel-main.ts` | `ui/src/views/DevPanel/DevPanelRoot.vue` |

**Pinia stores are not shared across windows.** Each `*-main.ts` creates
its own fresh Pinia instance. Anything that must be reflected in
multiple windows goes through a Tauri event.

**Opening utility windows.** The `prompts_open_editor` and
`dev_panel_open` IPC commands follow a show-or-build pattern:
`app.get_webview_window(label)` → if found `show + focus`, else build
from the config. `Ctrl+Shift+D` triggers `dev_panel_open` directly.

**Cross-window theme sync.** The main window owns the theme; on toggle
it emits `theme:changed` carrying `{ theme: 'light' | 'dark' }`. The
prompt-editor and dev-panel TS entries `listen('theme:changed', …)` and
flip the `theme-dark` class on `document.documentElement`.

---

## CLI subcommands

`crates/bestel/src/main.rs` reads `args[1]` and routes to one of five
functional subcommands, plus `--version` and `--help`:

| Subcommand | Purpose |
|---|---|
| *(no arg)* | Launch the Tauri app |
| `bestel mcp-serve` | Expose Bestel's tools over JSON-RPC stdio (`rmcp` server). Other agents can wire it as an MCP server |
| `bestel run-battery <dir> [--model] [--filter-name] [--out] [--pob-fixture] [--strict]` | Headless scenario runner. Drives every TOML scenario through `provider.run`; writes `PersistedRun` JSONs to `~/.bestel/runtime/debug-chats/` |
| `bestel eval-judge --runs-dir --eval-set [--judge-model] [--out] [--filter-id]` | Grade persisted runs against an eval set using a judge model |
| `bestel kb-eval --queries [--top-k] [--filter-id] [--out]` | Knowledge base retrieval eval |
| `bestel db [migrate \| stats \| import \| query \| bench]` | Inspect / manage the SQLite store (`~/.bestel/runtime/bestel.db`) |

API keys from `~/.bestel/runtime/keys.json` are injected into the
process env **before** any subcommand runs, so `mcp-serve` and the
battery runners see them transparently.

---

## IPC surface

Every Tauri command lives in `crates/bestel/src/commands.rs` (46
commands) or `crates/bestel/src/titlebar.rs` (3 commands). All 49 are
registered in `main.rs::invoke_handler`. Frontend wrappers expose them
through `crates/bestel/ui/src/api/tauri.ts`.

### Commands by domain

| Domain | Commands |
|---|---|
| Diagnostic | `ping` |
| Chat | `chat_start`, `chat_cancel`, `chat_reset` |
| Build (PoB) | `list_builds`, `get_active_build`, `set_active_build`, `clear_active_build`, `preview_build` |
| Models | `list_models`, `get_active_model`, `set_active_model`, `detect_providers` |
| Debug runs | `list_debug_runs`, `get_debug_run`, `delete_debug_run`, `delete_all_debug_runs`, `lint_debug_run`, `dev_export_all_runs` |
| Prompts | `prompts_list`, `prompts_read`, `prompts_write`, `prompts_reset`, `prompts_reset_all`, `prompts_open_editor`, `prompts_reveal_in_finder` |
| Dev panel | `dev_panel_open`, `dev_load_scenarios`, `dev_load_real_prompts` |
| API keys | `list_api_keys`, `set_api_key`, `delete_api_key` |
| Settings | `settings_get`, `settings_set_verify_enabled`, `debug_is_enabled` |
| Build sheets | `list_build_sheets`, `get_build_sheet`, `delete_build_sheet`, `delete_active_build_sheet`, `get_active_build_sheet_for_ui` |
| Link modal | `open_link_modal`, `update_link_modal_bounds`, `close_link_modal` |
| Chat logs | `chat_log_persist`, `chat_log_dir` |
| External | `open_external` |
| Titlebar | `window_minimize`, `window_toggle_maximize`, `window_close` |

### Events emitted by the backend

All event-name constants in `crates/bestel/src/events.rs`. Payloads in
`crates/bestel/src/dto.rs`.

| Event constant | String | Payload | Producer |
|---|---|---|---|
| `LLM_DELTA` | `llm:delta` | `DeltaEvent` (variant enum — see [02 § DeltaEvent](./02_agentic_system.md#deltaevent-on-the-wire)) | `events::pump_deltas` for each active session |
| `POB_BUILD` | `pob:build` | `BuildEvent { summary, build }` | `set_active_build` + the file watcher |
| `POB_CLEARED` | `pob:cleared` | empty | `clear_active_build` + file deletion |
| `PROVIDER_STATUS` | `provider:status` | `ProviderStatusEvent` | startup `detect_providers`, key changes |
| *(window-side)* `theme:changed` | `theme:changed` | `{ theme: 'light' \| 'dark' }` | `settings.ts` on toggle, frontend-only |

### Pump

`crates/bestel/src/events.rs::pump_deltas` is the one bridge between the
Rust async world and Vue. It owns no state — `tokio::select` reads the
delta receiver and the cancel oneshot, feeds the `Recorder`, and
`app.emit`s `LlmDelta::Cancelled` if cancelled. Recording is
side-by-side, not after the fact: every delta that makes it to the UI
is also applied to the `PersistedRun`.

---

## AppState

`crates/bestel/src/state.rs::AppState` is the single Tauri-managed
struct. Composition:

```rust
pub struct AppState {
    pub build_ctx: BuildContext,                     // Arc<RwLock<Option<PobBuild>>>
    pub watcher: OnceLock<Arc<PobWatcher>>,          // PoB folder watcher
    pub inner: Mutex<Inner>,                         // chat state
    prompts_self_writes: SelfWriteTracker,           // dedup self-triggered watcher events
}

pub struct Inner {
    pub history: Vec<ChatMessage>,                   // current session history
    pub next_session_id: u64,                        // monotonic, wrapping_add
    pub active_session: Option<u64>,
    pub cancel_tx: Option<oneshot::Sender<()>>,      // cancellation handle
    pub active_model_id: String,                     // mirror of model.json
}
```

**Cancellation** is a `tokio::sync::oneshot::Sender<()>` stored in
`AppState.inner.cancel_tx`. `chat_cancel` consumes the sender and
fires; the pump's `tokio::select` `biased` branch wins immediately,
emits `DeltaEvent::Cancelled`, and exits. The provider task also
inspects its own cancel receiver between API chunks (see
[02 § Cancellation](./02_agentic_system.md#cancellation)).

**Build context** is `Arc<RwLock<Option<PobBuild>>>` so both Tauri
commands and the streaming provider share one view. Replace-on-write.

**Watcher self-writes.** When the prompt-editor saves a file, the
`notify` watcher would echo the event back and cause a reload loop.
`prompts_self_writes: SelfWriteTracker` swallows the immediate echo for
each path the editor just wrote.

---

## Frontend tree

### Entry points

```
ui/src/
├── main.ts                  # main window — Pinia + router + theme apply + App.vue
├── prompt-editor-main.ts    # prompt-editor window — Pinia + theme listen + PromptEditorRoot
├── dev-panel-main.ts        # dev-panel window — Pinia + theme listen + DevPanelRoot
├── App.vue                  # main router shell
├── router.ts                # hash-based, 2 routes (/, /components) + catch-all
└── api/
    ├── tauri.ts             # IPC wrappers for every command
    └── markdown.ts          # markdown-it pipeline (see below)
```

### Pinia stores

Eight stores under `ui/src/stores/`:

| Store | Scope |
|---|---|
| `chat.ts` | Streaming message segments, current session id, send / cancel, transcript |
| `build.ts` | Active PoB build, sheet rehydration, list, loading flag |
| `sheet.ts` | Build Sheet FSM (`none → invite → interview → edit → finalized → stale`), `draftSections`, `pendingAsk`, `activeInterview` — see [06](./06_build_sheet_workflow.md) |
| `chatHistory.ts` | Saved chats (legacy, separate from `debug-chats/`) |
| `prompts.ts` | Prompt-editor file tree, dirty tracking, save / reset |
| `settings.ts` | Theme, density, palette, active model, API keys (proxied), feature flags incl. verifier toggle |
| `toasts.ts` | Toast queue (info / error) |
| `ui.ts` | Side-panel open state, picker visibility, transient flags |

### Component folders

Thirteen folders under `ui/src/components/`, ~95 `.vue` files. Per-folder role:

| Folder | Role |
|---|---|
| `chat/` | Composer, message renderer, transcript, tool-call badge, verify-claims card, artifacts (`ArtShell`, `ArtHead`, `ArtThinking`, `ArtSources`, `ArtWebSearch`, `ArtTradeScan`, `ArtWikiPage`, `ArtDiff`, `ArtPoBImport`, `ArtInterviewSubmission`, `AttachmentChip`) |
| `build/` | `BuildPanel`, `BuildPicker`, `BuildItemCard`, `BuildSection`, `BuildSkillGroup`, `BuildVitalsTable`, `BuildResistancesGrid`, `SlotIcon`, `ModelPicker`, `ProviderStatus` |
| `build-sheet/` | The 10 BS\* components for the Build Sheet feature — see [06](./06_build_sheet_workflow.md) |
| `settings/` | Settings rail + 8 panes (rail item, field, segmented, toggle, theme card, radio card, model row, prompts pane, developer pane, picker) |
| `pickers/` | Legacy picker primitives (search input, list item, status dot, section head, group label, layout, api key field) |
| `pickers-v2/` | Phase-1 design-system picker primitives (modal, list item, button, search input, leader, kbd, icon, status dot, section head, group label) |
| `runic/` | Hover / popover tooltips for entity backticks (`GemTooltip`, `ItemTooltip`, `NodeTooltip`) + runic primitives (modal, select, toggle, slider, radio, checkbox, input, number, tabs, header, divider, box, progress bar, tooltip, toast, icon, button) |
| `prompt-editor/` | CodeMirror editor + title bar + toolbar + file tree + outline panel + status bar |
| `link-viewer/` | Sub-webview modal for clicked external links |
| `titlebar/` | Custom Tauri-aware titlebar shared by all three windows |
| `panel/` | Side-panel host + artifact renderers (item-card, gem-detail, mechanic, markdown, empty, view) driven by `⟦panel:type:name⟧` markers |
| `topbar/` | Top action row (model trigger, theme, prompt-editor button) |
| *(views)* | `views/ChatView.vue`, `views/ComponentsLab.vue`, plus `views/DevPanel/DevPanelRoot.vue` |

### Markdown pipeline

`ui/src/api/markdown.ts` configures a single markdown-it instance with:

- `html: false`, `linkify: false`, `breaks: false`
- highlight.js for code blocks
- DOMPurify for sanitization
- Custom panel parser that replaces `⟦panel:type:name⟧` markers with
  `<button>` placeholders that resolve into `panel/` artifact components
- Optional sidecar JSON `⟦panel-data⟧{...}⟦/panel-data⟧` to inject
  structured artifact data
- Inline backticks tagged as PoE entities (gem, item, node, mechanic)
  → fire/cold/lit/chaos/phys/good/bad/note chips or wiki links

`linkify: false` is deliberate — auto-linking text like `wiki.La`
inside prose used to break tooltips and panel markers.

### Style tokens

`ui/src/styles/tokens.css` defines **148 CSS custom properties** with
semantic naming:

- Surfaces: `--paper`, `--paper-shade`, `--paper-line`, `--paper-deep`
- Ink: `--ink`, `--ink-soft`, `--ink-faint`, `--ink-ghost`
- Accents: `--amber`, `--good`, `--bad`, `--note`
- Elemental: `--el-fire`, `--el-cold`, `--el-lit`, `--el-chaos`,
  `--el-phys` (+ `-deep` + `-bg` variants per element)
- Type scale: `--fs-display` → `--fs-micro` (7 levels)
- Density: `--pad-card`, `--gap-card`, `--row-h` (driven by
  `data-density='compact' | 'standard' | 'airy'` on `<html>`)
- Transitions: `--transition-fast` / `-base` / `-smooth` / `-bounce`
- Z-index: `--z-modal`, `--z-dropdown`, `--z-tooltip`, `--z-toast`

Light + dark are sister token sets activated by the `theme-dark` class
on `<html>`. Persisted via `localStorage['bestel.theme']` on the main
window and event-synced to the other two.

### Fonts

`ui/src/styles/fonts.css` loads EB Garamond (400 / 500 / 600 / 700),
Kalam, Caveat, and JetBrains Mono. Self-hosted assets under
`ui/public/fonts/`.

---

## Build / dev workflow

| Command | Effect |
|---|---|
| `cd crates/bestel && cargo tauri dev` | Vite dev server + Rust hot reload. Default for iteration |
| `cargo tauri build` | Full npm build + Rust release + NSIS / MSI installers |
| `(cd crates/bestel/ui && npm run build) && cargo build --release -p bestel -j 1` | Same effect, no installer. Faster iteration |
| `bestel run-battery <dir>` | Headless scenario runner |
| `bestel mcp-serve` | Expose tools over MCP |

Pitfalls:

- **Never** `cargo build --release` alone — it embeds whatever is
  currently in `ui/dist/`. Run the Vue build first.
- `-j 1` on the Rust release is intentional. Tauri's macro pipeline
  trips on parallel builds of the auto-generated TS bindings.
- `vue-tsc` only runs on `npm run build`, not on `npm run dev`.
  Missing-variant / null-narrowing errors pass dev and explode at
  release time.
- Always kill the running `bestel.exe` before re-running `cargo build`
  — Windows file locks prevent overwriting the `.exe`.

---

## See also

- [02 — Agentic system](./02_agentic_system.md) for what happens *inside*
  `chat_start`.
- [06 — Build Sheet workflow](./06_build_sheet_workflow.md) for the
  store FSM that drives the BS\* components.
- [07 — Persistence](./07_persistence_and_storage.md) for what each
  command writes / reads on disk.
