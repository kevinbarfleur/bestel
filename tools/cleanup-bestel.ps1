# Properly tears down all Bestel processes — including the WebView2
# child processes that `Stop-Process -Force` on bestel.exe leaves
# orphaned on Windows.
#
# Tauri 2 + WebView2 on Windows spawn 4-8 child processes per window
# (renderer, GPU, network, utility, plugin-host). Windows has no native
# process group concept, so killing only the parent leaves the children
# as orphans tied to the WebView2 Runtime — they survive until reboot
# unless explicitly killed.
#
# Strategy: walk the process tree from any running bestel.exe via CIM
# `ParentProcessId`, kill the descendants first, then the parents. The
# script is safe to call repeatedly — it's a no-op when nothing is
# running.
#
# Usage:
#   pwsh tools\cleanup-bestel.ps1                  # standard cleanup
#   pwsh tools\cleanup-bestel.ps1 -All             # also kill ALL msedgewebview2
#                                                    (use only if other Edge-based
#                                                    apps are already closed)
#   pwsh tools\cleanup-bestel.ps1 -Verbose         # show what's being killed

[CmdletBinding()]
param(
    [switch]$All
)

$ErrorActionPreference = 'Continue'

function Get-Descendants {
    param([int[]]$ParentIds)
    $allProcs = Get-CimInstance Win32_Process -ErrorAction SilentlyContinue
    $found = @{}
    $queue = New-Object System.Collections.Queue
    foreach ($id in $ParentIds) { $queue.Enqueue($id); $found[$id] = $true }
    while ($queue.Count -gt 0) {
        $current = $queue.Dequeue()
        $children = $allProcs | Where-Object { $_.ParentProcessId -eq $current }
        foreach ($c in $children) {
            if (-not $found.ContainsKey($c.ProcessId)) {
                $found[$c.ProcessId] = $true
                $queue.Enqueue($c.ProcessId)
            }
        }
    }
    $found.Keys
}

# 1. Find all bestel.exe instances.
$bestels = Get-Process -Name bestel -ErrorAction SilentlyContinue
$bestelIds = @($bestels | ForEach-Object { $_.Id })

if ($bestelIds.Count -gt 0) {
    Write-Host "==> Found $($bestelIds.Count) bestel.exe instance(s): $($bestelIds -join ', ')" -ForegroundColor Cyan
    # 2. Walk the descendant tree to enumerate WebView2 children.
    $descendants = Get-Descendants -ParentIds $bestelIds
    $childrenOnly = @($descendants | Where-Object { $bestelIds -notcontains $_ })
    if ($childrenOnly.Count -gt 0) {
        Write-Host "==> Found $($childrenOnly.Count) descendant process(es) — killing children first" -ForegroundColor Cyan
        foreach ($id in $childrenOnly) {
            $p = Get-Process -Id $id -ErrorAction SilentlyContinue
            if ($p) {
                Write-Verbose "    kill $id ($($p.ProcessName))"
                Stop-Process -Id $id -Force -ErrorAction SilentlyContinue
            }
        }
    }
    # 3. Now kill the bestel.exe parents.
    foreach ($id in $bestelIds) {
        Write-Verbose "    kill $id (bestel.exe)"
        Stop-Process -Id $id -Force -ErrorAction SilentlyContinue
    }
    Start-Sleep -Milliseconds 500
} else {
    Write-Host "==> No bestel.exe running" -ForegroundColor DarkGray
}

# 4. Optional sweep: kill any remaining msedgewebview2 (use with caution).
if ($All) {
    $remaining = Get-Process -Name msedgewebview2 -ErrorAction SilentlyContinue
    if ($remaining) {
        Write-Host "==> -All requested: killing $($remaining.Count) remaining msedgewebview2 process(es)" -ForegroundColor Yellow
        Write-Host "    (this affects ALL Edge-based apps, not just Bestel)" -ForegroundColor Yellow
        $remaining | Stop-Process -Force -ErrorAction SilentlyContinue
    }
}

# 5. Final state report.
$leftover = Get-Process -Name bestel -ErrorAction SilentlyContinue
$wvStill = Get-Process -Name msedgewebview2 -ErrorAction SilentlyContinue
Write-Host ""
Write-Host "==> Done. bestel: $($leftover.Count) running. msedgewebview2 (system-wide): $($wvStill.Count) running." -ForegroundColor Green
