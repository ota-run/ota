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

Status: active next slice.

Release target:

- active trust-refinement slice after `v11.8`

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
- reconciliation should start from the published governance outcome against its actual decision
  inputs and result, not from a weaker first question like "did this branch execute?"
- where Ota already has truthful blocker or gate structure, it should publish that decomposition
  instead of collapsing the decision into an undecomposed flat verdict
- flat or pure verdicts remain acceptable only on lanes Ota cannot yet decompose honestly without
  inventing false precision
- authoritative governance should distinguish cited decision inputs from ambient reads:
  - cited inputs are the exact inputs the decision used and can stand behind a trust claim
  - ambient reads are world state that happened to be visible during evaluation but were not
    recorded as authoritative decision inputs
  - replay-grade governance should reuse cited inputs instead of re-reading the world

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

There is also a third trust weakness:

3. undecomposed governance verdicts where richer structure already exists
- a path is published only as `allow` or `deny`
- but the engine already knows the blocker set, gate set, or closure conditions that produced the
  verdict
- downstream consumers are then forced to trust a flat claim instead of the decision shape Ota
  already has

V11.9 should tighten that honesty boundary:

- do not force every lane into decomposition before Ota can explain it truthfully
- do require decomposition on lanes where Ota already has stable blocker or gate structure and is
  simply failing to publish it

There is also a fourth trust weakness:

4. decomposed decisions without stable machine citations
- a path publishes blockers or gates
- but those citations are only prose or weak labels
- replay and reconciliation then cannot compare the decision basis semantically without string
  heuristics

V11.9 should tighten that trust boundary too:

- do not treat human-readable blocker or gate text as the canonical basis identity
- do require a stable machine-readable decision-basis citation set on decomposed authoritative
  governance lanes

## Included capabilities

- field-level reconciliation between published governance outcome and the actual decision inputs
  plus result
- outcome decomposition for governance lanes where Ota already has truthful blocker or gate
  structure
- stable machine-readable decision-basis citation for decomposed authoritative governance records
- decision-site governance emission rules
- mandatory evidence-class typing for authoritative machine-readable governance fields
- preflight/post-execution consistency checks for governance records
- explicit separation between caller assertion, runner derivation, and runner/harness attestation
- additive machine-readable state for reconciliation success, failure, or unknown posture
- pinned replay for authoritative governance paths once reconciliation shape exists
- explicit hidden decision-input capture for fields replay depends on
- narrow mechanism-level hook/branch identity checks only where stronger outcome-level
  reconciliation is not available

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
- one localized decision owner should own each authoritative governance record; if a verdict is
  still smeared across modules and then reassembled later, that is an architecture defect this
  slice should close before calling the field trustworthy

### 2. Evidence classes are still too implicit

Today a field like `reason` can be machine-readable without being machine-verified.

That is still progress, but the schema should say so.

V11.9 should make the evidence-class boundary explicit instead of leaving downstream systems to
guess.

The important bar here is mandatory, not advisory:

- every authoritative governance field should carry an explicit evidence class
- downstream consumers should never have to infer whether a field was asserted, derived, or
  attested from field name alone

### 3. Claimed enforcement still needs reconciliation

It is not enough for the record to say:

- this boundary was crossed
- this refusal reason applied
- this merge gate fired

Ota should also be able to say whether:

- the claimed enforcement path actually fired
- the emitted record still matches the decision hook that ran

But that mechanism-level question is not the first trust bar.

The stronger first question is:

- does the published governance outcome reconcile to the actual decision inputs and result?

Only after that should Ota ask the narrower follow-on question:

- can the system also prove that the expected mechanism or hook identity fired?

## Implementation order

V11.9 should be built in this trust order:

1. field-level reconciliation first
2. mandatory evidence classes second
3. localized decision ownership third
4. phase-accurate preflight/post-execution consistency fourth
5. pinned replay for authoritative governance paths fifth
6. hidden decision-input hardening from replay gaps sixth
7. narrow mechanism-level hook/branch identity checks last

