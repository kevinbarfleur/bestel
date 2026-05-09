# Side panel markers — `⟦panel:type:name⟧`

This reference holds the full panel-marker grammar. The runtime contract in `SYSTEM_PROMPT.md` carries only the placement rule and the minimum payload; everything below — payload shapes, worked examples, REQUIRED triggers, optional click-to-open variants — lives here so the system prompt stays small. Fetch this ref when you need the exact JSON schema for a panel type, or when you want to study a worked example before composing a marker.

Bestel has a side panel on the right of the chat. You do **NOT** open it with
a tool call. Instead, you embed two things directly in your **final answer
text**:

1. **A hidden sidecar block at the very TOP of the answer** (before any
   prose) carrying the typed payloads keyed by name.
2. **Inline markers** at the position where each panel button should
   appear in the prose, embedded INSIDE a sentence that's actually about
   that entity.

The UI parses the sidecar incrementally as it streams, then renders each
marker as a small loupe-icon button when it arrives in prose. The
**primary** marker (`⟦panel*:…⟧`) auto-opens its panel **the moment it is
streamed**, mid-reply, so the structured view is already on screen when
the exile reads the surrounding sentence.

## Where to place the marker — the conversational anchor

The marker sits at the moment the entity becomes the conversational
focus — never elsewhere. Two patterns:

- **Deep-dive** ("tell me about X") — emit the marker at the end of the
  first or second sentence that introduces X:
  > Mageblood is a Heavy Belt that constantly applies the effects of
  > your leftmost magic utility flasks ⟦panel*:item-card:Mageblood⟧.

- **Swap recommendation** — emit the marker at the moment you name the
  proposed item, INSIDE the recommendation sentence, NEVER at the start
  of a sentence that describes the exile's current gear:
  > Your `Storm Hide` is fine for clear but the explode procs collide
  > with your phys-to-lightning conversion. Swap it for a
  > ⟦panel*:item-card:Carcass Jack⟧ — `+50%` area of effect on the
  > body slot lets you trigger explosions before the pack disperses.

NEVER do this:

> Your `Storm Hide` ⟦panel*:item-card:Carcass Jack⟧ is a rare Crusader
> Sacred Chainmail that gives `+104` life…

The marker is interpreted by the reader as the visual anchor for the
entity it carries (Carcass Jack). Placing it next to a different entity
(Storm Hide) creates a parse failure — the exile reads "Storm Hide
[Carcass Jack button] is rare" and cannot tell which item the sentence is
about. The recommendation panel pops up while you're still describing
the OLD item.

## Coexistence with backtick wiki pills — panel marker WINS

Bestel's chat surface renders backticked entities as small clickable
pills (chain icon → wiki webview). When an entity also has a panel
marker, the wiki pill is REDUNDANT — the panel itself carries an
"open external" button that takes the exile to the wiki page from
inside the panel. To avoid the double-button clutter:

- **If an entity has a panel marker anywhere in your answer, do NOT
  also wrap its name in backticks.** The panel button supersedes the
  wiki pill — its loupe icon is the only affordance for that entity.
- Backtick wiki pills are reserved for entities that do NOT have a
  panel marker (the exile's current gear when discussing a swap, named
  synergy candidates listed in passing, etc.).

Example — discussing a swap:

> Your `Storm Hide` is fine for clear but collides with your phys-to-
> lightning conversion. Swap it for a ⟦panel*:item-card:Carcass Jack⟧
> — `+50%` area of effect lets you trigger explosions before the pack
> disperses. Pair with `Inpulsa's Broken Heart` chest mod for the
> chain.

`Storm Hide` and `Inpulsa's Broken Heart` get backtick pills (no
panel). `Carcass Jack` gets ONLY the panel marker (no backticks
anywhere). When the exile clicks the panel button and wants the wiki
page for Carcass Jack, the panel chrome has an external-link icon for
that.

Markers must NEVER appear in your reasoning — only in the final
user-facing answer.

## Inline marker grammar

```
⟦panel:<type>:<name>⟧            — click-to-open button
⟦panel*:<type>:<name>⟧           — at most ONE per message; auto-opens
                                   at message finalization
```

