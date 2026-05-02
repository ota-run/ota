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

# Ota Studio Cleanup and Migration Plan

Status: planned.

This document defines how the earlier static Studio prototype should be retired while preserving
any internal code that directly accelerates the real interactive Studio.

## Core decision

The static Studio prototype is not a supported product surface.

It proved:

- data-contract viability
- boundary viability
- early UX direction

It should now be treated as scaffolding, not as a mode to preserve.

## Cleanup rule

For every existing static-Studio implementation piece, ask:

1. does this directly help the interactive local Studio shell?
2. does it preserve one product story instead of two?

If both answers are not yes, delete it.

## Keep versus delete

### Keep and repurpose

Allowed to survive if still useful:

- repo resolution code
- payload assembly and normalization code
- current contract and inferred draft preparation
- topology payload assembly
- archived activity/evidence normalization
- local server bootstrapping

### Delete or rewrite

Should not survive as-is:

- snapshot-first command semantics
- user-facing language that treats HTML export as Studio
- report-style layout assumptions
- snapshot-only product flags and branches
- snapshot-specific tests that no longer match the supported product

## Migration sequence

1. keep only reusable backend normalization and server logic
2. replace the user-facing shell with the real interactive app shell
3. move docs/help to a single interactive `ota studio` story
4. delete dead snapshot-specific rendering and command branches
5. tighten tests so they protect the interactive shell, not the retired prototype

## Immediate cleanup targets

Before or during Phase 1 implementation:

- remove snapshot as a supported Studio concept from the spec and product language
- stop adding snapshot-only UX features
- reshape existing Studio code around:
  - server
  - pane payloads
  - interactive shell rendering

## Removal criteria

The old static product surface is considered retired when:

- `ota studio` launches the interactive shell by default
- product/docs no longer describe static Studio as a supported experience
- the main Studio tests assert pane-based app behavior rather than snapshot-export behavior
- any surviving old code exists only because it is reused internally by the interactive shell

## Non-goals

This cleanup plan does not require:

- throwing away useful normalization logic
- rewriting everything from scratch
- keeping a transitional product mode for caution alone

The goal is one product direction, not maximal deletion for its own sake.
