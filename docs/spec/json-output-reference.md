# ota JSON Output Reference

This document records the current machine-readable output shapes for ota commands that support `--json`.

`docs/spec` is the canonical source of truth. This page is part of that spec
corpus and the public reference pages are derived from it with examples and
operator guidance added where useful.

The goal is stability for humans, CI, editors, and agents.

For the operator guide to the currently shipped assist flow, see [assist-workflow.md](assist-workflow.md).

Editor and CI integrations should treat the JSON surfaces in this document as the stable contract
and avoid scraping human-readable text output.

Canonical JSON Schema files for the current shipped shapes live in:

- [json-schemas/validate.json](json-schemas/validate.json)
- [json-schemas/env.json](json-schemas/env.json)
- [json-schemas/execution.json](json-schemas/execution.json)
- [json-schemas/execution-topology.json](json-schemas/execution-topology.json)
- [json-schemas/proof-runtime.json](json-schemas/proof-runtime.json)
- [json-schemas/proof-lifecycle.json](json-schemas/proof-lifecycle.json)
- [json-schemas/refusal-canary.json](json-schemas/refusal-canary.json)
- [json-schemas/effect-refusal-canary.json](json-schemas/effect-refusal-canary.json)
- [json-schemas/authority-inspect.json](json-schemas/authority-inspect.json)
- [json-schemas/replay-baseline.json](json-schemas/replay-baseline.json)
- [json-schemas/replay-baseline-authority.json](json-schemas/replay-baseline-authority.json)
- [json-schemas/services.json](json-schemas/services.json)
- [json-schemas/tasks.json](json-schemas/tasks.json)
- [json-schemas/assist-declare-readiness.json](json-schemas/assist-declare-readiness.json)
- [json-schemas/assist-declare-service.json](json-schemas/assist-declare-service.json)
- [json-schemas/assist-bind-task.json](json-schemas/assist-bind-task.json)
- [json-schemas/assist-declare-env.json](json-schemas/assist-declare-env.json)
- [json-schemas/assist-add-task.json](json-schemas/assist-add-task.json)
- [json-schemas/assist-normalize.json](json-schemas/assist-normalize.json)
- [json-schemas/assist-wire-setup.json](json-schemas/assist-wire-setup.json)
- [json-schemas/agents.json](json-schemas/agents.json)
- [json-schemas/doctor.json](json-schemas/doctor.json)
- [json-schemas/check.json](json-schemas/check.json)
- [json-schemas/clean.json](json-schemas/clean.json)
- [json-schemas/receipt.json](json-schemas/receipt.json)
- [json-schemas/init.json](json-schemas/init.json)
- [json-schemas/policy-init.json](json-schemas/policy-init.json)
- [json-schemas/up.json](json-schemas/up.json)
- [json-schemas/run-preview.json](json-schemas/run-preview.json)
- [json-schemas/detect.json](json-schemas/detect.json)
- [json-schemas/contract-candidate.json](json-schemas/contract-candidate.json)
- [json-schemas/contract-candidate-application.json](json-schemas/contract-candidate-application.json)
- [json-schemas/contract-upgrade.json](json-schemas/contract-upgrade.json)
- [json-schemas/policy-review.json](json-schemas/policy-review.json)
- [json-schemas/workspace-init.json](json-schemas/workspace-init.json)
- [json-schemas/workspace-tasks.json](json-schemas/workspace-tasks.json)
- [json-schemas/workspace-execution.json](json-schemas/workspace-execution.json)
- [json-schemas/workspace-run.json](json-schemas/workspace-run.json)
- [json-schemas/workspace-check.json](json-schemas/workspace-check.json)
- [json-schemas/workspace-doctor.json](json-schemas/workspace-doctor.json)
- [json-schemas/workspace-explain.json](json-schemas/workspace-explain.json)
- [json-schemas/workspace-up.json](json-schemas/workspace-up.json)
- [json-schemas/diff.json](json-schemas/diff.json)
- [json-schemas/explain.json](json-schemas/explain.json)
- [json-schemas/github-projection.json](json-schemas/github-projection.json)
- [json-schemas/ci-projection.json](json-schemas/ci-projection.json)
- [json-schemas/version.json](json-schemas/version.json)

## General notes

- success output is printed to stdout
- `ota run <task> --dry-run --json` keeps preview JSON on stdout for both ready and blocked
  previews; only pre-preview command failures fall back to the simpler error envelope
- command failures may still use stderr when the command cannot produce its normal JSON result
- some JSON failures include an optional `next` string when ota can point to one safe follow-up command
- execution receipts may also expose additive `next_steps` when the same follow-up lane is available as ordered machine-readable steps
- `ok: true` does not always mean zero findings; warning-only diagnosis can still be `ok: true`
- `path` refers to the resolved contract path as rendered by current CLI path compaction (often cwd-relative such as `./ota.yaml`)

## Which JSON surface to use

- use `ota validate --json` or `ota workspace validate --json` for contract gating
- use `ota --version --json` when you need machine-readable build identity and contract capability support
- use `ota env --json` for read-only environment inspection and validation
- use `ota execution plan --json` when you want the resolved backend, lifecycle, image, and target selection without running anything
- use `ota execution topology --json` when you want the declared execution graph for contract or topology inspection without running anything, including reusable readiness probes, reusable runtime surfaces, structured task launch sources, normalized listeners, and attached surface names
- use `ota proof runtime --json` when you want one thin clean-machine runtime-proof wrapper that captures the canonical topology, doctor, and up artifacts for a selected runtime path
- use `ota ci github render --json`, `check --json`, or `sync --json` when you need the
  deterministic GitHub reusable-workflow projection, its caller binding, and managed-file drift
  result without scraping YAML or text output
- use `ota services --json` when you want the declared managed-service inventory, including manager shape, readiness declaration, endpoint projections, and dependencies
- use `ota assist declare-readiness --json` when you want a deterministic readiness proposal or apply result without scraping review text
- use `ota assist declare-service --json` when you want a deterministic managed-service proposal or apply result without scraping review text
- use `ota assist bind-task --json` when you want a deterministic target-binding proposal or apply result without scraping review text
- use `ota assist declare-env --json` when you want a deterministic env proposal or apply result without scraping review text
- use `ota assist add-task --json` when you want a deterministic new-task proposal or apply result without scraping review text
- use `ota assist normalize --json` when you want a deterministic canonical-setup normalization proposal or apply result without scraping review text
- use `ota assist wire-setup --json` when you want a deterministic setup-wiring proposal or apply result without scraping review text
- use `ota workspace execution plan --json` when you want per-repo execution resolution across a workspace without running anything
- use `ota agents --json` when you want a repo-local `AGENTS.md` export preview or sync report
- use `ota skills install --json --agent <agent>` when you want the installed first-party skill target path
- use `ota doctor --json` or `ota workspace doctor --json` for readiness diagnosis and blocking findings
- use `ota policy init --json` when you want the starter org policy pack preview or write result
- use `ota policy review --json` when you need policy-authority review over a repo contract
- use `ota authority inspect --json` when a controlled runner must inspect the fixed prebound-file
  hardening profile without selecting a grant or creating crossing authority
- use `ota workspace explain --json` when you want an ordered workspace remediation plan
- use `ota workspace tasks --json` when you want workspace inventory and task availability
- use `ota workspace list --json` when you want lightweight workspace inventory and readiness
- use `ota workspace check --json` when you want checks-only workspace readiness with a roll-up summary
- use `ota receipt --json` when you want a read-only repo receipt artifact
- use `ota clean --json` or `ota clean --stale --json` when you want deterministic cleanup reports or structured cleanup failure details instead of scraping text receipts
- use `ota up --json` or `ota workspace up --json` when you want preparation or readiness roll-up data
- use `ota run <task> --dry-run --json` when you want a repo task execution preview without
  starting dependencies, processes, or containers
- use `ota workspace run --json` when you want coordinated multi-repo execution roll-up data and receipts
- use `ota workspace run --json --progress-json`, `ota workspace up --json --progress-json`, or
  `ota workspace refresh --json --progress-json` when you also need live machine-readable
  workspace progress events on stderr before the final stdout JSON report arrives
- use `ota workspace receipt --json` when you want a read-only workspace receipt artifact
- use `ota diff --json` or `ota explain --json` when you want contract change impact or remediation planning

## Editor and IDE contract rules

Editor and IDE consumers should prefer the smallest stable fields for the job instead of parsing
human text output:

- `ota validate --json` and `ota workspace validate --json`: use `ok`, `summary.error_count`, `errors` or `error`, and `next`
- `ota --version --json`: use `semver`, `version`, `source_build`, `commit`, `dirty`,
  `schema_version`, and `contract_capabilities[]`
- `ota agents --json`: use `ok`, `path`, `output`, `written`, `mode`, and `content`
- `ota skills install --json`: use `ok`, `skill`, `agent`, and `path`
- `ota execution plan --json`: use `contract_identity`, `declared_execution`, `resolved`, and `overrides`
- `ota execution topology --json`: use `contract_identity`, `declared_execution`, `shared_backends`, `readiness_probes`, `surfaces`, `services`, and `tasks`
- `ota run <task> --dry-run --json`: use `summary`, `resolved`, `requested_task`, `env`,
  `toolchains`, and `plan`; repo-level run JSON is currently preview-only and requires `--dry-run`
- `ota proof runtime --json`: use `mode`, `workflow`, `phase`, `summary`, and `artifacts`; inspect the referenced `doctor.json` and `topology.json` artifacts instead of expecting the proof wrapper to duplicate those payloads
- `ota ci github <render|check|sync> --json`: use `ok`, `operation`, `projection.identity`,
  `projection.workflow`, `projection.task`, `projection.merge_check_ids`, `output_path`,
  `caller_path`, and `mutated`; on failure use stable `code` and `message`, not text rendering
- `ota services --json`: use `services`, grouped `members` when present, and nested `manager`, `readiness`, `endpoints`, and `depends_on`; readiness may include `endpoint`, and endpoint objects may include `context` when the endpoint name and execution context differ
- `ota assist declare-readiness --json`: use `mode`, `subject`, `inputs.style`, `changes`, `validation`, and `next`
- `ota assist declare-service --json`: use `mode`, `subject`, `inputs`, `changes`, `validation`, and `next`
- `ota assist bind-task --json`: use `mode`, `subject.task`, `subject.target`, `inputs`, `changes`, `validation`, and `next`
- `ota assist declare-env --json`: use `mode`, `subject`, `inputs`, `changes`, `validation`, and `next`
- `ota assist add-task --json`: use `mode`, `subject.task`, `inputs`, `changes`, `validation`, and `next`
- `ota assist normalize --json`: use `mode`, `subject.task`, `subject.into`, `changes`, `validation`, and `next`
- `ota assist wire-setup --json`: use `mode`, `subject.task`, `inputs`, `changes`, `validation`, and `next`
- `ota workspace execution plan --json`: use the top-level `summary`, per-repo `resolved`, per-repo `error` / `next`, and optional top-level `overrides`
- `ota doctor --json` and `ota workspace doctor --json`: use the top-level `summary`, `finding_groups` when present, per-repo `findings`, per-repo `primary_blocker` when present, and `execution`; repo doctor also includes `mode`
- `ota workspace explain --json`: use the top-level `summary`, top-level ordered `actions`, per-repo grouped `actions`, and per-repo `steps` with stable codes
- `ota workspace tasks --json`: use the top-level `summary`, per-repo `tasks`, and dependency order
- `ota workspace list --json`: use the top-level `summary`, per-repo readiness, and contract presence
- `ota workspace check --json`: use the top-level `summary`, per-repo findings, and per-repo `primary_blocker` when present
- `ota receipt --json`: use the top-level `summary`, `receipt`, and `findings`
- `ota run --json`, `ota up --json`, `ota workspace run --json`, and `ota workspace up --json`:
  inspect `receipt.host_service_cleanup[]` when you need machine-readable evidence that ota
  attempted interrupt-driven host-managed service cleanup and whether each stop succeeded or failed
- `ota up --json`: inspect additive `governance.crossing` and `receipt.crossing` when the
  selected workflow crossed a heavier audited execution boundary and the operator supplied intent
  with `--reason`; the record keeps `requirement_source: derived` honest, and
  `crossing.evidence_classes`
  distinguishes caller-asserted fields such as `reason` from runner-derived or runner-attested
  fields. When a contract-bound signed grant admitted the crossing, `crossing.authority` carries
  the verified authority, exact semantic scope, carrier-specific admission, and archived signed
  evidence. `prebound_file` carries bundle/grant freshness and revocation posture.
  `authority_broker` carries launcher attestation, signed authorization, prepared lease, atomic
  consume exchange, and broker revision. If consume acknowledgement is uncertain, internal
  runner-owned state retains the exact consume intent and a separately domain-bound signed status;
  recovery always closes the abandoned transaction as incomplete and never resumes work. A
  consumed recovery retains its exact intent until the atomic terminal write. Pre-recovery broker
  archives retain their original seven-domain binding identity, while live bindings require the
  current nine-domain profile. These pending recovery fields are not successful receipt evidence.
  Real execution also carries a terminal
  `authority.transaction`; dry-run admission does not create one. Transaction schema v2 binds
  `authority_carrier` and a carrier-neutral `authorization_identity`; broker transactions also
  require `broker_consumption`. Its archive carries a matching carrier admission envelope.
  The semantic scope binds the selected workflow instance and its ordered prerequisite-instance
  closure. It also includes runner-derived `breadth`: selected closure node/edge counts, effect
  categories, and content-addressed resource identities/counts. Archive verification re-derives
  that breadth from the archived contract rather than trusting the summary. Ordinary workflow
  `--ready-timeout` is also part of execution selection, so changing it changes authority scope.
  The archived broker binding is a public verification snapshot that omits the protected live
  launcher descriptor. Signed broker protocol payloads retained for re-verification accept only bounded public-safe
  invocation, principal, and authority-mount labels. Raw nonce values, descriptors, credentials,
  filesystem paths, and secret provider material are not public evidence.
  Legacy v1 transaction history remains `prebound_file`-only and uses `grant_identity` instead.
  Broker attestation protocol v1 retains `authority_separation_posture:
  launcher_attested_one_use`. A strict v2 binding and signed payload instead carry one exact
  protocol-published protected-launcher profile, its content identity, ordered required
  observations, launcher/session identities, and a distinct attestor authority. Only a completely
  verified v2 profile emits `protected_launcher_attested_one_use`; missing, reordered, failed,
  unknown, downgraded, or substituted profile evidence refuses. Archive verification preserves the
  original v1/v2 binding, payload, response-domain, and identity-domain branch and cannot upgrade
  v1 by injecting defaults. The additive v3 systemd protected-launcher carrier instead binds
  Core's process-posture preface to the complete ordered launcher and job-principal profile
  observations; it emits `systemd_protected_launcher_attested_one_use` only when every required
  observation verifies. Selected execution additionally requires atomic
  lease consumption and one terminal Core crossing transaction before the outer launcher can emit
  exact child/scope/cgroup/active-slot finalization. The portable archive-attachment candidate uses
  a protected post-cleanup recovery journal plus separate producer signatures for cleanup and the
  exact receipt-archive/crossing-transaction association. The signed consume exchange binds
  launcher-owned transaction schema v3; that transaction requires broker-archive schema v2 and
  portable finalization re-verification. The root launcher, not the job principal, verifies the
  private archive through the execution-principal repository descriptor and atomically publishes
  the root-owned sidecar before exact client acknowledgement. Core first uses atomic create-new
  archive publication and file/directory sync; the launcher requires execution-principal-owned
  `0700` archive directories and retains the exact terminal until its separate identity-bound
  acknowledgement. Historical transaction v2 and
  broker-archive v1 evidence are not upgraded. Immutable Linux/x64 PID 1 pressure proves the
  production client, protected-history source, independently administered hardened-launcher path,
  and administrator-driven reboot/fault recovery. Provider attestation remains optional stronger
  hardening rather than a V11.7 completion gate.
  Neither posture implies provider-attested separation.
- `ota run <task> --dry-run --json` may include `crossing_grant_admission` after successful
  fixed-authority admission with `decision: admissible_not_consumed`, or after protected broker
  selection with `authority_carrier: authority_broker` and
  `decision: requires_live_authorization`. Broker preview does not contact the launcher or consume
  a lease. Neither form carries a transaction. A grant
  refusal instead returns `execution_started: false` plus
  `crossing_grant_admission.decision: refused`, `authority_source: prebound_file`, the configured
  authority and requested grant when present, a stable `reason_family`, and evaluation details.
  When the runner derived a complete scope before refusal, it also includes the scope identity,
  contract identity, boundary family, and classification needed by an external issuer to create
  an exact reviewable grant. It never includes task-input values or trust material.
  Admission-produced `ota up --json` refusal receipts carry the same scope binding under
  `receipt.refusal` and never carry `receipt.crossing`; its `boundary_family` remains the refusal
  boundary, while `scope_boundary_family` and `scope_classification` describe the selected lane.
  Mutating repo-level `ota run` remains a
  text execution surface; `ota run --receipt` renders the same typed refusal fields rather than
  inventing a mutating run JSON contract
- `ota run <task> --dry-run --json` and `ota up --json`: inspect additive
  `governance.post_execution.not_run_reason` and `governance.post_execution.crossing_record_state`
  when you need phase-accurate non-run and crossing-evidence posture instead of inferring from
  null fields alone. A preflight refusal remains `post_execution.state: not_run` because execution
  never began, but it sets `refusal_occurred: true` and carries the same refusal reason and record
  as preflight.
- the same `governance.post_execution.decision_basis[]` surface now carries the cited
  post-execution evidence-state basis when ota can decompose it honestly, such as
  `not_run:preview_only`, `not_run:preflight_refusal`, `evidence:receipt_present`,
  `evidence:proof_present`, or `crossing_record:attached`
- `ota clean --json` and `ota clean --stale --json`: use cleanup counters and `queried_engines` on success; on classified cleanup failures use `summary`, `reason`, ordered `next` steps, and the matching structured lane: engine/resource failures expose `engine`, `resource_kind`, `resource_name`, and `details`, while active execution cleanup barriers expose `registry_path`, typed `reasons[]`, `active_execution_count`, and `owners[]`; generic repo-state failures still fall back to `summary` plus `error`
- `ota up --json` and `ota workspace up --json`: use the top-level `summary`, `receipt`, and per-repo results; workspace repo results may also include additive `next` / `next_steps`
- repo-target `ota up --json` may also include additive `governance.crossing` when the selected
  workflow execution crossed a heavier audited boundary; this mirrors the same ota-authored
  record attached to `receipt.crossing`, including field-level `evidence_classes`
- `ota workspace run --json`: use the top-level `summary`, `receipt`, and per-repo results; repo results may also include additive `next` / `next_steps`
- `ota workspace run --json --progress-json`, `ota workspace up --json --progress-json`, and
  `ota workspace refresh --json --progress-json`: consume newline-delimited progress events from
  stderr for live repo transitions, then consume the final stdout JSON report as the canonical
  roll-up artifact
- `ota workspace receipt --json`: use the top-level `summary`, `receipt`, and per-repo results
- `ota diff --json`: use the readiness-impact summary and semantic `changes[]`; archived receipt
  JSON and archived `.ota/contracts/...` snapshot JSON are also valid diff inputs; additive
  `base_input` / `target_input` identify which semantic input kind each side resolved to and
  publish `snapshot_path` when a receipt-side diff resolves through `.ota/contracts/...`
- `ota explain --json`: use grouped `actions` for the ordered remediation plan and `steps` for stable finding-level detail

Hosted CI can use the same fields as annotations or check-run summaries:

- `summary.primary_blocker` when present, for the headline
- `findings[]` or per-repo `findings[]` as the annotation stream
- `finding_groups[]` when present, for grouped human-facing remediation summaries only
- `severity` to decide blocking versus warning annotations
- `why` for the annotation body

## `ota --version --json`

Purpose: expose machine-readable build identity and the contract capability surface this binary
supports without scraping human version text.

Current shape:

```json
{
  "ok": true,
  "semver": "1.6.27",
  "version": "v1.6.27",
  "source_build": false,
  "commit": null,
  "dirty": false,
  "schema_version": 1,
  "contract_capabilities": [
    {
      "id": "toolchains",
      "introduced_in": "1.6.15"
    },
    {
      "id": "execution.contexts.only_on",
      "introduced_in": "1.6.15"
    },
    {
      "id": "metadata.ota.minimum_version",
      "introduced_in": "1.6.15"
    },
    {
      "id": "resource_bindings",
      "introduced_in": "1.6.27"
    },
    {
      "id": "effect_definitions",
      "introduced_in": "1.6.27"
    },
    {
      "id": "tasks.effects.declared",
      "introduced_in": "1.6.27"
    },
    {
      "id": "tasks.effects.writes",
      "introduced_in": "1.6.15"
    },
    {
      "id": "tasks.effects.network",
      "introduced_in": "1.6.15"
    },
    {
      "id": "tasks.effects.network_kind",
      "introduced_in": "1.6.16"
    },
    {
      "id": "tasks.effects.external_state",
      "introduced_in": "1.6.15"
    },
    {
      "id": "tasks.action.ensure_env_file",
      "introduced_in": "1.6.16"
    },
    {
      "id": "tasks.action.ensure_file",
      "introduced_in": "1.6.16"
    },
    {
      "id": "tasks.action.ensure_directory",
      "introduced_in": "1.6.16"
    },
    {
      "id": "tasks.action.ensure_git_checkout",
      "introduced_in": "1.6.22"
    },
    {
      "id": "tasks.action.ensure_git_template",
      "introduced_in": "1.6.23"
    },
    {
      "id": "tasks.action.ensure_git_checkout.remotes",
      "introduced_in": "1.6.23"
    },
    {
      "id": "tasks.action.ensure_git_checkouts",
      "introduced_in": "1.6.23"
    },
    {
      "id": "tasks.action.ensure_container_network",
      "introduced_in": "1.6.21"
    },
    {
      "id": "tasks.action.build_container_image",
      "introduced_in": "1.6.24"
    },
    {
      "id": "tasks.action.reset_compose_service_volume",
      "introduced_in": "1.6.22"
    },
    {
      "id": "tasks.action.ensure_bundle",
      "introduced_in": "1.6.17"
    },
    {
      "id": "tasks.action.database_schema_mutation",
      "introduced_in": "1.6.27"
    },
    {
      "id": "checks.changed_files",
      "introduced_in": "1.6.16"
    },
    {
      "id": "tasks.when.checks",
      "introduced_in": "1.6.17"
    },
    {
      "id": "services.readiness.compose_health",
      "introduced_in": "1.6.16"
    },
    {
      "id": "tasks.runtime.readiness.signal_probes",
      "introduced_in": "1.6.16"
    },
    {
      "id": "agent.posture",
      "introduced_in": "1.6.15"
    },
    {
      "id": "agent.exceptions.sensitive_writes",
      "introduced_in": "1.6.15"
    },
    {
      "id": "native_prerequisites.visual_studio",
      "introduced_in": "1.6.15"
    },
    {
      "id": "native_prerequisites.requires",
      "introduced_in": "1.6.15"
    }
  ]
}
```

Notes:

- `semver`, `commit`, `dirty`, and `source_build` identify the exact binary
- `schema_version` is the current contract schema generation this binary speaks
- `schema_version` moves only when ota changes the machine-readable contract generation or
  compatibility interpretation in a way that is not just additive
- `contract_capabilities[]` is additive and lists high-signal contract features this binary
  understands, including the Ota version where each feature first shipped
- additive contract feature support should extend `contract_capabilities[]` without changing
  `schema_version`

## `ota validate --json`

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "error_count": 0,
    "warn_count": 0
  },
  "warnings": [],
  "warning_details": []
}
```

When validate warnings exist, `warnings[]` remains the backward-compatible string lane and
`warning_details[]` carries the stable machine-readable advisory objects with:
`code`, `category`, `owner`, `severity`, `summary`, `why`, and `next`.

Some advisory families also carry additive `provenance`. For selected dependency-plane boundary
advisories, that includes `provenance.parent_backend_selection_source` and
`provenance.dependency_backend_selection_source`, so automation can see whether the boundary came
from a task default mode, inherited parent backend, override, or another execution-selection lane
without scraping the summary text.

Failure shape can also include:

- `next`: optional safe follow-up command, used for trust-sensitive refusal and review-first flows
- `summary.error_count`: stable machine-facing count of validation errors or load failures

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "error_count": 2,
    "warn_count": 0
  },
  "errors": ["..."]
}
```

Or:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "error_count": 1,
    "warn_count": 0
  },
  "error": "..."
}
```

## `ota env --json`

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "workflow": "docker-build",
  "profile": "docker-build",
  "summary": {
    "contract_count": 3,
    "source_count": 2,
    "source_issue_count": 0,
    "task_count": 1,
    "resolved_count": 3,
    "missing_count": 0,
    "invalid_count": 0
  },
  "sources": [
    {
      "kind": "properties",
      "path": "app.properties",
      "label": "properties:app.properties",
      "must_exist": false,
      "status": "loaded"
    },
    {
      "kind": "json",
      "path": "env/runtime.json",
      "label": "json:env/runtime.json",
      "must_exist": true,
      "status": "loaded"
    },
    {
      "kind": "yaml",
      "path": "env/runtime.yaml",
      "label": "yaml:env/runtime.yaml",
      "must_exist": false,
      "status": "loaded"
    }
  ],
  "rendered_artifacts": [
    {
      "path": ".env.docker-build",
      "kind": "dotenv",
      "includes": ["DISCORD_TOKEN", "DOCS_SITE_BASE_URL"],
      "exists": true
    }
  ],
  "env": [
    {
      "name": "DISCORD_TOKEN",
      "kind": "contract",
      "required": true,
      "value": "***",
      "source": "properties:app.properties",
      "source_kind": "properties",
      "source_path": "app.properties",
      "source_status": "loaded",
      "source_label": "properties:app.properties",
      "status": "resolved"
    },
    {
      "name": "DOCS_SITE_BASE_URL",
      "kind": "contract",
      "required": true,
      "value": "https://docs.internal.example",
      "source": "org policy",
      "status": "resolved"
    },
    {
      "name": "CI",
      "kind": "task",
      "required": false,
      "value": "true",
      "source": "task",
      "status": "task"
    }
  ]
}
```

When a workflow is selected, success output may also include `workflow`, `profile`, and
`rendered_artifacts`. `rendered_artifacts` reports workflow-owned env materialization such as a
rendered dotenv file, including its repo-relative `path`, artifact `kind`, ordered `includes`, and
whether that artifact currently exists on disk.

When a declared source is missing or invalid, `sources` carries additive source metadata:
`kind`, `path`, `label`, `status`, optional `detail`, and optional `next`. Resolved env entries
loaded from declared sources also carry additive `source_kind`, `source_path`, `source_status`, and
`source_label` fields. Missing or invalid required env values remain visible in `env` with `status`
such as `missing` or `invalid`. The `kind` field is an explicit curated source kind such as
`dotenv`, `properties`, `json`, `yaml`, or `toml`.

When `--task` is set, `env[].required` reflects the selected task path, not only repo-global
`env.vars.<name>.required`. A top-level env var that is optional repo-wide may still report
`required: true` in task-scoped output when the selected task/workflow closure references it from
`tasks.<name>.requirements.env`.

Canonical source status values are:

- `loaded`
- `missing_optional`
- `missing_required`
- `parse_failed`
- `invalid_structure`
- `collision`

When `--task` is set, the payload includes the task name:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "task": "test",
  "summary": {
    "contract_count": 2,
    "source_count": 1,
    "source_issue_count": 0,
    "task_count": 1,
    "resolved_count": 2,
    "missing_count": 0,
    "invalid_count": 0
  },
  "sources": [],
  "env": []
}
```

When the selected closure contains a typed V12 effect and an organization policy is available,
`projection.governance.effect_policy_decision` carries the complete non-secret canonical decision.
It is part of `projection.identity`. An explicit typed deny returns an inspectable refusal with
`code: "effect_policy_denied"`; the generated provider workflow re-runs this projection against
its own checkout and refuses before setup or selected execution when policy or typed-effect truth
has drifted.

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "task": "test",
  "error": "task `test` is not defined in ota.yaml"
}
```

## `ota execution plan --json`

When the repo declares `workflows`, `ota execution plan --json` may include additive top-level
`workflow` and `task` fields. `workflow` mirrors the selected canonical operational path, and
`task` names the concrete workflow run task that drove execution planning, or the workflow setup
task when the workflow does not declare a run phase. `workflow.prepare_task` and
`workflow.prepare_action` are additive path context only; they do not replace the concrete
execution `task` because host bootstrap is not the selected runtime identity. The workflow object
may also include additive `notes` and `readiness_probes` when the selected workflow declares notes
or references reusable named probes.
When the selected workflow owns a rendered env artifact, success output also includes
`workflow_env_artifacts[]` with the artifact `path`, `kind`, `profile`, ordered `includes`,
current `exists` state, and consuming task/service lanes.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "contract": "/abs/path/to/ota.yaml",
  "workflow": {
    "name": "app",
    "intent": "local_development",
    "notes": "Use this workflow as the operator-first local path.\n",
    "prepare_task": "setup:env:local",
    "setup_task": "setup",
    "run_task": "dev",
    "required_services": ["postgres"],
    "readiness_checks": ["app-health"],
    "readiness_probes": ["app-ready"],
    "exposes": ["http://127.0.0.1:5678"]
  },
  "task": "dev",
  "workflow_env_artifacts": [
    {
      "path": ".env.docker-build",
      "kind": "dotenv",
      "profile": "docker-build",
      "includes": ["DATABASE_URL"],
      "exists": true,
      "consumers": ["task:dev", "service:postgres"]
    }
  ],
  "contract_identity": {
    "version": 1,
    "project": {
      "name": "ota"
    },
    "counts": {
      "runtimes": 0,
      "tools": 1,
      "env": 1,
      "services": 0,
      "checks": 1,
      "tasks": 4
    }
  },
  "declared_execution": {
    "preferred": "container",
    "supported": ["native", "container"],
    "lifecycle": "ephemeral",
    "backends": {
      "container": {
        "image": "rust:1.94-bookworm"
      }
    }
  },
  "resolved": {
    "backend": "container",
    "backend_source": "contract preferred",
    "lifecycle": "ephemeral",
    "lifecycle_source": "contract lifecycle",
    "image": "rust:1.94-bookworm",
    "engine_candidates": ["docker", "podman"],
    "target_strategy": "ephemeral per-run container"
  }
}
```

Notes:

- `governance.preflight.proof_expected: true` on `ota up --json` means the selected workflow is a
  proof-owning readiness lane, not just a receipt-only execution lane
- when the executed `up` result still carries resolved preflight governance from the selected
  workflow lane, `governance.preflight` preserves the same crossing posture fields used by preview
  and harness export, including `crossing_required`, `crossing_classification`, and
  `crossing_boundary_family` when ota can recover them honestly
- `governance.post_execution.proof_present: true` means the `up` pipeline reached proof/readiness
  evidence emission during execution
- `governance.post_execution.decision_basis[]` is the additive machine-readable citation set for
  the current post-execution evidence posture:
  - non-run basis such as `not_run:preview_only`, `not_run:preflight_blocked`, or
    `not_run:preflight_refusal`
  - evidence basis such as `evidence:receipt_present` or `evidence:proof_present`
  - crossing-record basis such as `crossing_record:attached`,
    `crossing_record:suppressed_by_refusal`, or `crossing_record:not_required`

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "errors": ["..."]
}
```

