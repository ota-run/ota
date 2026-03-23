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

# Fixture Repo Plan

Ota needs real repo shapes to avoid designing in fantasy.

## Current fixture coverage

- Node
- Python
- Go
- mixed Node/Python
- mixed Node/Go
- Java
- Java Maven repo
- Java Gradle multi-module repo
- Docker-heavy repo
- unsupported Docker-only repo
- Node conflict monorepo
- ugly real-world repo
- polyglot ops repo
- contract-discovery and task-variant repo

These now exist as canonical real-shape fixtures under `tests/fixtures/real`.

The binding V1 bar for these fixtures is in [v1/release-gate.md](v1/release-gate.md).

## Next fixture pressure

- deeper `doctor` assertions against service and lifecycle behavior
- more precedence/conflict assertions on mixed-reality repos
- targeted follow-up fixtures only when they expose a real product gap

## Purpose

Fixture repos should drive:

- `doctor` trust
- `init` usefulness
- `detect` precedence and coverage
- output stability across real shapes
