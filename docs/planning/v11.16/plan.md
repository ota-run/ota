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

# V11.16: Fresh-Boundary Setup Proof

Status: planned. This follows active V11.15. Do not begin implementation until managed GitHub
projection has one renderer shared by render, check, and sync.

## Problem

A contract can declare setup accurately while a run succeeds only because ambient state already
exists: a warmed virtual environment, downloaded model, inherited environment variable, running
service, or hand-applied migration. Probing a repo-owned executable before its declared setup
closure materializes it is the same failure class.

A green task must not imply that the selected setup path created every prerequisite this run used.
Likewise, an ordinary native host run must not be described as a cold start merely because Ota did
not observe pre-existing state.

## Product Principle

Materialize first, then probe. A setup or runtime assertion may rely only on state the selected
execution closure created or explicitly inherited. Freshness is runner-derived execution truth,
not maintainer prose.

## Scope

V11.16 adds one receipt/proof-owned execution-boundary record. The first shape distinguishes:

- `boundary_kind`: `ephemeral_container`, `provider_ephemeral_runner`, `persistent_context`, or
  `host_unknown`;
- `freshness`: `cold_start_verified`, `persistent_state_reused`, or `unknown`;
- `setup_materialization`: selected prerequisite outputs created this run, reused intentionally,
  or not observed;
- `probe_admission`: whether a repo-owned prerequisite was probed only after its selected producer
  closure succeeded;
- `ambient_state_boundaries[]`: named unresolved state classes such as `host_environment`,
  `preexisting_service`, or `persistent_dependency_cache`.

The record is runner-derived or boundary-attested. It does not become caller assertion merely
because the contract selects a container mode.

## Truth Rules

- `cold_start_verified` requires a runner-owned isolated boundary plus evidence that the selected
  setup closure materialized the prerequisites it later probed or executed.
- `provider_ephemeral_runner` is not automatically cold proof. Ota may report the provider fact,
  but must retain `unknown` freshness unless it can establish the relevant filesystem, service, and
  cache boundary honestly.
- Native host execution defaults to `host_unknown`; Ota must not infer absence of ambient state.
- Persistent lifecycle execution must report `persistent_state_reused` whenever selected paths use
  a known persistent context, volume, cache, or already-running service.
- A required repo-owned executable, generated config, or declared setup artifact may not be probed
  before its selected producer closure succeeds. Failure to establish that ordering is an explicit
  preflight boundary, not a best-effort probe.

## Contract Boundary

Do not add a broad `fresh` or `cold_start` authoring flag. Existing contract truth already owns
producer tasks, artifact lineage, execution lifecycle, shared backends, attachments, and runtime
paths. V11.16 derives the execution-boundary record from that canonical truth.

A later policy slice may require cold-start verification for selected lanes. This slice only emits
honest evidence and preserves existing execution behavior by default.

## First Carrier

The first carrier is `ota proof runtime --json`, because it already owns selected runtime proof and
cleanup. Its archived proof record must preserve the same execution-boundary record. Receipts may
receive an additive projection later; ordinary `ota run` success must not claim runtime freshness
it did not prove.

Human proof output must render the freshness state and every unresolved ambient boundary beside the
terminal proof verdict. A green proof with `host_unknown` must remain visibly qualified.

## Implementation Order

1. Define runner-owned execution-boundary types and derive them from selected execution scope,
   lifecycle, producer closure, and observed persistent attachments/services.
2. Enforce producer-before-probe admission for repo-owned executables and generated artifacts on
   the selected native and container paths.
3. Emit the record in runtime-proof JSON, archive it with the proof, and render it in human proof
   output.
4. Add fixtures for cold ephemeral materialization, host-unknown execution, persistent reuse, and
   rejected premature probing.
5. Pressure-test a repo-local virtualenv lane and a container-backed setup lane on clean CI.

## Non-Goals

- No claim that Ota can prove a physical host has never been used before.
- No automatic deletion of user state, caches, services, volumes, or credentials to manufacture a
  cold run.
- No arbitrary process sandboxing or new environment-manager DSL.
- No rewriting of setup commands that Ota cannot model truthfully.

## Acceptance Bar

V11.16 is complete when:

- runtime proof distinguishes verified cold start, known persistent reuse, and unknown host state;
- no selected repo-owned prerequisite is probed before its declared producer closure succeeds;
- proof archives preserve the exact boundary evidence that supported the freshness result;
- human and JSON output keep a green proof from over-reading unresolved ambient state;
- native and container fixtures cover the same prerequisite ordering rule;
- one real Python/virtualenv repo and one container-backed repo pressure-test the model without
  repo-local shell workarounds.
