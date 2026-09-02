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

# Cross-Cutting Plan: Execution Contract Follow-Ons

Status: planned and inactive. This register does not activate an implementation slice, change
released behavior, delay the Eris adoption PR, or interrupt active V12.1 secret-delivery work.

## Purpose

Bounded Eris adoption pressure proved that released Ota v1.6.27 can govern one native agent lane
and a smaller digest-pinned container lane without claiming that the model, Qdrant, external
integrations, or the complete repository ran. The same pressure exposed six product opportunities:

1. lock enforcement for typed Cargo hydration;
2. deterministic mode eligibility for aggregate tasks;
3. clearer default and explicit selection for mixed-mode previews;
4. root-cause reconciliation for repeated Doctor advisories;
5. stronger declaration and evidence boundaries for source-owned shell orchestration; and
6. identity-bound terminal cleanup evidence for container-created resources.

These are not one feature. This document records their separate ownership and activation bars so
that pressure findings do not become either forgotten backlog or an unjustified expansion of the
active version.

## Activation And Sequencing

Each sub-slice is independently activatable and closable. An active version plan must name the
sub-slice before implementation begins. Activating one does not activate the others.

V12.1 remains the only active product version. None of these follow-ons may enter V12.1 merely
because Eris exposed them. A follow-on may be selected only at a between-batch boundary or by a
later version after the current V12.1 step is committed, reconciled, and pressure-ready.

Activation requires:

- one named implementation owner and exact affected product surface;
- regression fixtures that reproduce the pressure signal without depending on the Eris fork;
- an explicit compatibility posture for existing contracts and JSON consumers;
- a bounded pressure target able to exercise the new behavior;
- connected Core, Site, Skill, Example, Learn, FAQ, and Glossary assessment; and
- independent review before the implementation is called complete.

Eris alone is sufficient discovery evidence for this register. It is not sufficient support,
conformance, or release evidence for any activated implementation.

## A. Lock-Strict Cargo Hydration

### Problem

The typed Cargo dependency-hydration source currently executes `cargo fetch`. A repository that
requires lock-enforced resolution must fall back to a structured command such as
`cargo fetch --locked`. That preserves correctness, but it prevents the typed hydration source
from owning the lock policy it is meant to describe.

### Canonical owner

The existing `prepare.kind: dependency_hydration` / `prepare.source.kind: cargo` source owns this
truth. Do not add a task-global lock flag or infer strictness from the presence of `Cargo.lock`.

The first compatible extension should be an optional Cargo-owned `locked: true` posture. Omission
retains current `cargo fetch` behavior. When true, Core must:

- require a canonical repository-relative `Cargo.lock` at the selected Cargo source root;
- bind the authored lock posture and observed lockfile identity into hydration planning and
  evidence;
- execute the semantic equivalent of `cargo fetch --locked` through the existing typed source;
- refuse missing, aliased, escaping, changed, or concurrently replaced lockfiles;
- verify the retained lockfile bytes again before execution and record post-run drift; and
- never relabel registry reachability or downloaded package integrity as hermetic resolution.

### Acceptance bar

- absent, false, and true postures have explicit compatibility tests;
- native and container execution use the same semantic lock policy;
- dry-run, Doctor, execution, receipt, and replay surfaces agree on the selected posture;
- unlocked mutation succeeds only under the explicitly unlocked compatibility branch;
- lockfile substitution and update attempts refuse or produce bounded drift evidence; and
- at least one real Rust repository replaces a command fallback with the typed source and passes
  immutable native and container pressure.

## B. Aggregate Mode Eligibility

### Problem

An aggregate has no executable body of its own. Its usable modes depend on every selected member,
dependency, hook, and context in the exact closure. Treating the aggregate as universally portable
or selecting different modes for hidden members would make the invocation identity misleading.

### Canonical owner

Execution planning owns aggregate mode eligibility. It is derived from the exact selected closure,
not declared as a second list on the aggregate and not inferred from unrelated repository contexts.

Core should publish, for each requested mode:

- the exact closure members considered;
- each member's supported or unavailable posture;
- the intersection that makes the complete aggregate callable; and
- a stable blocker identifying every member that prevents the requested mode.

One aggregate invocation uses one mode. Per-member mixed execution remains unsupported unless a
future workflow model explicitly binds every transition, runtime, artifact handoff, and cleanup
obligation. Ota must not silently split one aggregate across host and container execution.

### Acceptance bar

- an all-native, all-container, and dual-mode aggregate resolve deterministically;
- one unavailable member blocks the complete aggregate before setup or child execution;
- dependencies, hooks, repeated roles, variants, overlays, and generated instances participate;
- reordering or omitting a member changes or invalidates the execution graph identity;
- task, workflow, dry-run, proof, CI projection, and agent-safe listings agree; and
- pressure includes one repository with a portable subset and a larger native-only aggregate.

## C. Mixed-Mode Preview Selection

### Problem

A repository may declare both native and container contexts while one selected workflow is
intended to run natively. A preview that inspects an unselected container or silently chooses a
mode based on host availability can report the wrong blocker.

### Canonical owner

CLI selection and execution planning jointly own one versioned `mode_selection` posture. Selection
must be derived before runtime/tool admission and must bind:

- the selected task or workflow invocation;
- the authored default, explicit CLI override, or absence of either;
- the exact mode selected;
- available and unavailable modes for the selected closure only; and
- a stable reason when explicit selection is required.

An explicit `--native` must not require Docker for an unselected container context. An explicit
`--mode container` must not fall back to native. If the selected closure has multiple viable modes
and no canonical default, preview must refuse with a stable `mode_selection_required` posture
rather than choosing from ambient host state.

### Acceptance bar

