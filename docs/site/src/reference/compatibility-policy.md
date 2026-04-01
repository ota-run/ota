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

# Compatibility Policy

This page explains what users can rely on when ota evolves.

Use it when you need to know:

- whether a contract change is backward compatible
- how unknown keys are handled
- what happens when a field is replaced
- how to think about version bumps

## Why this matters

Users do not just need a working command. They need to know whether an existing repo contract will
keep working after an ota update.

This page gives that answer in plain language.

## Current version rule

- `version: 1` is the current contract version.
- breaking schema changes require a version bump.
- additive fields can be introduced within V1 when they do not break valid contracts.

That means users should expect stability, but not a frozen schema forever.

## Unknown keys

Current V1 policy is strict:

- unknown keys fail parsing
- there is no warning-only unknown-key mode today
- known fields are the contract surface; everything else is rejected

This favors clear contracts over permissive guessing.

## Deprecation and migration

When ota replaces a field or command shape, the safest path is:

1. document the replacement first
2. keep the old shape working when feasible
3. call out the migration path explicitly
4. avoid silent removal

That keeps changes reviewable for both humans and automation.

## Practical implication

If you are building around `ota.yaml`, treat the public contract as stable, but not magic.

Do not rely on:

- undocumented keys
- silent fallback behavior
- changes that only exist in implementation

Do rely on:

- explicit contract fields
- documented command output
- versioned behavior changes

## Use cases

- a maintainer wants to know whether a new release can break old contracts
- a CI owner wants to understand if a repo with unknown keys will still parse
- a platform team needs a migration path for renamed fields

## Related docs

- [Compatibility surface](compatibility-surface.md)
- [Contract](contract.md)
- [Commands](commands.md)
