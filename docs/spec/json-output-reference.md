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

## General notes

- success output is printed to stdout
- command failures may still use stderr when the command cannot produce its normal JSON result
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

## `ota check --json`

`ota check --json` uses the same shape as `ota doctor --json`:

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

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "written": false,
  "config": {
    "version": 1
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
