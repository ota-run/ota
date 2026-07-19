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

# V11.5 Plan

Status: implementation and real-repo pressure complete. Required-lane projection, merge-check
identity, workflow drift evaluation, and CI-owned refusal-canary execution are shipped.

Release target:

- completed continuation after `v11.4`; GitHub-managed canary checks are pressure-proven through
  V11.15

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.4 plan](../v11.4/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.5 theme:

- CI and merge-gate projection

This slice moves Ota from local governance evaluation into merge-relevant enforcement scaffolding.

The product goal is not:

- replace every CI system

The product goal is:

- let CI and merge gates enforce contract-owned verification and governance truth without
  re-deriving that truth from ad hoc repo conventions

## Canonical product principle

The repo contract should be able to say what makes the repo mergeable.
CI should enforce that declared truth, not a separate hand-maintained approximation.

That means:

- required verification lanes come from the contract
- required proof / refusal / drift posture comes from the contract and policy
- CI becomes an enforcement consumer of Ota truth, not a parallel source of governance semantics
- Ota-owned merge-check identity is stable even when provider workflow/job naming changes

## Problem statement

Even with strong local enforcement and machine-readable governance output, a repo can still be
merged using:

- stale workflow habits
- partial verification
- unreviewed drift between contract and CI

If Ota stops at the local runner, teams can still route around the contract at merge time.

V11.5 is the slice for turning contract-owned governance into merge-gate consumable truth.

## Included capabilities

- contract-owned required verification lane projection
- machine-readable merge-blocking governance verdicts
- check-summary / annotation-friendly output for CI
- generated CI guidance or snippets where appropriate
- validation surfaces that compare contract-owned required lanes against actual CI enforcement

## Non-goals

- do not build hosted branch-protection management in OSS
- do not become a generic CI vendor abstraction layer
- do not auto-rewrite every repo workflow in this slice
- do not collapse runtime proof and merge policy into one opaque pass/fail bit
- do not rely on brittle provider-specific name matching as the only CI identity model

## Core product gaps

### 1. Mergeability is still too convention-driven

Many repos still rely on:

- CI habits
- manually curated required checks
- inconsistent workflow naming

instead of a contract-owned completion definition.

### 2. Contract and CI can still drift silently

V11.2 and later drift work can warn about divergence, but the merge path still needs stronger
projection of required truth.

### 3. Required verification should be contract-owned

The repo contract should be able to declare:

- which verification lanes are merge-relevant
- whether proof artifacts are required
- whether refusal or unsafe execution should fail the gate
- which contract-declared refusal canaries must prove the agent enforcement boundary remains live

### 4. CI comparison needs stable check identity

If Ota wants to compare contract-required lanes against actual CI wiring honestly, it needs a
canonical identity layer first.

Without that, drift detection becomes brittle:

- workflow filenames drift
- job names drift
- provider check names drift
- renderers emit different display strings

V11.5 should define one Ota-owned merge-check id per required lane, then map that id onto provider
surfaces.

## Proposed implementation slices

### 1. Required-lane projection

Define a contract-owned surface for required verification / completion lanes that CI can consume.

This should not invent a second parallel workflow taxonomy.
It should build from declared tasks, workflows, and completion/governance truth.

### 2. Merge-check identity model

Define one stable Ota-owned identity for each required merge-relevant lane.

Direction:

- one canonical `merge_check_id` per required lane
- one canonical relationship from:
  - contract lane
  - governance verdict
  - provider render / status context / workflow-job mapping

That identity should be the comparison key for:

- generated CI-facing projections
- merge-gate JSON
- drift comparison against actual CI wiring

Provider names are render targets, not canonical truth.

### 3. Merge-gate JSON

Add a machine-readable merge-gate result that answers:

- did all required contract-owned lanes pass
- is required proof present
- is unsafe/refused state blocking
- is contract drift blocking
- which `merge_check_id` values are satisfied, missing, or miswired
- which required enforcement canaries produced `refused_as_expected` versus admitted, failed, or
  were not run

### 4. CI-facing renderers

Provide OSS-friendly renderers such as:

- check summaries
- annotations
- generated snippets
- validation of declared-versus-actual required CI lanes

The exact renderer set can stay narrow, but the merge-gate semantics must be explicit.

### 5. Drift comparison against actual workflows

Do not stop at generation.

Ota should also be able to say:

- the contract says these are merge-required
- the current CI wiring enforces something different
- these provider workflow/job/check surfaces currently map to or fail to map to the expected
  `merge_check_id` values

That is a governance feature, not only a convenience feature.

## Acceptance bar

V11.5 is complete when:

- Ota can project contract-owned required verification truth in a machine-readable CI-facing form
- Ota defines stable merge-check identity for required lanes
- CI can fail on contract-owned merge-blocking governance verdicts without reimplementing Ota
- CI can require both a positive safe lane and a contract-owned refusal canary without wrapping
  `ota run` or `ota up` in provider-specific assertion shell
- Ota can compare contract-required lanes against actual CI wiring honestly

## Follow-on boundary

V11.5 is about merge/CI consumption.

The next slice is:

- [V11.6](../v11.6/plan.md): harness and sandbox capability integration
