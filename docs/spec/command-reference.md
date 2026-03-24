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

# Ota Command Reference

This document describes the current shipped CLI surface.

Ota's canonical repo contract is `ota.yaml`. This reference covers the current repo-level CLI surface only.

For machine-readable command contracts, see [json-output-reference.md](json-output-reference.md).
For canonical exit-code behavior, see [exit-codes.md](exit-codes.md).
For service behavior across commands, see [service-behavior.md](service-behavior.md).
For platform shell behavior, see [shell-semantics.md](shell-semantics.md).

## Global

```bash
ota --help
ota --version
ota --debug <command>
ota --file /path/to/ota.yaml <command>
```

Repo commands that read an existing `ota.yaml` can also target a monorepo member with:

```bash
ota <command> --member <name> [PATH]
```

Ota currently ships these commands:

- `ota validate`
- `ota tasks`
- `ota run <task>`
- `ota doctor`
- `ota init`
- `ota check`
- `ota up`
- `ota detect`
- `ota workspace validate`
- `ota workspace tasks`
- `ota workspace run <task>`
- `ota workspace check`
- `ota workspace doctor`
- `ota workspace up`

The command set is intentionally small. V1 is about making the core readiness path trustworthy, inspectable, and stable on real repositories.

When a command accepts a `PATH`, it may be either:

- a direct path to `ota.yaml`
- a directory containing `ota.yaml`

For commands that read an existing contract, Ota now resolves in this order:

- `--file <path>`
- `OTA_FILE`
- explicit file `PATH`
- upward discovery from the provided directory `PATH`
- upward discovery from the current directory

When the discovered `ota.yaml` is a declared monorepo member contract, Ota now loads the merged
member contract automatically from that member path.

`ota detect` is different. Its `PATH` is a repo root to inspect.

## Current exit semantics

- `0`: success, ready state, or warning-only diagnosis
- `1`: invalid contract, blocking readiness issue, protected write failure, or general command failure
- `2`: CLI usage or argument parsing error
- `ota run`: preserves child task exit codes on task failure
- `ota up`: preserves service-start and setup task exit codes when those commands fail

The canonical registry is in [exit-codes.md](exit-codes.md).

## `--debug`

`--debug` emits command-phase tracing to stderr.

Current intent:

- help humans and agents understand which path or mode a command resolved
- keep normal stdout stable
- avoid persistent logs or verbose default output

## `ota validate`

Validate an Ota contract.

```bash
ota validate [PATH]
ota validate --json [PATH]
ota validate --member api [PATH]
```

Current behavior:

- resolves `ota.yaml` using `--file`, `OTA_FILE`, or upward discovery
- when `--member` is set, loads the root contract, merges the declared member override, and validates the merged contract
- when a root contract declares `workspace.type: monorepo`, `ota validate` also validates each declared merged member contract
- parses the contract
- applies semantic validation
- includes provider-specific target examples for remote target validation errors:
- `daytona`: `sandbox-dev`
- `ssh` / `tsh`: `user@host`
- `kubectl`: `pod/ota-dev`
- exits `0` on success and non-zero on failure

Text output:

- success: `VALID <path>`
- failure: validation or load error text

JSON output:

- success: `ok`, `path`
- failure: `ok`, `path`, and either `errors` or `error`

## `ota tasks`

List tasks from a validated contract.

```bash
ota tasks [PATH]
ota tasks --json [PATH]
ota tasks --member api [PATH]
ota tasks --member api --member web --json [PATH]
```

Current behavior:

- validates the contract first
- when a root contract declares `workspace.type: monorepo`, plain `ota tasks` lists root tasks and grouped summaries for each declared member
- when `--member` is set, lists tasks from the merged member contract
- repeated `--member` values list tasks for those members in the provided order
- prints tasks in deterministic order
- resolves the execution form for the current OS
- includes task metadata when present
- includes an `agent` summary when the contract declares one
- includes variant summaries when variants are declared

Text output:

- header: `TASKS <path>`
- each task may include `kind`, `os`, `category`, `depends_on`, `safe_for_agent`, and variant count
- each task includes a short execution preview

JSON output:

