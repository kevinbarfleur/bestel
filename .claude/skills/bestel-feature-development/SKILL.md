---
name: bestel-feature-development
description: Workflow for implementing a Bestel feature or bug fix across Rust, Tauri, Vue, prompts, tools, or evals. Use when the user asks to add, fix, refactor, or improve application behavior and wants production-quality code with minimal complexity.
---

# Bestel feature development workflow

## Steps

1. Classify the subsystem: backend, frontend, LLM/tooling, prompt/RAG, PoB, Build Sheet, eval, or release.
2. Read the relevant architecture doc and exact files.
3. Write a short implementation plan with the smallest viable patch.
4. Implement one coherent change at a time.
5. Update tests/evals/docs when behavior changes.
6. Run the narrowest relevant validation.
7. Report changed files, validation, and residual risks.

## Constraints

- Do not refactor unrelated code.
- Do not add abstractions for a single use case.
- Do not solve deterministic runtime problems with prompt text alone.
- Keep user-facing behavior consistent unless explicitly changing it.

## Done output

Use this structure:

```markdown
## Changed
- ...

## Validation
- `command`: result

## Risks
- ...

## Follow-up
- ...
```
