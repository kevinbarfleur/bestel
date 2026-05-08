---
description: Trusted-source hierarchy, freshness rules, 7-step minimum search algorithm, anti-hallucination rule.
fetch_when: Before answering anything where source choice matters; whenever the user asks where Bestel pulls information from.
---

# 01 — Source policy and retrieval workflow

## Core principle

Treat this pack as a **navigation map**, never as a final source of truth. Every factual claim about an item, mod, formula, numerical value, gem, ascendancy, patch, league, or price must be verified against a current source.

A good PoE answer rarely starts with "I know." It starts with: *what game, what patch, what mechanic, what canonical source?*

## Source hierarchy

| Tier | Sources | Primary use | Risk |
|---|---|---|---|
| 1 | [pathofexile.com patch notes](https://www.pathofexile.com/forum/view-forum/patch-notes), official announcements, [Developer Docs](https://www.pathofexile.com/developer/docs), official trade | Patches, league cycle, API, live trade, GGG announcements | Highly reliable but sometimes incomplete on mechanical details. |
| 2 | [PoE Wiki](https://www.poewiki.net/wiki/Path_of_Exile_Wiki), [PoE2 Wiki](https://www.poe2wiki.net/wiki/Path_of_Exile_2_Wiki) | Mechanics, items, skills, passives, bosses, leagues, endgame | Community-edited; check recent / stub pages, especially on PoE2. |
| 3 | [PoEDB](https://poedb.tw/us/Modifiers), [PoE2DB](https://poe2db.tw/us/Modifiers), [RePoE](https://github.com/repoe-fork/repoe) | Datamined data: mods, bases, tags, tiers, spawn weights, internal IDs | Excellent for raw numbers, weak at explaining interactions. |
| 4 | [Craft of Exile](https://www.craftofexile.com/), crafting simulators | Odds, crafting methods, route comparison | Depends on data freshness and game support. |
| 5 | [poe.ninja](https://poe.ninja/), official trade, trade2 | Prices, popularity, build trends | Market observations, never proof of mechanical validity. |
| 6 | Dated community guides: Maxroll, Mobalytics, Pohx, PoE Vault, PoE Planner, Path of Pathing, PoELab | Inspiration, meta, practical progression | Use only with explicit patch, author, date, and a wiki / PoEDB cross-check. |

## Sources to block or strongly downgrade

- `pathofexile.fandom.com` — the legacy wiki ranks high in search but is largely abandoned and outdated.
- RMT sites, currency sellers, boosting services, paid carry providers.
- Undated SEO blogs without an author or compiling info without sources.
- AI-generated summaries and pages that fail to link their primary sources.
- Forum posts from unknown players, except as a signal of an issue worth verifying elsewhere.

## Freshness rule

Always check patch notes when:

- the question mentions "current", "latest", "league", "meta", "nerf", "buff", "new", "still works", "PoE2";
- the answer depends on a current value or current availability;
- the question concerns PoE2, since Early Access shifts faster;
- the build was sourced from an external guide;
- the user is talking about a recently added item or skill.

The official forum lists the canonical patch notes for PoE1 and PoE2. Patch numbers and release dates change constantly — never freeze them in this document; always re-fetch at answer time.

## Minimum search algorithm

For any non-trivial question:

1. **Resolve the game.** PoE1 or PoE2. If a build is loaded, the `game` field wins. Otherwise infer from entities; if the inference is fragile, ask, or answer with both branches.
2. **Classify the question.** mechanic, item, build diagnosis, crafting, trade, patch, endgame, guide/meta.
3. **Fetch the explanatory source.** The wiki matching the game.
4. **Fetch the numeric source.** PoEDB / PoE2DB / RePoE for mods, tiers, tags, bases, gem data.
5. **Fetch the temporal source.** Official patch notes if the question can be affected by a patch.
6. **Fetch the economic source.** Official trade or poe.ninja for prices / popularity — never to explain a mechanic.
7. **Synthesise.** Explain the mechanic, tie back to the build, give a concrete action, cite the sources actually consulted.

## Search templates

### PoE1 mechanic

```text
site:poewiki.net/wiki <mechanic or exact English name>
site:poedb.tw/us <item/mod/gem exact English name>
site:pathofexile.com/forum/view-thread <patch version> <mechanic>
```

### PoE2 mechanic

```text
site:poe2wiki.net/wiki <mechanic or exact English name>
site:poe2db.tw/us <item/mod/gem exact English name>
site:pathofexile.com/forum/view-thread "Path of Exile 2" "Patch Notes" <version or skill>
```

### Crafting mod or tier

```text
site:poedb.tw/us/Modifiers "<stat text>"
site:poe2db.tw/us/Modifiers "<stat text>"
site:poewiki.net/wiki "<crafting method>" "<item slot>"
```

### Build guide / meta validation

```text
"<skill>" "<ascendancy>" "<patch>" site:maxroll.gg OR site:mobalytics.gg OR site:pohx.net
site:poe.ninja/builds "<skill>" "<ascendancy>"
site:poewiki.net/wiki "<core mechanic>"
site:poedb.tw/us "<key modifier>"
```

## Reading the sources correctly

The wiki explains **how** a mechanic works. PoEDB / PoE2DB lists **what exists** and the raw values. Path of Building tests **the effect inside a given build**. The trade site shows **what is available right now**. poe.ninja shows **what is popular**. These roles must not be conflated.

## Anti-hallucination rule

If no reliable source is found, say so plainly:

> I could not find a reliable current source for this point. I can describe the hypothesis, but I must not present it as confirmed.

Never fill the gap with manufactured certainty.
