span

# Exit Codes

Ota uses stable exit code behavior for scripting and CI.

When to use:

- any automated integration where command success/failure must be deterministic

Why:

- text output can vary for humans, but exit codes remain the control signal for machines

Common cases:

- `0` command succeeded
- `1` command failed due to validation, readiness, runtime, or task failure
- `2` command usage error (for example invalid flag combinations or duplicate member flags)

Use-case:

- in CI, treat `1` as contract/readiness failure and `2` as pipeline misconfiguration.

For full command-by-command semantics, use the canonical reference:


# Ota Exit Codes

This document records the current command exit-code contract for the shipped Ota surface.

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
- child task exit code is preserved for native, container, and current remote execution paths
- `1` when backend configuration is invalid or the requested backend/provider is unsupported
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
- setup task child exit code is preserved when `setup` runs through native, container, or current remote backend paths
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

## `ota workspace validate`

- `0` on valid workspace contract
- `1` on load or validation failure

## `ota workspace tasks`

- `0` on successful workspace task listing
- `1` on load or validation failure

## `ota workspace list`

- `0` on successful workspace repo inventory output
- `1` on load or validation failure

## `ota workspace run`

- `0` when all required repos complete the requested task
- `1` when any required repo task fails, acquisition fails, or is blocked by dependency failure
- `1` on load or validation failure

## `ota workspace check`

- `0` when all required repos are check-ready or warning-only
- `1` when any required repo has a blocking check finding
- `1` on load or validation failure

## `ota clean`

- `0` when persistent execution state is removed
- `0` when there is no cleanup action to perform
- `1` on load or validation failure
- `1` when persistent cleanup fails before Ota can report success

## `ota workspace doctor`

- `0` when all required repos are ready or warning-only
- `1` when any required repo has a blocking finding
- `1` on load or validation failure

## `ota workspace up`

- `0` when all required repos reach `READY`
- `1` when any required repo fails acquisition or does not become ready
- `1` on load or validation failure

## Notes

- JSON mode does not change exit-code behavior
- `ok: true` in JSON output is intentionally aligned with exit code `0`
- warning-only diagnosis is still success


