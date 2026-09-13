# Secret Delivery Protected Service-Path Gate

This pressure gate proves one exact Core command can traverse the protected Launcher service,
reconcile the retained same-child observation, administrator-installed synthetic authority
snapshot, reconstructed Step 1-6 candidate, and snapshot-bound V2 transaction, then refuse before
provider contact or task execution.

It is intentionally non-production. The default Launcher installation retains empty secret-delivery
authority stores. The synthetic authority is installed only when the root administrator supplies
both the non-default Core pressure builder and a root-owned mode `0400` request.

## Administrator Preparation

Dispatch `.github/workflows/secret-delivery-oidc-endpoint-evidence.yml` while the protected runner is
stopped. Create one closed request containing the exact queued run context:

```json
{
  "schema_version": 1,
  "record_kind": "secret_delivery_pressure_authority_request",
  "contract_path": "/srv/ota-v3-pressure/ota.yaml",
  "task": "governed",
  "repository": "ota-run/ota",
  "repository_id": "<github-repository-id>",
  "repository_owner_id": "<github-owner-id>",
  "actor_id": "<github-actor-id>",
  "event_name": "workflow_dispatch",
  "workflow_run_id": "<queued-run-id>",
  "workflow_run_attempt": "1",
  "workflow_reference": "ota-run/ota/.github/workflows/secret-delivery-oidc-endpoint-evidence.yml@refs/heads/1.6.28-implementation",
  "runner_version": "<installed-runner-semver>",
  "workflow_sha": "<exact-core-commit>",
  "git_ref": "refs/heads/1.6.28-implementation",
  "commit_sha": "<exact-core-commit>"
}
```

Install the tracked contract at `/srv/ota-v3-pressure/ota.yaml`, the exact source-built Core binary,
and the non-default `ota-secret-delivery-pressure-authority` builder. Reprovision with the ordinary
arguments plus:

```text
--secret-delivery-pressure-builder-binary /usr/lib/ota-authority/bin/ota-secret-delivery-pressure-authority
--secret-delivery-pressure-request /etc/ota/secret-delivery-pressure-request.json
```

The provisioner derives the installed Core build and artifact identities itself, requires the
request's Core revision to match the installed source build, generates a separate signing key, and
installs a bounded signed bundle and verifier store. The builder returns only the private authority
payload and the exact public run context required by the selected Ota child.

Provisioning also writes a root-owned mode `0644` public record at
`/usr/share/ota/authority-launcher/secret-delivery-pressure-installation.json`. It binds the exact
builder artifact, canonical public request, Core revision, bounded provider-free installation
posture, and four selected-process environment values. It contains no private authority payload or
store identity, provider locator, protected capability identity, signing key, bearer, or secret
value.

## Acceptance

The hosted command must return the specific provider-free Step 7 refusal, never create
`selected-work-executed`, remove the selected child and transient scope, leave no active Launcher
state, and avoid publishing protected binding, source, capability, invocation, or transaction
identities.

This gate does not prove a real GitHub OIDC request, Google STS/WIF, Secret Manager contact,
materialization, process-environment injection, positive evidence, Step 8, V12.2, or general
repository and agent governance.
