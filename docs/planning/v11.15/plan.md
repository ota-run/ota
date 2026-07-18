<!--
                █████
               ░░███
       ██████  ███████    ██████
      ███░░███░░░███░    ░░░░░███
     ░███ ░███  ░███      ███████
     ░░██████   ░░█████ ░░████████
      ░░░░░░     ░░░░░   ░░░░░░░░

   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

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

# V11.15: Managed GitHub Actions Governance Projection

Status: active. V11.14 assurance/policy truth is complete and pressure-proven. Implement this
slice without reopening its canonical evaluator or policy semantics.

V11.5 defines the required foundation: required-lane projection, stable `merge_check_id` identity,
merge-gate output, and contract-to-CI drift evaluation. Those semantics must be complete before
V11.15 implementation begins. V11.15 does not reopen them; it adds one narrow GitHub Actions
adapter that materializes the canonical V11.5 result as a managed reusable workflow.

## Problem

GitHub Actions workflows commonly duplicate Ota bootstrap, verification, and merge-gate truth.
Detection and drift review can identify that duplication, but they do not yet make the contract the
deterministic source of a maintained provider projection.

The product must avoid replacing two manually maintained sources with ambiguous bidirectional
synchronization. Existing workflow YAML is useful onboarding evidence; it is not a competing
runtime authority after an Ota contract has been reviewed.

## Product Principle

`ota.yaml` is canonical for Ota-owned execution truth. GitHub Actions is a provider adapter.

The first implementation owns only a dedicated generated reusable workflow. A small human-owned
caller workflow invokes it and retains provider policy. Ota must never rewrite arbitrary workflow
YAML, infer permissions, mutate deployment jobs, or silently take ownership of provider policy.

## Synchronization Model

V11.15 uses **bidirectional discovery with one-way authority**:

- existing GitHub workflow truth may be inspected by `ota detect` and surfaced as
  provenance-bearing, reviewable contract evidence or a contract-change candidate;
- only reviewed `ota.yaml` truth may render or sync Ota-owned GitHub workflow content;
- changes to human-owned workflow content produce drift findings or new review evidence, never
  silent reverse synchronization into the contract;
- conflicts between contract truth and provider truth require explicit review. Ota must not choose
  an authority heuristically.

This gives teams a migration path from existing workflows without preserving two execution
authorities after adoption.

## Authority Boundary

Ota owns the generated reusable governance workflow:

- Ota bootstrap source through `agent.bootstrap.ota.source`
- selected contract workflows/tasks and their execution mode
- required verification lanes, proof lanes, refusal canaries, and `merge_check_id` identity
- contract-to-provider drift verdicts for managed projection content

The human-owned GitHub caller workflow owns:

- triggers, branch filters, concurrency, permissions, secrets, environments, and runners
- deployment, release, notification, and provider-specific jobs
- non-Ota jobs and steps outside its explicit reusable-workflow call

The caller is not a second execution contract. It declares only provider scheduling and invokes
the generated workflow by its exact semantic projection identity. `ota ci github check` verifies
that relationship; it does not infer contract commands from the caller.

## First Contract Shape

Do not add a second workflow declaration language. V11.15 derives projection from existing
contract workflow, task, policy, V11.5 merge-check, V11.11 proof, and V11.14 assurance truth.

The first renderer may accept only explicit selection options such as provider, contract workflow,
generated workflow path, caller workflow path, and runner image. Those options configure the
adapter; they must not restate bootstrap or verification commands already declared in the contract.

## Command Surface

The first provider-specific surface is intentionally narrow:

```text
ota ci github render --workflow verify --output .github/workflows/ota-governance.yml
ota ci github check --workflow verify \
  --output .github/workflows/ota-governance.yml \
  --caller .github/workflows/ci.yml
ota ci github sync --workflow verify \
  --output .github/workflows/ota-governance.yml \
  --caller .github/workflows/ci.yml
```

- `render`, `check`, and `sync` all consume one canonical renderer and semantic projection model.
- `render` is pure and deterministic: it writes to stdout by default and never mutates files.
- `check` compares the rendered reusable workflow with the managed file, verifies the caller's
  exact reusable-workflow reference, and fails on stale or manually changed Ota-owned content.
