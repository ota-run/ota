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

# Up Preview

Status: shipped.

This document defines the proposed read-only preview surface for repo preparation:

- `ota up --dry-run`

The goal is to preview the exact repo-readiness execution plan without inventing a second planning
command and without overloading `doctor` or `explain`.

## Command

```bash
ota up --dry-run [PATH]
ota up --dry-run --json [PATH]
ota up --dry-run --mode container [PATH]
ota up --dry-run --mode native [PATH]
ota up --dry-run --member api [PATH]
```

`ota up --dry-run`:

- require `ota.yaml`
- resolve the same contract path and monorepo member selection as normal `ota up`
- resolve the same effective backend, lifecycle, and execution target as normal `ota up`
- stay read-only

`ota up --dry-run` must not:

- install or remove runtimes, tools, or adapters
- start services
- create or reuse persistent execution state
- execute `setup` or any repo task
- write repo files

## Boundary

- `ota doctor` answers what is wrong
- `ota explain` answers what to do
- `ota up --dry-run` answers what `ota up` would attempt right now

This keeps the preview aligned with the real `up` execution path instead of inventing a parallel
planner with different backend or provisioning resolution.

## Useful cases

- confirm which backend `ota up` would actually use after applying contract preference and CLI overrides
- preview provisioning and service actions before granting execution in CI or an agent workflow
- confirm whether `up` would reuse a persistent container backend or create a new one
- identify the first blocker that would stop `up` before mutating state

## Text output

The text surface stays premium and concise, using the existing `up` hierarchy.

Current shape:

```text
🦦 UP PREVIEW ./ota.yaml

➤ BLOCKED

❖ Mode: dry-run (no write)

Execution
 » Backend: `container`
 » Lifecycle: `persistent`
 » Image: `jdxcode/mise:latest`
 » Target: `ota-a6be4471a4598386`
 » Task: `setup`

Plan
 » provision `java` `21` via `sdkman`
 » provision `curl` `8.7.1` via `apt`
 » skip `jq`; already satisfies contract

➤ Primary Blocker
Adapter bootstrap failed: sdkman
Why: required commands are missing from the container: `curl` and `zip`
Next: install `curl` and `zip` in the container image, then rerun `ota up --dry-run --mode container`

Dry run only
 » no provisioning executed
 » no services started
 » no repo files changed
```

Required text fields:

- preview mode line: `Mode: dry-run (no write)`
- top-level preview status using the execution preview vocabulary: `RUNNABLE`,
  `RUNNABLE WITH WARNINGS`, or `BLOCKED`
- selected `Backend`
- selected `Lifecycle` when one exists
- selected container `Image` when container execution is active
- selected `Target` when one exists
- effective `Task` when `setup` would run
- the ordered action plan
- the first actionable readiness finding, when one exists
- an explicit dry-run note that nothing mutated

The preview should show:

- actions ota would attempt
- actions ota would skip because the current state already satisfies them
- whether service start and service readiness checks would be attempted before `setup`

The preview should not:

- repeat the full doctor finding dump
- collapse into generic prose like “would prepare the repo”
- pretend to know mutations that `ota up` would not actually attempt

## JSON output

`ota up --dry-run --json` mirrors the real planner state directly instead of forcing automation to
scrape text.

Suggested shape:

```json
{
  "ok": false,
  "path": "./ota.yaml",
  "dry_run": true,
  "status": "BLOCKED",
  "phase": "preview",
  "summary": {
    "verdict": "not_ready",
    "agent_verdict": "ready",
    "error_count": 1,
    "warn_count": 0,
    "info_count": 0,
    "primary_blocker": {
      "severity": "error",
      "summary": "Adapter bootstrap failed: sdkman",
      "why": "required commands are missing from the container: `curl` and `zip`",
      "next": "install `curl` and `zip` in the container image, then rerun `ota up --dry-run --mode container`"
    }
  },
  "execution": {
    "backend": "container",
    "lifecycle": "persistent",
    "image": "jdxcode/mise:latest",
    "target": "ota-a6be4471a4598386",
    "task": "setup"
  },
  "overrides": {
    "backend": "container"
  },
  "plan": {
    "actions": [
      "provision `java` `21` via `sdkman`",
      "provision `curl` `8.7.1` via `apt`",
      "run task `setup`",
      "re-check repo readiness"
    ],
    "skipped": [
      "skip `jq`; already satisfies the contract"
    ]
  },
  "blockers": [
    {
      "summary": "Adapter bootstrap failed: sdkman",
      "severity": "error",
      "why": "required commands are missing from the container: `curl` and `zip`",
      "next": "install `curl` and `zip` in the container image, then rerun `ota up --dry-run --mode container`"
    }
  ]
}
```

Required JSON fields:

- `ok`
- `path`
- `dry_run`
- `status`
- `phase`
- `summary`
- `execution`
- `overrides` when explicit execution options were requested
- `plan.actions`
- `plan.skipped`
- `blockers`

`execution` should record:

- `backend`
- `lifecycle` when present
- `image` when container execution is selected
- `target` when present
- `task` when `setup` would run

`overrides` records admitted execution-affecting options such as `backend`, `lifecycle`, or
`host_port`. This lets automation confirm that preview evaluated the requested option rather than
silently dropping it.

`plan.actions[]` is deterministic and ordered the same way the real `up` flow would attempt the
next mutating work.

`plan.skipped[]` only includes actions whose skip reason ota can prove from current state.

`summary` reuses the same verdict vocabulary and primary-blocker semantics as `ota doctor` and
`ota check`, so warning-only previews can stay actionable without pretending the repo is blocked.

`blockers[]` uses the same finding shape and `why` / `next` semantics as the rest of ota’s
machine-readable surfaces and remains limited to execution-stopping preview blockers.

## Exit codes

Exit code contract:

- `0` when the preview is actionable and unblocked
- `1` when the preview identifies a blocking condition that would stop `ota up`
- `1` on contract load or validation failure

This keeps the dry-run surface aligned with `ota up` truthfulness without pretending preview is
always success.

## Non-goals

- a separate `ota plan` command
- repo mutation
- task execution
- service orchestration side effects
- a duplicate readiness engine beside `doctor`

## Implementation rule

The preview should reuse the real `ota up` resolution path:

- contract loading
- member targeting
- backend resolution
- provisioning planning
- service planning
- setup planning

Do not fork a second planner that can drift from the real `up` path.
