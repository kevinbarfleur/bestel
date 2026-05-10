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

- Defined in `prompts/SYSTEM_PROMPT.md` and reinforced by `prompts/CORE_KNOWLEDGE.md`. Don't modify either without discussion.
- Always call `get_active_build` before commenting on the player's character.
- Refuse off-topic questions politely, in character.
- Default language is English; mirror the user's language for prose, keep proper nouns (skills, items, ascendancies, currencies, leagues) in English.

## Knowledge layer (`prompts/`)

The agent's load-bearing knowledge layer lives at the workspace root under `prompts/`, mirroring exactly the layout that gets seeded on user disk at `~/.bestel/prompts/`. Every file in this directory is bundled into the binary via `include_str!` and seeded to the user on first launch. Editing a file there changes what the agent sees on the next chat turn.

- `prompts/SYSTEM_PROMPT.md` — voice + hard rules. Always loaded.
- `prompts/CORE_KNOWLEDGE.md` — distilled invariants + tool inventory + reference index. Always loaded; concatenated to SYSTEM_PROMPT at compile time.
- `prompts/references/` — fetch-on-demand conceptual library. Numbered files `00_README` → `26_validation_and_self_correction` cover source policy, aRPG foundations, GGG philosophy, game model, build reasoning, character mechanics, offence / defence, itemisation / crafting, skills / gems / passives / ascendancies, endgame / economy / trade, vocabulary, retrieval playbooks, validation, source registry, build creators, archetype taxonomy, atlas mechanics, combat / movement / animation, basetype identity, currency taxonomy, trade etiquette, mode differences, patch history, PoB engine integration, self-correction. Subfolders: `creators_registry/` (per-creator profiles), `glossary/` (FR↔EN), `poe2/` (version-pinned PoE2 mechanics), `thresholds/` (red maps / pinnacle / uber bars), `maxroll/` (URL catalogue). The agent fetches these on demand via the `read_internal_reference` tool.

`docs/` keeps genuine developer-facing artefacts only (ROADMAP, test_prompts, brainstorm) — none of it is bundled into the agent.

## Test prompts (real user voice)

`docs/test_prompts/real_user_prompts.toml` is the **single source of truth** for evaluation prompts. ~30 prompts mined from real Reddit / official forum / Maxroll comments, voice-preserved (typos, slang, missing punctuation). Schema and methodology in `docs/test_prompts/README.md`. Every battery runner (current and future) loads from this file — don't hard-code prompts in Rust source.

## Build / dev workflow

- `cd crates/bestel && cargo tauri dev` for hot-reload Vue + Rust (default for iteration).
- For a release `.exe`, choose one — **never `cargo build --release` alone**, it embeds whatever is currently in `ui/dist/`:
  - `cargo tauri build` — npm build + Rust release + NSIS/MSI installers.
  - `(cd crates/bestel/ui && npm run build) && cargo build --release -p bestel -j 1` — same effect, no installer, faster iteration.
- Kill running `bestel.exe` before rebuilding.
- `vue-tsc` runs only on `npm run build`, not on `npm run dev`. Type errors from missing union variants or null narrowing across `<template v-else-if>` will pass dev and explode at release time.
