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

# Audit and Provenance

This page explains how ota keeps readiness decisions explainable after the fact.

Use this page when you need to know:

- where a value came from
- whether a result was declared, inferred, or policy-derived
- how receipts and diagnosis stay reviewable
- why a command produced the result it did

## Why this matters

If ota says a repo is ready, users should be able to answer:

- what was declared
- what was inferred
- what came from policy
- what changed
- what the command used to decide

That is the difference between a useful automation surface and a black box.

## Provenance categories

The public docs and outputs should make it clear when a value is:

- repo-declared
- policy-derived
- template-derived
- detector-inferred
- user-mutated

That helps users separate source truth from derived truth.

## Where users see it

Provenance shows up in:

- `ota detect`
- `ota doctor`
- `ota explain`
- execution receipts
- policy-aware findings

Examples:

- `detect` can show where a field came from
- `doctor` can show why a repo is blocked
- `receipt` can show which env source won
- `explain` can show the remediation order and the reason behind each step

## Example finding

```text
ERROR  Missing required task
Why: The repo contract declares agent-safe execution but does not define a runnable task.
Next: add a task or adjust the contract.
Source: repo contract
```

The important part is not the exact wording. It is that the user can tell where the decision came
from and what to fix.

## Use cases

- a maintainer wants to understand why `doctor` reported a blocker
- a CI owner wants to see whether policy or repo data produced the result
- an agent needs the value source before suggesting a change
- a team wants execution receipts that are reviewable later

## What this is not

- not a general audit database
- not hidden mutation logging
- not a ticketing system
- not a replacement for the contract
- not a fleet reporting layer

## Related docs

- [Doctor findings](doctor-finding-contract.md)
- [Execution receipt](execution-receipt.md)
- [Env resolution and policy](env-resolution-and-policy.md)
- [JSON output](json-output.md)
- [Policy packs](policy-packs.md)
