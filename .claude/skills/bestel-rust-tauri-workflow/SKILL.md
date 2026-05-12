---
name: bestel-rust-tauri-workflow
description: Workflow for Rust and Tauri backend work in Bestel, including IPC commands, AppState, events, provider dispatch, persistence, sources, and workspace crates. Use for backend implementation, debugging, or review.
---

# Rust/Tauri workflow

## Before code

Read the relevant architecture section and locate the runtime boundary.

Common boundaries:

- IPC command in `crates/bestel/src/commands.rs`
- app state in `crates/bestel/src/state.rs`
- event pump in `crates/bestel/src/events.rs`
- LLM/provider/tool code in `crates/bestel-core/src/llm/`
- persistence in `crates/bestel-core/src/persistence/`
- PoB code in `crates/bestel-core/src/pob/`

## Patch rules

- Keep structs typed and serializable at boundaries.
- Add `.context(...)` at subsystem crossings.
- Do not block async runtime with file/network I/O.
- Preserve cancellation behavior for long operations.
- Update frontend wrappers when IPC signatures change.

## Checks

Run the tightest set first:

```bash
cargo fmt --all
cargo test -p bestel-core
cargo test --workspace
```

Use clippy before finalizing broader Rust changes:

```bash
cargo clippy --workspace --all-targets -- -D warnings
```
