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

# V9 Plan

Status: complete.

Source direction:

- [Env resolution and policy](../../spec/env-resolution-and-policy.md)
- [Policy-backed provisioning sources](../../spec/policy-backed-provisioning.md)

V9 theme:

- policy-controlled env resolution
- execution provenance
- repository operability with explicit env boundaries

This slice is complete; the next active slice should be v10.

Follow-on direction:

- keep env resolution separate from provisioning source selection
- use policy-backed provisioning sources only when repo-declared runtimes or tools need approved install origins

## Included capabilities

- deterministic env resolution precedence
- policy-controlled env source selection
- provenance-aware env injection for execution

## Priorities

1. Keep env resolution explicit and auditable
2. Preserve repo/workspace trust boundaries
3. Keep app config ownership outside Ota

## Execution slices

1. Env resolution schema

- define approved env sources and precedence
- keep repo and workspace env contracts compatible

1. Provenance-aware injection

- record which source won for each env value
- surface that in execution receipts

1. Policy-controlled validation

- validate env against org policy
- keep resolution deterministic and read-only outside execution

## Success criteria

- env values can be resolved from approved sources
- execution receipts can show env provenance
- Ota remains bounded and does not become a generic app config system
