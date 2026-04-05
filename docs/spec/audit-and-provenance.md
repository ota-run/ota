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

This document defines the V5 target for audit-friendly machine output and provenance semantics in ota.

The goal is to make org policy, templates, and repo readiness explainable after the fact without guessing where the result came from.

## Purpose

Audit and provenance support:

- compliance review
- org policy enforcement
- reproducible diagnosis
- agent-safe decision tracing
- trust in inferred vs declared values

## Core principles

- declared contract state remains the source of truth
- derived output must stay clearly derived
- provenance must be visible for inferred or policy-derived values
- audit data should be stable and machine-readable
- mutation paths should leave an observable trail

## Provenance model

The current shipped output already uses per-field provenance in detection and diagnosis.
V5 extends that idea to org policy and template-derived behavior.

The intended provenance categories are:

- repo-declared
- policy-derived
- template-derived
- detector-inferred
- user-mutated

Policy-backed provisioning should use the same provenance idea once source-selection lands:
the repo declares what it needs, policy declares where it may come from, and the receipt records
which approved source won.

## Signed config direction

Signed config is an optional trust mechanism for environments that need stronger integrity guarantees.

The target behavior is:

- signed `ota.yaml` can be recognized as trusted source material
- unsigned or altered files remain valid unless policy says otherwise
- signatures do not replace validation or diagnosis
- signing is an additive trust layer, not a separate contract format

## Audit output expectations

Audit-friendly output should answer:

- what was declared
- what was inferred
- what was derived from policy
- what was changed
- why a command considered the result ready or not ready

## Scope

Audit/provenance is for:

- explainable diagnosis
- reviewable detection output
- policy pack traceability
- template application traceability

It is not for:

- hidden mutation logging
- a general audit database
- approval workflow orchestration
- replacing the contract with runtime state
- hosted control plane operations
- fleet-wide reporting or retention policy
