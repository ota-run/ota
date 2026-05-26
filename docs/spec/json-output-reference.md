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
- `ota services --json`: use `services`, grouped `members` when present, and nested `manager`, `readiness`, `endpoints`, and `depends_on`
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
- `ota clean --json` and `ota clean --stale --json`: use cleanup counters and `queried_engines` on success; on classified cleanup failures use `summary`, `reason`, `engine`, `resource_kind`, `resource_name`, `details`, and ordered `next` steps, while generic repo-state failures fall back to `summary` plus `error`
- `ota up --json` and `ota workspace up --json`: use the top-level `summary`, `receipt`, and per-repo results; workspace repo results may also include additive `next` / `next_steps`
- `ota workspace run --json`: use the top-level `summary`, `receipt`, and per-repo results; repo results may also include additive `next` / `next_steps`
- `ota workspace receipt --json`: use the top-level `summary`, `receipt`, and per-repo results
- `ota diff --json`: use the readiness-impact summary and changes
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
  "semver": "1.6.16",
  "version": "v1.6.16",
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
      "id": "tasks.action.ensure_bundle",
      "introduced_in": "1.6.17"
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
  }
}
```

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
task when the workflow does not declare a run phase. `workflow.prepare_task` is additive path
context only; it does not replace the concrete execution `task` because host file prep is not the
selected runtime identity. The workflow object may also include additive `notes` and
`readiness_probes` when the selected workflow declares notes or references reusable named probes.

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
- `tasks[*].launch` is present when one task uses structured `launch` instead of shell `run` or
  `script`; it exposes the launch kind plus the structured command or packaged container metadata
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

## `ota proof runtime --json`

Runtime proof stays intentionally thin. It proves one selected runtime path and captures the real
declared artifacts that remain canonical:

- `topology.json` from `ota execution topology --json`
- `doctor.json` from `ota doctor --json`
- `up.log` from the repo-level runtime-preparation lane

Notes:

- `mode` is always `runtime-proof`
- `workflow` is present when the proof targeted one explicit or effective workflow
- `phase` stays machine-stable and uses the underlying proof phase keys such as `preconditions`,
  `provisioning`, `activation`, `setup`, `run`, `services`, `post-up diagnosis`, and `cleanup`
- `summary` reuses the doctor verdict/count shape instead of inventing a second readiness dialect
- `artifacts` points at the captured canonical payloads; machine consumers should inspect those
  files directly when they need the full topology or doctor surface
- cleanup failures are reported through top-level `error` and `next` without duplicating the doctor
  findings stream

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "mode": "runtime-proof",
  "workflow": "app",
  "phase": "post-up diagnosis",
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
  }
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
  }
}
```

## `ota services --json`

Services inspection stays read-only. It reports the declared managed-service inventory without
starting or probing those services.

Notes:

- top-level `services` lists declared repo services in deterministic order
- `members` is present when a monorepo root request includes grouped member service summaries
- each service can expose `producer`, `manager`, legacy `provider`, `start`, `stop`,
  `healthcheck`, structured `readiness`, projected `endpoints`, `depends_on`, and `timeout`

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
      "healthcheck": "pg_isready -h 127.0.0.1 -p 5432",
      "endpoints": {
        "host": {
          "address": "127.0.0.1",
          "port": 5432
        }
      },
      "depends_on": [],
      "timeout": 30
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
    "task": "dev"
  },
  "inputs": {
    "style": "spring-http"
  },
  "assumptions": [
    "task `dev` already declares a service runtime",
    "listener `http` is the readiness surface"
  ],
  "changes": [
    {
      "path": "tasks.dev.runtime.readiness",
      "action": "set",
      "before": null,
      "after": {
        "kind": "http",
        "listener": "http",
        "method": "GET",
        "path": "/actuator/health"
      }
    }
  ],
  "diff": "tasks.dev.runtime.readiness\n- <absent>\n+ kind: http ...",
  "validation": [
    "ota validate /abs/path/to/ota.yaml",
    "ota doctor /abs/path/to/ota.yaml"
  ],
  "next": "rerun with `ota assist declare-readiness --task dev --style spring-http --write /abs/path/to/ota.yaml` to apply this readiness change"
}
```

Notes:

- `path` is the resolved repo contract path
- `member` is present only when `--member` targeted a merged monorepo member contract
- `subject` contains exactly one selector key today: `task` or `service`
- `changes[*].before` is present when assist is refining or replacing an existing readiness block, including legacy top-level readiness shapes
- `changes[*].after` uses the canonical readiness contract shape Ota would write
- `mode` is `preview` by default and `write` when `--write` succeeded

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
    "endpoint": "host",
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
          "host": {
            "address": "127.0.0.1",
            "port": 5432
          }
        },
        "readiness": {
          "from": "host",
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
  "next": "rerun with `ota assist declare-service --name postgres --manager compose --endpoint host --address 127.0.0.1 --port 5432 --required false --compose-file docker-compose.yml --compose-service postgres --style tcp --write /abs/path/to/ota.yaml` to apply this service change"
}
```

