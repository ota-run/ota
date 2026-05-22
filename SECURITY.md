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

# Security Policy

Ota treats the contract, diagnosis model, JSON output, installer paths, execution semantics, and
release distribution surfaces as security-sensitive infrastructure.

## Supported versions

Security fixes are only guaranteed on the latest released version of Ota.

If you believe a vulnerability affects an older release, reproduce it on the latest release first
before reporting it.

## Report a vulnerability

Do not open a public GitHub issue for a suspected security vulnerability.

Instead, report it privately to:

- `os@ota.run`

Include:

- affected Ota version from `ota --version`
- operating system and shell
- whether the issue affects `doctor`, `detect`, `init`, `up`, `run`, install/update flows, or
  release/distribution paths
- a minimal reproduction or proof-of-concept
- expected impact and any known mitigations

## What to expect

- Ota will acknowledge the report as quickly as practical
- reports are triaged under maintainer stewardship
- fixes may land privately first and be disclosed after a patched release is available

## Security scope

The most sensitive areas include:

- command execution and task launching
- backend/provider boundaries
- installer and self-update flows
- policy/env resolution
- JSON output consumed by automation
- release packaging and distribution
- agent-facing safety and writable-boundary guidance

## Public issues that are still welcome

Use normal GitHub issues for:

- non-sensitive bugs
- docs corrections
- example or fixture gaps
- false-positive diagnostics

When in doubt, prefer private disclosure first.
