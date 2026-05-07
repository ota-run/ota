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

# Ota Studio UI Components

Status: planned.

This document defines canonical Studio UI components and their acceptance criteria.

All components:
- render from Studio-owned read models
- do not duplicate validation or command execution semantics
- preserve source fields and explicit action context

## Visual component tokens

Use [design-system.md](design-system.md) for spacing, color, motion, and typography.

## Shared shells

### Top bar

Required fields:
- repo identity
- contract status tag
- activity indicator
- action state (idle / running / blocked)

Behavior:
- sticky; stays visible
- disables action surface if no valid action context

### Left rail

Required fields:
- repo switch entry
- route links (`Overview`, `Contract`, `Draft`, `Topology`, `Run / Evidence`)

Behavior:
- current route is always highlighted
- quick navigation does not trigger implicit actions

### Contextual detail panel

Modes:
- collapsed
- focused object view
- action result details
- evidence trace

Behavior:
- receives selected object ID from active pane
- preserves scroll position only when same focus type

## Core content components

### SummaryCard

Purpose:
- compact operational truth blocks

Required fields:
- title
- status
- primary value
- secondary context
- optional action hint

Rules:
- never show two contradictory statuses in one card
- use status palette only for canonical states

### ConfidenceGroup

Purpose:
- render draft inference by confidence tiers

Required fields:
- confidence bucket (`high`, `medium`, `low`, `unknown`)
- change count
- source summary
- applyability status

Rules:
- sort from highest confidence to lowest
- low bucket can only be launched via explicit confirmation

### EvidenceCard

Purpose:
- render operation summaries and proofs

Required fields:
- operation id
- operation kind
- status
- source
- started/completed times
- receipt path when available

Rules:
- show failed result with recovery link
- show ready/completed with proof path

### ActionCard

Purpose:
- present declared safe actions with explicit preview

Required fields:
- action name (`doctor`, `validate`, `up`, `run`, apply actions)
- request context (task/context/backend/member/flags)
- safety check result
- preview label

Rules:
- must require explicit confirm for stateful/agent-visible actions
- no ambiguous action text

### DiffPanel

Purpose:
- compare current vs reviewed / draft states

Required fields:
- base label
- candidate label
- changed lines or sections
- provenance markers (`contract` vs `inferred`)

Rules:
- current and proposal visibility must not be ambiguous
- destructive changes must be visually distinct

### TopologyNodeCard

Purpose:
- represent one logical topology object

Required fields:
- node name
- node kind (`service`, `workload`, `target`, `backend`, `context`)
- source status (`contract`, `detected`)
- relation count

Rules:
- selecting node opens detail panel with contract mapping and evidence links

### RunTimeline

Purpose:
- show active or recent operation activity

Required fields:
- operation id
- phase
- step-level events
- timestamp
- state transitions

Rules:
- tolerate duplicate events
- tolerate missing optional event fields

## Action components

### ReviewedActionBar

Used by Contract/Draft/Run.

Required checks:
- reviewed payload hash freshness
- source/target match check
- confirmation mode for write actions

Behavior:
- if stale, disable execute and show refresh-first message

### Modal patterns

Required for:
- run preview
- stale review warning
- destructive operation warning

Rules:
- modal text must include exact task/context/backend/flags
- modal has explicit cancel and confirm

## Reuse and extension policy

Component extension is permitted only when:
- new surface reuses existing component contracts
- new behavior is additive and testable
- no new semantics are introduced by frontend rendering

Do not add one-off, one-screen components for repeated behavior (status, evidence, actions).
