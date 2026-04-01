# Contract (`ota.yaml`)

`ota.yaml` is the one file ota uses to explain a repo to humans, CI, and agents.

Use it when you want:

- deterministic setup instead of tribal knowledge
- one place for runtime, tool, service, and task expectations
- the same contract for local development and automation

## Source model

This page is the canonical public reference for the repo contract. It adds
examples, use cases, and operator guidance so the page stands on its own while
staying aligned with shipped behavior.

For service behavior, see [service-behavior.md](service-behavior.md).
For policy packs, see [policy-packs.md](policy-packs.md).
For audit and provenance, see [audit-and-provenance.md](audit-and-provenance.md).
For compatibility rules, see [compatibility-policy.md](compatibility-policy.md) and
[compatibility-surface.md](compatibility-surface.md).
For extension execution boundaries, see [extension-execution-boundary.md](extension-execution-boundary.md).
For mutation and caching rules, see [mutation-controls-and-caching.md](mutation-controls-and-caching.md).

## Primary sections

- `version`: contract schema version. Today this is `1`.
- `project`: stable repo identity and high-level classification.
- `runtimes`: required language/runtime versions.
- `tools`: external CLI and tool dependencies.
- `env`: required environment variables, defaults, and allowed values.
- `services`: supporting services such as databases or queues.
- `checks`: explicit preconditions and health checks.
- `tasks`: named commands that humans and agents can run.
- `execution`: where tasks run, such as native, container, or remote.
- `agent`: safe-task and writable-path hints for agents.
- `extensions`: staged extension-contract data that ota parses but does not execute yet.
- `metadata`: open map for repo-specific values that do not need a first-class field yet.
- `workspace`: monorepo root/member mapping.

## Quick read

Think about the file in this order:

1. `version` and `project` identify the repo.
2. `runtimes`, `tools`, `env`, and `services` describe what the repo needs.
3. `checks` and `tasks` describe what the repo can verify and run.
4. `execution`, `agent`, and `extensions` describe how ota should run, expose those actions, and stage future extension behavior.
5. `metadata` carries extra repo-specific values for ownership, provenance, or local conventions.
6. `workspace` is only for monorepo root/member orchestration.

## Example

```yaml
version: 1
project:
  name: example-repo
  type: application
runtimes:
  node: "22"
tools:
  pnpm: "10"
env:
  OTA_ENV:
    required: true
    default: local
services:
  postgres:
    required: true
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -U app -d app
checks:
  - name: node-installed
    kind: precondition
    severity: error
    run: node --version
tasks:
  setup:
    run: pnpm install
    safe_for_agent: true
  test:
    variants:
      - when:
          os: linux
        run: pnpm test -- --runInBand
      - when:
          os: macos
        run: pnpm test -- --runInBand
      - when:
          os: windows
        script: |
          pnpm test -- --runInBand
  lint:
    description: Run lint checks
    run: pnpm lint
    safe_for_agent: true
execution:
  preferred: native
extensions:
  demo:
    kind: checker
    command: ota-ext-demo
    api_version: 1
agent:
  default_task: setup
metadata:
  team: platform
workspace:
  type: monorepo
  members:
    - apps/web
    - services/api
```

## What each section means

### `project`

Use `project` for the repo’s stable identity. Keep churn-heavy metadata out of it unless the contract
explicitly needs it.

Why users need it:

- gives ota a stable repo name for diagnosis and reporting
- keeps the repo from being guessed from surrounding paths or README text
- makes CI, agents, and workspace reports easier to read

### `runtimes`

Use `runtimes` for the language/runtime versions the repo needs before it is runnable.

Why users need it:

- tells ota doctor what runtime is missing or too old
- avoids hidden setup assumptions in README prose
- keeps local, CI, and agent environments aligned

### `tools`

Use `tools` for command-line dependencies that must be present on PATH.

Why users need it:

- surfaces hidden CLI prerequisites before a task fails
- lets ota doctor explain missing commands directly
- keeps install steps separate from repo tasks

### `env`

Use `env` for required environment values, defaults, and allowed values. If `secret: true` is
set, ota redacts the value in execution receipts and refuses to inline it through remote shell
wrappers.

Why users need it:

- makes required runtime env explicit
- gives receipts and doctor a single source for missing or inherited values
- keeps secrets out of logs and remote command strings

### `services`

Use `services` for supporting infrastructure the repo expects to start, stop, or health-check.

Why users need it:

