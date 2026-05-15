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
    [switch]$FromSource,
    [switch]$FromGit,
    [switch]$FromRelease,
    [switch]$SetupPath
)

$ErrorActionPreference = "Stop"

$bootstrapUrl = if ($env:OTA_BOOTSTRAP_URL) { $env:OTA_BOOTSTRAP_URL } else { "https://dist.ota.run/bootstrap.ps1" }
$bootstrapPath = $null
$localBootstrapPath = $null
$downloadBootstrap = $false
$tempBootstrapDir = $null

function Test-OtaCheckoutScriptRoot {
    if ([string]::IsNullOrWhiteSpace($PSScriptRoot)) {
        return $false
    }

    if ((Split-Path -Leaf $PSScriptRoot) -ne "scripts") {
        return $false
    }

    $repoRoot = Split-Path -Parent $PSScriptRoot
    $manifest = Join-Path $repoRoot "Cargo.toml"
    if (-not (Test-Path -LiteralPath $manifest -PathType Leaf)) {
        return $false
    }

    return [bool](Select-String -Path $manifest -Pattern '^name = "ota"$' -Quiet)
}

if (-not [string]::IsNullOrWhiteSpace($PSScriptRoot)) {
    $localBootstrapPath = Join-Path $PSScriptRoot "bootstrap.ps1"
}

if ($FromSource.IsPresent -and (Test-OtaCheckoutScriptRoot) -and $localBootstrapPath -and (Test-Path -LiteralPath $localBootstrapPath -PathType Leaf)) {
  $bootstrapPath = $localBootstrapPath
  Write-Output "Info: using local bootstrap from ${bootstrapPath}."
} else {
  $downloadBootstrap = $true
  $tempBootstrapDir = Join-Path ([System.IO.Path]::GetTempPath()) ("ota-install-" + [Guid]::NewGuid().ToString("N"))
  New-Item -ItemType Directory -Force -Path $tempBootstrapDir | Out-Null
  $bootstrapPath = Join-Path $tempBootstrapDir "bootstrap.ps1"
  Write-Output "Info: downloading bootstrap from ${bootstrapUrl}."
  Invoke-WebRequest -Uri $bootstrapUrl -OutFile $bootstrapPath -ErrorAction Stop | Out-Null
}

$bootstrapArgs = @()
if ($FromSource.IsPresent)
{
    $bootstrapArgs += "-FromSource"
}
elseif ($FromGit.IsPresent)
{
    $bootstrapArgs += "-FromGit"
}
elseif ($FromRelease.IsPresent)
{
    $bootstrapArgs += "-FromRelease"
}
if ($SetupPath.IsPresent)
{
    $bootstrapArgs += "-SetupPath"
}

try {
    $powershellExe = if (Get-Command pwsh -ErrorAction SilentlyContinue) { "pwsh" } else { "powershell" }
    & $powershellExe -NoLogo -NoProfile -ExecutionPolicy Bypass -File $bootstrapPath @bootstrapArgs
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}
finally {
    if ($downloadBootstrap -and $tempBootstrapDir -and (Test-Path -LiteralPath $tempBootstrapDir)) {
        Remove-Item -LiteralPath $tempBootstrapDir -Recurse -Force
    }
}
