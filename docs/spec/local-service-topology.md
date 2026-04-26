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

# Local Service Topology

Status: proposed.

This spec defines the long-term Ota feature program for container-first local development:

- first-class task target bindings
- intentional shared local backends for long-running tasks
- backend-scoped run-path fulfillment
- policy-governed fulfillment profiles and image resolution
- advanced examples and adoption surface

This spec extends [execution-topology.md](execution-topology.md). It does not replace it.

## Core truth

Ota already models:

- where a task runs
- what services exist
- which listeners a service task exposes
- whether the host sees a published endpoint

Ota does not yet model strongly enough:

- which local service another task intends to target by identity
- which long-running tasks intentionally share one local backend
- which address is correct inside that shared topology
- how a mixed-runtime local backend is fulfilled on the actual run path

That gap forces repos into glue:

- guessed defaults like `127.0.0.1` or `host.docker.internal`
- repo-local workbench bootstrap scripts
- fake orchestration through sequential `depends_on`
- hidden assumptions about which address is valid from which container

The product boundary should be:

- repos declare intent
- Ota serves the topology

## Design goals

1. Make topology relationships first-class product concepts.
2. Keep explicit operator overrides available without making them the primary topology model.
3. Let multiple long-running local tasks intentionally share one backend boundary.
4. Let Ota resolve the correct reachable address for that topology.
5. Let Ota fulfill the effective backend requirements for that topology on the run path.
6. Let policy govern approved fulfillment profiles and effective environment resolution cleanly.
7. Keep receipts and human output honest about declared versus effective addresses and environments.

## Non-goals

- no remote-control-plane redesign
- no enterprise policy redesign in this slice
- no replacement for explicit literal URL inputs
- no Docker-only heuristics disguised as product truth
- no hidden silent mutation during `doctor`

## Product principles

### Make topology relationships first-class

The main abstraction should not be "an input default comes from a service".

The main abstraction should be:

- this task targets that service
- this task shares that backend
- this backend fulfills these effective requirements

Operator inputs remain important, but they are override surfaces, not the primary topology model.

### Keep open overrides

Tasks like `sandbox` must still allow open operator overrides such as:

- `--base-url https://staging.example.com`

Ota should not remove that flexibility. It should attach that override cleanly to a first-class
target binding.

### Provide topology-native resolution

When the operator does not pass an explicit override, Ota should resolve the obvious local target
from declared topology truth.

### Do not guess `localhost`

`localhost`, `127.0.0.1`, and `host.docker.internal` are topology views, not universal truths.
Ota should compute the right address for the selected local topology instead of asking repos to
guess.

### Do not use `depends_on` as service-stack orchestration

`depends_on` remains correct for:

- setup ordering
- finite prerequisite tasks

It is not the right primitive for:

- "run these two long-lived services in one local stack"

## Feature surface

This spec introduces five connected slices.

### 1. Task target bindings

A task may declare one or more named targets that point at declared local services by identity.

Conceptual shape:

```yaml
tasks:
  sandbox:
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: topology
        override_input: base_url
```

Meaning:

- `targets.api` is the topology truth
- `service.task` identifies the service task
- `service.listener` identifies the listener on that task
- `address_view: topology` asks Ota for the correct reachable URL for the current local topology
- `override_input: base_url` keeps an explicit operator override hook without making the input the
  primary topology abstraction

Rules:

- target bindings are first-class and validated
- explicit operator override wins when present
- otherwise Ota resolves the effective target from declared topology truth
- Ota must reject ambiguous, invalid, or recursive references

### 2. Shared local backend binding

Multiple long-running tasks may intentionally share one local backend instance.

Conceptual shape:

```yaml
execution:
  local_backends:
    workbench:
      backend: container
      lifecycle: persistent
      context: app-base
      fulfillment: run

tasks:
  dev:
    context: app
    runtime:
      kind: service
      backend_binding: workbench

  sandbox:
    context: sandbox
    runtime:
      kind: service
      backend_binding: workbench
```

Meaning:

- `local_backends.workbench` defines one intentional reusable local backend boundary
- multiple tasks can bind to it
- Ota owns reuse, lifecycle, addressing, and cleanup semantics for that boundary

This is distinct from:

- `execution.contexts`
- `depends_on`
- service-manager attachments

Contexts still describe workload shape. Backend bindings describe shared local realization.

### 3. Backend-scoped run-path fulfillment

When a local backend binding is selected, Ota should fulfill the effective backend requirements
needed to realize that backend for the tasks being started there.

Meaning:

- if one bound task needs Java + Maven
- and another bound task needs Node + curl
- Ota should fulfill the effective backend environment or fail with a truthful fulfillment error

This is an execution concern, not repo-local glue.

The repo should not need a custom bootstrap task merely to make Ota’s declared topology usable.

### 4. Policy-governed fulfillment profiles and image resolution

