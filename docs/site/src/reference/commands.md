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
- `--debug`: command-phase tracing to stderr for multi-step or diagnosis-heavy commands.
- `--json`: stable machine output; not affected by concise/verbose text shaping.

Progress behavior:

- quiet blocking commands show a delayed spinner in interactive terminals
- `ota doctor` and `ota check` keep their own progress handling
- `ota run` and `ota up` keep streaming/progress-focused behavior instead of the shared spinner
- `ota workspace doctor` uses the shared spinner
- `ota workspace doctor --json` still uses the shared spinner on stderr in interactive terminals, while stdout remains valid JSON
- `ota workspace list --json` also uses the shared spinner on stderr in interactive terminals, while stdout remains valid JSON
- `ota workspace validate`, `ota workspace tasks`, `ota workspace list`, `ota workspace detect`, and `ota workspace init` use the shared spinner when they are waiting on work
- `ota workspace list` shows execution metadata when the repo contract declares it
- successful interactive commands may print a best-effort update notice when a newer release exists, and the notice points to `ota self-update` or `ota upgrade`

When to use debug:

- `ota up`, `ota run <task>`, `ota workspace up`, and `ota workspace run <task>` are the best candidates for debug traces because they orchestrate multiple steps or multiple repos
- `ota doctor`, `ota detect`, `ota diff`, and `ota explain` also benefit from debug traces when you are diagnosing a failure or reviewing provenance
- `ota validate`, `ota tasks`, `ota workspace validate`, `ota workspace tasks`, and `ota workspace list` should usually remain summary-only unless you are actively debugging

Hosted validation:

- use `ota validate --json` and `ota doctor --json` for repo gating
- use `ota workspace validate --json`, `ota workspace doctor --json`, and `ota workspace explain --json` for workspace gating and remediation planning
- use `ota workspace tasks --json` and `ota workspace list --json` for workspace inventory, task availability, and preflight readiness summaries
- do not mutate contracts during hosted validation

Execution modes:

- `native` runs tasks on the host machine and is useful when you want to debug against the real local environment
- `container` runs tasks in an OCI-compatible container using the image declared in `ota.yaml` and is useful when you want a fixed toolchain and CI-like behavior
- `remote` runs tasks on a separate machine or workspace through a provider and is useful when work must happen off-host
- use `ota run <task> --backend native|container|remote` and `ota up --backend native|container|remote` to override the contract for one invocation
- use `ota run <task> --lifecycle persistent|ephemeral` and `ota up --lifecycle persistent|ephemeral` to override container reuse for one invocation
- container execution requires a valid `execution.backends.container.image` and at least one supported container engine CLI such as Docker or Podman
- Ota provisions declared repo services through `ota up`, but it does not replace the OS package manager or language installer on the host

## Start with this flow

1. `ota doctor` to understand readiness blockers.
1. `ota up` to make the repo runnable.
1. `ota run <task>` for day-to-day task execution.
1. `ota diff <base> <target>` to compare contract impact before writing changes.
1. `ota explain` to turn findings into an ordered remediation plan.
1. `ota detect --dry-run` before writing any new contract.
1. `ota workspace explain` when you want workspace-level remediation ordering.

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

JSON output for `ota validate --json` includes `summary.error_count` so hosted gates can read a
single machine-facing error count before parsing `errors`.

### `ota doctor`

When to use:

- first command in a new repo or broken environment

Why:

- shows actionable blockers and warnings with explicit next steps
- still gives a useful repo/host diagnosis even before `ota.yaml` exists
- leads with the highest-priority blocker first so the next action is obvious
- reports contract drift as warnings when repo signals no longer match the declared contract
- tags drift warnings with repo-contract ownership and provenance so consumers can separate stale contract truth from host or service failures
- text output includes repo verdict and agent verdict before per-finding details
- `--concise` keeps severity/summary/next action and omits `Why` detail
- also surfaces inert top-level `extensions` entries so adapter metadata is visible without execution

Use-case:

