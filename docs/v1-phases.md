# Ota V1 Phases

This document keeps the V1 spec intact and breaks implementation into phases.

The rule is simple:

- V1 is the product contract.
- Phases are the execution plan.
- A later phase does not loosen or redefine an earlier phase.

## Why Phase V1

V1 is broad enough that building every command at once would increase surface area, weaken validation, and reduce trust in the core behavior.

The phased plan exists to:

- preserve the full V1 scope
- ship in a technically honest order
- lock deterministic behavior before adding inference or automation
- keep testing and output semantics stable as the surface grows

## Phase Order

1. V1a: Contract Core
2. V1b: Read Path
3. V1c: Execution Core
4. V1d: Diagnosis Core
5. V1e: Onboarding Path
6. V1f: Detection
7. V1g: Agent Surface and Polish

## Cross-Phase Rules

- Native execution is the only fully implemented backend in V1.
- Intelligence is optional and must never define runtime truth.
- Every phase must leave the CLI in a coherent, testable state.
- Output shape and exit behavior must be treated as contracts.
- Do not start the next phase until the current phase has fixtures and verification.

## V1a: Contract Core

### Goal

Establish the canonical contract model and make invalid config fail clearly.

### Scope

- define typed internal schema for V1 sections
- parse `ota.yaml`
- validate structure and semantic rules
- validate task references
- reject circular task dependencies
- implement `ota validate`

### Required V1 Sections

- `version`
- `project`
- `execution`
- `runtimes`
- `tools`
- `env`
- `tasks`
- `checks`
- `agent`
- `metadata`

### Exit Criteria

- invalid `ota.yaml` fails with actionable diagnostics
- valid `ota.yaml` parses into stable internal types
- task dependency cycles are rejected deterministically
- `ota validate` has stable human output and exit codes

## V1b: Read Path

### Goal

Prove the contract can be consumed safely before execution begins.

### Scope

- implement `ota tasks`
- list tasks and core task metadata
- add JSON output for `ota validate`
- add JSON output for `ota tasks`
- add fixtures and golden outputs for read-only commands

### Exit Criteria

- `ota tasks` is deterministic
- task listing reflects validated contract state, not ad hoc parsing
- JSON output is stable enough for downstream tooling

## V1c: Execution Core

### Goal

Run declared tasks deterministically from the contract.

### Scope

- implement `ota run <task>`
- resolve task dependencies
- honor explicit environment configuration
- support native execution only
- define clear process and exit semantics

### Exit Criteria

- `ota run` executes validated tasks only
- task dependency ordering is deterministic
- task failures surface cleanly and do not fail silently
- clean-machine execution behavior is covered by tests

## V1d: Diagnosis Core

### Goal

Make a broken repo understandable fast.

### Scope

- implement `ota doctor`
- validate contract before readiness checks
- check runtimes
- check tools
- check environment requirements
- run configured checks
- order findings by severity and blocking priority
- include what failed, why it matters, and next action
- add JSON output for `ota doctor`

### Exit Criteria

- the highest-priority blocking issue is reported first
- no blocking false positives on initial fixture repos
- every `error` and `warn` includes a next action
- output is clear enough for humans and machine-readable enough for tools

## V1e: Onboarding Path

### Goal

Provide the zero-knowledge command that takes a user from unknown state to clear readiness.

### Scope

- implement `ota up`
- validate config first
- run blocking precondition checks
- verify runtimes and tools required for setup
- run `setup` task if present
- re-run readiness validation after setup
- return explicit ready or not-ready result

### Exit Criteria

- `ota up` does not require prior task knowledge
- setup path is deterministic and explicit about mutations
- post-setup state is revalidated before success is reported

## V1f: Detection

### Goal

Generate a trustworthy starting contract from repo reality.

### Scope

- implement `ota detect --dry-run`
- infer from explicit repo signals first
- emit provenance per inferred field
- emit confidence per inferred field
- use only `high`, `medium`, and `low`
- add write mode only after dry-run trust is proven

### Initial Detection Sources

- `package.json`
- `.nvmrc`
- `.tool-versions`
- `pyproject.toml`
- `go.mod`
- `Dockerfile`
- `docker-compose.yml`
- `.devcontainer/devcontainer.json`
- CI config
- README commands

### Exit Criteria

- low-confidence inference is never presented as certain
- dry-run output is reviewable and auditable
- provenance is visible for every inferred field
- write mode is gated on dry-run quality, not convenience

### V1f Delivery Sequence

Detection should ship in three internal steps:

1. `V1f-a` implement `ota detect --dry-run`
2. `V1f-b` harden inference quality on fixture repos
3. `V1f-c` add write mode only after the write gate is satisfied

### Detect Write Gate

Write mode must not ship until all of the following are true:

- `ota detect --dry-run` is exercised on the initial fixture set across the first supported stacks
- every inferred field includes provenance
- every inferred field includes confidence
- `low` confidence fields are never written automatically
- initial write mode writes only `high` confidence fields
- generated write candidates pass `ota validate`
- the generated contract is reviewable without guessing where fields came from

Practical quality bar:

- `high` confidence fields should have effectively zero false positives on the initial fixture set
- `medium` confidence fields may appear in dry-run output but should not be written in the first write mode
- write mode should stay conservative until fixture evidence proves it can be broadened safely

## V1g: Agent Surface and Polish

### Goal

Complete the V1 contract and harden the product for real-repo adoption.

### Scope

- support the V1 `agent` section in the schema and command surfaces
- include `verify_after_changes`
- complete JSON output where V1 requires it
- refine help text and CLI semantics
- add examples for initial supported stacks
- validate against real repos across multiple stacks
- tighten docs and quickstart

### Exit Criteria

- agents can determine safe default verification behavior from contract data
- V1 command set is coherent end-to-end
- docs reflect shipped behavior, not aspirational behavior
- quality bar is demonstrated on real repositories

## Quality Gates

Each phase should satisfy the following before the next phase begins:

- command behavior is covered by tests or fixtures
- human output is intentionally designed, not incidental
- exit codes are explicit
- no hidden mutation is introduced
- implementation remains layer-correct

## Out of Scope for V1

These remain outside V1 even if the architecture anticipates them:

- full container execution
- full remote execution
- backend plugins
- monorepo inheritance
- policy packs
- task caching
- extension system

## Implementation Note

The recommended execution model for this repository is:

- keep the spec whole
- implement phases in order
- treat `doctor` and `detect` as trust-sensitive surfaces
- do not expand scope inside a phase unless correctness requires it

The build strategy should favor a small, explicit Rust codebase with stable contracts over a large early surface area.
