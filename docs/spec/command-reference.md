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

# ota Command Reference

This document describes the current shipped CLI surface.

ota's canonical repo contract is `ota.yaml`. This reference covers the current repo-level CLI surface only.

For machine-readable command contracts, see [json-output-reference.md](json-output-reference.md).
For canonical exit-code behavior, see [exit-codes.md](exit-codes.md).
For service behavior across commands, see [service-behavior.md](service-behavior.md).
For platform shell behavior, see [shell-semantics.md](shell-semantics.md).
For text rendering and plain-mode behavior, see [output-style.md](output-style.md).
For visual identity tokens and CLI/docs branding, see [brand-style.md](brand-style.md).
For docs clarity rules and command-UX wording standards, see [docs-clarity-spec.md](docs-clarity-spec.md).
For compatibility boundaries in the active version, see [compatibility-surface.md](compatibility-surface.md).
For extension execution staging, see [extension-execution-boundary.md](extension-execution-boundary.md).
For provider-neutral CI shells and receipt/annotation wiring, see [ci-pipeline-workflow.md](ci-pipeline-workflow.md).
For hosted validation and PR-gating guidance, see [hosted-validation-workflow.md](hosted-validation-workflow.md).
For the official GitHub Actions wrapper, see [github-action-workflow.md](github-action-workflow.md).

Doctor first, contract second.

## Recommended onboarding flow

1. `ota doctor`
2. `ota explain`
3. if the repo does not yet have `ota.yaml`, preview with `ota init --dry-run` or `ota detect --dry-run .`
4. choose an explicit first write with `ota init` or `ota detect --write .`
5. if the repo already has `ota.yaml`, review changes with `ota detect --merge --dry-run .` or `ota detect --rewrite --dry-run .`
6. `ota up`

## Global

```bash
ota --help
ota --version
ota --debug <command>
ota --plain <command>
ota --concise <command>
ota --verbose <command>
ota --file /path/to/ota.yaml <command>
```

Repo commands that read an existing `ota.yaml` can also target a monorepo member with:

```bash
ota <command> --member <name> [PATH]
```

ota currently ships these commands:

- `ota doctor`
- `ota explain`
- `ota up`
- `ota run <task>`
- `ota init`
- `ota env`
- `ota execution plan`
- `ota detect`
- `ota validate`
- `ota tasks`
- `ota services`
- `ota diff`
- `ota check`
- `ota annotations`
- `ota agents`
- `ota clean`
- `ota extensions`
- `ota policy`
- `ota policy init`
- `ota policy review`
- `ota uninstall`
- `ota self-update` / `ota upgrade`
- `ota workspace init`
- `ota workspace detect`
- `ota workspace validate`
- `ota workspace tasks`
- `ota workspace list`
- `ota workspace execution plan`
- `ota workspace run <task>`
- `ota workspace check`
- `ota workspace doctor`
- `ota workspace explain`
- `ota workspace up`
- `ota workspace refresh`
- `ota workspace diff`
- `ota workspace status`
- `ota workspace receipt`

Start here:

```bash
ota doctor
ota explain
ota detect --dry-run .
ota init --dry-run
ota up
ota run ci
```

Workspace:

```bash
ota workspace doctor .
ota workspace up
```

The command set is intentionally small. V1 is about making the core readiness path trustworthy, inspectable, and stable on real repositories.

When a command accepts a `PATH`, it may be either:

- a direct path to `ota.yaml`
- a directory containing `ota.yaml`

For commands that read an existing contract, ota now resolves in this order:

- `--file <path>`
- `OTA_FILE`
- explicit file `PATH`
- an explicitly supplied directory `PATH` is treated as the contract boundary
- upward discovery from the current directory when no `PATH` is supplied

When the discovered `ota.yaml` is a declared monorepo member contract, ota now loads the merged
member contract automatically from that member path.

`ota detect` is different. Its `PATH` is a repo root to inspect.

Global output modifiers:

- `--concise`: reduce high-noise text output while preserving decisions and actions
- `--verbose`: preserve full explanatory text output
- `--json`: unaffected by `--concise`/`--verbose`
- `--debug` emits command-phase tracing to stderr

Current progress behavior:

- quiet blocking commands show a delayed spinner in interactive terminals
- `ota doctor` and `ota check` keep their own check/progress handling
- `ota run` keeps streaming/progress-focused behavior instead of the shared spinner
- `ota up` uses the shared spinner by default; `ota up --stream` opts into raw live provisioning, service-start, and setup output
- `ota workspace doctor` uses the shared spinner
- `ota workspace status` uses the shared spinner
- `ota workspace doctor --json` still uses the shared spinner on stderr in interactive terminals, while stdout remains valid JSON
- `ota workspace list --json` also uses the shared spinner on stderr in interactive terminals, while stdout remains valid JSON
- `ota workspace validate`, `ota workspace tasks`, `ota workspace list`, `ota workspace detect`, and `ota workspace init` use the shared spinner when they are waiting on work
- successful interactive commands may print a best-effort update notice when a newer release exists, and the notice says `A newer \`ota\` release is available: vX.Y.Z` and points to `ota self-update` or `ota upgrade`

Hosted validation guidance:

- use `ota validate --json` and `ota doctor --json` for repo gating
- use `ota workspace validate --json`, `ota workspace doctor --json`, and `ota workspace explain --json` for workspace gating and remediation planning
- use `ota workspace tasks --json` and `ota workspace list --json` for workspace inventory, task availability, and preflight readiness summaries
- do not mutate contracts during hosted validation

## Current exit semantics

- `0`: success, ready state, or warning-only diagnosis
- `1`: invalid contract, blocking readiness issue, protected write failure, or general command failure
- `2`: CLI usage or argument parsing error
- `ota run`: preserves child task exit codes on task failure
- `ota up`: preserves provisioning, service-start, and setup child exit codes when those commands fail

The canonical registry is in [exit-codes.md](exit-codes.md).

## `--debug`

`--debug` emits command-phase tracing to stderr.

Current intent:

- help humans and agents understand which path or mode a command resolved
- keep normal stdout stable
- avoid persistent trace output or verbose default output
- use the trace channel for multi-step commands like `ota up`, `ota run`, `ota workspace up`,
  `ota workspace refresh`, `ota workspace diff`, `ota workspace status`, `ota workspace run`, `ota doctor`, `ota detect`, `ota diff`, and
  `ota explain`

## `ota validate`

Validate an ota contract.

```bash
ota validate [PATH]
ota validate --json [PATH]
ota validate --member api [PATH]
```

Current behavior:

- resolves `ota.yaml` using `--file`, `OTA_FILE`, or an explicit directory boundary
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

- success: `VALID <path>` followed by next-step guidance into `ota doctor` and `ota tasks --use`
- failure: validation or load error text

JSON output:

- success: `ok`, `path`, `summary.error_count`
- failure: `ok`, `path`, `summary.error_count`, and either `errors` or `error`

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
- includes task `env` and `inputs` when present
- includes task `description` and optional `notes` when present, where `notes` carries purpose and
  extra guidance
- includes an `agent` summary when the contract declares one
- includes variant summaries when variants are declared
- `--use` keeps the usage line but also shows `description` and `notes` when present

Text output:

- header: `TASKS <path>`
- each task may include `kind`, `os`, `category`, `depends_on`, `safe_for_agent`, and variant count
- each task may include `env` and `inputs`
- each task may include `Description` and `Notes`, where `Notes` can describe purpose and usage
- each task includes a short execution preview

JSON output:

- success: `ok`, `path`, `tasks`
- `agent` is included when the contract declares agent guidance
- monorepo root summaries include grouped per-member results in `members`
- repeated `--member` values return grouped per-member results in `members`
- each task includes the resolved execution plus optional `selected_variant_os` and `variants`
- failure: `ok`, `path`, and either `errors` or `error`

## `ota services`

List declared services from a validated contract.

```bash
ota services [PATH]
ota services --json [PATH]
ota services --member api [PATH]
ota services --member api --member web --json [PATH]
```

Current behavior:

- validates the contract first
- when a root contract declares `workspace.type: monorepo`, plain `ota services` lists root services and grouped summaries for each declared member
- when `--member` is set, lists services from the merged member contract
- repeated `--member` values list services for those members in the provided order
- prints declared service fields in deterministic order
- services are not direct task entrypoints; they are managed by `ota doctor` and `ota up`

Text output:

- header: `SERVICES <path>`
- each service may include `required`, `provider`, `depends_on`, `start`, `stop`, `healthcheck`, `timeout`, and a management note
- when no services are declared, the text output says so explicitly and points users back to
  `ota doctor` or contract authoring instead of ending empty

JSON output:

- success: `ok`, `path`, `services`
- monorepo root summaries include grouped per-member results in `members`
- repeated `--member` values return grouped per-member results in `members`
- failure: `ok`, `path`, and either `errors` or `error`

## `ota env`

Inspect resolved environment requirements from a validated contract.

```bash
ota env [PATH]
ota env --json [PATH]
ota env --task test [PATH]
ota env --member api --task test [PATH]
```

Current behavior:

- validates the contract first
- when `--member` is set, inspects the merged member contract
- when `--task` is set, includes task-scoped env alongside the contract env view
- resolves values in the same precedence order as task execution
- reports declared env source status alongside the env-variable view
- shows the winning source for each contract env entry
- reports missing required env and invalid allowed values
- stays read-only

Text output:

- header: `ENV <path>`
- includes a readiness status line, a short overview, a `Declared env sources` section when sources exist, and separate `Contract env` / `Task env` sections when task-specific env is present
- each env entry may include `kind`, `required`, `value`, `source`, `status`, `allowed`, `default`, and `Next`
- each declared source may include `kind`, `path`, `must_exist`, `status`, `detail`, and `Next`
- missing or invalid contract env entries point to a specific fix rather than guessing

Example:

