# Roadmap

## v0.1 — vertical slice (livré ce soir)

- Workspace cargo, 2 crates (`bestel-core`, `bestel-tui`)
- Parser PoB XML (PoE1 + PoE2)
- Watcher filesystem `notify` sur les 2 dossiers Builds
- TUI Ratatui 2-pane (build context + chat)
- Provider Anthropic (clé API, streaming SSE)
- Tool LLM `get_active_build`
- System prompt Bestel

## v0.2 — multi-provider

- Adapter OpenAI (GPT-4o + 4-turbo)
- Adapter Gemini (1.5 Pro/Flash + 2.0)
- Détection automatique des CLI installés (Codex CLI, Gemini CLI) via subprocess
- Sélecteur de provider dans le TUI (`/provider`)

## v0.3 — données live

- `wiki_fetch` : poewiki Cargo API + cache SQLite TTL 24h
- `ninja_fetch` : poe.ninja (économie + builds top ladder)
- `poe2db_fetch`
- `patch_ctx` : league active, version courante, dernier patch note
- Tools exposés au LLM, RAG léger sur le cache

## v0.4 — import externe

- Décodage PoB share code (base64 + zlib + XML)
- Support `pobb.in/...`, `maxroll.gg/poe/pob/...`, `pastebin.com/...`
- Clipboard watcher (`arboard`) : import auto au copier d'un code

## v0.5 — vision

- Drop zone screenshot dans le TUI
- Multimodal Anthropic / OpenAI / Gemini
- Tool `analyze_screenshot` pour items, atlas, compétences

## v0.6 — façade MCP

- Crate `bestel-mcp` (rmcp)
- Tools partagés exposés en MCP pour Claude Desktop / Code, Cursor, Windsurf
- Mêmes outils, deux façades

## v0.7 — PoB headless

- Sous-process Lua avec wrapper inspiré `BPL-v2/PathOfBuildingHttpServer`
- Calculs *what-if* (swap d'item, changement de tree)
- Migration sur l'API JSON-RPC officielle quand le PR #9505 est mergé

## v1.0 — distribution

- Installer MSI signé via `cargo-wix`
- Auto-update via `self_update` (GitHub releases)
- CI GitHub Actions Windows
- Démo vidéo, page de release
