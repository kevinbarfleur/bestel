---
description: Project orientation, reading order, maintenance contract, source-hierarchy summary.
fetch_when: When the user asks how the reference library is structured, what's where, or maintenance/contributing rules.
---

# Path of Exile reference documentation

This folder is the **conceptual knowledge base** that Bestel — and any future maintainer — uses to plan reasoning about Path of Exile 1 / 2.

It is deliberately **structural, not factually current**. It describes how systems interact, how to reason about a build, and where to verify live data. It contains **no precise stats, no current league meta, no tier lists, no prices** — those expire on every patch. The agent verifies live values on the wiki / PoEDB / patch notes BEFORE ANY ANSWER.

## Reading order

The files are numbered to match the agent's reasoning flow:

```
00_README.md                         this file — orientation + maintenance contract
01_source_policy.md                  hierarchy of trusted sources, allowlist, retrieval templates
02_arpg_foundations.md               action-RPG genre conventions (the floor PoE inherits)
03_ggg_design_philosophy.md          GGG's design culture (the priors that explain "why")
04_game_model_poe1_poe2.md           PoE1 / PoE2 mental map and structural divergences
05_build_reasoning_framework.md      ontology of a build + analysis workflow
06_character_stats_and_mechanics.md  attributes, tags, local/global, resources, charges, ailments
07_offence_damage_scaling.md         hit / DoT / ailment scaling, supports, enemy mitigation
08_defence_recovery_survivability.md EHP, max hit, mitigation layers, recovery
09_itemisation_crafting.md           bases, affixes, ilvl, mod tiers, crafting, slot pressure
10_skills_gems_passives_ascendancies.md skills, supports, sockets, Spirit, passives, ascendancies
11_endgame_economy_trade_leagues.md  Atlas / maps / leagues / trade / economy / meta
12_vocabulary_glossary.md            jargon and acronyms (PoE1 + PoE2)
13_retrieval_playbooks.md            ready-to-use search recipes by question type
14_validation_and_failure_modes.md   common errors and pre-answer coherence checks
15_source_registry.md                allowlist of trusted sources + role per source
16_build_methodology_and_creators.md anatomy of a build guide, creator profiles, build-crafting decision framework
maxroll/                             secondary-source catalog of Maxroll articles (URLs + routing hints)
```

A distilled "always-loaded" version of these files lives in `prompts/CORE_KNOWLEDGE.md` — that file is regenerated from the docs above and is what the agent actually reads at runtime.

## How the agent uses these docs

1. **Identify the game first.** Many terms exist in both games with different mechanics (Witch, Ranger, Chaos Orb, Atlas, Map, …). Resolve PoE1 / PoE2 before reasoning. If the active build is loaded, `build.game` is ground truth. Otherwise: ask, or branch the answer.
2. **Use the docs as a springboard.** The conceptual skeleton is here; precise values are not. For any fact (numbers, prices, current meta), the agent must perform a targeted search per `01_source_policy.md` and the playbooks in `13_retrieval_playbooks.md`.
3. **State the patch / version.** Every answer that depends on current state must mention the league or patch version it was sourced against.
4. **Cite real URLs from real fetches.** Never invent a link. Every general claim ends with a `Sources:` section.

## Hierarchy of trusted sources (summary)

The full list lives in `15_source_registry.md`. Tiers, in order of trust:

1. **Canonical** — `pathofexile.com` (forum, patch notes, news, trade, developer docs).
2. **Official wikis** — `poewiki.net` (PoE1), `poe2wiki.net` (PoE2).
3. **Datamined** — `poedb.tw` (PoE1), `poe2db.tw` (PoE2), `repoe-fork.github.io`.
4. **Calculators** — `pathofbuilding.community`, `craftofexile.com`.
5. **Economy** — `poe.ninja`.
6. **Filters** — `filterblade.xyz`.
7. **Trusted creator guides** — `maxroll.gg`, `mobalytics.gg`, `pohx.net`, `poe-vault.com`, `heartofphos.github.io` (Exile Leveling), `poeplanner.com`, `pathofpathing.com`, `poelab.com`, `poe.re`, `exile.re`. Always require explicit patch + author + date; cross-check with the wiki.

**Blocked** (never trust): `pathofexile.fandom.com` (abandoned), `*.fextralife.com`, RMT / boost / currency-selling sites, AI-generated answer aggregators, generic SEO blogs without author/date.

## Maintenance contract

This documentation is regenerated on a small set of triggers:

- **Every major patch.** PoE1 league launches and PoE2 `0.X.0` patches both qualify. Run a quick scan of `04_game_model_poe1_poe2.md`, `11_endgame_economy_trade_leagues.md`, and the affected mechanic files.
- **Every conceptual system change.** When GGG ships a structural rework — Eldritch Altars (PoE1 3.18), Atlas Tree (PoE1 3.17), Spirit (PoE2 0.1), Recombination (PoE2 0.2) — the relevant file gets updated.
- **PoE2 → 1.0.** Full revision of all PoE2-tagged sections.
- ⚠️ **PoE2 0.5 (end of May 2026) overhauls the endgame mapping system.** `11_endgame_economy_trade_leagues.md` and `12_vocabulary_glossary.md` describe the legacy system; both will need a pass when 0.5 ships. Until then the docs flag PoE2 endgame terms with a "subject to 0.5 rework" caveat.

After each documentation revision, regenerate `prompts/CORE_KNOWLEDGE.md` from these files. The core is a distilled snapshot, not a manually edited file.

## What does and does not belong here

**Belongs**:
- Conceptual structure of systems (existence and shape, not magnitude).
- Vocabulary and disambiguation between PoE1 / PoE2.
- Reasoning frameworks, search playbooks, validation checklists.
- Pointers to authoritative sources.
- Stable architectural choices (Rule of Three, pay-no-advantage, trade friction).

**Does not belong**:
- Precise numerical values that change per patch (caps, multipliers, level requirements, league cadence).
- Tier lists, named "best builds," prices.
- Ephemeral meta snapshots.
- Implementation details copied from the wiki — link to the wiki instead.

If a precise number is essential to convey a concept, mark it `[verify on wiki]` rather than freezing the value.

## Languages and naming

Documents are written in English to match the wiki, the trade site, and the game. Game-internal proper nouns (skills, items, ascendancies, keystones, ailments, currencies, league names) are always in English regardless of conversation language.

The agent itself answers in the language the user uses, but lookups always use English names.

## Contributing

When updating these files:
- Keep changes conceptual; resist the urge to "freshen" with current numbers.
- If you add a new doc, slot it into the numbering and update this README's reading order.
- Regenerate `CORE_KNOWLEDGE.md` after any non-trivial change.
- Add a note to the maintenance section if your change should trigger future re-reviews.
