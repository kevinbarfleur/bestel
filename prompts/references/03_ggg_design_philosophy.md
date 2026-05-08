---
description: GGG design priors — pay-no-advantage, the Vision, trade friction, league cycle, power-creep cadence, PoE2 as sibling.
fetch_when: When the user asks why GGG made a choice, evaluates monetisation/trade design, or compares PoE2 to PoE1 strategically.
---

# 03 — GGG design philosophy

This document captures **what makes Grinding Gear Games' design choices coherent**. It is the cultural / philosophical layer that explains *why* PoE looks the way it does. The agent should hold these as priors when interpreting balance changes, league design, monetisation, or community responses.

GGG (Auckland, NZ) was founded by Chris Wilson, Jonathan Rogers, Erik Olofsson, and others. Wilson and Rogers have been the public-facing design voices since the 2006 conception. The studio has cultivated a recognisable culture around "the Vision" — a label community uses (sometimes affectionately, sometimes critically) for GGG's stubborn adherence to specific design tenets.

When a player asks "why did GGG do X?" the answer almost always traces back to one of the seven principles below.

## 1. Pay-no-advantage

GGG monetises **only via cosmetics, stash tabs, and supporter packs**. There is no XP boost, no power-up, no gameplay shortcut sold for money. This is non-negotiable for the studio.

Quotes (Wilson and Rogers, 2014 Polygon / Gamasutra interviews):

- "We've been careful when designing the game so there's no paying for game content or advantage in the game. We've purposefully divorced any game mechanics from the monetization."
- "As soon as you tie your game design to your monetization, it affects it. If you can buy a boost of experience, how is that separating gameplay and monetization?"

**Consequences for the agent:**
- No legitimate guide will ever say "pay for X to unlock Y." If a user reports such a thing, it is RMT, not GGG.
- Stash tabs improve quality-of-life (sorting currencies, fragments, divcards). They do not improve build power.
- Supporter packs include MTX credit + cosmetics; the financial relationship is patronage, not transaction.
- The agent should never recommend buying credit or items as a build solution.

## 2. The "Vision"

A recognisable cultural artefact: GGG publishes **developer manifestos** explaining design decisions in long-form prose. The most-cited examples:
- *Trade Manifesto* (Wilson, 2017): explains why trade is intentionally friction-rich.
- *Item / drop philosophy*: items must be tradeable, items must matter, items must be earned.
- *On the Atlas, on Maps, on Currency*: many over the years.

The Vision tenets that recur:
- **Items must matter.** Items are the primary measurement of progress, more than character level. They must be tradeable so they can have value.
- **Trade must be friction-rich.** Easy trade (auction-house style) reduces the number of in-game character improvements, requires drop-rate cuts, and amplifies disparity between high-engagement and casual players. GGG accepts trade friction as a feature, not a bug.
- **Drops are the primary motivator.** "Knowing that a monster could drop something that improves your character is a great motivator for playing one more level."
- **Long iterative gear progression.** Players should slowly upgrade rather than buy to spec. The journey of gearing up is a substantial part of the gameplay.
- **Power should escalate but be earned.** Endgame characters are dramatically more powerful than early-game ones, but only after substantial investment.

When a community thread complains "the Vision strikes again," it usually means GGG made a balance call that prioritised one of these tenets over short-term player convenience. The agent should not echo player frustration — it should recognise the underlying design rationale.

## 3. Trade friction is intentional

A specific corollary of (2), worth singling out because of how often it surfaces.

GGG explicitly rejected the auction-house model. Their stated reasons:
- Easy trade reduces character improvements per session.
- Easy trade requires lower drop rates to maintain economy balance.
- Easy trade amplifies disparity (heavy traders get hundreds of Exalted Orbs in the time a casual gets a few).

The trade flow is therefore **intentionally manual**: search the trade site, whisper the seller, party-invite, trade window, hope the seller is afk-aware. The friction is a deliberate game-design lever.

**Consequences for the agent:**
- Don't suggest "just buy it on the auction house" — the user must navigate trade2 / trade.
- Don't suggest "use a trade bot" — RMT-adjacent; out of scope.
- Currency Exchange (Faustus / Alva) is an in-game alternative for currency-currency trades; legitimate.
- For SSF questions, the trade context is irrelevant; treat the question as "can the player self-craft / self-find this?"

