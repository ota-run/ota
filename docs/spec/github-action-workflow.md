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
- the action only translates ota JSON into step summaries, annotations, pull-request comments,
  and uploaded artifacts

It is not a second diagnosis engine.

## Use when

Use `ota-run/action@v1` when:

- a pull request needs a read-only readiness gate
- you want archived `ota receipt --json` artifacts in Actions
- you want GitHub annotations without writing `ota annotations` glue yourself
- you want a sticky pull-request comment that tracks the latest repo readiness result

Use direct `ota` commands in workflow steps when:

- the workflow must run `ota up`, `ota run`, or other mutating/setup commands
- you need full control over task orchestration inside the job

## Quick start

```yaml
name: readiness

on:
  pull_request:
  push:

permissions:
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
          annotate: true
          comment-pr: true
          github-token: ${{ github.token }}
```

## What the action does

Current `v1` behavior:

- runs `ota receipt --json --archive` or `ota doctor --json`
- auto-installs ota by default when the runner does not already have it
- writes a GitHub step summary
- emits GitHub annotations from findings
- optionally creates or updates a sticky pull-request comment
- uploads the captured JSON output and any archived receipt file as workflow artifacts

## Choose the command

- use `command: receipt` when you want an archive-friendly, read-only repo receipt artifact
- use `command: doctor` when you want the richer readiness verdict and primary-blocker surface

`receipt` is the better default for CI because it stays read-only, packages the current readiness
scan as a durable artifact, and keeps later automation pointed at the receipt surface.

## Install behavior

The action supports:

- `install: auto` (default) to reuse an existing ota binary or install ota if it is missing
- `install: always` to force installer use for the requested version
- `install: never` to fail closed unless ota is already available on the runner
- `ota-version` to pin the installed ota release explicitly

The action currently supports Linux, macOS, and Windows GitHub Actions runners.

## Job boundaries

When the action installs ota, it adds the install directory to `PATH` for later steps in the same
job.

That does not cross job boundaries.

If a later job needs to invoke `ota` directly, install ota again in that job or run
`ota-run/action@v1` in that job too.

## Pull-request comments

If `comment-pr: true` is set:

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
