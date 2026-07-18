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

# Current Ota Development State

Update this file at a completed implementation, pressure, release, or handoff boundary. Replace
stale state; do not turn it into an activity log. Durable decisions belong in `docs/adr/` and
durable agent workflow belongs in the canonical Ota skill.

## Active Work

- branch: `1.6.25-implementation`
- released baseline: `v1.6.24`
- active replay-input identity hardening: optional task `replay_inputs[].expected_identity` pins
  now validate canonical SHA-256 values, surface missing or mismatched artifacts through Doctor,
  block dry-run/run/up before task startup, and preserve expected plus observed identity in the
  receipt evaluated-input carrier. Bedrock pressure proves matching frozen inputs through Doctor,
  dry-run, and real native plus container agent-safe execution. Strict-policy admission for lanes
  that require declared pins remains a future policy refinement rather than an implicit default.
- V11.14 contract-claim assurance implementation is complete; pressure and release reconciliation
  remain active. The shared `claim_assurance` domain now supplies the
  first additive `ota doctor --json` carrier for declared agent-safe tasks and workflow proof
  claims. It keeps maintainer
  declaration, derived V11.3 closure, policy-independent assurance, and policy decision separate;
  declaration plus closure remains `unknown` without non-self-origin evidence. Its first
  deterministic contradiction is a typed `reset_compose_service_volume` action that omits the
  exact `effects.adapter_state: compose_volume:<volume>` it mutates; opaque shell remains
  `unknown`. `ota proof runtime --json --archive` now creates a content-addressed proof-owned
  record bound to the terminal proof output, archived contract snapshot, clean source identity
  when available, resolved execution scope, and explicit witness-only replay posture. The shared
  proof-breadth evaluator consumes only a matching immutable archive: matching failed proof is
  cited as `contradicted`; missing, stale, source-mismatched, or scope-mismatched evidence remains
  `unknown`. Ota-owned `.ota` runtime state is excluded from the source-identity cleanliness check,
  so a fresh archive cannot invalidate its own proof claim.
  Opt-in
  `policies.agent.claim_assurance` requirements now drive the same canonical `deny` or `review`
  decision through Doctor, `ota run --agent`, previews, and `ota up --agent`; default agent
  admission remains unchanged without that policy requirement. Generic deterministic workflows
  without a declared dependency seam can opt into this same assurance through
  `workflows.<name>.proof.claim: bounded`; Doctor reports `bounded_proof` as `unknown` until a
  matching immutable archive exists, then `supported`. Bedrock proves that transition on its
  offline replay lane without inventing seam or negative-control evidence; Lead Quorum proves the
  independent `unknown` path without an archive.
- active V11.15 managed CI projection: `ota ci projection --workflow <name> --mode <mode> --target-os <linux|macos|windows> --json`
  now emits the provider-neutral governance lane with a semantic identity, merge-check identities,
  proof requirement, and provider-neutral ownership categories. The GitHub adapter consumes that
  object through one renderer powering `ota ci github render`, `check`, and atomic `sync`.
  It emits separate projection, render, and parsed-caller binding identities; generated content
  runs validation, doctor, safe discovery, agent dry-run, execution, receipt archival, and a
  declared runtime proof when the selected workflow owns one. Agent-safe lanes retain `--agent`;
  proof claims do not bypass agent admission, and proof-required lanes use one authoritative execution.
  Each unique contract refusal canary is now emitted as its own provider check with a stable
  `merge_check_id`; the generated check invokes Ota's `--expect-refusal` runner boundary directly.
  Kylrix renders valid distinct native
  and container reusable lanes without collapsing its separately selected `sqlite-dev` runtime
  proof into `verify`. Strict V11.14 agent and proof assurance admission is evaluated before
  projection render/check; denied or review-required lanes return their canonical refusal rather
  than generating a green wrapper. Remaining V11.15 proof is a committed Kylrix caller/matrix
  pressure pass; do not claim full CI migration until generated lanes preserve the existing
  native/container evidence.
  Projection identity now reuses the canonical normalized semantic snapshot identity used by
  receipts; omitted mode resolves from the selected task's effective contract default, while an
  unavailable explicit mode is refused. Denied provider-neutral JSON preserves the evaluated
  projection and typed refusal. Managed workflow paths reject symlink escape, and the neutral
  projection carries bootstrap posture, proof claim, and target-OS identity for the first GitHub
  adapter.
