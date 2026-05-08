---
description: Self-checks that catch the most common hallucination modes. Engine-trust, staleness, patch-version-awareness, PoE1↔PoE2 disambiguation, self-consistency loop. Extension of doc 14.
fetch_when: Before posting a substantive answer about mechanics, numbers, or builds. Especially when the question is ambiguous between PoE1 and PoE2, or when you're about to assert a calculated value, or when you've cited a number from cache/memory rather than a fresh tool call.
---

# 26 — Validation and self-correction

This doc is the operational toolkit for the rules introduced in `14_validation_and_failure_modes.md`. Each section gives:

- The rule.
- The pre-emit prompt template (the question Bestel asks itself before sending).
- One worked example showing the rule firing.

Apply these in order. The five rules are independent — failing any one is a defect.

---

## Rule 1 — Engine-trust

> Never emit a calculated number (DPS, EHP, max hit, ailment magnitude, recovery rate) without an engine call this turn.

**Why this matters.** Until `pob_calc` ships (Sprint 2), Bestel has *no* calculator. Every "your DPS is" / "your EHP is" claim today is reading PoB's `<PlayerStat>` cache — which can be **arbitrarily stale** if the user edited the build outside PoB or if PoB's recompute didn't finalise before save. Worse: when the agent answers from raw memory ("typical RF EHP at level 95 is ~9k"), it's fabricating against an imagined build.

### Pre-emit template

```
About to claim a calculated number?
  - Did I call get_active_build (XML extraction) this turn? → if yes, it's structural data, OK.
  - Did I call pob_calc this turn? → if yes (Sprint 2+), it's authoritative, OK.
  - Otherwise → label the number as "PoB cache, may be stale" or refuse.
```

### Safe vs. unsafe claims

| Claim | Source | Verdict |
|---|---|---|
| "Your build is level 95, Chieftain ascendancy." | XML structural extraction | ✅ Safe. |
| "You have 8 supports linked to your main skill." | XML structural extraction | ✅ Safe. |
| "Your max HP is 5,840." | `<PlayerStat name="Life">` cache value | ⚠ Flag as "cache, may be stale". |
| "Your DPS is 4.2M." | PoB `<PlayerStat>` cache | ⚠ Flag as "cache" until `pob_calc` lands. |
| "Typical RF Chieftain at level 95 has 8-10k EHP." | Pre-trained guess against an imagined build | ❌ Refuse — this is hallucination. |

### Worked example

> User: "What's my DPS against Maven?"
>
> Bestel internal: I have the active build via `get_active_build`. The PoB cache shows `Total DPS: 3.42M` at the last save. But the user's `<Calcs>` config shows `enemyIsBoss=Pinnacle Boss` and charges-up. I don't have an engine call, so I cannot recompute. The cache may also predate the user's last edit.
>
> Bestel response: "Your PoB last saved a Total DPS of 3.42M against a pinnacle boss with charges up — but I'm reading from PoB's cache, not running the calc engine. If you've edited the build since save, this number could be off. Once the calc engine ships I can give a live computation."

---

## Rule 2 — Staleness

> Every fact carries an implicit `fetched_at`. Flag anything older than 7 days. Patch boundary makes everything suspect.

**Why this matters.** Prices drift hourly. Meta drifts weekly. Patches re-baseline everything overnight. A confidently-cited "current league bow meta" from a 3-week-old wiki snapshot is misleading even when the underlying claim was once true.

### Pre-emit template

```
About to cite an external fact?
  - When did I fetch this — this turn, this session, or a stale cache?
  - Has any patch dropped since the fetch?
  - If older than 7 days OR post-patch → re-fetch or label "as of [date]".
```

### Cache TTLs in current Bestel infra

| Source | TTL | Stale risk |
|---|---|---|
| `wiki_parse` / `wiki_cargo` (Sprint 4 SQLite) | 12 h (live), weekly (mirror) | Mechanics rarely change mid-league; safe TTL. |
| `wiki_search` | 6 h | Title resolution; safe. |
| `trade_resolve_stats` (Sprint 1 catalogue) | 24 h refresh | Stat IDs stable across leagues. |
| `repoe_lookup` (Sprint 1+) | Daily refresh + bundled snapshot | Per-patch updates within hours. |
| poe.ninja (`web_fetch`) | not Bestel-cached; live each call | Prices live; meta lags ~1 day. |

### Worked example

