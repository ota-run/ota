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

# Compatibility Surface

This page lists the user-facing surfaces that should stay stable unless ota makes an explicit
versioned change.

Use this page when you build around ota output or behavior and need to know what is safe to depend
on.

## Stable surfaces

Repo commands:

- `ota validate`
- `ota tasks`
- `ota services`
- `ota run`
- `ota doctor`
- `ota check`
- `ota init`
- `ota detect`
- `ota up`
- `ota clean`
- `ota diff`
- `ota explain`
- `ota agents`
- `ota extensions`

Workspace commands:

- `ota workspace validate`
- `ota workspace tasks`
- `ota workspace list`
- `ota workspace doctor`
- `ota workspace check`
- `ota workspace run`
- `ota workspace up`
- `ota workspace explain`
- `ota workspace init`
- `ota workspace detect`

## What should stay stable

Users and tools should be able to rely on:

- exit behavior
- JSON top-level shape
- key meaning
- deterministic ordering for list outputs
- clear human status lines such as `VALID`, `READY`, or `NOT READY`

If one of those changes, the change should be explicit and versioned.

## Why this matters

This is the page for tool authors, CI owners, and maintainers who wrap ota output.

If you depend on:

- JSON fields
- output order
- exit codes
- command presence

then you need a stable surface inventory, not just a command list.

## Use cases

- a CI integration reads `ota doctor --json`
- a wrapper tool depends on the order of `ota tasks`
- a support team needs to know whether a new command output is a breaking change
- a maintainer wants to keep compatibility gates honest

## Related docs

- [Compatibility policy](compatibility-policy.md)
- [JSON output](json-output.md)
- [Exit codes](exit-codes.md)