- V11.3 refusal-canary implementation is pressure-proven on Athena and Kylrix: `agent.refusal_canaries` names
  one task or workflow negative control, and `ota run --agent --expect-refusal <task>` or
  `ota up --agent --expect-refusal --workflow <workflow>` passes only when the agent-safety
  closure refuses before selected work begins. A policy-only denial is the failing
  `wrong_refusal_boundary` outcome. The contract never supplies an expected reason; Ota emits the
  runner-derived refusal and blocked receipt for later comparison. First-party docs/site/skill are
  aligned; do not add it to released examples until `v1.6.25` is available.
- completed `V11.10` replay trust refinement: `ota up
  --replay-baseline ... --json` now carries replay-authored baseline posture directly through
  `replay.baseline.last_known_good`, while declared static replay inputs remain receipt
  `evaluated_inputs[]` and Bedrock-style historical query traces stay separate as attested
  `witnessed_observations.query_traces[]`; plain-text replay output now mirrors the same trust
  split by rendering matched acquitting, narrowing, and pointer-only evidence separately from
  changed inputs, and hermeticity now requires at least one matched material runtime,
  dependency-resolution, or presentation anchor rather than over-reading same-contract reruns as
  hermetic; hidden-input replay failure now emits ordered `hidden_input_candidates` so operators
  can promote the next likely ambient class instead of reading one generic suspicion bucket
- completed V11.12 typed hydration provenance across two ecosystems: successful `ota up --json`
  records selected structured hydration lanes through typed `receipt.evaluated_inputs[]` hydration
  records. The record captures contract-declared source posture and runner-resolved feed identity
  before execution, preserving explicit `resolution: unavailable` when source choice remains
  ambient. Azure SDK for .NET proves config-backed NuGet identity; Lead Quorum proves explicit uv
  PyPI index posture across native and container lanes on Ubuntu, macOS, and Windows. Ota projects
  uv index truth through flags supported by both older and current uv releases. Replay treats
  unavailable hydration resolution as narrowing evidence, never as a hermetic dependency-resolution
  anchor.
- completed V11.9 governance reconciliation: Athena exposed a preview that named
  `not_run_reason: preflight_refusal` while reporting `refusal_occurred: false`. The canonical
  preview now carries the same refusal record, reason, and basis in both phases while preserving
  post-execution `state: not_run` because no execution began.
- completed V11.11 proof-boundary and seam-control carrier: `ota proof runtime --json` publishes
  qualified proof verdicts, scoped `not_proved` boundaries, provenance-aware seam evidence, and
  canonical negative-control records. Athena's Rails/PostgreSQL lane proves transaction-bound
  marker recovery and same-obligation fault control across its green matrix. Every marker-bound
  seam retains `dependency_output_shaping_not_proved`; invariant coverage proves the pairing and
  derived control projection. Generic `ota up` and ordinary receipts intentionally do not inherit
  this evidence because they did not execute the proof lane. The canonical runtime-proof example
  now demonstrates the same carrier end to end. Compose dependencies started by proof are cleaned
  on success and readiness failure, while services already running before proof are preserved.
- completed V11.13 generated-artifact lineage: Dagger proves the generator path and EventCatalog
  proves an independent sibling-consumer closure. Contract-owned producer, output-path, and input
  lineage is validated, surfaced in task discovery, checked before consumer execution, and carried
  into receipts as pointer-only evidence without claiming freshness.
- completed the 1.6.24 release-readiness sweep: the active pressure set has no unresolved Ota
  platform gap, V11.11 proof evidence is propagated through the canonical and public examples,
  skill guidance, site reference, generated contract schema, and changelog, and the complete native
  `release-gate` passes. Lead Quorum's newest local-only pressure commit remains unpushed and is not
  part of the matrix-backed release claim.

- V11.10 Bedrock replay proves native baseline replay as `replay_verified` and `partly_ambient`.
  A container replay against that native archive correctly returns `replay_unavailable` with
  `baseline_scope_mismatch`: workflow, backend, provider, remote target, and lifecycle identity are
  required for `last_known_good`.
  A freshly archived container witness then replays as `replay_verified` and `partly_ambient` on
  the same container/ephemeral scope. Backend-scoped informational doctor notes remain visible but
  do not stale an otherwise same-scope witness.

## Recent Completed Slice

