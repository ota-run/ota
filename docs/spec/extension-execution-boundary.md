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

# Extension Execution Boundary

This document defines the current implementation boundary for extensions during V6.

## Current boundary (shipped)

- Ota core commands do not execute extension providers at runtime.
- Top-level `extensions` in `ota.yaml` is parsed for discovery and inspection.
- Supported kinds today are `checker` and `publisher`.
- `ota extensions --run <name>` can execute one explicitly named `checker` descriptor with
  `api_version: 1`.
- `ota extensions --publish <name>` can execute one explicitly named `publisher` descriptor with
  `api_version: 1`.
- `ota doctor`, `ota check`, `ota run`, `ota up`, and `ota export` behavior remains core-only.

## Why this boundary exists

- preserve deterministic command behavior while compatibility contracts are locked
- avoid hidden runtime/plugin drift while V6 extension work is rolling out
- keep machine output and exit behavior stable

## Contract target

The normative extension contract target is:

- [21a-v6-extension-contract-normative.md](/Users/bobai/Desktop/Ota.run/Spec/new/21a-v6-extension-contract-normative.md)

Earlier compatibility and protocol work can prepare the surface, but runtime extension execution is
still constrained to the explicit `ota extensions --run <name>` seam.

## Enforcement in this repo

- validation accepts `extensions` as contract data today
- compatibility tests guard current JSON/exit contracts
- no command path should silently load or execute extension commands
