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

# Broker Crossing Authority Operations

Use this guide for the bounded V11.7 Unix launcher-session `authority_broker` carrier. It gives
governed `ota run` and `ota up` one independently authorized, atomically consumed lease for one
exact semantic work unit. The repository declares only:

```yaml
governance:
  crossing_authority:
    authority_id: platform-release-authority
```

The stable public reference is
[Broker Crossing Authority](https://ota.run/docs/reference/broker-crossing-authority).

## Ownership and fixed source

| Owner | Owns | Must not expose |
| --- | --- | --- |
| Administrator | `/etc/ota/crossing-brokers.json`, verifier keys, accepted issuer/audience, timing bounds | Writable trust state to the repository or job |
| Launcher | Authenticated broker transport, provider/launcher attestation, one connected Unix session | Broker credentials, metadata endpoints, or a reusable token to Ota or task code |
| Broker | Approval decision, prepared one-use lease, atomic consume state, revocation and revision truth | A lease reusable across work units |
| Ota | Frozen scope, nonce commitment, durable pending transaction, verification, receipt/archive | Authority issuance or caller-authored approval |
| Repository | Contract lane and optional non-secret authority label | Descriptor number, verifier keys, broker origin, or lease identity |

Linux reads broker bindings only from `/etc/ota/crossing-brokers.json`. Repository content,
`OTA_POLICY`, environment variables, workflow fields, and CLI flags cannot redirect that path.
The store, file, and every canonical parent must pass the same protected regular-file checks used
by the prebound carrier. Duplicate authority IDs, duplicate binding identities, symlinks, unknown
fields, malformed records, or writable parents fail closed.

The initial carrier is Unix-only. Other platforms refuse before launcher contact or selected work.

## Protected binding

The store contains one or more versioned bindings. Each binding identifies:

- the contract-visible `authority_id` and canonical binding digest;
- expected broker origin, server name, protocol version, transport trust identity, and credential
  source identity;
- the fixed inherited Unix descriptor and launcher-session audience;
- broker and attestation verifier key sets, issuer, audience, mandatory claims, and freshness;
- phase-separated signed message domains; and
- maximum approval wait, minimum remaining freshness after approval, and maximum lease lifetime.

The protected file is a store, not a single binding:

New v1 bindings should carry `"schema_version": 1` explicitly. Unversioned v1 bindings remain
accepted only so already-issued bindings and archives preserve their original identity. A strict
protected-launcher v2 binding must use `"schema_version": 2` and replace the nested `attestation`
object with the v2 profile-bound shape. Ota does not infer v2 from nested fields.

```json
{
  "schema_version": 1,
  "bindings": [
    {
      "schema_version": 1,
      "identity": "sha256:<canonical binding digest>",
      "authority_id": "platform-release-authority",
      "broker_id": "platform-crossing-broker",
      "origin": "https://broker.example.internal",
      "server_name": "broker.example.internal",
      "protocol_version": "ota-crossing-broker/v1",
      "transport_authentication": {
        "kind": "mtls",
        "trust_bundle_identity": "sha256:<broker trust bundle>",
        "credential_source_identity": "launcher:workload-session/v1"
      },
      "credential_delivery": {
        "kind": "launcher_session_fd",
        "descriptor": 3,
        "session_audience": "ota-crossing-broker"
      },
      "broker_verifiers": [
        { "key_id": "broker-2026-01", "algorithm": "ed25519", "public_key": "..." }
      ],
      "attestation": {
        "issuer": "runner-launcher",
        "audience": "ota-crossing-broker",
        "trust_bundle_identity": "sha256:<launcher trust bundle>",
        "verifiers": [
          { "key_id": "launcher-2026-01", "algorithm": "ed25519", "public_key": "..." }
        ],
        "maximum_age_seconds": 180,
        "maximum_clock_skew_seconds": 5,
        "key_rotation_overlap_seconds": 300,
        "mandatory_protocol_claims": [
          "binding_identity",
          "challenge_nonce_commitment",
          "invocation_id",
          "work_unit_identity",
          "semantic_scope_identity",
          "runner_principal",
          "channel_delivery",
          "authenticated_origin",
          "authority_mounts"
        ],
        "required_administrator_claims": []
      },
      "message_domains": {
        "challenge_request": "ota-crossing-broker/challenge-request/v1",
        "attestation_response": "ota-crossing-broker/attestation-response/v1",
        "authorization_request": "ota-crossing-broker/authorization-request/v1",
        "authorization_decision": "ota-crossing-broker/authorization-decision/v1",
        "lease_issuance": "ota-crossing-broker/lease-issuance/v1",
        "lease_consume": "ota-crossing-broker/lease-consume/v1",
        "lease_consume_response": "ota-crossing-broker/lease-consume-response/v1",
        "lease_consumption_query": "ota-crossing-broker/lease-consumption-query/v1",
        "lease_consumption_status": "ota-crossing-broker/lease-consumption-status/v1"
      },
      "maximum_approval_wait_seconds": 120,
      "minimum_post_approval_freshness_seconds": 30,
      "maximum_lease_seconds": 300
    }
  ]
}
```

Live bindings require all nine domains shown above. Receipt history also accepts the exact earlier
seven-domain broker snapshot that predates consumption recovery. Ota preserves that snapshot's
original identity and compares it with a re-derived legacy projection of the protected current
binding; it never inserts recovery fields before verifying archived identity.

The example public keys are placeholders. The administrator derives `identity` from the exact
canonical binding using Ota's domain-separated JCS identity rules; hand-written identities or
repository-generated keys are not valid provisioning.

The descriptor exists when the launcher starts Ota, but begins as untrusted IPC. Ota verifies that
it is a connected Unix stream, sets `FD_CLOEXEC`, sends a fresh challenge, and treats it as
authority-capable only after verifying the signed challenge-bound attestation. Selected tasks and
all child processes receive neither the descriptor nor broker credentials.

Receipts and archives retain the signed protocol payloads required for later verification. Those
payloads must be public-safe by construction: `invocation_id`, `runner_principal`, and
`authority_mounts` are bounded non-secret labels and cannot contain filesystem paths, whitespace,
credentials, tokens, or free-form user text. Ota never archives the raw nonce, descriptor, broker
transport credential, or secret provider material.

## Invocation flow

```text
Repository contract selects authority_id
        -> Ota derives exact semantic work unit
        -> Ota selects exactly one protected broker binding
        -> Ota challenges the launcher over the fixed descriptor
        -> Ota verifies launcher attestation and signed authorization
        -> Ota receives a prepared one-use lease
        -> Ota durably creates the pending crossing transaction
        -> broker atomically consumes the lease for that transaction
        -> selected run/up work starts
        -> Ota finalizes and archives the terminal transaction
```

Routine use does not require a copied lease ID:

```sh
ota run publish
ota up --workflow release
```

When `--grant` is supplied for this carrier, it may only equal the configured non-secret
`authority_id`. It confirms or disambiguates selection; it never supplies authorization or a
lease. Zero matches, multiple matches, or a different label refuse.

Dry-run resolves the exact scope and binding but deliberately does not contact the launcher:

```sh
ota run publish --dry-run --json
```

It reports `crossing_grant_admission.decision: requires_live_authorization` with
`authority_carrier: authority_broker`. It creates no transaction, consumes no lease, and does not
claim admission.

## Admission and refusal

Real execution freezes the contract identity, ordered closure graph, hooks, target platform,
execution/effect selection, crossing family/classification, and bounded actor mode before broker
contact. Ota then verifies every signed phase and refuses before selected work for:

- missing, ambiguous, malformed, or unprotected binding state;
- missing/wrong descriptor, non-socket descriptor, framing error, or inheritable channel;
- wrong nonce, work unit, scope, contract, issuer, audience, origin, principal, or message phase;
- stale attestation, denied/ambiguous approval, timeout, interruption, or changed scope;
- expired/revoked lease, stale broker revision, failed consumption, or replay/double spend; and
- any transaction journal that was not durably pending before consumption.

If consumption has an ambiguous transport outcome, Ota refuses rather than starting work. A later
approval after local cancellation cannot become executable.

Before sending a consume request, Ota durably records the exact signed admission, lease,
consume-request identity, work-unit identity, and pending transaction identity. A later invocation
with the same semantic scope first obtains fresh launcher attestation and queries the broker for
that exact intent. Verified `consumed`, `not_consumed`, and `unknown` statuses all close the abandoned transaction as
`incomplete`; recovery never resumes its selected work. A new execution requires a fresh
authorization and lease. If a signed status was already journaled before interruption, Ota
re-verifies it and completes local finalization without querying again.

## Receipts and archives

Successful evidence uses transaction schema v2 with `authority_carrier: authority_broker` and
binds:

- protected binding and broker admission identities;
- launcher attestation, frozen nonce commitment, work-unit identity, and runner principal claim;
- signed authorization decision and optional approval reference;
- prepared lease, consume request/response, broker revision, and consume time;
- exact semantic scope and contract identity, including selected workflow instance, ordered
  prerequisite-instance closure, normalized workflow readiness timeout, and runner-derived
  closure/effect/resource breadth; and
- pending and terminal transaction identities, state, outcome, and finalization time.

Breadth carries counts, effect categories, and hashed resource identities rather than raw resource
values. The archived binding is a public verification snapshot and omits the protected live
launcher descriptor. Signed protocol payloads retained for archive re-verification accept only bounded
public-safe invocation, principal, and authority-mount labels. Raw nonce values, descriptors,
credentials, filesystem paths, and secret provider material are excluded from public evidence.

`ota receipt --history --json` re-verifies these fields against the protected binding and archived
contract snapshot. Missing consumption, carrier substitution, scope drift, signature mismatch,
replay, incomplete finalization, or altered identities invalidate the archive.

`authority_separation_posture: launcher_attested_one_use` is the immutable v1 posture: Ota verified
the challenge-bound launcher session and one-use broker consumption, but no structured runtime
profile. V2 bindings require one exact protocol-published protected-launcher profile, ordered
required observations, content-addressed launcher and configuration identities, and a separate
attestor key authority. When every required observation verifies, receipts use
`protected_launcher_attested_one_use`. That posture proves only the signed profile; it does not
independently prove provider, host, namespace, or privilege-separation facts outside those exact
observations. V1 and v2 payload, response-domain, identity-domain, and binding branches are
mutually exclusive, and archive verification preserves the original branch without injecting
defaults.

## Current limits

- Governed `ota run`, `ota up`, `ota proof runtime`, and `ota proof lifecycle` consume broker
  authority. Each proof command owns one transaction across its complete
  helper/service/assertion and cleanup set.
- The first adapter uses a launcher-supplied Unix stream; direct Ota-to-broker credentials are not
  supported.
- Immutable Linux/x64 PID 1 pressure proves the protected-launcher profiles, production client and
  history source, independently administered positive execution, and administrator-driven
  reboot/fault recovery. Those runs satisfy V11.7 through the hardened-launcher alternative.
  Provider-attested boundary claims remain optional stronger hardening rather than a completion
  gate. Core regression proof covers profile downgrade and mutation refusal, consume-intent
  durability, exact re-query, consumed/not-consumed/unknown outcomes, and restart after a durably
  recorded status.
- Ota does not ship or operate the approval broker, provisioner, signing keys, or launcher.
