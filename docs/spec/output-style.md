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

# Output Style Contract

This document defines the text rendering contract for Ota command output.

## Goals

- preserve deterministic, parse-friendly text output
- improve readability and scannability for humans
- keep machine contracts in JSON stable and separate

## Text style rules

- rich text mode uses signature headers: `🦦 <COMMAND> <target>`
- list bullets use `▸` in rich text mode
- field labels use one consistent key color in TTY mode
- command examples use Ota command accent color in TTY mode
- banner and update surfaces use one consistent light brown/gold accent in TTY mode (`214,161,95`)
- key commands include one consistent signature line: `doctor first, contract second`
- sections are separated with explicit spacing (`---` where used today)
- per-item output (tasks/repos/findings) keeps one blank line between items

## Plain mode (`--plain`)

Use `--plain` for minimal, ASCII-only text output:

- disables icons and emoji in headers
- disables ANSI styling and colors
- uses ASCII `-` bullets
- uses `Signature: doctor first, contract second` text line for signature-bearing commands
- preserves command semantics and ordering
- does not affect JSON output shape

Example:

```bash
ota --plain tasks .
ota --plain detect --dry-run .
ota --plain workspace up .
```

## Compatibility constraints

- one-line compatibility outputs remain unchanged where tests assert exact strings:
  - `VALID ...`
  - `VALID WORKSPACE ...`
  - `CLEANED ...`
  - `NO CLEANUP NEEDED ...`
- text output may evolve visually, but should preserve:
  - deterministic ordering
  - stable section meaning
  - clear next-action guidance

## Testing policy

- key text surfaces must have style regression tests
- `--plain` behavior must be tested for:
  - no `🦦`
  - no `▸`
  - ASCII `-` bullets for list/next blocks
