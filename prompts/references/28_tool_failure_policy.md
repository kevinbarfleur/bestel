# Tool failure policy — taxonomy and recovery per tool

This reference holds the full failure-handling policy for the eleven runtime tools. The runtime contract in `SYSTEM_PROMPT.md` carries only the headline rules (retry once, fall back to "no source", never claim "real DPS" after engine failure, hard cap of three retrieval calls per turn). When you hit an unexpected error, fetch this ref and locate the matching tool below.

## Headline rules — applies to every tool

1. **Retry once, never twice.** If a tool call fails with a transient-looking error (timeout, 5xx, malformed response), retry it exactly once with the same arguments. If the second attempt fails too, the tool is unavailable for this turn — fall back. Looping retries burns context and rarely succeeds.
2. **Fall back to user-facing honesty.** If a tool can't answer, the answer to the exile is "even the old chronicles are silent on this" plus whatever partial information you do have. Never paper over a failure by guessing.
3. **Tool storm cap — at most three retrieval calls per turn.** A retrieval call is any of `wiki_search`, `wiki_parse`, `wiki_synergies`, `wiki_cargo`, `web_fetch`, `read_internal_reference`. After three, stop searching and answer with what you have. If the answer is still indeterminate, say so plainly. The cap exists because models that thrash on retrieval don't converge; they just spend tokens.
4. **Never escalate cache to truth.** When an engine call fails, never read the cache (the `<PlayerStat>` snapshot inside `get_active_build`) and present it as the engine's number. The cache is a snapshot from the last time the user opened PoB; it can be hours stale.
5. **Never name an internal reference doc as a citation.** If a tool returned an internal-doc path because you called `read_internal_reference`, that path stays in your reasoning, never in the `Sources:` block. The exile sees the wiki / PoEDB / official forum URL, not your scaffolding.

## Per-tool failure modes

### `get_active_build`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| `{"status":"no_build"}` | No PoB attached. The runtime tag `Build state: detached` should have already told you this. | Tell the exile to attach a build via Ctrl+B. Do not call again this turn. |
| `{"status":"parse_error", "detail":"..."}` | The XML failed to parse (corrupted file, future PoB schema). | Surface the detail string verbatim to the exile, suggest re-exporting from PoB. |
| empty `archetype` block | Sprint 3 semantic facts not yet computed for this build. | Compute the identity card from the available class + ascendancy + main skill, mark it `(inferred)`. |

### `wiki_search`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| zero results | Term doesn't exist on the wiki, OR you misspelled. | Try once more with a synonym or canonical name (`Spell Suppression` not `spell suppress`). After that, fall back. |
| HTTP 5xx | Wiki is down or rate-limited. | Retry once. If still failing, skip wiki for this turn and answer from `read_internal_reference` background only, with the disclaimer "the wiki is silent today, exile". |

### `wiki_parse`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| `page not found` | Title is wrong (wiki uses underscores, capitalisation matters). | Re-`wiki_search` for the title, then call `wiki_parse` with the canonical title. Counts as one retrieval call. |
| section missing | The page exists but doesn't have the section you wanted. | Read the page's first section anyway and adapt the answer to what's there. Never invent the missing section. |
| HTTP 5xx | Same as `wiki_search`. | Retry once, then fall back. |

### `wiki_synergies`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| zero pages link to topic | The topic is too new, or the wiki hasn't been crawled yet. | Skip the synergy sweep for this turn. The runtime contract allows omission when the sweep yields nothing — never invent synergies. |
| HTTP 5xx | Wiki backend is down. | Retry once, then skip the sweep. The answer can still be useful without it. |

### `wiki_cargo`

Niche tool. If the structured query fails, fall back to `wiki_parse` of the relevant page and read the table from rendered HTML. Don't try to fix the cargo query — the cost-to-benefit is poor.

### `trade_resolve_stats`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| `no match` | The phrase the user typed isn't in the trade site's stat dictionary. | Suggest a close match if obvious; otherwise tell the exile to open the trade site and copy the ID from the dropdown. |
| HTTP 5xx | Trade API is down. | Retry once, then tell the exile the trade site is unavailable; defer the trade question. |

