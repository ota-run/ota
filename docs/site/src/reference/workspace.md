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

# Workspace (`ota.workspace.yaml`)

`ota.workspace.yaml` is the canonical workspace bootstrap contract for multi-repo orchestration.

When to use:

- teams with multiple repos that need a single bootstrap and execution entrypoint

Why:

- centralizes acquisition and dependency order while reusing repo-level contracts

Typical use-cases:

- bootstrapping a fullstack system spread across `api`, `web`, and `infra` repos
- running one task (like `test`) across repos in deterministic order
- diagnosing cross-repo readiness failures from one command

It defines:

- workspace identity
- repo paths and dependency graph
- acquisition source for missing repos
- deterministic execution order for workspace commands

Minimal example:

```yaml
version: 1
workspace:
  name: example-workspace
repos:
  api:
    path: repos/api
    required: true
    source:
      git: https://github.com/example/api.git
```

Execution model:

- `ota workspace validate` checks workspace contract correctness.
- `ota workspace up` can acquire missing repos from `source.git`.
- workspace orchestration reuses repo-level `ota up` and `ota run` behavior.
- `ota workspace explain` turns workspace readiness findings into ordered remediation steps.
- dependency order is deterministic.

## Practical workflow

1. `ota workspace validate`
1. `ota workspace doctor`
1. `ota workspace explain`
1. `ota workspace up`
1. `ota workspace run <task>`

Design rule:

- workspace contract orchestrates repos
- repo contracts remain the source of truth for repo readiness

Canonical workspace reference in repository:

- `docs/spec/workspace-reference.md`
- <https://github.com/ota-run/ota/blob/main/docs/spec/workspace-reference.md>
