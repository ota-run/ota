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

# V11.2 Plan

Status: planned.

Release target:

- planned slice after `v11.1`

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.1 plan](../v11.1/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Doctor finding contract](../../spec/doctor-finding-contract.md)

V11.2 theme:

- source convergence and detection governance

This slice is about how Ota should learn from existing environment, workflow, and repo-automation
tools without becoming subordinate to them.

The product goal is not "support every config file."

The product goal is:

- detect ecosystem tools as evidence
- govern precedence and conflicts explicitly
- converge stable truth into one execution contract

## Canonical product principle

Do not fight environment tools.
Detect them, learn from them, and converge them into one explicit execution contract.

That means:

- external tool files are evidence
- `ota.yaml` is the canonical execution-governance contract
- `ota detect` and `ota init` are the bridge

What this does not mean:

- blindly mirror every tool file into `ota.yaml`
- keep five equal sources of truth forever
- treat ecosystem config as more authoritative than the declared Ota contract
- turn Ota into a wrapper around every task runner, environment tool, and CI file

## Problem statement

Modern repos already carry execution truth across many files:

- environment managers
- shell task runners
- CI workflows
- dev environment definitions
- agent guidance docs

If Ota ignores them, it loses leverage and looks detached from how repos actually work.

If Ota submits to them, it becomes a passive wrapper with no canonical contract of its own.

The mature path is between those extremes:

- detect them
- classify what kind of truth they contain
- surface provenance and confidence honestly
- promote stable execution truth into `ota.yaml`
- warn when repo truth is split or conflicting

V11.2 is the slice for defining that posture formally.

## Included capabilities

- one canonical product principle for source convergence
- a governance model for external source classes, precedence, provenance, confidence, and drift
- a disciplined widening roadmap for high-value external source families
- explicit conflict handling rules for detect/init/doctor surfaces
- pressure-test criteria for deciding when a source widening is real leverage instead of noise

## Non-goals

- do not make external files co-equal canonical contracts with `ota.yaml`
- do not auto-import every observed tool file into the contract
- do not broaden detection just to increase source count
- do not hide source conflicts behind overconfident normalization
- do not widen into generic config management or secret management scope

## Governance model

### 1. Source classes

Ota should classify external inputs by the kind of truth they carry, not only by filename.

Primary source classes:

- environment/toolchain sources
- task/command sources
- runtime/service sources
- CI/verification sources
- agent-boundary sources
- workspace/bootstrap sources

Example mappings:

- `.devcontainer`
  - environment/toolchain
  - runtime/service
- `devbox.json`
  - environment/toolchain
- `mise.toml`
  - environment/toolchain
- `devenv.nix`
  - environment/toolchain
  - runtime/service
  - workspace/bootstrap
- `Taskfile.yml`
  - task/command
- `justfile`
  - task/command
- `package.json`
  - task/command
  - environment/toolchain
- GitHub Actions
  - CI/verification
  - runtime/service
- `AGENTS.md`
  - agent-boundary
- `CLAUDE.md`
  - agent-boundary

### 2. Authority and precedence

Ota should rank sources by product role, not by accidental popularity.

Recommended precedence:

1. declared Ota contract truth in `ota.yaml`
2. explicit Ota workspace truth in `ota.workspace.yaml`
3. direct ecosystem execution/config sources
4. CI workflow sources
5. human instruction sources such as `AGENTS.md` / `CLAUDE.md`
6. weak indirect heuristics

Meaning:

- declared Ota contract beats inferred ecosystem truth
- ecosystem config beats guesswork
- CI is evidence of what the repo verifies, not automatically the canonical local contract
- agent docs are guidance evidence, not execution truth by themselves

### 3. Provenance and confidence

Every promoted detection should carry:

- source file
- source class
- inferred field ownership
- confidence
- conflict notes when applicable

Confidence should be driven by source strength:

- high
  - direct structured execution/environment sources with stable semantics
- medium
  - task runner or CI sources that imply truth but may include historical or partial lanes
- low
  - prose docs or heuristic synthesis

### 4. Conflict behavior

When sources disagree, Ota should not silently pick one and bury the disagreement.

Required behavior:

- preserve canonical Ota contract truth when it already exists
- surface direct source conflicts in detect/init comparison and doctor-adjacent advisory surfaces
- prefer additive or review-required promotion over silent overwrite
- explain why one source was promoted and another was not

Conflict classes Ota should call out clearly:

- toolchain version drift
- task command drift
- verification lane drift
- service/runtime drift
- agent-boundary drift

### 5. Convergence behavior

The goal is not permanent multi-source coexistence.

The goal is convergence:

- detect useful truth from ecosystem files
- propose or promote the stable contract-owned version
- let the repo keep Ota as the explicit execution-governance contract

This means `ota detect` and `ota init` should act as convergence tools, not file mirrors.

## Detection roadmap

Widening should be grouped by leverage and governance clarity, not by random file popularity.

### 1. Environment and toolchain sources first

These are the strongest next inputs because they often contain stable version/setup truth.

Primary candidates:

- `.devcontainer`
- `devbox.json`
- `mise.toml`
- `devenv.nix`

Questions to extract:

- what toolchains are required
- what runtime image or environment is assumed
- what declared services or shells define the local execution path

### 2. Task and command sources second

These files often encode real runnable surfaces, but they need stronger governance because they can
contain convenience aliases and historical leftovers.

Primary candidates:

- `Taskfile.yml`
- `justfile`
- `package.json`

Questions to extract:

- what finite tasks are real public lanes
- which commands are setup, verify, or long-running launch
- where shell glue should be decomposed into stronger Ota task bodies

### 3. CI and verification sources third

CI is important, but it must not automatically become the repo's canonical local truth.

Primary candidates:

- GitHub Actions workflows

Questions to extract:

- what verification lanes are actually enforced
- what setup and service truth CI requires
- where CI and repo-local contract truth drift

### 4. Agent-boundary sources fourth

These files matter, but they are weaker than executable config.

Primary candidates:

- `AGENTS.md`
- `CLAUDE.md`

Questions to extract:

- what writable/protected path boundaries are declared
- what verification or stop/review guidance is explicit
- where prose instruction drifts from contract truth

## Proposed rollout order

1. Publish the canonical product principle.
2. Implement the governance model in planning and product framing.
3. Widen high-confidence environment/toolchain sources first.
4. Add stronger task-runner source integration with explicit conflict governance.
5. Widen CI and agent-doc integrations after the precedence model is already stable.

This order keeps widening disciplined:

- posture first
- governance second
- high-confidence detection third
- noisier source families later

## Pressure-test bar

V11.2 is not done when more filenames are recognized.

It is done when Ota can prove that widened source detection improves convergence without creating
split-brain truth.

Every source-family widening used for this slice should prove:

- source provenance is published clearly
- confidence is honest
- conflicting inputs do not silently overwrite contract truth
- promoted fields improve `ota detect` / `ota init` utility materially
- doctor or comparison surfaces can explain meaningful drift when sources disagree

## Acceptance bar

- Ota has one canonical public principle for integrating ecosystem tools
- source precedence is explicit and product-aligned
- conflict handling is defined before broad detection widening
- the first widened source families are chosen for leverage, not popularity
- `ota.yaml` remains the canonical execution-governance contract while Ota still learns from the
  tools repos already use
