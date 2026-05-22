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

---
title: "Pressure-testing Ota on Supabase: from setup prose to executable repo readiness"
description: "A technical write-up on integrating Ota into Supabase, keeping the upstream PR lean, and using a full pressure branch to expose and fix real Ota platform gaps."
author: "Bobai Kato"
date: "2026-05-22"
tags:
  - ota
  - supabase
  - developer-experience
  - ci
  - repo-readiness
  - ai-agents
---

# Pressure-testing Ota on Supabase: from setup prose to executable repo readiness

Supabase already has strong contributor documentation.

That is not faint praise. The docs are good. But good docs and executable readiness are not the same thing.

Before Ota, the `www/docs` contributor path still depended on a familiar manual loop:

- read the docs
- infer which workflow is the intended front door
- install the right runtime and package manager versions
- guess which commands are safe to run
- decide whether the repo is actually ready, or only partially set up
- if something fails on macOS, Windows, or Linux, figure out whether the problem is the repo, the machine, CI, or your shell

That is a normal setup experience in modern repos. It is also exactly where drift starts.

This post is about what changed when I added Ota to a scoped slice of the Supabase monorepo, how that improved the repo surface itself, and what the deeper pressure test exposed in Ota core.

## Why this matters

Most repos still answer “why does this not run?” badly.

The failure mode usually looks like this:

- local setup works differently from CI
- one workflow is implied, but not explicitly declared
- README setup and repo reality slowly diverge
- native and container paths are present, but not clearly separated
- readiness is inferred from “it seems to work” instead of declared checks
- contributors and agents have to reverse-engineer what the repo expects

That is painful for humans. It is worse for agents.

If an AI agent is dropped into an unfamiliar repo, the first problem is usually not the code change itself. The first problem is operational ambiguity:

- what this repo slice needs
- which workflow is the front door
- which tasks are safe
- what “ready” means
- what is actually blocking progress
- what the next safe action should be

If the repo cannot expose that clearly, the agent is forced to guess from README prose, scripts, lockfiles, CI output, and convention.

That is what Ota is designed to remove.

## What Ota added to Supabase

The upstream PR is intentionally scoped and intentionally lean:

- PR: [supabase/supabase#46269](https://github.com/supabase/supabase/pull/46269)
- Contract: [ota.yaml](https://github.com/bobaikato/supabase/blob/bobai/supabase-ota-readiness/ota.yaml)

The PR adds:

- an `ota.yaml` contract for the `www/docs` slice
- a cross-OS contract matrix
- Ota local artifact ignores
- a small optional Ota section in `CONTRIBUTING.md`

That gives Supabase a machine-readable contract for:

- the runtime and toolchain it expects
- the workflows contributors should use
- the setup tasks that make those workflows runnable
- the readiness checks that prove the slice is actually up
- the tasks and topology surfaces tooling can inspect directly

This is the shift from setup prose to executable repo readiness.

## Before and after

### Before Ota

The `www/docs` path was documented, but it was not encoded as repo truth.

That left both contributors and agents to reconstruct:

- which commands matter
- which runtime/toolchain versions are expected
- whether native and container paths are meant to be equivalent
- what should count as “ready”
- how to distinguish a local issue from a repo contract issue

### After Ota

That same slice can now answer those questions directly.

You can ask the repo:

```bash
ota validate .
ota doctor --workflow instant --native .
ota up --workflow instant --native --dry-run .
ota tasks --workflow app .
ota execution topology --json .
```

And the repo can answer from declared contract truth, not inferred setup lore.

For Supabase, that reduces four kinds of drift:

1. **README drift**  
   Setup prose can go stale. A contract is executable and testable.

2. **Local vs CI drift**  
   The same declared workflow can be pressure-tested in matrix CI.

3. **Human vs agent drift**  
   Humans and agents can consume the same operational truth surface.

4. **Platform drift**  
   Windows, macOS, and Linux differences get surfaced earlier instead of being rediscovered ad hoc.

## Why the upstream PR stayed lean

This part was deliberate.

The public PR models the `www/docs` slice. It does **not** claim full Supabase readiness.

That is not a limitation of ambition. It is a review strategy.

A smaller honest contract is easier to review, easier to approve, and easier to trust than a broad fuzzy one. So the public change stays disciplined:

- scoped slice
- clear workflow surface
- concrete CI pressure around that surface

But that did not mean I only did the minimum.

In parallel, I ran a deeper pressure test on a separate branch:

- [Supabase pressure branch](https://github.com/bobaikato/supabase/tree/bobai/supabase-ota-pressure-full)

That separation mattered:

- keep the upstream PR lean for higher approval odds
- do the heavier product hardening outside the PR
- fix real Ota gaps in Ota itself instead of hiding them in repo-local glue

That is the right way to use a serious repo as a product test surface.

## The pressure test

The first PR branch was only the start.

The full pressure cycle expanded into:

- native matrix coverage across Ubuntu, macOS, Windows PowerShell, and Windows Git Bash
- container matrix coverage across supported runner surfaces
- strict E2E parity checks
- bounded verification tasks
- a broader `studio` slice on the pressure branch
- repeated reruns to separate repo noise from real Ota defects

Representative runs:

- Lean PR branch matrix: [run #26281955993](https://github.com/bobaikato/supabase/actions/runs/26281955993)
- Full pressure branch green run: [run #26300846166](https://github.com/bobaikato/supabase/actions/runs/26300846166)

This is where the value of Ota becomes clearer.

The point was not just to prove that `ota.yaml` could validate.

The point was to prove that the readiness model, workflow targeting, and platform behavior held up under real pressure.

## What the contract actually models

At a high level, the contract models:

- Node + pnpm toolchain expectations
- native and container execution surfaces
- workflow-specific setup paths
- HTTP readiness for `www` and `docs`
- safe bounded tasks for validation and verification

That gives the repo a surface that tools can inspect directly instead of inferring from scattered files.

For example:

```bash
ota validate .
ota doctor --workflow instant --native .
ota up --workflow instant --native --dry-run .
ota tasks --workflow app .
ota execution topology --json .
```

Those commands are not wrappers around documentation. They operate from declared repo truth.

## What actually broke in Ota

Supabase exposed a real Ota bug.

The problem was selected workflow toolchain scoping.

In practice, Ota could let unrelated toolchain-owned tools leak into a selected workflow surface just because they were declared elsewhere in the contract.

That sounds small. It is not.

If a selected workflow does not require something, Ota should not silently activate or diagnose against it just because the broader repo declares it somewhere else.

If it does, you get:

- noisy dry-runs
- misleading readiness signals
- false activation pressure
- lower trust in workflow targeting

That bug was not fixed in the Supabase contract.

It was fixed in Ota core.

That distinction matters. If the platform is wrong, the correct move is to fix the platform, not teach users to work around it.

## What did not count as an Ota bug

The pressure test also surfaced failures that looked suspicious at first but were not Ota defects.

The clearest example was the Docker-backed Studio path on hosted CI.

That failure came from runner and stack-specific behavior in the Supabase Docker environment, including hosted `ulimit` constraints. That is real repo/runtime behavior, but it is not an Ota orchestration bug.

So the right response was:

- do not misclassify repo or runner behavior as an Ota platform defect
- do not hide runtime limitations just to force more green boxes
- keep the matrix useful for signal, not vanity

Trust is not built by making every box green. Trust is built by classifying failures correctly.

## What value this adds to Supabase

The value here is not “Supabase now has another YAML file.”

The value is that a real slice of the repo now has:

- an explicit operational contract
- declared workflow entry points
- repeatable readiness checks
- machine-readable task and topology surfaces
- cross-platform pressure testing around that declared truth

For Supabase maintainers, that means:

- less rediscovery of the same setup blockers
- earlier visibility into platform-specific drift
- a clearer operational model for contributors

For contributors, that means:

- less guessing
- less README archaeology
- less ambiguity around which path is actually supported

For agents, that means:

- fewer blind setup guesses
- a clearer answer to “what should I run?”
- a more reliable signal for “is this repo ready, or am I about to make changes in a broken environment?”

## Why Ota still adds value when a repo already has good docs

A fair question is: if Supabase already has solid docs, what does Ota add?

Docs and contracts do different jobs.

Docs explain.  
Contracts declare.

Docs are excellent for onboarding context, caveats, workflow explanation, and human guidance. But docs do not usually answer machine-checkable questions like:

- is this workflow ready right now?
- which workflow is the front door?
- which setup path belongs to this slice?
- which checks prove readiness?
- which tasks are safe for agents?
- how does this differ across native and container execution?

That is the gap Ota fills.

It does not replace good docs. It gives the repo an executable readiness layer that docs alone cannot provide.

## Why this matters beyond Supabase

The Supabase case study is really about a broader repo question:

**What should happen when a repo does not run?**

Most repos still answer that question badly.

They fail, and then they force the contributor to become the diagnosis layer.

Ota changes that shape.

A repo can declare:

- what it needs
- how it becomes runnable
- what ready means
- what is safe to run
- what should block progress

That turns readiness from tribal knowledge into contract truth.

Supabase is a useful example because it is large enough to expose real pressure, but still scoped enough to model honestly.

## Current state of the PR

The upstream PR is intact:

- [supabase/supabase#46269](https://github.com/supabase/supabase/pull/46269)

From our side:

- no unresolved actionable review threads
- contract and matrix changes are clean
- remaining red checks are upstream Vercel authorization and deploy noise, not Ota contract failures

So the Ota side of the work is sound.

## Try it on your repo

If your repo already has setup docs, that is a good start.

The harder question is whether the repo can expose, in a machine-readable and testable way:

- what it needs
- how it becomes runnable
- what ready means
- what is safe to run
- what should block progress

That is where Ota adds value.

If you want to pressure-test that on a real codebase, start here:

- [ota-run/ota](https://github.com/ota-run/ota)

And if you want to start from the Supabase example specifically:

- [Supabase PR](https://github.com/supabase/supabase/pull/46269)
- [Supabase `ota.yaml`](https://github.com/bobaikato/supabase/blob/bobai/supabase-ota-readiness/ota.yaml)
- [Supabase pressure branch](https://github.com/bobaikato/supabase/tree/bobai/supabase-ota-pressure-full)

> Note: at the time of writing, the upstream Supabase PR is still open. I’ll update this post with the final outcome once maintainers finish review.

## References

The concrete artifacts from this work are here:

- [Supabase upstream PR](https://github.com/supabase/supabase/pull/46269)
- [Supabase `ota.yaml`](https://github.com/bobaikato/supabase/blob/bobai/supabase-ota-readiness/ota.yaml)
- [Supabase contract matrix workflow](https://github.com/bobaikato/supabase/blob/bobai/supabase-ota-readiness/.github/workflows/test-ota-contract-matrix.yml)
- [Lean PR branch green run #26281955993](https://github.com/bobaikato/supabase/actions/runs/26281955993)
- [Full pressure branch green run #26300846166](https://github.com/bobaikato/supabase/actions/runs/26300846166)
