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

# Roadmap

## Source Of Truth

[`docs/ai/current-state.md`](docs/ai/current-state.md) is the live handoff for the active branch,
verified proof, and next gate. The active version's detailed scope and activation records live in
[`docs/planning/v12.1/plan.md`](docs/planning/v12.1/plan.md). This roadmap ranks work; it does not
activate a planned slice or turn a successful narrow pressure run into a support claim.

## Current State

- `v1.6.28` is the released baseline. Earlier completed slices retain their exact closure evidence
  in their version plans and the live handoff.
- V12.1 Step 7 is the sole active implementation slice. Its earlier provider-free hosted-evidence
  custody gate is closed only for the recorded Core, Launcher, Protocol, and runner revisions.
- The later public-evidence-root correction needs a fresh protected Linux/X64 run for its paired
  Core and Launcher revisions. GitHub-hosted Launcher jobs do not replace that proof.
- Real GitHub OIDC contact, Google STS/WIF, Secret Manager access, materialization, injection,
  selected-work execution, and positive provider evidence remain unproved. V12.1 Steps 8-10 and
  V12.2 or later versions remain inactive.
- `1.6.29-implementation` is a working branch and candidate patch-release train, not an activation
  record or a promise that V12.1 will be complete before the patch release.

## Current Focus

1. Reconcile the exact Core, Launcher, and Protocol revisions and complete the fresh protected
   Linux/X64 public-root proof. Preserve the prior custody record as historical evidence; do not
   treat it as proof for a changed installation path.
2. Continue the separately activated Step 7 provider-contact sequence only through its reviewed
   proof gates. A provider-free refusal must not be described as provider delivery.
3. Keep design-partner pressure and narrow product fixes moving without promoting draft PRs or
   green fork matrices into upstream adoption or repository-wide governance.
4. Decide the final 1.6.29 contents from completed, independently reviewed behavior and explicit
   proof limits. Do not hold a bounded patch release hostage to unfinished V12.1 Step 8-10 work.

## Active Version

- [V12.1 secret-delivery governance](docs/planning/v12.1/plan.md): Step 7 only.

## Planned, Inactive Work

- [V12.2 contract-authored crossing requirements](docs/planning/v12.2/plan.md) and later versions
  retain their own activation gates; V12.1 progress does not activate them.
- [Execution-contract follow-ons](docs/planning/execution-contract-follow-ons/plan.md) retain the
  lock-strict Cargo, mixed-mode selection, Doctor grouping, opaque-shell, and container-cleanup
  questions. Aggregate selected-graph correctness has already been handled inside V12.1; do not
  reopen it merely because an older pressure workflow still splits its tasks.
- [Design-partner discovery](docs/pressure/design-partner-discovery-2026-09.md) separates current
  evidence from hypotheses such as run-to-measurement binding and tool-version probing. Reproduce
  each candidate before assigning release scope.

Each new sub-slice needs one owner, exact pressure target, compatibility decision, tests,
connected-surface assessment, and independent review. A follow-on may be selected only at a
reviewed between-batch boundary of the active version; this roadmap does not activate a second
version. Prefer at most one or two high-value, reproduced fixes in 1.6.29 and otherwise defer them.

## 1.6.29 Release Decision

The patch release contains only changes that pass their own implementation, compatibility,
pressure, and first-party propagation gates. Its candidate order is:

1. Finish and independently review the paired Core/Launcher public-root correction, including
   fresh protected Linux/X64 proof against exact pinned revisions.
2. Assess separately reviewed Step 7 provider-contact work against its own protected proof bar.
   If incomplete, retain it as gated development work, not released provider capability.
3. Select no more than two independently activated, reproducible adoption fixes from the
   follow-on register. Lock-strict Cargo hydration and mixed-mode preview clarity are candidates,
   not commitments; opaque-shell substep assurance and container cleanup retain their stronger
   prerequisite gates.
4. Freeze exact Core, Launcher, Protocol, Action, Setup, Skills, Examples, and Site revisions;
   reconcile every affected consumer and the changelog; run the selected native, container,
   cross-platform, protected, and release gates; then cut an immutable tag only for what passed.

Before calling the release ready, verify that no post-tag merge is described as part of that tag,
no source SHA is mistaken for installed authority, and public copy distinguishes a narrow
provider-free refusal from provider contact or delivery. The [live handoff](docs/ai/current-state.md)
records the current gate and exact proof links; this roadmap does not replace it.

## Product Direction

Keep the execution contract useful before full adoption: Doctor first, contract second.
Preserve one selected execution graph for humans, CI, and agents; retain explicit proof limits
and the separation between repository truth, protected authority, and observed outcomes.
Future provider, adapter, enterprise, and cross-platform claims activate only through their
own plans and pressure gates.

## Historical Plans

Completed foundations and exact closure records remain in the [V1 phases](docs/planning/v1/phases.md),
[V2](docs/planning/v2/plan.md), [V2.1](docs/planning/v2.1/plan.md),
[V6](docs/planning/v6/plan.md), [V7](docs/planning/v7/plan.md),
[V7.1](docs/planning/v7.1/plan.md), [V7.2](docs/planning/v7.2/plan.md),
[V8](docs/planning/v8/plan.md), [V9](docs/planning/v9/plan.md),
[V11](docs/planning/v11/plan.md), and [V12](docs/planning/v12/plan.md) plans.
The [V1 release gate](docs/planning/v1/release-gate.md) is historical, not a substitute for the
current release contract and first-party matrix.
