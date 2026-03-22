# Ota Philosophy

Ota is being built as open infrastructure for repo readiness.

The project is guided by a small set of principles:

## One canonical contract

A repo should have one explicit readiness contract.

Ota exists to consolidate scattered setup truth into `ota.yaml`, not to create another hidden layer beside the repo.

## Determinism before intelligence

Core commands should be deterministic, inspectable, and safe without any model dependency.

Inference is useful, but it is downstream of the contract, not a replacement for it.

## Trust is the product

Commands like `validate`, `run`, `doctor`, `up`, and `detect` only matter if users trust them.

That means:

- stable command semantics
- explicit errors
- conservative write behavior
- provenance for inferred fields
- confidence instead of false certainty

## Humans and agents should use the same truth

Ota is not meant to create one path for humans and a different hidden path for automation.

The same contract should serve:

- maintainers
- contributors
- CI
- local tooling
- agents

## Adoption before elegance

Ota should earn its place on a repo by solving a real problem immediately.

The first value is not abstract standardization.

The first value is helping someone understand why a repo is not ready and what to do next.

## Conservative generation

Generated configuration is trust-sensitive.

Ota prefers:

- reviewable dry-runs
- explicit write behavior
- provenance for inferred fields
- confidence instead of false certainty
- refusal to overwrite existing contracts silently

It does not try to look smart by writing ambiguous configuration.

## Open infrastructure, not a closed workflow

Ota is being shaped as an open standard and CLI, not as a vendor-specific control plane.

The project should stay legible, portable, and interoperable with the tooling ecosystems that repos already use.

## Runtime discipline

Ota should stay lightweight and resource-conscious by default.

That means:

- prefer narrow targeted reads over broad repo scans
- prefer simple stateless commands over background daemons
- keep caching explicit, small, and easy to invalidate
- use bounded concurrency when concurrency is introduced
- make large repos degrade gracefully instead of exploding process or memory usage
- avoid hidden long-lived runtime state in v1
- keep process spawning, log capture, and detection behavior conservative by default

## Small surface, strong guarantees

The goal is not a large command catalog.

The goal is a small set of commands that are dependable on real repositories:

- `ota init`
- `ota validate`
- `ota run`
- `ota doctor`
- `ota up`
- `ota detect`

If those commands are trustworthy, Ota can become useful infrastructure.
