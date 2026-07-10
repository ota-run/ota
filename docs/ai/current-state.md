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
- immediate proof gate: finish review and commit the first receipt-authored pnpm lockfile identity
  cut, then pressure a changed archived-versus-current lockfile comparison without reading the
  current filesystem as if it were historical evidence

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
- The in-progress V11.10 cut captures a declared lockfile-strict pnpm `pnpm-lock.yaml` SHA-256 in
  `receipt.evaluated_inputs[]` at receipt authoring time, carries it through archived baseline and
  current receipt diff, and labels only a matching `declared_dependency_resolution` identity as
  `acquitting`. Directus proved the matching archived/current path with the source-built binary;
  its local Node-version diagnosis remains a separate repo/runtime finding.

## Handoff To The Next Chat

Start by reading `AGENTS.md`, this file, the canonical Ota skill, and
`docs/planning/v11.10/plan.md`. Then run `git status --short` in both `ota` and
`/Users/bobai/Workspace/Ota.run/skills` before editing.

The continuity batch immediately before this handoff added the canonical pressure-testing protocol,
required a connected-surface decision for Ota changes, and synchronized the skill into the global
Codex and agent skill stores. Its installation smoke test could not run locally because this
machine does not currently provide `node` or `npx`; source integrity was verified with diff and
shell-syntax checks.

Continue V11.10 by pressure-testing a changed archived-versus-current receipt identity on a
deterministic lockfile-strict pnpm lane. Do not infer lockfile identity from a later filesystem
read. Follow the pressure-testing protocol exactly and record whether the repo exposes a contract
issue, implementation issue, or Ota platform gap.

## Working Rules

- Read the canonical Ota skill before Ota-specific work.
- Use `references/pressure-testing-protocol.md` for every pressure pass.
- Make and record the connected-surface decision for core docs, examples, skills, and site.
- Use released Ota versions for released proof; use the active branch only for explicit unreleased
  pressure testing.
