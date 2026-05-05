# 14 — Validation and failure modes

## Failure mode 1: PoE1 / PoE2 contamination

Symptom: the agent advises a PoE1 system to a PoE2 player, or vice versa.

Examples:

- talking about 6-link chest in the context of a PoE2 skill without clarifying;
- recommending a PoE1 item that does not exist in PoE2;
- applying a PoE2 ES / chaos rule to PoE1;
- using poewiki.net instead of poe2wiki.net.

Countermeasure: `game` first. If unknown, ask, or present both branches.

## Failure mode 2: static knowledge used as current truth

Symptom: the agent gives a meta / price / patch recommendation without a search.

Countermeasure: any answer about current state must consult official patch notes, trade, or an economic source. The reference pack contains no prices and no tier lists.

## Failure mode 3: source misuse

Symptom: the agent uses poe.ninja to prove a mechanic, or PoEDB to explain an interaction.

Countermeasure:

- wiki = explanation;
- PoEDB / PoE2DB / RePoE = raw data;
- PoB = build simulation;
- trade = availability / price;
- poe.ninja = trends.

## Failure mode 4: vague build advice

Symptom: "get more life / res / damage".

Countermeasure: every recommendation must include a slot, a stat, a reason, a method, and a trade-off.

## Failure mode 5: ignoring PoB config

Symptom: the agent accepts a DPS number computed with unrealistic charges, shock, rage, flasks, exposure, or enemy settings.

Countermeasure: cite the active conditions. Distinguish mapping uptime from boss uptime.

## Failure mode 6: wrong support recommendation

Symptom: the agent recommends an incompatible or weak support because it only looked at tags.

Countermeasure: verify the support's exact text and the skill's exact text. The PoE1 `Support gem` page explicitly states that tags alone do not determine supportability.

## Failure mode 7: impossible crafting route

Symptom: a route that ignores item level, influence, corruption, prefix / suffix, metacraft restrictions, or mod group.

Countermeasure: before any route, fill in:

```text
Base:
Item level:
Target prefixes:
Target suffixes:
Open affix required:
Crafting method:
State restrictions: influence / fracture / corruption / eldritch / desecrated / rune / soul core
Expected cost:
Trade alternative:
```

## Failure mode 8: forum post treated as canonical

Symptom: an old forum post is taken as final proof.

Countermeasure: GGG posts can be valuable for edge cases, but must be dated and cross-checked with the current wiki and patch notes. Player posts are not sources of truth.

## Failure mode 9: over-answering without a decision

Symptom: the agent gives a full lecture but no recommendation.

Countermeasure: end on a concrete choice:

- "do X first";
- "don't do Y before Z";
- "search this filter";
- "test this change in PoB";
- "this interaction remains unconfirmed".

## Failure mode 10: no uncertainty where source coverage is weak

Symptom: the agent is confident even though the PoE2 wiki page is a stub or the source pre-dates a patch.

Countermeasure: state plainly:

```text
The current page is incomplete / dated / does not cover this interaction. I can give the likely hypothesis, but I will not treat it as confirmed.
```

## Validation rubric

A response is acceptable if:

- the game is identified;
- the build was read where necessary;
- at least one reliable source was consulted for mechanical facts;
- the agent gives concrete numbers or constraints;
- the recommendation is feasible;
- limits are explicit;
- sources are listed.

A response is excellent if:

- it diagnoses the real bottleneck;
- it provides an ordered action path;
- it compares trade vs craft;
- it warns about the trade-off;
- it suggests a validation test;
- it connects one or two non-obvious but mechanically relevant synergies.
