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
  <img src="https://img.shields.io/badge/platform-Windows-lightgrey" alt="Windows" />
  <img src="https://img.shields.io/badge/built%20with-Rust%20%2B%20Tauri%202-orange" alt="Built with Rust + Tauri 2" />
  <img src="https://img.shields.io/badge/status-early%20preview-yellow" alt="Early preview" />
</p>

---

The voice is Bestel, chronicler of Lioneye's Watch. Wraeclast wisdom,
no pep talks.

## Project status — read this before you try it

Bestel is an **early preview**. It does a lot of useful things already,
but it is not production-grade yet — answers can be wrong, especially
on build-specific math. The whole point of this release is to share it
with a few friendly testers so I can see where it actually breaks.

### ✅ What works today

- **Live PoB watcher** — save a build in Path of Building / PoB2, it
  appears in the sidebar within ~1 s. Switching builds updates the
  active context automatically.
- **PoB engine sidecar** — a bundled LuaJIT subprocess runs the real
  PathOfBuilding calc pipeline. The agent can ask for actual DPS / EHP
  / max hit numbers against a Pinnacle profile, not just paraphrased
  sheet values.
- **Tool-calling research loop** — Anthropic Messages API (native
  tool-use + thinking blocks) and Ollama, sharing the same dispatch
  layer. Tools: wiki parse / search / synergies, PoEDB lookups, RePoE
  mods/base-item joins, GGG trade stat resolution, whitelisted web
  fetches, and the PoB calc.
- **RAG over a curated PoE knowledge base** — LanceDB hybrid retrieval
  (vector + BM25) over the bundled reference docs (build methodology,
  source registry, item taxonomy, mod tier conventions, etc.). The
  agent searches the KB before guessing.
- **Side-panel artifacts** — when the answer hinges on a specific item
  / keystone / mechanic, Bestel pops a structured panel with the mod
  list, scaling notes, and a one-click "open the wiki page in app".
- **Build Sheet workflow** — guided interview that distills your build
  into a structured spec the agent can re-read on every later turn,
  with drift detection when your gear / tree / skills change.
- **Persistent chats** — every conversation is saved locally with the
  attached build, restored on the next launch.
- **MCP server mode** — `bestel mcp-serve` exposes the PoE tools over
  JSON-RPC so you can wire them into Claude Desktop, Cursor, etc.

### ⚠️ Integrated but not reliable yet

This is the part I am actively working on:

- **Claim grounding.** The model sometimes invents build-specific
  numbers ("you'll oom in 2 s", "your resists are uncapped at 60 %")
  without checking the actual PoB data — even when the data is one
  tool call away. There's a Chain-of-Verification pass, a response
  linter and a turn classifier helping, but **don't trust a numeric
  claim about your build without spot-checking it in PoB**.
- **Process narration leakage.** Phrases like "let me check" or "I'll
  search the wiki" sometimes leak into the final answer instead of
  staying internal. Cosmetic but annoying.
- **PoE2 coverage.** PoE1 is the better-tested side. PoE2 is supported
  (parsing, trade, wiki) but the bundled methodology references are
  thinner.
- **Cost on big builds.** A full build audit with Sonnet can spend
  10–30 ¢ per turn. The free DeepSeek tier and Haiku are cheaper but
  weaker on long reasoning chains.

### 🛠️ On the roadmap

- Active response-lint mode (rewrite-on-the-fly instead of warn-only)
- Verifier that always reads the same-turn tool transcript before
  judging build claims (Sprint v6 is shipping the foundations)
- Better PoE2 reference coverage (keystones, mod tiers, league
  mechanics)
- macOS build (currently Windows-only release; Linux compiles)
- Curated build templates so you don't have to start from a blank
  conversation
- Reliability evals you can run yourself to verify a new model is not
  worse than the previous one

If something on this list matters to you, open an issue — I prioritise
by what testers actually run into.

---

# Part 1 — Using Bestel

## What it does, in one paragraph

You save a build in Path of Building. Bestel sees it appear and reads
the XML. You ask a question — "am I going to one-shot in Maven phase
3?", "should I swap my chest for a Cloak of Defiance?", "how much
chill effect does Hatred give me with Elemental Focus support?". The
agent calls the right tools (active build, PoB calc, wiki, datamine,
trade), composes a short answer, and pops a side panel for the
specific item / skill / mechanic when there is one to look at.

## Install

