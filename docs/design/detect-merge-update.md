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

# Detect Merge/Update Design

`ota detect` now has three trust levels of behavior:

1. dry-run candidate output
2. conservative first-write for repos without `ota.yaml`
3. dry-run comparison preview when `ota.yaml` already exists

The next possible step is merge/update behavior for existing contracts.

This document defines the guardrails for that step before any write path is implemented.

## Goal

Reduce adoption friction for repos that already have a partial `ota.yaml` without weakening trust in
`ota detect`.

## Non-goals

- silent mutation of an existing contract
- overwriting human-authored values without review
- broad reformatting of the contract
- changing fields outside detect-owned surfaces
- inventing values not backed by existing detect provenance

## Proposed command shape

The first merge/update surface should be review-first:

```bash
ota detect --merge --dry-run [PATH]
```

Only after that mode is trusted should Ota consider:

```bash
ota detect --merge [PATH]
```

## Merge/update scope

The first merge scope should stay narrow and detect-owned:

- `project.name`
- `runtimes.*`
- `tools.*`
- `services.*.{provider,start,stop,healthcheck}`
- `tasks.*.run`

Do not merge:

- `agent`
- `metadata`
- `env`
- `checks`
- `execution`
- task descriptions, categories, dependencies, or variants

Those fields are too human-authored or too semantically loaded for an early merge path.

## Merge policy

For each detected field:

- if the field is missing in `ota.yaml`, classify it as `add`
- if the field exists and differs, classify it as `update`
- if the field exists and matches, classify it as `unchanged`

The dry-run surface may omit `unchanged` fields from the main summary, but the merge engine must
still compute them deterministically.

## Write policy

The first write policy should be conservative:

- apply only `high` confidence adds automatically
- do not apply `medium` or `low` confidence fields
- do not auto-apply any conflicting updates at first
- conflicting updates must remain preview-only until proven safe

That means the first merge write, if shipped, is really an additive fill-in mode, not a true
overwrite mode.

## Conflict policy

A conflict is any detected field where:

- the existing contract already has a value
- the detected value differs

Conflicts must:

- be shown explicitly in dry-run output
- include existing and detected values
- never be silently written in the first merge implementation

## Output requirements

`ota detect --merge --dry-run` should report:

- candidate contract fragment or effective diff summary
- per-field status: `add`, `update`, or `unchanged`
- provenance
- confidence
- clear distinction between write-eligible and preview-only fields

## Trust gate for merge/update

Before any merge write mode ships:

1. fixture coverage must include existing-contract update cases
2. dry-run comparison output must be stable and reviewable
3. add-only write behavior must pass `ota validate`
4. conflicting updates must remain non-destructive
5. no human-authored field outside detect-owned scope may be rewritten

## Why this boundary

Ota should help a repo adopt the contract faster, but it must not become a surprising config
rewriter.

The right sequence is:

1. detect
2. compare
3. additive merge only
4. only later consider carefully gated conflict-resolution behavior