```text
ENV ./ota.yaml

Ready: yes

Declared env sources
- dotenv .env.local status=loaded
- dotenv .env must_exist=true status=loaded

Contract env
- DISCORD_TOKEN required=true value=*** source=dotenv:.env status=resolved
- DOCS_SITE_BASE_URL required=true value=https://docs.internal.example source=org policy status=resolved
- RELEASE_CHANNEL required=false value=stable source=default status=resolved allowed=[stable, canary]

Task env
- CI value=true source=task status=task
```

JSON output:

- success: `ok`, `path`, `summary`, `sources`, `env`
- success with task scope also includes `task`
- `summary` includes contract, declared-source, task, resolved, missing, and invalid counts
- failure: `ok`, `path`, `task` when relevant, and `error`

## `ota execution plan`

Inspect the resolved execution context without provisioning, starting services, or running tasks.

```bash
ota execution plan [PATH]
ota execution plan --json [PATH]
ota execution plan --mode container --ephemeral [PATH]
ota execution plan --member api [PATH]
```

Current behavior:

- validates the contract first
- when `--member` is set, inspects the merged member contract
- reuses the same backend and lifecycle resolution path as `ota run` and `ota up`
- reports the resolved backend, lifecycle, image, container-engine selection, and target strategy
- fails with the same backend-configuration errors as runtime execution when the selected native/container/remote path is not actually runnable from the current contract
- shows the deterministic per-run target name for ephemeral containers, but does not create that target
- stays read-only

Text output:

- header: `EXECUTION PLAN <path>`
- status line: `RESOLVED`
- `Resolved` section with selected backend, lifecycle, image, engine candidates, target, and target strategy
- `Contract` section with the same compact contract identity used by receipts
- `Execution` section when the contract declares execution intent
- `Overrides` section when `--mode`, `--lifecycle`, or `--ephemeral` changed the resolved result

JSON output:

- success: `ok`, `path`, `contract`, `member` when relevant, `contract_identity`, `declared_execution`, `resolved`, and `overrides`
- failure: `ok`, `path`, `member` when relevant, and either `errors` or `error`

## `ota diff`

Compare two ota contracts semantically.

```bash
ota diff ./before/ota.yaml ./after/ota.yaml
ota diff ./repo-a ./repo-b
ota diff --json ./before/ota.yaml ./after/ota.yaml
```

Current behavior:

- compares two repo or workspace contracts as structured YAML
- reports added, missing-in-target, and changed fields in deterministic order
- remains read-only
- exits `0` when the comparison succeeds, even if differences exist
- surfaces load and parse errors clearly

Text output:

- header: `DIFF <base> -> <target>`
- `MATCH` or `DIFFERENT`
- readiness impact summary
- grouped added, missing-in-target, and changed paths
- policy-section changes may include provenance labels
- summary counts at the end

JSON output:

- success: `ok`, `base`, `target`, `summary`, `changes`
- policy-section changes may include `provenance`
- failure: `ok`, `base`, `target`, and `error`

Use this when you want to compare contract states before writing changes or to review the impact of a proposed edit in CI.

## `ota explain`

Explain readiness findings as an ordered remediation plan.

```bash
ota explain ./repo
ota explain --json ./repo
ota explain --member api ./repo
```

Current behavior:

- diagnoses the contract first
- turns each finding into an ordered remediation step
- stays read-only and deterministic
- prints a compact overview with step counts at the end

Text output:

- `Plan` section with ordered remediation steps
- stable finding code for each step
- `Why` and `Next` lines for each step
- `Provenance` lines when ota can trace the diagnosis source for that step
- `Overview` counts at the end

JSON output:

- success: `ok`, `path`, `summary`, `steps`
- each step includes `order`, `code`, `severity`, `summary`, `why`, and `next`
- steps may also include `provenance` and `provenance_key`
- failure: `ok`, `path`, and `error`

## `ota annotations`

Render ota doctor findings as CI annotations or provider-neutral log lines.

```bash
ota annotations --mode doctor --format github --input ./doctor.json
ota annotations --mode workspace-doctor --format plain --input ./workspace-doctor.json
ota annotations --mode doctor --format markdown --input ./doctor.json
ota annotations --mode receipt-diff --format markdown --input ./receipt-diff.json
ota doctor --json | ota annotations --mode doctor --format github --input -
```

Current behavior:

- reads ota JSON from a file or from stdin when `--input -` is used
- emits one primary blocker line when `summary.primary_blocker` is present
- does not repeat that same primary blocker as a second finding line
- emits one line per remaining finding
- ignores `finding_groups` and stays one-annotation-per-finding by default
- maps `severity: error` to `::error` or `ERROR` and all other severities to
  `::warning` or `WARNING`
- `--format markdown` renders a compact summary block for step summaries or PR comments with status,
  counts, the primary blocker when present, and remaining findings
- `--mode receipt-diff` expects `ota receipt --json --baseline ...` diff output and currently
  supports `--format markdown` only
- scopes workspace findings with the repo name and path so annotations stay actionable
- labels additive `Provenance:` and `Next:` segments when those fields are present in the input JSON
- serves as the canonical binary entrypoint for repo-local and CI annotation adapters

Text output:

- `ERROR: ...`, `WARNING: ...`, or `NOTICE: ...` for primary blockers depending on their severity
- `ERROR: ...` and `WARNING: ...` for findings
- markdown output uses `## <title>`, `Status`, `Counts`, optional `Primary blocker`, and `Findings`
  sections instead of one-line annotations
- receipt-diff markdown output uses `Baseline source`, `Compare`, `Drift`, `Counts`, optional
  `Gate`, optional `Primary blocker`, and compact `Introduced` / `Resolved` sections

JSON output:

- none; this is a rendering command, not a contract reader

## `ota extensions`

List staged extension descriptors declared in `ota.yaml`.

```bash
ota extensions [PATH]
ota extensions --json [PATH]
ota extensions --member api [PATH]
ota extensions --run demo-check [PATH]
ota extensions --publish release-upload [PATH]
```

Current behavior:

- validates the contract first
- when a root contract declares `workspace.type: monorepo`, plain `ota extensions` lists root
  descriptors and grouped member results for each declared member
- when `--member` is set, lists descriptors from the merged member contract
- repeated `--member` values list descriptors for those members in the provided order
- `ota extensions --run <name>` executes one explicitly named, allowlisted descriptor in the
  current repo or member context
- `ota extensions --publish <name>` executes one explicitly named, allowlisted `export_provider`
  descriptor in the current repo or member context
- execution currently accepts `kind: check_provider` descriptors with `api_version: 1`
- execution currently accepts `kind: export_provider` descriptors with `api_version: 1`
- execution also accepts `kind: backend_provider` descriptors for remote execution when named by
  `execution.backends.remote.provider`
- backend providers receive a structured JSON request and must return a structured JSON response;
  the request is delivered on stdin and mirrored in `OTA_BACKEND_PROVIDER_REQUEST_JSON` for shell
  adapters
- the seam is useful for external adapter contracts such as check providers, export targets, and
  execution backends that should be discoverable without being hidden in shell scripts

Text output:

- header: `EXTENSIONS <path>`
- each descriptor may include `kind`, `command`, `api_version`, `description`, and `config`
- the report is read-only unless `--run <name>` is set
- when no descriptors are staged, the text output says so explicitly and points users back to
  `ota doctor` or adding `extensions` to the contract

JSON output:

- success: `ok`, `path`, `extensions`
- monorepo root summaries include grouped per-member results in `members`
- repeated `--member` values return grouped per-member results in `members`
- `--run <name>` returns the executed descriptor, `exit_code`, and captured `stdout`/`stderr`
- `--publish <name>` returns the executed descriptor, `exit_code`, and captured `stdout`/`stderr`
- failure: `ok`, `path`, and either `errors` or `error`

## `ota run`

Run a validated task.

```bash
ota run <task> [PATH]
ota run <task> --stream [PATH]
ota run <task> --member api [PATH]
ota run <task> --member api --member web [PATH]
ota run <task> --mode native [PATH]
ota run <task> --mode container --ephemeral [PATH]
ota run <task> --mode remote [PATH]
ota run <task> [PATH] --base-url http://localhost:8080
```

Current behavior:

- validates the contract first
- when `--member` is set, resolves the merged member contract from the monorepo root
- repeated `--member` values run the task across those members in the provided order
- `--mode`, `--lifecycle`, and `--ephemeral` can override the contract for one invocation
- task inputs are declared in `tasks.<name>.inputs` and are passed as `--kebab-case value` flags
- when a task input overlaps an ota command flag name such as `mode` or `jobs`, put the ota command flag before the task and the task input after the task
- task inputs are exposed to the task process as `OTA_INPUT_<NAME>` env variables
- `default` values are applied when the caller omits an input
- `required: true` makes an input mandatory unless a default exists
- `allowed` limits the accepted values for that input
- task inputs only apply to the task you invoked, not its dependencies
- if every declared input has a default, you can omit all input flags
- by default, interactive terminals stream raw child output live, while non-interactive text runs buffer output into the final report for a cleaner failure/success surface
- `--stream` forces raw live child output in text mode when you want the old firehose behavior explicitly
- on failure, text output keeps `Why` and `Next` first, then appends a compact `RUN SUMMARY` block with the selected mode, container image when relevant, target when one exists, and task
- on non-interactive text success, large task output is shown as a bounded excerpt before the compact `RUN SUMMARY`
- on non-interactive text failure, task output is shown as a bounded excerpt with a `--stream` rerun hint before the compact `RUN SUMMARY`
- on success, text output includes the compact `RUN SUMMARY` block with the selected mode, container image when relevant, target when one exists, and task
- `--receipt` adds the full execution receipt when you need the detailed trail

Example:

```yaml
tasks:
  api-automation-tests:
    inputs:
      base_url:
        default: http://localhost:8080
      suite_mode:
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
ota run api-automation-tests --base-url http://localhost:8080 --suite-mode contract-drift
ota run version:bump --version minor
ota run version:bump --version 0.2.0
ota run version:bump --version major
```

