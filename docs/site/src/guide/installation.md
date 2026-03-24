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

# Installation

Prerequisite:

- Rust toolchain with `cargo` on `PATH`

Choose the path based on your use-case:

- local development from a cloned repo: install from source
- quick bootstrap without cloning first: install from git

macOS/Linux:

```bash
./scripts/install.sh --from-source
```

Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1 -FromSource
```

Git install:

```bash
./scripts/install.sh
```

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1
```

Verify installation:

```bash
ota --version
ota validate --help
```

```bash
ota doctor --help
```

For pinning options and details, see `docs/installation.md` in the repository.

## When to use each installer

### `install.sh --from-source` / `install.ps1 -FromSource`

When:

- you are developing Ota itself or testing local changes

Why:

- installs the current checked-out source deterministically

### `install.sh` / `install.ps1`

When:

- you need Ota quickly on a workstation without building a custom checkout

Why:

- installs from git with optional pinning (`OTA_GIT_TAG`, `OTA_GIT_REV`)
