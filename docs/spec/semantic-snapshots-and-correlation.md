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

# Semantic Snapshots and Correlation

Status: shipped.

Public operator reference:
[`Semantic Snapshots and Correlation`](https://ota.run/docs/reference/semantic-snapshots-and-correlation)

This document defines the current local-core contract for:

- archived semantic contract snapshots
- canonical assumption-set identity
- `ota receipt --snapshot`
- `ota diff` against current or archived semantic truth
- receipt-to-receipt semantic drift correlation

## Why this exists

Ota should answer one question honestly:

`what changed in repo truth, and did that explain the failure?`

That requires three separate but connected surfaces:

1. semantic contract snapshot identity
2. semantic contract diff
3. receipt correlation between introduced failures and changed assumptions

Ota keeps those surfaces separate on purpose.

- snapshot identity is contract truth
- receipt evidence is execution truth
- correlation is advisory, not claimed causality

## Snapshot identity

Every successful receipt now carries additive semantic identity:

- `receipt.contract_snapshot_hash`
- `receipt.assumption_set_hash`

When `--archive` is used, Ota also archives the normalized semantic contract JSON under
`.ota/contracts/sha256-....json`.

These identities serve different purposes:

- `contract_snapshot_hash`: whole normalized contract snapshot identity
- `assumption_set_hash`: canonical extracted assumption-map identity

Use `assumption_set_hash` when you want to know whether semantic assumptions changed even if other
snapshot structure stayed stable.

## Snapshot inspection

Use `ota receipt --snapshot` to read archived semantic contract truth directly.

Examples:

```text
ota receipt --snapshot latest
ota receipt --snapshot promoted
ota receipt --snapshot ./.ota/contracts/sha256-....json
ota receipt --snapshot ./.ota/receipts/repo-receipt-....json
```

This stays read-only.

It exists so operators and agents can inspect archived contract truth without routing that work
through diff or receipt correlation first.

## Semantic diff

`ota diff` compares normalized semantic contract truth, not raw YAML layout.

Current shipped behavior:

- compare repo or workspace contracts semantically
- accept archived receipt JSON or archived `.ota/contracts/...` snapshot JSON as diff inputs
- emit normalized `changes[]` with additive `category` and `risk`
- keep deterministic text and JSON ordering
- stay read-only

Use `ota diff` when you want:

- branch-to-branch contract review
- current-vs-archived semantic comparison
- a stable machine-readable assumption diff

## Receipt correlation

`ota receipt --json --baseline` can compare the current receipt against a promoted baseline, latest
archived receipt, or explicit archived receipt file.

When the selected baseline carries archived semantic snapshot truth, the compare surface also emits:

- `contract_changes[]`
- `likely_related_changes[]`
- `summary.comparison.contract_snapshot_changed`
- `summary.comparison.correlation`

Correlation is advisory and honest:

- `likely_related`: Ota found specific semantic contract changes that plausibly explain newly
  introduced blocker findings
- `possibly_related`: Ota could not recover a sharp direct match, but the newly introduced blockers
  and semantic drift still overlap in the same declared contract family
- `no_clear_correlation`: Ota could not honestly recover even that coarse overlap

## Correlation ordering

When `likely_related_changes[]` is present, ordering is intentional.

Ota prefers the sharpest declared semantic evidence it can recover, in this order:

1. exact declared owner truth
2. declared owner subtree drift
3. named references
4. broader same-family overlap

Examples of stronger declared evidence:

- reusable `surfaces.<name>`
- reusable `readiness.probes.<name>`
- task or workflow requirement references
- runtime surface definitions attached to the selected path

This means Ota should publish a reusable top-level surface owner ahead of a weaker workflow surface
reference when both are plausible for the same failure.

## Non-goals

- YAML formatting diff
- Git history as the source of semantic truth
- receipt evidence and contract diff collapsed into one artifact
- fake causality claims
- hidden mutation or auto-repair

## Relationship to adjacent commands

- `ota receipt --snapshot`: read archived semantic truth
- `ota diff`: compare semantic truth
- `ota receipt --json --baseline`: compare receipt evidence plus semantic contract drift
- `ota doctor`: diagnose current readiness
- `ota explain`: produce remediation ordering from readiness findings

## Machine-readable contract

For precise JSON fields, see:

- [json-output-reference.md](json-output-reference.md)
- [execution-receipt.md](execution-receipt.md)
- [semantic-diff-and-explain.md](semantic-diff-and-explain.md)