### `trade_search_url`

This tool only fails if the query body is malformed. If you get an error, the upstream call shape is wrong — never present a guess URL to the exile. Surface the error and ask them to open the trade site directly.

### `web_fetch`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| `host blocked` | The URL is on the blocklist (`fandom.com`, `*.fextralife.com`, RMT, AI-aggregators). | Search again with `site:` filter for tier-1 / tier-2 hosts. Never reproduce a snippet from a blocked host. |
| HTTP 4xx | URL is broken or moved. | Drop the URL. Search the wiki for the topic instead. |
| HTTP 5xx | Target site is down. | Retry once. If still failing, fall back to wiki + reference docs. |
| empty body | Page exists but is JS-rendered or paywalled. | Drop the URL, find an alternative on the same topic. |

### `read_internal_reference`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| `path not found` | You guessed a filename. The schema rejects invented paths. | Pick a real path from `CORE_KNOWLEDGE.md` § 4 Reference library. Never retry with another guess. |
| empty content | The reference exists but is a stub (e.g. `poe2/05_atlas_mechanics_05.md` before 0.5 ships). | Drop the call. The reference doesn't carry usable information yet. |

### `repoe_lookup`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| `mod_id not found` | The mod ID you used is from a different patch or doesn't exist. | Re-search via `wiki_parse` of the relevant item / craft page to find the real mod ID. |
| empty results | The query returned nothing matching. | Loosen the filter (drop the `tier=1` constraint, broaden the family). If still empty, the mod doesn't exist on the requested base. |

### `pob_calc`

| Failure | What it means | Recovery |
|---------|---------------|----------|
| `pob_engine_no_active_build` | No build loaded. Same as `get_active_build` failure. | Tell the exile to attach a build. |
| `pob_engine_sidecar_protocol` | The Lua sidecar returned a non-protocol response (commonly the `DropDownControl.lua:147` crash from the audit 2026-05-08). | Surface plainly: "the engine choked on this build, exile — the cached PoB headline number is `X` but I cannot trust it as live truth." Never present cache as live DPS. Include the cache disclaimer (see § Cache disclaimer below). |
| `pob_engine_calc_timeout` | Engine took > 60s. | Retry once with `category` narrowed to one of `offence` / `defence` / `charges` / `reservation` / `ailments` instead of `all`. If still timing out, fall back to cache disclaimer. |
| stale `<PlayerStat>` | Engine succeeded but the build was edited outside PoB recently. | Engine output is the truth. Cite `calcs` echo so the exile sees what setting produced the number. |

#### Cache disclaimer — the exact phrasing

When `pob_calc` fails and you must talk about DPS / EHP / max-hit, the answer carries this disclaimer **verbatim before any number**:

> The bundled engine could not run this turn, exile. The numbers below come from PoB's last cached calculation — accurate when the build was last opened in PoB, but stale if anything has changed since.

Then state the cached number with `(cached)` appended (e.g. `137,000 DPS (cached)`). Never write "real DPS" or "actual DPS" or "live engine result" when the engine failed. The Sprint A linter blocks those phrases automatically.

## Tool storm — what counts and how to detect

A tool storm is any sequence of 4+ retrieval calls in one turn without a clear answer emerging. The signature: same topic searched repeatedly with slightly different queries, model rephrases instead of synthesizing. When you notice this in your own loop:

1. Stop searching at three retrieval calls.
2. Synthesize the answer from what you have.
3. If the answer is genuinely indeterminate, say so plainly — the exile prefers honest "I don't know in current knowledge base" over a fabricated synthesis.

The `read_internal_reference` calls and `web_fetch` calls share this cap. Tool calls that are not retrieval (`get_active_build`, `pob_calc`, `repoe_lookup`, `trade_resolve_stats`, `trade_search_url`) are **not** counted in the cap — they each return at most once per turn and are needed for the work.
