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

Use this page as the canonical field and validation reference for the shipped contract surface.
When you need operator guidance for targets, shared backends, activation, and backend fulfillment,
follow it with [local-service-topology.md](local-service-topology.md).

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
- `readiness`: reusable named readiness probes for workflow and check reuse.
- `workflows`: canonical operational paths built from setup/run tasks, required services, and readiness gates.
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
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  setup:
    run: pnpm install
workflows:
  default: app
  app:
    intent: local_development
    setup:
      task: setup
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
`description`, `activation`, and `config`.
Supported kinds today are `check_provider`, `export_provider`, and `backend_provider`.
`check_provider` is runnable with `ota extensions --run <name>` when `api_version: 1` is declared.
`export_provider` is runnable with `ota extensions --publish <name>` when `api_version: 1` is declared.
`backend_provider` is reserved for task execution backends, is discoverable in the contract, and
can be named by `execution.backends.remote.provider` when the repo wants a custom execution
backend. Runtime backend providers receive a structured JSON request on stdin and via
`OTA_BACKEND_PROVIDER_REQUEST_JSON`, then return a structured JSON response on stdout. The
request includes the extension id, kind, api version, command context, repo context path, working
directory, task name, task command, execution mode, target, cwd, and resolved environment values.
When a backend provider should participate in non-manual target activation, declare
`activation.provider_managed_cleanup: true`; that tells ota the provider can also handle the
follow-up `activation_probe` and `activation_cleanup` command contexts for activation-started
producer services.
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
      # Optional for provider: ssh only. When omitted, ota uses normal OpenSSH
      # behavior (`~/.ssh/config`, agent/default identity selection, and host aliases).
      ssh:
        config_file: ~/.ssh/work.conf
        identity_file: ~/.ssh/work_rsa
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
      fulfillment: run
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

1. Named contexts (repos with multiple explicit execution planes):

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

1. Named contexts with `extends` (multi-context repos that want less repetition):

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
  - optional `env` for context-wide environment defaults that apply before task-level and mode-level env overrides
  - optional `requirements.<runtimes|tools>` to scope readiness checks to that context
  - optional `attachments.compose` to attach container workloads to compose project networks
  - optional `attachments.isolated_paths` to mount Ota-managed, engine-owned named volumes over workspace-relative dependency paths such as `node_modules`
