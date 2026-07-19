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

Status: active. The shared semantic evaluator and additive `ota proof runtime --json` carrier are
implemented; they conservatively inventory declared generated filesystem targets as `unknown`
until the runner can witness their complete precondition, materialization, and assertion chain.
Managed GitHub projection is not an architectural prerequisite: V11.16 derives execution-boundary
truth from the selected runtime and existing canonical producer/admission evidence, whether the
lane is invoked locally, in CI, or through a future generated workflow.

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
execution closure created or explicitly inherited. Ota records runner-authored prerequisite
provenance, not a maintainer-authored cold-start claim. Each material prerequisite has
runner-derived boundary evidence; the human-facing freshness summary is a strict function of that
evidence, not maintainer prose or a caller-controlled environment signal.

## Scope

V11.16 adds one receipt/proof-owned `execution_boundary` record. Its authoritative truth is a
runner-authored prerequisite evidence graph with two explicit closures, not a single global state
or an unordered digest list:

- `asserted_target_closure`: mutable and immutable targets whose state the selected proof asserts;
- `derivation_input_closure`: upstream inputs used to materialize those targets, including caches.

Only the asserted target closure can determine target freshness. The derivation-input closure
determines derivation posture and must never turn a freshly rebuilt target into persistent state.

For an ephemeral selected closure, the runner owns one run-scoped execution session for every
compatible container boundary. Dependency producers and consumers may share that session only when
their resolved image, context, lifecycle, isolation mounts, network, publication shape, and
execution identity match. Ota creates it before the first compatible closure step and destroys it
only after the requested closure terminates. A later CLI invocation is a distinct run and never
inherits that session. Incompatible boundaries remain isolated and must not be silently merged.

The graph has stable nodes for prerequisites, producer executions, consumer executions,
boundaries, and immutable inputs. It has typed edges:

- `produced`: a producer execution materialized a prerequisite identity;
- `asserted_at`: a real consumer execution had an asserted prerequisite identity available at its
  execution boundary;
- `consumed`: adapter-instrumented evidence proves the consumer actually used that prerequisite;
- `derived_from`: one prerequisite identity was derived from another input;
- `reused_from`: a prerequisite was reconstructed with a persistent cache or boundary input;
- `cleared_before`: Ota witnessed an empty or cleared mutable target before this producer ran.

Every edge carries the execution ID, boundary ID, prerequisite ID, selected task/workflow scope,
producer or consumer identity where applicable, evidence class, typed observed identity, and a
runner-owned monotonic `sequence`. Wall-clock time is diagnostic only: `sequence` and graph
causality decide which producer established the identity available to, or consumed by, each
assertion.

`prerequisites[]` is a derived, readable projection over the asserted-target closure, with linked
derivation-input edges retained in the graph.
For each prerequisite it records:

- `id` and `class`: stable prerequisite identity and one of `filesystem`, `dependency_cache`,
  `service`, `volume`, `environment`, or `image_runtime`;
- `precondition`: `absent`, `present`, `cleared`, or `unknown`, with the runner observation that
  established it;
- ordered `materializations[]`: producer execution, created identity, evidence class, and
  sequence;
- ordered `assertions[]`: consumer execution, recovered identity, evidence class, sequence, and
  `established_by_edge_id` pointing to the causally matching producer edge for that assertion;
- optional `terminal_established_by_edge_id`: the final producer for display only; it cannot replace
  per-assertion causal bindings when a prerequisite is changed and asserted again;
- `state`: `created_this_run`, `declared_immutable_input`, `verified_reused`, or `unknown`;
- `ambient_boundary`: an explicit unresolved class when the state is `unknown`.

The first cut is intentionally narrow: filesystem artifacts such as `.venv`, `node_modules`,
generated SDKs, and rendered configuration receive full graph evidence. Services, databases,
volumes, and provider-managed runtime state remain `unknown` unless a shipped adapter can provide
the same trustworthy boundary and identity evidence.

`execution_boundary.target_freshness` is strictly derived from the asserted target closure only:

- `cold_start_verified` only when every mutable prerequisite was runner-observed absent or cleared
  at the selected boundary, materialized by a producer in this run, and asserted with the matching
  identity; every immutable asserted target must be declared and identity-verified; no asserted
  target prerequisite may be `verified_reused` or `unknown`;
- `persistent_state_reused` when any asserted target prerequisite is `verified_reused`;
- `unknown` otherwise.

`execution_boundary.derivation_posture` is independent from freshness. It must preserve both:

- `materialization`: `fully_derived`, `cache_assisted`, or `unknown`;
- `immutable_inputs`: `none`, `inherited_immutable`, or `unknown`.

A clean target reconstructed from a verified download cache may therefore be
`cold_start_verified` and `cache_assisted`: the cache is in the derivation-input closure, while the
new target is in the asserted target closure. An immutable container image or pinned toolchain is a
legitimate `inherited_immutable` input; it need not be recreated by setup, but its declared
identity must be verified. This split prevents a fast cache from pretending to be hermetic while
also preventing legitimate immutable inputs from being treated as unexplained ambient state.

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
- A consumer assertion is authoritative only when it is emitted from the real selected consumer
  execution and its recovered identity is causally bound through that assertion's
  `established_by_edge_id`. A post-run hash or path check may emit `asserted_at`; it cannot emit
  `consumed` or promote a prerequisite to `created_this_run` without stronger runner or adapter
  evidence.
- Multiple producers preserve ordered mutation provenance. Ota must emit the producer that
  established the final asserted identity, not merely an unordered list of matching producers.
- A required repo-owned executable, generated config, or declared setup artifact must consume the
  existing canonical producer-before-probe admission result. V11.16 must close only remaining
  selected runtime paths; it must not create proof-specific ordering logic parallel to Doctor's
  shipped producer/admission evaluator.
