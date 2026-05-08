---
description: PoE2 Weapon Set 1/2 mechanics, set-specific passive points via the Book of Specialization, gem socketing per set, swap timings, common patterns. PoE2-only mechanic, no PoE1 equivalent.
fetch_when: User asks about weapon swapping (PoE2 context), "should I run two weapon sets?", references the Book of Specialization, or has a build that switches weapons mid-fight. Always apply to PoE2 only.
---

# PoE2 — Weapon Sets

> Always check `00_version_pinning.md` for current PoE2 version. Mechanics shifted in 0.3 (Book of Specialization introduction).

This doc is about the **dual-weapon system** — one of PoE2's most distinctive build-design tools. There is no PoE1 equivalent (PoE1's "weapon swap" is a single press; PoE2's is a stateful swap with persistent gems and per-set passives).

The single most useful question this doc answers: *"Should this build use two weapon sets, and how should they be configured?"*

## Concept

PoE2 lets each character carry **Weapon Set 1** and **Weapon Set 2** with:

1. Independent equipped weapons (1H or 2H, off-hand or shield).
2. Independent socketed gems (skills socketed in Set 1 weapons fire only when Set 1 is active).
3. **Per-set passive points** via the **Book of Specialization** (introduced 0.3).

The active set is hot-swappable via a hotkey-bound action.

## Per-set passive points (Book of Specialization)

The **Book of Specialization** is a quest-acquired item that grants passive-skill-tree points which are **only active when a specific weapon set is in use**.

- Allocate Bow nodes to Set 1 (active when Bow is the equipped weapon).
- Allocate Dagger nodes to Set 2 (active when Dagger equipped).
- A node allocated to Set 1 contributes *zero* when Set 2 is active.
- Nodes flagged as Set 1 / Set 2 are visually distinct in the passive tree UI.

This lets a build commit to two distinct weapon paths without overlap penalty.

### Limit

The Book of Specialization grants a **fixed number of "marker" points** (typically lower than your total tree allocation). Most build slots are still global — only the marked subset is per-set. Verify current count via PoE2 wiki.

## Active set switching

- Hotkey-bound (default key bind shown in options menu).
- **Animation cost**: ~250ms swap animation (verify per current version).
- During the swap animation:
  - Skills auto-buffered? Some yes, some no — depends on skill class.
  - Persistent buffs (heralds, auras) don't reset.
  - Charges generally persist across swap.
- Some movement skills handle swaps differently — verify in-game.

## Per-set gem socketing

- Each weapon has its own sockets.
- Skill gems socketed in Set 1 weapons: only fire while Set 1 is active.
- Skill gems socketed in Set 2 weapons: only fire while Set 2 is active.
- Sockets in armours / boots / gloves (non-weapon slots): always active regardless of set.

This lets a build run *different active skills per set* (e.g., bow skills in Set 1, crossbow skills in Set 2).

## Common build patterns

### 1. Buffer set + boss set

- **Set 1**: bow with AoE clear skill (Lightning Arrow + chain support).
- **Set 2**: crossbow with single-target burst (Galvanic Shards or similar).
- Mapping with Set 1, swap to Set 2 for boss / rare encounters.

### 2. Defensive swap

- **Set 1**: 2-hand weapon (offence).
- **Set 2**: 1-hand + shield (defense; block + spell block).
- Set 1 for rapid clear, Set 2 for tough encounters or rare-mod combinations.

### 3. Element pivot

- **Set 1**: fire-damage weapon + fire skills.
- **Set 2**: cold/lightning weapon + alternate-element skills.
- Counters resist-mod maps (e.g., monsters resist fire → swap to cold for that map).

### 4. Combo-builder split (Monk / Huntress)

- **Set 1**: primer-skill weapon (e.g., quarterstaff with Tempest Bell).
- **Set 2**: executor-skill weapon (e.g., quarterstaff with Charged Staff).
- Apply primer with Set 1, swap to Set 2 to consume primer.
- Note: combo state may be lost across swap depending on which game-state is preserved (verify per skill).

### 5. Curse / Mark applicator

- **Set 1**: caster wand + curse-on-hit setup.
- **Set 2**: physical attack setup.
- Apply curse via spell on Set 1, swap to Set 2 for physical damage on cursed target.

## PoB2 representation

PoB2 stores weapon-set assignment via:

- `<Items>` slots flagged with `weaponSet="1"` / `weaponSet="2"` attributes.
- `<Skill>` socket groups associated with the weapon they're socketed in.
- Active-set toggling rebuilds the calc graph for the current set.

When parsing a PoB XML for a dual-set build, the agent should:

1. Note which weapons are flagged Set 1 vs Set 2.
2. Identify which skill groups fire in which set.
3. Recognise which Book of Specialization nodes apply per set.
4. Avoid over-summing DPS — only one set's skills fire at a time.

## Pitfalls (common LLM hallucinations)

1. **Treating both sets as concurrently active** — they're not. Skills in Set 1 don't fire during Set 2.
2. **Adding both sets' DPS together** — PoB will display each set's calc separately; the agent should report the *active set's* DPS, or both labeled.
3. **Missing per-set passive interactions** — Book of Specialization nodes apply per-set; saying "you have +200% damage" without checking which set the user is asking about is wrong.
4. **Treating swap as instant** — there's a real animation. A boss can hit you during the swap window.
5. **Conflating with PoE1 "weapon swap"** — PoE1 weapon swap is a single key, no per-set passives, no per-set skill sockets. Different feature entirely.

## How to reason about weapon-set choice

When user asks "should I run two sets?":

1. **What are the Book of Specialization marker points?** A build with significant per-set divergence (different damage type, different defensive profile) benefits.
2. **What's the build's bottleneck?** If clear is fine and bossing is weak, single-target Set 2 is the right call.
3. **Can the player handle the swap-fluency?** Beginners often perform worse with a swap-style build vs single-set. Default to single-set for new players.
4. **What's the gear cost?** Two weapon sets = double weapon-craft budget. SSF players may not afford both.

## Cross-references

- `06_character_resources_action_speed.md` — action speed fundamentals.
- `09_itemisation_crafting.md` — weapon-base crafting.
- `19_combat_movement_animation.md` — swap-animation timing and feel.
- `20_item_basetype_identity.md` — PoE2 weapon bases per archetype.
- `26_validation_and_self_correction.md` Rule 4 — PoE1 weapon swap vs PoE2 Weapon Sets disambiguation.
- `00_version_pinning.md` — current PoE2 version.
- `01_spirit_economy.md` — Spirit budget changes per weapon-set if weapons grant Spirit.
