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
    [ValidateSet("doctor", "workspace-doctor")]
    [string]$Mode,

    [Parameter(Mandatory = $true)]
    [ValidateSet("plain", "github")]
    [string]$Format,

    [Alias("Input")]
    [Parameter(Mandatory = $true)]
    [string]$JsonPath,

    [string]$Title
)

$ErrorActionPreference = "Stop"

function Escape-GhaValue {
    param([string]$Value)
    if ($null -eq $Value) {
        return ""
    }
    return $Value.Replace("%", "%25").Replace("`r", "%0D").Replace("`n", "%0A")
}

function Write-FindingLine {
    param(
        [string]$Severity,
        [string]$Heading,
        [string]$Body,
        [string]$Next
    )

    $safeHeading = Escape-GhaValue $Heading
    $safeBody = Escape-GhaValue $Body
    $safeNext = Escape-GhaValue $Next

    switch ($Format) {
        "github" {
            if ($Severity -eq "error") {
                Write-Host "::error title=$safeHeading::$safeBody | $safeNext"
            } else {
                Write-Host "::warning title=$safeHeading::$safeBody | $safeNext"
            }
        }
        "plain" {
            if ($Severity -eq "error") {
                Write-Host "ERROR: ${Heading}: ${Body} | ${Next}"
            } else {
                Write-Host "WARNING: ${Heading}: ${Body} | ${Next}"
            }
        }
    }
}

function Write-PrimaryBlockerLine {
    param(
        [string]$Heading,
        [string]$Body,
        [string]$Next
    )

    $safeHeading = Escape-GhaValue $Heading
    $safeBody = Escape-GhaValue $Body
    $safeNext = Escape-GhaValue $Next

    switch ($Format) {
        "github" {
            Write-Host "::notice title=$safeHeading::$safeBody | $safeNext"
        }
        "plain" {
            Write-Host "NOTICE: ${Heading}: ${Body} | ${Next}"
        }
    }
}

if ([string]::IsNullOrWhiteSpace($Title)) {
    $Title = if ($Mode -eq "doctor") { "ota doctor" } else { "ota workspace doctor" }
}

$json = Get-Content -Raw -LiteralPath $JsonPath | ConvertFrom-Json

switch ($Mode) {
    "doctor" {
        if ($json.summary.primary_blocker) {
            Write-PrimaryBlockerLine -Heading "$Title primary blocker" -Body $json.summary.primary_blocker.summary -Next $json.summary.primary_blocker.next
        }

        foreach ($finding in @($json.findings)) {
            if ($null -eq $finding) { continue }
            Write-FindingLine -Severity $finding.severity -Heading "$Title finding" -Body $finding.summary -Next $finding.next
        }
    }
    "workspace-doctor" {
        if ($json.summary.primary_blocker) {
            $repo = $json.summary.primary_blocker.repo
            Write-PrimaryBlockerLine -Heading "$Title primary blocker [$repo]" -Body $json.summary.primary_blocker.summary -Next $json.summary.primary_blocker.next
        }

        foreach ($repo in @($json.repos)) {
            if ($null -eq $repo) { continue }
            foreach ($finding in @($repo.findings)) {
                if ($null -eq $finding) { continue }
                Write-FindingLine -Severity $finding.severity -Heading "$Title finding [$($repo.name)]" -Body "$($repo.path): $($finding.summary)" -Next $finding.next
            }
        }
    }
}
