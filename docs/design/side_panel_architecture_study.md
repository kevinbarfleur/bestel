# Side panel — architecture study

Design doc for the maintainer. **No implementation in this pass.** Goal: reframe the
right side panel from an *agent-tool artifact* to a *call-to-action embedded in
Bestel's final answer*.

---

## 1. The mistake we're correcting

Today the panel is wired to the **tool dispatch cycle**. The agent calls
`show_in_panel` mid-turn; the streaming pump (`useStreaming.ts`) parses the tool
output on `tool_end` and pushes a `PanelArtifact` onto the UI stack. The panel pops
*while Bestel is still reasoning / calling other tools*. From the user's seat:

- Panel pops randomly during reasoning, often before any prose has arrived.
- Content == "what the wiki tool happened to scrape", not "what Bestel decided
  to highlight in the answer".
- No relationship between the panel and the *final paragraph* the user is reading.
  By the time the answer is written, the panel may already have been replaced 2x.

The user's actual intent (literal quote, translated):

> The side panel should be a panel that opens at **specific moments of the final
> answer**, of Bestel's **official reply**. Not during reasoning. While he is
> writing the final answer with links — alongside the links, also panel buttons.

So a panel button is a **CTA inside Bestel's prose**, equivalent to a wiki link
pill, but instead of opening a browser/webview it opens an enriched detail card
on the right with optional secondary actions ("Open trade query", "Allocate in
tree", future).

Concrete excitement case: *"Swap your boots to `The Stampede`"* in the answer
should render `The Stampede` as both a wiki pill *and* a side-panel button; the
panel shows the item card + an **"Open trade query"** button that pre-fills a
poe.trade search with sensible defaults.

---

## 2. Audit — current state, honestly

### 2.1 What works and should survive

- `PanelArtifact` model + stack semantics in `stores/ui.ts` (id-keyed dedup, back
  history, close-clears-stack). Solid; reuse as-is.
- `PanelView.vue` + the four artifact components (`PanelItemCard`,
  `PanelGemDetail`, `PanelMechanic`, `PanelMarkdown`). Type-specific renderers
  with tolerant payload coercion. Reuse.
- Streaming-phase tolerant JSON parse (`parsePanelHint`) — handles incomplete
  output. Pattern reusable for new inline-marker parser.
- `markdown.ts` already turns single-backtick spans into wiki pills via a custom
  `code_inline` rule. The mechanism we need is a sibling rule.

### 2.2 What's wrong for the new vision

| Symptom | Root cause |
|---|---|
| Panel pops during reasoning | Promotion happens on `tool_end`, not on assistant text |
| Panel content == raw wiki dump | `show_in_panel` payloads are produced by the agent reflexively after a `wiki_parse` call |
| User can't anchor a panel to a specific sentence in the answer | No structural link between an inline mark in prose and the panel state |
| No CTA inside the panel (e.g. "open trade query") | Panel components are passive renderers with no action surface |
| Streaming triggers premature open | Output JSON arrives in one chunk → opens immediately, even mid-stream |
| Two distinct user intents merged into one tool | "I researched X" vs "exile, study X" — same `show_in_panel` call serves both |
| Inline pill duplicates info | Today the chat shows a `[highlighted: title]` pill (acts as re-open button); but the panel was already opened by the agent. The pill is a fallback, not the trigger. |

### 2.3 Today's pill — what it actually is

Look at `ChatMessage.vue`:

```ts
function reopenInPanel(seg: ToolSegment) { ... ui.openPanel({ ...source: 'click' }) }
```

The `turn--tool-panel` button is rendered inline **at the position of the
`show_in_panel` tool call** in the agent stream. So it currently sits *before*
the final answer prose — not within it. The right design swaps this around: no
inline pill at the tool-call position, instead a marker injected by the agent
into the assistant text stream itself.

---

## 3. The new vision in one diagram

```
                     ┌────────────────────── chat (left) ──────────────────────┐
USER  ──>            │ "swap boots?"                                            │
                     │                                                          │
ASSISTANT  ──>       │ thinking… (reasoning)                                    │
                     │ tool: get_active_build  ✓                                │
                     │ tool: wiki_parse(The Stampede)  ✓     ← these are silent
                     │ tool: trade_resolve_stats  ✓             markers, no panel
                     │ ────────────────────────────────────────                 │
                     │ Bestel:                                                  │
                     │   The boots burdening you, exile, are                    │
                     │   `Two-Toned Boots` — adequate but plain.                │
                     │   Trade them for [⇗ The Stampede]  ← side-panel BUTTON  │──> opens right panel
                     │   ([wiki](https://...))                                  │     with item card
                     │                                                          │     + "Open trade"
                     │   Sources: …                                             │
                     └──────────────────────────────────────────────────────────┘
                                                                                    ┌─ panel (right) ─
                                                                                    │ The Stampede
                                                                                    │ Reinforced Greaves
                                                                                    │ ───── mods ─────
                                                                                    │ +20% movement…
                                                                                    │ ───── actions ─
                                                                                    │ [Open trade]
                                                                                    │ [Open wiki]
                                                                                    └─────────────────
```

Two flows are now **fully separated**:

1. **Reasoning artifacts** (wiki_parse output, tool dumps) — stay in the chat
   stream as `ToolCallBadge` / `ArtWikiPage`, never auto-open the panel.
2. **Answer-embedded CTAs** — appear in the final assistant prose as inline
   buttons, click-to-open, panel state is owned by the user click, never by the
   agent's tool dispatch.

---

## 4. Proposed architecture — separation of concerns

### 4.1 Kill or repurpose `show_in_panel`

Three options, ranked:

**A. Kill outright.** No more agent-triggered panel pops. The panel is opened
*only* by user click on inline buttons rendered from final-answer markers. The
`show_in_panel` tool is removed from the schema. Simplest. **Recommended.**

**B. Repurpose as "panel preload".** Keep the tool but change semantics: it
silently caches the artifact payload keyed by entity id, and the inline marker
in the answer references that id. Agent can preload deep payloads during
reasoning without flashing the panel. *Tradeoff:* requires a per-turn cache and
adds invariants (marker without preload = late fallback fetch). Worth it only if
the agent actually pre-parses many entities it later cites.

**C. Keep but defer.** Buffer all `show_in_panel` calls; only commit them to
the panel stack at `message_stop` *if* the final assistant text references the
artifact title. Heuristic, fragile — rejected.

The user's framing strongly points at **A**. Even option B is a hedge.

### 4.2 The new mechanism — inline markdown markers in the assistant text stream

The agent embeds a **marker** directly in its prose, parsed by the chat
markdown renderer. Two forms worth considering, examined in §6:

```
Trade your boots for ⟦panel:item:The Stampede⟧.
```

or

```
Trade your boots for [[The Stampede|item]].
```

The renderer turns the marker into a clickable button that calls
`ui.openPanel({...})` with a payload constructed from a **typed sidecar**
emitted by the agent (see §4.4).

### 4.3 Why markers in prose, not a tool call

| Property | Inline marker | `show_in_panel` tool call |
|---|---|---|
| Fires only inside the final answer | ✓ (text stream is the answer) | ✗ (fires anytime) |
| Survives partial token streams | ✓ (defer render until closing `⟧`) | ✓ already today |
| Multiple buttons in one answer | ✓ trivially | clunky (multiple tool calls bloat reasoning) |
| Re-renderable from saved transcripts | ✓ (text is persisted) | requires persisting tool segments + wiring |
| Playable on history reload | ✓ deterministic | needs replay |
| Discoverable from the agent (prompt) | medium — needs SYSTEM_PROMPT update | already there |
| Streaming UX | mark renders when `⟧` arrives — no flash | flash on first arrival |

The killer property is the third row: every cited entity in the answer can have
its own button without spawning a corresponding tool call per entity. A response
listing 4 swap candidates → 4 buttons, 0 tool calls.

### 4.4 The artifact resolution problem

A marker like `⟦panel:item:The Stampede⟧` is a *reference*. Where does the
data come from?

Three layers, in order of preference:

1. **Reasoning-time wiki cache.** The wiki client (`WikiClient::parse`) already
   caches pages 12h on disk. If the agent called `wiki_parse("The Stampede")`
   during reasoning, the parsed page is on disk. The marker resolves by hitting
   the same cache from the IPC backend.
2. **Cold lookup.** If the cache misses, the panel renders a "loading" state and
   the backend fires `wiki_parse` on demand.
3. **Inline structured payload (sidecar).** For high-fidelity cards
   (item-card with `comparison`, gem-detail with `recommended_supports`), the
   agent emits a small JSON block at the end of the answer with the payloads
   keyed by marker id:

   ```
   ⟦panel-data⟧
   { "The Stampede": { "type": "item-card", "payload": {...} },
     "Mageblood":    { "type": "item-card", "payload": {...} } }
   ⟦/panel-data⟧
   ```

   Hidden from the rendered prose. Parsed by the markdown layer pre-render and
   merged into a per-message `panelDataMap`.

Recommendation: **start with layer 3 (sidecar) only.** Agent carries the data;
no async resolve, no cache lookup, no spinner. The wiki cache is a fallback we
add when the agent refuses to inline. See MVP scope §10.

### 4.5 New data flow (MVP)

```
Anthropic stream
  ├─ text_delta:  "...trade them for ⟦panel:item:The Stampede⟧..."
  └─ message_stop fires panel-data sidecar at end of text:
                  "⟦panel-data⟧{...}⟦/panel-data⟧"
                                                          │
                                              v
                            ChatMessage.vue intercepts in renderText:
                              1. extracts ⟦panel-data⟧ block, stores on segment
                              2. replaces ⟦panel:T:N⟧ with <button data-panel="T:N">
                                                          │
                                              v (user clicks)
                            handleClick walks button[data-panel],
                              looks up payload in segment.panelData[N],
                              calls ui.openPanel({...})
```

No backend change beyond removing `show_in_panel`. Streaming-friendly: button
renders only after the closing `⟧` arrives in the token stream. The sidecar
arriving at the end is fine because the marker click is async (user-driven) —
when the click happens the data is there.

### 4.6 Streaming concerns (the partial-token problem)

A token stream might split a marker mid-string. Three strategies:

- **Defer render until closing fence.** The renderer keeps an open marker as
  literal text until `⟧` arrives, then re-renders. Works because `markdown.ts`
  re-runs on every text update (Vue computed). Cheap.
- **Skip-and-leave-as-text on close-of-message.** If we hit `message_stop`
  with an unclosed marker, render it literal (better than crashing). Logging.
- **Sidecar arrives after markers** — fine, button without payload renders as
  disabled until sidecar parses on next text update. Same idiom as today's
  `parsePanelHint` returning null while incomplete.

---

## 5. Artifact taxonomy

The current four (item-card, gem-detail, mechanic, markdown) are good. New ones
worth adding for the answer-CTA model:

| Type | Payload | Primary CTA in panel | Source of payload |
|---|---|---|---|
| `item-card` | `{name, base, rarity, ilvl?, slot?, mods[], comparison?}` | **Open trade query** | wiki_parse + agent reasoning |
| `gem-detail` | `{name, level?, quality?, tags[], scaling[], recommended_supports[]}` | (none today; future: "show in build") | wiki_parse |
| `mechanic` | `{summary, sections[]}` | "Open wiki page" | wiki_parse |
| `passive-cluster` *(new)* | `{notable_name, cluster_size, allocations[]}` | "Allocate in tree viewer" | wiki_parse + cargo |
| `swap-proposal` *(new)* | `{from_item: PobItem, to_item: ItemCardPayload, deltas[]}` | **Open trade query** + "Show comparison" | get_active_build + wiki_parse |
| `trade-query` *(new)* | `{game, league, query_body, label}` | **Open trade URL** | trade_resolve_stats |
| `markdown` | `{body_md}` | (none) | catch-all |

`swap-proposal` deserves a dedicated type — it merges item-card + comparison
*and* exposes the trade button without the agent re-deriving the query at click
time. Sketched in §7.

---

## 6. Marker syntax — three alternatives

### 6.1 Custom Unicode brackets — `⟦panel:type:name⟧`

```
Trade your boots for ⟦panel:item:The Stampede⟧.
```

- **Pros:** unambiguous, no markdown parser conflicts, won't appear in
  organic text, easy to lex.
- **Cons:** unfamiliar to LLMs; may need a few system-prompt examples; some
  tokenizers split the brackets oddly (cost: low).

### 6.2 Markdown-extension link — `[[name|type]]`

```
Trade your boots for [[The Stampede|item]].
```

- **Pros:** wiki-style, idiomatic for LLMs (MediaWiki/Obsidian syntax).
- **Cons:** double-bracket clashes with footnotes/citations in some markdown
  flavors; easier for the agent to leak inside a code block by accident.

### 6.3 Reuse single-backtick + sentinel prefix — `` `panel:item:The Stampede` ``

- **Pros:** zero new tokens; the existing `code_inline` rule already routes
  backtick spans through `markdown.ts`.
- **Cons:** confuses the existing wiki-pill heuristic; values like `panel:foo`
  collide with the entity-tag pattern (`fire:75%`, `note:stale`). Adding
  another pseudo-namespace is fragile.

**Recommendation: 6.1 (`⟦…⟧`).** Cleanest separation from existing markdown
rules. The system prompt teaches the agent the syntax explicitly. Bonus: the
brackets are visually distinct in the system prompt itself, easier to copy.

### 6.4 The sidecar block

```
⟦panel-data⟧
{
  "The Stampede": {
    "type": "item-card",
    "payload": { ... }
  },
  "Mageblood": { ... }
}
⟦/panel-data⟧
```

- Hidden in the rendered prose (CSS `display: none`, or stripped pre-render).
- Parsed by `markdown.ts` before the `code_inline` rule runs.
- Stored on the `TextSegment` (extend `chat.ts` schema with `panelData?:
  Record<string, PanelArtifact>`).
- One sidecar per assistant message, at the very end (after `Sources:`). Or two
  if the message has multiple text segments — handle both.

---

## 7. Trade query integration — the headline use case

### 7.1 What the user wants

> Swap your boots to `The Stampede` → side panel → **"Open trade query"** → opens
> `https://www.pathofexile.com/trade/search/Standard?q=...` (or trade2 for PoE2)
> with `name="The Stampede"` and a sensible mod filter pre-set.

### 7.2 Endpoints (already wired in `trade.rs`)

| Game | API search | Share URL |
|---|---|---|
| PoE1 | `/api/trade/search/{league}` | `/trade/search/{league}/{queryId}` |
| PoE2 | `/api/trade2/search/poe2/{league}` | `/trade2/search/poe2/{league}/{queryId}` |

The `TradeClient::search` POSTs the JSON body, gets a `queryId`, returns
`share_url`. `tauri-plugin-opener` (already used for wiki links) handles
launching the URL in the browser.

### 7.3 Two query-shape strategies

**Unique-item swap (simple).** Body is just `{"query":{"name":"The Stampede"}}`.
Trade site auto-fills the rest. This covers ~80% of swap CTAs (uniques). No
mod resolution needed.

**Rare-item template (advanced).** "Swap your helmet for a `+life +res +intel`
rare in this slot." Requires:
1. `trade_resolve_stats` to map each phrase → stat id.
2. Build `query.stats[].filters` with `{id, value:{min}}`.
3. POST search → get `queryId`.

Cost concern: every rare swap CTA = 1 stat-resolve call + 1 search call. The
agent's per-message budget is finite. Two mitigations:

- **Lazy resolve.** The marker carries only the *intent* (`{slot:"helmet",
  desired:["+80 life","+30 all res","+20 int"]}`); the panel button click
  triggers stat resolution + search server-side.
- **Pre-resolve at marker time.** Agent calls `trade_resolve_stats` and embeds
  full ids in sidecar. More tokens but click is instant.

Recommendation: **lazy resolve for rares, eager URL for uniques.** Uniques
trade-search by `name` only — no resolution needed, the URL builds
deterministically from `name + game + league`. Avoids hitting the GGG API at
all on hover/cold-load.

### 7.4 Sketch — `trade-query` artifact panel button

```ts
// PanelItemCard.vue (extended)
const tradeQuery = computed<TradeQuery | undefined>(() => data.value.trade_query);

async function openTrade() {
  const q = tradeQuery.value;
  if (!q) return;
  // Unique fast path: name-only URL we can build client-side.
  if (q.kind === 'unique-name') {
    const base = q.game === 'poe2'
      ? 'https://www.pathofexile.com/trade2/search/poe2/'
      : 'https://www.pathofexile.com/trade/search/';
    const url = `${base}${encodeURIComponent(q.league)}?q=${
      encodeURIComponent(JSON.stringify({ query: { name: q.name }, status: { option: 'online' } }))
    }`;
    void openLink(url);
    return;
  }
  // Rare lazy path: backend builds the query, posts the search, returns share_url.
  const url = await invoke<string>('trade_swap_query', { game: q.game, league: q.league, intent: q.intent });
  void openLink(url);
}
```

Note the `q=<JSON>` URL form — that's the GGG trade site's query-deep-link
convention; it skips the POST hop entirely for static queries.

### 7.5 League selection

The agent needs the league to build the URL. Sources:
- The build's loaded league (PoB stores it sometimes; not reliable).
- A user setting (`useSettingsStore.league`) the user picks in a dropdown — to
  add. Default to "Standard" if unset.

This deserves its own UI affordance but **can be hard-coded to Standard for the
MVP**, with a follow-up to add a league picker.

---

## 8. Edge cases

### 8.1 Marker references an entity not in the wiki cache

- Sidecar present → render normally, panel opens with sidecar payload.
- Sidecar missing AND cache miss → button renders disabled with tooltip
  *"data not bundled — Bestel didn't include details for this entity"*. Logged.
  Fallback: a one-shot `wiki_parse` on click, with a loading state in the panel.

### 8.2 Multiple buttons reference the same entity in one answer

Today's `openPanel` already dedupes by id (top-of-stack same id → in-place
update, no history growth). So clicking the same name twice is a no-op. Across
distinct names: each click pushes a new artifact, panel grows a back-stack.
That's the intended UX (study A, back, study B).

