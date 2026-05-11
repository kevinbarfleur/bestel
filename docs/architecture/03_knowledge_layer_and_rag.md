# 03 — Knowledge layer and RAG

What the agent *knows* and how it *retrieves*. Three concentric layers:

1. **Always-loaded prompts** (`SYSTEM_PROMPT.md` + `CORE_KNOWLEDGE.md`) — burned into every turn.
2. **Bundled reference library** (69 files under `prompts/references/`) — fetched on demand via the `read_internal_reference` tool.
3. **LanceDB hybrid KB** (`~/.bestel/index/kb.lance/`) — semantic + BM25 search over the same references, queried by the `kb_search` tool and the verifier.

---

## Layout on disk

```
prompts/                            (repo root — bundled into the binary)
├── SYSTEM_PROMPT.md                # voice + hard rules + identity card requirement
├── CORE_KNOWLEDGE.md               # distilled invariants + tool inventory + reference index
└── references/
    ├── 00_README.md
    ├── 01_source_policy.md
    ├── 02_arpg_foundations.md
    ├── 03_ggg_design_philosophy.md
    ├── 04_game_model_poe1_poe2.md
    ├── 05_build_reasoning_framework.md
    ├── 06_character_stats_and_mechanics.md
    ├── 07_offence_damage_scaling.md
    ├── 08_defence_recovery_survivability.md
    ├── 09_itemisation_crafting.md
    ├── 10_skills_gems_passives_ascendancies.md
    ├── 11_endgame_economy_trade_leagues.md
    ├── 12_vocabulary_glossary.md
    ├── 13_retrieval_playbooks.md
    ├── 14_validation_and_failure_modes.md
    ├── 15_source_registry.md
    ├── 16_build_methodology_and_creators.md
    ├── 17_build_archetype_taxonomy.md
    ├── 18_atlas_and_endgame_mechanics.md
    ├── 19_combat_movement_animation.md
    ├── 20_item_basetype_identity.md
    ├── 21_currency_and_barter_taxonomy.md
    ├── 22_trade_etiquette_and_scams.md
    ├── 23_hardcore_softcore_ssf_mode_differences.md
    ├── 24_patch_history_meta.md
    ├── 25_pob_engine_integration.md
    ├── 26_validation_and_self_correction.md
    ├── 27_response_contracts.md         # Sprint B runtime extension
    ├── 28_tool_failure_policy.md
    ├── 29_known_mechanics_tripwires.md
    ├── 30_panel_marker_grammar.md
    ├── 31_answer_mode_examples.md
    ├── 32_build_sheets.md               # Build Sheets feature
    ├── creators_registry/               # 21 files (00_README + 20 creator profiles)
    ├── poe2/                            # 7 files (version-pinned mechanics)
    ├── thresholds/                      # 3 files (red_maps, pinnacle, uber_pinnacle)
    └── maxroll/                         # 5 files (curated URL catalogues)
```

On first launch the binary mirrors all of `prompts/` to
`~/.bestel/prompts/` so the user can edit anything without rebuilding.
The mirror is one-way: edits are never copied back to the repo.

---

## Bundling

`crates/bestel-core/src/prompts.rs` declares three system prompts and
one big reference table:

```rust
pub const BUNDLED_SYSTEM_PROMPT: &str = include_str!("../../../prompts/SYSTEM_PROMPT.md");
pub const BUNDLED_CORE_KNOWLEDGE: &str = include_str!("../../../prompts/CORE_KNOWLEDGE.md");
pub const BUNDLED_LOCAL_ADDENDUM: &str = include_str!("./llm/local_addendum.md");

pub const BUNDLED_REFERENCES: &[(&str, &str)] = &[
    ("00_README.md", include_str!("../../../prompts/references/00_README.md")),
    // …
];
```

