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
    [switch]$FromSource
)

$ErrorActionPreference = "Stop"

if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Error "cargo is required to install ota"
    exit 1
}

$installFromSource = $FromSource.IsPresent
if ((Test-Path ".\Cargo.toml") -and (Select-String -Path ".\Cargo.toml" -Pattern '^name = "ota"$' -Quiet)) {
    $installFromSource = $true
}

if ($installFromSource) {
    Write-Host "installing ota from local source (cargo install --path .)..."
    & cargo install --path . --locked --force
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} else {
    $gitUrl = if ($env:OTA_GIT_URL) { $env:OTA_GIT_URL } else { "https://github.com/ota-run/ota.git" }
    $tag = $env:OTA_GIT_TAG
    $branch = $env:OTA_GIT_BRANCH
    $rev = $env:OTA_GIT_REV

    $refsSet = 0
    if ($tag) { $refsSet++ }
    if ($branch) { $refsSet++ }
    if ($rev) { $refsSet++ }
    if ($refsSet -gt 1) {
        Write-Error "set only one of OTA_GIT_TAG, OTA_GIT_BRANCH, OTA_GIT_REV"
        exit 1
    }

    Write-Host "installing ota from $gitUrl..."
    if ($tag) {
        & cargo install --git $gitUrl --tag $tag ota --locked --force
    } elseif ($branch) {
        & cargo install --git $gitUrl --branch $branch ota --locked --force
    } elseif ($rev) {
        & cargo install --git $gitUrl --rev $rev ota --locked --force
    } else {
        & cargo install --git $gitUrl ota --locked --force
    }
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

if (Get-Command ota -ErrorAction SilentlyContinue) {
    & ota --version
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} else {
    Write-Error "install completed but 'ota' is not on PATH yet"
    exit 1
}
