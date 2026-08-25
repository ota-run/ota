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

# V12.6: OSS Enterprise Interoperability Foundation

Status: planned and inactive. This plan does not authorize a hosted service, Enterprise control
plane, account system, approval UI, fleet database, or private implementation.

## Activation Gates

V12.6 may be activated only after:

- V12.5 completes or is formally deferred;
- V12 through V12.5 have stable public identities and evidence branches for every slice that
  actually shipped;
- at least one independent consumer needs to ingest Ota evidence across repositories without
  replacing Core verification;
- one real organization case demonstrates a concrete approval-reference, waiver-reference,
  evidence-portability, retention, or repository-reporting need;
- public/private disclosure boundaries receive independent privacy and security review; and
- the current handoff explicitly activates V12.6 after independent plan review.

This sequencing does not require every optional provider or platform carrier to ship. A predecessor
may be formally deferred, but its unsupported evidence branch cannot be invented by V12.6.

## Why This Is V12.6

Ota already owns substantial OSS evidence truth:

- V11.7 grants, signed authorization decisions, optional approval references, one-use leases,
  receipts, protected history, and archive verification;
- local organization-policy loading, validation, evaluation, provenance, and JSON output;
- verifier identities, key IDs, bounded overlap posture, and historical verification;
- local receipts, archives, snapshots, baselines, and machine-readable repository reports; and
- versioned protocol domains, JSON schemas, and semantic identities.

Those capabilities are individually useful, but Enterprise consumers must not infer missing
relationships, invent waiver semantics, scrape command output, or reinterpret a local archive as a
fleet-wide claim. V12.6 defines the public interoperability boundary that every hosted, private,
or third-party consumer must respect.

## Product Boundary

OSS Core owns canonical evidence semantics and independent verification. Enterprise may administer,
route, retain, aggregate, and present that evidence, but it cannot redefine its identity, authority
posture, result, scope, or proof limits.

The boundary is one-way with respect to truth:

- Core emits or verifies canonical evidence;
- an external system may retain, index, correlate, or display it;
- external approval, waiver, policy, or verifier inputs become authoritative only through a
  registered, independently verified Core admission path; and
- importing an Enterprise record never makes it true merely because Enterprise stored it.

V12.6 does not make Ota a certificate authority, identity provider, policy host, approval service,
key manager, evidence warehouse, SIEM, or fleet control plane.

## Umbrella And Sub-Slice Closure

V12.6 is a roadmap umbrella, not one atomic implementation slice. Its trust-sensitive surfaces
activate and close independently in this order unless independent plan review establishes a safer
dependency order:

1. `V12.6a portable_export`: Core-owned export profiles, deterministic packaging, and offline
   verification;
2. `V12.6b repository_reporting`: canonical per-repository posture reports and bounded consumer
   ingestion;
3. `V12.6c authority_history`: normalized policy-authority evidence plus public verifier-history
   and rotation-event evidence; and
4. `V12.6d authority_references`: approval- and waiver-reference semantics over already registered
   Core admission surfaces.

Each sub-slice requires its own activation statement, schema and identity review, implementation
boundary, compatibility fixtures, pressure artifacts, first-party propagation, and independent
closure review. Only one sub-slice may be active at a time, and each successor requires its
predecessor to complete or be formally deferred. Completion of one authorizes claims only for that
sub-slice and does not imply the others exist.

V12.6 as an umbrella completes only when every sub-slice is either complete or formally deferred
with its unsupported public claims removed. A deferral cannot invent placeholder evidence or let a
consumer infer an unimplemented authority relationship.

## Existing Foundations And Named Gaps

### Already owned by OSS

- grant, authorization-decision, lease, transaction, receipt, and archive identities;
- optional broker approval-reference carriage;
- local policy-pack schema, resolution, validation, evaluation, and provenance;
- verifier-set and rotation-overlap posture used by signed authority evidence;
- repository-scoped receipt history, snapshots, baselines, and JSON output; and
- public protocol and schema versioning per shipped surface.

### Gaps allocated to the sub-slices

- `authority_references`: canonical approval- and waiver-reference evidence without implementing
  hosted approval or waiver workflows;
