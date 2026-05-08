#!/usr/bin/env pwsh
# vendor-luajit.ps1 — populate external/luajit/windows-x86_64/ with a
# known-good LuaJIT 2.1 standalone interpreter.
#
# Tries (in order):
#   1) Prebuilt mirror download (fast path)
#   2) Source build via MSVC (requires VS Build Tools + x64 Native Tools cmd)
#
# Idempotent: re-runs are no-ops if the existing luajit.exe matches the
# pinned SHA256.

$ErrorActionPreference = "Stop"
Set-StrictMode -Version Latest

$RepoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$VendorDir = Join-Path $RepoRoot "external\luajit\windows-x86_64"
$LuaJITExe = Join-Path $VendorDir "luajit.exe"
$Lua51Dll  = Join-Path $VendorDir "lua51.dll"
$SHAFile   = Join-Path $RepoRoot "scripts\luajit.sha256.txt"

New-Item -ItemType Directory -Path $VendorDir -Force | Out-Null

function Test-PinnedHash([string]$exe) {
    if (-not (Test-Path $exe)) { return $false }
    if (-not (Test-Path $SHAFile)) { return $false }
    $expected = (Get-Content $SHAFile -Raw).Trim().Split()[0].ToLower()
    if ([string]::IsNullOrEmpty($expected)) { return $false }
    $actual = (Get-FileHash -Algorithm SHA256 $exe).Hash.ToLower()
    return $actual -eq $expected
}

if (Test-PinnedHash $LuaJITExe) {
    Write-Host "luajit.exe already matches pinned SHA256; nothing to do."
    & $LuaJITExe -e "print('LuaJIT ready:', jit.version)"
    exit 0
}

Write-Host "Vendoring LuaJIT 2.1 to $VendorDir ..."

# Strategy 2: source build via MSVC. Strategy 1 (prebuilt mirror) is left
# unimplemented because there is no canonical Windows binary distribution
# for LuaJIT 2.1 — building from upstream is the supported path.

$BuildDir = Join-Path $env:TEMP "bestel-luajit-build"
if (Test-Path $BuildDir) { Remove-Item -Recurse -Force $BuildDir }
git clone --depth 1 --branch v2.1.0 https://github.com/LuaJIT/LuaJIT $BuildDir
if ($LASTEXITCODE -ne 0) {
    throw "git clone of LuaJIT failed. Ensure git is on PATH."
}

$srcDir = Join-Path $BuildDir "src"
$msvcbuild = Join-Path $srcDir "msvcbuild.bat"
if (-not (Test-Path $msvcbuild)) {
    throw "msvcbuild.bat not found at $msvcbuild — repo layout changed?"
}

# Run msvcbuild from inside src/. Requires cl.exe on PATH (x64 Native Tools).
Push-Location $srcDir
try {
    & cmd /c "msvcbuild.bat amalg static"
    if ($LASTEXITCODE -ne 0) {
        throw @"
LuaJIT msvcbuild.bat failed. You probably need to run this script from a
'x64 Native Tools Command Prompt for VS 2022' (or PowerShell launched from
one). Visual Studio Build Tools must be installed.
"@
    }
} finally {
    Pop-Location
}

Copy-Item (Join-Path $srcDir "luajit.exe") $LuaJITExe -Force
$dllSrc = Join-Path $srcDir "lua51.dll"
if (Test-Path $dllSrc) { Copy-Item $dllSrc $Lua51Dll -Force }

# Smoke test.
& $LuaJITExe -e "print('LuaJIT ready:', jit.version)"
if ($LASTEXITCODE -ne 0) { throw "luajit.exe smoke test failed." }

# Record SHA256 for future runs.
$hash = (Get-FileHash -Algorithm SHA256 $LuaJITExe).Hash.ToLower()
"$hash  luajit.exe" | Set-Content -Path $SHAFile -NoNewline
Write-Host "Vendored. SHA256 recorded at $SHAFile"
