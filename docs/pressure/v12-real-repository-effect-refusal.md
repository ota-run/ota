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

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
-->

# V12 Real-Repository Effect-Refusal Pressure

Status: bounded immutable pre-release pressure complete; independent V12 closure review pending.
This note does not itself close V12, assert maintainer adoption, or claim that either upstream
repository is governed by Ota.

## Evidence Matrix

| Repository | Upstream revision | Static-control fork / matrix | Archive-candidate fork / matrix |
| --- | --- | --- | --- |
| [Plausible Analytics](https://github.com/plausible/analytics) | `1cdff8b2d1e310cee731ac648b45875ec1fbd131` | `a3d9ca9ea3d92a0b1a3c8588436076c734665b74` / [33271676531](https://github.com/bobaikato/analytics/actions/runs/33271676531) | `fa24db238dae39a277e5fbfc08519488a32c1020` / [33391482073](https://github.com/bobaikato/analytics/actions/runs/33391482073) |
| [Outline](https://github.com/outline/outline) | `d8d8f2fffed97eaf3e9d48d820c8f1e1facf4008` | `c8c821ced9b7f14b7fdffbc2c737165e21b79653` / [33271681072](https://github.com/bobaikato/outline/actions/runs/33271681072) | `58b6a7731aff1a1237da1d9ade6021114b0a1c6e` / [33391486538](https://github.com/bobaikato/outline/actions/runs/33391486538) |

Both fork-only matrices source-build Core `84a988433cb3c0226a3569cdc2ee5202d3d5d375` and
complete on Linux and macOS. They retain contract validation, Doctor coverage output, direct task
and workflow-canary JSON, and one rejected generic caller-selected command for the exact fork
revision.

The final archive-candidate and selected-path witness matrices source-build Core
`e96cad13db9e4289c0985fca2ce6d8353a896da4` on Linux and macOS. Each retained artifact includes
`ota-version.json`, binding `v1.6.27`, that exact commit, `source_build: true`, and `dirty: false`
before it validates the contract or creates local Ota state. Plausible jobs
[`99485902001`](https://github.com/bobaikato/analytics/actions/runs/33391482073/job/99485902001)
and [`99485901821`](https://github.com/bobaikato/analytics/actions/runs/33391482073/job/99485901821)
retain artifacts `9757737269` and `9757842630`; Outline jobs
[`99485914019`](https://github.com/bobaikato/outline/actions/runs/33391486538/job/99485914019)
and [`99485913737`](https://github.com/bobaikato/outline/actions/runs/33391486538/job/99485913737)
retain artifacts `9757768136` and `9757797795`.

## Observed Controls

Plausible binds `priv/repo/migrations` and its committed Elixir CI migration prerequisite.
Outline binds `server/migrations` and its committed Sequelize server-test CI migration prerequisite.
For each repository, the retained task and workflow records show:

- `status: passed` for the effect-refusal canary;
- `actual_decision: deny` through one explicit typed policy rule;
- `execution_started: false`;
- distinct task and workflow selected-invocation identities; and
- the exact declared `ci_schema_migration` effect reference.

The retained `tasks.json` in every final artifact binds the selected migration task to one
provider/database precursor, one worktree/child-command precursor, and one `after_always` hook.
The selected workflow additionally binds one setup task. The task canary artifacts record all
three task-closure sentinels absent immediately after refusal. The workflow canary artifacts record
those three plus the setup sentinel absent immediately after refusal. The captured closure contains
no selected service, recorded as `no_selected_service_declared`; this is closure evidence, not a
fabricated service sentinel.

Both Doctor outputs report `effect_refusal_assurance.status: unknown`, one intentionally
unchallenged exact-equivalent attachment as `not_proved`, and the fixed opaque-execution-path gap.
The generic caller-selected command returns `not_evaluated`, never a passed canary. These artifacts
prove Ota's execution-free refusal and static coverage-honesty behavior at the selected
typed-effect boundary only.

## Archive, Assurance, And Candidate Reconciliation

Each archive-candidate matrix creates one workflow-scoped private refusal archive through the
selected committed migration lane. History first reports `1` valid archive and `0` invalid
archives. The matching workflow-only `ci_schema_archive_refusal` claim is then `supported` from
that verified archive; removing its retained archive context changes history to `0` valid and `1`
invalid and returns that claim to `unknown`. The original task-and-workflow canary remains
`unknown`, because it is not the exact workflow-only subject supported by the archive.

Both matrices then derive one schema-v5 `unknown` candidate with no application projection, reject
`apply-candidate --write --carrier git` as `candidate_read_only`, and emit a
reconciliation-bound `already_declared` no-op for the predeclared workflow-only canary. Changing
the selected migration bytes returns `effect_refusal_candidate_stale`; replacing `ota.yaml` with a
symlink returns `effect_refusal_candidate_failed`. Neither condition publishes a candidate.

Plausible's overall Doctor envelope remains `not_ready` in these hosted jobs because `mix` is not
available on the runner, while the exact archived effect-refusal claim is `supported`. Outline's
overall Doctor envelope remains `risky` because local Ota artifacts are not ignored. Those are
separate readiness findings, not evidence of repository-wide readiness or a qualification of the
archive reconciliation result.

## Internal Mixed-Realization Control

Separate immutable internal pressure in [run 33300446201](https://github.com/ota-run/ota/actions/runs/33300446201)
binds Core `974caf686a45093587058ea140b82f1a81c0fa70` on Linux/x64 and macOS. Its selected closure
contains an eligible `declared_and_typed` migration realization and an ineligible `declared_only`
realization for the same effect. Both artifacts retain a blocked preview, one shared effect identity,
distinct attachment and realization identities, absent command sentinels, and an
`effect_canary_realization_ineligible` assurance gap with `execution_started: false` for the
declared-only origin. It is a synthetic, provider-disabled control and does not extend the
real-repository claims above.

## Internal Namespace And Policy-Source Control

Separate immutable internal pressure in [run 33301627289](https://github.com/ota-run/ota/actions/runs/33301627289)
binds Core `e7682a62287b173edaa8e2a18f57fc1593359dec` on Linux/x64 and macOS. It declares two
identical local migration sets against resource bindings that differ only by canonical namespace
account. Both artifacts retain distinct effect and attachment identities, an exact primary-only
deny while the secondary remains compatibility `warn`, and validation refusal for an empty
namespace authority. The same policy bytes loaded from the repository versus `OTA_POLICY` retain
one policy snapshot identity but distinct repository-controlled and caller-selected source-evidence
and decision identities. A create-new refusal receipt archived under the caller-selected posture is
valid (`1` valid, `0` invalid); rewriting that receipt to repository-controlled is invalid (`0`,
`1`); restoring its exact bytes returns it to valid (`1`, `0`). This is synthetic,
provider-disabled source-provenance evidence only. It does not attest a provider, independent
policy administration, arbitrary process absence, or real-repository behavior.

## Internal CI Provider-Checkout Re-evaluation Control

Separate immutable internal pressure in [run 33302123045](https://github.com/ota-run/ota/actions/runs/33302123045)
binds Core `d0178b2013efd3d12f6baa0a94bb572f162c70a7` on Linux/x64 and macOS. It creates the
compatibility-policy projection identity that a rendered provider workflow would carry, changes the
checkout policy to an exact typed deny, then executes the same
`ota ci projection --expect-identity <rendered-identity>` command. Both artifacts retain a changed
projection identity, `effect_policy_denied`, explicit typed deny, repository-controlled source
posture, and absent fixture setup and durable-log paths. The same jobs also pass Core's
plan-to-executor source/plan/materialized-input substitution regression. This verifies Ota's local
provider-checkout re-evaluation boundary only; it does not prove third-party provider execution,
contact, mutation, arbitrary child-process absence, or real-repository coverage.

## Internal Sandbox Capability Control

Separate immutable internal pressure in [run 33303689321](https://github.com/ota-run/ota/actions/runs/33303689321)
binds Core `49a1a486a4431749ff33ec50ea4265afbc2a64f2` on Linux/x64 and macOS. Both retained artifacts
show a typed-deny task and workflow with refused preflight and `provider_execution: disabled`, and a
typed-warn task that remains refused as provider-disabled rather than becoming callable. The same
jobs pass Core's task/workflow retained command-admission sandbox control. This proves only Ota's
local capability projection and command-admission reuse; it does not establish authoritative
sandbox enforcement, provider contact, provider mutation, arbitrary child-process absence, or
real-repository coverage.

## Unproved Boundaries

- provider contact, authority, re-evaluation, or mutation;
- database correctness, migration success, rollback, or data integrity;
- arbitrary child-process absence or complete repository immutability;
- independently administered policy authority;
- positive effect receipt/archive, export, or assurance; and
- complete coverage of either repository's migration, deployment, or raw-shell paths.

## V12 Pressure Reconciliation

The V12 pressure set now covers external selected-lane refusal, static coverage honesty, private
archive integrity, exact workflow-only assurance promotion and fallback, review-only candidate
reconciliation, and bounded witnesses for each applicable selected local mutation precursor in
both repositories. Retained closure evidence establishes that neither selected workflow declares a
service. It does not prove provider contact, provider mutation, arbitrary child-process absence,
repository-wide immutability, database correctness, independently administered policy, positive
assurance, archive export safety, or full repository coverage. The bounded implementation and
pressure requirements are complete; one independent closure review must reconcile the complete
evidence set before V12 is marked complete.