- `sync` is explicit, atomic, idempotent, and writes only a file bearing Ota's ownership marker;
  it never mutates the human-owned caller.
- all three emit stable JSON with projection identity, selected contract lane, output identity,
  merge-check mappings, and drift/refusal detail.

The generated workflow must consume contract truth through released `ota-run/setup` and Ota CLI
commands. It must not duplicate install scripts or verification shell commands.

## Generated Reusable Workflow Contract

The first generated workflow is invoked through `workflow_call` and contains only:

1. checkout;
2. `ota-run/setup` with `source: contract`;
3. contract-owned `ota up` / `ota run` / proof or refusal-canary lanes;
4. provider check naming derived from canonical V11.5 `merge_check_id` values;
5. Ota JSON/annotation reporting.

The human-owned caller supplies `on`, permissions, concurrency, environments, and its non-Ota
jobs. It invokes the generated reusable workflow by a marker-backed semantic projection identity.
Generated content must include a versioned ownership marker and semantic projection identity.
`check` compares semantic projection content and the caller's reference, not whitespace or provider
reformatting alone.

## Safety And Assurance Admission

V11.15 must consume, not recreate:

- V11.3 agent closure enforcement;
- V11.4 governance verdicts;
- V11.5 required-lane and merge-check identities;
- V11.11 proof breadth and `not_proved` boundaries;
- V11.14 claim-assurance policy decisions.

If a required lane is denied, unknown under strict assurance policy, stale, or insufficiently
proved, rendering/checking must report the canonical reason rather than emit a green wrapper job.

## Non-Goals

- No bidirectional synchronization from arbitrary GitHub workflow edits into `ota.yaml`.
- No full caller workflow generation in the first cut.
- No generated deployment, release, secret, permission, trigger, concurrency, or environment
  policy.
- No GitHub App or autonomous pull-request mutation.
- No provider-neutral CI DSL beyond the existing contract.

## Implementation Order

1. Define one provider-projection domain, canonical renderer, stable managed-file marker, and
   semantic identity.
2. Implement deterministic GitHub reusable-workflow rendering for one selected contract workflow.
3. Map required lanes to existing V11.5 `merge_check_id` values without name heuristics.
4. Implement `check` against the same renderer: verify managed-file ownership, semantic content,
   and the human caller's exact reference.
5. Implement explicit atomic `sync` against the same renderer; refuse unmarked, externally owned,
   or caller paths.
6. Carry V11.3/V11.11/V11.14 blocking decisions into generated output and JSON.
7. Add `ota-run/action` integration only as a consumer of `check`, never as parallel drift logic.
8. Pressure-test `render`, `check`, and `sync` together on generated and mixed handwritten
   workflow repositories before any GitHub App work.

## Acceptance Bar

V11.15 is complete when:

- a reviewed contract produces deterministic reusable GitHub Actions YAML for one contract-owned
  verification lane;
- existing workflow truth can enter through detection as reviewable evidence without mutating the
  reviewed contract or becoming a second execution authority;
- generated workflow bootstrap and task/workflow commands are derived from contract truth, not
  duplicated shell strings;
- stable `merge_check_id` values map directly to generated provider checks;
- `check` fails on changed contract truth, stale generated content, manual modification of an
  Ota-owned projection, or a caller that does not invoke the expected projection identity;
- `sync` is atomic and idempotent, writes only the marked reusable workflow, and refuses to
  overwrite unowned workflow or caller files;
- generated lanes preserve V11.3 refusal enforcement, V11.11 proof boundaries, and V11.14 policy
  decisions instead of reporting a flattened green CI result;
- JSON distinguishes provider-owned configuration from Ota-owned projection content;
- tests prove `render`, `check`, and `sync` share the same semantic projection; cover managed
  ownership, caller reference, stale projection, policy denial, proof-boundary propagation, and
  merge-check identity;
- pressure tests prove one generated reusable workflow plus caller and one repo with mixed
  handwritten deployment/provider jobs.

## Pressure Targets

Start with Kylrix: it already exposes contract-to-CI bootstrap drift and advertises native plus
container verification. Its existing CI workflow is the human-owned caller. The second target must
keep substantial handwritten deployment or release jobs, proving Ota's projection remains an
adapter rather than a workflow replacement.