- makes local service dependencies visible instead of hiding them in shell docs
- lets ota up start and verify the right supporting services in order
- lets ota doctor explain why the repo is not yet ready

### `checks`

Use `checks` for explicit preconditions and health checks that should be run and reported.

Why users need it:

- captures “must be true” conditions that are not task execution
- lets ota doctor and ota check report readiness without mutating state
- gives CI a deterministic gate before merge

### `tasks`

Use `tasks` for deterministic repo commands such as `setup`, `test`, `lint`, and `dev`.
Use task `description` for the short summary and task `notes` for the task purpose plus extra
guidance like when to run it or what it is for.
Use task `env` when a task needs fixed environment values that should override repo-level env for that task.
Use task `inputs` when a task needs named per-run values like `base_url`, `tenant`, or `mode`.
Use task `variants` when one task name needs different commands on different operating systems.
Input names use lowercase snake_case. ota maps them to `--kebab-case` flags and injects them as `OTA_INPUT_<NAME>`.
Defaults are optional; `required: true` makes an input mandatory unless a default exists; `allowed`
limits accepted values.
If every declared input has a default, the task can be run with no input flags.
Use `safe_for_agent` when a task is safe for agents to run without extra guardrails.

Why users need it:

- gives one canonical task list instead of scattered README shell snippets
- lets ota run tasks in dependency order
- tells agents which tasks are safe to run automatically

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
        default: http://localhost:8080
      mode:
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
ota run api-automation-tests --base-url http://localhost:8080 --mode contract-drift
ota run version:bump --version 0.2.0
```

Variants:

- `when.os` selects the task body for `linux`, `macos`, or `windows`
- each variant must define exactly one of `run` or `script`
- ota picks the matching OS variant first, then falls back to the default task body

Use cases:

- use `variants` when Windows needs a different shell command from macOS or Linux
- use `variants` when the same task name should stay stable but the command differs by platform
- use `variants` when a single repo must support developer machines across multiple operating systems
- use `safe_for_agent` when the task is a normal repo action like setup, lint, or test and should be runnable without manual review

Example flow:

```yaml
tasks:
  setup:
    run: pnpm install
  build:
    depends_on:
      - setup
    run: pnpm build
  package:
    depends_on:
      - build
    run: tar -czf dist/release.tar.gz dist/
  upload:
    depends_on:
      - package
    run: ./scripts/upload-artifact.sh dist/release.tar.gz
  test:
    variants:
      - when:
          os: linux
        run: pnpm test
      - when:
          os: macos
        run: pnpm test
      - when:
          os: windows
        script: |
          pnpm test
```

## How to use it

- start with `ota doctor` to see what the repo is missing before you write a contract
- use `ota init` when you need a starter `ota.yaml` for a new or partially described repo
- use `ota detect --dry-run` when you want ota to compare the repo against the declared contract
- use `ota run` when you want a task to execute under the contract
- use `ota up` when you want ota to prepare the repo and report ready/not-ready state

The contract is the source of truth. The command output should reflect it, not replace it.

## Use cases

- onboarding a new repository with explicit setup, test, and release commands
- making CI and agent behavior deterministic instead of guessing from README text
- defining service dependencies and health checks in one file
- capturing allowed runtime and tool versions for a library or SDK repo
- describing workspace members so monorepos can be bootstrapped consistently
- keeping execution boundaries explicit when native, container, or remote backends are needed

## Practical examples

### Application repo

Use `ota.yaml` to describe the app’s runtime, tools, checks, and day-to-day tasks:

```yaml
version: 1
project:
  name: acme-web
  type: application
runtimes:
  node: "22"
tools:
  pnpm: "10"
tasks:
  setup:
    run: pnpm install
    safe_for_agent: true
  build:
    depends_on:
      - setup
    run: pnpm build
  test:
    depends_on:
      - setup
    run: pnpm test
  ci:
    depends_on:
      - build
      - test
    run: pnpm build && pnpm test
agent:
  entrypoint: setup
  default_task: ci
```

This keeps the repo readable for humans and predictable for automation.

### Service repo

Use the contract to describe what must be running before the service is ready:

```yaml
version: 1
project:
  name: acme-api
  type: application
services:
  postgres:
    required: true
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -U app -d app
checks:
  - name: database-ready
    kind: service
    severity: error
    run: pg_isready -U app -d app
tasks:
  setup:
    run: npm ci
  test:
    depends_on:
      - setup
    run: npm test
