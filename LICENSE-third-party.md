# Third-party licenses

Bestel ships with several vendored or bundled third-party components.
Each retains its original license; this file aggregates the notices for
distribution compliance. Source code in this repository is MIT-licensed
(see `LICENSE`).

## LuaJIT 2.1 ROLLING (vendored binary)

`external/luajit/windows-x86_64/luajit.exe`
`external/luajit/windows-x86_64/lua51.dll`

Built from upstream `https://github.com/LuaJIT/LuaJIT` (tag
`v2.1.ROLLING`) using MSVC `msvcbuild.bat amalg static`. SHA256 of the
shipped binary is recorded in `scripts/luajit.sha256.txt`.

LuaJIT is © 2005-present Mike Pall, MIT-licensed. Full text:
<https://luajit.org/luajit.html#license>.

### VirusTotal scan

The unsigned `luajit.exe` triggers SmartScreen "unknown publisher" on
first launch and may flag a small number of vendor heuristics on
VirusTotal — typical for unsigned LuaJIT distributions. Run a scan
before shipping a release and record the result here. Reproducing the
binary from source via `scripts/vendor-luajit.ps1` is the canonical
verification path.

## PathOfBuildingCommunity (PoE1 PoB) — submodule

`crates/bestel-pob-engine/vendor/PathOfBuildingCommunity/`

Pinned to release tag `v2.65.0`. © PathOfBuildingCommunity authors,
MIT-licensed. Source:
<https://github.com/PathOfBuildingCommunity/PathOfBuilding>.

## PathOfBuilding-PoE2 (PoE2 PoB) — submodule

`crates/bestel-pob-engine/vendor/PathOfBuilding-PoE2/`

Pinned to release tag `v2.49.3`. © PathOfBuildingCommunity authors,
MIT-licensed. Source:
<https://github.com/PathOfBuildingCommunity/PathOfBuilding-PoE2>.

## api-stdio-bestel — forked Lua harness

`crates/bestel-pob-engine/vendor/api-stdio-bestel/api-stdio.lua`

Conceptually inspired by `ianderse/PathOfBuilding` branch `api-stdio`
(MIT). Rewritten from scratch against the upstream `HeadlessWrapper.lua`
to extend the `set_config` command with the full Calcs key set Bestel
needs. Distributed under the same MIT terms as the upstream PoB.

## dkjson (used by harness)

The harness loads `dkjson.lua` from PoB's
`runtime/lua/dkjson.lua`. © David Heiko Kolf, MIT-licensed.

## sha1 (used by harness via PoB)

PoB's `runtime/lua/sha1/` is a pure-Lua SHA1. © its authors,
MIT-licensed.
