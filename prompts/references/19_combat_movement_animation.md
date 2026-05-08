---
description: Action speed, attack/cast speed, animation cancelling, dodge-roll-cancel windows, weapon-swap timings, leap-slam/movement-skill cancel, totem placement speed, cooldown recovery rate. The mechanics veteran players use constantly but that rarely show up in build guides.
fetch_when: User mentions "build feels clunky", "I keep dying mid-attack", "my movement is bad", or asks about specific animation/cancel mechanics; whenever the topic shifts from theoretical DPS to actual gameplay feel. Always prioritise this doc over raw DPS reasoning when the user complains about *feel* rather than *numbers*.
---

# 19 — Combat, movement, animation

This doc covers the **gameplay-feel layer** of PoE — what makes a build *feel* good or bad regardless of its stat sheet. PoB will tell you "5M DPS, 12k EHP" and the build will still feel awful if the player can't cancel into a dash or repositions during a 0.6s attack-windup. Most guides under-serve this layer; veteran players rely on it constantly.

The single most useful question this doc answers: *"Why does this build feel bad even though the numbers look right?"*

## Why feel matters separately from stats

Three structural reasons:

1. **Animation lock** — every skill has a startup + recovery window. If a 6M-DPS build has 600ms attack animation and no cancel option, it loses to a 4M-DPS build with 350ms + cancel-into-dash.
2. **Reactivity floor** — bosses telegraph attacks in fixed time windows. If your build can't dodge in that window, no amount of EHP saves you.
3. **Effort-to-reward ratio** — players quit builds that feel bad. The most-played builds in any league are usually feel-good more than highest-DPS.

## Action speed (the master stat — PoE1)

**Action speed** is the multiplier on **everything you do**: attack speed, cast speed, movement speed, animation speed. It is rare on items but is the highest-leverage feel-stat in the game.

- Sources: Tailwind (Charm of Tailwind boots Eldritch implicit, Pathfinder ascendancy node, Quicksilver flask), Onslaught (item mods, flasks, gems), specific uniques.
- Stacking principle: each independent action-speed source is multiplicative with attack/cast/move speed. A 20% Tailwind + 20% Onslaught + 100% attack speed = significantly faster than just 140% attack speed.

## Attack speed and cast speed (PoE1 / PoE2)

These are the *narrow* feel-stats. They scale only their respective category.

- **Attack speed**: scales melee swings, bow shots, attack-tagged movement skills (Whirling Blades, Shield Charge, Leap Slam in PoE1).
- **Cast speed**: scales spell windups, channelling tick-rate (with caveats), Spell Echo gem.
- **Quality on weapons** (PoE1) typically grants flat attack speed; quality on gems often grants attack speed too.

### Breakpoints to watch

- **Sub-1.0 attacks/sec total**: feels stiff, hard to react.
- **3-4 attacks/sec**: comfortable mapping.
- **5+ attacks/sec**: clearspeed-tier; very smooth.
- **Cast speed for trigger setups**: trigger has a fixed cooldown; cast speed beyond that wastes mana with no DPS gain.

## Animation cancelling (PoE1)

Several skills can be **canceled into another skill** mid-windup, freeing the player from animation lock. This is one of the most important veteran-tier feel skills.

| Skill | Cancel pattern |
|---|---|
| Dash / Flame Dash | Cancellable into any skill at any frame after startup. |
| Leap Slam | Cancellable into another attack mid-arc. |
| Whirling Blades | Mostly chained; cancel into attack at end of dash. |
| Shield Charge | Cancellable into final hit on target. |
| Cast When Damage Taken (CWDT) | Auto-fires; doesn't lock animation. |

**Diagnostic for "build feels clunky"**: ask which movement skill the user is binding. If they're using Leap Slam without Faster Attacks support, or no movement skill at all, that's almost always the culprit.

## Movement skills (PoE1 — by archetype)

| Skill | Best for | Notes |
|---|---|---|
| Flame Dash | Caster, ranged, MoM stack | 3 charges, 10s recovery; Vaal Flame Dash for emergency. |
| Dash | Generic | Smaller distance; cooldown-based. |
| Leap Slam | Strength-attack builds | Attack-speed scaled; mounts onto targets. |
| Shield Charge | Block / phys-stack | Attack-speed scaled, knockback. |
| Whirling Blades | Dual-wield / dagger | Attack-speed scaled; chains seamlessly. |
| Frostblink | Ice / cold-themed | Cooldown-based but instant. |
| Lightning Warp | Niche | Pre-windup delay; rarely used. |

