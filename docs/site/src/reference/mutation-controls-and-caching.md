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

This page explains when ota reads, when it writes, and why those decisions stay conservative.

Use it when you need to know:

- whether a command can mutate files
- which command mode performs review only
- when a write happens
- why ota uses small caches without changing meaning

## Why this matters

Users should not have to guess whether `detect` or `init` will write to disk.

They also should not have to wonder whether a cached contract lookup changes command truth.

This page is about keeping writes explicit and reads trustworthy.

## Mutation rules

The current public rule is simple:

- review-only commands do not mutate source contracts
- merge commands only apply eligible additive changes
- rewrite commands are destructive and require explicit confirmation
- policy-aware diagnosis stays read-only
- hidden mutation paths are not allowed

## Current write surfaces

Current commands with write behavior include:

- `ota detect --merge`
- `ota detect --rewrite --yes`
- `ota workspace detect --merge`
- `ota workspace detect --rewrite --yes`
- `ota init`
- `ota workspace init`

These commands make the write boundary explicit so users can review before changing files.

## Caching behavior

ota may cache small, explicit reads when that preserves determinism.

The important user-facing rule is:

- caches must not change command meaning
- rewritten files must be re-read
- cache state must never outrank the source contract

That means cache exists only to avoid unnecessary re-parsing, not to become hidden truth.

## Use cases

- a maintainer wants to preview a contract change before writing it
- a CI owner wants `detect --dry-run` to stay review-only
- a team wants to know that `init` writes only when asked
- an operator wants confidence that ota is not hiding stale contract state

## What this is not

- not a background sync engine
- not a persistent hidden state machine
- not a general-purpose cache policy surface
- not a replacement for the contract or filesystem truth

## Related docs

- [Commands](commands.md)
- [Compatibility surface](compatibility-surface.md)
- [Audit and provenance](audit-and-provenance.md)
