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

# V1.0.0 Release Checklist

Status: active.

This checklist defines the minimum work that should be complete before the first major public release.

`v1.0.0` is the boundary where Ota becomes a stable public promise for repo readiness. It should not ship until the public story, docs, and core UX are aligned.

## Release intent

- Open-core repo readiness is the product boundary
- `doctor`, `detect`, `init`, `up`, and `run` are the core adoption wedge
- Enterprise dashboard/control-plane work remains a later phase
- The public docs must describe the shipped product, not future ambitions

## Must be complete

- landing page copy and structure
- README introduction and quickstart aligned to the landing page
- docs home / command reference aligned to the same public story
- install path and first-run path confirmed
- `ota doctor`, `ota detect`, `ota init`, `ota up`, and `ota run` examples are current
- JSON output examples and schemas match the shipped contract
- release notes explain what `v1.0.0` means and what is intentionally deferred

## Must stay green

- `cargo test --quiet --lib -- --test-threads=1`
- `cargo test --quiet --tests -- --test-threads=1`
- `cargo fmt --check`
- release workflow on GitHub

## Compatibility bar

- no unannounced JSON breaking changes
- no breaking command semantics without a migration note
- no hidden repo/workspace behavior drift
- no enterprise control-plane promises in the public `v1.0.0` story

## Ship sequence

1. Freeze the current core release line
2. Finish landing page and documentation alignment
3. Re-validate the public install and first-run path
4. Cut `v1.0.0`
5. Start the enterprise phase separately

