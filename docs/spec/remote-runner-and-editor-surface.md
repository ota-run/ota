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

This document defines the V7 target for remote-runner metadata and editor/IDE integration in Ota.

The goal is to make remote execution and editor discovery use the same contract language instead of separate ad hoc surfaces.

## Purpose

This surface supports:

- remote execution discovery
- editor task and readiness visibility
- reproducible runner metadata
- explicit integration points for IDEs and tooling

## Remote runner metadata

Remote runner metadata should describe the execution environment without requiring tools to infer it from host-specific behavior.

Target metadata includes:

- provider
- target
- working directory
- supported command shape
- any required runtime or tool hints

Example shape:

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

## Editor/IDE surface

The editor surface should make it easy to:

- discover tasks
- view readiness status
- inspect diagnostics
- launch runnable tasks from the IDE
- surface remote execution hints when they matter

The editor integration model should remain:

- contract-driven
- read-only by default
- deterministic
- usable without custom repo-specific code

## Editor and CI consumption contract

Editors, hosted validation systems, and CI should consume the same stable JSON surfaces as the CLI.

Recommended inputs:

- `ota doctor --json` for repo readiness diagnostics
- `ota workspace doctor --json` for workspace readiness diagnostics
- `ota workspace list --json` for repo inventory, contract presence, readiness, and execution metadata
- `ota extensions --json` for declared adapter descriptors when a repo exposes them

Recommended consumption rules:

- treat `ok` and exit code together
- surface `findings` directly without reinterpreting them
- preserve the distinction between repo and workspace scope
- do not infer extra execution behavior from text output
- use `execution` metadata only as descriptive contract data unless the command explicitly runs it

Typical editor behavior:

- annotate missing tasks, runtimes, tools, and workspace acquisition problems inline
- expose a readiness summary panel sourced from JSON
- surface explicit actions that map back to Ota commands
- avoid custom per-repo heuristics when the contract already says what to do

## Semantics

- remote-runner metadata is descriptive, not magical
- editor integrations should consume the same canonical contract data as the CLI
- UI surfaces may present richer affordances, but they must not invent execution rules

## Scope

This surface is for:

- implementation guidance
- editor/tool compatibility
- remote runner discoverability
- stable metadata shape
- hosted validation consumption

It is not for:

- provider-specific runtime implementation details
- hidden transport negotiation
- general plugin execution policy
