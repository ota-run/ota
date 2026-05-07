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

# Ota Studio Product Spec

Status: planned.

## Purpose

Ota Studio is the visual operational surface of Ota.

It exists to make repo readiness, contract review, execution, and observation feel immediate and
high-confidence for humans working locally, while keeping the CLI and machine-readable contracts as
the canonical engine and truth source.

Studio is not a general IDE replacement and not a web dashboard for arbitrary infrastructure.
Studio is the local Ota app surface for repos and, later, workspaces.

Studio should not be framed as “replacing the terminal.”
The stronger and more truthful framing is:

- Ota becomes the canonical operational surface
- CLI and Studio are equal first-party clients of the same engine
- agents can drive the same underlying truth model

## Product position

The current Studio MVP proved:

- Ota can render a truthful visual surface from existing JSON outputs
- users benefit from visual contract review and topology inspection
- recent activity, reviewed writes, and top-level next-step guidance belong in Studio

The mature product direction is stronger:

- interactive by default
- serve-first and app-like
- repo registry and switching
- reviewed write flows
- execution triggers
- live operational visibility from Ota events

The earlier static export prototype has served its purpose and should not remain as a supported
Studio experience. Any reusable internals may be salvaged, but the product surface should converge
on one interactive `ota studio` path.

## Long-term delivery baseline

Studio’s long-term baseline remains:

- localserver + localhost browser UI as the default and free path
- no separate desktop client as a product-critical execution environment
- optional thin desktop wrapper only as an optional premium distribution layer

The product boundary is therefore:

- local Ota CLI owns contract and execution truth
- Studio renders and orchestrates local Studio server payloads
- the UI should remain easy to inspect, debug, and automate through the local HTTP/API contract

## Core principles

1. Studio is an Ota client, not a second system.

- `ota.yaml` remains the source of truth
- Ota core remains the execution engine
- Studio does not invent runtime semantics
- operation/event payloads are sourced from [`event-schema.md`](event-schema.md)

2. Interactive is the default.

- `ota studio` should open the interactive local Studio
- users should not need to think in terms of exported HTML snapshots or alternate Studio modes

3. Full-screen local app quality matters.

- Studio should use the full viewport intentionally
- it should feel like a premium operational tool, not a long document page

4. Review before mutation.

- Studio can trigger safe writes and safe runs
- all mutation paths must be explicit, reviewed, and routed through Ota core

5. Observation is a first-class value proposition.

- Studio should show what Ota is doing, not just what the contract says
- activity, orchestration, logs, and receipts are part of the product, not a later extra

6. One product, multiple interfaces.

- CLI, Studio, and agents should share one execution and evidence model
- users who prefer terminal workflows remain first-class

## Product layers

Studio should eventually support five layers inside one product:

1. Inspect

- doctor
- detect
- topology
- env
- receipts

2. Review

- contract diff
- inferred draft
- readiness blockers
- execution plan

3. Apply

- reviewed contract writes
- deterministic fixes
- later safe operational actions

4. Trigger

- run tasks
- up
- later targeted service or task actions where Ota can own them honestly

5. Observe

- live task status
- dependency and orchestration graph
- logs
- readiness changes
- receipt and archive state

This layered model is bigger than a GUI wrapper and should shape the Studio roadmap directly.

## Primary user outcomes

Studio should let a user:

1. understand a repo quickly
2. review the current or inferred contract clearly
3. see how services, tasks, and targets relate
4. trigger safe Ota operations without leaving the app
5. watch Ota activity and receipts as they happen
6. switch across known repos without losing context

## Command model

Primary command:

```bash
ota studio
```

Expected mature behavior:

- starts the local interactive Studio server
- auto-registers the repo if it has `ota.yaml`
- opens or switches Studio to that repo

Secondary flags and variants:

- `ota studio --repo <path>`
  - open Studio focused on a specific repo
- `ota studio --file <path>`
  - open Studio focused on an explicit non-default contract path
- `ota workspace studio`
  - future explicit workspace Studio surface

## Repo registration

When `ota studio` is run inside a repo that contains `ota.yaml`, or when Studio is opened through
an explicit contract file path, Studio should auto-register that repo and contract in a global
local registry.

