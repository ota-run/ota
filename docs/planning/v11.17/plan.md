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

# V11.17: Trusted Replay Baseline Regeneration

Status: active implementation. This follows V11.16 under version discipline. It extends V11.10
replay-input governance and V11.13 producer-owned artifact lineage; it does not reopen their
completed core models. Core validation is in progress; Bedrock promotion pressure requires an
intentional credentialed recording and must not be fabricated from the committed fixture.

## V11.16 Evidence Dependency

V11.17 consumes, but does not reimplement, V11.16's runner-authored prerequisite evidence graph.
A recorded regeneration attestation binds the immutable `execution_boundary_graph_identity` and
its asserted-target and derivation-input closure identities that produced its artifact set. That
establishes the historical derivation posture of the promoted baseline.

A later replay must capture and evaluate its own V11.16 graph for its own execution. It must never
copy, recompute from, or promote the recording graph into a current-run freshness claim. Promotion
therefore preserves provenance of the accepted baseline; it does not certify that a future replay
was cold-start verified, hermetic, or derived under the same conditions.

## Problem

`expected_identity` is appropriate when a contract consumes an independently immutable input. It
cannot by itself prove how a mutable recorded fixture, baseline, or model witness was produced. A
hand-edited digest merely makes changed output look declared; it cannot establish regeneration
provenance or whether the run was allowed to update the replay corpus.

The trustworthy unit of review is therefore not a typed hash. It is an explicit regeneration run,
the generated artifact diff, and an Ota-authored attestation binding those outputs to that run.

## Product Principle

Replay consumes one explicitly promoted frozen artifact set read-only. Only an explicit, opt-in,
contract-declared producer lane may record a candidate. Ota derives identities and provenance from
that lane; reviewers promote an attestation reference and its complete expected identity set, never
an arbitrary digest or the latest successful recording.

## Canonical Boundary

V11.17 extends the existing `artifacts.<name>` producer/consumer lineage instead of creating a
parallel replay-baseline map. The first cut uses the existing one-task producer model; workflows
may orchestrate that task, but are not independent artifact producers.

```yaml
artifacts:
  recorded-baseline:
    kind: replay_baseline
    producer: record:live
    paths:
      - data/fixture.jsonl
      - data/store.db
      - data/baseline.json
    replay:
      authority_manifest: replay/recorded-baseline.ota.json
      consumption: read_only
```

The contract declares the producer, output boundary, and portable authority-manifest location. It
does not contain a mutable hand-authored digest. The generated, committed manifest selects the
accepted attestation and identities.

The authority chain is:

```text
producer run -> recorded attestation -> explicit promotion -> immutable selected reference -> read-only replay
```

The resulting rules are:

- a replay baseline is a named artifact with one declared producer task;
- the producer is explicitly classified as a regeneration lane and is never selected by ordinary
  replay, default verification, or agent-safe execution;
- replay consumers declare the baseline through existing artifact/input ownership and may read it,
  but cannot rewrite it;
- Ota captures the produced path identities only after the declared producer succeeds and records
  one immutable, content-addressed regeneration attestation;
- the attestation, not a contract literal, binds artifact name, output paths and SHA-256 identities,
  source identity, contract snapshot, selected lane and mode, execution receipt/proof identity, and
  creation time, V11.16 `execution_boundary_graph_identity`, asserted-target closure identity, and
  derivation-input closure identity;
- an explicit promotion operation selects one recorded attestation and atomically writes the
  portable authority manifest. It may only promote a complete, scope-matching recorded attestation;
  it never chooses the newest successful recording implicitly;
- an attestation begins `recorded`; the authority manifest may explicitly select it as `promoted`
  or later mark that selection `revoked`. Replay accepts only a promoted, non-revoked attestation
  identity and its expected output identities;
- a replay artifact is trusted only when its current identities match that selected manifest.
  Otherwise it is unavailable or drifted, never silently re-baselined.

`expected_identity` remains additive for inputs whose identity must be independently pinned before
execution. Promotion supplies the same pinning function for generated baselines by copying Ota's
attested output identities into the authority manifest. Ota must never auto-update either an
`expected_identity` value or a promoted authority manifest.

## Regeneration Admission

The first implementation must make regeneration intentional and auditable:

- the contract names the sole producer and its artifact outputs; Ota rejects ambiguous producers or
  output overlap;
- the regeneration command requires explicit opt-in. It must not run from `ota up`, ordinary
  replay, or a default task dependency;
- the lane carries its actual effects and external-state posture. A live-model or live-service
  recording lane remains unsafe for agent mode unless separately admitted through the existing
  crossing/policy model;
- Ota captures output identities after successful production and records the attestation atomically.
  Promotion is a separately explicit operation that writes only the contract-declared portable
  authority manifest; it never changes contract text;
- an interrupted, partial, unverified, or scope-mismatched producer cannot publish an attestation;
- strict replay executes against a runner-owned copy or read-only mount of the promoted artifact
  set. Ota refuses strict replay when its selected backend cannot enforce that boundary;
- non-strict native replay may detect a post-run mutation, but must report
  `replay_artifact_mutation_detected` rather than claiming that the write was refused.

