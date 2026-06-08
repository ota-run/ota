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

# Surfaces

Use surfaces when the same runtime endpoint appears in more than one task or workflow and the
port, path, readiness, and exposed URL should stay as one named piece of contract truth.

The short version:

- listener shorthand reduces YAML for one task-local listener
- full listeners keep advanced bind and projection control
- surfaces remove repeated endpoint truth across several tasks and workflows

For one canonical contract that shows all three together, see
[`examples/full-contract/ota.yaml`](../../examples/full-contract/ota.yaml).

## Recommended learning path

Learn surfaces in this order:

1. native simple case
   - `runtime.surfaces: [backend]`
2. container publication case
   - `runtime.surfaces.backend.bind` / `project`
3. multi-surface app
   - `backend` + `frontend`, with one `project.host.primary: true`
4. workflow surface readiness and exposes
   - `readiness.surfaces` + `exposes: [{ surface: backend }]`
5. literal external URL
   - only for external, third-party, or otherwise non-Ota-owned endpoints

That order keeps the model honest:

- top-level `surfaces` own endpoint meaning
- task attachment owns runtime publication
- workflows consume attached surfaces instead of repeating URLs
- literal URLs stay the escape hatch for endpoints outside Ota topology

## What a surface is

A surface is a reusable runtime endpoint shape.

It is not operational on its own.
It becomes operational only when a service-task runtime attaches it.

Example:

```yaml
surfaces:
  backend:
    kind: http
    label: Backend API
    purpose: Primary application API for local development
    visibility: internal
    port: 5678
    path: /
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 10000
```

That declares one reusable `backend` endpoint shape.
Tasks still decide whether they actually expose it.

Optional surface metadata can improve command and topology UX without changing runtime behavior:

- `label`: short operator-facing name
- `purpose`: short explanation of why the surface exists
- `visibility`: `public` or `internal`

## Why surfaces exist

Large app repos often repeat the same endpoint meaning several times:

- the same backend listener on `dev`, `backend`, `worker`, and `start`
- the same frontend listener on `dev` and `frontend`
- the same readiness path in several runtime blocks
- the same workflow expose URL repeated as a literal string

Surfaces let the contract define that endpoint once and attach it where it is actually published.

## Attach surfaces to tasks

Tasks attach declared surfaces through `tasks.<name>.runtime.surfaces`.

### Native list form

Use the list form when the task can publish the surface with defaults:

```yaml
surfaces:
  backend:
    kind: http
    port: 5678
    path: /
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 10000
  frontend:
    kind: http
    port: 8080
    path: /
    readiness:
      kind: http
      path: /
      timeout: 10000

tasks:
  dev:
    launch:
      kind: command
      exe: pnpm
      args: [dev]
    runtime:
      kind: service
      surfaces:
        - backend
        - frontend

  backend:
    run: pnpm dev:backend
    runtime:
      kind: service
      surfaces:
        - backend

  frontend:
    run: pnpm dev:frontend
    runtime:
      kind: service
      surfaces:
        - frontend
```

Native list form is intentionally small.
The top-level surface owns endpoint meaning.
Each task only opts into the surfaces it actually publishes.
When that task is a long-running service, prefer `launch.kind: command` so Ota can reason about
the executable separately from the declared surface contract.

### Container attachment override

Use the object form when the runtime needs explicit publication:

```yaml
surfaces:
  site:
    kind: http
    port: 3000
    path: /
    readiness:
      kind: http
      path: /

tasks:
  dev:
    launch:
      kind: command
      exe: npm
      args: [run, dev]
    runtime:
      kind: service
      surfaces:
        site:
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
              path: /
              primary: true
```

Container-backed runtimes often need this form because the container bind address and the host
projection are not always the same endpoint.
The attachment owns only publication shape.
It does not redefine what the `site` surface means.

Read the object form like this:

- `runtime.surfaces.site` still refers to the declared top-level reusable `surfaces.site`
- object form means "attach that reusable surface with publication overrides for this runtime"
- `bind` means where the workload listens inside its own runtime context
- `project.host` means the host-facing projected endpoint ota reports, checks, and exposes

Attachment behavior:

- each attached surface normalizes into the existing runtime listener model internally
- surface name becomes the normalized listener name
- list form uses conservative defaults:
  - bind address `127.0.0.1`
  - fixed bind port `surface.port`
  - host projection `127.0.0.1:<surface.port>`
  - HTTP host projection path from `surface.path` or `/`
- object form is an attachment override:
  - it may shape `bind`
  - it may shape `project`
  - it may select `project.host.primary`
  - it must not override surface `kind`, readiness, or endpoint identity
- `kind: https` is also supported when the runtime already exposes HTTPS honestly; ota reuses the
  existing HTTPS listener protocol and the normal HTTP readiness request semantics instead of
  inventing certificate-management fields
- if one runtime attaches exactly one surface, has no inline `runtime.readiness`, and that surface
  declares readiness, ota derives the equivalent runtime readiness automatically
- if a runtime attaches multiple surfaces, has no inline `runtime.readiness`, and exactly one
  attached surface is marked `project.host.primary: true`, ota derives runtime readiness from that
  primary surface

### Multi-surface app

Use several surfaces when one task publishes more than one user-facing endpoint.
Mark the primary host projection when the task needs one canonical readiness target.

```yaml
surfaces:
  backend:
    kind: http
    port: 5678
    path: /
    readiness:
      kind: http
      path: /healthz/readiness
  frontend:
    kind: http
    port: 8080
    path: /
    readiness:
      kind: http
      path: /

tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        backend:
          project:
            host:
              primary: true
        frontend:
          project:
            host:
              path: /

  backend:
    run: pnpm dev:backend
    runtime:
      kind: service
      surfaces:
        - backend

  frontend:
    run: pnpm dev:frontend
    runtime:
      kind: service
      surfaces:
        - frontend
```

## Workflow readiness and exposes

Workflows can reference surfaces directly.

```yaml
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - backend
        - frontend
    exposes:
      - surface: backend
      - surface: frontend

  backend:
    run:
      task: backend
    readiness:
      surfaces:
        - backend
    exposes:
      - surface: backend
```

Current behavior:

- `workflows.<name>.readiness.surfaces` resolves through the workflow `run.task`
- validation rejects a workflow surface that the selected `run.task` does not attach
- `workflows.<name>.exposes` supports:
  - literal URL strings
  - object form `{ surface: <name> }`
- surface exposes resolve to the attached run-task host URL instead of repeating a hardcoded URL

## Literal external URLs

Use literal URL probes and exposes when the endpoint is external, third-party, or not owned by an
Ota task runtime.

```yaml
readiness:
  probes:
    billing-api:
      kind: http
      url: https://billing.example.com/health
      expect_status: 200
      timeout: 10000

checks:
  - name: billing-api-ready
    kind: health
    severity: warning
    probe: billing-api
```

Use surfaces for Ota-owned runtime endpoints.
Use literal URLs for endpoints that should stay outside Ota's topology model.

## When to use surfaces

Use surfaces when:

- the same endpoint appears in more than one task
- readiness belongs to the endpoint and should not drift across tasks
- workflows should expose or prove one runtime endpoint without repeating literal URLs
- operator-facing commands or topology JSON should preserve one short label or purpose for that
  endpoint

Do not use surfaces when:

- the listener belongs to only one task and the loopback defaults are fine
- the endpoint is external or third-party
- you need listener behavior beyond publication shaping

## Surfaces vs shorthand vs full listeners

- listener shorthand:
  - best for one task-local fixed loopback listener
  - example: `listeners.http.http: 3000`
- surfaces:
  - best for one reusable endpoint meaning shared by tasks and workflows
  - example: `surfaces.backend`
- attachment override:
  - best when a reusable surface needs task-specific bind, projection, or primary selection
  - example: `runtime.surfaces.backend`
- full listeners:
  - best for advanced topology beyond the surface attachment model
  - example: `protocol` + `bind` + `project.host`
- topology output:
  - preserves both the additive `surface_attachments` author intent and the normalized listener
    truth used operationally

## Design rule

Surfaces are a reusable endpoint primitive, not a second runtime system.

The source of operational truth remains:

- top-level `surfaces` for reusable endpoint meaning
- task runtime attachment for operational publication
- workflow selection for which attached surfaces matter on that path
