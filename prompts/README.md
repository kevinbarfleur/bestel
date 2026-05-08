# prompts/ — agent knowledge layer

This directory holds the agent's load-bearing knowledge files. **Every file
here is bundled into the binary at compile time** via `include_str!` and
seeded to the user's home directory under `~/.bestel/prompts/` on first
launch. The layout in this directory mirrors that home location exactly.

> Edit a file here → next `cargo build --release` → next chat turn the
> agent sees the change. This is **not** documentation; it is the agent.

## Layout

| Path | Role | Loaded |
|---|---|---|
| `SYSTEM_PROMPT.md` | Voice + hard rules. | Always, every turn. |
| `CORE_KNOWLEDGE.md` | Distilled invariants + tool inventory + reference index. | Always, every turn (concatenated to SYSTEM_PROMPT). |
| `references/*.md` | Conceptual library, fetched on demand. | Lazily, via the `read_internal_reference` tool. |
| `references/creators_registry/*.md` | Per-creator profiles (Maxroll authors, streamers). | Lazily. |
| `references/poe2/*.md` | Version-pinned PoE2 mechanics. Always check `00_version_pinning.md`. | Lazily. |
| `references/thresholds/*.md` | Version-pinned numerical bars (red maps / pinnacle / uber). | Lazily. |
| `references/maxroll/*.md` | Curated URL catalogues to applied Maxroll articles. | Lazily. |

## Where this lives at runtime

| Surface | Path |
|---|---|
| Project source | `<repo>/prompts/...` |
| Bundled into binary | `crates/bestel-core/src/prompts.rs` constants (`include_str!`). |
| User-editable copy | `~/.bestel/prompts/...` (Windows: `%USERPROFILE%\.bestel\prompts\`). |
| Editable from the desktop app | Settings → Prompts & docs (open the prompt-editor window). |

The runtime always prefers the user-editable copy. If a file is missing on
disk it falls back to the bundled default. `seed_defaults_if_missing()`
runs at boot and writes any missing default into the user's home — it
never overwrites user edits.

## Adding a new reference doc

1. Drop the new `.md` under `prompts/references/<category>/`.
2. Add a one-line entry in
   `crates/bestel-core/src/prompts.rs::BUNDLED_REFERENCES`
   so it ships with the binary.
3. Add a fetch trigger in `prompts/CORE_KNOWLEDGE.md` so the agent
   knows when to call `read_internal_reference("…")` for it.
4. `cargo build --release -p bestel` — the new file is bundled.
5. (Optional) cross-link from `prompts/references/00_README.md`.

## What does **not** belong here

- ROADMAP, TODO lists, brainstorm scratchpads → `docs/`.
- Test prompt fixtures (`real_user_prompts.toml`, scenarios) → `docs/`,
  `tests/`.
- API key cache, runtime cache, debug runs → `~/.bestel/runtime/` (user
  side, not in repo).
- Anything not meant to influence the agent's reasoning.
