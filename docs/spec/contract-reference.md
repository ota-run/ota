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

# ota Contract Reference

This document describes the current `ota.yaml` contract accepted by the shipped parser and validator.

## Minimal contract

```yaml
version: 1
project:
  name: my-repo
```

In practice, most useful contracts also define tasks, runtimes, or checks.

## Primary sections at a glance

- `version` (required): schema version for the contract itself. Today this is `1`.
- `project` (required): stable repo identity and high-level classification.
- `runtimes`: required language/runtime versions for the repo to be runnable.
- `tools`: external CLI and tool dependencies the repo expects on PATH.
- `env`: required environment variables, defaults, allowed values, and provenance-aware resolution.
- `services`: supporting services such as databases, queues, or local infra.
- `checks`: explicit preconditions and health checks that should pass.
- `tasks`: named commands humans and agents can run deterministically.
- `execution`: where tasks run, such as native, container, or remote backends.
- `agent`: AI-agent task hints and writable-path boundaries.
- `exports`: downstream generation preferences and export metadata.
- `policies`: repo-local policy overlays and guardrails.
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
extensions:
  demo:
    kind: check_provider
    command: ota-ext-demo
    api_version: 1
runtimes:
  node: "22"
tools:
  pnpm: "10"
env:
  vars:
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

Top-level `extensions` is now recognized as adapter contract data.
Each entry is a typed adapter descriptor with `kind`, `command`, and `api_version`, plus optional
`description` and `config`.
Supported kinds today are `check_provider`, `export_provider`, and `backend_provider`.
`check_provider` is runnable with `ota extensions --run <name>` when `api_version: 1` is declared.
`export_provider` is runnable with `ota extensions --publish <name>` when `api_version: 1` is declared.
`backend_provider` is reserved for task execution backends, is discoverable in the contract, and
can be named by `execution.backends.remote.provider` when the repo wants a custom execution
backend. Runtime backend providers receive a structured JSON request on stdin and via
`OTA_BACKEND_PROVIDER_REQUEST_JSON`, then return a structured JSON response on stdout. The
request includes the extension id, kind, api version, command context, repo context path, working
directory, task name, task command, execution mode, target, cwd, and resolved environment values.
The validator requires `kind` to be one of the supported kinds, `command` to be non-empty, and
`api_version` to be greater than zero.

Real-world use cases:

- upload a release artifact bundle to an internal endpoint
- publish scan or compliance reports through one standard adapter
- expose a custom check provider, export target generator, or execution backend in a stable
  contract slot

Example:

```yaml
extensions:
  release-upload:
    kind: export_provider
    command: ota-ext-upload
    api_version: 1
    description: Upload the release bundle to the artifact endpoint
    config:
      endpoint: https://artifacts.example.com/upload
      artifact: dist/release.zip
```

```yaml
extensions:
  remote-shell:
    kind: backend_provider
    command: ota-ext-remote-shell
    api_version: 1
    description: Execute tasks through a custom remote backend
    config:
      transport: ssh
      workspace_root: /workspace
```

Use `ota extensions` to inspect this contract data. Use `ota extensions --run <name>` for
`check_provider` descriptors and `ota extensions --publish <name>` for `export_provider` descriptors.
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
`created_at`, or publishing metadata should live under `metadata` unless ota grows a dedicated
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

## `exports` and `policies`

Optional.

These sections are the current shipped overlay surfaces for downstream generation and policy-driven
guardrails.

Use them when you want the contract to describe derived outputs or repo-local policy intent without
turning those derived artifacts into a second source of execution truth.

Current guidance:

- `exports` should describe export preferences or downstream artifact intent
- `policies` should describe repo-local policy overlays and guardrails
- repo contracts must not declare `policies.env`; approved env values now live in the org policy pack under `policies.env.values`
- repo contracts must not declare `policies.version_policy`, `policies.provisioning`, or `policies.adapter_bootstrap`; approved version and provisioning authority now live in `.ota/org-policy.yaml`
- `policies.env.values` is the shipped approved-value map for environment variables in the org policy pack
- neither section should replace core readiness fields such as `tasks`, `services`, or `checks`
- newer spec drafts discuss additional policy and readiness-gate behavior; those are not part of
  the current shipped parser unless the implementation explicitly accepts them

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
  # Context model (shipped)
  default_context: app
  contexts:
    host:
      backend: native
      requirements:
        tools:
          docker: "*"
          podman: "*"
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
      requirements:
        runtimes:
          node: ">=24.14.1"
        tools:
          npm: ">=10.5"
      attachments:
        compose:
          - local
