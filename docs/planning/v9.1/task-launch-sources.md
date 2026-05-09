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

# Task Launch Sources

Status: planned branch-local design for `bobai/task-launch-sources`.

## Problem

Ota can already model:

- repo tasks
- runtime surfaces
- workflow path selection
- readiness and exposes

But famous adoption repos often present front doors like:

- `npx n8n`
- `docker run ...`
- `uvx ...`
- one packaged container image with a named volume and a published port

Today Ota can only express those as opaque shell-backed tasks. That is useful, but weakens the
product story:

- the contract hides the launch source behind a shell string
- users cannot see image/volume/port intent as first-class task truth
- Ota looks more like a wrapper around shell than a runtime/readiness contract

## Design rule

Do not put new launch sources under `workflows`.

Workflows remain selected operational paths. Tasks remain executable units.

The durable boundary is:

- `tasks` = executable units
- `tasks.<name>.launch` = how that unit starts
- `tasks.<name>.runtime.surfaces` = what the running unit exposes
- `workflows` = which task path is canonical for a use case

This preserves:

- `ota run`
- receipts
- agent safety boundaries
- task inputs/env/dependencies
- one execution model instead of task execution plus workflow execution

## Non-goals

- do not turn workflows into executable units
- do not replace existing `run` / `script` execution
- do not add a second runtime topology model next to surfaces/listeners
- do not add orchestration-wide compose or Kubernetes abstractions in this slice

## Contract shape

Current shell-backed tasks stay valid:

```yaml
tasks:
  dev:
    run: pnpm dev
```

New structured launch sources live on tasks:

```yaml
tasks:
  quickstart:
    launch:
      kind: command
      exe: npx
      args: [n8n]
    runtime:
      kind: service
      surfaces:
        - backend

  packaged:
    launch:
      kind: container
      image: docker.n8n.io/n8nio/n8n
      volumes:
        - kind: named
          source: n8n_data
          target: /home/node/.n8n
    runtime:
      kind: service
      surfaces:
        - backend

workflows:
  instant:
    intent: quickstart
    run:
      task: quickstart
    readiness:
      surfaces: [backend]
    exposes:
      - surface: backend

  docker:
    intent: packaged_runtime
    run:
      task: packaged
    readiness:
      surfaces: [backend]
    exposes:
      - surface: backend
```

## Launch kinds

### `launch.kind: command`

Purpose:

- model packaged command front doors explicitly instead of hiding them in `run`

Shape:

```yaml
launch:
  kind: command
  exe: npx
  args: [n8n]
```

Rules:

- `exe` is required and must not be empty
- `args` is optional
- task `env`, `inputs`, `depends_on`, `runtime`, and workflows continue to work normally
- execution should invoke the command directly as structured argv, not via shell interpolation

### `launch.kind: container`

Purpose:

- model packaged runtime images explicitly instead of hiding `docker run ...` in `run`

Initial shape:

```yaml
launch:
  kind: container
  image: docker.n8n.io/n8nio/n8n
  engine: docker
  args: []
  name: n8n
  remove: true
  volumes:
    - kind: named
      source: n8n_data
      target: /home/node/.n8n
```

Rules:

- `image` is required and must not be empty
- `engine` defaults to `docker`
- `volumes` is optional
- `name` is optional
- `remove` is optional and controls container cleanup after task exit

## Surface relationship

Surfaces remain the endpoint truth.

That means `launch.container` must not become a second place to declare:

- published host URLs
- readiness endpoints
- workflow exposes

Instead:

- `runtime.surfaces` declares the endpoint identity and publication intent
- `launch.container` declares the packaged image/runtime source

### Important boundary

For packaged container launch, attached surfaces drive publication.

The launch source must not duplicate host endpoint truth with another `ports` block unless Ota
cannot derive publication from the attached surfaces.

Initial planned restriction:

- packaged container launch requires attached service surfaces to resolve to fixed host
  projections
- automatic host port assignment is out of scope for the first slice

This keeps the source of truth clean:

- surface = published endpoint identity
- launch container = image and runtime source

## Validation rules

Base tasks:

- a task must declare exactly one execution source:
  - `run`
  - `script`
  - `launch`

Mode branches:

- each branch must declare at most one of:
  - `run`
  - `script`
  - `launch`
- if the task has no fallback execution, each branch must still resolve one execution source

Variants:

- variants should eventually follow the same rule
- if variant launch support is not in the first code slice, variants remain shell-only until the
  follow-up lands

Container launch:

- `launch.kind: container` requires a service runtime with at least one attached surface
- attached surfaces must resolve to fixed host publication in the first slice
- named volume `source` and `target` must not be empty

## Output contract

Task and workflow output should surface the structured launch source instead of collapsing it back
to shell-only wording.

Examples:

- `ota tasks`
  - show task kind plus launch source
  - examples: `command npx n8n`, `container docker.n8n.io/n8nio/n8n`
- `ota workflows`
  - still point to tasks
  - do not become executable directly
- `ota execution topology --json`
  - add task launch source details additively
- receipts
  - preserve the actual launch source used

## Implementation slices

### Slice 1

- add task-level `launch.kind: command`
- keep `run` / `script` backward-compatible
- wire validation, `ota tasks`, `ota workflows`, and `ota run`

### Slice 2

- add task-level `launch.kind: container`
- support named volumes
- derive publication from attached surfaces with fixed host projections
- surface launch details in topology/receipts

### Slice 3

- expand container launch support only if the first slices prove the model:
  - bind volumes
  - richer engine support
  - explicit lifecycle cleanup knobs

## Acceptance pressure tests

The design is successful when Ota can model famous repo front doors honestly:

- source repo contributor path:
  - `pnpm dev`
- packaged command path:
  - `npx n8n`
- packaged container path:
  - `docker run ... docker.n8n.io/n8nio/n8n`

without:

- moving execution into workflows
- duplicating endpoint truth outside surfaces
- hiding packaged launch intent inside opaque shell strings
