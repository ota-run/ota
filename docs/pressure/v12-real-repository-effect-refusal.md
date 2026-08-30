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

Status: partial immutable pre-release pressure. This note does not close V12, assert maintainer
adoption, or claim that either upstream repository is governed by Ota.

## Evidence Matrix

| Repository | Upstream revision | Fork revision | Hosted matrix |
| --- | --- | --- | --- |
| [Plausible Analytics](https://github.com/plausible/analytics) | `1cdff8b2d1e310cee731ac648b45875ec1fbd131` | `a3d9ca9ea3d92a0b1a3c8588436076c734665b74` | [33271676531](https://github.com/bobaikato/analytics/actions/runs/33271676531) |
| [Outline](https://github.com/outline/outline) | `d8d8f2fffed97eaf3e9d48d820c8f1e1facf4008` | `c8c821ced9b7f14b7fdffbc2c737165e21b79653` | [33271681072](https://github.com/bobaikato/outline/actions/runs/33271681072) |

Both fork-only matrices source-build Core `84a988433cb3c0226a3569cdc2ee5202d3d5d375` and
complete on Linux and macOS. They retain contract validation, Doctor coverage output, direct task
and workflow-canary JSON, and one rejected generic caller-selected command for the exact fork
revision.

## Observed Controls

Plausible binds `priv/repo/migrations` and its committed Elixir CI migration prerequisite.
Outline binds `server/migrations` and its committed Sequelize server-test CI migration prerequisite.
For each repository, the retained task and workflow records show:

- `status: passed` for the effect-refusal canary;
- `actual_decision: deny` through one explicit typed policy rule;
- `execution_started: false`;
- distinct task and workflow selected-invocation identities; and
- the exact declared `ci_schema_migration` effect reference.

Both Doctor outputs report `effect_refusal_assurance.status: unknown`, one intentionally
unchallenged exact-equivalent attachment as `not_proved`, and the fixed opaque-execution-path gap.
The generic caller-selected command returns `not_evaluated`, never a passed canary. These artifacts
prove Ota's execution-free refusal and static coverage-honesty behavior at the selected
typed-effect boundary only.

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

## Unproved Boundaries

- provider contact, authority, re-evaluation, or mutation;
- database correctness, migration success, rollback, or data integrity;
- arbitrary child-process absence or complete repository immutability;
- independently administered policy authority;
- positive receipt, archive, export, or assurance; and
- complete coverage of either repository's migration, deployment, or raw-shell paths.

## Remaining V12 Pressure

The V12 pressure bar remains open. This matrix closes only the external static
coverage-honesty control: an intentionally unchallenged equal-effect path reports
`equivalent_execution_paths_not_proved`, opaque execution remains a gap, and a generic caller
refusal cannot false-green the canary. The separate internal mixed-realization and
namespace/policy-source and CI provider-checkout controls are complete; sandbox capability,
refusal receipt/archive beyond source-posture reconciliation, and V11.14 assurance pressure also
remain required.
