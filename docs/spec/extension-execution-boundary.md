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

- ota core commands do not execute extension providers at runtime.
- Top-level `extensions` in `ota.yaml` is parsed for discovery and inspection.
- Supported kinds today are `check_provider`, `export_provider`, and `backend_provider`.
- `ota extensions --run <name>` can execute one explicitly named `check_provider` descriptor with
  `api_version: 1`.
- `ota extensions --publish <name>` can execute one explicitly named `export_provider` descriptor with
  `api_version: 1`.
- `backend_provider` is parsed and preserved for discovery, and can be selected by
  `execution.backends.remote.provider` for custom remote execution.
- backend providers receive a structured JSON request on stdin and via
  `OTA_BACKEND_PROVIDER_REQUEST_JSON`, then return a structured JSON response on stdout.
- the request includes:
  - `extension_id`
  - `extension_kind`
  - `api_version`
  - `command_context` (`run` for task execution today)
  - `repo_context_path`
  - `working_dir`
  - `task.name`
  - `task.command`
  - `task.mode`
  - `task.target`
  - `task.cwd`
  - `task.environment`
- the response includes:
  - `ok`
  - `result`
  - `errors`
- the result object includes:
  - `exit_code`
  - `stdout`
  - `stderr`
  - `target`
- shell adapters may also read these env vars:
  - `OTA_BACKEND_PROVIDER_NAME`
  - `OTA_BACKEND_PROVIDER_KIND`
  - `OTA_BACKEND_PROVIDER_API_VERSION`
  - `OTA_BACKEND_PROVIDER_TARGET`
  - `OTA_BACKEND_PROVIDER_COMMAND`
  - `OTA_BACKEND_PROVIDER_WORKDIR`
  - `OTA_BACKEND_PROVIDER_CWD`
  - `OTA_BACKEND_PROVIDER_MODE`
  - `OTA_BACKEND_PROVIDER_REQUEST_JSON`

Example request:

```json
{
  "extension_id": "backend-demo",
  "extension_kind": "backend_provider",
  "api_version": 1,
  "command_context": "run",
  "repo_context_path": "/workspace/repo",
  "working_dir": "/workspace/repo",
  "task": {
    "name": "setup",
    "command": "echo backend-provider-run",
    "mode": "capture",
    "target": "sandbox-dev",
    "cwd": "/workspace",
    "environment": {
      "TASK_TOKEN": "sample-token",
      "OTA_BACKEND_PROVIDER_NAME": "backend-demo"
    }
  }
}
```

Example response:

```json
{
  "ok": true,
  "result": {
    "exit_code": 0,
    "stdout": "backend-provider-run",
    "stderr": "",
    "target": "sandbox-dev"
  },
  "errors": []
}
```

- `ota doctor`, `ota check`, `ota run`, `ota up`, and `ota export` behavior remains core-only.

## Why this boundary exists

- preserve deterministic command behavior while compatibility contracts are locked
- avoid hidden runtime/plugin drift while V6 extension work is rolling out
- keep machine output and exit behavior stable

## Contract target

The normative extension contract target is defined in the newer V6 spec corpus, and this repo keeps
the shipped boundary centered on the current `check_provider`, `export_provider`, and
`backend_provider` contract seam.

Earlier compatibility and protocol work can prepare the surface, but runtime extension execution is
still constrained to the explicit `ota extensions --run <name>` and `ota extensions --publish
<name>` seams in the current implementation.

## Enforcement in this repo

- validation accepts `extensions` as contract data today
- compatibility tests guard current JSON/exit contracts
- no command path should silently load or execute extension commands