- success: `ok`, `path`, `tasks`
- `agent` is included when the contract declares agent guidance
- monorepo root summaries include grouped per-member results in `members`
- repeated `--member` values return grouped per-member results in `members`
- each task includes the resolved execution plus optional `selected_variant_os` and `variants`
- failure: `ok`, `path`, and either `errors` or `error`

## `ota run`

Run a validated task.

```bash
ota run <task> [PATH]
ota run <task> --member api [PATH]
ota run <task> --member api --member web [PATH]
ota run <task> --backend native [PATH]
ota run <task> --backend container --lifecycle ephemeral [PATH]
ota run <task> --backend remote [PATH]
```

Current behavior:

- validates the contract first
- when `--member` is set, resolves the merged member contract from the monorepo root
- repeated `--member` values run the task across those members in the provided order
- `--backend` and `--lifecycle` can override the contract for one invocation
- resolves task dependencies before execution
- resolves the best matching task variant for the current OS when variants are declared
- executes either `run` or `script`
- when `execution.preferred: container` is configured with `execution.backends.container.image`, runs tasks through the local `docker` CLI
- when container execution is configured, `execution.lifecycle: ephemeral` uses a fresh container and `execution.lifecycle: persistent` reuses a named container
- supports remote execution when `execution.backends.remote.provider` and `execution.backends.remote.target` are configured
- current shipped remote providers are `daytona`, `ssh`, `tsh`, and `kubectl`
- remote target guidance:
- `daytona`: `sandbox-dev`
- `ssh` / `tsh`: `user@host`
- `kubectl`: `pod/ota-dev`
- passes `execution.backends.remote.cwd` to the provider CLI when set
- runs in the effective target contract directory
- applies configured environment values
- prints task progress and advisory notes on stderr
- returns the child process exit code

Use this when the contract is already the source of truth and you want deterministic task execution.

## `ota doctor`

Diagnose repo readiness from a validated contract.

```bash
ota doctor [PATH]
ota doctor --json [PATH]
ota doctor --member api [PATH]
ota doctor --member api --member web --json [PATH]
```

Current behavior:

- validates the contract first
- when a root contract declares `workspace.type: monorepo`, plain `ota doctor` diagnoses the root contract and grouped summaries for each declared member
- when `--member` is set, diagnoses the merged member contract
- repeated `--member` values diagnose those members in the provided order
- checks configured env requirements
- checks preferred execution backend prerequisites such as `docker`, `daytona`, `ssh`, `tsh`, or `kubectl` when backend-backed execution is configured
- warns on suspicious remote target shape:
- `ssh` / `tsh` targets without `user@host`
- `kubectl` targets not starting with `pod/`
- checks runtime and tool presence on `PATH`
- runs declared service healthchecks
- warns when a required service has no healthcheck, because readiness cannot be verified
- honors `services.<name>.timeout` when a service healthcheck is declared
- warns when `execution.lifecycle: ephemeral` is declared and clarifies that current isolated execution applies to `ota run` and the `setup` phase of `ota up`, not the full repo lifecycle
- runs configured checks
- orders findings by severity
- includes an `agent` summary when the contract declares one
- prints the reason and next action for each finding

Text output:

- header: `DOCTOR <path>`
- status line: `READY` or `NOT READY`

JSON output:

- `ok`
- `path`
- `agent` when the contract declares agent guidance
- `findings`
- monorepo root summaries include grouped per-member results in `members`
- repeated `--member` values return grouped per-member results in `members`

Warnings can still produce `READY`. Errors produce `NOT READY`.

## `ota init`

Create a starter Ota contract for a repo that does not yet have one.

```bash
ota init [PATH]
ota init --dry-run [PATH]
ota init --json [PATH]
```

Current behavior:

- inspects the repo using the detection engine
- writes by default
- supports preview mode with `--dry-run`
- refuses to run when `ota.yaml` already exists
- can initialize both detected repos and blank repos
- keeps JSON output stable while using text output to guide review, write, and first validation steps
- in `detected` mode, write behavior is conservative and writes only the `high` confidence projection when it is sufficient
- in `detected` mode, write fails rather than silently writing an invalid contract when medium/low confidence fields would be required

Modes:

- `blank`: starter contract derived from minimal repo context
- `detected`: starter contract derived from detected repo signals

Text output:

