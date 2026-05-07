---
description: League cycle, PoE1 endgame stages, PoE2 endgame (LEGACY pre-0.5), economy/trade source roles, trade & meta answer patterns.
fetch_when: For Atlas progression, league cycle, trade strategy, currency farming, meta questions, "what to do at endgame".
---

# 11 — Endgame, economy, trade, and leagues

## League cycle

PoE changes in cycles. The consequence for the agent: any answer about "current," "meta," "price," "league," "patch," "drop," "nerf / buff," "start," or "still works" requires a fresh search.

The official patch-notes forum is the primary source for versions. The wikis can lag; guides become wrong as soon as a patch changes a gem, an item, an ascendancy, or a league mechanic.

## PoE1 endgame

PoE1 endgame revolves around maps and the Atlas of Worlds. The `Map` page describes maps as items that open portals to zones with monsters and league encounters, central to endgame. The `Atlas of Worlds` page describes the Atlas as the endgame system explored through map items in the Map Device.

The agent must connect advice to the player's progression stage:

- campaign → resistances, gems, simple bases;
- white maps → caps, life, 4 / 5-link, early weapon;
- yellow maps → chaos res, mitigation, ailment coverage, map mods;
- red maps → sustain, atlas strategy, boss damage, recovery;
- pinnacles → mechanics, max hit, uptime, single target, ailment / curse / debuff coverage;
- farming strategy → atlas passives, scarabs / fragments, clear speed, investment and returns.

## PoE2 endgame

> ⚠️ **Important: PoE2 0.5 (end of May 2026) overhauls the endgame mapping system.** All concepts below (Waystones, Towers, Precursor Tablets, Citadels, Crisis Fragments, Burning Monolith, Arbiter of Ash) describe the legacy pre-0.5 state. Before any PoE2 endgame advice, the agent must check the current patch notes and confirm whether the described structure still applies. The conceptual pillars (procedural Atlas, map-as-consumable, tier system, Atlas Passive Tree, pinnacle bosses, Realmgate hub) should survive the rework; names and exact mechanics may change.

PoE2 uses a different Atlas. The PoE2 `Atlas` page describes it as the main endgame system, infinite and expanding in all directions. It also states that `Waystones` open maps, that tier determines monster level, and that a map can be failed if the player dies or runs out of portals. `Precursor Tablets` add map mechanics through the Map Device.

For PoE2, the agent must always verify the current patch before giving endgame advice, because towers, tablets, citadels, and league mechanics are recent and shifting.

## Economy and trade

Prices are not permanent facts. An item may be:

- expensive at league start and common later;
- absent in SSF;
- unavailable in PoE2 but common in PoE1;
- available only in certain leagues;
- legacy / permanent-league only;
- cheap without specific corruptions / tiers, very expensive with them.

## Source roles for economy

| Source | Role |
|---|---|
| Official trade / trade2 | Live listings, current prices, real availability. |
| Official trade API `data/static` and `data/stats` | Static IDs, currencies, stat filters. |
| poe.ninja | Trends, ratios, build / item popularity, market snapshots. |
| Community guides | Purchase context, progression, estimated budgets — must be dated. |
| PoEDB / PoE2DB | Existence of mods / items and their values; not prices. |

The official Developer Docs state that the APIs primarily expose read-only snapshots and that details may change to protect the servers. The agent must therefore avoid promising API stability that is not guaranteed.

## Trade answer pattern

When the user asks for an item to buy:

1. define the mandatory mods;
2. distinguish explicit vs pseudo filters;
3. specify league and platform if necessary;
4. suggest a range of filters rather than a single exact roll;
5. explain which filters to relax if nothing is found;
6. never conclude "it doesn't exist" from zero listings;
7. if a price is requested, consult live trade or a current economic source.

## Meta answer pattern

When the user asks "what build is strong?":

1. ask or infer the game;
2. verify the patch / league;
3. use poe.ninja for popularity, not as mechanical proof;
4. use dated guides for practical routes;
5. verify core skill / support / items on wiki / PoEDB;
6. propose based on budget, playstyle, and target content.

## Useful sources

- [Official PoE Patch Notes forum](https://www.pathofexile.com/forum/view-forum/patch-notes)
- [Path of Exile Developer Docs](https://www.pathofexile.com/developer/docs)
- [Official trade data/static](https://www.pathofexile.com/api/trade/data/static)
- [Official trade data/stats](https://www.pathofexile.com/api/trade/data/stats)
- [PoE1 Map](https://www.poewiki.net/wiki/Map)
- [PoE1 Atlas of Worlds](https://www.poewiki.net/wiki/Atlas_of_Worlds)
- [PoE2 Atlas](https://www.poe2wiki.net/wiki/Atlas)
- [PoE2 Precursor tablet](https://www.poe2wiki.net/wiki/Precursor_tablet)
- [poe.ninja](https://poe.ninja/)
