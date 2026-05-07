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
```
