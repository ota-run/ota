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

Status: proposed, not yet shipped.

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

`ota up --dry-run` should:

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

The text surface should stay premium and concise, using the existing `up` hierarchy.

Suggested shape:

```text
🦦 UP PREVIEW ./ota.yaml

➤ NOT READY

Execution
 » Mode: `container`
 » Lifecycle: `persistent`
 » Target: `ota-a6be4471a4598386`
 » Task: `provisioning`
 » Backend: `container`

Plan
 » provision `java` `21` via `sdkman`
 » provision `curl` `8.7.1` via `apt`
 » reuse persistent container backend
 » skip `jq`; already satisfies contract

Blocked by
 » adapter bootstrap for `sdkman` is not available in the selected environment

Next
 » install the bootstrap prerequisites in the container image, then rerun `ota up --mode container`

Dry run only
 » no provisioning executed
 » no services started
 » no repo files changed
```

Required text fields:

- selected `Mode`
- selected `Lifecycle` when one exists
- selected `Target` when one exists
- effective `Task` or `Scope`
- the ordered action plan
- the first actionable blocker, when one exists
- an explicit dry-run note that nothing mutated

The preview should show:

- actions ota would attempt
- actions ota would skip because the current state already satisfies them
- whether persistent backend state would be reused or created
- whether service start would be attempted before `setup`

The preview should not:

- repeat the full doctor finding dump
- collapse into generic prose like “would prepare the repo”
- pretend to know mutations that `ota up` would not actually attempt

## JSON output

`ota up --dry-run --json` should mirror the real planner state directly instead of forcing
automation to scrape text.

Suggested shape:

```json
{
  "ok": false,
  "path": "./ota.yaml",
  "dry_run": true,
  "execution": {
    "backend": "container",
    "mode": "container",
    "lifecycle": "persistent",
    "target": "ota-a6be4471a4598386",
    "task": "provisioning"
  },
  "plan": {
    "actions": [
      {
        "kind": "provision",
        "subject": "java",
        "version": "21",
        "source": "sdkman"
      },
      {
        "kind": "provision",
        "subject": "curl",
        "version": "8.7.1",
        "source": "apt"
      },
      {
        "kind": "reuse_backend",
        "backend": "container",
        "target": "ota-a6be4471a4598386"
      }
    ],
    "skips": [
      {
        "kind": "already_satisfied",
        "subject": "jq",
        "reason": "already satisfies contract"
      }
    ]
  },
  "blockers": [
    {
      "code": "OTA_ADAPTER_BOOTSTRAP_FAILED",
      "summary": "Adapter bootstrap failed: sdkman",
      "why": "adapter bootstrap for missing adapter `sdkman` via approved source `sdkman-bootstrap` could not complete in the selected execution environment",
      "next": "install the bootstrap prerequisites in the container image, then rerun `ota up --mode container`"
    }
  ]
}
```

Required JSON fields:

- `ok`
- `path`
- `dry_run`
- `execution`
- `plan.actions`
- `plan.skips`
- `blockers`

`execution` should record:

- `backend`
- `mode`
- `lifecycle` when present
- `target` when present
- `task` or `scope`

`plan.actions[]` should be deterministic and ordered the same way the real `up` flow would attempt
them.

`plan.skips[]` should only include actions whose skip reason ota can prove from current state.

`blockers[]` should use the same stable finding codes and `why` / `next` semantics as the rest of
ota’s machine-readable surfaces.

## Exit codes

Planned exit code contract:

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
