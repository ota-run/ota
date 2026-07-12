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

- branch: `1.6.24-implementation`
- released baseline: `v1.6.23`
- active implementation and proof slice: `V11.10` witnessed query-identity evidence; declared
  static replay inputs remain receipt `evaluated_inputs[]`, while Bedrock-style historical query
  traces are emitted separately as attested `witnessed_observations.query_traces[]`
- queued design refinement: `V11.11` seam-evidence provenance, which keeps caller-side attempts,
  dependency-side or round-trip exercise, and separately recorded negative controls distinct on
  the existing proof carrier
- immediate proof gate: review this receipt-carrier widening, then publish the Bedrock matrix on
  the active branch. Continue V11.13 generated-artifact pressure only after the query observation
  carrier is proven across the advertised OS lanes.

## Recent Completed Slice

- The current uncommitted V11.10 refinement adds contract-owned
  `tasks.<name>.witnessed_observations.query_traces[]` for existing JSONL query traces. Ota
  validates immutable repo-relative trace paths, captures the selected closure before execution,
  and emits source identity, full run records, and divergent-subject summary under receipt
  `witnessed_observations`. It deliberately keeps the trace outside `evaluated_inputs[]` so
  historical observed behavior cannot be over-read as a current-run decision input. Bedrock's
  recorded SQL trace proves the narrow admission: three subjects diverge while stable repeated
  queries retain one identity across runs.
- The in-progress `V11.13` core cut makes generated source a named repo-scoped contract artifact:
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

- The uncommitted task-discovery UX batch renders closure-aware `Human Run`, `Agent Run`, and
  `Agent Policy` sections in `ota tasks` and `ota tasks --use`. It keeps the `ota-site` internal
  verification setup task agent-callable so its declared-safe public verification closures remain
  truthful without exposing setup in the default task inventory. Task mode rows now use stable
  `Container`, `Native`, then `Remote` presentation, show unsupported local planes explicitly,
  and recover native override support for container-context tasks without requiring a redundant
  task mode branch. `ota tasks --json` now carries the same canonical per-mode truth under
  `tasks[].use.modes[]`, while the existing `use.human` and `use.agent` remain selected-mode
  compatibility projections.
- `V11.11` contract-derived proof boundaries is implemented in Ota commit `e3bbdf02`.
  `ota proof runtime --json` now emits terminal `proof_verdict`, and Lead Quorum pressure proved
  `passed_with_unproven_boundaries` on the app lane across Ubuntu and macOS.
- The same commit fixes detached native proof lifecycle ownership: nested `ota up --detach` leaves
  the service running for the outer proof to observe and clean up, preventing recursive teardown.
- V11.10 now emits an initial runner-derived receipt-comparison artifact-trust record for matching
  semantic contract snapshots. It is `acquitting` for `contract_truth` only; lockfile/runtime
  artifact capture remains the next implementation cut.
- The in-progress V11.10 cut captures declared lockfile-strict Node identity in
  `receipt.evaluated_inputs[]` at receipt authoring time: `pnpm-lock.yaml` for frozen pnpm and
  `package-lock.json` or authoritative `npm-shrinkwrap.json` for `npm ci`. It carries this through
  archived baseline and current receipt diff and labels only matching
  `declared_dependency_resolution` identity as `acquitting`. Directus and ota-site proved matching
  archived/current paths with the source-built binary; unrelated runtime findings remain separate.
- Lead Quorum is not yet the first hermetic replay target: its typed `uv pip_requirements` lane
  and Python range are real current repo truth, but not the pinned dependency/runtime pair V11.10
  needs. Treat that as a repo contract/replay-readiness gap, not a reason to weaken Ota evidence.
- The in-progress next cut captures `runtime:node` through contract-local `node --version` on the
  same typed lockfile-strict Node hydration path. It is deliberately `narrowing` for
  `selected_runtime_version`, not an executable/image-digest acquittal.
- The current V11.10 cut adds the first immutable runtime-artifact carrier. Receipts
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
`docs/planning/v11.13/plan.md`. Then run `git status --short` in `ota`, `ota-site`,
`/Users/bobai/Workspace/Ota.run/skills`, and the active pressure repo before editing.

The continuity batch immediately before this handoff added the canonical pressure-testing protocol,
required a connected-surface decision for Ota changes, and synchronized the skill into the global
Codex and agent skill stores. Its installation smoke test could not run locally because this
machine does not currently provide `node` or `npx`; source integrity was verified with diff and
shell-syntax checks.

Before widening the V11.13 model further, finish one healthy real generator lane and one
independent sibling-consumer lane. Do not infer generated-file freshness from timestamps or later
filesystem reads. Follow the pressure-testing protocol exactly and record whether the repo exposes
a contract issue, implementation issue, or Ota platform gap.

## Working Rules

- Read the canonical Ota skill before Ota-specific work.
- Use `references/pressure-testing-protocol.md` for every pressure pass.
- Make and record the connected-surface decision for core docs, examples, skills, and site.
- Use released Ota versions for released proof; use the active branch only for explicit unreleased
  pressure testing.
