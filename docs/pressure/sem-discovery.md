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

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
-->

# sem Pressure Discovery

## Source

- pressure revision: `a3b08cd5f38917dbe4c12b9f724369a47a70f61c`
- hosted matrix: [run 34962979289](https://github.com/bobaikato/sem/actions/runs/34962979289)
- exercised paths: Ubuntu native and Linux/AMD64 container local-index freshness proof.

This is an Ota product-discovery record, not a public evidence-registry entry and not a claim
about sem-wide governance.

## Ota Widening Opportunities

### Aggregate execution-mode propagation

**Observed behavior:** released Ota `v1.6.27` does not apply a caller-selected execution mode to
an aggregate task's declared closure. The sem Linux container lane consequently invokes its exact
portable child tasks independently rather than `ota run verify --mode container`.

**Owner:** Core execution planner and aggregate-task mode selection. This is an unallocated,
non-blocking widening item; it must not interrupt active V12.1 Step 7 work.

**Return trigger:** when Core schedules aggregate mode propagation, rerun the sem container lane
through the aggregate workflow and require the selected closure to retain the declared container
context without caller-side decomposition.

**Not proved:** arbitrary aggregate semantics, repository-wide sem governance, or that a green
child-task composition is equivalent to an aggregate-mode execution path.

### Opaque shell-orchestration modeling

**Observed behavior:** Ota governs selection, revision context, isolated cache paths, and the
terminal result of sem's stale-index fixture, but it does not independently model each command
inside the checked-in shell carrier.

**Owner:** Core execution/evidence modeling. This is an unallocated, non-blocking widening item;
any design must preserve a repository-owned opaque-script boundary where detailed command modeling
is unavailable or inappropriate.

**Return trigger:** when Core schedules structured opaque-runner evidence or script-step modeling,
pressure the sem fixture again and distinguish independently modeled steps from retained
source-owned shell behavior.

**Not proved:** cloud-index equivalence, parser correctness, MCP-consumer behavior, package or
release behavior, or repository-wide sem governance.

## Current Partner Boundary

Neither widening item blocks the bounded sem artifact. The hosted matrix proves only the selected
native and container local-index freshness paths; the upstream review artifact must retain that
scope and must not include this internal product backlog.
