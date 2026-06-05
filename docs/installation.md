<!--
                █████
               ░░███
       ██████  ███████    ██████
      ███░░███░░░███░    ░░░░░███
     ░███ ░███  ░███      ███████
     ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
      ░░░░░░     ░░░░░   ░░░░░░░░

   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
   You may not use this file except in compliance with that License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# ota Installation

ota ships prebuilt release binaries for the mainstream GitHub-hosted target matrix:

- Linux: `x86_64-unknown-linux-gnu`, `aarch64-unknown-linux-gnu`
- macOS: `x86_64-apple-darwin`, `aarch64-apple-darwin`
- Windows: `x86_64-pc-windows-msvc`, `aarch64-pc-windows-msvc`

Use source install only when developing ota from a cloned checkout.
The installers include a branded otter banner and a compact success receipt so the install path feels intentionally ota-native.
The hosted install scripts are intentionally stable root URLs:
[`install.sh`](https://dist.ota.run/install.sh) and [`install.ps1`](https://dist.ota.run/install.ps1).

## macOS/Linux

Install the latest release binary:

```bash
curl -fsSL https://dist.ota.run/install.sh | sh
```

Persist the install directory on `PATH` explicitly:

```bash
curl -fsSL https://dist.ota.run/install.sh | sh -s -- --setup-path
```

Pin a release:

```bash
curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION=vX.Y.Z sh
```

From a cloned ota repository:

```bash
./scripts/install.sh --from-source
```

Persist the source-install binary directory on `PATH` explicitly:

```bash
./scripts/install.sh --from-source --setup-path
```

The shell installer also supports `OTA_RELEASE_BASE` if you host the release assets on a mirror or CDN.
If a prebuilt release is not published for the detected target, the installer now says so explicitly before trying the cargo fallback.
`--setup-path` is opt-in: ota will not silently modify shell startup files unless you ask it to.

### Install the Ota skill (PowerShell)

```bash
npx skills add ota-run/skills --full-depth
```

## Windows (PowerShell)

Install the latest release binary:

```powershell
irm https://dist.ota.run/install.ps1 | iex
```

Persist the install directory on `PATH` explicitly:

```powershell
& ([scriptblock]::Create((irm https://dist.ota.run/install.ps1))) -SetupPath
```

Pin a release:

```powershell
$env:OTA_VERSION = "vX.Y.Z"
irm https://dist.ota.run/install.ps1 | iex
```

From a cloned ota repository:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1 -FromSource
```

Persist the source-install binary directory on `PATH` explicitly:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1 -FromSource -SetupPath
```

The PowerShell installer also supports `OTA_RELEASE_BASE` for a mirror or CDN.
If a prebuilt release is not published for the detected target, the installer now says so explicitly before trying the cargo fallback.
Official release installs require published checksums; custom mirrors can set `OTA_ALLOW_MISSING_CHECKSUMS=1` if they intentionally host assets without checksum files.

### Install the Ota skill

```powershell
npx skills add ota-run/skills --full-depth
```

## Windows (Git Bash / MSYS / MinGW / Cygwin)

Install the latest release binary:

```bash
curl -fsSL https://dist.ota.run/install.sh | sh
```

This path now recognizes Windows-style shells and downloads the release binary instead of falling back to cargo.
If the existing `ota.exe` is locked by a running process, the installer stages the replacement, reports it as pending, and exits successfully.

## Verify

```bash
ota --version
ota validate --help
```

## Maintainer version bump

Use the Ota task surface to update the repo release version surfaces consistently:

```bash
ota run bump:version --version patch
ota run bump:version --version minor
ota run bump:version --version major
ota run bump:version --version 0.2.0
```

```powershell
ota run bump:version --version patch
ota run bump:version --version 0.2.0
```
