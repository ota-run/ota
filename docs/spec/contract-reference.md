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

This document describes the canonical `ota.yaml` authoring surface accepted by the shipped parser and validator.

Use this page as the canonical field and validation reference for the shipped contract surface.
For machine-readable contract publication, use
[`json-schemas/contract.json`](json-schemas/contract.json) locally or
`https://dist.ota.run/spec/json-schemas/latest/contract.json` in CI, editors, and other tooling.
For machine-readable docs ownership publication, use
[`published-docs/canonical-docs.json`](published-docs/canonical-docs.json) locally or
`https://dist.ota.run/spec/published-docs/latest/canonical-docs.json` when a downstream surface
needs the canonical upstream source boundary for this page.
When you need operator guidance for targets, shared backends, activation, and backend fulfillment,
follow it with [local-service-topology.md](local-service-topology.md).
Backward-compatibility parsing may continue to accept older shapes, but new and updated contracts
should normalize to the canonical surfaces documented here.

## Minimal contract

```yaml
version: 1
project:
  name: my-repo
```

In practice, most useful contracts also define tasks, runtimes, toolchains, or checks.

## Primary sections at a glance

- `version` (required): schema version for the contract itself. Today this is `1`.
- `project` (required): stable repo identity and high-level classification.
- `toolchains`: managed ecosystem capabilities such as Rust, Node, Java, or Python.
- `orchestrators`: repo-level task and environment mediation such as `mise`.
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
toolchains:
  rust:
    version: "1.94.0"
    components:
      - rustfmt
    fulfillment:
      mode: run
  node:
    version: "22"
    package_managers:
      pnpm: "10.0.0"
    fulfillment:
      source: mise
      mode: run
orchestrators:
  mise:
    kind: mise
    required: true
    config_files:
      - mise.toml
    activation:
      trust: true
    prepare:
      install: true
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
  description: Execution governance for humans and AI agents
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

## `artifacts`

Optional.

Use `artifacts` when a task generates a named repo-local artifact that later tasks consume. This
keeps code-generation ownership separate from broad `effects.writes` bookkeeping.

```yaml
artifacts:
  typescript-sdk:
    kind: generated_source
    producer: sdk:generate
    paths:
      - sdk/typescript/src/api/client.gen.ts
    inputs:
      - core/schema
```

- `kind: generated_source`: a normal generated artifact. Consumers explicitly depend on its
  producer task before Ota checks that its declared outputs exist. It may additionally declare
  `replay` when its existing generated lineage also needs an explicit regeneration and promotion
  authority chain.
- `kind: replay_baseline`: a baseline-first artifact. It must declare `replay`; use it when no
  other generated-artifact lineage applies.
- Any artifact with `replay` additionally declares:

  ```yaml
  replay:
    authority_manifest: replay/recorded-baseline.ota.json
    consumption: read_only
  ```

  For a `generated_source` artifact, producer safety remains the ordinary task-governance decision.
  A task that requires the artifact and lists its producer in top-level `depends_on` remains an
  ordinary lineage consumer; it runs the current producer and does not consume promoted replay
  authority. A task that requires that artifact without the unconditional producer dependency is a
  replay consumer. A dedicated `replay_baseline` artifact's producer must not be agent-safe, and
  every consumer of that dedicated artifact uses replay authority. `ota baseline record` is the separate explicit
  recording command; it is never inferred from an agent-safe task. The artifact keeps its declared `kind`, so replay authority
  does not erase generated-source lineage. Run
  `ota baseline record --artifact <name>` to issue a receipt-bound recorded attestation, then
  explicitly select that exact attestation with `ota baseline promote --artifact <name> --attestation <path>`.
  Ota rejects a selected replay workflow that includes the producer, and it rejects replay consumer
  closures that declare writes overlapping the baseline outputs, so replay cannot regenerate or
  mutate its own authority through a normal verification path.
  The committed authority manifest, not local `.ota` history and not a hand-authored digest, is
  the portable replay authority. Its `trust_root: scm_review` relies on the repository delivery
  path to review that committed selection; Ota does not verify reviewer or signer provenance.
  Consumers verify its complete output identities before execution;
  `consumption: read_only` is strict and requires an enforceable runner-owned ephemeral-container
  boundary for the entire selected closure. Ota snapshots the promoted outputs outside the writable
  workspace mount and mounts those snapshots read-only at their declared paths. `verify_unchanged`
  is not a weaker read-only mount: it detects a changed output after execution.
  Dependency and post-hook steps are part of the selected strict closure and receive the same
  runner-owned read-only mounts. Command-capable typed preparation remains typed in the contract
  and is projected through that boundary; a step that cannot execute or be projected there is
  refused before it runs.
  `consumption: verify_unchanged` is the non-strict fallback for a backend such as native execution:
  Ota verifies the selected authority before launch, re-captures the complete output manifest after
  the consumer ends, and fails with `replay_artifact_mutation_detected` if the baseline changed.
  It reports a detected write and never claims that it refused one.
  Replay-baseline symlinks must resolve within the declared artifact output boundary; Ota rejects
  escaping targets instead of mounting a link that can resolve into the mutable worktree.
- `producer`: required task name that materializes the artifact
- `paths`: required non-empty repo-relative output paths owned by the artifact
- `inputs`: optional repo-relative source paths that define the declared derivation boundary
- ordinary generated-artifact consumers declare `requires_artifacts: [typescript-sdk]` and directly
  depend on the producer task; replay consumers declare `requires_artifacts` without that producer
  dependency, because Ota resolves their promoted authority before the consumer body starts
- presence is not freshness: Ota does not infer currentness from timestamps or Git state

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
Named-context execution is the canonical selector whenever `execution.default_context` /
`execution.contexts` are present. In that mode:

- `execution.preferred` is not allowed
- `execution.lifecycle` and `execution.backends` may still be used as root defaults that named
  contexts inherit from when context-local values are omitted

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
- `execution.backends.container.platform` may pin the Linux OCI target as `linux/arch` or
  `linux/arch/variant` (for example `linux/amd64`). Ota applies the pin to every execution-backend
  container it creates, and changing it invalidates persistent-container reuse. The current
  container backend does not claim Windows-container execution. The `oci_local` enforcing target
  requires this field instead of inferring the target from the runner host.
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
- each `execution.contexts.<name>` requires:
  - `backend` and matching backend settings; contexts may declare those settings inline or inherit
    them from root defaults (`execution.lifecycle` / `execution.backends`)
  - optional `only_on` to scope the context to supported host OS values (`linux`, `macos`, `windows`)
  - optional `only_arch` to scope the context to supported host architectures (for example `x64`, `arm64`)
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
- when a selected context declares `only_on` and/or `only_arch`, `ota doctor`, `ota up`, and task execution fail early and explicitly on unsupported host OSes or architectures instead of falling through to later command noise
- `execution.contexts` are used for context-scoped requirement checks and receipts
- `tasks.<name>.context` lets a task declare a non-default execution context
- named contexts can now share a base execution shape through `extends`, while shorthand remains the lean authoring path for shorthand-only repos
- `ota run` now supports container execution when context or legacy config provides `execution.*.container.image`
- the container path uses the first available configured container engine, mounts the effective contract directory at `/workspace`, overlays any declared `attachments.isolated_paths` with Ota-managed named volumes, and runs task bodies with `sh -lc`
- ota injects `OTA_WORKSPACE` into task execution so backend-aware workspace-relative paths stay explicit without hardcoding `/workspace`
- ota also injects `OTA_HOST_WORKSPACE` into task execution so host-launched tasks can still refer to the real repo path even when the selected backend or helper workflow path also publishes `OTA_WORKSPACE`
- ota also injects `OTA_HOST_HOME` into task execution so selected workflow instances can derive stable host-owned clone or cache roots without shell `echo $HOME` / `%USERPROFILE%` glue
- ota injects `OTA_HOST_UID` and `OTA_HOST_GID` on Unix-like hosts so host-launched service and compose paths can pass the real host user/group ids into deterministic env interpolation without shell `id -u` / `id -g` glue; on non-Unix hosts those values are absent unless the contract resolves them some other way
- task env precedence is: resolved context env, then `tasks.<name>.env`, then selected `tasks.<name>.execution.modes.<mode>.env`
- ota-derived cache env is fallback-only and currently covers `MAVEN_OPTS` for isolated `.m2`, `NUGET_PACKAGES` for isolated `.nuget/packages`, `NPM_CONFIG_CACHE` for isolated `.npm`, `PNPM_STORE_DIR` for isolated `.pnpm-store`, `GRADLE_USER_HOME` for isolated `.gradle`, `PIP_CACHE_DIR` for isolated `.pip-cache`, and `POETRY_CACHE_DIR` for isolated `.pypoetry-cache`; explicit task or context env still wins
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
- outside backend-backed task execution, such as service startup, service readiness, and diagnosis, lifecycle remains advisory today
- explicit `ota run` / task-backed `ota up` lifecycle overrides are never advisory: a selected
  native or remote task path without a managed shared backend refuses `--ephemeral` or
  `--persistent` before execution starts

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
- when preconditions already contain blocking errors, `ota doctor` stops before later service/check readiness probing so blocked setups stay bounded and the primary blocker remains obvious
- `ota up` still runs service startup, service readiness, and diagnosis on the host unless the resolved execution path is containerized

## `services`

Optional.

```yaml
services:
  api:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: api
    endpoints:
      host:
        address: 127.0.0.1
        port: 3000
    readiness:
      from: host
      kind: http
      path: /health
      success:
        status: [200]
    depends_on:
      - postgres
  postgres:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
      app:
        address: postgres
        port: 5432
    readiness:
      from: host
      kind: tcp
      interval: 5s
      timeout: 3s
      retries: 5
      start_period: 10s
  redis:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: redis
    readiness:
      kind: compose_health
  postgres-host:
    manager:
      kind: host
      start:
        exe: brew
        args:
          - services
          - start
          - postgresql@17
      stop:
        exe: brew
        args:
          - services
          - stop
          - postgresql@17
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
    readiness:
      from: host
      kind: tcp
```

Fields:

- `required`: optional boolean
- `producer`: optional object declaring that this service is owned by another workspace repo task instead of local service-manager truth
- `producer.repo`: required workspace repo name declared under `ota.workspace.yaml`
- `producer.task`: required producing task name in that repo's `ota.yaml`
- `producer.listener`: optional named runtime listener on that producer task; omit it only when the producer exposes exactly one declared listener
- `producer.address_view`: optional reachable address shape; the current shipped cross-repo service slice supports `host` only and defaults to `host`
- `manager`: optional object with:
  - `kind: compose|host`
  - `engine`: optional compose CLI engine for `kind: compose`; `docker` by default, `podman` also supported
  - `name`: compose project name (`compose` required)
  - `service`: compose service name when `kind: compose`
  - `file`: optional compose file path when `kind: compose`
  - `env_file`: optional Compose interpolation file when `kind: compose`
  - `profiles`: optional Compose profile list when `kind: compose`
  - `start`: optional structured host start command when `kind: host`
  - `stop`: optional structured host stop command when `kind: host`
- `endpoints`: optional named projections of reachable service address/port
- `depends_on`: optional list of service names
- `readiness`: optional explicit readiness check that runs in a named execution context
- `readiness.from`: context name that owns the runtime for the check
- `readiness.endpoint`: optional named endpoint projection to use when one context has multiple service endpoints
- structured `readiness.kind`: `tcp`, `http`, or `compose_health`
- structured `readiness.method`: optional HTTP method, default `GET`
- structured `readiness.path`: required for structured HTTP readiness and must start with `/`
- structured `readiness.headers`: optional HTTP request headers
- structured `readiness.success.status`: optional exact accepted HTTP status-code set
- structured `readiness.body.contains`: optional exact required response substring; it must not be combined with `method: HEAD`
- structured `readiness.interval`: optional wait between probe attempts
- structured `readiness.timeout`: optional per-attempt probe timeout
- structured `readiness.retries`: optional failed probe budget before the service readiness gate fails; when omitted, Ota uses a bounded default budget (120 probe attempts) and reports failure when that budget is exhausted
- structured `readiness.start_period`: optional delay before the first structured readiness probe

Current behavior:

- services are part of the accepted V1 contract surface
- `services.<name>.producer` is the canonical cross-repo service-ownership surface when a required service is produced by another repo in the same `ota.workspace.yaml`
- producer-owned services stay intentionally explicit today:
  - only `producer.address_view: host` is supported
  - the producer listener must declare one fixed `project.host` endpoint
  - `ota doctor`, `ota up`, and `ota run` may reuse or start that producer through the owning repo contract before the consumer proceeds
- producer-owned services must not also declare local manager truth such as `manager`, `endpoints`, or `readiness`
- `tasks.<name>.requires_services` remains the consumer-side dependency truth; producer ownership lives on `services.<name>`, not inside each consumer task
- service declarations should model local service ownership with `manager`, reachability with `endpoints`, and readiness with `readiness`
- `services.<name>.readiness` has three canonical forms:
  - reusable probe form: `from` + `probe` (+ optional polling controls such as `interval`, `retries`, and `start_period`)
  - structured probe form: `from` + `kind` (+ `path` for HTTP, with optional request/response/timing controls)
  - structured compose-health form: `kind: compose_health` for compose-managed container health state without endpoint/host-port probing
- unknown `depends_on` references are invalid
- service dependency cycles are invalid
- `readiness_gate` is a later-spec draft field and is not accepted by the current shipped parser
- `ota doctor` evaluates manager-owned service readiness through the declared control plane and endpoint topology
- for `manager.kind: compose`, `ota doctor` derives compose lifecycle commands from manager metadata
- for `manager.kind: host`, canonical lifecycle ownership lives on `manager.start` / `manager.stop`; legacy top-level `start` / `stop` still parse for compatibility, but new authoring should keep host service lifecycle under the manager block
- for `manager.kind: host` with `manager.host.kind: systemd`, ota derives lifecycle from the declared unit instead of requiring shell `systemctl` glue
- `services.<name>.lifecycle.teardown_assertion: manager_inactive` declares that a future
  lifecycle-proof lane may require the manager's positive inactive-state observation after
  transaction-owned teardown; it is valid only with a typed manager, never an inverse readiness
  probe. Systemd observation also requires the declared unit to resolve as loaded; an unknown unit
  is not treated as inactive
- `services.<name>.lifecycle.teardown_assertion: boundary_terminated` is a narrower lifecycle
  proof capability for structured host-manager commands. It is admitted only when the selected
  lifecycle workflow resolves to an Ota-owned ephemeral container session; Ota runs the declared
  `manager.start` and `manager.stop` commands inside that session and must confirm removal of the
  exact session before publishing the terminal observation. It does not claim the host manager is
  inactive or that application output was proved
- for `manager.kind: host`, `ota doctor` runs readiness checks in the resolved host command context
- legacy `services.<name>.readiness.run` still parses for compatibility, but new authoring should keep service readiness on structured `readiness.kind` or reusable `readiness.probe`
- `services.<name>.readiness.from` selects the execution context for service readiness
- `services.<name>.readiness.endpoint` selects one named endpoint projection when `from` alone is ambiguous
- `services.<name>.readiness.probe` can reference one top-level `readiness.probes.<name>` declaration so service-manager readiness reuses the same transport and timeout truth as checks and workflows while `from` / `endpoint` still select the service endpoint projection
- structured `services.<name>.readiness.kind: http` probes the declared endpoint with the same request/response model shipped for task runtime readiness
- structured `services.<name>.readiness.kind: tcp` probes the declared endpoint for listener reachability from the declared context
- structured `services.<name>.readiness.kind: compose_health` reads the compose-managed container health status directly (`healthy`) and does not require `readiness.from` or `services.<name>.endpoints`
- structured `services.<name>.readiness.kind: systemd_active` reads systemd unit state directly (`systemctl is-active --quiet`) and does not require `readiness.from` or `services.<name>.endpoints`
- `kind: compose_health` requires `services.<name>.manager.kind: compose` and must not declare endpoint-probe fields such as `from`, `endpoint`, `method`, `path`, `headers`, `success`, `body`, or `timeout`
- `kind: systemd_active` requires `services.<name>.manager.kind: host` together with `services.<name>.manager.host.kind: systemd` and must not declare endpoint-probe fields such as `from`, `endpoint`, `method`, `path`, `headers`, `success`, `body`, or `timeout`
- reusable and structured top-level service readiness use the same default wait model as task runtime readiness: when `retries` is omitted, Ota uses the default bounded budget and reports failure after the limit is reached; declaring `retries` makes that budget explicit and tuned for the service
- `services.<name>.endpoints.<name>` declares one named endpoint projection:
  - `context`: optional execution context for that projection; when omitted, the endpoint name is also the context name for backward compatibility
  - `address`: required reachable address for that context projection
  - `port`: required reachable port for that context projection
- failed required service readiness checks are blocking errors
- failed optional service readiness checks are warnings
- timed out required service readiness checks are blocking errors
- timed out optional service readiness checks are warnings
- required services without declared readiness produce a warning because readiness cannot be verified yet
- `ota up` starts required services, and required-service dependencies, in declared dependency order before `setup`
- `ota up` treats each required service readiness check as a readiness gate before moving on to dependents
- ota still does not provide deep service orchestration beyond explicit contract commands

## `toolchains`

Optional.

Use `toolchains` when ota must understand more than "does this executable exist?" This is the
managed ecosystem layer. It owns capability truth for language environments such as Rust, Node,
Java, Python, Go, Ruby, and .NET, without forcing that truth into ad hoc shell setup.

Current shipped scope is intentionally narrow:

- top-level `toolchains`
- top-level `orchestrators`
- execution-context-scoped `execution.contexts.<name>.requirements.toolchains`
- task-scoped `requirements.toolchains`
- canonical shipped fulfillment paths for Rust, Node, Java, Python, Go, Ruby, and .NET toolchains
- explicit non-canonical run-path fulfillment when `fulfillment.source` points at another shipped
  source such as `mise`
- task execution mediation through `tasks.<name>.execution.orchestrator`
- duplicate ownership is invalid when the same prerequisite is declared under both `toolchains`
  and `runtimes` or `tools`

Example:

```yaml
toolchains:
  rust:
    version: "1.94.0"
    profile: minimal
    components:
      - rustfmt
    targets:
      - x86_64-unknown-linux-musl
    fulfillment:
      mode: run
  node:
    version: "24.15.0"
    package_managers:
      pnpm: "10.33.4"
    fulfillment:
      source: mise
      mode: run

orchestrators:
  mise:
    kind: mise
    required: true
    config_files:
      - mise.toml
    activation:
      trust: true
    prepare:
      install: true

tasks:
  setup:
    requirements:
      toolchains:
        - rust
    run: cargo fetch
  server:verify:
    run: //server:ci-unit
    execution:
      orchestrator:
        ref: mise
        mode: task
```

Rules:

- toolchain names must not be empty
- `version` must not be empty
- shared toolchain fields are `version`, `fulfillment`, `required`, `only_on`, and
  `platforms.<os>.version`; legacy `provider` is still accepted for compatibility, but it is no
  longer the canonical public shape
- shipped toolchain names are fixed: `toolchains.rust`, `toolchains.node`, `toolchains.java`,
  `toolchains.python`, `toolchains.go`, `toolchains.ruby`, and `toolchains.dotnet`
- `required` defaults to `true` and controls whether missing or mismatched toolchains are blocking
- ota validates and interprets toolchains through a shipped ownership contract for each toolchain
  name; the canonical shipped fulfillment sources are `rustup`, `corepack`, `sdkman`, `uv`, `go`,
  `ruby`, and `dotnet`
- supported `fulfillment.mode` values today are only `none` and `run`
- `fulfillment.mode: none` is the default diagnose-only lane; use it when ota should check the
  toolchain truth but not provision or activate it on the selected path
- `fulfillment.mode: run` allows selected `ota run` / workflow `ota up` execution paths to
  provision the declared toolchain on the run path; use it only when the selected repo path should
  let ota own fulfillment through the declared source
- `fulfillment.source` is optional when the toolchain uses its canonical shipped fulfillment path
- `fulfillment.source: mise` is the current non-canonical supported source for repos whose selected
  path is mediated by `mise`
- legacy flat `fulfillment: run` / `fulfillment: none` still parse for compatibility, but
  `ota validate` and `ota doctor` now warn and push authors onto structured `fulfillment`
- legacy `provider` must match the shipped canonical source for that toolchain name; mismatched
  legacy providers fail validation, and matching legacy providers now also warn so authors migrate
  onto canonical toolchain-owned structured `fulfillment`
