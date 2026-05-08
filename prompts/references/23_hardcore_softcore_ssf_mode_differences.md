---
description: How advice diverges across game modes. Hardcore (HC) = max-hit > DPS, deathless mapping. Solo Self-Found (SSF) = self-found viability, no trade. Trade league = trade is part of build power. Ruthless = constrained-economy mode. Per-mode build-viability and pacing differences.
fetch_when: User mentions HC, SSF, trade league, Ruthless, or asks "is this build viable in X mode?". Always check this before recommending a build that assumes trade access or accepts player deaths. Never apply trade-SC defaults silently to a HC or SSF query.
---

# 23 — Hardcore / Softcore / SSF mode differences

This doc covers **how advice changes by mode**. Same build can be excellent in Trade SC and unviable in HC SSF — the constraints are dramatic.

The single most useful question this doc answers: *"Does my advice for mode X actually apply to mode Y?"*

## The four-mode matrix

PoE has four major mode combinations + Ruthless variants:

| Mode | Death penalty | Trade access | Typical concern |
|---|---|---|---|
| Trade Softcore (Trade SC) | XP loss only | Full | Maximise build power; chase uniques fine. |
| Trade Hardcore (Trade HC) | Character to standard | Full | Survive every encounter; max-hit > DPS. |
| Solo Self-Found Softcore (SSF SC) | XP loss only | None — what drops, drops | Build viable from drops; no Mageblood assumed. |
| Solo Self-Found Hardcore (SSF HC) | Character lost + character to standard | None | Conservative build, survive what drops. |
| Ruthless (variant of SC/HC/SSF) | varies | varies | Stripped economy + reduced drops + selective gem availability. |