```

This makes readiness visible instead of leaving it inside tribal knowledge or shell scripts.

### Workspace repo

Use the workspace section when a monorepo or workspace needs multiple members bootstrapped together:

```yaml
workspace:
  type: monorepo
  members:
    - apps/web
    - services/api
    - packages/sdk
```

That lets ota understand the repo boundary and the member layout without guessing.

### `execution`

Use `execution` to describe where ota should run those tasks when native execution is not enough.

For the exact shell model ota uses to run commands, see [shell-semantics.md](shell-semantics.md).
For the platform support boundary, see [support-policy.md](support-policy.md).

Why users need it:

- tells ota whether tasks should run on the host, in a container, or through a remote provider
- makes the execution choice explicit instead of hiding it in scripts or CI glue
- helps receipts explain why a task ran the way it did

Supported backend values today:

- `native`
- `container`
- `remote`

Use `native` when you want the task to run on the host machine with the tools already installed there.
Use `container` when you want a fixed toolchain in an OCI-compatible container.
Use `remote` when execution should happen on another machine or workspace through a provider.

Use cases:

- use `native` when the repo depends on tools already installed on the developer machine
- use `container` when you want the same toolchain in local development and CI
- use `remote` when the work needs a separate machine or workspace

Container execution requires:

- `execution.backends.container.image`
- `execution.backends.container.engines` can list supported OCI engine CLIs in preference order; when omitted, ota falls back to `docker`
- at least one supported container engine CLI installed and running

Remote execution requires:

- `execution.backends.remote.provider`
- `execution.backends.remote.target`
- optional `execution.backends.remote.cwd`

Current shipped behavior:

- `ota run` supports native, container, and shipped remote providers
- `ota up` can use the same backend path for `setup`
- `ota clean` removes persistent container state when container lifecycle is persistent
- remote cleanup is not implemented yet

### `extensions`

Use `extensions` for adapter contract data. Each entry is a typed adapter descriptor with `kind`,
`command`, and `api_version`, plus optional `description` and `config`. Supported kinds today are
`checker` and `publisher`. `checker` is runnable with `ota extensions --run <name>` when
`api_version: 1` is declared. `publisher` is runnable with `ota extensions --publish <name>` when
`api_version: 1` is declared. The validator requires `kind` to be one of the supported kinds,
`command` to be non-empty, and `api_version` to be greater than zero.

Real-world uses include:

- uploading a release artifact bundle to an internal endpoint
- publishing scan or compliance reports through one standard adapter
- exposing a custom checker, codegen helper, or sync tool in a stable contract slot

Example:

```yaml
extensions:
  release-upload:
    kind: publisher
    command: ota-ext-upload
    api_version: 1
    description: Upload the release bundle to the artifact endpoint
    config:
      endpoint: https://artifacts.example.com/upload
      artifact: dist/release.zip
```

Use `ota extensions` to inspect the contract data. Use `ota extensions --run <name>` for
`checker` descriptors and `ota extensions --publish <name>` for `publisher` descriptors.

### `agent`

Use `agent` to tell ota which tasks are safe for agents, which paths are writable, which paths are
protected, and what repo-specific guidance applies. Protected paths are enforced by `ota detect --merge` and `ota detect --rewrite`.

Why users need it:

- makes repo-edit boundaries explicit for humans and agents
- tells ota which tasks are safe to run automatically
- keeps dangerous files protected from accidental rewrite flows

### `metadata`

Use `metadata` for extra repo-specific values that do not need their own first-class field yet.
Keep it open so teams can carry ownership, provenance, or rollout metadata alongside the contract.

Why users need it:

- lets repos attach custom values without inventing a parallel config file
- keeps repo-specific context close to the contract it describes
- gives tools a predictable place to read extra metadata when they need it

Example:

```yaml
metadata:
  team: platform
  owner: ota
  created_at: 2026-03-23
```

### `workspace`

Use `workspace` only for monorepo root/member orchestration across multiple repos.

Why users need it:

- lets ota understand which repos belong together
- makes cross-repo bootstrapping and dependency ordering explicit
- keeps root and member responsibility separate

## Good starting point

Start minimal, then expand:

1. define `project`
2. add required `runtimes`
3. add real `tools`, `env`, and `services`
4. add `checks`
5. add `tasks`
6. add `execution`, `agent`, `extensions`, and `workspace` only when they are actually needed

## Canonical reference

This page is the public contract reference on the site. It explains the shipped
contract shape, practical examples, and how to use it without sending readers
back to the spec for basic understanding.