- `profile`, `components`, and `targets` remain Rust-specific managed-surface fields
- `package_managers` remains the toolchain-owned package-manager surface for Node, Python, and
  Ruby where applicable
- `toolchains.python.package_managers` currently accepts `uv` and `poetry`; `uv` remains the
  canonical Python fulfillment source today, and ota can now use that same selected Python
  fulfillment lane to install declared Poetry versions on the selected run path before task
  execution
- standalone `tools.poetry` remains temporarily accepted for compatibility, but `ota validate`
  and `ota doctor` now warn and recommend migrating Poetry ownership to
  `toolchains.python.package_managers.poetry`
- `only_on`, when set, scopes the toolchain to `linux`, `macos`, or `windows`
- `platforms` may override `version`, `profile`, `components`, `package_managers`, and `targets`
  per OS using `linux`, `macos`, or `windows`
- `platforms` entries must also appear in `only_on` when `only_on` is declared
- `profile`, when set, must not be empty
- `components` and `targets` entries must not be empty
- for canonical Rust fulfillment with `fulfillment.mode: run`, `version` must be one installable Rustup
  toolchain reference such as `stable`, `beta`, `nightly`, or `1.94.0`
- for canonical Python fulfillment with `fulfillment.mode: run`, `version` must be one installable uv Python
  reference such as `3.12`, `3.12.10`, or `3.13`
- canonical Node, Java, Go, Ruby, and .NET fulfillment all support `fulfillment.mode: run` on the
  selected path; provisioning authority still stays with org policy and backend requirement
  fulfillment
- for canonical Ruby fulfillment with `fulfillment.mode: run`, ota currently uses the selected
  Ruby to hydrate the declared Bundler lane structurally via `ruby -S gem install bundler
  --no-document --version <constraint>` when `toolchains.ruby.package_managers.bundler` is
  declared
- `fulfillment.source: mise` with `fulfillment.mode: run` allows ota to use `mise install` for the
  selected toolchain and any declared package-manager entries on that path
- `fulfillment.source: mise` must not be combined with managed-surface fields such as
  `components` or `targets`
- duplicate ownership is invalid; if the same Rust capability is also declared under `runtimes`
  or `tools`, validation fails and the duplicate must be removed; the same applies to
  `toolchains.node` versus `runtimes.node` or `tools.node`, and `toolchains.python` versus
  `runtimes.python`, `toolchains.go` versus `runtimes.go`, `toolchains.ruby` versus
  `runtimes.ruby`, and `toolchains.dotnet` versus `runtimes.dotnet` or `tools.dotnet`

Ownership boundary:

- use `toolchains` for managed ecosystems
- use `orchestrators` when the repo has one declared manager that mediates trust, install, and
  task execution on the selected path
- use `runtimes` for simple unmanaged runtime version checks
- use `tools` for standalone commands on PATH
- use `native_prerequisites` for host-native build bundles and shell activation
- current shipped ownership is name-defined, not free-form: ota derives Rust capability ownership
  from `toolchains.rust`, Node runtime/executable plus declared package-manager ownership from
  `toolchains.node`, Java plus `javac` ownership from `toolchains.java`, Python runtime ownership
  from `toolchains.python`, Go runtime ownership from `toolchains.go`, Ruby runtime plus Bundler
  ownership from `toolchains.ruby`, and .NET runtime/CLI ownership from `toolchains.dotnet`

If a declared toolchain owns the capability, require the toolchain. Do not also require the same
runtime or tool unless it is deliberately standalone outside that toolchain.

For the full ownership model and migration examples, see
[toolchains-runtimes-tools.md](toolchains-runtimes-tools.md).

## `orchestrators`

Optional.

Use `orchestrators` when the repo has one declared manager that owns selected-path trust,
environment preparation, and mediated task execution.

Example:

```yaml
orchestrators:
  mise:
    kind: mise
    required: true
    config_files:
      - mise.toml
    activation:
      trust: true
    prepare:
      install: true

  devenv:
    kind: devenv
    required: true
    config_files:
      - devenv.nix
    launcher:
      exe: nix
      args:
        - run
        - github:cachix/devenv/main#devenv
        - --
```

Rules:

- orchestrator names must not be empty
- shipped orchestrator kinds are currently `mise`, `devbox`, and `devenv`
- `config_files` entries must not be empty
- `launcher.exe`, when declared, must not be empty
- `launcher.args`, when declared, must not contain empty entries
- `activation.trust: true` is currently supported only for `mise`
- `prepare.install: true` is currently supported for `mise` and `devbox`, but not `devenv`
- `launcher` is the first-class lane for repos whose orchestrator is executed through another host
  command instead of a direct PATH binary; ota then requires and probes the launcher executable on
  the selected path instead of the orchestrator name itself
- orchestrators do not replace `toolchains`; use `toolchains` for capability truth and
  `tasks.<name>.execution.orchestrator` when the selected task body must run through that manager

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
- when a managed ecosystem is already declared under `toolchains`, prefer that owner and avoid
  repeating the same capability here

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
Use `runtimes` for simple unmanaged runtime checks. For the ownership boundary with managed
toolchains and standalone tools, see [toolchains-runtimes-tools.md](toolchains-runtimes-tools.md).

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
    acquisition:
      provider: corepack
      package: pnpm
      version: "10.0.0"
  helm:
    version: ">=3.8"
    platforms:
      linux:
        acquisition:
          provider: apt
          package: helm
      macos:
        acquisition:
          provider: brew
          package: helm
          source_config:
            tap_name: vendor/tap
            tap_url: https://github.com/vendor/homebrew-tap
      windows:
        acquisition:
          provider: winget
          package: Helm.Helm
  pwsh:
    version: "7.6.0"
    only_on:
      - windows
  bun:
    version: ">=1.2.0"
    acquisition:
      provider: command
      shell: sh
      run: curl -fsSL https://bun.sh/install | sh
  yq:
    version: "4.52.5"
    acquisition:
      provider: release_asset
      source_config:
        asset_by_platform:
          linux_x86_64: https://example.com/releases/v{version}/yq_linux_amd64
          linux_aarch64: https://example.com/releases/v{version}/yq_linux_arm64
          macos_x86_64: https://example.com/releases/v{version}/yq_darwin_amd64
          macos_aarch64: https://example.com/releases/v{version}/yq_darwin_arm64
          windows_x86_64: https://example.com/releases/v{version}/yq_windows_amd64.exe
        version_args:
          - --version
  migrate:
    version: "4.19.1"
    acquisition:
      provider: release_asset
      source_config:
        asset_by_platform:
          macos_aarch64:
            url: https://github.com/golang-migrate/migrate/releases/download/v{version}/migrate.darwin-arm64.tar.gz
            archive:
              format: tar_gz
              executable_path: migrate
        version_args:
          - -version
```

Rules:

- tool names must not be empty
- versions must not be empty
- `required` defaults to `true` and controls whether missing or mismatched tools are blocking
- `only_on`, when set, scopes the tool to `linux`, `macos`, or `windows`
- `platforms` may override `version` and `acquisition` per OS using `linux`, `macos`, or `windows`
- `platforms` entries must also appear in `only_on` when `only_on` is declared
- `acquisition` optionally declares how ota can activate or provision the tool safely when a
  selected task/workflow requires it
- `acquisition.provider`: supported values are `corepack`, `command`, `release_asset`, `apt`,
  `brew`, `winget`, `choco`, and `scoop`
- `acquisition.package` and `acquisition.version` are required for `provider: corepack`
- `tool node` cannot use `provider: corepack`; declare Node under `toolchains.node` instead, and
  use structured `fulfillment` there when ota should own package-manager activation on the
  selected path.
- `acquisition.shell` and `acquisition.run` are required for `provider: command`
- `provider: release_asset` is provisioning-owned rather than activation-owned; it must declare
  `source_config.asset_by_platform`, may declare optional `source_config.version_args`, and must
  not declare `package`, `version`, `shell`, or `run`
- each `source_config.asset_by_platform.<platform>` entry may be either a direct asset URL string
  or an object with `url` plus optional archive extraction metadata
- `source_config.asset_by_platform.<platform>.archive.format` currently supports `tar_gz` and `zip`
- `source_config.asset_by_platform.<platform>.archive.executable_path` tells ota which file inside
  the extracted archive becomes the final executable in `.ota/state/source-managed/bin`
- package-manager-backed tool acquisition (`apt`, `brew`, `winget`, `choco`, `scoop`) is
  provisioning-owned rather than activation-owned; ota keeps the tool identity under `tools` and
  emits a provisioning request for the selected task/workflow path instead of treating the tool as
  a native prerequisite
- `release_asset` uses the tool key as the executable identity and projects an exact selected-path
  `release-asset` provisioning request instead of asking authors to hide binary downloads in shell
  glue
- package-manager-backed tool acquisition may also declare provider-owned `source_config` when the
  package manager itself needs explicit source truth such as a Homebrew tap, Winget source,
  Chocolatey feed, Scoop bucket, or apt sources list
- package-manager-backed tool acquisition may declare `package` when the install identifier differs
  from the tool key; it must not declare `version`, `shell`, or `run`
- `source_config` must not be empty and is valid on package-manager-backed acquisition and
  `release_asset`
- `provider: corepack` activates package-manager-managed tools such as `pnpm` through
  `corepack enable && corepack prepare <package>@<version> --activate`
- `provider: command` runs one explicit shell command as the acquisition lane for that tool; use it
  when the repo truth is "this tool becomes available through this command", not "install it any
  way you want"
- `provider: release_asset` tells ota to download an approved executable artifact into its
  source-managed tool path when the selected task/workflow path requires that tool; when a
  platform asset declares `archive`, ota downloads the archive, extracts the declared executable,
  and installs that executable into the same managed path
- selected task/workflow paths may still require a release-asset tool with a wildcard such as
  `tools: { yq: "*" }`; ota keeps the exact owned version from `tools.<name>.version` for doctor,
  dry-run, and execution instead of downgrading selected acquisition truth to the wildcard
- native follow-up diagnosis also checks ota's managed `.ota/state/source-managed/bin` tool path
  for `release_asset` tools after fulfillment, so repo-owned standalone binaries do not
  immediately misdiagnose as missing just because they were not installed into a host-global PATH
- package-manager-backed acquisition belongs in `tools`, not `native_prerequisites`; use
  `native_prerequisites` for host-native bundles such as compiler stacks, Xcode CLT, or Visual
  Studio Build Tools
- Corepack `package` and `version` values must be shell-safe tokens; use package names like `pnpm`
  and activation versions like `10.22.0`
- command acquisition `shell` may use `sh`, `bash`, `zsh`, `pwsh`, or `cmd`
- some tool keys map to different executables; for example, `tools.maven` is checked via `mvn`
- workspace overlays may specialize member tool requirements, but provenance must remain visible in diagnosis output
- when a managed ecosystem is already declared under `toolchains`, prefer that owner and avoid
  repeating the same capability here
- selected non-native task/workflow paths (container or remote) do not automatically inherit
  host-global `tools` fallback when no scoped tool requirements are declared; declare non-native
  tool requirements on the selected task path (`tasks.<name>.requirements.tools`) or selected
  execution context (`execution.contexts.<name>.requirements.tools`)
- selected execution contexts may also own toolchain capability truth directly through
  `execution.contexts.<name>.requirements.toolchains`; use that when a managed ecosystem such as
  Python or Node applies only on one named path instead of the whole repo

Use `only_on` to scope where a tool is required, and use `platforms` only when values change on a matching OS.
`required: false` keeps the tool active but downgrades missing/version mismatch findings to warnings.
Root fields act as the default values, and the matching `platforms.<os>` entry overrides them for that OS.
`acquisition` is attached to tool truth, while `tasks.<name>.requirements.tools` selects when that
tool actually applies. Use `tools.<name>.acquisition` for tool availability. Use
`native_prerequisites` for OS-native build bundles such as compilers or Visual Studio Build Tools.
Use `tools` for standalone commands, not for toolchain-owned capabilities such as Rustup-managed
`cargo` or `rustfmt`. For the ownership boundary, see
[toolchains-runtimes-tools.md](toolchains-runtimes-tools.md).

## `native_prerequisites`

Optional.

Use `native_prerequisites` for OS-native build-tool bundles that are not language runtimes or CLI
tools. This is the right fit for prerequisites such as Linux compiler packages, macOS Xcode Command
Line Tools, or Windows Visual Studio Build Tools. Ota diagnoses these through the selected
platform precondition check or a structured platform probe and gives OS-specific install guidance.
For selected native task paths, `ota up` and `ota run` may also fulfill declared package-manager
guidance from this surface (`apt`, `brew`, `winget`, `choco`, `scoop`) before rerunning
preconditions. Advisory host setup such as `xcode_clt`, `visual_studio`, `install`, and `note`
remains explicit guidance rather than a silent host mutation path.

Example:

```yaml
native_prerequisites:
  node-native-build-tools:
    description: Native compiler toolchain for packages with native addons
    platforms:
      linux:
        check: node-native-build-tools-linux
        apt:
          - build-essential
          - python3
      macos:
        check: node-native-build-tools-macos
        xcode_clt: true
      windows:
        visual_studio:
          components:
            - Microsoft.VisualStudio.Component.VC.Tools.x86.x64
        requires:
          runtimes:
            python: ">=3.10"
        winget:
          - Microsoft.VisualStudio.2022.BuildTools
        activation:
          kind: visual_studio_dev_shell
          arch: x64
  nix-shell:
    description: Repo-local shell environment that exports compiler and package paths
    platforms:
      linux:
        check: nix-shell-ready
        activation:
          kind: command
          shell: bash
          run: source .env/nix-shell.sh

checks:
  - name: node-native-build-tools-linux
    kind: precondition
    severity: error
    run: sh -c "cc --version && python3 --version"
  - name: node-native-build-tools-macos
    kind: precondition
    severity: error
    run: sh -c "xcode-select -p && python3 --version"
  - name: nix-shell-ready
    kind: precondition
    severity: error
    run: env | grep NIX_CC

tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
```

Rules:

- each native prerequisite must declare at least one `platforms.<os>` guidance entry
- `native_prerequisites.<name>.check` may provide a shared fallback check; otherwise each
  `platforms.<os>.check` must reference a declared `kind: precondition` check, unless the
  selected platform entry declares a structured Ota-owned probe such as `windows.visual_studio`
- `platforms` may use `linux`, `macos`, and `windows`
- platform entries may declare `apt`, `brew`, `winget`, `choco`, `scoop`, `xcode_clt`,
  `visual_studio`, `activation`, `install`, or `note` guidance
- `platforms.windows.visual_studio.components` lists Visual Studio Installer component IDs that
  Ota checks with `vswhere`; this is preferred over embedding a raw PowerShell `vswhere` command
  in `checks.run`
- `platforms.<os>.requires` may declare `runtimes`, `tools`, `toolchains`, `env`, and `checks`
  that belong to that native prerequisite on that host OS; tasks should reference the native
  bundle through `requirements.native` instead of duplicating those checks at the task level
- `activation.kind: visual_studio_dev_shell` is the Windows MSVC activation hint for checks
  and native task execution that require `cl`/MSVC tools from a Visual Studio Developer shell;
  `arch` defaults to `x64`
- `activation.kind: command` is the generic task-scoped shell-environment activation form; it
  must declare both `shell` and `run`, and ota executes that shell activation before the selected
  native check or native task body
- when a native task path selected by `ota up` or `ota run` references a Windows native
  prerequisite with `activation.kind: visual_studio_dev_shell`, ota runs that task inside the
  activated Developer Shell instead of assuming the user already opened one manually
- when a native task path selected by `ota up` or `ota run` references
  `activation.kind: command`, ota captures the declared shell environment and applies it to the
  selected native task body; the same activation also wraps the selected precondition check path
- when one task references multiple native prerequisites for the same platform, any declared
  activation hints must agree; conflicting activation kinds or architectures are rejected
- Ota only evaluates native prerequisites selected by `tasks.<name>.requirements.native`
- `ota doctor` remains non-mutating for native prerequisites
- `ota up` and `ota run` may fulfill selected native prerequisite package-manager guidance from
  `apt`, `brew`, `winget`, `choco`, and `scoop`, then rerun preconditions
- keep package-manager lanes aligned to the platform entry they live under; for example, prefer
  `apt` under `linux`, `brew` under `macos`, and `winget` / `choco` / `scoop` under `windows`
  instead of mixing likely wrong-OS package-manager lanes into the same platform entry
- avoid mixing opaque `install` shell glue with manager-owned package lanes on the same platform
  entry when the shell command is only there to install host packages; keep package-manager truth
  under `apt` / `brew` / `winget` / `choco` / `scoop`, and reserve `install` for the remaining
  manual step only when no first-class lane owns it yet
- when an org policy pack is active, native prerequisite package fulfillment must also be approved
  under `policies.native_packages.<manager>.approved`; Ota does not silently bypass that policy
  gate
- use policy-backed provisioning for standalone tools or runtimes Ota is allowed to install
  through approved source selection; use `policies.native_packages` for host package-manager
  bundles owned by `native_prerequisites`

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
  profiles:
    docker-build:
      sources:
        - kind: dotenv
          path: .env.docker-build
      env:
        REDIS_HOST: redis
        NEXTAUTH_URL: http://web:3000
      render:
        dotenv:
          path: .env.docker-build
          include:
            - DATABASE_URL
```

Fields:

- `vars`: env-variable requirements keyed by env name
- `sources`: ordered declared env sources
- `profiles`: optional reusable env overlay profiles keyed by profile name

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

`env.profiles.<name>` fields:

- `sources`: optional ordered declared env sources prepended for the selected profile
- `env_files`: optional ordered repo-relative dotenv overlays injected into selected workflow tasks
  before task-owned `env_files`
- `env`: optional literal env overlay injected into selected workflow tasks before task-owned `env`
- `render.dotenv.path`: optional repo-relative dotenv artifact path ota should materialize during
  workflow execution
- `render.dotenv.template`: optional repo-relative dotenv template ota should use as the base
  content before applying rendered env overlays; keep it separate from `render.dotenv.path`
- `render.dotenv.include`: optional ordered env names ota should resolve and emit into that dotenv
  artifact
- `render.files[]`: optional ordered structured artifact renders ota should materialize during
  workflow execution
- `render.files[].path`: required repo-relative output path for the rendered artifact
- `render.files[].format`: required structured format; current shipped values are `json` and `toml`
- `render.files[].sources`: required ordered repo-relative source chunks merged from lowest to
  highest precedence after placeholder substitution
- `render.files[].merge_into_existing`: optional boolean; when true, ota treats an existing output
  file as the lowest-precedence merge layer before applying the declared source chunks

Profile rules:

- profiles do not replace repo-wide `env.vars`; they specialize how one selected workflow path
  resolves and injects env truth
- profile `sources` are prepended ahead of root `env.sources` so profile-owned source truth wins
  before falling back to the repo baseline
- profile `env_files` and `env` are low-precedence workflow overlays: task-owned `env_files`,
  task `env`, and mode-branch `env` still win when they declare the same values
- rendered dotenv artifacts are injected automatically into selected workflow tasks as the first
  profile-owned `env_file`; do not duplicate the same path in `env_files`
- rendered dotenv artifacts are re-rendered deterministically on each workflow-owned execution
  path before they are consumed: `ota up`, `ota proof runtime`, and direct `ota run` for tasks in
  the selected workflow task closure all materialize the artifact from contract truth
- when `render.dotenv.template` is declared, ota starts from that template and then replaces the
  declared profile/env keys, so workflow-owned compose interpolation no longer requires a separate
  `ensure_env_file` prepare task
- rendered structured artifacts are also re-materialized deterministically on workflow-owned
  execution paths, so generated JSON/TOML config truth no longer needs repo-local merge helpers
- structured render source chunks stay immutable repo truth; ota writes only the declared
  `render.files[].path` artifact and merges later `sources[]` entries over earlier ones
- selected workflow-instance env overlays participate in structured render placeholder substitution,
  so one workflow family can materialize per-instance client config without flattening that truth
  into helper scripts
- use profiles when one workflow/runtime shape needs a truthful env overlay without hiding that
  selection in shell glue or ad hoc setup notes

Example:

```yaml
version: 1
project:
  name: workflow-rendered-env
env:
  vars:
    DATABASE_URL:
      required: true
  profiles:
    compose:
      sources:
        - kind: dotenv
          path: .env.local
      env:
        REDIS_HOST: redis
      render:
        dotenv:
          path: .env.compose
          template: .env.example
          include:
            - DATABASE_URL
services:
  api:
    manager:
      kind: compose
      name: local
      file: docker-compose.yml
      service: api
      env_file: .env.compose
tasks:
  dev:
    run: pnpm dev
    requires_services:
      - api
workflows:
  default: compose-dev
  compose-dev:
    env:
      profile: compose
    run:
      task: dev
```

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

```yaml
env:
  vars:
    DATABASE_URL:
      secret: true
    POSTGRES_PASSWORD:
      secret: true
execution:
  contexts:
    host:
      backend: native
services:
  postgres:
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432

tasks:
  test:
    requires_services:
      - postgres
    env_bindings:
      DATABASE_URL:
        from_service:
          service: postgres
          view: host
          format: url
          scheme: postgres
          username: postgres
          password_env: POSTGRES_PASSWORD
          database: app_test
      DB_HOST:
        from_service:
          service: postgres
          view: host
          format: host
```

This derives task env values from declared service endpoints. It is useful when the same task can
run natively or inside a container: Ota keeps the service dependency in `requires_services`, then
projects the selected service view into env without hand-writing container host aliases such as
`host.docker.internal`.

When a service exposes more than one endpoint in the same view/context, declare
`from_service.endpoint` to pick the exact named service endpoint instead of relying on view-only
selection.

Use `password_env` for real credentials. The referenced env var must be declared under `env.vars`
with `secret: true`, and the generated env value (for example `DATABASE_URL`) must also be declared
secret so Ota can redact it in receipts and summaries.

Literal `password` is supported only for local/dev fixtures with non-sensitive credentials, for
example a disposable `postgres` password in a test container:

```yaml
env:
  vars:
    DATABASE_URL:
      secret: true
execution:
  contexts:
    host:
      backend: native
services:
  postgres:
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432

tasks:
  test:
    env_bindings:
      DATABASE_URL:
        from_service:
          service: postgres
          format: url
          scheme: postgres
          username: postgres
          password: postgres
          database: app_test
```

Do not store production, shared, or personal credentials in `password`; use `password_env` instead.

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
  setup:docker:images:
    description: Pre-pull registry-backed docker dependencies
    category: setup
    prepare:
      kind: dependency_hydration
      medium: container_images
      source:
        kind: docker_compose
        engine: docker
        cwd: docker
        files:
          - docker-compose.base.yml
          - docker-compose.dev.yml
        env_files:
          - .env.compose
      targets:
        - redis
        - database
    requirements:
      tools:
        docker: "*"
    effects:
      network: true
      network_kind: container_image_hydration
      external_state:
        - docker
  setup:container:deps:
    description: Hydrate container-owned dependencies through a declared Compose service
    category: setup
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: node_package_manager
        cwd: app
        manager: npm
        mode: ci
        compose:
          kind: run
          service: api
          workdir: /workspace/app
          rm: true
    requirements:
      tools:
        docker: "*"
    effects:
      adapter_state:
        - compose_volume:node_modules
      network: true
      network_kind: dependency_hydration
  setup:
    description: Install dependencies
    category: setup
    run: pnpm install
    safe_for_agent: true
    effects:
      writes:
        - node_modules
  build:
    context: app
    requires_services:
      - postgres
    depends_on:
      - setup
    run: pnpm build
  db:migrate:
    context: host
    adapter_inputs:
      overlays:
        compose:
          cwd: docker
          files:
            - docker-compose.yml
    compose:
      kind: exec
      detach: true
      service: api
      workdir: /workspace
      exe: bundle
      args:
        - exec
        - rails
        - db:migrate
    requirements:
      tools:
        docker: "*"
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
  verify:
    description: Run the canonical verification entrypoint
    aggregate:
      tasks:
        - build
        - upload
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

Compose execution notes:

- use `tasks.<name>.compose` when the truthful task body is a finite `docker|podman compose`
  lane ota should own structurally, whether that is a service-side `exec`/`run`/`attach` command
  or a staged `compose up`, `compose build`, or project-scoped `compose down` adapter invocation
- use `compose.kind: attach` when the truthful lane re-attaches to an existing interactive
  session inside a declared Compose service, such as `tmux attach` inside a running dev container
- `compose.detach: true` is supported for `kind: exec` when the truthful lane starts a detached
  in-service bootstrap or background process and Ota should own that launch directly; `kind: up`
  also supports `detach: true` for staged service-group bring-up
- `compose.rm: true` remains valid only for `kind: run`

Fields:

- `description`: optional string
- `notes`: optional multiline guidance for humans and agents
- `category`: optional string
- `env`: optional map of fixed task-scoped environment overrides
- `env_bindings`: optional map of task env values derived from declared services
- `inputs`: optional map of named task inputs
- `context`: optional execution context name
- `run`: optional string for a single shell-compatible command
- `script`: optional string for an inline multiline shell script
- `command`: optional structured finite command body
- `compose`: optional structured Compose execution body for `docker|podman compose exec/run/attach/up/down/build`
- `prepare`: optional first-class finite preparation body for machine-readable setup or dependency hydration
- `launch`: optional structured launch source for inspectable command or packaged container starts
- `action`: optional first-class native setup action for small cross-platform repo-file mutations
- `aggregate`: optional first-class aggregate body for named dependency-closure entrypoints
- `effects`: optional structured side-effect metadata for the task body
- `requirements`: optional task-scoped prerequisite surface for this executable path
- `execution`: optional mode-aware execution branches for one task intent
- `when`: optional execution-guard conditions for the selected task node
- `runtime`: optional long-running workload shape for endpoint-bearing tasks
- `runtime_boundary`: optional canonical runtime sandbox baseline for this task lane
- `variants`: optional list of conditional task executions
- `requires_services`: optional list of service names that must be ready before the task body runs
- `requires_artifacts`: optional list of named generated artifacts the task consumes; each consumer
  must directly depend on the named artifact's producer task
- `replay_inputs`: optional repo-owned replay identity artifacts captured before the selected task
  or workflow closure begins; each entry declares `id`, a repo-relative `path`, and one of:
  - `kind: static_file` for a generic immutable repo file the deterministic lane consumes directly,
    such as a frozen fixture, SQLite store, lockfile-shaped baseline, or other named replay input;
    these are narrowing replay evidence
  - `kind: presentation_profile` for a declared execution-presentation profile file when the lane's
    output shape depends on a named rendering or normalization policy; matching identity closes only
    the named presentation-semantics class for that lane
  - `kind: comparator_profile` for declared comparison semantics such as equivalence, tolerance,
    ignored labels, or threshold posture; matching identity narrows comparator drift but does not
    by itself make the lane hermetic
  - `expected_identity`: optional immutable `sha256:<64 lowercase hex characters>` pin. When
    declared, Ota hashes the input before execution and blocks the selected closure before any task
    starts if the observed identity differs. Ota never rewrites this value; intentional input
    changes require a reviewed contract update.
  replay inputs cannot overlap declared closure writes
- replay-input kind guide:
  - use `static_file` when the file is part of the deterministic selected-lane input set
  - use `presentation_profile` when the file controls how runtime output is shaped or normalized
    before comparison
  - use `comparator_profile` when the file controls how Ota should compare or judge the result
- example:

  ```yaml
  verify:
    replay_inputs:
      - id: recorded_sql
        kind: static_file
        path: data/fixture.jsonl
        expected_identity: sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef
      - id: frozen_store
        kind: static_file
        path: data/store.db
      - id: runtime-presentation
        kind: presentation_profile
        path: replay/presentation-profile.yaml
      - id: equivalence
        kind: comparator_profile
        path: replay/comparator-profile.yaml
  ```

- `witnessed_observations.query_traces`: optional JSONL query-trace artifacts emitted by a prior
  or external execution; each entry declares an `id` and repo-relative `path`. Ota preserves
  these as attested observations in the receipt rather than treating per-query identities as
  current-run replay inputs. Each nonblank line must contain `id` (subject), `run` (non-negative
  integer), and `sql` (query text); the selected closure must not write the trace path.
- `depends_on`: optional list of task names
- `only_on`: optional host OS inclusion list (`linux`, `macos`, `windows`). When declared, the
  task and any selected dependency closure that reaches it are unavailable on other hosts; Ota
  refuses before provisioning or execution. Use this for platform support truth. Keep
  `variants.<i>.when.os` for choosing an alternate body on a supported host.
- `safe_for_agent`: optional boolean (`false` when omitted)
- `internal`: optional boolean; marks orchestration plumbing tasks that stay in the graph but are hidden from default `ota tasks` discovery surfaces

`effects` fields:

- `writes`: optional list of normalized relative paths the task body is expected to mutate
- `workspace_writes`: optional list of normalized workspace-relative paths the task body is
  expected to mutate when the runnable path truthfully writes outside the repo root, such as a
  sibling checkout at `../ui`
- `network`: optional boolean; set `true` when the task requires network access or reaches out to
  remote services during execution
- `network_kind`: optional network lane classifier (`broad`, `dependency_hydration`,
  `container_image_hydration`, `service_readiness`, `integration_test`, or `tool_bootstrap`) for
  networked task paths
- `adapter_state`: optional list of lowercase `<adapter_family>:<state_name>` tokens naming
  durable adapter-owned state the task mutates, such as `compose_volume:bundle_data`
- `external_state`: optional list of lowercase tokens naming out-of-repo state the task mutates,
  such as `docker` or `postgres`

Task-effect rules:

- use `effects.writes` for durable repo paths the task mutates directly
- use `effects.workspace_writes` when the task mutates sibling or workspace-relative filesystem
  paths outside the repo root but still inside the intended local workspace layout
- use `effects.network: true` when the task depends on networked fetches or remote calls and that
  dependency should stay explicit for CI and agent execution
- use `effects.network_kind: dependency_hydration` for finite package, module, chart, or other
  dependency acquisition lanes such as lockfile-backed package-manager install; keep
- use `effects.network_kind: container_image_hydration` for Compose-managed image pull lanes,
  including `prepare.medium: container_images` and Compose startup paths that may pull declared
  images; keep image identity and runtime receipt evidence explicit rather than collapsing it into
  package dependency hydration
- use `effects.network_kind: service_readiness` for finite health or protocol assertions against
  an endpoint of a declared repo-managed service; pair it with `requires_services` and keep it
  distinct from external integration truth
- use `effects.network_kind: integration_test` for live, staging, or remote-backed verification
  lanes that depend on real services, credentials, or seeded environments beyond the local repo
- test tasks that declare `requires_services` should classify a repo-managed endpoint probe as
  `service_readiness` and a live, staging, or remote-backed lane as `integration_test`;
  validate/doctor surface an advisory when that service-verification truth is left broad or omitted
- use `effects.network_kind: tool_bootstrap` for finite contract-owned tool installation lanes
  such as bootstrapping `uv` through `pip`; keep
  `effects.network_kind: broad` (or omit `network_kind`) for wider API/remote-call execution
- use `effects.external_state` when the task mutates state outside the repo filesystem, such as
  Docker resources, databases, or hosted services
- use `effects.adapter_state` when durable state lives behind an adapter boundary instead of a
  repo path, such as a Compose volume that persists Bundler gems or `node_modules`
- `effects.network_kind` requires `effects.network: true`
- keep entries relative, normalized, and free of `..` segments
- keep `effects.workspace_writes` entries normalized and workspace-relative; they may include
  `..` segments for sibling layout truth, but must not be absolute or drive-prefixed
- keep `effects.adapter_state` entries as lowercase `<adapter_family>:<state_name>` tokens so
  adapter-owned durability stays machine-readable instead of collapsing back to prose
- keep `effects.external_state` entries as lowercase tokens so the side-effect surface stays
  machine-readable instead of turning into prose
- prefer shipped canonical `effects.external_state` tokens when they fit:
  `docker`, `postgres`, `redis`, `mysql`, `mariadb`, `kafka`, `rabbitmq`, `elasticsearch`,
  `opensearch`, `s3`, `gcs`, `azure_blob`, `cloudflare`, `kubernetes`, `terraform`
- avoid obvious repo-local aliases when a shipped token already exists; for example use `docker`
  instead of `docker_compose`, `postgres` instead of `postgresql`, and `kubernetes` instead of
  `k8s`
- `effects.writes` is contract truth for agent-safety review, not a log of every transient scratch file
- `effects.workspace_writes` is the explicit widening for sibling/workspace materialization truth;
  do not weaken `effects.writes` just to encode out-of-repo paths
- when a task is agent-safe, declared writes should stay inside `agent.writable_paths` when that boundary is declared
- agent-safe task writes must not overlap `agent.protected_paths`
- agent-safe tasks must not declare `effects.workspace_writes` today; repo-scoped
  `agent.writable_paths` / `agent.protected_paths` do not yet model sibling workspace boundaries

`aggregate` fields:

- `aggregate.tasks`: required non-empty ordered list of task names ota should execute as the aggregate body

`aggregate` rules:

- use `aggregate` when the task is only a named dependency-closure entrypoint and does not have its own command body
- `aggregate` is a task body, so it is mutually exclusive with `run`, `script`, `command`, `compose`, `prepare`, `launch`, and `action`
- `aggregate.tasks` entries must resolve to known tasks
- `aggregate.tasks` order is preserved during execution
- aggregate-member failures become the aggregate task result
- `aggregate` must not be combined with task-local execution fields such as `context`, `env`, `inputs`, `targets`, `requires_services`, `runtime`, `effects`, `requirements`, `variants`, or `execution`
- aggregate tasks are user-facing entrypoints; other tasks should not reference them from `depends_on` or hook edges
- `aggregate.tasks` must reference executable child tasks, not other aggregate entrypoints
- keep prerequisite edges in the child tasks; do not duplicate aggregate membership under `depends_on`

`compose` fields:

- `compose.kind`: required Compose execution shape; ota ships `exec`, `run`, `attach`, `up`, `down`, `build`, `stop`, `restart`, `rm`, `logs`, and `ps`
- `compose.engine`: optional Compose CLI engine; `docker` by default, `podman` also supported
- `compose.service`: required Compose service name for `kind: exec`, `run`, or `attach`
- `compose.services`: optional ordered service list for `kind: up` or `kind: build`
- `compose.workdir`: optional in-container working directory passed through `compose exec/run -w`
- `compose.exe`: required executable to run inside the selected service for `kind: exec`, `run`, or `attach`
- `compose.args`: optional argument list passed to `compose.exe` for `kind: exec`, `run`, or `attach`
- `compose.rm`: optional `compose run --rm`; valid only with `kind: run`
- `compose.build`: optional `compose run --build`; valid only with `kind: run`
- `compose.service_ports`: optional `compose run --service-ports`; valid only with `kind: run`
- `compose.force_recreate`: optional `compose up --force-recreate`; valid only with `kind: up`
- `compose.force`: optional `compose rm -f`; valid only with `kind: rm`
- `compose.follow`: optional `compose logs -f`; valid only with `kind: logs`
- `compose.remove_volumes`: optional `compose down -v`; valid only with `kind: down`
- `compose.timeout_seconds`: optional `compose down -t <seconds>` graceful shutdown timeout;
  valid only with `kind: down`
- `compose.tty`: optional TTY preservation for `exec` and `run`; omitted means ota adds `-T` for deterministic non-interactive execution. `kind: attach` is always interactive and preserves TTY by definition.

`compose` rules:

- use `compose` when the repo truth is a service-side finite command such as `bundle exec rails db:migrate`, `python manage.py migrate`, or `npm run lint` inside a declared Compose service, or when one finite staged task truthfully owns `docker compose up [-d] [--force-recreate] <services...>`, `docker compose build [services...]`, `docker compose restart [services...]`, `docker compose rm [-f] [services...]`, `docker compose logs [-f] [services...]`, `docker compose ps [services...]`, or `docker compose down`
- `compose` is a task body, so it is mutually exclusive with `run`, `script`, `command`, `prepare`, `launch`, `action`, and `aggregate`
- `compose` currently requires `requirements.tools.docker` or `requirements.tools.podman` to keep the host Compose engine truthful
- keep host-side Compose adapter truth under `adapter_inputs.compose` or workflow overlays; keep service-side command truth or staged compose subcommand truth under `compose`
- `compose.kind: exec`, `run`, and `attach` require `compose.service`
- `compose.kind: up` uses `compose.services` for staged service-group activation and must not declare `compose.service`
- `compose.kind: build` uses optional `compose.services` for staged image build selection and must not declare `compose.service`
- `compose.kind: stop`, `restart`, `rm`, `logs`, and `ps` use optional `compose.services` for staged service selection and must not declare `compose.service`
- `compose.kind: down` is project-scoped and must not declare `compose.service` or `compose.services`
- `compose.workdir`, `compose.exe`, and `compose.args` only apply to `compose.kind: exec`, `run`, or `attach`
- `compose.rm` is only valid with `compose.kind: run`
- `compose.build` is only valid with `compose.kind: run`
- `compose.service_ports` is only valid with `compose.kind: run`
- `compose.force_recreate` is only valid with `compose.kind: up`
- `compose.force` is only valid with `compose.kind: rm`
- `compose.follow` is only valid with `compose.kind: logs`
- `compose.remove_volumes` is only valid with `compose.kind: down`
- `compose.timeout_seconds` is only valid with `compose.kind: down`
- `compose.detach` is only valid with `compose.kind: exec` or `compose.kind: up`
- `compose.kind: attach` must not declare `compose.rm` or `compose.detach`

`prepare` fields:

- `prepare.kind`: required preparation classifier; ota currently ships `dependency_hydration`, `tool_bootstrap`, and `sequence`
- `prepare.kind: sequence`
  - `prepare.steps`: required non-empty ordered list of child setup steps
  - `prepare.steps.kind`: ordered step kind; `sequence` accepts the structural prepare kinds
    (`dependency_hydration`, `tool_bootstrap`, `sequence`) plus deterministic native bootstrap
    mutations (`copy_if_missing`, `ensure_env_file`, `ensure_file`, `ensure_directory`,
    `ensure_git_checkout`, `ensure_git_template`, `ensure_container_network`, `reset_compose_service_volume`)
- `prepare.kind: tool_bootstrap`
  - `prepare.tool`: required bootstrap target; ota currently ships `uv`, `playwright_browsers`, and `cypress_browsers`
  - `prepare.browsers`: optional explicit Playwright browser subset; ota currently ships `chromium`, `firefox`, `webkit`, `chrome`, and `msedge`
  - `prepare.with_deps`: optional Playwright dependency lane for `playwright install --with-deps`
  - `prepare.source.kind: pip`
  - `prepare.source.exe`: required Python executable ota should use for `-m pip install ...`
  - `prepare.source.kind: poetry`
  - `prepare.source.cwd`: required repo-relative working directory for `poetry run ...` bootstrap
  - `prepare.source.kind: node_package_manager`
  - `prepare.source.cwd`: required repo-relative working directory for the bootstrap invocation
  - `prepare.source.manager`: required package manager; ota currently ships `npm`, `pnpm`, `yarn`, and `bun`
  - `prepare.source.filter`: optional workspace/package selector for manager-owned targeted hydration or bootstrap; ota currently only owns this shape for `manager: pnpm` and renders it before the manager action