| Group | Count | Notes |
|---|---|---|
| Numbered conceptual docs (00–26) | 27 | Source policy, foundations, mechanics, scaling, defence, items, skills, endgame, vocabulary, retrieval playbooks, validation, source registry, build methodology, archetype taxonomy, atlas, combat / movement / animation, basetype identity, currency taxonomy, trade etiquette, mode differences, patch history, PoB engine integration, self-correction |
| Sprint B runtime extension (27–31) | 5 | Output contracts, failure policy, tripwires, panel grammar, answer-mode examples |
| Build Sheets (32) | 1 | Sheet entry rule + 6 fixed sections + leverage-purpose questions |
| `creators_registry/` | 21 | 00_README + 20 creator profiles (Pohx, Quin, Mathil, Ziggy, Ben, Goratha, Subtractem, Tripolarbear, Furty, Ghazzy, Octoxy, Palsteron, Coconutmage, Darthmicrotransaction, Dslily, Fubgun, Kobeblaubeere, Kripparrian, Ruetoo, Zizaran) |
| `poe2/` | 7 | 00_version_pinning, 01_spirit_economy, 02_weapon_sets, 03_trials_sekhemas_chaos, 04_runes_soul_cores_talismans, 05_atlas_mechanics_05, 06_runes_of_aldur |
| `thresholds/` | 3 | red_maps, pinnacle, uber_pinnacle |
| `maxroll/` | 5 | 00_README + poe1_bosses, poe1_crafting, poe1_currency, poe1_getting_started |
| **Total** | **69** | |

Compile-time inclusion via `include_str!` so the binary always ships
with a complete fallback; wiping `~/.bestel/prompts/` cannot brick the
agent.

`crates/bestel-core/src/llm/local_addendum.md` lives **inside the
source tree, next to the LLM code**, not under `prompts/`. It carries
the Ollama-specific tool restrictions seeded to
`~/.bestel/prompts/local_addendum.md` on first launch.

---

## Runtime resolution

Every chat turn:

1. `prompts::load_anthropic_blocks(provider, model_id)` reads
   `SYSTEM_PROMPT.md` + `CORE_KNOWLEDGE.md` from `~/.bestel/prompts/`.
2. On any IO error, the bundled string wins.
3. `SYSTEM_PROMPT.md` is split on the literal `<tool_policy>` marker
   into BP2 (persona + runtime contract) and BP3 (tool policy + answer
   mode router + contracts + failure policy).