Once local topology and backend fulfillment are first-class, policy should be able to govern:

- approved fulfillment profiles
- image or base-environment aliases
- allowed fulfillment sources
- declared versus effective environment evidence

The long-term enterprise-safe design is:

- repo declares intent
- policy resolves approved fulfillment profile or image
- Ota realizes the effective backend
- receipts show declared versus effective environment truth

### 5. Declared-versus-effective evidence and adoption surface

Human output and JSON receipts must expose:

- declared target reference
- effective resolved target URL
- declared backend binding
- effective backend identity
- declared requirements
- effective fulfilled environment
- declared fulfillment profile or backend class when applicable
- effective profile/image/source when policy later governs resolution

This becomes mandatory once Ota starts resolving target defaults and backend fulfillment for the
operator.

## Proposed contract details

### Task target references

`tasks.<name>.targets.<target>.service` is a typed reference:

```yaml
targets:
  api:
    service:
      task: dev
      listener: http
      address_view: topology
    override_input: base_url
```

Required fields:

- `task`
- `listener`

Optional fields:

- `address_view`
- `override_input`

Allowed `address_view` values:

- `topology` = best reachable address for the current local topology
- `host` = host-published address if published
- `internal` = in-backend address if one exists

`topology` is the recommended default because it lets Ota choose correctly for the selected
execution shape.

`override_input` is optional but recommended when operators may need to point the task at a
different environment explicitly.

Override precedence:

1. explicit `override_input`
2. resolved target binding
3. compatibility literal default only where older input-based contracts still rely on one

### Local backend bindings

`execution.local_backends.<name>` declares an Ota-owned local backend instance.

Required fields:

- `backend`
- `lifecycle`

Optional fields:

- `context`
- `attachments`
- `fulfillment`
- `publish`

Rules:

- only local execution backends are in scope for this feature slice
- `backend: container` is the primary initial target
- multiple tasks may bind to the same local backend
- service identity remains task-scoped even when the backend is shared
- backend bindings must not replace service managers; they describe workload colocation

### Fulfillment mode

`execution.local_backends.<name>.fulfillment` is proposed as:

- `none` = do not fulfill; fail if requirements are missing
- `run` = fulfill on the run path before the backend is used

Initial product recommendation:

- default to `run` only when the backend binding explicitly asks for it
- keep fulfillment explicit in early versions

### Effective backend requirement resolution

For a shared local backend, Ota computes the effective requirements as the union of:

- bound task context requirements
- bound task runtime/tool requirements if Ota later supports them explicitly
- any backend-binding-specific requirements

Conflicts must fail clearly.

Example:

- `java >=21`
- `node >=24`
- `maven *`
- `curl *`

Ota should either:

- fulfill that environment
- or report exactly why it cannot

## Address resolution semantics

Ota must distinguish at least three views:

1. `host`
- the host-published address visible outside the local backend

2. `internal`
- the address reachable from another workload in the same backend or network

3. `topology`
- the best address for the current task in the current local topology

Examples:

- host sees `http://127.0.0.1:8080`
- sibling task in same shared backend sees `http://127.0.0.1:8080`
- sibling task in separate but connected backend may see `http://dev:8080`
- sibling task in an ephemeral helper container attached through the host bridge may see `http://host.docker.internal:8080`

The repo should not have to encode those distinctions manually when Ota already knows the chosen
topology.

## Orchestration semantics

### What stays true

`depends_on` remains:

- sequential
- finite
- non-stack-oriented

### What this feature adds

This feature adds intentional local stack semantics without changing `depends_on`.

Ota should be able to:

- start one long-running task in a shared backend
- start another long-running task bound to the same backend
- preserve independent task identity and receipts
- keep one shared local backend alive while either or both services remain in use

### Cleanup and reuse

For shared persistent local backends, Ota must define:

- what creates the backend
- what marks it reusable
- which workloads survive task interruption
- what `ota clean` removes
- what `ota run` reuses

The UX must stop feeling magical or accidental.

## Output and evidence contract

When Ota resolves a task target binding, the human output should be able to say:

- declared target binding: `service(dev.http)`
- effective target URL: `http://127.0.0.1:8080`
- source: topology resolution

When the user overrides:

- declared target binding: `service(dev.http)`
- override input: `base_url`
- effective target URL: `https://staging.example.com`
- source: user override

JSON receipts should carry structured equivalents.

Suggested JSON shape:

```json
{
  "target_resolution": {
    "target": "api",
    "override_input": "base_url",
    "source": "target_binding",
    "service_ref": {
      "task": "dev",
      "listener": "http",
      "address_view": "topology"
    },
    "effective_url": "http://127.0.0.1:8080"
  }
}
```

When a shared backend is used, receipts should also expose:

- declared backend binding
- effective backend id/name
- effective fulfilled requirements
- declared fulfillment profile when present
- effective profile/image/source when policy resolution applies