### 8.3 Streaming — partial token across `⟦panel:`

Renderer must lex raw text per re-render. If an unclosed `⟦panel:` is found:
emit the prefix as literal text, do not render a button. Next tokens close the
marker → re-render, button appears. Vue's reactivity handles this naturally
since `renderText` is a computed over `seg.text`.

### 8.4 Sidecar arrives after the marker

Same idiom: render button, `data-panel` attribute set, but on click the
`panelData` lookup returns undefined → show "not yet ready" toast. In practice
the sidecar is at the end of the message and the user can only click after it
all renders, so this is theoretical. Worth a guard.

### 8.5 Agent emits an invalid sidecar JSON

`tryParsePanelPayload` already swallows JSON errors silently (returns null).
Apply the same pattern to the sidecar parser: bad sidecar → all buttons
disabled, single console warning. Don't break the rest of the message.

### 8.6 Agent re-uses `show_in_panel` against intent

The user explicitly doesn't want this. Two enforcement levels:

- **Soft (prompt).** Update SYSTEM_PROMPT.md to remove `show_in_panel` and
  document the new marker syntax. Old `show_in_panel` calls become a no-op
  (or are dropped at the schema level).
- **Hard (schema).** Remove `show_in_panel` from `tool_schemas()`. The model
  literally cannot call it — no chance of regression.