- dry-run header: `INIT <path>`
- write success: `WROTE <path>`
- includes `Mode: blank` or `Mode: detected`
- includes a `Next:` line that tells the user how to review or validate the starter contract
- `blank` mode explicitly warns that the starter contract is minimal coverage only
- `detected` mode write output explicitly calls out the conservative write policy and any excluded fields
- includes inferred-field annotations with source and confidence

JSON output:

- `ok`
- `path`
- `written`
- `mode`
- `config`
- `inferred`
- failure responses can include `next` when Ota can point to one safe follow-up command

## `ota check`

Run configured checks from a validated contract.

```bash
ota check [PATH]
ota check --json [PATH]
ota check --member api [PATH]
ota check --member api --member web --json [PATH]
```

Current behavior:

- validates the contract first
- when a root contract declares `workspace.type: monorepo`, plain `ota check` runs root checks and grouped check summaries for each declared member
- when `--member` is set, runs checks from the merged member contract only
- repeated `--member` values run checks for those members in the provided order
- runs configured checks only
- does not perform runtime, tool, or env diagnosis
- does not execute tasks

Text output:

- header: `CHECK <path>`
- status line: `READY` or `NOT READY`

JSON output:

- `ok`
- `path`
- `findings`
- monorepo root summaries include grouped per-member results in `members`

## `ota up`

Prepare a repo for use with minimal prior knowledge.

```bash
ota up [PATH]
ota up --json [PATH]
ota up --backend container --lifecycle ephemeral [PATH]
ota up --member api [PATH]
ota up --member api --member web [PATH]
```

Current behavior:

- validates the contract first
- when a root contract declares `workspace.type: monorepo`, plain `ota up` prepares the root contract and grouped member summaries for each declared member
- when `--member` is set, prepares the merged member contract
- repeated `--member` values prepare those members in the provided order
- runs inherited or overridden setup in the effective member directory
- runs blocking precondition checks
- runs explicit `services.<name>.start` commands for required services before setup
- starts required services, and required-service dependencies, in declared dependency order
- verifies required service healthchecks before setup and treats them as readiness gates
- stops in the `services` phase when required-service readiness still fails
- runs the `setup` task if one exists, using the configured execution backend when present
- can override backend and lifecycle for the `setup` phase with `--backend` and `--lifecycle`
- the current `setup` backend path supports native, container, and the shipped remote providers
- prints a lifecycle note on stderr when the `setup` phase uses backend-backed execution
- re-runs readiness diagnosis
- still runs service start commands, service healthchecks, and diagnosis on the host today
- returns `READY` or `NOT READY`
- reports the phase where execution stopped: `preconditions`, `services`, `setup`, or `post-setup diagnosis`
- includes setup exit code details when the `setup` task fails
- includes service start exit code details when a required service start command fails

This is the onboarding command. It is intentionally narrower than a general-purpose environment orchestrator.

JSON output:

- `ok`
- `path`
- `status`
- `phase`
- `findings`
- `service` when a service-start failure occurs
- `task` when a task failure occurs
- `exit_code` when a child command failure occurs
- monorepo root and repeated `--member` summaries include grouped per-member results in `members`

## `ota clean`

Clean persistent execution state for a repo.

```bash
ota clean [PATH]
ota clean --member api [PATH]
ota clean --member api --member web [PATH]
```

Current behavior:

- validates the contract first
- when a root contract declares `workspace.type: monorepo`, plain `ota clean` reports the root cleanup result and grouped member cleanup results
- when `--member` is set, targets those merged member contracts in the provided order
- when the effective execution mode is `container` with `lifecycle: persistent`, removes the named persistent container for that repo
- remote backends do not currently define cleanup semantics; they report `NO CLEANUP NEEDED`
- reports `NO CLEANUP NEEDED` when there is no persistent container state to remove
- does not stop services or perform workspace-wide cleanup

## `ota detect`

Infer a starting contract from repo state.

```bash
ota detect --dry-run [PATH]
ota detect --json --dry-run [PATH]
ota detect --write [PATH]
ota detect --json --write [PATH]
ota detect --merge --dry-run [PATH]
ota detect --merge [PATH]
ota detect [PATH]
```

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
- `mvnw`
- `.mvn/wrapper/maven-wrapper.properties`
- `composer.json`
- `.ruby-version`
- `Gemfile`
- `global.json`
- `*.sln` / `*.csproj` / `*.fsproj`
- `mix.exs`
- `docker-compose.yml` / `docker-compose.yaml`
- `compose.yml` / `compose.yaml`

