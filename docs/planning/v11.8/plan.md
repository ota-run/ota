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

Status: planned.

Release target:

- planned slice after `v11.7`

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
2. task/workflow-scoped runtime boundary truth
3. repo-level runtime boundary truth
4. derived defaults from existing contract fields such as `agent.writable_paths` and
   `agent.protected_paths`
5. effect metadata and posture signals only as advisory inputs where no stronger runtime boundary
   truth exists

That keeps:

- hard deny stronger than local allow
- lane-specific runtime truth stronger than repo-wide defaults
- derived defaults weaker than explicit runtime-boundary declarations
- effect posture from masquerading as authoritative allowlist truth

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
- why a lane was denied or widened

This should stay machine-readable and aligned with the V11.4 governance model rather than
becoming provider-specific prose.

## Acceptance bar

V11.8 is complete when:

- Ota can compile contract/governance truth into a stable runtime-boundary policy model
- the model covers callable surface, writable boundaries, and network posture coherently
- the model distinguishes simple host allowlists from multi-tenant, relay, and send-style
  outbound destinations without inventing a second policy taxonomy
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
