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

# Commands

Repo commands:

- `ota validate`
- `ota tasks`
- `ota run <task>`
- `ota doctor`
- `ota init`
- `ota detect`
- `ota check`
- `ota up`
- `ota clean`

Workspace commands:

- `ota workspace validate`
- `ota workspace tasks`
- `ota workspace run <task>`
- `ota workspace check`
- `ota workspace doctor`
- `ota workspace up`

Operational guidance:

- Use `ota doctor` first for readiness diagnosis.
- Use `ota up` to move from diagnosed to runnable.
- Use `ota run <task>` for deterministic task execution once ready.
- Use `ota detect --dry-run` before any contract write.
- Use `ota detect --merge --dry-run` before any merge write.

JSON-capable commands:

- `ota validate --json`
- `ota tasks --json`
- `ota doctor --json`
- `ota check --json`
- `ota up --json`
- `ota init --json`
- `ota detect --json`
- `ota workspace validate --json`
- `ota workspace tasks --json`
- `ota workspace run --json`
- `ota workspace check --json`
- `ota workspace doctor --json`
- `ota workspace up --json`

For full option-level reference, use:

- `ota --help`
- `ota <command> --help`

Canonical command reference in repository:

- `docs/spec/command-reference.md`
- <https://github.com/ota-run/ota/blob/main/docs/spec/command-reference.md>
