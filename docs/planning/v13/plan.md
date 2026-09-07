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
   You may not use this file except in compliance with the License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# V13: Protected Repository Adoption

Status: planned and inactive. This plan does not authorize schema, command, provider integration,
repository enrollment, branch-protection mutation, required checks, execution, or support claims.

## Activation Gates

V13 may be activated only after:

- V12.1 is complete or formally deferred without moving unfinished secret-delivery claims here;
- no other implementation version is active;
- each V12.2 through V12.6 capability required by the selected provider profile is complete or
  formally deferred with its unsupported capability and evidence branches remaining explicit;
  unrelated demand-gated slices remain planned and inactive rather than becoming prerequisites by
  version order;
- one consenting maintainer needs an exact repository verification or protected-operation boundary
  to be mandatory rather than advisory;
- one external enforcement owner, initially a specific source-control or release provider profile,
  can prevent acceptance when the Ota gate is missing, stale, skipped, cancelled or refused;
- the approved contract and verification-obligation source, update authority, recovery authority,
  exact repository identity, governed operations and residual administrators are selected;
- the implementation reuses canonical Core admission and evidence truth rather than creating a
  provider-specific evaluator; and
- an independent review explicitly activates one bounded implementation step.

Planned follow-on: [V13.1 Governed Agent Runtime Integration](../v13.1/plan.md) remains inactive
until V13 completes or is formally deferred under V13.1's own activation gates.

## Why This Is V13

Ota can declare, validate, execute and retain bounded evidence for repository work. Those
capabilities do not make adoption mandatory. A repository-owned contract, generated workflow,
agent instruction or same-name CI check can be edited, omitted or bypassed unless an independently
controlled boundary requires the exact result.

V13 turns one voluntary integration into one independently enforced repository acceptance path. It
does not claim to govern every action taken while a human or agent authors a change.

## Product Boundary

The initial activated profile must bind one exact repository and one exact acceptance operation,
such as protected merge or release. Its protected enrollment names:

- repository and provider installation identity;
- approved contract and verification-obligation basis;
- required selected roots, targets, matrix dimensions and freshness;
- trusted workflow, executor/profile and evidence-producer identities;
- provider-owned gate configuration and observed enforcement posture;
- update, expiry, suspension, removal and recovery authority; and
- every administrator or provider route that remains capable of bypass.

Repository files may propose changes but cannot approve enrollment, weaken obligations, select a
trusted producer, mark a check required, or manufacture current enforcement. Provider configuration
may enforce acceptance but cannot redefine Ota's contract, identity, result or proof limits.

## Caller-Independent Trust Posture

Within an enrolled boundary, absence of independently verifiable authority yields the untrusted
baseline regardless of `--agent`, TTY state, process ancestry, executable name or claimed human
identity. `--agent` may remain useful for explicit intent, compatibility and UX, but it is not the
trust root. Additional scope requires exact externally supplied authority reconciled by Core.

An agent or human may still run ordinary commands in an independently controlled local or hosted
sandbox. Those commands do not thereby produce accepted Ota evidence, alter the approved obligation,
obtain a protected consequential capability or satisfy the provider-owned gate. V13 controls what
becomes a trusted repository outcome; it does not claim to intercept every local command.

## Canonical Model

The slice owns four distinct records:

1. `ProtectedRepositoryEnrollment` binds external repository identity, governed operations,
   authority source, lifecycle and required gate profile.
2. `ApprovedVerificationObligation` binds the independently accepted contract basis, selected
   semantic scope, required lanes, targets, inputs and change-control posture.
3. `VerificationSubmission` binds the candidate revision or merge group, exact captured inputs,
   command admission, executor/profile, results and retained evidence.
4. `RepositoryOutcomeGateDecision` reconciles enrollment, obligation, submission, provider gate
   observation, freshness and terminal decision without becoming execution authority.

Each record has a domain-separated semantic identity. Mutable lifecycle and observation snapshots
are separately identified. A repository name, path, remote URL, branch, check name, actor label,
signature or commit SHA alone is not repository enrollment, producer authority or gate proof.

## Required Verification Semantics

The candidate working tree and approved governance basis remain separate. Core derives the exact
candidate closure while reconciling independently approved obligations. Removal, skip, timeout,
reduced matrix, changed assertions, changed test discovery, changed lockfiles, stale generated
output and altered workflow inputs must be represented rather than collapsed into success.

A legitimate reviewed change may update the approved basis. The candidate cannot authorize its own
weakening. Structural reconciliation does not prove test adequacy or application correctness; those
remain responsibilities of the selected trusted obligations and reviewers.

## Exact Input And Execution Boundary

