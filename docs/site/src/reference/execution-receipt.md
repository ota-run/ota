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

# Execution Receipt

The execution receipt is the machine-readable record for ota execution commands.

Use it to answer:

- what `ota` believed was required
- what it chose to run against
- what it reused or acquired
- what succeeded, failed, or was blocked
- what the safe next step is

## Source model

This page is the canonical public reference for execution receipts. It adds
examples, use cases, and operator guidance so the page stands on its own while
staying aligned with shipped behavior.

This surface applies to execution flows, not diagnosis or inference.

## What it records

The receipt is most useful when you want to know:

- what repo or workspace ota ran against
- what contract it used
- what backend and lifecycle were chosen
- what tasks, steps, or services actually ran
- what env or policy values shaped execution
- what was blocked
- what the safe next step is

## Example

For a repo task run, the receipt can look like this in JSON:

```json
{
  "ok": true,
  "path": "/workspace/acme/app/ota.yaml",
  "scope": "repo",
  "contract": "/workspace/acme/app/ota.yaml",
  "backend": "container",
  "lifecycle": "ephemeral",
  "env_sources": [
    {
      "name": "DATABASE_URL",
      "value": "postgres://localhost/app",
      "source": "task env"
    },
    {
      "name": "JAVA_HOME",
      "value": "/opt/jdk-22",
      "source": "org policy"
    }
  ],
  "steps": [
    {
      "order": 1,
      "label": "setup",
      "status": "ok"
    },
    {
      "order": 2,
      "label": "test",
      "status": "ok"
    }
  ],
  "summary": {
    "error_count": 0,
    "warn_count": 0,
    "info_count": 0,
    "step_count": 2
  },
  "next": "run `ota run test` again after your code changes"
}
```

For a workspace run, the same receipt tells you which repos were ready,
which were not, and where the blocker came from. That is the value: one
structured record you can compare across runs.

Workspace example:

```json
{
  "ok": false,
  "path": "/workspace/acme/ota.workspace.yaml",
  "scope": "workspace",
  "contract": "/workspace/acme/ota.workspace.yaml",
  "summary": {
    "error_count": 1,
    "warn_count": 0,
    "info_count": 0,
    "step_count": 2
  },
  "steps": [
    {
      "order": 1,
      "label": "api",
      "status": "ok"
    },
    {
      "order": 2,
      "label": "web",
      "status": "blocked"
    }
  ]
}
```

That shape is useful when the first repo succeeded, the second repo was blocked, and you need to
see the exact handoff point without reading terminal logs.

## Use cases

- confirm what ota actually executed after a successful `ota run`
- debug a failed `ota up` without guessing which backend or lifecycle was used
- inspect which steps were blocked before a task could finish
- compare a native run and a container-backed run
- feed a machine-readable execution trail into CI or agent workflows
- review workspace roll-ups after multi-repo bootstrap or execution

## How to use it

- use `ota run --json` when you need the task-level execution record
- use `ota up --json` when you need the ready/not-ready roll-up and receipt
- use `ota workspace run --json` or `ota workspace up --json` when you need
  workspace-wide execution records

The receipt is meant to be read after execution, not before it. If you need to
know whether a repo is ready to run, use `ota doctor` or `ota detect` first.

## What it is not

- it is not a diagnosis report
- it is not a contract inference result
- it is not a replacement for `doctor`
- it is not a replacement for `detect`
- it is not live logs
