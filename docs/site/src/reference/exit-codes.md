<!--
                █████
               ░░███
       ██████  ███████    ██████
      ███░░███░░░███░    ░░░░░███
     ░███ ░███  ░███      ███████
     ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
      ░░░░░░     ░░░░░   ░░░░░░░░

   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
   You may not use this file except in compliance with that License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Exit Codes

Ota uses stable exit codes so CI, shell scripts, and agents can react to success and failure
without parsing human text output.

## Source model

`docs/spec` is the canonical source of truth. This page is the public reference
layer derived from it. It adds examples, use cases, and operator guidance so the
page stands on its own while staying aligned with shipped behavior.

## Why it matters

- exit codes are the control signal for automation
- text output can change for humans without changing machine behavior
- the same code should mean the same thing across repo and workspace commands

## Global registry

- `0`: success, ready state, or warning-only diagnosis
- `1`: invalid contract, blocking readiness issue, protected write failure, or general command failure
- `2`: CLI usage or argument parsing error

## Common cases

- `0` means the command succeeded
- `1` means the command failed because the contract, readiness, backend, or task execution failed
- `2` means the command was called incorrectly

In CI, treat `1` as a contract or readiness failure and `2` as a pipeline misuse or bad invocation.

## Command-specific rules

### `ota validate`

- `0` on valid contract
- `1` on load or validation failure

### `ota tasks`

- `0` on valid contract and successful task listing
- `1` on load or validation failure

### `ota run`

- `0` on successful task execution
- child task exit code on task failure
- child task exit code is preserved for native, container, and current remote execution paths
- `1` when backend configuration is invalid or the requested backend/provider is unsupported
- `1` on load/validation failure or runner failure before the child exit code is available

### `ota doctor`

- `0` when findings are empty or warning-only
- `1` when any blocking readiness finding exists
- `1` on load or validation failure

### `ota check`

- `0` when configured checks are empty or warning/info-only
- `1` when any configured check produces an error-severity finding
- `1` on load or validation failure

### `ota init`

- `0` on successful review output or write
- `1` when an existing `ota.yaml` blocks init
- `1` on detection failure
- `1` on write failure

### `ota up`

- `0` when the repo reaches `READY`
- service-start child exit code when a required service `start` command fails
- setup task child exit code when `setup` fails
- setup task child exit code is preserved when `setup` runs through native, container, or current remote backend paths
- `1` when preconditions fail
- `1` when required-service readiness fails in the `services` phase
- `1` when post-setup diagnosis is still not ready
- `1` on load or validation failure

### `ota detect`

- `0` on successful dry-run output
- `0` on successful write
- `1` when an existing `ota.yaml` blocks write
- `1` when the high-confidence projection is insufficient to produce a valid contract
- `1` on detection failure
- `1` on write failure

### `ota workspace validate`

- `0` on valid workspace contract
- `1` on load or validation failure

### `ota workspace tasks`

- `0` on successful workspace task listing
- `1` on load or validation failure

### `ota workspace list`

- `0` on successful workspace repo inventory output
- `1` on load or validation failure

### `ota workspace run`

- `0` when all required repos complete the requested task
- `1` when any required repo task fails, acquisition fails, or is blocked by dependency failure
- `1` on load or validation failure

### `ota workspace check`

- `0` when all required repos are check-ready or warning-only
- `1` when any required repo has a blocking check finding
- `1` on load or validation failure

### `ota clean`

- `0` when persistent execution state is removed
- `0` when there is no cleanup action to perform
- `1` on load or validation failure
- `1` when persistent cleanup fails before Ota can report success

### `ota workspace doctor`

- `0` when all required repos are ready or warning-only
- `1` when any required repo has a blocking finding
- `1` on load or validation failure

### `ota workspace up`

- `0` when all required repos reach `READY`
- `1` when any required repo fails acquisition or does not become ready
- `1` on load or validation failure

## Use cases

- CI decides whether to fail a pipeline
- shell scripts branch on success or failure
- agents distinguish between “bad invocation” and “repo is not ready”
- hosted validation maps exit codes to check conclusions
- editors can surface the right error state without parsing text output

## JSON alignment

- JSON mode does not change exit-code behavior
- `ok: true` in JSON output is intentionally aligned with exit code `0`
- warning-only diagnosis is still success
