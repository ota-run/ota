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

# CI Pipeline Workflow

This document defines the provider-neutral CI path for ota.

Use this when the runner is not GitHub Actions, or when you want the generic shell shape that can
be translated into GitLab CI, Jenkins, CircleCI, Buildkite, or another pipeline system without
changing the ota contract.

## Purpose

The provider-neutral path keeps the pipeline honest:

- `ota validate --json` checks the contract
- `ota doctor --json` gives machine-readable readiness findings
- `ota annotations --format plain` turns those findings into portable log lines
- `ota receipt --json --archive` produces the durable readiness artifact

That separates:

- feedback and blocking logic
- portable log rendering
- archived receipt output

## Use when

Use this path when:

- the CI runner is GitLab CI, Jenkins, CircleCI, Buildkite, or another non-GitHub provider
- the team wants one copyable shell shape across multiple runners
- the pipeline needs durable receipt artifacts without provider-specific wrapper code

## Recommended command split

- use `ota doctor --json` as the readiness feedback surface
- use `ota annotations --mode doctor --format plain --input ...` as the portable log adapter
- use `ota receipt --json --archive` when you want the same readiness scan packaged as a durable
  artifact

Do not treat `receipt` as the annotation source. `ota annotations` currently consumes doctor-style
JSON, not receipt JSON.

## Canonical shell wrapper

The generic CI shape is:

```bash
#!/usr/bin/env bash
set -uo pipefail

mkdir -p .ota/ci

ota validate --json . | tee .ota/ci/validate.json
validate_status=${PIPESTATUS[0]}

ota doctor --json . | tee .ota/ci/doctor.json
doctor_status=${PIPESTATUS[0]}

ota annotations --mode doctor --format plain --input .ota/ci/doctor.json \
  | tee .ota/ci/annotations.log

ota receipt --json --archive . | tee .ota/ci/receipt.json
receipt_status=${PIPESTATUS[0]}

if [ "${validate_status}" -ne 0 ] || [ "${doctor_status}" -ne 0 ] || [ "${receipt_status}" -ne 0 ]; then
  exit 1
fi
```

Use this shape when the provider can still upload artifacts after the shell step. If the provider
stops the job immediately on a non-zero command and skips artifact upload, capture the statuses,
upload `.ota/ci/` and `.ota/receipts/`, and fail the job in a final step after the artifacts are
persisted.

## Baseline gate follow-up

Once a repo has one accepted archived receipt, keep later compare gates on the same receipt
surface:

```bash
ota receipt --json --archive --promote-baseline .
ota receipt --json --baseline promoted . | tee .ota/ci/receipt-diff.json
diff_status=${PIPESTATUS[0]}
```

Use `promoted` when the team wants an explicit accepted repo state owned by the repo itself. Use
`latest` when the newest archived receipt is enough for a lighter local or branch-level compare.
If the pipeline needs a PR or step-summary rendering for the compare result, reuse:

```bash
ota annotations --mode receipt-diff --format markdown --input .ota/ci/receipt-diff.json
```

## Install rule

For direct `ota` commands in CI:

- install ota in every job that executes ota directly
- ensure the install directory is on `PATH` in that same job before later steps run

The current installer defaults to `~/.local/bin` on Unix-like runners.

## Provider notes

- GitLab CI: use `artifacts:when: always` so blocked readiness jobs still keep `.ota/ci/` and
  `.ota/receipts/`
- Jenkins: archive `.ota/ci/**` and `.ota/receipts/**` in `post { always { ... } }`
- CircleCI: store `.ota/ci` and `.ota/receipts` as artifacts before the final fail step

## Relationship to the GitHub Action

If the runner is GitHub Actions and you want GitHub-native summaries, annotations, pull-request
comments, and uploaded receipt artifacts, prefer the official wrapper:

- [github-action-workflow.md](github-action-workflow.md)

The generic CI path stays the canonical cross-provider shape underneath that wrapper.