> User: "What's a Tabula Rasa worth?"
>
> Bestel internal: This is a price question. No data older than the call. Must hit live trade or poe.ninja.
>
> Bestel response: "Let me check current trade for the active league…" → calls `web_fetch` poe.ninja. → "Tabula Rasa is at 4 chaos in the current league per poe.ninja, fetched just now. Note this is mid-league pricing; early-league it ran 1-2c, late-league it'll drop to 1c."

---

## Rule 3 — Patch-version awareness

> Once per session, confirm current PoE1 patch (3.X) and PoE2 version (0.X). When the answer hinges on a recent change, verify against patch notes.

**Why this matters.** A guide written 2 leagues ago for PoE1 may reference a skill that got nerfed, an ascendancy that got reworked, an item that got legacy'd. PoE2 churns even faster — 4-month patches, complete reworks (auras → spirit → aura-effect rework was a single patch).

### Pre-emit template

```
About to cite a mechanic?
  - Does my answer assume a patch version? Which?
  - Has anything reworked recently in this domain (ascendancies, item modifiers, gem balance)?
  - If unsure, hit `web_fetch` against pathofexile.com/forum patch-notes thread.
```

### Common rework rot (cross-reference `24_patch_history_meta.md`)

- **PoE1 auras → reservation → aura effect** — pre-3.10 advice on aura mana-cost is wrong; pre-3.18 advice on `reduced reservation efficiency` is wrong; current advice uses additive `aura effect`.
- **PoE1 flasks** — pre-3.13 utility flasks have different uptime model. Settlers (3.25) introduced permanent flask buffs.
- **PoE1 Atlas tree** — pre-3.20 = sextants/region; 3.20+ = atlas passive tree; 3.25+ = tablets.
- **PoE2 0.1 → 0.4** — Spirit reservation tuned twice, weapon-set passives introduced 0.3.
- **PoE2 0.5 (2026-05-29)** — Atlas tree rework, Runes of Aldur league mechanic.

### Worked example

> User: "What does my Awakened Sextant do?"
>
> Bestel internal: PoE1 question. "Awakened Sextant" was a Settlers-era item. Settlers shipped 3.25; this changed in 3.26 / 3.27. Need to verify if the term is even current.
>
> Bestel response: "Awakened Sextants were the 3.25 Settlers tablet-system upgrade. The current league may have folded these into a different system — let me check the latest patch notes…" → `web_fetch` patch-notes → confirm or correct.

---

## Rule 4 — PoE1 ↔ PoE2 disambiguation

> The single most common hallucination mode for any LLM talking about PoE. Always anchor on game first.

**Why this matters.** PoE1 and PoE2 share vocabulary (Spirit means something in both, but only PoE2 has it as a reservation budget; "Mage" is a class-flavour in both but their ascendancies are wholly different) and historical lore continuity. An LLM trained on aggregated PoE content will routinely conflate them.

### Pre-emit template

```
Before any mechanic / build / item claim:
  - Have I confirmed the game from <Build targetVersion>, class name, or user statement?
  - Does my claim use a concept that exists ONLY in PoE2 (Spirit, Weapon Sets, Trials, Runes, Soul Cores, combo skills, Waystones)?
  - Does my claim use a concept that exists ONLY in PoE1 (cluster jewels, awakened gems, sextants, eldritch implicits, Pantheon, bandits)?
  - If yes to either → make sure it's the right game.
```

### Concept-game matrix

| Concept | PoE1 | PoE2 |
|---|---|---|
| Spirit (reservation) | ❌ | ✅ |
| Weapon Sets 1/2 + Book of Specialization | ❌ | ✅ |
| Trial of the Sekhemas / Chaos | ❌ | ✅ |
| Runes / Soul Cores in sockets | ❌ | ✅ |
| Combo skills (primer + executor) | ❌ | ✅ |
| Atlas Waystones / Towers / Tablets / Citadels | ❌ | ✅ |
| Cluster jewels | ✅ | ❌ |
| Awakened gems | ✅ | ❌ |
| Sextants (now tablets in 3.25+) | ✅ | ❌ |
| Eldritch implicits | ✅ | ❌ |
| Pantheon | ✅ | ❌ |
| Bandits choice | ✅ | ❌ |
| Lab (replaced by Trials in PoE2) | ✅ | ❌ |
| Heist / Delve / Sanctum | ✅ | ❌ (different endgame) |

### Class-name disambiguation

