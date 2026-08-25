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

# Cross-Cutting Plan: Authority Distribution Lifecycle

Status: planned and inactive. Current authority components remain source-distributed at immutable
reviewed revisions unless their repository documentation states otherwise. This plan does not
publish, install, upgrade, sign, support, or deprecate a release artifact.

## Purpose

V11.7 proves the Linux/systemd authority carrier from reviewed immutable source. Later plans add
provider-attested, macOS, and Windows carriers. Organizations need more than source availability:
they need reproducible artifacts, compatible versions, protected installation, safe upgrade and
rollback, security response, and durable state handling.

Distribution is part of the trust boundary. An installer, package, unit file, socket, credential
binding, upgrade script, or rollback path that is weaker than the reviewed carrier can invalidate
the authority model even when the Rust implementation is correct.

This plan defines the OSS operational lifecycle for:

- `authority-protocol` source/crate releases and compatibility fixtures;
- `authority-launcher` services and supporting binaries;
- protected attestor, operator client, history client, and provisioning components;
- platform packages, installation manifests, service definitions, and configuration;
- Core compatibility with Protocol, Launcher, and carrier profiles; and
- public verifier material, protected state, migration, recovery, and uninstall posture.

## Activation And Sequencing

This document is a normative acceptance standard, not an activatable implementation slice. It
cannot authorize packaging, installation, upgrade, recovery, or release work by itself. A future
version plan must explicitly name the bounded distribution work it owns, its dependencies,
implementation scope, artifacts, pressure matrix, and closure bar before any implementation begins.
No existing V12.3, V12.4, or V12.5 carrier plan implicitly owns this work.

The first future owner should mature the completed Linux/systemd carrier before promising macOS or
Windows packages, unless real adoption evidence establishes a different priority. Platform
artifacts close independently. Naming that owner is a separate roadmap decision; this standard
does not assign a version number or bypass the one-active-version rule.

Every distributed profile must also satisfy
[OSS Adapter and Profile Conformance](../adapter-profile-conformance/plan.md).

## Product Boundary

Ota distributes reviewed code and non-secret configuration templates. It does not distribute:

- organization signing keys, broker credentials, private attestor material, provider secrets, or
  reusable authority;
- pre-authorized grants, leases, approval state, or repository-specific trust decisions;
- an Ota-hosted authority service; or
- administrator policy that an operator has not reviewed and installed.

Installation creates an empty authority boundary. Administrators separately provision trust roots,
verifier sets, provider bindings, and credentials through protected channels. Example packages and
test fixtures deliberately contain no usable production authority.

## Version And Compatibility Model

Human compatibility uses semantic versions. Trust and evidence use immutable identities.

Each release records:

- component name and semantic version;
- source commit and source-tree identity;
- build recipe/toolchain identity;
- artifact digest, package identity, SBOM identity, provenance identity, and signature identity;
- Protocol schema/profile capability set;
- compatible Core, Protocol, Launcher, client, attestor, and carrier-profile ranges;
- supported OS, architecture, package format, and service-manager posture;
- installed profile-registry snapshot identity, source/verifier identity, monotonic sequence or
  predecessor posture, and freshness/expiry rules;
- protected-state schema versions and migration support; and
- release, deprecation, and security-support status.

A semantic version is never sufficient for trust-sensitive pinning. Operators and workflows select
a reviewed version while installation evidence and runtime verification bind immutable artifact,
manifest, configuration, and profile identities.

Compatibility is explicit, not inferred from successful startup. Unsupported combinations refuse
before authority service activation or Core protocol traffic.

## Reproducible Release Set

Every supported artifact set includes:

- canonical source revision and source archive;
- reproducible build instructions and hermetic-enough pinned toolchain/dependency inputs;
- platform binaries and packages built from that source;
- installation manifest binding every binary, service/unit definition, socket, configuration,
  drop-in, verifier template, and expected protected path;
- CycloneDX or SPDX SBOM;
- build provenance/attestation with builder identity and artifact digests;
- detached artifact/package signatures with public verifier identity;
- checksums and one machine-readable release manifest;
- compatibility matrix and upgrade/downgrade notes; and
- operator verification instructions that do not require trusting a mutable web page.

Reproducibility claims require an independent rebuild comparison. A signed artifact without
reproducible source/build binding remains signed but not reproducibly verified.

## Platform Packaging

Each platform package owns its exact layout and service lifecycle.

The Linux/systemd package must bind:

- root-owned binaries, canonical units/sockets/drop-ins, fixed runtime and state paths, service
  users/groups, capabilities, namespace/network hardening, and credential sources;
