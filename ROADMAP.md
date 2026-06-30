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
V2 is complete and archived.
V2.1 is complete and archived.
V3 is complete.
V4 is complete.
V5 is complete.
V6 is complete.
V7 is complete and archived.
V7.1 is complete and archived.
V7.2 is complete and archived.
V8 is complete and archived.
V9 is complete and archived.
V9.1 is complete and archived.
V10 is complete and archived.
V11 is planned.
V11.1 is active.
V11.2 is planned.

## V1 archive

- [V1 phases](docs/planning/v1/phases.md)
- [V1 next plan](docs/planning/v1/next-plan.md)
- [V1 release gate](docs/planning/v1/release-gate.md)
- [V2 plan](docs/planning/v2/plan.md)

## Current focus

- keep the V1 release gate green
- keep public docs aligned with shipped behavior
- keep `ota agents` visible as a first-class derived adoption surface
- keep the premium UX review loop green when text/help surfaces change
- hold enterprise-facing scope behind the adoption readiness gate
- keep the V11.1 execution-governance visibility and proof slice narrow and reviewable
- keep the canonical roadmap aligned with the spec repo versioning

## V2 archive

- [V2 plan](docs/planning/v2/plan.md)
- [V2.1 plan](docs/planning/v2.1/plan.md)

## V6 archive

- [V6 plan](docs/planning/v6/plan.md)

## V7 archive

- [V7 plan](docs/planning/v7/plan.md)
- [V7.1 plan](docs/planning/v7.1/plan.md)
- [V7.2 plan](docs/planning/v7.2/plan.md)
- [V8 plan](docs/planning/v8/plan.md)
- [V9 plan](docs/planning/v9/plan.md)

## Active version

- [V11.1 plan](docs/planning/v11.1/plan.md)

## Next planned version

- [V11 plan](docs/planning/v11/plan.md)
- [V11.2 plan](docs/planning/v11.2/plan.md)

## Implemented foundation

- repo contract validation and execution
- repo diagnosis and onboarding
- conservative detection and init
- workspace validation, diagnosis, bootstrap, and git-based acquisition

## Near-term next steps

- preserve the shipped V10 semantic snapshot and correlation foundation
- preserve the shipped repo/workspace trust baseline
- keep docs and active planning aligned with the canonical spec repo
- keep extension and editor surfaces contract-bound
- avoid widening into generic plugin-runtime scope
- stage V11 completion truth, reviewer evidence, and local/CI/agent convergence on top of the
  shipped semantic snapshot foundation
- make V11.1 execution governance visibility and proof the active `1.6.23` slice
- make V11.2 source convergence and detection governance the next disciplined widening slice after
  V11.1

## Archived V2 shape

Shipped V2 work so far is still intentionally narrow:

- better real-repo detect coverage for common Node and Python repo shapes
- conservative existing-contract comparison and additive merge for `ota detect`
- stronger real-fixture coverage for mixed, legacy, and conflicting repo shapes
- agent guidance surfaced on existing machine-readable and human-readable command paths

## Archived V2.1 shape

The repo-first bridge work beyond that foundation focused on:

- narrow interoperability framing without broad exports
- stronger machine-facing guidance and JSON clarity
- tighter workspace/team leverage boundaries without a second bootstrap engine

## Product direction

V7 themes build on the shipped repo/workspace foundation with:

- editor/IDE integration contract stabilization
- remote runner metadata standard finalization
- hosted validation workflow shape
- stronger machine-facing operational summaries
