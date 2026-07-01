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

# V11.3 Plan

Status: planned.

Release target:

- planned slice after `v11.2`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.1 plan](../v11.1/plan.md)
- [V11.2 plan](../v11.2/plan.md)
- [Doctor finding contract](../../spec/doctor-finding-contract.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.3 theme:

- agent-scoped execution enforcement

This slice closes the gap between safe-task declaration and runtime control.

Today Ota already enforces that execution goes through declared contract tasks and workflows.

What it does not yet enforce is:

- agent execution may run only the safe subset

That means `safe_for_agent` and `agent.safe_tasks` are already useful governance truth, but not yet
the final execution boundary.

V11.3 is the slice for making that boundary real in the runner.

## Canonical product principle

Declaration is not enough.
Agent-safe execution must be enforced by the runner, not only described by the contract.

That means:

- `ota.yaml` declares the safe surface
- the runner refuses agent-scoped execution outside that surface
- receipts stay harness-authored evidence of what actually ran

What this does not mean:

- turning all human `ota run` usage into deny-by-default immediately
- turning all human `ota up` usage into deny-by-default immediately
- asking external agent wrappers to remember the safe list on Ota’s behalf
- treating `ota tasks --safe` visibility as equivalent to runtime enforcement

## Problem statement

Ota is already stronger than prose-only guidance because it is contract-bound:

- `ota run` executes declared tasks
- `ota up` executes declared workflows and preparation
- validator, doctor, and policy can reason over declared effects and boundaries structurally

But one important trust gap still remains:

- an agent-scoped caller can still request a declared task that is outside the safe set
- an agent-scoped caller can still execute workflow/task closure through `ota up` without the same
  safe boundary

That means the current model is:

- contract-bounded execution
- safe-task governance
- incomplete runtime enforcement

The missing step is runner-owned deny-by-default behavior for agent-scoped execution.

V11.3 is the slice for adding that runtime boundary without weakening the existing human operator
path.

## Included capabilities

- an explicit agent-enforced execution mode for `ota run`
- an explicit agent-enforced execution mode for `ota up`
- deny-before-execution behavior when the requested task is outside the effective safe set
- closure enforcement so safe top-level tasks cannot reach unsafe dependency paths
- closure enforcement so workflow-selected setup/run/verify paths cannot reach unsafe task closure
- structured machine-readable refusal output
- explicit receipt/outcome semantics for refused execution
- policy alignment so agent safety can become runtime truth, not only review-time governance

## Non-goals

- do not make undeclared tasks runnable through an agent mode
- do not overload `safe_for_agent` into a general human permission model
- do not silently change default human `ota run` semantics in this slice
- do not push enforcement responsibility out to repo-local wrappers or agent prompt text
- do not treat doctor warnings as a substitute for runtime refusal

## Core product gap

### 1. Safe-task truth is not yet an enforced execution boundary

Today `safe_for_agent` and `agent.safe_tasks` power:

- `ota tasks --safe`
- detect/init starter guidance
- doctor and validator governance
- policy review around effects, writable paths, protected paths, network, and external state

What they do not yet do is:

- cause the runner to refuse an unsafe declared task in an agent-scoped run

That is the exact gap V11.3 should close.

### 2. Safe-task closure is not yet a runtime truth

It is not enough to allow only safe top-level tasks.

If a safe task reaches an unsafe dependency closure, Ota should refuse the whole path in enforced
agent mode.

The maturity bar is:

- effective safe execution means the reachable execution graph is safe

### 3. Workflow execution is still a hole if only direct task runs are enforced

If `ota run --agent` is enforced but `ota up` can still execute workflow-selected task closure
without the same boundary, the product story stays split-brain.

The maturity bar is:

- agent-scoped execution means both direct task execution and workflow-driven execution respect the
  same safe closure model

### 4. Refusal evidence must be explicit, not implied

This slice is trust-sensitive.

It is not enough to say that summaries and receipts should distinguish refusal from attempted
execution.

V11.3 must define that refusal is:

- a first-class execution outcome
- emitted by the harness/runner
- machine-readable as a dedicated refusal kind

The maturity bar is:

- no silent drop
- no ambiguous generic failure
- no agent-authored narration standing in for refusal evidence

### 5. Policy is stronger as runtime truth than review-only truth

`require_safe_tasks` is already meaningful governance, but it is still weaker than it should be if
it only affects authoring/review and not execution behavior.

V11.3 should make policy and runner behavior converge better.

## Proposed implementation slices

### 1. Explicit agent-enforced run mode

Add a first-class execution mode for agent-scoped task execution.

Shape direction:

- `ota run <task> --agent`

The important part is not the exact flag spelling.
The important part is:

- the mode must be explicit
- the mode must be machine-selectable
- the mode must be enforceable in the runner before task execution begins

### 2. Explicit agent-enforced workflow mode

Add the same first-class enforcement posture to workflow execution.

Shape direction:

- `ota up --agent`

The important part is not the exact flag spelling.
The important part is:

- workflow-driven execution must not remain a hole
- setup/run/verify task closure selected by `ota up` must use the same safe enforcement model
- the mode must be machine-selectable and enforceable before execution starts

### 3. Effective safe-set resolution

Define one canonical effective safe set from:

- `tasks.<name>.safe_for_agent: true`
- `agent.safe_tasks`

The runner should evaluate the requested task against that effective safe set before execution.

No new contract shape is required first.
V11.3 should reuse the existing task-safety truth before inventing a new surface.

### 4. Dependency-closure enforcement

If the requested task is safe but any reachable dependency path is not safe, deny execution in
agent-enforced mode.

This should apply to:

- direct `depends_on`
- aggregate child-task membership
- execution-plan selected task closures where applicable
- workflow-selected setup/run/verify task closures

### 5. Structured refusal outcome and receipt semantics

When Ota refuses execution in agent-enforced mode, refusal should not be an absence of evidence.

The mature shape is:

- a normal harness-authored execution artifact with refusal outcome
- no task/workflow side effects started
- machine-readable refusal kind and reason

Direction:

- refusal emits a normal execution/receipt surface with `status: refused`
- refusal kind is explicit, for example `agent_execution_refused`
- refusal reason is explicit, for example:
  - `requested_task_not_safe`
  - `unsafe_dependency_closure`
  - `unsafe_workflow_closure`

Required fields should include:

- requested task or requested workflow
- blocked task
- dependency/workflow path
- next step

This keeps trust clean:

- the harness records that execution was requested
- the harness records that policy/boundary stopped it
- the harness does not pretend execution happened
- the agent does not author its own evidence

### 6. Policy/runtime alignment

V11.3 should also make policy truth more operationally honest.

Direction:

- `require_safe_tasks` should align with runner behavior in agent-enforced execution
- policy should be able to require agent-enforced mode in governed contexts later
- policy should be able to validate unsafe workflow closure, not only direct task requests

This slice does not need to solve every future policy mode.
It should establish the runner-owned enforcement surface first.

## Command-surface expectations

### `ota run`

V11.3 should widen `ota run` with an explicit agent-scoped enforcement lane.

In that lane:

- non-safe requested tasks are refused
- unsafe reachable dependency closures are refused
- refusal happens before execution starts

Outside that lane:

- current human operator behavior remains unchanged in this slice

### `ota up`

V11.3 should widen `ota up` with the same explicit agent-scoped enforcement lane.

In that lane:

- workflow-selected task closure is resolved before execution starts
- unsafe workflow closure is refused
- refusal happens before setup/run side effects begin

Outside that lane:

- current human operator behavior remains unchanged in this slice

### `ota tasks`

`ota tasks --safe --use` remains the discoverability surface.

V11.3 should not pretend that discoverability equals enforcement.
It should make the relationship explicit:

- `ota tasks --safe` shows the intended safe surface
- agent-enforced `ota run` enforces it for direct task execution
- agent-enforced `ota up` enforces it for workflow-driven execution

### `ota doctor`

Doctor should remain the review/governance surface:

- it explains why tasks are or are not safe
- it explains boundary and effect problems

But V11.3 should not try to use doctor as the enforcement mechanism itself.

## Pressure-test criteria

V11.3 is not done when one refusal path works in a fixture.

It is done when real repos prove:

- safe tasks run normally in agent-enforced mode
- unsafe declared tasks are refused clearly
- safe tasks with unsafe dependency closure are refused clearly
- unsafe workflow closures are refused clearly
- receipts and summaries distinguish refusal from execution failure explicitly
- human `ota run` behavior is unchanged outside agent-enforced mode
- human `ota up` behavior is unchanged outside agent-enforced mode

Pressure repos should include:

- a repo with explicit `agent.safe_tasks`
- a repo using `safe_for_agent: true`
- a repo where a nominally safe task reaches an unsafe dependency path
- a repo where `ota up` selects workflow task closure that reaches an unsafe path

## Acceptance bar

V11.3 is complete when all of the following are true:

- safe-task truth is enforceable at runtime for agent-scoped execution
- both `ota run` and `ota up` use the same agent-scoped enforcement model
- enforcement happens before execution side effects begin
- dependency closure safety is enforced, not only top-level task safety
- refusal output/receipt semantics are explicit, structured, and machine-usable
- policy and runner semantics no longer drift on what safe-task truth means
- real pressure repos prove the surface honestly

## What comes after

Once V11.3 exists, Ota’s agent story becomes materially stronger:

- contract declares the safe surface
- runner enforces the safe surface
- receipts remain harness-authored evidence

That is the point where Ota moves from safe-task governance toward real agent execution control,
without confusing the human operator path with the agent-enforced path.
