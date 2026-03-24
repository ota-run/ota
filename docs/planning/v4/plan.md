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

# V4 Plan

Status: active (started 2026-03-24).

Source direction:
- [09-v4-spec.md](/Users/bobai/Desktop/Ota.run/Spec/new/09-v4-spec.md)
- [ACTIVE_VERSION.md](/Users/bobai/Desktop/Ota.run/Spec/new/ACTIVE_VERSION.md)

V4 theme:

- ecosystem standardization
- compatibility reliability
- extension/adapter readiness without trust regressions

## Included capabilities

### Compatibility baseline

- define stable command/JSON/exit compatibility assertions
- add compatibility-oriented fixtures for core repo shapes
- lock output contracts for trust-sensitive commands

### Extension preparation

- align implementation work to the normative extension contract target
- keep extension behavior explicit, deterministic, and opt-in
- preserve existing command semantics when extensions are absent

### Policy-aware diagnostics prep

- shape `doctor`/machine outputs so installable-vs-blocked policy signals are representable
- keep diagnostics non-mutating by default

## Execution slices

1. Compatibility surface inventory
- enumerate current command/JSON/exit contracts that must be stabilized in V4
- produce a single compatibility checklist in repo docs (`docs/spec/compatibility-surface.md`)

2. Compatibility test harness baseline
- add test grouping/layout for compatibility assertions
- lock current expected behavior for validate/tasks/doctor/detect/up/check

3. Extension contract alignment pass
- map existing extension mentions to one implementation-ready contract target
- avoid introducing runtime plugin loading until contract and tests are stable

4. Policy-aware doctor shape prep
- add bounded machine fields for policy-aware readiness outcomes (diagnostic only)
- keep exit behavior compatible with current global rules

5. Docs and conformance sync
- update docs pages that define command/machine behavior
- ensure planning/spec links remain consistent

## Current progress

Completed:

- slice 1: compatibility surface inventory added in `docs/spec/compatibility-surface.md`
- slice 2 baseline: consolidated repo JSON contract-stability tests added for success/failure paths
- workspace JSON contract-stability tests validated against the new baseline
- `ota up --json` validation/load failure paths aligned to return JSON errors consistently
- slice 3 guardrails started: extension execution boundary documented in `docs/spec/extension-execution-boundary.md`
- slice 3 guardrails started: validation test locks rejection of top-level `extensions` until V6
- slice 4 prep: policy-aware diagnosis machine-shape draft added in `docs/spec/policy-aware-diagnosis-shape.md`

## Acceptance criteria

- V4 changes preserve deterministic behavior across core commands
- compatibility tests prove no accidental machine/output drift on core surfaces
- extension-facing work is contract-bound and backward-compatible by default
- diagnostics stay non-mutating unless an explicit command mode allows mutation
- docs/planning/spec links for V4 are aligned and current

## Out of scope for V4

- enterprise artifact provisioning and installer/source enforcement (V9)
- new exit-code family beyond current shipped compatibility contract
- mandatory plugin runtime for core command execution
