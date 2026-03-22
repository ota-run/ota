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

Current behavior:

- `ota validate` parses and semantically validates `ota.yaml`
- `ota tasks` lists validated tasks and their execution form
- `ota run <task>` resolves dependencies and executes `run` or `script` tasks deterministically
- `ota doctor` reports readiness findings for env, runtimes, tools, services, and checks with severity, explanation, and next action
- `ota init` creates a starter contract for repos that do not yet have `ota.yaml`
- `ota check` runs configured checks without runtime, tool, env, or task execution
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
- fixture-backed coverage for Java, Docker-heavy, and ugly mixed-reality repo shapes

Implementation status and sequencing live in:

- [docs/planning/v1-phases.md](docs/planning/v1-phases.md)
- [ROADMAP.md](ROADMAP.md)
- [docs/design/detect-write-gate.md](docs/design/detect-write-gate.md)

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

More detail:

- [docs/guides/quickstart.md](docs/guides/quickstart.md)
- [docs/spec/command-reference.md](docs/spec/command-reference.md)
- [docs/spec/contract-reference.md](docs/spec/contract-reference.md)
- [docs/spec/compatibility-policy.md](docs/spec/compatibility-policy.md)
- [docs/spec/support-policy.md](docs/spec/support-policy.md)
- [docs/design/security-posture.md](docs/design/security-posture.md)
- [docs/design/performance-budget.md](docs/design/performance-budget.md)
- [docs/planning/fixture-repo-plan.md](docs/planning/fixture-repo-plan.md)
- [docs/design/doctor-quality-bar.md](docs/design/doctor-quality-bar.md)
- [docs/guides/philosophy.md](docs/guides/philosophy.md)
- [CONTRIBUTING.md](CONTRIBUTING.md)
- [examples/basic-node/ota.yaml](examples/basic-node/ota.yaml)
- [examples/basic-python/ota.yaml](examples/basic-python/ota.yaml)
- [examples/basic-go/ota.yaml](examples/basic-go/ota.yaml)
- [examples/mixed-node-python/ota.yaml](examples/mixed-node-python/ota.yaml)
- [examples/fullstack-node-go/ota.yaml](examples/fullstack-node-go/ota.yaml)
- [examples/full-contract/ota.yaml](examples/full-contract/ota.yaml)
