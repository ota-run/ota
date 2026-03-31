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
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Doctor Finding Contract

Status: draft.

Source direction:

- current doctor and finding serialization in `src/doctor.rs`
- doctor output rendering in `src/cli/commands.rs`
- shared finding schema in `docs/spec/json-schemas/shared.json`
- JSON output reference in `docs/spec/json-output-reference.md`

## Purpose

`ota doctor` is the authoritative repo-readiness and agent-safety verdict for a contract.
This spec defines the stable finding contract that lets humans, agents, CI, and future policy layers consume
doctor output deterministically.

## Scope

This contract applies to:

- repo-level `ota doctor`
- workspace-level `ota workspace doctor`
- JSON output surfaces derived from doctor findings

This contract does not define the hosted control plane, waiver lifecycle, or fleet reporting.
Those belong to later enterprise work.

## Required finding shape

Every doctor finding must include:

- `severity`
- `code`
- `category`
- `owner`
- `summary`
- `why`
- `next`
- `evidence`

## Field semantics

### `severity`

The human-facing impact class for the finding.

Allowed values:

- `error`
- `warn`
- `info`

### `code`

A stable machine identifier for the finding.

Rules:

- must be namespaced with `OTA_`
- must remain stable across wording changes
- must not depend on punctuation or free-form text
- must identify the underlying condition, not the rendered sentence

Examples:

- `OTA_CONTRACT_MISSING`
- `OTA_BACKEND_CLI_MISSING`
- `OTA_POLICY_PACK_VIOLATION`
- `OTA_SERVICE_CHECK_FAILED`

### `category`

A coarse diagnostic class used for filtering and aggregation.

Allowed values:

- `contract`
- `execution`
- `policy`
- `service`
- `environment`
- `remote`
- `workspace`

### `owner`

The primary responsibility bucket for the finding.

Allowed values:

- `repo_contract`
- `host`
- `service`
- `workspace_acquisition`
- `org_policy`
- `remote_backend`
- `agent_safety`

### `summary`

A short human-readable headline.

### `why`

An explanation of the condition and why it matters.

### `next`

The next recommended action that clears or reduces the finding.

### `evidence`

A structured evidence object that carries the facts behind the finding.

Required evidence fields:

- `observed`
- `expected`
- `source`
- `checked_at`
- `command`
- `path`

Rules:

- fields may be empty when the data is unavailable
- the object itself must still be present on every finding
- evidence must support the code and category, not replace them

## Stability rules

- existing `summary` wording may change without changing the code
- `code`, `category`, and `owner` are the stable machine contract
- `evidence` must be deterministic enough for JSON consumers to compare across runs
- `ota doctor --json` and `ota workspace doctor --json` must stay schema compatible once this contract ships

## Deferred work

The following are intentionally out of scope for this slice:

- explicit repo-vs-agent top-level verdict split
- waiver and exception lifecycle
- fleet reporting
- hosted control plane APIs
- policy distribution and rollout
- approval workflow orchestration
- org RBAC / SSO / retention policy

Those can build on this contract later, but they are not required to make the finding model stable.

## Success criteria

This contract is complete when:

- every doctor finding serializes with a stable `code`, `category`, `owner`, and `evidence`
- JSON consumers can filter and group findings without depending on summary text
- wording changes do not break machine consumers
- the current `ok` / `findings` response shape remains intact for v7.2
