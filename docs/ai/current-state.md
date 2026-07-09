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
- active planning slice: `V11.12` hydration input provenance
- immediate proof gate: pressure-test explicit .NET NuGet source/config provenance on a real repo
  before broadening the source model

## Recent Completed Slice

- `V11.11` contract-derived proof boundaries is implemented locally in Ota commit `58bd5ccc`.
- It adds machine-readable `proof_scope` and contract-derived `not_proved` truth so a narrow green
  runtime proof cannot be read as repo-global proof.

## Handoff To The Next Chat

Start by reading `AGENTS.md`, this file, the canonical Ota skill, and
`docs/planning/v11.12/plan.md`. Then run `git status --short` in both `ota` and
`/Users/bobai/Workspace/Ota.run/skills` before editing.

The continuity batch immediately before this handoff added the canonical pressure-testing protocol,
required a connected-surface decision for Ota changes, and synchronized the skill into the global
Codex and agent skill stores. Its installation smoke test could not run locally because this
machine does not currently provide `node` or `npx`; source integrity was verified with diff and
shell-syntax checks.

Continue with V11.12 by selecting a real .NET repo that declares explicit NuGet source or config
truth. Pressure the declared-versus-resolved hydration provenance on the selected `ota up --json`
path before broadening source semantics. Follow the pressure-testing protocol exactly and record
whether the repo exposes a contract issue, implementation issue, or Ota platform gap.

## Working Rules

- Read the canonical Ota skill before Ota-specific work.
- Use `references/pressure-testing-protocol.md` for every pressure pass.
- Make and record the connected-surface decision for core docs, examples, skills, and site.
- Use released Ota versions for released proof; use the active branch only for explicit unreleased
  pressure testing.
