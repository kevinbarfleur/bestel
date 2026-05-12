# Model routing for Bestel development agents

Use capability only where it pays for itself.

## Cheap/fast model tasks

- localized code edits
- simple TypeScript fixes
- mechanical Rust refactors
- docs cleanup
- test scenario drafting
- summarizing recorded runs
- applying known patterns from a skill

## Strong model tasks

- cross-cutting architecture
- LLM provider/tool loop changes
- prompt/system contract rewrites
- unexplained eval regressions
- PoB engine debugging with ambiguous failures
- security-sensitive source fetching changes
- final review before large merge

## Best default pattern

1. Cheap model implements or drafts.
2. Deterministic checks run.
3. Cheap model repairs minor failures.
4. Strong model reviews only if checks fail or the change is high risk.

## Do not

- use a strong model to compensate for missing tests
- add a new specialist agent for a one-off task
- solve deterministic runtime behavior with longer prompt instructions
