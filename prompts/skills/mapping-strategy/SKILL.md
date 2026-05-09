+++
name = "mapping-strategy"
description = "Use when the exile asks about atlas progression, what mechanic to focus on, what to run for currency vs experience vs exotic drops, or how to optimise their atlas tree for a target reward. Covers PoE1 atlas (voidstones, sextants, scarabs, expedition logbooks, blight, delirium, breach, harvest, ritual, legion, betrayal Aisling/Catarina, Maven witnesses) and PoE2 atlas towers/citadels. Skip when the user is just asking 'what is X mechanic' — that's Brief mechanics."
when_to_use = [
  "User asks 'what should I run' / 'what's profitable' / 'what's the atlas meta'",
  "User wants atlas tree advice for a specific mechanic (juiced ritual, breach domains, expedition Tujen, scarab farming)",
  "User compares two mechanics for currency vs xp vs exotic drops",
  "User asks about voidstones, atlas progression, or unlocking pinnacle bosses",
  "User asks about PoE2 atlas — towers, citadels, breach domains in 0.5",
]
trigger_examples = [
  "what should I farm in this league",
  "best atlas tree for breach",
  "is expedition or ritual better for divines",
  "how do I unlock the searing exarch",
  "what towers should I build in poe2",
]
+++

# Mapping strategy — workflow

Length target: 6–12 sentences. Recommend ONE primary path; flag alternates only if the user explicitly asks for comparison. Cite atlas mechanic references and at least one community guide.

## Step 1 — Confirm the goal

Three goals dominate; the right strategy diverges sharply.

| Goal | Optimised metric | Typical answer pattern |
|---|---|---|
| **Currency** | divines/hour | scarab-stacked single-mechanic farming (expedition Tujen, ritual omen, breach splinters) |
| **Exotic drops** | drop chance / hour for one item | targeted scarab + sextant + atlas keystone for that drop |
| **Experience** | level/hour at red maps | juicer alch-and-go with delirium or beyond |

If the user didn't state the goal, ask once OR pick currency (the most common ask).

## Step 2 — Atlas tree layout

Sketch the tree shape, not the exact node order. Examples:

- **Breach domains farm** — keystone "Expanding the Breach", clusters: breach (3-4 nodes), splinters, scarabs, map sustain.
- **Ritual omen farm** — keystone "Vivid Memories" (boosts ritual rewards), clusters: ritual deferred chance, ritual rerolls, scarabs.
- **Expedition Tujen** — keystone "Plundering Expedition", clusters: Tujen-specific, expedition logbook drop chance, NO Rog/Gwennen unless user wants the trade-off.
- **Scarab farming (Sirus map)** — atlas keystone "Wandering Path" + scarabs + Eldritch Altar bonuses.
- **Map sustain** — keystone "Singular Focus" (only your favored mechanic), high pack size, beyond, map-drop nodes.

Reference the full atlas mechanic taxonomy via `read_internal_reference("18_atlas_and_endgame_mechanics.md")` when the user wants the deep mechanic background.

## Step 3 — Sextants + scarabs + altar choice

Name the actual scarabs the build runs. Examples:

- **Breach** → 4× breach scarab (rumour, trees, splinters, monstrous) + sextant "X breach in your maps".
- **Expedition** → 2× expedition scarab + 1× Logbook scarab; sextant "expedition explosives" if running Tujen.
- **Sirus farming** → 4× Sirus scarab (titanic + influencing + multiplying + completion) + sextant "Conqueror appears".

Eldritch altar choice: "+life from monsters / +rares / +pack size" for experience; "+specific currency / +scarab / +map" for divine farming.

## Step 4 — Map base + roll quality

Recommend a map base type (T16 Crimson Temple, Beach, Cemetery, etc.) when relevant. Roll quality: alch-and-go for currency farms; chisel-alch-vaal-corrupt for juicer + delirium runs; quant scarabs + corrupt for high-stake exotic farms.

## Step 5 — PoE2 specifics (when applicable)

PoE2 atlas (0.5 onwards) is structurally different — towers, citadels, breach domains as map-modifiers, no scarabs. If the user asked about PoE2:

1. Load `read_internal_reference("poe2/05_atlas_mechanics_05.md")` for the current patch's atlas state.
2. Recommend tower placements based on goal: corruption towers for breach domains, expedition towers for logbooks, ritual towers for ritual currency.
3. Citadels = pinnacle access. Don't conflate with PoE1 voidstones.

## Cite

Wiki for each mechanic, Maxroll mapping/atlas guide if one exists, and one community video/article when the user wants validation. The internal references (`18_atlas_*`, `poe2/05_atlas_*`) are scaffolding — don't cite them as primary sources.