- `authority_history`: one normalized policy-authority evidence envelope plus verifier-history and
  rotation-event evidence without operating policy distribution or key rotation;
- `portable_export`: deterministic portable evidence-export manifests with explicit disclosure
  posture and offline verification;
- `repository_reporting`: one canonical per-repository posture report suitable for external fleet
  ingestion; and
- explicit compatibility, deprecation, and unknown-schema behavior across the interoperability
  surface owned by each sub-slice.

## Canonical Authority References

V12.6 defines two separate reference families. Neither is authorization by itself.

### Approval reference

`ApprovalReferenceEvidence` binds:

- schema and profile version;
- external authority namespace and issuer identity;
- opaque provider decision/reference identity;
- exact Ota authorization-request and decision identities;
- actor/subject evidence posture where independently supplied;
- issued, expiry, cancellation, and supersession posture when the provider supplies them;
- verifier/source identity and authority posture; and
- public disclosure class.

An approval reference cannot widen semantic scope, substitute for a signed authorization decision,
or survive cancellation, expiry, issuer substitution, request substitution, or verifier changes
without re-verification.

### Waiver reference

`WaiverReferenceEvidence` binds:

- external authority namespace, issuer, and opaque waiver identity;
- exact policy rule, finding, requirement, or refusal family being waived;
- exact repository, contract, selected scope, actor/environment constraints, and validity window;
- reason classification and optional non-authoritative display text;
- source/verifier identity, authority posture, revocation/supersession posture, and disclosure
  class; and
- the Core decision that accepted or rejected the reference.

A waiver can apply only where the owning OSS policy surface explicitly defines a waivable result.
It cannot waive structural invalidity, missing execution truth, unsupported enforcement,
cryptographic failure, stale evidence, secret-delivery ambiguity, effect incompleteness, or another
hard refusal unless that surface separately and explicitly defines safe waiver semantics.

Repositories, callers, candidates, environment variables, and CLI prose cannot self-issue approval
or waiver authority. Unknown, duplicate, conflicting, expired, revoked, broader, or unverifiable
references refuse or remain non-authoritative according to the owning decision profile.

## Organization-Policy Authority Evidence

One `PolicyAuthorityEvidence` envelope binds:

- normalized policy content identity and schema/profile version;
- policy source kind and immutable source identity;
- authority namespace and posture;
- verifier/trust-root identity when independently verified;
- repository, workspace, organization, environment, and target applicability bounds;
- resolution precedence and every competing source considered;
- fetch/load and freshness posture without placing volatile timestamps in policy semantic identity;
- exact evaluator profile and decision identities; and
- disclosure class and re-derivation posture.

Caller-selected, repository-controlled, workspace-controlled, administrator-controlled, and
provider-attested sources remain distinct. Identical policy bytes from different authority
postures cannot become the same decision evidence. A hosted distributor cannot relabel a weaker
source as independently administered.

## Verifier History And Rotation Evidence

OSS defines verifier history; it does not rotate keys.

One versioned history record binds:

- authority namespace, verifier-set identity, key IDs, algorithms, and public verifier identities;
- activation, retirement, bounded overlap, revocation, and supersession statements;
- issuer and source-verifier identity for the history record;
- predecessor/successor record identities and monotonic sequence where supported;
- affected protocol/profile versions; and
- disclosure and protected-retention posture.

Private keys, credentials, recovery material, provider secrets, and rotation commands never enter
the record. Historical verification uses the verifier set valid for the signed event and rejects
backdated activation, overlap widening, sequence rollback, key-ID reuse with different public
identity, history truncation, and verifier-source substitution.

When no independently verifiable rotation history exists, evidence states the bounded static
binding posture already established by the owning carrier. It cannot claim managed rotation.

## Portable Evidence Export

An export is a deterministic manifest over existing evidence; it is not a new receipt, archive,
signature, decision, or authority source.

Core defines one versioned `EvidenceExportProfile` for every exportable artifact kind and schema
version. The profile classifies every field and byte range as public-safe, protected, secret, or
unsupported for export and binds that classification, projection rules, verifier requirements,
and profile identity. Exporters and Enterprise consumers cannot supply or relax this profile.

