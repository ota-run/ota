<!--
               █████
              ░░███
      ██████  ███████    ██████
     ███░░███░░░███░    ░░░░░███
    ░███ ░███  ░███      ███████
    ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
     ░░░░░░     ░░░░░░   ░░░░░░░░

  Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

  Do NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

  Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
  You may not use this file except in compliance with that License.
  Unless required by applicable law or agreed to in writing, software distributed under the
  License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
  either express or implied. See the License for the specific language governing permissions
  and limitations under the License.

  If you need additional information or have any questions, please email: os@ota.run
-->

# Ota Studio User Journeys

Status: planned.

Studio UX should be validated through explicit journeys before implementation.

Each journey is measurable by screen state + server payload checkpoints.

## Journey 1: first run with a repo that already has contract

Goal:
- user opens repo and gets a trustworthy immediate state.

Expected steps:
1. Launch `ota studio` from repo root.
2. Studio opens, auto-registers repo.
3. Overview shows:
   - repo identity
   - doctor status
   - latest activity
4. User opens Topology and Contract without page reload.
5. User opens Run / Evidence and launches `doctor`.
6. Result appears with receipt path and timeline.

Acceptance:
- no ambiguous spinner-only moments longer than 2s without status text
- action request payload is shown in preview before run

## Journey 2: first run with no `ota.yaml`

Goal:
- keep Studio helpful and non-blocking.

Expected steps:
1. Launch `ota studio` from repo without contract.
2. Home/Overview shows "no contract" banner and inference option.
3. Draft opens with inferred startup proposal.
4. User can read proposal with confidence tiers.
5. User chooses contract action and is guided to review path.

Acceptance:
- no raw error dump
- no claim of successful contract state when none exists

## Journey 3: stale draft review recovery

Goal:
- prevent unsafe writes from stale context.

Expected steps:
1. User opens Draft and reviews proposed changes.
2. Background file changes invalidate draft freshness.
3. Studio surfaces stale warning in Contract and Draft.
4. Run / Evidence action is disabled.
5. User triggers refresh, then review re-validates.
6. Action enabled only after refreshed state passes.

Acceptance:
- stale transitions are explicit
- stale action attempt must not emit mutation

## Journey 4: run task from Studio while terminal also active

Goal:
- demonstrate cross-process visibility and continuity.

Expected steps:
1. Terminal runs `ota run typecheck` in same repo.
2. User opens Studio and visits Run / Evidence.
3. Studio shows the active or recent operation with operation id.
4. Timeline updates while operation is running (or shows last completed state).
5. User can inspect logs and receipt details.

Acceptance:
- the operation is visible even when started outside Studio (requires operation_id propagation in result/receipts)

## Journey 5: switching repos quickly

Goal:
- make local Studio durable across multiple repos.

Expected steps:
1. Open Repo A from terminal.
2. Switch to Repo B from repo rail.
3. State model and shell route persists only UI context that is route-safe.
4. Recent actions are shown per selected repo.

Acceptance:
- no cross-repo contamination of contract data
- last-open metadata updates in registry

## Journey 6: blocked action

Goal:
- avoid unsafe or unsupported actions.

Expected steps:
1. User attempts stateful action from Run / Evidence.
2. Studio requires preview with exact context flags.
3. If blocked by policy/safety check, action shows blocker reason.
4. User follows suggested corrective action (doctor / detect / config path).

Acceptance:
- zero hidden mutations
- reason is action-specific and actionable

## Journey 7: recovery after failed action

Goal:
- keep operational confidence after failure.

Expected steps:
1. user launches `up` and gets failed execution.
2. Failure origin is visible (operation kind + phase + message).
3. Recovery suggestions appear with copy that maps to next command/doc.
4. Logs/receipt are one click away.

Acceptance:
- failure is never a dead-end
- recovery CTA is present and specific

## QA journey matrix

For each journey, validate:
- route model consistency
- action preview correctness
- stale/blocked state handling
- evidence trace presence
- no client-side semantic inference

## Negative journeys

- repository disappears after registration
- registry read-only filesystem
- malformed API payload shape
- operation endpoint returns 4xx

Negative outcomes should show:
- root-cause
- explicit next step
- link/path to stable recovery entry points
