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
    Write-Host "  .\scripts\bump-version.ps1 <new-version|patch|minor|major>"
    Write-Host ""
    Write-Host "Example:"
    Write-Host "  .\scripts\bump-version.ps1 patch"
    Write-Host "  .\scripts\bump-version.ps1 minor"
    Write-Host "  .\scripts\bump-version.ps1 major"
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

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$cargoToml = Join-Path $repoRoot "Cargo.toml"
$changelog = Join-Path $repoRoot "CHANGELOG.md"
$readinessWorkflow = Join-Path $repoRoot ".github/workflows/ota-readiness.yml"
if (-not (Test-Path $cargoToml)) {
    Write-Error "Cargo.toml not found at $cargoToml"
    exit 1
}
if (-not (Test-Path $changelog)) {
    Write-Error "CHANGELOG.md not found at $changelog"
    exit 1
}
if (-not (Test-Path $readinessWorkflow)) {
    Write-Error "readiness workflow not found at $readinessWorkflow"
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
        $found = $true
    }
}

if (-not $found -or -not $currentVersion) {
    Write-Error "failed to locate [package] version in Cargo.toml"
    exit 1
}

function Resolve-TargetVersion {
    param(
        [string]$CurrentVersion,
        [string]$RequestedVersion
    )

    switch ($RequestedVersion) {
        "patch" {
            $core = ($CurrentVersion -split '[+-]')[0]
            $parts = $core -split '\.'
            return "{0}.{1}.{2}" -f [int]$parts[0], [int]$parts[1], ([int]$parts[2] + 1)
        }
        "minor" {
            $core = ($CurrentVersion -split '[+-]')[0]
            $parts = $core -split '\.'
            return "{0}.{1}.0" -f [int]$parts[0], ([int]$parts[1] + 1)
        }
        "major" {
            $core = ($CurrentVersion -split '[+-]')[0]
            $parts = $core -split '\.'
            return "{0}.0.0" -f ([int]$parts[0] + 1)
        }
        default {
            if ($RequestedVersion -notmatch '^[0-9]+\.[0-9]+\.[0-9]+([-.][0-9A-Za-z.-]+)?$') {
                Write-Error "version must be patch, minor, major, or look like semver (for example 0.2.0 or 0.2.0-rc.1)"
                exit 2
            }
            return $RequestedVersion
        }
    }
}

$TargetVersion = Resolve-TargetVersion -CurrentVersion $currentVersion -RequestedVersion $Version
$lines = Get-Content -Path $cargoToml
$inPackage = $false
$found = $false
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
        $lines[$i] = "version = `"$TargetVersion`""
        $found = $true
    }
}

$changelogLines = [System.Collections.Generic.List[string]]::new()
$changelogLines.AddRange([string[]](Get-Content -Path $changelog))

if ($changelogLines.Contains("## $TargetVersion")) {
    Write-Error "CHANGELOG.md already contains ## $TargetVersion"
    exit 1
}

$unreleasedIndex = $changelogLines.IndexOf("## Unreleased")
if ($unreleasedIndex -lt 0) {
    Write-Error "failed to locate ## Unreleased in CHANGELOG.md"
    exit 1
}

Set-Content -Path $cargoToml -Value $lines

$workflowLines = Get-Content -Path $readinessWorkflow
$updatedWorkflow = $false
for ($i = 0; $i -lt $workflowLines.Count; $i++) {
    if (-not $updatedWorkflow -and $workflowLines[$i] -match '^(\s*ota-version:\s*)(\S+)\s*$') {
        $workflowLines[$i] = "$($Matches[1])$TargetVersion"
        $updatedWorkflow = $true
    }
}

if (-not $updatedWorkflow) {
    Write-Error "failed to update ota-version in $readinessWorkflow"
    exit 1
}

Set-Content -Path $readinessWorkflow -Value $workflowLines

$changelogLines.Insert($unreleasedIndex + 1, "")
$changelogLines.Insert($unreleasedIndex + 2, "## $TargetVersion")

Set-Content -Path $changelog -Value $changelogLines

Write-Host "🦦 VERSION BUMP" -ForegroundColor Cyan
Write-Host "Updated: Cargo.toml, CHANGELOG.md, .github/workflows/ota-readiness.yml"
Write-Host "From: $currentVersion"
Write-Host "To:   $TargetVersion"
Write-Host ""
Write-Host "Next:"
Write-Host '  » run `ota run ci` to execute the canonical local verification task'
Write-Host ('  » commit with message like `release: v{0}`' -f $TargetVersion)
Write-Host ('  » push to `main`; GitHub Actions will create `v{0}` after the gate passes' -f $TargetVersion)
