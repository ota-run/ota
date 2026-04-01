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

# Semantic Diff and Explain

`ota diff` and `ota explain` are shipped read-only contract commands.

Use `ota diff` when you want to compare two contract states semantically.
Use `ota explain` when you want a readiness report turned into a fix plan.

## Source model

`docs/spec` is the canonical source of truth. This page is the public reference
layer derived from it. It adds examples, use cases, and operator guidance so the
page stands on its own while staying aligned with shipped behavior.

The spec keeps them separate from:

- `ota detect`, which infers
- `ota doctor`, which diagnoses
- `ota init`, which bootstraps

## Why it matters

- helps agents propose smaller, safer edits
- helps humans review contract impact without reading raw YAML diffs
- keeps remediation separate from inference
- gives CI a deterministic impact or fix-plan surface

## `ota diff`

`ota diff` compares two repo or workspace contracts as structured YAML and reports added,
missing-in-target, and changed fields in deterministic order.
The summary counts appear after the field-level sections.
Use it before writing contract changes or in CI when you need semantic impact instead of a raw YAML
diff.

### What it tells you

- which contract fields changed
- whether the change improves, degrades, or mixes readiness impact
- whether a policy section changed and where the provenance came from
- how many fields were added, removed, changed, strengthened, or weakened

### Current JSON shape

`ota diff --json` returns:

- `ok`
- `path`
- `base`
- `target`
- `summary`
- `changes`

The summary includes readiness-impact data and field counts.
The `changes` array preserves deterministic ordering and may include optional `provenance`
for policy-section changes.

### Useful cases

- review what a proposed contract change will do before writing it
- compare a branch against main in CI
- summarize the impact of a workspace bootstrap change
- check whether a contract edit is mostly operational or mostly behavioral

### Example

```bash
ota diff ./before/ota.yaml ./after/ota.yaml
ota diff --json ./before/ota.yaml ./after/ota.yaml
```

For policy-aware changes, the output can call out provenance so reviewers can see whether the
change came from the repo contract or a policy layer.

## `ota explain`

`ota explain` turns readiness findings into ordered remediation steps.

It stays read-only and deterministic.
Use it when you want a blocker list converted into a fix order that a human or agent can follow
without re-reading the raw findings.

### What it tells you

- the order in which to fix blockers
- the severity of each step
- the stable finding code for each step
- why each step exists
- the next safe action when Ota can name one
- provenance when the finding came from policy or drift context

### Current JSON shape

`ota explain --json` returns:

- `ok`
- `path`
- `summary`
- `steps`

Each step includes:

- `order`
- `code`
- `severity`
- `summary`
- `why`
- `next`
- optional `provenance`

### Useful cases

- an agent asks for the next best fix order
- a human wants one concise path from blockers to readiness
- CI wants a stable remediation summary to paste into a ticket or comment
- a policy finding needs a direct explanation instead of a generic blocker list

### Example

```bash
ota explain ./repo
ota explain --json ./repo
```

If the underlying finding came from policy or drift, the step can carry provenance so the
reason for the remediation is visible instead of implied.

## Non-goals

- auto-writing contract changes
- fuzzy natural-language repair
- hiding readiness blockers behind a generic suggestion engine
- replacing `doctor` or `detect`
