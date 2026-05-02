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

# Ota Studio Design System

Status: planned.

This document defines the implementation-facing visual system for Studio.

The goal is not generic “nice UI.” The goal is a premium local operations app that feels
purpose-built for repo truth, review, execution, and evidence.

## Design rules

1. Studio is an app, not a docs page.
2. Use the full viewport.
3. Dense does not mean cluttered.
4. Contract, topology, and evidence must feel like first-class surfaces.
5. Visual hierarchy must make operational truth obvious at a glance.

## Initial visual direction

Initial target:

- desktop-grade dark workspace
- warm neutral surfaces
- high-clarity signal colors
- restrained motion
- heavy emphasis on grid, alignment, and structured density

Light mode can come later. Phase 1 should optimize for one strong mode, not two weaker ones.

## Core layout

Studio should use a four-region shell:

1. top bar
2. left rail
3. primary content canvas
4. contextual detail panel

### Top bar

Must include:

- repo identity
- contract state
- activity state
- primary action surface

### Left rail

Must include:

- Studio Home / Repos entry
- Overview
- Contract
- Draft
- Topology
- Run / Evidence

### Primary content canvas

Purpose:

- main pane rendering
- split views when needed

### Contextual detail panel

Purpose:

- object detail
- logs
- receipt metadata
- action preview

The detail panel may collapse, but the shell should be designed assuming it exists.

## Pane design rules

### Overview

Should read like an executive operational brief:

- strongest summary cards first
- next action visible above the fold
- no low-signal tables at the top

### Contract

Must feel editorial and review-oriented:

- current contract
- normalized view
- diff context
- reviewed action controls

Never collapse into a generic form-builder-only experience.

### Draft

Must make inference confidence obvious:

- confidence grouping
- source/provenance
- pack suggestions
- merge/apply preview

Draft should feel distinct from Contract. Users must never wonder whether they are looking at
current truth or inferred proposal.

### Topology

Must support:

- graph view
- list/detail view
- focused inspector

No graph-only implementation is acceptable.

### Run / Evidence

Must feel operational:

- actions on top
- current/recent history close behind
- evidence, logs, and receipts easy to inspect

## Components

### Summary cards

Use for:

- doctor verdict
- contract review state
- activity state
- action needed

Rules:

- one primary point per card
- strong title
- short supporting text
- visible state color

### Action cards

Use for:

- reviewed apply actions
- `doctor`
- `validate`
- `up`
- declared task runs

Rules:

- show exact action name
- show context/flags when relevant
- show risk level when relevant
- show resulting state clearly after completion

### Evidence panels

Use for:

- receipt summary
- timeline
- logs
- recovery guidance

Evidence should feel durable and inspectable, not ephemeral.

### Diff panels

Rules:

- current and proposed state visible together when space allows
- destructive or rewrite implications must be visually distinct
- stale review must interrupt the apply surface clearly

## Visual tokens

Phase 1 should define CSS variables for:

- background
- surface
- surface-elevated
- border
- text-primary
- text-secondary
- accent
- success
- warning
- danger
- info
- focus

The implementation may choose exact hex values, but the semantic token names should be fixed from
the start.

## Typography

Use two families only:

- UI/display family
- mono/code family

Rules:

- headings compact and strong
- body text highly readable at dense sizes
- code and YAML always in mono
- do not rely on browser default typography

## Motion

Motion should communicate state, not decorate it.

Allowed:

- pane transitions
- detail panel reveal
- loading skeletons
- event arrival emphasis

Avoid:

- constant animation
- ornamental motion loops
- graph motion that obscures operational truth

## Empty, loading, and blocked states

These states need design, not placeholders.

### Empty

Example:

- no known repos yet
- no contract yet
- no recent activity yet

Must provide:

- one clear explanation
- one recommended next action

### Loading

Must feel deliberate:

- shell remains stable
- pane skeletons preserve layout
- no full-page spinner if avoidable

### Blocked

Must feel actionable:

- explain the blocker
- preserve context
- show the next safe action

## Operational color semantics

Canonical meaning:

- success = ready / passed
- warning = degraded / review needed
- danger = blocked / failed
- info = neutral execution or topology explanation

Never use color alone for meaning. Pair with title or iconography.

## Build-facing anti-patterns

Do not ship:

- a centered document page inside a browser
- graph-only topology
- giant undifferentiated cards
- UI that hides exact actions
- generic form-builder styling
- a light layer of polish over an obviously snapshot-first layout

Studio must feel intentional on a large screen from the first serious implementation.