- inheritance merge rules for `extends`:
  - scalar fields override within a backend family (`lifecycle`, image/target/provider)
  - maps merge recursively (`container.resources`, `env`, `requirements`, `attachments`)
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
- ota injects `OTA_WORKSPACE` into task execution so backend-aware workspace-relative paths stay explicit without hardcoding `/workspace`
- task env precedence is: resolved context env, then `tasks.<name>.env`, then selected `tasks.<name>.execution.modes.<mode>.env`
- ota-derived cache env is fallback-only and currently covers `MAVEN_OPTS` for isolated `.m2`, `NPM_CONFIG_CACHE` for isolated `.npm`, `PNPM_STORE_DIR` for isolated `.pnpm-store`, `GRADLE_USER_HOME` for isolated `.gradle`, `PIP_CACHE_DIR` for isolated `.pip-cache`, and `POETRY_CACHE_DIR` for isolated `.pypoetry-cache`; explicit task or context env still wins
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
- `ota validate` and `ota doctor` also warn when `depends_on` crosses execution boundaries in a way that drops in-place prep value, and when a declared isolated cache path is likely unused by the tool configuration
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
      kind: tcp
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 10s
```

Fields:

- `required`: optional boolean
- `producer`: optional object declaring that this service is owned by another workspace repo task instead of local service-manager truth
- `producer.repo`: required workspace repo name declared under `ota.workspace.yaml`
- `producer.task`: required producing task name in that repo's `ota.yaml`
- `producer.listener`: optional named runtime listener on that producer task; omit it only when the producer exposes exactly one declared listener
- `producer.address_view`: optional reachable address shape; the current shipped cross-repo service slice supports `host` only and defaults to `host`
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
- `readiness.from`: context name that owns the runtime for the check and matches one declared endpoint projection
- legacy `readiness.run`: command to execute in that context
- structured `readiness.kind`: `tcp` or `http`
- structured `readiness.method`: optional HTTP method, default `GET`
- structured `readiness.path`: required for structured HTTP readiness and must start with `/`
- structured `readiness.headers`: optional HTTP request headers
- structured `readiness.success.status`: optional exact accepted HTTP status-code set
- structured `readiness.body.contains`: optional exact required response substring; it must not be combined with `method: HEAD`
- structured `readiness.interval`: optional wait between probe attempts
- structured `readiness.timeout`: optional per-attempt probe timeout
- structured `readiness.retries`: optional failed probe budget before the service readiness gate fails; when omitted, ota keeps waiting until readiness passes, the run is interrupted, or a higher-level command boundary ends the wait
- structured `readiness.start_period`: optional delay before the first structured readiness probe

Current behavior:

- services are part of the accepted V1 contract surface
- `services.<name>.producer` is the canonical cross-repo service-ownership surface when a required service is produced by another repo in the same `ota.workspace.yaml`
- producer-owned services stay intentionally explicit today:
  - only `producer.address_view: host` is supported
  - the producer listener must declare one fixed `project.host` endpoint
  - `ota doctor`, `ota up`, and `ota run` may reuse or start that producer through the owning repo contract before the consumer proceeds
- producer-owned services must not also declare local manager truth such as `manager`, `provider`, `start`, `stop`, `healthcheck`, `endpoints`, `readiness`, or `timeout`
- `tasks.<name>.requires_services` remains the consumer-side dependency truth; producer ownership lives on `services.<name>`, not inside each consumer task
- service declarations may use legacy `provider/start/stop/healthcheck` fields or new context-aware `manager/endpoints/readiness` fields
- `services.<name>.readiness` now supports three valid forms:
  - legacy command form: `from` + `run`
  - reusable probe form: `from` + `probe` (+ optional polling controls such as `interval`, `retries`, and `start_period`)
  - structured probe form: `from` + `kind` (+ `path` for HTTP, with optional request/response/timing controls)
- unknown `depends_on` references are invalid
- service dependency cycles are invalid
- `timeout` must be greater than zero when set
- `readiness_gate` is a later-spec draft field and is not accepted by the current shipped parser
- `ota doctor` runs declared service `healthcheck` commands
- for `provider: docker-compose`, `ota doctor` runs the healthcheck inside the service container via `docker compose exec -T <service> sh -lc <healthcheck>`
- for `manager.kind: compose`, `ota doctor` derives `start/stop/healthcheck` commands from compose metadata
- for `manager.kind: host`, `ota doctor` runs healthchecks in the resolved host command context
- `services.<name>.readiness.from` with a named endpoint projection validates readiness from that execution context
- `services.<name>.readiness.probe` can reference one top-level `readiness.probes.<name>` declaration so service-manager readiness reuses the same transport and timeout truth as checks and workflows while `from` still selects the service endpoint projection
- structured `services.<name>.readiness.kind: http` probes the declared endpoint with the same request/response model shipped for task runtime readiness
- structured `services.<name>.readiness.kind: tcp` probes the declared endpoint for listener reachability from the declared context
- legacy `services.<name>.readiness.run` remains supported for repo-specific command probes that do not fit the structured HTTP/TCP model yet
- reusable and structured top-level service readiness use the same default wait model as task runtime readiness: when `retries` is omitted, ota keeps waiting until readiness passes or the surrounding run is interrupted; declaring `retries` makes the failure budget explicit and bounded
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
    - kind: properties
      path: config/runtime.properties
    - kind: json
      path: config/runtime.json
    - kind: yaml
      path: config/runtime.yaml
    - kind: toml
      path: config/runtime.toml
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

- `kind`: source type; ota ships curated `dotenv`, `properties`, `json`, `yaml`, and `toml`
- `path`: source path relative to the contract directory
- `must_exist`: optional boolean; when `true`, the source artifact itself is part of readiness

Declared source rules:

- source files are loaded only when explicitly declared in `env.sources`
- precedence is unchanged: policy values, then process env, then declared sources in order, then
  `default`
- `properties` is a flat key-value source
- `json` must have an object root
- `yaml` must have an object root
- `toml` must have a table root
- nested `json`, `yaml`, and `toml` objects flatten with `.` before env-key normalization
- only scalar leaf values are allowed in structured sources: string, number, bool
- `null`, arrays, object leaf values, and unsupported scalar classes such as TOML datetimes are rejected
- for `properties`, `json`, `yaml`, and `toml`, ota normalizes keys by trimming, replacing `.`, `-`, whitespace,
  `/`, and `:` with `_`, collapsing repeated separators, and uppercasing the final env key
- if two keys in the same declared source normalize to the same env key, ota fails that source
  load explicitly

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

This makes declared source loading explicit instead of magical:

- ota reads `.env.local`, then `.env`
- `.env.local` is optional
- `.env` must exist
- process env and `policies.env.values` still outrank both files

```yaml
env:
  vars:
    APP_PORT:
      required: true
    FEATURE_FLAGS_BETA_ENABLED:
      required: true
  sources:
    - kind: properties
      path: config/runtime.properties
    - kind: json
      path: config/runtime.json
```

With:

- `config/runtime.properties`: `app.port=8080`
- `config/runtime.json`: `{"feature flags":{"beta-enabled":true}}`

ota resolves:

- `APP_PORT=8080`
- `FEATURE_FLAGS_BETA_ENABLED=true`

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

## `surfaces`

Optional.

For the operator guide to what surfaces are, when to add them, and how they relate to listener
shorthand and full listeners, see [surfaces.md](surfaces.md).

```yaml
surfaces:
  backend:
    kind: http
    label: Backend API
    purpose: Primary application API for local development
    visibility: internal
    port: 5678
    path: /
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 10000
  frontend:
    kind: http
    label: Editor UI
    purpose: Browser-facing editor surface
    visibility: public
    port: 8080
    path: /
    readiness:
      kind: http
      path: /
      timeout: 10000
