# Bestel agent contract

This file is for coding agents that read `AGENTS.md` instead of `CLAUDE.md`.

## Mission

Build Bestel as a reliable Windows-first desktop AI companion for Path of Exile 1 and 2. Preserve the existing architecture unless a change has a clear reliability, maintainability, latency, cost, or UX benefit.

## Non-negotiable project rules

- Code, comments, persisted artifacts, commit messages, and documentation are English only.
- French is allowed only in conversation with the maintainer.
- Do not add broad abstractions for one use case.
- Do not use `unwrap()` or `expect()` in hot/runtime paths. Return typed errors or `anyhow::Result` with context.
- All I/O is async except CPU-bound parsing that is explicitly isolated.
- Keep Rust provider abstractions concrete and inspectable. Do not introduce `async-trait` unless there is a documented reason.
- Windows-first behavior matters: paths, installers, process spawning, and file locks must be handled deliberately.
- Never run a release Rust build that embeds stale frontend output. Build the Vue UI first.

## Development loop

1. Inspect the relevant architecture doc before editing.
2. Make the smallest coherent change.
3. Add or update tests for every behavior change.
4. Run the narrowest useful validation first.
5. Run broader validation before claiming done.
6. Summarize changed files, risks, and follow-up work.

## Required checks by area

Rust backend:

```bash
cargo fmt --all
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

Vue/Tauri frontend:

```bash
cd crates/bestel/ui
npm run build
```

Release-style check:

```bash
(cd crates/bestel/ui && npm run build) && cargo build --release -p bestel -j 1
```

Agent/eval changes:

```bash
cargo run --release -p bestel -- run-battery tests/eval/scenarios --strict
```

Use the eval command only when API keys, fixtures, and cost constraints are appropriate.
