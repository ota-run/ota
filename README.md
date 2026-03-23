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

Ota is a readiness contract and CLI for modern repositories. It gives every repo one source of truth for what it needs, how it becomes ready, how tasks run, and how humans and AI agents operate. Run any repo without manual setup guesswork.

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
ota doctor
ota init
ota check
ota up
ota detect --dry-run
ota detect
ota detect --merge --dry-run
ota detect --merge
ota workspace validate
ota workspace doctor
ota workspace up
```

Global flag:

```bash
ota --debug <command>
```

`--debug` emits command-phase tracing to stderr without changing normal stdout.

Current behavior:

- `ota validate` parses and semantically validates `ota.yaml`
- `ota tasks` lists validated tasks and their execution form
- `ota run <task>` resolves dependencies and executes `run` or `script` tasks deterministically
- `ota doctor` reports readiness findings for env, runtimes, tools, services, and checks with severity, explanation, and next action
- `ota init` creates a starter contract for repos that do not yet have `ota.yaml`
- `ota check` runs configured checks without runtime, tool, env, or task execution
- `ota up` validates, runs blocking preconditions, starts required services in declared dependency order, uses required service healthchecks as readiness gates, runs `setup` if present, and re-checks readiness
- `ota detect --dry-run` infers a candidate contract from repo signals such as package manifests, runtime files, Java build wrappers, build files, and Docker Compose service declarations, then prints provenance and confidence
- `ota detect` writes a contract conservatively from `high` confidence fields only
- `ota detect --merge --dry-run` compares detected repo signals against an existing `ota.yaml` without writing
- `ota detect --merge` applies only additive `high` confidence missing fields to an existing `ota.yaml`
- `ota workspace validate` validates `ota.workspace.yaml` separately from repo contracts
- `ota workspace doctor` aggregates repo readiness across a workspace contract without merging repo and workspace truth, including repos that are not acquired yet
- `ota workspace up` can acquire missing repos from git sources and then orchestrates repo-level `up` across the workspace contract without inventing a second bootstrap model

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
- fixture-backed coverage for Java detection, Docker-heavy, Docker-only, conflict-heavy Node, mixed Node/Python, legacy Python, and ugly/polyglot mixed-reality repo shapes

Current planning state:

- V1 archive: [docs/planning/v1/phases.md](docs/planning/v1/phases.md)
- V1 release gate: [docs/planning/v1/release-gate.md](docs/planning/v1/release-gate.md)
- V2 archive: [docs/planning/v2/plan.md](docs/planning/v2/plan.md)
- V2.1 archive: [docs/planning/v2.1/plan.md](docs/planning/v2.1/plan.md)
- Active version: [docs/planning/v3/plan.md](docs/planning/v3/plan.md)

## Quickstart

Use an existing contract:

```bash
cargo run -- validate
cargo run -- tasks --json
cargo run -- doctor
cargo run -- up
```

If the contract declares agent guidance, `ota tasks --json` and `ota doctor --json` surface the
same safe-task, verification, and writable-path hints that humans can review in `ota.yaml`.

Infer a starting contract from an existing repo:

```bash
cargo run -- init
cargo run -- detect --dry-run /path/to/repo
```

Write a conservative first contract:

```bash
cargo run -- detect /path/to/repo
```

Review or conservatively merge into an existing contract:

```bash
cargo run -- detect --merge --dry-run /path/to/repo
cargo run -- detect --merge /path/to/repo
```

Example contracts:
- [basic-node](/Users/bobai/Workspace/Ota.run/ota/examples/basic-node/ota.yaml)
- [basic-java](/Users/bobai/Workspace/Ota.run/ota/examples/basic-java/ota.yaml)
- [basic-rust](/Users/bobai/Workspace/Ota.run/ota/examples/basic-rust/ota.yaml)
- [basic-script](/Users/bobai/Workspace/Ota.run/ota/examples/basic-script/ota.yaml)
- [basic-services](/Users/bobai/Workspace/Ota.run/ota/examples/basic-services/ota.yaml)
- [full-contract](/Users/bobai/Workspace/Ota.run/ota/examples/full-contract/ota.yaml)
- [workspace-acquire](/Users/bobai/Workspace/Ota.run/ota/examples/workspace-acquire/ota.workspace.yaml)

## Documentation

### Start here
- [Quickstart](docs/quickstart.md)
- [Command reference](docs/spec/command-reference.md)
- [Contract reference](docs/spec/contract-reference.md)
- [Workspace reference](docs/spec/workspace-reference.md)
- [Service behavior](docs/spec/service-behavior.md)
- [Shell semantics](docs/spec/shell-semantics.md)
- [JSON output reference](docs/spec/json-output-reference.md)
- [Exit codes](docs/spec/exit-codes.md)

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
- [Fixture repo plan](docs/planning/fixture-repo-plan.md)
- [Roadmap](ROADMAP.md)

### Contributing
- [Contributing guide](CONTRIBUTING.md)

## Examples

### Minimal contracts
- [Basic Node](examples/basic-node/ota.yaml)
- [Basic Java](examples/basic-java/ota.yaml)
- [Basic Python](examples/basic-python/ota.yaml)
- [Basic Go](examples/basic-go/ota.yaml)
- [Basic Rust](examples/basic-rust/ota.yaml)
- [Basic Script](examples/basic-script/ota.yaml)

### Mixed and realistic repos
- [Mixed Node + Python](examples/mixed-node-python/ota.yaml)
- [Fullstack Node + Go](examples/fullstack-node-go/ota.yaml)
- [Full contract example](examples/full-contract/ota.yaml)

### Workspace
- [Basic Workspace](examples/workspace-basic/ota.workspace.yaml)
- [Acquisition Workspace](examples/workspace-acquire/ota.workspace.yaml)
