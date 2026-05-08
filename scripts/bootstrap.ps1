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

function Write-OtaReceipt {
    param([string]$Message)
    Write-Host "${Esc}[1;38;2;214;161;95m$Message${Esc}[0m"
}

function Write-OtaReceiptLine {
    param([string]$Message)
    Write-Host "${Esc}[1;38;2;214;161;95m➤${Esc}[0m ${Esc}[1;37m$Message${Esc}[0m"
}

function Write-OtaWarn {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Yellow
}

function Write-OtaError {
    param([string]$Message)
    Write-Host $Message -ForegroundColor Red
}

function Normalize-VersionOutput {
    param([object]$Value)

    if ($null -eq $Value) {
        return ""
    }

    $text = [System.String]::Format("{0}", $Value)
    if ([string]::IsNullOrWhiteSpace($text)) {
        return ""
    }

    return $text.Trim()
}

function Get-OtaTarget {
    $arch = [System.Runtime.InteropServices.RuntimeInformation]::OSArchitecture.ToString().ToLowerInvariant()
    switch ($arch) {
        "x64" { $arch = "x86_64" }
        "arm64" { $arch = "aarch64" }
        default { return $null }
    }

    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
        return "${arch}-pc-windows-msvc"
    }
    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Linux)) {
        return "${arch}-unknown-linux-gnu"
    }
    if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::OSX)) {
        return "${arch}-apple-darwin"
    }

    return $null
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

    $separator = [System.IO.Path]::PathSeparator
    if ([string]::IsNullOrWhiteSpace($env:Path)) {
        $env:Path = $Dir
    } else {
        $env:Path = "$Dir$separator$env:Path"
    }

    try {
        if (-not [System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) {
            return
        }

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

function Test-OtaFileInUseError
{
    param([System.Management.Automation.ErrorRecord]$ErrorRecord)

    $exception = $ErrorRecord.Exception
    while ($exception)
    {
        if ($exception -is [System.IO.IOException])
        {
            $hresult = $exception.HResult -band 0xFFFF
            if ($hresult -eq 32 -or $hresult -eq 33)
            {
                return $true
            }

            if ([string]$exception.Message -match "being used by another process|cannot access the file")
            {
                return $true
            }
        }

        $exception = $exception.InnerException
    }

    return $false
}

function Invoke-DetachedShell
{
    param([string]$Command)

    $arguments = @(
        "-NoLogo"
        "-NoProfile"
        "-ExecutionPolicy"
        "Bypass"
        "-Command"
        $Command
    )

    try
    {
        Start-Process -FilePath "pwsh" -ArgumentList $arguments -WindowStyle Hidden -ErrorAction Stop | Out-Null
        return $true
    }
    catch
    {
        if ($_.Exception -is [System.ComponentModel.Win32Exception])
        {
            return $false
        }
    }

    try
    {
        Start-Process -FilePath "powershell" -ArgumentList $arguments -WindowStyle Hidden -ErrorAction Stop | Out-Null
        return $true
    }
    catch
    {
        return $false
    }
}

function Get-OtaSelfUpdateParentPid
{
    $envParent = [Environment]::GetEnvironmentVariable("OTA_SELF_UPDATE_PARENT_PID")
    if ($envParent -and $envParent -match "^\d+$")
    {
        return [int]$envParent
    }

    try
    {
        $process = Get-CimInstance Win32_Process -Filter "ProcessId = $PID" -ErrorAction Stop
        if ($process -and $process.ParentProcessId)
        {
            return [int]$process.ParentProcessId
        }
    }
    catch
    {
    }

    return 0
}

function Schedule-OtaReplacementAfterExit
{
    param(
        [string]$Source,
        [string]$Destination,
        [int]$ParentPid
    )

    $helper = Join-Path $env:TEMP ("ota-self-update-" + [Guid]::NewGuid().ToString("N") + ".ps1")

    $sourceEsc = $Source -replace "'", "''"
    $destinationEsc = $Destination -replace "'", "''"
    $helperScript = @"
`$parentPid = $ParentPid
`$source = '$sourceEsc'
`$destination = '$destinationEsc'

`$waited = 0
while (`$parentPid -gt 0 -and `$waited -lt 1800 -and (Get-Process -Id `$parentPid -ErrorAction SilentlyContinue)) {
    Start-Sleep -Milliseconds 200
    `$waited++
}

`$attempt = 0
while (`$attempt -lt 1800 -and (Test-Path -LiteralPath `$source)) {
    try {
        Copy-Item -Path `$source -Destination `$destination -Force
        Remove-Item -LiteralPath `$source -Force -ErrorAction SilentlyContinue
        Remove-Item -LiteralPath `$MyInvocation.MyCommand.Path -Force -ErrorAction SilentlyContinue
        exit 0
    } catch {
        Start-Sleep -Milliseconds 200
        `$attempt++
    }
}

Remove-Item -LiteralPath `$MyInvocation.MyCommand.Path -Force -ErrorAction SilentlyContinue
exit 1
"@

    Set-Content -Path $helper -Value $helperScript -Encoding UTF8 -Force
    if (Test-Path $helper)
    {
        return Invoke-DetachedShell -Command "& '$helper'"
    }
    return $false
}

function Resolve-OtaLockedReplacement
{
    param(
        [System.Management.Automation.ErrorRecord]$ErrorRecord,
        [string]$Staged,
        [string]$Destination
    )

    if (-not $IsWindows -or -not (Test-OtaFileInUseError $ErrorRecord))
    {
        return $null
    }

    if (-not (Test-Path $Staged))
    {
        return $null
    }

    if (-not (Schedule-OtaReplacementAfterExit -Source $Staged -Destination $Destination -ParentPid (Get-OtaSelfUpdateParentPid)))
    {
        Write-OtaError "error: ota is running but the staged replacement could not be scheduled"
        if (Test-Path $Staged)
        {
            Remove-Item -LiteralPath $Staged -Force -ErrorAction SilentlyContinue
        }
        return "failed"
    }

    Write-OtaError "error: ota is still running; staged update at $Staged but replacement is not yet verified"
    Write-OtaWarn "close running ota processes, then rerun `ota --version` to confirm the new version"
    return "pending"
}

function Install-FromSource {
    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-OtaError "cargo is required for source install"
        Write-OtaError "install Rust/cargo or use a published prebuilt ota release for your target"
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
        Write-OtaError "install Rust/cargo or use a published prebuilt ota release for your target"
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
        Write-OtaWarn "warning: no published prebuilt ota release is configured for this OS/arch"
        return $false
    }

    $version = if ($env:OTA_VERSION) { $env:OTA_VERSION } else { "latest" }
    $releaseBase = if ($env:OTA_RELEASE_BASE) { $env:OTA_RELEASE_BASE } else { "https://github.com/ota-run/ota/releases" }
    $asset = if ($target -like "*-pc-windows-msvc") { "ota-$target.zip" } else { "ota-$target.tar.gz" }
    $checksumAsset = "ota-checksums.txt"
    $downloadPrefix = if ($version -eq "latest") { "$releaseBase/latest/download" } else { "$releaseBase/download/$version" }
    $tmpdir = Join-Path ([System.IO.Path]::GetTempPath()) ("ota-install-" + [Guid]::NewGuid().ToString("N"))
    $archive = Join-Path $tmpdir $asset
    $checksums = Join-Path $tmpdir $checksumAsset

    New-Item -ItemType Directory -Force -Path $tmpdir | Out-Null

    try {
        Write-OtaInfo "installing ota $version for $target..."
        if (-not (Download-OtaFile "$downloadPrefix/$asset" $archive)) {
            Write-OtaWarn "warning: could not download prebuilt ota release asset for $target at $version ($asset)"
            return $false
        }

        if (Download-OtaFile "$downloadPrefix/$checksumAsset" $checksums) {
            $checksumLine = Get-Content $checksums | Where-Object {
                $parts = $_ -split "\s+", 2
                if ($parts.Count -lt 2) { return $false }
                $name = $parts[1].Trim()
                $name -eq $asset -or $name -eq "dist/$asset"
            } | Select-Object -First 1
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

        if ($asset.EndsWith(".zip")) {
            if (-not (Get-Command Expand-Archive -ErrorAction SilentlyContinue)) {
                Write-OtaError "error: Expand-Archive is required to unpack release artifacts"
                return $false
            }

            Expand-Archive -Path $archive -DestinationPath $tmpdir -Force
        } else {
            if (-not (Get-Command tar -ErrorAction SilentlyContinue)) {
                Write-OtaError "error: tar is required to unpack release artifacts"
                return $false
            }

            & tar -xzf $archive -C $tmpdir
        }

        $binaryName = if ($asset.EndsWith(".zip")) { "ota.exe" } else { "ota" }
        $binary = Get-ChildItem -Path $tmpdir -Filter $binaryName -Recurse | Select-Object -First 1
        if (-not $binary) {
            Write-OtaError "error: release artifact did not contain $binaryName"
            return $false
        }

        $binDir = Get-OtaBinDir
        New-Item -ItemType Directory -Force -Path $binDir | Out-Null
        $destination = Join-Path $binDir $binaryName
        $staged = "$destination.new"
        try
        {
            Copy-Item -Path $binary.FullName -Destination $staged -Force
            try
            {
                Copy-Item -Path $staged -Destination $destination -Force
                Remove-Item -LiteralPath $staged -Force -ErrorAction SilentlyContinue
            }
            catch
            {
                $replacementStatus = Resolve-OtaLockedReplacement -ErrorRecord $_ -Staged $staged -Destination $destination
                if ($replacementStatus)
                {
                    return $replacementStatus
                }
                throw
            }

            Ensure-OtaOnPath $binDir
            Write-OtaInfo "installed ota to $destination"
            return "installed"
        } catch {
            $replacementStatus = Resolve-OtaLockedReplacement -ErrorRecord $_ -Staged $staged -Destination $destination
            if ($replacementStatus)
            {
                return $replacementStatus
            }
            if ($IsWindows) {
                Write-OtaError "error: could not replace ${destination}: $($_.Exception.Message)"
                if (Test-Path $staged) {
                Remove-Item -LiteralPath $staged -Force -ErrorAction SilentlyContinue
                }
                return "failed"
            }
            throw
        }
    } finally {
        if (Test-Path $tmpdir) {
            Remove-Item -Recurse -Force $tmpdir
        }
    }
}

Write-OtaHeader

$installMode = if ($FromSource.IsPresent)
{
    "source"
}
elseif ($FromGit.IsPresent)
{
    "git"
}
elseif ($FromRelease.IsPresent)
{
    "release"
}
elseif (-not [string]::IsNullOrWhiteSpace($env:OTA_INSTALL_MODE))
{
    $env:OTA_INSTALL_MODE.Trim().ToLowerInvariant()
}
else
{
    "release"
}

$installModeForced = $FromSource.IsPresent -or $FromGit.IsPresent -or $FromRelease.IsPresent -or
        (-not [string]::IsNullOrWhiteSpace($env:OTA_INSTALL_MODE))

if (-not $installModeForced -and (Test-Path ".\Cargo.toml") -and (Select-String -Path ".\Cargo.toml" -Pattern '^name = "ota"$' -Quiet))
{
    $installMode = "source"
}

if ($installMode -eq "source")
{
    Install-FromSource
}
elseif ($installMode -eq "git")
{
    Install-FromGit
}
else
{
    $releaseInstallStatus = Install-ReleaseBinary
    if ($releaseInstallStatus -eq "pending")
    {
        exit 1
    }
    if ($releaseInstallStatus -ne "installed")
    {
        if ($installMode -eq "release" -and $installModeForced)
        {
            Write-OtaError "error: prebuilt release install failed; refusing cargo fallback in explicit release mode"
            exit 1
        }
        Write-OtaWarn "warning: falling back to git install via cargo"
        Install-FromGit
    }
}

$binaryName = if ([System.Runtime.InteropServices.RuntimeInformation]::IsOSPlatform([System.Runtime.InteropServices.OSPlatform]::Windows)) { "ota.exe" } else { "ota" }

if (Get-Command ota -ErrorAction SilentlyContinue) {
    $binaryPath = (Get-Command ota).Source
    $versionOutput = Normalize-VersionOutput (& ota --version 2>$null | Out-String)
} elseif ($env:OTA_BIN_DIR -and (Test-Path (Join-Path $env:OTA_BIN_DIR $binaryName))) {
    $binaryPath = Join-Path $env:OTA_BIN_DIR $binaryName
    $versionOutput = Normalize-VersionOutput (& $binaryPath --version 2>$null | Out-String)
} elseif (Test-Path (Join-Path (Get-OtaBinDir) $binaryName)) {
    $binaryPath = Join-Path (Get-OtaBinDir) $binaryName
    $versionOutput = Normalize-VersionOutput (& $binaryPath --version 2>$null | Out-String)
} elseif (Test-Path (Join-Path $HOME ".local/bin/$binaryName")) {
    $binaryPath = Join-Path $HOME ".local/bin/$binaryName"
    $versionOutput = Normalize-VersionOutput (& $binaryPath --version 2>$null | Out-String)
} elseif (Test-Path (Join-Path $HOME ".cargo/bin/$binaryName")) {
    $binaryPath = Join-Path $HOME ".cargo/bin/$binaryName"
    $versionOutput = Normalize-VersionOutput (& $binaryPath --version 2>$null | Out-String)
} else {
    Write-OtaError "install completed but 'ota' is not on PATH yet"
    exit 1
}

if ([string]::IsNullOrWhiteSpace($versionOutput)) {
    $versionOutput = "unknown"
} else {
    $versionOutput = $versionOutput -replace '^ota\s+', ''
    $versionOutput = $versionOutput -replace '^🦦\s+', ''
}

$duplicatePaths = @()
if ((Test-Path (Join-Path $HOME ".local/bin/$binaryName")) -and $binaryPath -ne (Join-Path $HOME ".local/bin/$binaryName")) {
    $duplicatePaths += (Join-Path $HOME ".local/bin/$binaryName")
}
if ((Test-Path (Join-Path $HOME ".cargo/bin/$binaryName")) -and $binaryPath -ne (Join-Path $HOME ".cargo/bin/$binaryName")) {
    $duplicatePaths += (Join-Path $HOME ".cargo/bin/$binaryName")
}
if ($duplicatePaths.Count -gt 0) {
    Write-OtaWarn "warning: multiple ota binaries were found; PATH is using $binaryPath"
    Write-OtaWarn "warning: remove or de-prioritize the other copy/copies: $($duplicatePaths -join ', ')"
}

Write-OtaReceipt "🦦 READY"
Write-OtaReceiptLine $versionOutput
