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

# Repo-Scoped Provisioning

Status: spec candidate.

This document defines a bounded, repo-scoped provisioning capability for Ota.

The purpose is to let Ota prepare declared repo prerequisites explicitly and predictably,
without turning Ota into a workstation manager or hidden policy engine.

## Goal

ota should help a repo become runnable when its declared prerequisites are missing.

That means:

- reducing first-run failures from missing toolchains
- removing repetitive setup work from onboarding
- keeping the preparation path reviewable for humans
- giving agents a deterministic path for setup decisions

## Non-goals

Provisioning should not become:

- silent workstation management
- arbitrary system-wide installation
- a background daemon
- a hosted control plane
- a policy engine for fleet compliance
- remote orchestration disguised as local setup
- hidden mutation during `doctor`, `init`, or `detect`

## Relationship to existing surfaces

Repo-scoped provisioning extends the current trust path:

- `ota doctor` diagnoses what is missing and whether it is provisionable
- `ota init` and `ota detect` may infer provisioning hints, but must stay honest about confidence
- `ota up` is the user-facing entrypoint
- `ota up --provision` explicitly allows repo-scoped provisioning when the contract already
  declares the needed setup
- `ota up --dry-run --provision` shows the exact preparation actions before any change
- `ota workspace init --bootstrap` remains the workspace contract bootstrap path, not a general
  prerequisite installer

Provisioning must stay separate from workspace bootstrap.

## Contract shape

The repo contract may grow a `provision` block that describes prerequisite intent.

The shape should stay declarative and reviewable. It should express:

- required tools
- version constraints
- preferred manager or installer hints
- whether a prerequisite is required or optional
- what must be verified after provisioning

Examples of provisionable prerequisites:

- Node via `nvm`, `mise`, or `asdf`
- Python via `uv`, `pyenv`, or `mise`
- Java via `sdkman`, `jabba`, or `mise`
- repo-local tooling prerequisites

The contract should describe intent, not arbitrary install scripts by default.

## UX

Recommended command shape:

- `ota doctor` reports missing prerequisites and whether they can be provisioned
- `ota up --dry-run` shows the exact preparation actions that would happen without provisioning
- `ota up --provision` applies only declared repo-scoped prerequisites when provisioning is allowed
- `ota up` remains the main prepare-and-run path

Suggested operator flow:

1. `ota doctor`
1. `ota up --dry-run`
1. `ota up --dry-run --provision`
1. `ota up --provision`
1. `ota run ...`

## Output expectations

Provisioning output should be explicit and reviewable.

It should show:

- what is missing
- what Ota thinks it can provision
- what source or manager it would use
- what it would verify after the change
- whether the action is safe to run now or only safe to preview

If the signal is weak, Ota should say so rather than invent confidence.

## Risks

The main risks are:

- becoming a workstation manager by accident
- making `up` too magical
- overpromising cross-platform install behavior
- drifting into policy enforcement instead of repo readiness
- conflicting with user-managed tools like `nvm`, `asdf`, `mise`, `sdkman`, or `uv`

## Exit criteria

This surface is ready when:

- a repo can declare prerequisite intent cleanly
- `doctor` can say what is missing and whether it can be provisioned
- dry-run is deterministic
- actual provisioning is bounded and reviewable
- humans trust the flow
- agents can use it without guessing

## Why this matters

This feature is the next useful product step before enterprise surfaces.

It reduces onboarding friction, keeps setup explicit, and gives Ota a practical way to help
real repos become runnable without hiding the work from the operator.
