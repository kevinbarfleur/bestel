---
name: bestel-pob-engine-workflow
description: Workflow for PoB parsing, build semantics, LuaJIT engine sidecar, `pob_calc`, active skill verification, Calcs config, and build hash/fingerprint behavior. Use for any Path of Building integration work.
---

# PoB engine workflow

## Steps

1. Classify the layer: parser, semantic identity, watcher, engine protocol, tool output, or sheet fingerprint.
2. Use fixtures to reproduce behavior.
3. Keep raw item/gem text lossless.
4. Keep engine output tied to effective Calcs config.
5. Verify active skill metadata before trusting numbers.
6. Surface warnings and cache fallback explicitly.

## Tests

Prefer fixture-based tests for parser and semantic changes.

For engine changes, test:

- load XML
- set config
- select main skill
- get stats
- crash/timeout behavior

## High-risk changes

- canonical hash or fingerprint changes
- vendor harness changes
- sidecar lifecycle changes
- silent fallback behavior
