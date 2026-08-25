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

# Cross-Cutting Plan: OSS Adapter and Profile Conformance

Status: planned and inactive. This plan does not register, support, certify, activate, or ship any
adapter, profile, carrier, provider, effect family, or external consumer.

## Purpose

Ota's typed surfaces intentionally expand through bounded implementations:

- V12 effect families and realizations;
- V12.1 secret-delivery providers and adapters;
- V12.3 provider-attestation profiles;
- V12.4 and V12.5 protected platform carriers;
- future runtime, sandbox, provisioning, lifecycle, and policy adapters; and
- V12.6 evidence-export and repository-report consumers.

Without one conformance boundary, `supported` can collapse into `someone wrote an adapter`. That
would let incomplete capabilities, caller-authored trust labels, optimistic defaults, weak
cleanup, or one green demo enter Ota's authority and evidence model.

This plan defines the public registration, evidence, pressure, compatibility, and lifecycle rules
that an implementation must satisfy before an Ota release may call it supported.

## Activation And Ownership

This cross-cutting plan may be activated only by an active version slice that introduces the first
new registered implementation after independent review of this plan. It never activates a product
version on its own and cannot bypass that slice's acceptance bar.

Core owns:

- canonical profile kinds, identities, registry semantics, capability vocabulary, and refusal;
- independent verification and fail-closed dispatch;
- conformance fixtures and machine-readable results; and
- the support/deprecation/revocation posture shipped by an Ota release.

An adapter implementation owns only its bounded observation, materialization, enforcement,
cleanup, or consumption behavior. It cannot assign its own authority posture, support status,
assurance result, or compatibility claim.

Repository contracts, policy files, callers, plugins, environment variables, and Enterprise
control planes cannot register a trust-sensitive profile dynamically.

## Applicability Classes

Every implementation declares exactly one primary class:

1. `observation`: reads bounded external truth and cannot authorize or mutate;
2. `projection`: renders a provider/runtime plan but does not execute it;
3. `materialization`: creates bounded resources or delivers material;
4. `enforcement`: establishes and verifies an execution/effect boundary;
5. `authority`: supplies or verifies identity, attestation, decision, grant, or lease truth; or
6. `consumer`: verifies, retains, indexes, or displays exported Ota evidence.

A profile may depend on lower-authority profiles, but it cannot inherit a stronger class through
composition. Any profile performing more than its declared class is invalid.

Observation and consumer extensions may eventually support third-party packaging. Materialization,
enforcement, and authority profiles remain statically registered or installed through an equally
protected reviewed mechanism; arbitrary repository-loaded code cannot enter those paths.

## Canonical Semantic, Subject, And Registration Descriptors

Profile behavior, the exact implementation claim being tested, and reviewed registration evidence
are three separate immutable identity layers.

One versioned `AdapterProfileDefinition` derives `profile_semantic_identity` from only the shared
behavioral contract:

- profile kind, canonical profile ID, and schema/profile version;
- supported operating systems, architectures, providers, runtimes, and deployment postures;
- applicability class and complete capability set;
- required inputs, protected inputs, outputs, effects, resources, and disclosure classes;
- authority source and verifier semantics where applicable;
- dry-run, real-execution, cancellation, recovery, cleanup, and persistence semantics; and
- unsupported, unknown, conditional, and explicitly unproved behaviors.

It excludes implementation owner, source repository, source tree, build artifact, mutable build
location, threat model, pressure evidence, release evidence, lifecycle, display labels,
documentation URLs, and timestamps. Capability, authority, target, effect, cleanup, persistence,
verifier, or proof-boundary changes require a new profile version and
`profile_semantic_identity`.

One versioned `AdapterImplementationSubject` derives `implementation_subject_identity` from:

- the exact `profile_semantic_identity` implemented;
- implementation owner, source repository, source tree, build, and artifact identities;
- claimed Core and Protocol capability ranges; and
- the exact claimed operating-system, architecture, provider, runtime, and deployment posture.

The claimed target posture must be a subset of the profile definition's permitted targets. An
implementation may register separately proved target subsets, but it cannot reinterpret shared
behavior or claim an untested target through another subject identity.

Every registration evidence artifact, including the threat model, conformance suite/result,
pressure manifest/evidence, and release evidence, binds the exact
`implementation_subject_identity`. One versioned `AdapterImplementationRegistration` then derives
`implementation_registration_identity` from:

- the exact `implementation_subject_identity`; and
- threat-model, conformance-suite, conformance-result, pressure-manifest, pressure-evidence, and
  release-evidence identities.

The final registration reconciles every evidence record to the subject before including the
evidence identities, avoiding a self-referential identity cycle. Changing implementation ownership,
source, build, compatibility range, or target posture changes `implementation_subject_identity` and
therefore `implementation_registration_identity`. Changing pressure or release evidence changes the
registration identity without changing the profile or implementation subject identity.

