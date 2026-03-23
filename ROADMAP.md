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
V3 is active.

## V1 archive

- [V1 phases](docs/planning/v1/phases.md)
- [V1 next plan](docs/planning/v1/next-plan.md)
- [V1 release gate](docs/planning/v1/release-gate.md)
- [V2 plan](docs/planning/v2/plan.md)

## Current focus

- keep the V1 release gate green
- keep public docs aligned with shipped behavior
- ship V3 in narrow, real slices
- start with monorepo/root-member support before backend expansion

## V2 archive

- [V2 plan](docs/planning/v2/plan.md)
- [V2.1 plan](docs/planning/v2.1/plan.md)

## Active version

- [V3 plan](docs/planning/v3/plan.md)

## Implemented foundation

- repo contract validation and execution
- repo diagnosis and onboarding
- conservative detection and init
- workspace validation, diagnosis, bootstrap, and git-based acquisition

## Near-term next steps

- keep the V2 trust/adoption baseline stable
- ship root/member monorepo support across repo commands
- keep extending fixtures only when they expose real gaps
- avoid V3 scope that weakens V1, V2, or V2.1 trust or determinism
- keep exports optional unless one exact target proves real duplicated-truth pain

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

V3 themes build on the fully closed V2.1 foundation with:
- monorepo and workspace maturity
- first-class backend execution beyond native-only operation
- stable machine-readable diagnostics and richer operational policy
- serious multi-package and remote/container team workflows
