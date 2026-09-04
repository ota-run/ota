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

# Cross-Cutting Strategy: Agent Execution Governance Core

Status: planning guidance only. This strategy does not activate a product version, implementation
slice, provider, adapter, carrier, public command, schema, evidence claim, or support posture.

## Strategic Thesis

Ota should become the independent execution-governance and evidence foundation between agents,
repositories, and consequential infrastructure.

The canonical flow is:

1. an agent, human, or CI system selects repository work;
2. Core derives the exact contract snapshot, selected closure, requirements, effects, recipients,
   target, and expected evidence posture;
3. independently sourced authority supplies protected truth that Core reconciles before producing
   one exact admission decision;
4. after admission, a reviewed adapter opens one provider/delivery transaction and obtains and
   exercises only the admitted external capability;
5. Core reconciles provider acknowledgement with bounded effective-runtime observation;
6. the selected transaction terminates with explicit execution, cleanup, and recovery posture; and
7. a privileged verifier with the required protected attachments can independently re-derive the
   complete retained evidence, while a public verifier can validate only the bounded public
   projection and its declared verification loss.

The repository contract requests and describes. Independent authority supplies protected truth.
Core reconciles that truth and admits or refuses. Runtime evidence witnesses. No one plane may
manufacture truth owned by another.

## Why This Strategy Exists

More capable agents reduce the value of documentation-only command discovery. They increase the
value of an external boundary that can determine what was selected, what authority applied, which
effects were permitted, whether the exact operation occurred, and what the retained result proves.

Ota must therefore preserve repo readiness as its adoption wedge while progressing toward governed
execution. A model's confidence, generated plan, successful command, provider acknowledgement, or
green workflow conclusion is never sufficient evidence by itself.

This is a direction for Core, not a claim that the complete system exists today.

## Core Planes

### Contract Plane

Repository-owned truth defines tasks, workflows, closure, requirements, effects, recipients,
targets, and required evidence posture. A contract may request authority but cannot grant it,
select a credential, register an implementation, or declare an observed result.

### Authority Plane

Separately sourced policy, bindings, trust roots, grants, approvals, implementation registrations,
and lifecycle state supply protected authority truth that may narrow already valid contract truth.
Core independently verifies and reconciles those inputs before producing admission. Repository
files, task output, caller labels, environment variables, and model assertions cannot manufacture
authority or admission.

### Admission And Transaction Plane

One retained command-scoped admission per invocation binds the exact selected graph, contract,
requirements, effects, authority inputs, policy decision, and structural eligibility before
provider contact or side effects. Only after that admission may a separately identified
provider/delivery transaction bind the adapter, provider operation, recipient process tree,
interruption posture, cleanup authority, and terminal state. Re-resolution, substitution, partial
progress, retry, recovery, and replay remain explicit. Admission never implies that a provider
transaction opened or completed.

### Observation Plane

Provider acknowledgements and effective-runtime observations are distinct. An acknowledgement may
show that a provider accepted or reported an operation. Only a versioned observation profile may
witness the exact dimensions it can observe. Missing, stale, incomplete, or unsupported observation
remains `unknown`.

### Evidence Plane

Phase-accurate receipts and archives bind the transaction, source authority, observed state,
cleanup, disclosure posture, and retained proof limits. Public projections are separately derived
and cannot expose private correlation material or become selectors, trust roots, or current
admission authority.

### Enforcement Plane

Ota is a mandatory chokepoint only when an adopted external boundary delegates execution to Ota or
withholds a required capability unless an Ota admission succeeds. A wrapper, convention, generated
workflow, or agent instruction is not a mandatory chokepoint. Unrestricted raw shell remains outside
Ota's control unless the runtime, credential owner, provider, or protected carrier enforces the
boundary.

## Existing Roadmap Ownership

This strategy maps existing plans; it does not duplicate or reorder them:

- V12 owns typed effects, realizations, shared policy admission, refusal controls, and bounded
  negative evidence.
- V12.1 owns the first end-to-end provider transaction: GitHub Actions OIDC, Google Workload
  Identity Federation, one exact Secret Manager version, selected-process delivery, cleanup, and
  phase-accurate evidence.
- V12.2 owns monotonic contract-authored crossing requirements without contract-authored authority.
- V12.3 owns one demand-gated provider-attested authority carrier.
- V12.4 and V12.5 own demand-gated protected macOS and Windows authority carriers.
- V12.6 is demand-gated by an independent evidence consumer and a real organizational
  interoperability need. It owns OSS evidence portability, repository reporting, authority
  history, and external interoperability without allowing consumers to redefine Core truth.
- the adapter/profile conformance plan owns registration, capability, effective-runtime
  observation, pressure, support, deprecation, and revocation requirements;
- the authority-distribution lifecycle plan owns protected artifact installation, upgrade,
  rollback, recovery, and removal.