- installation-manifest and profile identities used by runtime attestation;
- package-manager and portable archive posture where each is supported; and
- x86_64 first, with additional architectures supported only after native pressure.

macOS packages must bind package/signing/notarization posture, launchd definitions, designated
requirements, protected paths, and supported architectures as required by V12.4.

Windows packages must bind Authenticode/package posture, Windows Service configuration, SIDs,
ACLs, named pipes, protected ProgramData paths, and supported editions/architectures as required by
V12.5.

One platform package cannot establish another platform's support.

## Protected Installation

Provisioning verifies before its first mutation:

- administrator/root authority and acceptable installer identity;
- canonical package/release manifest and every artifact signature/digest;
- service stopped/inactive posture and no active job/execution principal processes;
- exact existing authority-state posture;
- every managed path and existing ancestor without symlink, reparse, alias, hardlink, ownership,
  permission, or writable-parent ambiguity; and
- adequate disk, filesystem, durability, and service-manager capabilities.

Writes use no-follow directory-relative operations, create-new or verified replacement semantics,
file sync, directory sync, metadata verification, and interruption-safe journals. Installation
evidence records exact preconditions, package/artifact identities, managed paths, resulting
configuration/profile identity, and terminal service posture.

Repository-controlled execution cannot invoke or race administrator provisioning.

## Upgrade State Machine

Upgrade is an explicit protected transaction:

```text
preflight
  -> quiesce
  -> protected backup/checkpoint where applicable
  -> stage and verify
  -> migrate protected schema/configuration
  -> atomic activate
  -> service and protocol health verification
  -> finalize or rollback
```

Required rules:

- refuse while selected executions, active slots, uncertain transactions, pending finalization, or
  unreconciled recovery state exist unless the exact release profile defines safe recovery;
- retain the previous verified artifact/configuration set until new health and compatibility pass;
- migrate state once under a domain-separated migration identity and durable journal;
- never reinterpret historical evidence into a newer profile;
- re-derive installation, service, verifier, and compatibility identities after activation;
- report whether activation occurred when later durability or health becomes uncertain; and
- never silently fall back to an older binary after a new process has consumed authority under a
  newer incompatible state schema.

Automatic background self-update is not part of the first release. Operators initiate a reviewed
upgrade and receive dry-run/preflight evidence before mutation. That operation installs a verified
registry snapshot alongside the compatible release. Until a newer accepted snapshot is installed,
Ota reports `installed_snapshot_only` rather than claiming awareness of later revocations.

## Rollback And Downgrade

Rollback is permitted only when:

- no authority transaction crossed an incompatible irreversible migration boundary;
- the prior artifact, configuration, verifier, profile, and state-schema identities remain present
  and verified;
- historical/current evidence remains readable without downgrade reinterpretation; and
- the rollback profile explicitly supports the current protected-state version.

Otherwise Ota refuses and retains recoverable state with operator guidance. `--force`, deleting a
journal, or restoring binaries without matching protected state is not a supported recovery path.

Security rollback to a release marked revoked by the accepted installed registry snapshot is
forbidden even if technically compatible.

## Protected State, Backup, And Recovery

State is classified before backup:

- public verifier, manifest, profile, receipt, history, and compatibility evidence;
- protected but non-secret transaction, recovery, finalization, and catalog state;
- credential references and protected provider bindings; and
- private keys, credentials, and secret material owned outside Ota distribution.

Ota may define backup/export rules only for the first two classes. It never exports private keys,
credentials, secret values, provider material, or reusable authority.

Backups are content-addressed, encrypted/transported by operator-owned tooling, versioned, and
restored only onto a compatible empty or explicitly reconciled installation. Active or uncertain
transactions cannot be snapshotted as if terminal. Restore re-verifies ownership, permissions,
installation/profile compatibility, boot/service identity, monotonic sequence, and archive
integrity before service activation.

Loss of unrecoverable credential material produces a new binding/verifier identity and explicit
historical verification posture; it is never repaired with defaults.

## Uninstall

Uninstall stops and disables services, proves child/scope/job absence, removes distributed binaries
and service definitions, and preserves protected state by default.

Destructive state removal requires a separate explicit administrator operation after recovery and
retention checks. It refuses when active/uncertain transactions, unresolved cleanup, protected
history obligations, or shared verifier dependencies exist.

Uninstall evidence names what was removed, preserved, unavailable, and not proved. Package-manager
success alone is insufficient.

## Release Channels And Support

Initial channels are:

- `candidate`: immutable review/pressure artifact, never production-supported;
- `stable`: independently reviewed, pressure-proven, documented, and supported; and
- `security`: stable-compatible urgent repair with explicit affected versions and migration notes.

