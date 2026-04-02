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

# Brand Style

This document is the canonical visual system for Ota CLI + docs.

## Core identity

- mark: `🦦` in rich CLI headers
- product line: `doctor first, contract second`
- voice: direct, explicit, deterministic, no hidden magic

## Color tokens

- `ota-accent`: `#0f766e` (primary Ota identity)
- `ota-accent-soft`: `#14b8a6` (secondary accent)
- `ota-command`: `rgb(214, 161, 95)` (CLI command highlight)
- `ota-key`: `rgb(102, 217, 255)` (CLI field labels)
- confidence:
  - high: green
  - medium: yellow
  - low: red

## CLI style

- rich header format: `🦦 <COMMAND> <target>`
- key commands (`doctor`, `init`, `detect`, `up`, `run`) include:
  - rich: `◉ doctor first, contract second`
  - plain: `Signature: doctor first, contract second`
- structured bullets:
  - rich: `▸`
  - plain: `-`
- commands in guidance text are always backticked and accent-styled in TTY.

## Docs style

- use the Ota accent tokens in the site docs theme
- keep the primary logo source in the site docs asset set
- keep approved logo-size variants in the site docs asset set
- keep the favicon in the site docs asset set
- docs copy should reuse the same core phrase:
  - `doctor first, contract second`

## Guardrails

- never mix multiple leading symbols in one header line
- keep one key-label color across sections
- preserve plain mode readability and parity with rich mode semantics
- avoid command docs that are list-only; always include when/why/use-case
