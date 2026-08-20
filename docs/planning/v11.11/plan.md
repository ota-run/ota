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

Status: first proof carrier and Athena seam-control pressure target complete. A later dedicated
proof-receipt archive must remain execution-authored; ordinary `ota up` receipts do not inherit
runtime proof evidence they did not execute.

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
- build it on top of `v11.10` replay input identity instead of inventing a second proof-input
  vocabulary
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
- first-class seam-exercise evidence where Ota can observe that a declared dependency interaction
  actually occurred
- separate recorded negative-control proof runs for dependency disruption or null-substitution
  lanes when a repo chooses to prove causal dependency necessity
- a generic bounded workflow-proof declaration for deterministic verification lanes that have no
  dependency seam; it reuses the same archive-backed proof carrier and never manufactures seam
  evidence

## Non-goals

- do not invent one universal proof taxonomy for every future lane up front
- do not claim that every repo must declare every excluded thing manually
- do not collapse proof scope into semantic snapshot or governance verdict identity
- do not use this slice to widen execution capability or replay trust directly
- do not infer that a dependency-down or null-substitution lane would fail without executing and
  recording that control run
- do not let a reachability check or a dependency fingerprint masquerade as causal dependency
  necessity

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

### 5. Reachability is weaker than dependency exercise

A green readiness or runtime proof can show that a neighboring dependency was reachable without
showing that the selected lane actually crossed that seam.

These claims are materially different:

- the dependency accepted a connection
- the selected lane performed a declared interaction against it
- the selected lane failed as expected when that dependency was disrupted or substituted

Ota must not collapse those levels into one green dependency claim. In particular, a contract
declaration, reachable socket, or observed fingerprint does not prove that the dependency is
causally required at runtime.

## Proposed implementation order

1. define the first honest proof-scope model
2. define the corresponding relative not-proved model
3. choose one first JSON carrier for the first honest cut
4. attach both to that carrier first
5. add ordinary seam-exercise evidence only where the selected lane already exposes it
6. add separately recorded negative-control proof truth without changing ordinary green semantics
   or overloading replay artifacts from `v11.10`
7. pressure-test on repos that intentionally carry narrow proof and one declared dependency seam
8. only then widen the taxonomy or replicate across other artifacts
9. only after that move to hydration source/feed posture in `v11.12`

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
  - `dependency_output_shaping_not_proved`
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

### 3a. Dependency seam-evidence ladder

Direction:

- publish dependency evidence at the strongest level the selected proof actually recorded:
  - `reachable`: Ota observed availability or transport/readiness only
  - `exercised`: Ota observed a contract-specific interaction across the declared dependency seam
  - `fault_tested`: a separate negative-control proof run disrupted or substituted the dependency
    and recorded the expected selected-lane failure
- keep this ladder closed. Do not add composite levels such as
  `exercised_with_negative_control`; negative-control validation strengthens the evidence attached
  to `exercised` and is the sole promotion path to `fault_tested`
- keep `reachable` and `exercised` as ordinary selected-proof evidence; neither alone claims
  causal dependency necessity
- permit a non-secret dependency fingerprint to strengthen an `exercised` record when it was
  observed at the seam, but never let that fingerprint promote the record to `fault_tested`
- preserve unrun counterfactuals as `not_proved`; Ota must not emit “would fail” language from a
  declaration, heuristic, or ordinary green run

#### Evidence provenance is part of the claim

A seam fingerprint is not a single kind of evidence. The canonical carrier must classify its
origin as runner-derived evidence metadata:

- `caller_side`: the selected lane emitted a request payload, client trace, or command-side log
  consistent with the declared interaction
- `dependency_side`: the dependency emitted an independently observed server log, queue insert,
  database mutation, or other dependency-owned state change
- `round_trip_effect`: the selected lane observed the dependency's externally visible result, such
  as a sink delivery, callback, or stored object

The promotion rule is deliberately strict:

- `interaction_attempted` is an additive observation flag, not a fourth evidence level and not a
  downgraded `exercised` record
- caller-side evidence may publish `interaction_attempted: true`, but cannot establish
  `reachable` or promote a dependency to `exercised`; either level needs its own observed evidence
- every caller-side-only `interaction_attempted` observation must pair with a contract-derived
  `not_proved` entry for dependency exercise on that same dependency seam
- dependency-side evidence may support `exercised` only when it is bound to the selected proof
  transaction or to a dependency-owned causal token that Ota can verify; seam mapping alone is
  insufficient
- round-trip-effect evidence may support `exercised` only when Ota records why an inert stand-in
  could not satisfy the same assertion for this interaction, or when the same transaction also
  carries runner-verified seam attestation that binds the observed effect back to the declared
  dependency; seam mapping alone is not sufficient
- a non-secret fingerprint strengthens its observed provenance; it does not change the evidence
  level by itself
- missing, ambiguous, or caller-side-only evidence must remain an explicit `not_proved` boundary
  for dependency exercise rather than being summarized as an exercised seam

This reuses V11.9 provenance classes for whether a value is asserted, derived, or attested. The
canonical record nests both dimensions under `observation`:

- `observation.origin` is the runner-derived seam-observation origin: `caller_side`,
  `dependency_side`, or `round_trip_effect`
- `observation.evidence_class` is the V11.9 authority class for the observation: for example,
  `derived` or `attested`

Downstream policy and UX consume the level plus `observation.evidence_class` as the authority
boundary. `observation.origin` explains the evidence's location and controls promotion to
`exercised`; it is not a second authority taxonomy.

#### First executable seam-observer transaction

The first implementation should stay deliberately strict and runner-owned:

1. the workflow declares `proof.seam_observations[]` with an observer id, one existing dependency,
   one normal-closure producer task, one finite observer task, and an Ota-owned marker environment
   name
2. Ota issues one opaque marker per proof transaction, carries its binding internally, and injects
   it only into the declared producer task. It never passes that marker to the observer.
3. after readiness succeeds but before Ota tears down the runtime, Ota invokes each observer on the
   same execution mode with prerequisites skipped, giving it only a non-secret transaction id,
   observation id, and runner-owned transient attestation path
4. validation requires the observer to stay outside the normal workflow closure, require the named
   dependency, and have every prerequisite already owned by that normal closure; the declared
   producer must be in that closure and require the same dependency
5. only an observer that writes a valid attestation containing the exact runner-issued marker for
   this transaction and observation records `outcome: observed` and promotes the matching
   dependency to attested `exercised` evidence with
   `observation.origin: round_trip_effect`
6. observer failure, inability to run, or a failed ordinary runtime proof retains the matching
   `dependency_exercise_not_proved` boundary and fails the proof carrier when ordinary proof had
   otherwise passed

The marker is never emitted. Ota consumes and removes the transient attestation after validating
it, retaining only non-secret transaction and attestation digests. This makes a successful but
inert observer fail: it cannot reproduce an opaque marker it never recovered from the dependency.

The first machine-readable shape should stay narrow:

```json
{
  "dependency_evidence": [
    {
      "dependency_id": "service:postgres",
      "level": "exercised",
      "interaction": "migration_connection",
      "observation": {
        "origin": "dependency_side",
        "evidence_class": "attested"
      },
      "fingerprint": "postgres:16"
    },
    {
      "dependency_id": "service:postgres",
      "interaction": "report_write",
      "interaction_attempted": true,
      "observation": {
        "origin": "caller_side",
        "evidence_class": "attested"
      }
    }
  ],
  "not_proved": [
    {
      "kind": "dependency_exercise_not_proved",
      "dependency_id": "service:postgres",
      "interaction": "report_write",
      "reason": "caller_side_only_evidence"
    }
  ]
}
```

`interaction` and `fingerprint` are optional and must be omitted when Ota cannot recover them
from execution evidence without guessing. Fingerprints must be non-secret and safe to publish.

### 3b. Separate negative-control proof records

Direction:

- model a disruption or null-substitution lane as a separate control run, not as an annotation on
  the ordinary green proof
- reuse an already declared task, workflow, or adapter action for the first control lane; do not
  introduce a generic fault-injection executor in this slice
- link the control record to the same selected proof scope, dependency identity, and green
  obligation, while carrying the contract-declared intervention identity beside the runner-observed
  outcome
- treat a plain control non-zero exit as a bounded `nonzero_exit_observed` result, not as evidence
  of expected dependency failure
- classify the observed control failure as runner-derived evidence: `expected_missing_effect`,
  `setup_failure`, `timeout`, `crash`, `transport_failure`, `wrong_assertion`, or
  `unclassified_nonzero`; contract prose and caller labels cannot select that classification
- publish `fault_tested` only after a structured failure attestation binds the control to the same
  green obligation and proves `expected_missing_effect` under the recorded intervention
- publish an unexpected control success or an invalid control setup as evidence that the seam is
  not yet proven, never as a passing fault test
- never derive `fault_tested` from a fingerprint, ordinary execution log, or successful selected
  lane; the negative-control record is the only authority for that level

#### Negative-control validation is evidence, not a fourth verdict

`negative_control` is an additive structured qualifier on the dependency evidence record. It is a
derived projection of the separate negative-control record above, not a second authority surface.
It must not create another top-level dependency level or let a caller-selected label change
promotion.

```json
{
  "level": "fault_tested",
  "negative_control": {
    "status": "validated",
    "same_obligation": true,
    "negative_control_id": "postgres-unavailable",
    "failure_mode": "expected_missing_effect",
    "failure_attestation_digest": "sha256:..."
  }
}
```

Rules:

- `unrun` means no selected control transaction exists; it adds no causal claim
- `invalid` means the control was unable to prove the expected missing-effect failure, including
  an unrelated non-zero exit, `setup_failure`, `timeout`, `crash`, `transport_failure`,
  `wrong_assertion`, `unclassified_nonzero`, missing correlation, or a mismatched obligation; it
  must preserve `not_proved` for causal dependency necessity