- `prepare.kind: dependency_hydration`
  - `prepare.medium: container_images`
    - `prepare.source.kind: docker_compose`
    - `prepare.source.engine`: optional compose CLI engine; `docker` by default, `podman` also supported
    - `prepare.source.cwd`: required repo-relative working directory for the compose invocation
    - `prepare.source.file`: optional single-file compatibility alias relative to `prepare.source.cwd`
    - `prepare.source.files`: optional ordered compose file stack relative to `prepare.source.cwd`; declare at least one compose file through `prepare.source.file` or `prepare.source.files`
    - `prepare.source.env_files`: optional ordered compose interpolation env-file stack relative to `prepare.source.cwd`
    - `prepare.targets`: required non-empty list of concrete dependencies ota should hydrate
  - `prepare.medium: package_dependencies`
    - `prepare.source.kind: node_package_manager`
    - `prepare.source.cwd`: required repo-relative working directory for the install invocation
    - `prepare.source.manager`: required package manager; ota currently ships `npm`, `pnpm`, `yarn`, and `bun`
    - `prepare.source.mode`: required install mode; ota currently ships `install` and `ci`
    - `prepare.source.frozen_lockfile`: optional explicit lockfile strictness for `pnpm install --frozen-lockfile`, `yarn install --immutable`, or `bun install --frozen-lockfile`
    - `prepare.source.inline_builds`: optional explicit Yarn inline-build ownership for `yarn install --inline-builds`; valid only with `manager: yarn` and `mode: install`
    - `prepare.source.force`: optional explicit npm override ownership for `npm install --force` or `npm ci --force`; valid only with `manager: npm` and `mode: install` or `mode: ci`, and should be treated as an exceptional hydration lane rather than a normal default
    - `prepare.source.compose`: optional Compose invocation wrapper for typed hydration through a declared service container
      - `prepare.source.compose.kind`: required Compose execution shape; ota ships `exec` and `run`
      - `prepare.source.compose.engine`: optional Compose CLI engine; `docker` by default, `podman` also supported
      - `prepare.source.compose.service`: required Compose service name
      - `prepare.source.compose.workdir`: optional in-container working directory passed through `compose exec/run -w`
      - `prepare.source.compose.rm`: optional `compose run --rm`; valid only with `kind: run`
      - `prepare.source.compose.tty`: optional TTY preservation; omitted means ota adds `-T` for deterministic non-interactive execution
    - `prepare.source.kind: bundler`
    - `prepare.source.cwd`: required repo-relative working directory for the Bundler invocation
    - `prepare.source.path`: optional repo-relative bundle install path; required for the repo-local gem lane, omitted for compose-wrapped lanes that truthfully use the container-default Bundler path
    - `prepare.source.kind: composer`
    - `prepare.source.cwd`: required repo-relative working directory for the Composer install invocation
    - `prepare.source.kind: uv`
    - `prepare.source.cwd`: required repo-relative working directory for the uv hydration invocation
    - `prepare.source.mode`: optional uv hydration mode; ota currently ships `sync`, `pip_requirements`, and `pip_local_project`
    - `prepare.source.default_index`: optional authoritative default Python package index projected as `uv --index-url <url>`
    - `prepare.source.indexes`: optional ordered additional Python package indexes projected as repeated `uv --extra-index-url <url>`
      for compatibility across supported uv releases
    - `prepare.source.offline`: optional cache-only hydration posture projected as `uv --offline`; this prevents network access but does not make an undeclared local cache source replayable
    - `prepare.source.requirements_file`: required repo-relative requirements file when `prepare.source.mode: pip_requirements`; omit it for `mode: sync`
    - `prepare.source.local_project`: required source-`cwd`-relative local project declaration when `prepare.source.mode: pip_local_project`
      - `prepare.source.local_project.path`: required source-`cwd`-relative directory containing the local project's `pyproject.toml`; `..` segments are valid only when the resolved path remains inside the contract root
      - `prepare.source.local_project.editable`: optional editable posture projected as `uv pip install -e`
      - `prepare.source.local_project.extras`: optional ordered extras list projected onto the declared local project target
      - `prepare.source.local_project.groups`: optional ordered dependency groups projected as separate `uv pip install --group <path>/pyproject.toml:<group>` invocations after the primary local-project install
      - `prepare.source.local_project.lockfile`: optional source-`cwd`-relative lockfile whose identity Ota captures alongside the local project manifest before execution; `..` segments are valid only when the resolved path remains inside the contract root
      - Ota separately records a clean Git source identity for the local project when it is available. An editable local project is replay-acquitting only when both receipts carry resolved hydration posture, its declared lockfile identity, and its clean source identity. A missing lockfile or dirty/unavailable source remains narrowing evidence; Doctor reports that boundary.
    - `prepare.source.kind: poetry`
    - `prepare.source.cwd`: required repo-relative working directory for the Poetry install invocation
    - `prepare.source.groups`: optional dependency-group list for `poetry install --with ...` or `--only ...`
    - `prepare.source.group_mode`: optional group selector; ota currently ships `with` and `only`
    - `prepare.source.no_root`: optional `poetry install --no-root`
    - `prepare.source.kind: go_modules`
    - `prepare.source.cwd`: required repo-relative working directory for the `go mod download` invocation
    - `prepare.source.kind: helm`
    - `prepare.source.cwd`: required repo-relative working directory for the `helm dependency build .` invocation
    - `prepare.source.kind: maven`
    - `prepare.source.cwd`: required repo-relative working directory for the Maven invocation
    - `prepare.source.wrapper`: optional explicit wrapper ownership; when `true`, ota runs `./mvnw ...`, otherwise it runs `mvn ...`
    - `prepare.source.mode`: optional Maven hydration goal; ota currently ships `resolve` and `go_offline`
    - `prepare.source.skip_tests`: optional `-DskipTests` for hydration lanes that should keep test execution disabled while Maven resolves dependencies
    - `prepare.source.kind: gradle`
    - `prepare.source.cwd`: required repo-relative working directory for the Gradle invocation
    - `prepare.source.wrapper`: optional explicit wrapper ownership; when `true`, ota runs `./gradlew dependencies`, otherwise it runs `gradle dependencies`
    - `prepare.source.kind: cargo`
    - `prepare.source.cwd`: required repo-relative working directory for the `cargo fetch` invocation
    - `prepare.source.kind: dotnet_restore`
    - `prepare.source.cwd`: required repo-relative working directory for the `dotnet restore` invocation
    - `prepare.source.config_file`: optional repo-relative NuGet config passed as `dotnet restore --configfile <path>`
    - `prepare.source.sources`: optional explicit restore feeds passed as repeated `dotnet restore --source <url>` arguments

`prepare` rules:

- use `prepare` when the task is a finite setup phase ota should understand structurally instead of as opaque shell glue
- `prepare` is an executable task body, so a task may declare `prepare` without `run`
- `prepare` still needs explicit `requirements` and `effects`; ota should understand both intent and side effects
- `prepare.kind: sequence` uses the parent task's `requirements`, `effects`, and execution path for each ordered child step
- `prepare.kind: sequence` must declare at least one child step under `prepare.steps`
- use `prepare.kind: sequence` when one honest setup lane needs more than one typed finite step,
  such as env-file materialization plus Playwright browser bootstrap, or Node hydration plus
  Python hydration in one repo-level `setup` task
- `prepare.kind: tool_bootstrap` currently requires `effects.network: true` and `effects.network_kind: tool_bootstrap`; add toolchain requirements that match the selected bootstrap source
- `prepare.kind: tool_bootstrap` with `source.kind: pip` currently requires `requirements.toolchains: [python]`
- `prepare.kind: tool_bootstrap` with `source.kind: poetry` currently requires `requirements.toolchains: [python]` and currently only supports `prepare.tool: playwright_browsers`
- `prepare.kind: tool_bootstrap` with `source.kind: node_package_manager` currently requires `requirements.toolchains: [node]` and currently only supports `prepare.tool: playwright_browsers` or `prepare.tool: cypress_browsers`
- `prepare.kind: tool_bootstrap` with `prepare.browsers` currently only applies to `prepare.tool: playwright_browsers`
- `prepare.kind: tool_bootstrap` with `prepare.with_deps: true` currently only applies to `prepare.tool: playwright_browsers`
- `prepare.source.filter` currently only applies to `source.kind: node_package_manager` with `manager: pnpm`; dependency hydration uses it for a scoped `pnpm --filter <selector> install`, while browser tool bootstrap uses it for its selected package scope
- use `prepare.kind: tool_bootstrap` when the task truth is contract-owned tool installation rather than repo dependency hydration; for example bootstrapping `uv` through `pip` or downloading Playwright or Cypress browsers through a repo-owned Node package manager
- `prepare.kind: dependency_hydration` with `medium: container_images` and `source.kind: docker_compose` requires `requirements.tools.docker`, `effects.network: true`, and `effects.network_kind: container_image_hydration`; keep the declared Compose files and selected image targets explicit so image-pull truth is not collapsed into package dependency hydration
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` requires `effects.network: true` and `effects.network_kind: dependency_hydration`; add tool or toolchain requirements that match the selected hydration source and wrapper
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: node_package_manager` currently requires `requirements.toolchains: [node]`, `effects.network: true`, `effects.network_kind: dependency_hydration`, and durable state in `effects.writes` or `effects.adapter_state`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Node toolchain truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: bundler` currently requires `requirements.toolchains: [ruby]`, `effects.network: true`, `effects.network_kind: dependency_hydration`, and durable state in `effects.writes` or `effects.adapter_state`; host-side repo-local gem hydration also requires `prepare.source.path`, while compose-wrapped lanes may omit it when Bundler truthfully uses the container-default install path; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Ruby toolchain truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: composer` currently requires `requirements.tools.composer`, `effects.network: true`, `effects.network_kind: dependency_hydration`, and durable state in `effects.writes` or `effects.adapter_state`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Composer tool truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: uv` currently requires `requirements.toolchains: [python]`, `effects.network: true`, `effects.network_kind: dependency_hydration`, and durable state in `effects.writes` or `effects.adapter_state`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Python toolchain truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: poetry` currently requires `requirements.toolchains: [python]`, `effects.network: true`, `effects.network_kind: dependency_hydration`, and durable state in `effects.writes` or `effects.adapter_state`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Python toolchain truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: go_modules` currently requires `requirements.toolchains: [go]`, `effects.network: true`, and `effects.network_kind: dependency_hydration`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Go toolchain truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: helm` currently requires `requirements.tools.helm`, `effects.network: true`, `effects.network_kind: dependency_hydration`, and durable state in `effects.writes` or `effects.adapter_state`
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: maven` currently requires `requirements.toolchains: [java]`, `effects.network: true`, and `effects.network_kind: dependency_hydration`; when `prepare.source.wrapper` is absent or false, it also requires `requirements.tools.maven`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Java or Maven tool truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: gradle` currently requires `requirements.toolchains: [java]`, `effects.network: true`, and `effects.network_kind: dependency_hydration`; when `prepare.source.wrapper` is absent or false, it also requires `requirements.tools.gradle`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Java or Gradle tool truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: cargo` currently requires `requirements.toolchains: [rust]`, `effects.network: true`, and `effects.network_kind: dependency_hydration`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host Rust toolchain truth for the in-container command
- `prepare.kind: dependency_hydration` with `medium: package_dependencies` and `source.kind: dotnet_restore` currently requires `requirements.toolchains: [dotnet]`, `effects.network: true`, and `effects.network_kind: dependency_hydration`; when the lane is wrapped through `prepare.source.compose`, keep the host wrapper truthful with `requirements.tools.docker` or `requirements.tools.podman` and do not duplicate host dotnet toolchain truth for the in-container command
  - for an ephemeral container context that runs `dotnet restore` followed by `dotnet build --no-restore` or `dotnet test --no-restore`, declare `attachments.isolated_paths: [.nuget/packages]`; ota mounts that path as one engine-owned volume and derives `NUGET_PACKAGES=/workspace/.nuget/packages` for every task in the context so restored packages remain available without polluting the repo worktree
- `prepare.source.manager: pnpm` currently uses `mode: install`; use `frozen_lockfile: true` when the repo truth is strict `pnpm install --frozen-lockfile`
- `prepare.source.manager: npm` currently supports `mode: install` or `mode: ci`; use `mode: ci` when the repo truth is lockfile-strict npm hydration
- `prepare.source.manager: npm` may also declare `force: true` when the repo truth is explicitly `npm install --force` or `npm ci --force`; keep that override deliberate because it weakens normal npm safety semantics
- `prepare.source.manager: yarn` currently uses `mode: install`; use `frozen_lockfile: true` when the repo truth is strict `yarn install --immutable`
- `prepare.source.manager: yarn` may also declare `inline_builds: true` when the repo truth is `yarn install --inline-builds`
- `prepare.source.manager: bun` currently uses `mode: install`; use `frozen_lockfile: true` when the repo truth is strict `bun install --frozen-lockfile`
- `prepare.source.compose` wraps the typed hydration lane through `docker|podman compose exec/run`; keep package-manager truth under `prepare.source.kind` and service/container truth under `prepare.source.compose`
- see [`examples/reference/task-prepare-compose-hydration/ota.yaml`](../../examples/reference/task-prepare-compose-hydration/ota.yaml) for the canonical compose-wrapped Node package hydration shape
- `prepare.source.kind: bundler` currently executes one of two narrow canonical hydration lanes:
  - host-side or explicit repo-local path truth: `bundle config set path <path> && bundle install`
  - compose-wrapped container-default path truth: `bundle install`
- `prepare.source.kind: composer` currently executes the narrow canonical Composer hydration lane: `composer install`
- `prepare.source.kind: uv` currently executes one of three narrow canonical uv hydration lanes:
  - `uv sync`
  - `uv pip install -r <requirements_file>`
  - `uv pip install [-e] <local_project>[extras...]` followed by ordered `uv pip install --group <local_project>/pyproject.toml:<group>` for each declared group
  - optional `--default-index <url>`, ordered repeated `--index <url>`, and `--offline` are projected from the declared source posture; leave index selection undeclared only when the repo intentionally accepts ambient uv/pip configuration, in which case Ota records resolved provenance as unavailable rather than guessing from the host
  - for the local-project lane, Ota captures the declared local project's `pyproject.toml` identity and any declared `lockfile` identity before execution so dry-run, doctor, and replay-grade preflight can refuse a missing or changed contract-owned baseline early
- `prepare.source.kind: poetry` currently executes the narrow canonical Poetry hydration lane: `poetry install`, with optional `--with` or `--only` group selection and optional `--no-root`
- `prepare.source.kind: go_modules` currently executes the narrow canonical Go module hydration lane: `go mod download`
- `prepare.source.kind: helm` currently executes the narrow canonical Helm chart hydration lane: ota reads `Chart.yaml`, seeds any declared HTTP(S) chart repositories into isolated repo-owned Helm repository state under `.ota/state/helm/...`, then runs `helm dependency build .`
- `prepare.source.kind: maven` currently executes the narrow canonical Maven hydration lane: `./mvnw -q dependency:resolve` or `./mvnw -q dependency:go-offline` when wrapper-owned, otherwise `mvn -q dependency:resolve` or `mvn -q dependency:go-offline`
- `prepare.source.kind: gradle` currently executes the narrow canonical Gradle hydration lane: `./gradlew dependencies` when wrapper-owned, otherwise `gradle dependencies`
- `prepare.source.kind: cargo` currently executes the narrow canonical Cargo hydration lane: `cargo fetch`
- `prepare.source.kind: dotnet_restore` currently executes the narrow canonical .NET hydration lane:
  - `dotnet restore`
  - optional `--configfile <path>` when `prepare.source.config_file` is declared
  - optional repeated `--source <url>` when `prepare.source.sources[]` is declared
  - keep repository-owned feed selection in `config_file` when that is the real NuGet truth;
    `sources[]` is a command override, not a second copy of the config file
  - on the first canonical machine carrier, `ota up --json`, Ota records the selected lane as a
    typed `receipt.evaluated_inputs[]` `hydration_provenance` record. Its nested declared posture
    stays separate from runner-resolved feed identities; it publishes
    `hydration_provenance.resolved.source_identities[]` for active config-backed feeds and
    `resolution: unavailable` plus `resolution_error` instead of claiming a config was resolved
    when it could not read, parse, or resolve it without ambient environment substitution; an
    undeclared `ambient_default` source posture is also `unavailable` because user/global NuGet
    configuration is not contract-owned source truth
- `prepare.kind: tool_bootstrap` currently executes two narrow canonical tool-bootstrap lanes:
  - Python lane: `<exe> -m pip install --disable-pip-version-check -q uv`
  - Poetry lane for `prepare.tool: playwright_browsers`:
    - `poetry run playwright install [browsers...]`
    - `poetry run playwright install --with-deps [browsers...]`
  - Node lane for `prepare.tool: playwright_browsers`:
    - `npx playwright install [browsers...]`
    - `npx playwright install --with-deps [browsers...]`
    - `pnpm exec playwright install [browsers...]`
    - `pnpm exec playwright install --with-deps [browsers...]`
    - `pnpm --filter <selector> exec playwright install [browsers...]`
    - `pnpm --filter <selector> exec playwright install --with-deps [browsers...]`
    - `yarn playwright install [browsers...]`
    - `yarn playwright install --with-deps [browsers...]`
    - `bunx playwright install [browsers...]`
    - `bunx playwright install --with-deps [browsers...]`
  - Node lane for `prepare.tool: cypress_browsers`:
    - `npx cypress install`
    - `pnpm cypress install`
    - `yarn cypress install`
    - `bunx cypress install`
- `prepare` does not replace workflow-owned host bootstrap; workflow prepare is still the explicit host bootstrap lane and now points at one native finite owner (`prepare.task` or `prepare.action`)
- `execution.orchestrator.mode: exec` can mediate command-backed `prepare.kind: dependency_hydration` and `prepare.kind: tool_bootstrap`
- `execution.orchestrator.mode: subcommand` is the direct orchestrator CLI lane for structured
  task `command` bodies and `launch.kind: command`; use it for truth like `devenv test` or
  `devenv up` rather than forcing those lanes into `task` or `exec`
- mixed/native `prepare.kind: sequence` stays outside orchestrator mediation in the current shipped slice

Example:

```yaml
tasks:
  setup:
    description: Materialize local env and hydrate frontend and backend dependencies
    prepare:
      kind: sequence
      steps:
        - kind: ensure_env_file
          path: .env.local
          vars:
            APP_ENV:
              value: local
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
            frozen_lockfile: true
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: .
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
```

Command-backed prepare mediation example:

```yaml
orchestrators:
  devbox:
    kind: devbox
    required: true
    config_files:
      - devbox.json
    prepare:
      install: true

toolchains:
  node:
    version: "*"
    package_managers:
      pnpm: "*"

tasks:
  setup:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: node_package_manager
        cwd: .
        manager: pnpm
        mode: install
    requirements:
      toolchains:
        - node
    effects:
      writes:
        - node_modules
      network: true
      network_kind: dependency_hydration
    execution:
      orchestrator:
        ref: devbox
        mode: exec
