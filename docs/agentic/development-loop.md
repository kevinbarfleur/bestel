# Bestel development loop

## 1. Triage

- What subsystem owns the behavior?
- Is this a bug, feature, reliability issue, UX issue, or documentation drift?
- Which specialist should own it?

## 2. Inspect

Read architecture first, code second. Do not edit from memory.

## 3. Plan

Write the smallest plan that can solve the issue. Name expected files and checks.

## 4. Patch

Make one coherent change. Keep unrelated cleanup out of the patch.

## 5. Validate

Run the narrowest command that proves the change, then broader checks if risk warrants it.

## 6. Report

Summarize:

- what changed
- why it changed
- validation run
- risks and follow-up

## 7. Capture recurring patterns

If the same workflow repeats twice, add or update a skill. If the same failure repeats twice, add a lint or eval.