```

Fields:

- `<name>.kind`: required `http`, `https`, or `tcp`
- `<name>.port`: required fixed port number
- `<name>.label`: optional short operator-facing label for command and topology rendering
- `<name>.purpose`: optional short purpose string for operators and docs
- `<name>.visibility`: optional `public` or `internal` metadata for output and UX grouping
- `<name>.path`: optional HTTP/HTTPS path; defaults to `/` for HTTP/HTTPS surfaces
- `<name>.readiness`: optional reusable readiness contract for that surface
- `<name>.readiness.kind`: required when readiness is declared; `http` or `tcp`
- `<name>.readiness.path`: optional for HTTP readiness when the surface path is already sufficient;
  otherwise required
- `<name>.readiness.method`: optional HTTP method; defaults to `GET`
- `<name>.readiness.headers`: optional HTTP request headers
- `<name>.readiness.success.status`: optional accepted HTTP status list
- `<name>.readiness.body.contains`: optional required response substring
- `<name>.readiness.interval`: optional polling interval
- `<name>.readiness.timeout`: optional per-attempt timeout
- `<name>.readiness.retries`: optional consecutive failure budget
- `<name>.readiness.start_period`: optional delay before the first probe

Current behavior:

- surfaces are reusable endpoint truth, not standalone operational URLs
- a surface becomes operational only when a service task runtime attaches it through
  `tasks.<name>.runtime.surfaces`
- attached surfaces normalize into the existing runtime listener model with conservative loopback
  defaults
- `kind: https` reuses the existing HTTPS listener protocol and HTTP readiness semantics without
  inventing separate certificate or trust-management contract fields
- workflows may reference attached surfaces for readiness and exposes without repeating host URLs
- `ota execution topology` reports both top-level declared surfaces and the normalized listener
  shape on attached runtimes
- `ota execution topology` also reports additive `surface_attachments` on task runtimes so machine
  consumers can see whether one attached surface used defaults or explicit bind/project overrides

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
      readiness:
        kind: http
        listener: http
        method: GET
        path: /health
        headers:
          Accept: application/json
        success:
          status: [200]
        body:
          contains: '"status":"UP"'
        interval: 5s
        timeout: 3s
        retries: 5
        start_period: 10s
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

# Common local listener shorthand:
#
# listeners:
#   http:
#     http: 3000
#
# This is authoring sugar only. Ota normalizes it to the full listener form with:
# - bind address `127.0.0.1`
# - fixed bind port `3000`
# - projected host `127.0.0.1:3000`
# - projected host path `/` for HTTP
#
# Use the full `protocol` / `bind` / `project` form whenever bind address, host address,
# host-port mode, primary projection, or path needs to be customized.
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
- `launch`: optional structured launch source for inspectable command or packaged container starts
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
- `modes.<mode>.launch`: optional structured launch override for that mode
- `modes.<mode>.runtime`: optional runtime/listener override for that mode

`execution` mode rules:

- `--mode` changes execution plane, not task identity; one task name can carry multiple mode branches
- `default_mode` can stand alone when the task-level `run`/`script` already describes the default path
- when a branch is selected, branch values override task-level values for `context`, `lifecycle`, `env`, `run`/`script`/`launch`, and `runtime`
- when a selected branch omits `run`/`script`/`launch`, or when no branch exists for the selected mode, ota falls back to the task-level execution body (including OS variants)
- use `modes.<mode>` only for mode-specific overrides; you do not need an empty branch such as `modes.native: {}` just to pair with `default_mode: native`
- `modes.native.lifecycle` and `modes.remote.lifecycle` are invalid; lifecycle is only valid for container execution

`launch` fields:

- `launch.kind`: required `command` or `container`
- `launch.kind: command`
  - `launch.exe`: required executable name or path
  - `launch.args`: optional argument list
- `launch.kind: container`
  - `launch.image`: required packaged runtime image
  - `launch.engine`: optional engine override; defaults to `docker`
  - `launch.args`: optional container command arguments
  - `launch.name`: optional stable container name
  - `launch.remove`: reserved for future non-service container launches; omit it for service
    runtimes in this slice
  - `launch.volumes`: optional named-volume mounts
  - `launch.volumes[].name` or `launch.volumes[].source`: required source identifier
  - `launch.volumes[].target`: required container path

`launch` rules:

- each task must declare exactly one executable source:
  - `run`
  - `script`
  - `launch`
- `run` stays the simple shell shorthand
- `script` stays the multiline shell escape hatch
- `launch` is for structured, inspectable starts that Ota should render and reason about without
  hiding everything inside one shell string
- `launch.kind: command` reuses existing task env, input, receipt, dependency, and agent-safety
  behavior
- `launch.kind: container` is a task launch source, not an execution context
- service tasks that use `launch.kind: container` still treat `runtime.surfaces` as the canonical
  public endpoint truth; launch must not create a competing published-port contract
- container launch service tasks are persistent Ota-managed services in this slice; Ota may replace
  an existing named launch container only when its ownership labels prove it belongs to the same
  repo and task family
- packaged containers must attach surfaces with container-safe publication overrides when the
  default loopback bind is not valid, for example `bind.address: 0.0.0.0` plus a loopback host
  projection

Examples:

```yaml
tasks:
  quickstart:
    launch:
      kind: command
      exe: npx
      args: [n8n]
    runtime:
      kind: service
      surfaces:
        - backend

  packaged:
    launch:
      kind: container
      image: docker.n8n.io/n8nio/n8n
      volumes:
        - name: n8n_data
          target: /home/node/.n8n
    runtime:
      kind: service
      surfaces:
        backend:
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 5678
              path: /
              primary: true