`implementation_lifecycle` is separate registry state:
`registered_unproved | pressure_proven_candidate | release_enabled | deprecated | revoked`. A
lifecycle transition changes the signed `ProfileRegistrySnapshot` identity, not any registered
identity. A registry entry and every positive admission bind the profile semantic, implementation
subject, and implementation registration identities plus the observed lifecycle and registry
snapshot. Only Ota-owned release truth may assign `implementation_lifecycle`; an adapter manifest
cannot self-label itself pressure-proven or `release_enabled`.

## Required Registration Package

Every profile submission includes:

- a narrowly scoped design and ownership statement;
- a threat model naming trusted, untrusted, repository-controlled, operator-controlled, and
  provider-controlled components;
- canonical schemas, identities, deterministic fixture vectors, and implementation profile;
- exact capability and unsupported-behavior declarations;
- positive control, negative controls, substitution controls, interruption/recovery controls, and
  cleanup controls;
- bounded real-repository pressure with an uncovered-material-behavior inventory;
- compatibility, upgrade, downgrade, deprecation, revocation, and security-response rules;
- non-secret public evidence suitable for independent verification; and
- connected Core docs, JSON reference, Example, Skill, Site, and Learn updates when behavior ships.

Generated fixtures cannot establish provider, OS, or authority claims that require a real external
boundary. Emulators and mocks are development controls only unless the profile itself explicitly
governs an emulator.

## Capability Semantics

Capabilities are monotonic facts, not marketing labels. The canonical vocabulary distinguishes:

- `supported`: the exact profile enforces and pressure-proves the behavior;
- `conditional`: the behavior exists only under identity-bound named prerequisites;
- `unsupported`: the profile refuses before relevant mutation;
- `unknown`: the implementation cannot currently establish the fact; and
- `not_applicable`: the behavior cannot arise under that exact profile.

Absence never means supported. Unknown cannot be promoted to supported by policy, caller input,
documentation, Enterprise configuration, or another profile's evidence.

The profile definition declares capability semantics, but positive support is evaluated for the
exact profile semantic, implementation subject, and implementation registration identities. Two
implementations may have different target subsets and lifecycle posture, but neither may reinterpret
or partially satisfy behavior claimed by its exact subject. Linux evidence cannot establish macOS
or Windows support. One cloud provider cannot establish another provider. One secret destination,
effect action, runtime mode, or evidence consumer cannot establish a broader family.

## Conformance Suite

Core provides a versioned conformance harness that verifies:

- descriptor schema, canonical identity, capability closure, and compatibility ranges;
- deterministic request/response, plan, decision, evidence, receipt, and archive fixtures;
- missing, extra, contradictory, reordered, downgraded, cross-profile, and cross-version fields;
- authority, verifier, scope, target, resource, effect, recipient, transaction, and origin
  substitution;
- dry-run non-mutation and fail-closed unsupported behavior;
- cancellation, timeout, duplicate response, ambiguity, replay, recovery, and cleanup where the
  class can mutate or authorize;
- secret and protected-material disclosure controls;
- archive/export re-verification and historical compatibility; and
- human, plain, JSON, schema, and exit-code consistency.

The harness emits one content-addressed `ConformanceResult` binding the exact
`implementation_subject_identity` and naming every executed, skipped, unsupported, and unavailable
control. Registration verifies that subject identity before including the result identity. Skipped
or unavailable mandatory controls prevent
`pressure_proven_candidate` and `release_enabled` implementation lifecycle posture.

Passing conformance proves only implementation agreement with the registered profile. It does not
prove the external provider, OS, application, or repository is correct.

## Pressure Requirements

Trust-sensitive profiles require immutable pressure against the real claimed boundary:

- exact Core, Protocol, adapter/carrier, registry, provider/configuration, and fixture revisions;
- at least one positive path where the profile supports one;
- every material refusal, downgrade, replay, cleanup, and recovery path;
- registry snapshot substitution, sequence rollback, stale/expired freshness, unavailable refresh,
  and a revocation present only in a later not-yet-installed snapshot;
- repository/provider/host sentinels proving bounded mutation or zero mutation as claimed;
- unchanged unrelated repository and external state;
- retained public-safe artifacts sufficient for independent verification; and
- an uncovered-material-behavior inventory.

Pressure is target-specific. A multi-target profile closes only for targets independently proved.
Hosted CI counts only when it exposes the exact provider, privilege, reboot, process, storage, and
recovery semantics being claimed.

## Implementation Lifecycle

Lifecycle transitions are explicit:

```text
registered_unproved
  -> pressure_proven_candidate
  -> release_enabled
  -> deprecated
  -> revoked
```

- `registered_unproved` is unavailable for positive assurance and real trust-sensitive execution;
- `pressure_proven_candidate` is usable only in explicit pressure/development posture;
- `release_enabled` requires release inclusion, compatibility evidence, docs, and support ownership;
- `deprecated` remains verifiable for bounded historical/current compatibility while warning or
  refusing new selection according to a published deadline; and
