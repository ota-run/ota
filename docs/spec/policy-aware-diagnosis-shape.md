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

# Policy-Aware Diagnosis Shape (V4 Prep)

Purpose: define a non-breaking machine-shape target for policy-aware readiness diagnostics.

## Scope

- diagnostic only
- no provisioning side effects
- no new exit-code family in V4

## Proposed additive finding context

When policy-aware requirement diagnosis is introduced, finding objects may include additive keys:

- `policy_outcome`: `satisfied` | `installable` | `blocked_by_policy` | `blocked_by_source_unavailable` | `blocked_by_integrity_policy` | `unsupported_resolution_path`
- `policy_reason`: short machine-friendly reason code when blocked
- `policy_source`: `repo` | `local` | `org` | `dashboard` when known
- `install_scope`: `host` | `repo_local` | `container` | `remote` when relevant
- `mutation_allowed`: boolean for the current command mode

These are additive only and must not break current consumers.

## Compatibility rule

- Existing top-level command JSON keys remain stable in V4.
- Existing finding keys remain valid; new keys are optional.
- Commands remain non-mutating by default for policy-aware diagnosis paths.

## Command expectations

- `ota doctor --json`: may surface additive policy-aware finding context.
- `ota check --json`: may surface additive policy-aware finding context for check-only flows.
- `ota init` / `ota detect`: may report implications but must not perform provisioning mutation.
