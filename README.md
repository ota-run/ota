<div align="center">
  <img src="docs/assets/ota-icon.svg" alt="Ota Logo" width="100" height="100">
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

## Detect trust model

Ota treats detection as trust-sensitive.

- `ota detect --dry-run` is the review path
- every inferred field includes provenance
- every inferred field includes confidence
- write mode uses only `high` confidence fields
- write mode validates before writing
- write mode refuses to overwrite an existing `ota.yaml`

This is intentional. The project prefers conservative correctness over aggressive generation.

## Open standard intent

Ota is being built as open infrastructure, not as a vendor-specific workflow.

The long-term aim is:

- one canonical readiness contract per repo
- deterministic behavior without LLM dependency in the core path
- human and agent symmetry
- interoperability with the existing tool ecosystem

## Current status

The repository already contains a working core for the v1 path:

- contract validation
- task listing
- deterministic task execution
- readiness diagnosis
- onboarding via `up`
- detection with dry-run and conservative write mode
- fixture-backed coverage for Java detection, Docker-heavy, Docker-only, conflict-heavy Node, and ugly/polyglot mixed-reality repo shapes

## Quickstart

Use an existing contract:

```bash
cargo run -- validate
cargo run -- doctor
cargo run -- up
```

Infer a starting contract from an existing repo:

```bash
cargo run -- init
cargo run -- detect --dry-run /path/to/repo
```

Write a conservative first contract:

```bash
cargo run -- detect /path/to/repo
```

Example contracts:
- [basic-node](/Users/bobai/Workspace/Ota.run/ota/examples/basic-node/ota.yaml)
- [basic-script](/Users/bobai/Workspace/Ota.run/ota/examples/basic-script/ota.yaml)
- [basic-services](/Users/bobai/Workspace/Ota.run/ota/examples/basic-services/ota.yaml)

## Documentation

### Start here
- [Quickstart](docs/quickstart.md)
- [Command reference](docs/spec/command-reference.md)
- [Contract reference](docs/spec/contract-reference.md)
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
- [V1 phases](docs/planning/v1-phases.md)
- [Fixture repo plan](docs/planning/fixture-repo-plan.md)
- [Roadmap](ROADMAP.md)

### Contributing
- [Contributing guide](CONTRIBUTING.md)

## Examples

### Minimal contracts
- [Basic Node](examples/basic-node/ota.yaml)
- [Basic Python](examples/basic-python/ota.yaml)
- [Basic Go](examples/basic-go/ota.yaml)
- [Basic Script](examples/basic-script/ota.yaml)

### Mixed and realistic repos
- [Mixed Node + Python](examples/mixed-node-python/ota.yaml)
- [Fullstack Node + Go](examples/fullstack-node-go/ota.yaml)
- [Full contract example](examples/full-contract/ota.yaml)
