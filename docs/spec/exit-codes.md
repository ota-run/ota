# Ota Exit Codes

This document records the current command exit-code contract for the shipped V1 surface.

## Global registry

- `0`: success, ready state, or warning-only diagnosis
- `1`: invalid contract, blocking readiness issue, protected write failure, or general command failure
- `2`: CLI usage or argument parsing error

## Command-specific rules

## `ota validate`

- `0` on valid contract
- `1` on load or validation failure

## `ota tasks`

- `0` on valid contract and successful task listing
- `1` on load or validation failure

## `ota run`

- `0` on successful task execution
- child task exit code on task failure
- `1` on load/validation failure or runner failure before the child exit code is available

## `ota doctor`

- `0` when findings are empty or warning-only
- `1` when any blocking readiness finding exists
- `1` on load or validation failure

## `ota check`

- `0` when configured checks are empty or warning/info-only
- `1` when any configured check produces an error-severity finding
- `1` on load or validation failure

## `ota init`

- `0` on successful review output or write
- `1` when an existing `ota.yaml` blocks init
- `1` on detection failure
- `1` on write failure

## `ota up`

- `0` when the repo reaches `READY`
- service-start child exit code when a required service `start` command fails
- setup task child exit code when `setup` fails
- `1` when preconditions fail
- `1` when required-service readiness fails in the `services` phase
- `1` when post-setup diagnosis is still not ready
- `1` on load or validation failure

## `ota detect`

- `0` on successful dry-run output
- `0` on successful write
- `1` when an existing `ota.yaml` blocks write
- `1` when the high-confidence projection is insufficient to produce a valid contract
- `1` on detection failure
- `1` on write failure

## Notes

- JSON mode does not change exit-code behavior
- `ok: true` in JSON output is intentionally aligned with exit code `0`
- warning-only diagnosis is still success