- resolves task dependencies before execution
- if the task body exits successfully, runs `after_success` hooks in declared order
- if the task body exits with a failure, runs `after_failure` hooks in declared order
- runs `after_always` hooks after either outcome when the task body was actually attempted
- hook task failures affect the final `ota run` exit code for the parent task
- resolves the best matching task variant for the current OS when variants are declared
- executes either `run` or `script`
- when `execution.preferred: container` is configured with `execution.backends.container.image`, runs tasks through the first available configured container engine CLI, falling back to `docker` when no engines are listed
- when container execution is configured, `execution.lifecycle: ephemeral` uses a fresh container and `execution.lifecycle: persistent` reuses a named container
- supports remote execution when `execution.backends.remote.provider` and `execution.backends.remote.target` are configured
- current shipped remote providers are `daytona`, `ssh`, `tsh`, and `kubectl`
- remote target guidance:
- `daytona`: `sandbox-dev`
- `ssh` / `tsh`: `user@host`
- `kubectl`: `pod/ota-dev`
- passes `execution.backends.remote.cwd` to the provider CLI when set
- runs in the effective target contract directory
- applies configured environment values, approved policy env values, and task input env variables
- prints task progress and advisory notes on stderr when output is streaming
- prints a summary in text output, and emits an execution receipt on stderr after task output when `--receipt` is set
- execution receipts include backend, lifecycle, container image when relevant, remote target when set, acquired paths, env sources, and step summary data; text receipts also print the winning env source for each resolved value
- returns the child process exit code

Use this when the contract is already the source of truth and you want deterministic task execution.

## `ota doctor`

Diagnose repo readiness from a validated contract.

```bash
ota doctor [PATH]
ota doctor --mode native [PATH]
ota doctor --mode container [PATH]
ota doctor --json [PATH]
ota doctor --member api [PATH]
ota doctor --member api --member web --json [PATH]
```

- Current behavior:

- when no contract exists, inspects repo and host signals and reports the best next step instead of only telling the user to create a contract
- validates the contract first when one is present
- when a root contract declares `workspace.type: monorepo`, plain `ota doctor` diagnoses the root contract and grouped summaries for each declared member
- when `--member` is set, diagnoses the merged member contract
- repeated `--member` values diagnose those members in the provided order
- prints the highest-priority blocker first in the human-readable output so the fastest next action is visible immediately
- checks configured env requirements, declared checks, and service healthchecks in native mode
- checks preferred execution backend prerequisites such as `docker` / `podman` / `nerdctl`, `daytona`, `ssh`, `tsh`, or `kubectl` when backend-backed execution is configured
- `--mode native` diagnoses host/native readiness; `--mode container` diagnoses the selected container execution context when container backends are declared
- warns on suspicious remote target shape:
- `ssh` / `tsh` targets without `user@host`
- `kubectl` targets not starting with `pod/`
- checks runtime and tool presence on `PATH`
- in container mode, runtime and tool findings are evaluated against the selected container image instead of the host PATH
- in container mode, ota also uses safe non-mutating installability probes for the shipped mutating provisioning adapters when policy-backed provisioning is declared
- in container mode, `apt` findings distinguish pinned-version unavailable, package unavailable, and apt-index/source failures when the backend evidence supports that classification
- in container mode, host-bound env, check, and service healthchecks are omitted so container diagnosis does not mix execution contexts
- shows any inert top-level `extensions` entries in the human-readable report so adapter metadata is visible without execution
- warns when a required service has no healthcheck, because readiness cannot be verified
- honors `services.<name>.timeout` when a service healthcheck is declared
- warns when `execution.lifecycle: ephemeral` is declared and clarifies that current isolation applies to `ota run <task>` and the setup step inside `ota up`; diagnosis, healthchecks, and full repo cleanup are not ephemeral yet, and `--ephemeral` remains the shorthand for a fresh task-execution path when supported
- reports contract drift as warning findings when repo signals no longer match the declared
  contract, and still preserves the most important blocker first
- tags contract-drift findings with repo-contract ownership and provenance so consumers can
  distinguish stale contract truth from host or service failures
- reports an error when no `tasks` are declared, because the contract is not operational for `ota run`
- runs configured checks
- orders findings by severity
- includes an `agent` summary when the contract declares one
- may include a `provisioning` plan when the contract declares runtimes or tools and policy
  provides approved provisioning sources
- prints the reason and next action for each finding

Text output:

- header: `DOCTOR <path>`
- status line: `READY` or `NOT READY`
- `Execution` includes a `Mode:` line in text output so the selected diagnosis context is explicit
- summary includes repo verdict and agent verdict before per-finding details
- grouped finding sections include `Provenance:` when the grouped findings share one diagnosis source
- with `--concise`, findings keep severity + summary + `Next`, while `Why` detail is omitted

JSON output:

- `ok`
- `path`
- `agent` when the contract declares agent guidance
- `findings`
- findings may also include `provenance` / `provenance_key` when ota can trace the diagnosis back to the repo contract, org policy, or repo signals
- monorepo root summaries include grouped per-member results in `members`
- repeated `--member` values return grouped per-member results in `members`

Warnings can still produce `READY`. Errors produce `NOT READY`.

## `ota init`

Create a starter ota contract for a repo that does not yet have one.

```bash
ota init [PATH]
ota init --bootstrap [PATH]
ota init --pack <node|python|go|rust|dotnet|php-composer|java-maven|java-gradle> [PATH]
ota init --pack node --package-manager <npm|pnpm|yarn|bun> [PATH]
ota init --pack python --test-runner <pytest|unittest> [PATH]
ota init --packs
ota init --dry-run [PATH]
ota init --json [PATH]
```

Current behavior:

- inspects the repo using the detection engine
- writes by default
- `--bootstrap` writes the fuller detected starter contract when it is safe to do so
- `--pack <node|python|go|rust|dotnet|php-composer|java-maven|java-gradle>` skips detector-led starter selection and seeds an explicit conventional starter contract pack, including short task `description` fields on the seeded starter tasks
- `--pack node --package-manager <npm|pnpm|yarn|bun>` keeps pack mode explicit while swapping the conventional Node starter commands and seeded tool requirement to the selected package manager
- `--pack python --test-runner <pytest|unittest>` keeps pack mode explicit while swapping the conventional Python test entrypoint to the selected runner
- `--packs` lists the built-in starter packs, what they seed, the exact `ota init --pack ...` selection command, the safe dry-run preview command to use next, and any explicit starter knobs exposed by that pack
- when no stronger project identity is inferred, `--bootstrap` can fall back to the repo directory name for `project.name`
- supports preview mode with `--dry-run`
- refuses to run when `ota.yaml` already exists
- can initialize both detected repos and blank repos
- keeps JSON output stable while using text output to guide review, write, and first validation steps
- in `detected` mode, plain `ota init` writes the smallest valid starter contract for the repo
- in `detected` mode, `ota init --bootstrap` can include lower-confidence fields when they are needed to capture the fuller starter contract
- when repo-root `.env.local` or `.env` files already exist, detector-led init can declare them as explicit `env.sources` in the starter contract; explicit `--pack` mode does not infer env sources from repo files
- when `project.name` is still missing in bootstrap mode, ota falls back to the repo directory name rather than leaving the contract invalid
- low-confidence fields remain excluded from plain `ota init` writes
- canonical detected tasks can include short `description` fields so the starter contract teaches the task-authoring pattern immediately instead of only relying on notes
- confident detected tasks may include a `notes` field that points to the matching `ota run <task>` command
- when the detected tasks are confident enough and ota can infer safe writable paths, the starter contract may also include a minimal `agent` block and review notes; see [`contract-reference.md`](contract-reference.md) for the `agent` field semantics

Choosing an init path:

- use `ota init --dry-run` when detector-led init should shape the first draft from repo signals
- use detector-led init when you want ota to carry existing runtime dotenv sources such as `.env.local` or `.env` into `env.sources`
- use plain `ota init` only after reviewing that detector-led starter
- use `ota init --packs` when you want to compare the explicit starter catalog first
- use `ota init --pack <name> --dry-run` when you want an explicit conventional starter without detector-led selection
- use `ota init --pack node --package-manager <name> --dry-run` when the repo is intentionally npm-, pnpm-, yarn-, or bun-based and you want the starter to match that package-manager boundary from the first write
- use `ota init --pack python --test-runner <name> --dry-run` when the repo is intentionally `pytest`- or `unittest`-driven and you want the starter to reflect that test command directly
- use `ota init --pack dotnet --dry-run` when the repo is intentionally .NET-first and the standard `dotnet restore` / `dotnet build` / `dotnet test` loop is already the honest first draft
- use `ota init --pack php-composer --dry-run` when the repo is intentionally Composer-managed PHP and `composer install` plus reuse of an existing `scripts.test` entry is the honest first draft you want to review
- the Java packs prefer `mvnw` or `gradlew` when those wrappers already exist
- explicit packs seed short task `description` fields so the authoring pattern is visible immediately

Examples:

```bash
# detector-led path
ota init --dry-run
ota init

# pack-led path
ota init --packs
ota init --pack node --dry-run
ota init --pack node --package-manager yarn --dry-run
ota init --pack python --dry-run
ota init --pack python --test-runner unittest --dry-run
ota init --pack go --dry-run
ota init --pack rust --dry-run
ota init --pack dotnet --dry-run
ota init --pack php-composer --dry-run
ota init --pack java-maven --dry-run
ota init --pack java-gradle --dry-run
```

Modes:

- `blank`: starter contract derived from minimal repo context
- `detected`: starter contract derived from detected repo signals
- `pack`: starter contract derived from an explicit built-in starter pack
- `catalog`: starter-pack discovery output from `ota init --packs`

Text output:

