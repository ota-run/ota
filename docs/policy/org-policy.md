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

# Recommended Org Policy Baseline

This is a solid starting policy for an org that wants ota to stay honest, explicit, and
repeatable across repos.

It keeps the contract discipline high without turning the policy pack into a hidden control plane.

## What it is

- require the core repo contract sections that make `doctor` and `up` useful
- require `AGENTS.md` so agent guidance stays visible and reviewable
- constrain runtime and tool versions explicitly with `version_policy`
- require explicit agent safety surfaces
- require `AGENTS.md` generation in exports
- approve source managers and adapter bootstrap explicitly for the platforms you actually use

## Why it matters

- `ota doctor` becomes a real governance check, not just a local readiness scan
- repo contracts stay explicit instead of drifting into shell scripts
- org-approved install sources stay reviewable by platform and by tool
- agent execution surfaces stay safe by default
- provisioning errors become policy issues instead of mysterious backend failures
- new repos can adopt the same baseline without inventing their own rules

## Baseline policy

Copy this into `.ota/org-policy.yaml` and tune versions to match your fleet:

```yaml
policies:
  required_sections:
    - runtimes
    - tasks
    - agent
  required_files:
    - AGENTS.md
  version_policy:
    runtimes:
      node:
        approved_versions:
          - "22"
      java:
        approved_versions:
          - "21"
        platforms:
          windows:
            approved_versions:
              - "21"
    tools:
      pwsh:
        platforms:
          windows:
            approved_versions:
              - "7.6.0"
  agent:
    require_safe_tasks: true
    require_writable_paths: true
  exports:
    require_agents_md: true
  provisioning:
    curl:
      source: brew
      approved_versions:
        - "8.7.1"
      platforms:
        macos:
          source: brew
          approved_versions:
            - "8.7.1"
        linux:
          source: apt
          approved_versions:
            - "8.7.1"
        windows:
          source: choco
          approved_versions:
            - "8.7.1"
    jq:
      source: brew
      approved_versions:
        - "1.7.1"
      platforms:
        macos:
          source: brew
          approved_versions:
            - "1.7.1"
        linux:
          source: apt
          approved_versions:
            - "1.7.1"
        windows:
          source: choco
          approved_versions:
            - "1.7.1"
    java:
      source: sdkman
      approved_versions:
        - "21"
      platforms:
        macos:
          source: sdkman
          approved_versions:
            - "21"
        linux:
          source: sdkman
          approved_versions:
            - "21"
        windows:
          source: choco
          approved_versions:
            - "21"
    maven:
      source: brew
      approved_versions:
        - "3.9.9"
      platforms:
        macos:
          source: brew
          approved_versions:
            - "3.9.9"
        linux:
          source: apt
          approved_versions:
            - "3.9.9"
        windows:
          source: choco
          approved_versions:
            - "3.9.9"
    node:
      source: brew
      approved_versions:
        - "22"
      platforms:
        macos:
          source: brew
          approved_versions:
            - "22"
        linux:
          source: apt
          approved_versions:
            - "22"
        windows:
          source: choco
          approved_versions:
            - "22"
    yq:
      source: brew
      approved_versions:
        - "4.52.5"
      platforms:
        macos:
          source: brew
          approved_versions:
            - "4.52.5"
        linux:
          source: apt
          approved_versions:
            - "4.52.5"
        windows:
          source: choco
          approved_versions:
            - "4.52.5"
  adapter_bootstrap:
    brew:
      source: brew-bootstrap
      approved_versions:
        - "4.4"
    choco:
      source: choco-bootstrap
      approved_versions:
        - "2.0.0"
    sdkman:
      source: sdkman-bootstrap
      approved_versions:
        - "1.0"
```

## Value

This policy gives you:

- one consistent baseline for repo readiness
- clearer source provenance in `doctor`
- explicit review gates for agent safety
- macOS/Linux provisioning that stays policy-driven
- adapter bootstrap that is separate from repo provisioning
- less room for silent host-first drift

It is intentionally opinionated, but still small enough to be understandable in a review.

## How to use it

1. Copy the baseline into `.ota/org-policy.yaml`.
2. Adjust versions to the installable versions in your approved sources.
3. Keep per-repo `ota.yaml` contracts focused on actual repo needs.
4. Let `ota doctor` tell you when the repo and org policy disagree.
