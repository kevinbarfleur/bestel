<p align="center">
  <img src="docs/assets/bestel-avatar.png" width="140" alt="Bestel" />
</p>

<h1 align="center">Bestel</h1>

<p align="center">
  <em>"Let me tell you a tale, exile."</em>
</p>

<p align="center">
  Desktop AI companion for <b>Path of Exile 1 &amp; 2</b> — reads your
  live Path of Building file, reasons about your build with a PoE-veteran
  research loop, and surfaces the named items, keystones and supports
  that actually solve your problem.
</p>

<p align="center">
  <img src="https://img.shields.io/badge/license-MIT-amber" alt="MIT license" />
  <img src="https://img.shields.io/badge/platform-Windows%20%7C%20Linux-lightgrey" alt="Windows | Linux" />
  <img src="https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri%202-orange" alt="Built with Rust + Tauri 2" />
</p>

---

The voice is Bestel, chronicler of Lioneye's Watch. Wraeclast wisdom,
no pep talks.

## What it does

- **Live Path of Building watcher.** Save a build in PoB / PoB2 → it
  appears in the sidebar. Switch builds, the panel updates. The active
  build is sent to the LLM as ground truth.
- **Structured research.** Bestel calls a curated PoE toolkit while
  answering your question — official wiki pages, synergy sweeps
  (`Special:WhatLinksHere`), datamined PoEDB lookups, GGG trade stat
  resolution, allowlisted web fetches. Every tool call shows up as its
  own artifact card in the chat so you can see the work that produced
  the answer.
- **Side panels for items / gems / mechanics.** When the answer hinges
  on a specific entity (a unique to swap, a keystone to allocate, a
  mechanic to understand), Bestel pops a structured side panel with the
  full mod list, comparison vs your current gear, scaling notes. One
  click on the panel chrome opens the wiki page in the in-app webview.
- **Persistent chat history.** Conversations are saved locally with
  their attached build, restored on the next launch.
- **PoE-aware reasoning core.** A baked-in knowledge layer
  (`crates/bestel-core/CORE_KNOWLEDGE.md`) gives the agent build
  ontology, search planning, validation reflexes, and the genre / GGG
  priors — so it plans searches well rather than guessing from memory.
- **MCP server mode.** Run `bestel mcp-serve` to expose Bestel's PoE
  tools to other AI tools (Claude Desktop, Cursor, Windsurf, etc.).

## Two ways to run it

Bestel ships with **two operation modes**. Pick one based on whether
you want frontier-model quality (paid) or fully local / free.

### Mode 1 — Anthropic API (recommended)

