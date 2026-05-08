# Bestel — Intelligence Roadmap (v15+)

> Living roadmap. Update the **Status** section as sprints progress.
> Last revision: 2026-05-08.

## Context

Bestel today is a chat agent with 8 tools (build, wiki ×4, trade ×2, web,
reference) backed by 22 bundled reference docs. It can read a PoB XML
file structurally, fetch wiki pages, resolve trade stats, and search the
web — but it cannot **calculate** anything (no DPS, no EHP, no max-hit),
it has no offline data catalogue (every "mod ranges on Stellar Amulet"
query hits the network), and its knowledge base has six load-bearing
gaps (archetype taxonomy, atlas mechanics, animation/movement, basetype
identity, currency taxonomy, validation rules).

The 2026-05-08 research report identified ten high-leverage improvements.
This roadmap groups them into six sprints, locked by user decisions on
2026-05-08:

1. Knowledge-base + validation first (text-heavy, fast wins, all editable
   via the v13 prompt-editor).
2. Data layer (repoe-fork JSON + trade-stats catalogue) — hybrid bundled
   snapshot + nightly refresh.
3. PoB calculation engine — LuaJIT sidecar bundled via Tauri externalBin.
4. PoB semantic-facts extractor.
5. Wiki SQLite mirror.
6. PoE2 0.5 absorption (release 2026-05-29 — scaffold now, fill on
   launch).

Outcome: Bestel goes from "veteran-flavoured chat" to "veteran with a
calculator and a curated database". The prompt-editor stays the single
authoring surface for everything text — agents and humans both edit the
same files under `~/.bestel/prompts/`.

## Architecture decisions (locked 2026-05-08)

- **Doc surface**: every reference, glossary, threshold table, creator
  profile, validation rule lives under `prompts/references/` and is bundled
  via `BUNDLED_REFERENCES` in `crates/bestel-core/src/prompts.rs`. Adding
  a doc = one line in the array + `cargo build --release`. End-users
  edit copies under `~/.bestel/prompts/references/` via the existing
  prompt-editor window.
- **Tooling expansion**: every new tool (`repoe_lookup`, `pob_calc`,
  `wiki_sqlite`) follows the existing pattern in
  `crates/bestel-core/src/llm/tools.rs` — constant + schema in
  `tool_schemas()` + match arm in `dispatch()` + 25–30 KB truncation.
  Local-friendly subset selectively gated (≤ 8 B models stay simple).
- **Data layer strategy**: hybrid `include_bytes!` snapshot (zstd-19) +
  background nightly refresh task on startup. Atomic rename on disk; the
  agent never sees a half-written file. Bundled snapshot is the
  offline-first fallback.
- **PoB engine strategy**: LuaJIT sidecar binary bundled via
  `tauri.conf.json` `bundle.externalBin`. Vendored
  PathOfBuildingCommunity + PathOfBuilding-PoE2 sources as git submodules
  pinned to release tags. Lazy spawn on first `pob_calc` call.
  `tauri-plugin-shell` added. In-process `mlua` rejected (panic
  isolation, MSVC LuaJIT pain on Windows).
- **PoE2 0.5 (2026-05-29)**: scaffold `references/poe2/` subdirectory in
  Sprint 0 with stub files + `00_version_pinning.md`. Fill content on
  launch day — 2-day update instead of 2-week scramble.
- **Source-quality enforcement**: `web_fetch` gains a hard tier-4 ban
  list (aoeah, mmogah, iggm, ggwtb, boostmatch, sportskeeda, gamewatcher,
  switchbladegaming, dotesports). Tier-1/2 only.
- **Multilingual**: agent reasons in English internally, translates to
  the user's language at output. Glossary FR↔EN bundled.

## Sprint 0 — Knowledge base + validation hardening (~2–3 weeks)

### Scope

Pure text + small `web_fetch` change. No new tools, no new crates, no
network code. Highest agent-quality lift per dev-day.

**New reference docs** under `prompts/references/`:

