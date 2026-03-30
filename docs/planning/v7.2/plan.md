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

# V7.2 Plan

Status: planned.

Source direction:

- [Doctor finding contract](../../spec/doctor-finding-contract.md)
- [Doctor quality bar](../../design/doctor-quality-bar.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Doctor / semantic diff / explain surfaces](../../spec/semantic-diff-and-explain.md)

V7.2 theme:

- make `ota doctor` findings machine-stable
- separate diagnosis text from diagnostic identity
- keep the current doctor response shape intact while adding stronger metadata

## Included capabilities

- stable doctor finding codes
- finding categories and ownership buckets
- structured evidence objects on every finding
- JSON contract updates for doctor and workspace doctor findings

## Non-goals

- do not add hosted control plane APIs in v7.2
- do not add waiver or exception lifecycle in v7.2
- do not split repo readiness and agent readiness into separate top-level verdicts yet
- do not turn `doctor` into a generic policy engine

## Priorities

1. Make doctor findings stable enough for CI, policy, and agent workflows.
2. Preserve the existing human output shape while improving the machine contract.
3. Keep the implementation narrow enough that the doctor engine remains trustworthy.

## Execution slices

1. Finding identity

- assign a stable `code` to each doctor finding family
- add a coarse `category`
- add a primary `owner`
- keep current `severity`, `summary`, `why`, and `next` semantics intact

1. Evidence contract

- attach a structured evidence object to every finding
- include `observed`, `expected`, `source`, `checked_at`, `command`, and `path`
- keep evidence deterministic and machine-readable

1. JSON schema and docs

- update the shared finding schema
- update doctor and workspace doctor JSON references
- document the finding contract as a stable machine surface

1. Regression coverage

- lock the new fields into JSON contract tests
- verify existing doctor and workspace doctor output still behaves as expected

## Success criteria

- every doctor finding has a stable code, category, owner, and evidence payload
- JSON consumers can key off machine-stable diagnostics without parsing prose
- human-facing doctor output remains familiar
- no new doctor behavior drifts beyond the intended contract metadata change
