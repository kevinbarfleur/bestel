# Real-user prompt fixtures

Authentic-voice prompts mined from public PoE communities, used by every Bestel test battery to evaluate model behaviour on the kinds of questions actual exiles ask. Single source of truth — any new battery runner reads from here.

## Where this lives

Path: `docs/test_prompts/real_user_prompts.toml`

This is **plain TOML**, human-readable and machine-parseable. Loaders in
`crates/bestel-core/src/llm/` and example battery runners (`crates/bestel-core/examples/*_battery.rs`) deserialize it via `serde` + `toml`.

Don't move it. Anything that depends on it does so by relative path from the repo root.

## Methodology

Each prompt was lifted (verbatim or lightly normalised for length) from one
of these sources:

- `reddit.com/r/pathofexile`
- `reddit.com/r/PathOfExileBuilds`
- `reddit.com/r/PathOfExile2`
- `reddit.com/r/PathOfExileSSF`
- `pathofexile.com/forum` (Help & Information, Bug Reports, General)
- Maxroll guide comments
- Mobalytics / GamesRadar / GameFAQs PoE2 boards

Prompts retain real user voice: lowercase starts, missing punctuation,
typos, slang (`tbh`, `idk`, `tripping`, `gutted`), abbreviations (`t16`,
`RT`, `BV`, `PF`, `RF`), bare URLs pasted mid-sentence, and frustrated
tones. **Do not "clean up" prose** — that's the entire point.

## Schema

```toml
[[prompt]]
id          = "kebab_case_unique_slug"
category    = "vague" | "beginner" | "mechanics" | "diagnosis"
            | "build_choice" | "external_link" | "crafting"
            | "currency_chase" | "boss" | "poe2"
            | "comparison_lore" | "off_topic_refusal"
text        = "the literal prompt sent to the model"
intent      = "what the user actually wants (1 line)"
expected    = "what good model behaviour looks like (1-3 lines)"
needs_build = false  # true → battery runner attaches a PoB before sending
source      = "https://reddit.com/..."  # optional
```

## Categories — what each tests

| Category | What it pressures | Failure modes to watch for |
|---|---|---|
| `vague` | Clarification reflex | Model dumps generic advice instead of asking 1-2 focused questions |
| `beginner` | Patient onboarding | Talking down, jargon dump, "go read the wiki" |
| `mechanics` | Tool usage on small queries | Pavé-of-text instead of 2-line answer + wiki cite |
| `diagnosis` | `get_active_build` reflex + chiffrage | Forgetting to call the tool, or vague "more life lol" |
| `build_choice` | Comparative reasoning under uncertainty | Picking one without trade-offs, or straight refusal |
| `external_link` | `web_fetch` against allowlist | Hallucinating maxroll content instead of fetching |
| `crafting` | Step-by-step explanation | Skipping fundamentals, using deprecated currency names |
| `currency_chase` | Cost-benefit framing | Hard "yes" / "no" without context (build, league, budget) |
| `boss` | Concrete mechanic + dodge tip | Generic "git gud", or wiki dump without practical edge |
| `poe2` | Verification reflex (training cutoff) | Memory-quoting outdated PoE2 facts without `wiki_parse` |
| `comparison_lore` | Honest "I'm not sure" + check | Confidently wrong on cross-game differences |
| `off_topic_refusal` | In-character refusal | Recommending other ARPGs, breaking persona |

## Adding a prompt

1. Find a real source. Don't invent voice — mine it.
2. Pick the smallest applicable category. New categories require updating this README + every loader.
3. Use a `kebab_case` id that hints at the test (`vague_squishy_build`, `mechanics_rt_ailments`).
4. Fill `intent` and `expected` honestly. If you can't articulate what good looks like, the prompt isn't useful.
5. Don't normalise grammar. Don't add quotation marks the user wouldn't have typed. Don't capitalise "what".

## Loader contract

Loaders should:
- Accept an optional `category` filter and an optional id-list filter for quick smoke runs
- Surface `needs_build = true` so the runner attaches the active PoB before send
- Pass `text` verbatim — no template substitution
