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

# V13.1: Governed Agent Runtime Integration

Status: planned and inactive. This plan does not select an agent vendor, runtime, operating system,
adapter, capability policy, credential, command, schema, enforcement claim or support posture.

## Activation Gates

V13.1 may be activated only after:

- V13 is complete or formally deferred with the exact protected enrollment, approved-obligation
  and accepted-evidence foundations required by this integration retained elsewhere;
- one consenting maintainer has a recurring need to constrain actions during an agent session, not
  merely to verify the resulting change before acceptance;
- V13 acceptance governance plus the selected agent provider's native sandbox and permission model
  are shown insufficient for that exact in-session risk;
- one exact agent, runtime, operating system, architecture and execution mode are selected from
  demonstrated demand and technically enforceable hooks;
- an external runtime, credential, provider or protected-carrier boundary can deny required
  capabilities outside admitted Ota execution;
- terminal, filesystem, network, process, credential, connector, browser/download, host-socket,
  remote-tool and delegated-agent routes have an initial bypass inventory;
- the selected adapter can meet [adapter/profile conformance](../adapter-profile-conformance/plan.md)
  and [authority distribution](../authority-distribution-lifecycle/plan.md) requirements; and
- an independent review explicitly activates one bounded implementation step.

No vendor relationship, available plugin, agent instruction or local wrapper is sufficient demand
or enforcement evidence. If the provider exposes no caller-independent mediation or containment
hook, V13.1 remains inactive for that provider rather than claiming control from repository files.

## Why This Is V13.1

V13 governs whether a change can be accepted. It does not prevent an agent from reading an
unintended file, contacting a network endpoint, using an ambient credential or starting a process
during authorship. V13.1 adds one exact in-session enforcement integration where that additional
control is demanded and technically possible.

The first integration is intentionally singular. Universal agent compatibility is not a valid
starting claim. V13.1 is optional rather than the automatic next adoption stage: V13 may fully solve
the maintainer's need by governing trusted outcomes while the agent provider governs the terminal.

## Product Boundary

Activation selects one of two profiles:

- `declared_task_only`: the agent may execute only admitted Ota task/workflow closures; or
- `bounded_exploration`: exploratory commands are allowed only inside an independently enforced
  capability profile and cannot obtain protected consequential authority without a separate typed
  admission.

The first release need not support both. Instructions, Skills, environment markers, parent process
names, TTY state and an agent's self-identification may provide attribution or UX context but cannot
grant authority or establish enforcement.

An unsupported agent/runtime may still use Ota discovery and V13 verification. It cannot claim
whole-session governance.

## Canonical Model

The integration reuses the adapter/profile conformance identity model rather than defining a
parallel runtime registry:

1. `GovernedAgentIntegrationProfile` is a specialization whose stable behavior derives the existing
   `profile_semantic_identity`; it binds mediation points, permitted targets, mandatory hooks and
   declared verification loss.
2. Existing `implementation_subject_identity`, `implementation_registration_identity` and lifecycle
   snapshot identities bind exact owner, source, build, artifact, compatibility, OS, architecture,
   execution mode, evidence-backed registration and current availability.
3. `GovernedAgentSession` binds repository enrollment, approved contract basis, selected registered
   implementation, initial capability policy, delegated identities, freshness and lifecycle.
4. `EffectiveAgentRuntimeObservation` records only the dimensions independently observed as
   enforced and leaves unavailable dimensions `unknown`.

Profile, subject, registration, lifecycle, session, capability decision and observation identities
remain domain-separated. A successful command, adapter acknowledgement, signed agent message or
configured policy is not effective enforcement evidence.

## Capability And Admission Boundary

The activated profile defines exact posture for:

- repository and external filesystem reads/writes;
- network, DNS and downloads;
- child processes, daemonization and delegated agents;
- environment, credentials and provider metadata;
- IPC, mounts, device access, host control sockets and privilege escalation;
- MCP servers, connectors, browser tools and remote execution; and
- CPU, memory, process, storage and time limits.

Every route is mediated, disabled, independently bounded or explicitly outside the claim. Ota Core
derives one retained admission per command invocation. Admission does not open a provider
transaction. Consequential capabilities remain unavailable until their separately typed,
independently authorized transaction is admitted.

Policy may narrow valid contract and protected enrollment truth. It cannot invent tasks, effects,
credentials, provider bindings, implementation registration or effective observations.

## Session And Delegation Semantics

Session continuity revalidates on resume, reconnect, checkout change, contract or policy update,
runtime replacement and delegation. A child, helper, nested shell, nested Ota invocation or
delegated agent cannot exceed the parent session's effective capabilities.