There is no mutable `latest` trust selector in installation evidence. Convenience download URLs may
resolve a version, but the installer verifies the immutable release manifest before mutation.

Every stable release publishes:

- supported Core/Protocol/Launcher/profile matrix;
- minimum secure versions and known revoked versions;
- an immutable signed profile-registry snapshot or exact registry-snapshot reference, including
  source/verifier identity, monotonic predecessor posture, and freshness/expiry rules;
- deprecation and end-of-support dates;
- migration/rollback posture;
- security contact and advisory process; and
- retained reproducibility and pressure evidence.

## Security Response

The lifecycle defines:

- vulnerability intake and embargoed triage;
- affected component/profile/version identification;
- revocation and minimum-version publication;
- patched reproducible artifacts and compatibility evidence;
- emergency upgrade and rollback restrictions;
- historical evidence verification after key/artifact revocation; and
- public advisory language that distinguishes code compromise, key compromise, provider compromise,
  and unproved impact.

Publishing a revocation does not reach an offline installation immediately. Once an operator
installs and accepts the signed registry/advisory snapshot containing that revocation, new
admission refuses relative to that snapshot while historical evidence remains honest. Admissions,
receipts, and archives expose the registry identity and freshness posture they actually observed.
Advisories never silently rewrite old receipts or manifests.

## Implementation Order

This is the required order for a future version plan that adopts the standard. It is not an active
backlog and does not authorize implementation in the absence of that named owner.

1. Freeze component versioning, compatibility, release-manifest, and support semantics.
2. Produce reproducible Linux/x86_64 candidate artifacts for the completed V11.7 carrier.
3. Implement protected install preflight, manifest verification, and installation evidence.
4. Implement upgrade journal, compatibility preflight, activation, health, and rollback.
5. Add protected-state classification, backup/restore, uninstall, and recovery procedures.
6. Add SBOM, provenance, signatures, independent rebuild comparison, and advisory metadata.
7. Run fresh-install, upgrade, rollback, interruption, corruption, and security-revocation pressure.
8. Publish stable artifacts only after independent release audit.
9. Reuse the model for macOS, Windows, and provider packages only after their carrier plans pass.

## Acceptance And Pressure Bar

- source, artifacts, package, manifest, SBOM, provenance, and signatures reconcile exactly;
- independent rebuilds reproduce the claimed artifact or report the bounded difference honestly;
- wrong version, digest, signer, build provenance, unit/service definition, configuration, profile,
  path, owner, permission, or compatibility range refuses before service activation;
- fresh install, idempotent verified reinstall, supported upgrade, rollback, unsupported downgrade,
  and uninstall behave deterministically;
- interruption at every write, sync, migration, activation, health, finalization, and cleanup stage
  recovers without mixed-version service state;
- active/uncertain authority state blocks unsafe upgrade, rollback, backup, restore, and uninstall;
- restored state cannot roll back monotonic verifier, transaction, catalog, or history truth;
- installed registry snapshot identity, signer, sequence/predecessor, and freshness posture cannot
  be substituted, rolled back, or relabeled as globally current;
- packages contain no production keys, credentials, grants, leases, provider bindings, or reusable
  authority;
- stable channel artifacts have immutable retained release and native pressure evidence;
- candidate artifacts cannot be mistaken for supported stable releases; and
- Core, Protocol, Launcher, clients, attestor, packages, and profiles refuse unsupported
  combinations before authority traffic.

Initial immutable pressure uses real PID 1 systemd hosts and covers fresh installation, configured
but authority-empty startup, protected provisioning, normal operation, crash recovery, in-place
upgrade, rollback, incompatible downgrade, package corruption, interrupted installation/migration,
backup/restore, uninstall preservation, and security revocation. Revocation pressure proves the
bounded offline posture before a newer snapshot is installed, then exact refusal after its verified
installation. Repository and protected-state sentinels prove exact mutation and zero unrelated
residue.

## Non-Goals

- hosted authority or approval services;
- Enterprise policy distribution, rotation execution, retention service, or fleet management;
- bundling administrator authority or production credentials;
- unattended background self-update in the first release;
- claiming package signatures prove reproducibility or runtime isolation;
- shipping macOS/Windows artifacts before their carrier plans pass; or
- replacing immutable identities with semantic versions alone.

## Definition Of Done

This plan completes for one platform/release set only after reproducible artifacts, release
manifest, protected install, compatibility refusal, upgrade, rollback, state recovery, uninstall,
SBOM, provenance, signatures, support policy, security response, native pressure, and independent
release audit pass. Another platform or architecture remains unsupported until independently
closed.
