# Ota JSON Output Reference

This document records the current machine-readable output shapes for Ota commands that support `--json`.

The goal is stability for humans, CI, editors, and agents.

Canonical JSON Schema files for the current shipped shapes live in:

- [json-schemas/validate.json](json-schemas/validate.json)
- [json-schemas/tasks.json](json-schemas/tasks.json)
- [json-schemas/doctor.json](json-schemas/doctor.json)
- [json-schemas/check.json](json-schemas/check.json)
- [json-schemas/init.json](json-schemas/init.json)
- [json-schemas/up.json](json-schemas/up.json)
- [json-schemas/detect.json](json-schemas/detect.json)
- [json-schemas/workspace-doctor.json](json-schemas/workspace-doctor.json)
- [json-schemas/workspace-up.json](json-schemas/workspace-up.json)

## General notes

- success output is printed to stdout
- command failures may still use stderr when the command cannot produce its normal JSON result
- some JSON failures include an optional `next` string when Ota can point to one safe follow-up command
- `ok: true` does not always mean zero findings; warning-only diagnosis can still be `ok: true`
- `path` always refers to the resolved contract path

## `ota validate --json`

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml"
}
```

Failure shape can also include:

- `next`: optional safe follow-up command, used for trust-sensitive refusal and review-first flows

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "errors": ["..."]
}
```

Or:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
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
    "writable_paths": ["src", "docs"]
  },
  "tasks": [
    {
      "name": "setup",
      "kind": "script",
      "script": "printf ready > prepared.txt\n",
      "depends_on": [],
      "safe_for_agent": false
    }
  ]
}
```

## `ota doctor --json`

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
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

## `ota workspace validate --json`

`ota workspace validate --json` uses the same success/failure shape as `ota validate --json`,
but `path` refers to the resolved `ota.workspace.yaml`.

Success:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml"
}
```

Failure:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "errors": ["..."]
}
```

## `ota workspace doctor --json`

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
          "summary": "Repo not acquired: web",
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
  "path": "/abs/path/to/ota.yaml",
  "written": false,
  "error": "`/abs/path/to/ota.yaml` already exists; `ota init` is only for repos without an Ota contract\nNext: review the existing contract with `ota validate /abs/path/to/ota.yaml` or `ota doctor /abs/path/to/ota.yaml`\nNext: if you want to compare detected repo signals against it, run `ota detect --merge --dry-run /abs/path/to/ota.yaml`",
  "next": "ota detect --merge --dry-run /abs/path/to/ota.yaml"
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

## `ota up --json`

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
  "path": "/abs/path/to/ota.yaml",
  "written": false,
  "error": "`/abs/path/to/ota.yaml` already exists; refusing to overwrite an existing contract\nNext: review detected changes with `ota detect --merge --dry-run /abs/path/to/repo`",
  "next": "ota detect --merge --dry-run /abs/path/to/repo"
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
