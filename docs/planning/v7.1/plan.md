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

# V7.1 Plan

Status: planned.

Source direction:

- [Doctor quality bar](/Users/bobai/Workspace/Ota.run/ota/docs/design/doctor-quality-bar.md)
- [Semantic diff and explain](/Users/bobai/Workspace/Ota.run/ota/docs/spec/semantic-diff-and-explain.md)
- [Hosted validation workflow](/Users/bobai/Workspace/Ota.run/ota/docs/spec/hosted-validation-workflow.md)
- [JSON output reference](/Users/bobai/Workspace/Ota.run/ota/docs/spec/json-output-reference.md)
- [Command reference](/Users/bobai/Workspace/Ota.run/ota/docs/spec/command-reference.md)

V7.1 theme:

- doctor as the fastest correct answer
- contractless diagnosis and repo/host trust
- drift awareness without a separate drift command
- CI/PR delivery of readiness findings

## Included capabilities

- contractless `ota doctor`
- primary blocker selection in `ota doctor`
- provenance on findings from repo, host, policy, and detection signals
- ownership classification for findings
- drift reporting in `ota detect --merge --dry-run` and `ota doctor`
- CI/PR annotation delivery from JSON output

## Priorities

1. Make `ota doctor` the fastest correct answer to “why is this repo not runnable?”
2. Keep `detect` focused on inference and `doctor` focused on trust and readiness
3. Keep delivery surfaces machine-readable for CI and editors

## Execution slices

1. Contractless doctor

- inspect repo and host signals even when `ota.yaml` is missing
- show the best next step instead of only telling the user to create a contract
- keep the output deterministic and honest about uncertainty

1. Primary blocker and provenance

- promote the top blocking issue first
- attach the source of each finding
- classify findings by repo contract, host machine, service, workspace acquisition, policy, or backend ownership

1. Drift awareness

- report stale or contradictory contract fields through `detect` and `doctor`
- reuse existing inference and diagnosis paths instead of adding a separate `ota drift` command too early
- keep drift detection read-only

1. CI/PR annotation delivery

- consume JSON findings as checks, annotations, or PR comments
- keep the JSON schema stable enough for hosted systems to gate on it directly
- avoid adding new mutation behavior to the core commands

## Success criteria

- `ota doctor` gives a useful answer even when no contract exists
- `ota doctor` surfaces one primary blocker and a concrete next action
- `ota detect` and `ota doctor` can report drift without blurring inference and diagnosis
- hosted CI can present findings without custom per-repo parsing
- editor and agent consumers can use the same JSON surfaces as hosted CI

