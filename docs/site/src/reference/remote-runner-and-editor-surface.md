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

# Remote Runner Metadata and Editor Surface

This page defines the current Ota surface for remote-runner metadata and editor/IDE integration.

Use it when you want the same contract language to drive:

- remote execution discovery
- editor task and readiness visibility
- hosted validation consumption
- deterministic machine-facing summaries

## Remote runner metadata

Remote runner metadata describes the execution environment without asking tools to infer it from host-specific behavior.

Typical fields:

- provider
- target
- working directory
- supported command shape
- runtime or tool hints

Example:

```yaml
execution:
  preferred: remote
  supported:
    - remote
  backends:
    remote:
      provider: ssh
      target: user@host
      cwd: /workspace
```

## Editor and IDE surface

The editor surface should make it easy to:

- discover tasks
- inspect readiness
- launch runnable tasks
- view remote execution hints
- surface repo and workspace diagnostics without custom glue

The integration model stays:

- contract-driven
- read-only by default
- deterministic
- usable without repo-specific heuristics

## JSON consumers

Editors, hosted validation systems, and CI should consume the same JSON surfaces as the CLI.

Recommended commands:

- `ota doctor --json`
- `ota workspace doctor --json`
- `ota workspace list --json`
- `ota extensions --json`

Consumption rules:

- treat `ok` and exit code together
- surface `findings` directly
- preserve repo/workspace scope boundaries
- do not infer extra execution behavior from text output
- treat `execution` metadata as descriptive contract data unless the command explicitly runs it

## Semantics

- remote-runner metadata is descriptive, not magical
- editor integrations should consume the canonical contract data
- UI surfaces may present richer affordances, but they must not invent execution rules

## Scope

This surface is for:

- editor/tool compatibility
- remote runner discoverability
- stable metadata shape
- hosted validation consumption

It is not for:

- provider-specific runtime implementation details
- hidden transport negotiation
- general plugin execution policy
