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
- active planning slice: `V11.10` replay artifact trust
- immediate proof gate: the first immutable runtime-artifact lane is proven locally on Immich;
  commit the Ota and pressure-repo batches, then push the Immich branch for matrix proof before
  widening the artifact taxonomy

## Recent Completed Slice

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
  explicitly declared files as `selected_runtime_artifact`; receipt diff treats a matching digest
  as `acquitting` only for that named artifact. Mutable tags, interpolation, and inferred files
  remain outside the claim. Immich pressure also exposed and fixed an Ota runner gap: Compose
  adapter-file preflight now resolves files relative to the same adapter `cwd` used by execution.
  The narrow Redis/PostgreSQL launch, status, and stop path passed locally with the source-built
  binary.
- Immich also exposed a follow-on taxonomy opportunity, not a bug in the current receipt lane:
  Compose image pulls are declared as broad network effects today. Do not mislabel them as
  package-manager `dependency_hydration`; consider a separate `container_image_hydration`
  network-effect kind only if another real repo confirms the need.

## Handoff To The Next Chat

Start by reading `AGENTS.md`, this file, the canonical Ota skill, and
`docs/planning/v11.10/plan.md`. Then run `git status --short` in `ota`, `ota-site`,
`/Users/bobai/Workspace/Ota.run/skills`, and the active pressure repo before editing.

The continuity batch immediately before this handoff added the canonical pressure-testing protocol,
required a connected-surface decision for Ota changes, and synchronized the skill into the global
Codex and agent skill stores. Its installation smoke test could not run locally because this
machine does not currently provide `node` or `npx`; source integrity was verified with diff and
shell-syntax checks.

After the Immich GitHub matrix, choose the next immutable executable or image-digest carrier only
from another real pressure repo. Do not infer identity from a later filesystem read. Follow the
pressure-testing protocol exactly and record whether the repo exposes a contract issue,
implementation issue, or Ota platform gap.

## Working Rules

- Read the canonical Ota skill before Ota-specific work.
- Use `references/pressure-testing-protocol.md` for every pressure pass.
- Make and record the connected-surface decision for core docs, examples, skills, and site.
- Use released Ota versions for released proof; use the active branch only for explicit unreleased
  pressure testing.
