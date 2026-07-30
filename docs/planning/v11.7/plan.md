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

# V11.7 Plan

Status: partially implemented. Boundary crossing records and provenance are shipped; reusable
grant authority, crossing-time liveness/scope checks, and authorizer binding remain open.

Deferral: this remaining grant-authority work is explicitly deferred. It is not approval authority
for any active execution or contract-authoring slice, and no later slice may consume a crossing
record as a reusable grant until this plan is resumed and its acceptance bar is completed.

Release target:

- partially implemented continuation after `v11.6`; reusable grant authority remains open

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.3 plan](../v11.3/plan.md)
- [V11.4 plan](../v11.4/plan.md)
- [V11.5 plan](../v11.5/plan.md)
- [V11.6 plan](../v11.6/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Execution receipt](../../spec/execution-receipt.md)

V11.7 theme:

- audited execution boundary crossings

This slice closes the remaining OSS governance gap after runner enforcement, portable governance
verdicts, merge-gate projection, and harness export already exist.

The product goal is not:

- enterprise approvals and waiver workflow

The product goal is:

- make intentional boundary crossing explicit, classifiable, and auditable in OSS
- create a first-class crossing record that anchors later reason, receipt, proof, and enterprise
  approval evidence

## Canonical product principle

Ota should not treat all non-default execution equally.

If an operator or agent intentionally crosses the routine safe/default lane, the crossing should be:

- explicit
- attributable to a requested path
- classifiable
- emitted as harness-authored evidence
- anchored by a boundary-authored crossing record that the crosser cannot author on their own

That means:

- `ota.yaml` remains the canonical execution-governance spec
- V11.3 defines what is refused versus allowed in agent-enforced execution
- V11.4 defines the canonical governance model and phase semantics
- V11.5 and V11.6 let CI and harnesses consume that truth
- V11.7 adds explicit audited crossing records for allowed-but-heavier execution paths
- the contract and its derived governance model, not caller prose alone, determine whether a
  crossing was required and how it should be classified
- reason and runtime evidence attach to the crossing record; they do not replace it

What this does not mean:

- inventing enterprise approval workflow in OSS
- turning every non-safe task into a denied path
- replacing receipts with a separate audit store
- collapsing refusal and allowed escalation into one ambiguous “warning” state
- letting the crosser author the audit truth about their own boundary crossing

## Problem statement

After V11.3 through V11.6, Ota can answer:

- what is callable
- what is refused
- what CI should enforce
- what a harness may expose

What is still weaker than it should be is the execution story for paths that are:

- allowed, but heavier than the routine safe/default lane
- intentionally chosen because the routine lane was insufficient
- relevant to later review, merge, or organizational evidence

Today the contract can distinguish safety and effect posture, but it still does not publish one
first-class OSS record for:

- which boundary was crossed
- which exact lane was crossed
- whether the selected lane required an explicit audited crossing
- whether the crossing was routine, escalated, or exceptional
- who or what principal triggered it
- what grant or approval binding allowed it, where applicable
- why the crossing happened
- which runtime evidence later attached to that crossing

It also does not yet distinguish cleanly between:

- a reusable grant that authorizes a class of crossings while live and in scope
- a fresh crossing record that must be emitted every time the boundary is actually crossed

V11.7 is the slice for making that crossing explicit instead of leaving it as implicit context in
task choice alone.

## Included capabilities

- contract-owned or contract-derived crossing-required truth
- first-class boundary-authored crossing records
- explicit audited crossing intent for allowed higher-risk execution paths
- optional or required machine-readable crossing reason capture
- stable crossing classification in receipts and governance output
- explicit actor/principal attribution in crossing evidence
- exact lane and grant or approval binding capture where applicable
- explicit separation between reusable grant authority and one-use crossing evidence
- grant liveness and scope re-check at crossing time
- explicit linkage from the crossing to the boundary that was crossed
- reason and runtime evidence attachments to the crossing record
- OSS evidence semantics that enterprise approvals and waivers can build on later

## Non-goals

