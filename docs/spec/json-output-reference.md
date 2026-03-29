# Ota JSON Output Reference

This document records the current machine-readable output shapes for Ota commands that support `--json`.

The goal is stability for humans, CI, editors, and agents.

Editor and CI integrations should treat the JSON surfaces in this document as the stable contract
and avoid scraping human-readable text output.

Canonical JSON Schema files for the current shipped shapes live in:

- [json-schemas/validate.json](json-schemas/validate.json)
- [json-schemas/tasks.json](json-schemas/tasks.json)
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
- [json-schemas/workspace-up.json](json-schemas/workspace-up.json)

## General notes

- success output is printed to stdout
- command failures may still use stderr when the command cannot produce its normal JSON result
- some JSON failures include an optional `next` string when Ota can point to one safe follow-up command
- `ok: true` does not always mean zero findings; warning-only diagnosis can still be `ok: true`
- `path` refers to the resolved contract path as rendered by current CLI path compaction (often cwd-relative such as `./ota.yaml`)

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
  "findings": [
    {
      "severity": "warn",
      "summary": "...",
      "why": "...",
      "next": "..."
    }
  ]
}
```

Finding objects may include additive policy context keys when policy-aware diagnosis is surfaced:
`policy_outcome`, `policy_reason`, `policy_source`, `install_scope`, and `mutation_allowed`.
These keys are optional and backward-compatible.

`ota workspace doctor --json` uses the same finding shape for per-repo findings, so the same
additive policy keys may appear there as well.

`ota doctor --json` may also include an `execution` object when the contract declares execution
metadata that editors and remote-runner tooling can consume.

`ota doctor --json` also includes a top-level `summary` object with finding counts so hosted
validation and editor tooling do not need to recompute them.

`ota doctor --json` may also include an `extensions` object when the contract declares top-level
extension data. Each entry is a typed adapter descriptor with `kind`, `command`, and
`api_version`, plus optional `description` and `config`. Supported kinds today are `checker` and
`publisher`. The field is parsed and preserved for discovery, and `ota extensions --run <name>`
can execute one explicitly named `checker` descriptor with `api_version: 1`; `ota extensions
--publish <name>` can execute one explicitly named `publisher` descriptor with `api_version: 1`.

`ota workspace doctor --json` may include the same `execution` object on each repo item when the
underlying repo contract declares execution metadata.

`ota workspace doctor --json` may also include the same `extensions` object on each repo item when
the underlying repo contract declares it. The descriptor shape matches `ota doctor --json`.

`ota workspace doctor --json` also includes a top-level `summary` object with repo and finding
counts for hosted validation and editor consumers.

`ota workspace list --json` also includes a top-level `summary` object with repo inventory counts
for editor, CI, and hosted preflight tooling.

Root monorepo summary output can also include grouped member findings under `members`.

Doctor JSON findings also include remote target-shape warnings when relevant, such as suspicious
`ssh`/`tsh` targets without `user@host` or `kubectl` targets that do not start with `pod/`.

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

Optional per-repo fields:

- `exit_code`
- `stdout`
- `stderr`

## `ota workspace check --json`

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
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

Optional per-repo fields:

- `service`
- `task`
- `exit_code`
- `stdout`
- `stderr`

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
