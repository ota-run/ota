# Roadmap

## Current build path

Ota is being built in phased V1 delivery rather than by cutting V1 scope.

Current phase map:

1. `V1a` Contract Core
2. `V1b` Read Path
3. `V1c` Execution Core
4. `V1d` Diagnosis Core
5. `V1e` Onboarding Path
6. `V1f` Detection
7. `V1g` Agent Surface and Polish

The detailed implementation plan is in [docs/planning/v1-phases.md](docs/planning/v1-phases.md).
The detect write threshold is documented in [docs/design/detect-write-gate.md](docs/design/detect-write-gate.md).
Future task executor expansion is tracked in the product spec.
The next execution plan is in [docs/planning/v1-next-plan.md](docs/planning/v1-next-plan.md).

## Implemented so far

- `ota validate`
- `ota tasks`
- `ota run`
- `ota doctor`
- `ota init`
- `ota check`
- `ota up`
- `ota detect --dry-run`
- conservative `ota detect` write mode

## Current focus

- service behavior that stays explicit and non-orchestrator
- honest `execution.lifecycle` semantics
- exit-code and debug-mode hardening

## Near-term next steps

- finish `services` behavior in `doctor` and `up`
- define and implement honest `execution.lifecycle` behavior
- formalize exit codes and add debug mode

## Product direction

Longer-term work is expected to include:

- broader interop coverage
- richer repo examples
- stronger agent-facing contract guidance
- more spec/documentation polish for external adoption
- a possible post-V1 execution model beyond shell-native `run` and `script`
