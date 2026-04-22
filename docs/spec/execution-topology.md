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
   License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Execution Topology

Status: evolving. The execution-topology foundation is shipped: `execution.default_context`, `execution.contexts`, `tasks.<name>.context`, `tasks.<name>.requires_services`, task-scoped workload listeners via `tasks.<name>.runtime.kind: service`, typed Compose service managers, context-scoped `services.<name>.endpoints`, `services.<name>.readiness.from`, and Compose-network attachment for container contexts. Broader manager coverage and deeper topology validation are still in progress.

## Core truth

Ota does not have a backend problem. Ota has a topology problem.

Today Ota knows:

- where tasks run

Today Ota does not know:

- where services are controlled from
- where services are reachable from
- which tools belong to which execution plane
- where readiness should be evaluated from

That gap is why mixed host-plus-container repos drift into contradictions:

- `docker compose` is a host control-plane concern
- `mvn test`, `pnpm dev`, or `uv run pytest` are workload-plane concerns
- a database may be reachable on a service network from one context and unreachable from another
- the same repo may honestly need `docker` on the host and not inside the app container

The current single repo-wide execution story is too coarse for that reality.

## Design goal

Model the repo as execution topology, not as one flat backend choice.

The current shipped model makes these things first-class:

1. execution contexts
2. typed service managers
3. task-level service requirements
4. task-level workload listeners
5. context-scoped endpoint projection
6. context-scoped readiness
7. context-scoped requirements

## Design principles

- Keep the control plane separate from the workload plane.
- Never make `localhost` mean the same thing everywhere.
- Do not run service-manager CLIs inside workload containers by default.
- Prefer explicit topology over Docker-specific heuristics.
- Keep `doctor`, `up`, `run`, JSON output, and receipts on one topology truth.

## Proposed contract model

### `execution.contexts` model

Execution contexts define where workloads run.

Each context answers:

- which backend it uses
- which lifecycle it uses when relevant
- which image or target it uses when relevant
- which service networks it can attach to
- which runtimes and tools belong to that context

Proposed direction:

```yaml
execution:
  default_context: app

  contexts:
    host:
      backend: native
      requirements:
        tools:
          docker: "*"
          lsof: "*"

    app:
      backend: container
      lifecycle: persistent
      container:
        image: maven:3.9.14-eclipse-temurin-21-noble
      attachments:
        compose:
          - local
      requirements:
        runtimes:
          java: ">=21"
          node: ">=24.14.1"
        tools:
          maven: "*"
```

This keeps `docker` on the host context instead of pretending the app container should carry it.

### `tasks.<name>.context`

Tasks bind to a context or inherit `execution.default_context`.

```yaml
tasks:
  compose:up:
    context: host
    run: docker compose up -d

  compose:down:
    context: host
    run: docker compose down -v

  stop:
    context: host
    run: lsof -ti:8080 | xargs kill -9 || true

  setup:
    context: app
    run: mvn -q -DskipTests dependency:go-offline

  db:integration:
    context: app
    requires_services:
      - postgres
    run: mvn -B -Dgroups=db-integration test

  build:
    context: app
    run: mvn package

  test:
    context: app
    run: mvn test
```

This makes orchestration tasks and workload tasks honest without forcing Docker into the app image.
`requires_services` lets a task declare that canonical services must be ready before its body runs, while ownership still stays with `services.<name>.manager`.

### `tasks.<name>.execution.modes`

Use mode branches when one task intent should run across multiple execution planes without duplicating task names.

```yaml
tasks:
  start:
    execution:
      default_mode: container
      modes:
        native:
          context: host
          env:
            DB_URL: jdbc:postgresql://127.0.0.1:5432/app
          run: mvn spring-boot:run
        container:
          context: app
          lifecycle: persistent
          env:
            DB_URL: jdbc:postgresql://postgres:5432/app
          run: mvn spring-boot:run -Dspring-boot.run.arguments=--server.address=0.0.0.0,--server.port=8080
```

This keeps task identity stable:

- `ota run start` uses `default_mode` when declared
- `ota run start --mode native` selects `modes.native`
- `ota run start --mode container` selects `modes.container`
- mode branches can override `context`, `lifecycle`, `env`, `run`/`script`, and `runtime`
- if a selected mode branch is missing, ota fails clearly instead of guessing another mode

### `execution.contexts.<name>.attachments.isolated_paths`

Container contexts can keep platform-sensitive dependency trees isolated from the host tree by declaring workspace-relative paths that Ota should back with engine-managed named volumes.

```yaml
execution:
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: node:24
      attachments:
        compose:
          - local
        isolated_paths:
          - node_modules
```

