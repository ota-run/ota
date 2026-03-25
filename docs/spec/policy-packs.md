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

# Policy Packs

This document defines the V5 policy-pack target contract for Ota.

Policy packs are org-scoped rules that apply consistently across multiple repos without changing each repo’s source contract shape.

## Purpose

Policy packs let a platform team define shared standards once and apply them deterministically across repos.

They are intended to support:

- required contract sections
- safer agent task execution
- org-level template and convention enforcement
- audit-friendly machine output
- mutation controls for sensitive operations

## Target location

The canonical policy pack lives at:

```yaml
.ota/org-policy.yaml
```

## Target shape

```yaml
policies:
  required_sections:
    - runtimes
    - tasks
    - agent
  strict_versions: true
  agent:
    require_safe_tasks: true
    require_writable_paths: true
  exports:
    require_agents_md: true
```

## Semantics

- `required_sections` defines contract sections that every governed repo must provide.
- `strict_versions` requires the repo contract to stay on the declared contract version family.
- `agent.require_safe_tasks` requires agent-visible execution surfaces to be explicitly marked safe.
- `agent.require_writable_paths` requires writable-path intent to be declared instead of assumed.
- `exports.require_agents_md` requires repo-side agent guidance to be present when the policy pack says so.

## Enforcement model

Policy packs are intended to be:

- deterministic
- explicit
- additive to repo contracts
- visible in diagnosis
- non-mutating by default

The policy pack does not replace `ota.yaml`.
It constrains and interprets it at the org layer.

## Current implementation

`ota doctor` reads `.ota/org-policy.yaml` when it exists, validates the file shape, and reports a finding if:

- the policy pack cannot be read or parsed
- required sections declared by the policy pack are missing from the repo contract

The current implementation is read-only. It does not mutate repo contracts or apply policy remediation automatically.

## Scope

Policy packs are for:

- repo readiness governance
- org-wide standards
- policy-aware diagnosis
- audit and compliance support

They are not for:

- a general-purpose workflow engine
- arbitrary org RBAC design
- ticketing or approval orchestration
- hidden mutation behavior
