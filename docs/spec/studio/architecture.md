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

# Ota Studio Architecture

Status: planned.

## Architecture goal

Studio must become interactive and premium without splitting truth away from Ota core.

The right architecture is:

- one Ota engine
- one local Studio service
- one repo registry
- one operation and event model
- multiple clients
  - CLI
  - Studio
  - agents
  - later CI or hosted adapters where useful

The operation/event model is the schema authority for Studio interactions and must remain aligned with
[`event-schema.md`](event-schema.md).

## Primary model

Studio is a local app served by Ota.

### Required properties

- local-first
- no hosted dependency for core usage
- deterministic
- repo-aware
- safe when Studio is closed
- file-backed local persistence by default

CLI must continue to work fully without Studio running.

Studio must never become a required daemon for ordinary Ota operations.

The initial technical posture should stay thin:

- start with Ota-owned JSON command surfaces
- do not add a custom parser in JS
- do not recreate contract or execution semantics in frontend code
- avoid a daemon-first design before the local web flow is proven
- do not introduce a local database as part of the baseline Studio architecture

Local Studio persistence should stay file-backed for offline and non-enterprise users.

Allowed local persistence:

- JSON registry/state files
- repo receipts and durable log artifacts
- other small explicit file-backed caches when Ota already has the same pattern elsewhere

Disallowed as baseline Studio architecture:

- SQLite or another embedded database by default
- a local persistence layer that is harder to inspect or reset than plain files

If richer synced persistence is ever needed later, that should be treated as a separate enterprise
or hosted boundary, not silently pulled into the default offline Studio architecture.

## Server model

Primary future model:

- `ota studio` starts a local Studio server
- server listens on loopback only
- server opens the default browser
- browser UI connects back to the local Studio server

The server owns:

- current repo focus
- repo registry access
- Studio-specific HTTP endpoints
- Studio-triggered Ota operations
- later event subscriptions

The server is an on-demand local session service, not a permanent required daemon.

## Repo registry

Studio needs a global local repo registry.

Proposed path:

- Linux/macOS: `$HOME/.ota/studio/registry.json`
- Windows: `%APPDATA%\\ota\\studio\\registry.json`

Platform resolution rule:

- use the Ota user root resolution helper and the OS config-data path when available
- ensure the resolved path is user-scoped and non-root-inaccessible

Registry data should include:

- repo root
- contract path
- project name when available
- last opened timestamp
- last known Studio mode/version metadata
- last known readiness summary
- optional favorite/pinned flag later

Auto-registration rule:

- when `ota studio` is run inside a repo with `ota.yaml`, add or refresh that repo in the registry

Registry responsibilities:

- recent repos
- repo switching
- remembered state
- future landing/home screen

## State model

Separate state layers clearly.

### Canonical state

Owned by Ota core:

- contract
- validation
- doctor findings
- detect draft
- topology
- receipts
- logs
- execution results

### Studio local UI state

Owned by Studio:

- active repo selection
- open pane
- split-view state
- temporary filters
- panel expansion/collapse

Studio local UI state must never become canonical contract or runtime truth.

## Operation model

Studio-triggered actions and CLI-triggered actions should converge on one operation model.

Every meaningful Ota action should have:

- operation id
- repo root
- operation kind
- timestamp
- current phase
- final result

Every meaningful Ota action should produce two things:

1. result

- exit code
- receipt
- artifacts
- structured JSON

2. event stream

- what is happening while it runs

Examples:

- doctor
- validate
- detect_dry_run
- init_dry_run
- init_apply
- detect_merge_apply
- up
- run_task

This is the right foundation for future live activity.

## Event model

Studio should ultimately consume structured Ota events instead of scraping terminal text.

Proposed event families:

- `operation.started`
- `operation.phase_changed`
- `task.started`
- `task.step.started`
- `task.step.output`
- `task.step.ready`
- `task.step.finished`
- `task.finished`
- `receipt.written`
- `operation.finished`

Required event fields:

- `operation_id`
- `repo_root`
- `timestamp`
- `kind`
- `phase`
- `status`
- `message`
- optional task/step identity
- optional backend/context/target/provider
- optional receipt path
- optional log chunk

Recommended execution/event detail fields:

- `member`
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
- `requested_by`
- `source`

`requested_by` and `source` are important for future agent visibility. Studio should be able to
show whether an operation came from:

- Studio
- terminal
- agent
- later other Ota-integrated clients

## Event transport direction

Do not begin with a permanent global daemon.

Preferred order:

1. interactive local Studio server
2. Studio-triggered operation status
3. on-demand repo-scoped event publishing/subscription
4. broader CLI/agent event integration

Avoid:

- a global always-on daemon as the starting point
- terminal scraping
- frontend-side orchestration inference

Studio should eventually receive event updates from:

- its own triggered operations
- other Ota processes in the same repo

But that should come from structured events, not terminal scraping.

## UI composition

The Studio frontend should be app-structured.

Suggested high-level composition:

- app shell
- repo switcher
- overview dashboard
- contract pane
- draft pane
- topology pane
- run / evidence pane

The frontend must be thin with respect to semantics.

It should:

- render Ota-owned data
- trigger Ota-owned operations
- not duplicate planner, validator, or execution logic

## Initial integration posture

The early Studio server should consume existing Ota machine-readable surfaces first, for example:

- `ota doctor --json`
- `ota detect --dry-run --json`
- `ota validate --json`
- `ota env --json`
- `ota tasks ... --json`
- `ota services ... --json`
- receipts JSON

This is the right starting posture because it proves product value without inventing parallel core
logic.

Studio should not parse many unrelated command outputs forever, though. Once Studio needs become
stable, Ota core should promote those needs into dedicated UI-safe read models.

## HTTP/API surface

Studio server endpoints should remain local and internal, but still disciplined.

Expected endpoint families:

- repo registry
- repo snapshot/overview
- contract review/apply
- topology
- activity
- operations
- later event subscriptions

The long-term direction is not “many command-shaped endpoints.”
The direction is:

- Studio-focused read models
- Studio-focused mutation entrypoints
- structured operation/event feeds

The long-term core API families should be:

1. Contract View API

- declared contract model
- normalized, schema-stable, UI-safe representation

2. Draft / Inference API

- detected fields
- confidence
- source
- merge/write eligibility
- suggested packs

3. Topology API

- normalized graph of workloads, services, targets, contexts, and backends
- declared versus effective address resolution
- activation and readiness edges
- links back to contract and evidence surfaces

## Mutation model

All Studio writes must route through Ota core write paths.

Allowed direction:

- Studio asks Ota core to perform a named reviewed action
- Ota core applies it
- Studio refreshes on the resulting truth

Required safety boundary:

- reviewed writes must not execute against stale contract state
- Ota core must enforce either a repo-scoped write lease or an equivalent contract revision check
  before any Studio-reviewed apply action executes
- if the contract changed after review, Studio must refresh the review state before allowing the
  write to proceed

Disallowed direction:

- Studio edits YAML directly and treats that as canonical
- Studio invents merge/rewrite semantics in JS

## Execution model

Studio-triggered execution should route through the same Ota run/up flows as the CLI.

The UI must show:

- exact requested action
- selected backend/context/lifecycle
- selected flags when they materially affect execution
- logs
- outcome
- resulting receipt/evidence

Studio must not become an opaque launcher.

## Permissions and trust

Studio is a local power surface, so boundaries must stay explicit.

Rules:

- reviewed writes only
- explicit run actions only
- clear command/result visibility
- preview exact task/context/backend before execution
- no hidden background mutation
- no silent shell configuration edits
- no invented “healthy” status without receipt or runtime evidence

## Static prototype retirement

The earlier static Studio implementation is not the target architecture and should not survive as a
supported product mode.

Allowed preservation:

- payload normalization code that still serves the interactive shell
- server bootstrapping code that still serves the interactive shell
- rendering fragments that materially accelerate the real app shell

Disallowed preservation:

- a second user-facing Studio mode
- snapshot-specific product semantics
- continued product investment in report-style rendering