## 4. Free-to-play with a hobbyist supporter base

PoE1 launched (2013) as a fully free-to-play product, no gates, no time-limited content. Revenue comes from:
- **Stash tabs** (one-time purchase, cosmetic + storage).
- **Microtransaction (MTX) cosmetics** (skins, weapon effects, hideout decorations, pets).
- **Supporter packs** (premium tiers with credit + exclusive forum titles, portraits, sometimes physical merchandise).

This model only works because a small percentage of players treat PoE as a hobby and willingly pay $50, $200, even $1000 for supporter packs. The development team is funded by patronage, not by squeezing cents out of casuals.

**Consequences:**
- The agent should never assume the player has paid anything. The full game is free.
- "Premium quad stash" is the most-recommended QoL purchase; don't push it as essential.
- Game balance is not influenced by "what would make players pay more." It is influenced by GGG's internal design loop.

## 5. The Rule of Three (cultural pattern, not a hard rule)

A recurring observation made publicly by Chris Wilson: **three strong choices beat two**. Two-option choices feel binary; three-option choices invite synthesis. The agent will see this pattern in the architecture:

- Three damage types in the elemental triangle (fire / cold / lightning).
- Three primary attributes (STR / DEX / INT).
- Historically three ascendancies per class (PoE1; PoE2 follows the same in most cases).
- Many three-way interactions in itemisation (Shaper / Elder / Conqueror; Searing Exarch / Eater of Worlds; etc.).

This is **not a literal balance rule** — GGG breaks it when it suits the design. Treat it as a cultural prior: when GGG adds a new system, it often comes in threes.

## 6. League cycle as continuous expansion

GGG ships **major content updates on a recurring cadence**. The exact period varies by patch — verify against current GGG announcements before quoting timing. The pattern:

- Each new league introduces a temporary mechanic + balance changes + new uniques + endgame iterations.
- Old league mechanics are sometimes integrated into core, sometimes removed, sometimes rotated through a "Legacy" / "Phrecia" / "Ruthless" variant.
- A "league reset" wipes characters back to Standard / Hardcore at the end of each league. The new league starts a fresh economy.
- Hotfixes are infrequent and conservative. Major balance changes usually wait for the next patch / league launch.

**Consequences for the agent:**
- Build advice has a shelf life. A Ngamahu's Flame build optimised in 3.20 may be dead in the current league.
- "Patch X says Y" must be cited with an explicit version number.
- Players migrating from a previous league may carry stale assumptions; the agent should flag patch-relevant changes.
- "When does the next league start?" → search for the latest official announcement; never guess.

## 7. Power creep is allowed, then recalibrated

GGG explicitly allows **power creep within a league** and then recalibrates major outliers in the following league. The cycle:

1. New league introduces strong items, skills, league-mechanic loot.
2. Players discover dominant builds. Top builds reach absurd numbers.
3. End of league: GGG reviews telemetry + community discourse.
4. New league nerfs / re-tunes the worst offenders, often substantially.

This is a feature, not a bug. The "broken build" of last league is part of the lore; the rebalance is the cycle's payoff.

Wilson and Rogers have also publicly acknowledged that **the game tends towards speed-clear builds over thoughtful builds**, because "more maps = more loot." PoE2 was designed in part to push back against this drift via slower combat, dodge roll, more deliberate boss fights — though the same speed-clear pressure is already emerging in PoE2 endgame.

**Consequences for the agent:**
- Don't recommend a build because "it was best last league" without a current source. Major nerfs are plausible.
- When a user asks "is this skill broken?" the answer is often "currently yes, and it will likely be tuned next patch."

## 8. Ruthless and the meta-statement it makes

GGG ships a **Ruthless** game variant alongside the standard mode. Ruthless drops fewer items, fewer currencies, slower XP, no trade. It is intentionally punishing.

Ruthless exists for two reasons:
- A subset of the playerbase wants the original D2-feel of scarcity-driven progression.
- It serves as a **design mirror**: by cutting the standard game's affordances, GGG can study what is actually load-bearing in the experience.

