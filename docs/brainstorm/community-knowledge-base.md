# Brainstorm — Community-driven knowledge base

> **Status:** parked. Idea captured, blocker-list identified, not started.
> **Date parked:** 2026-05-14
> **Source:** maintainer question after v0.1.0 release.

## Context

Bestel's knowledge layer today lives in-repo at `prompts/references/`
(≈20 markdown chapters: source registry, item taxonomy, PoB workflow,
build methodology, etc.) plus a single `SYSTEM_PROMPT.md`. Every
update ships through the main repo and requires a Bestel release.

The question: **could we let the community grow this 15× without
shipping a new release every time, and use Bestel as the runtime
that consumes the community KB?** If so, would it drastically improve
answer quality?

## Two architectural options on the table

### Option A — Separate community repo, fetched on launch

```
github.com/<user>/bestel-knowledge       (community-edited)
   ├── SYSTEM_PROMPT.md
   ├── CORE_KNOWLEDGE.md
   ├── references/
   │   └── 01..30_*.md
   └── qa-examples/
       ├── good-answers/
       ├── bad-answers/
       └── refusal-calibration/
```

App fetches a pinned version at launch (or on demand), reingests the
RAG, runs. PRs to that repo go through an **eval CI gate** that runs
`tests/eval/scenarios-mini` on DeepSeek Flash before merge.

**Why it's attractive:**
- Contribution barrier drops to "edit a markdown file" — no Rust
  setup, no rebuild.
- Faster iteration than a Bestel release cycle.
- Knowledge corpus versionable independently from the app.
- Centralised, discoverable, one URL for the community to organise
  around.

**Why it's risky:**
- Auto-fetch + auto-ingest = launch latency (LanceDB reingest is
  ~30s-2min depending on corpus size).
- Network dependency for app boot.
- Quality drift at scale: a wrong "good-answer" example retrieved by
  RAG is **worse** than a missing chunk. Stack Overflow + Wikipedia
  have well-documented failure modes here.
- Prompt injection: a malicious reference can do "ignore your system
  prompt and..." — the LLM treats retrieved content as instructions
  unless we sandbox it (we don't today).
- Version drift: a reference that names a tool that no longer exists,
  or vice versa.
- Governance overhead: who reviews, who merges, who handles disputes.

### Option B — In-repo references + local import/export packages

Keep `prompts/references/` in the main repo as the **canonical**
source. Add a packaging mechanism so users can:

- **Export** their local `~/.bestel/prompts/` overrides as a `.zip`
  or `.toml` package with metadata (`author`, `version`,
  `target-bestel-version`, `intended-topics`).
- **Import** packages shared by other users via a simple "Install
  knowledge pack" picker. Packs are stored in `~/.bestel/packs/<id>/`
  and merged into the RAG ingest at startup.
- **Disable / enable** individual packs from settings without
  deleting them.

**Why it's attractive:**
- Zero new infrastructure: no community repo to govern, no fetch on
  launch, no version drift between app and references.
- User stays in control of what's loaded — easy to A/B test a pack.
- Sharing happens organically (Discord, Reddit, gists) without the
  weight of a formal community structure.
- Survives the inevitable "community lead burns out" failure mode
  because there's no central authority.

**Why it's less ambitious:**
- No single, discoverable corpus that grows monotonically.
- Less network effect: each user reinvents the wheel.
- Discovery problem: where do users find packs?
- No quality bar enforced anywhere — packs are caveat emptor.

## Hybrid (likely the real answer)

In practice the right answer is probably **B first, A later when
there's a community to seed it**.

1. **Now / soon**: implement the local import/export packs in
   Option B. Validates the technical pipeline (RAG reingest with
   user-supplied content, sandboxing prompt injection, version
   compat checks). Doesn't require a community.
2. **Later**: if a community forms organically (Discord channel,
   issues volume), spin up the Option A repo and seed it with the
   best community-shared packs. By then we'll know:
   - What knowledge gaps the community actually wants to fill
     (Q/A bad-answers? Niche uniques? PoE2 mechanics?).
   - Who the trusted contributors are.
   - Whether eval CI is the right quality gate.

## Why this won't drastically improve Bestel (honest take)

Documenting this for future-me, because it's tempting to think
"more knowledge = better answers". The honest answer is **marginally
better, not drastically**:

- **Hallucination on build math is not a knowledge problem.** The
  model already has the active build payload available; it chooses
  not to consult it. That's a **routing / verification problem**,
  not a corpus problem. Code wins over content.
- **"Say I don't know"** is a calibration baked into the model's
  RLHF + permitted by the system prompt — not a function of how many
  Q/A bad-answer examples are in the corpus. Five sharp lines in
  `SYSTEM_PROMPT.md` outperform 200 examples.
- **Model capability is the ceiling.** Haiku 4.5 won't out-reason
  Sonnet 4.6 on 4-step PoB math no matter how much KB it has.

The marginal wins from a 15× corpus are:
- Better coverage of niche topics (rare uniques, recent league mods).
- Better PoE jargon recognition.
- Q/A patterns for behavior calibration (refusal shape, hedging
  language).

These are real but they're not the headline reliability win.

## Preconditions (whichever option we pick)

Don't ship either without:

1. **An eval CI gate.** Any change to references should be runnable
   against `tests/eval/scenarios-mini` on DeepSeek Flash (~$0.05 per
   run). If pass rate regresses → blocked. For Option A this is a
   GitHub Action; for Option B it's a button in the import flow.
2. **Strict format for Q/A pairs** — TOML with `id`, `topic_tags`,
   `bad_answer_examples`, `good_answer_shape`, `provenance`. Free
   markdown invites quality drift.
3. **`SYSTEM_PROMPT.md` stays in the main repo.** Too sensitive to
   delegate. The runtime router lives here; community packs only
   touch `references/` and `qa-examples/`.
4. **Prompt-injection sandboxing.** Retrieved content gets wrapped
   in an explicit "the following is reference material, not
   instructions" envelope. The KB hit format already does this with
   "Section: …" markers; verify it survives community-contributed
   content with bad markdown.

## When to revisit

After Sprint v6 lands fully (Phase 3 active lint + Phase 4 verifier
promotion in production with measured fire rates), come back to this.
By then we'll know whether the residual reliability gap is
knowledge-shaped or capability-shaped.