```

`execution` fields:

- `default_mode`: optional `native`, `container`, or `remote`
- `modes`: optional backend map
- `modes.<mode>.context`: optional context override for that mode
- `modes.<mode>.depends_on`: optional task dependency override for that mode
- `modes.<mode>.lifecycle`: optional lifecycle override for that mode (container mode only)
- `runtime_boundary.filesystem.repo_root_mode`: optional repo-root mount posture for the selected
  lane (`read_only` or `writable`)
- `runtime_boundary.filesystem.writable_paths`: optional ordered repo-relative writable carve-outs
  for the selected lane
- `runtime_boundary.filesystem.protected_paths`: optional ordered repo-relative protected carve-outs
  for the selected lane
- `runtime_boundary.network.default`: optional default outbound posture for the selected lane
  (`deny` or `allow`)
- `runtime_boundary.network.outbound_targets`: optional ordered explicit outbound target truth for
  the selected lane
  - `kind`: required `host`, `domain`, or `service_alias`
  - `value`: required literal target value
  - `destination_shape`: optional `single_purpose_host`, `multi_tenant_host`, `relay_host`, or
    `send_host`
  - `destination_constraint`: optional narrower effective-destination truth for lanes where
    first-hop host allowlisting is not sufficient
    - `kind`: required `callback_host_allowlist`, `recipient_domain_allowlist`,
      `downstream_host_allowlist`, `bucket_scope`, `tenant_scope`, or
      `sms_destination_allowlist`
    - `values`: required ordered constrained destination values
    - `source_posture`: required `repo_local_authoritative`, `shared_pinned_authoritative`, or
      `non_authoritative`
    - `enforcement`: required `authoritative_runtime_enforced`,
      `authoritative_app_enforced`, or `advisory_only`
    - `shared_pin`: required when `source_posture: shared_pinned_authoritative`
      - `ref`: required pinned shared truth identifier
      - `freshness`: required `fresh`, `warning`, or `blocking`
- `env_files`: optional ordered repo-relative dotenv overlays injected into the task process before task-level `env`
- `adapter_inputs.overlays.compose.cwd`: canonical optional repo-relative adapter working directory ota should enter before executing the selected `docker compose` or `podman compose` task path; declared compose env files and compose files stay repo-relative in the contract and are projected relative to this adapter root at runtime
- `adapter_inputs.overlays.compose.env_files`: canonical optional ordered repo-relative compose interpolation files projected to the selected task mode through `COMPOSE_ENV_FILES`; use this for task-owned `docker compose` or `podman compose` adapter input truth rather than process dotenv injection
- `adapter_inputs.overlays.compose.files`: canonical optional ordered repo-relative compose file list projected to the selected task mode through `COMPOSE_FILE`
- `adapter_inputs.overlays.compose.profiles`: canonical optional ordered compose profile list projected to the selected task mode through `COMPOSE_PROFILES`
- `adapter_inputs.overlays.compose.project_name`: canonical optional compose project name projected to the selected task mode through `COMPOSE_PROJECT_NAME`
- `adapter_inputs.overlays.bake.cwd`: canonical optional repo-relative adapter working directory ota should enter before executing the selected `docker buildx bake` task path; declared Bake files stay repo-relative in the contract and are projected relative to this adapter root at runtime
- `adapter_inputs.overlays.bake.files`: canonical optional ordered repo-relative Bake file list projected to the selected task mode through `BUILDX_BAKE_FILE`
- `adapter_inputs.overlays.helm.cwd`: canonical optional repo-relative Helm adapter working directory ota should enter before executing the selected Helm task path; declared chart and values-file paths stay repo-relative in the contract and are projected relative to this adapter root at runtime
- `adapter_inputs.overlays.helm.values_files`: canonical optional ordered repo-relative Helm values-file list ota projects into the selected Helm task path as repeated `-f` / `--values` arguments
- `adapter_inputs.overlays.helm.chart`: canonical optional repo-relative Helm chart path ota projects into the selected Helm task path instead of hard-coding chart selection in argv
- `adapter_inputs.overlays.helm.release_name`: canonical optional Helm release name ota projects into the selected Helm task path instead of hard-coding release naming in argv
- `adapter_inputs.overlays.helm.namespace`: canonical optional Helm namespace ota projects into the selected Helm task path instead of hard-coding `--namespace` in argv
- `adapter_inputs.compose.*` / `adapter_inputs.bake.*` / `adapter_inputs.helm.*`: compatibility aliases for existing contracts; prefer `adapter_inputs.overlays.<family>.*` for new authoring
- the public map shape is generalized, and shipped runtime semantics currently exist for overlay families `compose`, `bake`, and `helm`; other family keys are rejected by validation until ota ships their runtime and governance model
- `modes.<mode>.env`: optional env map merged over task-level `env`
- `modes.<mode>.env_files`: optional ordered repo-relative dotenv overlays appended after task-level `env_files`
- `modes.<mode>.adapter_inputs.overlays.compose.cwd`: optional compose adapter working-directory override for that mode
- `modes.<mode>.adapter_inputs.overlays.compose.env_files`: optional ordered repo-relative compose interpolation files appended after task-level `adapter_inputs.overlays.compose.env_files`
- `modes.<mode>.adapter_inputs.overlays.compose.files`: optional ordered repo-relative compose file list appended after task-level `adapter_inputs.overlays.compose.files`
- `modes.<mode>.adapter_inputs.overlays.compose.profiles`: optional ordered compose profile list appended after task-level `adapter_inputs.overlays.compose.profiles`
- `modes.<mode>.adapter_inputs.overlays.compose.project_name`: optional compose project name override for that mode
- `modes.<mode>.adapter_inputs.overlays.bake.cwd`: optional Bake adapter working-directory override for that mode
- `modes.<mode>.adapter_inputs.overlays.bake.files`: optional ordered repo-relative Bake file list appended after task-level `adapter_inputs.overlays.bake.files`
- `modes.<mode>.adapter_inputs.overlays.helm.cwd`: optional Helm adapter working-directory override for that mode
- `modes.<mode>.adapter_inputs.overlays.helm.values_files`: optional ordered repo-relative Helm values-file list appended after task-level `adapter_inputs.overlays.helm.values_files`
- `modes.<mode>.adapter_inputs.overlays.helm.chart`: optional repo-relative Helm chart override for that mode
- `modes.<mode>.adapter_inputs.overlays.helm.release_name`: optional Helm release-name override for that mode
- `modes.<mode>.adapter_inputs.overlays.helm.namespace`: optional Helm namespace override for that mode
- `modes.<mode>.run`: optional single-line command override for that mode
- `modes.<mode>.script`: optional multiline script override for that mode
- `modes.<mode>.command`: optional structured finite command override for that mode
- `modes.<mode>.prepare`: optional structured preparation override for that mode
- `modes.<mode>.launch`: optional structured launch override for that mode
- `modes.<mode>.runtime`: optional runtime/listener override for that mode

`when` fields:

- `checks`: optional list of check names that must pass before this task node executes

`when` rules:

- `when.checks` gates the selected task node only; dependency ordering and service requirements are still declared separately
- allowed condition-check kinds are `precondition` (with `run`), `file`, `env`, and `changed_files`
- probe-driven preconditions are not valid condition checks for `when.checks`
- condition checks run before dependency/service startup for the selected task node; if any condition fails, ota skips the task deterministically

`execution` mode rules:

- `--mode` changes execution plane, not task identity; one task name can carry multiple mode branches
- `default_mode` can stand alone when the task-level `run`/`script` already describes the default path
- when a branch is selected, branch values override task-level values for `context`, `depends_on`, `lifecycle`, `env`, `run`/`script`/`command`/`prepare`/`launch`, and `runtime`
- `execution.runtime_boundary` is the repo baseline for that task lane; task-local
  `runtime_boundary` can narrow or replace it for the selected task instead of leaving sandbox
  posture split across agent metadata and shell conventions
- runtime-boundary declaration and runtime enforcement remain separate truths. In agent execution,
  authoritative selected-lane controls must be applied by a compatible provider or Ota refuses
  before preparation. The first enforcing provider, `oci_local`, only applies to already-declared
  ephemeral container lanes with an explicit `container.platform`; provider selection cannot
  change task mode, lifecycle, command, or context
- `oci_local` requires writable carve-outs to exist before admission and rejects symlink, hardlink,
  writable/protected overlap, inherited-network, runtime-socket, and targeted-egress shapes it
  cannot prove safely. These are provider capability boundaries, not reasons to weaken the
  contract declaration
- its first implementation admits finite `run`, `script`, and `command` task bodies only. Typed
  prepare/action/Compose/launch/attach bodies, task requirements, required services, and
  conditional checks remain explicit unsupported pre-boundary work and cause refusal
- `modes.<mode>.depends_on` replaces the task-level dependency list for that mode; omit it when the task-level `depends_on` already matches the selected execution plane
- use `modes.<mode>.depends_on` when one task keeps the same identity but needs different preflight on host vs container instead of cloning tasks like `build:host`
- use `env_files` for task-process dotenv overlays; use `adapter_inputs.overlays.compose.*` when one task path owns `docker compose` adapter root, interpolation input, compose file selection, compose profiles, or project naming and that ownership should stay declarative instead of being hard-coded into the shell body
- use `adapter_inputs.overlays.bake.*` when one task path owns `docker buildx bake` adapter root or file selection and that truth should project through a first-class adapter surface instead of shell `cd ... &&` / `-f`
- use `adapter_inputs.overlays.helm.*` when one task path owns Helm chart root, values-file selection, release naming, or namespace truth and that truth should project through a first-class adapter surface instead of shell `cd ... && helm ...`, chart positionals, or `--namespace`
- keep `adapter_inputs.overlays.<family>` non-empty: empty family markers are a governance smell and validate/doctor warn on them; either declare concrete adapter-owned fields or omit the family entirely
- do not infer that arbitrary families are executable just because the map is generic; today ota only executes overlay semantics for `compose`, `bake`, and `helm`, and validator rejects unsupported families
- validate/doctor warn when Compose adapter root or file/profile/project truth stays hard-coded in shell `docker compose --project-directory ...`, `cd ... && docker compose ...`, `--env-file`, `-f`, `--file`, `--profile`, `-p`, or `--project-name` instead of `adapter_inputs.overlays.compose.*`
- validate/doctor warn when Bake file truth stays hard-coded in shell `docker buildx bake -f` / `--file` flags, or when Bake adapter root stays hard-coded in shell `cd ... && docker buildx bake ...`, instead of `adapter_inputs.overlays.bake.*`
- validate/doctor warn when Helm chart, values-file, namespace, or adapter-root truth stays hard-coded in shell `cd ... && helm ...`, Helm `-f` / `--values`, or Helm `-n` / `--namespace` flags, instead of `adapter_inputs.overlays.helm.*`
- when a selected branch omits `run`/`script`/`command`/`prepare`/`launch`, ota falls back to the task-level execution body (including OS variants)
- when a task declares `execution.modes`, an explicit `--mode` must resolve to a declared branch
  unless it matches `default_mode`; unsupported explicit overrides fail early with a mode-branch error
- use `modes.<mode>` only for mode-specific overrides; you do not need an empty branch such as `modes.native: {}` just to pair with `default_mode: native`
- `modes.native.lifecycle` and `modes.remote.lifecycle` are invalid; lifecycle is only valid for container execution

`command` fields:

- `command.exe`: required executable name or path; this is generic process truth rather than an npm-specific field, and may be `npm`, `pnpm`, `yarn`, `bun`, `node`, `python3`, `go`, `bundle`, `docker`, an absolute path, or a repo-local executable path
- `command.args`: optional argument list
- `command.cwd`: optional repo-relative working directory for that structured finite command; use it
  when the task truth is one executable plus stable argv rooted in a repo subdirectory instead of
  hiding `cd ... && ...` shell glue in `run` or `script`
- `command.interaction`: optional terminal interaction posture for the child process; controls
  whether the command may inherit a real interactive TTY from the host terminal; omitting this
  field is equivalent to `auto`
  - `auto` (default): when Ota itself is attached to a real TTY and the execution backend is
    native, stdin/stdout/stderr are inherited so the child process detects an interactive
    terminal; in agent mode and ordinary non-TTY CI or other non-TTY contexts the process uses
    non-interactive captured execution
  - `forbidden`: always use non-interactive execution with stdin closed regardless of context; the
    child process never sees a TTY
  - `required`: like `auto` in a human TTY context, but refuses execution with a preflight error
    when no interactive terminal can be provided (ordinary CI, agent mode, non-TTY); use this when the
    task cannot make progress without interactive prompts (for example, Wrangler OAuth login).
    `required` takes precedence over `--stream`: Ota passes the terminal through and does not
    claim pipe-captured command output for that invocation.

Ota does not maintain an allowlist for `command.exe`. The examples above are illustrative, not exhaustive; the executable may be any repo-truthful binary or path as long as the contract declares its requirements honestly.

`launch` fields:

- `launch.kind`: required `command`, `compose`, or `container`
- `launch.kind: command`
  - `launch.exe`: required executable name or path
  - `launch.args`: optional argument list
  - `launch.cwd`: optional repo-relative working directory for that structured long-running process
    start; use it when the service launch truth is subdirectory-rooted but still one executable
    plus stable argv
  - `launch.runtime_projection.listener`: optional explicit `runtime.listeners.<name>` selector
    for supported server adapters whose bind argv should be projected from canonical runtime
    listener truth instead of duplicated in `launch.args`
  - `launch.runtime_projection.adapter`: required when `launch.runtime_projection.listener` is
    set; currently `uvicorn`, `rails`, and `nextjs`. Use `nextjs` with a direct structured
    `next dev` invocation (for example `pnpm exec next dev --turbopack`), not a package-script
    wrapper that already owns host or port flags.
- `launch.kind: compose`
  - `launch.engine`: optional compose CLI engine; `docker` by default, `podman` also supported
  - `launch.action`: required compose launch action; currently `up`
  - `launch.services`: optional ordered service list for scoped `compose up`
  - `launch.detach`: optional detached startup flag for `compose up -d`
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

`action` fields:

- `action.kind`: required action kind; currently `copy_if_missing`, `ensure_env_file`, or
  `ensure_file`, `ensure_directory`, `ensure_virtualenv`, `ensure_git_checkout`, `ensure_git_template`, `ensure_git_checkouts`, `ensure_container_network`, `build_container_image`,
  `reset_compose_service_volume`, or `ensure_bundle`
- `action` is the first-class surface for deterministic preparation and explicitly governed local
  materialization. File-preparation actions are native-only because they mutate the host working
  tree directly; `build_container_image` is a direct Docker-owned task action and materializes
  external container state instead
  and Ota does not yet claim one cross-backend persistence/ownership model for container or remote
  mutations
- `action.kind: copy_if_missing`
  - `action.from`: required repo-relative source file
  - `action.to`: required repo-relative destination file
- `action.kind: ensure_env_file`
  - `action.path`: required repo-relative env-file path to create/update
  - `action.template`: optional repo-relative seed file copied when `action.path` is missing
  - `action.template_mode`: optional `missing` or `replace` (default `missing`)
  - `action.vars`: required key map for env keys ota should enforce
  - `action.vars.<KEY>.value`: literal value for the declared key
  - `action.vars.<KEY>.random`: generated value for the declared key
  - `action.vars.<KEY>.from_env`: copy the effective resolved value of another declared env name
    into this key
  - `action.vars.<KEY>.mode`: optional `missing`, `replace`, or `remove` (default `missing`)
  - `action.vars.<KEY>.random.bytes`: optional byte length (default `32`)
  - `action.vars.<KEY>.random.encoding`: optional `hex` or `base64` (default `hex`)
- `action.kind: ensure_file`
  - `action.path`: required repo-relative file path to create when missing
  - exactly one bootstrap source is required:
    - `action.template`: copy this repo-relative template file
    - `action.value`: write this literal file content
    - `action.random`: generate random file content
  - `action.random.bytes`: optional byte length (default `32`)
  - `action.random.encoding`: optional `hex` or `base64` (default `hex`)
- `action.kind: ensure_directory`
  - `action.path`: required repo-relative directory path to create when missing
- `action.kind: ensure_virtualenv`
  - `action.path`: required repo-relative virtualenv path to create when missing
  - `action.provider`: optional virtualenv provider; currently `uv` (default `uv`)
  - `action.python`: optional Python selector. An explicit path is forwarded unchanged. For a
    version selector such as `"3.12"`, native execution prefers a version-compatible local
    interpreter matching the host architecture, then passes that absolute path to `uv`; when no
    such local candidate exists, Ota forwards the selector to the provider unchanged.
- `action.kind: ensure_git_checkout`
  - `action.path`: required repo-relative checkout path to create when missing
  - `action.source.git`: required Git remote URL or clone source
  - `action.source.ref`: optional Git ref Ota should check out after clone
  - `action.remotes`: optional ordered list of declared Git remotes Ota should reconcile inside
    the materialized checkout
  - `action.remotes[].name`: required Git remote name such as `origin` or `upstream`
  - `action.remotes[].git`: required Git remote URL Ota should add or set for that name
- `action.kind: ensure_git_template`
  - `action.path`: required repo-relative scaffold path to create when missing
  - `action.source.git`: required Git remote URL or clone source
  - `action.source.ref`: optional Git ref Ota should check out before Ota strips inherited Git metadata
- `action.kind: ensure_git_checkouts`
  - `action.checkouts`: required ordered list of git checkouts Ota should materialize
  - each `action.checkouts[]` entry uses the same fields and validation rules as
    `action.kind: ensure_git_checkout`
- `action.kind: ensure_container_network`
  - `action.provider`: optional container runtime provider; currently `docker` (default `docker`)
  - `action.name`: required container network name to inspect/create
- `action.kind: build_container_image`
  - `action.provider`: optional container runtime provider; currently `docker` (default `docker`)
  - `action.file`: required repo-relative Dockerfile path
  - `action.context`: required repo-relative Docker build context
  - `action.tag`: required local image tag; it must not be empty or contain whitespace
  - this is a direct task action, not an `ensure_bundle` or `prepare.steps` entry: building a
    local image is material execution with an explicit Docker side effect, not idempotent host
    bootstrap
- `action.kind: reset_compose_service_volume`
  - `action.provider`: optional compose runtime provider; currently `docker` (default `docker`)
  - `action.service`: required non-empty compose service name ota should stop, remove, and restart
  - `action.volume`: required non-empty container volume name ota should remove before restart
  - `action.compose.cwd`: optional repo-relative compose adapter working directory
  - `action.compose.env_files`: optional ordered compose interpolation files projected through `COMPOSE_ENV_FILES`
  - `action.compose.files`: optional ordered compose file list projected through `COMPOSE_FILE`
  - `action.compose.profiles`: optional ordered compose profile list projected through `COMPOSE_PROFILES`
  - `action.compose.project_name`: optional compose project name projected through `COMPOSE_PROJECT_NAME`
- `action.kind: ensure_bundle`
  - `action.steps`: required ordered list of deterministic bootstrap steps
  - each `action.steps[]` entry uses one of:
    - `kind: copy_if_missing`
    - `kind: ensure_env_file`
    - `kind: ensure_file`
    - `kind: ensure_directory`
    - `kind: ensure_virtualenv`
    - `kind: ensure_git_checkout`
    - `kind: ensure_git_template`
    - `kind: ensure_git_checkouts`
    - `kind: ensure_container_network`
    - `kind: reset_compose_service_volume`
  - each step uses the same fields and validation rules as the corresponding top-level action kind

Use `action.kind: copy_if_missing` for setup steps like creating `.env.local` from
`.env.example` without depending on POSIX `test` / `cp` or PowerShell conditionals. The action is
idempotent: if `to` already exists, Ota leaves it untouched.

Use `action` when the task is deterministic host repo preparation such as copying templates,
seeding env files, creating bootstrap files, or creating directories. Do not use `action` for
arbitrary command execution or backend-local mutation paths; use `run`, `script`, `command`, or
`launch` there instead.

Use `action.kind: ensure_env_file` when a setup path needs deterministic env bootstrap without a
shell script. It creates `action.path` (optionally seeded from `action.template`) and appends only
missing keys by default. Use `action.vars.<KEY>.mode: replace` when a workflow-scoped overlay must
rewrite specific keys deterministically. Use `action.template_mode: replace` when the destination
should be re-derived from `action.template` on every run before applying the declared key updates;
this is the governed replacement for shell copy-plus-`sed` env normalization. Use
`action.vars.<KEY>.from_env` when one generated env file should project already-declared Ota env
truth into a workflow-specific overlay, and use `action.vars.<KEY>.mode: remove` when stale keys
must be deleted deterministically instead of relying on shell mutation.

When a dependent task declares the same path under `env_files`, Ota treats a preceding declared
`ensure_env_file` action in its closure as planned setup output during dry-run. This keeps previews
usable from a clean checkout while real execution still validates the rendered dotenv file after
its dependencies complete.

Use `action.kind: ensure_file` when setup needs one deterministic bootstrap file (for example a
secret token file) without shell glue. It creates `action.path` once from one explicit source
(`template`, `value`, or `random`) and leaves existing files untouched on repeat runs.

Use `action.kind: ensure_directory` when setup needs a deterministic repo-local directory without
shell glue. It creates `action.path` when missing, no-ops when it already exists as a directory,
and fails if the path already exists as a non-directory.

Use `action.kind: ensure_virtualenv` when setup truthfully owns creation of one repo-local Python
virtualenv such as `.venv` without shell glue. Keep dependency installation itself under
`prepare.kind: dependency_hydration` so Ota owns virtualenv materialization and package hydration
as separate first-class setup truths instead of collapsing both into one opaque shell lane.

Use `action.kind: ensure_git_checkout` when setup truthfully owns deterministic materialization of
one sibling checkout, vendored dependency repo, or other Git-backed working tree without shell
bootstrap glue. Ota clones `action.source.git` into `action.path` only when that path is missing,
optionally checks out `action.source.ref`, optionally reconciles declared `action.remotes[]` by
adding missing remotes or updating existing remote URLs, and then leaves existing directories
otherwise untouched on repeat runs. `action.path` may stay repo-internal (`vendor/wagtail`) or
point at a declared sibling / workspace-relative target (`../ui`) as long as it remains a
relative path without an absolute prefix. This is intentionally a materialization surface, not an
update/reset surface: use it when bootstrap needs “make sure this checkout exists and has the
declared remote wiring”, not when setup should implicitly pull, fetch, or rewrite repository
history that is already present. When `action.source.ref` is omitted, Ota intentionally tracks the
remote default branch head and `ota validate` / `ota doctor` warn that the checkout is moving-head
pressure truth rather than deterministic proof truth.

Use `action.kind: ensure_git_template` when setup truthfully owns deterministic factory
materialization from a Git-backed scaffold that should become a fresh local repository instead of
remaining a clone of the upstream template. Ota clones `action.source.git` into `action.path`
only when that path is missing, optionally checks out `action.source.ref`, removes inherited Git
metadata from the cloned scaffold, initializes a fresh Git repository in place, and then leaves
existing directories untouched on repeat runs. This is the governed replacement for shell flows
like `git clone ...`, `rm -rf .git`, and `git init` when a repo or starter skill documents
template-style bootstrap.

Use `action.kind: ensure_git_checkouts` when setup truthfully owns several deterministic sibling or
vendored Git checkouts and repeating `ensure_git_checkout` entries would flatten one cohesive
materialization lane into low-level boilerplate. Ota applies the declared checkouts in order, uses
the same idempotent semantics as `ensure_git_checkout` for each entry, and keeps moving-head
advisories per checkout when `source.ref` is omitted.

Use `action.kind: ensure_container_network` when setup needs one deterministic external container
network without shell glue. It inspects the named provider-owned network, creates it only when
missing, and keeps Docker network bootstrap on a first-class declarative surface instead of
hard-coding `docker network inspect/create` logic into `run`, `script`, or `command`.

Use `action.kind: build_container_image` when a task must materialize a Dockerfile-backed local
image for a declared container or Compose lane. Ota invokes `docker build --file <file> --tag <tag>
<context>` from the repository working directory and records the Dockerfile-to-tag action in task
discovery. Keep the build's network and Docker state effects explicit on the task; do not hide a
locally tagged image build in `run`, `script`, or `command`.

Use `action.kind: reset_compose_service_volume` when one setup or recovery lane truthfully owns a
destructive Compose-managed data reset such as “stop the service, remove its named volume, then
restart that service”. Keep compose adapter cwd, env files, file selection, profiles, and project
name on the structured `action.compose.*` fields instead of hiding them in shell
`docker compose ...` flags.

Use `action.kind: ensure_bundle` when setup needs multiple deterministic bootstrap mutations in one
task (for example env file seeding plus secret file creation plus cache directory creation, or one
shared Docker network plus host file prep) without shell orchestration. Ota executes steps in
order, preserves the same idempotent semantics as each step kind, and keeps validation/error
reporting inside the contract surface.

`requirements` fields:

- `requirements.runtimes`: optional runtime requirements that apply only to this task path
- `requirements.tools`: optional tool requirements that apply only to this task path
- `requirements.any_of`: optional context/backend-scoped alternative requirement branches for
  path-scoped requirements (`runtimes`, `tools`, `toolchains`, `native`, `env`, `checks`) useful
  for "A or B" prerequisite modeling without shell conditionals
- `requirements.native`: optional native prerequisite names from top-level `native_prerequisites`
- `requirements.env`: optional list of names from top-level `env.vars`
- `requirements.checks`: optional list of names from top-level `checks`

`requirements` rules:

- use top-level `runtimes`, `tools`, and required `env.vars` only when the prerequisite is truly
  repo-global
- use `tasks.<name>.requirements` when a prerequisite belongs only to one contributor, quickstart,
  or packaged-runtime path
- `command.exe` and `launch.kind: command` both implicitly scope their executable as a task-path
  tool requirement (default version `*`) so selected-workflow precondition diagnosis and
  activation surfaces include those executables even when `requirements.tools` is omitted
- declare `requirements.tools.<name>` explicitly when you need to pin a version, attach
  acquisition metadata, or override defaults for that executable
- `requirements.tools` is task-path truth and can be self-contained: tool names do not need a
  matching top-level `tools.<name>` declaration to validate
- if a required tool is owned by one or more declared toolchains, keep ownership deterministic by
  declaring `requirements.toolchains` explicitly on that task path
- `requirements.any_of[]` entries must declare at least one requirement
  (`runtimes`, `tools`, `toolchains`, `native`, `env`, or `checks`) plus a deterministic selector
  (`when.context` or `when.backend`) so Ota can resolve one branch on the selected path
- use `requirements.any_of` for disjunctive prerequisite paths such as "host-local service lane"
  vs "docker-host lane" while keeping each branch explicit and reviewable in contract truth
- use `requirements.native` when a task needs host-native build tools but Ota should diagnose and
  guide instead of silently installing OS packages
- workflow-aware readiness commands evaluate the selected workflow's `prepare.task` / `setup.task` / `run.task`
  dependency closure and merge those task-scoped requirements before diagnosing preconditions
- an explicitly selected workflow with no `setup.task` has no setup prerequisite phase; legacy
  `tasks.setup` fallback is reserved for the unselected default compatibility path
- `requirements.checks` must reference top-level checks declared with `kind: precondition`,
  `kind: file`, `kind: env`, or `kind: changed_files`
- when the selected workflow task closure declares any task-scoped requirements, unreferenced
  top-level precondition checks are treated as reusable definitions rather than global gates for
  that path; reference a check from `requirements.checks` when that task path needs it
- `requirements.env` must reference declared top-level `env.vars` names; it does not create new env
  requirements inline
- selected task/workflow evaluation treats `requirements.env` as path-scoped required env truth:
  the referenced top-level names become required for `ota doctor`, `ota env --task`, `ota up`, and
  task execution on that selected path even when `env.vars.<name>.required` is not repo-global
  `true`

`launch` rules:

- each task must declare exactly one executable source:
  - `run`
  - `script`
  - `command`
  - `launch`
  - `action`
- `run` stays the simple shell shorthand
- `script` stays the multiline shell escape hatch
- `command` is for finite argv-owned execution that Ota should model structurally without shell
  parsing
- `launch` is for structured, inspectable long-running starts that Ota should render and reason
  about without hiding everything inside one shell string
- `action` is for small built-in setup mutations that should stay deterministic and
  cross-platform instead of becoming shell-specific snippets
- prefer `command` for finite task bodies such as `uv run pytest`, `poetry run pytest`,
  `bundle exec rake test`, or `npm run build` when the repo truth is one executable plus a stable
  argument vector rather than shell composition; if that executable should run from a repo
  subdirectory, prefer `command.cwd` over fake `cd ... && ...` shell glue
- for long-running service processes, prefer `launch.kind: command` over opaque shell `run` or
  `script`; if the service start is rooted in a repo subdirectory, prefer `launch.cwd` over fake
  shell `cd ... && ...`
- when a supported long-running adapter would otherwise duplicate bind flags already declared
  under `runtime.listeners`, prefer `launch.runtime_projection` so Ota projects bind argv from
  canonical listener truth instead of carrying `--host` / `--port` or `-b` / `-p` twice
- for long-running Compose stack startup, prefer `launch.kind: compose` over raw shell or
  `launch.kind: command` carrying `docker compose up ...` argv
- pair `launch.kind: command` with `runtime.kind: service` plus `runtime.surfaces` or
  `runtime.listeners`; `launch` starts the process, while `runtime` declares what becomes reachable
  and how readiness is proved
- `launch.runtime_projection` currently binds only to explicit `runtime.listeners.<name>` entries;
  keep the listener declared explicitly when the launch adapter should project bind argv from
  runtime-owned listener truth
- `launch.runtime_projection` requires a fixed `runtime.listeners.<name>.bind.address` plus fixed
  `bind.port.value`; Ota rejects conflicting manual bind flags in `launch.args` for the supported
  adapter
- pair `launch.kind: compose` with `runtime.kind: service`; `launch` owns the persistent
  `compose up` start, while `runtime` still declares what becomes reachable and how readiness is
  proved
- `ota validate` and `ota doctor` warn when a task path resolves to shell `run` or `script`
  together with `runtime.kind: service`; migrate that path to `launch.kind: command` or
  `launch.kind: compose` unless the task is truly a shell-oriented finite escape hatch
- reserve `run` for finite shell tasks, pipelines, or real escape-hatch cases where a structured
  executable shape would be misleading or unavailable
- reserve `script` for multiline finite shell escape hatches, not long-running service ownership
- `command` reuses existing task env, input, receipt, dependency, and agent-safety behavior
- `launch.kind: command` reuses existing task env, input, receipt, dependency, and agent-safety
  behavior
- `launch.kind: compose` keeps host-side Compose cwd, env-file, file-stack, profile, and
  project-name truth under `adapter_inputs.overlays.compose.*`, and keeps only persistent
  `compose up` launch truth under `launch`
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
  test:
    command:
      exe: uv
      args: [run, pytest]

  dev:
    launch:
      kind: command
      exe: bundle
      args: [exec, rails, server]
      runtime_projection:
        listener: api
        adapter: rails
    runtime:
      kind: service
      listeners:
        api:
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
                mode: fixed
                value: 3000
              path: /
              primary: true
      surfaces: [api]

  quickstart:
    launch:
      kind: command
      exe: npx
      args: [n8n]
    runtime:
      kind: service
      surfaces:
        - backend

  selfhost:
    adapter_inputs:
      overlays:
        compose:
          env_files:
            - .env.selfhost
    launch:
      kind: compose
      engine: docker
      action: up
      detach: true
      services: [langfuse, langfuse-worker]
    runtime:
      kind: service
      surfaces:
        - web

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
- `listeners.<name>.project.publication.compose.service`: optional explicit service owner for a native `docker compose up` publication when ota should remap that host-visible port through `--host-port`

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
- `runtime.surfaces.<name>` still refers to the declared top-level reusable `surfaces.<name>`;
  object form means "attach that surface with publication overrides for this runtime", not
  "redefine the surface"
- `runtime.surfaces.<name>` attachment overrides are publication-only:
  - `bind`
  - `project`
  - `project.host.primary`
- `bind` is where the workload listens inside its execution context
- `project.host` is the host-facing projected endpoint ota reports, checks, and exposes
- each attached surface normalizes into the same runtime listener model used by explicit
  `runtime.listeners`
- when an explicit `runtime.readiness.listener` has a host projection but is not attached through
  `runtime.surfaces`, `ota validate` and `ota doctor` emit an advisory. Promote that endpoint to
  a named top-level surface when it is an operator-facing URL; keep raw listeners only for
  runtime-private endpoints that should not become reusable contract surface
- topology JSON now also exposes additive `runtime.surface_attachments.<name>` intent alongside the
  normalized listener truth
- attached surface names become normalized listener names
- a runtime must not attach an unknown surface
- a runtime may intentionally declare `runtime.listeners.<name>` and also attach
  `runtime.surfaces.<name>` when one published surface maps 1:1 to one explicit listener; this is
  the canonical same-name publication shape for workflow exposes and surface-backed runtime proof
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
- `ota run <task> --host-port <port>` can select one run's host-facing port on a primary listener with fixed bind and fixed `project.host.port` truth; containers and native Compose retain the internal bind, while direct native execution applies the selected port to bind and projection together
- direct native host-port overrides reproject supported typed launch arguments and Ota's canonical bind/public runtime env; the explicit execution option takes precedence over conflicting task-level bind env for that invocation
- for native structured `docker compose up` lanes, `--host-port` also requires explicit publication ownership through `project.publication.compose.service`; ota uses that declared compose service to render a temporary override stack instead of guessing service publication truth
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
- `--host-port` rejects invalid shapes before task spawn: listeners with `project.host.port.mode: auto`, no projected host listeners, ambiguous multi-listener projection without one primary listener, or native compose publications that omit `project.publication.compose.service`
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
  - practical rule of thumb:
    - use `manual` when target resolution is enough and the producer must already exist
    - use `ensure_started` when ota should start the producer and hand off immediately
    - use `restart_ready` when ota should bounce a reachable producer and verify it again
    - use `ensure_running` when listener reachability is enough
    - use `ensure_ready` when the producer declares a deeper readiness contract that should be satisfied before the consumer runs
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

- startup semantics:
  - for `ota run` service tasks, the declared readiness budget is authoritative during startup
  - ota now fails startup when the configured `start_period` / `interval` / `timeout` /
    `retries` budget is exhausted before the declared runtime endpoint becomes reachable
  - this is current runtime behavior, not a separate contract field such as
    `readiness.enforcement`
  - `ota run --stream` shows live readiness attempt progress while ota is still waiting
  - all run modes keep the final readiness-budget summary on startup failure, including attempts,
    timeout per attempt, interval, start period, and the last probe failure
  - once readiness is observed, the service is considered started and normal long-running task
    behavior resumes
- `probe: <name>`
  - references one top-level `readiness.probes.<name>` declaration
  - reuses that probe's transport and timeout contract while the selected listener still determines the runtime endpoint
  - may optionally declare `listener` when the readiness target should bind to one non-default runtime listener explicitly
  - may declare `signal_probes: [<name>, ...]` to require additional named task-listener probes
    before Ota reports the runtime as ready; each signal probe must target the same task runtime
    (`target.kind: task`, `target.name: <task>`) and may use:
    - `target.address_view: host` for projected host listener probes
    - `target.address_view: internal` for fixed same-task listener probes on native runtime paths
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
          launch:
            kind: command
            exe: mvn
            args: [spring-boot:run]
        container:
          context: app
          lifecycle: persistent
          env:
            DB_URL: jdbc:postgresql://postgres:5432/app
          launch:
            kind: command
            exe: mvn
            args:
              - spring-boot:run
              - -Dspring-boot.run.arguments=--server.address=0.0.0.0,--server.port=8080
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
- optional one execution override: `run`, `script`, `command`, or `compose`
- optional `env` / `env_files` / `env_bindings` / `inputs` / `requirements` when the task keeps the same body but needs OS-scoped process or requirement truth
- optional `adapter_inputs` when the task keeps the same body but needs OS-scoped Compose/Bake/Helm overlay truth

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
- tasks must declare exactly one task body: `run`, `script`, `command`, `compose`, `prepare`, `launch`, `action`, or `aggregate`, unless the task intentionally resolves through variants or execution-mode inheritance
- input names must use lowercase snake_case
- input defaults must not be empty
- input allowed values must not be empty
- `run` must be non-empty when present
- `script` must be non-empty when present
- variant entries must declare `when.os`
- variant entries may declare one execution override (`run`, `script`, `command`, or `compose`) and/or non-empty `env`, `env_files`, `env_bindings`, `inputs`, `requirements`, or `adapter_inputs`
- variant entries must not declare more than one execution override
- duplicate variants for the same `when.os` are rejected
- dependency references must resolve to known tasks
- aggregate member references must resolve to known tasks
- hook references must resolve to known tasks
- task dependency cycles are rejected
- aggregate membership and hook edges participate in the same task cycle detection as `depends_on`
- `depends_on` is the canonical prerequisite edge between executable tasks; `aggregate` is the canonical body for named dependency-closure entrypoints
- `requires_services` references must resolve to known services
- each required service must declare an actionable manager or readiness surface so ota can enforce the requirement

Current execution model:

- `run` and `script` are shell-compatible execution forms
- `command` is the structured finite argv execution form
- `aggregate` executes its declared `aggregate.tasks` in order and records the parent aggregate task as the requested entrypoint
- task `env` values are applied when ota runs the task and override repo-level env with the same name
- selected variant `env`, `env_files`, and `env_bindings` overlay onto the task-level process input truth before backend-mode-specific overrides apply
- when variants are declared, ota resolves the best matching `when.os` entry first and falls back to the default execution
- if the selected variant only declares task-input overlays (`env`, `env_files`, `env_bindings`, `inputs`, `requirements`, or `adapter_inputs`), ota keeps the base task body and applies the selected OS-scoped overlays on top of task-level truth
- `depends_on` runs before the task body
- dependency-plane selection is explicit: when the parent task resolves to a backend and a dependency truthfully supports that same backend through `execution.default_mode` or `execution.modes.<mode>`, ota keeps the dependency on that plane for the invocation; otherwise the dependency falls back to its own canonical backend selection
- `aggregate` replaces fake no-op wrappers such as `run: "true"` for verification entrypoints like `verify`
- `after_success` runs only when the task body exits `0`
- `after_failure` runs only when the task body exits non-zero
- `after_always` runs after either branch, but only when the task body actually ran
- hook tasks run in declared order
- tasks marked `internal: true` (commonly `setup`) remain normal graph nodes for `depends_on` and hooks, still run when referenced directly, and are hidden from default `ota tasks` output unless `--all` is requested
- hook failures affect the final task result for the parent task
- richer non-shell execution remains intentionally narrow; use `launch` for packaged starts and
  `action` for first-class setup mutations Ota can make cross-platform
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
    prepare:
      task: setup:env:local
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
- `<name>.notes`: optional multiline notes shown during `ota workflows` and `ota tasks --workflow` summaries
- `<name>.runtime_boundary`: optional canonical runtime sandbox baseline for the selected workflow
  path
  - `filesystem.repo_root_mode`: optional repo-root mount posture (`read_only` or `writable`)
  - `filesystem.writable_paths`: optional ordered repo-relative writable carve-outs
  - `filesystem.protected_paths`: optional ordered repo-relative protected carve-outs
  - `network.default`: optional default outbound posture (`deny` or `allow`)
  - `network.outbound_targets`: optional ordered explicit outbound target truth
    - `kind`: required `host`, `domain`, or `service_alias`
    - `value`: required literal target value
    - `destination_shape`: optional `single_purpose_host`, `multi_tenant_host`, `relay_host`, or
      `send_host`
    - `destination_constraint`: optional narrower effective-destination truth for lanes where
      first-hop host allowlisting is not sufficient
      - `kind`: required `callback_host_allowlist`, `recipient_domain_allowlist`,
        `downstream_host_allowlist`, `bucket_scope`, `tenant_scope`, or
        `sms_destination_allowlist`
      - `values`: required ordered constrained destination values
      - `source_posture`: required `repo_local_authoritative`,
        `shared_pinned_authoritative`, or `non_authoritative`
      - `enforcement`: required `authoritative_runtime_enforced`,
        `authoritative_app_enforced`, or `advisory_only`
      - `shared_pin`: required when `source_posture: shared_pinned_authoritative`
        - `ref`: required pinned shared truth identifier
        - `freshness`: required `fresh`, `warning`, or `blocking`
- `<name>.adapter_inputs.overlays.compose.cwd`: canonical optional repo-relative adapter working directory the
  workflow should project into selected compose task paths when that path does not already declare one
- `<name>.adapter_inputs.overlays.compose.env_files`: canonical optional ordered repo-relative adapter-owned
  compose interpolation files the workflow should project into selected compose task paths
- `<name>.adapter_inputs.overlays.compose.files`: canonical optional ordered repo-relative adapter-owned compose
  file overlays the workflow should project into selected compose task paths
- `<name>.adapter_inputs.overlays.compose.profiles`: canonical optional ordered adapter-owned compose profile list
  the workflow should project into selected compose task paths
- `<name>.adapter_inputs.overlays.compose.project_name`: canonical optional adapter-owned compose project name the
  workflow should project into selected compose task paths when that path does not already declare
  one
- `<name>.adapter_inputs.overlays.bake.cwd`: canonical optional repo-relative adapter working directory the
  workflow should project into selected `docker buildx bake` task paths when that path does not already declare one
- `<name>.adapter_inputs.overlays.bake.files`: canonical optional ordered repo-relative adapter-owned Bake file
  overlays the workflow should project into selected `docker buildx bake` task paths
- `<name>.adapter_inputs.overlays.helm.cwd`: canonical optional repo-relative adapter working directory the
  workflow should project into selected Helm task paths when that path does not already declare one
- `<name>.adapter_inputs.overlays.helm.values_files`: canonical optional ordered repo-relative adapter-owned Helm values-file overlays
  the workflow should project into selected Helm task paths
- `<name>.adapter_inputs.overlays.helm.chart`: canonical optional repo-relative Helm chart path the workflow should project into selected Helm task paths
- `<name>.adapter_inputs.overlays.helm.release_name`: canonical optional Helm release name the workflow should project into selected Helm task paths
- `<name>.adapter_inputs.overlays.helm.namespace`: canonical optional Helm namespace the workflow should project into selected Helm task paths
- `<name>.adapter_inputs.compose.*` / `<name>.adapter_inputs.bake.*` / `<name>.adapter_inputs.helm.*`: compatibility aliases for existing workflow contracts; prefer `<name>.adapter_inputs.overlays.<family>.*` for new authoring
- the workflow overlay map is generalized structurally, and shipped workflow runtime semantics currently exist for `compose`, `bake`, and `helm`; unsupported overlay families fail validation instead of silently acting as inert metadata
- `<name>.env.profile`: optional env profile name from `env.profiles`
- `<name>.env.compose_env_file_services`: optional compose-managed services that should consume the
  selected workflow profile's rendered dotenv artifact as `services.<service>.manager.env_file`
- `<name>.prepare.task`: optional native finite task ota should run first as explicit host preparation for that workflow
  - must reference a declared task with one finite body: `run`, `script`, `command`, `prepare`, or `action`
  - must not reference a `launch` task or a task with `runtime`
  - must resolve to native execution
- `<name>.prepare.action`: optional workflow-owned deterministic host prepare action ota should run first for that workflow
  - must declare one first-class `action` body such as `copy_if_missing`, `ensure_env_file`, `ensure_container_network`, or `ensure_bundle`
  - current workflow prepare shape requires exactly one of `<name>.prepare.task` or `<name>.prepare.action`
- `<name>.setup.task`: optional task ota should treat as the preparation phase for that workflow
- `<name>.run.task`: optional task ota should treat as the primary runnable surface for that workflow
- `<name>.attach.task`: optional task ota should treat as the canonical interactive re-attach lane for that workflow
  - use this when the truthful workflow path starts in the background and a separate named task re-attaches to an existing interactive session
  - `ota up --attach` uses this lane after readiness when it is declared instead of assuming the run task itself stays foreground
- `<name>.services.required`: optional services that belong to that workflow
- `<name>.readiness.checks`: optional readiness checks that belong to that workflow
- `<name>.readiness.probes`: optional reusable readiness probes that belong to that workflow
- `<name>.readiness.surfaces`: optional attached runtime surfaces that belong to that workflow's selected run task
- `<name>.readiness.signal.checks`: optional non-gating checks surfaced as informational readiness signals
- `<name>.readiness.signal.probes`: optional non-gating reusable probes surfaced as informational readiness signals
- `<name>.readiness.signal.surfaces`: optional non-gating attached surfaces surfaced as informational readiness signals
- a readiness entry must be declared in exactly one lane (`readiness.*` or `readiness.signal.*`)
- `<name>.exposes`: optional human-readable endpoints or URLs the workflow is expected to surface
  - literal string form keeps a fixed URL
  - object form `{ surface: <name> }` resolves through the selected workflow run task

Service manager fields:

- `services.<name>.manager.kind`: `compose` or `host`
- `services.<name>.manager.engine`: optional compose CLI engine for `kind: compose`; `docker` by default, `podman` also supported. Keep task `command.exe` / `launch.exe` aligned to the same engine when the task body directly executes the compose CLI.
- `services.<name>.manager.name`: optional manager/project name; required today for `kind: compose`
- `services.<name>.manager.file`: optional compose file path for `kind: compose`
- `services.<name>.manager.files`: optional ordered compose file stack for `kind: compose`
- `services.<name>.manager.env_file`: optional repo-relative compose env-file path for `kind: compose`
- `services.<name>.manager.env_files`: optional ordered repo-relative compose env-file stack for `kind: compose`
- `services.<name>.manager.profiles`: optional compose profile list for `kind: compose`
- `services.<name>.manager.service`: optional compose service name override; required today for `kind: compose`
- `services.<name>.manager.host`: optional typed host-manager owner for `kind: host`
  - `kind`: `systemd`
  - `unit`: required systemd unit name ota should start/stop/check
  - `scope`: optional `system` or `user`; defaults to `system`
- `ota assist declare-service --manager host --host-unit <unit> --style systemd-active` is the canonical authoring path when ota should own one systemd-managed host service directly
- `services.<name>.manager.start` / `manager.stop`: optional explicit host lifecycle commands for `kind: host`; do not combine these with typed `manager.host` ownership because ota derives lifecycle from the typed host manager
- validate/doctor warn when tasks still shell `systemctl start`, `stop`, or `is-active` directly for service ownership that should live on the typed `manager.host` + `readiness.kind: systemd_active` surface

Workflow env adapter rules:

- use `<name>.env.compose_env_file_services` when the workflow owns one rendered dotenv artifact
  and named compose services should consume that exact file
- treat `<name>.adapter_inputs.*` as the shared workflow adapter overlay surface: declare the
  base adapter-owned truth there once, then let task-local adapter inputs carry only the narrower
  additions that are specific to one selected path
- ota resolves that workflow overlay through one adapter-field registry across the shipped Compose,
  Bake, and Helm families, so workflow/task/mode precedence, duplicate-ownership governance, and runtime
  env projection stay aligned on one contract-owned field map instead of family-specific drift
- use `<name>.adapter_inputs.overlays.compose.*` when one workflow owns adapter-scoped compose input
  truth for the selected runnable path and task-local compose adapter inputs should only carry
  narrower path-specific additions
- use `<name>.adapter_inputs.overlays.bake.*` when one workflow owns the base Bake adapter root or file stack for
  selected `docker buildx bake` task paths and task-local adapter inputs should only carry narrower
  additions
- use `<name>.adapter_inputs.overlays.helm.*` when one workflow owns the base Helm adapter root, values-file stack, chart selection, release naming, or namespace for selected Helm task paths and task-local adapter inputs should only carry narrower additions
- this keeps compose adapter input ownership on the workflow surface instead of duplicating
  the same `manager.env_file` path across services
- use `services.<name>.manager.files` / `.env_files` when the managed compose service identity
  itself depends on an ordered overlay stack, such as a sidecar service declared only through an
  additional compose file; keep task or workflow `adapter_inputs.overlays.compose.*` for selected
  runnable-path ownership
- when the selected workflow run path includes a compose-running task, ota also projects that
  rendered dotenv artifact into `tasks.<name>.adapter_inputs.overlays.compose.env_files` instead of
  misrouting it through process `env_files`
- ota applies `<name>.adapter_inputs.overlays.compose.cwd` only when the selected compose task path
  does not already declare one; this lets one workflow own `docker/` or similar adapter roots
  without forcing shell `cd ... && docker compose ...` glue back into task bodies
- ota prepends `<name>.adapter_inputs.overlays.compose.files` ahead of task-local
  `adapter_inputs.overlays.compose.files`, preserving declared task additions without letting workflow-owned
  base compose files drift back into shell `docker compose -f` flags
- ota prepends `<name>.adapter_inputs.overlays.compose.profiles` ahead of task-local
  `adapter_inputs.overlays.compose.profiles`, preserving narrower task additions without forcing workflow
  profile truth back into shell `docker compose --profile ...` flags
- ota applies `<name>.adapter_inputs.overlays.compose.project_name` only when the selected task path
  does not already declare one; validate/doctor warn if task-local compose project naming
  duplicates workflow truth
- ota applies `<name>.adapter_inputs.overlays.bake.cwd` only when the selected Bake task path does not
  already declare one; this lets one workflow own a subdirectory-rooted Bake lane without forcing
  shell `cd ... && docker buildx bake ...` glue back into task bodies
- ota prepends `<name>.adapter_inputs.overlays.bake.files` ahead of task-local
  `adapter_inputs.overlays.bake.files`, preserving narrower Bake file additions without forcing workflow
  truth back into shell `docker buildx bake -f` flags
- ota applies `<name>.adapter_inputs.overlays.helm.cwd` only when the selected Helm task path does not
  already declare one, so one workflow can own a chart-rooted Helm lane without forcing shell
  `cd ... && helm ...` glue back into task bodies
- ota prepends `<name>.adapter_inputs.overlays.helm.values_files` ahead of task-local
  `adapter_inputs.overlays.helm.values_files`, preserving narrower values-file additions without forcing workflow
  truth back into shell Helm `-f` flags
- ota applies `<name>.adapter_inputs.overlays.helm.chart`, `.release_name`, and `.namespace` only when the selected Helm task path does not already declare them
- every referenced service must declare `manager.kind: compose`
- the selected profile must declare `render.dotenv`
- if a referenced service also declares `manager.env_file`, it must match the workflow-owned
  rendered dotenv path exactly
- `<name>.adapter_inputs.overlays.compose.env_files` / `.files` must stay repo-relative and must not
  escape the repo
- `<name>.adapter_inputs.overlays.compose.profiles[*]` must not be empty
- `<name>.adapter_inputs.overlays.bake.files` must stay repo-relative and must not escape the repo
- `<name>.adapter_inputs.overlays.helm.values_files` and `.chart` must stay repo-relative and must not escape the repo
- `<name>.adapter_inputs` requires the selected workflow run path to include task paths
  that support each declared adapter input family

Compatibility:

- `<name>.env.adapter_inputs.*` remains accepted as a compatibility lane for older contracts
- `<name>.env.compose_files` and `<name>.env.compose_project_name` remain accepted as compatibility
  aliases for existing contracts
- `services.<name>.manager.file` and `services.<name>.manager.env_file` remain accepted as
  single-entry compatibility aliases; use `manager.files` and `manager.env_files` when a compose
  service owns an ordered overlay stack
- new and updated contracts should use `<name>.adapter_inputs.overlays.*` for workflow-owned adapter truth
- do not declare the compatibility aliases together with the canonical
  `adapter_inputs.overlays.compose.files` / `.project_name` fields

Prepare vs setup vs run:

- `prepare`
  - host-side deterministic bootstrap before setup
  - must point to one native finite prepare owner: either `prepare.task` or inline `prepare.action`
  - use it for file or network preparation such as `copy_if_missing`, `ensure_env_file`, `ensure_container_network`, or `ensure_bundle`
- `setup`
  - repo preparation
  - use it for dependency install, generated artifacts, and other normal bootstrap work
- `run`
  - primary operational path
  - use it for the app, dev server, worker, packaged command, or packaged container launch the workflow is meant to make useful

Do not blur these boundaries:

- do not put ordinary shell setup or runtime startup in `prepare`
- do not use `setup` as a hidden runtime phase
- do not add `prepare` unless the workflow genuinely needs one explicit host bootstrap step before setup

Current behavior:

- workflows do not replace `tasks`, `services`, or `checks`; they compose those primitives into one canonical operational path
- use `workflows.<name>.instances` when one workflow is really a named family of runtime instances such as `ws0`, `ws1`, or `staging` / `prod-preview` rather than a single flat environment
- `workflows.<name>.instances.default` selects the implicit instance for `ota up --workflow <name>` and other selected-workflow commands
- select a non-default instance with `ota up --workflow <name>@<instance>`, `ota proof runtime --workflow <name>@<instance>`, or other workflow-selecting commands
- use `workflows.<name>.instances.generated.<family>` when one workflow owns a bounded repeated instance family such as `ws1..ws8` and the repeated overlays should stay on the existing instance boundary instead of being duplicated across many explicit named items
- generated instance families are finite and deterministic: each family declares one `prefix`, `start`, `end`, and `template`; ota expands only those concrete selectors and does not invent open-ended instance names or general expression evaluation
- use `workflows.<name>.instances.<instance>.topology.requires_instances` when one selected instance must bring up another declared instance first, such as `ws1+` depending on `ws0`
- keep instance truth bounded and explicit: this first-class lane is for instance-specific env overrides, task adapter input overrides, and surface port/path overlays, not arbitrary free-form templating across the whole contract
- generated instance templates may interpolate `${OTA_WORKFLOW_INSTANCE}` and `${OTA_WORKFLOW_INSTANCE_INDEX}` inside string-valued instance overlay fields such as env values, compose project names, or overlay paths
- use `workflows.<name>.instances.<instance>.env` when every task on the selected workflow path should inherit one instance-specific env value such as a cloned workspace root
- use `workflows.<name>.instances.<instance>.tasks.<task>.adapter_inputs` when one selected task path needs instance-specific compose project naming, bake files, or other adapter-owned truth without splitting the workflow into repo-local shell variants
- use `tasks.<name>.variants` with `variants.<i>.env`, `variants.<i>.env_files`, `variants.<i>.env_bindings`, `variants.<i>.inputs`, `variants.<i>.requirements`, or `variants.<i>.adapter_inputs` when one task keeps the same body but needs OS-scoped process, prerequisite, or adapter overlays such as Linux-only host uid/gid interpolation, service-derived URLs, input defaults/allowed values, platform-specific tool requirements, Compose files, env files, or profiles
- use `workflows.<name>.instances.<instance>.tasks.<task>.runtime` when one selected task path needs instance-specific service listener publication or readiness selection and the base task already owns explicit `runtime` truth
- use `workflows.<name>.proof.claim: bounded` when a workflow is a real, archive-backed proof lane
  but has no declared dependency seam. It opts the workflow into `proof_breadth` assurance without
  claiming repo-wide completion; Ota keeps the claim `unknown` until a matching immutable proof
  archive exists. A bounded claim requires `workflows.<name>.run.task` so it always binds to an
  executed lane rather than contract prose.
- use `workflows.<name>.proof.negative_controls` when one declared service seam has a separate,
  finite failure-control task that should run only when explicitly selected by
  `ota proof runtime --negative-control <id>`; the task must remain outside the normal workflow
  closure and must not require the controlled service. It names the marker-bound seam obligation
  it negates and the expected typed failure. Ota records a validated control only when the task
  writes a matching transaction-bound failure attestation after the expected failure; a generic
  non-zero exit remains invalid evidence and does not itself prove causality
- use `workflows.<name>.proof.lifecycle` to declare a bounded lifecycle-proof lane from existing
  `services.<name>` manager truth. `services[]` contains only canonical service references and an
  optional `assertion.task` names one finite task Ota executes after those services are ready. The
  assertion's full dependency closure must remain finite and outside the workflow's normal closure;
  lifecycle declarations never copy start, stop, readiness, or status commands.
- keep workflow-instance runtime specialization on the task runtime boundary: override existing listener bind/project ports or readiness fields there instead of inventing a separate workflow-level listener model
- workflow-instance task runtime overlays currently merge onto an existing top-level task `runtime`; they do not invent new listeners or replace runtime ownership from scratch
- `runtime_boundary` follows the same selected-path precedence ladder: `execution.runtime_boundary`
  is the repo baseline, `workflows.<name>.runtime_boundary` can specialize the selected workflow
  path, and `tasks.<name>.runtime_boundary` remains the narrowest selected-lane owner when the
  workflow resolves through that task
- generated instance templates may derive repeated port families without shell math: `surfaces.<name>.port_stride`,
  `tasks.<name>.runtime.listeners.<listener>.bind.port.stride`, and
  `tasks.<name>.runtime.listeners.<listener>.project.host.port.stride` multiply the generated
  instance index and add it to the declared base port value
- use `workflows.<name>.instances.<instance>.surfaces.<surface>` when the selected instance publishes the same surface shape on different host ports or paths and Ota should keep proof, exposure, and command guidance aligned on the selected instance
- use `prepare.task` when the workflow needs one explicit host-side finite normalization/bootstrap step before setup and that step already deserves its own reusable task identity
- use `prepare.action` when the workflow itself honestly owns one deterministic bootstrap action or bundle and creating a synthetic helper task would only add glue
- use `env.profile` when one workflow owns a truthful runtime env overlay or declared-source specialization
  and that selection should stay declarative instead of being repeated across tasks or hidden in shell
- use `env.profile.render.dotenv` when that workflow also needs one concrete dotenv artifact on disk
  for compose interpolation or another repo-owned runtime input, and Ota should materialize it
  automatically instead of routing through a synthetic prepare action
- add `env.profile.render.dotenv.template` when that artifact should preserve repo-owned baseline
  entries from an example file while Ota deterministically overlays the workflow-specific values
- do not use workflow `prepare` for service startup or runtime launch; that still belongs in `setup.task`, `services`, or `run.task`
- use `services.<name>.manager.env_file` when one compose-managed service depends on a workflow/runtime-specific interpolation file and Ota should own that `docker compose --env-file ...` input instead of repeating it inside shell commands
- use `services.<name>.manager.profiles` when one compose-managed service owns stable profile
  selection and Ota should pass those `docker compose --profile ...` inputs declaratively instead
  of repeating them inside shell commands
- `doctor` diagnoses the default workflow by default when it declares workflow readiness probes,
  workflow readiness checks, or workflow services
- selected-workflow `doctor`, `up`, and `proof runtime` paths apply `workflows.<name>.env.profile`
  before evaluating task env overlays and declared env sources
- `check` follows the same selected workflow readiness boundary when a workflow declares explicit
  readiness probes or checks, and otherwise falls back to the repo-wide `checks` surface
- `doctor` and `check` may also validate `workflows.<name>.readiness.surfaces` through the selected
  workflow run task without hardcoding host URLs into the workflow
- workflow `exposes` may point at attached surfaces instead of repeating host URLs that the
  contract already owns under `surfaces`
- use `checks[].probe` when a named check should reuse a named readiness probe outside that
  workflow-scoped path or when the repo does not declare workflows
- `ota up` now targets the default workflow instead of assuming repo-wide `setup` semantics
- if `workflows.<default>.prepare.task` or `.prepare.action` is declared, `ota up` runs that host prepare phase before required service startup or setup
- execution planning carries workflow `prepare` as additive workflow context, but it does not become the concrete execution identity for `ota execution plan`

Bounded proof shape:

```yaml
workflows:
  verify:
    run:
      task: gate
    proof:
      claim: bounded
