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

# Service Behavior

ota services are supporting processes, databases, or local infrastructure that the repo needs in
order to become ready. This page explains how ota treats them in `doctor`, `services`, and `up`.

Use this page when you need to know:

- what service fields mean
- when a service is treated as blocking
- how readiness is checked
- what `ota up` does before setup
- what `ota detect` can infer from Compose files

## Why services matter

Services are how ota turns “the app needs Postgres, Redis, or a queue” into explicit contract data
instead of tribal knowledge.

That matters because:

- `ota doctor` can tell you whether the repo is actually ready
- `ota up` can bring required services up before setup runs
- optional services can be visible without becoming blockers
- CI and agents can see startup dependencies instead of guessing

## What lives in the service block

Current service fields are:

- `required`: whether the service must be ready before the repo is considered ready
- `provider`: where the service is managed from, such as Docker Compose
- `start`: the command ota uses to start it
- `stop`: the command ota uses to stop it
- `healthcheck`: the command ota uses to verify it is ready
- `depends_on`: service dependencies that must start first
- `timeout`: the healthcheck timeout in milliseconds

At least one actionable field must be present:

- `provider`
- `start`
- `stop`
- `healthcheck`

## How ota uses services

### `ota services`

Use `ota services` when you want to inspect the declared service surface without starting
anything.

It shows:

- which services exist
- whether they are required
- how ota expects to start and check them
- what dependency order applies

### `ota doctor`

Use `ota doctor` when you want to know whether the services are actually usable.

It checks:

- declared healthchecks
- whether required services are missing a healthcheck
- whether healthchecks time out
- whether Docker Compose services are reachable when that provider is used

Required service failures block readiness. Optional service failures become warnings.

### `ota up`

Use `ota up` when you want ota to make the repo runnable.

It:

- validates the contract first
- starts required services and their required dependencies in order
- waits for required healthchecks before setup runs
- stops at the services phase if the repo still is not ready

### `ota detect`

Use `ota detect` when you want ota to infer a starter service contract from Compose files.

Current Compose filenames ota recognizes include:

- `docker-compose.yml`
- `docker-compose.yaml`
- `compose.yml`
- `compose.yaml`

For Compose-based repos, ota can infer:

- `provider`
- `start`
- `stop`
- `healthcheck.test`

## Practical example

```yaml
services:
  postgres:
    required: true
    provider: docker-compose
    start: docker compose up -d postgres
    stop: docker compose stop postgres
    healthcheck: pg_isready -h localhost -p 5432
    timeout: 5000
  redis:
    required: false
    provider: docker-compose
    start: docker compose up -d redis
    stop: docker compose stop redis
    healthcheck: redis-cli ping
```

In this example:

- `postgres` blocks readiness until it passes healthcheck
- `redis` is visible and managed, but it does not block the repo if it fails
- `timeout` makes the readiness expectation explicit

## Use cases

- a repo needs a database before tests can run
- a team wants optional local infra visible without making it blocking
- a maintainer wants `up` to bring services up before `setup`
- CI needs to surface service drift as a clear readiness issue

## What ota does not do

- ota does not invent services that are not declared
- ota does not guess dependency ordering beyond `depends_on`
- ota does not provide deep service orchestration
- ota does not replace your OS package manager or container runtime

## Related docs

- [Commands](commands.md)
- [Contract](contract.md)
- [Shell semantics](shell-semantics.md)
- [Audit and provenance](audit-and-provenance.md)
