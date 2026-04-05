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

# V2.1 Plan

Status: completed and archived.

V2.1 is the bridge after V2:

- V1 proved repo readiness, workspace bootstrap, and deterministic execution
- V2 proved conservative onboarding, real-repo handling, and agent-useful contract surfaces

The point of V2.1 is not more breadth for its own sake.

The point of V2.1 is to increase ota's leverage as infrastructure around the now-trusted core.

V2.1 makes ota:

- easier to integrate into existing toolchains without creating parallel truth
- more useful for teams coordinating across many repos
- stronger as a machine-facing contract while staying human-legible
- more valuable as shared infrastructure, not just a repo-local CLI

## Priorities

1. Narrow interoperability
2. Team and workspace leverage
3. Machine integration stability
4. Optional execution expansion only when clearly justified

## Rules

- `ota.yaml` remains the canonical repo contract
- `ota.workspace.yaml` remains the canonical workspace/bootstrap contract
- derived artifacts must stay one-way outputs, not peer sources of truth
- no provider-specific GitHub, GitLab, or Bitbucket product logic in the core model
- no hidden mutation, background daemons, or opaque state
- no broad export family until one target proves clear duplicated-truth pain
- no workspace feature that duplicates repo readiness semantics

## Candidate tracks

### Track 1: Narrow interoperability

- pick at most one derived output first
- only ship it if it clearly removes duplicated truth
- keep ota canonical and the output disposable/regenerable

Likely first candidate:

- `AGENTS.md` derived from `ota.yaml`, only if teams are already duplicating that guidance manually

### Track 2: Team and workspace leverage

- improve workspace summaries and status surfaces
- consider bounded workspace task orchestration only if it composes existing repo truth cleanly
- keep workspace logic orchestration-only

### Track 3: Machine integration stability

- keep JSON output and schema stability high as the integration surface grows
- extend machine-readable guidance only where it improves trust or automation quality
- prefer additive, explicit contract evolution over implicit behavior

### Track 4: Execution model expansion

- only consider broader execution backends if the current shell-native model proves insufficient
- require determinism, debuggability, and clear contract semantics before expanding here

## Non-goals

- replacing ota with a generator-first system
- broad export matrices
- hidden workstation or host provisioning
- provider-specific platform integrations
- a second execution truth outside the contract
- workspace features that collapse repo/workspace boundaries

## Completion bar

V2.1 is complete only when:

- machine-readable command surfaces are documented and backed by canonical schemas where ota publishes them
- workspace JSON and orchestration behavior are covered through the real CLI path, not only unit tests
- top-level docs and planning state were aligned while closing V2.1
- V3 should not start until the above is true
