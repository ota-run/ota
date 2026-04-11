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

# Provisioning Sources

Status: shipped in part.

This document explains how ota chooses approved sources for declared runtimes and tools.

For the adapter families and rollout order, see [`docs/spec/adapters.md`](adapters.md).

This is not the current shipped `env` resolver. The shipped contract still treats `policies.env`
as a flat approved-value map. This page covers the provisioning source selection layer that
decides where an approved runtime or tool comes from when a repo asks for one.
Org policy packs are currently discovered from `.ota/org-policy.yaml` by walking ancestor
directories from the repo contract path; one ancestor policy file can therefore cover a workspace
tree.
See [`policy-packs.md`](policy-packs.md) for the current file-based behavior and the future
policy-source precedence model.
`OTA_POLICY` can already point at a local file path or an HTTP(S) URL; a separately declared
remote policy source remains a future enterprise feature, not a repo-local default.

## Goal

Let an organization say:

- which sources are approved for a runtime or tool
- which versions are allowed from each source
- which install or selection path ota may use
- which provenance should be recorded when the source wins

The point is to keep provisioning explicit, reviewable, and policy-controlled without turning ota
into a general-purpose package manager.

The shipped mutating backends currently use `mise`, `asdf`, `sdkman`, `uv`, `winget`, `choco`,
`scoop`, `brew`, `apt`, `dnf`, and `pacman` as approved source/managers. `sdkman` and `uv` are the runtime-oriented
backends in that set; `mise`, `asdf`, `winget`, `choco`, `scoop`, `brew`, `apt`, `dnf`, and `pacman` can flow through
declared runtime and tool entries where the adapter supports them.
A custom feed or mirror for an existing package manager, for example, Chocolatey, remains within
the same adapter family and does not create a new ota source.

Chocolatey is the first concrete example, not the only one. The same `source_config` pattern also
works for `winget`, `scoop`, `apt`, `dnf`, and `brew`.

ota can provide an approved feed through policy where the adapter supports that behavior, while
other adapters may specify equivalent source details through their own `source_config` keys.

## Supported Today

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

For container-backed execution, policy pins must still be installable in the selected image.
Ota now surfaces backend-aware provisioning failures across the shipped adapters, and container
doctor uses safe non-mutating installability probes for the supported backends in that path.
Today those probes ship for all built-in mutating provisioning adapters. `apt` additionally distinguishes
pinned-version unavailable, package unavailable, and apt index/source failures when backend
evidence supports that classification.
When a mutating provisioning command fails with only generic backend stderr, `ota up` can reuse the
same read-only probe path to refine the failure class while preserving the original backend output.

Policy entries should use `source: mise`, `source: asdf`, `source: sdkman`, `source: uv`,
`source: winget`, `source: choco`, `source: scoop`, `source: brew`, `source: apt`, `source: dnf`, or `source: pacman` when they are
meant to flow through the shipped backends.
`sdkman` and `uv` are best suited to runtime entries.
All other sources remain policy-visible and read-only until a matching adapter is added.

## Platform-Specific Provisioning

Use `platforms` when the same runtime or tool should resolve to different sources on macOS,
Linux, and Windows. The root rule acts as the default, and platform entries override it when the
current OS matches.

Example:

```yaml
policies:
  provisioning:
    node:
      source: brew
      approved_versions:
        - "22"
      platforms:
        macos:
          source: brew
          approved_versions:
            - "22"
        linux:
          source: apt
          package: nodejs
          source_config:
            sources_list:
              - deb http://mirror.local/debian bookworm main
          approved_versions:
            - "22"
        windows:
          source: choco
          package: nodejs
          source_config:
            feed: internal-choco
          approved_versions:
            - "22"
```

This lets ota keep one contract for the repo while still choosing the approved source that
matches the OS it is running on.

## Version Resolution Discipline

Policy-backed provisioning must keep **approval** separate from **exact install selection**.

That distinction matters because a semver range can authorize a family of versions without
telling ota which exact version should be installed.

### Rule

- ranges can authorize
- only explicit concrete versions can select

### Shipped today

The current shipped behavior is:

- exact string matches still work as before
- major or minor shorthand such as `24` or `3.9` is normalized for policy matching
- policy `approved_versions` can authorize with semver ranges such as `^24` or `~3.9`
- ota records the original `requested_version`, the semver `normalized_requirement`,
  the matched `policy_match`, and `resolved_version` when an exact policy candidate won
- when `resolved_version` is absent, ota keeps the original contract version as the backend input
- true range requests such as `>=18` are provisionable only when policy supplies an
  explicit exact candidate like `22.11.0`
- if policy approval succeeds but no deterministic concrete version exists, ota blocks
  provisioning instead of guessing

## Package Mapping Discipline

Some provisioning adapters need an explicit install identifier that differs from the contract
name. Policy-backed provisioning handles this through an optional `package` field on each rule.

### Package rule

- `package` may override the install identifier while keeping the contract key stable
- `package` is required for OS package managers (`apt`, `dnf`, `pacman`, `winget`, `choco`, `scoop`)
- `package` is optional for runtime managers (`brew`, `mise`, `asdf`, `sdkman`, `uv`)

Example:

```yaml
policies:
  provisioning:
    java:
      source: apt
      package: openjdk-22-jdk
      approved_versions:
        - "22"
```

