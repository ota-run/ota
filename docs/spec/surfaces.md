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

## What a surface is

A surface is a reusable runtime endpoint shape.

It is not operational on its own.
It becomes operational only when a service-task runtime attaches it.

Example:

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
```

That declares one reusable `backend` endpoint shape.
Tasks still decide whether they actually expose it.

## Why surfaces exist

Large app repos often repeat the same endpoint meaning several times:

- the same backend listener on `dev`, `dev:be`, `dev:ai`, and `start`
- the same editor listener on `dev` and `dev:fe`
- the same readiness path in several runtime blocks
- the same workflow expose URL repeated as a literal string

Surfaces let the contract define that endpoint once and attach it where it is actually published.

## Attach surfaces to tasks

Tasks attach declared surfaces through `tasks.<name>.runtime.surfaces`.

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
  editor:
    kind: http
    port: 8080
    path: /
    readiness:
      kind: http
      path: /
      timeout: 10000

tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - backend
        - editor

  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
```

Current behavior:

- each attached surface normalizes into the existing runtime listener model internally
- surface name becomes the normalized listener name
- the normalized listener uses conservative loopback defaults:
  - bind address `127.0.0.1`
  - fixed bind port `surface.port`
  - host projection `127.0.0.1:<surface.port>`
  - HTTP host projection path from `surface.path` or `/`
- if one runtime attaches exactly one surface, has no inline `runtime.readiness`, and that surface
  declares readiness, ota derives the equivalent runtime readiness automatically

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
        - editor
    exposes:
      - surface: backend
      - surface: editor

  backend:
    run:
      task: dev:be
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

## When to use surfaces

Use surfaces when:

- the same endpoint appears in more than one task
- readiness belongs to the endpoint and should not drift across tasks
- workflows should expose or prove one runtime endpoint without repeating literal URLs

Do not use surfaces when:

- the listener belongs to only one task and the loopback defaults are fine
- the endpoint is external or third-party
- the runtime needs task-specific bind or projection behavior

## Surfaces vs shorthand vs full listeners

- listener shorthand:
  - best for one task-local fixed loopback listener
  - example: `listeners.http.http: 3000`
- surfaces:
  - best for one reusable endpoint meaning shared by tasks and workflows
  - example: `surfaces.backend`
- full listeners:
  - best for custom bind addresses, projected host ports, primary selection, or non-default paths
  - example: `protocol` + `bind` + `project.host`

## Design rule

Surfaces are a reusable endpoint primitive, not a second runtime system.

The source of operational truth remains:

- top-level `surfaces` for reusable endpoint meaning
- task runtime attachment for operational publication
- workflow selection for which attached surfaces matter on that path