Recommendation: **hard removal**. Cheap, deterministic.

### 8.7 Marker inside a code block

Code-fenced or backtick-inline content must NOT be parsed as markers — that
would make documentation about the marker syntax recursively render. Fix: run
the marker rule **after** markdown's code lexer, on inline text tokens only.
Same approach as the existing `code_inline` rule (which is already fenced-aware
because `markdown-it` only invokes `code_inline` for `` ` `` spans, not for
fenced code blocks).

### 8.8 Persistence in chat history

The marker is plain text, lives in `seg.text`, persists naturally via the chat
history store. The sidecar lives in the same text — also persisted. On reload
the renderer rebuilds buttons from text. **Zero schema migration.** Big win
over storing tool segments and re-running them.

### 8.9 Internationalisation / proper-noun marker collision

Marker name is the canonical entity name (English, per SYSTEM_PROMPT.md
"Language" rule). The user can be in French; entity names stay English; marker
key matches sidecar key trivially. No collision.

### 8.10 Race: user closes panel while click in flight

`openPanel` is synchronous (Pinia store mutation). No race.

For the rare-trade lazy-resolve path: user clicks button → backend trade-search
is async (~300ms). If they close the panel mid-flight, the resolved URL still
opens in browser via `tauri-plugin-opener`. Acceptable — they asked for it.
Optional: cancel via abort signal if it bothers us.

---

## 9. Three rejected alternatives

### 9.1 Keep tool-driven, add buffering

Buffer all `show_in_panel` calls until the message ends, then promote them all
at once. Solves "panel pops during reasoning" but **does not solve the core
issue**: panels are still tied to whatever the wiki tool happened to fetch, not
to entities Bestel cited in the final prose. Rejected.

### 9.2 Tool-call-as-prose-marker

Have the agent emit a *new* tool call (`embed_panel_button`) inside the final
answer and parse those segments specially. Tradeoffs:

- ✓ structured input schema (no parsing)
- ✗ tool calls can't be embedded in free text — they're separate stream blocks.
  Result: button appears between paragraphs, not inline with words.
- ✗ providers may insert reasoning around tool calls.
- ✗ Anthropic forbids `tool_use` inside text blocks.

Inline-in-text is structurally a markdown problem, not a tool problem. Rejected.

### 9.3 Render every wiki link as a panel button

Drop the marker entirely; turn every existing backtick wiki pill into a panel
button (any backticked entity opens the side panel instead of the wiki).

- ✓ zero agent prompt change.
- ✗ user explicitly wants links AND panel buttons to coexist as two distinct
  affordances. Folding them loses the separation. The user's quote: *"in
  addition to links, also put side-panel buttons"* — so wiki pill (browser) and
  panel button (sidebar) must be visually and behaviourally distinct.
- ✗ today's wiki pill opens the in-app webview overlay; that flow is already
  good. We're adding to it, not replacing.

Rejected.

---

## 10. MVP scope vs nice-to-haves

### 10.1 MVP — minimum to deliver the user's vision

1. Remove `show_in_panel` from `tool_schemas()` and `dispatch()`. Audit
   `useStreaming.ts` for the `tool_end → openPanel` branch and delete it.
2. Define the marker syntax `⟦panel:type:name⟧` and the sidecar
   `⟦panel-data⟧{...}⟦/panel-data⟧`.
3. Extend `markdown.ts`:
   - Pre-pass: extract sidecar block, attach to a render context.
   - Inline rule: replace `⟦panel:type:name⟧` with `<button class="panel-btn"
     data-panel-type="type" data-panel-name="name">name</button>`.
   - Strip the sidecar from the rendered output.
4. Extend `chat.ts` `TextSegment` with `panelData?: Record<string, PanelArtifact>`.
   Populate during stream parse (split sidecar before storing the text).
5. `ChatMessage.vue` `handleClick`: detect `button[data-panel-name]`, look up
   `seg.panelData[name]`, call `ui.openPanel(...)`.
6. Add a CSS pill style for the button — the existing `.turn__panel-hint` is
   already 90% of the way there.
7. Update SYSTEM_PROMPT.md `Right adaptive panel` section: remove
   `show_in_panel` references; document the marker; add 2 examples (one
   item-card with a swap, one mechanic).
8. Add a single new artifact type — `swap-proposal` — to support the headline
   use case. Extend `PanelItemCard` to render the optional `trade_query` action
   when present in the payload.
9. Hard-code league = "Standard" for trade-button URL building. League picker
   is a follow-up.
10. Unique-only trade fast path (`q={name}` URL form). No backend roundtrip.

That's it. Everything else stays.

### 10.2 Phase 2 — nice-to-haves

- Rare-item lazy stat resolution (`trade_swap_query` Tauri command +
  `trade_resolve_stats` chain).
- League picker in settings.
- `passive-cluster` artifact type with a "show in tree viewer" CTA hooked into
  the existing Pixi.js passive-tree modal.
- Wiki-cache resolve fallback (when sidecar payload is missing).
- "Open in PoB" CTA on item-card that exports the item to clipboard in PoB
  format.
- Keyboard shortcut: `j/k` to navigate panel-buttons in the latest message;
  `Enter` to activate.
- Telemetry: count `source: 'click'` panel opens to validate engagement.

### 10.3 Out of scope

- Replacing the in-app webview link viewer (still fine).
- Reworking `PanelMechanic` / `PanelGemDetail` rendering (good enough).
- Anything in the picker brief (separate doc).

---

## 11. Cross-cutting changes summary

| File | Change |
|---|---|
| `crates/bestel-core/src/llm/tools.rs` | Remove `SHOW_IN_PANEL` const, schema entry, dispatch arm |
| `crates/bestel-core/SYSTEM_PROMPT.md` | Replace `Right adaptive panel` section; document marker; add examples |
| `crates/bestel/ui/src/api/markdown.ts` | Add sidecar pre-pass + marker inline rule |
| `crates/bestel/ui/src/stores/chat.ts` | Extend `TextSegment` with optional `panelData` |
| `crates/bestel/ui/src/composables/useStreaming.ts` | Delete the `tool_end → openPanel` branch |
| `crates/bestel/ui/src/components/chat/ChatMessage.vue` | Click handler routes `button[data-panel-name]` to `ui.openPanel` |
| `crates/bestel/ui/src/components/panel/PanelItemCard.vue` | Optional `trade_query` action region with "Open trade" button |
| `crates/bestel/ui/src/stores/ui.ts` | New artifact type `swap-proposal`; payload typing |
| `crates/bestel/ui/src/components/panel/PanelSwapProposal.vue` *(new)* | Renderer for the swap-proposal artifact |
| `crates/bestel/ui/src/stores/settings.ts` | Add `league: string` (default "Standard") |
| `crates/bestel-test/...` | New scenario: assert marker is in final prose, sidecar is parseable, zero `show_in_panel` calls |

The `bestel-test` scenario is critical — it's the regression net that keeps
the agent from re-emitting `show_in_panel` after we kill it (a future model
upgrade may try). Add a battery prompt: "swap my boots, exile" + assert: at
least one `⟦panel:` marker in the final text, valid sidecar JSON.

---

## 12. Open questions for the maintainer

1. **Hard remove `show_in_panel` or repurpose as preload?** Recommendation is
   hard remove. Confirm the team has no parallel use case (MCP exposure?).
2. **Marker syntax — `⟦…⟧` vs `[[…|…]]` vs custom?** I lean `⟦…⟧`. Any
   tokenizer reason to avoid Unicode brackets on smaller models (Ollama)?
3. **Sidecar format — JSON object keyed by name, or array with explicit
   marker-id?** Object is simpler; array allows duplicates with different
   payloads but I don't see when that's useful.
4. **League source.** Hard-code Standard for v1, or block on a settings
   picker? Hard-code is faster; picker is one panel away.
5. **Rare-item trade path** — defer to Phase 2 entirely, or include a
   minimal version (top-3 mods, no tier ranges) in the MVP?
6. **What happens to the existing inline `[highlighted: title]` pill in
   `ChatMessage.vue`?** It's bound to the now-removed `show_in_panel` tool.
   Plan: delete the `tool-panel` branch entirely (no equivalent needed; the
   marker IS the inline pill in the new model).
7. **Should the panel button visually distinguish itself from a wiki link
   pill?** User said "in addition to links". Today wiki pill = pill + chain
   icon. Proposal: panel button = pill + arrow-right icon (`→` or a small
   "open in side" glyph). Final visual decision is the designer's brief, but
   the *signal* should be unambiguous.
8. **Multiple sidecars per message** (assistant streams text → tool → text
   → tool → text). Each `TextSegment` could carry its own sidecar, or we
   merge into a per-message map at `message_stop`. Prefer per-segment
   (simpler, no merge).
9. **Backwards compat with persisted chats** that contain old
   `show_in_panel` tool segments. Today they re-render as the inline pill
   button. After removal: render as a generic ToolCallBadge labeled "panel
   (legacy)", or strip on history load? Pick one and stick.
10. **Does the agent need to know the loaded build's items to generate a
    swap proposal?** Yes — currently `get_active_build` returns full items.
    For the swap CTA, the agent should diff the current slot item with the
    proposed item and embed the comparison in the sidecar payload. Confirm
    the prompt updates make this explicit.

---

## 13. Closing note

The new architecture is small. Three properties matter:

- The agent **never** opens the panel directly. The user does, by clicking a
  button in Bestel's prose.
- Buttons are **inline marks in the assistant text**, parsed by the same
  layer that already turns backticks into wiki pills. One mechanism, two
  affordances (link pill / panel button), visually distinct.
- The data behind a button travels in a **sidecar JSON block** at the end of
  the same message — self-contained, persistable, no async resolve in the MVP.

The "open trade query" CTA inside the item card is the headline payoff and
fits naturally as an action region in `PanelItemCard.vue`. For uniques, no
backend roundtrip; for rares, a Phase-2 lazy resolver. League picker is a
trivial follow-up.