```

Named-context inheritance example (additive to existing context and shorthand support):

```yaml
execution:
  default_context: development
  contexts:
    node-base:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
      attachments:
        isolated_paths:
          - node_modules
          - .next
    development:
      extends: node-base
      container:
        resources:
          memory:
            minimum: 2GiB
            default: 3GiB
```

Execution authoring patterns (choose one default execution declaration mode):

1. Single-context shorthand (lean repos with one execution shape):

```yaml
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: node:24-bookworm
```

2. Named contexts (repos with multiple explicit execution planes):

```yaml
execution:
  default_context: development
  contexts:
    development:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
    verify:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
```

3. Named contexts with `extends` (multi-context repos that want less repetition):

```yaml
execution:
  default_context: development
  contexts:
    node-base:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
    development:
      extends: node-base
      container:
        resources:
          memory:
            minimum: 2GiB
            default: 3GiB
    verify:
      extends: node-base
```

`extends` is optional. It reduces repetition for named contexts; it does not replace shorthand for simple repos.
Single-context shorthand and named contexts are separate declaration modes: once a contract declares `execution.default_context` or `execution.contexts`, it must stop using root shorthand (`execution.preferred` / `execution.lifecycle` / `execution.backends`) as overlapping default execution truth.

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
- `execution.backends.container.engines` can list supported OCI engine CLIs in preference order; when omitted, ota falls back to `docker`
- `execution.preferred: remote` requires `execution.backends.remote.provider`
- `execution.preferred: remote` requires `execution.backends.remote.target`
- remote target guidance by provider:
- `daytona`: `sandbox-dev`
- `ssh` / `tsh`: `user@host`
- `kubectl`: `pod/ota-dev`
- `execution.default_context` declares the context used when task-level `context` is not set
- `execution.contexts` defines backend and requirement surfaces per context
- `execution.contexts.<name>.extends` lets a named context inherit from one parent context to avoid repetition
- root shorthand (`execution.preferred` / `execution.lifecycle` / `execution.backends`) must not be combined with `execution.default_context` or `execution.contexts`; pick either shorthand-only or named-context mode
- each `execution.contexts.<name>` requires:
  - `backend` and matching backend settings (`container.image` + `lifecycle`, or `remote.provider` + `remote.target`)
  - optional `container.resources.memory.minimum` and `container.resources.memory.default` for container contexts
  - optional `requirements.<runtimes|tools>` to scope readiness checks to that context
  - optional `attachments.compose` to attach container workloads to compose project networks
  - optional `attachments.isolated_paths` to mount Ota-managed, engine-owned named volumes over workspace-relative dependency paths such as `node_modules`
- inheritance merge rules for `extends`:
  - scalar fields override within a backend family (`lifecycle`, image/target/provider)
  - maps merge recursively (`container.resources`, `requirements`, `attachments`)
  - lists replace (`container.engines`, `attachments.compose`, `attachments.isolated_paths`)
- backend-family switches across `extends` are rejected (for example inheriting from a `container` parent and setting child `backend: native`)
- `extends` is additive inheritance within one backend family, not a generic "inherit anything, then replace `backend` later" escape hatch
- invalid example:
  - parent `backend: native`, child `extends: parent`, child `backend: container`
  - ota rejects this because the parent and child do not share one execution shape

Current implementation:

- `ota run` resolves a task context from `tasks.<name>.context` and `execution.default_context`, then executes that context's backend
- runtime selection consumes resolved named contexts after `extends` merge, so `ota run`, `ota up`, `ota doctor`, and `ota execution plan` execute the merged concrete context shape instead of partial parent/child declarations
- `execution.contexts` are used for context-scoped requirement checks and receipts
- `tasks.<name>.context` lets a task declare a non-default execution context
- named contexts can now share a base execution shape through `extends`, while shorthand remains the lean authoring path for shorthand-only repos
- `ota run` now supports container execution when context or legacy config provides `execution.*.container.image`
- the container path uses the first available configured container engine, mounts the effective contract directory at `/workspace`, overlays any declared `attachments.isolated_paths` with Ota-managed named volumes, and runs task bodies with `sh -lc`
- container contexts can declare `container.resources.memory` so ota requests a deterministic container memory limit; `ota run --memory <size>` overrides one run while keeping task identity and internal listener bind ports unchanged
- `ota up` now runs the `setup` task in the task's resolved context backend
- `ota run` supports remote execution when the resolved context or legacy `execution.backends.remote` declares `provider` and `target`
- current shipped remote providers are `daytona`, `ssh`, `tsh`, and `kubectl`
- the current remote path shells out to the local provider CLI with optional `execution.backends.remote.cwd`
- `ota up` runs its `setup` task through the same remote context/backend path when remote execution is selected or explicitly overridden
- remote provisioning and remote workspace selection are still out of scope today

Current lifecycle meaning:

- `persistent`: when `execution.preferred: container` is configured, `ota run` and the `setup` task inside `ota up` reuse a persistent named container for the effective contract directory
- `ephemeral`: when `execution.preferred: container` is configured, `ota run` and the `setup` task inside `ota up` use a fresh `run --rm` container with the first available configured engine for each invocation
- outside backend-backed task execution, such as service commands, healthchecks, and diagnosis, lifecycle remains advisory today

Current command behavior:

- `ota doctor` warns when `ephemeral` is declared and surfaces container dependency isolation in execution summaries when contexts declare `attachments.isolated_paths`
- `ota run` prints a lifecycle note on stderr and can execute via the configured container backend
- `ota run` can also override execution mode and lifecycle for one invocation with `--mode`, `--lifecycle`, or the shorthand `--ephemeral`
- `ota up` can also override execution mode and lifecycle for the `setup` phase with `--mode`, `--lifecycle`, or the shorthand `--ephemeral`
- `ota up` prints the same lifecycle note on stderr when its `setup` phase uses backend-backed execution
- `ota clean` removes current contract-derived Ota-managed persistent containers and dependency-isolation volumes for container contexts
- `ota clean` also rediscovers drifted Ota-managed persistent containers and dependency-isolation volumes by ownership metadata (`dev.ota.managed`, cleanup-kind/lifecycle labels, and repo ownership token), even when the contract has drifted away from the original declaration
- persistent container reconciliation treats execution shape drift as recreate-worthy, including Compose attachment namespace changes from `execution.contexts.<name>.attachments.compose`
- repo cleanup identity is anchored by `.ota/state/ownership-id` and tracked repo-used engines in `.ota/state/managed-engines`, so drifted cleanup can stay scoped to the repo instead of matching by `project.name`
- `ota clean` currently has no remote cleanup action; remote-backed repos report `No cleanup needed` today
- `ota doctor` checks the required backend CLI for the selected execution context or preferred backend and reports unsupported shipped remote providers early
- `ota doctor` warns on suspicious remote target shape (`ssh`/`tsh` without `user@host`, `kubectl` not starting `pod/`)
- `ota doctor` evaluates context-specific requirements for declared contexts
- `ota up` still runs service start commands, service healthchecks, and diagnosis on the host unless the resolved execution path is containerized

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
  billing-db:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: billing-db
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
      app:
        address: billing-db
        port: 5432
    readiness:
      from: app
      run: pg_isready -h billing-db -p 5432
```

