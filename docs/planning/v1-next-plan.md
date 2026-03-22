# V1 Next Plan

This document turns the next four high-value tracks into an execution plan.

The intent is simple:

- close the remaining V1 product gaps without widening scope carelessly
- use real repos to pressure-test behavior before adding more abstraction
- keep implementation honest to the consolidated spec

## Order

Execute these tracks in this order:

1. Fixture repos
2. Services behavior
3. `execution.lifecycle`
4. Exit codes and debug mode

This order is deliberate. Real fixtures should pressure-test the product before deeper behavior work lands.

## Track 1: Fixture Repos

Status: complete

### Goal

Stop designing against idealized examples and start validating against more realistic repository shapes.

### Deliverables

- add a Java fixture repo
- add a Docker-heavy fixture repo
- add one ugly real-world shaped fixture repo
- extend fixture assertions for:
  - `ota doctor`
  - `ota init`
  - `ota detect`

### Rules

- fixtures should be small but realistic
- each fixture should pressure at least one trust-sensitive behavior
- avoid adding sources we do not actually support just to make fixtures look broad

### Exit Criteria

- the repo has canonical fixture coverage for Node, Python, Go, Java, Docker-heavy, and ugly mixed-reality shapes
- `doctor`, `init`, and `detect` are exercised against those fixtures
- fixture failures explain real product gaps instead of test-only assumptions

## Track 2: Services Behavior

Status: in progress

### Goal

Make the `services` section useful in V1 without turning Ota into an orchestrator.

### Scope

#### Phase 2a: Validation hardening

- keep `services` in the accepted contract
- tighten validation for empty service fields and invalid shapes

#### Phase 2b: Diagnosis awareness

- teach `ota doctor` to reason about declared services
- if a required service has a `healthcheck`, run it and report the result clearly
- if a required service is declared without enough executable information, report that clearly

#### Phase 2c: `up` integration

- allow `ota up` to run explicit service `start` commands before setup when the contract declares them
- re-check service health after setup
- keep behavior explicit and contract-driven

### Non-goals

- deep orchestration
- background service supervisors
- container abstraction layers
- provider-specific adapters beyond explicit shell commands

### Exit Criteria

- `services` are no longer passive schema baggage
- `doctor` can explain service-related blockers
- `up` can use explicit service start commands when the contract provides them
- no hidden long-lived service state is introduced

## Track 3: `execution.lifecycle`

### Goal

Define and implement an honest V1 meaning for `persistent` and `ephemeral`.

### Scope

#### Phase 3a: Policy and support matrix

- document current lifecycle semantics clearly
- define what Ota does and does not promise for `persistent` and `ephemeral`

#### Phase 3b: Validation and output

- validate allowed lifecycle values
- surface lifecycle in command output where it materially affects behavior

#### Phase 3c: Honest limited behavior

- `persistent`: current default model
- `ephemeral`: supported only where Ota can honor it honestly without pretending full isolation
- if a command cannot honor `ephemeral` meaningfully, it should say so clearly rather than fake support

### Non-goals

- full virtual-environment orchestration
- full temp-workspace cloning
- containerized isolation
- hidden cleanup daemons

### Exit Criteria

- lifecycle is no longer just parsed metadata
- users can tell what `persistent` and `ephemeral` mean in current Ota behavior
- unsupported lifecycle behavior fails or warns explicitly instead of implying false guarantees

## Track 4: Exit Codes and Debug Mode

### Goal

Make command behavior more operationally reliable for humans, CI, agents, and editor tooling.

### Scope

#### Phase 4a: Exit code table

- define per-command exit semantics
- keep child task exit propagation where appropriate
- use explicit codes only when they improve operability meaningfully

#### Phase 4b: Debug mode

- add a small `--debug` path
- emit command-phase tracing and resolution details to stderr
- keep normal output stable and uncluttered

#### Phase 4c: Docs and tests

- document exit code policy
- document debug-mode intent
- add tests for command exit semantics where behavior is contract-sensitive

### Non-goals

- verbose logging by default
- persistent log files
- telemetry systems

### Exit Criteria

- command exit semantics are documented and intentional
- `--debug` helps humans and agents understand what a command is doing
- normal command output remains concise and stable

## Execution Strategy

Use the same discipline across all four tracks:

- keep changes layer-correct
- prefer reuse over parallel paths
- validate on fixtures and contract-level tests
- do not hide expensive behavior behind convenience
- preserve the trust model first, polish second