This ordering matters.

- outcome checks are stronger than mechanism checks
- outcome decomposition is stronger than flat verdict summary when Ota already has the structure
- provenance classes are stronger than implicit interpretation
- localized decision ownership is a prerequisite for trustworthy reconciliation
- replay is stronger once the record, inputs, and decomposition are already trustworthy
- hidden inputs should be hardened from replay failures instead of guessed upfront
- hook or branch identity checks are useful tripwires, but they should not become the primary
  trust story

## Proposed implementation slices

### 1. Field-level reconciliation first

Start with the published governance outcome itself.

Direction:

- check whether the emitted governance record reconciles to the actual decision inputs and result
- do this at field level for high-value fields such as:
  - refusal reason family
  - allowed / refused posture
  - crossing required / not required
  - crossing classification
  - merge-required / not-required lane posture
  - evidence-required / evidence-missing posture
- publish reconciliation posture as part of the canonical governance record instead of a separate
  auxiliary export
- reconcile both directions:
  - cited blocker or gate basis should be stable and machine-readable, not only prose
  - `deny` should cite the blocker set, blocker codes, refusal basis, or closure condition that
    forced the denial
  - `allow` should cite the satisfied gates, checks, closure conditions, or admission basis that
    made the allowance truthful
- do not accept a flat `allow` or `deny` as the default shape when Ota already has truthful
  blocker or gate structure available
- keep flat verdicts only on lanes where decomposition would currently require invented or weakly
  understood logic

This is the real trust bar.

If a field like `reason`, `refusal`, `crossing`, `required`, or `allowed` can drift from the
actual decision path, then the JSON is dressed-up prose.

### 1a. Canonical decision-basis citation model

Add one explicit machine-readable basis model for decomposed governance paths.

Direction:

- authoritative `allow` / `deny` decomposition should cite a stable decision-basis set
- each cited entry should be semantically comparable across replay and later evaluation
- the basis model should avoid free-text matching as the canonical identity surface

Minimum shape:

- basis id or code
- basis family
- optional scoped owner or lane reference
- optional additive human explanation
- optional cited input reference when the basis depends on a replay-critical external input

For the cited decision-input lane itself, the first shipped additive shape should be:

- `decision_inputs[]`
  - stable input id
  - input family
  - evidence class
  - replay class
  - optional additive detail

Replay class should stay narrow and honest:

- `pinned`
  - reusable immutable or selector-resolved input suitable for authoritative replay
- `witnessed`
  - observed execution/evidence input reused as a witnessed artifact, not re-fetched ambient world
    state

Examples of the kinds of citations this should cover:

- blocker code
- gate id
- refusal basis id
- closure condition id
- admission basis id

The product rule is:

- prose may explain a basis
- prose must not be the only canonical identity for a basis

### 1b. Decomposition honesty rule

Add one explicit product rule for authoritative governance paths:

- if Ota already knows the decision shape, it should publish the decision shape
- if Ota only knows the final verdict honestly, it may publish the final verdict without fake
  decomposition

This avoids two opposite failures:

- under-modeling a lane Ota already understands well
- over-modeling a lane Ota does not yet understand well enough to explain honestly

### 2. Decision-site emission rule

Define a strict product rule for governance records:

- authoritative governance records are emitted from the same decision path that made the verdict
- later serializers may carry or render those records, but they do not infer or recreate the
  authoritative decision content from scratch
- each authoritative record should have one localized decision owner
- if no localized decision owner exists yet for a governance field, this slice should first move
  the field onto one before claiming full trust refinement

Direction:

- refusal records are created where refusal is decided
- crossing records are created where crossing is decided and allowed
- merge-gate / required-lane records are created where the governing decision is classified

### 3. Evidence-class model

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

And it should be mandatory for authoritative governance fields:

- if a field is authoritative enough to drive downstream CI, harness, or policy behavior, it
  should publish an evidence class
- missing provenance class on an authoritative governance field should be treated as incomplete
  trust modeling, not as acceptable omission

