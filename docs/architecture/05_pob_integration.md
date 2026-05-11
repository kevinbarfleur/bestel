# 05 — PoB integration

Path of Building is the single most important external dependency. Bestel
reads PoB XML exports, derives **semantic facts** about the build, and
optionally runs the upstream PoB Lua engine headlessly to get
canonical DPS / EHP numbers under specific assumptions.

Three layers, called in order of cost. Each is independently useful.

```
┌────────────────────────────────────────────────────────────────┐
│ Layer 1 — Parser   (crates/bestel-core/src/pob/parser.rs)      │
│   PoB XML → PobBuild         ~10-30 ms, sync, no I/O          │
└────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────┐
│ Layer 2 — Semantic (crates/bestel-core/src/pob/semantic.rs)    │
│   PobBuild → BuildIdentity   ~1-5 ms, pure heuristics         │
└────────────────────────────────────────────────────────────────┘
                                │
                                ▼
┌────────────────────────────────────────────────────────────────┐
│ Layer 3 — Engine   (crates/bestel-pob-engine/)                 │
│   PobBuild XML → CalcResponse                                  │
│   ~200-800 ms per call, LuaJIT subprocess, ndJSON over stdio  │
└────────────────────────────────────────────────────────────────┘
```

---

## Layer 1 — Parser

`crates/bestel-core/src/pob/parser.rs::parse_file` deserialises a PoB
XML export into a `PobBuild` struct. Sync, no I/O after the read,
~10–30 ms per file (`quick-xml`).

`PobBuild` carries everything the file declares:

- Game (`PoeVersion: Poe1 | Poe2`)
- Class, ascendancy, level
- Items (full raw mod text preserved verbatim — the LLM cites this
  for tooltips and re-emission)
- Skill groups with linked gems
- Full passive node-id list, mastery picks, jewel placements
- Defences (life, mana, ES, EHP, armour, evasion, suppression, block,
  dodge, per-element resistances + max-hit values)
- Charges (power / frenzy / endurance current+max)
- Buffs (combat / buff / curse lists)
- Config (boss profile, enemy resists, flask uptimes, custom mods)
- Tree summary (class id, ascendancy id, version, node count, mastery
  count, weapon-set node split)
- `pobb.in` import link (when present)
- Stats dictionary (every named PoB stat: `PlayerDPS`, `LifeUnreserved`,
  `EnergyShield`, etc.)

`PobBuild` is wrapped in `Arc<RwLock<Option<PobBuild>>>` inside
`BuildContext` (`crates/bestel-core/src/llm/tools.rs`) so commands and
the streaming provider share the same view without copying.

Supporting modules:

- `pob/mod.rs` — public types (`PobBuild`, `PobItem`, `PobSkillGroup`,
  `PobSkillGem`, `PobDefenses`, `PobCharges`, `PobBuffs`, `PobConfig`,
  `PobTree`, `JewelPlacement`, `PoeVersion`, `PobBuildSummary`)
- `pob/dict.rs` — vendored skill / gem / item lookup tables
- `pob/locator.rs` — find PoB Builds folder (PoE1 + PoE2 trees under
  `Documents/Path of Building` and `Documents/Path of Building (PoE2)`)
- `pob/watcher.rs` — `notify`-based filesystem watcher; emits
  `pob:build` / `pob:cleared` events to the Tauri app

The parser is intentionally sync. CPU-bound, deterministic, no
network — no benefit from async, and a regular `tokio::task::spawn_blocking`
keeps the runtime free during the brief parse.

---

## Layer 2 — Semantic (`BuildIdentity`)

`crates/bestel-core/src/pob/semantic.rs::BuildIdentity::from_build(&PobBuild)`
adds three derived facts per build. Pure heuristics, ~1–5 ms, no I/O.

