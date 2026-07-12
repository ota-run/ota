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
- keep registry-backed container image acquisition as `effects.network_kind:
  container_image_hydration`, distinct from package dependency hydration and from immutable image
  receipt evidence

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

### 1a. Task-scoped declared replay inputs

The first honest static-input widening should be task-scoped rather than reusing `artifacts`:

- `artifacts` remains producer-owned generated-output lineage; a committed fixture, baseline, or
  frozen data store is an execution input, not generated output
- add `tasks.<name>.replay_inputs[]` for immutable repo-relative static files consumed by that
  task; declarations may live on any task in the selected dependency closure
- the first kind is `static_file`, with a stable `id` and one repo-relative `path`
- Ota resolves the selected task or workflow closure, aggregates every reachable declaration with
  a stable task-qualified identity, and captures content identities before the closure begins;
  receipt rendering must not re-read paths later
- a declared replay input is runner-derived `narrowing` evidence for the canonical
  `declared_replay_input` family: matching content proves that named file held still, not that
  every ambient input did
- validation rejects duplicate ids or paths, paths that escape the repo, and declared replay
  inputs that overlap writes in the selected closure; duplicate ids across closure tasks remain
  distinct only through their task-qualified receipt identities

The first pressure shape is a Bedrock-style finite offline verification lane:

```yaml
tasks:
  gate:
    replay_inputs:
      - id: recorded_sql
        kind: static_file
        path: data/fixture.jsonl
      - id: frozen_store
        kind: static_file
        path: data/store.db
      - id: defended_baseline
        kind: static_file
        path: data/baseline.json
```

This does not make the lane hermetic by itself. It gives the receipt an exact, decision-time
record of the frozen inputs it did consume. Lockfile identity, runtime identity, source identity,
and any live-world boundary remain separate inputs with their own trust semantics.

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

The first typed replay input taxonomy is runner-owned and closed:

- `contract_truth`
- `declared_dependency_resolution`
- `selected_runtime_version`
- `selected_runtime_artifact`
- `declared_replay_input`

`selected_runtime_version` is intentionally `narrowing` when it comes from a command version
probe. `selected_runtime_artifact` is `acquitting` only when Ota captures an immutable executable
or image digest from the selected declared runtime path. The first shipped artifact lane is a
literal digest-pinned Compose service image in an explicitly declared file, including the selected
service's declared `depends_on` closure; mutable tags, interpolation, inferred Compose files, and
unrelated stack services stay outside the claim.

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
  - deterministic presentation/runtime identity where it can change an otherwise fixed result:
    ordering, NULL ordering, timezone, locale, numeric formatting, or equivalent adapter behavior
  - relevant env/profile selection identity
  - policy or ruleset identity when that policy or ruleset participates in the evaluated replay
    truth
  - baseline receipt or snapshot identity already used in compare
- treat CI or container witness as helpful evidence, but not a full substitute for named inputs

This is the input side of replay trust.

### 4a. Presentation profile ownership

`execution_presentation_profile` must not blur declared contract truth with runner observation.
The first honest shape is deliberately asymmetric:

- a task may declare one content-addressed replay presentation profile file alongside its static
  replay inputs; this is contract-owned selected-lane truth and is captured before execution
- Ota may later capture an observed runtime profile, but it remains a distinct runner-derived
  record with its own source and completeness posture; it must never silently override or stand
  in for the declared profile
- the first implementation does not infer ordering, NULL placement, timezone, locale, or numeric
  formatting from arbitrary process state
- when no declared profile exists for semantics that can affect the claimed output, Ota publishes
  the relevant replay boundary as ambient or narrowing rather than treating the frozen data as
  hermetic

The first production target is a declared profile file because it has clear contract ownership and
immutable capture semantics. Runner-observed profiles are a later widening only after Ota can name
their completeness truthfully.

### 4b. Comparator semantics are not presentation identity

Some deterministic replay lanes define an equivalence relation that intentionally absorbs selected
presentation differences. That is a distinct control from pinning the runtime presentation:

- a declared comparator may normalize labels, numeric tolerance, ordering, or another bounded
  output dimension before deciding whether two results are equivalent
- the comparator defines the semantic claim of the proof; it does not prove raw output bytes or
  the full runtime presentation were identical
- a comparator identity should be captured as a declared replay input when it participates in the
  selected lane, but it remains `narrowing` for runtime presentation unless Ota also has a complete
  declared presentation profile
- a repo must not use a comparator to silently turn an ambient runtime claim into a hermetic one

