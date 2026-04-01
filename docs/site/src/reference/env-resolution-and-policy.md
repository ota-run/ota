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

# Env Resolution and Policy

ota already supports declaring env requirements in `ota.yaml`.

This future surface adds policy-controlled resolution and injection so repos and
workspaces can explain where env values came from without becoming a full app config
system.

## Source model

This page is the canonical public reference for env resolution and policy. It
adds examples, use cases, and operator guidance so the page stands on its own
while staying aligned with shipped behavior.

Use it for:

- required runtime env
- approved source resolution
- consistent execution injection
- provenance-aware receipts

## Current baseline

The shipped contract already supports:

- required values
- defaults
- allowed values
- validation in `doctor`
- default application in `run`

Policy should extend, not replace, that baseline.

## Resolution model

Resolution should be deterministic and layer-aware.

For task execution, the recommended precedence is:

1. task-scoped overrides
2. member contract values
3. workspace contract values
4. repo contract values
5. org policy values
6. shell process environment
7. declared defaults

The policy layer must not silently rewrite repo-declared truth. It may only supply approved
values, explain why they won, and leave a provenance trail.

Runtime and tool resolution follow the same inheritance principle:

- repo declarations remain canonical for required versions and tool names
- workspace overlays may tighten or specialize member expectations
- policy may provide approved defaults or provisioning hints
- provenance must record which layer supplied the final value

## Example

Repo contract:

```yaml
env:
  DATABASE_URL:
    required: true
  JAVA_HOME:
    required: true
```

Policy-controlled values:

```yaml
policies:
  env:
    JAVA_HOME:
      source: org-default
      value: /opt/jdk-22
    AWS_PROFILE:
      source: approved-profile
      value: ota-prod
```

In this shape:

- `DATABASE_URL` still needs to come from the repo, task, or shell
- `JAVA_HOME` can be injected from policy if the repo allows it
- `AWS_PROFILE` can be supplied from an approved org source

## Use cases

- a repo needs `JAVA_HOME` or `DATABASE_URL` to run correctly
- an org wants `AWS_PROFILE` or `GOOGLE_APPLICATION_CREDENTIALS` sourced from an approved policy
- a workspace wants consistent env injection across repos without hardcoding secrets into each repo
- `ota run` and `ota up` need to explain exactly where env came from

## How to use it

Start by declaring the env requirement in `ota.yaml`:

```yaml
env:
  DATABASE_URL:
    required: true
  JAVA_HOME:
    required: true
    default: /opt/jdk-22
```

Then, if your org wants to provide approved values, add a policy layer:

```yaml
policies:
  env:
    JAVA_HOME:
      source: org-default
      value: /opt/jdk-22
```

Practical flow:

1. run `ota doctor` to see which env values are missing or inherited
1. run `ota run test` or `ota up` to confirm the resolved env actually works
1. inspect the execution receipt or JSON output when you need provenance

Example workflow:

```bash
ota doctor
ota run test
ota up
```

If the repo needs a policy-provided value, the output should tell you:

- which value won
- which layer supplied it
- whether the value came from the repo, workspace, policy, or shell

## What policy is not

- not a replacement for `.env`
- not a general application config system
- not silent env mutation
- not unapproved source resolution
- not hosted control-plane workflow logic
- not waiver or approval orchestration
- not fleet reporting or retention policy

## Relationship to other surfaces

- `doctor` diagnoses missing or invalid env
- `run` and `up` consume env
- `diff` should show env requirement impact
- `explain` should turn env failures into a fix plan
- receipts should record which env source won
- `workspace doctor` should preserve root/member provenance instead of flattening it
