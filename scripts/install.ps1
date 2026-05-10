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
    [switch]$FromRelease
)

$bootstrapUrl = if ($env:OTA_BOOTSTRAP_URL) { $env:OTA_BOOTSTRAP_URL } else { "https://dist.ota.run/bootstrap.ps1" }
$bootstrapPath = $null
$localBootstrapPath = $null
$downloadBootstrap = $false

if (-not [string]::IsNullOrWhiteSpace($PSScriptRoot)) {
    $localBootstrapPath = Join-Path $PSScriptRoot "bootstrap.ps1"
}

if ($localBootstrapPath -and (Test-Path -LiteralPath $localBootstrapPath -PathType Leaf)) {
  $bootstrapPath = $localBootstrapPath
  Write-Output "Info: using local bootstrap from ${bootstrapPath}."
} else {
  $downloadBootstrap = $true
  $bootstrapPath = Join-Path ([System.IO.Path]::GetTempPath()) ("ota-bootstrap-" + [Guid]::NewGuid().ToString("N") + ".ps1")
  Write-Output "Info: downloading bootstrap from ${bootstrapUrl}."
  irm $bootstrapUrl -OutFile $bootstrapPath
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

try {
    powershell -ExecutionPolicy Bypass -File $bootstrapPath @bootstrapArgs
}
finally {
    if ($downloadBootstrap -and (Test-Path -LiteralPath $bootstrapPath -PathType Leaf)) {
        Remove-Item -LiteralPath $bootstrapPath -Force
    }
}
