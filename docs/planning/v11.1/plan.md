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

# V11.1 Plan

Status: planned.

Release target:

- first implementation target after `1.6.22`

Source direction:

- [V11 plan](../v11/plan.md)
- [Execution receipt](../../spec/execution-receipt.md)
- [Doctor finding contract](../../spec/doctor-finding-contract.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Semantic snapshots and correlation](../../spec/semantic-snapshots-and-correlation.md)

V11.1 theme:

- execution governance visibility and proof

This is the first concrete implementation slice inside V11.

The product claim is already directionally true:

- Ota governs how a repo is prepared
- Ota governs what can run safely
- Ota governs what verification ran
- Ota governs what evidence remains afterward

What is still too weak is how quickly that governance becomes visible to operators, CI, and agents.

V11.1 makes that proof legible fast.

## Problem statement

`Execution governance` is a strong phrase, but today too much of its proof is still reconstructive.

The current product has the underlying pieces:

- receipts
- doctor findings
- task safety surfaces
- dry-run previews
- workflow and runtime proof
- machine-readable JSON output

What is still weaker than it should be is the immediate operator answer to:

- what path Ota selected
- what phases ran
- what was safe versus unsafe
- what verification actually happened
- what evidence exists now
- whether the repo was merely executed or actually governed to a completion bar

V11.1 is the slice for turning that from architectural truth into fast visible proof.

## Included capabilities

- stronger receipt discovery and inspection guidance from normal command flows
- clearer task and workflow safety visibility in human and machine output
- explicit staged execution and verification phase visibility
- tighter machine-readable governance summaries for CI and agents
- public governance-first reference and example coverage for the surfaced behavior

## Non-goals

- do not invent a second execution engine
- do not collapse receipts, doctor findings, and semantic contract diff into one overloaded blob
- do not claim that safe task visibility alone makes a repo autonomous for agents
- do not build a generic PR review dashboard
- do not widen execution governance into hosted workflow approval product scope

## Product outcomes

After V11.1, an operator should be able to answer quickly:

- Which contract-owned path did Ota select?
- Which phases ran, in what order?
- Which steps were verification versus setup versus proof?
- Which runnable lanes are safe for routine agent execution, and why?
- What receipt or JSON artifact should CI or an agent keep?

After V11.1, a machine consumer should be able to answer quickly:

- what selected task/workflow/mode/context was used
- what governance phase each step belonged to
- what stop/review signals applied
- where the durable receipt or proof artifact lives

## Core product gaps

### 1. Receipt visibility is still too implicit

Receipts exist, archive cleanly, and now carry stronger snapshot linkage.

What is still too weak is the fast discovery path from ordinary commands:

- where the receipt is
- when to inspect it
- how it relates to the selected execution path

V11.1 should make receipt visibility feel native to execution, not like a secondary expert lane.

### 2. Safe-command policy is still too compressed

Ota already models:

- `agent.safe_tasks`
- effect policy
- execution boundaries
- explicit stop/review semantics

What is still too weak is the visible explanation of safety:

- why a task is safe
- why a task is not safe
- whether the blocker is external effects, setup mutation, proof incompleteness, or boundary drift

V11.1 should make the safety posture legible in `tasks`, dry-run, and doctor-adjacent flows.

### 3. Verification and proof stages are still too easy to infer instead of see

Ota already runs meaningful stages:

- prepare
- setup
- verification
- runtime proof
- receipt/archive

What is still too weak is publishing those stages as the execution-governance story instead of
leaving operators to infer them from logs and summaries.

V11.1 should make staged execution visible in both human and machine surfaces.

### 4. Machine-readable governance proof is still too fragmented

The pieces exist across:

- `ota run --dry-run --json`
- receipts
- doctor JSON
- proof JSON

What is still too weak is the joined governance summary for CI and agents.

V11.1 should not collapse those outputs into one schema, but it should make their relationship
clearer and more consumable.

### 5. Public governance-first guidance is still too diffuse

We already have strong reference material, but the execution-governance proof story is spread across
multiple pages.

V11.1 should make the operator-facing story direct:

- safe execution
- staged verification
- receipts
- machine-readable output
- CI and agent consumption

## Proposed implementation slices

### 1. Receipt visibility and traceability

Strengthen how normal execution surfaces point to:

- selected path
- archived receipt
- proof artifact
- semantic snapshot reference when relevant

Design bar:

- execution should stay primary
- receipt discovery should become immediate
- traceability should be explicit without bloating normal success output

### 2. Safe-task and policy visibility

Widen the human and JSON surfaces for:

- safe task posture
- unsafe reason categories
- review-required stop signals
- selected execution boundary implications

Likely command surfaces:

- `ota tasks --use`
- `ota tasks --safe --use`
- `ota run --dry-run --json`
- `ota doctor`

### 3. Staged execution rendering

Publish clearer stage identity for execution-governance phases.

Expected stage families:

- prepare
- setup
- verify
- proof
- receipt

This should appear in:

- human summaries where useful
- execution receipts
- machine-readable progress and/or summary JSON

### 4. Unified machine-readable governance story

Define and document the machine-consumption story across:

- dry-run selection truth
- execution receipt truth
- proof/runtime truth
- stop/review semantics

This is not a new monolithic schema.
It is a clearer contract for how the existing governance outputs relate.

### 5. Governance-first docs, examples, and reference

Publish the public operator story with:

- one sharp reference page for execution governance visibility and proof
- refreshed examples showing safe-task, dry-run, receipt, and staged execution surfaces together
- examples and first-party guidance that show CI and agent consumption explicitly

## Proposed operator questions

V11.1 should make Ota better at answering these directly:

- What did Ota actually execute?
- What verification stage am I looking at?
- Is this lane safe for routine agent use?
- What proof artifact should I inspect next?
- Did this run stop at setup, verification, proof, or readiness?
- What machine-readable artifact should CI keep to prove what happened?

## Rollout order

1. Tighten receipt visibility and selected-path traceability.
2. Widen safe-task and stop/review visibility.
3. Publish staged execution/proof rendering.
4. Tighten machine-readable governance summaries and documentation.
5. Pressure-test on repos where agent-safe and proof-heavy lanes are both meaningful.

This keeps the work honest:

- traceability first
- safety posture second
- stage proof third
- broader documentation and pressure proof last

## Pressure-test bar

V11.1 is not done until real repos prove that operators can see execution governance quickly.

Every pressure repo used for this slice should prove:

- `ota tasks --use`
- `ota tasks --safe --use`
- `ota run --dry-run --json`
- real task execution on at least one meaningful lane
- receipt/archive inspection after execution
- workflow dry-run and proof lanes where advertised
- CI-usable machine-readable artifact retention for the executed path

## Acceptance bar

- normal execution surfaces point operators clearly to the right receipt/proof artifact
- task and workflow safety posture is visible and understandable without reading the contract first
- staged verification and proof phases are visible in execution-governance output
- CI and agents can consume the governance story without stitching together unrelated heuristics
- public docs and examples make execution governance concrete within the first few operator steps