- teammate cannot run a repo; doctor reports missing runtime/tool/env quickly, or they have not created `ota.yaml` yet and need the best next step

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

### `ota diff`

When to use:

- when you want to review contract impact before writing changes or compare two contract states in CI

Why:

- shows semantic additions, removals, and changes in deterministic order
- stays read-only and compares structured contract state rather than raw YAML text

Use-case:

- compare a branch contract against the main branch contract before merging

```bash
ota diff ./before/ota.yaml ./after/ota.yaml
ota diff --json ./before/ota.yaml ./after/ota.yaml
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota diff --json ./before/ota.yaml ./after/ota.yaml | tee .ota-diff.json
```

### `ota explain`

When to use:

- when you want findings turned into an ordered remediation plan

Why:

- stays read-only and deterministic
- turns doctor output into a concrete next-fix sequence

Use-case:

- copy a remediation plan into a ticket or hand it to an agent

```bash
ota explain ./repo
ota explain --json ./repo
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota explain --json ./repo | tee .ota-explain.json
```

### `ota annotations`

When to use:

- when CI needs annotations instead of raw JSON
- when hosted validation should surface blocker and warning lines in the job log
- when you want a portable wrapper around `ota doctor --json` or `ota workspace doctor --json`

Why:

- keeps Ota’s JSON contract as the source of truth
- lets GitHub and non-GitHub CI render the same findings differently without changing Ota itself
- keeps repo-local annotation adapters first-class and deterministic

Use-case:

- a PR gate runs `ota doctor --json | ota annotations --mode doctor --format github --input -` so
  reviewers see the blocker directly in the checks UI

```bash
ota annotations --mode doctor --format github --input ./doctor.json
ota annotations --mode workspace-doctor --format plain --input ./workspace-doctor.json
ota doctor --json | ota annotations --mode doctor --format github --input -
```

Current behavior:

- reads Ota JSON from a file or from stdin when `--input -` is used
- emits one primary blocker line when `summary.primary_blocker` is present
- emits one line per finding
- maps `severity: error` to `::error` or `ERROR` and all other severities to
  `::warning` or `WARNING`
- scopes workspace findings with the repo name and path so annotations stay actionable
- serves as the canonical binary entrypoint for repo-local and CI annotation adapters

Text output:

- `NOTICE: ...` for primary blockers
- `ERROR: ...` and `WARNING: ...` for findings

JSON output:

- none; this is a rendering command, not a contract reader

### `ota extensions`

When to use:

- when you want to inspect staged extension descriptors or explicitly run one allowlisted
  descriptor

Why:

- shows typed adapter metadata in one command
- keeps extension discovery contract-driven and deterministic
- `ota extensions --run <name>` executes one explicit `checker` descriptor with
  `api_version: 1`
- `ota extensions --publish <name>` executes one explicit `publisher` descriptor with
  `api_version: 1`
- makes external adapter use cases visible, such as publishers, compliance scanners, and
  codegen helpers

Use-case:

- editor or CI tooling wants to confirm what extension descriptors a repo declares, or a release
  operator wants to run one named publisher against an artifact endpoint

```bash
ota extensions
ota extensions --json
ota extensions --run demo-check
ota extensions --publish release-upload
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota extensions --json | tee .ota-extensions.json
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

Receipt:

- prints a summary in text output, emits an execution receipt when `--receipt` is set, and JSON output
- the JSON payload includes a top-level summary mirroring the receipt roll-up

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota up
ota run test
```

### `ota self-update`

When to use:

- when Ota is already installed and you want to update it in place

Why:

- downloads and installs the newest release binary or a pinned release
- alias: `ota upgrade`

Use-case:

- update the current machine after seeing the update notice from another Ota command

```bash
ota self-update
ota self-update --version v0.1.3
ota self-update --channel stable
ota upgrade
ota upgrade --version v0.1.3
ota upgrade --channel stable
```

Current behavior:

- `--version` pins a specific release
- `--channel` currently accepts `stable` and `latest`
- `stable` resolves the latest stable release tag
- `latest` resolves the newest release entry, including prereleases if present
- `--version` overrides the channel when both are set
- when the chosen target matches the installed binary, the command exits successfully and prints the up-to-date banner instead of reinstalling
- success runs the installer for the chosen release target

