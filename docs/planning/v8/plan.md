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

# V8 Plan

Status: planned.

Source direction:

- [Execution receipt](../../spec/execution-receipt.md)
- [Semantic diff and explain](../../spec/semantic-diff-and-explain.md)

V8 theme:

- execution receipts and operability
- semantic contract impact
- remediation visibility for humans and agents

This is the next active slice after v7.2.

## Included capabilities

- deterministic execution receipt for `ota run` and `ota up`
- semantic impact diff
- deterministic remediation plan

## Current implementation gaps

- receipts exist, but text and JSON still need tighter normalization across repo and workspace execution surfaces
- semantic diff exists as a command surface, but the impact model still needs to be tightened into a stable contract-facing artifact
- remediation output exists, but it should become more deterministic and explicitly ordered

## Priorities

1. Make execution outcomes reviewable and supportable
2. Make contract change impact explicit
3. Keep diagnosis, inference, and execution separate

## Execution slices

1. Execution receipt schema

- normalize the receipt as the deterministic machine-readable artifact for execution commands
- keep repo, workspace, and JSON receipt fields aligned
- preserve stable summary data without duplicating business logic in renderers

1. Semantic impact diff

- compare contract states by meaning, not raw YAML
- report readiness impact, execution impact, and safe next actions
- keep the impact model deterministic and contract-focused

1. Deterministic remediation plan

- turn findings into ordered fix steps
- keep it read-only and explicit
- make the remediation order stable for humans and agents

## Success criteria

- `ota run` and `ota up` can emit a structured receipt
- semantic diff explains change impact without raw YAML parsing
- remediation plans are deterministic and safe
