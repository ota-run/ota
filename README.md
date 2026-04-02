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

<div align="center">
  <img src="docs/assets/ota-icon.svg" alt="Ota Logo" width="110" height="110" />
</div>

# `ota`

Ota is open infrastructure for repo readiness, not another task runner or package manager. It gives every repo one explicit contract for what it needs, how it is diagnosed, how it is prepared, and how tasks run, so humans and AI agents can answer why a repo is or is not runnable without guesswork.

Doctor first, contract second.

## Installation

Install the latest release binary:

```bash
curl -fsSL https://dist.ota.run/install.sh | sh
```

Windows PowerShell:

```powershell
iwr https://dist.ota.run/install.ps1 | iex
```

Pin a release:

```bash
OTA_VERSION=vX.Y.Z curl -fsSL https://dist.ota.run/install.sh | sh
```

Windows PowerShell:

```powershell
$env:OTA_VERSION = "vX.Y.Z"
iwr https://dist.ota.run/install.ps1 | iex
```

Update an existing install:

```bash
ota upgrade
```

Install from a local checkout:

```bash
./scripts/install.sh --from-source
```

Windows PowerShell:

```powershell
powershell -ExecutionPolicy Bypass -File .\scripts\install.ps1 -FromSource
```

See [docs/installation.md](docs/installation.md) for mirror/CDN overrides and source fallback details.

## Quickstart

Use an existing contract:

```bash
ota doctor
ota validate
ota tasks --json
ota up
```

If the contract declares agent guidance, `ota tasks --json` and `ota doctor --json` surface the
same safe-task, verification, and writable-path hints that humans can review in `ota.yaml`.

Infer a starting contract from an existing repo:

```bash
ota doctor
ota detect --dry-run .
```

Write a conservative first contract:

```bash
ota detect --write .
```

Review or conservatively merge into an existing contract:

```bash
ota detect --merge --dry-run .
ota detect --merge .
```

Example contracts:
- [basic-node](examples/basic-node/ota.yaml) - Node / TypeScript starter
- [basic-dotnet](examples/basic-dotnet/ota.yaml) - C# / .NET starter
- [basic-java](examples/basic-java/ota.yaml) - Maven starter
- [basic-rust](examples/basic-rust/ota.yaml) - Cargo starter
- [basic-script](examples/basic-script/ota.yaml) - Script-only starter
- [basic-services](examples/basic-services/ota.yaml) - Service-backed repo starter
- [full-contract](examples/full-contract/ota.yaml) - Full contract surface
- [workspace-acquire](examples/workspace-acquire/ota.workspace.yaml) - Workspace acquisition flow

## Why Ota exists

Repo setup truth is usually fragmented across:

- language manifests
- runtime version files
- task scripts
- container config
- CI config
- README setup notes
- tribal knowledge

Ota consolidates that into one canonical contract:

```yaml
ota.yaml
```

The goal is not hidden automation. The goal is deterministic, inspectable repo readiness.

## What Ota does today

Current commands:

```bash
ota validate
ota tasks
ota run <task>
ota diff
ota explain
ota doctor
ota init
ota agents
ota check
ota up
ota detect --dry-run
ota detect --write
ota detect --merge --dry-run
ota detect --merge
ota workspace validate
ota workspace tasks
ota workspace run <task>
ota workspace explain
ota workspace check
ota workspace doctor
ota workspace up
ota workspace refresh
ota run bump-version --version x.y.z
```

Global flag:

```bash
ota --debug <command>
ota --plain <command>
```

`--debug` emits command-phase tracing to stderr without changing normal stdout.
`--plain` emits ASCII-first output without emoji, icons, or ANSI color.

Use `--debug` when you want command traces for `ota up`, `ota run <task>`, `ota workspace up`,
`ota workspace refresh`, `ota workspace run <task>`, `ota doctor`, `ota detect`, `ota diff`, and
`ota explain`.
Commands like `ota validate`, `ota tasks`, `ota workspace validate`, `ota workspace tasks`, and
`ota workspace list` should usually stay quiet unless you are actively debugging.

Current behavior:

- `ota validate` parses and semantically validates `ota.yaml`
- `ota tasks` lists validated tasks and their execution form
- `ota run <task>` resolves dependencies and executes `run` or `script` tasks deterministically
- `ota diff` compares two contracts semantically and reports added, missing, and changed fields in deterministic order
- `ota explain` turns readiness findings into an ordered remediation plan
- `ota doctor` reports readiness findings for env, runtimes, tools, services, and checks with severity, explanation, and next action, still gives a useful repo/host diagnosis when no `ota.yaml` exists yet, and leads with the highest-priority blocker first
- `ota init` creates a starter contract for repos that do not yet have `ota.yaml`
- `ota agents` exports or syncs a repo-local `AGENTS.md` from the contract’s agent guidance, preserves existing user-authored content by appending an Ota-managed block, skips the write when the generated content is already present, and shows a `Managed block:` label in text output so the Ota-owned section is explicit, including the `ota run ...` command form for each listed task
- `ota check` runs configured checks without runtime, tool, env, or task execution
- `ota up` validates, runs blocking preconditions, starts required services in declared dependency order, uses required service healthchecks as readiness gates, runs `setup` if present, and re-checks readiness
- `ota detect` (default) infers a candidate contract and prints provenance/confidence without writing
- `ota detect --write` writes a contract conservatively from `high` confidence fields only
- `ota detect --merge --dry-run` compares detected repo signals against an existing `ota.yaml` without writing and surfaces stale contract fields that no longer match repo reality
- `ota detect --merge` applies only additive `high` confidence missing fields to an existing `ota.yaml`
- there is no standalone `ota drift` command yet; drift review stays on `ota detect --merge --dry-run` and trust/readiness drift stays on `ota doctor`
- `ota workspace validate` validates `ota.workspace.yaml` separately from repo contracts
- `ota workspace tasks` lists workspace repo tasks in dependency order without executing them
- `ota workspace run <task>` executes one task across workspace repos in dependency order with deterministic reporting
- `ota workspace explain` turns workspace readiness findings into ordered remediation steps
- `ota workspace check` runs configured checks across workspace repos with deterministic reporting
- `ota workspace doctor` aggregates repo readiness across a workspace contract without merging repo and workspace truth, including repos that are not acquired yet
- `ota workspace up` can acquire missing repos from git sources and then orchestrates repo-level `up` across the workspace contract without inventing a second bootstrap model
- `ota workspace refresh` re-syncs repos that already exist locally without cloning missing ones
- `ota workspace refresh --force` hard-resets refreshed repos to the declared source or `--ref` override
- `ota workspace refresh --prune` drops stale remote-tracking refs during refresh
- `ota workspace refresh --ref <branch|tag|sha>` overrides the source ref used for refresh
- editor and CI consumers should prefer `--json` surfaces such as `ota doctor --json`, `ota workspace doctor --json`, `ota workspace list --json`, and `ota up --json` instead of scraping text output

## Execution Modes and Provisioning

Ota supports three execution backends for task-oriented commands:

- `native` runs tasks on the host machine.
- `container` runs tasks in an OCI-compatible container using the image defined by the repo contract.
- `remote` runs tasks on a separate machine or workspace through a remote provider.

### Why the three modes exist

- `native` is the simplest path when the host already has the right toolchain and you want to debug against the real machine.
- `container` is the reproducible path when you want a fixed toolchain, deterministic setup, and CI-like behavior.
- `remote` is the off-host path when execution needs to happen somewhere else entirely, such as a dev box, cluster, or managed workspace.

### What Ota does today

- `ota run` and `ota up` can execute through the configured backend path.
- `ota up` can run the `setup` task through the same backend selection as `ota run`.
- `ota doctor` checks the prerequisites for the preferred backend and reports missing tools or suspicious remote target shape early.
- `ota clean` can remove persistent container state for container-backed repos.

### What Ota does not do today

- Ota does not automatically install every missing host tool or language runtime.
- Ota does not turn a laptop into a fully managed workstation.
- Ota does not invent remote provisioning or remote workspace selection beyond the configured provider path.

### Container backend

Container execution is useful when you want Ota to run repo tasks in a known environment instead of relying on whatever happens to be installed locally.

Benefits:

- removes drift in Java, Maven, shell, and other repo tool versions
- makes local execution closer to CI
- gives agents a stable execution surface
- supports persistent or ephemeral lifecycle behavior where the contract and command support it

