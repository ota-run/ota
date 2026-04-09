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
- [Up preview](../../spec/up-preview.md)

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
- backend-aware diagnosis for container-first repos
- a read-only policy review command surface
- a read-only `ota up --dry-run` preview tied to the real repo-preparation path
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

1. Target-aware diagnosis and policy review

- let `ota doctor` diagnose the execution target the repo actually uses, not only the host
- keep container diagnosis explicit so host readiness and container readiness do not get conflated
- add a read-only `ota policy review` lens for policy-versus-contract drift without introducing policy sync behavior
- keep policy review separate from repo readiness diagnosis so each command has one authority boundary

Proposed command contract:

- `ota doctor` remains the current readiness view, but its diagnosis should follow the repo's actual execution target when the contract supports one
- `ota doctor` should be able to report host/native readiness and container-target readiness explicitly, without conflating the two
- any explicit target override for doctor should be read-only and must not mutate the repo, policy, or container state
- `ota policy review` is a read-only policy-authority lens under `ota policy`
- `ota policy review` reports policy source, policy path, and policy-versus-contract conflicts without trying to sync them
- `ota policy review` does not replace `ota doctor`; it answers whether the repo contract is aligned with the active org policy
- `ota policy review` should tell the user whether the repo needs to change, the policy pack needs to change, or both need review
- `ota policy` itself remains the active-policy inspection command

Output contract:

- target-aware `ota doctor` keeps the same premium section structure as the current doctor output
- the primary blocker or primary finding should describe the selected execution context, not just the host
- `Execution` should continue to show preferred, supported, lifecycle, and container details in a stable order
- `ota policy review` should lead with `Policy`, then grouped conflicts, then deterministic next steps
- when the repo asks for an unapproved runtime or source, `ota policy review` should point to `ota.yaml`
- when the policy pack is missing an approved source or required rule, `ota policy review` should point to `.ota/org-policy.yaml`
- both commands should keep JSON output stable and read-only

1. Repo preparation preview

- add `ota up --dry-run` as the read-only execution-plan preview for repo readiness
- keep the preview on `up` instead of introducing a separate planning command
- require `ota.yaml` and reuse the same backend, lifecycle, target, provisioning, service, and setup resolution as real `ota up`
- show what `up` would attempt, what it would skip, and the first blocker that would stop execution
- keep the preview read-only: no provisioning, no service start, no file writes, no task execution, and no persistent backend mutation

Proposed command contract:

- `ota up --dry-run` previews the exact `up` plan for the selected repo or member
- `ota up --dry-run --json` mirrors the same execution, action, skip, and blocker state in machine-readable form
- `ota up --dry-run` respects `--mode`, `--lifecycle`, `--ephemeral`, and `--member` the same way `ota up` does
- `ota up --dry-run` does not replace `doctor` or `explain`; it answers what `up` would do right now

Output contract:

- text output keeps the premium `up` hierarchy with `Execution`, `Plan`, `Blocked by`, `Next`, and an explicit dry-run note
- JSON output includes `dry_run`, `execution`, `plan.actions`, `plan.skips`, and `blockers`
- exit code is `0` when the preview is actionable and unblocked, `1` when `up` would be blocked, and `1` on load or validation failure
- the preview must stay deterministic and reuse the real `up` planning path instead of inventing a second planner

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
- container-first repos can be diagnosed against the container target when needed
- policy drift can be reviewed without implying automatic policy synchronization
- `ota init` produces a more immediately useful starter contract
- `AGENTS.md` can be generated or synchronized from the existing contract
