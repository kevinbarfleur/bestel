# Contributing to Bestel

Thanks for taking the time. Bestel is an early preview I share with a
few testers — every issue, PR, or prompt tweak helps me see where it
actually breaks.

This guide covers:

- [Where contributions help most](#where-contributions-help-most)
- [Local development setup](#local-development-setup)
- [Workflow: from an idea to a merged PR](#workflow-from-an-idea-to-a-merged-pr)
- [Design system rules](#design-system-rules)
- [Testing](#testing)
- [Code style](#code-style)

---

## Where contributions help most

In rough order of impact for the current state of the project:

1. **The knowledge layer** — `prompts/references/*.md`. These markdown
   chapters are what the agent retrieves when it researches a topic.
   Adding a well-structured reference about a niche mechanic, a build
   archetype, or a crafting workflow gives Bestel a permanent boost
   on that subject. **No Rust required.**
2. **Prompts** — `prompts/SYSTEM_PROMPT.md`, `prompts/CORE_KNOWLEDGE.md`.
   Treat these as code: a one-word change can swing eval pass rates.
   See the [Prompts](#modifying-the-system-prompt) section.
3. **Frontend / UX** — Vue 3 + Pinia + Tauri 2. Components live in
   `crates/bestel/ui/src/components/`. See [Design system rules](#design-system-rules)
   before opening a UI PR.
4. **Backend** — Rust workspace under `crates/`. The architecture
   entry points are in [`docs/architecture/`](docs/architecture/); start
   with `01_runtime_topology.md`.

If you're not sure where your idea fits, **open an issue first** — it's
faster than guessing.

---

## Local development setup

```sh
git clone https://github.com/kevinbarfleur/bestel
cd bestel

# Frontend deps (once)
cd crates/bestel/ui && npm install && cd ../../..

# Hot-reload dev loop (Vite + Tauri)
cd crates/bestel
cargo tauri dev
```

Requirements: **Rust 1.80+**, **Node 18+**, **Windows 10/11** (Linux
compiles, macOS is on hold).

### Useful test commands

> Don't use `cargo test --workspace` on low-RAM machines — the
> repo's documented to OOM at workspace scope. Scope every command
> to a single crate.

```sh
# Rust lib tests for the core crate
cargo test -p bestel-core --lib

# Format + clippy (no -D warnings, the baseline has pre-existing ones)
cargo fmt --all
cargo clippy -p bestel-core --lib --tests --no-deps

# Frontend type-check + build
cd crates/bestel/ui && npm run build

# Headless eval — runs a TOML scenario against any provider
cargo build --release -p bestel
./target/release/bestel.exe run-battery tests/eval/scenarios-mini \
    --model deepseek-v4-flash --out /tmp/bestel-battery
```

---

## Workflow: from an idea to a merged PR

1. **Open an issue first** if the change is non-trivial (new feature,
   architectural change, prompt rewrite). For typos, small bug fixes,
   or doc tweaks, skip straight to the PR.
2. **Fork** the repo and create a branch off `main`. Branch name
   convention: `feat/<short-slug>`, `fix/<short-slug>`,
   `docs/<short-slug>`, `chore/<short-slug>`.
3. **Make the change**, keeping the diff focused. One concern per PR.
4. **Run the relevant tests** (see [Testing](#testing)).
5. **Open the PR**. The template (`.github/PULL_REQUEST_TEMPLATE.md`)
   pre-fills the sections I want to see. Don't strip them — the
   checklist exists so reviews are fast.

### What makes a PR easy to merge

- **Clear "Why".** Bullet points: the problem, what the user
  experiences today, what they'll experience after. One screenshot is
  worth a thousand words for any UI change.
- **Small surface.** A 200-line PR with one reviewer's eyes on it
  ships faster than a 2000-line PR that needs three rounds.
- **Tests for the bug class, not just the bug.** If you fix a
  fabrication pattern in a prompt, add an eval scenario in
  `tests/eval/scenarios/` so the next regression is caught
  automatically.
- **Commit messages follow the existing style.** Look at recent
  commits (`git log --oneline -10`): `feat(scope): …`, `fix(scope): …`,
  `chore(scope): …`. Subject line under ~70 chars.

---

## Design system rules

Bestel has a deliberate visual identity — manuscript-style, small
caps, leader dots, EB Garamond + Kalam typography, parchment palette
in light and slate palette in dark. **Don't fight it.**

### Browse what already exists

In dev mode, the components lab is at `/components`:

```sh
cd crates/bestel
cargo tauri dev
# in another shell, open http://tauri.localhost/#/components in the
# WebView, or just navigate inside the app to that hash
```

It shows every Runic primitive (buttons, inputs, selects, modals,
toasts, etc.) and a Design Tokens panel — palette, type scale, motion
tokens — all reading live from `src/styles/tokens.css`.

> **The components lab is dev-only.** It's lazy-imported behind
> `import.meta.env.DEV` in the router, so it doesn't ship in the
> production bundle.

### Rules of thumb

- **Tokens, not hex codes.** Always reference CSS variables from
  `src/styles/tokens.css` (`var(--ink)`, `var(--paper)`, `var(--amber)`,
  `var(--el-fire)`, etc.). Adding a one-off `#aa6633` somewhere is the
  fastest way to drift the palette.
- **Prefer existing Runic primitives.** Need a button? `RunicButton`.
  A select? `RunicSelect`. A toggle? `RunicToggle`. Building a new
  primitive should be a last resort.
- **If you add a new primitive,** include it in `ComponentsLab.vue` so
  the next contributor can find it.
- **Type families have semantic meaning.** EB Garamond (`--serif` /
  `--hand`) for prose and labels. Kalam / Caveat (`--script`) for
  handwritten annotations only. JetBrains Mono (`--mono`) for code,
  paths, masked keys. Don't swap them around for style.
- **Functional colors stay functional.** `--good` is green for "API
  key set / online", `--bad` is red for "invalid / offline",
  `--note` is amber for "warning / stale". Never use them as chrome
  accents.
- **Density matters.** Default is compact (`[data-density='compact']`).
  Anywhere you set padding or row height, use the density tokens
  (`--pad-card`, `--gap-section`, `--row-h`) so the whole app scales
  consistently.

---

## Modifying the system prompt

The shipped prompt lives at `prompts/SYSTEM_PROMPT.md`. On first
launch Bestel **copies** it to `~/.bestel/prompts/SYSTEM_PROMPT.md`
and from then on reads the user copy. This lets you tweak the prompt
without rebuilding:

```sh
notepad %USERPROFILE%\.bestel\prompts\SYSTEM_PROMPT.md
# then restart Bestel
```

The in-app **Prompts & documentation** window has a side-by-side
diff between the shipped and the user copy.

If your prompt change is general-purpose and improves results:

1. Test it against `tests/eval/scenarios-mini/` on at least
   DeepSeek V4-Flash (cheap) and Claude Haiku 4.5.
2. Don't add hardcoded game values to conceptual references — patches
   move fast and a stale number is worse than no number.
3. Open a PR against `prompts/SYSTEM_PROMPT.md`. Mention which
   scenarios you re-ran and what changed.

---

## Adding knowledge to `prompts/references/`

Each file there is a chapter the agent retrieves on demand.

1. Drop a new markdown file with a numbered prefix matching the
   neighbourhood:
   - `0x_*.md` — player + reasoning fundamentals
   - `1x_*.md` — skills, gems, passives
   - `2x_*.md` — items, mod tiers
   - `3x_*.md` — panel grammar, build sheets
2. Write the file as a chapter — explain the concept, give 2–3
   worked examples, list edge cases. Avoid version-pinned numbers
   unless you explicitly mark them (`(as of patch 3.25)`).
3. Restart Bestel. The KB ingest runs at startup and picks up new
   chunks automatically.
4. Verify the chunk is reachable: open the dev panel (`Ctrl+Shift+D`),
   pick the **KB** tab, type a query that should hit your chunk.

---

## Adding a tool

Tools are how the agent reaches outside the conversation — wiki,
PoEDB, RePoE, trade, PoB calc, web. To add one:

1. Define the JSON schema in `tool_schemas()` in
   `crates/bestel-core/src/llm/tools.rs`.
2. Write the dispatch arm in the `dispatch()` async match (same file).
3. If the tool fetches from a new domain, add the host to the source
   allowlist in `prompts/references/15_source_registry.md`.
4. Document it in `docs/architecture/04_tools_catalogue.md`.
5. Add an integration scenario in `tests/eval/scenarios/` that calls
   the new tool with a representative prompt.

---

## Testing

| Area | Command |
|---|---|
| Core lib tests | `cargo test -p bestel-core --lib` |
| Format + lint | `cargo fmt --all && cargo clippy -p bestel-core --lib --tests --no-deps` |
| Frontend type-check | `cd crates/bestel/ui && npm run build` |
| Single eval scenario | `./target/release/bestel.exe run-battery tests/eval/scenarios --filter-name <name>` |
| Live UI driver | `./target/release/bestel-driver.exe attach --port 9222 && … send "…"` |

If you change anything in the LLM pipeline (provider loop, tool
dispatch, verifier, lint), **add or update an eval scenario** so the
next regression is caught automatically.

---

## Code style

- **English only** in code, comments, commits, and persisted artifacts.
  The conversation between maintainers can be in any language but the
  repo is English-only.
- **No `unwrap()` / `expect()` in hot paths.** Use `anyhow::Result`
  and `.context(...)`.
- **No useless comments.** Names should carry intent; comments
  explain WHY when the WHY isn't obvious (a hidden constraint, a
  workaround for a bug, an invariant).
- **No new abstractions for the first occurrence.** Wait for the
  second real use case before refactoring into a helper.
- **Async by default.** Sync I/O only when the operation is CPU-bound
  and isolated.

---

## License

By contributing, you agree your changes ship under the [MIT license](LICENSE)
that covers the rest of the repo.
