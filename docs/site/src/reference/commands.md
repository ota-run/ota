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

# Commands

This page is adoption-first: each command explains when to use it, why it exists,
and a practical use-case.

## Start with this flow

1. `ota doctor` to understand readiness blockers.
1. `ota up` to make the repo runnable.
1. `ota run <task>` for day-to-day task execution.
1. `ota detect --dry-run` before writing any new contract.

## Repo commands

### `ota validate`

When to use:

- before commit or CI to prove contract correctness

Why:

- prevents invalid `ota.yaml` from breaking execution workflows

Use-case:

- guard PRs that modify tasks, services, or runtime requirements

```bash
ota validate
ota validate --json
```

### `ota doctor`

When to use:

- first command in a new repo or broken environment

Why:

- shows actionable blockers and warnings with explicit next steps

Use-case:

- teammate cannot run a repo; doctor reports missing runtime/tool/env quickly

```bash
ota doctor
ota doctor --json
```

### `ota up`

When to use:

- after diagnosis, when you want repo-ready state with minimal manual sequencing

Why:

- executes deterministic setup path: validate, preconditions, services, setup, post-check

Use-case:

- onboarding a new contributor who just cloned the repo

```bash
ota up
ota up --json
```

### `ota run <task>`

When to use:

- day-to-day execution after repo readiness is established

Why:

- runs named tasks with dependency ordering and stable behavior

Use-case:

- `ota run test`, `ota run dev`, `ota run lint` in CI or local loops

```bash
ota run test
```

### `ota tasks`

When to use:

- to discover supported task surface and resolved task variants

Why:

- gives one canonical list for humans and agents

Use-case:

- quickly inspect what the repo considers safe/official entrypoint tasks

```bash
ota tasks
ota tasks --json
```

### `ota check`

When to use:

- when you want checks only, without setup/task execution

Why:

- faster signal for CI or pre-commit verification

Use-case:

- run policy/health checks in PR validation

```bash
ota check
ota check --json
```

### `ota init`

When to use:

- repo has no `ota.yaml` and you want a starter contract

Why:

- provides safe onboarding entry before full manual authoring

Use-case:

- bootstrap Ota adoption for an existing project

```bash
ota init
ota init --json
```

### `ota detect`

When to use:

- you want contract inference from existing repo signals

Why:

- speeds adoption while preserving trust with confidence/provenance model

Use-case:

- infer runtimes/tools/services from manifests and version files

```bash
ota detect --dry-run .
ota detect .
ota detect --merge --dry-run .
ota detect --merge .
```

### `ota clean`

When to use:

- remove persistent execution artifacts (for example persistent container state)

Why:

- keeps local environment predictable and recoverable

Use-case:

- reset stale persistent backend before rerunning setup

```bash
ota clean
```

## Workspace commands

### `ota workspace validate`

When to use:

- before running multi-repo orchestration

Why:

- confirms repo graph and source declarations are valid

```bash
ota workspace validate
ota workspace validate --json
```

### `ota workspace tasks`

When to use:

- inspect task availability across repos

Why:

- shows deterministic dependency order and task surface

```bash
ota workspace tasks
ota workspace tasks --json
```

### `ota workspace doctor`

When to use:

- diagnose readiness across all repos in a workspace

Why:

- central view of blockers without hiding per-repo context

```bash
ota workspace doctor
ota workspace doctor --json
```

### `ota workspace check`

When to use:

- checks-only pass across workspace repos

Why:

- lightweight CI signal across multiple repositories

```bash
ota workspace check
ota workspace check --json
```

### `ota workspace run <task>`

When to use:

- run same task across multiple repos with dependency ordering

Why:

- single command for coordinated multi-repo execution

```bash
ota workspace run test
ota workspace run test --json
```

### `ota workspace up`

When to use:

- acquire missing repos and prepare full workspace

Why:

- one deterministic bootstrap path that reuses repo-level `ota up`

```bash
ota workspace up
ota workspace up --json
```

## Machine integration

Use `--json` whenever output is consumed by scripts, CI, or agents.
Use exit codes together with JSON payloads for reliable automation.

Canonical command reference in repository:

- `docs/spec/command-reference.md`
- <https://github.com/ota-run/ota/blob/main/docs/spec/command-reference.md>