In that case:

- the contract still declares `runtimes.java`
- policy selects `apt` as the backend
- `openjdk-22-jdk` is the install identifier passed to `apt`

### What ota should not do

Ota should not invent an install version from a range-only policy.

Example:

```yaml
runtimes:
  node: ">=18"
policies:
  provisioning:
    node:
      source: brew
      approved_versions:
        - "^22"
```

In that case:

- policy approval can succeed because the policy allows the `22.x` line
- exact install selection is still unknown
- ota should not silently choose `22.99.99`, `latest 22`, or any other undeclared concrete version

That would be a hidden policy decision rather than a declared one, and it would weaken
determinism and trust.

### Safe concrete selection

Concrete selection is safe only when policy provides an explicit concrete candidate.

Example:

```yaml
runtimes:
  node: ">=18"
policies:
  provisioning:
    node:
      source: brew
      approved_versions:
        - "22.11.0"
```

Now ota can resolve concretely to `22.11.0` because the policy provided an exact installable
candidate.

### Semver-aware approval is still valuable

Ota should still become semver-aware for policy approval and auditability even when exact
selection is deferred.

Example:

```yaml
runtimes:
  node: "24"
policies:
  provisioning:
    node:
      source: brew
      approved_versions:
        - "^24"
```

A naive exact-string comparison would reject that. Semver-aware approval should understand:

- requested intent is the `24` line
- policy allows the `24` line
- policy approval therefore succeeds

The audit surface should make that explicit, for example:

- requested version: `24`
- normalized requirement: `>=24.0.0 <25.0.0`
- matched policy rule: `^24`
- selected source: `brew`

That improves policy matching and diagnostics without pretending ota already knows the exact
install version.

### Cross-backend selection stays deferred

Ota should not force one global concrete-resolution rule across all shipped provisioning adapters
until there is an explicit deterministic model.

That matters because:

- `brew`, `apt`, `sdkman`, `mise`, `asdf`, `winget`, `choco`, `scoop`, `dnf`, and `pacman` all
  behave differently
- some accept majors
- some require exact versions
- some map logical runtime names onto different package identifiers or formulas
- some expose only the versions currently present in their repositories or feeds

Until ota has a declared, reproducible selection model, the safer truth is:

- this contract version is policy-approved
- this is the policy rule that matched
- no concrete install version is available yet for deterministic provisioning

### Current implementation order

1. semver-aware policy approval is shipped
2. requested version, normalized requirement, matched policy rule, and resolved version are exposed in doctor JSON and policy-aware receipt text
3. broader cross-backend concrete selection remains deferred until ota has a declared deterministic model

## Non-Goals

- no hidden workstation management
- no arbitrary download URLs in repo contracts
- no silent installs from unapproved sources
- no general package-manager replacement
- no broad control plane for every software installation case
- adapter bootstrap policy is a separate layer for getting ota's adapter binaries onto the host or into the container; see [`adapter-bootstrap.md`](adapter-bootstrap.md). The shipped bootstrap backends are named separately from repo provisioning backends, for example `brew-bootstrap`, `mise-bootstrap`, `sdkman-bootstrap`, `asdf-bootstrap`, `uv-bootstrap`, `winget-bootstrap`, `choco-bootstrap`, and `scoop-bootstrap`.

## Relationship to Current Contract Surfaces

- `runtimes` declares the version or distribution a repo needs
- `tools` declares supporting CLI dependencies
- `checks` proves readiness before execution
- `env` declares runtime environment requirements
- `policies.env` supplies approved env values today
- `policies.provisioning` supplies approved source selection for provisioning

## Proposed Policy Shape

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

## Concrete Flow

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
        package: nodejs
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

## Source Meaning

`source` should identify the approved provisioning origin, such as:

- an internal mirror
- a private registry
- an enterprise manager
- a vendor-approved source
- an offline bundle or artifact cache

It should not encode raw download scripts or ad hoc shell commands.

### Windows

- `choco` can use `source_config.feed` for an approved internal feed or mirror
- `winget` can use `source_config.source_name` for an approved Windows source
- `scoop` can use `source_config.bucket_name` and `bucket_url` for an approved Windows bucket

### Linux

- `apt` can use `source_config.sources_list` for approved `deb ...` entries
- `dnf` can use `source_config.baseurl` and `repo_id` for an approved repo mirror

### macOS

- `brew` can use `source_config.tap_name` and `tap_url` for an approved Homebrew tap

## Expected Behavior

When this layer exists, ota should be able to:

- explain which source is approved for a requested runtime or tool
- reject unapproved sources
- prefer org policy over repo inference when the policy is explicit
- record the chosen source in receipts and diagnostics
- keep the decision deterministic and reviewable

## Command Relationship

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

## Why This Is Separate from Env Policy

Env policy answers:

- what values should execution use

Provisioning policy answers:

- where should the repo’s declared tools and runtimes come from

That separation keeps policy readable and keeps the shipped `env` layer honest.

## Exit Criteria

This spec becomes implementation-bound when ota can:

- resolve a repo-declared runtime or tool through an approved source
- refuse unapproved sources
- explain the chosen source in diagnostics and receipts
- keep the current `policies.env` behavior unchanged