- Kylrix pressure exposed two connected Ota execution gaps and proved their fixes on the
  deterministic SQLite contributor lane. `launch.runtime_projection.adapter: nextjs` now
  projects `--hostname` / `--port` from the runtime listener into direct `next dev` launches,
  while validation rejects package-script wrappers that would make projection ambiguous. Dry-run
  input resolution now recognizes `ensure_env_file` output from the selected dependency closure as
  planned setup state on a clean checkout, while real execution waits for dependencies and then
  validates the rendered dotenv input. Published contract-schema coverage was synchronized for
  command runtime projection and already-shipped generated workflow-instance fields; the full
  examples gate now passes. Kylrix itself proves idempotent SQLite env materialization, agent-safe
  Vitest/lint/build verification, workflow preparation, archived receipt, and isolated native
  runtime proof. Its interactive Appwrite topology and credential/schema-provisioning paths remain
  explicitly outside this narrow proof.

- Kylrix also exposed a native long-running task UX gap: applications such as Next.js can exit
  non-zero after a user `Ctrl+C`. Explicit runner interruption evidence, or a raw signal before a
  clean completion, returns canonical exit `130` with an `interrupted` receipt and summary. A late
  raw signal cannot overwrite an already-established non-interrupt task or service failure.

- Dagger generated-SDK pressure exposed and fixed native source-managed tool activation: a
  release-asset tool was fulfilled and version-probed correctly, but the native shell task path
  discarded the managed PATH before executing its command. Ota now applies the resolved PATH to
  native shell execution and has a focused regression test. The narrowed Dagger contract proves
  release-asset fulfillment, workflow preparation, selected generator execution, generated-source
  lineage, a clean consumer diff, scoped doctor, and archived receipt locally. The selected
  closure requires Dagger v0.21.7 despite root `dagger.json` still naming v0.21.0; that is
  recorded as repo truth rather than hidden by the pressure contract.

- Task platform availability is now contract-owned. `tasks.<name>.only_on` uses the same
  `linux` / `macos` / `windows` vocabulary as prerequisite and context scope; runner planning,
  `ota run`, and dry-run preview refuse an unsupported dependency closure before side effects.
  `ota tasks --use` marks context- or task-unavailable modes non-callable, and doctor filters
  unavailable closures before probing their requirements. Athena pressure uses this truth through
  its Linux/macOS Ruby context and proves the expected Windows refusal rather than hiding it with
  a workflow skip.

- The current V11.10 refinement adds contract-owned
  `tasks.<name>.witnessed_observations.query_traces[]` for existing JSONL query traces. Ota
  validates immutable repo-relative trace paths, captures the selected closure before execution,
  and emits source identity, full run records, and divergent-subject summary under receipt
  `witnessed_observations`. It deliberately keeps the trace outside `evaluated_inputs[]` so
  historical observed behavior cannot be over-read as a current-run decision input. Bedrock's
  recorded SQL trace proves the narrow admission: three subjects diverge while stable repeated
  queries retain one identity across runs.
- The completed `V11.13` core cut makes generated source a named repo-scoped contract artifact:
  `artifacts.<name>` declares `kind: generated_source`, one producer task, output paths, and
  optional source inputs; consumers declare `requires_artifacts` and directly depend on the
  producer. Validation rejects dangling, overlapping, and dependency-disconnected lineage. The
  runner checks declared outputs after the producer closure and before consumers execute. Task
  JSON carries the producer map plus consumer references, and receipt `evaluated_inputs[]` captures
  producer/path/input lineage at issue time as pointer-only evidence, never as a freshness claim.
- EventCatalog pressure proved the first healthy generator and sibling-package consumer closure:
  typed `pnpm --filter @eventcatalog/language-server install`, Langium generation, and the
  downstream VS Code extension build. It also widened `prepare.source.filter` from its old
  browser-bootstrap-only boundary into a pnpm-owned dependency-hydration selector. The first
  sibling build failure identified real missing SDK and visualiser build dependencies; modeling
  those finite tasks made the final extension build pass without shell orchestration.
- Bedrock pressure proved the V11.10 replay-artifact shape on a deterministic offline NL-to-SQL
  stability harness across Ubuntu, macOS, and Windows: explicit script-test aggregation, committed
  SQL fixture replay, and the defended baseline gate all run in agent mode with no model key. Its
  live recording lane remains intentionally outside that claim because it reaches Claude, rewrites
  the fixture, and depends on an unpinned generic-pip requirements path that Ota does not yet own
  through typed dependency hydration.
