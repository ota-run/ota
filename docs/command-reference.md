# Ota Command Reference

This document describes the current shipped CLI surface.

## Global

```bash
ota --help
ota --version
```

Ota currently ships these commands:

- `ota validate`
- `ota tasks`
- `ota run <task>`
- `ota doctor`
- `ota up`
- `ota detect`

When a command accepts a `PATH`, it may be either:

- a direct path to `ota.yaml`
- a directory containing `ota.yaml`

`ota detect` is different. Its `PATH` is a repo root to inspect.

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

Text output:

- header: `TASKS <path>`
- each task may include `category`, `depends_on`, and `safe_for_agent`

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
- runs in the contract directory
- applies configured environment values
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

## `ota up`

Prepare a repo for use with minimal prior knowledge.

```bash
ota up [PATH]
```

Current behavior:

- validates the contract first
- runs blocking precondition checks
- runs the `setup` task if one exists
- re-runs readiness diagnosis
- returns `READY` or `NOT READY`

This is the onboarding command. It is intentionally narrower than a general-purpose environment orchestrator.

## `ota detect`

Infer a starting contract from repo state.

```bash
ota detect --dry-run [PATH]
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

Dry-run behavior:

- prints a candidate `ota.yaml`
- prints per-field provenance
- prints per-field confidence
- does not write anything

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
