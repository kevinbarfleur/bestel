---
description: Per-question-type retrieval recipes (mechanic, build, DPS gap, survival, mod, item, crafting, trade, patch, guide validation).
fetch_when: When unsure of the right search strategy for a question; routes you to the matching playbook + relevant reference + Maxroll catalog.
---

# 13 — Agent retrieval playbooks

## Universal pre-flight

Before any non-trivial answer:

```text
1. Game? PoE1 / PoE2 / unknown.
2. Current build available? If yes, read it first.
3. Question type?
4. Patch sensitivity?
5. Required source tier?
6. What exact English entity names are needed for search?
7. What would make the answer wrong?
```

## Playbook: mechanic explanation

Use when: "how does X work?", "explain X", "does X interact with Y?".

```text
1. Resolve game.
2. Search wiki page for X.
3. Read mechanics / caps / interactions / version history.
4. If X has numbers, search PoEDB / PoE2DB.
5. If X recently changed, search official patch notes.
6. If Y is named, repeat for Y.
7. Answer with exact rule, edge cases, practical consequence, sources.
```

Search examples:

```text
site:poewiki.net/wiki "Spell suppression"
site:poe2wiki.net/wiki "Spirit"
site:poedb.tw/us/Modifiers "maximum Chaos Resistance"
site:poe2db.tw/us/Modifiers "Armour Break"
```

## Playbook: build diagnosis

Use when: "look at my build", "why am I dying?", "what should I upgrade?".

```text
1. Read active build data.
2. Identify game from build.
3. Quote concrete stats: life / ES, res, max hits, DPS, recovery, supports.
4. Pick one bottleneck.
5. Verify the relevant mechanic on wiki.
6. Verify a concrete mod / item / passive candidate on wiki / PoEDB.
7. Give one immediate fix and one optional next step.
```

Output must include a trade-off. A fix that breaks attributes / resistances / resource setup is not a fix.

## Playbook: DPS gap diagnosis

Use when the user's complaint is "my DPS is too low" or "I expected more damage."

```text
1. Identify the delivery type (hit / DoT / ailment / minion / trigger).
2. Read the active scaling axes the build leans on.
3. Find the largest available "more" multiplier currently missing.
4. Verify support compatibility by exact text, not only by tag.
5. Check enemy mitigation: resistance, exposure, curse, penetration, armour break.
6. Identify the slot or passive that delivers the largest gain per cost.
7. Quantify the expected change and name the next test in PoB.
```

## Playbook: survival gap diagnosis

Use when the user's complaint is "I'm dying" or "this build feels too fragile."

```text
1. Identify the killing damage type and delivery (hit / DoT / ailment / one-shot / sustained).
2. Read the build's max hit by type.
3. Read recovery under boss conditions, not just standing still.
4. Find the weakest defensive layer (avoidance / mitigation / recovery / hit pool / ailment coverage).
5. Verify the relevant cap on the wiki for the current patch.
6. Pick the slot or passive that fills the gap with the lowest sacrifice.
7. Name the test the user must run to confirm the layer is actually fixed.
```

## Playbook: mod / affix lookup

Use when the user names a stat phrase ("I want chaos res on rings", "where can I get +1 to all skill gems?").

```text
1. Resolve game.
2. Locate the mod text on PoEDB / PoE2DB.
3. Identify prefix / suffix and required item level.
4. Identify the base types that can roll it.
5. Identify any influence / fracture / soul-core / desecrated / corrupted constraint.
6. Identify the deterministic crafting paths if any.
7. Compare against trade prices for ready-rolled items.
```

## Playbook: item lookup

Use when: "what does item Y do?", "is this item good?", "can I use Y?".

```text
1. Resolve game.
2. Search the exact item name on the correct wiki.
3. If stats / tier / drop are needed, search PoEDB / PoE2DB.
4. If price / current availability is needed, search trade / poe.ninja.
5. Explain the item role: build-enabling, stat stick, leveling, boss drop, chase, trap.
6. Compare with the rare alternative if relevant.
```

