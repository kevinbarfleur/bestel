---
description: Conceptual entries on how PoE has historically reworked X. Helps temporal reasoning when a user references a 3.X / 0.X mechanic that may have changed.
fetch_when: User references a mechanic with "back in" / "since" / "before X.Y" framing, or you're reasoning about whether a piece of advice from an old guide still applies; whenever you're about to parrot a forum / video / Maxroll claim that might predate a major rework.
---

# 24 — Patch history meta

This doc is **not a per-patch changelog**. For exact patch notes, fetch
`pathofexile.com/forum/view-forum/patch-notes` (PoE1) or the PoE2 forum
equivalent. This doc captures **conceptual reworks** so the agent can
reason about temporal context without reading every patch.

The single most useful question this doc answers: *"Is the advice in this
guide still valid?"*

## Heuristic — when to suspect rot

A guide / forum post / video should trigger temporal-rot suspicion if any of:

- **Posted more than 2 leagues ago** (~6+ months for PoE1, ~8+ months for PoE2).
- **References a system listed below as reworked** without a date stamp.
- **Cites numbers** (cap %, multiplier %, threshold) that haven't been verified against the current wiki / patch notes.
- **Author hasn't posted updates** during the latest league (creator-account dormant).

When suspicion fires: cross-reference the live wiki page + most recent patch notes before treating the guide's claim as canonical.

---

## PoE1 — major conceptual reworks

Listed chronologically by patch number with conceptual delta. Patch notes are the authoritative source — this is the navigation map.

### Auras → reservation → aura effect

- **Pre-3.10**: auras cost flat mana to cast, reduced max mana. "Reduced Mana cost of Skills" mods.
- **3.10 (Delirium)**: replaced with reservation %. Enlighten support reduces reservation. New "Reduced Mana Reserved" mod pool.
- **3.18 (Sentinel)**: reservation efficiency rework. "Reduced Reservation Efficiency" became "Aura Effect" framing on most sources. Multiplicative interactions changed.
- **Modern (3.20+)**: aura effect modifier stacks additively with cluster-jewel `Aura Effect` notables.
- **Rot trigger**: any guide saying "+X% Reduced Mana Reserved" instead of "Aura Effect" predates 3.18.

### Flasks → Instilled → permanent buffs

- **Pre-3.13**: utility flasks consumed charges, manual activation. Flask piano was the meta.
- **3.13 (Ritual)**: Pathfinder rework + flask passives consolidated.
- **3.21 (Crucible)**: Instilled Orb craft system added (auto-trigger flasks on conditions).
- **3.25 (Settlers)**: utility flasks made *permanent* — passive buff once equipped, no more flask piano. Manual activation only for Life flasks + Mana flasks.
- **Rot trigger**: any guide describing "flask piano" / "manual flask activation for Quicksilver" is pre-3.25.

### Cluster jewels

- **3.10 (Delirium)**: introduced. Large (8-12 nodes), Medium (2-3 nodes), Small (1 node). Notable rolls + small-passive bonuses.
- **3.14 (Ultimatum)**: re-tuned. Several notables added / removed.
- **3.16 (Scourge)**: another rebalance. Cluster jewel small-passive minor bonuses standardised.
- **Modern (3.20+)**: relatively stable. Crafting via Harvest reforge "with X tag" for targeted notables.
- **Rot trigger**: pre-3.14 cluster-jewel guides reference notables that don't exist anymore (e.g., "Pure Commander" got reworked).

### Atlas tree

- **Pre-3.13**: sextants applied to map watchstone slots. Region atlas.
- **3.13 → 3.16**: region atlas + watchstones + voidstones progression.
- **3.17 (Siege of the Atlas)**: Eater of Worlds + Searing Exarch as endgame split. Atlas-wide voidstones.
- **3.20 (Forbidden Sanctum)**: Atlas Passive Tree introduced. Sub-trees per league mechanic.
- **3.25 (Settlers)**: Tablets replaced sextants. Towers as tablet-application points.
- **Modern (3.27+)**: Atlas tree is the dominant farming-strategy artifact. Specialise sub-trees per session.
- **Rot trigger**: any guide saying "allocate sextants to your watchstones" is pre-3.20.

### Ascendancies (rolling waves of reworks)

- **3.20 (Forbidden Sanctum)**: Inquisitor consecrated-ground rework, Pathfinder flask system overhaul.
- **3.21 (Crucible)**: Trickster Polymath redesign, Champion Fortify changes.
- **3.22 (Trial of the Ancestors)**: Hierophant totem rework, Slayer overleech.
- **3.23 (Affliction)**: Necromancer minion-buff consolidation.
- **3.24 (Necropolis)**: Berserker Rage rework.
- **3.25 (Settlers)**: Pathfinder flask-permanent shift.
- **Modern**: assume any ascendancy guide >2 leagues old is partially stale.
- **Rot trigger**: ascendancy nodes cited that don't appear on the current wiki page = reworked.

### Defensive system tuning

