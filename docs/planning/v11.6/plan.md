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

# V11.6 Plan

Status: planned.

Release target:

- planned slice after `v11.5`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.4 plan](../v11.4/plan.md)
- [V11.5 plan](../v11.5/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.6 theme:

- harness and sandbox capability integration

This slice makes Ota consumable by execution harnesses that need a bounded callable surface.

The product goal is not:

- build Ota’s own sandbox product
- fully define compiled filesystem and egress enforcement policy

The product goal is:

- let a harness consume Ota’s declared governance truth and expose only the allowed execution
  surface

## Canonical product principle

Ota should describe the allowed execution surface clearly enough that another execution boundary
can enforce it without guessing.

That means:

- the contract remains the spec
- V11.4 defines the canonical governance model
- V11.6 publishes a harness-oriented profile derived from that canonical model
- harnesses and sandboxes consume that truth
- deeper runtime policy compilation stays a follow-on slice, not a vague side effect of export

## Problem statement

Even if local Ota enforcement exists and CI enforces merge rules, runtime systems still need to
know:

- what tasks or workflows are callable in agent mode
- what refusal conditions exist
- what mode/context/backend is required
- what writable/protected boundaries matter
- what effect or egress class the selected path implies

Without a deliberate harness-facing surface, each runner will invent its own approximation.

V11.6 is the slice for defining that integration contract.

## Included capabilities

- machine-readable callable task/workflow surface for agent mode
- explicit refusal conditions and closure-safety results for harness consumption
- mode/context/backend requirements in portable form
- writable/protected boundary export in portable form, authoritative where the contract already
  owns it and advisory only where Ota cannot yet claim hard runtime control
- effect-class / external-state / network posture in portable form

## Non-goals

- do not build a general-purpose sandbox manager in OSS
- do not require one specific runner vendor
- do not pretend Ota itself controls host capability boundaries outside its own execution path
- do not overload this slice into org policy distribution
- do not create a second machine-readable governance taxonomy parallel to V11.4

## Core product gaps

### 1. Safe tasks are still too metadata-like outside Ota itself

Even after `v11.3`, many external runners would still only see:

- a list of tasks
- a recommendation to prefer the safe ones

That is not enough.

### 2. Capability boundaries need declared execution truth

A harness needs to know more than a task name.
It needs:

- callable lane
- required mode/context/backend
- likely effects
- refusal conditions

### 3. External runners should not re-derive Ota semantics

If every harness guesses what `safe_tasks`, effects, or refusal conditions mean, the contract is
back to being advisory instead of enforceable.

### 4. Harness export must stay a derived profile, not a second truth

The harness-facing export should not invent new meanings for:

- safety
- refusal
- effects
- boundaries
- execution requirements

It should be a constrained consumer profile over the canonical V11.4 governance model plus
existing contract metadata.

## Proposed implementation slices

### 1. Callable-surface export

Add a machine-readable surface that declares:

- allowed tasks in agent mode
- allowed workflows in agent mode
- effective closure-safe callable units

This export should be explicitly defined as a derived profile over:

- V11.4 governance truth
- existing contract metadata
- existing task/workflow execution truth

### 2. Execution requirements export

Include:

- effective mode
- effective context/backend
- runtime/proof expectations where relevant

### 3. Boundary and effect export

Include:

- authoritative writable/protected boundary fields where the contract already declares them
- informational boundary hints only where Ota cannot yet claim hard authority
- effect classes
- external-state posture
- network / egress-relevant posture where already modeled

This slice should stop at export.

It should not yet claim:

- default-deny network enforcement
- compiled outbound allowlists
- writable mount compilation

Those stronger runtime-policy targets belong to the follow-on sandbox-compilation slice:

- [V11.8](../v11.8/plan.md)

### 4. Refusal and review export

The harness should be able to know:

- which requests would be refused
- which requests require review or are outside the callable surface

That should come from Ota directly, not a prompt convention.

## Acceptance bar

V11.6 is complete when:

- Ota can export an agent-callable execution surface in stable machine-readable form
- harnesses can consume that surface without scraping human output
- refusal conditions and execution requirements are explicit enough that external runners do not
  need to guess Ota semantics
- the harness-facing export is explicitly derived from the V11.4 governance model instead of
  defining a parallel taxonomy

## OSS / enterprise boundary

This harness-facing integration surface stays in OSS.

What can later build on top of it in enterprise is:

- centralized policy rollout
- org-wide runner integration management
- fleet visibility and audit
- exception handling and control-plane orchestration

The next OSS slice before those enterprise approval and exception systems should be:

- [V11.7](../v11.7/plan.md): audited execution boundary crossings
- [V11.8](../v11.8/plan.md): sandbox policy compilation from the execution contract
