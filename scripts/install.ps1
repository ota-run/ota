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

function Write-OtaHeader {
    Write-Host "                █████" -ForegroundColor DarkYellow
    Write-Host "               ░░███" -ForegroundColor DarkYellow
    Write-Host "       ██████  ███████    ██████" -ForegroundColor DarkYellow
    Write-Host "      ███░░███░░░███░    ░░░░░███" -ForegroundColor DarkYellow
    Write-Host "     ░███ ░███  ░███      ███████" -ForegroundColor DarkYellow
    Write-Host "     ░███ ░███  ░███ ███ ███░░███" -ForegroundColor DarkYellow
    Write-Host "     ░░██████   ░░█████ ░░████████" -ForegroundColor DarkYellow
    Write-Host "      ░░░░░░     ░░░░░   ░░░░░░░░" -ForegroundColor DarkYellow
    Write-Host ""
    Write-Host "     DOCTOR FIRST, CONTRACT SECOND" -ForegroundColor DarkYellow
    Write-Host ""
}

function Write-OtaInfo {
    param([string]$Message)
    Write-Host $Message -ForegroundColor DarkYellow
}

function Write-OtaWarn {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Yellow
}

function Write-OtaError {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Red
}

function Get-OtaTarget {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
    switch ($arch) {
        "x64" { return "x86_64-pc-windows-msvc" }
        "arm64" { return "aarch64-pc-windows-msvc" }
        default { return $null }
    }
}

function Get-OtaBinDir {
    if ($env:OTA_BIN_DIR) {
        return $env:OTA_BIN_DIR
    }
    if ($env:LOCALAPPDATA) {
        return (Join-Path $env:LOCALAPPDATA "ota\bin")
    }
    return (Join-Path $HOME ".local/bin")
}

function Download-OtaFile {
    param(
        [string]$Url,
        [string]$OutFile
    )

    try {
        Invoke-WebRequest -Uri $Url -OutFile $OutFile -ErrorAction Stop | Out-Null
        return $true
    } catch {
        return $false
    }
}

function Ensure-OtaOnPath {
    param([string]$Dir)

    if ([string]::IsNullOrWhiteSpace($Dir)) {
        return
    }

    $env:Path = "$Dir;$env:Path"

    try {
        $userPath = [Environment]::GetEnvironmentVariable("Path", "User")
        $parts = @()
        if (-not [string]::IsNullOrWhiteSpace($userPath)) {
            $parts = $userPath -split ";" | Where-Object { $_ -and $_.Trim() -ne "" }
        }

        if ($parts -contains $Dir) {
            return
        }

        $updatedPath = if ([string]::IsNullOrWhiteSpace($userPath)) { $Dir } else { "$userPath;$Dir" }
        [Environment]::SetEnvironmentVariable("Path", $updatedPath, "User")
    } catch {
        Write-OtaWarn "warning: could not persist PATH update; add $Dir to PATH manually"
    }
}

function Install-FromSource {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-OtaError "cargo is required for source install"
        exit 1
    }

    Write-OtaInfo "installing ota from local source (cargo install --path .)..."
    & cargo install --path . --locked --force
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
}

function Install-FromGit {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-OtaError "cargo is required for git install fallback"
        exit 1
    }

    $gitUrl = if ($env:OTA_GIT_URL) { $env:OTA_GIT_URL } else { "https://github.com/ota-run/ota.git" }
    $tag = $env:OTA_GIT_TAG
    $branch = $env:OTA_GIT_BRANCH
    $rev = $env:OTA_GIT_REV

    $refsSet = 0
    if ($tag) { $refsSet++ }
    if ($branch) { $refsSet++ }
    if ($rev) { $refsSet++ }
    if ($refsSet -gt 1) {
        Write-OtaError "set only one of OTA_GIT_TAG, OTA_GIT_BRANCH, OTA_GIT_REV"
        exit 1
    }

    Write-OtaInfo "installing ota from $gitUrl..."
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