```

`runtime` fields:

- `kind`: currently `service`
- `backend_binding`: optional shared backend binding name declared under `execution.shared_backends`
- `surfaces`: optional list of reusable top-level runtime surfaces declared under `surfaces`
- `listeners`: named listener map
- `listeners.<name>.http: <port>`: shorthand for the common local HTTP listener shape
- `listeners.<name>.tcp: <port>`: shorthand for the common local TCP listener shape
- `listeners.<name>.protocol`: `http`, `https`, or `tcp`
- `listeners.<name>.bind.address`: bind address inside the task execution context
- `listeners.<name>.bind.port.mode`: `fixed` or `discover`
- `listeners.<name>.bind.port.value`: required when `mode: fixed`
- `listeners.<name>.project.host.address`: host-visible address for the projected listener
- `listeners.<name>.project.host.port.mode`: `fixed` or `auto`
- `listeners.<name>.project.host.port.value`: required when host port `mode: fixed`
- `listeners.<name>.project.host.primary`: optional boolean; mark exactly one projected listener as primary when multiple listeners are projected
- `listeners.<name>.project.host.path`: optional URL path for `http` and `https`

Listener shorthand rules:

- shorthand is authoring sugar only; ota normalizes it into the full listener model internally
- `http: <port>` expands to:
  - `protocol: http`
  - `bind.address: 127.0.0.1`
  - fixed bind port `<port>`
  - fixed host projection `127.0.0.1:<port>`
  - host projection path `/`
- `tcp: <port>` expands to:
  - `protocol: tcp`
  - `bind.address: 127.0.0.1`
  - fixed bind port `<port>`
  - fixed host projection `127.0.0.1:<port>`
- shorthand cannot be mixed with `protocol`, `bind`, or `project`
- shorthand supports exactly one of `http` or `tcp`
- use the verbose form when bind address, host address, host-port mode, primary projection, or path must be customized

Surface attachment rules:

- use top-level `surfaces` when one endpoint meaning should stay shared across tasks and workflows;
  see [surfaces.md](surfaces.md)
- `runtime.surfaces` supports two attachment forms:
  - list form like `runtime.surfaces: [backend]` for default publication
  - object form like `runtime.surfaces.backend` for attachment overrides
- `runtime.surfaces.<name>` attachment overrides are publication-only:
  - `bind`
  - `project`
  - `project.host.primary`
- each attached surface normalizes into the same runtime listener model used by explicit
  `runtime.listeners`
- topology JSON now also exposes additive `runtime.surface_attachments.<name>` intent alongside the
  normalized listener truth
- attached surface names become normalized listener names
- a runtime must not attach an unknown surface
- a runtime must not declare `runtime.listeners.<name>` and also attach `runtime.surfaces.<name>`
  for the same name
- `runtime.surfaces.<name>.bind.port` must preserve the declared top-level surface port with
  `mode: fixed`
- if a runtime attaches exactly one surface, has no inline `runtime.readiness`, and that surface
  declares readiness, ota derives the equivalent runtime readiness automatically
- if a runtime attaches multiple surfaces, has no inline `runtime.readiness`, and exactly one
  attached surface is marked `project.host.primary: true`, ota derives runtime readiness from that
  primary surface

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
- the selected workflow setup task's `requires_services` entries become the pre-setup service phase for `ota up`
- `setup.requires_services` remains the compatibility fallback when the default workflow setup task is `setup`
- `workflows.<default>.services.required` defines the canonical post-setup service plane for `ota up`; repo-level `services.<name>.required` remains the fallback when no workflow services are declared
- `runtime.listeners` keep workload ingress with the task instead of overloading `services`
- `ota run` records the resolved runtime endpoint in receipts and JSON output when ota can authoritatively resolve it
- `ota up` reports workload endpoints for runtime-bearing tasks it actually executes while bringing the selected workflow to readiness
- container runtime listeners export these env values before process start when host projection resolves:
- `OTA_PUBLIC_URL`
- `OTA_PUBLIC_HOST`
- `OTA_PUBLIC_PORT`
- `OTA_PUBLIC_URL_<LISTENER>`
- `OTA_PUBLIC_URL` is the primary projected listener URL; use `OTA_PUBLIC_URL_<LISTENER>` for secondary listeners

Task target binding semantics:

- `tasks.<name>.targets.<target>` declares first-class local topology target identity
- `<target>` is the consumer-local target name
  - use a short stable name such as `api`, `admin`, or `billing`
  - ota uses that name for evidence and, when `override_input` is omitted, for runtime export as `OTA_TARGET_<TARGET>`
- each target binding declares exactly one identity shape:
  - `service`
  - `url`
- `url` is a fixed declared URL target
  - use it when the target is explicit and should not resolve through repo-managed service topology
  - value must be non-empty
- `service.member`, `service.repo`, `service.task`, `service.listener`, and optional `service.address_view` (`topology`, `host`, `internal`) declare a repo-managed service target
- `service.member` is an optional monorepo member selector
  - use it when the producer lives in another member declared under `workspace.members`
  - value must name an existing declared monorepo member
  - current shipped cross-member slice is intentionally narrow:
    - `address_view: host` always works when the producer declares a fixed `project.host` endpoint and remains `activation.mode: manual` only
    - `address_view: topology` / `address_view: internal` work only when consumer and producer share one declared backend binding on the active plane
    - non-manual activation (`ensure_started`, `restart_ready`, `ensure_running`, `ensure_ready`) is shipped only for those shared-backend `topology` / `internal` member targets
- `service.repo` is an optional workspace repo selector
  - use it when the producer lives in another repo declared under `ota.workspace.yaml`
  - value must name an existing declared workspace repo
  - current shipped workspace cross-repo slice is intentionally explicit:
    - only `address_view: host` is supported
    - the producer listener must declare one fixed `project.host` endpoint
    - `activation.mode: ensure_started`, `restart_ready`, `ensure_running`, and `ensure_ready` may reuse or start that producer through the owning repo contract before the consumer runs
- `service.task` is the producing task name
  - use it when the target should follow a repo-managed service task instead of a guessed literal URL
  - value must name an existing service task in the same contract, or in `service.member` / `service.repo` when one selector is present
- `service.listener` is the named listener exposed by the producing task runtime
  - use it when the producer exposes more than one endpoint and the consumer must target one specific listener
  - value must name an existing listener under `tasks.<producer>.runtime.listeners`
  - it may be omitted only when the producing service task exposes exactly one declared listener name across its service runtime shapes
- `service.address_view` tells ota which reachable address shape to resolve for the consumer
  - `host`: use the producer's published host URL
  - `topology`: use the current local topology address when ota can resolve it truthfully
  - `internal`: use the producer's in-backend address when ota can resolve that internal plane truthfully
  - omit it only when the default resolution rules for that binding are already sufficient and explicit in the surrounding contract
- optional `override_input` points at a declared task input used as an explicit operator override channel
  - use it when an operator may intentionally point the consumer at another target such as staging, preview, or a separately started local app
  - value must name an input declared on the same consuming task
- optional `activation.mode` controls whether ota should auto-start and observe the local producer service before the consumer task runs
  - `manual` = resolve target only; never auto-start the producer
  - `ensure_started` = when ota resolves a local target binding and no explicit override input wins, reuse the producer if it already appears reachable or start it without waiting for listener reachability or deeper readiness
  - `restart_ready` = when ota resolves a local target binding and no explicit override input wins, restart a currently reachable producer and wait for readiness before continuing
  - `ensure_running` = when ota resolves a local target binding and no explicit override input wins, reuse the producer if the declared target listener is already reachable or start it and wait until that listener becomes reachable
  - `ensure_ready` = when ota resolves a local target binding and no explicit override input wins, reuse the producer if already reachable or start it and wait until ready
- `url` targets support `activation.mode: manual` only
- service runtimes may declare `runtime.readiness` when “ready” must mean more than “the socket is accepting connections”
- resolution precedence is:
  - explicit `override_input` value supplied by the operator
  - resolved target binding URL
  - compatibility literal input default (when declared and binding resolution is unavailable)
- when `override_input` is omitted, resolved target bindings are exported to task execution as
  `OTA_TARGET_<TARGET>` (for example target `api` -> `OTA_TARGET_API`)
- `ota run` records target-resolution evidence in run receipts JSON under `receipt.steps[*].target_resolutions`
- target-activation evidence is recorded alongside target resolution under `receipt.steps[*].target_resolutions[*].activation`
  - human meaning:
    - `started_started` = ota started the producer without waiting for listener reachability or deeper readiness
    - `reused_started` = ota reused a producer that already appeared started enough for the target edge
    - `started_running` = ota started the producer and waited for the declared listener to become reachable
    - `reused_running` = ota found the declared listener already reachable and reused the producer
    - `restarted_ready` = ota found the producer already reachable, restarted it deliberately, and waited for readiness again
    - `started_ready` = ota started the producer and waited for readiness
    - `reused_ready` = ota found the producer already ready and reused it
- topology resolution rules are:
  - native caller: resolves from declared fixed `project.host` endpoint unless caller and producer share one declared native `runtime.backend_binding`, in which case ota resolves to the producer fixed bind endpoint inside that shared boundary
  - container caller: resolves only when caller and producer share one declared container `runtime.backend_binding`; ota resolves to the producer fixed bind endpoint inside that shared boundary
  - remote caller: resolves only when caller and producer share one declared remote `runtime.backend_binding`; ota resolves to the producer fixed bind endpoint inside that shared boundary
- internal resolution rules are:
  - container caller: resolves only when caller and producer share one declared container `runtime.backend_binding`; ota resolves to the producer fixed bind endpoint inside that shared boundary
  - native caller: resolves only when caller and producer share one declared native `runtime.backend_binding`; ota resolves to the producer fixed bind endpoint inside that shared boundary
  - remote caller: resolves only when caller and producer share one declared remote `runtime.backend_binding`; ota resolves to the producer fixed bind endpoint inside that shared boundary
  - unresolved topology and unresolved `internal` views fail clearly at run time without host/bridge guessing
- current non-manual activation constraints:
  - only `service` targets participate in activation; `url` targets are always manual
  - explicit operator override inputs skip producer auto-start and preserve the override value
  - compatibility literal default fallbacks do not auto-start and fail clearly if `ensure_started`, `restart_ready`, `ensure_running`, or `ensure_ready` was requested
  - `ensure_started` launches the producer and returns immediately after startup is handed off; it does not wait for listener reachability or deeper readiness
  - `restart_ready` stops a currently reachable producer through ota's owned cleanup path, then starts it again and waits for readiness
  - `ensure_running` waits only for the declared target listener plane, even when the producer also declares a deeper `runtime.readiness` contract
  - when the producer service task declares `runtime.readiness`, ota waits for that readiness contract instead of treating an open listener socket as sufficient
  - the current shipped slice supports actual producer auto-start only when ota can own the producer honestly:
    - persistent container producer services
    - unix native producer services started through the activation-owned native path
    - built-in remote producer services (`ssh`, `tsh`, `kubectl`, `daytona`) only when the caller and producer share one declared remote backend binding:
      - `address_view: host` requires a fixed `project.host` endpoint
      - `address_view: topology` and `address_view: internal` may probe the fixed remote-plane bind endpoint
      - readiness may be `tcp` or `http`
    - backend-provider remote producer services only when the caller and producer share one declared remote backend binding and the matching `backend_provider` extension declares `activation.provider_managed_cleanup: true`:
      - `address_view: host` uses the listener fixed `project.host` endpoint
      - shared-remote `address_view: topology` / `address_view: internal` use the listener fixed `bind.port.value` on the remote plane through provider-owned `activation_probe`
      - `ensure_started` hands startup off immediately
      - `restart_ready` cleans up the currently reachable provider-owned producer, then restarts it and waits for readiness again
      - `ensure_running` waits for the declared reachable endpoint on the selected plane
      - `ensure_ready` may wait for deeper declared `runtime.readiness`
  - unsupported producer backend shapes fail clearly instead of guessing orchestration
  - stream-mode runs show an explicit activation wait phase while ota is starting or waiting on the producer readiness contract
  - on interrupt, ota cleans up producer services that this consumer run activation-started; reused producers are left running intentionally

Current `runtime.readiness` support for service tasks:

- `probe: <name>`
  - references one top-level `readiness.probes.<name>` declaration
  - reuses that probe's transport and timeout contract while the selected listener still determines the runtime endpoint
  - may optionally declare `listener` when the readiness target should bind to one non-default runtime listener explicitly
  - may still declare `interval`, `retries`, and `start_period` to control polling semantics for this runtime
  - must not also declare inline `kind`, `method`, `path`, `headers`, `success`, `body`, or `timeout`
- `kind: http`
  - requires `listener`
  - requires `path`
  - optional `method` supports `GET` and `HEAD`; default is `GET`
  - optional `headers` adds request headers to the readiness probe
  - optional `success.status` overrides the accepted HTTP status codes; default is any `2xx` or `3xx`
  - optional `body.contains` requires the response body to contain one exact substring after the status code matches, and it must not be used with `method: HEAD`
  - optional `interval` sets the wait between probe attempts; when omitted, ota uses the current small internal poll cadence
  - optional `timeout` sets the per-attempt probe timeout
  - optional `retries` sets the consecutive failed probe budget before activation fails
  - optional `start_period` delays the first readiness probe after activation starts
  - requires the referenced listener to declare `project.host`, except for shared-remote
    `ensure_ready` on built-in remote providers where ota may instead probe the declared
    remote-plane listener address and fixed `bind.port.value`
  - ota waits for the declared response contract from the selected probe endpoint
- `kind: tcp`
  - requires `listener`
  - must not declare `method`, `headers`, `success`, or `body`
  - may still declare `interval`, `timeout`, `retries`, and `start_period` to control readiness wait behavior
  - for local host-projected readiness, requires the referenced listener to declare `project.host`
  - for shared-remote `ensure_ready`, built-in remote providers may instead use the listener `bind.port.value` on the remote plane
  - ota waits until the selected probe endpoint accepts TCP connections or is listening on the shared remote plane

Shared local backend semantics:

- `execution.shared_backends.<name>` declares an explicit ota-owned shared backend boundary for co-located long-running tasks
- required fields:
  - `scope`
  - `backend`
  - `lifecycle`
- optional fields:
  - `context` to pin the shared backend to one named execution context
  - `fulfillment` (`none` or `run`) to control whether ota may prepare missing backend requirements on the actual `ota run` path
  - `environment` to declare backend environment intent for policy-governed image/profile resolution:
    - `profile` (policy-backed fulfillment profile name)
    - `image_alias` (policy-backed image alias name)
    - `image` (literal image intent for compatibility)
    - `source` (optional source class, valid only with literal `image`)
    - an empty `environment: {}` is allowed when the repo wants policy `default_profile` resolution to choose the effective backend image
    - if no policy `default_profile` resolves, ota falls back to the task/container image and shared-backend shape validation follows that same fallback instead of assuming one synthetic shared image
- tasks opt in through `tasks.<name>.runtime.backend_binding: <name>`
- `execution.shared_backends.<name>.backend` currently supports:
  - `container`
  - `native`
  - `remote`
- current constraints by backend family:
  - `container` may use `environment`, shared publications, and container-shape reconciliation
  - `native` is currently `scope: local` + `lifecycle: persistent` only, and does not support `environment`
  - `remote` is currently `scope: remote` + `lifecycle: persistent` only, and does not support `environment`
  - remote service listeners are currently contract-driven fixed endpoints: declare `bind.port.mode: fixed`, `bind.port.value`, and if `project.host` is used declare `project.host.port.mode: fixed`
- contract meaning:
  - `requirements` still declare what the backend or context needs
  - `fulfillment` declares whether ota may try to make that true on the `ota run` path
  - org policy still decides which provisioning sources and versions are approved
- shared backend identity is deterministic and drives:
  - persistent container family/shape reconciliation for create/reuse/recreate
  - topology/internal addressability for shared `container`, `native`, and `remote` target bindings when ota can prove that shared boundary
  - backend-scoped run-path fulfillment when the group declares `fulfillment: run`
  - receipt evidence in `receipt.steps[*].shared_local_backend` (`name`, `backend`, `lifecycle`, declared environment intent, effective profile/image/source/registry, effective identity, and reuse state when known)
  - receipt evidence in `receipt.steps[*].backend_fulfillment` when ota probes or prepares the backend
    - human meaning:
      - `requirements_satisfied` = the backend already had what the contract required
      - `fulfilled` = ota had to provision something and finished successfully
      - `missing_requirements` = requirements were missing and the contract/policy did not allow run-path fulfillment
      - `failed` = ota attempted fulfillment or setup, but it did not complete successfully
  - `ota execution plan`, `ota run`, and run receipts all resolve the same effective backend image for explicit-context and inferred-context shared backend groups
- validation rules include:
  - binding must reference declared `execution.shared_backends.<name>`
  - binding backend family must match task runtime backend
  - when a local backend omits `context`, bound tasks must not span multiple resolved contexts
- current slice constraints:
  - shared backend groups must resolve one deterministic backend shape within their shipped backend family (same effective image, dependency-isolation shape, and memory shape where those dimensions apply)
  - bound workloads may differ in commands, listeners, readiness, and publications
  - ota rejects real workload-local conflicts inside that shared boundary, including conflicting in-backend bind endpoints and conflicting fixed host publications
  - persistent shared backend reconciliation uses the shared union of declared workload publications, while per-task runtime evidence and listener resolution remain task-scoped
  - non-manual target activation currently auto-starts:
    - persistent container producer services
    - unix native producer services
    - built-in remote producer services for shared-remote `address_view: host` / `address_view: topology` / `address_view: internal`
      - `address_view: host` uses the listener fixed `project.host` endpoint
      - shared-remote `address_view: topology` / `address_view: internal` use the listener fixed `bind.port.value` on the remote plane
      - `ensure_started` hands startup off immediately; `restart_ready` bounces a reachable producer and waits for readiness; `ensure_running` observes listener reachability; `ensure_ready` may observe `tcp` or `http` runtime readiness
      - backend-provider remote activation now covers shared-remote `address_view: host` / `address_view: topology` / `address_view: internal` when `activation.provider_managed_cleanup: true`
      - built-in remote providers:
        - `ssh`: `user@host`
        - `tsh`: `user@host`
        - `kubectl`: `pod/ota-dev`
        - `daytona`: `sandbox-dev`
      - for `provider: ssh`, omit `remote.ssh` unless the repo must force a non-default SSH config
        or identity file; when omitted, ota delegates host alias and identity selection to normal
        OpenSSH behavior
  - shared backends currently ship as:
    - local `container`
    - local `native`
    - remote `remote`
  - fulfillment currently acts on the effective shared backend requirement union for container, native, and remote shared backend families
  - profile/alias environment intent requires an active org policy pack under `.ota/org-policy.yaml` (`policies.backend_environment`)
  - policy may govern allowed/denied `source` classes and registries for the effective backend image
  - `fulfillment: none` fails clearly when required runtimes or tools are missing, while `fulfillment: run` attempts approved provisioning before any bound task body or dependency task uses that backend
- later slices are expected to relax some of that strictness by extending the same model, not by replacing it with guessed addressing or implicit backend-sharing behavior

Direct container-context fulfillment semantics:

- `execution.contexts.<name>.fulfillment` is valid only for `backend: container`
- valid values are:
  - `none`
  - `run`
- `execution.contexts.<name>.requirements` still declares what that execution plane needs; `fulfillment` does not replace or infer the requirements surface
- `fulfillment: run` tells ota to satisfy declared context requirements on the actual `ota run` path when policy-approved provisioning is available
- `fulfillment: none` keeps the same requirements truth but fails clearly instead of mutating the execution environment
- org policy still governs whether ota may provision and which sources/versions are approved; `fulfillment: run` is runtime intent, not a policy bypass
- when the active org policy enables `strict_versions`, ota also treats already-installed repo-satisfying versions as missing if they are not policy-compliant, and `fulfillment: run` may repair them with an approved exact version
- current slice behavior:
  - persistent container contexts are fulfilled immediately against the resolved persistent execution container
  - ephemeral container contexts are fulfilled inside the same named ephemeral execution container before the task body runs
  - this first direct-ephemeral slice currently supports non-service task execution only; ota does not yet claim service-task run-path fulfillment for direct ephemeral container contexts
- validation rules:
  - `backend: native` contexts must not declare `fulfillment`
  - `backend: remote` contexts must not declare `fulfillment`

Example:

```yaml
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 8080
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
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: topology
        override_input: base_url
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
- reusable probes are now shipped through top-level `readiness.probes`, `checks[].probe`, `workflows.<name>.readiness.probes`, and runtime/service readiness `probe` references
- use task names to describe intent: `setup`, `dev`, `dev_clean`, `test`, `lint`