- dry-run header: `INIT <path>`
- write success: `WROTE <path>`
- includes `Mode: blank` or `Mode: detected`
- `pack` mode also includes `Pack: <name>`, optional `Options: ...` when the selected starter pack supports explicit knobs, plus an explicit pack-policy note
- explicit pack mode can also include an advisory note with `Why`, weighted `Signals`, `Selected signals`, `Strength`, `Gap`, and `Next` rows when strong repo signals disagree with the selected pack; ota does not auto-switch or merge detector output into the pack
- `--packs` renders `INIT PACKS starter packs`, one entry per pack, the exact `ota init --pack ...` command, any starter-specific option rows, and a `Next:` line with the matching `ota init --pack ... --dry-run .` preview command
- includes a `Next:` line that tells the user how to review or validate the starter contract
- `blank` mode explicitly warns that the starter contract is minimal coverage only
- `detected` mode write output explicitly calls out the write policy and any excluded low-confidence fields
- includes inferred-field annotations with source and confidence

JSON output:

- `ok`
- `path`
- `written`
- `mode`
- optional `pack` when explicit pack mode is used
- optional `pack_options` when explicit pack mode selected a starter-specific knob such as Node package manager or Python test runner
- optional `pack_advisory` when explicit pack mode disagrees with strong detected repo signals; it includes the selected pack, suggested pack, distinct-signal scores, score gap, normalized signal markers, weighted signal details for both the suggested and selected pack, and a safe dry-run follow-up command
- `config`
- `inferred`
- `packs` when `mode` is `catalog` and ota is listing the built-in starter packs instead of previewing one contract; each entry includes `name`, `summary`, `when`, the exact `command`, a safe `next` preview command, optional starter `options`, explicit `does_not_infer` boundaries, and the seeded runtimes, tools, checks, and tasks
- failure responses can include `next` when ota can point to one safe follow-up command

## `ota agents`

Generate or sync a repo-local `AGENTS.md` from the current contract.

Use this after `ota doctor`, `ota explain`, or `ota up` when you want the same repo contract to
produce reviewable agent guidance for humans and coding agents.

```bash
ota agents [PATH]
ota agents --write [PATH]
ota agents --json [PATH]
ota agents --write --output AGENTS.md [PATH]
```

Current behavior:

- derives `AGENTS.md` from the repo contract’s `agent` block when one is present
- falls back to a lightweight scaffold that makes the missing `agent` block explicit when one is not present
- includes a fallback hint to run `ota tasks` when you want to inspect runnable task commands before generating or editing agent guidance
- renders an explicit `Bootstrap` section when `agent.bootstrap.ota` is present, including the approved shell and PowerShell install commands for `ota`
- preserves existing `AGENTS.md` content and appends or refreshes an ota-managed block instead of overwriting user-authored guidance
- skips the write if the existing file already contains the generated AGENTS content
- keeps the generated file lightweight by using short provenance (`Generated from ... by \`ota agents\`.`) instead of an Ota copyright or license banner
- renders a `Managed block:` label in text output so the ota-owned section is explicit and shows each task list item together with its `ota run ...` command form
- text preview points directly at `ota agents --write` and `ota doctor` so the guidance can be written and then verified from the same contract
- pairs naturally with `ota tasks --use` when you want to confirm runnable task shapes before writing the guidance file
- writes to `AGENTS.md` by default when `--write` is set
- accepts `--output` to write elsewhere
- keeps output deterministic and reviewable

Text output:

- header: `AGENTS <path>`
- preview mode shows the generated markdown content together with the write and verification next steps
- write mode reports whether the target was written or already in sync and points back to `ota doctor`

JSON output:

- `ok`
- `path`
- `output`
- `written`
- `content`
- failure responses can include `next` when ota can point to one safe follow-up command

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

## `ota receipt`

Capture the current repo readiness scan as a read-only receipt artifact for CI or archival use.

```bash
ota receipt [PATH]
ota receipt --json [PATH]
ota receipt --mode container [PATH]
ota receipt --archive [PATH]
ota receipt --archive --promote-baseline [PATH]
ota receipt --baseline promoted [PATH]
ota receipt --baseline latest [PATH]
ota receipt --baseline ./baseline-receipt.json [PATH]
ota receipt --baseline latest --fail-on-new-blockers [PATH]
ota receipt --history [PATH]
ota receipt --member api [PATH]
```

Current behavior:

- resolves `ota.yaml` using `--file`, `OTA_FILE`, or upward discovery
- `--member <name>` captures the merged monorepo member contract instead of the root contract
- validates the contract first
- runs repo readiness diagnosis in the selected execution context
- includes repo contract drift findings from the same `ota detect` comparison path used by `ota doctor`
- captures the current repo state as an execution receipt with one `readiness` step
- never provisions, runs tasks, starts services, or writes repo state
- `--json` returns a repo receipt artifact with `mode: "receipt"`
- `--archive` writes the JSON receipt to `.ota/receipts` and keeps the newest 50 archives
- `--archive --promote-baseline` also writes `.ota/receipts/repo-baseline.json`, pointing at the archived receipt as the repo's explicit promoted baseline
- `--history` lists archived repo receipts from `.ota/receipts` newest first without loading or validating the current contract; explicit paths must be a repo directory or an `ota.yaml` file
- `--baseline promoted` compares the current receipt against the explicit promoted baseline pointer under `.ota/receipts/repo-baseline.json`
- `--baseline latest` compares the current receipt against the newest valid archived repo receipt for the same contract under `.ota/receipts`
- `--baseline <file>` compares the current receipt against an explicit repo receipt JSON file
- compare mode is read-only and does not archive or mutate repo state; it exits `0` when the comparison itself succeeds, even if the current or baseline receipt is not ready
- `--fail-on-new-blockers` requires `--baseline` and exits `1` when the diff introduces one or more new `severity: error` findings relative to the baseline

Text output:

- header: `RECEIPT <path>`
- prints the receipt steps, compact contract identity, summary, env sources, policy lines, and blocked items when present
- `--archive --promote-baseline` adds `Baseline:` and `Promoted:` summary lines so the operator can see which archive became the explicit repo baseline
- `--history` switches the text header to `RECEIPT HISTORY <path>` and lists archived receipt files with their archived time, readiness result, and summary counts; malformed archived files are skipped and surfaced under `Skipped Archives`
- `--baseline` switches the text header to `RECEIPT DIFF <path>` and reports the baseline source plus provenance such as the selection path, promoted time, contract identity, introduced findings, resolved findings, and unchanged findings when there are no newly introduced or resolved changes
- `--fail-on-new-blockers` adds a `Gate:` overview line showing whether the current diff passed or was blocked by newly introduced blockers

JSON output:

- `ok`
- `path`
- `mode: "receipt"`
- `archive_path` (when `--archive` is set)
- `promoted_baseline.path`, `promoted_baseline.archive_path`, and `promoted_baseline.promoted_at` (when `--archive --promote-baseline` is set)
- `summary` mirroring the receipt summary with `error_count`, `warn_count`, `info_count`, and `step_count`
- `receipt`, including additive `receipt.contract_identity` with declared project, selected metadata, execution intent, and compact contract counts
- `findings`
- `--history` switches `mode` to `history` and returns `summary.archive_count`, `summary.invalid_archive_count`, an `archives` array for valid archived receipts, and `invalid_archives` when malformed archive files were skipped
- `--baseline` switches `mode` to `diff` and returns `baseline`, `current`, `summary`, `introduced`, `resolved`, and `unchanged`, with additive provenance fields on `baseline`
- diff `summary` also carries a compact `comparison` block so wrappers can show baseline/current identity labels plus readiness drift without reconstructing it from the full baseline/current sections
- `--fail-on-new-blockers` adds `gate.rule`, `gate.passed`, and `gate.new_blocker_count` to diff JSON when the compare gate is active
- when that compare gate blocks, diff JSON also carries the first blocking summary, next step, and provenance so CI summaries and PR comments do not need to scrape the full `introduced` array

Current non-goals:

- mutating repo state
- replacing `ota doctor` as the full readiness explanation surface
- separate receipt storage outside the explicit `.ota/receipts` archive directory
- monorepo multi-member roll-up beyond the selected resolved contract target
- multi-rule diff gating beyond the explicit `--fail-on-new-blockers` compare gate

## `ota up`

Prepare a repo for use with minimal prior knowledge.

