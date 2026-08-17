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

# Execution Governance Capability Reference

This page is the canonical map for Ota's major execution-governance capabilities.

Use it to find the focused operator reference for one job. It is not a second command or contract
reference: exact field, command, and JSON semantics remain in the linked specifications.

## Capability map

| Capability | Operator question | Status | Public reference |
| --- | --- | --- | --- |
| Safe agent execution and refusal | Which declared task or workflow closure may an agent execute, and how does Ota prove refusal still works? | Released | [Safe Agent Execution and Refusal](https://ota.run/docs/reference/safe-agent-execution-and-refusal) |
| Contract-to-CI governance | How does contract-owned verification become a stable merge check without making provider workflow YAML a second authority? | Released | [Contract-to-CI Governance](https://ota.run/docs/reference/contract-to-ci-governance) |
| Sandbox policy and runtime enforcement | Which filesystem and network controls were compiled, applied, inspected, and cleaned up by a compatible provider? | V11.21 complete on the `1.6.26-implementation` branch; not yet released | [Sandbox Policy and Runtime Enforcement](https://ota.run/docs/reference/sandbox-policy-and-runtime-enforcement) |
| Proof evidence and honest boundaries | What did the selected proof establish, and what explicitly remains `not_proved`? | Released | [Proof Evidence and Honest Boundaries](https://ota.run/docs/reference/proof-evidence-and-honest-boundaries) |
| Replay inputs and trusted baselines | Which immutable inputs and explicitly promoted generated artifacts governed replay? | Released | [Replay Inputs and Trusted Baselines](https://ota.run/docs/reference/replay-inputs-and-trusted-baselines) |
| Contract-claim assurance | Is a maintainer-authored safety or proof-breadth claim supported, contradicted, or still unknown? | Released | [Contract-Claim Assurance](https://ota.run/docs/reference/contract-claim-assurance) |
| Audited execution boundary crossings | When a heavier non-agent lane is crossed, what was authorized, why, by which bounded authority, and what actually executed? | Bounded V11.7 OSS slice complete on the `1.6.26-implementation` branch; not yet released | [Audited Execution Boundary Crossings](https://ota.run/docs/reference/audited-execution-boundary-crossings) |
| Semantic snapshots and correlation | What semantic contract truth governed a run, what changed, and which change may relate to a new failure? | Released | [Semantic Snapshots and Correlation](https://ota.run/docs/reference/semantic-snapshots-and-correlation) |

## One architecture, separate claims

- `ota.yaml` declares repo-owned execution truth.
- runner admission decides whether the selected closure may start.
- provider application proves only controls the provider actually applied and Ota inspected.
- receipts and proof artifacts record execution evidence without becoming reusable authority.
- claim assurance evaluates observable support without inventing missing maintainer intent.
- semantic snapshots and diffs explain contract change without collapsing into runtime evidence.
- CI and external harnesses consume canonical Ota governance truth; they do not redefine it.

## Boundary rules

- Ota cannot constrain raw shell outside an adopted runner, sandbox, or merge chokepoint.
- A green selected lane does not prove the whole repository.
- `ok: true` is scoped execution success, not automatic application correctness.
- `unknown` and `not_proved` are deliberate outcomes, not weaker spellings of success.
- A crossing record is execution evidence, never reusable approval authority.
- A signed grant must come from a trust root the repository and caller cannot self-issue.
- Provider-owned triggers, credentials, deployment policy, and branch protection remain
  provider-owned even when Ota projects the required verification lane.

## Canonical supporting specifications

- [Command reference](command-reference.md)
- [Contract reference](contract-reference.md)
- [JSON output reference](json-output-reference.md)
- [Execution receipt](execution-receipt.md)
- [Execution governance loop](execution-governance-loop.md)
- [Runtime proof evidence](../planning/v11.11/plan.md)
- [Contract-claim assurance](../planning/v11.14/plan.md)
- [Trusted replay baseline regeneration](../planning/v11.17/plan.md)
- [Policy-governed replay-input identity](../planning/v11.20/plan.md)
- [Enforced sandbox policy application](../planning/v11.21/plan.md)
- [Audited execution boundary crossings](../planning/v11.7/plan.md)