- The completed V11.10 replay carrier distinguishes whether the selected baseline is still the
  last known good witness. `ota up --replay-baseline ... --json` adds
  `replay.baseline.last_known_good` with `replay_verified`, `stale_witness`, or `unavailable`
  derived from the replay result itself, so promoted archives no longer all read as equally
  current after drift or unavailable-baseline failures.
- V11.10 also names active execution governance as a replay-grade input.
  Receipts now capture a loaded org policy pack as `policy_ruleset_identity`, and replay treats a
  changed ruleset as named input drift rather than generic hidden-input suspicion.
- V11.10 also names declared env-source files when the selected lane
  actually resolved from them. Receipts capture `env_source_identity` without recording values, so
  replay can distinguish declared env-source drift from still-ambient process or policy env.

- The completed task-discovery UX batch renders closure-aware `Human Run`, `Agent Run`, and
  `Agent Policy` sections in `ota tasks` and `ota tasks --use`. It keeps the `ota-site` internal
  verification setup task agent-callable so its declared-safe public verification closures remain
  truthful without exposing setup in the default task inventory. Task mode rows now use stable
  `Container`, `Native`, then `Remote` presentation, show unsupported local planes explicitly,
  and recover native override support for container-context tasks without requiring a redundant
  task mode branch. `ota tasks --json` now carries the same canonical per-mode truth under
  `tasks[].use.modes[]`, while the existing `use.human` and `use.agent` remain selected-mode
  compatibility projections.
- Flagr pressure confirmed native and container Go module hydration, binary build, aggregate
  verification, and runtime proof. It also closed two task-discovery regressions: aggregate mode
  rows now inherit executable closure support, and Doctor no longer version-probes repo-owned
  command paths before their producer task exists. The previously open locally tagged Dockerfile
  image-build gap is now closed by first-class `action.kind: build_container_image`: contracts own
  the provider, Dockerfile, repo-relative context, and local tag without raw `docker build` glue.
  Lead Quorum proves the direct image build; Flagr carries the equivalent integration-image task,
  with its live build awaiting a healthy Docker daemon on the pressure host.
- `V11.11` contract-derived proof boundaries is implemented in Ota commit `e3bbdf02`.
  `ota proof runtime --json` now emits terminal `proof_verdict`, and Lead Quorum pressure proved
  `passed_with_unproven_boundaries` on the app lane across Ubuntu and macOS.
- V11.11 keeps that qualified proof boundary visible in human output too:
  `ota proof runtime` now renders concrete `Proof Boundaries` entries whenever `not_proved[]`
  exists, so external-network and broader-scope exclusions travel with the green proof instead of
  living only in JSON.
- V11.11 makes those proof boundaries machine-actionable too: each
  `not_proved` entry now carries an explicit `reason`, and the human proof render includes the
  same reason label for seam, adjacent-lane, and broader-scope exclusions.
- The completed V11.11 seam-evidence carrier on
  `ota proof runtime --json`: `dependency_evidence[]` now publishes runner-derived
  `level: reachable` only for declared service seams that are also on the selected
  workflow-owned required-service path and have structured readiness Ota actually owns. This
  keeps selected service reachability distinct from still-unproved exercised interaction.
- V11.11 keeps caller-side seam attempts separate from proved reachability:
  proof-derived DNS, auth, and loopback service failure signals can now publish additive
  `interaction_attempted: true` with `observation.origin: caller_side`, while the paired
  `dependency_exercise_not_proved` boundary tightens to `caller_side_only_evidence` instead of
  generic missing evidence.
- The same commit fixes detached native proof lifecycle ownership: nested `ota up --detach` leaves
  the service running for the outer proof to observe and clean up, preventing recursive teardown.
- V11.10 emits a runner-derived receipt-comparison artifact-trust record for matching
  semantic contract snapshots. It is `acquitting` for `contract_truth` only; lockfile/runtime
  artifact capture remains the next implementation cut.
- V11.10 captures declared lockfile-strict Node identity in
  `receipt.evaluated_inputs[]` at receipt authoring time: `pnpm-lock.yaml` for frozen pnpm and
  `package-lock.json` or authoritative `npm-shrinkwrap.json` for `npm ci`. It carries this through
  archived baseline and current receipt diff and labels only matching
  `declared_dependency_resolution` identity as `acquitting`. Directus and ota-site proved matching
  archived/current paths with the source-built binary; unrelated runtime findings remain separate.
