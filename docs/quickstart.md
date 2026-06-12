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

If you are rolling ota out to one team instead of just one repo, use
[adoption/one-team-rollout.md](adoption/one-team-rollout.md) alongside this quickstart.
If you want one concrete existing-repo sequence, use
[adoption/worked-example-existing-repo.md](adoption/worked-example-existing-repo.md).
If you want the fastest path to the right example by repo shape, use
[adoption/examples-by-goal.md](adoption/examples-by-goal.md).

## Start Here

Doctor first, contract second.

Choose the path that matches the repo.

## Existing Repo With `ota.yaml`

Fastest proof of value:

```bash
ota doctor
ota up --dry-run
ota up
ota run <task>
ota proof runtime --workflow <name>
```

What this gives you:

- `ota doctor` tells you what is broken and what to do next
- `ota up --dry-run` shows the exact preparation lane before anything mutates
- `ota up` prepares the repo from the contract instead of from guesswork
- `ota run <task>` executes a declared task through the same contract
- `ota proof runtime` proves one declared front door becomes operational and captures the same canonical artifacts ota already uses internally

When one workflow path needs its own dotenv overlay, declare it on the task with
`tasks.<name>.env_files` instead of hard-coding `--env-file` or inline exports into the shell body.
When the repo also needs deterministic local env bootstrap, use a native `action` task such as
`copy_if_missing` or `ensure_env_file` and point workflow `prepare.task` at that finite host step.
When the workflow itself owns one rendered runtime artifact, prefer
`env.profiles.<name>.render.dotenv`; add `render.dotenv.template` when Ota should start from a
repo example file and then overlay the workflow-specific values deterministically.

If you are not sure which task to run next, use:

```bash
ota tasks --use
```

If you need a tighter task lane:

```bash
ota tasks --safe --use
ota tasks --unsafe --use
ota tasks --via native
ota tasks --via container
```

Use `--safe`/`--unsafe` to split by effective agent-safe status, and `--via` to filter by execution
backend lane.

If you want repo-local agent guidance from the same contract after the core loop is working, use:

```bash
ota agents
ota agents --write
```

## Repo Without `ota.yaml`

Use the authoring path first:

```bash
ota doctor
ota detect --dry-run .
ota init --dry-run
ota validate
ota up --dry-run
```

Then choose one explicit write path:

```bash
ota init
```

Use `ota detect --write .` when you want the detector-led authoring path instead of the starter contract path.

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

For full contract authoring guidance, use:

- [spec/contract-reference.md](spec/contract-reference.md) for `agent`, `effects`, `bootstrap`, and
  workflow fields
- [spec/command-reference.md](spec/command-reference.md) for command output semantics and filtering

## One Repo Rollout Story

For one repo, the repeatable local path is:

```bash
ota doctor
ota validate
ota up --dry-run
ota up
ota run ci
ota proof runtime --workflow <name>
```

For CI, keep the same contract boundary and archive the read-only receipt:

```bash
ota validate
ota doctor --json
ota receipt --json --archive
```

That keeps local readiness, CI evidence, and later receipt comparison on one surface.

When you want a repo-owned baseline for later compare gates:

```bash
ota receipt --json --archive --promote-baseline
ota receipt --json --baseline promoted
```

Use `promoted` when the team wants an explicit accepted repo state. Use `latest` when the newest
archived receipt is enough for a local drift check.

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

Choose by goal:

- first contract for one stack:
  [Basic Node](../../examples/basic-node/ota.yaml),
  [Basic Python](../../examples/basic-python/ota.yaml),
  [Basic Ruby](../../examples/basic-ruby/ota.yaml),
  [Basic Go](../../examples/basic-go/ota.yaml),
  [Basic Java](../../examples/basic-java/ota.yaml),
  [Basic Rust](../../examples/basic-rust/ota.yaml),
  [Basic .NET](../../examples/basic-dotnet/ota.yaml),
  [Basic Script](../../examples/basic-script/ota.yaml)
- normal app repo with services:
  [Basic Services](../../examples/basic-services/ota.yaml),
  [Mixed Node + Python](../../examples/mixed-node-python/ota.yaml),
  [Fullstack Node + Go](../../examples/fullstack-node-go/ota.yaml)
- shared topology:
  [Shared Local Topology](../../examples/shared-local-topology/ota.yaml),
  [Shared Remote Topology](../../examples/shared-remote-topology/README.md)
- multi-repo bootstrap:
  [Basic Workspace](../../examples/workspace-basic/ota.workspace.yaml),
  [Acquisition Workspace](../../examples/workspace-acquire/ota.workspace.yaml)
- broad reference:
  [Full Contract Example](../../examples/full-contract/ota.yaml),
  [ota-run/examples](https://github.com/ota-run/examples)

If you want the shortest “which example proves what?” guide, use
[adoption/examples-by-goal.md](adoption/examples-by-goal.md).

Reference docs:

- [../spec/command-reference.md](../spec/command-reference.md)
- [../spec/contract-reference.md](../spec/contract-reference.md)
- [../spec/workspace-reference.md](../spec/workspace-reference.md)
- [philosophy.md](philosophy.md)
