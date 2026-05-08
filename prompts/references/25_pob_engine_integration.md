---
description: How the agent talks to the bundled headless PoB engine (`pob_calc`). Output keys to trust, when to defer to the engine vs `<PlayerStat>` cache, how to interpret Calcs config mismatches.
fetch_when: User asks "what's my real DPS?", "show me my EHP", or any question that requires calculated stats rather than just structural extraction.
---

# 25 — PoB engine integration

> **Status (2026-05-08): active.** Bestel ships a forked
> `api-stdio-bestel/api-stdio.lua` harness on top of vendored
> PathOfBuildingCommunity (PoE1) + PathOfBuilding-PoE2 (PoE2) running under
> a bundled LuaJIT 2.1 subprocess. The new tool `pob_calc(category)` returns
> canonical PoB `output` table keys plus a Calcs-config echo. **Use it any
> time the user asks for a calculated number** — do not quote
> `<PlayerStat>` cache values when the engine is reachable.

## When to call `pob_calc`

| Question shape | Tool to use |
|---|---|
| "What's my DPS / EHP / max-hit?" | `pob_calc` (offence / defence) |
| "Why am I dying to fire bosses?" | `pob_calc(category=defence)` then read `MaxHitFire` |
| "Which charge config is best?" | Two `pob_calc` calls with different `calcs.useFrenzyCharges`, compare |
| "What's the assumption baked into this build?" | `pob_calc(category=all)` and surface the `calcs` echo |
| Structural / equipment listing | `get_active_build` (no engine call) |
| Mod ranges / base item data | `repoe_lookup` (no engine call) |

If the user has no active build loaded, `pob_calc` returns an explicit
"no active build" error — instruct the exile to save a build in PoB so
the chronicler can read it.

## Categories and key output names

`pob_calc` accepts `category` ∈ `offence | defence | charges | reservation | ailments | all`:

- **offence**: `TotalDPS`, `CombinedDPS`, `FullDPS`, `AverageHit`,
  `Speed`, `HitChance`, `CritChance`, `CritMultiplier`, `IgniteDPS`,
  `BleedDPS`, `PoisonDPS`, `TotalDot`.
- **defence**: `EHP`, `LifeUnreserved`, `LifeRecoverable`, `EnergyShield`,
  `Mana`, `Armour`, `Evasion`, `MeleeEvadeChance`,
  `PhysicalDamageReduction`, `MaxHitFire`, `MaxHitCold`, `MaxHitLightning`,
  `MaxHitPhysical`, `MaxHitChaos`, `BlockChance`, `SpellBlockChance`,
  `SpellSuppressionChance`.
- **charges**: `PowerChargesMax`, `FrenzyChargesMax`, `EnduranceChargesMax`.
- **reservation**: `LifeReserved`, `ManaReserved`, `LifeReservedPercent`,
  `ManaReservedPercent`, `SpiritReserved`.
- **ailments**: `EnemyShockChance`, `ShockEffect`, `EnemyChillEffect`,
  `EnemyFreezeChance`, `IgniteChance`.
- **all**: full PoB `output` table — use sparingly, response is large.

## The Calcs-echo rule (load-bearing)

Two PoBs with identical gear can show **10× DPS swings** purely from
`<Calcs>` config. The forked harness exposes:

- `enemyIsBoss` — `true` (== Pinnacle) / `false` (== None) / explicit
  string `"None" | "Boss" | "Pinnacle" | "Uber"`.
- `usePowerCharges` / `useFrenzyCharges` / `useEnduranceCharges` — booleans.
- `forceBuffOnslaught` — boolean.
- `multiplierImpaleStacks` — integer.
- `useFlask1` … `useFlask5` — booleans (toggle each flask slot).

The response **always** includes a `calcs` object echoing the effective
config. **Surface those assumptions in your answer**, every time:

> "Against a Pinnacle boss, with Power and Frenzy charges up and Flask 1-3
> active, your TotalDPS is **3.4M**. With charges off and Flask 1-3
> inactive, the same build does **1.1M** — so a clearing build profile
> changes the answer ~3×."

If the user asks "what's my DPS?" without specifying assumptions, default
to `enemyIsBoss=true, charges=true` (the load-bearing pinnacle profile)
and **state that explicitly**.

## Engine vs. `<PlayerStat>` cache divergence

`get_active_build` returns the cached `<PlayerStat>` block from the PoB
XML. That cache is stale whenever:

- The user edited the PoB outside the desktop client.
- The PoB version was bumped between save and load.
- The user toggled a Calcs config without re-saving.

Whenever `pob_calc` returns a number that differs from
`<PlayerStat>` by more than ~5%, **trust the engine** and flag the
cache as stale: "PoB cache shows 1.2M DPS, engine shows 950k — your
saved file is out of date by [N] minutes; the live calc is the truth."

## Failure modes and how to handle them

| Failure | Surface to the user | Action |
|---|---|---|
| Engine not configured (MCP serve mode) | "the calculator only runs in the Bestel desktop window" | Suggest opening the desktop app |
| LuaJIT subprocess crashes mid-call | "the engine restarted, retrying" | Auto-retry once; if 3 in 5 min → circuit-break |
| 8 s timeout | "your build's calc is unusually heavy — falling back to PoB cache" | Use `<PlayerStat>` cache, flag staleness |
| Build XML rejected | "PoB couldn't parse the build — version mismatch?" | Suggest re-saving in current PoB |
| Calcs config rejected | Echo the rejected key | Retry without that key |

## Bundle layout

The desktop install ships:

- `external/luajit/windows-x86_64/luajit.exe` (~290 KB, LuaJIT 2.1
  ROLLING built from upstream MIT source).
- `external/luajit/windows-x86_64/lua51.dll`.
- `crates/bestel-pob-engine/vendor/api-stdio-bestel/api-stdio.lua`
  (forked harness, MIT, conceptually inspired by ianderse/PathOfBuilding
  `api-stdio` branch but rewritten to extend `set_config`).
- `crates/bestel-pob-engine/vendor/PathOfBuildingCommunity/` (PoE1 PoB,
  pinned to `v2.65.0`).
- `crates/bestel-pob-engine/vendor/PathOfBuilding-PoE2/` (PoE2 PoB,
  pinned to `v2.49.3`).

PoE2 calc accuracy depends on the PoB-PoE2 fork keeping pace with GGG
patches. After the 0.5 release (target 2026-05-29), Sprint 5 bumps the
submodule pin and re-validates the harness.

## Sprint 2 scope notes

- **Windows-only** for the binary build. macOS / Linux refuse `pob_calc`
  with a clear "engine not bundled for this platform" error.
- **Unsigned**. SmartScreen flags "unknown publisher"; documented in
  the README onboarding. VirusTotal scan recorded in
  `LICENSE-third-party.md`.
- The 600 MB+ vendored PoB tree is **not** in the desktop installer —
  it's a development-time mirror. Production bundling is deferred to a
  follow-up sub-sprint (see `docs/ROADMAP.md`).