- `17_build_archetype_taxonomy.md` — formal taxonomy (hit/crit, hit/non-
  crit, ailment-stack, DoT, totem, mine, trap, minion, trigger,
  self-cast, channelling, autobomber, summoner-of-summoners). PoE2:
  combo-builder, companion-pet, herald-stack, weapon-set swap. Per
  archetype: identifying tags, scaling levers, defensive defaults,
  common pitfalls, diagnostic checklist.
- `18_atlas_and_endgame_mechanics.md` — separate from league mechanics.
  PoE1: tree, sextants→tablets, Maven, Sanctum, Heist, Delve, Ritual,
  Expedition, Settlers. PoE2: Atlas + Waystones + Towers + Tablets +
  Citadels + Corrupted Nexus + Pinnacle bosses. Pre/post-0.5 markers.
- `19_combat_movement_animation.md` — action speed, attack/cast speed,
  animation cancelling, dodge-roll-cancel, weapon-swap timings,
  leap-slam cancel, totem placement speed.
- `20_item_basetype_identity.md` — why specific bases matter (Spine
  Bow, Eternal Burgonet, Sorcerer Boots/Two-Toned, Convoking Wand,
  Stellar Amulet). PoE2: Expert bases, base implicits.
- `21_currency_and_barter_taxonomy.md` — orb categories, bulk
  fragments, splinter shards, scarab/sextant ecosystem. PoE2 narrower
  currency tree, runes, Verisium (0.5).
- `22_trade_etiquette_and_scams.md` — whisper templates, courtesy
  windows, common scams, live-search expectations, hideout afk,
  party-kick.
- `23_hardcore_softcore_ssf_mode_differences.md` — when advice
  diverges. HC = max-hit > DPS. SSF = self-found viability. Trade
  league = trade is build power.
- `24_patch_history_meta.md` — short conceptual entries on historical
  reworks. Helps temporal reasoning.
- `25_pob_engine_integration.md` — stub doc filled in Sprint 2.
- `26_validation_and_self_correction.md` — extension of doc 14.
  Engine-trust rule, live-vs-cached check, patch-version awareness,
  PoE1↔PoE2 disambiguation rule, self-consistency loop.

**Reorganisation**:

- `creators_registry/` subdirectory — split out doc 16 personalities to
  per-file profiles. PoE1: `mathil`, `zizaran`, `palsteron`, `pohx`,
  `subtractem`, `kobeblaubeere`, `ben_`, `goratha`, `ghazzy`, `furty`,
  `tripolarbear`, `quin69`, `octoxy`. PoE2: `ziggyd`,
  `darthmicrotransaction`, `kripparrian`, `dslily`, `coconutmage`. Doc
  16 becomes a registry index.

**PoE2 scaffolding** under `references/poe2/`:

- `00_version_pinning.md` — current PoE2 version + "facts as of 0.X"
  annotations.
- `01_spirit_economy.md` — Spirit budget, breakpoints, mana sustain.
- `02_weapon_sets.md` — Weapon Set 1/2 mechanics, set passives.
- `03_trials_sekhemas_chaos.md` — ascendancy mechanics replacing Lab.
- `04_runes_soul_cores_talismans.md` — itemization extras.
- `05_atlas_mechanics_05.md` — empty stub, filled 2026-05-29.
- `06_runes_of_aldur.md` — empty stub, filled 2026-05-29.

**Glossary** under `references/glossary/`:

- `fr_en.md` — top 200 PoE skill/item/mechanic names FR↔EN.

**Threshold tables** under `references/thresholds/`:

- `red_maps.md`, `pinnacle.md`, `uber_pinnacle.md` — version-pinned
  numerical bars (DPS, EHP, max-hit, recovery), separately for hit-
  based and DoT.

**Updates to existing**:

- `14_validation_and_failure_modes.md` — add engine-trust + staleness
  + patch-version-awareness rules.
- `15_source_registry.md` — explicit tier-4 SEO blog ban with named
  blocklist.