- `revoked` refuses new admission when the accepted installed registry snapshot records that
  posture and remains readable only for historical evidence under explicit security guidance.

Implementation lifecycle is registration-identity-bound and machine-readable. Revocation cannot
erase historical evidence, and historical evidence cannot reactivate a revoked implementation.
Changing shared profile semantics requires a new `profile_semantic_identity`; when a semantic
profile is found unsafe, the registry revokes every affected implementation registration rather
than mutating the old semantic identity.

### Registry snapshot and freshness

Lifecycle is evaluated against one accepted installed `ProfileRegistrySnapshot`, not assumed
global state. The snapshot binds its schema and semantic identity, source and verifier identity,
monotonic sequence and predecessor identity where available, publication posture, optional expiry
or maximum-age rule, and every implementation lifecycle entry with its exact profile semantic,
implementation subject, and implementation registration identities.

Every positive trust-sensitive admission, receipt, and archive records the profile semantic,
implementation subject, implementation registration, and registry snapshot identities, observed
implementation lifecycle, source/verifier identity, and freshness posture
(`fresh | stale | installed_snapshot_only | unknown`). Historical verification retains that exact
snapshot rather than applying a later lifecycle retroactively.

An accepted snapshot that marks an implementation registration `revoked` blocks new admission
immediately relative to that installed snapshot. An older offline snapshot cannot prove awareness
of a later revocation.
When no freshness-bounded update source exists, evidence states `installed_snapshot_only`; it must
not claim globally current revocation knowledge. A profile requiring fresh registry truth refuses
when the required freshness cannot be established.

## Third-Party And Enterprise Boundary

Third parties may implement public interfaces and run the conformance harness. That does not make
their implementation Ota-supported. Ota may publish the result as third-party, unverified,
candidate, or `release_enabled` only according to `implementation_lifecycle` above. Per-capability
`supported` remains a separate result and cannot promote lifecycle.

Enterprise may distribute registered implementations and centrally manage allowed profile sets.
It may narrow availability but cannot register an unknown profile, widen capabilities, override a
revocation, or convert candidate pressure into OSS support.

## Implementation Order

1. Inventory every shipped and planned adapter/profile family and remove overlapping terminology.
2. Define descriptor, capability, lifecycle, and conformance-result schemas and identities.
3. Build deterministic fixture vectors and the class-aware conformance harness.
4. Register existing shipped implementations from current truth without upgrading their claims.
5. Add pressure-manifest and immutable-evidence verification.
6. Add deprecation/revocation and unknown-version behavior.
7. Integrate registration into capability output, receipts, archives, and assurance.
8. Publish contributor, provider, operator, Example, Skill, Site, and Learn guidance.
9. Require the first new V12+ profile to pass the complete path before closure.

## Acceptance Bar

- no implementation can self-assign support or authority posture;
- unknown, unregistered, incompatible, deprecated-after-deadline, or implementation registrations
  marked revoked by the accepted installed registry snapshot refuse before protected mutation or
  authority contact;
- capability omission, contradiction, widening, target substitution, profile downgrade,
  implementation-subject substitution, and `implementation_lifecycle` substitution invalidate
  admission and archived verification;
- two implementations of one profile produce identical canonical semantic fixtures and the same
  `profile_semantic_identity`, while retaining distinct `implementation_subject_identity` and
  `implementation_registration_identity` values and implementation evidence;
- different material capabilities, targets, authority sources, effects, recipients, or cleanup
  postures cannot alias one profile semantic identity;
- owner, source, build, compatibility-range, or target substitution cannot retain one implementation
  subject identity; pressure or release-evidence substitution cannot retain one implementation
  registration identity; lifecycle substitution cannot retain one registry snapshot identity;
- skipped mandatory controls prevent positive lifecycle promotion;
- historical evidence retains all three original identities and lifecycle-at-execution posture;
- admission, receipts, and archives retain all three exact identities plus the exact registry
  snapshot identity and freshness posture used for lifecycle evaluation;
- snapshot substitution, sequence rollback, stale-as-fresh relabeling, and claims of awareness of
  a later uninstalled revocation refuse verification;
- docs and capability output distinguish implementation lifecycle
  (`registered_unproved | pressure_proven_candidate | release_enabled | deprecated | revoked`)
  from capability status (`supported | conditional | unsupported | unknown | not_applicable`); and
- at least one independent implementation can run the OSS harness without access to private Ota or
  Enterprise code.

## Non-Goals

- a dynamic plugin marketplace for authority-sensitive code;
- certifying provider security or application correctness;
- allowing repositories or Enterprise to define trusted profile semantics;
- replacing version-specific acceptance bars;
- promising support for every third-party implementation; or
- planning specific effect families, secret providers, cloud providers, or platforms before demand.

## Definition Of Done

This plan completes only when registry semantics, all three identity layers, capability vocabulary,
implementation lifecycle, conformance harness, immutable pressure binding, historical verification,
public contribution guidance, and one real V12+ profile pass independent review. It remains
cross-cutting rather than a new Ota product version.
