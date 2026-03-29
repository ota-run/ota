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

# JSON Output

Ota supports machine-readable JSON for core commands and workspace commands.

When to use:

- CI pipelines
- editor tooling
- agent workflows
- scripts that need stable parsing
- editor/IDE integrations that need contract, readiness, and execution metadata

JSON output is part of Ota’s integration contract for:

- CI
- editor integrations
- agent tooling

Editor and CI tooling should prefer the JSON surfaces here over parsing human-readable command
output. The stable inputs are `ok`, `errors`, `findings`, `summary`, per-repo results, `execution`,
and declared extension descriptors.

Use the smallest surface that matches the job:

- `validate` for contract gating
- `doctor` for readiness diagnosis
- `workspace explain` for ordered remediation
- `workspace tasks` for workspace inventory and task availability
- `workspace list` for lightweight workspace inventory and readiness
- `up` and `workspace up` for preparation and readiness roll-up
- `workspace run` for coordinated multi-repo execution roll-up and receipts
- `diff` and `explain` for change impact and remediation planning

Editor and IDE consumers should prefer the smallest stable fields for the job instead of parsing
human-readable text output:

- `ota validate --json` and `ota workspace validate --json` for `ok`, `summary.error_count`, `errors` or `error`, and `next`
- `ota doctor --json` and `ota workspace doctor --json` for the top-level `summary`, per-repo `findings`, and `execution`
- `ota workspace explain --json` for the top-level `summary`, per-repo `findings`, and per-repo `steps`
- `ota workspace tasks --json` for the top-level `summary`, per-repo `tasks`, and dependency order
- `ota workspace list --json` for the top-level `summary`, per-repo readiness, and contract presence
- `ota up --json` and `ota workspace up --json` for the top-level `summary`, `receipt`, and per-repo results
- `ota workspace run --json` for the top-level `summary`, `receipt`, and per-repo results
- `ota diff --json` and `ota explain --json` for the change summary and remediation steps

For v7 operator workflows, the most useful JSON surfaces are:

- `ota validate --json` and `ota workspace validate --json` for deterministic contract gating
- `ota doctor --json` for repo readiness and execution metadata
- `ota workspace doctor --json` for per-repo readiness, execution metadata, and workspace roll-up summaries
- `ota workspace explain --json` for workspace remediation steps and summary counts
- `ota workspace tasks --json` for workspace task inventory and summary counts
- `ota workspace list --json` for repo inventory, readiness, and contract presence
- `ota up --json` for preparation status and backend-driven readiness flow
- `ota workspace run --json` for coordinated multi-repo execution roll-ups and execution receipts
- `ota workspace up --json` for workspace preparation roll-up and receipt summary
- `ota diff --json` for semantic contract comparison
- `ota explain --json` for remediation plans

Design intent:

- JSON shapes are treated as stable integration surfaces.
- Human text output and JSON output are intentionally separate.
- Exit code behavior and JSON payloads should be consumed together in automation.
- validation JSON includes a compact `summary.error_count` so hosted gates can read one field
  instead of re-counting errors themselves
- workspace doctor and explain JSON include top-level summary roll-ups for repo, finding, and step
  counts so hosted consumers do not have to derive them from nested reports

Common patterns:

- success payloads include `ok: true`
- failure payloads include `ok: false` and structured error/findings context
- workspace commands include per-repo result objects when applicable
- execution metadata is descriptive and should be consumed directly rather than reconstructed from text output

## Practical integration pattern

For each command execution in automation:

1. run with `--json`
1. check process exit code first
1. parse payload fields (`ok`, `errors`, `findings`, per-repo reports)

Use-case:

- a CI job runs `ota doctor --json`, fails on errors, and posts warnings as annotations.

Canonical JSON references in repository:

- `docs/spec/json-output-reference.md`
- `docs/spec/json-schemas/`
- <https://github.com/ota-run/ota/tree/main/docs/spec/json-schemas>