```

This declares a truthful proof claim for a finite verification lane such as an offline replay,
build, or deterministic test gate. It does not claim a dependency seam, causal interaction, or
broader repository completion. Run `ota proof runtime --workflow verify --archive`; Doctor emits
`proof_breadth: unknown` until the archive matches the current contract, source identity, and
execution scope.

Negative-control shape:

```yaml
workflows:
  app:
    run:
      task: serve
    proof:
      seam_observations:
        - id: postgres-marker
          dependency: postgres
          producer_task: write-proof-marker
          task: observe-proof-marker
          marker_env: OTA_PROOF_SEAM_MARKER
      negative_controls:
        - id: postgres-unavailable
          dependency: postgres
          obligation: postgres-marker
          task: verify:postgres-unavailable
          intervention:
            kind: dependency_endpoint_override
          expected_failure: dependency_unavailable
```

The contract declares the controlled dependency, the already-observed green seam obligation, the
separate task, the intervention family, and a typed expected failure. Ota executes the task only when selected. It supplies
the active transaction, control, obligation, and transient attestation coordinates; the control
must write a matching `dependency_unavailable` attestation after that failure is actually caught.
A successful control task, an unrelated non-zero exit, or a stale/mismatched attestation is
`invalid`, not a fault test. This surface does not infer disruption from prose or fabricate a
generic fault injector. Until a matching control validates, an otherwise exercised seam retains
the machine-readable `dependency_causality_not_proved` boundary.

`intervention.kind` is a bounded contract declaration of what the finite control task changes:

- `dependency_disruption` blocks or removes the declared dependency from the control lane.
- `dependency_endpoint_override` redirects the control lane away from the declared dependency.
- `null_substitution` replaces the declared dependency with a bounded null implementation.

This is not the proof result. Ota records the declared intervention beside runner-derived control
status, failure mode, transaction binding, and attestation digest. Only a matching
`expected_missing_effect` attestation promotes the dependency evidence to `fault_tested`.
That promotion is deliberately limited to the declared seam obligation. The proof receipt retains
`dependency_output_shaping_not_proved` for that same obligation, so consumers cannot read a
validated control as proof that the dependency shaped a broader application output.

Seam-observation shape:

```yaml
workflows:
  app:
    proof:
      seam_observations:
        - id: postgres-marker
          dependency: postgres
          producer_task: write-proof-marker
          task: observe-proof-marker
          marker_env: OTA_PROOF_SEAM_MARKER
