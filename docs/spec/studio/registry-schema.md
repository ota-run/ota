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
   License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Ota Studio Registry Schema

Status: planned.

This document defines the persisted local repo registry used by Studio Home and repo switching.

## Location

Canonical path:

```text
~/.ota/studio/registry.json
```

The registry is local-user state, not repo state.
It is intentionally file-backed JSON, not a local database.

## Design rules

1. The registry is an index of known repos, not a second source of repo truth.
2. Contract and runtime truth still come from the repo and Ota core.
3. Registry writes must be atomic.
4. Corrupt registry state must degrade safely.
5. The registry must stay simple enough to inspect, back up, reset, or hand-edit when needed.

## Top-level shape

```json
{
  "schema_version": 1,
  "updated_at": "2026-05-02T12:55:00Z",
  "repos": []
}
```

Top-level fields:

- `schema_version`
- `updated_at`
- `repos`

## Repo entry identity

A repo entry is uniquely identified by:

- `repo_root`
- `contract_path`

This pair is required so Studio can support:

- default `ota.yaml`
- non-default contract file paths
- future repo variants that share a root but differ by explicit contract file

## Repo entry shape

Each `repos[]` entry should include:

- `repo_root`
- `contract_path`
- optional `project_name`
- `last_opened_at`
- optional `last_known_readiness`
- optional `last_known_activity`
- optional `last_known_contract_state`
- optional `favorite`
- optional `last_studio_version`
- optional `last_view`

### `last_known_readiness`

Canonical values:

- `ready`
- `blocked`
- `warning`
- `unknown`

### `last_known_activity`

Canonical values:

- `running`
- `failed`
- `ready`
- `idle`
- `unknown`

### `last_known_contract_state`

Canonical values:

- `present`
- `missing`
- `invalid`
- `unknown`

## Example

```json
{
  "schema_version": 1,
  "updated_at": "2026-05-02T12:55:00Z",
  "repos": [
    {
      "repo_root": "/work/acme/api",
      "contract_path": "/work/acme/api/ota.yaml",
      "project_name": "acme-api",
      "last_opened_at": "2026-05-02T12:54:22Z",
      "last_known_readiness": "blocked",
      "last_known_activity": "failed",
      "last_known_contract_state": "present",
      "favorite": false,
      "last_studio_version": "1",
      "last_view": "overview"
    }
  ]
}
```

## Update behavior

When `ota studio` opens a repo:

1. resolve `repo_root`
2. resolve the effective `contract_path`
3. insert or update the matching entry
4. refresh `last_opened_at`
5. refresh cheap known summaries when available

When no contract exists yet:

- Studio may still register the repo root
- `contract_path` should be omitted or null in the in-memory model until a contract exists
- the persisted registry should prefer only stable entries with a known repo root and a clear current
  contract situation

## Write discipline

Registry writes must:

- write to a temp file
- fsync if the platform path already does that elsewhere
- rename atomically into place

The registry is logically single-writer from the Studio server perspective, but clients must still
tolerate concurrent writers by:

- re-reading before write
- merging by `(repo_root, contract_path)`
- keeping the newest `last_opened_at`

## Corruption handling

If the registry is unreadable:

- Studio must not fail to open
- Studio should start with an empty in-memory registry
- Studio may move the corrupt file aside to `registry.corrupt.<timestamp>.json`
- Studio should write a new clean registry on next successful update

## Migration rules

Allowed schema evolution:

- additive optional fields within the same schema version

Breaking changes require:

- `schema_version` bump
- migration code or explicit reset behavior

If a future version cannot migrate safely, it may rebuild from the current repo and preserve only
entries it can still interpret correctly.
