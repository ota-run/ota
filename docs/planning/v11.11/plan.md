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

Status: active refinement after the completed foundation.

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

Priority:

- implement this before hydration source/feed posture widening
- proof-boundary truth is the first trust move because silent proof over-read is worse than a loud
  restore or feed failure

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
- keep `not_proved` relative to declared proof scope instead of letting it drift into free-floating
  exclusion prose
- make the proof boundary structured enough that downstream consumers cannot silently over-read a
  green narrow proof as broader repo truth

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

`not_proved` should be relative, not free-floating.

The first honest rule is:

- `scope` says what this proof covered
- `not_proved` says which adjacent declared proof families remain outside that scope
- `not_proved` is not repo-global commentary and is not a second independent taxonomy beside scope

The first honest ownership rule is:

- adjacent declared proof families come from canonical contract-owned lane truth, not runner
  heuristics
- for the first cut, Ota derives them from the selected task/workflow/runtime path and its
  declared neighboring proof families on that same contract-owned lane
- if Ota cannot recover adjacent proof-family truth from the selected contract lane and its
  declared proof obligations, it should omit `not_proved` entries rather than inventing them

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
- one named first JSON carrier for the first honest implementation cut
- a non-collapsible qualified proof verdict so a green narrow proof cannot be read as repo-global
  completion by consumers that only inspect the top-level status

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

### 4. `not_proved` is still under-anchored

If `not_proved` is emitted without an explicit anchor, it can degrade into vague exclusion prose.

The first honest boundary is:

- `not_proved` entries are scoped relative to the declared proof scope
- they should describe adjacent declared proof families outside the selected proof boundary
- they should not act as repo-global completion commentary
- those adjacent families must be recovered from contract-owned proof truth on the selected lane,
  not inferred ad hoc from runtime happenstance

## Proposed implementation order

1. define the first honest proof-scope model
2. define the corresponding relative not-proved model
3. choose one first JSON carrier for the first honest cut
4. attach both to that carrier first
5. pressure-test on repos that intentionally carry narrow proof
6. only then widen the taxonomy or replicate across other artifacts
7. only after that move to hydration source/feed posture in `v11.12`

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
- keep `not_proved` relative to the declared proof scope and adjacent declared proof families,
  not as free-floating repo commentary
- derive adjacent proof-family truth from the selected contract-owned lane and its declared proof
  obligations instead of from runner-side guesswork
- keep the first shape narrow and machine-readable, for example:
  - `functional_runtime_not_proved`
  - `database_path_not_proved`
  - `external_network_path_not_proved`
  - `broader_repo_completion_not_proved`
- prefer explicit reason families over prose-only notes where Ota can already classify honestly

The first honest interpretation should be:

- if scope is `task` or `workflow`, `not_proved` can describe adjacent runtime or broader
  completion families that remain outside the exercised lane
- if scope is `runtime_path`, `not_proved` can describe adjacent integration or broader repo
  completion families outside that runtime boundary
- the relative comparison anchor is the selected contract-owned proof lane plus its adjacent
  declared proof families, not repo-global completion state
- if Ota cannot anchor an exclusion relative to declared scope and adjacent proof families, it
  should omit it rather than invent loose taxonomy prose

The refinement should publish the sharpest contract-derived boundary first:

- boundaries with a direct declared workflow, task, service, or external-state citation outrank
  generic scope remainder
- explicit skipped lanes outrank generic scope remainder, but do not automatically outrank a more
  specific dependency or external-state boundary
- ties use stable contract-local identity ordering, not a hardcoded family preference
- `broader_repo_completion_not_proved` stays last because it is the scope-derived remainder

Skipped lanes are already visible in ordinary execution output. The more dangerous boundary is a
green proof that depended on external state or neighboring dependency truth it never exercised.
Each entry remains relative to the selected proof scope and cites its declared workflow or task
owner; omit it when the contract does not provide that truth.

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

### 3a. Qualified top-level proof verdict

Direction:

- preserve `ok` as the execution outcome for compatibility
- add one terminal post-evaluation `proof_verdict` field to the first carrier:
  - `passed`
  - `passed_with_unproven_boundaries`
  - `failed`
- a successful proof with any `not_proved[]` entries must use
  `passed_with_unproven_boundaries`; consumers must not need to infer that qualification from a
  nested optional array
- `passed` means the selected proof lane completed and evaluated ready with no emitted unproved
  boundary; `passed_with_unproven_boundaries` means the same selected lane passed but carries one
  or more contract-derived exclusions; `failed` means the selected lane did not complete its
  runtime-proof execution or readiness evaluation, with `failure_class` retaining the precise
  reason
- parse, contract-load, and other pre-proof command failures remain outside this terminal carrier;
  they must not be collapsed into `proof_verdict: failed`
- derive this field from the same proof scope and not-proved record at the decision site; do not
  reconstruct it later in output formatting

### 4. First carrier before broader propagation

Direction:

- pick one first JSON carrier for the first implementation cut
- the strongest first carrier is `ota proof runtime --json` because it is the dedicated
  runtime-proof artifact and already owns the selected proof lane, phase, artifacts, and cleanup
  boundary without mixing proof coverage into broader readiness/execution output
- treat `ota up --json`, receipt archive JSON, and other proof/receipt carriers as later
  derivations from the same canonical shape instead of widening all artifact surfaces at once

### 5. Canonical output placement

Direction:

- attach proof-scope and not-proved boundary to the first chosen carrier, then derive outward
  from there
- keep the human-readable summary aligned with the same fields instead of inventing a second
  narrative-only explanation
- ensure future docs and engineering notes can point to the artifact directly

## Acceptance bar

V11.11 is complete when:

- Ota can publish the covered proof boundary and adjacent not-proved boundary for at least one
  real narrow-proof repo lane
- `not_proved` is explicitly relative to declared scope and adjacent proof families instead of
  free-floating exclusion prose
- that boundary is machine-readable on the first chosen JSON carrier
- the classification does not overclaim beyond what the exercised lane actually proved
- a top-level successful proof with boundaries is explicitly qualified as
  `passed_with_unproven_boundaries`, not only `ok: true` plus nested exclusions
- multiple exclusions are ordered by sharpest contract-derived evidence first and generic
  scope-derived remainder last; category names alone must not decide the ordering
- engineering notes no longer need to carry the only truthful statement of proof scope
- downstream consumers can distinguish a green narrow proof from a broader runtime or repo proof
  without relying on narrative prose

## Pressure-test target

The first real bar should be a repo where Ota intentionally proves a narrow lane and must not
overclaim broader success.

Strong examples include:

- a .NET restore / build slice that intentionally stops short of full runtime proof
- a repo with deterministic verification but intentionally unproved live external paths

The point is not breadth. The point is proving that Ota can carry bounded honesty as contract and
artifact truth.
