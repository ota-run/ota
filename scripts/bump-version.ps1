#
#                █████
#               ░░███
#       ██████  ███████    ██████
#      ███░░███░░░███░    ░░░░░███
#     ░███ ░███  ░███      ███████
#     ░███ ░███  ░███ ███ ███░░███
#     ░░██████   ░░█████ ░░████████
#      ░░░░░░     ░░░░░   ░░░░░░░░
#
#   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
#
#   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
#
#   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
#   You may not use this file except in compliance with that License.
#   Unless required by applicable law or agreed to in writing, software distributed under the
#   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
#   either express or implied. See the License for the specific language governing permissions
#   and limitations under the License.
#
#   If you need additional information or have any questions, please email: os@ota.run

[CmdletBinding()]
param(
    [Parameter(Position=0)]
    [string]$Version,
    [switch]$Help
)

$ErrorActionPreference = "Stop"

function Show-Usage {
    Write-Host "Usage:"
    Write-Host "  .\scripts\bump-version.ps1 <new-version>"
    Write-Host ""
    Write-Host "Example:"
    Write-Host "  .\scripts\bump-version.ps1 0.2.0"
    Write-Host "  .\scripts\bump-version.ps1 0.2.0-rc.1"
}

if ($Help) {
    Show-Usage
    exit 0
}

if (-not $Version) {
    Show-Usage
    exit 2
}

if ($Version -notmatch '^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$') {
    Write-Error "version must look like semver (for example 0.2.0 or 0.2.0-rc.1)"
    exit 2
}

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$cargoToml = Join-Path $repoRoot "Cargo.toml"
if (-not (Test-Path $cargoToml)) {
    Write-Error "Cargo.toml not found at $cargoToml"
    exit 1
}

$lines = Get-Content -Path $cargoToml
$inPackage = $false
$found = $false
$currentVersion = $null

for ($i = 0; $i -lt $lines.Count; $i++) {
    $line = $lines[$i]
    if ($line -match '^\[package\]$') {
        $inPackage = $true
        continue
    }
    if ($line -match '^\[' -and $line -notmatch '^\[package\]$') {
        $inPackage = $false
    }
    if ($inPackage -and -not $found -and $line -match '^version = "([^"]+)"$') {
        $currentVersion = $Matches[1]
        $lines[$i] = "version = `"$Version`""
        $found = $true
    }
}

if (-not $found -or -not $currentVersion) {
    Write-Error "failed to update [package] version in Cargo.toml"
    exit 1
}

Set-Content -Path $cargoToml -Value $lines

Write-Host "🦦 VERSION BUMP" -ForegroundColor Cyan
Write-Host "Updated: Cargo.toml"
Write-Host "From: $currentVersion"
Write-Host "To:   $Version"
Write-Host ""
Write-Host "Next:"
Write-Host "▸  run `cargo test`"
Write-Host "▸  commit with message like `release: v$Version`"
Write-Host "▸  push to `main`; GitHub Actions will create `v$Version` after the gate passes"
