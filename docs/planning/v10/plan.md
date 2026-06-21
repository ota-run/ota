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

# V10 Plan

Status: planned.

Release target:

- first implementation target: `ota 1.6.22`

Source direction:

- [Semantic diff and explain](../../spec/semantic-diff-and-explain.md)
- [Execution receipt](../../spec/execution-receipt.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Trust and governance hardening](../v9.1/trust-and-governance-hardening.md)

V10 theme:

- semantic contract snapshots
- assumption diffs
- failure correlation

This is the next post-`1.6.21` implementation slice.

The goal is not another contract versioning system. The goal is to make Ota better at answering:

- what changed in declared repo truth
- whether that change is likely related to the failure that followed
- whether the changed field came from manual authorship or a machine-authored path

## Included capabilities

- contract snapshot identity in receipts
- normalized semantic contract snapshot archive
- canonical assumption-key extraction from the parsed contract model
- semantic contract diff as a first-class Ota surface
- receipt-to-receipt contract-change correlation
- honest confidence-based failure correlation

## Non-goals

- do not introduce per-field schema versioning
- do not create a second contract version system inside `ota.yaml`
- do not diff raw YAML formatting, comments, aliases, or key order
- do not make Git the source of truth for contract history
- do not collapse execution evidence, provenance, and contract change into one overloaded receipt
- do not overclaim causality when only weak correlation exists

## Product framing

Retire the framing:

- assumption-level contract versioning

Use the framing:

- semantic contract snapshots
- assumption diffs
- failure correlation

This keeps three separate truths clean:

1. contract truth
2. provenance truth
3. execution truth

Ota can join them when useful without turning them into one noisy blob.

## Core model

### 1. Semantic contract snapshot

Every archived execution receipt should be able to point at the exact normalized contract truth it
ran against.

The snapshot model should:

- normalize parsed contract truth, not raw YAML text
- be stable under formatting-only edits
- be addressable by content hash
- be retrievable from Ota-managed archive state

Minimum first shipped surface:

- `contract_snapshot_hash`
- optional `contract_snapshot_ref`

Later acceptable widening:

- inline normalized snapshot for explicit debug/archive modes

### 2. Assumption index

Ota should derive stable semantic keys from the canonical contract model.

Examples:

- `env.vars.DATABASE_URL.required`
- `services.redis.readiness.kind`
- `tasks.setup.prepare.source.manager`
- `tasks.test.command.exe`
- `workflows.default`
- `agent.safe_tasks`

These keys are semantic contract paths, not YAML line numbers.

### 3. Assumption diff

Ota should compare two normalized snapshots and report:

- added assumptions
- removed assumptions
- changed assumptions

Every diff item should include:

- `key`
- `kind: added | removed | changed`
- `before`
- `after`
- `category`
- `risk`

Planned categories:

- `runtime`
- `toolchain`
- `env`
- `service`
- `task`
- `workflow`
- `execution`
- `agent`
- `policy`
- `topology`

### 4. Failure correlation

When `ota doctor`, `ota up`, `ota run`, or proof fails, Ota should be able to correlate the
failure against recent semantic contract change.

This surface must stay honest and confidence-based:

- `likely_related`
- `possibly_related`
- `no_clear_correlation`

Example:

- failure: missing `DATABASE_URL`
- recent contract change: `env.vars.DATABASE_URL.required: false -> true`
- result: `likely_related`

## Planned command/output surfaces

### Receipts

Planned receipt additions:

```json
{
  "contract_snapshot_hash": "sha256:...",
  "contract_snapshot_ref": ".ota/contracts/sha256-....json",
  "assumption_set_hash": "sha256:..."
}
```

The receipt remains execution evidence first.

It should not inline a full contract snapshot by default in every normal mode.

### `ota diff`

Planned read-only surface:

- widen the shipped semantic diff surface instead of introducing a second competing command
- compare two semantic contract snapshots
- compare two contract files after normalization
- compare a receipt snapshot to current repo truth

Planned JSON shape:

```json
{
  "ok": true,
  "base": "...",
  "target": "...",
  "summary": {
    "added": 1,
    "removed": 0,
    "changed": 2
  },
  "contract_changes": [
    {
      "key": "env.vars.DATABASE_URL.required",
      "kind": "changed",
      "before": false,
      "after": true,
      "category": "env",
      "risk": "high"
    }
  ]
}
```

### Receipt-to-receipt correlation

Planned diff surface:

- compare execution receipts
- surface execution-state changes separately from contract changes
- join them only when useful

Planned joined reporting:

- `Contract Changes`
- `Execution Changes`
- `Likely Related Changes`

### Machine-authored provenance

This remains a separate surface from semantic contract change.

It answers:

- why Ota wrote or promoted a field
- whether the field was manual, detector-owned, promoted, policy-derived, or template-derived

It should continue to build on existing Ota provenance surfaces such as:

- `metadata.ota.detect.field_ownership`
- `metadata.ota.detect.field_admission`
- `owner_kind`

This slice does not collapse provenance and semantic diff into the same structure.

## Rollout order

1. Add normalized contract snapshot identity to receipts.
2. Add normalized snapshot archive and retrieval path.
3. Add canonical assumption extraction and semantic diff engine.
4. Widen shipped `ota diff`.
5. Add receipt-to-receipt correlation.
6. Add confidence-based failure correlation heuristics last.

This order preserves trust before inference.

## Archive layout direction

Planned shape:

- receipts remain under the shipped `.ota/receipts` archive root
- normalized contract snapshots are archived separately
- receipts reference snapshots by hash and optional local ref

Example direction:

- `.ota/receipts/...`
- `.ota/contracts/<hash>.json`

The hash is the identity surface.
The archived normalized snapshot is the inspectability surface.

## Acceptance bar

- Ota can compare contract truth semantically without YAML noise
- receipts can point to exact contract truth used at runtime
- operators can inspect changed semantic contract assumptions directly
- failure correlation stays clearly advisory and confidence-based
- provenance and semantic change stay distinct in both text and JSON output
