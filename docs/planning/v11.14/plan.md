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

# V11.14: Contract-Claim Assurance

Status: planned. This follows V11.13 and does not reopen completed V11.3 runner enforcement.

## Problem

Ota can validate that a contract is structurally coherent and enforce the declared safe task
closure in `--agent` mode. Neither fact proves that the maintainer made a correct declaration.

A task can be marked safe while its body performs a risky migration, its effects can be omitted,
or its success criterion can be too weak. A wrong-but-internally-consistent contract must not read
as independently verified merely because Ota can execute it consistently.

The inverse limit matters too: Ota cannot recover an omitted assumption that leaves no observable
repo evidence. This slice must make that uncertainty explicit rather than pretending static
inspection proves intent.

## Product Boundary

V11.3 remains the runner-owned answer to:

- is this declared task and its reachable closure admitted for agent execution?

V11.14 adds the review and policy answer to:

- what observable evidence supports, contradicts, or fails to establish the declaration?

Those are separate questions. A declaration is not its corroboration, and corroboration is not a
policy decision.

## Canonical Model

For every evaluated contract claim, Ota must keep four truth classes separate:

```yaml
claim:
  subject:
    kind: task
    name: db:migrate
  family: agent_safety
  declaration:
    value: safe
  closure:
    status: safe
  assurance:
    status: supported # supported | contradicted | unknown
    coverage:
      - task_body
      - ci
      - manifests
    evidence: []
    contradictions: []
  policy:
    decision: allow # allow | deny | review
```

`subject.kind`, `subject.name`, and `family` are the canonical machine identity. An implementation
may derive a compact display or storage ID from that structured identity, but consumers must not
parse an undocumented delimiter-based string such as `task:db:migrate:agent_safety`.

- `declaration` is contract-owned maintainer intent. For agent safety, it is derived from the
  existing `safe_for_agent` and `agent.safe_tasks` surfaces; this first cut adds no duplicate
  safety authoring field.
- `closure` is runner-derived execution-graph truth. It reuses V11.3 closure resolution and must
  never be inferred from prose or copied from the declaration.
- `assurance` is Ota-derived assessment of inspected observable evidence. Its status is:
  - `supported`: the evaluator's canonical evidence requirements for this claim family are
    satisfied and no accepted contradiction exists;
  - `contradicted`: a high-confidence observed source conflicts with the claim;
  - `unknown`: the evaluator's canonical evidence requirements are absent, incomplete, weak, or
    cannot establish the claim.
- `policy.decision` is separately derived from the assurance record and the applicable policy
  pack. It is not a maintainer assertion and it does not rewrite the evidence status.

`supported` never means universally proven safe. Coverage is mandatory alongside the status so a
consumer can see which source families Ota actually inspected. An absent or weak source remains
`unknown`, never implicitly supported.

Assurance is a total, policy-independent evaluator result:

1. `contradicted` when one or more accepted deterministic contradictions exist;
2. otherwise `supported` when the claim family's canonical evidence requirements are satisfied;
3. otherwise `unknown`.

Policy may require more coverage for admission, but it must evaluate the same emitted assurance
result afterward. A stricter policy cannot turn a supported claim into `unknown` or manufacture a
contradiction; it can only return `deny` or `review` with its own policy basis.

## Claim Families

The first implementation should remain narrow:

1. `agent_safety`
   - the task is declared safe for agent execution;
   - the closure is safe under the V11.3 model;
   - observable task, adapter, and repository evidence does not contradict the declared risk
     posture.
2. `declared_effects`
   - declared writes, network, adapter state, and external state are consistent with observable
     structured command/action/adapter facts where Ota can inspect them honestly.
3. `proof_breadth`
   - V11.11 proof evidence and `not_proved[]` determine whether a claimed verification result is
     broad enough for the selected policy. Exit code zero is execution success only; this slice
     must not create a second proof taxonomy.

Do not broaden into code correctness, intent inference from opaque shell text, or a generic code
review score.

## Evidence Ownership And Confidence

Assurance evidence must be Ota-derived and carry V11.9 evidence class, source identity, and claim
family. A contract annotation must not support its own claim. Contract-owned structured execution
truth may establish closure or expose a direct contradiction, but it cannot be the sole positive
basis for `supported` assurance.

The first source families are:

- contract-owned structured task command, action, adapter, runtime, and effect declarations as
  consistency sources only;
- task closure and declared artifact lineage;
- inspected task bodies, manifests, and toolchain/source files already covered by V11.2 detection
  governance;
- CI workflow verification wiring already covered by V11.2 and V11.5;
- V11.11 proof boundaries and V11.10 replay posture where they participate in the claim.

High-confidence contradiction requires a deterministic conflict that Ota can identify precisely.
Examples include a declared safe structured action with an undeclared external-state effect, or a
required proof claim whose selected proof record carries a relevant `not_proved` boundary.

Opaque shell bodies, uninspected helpers, live external state, and missing source families do not
become contradictions by guesswork. They produce `unknown` with a precise coverage gap.

### Proof Artifact Binding

`proof_breadth` can consume a proof artifact only when Ota selects one immutable, matching record.
The cited proof basis must carry:

- immutable receipt or proof artifact identity;
- semantic contract snapshot identity matching the evaluated contract;
- selected task or workflow identity and canonical execution scope, including mode, provider,
  remote target when applicable, and lifecycle where it changes the claim;
