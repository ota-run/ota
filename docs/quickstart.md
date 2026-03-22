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

# Ota Quickstart

## Use an existing contract

From a repo with `ota.yaml`:

```bash
cargo run -- validate
cargo run -- tasks
cargo run -- doctor
cargo run -- up
```

Run a task explicitly:

```bash
cargo run -- run test
```

Tasks can use either a single-command `run` or an inline multiline `script`.

Tasks can also declare OS-specific variants while keeping one stable task name.

## Validate a workspace

From a directory with `ota.workspace.yaml`:

```bash
cargo run -- workspace validate
cargo run -- workspace doctor
cargo run -- workspace up
```

Ota resolves `ota.workspace.yaml` upward the same way repo commands resolve `ota.yaml`.

## Detect a starting contract

Review first:

```bash
cargo run -- init
cargo run -- detect --dry-run /path/to/repo
```

Write only if the `high` confidence projection is sufficient:

```bash
cargo run -- init --write
cargo run -- detect /path/to/repo
```

Current detect sources:

- `package.json`
- `.nvmrc`
- `.node-version`
- `.tool-versions`
- `pyproject.toml`
- `.python-version`
- `go.mod`
- `settings.gradle(.kts)`
- `build.gradle(.kts)`
- `gradle/wrapper/gradle-wrapper.properties`
- `pom.xml`
- `docker-compose.yml` / `docker-compose.yaml`
- `compose.yml` / `compose.yaml`

## Example contract

Examples:

- [../../examples/basic-node/ota.yaml](../../examples/basic-node/ota.yaml)
- [../../examples/basic-python/ota.yaml](../../examples/basic-python/ota.yaml)
- [../../examples/basic-go/ota.yaml](../../examples/basic-go/ota.yaml)
- [../../examples/basic-script/ota.yaml](../../examples/basic-script/ota.yaml)
- [../../examples/basic-services/ota.yaml](../../examples/basic-services/ota.yaml)
- [../../examples/mixed-node-python/ota.yaml](../../examples/mixed-node-python/ota.yaml)
- [../../examples/fullstack-node-go/ota.yaml](../../examples/fullstack-node-go/ota.yaml)
- [../../examples/full-contract/ota.yaml](../../examples/full-contract/ota.yaml)
- [../spec/command-reference.md](../spec/command-reference.md)
- [../spec/contract-reference.md](../spec/contract-reference.md)
- [../spec/workspace-reference.md](../spec/workspace-reference.md)
- [philosophy.md](philosophy.md)
