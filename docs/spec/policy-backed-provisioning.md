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

This document defines the next policy extension point for Ota: org-approved provisioning
sources for declared runtimes and tools.

This is not the current shipped `env` resolver. The shipped contract still treats `policies.env`
as a flat approved-value map. This spec describes the later layer that can decide where an
approved runtime or tool comes from when a repo asks for one.

## Goal

Let an organization say:

- which sources are approved for a runtime or tool
- which versions are allowed from each source
- which install or selection path Ota may use
- which provenance should be recorded when the source wins

The point is to keep provisioning explicit, reviewable, and policy-controlled without turning Ota
into a general-purpose package manager.

## Non-goals

- no hidden workstation management
- no arbitrary download URLs in repo contracts
- no silent installs from unapproved sources
- no general package-manager replacement
- no broad control plane for every software installation case

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
    java:
      source: org-mirror
      approved_versions:
        - "21"
        - "22"
    node:
      source: approved-manager
      approved_versions:
        - "22"
    tools:
      pnpm:
        source: approved-manager
        approved_versions:
          - "10"
```

The exact field names may change, but the shape should remain:

- repo declares what it needs
- policy declares where it may come from
- Ota resolves the approved source
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
    java:
      source: org-mirror
      approved_versions:
        - "22"
    tools:
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
- a later provisioning layer can use policy to select Java 22 from the internal mirror and Maven 3.9 from the approved manager
- receipts can show the source that won

## Source meaning

`source` should identify the approved provisioning origin, such as:

- an internal mirror
- a private registry
- an enterprise manager
- a vendor-approved source
- an offline bundle or artifact cache

It should not encode raw download scripts or ad hoc shell commands.

## Expected behavior

When this layer exists, Ota should be able to:

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
- only provision declared prerequisites
- support dry-run
- show the source and version that would be used
- fail clearly when no approved source exists

## Provenance

When a policy-approved source wins, Ota should explain:

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

This spec becomes implementation-bound when Ota can:

- resolve a repo-declared runtime or tool through an approved source
- refuse unapproved sources
- explain the chosen source in diagnostics and receipts
- keep the current `policies.env` behavior unchanged