For Docker Compose service inference, Ota currently derives:

- `provider` at high confidence
- `start` / `stop` at medium confidence
- declared `healthcheck.test` at medium confidence

Dry-run behavior:

- `ota detect` is read-only by default
- prints a candidate `ota.yaml`
- prints per-field provenance
- prints per-field confidence
- when `ota.yaml` already exists, prints a non-destructive comparison preview for detected fields
- does not write anything

Current write behavior:

- `ota detect --write` writes using only `high` confidence fields
- validates the generated contract before writing
- refuses to overwrite an existing `ota.yaml`

Current merge-preview behavior:

- `ota detect --merge --dry-run` is a review-only mode
- it requires an existing `ota.yaml`
- it does not write
- it reuses the comparison preview instead of applying changes

Current merge-write behavior:

- `ota detect --merge` requires an existing `ota.yaml`
- it applies only `high` confidence missing fields
- it does not overwrite conflicting existing values
- it validates the merged contract before writing
- it is additive only in the current implementation
- on mixed repos, lower-confidence fields can still appear in `comparison` without being written
- if nothing eligible can be added, it returns success with `written: false` and leaves `ota.yaml` unchanged

Example dry-run annotations for detected Compose services:

```text
---
Annotations:
- services.db.provider: docker-compose <- from docker-compose.yml#services.db [high]
- services.db.start: docker compose up -d db <- from docker-compose.yml#services.db [medium]
- services.db.stop: docker compose stop db <- from docker-compose.yml#services.db [medium]
- services.db.healthcheck: pg_isready -h localhost -p 5432 <- from docker-compose.yml#services.db.healthcheck.test [medium]
```

Current precedence is conservative:

- higher confidence beats lower confidence
- when confidence is equal, more repo-specific runtime sources win before generic version-manager aggregation
- when confidence is equal for project names, `package.json` wins over conflicting Python or Go manifest names
- when confidence is equal for package-manager tools, `package.json#packageManager` wins over conflicting `.tool-versions` values
- when `package.json#packageManager` is absent, known repo-local Node package-manager markers such as workspace files and lockfiles can determine the tool and task command prefix conservatively
- `Pipfile` can contribute `python` runtime inference and `pipenv` tool inference conservatively
- `uv.lock` can contribute `uv` tool inference conservatively
- `requirements.txt` can contribute `pip` tool inference conservatively
- `setup.cfg` can contribute project name and `python` runtime inference conservatively
- for example, `.nvmrc`, `.node-version`, `.python-version`, `.java-version`, `.sdkmanrc`, `go.mod`, `rust-toolchain.toml`, and `rust-toolchain` win over conflicting `.tool-versions` runtime values

Write behavior:

- `ota detect --write` writes only `high` confidence fields
- validates the projected contract before writing
- refuses to overwrite an existing `ota.yaml`
- when `ota.yaml` already exists, points the user at `ota detect --merge --dry-run`
- fails if the high-confidence projection is not sufficient
- JSON failure responses can include `next` when Ota can point to one safe follow-up command

This is intentionally conservative. Review mode comes first, write mode second.

## `ota workspace validate`

Validate an Ota workspace contract.