- Lead Quorum is not yet the first hermetic replay target: its typed `uv pip_requirements` lane
  and Python range are real current repo truth, but not the pinned dependency/runtime pair V11.10
  needs. Treat that as a repo contract/replay-readiness gap, not a reason to weaken Ota evidence.
- V11.10 captures `runtime:node` through contract-local `node --version` on the
  same typed lockfile-strict Node hydration path. It is deliberately `narrowing` for
  `selected_runtime_version`, not an executable/image-digest acquittal.
- V11.10 adds the first immutable runtime-artifact carrier. Receipts
  recover literal digest-pinned Compose `image` values only for explicitly selected services in
  explicitly declared files and their declared Compose `depends_on` closure as
  `selected_runtime_artifact`; receipt diff treats a matching digest as `acquitting` only for that
  named artifact. Mutable tags, interpolation, inferred files, and unrelated stack services remain
  outside the claim. Immich pressure also exposed and fixed an Ota runner gap: Compose adapter-file
  preflight now resolves files relative to the same adapter `cwd` used by execution. The narrow
  Redis/PostgreSQL launch, status, and stop path passed locally with the source-built binary.
- Immich and Grafana confirmed the follow-on taxonomy need. `effects.network_kind:
  container_image_hydration` now owns registry-backed Compose image acquisition independently from
  package dependency hydration; `prepare.medium: container_images` requires this label, doctor and
  policy packs expose the same lane, and immutable image receipt evidence remains separate from
  the effect declaration.
- The same branch upgraded the direct `quick-xml` dependency to `0.41.0` after `cargo deny`
  surfaced the two XML denial-of-service advisories in `0.38.4`; the NuGet feed-provenance parser
  uses the current XML 1.0 attribute-normalization API and its focused tests pass.
- Grafana confirmed the receipt carrier on a mixed Compose stack with locally built, mutable, and
  digest-pinned services. The selected observability lane records four explicit digest-pinned
  services plus `tempo-init` through Tempo's declared `depends_on` closure, while excluding
  unrelated built and mutable stack services. This exposed and fixed the closure-recovery gap in
  Ota rather than leaving the init image absent from a selected runtime receipt.
- The same Grafana pass exposed and fixed a doctor semver gap: whitespace-separated compound
  ranges such as `>=1.26.3 <1.27` now use the canonical normalized semver path while preserving
  Ota's established shorthand comparator behavior.

## Handoff To The Next Chat

Start by reading `AGENTS.md`, this file, the canonical Ota skill, and
`docs/planning/v11.9/plan.md` through `docs/planning/v11.13/plan.md`. Then run `git status --short` in `ota`, `ota-site`,
`/Users/bobai/Workspace/Ota.run/skills`, and the active pressure repo before editing.

The continuity batch immediately before this handoff added the canonical pressure-testing protocol,
required a connected-surface decision for Ota changes, and synchronized the skill into the global
Codex and agent skill stores. Its installation smoke test could not run locally because this
machine does not currently provide `node` or `npx`; source integrity was verified with diff and
shell-syntax checks.

V11.9 through V11.13 first OSS cuts are complete and pressure-proven. Do not create a parallel
proof carrier by copying `ota proof runtime` evidence into generic `ota up` or readiness receipts:
those artifacts did not execute the proof lane. Start the next product slice only from a real repo
gap or an explicitly approved follow-on design, and follow the pressure-testing protocol exactly.

The current `1.6.24` branch-pinned pressure subset has green pushed matrices for Athena, Azure SDK
for .NET, Bedrock, Dagger, EventCatalog, Grafana, Immich, Kylrix, Lead Quorum, and Flagr. Lead
Quorum also has one newer local-only contract commit that is not yet matrix-proven; do not present
that local tip as release evidence until it is pushed and rerun. The case-study root README is
stale inventory and should not be used as the release ledger.

## Working Rules

- Read the canonical Ota skill before Ota-specific work.
- Use `references/pressure-testing-protocol.md` for every pressure pass.
- Make and record the connected-surface decision for core docs, examples, skills, and site.
- Use released Ota versions for released proof; use the active branch only for explicit unreleased
  pressure testing.
