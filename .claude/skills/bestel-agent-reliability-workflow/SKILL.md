---
name: bestel-agent-reliability-workflow
description: Workflow for LLM agent reliability improvements in Bestel: tool loop, orchestration, model routing, verifier, source grounding, hard gates, and small-model behavior. Use when changing agent behavior or reducing hallucinations.
---

# Agent reliability workflow

## Reliability hierarchy

1. Code-level deterministic orchestration.
2. Typed tool outputs and evidence packets.
3. Hard gates and lints.
4. Eval scenarios and recorded-run replay.
5. Prompt instructions.
6. Stronger model fallback.

Do not invert this order without a clear reason.

## Steps

1. Name the failure class.
2. Find where the failure currently enters: router, tool selection, tool output, synthesis, verifier, UI, or recorder.
3. Add code enforcement if possible.
4. Add a lint or eval scenario.
5. Only then edit prompt text.
6. Test with at least one cheap-model path when possible.

## Required checks for factual outputs

- Same-turn fetched URL for final citations.
- Tool/source/build/sheet origin for numerical claims.
- PoB active skill verification before DPS/EHP claims.
- Explicit staleness label for cached engine values.
- PoE1/PoE2 separation.