Fields:

- `required`: optional boolean
- `provider`: optional string
- `start`: optional string
- `stop`: optional string
- `healthcheck`: optional string
- `manager`: optional object with:
  - `kind: compose|host`
  - `name`: compose project name (`compose` required)
  - `service`: compose service name when `kind: compose`
  - `file`: optional compose file path when `kind: compose`
- `endpoints`: optional per-context projections of reachable service address/port
- `depends_on`: optional list of service names
- `timeout`: optional healthcheck timeout in milliseconds
- `readiness`: optional explicit readiness check that runs in a named execution context
- `readiness.from`: context name that owns the runtime for the check
- `readiness.run`: command to execute in that context

Current behavior:

- services are part of the accepted V1 contract surface
- service declarations may use legacy `provider/start/stop/healthcheck` fields or new context-aware `manager/endpoints/readiness` fields
- unknown `depends_on` references are invalid
- service dependency cycles are invalid
- `timeout` must be greater than zero when set
- `readiness_gate` is a later-spec draft field and is not accepted by the current shipped parser
- `ota doctor` runs declared service `healthcheck` commands
- for `provider: docker-compose`, `ota doctor` runs the healthcheck inside the service container via `docker compose exec -T <service> sh -lc <healthcheck>`
- for `manager.kind: compose`, `ota doctor` derives `start/stop/healthcheck` commands from compose metadata
- for `manager.kind: host`, `ota doctor` runs healthchecks in the resolved host command context
- `services.<name>.readiness.from` with a named endpoint projection validates readiness from that execution context
- `services.<name>.endpoints.<context>` projects a context-specific address/port pair for readiness reporting and topology checks
- failed required service healthchecks are blocking errors
- failed optional service healthchecks are warnings
- timed out required service healthchecks are blocking errors
- timed out optional service healthchecks are warnings
- required services without a `healthcheck` produce a warning because readiness cannot be verified yet
- `ota up` starts required services, and required-service dependencies, in declared dependency order before `setup`
- `ota up` treats each required service healthcheck as a readiness gate before moving on to dependents
- ota still does not provide deep service orchestration beyond explicit contract commands

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
  pwsh:
    version: "7.6.0"
    only_on:
      - windows
    platforms:
      windows:
        distribution: zulu