Requirements:

- at least one supported container engine CLI must be installed and running
- the contract must declare `execution.backends.container.image`
- the image must be pullable and runnable by the selected engine

In this repository, the container image is:

```yaml
execution:
  preferred: container
  lifecycle: persistent
  supported:
    - native
    - container
  backends:
    container:
      image: rust:1.85
```

### Native backend

Native execution is useful when:

- you want to debug against the exact host environment
- the repo already has the required toolchain installed
- you want the fewest moving parts and no container boundary

Native execution does not require a container engine, but it does require the host tools that the task depends on.

### Remote backend

Remote execution is useful when the work should happen outside the local machine:

- `ssh` for a team dev box or dedicated host
- `kubectl` for execution inside a Kubernetes-backed environment
- `tsh` for Teleport-managed infrastructure
- `daytona` for a managed remote development workspace

Remote execution is only available when the contract declares the provider and target fields required by the backend.

### Which commands use the backend

These commands execute repo tasks and therefore respect the backend selection:

- `ota run <task>`
- `ota up`

These commands do not run repo tasks and therefore do not use the execution backend:

- `ota validate`
- `ota doctor`
- `ota detect`
- `ota init`
- `ota tasks`

### Override syntax

Use `--backend` to force one invocation to use a specific backend:

```bash
ota run test --backend native
ota run test --backend container
ota up --backend native
ota up --backend container
```

Use `--lifecycle` when you need to override container reuse for one invocation:

```bash
ota run test --backend container --lifecycle persistent
ota run test --backend container --lifecycle ephemeral
```

Use `ota tasks --use` to see the exact runnable task commands for the current contract:

```bash
ota tasks --use
```

## Hosted validation and service provisioning

In CI, the runner still owns the job. Ota owns the repo contract and can provision declared
services such as Postgres through `ota up`, so the workflow stays thin and the service intent lives
with the repo instead of being duplicated in pipeline YAML.

That is different from host provisioning. Ota can provision declared services and run tasks in a
container or remote backend, but it does not replace the OS package manager, language installer,
or workstation bootstrap process.

```yaml
name: ci

on:
  push:
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4
      - name: Install Ota
        run: curl -fsSL https://dist.ota.run/install.sh | sh
      - name: Validate contract
        run: ota validate
      - name: Prepare repo
        run: ota up
      - name: Run lint
        run: ota run lint
      - name: Run tests
        run: ota run test
```

## Detect trust model

Ota treats detection as trust-sensitive.

- `ota detect --dry-run` is the review path
- `ota detect --merge --dry-run` is the review path for existing contracts
- every inferred field includes provenance
- every inferred field includes confidence
- write mode uses only `high` confidence fields
- write mode validates before writing
- write mode refuses to overwrite an existing `ota.yaml`
- merge write is additive only in the current implementation
- lower-confidence or conflicting changes stay review-only

This is intentional. The project prefers conservative correctness over aggressive generation.

## Open standard intent

Ota is being built as open infrastructure, not as a vendor-specific workflow.

The long-term aim is:

- one canonical readiness contract per repo
- one canonical bootstrap contract per workspace
- deterministic behavior without LLM dependency in the core path
- human and agent symmetry
- interoperability with the existing tool ecosystem

## Current status

V1 is complete and the release gate is green.

The current shipped foundation includes:

- contract validation
- task listing
- deterministic task execution
- readiness diagnosis
- onboarding via `up`
- detection with dry-run, conservative first write, and conservative additive merge
- separate workspace contract validation, diagnosis, and bootstrap
- generic git-based workspace acquisition for missing repos
- monorepo root/member loading for repo commands via `--member`
- fixture-backed coverage for Java detection, container-heavy, container-only, conflict-heavy Node, mixed Node/Python, legacy Python, and ugly/polyglot mixed-reality repo shapes

Current planning state:

- V1 archive: [docs/planning/v1/phases.md](docs/planning/v1/phases.md)
- V1 release gate: [docs/planning/v1/release-gate.md](docs/planning/v1/release-gate.md)
- V2 archive: [docs/planning/v2/plan.md](docs/planning/v2/plan.md)
- V2.1 archive: [docs/planning/v2.1/plan.md](docs/planning/v2.1/plan.md)
- V6 archive: [docs/planning/v6/plan.md](docs/planning/v6/plan.md)
- Active version: [docs/planning/v7/plan.md](docs/planning/v7/plan.md)
- Archived local UX hardening slice: [docs/planning/v5-ux-hardening.md](docs/planning/v5-ux-hardening.md)
- V5 mutation controls and caching: [docs/spec/mutation-controls-and-caching.md](docs/spec/mutation-controls-and-caching.md)

