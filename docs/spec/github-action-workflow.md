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

# GitHub Action Workflow

This document defines the official GitHub Actions integration surface for ota.

Use `ota-run/action@v1` when your runner is GitHub Actions and you want ota to publish a
step summary, annotations, pull-request comments, and receipt artifacts without writing your own
JSON glue.

## Purpose

The GitHub Action keeps the boundary thin and honest:

- ota still owns diagnosis and receipt truth
- GitHub Actions still owns workflow scheduling and permissions
- the action should reuse `ota annotations --format markdown` for summaries/comments,
  `ota annotations --format github` for line annotations, and upload the captured ota artifacts

It is not a second diagnosis engine.

The action repo lives at [github.com/ota-run/action](https://github.com/ota-run/action). This
page mirrors the shipped action contract so users do not need to leave the ota docs to find the
supported inputs, outputs, and default behavior.

## Use when

Use `ota-run/action@v1` when:

- a pull request needs a read-only readiness gate
- you want archived `ota receipt --json` artifacts in Actions
- you want GitHub annotations without writing `ota annotations` glue yourself
- you want a sticky pull-request comment that tracks the latest repo readiness result

Use direct `ota` commands in workflow steps when:

- the workflow must run `ota up`, `ota run`, or other mutating/setup commands
- you need full control over task orchestration inside the job

When a job needs direct `ota` commands, prefer installing ota from repo-owned
`agent.bootstrap.ota.source` truth:

```yaml
- uses: actions/checkout@v6
- uses: ota-run/setup@v1
  with:
    source: contract
```

When a workflow intentionally skips `ota-run/setup@v1` but still wants the GitHub wrapper to
honor repo-owned bootstrap truth, `ota-run/action@v1` can consume the same contract directly:

```yaml
- uses: actions/checkout@v6
- uses: ota-run/action@v1
  with:
    command: receipt
    source: contract
    contract-path: ota.yaml
```

## Quick start

```yaml
name: readiness

on:
  pull_request:
  push:

permissions:
  actions: read
  contents: read
  pull-requests: write

jobs:
  readiness:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v5
      - name: Publish ota readiness
        uses: ota-run/action@v1
        with:
          command: receipt
          archive: true
          fail-on-new-blockers: true
          github-token: ${{ github.token }}
```

## What the action does

Current `v1` behavior:

- runs `ota receipt --json --archive` or `ota doctor --json`
- installs ota by default on every run unless `install: never` is set
- supports both workflow-owned explicit install truth and contract-owned install truth through
  `source: explicit | contract`
- writes a GitHub step summary through `ota annotations --format markdown`
- emits GitHub annotations through `ota annotations --format github`
- optionally creates or updates a sticky pull-request comment
- uploads the captured JSON output and any archived receipt file as workflow artifacts

## Canonical renderer reuse

The GitHub wrapper should not maintain its own summary or annotation phrasing. Reuse the CLI
adapters directly:

- `ota annotations --mode doctor --format markdown` for repo doctor step summaries and PR comments
- `ota annotations --mode workspace-doctor --format markdown` for workspace doctor summaries and
  PR comments
- `ota annotations --mode receipt-diff --format markdown` for receipt baseline gate summaries and
  PR comments
- `ota annotations --mode doctor --format github` or
  `ota annotations --mode workspace-doctor --format github` for line annotations

That keeps local CLI output, CI summaries, and pull-request comments on one canonical rendering
path instead of re-assembling headline, blocker, provenance, and next-step text inside the
wrapper.

## Choose the command

- use `command: receipt` when you want an archive-friendly, read-only repo receipt artifact
- use `command: doctor` when you want the richer readiness verdict and primary-blocker surface

`receipt` is the better default for CI because it stays read-only, packages the current readiness
scan as a durable artifact, and keeps later automation pointed at the receipt surface.

## Install behavior

The action supports:

- `install: auto` (default) to reuse an existing ota binary in explicit mode or install the
  contract-declared source in contract mode
- `install: always` to run the official installer on every run
- `install: never` to fail closed unless ota is already available on the runner
- `source: explicit` to keep workflow-owned install truth through `ota-version`
- `source: contract` to read `agent.bootstrap.ota.source` from `ota.yaml`
- `contract-path` to point at the target `ota.yaml` when `source=contract`
- `ota-version` to pin the installed ota release explicitly in explicit mode

The action currently supports Linux, macOS, and Windows GitHub Actions runners.

When `source: contract` is used, the action and `ota-run/setup@v1` both consume the same bootstrap
mapping from `agent.bootstrap.ota.source`:

- `kind: version`
  - install through the release lane
  - shell mapping:
    `curl -fsSL https://dist.ota.run/install.sh | OTA_VERSION=<version> sh`
  - PowerShell mapping:
    `$env:OTA_VERSION='<version>'; irm https://dist.ota.run/install.ps1 | iex`
- `kind: git_rev`
  - install through the git lane
  - shell mapping:
    `curl -fsSL https://dist.ota.run/install.sh | OTA_GIT_REV=<rev> sh -s -- --from-git`
  - PowerShell mapping:
    `$env:OTA_GIT_REV='<rev>'; & ([scriptblock]::Create((irm https://dist.ota.run/install.ps1))) -FromGit`
- `kind: branch`
  - install through the same git lane
  - shell mapping:
    `curl -fsSL https://dist.ota.run/install.sh | OTA_GIT_BRANCH=<branch> sh -s -- --from-git`
  - PowerShell mapping:
    `$env:OTA_GIT_BRANCH='<branch>'; & ([scriptblock]::Create((irm https://dist.ota.run/install.ps1))) -FromGit`

## Direct command jobs

`ota-run/action@v1` is the reporting wrapper. When a later job needs direct `ota up`, `ota run`,
or `ota proof` commands, use the contract-owned installer action instead of duplicating `OTA_VERSION`,
`OTA_GIT_REV`, `OTA_GIT_BRANCH`, or `--from-git` choices in workflow YAML.

That setup mode reads `agent.bootstrap.ota.source` from the checked-out `ota.yaml` and runs the
matching official installer flow through the single public `ota-run/setup` surface.

## Managed reusable projection

When a repo wants Ota to own its CI verification lane rather than only report on it, generate a
dedicated reusable workflow and keep a small human-owned caller for scheduling and provider policy:

```bash
ota ci github render --workflow verify \
  --target-os linux \
  --output .github/workflows/ota-governance.yml
ota ci github check --workflow verify \
  --target-os linux \
  --output .github/workflows/ota-governance.yml \
  --caller .github/workflows/ci.yml
ota ci github sync --workflow verify \
  --target-os linux \
  --output .github/workflows/ota-governance.yml \
  --caller .github/workflows/ci.yml
ota ci projection --workflow verify --mode container --target-os linux --json
ota ci github render --workflow verify --mode container --target-os linux
```

The caller retains triggers, permissions, runners, secrets, concurrency, environments, and
deployment jobs. It must reference the generated workflow and its exact
`ota_projection_identity` and `ota_target_os`; it may pass `ota_runner` as provider-owned runner selection. `sync`
writes only the marked generated file, never the caller. This is one-way authority: existing
workflow files remain useful onboarding evidence, but reviewed
`ota.yaml` is the source of Ota-owned setup and verification truth.

The generated adapter verifies its identity with Ota itself, not a provider-shell primitive. The
caller keeps runner selection provider-owned, while `ota_target_os` binds the selected operating
system into the projection identity.

The generated lane runs contract validation, Doctor, safe-surface discovery, and an agent-mode dry
run. It prepares every workflow with `ota up`; when the selected run task is finite, it then runs
that exact task through `ota run --agent` before archiving workflow readiness. Service-runtime and
proof-required lanes retain one authoritative runtime execution path. A proof claim never admits an
otherwise unsafe task.
The generated action revisions are immutable so render identity covers the adapter dependencies.
A distinct runtime workflow remains a distinct projection;
Ota does not infer that one workflow's proof is evidence for another.

Use `--mode container` to materialize a separate container projection when the selected contract lane
advertises container execution. Native and container lanes intentionally have different projection
identities; a caller must select the generated lane it intends to schedule.

`ota doctor` now flags workflow-owned ota install truth when the repo already declares
`agent.bootstrap.ota.source`. If a job keeps explicit `ota-version`, raw installer commands,
branch/revision env markers, or source-install helpers in workflow YAML, treat that as
install-governance drift and move the job back onto `source: contract` unless the lane is an
intentional unreleased pressure path.

## Inputs

- `command` supports `receipt` or `doctor`; default: `receipt`
- `path` passes the repo or contract target to ota; default: `.`
- `working-directory` controls where ota is invoked; default: `.`
- `execution-mode` selects `native` or `container`; default: `native`
- `member` passes an optional monorepo member target
- `archive` adds `--archive` when `command=receipt`; default: `true`
- `annotate` emits GitHub annotations from ota findings; default: `true`
- `max-annotations` caps emitted annotations; default: `20`
- `comment-pr` creates or updates the sticky pull-request comment; default: `true`
- `comment-pr-only` limits comment behavior to pull-request events; default: `true`
- `artifact-name` sets the uploaded artifact name; default: `ota-readiness`
- `artifact-retention-days` sets optional artifact retention in days
- `fail-on-error` fails the action when ota reports a blocked outcome; default: `true`
- `fail-on-ci-drift` is an opt-in `command: doctor` gate that fails only when Ota establishes
  contract-to-CI workflow drift through `governance.merge_gate` or canonical CI drift findings;
  default: `false`
- `install` controls installation behavior with `auto`, `always`, or `never`; default: `auto`
- `source` chooses `explicit` or `contract`; default: `explicit`
- `contract-path` points at `ota.yaml` or a repo directory containing it when `source=contract`;
  default: `ota.yaml`
- `ota-version` pins installer-driven ota installation to a specific release such as `v1.4.3`
  when `source=explicit`
- `ota-bin` overrides the ota binary name or path after resolution; default: `ota`
- `output-path` chooses where the captured ota JSON payload is written; default:
  `.ota-action-output.json`
- `github-token` supplies the token used for sticky pull-request comment updates

## Outputs

- `ok` reports whether ota returned an ok result
- `status` exposes the derived action status: `ready`, `risky`, or `blocked`
- `output-path` returns the written path to the captured ota JSON output
- `archive-path` returns the archived receipt path when `receipt --archive` produced one
- `artifact-name` returns the uploaded artifact name
- `error-count` returns the error count from the ota summary
- `warn-count` returns the warning count from the ota summary
- `info-count` returns the info count from the ota summary
- `ci-drift-detected` reports whether the current Doctor output contains canonical
  contract-to-CI workflow drift
- `primary-summary` returns the primary blocker or top finding summary

## Job boundaries

When the action installs ota, it adds the install directory to `PATH` for later steps in the same
job.

That does not cross job boundaries.

If a later job needs to invoke `ota` directly, install ota again in that job or run
`ota-run/action@v1` in that job too.

## Pull-request comments

If `comment-pr: true` is set:

- the workflow should grant `actions: read` so pull-request `receipt` runs can restore the latest successful baseline artifact named by `artifact-name`
- the workflow should grant `pull-requests: write`
- the action updates one sticky comment instead of posting a new one each run
- `comment-pr-only: true` keeps comment behavior limited to pull-request events

## What it does not replace

`ota-run/action@v1` does not replace direct execution commands such as:

- `ota up`
- `ota run <task>`
- `ota workspace up`
- `ota workspace run <task>`

Use the action for GitHub-native reporting around ota. Use direct commands when the workflow
actually needs ota to prepare or execute repo work.

## Recommended split

The clean GitHub Actions shape is:

1. a read-only readiness job with `ota-run/action@v1`
2. a follow-on execution job that runs direct `ota` commands only after readiness is clear

That keeps pull-request feedback compact while preserving a clear execution boundary for setup,
task runs, and release steps.

## Repo and examples

- use [github.com/ota-run/action](https://github.com/ota-run/action) for source, release tags,
  and copyable workflow examples
- keep this page as the canonical product reference for when to use the action and what its
  contract surface is