## Playbook: crafting

Use when: "how do I craft X?", "best way to make this?", "what base / ilvl?".

```text
1. Define the target item in prefixes / suffixes.
2. Resolve base and item level.
3. Search mod pool on PoEDB / PoE2DB.
4. Search the crafting method wiki page.
5. If PoE1 and odds matter, use Craft of Exile.
6. Explain the route: base -> first step -> lock / protect -> finish -> failure modes.
7. Compare with buying.
```

Never skip item level and prefix / suffix classification.

## Playbook: trade / price check

Use when: "how much is this worth?", "what should I buy?", "find me an item".

```text
1. Resolve league and game.
2. If a live price is needed, query official trade / trade2, or ask for a trade URL if the tool can't access it.
3. Use pseudo filters for broad values; explicit filters for mandatory mods.
4. Check online / recent listings if possible.
5. Provide a filter-relaxation order.
```

Do not use poe.ninja as the sole price source for rare items. It is better for currency, uniques, and macro trends than for exact rare valuation.

## Playbook: current patch / league

Use when: "current league", "latest patch", "did this get nerfed?".

```text
1. Search the official forum patch notes and news.
2. Prefer official over wiki and guides.
3. Extract the exact version and date.
4. Only then compare old / new mechanics.
5. Mention uncertainty if patch notes do not cover the specific interaction.
```

## Playbook: guide validation

Use when: "is this guide still good?", "should I follow this Maxroll / Mobalytics / Pohx build?".

```text
1. Extract guide patch / date / author.
2. Resolve game and league.
3. Search current patch notes for skill / items / ascendancy.
4. Search wiki / PoEDB for core mechanics.
5. If possible, compare the guide's PoB to the user's PoB.
6. Verdict: valid / outdated / partially valid / needs changes.
```

## Playbook: PoE1 vs PoE2 ambiguity

If the user says "Path of Exile" without specifying the game and names an entity that exists in both games:

```text
1. Search both only if the answer can stay concise.
2. Otherwise ask which game.
3. If a build is loaded, use build.game.
4. Never merge mechanics unless verified in both games.
```

## Playbook: applied / step-by-step questions

Use when the player asks for something concrete and procedural: a campaign walkthrough, a boss strategy, a league-start farming plan, a specific crafting recipe, gearing tier-by-tier.

```text
1. Resolve the game and patch.
2. Open the relevant catalog in `maxroll/` (poe1_getting_started / poe1_bosses / poe1_currency / poe1_crafting).
3. Match the player's intent to a cataloged article and fetch its URL live.
4. Cross-check the live article's patch tag against the current patch.
5. Translate the article's procedure into the player's build / budget context.
6. Cite the live URL plus any wiki / PoEDB sources used to verify mechanics.
```

For build-evaluation or "is this guide good" questions, route to `16_build_methodology_and_creators.md` for the framework + creator URLs to verify the live state of the build.

## Retrieval output checklist

Before the final answer:

```text
- Did I cite sources I actually used?
- Did I avoid Fandom / RMT / SEO sources?
- Did I distinguish PoE1 / PoE2?
- Did I include one actionable next step?
- Did I state uncertainty where source coverage is weak?
- Did I avoid overfitting to stale patch / meta data?
- Did I emit calculated numbers without an engine call? (engine-trust)
- Did I report cached data without flagging its age? (staleness)
```

## Playbook: offline game-data lookup (`repoe_lookup`)

Use when: "what mods can roll on X base?", "what bases drop unique Z?", "what is the level-20 stat block for gem Y?", "show me the stat translations for `+# to Maximum Life`", "what implicit does Stellar Amulet have?".

