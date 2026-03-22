# Security Posture

Ota executes repo-declared commands. That makes trust boundaries explicit and important.

## Ota will not do silently

- overwrite an existing `ota.yaml`
- mutate repo config without an explicit command path
- auto-install tools
- execute inferred commands without review

## Current trust model

- `detect` is review-first
- `init` is review-first unless `--write` is provided
- `run`, `up`, `doctor`, and `check` execute repo-controlled commands from the contract

## Risks to acknowledge

- shell execution is a command-execution surface
- repo-controlled commands may be unsafe in untrusted repositories
- agents running repo tasks inherit that trust decision

## Safe default

Treat repo task execution as trusted-repo behavior, not untrusted-content sandboxing.