Notes:

- `subject.service` is the targeted top-level managed service name
- `inputs` records the explicit or defaulted service declaration inputs used to build the proposal
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
        "sh": "curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION=v1.6.16 sh",
        "powershell": "$env:OTA_VERSION='v1.6.16'; irm https://dist.ota.run/install.ps1 | iex"
      }
    }
  },
  "tasks": [
    {
      "name": "setup",
      "description": "Prepare the repo",
      "notes": "Use this after cloning the repo.\n",
      "kind": "copy_if_missing",
      "action": {
        "kind": "copy_if_missing",
        "from": ".env.example",
        "to": ".env.local"
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
        "writes": [".env.local"]
      },
      "depends_on": [],
      "requires_services": ["postgres"],
      "safe_for_agent": false
    }
  ]
}
```

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
          "safe_for_agent": false
        }
      ]
    }
  ]
}
```

Each task may also include additive `effects` when the contract declares durable writes
(`writes`), a connectivity dependency (`network`), an optional network lane classification
(`network_kind`), or out-of-repo mutation (`external_state`).
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
  "workflows": [
    {
      "name": "quickstart",
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
- each workflow entry includes additive fields only when declared or resolved:
  - `intent`
  - `description`
  - `notes`
  - `prepare_task`
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
        "sh": "curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION=v1.6.16 sh",
        "powershell": "$env:OTA_VERSION='v1.6.16'; irm https://dist.ota.run/install.ps1 | iex"
      }
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
      }
    }
  ]
}
```

Finding objects always include stable identity fields:
`code`, `category`, `owner`, and `evidence`.

Finding objects may also include additive policy context keys when policy-aware diagnosis is surfaced:
`policy_outcome`, `policy_reason`, `policy_source`, `install_scope`, and `mutation_allowed`.
These keys are optional and backward-compatible.
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
question “what should I fix first?” without scanning the full list.

When the repo contract declares an `agent` block, the additive `agent` summary can also include
`inferred_boundary_reviewed`. `false` means the current writable and protected boundary still comes
from starter or detector inference and has not been confirmed by the repo author yet.

`ota doctor --json` also includes a top-level `mode` string. It is `native` for host readiness
diagnosis and `container` when the report was produced with `ota doctor --mode container`, so
consumers can tell which execution context the findings describe without inferring it from the CLI
invocation.

When the repo signals no longer match the declared contract, `ota doctor --json` may include
warning findings that describe the drift and point back to `ota detect --merge --dry-run` for the
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

The published schema for this surface is
[json-schemas/run-preview.json](json-schemas/run-preview.json). It covers single-target ready or
blocked previews, aggregate member previews, and the simpler pre-preview error envelope.

Repo-level `ota run --json` is currently not a mutating execution receipt surface. Use
`ota run <task> --dry-run --json` for planning, `ota receipt --json` for repo readiness receipts,
and `ota workspace run --json` for coordinated multi-repo execution receipts.

`preview_status` is the operator-facing preview label for this dry-run surface. It is
`RUNNABLE`, `RUNNABLE WITH WARNINGS`, or `BLOCKED`. Keep using `summary.verdict` for the canonical
shared readiness verdict.

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "contract": "/path/to/ota.yaml",
  "task": "ci",
  "dry_run": true,
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
    "safe_for_agent": false
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
  "plan": {
    "dependency_chain": ["ci"],
    "actions": ["would execute `npm test` on the host"]
  }
}
```