### 4. Field-class guidance

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

### 5. Reconciliation checks

Add reconciliation posture to the governance story.

Direction:

- the system should first know whether the published governance outcome reconciles to the actual
  decision inputs and result
- consumers should see whether reconciliation is:
  - `satisfied`
  - `mismatch`
  - `unknown`

This is not a second governance model.
It is a trust check on the canonical one.

Mechanism-level identity checks should stay narrower:

- use hook or branch identity checks only where there is no stronger outcome-level reconciliation
  available
- treat them as supplemental trust tripwires, not the main trust story

### 6. Pinned replay for authoritative governance paths

After reconciliation, evidence classes, and localized decision ownership exist, add pinned replay
for the authoritative governance paths.

Direction:

- be able to re-run the governance evaluation against the same declared inputs and confirm the same
  verdict plus the same decomposition
- pin the replay to the actual decision inputs the verdict depended on, not just the contract file
- require authoritative replay-critical inputs to resolve to immutable identities before the replay
  can be called pinned
- start with the strongest governance paths first:
  - refusal
  - crossing-required / not-required
  - crossing classification
  - merge-required / not-required lane posture

Replay should answer:

- given the same inputs, does Ota produce the same governance verdict?
- given the same inputs, does Ota produce the same blocker or gate decomposition?

First honest replay target:

- selected task/workflow governance evaluation on local preview and `up` lanes where Ota already
  owns:
  - lane identity
  - actor mode
  - closure safety posture
  - receipt attachment presence/status
  - proof expectation/presence
  - crossing-record posture

That is the right first bar because it is already localized, additive, and free of ambient network
state on the canonical local path.

The cited-input rule for replay should stay explicit:

- replay reuses the cited decision inputs the authoritative verdict recorded
- replay does not silently re-read ambient world state and then claim the result is pinned
- if replay must fall back to ambient reads for a path, that path is weaker and must say so
- authoritative replay is about "decision from cited inputs", not rerunning an arbitrary external
  system end to end

The product posture should stay bug-driven:

- when replay exposes one hidden input that moved the verdict, the smallest truthful fix should be
  to promote that input into cited decision-input truth for the affected path
- this should feel like ordinary product hardening, not governance overhead layered on top
- the preferred operator experience is: replay fails, Ota proposes or assists promotion of the
  missing input class through the canonical record for the affected path, and replay can then go
  green again in the same session on that same path
- this slice should not imply silent or unguided auto-promotion of authoritative decision inputs;
  promotion should stay explicit, reviewable, and grounded in the canonical record that owns the
  affected truth

This is a stronger trust move than branch-identity checks because it validates outcome and
decision shape together.

Pinned must also mean:

- immutable or content-addressed receipt identity where receipt input matters
- immutable or content-addressed semantic snapshot identity where snapshot input matters
- immutable policy or ruleset identity when policy or rules move the verdict
- immutable baseline identity when baseline comparison participates in the decision

The following are not authoritative replay pins on their own:

- `latest`
- `promoted`
- drifting branch labels
- mutable policy aliases
- mutable ruleset labels

Those may remain operator selectors or convenience handles, but authoritative replay must resolve
them to immutable identities before using them in a trust claim.

### 7. Hidden decision-input hardening

Pinned replay is also how Ota should discover which hidden inputs still move the verdict.

Direction:

- make decision inputs explicit instead of letting them stay ambient
- start from the inputs replay proves are verdict-relevant

Likely hidden inputs include:

- time
- `evaluated_at` or equivalent effective decision timestamp
- environment variables
- feature flags
- mutable external config
- floating model aliases or other non-pinned external decision inputs
- lookup-table row identity
- deterministic tie-break identity when order or winner selection matters
- policy pack identity or version
- baseline receipt or snapshot identity
- ruleset identity or version
- any other non-contract input that can move the governance result without changing the contract

The product rule should be:

- if a hidden input can move an authoritative governance verdict, it should become an explicit
  decision input for that path
