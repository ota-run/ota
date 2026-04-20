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

# One Team Rollout

This guide is the practical rollout story for one team adopting ota across real repos.

Use it when the goal is not just to validate one contract once, but to make ota the default
readiness path for one repo, one CI job, and then one more repo.

Doctor first, contract second.

## Outcome

At the end of this rollout:

- one repo has a trusted local loop
- CI archives a read-only receipt for that repo
- the repo has an explicit promoted baseline for later compare gates
- the team can explain when to use `ota doctor`, `ota up`, `ota run`, `ota receipt`, `ota policy`,
  and `ota policy review`
- a second repo can adopt the same path without inventing a new workflow

## Phase 1: First Repo

Start with the repo that is easiest for the team to verify locally.

If the repo already has `ota.yaml`:

```bash
ota doctor
ota explain
ota up
ota run ci
ota receipt
```

If the repo does not have `ota.yaml` yet:

```bash
ota doctor
ota detect --dry-run .
ota init --dry-run
```

Then choose one explicit write path:

```bash
ota init
# or:
ota detect --write .
```

Then return to the same core loop:

```bash
ota doctor
ota up
ota run ci
ota receipt
```

Use `ota tasks --use` if the next runnable task is unclear after `ota up`.

## Phase 2: CI Artifact

Once the local loop is trustworthy, keep CI on the same contract boundary.

Use this provider-neutral shape:

```bash
ota validate --json . | tee .ota/ci/validate.json
ota doctor --json . | tee .ota/ci/doctor.json
ota receipt --json --archive . | tee .ota/ci/receipt.json
```

If the pipeline needs portable readiness logs:

```bash
ota annotations --mode doctor --format plain --input .ota/ci/doctor.json \
  | tee .ota/ci/annotations.log
```

This keeps:

- validation on `validate`
- diagnosis on `doctor`
- archived evidence on `receipt`

Do not invent a second readiness workflow in CI.

## Phase 3: Promoted Baseline

Once the repo has one accepted archived receipt, promote it:

```bash
ota receipt --json --archive --promote-baseline
```

Later compare gates can stay on the same receipt surface:

```bash
ota receipt --json --baseline promoted
```

Use `promoted` when the team wants an explicit accepted repo state.

Use `latest` when the newest archived receipt is enough for a lighter local or branch-level check:

```bash
ota receipt --json --baseline latest
```

If the compare result needs a PR or step-summary rendering:

```bash
ota annotations --mode receipt-diff --format markdown --input .ota/ci/receipt-diff.json
```

## Phase 4: Local Policy Pack

Only add local policy once the first repo loop is already boring and trusted.

Preview the starter pack first:

```bash
ota policy init --dry-run
```

Then write it explicitly:

```bash
ota policy init
```

Use:

- `ota policy` when you need to inspect the active policy source and loaded rules
- `ota policy review` when you need to inspect the policy-vs-contract boundary
- `ota doctor` when you need the full readiness view with that policy applied

Do not start with policy if the team still does not trust the basic repo loop.

## Phase 5: Second Repo

Only after the first repo has:

- a trusted local loop
- CI receipt archive
- one promoted baseline when needed

move to a second repo and repeat the same sequence:

```bash
ota doctor
ota up
ota run ci
ota receipt
```

If the second repo needs a different execution mode, keep the same commands and make the mode
explicit:

```bash
ota doctor --mode container
ota up --mode container
ota run ci --mode container
ota receipt --mode container
```

The rollout should still feel like one product path, not a new method per repo.

## What Done Looks Like

The rollout is working when:

- repo owners start with `ota doctor` instead of README guesswork
- `ota up` and `ota run ci` are the normal local verification path
- CI archives receipt artifacts without custom glue logic
- the team can explain the difference between `doctor`, `receipt`, and `policy review`
- policy packs, if used, are reviewed locally instead of treated as hidden governance
- adding the next repo feels repetitive, not exploratory

## Related Docs

- [Worked example: existing repo](./worked-example-existing-repo.md)
- [Quickstart](../quickstart.md)
- [CI Pipeline Workflow](../spec/ci-pipeline-workflow.md)
- [Execution Receipt](../spec/execution-receipt.md)
- [Policy Packs](../spec/policy-packs.md)
- [Command Reference](../spec/command-reference.md)