## `readiness`

Optional.

Use this section when one readiness target should be declared once and reused across workflow
readiness and explicit named checks.

```yaml
readiness:
  probes:
    backend-ready:
      kind: http
      target:
        kind: task
        name: backend
        listener: backend
        address_view: host
      method: GET
      path: /healthz/readiness
      headers:
        x-ota-probe: workflow
      success:
        status: [200]
      timeout: 10000
```

Fields:

- `probes.<name>.kind`: `http` or `tcp`
- `probes.<name>.url`: optional absolute `http://` URL for literal URL probes
- `probes.<name>.target`: optional topology-derived target
  - `target.kind`: `task` or `service`
  - `target.name`: required task or service name
  - `target.listener`: required for task targets
  - `target.address_view`: optional for task targets; defaults to `host`
  - `target.observer`: optional for task targets
    - `observer.kind`: `command_host` (default) or `task`
    - `observer.task`: required when `observer.kind: task`
  - `target.endpoint`: optional for service targets; required when the service declares more than
    one endpoint
  - `target.observer` is not valid for service targets
- `probes.<name>.method`: optional HTTP method (`GET` by default, `HEAD` supported)
- `probes.<name>.path`: required for target-based `kind: http` probes
- `probes.<name>.headers`: optional HTTP headers for `kind: http`
- `probes.<name>.success.status`: optional accepted HTTP status list for `kind: http`
- `probes.<name>.body.contains`: optional HTTP body substring match for `kind: http`
- `probes.<name>.expect_status`: optional shorthand for one accepted HTTP status when
  `success.status` is omitted
