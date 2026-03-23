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

# Roadmap

## Current state

V1 is complete and frozen.
V2 is complete enough to archive as the current trust/adoption foundation.
V3 planning is now the active track.

## V1 archive

- [V1 phases](docs/planning/v1/phases.md)
- [V1 next plan](docs/planning/v1/next-plan.md)
- [V1 release gate](docs/planning/v1/release-gate.md)
- [V2 plan](docs/planning/v2/plan.md)

## Current focus

- keep the V1 release gate green
- keep public docs aligned with shipped behavior
- start V3 from the now-proven V1 and V2 core instead of broadening V2 indefinitely

## V3 planning

- [V3 plan](docs/planning/v3/plan.md)

## Implemented foundation

- repo contract validation and execution
- repo diagnosis and onboarding
- conservative detection and init
- workspace validation, diagnosis, bootstrap, and git-based acquisition

## Near-term next steps

- keep the V2 trust/adoption baseline stable
- keep extending fixtures only when they expose real gaps
- avoid V3 scope that weakens V1 or V2 trust or determinism
- keep exports optional unless one exact target proves real duplicated-truth pain

## Archived V2 shape

Shipped V2 work so far is still intentionally narrow:

- better real-repo detect coverage for common Node and Python repo shapes
- conservative existing-contract comparison and additive merge for `ota detect`
- stronger real-fixture coverage for mixed, legacy, and conflicting repo shapes
- agent guidance surfaced on existing machine-readable and human-readable command paths

## Product direction

Planned V3 themes should build on that foundation with:

- narrow interoperability where Ota remains canonical
- team and workspace leverage without a second bootstrap engine
- stronger machine integration without hidden mutation
- no broad exports or provider-specific platform logic by default