## Contribution policy

Ota does not accept external code contributions. See [docs/policy/commercial-policy.md](docs/policy/commercial-policy.md) for the open-core and enterprise boundary.

Use the GitHub issue templates for bug reports, feature requests, and docs feedback.

See [docs/policy/support-and-enterprise.md](docs/policy/support-and-enterprise.md) for the current support and enterprise boundary.

## Documentation

### Start here
- [Command reference](docs/spec/command-reference.md)
- [Contract reference](docs/spec/contract-reference.md)
- [Workspace reference](docs/spec/workspace-reference.md)
- [Shell semantics](docs/spec/shell-semantics.md)
- [Service behavior](docs/spec/service-behavior.md)
- [JSON output reference](docs/spec/json-output-reference.md)
- [Audit and provenance](docs/spec/audit-and-provenance.md)
- [Policy packs](docs/spec/policy-packs.md)
- [Exit codes](docs/spec/exit-codes.md)
- [Docs clarity spec](docs/spec/docs-clarity-spec.md)

### Core concepts
- [Philosophy](docs/philosophy.md)
- [Compatibility policy](docs/spec/compatibility-policy.md)
- [Support policy](docs/spec/support-policy.md)

### Design and engineering
- [Security posture](docs/design/security-posture.md)
- [Performance budget](docs/design/performance-budget.md)
- [Doctor quality bar](docs/design/doctor-quality-bar.md)
- [Detect write gate](docs/design/detect-write-gate.md)

### Planning and roadmap
- [V1 phases](docs/planning/v1/phases.md)
- [V1 release gate](docs/planning/v1/release-gate.md)
- [V2 plan](docs/planning/v2/plan.md)
- [V2.1 plan](docs/planning/v2.1/plan.md)
- [V3 plan](docs/planning/v3/plan.md)
- [V4 plan](docs/planning/v4/plan.md)
- [V5 plan](docs/planning/v5/plan.md)
- [V6 plan](docs/planning/v6/plan.md)
- [V7 plan](docs/planning/v7/plan.md)
- [V8 plan](docs/planning/v8/plan.md)
- [V9 plan](docs/planning/v9/plan.md)
- [V5 UX hardening completion slice](docs/planning/v5-ux-hardening.md)
- [Hosted validation workflow](docs/spec/hosted-validation-workflow.md)
- [Fixture repo plan](docs/planning/fixture-repo-plan.md)
- [Roadmap](ROADMAP.md)

### Contributing
- [Contributing guide](CONTRIBUTING.md)

## Examples

### Minimal contracts
- [Basic Node](examples/basic-node/ota.yaml) - Node / TypeScript starter
- [Basic .NET](examples/basic-dotnet/ota.yaml) - C# / .NET starter
- [Basic Java](examples/basic-java/ota.yaml) - Maven starter
- [Basic Python](examples/basic-python/ota.yaml) - Python starter
- [Basic Go](examples/basic-go/ota.yaml) - Go module starter
- [Basic Rust](examples/basic-rust/ota.yaml) - Cargo starter
- [Basic Script](examples/basic-script/ota.yaml) - Script-only starter

### Mixed and realistic repos
- [Mixed Node + Python](examples/mixed-node-python/ota.yaml) - Polyglot app example
- [Fullstack Node + Go](examples/fullstack-node-go/ota.yaml) - Frontend/backend split example
- [Full contract example](examples/full-contract/ota.yaml) - Exhaustive contract reference

### Workspace
- [Basic Workspace](examples/workspace-basic/ota.workspace.yaml) - Multi-repo starter; use `ota workspace doctor` to review readiness and `ota workspace up` to prepare the stack.
- [Acquisition Workspace](examples/workspace-acquire/ota.workspace.yaml) - Workspace acquisition flow; use `ota workspace init` first, then `ota workspace up` to acquire and prepare repos.
