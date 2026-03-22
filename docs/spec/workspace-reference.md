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

# Ota Workspace Reference

This document describes the current `ota.workspace.yaml` contract accepted by the shipped workspace validator.

## Purpose

`ota.workspace.yaml` is the canonical workspace bootstrap contract.

It is separate from `ota.yaml`, which remains the canonical repo readiness contract.

## Minimal contract

```yaml
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
```

## Top-level fields

- `version`: required, currently only `1`
- `workspace`: required workspace metadata
- `repos`: required map of repo entries

## `workspace`

```yaml
workspace:
  name: ota-dev
  description: Local multi-repo development workspace
```

Fields:

- `name`: required, non-empty string
- `description`: optional string

## `repos`

```yaml
repos:
  web:
    path: apps/web
  api:
    path: services/api
    contract: services/api/ota.yaml
    required: true
```

Fields:

- `path`: required path to a repo directory, relative to `ota.workspace.yaml`
- `contract`: optional explicit repo contract path, relative to `ota.workspace.yaml`
- `required`: optional boolean

Current validation behavior:

- repo names must not be empty
- workspace must declare at least one repo
- repo `path` must be non-empty
- repo `path` must exist and point to a directory
- `contract` must be non-empty when present
- if `contract` is omitted, Ota expects `<repo path>/ota.yaml`
- each referenced repo contract must load and pass repo-level validation

## Current scope

The shipped workspace surface is intentionally narrow:

- workspace contract parsing
- workspace contract validation
- repo contract validation through the workspace contract

Current non-goals:

- multi-repo `up`
- workspace task orchestration
- workspace-wide environment mutation
- hidden repo bootstrap behavior

## `ota workspace doctor`

Current workspace diagnosis behavior:

- validates workspace structure first
- diagnoses each referenced repo through its own `ota.yaml`
- preserves repo-level diagnosis semantics for required repos
- downgrades optional repo errors to warnings at the workspace layer

This keeps workspace behavior as orchestration over repo readiness, not a parallel readiness system.
