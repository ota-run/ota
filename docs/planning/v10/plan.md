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

# V10 Plan

Status: planned.

Source direction:

- [Repo-scoped provisioning](../../spec/provisioning.md)
- [Doctor quality bar](../../spec/doctor-finding-contract.md)
- [Env resolution and policy](../../spec/env-resolution-and-policy.md)
- [Command reference](../../spec/command-reference.md)

V10 theme:

- repo-scoped provisioning as an explicit, bounded capability
- `ota up` as the primary operator command, with opt-in provisioning
- repo readiness that can prepare declared prerequisites without becoming workstation management

This slice is the next post-v1 product step after public adoption hardening.

## Included capabilities

- repo contracts can declare prerequisite intent in a reviewable way
- `ota doctor` can explain what is missing and whether it is provisionable
- `ota up --dry-run --provision` can preview preparation actions before change
- `ota up --provision` can prepare declared repo-scoped prerequisites when the contract allows it
- deterministic output for humans and agents

## Non-goals

- no hidden workstation management
- no system-wide install orchestration
- no background daemon or hosted control plane
- no remote orchestration disguised as local setup
- no silent mutation in `doctor`, `init`, or `detect`

## Priorities

1. Keep provisioning bounded to repo-declared intent
2. Preserve `ota up` as the primary operator entrypoint
3. Make dry-run and remediation explicit
4. Keep humans and agents aligned on what is safe to run

## Execution slices

1. Contract shape

- define a small `provision` block for declared prerequisite intent
- keep it declarative and reviewable
- avoid arbitrary install scripts by default

1. `doctor` and `up` integration

- surface provisionable missing prerequisites in `doctor`
- allow `ota up --provision` to prepare declared prerequisites
- keep `ota up` honest when provisioning is not allowed or not available

1. Output and provenance

- explain what would be provisioned and why
- record the chosen manager or source when known
- keep dry-run and JSON output deterministic

1. Docs and examples

- document when to use provisioning versus manual setup
- keep the README and quickstart aligned with the trust model
- add examples that show repo-scoped provisioning without turning Ota into a workstation manager

## Success criteria

- repo-scoped provisioning is explicit and reviewable
- `ota up --provision` can prepare declared prerequisites safely
- `ota doctor` explains provisionable gaps clearly
- docs tell users when to use provisioning and when not to
- the feature improves onboarding without weakening trust
