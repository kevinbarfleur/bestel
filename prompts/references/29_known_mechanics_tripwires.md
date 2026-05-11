# Known mechanics tripwires — frequently misremembered zones

This reference flags the mechanics where the 2026-05-12 audit caught LLM-class models hallucinating, recalling stale values, or conflating PoE1 with PoE2. **None of the entries below carry numeric values by design** — the audit found that storing values here was the failure mode: the model would recite the cached value (often wrong by then) instead of fetching the wiki.

The new contract for this file: **list the zones of confusion + force a fetch**. If a user's question touches one of these zones, the agent MUST call the cited tool (`wiki_parse`, `repoe_lookup`, or `pob_calc`) before giving a number.

> **Hard rule (mirrors SYSTEM_PROMPT Rule 2b)**: never cite a percentage, cooldown, charge count, socket count, magnitude, or implicit-tier value from memory while answering a question about any entity in this file. The wiki URL is provided next to each entry — call it.

## Spell Suppression — cap and prevented-damage value

**What gets misremembered**: the chance-to-suppress cap AND the percentage of spell damage prevented at that cap. Both are commonly stated wrong. The audit caught a "50% prevented" recall that had been stale since a wiki rebalance.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Spell_Suppression` — state both the current cap and the prevented-damage value as printed on the wiki.

**Cross-game watch**: PoE2's spell-defence layer is NOT the same system. Don't transfer suppression intuition.

## Cluster jewel hierarchy — socket / notable counts

**What gets misremembered**: how many Mediums fit in a Large, how many Smalls fit in a Medium, how many notables each size can roll. Cluster math is the single most-confused PoE1 mechanic.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Cluster_Jewel` — state the current ladder verbatim.

**Cross-game watch**: PoE2 does NOT have cluster jewels. The names `Voices`, `Megalomaniac`, `Luminous Trove`, `Heroic Tragedy`, `Undying Hate`, `Flesh Crucible`, `From Nothing` are **PoE1-only**. Citing them as PoE2 jewels is hallucination.

## Voices — socket count and notable count per socket

**What gets misremembered**: how many sockets Voices grants and how many notables fit in each. The audit found a stale claim about per-socket notable count.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Voices`.

## Eldritch implicits — eligible slots

**What gets misremembered**: which slots can roll Eldritch implicits. The set is small and stable, but stating it from memory is still risky.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Eldritch_implicit_modifier`. The Eater of Worlds and Searing Exarch altars target only specific armour slots — wiki lists them.

## Soul of Solaris — damage scope

**What gets misremembered**: whether Solaris reduces physical damage from hits, ailments, both. The audit found stale and wrong recalls of the percent value.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Soul_of_Solaris`. State whether the reduction applies to hits, DoT, or ailments — and the exact value — from the wiki, not memory.

## Trade pseudo stats — aggregation scope

**What gets misremembered**: whether `pseudo.*` filters aggregate within a single item or across the whole build. The mechanic is stable but easy to misstate.

**Always fetch**: `https://www.pathofexile.com/api/trade/data/stats` and the trade docs. Confirm per-item vs cross-build before answering.

## Resolute Technique — scope of "cannot crit"

**What gets misremembered**: whether RT only suppresses attack crits or also spell crits.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Resolute_Technique`. Read the keystone text verbatim before answering.

## Damage conversion — sequential vs parallel math

**What gets misremembered**: the order in which gear conversion vs skill conversion vs support conversion applies, and how the leftover physical pool is distributed. The audit found a 35/65 worked-example claim that the Deep Research agent flagged as not generally valid.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Damage_conversion`. Read the order-of-operations section before working through a conversion math problem. Better still: run the conversion through `pob_calc` on the actual build, which models the full sequence.

## PoE2 pinnacle boss life

**What gets misremembered**: a specific HP number for a current PoE2 pinnacle. Pinnacle HP is rebalanced regularly and PoE1 numbers (Sirus / Eater) do **not** transfer.