- if Ota cannot yet surface the input explicitly, replay for that path should stay weaker and say
  so honestly

Promotion should stay additive and low-friction:

- cited-input promotion should add new fields without forcing brittle migration on existing
  governance or receipt consumers
- replay should immediately reuse newly promoted cited inputs through the same canonical record
  shape instead of requiring a second orchestration surface
- the product success metric is not only "replay failed honestly"
- it is also "the hidden-input class became smaller after the fix"

Ota should make hidden-input debt visibly shrink over time:

- newly promoted cited inputs should retire that input class from the ambient/hidden bucket on the
  affected path
- governance and replay UX should make that retirement visible instead of only reporting another
  failure

Replay-critical selector rule:

- if a selector can drift while keeping the same human label, it is not itself a pinned replay
  input
- authoritative replay must record the immutable resolved identity behind any convenience selector
  before using that input in a trust claim

Model-mediated rule:

- if a governance path ever depends on model output, Ota should treat the model output as a
  witnessed input artifact, not as a decision function to be replayed
- replay for that path should reuse the witnessed model output as cited input unless Ota can prove
  a stronger deterministic decision boundary

### 8. Mechanism-level hook or branch checks last

Only after outcome reconciliation is in place should V11.9 add narrower mechanism checks.

Direction:

- link the emitted governance record back to the decision hook, branch, or equivalent mechanism
  identity where that identity is stable and useful
- keep this narrowest on pure allow/deny paths where the mechanism identity contributes real extra
  confidence
- do not let mechanism checks outrank stronger outcome reconciliation in either implementation
  order or product explanation

### 9. Preflight and post-execution consistency

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

### 10. Crossing-specific tightening

Apply the same trust rule explicitly to V11.7 crossing records.

Direction:

- crossing record remains the authoritative anchor
- caller `--reason` remains additive narrative context
- reason should carry an evidence class so consumers know it was asserted, not verified
- reason presence, attachment timing, and record emission should still be attested by the runner
- no later receipt formatter should invent or re-classify the crossing independently

### 11. Merge/CI-oriented tightening

Apply the same trust rule to merge-facing governance surfaces from V11.5.

Direction:

- the required-lane or merge-gate record should point back to the same governing decision path
- the emitted merge-facing governance block should be able to state whether it reconciles with the
  actual check-selection logic that fired
- no provider-specific second truth should outrank the canonical Ota record without being called
  out explicitly

## Acceptance bar

V11.9 is complete when:

- published governance outcome fields reconcile to the actual decision inputs and result before
  Ota relies on narrower hook/branch identity checks
- governance lanes that already have stable blocker or gate structure publish that decomposition
  instead of hiding behind undecomposed flat verdicts
- decomposed governance records cite a stable machine-readable decision-basis set instead of only
  prose blocker or gate text
- flat verdicts remain only on lanes Ota cannot yet decompose honestly
- Ota has an explicit rule that authoritative governance records are emitted from the same
  decision path that made the verdict
- each authoritative governance record has one localized decision owner instead of later
  cross-module reconstruction
- the canonical governance model can distinguish at least `asserted`, `derived`, and `attested`
  field posture
- authoritative governance fields require an explicit evidence class instead of leaving that
  posture implicit
- caller-supplied narrative fields such as `--reason` are explicitly typed as non-authoritative
  assertion context
- refusal, crossing, and merge-facing governance records can publish reconciliation posture against
  actual decision inputs and result, with narrower hook/branch checks only as additive tripwires
- authoritative governance replay can confirm the same verdict and the same published blocker/gate
  decomposition from pinned decision inputs on at least the first high-value governance paths
- authoritative replay never relies on mutable selectors such as `latest`, `promoted`, or mutable
  policy/ruleset labels as the final replay identity
- authoritative replay distinguishes cited inputs from ambient reads and downgrades any path that
  still depends on ambient world state
- verdict-relevant hidden inputs discovered through replay are either explicit in the decision
  input model or called out as a remaining trust boundary
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