## Compatibility and migration

This feature must be additive.

Compatibility rules:

- literal input defaults remain valid
- repos can adopt first-class `targets` incrementally
- input-based service defaults may exist only as a temporary compatibility bridge if needed during
  migration, but they are not the preferred long-term public model
- repos do not need to migrate all tasks at once
- human and JSON output must show whether a target came from:
  - explicit override
  - target binding
  - compatibility literal default

Migration goal:

- remove repo-local topology hacks over time
- do not force a flag day

## Examples and adoption

Each implementation slice in this spec must improve the public example surface as part of the
feature delivery, not as a later polish task.

Required adoption discipline:

- update at least one existing example in [`examples/`](../../examples/) to use the newly shipped
  feature slice when that example becomes clearer or more honest with the new surface
- add one advanced example when the slice introduces a new topology authoring pattern that is not
  already represented cleanly in the existing example set
- keep examples reviewable and minimal; they should demonstrate the canonical feature shape, not a
  repo-local workaround
- keep example contracts aligned with the actual shipped semantics and receipts

Recommended rollout by slice:

1. task target bindings
- update an existing service example to show first-class `targets`
- add an advanced example showing topology binding plus open operator override

2. shared local backends
- add an advanced example with two long-running local services intentionally sharing one backend
- make clear why this is not `depends_on`

3. backend-scoped run-path fulfillment
- extend the advanced example to show mixed-runtime fulfillment in the shared backend
- document the resulting declared-versus-effective environment evidence

4. policy-governed fulfillment and image/profile resolution
- add or extend an advanced example showing declared profile intent versus policy-resolved effective
  environment
- keep the local example usable without enterprise policy, but add a governed variant where helpful

5. advanced examples and adoption surface
- update core site/reference docs as each slice ships
- keep `examples/` and public teaching surfaces aligned with the canonical contract shape
- ensure at least one advanced topology example remains current end-to-end

The goal is adoption leverage, not example count. Every slice should leave Ota easier to understand
through a concrete contract in `examples/`.

## Example: `qredex-core`

Desired authoring shape:

```yaml
execution:
  default_context: application
  contexts:
    application:
      backend: container
      lifecycle: persistent
      container:
        image: maven:3.9.14-eclipse-temurin-21-noble
      requirements:
        runtimes:
          java: ">=21"
        tools:
          maven: "*"

    sandbox:
      backend: container
      lifecycle: persistent
      requirements:
        runtimes:
          node: ">=24.14.1"
        tools:
          curl: "*"

  local_backends:
    dev-stack:
      backend: container
      lifecycle: persistent
      fulfillment: run

tasks:
  dev:
    context: application
    runtime:
      kind: service
      backend_binding: dev-stack
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 8080

  sandbox:
    context: sandbox
    targets:
      api:
        service:
          task: dev
          listener: http
          address_view: topology
        override_input: base_url
    runtime:
      kind: service
      backend_binding: dev-stack
```

Meaning:

- `sandbox` still accepts explicit `--base-url`
- if not provided, Ota resolves the local API target from `dev` through the first-class `api`
  target binding
- `dev` and `sandbox` intentionally share one persistent local backend
- Ota fulfills the union runtime/tool requirements for that backend

What disappears:

- repo-local workbench bootstrap scripts
- host-bridge guessed defaults
- fake sequential service-stack orchestration

## Validation requirements

Implementation of this spec must add focused coverage for at least:

1. task target binding resolves from declared service identity
2. explicit override input overrides task target binding
3. invalid service references are rejected at validation time
4. multiple long-running tasks can bind to one local backend intentionally
5. shared backend fulfillment computes the union of effective requirements
6. fulfillment failures are reported as fulfillment failures, not generic task failures
7. receipts expose declared versus effective target and backend evidence
8. compatibility with literal defaults remains intact
9. policy resolution exposes declared versus effective profile/image/source evidence when present
10. at least one existing example and one advanced example are updated to teach the shipped slice

## Relationship to policy

This feature does not redefine enterprise policy, but it must join cleanly with it.

Future policy-backed environment resolution should be able to govern:

- which images or fulfillment profiles are allowed
- which sources may satisfy the effective requirements
- declared versus effective environment evidence

This local-topology feature should not block that future. It should make the local runtime story
strong enough that policy later governs an existing clean model instead of patching around a weak
one.

## Decision summary

Ota should own:

- first-class task target bindings
- shared long-running local backend topology
- backend-scoped run-path fulfillment
- policy-governed fulfillment profile and image resolution
- clear service identity and address resolution
- declared-versus-effective evidence

Repos should keep owning:

- task intent
- service listeners
- explicit override inputs
- repo-specific app behavior

The intended product behavior is:

**repo declares intent; Ota serves the topology and realizes the effective environment honestly.**