```

Ota issues one opaque marker for the proof, injects it into the declared producer task, and then
runs the finite observer before cleanup. The observer never receives the marker directly. It
receives a transaction identifier and runner-owned transient attestation path, and must recover
the marker through the declared dependency before writing its JSON attestation. Ota verifies the
transaction id, observation id, and marker before promoting the dependency to `exercised`; failed
or ambiguous observations retain `dependency_exercise_not_proved`.

The observer is deliberately narrow: it must be finite, remain outside the normal workflow
closure, require the named service, and have every prerequisite already owned by the normal
workflow closure. The observed service must also be part of that normal closure. This prevents a
proof-only task from silently repeating setup or minting an `exercised` claim for a service outside
the selected runtime path. `producer_task` must be in the selected workflow closure and require
the observed service. The marker value and transient attestation are never rendered in output;
the receipt retains only transaction and attestation digests. Ota carries marker bindings internally
and injects the raw marker only into `producer_task`; contract-owned environment defaults cannot
replace it at execution time.

- if `setup.task` already depends on the same action task, direct task execution still follows the task graph while workflow `ota up` avoids running the same prepare action twice
- if `workflows.<default>.setup.task` is declared, `ota up` uses that task as the setup phase
- if `workflows.<default>.run.task` is declared and the task has a service runtime, `ota up` activates that task as part of readiness
- `tasks.setup` remains the compatibility fallback for the unselected default path; `ota up
  --workflow <name>` does not invent a setup phase when that workflow omits `setup.task`
- `agent.default_task` and `agent.entrypoint` remain agent-facing hints, but the default workflow is now the canonical repo operational path

Use this shape when a repo needs `.env.local` or another deterministic local file before setup, but
the actual setup path should still stay in the repo's preferred execution plane:

```yaml
tasks:
  setup:env:local:
    execution:
      default_mode: native
    action:
      kind: copy_if_missing
      from: .env.example
      to: .env.local

  setup:
    run: pnpm install
    depends_on:
      - setup:env:local