- `probes.<name>.timeout`: required integer timeout in milliseconds

Current behavior:

- top-level probes are canonical reusable readiness definitions
- literal `url` probes stay first-class for external or intentionally non-topological endpoints
- target-based probes can resolve from declared task listeners or service endpoints instead of
  copying host/port values into one URL string
- `checks[].probe` can reference a named probe instead of repeating a shell command
- `workflows.<name>.readiness.probes` can reference probes directly when the workflow should be
  ready as soon as those probes pass
- `tasks.<name>.runtime.readiness.probe` and `services.<name>.readiness.probe` still reuse the
  named probe transport and timeout contract while keeping their own runtime/service endpoint
  selection semantics
- `kind: http` supports literal `url` probes and topology-derived `target` probes
- `kind: tcp` currently supports topology-derived `target` probes
- reusable `kind: http` probes now use the same request-shaping surface Ota already ships for
  runtime and service readiness: `method`, `headers`, `success.status`, and `body.contains`
- for plain `200`, authors may omit both `expect_status` and `success.status`
- both `expect_status` and `success.status` are fully supported for non-default success rules:
  - use `expect_status` when one shorthand status is clearer
  - use `success.status` when you want multiple accepted statuses
- task-target probes without `target.observer` still resolve from ota's invoking command plane, so
  `target.address_view: host` remains the correct default when one published host endpoint is the
  truth you want to reuse directly