This enables:

- recent repos
- explicit contract-path reopen for repos that do not use the default `ota.yaml` location
- pinned repos later
- fast repo switching
- last activity summaries per repo
- future workspace-aware landing pages

Studio should not require users to re-enter repo paths manually once a repo is known.

Studio should also be able to surface previously opened repos even when the current shell is not in
that repo, so the app becomes a durable Ota workspace rather than a one-shot per-repo page.

## Studio Home / Repos

Studio needs an explicit home surface for known repos.

Purpose:

- give users a persistent Ota landing place outside the currently opened repo
- make returning to active repos immediate

Must include:

- recent repos
- contract path per repo
- last-known readiness state
- last-known activity state
- last opened timestamp
- pinned or favorite repos later

Studio Home should feel like the operational entrypoint to Ota, not a thin repo-picker.

## Application shell

Studio should use an app shell, not a report layout.

Minimum mature structure:

- top bar
  - current repo
  - current contract status
  - activity indicator
  - action surface
- left rail
  - repo switcher
  - core sections
- main workspace
  - pane-based content with room for split views
- right-side or lower contextual panel
  - detail, logs, receipts, or action context

Studio should feel premium:

- deliberate hierarchy
- strong use of space
- high-density without clutter
- clear motion and state changes
- no “docs page in a browser” feeling

## Durable panes

Studio should converge on five durable panes:

1. Overview
2. Contract
3. Draft
4. Topology
5. Run / Evidence

The exact arrangement can evolve, but this information architecture should stay stable so Studio
grows like an app surface rather than a shifting collection of ad hoc cards.

## Core surfaces

### 1. Overview

Purpose:

- show current repo state quickly
- summarize what matters now

Must include:

- doctor verdict
- contract review state
- recent activity state
- primary next action
- repo metadata
- contract presence and location

### 2. Contract

Purpose:

- make `ota.yaml` understandable and reviewable
- reduce manual YAML burden for common authoring flows

Must include:

- current contract view
- structured editor for common fields
- guided authoring for common contract sections
- YAML side-by-side visibility
- diff before write
- exact apply paths

Rules:

- Studio should help the user do less manual YAML work, not require more
- Studio does not mutate YAML directly with UI-owned logic
- all writes route through Ota core
- guided authoring must write through reviewed Ota-core-backed mutation paths
- YAML must remain visible so users can see the exact contract truth being written
- Studio must not hide the contract behind forms completely

### 3. Draft

Purpose:

- make inference review explicit instead of hiding it inside contract editing

Must include:

- `ota detect --dry-run`-equivalent draft state
- inferred fields grouped by confidence
- provenance and source visibility
- pack suggestions when relevant
- merge/apply preview

Rules:

- Draft remains Ota-owned inference, not Studio-owned schema logic
- Studio must not blur inferred draft truth into current contract truth
- the winning interaction is inspect, explain, diff, and approve, not a form-first YAML builder

## Guided contract authoring

Guided contract authoring is part of the Studio plan.

Purpose:

- let users create or improve `ota.yaml` with less manual author burden
- turn Ota inference, packs, and review flows into a guided authoring experience

Studio should support:

- structured forms for common contract sections
- starter-contract authoring
- guided completion of missing required fields
- detect-backed suggestions and pack-backed suggestions
- reviewed merge/apply flows after guided edits

Studio should not become:

- a second contract engine
- a generic form builder with its own schema logic
- a surface that hides YAML or diff truth from the user

The intended user loop is:

1. inspect current repo or contract state
2. review inferred or suggested fields
3. fill or confirm common fields through guided controls
4. inspect the exact diff
5. apply through Ota core
6. continue with doctor, validate, up, or run

### 4. Topology

Purpose:

- explain how declared workloads and services relate

Must include:

- tasks
- services
- listeners
- targets
- shared backends
- contexts
- readiness
- activation relationships

The topology view must answer:

- what serves what
- what targets what
- what backend each workload binds to
- what readiness contract is declared
- why target resolution or activation behaves the way it does

The topology view should support multiple inspection modes:

- graph view for relationship discovery
- list/detail view for precise scanning
- focused task or service inspector for one object at a time

