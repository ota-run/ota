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

# Trust And Governance Hardening

Status: planned.

This document turns the current pressure-test maturity gaps into one explicit Ota work program.

The goal is not feature count. The goal is trust:

- proof output must say what actually failed
- env/runtime governance must be modeled as contract truth instead of shell glue
- doctor and validator must surface low-maturity authoring before it becomes repo drift

## Why this exists

Pressure-test repos are no longer mainly exposing missing basic capability.

The remaining gaps are mostly:

- trust gaps
- governance gaps
- weak authoring surfaces that still force shell glue where Ota should own the truth

These gaps should not be handled as scattered repo-local fixes. They need one product program.

## The seven gaps

1. Proof phase truth is still too loose.
2. Env compatibility policy is still too shell-shaped.
3. Ota does not yet warn strongly enough against replaceable shell glue.
4. Env overlay transformation is still not a first-class governed surface.
5. Adapter/runtime input ownership is still partly carried in shell bodies.
6. Proof root-cause diagnostics are still too narrow.
7. Governance advisories are still not productized broadly enough.

## Workstream A: Proof Trust

### Problems

- `ota proof runtime` can report a coarse phase even when the actual failure happened earlier in
  `prepare`, `checks`, or setup gating.
- proof JSON/text can stop at “not ready” when the stronger truth is already available from logs
  or the blocking contract node.
- the same proof failure class can currently represent very different operator actions.

### Proposed product surface

- make proof phases explicit and machine-stable:
  - `preconditions`
  - `prepare`
  - `setup`
  - `services`
  - `run`
  - `readiness`
  - `cleanup`
  - `interrupted`
- add a root-cause classifier surface that stays advisory but structured:
  - `failure_class`
  - optional `likely_cause`
  - optional `evidence`
- keep `doctor.json`, `topology.json`, and `up.log` canonical; the proof wrapper points to them
  instead of duplicating them

### Acceptance bar

- proof failures caused by checks or env/bootstrap prep do not report as runtime-readiness failures
- proof JSON can distinguish “blocked by repo check” from “runtime started but never became ready”
- likely-cause detection covers at least loopback drift, bind conflicts, clean early exit, and
  missing tool/runtime cases

## Workstream B: Env Governance

### Problems

- repos still encode env compatibility policy as `grep`, `sed`, or ad hoc scripts
- `env_files` improved task-path ownership, but not env assertions or higher-level transforms
- env overlays are still partly procedural instead of contract-native

### Proposed product surface

- first-class env assertion surface for contract checks and doctor:
  - key must exist
  - key must not equal one of these values
  - URL/DSN host must not be loopback
  - URL/DSN host must match one of these allowed names
  - key must match a literal, regex, or derived host class
- first-class env overlay transformation surface for deterministic repo-owned rewrites:
  - target file
  - source template optional
  - ordered set/replace/remove operations
  - future workflow-scoped overlay ownership
- preserve the current `ensure_env_file` surface as the finite bootstrap primitive, not the full
  env-governance language

### Acceptance bar

- common env compatibility checks no longer need `grep` shells
- common env overlay rewrites no longer need `sed` shells
- workflows can declare overlay truth without hiding adapter-specific env ownership in task bodies

## Workstream C: Contract Governance

### Problems

- authors can still write truthful-but-low-maturity contracts with shell glue that Ota can model
- there is not yet a strong maturity signal when a contract uses shell bodies for modeled
  bootstrap/governance problems
- contract quality still relies too much on human review instead of productized advisories

### Proposed product surface

- validator/doctor advisories for replaceable shell glue, including:
  - env-file mutation that should be `ensure_env_file` or overlay transforms
  - fake aggregate tasks that should be `aggregate`
  - long-running service processes modeled in `run` instead of `launch`
  - shell-carried adapter env inputs that should be contract-owned
- optional maturity-class wording in doctor or validate output:
  - `truthful but low-governance`
  - `replaceable shell glue`
  - `shell-owned adapter truth`

### Acceptance bar

- pressure-test repos get actionable warnings before CI/runtime proof exposes the weakness
- advisories recommend one stronger Ota surface, not generic “clean this up” messaging

## Workstream D: Adapter Ownership Cleanup

### Problems

- Compose env-file usage and similar adapter inputs are still too often expressed through shell
  command flags
- workflow truth can still depend on adapter-specific shell syntax instead of contract structure

### Proposed product surface

- identify adapter inputs Ota should own declaratively first, starting with:
  - Compose env-file ownership
  - workflow/runtime-scoped adapter env overlays
  - future structured adapter input references instead of free-form shell flags
- keep the contract boundary explicit: Ota should own input truth, not re-implement every adapter
  feature

### Acceptance bar

- common Compose env-file lanes do not require shell-owned `--env-file` truth as the primary model
- docs/examples stop teaching shell-first adapter ownership where a contract surface exists

## Mapping Back To The Seven Gaps

| Gap | Workstream |
| --- | --- |
| 1. Proof phase truth is still too loose | A |
| 2. Env compatibility policy is still too shell-shaped | B |
| 3. Ota does not yet warn strongly enough against replaceable shell glue | C |
| 4. Env overlay transformation is still not a first-class governed surface | B |
| 5. Adapter/runtime input ownership is still partly carried in shell bodies | D |
| 6. Proof root-cause diagnostics are still too narrow | A |
| 7. Governance advisories are still not productized broadly enough | C |

## Rollout Order

The recommended implementation order is:

1. Proof Trust

- This closes the sharpest trust leak first.
- It also gives later env/governance work better failure reporting immediately.

2. Env Governance

- This removes the highest-volume shell glue pressure from pressure-test repos.
- It creates the stronger contract target that governance advisories can recommend.

3. Contract Governance

- Warnings are more credible once the stronger replacement surfaces actually exist.

4. Adapter Ownership Cleanup

- Do this after env governance so adapter cleanup can project onto real contract surfaces instead
  of inventing parallel ones.

## Scope Boundaries

### In scope

- proof phase precision
- proof root-cause classification
- env assertion modeling
- env overlay transformation modeling
- governance advisories for replaceable shell glue
- adapter-input ownership cleanup where the contract can truthfully own it

### Not in scope

- full generic policy engine for arbitrary file mutation
- full Compose reimplementation inside Ota
- replacing all shell tasks with structured bodies
- speculative provider-specific governance surfaces beyond the current pressure-tested need

## Changelog Boundaries

When this program ships incrementally, changelog entries should stay grouped by workstream:

- `proof trust hardening`
- `env governance hardening`
- `contract governance advisories`
- `adapter input ownership cleanup`

Do not describe these as isolated repo-specific fixes. They are Ota product maturity changes.

## Success Criteria

This program is complete when:

- proof failures tell operators exactly which stage failed and why
- common env compatibility policy is contract-native instead of grep-driven
- common env overlay rewrites are contract-native instead of sed-driven
- validator and doctor warn on shell glue that Ota can replace
- docs/examples prefer governed contract surfaces over shell-first workarounds
- pressure-test repos stop widening Ota primarily through env/bootstrap shell glue
