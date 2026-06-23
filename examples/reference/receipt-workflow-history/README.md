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

# Receipt: workflow-scoped history

Use this when one repo declares more than one workflow and receipt archive or baseline truth
should stay inside the selected workflow lane instead of drifting to whichever receipt ran last.

This example shows the canonical shape:

- `workflows.default: verify`
- a separate `verify` workflow for finite validation
- a separate `app` workflow for the long-running service lane
- explicit `ota receipt --workflow <name>` archive, baseline, and snapshot commands

Open [`ota.yaml`](ota.yaml) for the exact contract shape.
