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

# Adoption Readiness Gate

Before pushing harder into enterprise mode, ota must already feel like a no-brainer on one repo for one developer in one session.

## Product bar

The minimum bar is:

- `ota doctor` makes the repo legible in under 10 seconds
- `ota explain` turns the findings into an ordered, believable fix plan
- `ota detect` is truthful about what it can write, merge, or only review
- `ota up` gets a healthy repo to a ready state without noisy guesswork
- `ota run <task>` feels dependable and fails cleanly when the task fails
- `ota agents` turns the same contract into useful repo-local agent guidance

## UX bar

The command surface must stay:

- calm
- exact
- grouped when repetition does not help
- visually consistent across repo and workspace commands
- honest in `Why:` and `Next:`

## Readiness checks

Before enterprise-first work is treated as the main path:

- dogfood ota on a representative set of messy real repos
- keep the premium text snapshots green
- keep `ota run ux-review` green when text UX or help output changes
- keep README, quickstart, examples, and command reference aligned with shipped behavior
- keep exact remediation coverage strong for the common manager flows users actually hit
- keep the workspace path at parity with the repo path

## Decision rule

If ota still feels like extra config before it feels like obvious help, the right move is to improve adoption value first and defer broader enterprise surface.
