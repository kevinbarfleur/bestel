+++
name = "craft-audit"
description = "Use when the exile asks about crafting an item, weighing a deterministic vs gambling craft path, decoding a base or its mod pool, or budgeting currency for a craft. Includes essence/fossil/harvest/embers/recombinator/eldritch annul flows, and explains weighted vs unweighted mod pools when the user's question hinges on tier rolls. Skip when the question is just 'what is this orb' (Brief mechanics handles that)."
when_to_use = [
  "User asks 'how do I craft X' on a specific base",
  "User compares two craft methods (essence vs harvest vs fossil vs embers)",
  "User asks about mod pools, mod weights, or 'can this base roll Y'",
  "User wants a currency budget for a craft path",
  "User asks about eldritch implicits, harvest reforges, recombinators, or specific essence outcomes",
]
trigger_examples = [
  "how do I craft a hubris circlet with -mana cost",
  "is essence or harvest cheaper for a phys two-hander",
  "what mods can roll on a vaal regalia",
  "best way to get +1 frenzy charges on amulet",
  "can I recombinate a flesh and stone setup",
]
+++

# Craft audit — workflow

Length target: 4–8 sentences. Optional panel marker (`item-card` type) when the answer hinges on one base. Cite wiki + PoEDB; cite the engine via `repoe_lookup` when tier weights matter.

## Step 1 — Confirm the base + ilvl

Identify the exact base type and minimum item level required for the desired mods. If user didn't specify, ask once OR pick the most common base for the desired build. Use `repoe_lookup(category="base_items", name="<base>")` to confirm exact ilvl thresholds for the relevant mod tier.

## Step 2 — Lay out the deterministic path first

Always start with the **deterministic option** before the gambling option. Examples:

- **Veiled crafts** → Aisling slam if available, else specific veil unlock from Jun.
- **Harvest reforges** → list the specific reforge keep recipe by tag (lightning, attack, mana, life, etc.).
- **Essences** → name the specific essence tier and the mod it forces.
- **Eldritch implicits** → embers + ichors + the slot list (helmet/body/gloves/boots).
- **Bench crafts** → Hillock at hideout, list the metacraft slot if relevant.
- **Recombinators (Settlers/PoE2)** → only when 2 input items both carry desired prefixes/suffixes.

## Step 3 — If pure deterministic isn't possible, name the hybrid path

The 80% case: combine deterministic suffixes (e.g. "-mana cost while focused") with a chaos-spam or fossil-spam to roll the prefixes. State explicitly:

1. The **fixed step** (what the bench / harvest / essence locks in)
2. The **rolling step** (what's spammed: alt-aug-regal-multimod, fossil, essence, harvest reforge-keep)
3. The **finishing step** (lock prefixes / suffixes, multimod, exalt slam)

## Step 4 — Currency budget (when asked or hinted)

If the user mentions budget, give a rough divine-count estimate per stage. Typical ranges (PoE1 LE league):

- Alt-aug-regal: 50–150 alts, 5–10 regals → 1–3 div equivalent.
- Essence spam: 100–300 essences of relevant tier → 1–5 div for shrieking, 5–25 div for deafening.
- Fossil socket-of-3 spam: 50–200 fossils → 2–10 div.
- Harvest reforge-keep: 20–80 reforges of relevant tag → 2–15 div.
- Aisling T4 slam: 1–3 slams typical.
- Multimod: 2 div bench cost + 1 div exalt for the 6th mod.

If the user is on PoE2, recombinators are the dominant deterministic path — don't quote PoE1 fossil prices.

## Step 5 — Cite

Wiki page for the base, PoEDB page for the mod pool, and Maxroll crafting catalogue (`maxroll/poe1_crafting.md`) for the workflow if a guide exists. Include `repoe_lookup` in the chain when the mod's *weight* matters (rare mods on common bases).

## Common pitfalls — flag these explicitly

- **"Just multimod it"** is wrong when prefixes aren't already locked — multimod fills suffix slots.
- **Chaos spam** unblocks below ~60 chaos / div; above that, alt-aug-regal+craft is cheaper.
- **Harvest reforge keep <tag>** can wipe non-tagged mods. State which tag and warn.
- **Eldritch annul** removes a random eldritch mod, not a base mod — clarify.
- **Veiled chaos orb** (Aisling) consumes the slot; the other affixes get rerolled. Don't recommend it on an item with already-good rolls elsewhere unless replacement is genuinely cheap.