- task-target probes may now declare `target.observer.kind: task` plus `target.observer.task` when
  `topology`, `internal`, or one caller-relative `host` view should be resolved exactly as that
  observer task sees it from its effective backend plane
- observer-backed task probes reuse the same target-binding semantics ota already ships for task
  targets instead of inventing a probe-only topology model
- unsupported schemes such as `https://` are rejected during validation instead of silently
  downgraded
- probe execution is direct inside ota; it does not depend on `curl`, `node`, or other repo-local
  tools

## `workflows`

Optional.

For the operator guide to what workflows are, when to add them, and how they relate to tasks,
surfaces, and agent hints, see [workflows.md](workflows.md).

```yaml
readiness:
  probes:
    app-ready:
      kind: http
      url: http://127.0.0.1:5678/healthz/readiness
      success:
        status: [200]
      timeout: 10000

workflows:
  default: app
  app:
    intent: local_development
    description: Canonical local app workflow
    setup:
      task: setup
    run:
      task: dev
    services:
      required:
        - postgres
    readiness:
      probes:
        - app-ready
      surfaces:
        - backend
    exposes:
      - surface: backend
      - http://127.0.0.1:5678
```

Fields:

- `default`: required when `workflows` is declared; names the canonical repo workflow
- `<name>.intent`: optional workflow classification such as `local_development`
- `<name>.description`: optional operator-facing summary
- `<name>.setup.task`: optional task ota should treat as the preparation phase for that workflow
- `<name>.run.task`: optional task ota should treat as the primary runnable surface for that workflow
- `<name>.services.required`: optional services that belong to that workflow
- `<name>.readiness.checks`: optional readiness checks that belong to that workflow
- `<name>.readiness.probes`: optional reusable readiness probes that belong to that workflow
- `<name>.readiness.surfaces`: optional attached runtime surfaces that belong to that workflow's selected run task
- `<name>.exposes`: optional human-readable endpoints or URLs the workflow is expected to surface
  - literal string form keeps a fixed URL
  - object form `{ surface: <name> }` resolves through the selected workflow run task

