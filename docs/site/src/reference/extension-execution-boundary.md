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

# Extension Execution Boundary

This page explains where ota stops today and what extension behavior is explicit versus implicit.

Use it when you need to know whether a declared extension will run automatically or only through an
explicit seam.

## Why this matters

Extensions are useful only if the execution boundary is honest.

Users need to know:

- whether core commands run extensions automatically
- how to invoke an extension explicitly
- what kinds of extension descriptors are supported today
- what stays core-only

That keeps command behavior deterministic and reviewable.

## Current boundary

Today, ota core commands do not execute extension providers at runtime.

What ota does today:

- parses top-level `extensions` as contract data
- exposes descriptors for inspection
- supports explicit execution through `ota extensions --run <name>`
- supports explicit publication through `ota extensions --publish <name>`

Supported kinds today are:

- `checker`
- `publisher`

## What core commands do not do

Core commands do not silently load or execute extension commands.

That means:

- `ota doctor` stays core-focused
- `ota run` stays task-focused
- `ota up` stays repo readiness focused
- `ota check` stays check-focused

If a user wants extension behavior, they must opt into the explicit seam.

## Example

```yaml
extensions:
  release-upload:
    kind: publisher
    command: ota-ext-upload
    api_version: 1
    description: Upload the release bundle to the artifact endpoint
    config:
      endpoint: https://artifacts.example.com/upload
      artifact: dist/release.zip
```

That example is useful because it shows:

- the descriptor is discoverable in the contract
- the runtime command is explicit
- config stays attached to the descriptor

## Use cases

- a team wants a checker to exist as contract data without becoming hidden runtime behavior
- a release flow needs a clearly named publish seam
- an agent or CI job needs to inspect extension intent before running it

## What this is not

- not automatic plugin execution
- not a hidden extension framework
- not a replacement for `tasks`
- not a general-purpose runtime plugin host

## Related docs

- [Contract](contract.md)
- [Commands](commands.md)
