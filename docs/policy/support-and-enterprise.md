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

# Support and Enterprise

ota’s open core stays Apache 2.0. Commercial value should come from separate offerings around the
core rather than from confusing the public contract.

## Good enterprise candidates

- hosted control plane for policy, audit, and fleet-level coordination
- private adapters or backends
- organization policy packs and approvals
- support, onboarding, and migration services
- compliance retention and reporting

## Keep separate from the core

- the CLI contract
- repo/workspace schemas
- JSON output shape
- public docs and examples

## Operating rule

If a feature needs to be paid, keep its contract boundary explicit and do not mix it into the
public open-core schema unless that is strategically unavoidable.