```rust
pub struct BuildIdentity {
    pub archetype: ArchetypeTags,
    pub defining_uniques: Vec<DefiningUniqueMatch>,
    pub conversion_chain: Option<ConversionChain>,
}

pub struct ArchetypeTags {
    /// life | ES | LL | CI | MoM | hybrid | RF | VLS
    pub defense: Vec<String>,
    /// crit | non-crit | non-crit-EO | ailment-stack | DoT
    pub hit_model: Vec<String>,
    /// self-cast | trigger | totem | mine | trap | minion | autobomber
    pub mechanic: Vec<String>,
}

pub struct DefiningUniqueMatch {
    pub name: String,
    pub category: String,       // engine | defining | amplifier
    pub identity_hint: String,
}

pub struct ConversionChain {
    pub steps: Vec<String>,
    pub final_type: String,
}
```

### Archetype tagging

Three-axis taxonomy. Each axis returns 0..N tags — most builds yield
exactly one per axis, but hybrids like `["life", "MoM"]` are valid.
Heuristics are priority-ordered, first-match-wins.

Detection sources (in order of reliability):

1. PoB stats dictionary (`LifeUnreservedPercent`, `EnergyShield`,
   `LifeUnreserved`, …)
2. Items raw text scan (for keystones: Chaos Inoculation, Eldritch
   Battery, Resolute Technique, Mind over Matter, Avatar of Fire, …)
3. Main-skill gem identity (RF / VLS only fire when the matching gem
   is the `main_skill`, never when it's a side group — otherwise
   crit / hit builds keep RF in a side group as a damage buff and would
   be mis-tagged)

The canonical taxonomy lives in
`prompts/references/17_build_archetype_taxonomy.md`. The semantic
layer encodes the matching rules; the reference document is the
explanation the agent reads when asked to justify a tag.

### Defining uniques — engine / defining / amplifier

Each unique item in the build is matched against a curated registry
and tagged:

- **`engine`** — the build collapses without it. Examples: Mageblood,
  Headhunter, Original Sin, Cospri's Malice, Mjölner, Doryani's
  Prototype, Voll's Devotion, Sign of the Sin Eater, The Squire,
  Skin of the Lords (when running specific keystones).
- **`defining`** — shapes the archetype but is replaceable with a
  rework. Examples: Replica Soul Tether, Asenath's Gentle Touch,
  Indigon, timeless jewels, Crown of Eyes, The Pariah.
- **`amplifier`** — replaceable boost; the build still works without
  it. Examples: Watcher's Eye, Bottled Faith, Thread of Hope, Forbidden
  Flame / Flesh, Aul's Uprising (when not the keystone source).

### Engine-aware swap rule

When the agent considers proposing an item swap, it must surface the
`category` flag. `prompts/SYSTEM_PROMPT.md` enforces an
**invitation-required swap rule**:

> Never recommend selling an item flagged `category: "engine"` without
> explicit user instruction; engine items collapse the build if
> removed.

A second helper, `detect_engine_mod_pattern(raw_text, main_skill_lower)`,
extends the rule to *engineered rares*: items whose mod text references
the build's main skill, gem level, or specific support items. If a
match fires, the item is treated as if it were engine-tagged — same
invitation rule applies.

This is the system-prompt embodiment of the deferred "counterfactual
engine-vs-filler classifier" from the original Sprint 3 roadmap. The
actual classifier (PoB engine XML round-trip: parse → drop slot →
reload → re-calc → diff DPS / EHP) was deferred because the
compile-time tags + the mod-pattern detector deliver most of the
practical value at zero runtime cost.

### Conversion chain

Manual line-by-line scan of item raw mods for
`(\d+)% of <Type> Damage Converted to <Type>` plus intrinsic gem
conversions (Cold to Fire Support, Glacial Cascade, Volatile Dead,
Hatred reservation conversions, etc.).

Returns `None` if no conversion is found. Otherwise carries the
ordered steps and the final damage type, which the agent surfaces in
its identity card.

### Where it's used

`BuildIdentity::from_build` is called at the top of
`render_build_for_llm` in `tools.rs`, before serialisation, so every
`get_active_build` response carries `archetype`, `defining_uniques`,
and `conversion_chain`. The agent reads these *before* commenting on
the build (the system prompt instructs "surface archetype tags FIRST"
— do not guess from class+ascendancy alone).

---

## Layer 3 — Engine sidecar

