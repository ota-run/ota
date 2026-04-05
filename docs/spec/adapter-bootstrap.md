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

This document defines the policy layer for how Ota should obtain its own provisioning
adapters when the host or container does not already have them installed.

This is intentionally separate from repo provisioning:

- repo provisioning answers what the repo needs
- adapter bootstrap answers how Ota gets the adapter binary it needs to satisfy that repo

That separation keeps repository readiness honest and keeps adapter installation policy-controlled.

## Why this exists

The current built-in provisioning adapters are useful only when the selected execution target
already has the adapter command available. If a repo wants `brew`, `mise`, `asdf`, `sdkman`,
`uv`, `winget`, `choco`, `scoop`, `apt`, `dnf`, or `pacman`, Ota still needs the corresponding
adapter binary or package manager on the machine or in the container image.

An adapter bootstrap policy would let an organization declare where those adapter binaries may
come from without turning Ota into a hidden workstation manager.

## Non-goals

- no silent installs
- no arbitrary shell fragments as policy
- no repo-local override for adapter bootstrap defaults
- no general package-manager replacement
- no hidden mutation during `doctor`

## Relationship to current policy surfaces

- `policies.provisioning` says where repo prerequisites may come from
- `policies.adapter_bootstrap` says where Ota may obtain the adapter binary that performs that work
- `policies.env` remains the shipped env-value resolver

## Proposed shape

The policy should stay explicit and source-oriented:

```yaml
policies:
  adapter_bootstrap:
    brew:
      source: approved-manager
      approved_versions:
        - "4.4"
    mise:
      source: internal-mirror
      approved_versions:
        - "2024.12"
```

In that example:

- `brew` and `mise` are the adapters Ota may need to bootstrap
- `source` names the approved origin for the adapter binary itself
- `approved_versions` limits which adapter versions are allowed

## Current implementation

Ota now validates `policies.adapter_bootstrap`, can resolve a plan for missing adapters,
and will bootstrap an approved adapter source before retrying repo provisioning.

The first shipped path reuses the built-in provisioning backends to install the missing
adapter binary from the approved source in policy.

## Expected behavior

When this layer is used, Ota should be able to:

- tell the user which adapter binary is missing
- explain the approved source for that adapter
- refuse unapproved adapter sources
- keep adapter bootstrap separate from repo provisioning
- record the selected bootstrap source in diagnostics and receipts

## Scope boundaries

Adapter bootstrap policy should apply to:

- host-side adapter availability
- container-side adapter availability when the execution target is container-backed

It should not:

- decide repo runtime versions
- replace the provisioning policy
- invent new install commands per repo

## Exit criteria

This spec becomes implementation-bound when Ota can:

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
