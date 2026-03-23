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

# V1 Release Gate

Status: satisfied and frozen as the V1 release record.

This document is the binding release bar for calling Ota V1 done.

V1 is not complete because a feature list looks good. V1 is complete when the shipped contract and command surface hold against the real-fixture matrix below.

## Binding test gate

These must stay green:

- `cargo test`
- `cargo test --test detect_fixtures`
- `cargo test --test real_repo_fixtures`
- `cargo test --test examples_validate`
- `cargo fmt --check`

These commands are now bound into CI via:

- `.github/workflows/v1-release-gate.yml`

## Binding real-fixture matrix

Current real fixtures under `tests/fixtures/real`:

- `docker-heavy-node`
- `docker-legacy`
- `java-gradle`
- `java-gradle-multimodule`
- `java-maven`
- `node-conflict-monorepo`
- `polyglot-ops`
- `ugly-polyglot`

## Required command confidence by fixture class

Detection gate:

- `ota init --json` must remain honest about `blank` vs `detected`
- `ota detect --json --dry-run` must remain provenance-first
- precedence behavior must stay stable on conflicting signals

Readiness gate:

- `ota doctor --json` must stay accurate on env, service, lifecycle, and check findings
- `ota up` must preserve phase semantics and child exit behavior
- required-service readiness must gate dependent progress

Execution gate:

- `ota run` must preserve child exit codes
- example contracts must remain valid
- execution behavior must stay shell-native and deterministic

## Binding quality criteria

V1 is ready when:

- all gate suites pass
- the real-fixture matrix spans the shipped core commands, not just detection
- `ota doctor` shows no known blocking false positives on the gated fixtures
- `ota detect` and `ota init` remain conservative and reviewable
- JSON schema files remain aligned with actual command output
- the exit-code registry remains aligned with actual command behavior
- contract, compatibility, and shell semantics docs remain aligned with implementation

## What is not enough

These do not count as V1 readiness by themselves:

- command count
- toy examples only
- docs without executable gate coverage
- passing unit tests without real-fixture pressure

## Current status

This gate is satisfied in the current repository state and now serves as the frozen V1 release record.
