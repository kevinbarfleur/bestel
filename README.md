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

## Status — read this before you try it

Bestel is an **early preview**. It does a lot of useful things already,
but it can still be wrong, especially on build-specific math. The
whole point of this release is to share it with a few friendly
testers so I can see where it actually breaks.

**✅ What works today**
- Live PoB watcher — save a build in Path of Building / PoB2, it
  appears in the sidebar within a second.
- PoB engine sidecar — Bestel can run the real PathOfBuilding calc
  pipeline to get actual DPS / EHP / max-hit numbers instead of
  paraphrasing the sheet.
- Curated PoE research toolkit — wiki, PoEDB, RePoE mod pools, trade
  stat resolution, whitelisted web fetches, plus a knowledge base
  with build methodology, item taxonomy, source policy.
- Side-panel artifacts — items, keystones, mechanics get a dedicated
  panel with mod lists, scaling notes, and a one-click "open the wiki
  page in app".
- Persistent chat history per build.
- Build Sheet workflow — a structured spec the agent re-reads on
  every later turn instead of redoing the interview.

**⚠️ Integrated but not reliable yet — what I'm actively working on**

These are real today, integrated end-to-end, but still rough. They
double as my current priority list:

- **Claim grounding** (top priority). The model sometimes invents
  build-specific numbers ("you'll oom in 2 s", "your resists are
  uncapped at 60 %") even when the real numbers are one tool call
  away. There's a verification pipeline catching some of it and I'm
  iterating on stronger guardrails (active rewrite-on-the-fly instead
  of warn-only). For now — **don't trust a numeric claim about your
  build without spot-checking it in PoB**.
- **PoE2 coverage.** PoE1 is the better-tested side. PoE2 is
  supported but the bundled methodology references are thinner; I'm
  filling those in (keystones, mod tiers, league mechanics).
- **Process-narration leaks.** Phrases like "let me check" sometimes
  end up in the final answer instead of staying internal. Cosmetic
  but annoying; on the cleanup list.

If something on this list blocks you, open an issue — I prioritise
by what testers actually run into.

---

# Part 1 — Using Bestel

## What it does, in one paragraph

You save a build in Path of Building. Bestel sees it appear and reads
the XML. You ask a question — "am I going to one-shot in Maven
phase 3?", "should I swap my chest for a Cloak of Defiance?", "how
much chill effect does Hatred give me with Elemental Focus support?".
The agent calls the right tools (active build, PoB calc, wiki,
datamine, trade), composes a short answer, and pops a side panel for
the specific item / skill / mechanic when there is one to look at.

## Install

1. Grab the latest **bestel-vX.Y.Z-windows.zip** from the
   [Releases page](https://github.com/kevinbarfleur/bestel/releases).
2. Unzip somewhere and double-click `bestel.exe`. Bestel stores its
   data under `%USERPROFILE%\.bestel\` (chats, cache, your API key) —
   nothing escapes that folder.

Requirements:
- Windows 10 / 11 (Linux compiles from source; macOS is on hold).
- **Path of Building** (PoE1) **or** **Path of Building 2** (PoE2)
  installed and used at least once. Bestel reads the build XML files
  PoB saves under your Documents folder — no extra config needed.

## First-time setup, inside the app

Everything happens in the app — no environment variables to set, no
command lines to run, no config files to edit.

1. **Launch Bestel.** The first launch sets up local caches; this is
   a one-shot, takes a few seconds.
2. **Open the model picker** — press `Ctrl+P` (or click the model name
   in the bottom-right corner).
3. **Pick a model** from the list. Each entry shows a one-line
   description, the price per million tokens, and a status dot
   (green = ready, dim = needs setup).
4. **Paste your API key** in the field on the right pane, then click
   **Save key**. The key is stored in `~/.bestel/runtime/keys.json`
   (per-user) — Bestel never sends it anywhere except the provider
   you picked.
5. Click **Use this model**. You're done — close the picker and ask
   Bestel a question.

That's the whole setup. You can swap models any time the same way.

## Pick your model

Bestel ships with three cloud profiles plus local Ollama. **You only
need one of them**. My recommendations:

| If you want… | Use this | Why |
|---|---|---|
| **The cheapest way to try Bestel** | **DeepSeek V4-Flash** | About $0.001–0.01 per answer. Lets you see what the tool is capable of without burning credits. Weaker on long build audits but great for fast lookups. |
| **The best answer quality** | **Claude Sonnet 4.6** | Frontier reasoning, deepest tool chains, best at staying grounded on build-specific math. About $0.02–0.20 per answer. |
| **A free local setup** | **Ollama** with `qwen3:14b` or bigger | Runs on your own GPU, nothing leaves your machine. Quality below cloud but usable for lookups. |

Full pricing comparison (per million tokens, input / output):

| Model | Provider | $/Mtok in | $/Mtok out | Speed | Quality |
|---|---|---|---|---|---|
| **DeepSeek V4-Flash** ⭐ first try | DeepSeek | $0.14 | $0.28 | Fast | ⭐⭐⭐ |
| DeepSeek V4-Pro | DeepSeek | $1.74 | $3.48 | Balanced | ⭐⭐⭐⭐ |
| Claude Haiku 4.5 | Anthropic | $1 | $5 | Fast | ⭐⭐⭐⭐ |
| **Claude Sonnet 4.6** ⭐ best | Anthropic | $3 | $15 | Balanced | ⭐⭐⭐⭐⭐ |
| Claude Opus 4.7 | Anthropic | $5 | $25 | Slow | ⭐⭐⭐⭐⭐ |
| Ollama (qwen3 / llama3 / etc.) | Local | free | free | Hardware-bound | ⭐⭐ → ⭐⭐⭐⭐ |

The picker shows the live cost telemetry per turn, so you can see
exactly what each answer costs as you go.

### Getting an API key

- **Anthropic (Haiku / Sonnet / Opus)** — sign up at
  [platform.claude.com](https://platform.claude.com/settings/keys),
  create a key starting with `sk-ant-`, paste it in the picker.
- **DeepSeek (V4-Flash / V4-Pro)** — sign up at
  [platform.deepseek.com](https://platform.deepseek.com/apikeys),
  create a key starting with `sk-`, paste it in the picker.
- **Ollama (local)** — install [Ollama](https://ollama.com), then
  `ollama pull qwen3:14b` (or any tool-calling model you like, see
  the [model library](https://ollama.com/library)). Bestel
  auto-discovers anything you've pulled — no key needed.

> **A note on local quality.** Ollama answers are still noticeably
> below the cloud models on long PoE reasoning chains — local
> open-weight models lag the frontier on agentic tool use. The local
> path works for quick lookups today; I'm actively working on the
> tool dispatch + prompt shape so the qwen3 / llama3 class can keep up
> on full build audits. Use Sonnet 4.6 (or DeepSeek V4-Flash if you
> want it cheap) until the local path catches up.

The picker has a **"Get a key from <provider> ↗"** link right under
the input field that opens the right page in your browser.

## How Bestel reads your build

You don't upload a file. Bestel watches the folder Path of Building
saves into — `%USERPROFILE%\Documents\Path of Building\Builds\` for
PoE1, the equivalent PoB2 path for PoE2 — and reacts when something
changes.

- **Save in PoB** → the build appears in Bestel's sidebar within ~1 s.
- **Swap an item, recompute, save again** → the next message you send
  already sees the new gear.
- **Multiple builds in PoB** → press `Ctrl+B` to open the build picker
  and pick which one is "active" for the current conversation. Each
  saved chat remembers which build it was attached to.

The Build Sheet workflow goes one step further: after a deep audit,
Bestel can write a structured "build sheet" (intent, archetype,
defining items, known gaps) that future conversations re-read instead
of redoing the whole interview. Drift detection flags when your gear
diverges from the sheet so the agent knows when to refresh.

## Keyboard shortcuts

| Shortcut | What it does |
|---|---|
| `Ctrl+P` | Open the model picker (pick model, paste API key) |
| `Ctrl+B` | Open the build picker (switch the active build) |
| `Ctrl+N` | Start a fresh conversation |
| `Ctrl+,` | Open settings |
| `Esc` | Close the current picker / panel, or cancel a streaming reply |

## What sources Bestel trusts

The agent uses a strict source allowlist: official `pathofexile.com`
→ official wikis (`poewiki.net`, `poe2wiki.net`) → datamined sources
(`poedb.tw`, `poe2db.tw`, `repoe-fork`) → calculators
(`pathofbuilding.community`, `craftofexile.com`) → economy
(`poe.ninja`) → trusted creator guides (Maxroll, Mobalytics, pohx.net)
with explicit patch + author + date. Fandom, Fextralife, RMT sites
and AI-aggregator answers are blocked. Full registry:
`prompts/references/15_source_registry.md`.

---

# Part 2 — Contributing

If you want to help Bestel get less wrong, this section is for you.
The two highest-leverage areas are **the knowledge layer** (what
Bestel knows about PoE before it starts) and **the prompts** (how
it behaves). Both are plain markdown you can edit.

## Local development

```sh
git clone https://github.com/kevinbarfleur/bestel
cd bestel

# frontend deps (once)
cd crates/bestel/ui && npm install && cd ../../..

# run with hot-reload (Vite + cargo watch)
cd crates/bestel
cargo tauri dev
```

Requirements: **Rust 1.80+**, **Node 18+**.

Focused tests (don't use `--workspace` on low-RAM machines):

```sh
cargo test -p bestel-core --lib
cargo fmt --all
cargo clippy -p bestel-core --lib --tests --no-deps
```

## The knowledge layer

There are **three** distinct knowledge surfaces, in increasing
specificity:

### `prompts/SYSTEM_PROMPT.md` — the runtime system prompt

Bestel's personality, refusal stance, response shape, source policy.
Kept tight on purpose. If you find Bestel doing the wrong thing
systematically (e.g. ignoring the active build, or padding answers
with disclaimers), the fix usually goes here.

### `prompts/CORE_KNOWLEDGE.md` — the conceptual layer

Build ontology (archetypes, ascendancy axes), search planning
heuristics, validation reflexes, PoE priors. Not facts that change
patch-to-patch — methodology only.

### `prompts/references/` — the long-form library

Markdown chapters the agent retrieves on-demand via
`read_internal_reference` and `kb_search`. **This is where contributors
add lasting knowledge.** Each file is a chapter:

```
prompts/references/
├── 01_player_taxonomy.md           # how to read what a player wants
├── 10_skills_gems_passives.md      # gem mechanics, passive tree basics
├── 14_pob_workflow.md              # PoB literacy for build advice
├── 15_source_registry.md           # the source allowlist
├── 16_build_methodology.md         # creator workflows + decision tree
├── 20_item_basetype_identity.md    # item taxonomies, mod tiers
├── 30_panel_marker_grammar.md      # how side panels are encoded
├── 32_build_sheets.md              # build-sheet schema and lifecycle
└── ...                             # several more — list in the dir
```

To add knowledge:

1. Drop a new markdown file with a numbered prefix matching the
   neighbourhood (`12_xxx.md` for skills, `2x_xxx.md` for items…).
2. Write it as a chapter — explain the concept, give 2–3 examples,
   list edge cases. Don't dump version-pinned numbers unless you
   explicitly mark them as such — PoE patches move fast and a stale
   number is worse than no number (the agent should always re-fetch
   live values).
3. Restart Bestel — the index ingest runs at startup and picks up
   new chunks automatically.

## Modifying the system prompt

The shipped prompt lives at `prompts/SYSTEM_PROMPT.md`. On first
launch Bestel **copies** it to `~/.bestel/prompts/SYSTEM_PROMPT.md`
and from then on reads the user copy. This lets you tweak the prompt
without rebuilding:

```sh
notepad %USERPROFILE%\.bestel\prompts\SYSTEM_PROMPT.md
# then restart Bestel
```

The in-app **Prompts & documentation** window exposes a side-by-side
diff between the shipped and the user copy, so you can see what
you've drifted and reset cleanly.

If your prompt change is general-purpose and improves results, open
a PR against `prompts/SYSTEM_PROMPT.md`. Treat prompt changes as
code — they go through the eval set the same way a Rust change would.

## Project layout

```
crates/
├── bestel-core/        # LLM providers, tools, prompts, PoB parser, RAG, sheets
├── bestel/             # Tauri binary + Vue UI (under ui/)
├── bestel-pob-engine/  # LuaJIT sidecar + vendored PathOfBuilding
├── bestel-rag/         # LanceDB hybrid retrieval layer
├── bestel-test/        # scenario + regression harness
└── bestel-driver/      # CDP harness for live UI tests
prompts/                # bundled system prompt, core knowledge, references
docs/architecture/      # entry-point architecture docs — read these first
tests/eval/             # scenarios + baselines
```

The architecture entry points are in
[`docs/architecture/`](docs/architecture/) — short, navigation-focused.
Start with `01_runtime_topology.md` and follow the links.

## License

[MIT](LICENSE). The bundled Path of Building vendor code keeps its own
license — see `crates/bestel-pob-engine/vendor/`.