- **3.16 (Scourge)**: Spell Suppression introduced as a Dexterity-side cap mechanic.
- **3.19 (Lake of Kalandra)**: Archnemesis-rare-mod overhaul affected on-death effects + rare drops.
- **3.20 (Forbidden Sanctum)**: Recoup mechanic introduced.
- **3.21 (Crucible)**: Armour rework — armour formula tuned, Armour-vs-large-hits effectiveness reduced.
- **3.22 (Trial of the Ancestors)**: max resistance softening.
- **Modern**: Spell Suppression + Armour + Block + Spell Block are the four primary mitigation layers.
- **Rot trigger**: any pre-3.16 guide that doesn't mention Spell Suppression for a dex-side build is now incomplete.

### Harvest, Beastcrafting, Crafting in general

- **3.11 (Harvest)**: Harvest crafting introduced. Reforge with X tag, Augment X tag, Remove X tag.
- **3.13 → 3.18**: progressive nerfs to Harvest "remove non-X / add X" recipes.
- **3.20+**: Harvest exists but reduced; Tier 17 maps + Beastcraft + essences fill the gap.
- **3.25 (Settlers)**: crafting-bench access tightened on certain mods.
- **Rot trigger**: "remove non-influence add influence" was Harvest's nuclear button pre-3.13; gone now.

### Map / atlas content lifecycle

- League content gets gradually folded into the core game over 1-3 leagues. Examples: Sanctum (3.20 league → core in 3.21), Heist (3.12 league → core in 3.13), Delve (3.4 league → core in 3.5), Ritual (3.13 league → core in 3.14).
- A guide writing about "the X league" usually means "the X mechanic now in core game".
- **Rot trigger**: a guide treating a league mechanic as temporary when it's been core for years.

---

## PoE2 — major conceptual shifts

### 0.1 (Early Access launch, December 2024)

- 6 ascendancies launch (3 base classes × 2 ascendancies each in EA).
- Spirit reservation system introduced as PoE2's primary aura mechanic.
- Trial of the Sekhemas + Trial of Chaos as ascendancy gates (replacing Lab).
- Atlas + Waystones + Towers + Tablets endgame.

### 0.2

- Spirit reservation rates tuned.
- Trial difficulty rebalanced.
- Combo skill (primer + executor) interactions clarified.

### 0.3

- Weapon Sets 1/2 + Book of Specialization introduced.
- Companion / pet system expanded.
- More ascendancies added.

### 0.4 (current as of 2026-05-08)

- Spirit reservation reductions tuned again.
- Companion AI improvements.
- Ascendancy balance pass.
- New uniques tied to combo skills.

### 0.5 (2026-05-29) — "Return of the Ancients"

- **Atlas tree rework** — guided fortress structure with sub-trees per league mechanic, ~400 nodes.
- **New league mechanic**: Runes of Aldur. Drops Verisium currency, Runic Ward, Alloys, 100+ runes.
- **New pinnacle bosses**.
- **Atlas expansion** to ocean tiles.
- Conceptual delta: PoE2 endgame moves from "infinite atlas with waystones" to "guided sub-tree progression". Specialisation choice becomes more load-bearing than per-map RNG.

### 1.0 (target December 2026, post-ExileCon)

- Full release. Expect another major rebalance pass.

### Rot trigger heuristics for PoE2

- **Pre-0.3** content predates Weapon Sets and Book of Specialization → set-swap advice missing.
- **Pre-0.4** content predates Companion AI improvements.
- **Pre-0.5** content (i.e., everything up to 2026-05-29) predates the Atlas rework.

PoE2 patch cadence is roughly **every 4 months** during early access. Anything older than that needs explicit version verification.

---

## Common temporal rot patterns

These are the patterns most likely to corrupt advice when consulting older sources:

1. **Aura system terminology drift** (PoE1) — "reduced reservation" vs "aura effect" framing.
2. **Flask piano references** (PoE1) — pre-3.25 utility-flask manual play.
3. **Cluster jewel notable names** (PoE1) — pre-3.16 notables that don't exist.
4. **Sextant references** (PoE1) — pre-3.25 atlas modifier system.
5. **Recoup non-existent before 3.20** (PoE1) — early defensive guides miss it.
6. **Spell Suppression non-existent before 3.16** (PoE1) — older defensive guides cap-stack only block + evasion.
7. **Harvest "remove non-X add X" recipes** (PoE1) — pre-3.13 nuclear option, gone.
8. **PoE2 weapon-set advice missing before 0.3**.
9. **PoE2 Atlas tree references** — entirely 0.5+, anything before is the old waystone-only system.

---

## How the agent should behave

When a user cites a guide / video / forum post / Maxroll article:

```
1. Note the publication date / patch tag if visible.
2. Compare to current patch (pull from session context or fetch live).
3. Map any mechanics it references against the rework table above.
4. If a mechanic is in the rework list and the source predates the rework — flag uncertainty.
5. Prefer current wiki / patch notes over the cited source for any rot-prone fact.
6. State the temporal context plainly: "This guide is from Crucible 3.21, before the X rework. The principle still holds, but Y has changed since."
```

The agent should never silently use stale information as ground truth. Either re-verify or surface the staleness.

## Cross-references

- `15_source_registry.md` — official patch notes URLs (tier-1 source for all temporal questions).
- `26_validation_and_self_correction.md` Rule 3 — patch-version-awareness self-check.
- `13_retrieval_playbooks.md` "Playbook: current patch / league" — search recipe for patch verification.
- `poe2/00_version_pinning.md` — current PoE2 version anchor (updated each patch).
