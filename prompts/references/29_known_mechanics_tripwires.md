# Known mechanics tripwires — frequently misremembered facts

This reference holds the mechanics that the audit 2026-05-08 caught models hallucinating or contradicting themselves on. Each entry is a single canonical fact plus the citation. **None of these entries replace a wiki fetch** — they exist so that when a model recalls a half-correct fragment from training data, the correct fragment is right here for cross-check. Always confirm the live value via `wiki_parse` before quoting in the answer; this reference is a tripwire, not a citation.

## Spell suppression — cap

**Cap is 100%.** Reaching 100% chance to suppress means every spell hit triggers the suppression check; the damage reduction itself is fixed at 50% (raisable via specific keystones / ascendancy nodes, e.g. the Slayer's "Unstoppable Hero" or `Wind Dancer` keystone interactions).

Common wrong answer: 75% (confusion with hard resistance cap). Common wrong answer: 50% (confusion between the suppression chance and the damage-reduction value).

Source: `https://www.poewiki.net/wiki/Spell_suppression`.

## Medium cluster jewel — Small cluster sockets

**A Medium cluster jewel hosts exactly one Small cluster jewel socket.** Not two, not three. The hierarchy is: Large hosts up to 2 Mediums, each Medium hosts 1 Small.

Common wrong answer: 2 small sockets per Medium. This is the single most common cluster-math mistake — it implies a tree-wide cluster network larger than what's actually possible.

Source: `https://www.poewiki.net/wiki/Cluster_Jewel`.

## Voices — total clusters in one tree

**Voices replaces a single jewel socket with three Small cluster jewel sockets.** Each Small holds 2 notable passives. The interaction with regular cluster sockets in the rest of the tree is independent — Voices does NOT change the count of Large or Medium sockets allocatable elsewhere.

Common wrong answer: "you can fit 9 clusters with Voices". The correct framing: Voices gives 3 *additional* Small sockets beyond the regular tree's cluster slots. Pair with `Luminous Trove` to access the slots quickly.

Source: `https://www.poewiki.net/wiki/Voices`.

## Eldritch implicits — eligible slots

**Eldritch implicits roll only on body armour, helmet, gloves, and boots.** They do NOT roll on weapons, amulets, rings, or belts.

Common wrong answer: "Eldritch implicits are on every slot". The Eater of Worlds and Searing Exarch altars target only the four armour slots above; the other slots use influence mods or other crafting routes.

Source: `https://www.poewiki.net/wiki/Eldritch_modifier`.

## Pantheon — Soul of Solaris damage scope

**Soul of Solaris reduces physical damage from hits, not from ailments.** The flat 6% additional physical damage reduction applies to incoming hits only. Bleed and physical damage over time bypass this layer entirely.

Common wrong answer: "Solaris reduces all damage from ailments". Mixing up the two Solaris bonuses (the elemental ailment chance reduction is a separate clause and only applies to ignite/shock/freeze caused by enemies near you).

Source: `https://www.poewiki.net/wiki/Soul_of_Solaris`.

## Trade pseudo stats — aggregation scope

**Pseudo stats on the trade site aggregate within a single listed item, not across the build.** A `pseudo.pseudo_total_life` filter on a chest searches chests where the implicit + explicit + crafted life rolls on **that one chest** sum to your minimum.

Common wrong answer: "pseudo stats search across my whole gear". The trade site never aggregates across slots; each search applies to one item at a time.

Source: `https://www.pathofexile.com/api/trade/data/stats`.

## Resolute Technique — scope of "cannot crit"

**Resolute Technique disables crit on every attack and spell from your character.** It is total — there is no partial Resolute Technique.

Common wrong answer: "spells can still crit with Resolute Technique". This was never the case in PoE1; the keystone explicitly says "Your hits cannot be evaded. Your hits cannot be Critical Strikes." Spells are hits.

Source: `https://www.poewiki.net/wiki/Resolute_Technique`.

## Conversion — double chain math

**Conversion is sequential, not parallel.** If gear converts 35% physical → lightning, then a support adds 100% physical → fire, the lightning conversion fires first on its 35% slice, leaving 65% physical to be reconverted by the support. Final: 35% lightning, 65% fire.

Common wrong answer: "100% lightning" or "100% fire" — assumes the second conversion overrides the first. Common wrong answer: split 50/50. Conversion never exceeds 100% total of the original element.

Source: `https://www.poewiki.net/wiki/Damage_conversion`.

## PoE2 — pinnacle boss life

**Pinnacle boss life in PoE2 0.5 is engine-derived; do not memorize.** The bundled `pob_calc` engine surfaces the live target HP via the `enemyIsBoss=true` setting. Numbers from PoE1 (Eater of Worlds, Sirus) do **not** transfer; PoE2 has its own pinnacle taxonomy (Arbiter of Ash, currently).

Common wrong answer: cite a specific HP number from memory. Always run `pob_calc(category="offence", calcs={enemyIsBoss=true})` and read the engine output.

Source: bundled engine; cross-check with `https://www.poe2wiki.net/wiki/Boss` for the canonical list.

## PoE2 — charms vs flasks

**Charms in PoE2 are NOT renamed flasks.** Flasks in PoE2 are exclusively for instant life / mana recovery (no utility flasks). Charms are slotted on the belt and provide passive effects with on-demand triggers (different cadence, different scaling). Treating them as drop-in flask replacements gives wrong answers about the build's utility uptime.

Common wrong answer: "charms are PoE2 flasks". They are a parallel system, not a rename.

Source: `https://www.poe2wiki.net/wiki/Charm`.

## Marble Amulet — mod pool

**Marble Amulet (PoE1) is the canonical chaos-res + life-regen base.** The amulet rolls life regeneration as an implicit (`+(1.0–1.4)% to maximum Life per second`) and has full access to the standard amulet explicit pool: chaos resistance, elemental resistances, life, attributes, crit multi, etc.

Common wrong answer: invent a specific T1 ilvl-86 spawn weight from memory. Use `repoe_lookup` to get the actual mod tier table; never quote a tier weight without the engine.

Source: `https://www.poewiki.net/wiki/Marble_Amulet`.

## Tabula Rasa — drop level

**Tabula Rasa drop level is 1.** It is the single notable exception to "drop level == base level" for many uniques and is intentional so brand-new characters can chase one.

Common wrong answer: drop level 3, drop level 8, "the wiki says one and the trade site says another" (the trade site's level filter shows level requirement, not drop level). Never quote a drop level from memory without `wiki_parse`.

Source: `https://www.poewiki.net/wiki/Tabula_Rasa`.

## Mob Mentality — cluster jewel notable

**Mob Mentality is a Cluster Jewel notable that grants Minion Damage scaling.** It rolls on cluster jewels with the `Minion Damage` enchantment, not on amulets or rings.

Common wrong answer: place it on the wrong slot (amulet / ring). The notable can only be reached via cluster jewel allocation.

Source: `https://www.poewiki.net/wiki/Mob_Mentality`.

## Synthesised implicit — distinction

**A synthesised implicit is a special implicit added by the Synthesis crafting process.** It is mechanically distinct from the base implicit of the item and shows in a separate row on the in-game item display. Some Synthesis implicits are unique to that crafting flow (e.g. "Trigger a Socketed Spell when you Skill a Rare or Unique Enemy" only spawns on Synthesised items).

Common confusion: treating a Synthesised item's implicit as identical to a regular implicit. Cortex (the Synthesis pinnacle map) drops items with both regular implicits and Synthesised implicit slots.

Source: `https://www.poewiki.net/wiki/Synthesis_(mechanic)`.

## Watcher's Eye — aura roll selection

**Watcher's Eye rolls are entirely random.** You cannot pre-pick which auras roll. The jewel rolls 1–3 mods, each tied to a specific aura that must be active for the mod to apply. You **cannot** target which aura the mod is tied to during crafting.

Common wrong answer: "I can pick Determination + Discipline rolls". The aura tie is part of the random roll; if your two-aura roll doesn't include your active auras, the jewel is dead weight in your build.

Source: `https://www.poewiki.net/wiki/Watcher%27s_Eye`.

## Bench-crafted suffixes — limit

**Maximum one bench-crafted suffix on a single item.** The crafting bench enforces this hard cap regardless of the prefix slot count.

Common wrong answer: "I can stack two bench suffixes". The bench refuses the second craft on the same affix axis; the only way to get two crafted suffixes is to abandon the limit via Veiled / Beastcraft / Aisling-imprint flows, none of which are simple bench crafts.

Source: `https://www.poewiki.net/wiki/Crafting_bench`.

## Fracturing Orb — outcome

**A Fracturing Orb fractures one random mod on a rare item with at least four mods.** You do not choose which mod gets fractured; the result is uniform random across the item's existing mods.

Common wrong answer: "I can pick which mod fractures". You cannot. The orb is high-variance; pre-removing unwanted mods (annul, Eldritch annul) before fracturing is the standard mitigation.

Source: `https://www.poewiki.net/wiki/Fracturing_Orb`.

## PoE2 cluster jewels — they don't exist

**Cluster jewels (Large / Medium / Small) are PoE1-only.** PoE2 has its own jewel system (Time-Lost Jewels and others) but **no cluster jewel hierarchy** and no cluster-jewel-style notables.

Common wrong answer: attributing PoE1 unique cluster jewels to PoE2. The names `Voices`, `Megalomaniac`, `Luminous Trove`, `Heroic Tragedy`, `Undying Hate`, `Flesh Crucible`, `From Nothing` are **PoE1 cluster jewel uniques**. Citing them as PoE2 jewels is hallucination. Do NOT.

Sources: `https://www.poewiki.net/wiki/Cluster_Jewel` (PoE1) — confirm PoE2 jewel system via `https://www.poe2wiki.net/wiki/Jewel`.

## PoE2 pantheon — does not exist

**The Pantheon system is PoE1-only.** PoE2 does not currently ship Soul-of-Solaris-style passives. Naming any pantheon (`Solaris`, `Lunaris`, `Brine King`, `Arakaali`, `Tukohama`, `Yugul`, `Abberath`, `Shakari`, `Ralakesh`, `Garukhan`, `Ryslatha`, `Gruthkul`) as a PoE2 mechanic is hallucination.

Common wrong answer: inventing a fake pantheon upgrade like `Sebbert, Crescent's Point` for PoE2. Pantheon upgrades only exist in PoE1, only have official GGG-named NPC sources, and are documented on the wiki — never invent.

Sources: `https://www.poewiki.net/wiki/Pantheon` (PoE1 system).

## PoE2 charms — separate system from PoE1 utility flasks

**Charms in PoE2 are NOT renamed flasks.** They are a distinct slot on the belt, recharge differently, and have their own pool of effects. Do not call them "renamed flasks" or treat the mechanics as identical.

Common wrong answer: inventing PoE2 charm names like `Ngamahu's Chosen`, `For Utopia`, `Apex Mode`. Specific charm names should always be confirmed via `https://www.poe2wiki.net/wiki/Charm` or the relevant patch notes — never recalled from training data.

Source: `https://www.poe2wiki.net/wiki/Charm`.

## Heist — contract grades exist, "modes" do not

**Heist contracts have job tags (Lockpicking, Perception, etc.) and area levels, but there are no named "modes" like `Apex Mode` or `Nadir Mode`.** Those names are fabrications.

Common wrong answer: "the higher tier is Apex Mode and the lower is Nadir Mode". Wrong. Heist progression is: contracts (single rooms) → blueprints (full multi-room mansions) → grand heists. The wiki documents the actual mechanics; never invent named modes.

Source: `https://www.poewiki.net/wiki/Heist`.

## Maven's Forgotten — invitation reward shape

**The Forgotten Maven invitation drops uber pinnacle invitation set fragments and limited unique items, NOT Cortex maps.** Cortex is a unique map associated with the Synthesis boss (Cortex itself, in the Synthesis areas), not a Maven Forgotten reward.

Common wrong answer: "Cortex maps drop from Maven Forgotten". The Forgotten Maven invitation rewards the next tier of the pinnacle progression and league-specific uniques; it does not drop Cortex. Confirm via `https://www.poewiki.net/wiki/Maven`.

Source: `https://www.poewiki.net/wiki/Maven`.

## Ritual deferral — exact mechanics need a wiki fetch

**The deferral cost formula and limits change with patches and are easy to misremember.** Specifically: do NOT assert a specific defer fee (10%, 5%, 15%) or hold count (50 items) without a wiki fetch confirming the current value. The mechanic exists, but the precise numbers are version-pinned.

Common wrong answer: "deferring costs 10% + 5% fee" with no source. Even if you recall a number, fetch `https://www.poewiki.net/wiki/Ritual_altar` first and quote it.

Source: `https://www.poewiki.net/wiki/Ritual_altar` (current PoE1) and `https://www.poewiki.net/wiki/Ritual_league` for historical mechanics.

## Marble Amulet — implicit + mod pool

**Marble Amulet has a flat life-regen implicit (PoE1).** The exact value (e.g. `(1.2-1.6)% of Life per second`) is patch-pinned and must be confirmed via `wiki_parse`. Do NOT cite tier ranges (`(64.1-96) Life per second`, `+(31-35)% to Chaos Resistance`, ilvl `74/68/81/65/56`) from memory — these are precisely the numbers Haiku-class models hallucinate. Always go to the wiki.

Common wrong answer: stating tier ranges with confidence and no source. Even if the numbers feel right, they're not yours to invent — fetch them.

Source: `https://www.poewiki.net/wiki/Marble_Amulet` and the related affix wiki pages.
