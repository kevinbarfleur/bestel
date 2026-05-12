# Bestel agentic development system

This directory explains how to use the repo-level agentic configuration.

## Layers

1. `CLAUDE.md` - global project memory and default rules.
2. `.claude/agents/` - narrow specialist subagents.
3. `.claude/skills/` - reusable workflows/playbooks.
4. `.claude/commands/` - shortcut prompts for common operations.
5. `scripts/agentic/lint-agentic-config.mjs` - drift checks for this configuration.

## Principle

Use one main coding agent plus a specialist only when the task needs bounded expertise. Do not create a permanent committee of agents for every change.

## Recommended default routing

- Small localized Rust fix: `rust-tauri-architect` if backend behavior is non-trivial.
- Small localized Vue fix: `vue-ui-engineer`.
- Prompt/tool/model behavior: `llm-agent-engineer` plus `prompt-reliability-engineer` review.
- Build Sheet user flow: `product-ux-flow-designer` plus `llm-agent-engineer` if tool flow changes.
- PoB engine/parser: `pob-engine-specialist`.
- RAG/reference retrieval: `rag-knowledge-engineer`.
- Eval/lint/baseline: `qa-eval-engineer`.
- Agent config itself: `agentic-config-architect`.
- Installer/resource paths: `release-build-engineer`.

## Escalation rule

Escalate to a stronger model only after one of these happens:

- architecture crosses more than two subsystems
- cheap-model attempt fails a hard gate
- source evidence conflicts
- bug cannot be localized after a focused trace
- prompt/tool change affects production answer reliability
