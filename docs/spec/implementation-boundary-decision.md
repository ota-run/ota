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

# Implementation Boundary Decision

This note records the current boundary decision so the shipped surface stays explicit.

## Decision

The extension taxonomy cutover is complete for this pass; no additional product implementation is
required right now.

The current shipped model stays:

- `extensions` as explicit `check_provider` / `export_provider` / `backend_provider` descriptors
- `exports` as shipped contract metadata for downstream generation intent
- `policies` as shipped contract metadata for repo-local policy overlays
- `readiness_gate` as a later-spec draft field that is not accepted by the current shipped parser
- the newer V6 extension provider taxonomy is now the current shipped taxonomy, with
  `backend_provider` discoverable but not yet directly executable through `ota extensions`

## Why

The repo already has a clear, working shipped boundary.
Changing it silently would create drift between:

- the parser
- the command behavior
- the public docs
- the newer spec corpus

That is not a useful product move right now.

## What to do instead

- Keep the current extension seam stable.
- Keep `exports` and `policies` documented as live but inert overlays.
- Keep `readiness_gate` deferred.
- Revisit backend-provider execution only when a task-execution integration slice is ready.

## Result

Users should not have to guess what is shipped.
Maintainers should not have to infer which future-spec terms are live.
The boundary is now explicit.