workflows:
  default: app
  app:
    prepare:
      task: setup:env:local
    setup:
      task: setup
    run:
      task: dev
```

This keeps the boundary honest:

- direct `ota run setup` still follows the task graph and bootstraps the file if needed
- `ota up` shows and runs one explicit host prepare phase before setup
- the repo does not need to make all of setup native just to create one local file

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
  - name: workspace-dependencies-installed
    kind: file
    severity: error
    path: node_modules
    expect: directory
  - name: compose-env-compatible
    kind: env
    severity: error
    env:
      path: .env.compose
      assertions:
        - key: REDIS_HOST
          host:
            allowed:
              - redis
              - cache
        - key: DATABASE_URL
          url_host:
            policy: not_loopback
        - key: APP_ENV
          not_equals:
            - development
            - local
  - name: app-source-changed
    kind: changed_files
    severity: info
    changed_files:
      paths:
        - apps/web/**
      include_untracked: true
```

Fields:

- `name`: required, non-empty string
- `kind`: `precondition`, `health`, `file`, `env`, or `changed_files`
- `severity`: `error`, `warn`, or `info`
- `run`: optional shell command when the check is command-backed
- `probe`: optional probe reference when the check is probe-backed
- `path`: optional repo-relative path when the check is file-backed
- `scope`: optional for `kind: file`; `repo` (default) keeps the path inside the repo, while
  `workspace` allows a relative sibling path such as `../task-sdk/schema.json`
- `expect`: required for `kind: file`; one of `exists`, `file`, `directory`, or `missing`
- `env`: required for `kind: env`
  - `env.path`: required repo-relative dotenv file path
  - `env.assertions`: required non-empty assertions
    - `key`: required env key
    - exactly one of `equals`, `not_equals`, `state`, `host`, or `url_host`
    - `not_equals`: optional non-empty list of disallowed exact values
    - `state`: optional `present` or `missing`
    - `host` / `url_host`: declare exactly one of:
      - `policy`: currently `not_loopback`
      - `allowed`: non-empty list of allowed hostnames
- `changed_files`: required for `kind: changed_files`
  - `changed_files.paths`: required non-empty repo-relative path matchers
  - `changed_files.base_ref`: optional git base ref for diff range
  - `changed_files.head_ref`: optional git head ref for diff range
  - `changed_files.include_untracked`: optional boolean; when true, untracked files matching
    `paths` also satisfy the check
- `timeout`: optional integer in milliseconds

Choose check kind by intent:

- use `kind: precondition` for prerequisite commands that must pass before setup or run
  (runtime/tool presence, host capability checks, policy gates)
- use `kind: health` with `probe` for readiness/liveness that should reuse one declared
  `readiness.probes.<name>` contract
- use `kind: file` for deterministic filesystem expectations without shell drift
  (`node_modules` exists, lockfile is present, bootstrap file is intentionally missing, sibling
  workspace schema input is present)
- use `kind: env` for deterministic dotenv assertions without shell grep drift
  (compose-compatible host rewrites, non-loopback service hosts, exact required values, key
  presence/absence)
- use `kind: changed_files` when the gate should depend on whether a path set changed in git
  rather than host/runtime state

`kind: file` + `expect` decision:

- `expect: exists` when either file or directory is acceptable
- `expect: file` when only regular file presence should satisfy the check
- `expect: directory` when only directory presence should satisfy the check
- `expect: missing` when absence is required (for example enforce no generated artifact in tree)

`kind: file` + `scope` decision:

- omit `scope` or use `scope: repo` when the path must stay inside the repo root
- use `scope: workspace` when the truthful input is a sibling or parent-relative workspace path
  such as `../task-sdk/schema.json`
- `scope: workspace` still requires a relative path and still rejects absolute paths

`kind: env` decision:

- use `state: present` when only key presence matters
- use `state: missing` when one key must stay absent from a workflow-scoped overlay
- use `not_equals` when one key must avoid a small set of known-bad exact values such as
  `localhost`, `development`, or `local`
- use `host.policy: not_loopback` when one env key should point at a service/container hostname
  rather than `localhost`, `127.0.0.1`, or `::1`
- use `url_host.policy: not_loopback` when the env value is a URL and only the URL host should
  be checked
- use `host.allowed` when one env key must resolve to one of a small set of truthful service host
  names such as `redis` or `cache`
- use `url_host.allowed` when the env value is a URL/DSN and only specific hostnames such as
  `postgres` or `db` are valid
- use `equals` when one exact deterministic value must be present

`kind: changed_files` decision:

- set only `paths` to compare against `HEAD` by default
- set both `base_ref` and `head_ref` for explicit CI ranges (for example PR base to branch head)
- set `include_untracked: true` when new untracked files in the matcher set should count as
  changed

`severity` decision:

- use `error` when failure should block readiness and execution
- use `warn` when failure is actionable but should not block the main path
- use `info` when the check is informational signal only (for example change-scope hints)

`timeout` decision:

- set timeout for checks that can hang or run unpredictably on shared CI agents
- keep timeout unset for fast deterministic checks (ota uses normal completion behavior)
- for probe-backed checks, set timeout on the check only when this check needs a tighter override
  than the shared probe timeout

Current behavior:

- `up` uses preconditions before setup
- `doctor` runs configured checks and reports findings by severity
- checks must declare exactly one of `run`, `probe`, `path`, `env`, or `changed_files`
- `checks[].probe` must reference a named `readiness.probes.<name>` declaration
- file checks use the filesystem directly and do not invoke a shell; prefer them over
  `run: test -d ...` or other OS-specific shell checks for file and directory state
- repo-scoped file checks stay inside the repo boundary by default; widen to `scope: workspace`
  only when the contract truth really depends on sibling workspace inputs
- env checks parse dotenv files directly and should be preferred over shell `grep`, `findstr`, or
  ad hoc scripting when the contract needs deterministic env-file assertions
- validate/doctor emit governance warnings for obvious shell file-state and env-file checks that
  should be rewritten as first-class `kind: file` or `kind: env` checks
- validate/doctor also warn when task bodies rewrite `.env*` files through obvious shell mutation
  (`sed -i`, `perl -pi`) that should instead use `action.kind: ensure_env_file` with explicit
  replacement keys
- validate/doctor warn when task bodies hard-code compose shell flags such as
  `docker compose --env-file ...`, `-f`, `--file`, `--profile`, `-p`, or `--project-name`; move
  that ownership to task `adapter_inputs.overlays.compose.*` or
  `services.<name>.manager.env_file` so Ota
  can reason about it
- changed-files checks evaluate tracked diffs via git (`base_ref..head_ref` when both refs are
  declared, otherwise against `HEAD`) and may include untracked matches when
  `include_untracked: true`
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
  exceptions:
    sensitive_writes:
      - .github/workflows
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
      source:
        kind: version
        version: v1.6.16
  notes: Keep agent edits narrow and add regressions for behavioral changes.
```

For deterministic unreleased proof, the same bootstrap surface may pin an exact git revision:

```yaml
agent:
  bootstrap:
    ota:
      source:
        kind: git_rev
        rev: e71931d6a41cc52a15966e0125725b67a05cc073
```

For active pressure testing, the bootstrap surface may intentionally track a branch tip:

```yaml
agent:
  bootstrap:
    ota:
      source:
        kind: branch
        branch: branch-name
```

Ota renders the matching shell and PowerShell installer commands from `bootstrap.ota.source`.
Legacy `bootstrap.ota.sh` / `bootstrap.ota.powershell` commands are still accepted for
compatibility, and Ota will infer `version`, `git_rev`, or `branch` truth from those commands
when it can.

Current validation rules:

- `entrypoint` must reference a known task when set
- `default_task` must reference a known task when set
- `safe_tasks` entries must reference known tasks
- every `refusal_canaries` entry must declare exactly one known `task` or known `workflow`
- `verify_after_changes` entries must reference known tasks
- `writable_paths` entries must not be empty and must be normalized relative paths
- `exceptions.sensitive_writes` entries must not be empty and must be normalized relative paths
- `protected_paths` entries must not be empty and must be normalized relative paths
- `writable_paths` entries must not duplicate `protected_paths`; protected descendants may still
  carve out narrower exceptions inside a broader writable root
- `exceptions.sensitive_writes` entries must overlap a declared `writable_paths` boundary
- task `effects.writes` entries must be normalized relative paths when present
- task `effects.external_state` entries must be non-empty lowercase tokens when present
- agent-safe task writes must not overlap declared `protected_paths`
- when `writable_paths` is declared, agent-safe task writes must stay inside that writable boundary
- `inferred_boundary.provenance.writable_paths` entries must not be empty when present
- `inferred_boundary.provenance.protected_paths` entries must not be empty when present
- `inferred_boundary` must include at least one provenance entry when present
- `bootstrap.ota` must include at least one of `source`, `sh`, or `powershell` when present
- `bootstrap.ota.source.kind: version` must declare a pinned ota release like `v1.6.21`
- `bootstrap.ota.source.kind: git_rev` must declare a full 40-character commit SHA
- `bootstrap.ota.source.kind: branch` must declare a non-empty branch name
- unpinned `bootstrap.ota.sh` / `bootstrap.ota.powershell` commands now warn during `ota validate`
  so repo agent bootstrap stays deterministic across release drift
- `bootstrap.ota.source.kind: branch` also warns during `ota validate` / `ota doctor`, because
  branch tracking is pressure truth rather than deterministic proof truth

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

- `posture` declares the default authority boundary for agent edits
  - `readiness_strict` is the default and treats repo-contract, CI, runtime-topology,
    env/config, and lockfile writable paths as sensitive
  - `contract_authoring` explicitly authorizes repo-contract editing
  - `infra_authoring` explicitly authorizes CI and runtime-topology editing
- `entrypoint` is the first task an AI agent should use to get oriented in the repo
- `default_task` is the normal verification task to run when no more specific task is needed
- `safe_tasks` are the tasks an AI agent can run without broad risk
- `refusal_canaries` declare negative controls that must be refused by the real agent runner;
  each entry names only a `task` or `workflow`, while Ota derives the current refusal reason and
  closure at execution time
- run a declared task canary with `ota run --agent --expect-refusal <task>`, or a declared
  workflow canary with `ota up --agent --expect-refusal --workflow <workflow>`; either command
  exits `0` only when the runner refuses before task or workflow execution begins
- when a repo declares a matching safe or verification task, agents should prefer `ota run <task>`
  over raw package-manager or language-tool commands; fall back only when no truthful Ota task
  exists or when isolating an Ota defect
- `safe_for_agent: false` is just the default; omit it unless explicit `true` improves readability
- task `effects.writes` makes the expected durable writes explicit so agent-safe task claims can be checked structurally
- task `effects.network` makes connectivity dependence explicit for agent-safe and CI-visible task review
- task `effects.external_state` marks out-of-repo mutation such as Docker, database, or hosted-service state
- agent-safe tasks with `effects.network` or `effects.external_state` can surface contract
  advisories so unattended execution risk stays explicit during validate and doctor flows
- first-class `prepare.kind: dependency_hydration` lanes are treated more narrowly than opaque
  shell network use: when the agent-safe task path is already on ota's typed dependency-hydration
  surface, ota keeps the network truth explicit but does not emit the generic agent-safe
  dependency-hydration warning just for that bounded lane
- `verify_after_changes` are the tasks an AI agent should rerun after modifying files
- `writable_paths` are the paths an AI agent may edit
- `exceptions.sensitive_writes` records narrow intentional exceptions for sensitive writable paths
  or broader writable boundaries when the declared `posture` is still otherwise correct
- when `ota.yaml` is intentionally writable, keep it explicit: starter contracts from `ota init`
  protect `ota.yaml` by default, and any broader contract-authoring slice should pair writable
  `ota.yaml` authority with `exceptions.sensitive_writes: [ota.yaml]`
- use `exceptions.sensitive_writes` only when the path is already writable through
  `agent.writable_paths`, the path is sensitive, and the contract intentionally grants that
  authority; do not use it merely to say a protected path is important
- do not use `exceptions.sensitive_writes` for normal readiness slices where the agent should not
  edit the contract, CI, topology, env/config, or lockfiles; put those files under
  `protected_paths` instead
- `protected_paths` are the paths an AI agent should avoid editing casually
- lockfiles, env/config files, runtime-topology files, CI workflow files, and repo contracts
  should usually stay under `protected_paths` for readiness slices unless the contract explicitly
  authorizes that authoring scope
- `inferred_boundary.reviewed: false` means ota inferred the current agent boundary but the repo author has not confirmed it yet
- `inferred_boundary.provenance` explains which starter or detector heuristics produced the current writable and protected boundary
- `bootstrap.ota` provides an approved `ota` install path for agents when the binary is missing
- `bootstrap.ota.note` should explain when that install path may be used
- `bootstrap.ota.source` is the canonical install truth for ota bootstrap
  - `kind: version` is the normal released proof lane
  - `kind: git_rev` is the deterministic unreleased proof lane
  - `kind: branch` is the active pressure-testing lane and is intentionally non-deterministic
- first-time consumers should treat `bootstrap.ota.source` as directly executable bootstrap truth
  through the official installer mapping:
  - `kind: version` maps to the release installer lane
    - POSIX shell:
      `curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION=<version> sh`
    - PowerShell:
      `$env:OTA_VERSION='<version>'; irm https://dist.ota.run/install.ps1 | iex`
  - `kind: git_rev` maps to the git installer lane
    - POSIX shell:
      `curl -fsSL https://dist.ota.run/install.sh | OTA_GIT_REV=<rev> sh -s -- --from-git`
    - PowerShell:
      `$env:OTA_GIT_REV='<rev>'; & ([scriptblock]::Create((irm https://dist.ota.run/install.ps1))) -FromGit`
  - `kind: branch` maps to the same git installer lane
    - POSIX shell:
      `curl -fsSL https://dist.ota.run/install.sh | OTA_GIT_BRANCH=<branch> sh -s -- --from-git`
    - PowerShell:
      `$env:OTA_GIT_BRANCH='<branch>'; & ([scriptblock]::Create((irm https://dist.ota.run/install.ps1))) -FromGit`
- GitHub Actions jobs that need direct `ota` commands should consume that same truth through the
  public `ota-run/setup@v1` action in `source: contract` mode instead of hardcoding
  release/source install choices separately in workflow YAML
- `bootstrap.ota.sh` and `bootstrap.ota.powershell` remain compatibility fields; when omitted,
  ota renders the approved shell and PowerShell install commands from `bootstrap.ota.source`
- `notes` is free-form repo guidance for humans and AI agents
- `ota detect --merge` and `ota detect --rewrite` refuse to write protected paths declared by the existing contract

Authoring ergonomics:

- readiness slice (default): keep `posture: readiness_strict`, keep lockfiles/env/config/topology/CI/contract under `protected_paths`, and leave `exceptions.sensitive_writes` empty
- contract-authoring slice: allow writable `ota.yaml` intentionally and acknowledge it with `exceptions.sensitive_writes: [ota.yaml]`
- infra-authoring slice: allow writable CI/topology files intentionally and acknowledge only those specific sensitive paths
- use `bootstrap.ota` only when you want agents to self-install ota if missing; prefer
  `bootstrap.ota.source` over raw shell strings, keep deterministic proof on `version` or
  `git_rev`, and use `branch` only for active pressure lanes

## `governance.crossing_authority`

Optional. Use this only when a platform-installed authority must approve human execution of
contract-derived heavier task or workflow closures.

```yaml
governance:
  crossing_authority:
    authority_id: platform-release-authority
```

The contract names an authority; it does not contain trust material. The first `prebound_file`
adapter resolves that identifier from a fixed protected system store:

- macOS: `/Library/Application Support/Ota/crossing-authorities.json`
- Linux: `/etc/ota/crossing-authorities.json`

The system binding owns the issuer, Ed25519 key, signed-bundle path, protected monotonic sequence
state, accepted contract identities, and freshness bounds. Ota has no environment, CLI,
repository, workspace, or `OTA_POLICY` override for these paths. The trust store, bundle, sequence
state, and their parent directories must be root-owned and not group/world writable. This protects
against Ota's current unprivileged process only; it does not prove that the invoking job lacks
`sudo`, capabilities, or namespace control. `prebound_file` therefore emits
`authority_separation_posture: current_process_filesystem_guarded` and is not a provider-attested
privilege-separation claim. Windows refuses this carrier until Ota can verify an equivalent ACL
boundary.
The independently managed platform-authority process advances the signed bundle and sequence state;
Ota reads and verifies them but does not run as root or mutate authority state.

For version-matched operator specifications, see
[Prebound crossing authority operations](crossing-authority-operations.md) and
[Broker crossing authority operations](broker-crossing-authority-operations.md). Public operator
references are [Prebound Crossing Authority](https://ota.run/docs/reference/prebound-crossing-authority)
and [Broker Crossing Authority](https://ota.run/docs/reference/broker-crossing-authority).

When configured:

- routine agent-safe closures run normally without a grant;
- non-agent execution of a derived heavier closure requires independently managed authority;
- `prebound_file` requires `--grant <id>`; `authority_broker` resolves exactly one protected
  binding and may accept `--grant <authority-id>` only as a non-secret label check;
- every authorization must match the exact semantic contract and selected execution graph;
- unresolved free-form task-input identity refuses instead of hashing or exposing secret values;
- authority attribution is runner-observed as `agent` or `non_agent`; Core does not infer human
  identity merely because `--agent` was omitted;
- grant admission happens before provider, dependency, setup, service, or child-process mutation;
- real execution persists a runner-owned per-scope crossing transaction before selected-lane
  mutation, finalizes it on every terminal path, and uses that transaction identity as the fresh
  crossing-record identity;
- admitted authority and terminal transaction evidence are archived with the normalized contract
  snapshot for later re-derivation;
- a grant never weakens `ota run --agent` or `ota up --agent` refusal.

The signed-file carrier uses short calendar validity and protected sequence/clock high-water
evidence. It is bounded offline authority, not an online approval system. The Unix broker carrier
uses a protected fixed binding and launcher-session descriptor to verify a challenge-bound
attestation, signed exact-scope decision, prepared lease, and atomic one-use consumption before
`ota run` or `ota up` starts selected work. Dry-run never contacts the launcher. Grant-required
runtime and lifecycle proof remain unsupported until one terminal broker transaction can cover the
complete proof invocation and cleanup set.

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
      field_source_class:
        project.name: environment_toolchain
        tools.pnpm: environment_toolchain
```

This is an open map for extra repo-specific values.

`ota detect --write`, `ota detect --merge`, and `ota detect --rewrite` record ota-managed detect
fields under `metadata.ota.detect.field_ownership` using `merged`.

When ota writes detector-owned fields, it can also record the additive detector-governance class
for those fields under `metadata.ota.detect.field_source_class`, for example
`environment_toolchain`, `task_command`, `runtime_service`, `ci_verification`,
`agent_boundary`, `workspace_bootstrap`, or `heuristic`.

The `metadata.ota.detect` subtree is ota-reserved and must remain mapping-shaped. If `metadata.ota`
or `metadata.ota.detect` is repurposed as a scalar or list, detect merge cannot persist ownership
metadata and will fail until that path is repaired.

`metadata.ota.minimum_version` is the reserved compatibility hint for contracts that require a
newer ota binary than the one an operator may have installed. Keep it as a semver string such as
`1.6.16`. Current ota builds validate it, reject contracts whose declared minimum exceeds the
running binary, and use it to explain newer-field parse failures more clearly than a raw
`unknown field` message. Compatibility failures now report the contract minimum, the current
binary identity, any detected unsupported contract feature, and the next upgrade or rebuild step;
`ota --version --json` is the confirmation surface for that operator path.

You can also pin curated fields explicitly with `manual` there when detector silence should not be
treated as contract drift. When a field has no detect ownership entry, ota treats the existing
contract value as `manual` by default.

## Full example

See:

- [../../examples/full-contract/ota.yaml](../../examples/full-contract/ota.yaml)
