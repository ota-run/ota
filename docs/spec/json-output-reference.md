# Ota JSON Output Reference

This document records the current machine-readable output shapes for Ota commands that support `--json`.

`docs/spec` is the canonical source of truth. This page is part of that spec
corpus and the public reference pages are derived from it with examples and
operator guidance added where useful.

The goal is stability for humans, CI, editors, and agents.

Editor and CI integrations should treat the JSON surfaces in this document as the stable contract
and avoid scraping human-readable text output.

Canonical JSON Schema files for the current shipped shapes live in:

- [json-schemas/validate.json](json-schemas/validate.json)
- [json-schemas/tasks.json](json-schemas/tasks.json)
- [json-schemas/agents.json](json-schemas/agents.json)
- [json-schemas/doctor.json](json-schemas/doctor.json)
- [json-schemas/check.json](json-schemas/check.json)
- [json-schemas/init.json](json-schemas/init.json)
- [json-schemas/up.json](json-schemas/up.json)
- [json-schemas/detect.json](json-schemas/detect.json)
- [json-schemas/workspace-init.json](json-schemas/workspace-init.json)
- [json-schemas/workspace-tasks.json](json-schemas/workspace-tasks.json)
- [json-schemas/workspace-run.json](json-schemas/workspace-run.json)
- [json-schemas/workspace-check.json](json-schemas/workspace-check.json)
- [json-schemas/workspace-doctor.json](json-schemas/workspace-doctor.json)
- [json-schemas/workspace-explain.json](json-schemas/workspace-explain.json)
- [json-schemas/workspace-up.json](json-schemas/workspace-up.json)
- [json-schemas/diff.json](json-schemas/diff.json)
- [json-schemas/explain.json](json-schemas/explain.json)

## General notes

- success output is printed to stdout
- command failures may still use stderr when the command cannot produce its normal JSON result
- some JSON failures include an optional `next` string when Ota can point to one safe follow-up command
- `ok: true` does not always mean zero findings; warning-only diagnosis can still be `ok: true`
- `path` refers to the resolved contract path as rendered by current CLI path compaction (often cwd-relative such as `./ota.yaml`)

## Which JSON surface to use

- use `ota validate --json` or `ota workspace validate --json` for contract gating
- use `ota agents --json` when you want a repo-local `AGENTS.md` export preview or sync report
- use `ota doctor --json` or `ota workspace doctor --json` for readiness diagnosis and blocking findings
- use `ota workspace explain --json` when you want an ordered workspace remediation plan
- use `ota workspace tasks --json` when you want workspace inventory and task availability
- use `ota workspace list --json` when you want lightweight workspace inventory and readiness
- use `ota workspace check --json` when you want checks-only workspace readiness with a roll-up summary
- use `ota up --json` or `ota workspace up --json` when you want preparation or readiness roll-up data
- use `ota workspace run --json` when you want coordinated multi-repo execution roll-up data and receipts
- use `ota workspace receipt --json` when you want a read-only workspace receipt artifact
- use `ota diff --json` or `ota explain --json` when you want contract change impact or remediation planning

## Editor and IDE contract rules

Editor and IDE consumers should prefer the smallest stable fields for the job instead of parsing
human text output:

- `ota validate --json` and `ota workspace validate --json`: use `ok`, `summary.error_count`, `errors` or `error`, and `next`
- `ota agents --json`: use `ok`, `path`, `output`, `written`, `mode`, and `content`
- `ota doctor --json` and `ota workspace doctor --json`: use the top-level `summary`, per-repo `findings`, and `execution`
- `ota workspace explain --json`: use the top-level `summary`, per-repo `findings`, and per-repo `steps` with stable codes
- `ota workspace tasks --json`: use the top-level `summary`, per-repo `tasks`, and dependency order
- `ota workspace list --json`: use the top-level `summary`, per-repo readiness, and contract presence
- `ota workspace check --json`: use the top-level `summary` and per-repo findings
- `ota up --json` and `ota workspace up --json`: use the top-level `summary`, `receipt`, and per-repo results
- `ota workspace run --json`: use the top-level `summary`, `receipt`, and per-repo results
- `ota workspace receipt --json`: use the top-level `summary`, `receipt`, and per-repo results
- `ota diff --json`: use the readiness-impact summary and changes
- `ota explain --json`: use the remediation steps and stable step codes

Hosted CI can use the same fields as annotations or check-run summaries:

- `summary.primary_blocker` when present, for the headline
- `findings[]` or per-repo `findings[]` as the annotation stream
- `severity` to decide blocking versus warning annotations
- `why` for the annotation body
- `next` for the suggested fix or link target

## `ota validate --json`

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "error_count": 0
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
    "error_count": 2
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
    "error_count": 1
  },
  "error": "..."
}
```

## `ota tasks --json`

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "agent": {
    "entrypoint": "setup",
    "safe_tasks": ["setup", "test"],
    "verify_after_changes": ["test"],
    "writable_paths": ["src", "docs"],
    "protected_paths": ["Cargo.lock", "LICENSE"]
  },
  "tasks": [
    {
      "name": "setup",
      "description": "Prepare the repo",
      "notes": "Use this after cloning the repo.\n",
      "kind": "script",
      "script": "printf ready > prepared.txt\n",
      "env": {
        "JAVA_HOME": "/opt/jdk-21"
      },
      "inputs": {
        "base_url": {
          "required": true
        }
      },
      "depends_on": [],
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
          "depends_on": [],
          "safe_for_agent": false
        }
      ]
    }
  ]
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
  "agent": {
    "entrypoint": "setup",
    "verify_after_changes": ["test"]
  },
  "provisioning": {
    "allowed": [
      {
        "kind": "runtime",
        "name": "java",
        "requested_version": "22",
        "source": "org-mirror",
        "approved_version": "22",
        "blocked_reason": null
      },
      {
        "kind": "tool",
        "name": "maven",
        "requested_version": "3.9",
        "source": "approved-manager",
        "approved_version": "3.9",
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
        "source": "org-mirror",
        "approved_version": "22"
      },
      {
        "kind": "select_source",
        "target_kind": "tool",
        "name": "maven",
        "requested_version": "3.9",
        "source": "approved-manager",
        "approved_version": "3.9"
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

When the repo declares runtimes or tools and policy provides approved sources for them,
`ota doctor --json` may also include a top-level `provisioning` object. That object is a read-only
plan with `provisionable` entries for targets policy approves and `blocked` entries for declared
targets that policy does not currently approve. It exists so humans and agents can see what would
be provisionable later without mutating the machine today.

`ota workspace doctor --json` uses the same finding shape for per-repo findings, so the same
additive policy keys may appear there as well. When a repo declares execution metadata, the shared
`execution.env` array may include policy provenance with `source` values such as `repo policy`
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

`ota doctor --json` also includes a top-level `summary` object with finding counts and
machine-readable `verdict` / `agent_verdict` values so hosted validation and editor tooling do
not need to recompute them. When there is at least one finding, the summary may also include
`primary_blocker` with the highest-priority blocker details so CI and editors can answer the
question “what should I fix first?” without scanning the full list.

When the repo signals no longer match the declared contract, `ota doctor --json` may include
warning findings that describe the drift and point back to `ota detect --merge --dry-run` for the
comparison preview. Drift findings also include optional `ownership` and `provenance` fields so
CI and editors can classify the mismatch as a repo-contract issue and trace the source of the
comparison.

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

`ota workspace doctor --json` may include the same `execution` object on each repo item when the
underlying repo contract declares execution metadata, including env provenance for inherited
workspace policy values.

`ota workspace doctor --json` may also include the same `extensions` object on each repo item when
the underlying repo contract declares it. The descriptor shape matches `ota doctor --json`.

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

Explain steps may also include `provenance` when the underlying finding carries policy or drift context.

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
  "steps": [
    {
      "order": 1,
      "code": "OTA_TASKS_MISSING",
      "severity": "error",
      "summary": "No tasks defined in contract",
      "why": "...",
      "next": "...",
      "provenance": "org policy"
    }
  ]
}
```

## `ota workspace explain --json`

Workspace explain steps may also include `provenance` when the underlying finding carries policy or drift context.

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
      "steps": [
        {
          "order": 1,
          "code": "OTA_TASKS_MISSING",
          "severity": "error",
          "summary": "No tasks defined in contract",
          "why": "...",
          "next": "...",
          "provenance": "org policy"
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
      "required": true,
      "acquired": true,
      "depends_on": ["db"],
      "tasks": [
        {
          "name": "setup",
          "kind": "run",
          "run": "pnpm install",
          "depends_on": []
        }
      ]
    }
  ]
}
```

Non-acquired repos keep `acquired: false` and `tasks: []`.

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
      "required": true,
      "ok": true,
      "status": "READY",
      "task": "setup",
      "findings": []
    }
  ]
}
```

