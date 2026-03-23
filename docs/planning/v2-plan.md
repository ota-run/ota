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

# V2 Plan

Status: active planning.

V2 should expand Ota only after preserving the shipped V1 trust model.

## Priorities

1. Interop and export surfaces
2. Stronger agent-facing contract guidance
3. Broader workspace ergonomics above the current bootstrap layer
4. Post-V1 execution model expansion only where it improves determinism and clarity

## Rules

- preserve the V1 repo and workspace contract boundaries
- preserve doctor-first and trust-sensitive detection/init behavior
- do not weaken deterministic shell-native execution without an explicit replacement model
- keep new abstractions justified by real fixture or integration pressure

## First candidate tracks

### Track 1: Interop

- export Ota contract data into other ecosystem surfaces where it reduces duplication
- keep Ota as the source of truth instead of generating parallel handwritten config

### Track 2: Agent Contract

- tighten the `agent` section into a clearer, more useful operational surface
- improve verification defaults and writable-path guidance

### Track 3: Workspace UX

- consider richer workspace-level status and orchestration only when it composes repo truth cleanly
- keep acquisition and bootstrap explicit and inspectable

### Track 4: Execution Model

- evaluate post-V1 task execution expansion only if it remains deterministic, debuggable, and contract-first

## Non-goals

- replacing `ota.yaml` as the repo source of truth
- turning Ota into a hidden workstation manager
- adding host-specific magic or provider-specific GitHub behavior into the core model
