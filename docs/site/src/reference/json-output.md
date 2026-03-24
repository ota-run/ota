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

# JSON Output

Ota supports machine-readable JSON for core commands and workspace commands.

When to use:

- CI pipelines
- editor tooling
- agent workflows
- scripts that need stable parsing

JSON output is part of Ota’s integration contract for:

- CI
- editor integrations
- agent tooling

Design intent:

- JSON shapes are treated as stable integration surfaces.
- Human text output and JSON output are intentionally separate.
- Exit code behavior and JSON payloads should be consumed together in automation.

Common patterns:

- success payloads include `ok: true`
- failure payloads include `ok: false` and structured error/findings context
- workspace commands include per-repo result objects when applicable

## Practical integration pattern

For each command execution in automation:

1. run with `--json`
1. check process exit code first
1. parse payload fields (`ok`, `errors`, `findings`, per-repo reports)

Use-case:

- a CI job runs `ota doctor --json`, fails on errors, and posts warnings as annotations.

Canonical JSON references in repository:

- `docs/spec/json-output-reference.md`
- `docs/spec/json-schemas/`
- <https://github.com/ota-run/ota/tree/main/docs/spec/json-schemas>
