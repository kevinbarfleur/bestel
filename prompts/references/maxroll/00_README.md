---
description: Maxroll subfolder orientation - catalog format, routing heuristics (how-to -> getting_started, bosses -> bosses, etc).
fetch_when: When you need an applied step-by-step source and aren't sure which Maxroll catalog to consult.
---

# Maxroll catalogs

This subfolder catalogs the Maxroll.gg articles the agent should consult when it needs **concrete, applied information** beyond the conceptual core in the numbered top-level docs (`01_…` to `16_…`). Every entry lists the URL, what the article covers, and when to route to it.

Maxroll is a high-quality **secondary source**: well-written, regularly updated, and reviewed by experienced creators. It is not the source of mechanical truth — for raw mechanics (formulas, status effects, base stats), prefer `poewiki.net` / `poe2wiki.net`. Use Maxroll for synthesis, applied advice, and step-by-step recipes.

## How the agent uses this folder

1. **Identify the game.** PoE1 catalogs are filenames `poe1_*.md`. PoE2 catalogs (when they ship) will be `poe2_*.md`.
2. **Use the conceptual docs first.** The numbered top-level docs explain the mechanics and vocabulary. Maxroll adds applied steps once the player asks for something concrete.
3. **Pick the right catalog.** Files map roughly to the player's intent: getting started, bosses, currency strategies, crafting recipes.
4. **Open the live URL.** The catalog lists URLs; the agent must `web_fetch` the live page to get current numbers, builds, and meta. The catalog itself is patch-agnostic — it tells the agent *what exists* and *where to look*, not the current state.

## Files

### Path of Exile 1

| File | What it covers | Article count |
|---|---|---|
| `poe1_getting_started.md` | Beginner mechanics, options/key binds, campaign walkthrough, Atlas progression, endgame onboarding. | 21 |
| `poe1_bosses.md` | Pinnacle bosses, Atlas bosses, Conquerors, Guardians, Breachlords, mechanic bosses. | 32 |
| `poe1_currency.md` | League-start strategies, endgame farming strategies, mechanic deep-dives. | 34 |
| `poe1_crafting.md` | Crafting fundamentals + applied step-by-step recipes for specific items. | 35 |

### Path of Exile 2

PoE2 catalogs are not yet present. When they are added, they will follow the same `poe2_*.md` naming and the same entry format.

## Source quality reminder

- Numbers and meta strategies update **per league**. Anything tagged "league start" or "endgame strategy" is meta-dependent and shifts after patches. Always re-fetch and cite the patch.
- Boss mechanics are largely stable, but loot tables and difficulty thresholds can change. Mention the patch version when known.
- For raw mechanics (damage formulas, status effects, item base stats), prefer `poewiki.net` (PoE1) / `poe2wiki.net` (PoE2). Maxroll for synthesis and applied advice.

## Routing heuristics

- "How do I…" / game basics → `poe1_getting_started.md`.
- Specific boss fight, mechanics, or how to access a fight → `poe1_bosses.md`.
- Making currency, league-start direction, mechanic profitability → `poe1_currency.md`.
- Craft a specific item or learn a crafting method → `poe1_crafting.md`.
- Build evaluation, "what makes a good build", choosing a creator to follow → top-level `16_build_methodology_and_creators.md` for the framework + creator URLs.

## Entry format

Every article entry follows:

- **Title** — the article's name on Maxroll.
- **URL** — the canonical link.
- **Topics covered** — what the article actually contains.
- **When to consult** — agent routing hint, written so the agent can match against player intent.
- **Recency notes** (when relevant) — whether the article is patch-stable or meta-dependent.
