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

Use `project` for stable repo identity only. Churn-heavy descriptive fields such as `author`,
`created_at`, or publishing metadata should live under `metadata` unless Ota grows a dedicated
package or distribution contract later.

## `workspace`

Optional.

Current V3 support is repo-level monorepo declaration:

```yaml
workspace:
  type: monorepo
  members:
    - api
    - web
```

Fields:

- `type`: currently only `monorepo`
- `members`: required, non-empty list of member paths relative to the root contract directory

Current behavior:

- the root contract remains a normal `ota.yaml`
- member contracts live at `<member>/ota.yaml`
- member contracts inherit the root contract and override only what they declare
- member contracts must not declare a top-level `workspace` block
- repo commands can target a member with `--member <name>`
- repo commands run from inside a member directory automatically load the merged member contract
- current member targeting expects the named member to be declared in `workspace.members`

## `execution`

Optional.

```yaml
execution:
  preferred: native
  lifecycle: persistent
  supported:
    - native
    - container
```

Supported backend values:

- `native`
- `container`
- `remote`

Supported lifecycle values:

- `persistent`
- `ephemeral`

Current validation rule:

- if `preferred` is set and `supported` is not empty, `preferred` must also appear in `supported`

Current implementation only executes tasks natively.

Current lifecycle meaning:

- `persistent`: current default behavior
- `ephemeral`: advisory only in V1; Ota still executes in the current shell environment and does not provide isolated temporary environments or automatic cleanup

Current command behavior:

- `ota doctor` warns when `ephemeral` is declared
- `ota run` prints an advisory lifecycle note on stderr
- `ota up` remains shell-native and does not provide isolation

## `services`

Optional.

```yaml
services:
  api:
    required: true
    provider: docker-compose
    start: docker compose up -d api
    stop: docker compose stop api
    healthcheck: curl -fsS http://localhost:3000/health
    depends_on:
      - postgres
    timeout: 5000
  postgres:
    required: true
    provider: docker-compose
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -h localhost -p 5432
```

Fields:

- `required`: optional boolean
- `provider`: optional string
- `start`: optional string
- `stop`: optional string
- `healthcheck`: optional string
- `depends_on`: optional list of service names
- `timeout`: optional healthcheck timeout in milliseconds

Current behavior:

- services are part of the accepted V1 contract surface
- service declarations must include at least one actionable field: `provider`, `start`, `stop`, or `healthcheck`
- unknown `depends_on` references are invalid
- service dependency cycles are invalid
- `timeout` must be greater than zero when set
- `ota doctor` runs declared service `healthcheck` commands
- failed required service healthchecks are blocking errors
- failed optional service healthchecks are warnings
- timed out required service healthchecks are blocking errors
- timed out optional service healthchecks are warnings
- required services without a `healthcheck` produce a warning because readiness cannot be verified yet
- `ota up` starts required services, and required-service dependencies, in declared dependency order before `setup`
- `ota up` treats each required service healthcheck as a readiness gate before moving on to dependents
- Ota still does not provide deep service orchestration beyond explicit contract commands

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
    script: |
      export APP_ENV=development
      pnpm dev
    depends_on:
      - setup
  bootstrap:
    run: ./scripts/bootstrap.sh
    variants:
      - when:
          os: windows
        run: .\scripts\bootstrap.ps1
```

Fields:

- `description`: optional string
- `category`: optional string
- `run`: optional string for a single shell-compatible command
- `script`: optional string for an inline multiline shell script
- `variants`: optional list of conditional task executions
- `depends_on`: optional list of task names
- `safe_for_agent`: optional boolean

Variant fields:

- `when.os`: required for each current variant entry; supported values are `linux`, `macos`, and `windows`
- exactly one of `run` or `script`

Rules:

- task names must not be empty
- tasks must declare a default `run` or `script`, or at least one variant
- `run` must be non-empty when present
- `script` must be non-empty when present
- variant entries must declare `when.os`
- variant entries must declare exactly one of `run` or `script`
- duplicate variants for the same `when.os` are rejected
- dependency references must resolve to known tasks
- task dependency cycles are rejected

Current execution model:

- `run` and `script` are shell-compatible execution forms
- when variants are declared, Ota resolves the best matching `when.os` entry first and falls back to the default execution
- richer non-shell executors are intentionally out of V1 scope
- future direction is tracked in the product spec

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
  author: Ota Maintainers
  created_at: 2026-03-23
```

This is an open map for extra repo-specific values.

## Full example

See:

- [../../examples/full-contract/ota.yaml](../../examples/full-contract/ota.yaml)