You bring your own [Anthropic API key](https://platform.claude.com/settings/keys)
(or DeepSeek API key for the cheaper alternative). Bestel calls the
API directly, with the full tool ecosystem and structured thinking
blocks. Pay-per-token, with prompt caching cutting recurring costs by
~75% per turn.

| Model | Speed | Cost (input / output per Mtok) | Quality |
|-------|-------|-------------------------------|---------|
| Claude Haiku 4.5 (default) | Fast | $1 / $5 | ⭐⭐⭐⭐ |
| Claude Sonnet 4.6 | Balanced | $3 / $15 | ⭐⭐⭐⭐⭐ |
| Claude Opus 4.7 | Heavy | $5 / $25 | ⭐⭐⭐⭐⭐ |
| DeepSeek V3.2 (Anthropic-compat) | Balanced | $0.28 / $0.42 | ⭐⭐⭐ |

**Typical cost per answer**: $0.005–$0.05 with Haiku, $0.02–$0.20 with
Sonnet, $0.05–$0.50 with Opus. Build-coaching answers run wider
research loops (more tool calls) so they hit the higher end. The chat
shows the live cost + cache hit rate per turn.

### Mode 2 — Ollama (local, free)

Run an open-weight model on your own machine via [Ollama](https://ollama.com).
No API key, no subscription, no data leaves your computer. Quality
depends entirely on the model you pull and your GPU/RAM budget.

| Model size | RAM/VRAM | Quality | Note |
|-----------|----------|---------|------|
| 7–8B (e.g. `llama3.1:8b`, `qwen3:8b`) | 8 GB | ⭐⭐ | Reduced tool schema; struggles on multi-step research. Fine for simple lookups. |
| 14–32B (e.g. `qwen3:14b`, `qwen2.5:32b`) | 16–32 GB | ⭐⭐⭐ | Full tool ecosystem; usable for build coaching. The local sweet spot. |
| 70B+ (e.g. `llama3.1:70b`) | 48+ GB | ⭐⭐⭐⭐ | Approaches Haiku quality on long-context PoE questions. |

No cloud-grade model rivals Sonnet/Opus on the depth of PoE research
yet. Local mode is great for the "I want to ask a question about my
build without paying" use case; for hard build-coaching the cloud
modes pull ahead.

### Comparison

| | Anthropic API | Ollama (local) |
|--|---------------|----------------|
| **Cost** | Pay-per-token (cents per answer) | Free, runs on your hardware |
| **Privacy** | Prompts sent to Anthropic | Stays on your machine |
| **Quality** | Frontier (Haiku → Opus) | Depends on the model you pull |
| **Setup** | Drop your API key in settings | Install Ollama, `ollama pull <model>` |
| **Internet required** | Yes (API + tool fetches) | Tool fetches only (wiki, trade); model itself is offline |
| **Tool ecosystem** | Full (build, wiki, trade, synergies, web) | Full (reduced for ≤8B models) |
| **Streaming reasoning** | Yes (Anthropic thinking blocks) | Model-dependent |

You can switch between modes any time via `Ctrl+P` (model picker). The
active conversation continues with whichever mode you pick next.

## Requirements

- **Path of Building** (PoE1) and / or **Path of Building 2** (PoE2)
  installed and used at least once. Bestel reads the build XML files
  those tools save under your Documents directory.
- One of:
  - An **Anthropic API key** ([get one here](https://platform.claude.com/settings/keys))
    — for Mode 1.
  - A **DeepSeek API key** ([get one here](https://platform.deepseek.com/apikeys))
    — also Mode 1, ~10× cheaper than Haiku.
  - **[Ollama](https://ollama.com)** with at least one model pulled —
    for Mode 2.
- **Rust 1.80+** and **Node 18+** to build from source.
- Windows is the primary target. Linux build is supported (less
  testing). macOS support is on hold.

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
./target/release/bestel            # Linux
```

For development with hot-reload:

```sh
cd crates/bestel
cargo tauri dev
```

## Setting up Mode 1 — Anthropic API

1. Get an API key from
   [platform.claude.com/settings/keys](https://platform.claude.com/settings/keys)
   (or [platform.deepseek.com/apikeys](https://platform.deepseek.com/apikeys)
   for DeepSeek).
2. Launch Bestel.
3. Open the model picker (`Ctrl+P`), pick a Claude or DeepSeek
   profile, paste the API key into the field on the right, click
   **Use this model**.
4. The key is stored securely in `~/.bestel/runtime/keys.json` (per-user,
   not in the repo). You can also set `ANTHROPIC_API_KEY` /
   `DEEPSEEK_API_KEY` as environment variables and Bestel will pick
   them up.

That's it. The model picker shows the current cost telemetry per turn
and you can hot-swap between Haiku/Sonnet/Opus/DeepSeek any time.

## Setting up Mode 2 — Ollama (local)

1. Install Ollama from [ollama.com](https://ollama.com) (one-time
   install, system tray on Windows, daemon on Linux).
2. Pull a model with native tool calling. Pick based on your hardware:
   ```sh
   # Light, fast, 8 GB VRAM is enough
   ollama pull qwen3:8b

   # Better quality, 16+ GB recommended
   ollama pull qwen3:14b

   # Frontier-adjacent local, needs serious GPU
   ollama pull llama3.1:70b
   ```
3. Make sure Ollama is running (`ollama serve` if it's not auto-started).
4. Launch Bestel. The model picker auto-discovers every model you've
   pulled and lists them under **Ollama (local)**. Pick one, click
   **Use this model**.

Bestel polls the daemon's `/api/tags` endpoint at startup, so the
picker only shows models you've actually pulled — no fictional ghosts.
If your Ollama is on a different host or port, set
`OLLAMA_HOST=http://your-host:11434`.

**Tested model recommendations for PoE coaching**:
- `qwen3:14b` — best balance of quality and resource budget on a
  single consumer GPU.
- `qwen2.5:32b` — better at multi-step reasoning if you have the VRAM.
- `llama3.1:8b` — reliable fallback when nothing bigger fits.
- For images (screenshots in chat): `llama3.2-vision` or `qwen2.5vl`.

## MCP server mode

```sh
bestel mcp-serve
```

Bestel speaks JSON-RPC on stdin / stdout. Wire it into Claude Desktop,
Cursor or any MCP-aware client by pointing the client at the
`bestel.exe` binary with the `mcp-serve` argument. The same PoE tools
the in-app chat uses become available to the host model.

## Configuration

| Variable | Default | Role |
|---|---|---|
| `ANTHROPIC_API_KEY` | — | Anthropic API key. Read from env if not set in the picker. |
| `DEEPSEEK_API_KEY` | — | DeepSeek API key (Anthropic-compat endpoint). |
| `BESTEL_MODEL` | (picker selection) | Override the model id for one-off testing. |
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama daemon URL (override for remote / non-default port). |
| `BESTEL_CACHE_DIR` | `~/.bestel/cache` | Cache directory for wiki / trade / fetch tools. |
| `BESTEL_DEV_LOG` | unset | `1` to enable JSONL devlog under `~/.bestel/logs/`. |
| `BESTEL_DEV_LOG_DIR` | `~/.bestel/logs` | Devlog directory override. |
| `BESTEL_CONTACT_EMAIL` | `hi@kevinbarfleur.dev` | Email injected into the User-Agent sent to GGG APIs (their ToS asks for a contact). Override when forking. |

## Sources Bestel trusts

The agent uses a strict source allowlist: official `pathofexile.com`
(forum, patch notes, trade, developer docs) → official wikis
(`poewiki.net`, `poe2wiki.net`) → datamined data (`poedb.tw`,
`poe2db.tw`, `repoe-fork`) → calculators
(`pathofbuilding.community`, `craftofexile.com`) → economy
(`poe.ninja`) → trusted creator guides (Maxroll, Mobalytics, pohx.net)
with explicit patch + author + date. Fandom, Fextralife, RMT sites,
and AI-aggregator answers are blocked.

The full registry lives in `docs/references/15_source_registry.md`.

## Stack

- **Backend** — Rust 2021 workspace. `tokio` async, `reqwest` (rustls),
  `quick-xml` for PoB, `notify` for the watcher, `rmcp` for MCP.
- **Frontend** — Tauri 2 + Vue 3 + TypeScript + Vite + Pinia. Custom
  titlebar, manuscript-style design system (small caps + leader dots +
  EB Garamond + Kalam).
- **LLM providers** — Anthropic Messages API with native tool-use
  (Anthropic, DeepSeek via Anthropic-compat) and Ollama HTTP API for
  local inference. Both share the same tool dispatch loop in
  `crates/bestel-core/src/llm/tools.rs`.

The full structure is documented in `docs/references/` (concept docs,
source policy, build reasoning, retrieval playbooks, validation
checklists, vocabulary, plus the Maxroll catalogs and the
build-creators methodology).

## License

[MIT](LICENSE).