```

Rules:

- runtime names must not be empty
- versions must not be empty
- `required` defaults to `true` and controls whether missing or mismatched runtimes are blocking
- `only_on`, when set, scopes the runtime to `linux`, `macos`, or `windows`
- `provider`, when set, must not be empty
- `distribution`, when set, must not be empty
- `platforms` may override `version`, `provider`, and `distribution` per OS using
  `linux`, `macos`, or `windows`
- `platforms` entries must also appear in `only_on` when `only_on` is declared
- workspace overlays may specialize member runtime requirements, but the winning value must be explainable

Version syntax examples:

- `8` is an example of an exact required version
- `>=8` is an example of accepting any version at or above `8`
- `^8` is an example of a compatible version range, usually the same major line
- ota compares numeric version parts and accepts common prefixes such as `go1.24.2` or `v1.24.2`
- use `>=` when you want to accept newer versions explicitly
- use `^` when you want to express compatibility rather than a strict floor

Runtime detail fields:

- `required`: optional boolean; defaults to `true`
- `only_on`: optional OS inclusion list; if omitted, the runtime is required on all supported OSes
- `provider`: optional runtime manager or provisioning source hint such as `volta`
- `distribution`: optional runtime flavor where version alone is not sufficient, especially Java
  distributions such as `temurin`, `corretto`, `graalvm`, `oracle`, or `zulu`
- `platforms`: optional per-OS overrides keyed by `linux`, `macos`, or `windows`

Use `only_on` to scope where a runtime is required, and use `platforms` only when values change on a matching OS.
`required: false` keeps the runtime active but downgrades missing/version mismatch findings to warnings.
Root fields act as the default values, and the matching `platforms.<os>` entry overrides them for that OS.

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
  pwsh:
    version: "7.6.0"
    only_on:
      - windows
```

Rules:

- tool names must not be empty
- versions must not be empty
- `required` defaults to `true` and controls whether missing or mismatched tools are blocking
- `only_on`, when set, scopes the tool to `linux`, `macos`, or `windows`
- `platforms` may override `version` per OS using `linux`, `macos`, or `windows`
- `platforms` entries must also appear in `only_on` when `only_on` is declared
- some tool keys map to different executables; for example, `tools.maven` is checked via `mvn`
- workspace overlays may specialize member tool requirements, but provenance must remain visible in diagnosis output

Use `only_on` to scope where a tool is required, and use `platforms` only when values change on a matching OS.
`required: false` keeps the tool active but downgrades missing/version mismatch findings to warnings.
Root fields act as the default values, and the matching `platforms.<os>` entry overrides them for that OS.

## `env`

Optional.

```yaml
env:
  vars:
    OTA_ENV:
      required: true
      secret: false
      default: local
      allowed:
        - local
        - ci
    PATH:
      prepend:
        - ./node_modules/.bin
        - /opt/ota/bin
  sources:
    - kind: dotenv
      path: .env.local
    - kind: dotenv
      path: .env
      must_exist: true
```

Fields:

- `vars`: env-variable requirements keyed by env name
- `sources`: ordered declared env sources

This is not only a validation surface. Root `env` is the repo-wide execution contract ota uses to
resolve values before `ota run` and `ota up` start a process.

`env.vars.<NAME>` fields:

- `required`: optional boolean
- `secret`: optional boolean; secret values are redacted in execution receipts and are not
  passed through remote shell wrappers
- `default`: optional string
- `allowed`: optional list of allowed values
- `prepend`: optional list of path entries to add before the resolved value when the env key is
  `PATH`; these take priority
- `append`: optional list of path entries to add after the resolved value when the env key is
  `PATH`; these act as fallback locations

`env.sources[]` fields:

- `kind`: source type; today ota ships `dotenv`
- `path`: source path relative to the contract directory
- `must_exist`: optional boolean; when `true`, the source artifact itself is part of readiness

`PATH` is a standard search-path env var, so it is the one env key that supports structured
composition. Most env vars are simple single values instead.

Use `PATH` when the repo needs to control executable search order. Use ordinary env values like
`JAVA_HOME` when the repo needs one explicit location.

Examples:

```yaml
env:
  vars:
    PATH:
      prepend:
        - ./node_modules/.bin
        - /usr/local/cargo/bin
```

If the existing `PATH` is `/usr/local/bin:/usr/bin:/bin`, the final value becomes
`./node_modules/.bin:/usr/local/cargo/bin:/usr/local/bin:/usr/bin:/bin`.

```yaml
env:
  vars:
    JAVA_HOME:
      required: true
      default: /opt/jdk-22
```

