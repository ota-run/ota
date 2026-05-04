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

### I need a first contract for one language repo

Use one of the smallest starters:

- [Basic Node](../../examples/basic-node/ota.yaml) for Node / TypeScript repos
- [Basic Python](../../examples/basic-python/ota.yaml) for Python repos
- [Basic Go](../../examples/basic-go/ota.yaml) for Go module repos
- [Basic Java](../../examples/basic-java/ota.yaml) for Maven repos
- [Basic Rust](../../examples/basic-rust/ota.yaml) for Cargo repos
- [Basic .NET](../../examples/basic-dotnet/ota.yaml) for C# / .NET repos
- [Basic Script](../../examples/basic-script/ota.yaml) for script-only repos

What this proves:

- the minimum `ota.yaml` shape for one stack
- `ota doctor`, `ota up`, and `ota run` on a simple repo
- the contract boundary without service topology or workspace complexity

### I need a normal app repo with services

Start with:

- [Basic Services](../../examples/basic-services/ota.yaml)

Then look at:

- [Mixed Node + Python](../../examples/mixed-node-python/ota.yaml)
- [Fullstack Node + Go](../../examples/fullstack-node-go/ota.yaml)

What this proves:

- service-backed tasks
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

## Related Docs

- [Quickstart](../quickstart.md)
- [Worked Example: Existing Repo](./worked-example-existing-repo.md)
- [One Team Rollout](./one-team-rollout.md)
- [Command Reference](../spec/command-reference.md)
