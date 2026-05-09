# PoE2 0.5 "Return of the Ancients" — launch-day playbook

Patch ships **2026-05-29**. This playbook is the ordered checklist Bestel
has to run through within ~72h of launch so the assistant stays
non-fabricating and PoE2-current. Each step lists the files / commands
that change.

## 0. Pre-launch (T-7 days, idle now)

- Skim GGG announcement materials (forum thread + livestream notes).
  Capture every concrete number / new mechanic name into a scratch file
  (do NOT commit them to `prompts/references/poe2/` yet — those need
  patch-confirmed sources).
- Pre-fetch `pathofexile.com/forum/view-thread/<patch-id>` URL pattern
  the moment the patch thread goes up; it's the canonical source for
  patch notes and is wiki-trusted.

## 1. Version pin (T+0, first 2h)

Edit `prompts/references/poe2/00_version_pinning.md`:

```diff
- > **As of 2026-05-08 (Bestel roadmap kickoff): PoE2 0.4 is current. PoE2 0.5 "Return of the Ancients" releases 2026-05-29.**
+ > **As of 2026-05-29 (PoE2 0.5 launch): PoE2 0.5 "Return of the Ancients" is current. Previous patch was 0.4.**
```

Bestel reads this on every PoE2-related answer; nothing else fires
correctly until this line is right.

## 2. RAG re-index with `applies_to=poe2` honoured (T+0, automatic on next bestel boot)

The Sprint H ingest config (`bestel-rag::IngestConfig::default()`)
already routes every `*.md` under `prompts/references/poe2/**` to
`applies_to = ["poe2"]`. Newly-edited files re-ingest by content hash on
the next bestel cold start; nothing manual required as long as the
edits land under `prompts/references/poe2/`.

To force a fresh ingest:

```powershell
Remove-Item -Recurse -Force "$env:USERPROFILE\.bestel\index\kb.lance"
```

then relaunch bestel — the next boot rebuilds the index from disk.
Verify with:

```powershell
.\target\release\bestel.exe db query --sql "SELECT doc_path, indexed_at FROM kb_versions WHERE doc_path LIKE 'poe2/%' ORDER BY indexed_at DESC"
```

## 3. Snapshot refreshes (T+0, ~1h hands-off)

Two GGG endpoints need re-fetching for the assistant's lookup tools to
return correct names + IDs:

```powershell
# Trade stats (item indexer + crafting helpers)
Invoke-WebRequest https://www.pathofexile.com/api/trade2/data/stats -OutFile snapshots/trade2-stats-0.5.json

# Static asset catalogue
Invoke-WebRequest https://www.pathofexile.com/api/trade2/data/static -OutFile snapshots/trade2-static-0.5.json
```

Then re-pull repoe-fork's PoE2 snapshot (force, even if no commits):

```bash
cd third_party/repoe-fork
git pull --ff-only
git submodule update --init
```

The bestel-core build script (`crates/bestel-core/build.rs`) compresses
the relevant files at build time; rebuilding `bestel` after the pull
embeds the new snapshot. **No manual conversion** — the build script
does it.

## 4. PoE2 references — fill the empty stubs (T+0 to T+72h)

Two files are intentionally empty placeholders; they have to be filled
from the patch notes + first-week wiki updates:

- `prompts/references/poe2/05_atlas_mechanics_05.md` — Atlas tree
  rework, Verisium / Runic Ward / Alloys mechanics, ocean tile
  expansion, ~400 nodes.
- `prompts/references/poe2/06_runes_of_aldur.md` — league mechanic
  body (currently empty by design).

Citation discipline: pin every claim to `pathofexile.com/forum` (patch
notes) for the first 48h; only switch to wiki citations once
PoEDB / poe2db catches up (~48-72h after launch).

## 5. Patch history meta (T+0, 30min)

Edit `prompts/references/24_patch_history_meta.md`. Append a 0.5 entry
with the conceptual delta: what's new, what's replaced, what's
deprecated. This file feeds the **answer_mode_router → Patch-current**
path, so getting it wrong leaves Bestel quoting a 0.4 mechanic on a 0.5
question.

## 6. PoB2 engine compatibility (T+0, 1h)

The bundled PoB2 submodule (`third_party/pathofbuilding-2`) usually
pins to whatever was current at the time of the last bestel release.
Verify:

```powershell
cd third_party\pathofbuilding-2
git fetch
git log HEAD..upstream/dev --oneline
```

If upstream PoB2 has 0.5-related schema changes, bump the submodule
pin. Then run the headless smoke:

```powershell
cargo run --release -p bestel-pob-engine --example pob_smoke -- tests/fixtures/pob/poe2_druid.xml
```

If the smoke fails to parse the new XML schema, the symptom is
`engine.failed=true` on every PoE2 build — surfaces clearly via the
Sprint G verifier (`cache_disclaimer_missing` finding) and via the
Sprint A linter (`POB_CALC_FAILURE_NO_REAL_NUMBER`). Fix path:
1. Update XML decoder in `crates/bestel-pob-engine/src/xml/...`.
2. Add a 0.5 fixture under `tests/fixtures/pob/poe2_druid_0_5.xml`.
3. Re-run `pob_smoke` until clean.

## 7. Eval re-run (T+72h, requires API tokens)

The Sprint H 60-scenario eval set already contains 11 PoE2-tagged
entries (`applies_to = ["poe2"]`). Once steps 1-6 are done, the
authoritative validation is:

```powershell
.\target\release\bestel.exe run-battery tests/eval/scenarios `
    --out tests/eval/runs/poe2-0.5-launch `
    --strict
.\target\release\bestel.exe eval-judge `
    --runs-dir tests/eval/runs/poe2-0.5-launch `
    --eval-set tests/eval/eval_set.toml `
    --out tests/eval/runs/poe2-0.5-launch/baseline.json
```

Exit criteria (ROADMAP § Sprint H):
- ≥ 90% pass rate on Sonnet 4.5
- ≥ 75% on Haiku 4.5
- ≥ 65% on DeepSeek V3.x
- **0 PoE1↔PoE2 contamination** on the 11 PoE2-tagged scenarios.

Inspect `lint_findings` for `cross_game_contamination` rows:

```powershell
.\target\release\bestel.exe db query --sql "SELECT r.id, r.model_id, l.message FROM runs r JOIN lint_findings l ON l.run_id = r.id WHERE l.rule = 'cross_game_contamination' ORDER BY r.started_at DESC LIMIT 20"
```

## 8. Conditional rerank decision (T+72h, after eval)

Per ROADMAP § Sprint H Task 4: if the eval plateaus below 90% top-K
precision and the failures cluster on retrieval (BM25 vs vector
disagreement), integrate `bge-reranker-v2-m3` via `ort` (ONNX Runtime)
or Candle. Otherwise skip — RRF k=60 is the default and rarely needs
tuning at this corpus size (~70 docs).

## What survives this playbook (single source of truth)

- Version pin file is THE answer to "which PoE2 are we talking about".
- `kb_versions` table is the record of what was indexed when, by which
  embedder. `bestel db stats` exposes the count.
- Sprint H 11 PoE2-tagged scenarios are the contamination probe; rerun
  any time someone suspects PoE1 leakage.
- Patch history meta is the rolling delta — never delete previous
  entries; the model needs to reason about "this changed from X.Y to
  X.Z" across patches.