This is the right boundary for dependency trees like `node_modules`, `.venv`, or other install artifacts that should be built for the container platform instead of inheriting host-native binaries.
Source still stays bind-mounted from the host at `/workspace`; only the declared dependency paths are overlaid with named volumes.
Ota labels those volumes with a stable repo ownership token stored under `.ota/ownership-id`, and `ota clean` rediscovers and removes them even if the repo path, image, engine, or declared isolated paths drift later.
Ota also records repo-used engines under `.ota/managed-engines` so `ota clean` can keep drift cleanup scoped to the engines that actually owned this repo's managed state.

### `tasks.<name>.runtime.kind: service`

Long-running app workloads should keep their ingress declaration with the task that owns the process.

```yaml
tasks:
  dev:
    context: app
    requires_services:
      - postgres
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: auto
              path: /
        metrics:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 9090
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /metrics
```

This keeps the boundary honest:

- `services` still model dependencies like Postgres
- `tasks.<name>.runtime.listeners` model app ingress like a dev server or Spring Boot API
- with multiple projected listeners, mark exactly one listener as `project.host.primary: true`; ota uses that listener for `OTA_PUBLIC_URL` and the primary endpoint line
- in container contexts with `project.host.port.mode: auto`, ota injects runtime URL env values before command execution; ephemeral runs pre-reserve host ports before start, while persistent runs reconcile named containers and then resolve the published mapping
- `ota run <task> --host-port <port>` overrides one run's published host/public port when the selected projected listener is `project.host.port.mode: fixed`; app bind ports stay unchanged
- `ota run dev` records the same resolved host endpoint ota injected into runtime env
- `ota up` only reports workload endpoints for runtime-bearing tasks it actually executes during preparation today; it does not yet discover arbitrary app tasks like `dev`
- readiness remains separate from ingress projection

Ingress troubleshooting:

- `task <name> declares multiple projected listeners ... but none sets project.host.primary: true`: set `project.host.primary: true` on exactly one projected listener
- `task <name> declares multiple listeners with project.host.primary: true`: keep one primary and remove the rest
- `loopback-only container bind address`: change the bind address to `0.0.0.0` before projecting to host
- `could not publish host port`: keep `project.host.port.mode: auto` and rerun; for ephemeral runs ota retries bounded times and then fails clearly when host publication still conflicts
- ``--host-port`` rejected: use it only with container tasks that project exactly one selected listener (`project.host.primary: true` when multiple) and keep that listener on `project.host.port.mode: fixed`

### `services.<name>.manager`

Services should be managed through typed manager blocks when topology matters.

```yaml
services:
  postgres:
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
```

Typed managers let Ota reason about:

- the control-plane tool it needs
- the project or cluster identity
- the network or namespace boundary
- the real service name, not just a shell snippet

Legacy `provider/start/stop/healthcheck` fields should remain as compatibility mode, but they stay a weaker host-bound model.

### `services.<name>.endpoints`

A service must declare how it is reached from each context that matters.

```yaml
services:
  postgres:
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
      app:
        address: postgres
        port: 5432
```

This makes the communication truth explicit:

- `127.0.0.1:5432` may be valid from the host
- `postgres:5432` may be valid from the app container
- they are different truths and Ota should model both

For typed Compose managers, Ota may infer default endpoints when unambiguous, but the contract surface must exist so ambiguity can be expressed explicitly.

### `services.<name>.readiness.from`

Readiness probes must say where they run from.

```yaml
services:
  postgres:
    readiness:
      from: app
      run: pg_isready -h postgres -p 5432
```

This replaces the current host-bound limitation with a truthful topology model:

- if the workload runs from `app`, readiness should usually be checked from `app`
- if the workload runs from `host`, readiness should usually be checked from `host`

### Context-scoped requirements

Requirements must stop being one flat repo-wide bucket.

- host-scoped tools such as `docker`, `podman`, `kubectl`, or `lsof` belong to host-like contexts
- app-scoped runtimes and tools such as `java`, `node`, `maven`, `pnpm`, or `uv` belong to workload contexts

That keeps `doctor` honest and prevents false failures like “docker missing from the app container” when the real requirement is host control-plane availability.

## Runtime semantics

### `ota doctor`

`doctor` should:

- resolve the effective workload context
- validate requirements for each referenced context
- validate service-manager availability in the control plane
- validate endpoint projection for referenced services
- surface `attachments.isolated_paths` in execution summaries so operators can see when a container context owns its dependency tree
- validate readiness from the declared context
- fail with topology errors when the contract describes an impossible communication path

Example topology error:

- task context is `app`
- service endpoint is only declared for `host`
- readiness is declared from `app`
- no projection from `app` exists

That should fail explicitly instead of falling back to a guessed host path.

### `ota up`

`up` should:

1. resolve the contexts needed by the setup flow
2. start required services through their managers in the control plane
3. ensure workload attachments are valid
4. run `setup` in the task context with the same dependency-isolation mounts that task context declares
5. re-check readiness from the declared contexts
6. surface workload endpoints for any runtime-bearing task `ota up` actually executes during preparation, without pretending arbitrary app tasks were started

### `ota run`

`run` should:

- resolve the task context
- attach the workload to declared service topology when needed
- mount any declared dependency-isolation paths before process start when the task runs in a container context
- inject resolved runtime URL env values before process start when the projection is known
- execute inside that context
- record the resolved workload endpoint when the task declares one
- verify container host publication matches reserved host projection ports
- retry bounded times for `project.host.port.mode: auto` when host-port conflicts occur in ephemeral container runs
- emit receipts that report the resolved context and attached topology

## Container task semantics

For a container workload context attached to Compose:

1. resolve the Compose manager
2. resolve the Compose project and network identity
3. ensure required services from that manager are running
4. run the task container attached to the declared network
5. let the app talk to `postgres:5432` by service name

That is the correct model.

The correct model is not:

- Docker inside the app image
- Docker socket hacks
- `host.docker.internal` as the main model
- published host ports as the main model

## Example 1: app container reaches Compose-managed Postgres

```yaml
version: 1
project:
  name: qredex-core

execution:
  default_context: app
  contexts:
    host:
      backend: native
      requirements:
        tools:
          docker: "*"
    app:
      backend: container
      lifecycle: persistent
      container:
        image: maven:3.9.14-eclipse-temurin-21-noble
      attachments:
        compose:
          - local
      requirements:
        runtimes:
          java: ">=21"
          node: ">=24.14.1"
        tools:
          maven: "*"

services:
  postgres:
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
    endpoints:
      app:
        address: postgres
        port: 5432
    readiness:
      from: app
      run: pg_isready -h postgres -p 5432

tasks:
  compose:up:
    context: host
    run: docker compose up -d postgres
  compose:down:
    context: host
    run: docker compose down -v
  setup:
    context: app
    run: mvn -q -DskipTests dependency:go-offline
  test:
    context: app
    run: mvn test
```

In this example:

- Compose is controlled from the host
- the app workload runs in the container context
- the app workload attaches to the Compose network
- Postgres is reached by service name, not through the host

## Example 2: app container reaches host-managed Postgres

```yaml
version: 1
project:
  name: qredex-core

execution:
  default_context: app
  contexts:
    host:
      backend: native
      requirements:
        tools:
          lsof: "*"
    app:
      backend: container
      lifecycle: persistent
      container:
        image: maven:3.9.14-eclipse-temurin-21-noble
      requirements:
        runtimes:
          java: ">=21"
        tools:
          maven: "*"

services:
  postgres:
    manager:
      kind: host
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
      app:
        address: host.docker.internal
        port: 5432
    readiness:
      from: app
      run: pg_isready -h host.docker.internal -p 5432

tasks:
  setup:
    context: app
    run: mvn -q -DskipTests dependency:go-offline
  test:
    context: app
    run: mvn test
```

In this example:

- Postgres is a host-managed service
- the app still runs in a container
- the app does not guess `localhost`
- the app context uses the explicitly projected host endpoint

## What is weaker

These designs are weaker:

1. repo-level backend only
2. task-level backend override only
3. auto-guess Compose network and attach implicitly
4. run `docker compose` inside the app container
5. use `host.docker.internal` plus published ports as the main model

Each of those avoids modeling topology truth directly.

## Migration path

Keep current fields as compatibility mode:

- `execution.preferred`
- `execution.supported`
- `services.provider`
- `services.start`
- `services.stop`
- `services.healthcheck`

Compatibility interpretation:

- legacy contracts use a single-context model
- legacy service fields remain host-bound unless upgraded to typed topology-aware service blocks
- task-scoped workload listeners are canonical on the named-context topology model and should not be backported into legacy service semantics

Warn when topology-sensitive container usage mixes with legacy service semantics.

Example warning:

- task context resolves to container
- service uses legacy host-bound `healthcheck`
- no endpoint projection exists for that container context

Ota should warn that the contract is ambiguous for container workloads and recommend an explicit topology upgrade.

## Acceptance bar

The design is only done when all of these are true:

- `compose:up` runs on the host
- `setup`, `build`, `test`, and `ci` run in the container workload context
- the app container reaches `postgres` by service name on the shared network when Compose manages the service
- host-managed databases are declared with explicit context-projected endpoints
- `docker` is required only for the host context
- `java`, `node`, `maven`, and similar tools are required only for the workload context that uses them
- `ota doctor`, `ota up`, `ota run`, JSON output, and receipts all report the same topology truth
- impossible topologies fail explicitly instead of “trying stuff”