- `<type>` is one of `item-card`, `gem-detail`, `mechanic`, `markdown`.
- `<name>` is the human-readable title shown on the button AND the key into
  the sidecar JSON. Case-sensitive — must match exactly.
- The starred `⟦panel*:…⟧` variant marks the **primary** artifact for the
  message — the one most worth studying. Use it sparingly; only use a star
  when the panel is the centerpiece of the answer (e.g. "tell me about
  Mageblood" — Mageblood is the answer). Most messages should use **no
  star** at all and let the exile click whichever button interests them.
- A button without a matching sidecar entry renders disabled.

## Sidecar block — emit FIRST, before any prose

The sidecar is the FIRST thing in your final answer when you use panel
markers. Before any greeting, narration, or prose, emit exactly one
block on its own lines:

```
⟦panel-data⟧
{
  "<name>": {
    "type": "item-card",
    "title": "Marble Amulet",
    "payload": { ... }
  },
  ...
}
⟦/panel-data⟧
```

Then write your prose with the markers embedded at conversationally
appropriate moments, then end with `Sources:`. Why first? The UI
auto-opens the primary panel **at the moment the marker is streamed**
into prose — the exile sees the panel pop open as you reach the
sentence that introduces the entity, not after the full answer
finishes streaming. For that to work, the data has to be already
parsed when the marker arrives. Emit the sidecar first.

The UI strips this block from rendered prose — the exile never sees the JSON.
Keys MUST match the `<name>` of every `⟦panel:…:<name>⟧` marker in the
prose; missing keys = disabled buttons.

## Payload shapes

```
type: 'item-card'
payload: {
  name, base, rarity, ilvl?, slot?,
  mods: [{ kind: 'implicit'|'enchant'|'explicit'|'crafted'|'fractured', text }],
  comparison?: {
    replaces,
    deltas: [{ stat, delta, tone: 'good'|'bad'|'note' }]
  }
}

type: 'gem-detail'
payload: {
  name, level?, quality?,
  tags: ['Spell', 'Cold', 'Aura', ...],
  scaling: [{ stat, value }],
  recommended_supports?: [{ name, role }]
}

type: 'mechanic'
payload: {
  summary: 'one-line essence',
  sections: [{ heading: 'Caps', body_md: '...' }, { heading: 'Interactions', body_md: '...' }]
}

type: 'markdown'
payload: { body_md: '...' }
```

## Worked examples

**Item swap (uses primary marker — the swap is the centerpiece):**

> ⟦panel-data⟧
> {
>   "Marble Amulet": {
>     "type": "item-card",
>     "title": "Marble Amulet",
>     "payload": {
>       "name": "Marble Amulet",
>       "base": "amulet",
>       "rarity": "rare",
>       "ilvl": 84,
>       "slot": "amulet",
>       "mods": [
>         { "kind": "implicit", "text": "+22 to Strength" },
>         { "kind": "explicit", "text": "+45% to Chaos Resistance" },
>         { "kind": "explicit", "text": "+22% to Cold Resistance" },
>         { "kind": "explicit", "text": "+68 to maximum Life" }
>       ],
>       "comparison": {
>         "replaces": "Stranglegasp",
>         "deltas": [
>           { "stat": "chaos res", "delta": "+33%", "tone": "good" },
>           { "stat": "abyss sockets", "delta": "−2", "tone": "bad" }
>         ]
>       }
>     }
>   }
> }
> ⟦/panel-data⟧
>
> The amulet slot is your weak point, exile. Swap `Stranglegasp` for a
> ⟦panel*:item-card:Marble Amulet⟧ — it pushes chaos res back to cap and
> hands you `+68` life on top.
>
> Sources:
> - [Wiki: Marble Amulet](https://www.poewiki.net/wiki/Marble_Amulet)

**Deep-dive on a single unique (the most common case — primary marker is REQUIRED):**

> ⟦panel-data⟧
> {
>   "Mageblood": {
>     "type": "item-card",
>     "title": "Mageblood",
>     "payload": {
>       "name": "Mageblood",
>       "base": "Heavy Belt",
>       "rarity": "unique",
>       "ilvl": 75,
>       "slot": "belt",
>       "mods": [
>         { "kind": "explicit", "text": "+(25-35) to Strength" },
>         { "kind": "explicit", "text": "+(30-50) to Dexterity" },
>         { "kind": "explicit", "text": "+(15-25)% to Fire Resistance" },
>         { "kind": "explicit", "text": "+(15-25)% to Cold Resistance" },
>         { "kind": "explicit", "text": "Magic Utility Flasks cannot be Used" },
>         { "kind": "explicit", "text": "Leftmost (2-4) Magic Utility Flasks constantly apply their Flask Effects to you" }
>       ]
>     }
>   }
> }
> ⟦/panel-data⟧
>
> Mageblood ⟦panel*:item-card:Mageblood⟧ is a Heavy Belt that constantly
> applies the effects of your leftmost 2–4 magic utility flasks without
> consuming charges. The mandatory `Alchemist's` prefix and `Enkindling Orb`
> enchantment together grant `+95%` increased flask effect…
>
> Sources:
> - [Wiki: Mageblood](https://www.poewiki.net/wiki/Mageblood)

Note that `Mageblood` is NOT wrapped in backticks anywhere in the
prose — only the panel marker carries it. The panel button is the
only affordance for that entity; its chrome includes an external-link
icon that opens the wiki page when the exile wants it.

**Mechanic explainer (no primary — exile can study the panel if they want):**

> ⟦panel-data⟧
> {
>   "Spell Suppression": {
>     "type": "mechanic",
>     "title": "Spell Suppression",
>     "payload": {
>       "summary": "A defense layer that reduces incoming spell damage by 50% on a successful suppression check.",
>       "sections": [
>         { "heading": "Cap", "body_md": "Caps at `100%` chance to suppress. The damage reduction itself is fixed at 50% (raisable via specific keystones)." },
>         { "heading": "Stacks with", "body_md": "Spell suppression is **multiplicative** with spell block, so layering both is strong against caster bosses." }
>       ]
>     }
>   }
> }
> ⟦/panel-data⟧
>
> Spell suppression is a defense layer ⟦panel:mechanic:Spell Suppression⟧ —
> on a successful check, incoming spell damage is reduced by `50%`. It caps
> at `100%` chance and stacks **multiplicatively** with spell block.
>
> Sources:
> - [Wiki: Spell Suppression](https://www.poewiki.net/wiki/Spell_suppression)

## When to use markers — REQUIRED triggers

The panel marker is **NOT optional** in any of these cases. Always emit it:

1. **Deep-dive on a single entity.** If the exile asks `"tell me about <X>"`,
   `"what is <X>"`, `"explain <X>"`, or any equivalent where `<X>` is a
   single named unique, gem, keystone, ascendancy notable, or mechanic,
   emit ONE primary marker `⟦panel*:<type>:<X>⟧` for that entity. The
   panel IS the structured deep-dive — without the marker, the exile gets
   prose only. Pick the most precise type:
   - Unique item (e.g. `Mageblood`) → `item-card`.
   - Skill or support gem (e.g. `Spark`) → `gem-detail`.
   - Mechanic / keystone (e.g. `Spell Suppression`, `Divine Flesh`) → `mechanic`.
   - Anything else worth a structured artifact → `markdown`.

2. **Item swap with mods + comparison.** When you recommend swapping a
   specific gear piece, emit a primary `⟦panel*:item-card:<NewItem>⟧`
   marker carrying the proposed item, its mods, and a `comparison` block
   against the current item. The panel is the swap proposal.

3. **Mechanic-driven build advice.** When the answer hinges on one mechanic
   (`Divine Flesh`, `Spell Suppression`, `Avatar of Fire`), emit a primary
   `⟦panel*:mechanic:<Name>⟧` so the structured breakdown is one click away.

## Optional click-to-open markers

For secondary entities you mention in passing — synergy candidates,
alternative items, related gems — you MAY emit non-primary markers
`⟦panel:<type>:<name>⟧` (no star). At most one primary per message; any
number of non-primaries. Each marker MUST have a matching key in the
`⟦panel-data⟧` sidecar or it renders disabled.

## Don't go overboard

Skip markers for:
- Short tactical advice or one-line answers.
- Tool-result summaries (the wiki tool already renders an artifact card).
- Sentences already complete on their own in the chat.

The chat should not become a button forest. One primary + 0–3 non-primaries
is the typical envelope.
