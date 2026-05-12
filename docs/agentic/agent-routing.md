# Specialist routing matrix

| Task | Primary specialist | Optional reviewer |
|---|---|---|
| Rust backend, IPC, AppState | rust-tauri-architect | qa-eval-engineer |
| Vue component or Pinia store | vue-ui-engineer | product-ux-flow-designer |
| LLM provider/tool loop | llm-agent-engineer | qa-eval-engineer |
| Prompt/reference edit | prompt-reliability-engineer | llm-agent-engineer |
| RAG, LanceDB, embeddings | rag-knowledge-engineer | qa-eval-engineer |
| PoB parser or engine | pob-engine-specialist | rust-tauri-architect |
| Build Sheet UX flow | product-ux-flow-designer | llm-agent-engineer |
| Eval/lint/baseline | qa-eval-engineer | llm-agent-engineer |
| Installer/release build | release-build-engineer | rust-tauri-architect |
| Agent config | agentic-config-architect | qa-eval-engineer |

## Multi-agent rule

Use at most one primary specialist for implementation. Add one reviewer only when a task crosses boundaries or has high reliability risk.

Avoid permanent multi-agent decomposition for normal feature work. Bestel benefits more from deterministic workflows, strong tests, and targeted specialists than from broad agent swarms.
