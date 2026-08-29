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
| [Plausible Analytics](https://github.com/plausible/analytics) | `1cdff8b2d1e310cee731ac648b45875ec1fbd131` | `0f3ab55002240e7cbbb15aa867b292cdc6d84c66` | [33263551116](https://github.com/bobaikato/analytics/actions/runs/33263551116) |
| [Outline](https://github.com/outline/outline) | `d8d8f2fffed97eaf3e9d48d820c8f1e1facf4008` | `20f576532d027039a280727fe6e26b6e1cc798ad` | [33263549770](https://github.com/bobaikato/outline/actions/runs/33263549770) |

Both fork-only matrices source-build Core `cd99c9abd2c0225b454371e897eca2486319db26` and
complete on Linux and macOS. They retain contract validation plus direct task and workflow-canary
JSON for the exact fork revision.

## Observed Controls

Plausible binds `priv/repo/migrations` and its committed Elixir CI migration prerequisite.
Outline binds `server/migrations` and its committed Sequelize server-test CI migration prerequisite.
For each repository, the retained task and workflow records show:

- `status: passed` for the effect-refusal canary;
- `actual_decision: deny` through one explicit typed policy rule;
- `execution_started: false`;
- distinct task and workflow selected-invocation identities; and
- the exact declared `ci_schema_migration` effect reference.

The workflows do not start PostgreSQL, Mix, Yarn, Sequelize, or a provider. They prove Ota's
execution-free refusal at the selected typed-effect boundary only.

## Unproved Boundaries

- provider contact, authority, re-evaluation, or mutation;
- database correctness, migration success, rollback, or data integrity;
- arbitrary child-process absence or complete repository immutability;
- independently administered policy authority;
- positive receipt, archive, export, or assurance; and
- complete coverage of either repository's migration, deployment, or raw-shell paths.

## Remaining V12 Pressure

The V12 pressure bar remains open. The next external control must prove coverage honesty: an
intentionally unchallenged equal-effect path reports `equivalent_execution_paths_not_proved`, an
omitted or obscured path remains unknown or an assurance gap, and a generic earlier refusal cannot
false-green the canary. Mixed-realization, namespace/posture substitution, CI provider-checkout,
sandbox capability, refusal receipt/archive, and V11.14 assurance pressure also remain required.