## `ota execution topology --json`

Execution topology inspection stays read-only. It reports the declared execution surface that a
contract or topology viewer can render without starting tasks or services.

Notes:

- top-level `readiness_probes` is present when the contract declares reusable readiness probes; it
  exposes the canonical literal-URL or topology-derived source plus the declared HTTP/TCP request
  contract for each probe
- top-level `surfaces` is present when the contract declares reusable runtime surfaces; it exposes
  the declared kind, port, optional label, optional purpose, optional visibility, optional path,
  and optional readiness contract for each reusable surface
- `tasks[*].env_files` is present when one task owns ordered task-local dotenv overlays for
  execution; these overlays are execution-only inputs, not root `env.sources`
- `tasks[*].adapter_inputs.compose.cwd` is present when one task owns the adapter working
  directory for `docker compose`; Ota enters that directory at runtime and reprojects declared
  compose file inputs relative to it
- `tasks[*].adapter_inputs.compose.env_files` is present when one task owns ordered compose
  interpolation files; Ota projects these through adapter-aware runtime input rather than
  pretending they are process dotenv overlays
- `tasks[*].adapter_inputs.compose.files` and `tasks[*].adapter_inputs.compose.project_name` are
  present when one task owns compose file selection or project naming; Ota projects these through
  adapter-aware runtime input instead of shell `-f` / `-p` flags
- `tasks[*].adapter_inputs.bake.cwd` and `tasks[*].adapter_inputs.bake.files` are present when one
  task owns Bake adapter root or Bake file selection; Ota projects these through adapter-aware
  runtime input instead of shell `cd ... &&` / `-f` glue
- `tasks[*].adapter_inputs.helm.cwd`, `.values_files`, `.chart`, `.release_name`, and
  `.namespace` are present when one task owns Helm adapter root, values-file selection, chart
  selection, release naming, or namespace truth; Ota projects these through adapter-aware runtime
  input instead of shell `cd ... && helm ...`, chart positionals, or `--namespace`
- `tasks[*].launch` is present when one task uses structured `launch` instead of shell `run` or
  `script`; it exposes the launch kind plus the structured command or packaged container metadata
- `tasks[*].command.interaction` is the resolved interaction posture for a structured `command:`
  task after defaulting. Consumers should treat it as one of `auto`, `forbidden`, or `required`.
- for a selected structured `command:` task, `ota run --dry-run --json` additionally publishes
  `interaction.posture`, `interaction.resolution`, and `interaction.terminal_available`;
  resolution is one of `terminal_passthrough`, `piped`, or `refused` for that invocation boundary.
  Other task-body families omit `interaction` rather than inventing a posture they do not own.
- task-target probe entries also expose `target.observer` and `target.resolution_plane`; the
  default command-host slice reports `command_host`, while observer-backed task probes report the
  named task plane they resolve through
- `tasks[*].runtime.readiness.probe` is present when one task runtime reuses a named
  `readiness.probes.<name>` declaration instead of declaring inline HTTP/TCP readiness transport;
  the named probe itself may be literal-URL-backed or target-backed
- `services[*].readiness.probe` is present when one managed service reuses a named
  `readiness.probes.<name>` declaration instead of declaring inline structured readiness transport;
  the named probe itself may be literal-URL-backed or target-backed

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "contract": "/abs/path/to/ota.yaml",
  "contract_identity": {
    "version": 1,
    "project": {
      "name": "topology-demo",
      "type": "application"
    }
  },
  "declared_execution": {
    "default_context": "development:ctx"
  },
  "shared_backends": [
    {
      "name": "workbench",
      "scope": "local",
      "backend": "container",
      "lifecycle": "persistent",
      "context": "development:ctx",
      "fulfillment": "run"
    }
  ],
  "readiness_probes": {
    "app-ready": {
      "kind": "http",
      "target": {
        "kind": "task",
        "name": "api",
        "listener": "backend",
        "address_view": "host",
        "observer": {
          "kind": "command_host"
        },
        "resolution_plane": "command_host"
      },
      "method": "GET",
      "path": "/health",
      "success": {
        "status": [200]
      },
      "timeout_ms": 10000
    }
  },
  "surfaces": {
    "backend": {
      "kind": "http",
      "port": 3000,
      "label": "Backend API",
      "purpose": "Primary local application API",
      "visibility": "internal",
      "path": "/",
      "readiness": {
        "kind": "http",
        "path": "/health"
      }
    }
  },
  "services": [],
  "tasks": [
    {
      "name": "api",
      "launch": {
        "kind": "command",
        "exe": "npx",
        "args": ["vite", "--host", "127.0.0.1", "--port", "3000"]
      },
      "runtime": {
        "kind": "service",
        "backend_binding": "workbench",
        "readiness": {
          "kind": "http",
          "listener": "backend",
          "path": "/health"
        },
        "attached_surfaces": ["backend"],
        "surface_attachments": {
          "backend": {
            "uses_defaults": false,
            "bind": {
              "address": "0.0.0.0",
              "port_mode": "fixed",
              "port_value": 3000
            },
            "project": {
              "host": {
                "address": "127.0.0.1",
                "port_mode": "fixed",
                "port_value": 3001,
                "primary": true
              }
            }
          }
        },
        "listeners": {
          "backend": {
            "protocol": "http",
            "bind_address": "0.0.0.0",
            "bind_port_mode": "fixed",
            "bind_port_value": 3000,
            "host_projection": {
              "address": "127.0.0.1",
              "port_mode": "fixed",
              "port_value": 3001,
              "primary": true
            }
          }
        }
      }
    },
    {
      "name": "web",
      "targets": [
        {
          "name": "api",
          "kind": "service",
          "activation_mode": "ensure_ready",
          "service": {
            "task": "api",
            "listener": "http",
            "address_view": "host"
          }
        }
      ]
    }
  ]
}
```

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "errors": ["..."]
}
```

## `ota ci projection --json` and `ota ci github <render|check|sync> --json`

`ota ci projection --workflow <name> --mode <native|container|remote> --target-os <linux|macos|windows> --json` emits the canonical
provider-neutral governance projection; its schema is [ci-projection.json](json-schemas/ci-projection.json).
GitHub projection commands consume that object. `render` has
`mutated: false` and returns the generated YAML under `projection.rendered`; an optional output path
is metadata only. `check` compares the marked managed file and caller reference without writing.
`sync` may set `mutated: true` only after it has verified the caller's structural binding and
the existing target is absent or Ota-owned.

Success:

```json
{
  "ok": true,
  "operation": "check",
  "projection": {
    "projection": {
      "workflow": "verify",
      "task": "verify",
      "run_execution": "finite_task",
      "mode": "native",
      "target_os": "linux",
      "refusal_canaries": [
        {
          "kind": "task",
          "target": "publish",
          "merge_check_id": "ota.refusal-canary.task.publish"
        }
      ],
      "toolchains": [
        { "name": "go", "source": "go", "version": "1.26", "execution_scopes": ["native"] }
      ],
      "bootstrap": { "source_kind": "version", "source_identity": "v1.6.24" },
      "proof_required": false,
      "governance": {
        "agent_admission": {
          "decision": "allow",
          "basis": ["agent_closure_admitted"]
        }
      },
      "identity": "sha256:..."
    },
    "runner": "ubuntu-latest",
    "provider_checks": [
      {
        "merge_check_id": "ota.verify.verify",
        "provider_check_name": "ota.verify.verify (linux/native)"
      }
    ],
    "content_identity": "sha256:...",
    "render_identity": "sha256:..."
  },
  "output_path": ".github/workflows/ota-governance.yml",
  "caller_path": ".github/workflows/ci.yml",
  "binding_identity": "sha256:...",
  "mutated": false
}
```

Failure:

```json
{
  "ok": false,
  "code": "managed_output_stale",
  "message": "..."
}
```

`managed_output_unowned`, `caller_projection_reference_mismatch`, and normalized
`.github/workflows`-path refusal are separate failures. `render_identity` binds the canonical
projection to the GitHub adapter version, normalized adapter inputs, and generated content;
`binding_identity` additionally binds that render to the parsed caller reference. A caller or output mismatch never triggers
a silent synchronization.

Generated lanes always follow the contract's agent-safe admission path. A `proof` claim qualifies
evidence breadth; it does not authorize an unsafe execution lane. For an admitted proof-required
workflow, the generated adapter uses `ota proof runtime` as the sole real execution lane.

`semantic_contract_identity` is the same normalized semantic snapshot identity carried by Ota
receipts. `bootstrap` carries the portable contract-owned bootstrap posture, and `proof_claim`
names the declared proof requirement. A denied provider-neutral projection still returns its
evaluated `projection` plus a typed `refusal` in JSON, so CI consumers can inspect the exact
identity, decision, and basis rather than infer them from an error string.

`projection.toolchains[]` is the normalized, selected-closure toolchain requirement for the target
OS. It is contract truth, not a runner-image assumption. Provider adapters may render supported
setup steps from it; unsupported required sources fail render rather than becoming ambient setup.

`projection.refusal_canaries[]` is the canonical contract-owned negative-control set for the
selected CI lane. Each unique entry carries `kind`, `target`, and a stable `merge_check_id` that
maps directly to one provider check; adapters run the corresponding Ota `--expect-refusal` command
and let its typed exit semantics enforce the control. A provider adapter must not substitute
shell/JQ assertions or contract-authored refusal reasons.

GitHub exposes that mapping in `projection.provider_checks[]`. `merge_check_id` stays canonical and
scope-free; `provider_check_name` adds the selected target OS and execution mode, preventing the
same canonical lane from collapsing when a caller runs it across multiple platforms.

## `ota proof lifecycle --json`

Lifecycle proof records one runner-owned service transition transaction. `ok` reports whether the
selected transaction completed and cleanup reached its declared terminal boundary; consumers must
read `proof_verdict` with required `not_proved[]` before treating it as application or repo proof.

- `mode` is `lifecycle-proof`, `phase` is `lifecycle`, and `stage_family` is `proof`.
- A typed-effect refusal before transaction creation uses the standard pre-execution failure
  envelope: `ok: false`, `code: OTA_EFFECT_POLICY_DENIED`, and
  `execution_started: false`. It records no positive lifecycle receipt or archive.
- A concurrent lifecycle transaction is rejected before a manager observation or cleanup lease;
  its finalization is `not_run` and its terminal record remains an unsuccessful, lease-free
  transaction.
- Each `services[]` record is bound to the same transaction identity and names the observed
  pre-existing state, cleanup lease, ownership, and start/readiness/teardown outcomes.
- An isolated-boundary record additionally carries runner-owned `boundary_identity`. The archive
  scope carries the same identity; `boundary_terminated` is invalid unless both point to the exact
  container session Ota created and later removed.
- `start` and `teardown` command outcomes are runner-attested. Readiness and manager-state
  observations are derived from the declared service manager; an omitted readiness remains
  `not_declared`, never a positive state claim. Phase-specific transition states are enforced:
  command phases cannot emit a readiness or boundary state, and `boundary_terminated` is always
  an attested teardown assertion. An isolated removal failure is also attested against its exact
  boundary identity and leaves finalization incomplete; a manager-state failure remains derived.
- When a workflow declares an assertion, `assertion` carries its task, terminal state, and exit
  code when available. Failed or interrupted assertions also carry bounded runner-captured
  stdout/stderr tails with declared secret values redacted. `output_truncated: true` means one or
  both tails were limited to 8 KiB; these diagnostics explain the transaction result but do not widen lifecycle proof into
  application-output proof.
- A successful lifecycle transition is always `passed_with_unproven_boundaries`, never bare
  `passed`. Ota always emits `application_output_not_proved` and
  `broader_repo_completion_not_proved` unless a future dedicated output-proof carrier establishes
  broader truth.
- When a selected service has no declared readiness or manager-owned start-state observation,
  Ota also emits obligation-scoped `service_started_state_not_proved`; a successful start command
  alone is never promoted into an observed started state.
- `ota proof lifecycle --json --archive` adds `archive.identity` and `archive.path`. The immutable
  local record binds the semantic contract snapshot and source identity when available, selected
  member/workflow/services plus the snapshot-derived dependency closure and teardown authority,
  transaction, complete service records, terminal verdict and finalization, exact isolated-boundary identity when used, and
  effective backend/mode/provider/lifecycle/target, requested backend/lifecycle overrides, target
  platform, host-port, memory, and
  dependency-selection posture. When authority is required, `crossing_evidence` embeds the exact
  proof-owned terminal authority transaction; archive verification re-derives it from the archived
  contract and selected service/assertion scope. Its content-addressed filename,
  content identity, contract identity, and archived
  semantic snapshot reference are verified before Ota accepts the record as locally well-formed.
  It does not establish application-output proof,
  CI eligibility, claim assurance, replay, or broader repo completion.
- Runtime proof emits a fresh `execution_id` for the exact proof run. Direct
  `crossing_evidence.proof_execution_id` must match it, and the terminal authority transaction
  binds the same value in its finalized proof status. Semantic scope identity remains reusable
  policy truth; execution identity prevents a different valid run with the same scope from being
  substituted into a proof archive.
- Lifecycle archive version `3` carries platform, execution-selection, and direct proof authority
  bindings. Released version `2` archives remain inspectable only in their original ungoverned
  shape; adding v3-only scope or crossing fields to a v2 record is rejected.
- `--agent`, `--mode <native|container|remote>`, and `--member <name>` reuse the selected
  workflow's agent admission, task-mode resolution, and monorepo target loading. Service-manager
  commands retain their declared manager boundary; the mode applies to workflow prerequisite and
  assertion tasks rather than pretending that a host manager was relocated.
- `finalization.state: completed_after_interruption` and `after_interruption: true` record that
  the runner completed its reverse finalizer after an interrupted start, readiness observation, or
  assertion; each service record retains the corresponding typed `interrupted` transition and
  teardown evidence. `incomplete_after_interruption` preserves both facts when a teardown
  interruption also leaves manager-state cleanup unproved.

The command schema is [proof-lifecycle.json](json-schemas/proof-lifecycle.json); archived records
use [proof-lifecycle-archive.json](json-schemas/proof-lifecycle-archive.json).

## `ota baseline record|promote --json`

The command-result schema is [replay-baseline.json](json-schemas/replay-baseline.json). The
portable recorded attestation and committed authority-manifest schema is
[replay-baseline-authority.json](json-schemas/replay-baseline-authority.json).

- `state: recorded` means Ota executed the declared producer successfully and emitted one
  content-addressed attestation bound to the archived execution receipt, exact task scope, and
  resolved backend/lifecycle. It does not select that record for replay.
- `state: promoted` means an explicit caller selected one previously recorded attestation and Ota
  atomically wrote the declared portable authority manifest. The manifest embeds that immutable
  attestation, so a fresh clone can inspect the selected record without local `.ota` archive
  retention. Its `trust_root: scm_review` declares repository review as the external authority
  boundary; Ota does not verify reviewer inclusion or signer provenance.
- `attestation_identity` always identifies the selected Ota-recorded attestation. `promotion_identity`
  is present only after promotion.
- `ok: false` has `code: replay_baseline_operation_failed`; no partial producer output, newest
  record, or handwritten digest is implicitly promoted.

Consumers validate the portable promoted authority and its complete output identity set before
execution. The record and promotion JSON do not claim that a human reviewed the change; the
manifest's explicit SCM-review trust root identifies the external authority boundary only.

## `ota proof runtime --json`

Runtime proof stays intentionally thin. It proves one selected runtime path and captures the real
declared artifacts that remain canonical:

- `topology.json` from `ota execution topology --json`
- `doctor.json` from `ota doctor --json`
- `up.log` from the repo-level runtime-preparation lane

Notes:

- `mode` is always `runtime-proof`
- A typed-effect refusal before proof artifact creation uses the standard pre-execution failure
  envelope: `ok: false`, `code: OTA_EFFECT_POLICY_DENIED`, and
  `execution_started: false`. It records no positive runtime-proof receipt or archive.
- `workflow` is present when the proof targeted one explicit or effective workflow
- `phase` stays machine-stable and uses the proof phase keys `preconditions`, `prepare`,
  `setup`, `services`, `run`, `readiness`, `cleanup`, and `interrupted`
- `stage_family` is always `proof`, so CI and agents can classify this wrapper without inferring
  from proof phase names
- `ok` is execution/readiness success only. It is necessary but insufficient for a proof claim:
  consumers must interpret `proof_verdict` together with `not_proved[]` before treating a selected
  lane as a proof result.
- `proof_verdict` is the terminal evaluation of this proof carrier: `passed` means the selected
  runtime path passed with no declared boundary, `passed_with_unproven_boundaries` means the path
  passed but must not be over-read beyond its published boundary, and `failed` means selected-lane
  execution or readiness evaluation failed; parse/load failures do not enter this carrier
- `proof_scope` is the first canonical machine-readable boundary for this carrier; it names the
  covered runtime-path lane and keeps narrow proof from being over-read as broader repo truth
- `crossing_evidence` binds a governed runtime proof to one proof-owned terminal crossing
  transaction across the detached workflow, nested task, ordered seam observers, negative
  controls, and cleanup. New archives embed the complete carrier admission and terminal
  transaction; legacy archives may retain a child-receipt link. Archive verification re-derives
  the exact scope and refuses missing, borrowed, mismatched, or nonterminal authority. This never
  widens proof to repository-wide safety or application correctness.
- `execution_boundary` is additive runner-authored prerequisite provenance. Its `identity` binds
  the sorted asserted-target and derivation-input closures, declared artifact/producer ownership,
  prerequisite records (including a verified precondition identity when state is reused), and
  ordered causal edges. `target_freshness` is independent from
  `derivation_posture`: `unknown` means Ota did not witness the complete precondition,
  materialization, and assertion chain; it must not be treated as a cold start. The first
  runner-attested paths cover native `ensure_virtualenv` followed by a runner-recorded native
  `.venv/bin/*` consumer, and frozen native pnpm hydration followed by a declared local `pnpm exec`
  or package-script consumer. The pnpm identity joins its generated `node_modules/.modules.yaml`
  layout marker to the declared `pnpm-lock.yaml`; it is not a claim to hash the entire mutable
  dependency tree. Both emit `asserted_at`, not `consumed`: Ota observed the recovered identity at
  the selected executable or local package-resolution boundary, but does not claim
  adapter-instrumented process-level use. Other filesystem artifacts, container paths, Windows
  virtualenv executables, other Node package-manager layouts, volumes, caches, and provider state
  remain `unknown` until Ota can attest the same complete chain.
- `dependency_evidence[]` is additive positive seam evidence for the selected runtime path.
  The current first carrier is intentionally narrow: Ota emits `level: reachable` only when a
  declared service seam is also part of the selected workflow's required-service closure and Ota
  has a structured readiness owner for that selected service. This is runner-derived reachability
  evidence, not proof of exercised interaction across the seam.
- `dependency_evidence[].interaction_attempted` is additive caller-side-only seam evidence. Ota
  emits it when the selected proof lane clearly attempted the dependency interaction from the
  caller side, but still lacks independent dependency-side or round-trip evidence to promote that
  seam beyond `not_proved`.
- `dependency_evidence[].observation.origin` names where the seam evidence came from. Current
  shipped reachable records use `dependency_side` for manager-owned readiness such as
  `compose_health` and `systemd_active`, and `round_trip_effect` for probe-owned readiness such as
  `http` and `tcp`; current shipped attempted records use `caller_side`.
- `dependency_evidence[].observation.evidence_class` is the V11.9 authority class for the
  observation itself. Current shipped reachable records are `derived`.
- `seam_observations[]` records each declared marker-bound producer/observer pair. Ota injects the opaque marker
  into the declared producer task, but never passes it to the observer. The observer must write a
  transient JSON attestation after recovering that marker through the dependency. Only a matching
  transaction id, observation id, and marker emits an attested
  `dependency_evidence[].level: exercised` record with `observation.origin: round_trip_effect`.
  Receipts retain non-secret transaction and attestation digests; the marker and transient
  attestation are removed after verification. Failures leave the seam explicitly not proved.
- Text output renders the same `Dependency Evidence` before `Proof Boundaries`, including the
  level, observation origin, and authority class. A caller-side-only attempt is rendered as not
  independently exercised; it does not become a green seam claim in human output.
- `negative_control` is optional canonical boundary-attested evidence from one explicitly selected
  `workflows.<name>.proof.negative_controls[]` task. It names its green `obligation_id`, carries
  the matching proof `transaction_id`, contract-declared `intervention`, and publishes
  `status: unrun | invalid | validated`. Failed controls also carry a runner-derived
  `failure_mode` when Ota can classify one. `nonzero_exit_observed` and
  `unclassified_nonzero` are bounded evidence, not evidence that the declared dependency caused
  the failure. Only `validated` with `outcome: expected_obligation_failed`,
  `failure_mode: expected_missing_effect`, and a matching failure-attestation digest may promote
  the exact `dependency_evidence[].proof_obligation_id` record to `fault_tested`. The nested
  `dependency_evidence[].negative_control` object carries `evidence_class: derived` as a
  self-describing projection of this canonical record, never a second source of truth. A
  `validated` projection includes `negative_control_id`, which must equal the canonical top-level
  `negative_control.id`; its parent dependency and obligation identities and its
  `failure_attestation_digest` must exactly equal the canonical record. Core performs that
  cross-record reconciliation before emission and when loading runtime-proof archives. When the
  archived scope selected a control, Core re-derives it from the archived contract and requires
  exactly one canonical record and matching projection; other consumers must do the same. JSON
  Schema enforces the local field shape but cannot compare sibling values.
  `unexpected_success`, `control_could_not_run`, stale evidence,
  and unrelated non-zero exits are `invalid` or `unrun` and fail the selected control proof.
- `not_proved[]` is relative to that declared runtime-path scope, not free-floating commentary;
  Ota emits `functional_runtime_not_proved` when proof fell back to a setup-only lane,
  `dependency_exercise_not_proved` for each declared service seam in the selected closure when
  Ota has not independently observed a contract-specific interaction across it,
  `external_network_path_not_proved` when an adjacent declared workflow owns an explicit external
  integration-test path for the selected lane's declared external state, and
  `dependency_causality_not_proved` when a seam was exercised but no matching negative control
  validated causal dependency necessity, and
  `dependency_output_shaping_not_proved` for every marker-bound seam obligation: an exercised or
  fault-tested seam proves only its declared seam obligation, never broader application-output
  shaping, and
  `broader_repo_completion_not_proved` as the scope-derived remainder outside this runtime slice
- `not_proved[].declared_by_workflows` names the adjacent contract workflows that establish a
  contract-derived exclusion; scope-derived exclusions omit it
- `not_proved[].dependency_id` and `not_proved[].declared_by_tasks` identify a declared service
  seam that remains unproved. This is deliberately not evidence that the service is unused or
  unreachable; it prevents a green runtime proof from being over-read as an exercised dependency.
- `not_proved[].proof_obligation_id` appears when a boundary is narrower than its dependency. Every
  marker-bound seam retains `dependency_output_shaping_not_proved` for the same obligation;
  absence is reserved for a future explicit output-proof carrier. Ota may prove dependency
  exercise or necessity for that seam, not that the dependency shaped a broader application
  output.
- when Ota has only caller-side seam evidence, the paired `dependency_exercise_not_proved`
  boundary tightens `reason` from generic missing evidence to `caller_side_only_evidence`
- `artifact_routing[]` points at the proof artifact bundle this wrapper governs, such as
  `proof_runtime_json`, `proof_topology`, `proof_doctor`, and `proof_up_log`
- `ota proof runtime --json --archive` additionally writes one immutable proof-owned record under
  `.ota/proof/archives/` and returns additive `archive.identity` and `archive.path`. The archive
  is content-addressed and binds the terminal proof JSON to an archived semantic contract snapshot,
  clean Git source identity when available, resolved workflow/task/backend/provider/lifecycle,
  target-platform, host-port, memory, dependency selection, and normalized readiness-timeout
  selection scope, plus explicit
  `replay_posture: witness_only`. The
  mutable `.ota/proof/<lane>/` working
  bundle remains supporting evidence, not a replay-grade witness. Archive consumers also verify
  that `contract_snapshot_ref` resolves to the same content-addressed snapshot as
  `contract_snapshot_hash` before using the record as assurance evidence. The archive retains the
  exact same `execution_boundary` record and identity as the emitted proof output.
- `summary` reuses the doctor verdict/count shape instead of inventing a second readiness dialect
- `artifacts` points at the captured canonical payloads; machine consumers should inspect those
  files directly when they need the full topology or doctor surface
- `workflow_env_artifacts` is additive and reports workflow-owned rendered env files plus the
  consuming task/service lanes that proof exercised
- when the selected run task is a service launcher that exits successfully before the runtime is
  actually ready, such as `docker compose up -d`, proof keeps using the declared readiness budget
  instead of collapsing that launcher exit into an immediate proof failure
- `likely_cause` is optional and appears only when ota can derive a higher-confidence runtime-drift
  hint from captured proof logs; treat it as advisory, not a replacement for `doctor.json` or
  `up.log`
- `likely_cause_evidence` is optional and publishes the machine-readable root-cause signal behind
  `likely_cause`; current shipped kinds are `auth_credential_failure`,
  `dns_service_name_resolution_failure`,
  `readiness_target_mismatch`,
  `loopback_service_drift`, `missing_env`, `install_or_toolchain_failure`, `bind_conflict`, and
  `detached_run_output`
- `likely_cause_evidence` can now publish additive target fields such as `listener`,
  `declared_target`, and `observed_target` when ota can compare the declared proof endpoint with
  the runtime endpoint it observed in proof artifacts
- `cleanup_failure` is optional and appears when ota can classify proof-boundary cleanup failure
  truth structurally; it keeps cleanup reason, owned resource lane, and next steps machine-readable
  without forcing automation back onto prose parsing
- cleanup failures still surface through top-level `error`, `failure_class: cleanup_failure`, and
  `next` so existing consumers remain compatible while richer cleanup evidence stays additive
- high-confidence runtime-drift hints upgrade readiness-shaped failures to
  `failure_class: config_drift` so automation can distinguish probable contract/config drift from
  generic slow-start timeouts
- high-confidence port/listener collisions upgrade readiness-shaped failures to
  `failure_class: bind_conflict` so automation can distinguish address-in-use failures from
  generic startup noise
- high-confidence install/hydration/compiler failures can now surface structured
  `likely_cause_evidence.kind: install_or_toolchain_failure` from captured `up.log`, so the proof
  lane does not reduce package-manager/toolchain breaks back to prose-only diagnostics
- high-confidence missing-config failures can now surface structured
  `likely_cause_evidence.kind: missing_env`, including the missing variable name when Ota can
  recover it from captured `up.log`
- high-confidence runtime endpoint mismatches can now surface structured
  `likely_cause_evidence.kind: readiness_target_mismatch`, including the declared readiness target
  and the observed runtime target when Ota can recover both from proof artifacts
- high-confidence dns/service-name resolution failures can now surface structured
  `likely_cause_evidence.kind: dns_service_name_resolution_failure`, including the unresolved host
  name when Ota can recover it from proof logs
- high-confidence auth/credential failures can now surface structured
  `likely_cause_evidence.kind: auth_credential_failure`, including the named backend service when
  Ota can recover it from proof logs
- timeout-only failures without a blocking primary doctor finding are normalized to
  `failure_class: readiness_timeout`
- signal-terminated proof runs are normalized to `failure_class: interrupted`
- pre-runtime blockers are normalized to phase `preconditions` with
  `failure_class: precondition_blocked`

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "runtime-proof",
  "workflow": "app",
  "phase": "readiness",
  "stage_family": "proof",
  "proof_verdict": "passed_with_unproven_boundaries",
  "proof_scope": {
    "kind": "runtime_path",
    "proof_class": "slice_proof",
    "workflow": "app",
    "task": "serve",
    "intent": "packaged_runtime"
  },
  "dependency_evidence": [
    {
      "dependency_id": "service:postgres",
      "proof_obligation_id": "postgres-marker",
      "level": "fault_tested",
      "observation": {
        "origin": "round_trip_effect",
        "evidence_class": "attested"
      },
      "declared_by_tasks": ["serve"],
      "declared_by_workflows": ["app"],
      "negative_control": {
        "status": "validated",
        "same_obligation": true,
        "negative_control_id": "postgres-unavailable",
        "failure_mode": "expected_missing_effect",
        "failure_attestation_digest": "sha256:..."
      }
    },
    {
      "dependency_id": "service:postgres",
      "interaction_attempted": true,
      "observation": {
        "origin": "caller_side",
        "evidence_class": "derived"
      },
      "declared_by_tasks": ["serve"],
      "declared_by_workflows": ["app"]
    }
  ],
  "negative_control": {
    "id": "postgres-unavailable",
    "dependency_id": "service:postgres",
    "obligation_id": "postgres-marker",
    "transaction_id": "sha256:...",
    "control_task": "verify:postgres-unavailable",
    "intervention": {
      "kind": "dependency_endpoint_override",
      "id": "postgres-unavailable"
    },
    "expected_failure": "dependency_unavailable",
    "outcome": "expected_obligation_failed",
    "status": "validated",
    "failure_mode": "expected_missing_effect",
    "proof_scope_ref": "workflow:app/negative_control:postgres-unavailable",
    "evidence_class": "attested",
    "failure_attestation_digest": "sha256:...",
    "exit_code": 1
  },
  "not_proved": [
    {
      "kind": "dependency_exercise_not_proved",
      "relative_to": "runtime_path",
      "source": "contract_lane",
      "dependency_id": "service:postgres",
      "reason": "caller_side_only_evidence",
      "declared_by_tasks": ["serve"],
      "declared_by_workflows": ["app"]
    },
    {
      "kind": "broader_repo_completion_not_proved",
      "relative_to": "runtime_path",
      "source": "proof_scope"
    }
  ],
  "summary": {
    "verdict": "ready",
    "agent_verdict": "ready",
    "error_count": 0,
    "warn_count": 0,
    "info_count": 0
  },
  "artifacts": {
    "topology": "./.ota/proof/app/topology.json",
    "doctor": "./.ota/proof/app/doctor.json",
    "up_log": "./.ota/proof/app/up.log"
  },
  "workflow_env_artifacts": [
    {
      "path": ".env.docker-build",
      "kind": "dotenv",
      "profile": "docker-build",
      "includes": ["DATABASE_URL"],
      "exists": true,
      "consumers": ["task:app", "service:postgres"]
    }
  ]
}
```

Blocked:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "mode": "runtime-proof",
  "workflow": "docker",
  "phase": "services",
  "summary": {
    "verdict": "not_ready",
    "agent_verdict": "ready",
    "error_count": 1,
    "warn_count": 0,
    "info_count": 0,
    "primary_blocker": {
      "severity": "error",
      "summary": "Service readiness failed: backend",
      "why": "readiness probe did not report success before timeout",
      "next": "inspect `./.ota/proof/docker/up.log`, then rerun `ota doctor --workflow docker .`"
    }
  },
  "artifacts": {
    "topology": "./.ota/proof/docker/topology.json",
    "doctor": "./.ota/proof/docker/doctor.json",
    "up_log": "./.ota/proof/docker/up.log"
  },
  "failure_class": "config_drift",
  "likely_cause": "likely config drift: the runtime is still targeting Redis on loopback (127.0.0.1:6379) inside a multi-service startup path; move that host binding into a workflow-scoped env overlay, rendered workflow env artifact, task `env_files`, or compose `manager.env_file` instead of `127.0.0.1` / `localhost`",
  "likely_cause_evidence": {
    "kind": "loopback_service_drift",
    "artifact": "./.ota/proof/docker/up-detached-run.log",
    "signal": "connection_refused",
    "service": "Redis",
    "host": "127.0.0.1",
    "port": 6379
  }
}
```

