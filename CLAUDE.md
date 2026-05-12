# Bestel - Claude project memory

Bestel is a Windows-first desktop AI companion for Path of Exile 1 and 2. It is a Tauri 2 application with a Rust workspace, Vue 3 + TypeScript frontend, a PoB XML parser, a LuaJIT Path of Building sidecar, a tool-calling LLM loop, LanceDB hybrid RAG, a Build Sheet workflow, and regression/eval tooling.

The goal is not to build a clever demo. The goal is a reliable local assistant that can answer PoE questions using actual sources, actual PoB data, deterministic validation, and minimal user friction.

## Core engineering stance

- Ship small, correct, reversible patches.
- Prefer deterministic code over prompt-only behavior.
- Do not add indirection until the second real use case appears.
- Treat the architecture docs as navigation, but treat code as the source of truth.
- Code, comments, persisted artifacts, docs, and commit messages are English only.
- French is only for conversation with the maintainer.
- No useless comments. Names should carry intent.
- No `unwrap()` or `expect()` in hot/runtime paths. Use `anyhow::Result`, typed errors, and `.context(...)`.
- All I/O is async unless the operation is intentionally CPU-bound and isolated.
- Do not introduce `async-trait` by default. Keep provider dispatch concrete and easy to inspect.
- Windows-first behavior matters: process spawning, paths, file locks, installers, and no-console child processes.

## Repository map

- `crates/bestel-core/` - core library: LLM providers, tools, prompts, PoB parser, semantic layer, RAG, persistence, sheets, sources, MCP.
- `crates/bestel/` - Tauri binary and Vue UI under `crates/bestel/ui/`.
- `crates/bestel-pob-engine/` - LuaJIT sidecar and Path of Building harness.
- `crates/bestel-rag/` - LanceDB hybrid retrieval layer.
- `crates/bestel-test/` - scenario and regression harness.
- `prompts/` - bundled runtime prompt, core knowledge, references, and in-app skills.
- `docs/architecture/` - architecture entry points. Read these before cross-cutting changes.
- `tests/eval/` - eval set, generated scenarios, judge rubric, and baseline outputs.

## First files to read by task

Backend runtime or IPC:

- `docs/architecture/01_runtime_topology.md`
- `crates/bestel/src/commands.rs`
- `crates/bestel/src/state.rs`
- `crates/bestel/src/events.rs`

Agent/tool/provider behavior:

- `docs/architecture/02_agentic_system.md`
- `docs/architecture/04_tools_catalogue.md`
- `crates/bestel-core/src/llm/anthropic.rs`
- `crates/bestel-core/src/llm/ollama.rs`
- `crates/bestel-core/src/llm/tools.rs`

Prompts, RAG, references, skills:

- `docs/architecture/03_knowledge_layer_and_rag.md`
- `prompts/SYSTEM_PROMPT.md`
- `prompts/CORE_KNOWLEDGE.md`
- `crates/bestel-core/src/prompts.rs`
- `crates/bestel-core/src/skills.rs`

PoB and build intelligence:

- `docs/architecture/05_pob_integration.md`
- `crates/bestel-core/src/pob/`
- `crates/bestel-pob-engine/`

Build Sheets:

- `docs/architecture/06_build_sheet_workflow.md`
- `prompts/references/32_build_sheets.md`
- `crates/bestel-core/src/llm/sheet_tools.rs`
- `crates/bestel-core/src/sheets/`
- `crates/bestel/ui/src/stores/sheet.ts`

Persistence and storage:

- `docs/architecture/07_persistence_and_storage.md`
- `crates/bestel-core/src/persistence/`
- `crates/bestel-core/src/sheets/store.rs`

Frontend and UX:

- `docs/architecture/01_runtime_topology.md`
- `crates/bestel/ui/src/stores/`
- `crates/bestel/ui/src/components/`
- `crates/bestel/ui/src/api/tauri.ts`
- `crates/bestel/ui/src/api/markdown.ts`

Quality, evals, regression:

- `tests/eval/README.md`
- `tests/eval/eval_set.toml`
- `crates/bestel-core/src/test_runner/`
- `crates/bestel-core/src/llm/recorder.rs`

## Build and validation commands

Default dev loop:

```bash
cd crates/bestel
cargo tauri dev
```

Rust checks:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Frontend type/build check:

```bash
cd crates/bestel/ui
npm run build
```

Fast release-style build without installer:

```bash
(cd crates/bestel/ui && npm run build) && cargo build --release -p bestel -j 1
```

Do not run `cargo build --release` alone for release validation. It embeds whatever is already in `ui/dist/`.

Kill any running `bestel.exe` before rebuilding on Windows.