- mixed-mode contracts behave identically across run, up, dry-run, proof, CI, and agent output;
- missing Docker cannot block an explicitly native selected closure;
- absent native tooling cannot block an explicitly container-selected closure;
- default, explicit, unavailable, and ambiguous mode states are schema-constrained;
- unselected contexts never enter readiness or execution identities; and
- Linux, macOS, and container pressure retain the exact selection source and refusal reason.

## D. Doctor Root-Cause Reconciliation

### Problem

One transitive hydration cause can produce the same advisory for several parent tasks. The warning
is truthful, but repeated prose makes the repository look noisier and can hide the one remediation
that resolves every affected path.

### Canonical owner

Doctor owns diagnosis. Human rendering may group findings, but Core must not deduplicate distinct
semantic paths into one weaker fact.

One domain-separated finding-group identity should bind:

- the exact underlying cause identity and code;
- every affected task, workflow, role, mode, platform, and context path;
- the strongest severity and agent posture;
- shared and path-specific remediation; and
- the original finding identities retained for machine reconciliation.

JSON keeps the complete findings and adds grouping rather than removing evidence. Plain and rich
output may render one cause with an affected-path count and expandable path details.

### Acceptance bar

- genuinely identical causes group deterministically regardless of map iteration order;
- different sources, modes, authorities, severities, or remediation never collapse;
- primary blocker selection remains stable;
- machine consumers can recover every original finding and affected path;
- agent admission consumes the original semantics, not presentation grouping; and
- a large aggregate pressure case demonstrates materially lower noise without information loss.

## E. Opaque Shell Execution Boundaries

### Problem

Eris's 47-batch runner is repository-owned shell orchestration. Ota can select it, bind its script
and invocation truth where declared, retain its output, and observe terminal success. It cannot
infer that every internal command ran correctly merely because the wrapper exited zero.

### Canonical owner

The contract remains authoritative. Detector, Doctor, and evidence consumers must label opaque
internals as opaque. Ota must not parse shell text into trusted substeps or treat output markers as
independent execution authority.

A future structured script-inventory profile may activate only when a repository authors and
pressure-tests it. Such a profile must bind the script bytes, interpreter, working directory,
ordered declared substeps, expected terminal markers, and drift posture. It remains observation of
the declared wrapper unless a registered execution adapter independently launches and witnesses
each substep.

### Acceptance bar

- opaque wrappers remain callable without gaining internal assurance;
- output and receipt schemas distinguish wrapper completion from independently witnessed steps;
- missing, reordered, duplicated, or forged markers cannot produce positive substep assurance;
- script-byte or interpreter drift invalidates the authored inventory;
- no generic shell parser becomes a trust source; and
- activation requires at least two materially different source-owned orchestration scripts.

## F. Identity-Bound Container Cleanup Evidence

### Problem

An ephemeral container log can name the selected image and container identity without proving that
the resource reached a terminal removed state after command completion.

### Canonical owner

The inactive [OSS Adapter and Profile Conformance](../adapter-profile-conformance/plan.md) plan
already owns `OwnedResourceCreationIntent`, `OwnedResourceHandle`, lifecycle observations, and
terminal cleanup evidence. This follow-on does not define another receipt or lifecycle taxonomy.

When a future version activates registered container execution under that conformance plan, the
container adapter must retain the stable handle through cleanup and emit exact terminal evidence:

- `cleaned`, with manager-observed absence for the same handle;
- `retained_by_contract`, with the explicit owner and cleanup obligation;
- `transferred`, with recipient acceptance and successor obligation; or
- `cleanup_uncertain`, which cannot support positive lifecycle assurance.

Process exit, an `ephemeral` label, client-side deletion intent, or absence from a later unrelated
listing is insufficient.

### Acceptance bar

- success, task failure, cancellation, timeout, runner crash, lost acknowledgement, and duplicate
  cleanup requests reconcile to the exact handle;
- no handle can disappear from the terminal inventory;
- protected provider/container locators remain protected evidence;
- public projections disclose only profile-approved fields and bounded verification loss;
- Linux evidence cannot establish macOS or Windows cleanup support; and
- immutable pressure includes leaked-resource negative controls and recovery after uncertainty.

## Implementation Order

This register does not impose one six-item release train. When future demand selects work, prefer:

1. lock-strict Cargo hydration as the smallest contract-truth improvement;
2. aggregate mode eligibility and mixed-mode preview selection as one reviewed execution-selection
   batch;
3. Doctor root-cause reconciliation as a presentation-plus-machine-grouping batch;
4. opaque shell inventory only after a second materially different pressure case; and
5. cleanup evidence only through an activated registered container adapter under conformance.

Do not start item 2 merely because item 1 completes. Each item needs its own activation record,
tests, propagation assessment, pressure evidence, and closure review.

## Current Product Boundary

Until a sub-slice activates:

- Cargo lock enforcement remains command-authored where required;
- aggregates are callable only in modes supported by their complete selected closure;
- mixed-mode callers should select the intended mode explicitly;
- repeated Doctor advisories remain truthful even when verbose;
- source-owned shell internals remain opaque; and
- an ephemeral container result does not establish terminal cleanup evidence.

These boundaries are acceptable for the bounded Eris draft because its contract and evidence state
them explicitly. They must not be omitted from future pressure summaries or converted into support
claims by the Site, Skills, Examples, Enterprise, or an agent.

## Propagation Posture

This planning-only register introduces no shipped command, schema, vocabulary, or operator flow.
Core implementation docs, Site, Skills, Examples, Learn, FAQ, and Glossary therefore remain
unchanged until one sub-slice activates. Every activation must reassess those surfaces rather than
using this inactive plan as public product documentation.
