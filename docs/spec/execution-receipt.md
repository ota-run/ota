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
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Execution Receipt

Status: shipped.

This document defines the execution receipt for ota mutation and execution commands.

The goal is to produce a deterministic, machine-readable record of what ota believed,
what it did, and what happened during execution.

## Purpose

The receipt is meant to answer:

- what scope ota used
- what contract version or hash it acted on
- what backend and lifecycle were chosen
- what was reused or acquired
- what env or policy affected the decision
- what tasks, checks, or services ran
- what failed or was blocked
- what the safe next step is

## Commands

The receipt applies to execution and mutation surfaces such as:

- `ota run`
- `ota up`
- workspace execution flows

It should not replace `doctor`, `detect`, or `init`.

## Planned shape

The receipt includes:

- `ok`
- `path`
- `scope`
- `contract`
- `workspace`
- `backend`
- `lifecycle`
- `acquired`
- `env`
- `policy`
- `steps`
- `blocked`
- `summary`
- `next`

The current shipped surface records `ok`, `path`, `scope`, `contract`, `backend`,
`lifecycle`, `steps`, `blocked`, `summary`, and `next` for `ota run`, `ota up`, and
workspace execution flows. On failure, the normal text output keeps `Why` and `Next`
before the trailing summary block. The receipt keeps the structured summary data for
JSON and optional receipt output.

Future policy-backed provisioning receipts should additionally be able to record:

- the requested runtime or tool
- the approved source that won
- the version or distribution that was selected
- whether the action was install, select, or verify-only
- whether policy allowed mutation for that decision

## Behavior

- deterministic ordering
- machine-readable first
- human-readable summary in the normal execution output, with failure output keeping `Why` and `Next` ahead of the trailing summary block
- explicit source of truth for decisions
- no hidden auto-fix behavior
- provenance for policy-backed provisioning when that layer exists

## Non-goals

- replacing live logs
- turning diagnosis into execution
- hiding failure details behind a generic summary
- introducing opaque background state

## Relationship to other surfaces

- `doctor` diagnoses readiness
- `detect` infers contract data
- `diff` compares contract meaning
- `explain` turns findings into a remediation plan
- `receipt` records what execution actually did
