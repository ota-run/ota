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

# Check: file workspace scope

Use this when a repo contract truthfully depends on a sibling workspace input and a shell check like
`test -f ../shared/schema.json` would otherwise be the only option.

This example shows the canonical shape:

- `checks[].kind: file`
- `checks[].scope: workspace`
- a relative sibling path such as `../task-sdk/schema.json`
- normal `expect: file` semantics without shell glue

Why this exists:

- some repo slices depend on checked-in inputs outside the repo subtree
- the contract should still use first-class file ownership instead of shell `run`
- repo-scoped file checks remain the default; widen only when sibling workspace truth is real

Open [`ota.yaml`](ota.yaml) for the exact contract shape.
