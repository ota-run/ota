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

# Skills Sync Governance

Ota releases and release-gated contract-surface changes must not widen the product quietly while
the first-party `ota-run/skills` package lags behind.

## What is enforced

When a change touches contract-shape or contract-governance files such as:

- `src/schema.rs`
- `src/validator.rs`
- key contract/spec docs under `docs/spec/`
- canonical examples under `examples/`

the release gate requires an update to [skills-sync-status.yaml](skills-sync-status.yaml).

That status file must say one of:

- `mode: synced`
  - the matching `ota-run/skills` update has been made
  - `skills_commit` must record the exact synced skills commit
- `mode: waived`
  - the maintainer is intentionally shipping without a skills update
  - `waiver_reason` must explain why

## Why this exists

The Ota skill package is part of the product’s execution-governance surface.
If schema and governance behavior widen without a corresponding skill update or explicit waiver,
agents drift behind the platform while appearing current.

This gate makes that drift visible and reviewable.