Every owning plan keeps its current status and activation gates. This strategy cannot move work
between them merely because the shared destination is now explicit.

## First Complete Vertical Slice

V12.1 is the first required vertical slice. Closure must demonstrate one bounded path:

```text
exact GitHub checkout
  -> exact selected repository closure
  -> independently sourced binding and policy
  -> admitted OIDC/WIF transaction
  -> exact Secret Manager version
  -> selected recipient process tree
  -> terminal cleanup and recovery posture
  -> protected evidence and public bounded projection
```

The slice must refuse stale checkout truth, substituted authority, wrong workload claims, ambiguous
bindings, unsupported targets, partial materialization, injection failure, interruption, cleanup
failure, replay, and evidence tampering. It must not claim application correctness, secret-value
correctness, host-global containment, arbitrary descendant exclusion, or organization-wide support.

Completing models, schemas, or provider-free tests is not completion of this vertical slice.

## Future Chokepoint Gate

No future slice may claim mandatory agent execution control until all of the following are true:

- one exact runtime, CI, credential, provider, or protected-carrier boundary is named;
- the required capability is unavailable to the selected agent path outside admitted execution;
- bypass, alternate entry points, raw shell, inherited credentials, and recovery paths are threat
  modeled and pressure tested;
- authority and execution ownership are independently administered or the weaker posture is named;
- effective-runtime observation covers each claimed enforced dimension;
- interruption and cleanup have terminal evidence; and
- public output states every material behavior that remains bypassable or `not_proved`.

Adoption is part of this gate. A technically enforceable integration that no real operator will
make mandatory is pressure evidence, not proof of a viable control point.

## Core And External Ownership

OSS Core owns canonical schemas, identities, semantic reconciliation, admission, transaction state,
adapter/profile interfaces, verification, local archives, public projections, and reference
implementations sufficient for independent end-to-end use.

External systems may supply authority, provider operations, observations, protected storage, or
evidence consumption. They cannot redefine Core identity, result, scope, authority posture, or proof
limits. This strategy does not authorize an Enterprise control plane or assign private product
scope.

## Product Non-Goals

Ota does not become:

- a general CI scheduler;
- an infrastructure-as-code engine;
- a secret manager or identity provider;
- a cloud, database, or sandbox provider;
- a universal endpoint monitor;
- an autonomous approval authority; or
- a substitute for application-specific correctness tests.

Ota governs and verifies bounded use of those systems through reviewed integrations.

## Proof Ladder

Claims advance only through these separately named execution stages:

1. `declared`: contract or protected source contains canonical intent;
2. `structurally_eligible`: complete retained truth reconciles before provider contact;
3. `admitted`: Core reconciles exact independently sourced authority truth and permits one exact
   command invocation to proceed toward, but does not open, a provider transaction;
4. `provider_acknowledged`: the provider reports accepting or completing a bounded operation;
5. `witnessed_effective`: a versioned observation profile witnesses named effective dimensions;
6. `terminally_reconciled`: execution, interruption, cleanup, and recovery reach an explicit state.

After terminal reconciliation, verification has two sibling postures rather than stronger and
weaker sequential stages:

- `privileged_rederived`: an independent verifier with the required protected attachments can
  re-derive the complete retained evidence; and
- `publicly_verifiable`: an independent verifier can validate the bounded public projection and its
  explicit verification loss without treating omitted protected truth as re-derived.

Neither verification posture implies the other. No execution stage implies a later stage. Positive
support claims require the exact profile's conformance, immutable pressure, release registration,
and lifecycle posture.

## Adoption And Falsification

Technical ambition is not sufficient. Before broadening beyond each vertical slice, Ota must record:

- the repeated operational failure or governance ambiguity;
- who pays the cost and how often;
- which existing tool or process fails to close it;
- whether an operator will route a real lane through Ota;
- whether the operator will make the boundary mandatory; and
- whether the retained evidence changes review, incident response, or audit decisions.

The strategy should be narrowed or rejected if capable agents make repository execution reliable
without an independent control boundary, operators consistently refuse to adopt a chokepoint, Ota
evidence does not change consequential decisions, or existing CI/provider controls solve the same
problem with less integration cost.

## Activation Discipline

Committing this strategy makes it a review and sequencing reference only. It never becomes an active
implementation version.

The active implementation remains V12.1 Step 6. No later step or version activates through this
strategy. V12.2 may activate only after V12.1 completes or is formally deferred and V12.2's own
gates are satisfied. Every later version and cross-cutting plan retains its own activation and
demand requirements.

New work not already owned by those plans requires a separately reviewed inactive plan that names
one bounded capability, one implementation owner, one pressure target, one proof boundary, and its
relationship to the active version. Strategic fit alone never authorizes implementation.
