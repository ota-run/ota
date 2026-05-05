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

# Assist Operations

Status: planned, with the first shipped slice now covering `ota assist declare-readiness`.

This document defines the long-term product contract for `ota assist`.

`ota assist` is not a chat product and must not become a second source of truth. It is the
deterministic contract-operation layer above Ota's existing repo truth:

- `ota.yaml` remains canonical
- Ota core remains the validator, detector, planner, and execution engine
- `ota assist` proposes bounded contract changes against that same truth

The goal is to reduce author burden without weakening trust.

## Product model

`ota assist` is a reviewed mutation surface.

It works like this:

1. parse one bounded assist intent
2. inspect current repo and contract truth
3. propose one deterministic contract change set
4. show exact assumptions, exact diff, and exact next validation
5. write only when the user explicitly requests apply
6. validate after apply

It must not:

- accept freeform fuzzy mutation as the core contract
- invent hidden state or a second registry of repo truth
- write arbitrary YAML without a structured proposal
- replace `doctor`, `detect`, `init`, `diff`, or `up`

## Core principles

### Deterministic first

The proposal engine must be rule-based, schema-aware, and testable.

Natural-language input may exist later, but only as a front-door translator into one of the
declared assist operations. The proposal logic itself must remain deterministic.

### Preview first

`ota assist` defaults to preview mode.

No assist operation may write by default.

### One mutation contract

Every assist operation must return the same proposal shape:

- what operation was requested
- what subject was targeted
- what assumptions were made
- what exact fields would change
- what exact YAML diff would be applied
- what validation should run next

### Refuse weak guesses

When repo or contract truth is too weak for a bounded proposal, `ota assist` must refuse cleanly
and tell the user what extra selector, declaration, or command is needed.

Examples:

- multiple candidate listeners
- no unique service producer
- no existing runtime surface to attach readiness to
- a requested normalization would mix unrelated concerns

## Canonical command catalog

These are the stable long-term assist operations for the current Ota surface:

- `ota assist add-task`
- `ota assist bind-task`
- `ota assist declare-readiness`
- `ota assist declare-service`
- `ota assist declare-env`
- `ota assist wire-setup`
- `ota assist normalize`

No alias duplication is required.

## Common flags

Every assist operation should converge on this common behavior:

```bash
ota assist <operation> [selectors...] --json
ota assist <operation> [selectors...] --write
ota assist <operation> [selectors...] --plain
ota assist <operation> [selectors...] --member <name>
ota assist <operation> [selectors...] [PATH]
```

Rules:

- default mode is preview
- `--write` applies the proposed contract mutation
- `--json` emits the machine-readable proposal or apply result
- `PATH` follows normal repo contract path resolution
- `--member` targets a monorepo member contract through the existing merged-member path

`--dry-run` is not required because preview is already the default.

## Stable proposal contract

Every assist operation should emit one stable proposal shape in JSON.

Minimum shape:

```json
{
  "ok": true,
  "mode": "preview",
  "operation": "declare-readiness",
  "subject": {
    "task": "dev"
  },
  "inputs": {
    "style": "spring-http"
  },
  "assumptions": [
    "task `dev` is the selected long-running app task",
    "listener `http` is the selected readiness surface"
  ],
  "changes": [
    {
      "path": "tasks.dev.runtime.readiness",
      "action": "set",
      "before": null,
      "after": {
        "kind": "http"
      }
    }
  ],
  "diff": "...",
  "validation": [
    "ota validate",
    "ota doctor"
  ],
  "next": "rerun with `--write` to apply"
}
```

Required top-level fields:

- `ok`
- `mode`: `preview` or `write`
- `operation`
- `subject`
- `inputs`
- `assumptions`
- `changes`
- `diff`
- `validation`
- `next`

When the operation refuses, it should return:

- `ok: false`
- `operation`
- `subject` when known
- `why`
- `next`
- `candidates` when useful

## Shared apply lifecycle

For every assist operation:

1. load and validate current contract state first
2. inspect repo signals only where the operation requires them
3. build one bounded proposal
4. render exact diff
5. if `--write` is set:
   - write through Ota core
   - re-run validation
   - optionally re-run `ota doctor` when the operation changes readiness, setup, or execution shape

`ota assist` must never patch raw YAML text directly when a typed contract write path is available.

## Operation definitions

### `ota assist add-task`

Purpose:

- add one new declared task to `ota.yaml`

Minimum required selector:

- `--name <task>`

Recommended selector:

- `--kind <command|service|setup|check|sandbox>`

Behavior:

- proposes the narrowest valid task skeleton
- may add `runtime`, `listeners`, `readiness`, or `targets` only when the selected task kind
  requires them and repo signals are strong enough
