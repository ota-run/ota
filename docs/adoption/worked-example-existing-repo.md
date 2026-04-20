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

# Worked Example: Existing Repo

This is the concrete existing-repo adoption path for a repo that already has `ota.yaml`.

Use it when you want one copyable example of the local loop, the CI artifact, and the promoted
baseline flow without piecing them together from multiple docs.

Doctor first, contract second.

## Starting point

Assume the repo already has a valid `ota.yaml` and a declared default verification task such as
`ci`.

The goal is to prove:

- the repo is understandable locally
- the repo can be prepared and verified from the contract
- CI can archive the same state as a receipt artifact
- the repo can later compare against one explicit accepted baseline

## Local loop

Start with diagnosis:

```bash
ota doctor
```

If the repo is ready enough to proceed, prepare it:

```bash
ota up
```

Run the canonical verification task:

```bash
ota run ci
```

Capture the current repo state as a read-only artifact:

```bash
ota receipt
```

If you are not sure which task to run after `ota up`, inspect the contract task surface first:

```bash
ota tasks --use
```

## What each step answers

- `ota doctor`: what is blocked, risky, or ready right now
- `ota up`: what ota needs to do to make the repo ready end to end
- `ota run ci`: how the repo’s declared verification path actually executes
- `ota receipt`: what state the repo is in after that run, recorded as a reviewable artifact

That is the core existing-repo loop.

## CI artifact

Keep CI on the same contract surface:

```bash
ota validate --json . | tee .ota/ci/validate.json
ota doctor --json . | tee .ota/ci/doctor.json
ota receipt --json --archive . | tee .ota/ci/receipt.json
```

If the pipeline needs portable log output from the doctor findings:

```bash
ota annotations --mode doctor --format plain --input .ota/ci/doctor.json \
  | tee .ota/ci/annotations.log
```

This keeps the workflow split clean:

- `validate` for contract correctness
- `doctor` for readiness feedback
- `receipt` for archived state

## Promoted baseline

Once the team accepts one archived receipt as the repo-owned baseline, promote it:

```bash
ota receipt --json --archive --promote-baseline
```

Later compare gates stay on the same surface:

```bash
ota receipt --json --baseline promoted
```

For a lighter local compare, use the newest archived receipt instead:

```bash
ota receipt --json --baseline latest
```

If the compare result needs a markdown step summary or PR note:

```bash
ota annotations --mode receipt-diff --format markdown --input .ota/ci/receipt-diff.json
```

## Optional local policy

Only add local policy after the repo loop is already trusted:

```bash
ota policy init --dry-run
ota policy init
ota policy
ota policy review
```

Use:

- `ota policy` to inspect the active policy source and loaded rules
- `ota policy review` to inspect the policy-vs-contract boundary
- `ota doctor` to inspect readiness with that policy applied

## Done signal

This repo is successfully adopted when:

- maintainers start with `ota doctor`
- `ota up` and `ota run ci` are the normal local path
- CI archives `.ota/ci/receipt.json` and `.ota/receipts/...`
- the team can promote and compare a baseline without custom glue
- the next repo can copy the same sequence without inventing a new method

## Related docs

- [One-team rollout](./one-team-rollout.md)
- [Quickstart](../quickstart.md)
- [CI Pipeline Workflow](../spec/ci-pipeline-workflow.md)
- [Execution Receipt](../spec/execution-receipt.md)
