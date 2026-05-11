# Bestel — Architecture

The architecture documentation moved into a structured folder so each
sub-system can be inspected at the right altitude without scrolling
through 800 lines.

**Start here: [`docs/architecture/README.md`](./docs/architecture/README.md)**

The folder contains:

| File | Scope |
|---|---|
| [`README.md`](./docs/architecture/README.md) | Overview, runtime diagram, navigation table |
| [`01_runtime_topology.md`](./docs/architecture/01_runtime_topology.md) | Tauri shell, 3 windows, IPC (49 commands), events, frontend tree, CLI subcommands |
| [`02_agentic_system.md`](./docs/architecture/02_agentic_system.md) | Chat flow, providers, `LlmDelta`, tool loop, CoVe verifier, recorder, MCP server |
| [`03_knowledge_layer_and_rag.md`](./docs/architecture/03_knowledge_layer_and_rag.md) | `prompts/` bundling (69 references), override hierarchy, LanceDB KB |
| [`04_tools_catalogue.md`](./docs/architecture/04_tools_catalogue.md) | The 18 tools the agent can invoke |
| [`05_pob_integration.md`](./docs/architecture/05_pob_integration.md) | PoB parser, semantic layer, headless engine sidecar |
| [`06_build_sheet_workflow.md`](./docs/architecture/06_build_sheet_workflow.md) | Build Sheet feature: phase FSM, 5 sheet tools, 10 components, SQLite |
| [`07_persistence_and_storage.md`](./docs/architecture/07_persistence_and_storage.md) | `~/.bestel/` map, SQLite tables, LanceDB, caches, bundled snapshots |

When in doubt, the source of truth is always the code. If anything
in these documents disagrees with `git`, trust the code and update
the doc.
