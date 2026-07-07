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

# V11.9 Plan

Status: planned.

Release target:

- planned slice after `v11.8`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.4 plan](../v11.4/plan.md)
- [V11.5 plan](../v11.5/plan.md)
- [V11.6 plan](../v11.6/plan.md)
- [V11.7 plan](../v11.7/plan.md)
- [V11.8 plan](../v11.8/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Execution receipt](../../spec/execution-receipt.md)

V11.9 theme:

- governance truth reconciliation and evidence classes

This slice tightens the trust model for the governance surfaces already introduced in V11.4
through V11.7.

The product goal is not:

- more governance fields for their own sake
- a second governance model beside V11.4
- re-litigating whether V11.4 or V11.7 were valid slices

The product goal is:

- make governance output harder to drift from the exact decision path that emitted it
- distinguish caller-supplied assertions from runner-derived and runner-attested truth
- add reconciliation checks so downstream consumers can trust that the claimed enforcement path
  actually fired

## Canonical product principle

Structured governance is only useful if it stays faithful to the branch that made the decision.

That means:

- the contract remains the canonical declared execution-governance spec
- V11.4 remains the canonical machine-readable governance model
- V11.7 remains the audited-crossing model for allowed heavier execution
- V11.9 tightens how those existing governance records are emitted, typed, and reconciled

The mature rule is:

- decision-site truth should be emitted at the decision site, not reconstructed later from a
  second read of mutable state
- caller-supplied fields should not be treated as equal to runner-derived or runner-attested
  evidence
- downstream systems should be able to tell whether a field is asserted, derived, or attested

What this does not mean:

- creating a parallel JSON export just for reconciliation
- forcing every narrative field to become machine-verified
- collapsing receipt, proof, and governance into one blob
- claiming that all governance fields are equally authoritative

## Problem statement

V11.4 through V11.7 gave Ota:

- portable governance verdicts
- merge-oriented governance projection
- harness-facing capability export
- audited crossing records and reason capture

That foundation exists.

What still remains weaker than it should be is the trust bar after those fields ship.

Two concrete failure modes still matter:

1. governance drift after the decision branch changes
- a runner branch is edited
- the serializer or assembler is not updated
- the JSON still emits a confident but stale governance field

2. evidence-class ambiguity
- a caller-supplied `--reason` is useful and attributable
- but it is still a caller assertion, not verified system truth
- if the schema does not distinguish that explicitly, downstream consumers will over-trust it

V11.9 is the slice for making that trust boundary explicit.

## Included capabilities

- decision-site governance emission rules
- evidence-class typing for machine-readable governance fields
- reconciliation checks between claimed governance output and fired enforcement path
- preflight/post-execution consistency checks for governance records
- explicit separation between caller assertion, runner derivation, and runner/harness attestation
- additive machine-readable state for reconciliation success, failure, or unknown posture

## Non-goals

- do not replace V11.4 as the canonical governance model
- do not replace V11.7 as the canonical audited-crossing model
- do not turn free-text operator reason into machine-verified truth
- do not require every field to be emitted only once globally if the same field is legitimately
  carried in more than one artifact
- do not introduce enterprise approval workflow here

## Core product gaps

### 1. Governance output can still be assembled too late

If the governance block is assembled after the decision branch from a second read of state, then:

- the output can drift silently
- a stale serializer can lie with machine confidence
- downstream consumers may trust the wrong thing more than a human comment

V11.9 should make the authoritative emission rule explicit:

- the record should be created or stamped on the same authoritative execution line where the
  refusal, crossing, merge-gate classification, or other governance decision is actually made

### 2. Evidence classes are still too implicit

Today a field like `reason` can be machine-readable without being machine-verified.

That is still progress, but the schema should say so.

V11.9 should make the evidence-class boundary explicit instead of leaving downstream systems to
guess.

### 3. Claimed enforcement still needs reconciliation

It is not enough for the record to say:

- this boundary was crossed
- this refusal reason applied
- this merge gate fired

Ota should also be able to say whether:

- the claimed enforcement path actually fired
- the emitted record still matches the decision hook that ran

