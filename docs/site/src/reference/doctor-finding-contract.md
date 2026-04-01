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

# Doctor Findings

This page explains how to read ota doctor findings.

Use it when you need to know:

- what a finding means
- why a repo is blocked or warned
- which fields are stable for CI and agents
- how to read `doctor`, `workspace doctor`, and `explain` output

## Why findings matter

`ota doctor` is only useful if the result is explainable.

Users need to know:

- what failed
- why it matters
- what to do next
- what part of the contract or host produced the finding

That is how ota stays trustworthy instead of becoming another noisy checklist.

## What a finding contains

Every finding includes:

- `severity`
- `code`
- `category`
- `owner`
- `summary`
- `why`
- `next`
- `evidence`

## What those fields mean

### `severity`

How serious the finding is.

- `error` blocks readiness
- `warn` is important but not blocking
- `info` is advisory

### `code`

A stable machine-readable identifier for the condition.

Use it when you need to group or filter findings without depending on wording.

### `category`

The broad area the finding belongs to.

Examples include:

- contract
- execution
- policy
- service
- environment
- remote
- workspace

### `owner`

Who should act on the finding.

Examples include:

- `repo_contract`
- `host`
- `service`
- `workspace_acquisition`
- `org_policy`
- `remote_backend`
- `agent_safety`

### `summary`

A short headline for humans.

### `why`

The reason the finding exists and why it matters.

### `next`

The safest next action to clear or reduce the finding.

### `evidence`

The structured facts behind the finding.

It usually includes:

- what ota observed
- what ota expected
- where the decision came from
- when ota checked
- which command was involved
- which path was involved

## How users should read findings

Read a finding in this order:

1. `severity`
2. `summary`
3. `why`
4. `next`
5. `evidence`

That sequence gives the fastest path from “what is wrong” to “what should I do.”

## Why the stable fields matter

The stable fields are what make findings useful in CI and automation.

You should depend on:

- `code`
- `category`
- `owner`

You should not depend on summary wording staying identical forever.

## Practical examples

### Blocking contract issue

```text
ERROR  No tasks defined
Why: The repo contract is not yet runnable.
Next: add a task such as `setup`, `test`, or `build`.
```

### Warning-only service issue

```text
WARNING  Required service has no healthcheck
Why: ota cannot verify readiness without a healthcheck.
Next: add a healthcheck command to the service contract.
```

### Policy finding

```text
ERROR  Repo does not satisfy org policy pack
Why: The policy pack requires `AGENTS.md` and this repo does not have it.
Next: add the missing file or update the policy pack.
```

## Use cases

- a maintainer wants the fastest fix for a blocked repo
- a CI gate wants stable machine identifiers for findings
- an agent wants to know whether a finding is contract, service, policy, or host related
- a workspace owner wants per-repo findings without reading raw JSON

## Related docs

- [Commands](commands.md)
- [Audit and provenance](audit-and-provenance.md)
- [JSON output](json-output.md)
- [Hosted validation](hosted-validation.md)
