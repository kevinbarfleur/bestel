# Bestel patches against vendored PoB sources

This file tracks every modification Bestel makes to its vendored copy of Path of Building. **If the vendor tree is re-synced from upstream, every patch listed here MUST be re-applied** or Bestel will regress to the failure modes documented below.

The patches live as inline edits to the vendor files (not as `.patch` files). This was a deliberate trade — `.patch` files are easier to reapply automatically but harder to debug when they fail to apply. Inline edits are obvious in `git diff` after a re-sync.

## Patch 1 — `DropDownControl:GetSelValueByKey` nil-guard (Sprint C2, 2026-05-08)

**Affected files:**
- `vendor/PathOfBuildingCommunity/src/Classes/DropDownControl.lua` (around line 146)
- `vendor/PathOfBuilding-PoE2/src/Classes/DropDownControl.lua` (around line 146)

**Failure observed:** 19 of 29 `pob_calc` invocations in the 2026-05-08 audit failed with:

```
pob engine: sidecar protocol broken: dispatch error: Classes/DropDownControl.lua:147: attempt to index a nil value
```

**Trigger:** `Build.lua` calls `classDrop:SelByValue(curClassId, "classId")` then `ascendDrop.list = classDrop:GetSelValueByKey("ascendancies")`. When `SelByValue` cannot find the value (e.g. classId missing from list after a build reload), `selIndex` stays stale and the next `GetSelValueByKey` call indexes a nil entry. The crash aborts the LuaJIT sidecar mid-recompute, returning a sidecar-protocol error to Rust.

**Patch:**

```lua
function DropDownClass:GetSelValueByKey(key)
    -- Bestel patch (Sprint C2): when SelByValue fails to locate the value
    -- (e.g. classId not in current list after a build reload), self.selIndex
    -- can point at a nil entry. Returning nil here propagates safely; the
    -- raw indexing crashed the entire LuaJIT sidecar in 19/29 calls observed
    -- in the 2026-05-08 audit. Re-apply this guard if vendor is re-synced.
    local sel = self.list and self.list[self.selIndex]
    if not sel then return nil end
    return sel[key]
end
```

**Why not upstream:** The upstream PoB code is fine for interactive UI use (the dropdown is always populated before this method is reached). Headless mode reaches this method through a different path (build-XML reload triggers `recompute_build` which re-syncs class state) where the dropdown can transiently be in an inconsistent state. A clean upstream PR would need to track down every caller and decide what nil-propagation means for their flow — out of scope for Bestel.

## Patch 2 — `safe()` wrapper around `recompute_build` (Sprint C2, 2026-05-08)

**Affected file:** `vendor/api-stdio-bestel/api-stdio.lua` — three call sites in handlers `load_build_xml`, `set_config`, `set_main_selection`.

**Failure mode being prevented:** Even with Patch 1 in place, future regressions inside the recompute pipeline (deep in vendored UI code or PoB calc) would crash the LuaJIT process and force a slot restart. Wrapping the call with the existing `safe()` `pcall` helper converts any future crash into a typed `err_reply("recompute failed: <msg>")` that the Rust side maps to `PobEngineError::BuildLoadFailed` / `ConfigRejected` — propagating cleanly to the agent's tool output where the Sprint A linter rule `POB_CALC_FAILURE_NO_REAL_NUMBER` blocks any "real DPS" claim.

**Patch shape (verbatim from one of the three sites):**

```lua
-- Before
recompute_build()
ok_reply({})

-- After
local _, rerr = safe(recompute_build)
if rerr then return err_reply("recompute failed after <action_name>: " .. rerr) end
ok_reply({})
```

The three sites are tagged with their action name (`load_build_xml`, `set_config`, `set_main_selection`) so a downstream error message tells the model which action triggered the recompute failure.

## Re-sync workflow

When updating the vendored PoB sources from upstream:

1. Replace the vendor tree as usual.
2. Open this file and re-apply each patch by searching for the marker comment (`Bestel patch (Sprint C2)`) — the marker tells you what shape the new code should take.
3. Run `cargo test -p bestel-core --lib response_lint` to confirm the linter still passes.
4. Run `cargo build --release -p bestel -j 1` to confirm the LuaJIT sidecar still builds.
5. Run a smoke `bestel run-battery` against `tests/eval/scenarios/build_real_dps_pinnacle.toml` to confirm `pob_calc` still runs end-to-end against a real build.