Cleanup-classified failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "mode": "runtime-proof",
  "workflow": "app",
  "phase": "cleanup",
  "summary": {
    "verdict": "ready",
    "agent_verdict": "ready",
    "error_count": 0,
    "warn_count": 0,
    "info_count": 0
  },
  "artifacts": {
    "topology": "./.ota/proof/app/topology.json",
    "doctor": "./.ota/proof/app/doctor.json",
    "up_log": "./.ota/proof/app/up.log"
  },
  "failure_class": "cleanup_failure",
  "error": "cleanup failed",
  "cleanup_failure": {
    "summary": "Cleanup failed",
    "error": "cleanup failed",
    "why": "`ota clean` could not stop host-managed service `redis` before cleaning repo state.",
    "next": ["inspect the stop command for `redis` and rerun `ota clean`"],
    "reason": "other",
    "engine": "host",
    "action": "stop",
    "resource_kind": "host_service",
    "resource_name": "redis",
    "details": "stop command exited with code 1 (permission denied)"
  },
  "next": "run `ota clean /abs/path/to/ota.yaml` to remove the remaining runtime state, then rerun proof"
}
```

## `ota services --json`

Services inspection stays read-only. It reports the declared managed-service inventory without
starting or probing those services.

Notes:

- top-level `services` lists declared repo services in deterministic order
- `members` is present when a monorepo root request includes grouped member service summaries
- canonical service docs focus on `producer`, `manager`, projected `endpoints`, structured
  `readiness`, and `depends_on`; compatibility-mode fields may still appear in output when older
  contracts are inspected

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "services": [
    {
      "name": "postgres",
      "required": true,
      "manager": {
        "kind": "compose",
        "name": "local",
        "file": "compose.yaml",
        "service": "postgres"
      },
      "readiness": {
        "kind": "compose_health"
      },
      "depends_on": []
    }
  ]
}
```

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "errors": ["..."]
}
```

## `ota assist declare-readiness --json`

Assist readiness output reports one deterministic proposal or apply result for an existing task
runtime service or managed service.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "preview",
  "operation": "declare-readiness",
  "subject": {
    "service": "api"
  },
  "inputs": {
    "endpoint": "host_3000",
    "style": "http"
  },
  "assumptions": [
    "service `api` already declares endpoint `host_3000`",
    "structured readiness should anchor to endpoint `host_3000` in context `host`"
  ],
  "changes": [
    {
      "path": "services.api.readiness",
      "action": "set",
      "before": null,
      "after": {
        "kind": "http",
        "from": "host",
        "endpoint": "host_3000",
        "method": "GET",
        "path": "/health"
      }
    }
  ],
  "diff": "services.api.readiness\n- <absent>\n+ kind: http ...",
  "validation": [
    "ota validate /abs/path/to/ota.yaml",
    "ota doctor /abs/path/to/ota.yaml"
  ],
  "next": "rerun with `ota assist declare-readiness --service api --endpoint host_3000 --style http --write /abs/path/to/ota.yaml` to apply this readiness change"
}
```

Notes:

- `path` is the resolved repo contract path
- `member` is present only when `--member` targeted a merged monorepo member contract
- `subject` contains exactly one selector key today: `task` or `service`
- `inputs.endpoint` is present only when `--endpoint` selected one explicit managed-service projection
- `changes[*].before` is present when assist is refining or replacing an existing readiness block, including legacy top-level readiness shapes
- `changes[*].after` uses the canonical readiness contract shape Ota would write
- `mode` is `preview` by default and `write` when `--write` succeeded
- `inputs.style` may also be `compose-health` for compose-managed `--service` targets

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "operation": "declare-readiness",
  "subject": {
    "task": "dev"
  },
  "why": "task `dev` has multiple readiness candidate listeners; assist cannot pick one safely",
  "next": "rerun with `--style <spring-http|http|tcp>` after narrowing the runtime surface"
}
```

## `ota assist declare-service --json`

Assist service output reports one deterministic proposal or apply result for one top-level managed
service declaration.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "preview",
  "operation": "declare-service",
  "subject": {
    "service": "postgres"
  },
  "inputs": {
    "manager": "compose",
    "endpoint": "web",
    "endpoint_context": "host",
    "address": "127.0.0.1",
    "port": "5432",
    "required": "false",
    "compose_file": "docker-compose.yml",
    "compose_service": "postgres",
    "style": "tcp"
  },
  "assumptions": [
    "service `postgres` will be created under `services`",
    "endpoint `host` is the service projection boundary",
    "manager `compose` is the service owner"
  ],
  "changes": [
    {
      "path": "services.postgres",
      "action": "set",
      "before": null,
      "after": {
        "required": false,
        "manager": {
          "kind": "compose",
          "name": "local",
          "file": "docker-compose.yml",
          "service": "postgres"
        },
        "endpoints": {
          "web": {
            "context": "host",
            "address": "127.0.0.1",
            "port": 5432
          }
        },
        "readiness": {
          "from": "host",
          "endpoint": "web",
          "kind": "tcp"
        }
      }
    }
  ],
  "diff": "services.postgres\n- <absent>\n+ required: false ...",
  "validation": [
    "ota validate /abs/path/to/ota.yaml",
    "ota doctor /abs/path/to/ota.yaml"
  ],
  "next": "rerun with `ota assist declare-service --name postgres --manager compose --endpoint web --endpoint-context host --address 127.0.0.1 --port 5432 --compose-file docker-compose.yml --compose-service postgres --style tcp --write /abs/path/to/ota.yaml` to apply this service change"
}
```

Notes:

- `subject.service` is the targeted top-level managed service name
- `inputs` records the explicit or defaulted service declaration inputs used to build the proposal
- `inputs.endpoint_context` is present when assist is authoring an endpoint whose identity and execution context differ
- `changes[*].before` is present when assist is refining an existing service block
- `changes[*].after` is the exact service block Ota would write

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "operation": "declare-service",
  "subject": {
    "service": "postgres"
  },
  "why": "service `postgres` needs an explicit manager kind",
  "next": "rerun with `--manager compose` or `--manager host`"
}
```

## `ota assist bind-task --json`

Assist target-binding output reports one deterministic proposal or apply result for
`tasks.<consumer>.targets.<name>`.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "preview",
  "operation": "bind-task",
  "subject": {
    "task": "smoke",
    "target": "api"
  },
  "inputs": {
    "to": "dev:http",
    "address_view": "topology",
    "activation": "manual"
  },
  "assumptions": [
    "a new target binding will be created under `tasks.smoke.targets.api`",
    "task `smoke` will resolve `api` through producer task `dev` listener `http`",
    "ota will resolve this edge with `address_view: topology`",
    "`activation.mode` will be `manual`"
  ],
  "changes": [
    {
      "path": "tasks.smoke.targets.api",
      "action": "set",
      "before": null,
      "after": {
        "service": {
          "task": "dev",
          "listener": "http",
          "address_view": "topology"
        },
        "activation": {
          "mode": "manual"
        }
      }
    }
  ],
  "diff": "tasks.smoke.targets.api\n- <absent>\n+ service: ...",
  "validation": [
    "ota validate /abs/path/to/ota.yaml",
    "ota execution topology /abs/path/to/ota.yaml"
  ],
  "next": "rerun with `ota assist bind-task --task smoke --target api --to dev:http --address-view topology --activation manual --write /abs/path/to/ota.yaml` to apply this task binding"
}
```

Notes:

- `subject.task` and `subject.target` identify the consumer task and the target key being changed
- `inputs.to` is normalized to the explicit `<producer>:<listener>` shape even when preview inferred the listener from a single-listener producer
- `changes[*].before` is present when assist is refining an existing target binding
- `changes[*].after` is the exact target block Ota would write, including `activation.mode` and any explicit `override_input`
- validation includes `ota execution topology` because target bindings change the declared execution graph directly

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "operation": "bind-task",
  "subject": {
    "task": "smoke",
    "target": "api"
  },
  "why": "producer task `dev` declares multiple listeners, so assist cannot pick one safely",
  "next": "rerun with `--to <task>:<listener>` after checking `ota execution topology`"
}
```

## `ota assist declare-env --json`

Assist env output reports one deterministic proposal or apply result for one root env requirement,
one declared env source, or one explicit task-local env override.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "preview",
  "operation": "declare-env",
  "subject": {
    "kind": "root_var",
    "name": "APP_PORT"
  },
  "inputs": {
    "required": "true",
    "default": "8080"
  },
  "assumptions": [
    "root env requirement `APP_PORT` will be declared under `env.vars`"
  ],
  "changes": [
    {
      "path": "env.vars.APP_PORT",
      "action": "set",
      "before": null,
      "after": {
        "required": true,
        "default": "8080"
      }
    }
  ],
  "diff": "env.vars.APP_PORT\n- <absent>\n+ required: true ...",
  "validation": [
    "ota validate /abs/path/to/ota.yaml",
    "ota env /abs/path/to/ota.yaml",
    "ota doctor /abs/path/to/ota.yaml"
  ],
  "next": "rerun with `ota assist declare-env --name APP_PORT --required true --default 8080 --write /abs/path/to/ota.yaml` to apply this env change"
}
```

Notes:

- `subject.kind` distinguishes `root_var`, `source`, and `task_env`
- `subject.task` is present only for task-local env writes
- `subject.source_kind` and `subject.source_path` are present only for declared env sources
- `inputs` records only the explicit assist inputs supplied for that one mutation
- validation uses `ota env` or `ota env --task <name>` because env declaration changes should be reviewed through the same read path Ota already trusts

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "operation": "declare-env",
  "subject": {
    "task": "smoke",
    "name": "API_BASE"
  },
  "why": "task-local env declaration needs `--value`",
  "next": "rerun with `--task <name> --name <ENV> --value <value>`"
}
```

## `ota assist add-task --json`

Assist add-task output reports one deterministic proposal or apply result for one new
`tasks.<name>` declaration.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "preview",
  "operation": "add-task",
  "subject": {
    "task": "dev"
  },
  "inputs": {
    "kind": "service",
    "run": "npm run dev",
    "internal": "false",
    "listener": "http",
    "protocol": "http",
    "address": "127.0.0.1",
    "port": "3000"
  },
  "assumptions": [
    "assist adds only one new task and does not infer env, targets, or readiness in this slice",
    "service task creation only declares one fixed listener and matching host projection; declare readiness separately if the app needs deeper truth"
  ],
  "changes": [
    {
      "path": "tasks.dev",
      "action": "set",
      "before": null,
      "after": {
        "run": "npm run dev",
        "runtime": {
          "kind": "service",
          "listeners": {
            "http": {
              "protocol": "http",
              "bind": {
                "address": "127.0.0.1",
                "port": {
                  "mode": "fixed",
                  "value": 3000
                }
              },
              "project": {
                "host": {
                  "address": "127.0.0.1",
                  "port": {
                    "mode": "fixed",
                    "value": 3000
                  },
                  "primary": true
                }
              }
            }
          }
        }
      }
    }
  ],
  "diff": "tasks.dev\n- null\n+ run: npm run dev ...",
  "validation": [
    "ota validate /abs/path/to/ota.yaml",
    "ota tasks /abs/path/to/ota.yaml",
    "ota execution topology /abs/path/to/ota.yaml"
  ],
  "next": "rerun with `ota assist add-task --name dev --kind service --run 'npm run dev' --internal false --listener http --protocol http --address 127.0.0.1 --port 3000 --write /abs/path/to/ota.yaml` to apply this task change"
}
```

Notes:

- `subject.task` is always the newly created task name
- `inputs.kind` is one of `command`, `service`, `setup`, `check`, or `sandbox`
- `inputs.run` or `inputs.script` records the explicit execution body; sandbox can use the bounded `echo sandbox` starter body
- `inputs.listener`, `inputs.protocol`, `inputs.address`, and `inputs.port` appear only for `service`
- `changes[0].before` is always `null` because this slice creates only new tasks

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "operation": "add-task",
  "subject": {
    "task": "smoke"
  },
  "why": "task `smoke` is already declared",
  "next": "choose a new task name, or use `ota tasks` to inspect the current inventory"
}
```

## `ota assist normalize --json`

Assist normalize output reports one deterministic proposal or apply result for moving one existing
task into the canonical `tasks.setup` slot.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "preview",
  "operation": "normalize",
  "subject": {
    "task": "bootstrap",
    "into": "setup"
  },
  "inputs": {
    "into": "setup"
  },
  "assumptions": [
    "`tasks.bootstrap` will move into the canonical `tasks.setup` slot",
    "`tasks.setup` will be normalized to `internal: true` so setup stays an `ota up` support task by default"
  ],
  "changes": [
    {
      "path": "tasks.bootstrap",
      "action": "delete",
      "before": {
        "run": "npm install"
      },
      "after": null
    },
    {
      "path": "tasks.setup",
      "action": "set",
      "before": null,
      "after": {
        "run": "npm install",
        "internal": true
      }
    }
  ],
  "diff": "normalize task bootstrap\n- run: npm install\n+ run: npm install ...",
  "validation": [
    "ota validate /abs/path/to/ota.yaml",
    "ota up --dry-run /abs/path/to/ota.yaml",
    "ota doctor /abs/path/to/ota.yaml"
  ],
  "next": "rerun with `ota assist normalize --task bootstrap --into setup --write /abs/path/to/ota.yaml` to apply this normalization"
}
```

Notes:

- `subject.into` is currently fixed to `setup` in the shipped slice
- `changes[0]` records deletion of the original `tasks.<name>` slot
- `changes[1]` records creation of the canonical `tasks.setup` slot
- apply removes the original `tasks.<name>` entry and writes the moved declaration under `tasks.setup`

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "operation": "normalize",
  "subject": {
    "task": "bootstrap"
  },
  "why": "the contract already declares `tasks.setup`",
  "next": "use `ota assist wire-setup` to refine setup instead of normalizing another task into it"
}
```

## `ota assist wire-setup --json`

Assist setup output reports one deterministic proposal or apply result for `tasks.setup`.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "preview",
  "operation": "wire-setup",
  "subject": {
    "task": "setup"
  },
  "inputs": {
    "run": "npm install",
    "services": "postgres"
  },
  "assumptions": [
    "a new `tasks.setup` declaration will be created",
    "new setup tasks default to `category: setup` and `internal: true` unless overridden",
    "setup will execute through a single `run` command",
    "`setup.requires_services` will define the pre-setup service phase: `postgres`"
  ],
  "changes": [
    {
      "path": "tasks.setup",
      "action": "set",
      "before": null,
      "after": {
        "category": "setup",
        "internal": true,
        "run": "npm install",
        "requires_services": ["postgres"]
      }
    }
  ],
  "diff": "tasks.setup\n- <absent>\n+ category: setup ...",
  "validation": [
    "ota validate /abs/path/to/ota.yaml",
    "ota up --dry-run /abs/path/to/ota.yaml",
    "ota doctor /abs/path/to/ota.yaml"
  ],
  "next": "rerun with `ota assist wire-setup --run 'npm install' --service postgres --write /abs/path/to/ota.yaml` to apply this setup change"
}
```

Notes:

- `subject.task` is always `setup`
- `inputs` records only the explicit setup inputs the operator passed to assist
- `changes[*].before` is present when assist is refining an existing `tasks.setup` block
- `changes[*].after` is the exact setup block Ota would write, including `setup.requires_services` ordering when that input was provided
- validation includes `ota up --dry-run` because setup wiring changes phased repo preparation behavior directly

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "operation": "wire-setup",
  "subject": {
    "task": "setup"
  },
  "why": "creating `tasks.setup` needs an explicit `--run`, `--script`, or copy action body",
  "next": "rerun with `--run '<command>'`, `--script '<body>'`, or `--copy-from <source> --copy-to <target>` to declare the setup task"
}
```

## `ota workspace execution plan --json`

Workspace execution planning stays read-only, but reports one resolved or unresolved execution
decision per selected repo. Repo items may include additive `workflow` and `task` whenever
workflow-aware planning selected a canonical repo path, whether that came from
`repos.<name>.workflow` or the repo contract's own default workflow.

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml",
  "mode": "execution-plan",
  "summary": {
    "repo_count": 2,
    "resolved_count": 1,
    "unresolved_count": 1,
    "required_unresolved_count": 1,
    "not_acquired_count": 0,
    "missing_contract_count": 0
  },
  "overrides": {
    "backend": "container",
    "lifecycle": "ephemeral"
  },
  "repos": [
    {
      "name": "api",
      "path": "/abs/path/to/services/api",
      "contract_path": "/abs/path/to/services/api/ota.yaml",
      "required": true,
      "acquired": true,
      "status": "RESOLVED",
      "workflow": "backend",
      "task": "dev",
      "contract_identity": {
        "version": 1,
        "project": {
          "name": "api"
        },
        "counts": {
          "runtimes": 0,
          "tools": 0,
          "env": 0,
          "services": 0,
          "checks": 0,
          "tasks": 1
        }
      },
      "declared_execution": {
        "preferred": "remote",
        "supported": ["remote"],
        "lifecycle": "ephemeral",
        "backends": {
          "remote": {
            "provider": "ssh",
            "target": "user@host",
            "cwd": "/srv/api"
          }
        }
      },
      "resolved": {
        "backend": "remote",
        "backend_source": "contract preferred",
        "lifecycle": "ephemeral",
        "lifecycle_source": "contract lifecycle",
        "provider": "ssh",
        "target": "user@host",
        "cwd": "/srv/api",
        "target_strategy": "remote target"
      }
    },
    {
      "name": "db",
      "path": "/abs/path/to/services/db",
      "contract_path": "/abs/path/to/services/db/ota.yaml",
      "required": true,
      "acquired": true,
      "status": "UNRESOLVED",
      "error": "`ota execution plan` requires `execution.backends.container.image` for container execution",
      "next": "repair `/abs/path/to/services/db/ota.yaml` so the selected execution mode is runnable, then rerun `ota workspace execution plan`"
    }
  ]
}
```

## `ota tasks --json`

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "workflow": {
    "name": "app",
    "intent": "local_development",
    "notes": "Use this path when validating readiness and preparing local app runs.\n",
    "prepare_task": "setup:env:local",
    "setup_task": "setup",
    "run_task": "dev",
    "required_services": ["postgres"],
    "readiness_checks": ["app-health"],
    "readiness_probes": ["app-ready"],
    "exposes": ["http://127.0.0.1:5678"]
  },
  "agent": {
    "entrypoint": "setup",
    "safe_tasks": ["setup", "test"],
    "verify_after_changes": ["test"],
    "writable_paths": ["src", "docs"],
    "protected_paths": ["Cargo.lock", "LICENSE"],
    "inferred_boundary_reviewed": false,
    "bootstrap": {
      "ota": {
        "note": "Only install ota if it is missing and installation is approved.",
        "source": {
          "kind": "version",
          "version": "v1.6.16",
          "deterministic": true,
          "pressure_only": false
        },
        "sh": "curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION=v1.6.16 sh",
        "powershell": "$env:OTA_VERSION='v1.6.16'; irm https://dist.ota.run/install.ps1 | iex"
      }
    }
  },
  "capability_profile": {
    "actor_mode": "agent",
    "writable_paths": ["src", "docs"],
    "protected_paths": ["Cargo.lock", "LICENSE"],
    "callable_tasks": [
      {
        "lane_id": "task:test",
        "lane_kind": "task",
        "name": "test",
        "command": "ota run test --agent",
        "environment_boundary": {
          "kind": "task",
          "backend": "native",
          "default_mode": "native"
        },
        "preflight": {
          "state": "allowed",
          "review_required": false,
          "declared_safe_for_agent": true,
          "effective_safe_for_agent": true,
          "crossing_required": false,
          "crossing_classification": "routine",
          "receipt_expected": true,
          "proof_expected": true
        },
        "effect_policy": {
          "status": "not_applicable",
          "provider_execution": "not_applicable"
        }
      }
    ],
    "refused_tasks": [
      {
        "lane_id": "task:setup",
        "lane_kind": "task",
        "name": "setup",
        "command": "ota run setup --agent",
        "environment_boundary": {
          "kind": "task",
          "backend": "native",
          "default_mode": "native"
        },
        "preflight": {
          "state": "refused",
          "review_required": true,
          "declared_safe_for_agent": false,
          "effective_safe_for_agent": false,
          "refusal_reason_family": "requested_task_not_safe",
          "refusal": {
            "reason_family": "requested_task_not_safe",
            "boundary_family": "agent_safety_boundary",
            "closure_status": "unsafe",
            "requested_task": "setup",
            "blocked_task": "setup",
            "evidence_class": "derived"
          },
          "receipt_expected": true,
          "proof_expected": false
        },
        "effect_policy": {
          "status": "not_applicable",
          "provider_execution": "not_applicable"
        }
      }
    ]
  },
  "tasks": [
    {
      "name": "setup",
      "use": {
        "human": "ota run setup --base-url <value>",
        "agent": {
          "callable": false,
          "reason": "not_safe"
        }
      },
      "description": "Prepare mixed repo dependencies",
      "notes": "Use this after cloning the repo.\n",
      "kind": "sequence",
      "prepare": {
        "kind": "sequence",
        "steps": [
          {
            "kind": "dependency_hydration",
            "medium": "package_dependencies",
            "source_kind": "node_package_manager",
            "cwd": ".",
            "manager": "pnpm",
            "mode": "install",
            "frozen_lockfile": true
          },
          {
            "kind": "dependency_hydration",
            "medium": "package_dependencies",
            "source_kind": "uv",
            "cwd": "api",
            "manager": "uv",
            "mode": "sync"
          }
        ]
      },
      "env": {
        "JAVA_HOME": "/opt/jdk-21"
      },
      "inputs": {
        "base_url": {
          "required": true
        }
      },
      "effects": {
        "writes": ["node_modules", ".venv"],
        "network": true,
        "network_kind": "dependency_hydration"
      },
      "depends_on": [],
      "requires_services": ["postgres"],
      "requires_artifacts": [],
      "safe_for_agent": false,
      "effective_safe_for_agent": false
    },
    {
      "name": "test",
      "use": {
        "human": "ota run test",
        "agent": {
          "callable": true,
          "command": "ota run test --agent"
        },
        "modes": [
          {
            "mode": "container",
            "default": false,
            "availability": "unavailable",
            "reason": "not_supported_by_task",
            "human": {
              "callable": false,
              "reason": "not_supported_by_task"
            },
            "agent": {
              "callable": false,
              "reason": "not_supported_by_task"
            }
          },
          {
            "mode": "native",
            "default": true,
            "availability": "supported",
            "human": {
              "callable": true,
              "command": "ota run test"
            },
            "agent": {
              "callable": true,
              "command": "ota run test --agent"
            }
          }
        ]
      },
      "kind": "command",
      "command": {
        "exe": "uv",
        "args": ["run", "pytest"]
      },
      "depends_on": ["setup"],
      "requires_services": [],
      "requires_artifacts": [],
      "safe_for_agent": true,
      "effective_safe_for_agent": true
    }
  ]
}
```

When ota can explain a refusal from the execution decision site, governance output now carries the
legacy `refusal_reason_family` plus additive structured `refusal` truth. That record keeps the
requested lane, blocked lane, closure posture, boundary family, optional closure path, and the
evidence class together so machine consumers do not have to reconstruct the refusal from flat
status fields.

`ota tasks --json` may also include additive top-level `capability_profile`, a derived
harness-facing agent surface built from the same closure-aware safety and refusal truth that powers
`ota run --agent`.

`tasks[].requires_artifacts` names generated artifacts consumed by that task. Their producer,
declared output paths, and source inputs remain canonical under top-level `artifacts` in the
contract; a non-empty list means the task must directly depend on each artifact producer.

Each task summary now also carries a canonical `use` object:

- `use.human` and `use.agent` remain compatibility projections for the selected default mode
- `use.agent.callable` reflects contract safety only. When `capability_profile` is present, its
  policy/provider-aware `preflight` and `effect_policy` fields are authoritative for actual harness
  callability
- `use.modes[]` is the canonical task-mode matrix for machine consumers; it lists `container`,
  `native`, and when advertised `remote` in a stable order
- every `use.modes[]` entry carries `default`, `availability`, and separate human/agent callable
  truth, including the exact command when callable
- aggregate task entries derive mode availability from their concrete dependency closure: a mode is
  supported only when every executable member supports that backend on the current platform

Receipt `evaluated_inputs[]` also includes task-declared `replay_inputs` captured before execution.
These records use `kind: static_file` and `input_class: declared_replay_input`; matching identities
only prove the named file held still and remain narrowing evidence rather than a hermetic replay
claim. When the contract declares `expected_identity`, the record retains both the observed
`identity` and `expected_identity`. A missing or mismatched pin does not begin execution: Ota emits
a blocked preconditions receipt with `failure_origin: replay_input_identity_missing` or
`replay_input_identity_mismatch`, one evaluated input containing the expected identity, the
observed identity when readable, plus `execution_started: false`, and the matching
`OTA_REPLAY_INPUT_IDENTITY_MISSING` or
`OTA_REPLAY_INPUT_IDENTITY_MISMATCH` finding.
When an active org policy requires replay-input identities for the selected lane, Doctor, run
preview, and admission-produced run/up execution or refusal receipts include additive
`replay_input_policy`. Generic `ota receipt --json` readiness receipts do not reconstruct a prior
admission decision after execution. The policy record carries:

- `subject`: the selected task or workflow;
- `required`, `coverage`, and the derived `decision` (`allow`, `review`, or `deny`);
- `applicable_rules[]`: every matching task/workflow rule, its own selected closure, coverage,
  decision, and typed insufficiency reasons;
- `inputs[]`: one task-qualified record per observed declared input, including expected and
  observed identities where available; and
- `unknown_selectors[]`: contextual policy selectors that do not resolve against the loaded
  contract.

`review` is not an allowed execution result in this slice. Both `review` and `deny` refuse before
native prerequisite provisioning, dependency hydration, or task startup. An unavailable
observation fails closed. An unreadable or mismatched declared pin always derives `deny`,
regardless of a rule's `on_insufficient` value, and its blocked receipt retains the active policy
record. Agent safety, claim assurance, replay admission, Doctor/provisioning findings, proof, and
receipt policy evidence consume one loaded policy snapshot for the command. Runtime proof pins the
same authority for its detached child. CI projection evaluates all governance domains from one
loaded snapshot and carries only the active semantic policy identity,
applicable rule identities, the canonical execution closure including recursive outcome hooks,
and unresolved selector identities; provider execution recomputes observed identities after
checkout rather than trusting the machine that rendered the projection.
An active policy source that cannot be loaded refuses as
`replay_input_policy_unavailable`; it never degrades to absent policy. Task and workflow
admission includes recursive outcome-hook execution edges, so hook replay inputs and matching task
rules are observed and enforced before the parent begins.
`ota proof runtime` captures one replay-input preflight over the full selected proof closure,
including post-readiness seam observers and the selected negative-control task, and admits it
before creating proof artifacts or spawning the child runtime. It then reuses that preflight across
readiness diagnosis and the emitted Doctor artifact; the artifact does not re-read the checkout
after runtime execution.
`ota proof lifecycle` resolves its explicit selected-service set and dependency closure, then
evaluates the exact workflow prerequisite-plus-assertion closure before creating its transaction
or running any task, service command, readiness observation, or assertion. Both proof commands
emit `execution_started: false` with typed hard-pin and policy evidence when admission refuses.
Aggregate monorepo Doctor JSON carries the same canonical `replay_input_policy` record inside each
applicable `members[]` result rather than dropping the member's policy identity and observed-input
status.

When `governance.crossing_authority` governs a selected non-agent proof workflow, both proof
commands require `--grant <id>` before any proof-owned side effect. Their refusal payloads carry
`crossing_grant_admission` with a stable reason family, typed semantic scope, and
`execution_started: false`; the public `error` and refusal detail are deliberately bounded and do
not expose protected authority-store, bundle, or sequence-state paths. Runtime proof scope always
includes its carrier, every selected seam/control invocation by role and declaration order, and the
normalized `--ready-timeout` selection. Lifecycle proof scope includes its selected-service closure
and lifecycle assertion. Both proof commands retain one proof-owned crossing transaction across
their complete invocation sets. Runtime authority covers the detached workflow/task path, ordered
seam observers, negative controls, and cleanup. Lifecycle authority covers prerequisites, selected
services and dependency closure, assertion, and cleanup. Runner-private authority travels only
between Ota processes over a bounded Unix descriptor and is removed before selected code executes.
Their terminal archives embed and re-derive the exact carrier admission and transaction rather
than inheriting an ordinary workflow grant or trusting a child-produced claim.
When an active org policy pack participates in the selected lane, receipt capture also adds
`kind: policy_ruleset_identity` with `input_class: policy_ruleset_identity` so replay can treat
policy/ruleset drift as named input drift instead of leaving governance movement ambient.
When the selected lane actually resolves a declared env source, receipt capture also adds
`kind: env_source_identity` with `input_class: declared_env_source_identity`. This hashes the
source file itself, never the resolved env values, so replay can classify declared env-source drift
as named input drift without leaking secrets or overclaiming process-env identity.

Receipt `witnessed_observations.query_traces[]` carries contract-declared historical query evidence
separately from `evaluated_inputs[]`. Each trace is `evidence_class: attested`, retains its source
path, source SHA-256 identity, and per-subject/per-run identity records, and summarizes divergent subjects. Query
identities are observed behavior, not current-run decision inputs; a divergence identifies a
changed query shape without claiming model causality or a negative-control result.

A baseline producer receipt carries its own Ota-recorded attestation reference in
`witnessed_observations.replay_baseline_recordings[]`. Each record is `evidence_class: attested`
and binds the artifact, producer task, actual scope/backend/lifecycle, attestation identity, and
local archive path. Promotion verifies this reference and the receipt evidence identity before it
can select the attestation; replay consumer receipts carry the separate promoted authority below.
The attestation also requires a V11.16 execution-boundary graph identity and identities for both
selected closures. An empty graph is explicit `unknown` runner evidence, not an omitted claim that
the producer had no material prerequisites.

When a selected task consumes an artifact as replay, receipt `evaluated_inputs[]` emits
`kind: promoted_replay_baseline` and `input_class: promoted_replay_baseline` only after Ota has
verified the current declared outputs against the portable promoted authority manifest. Its
`artifact_lineage.replay_authority` binds the receipt to the manifest path, explicit SCM-review
trust root, selected attestation identity, promotion identity, and declared consumption posture.
`read_only` means the selected closure ran with a runner-owned snapshot outside the writable
workspace. `verify_unchanged` means Ota verified the complete output set before and after
execution; `replay_artifact_mutation_detected` reports a changed baseline rather than claiming
the write was refused. This is selected replay authority, not a claim that the regeneration
producer ran in the replay lane.

For selected structured hydration lanes, `ota up --json` also emits a typed
`receipt.evaluated_inputs[]` record with `kind: hydration_provenance`. Its runner-derived nested
detail retains the contract-owned declared posture separately from the execution-time resolved
posture. For `.NET restore` and `uv`, the resolved record may include typed
`source_identities[]`, or an explicit unavailable resolution when source selection would require
ambient configuration. `uv` additionally carries its declared `offline` posture; cache-only
execution does not turn an undeclared cache source into resolved provenance.
For `uv` `mode: pip_local_project`, the same record carries `local_project`: the declared path,
editable posture, extras, and ordered groups together with runner-observed manifest and optional
lockfile identities. `source_identity` is present only when Ota can recover a clean Git identity
for that local project; `source_identity_error` keeps a dirty or unavailable source explicit
instead of treating the manifest hash as a source pin.
Consumers must not replace this captured input with a later config-file read when evaluating the
execution receipt. During replay, an unavailable hydration resolution remains `narrowing` evidence
and cannot make a lane hermetic. For an editable local project, the named dependency-resolution
class is acquitting only when both receipts carry resolved hydration posture plus matching declared
lockfile and clean source identities; a manifest identity alone only narrows the comparison.

- unavailable local planes remain explicit with `availability: "unavailable"` and
  `reason: "not_supported_by_task"`; this describes contract support, not current machine
  readiness, which remains the responsibility of `ota doctor` and `ota run --dry-run`
- `use.modes[].agent.callable: false` with `reason: "not_safe"` means the mode is declared but
  its full task closure is not agent-callable, without forcing consumers to scrape human text

Each task capability entry carries:

- stable lane identity via `lane_id` and `lane_kind`
- the canonical agent invocation in `command`
- an additive `environment_boundary`
- canonical `preflight` governance state
- an additive `sandbox_policy` for the first compiled runtime target, `codex_local`
- mandatory `effect_policy` posture. Untyped lanes report `status: "not_applicable"`. Typed lanes
  report either an identity-bound evaluated decision or `status: "unavailable"`; both retain
  `provider_execution: "disabled"` while the V12 provider adapter is unavailable, so a generic
  agent-safety allow cannot make the lane callable
- additive closure effect posture in `effects` when the selected task path owns network, write,
  adapter, or external-state behavior

`sandbox_policy` is still intentionally narrow in this first slice:

- `filesystem.state: "compiled"` means ota could derive a read-only repo posture plus
  writable/protected carve-outs from declared `agent.writable_paths` and
  `agent.protected_paths`
- `filesystem.source: "execution.runtime_boundary"`, `"workflows.<name>.runtime_boundary"`, or
  `"tasks.<name>.runtime_boundary"` means the lane publishes canonical runtime-boundary truth and
  ota is compiling that selected-path owner directly instead of falling back to derived agent
  boundary posture
- `filesystem.state: "unavailable"` means the lane does not yet carry enough declared agent
  boundary truth for ota to claim a trustworthy writable-mount policy
- `network.default: "deny"` with `scope: "none"` means the lane does not declare network use on
  its effect surface
- `network.default: "allow"` with `scope: "broad"` means the lane declares network use, but ota
  is still honestly compiling only the broad effect-owned posture here rather than a host or
  destination allowlist
- `network.scope: "targeted"` plus `outbound_targets[]` means the selected lane declares explicit
  runtime-boundary target truth
- `outbound_targets[].destination_constraint` means the lane declares narrower effective
  destination truth beyond the first-hop host; ota now preserves:
  - constraint `kind`
  - constrained `values[]`
  - `source_posture`
  - `enforcement`
  - optional `shared_pin.ref` plus `shared_pin.freshness` when the lane consumes pinned shared
    destination truth
- top-level `network.enforcement` still describes the broad compiled lane posture; per-target
  destination constraints can carry stronger app/runtime enforcement truth than the coarse lane
  default

V11.21 adds execution admission and runner-authored application evidence without relabeling the
older `codex_local` profile as enforcement:

- task/workflow dry-run JSON carries `sandbox_admission` with the canonical policy, explicit
  restriction-overlay identities, effective policy, provider capability decision, and typed
  refusals
- `oci_local` admission is valid only for an explicit ephemeral container platform whose
  authoritative filesystem/network controls the provider can enforce without widening
- canonical segments expose `execution_kind` and any `pre_boundary_actions`. The first OCI adapter
  refuses typed bodies, requirements, services, or conditional checks that would run outside the
  segment boundary instead of omitting them from the policy identity
- completed execution receipts carry
  `witnessed_observations.sandbox_application`, including the selected lane, execution selection,
  admitted segment/edge graph, per-invocation purpose (`task_execution` or
  `precondition_probe`), boundary identities, initial/terminal application identities, the
  complete admitted mount-set identity, canonical writable-mount source identities, and cleanup
  state. Provider-backed precondition probes are separate evidenced invocations of the exact
  admitted segment that owns the requirement. Blocking probes retain terminal cleanup evidence in
  the refusal receipt and cannot stand in for the matching task execution outcome.
- when an organization policy narrows the lane, `restriction_authority` carries the identified
  command-scoped policy source, the re-derivable semantic identity of its exact sandbox rules, and
  those rules as an immutable snapshot. The
  `restriction_overlays[]` projection must derive from that snapshot; an overlay without matching
  authority is invalid
- `enforced_through_completion` is derived only when every started boundary retained its admitted
  controls through terminal engine inspection and engine-confirmed removal
- provider-backed receipts are archived with their normalized contract snapshot. Receipt-history
  loading re-derives canonical policy from that snapshot and restriction overlays from the
  archived policy-authority snapshot, then rejects mismatched effective policy, segment order,
  conditional edges, capability identity, application-plan identity, or completed segment
  invocations that do not match archived task outcomes
- this record proves only the selected cooperating runtime boundary. It does not prove application
  output, merge eligibility, host-wide isolation, or execution performed through raw shell outside
  Ota

Refused task capability entries may also carry additive `blocked_task` and `closure_path` when a
declared-safe task is refused because its reachable closure leaves the safe surface.

Root monorepo summary output can also include grouped member results:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "tasks": [],
  "members": [
    {
      "member": "api",
      "workflow": {
        "name": "app",
        "run_task": "test",
        "required_services": [],
        "readiness_checks": [],
        "readiness_probes": [],
        "exposes": []
      },
      "tasks": [
        {
          "name": "test",
          "kind": "run",
          "run": "cargo test",
          "notes": "Use this to verify the code before merging.\n",
          "env": {
            "BASE_URL": "http://localhost:8080"
          },
          "inputs": {
            "mode": {
              "default": "live"
            }
          },
          "effects": {
            "network": true
          },
          "depends_on": [],
          "requires_services": ["postgres"],
          "safe_for_agent": false,
          "effective_safe_for_agent": false
        }
      ]
    }
  ]
}
```

