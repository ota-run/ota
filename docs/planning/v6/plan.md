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

# V6 Plan

Status: active.

Source direction:

- [/Users/bobai/Desktop/Ota.run/Spec/new/21a-v6-extension-contract-normative.md](/Users/bobai/Desktop/Ota.run/Spec/new/21a-v6-extension-contract-normative.md)
- [Extension execution boundary](/Users/bobai/Workspace/Ota.run/ota/docs/spec/extension-execution-boundary.md)
- [Compatibility policy](/Users/bobai/Workspace/Ota.run/ota/docs/spec/compatibility-policy.md)
- [Remote runner and editor surface](/Users/bobai/Workspace/Ota.run/ota/docs/spec/remote-runner-and-editor-surface.md)
- [Mutation controls and caching](/Users/bobai/Workspace/Ota.run/ota/docs/spec/mutation-controls-and-caching.md)

V6 theme:

- ecosystem and extension readiness
- compatibility and adapter stability
- editor/IDE and remote-runner integration

## Included capabilities

- explicit extension contract boundaries
- compatibility policy and conformance surfaces
- adapter and execution metadata stability
- editor/IDE integration hooks
- remote runner visibility
- deterministic mutation controls
- caching that preserves trust

## Priorities

1. Keep extension behavior explicit, opt-in, and testable
2. Preserve core command determinism while widening the ecosystem surface
3. Give editors and integrations a stable contract to consume
4. Keep mutation and caching rules narrow enough to trust

## Execution slices

1. Extension contract rollout

- keep top-level extension behavior contract-bound
- preserve the current rejection boundary until extension execution is ready
- align to the normative extension contract target

1. Compatibility and adapter stability

- keep compatibility rules explicit and testable
- preserve command/JSON/exit stability while the ecosystem widens
- align adapter behavior to the documented compatibility policy

1. Remote runner and editor integration

- surface integration metadata in a stable shape
- keep editor/IDE use cases first-class without changing core execution semantics

1. Mutation controls and caching

- keep write paths explicit and bounded
- use caching only where it improves determinism and operator trust

## Current progress

- extension contract is discoverable and `ota extensions --run <name>` provides the explicit
  execution seam
- compatibility policy documented
- remote-runner/editor surface documented
- mutation/caching target documented

## Success criteria

- extension behavior stays explicit and opt-in
- compatibility expectations are stable enough for external consumers
- editor and remote-runner integrations can consume a predictable contract
- caching and mutation controls do not weaken `ota doctor` or `ota detect` trust