The topology view must not be decorative. A user should be able to navigate from any meaningful
topology node back to:

- the originating contract field
- the inferred source when the shape came from detection rather than an existing contract
- runtime evidence or receipt-backed execution truth when available

### 5. Run / Evidence

Purpose:

- trigger safe operations and explain what happened

Must include:

- `doctor`
- `validate`
- `up`
- selected `run`
- reviewed apply actions
- Action Center for safe operation launch and recent results
- recent archived receipts
- recent durable logs metadata
- operation history
- receipt details
- failure and ready summaries
- provenance and age labels
- env resolution
- readiness explanation

The Action Center should make safe actions feel intentional and reviewable.

It should include:

- exact action preview
- exact task/context/backend/flags when relevant
- queued safe actions when the product later supports multi-step execution
- recent operation results

The operation history model should let a user move across:

- current operation
- recent operations
- failed operations
- ready operations
- agent-triggered operations when that source metadata exists

The evidence surface should be explicit, not implicit. It should include:

- execution timeline
- raw logs tab
- receipt summary
- failure origin when present
- recovery or next-step guidance
- durable links or paths to archived artifacts

Later phases should add:

- live operation events
- active task graph
- log streaming

Rules:

- show exactly what will run
- preserve explicit Ota flags and context
- do not hide operational consequences
- do not allow “magic run anything” behavior
- show action preview before dangerous or stateful operations where appropriate
- respect future safe-task policy boundaries for agent-driven flows

Selected `run` operations must mean:

- declared Ota tasks only
- safe tasks first
- stricter defaults for agent-triggered actions when Ota has enough policy/task-safety truth to
  enforce them

## Stale review and apply safety

Studio must handle stale review as a first-class user experience, not just a hidden safety check.

When a reviewed apply action becomes stale because the contract changed after review, Studio must:

- block the write
- tell the user why the review is stale
- refresh the contract and draft truth
- require re-review of the resulting diff before apply can continue

Studio should help multiple operator types without splitting into separate products:

- first-time user
- contract author
- CI owner
- agent operator

## Initial safe write and execution boundaries

Studio can grow into execution and mutation, but only in clear layers.

Safe order:

1. inspect
2. review
3. safe apply
4. safe run
5. live observe

The first write and execution boundaries should remain:

- starter contract apply
- additive detect merge
- deterministic `doctor`, `validate`, `up`, and `run` actions through Ota core

Studio should not begin with:

- freeform YAML editing
- rewrite-by-default flows
- arbitrary shell execution
- hidden state mutation

## Live activity direction

Studio should eventually show Ota activity from:

- Studio-triggered operations
- terminal-triggered Ota commands
- agent-triggered Ota commands

The user should be able to see:

- what is running
- dependency orchestration
- current phase
- logs
- result
- receipt

This is a major product value proposition and a key differentiator.

This direction is especially valuable for agent-heavy workflows:

- an agent changes code
- the agent runs `ota run typecheck`
- Studio lights up with live status, logs, and orchestration state
- a human can supervise the work without attaching to that exact shell

## Workspace direction

Workspace Studio is valuable, but must be explicit and separate.

Repo Studio remains the primary surface first.

Future workspace Studio should be:

- `ota workspace studio`

and not hidden inside repo Studio as mixed multi-repo behavior.

## Packaging and identity

Studio and Ota should remain one product family.

Correct long-term shape:

- `ota` = product and engine
- `ota studio` = first-party visual operational client

Separate packaging may happen later:

- CLI distribution
- Studio distribution

But they should not drift into separate brands or truth systems.

## Non-goals

Studio is not:

- a second contract format
- a hosted dashboard in the first serious release
- a browser extension as the main product
- a general terminal replacement
- an IDE clone
- a custom orchestration engine that bypasses Ota core

## Migration from the current MVP

The earlier static Studio MVP should be treated as:

- data-contract proof
- boundary proof
- early UX proof

The product-facing direction is now:

- one interactive `ota studio` experience
- no parallel static Studio product mode

Reusable implementation pieces may be preserved internally, but the static surface itself should be
retired as the interactive shell becomes real.
