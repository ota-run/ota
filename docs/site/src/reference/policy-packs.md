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

Policy packs let a platform or operations team apply shared rules across many repos without
rewriting each repo’s own `ota.yaml`.

Use this page when you need:

- org-wide standards
- repo readiness rules that are shared across teams
- a clear difference between repo contract and policy overlay
- diagnosis that explains why a repo fails policy, not just local validation

## Why policy packs matter

Without policy packs, every repo has to encode the same requirements again and again.

That becomes a problem when a team needs:

- consistent agent safety rules
- required sections or files across repos
- org-level contract expectations
- an audit trail for policy-driven findings

Policy packs keep the repo contract local and explicit while letting the org layer add shared
constraints.

## Where they live

The canonical policy pack lives at:

```yaml
.ota/org-policy.yaml
```

That location keeps policy close to the repo while still separate from the repo contract.

## What a policy pack can do

Policy packs can require:

- contract sections
- files at the repo root or governed repo boundary
- safer agent guidance
- explicit writable-path intent
- generated repo guidance such as `AGENTS.md`

## What a policy pack cannot do

Policy packs do not replace `ota.yaml`.

They do not:

- define repo readiness on their own
- act as a workflow engine
- replace approvals or ticketing
- silently mutate repo files

## Example

```yaml
policies:
  required_sections:
    - runtimes
    - tasks
  required_files:
    - AGENTS.md
  agent:
    require_safe_tasks: true
    require_writable_paths: true
```

This says:

- every governed repo should declare runtimes and tasks
- every governed repo should include `AGENTS.md`
- agent-facing execution should be explicit about safe tasks and writable paths

## How it shows up in ota

`ota doctor` is the main place users see policy packs.

When a repo is missing a required section or file, `doctor` should explain:

- what policy asked for
- what is missing
- what to change first

That is better than hiding policy in a separate report because users can see the failure in the same
place they already check readiness.

## Use cases

- a platform team wants every repo to declare tasks and runtimes
- an org wants agent-safe execution rules across many repos
- a maintainer needs `AGENTS.md` to exist everywhere
- a compliance team wants policy findings to stay readable and reviewable

## Related docs

- [Contract](contract.md)
- [Audit and provenance](audit-and-provenance.md)
- [Compatibility policy](compatibility-policy.md)