This sets one explicit Java location. It does not merge with other values.

```yaml
env:
  vars:
    DISCORD_TOKEN:
      required: true
      secret: true
    SUPABASE_URL:
      required: true
  sources:
    - kind: dotenv
      path: .env.local
    - kind: dotenv
      path: .env
      must_exist: true
```

This makes dotenv loading explicit instead of magical:

- ota reads `.env.local`, then `.env`
- `.env.local` is optional
- `.env` must exist
- process env and `policies.env.values` still outrank both files

```yaml
policies:
  env:
    values:
      DATABASE_URL: postgres://policy.internal/app
      RELEASE_CHANNEL: stable
```

This is the org policy side of env resolution:

- the repo contract still declares the env names in `env.vars`
- policy can supply the winning value through `policies.env.values`
- policy does not invent new repo requirements on its own

```yaml
tasks:
  test:
    env:
      CI: "true"
      NODE_ENV: test
```

This sets ordinary task-scoped env values directly.

Example:

```yaml
env:
  vars:
    PATH:
      prepend:
        - ./node_modules/.bin
        - /opt/ota/bin
```

If the existing `PATH` is `/usr/local/bin:/usr/bin:/bin`, the final value becomes
`./node_modules/.bin:/opt/ota/bin:/usr/local/bin:/usr/bin:/bin`.

Policy-aware env selection and workspace inheritance are described in
[Environment variables](env-resolution-and-policy.md).

Current behavior:

- `run` prefers approved org-policy env values, then process environment, then declared env sources in
  order, then `default`
- declared env values are injected into backend execution after resolution, so the spawned task
  process sees the same chosen value across native, container, and remote backends
- `run` rejects disallowed values
- `doctor` reports missing required vars, invalid values, and missing or invalid declared env
  sources
- `secret: true` may not be combined with a default value
- secret env values are redacted in execution receipts
- remote task execution rejects secret env values instead of inlining them into remote shell
  command strings
- `PATH` can be composed from `prepend` entries, the resolved base value, and `append` entries
- ota does not permanently mutate the user's shell session; resolved env values apply to the
  process ota starts

Resolution and provenance:

- repo-declared requirements remain the canonical source of truth
- workspace overlays may add or specialize member values when explicitly configured
- policy-derived values should be reported distinctly from repo-declared values
- execution receipts should explain which layer supplied the value that won
- `doctor` and `detect` should expose provenance instead of flattening the result into a bare string

## `tasks`

Optional.

```yaml
tasks:
  setup:
    description: Install dependencies
    category: setup
    run: pnpm install
    safe_for_agent: true
  build:
    context: app
    requires_services:
      - postgres
    depends_on:
      - setup
    run: pnpm build
  dev:
    context: app
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
  package:
    depends_on:
      - build
    run: tar -czf dist/release.tar.gz dist/
  upload:
    depends_on:
      - package
    run: ./scripts/upload-artifact.sh dist/release.tar.gz
extensions:
  release-upload:
    kind: export_provider
    command: ota-ext-upload
    api_version: 1
    description: Upload the release bundle to the artifact endpoint
    config:
      endpoint: https://artifacts.example.com/upload
      artifact: dist/release.tar.gz
```

Fields:

- `description`: optional string
- `notes`: optional multiline guidance for humans and agents
- `category`: optional string
- `env`: optional map of fixed task-scoped environment overrides
- `inputs`: optional map of named task inputs
- `context`: optional execution context name
- `run`: optional string for a single shell-compatible command
- `script`: optional string for an inline multiline shell script
- `execution`: optional mode-aware execution branches for one task intent
- `runtime`: optional long-running workload shape for endpoint-bearing tasks
- `variants`: optional list of conditional task executions
- `requires_services`: optional list of service names that must be ready before the task body runs
- `depends_on`: optional list of task names
- `safe_for_agent`: optional boolean
- `internal`: optional boolean; marks orchestration plumbing tasks that stay in the graph but are hidden from default `ota tasks` discovery surfaces

`execution` fields:

- `default_mode`: optional `native`, `container`, or `remote`
- `modes`: optional backend map
- `modes.<mode>.context`: optional context override for that mode
- `modes.<mode>.lifecycle`: optional lifecycle override for that mode (container mode only)
- `modes.<mode>.env`: optional env map merged over task-level `env`
- `modes.<mode>.run`: optional single-line command override for that mode
- `modes.<mode>.script`: optional multiline script override for that mode
- `modes.<mode>.runtime`: optional runtime/listener override for that mode

`execution` mode rules:

- `--mode` changes execution plane, not task identity; one task name can carry multiple mode branches
- `default_mode` can stand alone when the task-level `run`/`script` already describes the default path
- when a branch is selected, branch values override task-level values for `context`, `lifecycle`, `env`, `run`/`script`, and `runtime`
- when a selected branch omits `run`/`script`, or when no branch exists for the selected mode, ota falls back to the task-level execution body (including OS variants)
- use `modes.<mode>` only for mode-specific overrides; you do not need an empty branch such as `modes.native: {}` just to pair with `default_mode: native`
- `modes.native.lifecycle` and `modes.remote.lifecycle` are invalid; lifecycle is only valid for container execution

`runtime` fields:

- `kind`: currently `service`
- `listeners`: named listener map
- `listeners.<name>.protocol`: `http`, `https`, or `tcp`
- `listeners.<name>.bind.address`: bind address inside the task execution context
- `listeners.<name>.bind.port.mode`: `fixed` or `discover`
- `listeners.<name>.bind.port.value`: required when `mode: fixed`
- `listeners.<name>.project.host.address`: host-visible address for the projected listener
- `listeners.<name>.project.host.port.mode`: `fixed` or `auto`
- `listeners.<name>.project.host.port.value`: required when host port `mode: fixed`
- `listeners.<name>.project.host.primary`: optional boolean; mark exactly one projected listener as primary when multiple listeners are projected
- `listeners.<name>.project.host.path`: optional URL path for `http` and `https`

`runtime` mode semantics:

- `bind.port.mode: fixed`: the task must listen on one explicit port inside its execution context
- `bind.port.mode: discover`: ota discovers the final listening port after the task starts; use this only for native tasks where the process may auto-bump to a free port
- `project.host.port.mode: fixed`: ota uses one explicit host port and the contract should treat that URL as stable
- `project.host.port.mode: auto`: ota injects runtime URL env values before command execution and reports the resolved URL in receipts and JSON output; ephemeral container runs pre-reserve a host port, while persistent container runs reconcile the named container and then resolve the current published host mapping
- `ota run <task> --host-port <port>` can override one run's published host/public port on the selected primary projected listener when that listener uses `project.host.port.mode: fixed`; the workload bind port stays unchanged
- `ota run <task> --memory <size>` can override one run's requested container memory for container execution while preserving contract/task intent
- with multiple projected listeners, mark one listener as `project.host.primary: true`; ota uses that listener for `OTA_PUBLIC_URL` and primary endpoint rendering

Current execution rules:

- native tasks may use `bind.port.mode: fixed` or `discover`
- native tasks with `bind.port.mode: discover` must not also declare `project.host.port.mode: fixed`
- container tasks with `project.host` must use `bind.port.mode: fixed`
- container tasks may use `project.host.port.mode: fixed` or `auto`
- remote execution contexts do not support `runtime.kind: service` host projection yet
- loopback-only container binds such as `127.0.0.1` or `localhost` must not be projected to `host`
- for container tasks with `project.host.port.mode: auto`, ota verifies resolved host publication; ephemeral runs retry bounded times on host-port conflict before failing, and persistent runs recreate mismatched containers when reconciliation cannot safely reuse the existing publication shape
- `--host-port` rejects invalid shapes before task spawn: non-container execution, listeners with `project.host.port.mode: auto`, no projected host listeners, or ambiguous multi-listener projection without one primary listener
- container memory precedence is: `--memory` override, then `execution.contexts.<name>.container.resources.memory.default`, then `execution.contexts.<name>.container.resources.memory.minimum`, then engine default
- when `execution.contexts.<name>.container.resources.memory.minimum` is declared, ota rejects `--memory` values below the minimum before task spawn

Use cases:

- use `run` for one command you would normally type in a shell
- use `script` when the task needs multiple lines, shell setup, or cleanup steps
- use `env` when a task needs fixed environment values that should override repo-level env for that task
- use `inputs` when a task needs named per-run values like `base_url`, `tenant`, or `mode`
- use `context` when one task should run in a different execution plane from the repo default
- use `execution.modes` when one task intent should run differently across modes (for example `start` container by default and `start --mode native` on host) without splitting into `start` and `start:host`
- use `runtime.kind: service` when a task is a long-running workload that should publish a deterministic host endpoint
- use `bind.port.mode: fixed` when the app should stay on one known internal port
- use `bind.port.mode: discover` when a native dev server may choose its final port at runtime
- use `project.host.port.mode: auto` when the host port should be conflict-free but does not need to stay fixed
- use `execution.contexts.<name>.container.resources.memory.minimum/default` when container workloads need explicit memory truth to stay reliable across environments
- use `description` for the short summary and `notes` for the task purpose plus extra guidance
- use `requires_services` when a task needs canonical services brought up through their manager before the task runs
- use `depends_on` to model a build/package/upload chain without hiding order in shell scripts

