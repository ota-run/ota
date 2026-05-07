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

   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
   You may not use this file except in compliance with that License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Ota Studio Phase 1 Build Plan

Status: planned.

This document turns the Studio spec into an executable Phase 1 implementation plan.

It is intentionally narrower than the full Studio roadmap. Phase 1 builds the first serious
interactive Studio shell. It does not pull Phase 2 registry persistence or Phase 5 live event
streaming forward.

## Phase 1 goal

Replace the snapshot-first Studio mental model with a serve-first local app shell that is good
enough to become the primary `ota studio` experience.

Phase 1 should prove:

- `ota studio` is the one real Studio surface, not a static export
- the shell can render durable panes from Ota-owned read surfaces
- the frontend can stay thin while the server owns normalization

## Phase 1 scope

Included:

- `ota studio` starts the local Studio server
- browser opens automatically
- full-viewport Studio app shell
- current repo Overview pane
- current repo Contract pane
- current repo Draft pane
- current repo Topology pane
- current repo Run / Evidence pane
- current repo only
- read-only pane rendering from existing Ota-owned surfaces

Explicitly deferred:

- global repo registry persistence
- `Studio Home / Repos` backed by the real registry
- reviewed apply mutations
- Action Center launches
- operation history beyond existing archived evidence
- live event subscriptions
- workspace Studio

These belong to later roadmap phases and should not be smuggled into Phase 1.

## Modeling-first gate

Phase 1 starts after these Studio modeling docs are complete and reviewed:

- [ui-pages-and-flows.md](./ui-pages-and-flows.md)
- [ui-components.md](./ui-components.md)
- [ui-user-journeys.md](./ui-user-journeys.md)
- [ui-metrics-and-feedback.md](./ui-metrics-and-feedback.md)

Implementation should not begin until each file has an explicit acceptance check or equivalent
internal review note.

## Existing surfaces to reuse

Phase 1 should start from existing Ota read surfaces and current Studio data paths.

Backend inputs already available:

- `ota doctor --json`
- `ota detect --dry-run --json`
- rendered inferred contract draft text
- current contract text when present
- `ota execution topology --json`
- existing archived receipt/log metadata already used by the current Studio prototype

Current implementation touch points already in the repo:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands/studio_output.rs`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/command-reference.md`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/json-output-reference.md`

Phase 1 should reuse these paths rather than inventing a second Studio stack.

## Recommended implementation shape

### Backend/server side

The server should own:

- repo resolution
- current repo identity
- pane payload assembly
- HTML shell serving
- static asset serving if needed

The server should not:

- expose raw command output directly to the browser as the long-term contract
- let the browser decide contract or execution semantics

### Frontend/browser side

The frontend should own:

- app shell layout
- pane routing
- local UI state
- rendering of server-owned pane payloads

The frontend should not:

- parse CLI text
- rebuild Ota semantics
- infer topology or execution state on its own

## Phase 1 deliverables

### Deliverable 1: promote `ota studio` to serve-first

User-visible outcome:

- `ota studio` launches the interactive local Studio by default
- the static Studio prototype is no longer a supported user-facing mode

Concrete touch points:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/command-reference.md`

Acceptance checks:

- `ota studio .` starts a localhost server
- default help and docs describe the interactive path as primary
- the old static Studio surface is no longer treated as a supported Studio product path

### Deliverable 2: app shell and pane routing

User-visible outcome:

- Studio opens as a full-viewport app
- left rail and top bar exist
- panes are explicit:
  - `Overview`
  - `Contract`
  - `Draft`
  - `Topology`
  - `Run / Evidence`

Concrete touch points:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands/studio_output.rs`
- optional focused Studio-specific HTML/CSS/JS partials or companion modules if the current file
  becomes too monolithic

Acceptance checks:

- the shell no longer reads like a long report page
- the pane model matches the product spec exactly
- layout uses the full viewport intentionally

### Deliverable 3: current repo identity and repo-focused landing state

User-visible outcome:

- Studio clearly shows the current repo and contract identity
- first open lands inside the current repo context

Concrete touch points:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands/studio_output.rs`

Acceptance checks:

- current repo root is visible
- current contract path is visible when present
- contractless repos still open coherently

### Deliverable 4: normalized Overview pane

User-visible outcome:

- doctor verdict
- contract review state
- activity state
- primary next action

Backend inputs:

- `doctor --json`
- detect comparison state already available in current Studio payload
- archived activity summary already available in current Studio payload