- do not build human approval routing in OSS
- do not build centralized exception management in OSS
- do not force reasons on routine default-safe execution
- do not blur refused execution with allowed-but-audited execution
- do not create a second governance taxonomy outside V11.4
- do not treat caller-authored reason text as the primary audit record

## Core product gaps

### 1. Allowed heavier execution is still under-explained

There is a real difference between:

- default-safe routine execution
- allowed execution that crosses a heavier boundary
- refused execution

V11.3 covers refusal.
V11.7 should cover the middle category explicitly.

### 2. Intent is still reconstructive

Today a reviewer may be able to infer that a heavier lane was used from:

- the selected task
- the effect surface
- the resulting receipt

That is still weaker than an explicit crossing record saying:

- which exact lane was crossed
- which grant or approval binding allowed it, where applicable
- which boundary was crossed
- whether the crossing was routine, escalated, or exceptional
- what reason was supplied as context

The crossing record is the anchor. Reason and runtime evidence are useful only if they point back
to that boundary-authored record.

### 3. OSS evidence needs to be stronger before enterprise approval layers

Enterprise approvals and waivers should not be the first place this truth exists.

OSS should already be able to emit:

- a boundary-authored crossing record
- a separate grant reference model where reusable authority exists
- crossing-required truth
- crossing intent
- crossing classification
- crossing evidence

Then later enterprise layers can add:

- approvals
- waivers
- policy-based exception routing
- fleet-wide audit and reporting

## Proposed implementation slices

### 1. Audited crossing model

Define one additive governance concept for execution centered on a first-class crossing record.

The crossing record is:

- boundary-authored
- immutable after creation except for additive attachments
- single-use execution evidence, never a reusable approval object
- linked to one exact task or workflow lane
- linked to one boundary family and classification
- linked to the actor/principal mode that triggered it
- linked to the authorizer identity or authorizer-attribution state where a reusable grant or
  approval binding exists
- linked to a grant or approval binding where applicable
- linked to the runner attestation context that actually enforced and emitted the crossing
  evidence

This record is the durable anchor for later reason, receipt, proof, and enterprise approval
evidence.

The modeled crossing is:

- allowed
- non-routine relative to the default-safe lane
- worth publishing as explicit evidence

The important distinction is:

- grants may be reused while live and in scope
- crossing records may not be reused
- every crossing emits a fresh boundary-authored record, even when an existing live grant is what
  allowed the crossing

Direction:

- crossing classification values such as:
  - `routine`
  - `escalated`
  - `exceptional`
- stable boundary families such as:
  - `unsafe_task`
  - `heavier_workflow`
  - `external_effect_lane`
  - `runtime_proof_lane`
  - `blackbox_verification_lane`

The exact taxonomy can stay narrow first, but it must be stable and machine-readable.

### 2. Crossing-required truth

Ota should not leave “was an explicit crossing required here?” to runner folklore.

V11.7 should define one canonical source of truth for crossing-required posture:

- first, a contract-owned declaration where the repo explicitly marks a task or workflow lane as
  requiring audited crossing
- second, a narrow contract-derived fallback for existing lanes where the contract already makes
  the heavier boundary explicit and Ota can derive the requirement honestly

The fallback derivation should stay narrow first. Direction:

- agent-safe default lanes do not require crossing
- declared lanes outside the default-safe callable surface but still executable under a heavier
  posture may require crossing
- specifically declared heavier verification or external-effect lanes may require crossing only
  when the contract already exposes that distinction structurally

The important part is:

- no silent runner heuristics as the primary truth
- no caller deciding for itself whether crossing was required
- the repo contract and derived governance model remain canonical

### 3. Boundary-authored crossing record

When a crossing is required or explicitly requested, Ota should create the crossing record at the
moment the boundary is crossed.

Minimum record fields:

- crossing id
- exact task or workflow lane crossed
- boundary family
- crossing classification
- crossing requirement source
- actor mode
- principal attribution state
- authorizer attribution state, where applicable
- runner attestation state
- grant or approval binding reference, where applicable
- grant liveness state at crossing time
- grant scope-evaluation result at crossing time
- created timestamp
- reason state
- evidence attachment state