- `13_retrieval_playbooks.md` — add patterns for tools coming in
  Sprint 1+.

**Code change** (small):

- `crates/bestel-core/src/llm/tools.rs` `web_fetch` allowlist — add a
  hard `BLOCKED_DOMAINS` list checked before the tier-1/2 allowlist.
  Reject with explicit error message naming the rule.

### Verification (Sprint 0)

| ID | Setup | Expected |
|---|---|---|
| S0-A | First launch on existing `~/.bestel/prompts/` | New refs auto-seeded; existing files untouched. |
| S0-B | Open prompt-editor, browse new docs | All 30+ new files visible with correct group categorisation. |
| S0-C | Ask agent: "what archetype is RF/Juggernaut?" | Agent fetches `17_build_archetype_taxonomy.md`. |
| S0-D | Ask agent: "give me a Mageblood build from aoeah.com" | `web_fetch` rejects "blocked tier-4 source". |
| S0-E | Ask agent in French: "comment fonctionne la Brûlure ?" | Agent translates via glossary, answers about Ignite. |
| S0-F | `cargo build --release -p bestel` | Bundle size grows by ≤ 500 KB (text only). |

## Sprint 1 — Data layer (repoe-fork + trade-stats) (~1–2 weeks)

### Scope

New module `crates/bestel-core/src/sources/repoe.rs` +
`repoe_refresh.rs` + `trade_catalogue.rs`. New tool `repoe_lookup`.
Refactor existing `trade_resolve_stats` to consume the catalogue (no
schema change for the LLM).

### Bundling strategy

- `crates/bestel-core/src/sources/snapshots/{poe1,poe2}/<category>.json.zst`
- Compile-time include via `include_bytes!`
- Decompressed lazily via `zstd::decode_all`
- Per-category `OnceCell<HashMap<String, usize>>` index
- Memory budget ≈ 15 MB resident per game after warmup

### Categories bundled

`mods`, `base_items`, `gems`, `uniques`, `cluster_jewels`, `essences`,
`fossils`, `stat_translations` × `{poe1, poe2}` — total ~10 MB
compressed added to binary.

### Refresh task

- Spawned 10 s after boot
- HEAD `https://repoe-fork.github.io/version.txt`
- Atomic write under `~/.bestel/cache/repoe/{game}/<cat>.json.zst`
- Daily re-check while app is open
- Lookup precedence: in-memory → on-disk refreshed → bundled fallback
- Mid-flight refresh invisible (`Arc<DashMap>` atomic flip per
  category)

### Verification (Sprint 1)

| ID | Setup | Expected |
|---|---|---|
| S1-A | `cargo test -p bestel-core repoe::*` | All pass. |
| S1-B | First `repoe_lookup` cold call | < 300 ms wall. |
| S1-C | Ask agent: "what mods can roll on Stellar Amulet?" | Returns full mod pool offline. |
| S1-D | Network unplugged | All 8 categories resolvable from bundled. |
| S1-E | Bundled `version.txt` bumped | Atomic flip on next launch. |
| S1-F | `trade_resolve_stats` regression | All existing tests pass. |
| S1-G | Bundle size | `bestel.exe` grows by ≤ 12 MB. |

### Open questions (Sprint 1)

1. Does repoe-fork expose a stable `version.txt` URL, or do we scrape
   GitHub Actions artifacts?
2. PoE2 dataset parity per category — per-category "available" matrix
   needed?

## Sprint 2 — PoB engine headless (~3–4 weeks)

### Scope

New crate `crates/bestel-pob-engine/` (Rust wrapper). Bundled LuaJIT
binary per platform. Vendored PathOfBuildingCommunity +
PathOfBuilding-PoE2 as git submodules. New tool `pob_calc`.

### Layout

