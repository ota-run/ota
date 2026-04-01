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

# Output Style

This page explains how ota formats human-readable text output.

Use it when you need to know:

- what the CLI looks like in rich terminals
- how `--plain` changes the output
- what stays stable across text modes
- how to read command output consistently

## Why this matters

Users and agents often read text output before they ever use JSON.

That output needs to be:

- readable
- deterministic
- easy to scan
- stable enough to trust in scripts and logs

## Rich mode

In rich terminals, ota uses a consistent visual style.

Typical rules include:

- signature headers like `🦦 <COMMAND> <target>`
- consistent bullet styling
- one key label color
- one accent for commands and update surfaces
- clear section spacing

The point is not decoration. The point is fast scanning.

## Plain mode

Use `--plain` when you want ASCII-only output.

Plain mode:

- removes emoji and icons
- removes ANSI colors
- uses simple ASCII bullets
- preserves the same command meaning and ordering

Examples:

```bash
ota --plain tasks .
ota --plain detect --dry-run .
ota --plain workspace up .
```

## What stays stable

Text styling can evolve, but the meaning should not surprise users.

Users should be able to rely on:

- command ordering
- status words like `VALID`, `READY`, and `NOT READY`
- clear next-action guidance
- no change to JSON semantics

## When to use plain mode

- when a terminal cannot render color or emoji well
- when a script wants the simplest possible text output
- when logs need to stay ASCII-only
- when a human wants the least noisy readable output

## Use cases

- a CI job records plain text logs
- a user wants predictable output in an old terminal
- a support engineer needs to compare command output across systems
- an agent wants to scan status without rendering noise

## What this is not

- not JSON output
- not a machine contract
- not a styling guide for app UI
- not a promise that every visual detail stays fixed forever

## Related docs

- [Commands](commands.md)
- [Doctor findings](doctor-finding-contract.md)
- [JSON output](json-output.md)