Each task may also include additive `prepare` when the resolved task body is first-class setup
instead of shell `run` / `script`. Sequence prepares keep a nested `steps[]` tree, while
dependency-hydration prepares expose structural fields such as `medium`, `source_kind`, `cwd`,
`manager`, and `mode`.
Each task may also include additive `effects` when the contract declares durable repo writes
(`writes`), workspace/sibling writes (`workspace_writes`), a connectivity dependency (`network`),
an optional network lane classification (`network_kind`), or out-of-repo mutation
(`external_state`).
When the repo declares `workflows`, `ota tasks --json` includes an additive top-level `workflow`
object for the default workflow, and member summaries may include the same additive field.

## `ota workflows --json`

Workflow inventory stays read-only. It reports declared repo workflows without falling back to the
full task inventory.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "default": "app",
  "capability_profile": {
    "actor_mode": "agent",
    "callable_workflows": [
      {
        "lane_id": "workflow:quickstart",
        "lane_kind": "workflow",
        "name": "quickstart",
        "command": "ota up --workflow quickstart --agent",
        "environment_boundary": {
          "kind": "workflow",
          "backend": "container",
          "context": "app",
          "primary_task": "preview:quickstart"
        },
        "preflight": {
          "state": "allowed",
          "review_required": false,
          "declared_safe_for_agent": true,
          "effective_safe_for_agent": true,
          "crossing_required": false,
          "crossing_classification": "routine",
          "receipt_expected": true,
          "proof_expected": true
        }
      }
    ]
  },
  "workflows": [
    {
      "name": "quickstart",
      "use": {
        "human": "ota up --workflow quickstart",
        "agent": {
          "callable": true,
          "command": "ota up --workflow quickstart --agent"
        }
      },
      "intent": "quickstart",
      "description": "Structured packaged command path",
      "notes": "Use this path for local container-backed previews.\n",
      "prepare_task": "setup:env:local",
      "setup_task": "setup",
      "run_task": "preview:quickstart",
      "run_task_launch": {
        "kind": "command",
        "exe": "npx",
        "args": ["vite", "--host", "127.0.0.1", "--port", "3000"]
      },
      "required_services": ["postgres"],
      "readiness_checks": [],
      "readiness_probes": [],
      "readiness_surfaces": ["backend"],
      "signal_readiness_checks": [],
      "signal_readiness_probes": [],
      "signal_readiness_surfaces": [],
      "exposes": ["http://127.0.0.1:5678/"],
      "expose_surfaces": ["backend"],
      "default": true
    },
    {
      "name": "backend",
      "use": {
        "human": "ota up --workflow backend",
        "agent": {
          "callable": false,
          "reason": "not_safe"
        }
      },
      "run_task": "backend",
      "required_services": [],
      "readiness_checks": [],
      "readiness_probes": [],
      "readiness_surfaces": ["backend"],
      "signal_readiness_checks": [],
      "signal_readiness_probes": [],
      "signal_readiness_surfaces": [],
      "exposes": ["http://127.0.0.1:5678/"],
      "expose_surfaces": ["backend"],
      "default": false
    }
  ]
}
```

Notes:

- root success includes `ok`, `path`, optional `default`, and `workflows`
- each workflow summary carries a canonical `use` object with explicit human and agent
  invocations; non-callable agent lanes publish `callable: false` instead of forcing consumers to
  infer that from separate safety fields
- root success may also include additive `capability_profile`, a derived harness-facing workflow
  surface for agent mode
- `capability_profile.callable_workflows[]` and `capability_profile.refused_workflows[]` publish
  exact workflow lane identity, canonical `ota up --workflow ... --agent` commands, workflow
  environment boundaries, and canonical preflight governance state without requiring a harness to
  scrape human output or guess workflow closure safety
- workflow capability entries may also include additive `sandbox_policy` for the first compiled
  runtime target, `codex_local`, using the same semantics as task capability entries: filesystem
  policy compiles from declared runtime-boundary or derived agent boundary truth when present, and
  network policy currently compiles as either effect-owned `deny/none` / `allow/broad` or
  targeted runtime-boundary `outbound_targets[]`
- workflow capability entries use the same `effect_policy` binding as task entries. The binding
  carries decision, policy-snapshot, selected execution-graph, and effect-set identities without
  claiming provider execution. Only `not_applicable` untyped lanes may appear under
  `callable_tasks[]` or `callable_workflows[]`; typed lanes remain under the matching refused
  collection until execution is implemented. Evaluated `deny`, evaluated `allow`/`warn`, and
  unavailable posture each carry their corresponding refused preflight reason.
- workflow capability entries publish `preflight.proof_expected: true` because `ota up` is a
  proof-owning lane: the selected workflow is expected to drive readiness or runtime-proof
  evidence when executed
- workflow capability and run-preview preflight entries can also publish thin audited-crossing
  posture:
  - `crossing_required: false` with `crossing_classification: "routine"` means the lane stays
    inside the routine default-safe path
  - `crossing_required: true` with `crossing_classification: "escalated"` means the lane is
    allowed but crosses a heavier execution boundary such as an unsafe task closure
  - `crossing_boundary_family` names the crossed boundary when ota can recover it honestly
  - refused lanes do not publish crossing posture yet because no crossing is allowed
- `preflight.decision_basis[]` is the additive machine-readable citation set for the current
  governance outcome:
  - safety gates such as `agent_safe_closure`, `unsafe_closure`, or
    `declared_safe_closure_unsafe`
  - refusal basis such as `refusal:requested_task_not_safe`
  - crossing gates such as `crossing_not_required:routine` or
    `crossing_required:unsafe_task:escalated`
  - each entry carries stable `id`, `family`, and `evidence_class` instead of relying on prose
- authoritative crossing records in `receipt.crossing` and `governance.crossing` can also publish
  additive `evidence_classes`:
  - `asserted` means the caller supplied the value, such as free-text `reason`
  - `derived` means ota resolved the value from contract truth and decision inputs, such as lane,
    boundary, or classification
  - `attested` means ota recorded the field at the decision boundary itself, such as
    `reason_present`, `principal_attribution_state`, or attachment state
  - optional `authority` means the selected signed grant was verified before execution against
    the contract-bound fixed trust store. Its `archive_evidence` is re-verified with the archived
    contract snapshot and current fixed trust binding; it is not reusable authorization
- refused workflow entries may also carry additive `blocked_task` and `closure_path` when the
  selected workflow reaches a non-safe task in its prepare/setup/run/attach closure
- each workflow entry includes additive fields only when declared or resolved:
  - `intent`
  - `description`
  - `notes`
  - `prepare_task`
  - `prepare_action`
  - `setup_task`
  - `run_task`
  - `run_task_launch`
  - `required_services`
  - `readiness_checks`
  - `readiness_probes`
  - `readiness_surfaces`
  - `signal_readiness_checks`
  - `signal_readiness_probes`
  - `signal_readiness_surfaces`
  - `exposes`
  - `expose_surfaces`
  - `default`
- `exposes` contains resolved URL strings; `expose_surfaces` preserves the named surface refs that
  produced them
- `required_services` includes both workflow-declared `services.required` and transitive task-level
  `requires_services` from the selected workflow task closure
- `run_task_launch` preserves the selected run task's structured launch source when that workflow
  path runs through `launch` instead of shell `run` or `script`
- when the target is a monorepo root and members are requested, success may include additive
  top-level `members`, each with `member`, optional `default`, and `workflows`

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "error": "contract path does not exist: /abs/path/to/ota.yaml"
}
```

