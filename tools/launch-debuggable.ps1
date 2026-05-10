# Launches Bestel with the WebView2 CDP listener exposed and the Rust
# tracing pipeline writing to ~/.bestel/runtime/logs/bestel-{date}.log.
# Required for the `bestel-driver` debug harness to attach.
#
# Usage:
#   pwsh tools\launch-debuggable.ps1                  # release exe, port 9222
#   pwsh tools\launch-debuggable.ps1 -Port 9333
#   pwsh tools\launch-debuggable.ps1 -Dev             # cargo tauri dev instead
#
# DO NOT distribute the resulting build with these env vars set in the
# user environment. CDP exposes the chat store's full state to anyone who
# can reach localhost on the chosen port.

[CmdletBinding()]
param(
    [int]$Port = 9222,
    [switch]$Dev,
    [string]$ExePath
)

$ErrorActionPreference = 'Stop'

$repoRoot = Resolve-Path "$PSScriptRoot\.."

$env:BESTEL_DEBUG_PORT = $Port
$env:BESTEL_FILE_LOG = "1"
# Default tracing filter — verbose enough for debug, not noisy.
if (-not $env:RUST_LOG) {
    $env:RUST_LOG = "info,bestel=info,bestel_core=info,bestel.verifier=info,bestel.debug=info"
}

Write-Host "==> BESTEL_DEBUG_PORT = $Port" -ForegroundColor Cyan
Write-Host "==> BESTEL_FILE_LOG = 1 (logs at ~/.bestel/runtime/logs/)" -ForegroundColor Cyan
Write-Host "==> RUST_LOG = $env:RUST_LOG" -ForegroundColor Cyan
Write-Host ""

if ($Dev) {
    Write-Host "==> Running cargo tauri dev (Vite hot reload)" -ForegroundColor Green
    Push-Location "$repoRoot\crates\bestel"
    try {
        cargo tauri dev
    } finally {
        Pop-Location
    }
    return
}

if (-not $ExePath) {
    $ExePath = "$repoRoot\target\release\bestel.exe"
}
if (-not (Test-Path $ExePath)) {
    Write-Error "Bestel release binary not found at $ExePath. Build it first with:`n  (cd crates/bestel/ui && npm run build) && cargo build --release -p bestel"
    return
}

Write-Host "==> Launching $ExePath" -ForegroundColor Green
& $ExePath
