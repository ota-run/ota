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
    [Parameter(Mandatory = $true)]
    [ValidateSet("doctor", "workspace-doctor", "receipt-diff")]
    [string]$Mode,

    [Parameter(Mandatory = $true)]
    [ValidateSet("plain", "github", "markdown")]
    [string]$Format,

    [Alias("Input")]
    [Parameter(Mandatory = $true)]
    [string]$JsonPath,

    [string]$Title,

    [string]$OtaBin = $env:OTA_BIN
)

$ErrorActionPreference = "Stop"

$repoRoot = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path

if ([string]::IsNullOrWhiteSpace($OtaBin)) {
    foreach ($candidate in @(
        (Join-Path $repoRoot "target/debug/ota.exe"),
        (Join-Path $repoRoot "target/debug/ota"),
        (Join-Path $repoRoot "target/release/ota.exe"),
        (Join-Path $repoRoot "target/release/ota")
    )) {
        if (Test-Path $candidate) {
            $OtaBin = $candidate
            break
        }
    }
}

$cargoBin = $null
if ([string]::IsNullOrWhiteSpace($OtaBin) -and (Test-Path (Join-Path $repoRoot "Cargo.toml"))) {
    $cargo = Get-Command cargo -ErrorAction SilentlyContinue
    if ($cargo) {
        $cargoBin = $cargo.Source
    }
}

if ([string]::IsNullOrWhiteSpace($OtaBin) -and -not $cargoBin) {
    $ota = Get-Command ota -ErrorAction SilentlyContinue
    if ($ota) {
        $OtaBin = $ota.Source
    }
}

if ([string]::IsNullOrWhiteSpace($OtaBin) -and -not $cargoBin) {
    throw "could not resolve an ota binary; set OTA_BIN, pass -OtaBin, build the checkout, or install ota on PATH"
}

if ([string]::IsNullOrWhiteSpace($Title)) {
    switch ($Mode)
    {
        "doctor" {
            $Title = "ota doctor"
        }
        "workspace-doctor" {
            $Title = "ota workspace doctor"
        }
        "receipt-diff" {
            $Title = "ota receipt diff"
        }
    }
}

if ($cargoBin) {
    & $cargoBin run --quiet --manifest-path (Join-Path $repoRoot "Cargo.toml") -- annotations --mode $Mode --format $Format --title $Title --input $JsonPath
} else {
    & $OtaBin annotations --mode $Mode --format $Format --title $Title --input $JsonPath
}

if ($LASTEXITCODE -ne 0)
{
    exit $LASTEXITCODE
}
