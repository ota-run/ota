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

# Adapters

Status: spec candidate.

This page lists the adapter families Ota can support for policy-backed provisioning.
It separates execution backends from source origins so the contract stays clear.

## Supported today

The built-in mutating provisioning adapters currently support:

- `mise`
- `asdf`
- `sdkman` for Java/runtime provisioning
- `uv` for Python/runtime provisioning
- `winget` for Windows package installs
- `choco` for Windows package installs
- `brew` for macOS host tooling
- `apt` for Debian and Ubuntu package installs

Use these when a policy entry should flow through the shipped installer path.

## Planned installer families

These adapter families are the next likely additions, not current support:

- `dnf` for Fedora and RHEL-style systems
- `pacman` for Arch-based systems
- `scoop` for Windows developer tooling

## Source adapters

Source adapters describe where an approved prerequisite comes from.

Useful source families include:

- internal mirror
- private registry
- approved vendor source
- artifact cache
- offline bundle
- enterprise package proxy

These are policy targets, not installer implementations. They tell Ota which origin is allowed,
not how the package manager itself works.

## Recommended order

The least surprising rollout order is:

1. prove one installer backend first
2. add one adapter family at a time
3. keep source adapters policy-driven and read-only until a matching installer exists
4. record the selected source in doctor output and receipts before broadening behavior

## What this is not

- not a general package manager list
- not a promise that every planned installer family is shipped today
- not a hidden workstation manager
- not a replacement for repo contracts, env policy, or checks
