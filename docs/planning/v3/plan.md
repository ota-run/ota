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

# V3 Plan

Status: active.

Source direction: [08-v3-spec.md](/Users/bobai/Desktop/Ota.run/Spec/new/08-v3-spec.md)

V3 theme:

- real scale
- serious team use
- robust multi-package support

## Included capabilities

### Workspace and monorepo

- root/member contracts with inheritance
- member-level overrides
- workspace task graph
- selective execution for specific members

### Backends

- first-class container backend abstraction
- first-class remote backend abstraction
- backend capability negotiation
- explicit lifecycle semantics for isolated execution
- command-time backend and lifecycle overrides
- cleanup / teardown support for isolated execution
- runtime distribution support where version alone is not sufficient, especially Java

### Output

- stable machine-readable diagnostics schema
- richer exit codes

### Policy

- stricter validation modes
- task safety policy
- agent writable-path policy

## Priorities

1. Support real monorepo structures cleanly
2. Make container execution a real first-class path
3. Enable remote execution for coding agents and cloud runners
4. Lock in machine-readable output as a stable API surface

## Current progress

Implemented so far:

- root/member monorepo declaration in `ota.yaml`
- member override inheritance
- explicit `--member` targeting for repo commands
- automatic member discovery from inside member directories
- root validation that proves declared members exist and validate as merged contracts
- selective multi-member execution for `ota run`
- selective multi-member inspection for `ota tasks`
- member-aware `doctor`
- root-level `ota tasks` summary for monorepo roots
- root-level `ota doctor` summary for monorepo roots
- member-aware `check`
- root-level `ota check` summary for monorepo roots
- member-aware `up`
- root-level `ota up` summary for monorepo roots
- `ota workspace tasks` task-graph summary in dependency order
- `ota workspace run <task>` deterministic task orchestration across workspace repos
- `ota workspace check` deterministic checks-only orchestration across workspace repos
- first real container-backed `ota run`
- `execution.lifecycle` enforcement for container-backed `ota run`
- command-time `ota run --backend` / `--lifecycle` and `ota up --backend` / `--lifecycle` overrides
- repo-level `ota clean` for persistent container teardown
- runtime distribution support such as `runtimes.java.distribution`
- real remote-backed `ota run` for `provider: daytona`, `provider: ssh`, `provider: tsh`, and `provider: kubectl` with explicit `target`
- remote-backed `ota up` setup execution through the same backend path
- provider-specific remote target guidance in validator and runner errors (for `daytona`, `ssh`/`tsh`, and `kubectl`)
- doctor warnings for suspicious remote target shape (`ssh`/`tsh` without `user@host`, `kubectl` without `pod/...`)
- doctor JSON warning contract locks for suspicious remote target findings (`ssh`, `tsh`, `kubectl`) including full finding object stability
- aggregate doctor JSON monorepo coverage proving all suspicious remote target warning families can appear together in one run

Still to do in V3:

- no additional required scope items remain in this plan
- broader provider-specific remote semantics are deferred per [12-v3-gap-closure-and-deferred.md](/Users/bobai/Desktop/Ota.run/Spec/new/12-v3-gap-closure-and-deferred.md)

## Workspace model direction

```yaml
workspace:
  type: monorepo
  members:
    - api
    - web
    - sdk
```

Member contracts should inherit from the root and override only what differs.

## Backend model direction

```yaml
execution:
  preferred: container
  supported:
    - native
    - container
    - remote
  backends:
    container:
      image: ghcr.io/myorg/dev:latest
    remote:
      provider: ssh
      target: sandbox-dev
      cwd: /workspace
```

## Success criteria

V3 succeeds when:

- a monorepo with 3 or more members can be fully described and run via Ota
- task execution can target one member cleanly
- container execution is a real code path, not just declared metadata
- remote sandbox execution works end to end for coding-agent workflows
