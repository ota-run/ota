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

# Examples By Goal

Use this page when you want the fastest path to the right example instead of scanning raw
`ota.yaml` files.

Doctor first, contract second.

## Start Here

Pick the example that matches the outcome you want to prove first.

### I need a first meaningful contract for one language repo

Start with the public templates and references:

- [Node Service Template](https://github.com/ota-run/examples/tree/main/templates/node-service) for a real
  Node service with workflow, requirements, acquisition, and agent boundaries
- [Python Service Template](https://github.com/ota-run/examples/tree/main/templates/python-service) for a real
  Python service with setup, lint, test, and acquisition
- [Canonical advanced repo](https://github.com/ota-run/examples/tree/main/reference/canonical-team-repo) when
  the main question is how Ota models `.env.local` bootstrap, env contract truth, and one proofable workflow path together
- [Tool Acquisition Flow](https://github.com/ota-run/examples/tree/main/reference/tool-acquisition-flow) when
  the main question is how Ota makes missing tools available instead of only checking for them
- [Adoption Flow](https://github.com/ota-run/examples/tree/main/reference/adoption-flow) when you want to see
  how an existing repo moves from implicit setup to an explicit Ota front door

What this proves:

- Ota models the repo front door, not just individual commands
- setup and run are separate contract concerns
- requirements and acquisition can live in the contract instead of in README prose
- agent boundaries and repeatable verification are first-class

### I need the smallest syntax example for one language repo

Use one of the minimal starters:

- [Basic Node](../../examples/basic-node/ota.yaml) for Node / TypeScript repos
- [Basic Python](../../examples/basic-python/ota.yaml) for uv-backed Python repos
- [Basic Go](../../examples/basic-go/ota.yaml) for Go module repos
- [Basic Java](../../examples/basic-java/ota.yaml) for SDKMAN + Maven repos
- [Basic Ruby](../../examples/basic-ruby/ota.yaml) for Ruby + Bundler repos
- [Basic Rust](../../examples/basic-rust/ota.yaml) for Rustup-backed Cargo repos
- [Basic .NET](../../examples/basic-dotnet/ota.yaml) for .NET toolchain repos
- [Basic Script](../../examples/basic-script/ota.yaml) for script-only repos

What this proves:

- the minimum `ota.yaml` shape for one stack
- `ota doctor`, `ota up`, and `ota run` on a simple repo
- the smallest contract boundary without service topology or workspace complexity

Use these when you want the smallest valid shape first, not when you want the strongest example of
what Ota can model.

### I need a Windows-native repo with explicit compiler activation

Start with:

- [Windows-first adoption with explicit native activation](https://github.com/ota-run/examples)

What this proves:

- native compiler activation can be contract truth instead of shell tribal knowledge
- `visual_studio_dev_shell` is part of the selected task path, not a manual prerequisite outside Ota
- `ota doctor`, `ota up --dry-run`, and `ota proof runtime --workflow app` can describe and verify the same Windows-native path

Use this when the repo is Windows-heavy and the real question is whether Ota can keep native activation explicit and trustworthy.

### I need a normal app repo with services

Start with:

- [Basic Services](../../examples/basic-services/ota.yaml)

Then look at:

- [Mixed Node + Python](../../examples/mixed-node-python/ota.yaml)
- [Fullstack Node + Go](../../examples/fullstack-node-go/ota.yaml)

What this proves:

- service-backed tasks
- workflow-scoped setup and run for an app path
- workflow `prepare` for deterministic host file bootstrap before setup
- contract env policy plus task-scoped env requirements
- local readiness and targetable listeners
- a more realistic app contract than a single-script starter

### I need shared local topology across workloads

Start with:

- [Shared Local Topology](../../examples/shared-local-topology/ota.yaml)

What this proves:

- one declared shared local backend
- workload-local service publications on that backend
- truthful `address_view: internal` target binding for a co-located helper app

Use this when the real question is:

- “how do two local workloads intentionally share one backend boundary without pretending it is
  `depends_on`?”

### I need shared remote topology dogfood

Start with:

- [Shared Remote Topology](../../examples/shared-remote-topology/README.md)

What this proves:

- shared remote backend binding through the built-in `ssh` provider
- remote activation and readiness on the remote plane
- consumer targeting through `address_view: internal`

Use this only when you want to exercise the shipped shared-remote slice end to end. It is a
dogfood example, not the first example to hand to a new user.

### I need multi-repo workspace bootstrap

Start with:

- [Basic Workspace](../../examples/workspace-basic/ota.workspace.yaml)
- [Acquisition Workspace](../../examples/workspace-acquire/ota.workspace.yaml)

What this proves:

- `ota workspace doctor`
- `ota workspace up`
- multi-repo bootstrap on the workspace contract boundary

### I need the broadest contract reference

Use:

- [Full Contract Example](../../examples/full-contract/ota.yaml)

What this proves:

- the widest shipped contract surface in one place
- reference authoring after the smaller examples already make sense

Do not start here if your goal is first success.

## Recommended First-Run Paths

### Existing repo with `ota.yaml`

```bash
ota doctor
ota explain
ota up
ota run <task>
ota receipt
```

Use `ota tasks --use` when the next runnable task is unclear.

### Repo without `ota.yaml`

```bash
ota doctor
ota detect --dry-run .
ota init --dry-run
```

Then choose one explicit write path:

```bash
ota init
```

Use `ota detect --write .` when you want the detector-led authoring path instead of the starter contract path.

## When To Leave This Repo For More Examples

Use [ota-run/examples](https://github.com/ota-run/examples) when you want:

- production-adjacent templates
- more opinionated stack examples
- examples that go beyond the small canonical shapes kept in this repo
- stronger examples of workflows, acquisition, readiness, and agent-safe repo operation

## Related Docs

- [Quickstart](../quickstart.md)
- [Worked Example: Existing Repo](./worked-example-existing-repo.md)
- [One Team Rollout](./one-team-rollout.md)
- [Command Reference](../spec/command-reference.md)
