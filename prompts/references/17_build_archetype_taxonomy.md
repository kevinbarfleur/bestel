---
description: Formal taxonomy of build archetypes (PoE1 + PoE2) with identifying tags, scaling levers, defensive defaults, common pitfalls, and per-archetype diagnostic checklists.
fetch_when: User asks "what kind of build is this?", "is X archetype viable?", or you need to identify the scaling pattern of a damage delivery before commenting on it; whenever you start evaluating a PoB and need to anchor on the archetype before diving into stats.
---

# 17 — Build archetype taxonomy

This is the spine of every build evaluation. **Identify the archetype first, then reason about its scaling levers and failure modes — never the other way round.**

A wrong archetype tag corrupts every downstream recommendation: telling a DoT player to stack crit, telling a trigger player to stack cast speed, telling a totem player to stack movement-speed-as-attack-speed. The archetype is the model the agent uses to interpret every other stat.

## How to use this doc

For each archetype, the entry has 5 fields:

1. **Identifying tags** — what to look for in `get_active_build` output to recognise this archetype. Cross-reference gem tags from `repoe_lookup category=gems` (Sprint 1+) and tree keystones.
2. **Scaling levers** — the multipliers that actually move the DPS / EHP needle.
3. **Defensive defaults** — the typical layer ordering for this archetype.
4. **Common pitfalls** — the top 3 breaking points that derail this archetype.
5. **Diagnostic checklist (5 steps)** — when DPS or EHP feels wrong.

Once `pob_calc` ships (Sprint 2), use it to verify scaling-lever stacking. Until then, work from `<PlayerStat>` cache values + structural extraction and flag uncertainty.

---

## PoE1 archetypes

### Hit / crit

- **Identifying tags**: gem tag `attack` or `spell`, weapon with high base crit, tree allocates "Diamond Skin" / "Doom Cast" clusters, items roll `+% Critical Strike Chance` / `+% Critical Strike Multiplier`. Common ascendancies: Assassin, Inquisitor (PoE1), Deadeye (with crit nodes).
- **Scaling levers**: crit chance × crit multi × added flat × more multipliers. Penetration / exposure for capped-resist bosses. Power Charges if using Assassin / Inquisitor.
- **Defensive defaults**: max resists 75-80%, Spell Suppression cap (Acrobatics or evasion-based), Phasing or Pathfinder flask uptime, hybrid Life+ES is common. Recovery via leech.
- **Common pitfalls**:
  - **Crit cap not reached** — effective crit chance (base weapon crit × Diamond Flask × tree-increased) must approach the per-hit cap for "more damage on crit" supports to fire consistently. Cap value: `wiki_parse https://www.poewiki.net/wiki/Critical_strike_chance`.
  - **Accuracy floor missed** — hit chance against bosses requires accuracy-stacking; missed hit = no crit roll. The cap and the accuracy formula are wiki-current — fetch before tuning.
  - **Multimods drowning** — too many "increased X" sources hit diminishing returns; one well-chosen "more" support beats five increased.
- **Diagnostic checklist**: ① effective crit chance against boss-tier accuracy at the per-hit cap (fetch the cap)? ② crit multiplier appropriate for the build's "more" stack (no static target — fetch per-archetype creator guide)? ③ stacked "more" multipliers counted (`pob_calc` will list them)? ④ enemy resists addressed via penetration or curse on hit? ⑤ accuracy at the cap against boss level via Precision / tree / Mark of the Shaper?

### Hit / non-crit (Elemental Overload, more-mult chassis)

