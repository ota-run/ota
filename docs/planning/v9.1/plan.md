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

# V9.1 Plan

Status: planned.

Source direction:

- [Doctor quality bar](../../spec/doctor-finding-contract.md)
- [Docs clarity](../../spec/docs-clarity-spec.md)
- [Command reference](../../spec/command-reference.md)
- [Semantic diff and explain](../../spec/semantic-diff-and-explain.md)

V9.1 theme:

- narrow the first-contact story
- make onboarding feel immediately useful
- reduce trust leaks in public docs and command output
- ship one derived agent-facing artifact from the existing contract

This slice is the public-adoption hardening pass before `v1.0.0`.

## Included capabilities

- doctor-first onboarding language in public docs
- explicit repo-boundary behavior for path inputs
- more actionable remediation text from `doctor` and `explain`
- stronger starter contract inference for common repo workflows
- `AGENTS.md` export or sync from `ota.yaml`

## Non-goals

- do not add enterprise dashboard or hosted control-plane behavior
- do not widen policy or hosted validation beyond the current core contract
- do not introduce a separate drift or onboarding command
- do not add new backend abstractions unless they directly unblock public adoption

## Priorities

1. Make the first 5 minutes obviously useful
2. Keep repo-local onboarding honest and reviewable
3. Preserve `doctor first, contract second`
4. Ship the smallest derived artifact that helps agent adoption immediately

## Execution slices

1. Doctor-first public story

- rewrite README and site-facing docs so the first recommendation is `ota doctor`
- keep `ota init` and `ota detect` positioned as reviewable follow-ups, not the first headline
- align quickstart examples with the actual trust model

1. Explicit path boundaries

- treat an explicitly supplied directory as the repo boundary by default
- avoid surprising upward discovery when the user intentionally targeted a nested repo
- keep contract resolution predictable and easy to explain

1. Exact remediation

- replace generic “install X on PATH” guidance with exact next commands where signals are strong enough
- keep doctor/explain output deterministic and short
- preserve a stable review path for weaker signals

1. Starter contract quality

- infer the most common workflow shape with higher confidence
- improve starter `setup`, `check`, `ci`, and service hints where signals are strong
- keep the starter honest about what was inferred vs declared

1. `AGENTS.md` export

- derive a repo-local `AGENTS.md` from `ota.yaml`
- keep the export deterministic and reviewable
- make the export useful without turning ota into a policy control plane

## Success criteria

- first-contact docs lead with `ota doctor`
- explicit directory inputs no longer feel like a trap
- remediation text offers clear next commands instead of generic advice
- `ota init` produces a more immediately useful starter contract
- `AGENTS.md` can be generated or synchronized from the existing contract

