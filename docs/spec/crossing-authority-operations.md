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

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

       http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
-->

# Prebound Crossing Authority Operations

Use this operator guide only for the bounded V11.7 `prebound_file` carrier. It is a
provisioner-owned boundary for heavier **non-agent** execution. A repository names only an
`authority_id`; it must never create, update, or select authority files.

The stable public operator reference is
[Prebound Crossing Authority](https://ota.run/docs/reference/prebound-crossing-authority).
Use this Core specification for the exact record layout and verification rules.

This carrier is not a hosted-service approval system and does not provide the broker carrier's
one-use hardened-launcher authority. It is filesystem-guarded from Ota's current unprivileged
process. It does **not** prove that a CI job lacks `sudo`, host capabilities, or namespace control.
Do not use a GitHub-hosted runner to claim independent authority separation. The completed V11.7
slice satisfies its stronger acceptance bar through the separate independently administered
hardened-launcher carrier.

## Ownership model

| Role | Owns | Must not own |
| --- | --- | --- |
| Repository author | `governance.crossing_authority.authority_id` | Keys, signed bundles, state paths, or trust-store paths |
| Authority provisioner | Trust binding, issuer key, signed bundle, revocations, and sequence state | Repository task selection or Ota execution inputs |
| Ota runner user | Read-only admission verification and task execution | Any authority file, parent directory, or signing key |

The signing private key stays with the authority provisioner. Neither Ota nor the runner needs it.

## Linux layout

The only fixed Linux path is the trust store. The binding in that store points to the independently
managed bundle and sequence state.

| Path | Role | Required protection |
| --- | --- | --- |
| `/etc/ota/` | Fixed trust-store parent | Root-owned; not group/world writable |
| `/etc/ota/crossing-authorities.json` | Trust-store bindings | Root-owned regular file; not group/world writable |
| `/var/lib/ota/` | Example authority-state parent | Root-owned; not group/world writable |
| `/var/lib/ota/crossing-authority.json` | Signed grant bundle | Root-owned regular file; not group/world writable |
| `/var/lib/ota/crossing-authority-sequence.json` | Monotonic sequence/clock high-water state | Root-owned regular file; not group/world writable |

`0644` files under `0755` root-owned parents are a compatible read-only layout when the bundle and
state carry no secrets. A more restrictive group-readable mode is also valid if the unprivileged
runner can read every file and traverse every parent. Ota canonicalizes configured paths and opens
the resolved file without following a final symlink; the resolved file and every canonical parent
must be root-owned and not group/world writable. Repository-relative paths, non-regular resolved
files, and writable canonical parents refuse.

macOS uses `/Library/Application Support/Ota/crossing-authorities.json` as its fixed trust store.
Windows refuses `prebound_file` until Ota can verify an equivalent ACL boundary.

## Authority records

All records use `schema_version: 1`, reject unknown fields, and are canonicalized before identity
or signature verification. The provisioner must derive identities and signatures using Ota's
domain-separated RFC-8785 canonical JSON rules; do not hand-type an identity or signature.

### Fixed trust store

`/etc/ota/crossing-authorities.json` contains one binding per `authority_id`:

```json
{
  "schema_version": 1,
  "bindings": [
    {
      "identity": "sha256:<derived binding identity>",
      "authority_id": "platform-release-authority",
      "issuer_id": "release-authority",
      "key_id": "release-key-2026-01",
      "algorithm": "ed25519",
      "public_key": "<base64url-ed25519-public-key>",
      "key_fingerprint": "sha256:<public-key-sha256>",
      "bundle_path": "/var/lib/ota/crossing-authority.json",
      "sequence_state_path": "/var/lib/ota/crossing-authority-sequence.json",
      "minimum_sequence": 42,
      "max_bundle_age_seconds": 300,
      "max_clock_skew_seconds": 5,
      "clock_posture": "system_non_root",
      "allowed_contract_identities": ["sha256:<contract-identity>"]
    }
  ]
}
```

The binding identity covers every field other than its own `identity`. `public_key` is an unpadded
base64url Ed25519 verifying key. The runner refuses an unknown authority, incorrect key
fingerprint, non-absolute path, or binding identity mismatch.

### Signed grant bundle

The provisioner writes the bundle at the binding's `bundle_path`. Its payload contains
`bundle_id`, issuer/key IDs, monotonic `sequence`, RFC3339 `issued_at`, `not_before`,
`next_update`, sorted unique `grants`, and sorted unique `revocations`. The envelope contains:

```json
{
  "schema_version": 1,
  "payload": { "...": "signed canonical bundle payload" },
  "signature": {
    "algorithm": "ed25519",
    "key_id": "release-key-2026-01",
    "value": "<base64url-ed25519-signature>"
  }
}
```

Each grant binds its ID and derived identity to the exact contract identity, semantic scope
identity, boundary family, classification, `non_agent` actor posture, environment posture, action,
resource, RFC3339 validity window, and `calendar_ttl` expiry kind. A revocation names the grant ID,
RFC3339 revocation time, and optional reason. Ota verifies the signature over the domain-separated
canonical payload and refuses stale, future, expired, revoked, duplicated, or out-of-scope grants.

### Sequence state

The sequence state at `sequence_state_path` contains:

```json
{
  "authority_id": "platform-release-authority",
  "highest_sequence": 42,
  "last_observed_at": "2026-08-01T12:00:00Z"
}
```

It must agree with the signed bundle sequence. The provisioner updates bundle and sequence state as
one controlled authority operation; Ota refuses inconsistent or rolled-back state rather than
trying to repair it.

## Inspect the fixed boundary

Run the diagnostic before grant pressure:

```bash
ota authority inspect --json
```

The command reads the fixed store and every binding through the same protected-file verifier used
by admission. It does not select a grant, create a crossing transaction, write authority/high-water
state, create a receipt/archive, or execute repository work. A matched report is bounded to
`authority_separation_posture: current_process_filesystem_guarded`.

Each observation carries `required`, `status`, `method`, and a stable redacted `reason`. Only
`profile.verdict: matched_with_unknowns` exits zero. Informational unknowns include passwordless
sudo, namespace control, alternative container endpoints, provider metadata credentials, and
broader administrative escalation when Ota cannot observe them without crossing the boundary.
Therefore this diagnostic supplements, but never replaces, administrator or provider evidence for
the runner boundary.

## Provision and verify

1. Provision the trust-store binding, bundle, and sequence state as a system administrator before
   the Ota runner starts. Use root-owned, non-writable parents and regular files.
2. Run Ota as an unprivileged runner user. `prebound_file` refuses when Ota itself runs as root.
3. Let Ota derive the exact selected scope with a refusal or dry-run; an independent issuer uses
   that machine-readable scope/contract evidence to create the signed grant.
4. The provisioner publishes the signed bundle and matching high-water sequence state. Ota never
   writes either file.
5. The runner invokes the selected lane explicitly, for example:

   ```bash
   ota run publish --grant approved-publish
   ```

6. Preserve the receipt/archive. It binds the selected grant, authority binding, bundle, scope,
   and terminal crossing transaction. A successful run never turns the crossing record into a
   reusable grant.

Use `ota run publish --dry-run --grant approved-publish --json` to inspect an admissible scope
without consuming a crossing transaction. A matching grant still cannot override `--agent` safety.

## Prepare a dedicated GitHub Actions VPS runner

Use this sequence only for controlled pressure or operator validation. It prepares a Linux/x86_64
host for GitHub Actions; GitHub schedules the job but is not the authority source. Do not attach
this runner to untrusted pull-request workflows or install it on an administrator's daily-use
machine.

1. Create a dedicated Linux/x86_64 VPS and restrict SSH to the administrator's known network.
   The host needs a persistent disk because the provisioner-owned sequence state must survive jobs.
2. As the system administrator, create the non-root job principal and protected authority parents:

   ```sh
   sudo useradd --create-home --shell /bin/bash ota-runner
   sudo install -d -o root -g root -m 0755 /etc/ota /var/lib/ota
   ```

   Do not add `ota-runner` to `sudo`, Docker, or other host-control groups. A matching local
   inspection remains only `current_process_filesystem_guarded`; it does not prove every possible
   escalation path is absent.
3. Build the exact reviewed Ota commit from a clean Core checkout and install the binary at an
   administrator-controlled absolute path. Publish a root-owned build manifest outside the
   repository that binds the full source revision and the binary SHA-256, then have the workflow
   compare that manifest with the exact executable it invokes. Do not rely on inherited job `PATH`
   ordering for a protected-runner authority lane. Retain `ota --version --json` as corroborating
   source-build and clean-tree evidence; its embedded commit is abbreviated and must not be treated
   as the full revision proof.
   Do not substitute a release binary, a branch, or a repository-installed binary for this check.
4. In GitHub repository settings, open **Actions > Runners > New self-hosted runner**, select
   Linux/x64, and run GitHub's time-limited registration commands as `ota-runner`. Register only
   the reviewed repository with `self-hosted`, `linux`, and one scenario label such as
   `ota-crossing-live`. Install the runner service only after the Ota binary and protected parent
   directories are in place.
5. Separately, the authority provisioner derives the exact contract/scope from Ota's refusal or
   dry-run output, signs the bundle outside the repository and runner job, and atomically publishes
   the root-owned trust binding, bundle, and matching sequence state. The private signing key never
   enters the VPS.
6. Log in as `ota-runner` and run `ota authority inspect --json`. Require every required
   observation to pass before dispatching the pressure workflow. Preserve the inspection and Ota
   version JSON as workflow artifacts.
7. Dispatch exactly one administrator-provisioned scenario. For live authority, retain the receipt
   archive and verify it with `ota receipt --history --json`; for expired, revoked, and out-of-scope
   authority, prove refusal before any checkout mutation, task, service, child process, or provider
   work starts.

This is a runner preparation checklist, not a public authority-provisioning API. Ota does not yet
ship a signing CLI, provider-attested separation, or broker-backed one-use work-unit authority.

## Unsupported or bounded behavior

- Governed runtime and lifecycle proof retain one proof-owned terminal crossing transaction for
  their complete invocation and cleanup sets. Their archives embed and re-derive the exact carrier
  admission; selected task environments never receive the runner-private capability.
- Free-form task inputs refuse because this carrier will not hash or expose potentially secret
  values to manufacture scope identity.
- The `prebound_file` carrier supports only bounded calendar TTL. Broker-backed, nonce-bound,
  one-use work-unit leases use the separate protected `authority_broker` carrier.
- Provider-attested privilege separation, Windows ACL verification, and a public Ota signing CLI
  are not implemented. An authority issuer must be independently operated and should be reviewed
  before it provisions any runner state.
