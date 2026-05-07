---
description: Concrete URL allowlist by tier (canonical, wikis, datamined, calculators, economy, creator guides) + explicit blocklist.
fetch_when: When constructing a citation, choosing between sources, or verifying that a domain is on the allowlist before fetching.
---

# 15 — Source registry

This registry guides the agent toward the right sources. URLs must be re-verified at runtime when the answer depends on current state.

## Official / canonical

| Source | URL | Usage |
|---|---|---|
| Official PoE patch notes forum | https://www.pathofexile.com/forum/view-forum/patch-notes | PoE1 versions, hotfixes, current patch — search the latest thread at answer time. |
| Official PoE2 patch notes (forum) | https://www.pathofexile.com/forum/view-forum/2212 | Shared forum hosting PoE2 patch notes — search the most recent thread. |
| Developer Docs | https://www.pathofexile.com/developer/docs | API, OAuth, resources, policies, rate limits, caveats. |
| Official trade static data | https://www.pathofexile.com/api/trade/data/static | Static trade data such as currencies / categories. |
| Official trade stats data | https://www.pathofexile.com/api/trade/data/stats | Stat filter IDs and text. |

## Wikis

| Source | URL | Usage |
|---|---|---|
| PoE Wiki main | https://www.poewiki.net/wiki/Path_of_Exile_Wiki | Entry point for PoE1 mechanics / items / skills / passives. |
| PoE2 Wiki main | https://www.poe2wiki.net/wiki/Path_of_Exile_2_Wiki | Entry point for PoE2 mechanics / items / skills / passives. |
| PoE1 Resistance | https://www.poewiki.net/wiki/Resistance | Resistance mechanics. |
| PoE1 Armour | https://www.poewiki.net/wiki/Armour | Armour mechanics and limits. |
| PoE1 Spell suppression | https://www.poewiki.net/wiki/Spell_suppression | Suppression chance and prevention. |
| PoE1 Passive skill | https://www.poewiki.net/wiki/Passive_skill | Passive tree model. |
| PoE1 Support gem | https://www.poewiki.net/wiki/Support_gem | Support compatibility and linked gems. |
| PoE1 Crafting | https://www.poewiki.net/wiki/Crafting | Crafting overview. |
| PoE1 Rarity | https://www.poewiki.net/wiki/Rarity | Rare / magic affix budget. |
| PoE1 Item level | https://www.poewiki.net/wiki/Item_level | Item level and affix availability. |
| PoE1 Metamod | https://www.poewiki.net/wiki/Metamod | Meta-crafting rules. |
| PoE1 Essence | https://www.poewiki.net/wiki/Essence | Essence crafting. |
| PoE1 Fossil | https://www.poewiki.net/wiki/Fossil | Fossil / resonator crafting. |
| PoE1 Map | https://www.poewiki.net/wiki/Map | Map item / endgame basics. |
| PoE1 Atlas of Worlds | https://www.poewiki.net/wiki/Atlas_of_Worlds | Atlas progression. |
| PoE2 Gem | https://www.poe2wiki.net/wiki/Gem | Skill / support / meta / spirit gem structure. |
| PoE2 Gem socket | https://www.poe2wiki.net/wiki/Gem_socket | Gem sockets and supports. |
| PoE2 Spirit | https://www.poe2wiki.net/wiki/Spirit | Spirit resource and persistent skills. |
| PoE2 Passive Skill Tree | https://www.poe2wiki.net/wiki/Passive_skill_tree | Passive tree, attributes, weapon specialisation. |
| PoE2 Ascendancy class | https://www.poe2wiki.net/wiki/Ascendancy_class | Ascendancy trials and points. |
| PoE2 Modifier | https://www.poe2wiki.net/wiki/Modifier | Prefix / suffix budget and modifier basics. |
| PoE2 Item | https://www.poe2wiki.net/wiki/Item | Rarity and item structure. |
| PoE2 Crafting | https://www.poe2wiki.net/wiki/Crafting | PoE2 crafting overview. |
| PoE2 Armour | https://www.poe2wiki.net/wiki/Armour | Armour mechanics. |
| PoE2 Evasion | https://www.poe2wiki.net/wiki/Evasion | Evasion mechanics and cap. |
| PoE2 Atlas | https://www.poe2wiki.net/wiki/Atlas | Infinite Atlas, waystones, completion / failure. |
| PoE2 Precursor tablet | https://www.poe2wiki.net/wiki/Precursor_tablet | Map mechanics added via tablets. |

## Datamined / raw data

| Source | URL | Usage |
|---|---|---|
| PoEDB Modifiers | https://poedb.tw/us/Modifiers | PoE1 raw modifier data, tiers, tags. |
| PoEDB Craft Modifiers | https://poedb.tw/us/Craft_Modifiers | Bench / craft modifier data. |
| PoE2DB Modifiers | https://poe2db.tw/us/Modifiers | PoE2 raw modifiers / currency / mod data. |
| PoE2DB Items | https://poe2db.tw/us/Items | PoE2 item base lists. |
| PoE2DB Gems | https://poe2db.tw/us/Gem | PoE2 gem data. |
| RePoE GitHub | https://github.com/repoe-fork/repoe | JSON data for tool developers. |
| RePoE PoE2 exports | https://repoe-fork.github.io/poe2/ | PoE2 exported data files. |

## Calculators / economy

| Source | URL | Usage |
|---|---|---|
| Craft of Exile | https://www.craftofexile.com/ | PoE1 craft simulation and odds. |
| poe.ninja | https://poe.ninja/ | Economy, build popularity, trends. |

## Trusted creator guides (catalogued in `maxroll/`)

The Maxroll articles cataloged in `maxroll/` are pointers, not citations. Always re-fetch the live URL to get current numbers and meta. Build-creator profiles for the deep-dive layer are in `16_build_methodology_and_creators.md`.

| Source | URL | Usage |
|---|---|---|
| Maxroll PoE1 hub | https://maxroll.gg/poe | Entry point for PoE1 articles, builds, and league strategies. |
| Maxroll PoE2 hub | https://maxroll.gg/poe2 | Entry point for PoE2 articles, builds, and league strategies. |
| Maxroll Getting Started (PoE1) | https://maxroll.gg/poe/category/getting-started | Beginner mechanics, campaign, Atlas onboarding (cataloged in `maxroll/poe1_getting_started.md`). |
| Maxroll Bosses (PoE1) | https://maxroll.gg/poe/category/bosses | Pinnacle / Atlas / Conqueror / Guardian / Breachlord guides (cataloged in `maxroll/poe1_bosses.md`). |
| Maxroll Currency (PoE1) | https://maxroll.gg/poe/category/currency | League-start strategies, farming strategies, mechanic profitability (cataloged in `maxroll/poe1_currency.md`). |
| Maxroll Crafting (PoE1) | https://maxroll.gg/poe/category/crafting | Crafting fundamentals + applied recipes (cataloged in `maxroll/poe1_crafting.md`). |
| Mobalytics PoE1 hub | https://mobalytics.gg/poe | Build creator hub (Pohx, Ruetoo, Fubgun and others). |
| Mobalytics PoE2 hub | https://mobalytics.gg/poe-2 | PoE2 build creator hub. |
| Pohx Wiki | https://www.pohx.net/ | Reference site dedicated to Righteous Fire (PoE1 Chieftain). |

## Blocked / degraded sources

- Fandom Path of Exile wiki — do not use as a primary source.
- RMT / currency sellers / boosting services.
- Generic SEO guide pages without date / patch / author.
- AI answer aggregators (Perplexity summaries, Bing chat, etc.) — go to the source they cite instead.
- Unverified forum posts.