- **Identifying tags**: Elemental Overload keystone allocated, low or zero base crit, gem tag `spell` or `attack`. Often `attack` builds running Resolute Technique. Common ascendancies: Trickster (Polymath), Champion, Slayer (Vaal Pact + leech).
- **Scaling levers**: more multipliers compound (Trickster Polymath, Awakened gems, ailment-of supports). Trinity Support provides elemental penetration when all three resonances are stacked — current penetration per resonance and cap: `wiki_parse https://www.poewiki.net/wiki/Trinity_Support` (the 2026-05-12 audit corrected a stale "50% pen" claim — fetch live). Wise Oak or curses are alternatives. Conversion chains (e.g., Phys → Cold → Fire via Hatred + Cold-to-Fire support).
- **Defensive defaults**: max resists 78%+ (overcap chaos), Aegis Aurora or Vaal Pact leech for sustain, often life-stack with Replica Soul Tether + ES recharge as backup.
- **Common pitfalls**:
  - **EO uptime gap** — Elemental Overload requires a recent crit (window duration is version-pinned — `wiki_parse https://www.poewiki.net/wiki/Elemental_Overload`). Without a low-crit "trigger" source, builds fall off mid-fight.
  - **Trinity uptime gap** — needs 3 elements dealing damage simultaneously. Dropping one breaks the penetration bonus (current magnitude is wiki-fetched, see above).
  - **More-mult double-counting** — players assume two "more X damage" supports stack additively; they don't.
- **Diagnostic checklist**: ① EO active at the moment of measurement? ② How many distinct "more" multipliers in `pob_calc`? ③ Conversion chain modeled correctly in PoB Calcs? ④ Resist mitigation via Trinity / Wise Oak / curse — which? ⑤ Vaal Pact + leech rate sustaining hits?

### Ailment-stack (ignite-prolif, poison-stack, bleed-explode)

- **Identifying tags**: gem tag `poison` / `bleed` / `ignite`, supports like Deadly Ailments, Empower, Awakened Vicious Projectiles, Rapid Decay (Swift Affliction). Pathfinder, Assassin (Noxious Strike), Gladiator (Gratuitous Violence) common.
- **Scaling levers**: ailment magnitude (= base hit damage × ailment-of-multiplier), ailment duration, faster ailments, "more damage with ailments". Ailments **scale on the hit** that applied them — front-loading hit damage matters even if you barely care about the hit itself.
- **Defensive defaults**: Pathfinder flask uptime, Master Surgeon, Headhunter-flavour utility. Often hybrid life + suppression. Bleed-explode (Lacerate Glad) leans on block + life regen.
- **Common pitfalls**:
  - **Hit damage neglected** — players cap "more ailment damage" stacking but ignore the hit damage that ailments scale from.
  - **Ailment magnitude vs duration confusion** — ignite scales with magnitude (1 ignite cap); poison scales with stack count (cap from skill API). Different scaling.
  - **Application chance below the cap** — chance to ignite/poison/bleed must reach the per-hit cap from gear + tree + supports; below that, RNG ailment uptime kills DPS. Cap value: per-ailment, fetch via `wiki_parse` on the ailment page.
