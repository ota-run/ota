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

# V12.3: Provider-Attested Authority Carrier

Status: planned and inactive. Provider support, provider separation, and
`provider_attested_one_use` remain unimplemented and unproved.

## Activation Gates

V12.3 may be activated only after:

- V12.2 completes or is formally deferred;
- one named provider exposes a documented, cryptographically verifiable workload-attestation
  mechanism with immutable test infrastructure available to Ota;
- at least one real operator or design partner documents a current need for provider-owned
  separation beyond V11.7's independently administered hardened launcher and commits to pressure
  the named profile in its actual deployment posture;
- the exact provider claims, trust roots, verifier lifecycle, outage behavior, and replay boundary
  receive independent threat-model review; and
- activation names one provider/profile only. Generic multi-cloud claims are forbidden.

Provider availability, an interesting public repository, or a local attestation prototype is not
demand evidence. Without the operator/design-partner requirement and immutable provider pressure,
V12.3 remains inactive and should be formally deferred rather than implemented speculatively.

Planned follow-on: [V12.4 macOS Protected Authority Carrier](../v12.4/plan.md) remains inactive
until V12.3 completes or is formally deferred.

## Why This Is V12.3

V11.7 proves a Linux/systemd carrier through an independently administered hardened launcher. It
deliberately reserves `provider_attested_one_use` for a stronger profile in which provider-owned
evidence establishes the workload and execution boundary. V12.3 implements that reserved branch;
it does not reopen or downgrade the completed Linux carrier.

## Product Boundary

Provider attestation authenticates bounded provider-observed workload facts. It does not by itself
approve work, issue an Ota crossing grant, prove application correctness, establish human identity,
or demonstrate host-global isolation.

The provider attestor and the Ota authorization authority remain separate roles:

- provider attestation establishes eligible workload/execution identity;
- Ota Core independently verifies the attestation and exact selected semantic scope;
- the authority broker decides and issues one-use authority; and
- Core consumes the lease and records execution, cleanup, receipt, and archive evidence.

A provider API response, metadata label, environment variable, repository token, CI claim, or
unsigned instance description can never satisfy this profile.

## Provider Profile

Each provider profile is separately versioned and must define:

- issuer, audience, subject, tenant/account/project, region, and workload identity semantics;
- attestation mechanism, signature algorithm, trust-root source, verifier-set identity, and key
  rotation/overlap posture;
- freshness, nonce/challenge binding, clock/skew, expiry, and replay semantics;
- instance, VM, container, job, image, boot, runtime, and process claims actually supplied;
- which claims are provider-authenticated, operator-configured, repository-controlled, inferred,
  unavailable, or explicitly not proved;
- provider endpoint authentication, availability, retry, timeout, rate-limit, and ambiguity
  behavior;
- revocation and workload-termination observations where the provider exposes them; and
- exact public disclosure and protected-evidence posture.

Unknown claims remain absent or `unknown`; profiles cannot fill them with defaults. Two providers
or materially different attestation products cannot share a profile merely because field names
look similar.

## Canonical Identities

Domain-separated identities bind:

- provider profile and verifier binding;
- provider challenge and nonce commitment;
- canonical attestation claims and signed envelope;
- Ota invocation, selected semantic scope, work unit, contract, effects, and crossing requirement;
- attested workload, image/runtime posture where supplied, and provider authority namespace;
- authorization request, decision, lease, consumption, transaction, and recovery; and
- public/protected evidence disclosure posture.

Every signed response projects back to the exact canonical request and claims identity. Provider-
owned fields such as issuance time, expiry, key ID, signature, and provider event identity are
verified separately. Request hashes are integrity, not authentication.

## Trust And Replay Model

- every authorization attempt uses a fresh challenge bound to one invocation and work unit;
- provider evidence must be fresh through any approval wait and lease-consumption margin;
- attestation replay across tenant, repository, workload, invocation, scope, carrier profile, or
  verifier binding refuses;
- duplicate/ambiguous provider responses never produce two executable authority outcomes;
- verifier rotation changes binding identity and supports only explicitly bounded overlap;
- provider unavailability, timeout, stale evidence, unknown revocation, or incomplete claims refuse
  before selected work; and
- recovery re-verifies current protected transaction truth and never resumes abandoned work based
  only on a historical provider assertion.

## Ownership

`authority-protocol` owns provider-neutral wire domains, identity functions, and profile registry
interfaces. Core owns independent verification, semantic-scope reconciliation, transaction state,
lease consumption, receipts, and archive verification. Provider adapters may live with
`authority-launcher` or another reviewed first-party carrier package, but must run outside
repository-controlled execution and cannot own Core's final verification decision.

Every provider profile is governed by
[OSS Adapter and Profile Conformance](../adapter-profile-conformance/plan.md). Release artifacts,
installation, upgrade, rollback, compatibility, and support lifecycle are governed by
[Authority Distribution Lifecycle](../authority-distribution-lifecycle/plan.md). Neither supporting
plan makes an unproved provider profile supported.

Enterprise may administer provider enrollment, verifier distribution, retention, and organization
policy. Those management services are not required for the OSS protocol and verifier to remain
reviewable.

## Implementation Order

1. Select one provider and freeze its exact attestation/profile threat model.
2. Add protocol domains, canonical projections, verifier binding, and adversarial fixtures.
3. Implement a protected provider adapter with no repository-selected endpoint, credential, or
   trust root.
4. Integrate fresh provider evidence into existing V11.7 challenge, decision, lease, and recovery
   state machines.
5. Extend receipts, protected history, archive re-verification, and disclosure schemas.
6. Add outage, timeout, replay, rotation, ambiguity, stale-workload, and substitution controls.
7. Propagate bounded operator and public documentation only after behavior exists.
8. Run immutable provider-hosted pressure and independent security review.

## Acceptance And Pressure Bar

- only the named provider/profile may emit `provider_attested_one_use`;
- repository, caller, launcher, broker, and provider roles cannot impersonate one another;
- wrong tenant, project/account, region, workload, image/runtime claim, audience, issuer, key,
  challenge, invocation, scope, work unit, or contract refuses;
- omitted provider claims never inherit a stronger default posture;
- key rotation, overlap, revocation, stale evidence, provider outage, timeout, duplicate response,
  delayed response, and ambiguous response are pressure-tested;
- approval waits cannot outlive attestation freshness;
- one-use lease replay and lost-ack recovery preserve V11.7 semantics;
- provider-attested receipts and archives reject downgrade to hardened-launcher or legacy evidence,
  while historical carriers remain valid only under their original branch;
- selected execution starts only after provider verification, authority decision, and atomic lease
  consumption; and
- terminal evidence proves exact cleanup or reports bounded uncertainty without claiming removal.

Immutable pressure must run inside the named provider environment against pinned Core, Protocol,
carrier, image/configuration, and verifier revisions. Artifacts retain public verifier identities,
provider profile identity, bounded signed evidence, transaction/archive evidence, cleanup state,
and zero repository-secret material. A local emulator is a development control, not completion
evidence.

## Non-Goals

- generic multi-cloud attestation;
- hosted approval UI, policy management, billing, or fleet dashboards;
- treating OIDC authentication alone as workload attestation;
- proving provider infrastructure free of compromise;
- replacing the Linux hardened-launcher carrier;
- macOS or Windows carrier implementation; or
- provider-backed secret delivery, which remains V12.1 adapter work.

## Definition Of Done

V12.3 completes only for the named provider/profile after immutable provider-hosted pressure,
independent verification, recovery, cleanup, receipt/archive re-derivation, compatibility, and
bounded public documentation pass. It does not establish support for another provider.
