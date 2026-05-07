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
   License is distributed on an AS IS BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Ota Studio Spec Set

Status: planned.

This directory is the canonical specification set for the Ota Studio app.

Studio is not a separate product and must not become a parallel truth system.
Studio is the visual operational client of Ota:

- `ota.yaml` remains the source of truth
- Ota core remains the execution and diagnosis engine
- Studio renders, reviews, triggers, and observes that same truth

The earlier static Studio MVP proved the boundary and data-contract direction. This spec set now
defines one supported future Studio surface: interactive, local-first, repo-aware, operational,
and premium enough to stand as a first-class Ota client.

## Documents

- [product-spec.md](product-spec.md)
  - product goals
  - user experience model
  - feature set
  - commands and behavior boundaries
  - non-goals

- [architecture.md](architecture.md)
  - local server model
  - repo registry
  - operation and event model
  - state and storage
  - permissions and trust boundaries
  - integration points with Ota core

- [roadmap.md](roadmap.md)
  - phased delivery plan
  - acceptance criteria
  - cut lines and non-goals per phase

- [phase1-build-plan.md](phase1-build-plan.md)
  - concrete Phase 1 implementation order
  - deliverables and file touch points
  - backend/frontend split
  - acceptance and validation checklist

- [cleanup-plan.md](cleanup-plan.md)
  - static Studio retirement plan
  - salvage-versus-delete rules
  - migration order into the interactive shell

- [event-schema.md](event-schema.md)
  - operation ids
  - event envelope
  - event kinds and payload fields
  - source metadata and evolution rules

- [registry-schema.md](registry-schema.md)
  - Studio repo registry path and shape
  - repo identity rules
  - write/update semantics
  - corruption and migration behavior

- [http-api.md](http-api.md)
  - localhost server contract
  - endpoint families
  - request/response rules
  - action and event transport boundaries

- [design-system.md](design-system.md)
  - app shell layout
  - visual hierarchy
  - pane and component rules
  - interaction and motion rules

## Source direction

This spec set is driven by five product truths:

1. Studio should be interactive and serve-first as the only supported Studio product surface.
2. Studio should feel like a premium local app, not a rendered report.
3. Studio should handle repo review, execution, and observation in one place.
4. Studio should stay deterministic by consuming Ota-owned contract, JSON, and event surfaces.
5. Studio should grow as one Ota product surface, not drift into a second product identity.

Studio also follows one interface rule:

- CLI, Studio, and agents are equal first-party clients of the same Ota engine
- Studio must never be framed as a separate truth system or as a visual layer that invents its own
  execution semantics

The product model for Studio is:

1. Inspect
2. Review
3. Apply
4. Trigger
5. Observe

## Change discipline

Future notes, sketches, and implementation ideas should be folded into this spec set rather than
creating parallel Studio planning documents elsewhere.

When Studio scope changes materially, update:

- `product-spec.md` for behavior and UX truth
- `architecture.md` for system truth
- `roadmap.md` for sequencing truth
- `cleanup-plan.md` for retirement and migration truth
- `event-schema.md` for event and operation truth
- `registry-schema.md` for persisted repo-state truth
- `http-api.md` for local integration truth
- `design-system.md` for implementation-facing visual truth