Current behavior:

- workflows do not replace `tasks`, `services`, or `checks`; they compose those primitives into one canonical operational path
- `doctor` diagnoses the default workflow by default when it declares workflow readiness probes,
  workflow readiness checks, or workflow services
- `check` follows the same selected workflow readiness boundary when a workflow declares explicit
  readiness probes or checks, and otherwise falls back to the repo-wide `checks` surface
- `doctor` and `check` may also validate `workflows.<name>.readiness.surfaces` through the selected
  workflow run task without hardcoding host URLs into the workflow
- workflow `exposes` may point at attached surfaces instead of repeating host URLs that the
  contract already owns under `surfaces`
- use `checks[].probe` when a named check should reuse a named readiness probe outside that
  workflow-scoped path or when the repo does not declare workflows
- `ota up` now targets the default workflow instead of assuming repo-wide `setup` semantics
- if `workflows.<default>.setup.task` is declared, `ota up` uses that task as the setup phase
- if `workflows.<default>.run.task` is declared and the task has a service runtime, `ota up` activates that task as part of readiness
- `tasks.setup` remains the compatibility fallback when no workflow setup task is declared
- `agent.default_task` and `agent.entrypoint` remain agent-facing hints, but the default workflow is now the canonical repo operational path

## `checks`

Optional.

```yaml
checks:
  - name: node-installed
    kind: precondition
    severity: error
    run: node --version
    timeout: 10
  - name: backend-ready
    kind: health
    severity: error
    probe: backend-ready
```

Fields:

- `name`: required, non-empty string
- `kind`: `precondition` or `health`
- `severity`: `error`, `warn`, or `info`
- `run`: optional shell command when the check is command-backed
- `probe`: optional probe reference when the check is probe-backed
- `timeout`: optional integer in milliseconds

Current behavior:

- `up` uses preconditions before setup
- `doctor` runs configured checks and reports findings by severity
- checks must declare exactly one of `run` or `probe`
- `checks[].probe` must reference a named `readiness.probes.<name>` declaration
- probe-backed checks use the check timeout when one is declared, otherwise they inherit the probe
  timeout
- when `timeout` is set, `doctor` fails the check if it does not finish within the configured millisecond budget
- human output identifies probe-backed failures as probes instead of pretending a shell command was
  run

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
  inferred_boundary:
    reviewed: false
    provenance:
      writable_paths:
        - detect:semantic_root_inference
        - detect:stack_source_scan
      protected_paths:
        - detect:contract_file_default
        - detect:detected_control_files
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
- `inferred_boundary.provenance.writable_paths` entries must not be empty when present
- `inferred_boundary.provenance.protected_paths` entries must not be empty when present
- `inferred_boundary` must include at least one provenance entry when present
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
- `inferred_boundary.reviewed: false` means ota inferred the current agent boundary but the repo author has not confirmed it yet
- `inferred_boundary.provenance` explains which starter or detector heuristics produced the current writable and protected boundary
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
