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

# Multi-Workload Shared Local Backends

Status: proposed.

This spec defines the follow-on topology feature that lets one shared local backend host multiple
distinct long-running workloads honestly.

It extends:

- [execution-topology.md](execution-topology.md)
- [local-service-topology.md](local-service-topology.md)

It does not replace the current shared-local-backend model. It relaxes that model by separating
what is truly backend-shared from what is workload-local.

## Core truth

The current shared local backend model is strict:

- one backend
- one deterministic container shape
- one shared publication/listener shape

That is intentionally safe, but it is too narrow for real local stacks like:

- API on `8080`
- helper app or sandbox on `8787`
- both should share one local workbench/backend
- both should keep distinct listeners and publications

The missing concept is:

- one backend boundary
- multiple workload identities inside it

Ota should model that directly instead of forcing repos to choose between:

- fake co-location
- host-bridge workarounds
- or giving up on a shared backend entirely

## Product goal

Let multiple long-running tasks share one Ota-managed local backend while keeping distinct:

- commands
- listeners
- publications
- readiness
- task identity

At the same time, keep one shared:

- backend identity
- lifecycle
- fulfillment unit
- environment/profile resolution
- filesystem/workbench state

## Design goals

1. Preserve one backend identity while allowing multiple workload identities.
2. Keep service identity task-scoped.
3. Keep backend fulfillment backend-scoped.
4. Let target bindings resolve topology honestly inside the shared boundary.
5. Preserve declared-versus-effective evidence for both backend and workload surfaces.
6. Extend the current model instead of replacing it.

## Non-goals

- no remote orchestration redesign
- no generic multi-container compose replacement
- no silent weakening of validation
- no guessed internal addresses
- no repo-local bootstrap glue as the primary solution

## First-principles model

Today, shared backend validation effectively treats these as backend-global:

- image/profile
- lifecycle
- publications
- listeners
- memory
- dependency-isolation shape

That is too coarse.

The correct separation is:

### Backend-shared truth

- backend family
- lifecycle
- effective environment/profile/image
- fulfillment mode
- filesystem/workbench state
- compatible dependency-isolation contract
- compatible resource envelope

### Workload-local truth

- task command
- service listeners
- host publications
- readiness
- task env
- task identity

This is the heart of the feature.

## Contract direction

Keep the existing task-level binding model:

```yaml
execution:
  shared_backends:
    workbench:
      scope: local
      backend: container
      lifecycle: persistent
      context: app
      environment:
        profile: java-node-workbench
      fulfillment: run

tasks:
  dev:
    runtime:
      kind: service
      backend_binding: workbench

  sandbox:
    runtime:
      kind: service
      backend_binding: workbench
```

Do not introduce backend-declared workload slots yet.

The stronger extension is:

- keep `tasks.<name>.runtime.backend_binding`
- treat listeners/publications as workload-local within that backend
- keep backend environment and fulfillment on `execution.shared_backends.<name>`

That extends the current model cleanly without a second abstraction layer.

## Validation rules

Validation should split into two classes.

### Backend compatibility validation

All tasks bound to one local backend must still agree on:

- backend family
- lifecycle
- effective environment/profile/image intent
- compatible dependency-isolation contract
- compatible resource envelope rules

This is the shared backend truth.

### Workload distinctness validation

Tasks bound to one local backend may differ in:

- listeners
- host publications
- commands
- task env
- readiness

But Ota must still reject impossible shapes, for example:

- two workloads claiming the same fixed host publication on the same backend when that cannot
  coexist
- conflicting internal bind identity when Ota cannot disambiguate them
- incompatible backend context or environment intent

## Runtime model

When Ota resolves a shared local backend with multiple workloads:

1. resolve one backend identity
2. resolve one effective environment/profile/image
3. fulfill that backend once when enabled
4. run each workload inside that backend with its own task command and listener shape
5. track workload-specific listener/publication truth separately from backend truth

This means:

- backend reuse happens once per backend unit
- workload startup happens per task
- fulfillment is not duplicated across workloads

## Addressability semantics

This feature exists mainly to make one important topology statement honest:

- multiple workloads can share one backend
- therefore `address_view: topology` can resolve between them even when they expose different
  listeners/publications

Rules:

- topology resolution must still bind to a specific workload listener
- host resolution remains workload-specific
- no guessing of localhost or host bridges

## Receipts and output

Receipts and human output must preserve both layers of truth:

### Backend evidence

- declared backend binding
- effective backend identity
- declared environment intent
- effective environment/profile/image
- fulfillment mode and result

### Workload evidence

- task identity
- resolved listener/publication details
- resolved targets
- per-task readiness and execution outcome

## Compatibility and migration

This should be additive.

Existing repos that rely on the current strict shared-backend model should continue to work.

Migration path:

1. current shared local backends remain valid
2. multi-workload validation relaxes only the workload-local fields
3. backend-shared invariants stay strict
4. repos like `qredex-core` can then move helper apps and APIs into one shared backend honestly

## qredex-core success case

This feature should allow:

- `dev` and `sandbox` to bind the same shared local backend
- one effective backend environment/profile
- one backend fulfillment unit
- distinct workload listeners/publications:
  - `dev.http` on `8080`
  - `sandbox.http` on `8787`
- `sandbox` to target `dev` through `address_view: topology`

That is the real motivating case.

## Validation requirements for implementation

At minimum, implementation should prove:

1. two tasks can share one backend while exposing different listeners/publications
2. incompatible backend environment/profile/image intent is still rejected
3. backend fulfillment still runs once per backend unit
4. `address_view: topology` resolves between bound workloads truthfully
5. receipts show backend evidence and workload evidence separately

## Product principle

One shared backend does not mean one shared workload.

The intended product behavior is:

**tasks may differ as workloads while still sharing one backend honestly.**