Ruthless is a niche mode. The agent should treat questions about it as a separate context — never assume Ruthless rules apply by default.

## 9. PoE2 as a coherent re-imagining

PoE2 is **not a sequel that replaces PoE1**. GGG explicitly positioned it as a parallel game:
- PoE1 stays in active development with leagues.
- PoE2 ships separately, with its own leagues, its own economy, its own client.
- A user can have both games installed and active.

PoE2's design directives, as articulated by GGG in pre-launch interviews and in 0.x patch notes:
- Slower, more deliberate combat (active dodge roll, hit-react, longer boss fights).
- Less screen-clear pressure, more depth-per-encounter.
- Gem-as-item rather than socketed-gems-in-equipment, reducing socket/link friction.
- Spirit as a dedicated reservation resource, separating buff economy from mana economy.
- Procedural endgame (Atlas + Waystones), with iterative reworks (the 0.5 mapping rework end of May 2026 is the next major change).

PoE2 in Early Access status. The agent must treat PoE2 information as **more volatile** than PoE1 information. Patch notes are the only reliable source for current state.

## 10. How GGG communicates with the playerbase

Channels (the agent should know where to look):
- **Patch notes** on `pathofexile.com/forum/view-forum/2212` — absolute truth on current state. Always the first stop when patch changes are at issue.
- **News / announcements** on `pathofexile.com/forum/view-forum/40` — manifestos, league reveals, design rationale.
- **ExileCon** (occasional in-person event) — major roadmap reveals, often with developer Q&A.
- **Baeclast and other community podcasts** featuring Wilson / Rogers — long-form design conversations, valuable for context but not citable as canonical (they pre-date the corresponding patch's actual implementation).
- **Reddit / Twitter / forum discussions** featuring developer accounts — informal but sometimes load-bearing for clarification.

GGG **rarely tweets balance teasers**. They prefer the manifesto + patch-notes pattern. If a user reports a "rumour about a buff next league," the agent should treat it as unverified until a manifesto or patch notes confirm.

## Application to agent reasoning

When parsing a user's question, hold these priors:
- **Pay-no-advantage** — the answer never involves spending real money to fix a build problem.
- **Vision-coherence** — if a balance choice seems weird, it usually traces to one of the manifestos.
- **Trade friction** — assume the user must do manual trade unless told otherwise, and respect SSF as a first-class context.
- **Patch volatility** — quote a patch number with every factual claim about balance, mechanics, or economy.
- **Power creep cycle** — treat "current meta" as a snapshot, not a permanence.
- **PoE1 ≠ PoE2** — never carry an assumption from one to the other without verifying.

The agent does **not** lecture the player on philosophy. The agent uses these priors silently, to choose the right question to verify and the right source to cite.

## Sources informing this document

- Gamasutra / Game Developer, *The mechanics and ethics of free-to-play in Path of Exile* (2014): `https://www.gamedeveloper.com/business/the-mechanics-and-ethics-of-free-to-play-in-i-path-of-exile-i-`
- Polygon, *How is "ethical" free-to-play Path of Exile faring?* (2014): `https://www.polygon.com/2014/2/28/5451410/how-is-ethical-free-to-play-path-of-exile-faring/`
- Path of Exile forums, *Trade Manifesto* (2017): `https://www.pathofexile.com/forum/view-thread/2025870`
- AltChar, *Path of Exile philosophy is keeping MTX and main game separate* (2019): `https://altchar.com/game-news/path-of-exile-philosophy-is-keeping-mtx-and-main-game-separate-a7eiu8A4opq0`
- Prima Games, *Path of Exile: Delirium — An interview with Chris Wilson* (2020): `https://primagames.com/featured/path-of-exile-delirium-an-interview-with-grinding-gear-games-chris-wilson`
- Baeclast #61, *Chris Wilson on Item Philosophy* (2020): `https://www.youtube.com/watch?v=FpIB2dMzNu8`

These are background reading for the agent's reasoning. Citations to the user must always come from the live wiki, current patch notes, or current trade site.