```
crates/bestel-pob-engine/
  src/lib.rs                    PobEngine handle
  src/protocol.rs               typed Cmd / Reply
  src/lifecycle.rs              health, restart, idle timeout
  vendor/
    PathOfBuildingCommunity/    submodule, pinned tag
    PathOfBuilding-PoE2/        submodule, pinned tag
    api-stdio.lua               headless harness (forked from ianderse/pob-mcp)
external/luajit/
  windows-x86_64/luajit.exe
  macos-aarch64/luajit
  macos-x86_64/luajit
  linux-x86_64/luajit
```

### Sidecar protocol (v1)

5 commands. Newline-delimited JSON over stdio:

```
load_build         → load XML, return build_meta
get_stats          → category + optional skill_index, return canonical output keys + calcs config
set_calc_input     → toggle Calcs config (enemyIsBoss etc.)
select_main_skill  → switch active skill group
ping               → health check + pob_version + game
```

### Lifecycle

- **Lazy spawn** on first `pob_calc` call.
- 10-min idle timeout, graceful shutdown, transparent re-spawn.
- Crash recovery: 3 restarts in 5 min, then circuit-break.
- Single-flight per game.
- 8 s per-command timeout; `kill_on_drop` on hang.
- Length-prefixed stdio frames.

### New tool `pob_calc`

```json
{
  "name": "pob_calc",
  "input_schema": {
    "properties": {
      "category": { "enum": ["offence", "defence", "charges",
        "reservation", "ailments", "all"] },
      "skill_index": { "type": "integer" }
    },
    "required": ["category"]
  }
}
```

Output: `{"stats": {...}, "calcs": {...}, "warnings": [...]}`.
`calcs` echo is mandatory — agent flags assumption mismatches.

Must-have output keys (subset of canonical PoB `output` table):

- **offence**: `TotalDPS`, `CombinedDPS`, `FullDPS`, `AverageHit`,
  `Speed`, `HitChance`, `CritChance`, `CritMultiplier`, `IgniteDPS`,
  `BleedDPS`, `PoisonDPS`, `TotalDot`
- **defence**: `EHP`, `LifeUnreserved`, `LifeRecoverable`,
  `EnergyShield`, `Mana`, `Armour`, `Evasion`, `MeleeEvadeChance`,
  `PhysicalDamageReduction`, `MaxHitFire`, `MaxHitCold`,
  `MaxHitLightning`, `MaxHitPhysical`, `MaxHitChaos`, `BlockChance`,
  `SpellBlockChance`, `SpellSuppressionChance`
- **charges**: `PowerChargesMax`, `FrenzyChargesMax`,
  `EnduranceChargesMax`
- **reservation**: `LifeReserved`, `ManaReserved`,
  `LifeReservedPercent`, `ManaReservedPercent`, `SpiritReserved`
- **ailments**: `EnemyShockChance`, `ShockEffect`, `EnemyChillEffect`,
  `EnemyFreezeChance`, `IgniteChance`

### Bundle integration

- Add `tauri-plugin-shell = "2"` to `crates/bestel/Cargo.toml`.
- Add `shell:allow-execute`, `shell:allow-spawn`, `shell:allow-kill`
  to `crates/bestel/capabilities/default.json`.
- `tauri.conf.json` → `bundle.externalBin: ["external/luajit/luajit"]`.
- LuaJIT pre-built binaries vendored (~600 KB per target).

### Verification (Sprint 2)

| ID | Setup | Expected |
|---|---|---|
| S2-A | `cargo test -p bestel-pob-engine spawn_idle_shutdown` | Passes. |
| S2-B | `pob_calc` cold call | < 1.5 s; second call < 100 ms. |
| S2-C | Lua panic injection | Engine restarts within 2 s. |
| S2-D | `while true do end` injection | 8 s timeout, restart. |
| S2-E | Calcs echo | Toggle `enemyIsBoss`, response reflects. |
| S2-F | Bundle size | Windows installer grows ≤ 80 MB. |
| S2-G | Cross-platform smoke | Works on Windows, macOS arm64, Linux x86_64. |
| S2-H | Antivirus | VirusTotal scan unsigned `luajit.exe`; document; plan signing. |
| S2-I | Calcs mismatch detection | Build with `enemyIsBoss=false`, agent flags. |

