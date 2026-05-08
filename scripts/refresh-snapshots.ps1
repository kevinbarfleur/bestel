# Thin wrapper around the Rust example that does the actual fetch.
# Run from anywhere; resolves the workspace root via cargo.

param(
    [switch]$Release = $true
)

$ErrorActionPreference = 'Stop'

Push-Location (Split-Path -Parent $PSScriptRoot)
try {
    if ($Release) {
        cargo run --release -p bestel-core --example refresh_snapshots
    } else {
        cargo run -p bestel-core --example refresh_snapshots
    }
} finally {
    Pop-Location
}
