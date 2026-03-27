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

The current `env` contract already supports:

- required values
- defaults
- allowed values
- validation in `doctor`
- default application in `run`

This spec adds the next layer: policy-controlled env resolution and injection.

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

## Proposed precedence

The exact precedence should be explicit and deterministic.

A good starting order is:

1. repo contract
2. workspace override
3. org policy source
4. runtime shell environment
5. declared default

The final precedence rules must be documented and stable.

## Proposed contract shape

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
