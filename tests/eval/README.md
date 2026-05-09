# Bestel — Sprint A baseline eval set

40 questions across 4 categories (10 each) used to freeze the
**Sprint A baseline** before any prompt / RAG / skills change.
Re-running this set after each subsequent sprint is the only objective
way to see whether a change actually moved quality on Bestel's domain.

## Contents

- `eval_set.toml` — the master file: 40 prompts with per-prompt expected
  and forbidden signals, the build fixture for `needs_identity` cases,
  and category tags (`mechanic`, `craft`, `build`, `mapping`).
- `scenarios/` — 40 generated scenario files compatible with
  `bestel run-battery`. One per entry. **Generated**, not hand-edited;
  re-run the splitter (below) after editing `eval_set.toml`.
- `judge_prompt.md` — the rubric that the LLM-as-judge (Claude Sonnet 4.5,
  per Sprint A decision) is fed alongside the question / expected signals
  / actual answer.
- `baseline-YYYY-MM-DD-<model>.json` — frozen baseline. One file per
  re-run / model. Never overwrite an old baseline; cut a new dated file.

## Regenerating `scenarios/`

```pwsh
# From the repo root.
cargo run --release -p bestel-core --example eval_split -- `
  tests\eval\eval_set.toml tests\eval\scenarios
```

Idempotent — overwrites existing files. The splitter wires the Sprint A
specialised expectations onto each generated scenario:
`must_not_expose_reasoning`, `must_have_panel_data_first`,
`must_cite_only_fetched_urls` are always on; `must_have_identity_card_if_build`
is on for `needs_identity = true` entries; `must_surface_calcs_echo_if_pob_calc`
and `must_not_claim_real_number_if_pob_calc_failed` are on for
`needs_pob_calc = true` entries.

## Running the eval

The eval is intentionally not launched as part of `cargo test` — it
exercises live providers, costs real tokens, and has to be run manually
once API keys are configured. The recommended invocation is:

```pwsh
# From the repo root.
$env:ANTHROPIC_API_KEY = "sk-ant-…"
.\target\release\bestel.exe run-battery tests\eval\scenarios `
  --model claude-haiku-4-5 `
  --out tests\eval\runs\haiku-2026-05-08
```

The battery runner picks up each generated scenario file, drives it
through the live provider, and writes a `PersistedRun` JSON plus a
sidecar `<name>.lint.json` (Sprint A linter findings) to the `--out`
directory. Re-run with `--model claude-sonnet-4-5` and
`--model deepseek-v3-2` to populate the baseline across the three
focus models. Use `--strict` to fail-fast on lint FAIL findings; the
default is warn-only during the A→H transition.

## Judging

Once all three model passes have written runs, score each answer with:

```pwsh
.\target\release\bestel.exe eval-judge `
  --runs-dir tests\eval\runs\haiku-2026-05-08 `
  --eval-set tests\eval\eval_set.toml `
  --judge-model claude-sonnet-4-5 `
  --out tests\eval\baseline-2026-05-08-haiku.json
```

The `eval-judge` subcommand:

1. Loads `eval_set.toml` so it knows the per-prompt expected_signals.
2. For each PersistedRun in the runs directory, posts the run's
   `final_text` together with the prompt and the judge rubric to
   Claude Sonnet 4.5 via the Anthropic API.
3. Parses the strict JSON score returned by the judge:
   `{ "id": "...", "category": "...", "score": 0..100, "rubric": [...], "rationale": "..." }`.
4. Writes one record per scenario to the baseline JSON file.

The baseline is frozen — do not re-grade old runs after the rubric or
the judge model changes. Cut a fresh `baseline-YYYY-MM-DD-<model>.json`
instead, so the diff against prior baselines stays meaningful.
