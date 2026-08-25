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

# V12.5: Windows Protected Authority Carrier

Status: planned and inactive. Feasibility, implementation, packaging, recovery, and native pressure
evidence are unproved.

## Activation Gates

V12.5 may be activated only after:

- V12.4 completes or is formally deferred;
- an independent Windows threat model proves a viable protected boundary using supported Windows
  service, token, process, IPC, ACL, job-object, and storage primitives;
- at least one real operator or design partner documents a current need for protected Windows
  crossing rather than read-only Ota behavior and commits to native pressure on the deployment
  posture it intends to adopt;
- supported Windows editions, architectures, filesystems, and service-account postures are named;
  and
- the plan is re-reviewed by an engineer experienced with Windows security boundaries.

Windows is not a port of the Unix launcher. It is a separate carrier profile sharing Ota's
authority semantics while using Windows-native enforcement and evidence.

Broad Windows usage, a public repository with Windows CI, or a passing service prototype is not
demand evidence. Without the operator/design-partner requirement and immutable native pressure,
V12.5 remains inactive and should be formally deferred rather than implemented for nominal parity.

Planned follow-on: [V12.6 OSS Enterprise Interoperability Foundation](../v12.6/plan.md) remains
inactive until V12.5 completes or is formally deferred. This link does not authorize an Enterprise
service or control-plane implementation.

## Product Boundary

The carrier must establish protected service ownership, authenticated client identity, exact
prepared-process continuity, one-use authority, selected execution containment, terminal cleanup,
durable recovery, and protected history.

Candidate primitives include a Windows Service under an administrator-selected service account,
named pipes with explicit ACLs and kernel-reported client process identity, retained process and
token handles, process creation-time identity, restricted tokens where required, Job Objects with
terminal child containment, protected `%ProgramData%` state, Authenticode/verifier identity, and
transactional or write-through file publication. Activation must validate exact semantics and
filesystem guarantees rather than assume them.

Local Administrator membership, a process name, path spelling, environment variables, inherited
handles, repository ACL assertions, or an unsigned executable cannot satisfy the profile.

## Required Carrier Profile

One versioned Windows profile defines:

- supported Windows editions, versions, architectures, and filesystem posture;
- service name, Service Control Manager configuration, account SID, privileges, and executable
  identity;
- client and job principal SIDs, token type, integrity level, elevation, group, privilege, and
  restricted-token posture;
- named-pipe name, security descriptor, server identity, client PID/process handle, connection
  lifetime, framing, impersonation, and handle-inheritance rules;
- Authenticode signer/verifier identity plus content digest and installation-manifest identity;
- process creation identity, parent/child continuity, command/application identity, working
  directory, repository binding, and target platform;
- Job Object configuration, nested-job compatibility, descendant assignment, breakaway refusal,
  completion/termination observation, and `KILL_ON_JOB_CLOSE` posture where applicable;
- protected storage roots, ACL inheritance, reparse-point/hardlink handling, durable publication,
  reboot recovery, and installer upgrade semantics; and
- exact claims unavailable on Windows or weaker than other carriers.

Unknown or unavailable facts remain explicit. Unix UID/GID, systemd, cgroup, launchd, and Unix
socket claims cannot appear in the Windows profile.

## Trust, Recovery, And Evidence

- administrator provisioning verifies service stopped state, absence of job-principal processes,
  acceptable existing authority state, protected ancestors, and exact package identities before
  mutation;
- every pipe session retains the client process handle and revalidates process creation identity,
  token, executable, and liveness before and after protocol traffic;
- the service creates or adopts the selected process into the exact Job Object before authority can
  become executable;
- no descendant may escape through breakaway, nested-job ambiguity, handle inheritance, alternate
  token, or untracked process creation while the carrier claims containment;
- transaction and cleanup intent are durable before externally visible state transitions;
- service crash, host reboot, installer interruption, and partial cleanup retain recoverable
  protected state; and
- Core independently re-verifies signed carrier, authority, transaction, receipt, and history
  evidence.

## Implementation Order

1. Complete the Windows feasibility study and threat model.
2. Add Windows-specific protocol profiles and canonical identities.
3. Implement reproducible signed packaging, administrator provisioning, service/client IPC, and
   protected configuration.
4. Implement token/process verification and Job Object containment before authority wiring.
5. Add durable transaction, recovery, cleanup, finalization, and protected history.
6. Integrate the existing broker decision and one-use lease state machines.
7. Add Windows-native fault injection and adversarial tests.
8. Propagate public operator guidance only after immutable native pressure passes.

Profile support must satisfy
[OSS Adapter and Profile Conformance](../adapter-profile-conformance/plan.md). Packaging and
administrator lifecycle must satisfy
[Authority Distribution Lifecycle](../authority-distribution-lifecycle/plan.md) in addition to the
Windows-specific acceptance bar below.

## Acceptance And Pressure Bar

- wrong service, SID, token, integrity/elevation posture, privilege set, executable, signer,
  content digest, pipe ACL, client process, creation time, working directory, repository, Job
  Object, or semantic scope refuses;
- reparse points, junctions, symlinks, hardlinks, alternate data streams, ACL inheritance changes,
  case-folding aliases, and path-parent substitution cannot redirect protected state or execution;
- caller handle inheritance, PID reuse, pipe transfer, token substitution, service restart, and
  executable replacement refuse;
- nested-job, breakaway, child-spawn, and process-tree pressure prove exact claimed containment or
  block activation;
- staged crashes cover provisioning, challenge, decision, lease consumption, process creation,
  job assignment, execution, cleanup, receipt, and history publication;
- reboot recovery never resumes abandoned selected work and never emits false cleanup evidence;
- one-use replay, expiry, revocation, cancellation, timeout, ambiguity, and lost acknowledgement
  preserve platform-neutral authority semantics;
- package upgrade/uninstall cannot discard uncertain protected state silently;
- Windows JSON, receipts, archives, and history reject carrier/profile downgrade or cross-platform
  substitution; and
- immutable pressure runs on every supported Windows architecture/edition with pinned Core,
  Protocol, carrier, package, service configuration, and verifier identities.

Pressure artifacts retain non-secret service/package/verifier identities, exact transaction and
cleanup evidence, host boot transitions, unchanged repository manifests outside expected selected
effects, and zero residual process, job, pipe, slot, or recovery state. Hosted CI is acceptable
only when it exposes the administrator and reboot semantics claimed by the profile; otherwise a
dedicated native host is required.

## Non-Goals

- emulating Unix launcher behavior or claiming cross-platform evidence identity;
- generic Windows endpoint management, Active Directory, MDM, or fleet administration;
- provider-attested authority;
- macOS support;
- controlling arbitrary elevated processes outside the selected Job Object; or
- treating Authenticode alone as workload, actor, or execution attestation.

## Definition Of Done

V12.5 completes only after the Windows profile proves protected provisioning, process/token
identity, Job Object containment, one-use authority, recovery, cleanup, receipt/archive
re-verification, reproducible installation, and native adversarial pressure. Compilation or a
passing Windows unit-test matrix is not carrier proof.