Only an artifact whose Core-owned profile verifies the complete original bytes as public-safe may
include those original bytes. Otherwise Core must either:

- produce a separately typed, domain-separated `RedactedEvidenceProjection` that binds the source
  relationship only to the extent permitted by the export profile, plus the export-profile
  identity, every omitted or transformed dimension, and the resulting loss of re-verification
  capability; or
- refuse export when no registered redaction projection can preserve an honest bounded meaning.

A redacted projection is not the original artifact, cannot retain its artifact identity, and
cannot satisfy any admission, receipt, archive, or historical-verification requirement that needs
the omitted bytes. A disclosure label alone never makes protected original bytes safe.

Every redacted projection carries `source_linkage_posture: public | protected | omitted`:

- `public` may include only the source kind, schema, semantic identity, or byte digest fields that
  the Core-owned export profile separately marks public-safe;
- `protected` retains exact source linkage only in protected local evidence outside the public
  export, while the exported projection carries the linkage-profile identity and explicit loss of
  offline source reconciliation; and
- `omitted` exports no source identity, digest, or correlatable substitute and states that source
  correspondence cannot be independently verified.

Low-entropy, secret-bearing, protected, or otherwise correlatable source identities and digests
must never enter the public projection merely to preserve linkage. Core refuses rather than emit a
confirmation or correlation oracle.

`EvidenceExportManifest` binds:

- manifest schema/profile and export identity;
- originating Ota build/capability identity;
- repository and contract identities;
- ordered included artifact descriptors with kind, schema, representation
  (`original_public_safe | redacted_projection`), export-profile identity, semantic identity, byte
  digest, disclosure class, size, and required/optional posture;
- source-linkage posture for every redacted projection, plus source artifact kind, schema,
  semantic identity, or byte digest only when each field is independently public-safe under the
  selected export profile;
- every omitted protected dimension and its re-verification consequence;
- source archive/history posture and selection rule;
- compatibility requirements for readers; and
- manifest integrity/signature posture.

Exports are create-new, deterministic for the same selected artifact set and export-profile set,
non-secret, and safe to verify offline. They preserve original artifact bytes only for verified
public-safe profiles; otherwise they carry a registered redacted projection or refuse. Reordering,
replacement, truncation, duplicate identity, path traversal, symlink/hardlink substitution,
archive downgrade, export-profile substitution, redaction omission, disclosure upgrade, or hidden
required-artifact removal invalidates the export.

An unsigned export proves content integrity only after recomputation. A signed export proves only
the authority and claims of its registered signer profile. Packaging, transport, object storage,
encryption at rest, retention schedules, and deletion are operator or Enterprise concerns.

## Canonical Repository Posture Report

Every report uses one versioned, Core-owned `RepositoryReportProfile`. The profile classifies every
report field, identity, digest, finding/decision reference, and drill-down target independently as
`public | protected | omitted`. Repository input, policy, adapters, external consumers, and
Enterprise may select only a Core-registered profile permitted for the report destination; they
cannot relax or relabel its disclosure decisions.

Only fields classified `public` enter a fleet-ingestion report. `protected` fields remain in their
protected local evidence and are not replaced with stable hashes, pseudonyms, or reusable aliases.
`omitted` fields carry only a non-correlatable omission category, count, and bounded verification
loss where those values are themselves public-safe. The report identity derives from the exact
emitted public fields, report-profile identity, and public omission posture; it never incorporates
protected or omitted source values as a confirmation or correlation oracle.

A fleet-ingestion profile requires its repository-scope identity, report-profile identity, and
report identity to be public-safe. If any required scope identity is protected or omitted, Core
refuses that fleet report rather than emit an unscoped or correlatable substitute. A separate local
protected report profile may retain protected fields, but its output is not fleet-ingestion input.

One read-only `RepositoryPostureReport` provides a bounded ingestion surface containing:

- public-safe repository, contract, Ota build, report-profile, and report identities;
- selected snapshot/time basis and freshness posture;
- validation, Doctor/readiness, policy, effect, secret-delivery, crossing, proof, receipt/archive,
  and candidate posture only for supported evaluated families;
