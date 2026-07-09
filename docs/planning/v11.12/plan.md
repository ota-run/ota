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

# V11.12 Plan

Status: planned follow-on product slice.

Release target:

- follow-on slice after `v11.11`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.10 plan](../v11.10/plan.md)
- [V11.11 plan](../v11.11/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.12 theme:

- typed hydration input provenance

This slice tightens dependency hydration truth where the trust boundary is shaped not only by the
verb (`restore`, `install`, `sync`) but also by the selected source or feed posture that the
hydration lane reaches at run time.

The product goal is not:

- one giant cross-ecosystem package-feed feature blob
- pretending every ecosystem has the same source-trust model
- replacing lockfiles or replay identity with source prose
- widening hydration truth blindly before a real ecosystem pressure target exists

The product goal is:

- make hydration source provenance explicit where it materially changes trust or replayability
- let Ota name the difference between a pinned dependency graph and a moving source posture
- start narrow, then widen ecosystem by ecosystem only where the trust boundary is real

## Canonical product principle

Typed dependency hydration should name both what was requested and where that request resolved.

If the hydration lane's nondeterminism lives in a feed, index, registry, mirror, or online/offline
posture, Ota should not stop at the verb alone.

The mature claim is not only:

- `dotnet restore`

It is:

- `dotnet restore`
- against this declared feed or source posture
- with this online/offline trust boundary
- under this replayability claim

## Problem statement

Ota already owns typed hydration more strongly across:

- Bundler
- Poetry
- uv
- npm / pnpm / yarn widening
- dotnet restore as a first-class lane

That is meaningful progress.

But a real trust gap remains whenever the hydration source selection itself materially changes the
execution or replay claim.

For example:

- the lockfile is unchanged
- the restore verb is unchanged
- the build still flips from green to red because the source/feed moved, slowed down, or resolved
  against a different external input posture

That is not the same class of truth as the package verb alone.

V11.12 is the slice for making that distinction first-class where it matters.

## Included capabilities

- typed hydration input provenance where source/feed posture materially changes trust
- ecosystem-specific provenance widening instead of one forced universal source model
- online/offline posture where it materially affects replay or trust
- machine-readable hydration-source identity on the canonical structured hydration surface

## Non-goals

- do not try to solve every package ecosystem in the first cut
- do not redefine replay or proof-scope as hydration features
- do not add source provenance where the ecosystem does not materially expose a different trust
  boundary yet
- do not turn repo-local shell workarounds into the canonical long-term answer

## Core product gaps

### 1. Typed hydration can still stop too early at the verb

Ota can know:

- `dotnet restore`
- `uv sync`
- `bundle install`

But that is not always enough to explain trust or replayability when the real nondeterminism lives
in the selected source posture behind that verb.

### 2. Hydration trust still over-relies on ecosystem-agnostic fields

Lockfiles and runtime identity are necessary, but sometimes not sufficient.

If source or feed selection materially changes the lane's behavior, Ota should name that as
hydration input provenance instead of letting it remain hidden ambient state.

### 3. Source posture widening still needs to stay ecosystem-honest

The mature product move is not one abstract `feed_posture` blob for every ecosystem.

The mature move is:

- define typed hydration input provenance as the product concept
- widen it ecosystem by ecosystem where real repo pressure proves the source boundary matters

## Proposed implementation order

1. define typed hydration input provenance as the canonical concept
2. choose one first real ecosystem target
3. add source/feed posture only for that target
4. pressure-test on a real repo where the source boundary actually matters
5. only then widen to the next ecosystem

## Proposed implementation slices

### 1. Canonical hydration input provenance model

Direction:

- keep the existing typed hydration source as the primary lane owner
- add a narrower provenance layer for source/feed posture where it materially changes trust
- ensure this model stays separate from replay baseline identity and proof boundary identity

### 2. First ecosystem target

The strongest first target is `.NET restore`.

Reason:

- the trust boundary around restore source posture was surfaced directly by real repo pressure
- feed identity and online/offline posture are concrete and understandable here
- this gives Ota one honest first-class source-provenance lane before broader widening

### 3. Honest source posture fields

Direction:

- prefer typed fields over opaque prose
- publish only the source posture Ota can actually stand behind, such as:
  - feed or source identity
  - selected mirror or default source posture
  - online/offline mode when it materially changes the trust claim
- keep future ecosystem shapes free to differ where the real trust boundary differs

### 4. Replay and trust alignment

Direction:

- hydration input provenance should strengthen replay honesty, not replace it
- when source posture is still live and moving, Ota should be able to say so plainly
- when the source posture is pinned tightly enough for stronger replay claims, Ota should publish
  that stronger input truth explicitly

## Acceptance bar

V11.12 is complete when:

- Ota can express typed hydration input provenance for at least one real ecosystem where the
  source/feed materially affects trust
- that provenance is machine-readable on the structured hydration surface
- a real pressure repo can move from hidden source posture to declared source posture without
  dropping back to shell glue
- the model stays ecosystem-honest instead of forcing one fake universal source taxonomy

## Pressure-test target

The first real bar should be a repo whose deterministic typed hydration lane is already useful,
but where source/feed posture still changes the trust claim in practice.

The strongest nearby target is a .NET repo with first-class `restore` pressure.
