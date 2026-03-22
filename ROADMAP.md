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
The binding V1 closeout bar is in [docs/planning/v1-release-gate.md](docs/planning/v1-release-gate.md).

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

- binding the current implementation to the V1 release gate
- keeping the public spec and docs aligned with shipped behavior

## Near-term next steps

- keep the fixture matrix as the source of truth for V1 closeout
- expand fixtures only when they expose real gaps
- avoid post-V1 scope until the release gate is satisfied

## Product direction

Longer-term work is expected to include:

- broader interop coverage
- richer repo examples
- stronger agent-facing contract guidance
- more spec/documentation polish for external adoption
- a possible post-V1 execution model beyond shell-native `run` and `script`
