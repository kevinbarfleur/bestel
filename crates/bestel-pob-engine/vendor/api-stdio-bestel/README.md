# api-stdio-bestel

ndJSON-over-stdio harness for the Bestel headless PoB engine.

## Provenance

Conceptually inspired by `ianderse/PathOfBuilding` (branch `api-stdio`).
Rewritten from scratch against the upstream `HeadlessWrapper.lua` so we can
extend `set_config` to expose the full Calcs key set the agent needs:

- `enemyIsBoss` (bool → `"Pinnacle"` / `"None"`, or pass-through string)
- `usePowerCharges`, `useFrenzyCharges`, `useEnduranceCharges`
- `forceBuffOnslaught`
- `multiplierImpaleStacks`
- `useFlask1` … `useFlask5`
- `bandit`, `pantheonMajorGod`, `pantheonMinorGod`, `enemyLevel`

The protocol shape (single-line JSON in / single-line JSON out, `action` field,
`ok`/`error` reply envelope) deliberately matches the upstream contract.

## Invocation

```text
luajit api-stdio.lua <pob_root_dir> <game>
```

- `<pob_root_dir>` — absolute path to `PathOfBuilding{,-PoE2}/src/`
- `<game>` — `poe1` or `poe2` (currently informational)

The Rust supervisor sets `Command::current_dir(<pob_root_dir>)` before spawn so
`HeadlessWrapper.lua`'s `dofile("Launch.lua")` resolves correctly.

## Re-validation checklist

Whenever the PoB submodule is bumped (PoE1 or PoE2):

1. Confirm `src/HeadlessWrapper.lua` still defines `loadBuildFromXML` +
   `runCallback("OnFrame")` and exposes `build = mainObject.main.modes["BUILD"]`.
2. Confirm `src/Modules/ConfigOptions.lua` still uses the same `var` names
   for `enemyIsBoss`, `usePowerCharges`, etc.
3. Re-run `bestel-pob-engine` integration tests.

If any of those break, patch `api-stdio.lua` (look for the `CONFIG_KEYS`
table and the `set_one_config` dispatch).
