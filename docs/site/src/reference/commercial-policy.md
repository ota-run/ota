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

# Commercial Policy

Ota is open source under Apache 2.0. The commercial model stays separate from the core codebase.

## Source model

`docs/spec` is the canonical source of truth. This page is the public reference
layer derived from it. It adds examples, use cases, and operator guidance so the
page stands on its own while staying aligned with shipped behavior.

## Open core

- CLI
- repo and workspace contracts
- JSON output and docs
- contract-first readiness model

## Policy

- no external code contributions
- the `Ota` name and logo stay reserved
- enterprise packaging stays separate from the OSS core

See [brand-policy.md](brand-policy.md) for the brand usage boundary.

## Boundary

The core should stay small, deterministic, and broadly adoptable. Commercial value belongs in
separate services, private modules, hosting, support, or packaging around the open core.
