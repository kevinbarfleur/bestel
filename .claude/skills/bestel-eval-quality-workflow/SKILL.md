---
name: bestel-eval-quality-workflow
description: Workflow for Bestel evals, response lint, run-battery scenarios, LLM judge baselines, and quality regression analysis. Use when adding scenarios, changing agent behavior, or comparing models.
---

# Eval and quality workflow

## Steps

1. Define the failure or quality target.
2. Decide deterministic lint, scenario, judge rubric, or all three.
3. Add a minimal scenario with expected and forbidden signals.
4. Use fixtures for build-dependent cases.
5. Run a narrow battery when possible.
6. Preserve dated baselines.

## Scenario design

Good scenarios include:

- realistic user wording
- one clear expected behavior
- one or more forbidden failure modes
- game applicability for PoE1/PoE2 separation
- fixture name when build state matters

## Result analysis

Report:

- pass/fail by category
- source/citation failures
- tool order failures
- model-specific failures
- regressions compared to baseline
