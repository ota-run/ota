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

# Hosted Validation Workflow

This document defines the V7 hosted-validation shape for using Ota in CI and pull-request
gating without mutating repo state.

The goal is to keep validation deterministic, non-mutating, and easy to consume by hosted
systems.

## Purpose

Hosted validation should:

- prove contract correctness
- surface readiness blockers early
- stay read-only
- emit stable JSON for automation
- avoid hidden repo or workspace mutation

## Recommended workflow

Use the following commands as the canonical hosted-validation stack:

1. `ota validate --json` for repo contract syntax and structure
1. `ota doctor --json` for repo readiness and actionable findings
1. `ota workspace validate --json` for workspace contract syntax and structure
1. `ota workspace doctor --json` for workspace readiness and per-repo findings

For workspace inventory and readiness summaries, `ota workspace list --json` can be used as a
lightweight preflight signal.

## Gating rules

Hosted validation should treat the following as failures:

- `ok: false` in a JSON payload
- any `error` or `errors` field from a contract-validation command
- any `severity: error` finding from `doctor` or workspace doctor output
- non-zero process exit when the command is expected to validate successfully

Warnings should be surfaced to humans, but they do not necessarily fail the gate unless policy
requires it.

## What hosted validation should not do

Hosted validation must not:

- run `ota init`
- run `ota detect --write`
- run `ota workspace init --bootstrap`
- mutate repo or workspace contracts as part of the validation step
- infer execution behavior from human-readable text output

## Example CI flow

```bash
#!/usr/bin/env bash
set -euo pipefail

ota validate --json | tee .ota-validate.json
ota doctor --json | tee .ota-doctor.json
ota workspace validate --json | tee .ota-workspace-validate.json
ota workspace doctor --json | tee .ota-workspace-doctor.json
```

Example PR policy:

- fail the job on any `ok: false`
- fail the job on any `severity: error`
- post warnings as annotations
- keep JSON artifacts for traceability

## Editor and hosted validation overlap

Hosted validation systems and editor integrations should consume the same JSON shapes:

- `ota validate --json`
- `ota doctor --json`
- `ota workspace validate --json`
- `ota workspace doctor --json`
- `ota workspace list --json`

That keeps PR gating, local diagnostics, and editor feedback aligned.