`crates/bestel-pob-engine/` wraps a long-lived **LuaJIT 2.1 ROLLING**
subprocess that loads the upstream PoB Lua sources through a forked
harness. Used by the `pob_calc` tool to re-run PoB calculations with
custom Calcs config (boss profile, charges, flask toggles).

### Crate layout

| File | Role |
|---|---|
| `src/lib.rs` | Public exports: `CalcRequest`, `CalcResponse`, `PobEngineHandle`, `Category`, `EngineCalcs`, `PobEngineError` |
| `src/lifecycle.rs` | Spawn / kill / idle-restart of the LuaJIT child. Circuit-breaker on 3 crashes / 300 s. Per-game `Mutex` (PoE1 vs PoE2 independent) |
| `src/protocol.rs` | ndJSON wire protocol — `Cmd` (Ping, Version, Quit, LoadBuildXml, GetStats, SetConfig, GetConfig, SetMainSelection) and `Reply` (ok, error, version, game, stats, config, **`active_skill`**) |
| `src/calc.rs` | High-level `CalcRequest` builder + response decoder. `EngineCalcs` struct (10 toggles: boss flag, charges 3×, 5 flasks, impale stacks, onslaught) |
| `src/error.rs` | `PobEngineError` (`EngineCrashed`, `Timeout`, `ProtocolBroken`) |

### Lifecycle

- **Spawn on first use.** `lifecycle.rs::PobEngineHandle::ensure`
  starts the LuaJIT child via `tokio::process::Command`.
- **Windows.** `creation_flags(CREATE_NO_WINDOW = 0x0800_0000)` so no
  console pops up.
- **Idle timeout.** The child is kept alive for 600 s after the last
  call; subsequent calls reuse it. This amortises the ~1 s LuaJIT
  warm-up over a chat session.
- **Circuit-breaker.** If the engine crashes 3 times within 300 s,
  spawning is paused (a 60 s lockout). Avoids busy-spawning on a
  systematic bug.
- **Per-game isolation.** PoE1 and PoE2 engines run as separate
  processes — different upstream Lua, different `Modules/`, different
  build IDs. They are serialised behind a single per-game `Mutex` so
  concurrent calls queue rather than race the same stdin pipe.

### ndJSON protocol

One JSON object per line, newline-terminated.

```rust
pub enum Cmd {
    Ping,
    Version,
    Quit,
    LoadBuildXml { xml: String },
    GetStats { category: String },
    SetConfig { config: BTreeMap<String, Value> },
    GetConfig,
    SetMainSelection {
        main_socket_group: Option<u32>,
        main_active_skill: Option<u32>,
        skill_part: Option<u32>,
    },
}

pub struct Reply {
    pub ok: bool,
    pub error: Option<String>,
    pub version: Option<String>,
    pub game: Option<String>,
    pub stats: Option<Value>,
    pub config: Option<Value>,
    pub active_skill: Option<Value>,   // ALWAYS present in get_stats replies
}
```

### `CalcRequest` orchestration

`calc.rs::CalcRequest` builds the multi-step request the `pob_calc`
tool needs:

1. `LoadBuildXml { xml }` — hash-checked: only reloaded if the build
   text changed since the last call (load is the expensive step
   ~150 ms).
2. *(optional)* `SetMainSelection { … }` — pick which skill group to
   calc against.
3. `SetConfig { config }` — apply the agent's `EngineCalcs` overrides
   to the Calcs panel.
4. `GetStats { category }` — return a sub-block (`offence`,
   `defence`, `charges`, `reservation`, `ailments`, or `all`).

The response always echoes:

- The **effective Calcs config** (so the agent reports the assumptions
  it used).
- The **`active_skill` metadata** (`main_socket_group`,
  `main_active_skill`, `skill_part`, `active_skill_label`,
  `active_skill_gem`) — see below for why this matters.

### The `active_skill` verification

In the 2026-05-08 scenario battery, two builds silently produced wrong
DPS because the engine's main-skill resolver fell back to a different
gem than the build declared. The bug was *silent* — the JSON looked
correct, the number was just for the wrong skill.

The forked harness now ALWAYS exposes
`active_skill_meta()` and the dispatcher requires the agent to
**verify** the echoed label matches the build's `main_skill` from
`get_active_build`. The system prompt makes this a hard rule:

