# Bestel — project context for Claude

Desktop AI companion for Path of Exile 1 and 2. Persona: **Bestel**, chronicler of Lioneye's Watch (PoE1 Act 1 NPC).

## Stack

- Rust 2021, cargo workspace.
- Frontend: Tauri 2 + Vue 3 + TypeScript + Vite + Pinia.
- Async: `tokio`. HTTP: `reqwest` (rustls). XML: `quick-xml`. Watcher: `notify`. MCP: `rmcp`.
- Windows-first; uses `dirs::document_dir()` for portability.

## Crates

- `bestel-core` — library: PoB parser, file watcher, LLM providers (Anthropic API, Claude CLI, Codex CLI), system prompt, baked PoE knowledge layer (`CORE_KNOWLEDGE.md`), tool dispatch, MCP server, wiki / trade / poe.ninja clients.
- `bestel` — Tauri binary: window, IPC commands, event pump for streaming, custom titlebar, Vue UI under `crates/bestel/ui/`. Also exposes MCP via `bestel mcp-serve`.
- `bestel-test` — scenario-based regression harness for agent flows (TOML-driven).

## Conventions

- No useless comments. Names should document the code.
- No `unwrap()` in hot paths; bubble errors with `anyhow::Result`.
- All I/O is async except the PoB XML parser (CPU-bound, sync is fine).
- No `async-trait` — keep traits simple.
- Code, comments, persisted artifacts: **English only**. French is conversational only.

## Bestel's voice

- Defined in `SYSTEM_PROMPT.md` and reinforced by `crates/bestel-core/CORE_KNOWLEDGE.md`. Don't modify either without discussion.
- Always call `get_active_build` before commenting on the player's character.
- Refuse off-topic questions politely, in character.
- Default language is English; mirror the user's language for prose, keep proper nouns (skills, items, ascendancies, currencies, leagues) in English.

## Reference docs

`docs/references/` is the conceptual knowledge base. The numbered files (`00_README` → `16_build_methodology_and_creators`) cover source policy, aRPG foundations, GGG philosophy, game model, build reasoning, character mechanics, offence / defence, itemisation / crafting, skills / gems / passives / ascendancies, endgame / economy / trade, vocabulary, retrieval playbooks, validation, source registry, and build creators. The `maxroll/` subfolder catalogs Maxroll articles with URLs and routing hints.

`crates/bestel-core/CORE_KNOWLEDGE.md` is a distilled always-loaded subset of the references; it is concatenated to `SYSTEM_PROMPT.md` at compile time and shipped to every LLM call.

## Test prompts (real user voice)

`docs/test_prompts/real_user_prompts.toml` is the **single source of truth** for evaluation prompts. ~30 prompts mined from real Reddit / official forum / Maxroll comments, voice-preserved (typos, slang, missing punctuation). Schema and methodology in `docs/test_prompts/README.md`. Every battery runner (current and future) loads from this file — don't hard-code prompts in Rust source.

## Build / dev workflow

- `cargo build --release -p bestel -j 1` for release.
- `cd crates/bestel && cargo tauri dev` for hot-reload Vue + Rust.
- Kill running `bestel.exe` and rebuild after every code change.