- latest valid evidence identities only when each identity is public-safe under the selected report
  profile, otherwise explicit non-correlatable omission posture alongside
  invalid/unknown/unavailable counts;
- coverage and uncovered-material-behavior inventory;
- authority and disclosure posture for every summarized category; and
- ordered stable finding/decision references for drill-down only when each reference and target is
  independently public-safe; otherwise an explicit loss-of-drill-down posture.

The report is a repository snapshot, not fleet truth. It cannot claim organization compliance,
human approval, provider health, universal execution coverage, or current state after its snapshot
basis. Missing optional subsystems remain `not_evaluated` or `unsupported`, not passing.

Fleet systems aggregate immutable reports and preserve each report's scope and freshness. They may
compute views, but cannot rewrite Core verdicts or erase unknown and contradicted evidence.
Possession of protected local evidence does not authorize a fleet consumer to ingest its protected
identities or references.

## Compatibility Contract

Every interoperability artifact has:

- a stable kind and explicit schema/profile version;
- domain-separated semantic identity with self-fields excluded;
- closed required fields and fail-closed unknown-enum handling;
- documented additive versus breaking evolution rules;
- minimum reader/capability requirements where needed;
- archive/export downgrade and cross-profile substitution tests;
- stable refusal codes and JSON shape for unsupported versions; and
- a retention rule for historical readers or an explicit unsupported boundary.

Additive optional fields cannot change the meaning of existing fields. A semantic change, stronger
claim, new authority source, or new required verification dimension requires a new schema/profile
branch. Readers never reinterpret unknown future evidence as legacy success.

Public Rust protocol types, JSON schemas, canonical identity functions, and verification fixtures
remain OSS. Network APIs and SDKs may wrap these artifacts but cannot become the sole specification.

## Enterprise Ownership

Enterprise may own:

- approval and waiver routing, queues, escalation, expiry, cancellation, and reviewer UX;
- centrally administered policy authoring, protected distribution, rollout, exceptions, and drift;
- provider enrollment, verifier distribution, key-rotation execution, and operational recovery;
- organization identity, SSO, RBAC, service accounts, and administrative audit;
- centralized evidence ingestion, encryption, retention, legal hold, search, and deletion;
- fleet aggregation, dashboards, alerting, integrations, and management APIs; and
- billing, tenancy, support, and hosted operations.

Enterprise must consume OSS identities and verification results. It cannot define a parallel grant,
waiver, policy, verifier, receipt, archive, effect, secret, crossing, or posture taxonomy.

## Implementation Order

Each sub-slice repeats the schema, compatibility, propagation, pressure, and independent-review
gates before the next sub-slice activates.

1. `portable_export`: inventory shipped identities, define Core-owned export profiles and redacted
   projections, implement deterministic manifests and an offline verifier, and pressure secret and
   protected-dimension refusal.
2. `repository_reporting`: define the Core-owned `RepositoryReportProfile`, derive the canonical
   report only from verified Core results and public-safe fields, add bounded import/consumer
   verification, and pressure multi-repository aggregation without false-green or correlation
   leakage.
3. `authority_history`: normalize policy-authority evidence, then define verifier history and
   rotation events using public verifier material only.
4. `authority_references`: define approval and waiver references without wiring hosted workflows,
   and prove they cannot become authorization outside the owning Core decision path.
5. For each sub-slice, publish schemas, fixtures, command/JSON references, compatibility guidance,
   one canonical Example where authoring applies, Skill guidance, Site reference, Learn material,
   and changelog before closure.

No Enterprise service implementation is required or permitted to substitute for any sub-slice step
above. Any reference consumer assigned `implementation_lifecycle: release_enabled` must satisfy
[OSS Adapter and Profile Conformance](../adapter-profile-conformance/plan.md); parsing one fixture
or displaying one dashboard is not conformance.

## Acceptance Bar

Each bullet applies only when its owning sub-slice activates. A formally deferred sub-slice must
remove the corresponding positive claim rather than satisfy it with placeholders.

- approval and waiver references bind exact authority, decision/rule, repository, contract, scope,
  validity, source, verifier, and disclosure posture;
- references cannot authorize execution without the owning Core decision path;
- unstructured legacy approval references remain readable under their original schema but cannot be
  upgraded into verified approval-reference evidence;
