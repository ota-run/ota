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

# Adapter Bootstrap Policy

Status: implemented.

This document defines the policy layer for how ota should obtain its own provisioning
adapters when the host or container does not already have them installed.

This is intentionally separate from repo provisioning:

- repo provisioning answers what the repo needs
- adapter bootstrap answers how ota gets the adapter binary it needs to satisfy that repo

That separation keeps repository readiness honest and keeps adapter installation policy-controlled.

## Why this exists

The current built-in provisioning adapters are useful only when the selected execution target
already has the adapter command available. If a repo wants `brew`, `mise`, `asdf`, `sdkman`,
`uv`, `winget`, `choco`, `scoop`, `apt`, `dnf`, or `pacman`, ota still needs the corresponding
adapter binary or package manager on the machine or in the container image.

An adapter bootstrap policy would let an organization declare how ota may install missing
adapter binaries without turning ota into a hidden workstation manager.

## Non-goals

- no silent installs
- no arbitrary shell fragments as policy
- no repo-local override for adapter bootstrap defaults
- no general package-manager replacement
- no hidden mutation during `doctor`

## Relationship to current policy surfaces

- `policies.provisioning` says where repo prerequisites may come from
- `policies.adapter_bootstrap` says where ota may obtain the adapter binary that performs that work
- `policies.env` remains the shipped env-value resolver

## Proposed shape

The policy should stay explicit and backend-oriented:

```yaml
policies:
  adapter_bootstrap:
    brew:
      source: brew-bootstrap
      approved_versions:
        - "4.4"
    mise:
      source: mise-bootstrap
      approved_versions:
        - "2024.12"
```

In that example:

- `brew` and `mise` are the adapters ota may need to bootstrap
- `source` names the bootstrap backend ota should use for that adapter
- `approved_versions` limits which adapter versions are allowed after bootstrap

## Current implementation

ota now validates `policies.adapter_bootstrap`, can resolve a plan for missing adapters,
and will bootstrap an approved source-manager backend before retrying repo provisioning.

The shipped bootstrap backends are named separately from the repo provisioning backends:

- `brew-bootstrap`
- `asdf-bootstrap`
- `mise-bootstrap`
- `sdkman-bootstrap`
- `uv-bootstrap`
- `choco-bootstrap`
- `scoop-bootstrap`

Those bootstrap backends install the missing adapter binary first, then ota retries repo
provisioning with the now-available manager.

## Expected behavior

When this layer is used, ota should be able to:

- tell the user which adapter binary is missing
- explain the approved source for that adapter
- refuse unapproved adapter sources
- keep adapter bootstrap separate from repo provisioning
- record the selected bootstrap source in diagnostics and receipts

## Source bootstrap in practice

When the source manager itself is missing, policy can approve a bootstrap backend first and then
let repo provisioning use the newly available manager.

Example:

```yaml
policies:
  adapter_bootstrap:
    brew:
      source: brew-bootstrap
    sdkman:
      source: sdkman-bootstrap
  provisioning:
    node:
      source: brew
      approved_versions:
        - "22"
    java:
      source: sdkman
      approved_versions:
        - "21"
```

In that example:

- `brew-bootstrap` installs `brew` first when it is missing
- `sdkman-bootstrap` installs `sdkman` first when it is missing
- repo provisioning then uses the approved source managers to install the declared tools and runtimes

## Scope boundaries

Adapter bootstrap policy should apply to:

- host-side adapter availability
- container-side adapter availability when the execution target is container-backed

It should not:

- decide repo runtime versions
- replace the provisioning policy
- invent new install commands per repo

## Exit criteria

This spec becomes implementation-bound when ota can:

- resolve an adapter binary through an approved source
- report that source in doctor output
- bootstrap the adapter in the selected execution target
- keep repo provisioning behavior unchanged

## Implementation notes

The runtime path is intentionally narrow:

- adapter bootstrap is policy-controlled
- bootstrapping happens before repo provisioning retries
- repo provisioning still uses the existing adapter families
- no hidden installs happen when there is no approved bootstrap source