**Always**: run `pob_calc(category="offence", calcs={enemyIsBoss=true})` and read the engine output, OR `wiki_parse https://www.poe2wiki.net/wiki/<Boss>` for the current pinnacle taxonomy. The pinnacle roster itself shifts at major patch boundaries (notably 0.5).

## PoE2 charms vs flasks

**What gets misremembered**: that PoE2 "renamed flasks to charms". They are parallel systems, not a rename.

**Always fetch**: `wiki_parse https://www.poe2wiki.net/wiki/Charm` and `wiki_parse https://www.poe2wiki.net/wiki/Flask`. State the distinction explicitly. **Never invent specific charm names** like `Ngamahu's Chosen`, `For Utopia`, `Apex Mode` — they're fabrications.

## Marble Amulet — implicit range and explicit mod pool

**What gets misremembered**: the life-regen implicit range, the chaos-resistance tier weights, the ilvl gates on its mod pool. Multiple audit-flagged recalls.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Marble_Amulet` for the implicit, `repoe_lookup category=base_items name="Marble Amulet"` for the affix mod pool with tier weights and ilvl gates. **Do NOT cite tier ranges, ilvl numbers, or weights from memory**.

## Tabula Rasa — drop level

**What gets misremembered**: the drop level (it is an exception to "drop level == base level" for many uniques).

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Tabula_Rasa`. Don't quote a drop-level number from memory.

## Mob Mentality — cluster jewel notable scope

**What gets misremembered**: what stat the notable grants and what cluster-jewel enchantment family it rolls under. The 2026-05-12 audit caught a "minion damage" misclassification — the actual notable is in the warcry/exerted-attack family.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Mob_Mentality`. State its actual effect from the wiki page.

## Synthesised implicit — distinction from base implicit

**What gets misremembered**: whether a Synthesised implicit and a regular implicit are the same row on the item, or distinct.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Synthesis_(mechanic)`. They are distinct — but the agent should still confirm rather than recite.

## Watcher's Eye — aura roll selection