There are two ways: download the prebuilt Windows installer, or build
it from source.

### Option A — prebuilt Windows binary (easiest)

1. Grab the latest `bestel-vX.Y.Z-windows.zip` from the
   [Releases page](https://github.com/kevinbarfleur/bestel/releases).
2. Unzip somewhere and double-click `bestel.exe`. The app stores its
   data under `%USERPROFILE%\.bestel\` (chats, cache, keys) — nothing
   leaks into Program Files.
3. First launch downloads the PoB engine vendor files on demand
   (~30 MB) the first time you ask for a `pob_calc`. Internet required
   that first time only.

### Option B — build from source

```sh
git clone https://github.com/kevinbarfleur/bestel
cd bestel

# frontend deps (once)
cd crates/bestel/ui && npm install && cd ../../..

# release build — needs Rust 1.80+ and Node 18+
cargo build --release -p bestel

# launch
./target/release/bestel.exe          # Windows
./target/release/bestel              # Linux (built but lightly tested)
```

For development with hot-reload:

```sh
cd crates/bestel
cargo tauri dev
```

### Requirements

- **Path of Building** (PoE1) or **Path of Building 2** (PoE2)
  installed and used at least once. Bestel reads the build XML files
  PoB saves under your Documents directory — no extra config needed,
  it just watches the right folder.
- An API key for one of the supported cloud providers (see below) **or**
  a local Ollama install.
- Windows 10/11 is the primary platform. Linux compiles; macOS is on
  hold.

## Pick your model

Bestel ships with three cloud profiles plus local Ollama. **You only
need one of these to be useful.** My recommendations:

### Recommended starting points

| If you want… | Use this | Why |
|---|---|---|
| **The cheapest way to try Bestel out** | **DeepSeek V4-Flash** | ~$0.001–0.01 per answer. Lets you see what the tool can do without burning credits. Good enough on quick lookups; weaker on long build audits. |
| **The best answer quality** | **Claude Sonnet 4.6** | Frontier reasoning, deepest tool chains, best at staying grounded on build-specific math. ~$0.02–0.20 per answer. |
| **A free local setup** | **Ollama** with `qwen3:14b` (or bigger) | Runs on your own GPU, nothing leaves your machine. Quality below cloud but usable for lookups. |

Full cost comparison:

| Model | Provider | $/Mtok input | $/Mtok output | Speed | Quality |
|---|---|---|---|---|---|
| **DeepSeek V4-Flash** ⭐ first try | DeepSeek | $0.14 | $0.28 | Fast | ⭐⭐⭐ |
| DeepSeek V4-Pro | DeepSeek | $1.74 | $3.48 | Balanced | ⭐⭐⭐⭐ |
| Claude Haiku 4.5 | Anthropic | $1 | $5 | Fast | ⭐⭐⭐⭐ |
| **Claude Sonnet 4.6** ⭐ best | Anthropic | $3 | $15 | Balanced | ⭐⭐⭐⭐⭐ |
| Claude Opus 4.7 | Anthropic | $5 | $25 | Slow | ⭐⭐⭐⭐⭐ |
| Ollama (qwen3 / llama3 / etc.) | Local | free | free | Hardware-bound | ⭐⭐ → ⭐⭐⭐⭐ |

The picker (`Ctrl+P`) shows the live token / cost telemetry per turn,
so you can see exactly what each answer costs as you go.

### Setting up cloud (Anthropic / DeepSeek)

1. Grab an API key:
   - [platform.claude.com/settings/keys](https://platform.claude.com/settings/keys)
     for Anthropic models (Haiku / Sonnet / Opus).
   - [platform.deepseek.com/apikeys](https://platform.deepseek.com/apikeys)
     for DeepSeek V4-Flash and V4-Pro.
2. Launch Bestel, hit `Ctrl+P`, pick the profile, paste the key into
   the field on the right, click **Use this model**.
3. Keys are stored under `%USERPROFILE%\.bestel\runtime\keys.json`
   (per-user, never written into the repo). You can also set
   `ANTHROPIC_API_KEY` / `DEEPSEEK_API_KEY` as system environment
   variables — Bestel reads them as a fallback.

You can hot-swap between cloud profiles any time; the active
conversation keeps going with the new model.

### Setting up Ollama (local)

1. Install [Ollama](https://ollama.com) and let the daemon run.
2. Pull a model with native tool calling:
   ```sh
   # 8 GB VRAM — lightweight, OK for quick lookups
   ollama pull qwen3:8b

   # 16+ GB — best balance on consumer hardware
   ollama pull qwen3:14b

   # 48+ GB — approaches Haiku-class on long PoE chains
   ollama pull llama3.1:70b
   ```
3. Launch Bestel. The picker auto-discovers everything you've pulled
   under `Ollama (local)`. Pick one, click **Use this model**.
4. If your Ollama is on a different host or port, set
   `OLLAMA_HOST=http://your-host:11434`.

## How the build system works

The piece that makes Bestel different from a generic LLM chat:

1. **You don't upload a file.** Bestel watches the folder Path of
   Building saves into (`%USERPROFILE%\Documents\Path of Building\Builds\`
   for PoE1, the equivalent PoB2 path for PoE2) and reacts to changes.
2. **The XML is parsed locally.** Class, ascendancy, level, gear,
   passives, skill links, jewels, anointments, allocated keystones —
   all extracted into a structured payload.
3. **The active build is one tool call away.** Every conversation
   knows which build is loaded. When you ask a question that's
   actually about your character, the model calls `get_active_build`
   and reads the payload. You can also force a fresh `pob_calc` for
   computed numbers (real DPS, max hit, EHP, recovery breakdowns).
4. **The build moves with the conversation.** If you swap a chest in
   PoB, the next message you send already sees the new gear. If you
   start a new chat with `Ctrl+N`, it inherits the currently active
   build by default.

The Build Sheet workflow goes one step further: after a deep audit,
Bestel can write a structured "build sheet" (intent, archetype,
defining items, known gaps) that future conversations re-read instead
of redoing the whole interview. Drift detection flags when your gear
diverges from the sheet so the agent knows when to refresh.

## Configuration

| Variable | Default | Role |
|---|---|---|
| `ANTHROPIC_API_KEY` | — | Anthropic API key (env-var fallback). |
| `DEEPSEEK_API_KEY` | — | DeepSeek API key (Anthropic-compat endpoint). |
| `OLLAMA_HOST` | `http://localhost:11434` | Ollama daemon URL. |
| `BESTEL_MODEL` | (picker selection) | One-off override for the model id. |
| `BESTEL_CACHE_DIR` | `~/.bestel/cache` | Cache dir for wiki / trade / fetch tools. |
| `BESTEL_DEV_LOG` | unset | `1` enables the JSONL devlog under `~/.bestel/logs/`. |
| `BESTEL_FILE_LOG` | unset | `1` writes the full tracing log to `~/.bestel/runtime/logs/`. |
| `BESTEL_DEBUG_RECORDER` | unset | `1` enables the per-turn debug autosave to `~/.bestel/runtime/debug-chats/{run_id}.json`. Off by default so testers don't accumulate snapshot files. The in-app **debug** button always works — it reads the run from memory regardless. |
| `RUST_LOG` | `info` | Standard `tracing_subscriber` filter. |
| `BESTEL_CONTACT_EMAIL` | `hi@kevinbarfleur.dev` | Contact email injected into the User-Agent sent to GGG APIs (their ToS asks for one). Override when forking. |

Everything lives under `%USERPROFILE%\.bestel\`:

```
~/.bestel/
├── runtime/
│   ├── keys.json            # your API keys (gitignored, per-user)
│   ├── model.json           # last picked model
│   ├── bestel.sqlite3       # chats + lint findings + verifier records
│   └── debug-chats/         # per-run JSON snapshots
├── cache/                   # wiki / trade / RePoE fetches
├── index/                   # LanceDB knowledge-base index
├── logs/                    # tracing logs (when BESTEL_FILE_LOG=1)
└── prompts/                 # seeded copy of the bundled prompts (editable)
```

## Sources Bestel trusts

The agent uses a strict source allowlist: official `pathofexile.com`
(forum, patch notes, trade, dev docs) → official wikis (`poewiki.net`,
`poe2wiki.net`) → datamined data (`poedb.tw`, `poe2db.tw`,
`repoe-fork`) → calculators (`pathofbuilding.community`,
`craftofexile.com`) → economy (`poe.ninja`) → trusted creator guides
(Maxroll, Mobalytics, pohx.net) with explicit patch + author + date.
Fandom, Fextralife, RMT sites, and AI-aggregator answers are blocked.

Full registry: `prompts/references/15_source_registry.md`.

## MCP server mode

```sh
bestel mcp-serve
```

Bestel speaks JSON-RPC on stdin / stdout. Wire it into Claude Desktop,
Cursor or any MCP-aware client by pointing it at `bestel.exe` with
the `mcp-serve` argument. The same PoE tools the in-app chat uses
become available to the host model.

---

# Part 2 — Contributing

If you want to help Bestel get less wrong, this section is for you.
The two highest-leverage areas are **the knowledge layer** (what the
agent knows about PoE before it starts) and **the prompts** (how it
behaves). Both are plain markdown / TOML you can edit and rebuild.

## Local development setup

```sh
# 1. Clone and install
git clone https://github.com/kevinbarfleur/bestel
cd bestel
cd crates/bestel/ui && npm install && cd ../../..

# 2. Run the dev loop with hot-reload (Vite + cargo watch)
cd crates/bestel
cargo tauri dev

# 3. Run focused tests (don't use --workspace on low-RAM machines)
cargo test -p bestel-core --lib
cargo fmt --all
cargo clippy -p bestel-core --lib --tests --no-deps

# 4. Headless eval against a scenario set
./target/release/bestel.exe run-battery tests/eval/scenarios-mini \
    --model deepseek-v4-flash --out /tmp/bestel-battery
```

For UI-level diagnostics there's a dedicated debug harness — see
[`crates/bestel-driver/`](crates/bestel-driver/). It speaks Chrome
DevTools Protocol to a running Bestel instance:

```sh
# launch Bestel with the CDP listener open
pwsh tools/launch-debuggable.ps1

# in another shell — attach + scripted user turns
./target/release/bestel-driver.exe attach --port 9222
./target/release/bestel-driver.exe new-chat --build "C:\path\to\build.xml"
./target/release/bestel-driver.exe send "what's my fire res?" --wait-completion
./target/release/bestel-driver.exe chat-state | jq .
```

Useful for catching bugs the headless `run-battery` can't see (UI
state drift, streaming oddities, panel rendering).

## The knowledge layer — where Bestel's PoE smarts live

There are **three** distinct knowledge surfaces, in increasing
specificity:

### 1. The runtime system prompt — `prompts/SYSTEM_PROMPT.md`

This is the agent's personality and policy. It's kept tight on
purpose: persona, refusal stance, response shape, source policy. If
you find Bestel doing the wrong thing systematically (e.g. ignoring
the active build, or padding answers with disclaimers), the fix
usually goes here. **Test prompt changes against the eval set before
shipping** — a one-word change can swing pass rates.

### 2. The core knowledge file — `prompts/CORE_KNOWLEDGE.md`

A baked-in conceptual layer that travels with every conversation.
Build ontology (archetypes, ascendancy axes), search planning
heuristics, validation reflexes, GGG/PoE priors. Not facts that change
patch-to-patch — methodology only.

### 3. The references library — `prompts/references/`

Long-form markdown docs the agent retrieves on-demand via
`read_internal_reference` and `kb_search`. This is where contributors
add lasting knowledge. Each file is a chapter — see what's there:

```
prompts/references/
├── 01_player_taxonomy.md          # how to read what a player wants
├── 10_skills_gems_passives.md     # gem mechanics, passive tree basics
├── 14_pob_workflow.md             # PoB literacy for build advice
├── 15_source_registry.md          # the source allowlist
├── 16_build_methodology.md        # creator workflows + decision tree
├── 20_item_basetype_identity.md   # item taxonomies, mod tiers
├── 30_panel_marker_grammar.md     # how side panels are encoded
├── 32_build_sheets.md             # build-sheet schema and lifecycle
└── ...                            # several more — full list in the dir
```

**To add knowledge:**

1. Drop a new markdown file in `prompts/references/` with a numbered
   prefix that places it in the right neighbourhood
   (`12_xxx.md` for skills, `2x_xxx.md` for items, etc.).
2. Write it as a chapter — explain the concept once, give 2–3 worked
   examples, list edge cases. Don't dump version-pinned numbers
   unless explicitly marked (PoE patch-cycle moves fast and a stale
   number is worse than no number — the agent should always re-fetch
   live values).
3. Restart Bestel — the KB ingest runs at startup and picks up new
   chunks automatically.
4. Verify it's findable: open the dev panel (`Ctrl+D`), pick the
   **KB** tab, type a query that should match. You should see your
   chunks in the top results.

### 4. The skills layer — `prompts/skills/`

Tightly scoped behavioural recipes the agent can `load_skill` into
context on demand. Use these for repeatable workflows that don't
belong in the always-loaded core knowledge — for example a specific
crafting decision tree, or the steps to audit an Atziri-killer build.
They keep the system prompt thin.

## Modifying the system prompt

The shipped prompt lives at `prompts/SYSTEM_PROMPT.md`. On first
launch Bestel **copies** it to `~/.bestel/prompts/SYSTEM_PROMPT.md`,
and from then on reads the user copy. This lets you tweak the prompt
without rebuilding:

```sh
# 1. Edit your local copy
notepad %USERPROFILE%\.bestel\prompts\SYSTEM_PROMPT.md

# 2. Restart Bestel (prompt is loaded at chat-creation time)
```

The in-app **Prompts & documentation** window (`Ctrl+Shift+P`)
exposes a side-by-side diff between the shipped and the user copy, so
you can see what you've drifted and reset cleanly.

If your prompt change is general-purpose and improves results, open
a PR against `prompts/SYSTEM_PROMPT.md` in the repo. **Treat prompt
changes as code** — they go through the eval set the same way a Rust
change would.

## Adding a tool

Tools live under `crates/bestel-core/src/llm/tools.rs` (registry) and
each tool is a function in the same crate. To add one:

1. Define the JSON schema in the `tool_schemas()` builder.
2. Write the dispatch arm in the `dispatch()` async match.
3. Add an integration test under `crates/bestel-core/src/test_runner/`.
4. Document it in `docs/architecture/04_tools_catalogue.md`.
5. If the tool fetches from a new domain, add the host to the source
   allowlist in `prompts/references/15_source_registry.md`.

## The eval harness

Reliability is tracked via TOML scenarios in `tests/eval/`. Each
scenario is a prompt + a list of expectations (regex matches,
forbidden phrases, required tool calls). The harness runs them in
batch against any provider:

```sh
./target/release/bestel.exe run-battery tests/eval/scenarios-mini \
    --model claude-sonnet-4-6 \
    --filter-name "build_invented" \
    --out /tmp/eval-out

# Inspect the JSON + lint side-cars
ls /tmp/eval-out
```

Outputs land as `<scenario>.json` (full run trace: messages, tool
calls, stats, verifier verdict) plus `<scenario>.lint.json` (the 12
response-lint rules with severity). Use this to:

- Reproduce a bug a tester reported.
- Smoke-test a prompt change before committing.
- Compare two models on the same scenario set.

The scenarios are generated from `tests/eval/eval_set.toml` via
`cargo run --example eval_split` — keep the master TOML as the
single source of truth.

## Project layout

```
crates/
├── bestel-core/       # LLM providers, tools, prompts, PoB parser, RAG, sheets
├── bestel/            # Tauri binary + Vue UI (under ui/)
├── bestel-pob-engine/ # LuaJIT sidecar + vendored PathOfBuilding
├── bestel-rag/        # LanceDB hybrid retrieval layer
├── bestel-test/       # scenario + regression harness
└── bestel-driver/     # CDP harness for live UI tests
prompts/               # bundled system prompt, core knowledge, references
docs/architecture/     # entry-point architecture docs — read these first
tests/eval/            # scenarios, eval set, baselines
tools/                 # dev scripts (launch-debuggable, cleanup, scenarios)
```

The architecture entry points are in
[`docs/architecture/`](docs/architecture/) — short, navigation-focused.
Start with `01_runtime_topology.md` and follow the links from there.

## Stack at a glance

- **Backend** — Rust 2021 workspace. `tokio` async, `reqwest` (rustls),
  `quick-xml` for PoB, `notify` for the watcher, `rmcp` for MCP,
  `rusqlite` for persistence, `lance` for the KB index.
- **Frontend** — Tauri 2 + Vue 3 + TypeScript + Vite + Pinia. Custom
  titlebar, manuscript-style design system (small caps + leader dots +
  EB Garamond + Kalam).
- **LLM providers** — Anthropic Messages API with native tool-use
  (Anthropic, DeepSeek via Anthropic-compat) and Ollama HTTP API for
  local inference. Both share the dispatch loop in
  `crates/bestel-core/src/llm/tools.rs`.

## License

[MIT](LICENSE). The bundled Path of Building vendor code keeps its own
license — see `crates/bestel-pob-engine/vendor/`.
