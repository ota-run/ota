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

# UX Review Loop

Human-readable ota output is product surface, not incidental debug text.

When help text, rich/plain text output, or docs that promise command behavior change, maintainers
should review both the automated premium snapshots and a small live-command path.

## Automated gate

Run:

```bash
ota run ux-review
```

This keeps the snapshot-backed premium CLI surfaces stable:

- root help
- repo `doctor`
- repo `doctor --plain`
- repo `detect`
- repo `explain` at narrow terminal widths
- repo `up`
- repo `run`
- repo `agents`
- workspace `validate`
- workspace `doctor`
- workspace `explain`
- workspace `up`
- workspace `run`

## Live review path

Review these commands in a real terminal before shipping UX-sensitive changes:

```bash
ota --help
ota doctor .
ota --plain doctor .
ota detect --dry-run .
env COLUMNS=48 ota explain .
ota up .
ota agents .
ota run install
ota workspace doctor .
ota workspace explain .
ota workspace up .
```

## Review questions

The output should stay:

- calm
- exact
- grouped when repetition does not help
- honest in `Why:` and `Next:`
- visually consistent across repo and workspace surfaces
- truthful about what ota can write, merge, or only review

## Decision rule

If a change makes the CLI more technically complete but less legible in the first session, the
change is not ready yet.
