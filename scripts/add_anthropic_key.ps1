$ErrorActionPreference = 'Stop'
$path = Join-Path $env:USERPROFILE '.bestel\runtime\keys.json'
$dir = Split-Path -Parent $path
if (-not (Test-Path $dir)) { New-Item -ItemType Directory -Path $dir -Force | Out-Null }

$keyToAdd = $args[0]
if ([string]::IsNullOrWhiteSpace($keyToAdd)) { throw 'Pass the key as the first argument.' }

if (Test-Path $path) {
    $obj = Get-Content -Path $path -Raw | ConvertFrom-Json
} else {
    $obj = New-Object -TypeName PSObject
}

if ($obj.PSObject.Properties.Match('ANTHROPIC_API_KEY').Count -gt 0) {
    $obj.ANTHROPIC_API_KEY = $keyToAdd
} else {
    $obj | Add-Member -NotePropertyName 'ANTHROPIC_API_KEY' -NotePropertyValue $keyToAdd
}

$json = $obj | ConvertTo-Json -Compress:$false
Set-Content -Path $path -Value $json -NoNewline

# Print a redacted view so we can confirm the file shape without echoing keys.
$redacted = $json -replace 'sk-[A-Za-z0-9_-]{20,}','sk-***REDACTED***'
Write-Output $redacted