- `validated` requires runner-verified correlation to the selected green obligation and a
  non-secret digest of the structured failure attestation from that same control transaction, with
  runner-derived `failure_mode: expected_missing_effect`. Its projection must include
  `negative_control_id`, equal to the canonical top-level record's `id`; consumers reconcile that
  ID, the parent dependency and obligation, and the two failure-attestation digests exactly.
- `invalid` and `unrun` projections must set `same_obligation: false` and omit
  `negative_control_id` and `failure_attestation_digest`
- only `validated` may promote the record from `exercised` to `fault_tested`
- every marker-bound seam obligation must retain a separate
  `dependency_output_shaping_not_proved` entry keyed to the same dependency and seam obligation,
  whether it is `exercised` or `fault_tested`. Absence is reserved for a future explicit
  output-proof carrier, never inferred from stronger seam evidence or a validated control.
  Causal evidence for marker recovery does not prove that the dependency shaped a broader
  application output
- `negative_control_present` alone is never a promotion input; consumers must use `status`, not a
  boolean presence check
- the separate negative-control record remains canonical for intervention, transaction binding, and
  observed failure outcome; the dependency-level qualifier is a consumer-facing projection derived
  from that canonical record and must not drift from it
- an `exercised` seam without a `validated` qualifier must retain the machine-readable
  `dependency_causality_not_proved` boundary; absence of a selected control is not permission for
  consumers to infer causal necessity
- a `fault_tested` seam must retain `dependency_output_shaping_not_proved` with its
  `proof_obligation_id`; a valid control proves causality for that obligation, not broader output
  shaping

The first machine-readable shape should be additive:

```json
{
  "negative_control": {
    "id": "postgres-unavailable",
    "dependency_id": "service:postgres",
    "obligation_id": "postgres-marker",
    "transaction_id": "sha256:...",
    "control_task": "proof:postgres-unavailable",
    "intervention": {
      "kind": "dependency_disruption",
      "id": "postgres-unavailable"
    },
    "expected_failure": "dependency_unavailable",
    "outcome": "expected_obligation_failed",
    "failure_mode": "expected_missing_effect",
    "status": "validated",
    "failure_attestation_digest": "sha256:...",
    "proof_scope_ref": "workflow:app-proof/negative_control:postgres-unavailable"
  }
}
```

This standalone record is the canonical, boundary-attested control result. The matching
`dependency_evidence[].negative_control` record is a derived projection only and carries
`evidence_class: derived` explicitly. A caller-provided
reason or contract declaration can request the control lane but cannot substitute for its recorded
outcome, runner-derived failure classification, obligation binding, or failure attestation. A
control with no structured attestation remains `invalid` with
`failure_mode: unclassified_nonzero`; it cannot promote to `fault_tested`.

### 3c. Qualified top-level proof verdict

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
- keep seam-exercise evidence and separately recorded negative-control evidence as additive fields
  on that same canonical carrier instead of creating a second ordinary proof receipt shape
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
- ordinary proof output distinguishes `reachable` from `exercised` where Ota has seam evidence
  and leaves unobserved interactions as `not_proved`
- caller-side-only evidence records `interaction_attempted` but cannot publish `exercised`; a
  fixture proves it always pairs with a contract-derived `dependency_exercise_not_proved` entry
  and that transaction-bound dependency-side or inert-double-resistant round-trip evidence is
  required for ordinary seam exercise
- each seam observation nests runner-derived `observation.origin` with the V11.9
  `observation.evidence_class`; the latter is authoritative for downstream policy/UX while the
  former controls evidence-level promotion and is never caller-selected prose
- `fault_tested` appears only on a separately recorded negative-control run with a bound failure
  attestation for the same green obligation; ordinary green proof, fingerprint, caller-side trace,
  or a generic non-zero control exit never implies it
- a negative-control qualifier is consumer-visible under the same dependency record, has only
  `unrun`, `invalid`, or runner-verified `validated` status, and never creates a composite evidence
  level; it is derived from the canonical standalone control record rather than a second source of
  truth
- an exercised seam without a validated control carries `dependency_causality_not_proved`, so
  causal necessity cannot disappear merely because no control was selected
- every marker-bound seam retains `dependency_output_shaping_not_proved` keyed to its declared
  obligation, so seam exercise or causality cannot be over-read as broader application-output
  causality; absence is reserved for a future explicit output-proof carrier
- marker-bound seam observers run before teardown, prove their producer and observer closures are
  owned by the selected workflow, and only a runner-verified transaction attestation removes the
  matching `dependency_exercise_not_proved` boundary
- engineering notes no longer need to carry the only truthful statement of proof scope
- downstream consumers can distinguish a green narrow proof from a broader runtime or repo proof
  without relying on narrative prose

## Pressure-test target

The first real bar should be a repo where Ota intentionally proves a narrow lane and must not
overclaim broader success.

Strong examples include:

- a .NET restore / build slice that intentionally stops short of full runtime proof
- a repo with deterministic verification but intentionally unproved live external paths
- a service repo with one declared dependency seam that can first prove ordinary exercise, then
  run an isolated dependency-down control without widening the ordinary proof claim

The point is not breadth. The point is proving that Ota can carry bounded honesty as contract and
artifact truth.
