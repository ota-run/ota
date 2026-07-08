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

# V11.8 Plan

Status: complete.

Release target:

- completed slice after `v11.7`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.4 plan](../v11.4/plan.md)
- [V11.5 plan](../v11.5/plan.md)
- [V11.6 plan](../v11.6/plan.md)
- [V11.7 plan](../v11.7/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.8 theme:

- sandbox policy compilation from the execution contract

This slice closes the remaining OSS gap between:

- Ota exporting governance truth
- and real execution chokepoints enforcing runtime filesystem and egress boundaries directly

The product goal is not:

- build Ota’s own universal sandbox runtime
- replace every existing container, VM, or CI isolation layer

The product goal is:

- compile contract-owned execution boundary truth into stable provider-facing sandbox policy

This slice is compilation of canonical governance truth, not a second independent policy-authoring
surface.

## Canonical product principle

Ota should not stop at saying what is callable.

When the contract already owns enough truth about:

- callable lanes
- writable/protected boundaries
- network/effect posture
- execution mode and backend

Ota should be able to compile that truth into real runtime boundary policy for the execution
systems that already exist.

That means:

- `ota.yaml` remains the canonical execution-governance spec
- V11.4 defines the canonical governance model and phase semantics
- V11.6 exports a harness-consumable profile of that model
- V11.8 compiles the relevant subset of that same model into sandbox/runtime policy targets
- no independent sandbox taxonomy should appear beside the contract and governance model

## Problem statement

Even with:

- runner-enforced agent mode
- governance JSON
- CI merge-gate projection
- harness-facing callable export

the strongest execution boundary still fails if downstream systems must guess:

- whether network should be denied by default
- which outbound hosts are allowed
- which filesystem paths are writable
- which repo paths must stay read-only
- which runtime mounts correspond to contract-declared writable boundaries

If that policy remains ad hoc per runner, the contract is still only partially authoritative.

V11.8 is the slice for making Ota’s execution-governance truth compilable into real runtime
boundary policy.

## Included capabilities

- contract-owned runtime boundary policy model for sandbox compilation
- default-deny network posture where the contract and policy pack require it
- explicit outbound policy modeling for host/service/domain and destination-shaped targets
- explicit writable mount boundary modeling derived from contract-owned edit and runtime boundary
  truth
- stable provider-facing compilation surface for supported sandbox targets
- machine-readable explanation of what was compiled, what remained advisory, and why

## Non-goals

- do not claim Ota can enforce host isolation without a cooperating runtime
- do not invent provider-specific policy as the source of truth
- do not replace V11.6 with a sandbox-only model
- do not make every advisory effect field suddenly enforcement-grade without a contract basis
- do not broaden this slice into enterprise approval, waiver, or fleet management

## Core product gaps

### 1. Harness export is still weaker than compiled runtime policy

V11.6 can tell a harness:

- what is callable
- what is review-required
- what boundaries matter

That still leaves runtime policy construction outside Ota.

### 2. Network posture is not yet a first-class compiled execution boundary

Today Ota can describe:

- `effects.network`
- `effects.network_kind`
- external-state posture

What it cannot yet do cleanly is compile:

- default deny
- explicit allowlist
- policy-backed egress posture

into a runtime boundary contract a sandbox can consume directly.

It also cannot yet distinguish between outbound destinations where host allowlisting is enough and
outbound destinations where the real risk lives at a narrower or second-hop boundary, such as:

- multi-tenant hosts
- relay/fetcher hosts
- payload-directed send hosts

It also does not yet declare the downstream-destination truth strongly enough for the lanes where
first-hop host policy is not the real boundary, such as:

- webhook delivery targets
- email recipient domains
- SMS destination scope
- downstream fetch/import targets behind a relay host
- object-storage bucket or tenant targets behind shared storage hosts

The contract-placement rule for this slice should stay strict:

- do not add a broad new top-level `runtime_policy` block
- do not teach `effects.network` or `effects.network_kind` to become the allowlist itself
- keep effect metadata as signal input
- add only the narrow execution-boundary declarations that the current contract does not yet own

### 3. Writable boundaries are declared, but not yet compiled

The contract already owns:

- `agent.writable_paths`
- `agent.protected_paths`

What is still weaker than it should be is the direct compilation of that truth into:

- writable mount policy
- read-only repo posture by default
- machine-readable runtime boundary explanations

Those agent edit-governance fields are a strong input, but they are not yet the full runtime
filesystem policy model.

### 4. Provider targets need a disciplined portability layer

The product should not expose:

- one contract shape per sandbox vendor

It should expose:

- one Ota-owned boundary model
- plus narrow provider compilation targets

so the portable governance truth stays above runtime-specific policy syntax.

## Ownership and precedence rules

### 1. Outbound policy ownership

V11.8 must define where outbound runtime truth actually lives.

The mature shape is:

- repo-local execution contract truth declares the narrow runtime outbound surface Ota can own
- task or workflow execution scope may narrow or widen that runtime egress surface where the
  selected lane truthfully differs
- policy packs may further restrict or require stronger defaults, but should not silently invent
  repo-local business destinations

Direction:

- existing `effects.network` and `effects.network_kind` remain signal inputs, not the outbound
  policy itself
- repo-wide default egress truth should live on the execution-governance side of the contract,
  not in provider config and not in a parallel top-level policy tree
- V11.8 adds a dedicated runtime-boundary layer for outbound policy truth rather than overloading
  generic effect metadata
- that layer must be compilable at repo, workflow, and task scope
- outbound target shape must be contract-owned and not left to provider-specific heuristics

The important part is:

- outbound policy truth does not live in provider config
- outbound policy truth does not get inferred only from effect posture
- outbound policy truth is declared canonically and then compiled

### 2. Filesystem boundary ownership

V11.8 must keep edit-governance and runtime sandbox policy related, but not identical.

Direction:

- `agent.writable_paths` and `agent.protected_paths` remain canonical edit-governance inputs
- V11.8 derives default runtime filesystem policy from those fields first
- when runtime truth needs narrower or differently scoped writable boundaries than edit truth,
  V11.8 introduces a dedicated runtime-boundary layer instead of teaching one field to mean two
  different things

The mature rule is:

- edit-governance truth is the baseline input
- runtime filesystem policy is the compiled execution boundary
- provider mounts are downstream renderings of that compiled runtime boundary

The contract-placement rule should stay explicit:

- repo-level runtime filesystem defaults should derive from `agent.writable_paths` and
  `agent.protected_paths`
- explicit runtime filesystem widening belongs on execution/runtime boundary declarations, not by
  mutating the meaning of `agent.*_paths`
- `agent.*_paths` stay the edit-governance baseline even when runtime compilation grows richer

### 3. Precedence rule

V11.8 should define one explicit precedence model before implementation:

1. policy-pack hard restrictions
2. task-scoped runtime boundary truth
3. workflow-scoped runtime boundary truth
4. repo-level runtime boundary truth
5. derived defaults from existing contract fields such as `agent.writable_paths` and
   `agent.protected_paths`
6. effect metadata and posture signals only as advisory inputs where no stronger runtime boundary
   truth exists

That keeps:

- hard deny stronger than local allow
- task-owned executable truth stronger than workflow path restriction for the same concrete field
- lane-specific runtime truth stronger than repo-wide defaults
- derived defaults weaker than explicit runtime-boundary declarations
- effect posture from masquerading as authoritative allowlist truth

Same-lane merge semantics must also stay explicit:

- when the effective executable lane carries both workflow-scoped and task-scoped runtime boundary
  truth for the same boundary family, task-scoped truth is the narrower canonical owner
- workflow-scoped runtime boundary truth may add surrounding operational restriction for the
  selected path, but it must not silently contradict task-scoped truth for the same concrete
  field
- if both scopes declare incompatible values for the same concrete boundary field and the conflict
  cannot be reduced to a monotonic narrowing, validation should fail instead of letting the
  compiler guess

That keeps:

- the executable task as the owner of lane-local runtime truth
- workflow boundaries useful for path-level restriction and composition
- the compiler deterministic instead of heuristic when both scopes participate

## Proposed implementation slices

### 1. Runtime boundary policy model

Define one additive contract/governance layer for compiled sandbox policy.

Direction:

- callable surface still comes from V11.3/V11.6
- runtime boundary policy covers:
  - filesystem posture
  - writable mount set
  - read-only default posture
  - network default posture
  - outbound allowlist posture

This should remain a derived policy layer over existing contract truth plus narrow new
contract-owned declarations where the current contract is not yet sufficient.

The model should start as one Ota-owned runtime-boundary layer, not a provider-shaped schema.

Contract-shape direction:

- keep repo-wide edit authority under `agent`
- keep execution/provider selection under `execution`, task execution, and workflow execution
  truth
- add runtime-boundary declarations alongside execution truth, not as a second top-level taxonomy

That means the likely ownership shape is:

- repo baseline:
  - derive filesystem defaults from `agent.writable_paths` / `agent.protected_paths`
  - declare repo-wide runtime-boundary defaults in the execution/governance layer only where
    derivation is not sufficient, especially for outbound policy
- task lane:
  - allow task-scoped runtime-boundary specialization beside task execution/runtime truth
- workflow lane:
  - allow workflow-scoped runtime-boundary specialization where the operational path owns the
    effective boundary

The important constraint is:

- no parallel workflow-only or provider-only policy surface
- one boundary model, attached to the executable lane that actually owns the crossing

### 2. Egress policy shape

Define a first-class Ota model for outbound network control.

Direction:

- default posture:
  - `deny`
  - `allow`
- explicit ownership:
  - repo-level runtime boundary declaration for shared defaults
  - task/workflow execution-boundary declaration for lane-specific widening or narrowing
  - policy-pack restriction overlays for stronger org-wide control
- explicit outbound targets such as:
  - declared hosts
  - declared domains
  - declared service aliases
- explicit destination-shape classification where first-hop host truth is not enough:
  - `single_purpose_host`
  - `multi_tenant_host`
  - `relay_host`
  - `send_host`
- explicit destination-constrained outbound declarations for the lanes where the effective
  destination is narrower than the first-hop host
- narrow destination-constrained outbound exceptions where the effective destination is the real
  control surface rather than the first-hop host alone
- existing effect posture stays advisory unless promoted by explicit runtime-boundary truth

The destination-shape classes should mean:

- `single_purpose_host`
  - host/domain/service allowlisting is usually sufficient
- `multi_tenant_host`
  - host allowlisting alone is too broad because many tenants share one runtime destination
  - the contract should require a stronger narrowing signal such as tenant/bucket/project scope,
    private endpoint, or equivalent policy-backed constraint
- `relay_host`
  - the first-hop host can fetch or reach arbitrary downstream destinations
  - the contract should require downstream destination constraints instead of treating the first
    hop as sufficient
- `send_host`
  - the effective destination lives in payload-level recipients, callbacks, or webhook targets
  - the contract should require recipient/callback allowlist posture above raw host egress

This destination-constrained outbound shape should stay narrow:

- host allowlisting remains the default baseline
- destination-constrained outbound is the exception model for lanes where first-hop host truth is
  not the real boundary
- the contract should not require destination-shape policy for ordinary single-purpose outbound
  lanes where host/domain/service allowlisting is already sufficient

The likely contract-owned examples are:

- allowed webhook callback or delivery destinations
- allowed email recipient domains
- allowed SMS destination scope
- allowed object-storage targets where first-hop shared host truth is too broad

The destination-constrained declarations should be first-class contract truth, not just a note that
one host class is special.

The likely declaration shapes are:

- downstream webhook or callback destination allowlists
- recipient-domain allowlists
- SMS destination allowlists or scoped destination groups
- downstream host/path allowlists for relay-style fetchers
- bucket/tenant/project target constraints for shared storage or multi-tenant service lanes

This keeps three different control layers explicit:

- network boundary
- credential or tenant boundary
- effective destination boundary

The important part is:

- Ota declares the portable meaning
- provider compilation renders the provider syntax

The likely declaration point should be:

- repo-wide default outbound policy on the execution-governance boundary layer
- narrower lane-specific outbound policy attached to the task or workflow execution lane that
  actually performs the call
- no host allowlist embedded in provider config or effect metadata

The ownership and distribution rule should stay explicit:

- one canonical destination-policy declaration may live in a versioned shared source
- each repo or service must pin and consume that shared truth locally
- no central policy service should sit in the hot path for execution
- missing or unpinned destination-policy truth should fail closed when the selected lane requires
  destination constraints

Staleness needs a stricter lifecycle than a single binary state.

The model should distinguish:

- missing / unresolved pin
- pinned and fresh
- pinned but aging
- pinned and stale beyond tolerance

The intended posture is:

- missing or unresolved required destination truth blocks immediately
- aging pinned truth warns first
- only clearly stale pinned truth blocks

That avoids turning ordinary deploy lag into accidental service outages while still making pin age
 visible and governable

The intended reuse model should stay explicit:

- repo-local declaration and enforcement is the default honest path for one-off or low-fanout
  lanes
- central declaration becomes valuable when the same destination truth is reused across multiple
  repos or services and copy-paste drift becomes the real risk
- even in that shared case, the decision still executes locally in the repo-owned send or relay
  path
- Ota should not assume a live central decision service or central proxy to make the crossing
  authoritative

The operational stale-pin rule should also stay explicit:

- the dangerous failure is often not rollout coordination but quiet pin age
- rollout lag is loud; stale pins are silent unless Ota surfaces them on purpose
- the goal is not "fail every stale pin immediately"
- the goal is "make pin age visible early and blocking only past a hard limit"

Compilation rules should vary by class:

- `single_purpose_host`
  - compile direct host/domain/service allowlists where supported
- `multi_tenant_host`
  - compile first-hop host policy when possible, but mark tenant narrowing as additionally
    required unless the contract already owns it
- `relay_host`
  - compile first-hop host policy plus downstream-destination posture
- `send_host`
  - compile first-hop host policy plus recipient/callback destination posture, even when the
    runtime cannot hard-enforce payload recipients directly

Destination-constrained outbound should compile as:

- hard runtime policy where the runtime can really enforce the downstream destination boundary
- explicit local app-path enforcement requirement where the runtime cannot express the effective
  destination boundary directly but the repo-owned lane still can
- machine-readable advisory only where neither the runtime nor the selected lane can enforce the
  destination boundary directly

The enforcement posture must be explicit in compiled output:

- `authoritative_runtime_enforced`
- `authoritative_app_enforced`
- `advisory_only`

The source posture for destination-constrained truth must also be explicit:

- `repo_local_authoritative`
- `shared_pinned_authoritative`
- `non_authoritative`

Freshness posture for shared-pinned destination truth must also be explicit:

- `fresh`
- `warning`
- `blocking`

That source posture answers a different question from the enforcement posture:

- source posture says where the authoritative destination truth came from
- enforcement posture says where that truth is actually enforced
- freshness posture says whether pinned shared truth is still within the tolerated lifecycle

The mature destination-constrained model is therefore:

- data may be local or centrally declared
- the decision remains local
- the selected lane warns on aging pinned truth and fails closed only on missing, unresolved, or
  hard-stale pinned truth
- no live central dependency is required for governed execution

That posture should be published per destination-constrained lane so operators and harnesses can
see whether Ota is compiling:

- real runtime boundary policy
- required local lane enforcement
- or only a non-authoritative warning surface

Explainability should also stay explicit:

- host allowlist was authoritative or advisory
- destination constraint was required or not required
- destination constraint was runtime-enforced, app-enforced, or advisory-only
- destination truth was repo-local authoritative or shared-pinned authoritative
- shared destination-policy source was pinned, aging, stale, missing, or unresolved
- pinned destination-policy version or digest was visible in execution evidence
- pinned destination-policy age and hard-limit posture were visible in governance output

The important rule is:

- do not replace host allowlisting with destination-constrained outbound
- layer destination-constrained outbound on top only where the selected lane truthfully needs it

### 3. Filesystem policy shape

Define one portable writable/read-only boundary model.

Direction:

- repo root is read-only by default in compiled agent execution unless explicitly widened
- runtime filesystem defaults derive first from `agent.writable_paths` and `agent.protected_paths`
- protected paths compile as explicit read-only or denied writes where supported
- if a lane needs runtime writable boundaries that differ from edit-governance truth, the runtime
  boundary model owns that widening explicitly instead of mutating `agent.*_paths` semantics
- provider compilation may widen exact mount semantics, but not invent the policy

The likely declaration point should be:

- repo-wide edit defaults remain under `agent`
- lane-specific runtime writable/read-only truth attaches to the executable task/workflow runtime
  boundary when the sandboxed execution boundary is intentionally different from human edit
  authority
- provider mount generation consumes that compiled lane boundary rather than trying to reinterpret
  repo paths ad hoc

### 4. Provider compilation targets

Keep targets narrow first.

Direction:

- first real target:
  - Codex local sandbox / desktop agent execution target
- then:
  - CI runtime target
- generic harness-facing compiled profile

The acceptance bar is not “every provider.”
The acceptance bar is:

- Ota can compile one canonical runtime-boundary model into the Codex local sandbox target first
- and explain where another target would receive authoritative versus advisory fields
- and make destination-shape constraints explicit instead of pretending every outbound host is the
  same kind of control surface

### 5. Evidence and explainability

Compiled policy must not be silent magic.

Ota should publish:

- what boundary policy was compiled
- which fields were authoritative
- which fields stayed advisory
- which outbound targets required stronger narrowing than first-hop host allowlisting alone
- which destination-constrained lanes required pinned shared policy truth
- whether shared destination policy was resolved locally, missing, unresolved, aging, or stale
- whether a destination-constrained lane relied on runtime enforcement or local app-path
  enforcement
- whether destination truth was repo-local or shared-pinned at the time of compilation
- which pinned destination-policy version, revision, or digest was in force
- how old that pinned truth was
- whether the freshness posture was `fresh`, `warning`, or `blocking`
- why a lane was denied or widened

This visibility should not live only in one place.

Direction:

- governance output should carry source posture, freshness posture, and pinned identity
- receipts should carry the same pinned identity and freshness posture so later audits can answer
  "which truth did this run enforce"
- when the selected lane actually starts, the same pinned identity should be available to runtime
  logs or startup evidence so operators do not need a forensic dig to answer which destination
  truth was active

The mature operator question should become:

- which destination truth is this service or run on right now

not:

- which repo, build, or deploy six weeks ago happened to pull which shared config

This should stay machine-readable and aligned with the V11.4 governance model rather than
becoming provider-specific prose.

### 6. Freshness lifecycle and fan-out posture

Pinned shared destination truth should not be governed by one implicit age rule.

Direction:

- repos or policy packs should be able to define a warning threshold and a hard blocking limit
- the default operational stance should be "warn first, block later"
- hard blocking should be reserved for pins that are clearly outside tolerated freshness, not for
  every ordinary lagging consumer

This slice should also acknowledge, without overloading itself, the best follow-on automation:

- a change to central destination data should be able to fan out consumer rebuild or verification
  triggers where the surrounding CI platform supports it

That fan-out orchestration is adjacent and useful, but it is not the core of `11.8`.

The core of `11.8` is:

- declaring the pinned truth
- compiling the local enforcement boundary
- surfacing freshness early
- blocking only when the pin is truly too stale or missing

## Acceptance bar

V11.8 is complete when:

- Ota can compile contract/governance truth into a stable runtime-boundary policy model
- the model covers callable surface, writable boundaries, and network posture coherently
- the model distinguishes simple host allowlists from multi-tenant, relay, and send-style
  outbound destinations without inventing a second policy taxonomy
- destination-constrained outbound lanes are first-class contract truth rather than implied host
  annotations
- the compiled output makes central-declaration / local-enforcement posture explicit for
  destination-constrained lanes
- the compiled output distinguishes destination-truth source posture from enforcement posture
- the compiled output also distinguishes freshness posture for shared-pinned truth
- pinned destination-policy identity and age are visible in governance output and receipts
- warning thresholds and hard blocking limits for stale shared-pinned truth are explicit
- at least one real sandbox/runtime target can consume that compiled policy
- the compiled output makes authoritative versus advisory policy explicit
- the provider-facing compilation is clearly derived from the canonical governance model instead
  of inventing a parallel policy taxonomy

## OSS / enterprise boundary

This runtime-boundary compilation surface stays in OSS.

What can later build on top of it in enterprise is:

- centralized policy rollout
- org-wide sandbox target policy management
- approval and waiver workflows for widened lanes
- fleet-wide reporting on compiled boundary posture

The important OSS boundary is:

- Ota owns the portable execution-governance spec
- Ota can compile that spec into real runtime policy
- enterprise later manages that policy across many repos and runners