### `ota run <task>`

When to use:

- day-to-day execution after repo readiness is established

Why:

- runs named tasks with dependency ordering and stable behavior

Use-case:

- `ota run test`, `ota run dev`, `ota run lint` in CI or local loops

```bash
ota run test
ota run test --base-url http://localhost:8080
ota run version:bump --version 0.2.0
```

Task inputs are declared in `tasks.<name>.inputs` and are passed as `--kebab-case value` flags.
They are also exposed to the task as `OTA_INPUT_<NAME>` env variables.
`default` values apply when omitted, `required: true` makes an input mandatory unless a default exists,
and `allowed` limits accepted values.
If every declared input has a default, you can omit all input flags.

Example:

```yaml
tasks:
  api-automation-tests:
    inputs:
      base_url:
        default: http://localhost:8080
      mode:
        default: standard
        allowed:
          - standard
          - contract-drift
  version:bump:
    inputs:
      version:
        required: true
```

```bash
ota run api-automation-tests
ota run api-automation-tests --base-url http://localhost:8080 --mode contract-drift
ota run version:bump --version 0.2.0
```

Receipt:

- prints a summary in text output, and emits an execution receipt on stderr after task output when `--receipt` is set
- the receipt includes backend, lifecycle, remote target when set, env sources, and step summary data

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
- read the task `description` and optional `notes` before running a task
- use `--use` when you want the command line form plus the task purpose

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
- text output includes repo verdict and agent verdict before per-finding details

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
- `ota init --bootstrap` writes the fuller detected starter contract when the detector has enough confidence
- if no stronger project identity is inferred, `ota init --bootstrap` can fall back to the repo directory name for `project.name`
- plain `ota init` still excludes low-confidence fields

Use-case:

- bootstrap Ota adoption for an existing project

```bash
ota init
ota init --bootstrap
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
ota init --bootstrap
ota validate
```

### `ota agents`

When to use:

- you want a repo-local `AGENTS.md` generated from `ota.yaml`

Why:

- exports or syncs agent guidance from the contract instead of hand-maintaining a second file
- stays deterministic and reviewable
- falls back to a lightweight scaffold when the contract does not declare an `agent` block yet
- preserves any existing `AGENTS.md` content and appends or refreshes an Ota-managed block instead of overwriting it
- skips the write when the existing file already contains the generated content
- shows a `Managed block:` label in text output so the Ota-owned section is explicit

Use-case:

- generate a repo-local `AGENTS.md` before onboarding humans or agents

```bash
ota agents
ota agents --write
ota agents --json
ota agents --write --output AGENTS.md
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota agents --write
ota validate
ota doctor
```

### `ota detect`

When to use:

- you want contract inference from existing repo signals

Why:

- speeds adoption while preserving trust with confidence/provenance model
- `--write` stays conservative and only writes high-confidence fields
- there is no standalone `ota drift` command yet; use `ota detect --merge --dry-run` for contract-vs-repo drift review, including stale contract fields, and `ota doctor` for operator-facing trust/readiness drift

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
- `ota workspace init` writes `ota.workspace.yaml` by default
- `ota workspace init --bootstrap` can auto-provision missing repo contracts from detected repo signals before writing `ota.workspace.yaml`
- `--write` remains a compatibility alias for the write path
- when no repo contracts are found, points to `ota init <repo-path>`, `ota detect --dry-run <repo-path>`, and `ota workspace detect --write` or `ota workspace init` after repo contracts exist

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
- when no repo contracts are found, points to `ota init <repo-path>`, `ota detect --dry-run <repo-path>`, and `ota workspace detect --write` or `ota workspace init` after repo contracts exist

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

JSON output for `ota workspace validate --json` includes `summary.error_count` so hosted gates
can read a single machine-facing error count before parsing `errors`.

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

