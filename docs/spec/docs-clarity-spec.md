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

# Docs Clarity Spec

Purpose: define the documentation rules that make Ota easy to understand on first read.

This spec applies to:

- `docs/spec/*`
- site-native public docs
- command-help copy when it mirrors documentation language
- README command and adoption summaries when they describe shipped behavior

## Goals

- make the difference between `validate`, `doctor`, `check`, `init`, and `detect` obvious
- explain when to use a command, why it exists, and what result to expect
- keep repo-level and workspace-level behavior clearly separated
- keep trust-sensitive write paths distinct from review paths
- make version and confidence language explicit instead of implied

## Required structure for command docs

Every command doc should include:

- when to use
- why it exists
- at least one concrete use-case
- one or more runnable examples
- any important write / review / destructive boundary
- any related command that is the preferred follow-up

## Canonical command splits

The docs must consistently explain:

- `ota validate` checks contract correctness
- `ota doctor` checks readiness and blockers
- `ota check` runs explicit checks only
- `ota init` creates the first starter contract
- `ota detect --write` is conservative and high-confidence only
- `ota detect --merge` updates an existing contract additively
- `ota detect --rewrite` regenerates an existing contract from scratch with confirmation

## Version syntax clarity

Runtime and tool requirements should be documented with examples that make the semantics obvious:

- `8` is an example of an exact required version
- `>=8` is an example of accepting newer versions
- `^8` is an example of a compatible version range

The docs should state which syntax is the strictest and which syntax allows newer versions.

## Confidence clarity

Output and docs should consistently map confidence to meaning:

- `high` is the most trustworthy inferred value
- `medium` is plausible and may be needed for a valid starter contract
- `low` is weak and should remain excluded from automatic writes

## Workspace clarity

Workspace docs must clearly separate:

- repo readiness contract: `ota.yaml`
- workspace bootstrap contract: `ota.workspace.yaml`
- repo commands: operate on one repo contract
- workspace commands: orchestrate multiple repo contracts

## UX boundary rules

Docs should not:

- imply `ota init` is the right update path for existing contracts
- imply `ota detect --write` can safely replace `ota init`
- blur review paths and write paths together
- present command lists without operator guidance
- hide the distinction between required and optional runtime/tool mismatches

## Acceptance criteria

- readers can explain the role of `validate`, `doctor`, `check`, `init`, and `detect` after one pass
- all command pages include when/why/use-case structure
- docs explicitly distinguish first-write, update, and rewrite behaviors
- version syntax examples are present and unambiguous
- confidence color and meaning are described consistently
- workspace docs do not duplicate repo-readiness semantics