Task input semantics:

- `inputs` are declared in `tasks.<name>.inputs`
- each input name is lowercase snake_case in the contract and becomes `--kebab-case` on the CLI
- ota injects resolved values into the task process as `OTA_INPUT_<NAME>`
- `default` supplies a value when the caller omits the input
- `required: true` makes the input mandatory unless a default is present
- `allowed` limits the accepted values for that input
- task inputs override repo-level env only for the task they belong to
- task dependencies do not inherit the parent task’s declared inputs
- if every declared input has a default, the task can be run with no input flags
- task input names may overlap ota command flags such as `mode` or `jobs`; when they do, put ota command flags before the task and task inputs after the task
- `requires_services` resolves declared services before the task body and keeps lifecycle ownership with `services.<name>.manager`
- `runtime.listeners` keep workload ingress with the task instead of overloading `services`
- `ota run` records the resolved runtime endpoint in receipts and JSON output when ota can authoritatively resolve it
- `ota up` only reports workload endpoints for runtime-bearing tasks it actually executes during preparation today; it does not yet discover arbitrary app tasks like `dev`
- container runtime listeners export these env values before process start when host projection resolves:
- `OTA_PUBLIC_URL`
- `OTA_PUBLIC_HOST`
- `OTA_PUBLIC_PORT`
- `OTA_PUBLIC_URL_<LISTENER>`
- `OTA_PUBLIC_URL` is the primary projected listener URL; use `OTA_PUBLIC_URL_<LISTENER>` for secondary listeners

Example:

```yaml
tasks:
  api-automation-tests:
    description: Run API automation tests
    notes: |
      Use this to verify the API against a running local service.
      Prefer after `ota run setup` and before merging contract changes.
    inputs:
      base_url:
        description: API base URL for the live suite
        default: http://localhost:8080
      suite_mode:
        description: Run mode for the API suite
        default: standard
        allowed:
          - standard
          - contract-drift
      skip_api:
        description: Skip API execution and build reports only
        default: false
        allowed:
          - true
          - false
  version:bump:
    inputs:
      version:
        description: New release version for the Java SDK
        required: true
```

Run it as:

```bash
ota run api-automation-tests
ota run api-automation-tests --base-url http://localhost:8080 --suite-mode contract-drift
ota run version:bump --version minor
ota run version:bump --version 0.2.0
ota run version:bump --version major
```

Input fields:

- `description`: optional string
- `notes`: optional multiline string for purpose, when to use, and any operator guidance
- `required`: optional boolean
- `default`: optional string
- `allowed`: optional list of accepted string values

Input rules:

- input names must use lowercase snake_case
- input names must not collide with reserved `ota run` / `ota workspace run` flag names and aliases such as `backend`, `jobs`, `json`, `lifecycle`, `member`, `mode`, `receipt`, or `stream`
- if an existing contract already uses one of those names, rename it to a task-specific variant such as `suite_mode`, `output_json`, `target_member`, or `execution_backend` before upgrading
- `default` must be non-empty when present
- `allowed` values must be non-empty
- when `allowed` is declared, `default` must be one of the allowed values
- `required: true` cannot be satisfied by an empty value

Mode-aware task example:

```yaml
tasks:
  start:
    description: Start the app
    requires_services:
      - postgres
    execution:
      default_mode: container
      modes:
        native:
          context: host
          env:
            DB_URL: jdbc:postgresql://127.0.0.1:5432/app
          run: mvn spring-boot:run
        container:
          context: app
          lifecycle: persistent
          env:
            DB_URL: jdbc:postgresql://postgres:5432/app
          run: mvn spring-boot:run -Dspring-boot.run.arguments=--server.address=0.0.0.0,--server.port=8080
          runtime:
            kind: service
            listeners:
              http:
                protocol: http
                bind:
                  address: 0.0.0.0
                  port:
                    mode: fixed
                    value: 8080
                project:
                  host:
                    address: 127.0.0.1
                    port:
                      mode: auto
                    path: /
```

Run it as:

```bash
ota run start
ota run start --mode native
ota run start --mode container
```

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

Hook fields:

- `after_success`: optional ordered list of task names to run only after the task body exits successfully
- `after_failure`: optional ordered list of task names to run only after the task body exits with a failure
- `after_always`: optional ordered list of task names to run after either success or failure, but only when the task body was actually attempted

Example post-outcome hooks:

```yaml
tasks:
  build:
    run: pnpm build
    depends_on: [setup]
    after_success: [verify-dist]
    after_failure: [collect-build-diagnostics]
    after_always: [cleanup-temp]
```