Delegation records exact parent session, recipient, allowed capability subset, repository and
semantic scope, freshness and terminal posture. Missing or ambiguous lineage refuses required
operations. A model conversation or agent-generated token is not delegation authority.

## Interruption, Cleanup And Recovery

The selected runtime owns explicit interruption, timeout, orphan detection, resource limits and
cleanup for the resources it creates. Terminal evidence distinguishes completed cleanup,
cleanup-in-progress, cleanup-failed and cleanup-unknown. Controller loss cannot silently relabel a
session as terminated.

Retries after ambiguous consequential operations require provider/effect reconciliation. Recovery
and break-glass authority is separately sourced, narrower than ordinary administration and retained
in evidence. Externally owned processes or resources cannot be relabeled as Ota-owned.

## Evidence And Disclosure

Protected evidence binds profile, subject, registration/lifecycle snapshot, repository enrollment,
approved contract basis, session, admissions, capability decisions, effective observations,
delegations, effects, interruption and cleanup. Public output uses separately derived projections
and cannot expose private correlation, credential or provider-reference material.

Evidence states which dimensions were enforced, merely configured, unobserved or bypassable. A
required V13 outcome gate may consume the bounded result, but cannot upgrade missing runtime
observation into whole-session proof.

## Implementation Order

1. Select one demanded agent/runtime/target and freeze its bypass inventory and threat model.
2. Define the profile specialization, session, delegation and effective-observation identities
   while reusing conformance-owned subject, registration and lifecycle identities, without a
   runtime consumer.
3. Implement capability-policy derivation through canonical Core evaluators.
4. Implement and register one protected adapter with installation and lifecycle controls.
5. Mediate one useful allowed workflow and independently test prohibited filesystem, network,
   process and credential routes.
6. Add consequential-operation admission only by reusing an already activated typed effect path.
7. Add resume, delegation, interruption, cleanup and recovery.
8. Connect protected evidence, bounded public projection and optional V13 outcome-gate consumption.
9. Propagate commands, schemas, JSON, references, Site, Learn, FAQ, Glossary, Skills/mirrors and
   Examples only for the supported integration actually shipped.
10. Run adversarial Core fixtures and immutable native pressure with a real maintainer workflow,
    then complete independent closure review.

Every step requires a separate reviewed activation amendment. Later steps remain unauthorized until
the active step is reviewed and committed.

## Acceptance And Pressure Bar

- useful coding and verification work completes through the supported profile;
- alternate shells, interpreters, nested Ota, direct tools, helpers, daemonization, inherited
  handles and delegated agents cannot widen effective capabilities;
- prohibited filesystem, network, credential, provider-metadata, IPC and host-control access is
  denied where claimed, while allowed controls succeed;
- removing agent flags, changing TTY state, renaming processes or changing vendor markers cannot
  weaken the protected session;
- failure or absence of a mandatory adapter refuses governed operation without falling back to an
  unrestricted host path;
- changed checkout, stale policy, replaced runtime, expired session, invalid delegation and replay
  cannot inherit prior authority;
- interruption and controller failure produce bounded terminal and cleanup evidence;
- a required consequential capability is unavailable outside the admitted transaction; and
- the maintainer repeatedly operates and updates the integration without founder access and judges
  its in-session protection worth the maintenance and latency cost.

Pressure binds exact Core, adapter profile, implementation subject, registration/lifecycle,
installation, agent/runtime, OS/architecture and repository revisions. Unsupported routes remain
`not_proved`; no green workflow implies universal agent, host or repository governance.

## Non-Goals

- supporting every agent, editor, shell, connector, browser or operating system;
- inferring authority from agent identity, process ancestry or conversational intent;
- making every exploratory command a declared contract task;
- replacing V13 required verification or treating it as session containment;
- implementing session control where V13 and the provider's existing sandbox already solve the
  demonstrated operator problem;
- guaranteeing protection against privileged or kernel compromise;
- adding speculative provider adapters or V12.3-V12.5 carriers without their demand gates;
- implementing Enterprise fleet administration, monitoring, billing or support; or
- claiming that constrained execution proves code correctness or safe business outcomes.

## Definition Of Done

V13.1 completes only when one exact agent/runtime integration performs useful work, independently
enforces every claimed capability dimension, denies named bypasses, retains terminal evidence, and
is operated repeatedly by its maintainer. Unsupported tools, targets and observations remain
explicit. V13 verification remains independently valuable for agents that lack this integration.
