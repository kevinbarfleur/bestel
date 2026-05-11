# 07 — Persistence and storage

Everything Bestel writes lives under `~/.bestel/`. Five storage
technologies coexist: plain JSON, plain Markdown, SQLite, LanceDB, and
zstd-compressed JSON snapshots. No single global state server — each
subsystem owns its slice of the home dir.

---

## `~/.bestel/` map

```
~/.bestel/
├── prompts/                              # user-editable mirror of the bundled prompts
│   ├── SYSTEM_PROMPT.md
│   ├── CORE_KNOWLEDGE.md
│   ├── local_addendum.md
│   ├── references/                       # 69 .md files (see [03])
│   ├── providers/<name>.md               # optional per-provider override
│   └── models/<sanitized_id>.md          # optional per-model override
│
├── runtime/                              # short-lived state + DB
│   ├── keys.json                         # whitelisted API keys (plaintext)
│   ├── model.json                        # active profile id
│   ├── settings.json                     # app-level toggles (verifier enabled, etc.)
│   ├── debug-chats/<id>.json             # PersistedRun (one per turn)
│   ├── bestel.sqlite3                    # SQLite mirror — see § SQLite
│   └── logs/bestel.log                   # rolling-daily tracing log (when BESTEL_FILE_LOG=1)
│
├── index/
│   └── kb.lance/                         # LanceDB vector + FTS index
│       ├── <table>.lance/                # one Arrow file per chunk batch
│       └── _versions/
│
├── cache/                                # HTTP / snapshot caches (zstd JSON)
│   ├── <sha256>.json                     # generic FileCache entries
│   ├── repoe/{poe1,poe2}/<category>.json.zst        # refreshed snapshots override bundled
│   ├── trade_stats/{poe1,poe2}.json.zst             # refreshed trade catalogue
│   └── wiki/                              # wiki page bodies (cached 12h)
│
├── notes/<session_id>.json               # session_notes: per-session scratchpad
│
├── skills/<skill_name>/                  # bundled skills mirrored to disk
│   ├── SKILL.md
│   └── templates/*.md
│
├── exports/runs-export-<stamp>.json      # dev panel export bundle
│
└── logs/session-YYYYMMDD-HHMMSS.jsonl    # devlog (one per process)
```

`dirs::home_dir()` resolves the root on Windows / macOS / Linux.
Every writer guards with `home_dir()?` and falls through gracefully
when it's not resolvable (rare).

---

## JSON files