```text
1. Resolve game (poe1 / poe2).
2. Pick category:
   - mods               (affixes — domains, types, item-class restrictions, weights)
   - base_items         (item bases — implicits, requirements, drop level)
   - gems               (skill / support gems — tags, stats, level scaling)
   - uniques            (unique items — explicits, base, source)
   - cluster_jewels     (PoE1 only)
   - essences           (PoE1 only)
   - fossils            (PoE1 only)
   - stat_translations  (raw stat id ↔ formatted text)
3. Prefer `id` for exact lookups (e.g. id=`Metadata/Items/Amulets/AmuletStellar`).
4. Use `name` for fuzzy token-overlap (e.g. name="Stellar Amulet", limit=5).
5. Output ships `source: bundled|refreshed` + `fetched_at` — surface the
   age in the answer when it matters (>30d triggers a warning automatically).
```

Examples:

```text
repoe_lookup game=poe1 category=base_items name="Stellar Amulet"
  → Metadata/Items/Amulets/AmuletStellar with implicit + drop level.

repoe_lookup game=poe1 category=mods name="maximum life suffix"
  → top-K matching mods, each with domain / required level / item-class weights.

repoe_lookup game=poe1 category=uniques id="Metadata/Items/Armours/Helmets/HelmetUnique12"
  → exact unique JSON record (Headhunter-tier item).

repoe_lookup game=poe1 category=stat_translations name="maximum life"
  → trade-stat ↔ display string mapping.

repoe_lookup game=poe2 category=cluster_jewels …
  → explicit "not available for poe2" error. Fall back to `wiki_cargo`.
```

PoE2 currently bundles only `mods`, `base_items`, and `uniques`. The
others return "not available for poe2" — fall back to `wiki_cargo` or
`wiki_parse` against `poe2wiki.net` / `poe2db.tw`.

`repoe_lookup` is offline-first: the bundled snapshot ships with the app,
the daily refresh task tops it up against `https://repoe-fork.github.io/`.
A network outage does NOT break the tool — it just keeps returning the
last refreshed (or bundled) data with `source: bundled`.

## Playbook placeholders for upcoming tools (Sprint 2+)

These tools are planned but not yet shipped. When the agent receives a question that *would* be served by them, fall back to the closest existing tool and flag the gap.

### `pob_calc` (active — Sprint 2 ✅)

Use when: "what's my real DPS?", "EHP against fire hits?", "max hit by element?".

```text
1. Confirm a build is loaded (`get_active_build`).
2. Pick category (offence / defence / charges / reservation / ailments / all).
3. Optional `skill_index` for non-default skill group.
4. Optional `calcs` overrides: enemyIsBoss, usePowerCharges, useFrenzyCharges,
   useEnduranceCharges, forceBuffOnslaught, multiplierImpaleStacks, useFlask1..5.
5. Default profile when the user does not specify: enemyIsBoss=true (Pinnacle),
   charges up. State that default explicitly.
6. ALWAYS surface the Calcs echo from the response — two PoBs with identical
   gear can show 10× DPS swings purely from Calcs.
7. If the engine number conflicts with `<PlayerStat>` cache from get_active_build
   by more than ~5%, trust the engine and flag the cache as stale.
```

Examples:

```text
pob_calc category=offence calcs={"enemyIsBoss":true,"usePowerCharges":true,"useFrenzyCharges":true}
  → TotalDPS, CombinedDPS, FullDPS, AverageHit, ailment DPS — against pinnacle, charges up.

pob_calc category=defence calcs={"enemyIsBoss":true}
  → EHP + MaxHitFire/Cold/Lightning/Phys/Chaos against pinnacle.

pob_calc category=all
  → entire PoB output table (use sparingly — large).
```

If the user is on macOS or Linux, `pob_calc` returns "engine not bundled for
this platform" — fall back to `<PlayerStat>` cache from `get_active_build`
with explicit staleness disclaimer. Sprint 2 ships Windows-only.

### `wiki_sqlite` — Sprint 4 (mirror)

Use when: any wiki-cargo query that should be near-instant and offline-resilient.

```text
1. Drops in transparently — `wiki_cargo` queries SQLite first, falls back to live Cargo on miss.
2. No new API surface.
```

**Until shipped**: existing `wiki_cargo` works against live Cargo with 1h TTL cache.
