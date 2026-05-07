<!--
               █████
              ░░███
      ██████  ███████    ██████
     ███░░███░░░███░    ░░░░░███
    ░███ ░███  ░███      ███████
    ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
     ░░░░░░     ░░░░░░   ░░░░░░░░

  Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

  Do NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

  Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
  You may not use this file except in compliance with that License.
  Unless required by applicable law or agreed to in writing, software distributed under the
  License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
  either express or implied. See the License for the specific language governing permissions
  and limitations under the License.

  If you need additional information or have any questions, please email: os@ota.run
-->

# Ota Studio UI Pages and Flows

Status: planned.

This document is the Studio page-level blueprint. Build behavior is defined in:
- [product-spec.md](product-spec.md)
- [http-api.md](http-api.md)
- [architecture.md](architecture.md)
- [design-system.md](design-system.md)

No page should interpret command output semantics; all views render normalized Studio server models.

## Global shell structure

Each Studio screen uses one persistent frame:

- top bar
- left navigation rail
- content workspace
- contextual detail panel

The shell is always visible, even while nested views are loading or erroring.

## Route model

Primary routes:

1. `/` → Overview
2. `/contract` → Contract pane
3. `/draft` → Draft pane
4. `/topology` → Topology pane
5. `/run` → Run / Evidence pane

Home behavior:

- repository not selected: show Home screen with recent repos and quick actions
- repository selected: default to `/` (Overview)

## Shared layout invariants

Across all routes:

- shell top bar shows repo identity and contract presence
- rail keeps current route and repo context
- detail panel shows the selected object or event trace
- action surface is disabled unless route contract permits safe actions
- keyboard shortcuts apply only to navigation and close, not destructive operations

## Page definitions

### 1) Overview (`/`)

Purpose:
- establish "what matters now" in one pass
- prevent user confusion about readiness blockers

Core cards:
- Repo + mode
- Doctor summary (blocked / warning / ready)
- Last executed action and result state
- Draft readiness state
- Repo activity marker (idle / running / failed / unknown)

Flow:
1. Studio resolves current repo
2. Loads contract + doctor + activity summary models
3. Shows action hints when blockers exist
4. Routes user to Contract or Draft based on selected first action

Failure/blocked state:
- if repo path missing: guide to `--repo` / `--file`
- if contract malformed: show exact path and `ota validate` context

### 2) Contract (`/contract`)

Purpose:
- show canonical truth with no confusion between current and inferred state

Core zones:
- Contract metadata (path, kind, source)
- normalized contract view (readable)
- raw YAML preview (code panel)
- review status strip (up-to-date / stale / missing)

Flow:
1. Load current contract model
2. If missing, show discover mode with action to open Draft first run path
3. If stale relative to working tree, show stale flag and refresh intent
4. Keep write actions in Run / Evidence only

### 3) Draft (`/draft`)

Purpose:
- review inferred contract proposals with confidence and source

Core zones:
- proposal view (grouped by confidence)
- provenance table (inferred from, pack hint, confidence)
- merge preview diff
- stale check state and re-run draft button

Flow:
1. Load detect draft model
2. Group changes by confidence tier
3. Mark each group with applyability
4. Block reviewed actions when stale

### 4) Topology (`/topology`)

Purpose:
- allow topology comprehension before mutation

Core zones:
- context/service/workload list view
- detail panel for each selected topology node
- relationship chips (serves / targets / backends / readiness)

Flow:
1. Load normalized topology model
2. Build list and graph fallback
3. Selecting any node reveals source mapping and evidence links
4. If node source is inferred, show explicit inferred badge and confidence

### 5) Run / Evidence (`/run`)

Purpose:
- action launch hub and evidence review center

Core zones:
- Action center (doctor, validate, up, run, reviewed apply actions)
- recent operations list
- receipt and log metadata stream
- guidance panel (what will run / what changed / recovery)

Flow:
1. Load operation/recent evidence model
2. Show available operations for selected repo context
3. Require explicit preview before run
4. Render active operation progress and post-run proof

## Cross-route flow graph

```text
Home/No-repo
  └─> Overview
      ├─> Contract
      ├─> Draft
      ├─> Topology
      └─> Run / Evidence

Overview
  └─> Contract
      ├─> Draft (if inference-based review is needed)
      └─> Run / Evidence (for action launch)

Draft
  └─> Contract (to compare intended state)
  └─> Run / Evidence (for reviewed writes, then evidence)
```

## State transitions

| Source | Target state | Trigger | Guard |
|---|---|---|---|
| Home | Overview | repo selected | contract path resolved |
| Overview | Draft | blocked by detect suggestion | draft model available |
| Overview | Run/Evidence | explicit action required | user command |
| Contract | Draft | no obvious contract issues and draft preferred | draft not stale |
| Draft | Run/Evidence | action approved + reviewed stamp valid | freshness check pass |
| any | error screen | API read fails | explicit retry path |

## Interaction states per page

Each page must support:
- loading state with route-specific loading skeleton
- empty state with next action
- stale state with refresh CTA
- blocked state with exact blockers
- no-surprise failure state with copy that includes next step

## Non-goals for page models

- no inline shell command entry
- no freeform YAML editing inside Studio
- no topology inference done in browser
- no background mutation without explicit action and confirmation model
