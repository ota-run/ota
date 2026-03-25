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

# Conventions and Templates

This document defines the V5 target for org conventions and templates in Ota.

The goal is to make shared repo structure predictable without replacing the repo contract.

## Purpose

Conventions and templates let a platform team standardize how repos start and how they are shaped over time.

They are intended to support:

- consistent starter contracts
- shared repo structure conventions
- audit-friendly baseline files
- agent guidance files that org policy can require

## Principles

- repo contracts remain the source of truth
- templates are derived, not authoritative
- conventions must be explicit and reviewable
- application of templates must be deterministic
- policy packs may require conventions, but do not replace them

## Target model

The canonical convention surface is expected to define:

- shared file layout guidance
- starter-contract templates
- required guidance files such as `AGENTS.md` where policy demands them
- org-level defaults that can be applied consistently across repos

## Example convention set

```yaml
conventions:
  required_files:
    - AGENTS.md
    - README.md
  starter_contract:
    path: ota.yaml
    kind: repo
  templates:
    repo:
      path: .ota/templates/repo.yaml
    workspace:
      path: .ota/templates/workspace.yaml
```

## Semantics

- `required_files` defines baseline files that should exist for governed repos.
- `starter_contract` defines the repo contract shape that a template targets.
- `templates.repo` and `templates.workspace` are derived scaffolds, not sources of truth.
- policy packs may require a repo to conform to one or more conventions, including required files like `AGENTS.md`.

## Scope

Conventions and templates are for:

- standard repo onboarding
- org-level starter consistency
- agent-safe repo scaffolding
- auditable repo shape guidance

They are not for:

- hidden contract mutation
- broad workflow orchestration
- replacing `ota.yaml`
- turning Ota into a generic project generator
