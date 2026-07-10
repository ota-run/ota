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

# V11.10 Plan

Status: active implementation slice. The first receipt-authoring cut captures declared
lockfile-strict pnpm and npm dependency identity; broader replay posture remains planned.

Release target:

- follow-on slice after `v11.9`

Source direction:

- [V10 plan](../v10/plan.md)
- [V11 plan](../v11/plan.md)
- [V11.9 plan](../v11.9/plan.md)
- [Execution receipt](../../spec/execution-receipt.md)
- [Semantic snapshots and correlation](../../spec/semantic-snapshots-and-correlation.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.10 theme:

- replay-verified baseline trust and last-known-good posture

This slice takes the semantic snapshot and receipt foundation that already exists and tightens what
Ota means by a trustworthy baseline.

The product goal is not:

- another archive format
- treating a single past green receipt as durable truth
- collapsing receipts, snapshots, and replay checks into one blob
- inventing a second semantic-diff model beside V10

The product goal is:

- distinguish a past witness from a replay-verified baseline
- make the "last known good" idea honest and machine-readable
- name more of the verdict-relevant input set instead of letting it hide behind one green run on
  one machine on one day
- classify replay artifacts by whether a clean reading can actually acquit the input class they
  represent
- make hermetic replay versus fresh derivation explicit in the canonical replay model itself

## Canonical product principle

Last known good is not "latest green once."

The mature claim is:

- exact source identity
- exact contract snapshot
- pinned input set
- exact witness
- replay posture now

The derivation is the arithmetic.

That means:

- a receipt is still the witness of one execution
- the archived semantic snapshot is still the contract truth that execution used
- replay posture is the current trust check against that witness
- Ota should not promote a stronger claim than the evidence supports
- if replay reaches unnamed live external state, the result is a fresh derivation, not a hermetic
  replay
- replay should trust artifacts differently depending on whether a clean state is real proof or
  only narrowing evidence

## Problem statement

Today Ota already carries:

- archived receipts
- promoted and latest baselines
- semantic contract snapshots
- assumption-set identity
- receipt-to-baseline comparison
- semantic contract drift correlation

That is strong foundation, but it still leaves one important trust gap.

A green archived receipt can currently mean:

- the repo truly remained reproducible under the same pinned inputs
- or the repo was only green on one machine at one time with ambient hidden inputs that later
  moved

Those are not the same claim.

If Ota does not distinguish them, "baseline" can quietly drift from:

- trustworthy re-derivable truth

to:

- screenshot of a happier time

V11.10 is the slice for making that difference explicit.

## Included capabilities

- a first-class replay posture for baseline trust
- a first-class replay hermeticity model alongside baseline posture
- a clearer machine-readable definition of `last_known_good`
- stronger pinned-input identity around replay-sensitive baseline lanes
- explicit replay artifact trust classes for acquitting, narrowing, and pointer-only evidence
- replay-aware receipt comparison posture
- explicit separation between witness evidence and replay verification
- hidden-input surfacing when replay fails without semantic contract change

## Non-goals

- do not rerun every archived receipt automatically by default
- do not claim that Ota can replay opaque external systems perfectly
- do not redefine V10 semantic snapshots or V11.9 reconciliation
- do not collapse replay posture into one binary `good` / `bad` field without provenance
- do not treat container images or CI success alone as sufficient replay truth

## Core product gaps

### 1. Baseline trust is still too coarse

Today Ota can point to:

- latest archived receipt
- promoted baseline
- source identity
- exact semantic snapshot

But the trust posture of that baseline is still weaker than it should be.

A past green witness is useful, but it is not yet the same thing as:

- pinned inputs
- exact witness
- replay still holds now

V11.10 should make that distinction explicit.

### 2. `last_known_good` is not yet precise enough

What operators really want is not:

- "show me the last green thing"

They want:

- the newest commit and input set whose witness still replays

That means Ota needs a stronger machine-readable baseline concept than "promoted" or "latest"
alone.

### 3. Hidden inputs still hide inside replay drift

When replay fails, the interesting question is not only:

- "did the repo break?"

It is also:

- "which relevant input changed without the contract naming it?"

V11.10 should use replay failure as hidden-input evidence, not treat it as generic noise.

### 4. Replay artifacts are not equally trust-closing

Some replay artifacts can actually end the search when they are clean.

Examples:

- lockfile diff
- base image digest or equivalent pinned runtime image identity

Those are acquitting artifacts because a clean reading clears the whole named input class.

Other artifacts are still useful, but a clean reading only narrows the search.

Examples:

- environment snapshots
- external fixtures or captured world slices

Those are narrowing artifacts because they only prove the named subset held still, not that the
still-unnamed input did not move.

Logs are useful too, but in a different way:

- they localize the layer
- they do not acquit the input class

V11.10 should make that trust difference explicit instead of treating every replay artifact as if
it carried the same closing power.

### 5. Hermeticity is still too implicit in the replay model

The plan already relies on this distinction:

- hermetic replay
- fresh derivation against still-ambient inputs

But that distinction should not live only in prose.

The canonical replay model should make it explicit in machine-readable form so Ota does not rely
on readers inferring hermeticity from surrounding narrative.

## Proposed implementation order

1. define replay posture explicitly
2. define replay hermeticity explicitly beside that posture
3. define `last_known_good` against that combined model
4. strengthen pinned-input identity where Ota already has honest access to it
5. add replay-aware baseline comparison
6. classify replay artifacts by trust-closing power
7. surface hidden-input suspicion from replay failure
8. only then widen operator UX further

## Proposed implementation slices

### 1. Replay posture as first-class baseline truth

Direction:

- baseline trust should publish whether the archived witness is:
  - `witness_only`
  - `replay_verified`
  - `replay_failed`
  - `replay_unavailable`
- baseline trust should also publish hermeticity explicitly, for example:
  - `hermetic`
  - `partly_ambient`
  - `ambient_fresh_derivation`
- this posture should be explicit in machine-readable receipt/baseline comparison output
- Ota should avoid claiming a stronger trust state than the current evidence supports
- replay posture should be scoped canonically by the selected lane and execution boundary, not
  treated as one repo-global truth
- replay artifact trust classes should be derived by Ota from the replay input model and artifact
  semantics, not hand-labeled by operators or callers

Minimum replay scope should include:

- selected workflow or task lane identity
- selected execution mode
- relevant runtime or provider boundary when that boundary changes the replay claim

This keeps the difference explicit between:

- "this once passed"
- "this still re-derives"

First honest replay target:

- start with deterministic local finite-task witnesses only
- require one concrete first lane where Ota can already name the exercised source identity,
  contract snapshot identity, lockfile or equivalent dependency pin, and selected runtime identity
- keep broader workflow, live-network, or non-hermetic external-state replay claims explicitly out
  of scope for the first implementation cut

The point of the first shipped lane is not breadth. It is one honest replay-verified baseline
surface Ota can defend end to end.

First explicit pressure target:

- a lockfile-pinned local finite verification lane on a repo-local runtime with no live external
  dependency requirement
- the strongest nearby repo shape is a lead-quorum-style local Python verification lane once the
  dependency and runtime identities are fully pinned for replay
- the important property is not the language. It is:
  - finite local task
  - lockfile or equivalent dependency identity
  - pinned runtime identity
  - no required live external world in the first honest cut

### 2. Replay artifact trust classes

Direction:

- classify replay-relevant artifacts by whether a clean reading can actually close the case
- keep the first trust classes explicit:
  - `acquitting`
  - `narrowing`
  - `pointer_only`
- use those classes to guide both machine-readable replay output and operator-facing replay order

First canonical artifact record:

- attach trust roles only to artifacts Ota actually captured or resolved on the evaluated path
- publish the first record under the canonical receipt/baseline comparison replay output, rather
  than creating a second replay report or adding labels to prose-only diagnostics
- keep the record runner-derived and additive. The first implemented record is the already
  archived semantic contract snapshot, because it has immutable identity on both sides of a
  receipt comparison:

  ```json
  {
    "id": "semantic_contract_snapshot",
    "kind": "semantic_contract_snapshot",
    "input_classes": ["contract_truth"],
    "trust_role": "acquitting",
    "baseline_identity": "sha256:<baseline>",
    "current_identity": "sha256:<current>",
    "comparison": "matched"
  }
  ```

- later lockfile and runtime-digest records may use the same carrier only after receipts capture
  their immutable evaluated identities; do not fabricate them from current filesystem state

- `input_classes[]` is mandatory because an acquitting artifact clears only the class it names;
  a matching lockfile does not acquit ambient environment or live external-state drift
- `input_classes[]` reuses the canonical V11.9 cited decision-input family taxonomy; replay must
  not introduce a parallel local vocabulary. When a replay artifact needs a class V11.9 does not
  yet own, extend the canonical cited-input taxonomy there first and then consume that identity in
  replay output
- `identity` must be immutable or content-addressed for an `acquitting` role; mutable aliases,
  status labels, and operator notes cannot carry that role
- do not emit a trust role just because documentation mentions an artifact type; omit it until Ota
  has truthful captured identity and semantics for that artifact on the evaluated path

The first honest interpretation should be:

- acquitting artifact:
  - clean means the named input class is genuinely cleared
- narrowing artifact:
  - clean means only the named subset held still
  - the still-unnamed residue may still be the cause
- pointer-only artifact:
  - useful to point at the layer
  - not enough to conclude

This keeps replay from over-trusting artifacts whose clean state is only as wide as the naming
discipline behind them.

The first honest ownership rule should stay explicit too:

- these trust classes are replay-engine-derived
- they come from the canonical replay input model plus artifact semantics
- they are not caller-supplied labels and not contract-authored opinions
- artifact `input_classes[]` are canonical V11.9 decision-input family identities, not
  replay-specific strings or operator-supplied categories
- the first carrier is receipt-to-baseline replay comparison output; later command summaries may
  derive from that same record but must not invent an independent taxonomy

### 3. Honest `last_known_good`

Direction:

- define `last_known_good` as:
  - exact source identity
  - exact contract snapshot identity
  - pinned input set
  - exact green witness
  - replay posture still satisfied now
- keep `source identity` and `contract snapshot identity` separate because they anchor different
  truth:
  - source identity answers which code and repo state were exercised
  - contract snapshot identity answers which declared execution truth was exercised
- keep weaker states explicit instead of collapsing them into the same label

This means a baseline that was green once but no longer replays should not still masquerade as
fully known-good.

### 4. Stronger pinned-input identity

Direction:

- keep source identity separate from contract snapshot identity
- keep contract snapshot identity
- widen input identity where Ota can do so honestly:
  - source identity for the exercised repo state
  - lockfile identity and hashes
  - selected tool/runtime identity
  - selected workflow/task lane identity
  - selected execution mode and relevant runtime/provider boundary
  - relevant env/profile selection identity
  - policy or ruleset identity when that policy or ruleset participates in the evaluated replay
    truth
  - baseline receipt or snapshot identity already used in compare
- treat CI or container witness as helpful evidence, but not a full substitute for named inputs

This is the input side of replay trust.

### 5. Replay-aware comparison

Direction:

- when comparing current state against a baseline, publish whether the baseline is only a witness
  or still replay-verified
- keep that replay posture scoped to the same lane and execution boundary rather than implying one
  repo-global replay verdict
- when replay fails, keep contract drift and replay drift separate
- surface whether the failure looks like:
  - semantic contract drift
  - pinned input drift
  - hidden-input drift
  - external unreplayable dependency

This keeps replay from being misread as just another diff view.

### 6. Hidden-input hardening from replay failure

Direction:

- if replay fails without a meaningful semantic contract change, Ota should surface that as hidden
  input evidence instead of flattening it into generic execution failure
- likely hidden inputs include:
  - time-sensitive external state
  - ambient environment variables
  - unpinned dependency resolution
  - changed policy/ruleset identity where that identity was not already pinned on the evaluated
    path
  - unstated platform/runtime differences

The rule should be:

- if replay proves an input moves the trust claim, Ota should either name it explicitly or
  downgrade the trust claim honestly

The artifact-trust rule should stay explicit too:

- trust acquitting artifacts to close their named class
- trust narrowing artifacts only to narrow
- trust pointer-only artifacts only to point
- when a narrowing artifact looks clean and the replay still fails, treat that as evidence that a
  still-unnamed input class remains in play

The promotion order should stay explicit:

1. name dependencies
2. name runtime
3. name environment
4. snapshot the world or abstain from hermetic replay claims

This is the practical replay-hardening order because:

- unpinned dependencies are usually the highest-frequency ambient input class and close cleanly
  with lockfiles or equivalent dependency identity
- runtime and machine drift close next through pinned interpreter, base image, or equivalent
  runtime identity
- ambient environment drift is often the next residue once dependency and runtime identity are
  pinned
- live external state is the least pin-able class and therefore needs snapshotting or an honest
  downgrade from hermetic replay to fresh derivation

The operator ordering should follow that trust order:

1. check acquitting artifacts first
   - lockfile diff
   - pinned runtime or image digest
2. use narrowing artifacts second
   - env snapshot
   - external fixture snapshot
3. use logs as pointer-only evidence throughout
   - enough to point at the layer
   - never enough to close the case

The hermetic boundary should stay explicit:

- if the evaluated path depends on live external state that was not snapshotted or otherwise named
  as replay input, Ota should not present that path as hermetic replay
- a replay that reaches the live world is a fresh derivation wearing a replay's clothes
- the mature operator choices are:
  - snapshot or vendor the external state
  - replay against a frozen mirror
  - or abstain from calling the result replay-verified

This should stay aligned with V11.9:

- V11.9 promotes ambient inputs into cited inputs on the authoritative decision path
- V11.10 uses that stronger named-input set to decide whether a baseline is hermetic,
  replay-verified, witness-only, or still partly ambient

### 7. Operator UX only after evidence is solid

Direction:

- only after the evidence model is clear should Ota widen the operator surface with dedicated
  replay-check commands or richer summary lanes
- prefer additive widening of existing receipt/baseline surfaces over inventing a parallel replay
  product line

## Acceptance bar

V11.10 is complete when:

- Ota can distinguish a past witness from a replay-verified baseline in machine-readable output
- replay posture is explicitly scoped by selected lane and execution boundary on the first honest
  paths instead of overclaiming repo-global truth
- hermetic replay versus fresh derivation is explicit in the canonical replay model instead of
  living only in narrative explanation
- replay artifact trust classes are explicit enough that acquitting, narrowing, and pointer-only
  evidence are not treated as equivalent trust closers
- replay artifact trust classes are replay-engine-derived from the canonical replay input model and
  artifact semantics instead of hand-labeled by callers
- every emitted artifact trust role names the input class it can clear or narrow, and acquitting
  roles require immutable or content-addressed artifact identity
- replay artifact input classes reuse the V11.9 cited decision-input taxonomy rather than
  introducing a replay-local vocabulary
- receipt-to-baseline replay comparison is the first canonical artifact-trust carrier; later
  operator surfaces derive from that record instead of publishing parallel labels
- `last_known_good` is defined in terms of exact witness plus replay posture, not only "latest
  green"
- `last_known_good` keeps source identity and contract snapshot identity as separate pinned inputs
  instead of blurring them into one selector
- baseline trust output makes weaker states explicit instead of implying stronger reproducibility
- replay-sensitive input identity is stronger than contract hash alone on the first honest paths
- replay-sensitive input identity includes policy or ruleset identity where that policy
  participates in the evaluated trust claim
- replay failure can be surfaced separately from semantic contract drift
- hidden-input suspicion is treated as real evidence instead of generic flake where Ota can make
  that call honestly
- receipt remains the execution witness while replay remains the current trust check against that
  witness
- V11.10 strengthens baseline trust without inventing a second archive or diff model beside V10

## Follow-on boundary

V11.10 still stays inside OSS trust and operability.

What can later build on top of it is:

- stronger provider-backed replay lanes
- richer baseline promotion policy
- enterprise retention, audit, or approval overlays that can rely on explicit replay posture
  instead of treating all green baselines as equal

## Acknowledgment

The initial artifact-trust distinction was sharpened through discovery feedback from
[Vinicius Pereira](https://github.com/vinimabreu): a clean lockfile or image digest can acquit a
named class, while clean environment snapshots and fixtures can only narrow the search. When this
surface ships, acknowledge that contribution in the release changelog and public replay reference.
