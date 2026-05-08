---
description: Current PoE2 version + "facts as of 0.X" annotations. Updated every patch. The single source of truth for what version of PoE2 the rest of the poe2/ directory describes.
fetch_when: Always check this first when reasoning about a PoE2-specific mechanic. The version stamp here gates every other claim in poe2/.
---

# PoE2 — version pinning

## Current version

> **As of 2026-05-08 (Bestel roadmap kickoff): PoE2 0.4 is current. PoE2 0.5 "Return of the Ancients" releases 2026-05-29.**

## Versioning convention used in poe2/ docs

When writing a PoE2 fact, tag it with the version range:

- `[0.1+]` — true since the indicated version (still current).
- `[0.2 → 0.4]` — true between these versions (no longer current if absent from current).
- `[0.5+]` — known to ship with the indicated upcoming version (placeholder until the patch lands).

## Patch cadence

PoE2 patches roughly every 4 months. Major content patches: 0.1 (early access launch), 0.2, 0.3, 0.4, 0.5 (next).

## What to do when a patch ships

1. Update this file's "Current version" line.
2. Re-run `repoe_lookup` snapshot refresh (Sprint 1 task once shipped).
3. Re-fetch `/api/trade2/data/stats` and refresh trade catalogue.
4. Audit each `poe2/<topic>.md` for stale `[X.Y → ...]` claims.
5. Update `references/24_patch_history_meta.md` with the conceptual delta.

> Status: stub. Updated by hand at every PoE2 patch boundary.
