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

# Task Variants

This document defines the current post-V1 direction for task variants in shipped ota behavior.

## Goal

Add platform-specific task execution without creating a second task system.

## Contract shape

Simple tasks remain valid:

```yaml
tasks:
  setup:
    run: ./scripts/setup.sh
```

Expanded tasks may add variants:

```yaml
tasks:
  setup:
    run: ./scripts/setup.sh
    variants:
      - when:
          os: windows
        run: .\scripts\setup.ps1
```

## Current selector support

Current shipped selector support is:

- `when.os: linux`
- `when.os: macos`
- `when.os: windows`

## Resolution rules

- ota checks variants before the default execution
- the first matching variant for the current OS wins
- duplicate variants for the same `when.os` are rejected in validation
- if no variant matches, ota falls back to the default `run` or `script`
- if there is no default and no matching variant, task execution fails clearly

## Design intent

This is intentionally a general conditional-variant model, not a platform-only branch in the schema.

That keeps room for future selectors without changing task identity or command shape.