4. Provider + model overrides are appended to BP3 (see
   [Override hierarchy](#override-hierarchy)).
5. `CORE_KNOWLEDGE.md` becomes BP4. The compile-time skill descriptions
   block (`crate::skills::descriptions_block()`) is appended so the
   agent sees skill names + when-to-use hints without paying the cost
   of every skill body (load on demand via `load_skill`).
6. Each of BP2 / BP3 / BP4 gets `cache_control: ephemeral, ttl: 1h`
   for Anthropic prompt caching.

References are **NOT concatenated** into the system prompt. The agent
fetches each one on demand via `read_internal_reference(rel_path)`.
This keeps the system prompt small (~10–15 KB) and lets the model only
pay the token cost of references it actually needs.

---

## Override hierarchy

Three scopes, in order of specificity:

| Scope | Path | Loaded when |
|---|---|---|
| Global | `~/.bestel/prompts/SYSTEM_PROMPT.md` + `CORE_KNOWLEDGE.md` | Always |
| Per-provider | `~/.bestel/prompts/providers/<provider>.md` | Active profile's underlying `Provider` matches (e.g. `anthropic.md`, `ollama.md`) |
| Per-model | `~/.bestel/prompts/models/<sanitized_model_id>.md` | Active model id matches exactly |

`sanitize_model_id` lowercases and replaces `/`, `:`, `\` with `-` so
`anthropic/claude-haiku-4-5` becomes `anthropic-claude-haiku-4-5.md`.

Provider + model overrides ride along **inside BP3** so they share its
cache breakpoint. Picking a different model invalidates only BP3 — BP2
and BP4 stay warm.

Each override is separated from the base prompt with `\n\n---\n\n` so
the agent sees clear scope boundaries.

---

## `read_internal_reference` — strict enum

The schema for `rel_path` in the tool's JSON Schema is an `enum` listing
every key in `BUNDLED_REFERENCES`. The Anthropic API validates the
argument before the model can submit a `tool_use` event with a path
that doesn't exist. This eliminates an entire class of hallucination
where the agent invents filenames (the canonical bug: Haiku invented
`09_build_archetypes.md` when the real file is
`17_build_archetype_taxonomy.md`).

If the user has deleted a file from `~/.bestel/prompts/references/`,
`read_reference(rel_path)` falls back to the bundled string. So the
strict enum AND the runtime fallback both protect against missing
references.

On the rare path where the agent invents a path *not in the enum*
(possible only if the model bypasses validation), the error message
returns the top-5 token-overlap-ranked closest matches so the agent
can self-correct on its next iteration.

### Adding a reference

1. Drop the file under `prompts/references/<rel_path>`.
2. Add a tuple `("<rel_path>", include_str!("../../../prompts/references/<rel_path>"))`
   to `BUNDLED_REFERENCES` in `crates/bestel-core/src/prompts.rs`.
3. Rebuild — the JSON Schema enum regenerates from
   `BUNDLED_REFERENCES.iter().map(|(k, _)| k)`.

That's it. The agent sees the new path on the next turn, and the
`read_internal_reference` tool can serve it both from disk and from
the bundle.

---

## LanceDB KB

The reference library is also indexed semantically.
`crates/bestel-core/src/llm/kb.rs` boots an embedded
[LanceDB](https://lancedb.github.io/) instance at
`~/.bestel/index/kb.lance/` and runs a one-shot ingest of every
`.md` file under `~/.bestel/prompts/references/`.

### Backend

The crate `bestel-rag` (workspace-internal) provides:

- `KbEngine` — orchestrator
- `LanceIndex` — LanceDB wrapper with vector + full-text-search columns
- `EmbeddingProvider` trait — auto-detected via `detect_default(http)`:
  - **OpenAI** when `OPENAI_API_KEY` is set
    (`text-embedding-3-small` / `-large`)
  - **Ollama** when an Ollama daemon is reachable
    (`nomic-embed-text`, `mxbai-embed-large`, etc.)
  - Else `None` — KB is disabled, `kb_search` returns `"kb_not_ready"`,
    the verifier falls through to `unverified` for every claim.

### Ingest

- Triggered on app boot via `kb::bootstrap_global()` (best-effort, no
  user-facing error if no embedder is reachable).
- Background `tokio::spawn` walks the references corpus, chunks each
  file, computes embeddings, upserts into LanceDB.
- Blake3 file-hashes per chunk are stored in the SQLite
  `kb_versions` table (`upsert_kb_version`) so a re-ingest skips
  unchanged files.
- `ensure_fts_index()` builds the BM25 column.

### Retrieval

`KbEngine::search(query, top_k, filters, must_have_terms)` runs a
**hybrid** retrieval:

- Embed the query through the active backend.
- LanceDB vector search → top N semantic candidates.
- BM25 full-text search → top N lexical candidates.
- Rank-fuse with reciprocal rank.
- Return top `top_k` with chunk body, source path, score.

Surface tools:

- `kb_search { query, top_k, filters }` — the agent's primary RAG
  surface. **Capped at 3 calls per turn** (`KB_SEARCH_TURN_CAP = 3`);
  beyond that, dispatch returns `"tool_storm_kb_search"` so the model
  stops digging.
- The CoVe verifier (see [02 § Verifier](./02_agentic_system.md#verifier-cove))
  uses the same `kb.search` directly per claim with
  `KB_TOP_K_PER_CLAIM = 3` — bypassing the tool surface because it
  runs server-side after the user-facing answer drafts.

### Eval

`bestel kb-eval --queries <toml> [--top-k N] [--filter-id …] [--out …]`
runs the LanceDB retrieval against a curated set of queries with
expected source paths, reports recall@k. The eval set lives at
`tests/eval/kb_queries.toml`.

---

## `load_skill`

A second per-turn-capped tool, distinct from `kb_search`:

- Lists Sprint F **skills** (curated playbooks, not LanceDB chunks)
  living under `crates/bestel-core/src/skills/`.
- The agent sees skill *names + descriptions* in BP4 of the system
  prompt (via `crate::skills::descriptions_block()`).
- It calls `load_skill { name }` to pull the full skill body into the
  current turn's context.
- **Capped at 2 calls per turn** (`LOAD_SKILL_TURN_CAP = 2`) to guard
  against skill-load thrash; the cap rises to 2 only when the user
  explicitly requested a multi-skill audit.

Skills are *playbooks* (long-form, structured, hand-written), versus
references which are *encyclopedic* (atomic facts, indexed).

---

## Source allowlist and blocklist

The `web_fetch` tool enforces a two-list policy declared in
`crates/bestel-core/src/llm/tools.rs`:

### Blocklist (10 hosts)

Rejected outright before any HTTP call. SEO PoE blog farms that
recycle patch announcements without editorial integrity:

`aoeah.com`, `mmogah.com`, `iggm.com`, `ggwtb.com`, `boostmatch.com`,
`sportskeeda.com`, `gamewatcher.com`, `switchbladegaming.com`,
`dotesports.com`, `gamerant.com`.

### Allowlist (21 hosts, 7 tiers)

Mirrors `prompts/references/15_source_registry.md`. Subdomains match
too (e.g. `forum.pathofexile.com` matches `pathofexile.com`).

| Tier | Hosts | Use for |
|---|---|---|
| 1 — Canonical | `pathofexile.com`, `pathofexile2.com` | Patch notes, trade |
| 2 — Wikis | `poewiki.net`, `poe2wiki.net` | Mechanics, items, skills |
| 3 — Datamined | `poedb.tw`, `poe2db.tw`, `repoe-fork.github.io` | Mods, tags, spawn weights |
| 4 — Calculators | `pathofbuilding.community`, `craftofexile.com` | Build math, crafting odds |
| 5 — Economy | `poe.ninja` | Price trends, popular builds |
| 6 — Filters | `filterblade.xyz` | Loot filters |
| 7 — Creators | `maxroll.gg`, `mobalytics.gg`, `pohx.net`, `poe-vault.com`, `poeplanner.com`, `pathofpathing.com`, `poelab.com`, `heartofphos.github.io`, `poe.re`, `exile.re` | Secondary guides — only when patch number / author / date is shown |

Wiki tools (`wiki_search`, `wiki_parse`, `wiki_synergies`,
`wiki_cargo`) bypass `web_fetch` entirely — they hit the MediaWiki API
directly through `crates/bestel-core/src/sources/wiki.rs`.

---

## Bundled snapshots

Two additional sources are shipped *bundled* (zstd-compressed,
`include_bytes!`) so the agent has an offline-first path that doesn't
hit the network or care about rate limits.

| Snapshot | Game | Files | Compressed size |
|---|---|---|---|
| repoe-fork | PoE1 | 7 (mods, gems, base_items, stat_translations, uniques, essences, fossils, cluster_jewels) | ~2.05 MB |
| repoe-fork | PoE2 | 5 (mods, base_items, uniques + 2 placeholder 11-byte files for the 404-on-snapshot-day mods) | ~455 KB |
| trade catalogue | PoE1 | `poe1.json.zst` (stat ids → labels) | 187 KB |
| trade catalogue | PoE2 | `poe2.json.zst` | 59 KB |

Total bundled snapshots: **~2.75 MB compressed**, ~39 MB decompressed.

Path: `crates/bestel-core/src/sources/snapshots/{poe1,poe2}/*.zst` plus
`trade_catalogue` files. Version stamp:
`crates/bestel-core/src/sources/snapshots/version.txt`.

Refresh: `scripts/refresh-snapshots.ps1` re-pulls the source files and
recompresses. Run on patch days or whenever drift is suspected (the
2026-05-08 refresh caught two 11-byte placeholder files from upstream
404s — these will go away once repoe-fork ships full PoE2 parity).

---

## Editing prompts at runtime

The `prompt-editor` window (`Ctrl+Shift+P` or `prompts_open_editor`)
gives the user a CodeMirror surface over `~/.bestel/prompts/`. Edits
hit disk; the next chat turn picks them up immediately because the
loader re-reads per turn.

The IPC surface (`prompts_list`, `prompts_read`, `prompts_write`,
`prompts_reset`, `prompts_reset_all`) is symmetric: anything the editor
can do, the CLI / external agent can do too via IPC.

`prompts_reset` swaps a shipped file back to its bundled default.
`prompts_reset_all` wipes user edits across `SYSTEM_PROMPT.md`,
`CORE_KNOWLEDGE.md`, and `local_addendum.md`. References are reset by
deleting them — `seed_references_if_missing` re-creates them from the
bundle on next launch.

---

## See also

- [02 — Agentic system](./02_agentic_system.md) — how the cached
  blocks are wired into the request and when the verifier fires.
- [04 — Tools catalogue](./04_tools_catalogue.md) — full schema for
  `read_internal_reference`, `kb_search`, `load_skill`, `web_fetch`,
  `wiki_*`.
- [07 — Persistence](./07_persistence_and_storage.md) — what
  `~/.bestel/prompts/`, `~/.bestel/index/kb.lance/`, and the SQLite
  `kb_versions` table contain.
