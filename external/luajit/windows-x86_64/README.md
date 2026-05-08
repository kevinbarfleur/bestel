# LuaJIT 2.1 — Windows x86_64

Vendored standalone interpreter binary used by `bestel-pob-engine` to run the
forked `api-stdio-bestel/api-stdio.lua` harness against the bundled
PathOfBuilding source tree.

## Why vendored

LuaJIT does not ship official Windows binary releases. We bundle a known-good
build under this directory so end users do not need a Lua toolchain installed.

## Expected layout

```text
external/luajit/windows-x86_64/
├── README.md       (this file)
├── luajit.exe      (interpreter, ≈ 600 KB)
└── lua51.dll       (runtime DLL — required if luajit.exe is dynamic)
```

## How to populate

Run the vendoring script from the repo root:

```powershell
pwsh ./scripts/vendor-luajit.ps1
```

The script attempts (in order):

1. Downloading a pre-built LuaJIT 2.1 from a curated mirror.
2. If no binary is available, building from `git clone` of the official
   LuaJIT repo using MSVC (`msvcbuild.bat`) — requires Visual Studio Build
   Tools (`cl.exe` on PATH) and an x64 Native Tools cmd shell.

The script verifies the resulting `luajit.exe` SHA256 against
`scripts/luajit.sha256.txt`. Update that file when bumping LuaJIT.

## Verification

```powershell
./external/luajit/windows-x86_64/luajit.exe -e "print('LuaJIT', jit.version)"
# Expected: LuaJIT  LuaJIT 2.1.x
```

## License

LuaJIT is MIT-licensed (Mike Pall, 2005-present). License text is reproduced
in the project root `LICENSE-third-party.md`.
