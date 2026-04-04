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
- required files
- safer agent task execution
- org-level template and convention enforcement
- audit-friendly machine output
- mutation controls for sensitive operations

## Target location

The canonical policy pack lives at:

```yaml
.ota/org-policy.yaml
```

## Policy path and discovery

Today, Ota looks for the org policy pack by walking up from the repo contract path and checking
for `.ota/org-policy.yaml` in each ancestor directory.

That means:

- a single policy pack can apply to multiple repos inside one workspace tree
- a repo can inherit an org policy from a parent directory
- the canonical policy pack lives at `.ota/org-policy.yaml`, so shared org rules have one deterministic place to live today
- `OTA_POLICY`, a single remote policy source, and arbitrary policy file names are future work

If there is no ancestor policy file, Ota simply keeps running with repo-local contract behavior.

## Future policy-source model

The current implementation is file-based only. A later policy-source model could support:

- a local file path
- an environment override such as `OTA_POLICY`
- a workspace-root policy file that applies to multiple repos
- one remote URL or hosted policy source, intended as an enterprise feature

The intended precedence for that future model should be:

1. explicit environment override
2. nearest ancestor `.ota/org-policy.yaml`
3. workspace-root policy file, if the workspace declares one
4. one explicitly configured remote policy source, if present

That order keeps the most explicit source first and leaves the current file-based behavior intact
until the model is implemented.

## Target shape

```yaml
policies:
  required_sections:
    - runtimes
    - tasks
    - agent
  required_files:
    - AGENTS.md
  strict_versions: true
  agent:
    require_safe_tasks: true
    require_writable_paths: true
  exports:
    require_agents_md: true
```

## Adoption walk-through

For a team that wants a quick rollout, the practical path is:

1. add `.ota/org-policy.yaml` at the org root or any ancestor of the governed repos
2. start with a small set of required sections and files
3. run `ota doctor` in one repo and compare the output before and after
4. expand policy only after the first rules are easy to understand

Example policy pack:

```yaml
policies:
  required_sections:
    - runtimes
    - tasks
  required_files:
    - AGENTS.md
  agent:
    require_safe_tasks: true
```

Before the policy pack exists, `ota doctor` only reports repo-local readiness.

After the policy pack is added, a repo missing `tasks` or `AGENTS.md` will show an org policy finding like:

```text
◉ ERROR  Repo does not satisfy org policy pack
Why: `./.ota/org-policy.yaml` requires missing contract sections: tasks and missing files: AGENTS.md
Next: add the missing items or update `./.ota/org-policy.yaml`
```

That makes the value visible immediately:

- the repo contract stays local and explicit
- the org policy stays shared and reusable
- `ota doctor` becomes the review point for both

## Semantics

- `required_sections` defines contract sections that every governed repo must provide.
- `required_files` defines files that every governed repo must keep at the repo root or under the governed repo directory.
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

`ota doctor` reads `.ota/org-policy.yaml` from the nearest ancestor when it exists, validates the file shape, and reports a finding if:

- the policy pack cannot be read or parsed
- required sections declared by the policy pack are missing from the repo contract
- required files declared by the policy pack are missing from the repo root

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
- waiver lifecycle management
- fleet-wide reporting or retention