## `ota doctor --json`

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "error_count": 0,
    "warn_count": 1,
    "info_count": 0
  },
  "workflow": {
    "name": "app",
    "intent": "local_development",
    "prepare_task": "setup:env:local",
    "setup_task": "setup",
    "run_task": "dev",
    "required_services": ["postgres"],
    "readiness_checks": ["app-health"],
    "readiness_probes": ["app-ready"],
    "exposes": ["http://127.0.0.1:5678"]
  },
  "agent": {
    "entrypoint": "setup",
    "verify_after_changes": ["test"],
    "protected_paths": ["ota.yaml"],
    "inferred_boundary_reviewed": false,
    "bootstrap": {
      "ota": {
        "note": "Only install ota if it is missing and installation is approved.",
        "source": {
          "kind": "version",
          "version": "v1.6.16",
          "deterministic": true,
          "pressure_only": false
        },
        "sh": "curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION=v1.6.16 sh",
        "powershell": "$env:OTA_VERSION='v1.6.16'; irm https://dist.ota.run/install.ps1 | iex"
      }
    }
  },
  "governance": {
    "required_verification_lanes": [
      {
        "merge_check_id": "ota.verify.verify",
        "lane_task": "verify",
        "lane_kind": "aggregate",
        "contract_sources": ["workflows.verify.run.task"],
        "evidence_classes": {
          "merge_check_id": "derived",
          "lane_task": "derived",
          "lane_kind": "derived",
          "contract_sources": "derived"
        }
      }
    ],
    "merge_gate": {
      "state": "projected",
      "blocking": false,
      "required_lane_count": 1,
      "drift_lane_count": 0,
      "evidence_classes": {
        "state": "derived",
        "blocking": "derived",
        "required_lane_count": "derived",
        "drift_lane_count": "derived",
        "decision_inputs": "derived",
        "replay": "derived"
      },
      "decision_basis": [
        {
          "id": "projection:ota.verify.verify",
          "family": "required_lane",
          "evidence_class": "derived",
          "detail": "verify"
        }
      ],
      "decision_inputs": [
        {
          "id": "decision_owner:doctor_merge_gate_summary",
          "family": "decision_owner",
          "evidence_class": "derived",
          "replay_class": "pinned"
        },
        {
          "id": "required_lane_count:1",
          "family": "merge_gate",
          "evidence_class": "derived",
          "replay_class": "witnessed"
        },
        {
          "id": "drift_lane_count:0",
          "family": "merge_gate",
          "evidence_class": "derived",
          "replay_class": "witnessed"
        }
      ],
      "replay": {
        "status": "satisfied"
      },
      "lanes": [
        {
          "merge_check_id": "ota.verify.verify",
          "lane_task": "verify",
          "lane_kind": "aggregate",
          "state": "projected",
          "blocking": false,
          "evidence_classes": {
            "merge_check_id": "derived",
            "lane_task": "derived",
            "lane_kind": "derived",
            "state": "derived",
            "blocking": "derived",
            "decision_inputs": "derived",
            "replay": "derived",
            "contract_sources": "derived",
            "provider_sources": "derived"
          },
          "decision_basis": [
            {
              "id": "projection:ota.verify.verify",
              "family": "required_lane",
              "evidence_class": "derived",
              "detail": "verify"
            }
          ],
          "decision_inputs": [
            {
              "id": "decision_owner:doctor_merge_gate_lane",
              "family": "decision_owner",
              "evidence_class": "derived",
              "replay_class": "pinned"
            },
            {
              "id": "merge_check_id:ota.verify.verify",
              "family": "merge_lane",
              "evidence_class": "derived",
              "replay_class": "pinned"
            },
            {
              "id": "drift_detected:false",
              "family": "provider_drift",
              "evidence_class": "derived",
              "replay_class": "witnessed"
            }
          ],
          "replay": {
            "status": "satisfied"
          },
          "contract_sources": ["workflows.verify.run.task"],
          "provider_sources": []
        }
      ]
    }
  },
  "provisioning": {
    "allowed": [
      {
        "kind": "runtime",
        "name": "java",
        "requested_version": "22",
        "normalized_requirement": ">=22.0.0 <23.0.0",
        "package": "openjdk-22-jdk",
        "source": "org-mirror",
        "approved_version": "22",
        "policy_match": "22",
        "blocked_reason": null
      },
      {
        "kind": "tool",
        "name": "maven",
        "requested_version": "3.9",
        "normalized_requirement": ">=3.9.0 <3.10.0",
        "source": "approved-manager",
        "approved_version": "3.9",
        "policy_match": "3.9",
        "blocked_reason": null
      }
    ],
    "blocked": [],
    "actions": [
      {
        "kind": "select_source",
        "target_kind": "runtime",
        "name": "java",
        "requested_version": "22",
        "normalized_requirement": ">=22.0.0 <23.0.0",
        "package": "openjdk-22-jdk",
        "source": "org-mirror",
        "approved_version": "22",
        "policy_match": "22"
      },
      {
        "kind": "select_source",
        "target_kind": "tool",
        "name": "maven",
        "requested_version": "3.9",
        "normalized_requirement": ">=3.9.0 <3.10.0",
        "source": "approved-manager",
        "approved_version": "3.9",
        "policy_match": "3.9"
      }
    ]
  },
  "findings": [
    {
      "code": "OTA_TASKS_MISSING",
      "category": "contract",
      "owner": "repo_contract",
      "severity": "warn",
      "summary": "...",
      "why": "...",
      "next": "...",
      "evidence": {
        "observed": "...",
        "expected": "...",
        "source": "...",
        "checked_at": "...",
        "command": "...",
        "path": "..."
      },
      "metadata": {
        "governance": {
          "merge_check_id": "ota.verify.test",
          "lane_task": "test",
          "lane_kind": "task",
          "provider_sources": [
            ".github/workflows/ci.yml#jobs.verify.steps[0].run"
          ]
        }
      }
    }
  ]
}
```

Finding objects always include stable identity fields:
`code`, `category`, `owner`, and `evidence`.

### Contract-claim assurance

`ota doctor --json` may include additive top-level `claim_assurance[]`. This is the first carrier
for V11.14 contract-claim assurance. Each record uses structured subject identity instead of a
delimiter-encoded identifier:

```json
{
  "subject": { "kind": "task", "name": "setup" },
  "family": "agent_safety",
  "declaration": { "value": "safe", "evidence_class": "asserted" },
  "closure": { "status": "safe", "evidence_class": "derived" },
  "assurance": {
    "status": "unknown",
    "coverage": ["contract_declaration", "execution_closure"],
    "gaps": ["non_self_origin_evidence"]
  },
  "policy": {
    "decision": "allow",
    "basis": ["default_compatibility"],
    "evidence_class": "derived"
  }
}
```

The fields deliberately carry separate truths:

- `declaration` is the maintainer assertion.
- `closure` is Ota's derived V11.3 execution-closure result.
- `assurance` is policy-independent evidence posture: `supported`, `contradicted`, or `unknown`.
  Contract declaration and closure alone do not independently corroborate a safety claim, so the
  first carrier emits `unknown` until a later assurance source supplies matching evidence.
  A deterministic typed-action conflict is instead `contradicted`. For example, a declared-safe
  `reset_compose_service_volume` action without the matching
  `effects.adapter_state: [compose_volume:<volume>]` emits a cited contradiction from
  `task.action`; Ota does not make the equivalent claim for opaque shell text.
- `policy` is the derived policy decision over that canonical record. The initial default remains
  `allow` for compatibility and does not change `ota run --agent` admission behavior.

The current carrier emits agent-safety records for tasks declared safe and proof-breadth records
for workflows that declare runtime seam proof or `workflows.<name>.proof.claim: bounded`. A
bounded claim declares an archive-backed proof lane without inventing dependency-seam evidence. A
proof-breadth record is `supported` only when Ota finds
a content-addressed `runtime_proof` archive with matching semantic contract snapshot, clean source
identity, resolved execution scope, and `replay_posture: witness_only`. A matching failed proof is
`contradicted`; absent, stale, changed-source, or scope-mismatched archives remain `unknown`.
Consumers must not infer absent declared-effects records as support or contradiction.

V12 adds `effect_refusal_assurance` records for declared `agent.effect_refusal_canaries`. The
contract-graph carrier is initially `unknown`: it binds the current semantic contract
snapshot and `typed_contract_graph_v1` enumeration scope, then enumerates the declared challenge
lanes, exact effect and attachment identities, and every other declared attachment with the same
canonical effect identity. An unchallenged equivalent attachment is emitted as
`equivalent_execution_paths_not_proved:<attachment-identity>`. Every record also retains
`opaque_execution_paths_not_enumerated`, because Ota does not infer effects from shell, scripts,
or provider configuration. JSON Schema validates the snapshot digest's local shape only; Core
re-derives and reconciles its equality with the current semantic contract. A declaration or
ephemeral canary result cannot make this record
`supported`; an exact workflow challenge can reach `supported` only when the existing private
refusal archive verifies the current semantic contract snapshot, same workflow, exact eligible
effect attachment and realization, explicit typed deny, and `execution_started: false`. Task-only
challenges remain `unknown` because the current archive carrier is workflow-only. A missing,
invalid, stale, ambiguous duplicate, or mismatched archive remains `unknown` rather than being
matched by similarity. Any invalid sibling in the observed private archive set also prevents
support.

When an active org policy pack declares `policies.agent.claim_assurance.<family>`, `policy` is
derived from that requirement. For example, a policy requiring `agent_safety.minimum_status:
supported` changes an otherwise `unknown` record to `policy.decision: deny`; it does not relabel
the assurance itself as contradicted. Agent-mode runner admission consumes this same record when
the strict policy lane is enabled; ordinary agent execution remains backward-compatible without
an assurance requirement.

`ota doctor --json` may also include additive top-level `governance.required_verification_lanes`
when the contract already declares merge-relevant CI verification truth. Ota projects these lanes
from `workflows.*` with `intent: ci_verification` or legacy `ci_validation`, and falls back to
`agent.verify_after_changes` only when no explicit CI verification workflows are declared. Each
projected lane carries the same canonical `merge_check_id` identity that CI verification drift
findings use. `required_verification_lanes[*].evidence_classes` now makes that provenance
explicit; the current shipped lane identity and source fields are all `derived`.
`ota doctor --json` may also include additive `governance.merge_gate`, a first machine-readable
merge-oriented governance verdict built from the same projected lanes and CI drift metadata. This
surface stays honest:

- `state: projected` means Ota has projected the canonical required lanes but is not claiming full
  provider alignment from current recovery alone
- `state: drift_detected` means one or more projected lanes already have CI drift findings attached
  and should be treated as merge-blocking until the workflow wiring is reconciled
- `decision_basis[]` now carries the cited merge-gate basis instead of leaving `state` as a pure
  verdict:
  - `projection:<merge_check_id>` means the lane is projected from contract-owned CI truth
  - `drift:<merge_check_id>` means the same canonical lane already has attached CI drift metadata
- `evidence_classes` now makes the field-level provenance explicit for the authoritative merge
  verdict itself; the current shipped `state`, `blocking`, and lane-count fields are all
  `derived`
- merge-facing governance now also carries additive `decision_inputs[]` with stable
  `decision_owner:<stable-id>` mechanism identity on both the summary and per-lane records, so
  downstream consumers can link the projected merge verdict back to one canonical doctor-side
  decision owner without inventing a second merge model
- both the summary and each lane also carry `replay`, which reconciles the emitted merge-facing
  verdict back to those cited decision inputs and reports `satisfied`, `mismatch`, or
  `unavailable` without inventing a second merge-only command surface
- that replay lane now also checks the trust-sensitive cited-input classes on merge governance, so
  inputs like `decision_owner`, `merge_check_id`, or drift observations cannot silently downgrade
  from their expected `pinned` / `witnessed` posture without flipping replay to `mismatch`
- each `lanes[]` entry carries the same canonical `merge_check_id`, lane identity, and any
  recovered `provider_sources` from CI drift findings; per-lane `decision_basis[]` repeats the
  same cited lane basis at lane scope for downstream consumers that do not want to re-infer lane
  posture from the summary alone, and per-lane `evidence_classes` now makes the provenance of
  lane identity, state, blocking, and recovered source fields explicit

Finding objects may also include additive policy context keys when policy-aware diagnosis is surfaced:
`policy_outcome`, `policy_reason`, `policy_source`, `install_scope`, and `mutation_allowed`.
These keys are optional and backward-compatible.
CI verification drift findings may also include additive `metadata.governance` with a canonical
Ota-owned `merge_check_id`, the owning verifier `lane_task`, `lane_kind` (`task` or `aggregate`),
and the currently recovered provider source locations. This is the first stable machine-readable
merge-check identity layer for CI/merge consumers; workflow/job/check display names remain provider
render targets, not canonical truth.
When ota can trace the diagnosis source, finding objects may also include `provenance` and
`provenance_key`. Current shipped provenance keys include `repo_contract`, `org_policy`, and
`repo_signals`.
When doctor detects a managed-ecosystem opportunity that Ota does not yet ship as a toolchain
provider, the finding may also include an additive `toolchain_opportunity` object with
`ecosystem`, `fallback_runtime`, `fallback_tools`, `candidate_providers`, `shipped`, and
`agent_note`. This object is meant for editors and agents; the human-facing terminal finding keeps
the fallback guidance user-safe and does not have to expose provider-candidate wording directly.
Current shipped providers cover Rust, Node, Java, and uv-backed Python, so this additive object is
only expected when a future unsupported managed ecosystem is diagnosed.

When the repo declares runtimes or tools and policy provides approved sources for them,
`ota doctor --json` may also include a top-level `provisioning` object. That object is a read-only
plan with `provisionable` entries for targets policy approves and `blocked` entries for declared
targets that policy does not currently approve. It exists so humans and agents can see what would
be provisionable later without mutating the machine today.

When that plan exists, `ota doctor --json` also includes a top-level `provisioning_request`
object. It is the backend intake form and carries only the selected `actions` from the read-only
plan, so an installer backend can consume the request without re-deriving policy decisions from
the diagnostic payload.

Provisioning plan entries and request actions may also include additive semver-audit fields:
`normalized_requirement`, `resolved_version`, `policy_match`, and `package`. `normalized_requirement`
captures the semver intent ota matched against policy, `policy_match` records the exact approved
policy entry that authorized the request, and `resolved_version` appears only when policy provided
an explicit concrete version for deterministic installation. `package` is the backend install
identifier when policy specifies one (for example `openjdk-22-jdk` for apt). Range-only policy
approval does not invent a concrete install version, and when `resolved_version` is absent ota
continues to pass the original `requested_version` to the backend.

`ota doctor --json` and `ota workspace doctor --json` may also include a top-level
`finding_groups` array when the output contains repeated-action groups. Each entry includes a
stable semantic `action_key` derived from the grouped action class, plus the human-facing `action_title`,
`action_next`, and `count`. The grouped metadata is additive only; each `findings[]` entry remains
unchanged for machine consumers.

`ota workspace doctor --json` uses the same finding shape for per-repo findings, so the same
additive policy keys may appear there as well. When a repo declares execution metadata, the shared
`execution.env` array may include policy provenance with `source` values such as `org policy`
or `workspace policy`.

When a workspace repo declares runtimes or tools and policy provides approved sources for them,
`ota workspace doctor --json` may also include the same read-only `provisioning` plan on the
per-repo item. That plan uses the same `allowed`, `blocked`, and `actions` entries as repo-level
`ota doctor --json`, so editors and hosted validation can inspect the same future provisioning
signal without mutating anything. The action kinds are reserved for `select_source`, `install`,
and `verify` so the shape can grow without a breaking redesign.

`ota doctor --json` may also include an `execution` object when the contract declares execution
metadata that editors and remote-runner tooling can consume. Each `execution.env` entry may also
include an additive `policy` field when an approved policy value is available for that env key.

`ota doctor --json` may also include a top-level `toolchains` array for the selected workflow/task
path. Each entry records the selected toolchain name, provider, effective backend, target OS,
version, fulfillment mode, required flag, owned runtime, and any owned tools/components/targets
that ota is reasoning about on that selected path. Receipt-bearing execution surfaces may also add
`fulfilled` and `commands[]` when ota actually ran provider fulfillment commands on that execution
path. This is additive execution evidence; it does not
replace contract validation or finding-level detail.

`ota doctor --json` also includes a top-level `summary` object with finding counts and
machine-readable `verdict` / `agent_verdict` values so hosted validation and editor tooling do
not need to recompute them. When there is at least one finding, the summary may also include
`primary_blocker` with the highest-priority blocker details so CI and editors can answer the
question “what should I fix first?” without scanning the full list. When that blocker maps back to
a finding with stable identity, `summary.primary_blocker.code` is included additively alongside the
same blocker text and provenance fields.

For the main structured doctor finding families, `findings[].code`, `findings[].category`, and
`findings[].owner` are now emitted from explicit finding identity instead of being re-derived from
rendered English summary text. That includes contract advisories plus the main service, check,
runtime/tool, env-value and env-source, native prerequisite, backend/remote topology, workflow
probe/surface readiness, policy, repo-hygiene, and contract-drift finding lanes.

That identity surface is now guarded in CI: representative policy, workflow, service, env,
provisioning, and remote findings are contract-tested in JSON form, and shipped doctor findings are
rejected in test if they are introduced without explicit identity metadata.
The shipped code catalog and its published `category` / `owner` / `provenance_key` surfaces are
also synced into [`doctor-finding-reference.md`](doctor-finding-reference.md).

Policy-backed version-rule and strict-version findings also preserve the same `org_policy`
provenance and `policy_*` metadata as the older policy blocker and provisioning lanes.

When the repo contract declares an `agent` block, the additive `agent` summary can also include
`inferred_boundary_reviewed`. `false` means the current writable and protected boundary still comes
from starter or detector inference and has not been confirmed by the repo author yet.

`ota doctor --json` also includes a top-level `mode` string. It is `native` for host readiness
diagnosis, `container` when the report was produced with `ota doctor --mode container`, and
`remote` when the report was produced with `ota doctor --mode remote`, so consumers can tell which
execution context the findings describe without inferring it from the CLI invocation.

When the repo signals no longer match the declared contract, `ota doctor --json` may include
warning findings that describe the drift and point back to `ota detect --candidate-out` for the
comparison preview. Drift findings also include optional `owner_kind`, `ownership`, and
`provenance` fields so CI and editors can classify the mismatch as a repo-contract issue and
trace the source of the comparison. When that drift provenance is present, `owner_kind` is
currently `merged` and `provenance_key` is `repo_signals`.

`ota doctor --json` may also include an `extensions` object when the contract declares top-level
extension data. Each entry is a typed adapter descriptor with `kind`, `command`, and
`api_version`, plus optional `description` and `config`. Supported kinds today are
`check_provider`, `export_provider`, and `backend_provider`. The field is parsed and preserved for
discovery, and `ota extensions --run <name>` can execute one explicitly named `check_provider`
descriptor with `api_version: 1`; `ota extensions --publish <name>` can execute one explicitly
named `export_provider` descriptor with `api_version: 1`. `backend_provider` is discoverable in the
JSON output and can be selected by `execution.backends.remote.provider` for custom remote
execution. Backend providers receive a structured request on stdin and in
`OTA_BACKEND_PROVIDER_REQUEST_JSON`, then return a structured JSON response on stdout.

When the repo declares `workflows`, `ota doctor --json` includes an additive top-level `workflow`
object for the default workflow so editors and automation can reason about the canonical repo path
without inferring it from task names. That workflow summary may also include additive
`notes` and `readiness_probes` when the workflow declares them or references reusable named
probes.

`ota run <task> --dry-run --json` exposes selected task-path toolchains at top level in
`toolchains[]`. Receipt-bearing execution surfaces such as `ota receipt --json` and `ota up
--json` may also include additive `receipt.toolchains[]` entries with the same toolchain evidence
shape. Use those entries when you need to know which selected provider-backed ecosystem ota
checked or fulfilled on the recorded execution path, instead of inferring that from human text or
standalone runtime/tool fields. When fulfillment actually ran, `receipt.toolchains[]` can also
include `fulfilled: true` plus additive `commands[]` entries with the exact provider commands ota
executed for that toolchain during the recorded run path.

## `ota run --dry-run --json`

`ota run <task> --dry-run --json` is the read-only repo execution preview surface. It resolves the
same selected task path, execution backend, env requirements, toolchains, native prerequisites,
dependency order, and preview actions that text `RUN PREVIEW` uses, but it does not execute setup,
dependencies, containers, or task processes.
Every full preview payload therefore carries `execution_started: false`. When an explicit
execution option is unsupported, `ok` is false, `preview_status` is `BLOCKED`, `overrides` retains
the requested value, and `summary.primary_blocker.code` identifies the rejected option family
without implying that the requested backend, boundary, publication, resource limit, or dependency
policy started.
When that selected path also carries direct provisioning truth, the payload includes additive
top-level `provisioning` and `provisioning_request` fields using the same machine-readable shape as
`ota doctor --json`, so agents do not need to re-derive selected-path host fulfillment from
free-form `requirement_lines`.

For `action.kind: database_schema_mutation` on Unix, `plan.effect_application_plans` contains the exact
non-secret schema-v1 plan admitted by the typed adapter. It binds the adapter profile, selected
task, contract invocation origin, repository-relative effective working directory, effect reference,
attachment, consequence, resource binding, action, canonical discriminated action bounds, and ordered migration manifests. Apply and
rollback plans carry exactly one manifest; reset plans carry zero or one according to their declared
post-reset posture. File entries expose only root-relative paths and SHA-256 identities, never
migration bytes or credentials. Repo-level `ota run` and non-dry-run repo-level `ota up` perform one
closure-wide typed preflight before
command-scoped replay-input policy loading, agent/crossing/sandbox admission,
workflow-environment artifact rendering, or durable-log preparation. It opens every effective-cwd
and migration-root component relative to a retained repository descriptor without following
symlinks, verifies the retained materialized bytes, and returns the provider-disabled refusal before
task conditions, required services, dependencies, or provider contact. Proof paths that invoke
repo-level `ota up` inherit the same boundary. Non-Unix
execution refuses because race-safe source capture is unavailable. The field is plan-continuity evidence, not mutation, policy, receipt, archive, or
assurance evidence.

When one policy pack is active, `plan.effect_policy_decision` carries the shared schema-v1 typed
effect decision. It binds the policy snapshot and redacted source-location identities, source kind
and authority posture, selected invocation and execution graph, canonical effect and realization
sets, each applicable typed rule, current coarse-effect decisions, aggregate result, and fixed
`deny > warn > allow` precedence. `OTA_POLICY` is reported as `caller_selected`, nearest-ancestor
repository policy as `repository_controlled`, and workspace policy as `workspace_controlled`.
Matching policy bytes never upgrade that posture. An aggregate deny causes
`OTA_EFFECT_POLICY_DENIED` before side effects; allow and warn do not enable provider execution,
which remains disabled. The decision is not a canary, receipt, archive, provider attestation, or
positive assurance record. Each effect evaluation carries its `derivation_posture`: only
`declared_and_typed` is eligible for typed-policy admission. A `declared_only` attachment has no
exact adapter plan, carries no applicable typed rules, and remains an ineligible, fail-closed
execution boundary even when another attachment reaches the same effect definition. A run preview
with a declared-only attachment reports `BLOCKED` with
`typed_effect_admission_refused`; when no policy pack exists, its
`plan.effect_policy_decision` remains absent because the structural refusal does not invent policy
evidence.

`ota run --agent --expect-effect-refusal <id> --json <task>` and `ota up --workflow <name>
--agent --expect-effect-refusal <id> --json` emit the separate
[effect-refusal-canary.json](json-schemas/effect-refusal-canary.json) envelope. `status` is exactly
`passed`, `not_evaluated`, `assurance_gap`, or `failed`; only `passed` sets `ok: true` and exits `0`.
A pass carries the canary, effect, attachment, realization, selected invocation, policy decision,
policy snapshot, policy-source evidence, and explicit deny-rule identities with
`execution_started: false`. Generic or fallback-only refusal never satisfies the pass shape.
Unknown IDs or caller-selected overrides are `not_evaluated`; an absent exact origin is an
`assurance_gap`; an evaluated lane without explicit typed denial is `failed`. This is bounded
negative-control evidence, not provider mutation, receipt/archive evidence, or positive assurance.

The published schema for this surface is
[json-schemas/run-preview.json](json-schemas/run-preview.json). It covers single-target ready or
blocked previews, aggregate member previews, and the simpler pre-preview error envelope.

Repo-level `ota run --json` is currently not a mutating execution receipt surface. Use
`ota run <task> --dry-run --json` for planning, `ota receipt --json` for repo readiness receipts,
and `ota workspace run --json` for coordinated multi-repo execution receipts.

`preview_status` is the operator-facing preview label for this dry-run surface. It is
`RUNNABLE`, `RUNNABLE WITH WARNINGS`, or `BLOCKED`. Keep using `summary.verdict` for the canonical
shared readiness verdict.

`governance` is the compact machine-readable execution-governance summary for this selected task
path. Its effect and sandbox fields are derived from the full selected task closure, including
aggregate members and dependencies, rather than only the requested task node. It keeps the
fast-consumption fields together so CI and agents do not have to reconstruct the selected safety
posture from `requested_task`, effect declarations, and mode branches separately.
When the selected lane also carries compiled runtime-boundary truth, `governance.sandbox_policy`
now repeats the same first-target `codex_local` sandbox profile published by `ota tasks --json`,
so execution-facing preview consumers do not have to call a second command just to recover
filesystem or outbound boundary posture.
When live sandbox admission evaluates a typed lane, `sandbox_admission.effect_policy_decision`
carries the complete non-secret decision retained by that command's typed-effect admission. Sandbox
evaluation cannot re-plan the closure, replace its invocation origin, or reload policy truth.
Missing policy truth or aggregate denial refuses before canonical sandbox policy construction or
provider capability evaluation. An allow or warning still does not enable typed provider execution.

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "contract": "/path/to/ota.yaml",
  "task": "ci",
  "dry_run": true,
  "execution_started": false,
  "preview_status": "RUNNABLE",
  "summary": {
    "verdict": "ready",
    "agent_verdict": "ready",
    "error_count": 0,
    "warn_count": 0,
    "info_count": 0
  },
  "contract_identity": {
    "version": 1,
    "project": {
      "name": "demo"
    },
    "counts": {
      "runtimes": 0,
      "tools": 0,
      "env": 0,
      "services": 0,
      "checks": 0,
      "tasks": 1
    }
  },
  "resolved": {
    "backend": "native",
    "backend_source": "task",
    "target_strategy": "host process"
  },
  "requested_task": {
    "name": "ci",
    "kind": "run",
    "run": "npm test",
    "depends_on": [],
    "requires_services": [],
    "after_success": [],
    "after_failure": [],
    "after_always": [],
    "safe_for_agent": false,
    "effective_safe_for_agent": false
  },
  "requested_context": "host",
  "selected_context": "host",
  "env_summary": {
    "contract_count": 0,
    "source_count": 0,
    "source_issue_count": 0,
    "task_count": 1,
    "resolved_count": 0,
    "missing_count": 0,
    "invalid_count": 0
  },
  "env": [
    {
      "name": "OTA_WORKSPACE",
      "kind": "task",
      "required": false,
      "value": "/path/to/workspace",
      "source": "execution",
      "status": "task"
    }
  ],
  "provisioning_request": {
    "actions": [
      {
        "kind": "install",
        "target_kind": "tool",
        "name": "helm",
        "requested_version": ">=3.8",
        "package": "helm",
        "source": "brew",
        "source_config": {
          "tap_name": "vendor/tap",
          "tap_url": "https://github.com/vendor/homebrew-tap"
        }
      }
    ]
  },
  "governance": {
    "safety_posture": "review_required",
    "review_required": true,
    "effective_safe_for_agent": false,
    "default_mode": "native",
    "runnable_modes": [
      {
        "mode": "native",
        "default": true,
        "command": "ota run ci"
      }
    ],
    "network": false,
    "receipt_follow_up_command": "ota receipt --json --archive",
    "evaluation": {
      "preflight": {
        "state": "warning_only",
        "review_required": true,
        "declared_safe_for_agent": false,
        "effective_safe_for_agent": false,
        "crossing_required": true,
        "crossing_classification": "escalated",
        "crossing_boundary_family": "unsafe_task",
        "decision_inputs": [
          {
            "id": "task:ci",
            "family": "lane",
            "evidence_class": "derived",
            "replay_class": "pinned",
            "detail": "kind=task"
          },
          {
          "id": "actor_mode:non_agent",
            "family": "actor_mode",
            "evidence_class": "derived",
            "replay_class": "pinned"
          },
          {
            "id": "decision_owner:task_governance_preflight",
            "family": "decision_owner",
            "evidence_class": "derived",
            "replay_class": "pinned"
          },
          {
            "id": "declared_safe_for_agent:false",
            "family": "safety_declaration",
            "evidence_class": "derived",
            "replay_class": "pinned"
          },
          {
            "id": "effective_safe_for_agent:false",
            "family": "effective_safety",
            "evidence_class": "derived",
            "replay_class": "pinned"
          },
          {
            "id": "doctor_verdict:risky",
            "family": "readiness_gate",
            "evidence_class": "derived",
            "replay_class": "witnessed"
          },
          {
            "id": "receipt_expected:true",
            "family": "evidence_expectation",
            "evidence_class": "derived",
            "replay_class": "pinned"
          },
          {
            "id": "proof_expected:false",
            "family": "evidence_expectation",
            "evidence_class": "derived",
            "replay_class": "pinned"
          }
        ],
        "replay": {
          "status": "satisfied"
        },
        "evidence_classes": {
          "state": "derived",
          "review_required": "derived",
          "declared_safe_for_agent": "derived",
          "effective_safe_for_agent": "derived",
          "crossing_required": "derived",
          "crossing_classification": "derived",
          "crossing_boundary_family": "derived",
          "decision_inputs": "derived",
          "replay": "derived",
          "receipt_expected": "derived",
          "proof_expected": "derived"
        },
        "receipt_expected": true,
        "proof_expected": false
      },
      "post_execution": {
        "state": "not_run",
        "execution_attempted": false,
        "refusal_occurred": false,
        "decision_inputs": [
          {
            "id": "execution_attempted:false",
            "family": "execution_observation",
            "evidence_class": "derived",
            "replay_class": "witnessed"
          },
          {
            "id": "not_run_reason:preview_only",
            "family": "execution_observation",
            "evidence_class": "derived",
            "replay_class": "witnessed"
          },
          {
            "id": "receipt_present:false",
            "family": "receipt_observation",
            "evidence_class": "attested",
            "replay_class": "witnessed"
          },
          {
            "id": "proof_expected:false",
            "family": "proof_expectation",
            "evidence_class": "derived",
            "replay_class": "pinned"
          },
          {
            "id": "proof_present:false",
            "family": "proof_observation",
            "evidence_class": "derived",
            "replay_class": "witnessed"
          },
          {
            "id": "crossing_record_state:deferred_until_execution",
            "family": "crossing_observation",
            "evidence_class": "derived",
            "replay_class": "witnessed"
          },
          {
            "id": "decision_owner:preview_post_execution_evidence",
            "family": "decision_owner",
            "evidence_class": "derived",
            "replay_class": "pinned"
          }
        ],
        "replay": {
          "status": "satisfied"
        },
        "decision_basis": [
          {
            "id": "not_run:preview_only",
            "family": "execution_outcome",
            "evidence_class": "derived"
          },
          {
            "id": "evidence:proof_not_required",
            "family": "evidence_gate",
            "evidence_class": "derived"
          },
          {
            "id": "crossing_record:deferred_until_execution",
            "family": "crossing_evidence",
            "evidence_class": "derived"
          }
        ],
        "evidence_classes": {
          "state": "derived",
          "execution_attempted": "derived",
          "refusal_occurred": "derived",
          "crossing_record_state": "derived",
          "decision_inputs": "derived",
          "replay": "derived",
          "receipt_present": "attested",
          "proof_present": "derived"
        },
        "receipt_present": false,
        "proof_present": false
      }
    }
  },
  "plan": {
    "dependency_chain": ["ci"],
    "dependency_steps": [
      {
        "task": "ci",
        "backend": "native",
        "context": "host",
        "backend_selection_source": "task context"
      }
    ],
    "actions": ["would execute `npm test` on the host"]
  }
}
```

`ota up --dry-run --json` uses the same `execution_started: false` invariant. If a selected
workflow task cannot honor an explicit execution option, the preview is blocked before prepare,
setup, or run phases and `blockers[].identity.code` identifies the refused option family.

For non-dry-run `ota up --json`, an explicit typed policy deny retains an additive
`receipt.typed_effect_policy_refusal` record. Its schema-v1 payload carries
`reason_family: "effect_policy_denied"`, `execution_started: false`, and the exact
command-scoped `policy_decision`. It is absent for typed allow/warn provider-disabled refusal and
policy unavailability. Ordinary refusal remains non-durable.

`ota up --workflow <name> --archive-effect-refusal --json` explicitly creates a durable negative
receipt only when the retained command decision is an explicit typed deny. The live output adds
`policy_snapshot_archive { identity, path }` and `refusal_archive_path`. The archived receipt uses
`archive_context.kind: "effect_policy_refusal"` and retains the selected roots, ordered invocation
closure, execution selectors, and application plans. `ota receipt --history --json` accepts the
archive only after Core re-parses the immutable contract and private policy snapshots and
independently re-derives the closure, plans, source posture, and exact decision. JSON Schema checks
local shape; Core performs cross-record identity and semantic reconciliation. The private policy
snapshot is not a public export profile, and the archive does not prove provider contact, mutation,
positive execution, or assurance.

If atomic archive publication succeeds but the containing directory cannot be synchronized, the
command exits nonzero with `code: "effect_refusal_archive_durability_uncertain"`,
`published: true`, `durability: "uncertain"`, the exact `archive_path`, and recovery guidance. This
is not an ordinary write failure: the artifact exists and should be retained and verified through
`ota receipt --history --json` before repository transfer.

If receipt verification or retention pruning fails after the receipt directory has synchronized,
the command returns `effect_refusal_archive_post_publication_failed` with `published: true`,
`durability: "confirmed"`, and the exact `archive_path`. The receipt remains available for history
verification; resolve the maintenance failure before retrying archival.

If the same post-rename sync failure affects the private policy or contract snapshot before the
receipt is published, the code is `effect_refusal_snapshot_durability_uncertain`.
`artifact_kind` identifies `policy_snapshot` or `contract_snapshot`, `published` still describes
that exact snapshot, and `receipt_published: false` prevents consumers from treating the partial
transaction as durable refusal evidence. Rerun the same archive command to reuse the
content-addressed snapshot and complete receipt publication.

Use this when a human or agent needs the selected run plan before execution:

- `resolved` is the selected backend/lifecycle/image/provider plan
- `requested_task` is the selected task body after contract validation
- `requested_context` is the task-declared context (when present)
- `selected_context` is the resolved execution context ota will apply for this preview
- `env_summary`, `sources`, and `env` show the selected env state and blockers
- `governance` is the compact CI/agent-friendly summary for the selected lane:
  `safety_posture`, `review_required`, closure-aware effective safety, effective `default_mode`,
  runnable mode commands, selected effect surface, and the next durable receipt command
- `governance.evaluation` is the canonical phase-labeled machine surface for this lane:
  `preflight.state` tells consumers whether the selected path is `allowed`, `warning_only`,
  `blocked`, or `refused` before execution, while `post_execution.state` keeps execution evidence
  separate as `not_run` or a satisfied evidence state after execution surfaces exist. A preflight
  refusal is represented there as `not_run` with `refusal_occurred: true` and
  `not_run_reason: preflight_refusal`, never as fabricated attempted execution.
- `governance.evaluation.preflight.decision_basis[]` publishes the stable cited gate or refusal
  basis behind that preflight posture so CI or harness consumers do not have to infer it from
  human strings
- `governance.evaluation.preflight.decision_inputs[]` publishes the replay-grade cited inputs that
  posture depended on; `replay_class: "pinned"` means the input should be reusable for
  authoritative replay without silently re-reading ambient state, while `replay_class:
  "witnessed"` means the input came from observed execution evidence rather than a pinned selector
- the first shipped mechanism-tripwire on that same surface is
  `decision_owner:<stable-id>` with family `decision_owner` and `replay_class: "pinned"`, so the
  emitted governance record can still link back to the stable decision owner without inventing a
  second governance model
- `governance.evaluation.preflight.replay.status` publishes whether ota can re-derive that
  preflight verdict from the cited inputs it just emitted: `satisfied`, `mismatch`, or
  `unavailable`
- replay now also checks the trust-sensitive cited-input classes on that same canonical surface,
  so a selector that should be `pinned` or receipt evidence that should be `attested` cannot
  silently degrade into weaker machine truth without flipping replay to `mismatch`
- `governance.evaluation.preflight.evidence_classes` publishes field-level provenance for the
  authoritative preflight verdict, distinguishing ota-derived boundary truth from runner-attested
  attachment state
- `governance.evaluation.post_execution.decision_basis[]` publishes the stable cited evidence or
  non-run basis behind `post_execution.state`, so consumers can distinguish receipt/proof
  satisfaction from preview-only, blocked, or refusal-suppressed execution without scraping prose
- `governance.evaluation.post_execution.decision_inputs[]` publishes the replay-grade cited
  execution and evidence inputs behind that post-execution verdict, again distinguishing reusable
  pinned selectors from witnessed observations
- post-execution uses the same additive `decision_owner:<stable-id>` cited input so preview-only
  evidence and up-result evidence can publish a narrow mechanism identity on the same canonical
  record
- `governance.evaluation.post_execution.replay.status` does the same for the post-execution
  evidence verdict, so consumers can tell whether the emitted outcome still reconciles to the
  cited decision inputs instead of trusting a flatter claim
- `governance.evaluation.post_execution.evidence_classes` does the same for the post-execution
  evidence record, so downstream consumers can tell which fields are derived versus boundary-
  attested
- when post-execution already knows the evidence-state reason, it now cites that directly:
  `evidence:proof_present`, `evidence:proof_missing`, `evidence:proof_not_required`,
  `receipt_status:<status>`, and `crossing_record:<state>` are all stable machine-readable basis
  entries instead of flatter implicit outcomes
- `ota run <task> --dry-run --json --agent` now reflects the enforced runner boundary in
  `governance.evaluation.preflight`: unsafe selected tasks or unsafe reachable closures publish
  `state: "refused"` and return a blocked preview instead of looking runnable in JSON
- `ota run --agent --expect-refusal --json <task>` is the machine-readable negative-control
  surface. It emits `status: "refused_as_expected"` with `canary.kind`, `canary.target`,
  `canary.execution_started: false`, the derived refusal record, and the blocked receipt. If the
  target is admitted it emits `status: "refusal_not_observed"`. A policy-only refusal emits
  `status: "wrong_refusal_boundary"`: it is still blocked, but it does not prove the safety
  closure boundary. Both exit non-zero and record `execution_started: false`; consumers must
  treat either as enforcement drift. Its schema is
  [refusal-canary.json](json-schemas/refusal-canary.json). The top-level `ok` reports whether the
  canary assertion passed; nested `receipt.ok` remains `false` because agent execution was blocked.
- `requested_task.safe_for_agent` is the declared contract safe membership, while
  `requested_task.effective_safe_for_agent` reflects whether the reachable dependency/workflow
  closure remains agent-safe
- `governance.safety_posture` may be `declared_safe_closure_unsafe` when the top-level task is
  declared safe but reaches review-required closure
- `artifact_routing[]` is the additive artifact guide for this selected lane; it points to the
  next receipt/proof artifact or capture command using typed `role`, `kind`, and `stage_family`
- `toolchains[]` keeps toolchain-owned capabilities on the toolchain instead of duplicating them as
  standalone runtime/tool evidence
- `plan.dependency_chain` is the ordered task graph ota would execute
- `plan.dependency_steps[]` adds machine-readable backend provenance per planned task step, including
  selected backend, selected context, parent task, and whether the backend came from `override`,
  `task default mode`, `task context`, `mode context`, `mode branch support`, `default context`,
  `contract preferred`, `default`, or `inherited parent backend`
- when a dependency step is a structured setup lane, `plan.dependency_steps[].prepare` carries the
  same strict machine-readable prepare summary used by task/workflow discovery. It retains the
  selected source kind, working directory, Compose file and env-file sets, package filter/groups,
  browser targets, and hydration posture, including additive
  `declared_hydration_provenance` and `resolved_hydration_provenance` for hydration-owned source
  posture such as `.NET restore` `config_file`, explicit `sources[]`, or uv local-project
  declaration; the declared record is
  contract truth, while the resolved record can add parsed `source_identities[]` with stable feed
  `url` and, when config-backed, its declared `name`, plus `resolution: resolved|unavailable` and
  `resolution_error` when Ota could not recover the config-backed source set honestly; an
  `ambient_default` posture is intentionally `unavailable` until the contract declares a config
  file or explicit source URLs
- `plan.requirement_lines`, `plan.actions`, and `plan.notes` show what ota would check, activate,
  provision, or run
- exit `0` means the preview is actionable; exit `1` means the preview is blocked by contract,
  env, or execution-plan problems
- blocked previews still use the full preview envelope on stdout so automation can read
  `summary.primary_blocker` without scraping stderr

## `ota authority inspect --json`

`ota authority inspect --json` is a diagnostic-only view over the fixed `prebound_file` authority
boundary. It accepts no authority selector or path override, performs no execution or authority
mutation, and always keeps its strongest claim at
`current_process_filesystem_guarded`.

```json
{
  "ok": true,
  "kind": "authority_inspect",
  "profile": {
    "id": "prebound_file_hardening",
    "version": 1,
    "verdict": "matched_with_unknowns"
  },
  "authority_source": "prebound_file",
  "authority_separation_posture": "current_process_filesystem_guarded",
  "platform": {
    "os": "linux",
    "architecture": "x86_64"
  },
  "observations": [
    {
      "id": "trust_store",
      "required": true,
      "status": "passed",
      "method": "canonical_protected_file_verifier",
      "reason": "trust_store_verified"
    },
    {
      "id": "passwordless_sudo",
      "required": false,
      "status": "unknown",
      "method": "not_safely_observable",
      "reason": "passwordless_sudo_not_probed"
    }
  ],
  "summary": {
    "passed": 9,
    "failed": 0,
    "unknown": 5,
    "unavailable": 0,
    "authority_bindings_observed": 1
  }
}
```

Observation statuses are `passed`, `failed`, `unknown`, or `unavailable`. Only
`matched_with_unknowns` exits zero, and it requires every required check to pass. Required unknown
or unavailable checks produce `incomplete`; explicit required failures produce `failed`; an
unsupported fixed carrier produces `unsupported`. All verdicts use the same
[authority-inspect.json](json-schemas/authority-inspect.json) schema. Public output contains only
stable reason codes, never protected paths, keys, signatures, bundle contents, grant identities,
or parser/filesystem details.

## `ota policy review --json`

