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
---
name: ota
description: Use when working on anything Ota-specific: creating, refining, reviewing, or explaining Ota contracts (`ota.yaml`), repo readiness modeling, `ota doctor` / `ota up` / `ota run` flows, agent safety surfaces, Ota Studio boundaries, or when deciding whether a problem belongs in the repo contract or in Ota itself.
---

# Ota

Use this skill when the task is about Ota-specific contract authoring, review, readiness
modeling, or Ota platform judgment.

Do not use this skill for generic YAML generation. It is for truthful repo readiness
modeling, not schema-only completion.

## Core product posture

- Doctor first, contract second.
- `ota.yaml` is the canonical source of repo readiness truth.
- Prefer one explicit operational path over parallel scripts and tribal knowledge.
- If the repo needs ugly glue because Ota lacks a product feature, call it out as an Ota
  gap instead of normalizing the workaround.

## Source priority

Prefer sources in this order:

1. the repo you are actively working on
2. local canonical Ota references:
   - `ota.yaml`
   - `examples/`
   - `docs/spec/contract-reference.md`
   - `docs/spec/json-output-reference.md`
   - `README.md`
3. public references in `references/official-sources.md`

Do not jump straight to public docs if the local repository already contains the canonical
answer.

## Bootstrap flow

Always prefer using the real Ota binary when it is available.

1. Check whether `ota` exists.
2. If it is missing, ask for approval before installing it.
3. Use the official install commands only:
   - macOS / Linux:
     - `curl -fsSL https://dist.ota.run/install.sh | sh`
   - Windows PowerShell:
     - `irm https://dist.ota.run/install.ps1 | iex`
4. After install, use real Ota commands instead of hand-wavy advice.

Prefer the official install docs and repository references in
`references/official-sources.md` when you need canonical links.

## Canonical command workflow

Use the smallest real Ota workflow that fits the task:

- `ota doctor`
  - inspect readiness blockers, warnings, next actions, and agent guidance
- `ota validate`
  - verify structural and semantic contract correctness
- `ota tasks`
  - discover named task surfaces
- `ota run <task>`
  - execute canonical task flows
- `ota up`
  - prepare the repo into a ready state

If a contract already exists, start with `ota doctor` and `ota validate` before editing it.

If the repository has no contract yet:

1. inspect the smallest real operational surface
2. model the minimum truthful contract
3. validate it
4. only then broaden the contract if real repo truth requires it

## Contract authoring workflow

When creating or refining a contract:

1. Identify the real operational truth with the minimum repo reading needed:
   - setup path
   - runtime dependencies
   - env sources and required vars
   - service and readiness ownership
   - canonical tasks
   - agent-safe boundaries
2. Model only what is actually true today.
3. Keep the contract minimal but complete enough that:
   - `ota doctor` is useful
   - `ota up` has a clear path
   - `ota run` executes named tasks honestly
4. Prefer contract fields over repo-local helper scripts when Ota already supports the
   behavior cleanly.

Default modeling areas:

- `project`
- `execution.contexts`
- `env.sources` and `env.vars`
- `services` and readiness
- `tasks`
- `checks`
- `agent`

When deciding where something belongs, prefer:

- `env` for runtime config truth
- `services` for declared managed services and readiness ownership
- `checks` for operator-facing blockers and warnings
- `tasks` for named execution paths
- `agent` for safety and automation boundaries

## Contract review workflow

When reviewing an `ota.yaml`, look for:

- fake readiness
- duplicated truth between tasks, scripts, and contract
- weak or missing blocker checks
- unclear `ota up` path
- over-modeled speculative surfaces
- agent writable/protected paths that are too loose
- tasks that depend on hidden local state
- readiness that only proves a process started, not that the repo is truly ready

A good review should separate:

- repo contract issue
- repo implementation issue
- Ota platform gap

A good review response should usually end with:

- the top contract issues
- any Ota gaps
- the next best change

## Ota gap detection

If the strongest solution is blocked by Ota itself:

- say that explicitly
- identify the missing Ota capability precisely
- prefer the product fix over a repo-local workaround when feasible

Do not normalize:

- shell retry hacks
- duplicated config
- UI surfaces that bypass Ota truth
- local glue used only to route around an Ota defect

## Studio guidance

When the task touches Ota Studio:

- keep one UI system
- keep local and cloud behind adapters
- keep `ota` canonical
- keep the UI consuming shaped Ota data, not terminal scraping

Prefer:

- local mode through `ota studio`
- cloud mode through private backend adapters
- shared view models above both

## Anti-patterns to reject

- giant autogenerated contracts with low trust
- YAML that is valid but not operationally honest
- task surfaces that duplicate existing Ota semantics
- “just put it in a script” when a contract field already exists
- making users carry platform defects in repo-local glue
- overfitting the contract to one temporary workflow

## References

Read `references/official-sources.md` when you need canonical install links, official docs,
or public GitHub references for examples and contract behavior.

## Expected output style

When the task is analytical rather than editing, keep the response high-signal and concrete.

Prefer:

- what is modeled well
- what is weak or missing
- whether the issue belongs in the repo or in Ota
- the narrowest correct next step

Do not produce generic YAML commentary, long tutorials, or broad best-practice lists that are
not tied to the actual repo.