### Open questions (Sprint 2)

1. Canonical PoE2 PoB fork at submodule pin time?
2. Does `ianderse/pob-mcp` `api-stdio` already implement
   `set_calc_input`?
3. Windows EV cert availability for code-signing `luajit.exe`?
4. GitHub Actions sufficient for macOS arm64 LuaJIT build?

## Sprint 3 — PoB semantic-facts extractor (~1–2 weeks)

### Scope

Extend `crates/bestel-core/src/pob/parser.rs` with semantic tagging on
top of structural parsing. No new tool — enriches `get_active_build`
output.

- **Archetype tagger**: identity flags
  `{life|ES|LL|CI|MoM|hybrid|RF|VLS|trigger-bot}`. Cross-references gem
  tags + items + tree.
- **Build-defining uniques registry**: curated list (Mageblood, HH,
  Original Sin, Cospri, Replica Soul Tether, Replica Restless Ward,
  etc.).
- **"Engine vs filler" classifier**: uses Sprint-2 PoB engine to
  compute counterfactual DPS/EHP without each unique.
- **Conversion chain string**: surfaced verbatim from
  `breakdown.dmg`.

### Optional (stretch)

Extract enriched parser to standalone `pob-parse` crate, publish on
crates.io. Open-source contribution opportunity (no Rust crate of this
kind exists today).

## Sprint 4 — Wiki SQLite mirror (~1 week)

### Scope

New module `crates/bestel-core/src/sources/wiki_sqlite.rs`. Add
`rusqlite` workspace dep. Local SQLite at
`~/.bestel/cache/wiki.sqlite`. Tables mirroring poewiki + poe2wiki
Cargo: `items`, `mods`, `skill`, `skill_levels`, `passive_skill`,
`monsters`. Weekly refresh task. `wiki_cargo` tool queries SQLite first,
falls back to live Cargo on miss.

## Sprint 5 — PoE2 0.5 absorption (2026-05-29)

### Scope

Day-of work, scaffolding done in Sprint 0.

- Re-pull `/api/trade2/data/stats` + `/data/static`.
- Force-refresh repoe-fork PoE2 snapshot.
- Update `references/poe2/00_version_pinning.md` with new version.
- Fill `references/poe2/05_atlas_mechanics_05.md` (Atlas tree rework,
  Verisium, Runic Ward, Alloys).
- Fill `references/poe2/06_runes_of_aldur.md` (league mechanic).
- Update `references/24_patch_history_meta.md`.
- Verify PoB2 engine still runs against new XML schema; possibly bump
  submodule pin.
- Update `references/13_retrieval_playbooks.md` if new query patterns
  emerge.

## Risks

1. **Bundle size** — Sprint 1 ~10 MB, Sprint 2 ~80 MB.
2. **CI complexity** — multi-target LuaJIT build, weekly PoB submodule
   pumps, snapshot refresh job.
3. **Antivirus false positives** — unsigned `luajit.exe`. Mitigate with
   code-signing.
4. **PoB patch lag** — 48 h SLA for submodule bumps.
5. **PoE2 0.5 datamining lag** — wikis lag 1–3 days; prefer repoe-fork
   for first 72 h.
6. **Prompt drift from KB expansion** — `read_internal_reference` is
   fetch-on-demand; LOCAL_TOOL_ALLOWLIST keeps Ollama on a curated
   subset.
7. **SEO blog ban false positives** — per-domain blocklist, easy to
   override.
8. **PoB2 engine completeness** — `pob_calc` returns `warnings` array.
9. **License compliance** — PoB MIT, PoB2 MIT, LuaJIT MIT. Aggregate in
   `LICENSE-third-party.md`.
10. **GGG ToS** — no scraping of undocumented endpoints.

## Out of scope

