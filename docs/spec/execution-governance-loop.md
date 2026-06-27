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

# Execution Governance Loop

Status: shipped.

Public operator reference:
[`Execution Governance Loop`](https://ota.run/docs/reference/execution-governance-loop)

This document defines the current public architecture loop for Ota:

1. contract truth
2. execution truth
3. proof and receipt truth
4. semantic diff and correlation
5. policy and governance truth

Use this page when the question is not only what one command does, but how the shipped Ota surfaces
fit together as one execution-governance system.

## Why this exists

Ota is not only a command runner.

It is a closed execution-governance loop:

- the contract declares what should be true
- execution applies one selected path from that truth
- proof and receipts record what actually happened
- diff and correlation explain what changed in semantic repo truth
- policy constrains what is allowed, approved, or gated

If those layers are taught separately without one loop, operators and agents end up rebuilding the
model from command names, repo scripts, or CI glue.

## The five surfaces

### 1. Contract truth

This is the declared operational truth of the repo or workspace.

Primary surfaces:

- `ota.yaml`
- `ota.workspace.yaml`
- `ota validate`
- contract and workspace reference pages

Contract truth answers:

- what tasks exist
- what setup and runtime paths are declared
- which services, env vars, and requirements matter
- which execution modes are truthful
- which agent boundaries and repo-local policies are declared

### 2. Execution truth

This is the selected operational path Ota will actually use when asked to prepare or run.

Primary surfaces:

- `ota doctor`
- `ota up`
- `ota run`
- `ota execution plan`
- `ota tasks`

Execution truth answers:

- which workflow or task path was selected
- which backend, lifecycle, and context were chosen
- which dependency, setup, or launch closure will execute
- which preconditions or effect gates block that path

### 3. Proof and receipt truth

This is the evidence of what Ota actually executed and what readiness it could prove.

Primary surfaces:

- `ota receipt`
- archived repo or workspace receipts
- runtime proof artifacts

Proof and receipt truth answers:

- what actually ran
- which backend and env sources won
- which steps succeeded, failed, or were blocked
- what readiness or runtime evidence Ota confirmed

### 4. Semantic diff and correlation

This is the semantic repo-truth comparison lane.

Primary surfaces:

- `ota diff`
- `ota receipt --snapshot`
- `ota receipt --json --baseline ...`
- archived semantic contract snapshots

Diff and correlation answer:

- what semantic contract meaning changed
- which assumption set or snapshot identity changed
- whether new blocker findings are likely related to semantic contract drift

### 5. Policy and governance truth

This is the approval, restriction, and organizational overlay lane.

Primary surfaces:

- `.ota/org-policy.yaml`
- `ota policy`
- `ota policy review`
- policy-aware `doctor`, `up`, and `run`

Policy truth answers:

- which versions, provisioning sources, and effects are approved
- which paths are blocked by governance instead of repo-local readiness
- whether the contract and approved org policy still align

## How the loop fits together

The operator path should stay ordered:

1. validate the declared contract truth
2. inspect or execute the selected path
3. read the resulting proof and receipt
4. compare semantic drift when failures or regressions appear
5. reconcile repo truth with policy truth when approval or governance blocks execution

That is why Ota keeps these surfaces separate instead of collapsing them into one receipt or one
giant command.

## Command mapping

Use these commands by the boundary you are actually trying to inspect:

- contract truth: `ota validate`
- execution truth: `ota doctor`, `ota up`, `ota run`, `ota tasks`, `ota execution plan`
- proof and receipt truth: `ota receipt`
- semantic diff and correlation: `ota diff`, `ota receipt --snapshot`, `ota receipt --json --baseline ...`
- policy and governance truth: `ota policy`, `ota policy review`

## What this page is not

- not a replacement for the contract reference
- not a replacement for command reference
- not a single-command tutorial
- not a claim that one receipt should carry the whole governance story

## Relationship to adjacent references

- [contract-reference.md](contract-reference.md) defines repo contract field semantics
- [workspace-reference.md](workspace-reference.md) defines workspace contract field semantics
- [command-reference.md](command-reference.md) defines shipped CLI behavior
- [execution-receipt.md](execution-receipt.md) defines receipt semantics
- [semantic-snapshots-and-correlation.md](semantic-snapshots-and-correlation.md) defines snapshot,
  diff, and correlation semantics
- [policy-packs.md](policy-packs.md) defines org policy pack behavior