- must not guess repo-specific command bodies beyond bounded starter logic unless the user supplied
  them explicitly

Typical validation:

- `ota validate`
- `ota tasks`

### `ota assist bind-task`

Purpose:

- bind one declared task to a producer surface that already exists in the contract

Minimum required selectors:

- `--task <consumer-task>`
- `--to <binding-target>`

Canonical `--to` grammar:

- `<task>:<listener>` for a task runtime producer, for example `dev:http`
- `service:<name>:<endpoint>` for a top-level managed service endpoint

Behavior:

- proposes `tasks.<consumer>.targets.<name>`
- selects `address_view` deterministically from the active topology shape
- may propose `host`, `topology`, or `internal`
- refuses when multiple listeners or multiple producer surfaces are equally valid without more
  user input

Typical validation:

- `ota validate`
- `ota execution topology`

### `ota assist declare-readiness`

Purpose:

- declare or refine readiness for a task runtime service or top-level managed service

Selectors:

- one of:
  - `--task <task>`
  - `--service <service>`
- optional style selector:
  - `--style spring-http`
  - future styles may be added only when they map to deterministic templates

Behavior:

- proposes structured readiness only
- must use the shipped readiness model:
  - `kind`
  - `listener` or `from`
  - `method`
  - `headers`
  - `success.status`
  - `body.contains`
  - `interval`
  - `timeout`
  - `retries`
  - `start_period`
- may still preserve legacy top-level `readiness.run` when the user asked only for review and Ota
  cannot express the existing probe shape structurally
- must refuse contradictory output such as `method: HEAD` plus `body.contains`

Typical validation:

- `ota validate`
- `ota doctor`

### `ota assist declare-service`

Purpose:

- add or refine one top-level managed `services.<name>` declaration

Minimum required selector:

- `--name <service>`

Behavior:

- proposes the canonical service shape for the chosen manager/runtime surface
- may fill:
  - `manager`
  - `required`
  - `endpoints`
  - `readiness`
- must not invent undeclared topology outside the selected service

Typical validation:

- `ota validate`
- `ota services`
- `ota doctor`

### `ota assist declare-env`

Purpose:

- add or refine environment declarations and source-backed env inputs

Behavior:

- proposes changes under:
  - `env.vars`
  - `env.sources`
  - task-level env only when explicitly requested
- should prefer canonical declared source kinds over opaque shell glue
- must preserve existing deterministic precedence rules

Typical validation:

- `ota validate`
- `ota env`
- `ota doctor`

### `ota assist wire-setup`

Purpose:

- create or refine the `setup` path so `ota up` can prepare the repo truthfully

Behavior:

- may create or adjust `tasks.setup`
- may populate `setup.requires_services`
- must respect the shipped phased `ota up` model:
  - pre-setup services come only from `setup.requires_services`
  - remaining required services start after setup
- must not widen setup into a second orchestration model

Typical validation:

- `ota validate`
- `ota up --dry-run`
- `ota doctor`

### `ota assist normalize`

Purpose:

- repair contract shape so existing declarations live in the right canonical place

Behavior:

- may move misplaced or redundant declarations into the correct contract section
- may normalize execution-context placement, readiness ownership, or setup wiring
- must remain bounded to one normalization intent at a time
- must not act as an arbitrary beautifier or formatter-only command

Typical validation:

- `ota validate`
- operation-specific read path depending on what changed

## Adoption boundary

`ota assist` should improve adoption by reducing YAML burden, but it must not bypass the existing
doctor-first trust model.

That means:

- use `doctor`, `detect`, `execution topology`, and readiness truth as inputs
- keep every proposal reviewable
- keep every mutation explainable
- keep the write path explicit

## AI boundary

AI is optional.

If Ota later supports a natural-language front door, it should only translate fuzzy user text into
one of the declared assist operations.

Example:

- user says: `declare readiness for this Spring Boot app`
- translation result: `ota assist declare-readiness --task dev --style spring-http`

The deterministic proposal engine must remain the same whether the request came from:

- direct CLI flags
- Studio or a future local UI
- a hosted dashboard
- an AI translation layer

## Non-goals

- freeform conversational config editing as the primary product model
- hidden writes
- schema-less mutation actions
- a second contract store
- replacing `ota init`, `ota detect`, `ota doctor`, `ota diff`, or `ota up`

## Recommended implementation order

First implementation slice:

1. `ota assist declare-readiness`
2. `ota assist bind-task`
3. `ota assist wire-setup`
4. `ota assist add-task`

Reason:

- readiness, topology binding, and setup orchestration already have strong shipped primitives
- these operations solve immediate adoption pain
- they exercise the full preview/apply/validate model without needing a general-purpose mutator