`receipt` mirrors the workspace execution roll-up and keeps backend-aware execution metadata on the
same surface as the repo-level execution commands.

Optional per-repo fields:

- `exit_code`
- `stdout`
- `stderr`
- `env_sources`

## `ota workspace check --json`

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
counts from checks-only output.

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
      "value": "ota-app",
      "source": "package.json#name",
      "confidence": "high"
    }
  ]
}
```

Failure example:

```json
{
  "ok": false,
  "path": "./ota.yaml",
  "written": false,
  "error": "`./ota.yaml` already exists; ota init is only for repos without an Ota contract\n\nNext:\n▸  review the existing contract with `ota validate`\n▸  review the existing contract with `ota doctor`\n▸  compare detected repo signals with `ota detect --merge --dry-run`\n▸  apply detected add-only high-confidence fields now with `ota detect --merge`",
  "next": "ota detect --merge --dry-run"
}
```

## `ota check --json`

`ota check --json` uses the same finding shape as `ota doctor --json`, but does not include the
optional `agent` summary:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
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

- execution reached the `up` pipeline: returns `UpStatus` (`status`, `phase`, `findings`, optional `service`/`task`/`exit_code`)
- contract load/validation failed before the `up` pipeline: returns `ValidateFailure` shape (`ok`, `path`, and either `errors` or `error`)

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "status": "READY",
  "phase": "post-setup diagnosis",
  "findings": []
}
```

Optional fields:

- `service`: present when a required service start command fails
- `task`: present when a task failure is reported
- `exit_code`: present when a child command failure is reported
- `members`: present on monorepo-root aggregate output with grouped member readiness results

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

## `ota detect --json`

`ota detect --merge --json --dry-run` currently uses the same success shape as `ota detect --json
--dry-run`, but requires an existing contract and includes `comparison`.

`ota detect --merge --json` uses the same success shape with:

- `written: true` when additive high-confidence fields were applied
- `written: false` when there was nothing eligible to add
- `comparison` describing detected adds and updates against the existing contract
- `comparison.removals` describing stale contract fields that are no longer detected in the repo
- `comparison` may include lower-confidence add candidates that remain preview-only

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
        "detected": "ota-web"
      }
    ],
    "removals": [
      {
        "field": "tools.cargo",
        "existing": "1.78"
      },
      {
        "field": "tasks.build.run",
        "existing": "cargo build"
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
entries while `written` remains `false`. That means Ota found possible additions, but none were
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
      "phase": "post-setup diagnosis",
      "findings": []
    }
  ]
}
```

`ota workspace refresh --json` uses the same workspace roll-up shape, but reports refresh
status for existing repos instead of bootstrap status for missing ones. In preview mode it
adds `"mode": "preview"` and does not mutate repo state.

`ota workspace diff --json` uses a read-only workspace diff roll-up. It reports local git
state against the declared source ref or upstream branch, includes per-repo `status`,
`branch`, `head`, `target_ref`, `ahead`, `behind`, and `dirty` fields, and adds
`"mode": "diff"`.

`ota workspace status --json` uses the operational workspace roll-up. It reports readiness and
local git drift together, includes per-repo `ready`, `readiness_status`, `drift_status`,
`branch`, `head`, `target_ref`, `ahead`, `behind`, and `dirty` fields, and adds
`"mode": "status"`.

`ota workspace receipt --json` uses the same scan as `status`, but packages the result as a
receipt artifact. It records the same readiness and drift detail, adds `"mode": "receipt"`, and
keeps the receipt object available for CI or archive consumers.

`summary` mirrors the top-level execution receipt summary and lets hosted consumers read the roll-up
without opening `receipt` first.

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
- `mode` (`preview` for `ota workspace refresh --dry-run`, `diff` for `ota workspace diff --json`, `status` for `ota workspace status --json`, `receipt` for `ota workspace receipt --json`)

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
      "value": "docker-compose",
      "source": "docker-compose.yml#services.db",
      "confidence": "high"
    },
    {
      "field": "services.db.start",
      "value": "docker compose up -d db",
      "source": "docker-compose.yml#services.db",
      "confidence": "medium"
    },
    {
      "field": "services.db.healthcheck",
      "value": "pg_isready -h localhost -p 5432",
      "source": "docker-compose.yml#services.db.healthcheck.test",
      "confidence": "medium"
    }
  ]
}
```
