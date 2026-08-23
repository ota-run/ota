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

# Mutation Controls and Caching

This document defines the V5 target for explicit mutation boundaries and durable caching rules in ota.

The goal is to keep every write path honest, reviewable, and predictable while allowing small caches that improve performance without weakening trust.

## Purpose

Mutation controls and caching support:

- explicit write boundaries
- review-before-write flows
- safe additive updates
- audit-friendly change paths
- deterministic reuse of cached data where allowed

## Mutation principles

- review-only commands must not mutate source contracts
- candidate application may apply only explicit, reviewed operations
- destructive replacement must not exist without a versioned operation model and owned carrier
- policy-aware diagnosis remains read-only by default
- hidden mutation paths are not allowed

## Current shipped controls

ota already exposes the core mutation boundaries through existing commands:

- `ota detect --dry-run` is review-only
- `ota detect --candidate-out PATH` publishes reviewable source-bound changes
- `ota contract apply-candidate PATH --write` creates only a missing contract
- `ota contract apply-candidate PATH --write --carrier git` updates an existing tracked contract
  through reviewed candidate operations and expected-HEAD compare-and-swap
- removed repo-level `detect --merge`, `--apply`, `--apply-all`, `--rewrite`, and `--yes` flags
  refuse before repository access with a stable migration code
- `ota workspace detect --merge` adds discovered repos without overwriting existing entries
- `ota workspace detect --rewrite --yes` replaces the workspace contract with a regenerated result

Workspace mutation flags retain their separate existing semantics until that surface receives its
own candidate/application model.

## Mutation visibility

Mutation-aware output should make it obvious:

- whether a command can write
- whether a command wrote
- what was written
- what was intentionally left unchanged
- whether the command mode allows mutation at all

The additive `mutation_allowed` field in doctor JSON exists for policy-aware diagnosis and should remain stable.

## Caching rules

Caching may be used when it preserves determinism and trust.

Allowed cache behavior:

- small
- explicit
- easy to invalidate
- derived from stable source inputs
- never changes command semantics

The current implementation uses process-local success-only caches for loaded repo and workspace
contracts, keyed by path plus source file metadata so a rewritten file is re-read on the next
lookup.

Not allowed:

- hidden stale state
- cross-command ambiguity
- cache-based contract truth
- cache behavior that outlives source drift checks

## Scope

This surface is for:

- explicit write boundaries
- trusted repeatable reads
- small deterministic caches
- mutation-aware diagnostics

It is not for:

- background daemons
- invisible state machines
- generic persistence layers
- speculative optimization that changes observable output
