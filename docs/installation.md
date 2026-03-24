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

# Ota Installation

Ota currently installs through Cargo.

## macOS/Linux (shell)

From a cloned Ota repository:

```bash
./scripts/install.sh --from-source
```

This installs `ota` with:

```bash
cargo install --path . --locked --force
```

## Git install (without cloning first)

```bash
./scripts/install.sh
```

Defaults:

- `OTA_GIT_URL=https://github.com/ota-run/ota.git`
- latest git default branch

Optional pinning:

- `OTA_GIT_TAG=v0.1.0`
- `OTA_GIT_BRANCH=main`
- `OTA_GIT_REV=<commit-sha>`

Set at most one of `OTA_GIT_TAG`, `OTA_GIT_BRANCH`, `OTA_GIT_REV`.

## Windows (PowerShell)

From a cloned Ota repository:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1 -FromSource
```

Git install:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1
```

Optional pinning uses the same env vars:

- `OTA_GIT_URL`
- `OTA_GIT_TAG`
- `OTA_GIT_BRANCH`
- `OTA_GIT_REV`

## Verify

```bash
ota --version
ota validate --help
```