function Install-ReleaseBinary {
    $target = Get-OtaTarget
    if (-not $target) {
        Write-OtaWarn "warning: unsupported Windows architecture for release binaries; trying cargo fallback"
        return $false
    }

    $version = if ($env:OTA_VERSION) { $env:OTA_VERSION } else { "latest" }
    $releaseBase = if ($env:OTA_RELEASE_BASE) { $env:OTA_RELEASE_BASE } else { "https://github.com/ota-run/ota/releases" }
    $asset = "ota-$target.zip"
    $checksumAsset = "ota-checksums.txt"
    $downloadPrefix = if ($version -eq "latest") { "$releaseBase/latest/download" } else { "$releaseBase/download/$version" }
    $tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("ota-install-" + [Guid]::NewGuid().ToString("N"))
    $archive = Join-Path $tmpdir $asset
    $checksums = Join-Path $tmpdir $checksumAsset

    New-Item -ItemType Directory -Force -Path $tmpdir | Out-Null

    try {
        Write-OtaInfo "installing ota $version for $target..."
        if (-not (Download-OtaFile "$downloadPrefix/$asset" $archive)) {
            Write-OtaWarn "warning: release artifact not available ($asset); trying cargo fallback"
            return $false
        }

        if (Download-OtaFile "$downloadPrefix/$checksumAsset" $checksums) {
            $checksumLine = Get-Content $checksums | Where-Object { $_ -match " $([regex]::Escape($asset))$" } | Select-Object -First 1
            if ($checksumLine) {
                $expected = ($checksumLine -split "\s+")[0].ToLowerInvariant()
                $actual = (Get-FileHash -Algorithm SHA256 -Path $archive).Hash.ToLowerInvariant()
                if ($actual -ne $expected) {
                    Write-OtaError "error: checksum verification failed for $asset"
                    return $false
                }
            }
        } else {
            Write-OtaWarn "warning: checksums not found; skipping checksum verification"
        }

        if (-not (Get-Command Expand-Archive -ErrorAction SilentlyContinue)) {
            Write-OtaError "error: Expand-Archive is required to unpack release artifacts"
            return $false
        }

        Expand-Archive -Path $archive -DestinationPath $tmpdir -Force
        $binary = Get-ChildItem -Path $tmpdir -Filter "ota.exe" -Recurse | Select-Object -First 1
        if (-not $binary) {
            Write-OtaError "error: release artifact did not contain ota.exe"
            return $false
        }

        $binDir = Get-OtaBinDir
        New-Item -ItemType Directory -Force -Path $binDir | Out-Null
        Copy-Item -Path $binary.FullName -Destination (Join-Path $binDir "ota.exe") -Force
        Ensure-OtaOnPath $binDir
        Write-OtaInfo "installed ota to $(Join-Path $binDir 'ota.exe')"
        return $true
    } finally {
        if (Test-Path $tmpdir) {
            Remove-Item -Recurse -Force $tmpdir
        }
    }
}

Write-OtaHeader

$installFromSource = $FromSource.IsPresent
if ((Test-Path ".\Cargo.toml") -and (Select-String -Path ".\Cargo.toml" -Pattern '^name = "ota"$' -Quiet)) {
    $installFromSource = $true
}

if ($installFromSource) {
    Install-FromSource
} elseif (-not (Install-ReleaseBinary)) {
    Write-OtaWarn "warning: falling back to git install via cargo"
    Install-FromGit
}

if (Get-Command ota -ErrorAction SilentlyContinue) {
    & ota --version
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} elseif ($env:OTA_BIN_DIR -and (Test-Path (Join-Path $env:OTA_BIN_DIR "ota.exe"))) {
    & (Join-Path $env:OTA_BIN_DIR "ota.exe") --version
    if ($LASTEXITCODE -ne 0) {
        exit $LASTEXITCODE
    }
} else {
    Write-OtaError "install completed but 'ota' is not on PATH yet"
    exit 1
}