```bash
ota up [PATH]
ota up --json [PATH]
ota up --stream [PATH]
ota up --dry-run [PATH]
ota up --dry-run --json [PATH]
ota up --mode container --ephemeral [PATH]
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
- when blocking preconditions fail and the repo declares `setup`, runs `setup` early and then re-checks readiness
- when the effective execution mode is container, policy-backed provisioning adapters run inside that container instead of on the host
- explicit or effective container-backed `ota up` stays container-authoritative; if no supported container engine is available, ota stops in preconditions instead of falling back to host provisioning
- when provisioning fails, `ota up` now surfaces a higher-level backend diagnosis for every shipped adapter while still preserving the raw backend stdout/stderr in the failure output
- when the initial provisioning stderr is too generic to classify safely, `ota up` reuses the read-only installability probe for that adapter to refine the diagnosis without hiding the original backend output
- when container/Linux provisioning uses `apt`, ota also classifies supported provisioning failures as pinned-version unavailable, package unavailable, or apt-index/source failures
- `--dry-run` reuses the same contract path, member targeting, backend selection, lifecycle selection, and provisioning plan resolution as `ota up`, but does not mutate repo or execution state
- runs explicit `services.<name>.start` commands for required services before setup
- starts required services, and required-service dependencies, in declared dependency order
- verifies required service healthchecks before setup and treats them as readiness gates
- stops in the `services` phase when required-service readiness still fails
- runs the `setup` task if one exists, using the configured execution backend when present
- can override execution mode and lifecycle for the `setup` phase with `--mode`, `--lifecycle`, or the shorthand `--ephemeral`
- the current `setup` backend path supports native, container, and the shipped remote providers
- prints a lifecycle note on stderr when the `setup` phase uses backend-backed execution
- re-runs readiness diagnosis
- still runs service start commands, service healthchecks, and diagnosis on the host today
- returns `READY` or `NOT READY`
- reports the phase where execution stopped: `preconditions`, `services`, `setup`, or `post-setup diagnosis`
- reports `provisioning` when early setup ran but the repo is still not ready
- includes setup exit code details when the `setup` task fails
- includes service start exit code details when a required service start command fails
- keeps child output compact by default and surfaces failed service/setup output inside the final report
- `--stream` opts into raw live child output for provisioning, required service `start` commands, and the `setup` task
- `--stream` is text-only and is only supported for mutating `ota up`
- prints a summary in text output, emits an execution receipt when `--receipt` is set, and includes `summary` plus a `receipt` object in repo-target JSON output; monorepo aggregate JSON keeps grouped `members` results instead of inventing a top-level receipt
- `--dry-run` prints `UP PREVIEW`, shows the selected execution backend, lifecycle, container image when relevant, a real named target when one would exist, the setup task, the actions ota would attempt, the actions ota would skip because current state already satisfies them, the compact contract identity, and the first blocking readiness finding when one exists
- `--dry-run` never provisions, starts services, runs setup, or writes repo files
- `--receipt` is only for mutating `ota up`; it conflicts with `--dry-run`
- the detailed preview contract lives in [up-preview.md](up-preview.md)

This is the onboarding command. It is intentionally narrower than a general-purpose environment orchestrator.

## `ota self-update`

Update the installed ota binary.

```bash
ota self-update
ota self-update --version v0.1.3
ota self-update --channel stable
ota upgrade
ota upgrade --version v0.1.3
ota upgrade --channel stable
```

Current behavior:

- `ota self-update` and `ota upgrade` are aliases
- `--version` pins a specific release
- `--channel` currently accepts `stable` and `latest`
- `stable` resolves the latest stable release tag
- `latest` resolves the newest release entry, including prereleases if present
- `--version` overrides the channel when both are set
- when the chosen target matches the installed binary, the command exits successfully and prints the up-to-date banner instead of reinstalling
- on Windows, when `ota` is currently running, the downloaded binary is staged and applied after the current process exits

## `ota policy`

Show the active policy pack, its source, and the resolved path.

```bash
ota policy [PATH]
ota policy --json [PATH]
ota policy --file /path/to/ota.yaml
ota policy --file /path/to/ota.yaml --json
```

Current behavior:

- resolves the policy pack using the same precedence ota uses for repo commands
- shows the effective policy content and where it came from
- accepts `OTA_POLICY` as a local file path or `http(s)://` URL override
- falls back to the nearest ancestor `.ota/org-policy.yaml`
- falls back again to the nearest ancestor `ota.workspace.yaml` `workspace.policy` when present
- remains read-only

Text output:

- header: `POLICY <path>`
- `Policy source:` shows where ota loaded the policy from
- `Policy path:` shows the resolved policy file path or URL
- effective policy content when one is loaded
- when a policy pack is loaded, `Next:` points to `ota policy review` for boundary inspection and
  `ota doctor` for readiness with that active policy applied
- when no policy pack is found, the text output says so explicitly and points users back to repo
  readiness or `.ota/org-policy.yaml`

JSON output:

- `ok`
- `path`
- `policy_source` and `source`
- `policy_path`
- `policy`
- failure responses include `error`

Use this when you need to confirm which org policy ota actually applied before a run or diagnosis.

## `ota policy init`

Create a conservative starter org policy pack.

```bash
ota policy init [PATH]
ota policy init --preset required-sections [PATH]
ota policy init --preset provisioning [PATH]
ota policy init --preset agent [PATH]
ota policy init --dry-run [PATH]
ota policy init --json [PATH]
ota policy init --dry-run --json [PATH]
```

Current behavior:

- writes by default
- refuses to overwrite an existing policy pack
- defaults to `.ota/org-policy.yaml` under the current directory when no path is given
- accepts a repo root, a `.ota/` directory, or an explicit `.ota/org-policy.yaml` target path
- supports explicit starter presets: `required-sections`, `provisioning`, and `agent`
- writes the minimal valid starter today: `policies: {}`
- stays conservative and does not infer org rules or add provisioning approvals automatically
- `required-sections` starts with a small required-section policy (`runtimes` and `tasks`)
- `provisioning` scaffolds empty `provisioning` and `adapter_bootstrap` maps plus inline example guidance
- `agent` starts with agent-safety and `AGENTS.md` export requirements enabled

Text output:

- write header: `POLICY INIT <path>`
- preview header: `POLICY INIT PREVIEW <path>`
- `Preset:` is shown when a preset is selected
- preview shows the starter policy pack YAML without writing it
- write output confirms the written path and points back to `ota policy`
- overwrite refusal stays explicit and non-mutating

JSON output:

- `ok`
- `path`
- `written`
- `mode` (`policy`)
- optional `preset`
- `config`
- failure responses include `error`
- overwrite refusals may include `next`

Use this when a team needs a valid `.ota/org-policy.yaml` scaffold without guessing policy intent or hand-authoring the starter shape.

## `ota policy review`

Review the policy-vs-contract boundary and approved policy sources.

```bash
ota policy review [PATH]
ota policy review --json [PATH]
ota policy review --file /path/to/ota.yaml
ota policy review --file /path/to/ota.yaml --json
```

Example:

```bash
ota policy review
```

Current behavior:

- resolves the active policy pack using the same precedence as `ota policy`
- focuses only on policy-authority findings, approved provisioning sources, and adapter bootstrap sources
- stays read-only
- points repo-owned conflicts back to `ota.yaml`
- points governance-owned conflicts back to `.ota/org-policy.yaml`

Text output:

- header: `POLICY REVIEW <path>`
- `Policy source:` shows where ota loaded the policy from
- `Policy path:` shows the resolved policy file path or URL
- `Overview` rolls up the policy findings by severity
- grouped policy findings reuse the same premium remediation styling as `ota doctor`
- when no policy pack is found, the text output says so explicitly and points users back to `ota policy`

JSON output:

- `ok`
- `path`
- `policy_source`
- `policy_path`
- `summary`
- `finding_groups`
- `policy`
- `findings`

Use this when you need to understand what policy ota enforced, why a repo-contract request is outside the approved policy boundary, or whether the org policy pack itself needs to change.

## `ota completion`

Show how to enable shell completion for ota.

```bash
ota completion --setup
ota completion --remove
ota completion check
ota completion bash
ota completion bash --script
ota completion zsh
ota completion fish
ota completion powershell
ota completion elvish
```

Current behavior:

- `ota completion --setup` detects the current shell when possible and installs ota's managed hook into the shell profile or completion file idempotently
- `ota completion --remove` detects the current shell when possible and removes ota's managed hook plus any managed zsh support file idempotently
- `ota completion <shell> --setup` installs the managed hook for one explicit shell without relying on auto-detection
- `ota completion <shell> --remove` removes the managed hook for one explicit shell without relying on auto-detection
- `ota completion check` verifies the detected shell, the current ota binary path, the target profile or completion file, any managed zsh completion file, and whether the managed hook is present or needs refresh
- `ota completion <shell>` prints the manual shell setup ota expects for that shell; for zsh it includes both the `_ota` completion file and the `.zshrc` loader
- `ota completion <shell> --script` prints the exact raw registration script clap generates for that shell so users can inspect the shell-side function directly
- zsh setup writes a managed `_ota` completion file under `~/.config/ota/zsh/_ota` and loads that exact file through the shell completion path instead of relying on late runtime `compdef` registration alone
- once the shell has sourced that setup, `ota <TAB>` completes commands first and keeps global flags after them in zsh
- once the shell has sourced that setup, `ota run <TAB>` completes task names only when one shared invocation can satisfy the selected repo/member target set, and shells that support candidate help can also show each task description when the contract declares one
- once the shell has sourced that setup, `ota run <task> <TAB>` completes shared task input flags and any constrained values that remain valid across the selected repo/member target set
- once the shell has sourced that setup, `ota env --task <TAB>` completes task names from the active repo or selected monorepo member, using the same task-description metadata when available
- once the shell has sourced that setup, `ota extensions --run <TAB>` and `ota extensions --publish <TAB>` complete declared extension names for the active repo or selected member target
- once the shell has sourced that setup, `ota receipt --baseline <TAB>` completes `latest`, `promoted`, and archived receipt JSON files from the active repo's `.ota/receipts`
- once the shell has sourced that setup, `--member <TAB>` completes monorepo member names from the active repo contract
- once the shell has sourced that setup, `ota workspace run <TAB>` completes task names only when one shared invocation can satisfy the currently available workspace repos, with shared task descriptions when the participating repos agree on that description
- once the shell has sourced that setup, `ota workspace run <task> <TAB>` completes shared task input flags and any constrained values that remain valid across the currently available workspace repos
- once the shell has sourced that setup, `ota workspace doctor --repo <TAB>`, `ota workspace explain --repo <TAB>`, and `ota workspace list --repo <TAB>` complete declared workspace repo names
- when no repo contract is available, shell completion falls back to static command and flag suggestions
- the auto-installed hook is managed between `# >>> ota completion >>>` and `# <<< ota completion <<<` markers so rerunning setup updates or reuses the same block instead of appending duplicates
- `ota completion --remove` only strips ota's managed block and managed zsh support file; it does not try to edit unrelated shell completion setup
- users should reload or re-source their shell after upgrading ota so the shell-side glue and the installed binary stay in sync

Use this when you want contract-aware shell suggestions instead of memorizing task names and task input flags.

Automatic setup:

```bash
ota completion --setup
ota completion --remove
ota completion zsh --setup
ota completion zsh --remove
```

Verification and inspection:

```bash
ota completion check
ota completion bash --script
```

Manual setup examples:

`bash`

```bash
ota completion bash
# >>> ota completion >>>
if command -v ota >/dev/null 2>&1; then
  source <(COMPLETE=bash ota)
fi
# <<< ota completion <<<
```

`zsh`

