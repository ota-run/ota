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

# V12.2: Contract-Authored Crossing Requirements

Status: planned and inactive. This plan does not authorize schema, evaluator, CLI, receipt, or
carrier implementation.

## Activation Gates

V12.2 may be activated only after:

- V11.7 remains closed with its hardened Linux/systemd carrier and archive acceptance bar intact;
- V12 has completed typed effect identity, realization identity, and shared effect admission;
- V12.1 has completed or is formally deferred with no secret-delivery truth being silently moved
  into this slice;
- the current handoff explicitly activates V12.2 after independent plan review; and
- at least one real repository demonstrates that the shipped `unsafe_task` and
  `heavier_workflow` derivations cannot express an author-owned crossing requirement precisely.

Planned follow-on: [V12.3 Provider-Attested Authority Carrier](../v12.3/plan.md) remains inactive
until V12.2 completes or is formally deferred.

## Why This Is V12.2

V11.7 intentionally closed with two runner-derived requirement families: `unsafe_task` and
`heavier_workflow`. That is sufficient for bounded audited crossing, but it cannot express a
repository author's explicit requirement that one otherwise ordinary declared lane must cross an
authority boundary.

The missing surface is contract authoring, not another carrier. V12.2 adds monotonic requirement
truth to `ota.yaml` while preserving V11.7's authority, transaction, lease, cleanup, receipt, and
archive semantics.

## Product Boundary

The contract may require crossing. It may never waive a derived requirement, select an authority
decision, mint a grant, provide a credential, claim an actor identity, or choose a weaker carrier.

The canonical rule is:

```text
crossing required = contract-declared requirement OR runner-derived requirement
```

There is no contract form for `not_required`, `allow`, exemption, fallback, or caller override.
V12.2 is contract-only. Policy cannot create a crossing-requirement ground, select an eligible
carrier, narrow authority posture, or otherwise participate in crossing requirement or authority
eligibility decisions. Every policy-authored crossing behavior is deferred until a future plan
defines its selectors, authority source, precedence, identity, conflicts, snapshot retention, and
archive re-derivation.

## Provisional Contract Shape

```yaml
governance:
  crossing_requirements:
    tasks:
      publish:
        posture: required
    workflows:
      release:
        posture: required
```

Activation may refine spelling, but the public model must preserve these rules:

- selectors are typed task or workflow map keys, not dotted display paths or globs;
- every selector resolves exactly one declared root in the same contract snapshot;
- duplicate, unknown, ambiguous, generated, or dynamically expanded selectors refuse validation;
- a requirement binds the exact selected root, complete executable closure, normalized target,
  inputs, effects, resources, and work-unit identity derived at execution time;
- declarations are monotonic and cannot narrow the closure to hide dependencies, hooks, services,
  lifecycle operations, negative controls, or effect-bearing descendants;
- contract-local names locate truth but do not become authority evidence by themselves; and
- detection, init, candidate generation, incidents, or model output cannot add this declaration as
  an automatically applicable change.

The first shipped schema supports exact task and workflow selectors only. Tags, patterns, reusable
selector sets, organization policy references, and effect selectors remain unsupported until they
have unambiguous identity and pressure evidence.

## Canonical Requirement Model

One `CrossingRequirementDecision` records:

- contract identity and selected semantic-scope identity;
- selected root kind and exact root name;
- `required | not_required | unknown` result;
- every applicable requirement ground;
- declaration identity when contract-authored truth applies;
- every derived V11.7 family and classification reason;
- V12 effect and realization identities relevant to the selected closure;
- evaluator profile and implementation identity.

Contract-authored truth contributes the fixed family `contract_declared_boundary` and
classification `escalated`. It cannot self-assign a stronger family, actor, environment,
attestation, or issuer posture.

When multiple grounds apply, the canonical scope family uses fixed precedence:

```text
unsafe_task > heavier_workflow > contract_declared_boundary
```

The decision retains every ground even when one wins precedence. Removing or hiding a stronger
derived ground invalidates admission and archived re-verification.

## Identity And Compatibility

One domain-separated declaration identity binds schema version, selector kind, exact selector,
and `required` posture. One decision identity binds the complete ordered ground set and selected
semantic scope. Self-identity fields are excluded from their own derivation.

Compatibility requirements:

