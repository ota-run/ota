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

# V2 Plan

Status: active implementation.

V2 should expand Ota only after preserving the shipped V1 trust model.

The point of V2 is not broader surface area for its own sake.

The point of V2 is to make Ota:

- more trustworthy on real repos
- easier to adopt
- better at handling messy repo reality
- more useful to agents without introducing parallel truth

Current V2 slices already in motion include:

- better detect coverage for common Node and Python repo shapes
- conservative existing-contract comparison and additive merge for `ota detect`
- more real-fixture pressure on mixed, conflicting, and legacy repo shapes
- agent guidance surfaced through existing text and JSON command paths

## Priorities

1. Trust
2. Adoption
3. Better real-repo handling
4. Stronger agent usefulness

## Rules

- preserve the V1 repo and workspace contract boundaries
- preserve doctor-first and trust-sensitive detection/init behavior
- preserve deterministic shell-native execution unless an explicit replacement model is justified
- keep new abstractions justified by real fixture or integration pressure
- do not create parallel truth outside `ota.yaml` and `ota.workspace.yaml`
- do not build broad feature families before one narrow slice proves clear value

## First candidate tracks

### Track 1: Trust

- keep `ota doctor`, `ota detect`, and `ota init` sharp on real repos
- preserve conservative write behavior and provenance-first inference
- tighten edge cases only where they improve correctness or user trust

### Track 2: Adoption

- reduce the cost of getting a real repo into Ota
- improve import and migration paths without weakening trust-sensitive writes
- prioritize work that helps one repo or one team adopt Ota faster
- keep detect merge/update review-first and additive-only unless broader write behavior earns trust through real fixtures

### Track 3: Better Real-Repo Handling

- improve mixed-reality, conflicting-signal, and partial-repo handling
- expand fixture pressure only where it exposes real product gaps
- prefer better precedence and conflict handling over broader speculative coverage

### Track 4: Stronger Agent Usefulness

- make the `agent` section more operationally useful
- focus on concrete guidance such as:
  - `safe_tasks`
  - `verify_after_changes`
  - `writable_paths`
- keep agent guidance deterministic, human-readable, and shared with humans where possible

## Optional Later Tracks

These are not core V2 pillars.

### Narrow Export

- consider export only if one exact target proves real duplicated-truth pain
- Ota must remain the source of truth and exports must remain derived artifacts
- do not start a broad export program

### Workspace UX Extensions

- consider workspace improvements only when they compose existing repo truth cleanly
- do not invent a second bootstrap engine or duplicate repo readiness semantics

### Execution Model Expansion

- consider post-V1 execution model work only if it preserves determinism, debuggability, and contract clarity

## Non-goals

- replacing `ota.yaml` as the repo source of truth
- turning Ota into a hidden workstation manager
- adding provider-specific GitHub, GitLab, or Bitbucket behavior into the core model
- building a broad export matrix before a single target proves clear value
- introducing a second execution truth outside the contract