- ❌ OAuth integration (private stash, private characters).
- ❌ Public stash tab stream consumer.
- ❌ Awakened-PoE-Trade `apt-data` integration.
- ❌ poe.ninja API integration (dedicated tool deferred).
- ❌ Live markdown preview pane in prompt-editor.
- ❌ PoE1 3.X league absorption sprint.
- ❌ Vector index over wiki prose.
- ❌ Build comparison ("which of my 3 PoBs is best for Maven?").

## Status

> ☐ pending · 🟡 in progress · ✅ done

- ✅ **Sprint 0** — Knowledge base + validation hardening
  - notes: Day-1 kickoff done 2026-05-08 (ROADMAP, 30+ stubs, list_files()
    expansion, FETCH_BLOCKLIST, build clean, host_allowed tests pass).
    Content fill complete: docs 14/15/16/17/18/19/20/21/22/23/24/26 +
    thresholds/{red_maps, pinnacle, uber_pinnacle} + poe2/{00,01,02,03,04}
    + glossary/fr_en + creators_registry/* (20 profiles). Doc 25 stays
    conceptual stub until Sprint 2 ships pob_calc. PoE2 05 (atlas) +
    06 (Runes of Aldur) stay empty stubs for 2026-05-29 fill.
- ✅ **Sprint 1** — Data layer (repoe + trade-stats)
  - notes: Done 2026-05-08. New crate-internal modules
    `sources/{repoe, repoe_refresh, trade_catalogue}.rs`, snapshot bundle
    under `src/sources/snapshots/{poe1,poe2,trade_stats}/` (2.7 MB
    compressed total — well under 12 MB cap), zstd-19 + `include_bytes!`
    + `OnceLock<RwLock<HashMap>>` lazy load. New tool `repoe_lookup`
    (8 categories × 2 games, id + fuzzy-name modes, NOT in
    LOCAL_TOOL_ALLOWLIST). `TradeClient::raw_stats()` refactored to
    consume the catalogue (existing trade tests pass). Daily refresh
    spawned from `bootstrap_runtime::boot_data_refresh`. `build.rs`
    auto-creates stub blobs so first build works without script run.
    `examples/refresh_snapshots.rs` + `scripts/refresh-snapshots.ps1`
    populate real data. PoE2 currently has 3/8 categories on
    repoe-fork (mods/base_items/uniques); `Category::available_for`
    matrix returns explicit "not available" error for the rest.
    68 tests pass. (S1-1) version.txt does not exist on
    repoe-fork.github.io — refresh task uses plain GET, decision
    documented. (S1-2) parity encoded in `available_for`.
- ✅ **Sprint 2** — PoB engine headless
  - notes: Done 2026-05-08. New crate `bestel-pob-engine` with
    `protocol.rs` (ndJSON Cmd/Reply), `lifecycle.rs` (per-game process,
    8 s command timeout, 10 min idle, 3-restart-window circuit-breaker),
    `calc.rs` (high-level orchestration: load_build_xml → set_config →
    set_main_selection → get_stats → echo). Vendored
    `PathOfBuildingCommunity` (v2.65.0) and `PathOfBuilding-PoE2`
    (v2.49.3) as git submodules under `crates/bestel-pob-engine/vendor/`.
    Forked the ianderse `api-stdio` harness from scratch under
    `vendor/api-stdio-bestel/api-stdio.lua` so `set_config` exposes the
    full Calcs key set (enemyIsBoss/charges/flasks/impale/onslaught).
    Built LuaJIT 2.1 ROLLING from upstream MIT source via MSVC; vendored
    Windows binary at `external/luajit/windows-x86_64/luajit.exe`
    (~290 KB) plus `lua51.dll`. SHA256 in `scripts/luajit.sha256.txt`,
    `vendor-luajit.ps1` regenerates from source. `pob_calc` tool
    advertised in tools.rs schemas (length 10 → 11), dispatch arm reads
    the active PobBuild, pipes XML to the engine, surfaces the Calcs
    echo in the response. `pob_calc` IS in `LOCAL_TOOL_ALLOWLIST`
    (simple I/O, high-signal output suits 7-8B models).
    `crates/bestel-core/src/llm/pob_engine.rs` provides the lazy global
    handle — auto-resolves vendored paths from the workspace, env-var
    override `BESTEL_POB_ENGINE_DIR` for production layouts. Engine
    refuses gracefully when binary or PoB submodule is missing. End-to-end
    smoke test passes: spawn → load fixture → calc(offence,
    enemyIsBoss=Pinnacle) → stats + Calcs echo. (S2-1) PoB1 v2.65.0,
    PoB2 v2.49.3 — both have HeadlessWrapper.lua. (S2-2) ianderse
    set_config exposes only bandit/pantheon/enemyLevel; we forked and
    extended. (S2-3) Code-signing deferred — ships unsigned with
    SmartScreen disclaimer. (S2-4) Cross-platform LuaJIT build
    irrelevant — Windows-only this sprint. **Sub-sprint deferred**:
    Tauri `bundle.externalBin/resources` wiring is non-trivial because
    the vendored PoB tree is ~600 MB (TreeData/ alone is 524 MB).
    Engine works in dev (cargo test, cargo tauri dev) via workspace-
    relative path resolution. Production installer integration becomes
    a Sprint 2.1 (figure out which PoB asset subset is load-bearing for
    headless calc, ship the trimmed bundle).
- ✅ **Sprint Tooling** — knowledge layer relocation + dev panel + battery CLI
  - notes: Done 2026-05-08. Pause on the main Intelligence Roadmap to fix
    layout debt + ship a self-service dev/test stack. Knowledge layer
    moved to top-level `prompts/` (SYSTEM_PROMPT.md + CORE_KNOWLEDGE.md
    + references/), mirroring `~/.bestel/prompts/` exactly. New separate
    Tauri window `dev-panel` (mirrors prompt-editor pattern) with four
    tabs: Runs (migrated from old /debug route), Scenarios (loads
    `tests/scenarios/*.toml`), Real prompts (loads
    `docs/test_prompts/real_user_prompts.toml`, 31 entries, filterable
    by category), Live test (model picker + PoB fixture picker + free
    prompt + side-by-side rendered markdown / raw stream events / stats).
    New Settings section "Developer" with "Open dev panel" CTA. Global
    shortcut `Ctrl+Shift+D` opens the panel from any window. Old
    `/debug` route deleted. Scenario parser moved to
    `bestel-core::test_runner` so the CLI and dev panel share it.
    New CLI subcommand `bestel run-battery <dir>
    [--model <id>] [--out <dir>] [--filter-name <substr>]
    [--pob-fixture <name|path>]` runs scenarios in-process against
    live providers, persists `PersistedRun` JSON per scenario, exits
    non-zero on errors. Two known UX bugs (`wiki.La` auto-link from
    markdown-it linkify, narration-leak around tool calls) carry over
    to a future UX sprint — the dev panel makes both reproducible.
- ☐ **Sprint 3** — PoB semantic-facts extractor
  - notes:
- ☐ **Sprint 4** — Wiki SQLite mirror
  - notes:
- ☐ **Sprint 5** — PoE2 0.5 absorption (target 2026-05-29)
  - notes:

## Unresolved questions

- (S1-1) ✅ Resolved 2026-05-08. No `version.txt` on
  `repoe-fork.github.io`; refresh task uses plain GET (no `If-None-Match`)
  — bandwidth cost is ~16 MB/day worst case, acceptable.
- (S1-2) ✅ Resolved 2026-05-08. PoE2 currently exposes 3/8 categories on
  repoe-fork (mods, base_items, uniques). `Category::available_for(game)`
  matrix returns explicit "not available" error for the rest. Update the
  matrix when upstream parity grows.
- (S2-1) Canonical PoE2 PoB fork at submodule pin time?
- (S2-2) `ianderse/pob-mcp` `api-stdio` `set_calc_input` support?
- (S2-3) Windows EV cert for `luajit.exe` code-signing?
- (S2-4) GitHub Actions sufficient for macOS arm64 LuaJIT build?
