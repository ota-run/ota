# `ota`

<div align="center">
  <img src="docs/assets/ota-icon.svg" alt="Ota Logo" width="100" height="100">
</div>

Open repo readiness for humans and agents.

Ota is an open source readiness contract and CLI for modern repositories. It gives a repo one place to define what it needs, how it becomes ready, how tasks run, and how humans and agents can operate against the same truth.

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
ota up
ota detect --dry-run
ota detect
```

Current behavior:

- `ota validate` parses and semantically validates `ota.yaml`
- `ota tasks` lists validated tasks
- `ota run <task>` resolves dependencies and executes tasks deterministically
- `ota doctor` reports readiness findings with severity, explanation, and next action
- `ota up` validates, runs blocking preconditions, runs `setup` if present, and re-checks readiness
- `ota detect --dry-run` infers a candidate contract and prints provenance and confidence
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

The current repository has a real working core for the first V1 path:

- contract validation
- task listing
- deterministic task execution
- readiness diagnosis
- onboarding via `up`
- detection with dry-run and conservative write mode

Implementation status and sequencing live in:

- [docs/v1-phases.md](/Users/bobai/Workspace/Ota.run/ota/docs/v1-phases.md)
- [ROADMAP.md](/Users/bobai/Workspace/Ota.run/ota/ROADMAP.md)
- [docs/detect-write-gate.md](/Users/bobai/Workspace/Ota.run/ota/docs/detect-write-gate.md)

## Quickstart

Use an existing contract:

```bash
cargo run -- validate
cargo run -- doctor
cargo run -- up
```

Infer a starting contract from an existing repo:

```bash
cargo run -- detect --dry-run /path/to/repo
```

Write a conservative first contract:

```bash
cargo run -- detect /path/to/repo
```

More detail:

- [docs/quickstart.md](/Users/bobai/Workspace/Ota.run/ota/docs/quickstart.md)
- [docs/command-reference.md](/Users/bobai/Workspace/Ota.run/ota/docs/command-reference.md)
- [docs/contract-reference.md](/Users/bobai/Workspace/Ota.run/ota/docs/contract-reference.md)
- [docs/philosophy.md](/Users/bobai/Workspace/Ota.run/ota/docs/philosophy.md)
- [CONTRIBUTING.md](/Users/bobai/Workspace/Ota.run/ota/CONTRIBUTING.md)
- [examples/basic-node/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/basic-node/ota.yaml)
- [examples/basic-python/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/basic-python/ota.yaml)
- [examples/basic-go/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/basic-go/ota.yaml)
- [examples/mixed-node-python/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/mixed-node-python/ota.yaml)
- [examples/fullstack-node-go/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/fullstack-node-go/ota.yaml)
- [examples/full-contract/ota.yaml](/Users/bobai/Workspace/Ota.run/ota/examples/full-contract/ota.yaml)
