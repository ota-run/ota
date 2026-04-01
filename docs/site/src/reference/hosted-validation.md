# Hosted Validation

Hosted validation is the read-only CI path for ota.

Use it when you want to gate a pull request or CI run without mutating the repo.

## Source model

This page is the canonical public reference for hosted validation. It adds
examples, use cases, and operator guidance so the page stands on its own while
staying aligned with shipped behavior.

## What it is for

Use hosted validation when you need:

- a deterministic gate before merge
- machine-readable readiness data in CI
- repo, workspace, and task findings without local mutation
- annotations or check summaries derived from ota JSON

It is the right surface when the repo should be judged, not changed.

## What it checks

- `ota validate --json`
- `ota doctor --json`
- `ota check --json`
- `ota workspace validate --json`
- `ota workspace doctor --json`
- `ota workspace check --json`
- `ota workspace explain --json` for ordered workspace remediation
- `ota workspace list --json` for inventory and readiness summary

These commands answer different questions:

- `validate` checks contract structure
- `doctor` diagnoses readiness
- `check` runs checks-only readiness
- `workspace doctor` diagnoses the workspace as an orchestration layer
- `workspace check` runs checks-only workspace readiness
- `workspace explain` turns workspace findings into remediation steps
- `workspace list` gives inventory and readiness summary data

## Use cases

- a pull request needs a blocking readiness check before merge
- a CI pipeline wants annotations instead of raw JSON
- a workspace gate needs to show which repo is blocking the roll-up
- a platform team wants consistent validation results across repos
- a repo owner wants a machine-readable summary without mutating local state

## Infrastructure boundary

GitHub Actions or your CI runner still provisions infrastructure such as service containers.
ota removes the duplicated repo logic above it by keeping validation, readiness, env intent, and
task execution in the contract.
That means ota can provision declared repo services through `ota up`, but it does not replace the
CI runner, OS package manager, or language installer on the host.

## Example CI flow

```yaml
name: ci

on:
  pull_request:

jobs:
  validate:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4
      - name: Install ota
        run: curl -fsSL https://dist.ota.run/install.sh | sh
      - name: Validate contract
        run: ota validate --json | tee .ota-validate.json
      - name: Diagnose readiness
        run: ota doctor --json | tee .ota-doctor.json
      - name: Render annotations
        run: ota doctor --json | ota annotations --mode doctor --format github --input -
```

That keeps the CI job thin and lets ota own the repo-readiness logic.

## What to fail on

- `ok: false`
- `summary.error_count > 0` for `ota validate --json` and `ota workspace validate --json`
- any `error` or `errors`
- any `severity: error`
- any `severity: error` from `ota workspace explain --json` when used as a gate
- non-zero exit when validation is expected to pass

## What not to do

- do not run `ota init`
- do not run `ota detect --write`
- do not run `ota workspace init --bootstrap`
- do not infer behavior from human text output

## Practical example with Postgres

If the repo contract declares the database service, hosted validation should only validate and
diagnose. It should not duplicate service setup in the workflow.

## Example

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

## Example with ota-managed Postgres

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

ota starts and validates the service declared in `ota.yaml`; the runner does not duplicate the
Postgres setup.

Hosted validation is read-only. It surfaces blockers early and leaves mutation to local,
explicit commands.

## Annotation delivery

Hosted CI should treat the JSON payload as the source of truth and turn it into annotations or
check-run summaries without re-parsing text output.

Recommended mapping:

- use `summary.primary_blocker` as the headline when present
- emit one annotation per finding
- treat `severity: error` as blocking
- treat `severity: warn` as non-blocking unless policy says otherwise
- use `why` as the annotation body
- use `next` as the suggested fix

For workspace commands, keep the same mapping but scope annotations to the repo name and path in
the workspace payload.

Portable adapter:

`ota annotations` is the canonical JSON-to-CI adapter. It turns ota JSON into either plain CI log
lines or GitHub Actions annotations. Use `--format plain` for any provider, or `--format github`
when the CI platform understands annotation syntax.

For repo-local usage, the canonical entrypoint is `ota run doctor-annotations`, which now shells
out to `ota annotations` under the hood with the `--render-format` input set to `plain` or
`github`.

Example adapter:

```bash
ota doctor --json | ota annotations --mode doctor --format github --input -
ota workspace doctor --json | ota annotations --mode workspace-doctor --format github --input -
```