## Proposed implementation slices

### 1. Decision-site emission rule

Define a strict product rule for governance records:

- authoritative governance records are emitted from the same decision path that made the verdict
- later serializers may carry or render those records, but they do not infer or recreate the
  authoritative decision content from scratch

Direction:

- refusal records are created where refusal is decided
- crossing records are created where crossing is decided and allowed
- merge-gate / required-lane records are created where the governing decision is classified

### 2. Evidence-class model

Add explicit machine-readable evidence classes for governance fields.

Minimum direction:

- `asserted`
  - caller or operator supplied
- `derived`
  - runner/system determined from contract, runtime state, or policy state
- `attested`
  - emitted by the enforcing boundary/harness at the point of decision or execution

This does not require every field to use all classes.

It does require Ota to stop treating narrative caller input and verified boundary output as
indistinguishable machine truth.

### 3. Field-class guidance

Define the expected class posture for the main V11 governance fields.

Direction:

- selected task/workflow lane:
  - `derived`
- refusal reason family:
  - `derived`
- crossing required / not required:
  - `derived`
- crossing classification:
  - `derived`
- runner attestation posture:
  - `attested`
- caller-supplied `--reason` text:
  - `asserted`
- reason presence / absence:
  - `attested`
- proof or receipt artifact presence:
  - `derived` preflight expectation
  - `attested` or `derived` post-execution evidence depending on the artifact boundary

The exact field map can stay additive first, but the split must be explicit.

### 4. Reconciliation checks

Add reconciliation posture to the governance story.

Direction:

- the system should know whether the authoritative decision hook fired
- the emitted governance record should be checked against that hook identity or equivalent
  authoritative path
- consumers should see whether reconciliation is:
  - `satisfied`
  - `mismatch`
  - `unknown`

This is not a second governance model.
It is a trust check on the canonical one.

### 5. Preflight and post-execution consistency

Keep the phase model explicit:

- preflight can claim:
  - what should happen
  - what boundary applies
  - what evidence will be required
- post-execution can claim:
  - what did happen
  - what record fired
  - whether the expected record and the emitted record reconcile

The important part is:

- absence must not be mistaken for success
- caller assertion must not be mistaken for boundary attestation
- a fired hook and an emitted record should be linkable without re-scraping execution state

### 6. Crossing-specific tightening

Apply the same trust rule explicitly to V11.7 crossing records.

Direction:

- crossing record remains the authoritative anchor
- caller `--reason` remains additive narrative context
- reason should carry an evidence class so consumers know it was asserted, not verified
- reason presence, attachment timing, and record emission should still be attested by the runner
- no later receipt formatter should invent or re-classify the crossing independently

### 7. Merge/CI-oriented tightening

Apply the same trust rule to merge-facing governance surfaces from V11.5.

Direction:

- the required-lane or merge-gate record should point back to the same governing decision path
- the emitted merge-facing governance block should be able to state whether it reconciles with the
  actual check-selection logic that fired
- no provider-specific second truth should outrank the canonical Ota record without being called
  out explicitly

## Acceptance bar

V11.9 is complete when:

- Ota has an explicit rule that authoritative governance records are emitted from the same
  decision path that made the verdict
- the canonical governance model can distinguish at least `asserted`, `derived`, and `attested`
  field posture
- caller-supplied narrative fields such as `--reason` are explicitly typed as non-authoritative
  assertion context
- refusal, crossing, and merge-facing governance records can publish reconciliation posture against
  the decision hook that fired
- preflight and post-execution governance semantics remain phase-accurate while still exposing
  reconciliation state
- downstream consumers no longer need to assume all machine-readable governance fields are equally
  authoritative
- V11.9 tightens the trust model without inventing a second governance taxonomy beside V11.4

## Follow-on boundary

V11.9 is still OSS governance trust refinement.

What can later build on top of it is:

- stronger provider-specific attestation or sandbox compilation targets
- enterprise approval and grant-retention surfaces
- broader audit/reporting layers that can rely on evidence-class-aware governance records instead
  of flat JSON claims