**What gets misremembered**: whether you can target which aura the rolls tie to during crafting. (You can't — rolls are random.)

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Watcher%27s_Eye` for the current roll mechanics + the per-aura mod pool.

## Crafting bench — cap on bench-crafted modifiers

**What gets misremembered**: how many bench-crafted mods can sit on one item (1 by default, more under multimod / Veiled / Beastcraft routes).

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Crafting_bench`. State the current cap + the exceptions (multimod, etc.) from the wiki.

## Fracturing Orb — required mod count and target selection

**What gets misremembered**: how many mods the item must have, and whether the user picks the fractured mod.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Fracturing_Orb`.

## PoE2 Pantheon — does not exist

**What gets hallucinated**: that PoE2 ships a Pantheon system. **It does not** (as of the file's last audit). Pantheon is PoE1-only.

**Always**: confirm via `wiki_parse https://www.poe2wiki.net/wiki/<topic>` before naming a "PoE2 pantheon power". Specifically, **never invent** fake pantheon upgrades like `Sebbert, Crescent's Point`.

## Heist — contract / blueprint / grand-heist tiers

**What gets hallucinated**: named "modes" like `Apex Mode` or `Nadir Mode`. These do NOT exist.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Heist`. Heist progresses through contracts → blueprints → grand heists; there are no named difficulty tiers.

## Maven's Forgotten invitation — reward shape

**What gets misremembered**: that Cortex drops from a Maven Forgotten invitation. It does NOT — Cortex is a Synthesis pinnacle map.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Maven` for invitation rewards; `wiki_parse https://www.poewiki.net/wiki/Cortex_Map` (or current name) for the Synthesis side.

## Ritual deferral — fee / hold-count / formula

**What gets misremembered**: the deferral fee percent, the maximum hold count, the increment formula. All version-pinned and easy to misremember.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Ritual_altar`. Do NOT assert a specific defer fee (10%, 5%, 15%) or hold count (50 items) without a wiki fetch confirming the current value.

## Mind Over Matter — damage diversion percent

**What gets misremembered**: the percent of damage taken from mana before life. The audit caught a stale "30%" recall (the wiki value was bumped in a patch since the original reference was written).

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Mind_Over_Matter`.

## Trinity Support — elemental penetration value

**What gets misremembered**: the penetration % per stack and the cap when all three resonances are at the activation threshold. The audit caught a "50%" recall that was off by a large factor vs the current wiki.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Trinity_Support`.

## Flame Dash — charge count and cooldown

**What gets misremembered**: how many charges Flame Dash stores and how long the per-charge cooldown is. The audit caught a "3 charges / 10s" recall that the wiki contradicts.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Flame_Dash`.

## Bone Helmet — minion-damage implicit magnitude

**What gets misremembered**: the implicit's percent value. The audit caught a "+30%" recall that had been stale since a patch nerf years prior.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Bone_Helmet` or `repoe_lookup category=base_items name="Bone Helmet"`.

## Spine Bow — base attack speed

**What gets misremembered**: the base APS value. The audit caught a "1.50" recall that had been stale since a patch change.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Spine_Bow` or `repoe_lookup`.

## PoE2 Talismans — item class

**What gets misremembered**: that PoE2 Talismans are amulet-slot charge items (like PoE1 Talisman league items). They are NOT — in PoE2, Talismans are a two-handed melee weapon class used by shapeshift archetypes.

**Always fetch**: `wiki_parse https://www.poe2wiki.net/wiki/Talisman`. State the actual class + slot + role from the wiki, never from memory.

## PoE2 Weapon Swap timing

**What gets misremembered**: that PoE2 weapon-set swap takes ~250 ms. It was animated in early PoE2, then made instant in 0.3. The state may shift again.

**Always fetch**: `wiki_parse https://www.poe2wiki.net/wiki/Weapon_set`. State whatever the current swap window is from the wiki.

## Onyx Amulet — implicit type

**What gets misremembered**: that Onyx Amulet grants "Strength + Intelligence". It actually grants all attributes. The audit flagged this as a textbook recall error.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Onyx_Amulet`. Confirm the implicit before describing what the base does.

## Imbued Wand — implicit type

**What gets misremembered**: that Imbued Wand carries a "spell critical strike chance" implicit. The actual implicit is spell damage.

**Always fetch**: `wiki_parse https://www.poewiki.net/wiki/Imbued_Wand`.

## Stellar Amulet — game attribution

**What gets misremembered**: that Stellar Amulet is a PoE1 base. It is PoE2-only. PoE1's all-attribute amulet is Onyx (see above).

**Always fetch**: `wiki_parse https://www.poe2wiki.net/wiki/Stellar_Amulet` for the PoE2 base. Do not introduce it into a PoE1 context.

---

## How the agent should treat this file

1. **If the user's question touches an entity listed above**, the very next tool call must be the cited `wiki_parse` / `repoe_lookup` / `pob_calc`. Quote no number, percent, charge count, or magnitude until that fetch completes.
2. **If the user names an entity not in this list** but the answer needs a magnitude, fetch anyway. The file is illustrative, not exhaustive — every numeric claim on a PoE mechanic deserves a tool call.
3. **If the wiki page contradicts a long-held assumption** in your prose, trust the wiki and update the answer. The whole point of this sprint is that the in-context values are by design unreliable.

## Cross-references

- `01_source_policy.md` — tiered source list and re-search algorithm.
- `26_validation_and_self_correction.md` — extended validation toolkit (engine-trust, staleness, patch-version-awareness, disambiguation, self-consistency).
- `14_validation_and_failure_modes.md` — failure-mode taxonomy with recovery procedures.
- `13_retrieval_playbooks.md` — retrieval recipes per question type.
