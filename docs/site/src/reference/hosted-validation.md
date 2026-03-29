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

# Hosted Validation

Use hosted validation when Ota needs to gate a pull request or CI run without mutating the repo.

## What to run

- `ota validate --json`
- `ota doctor --json`
- `ota workspace validate --json`
- `ota workspace doctor --json`
- `ota workspace list --json` for inventory and readiness summary

## Infrastructure boundary

GitHub Actions or your CI runner still provisions infrastructure such as service containers.
Ota removes the duplicated repo logic above it by keeping validation, readiness, env intent, and
task execution in the contract.

## What to fail on

- `ok: false`
- any `error` or `errors`
- any `severity: error`
- non-zero exit when validation is expected to pass

## What not to do

- do not run `ota init`
- do not run `ota detect --write`
- do not run `ota workspace init --bootstrap`
- do not infer behavior from human text output

## Example

```bash
#!/usr/bin/env bash
set -euo pipefail

ota validate --json | tee .ota-validate.json
ota doctor --json | tee .ota-doctor.json
ota workspace validate --json | tee .ota-workspace-validate.json
ota workspace doctor --json | tee .ota-workspace-doctor.json
```

## Example with Postgres

GitHub Actions still provisions the database service container. Ota removes the repo-specific
setup above it.

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
      - name: Install Ota
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

## Example with Ota-managed Postgres

In this shape, the repo contract owns the database service and the CI job stays thin.

```yaml
version: 1
project:
  name: app

services:
  postgres:
    image: postgres:16
    env:
      POSTGRES_USER: postgres
      POSTGRES_PASSWORD: postgres
      POSTGRES_DB: app
    ports:
      - 5432:5432
    healthcheck:
      command: pg_isready -U postgres

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
      - name: Install Ota
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

Ota starts and validates the service declared in `ota.yaml`; the runner does not duplicate the
Postgres setup.

Hosted validation is read-only. It surfaces blockers early and leaves mutation to local,
explicit commands.
