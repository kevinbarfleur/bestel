# Bestel — contexte projet

Assistant IA en terminal pour Path of Exile 1 & 2. Personnalité : **Bestel**, chroniqueur de Lioneye's Watch (PNJ Acte 1 PoE1).

## Stack

- Rust 2021, workspace cargo
- TUI : `ratatui` + `crossterm`
- Async : `tokio`
- HTTP : `reqwest` (rustls)
- XML : `quick-xml`
- Watcher : `notify`

## Crates

- `bestel-core` (lib) — parsing PoB, watcher, client LLM, system prompt, tools
- `bestel-tui` (bin) — interface terminal

## Conventions

- Pas de commentaires inutiles. Le code se documente par des noms clairs.
- Pas de `unwrap()` en chemin chaud, on remonte les erreurs avec `anyhow::Result`.
- Toute I/O est async sauf le parser XML (CPU-bound, sync OK).
- Pas d'`async-trait` pour l'instant — on garde les traits simples.
- Windows-first, mais `dirs::document_dir()` pour rester portable.

## Voix de Bestel

- Voir `SYSTEM_PROMPT.md`. Ne PAS modifier sans en discuter.
- Toujours appeler `get_active_build` avant de commenter le perso.
- Refuser hors-PoE poliment, en restant in-character.

## Hors scope v0.1

OpenAI, Gemini, vision, MCP, wiki fetch, headless PoB, pastebin import, clipboard watcher, SQLite. Voir `ROADMAP.md`.
