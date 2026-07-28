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

# V11 Plan

Status: active version frame.

Release target:

- post-`1.6.22` planning slice
- broad version frame after `1.6.22`

Source direction:

- [Execution receipt](../../spec/execution-receipt.md)
- [Semantic diff and explain](../../spec/semantic-diff-and-explain.md)
- [Doctor finding contract](../../spec/doctor-finding-contract.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [V10 plan](../v10/plan.md)
- [V11.1 plan](../v11.1/plan.md)
- [V11.2 plan](../v11.2/plan.md)
- [V11.3 plan](../v11.3/plan.md)
- [V11.4 plan](../v11.4/plan.md)
- [V11.5 plan](../v11.5/plan.md)
- [V11.6 plan](../v11.6/plan.md)
- [V11.7 plan](../v11.7/plan.md)
- [V11.8 plan](../v11.8/plan.md)
- [V11.9 plan](../v11.9/plan.md)
- [V11.10 plan](../v11.10/plan.md)
- [V11.11 plan](../v11.11/plan.md)
- [V11.12 plan](../v11.12/plan.md)
- [V11.13 plan](../v11.13/plan.md)
- [V11.14 plan](../v11.14/plan.md)
- [V11.15 plan](../v11.15/plan.md)
- [V11.16 plan](../v11.16/plan.md)
- [V11.17 plan](../v11.17/plan.md)
- [V11.18 plan](../v11.18/plan.md)
- [V11.19 plan](../v11.19/plan.md)
- [V11.20 plan](../v11.20/plan.md)
- [V11.21 plan](../v11.21/plan.md)

V11 theme:

- completion truth for agents
- reviewer evidence for agent-authored work
- local, CI, and agent execution convergence

The first concrete V11 implementation slice is:

- [V11.1: execution governance visibility and proof](../v11.1/plan.md)

This slice turns a recurring product signal into an explicit planning surface:

- agents can produce code that looks plausible
- maintainers still reject the PR because the repo never made completion truth explicit enough

The product goal is not "better prompting."
The product goal is making the repo say what a correct, safe, complete change actually means.

## Problem statement

A large class of rejected agent-authored work is not only a model-quality problem.

It is an execution-governance problem:

- the agent prepared the repo incorrectly
- the agent ran the wrong verification lane
- the repo required services, env, or setup order that were never declared clearly
- the agent crossed unsafe boundaries because the repo did not make them explicit
- the maintainer had to reconstruct whether the change was actually complete

Ota already addresses parts of this, but the current product surface still leaves "what counts as
done?" too implicit in many repos.

V11 is the slice for making completion truth and reviewer evidence first-class.

## Version structure

V11 is the broader version frame.

Its first planned implementation slice is V11.1:

- execution governance visibility and proof

Later V11 slices should build on that foundation instead of competing with it.

The next planned V11 slice after that is:

- [V11.2: source convergence and detection governance](../v11.2/plan.md)

That slice is intentionally the repo-truth convergence layer before the next execution-surface
widenings. It should make later work such as container-backed hydration, deterministic
bootstrap/materialization, and richer runtime bootstrap ownership proceed from governed evidence
instead of ad hoc repo pressure alone.

The next planned runner-enforcement slice after that is:

- [V11.3: agent-scoped execution enforcement](../v11.3/plan.md)

That slice closes the remaining gap between safe-task and workflow-closure declaration and actual
runtime control, so agent-safe execution truth becomes enforceable by the runner instead of
staying only a governance and review surface.

The implemented OSS governance slices after that are:

- [V11.4: machine-readable governance evaluation output](../v11.4/plan.md)
- [V11.6: harness and sandbox capability integration](../v11.6/plan.md)

The following governance slice has open acceptance work and must not be represented as complete
until its stated pressure or implementation bars close:

- [V11.7: audited execution boundary crossings](../v11.7/plan.md) - crossing records are shipped;
  reusable grant authority and crossing-time liveness remain open.

The completed trust/product follow-ons are:

- [V11.3: agent-scoped execution enforcement](../v11.3/plan.md) - implementation and real-repo
  refusal-canary pressure complete.
- [V11.5: CI and merge-gate projection](../v11.5/plan.md) - required lanes, drift, and CI-owned
  refusal-canary checks complete through the V11.15 GitHub adapter.
- [V11.9: governance truth reconciliation and evidence classes](../v11.9/plan.md)
- [V11.10: replay-verified baseline trust and last-known-good posture](../v11.10/plan.md)
- [V11.11: machine-readable proof boundaries and not-proved scope](../v11.11/plan.md)
- [V11.12: typed hydration input provenance](../v11.12/plan.md)
- [V11.13: generated artifact lineage](../v11.13/plan.md)
- [V11.8: sandbox policy compilation from the execution contract](../v11.8/plan.md) - completed at
  the capability-profile compilation boundary; provider-enforced application is planned
  separately in V11.21.
- [V11.14: contract-claim assurance](../v11.14/plan.md) - implementation complete; release
  reconciliation remains active.

The completed provider-adapter slice is:

- [V11.15: managed GitHub Actions governance projection](../v11.15/plan.md) - implementation and
  real-repo pressure complete; release reconciliation remains active.

The completed execution-trust slice is:

- [V11.16: fresh-boundary setup proof](../v11.16/plan.md) - implementation and declared
  filesystem-boundary pressure complete; release reconciliation remains active.

The following planned replay-governance slice is:

- [V11.17: trusted replay baseline regeneration](../v11.17/plan.md)

The following planned lifecycle-proof slice is:

- [V11.18: managed lifecycle-sequence proof](../v11.18/plan.md)

The completed typed-hydration slice is:

- [V11.19: typed uv local-project hydration](../v11.19/plan.md) - Dograh proves the nested
  editable-project, full extras, and ordered dependency-group lane; Marimo independently proves
  the root-project test-group shape on Linux and macOS while remaining narrowing because it has no
  lockfile.

The active policy-governance slice is:

- [V11.20: policy-governed replay input identity](../v11.20/plan.md)

The next planned sandbox-enforcement slice is:

- [V11.21: enforced sandbox policy application](../v11.21/plan.md) - planned and inactive until
  V11.20 closes. It consumes V11.8's shipped `runtime_boundary` and capability-profile foundation;
  it does not reopen or replace that contract model.

Those slices make Ota higher in the stack without abandoning the open execution spec:

- V11.4 publishes portable governance truth
- V11.5 makes CI and merge gates enforce contract-owned completion truth
- V11.6 lets external harnesses enforce Ota’s callable boundary without guessing
- V11.7 makes allowed-but-heavier execution explicit, classifiable, and auditable in OSS before
  enterprise approval layers build on top
- V11.8 compiles contract-owned execution boundary truth into real runtime filesystem and egress
  policy for cooperating sandbox targets

The completed trust-refinement sequence established:

- V11.9 tightens the trust model so governance fields are emitted from the same decision line that
  made them, typed by evidence class, decomposed where Ota already knows truthful blocker or gate
  structure, and checked for post-decision reconciliation instead of drifting into second-read
  assembled JSON
- V11.10 then strengthens the receipt/baseline trust story so "last known good" means named
  inputs, exact witness, and replay posture instead of only one historical green outcome
- V11.11 then makes narrow proof honest in machine-readable form so proof artifacts can say what
  they covered and what they explicitly did not prove
- V11.12 then tightens typed dependency hydration trust where source or feed posture materially
  changes replayability and execution confidence
- V11.13 names generated source as a contract-owned producer/consumer artifact instead of relying
  on procedural ordering alone

V11.14 completes the trust-refinement sequence by keeping a maintainer claim, the closure the
runner can enforce, observable evidence supporting or contradicting that claim, and the policy
decision that admits it separate. Its Athena and Lead Quorum pressure paths prove both supported
and unknown assurance outcomes before V11.15 begins.

## Included capabilities

- explicit completion surfaces for agent-authored change validation
- stronger reviewer-facing evidence for what verification actually ran
- clearer separation between code failure, readiness failure, and contract drift
- better convergence between local, CI, and agent execution truth
- stronger machine-readable stop/review signals for unsafe or incomplete agent outcomes

## Non-goals

- do not turn Ota into a generic PR review platform
- do not build hosted human approval workflow as part of this slice
- do not claim that Ota can prove code correctness from receipts alone
- do not collapse execution evidence, semantic diff, and reviewer intent into one structure
- do not make agents autonomous over dangerous tasks just because the verification story improves

## Product framing

Do not frame this as:

- AI agent quality scoring
- agent leaderboard instrumentation
- prompt management

Frame it as:

- completion truth
- reviewer evidence
- execution convergence

The core question is:

- can the repo tell an agent what a correct, safe, complete change looks like?

## Core product gaps

### 1. Completion truth is still too implicit

Today a repo can declare tasks, workflows, readiness, and safe-task boundaries.

What is still weaker than it should be is the explicit answer to:

- what verification lane counts as completion for this class of change
- what evidence must exist before an agent should stop
- what should block "done" even when code changes look locally plausible

V11 should make that surface clearer and more machine-checkable.

### 2. Reviewer evidence is still too reconstructive

Today a maintainer can inspect:

- receipts
- proof artifacts
- doctor output
- semantic diff / snapshot correlation

That is already useful.

What is still weaker than it should be is the direct reviewer answer to:

- what contract/workflow/task the agent believed it was following
- what verification actually ran
- what failed versus what was skipped
- whether the fix changed repo truth, runtime state, or only code

V11 should reduce reviewer reconstruction work further.

### 3. Local, CI, and agent truth still drift too easily

Even when the repo declares useful contract truth, maintainers can still end up with:

- one local path
- one CI path
- one agent path

V11 should keep pushing toward one explicit execution contract instead of three partially aligned
conventions.

### 4. Unsafe or incomplete agent stopping conditions are still too soft

The repo should be able to say more clearly:

- this task is safe
- this effect is external or destructive
- this workflow is verification, not setup
- this repo is not ready, so code-level completion claims should stop here

V11 should strengthen those stop/review semantics.

## Proposed execution slices

### 1. Completion contract surface

Define a clearer contract-owned surface for completion truth.

This should let a repo say:

- which task or workflow is the canonical completion lane
- which verification steps must pass after changes
- when "done" is not satisfied even if the code change itself compiles or tests partially

Design bar:

- completion truth should be repo-owned
- completion truth should be machine-readable
- completion truth should reuse existing task/workflow structure where possible
- completion truth should not require reviewers to infer intent from prose alone

### 2. Reviewer evidence surface

Widen receipts / proof / JSON summaries so reviewers can answer:

- what path the agent selected
- what actually executed
- what was skipped
- what contract snapshot and semantic assumptions were in play
- whether the failure or stop condition was code, setup, readiness, or contract drift

This is not a new generic reporting engine.
It is the next trust layer on top of existing receipt and proof surfaces.

### 3. Execution convergence governance

Push repos harder toward one canonical verification truth across:

- local execution
- CI workflows
- agent execution

Expected direction:

- reuse repo-owned task/workflow truth
- reduce handwritten duplication in CI
- warn more clearly when the repo's public or machine-facing execution stories split

### 4. Stronger stop/review semantics

Improve machine-readable and human-readable signals for:

- incomplete verification
- readiness blockers
- skipped required proof
- unsafe mutation boundaries
- task paths that should not be treated as autonomous completion

This should sharpen the line between:

- change attempted
- change verified
- repo ready
- safe to merge

## Proposed operator questions

V11 should make Ota better at answering these directly:

- What does this repo require before a change can be considered complete?
- What exact verification path did the agent run?
- Did the repo become ready, or did the agent stop in an unready state?
- Did the fix change code, contract truth, or only local runtime state?
- Is the current PR failure a code problem, a readiness problem, or contract drift?
- Is this agent outcome reviewable as a safe completion candidate, or should it have stopped earlier?

## Rollout order

1. Define the completion-truth contract surface.
2. Publish machine-readable reviewer evidence for selected path and outcome.
3. Tighten stop/review semantics around incomplete verification and unready repo state.
4. Add governance for local/CI/agent execution drift.
5. Pressure-test on real repos with agent-facing verification lanes.

This order keeps the repo-owned truth first, the evidence second, and the stricter governance last.

## Acceptance bar

- a repo can declare a canonical completion lane without relying on prose alone
- Ota can publish what verification actually ran in a reviewer-useful way
- maintainers can distinguish code failure from readiness failure from contract drift more directly
- local, CI, and agent execution truth have a clearer contract-bound convergence path
- incomplete or unsafe agent outcomes surface stop/review signals earlier and more honestly