- An ephemeral dependency producer and its compatible consumer must execute in the same
  runner-owned session. It is not sufficient that each task independently receives an ephemeral
  container from the same image: unmounted package caches, tool activation, and process-local
  materialization would otherwise disappear between closure steps.
- Missing attestation identity, a scope mismatch, invalid verification, or an unclassified material
  prerequisite yields `unknown`; no partial attestation can be summarized as cold proof.

## Contract Boundary

Do not add a broad `fresh` or `cold_start` authoring flag. Existing contract truth already owns
producer tasks, artifact lineage, execution lifecycle, shared backends, attachments, and runtime
paths. V11.16 derives the execution-boundary record from that canonical truth.

V11.17 promotion and replay must reference the immutable V11.16 graph identity plus both selected
closure identities. It must not create a parallel freshness model, auto-select the newest graph, or
treat a typed expected digest as a substitute for runner-authored materialization provenance.

A later policy slice may require cold-start verification for selected lanes. This slice only emits
honest evidence and preserves existing execution behavior by default.

## First Carrier

The first carrier is `ota proof runtime --json`, because it already owns selected runtime proof and
cleanup. Its archived proof record must preserve the same execution-boundary record. Receipts may
receive an additive projection later; ordinary `ota run` success must not claim runtime freshness
it did not prove.

JSON Schema enforces the record structure: graph identity, both selected closures, nodes, ordered
edges, `prerequisites[]`, `target_freshness`, and `derivation_posture`. The shared semantic
execution-boundary evaluator enforces cross-record truth before serialization and archive: closure
membership, edge ordering, producer-to-assertion identity matching, per-assertion causal bindings,
freshness derivation, and attestation verification. It rejects dishonest graph states rather than
leaving those invariants to JSON Schema or output producers. The archive preserves byte-identical
boundary evidence.

Human proof output must render the derived summary and every unresolved ambient boundary beside the
terminal proof verdict. A green proof with `unknown` material state must remain visibly qualified.

## Implementation Order

1. Define runner-owned graph nodes, ordered edges, asserted-target and derivation-input closures,
   and canonical graph identity from execution scope, lifecycle, existing producer/admission
   results, attachments, services, and immutable contract inputs. Hash canonical JSON containing
   only semantic graph fields: schema version, normalized repo-relative identities, both sorted
   closures, nodes sorted by stable node ID, and edges sorted by `sequence` then stable edge ID.
   The semantic evaluator rejects duplicate edge IDs and ambiguous sequence/order relationships.
   Exclude diagnostic timestamps and presentation-only fields.
2. Implement filesystem prerequisite evidence first: precondition observation, producer
   materialization, `asserted_at` observation, optional instrumented `consumed` evidence,
   per-assertion causal producer binding, and typed identity recovery for `.venv`, `node_modules`,
   generated SDKs, and rendered configuration.
3. Reuse canonical producer-before-probe admission evidence on all remaining selected native and
   container runtime paths; add only the missing integration coverage.
4. Add compatible ephemeral-closure session ownership to the runner. Keep session identity and
   cleanup runner-authored, reuse only matching boundaries, and reject any attempt to cross an
   incompatible context or lifecycle implicitly.
5. Define verified boundary-attestation parsing and rejection, including issuer, run, scope, digest,
   and verification binding.
6. Implement the shared semantic evaluator and reject invalid cross-graph relationships before
   JSON serialization or archive.
7. Emit the graph, readable prerequisite projection, and derived verdicts in runtime-proof JSON,
   archive them with the proof, and render them in human proof output.
8. Add schema and semantic regression fixtures for ordered multi-producer mutation with two
   assertions, all-created/immutable cold proof, cold cache-assisted reconstruction, mixed
   persistent reuse, host unknown, forged or stale provider attestation, and rejected premature
   probing.
9. Pressure-test Lead Quorum's repo-local virtualenv lane, OrchardCore's typed .NET restore plus
   build/test closure in one ephemeral container session, and Athena's ephemeral container app plus
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

- asserted-target and derivation-input closures, graph nodes, graph edges, canonical graph identity,
  and readable prerequisite projection are schema-valid and pass the shared semantic evaluator;
- every asserted target prerequisite has a precondition observation, ordered producer and
  `asserted_at`/`consumed` evidence where applicable, and one derived state;
- `cold_start_verified` is emitted only when every mutable prerequisite was observed absent or
  cleared, materialized in this run, and asserted with a matching final-producer identity, while
  every immutable asserted target is declared and identity-verified;
- a mixed-state fixture with a reused cache, volume, or service derives
  `persistent_state_reused`, never cold proof;
- a cache-assisted reconstruction can remain `cold_start_verified` because its reused cache is in
  the derivation-input closure, while its independent derivation posture remains `cache_assisted`;
- a forged, stale, scope-mismatched, or caller-controlled provider assertion remains `unknown` and
  cannot produce `evidence_class: attested`;
- an unordered or stale producer record cannot establish a consumer assertion; a multi-producer,
  two-assertion fixture proves each assertion binds its own matching producer edge by runner-owned
  sequence and causality;
- no selected repo-owned prerequisite is probed before its canonical producer closure succeeds;
- a typed dependency hydration producer followed by a compatible finite consumer uses one
  run-scoped ephemeral session; a container-only OrchardCore restore/build/test fixture proves
  package materialization survives the closure and is removed after it;
- proof archives preserve the exact graph, prerequisite, and attestation evidence that supported
  the derived verdicts, and schema conformance covers live JSON and archived proof JSON;
- human and JSON output keep a green proof from over-reading unresolved ambient state;
- Lead Quorum and Athena pressure the native producer and container/service boundaries without
  repo-local shell workarounds.
