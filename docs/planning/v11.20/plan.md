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
   You may not use this file except in compliance with the License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# V11.20: Policy-Governed Replay Input Identity

Status: active. This follows V11.19. It is a narrow policy refinement of the shipped optional
`tasks.<name>.replay_inputs[].expected_identity` surface, not another replay model.

## Problem

Ota already lets a task declare an immutable SHA-256 identity for a replay input. Doctor, dry-run,
`ota run`, and `ota up` evaluate it before task startup; a missing or mismatched input blocks the
selected closure. That must remain optional by default because many ordinary verification lanes
are not strict replay lanes.

The remaining governance gap is selective admission. An organization may need a named offline,
baseline, or replay-sensitive lane to refuse unless every replay input it declares is both pinned
and currently matching. Repositories must not need shell/JQ policy glue to enforce that rule, and
policy must not make an unpinned lane look immutable merely because it ran successfully.

## Product Principle

The contract declares replay inputs and their optional immutable identities. Policy decides whether
the selected task or workflow requires those identities. The runner evaluates the same preflight
identity records that Doctor and dry-run show; it does not derive a pin from the working tree,
update a digest, or infer inputs the contract omitted.

An enforced pin proves only that each declared replay input held its named content at preflight.
It does not prove that the declaration is complete, that all ambient state is frozen, or that the
lane is hermetic.

## Scope

V11.20 adds one shared replay-input identity-policy evaluator.

Policy selects explicit task or workflow subjects and declares a requirement such as:

```yaml
policies:
  replay_inputs:
    identity:
      tasks:
        report:
          on_insufficient: deny
      workflows:
        offline_replay:
          on_insufficient: review
```

Rule presence means replay-input identity coverage is required for that subject. There is no
`required: false` override: removing the rule restores compatible optional-pin behavior. The final
schema names may follow existing policy-pack conventions, but the ownership is fixed:

- contract-owned task/workflow names select the closure;
- policy-owned rules select whether that closure requires replay-input pins;
- every reachable declared replay input, including inputs on recursive `after_success`,
  `after_failure`, and `after_always` hook edges, is task-qualified and its observed identity is
  computed exactly once per evaluation;
- an active policy source that cannot be loaded is a typed admission refusal, never equivalent to
  no policy;
- an unavailable preflight observation or an unknown observation status is an unconditional denial;
  absence of evidence must never satisfy complete policy coverage;
- each applicable rule needs at least one declared replay input in its own selected closure and a
  canonical `expected_identity` on every such input;
- a declared `expected_identity` is an unconditional preflight obligation: an unreadable input,
  missing artifact, or mismatched observed identity always refuses before execution and policy
  cannot downgrade it to `review`;
- `on_insufficient` governs only policy coverage: no declared replay inputs in a selected closure,
  or a declared replay input that lacks an `expected_identity`;
- unselected lanes preserve today's compatible optional-pin behavior.

Task and workflow rules are cumulative rather than competing:

- for a task invocation, every task rule whose subject is reachable in the selected task execution
  closure applies, including every potentially executable outcome-hook edge;
- for a workflow invocation, the exact workflow rule applies to the complete selected workflow
  phase/dependency closure, and every task rule whose subject is reachable in that closure also
  applies;
- each task rule evaluates only its subject task's own execution closure, including its recursive
  hooks, never unrelated inputs elsewhere in the parent task or workflow;
- the evaluator computes each task-qualified observed input identity once across the union of
  applicable rule closures, then lets each rule reference those canonical records for its own
  coverage decision.

This prevents a parent invocation from bypassing policy on a governed dependency and prevents an
unrelated pinned input from satisfying a narrower task rule. The policy evaluator resolves
selectors against the loaded contract. Unknown task or workflow selectors are typed contextual
policy findings, not `ota validate` contract errors: an active policy with an unresolved selector
blocks its governed execution and CI projection, while the contract remains deterministically
valid on its own. The evaluator retains every applicable rule identity and derives the effective
policy action deterministically: `deny` outranks `review`, which outranks compatible `allow`.
Unselected subjects have no policy coverage requirement.

The evaluator returns one machine-readable decision with the selected subject, closure inputs,
policy identities, coverage, and typed insufficiency reasons. Every governed command loads one
complete policy snapshot before admission and passes it through agent safety, claim assurance,
replay policy, Doctor, provisioning, receipts, and proof. CI projection does the same across all
three governance domains. Runtime proof gives its detached `ota up` child a private serialization
of that admitted snapshot, including an explicit no-policy posture, instead of allowing the child
to rediscover authority. A `review` decision refuses every execution surface in V11.20,
including attended native human execution, agent execution, and CI. V11.20 introduces neither a
bypass flag nor a grant-authority mechanism; a future authorization slice may define one
explicitly rather than treating a recorded crossing reason as approval.

CI projection carries only the canonical policy requirement, execution closure, and policy
identity. The closure uses the evaluator's canonical dependency, aggregate, and recursive outcome
hook expansion. It never embeds a developer checkout's observed file digest or decision. The
generated provider lane re-runs the same evaluator after checkout, where the declared inputs
actually exist.

## Truth Boundaries

- `expected_identity` remains an author-declared immutable value. Ota never automatically writes,
  promotes, or updates it.
