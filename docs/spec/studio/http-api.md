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
   You may not use this file except in compliance with that License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Ota Studio HTTP API

Status: planned.

This document defines the local-only HTTP contract for interactive Studio.

## Design rules

1. The API is loopback-only.
2. GET is read-only.
3. POST owns mutation and operation launch.
4. The frontend consumes Studio-focused read models, not raw command output where avoidable.
5. The server, not the browser, invokes Ota core actions.

## Transport

Initial binding:

- host: `127.0.0.1`
- port: ephemeral or configured by Ota

The server must:

- bind only to loopback
- reject non-local origins unless explicitly allowed later
- generate a per-session token for mutation endpoints

## Session metadata

Every booted Studio session should expose:

- `session_id`
- `schema_version`
- `server_started_at`
- `mode`

`mode` initial values:

- `interactive_server`

## Endpoint families

Phase 1 endpoints:

- app shell
- session
- repo registry
- pane read models

Later endpoints:

- reviewed apply actions
- operation launch
- operation history
- event stream

## Initial endpoint list

### `GET /`

Returns:

- Studio shell HTML

### `GET /api/session`

Returns:

```json
{
  "session_id": "01JV8M5M63A6H9WJ8B2YH3W3P4",
  "schema_version": 1,
  "mode": "interactive_server",
  "server_started_at": "2026-05-02T13:10:00Z"
}
```

### `GET /api/registry`

Returns:

- normalized Studio registry read model

### `GET /api/repos`

Returns:

- repo cards for Studio Home

Each repo card should include:

- identity
- display name
- contract status
- last-known readiness
- last-known activity
- last opened timestamp

### `GET /api/repos/current`

Returns:

- current repo identity
- effective contract identity
- summary status

### `GET /api/repos/current/overview`

Returns one normalized Overview pane payload built from Ota-owned read surfaces.

### `GET /api/repos/current/contract`

Returns:

- current contract text
- optional normalized contract view
- current contract status

### `GET /api/repos/current/draft`

Returns:

- detect draft view
- inferred contract text
- confidence-grouped inference summary
- pack suggestions when available

### `GET /api/repos/current/topology`

Returns:

- normalized topology read model

### `GET /api/repos/current/run-evidence`

Returns:

- recent operation history
- receipts/log metadata
- current action affordances

### `POST /api/repos/current/actions/init-apply`

Purpose:

- reviewed starter contract apply

Rules:

- requires session token
- requires stale-review safety check
- returns structured action result

### `POST /api/repos/current/actions/detect-merge-apply`

Purpose:

- additive reviewed merge apply

Same safety rules as above.

### `POST /api/repos/current/actions/doctor`
### `POST /api/repos/current/actions/validate`
### `POST /api/repos/current/actions/up`
### `POST /api/repos/current/actions/run`

Purpose:

- launch named Ota operations through the server

All operation launch endpoints must:

- require session token
- accept explicit launch parameters only
- reject implicit shell-like command strings

### `GET /api/repos/current/operations`

Returns:

- current and recent operation history

### `GET /api/repos/current/events`

Future direction:

- SSE stream of operation events for the current repo

Polling may come first, but SSE is the preferred long-term live transport.

## Request rules

Mutation and operation-launch POST bodies must be explicit.

Example `run` body:

```json
{
  "task": "ci",
  "member": null,
  "context": null,
  "backend": null,
  "flags": {
    "stream": true
  }
}
```

The server must reject:

- raw shell commands
- freeform YAML patches
- missing required fields

## Response rules

Every JSON response should include:

- `schema_version`
- `ok`

Read responses may additionally include:

- `data`
- `generated_at`

Mutation or operation-launch responses should additionally include:

- `operation_id`
- `status`
- optional `next`

## Stale-review protection

All reviewed apply endpoints must verify that the reviewed state still matches the current contract.

If stale:

- return `ok: false`
- return `status: "stale_review"`
- return enough identity data for the client to refresh

Example:

```json
{
  "schema_version": 1,
  "ok": false,
  "status": "stale_review",
  "next": "refresh contract and review the new diff before applying"
}
```

## Security and safety

The local API must enforce:

- loopback binding only
- per-session mutation token
- no side-effecting GETs
- no hidden background mutations
- explicit action preview in the UI before launch

## Versioning

The Studio HTTP API should version by `schema_version` in the response body first.

If future routing versioning is needed, prefer:

- `/api/v1/...`

But Phase 1 does not need path versioning if the body schema is explicit and stable.
