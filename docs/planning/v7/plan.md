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

# V7 Plan

Status: active.

Source direction:

- `/Users/bobai/Desktop/ota.run/Spec/New/22-v7-spec.md`
- [Remote runner and editor surface](../../spec/remote-runner-and-editor-surface.md)
- [Hosted validation workflow](../../spec/hosted-validation-workflow.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Command reference](../../spec/command-reference.md)
- [Semantic diff and explain](../../spec/semantic-diff-and-explain.md)

V7 theme:

- platform workflow and operator experience
- editor/IDE and CI integration stability
- remote-runner visibility

## Included capabilities

- editor/IDE integration contract stabilization
- remote runner metadata standard finalization
- hosted validation workflow shape
- stronger machine-facing operational summaries
- semantic contract diff and remediation planning

## Priorities

1. Keep ota signals consumable by editors and CI at scale
2. Preserve command semantics while broadening operational integrations
3. Keep human/agent symmetry across tooling surfaces

## Execution slices

1. Editor/IDE contract stabilization

- keep contract and JSON surfaces stable for editor consumers
- avoid introducing editor-specific behavior into core commands

1. Remote runner metadata finalization

- keep integration metadata in a stable, predictable shape
- preserve repo/workspace symmetry for remote-runner consumers

1. Hosted validation workflow shape

- define non-mutating validation flow and PR-gating semantics
- keep readiness checks deterministic and machine-readable

1. Machine-facing operational summaries

- strengthen summary lines for platform teams
- keep human-readable and JSON outputs aligned

## Current progress

- V6 has already surfaced execution metadata and extension boundaries
- editor/remote-runner contract surfaces are visible in text and JSON
- hosted validation remains a distinct V7 boundary

## Success criteria

- at least one editor integration can consume the stable contract successfully
- hosted validation can gate pull requests deterministically
- remote runner integrations consume ota metadata without custom per-repo glue
- semantic diff can explain contract change impact without raw YAML parsing
