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

# Exit Codes

Ota uses stable exit code behavior for scripting and CI.

When to use:

- any automated integration where command success/failure must be deterministic

Why:

- text output can vary for humans, but exit codes remain the control signal for machines

Common cases:

- `0` command succeeded
- `1` command failed due to validation, readiness, runtime, or task failure
- `2` command usage error (for example invalid flag combinations or duplicate member flags)

Use-case:

- in CI, treat `1` as contract/readiness failure and `2` as pipeline misconfiguration.

For full command-by-command semantics, use the canonical reference in:

- `docs/spec/exit-codes.md` in the repository
- <https://github.com/ota-run/ota/blob/main/docs/spec/exit-codes.md>
