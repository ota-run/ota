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

Status: active implementation and pressure slice.

Release target:

- follow-on slice after `v11.11`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.10 plan](../v11.10/plan.md)
- [V11.11 plan](../v11.11/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.12 theme:

- typed hydration input provenance

Priority:

- this follows `v11.11`; proof-boundary truth lands first
- hydration source/feed posture is the second trust move, after Ota can already publish what a
  proof did and did not cover machine-readably

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
- keep hydration provenance owned by the structured hydration lane itself instead of drifting into
  parallel metadata
- strengthen replay and dependency-trust claims only after proof-boundary truth is already
  machine-readable and hard to over-read

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

The first honest ownership rule is:

- hydration source or feed provenance lives on the existing structured hydration source
- receipts and output surfaces report the resolved provenance of that same canonical shape
- Ota should not create a second parallel provenance surface beside the hydration-owned contract
- declared source posture and resolved source posture must stay distinct
- if runtime, host, or ambient config widens, overrides, or diverges from the declared feed
  posture, Ota should publish that divergence explicitly instead of silently rewriting contract
  truth

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
- one canonical ownership point for contract truth and one derived ownership point for output truth
- one named first machine-readable output carrier for the first honest implementation cut

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

### 4. Hydration provenance ownership is still under-specified

The key design choice is not optional:

- contract truth should live on the structured hydration source itself
- machine output should report the provenance resolved from that same source-owned truth
- Ota should not split this into one contract lane plus a separate shared provenance blob with
  independent semantics

## Proposed implementation order

1. land `v11.11` proof-boundary truth first
2. define typed hydration input provenance as the canonical concept
3. define the canonical ownership point on the structured hydration source
4. choose one first real ecosystem target
5. add source/feed posture only for that target
6. pressure-test on a real repo where the source boundary actually matters
7. only then widen to the next ecosystem

## Proposed implementation slices

### 1. Canonical hydration input provenance model

Direction:

- keep the existing typed hydration source as the primary lane owner
- add a narrower provenance layer for source/feed posture on that existing hydration-owned source
  where it materially changes trust
- ensure this model stays separate from replay baseline identity and proof boundary identity

The first honest shape should be:

- contract truth:
  - hydration source owns declared source/feed posture
- output truth:
  - `ota up --json` is the first canonical carrier and publishes the resolved provenance of that
    declared hydration source through its typed `receipt.evaluated_inputs[]` record
- no parallel repo-global provenance object for the same hydration lane

The first honest declared-versus-resolved rule is:

- declared source/feed posture remains the contract-owned truth on the structured hydration source
- resolved source/feed posture is emitted on the first output carrier as execution-time evidence
  beside the declared posture on the same selected hydration record
- if the resolved posture differs from the declared posture because host, runtime, or ambient
  config widened or overrode it, Ota should publish the mismatch explicitly instead of collapsing
  both into one field

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

The first implementation should attach those typed fields to the existing structured hydration
source for the selected ecosystem, not to a separate metadata-only output branch.

For `.NET restore`, that means:

- declared feed/source posture belongs on the structured restore source
- resolved feed/source posture is emitted on `ota up --json`
- divergence between declared and resolved source posture is first-class trust output, not hidden
  runtime trivia

### 4. Replay and trust alignment

Direction:

- hydration input provenance should strengthen replay honesty, not replace it
- proof boundary stays the first trust boundary; hydration provenance refines the input side of
  that already-bounded claim
- when source posture is still live and moving, Ota should be able to say so plainly
- when the source posture is pinned tightly enough for stronger replay claims, Ota should publish
  that stronger input truth explicitly

### 5. First carrier before broader propagation

Direction:

- pick one first machine-readable carrier before widening hydration provenance across receipts or
  replay-specific output
- the strongest first carrier is `ota up --json` because it already concentrates readiness,
  hydration, and execution-boundary truth on one operator surface
- treat receipt JSON and later replay-facing output as derived carriers from that same canonical
  hydration provenance model

## Acceptance bar

V11.12 is complete when:

- Ota can express typed hydration input provenance for at least one real ecosystem where the
  source/feed materially affects trust
- that provenance lives on the structured hydration surface as the canonical contract-owned truth
- machine-readable output reports the resolved provenance of that same structured hydration truth
- a real pressure repo can move from hidden source posture to declared source posture without
  dropping back to shell glue
- the model stays ecosystem-honest instead of forcing one fake universal source taxonomy
- the slice remains clearly secondary to `v11.11`, refining hydration trust after proof-boundary
  truth is already consumer-visible

## Pressure-test target

The first real bar should be a repo whose deterministic typed hydration lane is already useful,
but where source/feed posture still changes the trust claim in practice.

The strongest nearby target is a .NET repo with first-class `restore` pressure.
