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

# ota Quickstart

Install ota first: [installation.md](installation.md)

## Start Here

Doctor first, contract second.

Choose the path that matches the repo.

## Existing Repo With `ota.yaml`

Fastest proof of value:

```bash
ota doctor
ota explain
ota up
ota agents
ota run <task>
```

What this gives you:

- `ota doctor` tells you what is broken and what to do next
- `ota explain` turns the current findings into an ordered fix plan
- `ota up` prepares the repo from the contract instead of from guesswork
- `ota agents` turns the same contract into repo-local agent guidance
- `ota run <task>` executes a declared task through the same contract

If you are not sure which task to run next, use:

```bash
ota tasks --use
```

## Repo Without `ota.yaml`

Use the authoring path first:

```bash
ota doctor
ota explain
ota detect --dry-run .
ota init --dry-run
```

Then choose one explicit write path:

```bash
ota init
# or:
ota detect --write .
```

## Existing Repo With `ota.yaml`, But Contract Drift Is Suspected

Review the delta first:

```bash
ota detect --merge --dry-run .
ota detect --rewrite --dry-run .
ota validate
```

## Agent Guidance From The Same Contract

If you want repo-local agent guidance from the same contract:

```bash
ota agents
ota agents --write
```

`ota explain`, `ota tasks`, and `ota run <task>` stay useful once the contract exists. If the contract declares an `agent` section, `ota doctor --json` and `ota explain --json` surface the same safe-task, verification, and writable-path hints that humans can review in `ota.yaml`.

## Validate A Workspace

From a directory with `ota.workspace.yaml`:

```bash
ota workspace doctor
ota workspace explain
ota workspace up
ota workspace validate
ota workspace tasks
ota workspace run setup
```

Follow-on workspace commands stay available after the first bootstrap path:

```bash
ota workspace check
ota workspace diff
ota workspace status
ota workspace receipt
ota workspace refresh --dry-run
```

ota resolves `ota.workspace.yaml` upward the same way repo commands resolve `ota.yaml`.

If a workspace repo is missing locally but declares `repos.<name>.source`, `ota workspace up` can acquire it first and then reuse the existing repo-level bootstrap flow.

Use `source.git` as the canonical explicit clone URL. Use `source.repo` only as shorthand when multiple repos share the same `workspace.git_base`.

If you want raw live child logs during workspace setup, opt in explicitly:

```bash
ota workspace up --stream
```

`--stream` is currently text-only and requires sequential execution.

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
- [ota-run/examples](https://github.com/ota-run/examples) - advanced, production-adjacent examples and templates
- [philosophy.md](philosophy.md)
