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

# Semantic Diff and Explain

Status: `ota diff` and `ota explain` are shipped.

This document defines two proposed read-only surfaces for Ota:

- `ota diff` for semantic contract comparison
- `ota explain` for remediation guidance from readiness findings

The goal is to help humans and agents understand contract change impact without turning
`detect` or `doctor` into overloaded commands.

## `ota diff`

Compare two Ota contract states semantically.

Current behavior:

- compare repo contracts or workspace contracts as structured objects
- report added, missing-in-target, and changed fields in deterministic order
- surface readiness impact directly instead of forcing users to infer it from raw YAML
- preserve deterministic ordering in text and JSON output
- remain read-only

Useful cases:

- review what a proposed contract change will do before writing it
- compare a branch against main in CI
- summarize the impact of a workspace bootstrap change

Text output:

- grouped field-level changes
- summary counts at the end
- readiness impact notes

JSON output:

- `ok`
- `path`
- `base`
- `target`
- `summary`
- `changes`

## `ota explain`

Turn readiness findings into a remediation plan.

Behavior:

- takes doctor findings as input
- groups findings into actionable fix steps
- keeps the plan read-only and deterministic
- never auto-applies changes

Useful cases:

- agent asks for the next best fix order
- a human wants one concise path from blockers to readiness
- CI wants a stable remediation summary to paste into a ticket or comment

Text output:

- ordered remediation steps
- confidence or priority where relevant
- explicit commands when safe

JSON output:

- `ok`
- `path`
- `summary`
- `steps`

## Non-goals

- auto-writing contract changes
- fuzzy natural-language repair
- hiding readiness blockers behind a generic suggestion engine
- replacing `doctor` or `detect`

## Boundary

- `diff` is comparison
- `explain` is remediation
- `detect` stays inference
- `doctor` stays diagnosis