The first OSS cut does not infer that a human review occurred. It makes the generated attestation
and artifact diff reviewable. Later policy may require approval of a regeneration attestation for
security-sensitive baselines without changing the producer or replay semantics.

## First Carrier

The portable authority is the contract-declared, committed authority manifest. It contains the
selected promoted attestation identity, complete canonical output manifest, promotion identity,
and revocation state, so a fresh clone or CI runner can verify replay without local receipt history.
An enterprise Evidence Service may later provide an immutable remote equivalent, but local receipt
archives are supporting evidence only.

The canonical recorded carrier is a `regeneration_attestation` attached to the producer's archived
execution receipt and referenced by the authority manifest and replay receipt/proof output. It
includes:

- stable artifact identity and complete output identity set;
- producer task, source identity, semantic contract snapshot, execution boundary, and
  receipt/proof identity;
- the immutable V11.16 execution-boundary graph identity plus asserted-target and derivation-input
  closure identities for the producer run;
- attestation digest and creation time;
- `evidence_class: attested` for the Ota-authored record;
- current replay comparison: `matched`, `drifted`, or `unavailable`, with the promoted selected
  attestation identity.

The canonical output manifest is recursively deterministic: normalized repo-relative paths sorted
bytewise; file SHA-256 and executable mode; symlink target identity without traversal; and explicit
entries for absent, added, or deleted declared outputs. Unsupported special files fail production
and promotion rather than yielding a platform-dependent manifest.

Human output must distinguish `recorded by regeneration` from `verified on replay`; neither may
collapse into a generic green result. JSON schema and archive schema share the same record; later
Doctor and policy surfaces derive from it rather than calculating a second baseline identity.

## Implementation Order

1. Extend V11.13 artifact lineage with `kind: replay_baseline`, one task producer, declared
   outputs, portable authority-manifest location, explicit consumer ownership, and no ordinary
   execution path to regeneration.
2. Add explicit regeneration admission and atomic Ota-authored recorded-attestation capture at the
   producer decision site, binding the producer run's V11.16 graph identity plus asserted-target
   and derivation-input closure identities.
3. Add an explicit promotion operation that validates recorded scope and complete identity before
   atomically writing the portable manifest; implement recorded, promoted, and revoked state.
4. Bind replay input resolution to the manifest's promoted immutable attestation and use a
   runner-owned copy or read-only mount for strict replay. Capture a separate V11.16 graph for the
   replay run; do not inherit the recorded baseline's freshness or derivation posture.
5. Emit the same promotion and attestation references in archived receipts, replay proof JSON, and
   human output; add schema conformance for every carrier.
6. Add fixtures for valid regeneration and promotion, newest-recording-not-promoted, revoked
   attestation, manual artifact edit, typed digest with no provenance, ambiguous producer,
   interrupted producer, strict replay write refusal, and native mutation detection.
7. Pressure-test Bedrock's explicit live recording lane against its frozen fixture, SQLite store,
   and baseline, then independently pressure one non-model generated-baseline repository.

## Non-Goals

- No automatic artifact regeneration, digest update, baseline promotion, or contract rewrite.
- No claim that ordinary code review or an external approval occurred.
- No approval workflow in the first OSS cut; policy may consume the attestation later.
- No replacement of `expected_identity` for externally immutable inputs.
- No provider-specific model replay or hidden instrumentation beyond a repo's declared producer.

## Acceptance Bar

V11.17 is complete when:

- contract validation rejects ambiguous baseline producer/consumer lineage, workflow-only producer
  ownership, missing portable authority-manifest location, and an ordinary replay path that can
  select a regeneration lane;
- a successful explicit regeneration produces one Ota-authored, content-addressed `recorded`
  attestation bound to the canonical output manifest, source, contract snapshot, scope, and
  archived receipt/proof, including the producer run's immutable V11.16 graph and selected
  asserted-target and derivation-input closure identities;
- only an explicit promotion can select a recorded attestation. Replay never selects newest, and a
  revoked or unpromoted attestation is unavailable;
- a fresh clone or CI runner can verify the promoted artifact identity from the committed authority
  manifest without local receipt history;
- strict replay uses a runner-owned copy or read-only mount. A backend unable to enforce that
  boundary refuses strict replay; non-strict mutation is explicitly detected and reported;
- replay output records its own V11.16 graph separately from the recorded baseline's graph; no
  promoted baseline can upgrade the current replay to `cold_start_verified` or otherwise inherit
  historical derivation posture;
- no command automatically updates a contract digest, promotion manifest, attestation, or baseline
  artifact during replay;
- JSON and archive schemas require the same recorded-attestation, promotion, canonical output
  manifest, and comparison identities;
- Bedrock proves explicit recording followed by read-only deterministic replay, and an independent
  generated-baseline repo proves this is not model-specific.

## Acknowledgment

This slice was sharpened through discovery feedback from
[Vinicius Pereira](https://github.com/vinimabreu): review the explicit recording that produced a
baseline, not a hand-entered hash that can merely make a changed output appear declared.
