# Bestel agentic development configuration

This pack is designed for the Bestel repository. It provides a pragmatic agentic setup for fast, high-quality development without adding unnecessary ceremony.

Install by copying the files into the repository root:

```bash
cp -r .claude CLAUDE.md AGENTS.md docs scripts /path/to/bestel/
```

Recommended usage:

1. Keep `CLAUDE.md` at the repo root as the default project memory for coding agents.
2. Use `.claude/agents/*.md` as specialist subagents when working in Claude Code or compatible agentic tools.
3. Use `.claude/skills/*/SKILL.md` as reusable playbooks for repeated workflows.
4. Use `.claude/commands/*.md` as slash-command prompts or copy/paste workflows.
5. Run `node scripts/agentic/lint-agentic-config.mjs` after editing the config.

Design principles:

- Prefer one primary coding agent plus narrowly scoped specialists.
- Keep orchestration deterministic whenever possible.
- Favor small, reversible patches.
- Require evidence from code, tests, architecture docs, or recorded runs.
- Avoid speculative abstractions and framework churn.
- Escalate to a stronger model only for ambiguity, cross-cutting architecture, hard debugging, or quality review.
