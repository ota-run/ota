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

Status: planned. This follows active V11.15 under version discipline only. Managed GitHub
projection is not an architectural prerequisite: V11.16 derives execution-boundary truth from the
selected runtime and existing canonical producer/admission evidence, whether the lane is invoked
locally, in CI, or through a future generated workflow.

## Problem

A contract can declare setup accurately while a run succeeds only because ambient state already
exists: a warmed virtual environment, downloaded model, inherited environment variable, running
service, or hand-applied migration. A fresh container can still mount a persistent cache or use an
existing database, so one boundary-wide label cannot honestly describe every material dependency.

A green task must not imply that the selected setup path created every prerequisite this run used.
Likewise, an ordinary native host run must not be described as a cold start merely because Ota did
not observe pre-existing state.

## Product Principle

Materialize first, then probe. A setup or runtime assertion may rely only on state the selected
execution closure created or explicitly inherited. Each material prerequisite has runner-derived
boundary evidence; the human-facing freshness summary is a strict function of that evidence, not
maintainer prose or a caller-controlled environment signal.

## Scope

V11.16 adds one receipt/proof-owned `execution_boundary` record. Its authoritative truth is a
per-prerequisite `prerequisites[]` evidence set, not a single global state:

- `id` and `class`: stable prerequisite identity and one of `filesystem`, `dependency_cache`,
  `service`, `volume`, `environment`, or `image_runtime`;
- `boundary`: selected boundary kind, lifecycle, and isolation posture used when deriving the
  summary;
- `selected_by`: the selected task/workflow and producer closure that made the prerequisite
  material to this proof;
- `state`: `created_this_run`, `declared_immutable_input`, `verified_reused`, or `unknown`;
- `basis`: the runner-derived producer/admission basis, or a verified boundary-attestation
  reference;
- `ambient_boundary`: an explicit unresolved class when the state is `unknown`.

`execution_boundary.summary` is strictly derived from `prerequisites[]`:

- `cold_start_verified` only when the selected boundary is isolated and every material
  prerequisite is `created_this_run` or `declared_immutable_input`;
- `persistent_state_reused` when any material prerequisite is `verified_reused`;
- `unknown` otherwise.

An immutable container image or pinned toolchain is a legitimate `declared_immutable_input`; it
need not be recreated by setup, but must carry the immutable identity Ota evaluated. A verified
reused cache, volume, service, or environment blocks a cold-start summary even when its reuse is
intentional.

## Evidence And Attestation Rules

- `provider_ephemeral_runner` is not automatically cold proof. A provider assertion can support a
  prerequisite only through a verified boundary attestation containing its issuer kind and immutable
  issuer identity, current run identity, selected-scope identity, payload digest, verification
  result, and `evidence_class: attested`.
- A caller-controlled signal such as `CI=true`, a workflow label, or free-text provider claim never
  promotes a prerequisite beyond `unknown`.
- Native host execution defaults material host, environment, cache, and service prerequisites to
  `unknown` unless Ota has stronger runner-owned evidence.
- A persistent attachment, volume, cache, or already-running service is `verified_reused` only
  when Ota can identify and verify the selected reused boundary; otherwise it remains `unknown`.
- A required repo-owned executable, generated config, or declared setup artifact must consume the
  existing canonical producer-before-probe admission result. V11.16 must close only remaining
  selected runtime paths; it must not create proof-specific ordering logic parallel to Doctor's
  shipped producer/admission evaluator.
- Missing attestation identity, a scope mismatch, invalid verification, or an unclassified material
  prerequisite yields `unknown`; no partial attestation can be summarized as cold proof.

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

The JSON schema must require `execution_boundary.prerequisites[]`, each prerequisite state and
basis, and the derived `execution_boundary.summary`. It must reject a cold summary when a material
prerequisite is `verified_reused` or `unknown`, and reject an `attested` basis without the complete
attestation identity/verification record. The archive preserves byte-identical boundary evidence.

Human proof output must render the derived summary and every unresolved ambient boundary beside the
terminal proof verdict. A green proof with `unknown` material state must remain visibly qualified.

## Implementation Order

1. Define runner-owned prerequisite evidence types and derive their selected material set from
   execution scope, lifecycle, existing producer/admission results, attachments, services, and
   immutable contract inputs.
2. Reuse canonical producer-before-probe admission evidence on all remaining selected native and
   container runtime paths; add only the missing integration coverage.
3. Define verified boundary-attestation parsing and rejection, including issuer, run, scope, digest,
   and verification binding.
4. Emit the per-prerequisite record and derived summary in runtime-proof JSON, archive it with the
   proof, and render it in human proof output.
5. Add schema and regression fixtures for all-created/immutable cold proof, mixed persistent reuse,
   host unknown, forged or stale provider attestation, and rejected premature probing.
6. Pressure-test Lead Quorum's repo-local virtualenv lane and Athena's ephemeral container app plus
   host-managed PostgreSQL runtime lane on clean CI. The latter must remain mixed/qualified unless
   Ota can identify a selected service boundary honestly.

## Non-Goals

- No claim that Ota can prove a physical host has never been used before.
- No automatic deletion of user state, caches, services, volumes, or credentials to manufacture a
  cold run.
- No arbitrary process sandboxing or new environment-manager DSL.
- No rewriting of setup commands that Ota cannot model truthfully.

## Acceptance Bar

V11.16 is complete when:

- every selected material prerequisite has exactly one state and basis in JSON;
- `cold_start_verified` is emitted only for an isolated boundary whose complete selected
  prerequisite set is `created_this_run` or `declared_immutable_input`;
- a mixed-state fixture with a reused cache, volume, or service derives
  `persistent_state_reused`, never cold proof;
- a forged, stale, scope-mismatched, or caller-controlled provider assertion remains `unknown` and
  cannot produce `evidence_class: attested`;
- no selected repo-owned prerequisite is probed before its canonical producer closure succeeds;
- proof archives preserve the exact prerequisite and attestation evidence that supported the
  derived summary, and schema conformance covers live JSON and archived proof JSON;
- human and JSON output keep a green proof from over-reading unresolved ambient state;
- Lead Quorum and Athena pressure the native producer and container/service boundaries without
  repo-local shell workarounds.
