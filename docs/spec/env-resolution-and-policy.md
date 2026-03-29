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

Status: spec candidate.

This document defines the planned env resolution layer for Ota.

This spec adds the next layer: policy-controlled env resolution and injection.

## Current baseline

The shipped contract already supports:

- required values
- defaults
- allowed values
- validation in `doctor`
- default application in `run`

The policy layer described here should extend, not replace, that baseline.

## Goal

Ota should help determine:

- which env values are required
- which values may be provided by policy-controlled sources
- which values must remain explicit in the shell or repo contract
- which values should be injected into execution commands

## Scope

This surface should remain narrow and operability-focused.

It should support:

- validating presence, non-empty state, and allowed values
- injecting env into `ota run` and `ota up`
- resolving env from approved sources under org policy
- reporting provenance for resolved env values
- keeping workspace and repo inheritance deterministic

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

The policy layer must not silently rewrite repo-declared truth. It may only supply
approved values, explain why they won, and leave a provenance trail.

Runtime and tool resolution follow the same inheritance principle:

- repo declarations remain canonical for required versions and tool names
- workspace overlays may tighten or specialize member expectations
- policy may provide approved defaults or provisioning hints
- provenance must record which layer supplied the final value

## Contract shape

The `env` section should continue to describe requirements, while policy may add
resolution metadata.

Examples of requirement fields:

- `required`
- `secret`
- `default`
- `allowed`

Future policy-controlled resolution may add:

- approved source references
- source provenance
- injection hints
- optional fallback rules

Workspace-level policy should remain additive. It can describe shared defaults or approved
sources for member repos, but it should not become a second repo contract.

## Provenance

Resolution output should identify the winning source for each value using the existing
audit/provenance vocabulary:

- repo-declared
- policy-derived
- template-derived
- detector-inferred
- user-mutated

For env resolution, `doctor`, `detect`, and execution receipts should explain:

- which value won
- which layer supplied it
- why lower-priority candidates lost
- whether the result is safe to reuse or only safe to run

## Use cases

- a repo needs `JAVA_HOME` or `DATABASE_URL` to run correctly
- an org wants `AWS_PROFILE` or `GOOGLE_APPLICATION_CREDENTIALS` sourced from an approved policy
- a workspace wants consistent env injection across repos without hardcoding secrets into each repo
- `ota run` and `ota up` need to explain exactly where env came from

## Non-goals

- replacing `.env` as a general application config system
- owning all app settings or secrets management
- silently mutating env values
- resolving from unapproved sources

## Relationship to other surfaces

- `doctor` diagnoses missing or invalid env
- `run` and `up` consume env
- `diff` should show env requirement impact
- `explain` should turn env failures into a fix plan
- receipts should record which env source won
- `workspace doctor` should preserve root/member provenance instead of flattening it
