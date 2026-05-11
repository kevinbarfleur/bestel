# Legacy snapshot — pre prompt-slim sprint (2026-05-11)

Snapshot of the always-loaded prompts before the `prompt-slim` sprint compression. Kept here for diff / rollback / forensics if a regression is found in production.

## Files

- `SYSTEM_PROMPT.md` — copy of `prompts/SYSTEM_PROMPT.md` at commit `cbea239` (HEAD of `main` when the sprint started). 219 lines, ~3 376 words.
- `CORE_KNOWLEDGE.md` — copy of `prompts/CORE_KNOWLEDGE.md` at the same commit. 145 lines, ~1 563 words.

Total always-loaded baseline : ~4 939 words.

## Context

The `prompt-slim` sprint compresses these two files by ~36 % (~1 775 words gained) while preserving every hard rule and invariant. See:

- Plan : `C:\Users\kbarf\.claude\plans\wondrous-puzzling-forest.md`
- Preservation audit : `tools/scenarios/runs/prompt-slim-preservation-audit.md`
- Quality baseline : `tools/scenarios/runs/quality-batch-3-models-20260510T154836Z/`
- Quality report : `tools/scenarios/runs/QUALITY-REPORT-2026-05-10.md`

## Rollback

If a regression is found after merge, revert by overwriting:

```
cp docs/prompt-slim/legacy-2026-05-11/SYSTEM_PROMPT.md prompts/SYSTEM_PROMPT.md
cp docs/prompt-slim/legacy-2026-05-11/CORE_KNOWLEDGE.md prompts/CORE_KNOWLEDGE.md
```

Or via git:

```
git checkout cbea239 -- prompts/SYSTEM_PROMPT.md prompts/CORE_KNOWLEDGE.md
```

The runtime tag injection in `crates/bestel-core/src/llm/anthropic.rs:166-428` is untouched by the sprint, so the only state to roll back is the two markdown files themselves.