The activated profile specifies which revision, dirty/untracked state, submodules, LFS objects,
symlinks, generated inputs, lockfiles, toolchains, container/build artifacts, workflow revision,
effective CWD and caches are captured, protected or explicitly unproved. Retained inputs are
reobserved before use and cannot cross worktrees or repository identities.

Required verification runs through a supported restricted executor profile. The gate must not run
untrusted change code with branch-protection administration, broad publication credentials or
authority to approve its own result.

## Provider Gate Reconciliation

Generated CI is not enforcement. Core must observe or consume protected evidence that the provider
requires the exact gate for the exact governed operation. Missing, queued, skipped, cancelled,
neutral, stale, action-required, unavailable and ambiguous outcomes cannot silently satisfy it.

The decision binds the revision actually accepted. Head changes, base changes, merge queues,
rebases, force pushes and concurrent completions require fresh reconciliation. A same-name check,
fork artifact, untrusted workflow or success for another revision cannot substitute.

Provider administration remains external authority. If Ota cannot observe an administrator bypass,
ruleset exception, emergency route or configuration change, the public result states that limit.

## Failure And Recovery

Stable failure families must distinguish at least:

- unenrolled, expired, suspended or substituted repository enrollment;
- missing, stale, conflicting or self-authored approved obligations;
- incomplete or changed captured inputs;
- unsupported executor, target, workflow or evidence producer;
- missing, stale, skipped, cancelled, ambiguous or contradictory verification;
- provider gate unavailable, changed or not independently observable;
- candidate revision, merge-group or base mismatch;
- evidence publication or durability uncertainty; and
- recovery or break-glass use with bounded authority and retained audit evidence.

Failure never silently converts a protected operation into ordinary unrestricted execution.
Unenrolled repositories retain documented compatibility behavior.

## Implementation Order

1. Freeze one provider-neutral enrollment, obligation, submission and gate-decision model.
2. Reconcile existing V11.7 repository mapping, V12 admission, proof/archive and semantic snapshot
   machinery before adding new identities.
3. Implement protected source loading and lifecycle without provider mutation.
4. Implement exact candidate-input and approved-obligation reconciliation.
5. Add one provider gate observation profile without changing provider configuration.
6. Add provider-owned enrollment and gate setup only through explicit operator review.
7. Connect required verification execution, evidence production and outcome reconciliation.
8. Add update, suspension, removal, recovery and break-glass paths.
9. Propagate commands, schemas, JSON, references, Site, Learn, FAQ, Glossary, Skills/mirrors and
   Examples only for the user-visible surface actually introduced.
10. Run adversarial Core fixtures and immutable real-repository pressure, then complete independent
    closure review.

Every step requires a separate reviewed activation amendment. Later steps remain unauthorized until
the active step is reviewed and committed.

## Acceptance And Pressure Bar

- deleting or replacing `ota.yaml`, using an older binary, changing entry points or omitting an
  agent flag cannot weaken an enrolled operation;
- a caller with no independently verifiable authority receives the untrusted baseline even when it
  omits `--agent`, claims to be human or invokes a differently named binary;
- obligation removal, `true` substitution, reduced matrix, skipped discovery, lockfile change,
  stale cache and post-preview source mutation refuse or require explicit review;
- a same-name check, wrong revision, wrong base, stale success, untrusted fork artifact, missing run,
  cancelled run and provider-unavailable state cannot satisfy the gate;
- exact allowed updates, ordinary development and unenrolled compatibility continue to work;
- gate administration and evidence production are unavailable to the selected untrusted workload;
- recovery cannot widen scope or erase uncertainty;
- another verifier can reconcile the bounded decision and every omitted protected input remains
  explicit; and
- one maintainer installs, operates, updates and removes the integration without founder access,
  retains it through meaningful repository changes, and confirms that the evidence affects an
  acceptance decision.

Pressure must bind exact Core, provider profile, workflow, executor, repository and candidate
revisions. It inventories every unexercised material route and does not infer repository-wide
readiness, session containment, application correctness or universal provider support.

## Non-Goals

- governing every tool or action in an agent authoring session;
- authenticating humans or agents from CLI flags, TTY state or process names;
- replacing source-control permissions, branch protection or release controls;
- defining a general CI scheduler or universal test-adequacy evaluator;
- activating V12.3-V12.5 carriers by dependency or roadmap order;
- implementing Enterprise fleet administration, SSO, billing, retention or support; or
- treating adoption, a merged PR or a green badge as endorsement.

## Definition Of Done

V13 completes only when one exact repository acceptance path is independently enforced, adverse
substitutions and bypasses fail, retained evidence supports the bounded decision, explicit limits
remain visible, and the maintainer can operate the integration independently. It establishes
agent-agnostic acceptance governance, not whole-session agent governance.