Use this when a human or agent needs the selected run plan before execution:

- `resolved` is the selected backend/lifecycle/image/provider plan
- `requested_task` is the selected task body after contract validation
- `requested_context` is the task-declared context (when present)
- `selected_context` is the resolved execution context ota will apply for this preview
- `env_summary`, `sources`, and `env` show the selected env state and blockers
- `toolchains[]` keeps toolchain-owned capabilities on the toolchain instead of duplicating them as
  standalone runtime/tool evidence
- `plan.dependency_chain`, `plan.requirement_lines`, `plan.actions`, and `plan.notes` show what
  ota would check, activate, provision, or run
- exit `0` means the preview is actionable; exit `1` means the preview is blocked by contract,
  env, or execution-plan problems
- blocked previews still use the full preview envelope on stdout so automation can read
  `summary.primary_blocker` without scraping stderr

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
finding (`severity`, `summary`, `why`, `next`, and optional provenance fields).

`ota workspace doctor --json` also includes a top-level `summary` object with repo and finding
counts for hosted validation and editor consumers. The workspace summary also carries
`verdict` / `agent_verdict` values. When there is at least one finding, the summary may also
include `primary_blocker` with the highest-priority blocker details and the repo name that owns it.

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
`provenance_key` when ota can trace the diagnosis source for the underlying finding.

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
        "next": "..."
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
first-class setup action such as `copy_if_missing`.
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
        "summary": "Check failed: health-check",
        "why": "...",
        "next": "..."
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
- `signal` is one of `config`, `script`, `lockfile`, `file`, `template`, or `convention`
- task-shaped entries can also include `agent_safe` (`yes`, `no`, `unknown`) and `agent_signal` (`verification_candidate` or `bootstrap_candidate`) when ota can classify the task for agent workflows

