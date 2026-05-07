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

# Ota Studio Event Schema

Status: planned.

This document defines the build-facing event and operation contract for Studio.

Studio, CLI, and agents must eventually publish or consume the same operation truth. The UI must
never infer execution state from terminal text scraping or filesystem guesswork when a structured
event exists.

## Design rules

1. Every meaningful Ota action yields:
   - one final structured result
   - zero or more structured events while the action is active
2. Events are append-only facts for one operation id.
3. The event schema is repo-scoped and deterministic.
4. Event consumers may render different views, but must not invent semantics missing from the event.
5. `operation_id` is the durable correlation key shared across events, results, receipts, and log lines.

## Operation identity

Each operation must carry:

- `schema_version`
- `operation_id`
- `repo_root`
- optional `session_id`
- optional `contract_path`
- optional `member`
- `kind`
- `requested_at`
- `source`
- optional `requested_by`

For cross-process visibility (e.g., terminal-launched operations surfaced in Studio), `operation_id`
must be emitted in:

- the final operation result object
- generated receipt metadata
- any machine-readable log event that belongs to the operation

### `operation_id`

Requirements:

- globally unique for the local machine session history
- opaque to clients
- stable across all events for the operation
- safe to use as the join key for logs, receipts, and UI cards

Preferred shape:

- ULID or UUIDv7

## Operation kinds

Initial canonical kinds:

- `doctor`
- `validate`
- `detect_dry_run`
- `detect_merge_apply`
- `detect_rewrite_apply`
- `init_dry_run`
- `init_apply`
- `env`
- `execution_topology`
- `up`
- `run_task`
- `workspace_doctor`
- `workspace_up`
- `workspace_run_task`

Studio Phase 1 does not need all of these live, but the enum should be defined once here.

## Source metadata

`source` identifies the interface that initiated the operation.

Canonical values:

- `studio`
- `terminal`
- `agent`
- `workspace`
- `automation`

`requested_by` is optional descriptive metadata.

Examples:

- authenticated local username
- agent display name
- automation label

Clients must treat `requested_by` as descriptive, not authorization truth.

## Event envelope

Every event must carry:

- `schema_version`
- `event_id`
- `operation_id`
- `repo_root`
- optional `contract_path`
- optional `member`
- `kind`
- `timestamp`
- `status`
- optional `phase`
- optional `message`

### `schema_version`

Rules:

- version the event schema independently from the Studio UI
- begin at `1`
- additive fields are allowed within a major version
- breaking field or semantic changes require a major version bump

### `status`

Canonical values:

- `queued`
- `running`
- `ready`
- `passed`
- `failed`
- `canceled`
- `blocked`

`ready` is reserved for readiness milestones, not final operation success.

## Event kinds

Canonical event kinds:

- `operation.started`
- `operation.phase_changed`
- `operation.blocked`
- `task.started`
- `task.step.started`
- `task.step.output`
- `task.step.ready`
- `task.step.finished`
- `task.finished`
- `receipt.written`
- `operation.finished`

### `operation.started`

Must include:

- initial `kind`
- initial `status`
- optional launch context

### `operation.phase_changed`

Must include:

- `phase`
- `message`

### `task.started`

Must include:

- `task`
- optional `relation`
- optional execution identity

### `task.step.started`

Must include:

- `task`
- `step`
- optional `relation`

### `task.step.output`

Must include:

- `task`
- `step`
- `stream`
- `chunk`

Canonical `stream` values:

- `stdout`
- `stderr`
- `system`

### `task.step.ready`

Must include:

- `task`
- `step`
- optional readiness metadata

### `task.step.finished`

Must include:

- `task`
- `step`
- `status`
- optional `exit_code`

### `task.finished`

Must include:

- `task`
- `status`
- optional `exit_code`

### `receipt.written`

Must include:

- `receipt_path`
- optional `archive_path`
- optional summary counts

### `operation.finished`

Must include:

- final `status`
- optional `exit_code`
- optional `receipt_path`
- optional `next`

## Optional execution detail fields

These fields are additive and should be attached when the underlying operation truth has them:

- `task`
- `step`
- `relation`
- `backend`
- `context`
- `lifecycle`
- `target`
- `provider`
- `cwd`
- `listener`
- `activation_mode`
- `readiness_kind`
- `target_name`
- `address_view`
- `receipt_path`
- `archive_path`
- `next`

## Example envelope

```json
{
  "schema_version": 1,
  "event_id": "01JV8JQ7K6VQ4N8S95VQZ2V1VG",
  "operation_id": "01JV8JPR4X4D3H6H6D12G9A6YQ",
  "repo_root": "/work/acme/api",
  "contract_path": "/work/acme/api/ota.yaml",
  "kind": "task.step.started",
  "timestamp": "2026-05-02T12:44:18Z",
  "status": "running",
  "phase": "execute",
  "task": "typecheck",
  "step": "task:typecheck",
  "backend": "native",
  "context": "app",
  "source": "agent",
  "requested_by": "codex"
}
```

## Transport rules

1. GET endpoints must not mutate state.
2. Studio may consume events by polling first, then by SSE later.
3. Event ordering is per operation id and timestamp order.
4. Consumers must tolerate duplicate delivery.
5. Consumers must tolerate missing optional fields.

## Result contract

Final operation results use the same canonical fields for API and process boundaries:

Events do not replace the final result. Every operation still needs a final result object with:

Required:

- `schema_version`
- `operation_id`
- `ok`
- `status`

Optional:

- `exit_code`
- `receipt_path`
- `archive_path`
- `next`
- operation-kind-specific summary data

Studio cards may render from events while active, but the final settled state should reconcile
against the final result and receipt truth.

## Evolution rules

Allowed without a major schema bump:

- new optional fields
- new event kinds ignored by older consumers
- additive metadata objects

Not allowed without a major schema bump:

- removing required fields
- changing field meaning
- changing canonical enum meanings
- changing status semantics
