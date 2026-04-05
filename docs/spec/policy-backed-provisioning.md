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

# Policy-Backed Provisioning Sources

Status: spec candidate.

This document defines ota's policy surface for approved provisioning sources for declared
runtimes and tools.

For the adapter families and rollout order, see [`docs/spec/adapters.md`](adapters.md).

This is not the current shipped `env` resolver. The shipped contract still treats `policies.env`
as a flat approved-value map. This page covers the provisioning source selection layer that
decides where an approved runtime or tool comes from when a repo asks for one.
Org policy packs are currently discovered from `.ota/org-policy.yaml` by walking ancestor
directories from the repo contract path; one ancestor policy file can therefore cover a workspace
tree.
See [`policy-packs.md`](policy-packs.md) for the current file-based behavior and the future
policy-source precedence model.
The future remote policy source is intended as an enterprise feature, not a repo-local default.

## Goal

Let an organization say:

- which sources are approved for a runtime or tool
- which versions are allowed from each source
- which install or selection path Ota may use
- which provenance should be recorded when the source wins

The point is to keep provisioning explicit, reviewable, and policy-controlled without turning Ota
into a general-purpose package manager.

The shipped mutating backends currently use `mise`, `asdf`, `sdkman`, `uv`, `winget`, `choco`,
`scoop`, `brew`, `apt`, `dnf`, and `pacman` as approved source/managers. `sdkman` and `uv` are the runtime-oriented
backends in that set; `mise`, `asdf`, `winget`, `choco`, `scoop`, `brew`, `apt`, `dnf`, and `pacman` can flow through
declared runtime and tool entries where the adapter supports them.
Chocolatey entries can also carry `source_config.feed` so an approved internal feed or mirror
stays reviewable in policy.
`source_config` is otherwise backend-specific, so backends can read only the keys they understand.

## Supported today

The built-in mutating adapters currently support:

- `mise`
- `asdf`
- `sdkman` for runtime-oriented provisioning
- `uv` for Python/runtime provisioning
- `winget` for Windows package installs
- `choco` for Windows package installs
- `scoop` for Windows developer tooling
- `brew` for macOS host tooling
- `apt` for Debian and Ubuntu package installs
- `dnf` for Fedora and RHEL-style package installs
- `pacman` for Arch package installs, with package-name installs in the current backend

Policy entries should use `source: mise`, `source: asdf`, `source: sdkman`, `source: uv`,
`source: winget`, `source: choco`, `source: scoop`, `source: brew`, `source: apt`, `source: dnf`, or `source: pacman` when they are
meant to flow through the shipped backends.
`sdkman` and `uv` are best suited to runtime entries.
All other sources remain policy-visible and read-only until a matching adapter is added.

## Non-goals

- no hidden workstation management
- no arbitrary download URLs in repo contracts
- no silent installs from unapproved sources
- no general package-manager replacement
- no broad control plane for every software installation case
- adapter bootstrap policy is a separate layer for getting Ota's adapter binaries onto the host or into the container; see [`adapter-bootstrap.md`](adapter-bootstrap.md). The shipped bootstrap backends are named separately from repo provisioning backends, for example `brew-bootstrap`, `mise-bootstrap`, `sdkman-bootstrap`, `asdf-bootstrap`, and `uv-bootstrap`.

## Relationship to current contract surfaces

- `runtimes` declares the version or distribution a repo needs
- `tools` declares supporting CLI dependencies
- `checks` proves readiness before execution
- `env` declares runtime environment requirements
- `policies.env` supplies approved env values today
- this future policy layer would supply approved source selection for provisioning

## Proposed policy shape

The first useful shape should stay small and declarative:

```yaml
policies:
  provisioning:
    defaults:
      source: approved-manager
      approved_versions:
        - "*"
    runtimes:
      java:
        source: org-mirror
        approved_versions:
          - "21"
          - "22"
    tools:
      pnpm:
        source: approved-manager
        approved_versions:
          - "10"
      node:
        source: approved-manager
        approved_versions:
          - "22"
```

The exact field names may change, but the shape should remain:

- repo declares what it needs in `runtimes` and `tools`
- policy can provide a default source for every declared runtime or tool
- policy can override the default source for a specific runtime or tool when needed
- ota resolves the approved source
- the approved `allowed` entries form the source-selection interface, and `selected_provisioning_actions` is the backend-agnostic action shape a future installer backend would consume
- the backend intake should use a serialized `ProvisioningBackendRequest { actions: [...] }` shape so the installer layer consumes only the selected actions, not the full diagnostic plan
- `ota doctor --json` should surface that request separately as `provisioning_request`, so machine consumers do not have to re-derive it from the plan
- the action kind space is intentionally reserved for `select_source`, `install`, and `verify` so the planner does not have to be redesigned when installer backends arrive
- the shipped mutating adapters accept `source: mise`, `source: asdf`, `source: sdkman`,
  `source: uv`, `source: winget`, `source: choco`, `source: scoop`, `source: brew`, `source: apt`, `source: dnf`, and `source: pacman`
  entries from policy, with `sdkman` and `uv` intended for runtime-oriented entries
- provenance is recorded in doctor, receipts, and execution summaries

## Concrete flow

```mermaid
flowchart LR
  A["Repo contract"] --> B["ota up"]
  B --> C["Checks readiness"]
  C -->|missing Java 22 / Maven| D["Policy lookup"]
  D --> E["Approved source"]
  E --> F["Provision / select"]
  F --> G["Re-check readiness"]
  G --> H["Run repo tasks"]
```

Example:

```yaml
runtimes:
  java: "22"
tools:
  maven: "3.9"
  node: "22"
checks:
  - name: java-installed
    kind: precondition
    severity: error
    run: java --version
  - name: maven-installed
    kind: precondition
    severity: error
    run: mvn -version
policies:
  provisioning:
    defaults:
      source: approved-manager
      approved_versions:
        - "*"
    runtimes:
      java:
        source: org-mirror
        approved_versions:
          - "22"
    tools:
      node:
        source: choco
        source_config:
          feed: internal-choco
        approved_versions:
          - "22"
      maven:
        source: approved-manager
        approved_versions:
          - "3.9"
tasks:
  setup:
    run: mvn -q -DskipTests package
  test:
    run: mvn test
```

With that shape:

- `ota doctor` can say Java 22 and Maven are missing or unverified
- `ota up` can run the repo-owned setup path when checks fail
- the provisioning layer can use the default policy rule for every declared runtime or tool, then override Java 22 from the internal mirror, Maven 3.9 from the approved manager, and Node 22 from the approved Chocolatey feed
- receipts can show the source that won

## Source meaning

`source` should identify the approved provisioning origin, such as:

- an internal mirror
- a private registry
- an enterprise manager
- a vendor-approved source
- an offline bundle or artifact cache

It should not encode raw download scripts or ad hoc shell commands.

For Chocolatey, `source_config.feed` can name an approved internal feed or mirror so the feed
selection stays reviewable in policy.

## Expected behavior

When this layer exists, ota should be able to:

- explain which source is approved for a requested runtime or tool
- reject unapproved sources
- prefer org policy over repo inference when the policy is explicit
- record the chosen source in receipts and diagnostics
- keep the decision deterministic and reviewable

## Command relationship

This layer would be consumed by repo-preparation commands, most likely `ota up`, once
provisioning from approved sources exists.

The command should:

- preserve normal readiness checks
- run approved provisioning inside the selected execution backend when the repo is configured for container execution
- only provision declared prerequisites
- support dry-run
- show the source and version that would be used
- fail clearly when no approved source exists

## Provenance

When a policy-approved source wins, ota should explain:

- the requested runtime or tool
- the approved source that was selected
- the version or distribution that was resolved
- whether the action was install, select, or verify-only

## Why this is separate from env policy

Env policy answers:

- what values should execution use

Provisioning policy answers:

- where should the repo’s declared tools and runtimes come from

That separation keeps policy readable and keeps the shipped `env` layer honest.

## Exit criteria

This spec becomes implementation-bound when ota can:

- resolve a repo-declared runtime or tool through an approved source
- refuse unapproved sources
- explain the chosen source in diagnostics and receipts
- keep the current `policies.env` behavior unchanged