In dry-run preview mode, `config` matches the starter contract ota would review or write,
including derived starter defaults such as a minimal `agent` block when ota can infer one
safely.

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
example, Python repos with `uv.lock` now converge directly on `toolchains.python` with
`provider: uv`, so they no longer emit fallback Python opportunity guidance.

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
  "error": "`./ota.yaml` already exists; ota init is only for repos without an ota contract\n\nNext:\n▸  review the existing contract with `ota validate`\n▸  review the existing contract with `ota doctor`\n▸  compare detected repo signals with `ota detect --merge --dry-run`\n▸  apply detected add-only high-confidence fields now with `ota detect --merge`",
  "next": "ota detect --merge --dry-run"
}
```

`ota init --packs --json` lists the available built-in starter packs without previewing one
contract:

```json
{
  "ok": true,
  "mode": "catalog",
  "packs": [
    {
      "name": "node",
      "summary": "Conventional Node starter with toolchain-owned Node and package-manager-driven setup, dev, and test tasks.",
      "when": "Use this for repo-level Node apps or services that need an explicit JavaScript starter instead of detector-led init. The default path keeps Node ownership under `toolchains.node` and uses pnpm via Corepack, and you can override the package manager with `--package-manager` when the repo is intentionally npm-, yarn-, or bun-based.",
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
        "checks": ["node-installed"],
        "tasks": ["setup", "dev", "test"]
      }
    },
    {
      "name": "go",
      "summary": "Conventional Go starter with module download, build, and test tasks.",
      "when": "Use this for Go module repos that should start from the standard `go mod download`, `go build`, and `go test` flow without relying on detector-led init.",
      "command": "ota init --pack go",
      "next": "ota init --pack go --dry-run .",
      "does_not_infer": [
        "workspace layout, code generation, or custom build flags beyond the standard module download/build/test loop"
      ],
      "seeds": {
        "runtimes": ["go"],
        "tools": [],
        "checks": ["go-installed"],
        "tasks": ["setup", "build", "test"]
      }
    },
    {
      "name": "dotnet",
      "summary": "Conventional .NET starter with restore, build, and test tasks.",
      "when": "Use this for .NET repos that should start from the standard `dotnet restore`, `dotnet build`, and `dotnet test` loop without relying on detector-led init.",
      "command": "ota init --pack dotnet",
      "next": "ota init --pack dotnet --dry-run .",
      "does_not_infer": [
        "solution-specific target selection, test filtering, or custom dotnet CLI flags beyond the standard restore/build/test loop"
      ],
      "seeds": {
        "runtimes": ["dotnet"],
        "tools": ["dotnet"],
        "checks": ["dotnet-installed"],
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
      "summary": "Conventional Java starter for Maven-driven repos with build and test lifecycles, preferring `mvnw` when the repo already ships it.",
      "when": "Use this when the repo is intentionally Maven-based and you want an explicit Java starter without relying on repo detection. If `mvnw` already exists, ota uses the wrapper instead of requiring a global Maven install.",
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

- execution reached the `up` pipeline: returns `UpStatus` (`status`, `phase`, `findings`, `receipt`, optional `service`/`task`/`exit_code`)
- contract load/validation failed before the `up` pipeline: returns `ValidateFailure` shape (`ok`, `path`, and either `errors` or `error`)

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "status": "READY",
  "phase": "post-up diagnosis",
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

- `receipt`: execution receipt for the executed repo `up` phase, including additive `receipt.contract_identity`; monorepo aggregate output keeps grouped `members` results instead of a top-level receipt
- `receipt.native_prerequisites`: additive selected native prerequisite detail for the executed native task/setup path, including provisioning guidance and any applied native activation
- `service`: present when a required service start command fails
- `task`: present when a task failure is reported
- `exit_code`: present when a child command failure is reported
- `members`: present on monorepo-root aggregate output with grouped member readiness results

Phase values:

- `preconditions`: prerequisite diagnosis blocked before provisioning or setup
- `provisioning`: policy-backed provisioning failed before setup
- `activation`: selected prerequisite activation (for example Corepack-managed tool activation)
  failed before setup
- `services`: required service start or readiness failed
- `setup`: setup task execution failed
- `run`: selected workflow run task execution failed
- `post-up diagnosis`: setup and service orchestration completed, but final readiness was still not
  satisfied

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
load/validation failures still emit the shared `ValidateFailure` JSON shape on stdout.

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
    "backend": "native",
    "steps": [
      {
        "order": 1,
        "label": "readiness",
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
- `summary`
- `receipt`
- `findings`

The nested `receipt` object can also include:

- `contract_identity` with the declared project, selected metadata, execution intent, and compact contract counts
- `backend`
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

`ok` mirrors the current repo receipt readiness result, so blocked repo receipts still return the
receipt success shape with `ok: false`.

When `--archive --promote-baseline` is set, the receipt success shape also includes:

- `promoted_baseline.path`
- `promoted_baseline.archive_path`
- `promoted_baseline.promoted_at`

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
      "readiness_change": "unchanged"
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
- `baseline.contract_identity_details` with the compact declared contract identity when the archived receipt recorded it
- `baseline.ok`
- `baseline.contract`
- `baseline.backend` / `baseline.lifecycle` when recorded
- `baseline.summary`
- `current.ok`
- `current.contract`
- `current.contract_identity` with the current repo-local contract identity
- `current.contract_identity_details` with the compact declared contract identity for the current receipt
- `current.backend` / `current.lifecycle` when recorded
- `current.summary`
- `summary.baseline_ok`
- `summary.current_ok`
- additive `summary.comparison` with baseline/current identity labels plus compact `identity_changed` and `readiness_change` drift signals
- `summary.introduced`
- `summary.resolved`
- `summary.unchanged`
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
      "error": "failed to parse receipt archive `./.ota/receipts/repo-receipt-20260412-090000-000Z.json`: EOF while parsing a value at line 1 column 10"
    }
  ]
}
```

Current receipt history JSON fields:

- `ok`
- `path` (resolved repo boundary for the archive read)
- `mode` (`history`)
- `summary.archive_count`
- `summary.invalid_archive_count`
- `archives[]`
- `archives[].archive_path`
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
- `invalid_archives[].error`

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
    "total_removed": 2,
    "total_skipped_ambiguous": 0
  },
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
        "total_removed": 0,
        "total_skipped_ambiguous": 0
      },
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
            "total_removed": 1,
            "total_skipped_ambiguous": 0
          },
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

`ota detect --merge --json --dry-run` currently uses the same success shape as `ota detect --json
--dry-run`, but requires an existing contract and includes `comparison`.

`ota detect --merge --json` uses the same success shape with:

- `written: true` when additive high-confidence fields were applied
- `written: false` when there was nothing eligible to add
- `config` is the detected candidate contract for preview-only results; when a detect write mode succeeds (`--write`, `--merge`, or `--rewrite` with `written: true`), `config` is the exact contract ota wrote to disk, including `metadata.ota.detect.field_ownership`
- `comparison` describing detected adds and updates against the existing contract
- `comparison.removals` describing stale contract fields that are no longer detected in the repo
- `comparison.changes[*].ownership` is `repo_signals` for add candidates and `repo_contract` for updates against existing fields
- `comparison.removals[*].ownership` is `repo_contract` because those entries describe stale declared contract data
- `comparison.changes[*].owner_kind` is `detected` for add candidates, `manual` for default hand-authored existing fields, and `merged` when ota previously wrote the field and recorded it under `metadata.ota.detect.field_ownership`
- `comparison.removals[*].owner_kind` is `merged` on normal drift surfaces, while rewrite preview can also surface `manual` removals because a full replacement would drop those fields
- `comparison.*.provenance` preserves the stable machine label `repo_signals`
- `comparison.*.provenance_key` is the stable machine label `repo_signals`
- `comparison.changes[*].source` and `comparison.changes[*].confidence` copy the detector evidence for that proposed add or update so consumers do not need to join back to `inferred[*]`
- `comparison` may include lower-confidence add candidates that remain preview-only
- `toolchain_opportunities` appears only when repo signals strongly suggest a managed ecosystem
  that ota still models through lower-level `runtimes` / `tools` declarations because no shipped
  provider contract owns it yet

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

In conservative mixed-repo or legacy-repo cases, `comparison.changes` can still include `add`
entries while `written` remains `false`. That means ota found possible additions, but none were
eligible for automatic merge under the current high-confidence-only rule.

Failure example:

```json
{
  "ok": false,
  "path": "./ota.yaml",
  "written": false,
  "error": "`./ota.yaml` already exists; refusing to overwrite an existing contract\n\nNext:\n▸  review detected changes with `ota detect --merge --dry-run .`",
  "next": "ota detect --merge --dry-run ."
}
```

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

`ota workspace execution plan --json` uses a read-only execution roll-up. It reports one
resolved or unresolved execution decision per selected repo, includes per-repo
`contract_identity`, `declared_execution`, `resolved`, `error`, and `next` fields when present,
adds additive per-repo `workflow` and `task` when workflow planning is selected, and adds
`"mode": "execution-plan"`.

`ota workspace receipt --json` uses the same scan as `status`, but packages the result as a
receipt artifact. It records the same readiness and drift detail, adds `"mode": "receipt"`, and
keeps the receipt object available for CI or archive consumers. When `--archive` is set,
the output also includes `archive_path` pointing at the persisted receipt JSON.

`summary` mirrors the top-level execution receipt summary and lets hosted consumers read the roll-up
without opening `receipt` first.

`receipt.contract_identity` uses the same compact identity block as repo execution receipts, but
identifies the workspace contract with `project.type = "workspace"` and workspace-level `repos` /
`policies` counts.

When an execution receipt includes `next`, additive `receipt.next_steps` carries the same
follow-up lane as an ordered string array so agents and CI do not need to split the human review
string themselves.

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
        "provider": "docker-compose",
        "start": "docker compose up -d db",
        "stop": "docker compose stop db",
        "healthcheck": "pg_isready -h localhost -p 5432"
      }
    }
  },
  "inferred": [
    {
      "field": "services.db.provider",
      "type": "service",
      "value": "docker-compose",
      "source": "docker-compose.yml#services.db",
      "signal": "config",
      "confidence": "high"
    },
    {
      "field": "services.db.start",
      "type": "service",
      "value": "docker compose up -d db",
      "source": "docker-compose.yml#services.db",
      "signal": "config",
      "confidence": "medium"
    },
    {
      "field": "services.db.healthcheck",
      "type": "service",
      "value": "pg_isready -h localhost -p 5432",
      "source": "docker-compose.yml#services.db.healthcheck.test",
      "signal": "config",
      "confidence": "medium"
    }
  ]
}
```