```bash
ota workspace validate [PATH]
ota workspace validate --json [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- parses the workspace contract
- validates the workspace shape
- validates each present referenced repo contract through the workspace contract
- allows missing repo paths only when `repos.<name>.source` is declared

Text output:

- success: `VALID WORKSPACE <path>`
- failure: validation or load error text

JSON output:

- success: `ok`, `path`
- failure: `ok`, `path`, and either `errors` or `error`

## `ota workspace tasks`

List workspace repo tasks in dependency order.

```bash
ota workspace tasks [PATH]
ota workspace tasks --json [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace shape and present repo contracts
- preserves workspace dependency order in output
- lists task declarations for each acquired repo contract
- reports non-acquired repos with `acquired: false` and empty task lists
- does not execute tasks

Text output:

- header: `WORKSPACE TASKS <path>`
- each repo includes required/optional status, acquisition status, dependency list, and task summaries

JSON output:

- `ok`
- `path`
- `repos`
- each repo includes: `name`, `path`, `contract_path`, `required`, `acquired`, `depends_on`, `tasks`

## `ota workspace run`

Run one task across workspace repos in dependency order.

```bash
ota workspace run <task> [PATH]
ota workspace run <task> --json [PATH]
ota workspace run <task> --jobs 4 [PATH]
ota workspace run <task> --stream [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure and repo contracts
- acquires missing repos declared with `repos.<name>.source` before execution
- executes the task for each repo in workspace dependency order
- can run independent repos concurrently when `--jobs` is greater than `1`
- blocks downstream repos when a dependency repo did not complete successfully
- captures per-repo stdout/stderr in default mode
- `--stream` opts into raw child output (text only, currently requires `--jobs 1`)
- optional repo task failures do not fail the overall workspace status

Text output:

- header: `WORKSPACE RUN <task> <path>`
- status line: `READY` or `NOT READY`
- per-repo status includes `required/optional`, task name, findings, and optional exit details

JSON output:

- `ok`
- `path`
- `task`
- `repos`
- each repo includes: `name`, `path`, `contract_path`, `required`, `ok`, `status`, `task`, `findings`, and optional `exit_code`/`stdout`/`stderr`

## `ota workspace check`

Run configured checks across workspace repos in dependency order.

```bash
ota workspace check [PATH]
ota workspace check --json [PATH]
ota workspace check --jobs 4 [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure and referenced repo contracts
- evaluates repo checks in workspace dependency order
- can check independent repos concurrently when `--jobs` is greater than `1`
- preserves deterministic repo ordering in text and JSON output even when checks run concurrently
- downgrades findings for optional repos to warnings

Text output:

- header: `WORKSPACE CHECK <path>`
- status line: `READY` or `NOT READY`
- each repo includes required/optional status, contract path, and findings

JSON output:

- `ok`
- `path`
- `repos`

## `ota workspace doctor`

Diagnose workspace repo readiness from an Ota workspace contract.

```bash
ota workspace doctor [PATH]
ota workspace doctor --json [PATH]
ota workspace doctor --jobs 4 [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure
- evaluates repos in dependency order
- can diagnose independent repos concurrently when `--jobs` is greater than `1`
- preserves deterministic repo ordering in text and JSON output even when diagnosis runs concurrently
- evaluates each referenced repo through its own `ota.yaml`
- reports missing-but-acquirable repos as not yet acquired
- keeps workspace logic above repo diagnosis instead of duplicating it
- downgrades findings for optional repos to warnings
- rejects required repos that depend on optional repos

Text output:

- header: `WORKSPACE DOCTOR <path>`
- status line: `READY` or `NOT READY`
- each repo includes required/optional status, contract path, and findings

JSON output:

- `ok`
- `path`
- `repos`

Current non-goals:

- passing a repo URL directly on the CLI without a workspace contract

## `ota workspace up`

Prepare every repo in an Ota workspace contract.

```bash
ota workspace up [PATH]
ota workspace up --json [PATH]
ota workspace up --jobs 4 [PATH]
ota workspace up --stream [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure
- clones missing repos declared with `repos.<name>.source` before repo-level prepare
- runs the existing repo-level `up` flow for each referenced repo
- can prepare independent repos concurrently when `--jobs` is greater than `1`
- respects declared workspace repo dependency order
- blocks downstream repos when a dependency does not become ready
- aggregates per-repo status, phase, findings, and exit details
- captures repo child stdout and stderr per repo so text and JSON output remain deterministic
- emits live repo progress on stderr in text mode so users can see queued/running/completed state while buffered output is still being collected
- optional repo failures do not fail the overall workspace result
- defaults to sequential execution because `--jobs` defaults to `1`
- `--stream` opts into raw live child process output instead of buffered per-repo output
- `--stream` is text-only and currently requires `--jobs 1`
- does not pull or update repos that already exist locally

Text output:

- header: `WORKSPACE UP <path>`
- status line: `READY` or `NOT READY`
- each repo includes required/optional status, phase, findings, exit details, and captured stdout/stderr when present

JSON output:

- `ok`
- `path`
- `repos`

Current non-goals:

- passing a repo URL directly on the CLI without a workspace contract
- host or workstation provisioning beyond workspace bootstrap plus repo readiness
- GitHub API integration or non-git acquisition modes