> Before quoting any `pob_calc` number, compare
> `active_skill.active_skill_label` with the build's `main_skill`. If
> they don't match, report the cached `<PlayerStat>` with an explicit
> staleness note, OR retry `pob_calc` with an explicit `skill_index`.

### Calcs disclosure

Two PoBs with identical gear can show 10× DPS swings purely from Calcs
config (enemy boss profile, flask uptime, charge assumptions, impale
stacks). The agent MUST surface its assumptions in the answer:

> *"Real DPS at Pinnacle with 3 power charges, useFlask3 on:
> 8.4M shock DPS."*

— not just "8.4M DPS". The system prompt and the tool description both
embed this rule.

---

## Forked harness — `api-stdio-bestel`

`crates/bestel-pob-engine/vendor/api-stdio-bestel/api-stdio.lua` is a
fork of the upstream `ianderse/PathOfBuilding` `api-stdio` branch.
Reasons for forking:

- **UI primitive stubs.** Upstream PoB depends on `NewDropDown`,
  `NewEditControl`, `NewButtonControl`, … which require a real
  Windows UI. The fork stubs these via `inert_control()` so PoB boots
  headless.
- **Sanitise cycle-breaker.** PoB's Calcs output contains recursive
  table references that `dkjson` cannot encode. `sanitise()` walks the
  tree once, replacing cycles with stringified placeholders.
- **`active_skill_meta()`.** Custom function returning
  `{main_socket_group, main_active_skill, skill_part,
  active_skill_label, active_skill_gem}` — the metadata the agent
  needs to verify the engine targeted the right skill.
- **`set_main_selection` fix.** Upstream had a silent wrong-skill
  fallback when the requested `main_active_skill` index was out of
  bounds; the fork errors explicitly.
- **`GetVirtualScreenSize`, `ConPrintf`, `ConPrintTable` stubs** so
  PoB's logging plumbing doesn't crash.

---

## Submodules

`.gitmodules` pins the upstream PoB sources:

```
[submodule "crates/bestel-pob-engine/vendor/PathOfBuildingCommunity"]
    url = https://github.com/PathOfBuildingCommunity/PathOfBuilding

[submodule "crates/bestel-pob-engine/vendor/PathOfBuilding-PoE2"]
    url = https://github.com/PathOfBuildingCommunity/PathOfBuilding-PoE2
```

The exact pinned commit is whatever `git submodule status` shows in
the working tree. To bump on a PoB patch day:

```pwsh
cd crates/bestel-pob-engine/vendor/PathOfBuildingCommunity
git fetch
git checkout <tag>          # e.g. v2.65.0
cd ../PathOfBuilding-PoE2
git fetch
git checkout <tag>          # e.g. v2.49.3
cd ../../..
git add crates/bestel-pob-engine/vendor/PathOfBuildingCommunity \
        crates/bestel-pob-engine/vendor/PathOfBuilding-PoE2
git commit -m "deps(pob): bump submodules to <versions>"
```

`docs/POE2_05_LAUNCH_PLAYBOOK.md` carries the patch-day checklist
(snapshot refresh, submodule bump, prompt updates).

---

## File watcher

`crates/bestel-core/src/pob/watcher.rs` wraps `notify` to watch the
two PoB Builds folders and surfaces filesystem events to the Tauri
side.

- On change: parse the file, emit `pob:build` with the new summary +
  full `PobBuildDto`.
- On delete: emit `pob:cleared`.
- The Tauri side debounces and threads the events through
  `AppState.watcher` so the chat session sees only one update per
  burst.

The prompt-editor's own writes go through `SelfWriteTracker` to avoid
echo loops, but the PoB watcher is on a different folder so it's
unaffected.

---

## See also

- [04 — Tools catalogue](./04_tools_catalogue.md) — full `pob_calc`
  schema and trigger rules.
- [02 — Agentic system](./02_agentic_system.md) — how
  `get_active_build` slots into the chat flow, and the
  anti-hallucination directive that forces the agent to cite ONLY
  values from the JSON.
- `prompts/references/25_pob_engine_integration.md` — the
  reference-library doc the agent reads when reasoning about the
  engine.