- policy bytes loaded from different source/authority postures remain distinguishable;
- verifier history rejects sequence rollback, overlap widening, backdating, truncation, key-ID
  reuse, and source substitution;
- exports are byte-stable for one selected artifact set and export-profile set and reject omission,
  reordering, duplication, substitution, downgrade, traversal, aliasing, export-profile
  substitution, redaction omission, and disclosure escalation;
- original artifact bytes are exported only under a matching Core-owned public-safe profile;
  protected or secret-bearing artifacts produce a distinctly identified registered redacted
  projection or refuse;
- redacted projections expose source identities or digests only when independently public-safe;
  protected or omitted linkage records the resulting loss of source reconciliation and cannot
  satisfy verification that requires those fields;
- low-entropy or secret-derived source identity and digest confirmation attempts never enter a
  public projection or manifest;
- private keys, credentials, secret values, protected provider references, reusable leases, and
  raw authority material never enter public exports or posture reports;
- every report identity, digest, reference, and drill-down target is classified by the exact
  Core-owned report profile; protected values are omitted rather than hashed or pseudonymized;
- report-profile substitution, disclosure escalation, protected-reference retention, and hidden
  protected-value correlation through report identity invalidate verification;
- a fleet report whose repository scope is protected, omitted, or unverifiable refuses rather than
  emitting an unscoped report or stable substitute;
- repository reports preserve `unknown`, `contradicted`, `not_evaluated`, unsupported, invalid,
  stale, and uncovered states without false-green aggregation;
- imported reports and exports cannot satisfy current execution, approval, policy, crossing,
  effect, secret-delivery, or carrier admission;
- unknown future schemas fail closed without invalidating independently readable historical
  artifacts;
- OSS and an independent consumer produce the same identity and verification result for canonical
  fixtures; and
- Enterprise cannot alter a Core verdict while retaining the same report/export identity.

## Pressure Bar

Each activated sub-slice has its own pressure matrix. The combined interoperability pressure uses
at least two independent consumers once both export and reporting are active:

1. an OSS offline verifier operating only on a portable export; and
2. a separate reference fleet-ingestion fixture that aggregates multiple repository reports while
   preserving repository scope, freshness, disclosure, and unknown states.

The applicable sub-slice matrices include valid and invalid profile selection; protected and
secret-bearing original-byte export attempts; low-entropy source-hash confirmation controls;
redacted-projection and linkage-posture substitution; archive and export stripping; malformed and
future schemas; mixed historical versions; secret-leak canaries; report-profile substitution;
protected identity/reference retention; low-entropy report-correlation controls; disclosure
downgrade/upgrade; and stale reports. Authority-history pressure additionally covers
policy-source substitution and verifier-history rollback. Authority-reference pressure additionally
covers expired, revoked, stale, broader-scope, wrong-issuer, wrong-verifier, duplicate, and
conflicting approval/waiver references plus attempted imported-evidence admission.

Before `repository_reporting` closes, at least one real multi-repository design-partner or operator
case must consume immutable reports without granting the consumer write or authority access.
Retained artifacts bind exact Core, Protocol, schemas, verifier, repositories, and consumer
revisions. A dashboard screenshot or a green parser test is not interoperability proof.

## Non-Goals

- hosted approval or waiver operation;
- organization accounts, SSO, RBAC, billing, or management UX;
- central policy hosting or distribution;
- private-key custody or rotation execution;
- centralized retention infrastructure or fleet dashboards;
- making imported evidence current authority;
- defining provider-specific management APIs; or
- promising that all prior Ota JSON is a permanent Enterprise API.

## Definition Of Done

Each V12.6 sub-slice closes only when its own schema, identities, compatibility, offline or Core
verification, consumer conformance where applicable, propagation, pressure, and independent
security/privacy review pass. The umbrella closes only when all four sub-slices are complete or
formally deferred. A successful Enterprise dashboard, one exported fixture, or a schema-only
implementation is not evidence of any sub-slice or V12.6 completion. Enterprise implementation
may begin only against the OSS boundaries that actually closed; its existence is not part of any
V12.6 closure.
