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
  git_base: https://github.com/ota
repos:
  web:
    path: apps/web
    source:
      repo: web
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
- `git_base`: optional clone base used by `repos.<name>.source.repo`

## `repos`

```yaml
repos:
  web:
    path: apps/web
    source:
      repo: web
  api:
    path: services/api
    contract: services/api/ota.yaml
    required: true
    depends_on:
      - web
    source:
      git: https://github.com/ota/api.git
      ref: main
```

Fields:

- `path`: required path to a repo directory, relative to `ota.workspace.yaml`
- `contract`: optional explicit repo contract path, relative to `ota.workspace.yaml`
- `required`: optional boolean
- `depends_on`: optional list of workspace repo names
- `source`: optional acquisition source for repos that are not present yet

`source` fields:

- `git`: explicit clone URL or git-accepted clone source
- `repo`: repo path or slug resolved against `workspace.git_base`
- `ref`: optional branch, tag, or ref to checkout after clone

Design intent:

- `source.git` is the canonical acquisition field
- `source.repo` is shorthand for multiple repos sharing the same `workspace.git_base`
- both are generic git concepts and work for GitHub, GitLab, Bitbucket, and internal git hosts

Current validation behavior:

- repo names must not be empty
- workspace must declare at least one repo
- repo `path` must be non-empty
- repo `path` must exist and point to a directory unless `source` is declared
- `contract` must be non-empty when present
- if `contract` is omitted, Ota expects `<repo path>/ota.yaml`
- `source` must declare exactly one of `git` or `repo`
- `source.repo` requires `workspace.git_base`
- `depends_on` references must resolve to known workspace repos
- workspace repo dependency cycles are rejected
- each present repo contract must load and pass repo-level validation

## Current scope

The shipped workspace surface is intentionally narrow:

- workspace contract parsing
- workspace contract validation
- repo contract validation through the workspace contract
- workspace-level diagnosis as orchestration over repo-level `doctor`
- workspace-level prepare flow as orchestration over repo-level `up`
- acquisition of missing repos before workspace prepare

Current non-goals:

- workspace task orchestration
- workspace-wide environment mutation
- hidden repo bootstrap behavior
- passing a repo URL directly on the CLI without a workspace contract
- GitHub API integration or non-git acquisition modes

## `ota workspace doctor`

Current workspace diagnosis behavior:

- validates workspace structure first
- evaluates repos in dependency order
- can diagnose independent repos concurrently when `--jobs` is greater than `1`
- preserves deterministic repo ordering in the final report
- diagnoses each referenced repo through its own `ota.yaml`
- reports missing-but-acquirable repos as not yet acquired instead of treating them as unreadable local paths
- preserves repo-level diagnosis semantics for required repos
- downgrades optional repo errors to warnings at the workspace layer
- rejects required repos that depend on optional repos

This keeps workspace behavior as orchestration over repo readiness, not a parallel readiness system.

## `ota workspace up`

Current workspace prepare behavior:

- validates workspace structure first
- acquires missing repos declared with `source` before repo-level bootstrap
- runs repo-level `up` for each referenced repo
- can prepare independent repos concurrently when `--jobs` is greater than `1`
- respects declared workspace repo dependency order
- blocks downstream repos when a dependency does not become ready
- aggregates repo-level status, phase, findings, and exit details
- captures repo child stdout and stderr per repo so the final report remains deterministic
- emits live repo progress on stderr in text mode so users can see execution moving without losing ordered final output
- optional repo failures do not fail the overall workspace status
- `--stream` opts into raw live child process output instead of buffered per-repo output

Current execution policy:

- workspace repo execution defaults to sequential because `--jobs` defaults to `1`
- Ota only parallelizes repos whose dependencies are already satisfied
- final reporting remains in deterministic repo order even when execution is concurrent
- required repos must not depend on optional repos, because required readiness cannot rest on optional guarantees
- `--stream` is currently text-only and requires `--jobs 1` so raw child logs do not interleave

Current non-goals:

- cross-repo dependency scheduling
- passing a repo URL directly on the CLI without a workspace contract
- host or workstation provisioning
- a workspace-only bootstrap engine that bypasses repo contracts
- implicit pull, fetch, or update behavior for repos that already exist locally
- GitHub API integration or non-git acquisition modes

## `ota workspace refresh`

Current workspace refresh behavior:

- validates workspace structure first
- refreshes repos that already exist locally and have a declared source
- leaves missing repos alone so `ota workspace up` remains the bootstrap path
- can refresh independent repos concurrently when `--jobs` is greater than `1`
- respects declared workspace repo dependency order
- blocks downstream repos when a dependency does not become ready
- aggregates repo-level status, phase, findings, and exit details
- captures repo child stdout and stderr per repo so the final report remains deterministic
- emits live repo progress on stderr in text mode so users can see execution moving without losing ordered final output
- optional repo failures do not fail the overall workspace status
- `--stream` opts into raw live child process output instead of buffered per-repo output

Current execution policy:

- workspace repo execution defaults to sequential because `--jobs` defaults to `1`
- Ota only parallelizes repos whose dependencies are already satisfied
- final reporting remains in deterministic repo order even when execution is concurrent
- required repos must not depend on optional repos, because required readiness cannot rest on optional guarantees
- `--stream` is currently text-only and requires `--jobs 1` so raw child logs do not interleave

Current non-goals:

- cloning missing repos
- cross-repo dependency scheduling
- passing a repo URL directly on the CLI without a workspace contract
- host or workstation provisioning
- a workspace-only bootstrap engine that bypasses repo contracts
- implicit pull, fetch, or update behavior for repos that already exist locally
- GitHub API integration or non-git acquisition modes