Bedrock is the first pressure example: its equivalence implementation deliberately ignores labels
and applies numeric tolerance. That makes its stated result-set claim cheaper and honest, but does
not itself pin timezone, NULL ordering, or every runtime formatting semantic.

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
3. name execution presentation semantics
4. name environment
5. snapshot the world or abstain from hermetic replay claims

This is the practical replay-hardening order because:

- unpinned dependencies are usually the highest-frequency ambient input class and close cleanly
  with lockfiles or equivalent dependency identity
- runtime and machine drift close next through pinned interpreter, base image, or equivalent
  runtime identity
- execution presentation drift is the next distinct residue when fixed inputs can still produce
  different observed results because ordering, NULL placement, timezone, locale, or numeric
  rendering changed
- ambient environment drift remains in play once dependency, runtime, and presentation identity
  are pinned
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
- a replay against frozen data with ambient presentation semantics is also not hermetic for any
  claim those semantics can influence; the snapshot narrows data drift but does not acquit runtime
  presentation drift
- the mature operator choices are:
  - snapshot or vendor the external state
  - replay against a frozen mirror
  - or abstain from calling the result replay-verified

This should stay aligned with V11.9:

- V11.9 promotes ambient inputs into cited inputs on the authoritative decision path
- V11.10 uses that stronger named-input set to decide whether a baseline is hermetic,
  replay-verified, witness-only, or still partly ambient

### 6a. Query identity carrier before attribution

Ota must define the machine record before publishing query-attribution guidance. The first carrier
is a receipt-attached witnessed record recovered from a contract-declared structured trace source:

```json
{
  "id": "query_identity:<task>:<run>",
  "kind": "emitted_query_identity",
  "input_class": "witnessed_query_output",
  "source_path": "<declared structured trace path>",
  "source_identity": "sha256:<captured-trace-file>",
  "evidence_class": "attested",
  "records": [
    { "subject": "<id>", "run": 0, "identity": "sha256:<canonical-subject-query>" }
  ]
}
```

The trace source is contract-declared; the identity and record are captured by Ota at the selected
execution boundary. The trace is a witnessed observation, not an `evaluated_input`: it must remain
separate from current-run decision inputs so receipt comparison can distinguish named inputs that
held still from observed behavior that changed.

The first concrete admission is the Bedrock JSONL shape, one record per `(id, run)` with a `sql`
field. Ota preserves `run` as trace context and hashes the exact `(id, sql)` query identity so
a stable query repeated across K runs does not become false divergence. It preserves the
repo-emitted trace as witnessed evidence rather than attempting to regenerate a query from the
model. Other trace formats stay out of scope until they have equally explicit run identity and
query fields.

### 6b. Query identity before data or model attribution

For model-mediated or generated-query lanes, Ota should use the strongest cheapest split first:

1. capture or consume the emitted query identity for each run
2. compare query identity before attributing a changed result to the model, data, or runtime
3. when query identity is fixed, compare against frozen data plus the declared comparator and any
   relevant presentation profile before attributing the remaining flap

The first Bedrock-shaped interpretation is intentionally narrow:

- changed captured SQL across K runs is witnessed evidence of generation divergence; attribute it
  to model variance only when prompt/context, model configuration, and runtime identity are held
  fixed
- fixed SQL that changes only against live data, with comparator and presentation semantics held
  fixed, points toward external-state drift
- fixed SQL that changes against the same frozen data but has an ambient comparator or
  presentation profile remains comparison/runtime drift, not model variance
- a data fingerprint or frozen fixture alone is `narrowing`; it becomes part of a hermetic claim
  only alongside the relevant runtime and presentation inputs

Ota should consume an existing repo-emitted query trace where it is already authoritative. It must
not duplicate application instrumentation or attempt to replay a model decision function from an
opaque provider. Query output is witnessed evidence; Ota evaluates the replay boundary around it.

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
- static replay-input identities are captured before selected execution begins, never reconstructed
  from a later filesystem read, and remain `narrowing` unless Ota has stronger input-specific
  semantics
- replay inputs declared by any reachable dependency task are aggregated into the selected lane's
  receipt with task-qualified identity; final-task declarations are never treated as the whole
  input set by default
- declared presentation profiles remain contract-owned and separate from any later runner-observed
  profile; absent or incomplete presentation truth downgrades hermeticity rather than being
  silently inferred
- query-attribution guidance ships only after its contract-declared trace source and receipt
  carrier use canonical `witnessed_query_output` identity
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