- V11.10 freshness or replay posture.

An absent artifact, contract-snapshot mismatch, scope mismatch, stale witness, or unavailable
replay posture leaves proof assurance `unknown`. A historical green artifact must never support a
changed contract or differently scoped lane.

## Canonical Evaluator

V11.14 introduces one shared `claim_assurance` domain evaluator. It owns the canonical claim
identity, family-specific evidence requirements, assurance status, coverage, cited basis, and
contradictions. Doctor, policy evaluation, CI/merge projection, and agent admission consume this
result; none may reconstruct assurance from repository state independently.

Doctor is the first carrier and review UX, not the owner of claim-assurance semantics. This keeps
strict policy or runner admission from coupling to Doctor formatting or recomputing a second
verdict.

## First Carrier And Command Behavior

`ota doctor --json` is the first canonical carrier because day-one claim review belongs in Doctor.
It should publish a stable claim-assurance collection with:

- stable claim identity and claim family;
- declaration and closure truth;
- assurance status, coverage, cited evidence, and contradictions;
- policy decision plus basis;
- explicit evidence provenance and source identity.

Human Doctor output should state the difference plainly:

- `declared safe; assurance unknown: task body inspected, CI coverage missing`;
- `declared safe; contradicted: structured external effect is undeclared`;
- `declared safe; supported within task-body and manifest coverage`.

Existing `ota run --agent` behavior remains backward-compatible. It continues to enforce the
declared safe closure. A policy pack may opt into stricter admission, for example requiring
supported `agent_safety` with named coverage for a governed context. That policy denial or review
must occur before execution and cite the same canonical assurance record.

## Policy Model

Policy may require:

- minimum assurance status for a claim family;
- minimum named coverage for a claim family;
- deny or review when a contradiction exists;
- qualified proof breadth under V11.11 for a merge- or agent-governed lane.

Policy must not manufacture support or alter assurance status. It may only decide whether the
emitted declaration, closure, assurance posture, and coverage are sufficient for its context.

The default policy remains compatible: unsupported claims are visible in Doctor but do not silently
change ordinary human execution or existing agent-mode admission. Strict organizations can require
supported assurance explicitly.

## Structural Invariants

- a policy decision is derived from one localized assurance record, not reassembled from a second
  read of repository state;
- `supported` requires non-empty coverage, at least one runner-derived evidence basis, and at
  least one cited non-self-origin source;
- `contradicted` requires at least one cited deterministic contradiction;
- `unknown` is required when the evaluator's canonical evidence requirements are missing or
  insufficient;
- proof breadth always consumes V11.11 `proof_verdict` and `not_proved[]`; `ok: true` alone never
  supports a proof-completeness claim;
- strict policy refusal must retain the claim identity and policy basis in the execution governance
  record, using V11.9 provenance rules.

## Non-Goals

- Do not claim static analysis can prove hidden intent or absence of all risk.
- Do not replace V11.3 safe-closure enforcement.
- Do not make a raw shell heuristic a definitive contradiction.
- Do not add a duplicate `safe_for_agent` declaration surface.
- Do not turn Doctor into a generic lint or PR-review product.
- Do not imply that supported coverage makes a proof repo-global; V11.11 boundaries remain
  authoritative.

## Implementation Order

1. Define the shared `claim_assurance` domain types and policy-independent evaluator.
2. Implement `agent_safety` from existing declaration and V11.3 closure truth.
3. Add deterministic structured-effect contradiction and explicit unknown-coverage paths.
4. Add policy evaluation that consumes the canonical record without changing the default policy.
5. Bind `proof_breadth` to immutable, scope-matching V11.11/V11.10 artifacts.
6. Carry the shared result through `ota doctor --json` as the first review carrier.
7. Propagate the canonical decision to agent-mode refusal only when policy requires it.
8. Pressure-test before widening source families or adding heuristic shell interpretation.

## Acceptance Bar

V11.14 is complete when:

- Doctor distinguishes declared, closure, assurance, and policy truth without collapsing them;
- unknown assurance is visible and cannot read as verified safety;
- high-confidence observed conflicts produce cited contradictions rather than generic warnings;
- strict policy can deny or require review before agent execution from the canonical assurance
  record, while default agent-mode semantics remain compatible;
- proof-completeness policy consumes V11.11 output instead of an independent success model;
- JSON evidence identifies source, provenance, coverage, and policy basis;
- proof-backed assurance rejects absent, stale, contract-mismatched, or scope-mismatched proof
  artifacts as `unknown`;
- regression fixtures prove that a wrong-but-consistent declaration is `unknown` or
  `contradicted`, never silently `supported`;
- pressure testing proves one repo with structured task/effect evidence and one repo whose CI or
  proof boundary exposes incomplete coverage.

## First Pressure Targets

The first target should be a real service repository with:

- an agent-safe deterministic verification lane;
- a separately declared migration, publish, or external-effect lane;
- CI wiring and at least one structured manifest or runtime source;
- a truthful proof boundary that remains narrower than whole-repo completion.

The first negative-control fixture must deliberately mislabel a structured external-effect task as
safe or omit its effect declaration. Ota must emit a cited contradiction before execution. A second
fixture with an opaque helper or missing CI source must remain `unknown`, demonstrating that Ota
does not pretend incomplete inspection is proof.