`ota policy review --json` is the policy-authority view over a repo contract. It is read-only and
keeps the active policy source/path explicit so editors and CI can tell whether the repo contract
or the org policy boundary needs to move.

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "policy_source": "repo policy",
  "policy_path": "./.ota/org-policy.yaml",
  "summary": {
    "ok": false,
    "error_count": 1,
    "warn_count": 1,
    "info_count": 0
  },
  "finding_groups": [
    {
      "action_key": "policy-provisioning-declared",
      "action_title": "Review active policy surfaces",
      "action_next": "use `ota policy review` to inspect the active policy source, or keep these approved sources in mind when provisioning or bootstrap needs a governed path",
      "count": 1
    }
  ],
  "findings": [
    {
      "code": "OTA_POLICY_PACK_VIOLATION",
      "category": "policy",
      "owner": "org_policy",
      "severity": "error",
      "summary": "Repo does not satisfy org policy pack",
      "why": "...",
      "next": "...",
      "evidence": {
        "observed": "...",
        "expected": "...",
        "source": "...",
        "checked_at": "...",
        "command": "...",
        "path": "..."
      }
    }
  ]
}
```

The optional `policy` payload mirrors the loaded policy pack when ota can read it. Consumers that
need the authoritative boundary can inspect `policy_source` and `policy_path` first, then read the
findings and grouped actions to decide whether the repo contract or the org policy pack should be
updated.

## `ota policy init --json`

`ota policy init --json` is the conservative starter-policy surface. It either previews or writes a
minimal valid `.ota/org-policy.yaml` without guessing org rules, provisioning approvals, or policy
intent.

```json
{
  "ok": true,
  "path": "/abs/path/to/.ota/org-policy.yaml",
  "written": false,
  "mode": "policy",
  "preset": "agent",
  "config": {
    "policies": {
      "agent": {
        "require_safe_tasks": true,
        "require_writable_paths": true
      },
      "exports": {
        "require_agents_md": true
      }
    }
  }
}
```

Current JSON fields:

- `ok`
- `path`
- `written`
- `mode` (`policy`)
- optional `preset` (`required-sections`, `provisioning`, or `agent`)
- `config`
- failure responses include `error`
- overwrite refusals may include `next`

Failure example:

```json
{
  "ok": false,
  "path": "/abs/path/to/.ota/org-policy.yaml",
  "written": false,
  "mode": "policy",
  "error": "`./.ota/org-policy.yaml` already exists; refusing to overwrite the existing policy pack",
  "next": "ota policy /abs/path/to/repo"
}
```

`ota workspace doctor --json` may include the same `execution` object on each repo item when the
underlying repo contract declares execution metadata, including env provenance for inherited
workspace policy values.

`ota workspace doctor --json` may also include the same `extensions` object on each repo item when
the underlying repo contract declares it. The descriptor shape matches `ota doctor --json`.

Each repo item may also include additive `primary_blocker` with that repo's current highest-priority
finding (`severity`, `summary`, `why`, `next`, optional `code`, and optional provenance fields).

`ota workspace doctor --json` also includes a top-level `summary` object with repo and finding
counts for hosted validation and editor consumers. The workspace summary also carries
`verdict` / `agent_verdict` values. When there is at least one finding, the summary may also
include `primary_blocker` with the highest-priority blocker details and the repo name that owns it.
When that blocker maps back to a stable finding identity, the workspace summary also includes
additive `primary_blocker.code`.

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "repo_count": 1,
    "ready_count": 0,
    "not_ready_count": 1,
    "verdict": "not_ready",
    "agent_verdict": "not_ready",
    "error_count": 1,
    "warn_count": 0,
    "info_count": 0,
    "primary_blocker": {
      "repo": "web",
      "severity": "error",
      "summary": "Repo not acquired: web",
      "why": "...",
      "next": "...",
      "code": "OTA_WORKSPACE_REPO_NOT_ACQUIRED"
    }
  },
  "repos": [
    {
      "name": "web",
      "path": "/abs/path/to/apps/web",
      "contract_path": "/abs/path/to/apps/web/ota.yaml",
      "workflow": "app",
      "required": true,
      "ok": false,
      "execution": {
        "preferred": "native",
        "supported": ["native"],
        "lifecycle": "persistent",
        "env": [
          {
            "name": "OTA_TEST_SHARED",
            "required": true,
            "policy": "workspace-policy",
            "source": "workspace policy"
          }
        ]
      },
      "findings": [
        {
          "severity": "error",
          "summary": "Repo not acquired: web",
          "why": "...",
          "next": "..."
        }
      ]
    }
  ]
}
```

`ota workspace list --json` also includes a top-level `summary` object with repo inventory counts
for editor, CI, and hosted preflight tooling. Its per-repo `execution` object mirrors workspace
doctor execution metadata, including env provenance when the repo contract declares execution env
requirements.

Root monorepo summary output can also include grouped member findings under `members`.

Doctor JSON findings also include remote target-shape warnings when relevant, such as suspicious
`ssh`/`tsh` targets without `user@host` or `kubectl` targets that do not start with `pod/`.

## `ota explain --json`

Explain JSON separates the grouped remediation plan from the detailed finding list:

- `actions` is the ordered grouped plan and is the best machine-readable "what should I do first?"
  surface
- `steps` keeps the finding-level detail with stable codes for deeper drill-in

Both actions and steps stay deterministic. Explain steps may also include `provenance` and
`provenance_key` when ota can trace the diagnosis source for the underlying finding. When a
finding already carries explicit identity, `steps[].code` preserves that stable code directly
instead of re-deriving advisory identity from rendered summary text.

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "error_count": 1,
    "warn_count": 1,
    "info_count": 0,
    "step_count": 2
  },
  "actions": [
    {
      "order": 1,
      "action_key": "tasks-missing",
      "action_title": "Add at least one declared task to the contract",
      "severity": "error",
      "count": 1,
      "why": "...",
      "next": "run `ota detect --dry-run .` to review inferred tasks before writing one"
    }
  ],
  "steps": [
    {
      "order": 1,
      "code": "OTA_TASKS_MISSING",
      "severity": "error",
      "summary": "No tasks defined in contract",
      "why": "...",
      "next": "...",
      "provenance": "org policy",
      "provenance_key": "org_policy"
    }
  ]
}
```

## `ota workspace explain --json`

Workspace explain now exposes two ordered action surfaces:

- top-level `actions` for the grouped workspace plan with explicit repo ownership
- per-repo `actions` for the grouped ordered remediation plan
- `steps` for the finding-level detail

Workspace explain steps may also include `provenance` and `provenance_key` when ota can trace the
diagnosis source for the underlying finding.

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "repo_count": 2,
    "ready_count": 0,
    "not_ready_count": 2,
    "error_count": 2,
    "warn_count": 0,
    "info_count": 0,
    "step_count": 2
  },
  "actions": [
    {
      "repo": "api",
      "path": "/abs/path/to/api",
      "contract_path": "/abs/path/to/api/ota.yaml",
      "required": true,
      "order": 1,
      "action_key": "tasks-missing",
      "action_title": "Add at least one declared task to the contract",
      "severity": "error",
      "count": 1,
      "why": "...",
      "next": "run `ota detect --dry-run .` to review inferred tasks before writing one"
    }
  ],
  "repos": [
    {
      "name": "api",
      "path": "/abs/path/to/api",
      "contract_path": "/abs/path/to/api/ota.yaml",
      "required": true,
      "ok": false,
      "summary": {
        "error_count": 1,
        "warn_count": 0,
        "info_count": 0,
        "step_count": 1
      },
      "actions": [
        {
          "order": 1,
          "action_key": "tasks-missing",
          "action_title": "Add at least one declared task to the contract",
          "severity": "error",
          "count": 1,
          "why": "...",
          "next": "run `ota detect --dry-run .` to review inferred tasks before writing one"
        }
      ],
      "steps": [
        {
          "order": 1,
          "code": "OTA_TASKS_MISSING",
          "severity": "error",
          "summary": "No tasks defined in contract",
          "why": "...",
          "next": "...",
          "provenance": "org policy",
          "provenance_key": "org_policy"
        }
      ]
    }
  ]
}
```

## `ota workspace validate --json`

`ota workspace validate --json` uses the same success/failure shape as `ota validate --json`,
but `path` refers to the resolved `ota.workspace.yaml`.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "error_count": 0
  }
}
```

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "error_count": 1
  },
  "errors": ["..."]
}
```

## `ota workspace init --json` and `ota workspace detect --json`

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml",
  "written": false,
  "mode": "scaffold",
  "config": {
    "version": 1,
    "workspace": {
      "name": "ota-workspace"
    },
    "repos": {
      "web": {
        "path": "apps/web",
        "required": true
      }
    }
  },
  "provenance": [
    {
      "field": "workspace.name",
      "provenance": "workspace-derived",
      "provenance_key": "workspace_scaffold",
      "source": "workspace-root-directory"
    },
    {
      "field": "repos.web.path",
      "provenance": "workspace-derived",
      "provenance_key": "workspace_scaffold",
      "source": "workspace-discovery"
    },
    {
      "field": "repos.web.required",
      "provenance": "template-derived",
      "provenance_key": "template_derived",
      "source": "ota.workspace.init#repo_required_default"
    }
  ],
  "included": [
    {
      "name": "web",
      "path": "apps/web"
    }
  ],
  "missing_contract": [],
  "comparison": {
    "existing_contract": true,
    "additions": [
      {
        "name": "api",
        "path": "services/api"
      }
    ]
  }
}
```

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "written": false,
  "mode": "scaffold",
  "error": "..."
}
```

Failure shape can also include:

- `next`: optional safe follow-up command when overwrite is refused

`provenance` is the per-field source map for the generated workspace scaffold or merged workspace contract:

- `workspace-derived` entries use `provenance_key: "workspace_scaffold"` for fields taken from workspace root naming or discovered repo contracts
- `workspace-declared` entries use `provenance_key: "workspace_contract"` for fields preserved from an existing `ota.workspace.yaml` during merge preview or merge write
- `template-derived` entries cover scaffold defaults such as `version` and the default `required: true` repo policy
- `source` tells you whether a field came from the workspace root directory, workspace discovery, the existing workspace contract, or an explicit workspace scaffold default

## `ota workspace doctor --json`

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "repo_count": 1,
    "ready_count": 0,
    "not_ready_count": 1,
    "error_count": 1,
    "warn_count": 0,
    "info_count": 0
  },
  "repos": [
    {
      "name": "web",
      "path": "/abs/path/to/apps/web",
      "contract_path": "/abs/path/to/apps/web/ota.yaml",
      "required": true,
      "ok": false,
      "primary_blocker": {
        "severity": "error",
        "summary": "Repo not acquired: web",
        "why": "...",
        "next": "...",
        "code": "OTA_WORKSPACE_REPO_NOT_ACQUIRED"
      },
      "findings": [
        {
          "severity": "error",
          "summary": "Repo not acquired: web",
          "why": "...",
          "next": "..."
        }
      ]
    }
  ]
}
```

When `ota workspace doctor --json --progress-json` is used, Ota also emits live workspace progress
events as compact one-line JSON on `stderr`. Those progress events use the same
`workspace_progress` event shape as `ota workspace run|up|refresh --json --progress-json`, while
the final doctor report remains the single JSON document on `stdout`.

When a workspace repo declares runtimes or tools and policy provides approved sources for them,
the per-repo item may also include the same `provisioning` diagnostics bundle as repo-level
doctor output. When policy declares adapter bootstrap sources, the per-repo item may also
include the same `adapter_bootstrap` diagnostics bundle. Both bundles carry the read-only plan
and backend intake request together, so workspace consumers can inspect the same future
provisioning signals without mutating anything.
When the workspace contract pins `repos.<name>.workflow`, each repo item may also include that
selected workflow name as additive `workflow`.

## `ota workspace tasks --json`

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "repo_count": 1,
    "ready_count": 1,
    "not_ready_count": 0,
    "acquired_count": 1,
    "missing_contract_count": 0
  },
  "repos": [
    {
      "name": "api",
      "path": "/abs/path/to/services/api",
      "contract_path": "/abs/path/to/services/api/ota.yaml",
      "workflow": "app",
      "required": true,
      "acquired": true,
      "depends_on": ["db"],
      "tasks": [
        {
          "name": "env-local",
          "description": "Create a local env overlay when missing.",
          "kind": "copy_if_missing",
          "action": {
            "kind": "copy_if_missing",
            "from": ".env.example",
            "to": ".env.local"
          },
          "depends_on": [],
          "after_success": [],
          "after_failure": [],
          "after_always": []
        },
        {
          "name": "setup",
          "description": "Install repo dependencies.",
          "kind": "run",
          "run": "pnpm install",
          "effects": {
            "writes": ["node_modules"],
            "network": true
          },
          "depends_on": ["env-local"],
          "after_success": ["verify-lockfile"],
          "after_failure": [],
          "after_always": ["cleanup-temp"]
        },
        {
          "name": "quickstart",
          "description": "Run the packaged preview command.",
          "kind": "command",
          "launch": {
            "kind": "command",
            "exe": "npx",
            "args": ["vite", "--host", "127.0.0.1", "--port", "3000"]
          },
          "depends_on": [],
          "after_success": [],
          "after_failure": [],
          "after_always": []
        }
      ]
    }
  ]
}
```

Non-acquired repos keep `acquired: false` and `tasks: []`. Each task report can also carry
`effects`, `requires_services`, `after_success`, `after_failure`, and `after_always` so
automation can see the same repo writes, connectivity needs, external-state mutations, service
requirements, and post-outcome task graph that `ota run` executes. Structured task launch is
additive through `tasks[*].launch` when the repo task uses command or packaged-container launch.
Structured task actions are additive through `tasks[*].action` when the repo task uses a
first-class action such as `copy_if_missing` or the execution-disabled V12
`database_schema_mutation` adapter. The latter reports no successful provider operation: current
Ota captures bounded migration bytes, binds the plan across dry-run and selected execution, then
refuses before provider contact.
For this action, the task JSON summary uses `action.kind: database_schema_mutation` and
`action.from` for the selected declared effect reference; the authored contract field remains
`action.effect`.
When the workspace contract pins `repos.<name>.workflow`, each repo item may also include additive
`workflow`.

## `ota workspace list --json`

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml",
  "repos": [
    {
      "name": "api",
      "path": "/abs/path/to/services/api",
      "contract_path": "/abs/path/to/services/api/ota.yaml",
      "workflow": "app",
      "contract_present": true,
      "required": true,
      "acquired": true,
      "status": "READY",
      "execution": {
        "preferred": "remote",
        "supported": ["remote"],
        "lifecycle": "ephemeral",
        "env": [
          {
            "name": "AWS_PROFILE",
            "required": true,
            "policy": "workspace-policy",
            "source": "workspace policy"
          }
        ],
        "backends": {
          "remote": {
            "provider": "ssh",
            "target": "user@host",
            "cwd": "/workspace"
          }
        }
      },
      "depends_on": ["db"]
    }
  ]
}
```

When the workspace contract pins `repos.<name>.workflow`, each repo item may also include that
selected workflow name as additive `workflow`.

## `ota workspace run --json`

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml",
  "task": "setup",
  "summary": {
    "error_count": 0,
    "warn_count": 0,
    "info_count": 0,
    "step_count": 1,
    "repo_count": 1,
    "ready_count": 1,
    "not_ready_count": 0
  },
    "receipt": {
      "ok": true,
      "path": "/abs/path/to/ota.workspace.yaml",
      "scope": "workspace",
      "contract": "/abs/path/to/ota.workspace.yaml",
      "contract_identity": {
        "version": 1,
        "project": {
          "name": "ota-dev",
          "type": "workspace"
        },
        "counts": {
          "runtimes": 0,
          "tools": 0,
          "env": 0,
          "services": 0,
          "checks": 0,
          "tasks": 0,
          "repos": 1,
          "policies": 0
        }
      },
      "workspace": "ota-dev",
      "env_sources": [
        {
          "name": "OTA_TEST_SHARED",
          "value": "workspace-policy",
          "source": "workspace policy"
        }
      ],
      "steps": [
        {
          "order": 1,
          "label": "web",
        "status": "READY",
        "detail": "task `setup`"
      }
    ],
    "summary": {
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1,
      "repo_count": 1,
      "ready_count": 1,
      "not_ready_count": 0
    }
  },
  "repos": [
    {
      "name": "web",
      "path": "/abs/path/to/apps/web",
      "contract_path": "/abs/path/to/apps/web/ota.yaml",
      "workflow": "app",
      "required": true,
      "ok": true,
      "status": "READY",
      "task": "setup",
      "findings": []
    }
  ]
}
```

`receipt` mirrors the workspace execution roll-up, keeps backend-aware execution metadata on the
same surface as the repo-level execution commands, and includes additive
`receipt.contract_identity` with workspace name/type plus compact workspace repo and policy counts.

Optional per-repo fields:

- `next`
- `next_steps`
- `exit_code`
- `stdout`
- `stderr`
- `env_sources`

When you add `--progress-json` to `ota workspace run --json`, `ota workspace up --json`, or
`ota workspace refresh --json`, ota keeps the final roll-up JSON report on stdout and emits
newline-delimited progress events on stderr.

```json
{
  "event": "workspace_progress",
  "command": "workspace.run",
  "workspace": "ota-dev",
  "status": "RUN",
  "repo": "api",
  "phase": "run",
  "stage_family": "verify",
  "tail": "workspace:entrypoint -> api:tests:contract",
  "task": "workspace:entrypoint",
  "repo_task": "api:tests:contract",
  "dependency": null
}
```

- `event` is always `workspace_progress`
- `command` identifies the workspace command emitting the event, such as `workspace.doctor`,
  `workspace.check`, `workspace.diff`, `workspace.status`, `workspace.receipt`, `workspace.run`,
  `workspace.up`, or `workspace.refresh`, so machine consumers do not need out-of-band stream
  context
- `workspace` is the resolved workspace name
- `status` is the live transition label such as `ACQUIRE`, `RUN`, `READY`, `BLOCKED`, `WARN`,
  `TASK FAILED`, `ACQUIRE FAILED`, `REFRESH`, or `REFRESH PREVIEW`
- `repo` is the workspace repo name
- `phase` is the command-local operational lane such as `acquisition`, `run`, `prepare`,
  `refresh`, `doctor`, `check`, `diff`, `status`, or `receipt`, so consumers do not need to infer
  intent from the `command` field alone
- `stage_family` carries the broader governance family for the event such as `prepare`, `setup`,
  `verify`, or `receipt`
- `tail` carries the same operator-facing suffix as the text progress line when one exists
- `task` and `repo_task` are populated for workspace-run task dispatch so agents can see the
  requested workspace task and the resolved repo-local task separately; those fields stay present
  on terminal repo-task events such as `READY` and `TASK FAILED`, not only the initial `RUN`
- `dependency` is populated for blocked transitions when a repo is waiting on a failed dependency;
  blocked workspace-run events also keep `task` and `repo_task` when ota already knows the mapped
  repo-local task

## `ota workspace check --json`

`ota workspace check --json` uses the same finding shape as `ota workspace doctor --json`,
including additive `finding_groups` and per-repo `primary_blocker` when present:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "repo_count": 1,
    "ready_count": 0,
    "not_ready_count": 1,
    "verdict": "not_ready",
    "agent_verdict": "not_ready",
    "error_count": 1,
    "warn_count": 0,
    "info_count": 0,
    "primary_blocker": {
      "repo": "web",
      "severity": "error",
      "summary": "Check failed: health-check",
      "why": "...",
      "next": "...",
      "code": "OTA_CHECK_FAILED"
    }
  },
  "repos": [
    {
      "name": "web",
      "path": "/abs/path/to/apps/web",
      "contract_path": "/abs/path/to/apps/web/ota.yaml",
      "required": true,
      "ok": false,
      "primary_blocker": {
        "severity": "error",
        "summary": "Check failed: health-check",
        "why": "...",
        "next": "...",
        "code": "OTA_CHECK_FAILED"
      },
      "findings": [
        {
          "severity": "error",
          "summary": "Check failed: health-check",
          "why": "...",
          "next": "..."
        }
      ]
    }
  ]
}
```

When `ota workspace check --json --progress-json` is used, Ota also emits live workspace progress
events as compact one-line JSON on `stderr`. Those progress events use the same
`workspace_progress` event shape as `ota workspace doctor|run|up|refresh --json --progress-json`,
while the final workspace-check report remains the single JSON document on `stdout`.

`summary` mirrors the workspace doctor roll-up so hosted gates can read the same repo and finding
counts from checks-only output. When one repo has several findings, additive `primary_blocker`
identifies the repo's current highest-priority next move without forcing consumers to choose one
from the full list themselves.

## `ota init --json`

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "written": false,
  "mode": "detected",
  "config": {
    "version": 1
  },
  "inferred": [
    {
      "field": "project.name",
      "type": "project",
      "source_class": "environment_toolchain",
      "value": "ota-app",
      "source": "package.json#name",
      "signal": "config",
      "confidence": "high"
    }
  ],
  "provenance": [
    {
      "field": "project.name",
      "provenance": "detector-inferred",
      "provenance_key": "repo_signals",
      "source": "package.json#name",
      "confidence": "high"
    },
    {
      "field": "agent.bootstrap.ota.sh",
      "provenance": "template-derived",
      "provenance_key": "template_derived",
      "source": "ota.init#starter_agent_bootstrap"
    }
  ]
}
```

`mode` is:

- `blank` for the minimal starter path
- `detected` for detector-led starter output
- `pack` for an explicit starter-pack preview or write
- `catalog` for `ota init --packs --json`

When task inference is confident enough to write, `config.tasks.<name>.notes` may also be
present and point at the matching `ota run <task>` command.

Each `inferred[*]` entry now carries additive metadata for human and machine consumers:

- `type` is one of `project`, `runtime`, `tool`, `env`, `service`, `check`, `task`, `agent`, or `field`
- `source_class` is one of `environment_toolchain`, `task_command`, `runtime_service`, `ci_verification`, `agent_boundary`, `workspace_bootstrap`, or `heuristic`
- `signal` is one of `config`, `script`, `lockfile`, `file`, `template`, or `convention`
- task-shaped entries can also include `agent_safe` (`yes`, `no`, `unknown`) and `agent_signal` (`verification_candidate` or `bootstrap_candidate`) when ota can classify the task for agent workflows

In dry-run preview mode, `config` matches the starter contract ota would review or write,
including derived starter defaults such as a minimal `agent` block when ota can infer one
and `preview_candidate` contains the complete source-bound candidate for that exact preview. Its
`schema_version` is `4`, its profile is `init_starter_preview_v1`, and
`resulting_contract_identity` is the semantic identity of `config`. Consumers must reconcile the
wrapper `identity`, schema version, and profile with `preview_candidate.candidate`, then parse
`config` and compare its semantic identity with `resulting_contract_identity`; JSON Schema enforces
the local shape but not those cross-record equalities. This is review evidence only: it has no
application projection and grants no contract-write authority.

When `mode` is `pack`, the payload also includes `pack` with the selected built-in starter pack
name. Pack-generated tasks can carry short `description` fields, optional `pack_options` records
the selected starter-specific knobs such as Node package manager or Python test runner, and
`provenance` records those fields as `template-derived` with the selected
`ota.init#starter_pack...` variant source while keeping directory-derived values such as
`project.name` traced to `ota.init#directory_name`.

```json
{
  "pack": "node",
  "pack_options": {
    "package_manager": "npm"
  },
  "config": {
    "tasks": {
      "setup": {
        "run": "npm install"
      }
    }
  },
  "provenance": [
    {
      "field": "tasks.setup.run",
      "provenance": "template-derived",
      "provenance_key": "template_derived",
      "source": "ota.init#starter_pack.node.package_manager.npm"
    }
  ]
}
```

When explicit pack mode disagrees with strong detected repo signals, ota adds `pack_advisory`
without changing the selected pack or merging detector output into the starter:

```json
{
  "pack": "python",
  "pack_advisory": {
    "selected_pack": "python",
    "suggested_pack": "node",
    "selected_pack_score": 0,
    "suggested_pack_score": 3,
    "score_gap": 3,
    "summary": "stronger distinct repo signals favor `node` over the selected pack `python`",
    "signals": ["package.json"],
    "signal_details": [
      {
        "signal": "package.json",
        "weight": 3
      }
    ],
    "next": "ota init --pack node --dry-run ."
  }
}
```

`selected_pack_score` and `suggested_pack_score` show the distinct-signal strength ota saw for
each pack, `score_gap` shows how far the suggested pack leads, `signal_details` preserves the
weighted signal markers behind the flat `signals` list for the suggested pack, and
`selected_signal_details` does the same for any incidental signals that still matched the
explicitly selected pack.

When the repo clearly looks like a managed ecosystem that ota does not ship as a toolchain yet,
`ota init --json` also adds `toolchain_opportunities` as additive agent-facing guidance. Terminal
text stays user-safe and only says to keep the current `runtimes` / `tools` fallback model for
now; provider-candidate detail stays in JSON. When ota already ships the ecosystem owner, the
starter `config` uses `toolchains.<name>` directly instead of adding a fallback opportunity. For
example, Python repos with `uv.lock` now converge directly on `toolchains.python` with the
canonical structured fulfillment model, so they no longer emit fallback Python opportunity
guidance. The same ownership promotion now applies to detected Go, Ruby, and .NET ecosystems,
yielding `toolchains.go`, `toolchains.ruby`, and `toolchains.dotnet` instead of split
runtime/tool declarations.

`provenance` is the per-field source map for the starter contract:

- detector-backed fields use `provenance: "detector-inferred"` and `provenance_key: "repo_signals"`
- starter-only defaults use `provenance: "template-derived"` and `provenance_key: "template_derived"`
- detector-backed entries also copy `source` and `confidence` from the matching `inferred[*]`
- template-derived entries use an `ota.init#...` source label so automation can distinguish starter defaults from repo evidence

Failure example:

```json
{
  "ok": false,
  "path": "./ota.yaml",
  "written": false,
  "error": "`./ota.yaml` already exists; ota init is only for repos without an ota contract\n\nNext:\n▸  review the existing contract with `ota validate`\n▸  review the existing contract with `ota doctor`\n▸  publish detected changes with `ota detect --candidate-out .ota/candidates/detect.json`",
  "next": "ota detect --candidate-out .ota/candidates/detect.json"
}
```

`ota init --packs --json` lists the available built-in starter packs without previewing one
contract. The current catalog includes `node`, `python`, `ruby`, `go`, `rust`, `dotnet`,
`php-composer`, `java-maven`, and `java-gradle`:

```json
{
  "ok": true,
  "mode": "catalog",
  "packs": [
    {
      "name": "node",
      "summary": "Conventional Node starter with toolchain-owned Node and first-class package-manager setup, dev, and test tasks.",
      "when": "Use this for repo-level Node apps or services that need an explicit JavaScript starter instead of detector-led init. The default path keeps Node ownership under `toolchains.node`, seeds first-class package-manager hydration for `setup`, and you can override the package manager with `--package-manager` when the repo is intentionally npm-, yarn-, or bun-based.",
      "command": "ota init --pack node",
      "next": "ota init --pack node --dry-run .",
      "does_not_infer": [
        "the repo's package manager unless `--package-manager` says so",
        "repo-specific script names or extra task variants beyond the seeded `setup`, `dev`, and `test` loop"
      ],
      "options": [
        {
          "flag": "--package-manager",
          "summary": "Choose the package manager used for setup and script execution.",
          "default": "pnpm",
          "values": ["npm", "pnpm", "yarn", "bun"]
        }
      ],
      "seeds": {
        "toolchains": ["node"],
        "runtimes": [],
        "tools": [],
        "checks": [],
        "tasks": ["setup", "dev", "test"]
      }
    },
    {
      "name": "ruby",
      "summary": "Conventional Ruby starter with toolchain-owned Ruby and Bundler-driven setup/test tasks.",
      "when": "Use this for Ruby repos that should start from `toolchains.ruby` ownership and the standard Bundler loop without relying on detector-led init.",
      "command": "ota init --pack ruby",
      "next": "ota init --pack ruby --dry-run .",
      "does_not_infer": [
        "framework-specific commands (for example Rails, Sinatra, or Hanami server entrypoints) beyond the seeded Bundler setup/test surface",
        "repo-specific test wrappers or flags beyond the seeded `bundle exec rake test` command"
      ],
      "seeds": {
        "toolchains": ["ruby"],
        "runtimes": [],
        "tools": [],
        "checks": [],
        "tasks": ["setup", "test"]
      }
    },
    {
      "name": "go",
      "summary": "Conventional Go starter with first-class module hydration, build, and test tasks.",
      "when": "Use this for Go module repos that should start from toolchain-owned Go module hydration plus the standard `go build` and `go test` flow without relying on detector-led init.",
      "command": "ota init --pack go",
      "next": "ota init --pack go --dry-run .",
      "does_not_infer": [
        "workspace layout, code generation, or custom build flags beyond the standard module download/build/test loop"
      ],
      "seeds": {
        "toolchains": ["go"],
        "runtimes": [],
        "tools": [],
        "checks": [],
        "tasks": ["setup", "build", "test"]
      }
    },
    {
      "name": "rust",
      "summary": "Conventional Rust starter with toolchain-owned Rust plus first-class Cargo hydration for `setup`, followed by build and test tasks.",
      "when": "Use this for Rust repos that should start from `toolchains.rust` ownership, first-class setup hydration through `prepare.kind: dependency_hydration`, and the standard `cargo build` / `cargo test` loop without relying on detector-led init.",
      "command": "ota init --pack rust",
      "next": "ota init --pack rust --dry-run .",
      "does_not_infer": [
        "workspace members, feature flags, or custom cargo aliases beyond the standard fetch/build/test loop"
      ],
      "seeds": {
        "toolchains": ["rust"],
        "runtimes": [],
        "tools": [],
        "checks": [],
        "tasks": ["setup", "build", "test"]
      }
    },
    {
      "name": "dotnet",
      "summary": "Conventional .NET starter with toolchain-owned .NET plus first-class `dotnet_restore` hydration for `setup`, followed by build and test tasks.",
      "when": "Use this for .NET repos that should start from `toolchains.dotnet` ownership, first-class setup hydration through `prepare.kind: dependency_hydration`, and the standard `dotnet build` / `dotnet test` loop without relying on detector-led init.",
      "command": "ota init --pack dotnet",
      "next": "ota init --pack dotnet --dry-run .",
      "does_not_infer": [
        "solution-specific target selection, test filtering, or custom dotnet CLI flags beyond the standard restore/build/test loop"
      ],
      "seeds": {
        "toolchains": ["dotnet"],
        "runtimes": [],
        "tools": [],
        "checks": [],
        "tasks": ["setup", "build", "test"]
      }
    },
    {
      "name": "php-composer",
      "summary": "Conventional PHP starter for Composer-managed repos with Composer install and optional existing test-script reuse.",
      "when": "Use this for Composer-managed PHP repos that should start from `composer install` and, when the repo already declares `scripts.test`, the existing Composer test script without relying on detector-led init.",
      "command": "ota init --pack php-composer",
      "next": "ota init --pack php-composer --dry-run .",
      "does_not_infer": [
        "framework-specific entrypoints, web server commands, or whether the repo uses phpunit, pest, artisan, or another test wrapper unless the repo already declares a Composer `scripts.test` entry"
      ],
      "seeds": {
        "toolchains": [],
        "runtimes": ["php"],
        "tools": ["composer"],
        "checks": ["php-installed", "composer-installed"],
        "tasks": ["setup"]
      }
    },
    {
      "name": "java-maven",
      "summary": "Conventional Java starter for Maven-driven repos with first-class Maven hydration for `setup`, plus build and test lifecycles, preferring `mvnw` when the repo already ships it.",
      "when": "Use this when the repo is intentionally Maven-based and you want an explicit Java starter without relying on repo detection. If `mvnw` already exists, ota uses wrapper-owned `prepare.kind: dependency_hydration` instead of requiring a global Maven install.",
      "command": "ota init --pack java-maven",
      "next": "ota init --pack java-maven --dry-run .",
      "does_not_infer": [
        "multi-module reactor details, plugin goals, or org-specific wrapper/bootstrap scripts beyond the standard Maven build/test loop"
      ],
      "seeds": {
        "toolchains": ["java"],
        "runtimes": [],
        "tools": [],
        "checks": [],
        "tasks": ["setup", "build", "test"]
      }
    },
    {
      "name": "java-gradle",
      "summary": "Conventional Java starter for Gradle-driven repos with first-class Gradle hydration for `setup`, plus build and test lifecycles, preferring `gradlew` when the repo already ships it.",
      "when": "Use this when the repo is intentionally Gradle-based and you want an explicit Java starter without relying on repo detection. If `gradlew` already exists, ota uses wrapper-owned `prepare.kind: dependency_hydration` instead of requiring a global Gradle install.",
      "command": "ota init --pack java-gradle",
      "next": "ota init --pack java-gradle --dry-run .",
      "does_not_infer": [
        "multi-project build logic, plugin conventions, or org-specific Gradle bootstrap beyond the standard build/test loop"
      ],
      "seeds": {
        "toolchains": ["java"],
        "runtimes": [],
        "tools": [],
        "checks": [],
        "tasks": ["setup", "build", "test"]
      }
    }
  ]
}
```

