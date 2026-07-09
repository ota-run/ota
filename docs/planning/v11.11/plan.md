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

# V11.11 Plan

Status: planned follow-on trust slice.

Release target:

- follow-on slice after `v11.10`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.9 plan](../v11.9/plan.md)
- [V11.10 plan](../v11.10/plan.md)
- [Execution receipt](../../spec/execution-receipt.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.11 theme:

- machine-readable proof boundaries and not-proved scope

This slice makes Ota's narrowing posture travel with the artifact instead of living only in
engineering notes, blog posts, or human interpretation.

The product goal is not:

- broader proof claims
- pretending every passing lane proves the whole repo
- replacing receipts or proof artifacts with prose summaries
- hiding narrow proof behind one generic `success` field

The product goal is:

- publish what a proof actually covered
- publish what it did not cover
- classify the proof boundary honestly
- let downstream humans, CI systems, and agents consume that boundary without reading a blog post

## Canonical product principle

Bounded honesty must be machine-readable.

If Ota proves only a narrow lane, the contract and proof surface should say so explicitly.

The mature claim is not:

- "this repo passed"

It is:

- this exact lane was exercised
- this exact execution boundary was covered
- these exact proof obligations were satisfied
- these adjacent obligations were intentionally not proved here

## Problem statement

Ota already pressures honest narrowing well:

- narrow task proof instead of repo-global overclaim
- lane-specific runtime proof instead of vague "works on my machine"
- separate contract drift, readiness, and execution truth

But one important trust gap remains.

Today the boundary of a proof is still too easy to over-read later because the contract and proof
artifacts do not yet carry one strong first-class answer to:

- what did this proof actually prove?
- what did it explicitly not prove?
- was this slice proof, lane proof, runtime proof, or broader integration proof?

Without that, today's honest narrow proof can become tomorrow's silent overclaim.

## Included capabilities

- first-class machine-readable proof scope
- first-class machine-readable not-proved boundary
- honest proof classification for narrow versus broader proof posture
- proof scope attached to the canonical contract/proof surfaces, not only human commentary
- phase-aligned JSON output so proof coverage and not-proved scope are inspectable by automation

## Non-goals

- do not invent one universal proof taxonomy for every future lane up front
- do not claim that every repo must declare every excluded thing manually
- do not collapse proof scope into semantic snapshot or governance verdict identity
- do not use this slice to widen execution capability or replay trust directly

## Core product gaps

### 1. Honest narrow proof still over-travels by implication

An engineering note can say:

- narrow .NET restore proof
- not full runtime proof
- not database-backed verification

But if the contract and proof artifact do not carry that same boundary, later readers and agents
can silently infer a broader claim than Ota actually proved.

### 2. Proof success is still flatter than it should be

Today a successful lane can still look too much like a repo-global pass when what Ota really
knows is narrower:

- one task passed
- one workflow passed
- one runtime path passed
- one integration surface was intentionally not exercised

V11.11 should publish that shape directly.

### 3. Narrowing posture is still stronger in prose than in machine truth

Ota already pushes operators toward truthful narrowing, but the machine-readable artifact needs to
carry the same truth so CI, agents, and later replay/refusal layers do not have to infer it.

## Proposed implementation order

1. define the first honest proof-scope model
2. define the corresponding not-proved model
3. attach both to the canonical proof/receipt JSON surfaces
4. pressure-test on repos that intentionally carry narrow proof
5. only then widen the taxonomy if real repos need it

## Proposed implementation slices

### 1. First honest proof-scope model

Direction:

- start with a narrow, truthful scope model such as:
  - `task`
  - `workflow`
  - `runtime_path`
  - `integration_path`
- require that the scope identify the exact selected lane and execution boundary it covers
- keep source identity, semantic snapshot identity, and proof scope separate because they answer
  different questions

### 2. First honest not-proved model

Direction:

- publish adjacent exclusions as explicit not-proved boundary, not only implicit absence
- keep the first shape narrow and machine-readable, for example:
  - `functional_runtime_not_proved`
  - `database_path_not_proved`
  - `external_network_path_not_proved`
  - `broader_repo_completion_not_proved`
- prefer explicit reason families over prose-only notes where Ota can already classify honestly

### 3. Proof classification without overclaim

Direction:

- classify proof posture honestly, for example:
  - `slice_proof`
  - `lane_proof`
  - `runtime_proof`
  - `integration_proof`
- do not force a broader class when Ota only has narrow evidence
- keep this classification derived from the actual exercised lane and published not-proved
  boundary, not from operator aspiration

### 4. Canonical output placement

Direction:

- attach proof-scope and not-proved boundary to the canonical proof/receipt JSON surfaces
- keep the human-readable summary aligned with the same fields instead of inventing a second
  narrative-only explanation
- ensure future docs and engineering notes can point to the artifact directly

## Acceptance bar

V11.11 is complete when:

- Ota can publish the covered proof boundary and adjacent not-proved boundary for at least one
  real narrow-proof repo lane
- that boundary is machine-readable on the canonical proof/receipt surface
- the classification does not overclaim beyond what the exercised lane actually proved
- engineering notes no longer need to carry the only truthful statement of proof scope

## Pressure-test target

The first real bar should be a repo where Ota intentionally proves a narrow lane and must not
overclaim broader success.

Strong examples include:

- a .NET restore / build slice that intentionally stops short of full runtime proof
- a repo with deterministic verification but intentionally unproved live external paths

The point is not breadth. The point is proving that Ota can carry bounded honesty as contract and
artifact truth.
