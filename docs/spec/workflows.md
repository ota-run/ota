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

# Workflows

Use workflows when a repo needs one canonical operational path instead of a loose pile of tasks.

The short version:

- `tasks` are execution primitives
- `checks` are named readiness assertions
- `readiness.probes` are reusable transport-level readiness definitions
- `services` are dependencies
- `workflows` are the repo's intended operational paths

If `ota.yaml` is the repo contract, `workflows` is where the contract says:

> this is the way this repo is meant to become useful

## Why workflows exist

Serious repos usually do not have one flat definition of "ready".

A repo may have:

- contributor setup readiness
- backend development readiness
- frontend development readiness
- full app readiness
- test readiness
- CI readiness

If ota only had `tasks`, operators and agents would have to guess which task is the actual front door.

Workflows solve that by turning low-level contract pieces into one canonical operational path.

## What a workflow is

A workflow is a named path built from existing contract primitives:

- an optional setup task
- an optional run task
- optional required services
- optional readiness checks
- optional exposed endpoints

Example:

```yaml
readiness:
  probes:
    app-ready:
      kind: http
      url: http://127.0.0.1:5678/healthz/readiness
      expect_status: 200
      timeout: 10000

workflows:
  default: app
  app:
    intent: local_development
    description: Canonical local app workflow
    setup:
      task: setup
    run:
      task: dev
    services:
      required:
        - postgres
    readiness:
      probes:
        - app-ready
    exposes:
      - http://127.0.0.1:5678
```

This does not replace `tasks`, `checks`, or `services`.
It composes them into a canonical path.

## When to use workflows

Add workflows when at least one of these is true:

- the repo has more than one valid development path
- `setup` and `run` are not the same thing
- humans keep asking "what should I run first?"
- `ota up` should target a specific app or service path, not generic repo setup
- the repo has backend/frontend or app/worker splits
- agent automation should optimize for a narrower path than the human default

You do not need workflows for every repo.

For a small library with one obvious `setup` and one obvious `test`, plain tasks may be enough.

## When not to use workflows

Do not add workflows just to rename tasks.

Bad reasons:

- the repo only has one trivial path
- the workflow adds no new operational meaning
- the workflow just duplicates task names without clarifying readiness

If the contract still answers the same questions without workflows, keep it simpler.

## How to choose the default workflow

`workflows.default` should name the repo's canonical operator path.

Choose the path that best answers:

- what should a new contributor run first?
- what should `ota doctor` and `ota up` optimize for by default?
- what path best represents the repo's main selfish utility?

That default should be the human-facing repo front door.

It does not have to be the narrowest or cheapest path.

Example:

- repo default workflow: full local app development
- agent default task: backend-only dev server

That split is valid when the human default is broader but the agent-safe runtime should stay narrower.

## How workflows interact with commands

Current command behavior:

- `ota doctor` diagnoses the default workflow by default
- `ota check` checks the default workflow by default
- `ota up` prepares the default workflow by default
- `ota execution plan` resolves the default workflow by default
- `--workflow <name>` selects another declared workflow explicitly

In practice this means a repo can expose:

- one canonical default path
- several secondary paths for backend, frontend, AI, runtime, or CI work

without forcing humans or agents to infer that from task names alone.

## How workflows relate to tasks

Tasks remain the execution layer.

Use tasks for:

- commands you actually run
- dependency chains
- runtime declarations
- execution contexts

Use workflows for:

- repo-level intent
- choosing the canonical setup/run path
- grouping readiness under one named operational target

Good boundary:

- `tasks.setup` installs dependencies
- `tasks.dev` starts the app
- `checks.app-health` probes the app
- `workflows.app` says those belong to the same operational path

## How workflows relate to agents

Workflows and agent hints are not the same thing.

- `workflows.default` is the canonical repo operational path
- `agent.default_task` is an agent-facing hint
- `agent.entrypoint` is an agent bootstrap hint

Use this split when needed:

- human default workflow = broader, more complete, more representative
- agent default task = narrower, cheaper, safer

Do not force the repo default workflow to become agent-shaped unless that really is the main operator path.

## Recommended patterns

Good workflow names:

- `app`
- `backend`
- `frontend`
- `runtime`
- `worker`
- `ai`

Good intents:

- `local_development`
- `backend_development`
- `frontend_development`
- `local_runtime`

Prefer names that describe the operational path, not the implementation detail.

## Readiness reuse

Workflows can now reference reusable `readiness.probes` directly.

Use that when:

- one HTTP readiness probe should be shared by more than one workflow
- the same readiness target should also appear as a named `check`
- you want `doctor` to validate workflow readiness without forcing an inline shell command such as
  `node -e "fetch(...)"` into the contract

Use `checks[].probe` when the same underlying probe should also participate in the explicit
`ota check` surface with a named severity.

## Design rule

Use workflows when they make the repo's operational truth more explicit.

Do not use them to create a second task system.

The best workflow is the one that makes `ota doctor`, `ota up`, and `ota execution plan` answer the same question clearly:

> what is the canonical way to make this repo useful?
