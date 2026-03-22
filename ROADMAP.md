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

The detailed implementation plan is in [docs/v1-phases.md](/Users/bobai/Workspace/Ota.run/ota/docs/v1-phases.md).

## Implemented so far

- `ota validate`
- `ota tasks`
- `ota run`
- `ota doctor`
- `ota up`
- `ota detect --dry-run`
- conservative `ota detect` write mode

## Current focus

- stronger public docs and examples
- broader fixture coverage
- better `doctor` quality
- better detect coverage across common repo shapes

## Near-term next steps

- add more real-repo detection fixtures
- improve command reference docs
- add more stack examples
- continue hardening output and exit semantics

## Product direction

Longer-term work is expected to include:

- broader interop coverage
- richer repo examples
- stronger agent-facing contract guidance
- more spec/documentation polish for external adoption
