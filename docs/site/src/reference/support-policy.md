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

# Support Policy

This page defines the current platform support stance for ota V1.

Use it when you need to know what users can rely on today and where the boundaries are.

## Current stance

- Linux: first-class target
- macOS: first-class target
- Windows: supported, but with narrower shell and portability guarantees

First-class means ota aims for the best behavior there first. Supported does not mean identical
behavior across all shell conventions or task scripts.

## Shell semantics

Current task execution is shell-compatible:

- Unix-like systems: `sh -lc`
- Windows: `cmd /C`

This is explicit by design. V1 does not provide per-task shell selection.

See [shell-semantics.md](shell-semantics.md) for the full command model.

## Lifecycle semantics

Current lifecycle support is intentionally limited:

- `persistent` maps to the current shell-native execution model
- `ephemeral` is accepted, but advisory only in V1

ota does not yet create isolated temporary environments, temporary workspaces, or automatic
cleanup flows for `ephemeral`.

## Practical implication

Repos should expect the best behavior today on Linux and macOS.

Windows support exists, but shell behavior and script portability are more constrained in V1.

If your repo must support Windows, prefer commands that are explicit about platform differences,
or use task variants to keep one task name with different platform-specific bodies.

## Use cases

- a team needs to know whether a repo is really ready for Windows contributors
- a CI owner wants to understand whether `ephemeral` changes actual runtime isolation
- a maintainer wants to explain why Linux and macOS are first-class while Windows is constrained
- an agent needs the support boundary before suggesting platform-specific changes

## What this is not

- not a promise of identical shell behavior on every OS
- not per-task shell selection
- not automatic shell portability translation
- not isolated ephemeral workspace creation in V1
- not host cleanup or provisioning policy

## Relationship to other surfaces

- `contract` defines the repo’s execution intent
- `shell-semantics` defines the actual command invocation model
- `doctor` should explain support and compatibility failures clearly
- `run` and `up` should honor the platform boundary instead of hiding it