Rules:

- task names must not be empty
- tasks must declare a default `run` or `script`, or at least one variant
- input names must use lowercase snake_case
- input defaults must not be empty
- input allowed values must not be empty
- `run` must be non-empty when present
- `script` must be non-empty when present
- variant entries must declare `when.os`
- variant entries must declare exactly one of `run` or `script`
- duplicate variants for the same `when.os` are rejected
- dependency references must resolve to known tasks
- hook references must resolve to known tasks
- task dependency cycles are rejected
- hook edges participate in the same task cycle detection as `depends_on`
- `depends_on` is the canonical way to reuse task steps instead of calling `ota run` from inside another task script
- `requires_services` references must resolve to known services
- each required service must declare an actionable manager or readiness surface so ota can enforce the requirement

Current execution model:

- `run` and `script` are shell-compatible execution forms
- task `env` values are applied when ota runs the task and override repo-level env with the same name
- when variants are declared, ota resolves the best matching `when.os` entry first and falls back to the default execution
- `depends_on` runs before the task body
- `after_success` runs only when the task body exits `0`
- `after_failure` runs only when the task body exits non-zero
- `after_always` runs after either branch, but only when the task body actually ran
- hook tasks run in declared order
- tasks marked `internal: true` (commonly `setup`) remain normal graph nodes for `depends_on` and hooks, still run when referenced directly, and are hidden from default `ota tasks` output unless `--all` is requested
- hook failures affect the final task result for the parent task
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
  protected_paths:
    - Cargo.lock
    - LICENSE
  bootstrap:
    ota:
      note: Only install ota if it is missing and installation is approved.
      sh: curl -fsSL https://dist.ota.run/install.sh | sh
      powershell: irm https://dist.ota.run/install.ps1 | iex
  notes: Keep agent edits narrow and add regressions for behavioral changes.
```

Current validation rules:

- `entrypoint` must reference a known task when set
- `default_task` must reference a known task when set
- `safe_tasks` entries must reference known tasks
- `verify_after_changes` entries must reference known tasks
- `writable_paths` entries must not be empty
- `protected_paths` entries must not be empty
- `bootstrap.ota` must include at least one install command when present

Current implementation treats this as contract surface and validation input. It is not yet a full agent runtime layer.

Starter contracts commonly use a minimal default AI-agent block when the detector has enough
confidence to write one and can infer safe writable paths. The block is stored under `agent`, and
it gives an AI agent the safe paths and tasks it should use first. That default usually includes
`setup` as the entrypoint when present, `test` as the verification task when present, `test` in
`verify_after_changes` when present, `ota.yaml` in `protected_paths`, and a short note pointing at the matching
`ota run <task>` command. When `ota` itself should be installable by an agent, the starter block
can also include an approved `bootstrap.ota` entry with the shell and PowerShell install
commands.

Agent semantics:

- `entrypoint` is the first task an AI agent should use to get oriented in the repo
- `default_task` is the normal verification task to run when no more specific task is needed
- `safe_tasks` are the tasks an AI agent can run without broad risk
- `verify_after_changes` are the tasks an AI agent should rerun after modifying files
- `writable_paths` are the paths an AI agent may edit
- `protected_paths` are the paths an AI agent should avoid editing casually
- `bootstrap.ota` provides an approved `ota` install path for agents when the binary is missing
- `bootstrap.ota.note` should explain when that install path may be used
- `bootstrap.ota.sh` and `bootstrap.ota.powershell` should give the approved shell and PowerShell install commands
- `notes` is free-form repo guidance for humans and AI agents
- `ota detect --merge` and `ota detect --rewrite` refuse to write protected paths declared by the existing contract

## `metadata`

Optional.

```yaml
metadata:
  team: platform
  owner: ota
  author: ota Maintainers
  created_at: 2026-03-23
  ota:
    detect:
      field_ownership:
        project.name: merged
        tools.pnpm: merged
        tools.curl: manual
```

This is an open map for extra repo-specific values.

`ota detect --write`, `ota detect --merge`, and `ota detect --rewrite` record ota-managed detect
fields under `metadata.ota.detect.field_ownership` using `merged`.

The `metadata.ota.detect` subtree is ota-reserved and must remain mapping-shaped. If `metadata.ota`
or `metadata.ota.detect` is repurposed as a scalar or list, detect merge cannot persist ownership
metadata and will fail until that path is repaired.

You can also pin curated fields explicitly with `manual` there when detector silence should not be
treated as contract drift. When a field has no detect ownership entry, ota treats the existing
contract value as `manual` by default.

## Full example

See:

- [../../examples/full-contract/ota.yaml](../../examples/full-contract/ota.yaml)