Each catalog entry now carries `does_not_infer` so automation can explain the deliberate boundary of
each starter pack instead of assuming the pack will absorb repo-specific workflow details.

Each catalog entry keeps the operator guidance machine-readable:

- `command` is the exact pack-selection command
- `next` is the safe dry-run preview command to review before writing
- `seeds` lists the unconditional starter fields, including shipped `toolchains` owners when a pack now starts from a managed ecosystem contract instead of separate `runtimes` / `tools`

## `ota check --json`

`ota check --json` uses the same finding shape as `ota doctor --json`, including additive
`finding_groups` when present. It may also include the same additive top-level `workflow`
summary and `toolchains[]` evidence for the selected workflow path, including workflow
`readiness_probes`, `readiness_surfaces`, `signal_readiness_*`, and `expose_surfaces` when declared:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "workflow": {
    "name": "app",
    "notes": "Use this for the primary readiness path.\n",
    "run_task": "dev",
    "required_services": ["postgres"],
    "readiness_checks": ["app-health"],
    "readiness_probes": ["app-ready"],
    "readiness_surfaces": ["backend"],
    "signal_readiness_checks": [],
    "signal_readiness_probes": [],
    "signal_readiness_surfaces": [],
    "exposes": []
  },
  "findings": [
    {
      "severity": "error",
      "summary": "...",
      "why": "...",
      "next": "..."
    }
  ]
}
```

Root monorepo summary output can also include grouped member findings under `members`.

## `ota up --json`

`ota up --json` has two failure classes:

- execution reached the `up` pipeline: returns `UpStatus` (`status`, `phase`, additive
  `governance`, `findings`, `receipt`, optional `service`/`task`/`exit_code`)
- contract load/validation failed before the `up` pipeline: returns `ValidateFailure` shape (`ok`, `path`, and either `errors` or `error`)

When an audited crossing actually occurs, `governance.crossing` and `receipt.crossing` carry the
same ota-authored record. That record now includes additive `evidence_classes` so downstream
consumers can tell which fields were caller-asserted, ota-derived from the decision inputs, or
runner-attested at the decision site itself.

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "status": "READY",
  "phase": "post-up diagnosis",
  "governance": {
    "preflight": {
      "state": "allowed",
      "crossing_required": false,
      "crossing_classification": "routine",
      "decision_inputs": [
        {
          "id": "workflow:verify",
          "family": "lane",
          "evidence_class": "derived",
          "replay_class": "pinned",
          "detail": "kind=workflow"
        },
        {
          "id": "actor_mode:non_agent",
          "family": "actor_mode",
          "evidence_class": "derived",
          "replay_class": "pinned"
        },
        {
          "id": "doctor_verdict:ready",
          "family": "readiness_gate",
          "evidence_class": "derived",
          "replay_class": "witnessed"
        },
        {
          "id": "receipt_expected:true",
          "family": "evidence_expectation",
          "evidence_class": "derived",
          "replay_class": "pinned"
        },
        {
          "id": "proof_expected:true",
          "family": "evidence_expectation",
          "evidence_class": "derived",
          "replay_class": "pinned"
        }
      ],
      "replay": {
        "status": "satisfied"
      },
      "evidence_classes": {
        "state": "derived",
        "crossing_required": "derived",
        "crossing_classification": "derived",
        "decision_inputs": "derived",
        "replay": "derived",
        "receipt_expected": "derived",
        "proof_expected": "derived"
      },
      "receipt_expected": true,
      "proof_expected": true
    },
    "post_execution": {
      "state": "evidence_missing",
      "execution_attempted": true,
      "refusal_occurred": false,
      "decision_inputs": [
        {
          "id": "execution_attempted:true",
          "family": "execution_observation",
          "evidence_class": "derived",
          "replay_class": "witnessed"
        },
        {
          "id": "receipt_present:true",
          "family": "receipt_observation",
          "evidence_class": "attested",
          "replay_class": "witnessed"
        },
        {
          "id": "proof_expected:true",
          "family": "proof_expectation",
          "evidence_class": "derived",
          "replay_class": "pinned"
        },
        {
          "id": "receipt_status:ready",
          "family": "receipt_observation",
          "evidence_class": "attested",
          "replay_class": "witnessed"
        },
        {
          "id": "proof_present:false",
          "family": "proof_observation",
          "evidence_class": "derived",
          "replay_class": "witnessed"
        },
        {
          "id": "crossing_record_state:not_required",
          "family": "crossing_observation",
          "evidence_class": "derived",
          "replay_class": "witnessed"
        }
      ],
      "replay": {
        "status": "satisfied"
      },
      "decision_basis": [
        {
          "id": "evidence:receipt_present",
          "family": "evidence_gate",
          "evidence_class": "attested"
        },
        {
          "id": "receipt_status:ready",
          "family": "receipt_evidence",
          "evidence_class": "attested"
        },
        {
          "id": "evidence:proof_missing",
          "family": "evidence_gate",
          "evidence_class": "derived"
        },
        {
          "id": "crossing_record:not_required",
          "family": "crossing_evidence",
          "evidence_class": "derived"
        }
      ],
      "evidence_classes": {
        "state": "derived",
        "execution_attempted": "derived",
        "refusal_occurred": "derived",
          "crossing_record_state": "derived",
          "decision_inputs": "derived",
          "replay": "derived",
          "receipt_present": "attested",
          "proof_present": "derived",
          "receipt_status": "attested"
      },
      "receipt_present": true,
      "proof_present": false,
      "receipt_status": "ready"
    }
  },
  "findings": [],
  "receipt": {
    "ok": true,
    "path": "/abs/path/to/ota.yaml",
    "scope": "repo",
    "contract": "/abs/path/to/ota.yaml",
    "contract_identity": {
      "version": 1,
      "project": {
        "name": "ota",
        "type": "application"
      },
      "metadata": {
        "owner": "ota"
      },
      "execution": {
        "preferred": "container",
        "lifecycle": "ephemeral",
        "supported": ["native", "container"],
        "image": "rust:1.94-bookworm"
      },
      "counts": {
        "runtimes": 0,
        "tools": 0,
        "env": 0,
        "services": 0,
        "checks": 0,
        "tasks": 1
      }
    },
    "backend": "native",
    "steps": [
      {
        "order": 1,
        "label": "post-up diagnosis",
        "status": "READY"
      }
    ],
    "summary": {
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1
    }
  }
}
```

Optional fields:

- `governance`: the canonical machine-readable governance verdict for the selected `up` path;
  `preflight` keeps boundary/block/refusal semantics distinct from `post_execution`, which reports
  what evidence actually exists after the attempted `up` lane; additive
  `preflight.decision_basis[]` carries the cited safety/refusal/crossing basis for the selected
  lane, while additive `post_execution.decision_basis[]` carries the cited non-run, evidence, and
  crossing-record basis for the resulting evidence state
- additive `preflight.decision_inputs[]` and `post_execution.decision_inputs[]` publish the
  replay-grade cited inputs behind those authoritative governance records; `replay_class:
  "pinned"` marks reusable pinned selectors, while `replay_class: "witnessed"` marks observed
  execution or evidence inputs
- additive `preflight.replay.status` and `post_execution.replay.status` publish whether ota can
  re-derive those emitted governance verdicts from the cited inputs it just recorded
- additive `preflight.evidence_classes` and `post_execution.evidence_classes` publish field-level
  provenance on those authoritative governance records, so consumers can distinguish ota-derived
  decision truth from boundary-attested receipt attachment truth
- `post_execution.state` is no longer a flatter success-only placeholder when ota already knows the
  evidence result: if execution happened but the expected proof bar was not met, ota emits
  `evidence_missing` and cites that exact basis in `post_execution.decision_basis[]`
- preview `governance.sandbox_policy` may also be present on `ota up --json --dry-run` when the
  selected workflow path carries compilable runtime-boundary truth; it repeats the same first
  `codex_local` sandbox target shape used by task and workflow discovery so preview consumers can
  recover writable-path and outbound-target posture directly from the selected `up` lane
- `receipt`: execution receipt for the executed repo `up` phase, including additive `receipt.contract_identity`; monorepo aggregate output keeps grouped `members` results instead of a top-level receipt
- `replay`: present when `ota up` was invoked with `--replay-baseline <selection>`; this carries
  the resolved baseline pointer (`source`, `selection_path`, `archive_path`, `promoted_at`, `ok`,
  archived `scope`, `last_known_good`),
  the selected lane scope, execution-authored replay posture (`replay_verified`,
  `replay_failed`, or `replay_unavailable`), hermeticity, optional `failure_kind`, a concise
  reason, the same compact comparison block used by receipt diff, and grouped
  introduced/resolved/unchanged counts
- `plan.dependency_steps[]`: additive selected setup/dependency-plane preview for the chosen `up`
  path; when a planned step is a structured hydration lane, `prepare.declared_hydration_provenance`
  and `prepare.resolved_hydration_provenance` publish the selected source posture, while
  `prepare.declared_uv_local_project` and `prepare.resolved_uv_local_project` keep declared and
  runner-observed local-project identity separate, on the same
  canonical carrier; the declared record preserves contract-owned `config_file` or explicit
  `sources[]`, while the resolved record adds parsed config-backed `source_identities[]` plus
  `resolution: resolved|unavailable` and an honest `resolution_error` when source recovery fails;
  explicit URL overrides retain URL identity without inventing a feed name; ambient default source
  selection remains `unavailable` because user/global NuGet configuration is not contract-owned
- `receipt.dependency_steps`: additive executed dependency-plane provenance for task-backed phases,
  using the same `task` / `backend` / optional `context` / optional `parent_task` /
  `backend_selection_source` shape as preview `plan.dependency_steps[]`
- `receipt.native_prerequisites`: additive selected native prerequisite detail for the executed native task/setup path, including provisioning guidance and any applied native activation
- `service`: present when a required service start command fails
- `task`: present when a task failure is reported
- `exit_code`: present when a child command failure is reported
- `members`: present on monorepo-root aggregate output with grouped member readiness results

Replay notes:

- `replay.comparison.artifact_trust[]` only covers artifacts captured by both the archived
  baseline and the current execution receipt
- matching `semantic_contract_snapshot` is acquitting for contract truth only
- matching `env_source_identity` is acquitting for the named declared env-source file class only
- matching task-declared `replay_inputs` remain narrowing evidence, so replay may be
  `replay_verified` with `hermeticity: partly_ambient`
- matching `policy_ruleset_identity` is acquitting for the named policy/ruleset class only
- informational findings remain visible in `findings[]` and the grouped diff counts, but do not
  by themselves make an otherwise ready replay witness stale when the archived and current
  execution boundaries match; replay drift remains sensitive to execution failure plus introduced
  or resolved warning/error findings
- a baseline from a different workflow, `backend`, `provider`, `target`, or `lifecycle` is not
  replay-comparable for the selected lane: Ota emits `replay_unavailable` with `failure_kind:
  baseline_scope_mismatch` rather than promoting that archive to `last_known_good`; compare
  `replay.baseline.scope` with `replay.scope` instead of inferring the mismatch from text
- `replay.failure_kind` is emitted only when replay is not verified:
  - `baseline_unavailable`
  - `baseline_scope_mismatch`
  - `semantic_contract_drift`
  - `named_input_drift`
  - `hidden_input_suspicion`
  - `witness_mismatch`
- `replay.baseline.last_known_good` is the replay-derived status of the selected baseline:
  - `replay_verified`
  - `stale_witness`
  - `unavailable`

Phase values:

- `preconditions`: prerequisite diagnosis or host/runtime gating blocked before prepare or setup
- `prepare`: workflow prepare execution failed
- `setup`: setup task execution failed
- `services`: required service start or readiness failed
- `run`: selected workflow run task exited before readiness was confirmed
- `readiness`: the runtime started, but readiness probes, surfaces, or readiness checks still
  failed or were the final proof-success boundary
- `cleanup`: runtime proof cleanup failed after artifact capture
- `interrupted`: runtime proof was interrupted by a signal

Example service-start failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "status": "SERVICE START FAILED",
  "phase": "services",
  "findings": [],
  "service": "postgres",
  "exit_code": 9
}
```

Example contract-validation failure (before `up` execution starts):

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "errors": [
    "tasks.build.run must not be empty"
  ]
}
```

`ota up --dry-run --json` is the read-only execution-plan preview surface for `ota up`.

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "dry_run": true,
  "status": "READY WITH WARNINGS",
  "preview_status": "RUNNABLE WITH WARNINGS",
  "phase": "preview",
  "summary": {
    "verdict": "risky",
    "agent_verdict": "not_ready",
    "error_count": 0,
    "warn_count": 1,
    "info_count": 0
  },
  "contract_identity": {
    "version": 1,
    "project": {
      "name": "ota"
    },
    "execution": {
      "preferred": "native",
      "lifecycle": "ephemeral"
    },
    "counts": {
      "runtimes": 0,
      "tools": 0,
      "env": 0,
      "services": 0,
      "checks": 0,
      "tasks": 1
    }
  },
  "execution": {
    "backend": "native",
    "lifecycle": "ephemeral",
    "task": "setup"
  },
  "overrides": {
    "backend": "native"
  },
  "plan": {
    "actions": [
      "run task `setup`",
      "re-check repo readiness"
    ]
  }
}
```

Current preview JSON fields:

- `ok`
- `path`
- `dry_run`
- `status`
- `phase` (`preview`)
- `summary` with the shared `doctor` / `check` verdict model; warning-only previews keep `ok: true` while surfacing `summary.verdict: "risky"`
- `contract_identity` with the declared project, selected metadata, execution intent, and compact contract counts
- `execution.backend`
- `execution.lifecycle` when one is selected
- `execution.image` when container execution is selected
- `execution.target` when the selected execution context has a real named target
- `execution.task` when `up` would run `setup`
- optional `overrides` with the explicit execution options admitted for the preview, including
  `host_port` when selected
- `plan.actions`
- `plan.skipped`
- `blockers`

Use `ota up --dry-run --json` when you need the selected backend and lifecycle plus the action and
skip plan without provisioning, starting services, or writing repo files. See
[up-preview.md](up-preview.md) for the preview contract.

## `ota receipt --json`

`ota receipt --json` is the read-only repo receipt artifact. It runs the same readiness scan as
repo diagnosis in the selected execution context, packages the result as an execution receipt, and
keeps the findings array alongside the receipt for CI and archival consumers. Contract
load/validation failures still emit the shared `ValidateFailure` JSON shape on stdout. When the
selected or effective workflow owns a rendered env artifact, receipt JSON keeps that under
`receipt.workflow_env_artifacts[]`.

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "receipt",
  "archive_path": "/abs/path/to/.ota/receipts/repo-receipt-20260411-113015-042Z.json",
  "summary": {
    "error_count": 0,
    "warn_count": 0,
    "info_count": 0,
    "step_count": 1
  },
  "receipt": {
    "ok": true,
    "path": "/abs/path/to/ota.yaml",
    "scope": "repo",
    "contract": "/abs/path/to/ota.yaml",
    "contract_identity": {
      "version": 1,
      "project": {
        "name": "receipt-demo"
      },
      "counts": {
        "runtimes": 0,
        "tools": 0,
        "env": 0,
        "services": 0,
        "checks": 0,
        "tasks": 1
      }
    },
    "contract_snapshot_hash": "sha256:5dc5c7f6e0bf...",
    "contract_snapshot_ref": ".ota/contracts/sha256-5dc5c7f6e0bf....json",
    "assumption_set_hash": "sha256:8b7e5f1c3a0d...",
    "evaluated_inputs": [
      {
        "id": "pnpm-lock.yaml",
        "kind": "lockfile",
        "input_class": "declared_dependency_resolution",
        "identity": "sha256:4ed6a1..."
      },
      {
        "id": "compose_image:compose.yaml#services.database",
        "kind": "container_image_digest",
        "input_class": "selected_runtime_artifact",
        "identity": "sha256:8b7e5f..."
      }
    ],
    "backend": "native",
    "workflow_env_artifacts": [
      {
        "path": ".env.docker-build",
        "kind": "dotenv",
        "profile": "docker-build",
        "includes": ["DATABASE_URL"],
        "exists": true,
        "consumers": ["task:build", "service:postgres"]
      }
    ],
    "steps": [
      {
        "order": 1,
        "label": "readiness",
        "stage_family": "proof",
        "status": "READY"
      }
    ],
    "summary": {
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1
    }
  },
  "findings": []
}
```

Current receipt JSON fields:

- `ok`
- `path`
- `mode` (`receipt`)
- `archive_path` (when `--archive` is set)
- `artifact_routing`
- `summary`
- `receipt`
- `findings`

The nested `receipt` object can also include:

- `contract_identity` with the declared project, selected metadata, execution intent, and compact contract counts
- `steps[*].stage_family` with the broad execution-governance family for that recorded step:
  `prepare`, `setup`, `verify`, `proof`, or `receipt`
- `artifact_routing[]` with the receipt/proof/snapshot artifact guide for the current receipt lane,
  including which artifact to inspect now and which archived artifact to keep when available
- `contract_snapshot_hash` with the normalized semantic contract snapshot identity used for this
  receipt; the hash is content-addressed and stable across formatting-only contract edits
- `contract_snapshot_ref` when Ota archived the normalized snapshot under `.ota/contracts`; plain
  read-only receipt JSON can still emit the hash without emitting a local archive ref. A persisted
  repo receipt archive requires both this reference and the matching `contract_snapshot_hash`;
  receipt history does not load the current contract as a substitute
- `archive_context` on a persisted repo receipt archive: `readiness` records a diagnosis-only
  lane. Authority-bearing `execution` archives use schema v2 and carry a canonical
  `semantic_scope` identity with the exact lane, closure graph, target platform, execution
  selection, workflow run behavior, and effect overrides. History re-derives it from the archived
  contract before deciding crossing necessity; it does not infer authority from global contract
  configuration or trust an editable lane label
- `invalid_archives[].posture: legacy_unverified` when an older archive lacks the immutable
  snapshot pair. These entries are visible for operator inspection but are never valid baseline,
  proof, or authority inputs
- `assumption_set_hash` with the canonical extracted assumption-set identity used for this
  receipt; the hash is derived from the normalized semantic path/value map rather than the raw
  archived snapshot file
- `evaluated_inputs[]` with execution inputs captured while Ota authored the receipt.
  Shipped records include clean git HEAD source identity as `source:git_head` when the selected
  repo worktree is inside git and has no uncommitted changes; declared Node lockfile identities:
  `pnpm-lock.yaml` for frozen pnpm and `package-lock.json` or authoritative `npm-shrinkwrap.json`
  for `npm ci`; `runtime:node` command version evidence for those selected typed lanes; and static
  digest-pinned Compose images for
  explicitly selected services and their declared `depends_on` closure in explicitly declared
  Compose files. It also includes task-declared replay inputs with distinct machine classes:
  generic `static_file`, `presentation_profile`, and `comparator_profile`. Ota never re-reads
  these identities later. Mutable tags, interpolated image
  references, inferred Compose files, unrelated stack services, ambient environment, and
  external-world inputs remain outside this evidence.
- `witnessed_observations.query_traces[]` when a selected task closure declares a JSONL query
  trace under `witnessed_observations.query_traces`. These are attested historical observations,
  not evaluated inputs. The nested summary reports subjects, records, and divergent subjects so
  consumers can distinguish stable query identity from observed variation without over-reading it
  as a current-run replay decision.
- a typed `evaluated_inputs[]` `hydration_provenance` record when `ota up` selects a structured
  hydration lane with source posture. Its runner-derived nested detail keeps contract-declared
  source posture separate from the resolved execution-time identity. For uv local-project hydration
  it also records manifest, optional lockfile, and clean source identity separately; it is not a
  historical query observation or a substitute for a later config-file read.
- the same selected Node hydration lane can also record `runtime:node` with the contract-local
  `node --version` observed while authoring the receipt. This is intentionally a runtime-version
  observation, not an executable or image digest.
- `backend`
- `workflow_env_artifacts` when the selected or effective workflow owns one rendered env artifact;
  each entry reports `path`, `kind`, `profile`, `includes`, `exists`, and the consuming
  task/service lanes
- `lifecycle`
- `image` when container execution is selected
- `target` when the recorded execution phase had a real named target, such as a persistent container, a named ephemeral task or diagnosis container, or a remote target
- `native_prerequisites` when the selected execution path uses native task prerequisites; each
  entry can include additive `check`, `activation`, `requires`, `provisioning`, and `note` fields
- `native_prerequisites[*].requires` records runtime, tool, toolchain, env, and check dependencies
  that came from `native_prerequisites.<name>.platforms.<os>.requires`, with a `source` marker so
  automation does not confuse native-bundle dependencies with direct task requirements
- `native_prerequisites[*].activation.applied` tells you whether this command actually ran inside
  the declared native activation; preview and read-only receipt paths can still report the
  declared activation with `applied: false`
- `execution_conflict.reasons[]` when the recorded failure was blocked by active execution
  ownership; entries use typed identities such as `active_execution_present`, `host_service`,
  `compose_project`, `persistent_backend_family`, `env_materialization_path`, `write_path`,
  `runtime_listener`, and `service_task`. Conflict owners can also expose additive
  `runtime_owners[]` entries with task, listener, namespace, protocol, address, port, and allocation
  posture (`fixed`, `managed_dynamic`, `isolated`, or `unresolved`). Internal-only container
  listeners use `isolated`; they are recorded without claiming or conflicting on a host listener.
  For a single fixed runtime-listener collision, receipt `next`/`next_steps` carries the same
  `--host-port <free port>` remediation as human output only when the selected lane admits that
  override; otherwise it points to the declared listener. Mixed conflicts identify that the port
  choice resolves only `runtime_listener` and retain the remaining typed reasons. Suggested
  commands preserve agent mode. The stable machine classification remains
  `active_execution_conflict` with `runtime_listener`; Ota does not silently remap an explicitly
  requested port.

`ok` mirrors the current repo receipt readiness result, so blocked repo receipts still return the
receipt success shape with `ok: false`.

When `--archive --promote-baseline` is set, the receipt success shape also includes:

- `promoted_baseline.path`
- `promoted_baseline.archive_path`
- `promoted_baseline.promoted_at`

## `ota receipt --json --snapshot`

`ota receipt --json --snapshot` reads one archived normalized semantic contract snapshot without
forcing operators to infer that truth indirectly through `ota diff` or receipt-correlation output.

Selection supports:

- `latest`: newest valid archived repo receipt for the same contract under `.ota/receipts`
- `promoted`: the explicit promoted repo baseline pointer under `.ota/receipts/repo-baseline.json`
- an explicit archived repo receipt JSON file path
- an explicit archived normalized snapshot JSON file path under `.ota/contracts`

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "snapshot",
  "summary": {
    "input_count": 1,
    "assumption_count": 3
  },
  "source": "latest",
  "selection_kind": "receipt_archive",
  "archive_path": ".ota/receipts/repo-receipt-20260621-101010-123Z.json",
  "archived_at": "2026-06-21T10:10:10.123Z",
  "snapshot_hash": "sha256:5dc5c7f6e0bf...",
  "assumption_set_hash": "sha256:9ab21d35d3ef...",
  "snapshot_path": ".ota/contracts/sha256-5dc5c7f6e0bf....json",
  "contract": {
    "contract": "/abs/path/to/ota.yaml",
    "contract_identity": "ota.yaml"
  },
  "snapshot": {
    "version": 1,
    "project": {
      "name": "receipt-demo"
    },
    "tasks": {
      "setup": {
        "run": "echo ready"
      }
    }
  }
}
```

Snapshot JSON fields:

- `ok`
- `path`
- `mode` (`snapshot`)
- `summary.input_count`
- `summary.assumption_count`
- `source` (`latest`, `promoted`, or `file`)
- `selection_kind` (`receipt_archive` or `snapshot_archive`)
- `selection_path` when the caller selected an explicit file or promoted pointer
- `archive_path` when the resolved selection came from a receipt archive or direct snapshot file
- `archived_at` and `promoted_at` when the resolved selection came through archived receipt state
- `snapshot_hash`
- `assumption_set_hash`
- `snapshot_path`
- `contract.contract` when the resolved receipt carried the source contract path
- `contract.contract_identity` when the resolved receipt carried compact contract identity
- `snapshot` with the normalized archived semantic contract JSON

## `ota receipt --json --baseline`

`ota receipt --json --baseline` compares the current repo receipt against either:

- `promoted`: the explicit promoted repo baseline under `.ota/receipts/repo-baseline.json`
- `latest`: the newest valid archived repo receipt for the same contract under `.ota/receipts`
- an explicit repo receipt JSON file path

The compare path is read-only. It does not rerun the baseline receipt, does not archive the
current receipt automatically, and exits `0` when comparison succeeds even if the current receipt
or baseline receipt is not ready. Add `--fail-on-new-blockers` when you want compare mode to exit
`1` after a successful diff whenever the current receipt introduces new blocker findings.

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "mode": "diff",
  "baseline": {
    "source": "promoted",
    "selection_path": "/abs/path/to/.ota/receipts/repo-baseline.json",
    "archive_path": "/abs/path/to/.ota/receipts/repo-receipt-20260412-101010-123Z.json",
    "archived_at": "2026-04-12T10:10:10.123Z",
    "promoted_at": "2026-04-12T10:20:30.456Z",
    "contract_identity": "ota.yaml",
    "contract_snapshot_hash": "sha256:1111111111111111111111111111111111111111111111111111111111111111",
    "contract_snapshot_ref": ".ota/contracts/sha256-1111111111111111111111111111111111111111111111111111111111111111.json",
    "assumption_set_hash": "sha256:3333333333333333333333333333333333333333333333333333333333333333",
    "contract_identity_details": {
      "project": {
        "name": "receipt-diff"
      },
      "counts": {
        "tasks": 1
      }
    },
    "ok": false,
    "contract": "/abs/path/to/ota.yaml",
    "backend": "native",
    "summary": {
      "error_count": 2,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1
    }
  },
  "current": {
    "ok": false,
    "contract": "/abs/path/to/ota.yaml",
    "contract_identity": "ota.yaml",
    "contract_snapshot_hash": "sha256:2222222222222222222222222222222222222222222222222222222222222222",
    "assumption_set_hash": "sha256:4444444444444444444444444444444444444444444444444444444444444444",
    "contract_identity_details": {
      "project": {
        "name": "receipt-diff"
      },
      "counts": {
        "env": 1,
        "tasks": 1
      }
    },
    "backend": "native",
    "summary": {
      "error_count": 2,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 1
    }
  },
  "summary": {
    "baseline_ok": false,
    "current_ok": false,
    "comparison": {
      "baseline_identity_label": "ota.yaml",
      "current_identity_label": "ota.yaml",
      "identity_changed": false,
      "readiness_change": "unchanged",
      "correlation": "likely_related",
      "contract_snapshot_changed": true
    },
    "introduced": {
      "count": 1,
      "error_count": 1,
      "warn_count": 0,
      "info_count": 0
    },
    "resolved": {
      "count": 1,
      "error_count": 1,
      "warn_count": 0,
      "info_count": 0
    },
    "unchanged": {
      "count": 1,
      "error_count": 1,
      "warn_count": 0,
      "info_count": 0
    }
  },
  "contract_changes": [
    {
      "path": "env.vars.OTA_BASELINE_REQUIRED.required",
      "status": "add",
      "category": "env",
      "risk": "medium"
    }
  ],
  "likely_related_changes": [
    {
      "path": "env.vars.OTA_BASELINE_REQUIRED.required",
      "status": "add",
      "category": "env",
      "risk": "medium"
    }
  ],
  "gate": {
    "rule": "fail_on_new_blockers",
    "passed": false,
    "new_blocker_count": 1,
    "blocking_summary": "Missing environment variable: OTA_BASELINE_REQUIRED",
    "blocking_next": "set `OTA_BASELINE_REQUIRED`, then rerun `ota doctor`",
    "blocking_provenance": "repo contract",
    "blocking_provenance_key": "repo_contract"
  },
  "introduced": [
    {
      "summary": "Missing environment variable: OTA_BASELINE_REQUIRED"
    }
  ],
  "resolved": [
    {
      "summary": "Missing tool: old-tool"
    }
  ],
  "unchanged": [
    {
      "summary": "No tasks defined in contract"
    }
  ]
}
```

Current receipt diff JSON fields:

- `ok` (current receipt readiness, not diff success/failure)
- `path`
- `mode` (`diff`)
- `baseline.source` (`promoted`, `latest`, or `file`)
- `baseline.selection_path` when compare selection came from a promoted baseline pointer or explicit file path
- `baseline.archive_path`
- `baseline.archived_at` (when the baseline file name encodes an archived timestamp)
- `baseline.promoted_at` when compare selection came from a promoted baseline pointer
- `baseline.contract_identity` when ota can resolve the repo-local contract identity for the selected baseline
- `baseline.contract_snapshot_hash` when the archived baseline carries normalized semantic contract identity
- `baseline.contract_snapshot_ref` when the archived baseline points at a normalized archived contract snapshot under `.ota/contracts`
- `baseline.assumption_set_hash` when the archived baseline carries canonical extracted assumption-set identity
- `baseline.evaluated_inputs[]` when the archived baseline captured immutable evaluated inputs
- `baseline.contract_identity_details` with the compact declared contract identity when the archived receipt recorded it
- `baseline.ok`
- `baseline.contract`
- `baseline.backend` / `baseline.lifecycle` when recorded
- `baseline.summary`
- `current.ok`
- `current.contract`
- `current.contract_identity` with the current repo-local contract identity
- `current.contract_snapshot_hash` with the normalized semantic contract hash for the current in-memory contract truth, even when the current receipt is not archived
- `current.assumption_set_hash` with the canonical extracted assumption-set identity for the current in-memory contract truth
- `current.evaluated_inputs[]` with immutable inputs captured while Ota authored the current receipt
- `current.contract_identity_details` with the compact declared contract identity for the current receipt
- `current.backend` / `current.lifecycle` when recorded
- `current.summary`
- `summary.baseline_ok`
- `summary.current_ok`
- additive `summary.comparison` with baseline/current identity labels plus compact `identity_changed`, `readiness_change`, and `contract_snapshot_changed` drift signals
- `summary.comparison.replay` names the selected workflow/runtime scope and replay trust posture. A
  plain `ota receipt --baseline` comparison is always `witness_only` with `hermeticity:
  unassessed`: it compares captured witnesses but does not execute the lane, so it must not be
  read as replay verification.
