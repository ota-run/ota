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

# Docs Sync Policy

Policy:

- `docs/spec/*` is normative for command/schema/output behavior
- `docs/site/src/*` is curated public presentation
- when behavior changes, update both in the same PR

Preferred pattern:

1. Update normative reference in `docs/spec/*`.
2. Update corresponding public page in `docs/site/src/*`.
3. Keep concise “canonical reference” links from site pages back to `docs/spec/*`.

Drift prevention:

- do not duplicate long field-by-field tables in site pages unless necessary
- use summary + canonical link when details are already maintained in spec docs
- treat stale examples as bugs and fix them in the same change that detects drift
