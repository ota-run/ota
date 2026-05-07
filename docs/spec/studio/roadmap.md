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

# Ota Studio Roadmap

Status: planned.

This roadmap turns the Studio spec into implementation slices.

The rule is:

- retire the static Studio prototype as a user-facing product surface
- build the interactive Studio as the only supported future Studio path
- keep every phase aligned with Ota core truth

The roadmap also follows the durable Studio progression:

1. inspect
2. review
3. apply
4. trigger
5. observe live activity

This progression preserves the product trust boundary:

- every meaningful Ota action should eventually yield a structured result and a structured event
  stream
- Studio should observe the same truth produced for CLI and agents, not a Studio-only execution
  model

## Phase 0: spec freeze

Goal:

- establish the Studio product and architecture contract before more implementation

Required outputs:

- Studio product spec
- Studio architecture spec
- Studio roadmap

Exit criteria:

- product direction is clear
- static MVP is explicitly marked as proof-only and scheduled for retirement
- implementation can sequence against named phases

## Phase 1: serve-first Studio shell

Goal:

- replace the static prototype with a true interactive local app shell

Included:

- `ota studio` starts the local Studio server
- browser opens automatically
- full-viewport app shell
- repo-focused landing state
- explicit Studio Home / Repos landing surface
- current repo overview
- current repo contract/topology/review panes using existing Ota read surfaces

Non-goals:

- no repo registry yet
- no cross-process live events yet
- no workspace Studio yet

Acceptance criteria:

- Studio feels like an app, not a document
- `ota studio` is the primary path
- the static Studio surface is no longer treated as a supported user-facing mode

## Phase 2: repo registry and switcher

Goal:

- make Studio useful across multiple real repos

Included:

- global repo registry
- auto-register repo on `ota studio` when `ota.yaml` exists
- auto-register repo when opened via explicit contract file path
- recent repos list
- repo switching UI
- last-opened metadata
- last-known readiness/activity hints per repo
- Studio Home with persistent repo cards

Non-goals:

- no multi-repo execution graph yet
- no workspace-level orchestration yet

Acceptance criteria:

- users can move across known repos inside Studio
- Studio feels persistent, not one-shot

## Phase 3: reviewed apply and polished contract surface

Goal:

- make Studio the best contract review surface

Included:

- first-class contract pane
- first-class draft pane
- current/inferred/reviewed outputs
- guided contract authoring for common fields and starter flows
- deterministic reviewed apply flows
- clear mutation feedback
- better editor-grade layout and diff clarity
- stale-review recovery UX with forced refresh and re-review

Non-goals:

- no freeform YAML editor as the primary mode
- no Studio-owned mutation semantics

Acceptance criteria:

- a user can confidently review and apply safe contract changes without dropping to the shell
- a user can complete common contract authoring flows with guided controls while still seeing the
  exact YAML and diff truth

## Phase 4: Studio-triggered operations

Goal:

- let users run core Ota actions from Studio

Included:

- trigger `doctor`
- trigger `validate`
- trigger `up`
- trigger selected declared `run` operations
- Action Center surface for safe launches and recent results
- exact action context shown before launch
- exact flags/context/backend visible before launch
- resulting logs and receipts shown in Studio
- explicit evidence panel shape with timeline, raw logs, receipt summary, failure origin, and
  recovery guidance

Non-goals:

- no generic arbitrary shell command launcher
- no hidden background runs

Acceptance criteria:

- users can stay inside Studio for common operational flows
- execution still routes through Ota core, not Studio-side logic

## Phase 5: live activity and event model

Goal:

- make Studio a real observation surface for terminal and agent activity

Included:

- structured Ota operation ids
- structured local event model
- on-demand local session service
- Studio subscriptions to repo activity
- operation history views for current, recent, failed, ready, and agent-triggered activity
- active task status
- dependency/orchestration visibility
- live logs
- receipt completion updates
- agent-visible source metadata
- cross-shell and cross-agent repo activity visibility for the same Ota contract and receipt model

Non-goals:

- no hosted control plane
- no permanent global daemon requirement

Acceptance criteria:

- `ota run` from terminal or an agent becomes visible in Studio for that repo
- Studio shows real operational state without scraping terminal output
- Studio can distinguish Studio-triggered, terminal-triggered, and agent-triggered operations
- one repo can be supervised visually even when execution starts outside the Studio UI

## Phase 6: workspace Studio

Goal:

- add an explicit multi-repo Studio surface after repo Studio is strong

Included:

- `ota workspace studio`
- workspace repo inventory
- workspace readiness rollups
- workspace activity visibility
- workspace execution and orchestration summaries

Non-goals:

- do not overload repo Studio with hidden workspace semantics

Acceptance criteria:

- workspace Studio feels intentional and explicit, not bolted onto repo Studio

## Product cuts

Things to avoid while building Studio:

- keeping snapshot export as the primary experience
- preserving static Studio as a parallel supported experience
- building a browser extension first
- jumping straight to Electron or a heavyweight desktop shell before the local app model is proven
- duplicating Ota semantics in frontend code
- adding freeform mutation before reviewed apply is solid
- shipping fake live state that is really file polling guesswork
- introducing a required always-on global daemon too early
- overbuilding workspace behavior before repo Studio is excellent
- building a graph-only topology experience without list/detail and object-focused inspection modes

## Success bar

Studio is successful when:

- it feels like a premium local Ota app
- it improves adoption beyond the terminal alone
- it remains trustworthy because it is Ota-core-driven
- it becomes the best place to inspect, review, trigger, and observe repo operations locally
