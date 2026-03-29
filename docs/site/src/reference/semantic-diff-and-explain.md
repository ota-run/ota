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

`ota diff` and `ota explain` are shipped read-only contract commands.

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
The summary counts appear after the field-level sections.
Use it before writing contract changes or in CI when you need semantic impact instead of a raw YAML
diff.

## `ota explain`

`ota explain` turns readiness findings into ordered remediation steps.

It stays read-only and deterministic.
Use it when you want a blocker list converted into a fix order that a human or agent can follow
without re-reading the raw findings.
