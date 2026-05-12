---
name: bestel-prompt-edit-workflow
description: Workflow for safe edits to Bestel prompts, bundled references, CORE_KNOWLEDGE, source policy, answer contracts, and in-app skills. Use whenever modifying `prompts/` or prompt-related Rust bundling.
---

# Prompt edit workflow

## Steps

1. Identify the observed behavior or desired contract.
2. Search for existing instructions that already cover it.
3. Remove contradictions before adding text.
4. Keep system prompt instructions short and operational.
5. Move long methodology to references or skills.
6. Update `crates/bestel-core/src/prompts.rs` if bundled references change.
7. Add or update eval/lint coverage.

## Safety checks

Search for:

- nonexistent tool names
- obsolete tool names
- hardcoded current values
- hidden reasoning tags
- source-policy contradictions
- Build Sheet workflow contradictions

## Final review

A prompt change is not complete unless the behavior can be tested or audited.