- An observed matching digest is runner-derived preflight evidence, not a new contract value.
- Policy evaluates declared replay inputs only. An omitted fixture, service, environment value, or
  other hidden input remains outside this claim and must not be inferred as pinned.
- A policy denial/review retains the selected closure, each task-qualified input identity, and the
  exact missing or mismatch reason in JSON. It must be inspectable even when execution is refused.
- Policy coverage is not an escape hatch from an already-declared pin: an identity mismatch or
  unreadable pinned artifact is a hard refusal in every mode and provider.
- A matching strict identity decision does not upgrade replay to hermetic or acquitting beyond the
  classes the existing replay model permits.

## Non-Goals

- Do not make `expected_identity` mandatory for ordinary tasks.
- Do not introduce a hand-authored `strict_replay: true` shortcut or a second replay taxonomy.
- Do not choose the newest receipt, update a digest, or promote an observed input automatically.
- Do not infer undeclared inputs from logs, command text, or a green prior receipt.
- Do not make policy selection provider-specific or replace CI projection with workflow shell
  assertions.

## Implementation Order

1. Define the generic policy-pack shape and validate duplicate selectors. Resolve selected
   subjects contextually during policy review; preserve unknown selections as typed findings rather
   than changing `ota validate` contract semantics.
2. Implement one closure-aware replay-input identity-policy evaluator by reusing the existing
   `expected_identity` preflight result; agent safety, claim assurance, replay policy, Doctor
   findings/JSON, provisioning, receipts, proof runtime, proof lifecycle, and CI projection must
   consume one command-scoped loaded authority and must not reconstruct observations independently.
   A detached proof child must consume a private snapshot of that authority. Aggregate
   monorepo Doctor JSON must retain each member's canonical evaluation. Runtime proof must admit
   its full selected proof closure, including seam observers and the selected negative control,
   before creating proof artifacts and pass that preflight through readiness diagnosis and its
   embedded Doctor artifact. Lifecycle proof must admit its exact prerequisite-plus-assertion
   closure before tasks, services, or assertions can start. CI projection carries the same
   canonical execution closure, including recursive outcome hooks, and must not embed render-host
   observations.
3. Add typed JSON/schema records for allowed, denied, and review-required decisions, including
   task-qualified input identity, every applicable policy rule, coverage, and the derived action.
4. Add regressions for matching pins, missing pins, missing artifacts, unavailable observations,
   mismatched artifacts, policy-load failure,
   parent-task dependency and outcome-hook rule enforcement, per-rule closure isolation, workflow/task rule
   aggregation and precedence, unknown-selector policy findings and governed refusal, unselected
   compatibility, CI checkout re-evaluation, no provisioning or proof-side effects before refusal,
   aggregate member policy JSON, and policy evidence retained on hard-pin refusal.
5. Update the policy-pack reference, command and JSON references, changelog, canonical example,
   skill, and site only where the new public policy surface changes behavior.
6. Pressure-test Bedrock's frozen SQL/store lane under a strict policy, then use one independent
   repository with a different fixture or baseline shape to prove an unpinned lane is refused
   without weakening ordinary optional-pin behavior.

## Acceptance Bar

V11.20 is complete when:

- policy can require pins for a selected task or workflow closure without changing default task
  behavior elsewhere;
- every declared replay input in the union of applicable rule closures has one stable,
  task-qualified observed identity, while each rule derives coverage only from its own subject
  closure;
- invoking a parent task or workflow cannot bypass a replay-input identity rule on a reachable
  dependency or outcome hook, and an unrelated pinned input cannot satisfy that reachable task's
  rule;
- missing pins, unavailable observations, unreadable inputs, and mismatches are typed, inspectable,
  and refuse before native provisioning, dependency hydration, or any task process begins;
  declared-pin failures refuse regardless of policy action and retain active policy evidence;
- an invalid or unavailable active policy source is a typed refusal before any parent, dependency,
  hook, proof, or lifecycle side effect begins;
- `review` is equally inspectable, refuses unattended agent/CI execution, and cannot be mistaken
  for an allowed strict lane; it refuses attended human execution too until an explicit future
  grant-authority model exists;
- Agent safety, claim assurance, replay policy, Doctor findings/JSON, dry-run, run, up,
  provisioning, proof runtime, proof lifecycle, CI projection, and admission-produced receipts
  reuse one command-scoped loaded policy snapshot and observation set.
  Aggregate Doctor
  retains the same canonical record for every member. Runtime proof admits its full selected proof
  closure before parent artifact creation and reuses one preflight across readiness diagnosis and
  its embedded Doctor artifact. Lifecycle proof admits the exact prerequisite-plus-assertion
  closure before any task, service, assertion, or lifecycle transaction starts. CI projection
  consumes the canonical requirement and execution closure, including recursive hooks,
  without embedding render-host observations, while every provider checkout evaluates its own
  observed input identities;
- no path rewrites a pin or treats a green execution as evidence that an omitted input was pinned;
- Bedrock and one independent real repository prove matching and refusal paths with exact Core
  bootstrap provenance; and
- every material unexercised input remains explicitly bounded rather than promoted into a
  hermetic replay claim.