- contracts without `governance.crossing_requirements` retain existing V11.7 derivation;
- historical receipts and archives remain readable under their original schema branch;
- no archive may inject a declaration into historical evidence or reinterpret derived evidence as
  contract-authored truth;
- current archives carrying a declaration must retain the exact contract snapshot and re-derive
  declaration, derived grounds, precedence, and selected scope;
- contract changes stale any prior grant or work-unit admission whose requirement decision or
  semantic scope changes; and
- JSON Schema enforces local shape while Core re-derives cross-record and contract relationships.

## Shared Enforcement

The same evaluator must drive:

- `ota run` and `ota up` preview and real admission;
- workflow and CI projection;
- task/workflow discovery output where crossing posture is shown;
- authority requests, grants, leases, and recovery;
- receipts, protected history, archive verification, and assurance; and
- human, plain, and JSON explanations.

No command, provider adapter, launcher, or caller may independently decide whether the selected
lane requires crossing.

## Failure Semantics

Stable refusal families must distinguish:

- invalid or unresolved declaration;
- crossing required but no carrier configured;
- declared/derived decision disagreement;
- stale contract or selected scope;
- authority decision, grant, lease, or attestation mismatch; and
- archive declaration or requirement-decision substitution.

All requirement failures occur before authority contact when detectable locally. Once authority
contact begins, V11.7 transaction, cancellation, ambiguity, recovery, and cleanup semantics remain
authoritative.

## Implementation Order

1. Finalize schema and semantic identities without wiring execution.
2. Add parser and validator support with compatibility fixtures.
3. Extend the single crossing-requirement evaluator and decision evidence.
4. Wire preview, `run`, `up`, workflows, and CI through that evaluator.
5. Bind requirement decisions into authority protocol messages and transaction identities without
   changing carrier trust claims.
6. Extend receipts, protected history, schemas, and archive re-derivation.
7. Add migration, downgrade, stale-scope, and adversarial substitution tests.
8. Propagate Core reference, JSON reference, changelog, Examples, canonical Skill, installed
   mirrors, Site reference, and Learn curriculum.
9. Run independent real-repository pressure before closure.

## Acceptance And Pressure Bar

- a declared ordinary task requires crossing while an undeclared equivalent ordinary task does
  not, unless another derived ground applies;
- declarations can add but never remove `unsafe_task` or `heavier_workflow` requirements;
- policy input cannot add a requirement ground, select a carrier, narrow authority posture, or
  enter `CrossingRequirementDecision` evidence under the V12.2 schema;
- unknown selectors, task/workflow kind substitution, dotted-name splitting, aliases, globs, and
  duplicate normalized selectors refuse;
- dependency, hook, service, effect, target, input, and workflow-instance changes alter the selected
  semantic scope and stale prior authority;
- contract, CLI, provider, and archive attempts to downgrade a required decision refuse;
- one authority decision cannot satisfy another declaration, lane, scope, work unit, contract, or
  evaluator profile;
- cancellation, replay, expiry, revocation, ambiguous consumption, and recovery retain V11.7's
  single-use guarantees;
- receipts and archives re-derive the declaration and every derived ground from the protected
  contract snapshot;
- historical declaration-free evidence remains valid only in its original schema branch; and
- docs state that a declaration requires authority but does not prove approval, correctness,
  effect success, provider attestation, or control of raw shell outside Ota.

Pressure uses at least one real repository with an otherwise ordinary release/deployment lane and
one adversarial Core fixture. The matrix covers task and workflow declarations, stronger derived
grounds, scope mutation, stale grants, missing carrier, wrong carrier, replay, cancellation,
recovery, archive stripping, declaration downgrade, and attempted policy-ground or
policy-eligibility injection. It must prove zero selected work and zero provider/repository mutation
on refusal.

## Non-Goals

- provider-attested carrier implementation;
- macOS or Windows protected carriers;
- policy-authored crossing requirements, carrier eligibility, or authority-posture narrowing;
- approval routing, waivers, dashboards, or fleet management;
- inferred declarations or automatic contract writes;
- effect-policy authoring already owned by V12;
- secret-delivery authority already owned by V12.1; or
- controlling execution outside Ota.

## Definition Of Done

V12.2 completes only when schema, validation, one shared evaluator, authority binding, receipts,
archive re-derivation, compatibility, propagation, and immutable pressure all pass independent
review. A contract example or local evaluator test alone is not completion evidence.
