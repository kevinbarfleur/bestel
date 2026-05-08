---
description: How the agent talks to the bundled headless PoB engine (planned Sprint 2). Output keys to trust, when to defer to the engine vs when XML extraction is enough, how to interpret Calcs config mismatches.
fetch_when: User asks "what's my real DPS?", "show me my EHP", or any question that requires calculated stats rather than just structural extraction. Currently a stub — see status note.
---

# 25 — PoB engine integration

> **Status (2026-05-08): stub.** As of this writing, Bestel has **no calc engine**. Rely on `<PlayerStat>` cache values from PoB XML (extracted by `get_active_build`) and **flag uncertainty explicitly** when reporting calculated numbers. Pre-cached stats can be stale if the file was edited externally.

> **Sprint 2 plan**: bundled LuaJIT sidecar running PathOfBuildingCommunity's `HeadlessWrapper.lua`. New tool `pob_calc(category)` returning canonical `output` table keys.

## What to do today (no engine available)

- Use `get_active_build` for structural facts: class, ascendancy, level, equipped items, skill groups, tree node count.
- Use `<PlayerStat>` cache values **only** as a "last seen by PoB" reference, never as authoritative.
- For DPS / EHP / max-hit questions: explicitly state "I don't have a calc engine yet — these are PoB's last-saved stat snapshots, which may be stale if you've edited the build outside PoB."

## What `pob_calc` will provide (Sprint 2)

- **offence**: TotalDPS, CombinedDPS, FullDPS, AverageHit, Speed, HitChance, CritChance, CritMultiplier, IgniteDPS, BleedDPS, PoisonDPS, TotalDot
- **defence**: EHP, LifeUnreserved, LifeRecoverable, EnergyShield, Mana, Armour, Evasion, MeleeEvadeChance, PhysicalDamageReduction, MaxHitFire/Cold/Lightning/Phys/Chaos, BlockChance, SpellBlockChance, SpellSuppressionChance
- **charges**: PowerChargesMax, FrenzyChargesMax, EnduranceChargesMax
- **reservation**: LifeReserved, ManaReserved, LifeReservedPercent, ManaReservedPercent, SpiritReserved
- **ailments**: EnemyShockChance, ShockEffect, EnemyChillEffect, EnemyFreezeChance, IgniteChance

## Calcs config awareness (critical, even today)

Two PoBs with identical gear can show 10× DPS differences purely from `<Calcs>` configuration (`enemyIsBoss`, `EnemyLevel`, `usePowerCharges`, `useFrenzyCharges`, `useEnduranceCharges`, `forceBuffOnslaught`, `multiplierImpaleStacks`, `useFlask{N}`). When you read a PoB and report any DPS number — calculated or cached — **always echo the user's Calcs assumptions**.

> Status: stub. TODO: refactor to "current operating model" once Sprint 2 ships. Keep the fallback-to-cache section as a documented degraded mode for when the sidecar is unavailable.
