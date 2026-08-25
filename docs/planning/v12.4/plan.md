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

# V12.4: macOS Protected Authority Carrier

Status: planned and inactive. Feasibility, equivalent containment, implementation, packaging, and
pressure evidence are unproved.

## Activation Gates

V12.4 may be activated only after:

- V12.3 completes or is formally deferred;
- an independent macOS feasibility study identifies OS-supported primitives for protected service
  ownership, authenticated peer identity, exact child containment, terminal cleanup, durable
  recovery, and protected evidence storage;
- at least one real operator or design partner documents a current need for protected macOS
  crossing and commits to native pressure on the deployment posture it intends to adopt;
- the study demonstrates a boundary materially stronger than an ordinary same-user process;
- unsupported Linux assumptions are removed rather than emulated in names only; and
- the plan is re-reviewed against the then-current macOS security and service model.

If exact descendant containment and cleanup cannot be established with supported macOS APIs, this
slice remains blocked or ships a differently named weaker profile. It must not claim parity by
copying the Linux profile name.

Feasibility, broad macOS usage, or a public repository that happens to support macOS is not demand
evidence. Without the operator/design-partner requirement and immutable native pressure, V12.4
remains inactive and should be formally deferred rather than implemented for nominal parity.

Planned follow-on: [V12.5 Windows Protected Authority Carrier](../v12.5/plan.md) remains inactive
until V12.4 completes or is formally deferred.

## Product Boundary

The macOS carrier must provide the same semantic Ota guarantees as V11.7 where the OS can enforce
them: protected admission, exact selected child identity, one-use authority, execution only after
consumption, terminal cleanup, recovery, receipts, and archive verification. OS evidence and
carrier claims remain macOS-specific.

Candidate primitives include a root-owned `launchd` daemon, protected Unix-domain or XPC transport,
kernel/audit-token peer identity, retained process handles and creation identity, code-signature
and installation-manifest verification, process-group or equivalent descendant ownership, and
root-owned state. Activation must verify supported APIs rather than treating this list as proof.

The plan does not rely on deprecated `sandbox-exec`, caller-supplied code-signing labels, process
names, path spelling alone, or repository-controlled launchd configuration.

## Required Carrier Profile

One versioned macOS profile must define:

- supported macOS versions and architectures;
- daemon, client, broker, attestor, and executable installation identities;
- launchd service and configuration identity;
- transport path, ownership, permissions, peer-credential, and connection-lifetime semantics;
- code-signing designated requirement, Team ID posture, binary digest, and verifier authority;
- process identity, parent/child continuity, working directory, executable, actor principal, and
  target repository binding;
- descendant containment and the exact limits of what is and is not contained;
- stop, kill, reap, absence, crash, reboot, and orphan-recovery semantics;
- protected transaction, finalization, history, and verifier storage roots; and
- explicit differences from Linux/systemd evidence.

Unavailable evidence remains absent or `unknown`. Linux cgroup, systemd unit, namespace, and
`no_new_privs` claims cannot appear in a macOS profile.

## Trust, Recovery, And Evidence

- administrator installation completes before any repository job can observe or race authority
  creation;
- every existing ancestor and managed path is verified without symlink or alias redirection;
- peer identity remains guarded for the complete protocol session;
- signed evidence reconciles the exact prepared child, invocation, posture, working directory,
  selected semantic scope, and principal;
- active-slot and transaction intent persist before authority or selected work;
- crash/reboot recovery never resumes abandoned work and retains uncertain state until exact
  cleanup is proved;
- terminal success/refusal claiming removal requires child and contained-descendant absence plus
  finalization evidence; and
- public history remains non-secret and independently re-verified by Core.

## Implementation Order

1. Publish the feasibility/threat-model decision, including unsupported guarantees.
2. Add a macOS profile to `authority-protocol` without weakening Linux profile validation.
3. Implement protected installation and a root-owned daemon/client transport.
4. Implement process identity, containment, cleanup, recovery, and protected storage.
5. Integrate existing broker challenge, one-use lease, execution, receipt, and archive state
   machines through the platform-neutral carrier boundary.
6. Package reproducibly and add administrator install/uninstall/upgrade procedures.
7. Add native Intel and Apple Silicon adversarial tests.
8. Propagate operator documentation only after immutable native pressure passes.

Profile support must satisfy
[OSS Adapter and Profile Conformance](../adapter-profile-conformance/plan.md). Packaging and
administrator lifecycle must satisfy
[Authority Distribution Lifecycle](../authority-distribution-lifecycle/plan.md) in addition to the
macOS-specific acceptance bar below.

## Acceptance And Pressure Bar

- unprivileged repository execution cannot install, reconfigure, redirect, impersonate, or read
  protected authority state;
- wrong client, code signature, Team ID/verifier, executable, parent, process creation identity,
  working directory, repository, principal, transport, or launchd service refuses;
- symlink, hardlink, clone/alias, path-parent, bundle replacement, and configuration substitution
  refuse before mutation;
- peer exit or descriptor transfer during the bridge refuses;
- selected child and all carrier-claimed descendants are terminated and absent before terminal
  boundary-removal evidence;
- crash before authority, after decision, after consumption, during execution, during cleanup, and
  during history publication recovers fail-closed;
- reboot recovery and installer upgrade preserve or explicitly migrate protected state;
- one-use replay, expiry, revocation, cancellation, ambiguity, and lost acknowledgement retain
  existing authority semantics;
- Intel and Apple Silicon evidence use exact immutable Core, Protocol, carrier, package, and OS
  revisions; and
- pressure artifacts distinguish macOS carrier proof from provider attestation and host-global
  isolation.

Native pressure must run on real macOS hosts with administrator-owned installation. Virtualized or
CI controls may supplement but cannot replace at least one retained physical-host run for each
supported architecture unless the feasibility review proves the virtualization boundary itself is
the claimed deployment target.

## Non-Goals

- claiming Linux cgroup/systemd equivalence;
- deprecated sandbox mechanisms;
- provider-attested authority;
- Windows support;
- MDM, Endpoint Security, notarization service, or enterprise fleet management unless separately
  required and owned; or
- controlling raw shell and processes outside the selected Ota carrier boundary.

## Definition Of Done

V12.4 completes only if the named macOS profile has a defensible OS-native containment model,
reproducible distribution, exact recovery/cleanup evidence, Core archive re-verification, native
adversarial pressure, and bounded public claims. A working launch daemon alone is not completion.