- `summary.comparison.artifact_trust[]` only for artifacts captured on both sides; each record names
  its input class, immutable identities, comparison state, and runner-derived trust role. The
  first shipped record is `semantic_contract_snapshot`, which is `acquitting` only for the named
  `contract_truth` class when its identities match, never for dependency, environment, runtime,
  or external-world inputs that the receipts did not capture. A matching clean `source:git_head`
  record is `acquitting` only for the named `source_identity` class; dirty or non-git worktrees
  do not emit that record. A matching captured
  typed Node lockfile is likewise `acquitting` only for `declared_dependency_resolution`. A
  matching static digest-pinned Compose image is `acquitting` only for the named
  `selected_runtime_artifact` input; it does not clear other runtime, environment, or external
  state classes.
  A matching `presentation_profile` replay input is `acquitting` only for the declared
  `execution_presentation_profile` class on that lane. A matching `comparator_profile` replay
  input is `narrowing` for `comparator_semantics`: it proves the comparator definition held still,
  but not that the whole runtime presentation was hermetic.
  A matching `runtime:node` record is `narrowing` for `selected_runtime_version`: it rules out a
  reported Node version change, but does not prove binary, image, host, environment, or external
  state identity.
- `summary.comparison.correlation` with an explicit advisory verdict:
  - `likely_related` when ota can correlate newly introduced blocker findings to specific semantic contract changes
  - `possibly_related` when ota cannot recover a strong direct match but the newly introduced blocker and contract drift still overlap in the same broad contract family
  - `no_clear_correlation` when ota cannot recover even that coarse overlap honestly
- `summary.introduced`
- `summary.resolved`
- `summary.unchanged`
- additive `contract_changes[]` with semantic normalized contract diff entries between the archived baseline snapshot and the current contract truth when the baseline carries `receipt.contract_snapshot_ref`
- additive `likely_related_changes[]` when ota can correlate one or more newly introduced blocker findings to those semantic contract changes
  - ordering prefers the sharpest declared semantic owner or named reference ota can recover
    honestly, for example reusable `surfaces.<name>` or `readiness.probes.<name>`, before broader
    adjacent workflow-family drift
  - when several nearby semantic changes tie on directness, ota then prefers the change whose
    governance stage best matches the failure lane, so check failures prefer verify-lane drift,
    setup blockers prefer prepare-lane drift, and readiness/proof failures prefer proof-lane drift
- `gate.rule`, `gate.passed`, and `gate.new_blocker_count` when `--fail-on-new-blockers` is active
- additive `gate.blocking_summary`, `gate.blocking_next`, and provenance fields when the gate is blocked by at least one newly introduced error
- `introduced[]`
- `resolved[]`
- `unchanged[]`

## `ota receipt --json --history`

`ota receipt --json --history` is the read-only archive index for repo receipts already written to
`.ota/receipts`. It does not rerun diagnosis; it lists archived receipt files newest first.

```json
{
  "ok": true,
  "path": "/abs/path/to/repo",
  "mode": "history",
  "history_source": "local",
  "completeness_posture": "local_archive_directory_observed",
  "summary": {
    "archive_count": 2,
    "invalid_archive_count": 1
  },
  "archives": [
    {
      "archive_path": "/abs/path/to/.ota/receipts/repo-receipt-20260412-091512-142Z.json",
      "archived_at": "2026-04-12T09:15:12.142Z",
      "ok": false,
      "contract": "/abs/path/to/ota.yaml",
      "backend": "native",
      "summary": {
        "error_count": 1,
        "warn_count": 0,
        "info_count": 0,
        "step_count": 1
      }
    }
  ],
  "invalid_archives": [
    {
      "archive_path": "/abs/path/to/.ota/receipts/repo-receipt-20260412-090000-000Z.json",
      "posture": "invalid",
      "error": "failed to parse receipt archive `./.ota/receipts/repo-receipt-20260412-090000-000Z.json`: EOF while parsing a value at line 1 column 10"
    }
  ]
}
```

Current receipt history JSON fields:

- `ok`
- `path` (resolved repo boundary for the archive read)
- `mode` (`history`)
- `history_source` (`local | systemd_protected_launcher`)
- `completeness_posture` (`local_archive_directory_observed` for the ordinary archive directory,
  or `complete_selected_catalog_snapshot` for a fully returned protected manifest)
- `operator_profile_posture`, `operator_profile_identity`, and `operator_peer_identity` for the
  protected least-privilege `non_agent` operator session
- `repository_binding_identity`, `catalog_namespace_identity`, and `catalog_snapshot_identity`
  for protected history; these are bounded selection evidence, not human or CI identity
- `summary.archive_count`
- `summary.invalid_archive_count`
- `archives[]`
- `archives[].archive_path`
- `archives[].archive_identity` and `archives[].catalog_identity` for protected history; local
  history omits both because it has no protected catalog selection
- `archives[].archived_at`
- `archives[].ok`
- `archives[].contract`
- `archives[].backend` (when the archived receipt recorded one)
- `archives[].provider` (when the archived receipt recorded one)
- `archives[].lifecycle` (when the archived receipt recorded one)
- `archives[].cwd` (when the archived receipt recorded one)
- `archives[].summary`
- `invalid_archives[]` when malformed archive files were skipped
- `invalid_archives[].archive_path`
- `invalid_archives[].posture` (`legacy_unverified | invalid`)
- `invalid_archives[].error`

`ota receipt --json --history --source systemd_protected_launcher` is Linux-only and uses the
fixed protected Launcher history socket. It accepts no path, `--file`, or `OTA_FILE` override and
never falls back to local archives. Launcher proves protected acquisition, catalog, and object
integrity; Core reconstructs the exact archive, immutable contract snapshot, and finalization
sidecar bytes and remains the sole semantic receipt verifier.

When `--member <name>` is set against a monorepo root, `receipt.contract` points at the selected
member contract path while the readiness findings reflect the merged member target.

When the receipt comes from remote execution, `receipt.provider` names the remote transport,
`receipt.target` names the resolved remote boundary, and `receipt.cwd` carries the declared remote
working directory when one exists.

Use `ota receipt --json` when you need a deterministic repo-local artifact for the current
readiness state without provisioning, starting services, or writing repo files.
Add `--archive` to persist the JSON receipt in `.ota/receipts` for later audit; ota keeps
the newest 50 archives.

## `ota clean --json`

`ota clean --json` reports repo-scoped cleanup counters for the selected contract target. When
the target is a monorepo root with workspace members, ota returns a root cleanup report plus
member reports. When cleanup fails, ota returns either a classified cleanup failure with
machine-readable engine and reason fields, or a generic repo-state failure when ota cannot
honestly classify the issue as a cleanup engine/resource problem.

Single-target success:

```json
{
  "ok": true,
  "path": "./ota.yaml",
  "summary": {
    "removed_current_persistent_containers": 1,
    "removed_drift_persistent_containers": 0,
    "removed_drift_attached_containers": 0,
    "removed_current_dependency_isolation_volumes": 1,
    "removed_drift_dependency_isolation_volumes": 0,
    "skipped_ambiguous_persistent_containers": 0,
    "skipped_ambiguous_dependency_isolation_volumes": 0,
    "stopped_host_services": 0,
    "total_removed": 2,
    "total_skipped_ambiguous": 0
  },
  "host_services": [],
  "queried_engines": [
    "docker"
  ]
}
```

Monorepo root success:

```json
{
  "ok": true,
  "path": "./ota.yaml",
  "workspace": {
    "root": {
      "ok": true,
      "path": "./ota.yaml",
      "summary": {
        "removed_current_persistent_containers": 0,
        "removed_drift_persistent_containers": 0,
        "removed_drift_attached_containers": 0,
        "removed_current_dependency_isolation_volumes": 0,
        "removed_drift_dependency_isolation_volumes": 0,
        "skipped_ambiguous_persistent_containers": 0,
        "skipped_ambiguous_dependency_isolation_volumes": 0,
        "stopped_host_services": 0,
        "total_removed": 0,
        "total_skipped_ambiguous": 0
      },
      "host_services": [],
      "queried_engines": [
        "docker"
      ]
    },
    "members": [
      {
        "member": "api",
        "report": {
          "ok": true,
          "path": "./ota.yaml#api",
          "summary": {
            "removed_current_persistent_containers": 1,
            "removed_drift_persistent_containers": 0,
            "removed_drift_attached_containers": 0,
            "removed_current_dependency_isolation_volumes": 0,
            "removed_drift_dependency_isolation_volumes": 0,
            "skipped_ambiguous_persistent_containers": 0,
            "skipped_ambiguous_dependency_isolation_volumes": 0,
            "stopped_host_services": 0,
            "total_removed": 1,
            "total_skipped_ambiguous": 0
          },
          "host_services": [],
          "queried_engines": [
            "docker"
          ]
        }
      }
    ]
  }
}
```

Failure:

```json
{
  "ok": false,
  "path": "./ota.yaml",
  "summary": "Container engine unavailable",
  "error": "task `clean` could not list dependency-isolation volume `dev.ota.repo=repo-1` using container engine `podman`: Cannot connect to Podman",
  "why": "`ota clean` needs Podman to remove dependency-isolation repo state for `dev.ota.repo=repo-1`, but Podman is not reachable.",
  "next": [
    "start Podman and rerun `ota clean`",
    "run `podman system connection list`",
    "if needed, run `podman machine init` and `podman machine start`"
  ],
  "reason": "engine_unavailable",
  "engine": "podman",
  "action": "list",
  "resource_kind": "dependency_isolation_volume",
  "resource_name": "dev.ota.repo=repo-1",
  "details": "unable to connect to Podman socket: dial tcp 127.0.0.1:57990: connect: connection refused"
}
```

Generic repo-state failure:

```json
{
  "ok": false,
  "path": "./ota.yaml",
  "summary": "Cleanup failed",
  "error": "projection `dev.http.host.port` must declare either `fixed` or `auto`"
}
```

## `ota clean --stale --json`

`ota clean --stale --json` is contract-free. It reports exited ota-managed containers that match
the local cleanup scan and tells automation whether the command removed them or only previewed
them. If no local container engine can be queried, ota returns the same structured cleanup failure
shape used by `ota clean --json` instead of this success shape.

```json
{
  "ok": true,
  "scope": "stale",
  "dry_run": false,
  "engines": [
    "docker"
  ],
  "summary": {
    "matched_count": 2,
    "removed_count": 2,
    "would_remove_count": 0
  },
  "containers": [
    {
      "engine": "docker",
      "name": "ota-a6be4471a4598386",
      "ownership": "label"
    },
    {
      "engine": "docker",
      "name": "ota-legacydeadbeef",
      "ownership": "legacy_name"
    }
  ]
}
```

`ownership` is `label` for containers matched through ota's management labels and
`legacy_name` for older `ota-*` containers that predate labels.

## `ota detect --json`

`ota detect --json` uses the same read-only success shape whether or not a contract exists. When
`ota.yaml` exists, it includes `comparison` without granting mutation authority.

- `written: false` for ordinary detection and candidate publication
- `config` is the detected candidate contract for preview-only results; when `--write` succeeds,
  it is the exact first contract Ota wrote, including `metadata.ota.detect.field_ownership` and
  `metadata.ota.detect.field_admission`
- `config.metadata.ota.detect.field_admission[*]` is `direct` when ota wrote the field from direct high-confidence detector evidence and `promoted` when ota admitted the field through the conservative detect-write promotion policy
- `config.metadata.ota.detect.field_source_class[*]` records the detector-governance class ota associated with each detect-owned field it wrote, such as `task_command` or `environment_toolchain`
- `write_candidate` appears only after a successful `detect --write`; it records the exact
  source-bound candidate `identity`, schema version, and
  `detect_conservative_first_contract_v1` profile whose verified projection became `ota.yaml`
- `comparison` describing detected adds and updates against the existing contract
- `comparison.removals` describing stale contract fields that are no longer detected in the repo
- `comparison.changes[*].ownership` is `repo_signals` for add candidates and `repo_contract` for updates against existing fields
- `comparison.removals[*].ownership` is `repo_contract` because those entries describe stale declared contract data
- `comparison.changes[*].owner_kind` is `detected` for add candidates, `manual` for default hand-authored existing fields, and `merged` when ota previously wrote the field and recorded it under `metadata.ota.detect.field_ownership`
- `comparison.removals[*].owner_kind` identifies detect-owned stale fields without implying that Ota will remove them
- `comparison.*.provenance` preserves the stable machine label `repo_signals`
- `comparison.*.provenance_key` is the stable machine label `repo_signals`
- `comparison.changes[*].source`, `comparison.changes[*].source_class`, and `comparison.changes[*].confidence` copy the detector evidence for that proposed add or update so consumers do not need to join back to `inferred[*]`
- `comparison` may include lower-confidence add candidates that remain preview-only
- `toolchain_opportunities` appears only when repo signals strongly suggest a managed ecosystem
  that ota still models through lower-level `runtimes` / `tools` declarations because no shipped
  provider contract owns it yet

`ota detect --candidate-out PATH --json` keeps `written: false` because it does not modify
`ota.yaml`, and adds:

- `candidate_path`: the written review-artifact path
- `candidate`: the complete self-verifying `contract-candidate.json` artifact

The candidate binds the observed discovery inventory with a required content identity for every
selected regular file, every selected source-evidence tuple, existing `ota.yaml` bytes when
present, implementation identity, proposed changes, and conservative task closure records. Ota
derives every candidate field from one command-owned immutable snapshot of the selected discovery
sources; it never emits a candidate assembled from mixed repository reads. Each change carries
`subject.path` as ordered contract-path segments and a field-family-specific canonical semantic
value. This preserves dotted map keys and lets consumers distinguish semantic task execution from
its YAML spelling. Ota omits proposals already represented by equivalent existing truth and marks
real disagreement as `conflict`, not `applicable`. Existing-truth traversal supports canonical
array indices, and task-command comparison normalizes schema-equivalent defaults including
`interaction: auto` and root `cwd: .`. When every selected applicable operation produces a
complete valid contract, `application_projection` binds the exact normalized operations, reviewed
base-contract identity, and resulting semantic contract identity. Candidates whose applicable
operations cannot produce a complete valid contract omit that projection and cannot enter
application admission. Unrelated `unknown` or `unsupported` residual findings do not make a
projection incomplete; they remain review state and `--require-complete` refuses them. A candidate
remains review input only; it does not establish agent safety or authorize execution.
Exact CI verifier commands may retain job-scoped lane identity when one workflow separates, for
example, unit and integration tests. Their closure records can carry the selected Cargo toolchain,
observed runner platform, service names/images, and environment requirement names. Those
observations are source-bound review evidence, not contract authority: CI-derived tasks remain
`unknown`, service or environment values are not promoted into `ota.yaml`, and no lane becomes
agent-safe without reviewed contract truth.

`ota contract apply-candidate CANDIDATE [--write] [--carrier git] --json [PATH]` is the matching admission
surface. Without `--write`, it returns `ok: true`, `mode: "dry_run"`, and `written: false` only
when current repository truth re-derives the exact candidate. With `--write`, Ota locks the
repository candidate lane, repeats that admission, rechecks current source and evidence, and
atomically creates a previously absent `ota.yaml` from the `Contract` returned by the shared
evaluator. Existing-contract updates require the explicit Git carrier, which requires a
non-detached tracked `ota.yaml` that matches `HEAD` in both the index and worktree, scrubs caller
`GIT_*` routing state and disables configured Git helper execution, uses expected-HEAD branch
compare-and-swap, and then verifies that the
worktree and index match the committed contract. A successful reapplication where
the current semantic contract already equals the reviewed result returns `mode: "write"`,
`written: false`, and `no_op: true`. Every success carries the reviewed candidate and
implementation identities plus the application-projection and resulting-contract identities. Git
carrier success also carries `carrier: "git"`, `branch_ref`, `previous_commit`, and
`resulting_commit`. A
ordinary refusal returns `ok: false`, `written: false`, one stable `code`, and an actionable
`error`; current codes are `candidate_malformed`, `candidate_identity_invalid`,
`candidate_implementation_incompatible`, `candidate_not_reproducible`, `candidate_stale`,
`candidate_contract_mismatch`, `candidate_conflict`, `candidate_incomplete`,
`candidate_unsupported`, `candidate_write_unsupported_platform`,
`candidate_write_failed`, and
`candidate_write_durability_uncertain`, and `candidate_write_committed_worktree_unsynced`. A
durability-uncertain or committed-but-worktree-unsynced failure has `written: true`, so
callers must inspect the contract before retrying. The committed-but-worktree-unsynced form also
carries `carrier: "git"`, `previous_commit`, `resulting_commit`, and `branch_ref` for recovery.
Residual `unknown` and `unsupported`
dispositions remain visible as review state and become a refusal only with `--require-complete`.
Both writers currently require Linux or macOS no-follow directory-descriptor support; the
create-new writer additionally uses Linux `renameat2(RENAME_NOREPLACE)` or macOS
`renameatx_np(RENAME_EXCL)`. Other platforms refuse rather than weaken publication guarantees.
Git-carrier write requests on those platforms return
`candidate_write_unsupported_platform` before candidate loading, repository locking, Git
invocation, or mutation; dry-run candidate admission remains available.
Removed repo-level legacy mutation flags return `detect_legacy_mutation_removed`; workspace
mutation remains a separate surface until it has its own candidate/apply model.

Candidate artifact publication uses the same platform boundary: Linux
`renameat2(RENAME_NOREPLACE)` or macOS `renameatx_np(RENAME_EXCL)` through a retained no-follow
directory descriptor. It never uses a temporary hardlink alias; other platforms refuse. Candidate
publication output is deliberately separate from contract mutation: `candidate_published` states
whether the artifact became visible, `candidate_publication` is `durable`, `not_published`, or
`durability_uncertain`, and `written` remains false because `ota.yaml` was not changed. A
`durability_uncertain` result means the candidate may exist but its directory-entry durability
was not proved; failure output retains `candidate_path` so automation can inspect the exact path
before retrying.

`ota contract upgrade --candidate-out PATH --json [PATH]` produces the schema-v2 upgrade branch of
`contract-candidate.json` and never changes `ota.yaml`. Its first registered migration is
`legacy_flat_toolchain_fulfillment_v1`, which replaces only legacy flat
`toolchains.<name>.fulfillment: run|none` values with the structured `fulfillment.mode` spelling.
The candidate binds the exact source bytes, registered migration implementation, semantic identity
before and after migration, canonical resulting-content identity, ordered replacement operations,
and application projection. A success uses `contract-upgrade.json`, returns `written: false`,
`candidate_published: true`, and `candidate_publication: "durable"`, and embeds the full
candidate. Failure output preserves the same publication distinction and returns
`upgrade_unsupported`, `upgrade_failed`, or `upgrade_write_failed`. `ota contract
apply-candidate` re-derives the exact migration for dry-run admission. Its `--write --carrier git`
form is the first owned existing-contract update path; ordinary `--write` remains create-new-only.

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "written": false,
  "config": {
    "version": 1
  },
  "comparison": {
    "existing_contract": true,
    "changes": [
      {
        "field": "project.name",
        "status": "update",
        "existing": "existing",
        "detected": "ota-web",
        "owner_kind": "manual",
        "ownership": "repo_contract",
        "provenance": "repo_signals",
        "provenance_key": "repo_signals",
        "source": "package.json#name",
        "confidence": "high"
      }
    ],
    "removals": [
      {
        "field": "tools.cargo",
        "existing": "1.78",
        "owner_kind": "merged",
        "ownership": "repo_contract",
        "provenance": "repo_signals",
        "provenance_key": "repo_signals"
      }
    ]
  },
  "inferred": [
    {
      "field": "runtimes.node",
      "value": "22",
      "source": ".nvmrc",
      "confidence": "high"
    }
  ]
}
```

In conservative mixed-repo or legacy-repo cases, `comparison.changes` can include `add` entries
while `written` remains `false`. Apply only a reviewed candidate through `apply-candidate`.

Failure example:

```json
{
  "ok": false,
  "path": "./ota.yaml",
  "written": false,
  "error": "`./ota.yaml` already exists; detect --write is create-new-only",
  "next": "ota detect --candidate-out .ota/candidates/detect.json ."
}
```

Removed repo-level mutation flags return `code: "detect_legacy_mutation_removed"`, the exact
`removed_flags`, and `replacement: "ota detect --candidate-out .ota/candidates/detect.json"`.
They refuse before repository access. Rewrite/removal intentionally has no replacement carrier.

## `ota workspace up --json`

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "error_count": 0,
    "warn_count": 0,
    "info_count": 0,
    "step_count": 4
  },
  "receipt": {
    "ok": true,
    "path": "/abs/path/to/ota.workspace.yaml",
    "scope": "workspace",
    "contract": "/abs/path/to/ota.workspace.yaml",
    "contract_identity": {
      "version": 1,
      "project": {
        "name": "ota-dev",
        "type": "workspace"
      },
      "counts": {
        "runtimes": 0,
        "tools": 0,
        "env": 0,
        "services": 0,
        "checks": 0,
        "tasks": 0,
        "repos": 1,
        "policies": 0
      }
    },
    "workspace": "ota-dev",
    "env_sources": [
      {
        "name": "OTA_TEST_SHARED",
        "value": "workspace-policy",
        "source": "workspace policy"
      }
    ],
    "steps": [
      {
        "order": 1,
        "label": "web",
        "status": "READY",
        "detail": "service `db`"
      }
    ],
    "summary": {
      "error_count": 0,
      "warn_count": 0,
      "info_count": 0,
      "step_count": 4
    }
  },
  "repos": [
    {
      "name": "web",
      "path": "/abs/path/to/apps/web",
      "contract_path": "/abs/path/to/apps/web/ota.yaml",
      "required": true,
      "ok": true,
      "status": "READY",
      "phase": "post-up diagnosis",
      "findings": []
    }
  ]
}
```

`ota workspace refresh --json` uses the same workspace roll-up shape, but reports refresh
status for existing repos instead of bootstrap status for missing ones. It switches `mode`
between `"preview"` and `"refresh"`, and the dedicated schema is `workspace-refresh.json`.
It always includes the shared workspace `receipt`, and in preview mode it does not mutate repo state.

Workspace repo items may also include additive `next` and `next_steps` when ota can name that
repo's current follow-up lane directly.

`ota workspace diff --json` uses a read-only workspace diff roll-up. It reports local git
state against the declared source ref or upstream branch, includes per-repo `status`,
`drift_kind`, `target_source`, `branch`, `head`, `target_ref`, `ahead`, `behind`, and `dirty`
fields, and adds `"mode": "diff"`. Additive top-level `next` and `next_steps` are present when
ota can name the safest acquisition or refresh follow-up lane directly, and per-repo items can
also carry additive `next` and `next_steps`. `summary` now also breaks the previously collapsed
`missing` and `unresolved` buckets into additive `missing_repo_count`, `missing_contract_count`,
`target_unavailable_count`, and `comparison_unresolved_count`.

When `ota workspace diff --json --progress-json` is used, Ota also emits live workspace progress
events as compact one-line JSON on `stderr`. Those progress events use the same
`workspace_progress` event shape as `ota workspace doctor|check|status|receipt|run|up|refresh --json --progress-json`,
use `status` for the repo diff verdict such as `MATCH`, `DIRTY`, or `UNRESOLVED`, and use `tail`
for the machine `drift_kind`, while the final workspace-diff report remains the single JSON
document on `stdout`.

`ota workspace status --json` uses the operational workspace roll-up. It reports readiness and
local git drift together, includes per-repo `ready`, `readiness_status`, `drift_status`,
`drift_kind`, `target_source`, `branch`, `head`, `target_ref`, `ahead`, `behind`, and `dirty`
fields, and adds `"mode": "status"`. Additive top-level `next` and `next_steps` are present when
ota can name the safest doctor, acquisition, or refresh follow-up lane directly, and per-repo
items can also carry additive `next`, `next_steps`, and `workflow`. `target_source` is `declared_ref` when
the comparison target came from `repos.<name>.source.ref` and `upstream_branch` when ota fell
back to the repo's configured upstream branch. `summary` now also breaks the previously collapsed
`missing` and `unresolved` buckets into additive `missing_repo_count`, `missing_contract_count`,
`target_unavailable_count`, and `comparison_unresolved_count`.

When `ota workspace status --json --progress-json` is used, Ota also emits live workspace progress
events as compact one-line JSON on `stderr`. Those progress events use the same
`workspace_progress` event shape as `ota workspace doctor|check|run|up|refresh --json --progress-json`,
and use `tail` for the repo drift state such as `MATCH`, `DIRTY`, or `UNRESOLVED`, while the
final workspace-status report remains the single JSON document on `stdout`.

`ota workspace execution plan --json` uses a read-only execution roll-up. It reports one
resolved or unresolved execution decision per selected repo, includes per-repo
`contract_identity`, `declared_execution`, `resolved`, `error`, and `next` fields when present,
adds additive per-repo `workflow` and `task` when workflow planning is selected, and adds
`"mode": "execution-plan"`.

`ota workspace receipt --json` uses the same scan as `status`, but packages the result as a
receipt artifact. It records the same readiness and drift detail, adds `"mode": "receipt"`, and
keeps the receipt object available for CI or archive consumers. When `--archive` is set,
the output also includes `archive_path` pointing at the persisted receipt JSON.

When `ota workspace receipt --json --progress-json` is used, Ota also emits live workspace
progress events as compact one-line JSON on `stderr`. Those progress events use the same
`workspace_progress` event shape as `ota workspace doctor|check|status|run|up|refresh --json --progress-json`,
and because receipt reuses the same scan as `workspace status`, `tail` carries the repo drift
state such as `MATCH`, `DIRTY`, or `UNRESOLVED`, while the final workspace-receipt report remains
the single JSON document on `stdout`.

`summary` mirrors the top-level execution receipt summary and lets hosted consumers read the roll-up
without opening `receipt` first.

`receipt.contract_identity` uses the same compact identity block as repo execution receipts, but
identifies the workspace contract with `project.type = "workspace"` and workspace-level `repos` /
`policies` counts.

When an execution receipt includes `next`, additive `receipt.next_steps` carries the same
follow-up lane as an ordered string array so agents and CI do not need to split the human review
string themselves.
When a receipt is tied to a selected task path, additive `receipt.dependency_steps[]` records the
executed dependency-plane truth for each task step using the same structured shape as preview
`plan.dependency_steps[]`, so archived receipts preserve which backend-selection lane actually won
during execution instead of only the flattened task order.

Optional per-repo fields:

- `service`
- `task`
- `exit_code`
- `stdout`
- `stderr`
- `env_sources`
- `ready`
- `readiness_status`
- `drift_status`
- `branch`
- `head`
- `target_ref`
- `ahead`
- `behind`
- `dirty`
- `mode` (`preview` for `ota workspace refresh --dry-run`, `diff` for `ota workspace diff --json`, `status` for `ota workspace status --json`, `receipt` for `ota workspace receipt --json`, `execution-plan` for `ota workspace execution plan --json`)

Example acquisition/setup failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "repos": [
    {
      "name": "web",
      "required": true,
      "ok": false,
      "status": "ACQUIRE FAILED",
      "phase": "acquisition",
      "findings": [
        {
          "severity": "error",
          "summary": "Repo acquisition failed: web",
          "why": "...",
          "next": "..."
        }
      ],
      "exit_code": 128,
      "stderr": "fatal: ..."
    }
  ]
}
```

Example with inferred Docker Compose services:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "written": false,
  "config": {
    "version": 1,
    "project": {
      "name": "docker-legacy"
    },
    "services": {
      "db": {
        "manager": {
          "kind": "compose",
          "name": "docker-legacy",
          "file": "docker-compose.yml",
          "service": "db"
        },
        "readiness": {
          "kind": "compose_health"
        }
      }
    }
  },
  "inferred": [
    {
      "field": "services.db.manager.kind",
      "type": "service",
      "value": "compose",
      "source": "docker-compose.yml#services.db",
      "signal": "config",
      "confidence": "high"
    },
    {
      "field": "services.db.manager.name",
      "type": "service",
      "value": "docker-legacy",
      "source": "directory-name",
      "signal": "convention",
      "confidence": "high"
    },
    {
      "field": "services.db.readiness.kind",
      "type": "service",
      "value": "compose_health",
      "source": "docker-compose.yml#services.db.healthcheck.test",
      "signal": "config",
      "confidence": "high"
    }
  ]
}
```

Example with inferred Docker Compose host topology from one deterministic host-published TCP port candidate:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "written": false,
  "config": {
    "version": 1,
    "project": {
      "name": "ota-containerized-web"
    },
    "execution": {
      "default_context": "host",
      "contexts": {
        "host": {
          "backend": "native"
        }
      }
    },
    "services": {
      "web": {
        "manager": {
          "kind": "compose",
          "name": "docker-heavy-node",
          "file": "docker-compose.yml",
          "service": "web"
        },
        "endpoints": {
          "host": {
            "address": "127.0.0.1",
            "port": 3000
          }
        },
        "readiness": {
          "from": "host",
          "kind": "tcp"
        }
      }
    }
  },
  "inferred": [
    {
      "field": "execution.default_context",
      "type": "execution",
      "value": "host",
      "source": "docker-compose.yml#services.web.ports[0]",
      "signal": "config",
      "confidence": "high"
    },
    {
      "field": "execution.contexts.host.backend",
      "type": "execution",
      "value": "native",
      "source": "docker-compose.yml#services.web.ports[0]",
      "signal": "config",
      "confidence": "high"
    },
    {
      "field": "services.web.readiness.kind",
      "type": "service",
      "value": "tcp",
      "source": "docker-compose.yml#services.web.ports[0]",
      "signal": "config",
      "confidence": "high"
    }
  ]
}
```

When a Compose service exposes multiple deterministic host-published TCP port candidates, Ota now
keeps the inferred host execution slice and emits named endpoints such as
`services.<name>.endpoints.host_3000.context: host`, while intentionally omitting
`services.<name>.readiness.endpoint` when no single truthful readiness target can be selected.