| Path | Writer | Read by | Shape |
|---|---|---|---|
| `runtime/keys.json` | `llm/keys.rs` | `apply_to_env` at boot, `list_api_keys`, `set_api_key`, `delete_api_key` | `{ "ANTHROPIC_API_KEY": "…", "DEEPSEEK_API_KEY": "…" }` — strict whitelist |
| `runtime/model.json` | `llm/models.rs::save_active_profile` | `active_profile()` at boot, `get_active_model`, `set_active_model` | `{ "profile_id": "anthropic-haiku-4-5" }` |
| `runtime/settings.json` | `crate::settings` | Toggle reads / writes (e.g. `is_verify_enabled`) | `{ "verifier_enabled": true, … }` |
| `runtime/debug-chats/<id>.json` | `llm/recorder.rs::save` (tempfile + rename) | Dev panel `list_debug_runs` / `get_debug_run`, `bestel eval-judge`, `bestel run-battery` | `PersistedRun` (see [02 § Recorder](./02_agentic_system.md#recorder)) |
| `notes/<session_id>.json` | `llm/session_notes.rs` | `tools.rs` (some tools embed `summary.session_notes` for the agent) | `SessionNotes` — append-only scratchpad |
| `exports/runs-export-<stamp>.json` | `dev_export_all_runs` IPC | External reviewers | Envelope reshaping every `PersistedRun` into `{ question, answer, reasoning, tools[], stats, error }`, tool outputs capped at 4 KB |

`keys.json` is **plaintext**. Bestel is single-user and Windows-first;
key encryption would require a keychain integration that is out of
scope for now. The risk is local-only; the file is `0600`-style on
Linux but not encrypted on Windows.

---

## SQLite

File: `~/.bestel/runtime/bestel.sqlite3` (note: `.sqlite3`, not `.db`).

`crates/bestel-core/src/persistence/mod.rs` is the entry point. Uses
`rusqlite` 0.32 with `bundled` feature — no system SQLite required.

### Opening

```rust
Db::open(path) →
    open + create + URI
    pragma journal_mode = WAL
    pragma synchronous = NORMAL
    pragma foreign_keys = ON
    run pending migrations (PRAGMA user_version)
```

Connections are wrapped in `Arc<Mutex<Connection>>` — one shared
writer + readers fine for the dev-panel access pattern (single user,
no concurrency).

### Tables (schema v1)

| Table | Module | Purpose |
|---|---|---|
| `runs` | `persistence/runs.rs` | One row per persisted chat turn — mirror of `PersistedRun` for queryability (provider, model_id, started_at, final_text_excerpt, verifier_status, tool_call_count, etc.) |
| `lint_findings` | `persistence/lint.rs` | One row per lint failure produced by the Sprint A linter (`bestel lint-debug-run` / `lint_debug_run` IPC) |
| `verifier_results` | `persistence/verifier_results.rs` | One row per CoVe verdict (status, claims_checked count, corrections, latency) |
| `sessions` | `persistence/sessions.rs` | Mirror of session_notes JSON for queryability |
| `builds_snapshot` | (legacy) | Point-in-time PoB snapshot keyed by `build_id` |
| `kb_versions` | `persistence/kb_versions.rs` | BLAKE3 fingerprint per indexed doc; lets the LanceDB ingest skip unchanged files |
| `build_sheets` | `sheets/store.rs` | Validated Build Sheets — see [06](./06_build_sheet_workflow.md) |

### Migrations

`persistence/migrations.rs` carries an append-only `MIGRATIONS`
vector. Each entry is a (version, SQL) pair. `migrate(conn)` reads
`PRAGMA user_version`, applies every migration with a higher version,
and bumps the pragma.

New migrations append; old DBs catch up on next open. No down
migrations.

### Inspection

`bestel db [subcommand]` exposes the SQLite store on the CLI:

| Subcommand | Effect |
|---|---|
| `bestel db migrate` | Apply pending migrations against the live file. Idempotent |
| `bestel db stats` | Print per-table row counts |
| `bestel db import <runs-dir>` | Import a directory of `PersistedRun` JSONs into the `runs` table |
| `bestel db query <sql>` | Run an arbitrary SELECT — for triage |
| `bestel db bench` | Measure insert / query latency |

The JSON layer remains authoritative for reads (dev panel reads
`debug-chats/<id>.json`, lint sidecar reads `runs.json`, eval-judge
reads JSON). SQLite is for ad-hoc querying:

```sql
SELECT id FROM runs
 WHERE provider = 'anthropic'
   AND model_id LIKE '%haiku%'
   AND verifier_status = 'fail';
```

---

## LanceDB

Path: `~/.bestel/index/kb.lance/`.

Holds vector embeddings + BM25 full-text-search columns for the
reference library. See [03 § LanceDB KB](./03_knowledge_layer_and_rag.md#lancedb-kb)
for the boot flow, the embedding backend detection, and the
`kb_search` retrieval surface.

Storage shape (LanceDB internals, simplified):

- `<table>.lance/` — Arrow file format, one segment per ingest batch.
- `_versions/` — version directory; LanceDB supports time-travel
  reads.
- BM25 columns built via `KbEngine::ensure_fts_index()` on first boot.

The `kb_versions` SQLite table holds `(rel_path, file_hash, ingested_at)`
so re-ingest skips files whose BLAKE3 hash hasn't changed.

---

## HTTP cache

`crates/bestel-core/src/sources/cache.rs` provides `FileCache`. Stores
one JSON file per cache key under `~/.bestel/cache/<sha256>.json`:

```json
{ "stamp": 1715515200, "data": <T> }
```

- `stamp` is the Unix timestamp at write.
- `data` is the typed payload (wiki page, repoe lookup, trade query).
- TTL is **caller-defined** — the cache returns `None` if `now -
  stamp > ttl`. The cache itself never deletes entries; stale rows
  are simply ignored.
- `FileCache::default_dir()` is `~/.bestel/cache/`.
- Cache misses never error: a corrupted file or unreadable path
  silently falls through to a fresh fetch.

Sub-directories:

- `cache/wiki/` — per-page wiki bodies (TTL 12h, used by
  `wiki_parse`).
- `cache/repoe/{poe1,poe2}/<category>.json.zst` — refreshed repoe
  snapshots. The runtime loader prefers a refreshed file over the
  bundled one if present.
- `cache/trade_stats/{poe1,poe2}.json.zst` — same pattern for the
  trade catalogue.

Refresh tooling: `scripts/refresh-snapshots.ps1` re-pulls the source
JSON, recompresses, and writes into `cache/`. No CI cadence — run on
patch days or when the bundled version stamp drifts.

---

## Bundled snapshots

Compile-time `include_bytes!` under
`crates/bestel-core/src/sources/snapshots/`. The runtime loader (`repoe.rs`,
`trade_catalogue.rs`) prefers a refreshed file from `~/.bestel/cache/`
when present, else falls back to the bundled bytes.

Version stamp: `crates/bestel-core/src/sources/snapshots/version.txt`
carries the ISO-8601 timestamp of the last bundle refresh.

### PoE1 (8 files, ~2.05 MB compressed)

| File | Size | Decompressed (approx) |
|---|---|---|
| `mods.json.zst` | 910,774 | ~12 MB |
| `gems.json.zst` | 662,761 | ~7 MB |
| `stat_translations.json.zst` | 335,716 | ~4 MB |
| `base_items.json.zst` | 130,144 | ~1.5 MB |
| `uniques.json.zst` | 39,051 | ~450 KB |
| `essences.json.zst` | 8,446 | ~100 KB |
| `fossils.json.zst` | 6,441 | ~80 KB |
| `cluster_jewels.json.zst` | 2,043 | ~30 KB |

### PoE2 (5 files, ~455 KB compressed)

| File | Size | Notes |
|---|---|---|
| `mods.json.zst` | 358,818 | ~5 MB decompressed |
| `base_items.json.zst` | 95,336 | ~1 MB |
| `uniques.json.zst` | 10,786 | ~100 KB |
| `gems.json.zst` | 11 | **placeholder** — upstream 404 on 2026-05-08 |
| `stat_translations.json.zst` | 11 | **placeholder** |

The two 11-byte placeholder files keep `include_bytes!` happy until
repoe-fork ships PoE2 parity for those categories. `repoe_lookup`
returns an explicit "not available for poe2" error when queried.

### Trade catalogues

| File | Size |
|---|---|
| `trade_stats/poe1.json.zst` | 190,466 |
| `trade_stats/poe2.json.zst` | 60,145 |

Used by `trade_resolve_stats` to map phrase → stat ID without hitting
`pathofexile.com/api/trade/data/stats`.

### Totals

| Category | Compressed | Decompressed (approx) |
|---|---|---|
| PoE1 repoe | 2.10 MB | ~25 MB |
| PoE2 repoe | 464 KB | ~6 MB |
| Trade catalogues | 245 KB | ~7 MB |
| **Total bundled** | **~2.80 MB** | **~38 MB** |

The decompression cost (one zstd decode + serde parse) is cached in a
process-lifetime `OnceLock` per category, so first call is ~100 ms
and subsequent calls are <10 ms.

---

## Logs

Two distinct log files coexist:

- **`~/.bestel/runtime/logs/bestel.log`** — Rust `tracing` rolling
  daily, only when `BESTEL_FILE_LOG=1`. The only way to capture
  Rust-side logs in release builds, since the GUI subsystem detaches
  stdio. The `bestel-driver` debug harness reads this file.
- **`~/.bestel/logs/session-YYYYMMDD-HHMMSS.jsonl`** — `devlog.rs`
  writes one JSONL file per process. Each line is a structured event
  (provider response, tool result excerpt, error). Used for post-hoc
  inspection without involving the SQLite mirror.

---

## Skills

`~/.bestel/skills/<name>/SKILL.md` mirrors the bundled skills from
`crates/bestel-core/src/skills/`. `seed_skills_if_missing` (similar
in spirit to `seed_references_if_missing`) writes them on first
launch.

Templates under `<name>/templates/*.md` carry the structured
artifacts a skill emits (e.g. a build-review template).

`load_skill` tool reads from `~/.bestel/skills/` first, falls back to
bundled. Same override pattern as the references.

---

## Exports

`dev_export_all_runs` IPC packages every persisted run into a single
`~/.bestel/exports/runs-export-<stamp>.json` envelope. Each run is
reshaped:

```json
{
  "id": "…",
  "question": "<user_text>",
  "answer": "<final_text>",
  "reasoning": "<concatenated reasoning segments>",
  "tools": [
    {
      "name": "wiki_parse",
      "input": "…",
      "output_excerpt": "…",  // capped at 4 KB
      "status": "done"
    }
  ],
  "stats": { … },
  "error": null
}
```

Designed to be pasted into an external expert-agent conversation
without bringing the full repo or 25 MB of raw runs.

---

## What is **not** persisted

To avoid surprises:

- **Chat conversation history.** Only the latest in-memory session is
  in `AppState.inner.history`. Each turn is recorded as a
  `PersistedRun`; the *conversation* across turns is not reassembled
  from disk. Closing the app discards conversation state. The
  separate `chatHistory.ts` Pinia store + `chat_log_persist` /
  `chat_log_dir` IPC handles user-initiated chat exports.
- **Tool *input* arguments.** Recorded in the streamed `ToolDetailUpdate`
  delta and rendered in the dev panel, but not surfaced in the
  exports envelope (only the output excerpt is).
- **Embeddings of historical chats.** Only references are
  embedded in the KB. Chat embeddings are not part of the design — the
  KB is for *retrieval grounding*, not *memory*.

---

## See also

- [02 — Agentic system](./02_agentic_system.md) — what fills
  `runtime/debug-chats/` and the `runs` / `verifier_results` /
  `sessions` tables.
- [03 — Knowledge layer and RAG](./03_knowledge_layer_and_rag.md) —
  what fills `prompts/`, `index/kb.lance/`, and `cache/repoe/`.
- [06 — Build Sheet workflow](./06_build_sheet_workflow.md) — what
  fills the `build_sheets` table.
