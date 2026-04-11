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

This document defines the V5 policy-pack target contract for ota.

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

## Target Location

The canonical policy pack lives at:

```yaml
.ota/org-policy.yaml
```

## Policy Path and Discovery

Today, ota resolves the org policy pack in this order:

1. the explicit `OTA_POLICY` file path or HTTP(S) URL override, when set
2. `.ota/org-policy.yaml` in the nearest ancestor directory of the repo contract path
3. the `workspace.policy` path declared in the nearest ancestor `ota.workspace.yaml`, when set

That means:

- a single policy pack can apply to multiple repos inside one workspace tree
- a repo can inherit an org policy from a parent directory
- `OTA_POLICY` gives operators an explicit override when they need a different file path or hosted URL
- the canonical policy pack still lives at `.ota/org-policy.yaml`, so shared org rules have one deterministic place to live today
- `workspace.policy` gives a workspace an explicit shared policy file or hosted URL when the workspace wants one

If there is no matching `OTA_POLICY`, ancestor policy file, or workspace policy source, ota simply
keeps running with repo-local contract behavior.

Example override:

```bash
OTA_POLICY=/path/to/custom-org-policy.yaml ota doctor
```

```bash
OTA_POLICY=https://config.example.com/custom-org-policy.yaml ota doctor
```

Use this when you want ota to read an explicit policy file path or hosted URL instead of the
nearest ancestor `.ota/org-policy.yaml` or workspace policy source.

## Current Policy-Source Model

The current implementation supports:

- an explicit `OTA_POLICY` override that can be a local file path or an HTTP(S) URL
- the nearest ancestor `.ota/org-policy.yaml`
- a workspace policy source declared in `ota.workspace.yaml`

The precedence is:

1. explicit environment override
2. nearest ancestor `.ota/org-policy.yaml`
3. workspace policy source, if the workspace declares one

If none of those sources exist, ota continues without org policy.

That order keeps the most local repo policy first while still letting a workspace declare one shared
policy source when it needs to.

## Future Policy-Source Model

One separately configured remote policy source remains future work.

## Target Shape

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

## Adoption Walk-Through

For a team that wants a quick rollout, the practical path is:

1. preview a starter pack with `ota policy init --dry-run`
2. write `.ota/org-policy.yaml` with `ota policy init`
3. start with a small set of required sections and files
4. run `ota doctor` in one repo and compare the output before and after
5. expand policy only after the first rules are easy to understand

`ota policy init` is deliberately conservative: it writes the minimal valid starter
`policies: {}` and does not guess provisioning approvals or org intent.

When the policy owner wants a stronger starting point, `ota policy init` also supports explicit
presets:

- `--preset required-sections` to start with `runtimes` and `tasks`
- `--preset provisioning` to scaffold empty provisioning and adapter-bootstrap maps with inline examples
- `--preset agent` to require safe tasks, writable-path intent, and `AGENTS.md`

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

## Enforcement Model

Policy packs are intended to be:

- deterministic
- explicit
- additive to repo contracts
- visible in diagnosis
- non-mutating by default

The policy pack does not replace `ota.yaml`.
It constrains and interprets it at the org layer.

## Current Implementation

`ota doctor` reads the explicit `OTA_POLICY` file path or HTTP(S) URL when set, otherwise it reads `.ota/org-policy.yaml`
from the nearest ancestor when it exists, validates the file shape, and reports a finding if:

- the policy pack cannot be read or parsed
- required sections declared by the policy pack are missing from the repo contract
- required files declared by the policy pack are missing from the repo root
- `policies.adapter_bootstrap` is malformed

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
