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

# V5 Plan

Status: planned (not started).

Source direction:
- [10-v5-spec.md](/Users/bobai/Desktop/Ota.run/Spec/new/10-v5-spec.md)
- [ACTIVE_VERSION.md](/Users/bobai/Desktop/Ota.run/Spec/new/ACTIVE_VERSION.md)

V5 theme:

- trustworthy operator UX
- real-repo/workspace readiness flow hardening
- stronger human/agent usability without semantic drift

## Included capabilities

### 1. UX contract consistency hardening

- normalize `Where/Why/Next` behavior across all repo/workspace commands
- remove circular or conflicting `Next` guidance
- keep status semantics strict (`VALID`/`READY`/`NOT READY` only where meaningful)
- keep list/inventory commands non-readiness by default

### 2. Path and command recommendation correctness

- make displayed/suggested paths current-directory-relative when possible
- keep fallback path rendering deterministic when relative conversion is not possible
- ensure repo-scoped recommendations use correct repo-local targets

### 3. Command help and docs parity

- ensure `--help` surfaces reflect shipped flags/filters
- sync command reference, JSON reference, exit-codes, and site docs for new/changed behavior
- keep docs adoption-first (when/why/use-case), not list-only

### 4. Concise/verbose behavioral split

- make `--concise` materially shorter on high-noise commands while preserving decisions/actions
- keep `--verbose` as full explanatory surface
- add tests to lock concise-vs-default output intent

### 5. Output-stability tests for UX surfaces

- add/extend text-output contract tests for trust-sensitive UX paths
- lock known failure envelopes (`Where/Why/Next`) for top commands
- preserve compatibility baseline from V4 (`scripts/test-compat.sh`)

## Execution slices

1. Error-envelope normalization
- consolidate no-contract/no-workspace failure wording across repo/workspace commands
- enforce single authoritative `Next` behavior per failure case

2. Path rendering standardization
- apply cwd-relative path rendering consistently to text outputs and recommended commands
- add regression tests for common cwd combinations

3. Concise-mode finishing pass
- identify highest-noise command outputs and trim non-essential detail in concise mode
- add output tests to prevent accidental regressions

4. Docs and help conformance
- sync command docs/spec docs with implemented UX/output behavior
- verify command help examples and flags are current

5. Compatibility gate + release readiness
- run compatibility gate and targeted UX tests
- document any intentional contract shifts in same change

## Acceptance criteria

- no circular/conflicting `Next` guidance in top repo/workspace commands
- missing-contract/missing-workspace failures provide actionable and consistent `Where/Why/Next`
- path and recommendation rendering is accurate from common operator cwd contexts
- concise mode is measurably leaner on selected high-noise commands
- docs and help reflect shipped behavior for all changed command surfaces
- V4 compatibility gate remains green

## Out of scope for V5

- new backend provider architecture or isolation runtime expansion
- enterprise artifact policy/provisioning enforcement layer
- major schema family expansion beyond current compatibility boundaries
- introducing new core command families not required for UX/flow hardening
