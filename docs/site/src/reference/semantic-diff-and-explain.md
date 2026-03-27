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

`ota diff` is the shipped read-only contract comparison command.
`ota explain` remains a proposed readiness-to-remediation command.

Use `ota diff` when you want to compare two contract states semantically.
Use `ota explain` when you want a readiness report turned into a fix plan.

The spec keeps them separate from:

- `ota detect`, which infers
- `ota doctor`, which diagnoses
- `ota init`, which bootstraps

## Why it matters

- helps agents propose smaller, safer edits
- helps humans review contract impact without reading raw YAML diffs
- keeps remediation separate from inference

## `ota diff`

`ota diff` compares two repo or workspace contracts as structured YAML and reports added,
missing-in-target, and changed fields in deterministic order.

## `ota explain`

`ota explain` is still planned. It will turn readiness findings into ordered remediation steps.

## Planned contract

- `ota explain` should turn findings into ordered remediation steps
- both should stay read-only and deterministic
