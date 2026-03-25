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

Doctor first, contract second.

Global output modifiers:

- `--concise`: shorter text output for high-noise commands while preserving decisions/actions.
- `--verbose`: full explanatory text output.
- `--json`: stable machine output; not affected by concise/verbose text shaping.

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

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota validate
ota doctor --json > .ota-doctor.json
```

### `ota doctor`

When to use:

- first command in a new repo or broken environment

Why:

- shows actionable blockers and warnings with explicit next steps
- `--concise` keeps severity/summary/next action and omits `Why` detail

Use-case:

- teammate cannot run a repo; doctor reports missing runtime/tool/env quickly

```bash
ota doctor
ota doctor --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

# fail fast in CI if repo is not ready
ota doctor --json | tee .ota-doctor.json
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

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota up
ota run test
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

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

TASK="${1:-test}"
ota run "$TASK"
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
ota tasks --use
ota tasks --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota tasks --json > .ota-tasks.json
cat .ota-tasks.json
```

### `ota services`

When to use:

- to inspect declared services and the contract fields that manage them

Why:

- services are readiness and startup dependencies, not direct task entrypoints

Use-case:

- confirm what `ota doctor` and `ota up` will manage before running them

```bash
ota services
ota services --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota services --json > .ota-services.json
cat .ota-services.json
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

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota check --json > .ota-check.json
```

### `ota init`

When to use:

- repo has no `ota.yaml` and you want a starter contract

Why:

- provides safe onboarding entry before full manual authoring
- writes the smallest valid starter contract for the repo
- may include medium-confidence fields when needed for a valid starter, while still excluding low-confidence fields

Use-case:

- bootstrap Ota adoption for an existing project

```bash
ota init
ota init --dry-run
ota init --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

# preview first, then write
ota init --dry-run
ota init
ota validate
```

### `ota detect`

When to use:

- you want contract inference from existing repo signals

Why:

- speeds adoption while preserving trust with confidence/provenance model
- `--write` stays conservative and only writes high-confidence fields

Use-case:

- infer runtimes/tools/services from manifests and version files
- refresh an existing contract with `--merge` instead of rerunning `init`

```bash
ota detect --dry-run .
ota detect --write .
ota detect --merge --dry-run .
ota detect --merge --apply tools.cargo --apply tasks.build.run .
ota detect --merge --apply-all .
ota detect --merge .
ota detect --rewrite --dry-run .
ota detect --rewrite --yes .
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

# contract does not exist yet
ota detect --dry-run .
ota detect --write .

# later, add high-confidence missing fields only
ota detect --merge --dry-run .
ota detect --merge .

# selectively apply only some detected fields
ota detect --merge --dry-run
ota detect --merge --apply tools.cargo --apply tasks.build.run
# leaves the rest of ota.yaml unchanged

# apply all eligible detected suggestions
ota detect --merge --apply-all

# if manual edits drift badly, preview full regenerate and then apply with confirmation
ota detect --rewrite --dry-run .
ota detect --rewrite --yes .
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

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota clean
ota up
```

## Workspace commands

### `ota workspace init`

When to use:

- create a first `ota.workspace.yaml` from already-initialized repos

Why:

- gives one deterministic workspace contract without hand-writing repo entries
- when no repo contracts are found, points to `ota init <repo-path>`, `ota detect --dry-run <repo-path>`, and `ota workspace init`

```bash
ota workspace init
ota workspace init --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace init --json > .ota-workspace-init.json
ota workspace init
```

### `ota workspace detect`

When to use:

- preview or merge inferred workspace repo entries

Why:

- keeps inferred/merge behavior explicit and reviewable, separate from init write
- when no repo contracts are found, points to `ota init <repo-path>`, `ota detect --dry-run <repo-path>`, and `ota workspace init`

```bash
ota workspace detect --dry-run
ota workspace detect --write
ota workspace detect --merge --dry-run
ota workspace detect --merge
ota workspace detect --rewrite --dry-run
ota workspace detect --rewrite --yes
ota workspace detect --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace detect --dry-run > /dev/null
ota workspace detect --merge --dry-run > /dev/null
ota workspace detect --merge

# when workspace contract is broken by manual edits, rewrite with confirmation
ota workspace detect --rewrite --dry-run > /dev/null
ota workspace detect --rewrite --yes
```

### `ota workspace validate`

When to use:

- before running multi-repo orchestration

Why:

- confirms repo graph and source declarations are valid

```bash
ota workspace validate
ota workspace validate --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace validate --json > .ota-workspace-validate.json
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

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace tasks --json > .ota-workspace-tasks.json
```

### `ota workspace list`

When to use:

- inventory workspace repos and contract presence without running diagnosis

Why:

- gives a fast, non-readiness view of required/optional repos, acquisition state, and missing contracts

```bash
ota workspace list
ota workspace list --repo api
ota workspace list --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace list --json > .ota-workspace-list.json
```

### `ota workspace doctor`

When to use:

- diagnose readiness across all repos in a workspace

Why:

- central view of blockers without hiding per-repo context
- `--concise` keeps repo status + finding summary/next action and omits per-repo path/contract and `Why` detail

```bash
ota workspace doctor
ota workspace doctor --json
ota workspace doctor --repo api
ota workspace doctor --status not-ready
ota workspace doctor --severity error
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace doctor --json > .ota-workspace-doctor.json
```

### `ota workspace check`

When to use:

- checks-only pass across workspace repos

Why:

- lightweight CI signal across multiple repositories
- `--concise` keeps repo status + finding summary/next action and omits per-repo path/contract and `Why` detail

```bash
ota workspace check
ota workspace check --json
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace check --json > .ota-workspace-check.json
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

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

TASK="${1:-test}"
ota workspace run "$TASK"
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

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace up
ota workspace run test
```

## Machine integration

Use `--json` whenever output is consumed by scripts, CI, or agents.
Use exit codes together with JSON payloads for reliable automation.

Canonical command reference in repository:

- `docs/spec/command-reference.md`
- <https://github.com/ota-run/ota/blob/main/docs/spec/command-reference.md>