`ota workspace tasks --json` includes a top-level summary with repo and task counts so CI and
editor tooling can read the inventory at a glance.

### `ota workspace list`

When to use:

- inventory workspace repos, contract presence, and lightweight readiness status without running workspace doctor

Why:

- gives a fast view of required/optional repos, acquisition state, lightweight readiness, and missing contracts
- shows acquisition in the repo summary and readiness on a dedicated `Status:` line
- shows execution metadata in a compact `Execution:` block when the repo contract declares it

```bash
ota workspace list
ota workspace list --status ready
ota workspace list --status not-ready
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
- text output includes a summary roll-up with repo counts and finding totals before per-repo details
- summary roll-up also includes repo verdict and agent verdict before the counts
- `--concise` keeps repo status + finding summary/next action and omits per-repo path/contract and `Why` detail
- `--stream` is text-only and emits repo completion updates while the final report is being built

```bash
ota workspace doctor
ota workspace doctor --json
ota workspace doctor --repo api
ota workspace doctor --status not-ready
ota workspace doctor --severity error
ota workspace doctor --stream
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace doctor --json > .ota-workspace-doctor.json
```

The JSON payload includes a top-level `summary` with repo and finding counts so CI and editor
consumers can read the roll-up directly.

### `ota workspace explain`

When to use:

- turn workspace readiness findings into an ordered remediation plan

Why:

- stays read-only and deterministic
- shows one plan per repo so the fix path stays local and actionable

Use-case:

- copy the workspace remediation plan into a ticket or agent task list

```bash
ota workspace explain
ota workspace explain --json
ota workspace explain --repo api
```

Script example:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota workspace explain --json > .ota-workspace-explain.json
```

The JSON payload includes a top-level `summary` with repo, finding, and step counts so hosted
validation and editors can consume the plan without re-deriving totals from nested steps.

### `ota workspace check`

When to use:

- checks-only pass across workspace repos

Why:

- lightweight CI signal across multiple repositories
- text output includes a summary roll-up with repo counts and finding totals at the bottom
- summary roll-up also includes repo verdict and agent verdict before the counts
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

The JSON payload includes a top-level `summary` that mirrors the workspace doctor roll-up so CI
and editor tooling can read repo and finding counts directly.

### `ota workspace run <task>`

When to use:

- run same task across multiple repos with dependency ordering

Why:

- single command for coordinated multi-repo execution

Receipt:

- prints a summary in text output and emits an execution receipt when `--receipt` is set
- the canonical receipt and summary layout lives in [`docs/spec/command-reference.md`](../../../spec/command-reference.md) and [`docs/spec/output-style.md`](../../../spec/output-style.md)

`ota workspace run --json` includes a top-level summary and receipt so hosted validation and
automation can read the roll-up without descending into the receipt object first.

```bash
ota workspace run test
ota workspace run test --json
ota workspace run test --base-url http://localhost:8080
ota workspace run version:bump --version 0.2.0
```

Task inputs are declared in `tasks.<name>.inputs` and are passed as `--kebab-case value` flags.
They are also exposed to each repo task as `OTA_INPUT_<NAME>` env variables.
`default` values apply when omitted, `required: true` makes an input mandatory unless a default exists,
and `allowed` limits accepted values.
If every declared input has a default, you can omit all input flags.

Example:

```yaml
tasks:
  api-automation-tests:
    inputs:
      base_url:
        default: http://localhost:8080
      mode:
        default: standard
        allowed:
          - standard
          - contract-drift
  version:bump:
    inputs:
      version:
        required: true
```

```bash
ota workspace run api-automation-tests
ota workspace run api-automation-tests --base-url http://localhost:8080 --mode contract-drift
ota workspace run version:bump --version 0.2.0
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

Receipt:

- prints a summary in text output and emits an execution receipt when `--receipt` is set
- the canonical receipt and summary layout lives in [`docs/spec/command-reference.md`](../../../spec/command-reference.md) and [`docs/spec/output-style.md`](../../../spec/output-style.md)

```bash
ota workspace up
ota workspace up --json
ota workspace up --quiet
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