**Build-feel principle**: every endgame build should bind ≥1 movement skill on left-click + 1 emergency-button (Vaal Flame Dash, Phase Run, etc.).

## Totem placement speed (PoE1)

Separate stat from cast speed. Matters for Hierophant / Chieftain / Soulwrest builds that spam totems.

- Sources: cluster jewel notable (`Place Totem 16% faster`), specific gloves implicit, Hierophant ascendancy.
- **Diagnostic**: a totem build that "feels slow" usually has cast speed but no totem placement speed. The two stack but solve different problems.

## Cooldown recovery rate (PoE1 / PoE2)

A separate stat that scales the rate at which **cooldowns** of skills/flasks recharge.

- Affects: Flame Dash (3 charges), Vaal skill recovery (rare), specific guard-skill recoveries (Steelskin, Molten Shell).
- **Build-feel impact**: a Flame Dash with 50% cooldown recovery feels dramatically smoother than one with 0%. Often missed by sub-optimal gear.
- Belt + Boots Eldritch implicit (Searing Exarch) commonly grants this.

## PoE2 — distinct feel layer

PoE2 deliberately chose a **slower, more committal combat** model. Veteran PoE1 instincts often produce wrong feel-judgments on PoE2 builds.

### Dodge roll fundamentals

- **Dodge roll** is the universal movement / defensive primitive in PoE2.
- I-frames during the roll = damage immunity window.
- Default dodge has a fixed cooldown (no recovery rate to speed it up by default).

### Dodge-roll cancel windows

- Most attack/skill animations can be canceled into a dodge roll mid-windup.
- A trained player threads dodges between every skill use.
- **Build-feel impact**: a build that doesn't bind dodge to a comfortable key dies dramatically more.

### Weapon-swap timing

- Switching Weapon Set 1 ↔ Weapon Set 2 has a fixed animation (~250ms; verify per current version).
- Frequent swap-builds need a "buffer skill" planted before the swap to absorb the lock.
- Skills auto-bound to specific weapon sets: a skill in Set 2 sockets cannot fire while Set 1 is active.

### Combo skill timing (Monk / Huntress / Druid)

PoE2 introduces the **combo** framework: a "primer" skill applies a status (freeze, electrocute, stun, ignite, primal mark), an "executor" skill consumes it for amplified damage.

- **Primer uptime is the limiting factor**. A 10M-burst executor on a build that cannot maintain primer is 4M effective DPS.
- Window between primer apply and executor fire: 1-3 seconds typical (verify per skill / current patch).
- **Diagnostic**: when a Monk / Huntress build "feels weak vs bosses", the issue is usually primer uptime, not executor scaling.

### Channelling and persistent skills

- Channelling skills (Eye of Winter etc.) lock animation while active. Cancel-into-dodge protocol is mandatory for survivability.
- Persistent skills (Heralds, Spirit-aura buffs) don't lock animation but compete for Spirit budget — see `poe2/01_spirit_economy.md`.

## Common gameplay-feel diagnostics

When a user says "this build feels off", run through these checks:

1. **Movement skill bound?** No → bind one. Yes but feels slow → check attack/cast speed scaling that movement skill.
2. **Cooldown recovery rate?** Some builds need 30-50% to be smooth.
3. **Animation cancel pattern in use?** PoE1: dash + skill rotation. PoE2: dodge between skills.
4. **Action speed sources?** Tailwind / Onslaught / Adrenaline often the missing layer.
5. **Cast / attack speed at floor (≥3.0/sec for clearing)?** Below = feels clunky.
6. **(PoE2) Dodge cooldown manageable?** Some builds need cooldown recovery on dodge specifically.
7. **(PoE2) Combo primer uptime modeled?** If executor > primer apply rate, feel suffers.
8. **(PoE2) Weapon-swap interrupt patterns?** A swap mid-fight without a buffer skill = hit window for boss.

## Cross-references

- `07_offence_damage_scaling.md` — DPS theory; complements feel.
- `08_defence_recovery_survivability.md` — defensive counterparts to dodge mechanics.
- `17_build_archetype_taxonomy.md` — per-archetype feel profile.
- `26_validation_and_self_correction.md` Rule 4 — PoE1↔PoE2 feel differences (don't apply PoE1 cancel patterns to PoE2 builds).
- `poe2/00_version_pinning.md` — current PoE2 version.
- `poe2/02_weapon_sets.md` — weapon-swap specifics.