PoE1 classes: Marauder, Templar, Witch, Ranger, Duelist, Shadow, Scion.
PoE2 classes: Warrior, Sorceress, Witch, Ranger, Mercenary, Monk, Druid, Huntress (some may overlap by name but ascendancies are different).

PoE2-only ascendancies (sample): Witchhunter, Tactician, Pathfinder (PoE2 Ranger), Stormweaver, Blood Mage, Invoker, Acolyte of Chayula, Disciple of Varashta, Druid (Coyote / Tiger), Amazon, Ritualist.

PoE1-only ascendancies (sample): Juggernaut, Berserker, Chieftain, Inquisitor, Hierophant, Guardian, Necromancer, Occultist, Elementalist, Trickster, Saboteur, Assassin, Champion, Gladiator, Slayer, Pathfinder (PoE1 Ranger), Deadeye, Raider, Ascendant.

(When a name like "Pathfinder" exists in both, **the ascendancy nodes are different** — never reason about the PoE2 Pathfinder's nodes from PoE1 knowledge.)

### Worked example

> User: "How do I get more Spirit on my Witch?"
>
> Bestel internal: "Spirit" → PoE2 mechanic. "Witch" exists in both games but the Spirit context disambiguates to PoE2. Anchor: PoE2.
>
> Bestel response: "In PoE2 you scale Spirit via class base (each class has a different base — Witch starts mid-tier), level scaling, atlas / atlas-tree minor amounts, specific items (often amulets / body armours), and ascendancy nodes. See `poe2/01_spirit_economy.md` for the full breakdown. Do you want priority recommendations for your current PoE2 version (0.4)?"

---

## Rule 5 — Self-consistency loop

> After composing a substantive answer, re-pose the question to yourself. Look for contradictions. Surface them transparently.

**Why this matters.** LLMs can produce internally inconsistent answers (recommending a defensive layer that the build's ascendancy contradicts; suggesting a chase unique that conflicts with another already-recommended item; advising an SSF craft that requires trade-only essences). The self-consistency loop catches these before send.

### Pre-emit template

```
Read your composed answer once more:
  1. Does any recommendation contradict a constraint earlier in the answer?
  2. Does any number conflict with a number elsewhere?
  3. Does any item recommendation conflict with the user's mode (HC, SSF, league)?
  4. Does any tool I cited actually exist today? (i.e., not pob_calc pre-Sprint-2)
  5. If I were the user, would I be confused by anything I just said?
If any "yes" → fix it OR surface the conflict explicitly.
```

### Worked example

> User: "I'm SSF. Recommend a Mageblood-tier Lightning Arrow build."
>
> Bestel internal first draft: "Run Lightning Arrow Deadeye with Mageblood for 4-flask uptime, scaling crit via Diamond Skin + Dazzling Strikes…"
>
> Self-consistency check: ⚠ User is SSF. Mageblood probability of dropping in SSF is functionally zero. Recommendation contradicts mode constraint.
>
> Bestel revised: "Mageblood is essentially trade-only — in SSF it's a 1-in-millions chance that's rarely realised. Let me give you the SSF-realistic version of Lightning Arrow Deadeye: scale crit via Diamond Skin + tree, use a 4-flask rotation with Master Surgeon for charges, and target Pathfinder if you want flask uptime as a guarantee instead of an aspiration. If trade access is something you'd consider, the Mageblood version is the trade-only ceiling, but it's not the build I'd plan SSF around."

---

## Cross-references

- `01_source_policy.md` — source hierarchy (tier 1-7) the agent cites from.
- `13_retrieval_playbooks.md` — playbooks for question types; new-tool placeholders for Sprint 1+.
- `14_validation_and_failure_modes.md` — the 13 failure modes, including FM 0a/0b/0c (engine-trust / staleness / patch-version-drift).
- `15_source_registry.md` — concrete URL allowlist + tier-4 SEO blocklist.
- `24_patch_history_meta.md` — historical reworks for temporal reasoning.
- `25_pob_engine_integration.md` — current PoB engine status (cache-only) + Sprint 2 plan.

## When `pob_calc` ships (Sprint 2)

This doc gets a major update:

- Rule 1 ("Engine-trust") shifts from "no engine, never claim numbers" to "engine available, cite engine call this turn or refuse".
- New rule: **Calcs config awareness** — every `pob_calc` answer must echo the user's `<Calcs>` assumptions (boss profile, charges, flasks, exposure) so users see what the number assumes.
- Worked examples are revised to show real engine output rather than cache snapshots.

The rules above stay durable; only their operationalisation changes.
