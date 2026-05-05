# 05 — Build reasoning framework

## Operational definition of a build

A build is not "a skill plus an ascendancy". It is a coherent assembly of nine blocks:

1. **Core skill package** — main skill, supports, trigger / meta setup, clear skill, single-target skill.
2. **Damage delivery** — hit, ailment, DoT, minion, totem, mine / trap, trigger, retaliation / counter, corpse, projectile, melee, spell.
3. **Scaling axes** — base damage, added damage, conversion, attack / cast speed, crit, DoT multiplier, more multipliers, penetration, exposure, curses, enemy debuffs.
4. **Resource engine** — mana, life cost, reservation, Spirit, charges, cooldowns, rage, flask sustain, leech / recoup / regen.
5. **Defensive stack** — life / ES, resistances, armour, evasion, block, suppression, max resist, physical mitigation, chaos mitigation, avoidance, ailment mitigation, stun / freeze / corrupting blood / bleed handling.
6. **Recovery model** — leech, regen, recoup, flasks, life gained on hit, ES recharge / recovery, guard skills, recharge interruption rules.
7. **Item budget** — required uniques, flexible rare slots, affix pressure, attribute pressure, resistance pressure, socket / support pressure, movement speed, implicits.
8. **Progression context** — campaign, early maps, red maps, pinnacle bosses, SSF, trade league, hardcore, ruthless-like constraints.
9. **Content target** — mapping speed, bossing, sanctum / trial, legion / breach / abyss, delirium, ultimatum, invitation, pinnacle, league mechanic.

The agent must always identify which block(s) are at issue before answering.

## Build diagnosis: priority order

When the user asks "why am I dying?", "how do I improve my build?", or "what is my biggest problem?", follow this order:

1. **Read current numbers** — life, ES, mana, armour, evasion, block, suppression, resistances, chaos resistance, max hits by type, EHP, recovery, ailments, flasks / buffs, main skill DPS.
2. **Identify the content target** — campaign, maps, bosses, league mechanic. A build can be fine for yellow maps and broken for pinnacles.
3. **Find the missing minimum** — uncapped resistance, chaos res, physical max hit, ailment immunity, stun / freeze, recovery, mana sustain, single-target damage, accuracy, crit chance, support mismatch.
4. **Compare against the build's constraints** — never recommend an item that breaks attributes, reservation / Spirit, resistances, sockets, weapon requirements, or a core unique.
5. **Propose an action with a cost** — slot, stat, budget, search string, trade-off.

## Quick diagnosis: hypothesis table

| User symptom | Mechanical hypotheses to check | Sources to consult |
|---|---|---|
| "I'm getting one-shot" | Max hit too low, damage type uncovered, dangerous map mods, armour ineffective vs large hit, low chaos res, enemy crits, no block / suppression. | Defence wiki pages, PoB, map mods, patch notes if PoE2. |
| "My DPS is bad" | Skill mis-supported, wrong scaling tag, low weapon / base damage, low accuracy / crit, unhandled enemy resistance, clear setup used for bossing, wrong PoB config. | Wiki skill / support, PoEDB gem stats, PoB config. |
| "I don't have enough mana / Spirit" | Support cost multipliers, excessive reservation, trigger cost, low regen / leech, insufficient Spirit, weapon set mismatch. | Wiki resource / gem / Spirit, build data. |
| "Which item should I upgrade?" | Slot with low affix value, wrong base, low life / ES, missing res, weapon DPS, support / gem level breakpoints, missing eldritch implicit. | PoB items, PoEDB tiers, trade, Craft of Exile. |
| "Does this guide still work?" | Patch mismatch, nerfed skill / support, changed item, changed league mechanic, changed economy. | Official patch notes, wiki history, guide date / patch. |
| "How do I craft this item?" | Base / ilvl, prefix / suffix target, fractured / influenced / eldritch constraints, expected cost, deterministic step. | PoEDB mods, Craft of Exile, wiki crafting system. |

## Build improvement algorithm

```text
Input: user question + active build + game + league
1. Classify: defence / offence / resource / item / progression / economy.
2. Extract current state: quote at least 2 relevant build numbers.
3. Define target state: what value/state solves the problem for the requested content.
4. Find bottleneck: identify one limiting layer, not ten generic ideas.
5. Search mechanic: wiki page for the bottleneck.
6. Search implementation: item / mod / passive / support in PoEDB or wiki.
7. Check feasibility: slot pressure, budget, build identity, trade availability.
8. Produce path: immediate fix -> medium upgrade -> aspirational upgrade.
```

## What a good recommendation contains

- the **named problem**;
- the **current value**;
- the **verified mechanic**;
- a **concrete solution**;
- the **cost or trade-off**;
- the **next test** in PoB or in game.

Example:

```text
Your chaos max hit is the lowest, not your fire max hit. You sit at a low chaos resistance, so the first defensive upgrade is a ring with chaos resistance, life, and an open elemental resistance. Priority: replace the least useful ring, not the body armour, since the ring slot easily carries chaos + res + life without breaking the skill setup. Verify the mod tier on PoE2DB, then search trade using pseudo total resistance + explicit chaos resistance.
```

## What to avoid

- Suggesting "more life" without slot, stat, tier, or method.
- Recommending a unique without verifying it exists in the right game and current patch.
- Recommending a PoE1 mechanic for a PoE2 build.
- Reading PoB DPS without verifying the configuration: charges, exposure, shock, enemy type, distance, rage, flask uptime.
- Forgetting the content target.
- Confusing poe.ninja popularity with optimal solution.
- Giving a crafting route without checking item level, mod group, prefix / suffix, and tags.

## Recommended output for a build analysis

```markdown
### Diagnosis
<1–2 paragraphs with build numbers and central problem>

### Why this is the problem
<verified mechanic + source>

### Priority fix
<one concrete change, slot by slot if needed>

### Trade-off
<what the player loses or must monitor>

### Next test
<what to verify in PoB / trade / in game>

Sources:
- ...
```