The matrix matters because **a "great build" is mode-relative**. A 30M-DPS Mageblood build in Trade SC is a 0M-DPS build in SSF (the gear doesn't exist).

## Trade SC — the baseline

- **Death penalty**: 5% XP loss in maps + drop nothing.
- **Trade access**: full.
- **Build pressure**: maximise damage / clearspeed / boss kill speed.
- **Typical chase**: Mageblood, Headhunter, Original Sin, perfect Awakened gems, mirror-tier rare gear.
- **Most build guides assume Trade SC unless stated otherwise** — this is the unstated default.

If a user doesn't state their mode, assume Trade SC. If they sound HC/SSF-flavoured, ask.

## Trade HC — survive every encounter

- **Death penalty**: character moved to **Standard league** (essentially a death-of-character).
- **Trade access**: full, but other HC players are the only sellers (smaller market, higher prices).
- **Build pressure**: every fight must be survivable, deathless mapping is the ethic.

### HC-specific advice deltas (vs SC)

- **Max hit > DPS** — clear speed comes after EHP, max-hit-per-element, instant recovery.
- **Recovery cap**: 30-40% higher than SC bars. 5k+ life/sec at red maps; 8k+ at pinnacle.
- **Defensive layer redundancy**: spell suppression *and* block, armour *and* fortify, Pantheon *and* charm — two layers per damage source.
- **Avoid certain mechanics**: Trial of the Sekhemas / Lab can't fail in HC; if your build can't deathlessly clear them, plan around (carry runs, rerollable trial).
- **Avoid certain map mods**: -max-resists, ele weakness on certain builds, no-leech, etc. Are run-stoppers in HC.
- **Phase-skip before combat**: ranged > melee at high tier; skip phases via DPS bursts vs taking hits.
- **Slow-and-steady**: HC progression is 2-4× slower than SC. Don't push tier until comfortable.

## SSF SC — self-found viability

- **Death penalty**: 5% XP loss only.
- **Trade access**: none. What drops is what you have.
- **Build pressure**: every gear slot must be filled with what's findable.

### SSF-specific advice deltas (vs Trade)

- **Self-found viability**: does this build *exist* without Mageblood / HH / Original Sin? If no → flag, recommend an alternative.
- **Drop-rate gates**: Awakened gem swaps, divine card pools, prophecy chains, league-mechanic-locked uniques.
- **Crafting reliance**: SSF often forces league-mechanic crafting — essence (single-mod craft), fossil (mod-tag-weighted), harvest residue (since 3.18 nerfs), Tier 17 maps, Beastcraft (legacy uniques).
- **League-mechanic priority**: SSF players prioritise league mechanics that drop currency / unique sources differently from trade players.
- **Goal pacing**: a Mageblood is a months-long project, not a "next week" plan.
- **Atlas-tree investment**: SSF atlas tree often biased toward currency-density (scarab fragments, league-mechanic spawn-rate).

### SSF Trade league SOLO viability

A "trade league" in SSF means the user is in trade league but choosing not to trade. Same constraints as SSF SC, but with the option to fall back on trade if a project hits a wall.

## SSF HC — the toughest mode

- **Death penalty**: character lost + transferred to Standard.
- **Trade access**: none.
- **Build pressure**: ultra-conservative. Survive what drops with what you find.

### SSF HC advice deltas (combined SSF + HC)

- **Defensive ceiling**: what max-hit-per-element can you reach with what you find? That's your ceiling.
- **Minimal exotic mechanics**: avoid build-defining uniques like Mageblood (impossible in SSF HC), avoid mechanic-stack identities (poison-stack requires consistent gear).
- **Reliable archetype**: RF Chieftain / armour-stack / minion-armour / explode-Necro — builds that scale with what's findable.
- **Mode lifestyle**: SSF HC players accept months-long character lifecycles and treat death as a normal restart event.

## Ruthless mode (variant)

PoE Ruthless is a "stripped economy" variant that can be applied to SC/HC/SSF.

- **Reduced drop rates** — substantially fewer items, fewer currency drops.
- **No alteration / regal / chaos orbs** — crafting via Vendor recipe, fossil, essence only.
- **No vendor recipes for many items** — full re-craft from essence required.
- **Skill gem availability** — locked behind quest rewards / drop, not buy-with-currency.
- **Build pressure**: pre-3.x crafting / leveling habits matter. Veteran players love it; new players struggle.

**The agent should NOT assume Ruthless** unless the user explicitly mentions it.

## Per-archetype mode-viability quick matrix

A cheat-sheet for "which archetype works in which mode":

| Archetype | Trade SC | Trade HC | SSF SC | SSF HC | Notes |
|---|---|---|---|---|---|
| RF Chieftain | ✅ | ✅ | ✅ | ✅ | Pohx archetype; reliable across all modes. |
| Lightning Arrow Deadeye | ✅ | ⚠️ (defensive layers required) | ⚠️ (gear-dependent) | ❌ | Crit-stack; high gear floor. |
| Armour-stack | ✅ | ✅ | ⚠️ | ⚠️ | Recovery-heavy; ascendancy choice matters. |
| Minion / Spectre | ✅ | ✅ | ✅ | ✅ | Spectre AI matters for HC; AG safety. |
| Trigger / CWC / CoC | ✅ | ⚠️ | ❌ | ❌ | High gear-quality floor. |
| Mageblood-required (Heralds-stack, etc.) | ✅ | ✅ | ❌ | ❌ | Mageblood unreachable in SSF. |
| Headhunter-required (HH MFer) | ✅ | ⚠️ | ❌ | ❌ | HH unreachable / unsafe in SSF/HC. |
| Original-Sin-required (CI ES caster) | ✅ | ⚠️ | ❌ | ❌ | OS rare in SSF. |
| Vaal RF | ✅ | ✅ | ✅ | ✅ | RF variant; same reliability. |
| Toxic Rain Pathfinder | ✅ | ✅ | ✅ | ⚠️ | Self-found-viable; HC requires scaling. |

The ⚠️ entries are *conditional* — viable with specific gear or specific defensive bar.

## How the agent should behave

1. **Always ask the mode** if not specified and the answer affects the recommendation.
2. **If user is HC**: check max-hit-per-element, EHP recovery, defensive layer redundancy. Reject DPS-only recommendations.
3. **If user is SSF**: check whether build-defining uniques exist in non-trade context. Recommend league-mechanic-fitting alternatives.
4. **Never silently apply trade-SC defaults** to a HC or SSF query — flag the assumption explicitly.
5. **Default to conservative** when mode is ambiguous.

## Cross-references

- `08_defence_recovery_survivability.md` — defensive layer mechanics (HC pressure).
- `17_build_archetype_taxonomy.md` — archetype-specific scaling levers.
- `09_itemisation_crafting.md` — SSF crafting reliance (essence, fossil, harvest residue).
- `11_endgame_economy_trade_leagues.md` — economy / league mechanics.
- `21_currency_and_barter_taxonomy.md` — currencies + Currency Exchange.
- `creators_registry/pohx.md` — RF Chieftain HC viability authority.
- `creators_registry/ben_.md` — HC / Gauntlet authority.
- `creators_registry/goratha.md` — Maxroll SSF-friendly league starters.