Concrete touch points:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands/studio_output.rs`

Acceptance checks:

- Overview never requires the browser to interpret raw CLI text
- the top-level cards read like an operational brief, not raw JSON fragments

### Deliverable 5: normalized Contract pane

User-visible outcome:

- current contract text
- readable contract state
- no mutation yet

Backend inputs:

- current contract text
- optional normalized contract metadata if cheap to expose now

Concrete touch points:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands/studio_output.rs`

Acceptance checks:

- current contract truth is clearly distinct from inferred draft truth
- the pane still works when no contract exists

### Deliverable 6: normalized Draft pane

User-visible outcome:

- inferred contract draft
- grouped confidence/provenance
- pack suggestions when available

Backend inputs:

- `ota detect --dry-run --json`
- rendered inferred contract text already available in current Studio payload

Concrete touch points:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands/studio_output.rs`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/json-output-reference.md`

Acceptance checks:

- Draft is its own pane, not merged into Contract
- contractless repos still get a useful first-run review path
- inference confidence is visually obvious

### Deliverable 7: normalized Topology pane

User-visible outcome:

- declared workloads and service relationships are understandable
- initial view can be list/detail first if graph polish is not ready yet

Backend inputs:

- `ota execution topology --json`

Concrete touch points:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands/studio_output.rs`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/json-output-reference.md`

Acceptance checks:

- pane answers:
  - what serves what
  - what targets what
  - what backend each workload uses
- UI does not invent topology semantics beyond the topology read model

### Deliverable 8: normalized Run / Evidence pane

User-visible outcome:

- recent archived receipts
- log metadata
- readiness/failure summaries
- evidence-oriented view, not just a dump

Backend inputs:

- existing archived activity payload already emitted by the current Studio prototype

Concrete touch points:

- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands/studio_output.rs`

Acceptance checks:

- evidence is easy to scan
- failure/ready summaries are visible
- raw artifact paths remain accessible

## Suggested code organization

Phase 1 does not require a broad refactor, but it should avoid burying the new app shell in one
ever-growing string template.

Preferred direction:

- keep CLI argument handling in `/Users/bobai/Workspace/Ota.run/ota/src/cli.rs`
- keep command dispatch in `/Users/bobai/Workspace/Ota.run/ota/src/cli/commands.rs`
- keep Studio rendering and server concerns in focused Studio-specific modules

If the current Studio code needs to split, prefer a small Studio-specific grouping such as:

- Studio command/server assembly
- Studio pane payload normalization
- Studio HTML/app shell rendering

The exact Rust module names can follow the repo’s existing patterns, but the separation of concerns
should be explicit.

## Required docs updates during Phase 1

When Phase 1 lands, update:

- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/command-reference.md`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/json-output-reference.md`
- `/Users/bobai/Workspace/Ota.run/ota/CHANGELOG.md`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/studio/ui-pages-and-flows.md`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/studio/ui-components.md`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/studio/ui-user-journeys.md`
- `/Users/bobai/Workspace/Ota.run/ota/docs/spec/studio/ui-metrics-and-feedback.md`

Update public docs only after the interactive shell behavior is stable enough to teach honestly.

## Acceptance checklist

Phase 1 is done only when all of these are true:

- `ota studio` is serve-first by default
- the browser opens to the local app shell
- the shell is full-viewport and pane-based
- panes match the spec:
  - `Overview`
  - `Contract`
  - `Draft`
  - `Topology`
  - `Run / Evidence`
- the frontend renders server-owned pane payloads
- the frontend does not parse CLI text or invent semantics
- contractless repos still get a useful Studio experience
- no Phase 2 registry persistence or Phase 4 execution launch logic was pulled in accidentally
- UI modeling docs exist and are approved as the implementation contract for Phase 1 screens/components/journeys

## Validation checklist

Minimum required validation:

- focused CLI tests for `ota studio` default mode and argument behavior
- focused Studio rendering tests for the five panes
- focused contractless and contract-bearing Studio regression coverage
- `cargo check -q`
- `git diff --check`

Optional but recommended manual checks:

- launch Studio on a repo with `ota.yaml`
- launch Studio on a repo without `ota.yaml`
- confirm pane routing and layout
- confirm Draft and Contract are visually distinct
- confirm Topology and Run / Evidence stay truthful and readable

## Immediate next phase after Phase 1

After Phase 1 is stable, move to Phase 2:

- real repo registry persistence
- Studio Home / Repos backed by the registry
- non-default contract reopen support through the registry

Do not jump straight from Phase 1 into live events or workspace Studio.