```zsh
ota completion zsh
Manual completion file (~/.config/ota/zsh/_ota):
#compdef ota
_ota() {
    local _CLAP_COMPLETE_INDEX=$(expr $CURRENT - 1)
    local _CLAP_IFS=$'\n'

    local completions=("${(@f)$( \
        _CLAP_IFS="$_CLAP_IFS" \
        _CLAP_COMPLETE_INDEX="$_CLAP_COMPLETE_INDEX" \
        COMPLETE="zsh" \
        ota -- "${words[@]}" 2>/dev/null \
    )}")

    if [[ -n $completions ]]; then
        local -a primary_values=()
        local -a primary_display=()
        local -a option_values=()
        local -a option_display=()
        local completion
        for completion in $completions; do
            local value="${completion%%:*}"
            if [[ "$value" == -* ]]; then
                option_values+=("$value")
                if [[ "$completion" == *:* ]]; then
                    option_display+=("$value -- ${completion#*:}")
                else
                    option_display+=("$value")
                fi
            else
                primary_values+=("$value")
                if [[ "$completion" == *:* ]]; then
                    primary_display+=("$value -- ${completion#*:}")
                else
                    primary_display+=("$value")
                fi
            fi
        done
        [[ -n $primary_values ]] && compadd -Q -V ota_primary -d primary_display -o nosort -- "${primary_values[@]}"
        [[ -n $option_values ]] && compadd -Q -X 'Options' -V ota_options -d option_display -o nosort -- "${option_values[@]}"
    fi
}

Manual setup (~/.zshrc):
# >>> ota completion >>>
if command -v ota >/dev/null 2>&1; then
  _ota_completion_file="$HOME/.config/ota/zsh/_ota"
  if [[ -f "$_ota_completion_file" ]]; then
    _ota_completion_dir="${_ota_completion_file:h}"
    if (( ${fpath[(Ie)$_ota_completion_dir]} == 0 )); then
      fpath=("$_ota_completion_dir" $fpath)
    fi
    autoload -Uz _ota 2>/dev/null
    if typeset -p _comps >/dev/null 2>&1; then
      _comps[ota]=_ota
    elif whence compdef >/dev/null 2>&1; then
      compdef _ota ota
    else
      autoload -Uz compinit
      compinit
      _comps[ota]=_ota
    fi
  fi
  unset _ota_completion_file _ota_completion_dir
fi
# <<< ota completion <<<
```

`fish`

```fish
ota completion fish
# >>> ota completion >>>
if type -q ota
    COMPLETE=fish ota | source
end
# <<< ota completion <<<
```

`PowerShell`

```powershell
ota completion powershell
# >>> ota completion >>>
if (Get-Command ota -ErrorAction SilentlyContinue) {
  $env:COMPLETE = "powershell"
  ota | Out-String | Invoke-Expression
  Remove-Item Env:\COMPLETE -ErrorAction SilentlyContinue
}
# <<< ota completion <<<
```

`elvish`

```elvish
ota completion elvish
# >>> ota completion >>>
if (has-external-command ota) {
  eval (E:COMPLETE=elvish ota | slurp)
}
# <<< ota completion <<<
```

Troubleshooting:

- `zsh`: if completions still do not appear after setup, reopen the shell or confirm `ota completion check` shows both `Hook: present` and a `Completion file:` line for the managed `_ota` file; `ota completion --remove` gives you a clean reinstall path
- `bash`: if completions still do not appear after setup, reopen the shell or source the profile again with `. ~/.bashrc`
- `ota completion check` should report `Hook: present`; if it reports `missing` or `needs update`, rerun `ota completion --setup`
- `ota completion <shell> --script` lets you inspect the exact raw registration script when the shell-side behavior itself looks wrong

## `ota uninstall`

Remove ota from this laptop.

```bash
ota uninstall
```

Current behavior:

- removes the installed ota binary from the current machine
- on Windows, schedules best-effort removal of the running executable after the current process exits and reports the result as pending until deletion can actually happen
- on Unix-like systems, removes the current executable directly when possible
- does not touch repo state, contracts, or workspace state

Text output:

- success: `removed ota from <path>` or `pending ota removal from <path> after the current process exits; removal is not yet verified`
- already removed: `ota was already removed from <path>`

Use this when you want to remove ota from the machine itself, not when you want to clean a repo.

- on success, the command runs the installer for the chosen release target

Use this when:

- you already have ota installed and want to update it in place

Use-case:

- a developer sees the update notice after `ota doctor` and runs `ota self-update`

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
- contract load/validation failures return the same failure envelope as `ota validate --json` (`ok`, `path`, and either `errors` or `error`)

## `ota clean`

Clean persistent execution state for a repo.

```bash
ota clean [PATH]
ota clean --member api [PATH]
ota clean --member api --member web [PATH]
ota clean --stale
ota clean --stale --dry-run
ota clean --stale --json
```

Current behavior:

- validates the contract first
- when a root contract declares `workspace.type: monorepo`, plain `ota clean` reports the root cleanup result and grouped member cleanup results
- when `--member` is set, targets those merged member contracts in the provided order
- when the effective execution mode is `container` with `lifecycle: persistent`, removes the named persistent container for that repo
- `ota clean --stale` does not require `ota.yaml`; it scans available local container engines for exited ota-managed containers from any repo
- stale cleanup uses ota ownership labels first and falls back to legacy `ota-*` container names for older persistent backends
- if a local container engine cannot answer `ps`, stale cleanup continues with other available engines and only fails when none of them can be queried
- `ota clean --stale --dry-run` previews stale containers without removing them
- `ota clean --stale --json` emits the matched engines, containers, and cleanup counts for automation
- `ota clean --stale` has its own exit-code contract and is separate from repo-scoped `ota clean`
- remote backends do not currently define cleanup semantics; they report `No cleanup needed`
- reports `No cleanup needed` when there is no persistent container state to remove
- does not stop services or perform workspace-wide cleanup

## `ota detect`

Infer a starting contract from repo state.

