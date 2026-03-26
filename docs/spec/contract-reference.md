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

## Primary sections at a glance

- `version`: schema version for the contract itself. Today this is `1`.
- `project`: stable repo identity and high-level classification.
- `runtimes`: required language/runtime versions for the repo to be runnable.
- `tools`: external CLI and tool dependencies the repo expects on PATH.
- `env`: required environment variables, defaults, and allowed values.
- `services`: supporting services such as databases, queues, or local infra.
- `checks`: explicit preconditions and health checks that should pass.
- `tasks`: named commands humans and agents can run deterministically.
- `execution`: where tasks run, such as native, container, or remote backends.
- `agent`: agent-safe task hints and writable-path guidance.
- `workspace`: monorepo root/member mapping for multi-repo orchestration.

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

Top-level `extensions` is intentionally not accepted in the shipped V4 parser/validator yet.
For the staged execution boundary and V6 target contract, see
[extension-execution-boundary.md](extension-execution-boundary.md).

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

Readiness note:

- set `project.type` to library-style values such as `sdk` or `library` when the repo is not meant
  to expose runnable entrypoint tasks; in that case, `ota doctor` treats missing `tasks` as a warning
  instead of a blocking error.

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
- validating the root monorepo contract also validates every declared merged member contract
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
    - remote
  backends:
    container:
      image: ghcr.io/ota/dev:latest
    remote:
      provider: ssh
      target: sandbox-dev
      cwd: /workspace
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
- `execution.preferred: container` requires `execution.backends.container.image`
- `execution.preferred: remote` requires `execution.backends.remote.provider`
- `execution.preferred: remote` requires `execution.backends.remote.target`
- remote target guidance by provider:
- `daytona`: `sandbox-dev`
- `ssh` / `tsh`: `user@host`
- `kubectl`: `pod/ota-dev`

Current implementation:

- `ota run` now supports `execution.preferred: container` when `execution.backends.container.image` is configured
- the first container path uses the local `docker` CLI, mounts the effective contract directory at `/workspace`, and runs task bodies with `sh -lc`
- `ota up` now runs the `setup` task through the same configured execution backend when one exists
- `ota run` now supports remote execution when `execution.backends.remote.provider` and `execution.backends.remote.target` are configured
- current shipped remote providers are `daytona`, `ssh`, `tsh`, and `kubectl`
- the current remote path shells out to the local provider CLI with optional `execution.backends.remote.cwd`
- `ota up` runs its `setup` task through the same remote backend path when remote execution is preferred or explicitly overridden
- remote provisioning and remote workspace selection are still out of scope today

Current lifecycle meaning:

- `persistent`: when `execution.preferred: container` is configured, `ota run` and the `setup` task inside `ota up` reuse a persistent named container for the effective contract directory
- `ephemeral`: when `execution.preferred: container` is configured, `ota run` and the `setup` task inside `ota up` use a fresh `docker run --rm` container for each invocation
- outside backend-backed task execution, such as service commands, healthchecks, and diagnosis, lifecycle remains advisory today

Current command behavior:

- `ota doctor` warns when `ephemeral` is declared, and clarifies that container-backed isolation currently applies to `ota run` and the `setup` task inside `ota up`, but not the full repo lifecycle
- `ota run` prints a lifecycle note on stderr and can execute via the configured container backend
- `ota run` can also override backend and lifecycle for one invocation with `--backend` and `--lifecycle`
- `ota up` can also override backend and lifecycle for the `setup` phase with `--backend` and `--lifecycle`
- `ota up` prints the same lifecycle note on stderr when its `setup` phase uses backend-backed execution
- `ota clean` removes persistent container state for repos using `execution.preferred: container` with `lifecycle: persistent`
- `ota clean` currently has no remote cleanup action; remote-backed repos report `NO CLEANUP NEEDED` today
- `ota doctor` checks the required backend CLI for the preferred execution backend and reports unsupported shipped remote providers early
- `ota doctor` warns on suspicious remote target shape (`ssh`/`tsh` without `user@host`, `kubectl` not starting `pod/`)
- `ota up` still runs service start commands, service healthchecks, and diagnosis on the host today

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
- for `provider: docker-compose`, `ota doctor` runs the healthcheck inside the service container via `docker compose exec -T <service> sh -lc <healthcheck>`
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
  java:
    version: "21"
    distribution: temurin
  node:
    version: "22"
    provider: volta
```

Rules:

- runtime names must not be empty
- versions must not be empty
- `provider`, when set, must not be empty
- `distribution`, when set, must not be empty

Version syntax examples:

- `8` is an example of an exact required version
- `>=8` is an example of accepting any version at or above `8`
- `^8` is an example of a compatible version range, usually the same major line
- Ota compares numeric version parts and accepts common prefixes such as `go1.24.2` or `v1.24.2`
- use `>=` when you want to accept newer versions explicitly
- use `^` when you want to express compatibility rather than a strict floor

Runtime detail fields:

- `provider`: optional runtime manager or provisioning source hint such as `volta`
- `distribution`: optional runtime flavor where version alone is not sufficient, especially Java
  distributions such as `temurin`, `corretto`, `graalvm`, `oracle`, or `zulu`

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
- some tool keys map to different executables; for example, `tools.maven` is checked via `mvn`

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
    depends_on:
      - setup
    script: pnpm dev
  dev_clean:
    depends_on:
      - setup
      - reset_db
    script: pnpm dev
  reset_db:
    run: docker compose down -v
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

Use cases:

- use `run` for one command you would normally type in a shell
- use `script` when the task needs multiple lines, shell setup, or cleanup steps

Example script forms:

```yaml
tasks:
  build:
    run: mvn package
  dev:
    script: |
      lsof -ti:8080 | xargs kill -9 || true
      mvn spring-boot:run
```

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
- `depends_on` is the canonical way to reuse task steps instead of calling `ota run` from inside another task script

Current execution model:

- `run` and `script` are shell-compatible execution forms
- when variants are declared, Ota resolves the best matching `when.os` entry first and falls back to the default execution
- richer non-shell executors are intentionally out of V1 scope
- future direction is tracked in the product spec
- use task names to describe intent: `setup`, `dev`, `dev_clean`, `test`, `lint`

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
