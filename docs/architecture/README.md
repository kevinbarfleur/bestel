# Bestel — Architecture

An entry point for anyone (human or agent) trying to understand how Bestel
is wired up. The single-file `ARCHITECTURE.md` at the repo root has been
split into seven specialised documents grouped under this folder so each
sub-system can be inspected at the right altitude.

The source of truth is always the code. When a number in these documents
disagrees with `git`, trust the code and update the doc.

---

## What is Bestel

Bestel is a Windows-first desktop AI companion for **Path of Exile 1 and 2**.
The persona is **Bestel, chronicler of Lioneye's Watch** — a PoE1 NPC who
answers in-character but only after running concrete tool calls against
the user's loaded Path of Building file, the wikis, the bundled
repoe-fork snapshot, the trade catalogue, and a curated reference
library. The audience is PoE veterans, not new players: answers should
land at creator-guide quality (Maxroll / Mobalytics / Pohx), with named
items, exact numbers, and citations.

The product runs as a Tauri 2 desktop app (three windows: chat,
prompt-editor, dev-panel) backed by a Rust workspace, a Vue 3 +
TypeScript frontend, and a vendored LuaJIT subprocess that wraps the
upstream Path of Building engine.

---

## Runtime diagram

```
                       ┌───────────────────────────────────────────┐
                       │            Vue 3 + TypeScript             │
                       │   (main, prompt-editor, dev-panel)        │
                       └────────────────┬──────────────────────────┘
                                        │ invoke / listen
                       ┌────────────────▼──────────────────────────┐
                       │           Tauri 2 IPC layer               │
                       │       49 commands · 5 events              │
                       └────────────────┬──────────────────────────┘
                                        │
              ┌─────────────────────────┼───────────────────────────┐
              │                         │                           │
   ┌──────────▼────────┐   ┌────────────▼──────────┐   ┌────────────▼────────┐
   │   bestel-core     │   │  bestel-pob-engine    │   │   bestel-test       │
   │ (library)         │   │  (LuaJIT sidecar)     │   │  (scenarios)        │
   ├───────────────────┤   ├───────────────────────┤   └─────────────────────┘
   │ llm/  providers   │   │ ndJSON over stdio     │
   │ pob/  parser+sem  │   │ PathOfBuilding-PoE1   │
   │ sources/  http+kb │   │ PathOfBuilding-PoE2   │
   │ mcp/  rmcp server │   │ vendored harness fork │
   │ prompts/ bundle   │   └───────────────────────┘
   │ test_runner/      │
   └────────┬──────────┘
            │
   ┌────────▼──────────┐         ┌──────────────────────────────────┐
   │ Anthropic / DeepSeek/Ollama │ ←──── 5 cloud profiles + dynamic │
   ├───────────────────┤         └──────────────────────────────────┘
   │ Tools loop  (18)  │ ←─── wiki, trade, repoe, pob_calc,
   │ Verifier (CoVe)   │       kb_search, sheet_*, …
   │ Recorder          │ ─→ ~/.bestel/runtime/debug-chats/<id>.json
   └───────────────────┘
            │
   ┌────────▼───────────────────────────────────────────────────────┐
   │  Persistence:  ~/.bestel/  (JSON · SQLite · LanceDB · cache)    │
   └────────────────────────────────────────────────────────────────┘
```

---

## Documents

| File | Scope |
|---|---|
| [`01_runtime_topology.md`](./01_runtime_topology.md) | Tauri shell, three-window topology, 49 IPC commands, 5 events, AppState + cancellation, CLI subcommands, build/dev workflow, frontend tree (Vue components, Pinia stores, styles, markdown pipeline) |
| [`02_agentic_system.md`](./02_agentic_system.md) | Chat flow end-to-end, provider abstraction, `LlmDelta` (17 variants), tool loop (`MAX_AGENT_ITERATIONS = 16`), CoVe verifier, recorder, model profiles, keys, MCP server |
| [`03_knowledge_layer_and_rag.md`](./03_knowledge_layer_and_rag.md) | `prompts/` bundling (69 references), override hierarchy, `read_internal_reference` enum, LanceDB hybrid KB, `kb_search` / `load_skill` caps, source allowlist + blocklist, snapshots refresh |
| [`04_tools_catalogue.md`](./04_tools_catalogue.md) | The 18 tools exposed to the agent — name, schema, trigger rules, dispatch path, allowlisted hosts |
| [`05_pob_integration.md`](./05_pob_integration.md) | PoB parser, semantic `BuildIdentity` layer, headless LuaJIT sidecar (`bestel-pob-engine`), harness fork, engine-aware swap rule, file watcher |
| [`06_build_sheet_workflow.md`](./06_build_sheet_workflow.md) | Build Sheets feature: phase FSM, 5 sheet tools, 10 BS\* Vue components, submission flow, SQLite `build_sheets` table, rehydration, staleness |
| [`07_persistence_and_storage.md`](./07_persistence_and_storage.md) | `~/.bestel/` map, SQLite tables, LanceDB index, HTTP cache, bundled snapshots (repoe + trade catalogue) |

---

## Where to look first

| Question | Read this |
|---|---|
| How does a single chat turn flow from the user to the API and back? | [02 § Chat flow](./02_agentic_system.md#chat-flow-end-to-end) |
| What tools does the agent actually have? | [04 — full catalogue](./04_tools_catalogue.md) |
| Where do system prompts come from and how does the agent override them? | [03 § Override hierarchy](./03_knowledge_layer_and_rag.md#override-hierarchy) |
| How is the PoB engine called and why is the harness forked? | [05 § Engine sidecar](./05_pob_integration.md#engine-sidecar) |
| What is a "Build Sheet" and why are there ten new BS\* components? | [06](./06_build_sheet_workflow.md) |
| Where on disk are chats / sheets / KB / caches stored? | [07](./07_persistence_and_storage.md) |
| What IPC commands exist and what events fire on the frontend? | [01 § IPC](./01_runtime_topology.md#ipc-surface) |
| How does the verifier work and when does it fire? | [02 § Verifier (CoVe)](./02_agentic_system.md#verifier-cove) |
| How is multi-window theme sync done? | [01 § Multi-window](./01_runtime_topology.md#multi-window-topology) |
| How do I add a new bundled reference doc? | [03 § Adding a reference](./03_knowledge_layer_and_rag.md#adding-a-reference) |
| Bestel's voice — who is he? | `prompts/SYSTEM_PROMPT.md` (not duplicated here) |
| Roadmap / sprints / pivots | `docs/ROADMAP.md` (not duplicated here) |

---

## Conventions

- **Code, comments, persisted artefacts: English only.** French is reserved
  for conversation between the developer and an agent.
- **No `unwrap()` in hot paths.** Errors bubble through `anyhow::Result`.
- **All I/O is async** except the PoB XML parser (CPU-bound, sync is fine).
- **No `async-trait`** — providers expose a plain enum and a `match`.
- **Windows-first.** Paths use `dirs::document_dir()` where appropriate;
  the LuaJIT child is spawned with `CREATE_NO_WINDOW`.

---

## Pointers outside this folder

| Concern | Source of truth |
|---|---|
| Voice + hard rules | `prompts/SYSTEM_PROMPT.md` |
| Distilled invariants + reference index | `prompts/CORE_KNOWLEDGE.md` |
| Project-level dev conventions | `CLAUDE.md` (repo root) |
| Sprints, pivots, deferred work | `docs/ROADMAP.md` |
| PoE2 0.5 launch playbook | `docs/POE2_05_LAUNCH_PLAYBOOK.md` |
| Real-user evaluation prompts | `tests/eval/scenarios/*.toml` |