- **Diagnostic checklist**: ① application chance at the per-hit cap on every hit? ② ailment magnitude scaling via "more ailment damage" *and* hit damage scaling? ③ proliferation source (Cospri's Will, Wildfire jewel, Voltaxic Burst, Master of Pain mastery)? ④ `<Calcs>` ailment uptime modelling realistic monster density? ⑤ for poison-stack: stack-cap reached within phase? The cap is per-build (scales with skills / supports / ascendancy) — confirm via `pob_calc` rather than assuming a static number.

### Damage-over-time (RF, Bane, ED-Contagion, Soulrend Totem)

- **Identifying tags**: gem tag `dot` / `degeneration`, Anomalous Bane (PoE1), Awakened Cast While Channelling, Burning Damage / Withering Touch. Chieftain (RF), Occultist (ED-Contagion), Hierophant (Soulrend Totem).
- **Scaling levers**: more DoT multiplier (key — increased % hits diminishing returns fast), area for proliferation, faster ailments / hexes for tick rate, ailment magnitude for ignite-DoT.
- **Defensive defaults**: high regen for RF (the exact % depends on the Chieftain ascendancy + Beacon-tier gear + tree — fetch `wiki_parse https://www.poewiki.net/wiki/Righteous_Fire` for the current self-burn rate to sustain against), Cloak of Flame for phys mitigation, MoM stack for spell-heavy maps. ED-Contagion leans on suppression + ES recharge.
- **Common pitfalls**:
  - **DoT cap unawareness** — RF, Caustic Arrow, Corrupting Fever all hit per-skill DoT caps. Once at cap, increased % does nothing — only more multipliers and ailment magnitude help.
  - **Single-target hole** — Maven, Sirus, pinnacle bosses degen DoT slowly compared to phase windows. Need a separate single-target burst (Cremation totem, Soul Mantle DD trigger, etc.) or accept slow kills.
  - **Proliferation gap** — without prolif (Cospri Will, Inpulsa-equivalent, Wildfire), DoT clear is slow.
- **Diagnostic checklist**: ① at DoT cap? (PoB Calcs shows the cap). ② more multipliers vs increased — count the more. ③ proliferation source on board? ④ single-target plan against pinnacle exists? ⑤ regen / recovery sustaining the self-DoT (RF) or hexed-recoup chassis?

### Totem / mine / trap

- **Identifying tags**: gem `totem` / `mine` / `trap`, Hierophant (totems + MoM), Saboteur (mines + traps), Trickster occasional. Items often roll `+% Cast Speed for Totems` / `+% Totem Damage`.
- **Scaling levers**: totem placement speed (separate stat from cast speed), more totems active, totem damage multipliers, mine throwing speed + detonation rate.
- **Defensive defaults**: stationary playstyle = high EHP needed. MoM stack typical for Hierophant. Saboteur leans on Born in the Shadows blind chance.
- **Common pitfalls**:
  - **Placement speed bottleneck** — slow placement = totems don't stay up between mob packs.
  - **Totem cap not reached** — base totem limit is low (fetch `wiki_parse https://www.poewiki.net/wiki/Totem` for the current base + Hierophant + cluster jewel scaling); endgame totem builds usually need ascendancy + tree investment to expand the cap.
  - **Totem AI / range gap** — totems can't path; melee-totem builds (Holy Flame Totem) need positioning.
- **Diagnostic checklist**: ① max totems / mines / traps active per skill? ② placement speed adequate for current map tier? ③ MoM stack health sustained under spell pressure? ④ totem damage via `Multiple Totems` / `Spell Totem` support comparisons? ⑤ enemy resist mitigation via totems' own curse-on-hit / penetration / Voltaic Mark?

### Minion (army, spectres, golems, SRS, AG)

- **Identifying tags**: gem tag `minion`, Necromancer ascendancy, items like The Baron, Bones of Ullr, +1 minion gear. Cluster jewels with `Renewal`, `Quickening Covenant`, `Vicious Bite`.
- **Scaling levers**: minion-specific damage modifiers (% increased Minion Damage on tree + items), minion supports (Awakened Minion Damage, Empower for +levels), aura support to feed the minions (Animate Guardian wearing Kingmaker etc.).
- **Defensive defaults**: minion EHP is *separate* from player EHP and matters as much. Necromancer's Mistress of Sacrifice. Player tankiness usually low-investment hybrid life.
- **Common pitfalls**:
  - **AG dies** — Animate Guardian is usually wearing 50ex+ of unique gear. Death = currency loss. Sustainability via Garukhan's Flight + To Dust + +life on minion gear is mandatory.
  - **Spectre gem-level cap** — spectres' damage scales with gem level. +levels from items (Mon'tregul's Grasp, Vis Mortis, +2 minion gem helmets) compound.
  - **Minion AI / aggro** — minions don't always engage the same target. Convocation + Withering Step pull them, but rotation matters.
- **Diagnostic checklist**: ① minion damage scaling via tree + supports + +gem-level? ② AG sustainability (life + ES + flask charge regen + minion regen aura)? ③ spectres being raised — which species (cf. Maxroll spectre tier)? ④ aura support feeding minions via Necromantic Aegis or Necromancer ascendancy? ⑤ player defensive layer at 6k+ EHP with leech / regen?

### Trigger (CWC, CoC, CWDT, Asenath, Cospri, Mjolner)

- **Identifying tags**: gem tag `trigger`, Cast While Channelling, Cast on Critical Strike, Cast When Damage Taken, Asenath's Gentle Touch unique gloves (Bone Chill on hit triggers), Cospri's Malice (CoC dagger), Mjolner (CoMK mace), Manaforged Arrows.
- **Scaling levers**: trigger source uptime (the channelling skill / hit / damage-taken event), cooldown gating (CoC 0.15s), attack rate, supports for the *triggered* skill rather than the trigger.
- **Defensive defaults**: highly variable. CoC / CWC builds usually combine attack-or-cast-speed defensive layers (Pathfinder flask, Trickster ES recharge). CWDT setups often layer a defensive guard skill auto-fired.
- **Common pitfalls**:
  - **Cooldown bottleneck** — Cast on Critical Strike has a hard server-side cooldown that caps trigger rate; over-stacking attack speed beyond it wastes hits. Current cooldown value: `wiki_parse https://www.poewiki.net/wiki/Cast_on_Critical_Strike_Support`.
  - **Trigger source death** — channelling builds (CWC) need uptime; if interrupted, no damage.
  - **Mana sustain** — triggered spells cost mana; mana flask / leech / Indigon stacking mandatory.
- **Diagnostic checklist**: ① trigger source uptime mechanic verified (e.g., CoC attack speed × cooldown hits ratio)? ② mana sustain on the triggered spell — flask, leech, or absurd regen? ③ supports stacked on triggered spell, *not* on trigger source skill? ④ defensive guard skill (Steelskin, Molten Shell) auto-firing? ⑤ Calcs `<usePowerCharges>`/`<useFrenzyCharges>` matching what trigger generates?

### Self-cast / channelling

- **Identifying tags**: gem `spell` / `channelling`, no trigger, no totem, no mine. Direct cast rate scaling. Common ascendancies: Elementalist, Inquisitor, Hierophant.
- **Scaling levers**: cast speed (or channelling speed for channelled skills), more spell damage multipliers, conversion chains, mana sustain via leech / Indigon.
- **Defensive defaults**: mid-range EHP target, Spell Suppression cap, Cloak of Flame for phys, MoM stack often.
- **Common pitfalls**:
  - **Cast speed cap on channelled skills** — channelling has different scaling math from cast speed. Verify per skill.
  - **Mana cost runaway** — Archmage Indigon scaling spirals mana cost; sustain via flask + Mana Leech or recoup.
  - **Animation lock** — non-channelled casts have animation lock that interrupts movement; mobility planning matters.
- **Diagnostic checklist**: ① cast / channelling speed against boss-frame realistic? ② mana sustain modelled? ③ `<Calcs>` realistic charges and flask uptime? ④ defensive guard skill on CWDT? ⑤ animation cancel plan (dash / Flame Dash) for boss phases?

### Autobomber / proximity-detonate

- **Identifying tags**: corpse-engine skills (Detonate Dead, Volatile Dead, Cremation), Forbidden Rite-of-Detonation, autobomber clusters. Common ascendancies: Necromancer, Elementalist (Penance Brand chain), Trickster.
- **Scaling levers**: corpse generation rate (Cremation, Desecrate, Bone Offering, Forbidden Rite), AoE / chain count, conversion chains, autobomber uniques (Inpulsa, Asenath's gloves, Crusader chest "explode on kill").
- **Defensive defaults**: range playstyle = mid EHP. Often hybrid life + ES with phasing for repositioning.
- **Common pitfalls**:
  - **Corpse generation gap** — Detonate Dead chains require corpses; against bosses (no corpses), single-target falls off without a Cremation-style alt.
  - **Chain explosions kill the player** — explode-on-kill effects can hit the player if reflect / immunity stacking missed.
  - **Pack size dependency** — autobombers shine on dense maps; they slog on low-density tiers.
- **Diagnostic checklist**: ① corpse source for boss phases? ② chain explosion math (count of jumps in PoB Calcs)? ③ reflect / on-death mitigation stacking? ④ pack size assumption realistic for current Atlas farming target? ⑤ AoE bounded (not over-stretched into single-target sacrifice)?

### Summoner-of-summoners (Animate Guardian / Animate Weapon overlords)

- **Identifying tags**: AG and AW + a permanent army feeding from raised weapons. Niche ascendancy: Necromancer with `Mistress of Sacrifice`. Items: The Squire, Ashes of the Stars, +1 minion gem helmets.
- **Scaling levers**: minion damage ramping per AG/AW gear quality, +gem levels, support gem chain on the minion-generating skill rather than directly on the minion.
- **Defensive defaults**: low player engagement, high minion uptime. Player EHP often 5-6k hybrid.
- **Common pitfalls**:
  - **AG/AW death = unrecoverable currency loss** — AG carries 50ex+ uniques. AW disappears on death; must be re-raised.
  - **Spectre / AW desync** — AW counts can drop unnoticed.
  - **Niche viability** — these archetypes are not first-pick league-start; they need investment infrastructure.
- **Diagnostic checklist**: ① AG sustainability (life-on-minion, regen aura, flask charge)? ② AW gem level + +gems on items (helmet, chest)? ③ Bonechill + Hatred + Generosity aura support? ④ player defensive layer 6-8k EHP minimum (you can't outheal a one-shot)? ⑤ build-defining unique: Squire, Ashes of the Stars, Convoking Wand+1?

---

## PoE2 archetypes

### Combo-builder (primer + executor)

- **Identifying tags**: PoE2 Monk / Huntress / Witch with primer skills (Frost Wall, Stun-applying, Electrocute). Executor skills consume the primer status for a multiplier.
- **Scaling levers**: executor's damage modifiers (the multiplier kicks in *only* when primer is up); primer application reliability (chance to apply, AoE, frequency); combo chain length (some skills combo off other combo'd statuses).
- **Defensive defaults**: PoE2 baseline — capped resists, Spirit-fed defensive auras (Grim Feast etc. for ES), dodge-roll i-frame fluency.
- **Common pitfalls**:
  - **Primer-up uptime** — the executor's "more damage" only applies during the primer status window. Boss with high stun threshold = no executor damage.
  - **Two-skill cognitive load** — players forget the primer in panic moments.
  - **Combo chain breaking** — boss phases interrupt; planning a fallback non-combo skill helps.
- **Diagnostic checklist**: ① primer status uptime against boss target? ② executor damage measured *with primer assumed up*? ③ fallback non-combo damage exists for primer-down windows? ④ stun threshold / freeze threshold of boss high enough to break primer? ⑤ animation chain (primer cast + executor windup) within boss phase?

### Companion-pet (single high-investment companion)

- **Identifying tags**: Druid / Witch with a single specced spectre / companion (Companion tree on Druid). Items / passives boosting one companion's damage and HP.
- **Scaling levers**: companion-specific gem level, companion skill-specific supports, damage-while-companion-alive multipliers.
- **Defensive defaults**: companion is the "tank"; player relies on positioning. Spirit budget feeds companion + 1-2 auras.
- **Common pitfalls**:
  - **Companion death cascades DPS** — a dead companion = no damage. EHP investment in companion mandatory.
  - **Single-companion fragility** — backup companion / re-summon plan needed.
  - **AI / positioning** — companion doesn't always engage; manual leash mechanics matter.
- **Diagnostic checklist**: ① companion EHP — life + resist + flat mitigation? ② re-summon cooldown vs phase length? ③ companion damage scaled via gem level + supports? ④ Spirit budget allowing companion + 2 auras? ⑤ player defensive plan when companion is down?

### Herald-stack (multi-herald linker)

- **Identifying tags**: PoE2 build running 3-5 heralds on a generous Spirit budget. Tactician (Mercenary), Pathfinder occasional. Items rolling `+% reduced Spirit Reservation` or large flat Spirit.
- **Scaling levers**: number of heralds simultaneously active; Spirit reservation reduction; herald-specific damage and effect modifiers.
- **Defensive defaults**: Spirit-heavy, mana sustain still required for skill costs. Often aura-stack ES chassis.
- **Common pitfalls**:
  - **Spirit overflow** — players forget they need a meaningful Spirit headroom to fit all heralds at base reservation. Exact required headroom is herald-specific × current league's Spirit costs; `wiki_parse https://www.poe2wiki.net/wiki/Spirit` and the relevant herald pages.
  - **Herald-of-X interaction misread** — heralds explode on conditions (Herald of Ash on ignite, Herald of Ice on freeze-shatter); without the condition, they're inert.
  - **Mana-cost spiral** — heralds reserve Spirit, but skills cost mana; Indigon-style mana scaling can break the herald-stack flow.
- **Diagnostic checklist**: ① Spirit budget *with* reservation reduction modeled? ② trigger condition for each herald present (ignite for HoA, freeze for HoI)? ③ mana sustain on the main skill? ④ defensive plan when one herald is missing? ⑤ scaling herald damage vs main skill damage — which dominates?

### Weapon-set swap (Set 1 clear + Set 2 boss)

- **Identifying tags**: PoB2 with two equipped weapon sets, separate gem groups assigned via `weaponSet="1"` / `weaponSet="2"`. Book of Specialization passives allocated per set.
- **Scaling levers**: per-set passive points, set-specific itemisation (Set 1 = AoE clear weapon, Set 2 = single-target burst), swap latency.
- **Defensive defaults**: typical PoE2 — capped resists, ES + life + dodge-roll.
- **Common pitfalls**:
  - **Set-swap latency in panic moments** — fumble the swap and you're in the wrong configuration.
  - **Per-set passive duplication** — wasting points by allocating both sets to the same notable.
  - **Resist gap on weapon swap** — swapping a weapon with `+% all res` craft drops you out of cap.
- **Diagnostic checklist**: ① Book of Specialization passives non-overlapping between sets? ② resist plan covers both weapon-set configurations? ③ swap-latency understood for the current patch (fetched from `wiki_parse https://www.poe2wiki.net/wiki/Weapon_set`)? ④ Set 1 vs Set 2 designated roles (clear vs boss) clear? ⑤ animation cancel plan for set-swap mid-combat?

### Hit + ailment-stack hybrid (PoE2)

- **Identifying tags**: PoE2 build leaning on slow heavy hits with high ailment chance. Common with Warrior / Witchhunter / Pathfinder. Big swings → big ailment stacks.
- **Scaling levers**: hit damage (front-loads ailment magnitude), ailment chance (must be 100%), ailment scaling via "increased ailment damage" + "more damage with bleeds/ignites/poisons", duration, faster ailments.
- **Defensive defaults**: PoE2 typical — capped resists, hybrid ES+life, dodge-roll fluency.
- **Common pitfalls**:
  - **Sub-100% ailment chance** — RNG kills ailment uptime. Must hit 100%.
  - **Ailment magnitude vs duration confusion** — same as PoE1.
  - **Slow combat windows** — PoE2 bosses have deliberate phase pacing. Ailments need to land *during* the punish window or the magnitude is wasted on phase transitions.
- **Diagnostic checklist**: ① ailment chance at 100% against boss-tier resists? ② hit damage scaling alongside ailment damage? ③ ailment magnitude × duration > phase length? ④ proliferation source for clear? ⑤ Spirit budget feeding defensive auras while leaving room for utility?

---

## Cross-references

- `14_validation_and_failure_modes.md` — engine-trust, staleness, patch-version-awareness rules.
- `26_validation_and_self_correction.md` — extended self-checks + PoE1↔PoE2 disambiguation.
- `05_build_reasoning_framework.md` — generic build-reasoning conceptual layer.
- `07_offence_damage_scaling.md` — DPS scaling math (more vs increased, conversion order, penetration).
- `08_defence_recovery_survivability.md` — EHP / max-hit / recovery framework.
- `10_skills_gems_passives_ascendancies.md` — gem / passive / ascendancy mechanics by game.
- `25_pob_engine_integration.md` — how to verify a calculated stat once `pob_calc` ships.
- `creators_registry/` — which creator's archive to consult for each archetype.

## Open work for this doc

- **Threshold tables per archetype** — cross-link `thresholds/red_maps.md`, `pinnacle.md`, `uber_pinnacle.md` with archetype-specific bars.
- **Creator-by-archetype matrix** — quick lookup: "RF → Pohx", "Lightning Arrow Deadeye → Fubgun", "Toxic Rain Ballista → Palsteron".
- **PoE2 archetype expansion** — re-audit after 0.5 launch to add or refactor archetypes affected by new mechanics.
- **0.5-specific archetypes** — Runes-of-Aldur-flavoured archetypes once mechanic is documented.
