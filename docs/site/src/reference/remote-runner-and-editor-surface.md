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

# Remote Runner Metadata

This page defines the current Ota surface for remote-runner metadata and editor/IDE integration.

Use it when you want the same contract language to drive:

- remote execution discovery
- editor task and readiness visibility
- hosted validation consumption
- deterministic machine-facing summaries

## Source model

`docs/spec` is the canonical source of truth. This page is the public reference
layer derived from it. It adds examples, use cases, and operator guidance so the
page stands on its own while staying aligned with shipped behavior.

## Why it matters

- remote execution should be explicit, not inferred from host quirks
- editors need one stable contract surface instead of repo-specific glue
- hosted validation should consume the same JSON that CI uses
- remote target shape should be readable without learning provider internals

## Remote runner metadata

Remote runner metadata describes the execution environment without asking tools to infer it from host-specific behavior.

Typical fields:

- `provider`
- `target`
- `cwd`
- `preferred`
- `supported`

Current shipped providers include:

- `daytona`
- `ssh`
- `tsh`
- `kubectl`

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

That says the repo is prepared for remote execution, the provider is SSH, and the command
should run in `/workspace` on the target host.

## Target shape guidance

Ota keeps basic target-shape checks honest so CI and editors can catch obvious mistakes early.

Current guidance:

- `ssh` and `tsh` targets should look like `user@host`
- `kubectl` targets should start with `pod/`
- `cwd` is passed through when set
- remote execution runs in the effective target contract directory

This is guidance, not hidden magic. Ota should show you what it saw and what it chose.

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

Practical editor use cases:

- show whether the current repo is ready before the user hits run
- surface the contract and execution metadata next to the task list
- offer remote-runner hints without opening the repo YAML manually
- point the user to the exact blocker when readiness is not met

## JSON consumers

Editors, hosted validation systems, and CI should consume the same JSON surfaces as the CLI.

Recommended commands:

- `ota validate --json`
- `ota doctor --json`
- `ota workspace validate --json`
- `ota workspace doctor --json`
- `ota workspace explain --json`
- `ota workspace tasks --json`
- `ota workspace list --json`
- `ota workspace check --json`
- `ota run --json`
- `ota workspace run --json`
- `ota up --json`
- `ota workspace up --json`
- `ota diff --json`
- `ota explain --json`
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