```bash
ota detect --dry-run [PATH]
ota detect --json --dry-run [PATH]
ota detect --contract [PATH]
ota detect --write [PATH]
ota detect --json --write [PATH]
ota detect --merge --dry-run [PATH]
ota detect --merge --apply FIELD [PATH]
ota detect --merge --apply-all [PATH]
ota detect --merge [PATH]
ota detect --rewrite --dry-run [PATH]
ota detect --rewrite --yes [PATH]
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
- `rebar.config`
- `build.zig`
- `dub.json` / `dub.sdl`
- `fpm.toml`
- `shard.yml`
- `elm.json`
- `cpanfile` / `Makefile.PL`
- `*.hxml`
- `docker-compose.yml` / `docker-compose.yaml`
- `compose.yml` / `compose.yaml`

For Docker Compose service inference, ota currently derives:

- `provider` at high confidence
- `start` / `stop` at medium confidence
- declared `healthcheck.test` at medium confidence

Dry-run behavior:

- `ota detect` is read-only by default
- prints a candidate `ota.yaml`
- prints per-field provenance
- prints per-field confidence
- when `ota.yaml` already exists, text output leads with the existing-contract comparison and drift review before the inferred contract details
- existing-contract add/update lines include the detector source and confidence for the proposed value
- when `ota.yaml` already exists and only drift is present, text output says there are no additive detected changes and points users at merge vs rewrite review
- does not write anything

Contract preview behavior:

- `ota detect --contract` prints the exact starter contract that `ota init` would write
- `ota detect --contract` omits annotations and comparison output
- `ota detect --contract` is text output only

Current write behavior:

- `ota detect --write` writes using only `high` confidence fields
- `ota detect --write` remains conservative even when `ota init` can write a valid starter
- validates the generated contract before writing
- refuses to overwrite an existing `ota.yaml`

Current merge-preview behavior:

- `ota detect --merge --dry-run` is a review-only mode
- it requires an existing `ota.yaml`
- it does not write
- it reuses the comparison preview instead of applying changes, including stale contract fields that no longer match repo reality
- JSON comparison entries carry stable ownership/provenance labels; add/update entries also carry direct detector source and confidence
- task drift in text output is grouped by task name instead of raw dotted paths
- when both kinds are present, task drift splits command removals from `safe_for_agent` removals
- task drift text starts with a compact summary showing affected task count and removal counts by kind
- with `--concise`, task drift collapses to one line per affected task with removal counts instead of listing every command
- there is no standalone `ota drift` command yet; drift review stays on `ota detect --merge --dry-run`, and operator-facing trust/readiness drift stays on `ota doctor`

Current merge-write behavior:

- `ota detect --merge` requires an existing `ota.yaml`
- it applies only `high` confidence missing fields
- `ota detect --merge --apply FIELD` applies only the selected high-confidence detected changes and leaves the rest of `ota.yaml` unchanged
- `ota detect --merge --apply-all` applies all eligible high-confidence detected changes and leaves the rest of `ota.yaml` unchanged
- it does not overwrite conflicting existing values
- it validates the merged contract before writing
- it is additive only in the current implementation
- on mixed repos, lower-confidence fields can still appear in `comparison` without being written
- if nothing eligible can be added, it returns success with `written: false` and leaves `ota.yaml` unchanged

Current rewrite behavior:

- `ota detect --rewrite` targets existing contracts only and is destructive
- `ota detect --rewrite --dry-run` previews replacement without writing
- `ota detect --rewrite --yes` replaces the existing `ota.yaml` with the regenerated detected contract
- rewrite creates a timestamped backup file (`ota.yaml.bak-<timestamp>`) before writing
- rewrite validates the regenerated contract before replacing the existing file

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
- verifier-style inferred tasks (for example `test`, `lint`, `typecheck`, `check`, `verify`, `fmt`) are marked with `safe_for_agent: true`; other inferred tasks stay unsafe-by-default
- `Pipfile` can contribute `python` runtime inference and `pipenv` tool inference conservatively
- `uv.lock` can contribute `uv` tool inference conservatively
- `requirements.txt` can contribute `pip` tool inference conservatively
- `setup.cfg` can contribute project name and `python` runtime inference conservatively
- for example, `.nvmrc`, `.node-version`, `.python-version`, `.java-version`, `.sdkmanrc`, `go.mod`, `rust-toolchain.toml`, and `rust-toolchain` win over conflicting `.tool-versions` runtime values

Write behavior:

- `ota detect --write` writes only `high` confidence fields
- validates the projected contract before writing
- refuses to overwrite an existing `ota.yaml`
- when `ota.yaml` already exists, points the user at `ota detect --merge --dry-run` and `ota detect --rewrite --dry-run`
- fails if the high-confidence projection is not sufficient
- JSON failure responses can include `next` when ota can point to one safe follow-up command

This is intentionally conservative. Review mode comes first, write mode second.

## `ota workspace init`

Create a starter workspace contract from existing repo contracts.

```bash
ota workspace init [PATH]
ota workspace init --json [PATH]
```

Current behavior:

- infers workspace repos by scanning common local repo roots (top-level plus containers like `apps/`, `services/`, `repos/`, `packages/`)
- includes only repos that already have `ota.yaml`
- skips candidate repos that do not yet have `ota.yaml`
- `ota workspace init` writes `ota.workspace.yaml` by default
- `ota workspace init --bootstrap` can auto-provision missing repo contracts from detected repo signals before writing `ota.workspace.yaml`
- `--write` remains a compatibility alias for the write path
- writes `ota.workspace.yaml`
- refuses to overwrite an existing `ota.workspace.yaml`
- when no repos are available to bootstrap, points to `ota init <repo-path>`, `ota detect --dry-run <repo-path>`, and `ota workspace detect --write` or `ota workspace init` after repo contracts exist
- when overwrite is refused, points to `ota workspace validate` and `ota workspace doctor`
- supports JSON for machine-readable write outcomes

Text output:

- write: `WORKSPACE INIT WRITE <path>`

JSON output:

- success: `ok`, `path`, `written`, `mode`, `config`, `included`, `missing_contract`
- failure: `ok`, `path`, `written`, `mode`, `error`, optional `next`

## `ota workspace detect`

Infer workspace contract shape and additive merge candidates.

```bash
ota workspace detect [PATH]
ota workspace detect --write [PATH]
ota workspace detect --dry-run [PATH]
ota workspace detect --merge [PATH]
ota workspace detect --merge --dry-run [PATH]
ota workspace detect --rewrite --dry-run [PATH]
ota workspace detect --rewrite --yes [PATH]
ota workspace detect --json [PATH]
```

Current behavior:

- infers workspace repos by scanning common local repo roots (top-level plus containers like `apps/`, `services/`, `repos/`, `packages/`)
- includes only repos that already have `ota.yaml`
- skips candidate repos that do not yet have `ota.yaml`
- default mode is preview
- `--write` writes `ota.workspace.yaml` only for first contract creation
- `--merge` requires an existing `ota.workspace.yaml` and adds only missing discovered repo entries under `repos`
- merge is additive-only and does not overwrite existing repo entries
- `--rewrite --dry-run` previews full replacement of an existing `ota.workspace.yaml`
- `--rewrite --yes` fully replaces existing `ota.workspace.yaml` with regenerated detected workspace contract
- rewrite creates a timestamped backup file (`ota.workspace.yaml.bak-<timestamp>`) before writing
- when no repo contracts are found, points to `ota init <repo-path>`, `ota detect --dry-run <repo-path>`, and `ota workspace detect --write` or `ota workspace init` after repo contracts exist
- supports JSON for machine-readable preview/write outcomes

## `ota workspace validate`

Validate an ota workspace contract.

```bash
ota workspace validate [PATH]
ota workspace validate --json [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or an explicit directory boundary
- parses the workspace contract
- validates the workspace shape
- validates each present referenced repo contract through the workspace contract
- allows missing repo paths only when `repos.<name>.source` is declared

Text output:

- header: `WORKSPACE VALIDATE <path>`
- success: `VALID` plus next steps into `ota workspace doctor`, `ota workspace up`, and `ota workspace tasks`
- failure: validation or load error text

JSON output:

- success: `ok`, `path`, `summary.error_count`
- failure: `ok`, `path`, `summary.error_count`, and either `errors` or `error`

## `ota workspace tasks`

List workspace repo tasks in dependency order.

```bash
ota workspace tasks [PATH]
ota workspace tasks --json [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or an explicit directory boundary
- validates workspace shape and present repo contracts
- preserves workspace dependency order in output
- lists task declarations for each acquired repo contract, including task descriptions and declared `after_success`, `after_failure`, and `after_always` hook relationships when the repo contract declares them
- reports non-acquired repos with `acquired: false` and empty task lists
- does not execute tasks

Text output:

- header: `WORKSPACE TASKS <path>`
- each repo includes required/optional status, acquisition status, dependency list, and task summaries

JSON output:

- `ok`
- `path`
- `summary` with `repo_count`, `acquired_count`, and `task_count`
- `repos`
- each repo includes: `name`, `path`, `contract_path`, `required`, `acquired`, `depends_on`, `tasks`
- each task includes: `name`, `kind`, optional `description`, one execution body field (`run` or `script`), `depends_on`, `after_success`, `after_failure`, `after_always`

## `ota workspace list`

List workspace repos, contract presence, and lightweight readiness status without running workspace doctor.

```bash
ota workspace list [PATH]
ota workspace list --status ready [PATH]
ota workspace list --status not-ready [PATH]
ota workspace list --repo <name> [PATH]
ota workspace list --json [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace shape for deterministic repo ordering
- lists all declared repos (or filters by `--repo` / `--status`)
- reports acquisition state per repo (`ACQUIRED` vs `NOT ACQUIRED`)
- reports lightweight readiness status per repo (`READY` vs `NOT READY`)
- shows execution metadata and env provenance when the repo contract declares it
- reports contract presence per repo (`contract_present`)
- for missing contracts in text output, embeds a repo-specific setup hint using `ota init <repo-path>`

Text output:

- header: `WORKSPACE LIST <path>`
- each repo includes required/optional status, acquisition status, readiness status, path, contract path state, dependencies, and execution metadata with env provenance when present
- each repo shows acquisition on the summary line, readiness on a dedicated `Status:` line, and execution metadata in a compact `Execution:` block with env provenance when present

JSON output:

- `ok`
- `path`
- `summary` mirroring the receipt summary with `error_count`, `warn_count`, `info_count`, and `step_count`
- `repos`
- each repo includes: `name`, `path`, `contract_path`, `contract_present`, `required`, `acquired`, `status`, `depends_on`

## `ota workspace execution plan`

Inspect the resolved execution context for each workspace repo without running anything.

```bash
ota workspace execution plan [PATH]
ota workspace execution plan --json [PATH]
ota workspace execution plan --repo api [PATH]
ota workspace execution plan --mode container --ephemeral [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure and keeps repo ordering deterministic
- reuses the same per-repo backend validation boundary as `ota execution plan`
- reports one execution plan per selected workspace repo
- supports `--repo` filtering for focused inspection
- supports `--mode`, `--lifecycle`, and `--ephemeral` overrides across the selected repos
- fails the command when any selected repo cannot produce a runnable execution plan
- still preserves each repo’s required/optional declaration in the report instead of flattening workspace metadata
- never mutates repo state

Text output:

- header: `WORKSPACE EXECUTION PLAN <path>`
- status line: `READY` or `NOT READY`
- optional `Overrides` section when backend or lifecycle is forced
- each repo includes required/optional status, path, contract path, acquired state, and either resolved execution details or an honest `Why` / `Next`
- when a repo contract loads, the report also includes the compact `Contract` block and declared `Execution` block for that repo
- a final `Summary` block reports resolved and unresolved repo counts

JSON output:

- `ok`
- `path`
- `mode: "execution-plan"`
- `summary` with `repo_count`, `resolved_count`, `unresolved_count`, `required_unresolved_count`, `not_acquired_count`, and `missing_contract_count`
- `repos`
- each repo includes: `name`, `path`, `contract_path`, `required`, `acquired`, `status`, optional `contract_identity`, optional `declared_execution`, optional `resolved`, optional `error`, and optional `next`
- `overrides` appears only when execution overrides are supplied

Current non-goals:

- running setup or task commands
- provisioning missing repos automatically
- hiding unrunnable execution choices behind a synthetic success state
- inventing one workspace-wide execution backend when repo contracts disagree

## `ota workspace run`

Run one task across workspace repos in dependency order.

```bash
ota workspace run <task> [PATH]
ota workspace run <task> --json [PATH]
ota workspace run <task> --jobs 4 [PATH]
ota workspace run <task> --stream [PATH]
ota workspace run <task> [PATH] --base-url http://localhost:8080
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
- task inputs are declared in `tasks.<name>.inputs` and are passed as `--kebab-case value` flags
- task inputs are exposed to each repo task process as `OTA_INPUT_<NAME>` env variables
- `default` values are applied when the caller omits an input
- `required: true` makes an input mandatory unless a default exists
- `allowed` limits the accepted values for that input
- task inputs only apply to the targeted repo task, not its dependencies
- if every declared input has a default, you can omit all input flags

Example:

```yaml
tasks:
  api-automation-tests:
    inputs:
      base_url:
        default: http://localhost:8080
      suite_mode:
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
ota workspace run api-automation-tests --base-url http://localhost:8080 --suite-mode contract-drift
ota workspace run version:bump --version 0.2.0
```

- prints a summary in text output, emits an execution receipt when `--receipt` is set, and a `receipt` object in JSON output
- the workspace receipt includes additive `receipt.contract_identity` with workspace name/type and compact workspace repo/policy counts

Text output:

- header: `WORKSPACE RUN <task> <path>`
- status line: `READY` or `NOT READY`
- per-repo status includes `required/optional`, task name, findings, and optional exit details

JSON output:

- `ok`
- `path`
- `task`
- `summary`
- `receipt`
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
- summary roll-up includes repo verdict and agent verdict before the `Overview` count block at the
  bottom of the report
- each repo includes required/optional status, contract path, and findings rendered through the
  shared grouped finding UX
- with `--concise`, repo `Path`/`Contract` and finding `Why` detail are omitted; summary + `Next` remain

JSON output:

- `ok`
- `path`
- `summary` with `repo_count`, `ready_count`, `not_ready_count`, `error_count`, `warn_count`, and `info_count`
- `repos`

## `ota workspace doctor`

Diagnose workspace repo readiness from an ota workspace contract.

```bash
ota workspace doctor [PATH]
ota workspace doctor --json [PATH]
ota workspace doctor --jobs 4 [PATH]
ota workspace doctor --repo <name> [PATH]
ota workspace doctor --status ready|not-ready [PATH]
ota workspace doctor --severity error|warn|info [PATH]
ota workspace doctor --stream [PATH]
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
- supports repo/status/severity filtering for focused diagnosis views
- `--stream` is text-only and emits repo completion updates while the final report is being built

Text output:

- header: `WORKSPACE DOCTOR <path>`
- status line: `READY` or `NOT READY`
- when the workspace is blocked, a primary blocker appears immediately under the readiness status
- summary roll-up includes repo verdict and agent verdict before the `Overview` count block at the
  bottom of the report
- each repo includes required/optional status, contract path, and findings rendered through the
  shared grouped finding UX
- with `--concise`, repo `Path`/`Contract` and finding `Why` detail are omitted; summary + `Next` remain

JSON output:

- `ok`
- `path`
- `summary` mirroring the workspace doctor roll-up with `repo_count`, `ready_count`, `not_ready_count`, `error_count`, `warn_count`, and `info_count`
- repo execution metadata may include env provenance for inherited workspace policy values
- `repos`

Current non-goals:

- passing a repo URL directly on the CLI without a workspace contract

## `ota workspace explain`

Explain workspace readiness findings as an ordered remediation plan.

```bash
ota workspace explain [PATH]
ota workspace explain --json [PATH]
ota workspace explain --repo api [PATH]
```

Current behavior:

- diagnoses the workspace first
- turns each repo finding into ordered remediation steps
- stays read-only and deterministic
- prints a summary with repo and step counts at the end

Text output:

- one section per workspace repo
- ordered remediation `Plan` steps under each repo
- an `Overview` count block at the end

JSON output:

- success: `ok`, `path`, `summary`, `repos`
- failure: `ok`, `path`, and either `errors` or `error`

The `summary` object on success mirrors the top-level receipt summary and includes
`error_count`, `warn_count`, `info_count`, and `step_count`.

## `ota workspace up`

Prepare every repo in an ota workspace contract.

```bash
ota workspace up [PATH]
ota workspace up --json [PATH]
ota workspace up --jobs 4 [PATH]
ota workspace up --quiet [PATH]
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
- `--quiet` suppresses live progress output and prints only the final workspace report
- optional repo failures do not fail the overall workspace result
- defaults to sequential execution because `--jobs` defaults to `1`
- `--stream` opts into raw live child process output instead of buffered per-repo output
- `--stream` is text-only and currently requires `--jobs 1`
- does not pull or update repos that already exist locally
- prints a summary in text output, emits an execution receipt when `--receipt` is set, and a `receipt` object in JSON output
- the workspace receipt includes additive `receipt.contract_identity` with workspace name/type and compact workspace repo/policy counts

Text output:

- header: `WORKSPACE UP <path>`
- status line: `READY` or `NOT READY`
- each repo includes required/optional status, phase, findings, exit details, and captured stdout/stderr when present

JSON output:

- `ok`
- `path`
- `summary` mirroring the workspace doctor roll-up with `repo_count`, `ready_count`, `not_ready_count`, `error_count`, `warn_count`, and `info_count`
- `repos`

Current non-goals:

- passing a repo URL directly on the CLI without a workspace contract
- host or workstation provisioning beyond workspace bootstrap plus repo readiness
- GitHub API integration or non-git acquisition modes

## `ota workspace refresh`

Refresh existing repos in an ota workspace contract without cloning missing ones.

```bash
ota workspace refresh [PATH]
ota workspace refresh --json [PATH]
ota workspace refresh --jobs 4 [PATH]
ota workspace refresh --dry-run [PATH]
ota workspace refresh --quiet [PATH]
ota workspace refresh --stream [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure
- refreshes repos that already exist locally and have a declared source
- leaves missing repos alone so `ota workspace up` remains the bootstrap path
- can refresh independent repos concurrently when `--jobs` is greater than `1`
- respects declared workspace repo dependency order
- blocks downstream repos when a dependency does not become ready
- aggregates per-repo status, phase, findings, and exit details
- captures repo child stdout and stderr per repo so text and JSON output remain deterministic
- emits live repo progress on stderr in text mode so users can see queued/running/completed state while buffered output is still being collected
- `--quiet` suppresses live progress output and prints only the final workspace report
- optional repo failures do not fail the overall workspace result
- defaults to sequential execution because `--jobs` defaults to `1`
- `--stream` opts into raw live child process output instead of buffered per-repo output
- `--stream` is text-only and currently requires `--jobs 1`
- `--dry-run` previews the refresh commands without changing repo state
- `--force` force-fetches and hard-resets refreshed repos to the declared source or `--ref` override
- `--prune` prunes stale remote-tracking refs during refresh
- `--ref <branch|tag|sha>` overrides the source ref used for refresh
- prints a summary in text output, emits an execution receipt when `--receipt` is set, and a `receipt` object in JSON output

Text output:

- header: `WORKSPACE REFRESH <path>` or `WORKSPACE REFRESH PREVIEW <path>` for `--dry-run`
- preview mode prints `Mode: dry-run (no write)`
- status line: `READY`, `NOT READY`, or `NOT ACQUIRED` for normal refresh; preview mode does not claim readiness
- each repo includes required/optional status, phase, findings, exit details, and captured stdout/stderr when present

JSON output:

- `ok`
- `path`
- `mode`: `refresh` for normal refresh, `preview` for `--dry-run`
- `summary` mirroring the workspace doctor roll-up with `repo_count`, `ready_count`, `not_ready_count`, `error_count`, `warn_count`, and `info_count`
- `repos`

Current non-goals:

- cloning missing repos
- passing a repo URL directly on the CLI without a workspace contract
- host or workstation provisioning beyond workspace bootstrap plus repo readiness
- GitHub API integration or non-git acquisition modes

## `ota workspace diff`

Compare local workspace repos against their declared source state without mutating anything.

```bash
ota workspace diff [PATH]
ota workspace diff --json [PATH]
ota workspace diff --jobs 4 [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure
- compares each acquired repo’s local git state against the declared source ref or upstream branch
- reports `MATCH` when a repo is clean and aligned, `DIRTY` when the worktree has local changes, `DIFFERENT` when commit counts differ, `MISSING` when the repo is absent, and `UNRESOLVED` when git state cannot be compared safely
- can compare independent repos concurrently when `--jobs` is greater than `1`
- never mutates repo state
- `--json` returns a workspace diff roll-up with `mode: "diff"`
- differences do not fail the command; the command succeeds and surfaces drift in the report

Current non-goals:

- refreshing or mutating repo state
- cloning missing repos automatically

## `ota workspace status`

Compact workspace status combines readiness and drift without mutating repo state.

```bash
ota workspace status [PATH]
ota workspace status --json [PATH]
ota workspace status --jobs 4 [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure
- reads repo readiness and local git drift for each workspace repo
- reports readiness and drift together so you can scan one operational summary
- can compare independent repos concurrently when `--jobs` is greater than `1`
- never mutates repo state
- `--json` returns a workspace status roll-up with `mode: "status"`
- readiness findings and drift findings are surfaced in the same report

Text output:

- header: `WORKSPACE STATUS <path>`
- each repo includes required/optional status, combined readiness and drift status, path, contract path, source metadata, and local git comparison details when present
- a summary block reports readiness and drift roll-ups in one place

JSON output:

- `ok`
- `path`
- `mode: "status"`
- `summary` with readiness counts and drift counts
- `repos`
- each repo includes: `name`, `path`, `contract_path`, `required`, `acquired`, `ready`, `readiness_status`, `drift_status`, `branch`, `head`, `target_ref`, `ahead`, `behind`, `dirty`, and `findings`

Current non-goals:

- mutating repo state
- cloning missing repos automatically
- cross-repo dependency scheduling
- passing a repo URL directly on the CLI without a workspace contract
- host or workstation provisioning
- a workspace-only bootstrap engine that bypasses repo contracts
- GitHub API integration or non-git acquisition modes

## `ota workspace receipt`

Capture the current workspace scan as a read-only receipt artifact for CI or archival use.

```bash
ota workspace receipt [PATH]
ota workspace receipt --json [PATH]
ota workspace receipt --jobs 4 [PATH]
ota workspace receipt --archive [PATH]
```

Current behavior:

- resolves `ota.workspace.yaml` using `--file`, `OTA_FILE`, or upward discovery
- validates workspace structure
- reads repo readiness and local git drift for each workspace repo without mutating anything
- captures the combined workspace state as an execution receipt with one step per repo
- can inspect independent repos concurrently when `--jobs` is greater than `1`
- never clones, fetches, resets, or writes repo state
- `--json` returns a workspace receipt roll-up with `mode: "receipt"`
- the workspace receipt includes additive `receipt.contract_identity` with workspace name/type and compact workspace repo/policy counts
- `--archive` writes the JSON receipt to `.ota/receipts` and keeps the newest 50 archives
- the receipt records the same readiness, drift, and findings scan so CI or agents can archive it deterministically

Text output:

- header: `WORKSPACE RECEIPT <path>`
- each receipt step shows the repo name, readiness status, and drift status
- the summary block mirrors the execution receipt counts

JSON output:

- `ok`
- `path`
- `mode: "receipt"`
- `archive_path` (when `--archive` is set)
- `summary` mirroring the receipt summary with `repo_count`, `ready_count`, `not_ready_count`, `error_count`, `warn_count`, `info_count`, and `step_count`
- `receipt`
- `repos`

Current non-goals:

- mutating repo state
- cloning missing repos automatically
- cross-repo dependency scheduling
- passing a repo URL directly on the CLI without a workspace contract
- host or workstation provisioning
