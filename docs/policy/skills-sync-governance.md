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

# First-Party Consumer Sync Governance

Ota releases and release-gated contract-surface changes must not widen the product quietly while
first-party consumer repos still teach or render an older product shape.

## What is enforced

When a change touches contract-shape or contract-governance files such as:

- `src/schema.rs`
- `src/validator.rs`
- key contract/spec docs under `docs/spec/`
- canonical examples under `examples/`

the release gate requires an update to the affected first-party consumer status file.

Current governed consumers:

- `ota-run/skills`
  - status file: [skills-sync-status.yaml](skills-sync-status.yaml)
  - records whether the first-party Ota skill package was updated or explicitly waived
- `ota-run/ota-site`
  - status file: [ota-site-sync-status.yaml](ota-site-sync-status.yaml)
  - records whether the public docs site was updated or explicitly waived when canonical docs
    surfaces or published-docs ownership changed

Each status file must say one of:

- `mode: synced`
  - the matching consumer update has been made
  - `consumer_commit` must record the exact synced consumer commit
- `mode: waived`
  - the maintainer is intentionally shipping without a consumer update
  - `waiver_reason` must explain why

## Why this exists

The first-party skill package and public docs site are both part of the product surface.
If schema, governance, or canonical docs behavior widen without a corresponding consumer update or
explicit waiver, agents and public docs drift behind the platform while appearing current.

This gate makes that drift visible and reviewable.
