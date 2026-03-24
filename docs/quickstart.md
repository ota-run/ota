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

Install Ota first: [installation.md](installation.md)

## Use an existing contract

From a repo with `ota.yaml`:

```bash
cargo run -- validate
cargo run -- tasks
cargo run -- tasks --json
cargo run -- doctor
cargo run -- up
```

Run a task explicitly:

```bash
cargo run -- run test
```

Tasks can use either a single-command `run` or an inline multiline `script`.

Tasks can also declare OS-specific variants while keeping one stable task name.

If the contract declares an `agent` section, `ota tasks --json` and `ota doctor --json` surface
the current entrypoint, safe task set, verification tasks, and writable paths without creating a
separate agent-only config path.

## Validate a workspace

From a directory with `ota.workspace.yaml`:

```bash
cargo run -- workspace validate
cargo run -- workspace tasks
cargo run -- workspace run setup
cargo run -- workspace check
cargo run -- workspace doctor
cargo run -- workspace up
```

Ota resolves `ota.workspace.yaml` upward the same way repo commands resolve `ota.yaml`.

If a workspace repo is missing locally but declares `repos.<name>.source`, `ota workspace up`
can acquire it first and then reuse the existing repo-level bootstrap flow.

Use `source.git` as the canonical explicit clone URL. Use `source.repo` only as shorthand when
multiple repos share the same `workspace.git_base`.

If you want raw live child logs during workspace setup, opt in explicitly:

```bash
cargo run -- workspace up --stream
```

`--stream` is currently text-only and requires sequential execution.

## Detect a starting contract

Review first:

```bash
cargo run -- init
cargo run -- detect --dry-run /path/to/repo
```

`ota init` is the repo-local starter path. It now tells you what to do next in text mode:

- review the generated starter contract first
- write it only when ready
- run `ota validate` and `ota doctor` after writing
- treat `Mode: blank` as minimal coverage, not a complete contract
- in `Mode: detected`, automatic write is conservative and only persists `high` confidence fields when that is enough for a valid contract

Write only if the `high` confidence projection is sufficient:

```bash
cargo run -- init --write
cargo run -- detect /path/to/repo
```

If `ota.yaml` already exists, review or conservatively merge instead of overwriting:

```bash
cargo run -- detect --merge --dry-run /path/to/repo
cargo run -- detect --merge /path/to/repo
```

Current merge behavior:

- `ota detect --merge --dry-run` is the review path for existing contracts
- `ota detect --merge` applies only additive `high` confidence missing fields
- conflicting or lower-confidence changes stay review-only
- when nothing eligible can be added, merge returns success with `written: false`

Current detect sources:

- `package.json`
- `pnpm-workspace.yaml`
- `pnpm-lock.yaml`
- `yarn.lock`
- `bun.lock` / `bun.lockb`
- `package-lock.json`
- `npm-shrinkwrap.json`
- `.nvmrc`
- `.node-version`
- `.tool-versions`
- `pyproject.toml`
- `Pipfile`
- `uv.lock`
- `requirements.txt`
- `setup.cfg`
- `.python-version`
- `.java-version`
- `.sdkmanrc`
- `go.mod`
- `Cargo.toml`
- `rust-toolchain.toml`
- `rust-toolchain`
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
- [../../examples/basic-java/ota.yaml](../../examples/basic-java/ota.yaml)
- [../../examples/basic-rust/ota.yaml](../../examples/basic-rust/ota.yaml)
- [../../examples/basic-script/ota.yaml](../../examples/basic-script/ota.yaml)
- [../../examples/basic-services/ota.yaml](../../examples/basic-services/ota.yaml)
- [../../examples/mixed-node-python/ota.yaml](../../examples/mixed-node-python/ota.yaml)
- [../../examples/fullstack-node-go/ota.yaml](../../examples/fullstack-node-go/ota.yaml)
- [../../examples/full-contract/ota.yaml](../../examples/full-contract/ota.yaml)
- [../spec/command-reference.md](../spec/command-reference.md)
- [../spec/contract-reference.md](../spec/contract-reference.md)
- [../spec/workspace-reference.md](../spec/workspace-reference.md)
- [../../examples/workspace-acquire/ota.workspace.yaml](../../examples/workspace-acquire/ota.workspace.yaml)
- [philosophy.md](philosophy.md)
