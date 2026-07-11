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

# V11.4 Plan

Status: complete.

Release target:

- completed slice after `v11.3`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.3 plan](../v11.3/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Doctor finding contract](../../spec/doctor-finding-contract.md)
- [Execution receipt](../../spec/execution-receipt.md)

V11.4 theme:

- machine-readable governance evaluation output

This slice turns runner-local enforcement into portable governance truth other systems can consume.

The product goal is not just:

- refuse unsafe execution locally

The product goal is:

- publish one canonical governance model with explicit phase semantics that local runners, CI
  systems, bots, and later control-plane integrations can all consume without re-deriving Ota
  semantics from prose or human output.

## Canonical product principle

The contract is the spec.
The runner is one enforcement consumer.
The governance output must be portable enough that other enforcement consumers can use the same
truth.

That means:

- `ota.yaml` remains the canonical declared execution-governance surface
- runner refusal is one expression of that truth
- machine-readable governance output is the transport layer for CI, bots, and platform systems
- phase semantics are explicit, so pre-execution governance is not confused with post-execution
  evidence state

What this does not mean:

- inventing a second policy language outside the contract
- requiring CI or sandboxes to parse rich text summaries
- duplicating the same decision logic independently in multiple wrappers

## Problem statement

After `v11.3`, Ota can enforce agent-safe execution locally.

That is necessary, but not sufficient for OSS execution governance.

Other mandatory chokepoints still need to understand the same truth:

- CI required checks
- merge gates
- bots or automation that decide whether a change is complete
- harnesses that need to expose only the allowed callable surface

Today those systems can inspect pieces of Ota output, but not one deliberately shaped governance
verdict.

V11.4 is the slice for defining that machine-readable governance layer.

## Included capabilities

- a first-class machine-readable governance output model
- explicit safe/unsafe/refused posture for selected tasks and workflows
- closure-aware safety results in JSON form
- explicit proof / receipt expectation status in JSON form
- explicit contract-drift / policy / missing-proof / refusal-state signals for external consumers
- additive command surfaces external systems can call without scraping human output

## Non-goals

- do not implement branch-protection orchestration in this slice
- do not generate CI workflow files in this slice
- do not make this output GitHub-specific
- do not create a second execution engine outside `ota run` / `ota up`
- do not overload receipts so they become the only governance artifact
- do not publish one ambiguous verdict where preflight state and post-execution evidence are mixed
  without phase labeling

## Core product gaps

### 1. Governance truth is still too runner-local

After `v11.3`, the runner can refuse unsafe agent execution.

What still remains weaker than it should be is:

- an external system cannot ask Ota for one canonical governance verdict surface without inferring
  it from multiple commands and output shapes

### 2. Policy/evaluation output is still too reconstructive

External consumers should not need to reconstruct:

- whether a lane is safe
- whether the full closure is safe
- whether proof is required or missing
- whether refusal occurred
- whether contract drift should block completion

V11.4 should make that direct.

### 3. Completion/merge systems need Ota semantics without reimplementing Ota

If CI, bots, and future org-level systems must each encode Ota’s safety and completion rules
themselves, drift becomes inevitable.

V11.4 should define one machine-facing governance contract they can consume.

### 4. Governance state is phase-sensitive

Before execution, Ota can honestly say:

- this lane is safe or refused
- this proof will be required
- this closure is blocked or allowed

After execution, Ota can honestly say:

- required proof was produced
- required proof is missing
- refusal occurred instead of execution

Those are not the same phase.

V11.4 must define them under one canonical model with explicit phase/state transitions instead of
leaving consumers to infer whether an absent proof is a preflight issue or a post-execution
failure.

## Proposed implementation slices

### 1. Governance JSON surface

Add a dedicated governance block and/or dedicated governance command surface with fields such as:

- selected task or workflow
- requested mode and effective mode
- explicit phase sections or one equivalent stable phase-labeled field model

Minimum semantic shape:

- `preflight_governance`
  - effective safe set result
  - closure safety verdict
  - refusal eligibility / refusal reason
  - required proof / receipt expectations
  - drift or blocker classes relevant before execution
- `post_execution_evidence`
  - execution attempted / not attempted
  - refusal occurred / did not occur
  - proof present / missing
  - receipt present / missing
  - post-execution blocker classes relevant to completion

The exact command shape can remain narrow, but the output must be stable and explicit.

### 2. Task and workflow parity

Do not make governance JSON task-only.

The surface should answer the same questions for:

- `ota run <task>`
- `ota up --workflow <name>`

### 3. Outcome taxonomy

Define the canonical machine-readable governance states, for example:

- preflight:
  - allowed
  - refused
  - blocked
  - warning_only
- post-execution:
  - not_run
  - executed
  - refused
  - refused_as_expected
  - evidence_missing
  - evidence_satisfied

And explicit reason families such as:

- unsafe_requested_task
- unsafe_dependency_closure
- unsafe_workflow_closure
- proof_required
- missing_required_proof
- contract_drift
- policy_violation

### 4. Evidence linkage

The governance output should point at the right evidence surfaces, not replace them.

That means:

- receipts remain execution evidence
- proof remains runtime evidence
- doctor findings remain diagnosis evidence
- governance output points at the relevant receipt/proof/doctor artifacts and statuses
- the governance model stays canonical; evidence links enrich it instead of creating a separate
  undocumented phase model

For an enforcement canary, `refused_as_expected` is a dedicated runner-authored outcome. It must
record the selected target, derived closure, derived refusal reason, and a zero-start execution
assertion. It is neither ordinary execution success nor an unstructured refusal failure.

### 5. Additive command discipline

Prefer additive surfaces.

Do not break:

- existing receipt semantics
- existing run/up dry-run JSON
- existing doctor JSON

Instead add a canonical governance evaluation surface those commands can align with over time.

## Expected command direction

The final command spelling can stay narrow, but the product should support a shape like:

- `ota run <task> --agent --json`
- `ota up --workflow <name> --agent --json`
- a dedicated governance evaluation command if needed later

The key is:

- external systems can ask Ota for a preflight governance verdict without executing side effects
- the same canonical model can later describe post-execution evidence status after execution occurs

## Acceptance bar

V11.4 is complete when:

- one machine-readable governance verdict exists for task and workflow selection
- preflight and post-execution semantics are explicit and non-ambiguous
- closure-aware safe/refused state is explicit
- refusal reason families are explicit and stable
- proof/receipt expectation state is explicit
- external consumers no longer need to scrape human output to understand Ota governance

## Follow-on boundary

V11.4 is the output layer.

The next slices should build on top of it:

- [V11.5](../v11.5/plan.md): CI and merge-gate projection
- [V11.6](../v11.6/plan.md): harness and sandbox capability integration
- [V11.9](../v11.9/plan.md): governance truth reconciliation and evidence classes for keeping
  shipped governance fields aligned with the exact decision path that emitted them