The important part is:

- the record is emitted by Ota or the harness boundary, not by the crosser
- the exact lane and grant binding are stamped synchronously
- grant liveness and scope are re-checked at crossing time before the crossing is finalized as
  allowed
- reason and runtime evidence attach to this record later or during execution
- routine crossings can create this record cheaply and automatically
- exceptional crossings can require louder reason or approval capture

### 3a. Grant model and liveness

V11.7 should keep grant authority and crossing evidence as separate objects.

Direction:

- a grant is reusable authority, not execution evidence
- a grant may authorize one lane or boundary family for some bounded scope
- a crossing record is emitted fresh for each actual crossing and may reference one grant
- grant reuse is valid only while the grant is:
  - live
  - not revoked
  - not expired
  - still in scope for the selected crossing

The mature rule is:

- cheap routine crossing comes from reusing a live in-scope grant
- loud crossing is triggered when the grant is missing, revoked, expired, or out of scope
- no previous crossing record can be replayed as authority for a new crossing
- grant age alone is not enough; stale standing authority should be cut off by bounded work-unit
  lifetime first, with calendar TTL only as fallback

### 3b. Grant scope dimensions

Grant scope should be machine-readable and narrow enough to avoid stale standing authority.

Direction:

- the core identity of a grant should be:
  - actor
  - action
  - resource
- repo and lane should usually be treated as resource-granularity choices inside that identity
  rather than as drifting parallel grant concepts
- environment should be treated as a hard wall around grant scope, not as an ordinary wildcardable
  dimension
- grant lifetime should default to the justified unit of work where Ota can model it honestly:
  - `this run`
  - `this task`
  - `this batch`
- calendar TTL should be the fallback only when no truthful work-unit boundary exists
- Ota should not hardcode one universal scope dimension if the governance model already exposes a
  narrower truthful boundary

The important part is:

- scope must be evaluated at crossing time, not only when the grant is first created
- crossing evidence should be able to say whether the selected crossing was in scope, not just
  whether some historical grant existed
- grants should be reviewable at a blast radius a human can understand in one breath
- the model should avoid both broad standing authority and approval-fatigue micro-grants
- grants carry the coarse reviewed scope; crossing records carry the fine-grained per-use detail

### 4. Execution-intent capture

Add an explicit opt-in lane for allowed audited crossings.

The exact command spelling can stay narrow, but the product should support a shape like:

- `ota run <task> --reason "..."`
- `ota up --workflow <name> --reason "..."`
- or an equivalent additive crossing flag/value pair if reason and crossing intent need to stay
  separate

The important part is:

- the crossing is intentional, not inferred only from task choice
- the runner/harness records the supplied intent
- the output distinguishes no-reason, optional-reason, and required-reason cases honestly
- reason capture remains additive evidence, not the source of truth for whether crossing was
  required
- caller-supplied reason is preserved as narrative context, not the authoritative audit anchor

### 5. Actor and principal attribution

Crossing evidence should answer more than “a crossing happened.”

It should also publish who or what triggered it, at the OSS level that Ota can claim honestly.

Direction:

- actor mode such as:
  - `human`
  - `agent`
  - `ci`
  - `harness`
- triggering principal attribution where available from the execution path
- explicit authorizer attribution where a reusable grant or approval binding exists
- explicit runner attestation posture for the execution context that enforced and emitted the
  crossing evidence
- explicit distinction between:
  - caller-supplied identity or label
  - runner-known execution mode / principal kind
  - authorizer identity or authorizer-attribution state
  - runner identity or runner-attestation state

The important part is:

- receipts and governance output can answer who or what crossed the boundary
- receipts and governance output can distinguish:
  - the acting principal
  - the human or policy authorizer who granted the reusable authority, where applicable
  - the runner context that actually enforced and attested the crossing
- OSS does not overclaim identity it cannot verify locally
- enterprise can later layer stronger organizational identity, retention, and approvals on top

### 6. Governance-model integration

Do not create a second machine output model.

V11.7 should extend the V11.4 governance model with additive fields for:

- crossing record id
- crossing required / not required
- crossing requirement source:
  - `declared`
  - `derived`
- crossing classification
- crossing classification source:
  - `runner_derived`
- crossing boundary family
- crossing lane id
- crossing actor mode
- crossing principal attribution state
- crossing authorizer attribution state
- crossing runner attestation state
- crossing grant binding state
- crossing grant liveness state
- crossing grant scope state
- crossing grant identity shape
- crossing grant environment boundary state
- crossing grant expiry kind
- crossing intent source:
  - `caller_supplied`
  - `runner_defaulted`
- crossing reason present / missing
- crossing evidence attachment state

Preflight and post-execution semantics must remain phase-accurate.

Classification should not be caller-authored truth.

The mature rule is:

- callers may request or justify a heavier lane
- the runner derives the crossing classification from contract/governance truth plus the selected
  execution path
- if caller intent is preserved, it is preserved as intent metadata, not canonical classification

### 7. Receipt and evidence linkage

Crossing evidence should be carried by the existing evidence story, not outside it.

That means:

- receipts remain harness-authored execution evidence
- governance output remains the portable evaluation layer
- crossing evidence is linked from both surfaces where relevant
- receipt-linked crossing evidence carries actor/principal attribution and reason state alongside
  boundary family and classification
- receipt-linked crossing evidence also carries authorizer attribution and runner-attestation
  posture so later review can separate actor, grant authority, and execution context honestly
- receipt-linked crossing evidence also carries grant binding, liveness, and scope-evaluation
  posture where applicable
- runtime proof and receipt evidence attach to the crossing record instead of floating as separate
  audit claims
- refusal and crossing stay distinct outcomes

### 8. OSS / enterprise boundary

Keep OSS focused on:

- first-class boundary-authored crossing records
- explicit crossing-required truth
- explicit crossing intent
- explicit crossing classification
- explicit actor/principal attribution at the level Ota can honestly verify
- machine-readable evidence

Reserve enterprise for:

- approvals
- waivers
- exception policy rollout
- fleet-wide visibility and reporting

## Acceptance bar

V11.7 is complete when:

- Ota creates a first-class crossing record as the immutable anchor for audited boundary crossings
- the crossing record is boundary-authored, not crosser-authored
- the crossing record stamps the exact lane crossed and grant or approval binding where applicable
- grant authority and crossing evidence remain separate objects
- a fresh crossing record is emitted for every actual crossing, even when a prior grant is reused
- Ota re-checks grant liveness and scope at crossing time instead of treating grants as stale
  standing authority
- grant identity is modeled around actor + action + resource rather than loose approval prose
- environment remains a hard grant boundary rather than a wildcardable scope field
- work-unit expiry is the default grant lifetime where Ota can model it honestly
- crossing evidence distinguishes the acting principal, the authorizer, and the runner attestation
  context instead of smearing them into one identity field
- Ota can answer whether a crossing was required from contract-owned or contract-derived truth
- Ota can distinguish routine execution from allowed audited boundary crossings
- crossing classification is runner-derived governance truth, not merely caller prose
- crossing evidence can attribute the crossing to an actor/principal mode honestly
- a crossing can carry machine-readable reason state without collapsing into receipt prose
- reason and runtime evidence attach to the crossing record instead of replacing it
- governance output publishes crossing posture in a stable additive form
- receipts preserve crossing evidence as harness-authored truth
- refusal remains a distinct execution/governance outcome from allowed audited crossing
- the OSS surface is strong enough that enterprise approval layers can build on it instead of
  inventing the concept later

## Follow-on boundary

V11.7 is the OSS audited-crossing layer.

What can later build on top of it in enterprise is:

- approval routing
- waiver lifecycle
- policy-scoped exception approval requirements
- centralized audit, retention, and fleet-level reporting

What should tighten the OSS trust model after this is:

- [V11.9](../v11.9/plan.md): governance truth reconciliation and evidence classes, so
  boundary-authored crossing records and attached reason/evidence fields stay emitted from the
  same authoritative decision line and clearly distinguish caller assertions from runner-derived or
  runner-attested truth
