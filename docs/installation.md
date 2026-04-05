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

ota ships prebuilt release binaries for macOS/Linux and Windows.
Use source install only when developing ota from a cloned checkout.
The installers include a branded otter banner and a compact success receipt so the install path feels intentionally ota-native.
The hosted install scripts are intentionally stable root URLs:
[`install.sh`](https://dist.ota.run/install.sh) and [`install.ps1`](https://dist.ota.run/install.ps1).

## macOS/Linux

Install the latest release binary:

```bash
curl -fsSL https://dist.ota.run/install.sh | sh
```

Pin a release:

```bash
OTA_VERSION=vX.Y.Z curl -fsSL https://dist.ota.run/install.sh | sh
```

From a cloned ota repository:

```bash
./scripts/install.sh --from-source
```

The shell installer also supports `OTA_RELEASE_BASE` if you host the release assets on a mirror or CDN.

## Windows (PowerShell)

Install the latest release binary:

```powershell
iwr https://dist.ota.run/install.ps1 | iex
```

Pin a release:

```powershell
$env:OTA_VERSION = "vX.Y.Z"
iwr https://dist.ota.run/install.ps1 | iex
```

From a cloned ota repository:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1 -FromSource
```

The PowerShell installer also supports `OTA_RELEASE_BASE` for a mirror or CDN.

## Windows (Git Bash / MSYS / MinGW / Cygwin)

Install the latest release binary:

```bash
curl -fsSL https://dist.ota.run/install.sh | sh
```

This path now recognizes Windows-style shells and downloads the release binary instead of falling back to cargo.

## Verify

```bash
ota --version
ota validate --help
```

## Maintainer version bump

Use the dedicated bump scripts to update `Cargo.toml` safely:

```bash
./scripts/bump-version.sh 0.2.0
```

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\bump-version.ps1 0.2.0
```
