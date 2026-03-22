# Ota Contract Reference

This document describes the current `ota.yaml` contract accepted by the shipped parser and validator.

## Minimal contract

```yaml
version: 1
project:
  name: my-repo
```

In practice, most useful contracts also define tasks, runtimes, or checks.

## Top-level fields

```yaml
version: 1
project:
  name: my-repo
  description: Optional description
  type: application
execution:
  preferred: native
  supported:
    - native
runtimes:
  node: "22"
tools:
  pnpm: "10"
env:
  OTA_ENV:
    required: true
    default: local
    allowed:
      - local
      - ci
tasks:
  setup:
    run: pnpm install
checks:
  - name: node-installed
    kind: precondition
    severity: error
    run: node --version
agent:
  entrypoint: setup
metadata:
  team: platform
```

## `version`

```yaml
version: 1
```

Current validator support is only for `1`.

## `project`

Required.

```yaml
project:
  name: ota
  description: Open repo readiness
  type: cli
```

Fields:

- `name`: required, non-empty string
- `description`: optional string
- `type`: optional string

## `execution`

Optional.

```yaml
execution:
  preferred: native
  supported:
    - native
    - container
```

Supported backend values:

- `native`
- `container`
- `remote`

Current validation rule:

- if `preferred` is set and `supported` is not empty, `preferred` must also appear in `supported`

Current implementation only executes tasks natively. The broader execution model remains part of the contract surface.

## `runtimes`

Optional.

Simple form:

```yaml
runtimes:
  node: "22"
  python: ">=3.12"
```

Detailed form:

```yaml
runtimes:
  node:
    version: "22"
    provider: volta
```

Rules:

- runtime names must not be empty
- versions must not be empty

## `tools`

Optional.

Simple form:

```yaml
tools:
  pnpm: "10"
  uv: "0.6.0"
```

Detailed form:

```yaml
tools:
  pnpm:
    version: "10"
    required: true
```

Rules:

- tool names must not be empty
- versions must not be empty
- `required` defaults to `true`

## `env`

Optional.

```yaml
env:
  OTA_ENV:
    required: true
    secret: false
    default: local
    allowed:
      - local
      - ci
```

Fields:

- `required`: optional boolean
- `secret`: optional boolean
- `default`: optional string
- `allowed`: optional list of allowed values

Current behavior:

- `run` uses `default` only when the process environment is missing the variable
- `run` rejects disallowed values
- `doctor` reports missing required vars and invalid values

## `tasks`

Optional.

```yaml
tasks:
  setup:
    description: Install dependencies
    category: setup
    run: pnpm install
    safe_for_agent: true
  dev:
    run: pnpm dev
    depends_on:
      - setup
```

Fields:

- `description`: optional string
- `category`: optional string
- `run`: required, non-empty string
- `depends_on`: optional list of task names
- `safe_for_agent`: optional boolean

Rules:

- task names must not be empty
- dependency references must resolve to known tasks
- task dependency cycles are rejected

## `checks`

Optional.

```yaml
checks:
  - name: node-installed
    kind: precondition
    severity: error
    run: node --version
    timeout: 10
```

Fields:

- `name`: required, non-empty string
- `kind`: `precondition` or `health`
- `severity`: `error`, `warn`, or `info`
- `run`: required, non-empty string
- `timeout`: optional integer in milliseconds

Current behavior:

- `up` uses preconditions before setup
- `doctor` runs configured checks and reports findings by severity
- when `timeout` is set, `doctor` fails the check if it does not finish within the configured millisecond budget

## `agent`

Optional.

```yaml
agent:
  entrypoint: setup
  default_task: test
  safe_tasks:
    - setup
    - test
  verify_after_changes:
    - test
  writable_paths:
    - src
    - docs
```

Current validation rules:

- `entrypoint` must reference a known task when set
- `default_task` must reference a known task when set
- `safe_tasks` entries must reference known tasks
- `verify_after_changes` entries must reference known tasks
- `writable_paths` entries must not be empty

Current implementation treats this as contract surface and validation input. It is not yet a full agent runtime layer.

## `metadata`

Optional.

```yaml
metadata:
  team: platform
  owner: ota
```

This is an open map for extra repo-specific values.

## Full example

See:

- [examples/full-contract/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/full-contract/ota.yaml)