## Agentic coding protocol

Before editing:

1. Identify the subsystem.
2. Read the relevant architecture doc and the exact files you will modify.
3. State the smallest plan that can solve the problem.
4. Avoid broad refactors unless the user explicitly asked for one.

While editing:

1. Preserve public behavior unless the task asks to change it.
2. Prefer adding a narrow helper over rewriting a module.
3. Keep tool schemas, prompt docs, runtime docs, and tests in sync.
4. For any LLM/tool/prompt change, update or add an eval scenario or lint rule.
5. For any UI event variant change, update Rust DTOs, Pinia handling, and Vue renderers together.

Before reporting done:

1. Run the narrowest tests that cover the changed area.
2. Report any checks not run and why.
3. Summarize files changed, behavior changed, and risk.

## Agent routing

Use specialists when a task is non-trivial or touches a risky area:

- `rust-tauri-architect` - Rust backend, Tauri IPC, AppState, persistence, provider plumbing.
- `vue-ui-engineer` - Vue, Pinia, Tauri events, markdown rendering, Build Sheet UI.
- `llm-agent-engineer` - provider loop, streaming, tool calls, model routing, deterministic orchestration.
- `prompt-reliability-engineer` - prompts, references, in-app skills, output contracts, source policy.
- `rag-knowledge-engineer` - LanceDB, embeddings, chunking, KB eval, retrieval metadata.
- `pob-engine-specialist` - PoB parser, semantic identity, LuaJIT sidecar, engine protocol.
- `qa-eval-engineer` - response lint, run-battery, eval scenarios, judge outputs, regression strategy.
- `product-ux-flow-designer` - chat flow, Build Sheet interview UX, side panels, user friction.
- `agentic-config-architect` - this config, subagent design, skills, slash commands, workflow hygiene.
- `release-build-engineer` - packaging, installers, Windows build, runtime paths, shipped resources.

Default to one specialist. Use multiple specialists only when the task crosses boundaries and needs review, not parallel speculation.

## Model and agent cost discipline

- Use a cheap/fast model for localized edits, summaries, mechanical fixes, test writing, and straightforward UI work.
- Use a stronger model for cross-cutting architecture, LLM reliability changes, prompt rewrites, and debugging failures that survived one focused attempt.
- Do not solve reliability by adding more prompt text. Prefer code-level orchestration, hard gates, lints, evals, and typed state.

## Bestel-specific reliability rules

- Build-loaded answers must be grounded in the active PoB snapshot.
- `pob_calc` numbers must name the effective Calcs config and verify active skill metadata before use.
- Cached PoB values are stale unless explicitly refreshed by the engine.
- Internal references are methodology, not final citations for current game values.
- Source URLs in user-facing answers must come from sources fetched during the same turn.
- PoE1 and PoE2 must never contaminate each other.
- Build Sheet state transitions must be deterministic where possible. Do not rely on a model to remember a critical FSM transition if code can enforce it.

## Prompt editing rules

Prompt changes are code changes. Treat them as risky.

- Do not add hardcoded game values to conceptual references unless they are version-pinned and explicitly marked.
- Do not mention nonexistent tool names.
- Keep tool names exactly aligned with `tool_schemas()`.
- If a prompt creates a new behavioral requirement, add a linter rule, eval scenario, or deterministic code path.
- Keep `SYSTEM_PROMPT.md` compact. Move long methodology into references or skills.

## Done definition

A change is done only when:

- The code compiles or the changed files are otherwise validated.
- The relevant tests/build checks were run or explicitly skipped with reason.
- Docs/prompts/tool schemas/evals are synced when behavior changes.
- The final answer names residual risks.

## Bestel persona and voice (project-specific)

- Bestel is the chronicler of Lioneye's Watch (PoE1 Act 1 NPC). Defined in `prompts/SYSTEM_PROMPT.md`, reinforced by `prompts/CORE_KNOWLEDGE.md`. Do not modify either without discussion.
- Always call `get_active_build` before commenting on the player's character.
- Refuse off-topic questions politely, in character.
- Default language is English; mirror the user's language for prose, keep proper nouns (skills, items, ascendancies, currencies, leagues) in English.

## Test prompts and MCP

- Real-user evaluation prompts live in `docs/test_prompts/real_user_prompts.toml` - single source of truth, voice-preserved. Schema in `docs/test_prompts/README.md`. Every battery runner loads from this file; don't hard-code prompts in Rust.
- Bestel also exposes its MCP server via `bestel mcp-serve` (binary subcommand).
- Prompts under `prompts/` are bundled into the binary via `include_str!` and seeded on first launch to `~/.bestel/prompts/`.
