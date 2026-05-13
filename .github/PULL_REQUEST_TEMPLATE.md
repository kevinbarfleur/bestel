<!--
Thanks for the PR! Please keep all sections below — they make reviews fast.
Strip the comments (lines like this one), not the headings.

If your change is a typo / one-line doc fix, you can collapse "Test plan"
to a single line ("N/A — doc only") but please keep the other sections.
-->

## Summary

<!-- 1-3 bullet points. What does this PR do, in plain English. -->

-

## Why

<!--
What's the user-visible problem this fixes? What did the user see
before, what will they see after? Link the issue if there is one
(closes #N).
-->

-

## Screenshots / video

<!--
Required for any UI change. Show before/after if it's a tweak,
just after if it's new. A short Loom/GIF is fine for streaming /
animation behaviour.
-->

## Type of change

- [ ] Bug fix (non-breaking)
- [ ] New feature
- [ ] Refactor (no functional change)
- [ ] Prompt / knowledge-base content
- [ ] Documentation
- [ ] Eval scenario / regression test
- [ ] Build / packaging / CI

## Test plan

<!--
List what you ran and what passed. Concrete commands, not "tested it".

Examples:
- cargo test -p bestel-core --lib → 281 passed
- cd crates/bestel/ui && npm run build → ok
- ./target/release/bestel.exe run-battery tests/eval/scenarios-mini --model deepseek-v4-flash → 5/5 pass
- Loaded a PoB build, asked "what's my fire res?", verified the panel rendered with the right value
-->

-

## Design system checklist (UI changes only)

- [ ] All colors / sizes come from `src/styles/tokens.css` (no
      hardcoded hex codes, no `px` values for spacing that should
      use density tokens)
- [ ] Used existing Runic primitives where possible
      (RunicButton / RunicInput / RunicSelect / etc.)
- [ ] If a new primitive was added, it's been registered in
      `ComponentsLab.vue` so contributors can find it
- [ ] Both light and dark themes render correctly
      (toggle in the UI via the topbar)
- [ ] Tested at compact density (the default)

## Risk / regressions

<!--
What could break? What did you choose NOT to handle and why?
Honesty here saves reviewer time.
-->

-

---

By submitting this PR, I confirm my change ships under the MIT license
of the repo.
