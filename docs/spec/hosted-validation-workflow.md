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

This document defines the V7 hosted-validation shape for using ota in CI and pull-request
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
1. `ota workspace explain --json` when you need ordered workspace remediation for blockers
1. `ota workspace list --json` when you want lightweight workspace inventory and readiness

For workspace inventory and readiness summaries, `ota workspace list --json` can be used as a
lightweight preflight signal. For ticketing or automated follow-up, `ota workspace explain --json`
gives an ordered plan without mutating state.

## Infrastructure boundary

ota does not replace the CI runner or its service provisioning layer.

For example, if your GitHub Actions job uses a Postgres service container, GitHub Actions still
starts that container. ota removes the repo-specific duplication above it:

- contract validation
- readiness diagnosis
- task execution
- env and service intent declared once in `ota.yaml`

That means the CI workflow stays thin, while the repo contract carries the real requirements.

## Gating rules

Hosted validation should treat the following as failures:

- `ok: false` in a JSON payload
- `summary.error_count > 0` in `ota validate --json` or `ota workspace validate --json`
- any `error` or `errors` field from a contract-validation command
- any `severity: error` finding from `doctor` or workspace doctor output
- any `severity: error` finding from `ota workspace explain --json` when the plan is being used as a gate
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
ota workspace list --json | tee .ota-workspace-list.json
ota workspace explain --json | tee .ota-workspace-explain.json
```

## Example with a Postgres service container

GitHub Actions still provisions the database container; ota removes the duplicate repo setup
above it.

```yaml
name: ci

on:
  push:
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest

    services:
      postgres:
        image: postgres:16
        env:
          POSTGRES_USER: postgres
          POSTGRES_PASSWORD: postgres
          POSTGRES_DB: app
        ports:
          - 5432:5432
        options: >-
          --health-cmd="pg_isready -U postgres"
          --health-interval=10s
          --health-timeout=5s
          --health-retries=5

    steps:
      - uses: actions/checkout@v4
      - name: Install ota
        run: curl -fsSL https://dist.ota.run/install.sh | sh
      - name: Validate contract
        run: ota validate
      - name: Diagnose readiness
        run: ota doctor --json
      - name: Prepare repo
        run: ota up
      - name: Run lint
        run: ota run lint
      - name: Run tests
        run: ota run test
```

## Example with ota-provisioned Postgres

In this shape, the repo contract owns the database service and the CI job stays thin.
The repo is expected to carry the matching Compose definition; ota only models and runs the
service boundary declared in `ota.yaml`.

```yaml
version: 1
project:
  name: app

services:
  postgres:
    required: true
    provider: docker-compose
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -h localhost -p 5432

env:
  DATABASE_URL:
    required: true

tasks:
  lint:
    run: npm run lint
  test:
    run: npm test
```

```yaml
name: ci

on:
  push:
  pull_request:

jobs:
  ci:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4
      - name: Install ota
        run: curl -fsSL https://dist.ota.run/install.sh | sh
      - name: Validate contract
        run: ota validate
      - name: Prepare repo
        run: ota up
      - name: Run lint
        run: ota run lint
      - name: Run tests
        run: ota run test
```

In this model, ota starts and validates the Compose service declared in `ota.yaml`; the runner
does not duplicate the Postgres setup.

Example PR policy:

- fail the job on any `ok: false`
- fail the job on any `severity: error`
- post warnings as annotations
- keep JSON artifacts for traceability

## CI and PR annotation delivery

The annotation layer should be a thin consumer of JSON, not a second diagnosis engine.

Recommended mapping:

- use `summary.primary_blocker` as the first check summary when present
- emit one annotation per finding
- map `severity: error` to failing annotations or a failed check
- map `severity: warn` to non-blocking annotations unless policy says otherwise
- use `summary` as the check-run headline
- use `why` as the annotation body
- use `next` as the suggested fix or link target

For workspace commands, keep the same mapping but scope annotations to the repo name and path in
the workspace payload. That keeps PR feedback aligned with the same JSON fields used by local
editor integrations.

Portable adapter:

`ota annotations` is the canonical JSON-to-CI adapter. It turns ota JSON into either plain CI log
lines or GitHub Actions annotations. Use `--format plain` when you want a provider-neutral output
stream, or `--format github` when the CI platform understands annotation syntax.

For repo-local usage, the canonical entrypoint is `ota run doctor-annotations`, which now shells
out to `ota annotations` under the hood with the `--render-format` input set to `plain` or
`github`.

Example shell adapter for GitHub Actions:

```bash
#!/usr/bin/env bash
set -euo pipefail

ota doctor --json | ota annotations --mode doctor --format github --input -

ota workspace doctor --json | ota annotations --mode workspace-doctor --format github --input -
```

## Editor and hosted validation overlap

Hosted validation systems and editor integrations should consume the same JSON shapes:

- `ota validate --json`
- `ota doctor --json`
- `ota workspace validate --json`
- `ota workspace doctor --json`
- `ota workspace explain --json`
- `ota workspace list --json`

That keeps PR gating, local diagnostics, and editor feedback aligned.
