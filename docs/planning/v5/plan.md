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

Status: active.

Source direction:
- [/Users/bobai/Desktop/Ota.run/Spec/New/10-v5-spec.md](/Users/bobai/Desktop/Ota.run/Spec/New/10-v5-spec.md)
- [/Users/bobai/Desktop/Ota.run/Spec/New/ACTIVE_VERSION.md](/Users/bobai/Desktop/Ota.run/Spec/New/ACTIVE_VERSION.md)

V5 theme:

- organizations
- governance
- standardization at scale

## Included capabilities

- policy packs applied across repos in an org
- org-level conventions and templates
- signed config and provenance options
- team-level templates
- audit-friendly machine output
- remote runner metadata standard
- editor/IDE integration spec
- advanced caching
- enterprise-safe mutation controls

## Priorities

1. Make Ota viable for organizations that need consistent standards across many repos
2. Enable platform teams to define and enforce policy once
3. Provide audit trails that stay useful under review and compliance pressure
4. Stabilize the editor integration surface for IDE adoption

## Execution slices

1. Policy pack model
- define an org-level policy contract that applies across repos
- keep policy evaluation deterministic and explicit

2. Conventions and templates
- standardize shared repo templates and org conventions
- keep template application auditable and non-magical

3. Audit and provenance
- make machine output suitable for auditing and traceability
- surface signed config/provenance in a stable way

4. Remote runner and editor surface
- standardize remote runner metadata
- keep editor/IDE integrations on the documented contract surface

5. Mutation controls and caching
- keep mutation paths explicit and enterprise-safe
- use caching only where it preserves determinism and trust

## Success criteria

- a platform team can define a policy pack that applies consistently across many repos
- audit logs remain useful for review and compliance use cases
- editor integrations can discover tasks and readiness without custom reverse-engineering
- signed `ota.yaml` is a viable option for security-sensitive environments
