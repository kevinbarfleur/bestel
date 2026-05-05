# Bestel

> *"Let me tell you a tale, exile."*

**Bestel** is a desktop AI companion for **Path of Exile 1 and Path of Exile 2**, written for the player who wants more than a wiki tab. It watches your live Path of Building file, reads it, and lets you have a real conversation about your build with an LLM that has been taught to think like a PoE veteran — verify everything on the wiki before answering, distinguish PoE1 from PoE2, and surface the named items, keystones and supports that actually solve your problem.

The voice is Bestel, chronicler of Lioneye's Watch. Wraeclast wisdom, no pep talks.

## What it does

- **Live Path of Building watcher.** Save a build in PoB / PoB2 → it appears in the sidebar. Switch builds, the panel updates. The active build is sent to the LLM as ground truth.
- **Four providers, auto-detected.** Anthropic API key, Claude Code CLI, Codex CLI, or **Ollama for free local inference** — whichever you have. The model picker lets you switch on the fly.
- **Structured chat.** Thinking, tool calls, web searches, build imports, and citations are rendered as inline manuscript artefacts (not buried in raw text).
- **Persistent chat history.** Conversations are saved locally with their attached build, restored on the next launch.
- **PoE-aware reasoning core.** A baked-in knowledge layer (`crates/bestel-core/CORE_KNOWLEDGE.md`) gives the agent build ontology, search planning, validation reflexes, and the genre / GGG priors — so it plans searches well rather than guessing from memory.
- **MCP server mode.** Run `bestel mcp-serve` to expose Bestel's PoE tools (`get_active_build`, `find_synergies`, `wiki_*`, `trade_*`, etc.) to other AI tools (Claude Desktop, Claude Code, Cursor, Windsurf).

## Stack

- **Backend** — Rust 2021 workspace. `tokio` async, `reqwest` (rustls), `quick-xml` for PoB, `notify` for the watcher, `rmcp` for MCP.
- **Frontend** — Tauri 2 + Vue 3 + TypeScript + Vite + Pinia. Custom titlebar, manuscript-style design system (small caps + leader dots + EB Garamond + Kalam).
- **LLM providers** — Anthropic Messages API with native tool-use, plus shell-out to Claude Code CLI and Codex CLI for users who already pay a subscription.

The full structure is documented in `docs/references/` (concept docs, source policy, build reasoning, retrieval playbooks, validation checklists, vocabulary, plus the Maxroll catalogs and the build-creators methodology).

## Requirements

- **Path of Building** (PoE1) and / or **Path of Building 2** (PoE2) installed and used at least once. Bestel reads the build XML files those tools save under your Documents directory.
- One of:
  - `ANTHROPIC_API_KEY` set in your environment, or
  - `claude` CLI ([Claude Code](https://docs.claude.com/en/docs/claude-code)) on your `PATH`, or
  - `codex` CLI ([OpenAI Codex CLI](https://github.com/openai/codex)) on your `PATH`, or
  - **[Ollama](https://ollama.com)** running locally with at least one model pulled (e.g. `ollama pull llama3.1:8b`) — the free / offline path.
- **Rust 1.80+** and **Node 18+** to build from source.
- Windows is the primary target. macOS and Linux build but have less testing coverage.

## Build and run

```sh
git clone https://github.com/kevinbarfleur/bestel
cd bestel

# install frontend deps
cd crates/bestel/ui && npm install && cd ../../..

# release build
cargo build --release -p bestel

# launch
./target/release/bestel.exe        # Windows
./target/release/bestel            # macOS / Linux
```

For development with hot-reload:

```sh
cd crates/bestel
cargo tauri dev
```

## Free / offline mode (Ollama)

Bestel can run end-to-end with no API key and no subscription, against a local [Ollama](https://ollama.com) daemon. The active build, the manuscript-style chat, the structured artefacts, and the PoE-aware reasoning core all keep working — you just lose access to frontier models.

```sh
# 1. install Ollama from https://ollama.com (one-time)
# 2. pull a model with native tool calling
ollama pull llama3.1:8b

# 3. start Ollama (a system tray on Windows / a daemon on macOS+Linux)
ollama serve

# 4. launch Bestel — it auto-detects the daemon and shows installed models in the picker
./target/release/bestel.exe
```

Recommended models for tool calling: `llama3.1:8b` (default), `qwen2.5:7b`, `mistral-nemo`. For vision (image attachments in chat) try `llama3.2-vision`. Heavier rigs can run `llama3.1:70b` for closer-to-frontier reasoning. The picker only shows models you have actually pulled — Bestel polls `/api/tags` at startup.

The persisted active-model preference (`~/.bestel/runtime/model.json`) is honoured across providers, so you can flip between Anthropic and Ollama via the picker without restarting.

## MCP server mode

```sh
bestel mcp-serve
```

Bestel speaks JSON-RPC on stdin / stdout. Wire it into Claude Desktop, Cursor or any MCP-aware client by pointing the client at the `bestel.exe` binary with the `mcp-serve` argument. The same PoE tools the in-app chat uses become available to the host model.

## Configuration

| Variable | Default | Role |
|---|---|---|
| `ANTHROPIC_API_KEY` | — | Anthropic API key. |
| `BESTEL_PROVIDER` | `auto` | Force `anthropic`, `claude`, `codex`, or `ollama`. |
| `BESTEL_MODEL` | latest Sonnet | Model id for the Anthropic / Ollama provider. |
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama daemon URL (override for remote / non-default port). |
| `BESTEL_CACHE_DIR` | `~/.bestel/cache` | Cache directory for wiki / trade / fetch tools. |
| `BESTEL_DEV_LOG` | unset | `1` to enable JSONL devlog under `~/.bestel/logs/`. |
| `BESTEL_DEV_LOG_DIR` | `~/.bestel/logs` | Devlog directory override. |
| `BESTEL_REASONING` | `low` | Codex CLI reasoning level (`minimal` / `low` / `medium` / `high` / `xhigh`). |
| `BESTEL_REASONING_SUMMARY` | `detailed` | Codex CLI reasoning summary (`auto` / `concise` / `detailed`). |
| `BESTEL_CONTACT_EMAIL` | `hi@kevinbarfleur.dev` | Email injected into the User-Agent sent to GGG APIs (their ToS asks for a contact). Override when forking. |

## Sources Bestel trusts

The agent uses a strict source allowlist: official `pathofexile.com` (forum, patch notes, trade, developer docs) → official wikis (`poewiki.net`, `poe2wiki.net`) → datamined data (`poedb.tw`, `poe2db.tw`, `repoe-fork`) → calculators (`pathofbuilding.community`, `craftofexile.com`) → economy (`poe.ninja`) → trusted creator guides (Maxroll, Mobalytics, pohx.net) with explicit patch + author + date. Fandom, Fextralife, RMT sites, and AI-aggregator answers are blocked.

The full registry lives in `docs/references/15_source_registry.md`. Build creator profiles and the build-crafting decision framework are in `docs/references/16_build_methodology_and_creators.md`. The Maxroll article catalogs sit under `docs/references/maxroll/`.

## License

[MIT](LICENSE).
