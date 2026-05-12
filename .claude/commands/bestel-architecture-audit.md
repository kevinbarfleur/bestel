# /bestel-architecture-audit

Audit target: $ARGUMENTS

Process:

1. Read the relevant `docs/architecture/*` files.
2. Inspect code paths that implement the documented behavior.
3. Find drift between docs, prompts, tools, and code.
4. Rank issues by reliability impact.
5. Propose minimal fixes.

Do not rewrite architecture unless asked. Prefer a prioritized punch list.
