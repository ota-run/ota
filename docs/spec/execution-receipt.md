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

# Execution Receipt

Status: shipped.

This document defines the execution receipt for ota mutation and execution commands.

The goal is to produce a deterministic, machine-readable record of what ota believed,
what it did, and what happened during execution.

## Purpose

The receipt is meant to answer:

- what scope ota used
- what contract version or hash it acted on
- what backend and lifecycle were chosen
- what container image was selected when container execution applies
- what was reused or acquired
- what env or policy affected the decision
- what policy-backed provisioning sources were selected
- what tasks, checks, or services ran
- what failed or was blocked
- what the safe next step is

## Commands

The receipt applies to execution and mutation surfaces such as:

- `ota receipt`
- `ota run`
- `ota up`
- workspace execution flows

It should not replace `doctor`, `detect`, or `init`.

## Planned shape

The receipt includes:

- `ok`
- `path`
- `scope`
- `contract`
- `workspace`
- `backend`
- `lifecycle`
- `image`
- `target`
- `acquired`
- `env`
- `policy`
- `dependency_steps`
- `runtime`
- `workloads`
- `service_termination`
- `host_service_cleanup`
- `execution_conflict`
- `steps`
- `blocked`
- `summary`
- `next`

The current shipped surface records `ok`, `path`, `scope`, `contract`, `backend`,
`lifecycle`, `image`, `target`, `steps`, `blocked`, `summary`, and `next` for `ota receipt`,
`ota run`, `ota up`, and workspace execution flows. Repo receipts now also include a `Policy:`
section when policy-backed provisioning is present, listing the selected source and
any backend-specific source config. When semver-aware policy approval selects an
exact install candidate, that `Policy:` section also shows the requested version,
the resolved install version, and the matched policy rule. On failure, the normal text output keeps `Why`
and `Next` before the trailing summary block. The receipt keeps the structured
summary data for JSON and optional receipt output. For container-backed service tasks, receipts can
also include `service_termination` when a task reached a projected runtime endpoint and then
stopped; this records whether the stop happened after readiness and whether the engine reported an
OOM kill. For host-managed service cleanup owned by ota, receipts can also include
`host_service_cleanup`; each entry records the service name, attempted action, outcome status,
cleanup trigger, and any failure detail ota captured while stopping that host-managed service.
When execution was blocked by an active repo-execution ownership conflict, receipts can also
include `execution_conflict`; this records the typed ownership reasons such as `host_service`,
`compose_project`, or `persistent_backend_family` while keeping the existing `blocked[]`
compatibility lane.
When a receipt comes from a selected task-backed execution path, it can also include
`dependency_steps`; each entry records the executed task step's selected backend, optional
selected context, optional parent task, and `backend_selection_source` such as `override`,
`task default mode`, or `inherited parent backend`.
Each `steps[]` entry also carries additive `stage_family` truth using the execution-governance
families `prepare`, `setup`, `verify`, `proof`, or `receipt`, so machine consumers do not need to
infer broad stage ownership from free-form labels alone.
`target` is only present when the actual recorded execution phase used a real named target.
That includes persistent backends, remote targets, and named ephemeral task or diagnosis
containers. Previews and host-side phases stay targetless.
When `--archive` is set on receipt commands, ota persists the JSON receipt under
`.ota/receipts` so CI or humans can audit the exact execution trail later.
When `--archive --promote-baseline` is set, ota also writes a repo-local promoted baseline
pointer under `.ota/receipts/repo-baseline.json` so later compares can target an explicit,
provider-neutral repo-owned baseline instead of only the latest archived receipt.
`ota receipt --history` is the read-only archive index for those repo receipt files; it lists the
existing archived receipts directly from `.ota/receipts` without rerunning diagnosis, and it
surfaces malformed archive files as skipped entries instead of failing the whole history read.
`ota receipt --baseline` is the first compare surface on top of that archive model; it compares
the current repo receipt against either the promoted repo baseline, the latest valid archived
receipt for the same contract, or an explicit repo receipt JSON file, then classifies findings as
introduced, resolved, or unchanged without rerunning the baseline or writing new archive state.
Diff output now includes baseline provenance such as the selection path, promoted time, and
contract identity when that metadata exists.

## Behavior

- deterministic ordering
- machine-readable first
- human-readable summary in the normal execution output, with failure output keeping `Why` and `Next` ahead of the trailing summary block
- explicit source of truth for decisions
- semantic drift correlation prefers the sharpest declared contract owner or named reference ota can
  recover honestly, such as reusable `surfaces.<name>` or `readiness.probes.<name>`, before broad
  same-family drift
- selected dependency-plane provenance should survive into receipts when ota executed a task-backed
  path, not only into dry-run preview output
- no hidden auto-fix behavior
- provenance for policy-backed provisioning when that layer exists

## Non-goals

- replacing live logs
- turning diagnosis into execution
- hiding failure details behind a generic summary
- introducing opaque background state

## Relationship to other surfaces

- `doctor` diagnoses readiness
- `detect` infers contract data
- `diff` compares contract meaning
- `explain` turns findings into a remediation plan
- `receipt` captures a read-only repo or workspace artifact for the current state
- execution receipts on `run` and `up` record what ota actually executed
