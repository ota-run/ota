# Ota Command Reference

This document describes the current shipped CLI surface.

For machine-readable command contracts, see [json-output-reference.md](json-output-reference.md).

## Global

```bash
ota --help
ota --version
ota --debug <command>
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

When a command accepts a `PATH`, it may be either:

- a direct path to `ota.yaml`
- a directory containing `ota.yaml`

`ota detect` is different. Its `PATH` is a repo root to inspect.

## Current exit semantics

- `0`: success, ready state, or warning-only diagnosis
- `1`: invalid contract, blocking readiness issue, protected write failure, or general command failure
- `2`: CLI usage or argument parsing error
- `ota run`: preserves child task exit codes on task failure
- `ota up`: preserves service-start and setup task exit codes when those commands fail

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
```

Current behavior:

- loads `ota.yaml`
- parses the contract
- applies semantic validation
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
```

Current behavior:

- validates the contract first
- prints tasks in deterministic order
- includes task metadata when present
- shows whether each task uses `run` or `script`

Text output:

- header: `TASKS <path>`
- each task may include `kind`, `category`, `depends_on`, and `safe_for_agent`

JSON output:

- success: `ok`, `path`, `tasks`
- failure: `ok`, `path`, and either `errors` or `error`

## `ota run`

Run a validated task.

```bash
ota run <task> [PATH]
```

Current behavior:

- validates the contract first
- resolves task dependencies before execution
- executes either `run` or `script`
- runs in the contract directory
- applies configured environment values
- prints an advisory stderr note when `execution.lifecycle: ephemeral` is declared
- returns the child process exit code

Use this when the contract is already the source of truth and you want deterministic task execution.

## `ota doctor`

Diagnose repo readiness from a validated contract.

```bash
ota doctor [PATH]
ota doctor --json [PATH]
```

Current behavior:

- validates the contract first
- checks configured env requirements
- checks runtime and tool presence on `PATH`
- runs declared service healthchecks
- warns when a required service has no healthcheck, because readiness cannot be verified
- warns when `execution.lifecycle: ephemeral` is declared, because V1 does not provide isolated temporary execution
- runs configured checks
- orders findings by severity
- prints the reason and next action for each finding

Text output:

- header: `DOCTOR <path>`
- status line: `READY` or `NOT READY`

JSON output:

- `ok`
- `path`
- `findings`

Warnings can still produce `READY`. Errors produce `NOT READY`.

## `ota init`

Create a starter Ota contract for a repo that does not yet have one.

```bash
ota init [PATH]
ota init --write [PATH]
ota init --json [PATH]
```

Current behavior:

- inspects the repo using the detection engine
- defaults to review mode and does not write
- writes only when `--write` is provided
- refuses to run when `ota.yaml` already exists
- can initialize both detected repos and blank repos

Modes:

- `blank`: starter contract derived from minimal repo context
- `detected`: starter contract derived from detected repo signals

Text output:

- dry-run header: `INIT <path>`
- write success: `WROTE <path>`
- includes `Mode: blank` or `Mode: detected`
- includes inferred-field annotations with source and confidence

JSON output:

- `ok`
- `path`
- `written`
- `mode`
- `config`
- `inferred`

## `ota check`

Run configured checks from a validated contract.

```bash
ota check [PATH]
ota check --json [PATH]
```

Current behavior:

- validates the contract first
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

## `ota up`

Prepare a repo for use with minimal prior knowledge.

```bash
ota up [PATH]
ota up --json [PATH]
```

Current behavior:

- validates the contract first
- runs blocking precondition checks
- runs explicit `services.<name>.start` commands for required services before setup
- verifies required service healthchecks before setup and stops in the `services` phase when readiness still fails
- runs the `setup` task if one exists
- re-runs readiness diagnosis
- remains shell-native even when `execution.lifecycle: ephemeral` is declared
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

## `ota detect`

Infer a starting contract from repo state.

```bash
ota detect --dry-run [PATH]
ota detect --json --dry-run [PATH]
ota detect [PATH]
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

For Docker Compose service inference, Ota currently derives:

- `provider` at high confidence
- `start` / `stop` at medium confidence
- declared `healthcheck.test` at medium confidence

Dry-run behavior:

- prints a candidate `ota.yaml`
- prints per-field provenance
- prints per-field confidence
- does not write anything

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
- for example, `.nvmrc`, `.node-version`, `.python-version`, and `go.mod` win over conflicting `.tool-versions` runtime values

Write behavior:

- writes only `high` confidence fields
- validates the projected contract before writing
- refuses to overwrite an existing `ota.yaml`
- fails if the high-confidence projection is not sufficient

This is intentionally conservative. Review mode comes first, write mode second.
