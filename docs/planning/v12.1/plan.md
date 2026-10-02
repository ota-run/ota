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

# V12.1: Secret Delivery Governance

Status: active. Activated on 2026-09-02 after V12 closure, PythiaLabs credentialed-CAEP discovery,
and feasibility review of a concrete Google Secret Manager adapter using GitHub OIDC and Google
Workload Identity Federation. Implementation-order step 1 is independently reviewed and committed
at Core `9835edfa`. Implementation-order step 2 was separately activated on 2026-09-03 after the
connected v1.6.28 proof-assurance hardening passed immutable Linux/macOS Release Gate evidence and
that evidence was reconciled at Core `a582b948`; step 2 was independently reviewed and committed
at Core `9218151b`. Implementation-order step 3 was separately activated and committed at Core
`9ac9274f`; implementation-order step 4 was separately activated at Core `a7dd07ba` and committed
at Core `f1178fe3`. Implementation-order step 5 was separately activated and committed at Core
`9fd4b4fb`; implementation-order step 6 was separately activated at Core `2dd20ab8` and committed
at Core `67de2b4d`. Implementation-order step 7 was independently reviewed and activated by the
amendment committed at Core `ad14ae8b`. That activation does not establish provider support,
secret delivery, execution authority, or pressure evidence; Step 8 and later steps remain
unauthorized.

## Activation Gates

V12.1 may be activated only after:

- V11.7 has completed its authority, transaction-lifetime, cleanup, archive, and pressure bar;
- V11.22 has shipped reviewed candidate creation and atomic application;
- V12 has shipped typed effect identity, realization identity, shared admission, and archive
  re-derivation;
- one real repository/provider case demonstrates a material delivery boundary that existing
  fail-closed behavior cannot express; and
- activation names one concrete provider adapter capable of late, transaction-bound delivery.

V12.1 must not widen V11.7 authority transport or reinterpret a crossing grant as secret-delivery
authority. Crossing admission and secret-delivery admission are separate and both must pass.

## Activation Record

- V11.7, V11.22, and V12 completed the prerequisite authority, candidate, typed-effect, shared
  admission, archive, pressure, and closure bars named above.
- PythiaLabs supplies the real repository boundary: its credentialed CAEP lane requires
  `GOOGLE_API_KEY` for consequential model-provider calls, while inherited GitHub environment
  state cannot prove late delivery to the exact selected recipient or exclusion from adjacent
  tasks, hooks, services, helpers, logs, and replay state.
- The first implementation subject is
  `google_secret_manager_github_oidc_process_environment_v1`, limited initially to a
  `linux/x86_64` GitHub Actions native-process recipient. After all Ota admission succeeds, the
  adapter requests the exact workflow-run OIDC token, exchanges it through Google Workload Identity
  Federation, reads one exact Google Secret Manager version, and injects the returned bytes only
  into the selected recipient process-tree environment.
- Before implementation-order step 3 may begin, the adapter profile must require and bind the exact
  GitHub OIDC issuer, audience, and admitted claims; Workload Identity Federation pool and provider;
  service account; Google project; Secret Manager resource; and numeric secret version. Every field
  is mandatory canonical identity input, and substitution of any field must refuse.
- GitHub's OIDC request capability and the resulting short-lived Google credential authenticate
  provider access; neither is the delivered repository secret or proof that delivery succeeded.
  A GitHub secret, workflow expression, action output, generated credentials file, inherited
  environment value, or shell bootstrap cannot substitute for the adapter transaction.
- Initial pressure will use Bobai's fork, a disposable Google Cloud provider scope, and synthetic
  canary material. It will not require an upstream maintainer secret or treat a model-provider call
  as proved until that separate consequential action is explicitly selected and evidenced.
- Local, macOS, Windows, container, remote, persistent-runtime, dynamic-lease, and additional
  provider delivery remain unsupported until separately implemented and pressured.

Activation does not make PythiaLabs `ALLOW` evidence into Ota authority, does not activate V12.2,
and does not authorize provider contact before the shared admission and transaction boundaries
ship.

### Step 2 Activation Gate

Step 2 is not authorized by the original step-1 activation record. It may be activated only through
an explicit plan and current-state amendment after the v1.6.28 proof-assurance hardening is committed
and its Linux/macOS Release Gate evidence is inspected. Clearing those prerequisites does not
activate step 2 by itself. The activation amendment must name the exact provider-binding ownership
and disclosure boundary being implemented and must preserve zero provider contact.

#### Step 2 Activation Amendment

Implementation-order step 2 is activated on 2026-09-03. The prerequisite proof-assurance
hardening is committed at Core `876680f074777aada0b3910a62aab9b245b34af7`; Release Gate
[run 33744200940](https://github.com/ota-run/ota/actions/runs/33744200940) passed the live valid
sibling-attestation substitution boundary on Ubuntu and macOS, and Core `a582b948` records the
inspected immutable evidence.

This amendment authorizes only a Core-owned provider-binding domain model and pure resolver. The
resolver consumes an explicitly authority-sourced protected snapshot that is separate from
`ota.yaml`; contracts, tasks, workflows, environment variables, and caller labels cannot create,
redirect, or elevate a binding. The private canonical record binds the exact requirement, provider
reference, authority scope, source evidence, adapter/capability constraints, and lifecycle posture.
Zero matches, duplicate matches, conflicting sources, unknown selectors, and substitution of any
identity input refuse without precedence fallback.

The private identity retains every material provider-reference input. Public output may expose only
the provider/binding class, disclosure class, and approved non-secret semantic or projection
identities that Core can reconcile with the private record. It must not expose secret values;
secret- or provider-reference-derived hashes, lengths, prefixes, or transformed values; raw
provider paths; secret names; tenancy details; workload metadata; credentials; descriptors; or
reusable handles. A redacted projection cannot become a selector or authority source.

Step 2 does not authorize a filesystem or control-plane loader, a new CLI route, provider adapter
registration, GitHub OIDC, Workload Identity Federation, Google Secret Manager contact, delivery
policy, effect derivation, admission, materialization, injection, execution, receipts, archives, or
support claims. Those boundaries remain owned by later implementation-order steps and require their
stated gates.

### Step 3 Activation Gate

Step 3 is not authorized by either earlier activation record. It may be activated only after the
step-2 provider-binding model is independently reviewed and committed. The activation amendment
must preserve the adapter/profile conformance separation between shared profile semantics, the
exact implementation subject, registration evidence, and release-owned lifecycle. It must also
freeze the first adapter's complete binding and target posture without implying that an
unimplemented adapter has source, build, artifact, conformance, pressure, registration, or support
evidence.

#### Step 3 Activation Amendment

Implementation-order step 3 is activated on 2026-09-03. The prerequisite provider-binding model is
independently reviewed and committed at Core `9218151b`.

This amendment authorizes only the Core-owned, crate-private profile definition and implementation-
subject descriptor model for `google_secret_manager_github_oidc_process_environment_v1`, together
with deterministic semantic identities and adversarial substitution tests. The profile definition
must bind its materialization applicability class; selected-recipient-process-tree boundary;
`process_environment` destination; `linux/x86_64` GitHub Actions native-process target;
provider-neutral requirement and protected-binding inputs; provider interaction, cleanup,
interruption, and disclosure capabilities; and every unsupported or unproved target and behavior.
The implementation-subject descriptor must bind the exact profile identity, implementation owner,
source, build, artifact, claimed Core and Protocol ranges, and exact target posture. Until those
implementation inputs exist, no concrete implementation-subject identity may be finalized or
registered.
Applying those identity and separation rules to this named V12.1 model does not activate the
cross-cutting adapter/profile conformance plan, its generic registry, registration packages,
conformance harness, pressure lifecycle, or release lifecycle.

Identity ownership is explicit. `profile_semantic_identity` binds the stable required claim
vocabulary, canonicalization and validation semantics, capability boundary, and permitted target
set; it does not bind concrete workflow-run or provider-reference values.
`implementation_subject_identity` binds the exact implementation owner, source, build, artifact,
claimed Core and Protocol ranges, and `linux/x86_64` GitHub Actions native-process target; it does
not vary per invocation. A separate protected `SecretDeliveryInvocationBindingIdentity` binds the
selected requirement and protected provider-binding identities plus the exact per-invocation
binding tuple. Registration and lifecycle identities remain uncreated.

That protected per-invocation tuple must contain the exact GitHub OIDC issuer and audience; the
complete admitted claim-name and expected-value map, including subject, repository and repository-
owner numeric identities, workflow reference and revision, repository ref and revision, actor
numeric identity, event, run identity, and run attempt; Workload Identity Federation pool and
provider; service account; Google project; Secret Manager resource; and numeric secret version.
Every field and every admitted claim is a mandatory canonical input to
`SecretDeliveryInvocationBindingIdentity`. Omission, duplication, aliasing, or substitution must
refuse rather than fall back to another binding or target. Semantically unordered claim
collections must normalize to one canonical order; any ordered claim input must preserve its
semantic order in identity.

The only initially permitted target is `linux/x86_64` GitHub Actions using native execution and a
transient selected recipient process tree. Other architectures, local, macOS, Windows, container,
remote, persistent-runtime, dynamic-lease, raw-shell, undeclared-descendant, additional-provider,
and mutable-secret-version postures must be represented as unsupported or unproved and must refuse
selection. The profile may describe required future provider and cleanup behavior, but this step
cannot claim that behavior is implemented, observed, pressured, or supported.

Step 3 does not authorize a binding loader, registry installation, implementation registration,
lifecycle promotion, CLI route, OIDC token request, Workload Identity Federation exchange, Google
Secret Manager contact, network access, effect derivation, policy or command admission,
materialization, injection, child execution, receipt, archive, assurance, conformance result,
pressure evidence, or support claim. Implementation-order step 4 and every later step remain
unauthorized.

### Step 4 Activation Gate

Step 4 is not authorized by any earlier activation record. It may be activated only after the
step-3 profile and implementation-subject model is independently reviewed and committed. The
activation amendment must preserve one canonical V12 effect-policy evaluator, keep provider and
secret material outside effect identities and public projections, and distinguish stable bounded
consequence truth from exact realization and invocation truth. Clearing those prerequisites does
not activate step 4 by itself.

#### Step 4 Activation Amendment

Implementation-order step 4 is activated by the commit carrying this amendment after independent
review. The prerequisite profile and implementation-subject model is independently reviewed and
committed at Core `9ac9274f`; Core `68b652db` records its completed status and first-party
reconciliation. Hosted checks for that exact closure revision remain a separate verification gate
and must not be represented as completed evidence until their terminal results are inspected.

This amendment authorizes only a Core-owned, crate-private derivation model for the
`secret_material_delivery` V12 effect, its realization inputs, and its internal refusal-assurance
eligibility profile, plus the narrow extension of the canonical V12 effect-policy evaluator needed
to consume that derived effect. It must not create a secret-specific or provider-specific parallel
policy evaluator, authority path, admission result, or decision vocabulary.

`EffectIdentity` binds stable bounded consequence truth: effect kind, secret class, purpose,
canonical destination class, environment/resource bounds, and the material consequence of making
the secret available through that destination. It must not bind or disclose secret bytes, a secret-
derived value, a private provider reference, a requirement label, a selected recipient, provider
subject, workflow-run claim, or invocation origin. Two requirements with the same canonical
bounded consequence may share an effect identity; changing destination or bounded consequence
must produce a different identity.

The realization identity binds the exact selected `SecretRequirementIdentity`, recipient segment
and process-tree boundary, selected closure role and invocation origin, protected provider-binding
and source-evidence identities, profile semantic identity, implementation-subject identity, and
exact target-execution posture. Reusing one requirement through different recipients, origins,
roles, subjects, or targets must retain distinct realization identities. Substituting protected
binding or source evidence must change or invalidate the realization without changing the effect
identity when the bounded consequence is unchanged.

Derivation must start from the validated provider-neutral requirement and the exact selected
closure. It must reconcile the required recipient, destination, execution constraints, protected
binding and source authority, profile, implementation subject, and target posture before policy
evaluation. Missing, ambiguous, conflicting, unsupported, unverified, noncanonical, or substituted
inputs refuse structurally. Neither an effect-policy allow nor any secret-delivery policy outcome
may manufacture a missing requirement, binding, source authority, profile, subject, recipient, or
target capability; policy may only narrow already valid authority.

The internal refusal-assurance profile may describe only derivation eligibility, identity inputs,
and fail-closed reasons needed by later command admission. It is not a canary, Doctor claim,
receipt, archive, public JSON result, or positive assurance statement. Step 4 does not authorize a
binding loader, CLI route, dry-run delivery plan, command-scoped admission, OIDC token request,
Workload Identity Federation exchange, Google Secret Manager contact, network access,
materialization, process-environment injection, child execution, receipt, archive, public output,
assurance promotion, pressure evidence, or support claim. At the step-4 activation boundary,
implementation-order step 5 and every later step remained unauthorized.

### Step 5 Activation Gate

Step 5 is not authorized by any earlier activation record. It may be activated only after the
step-4 effect derivation and canonical policy integration are independently reviewed and committed.
The activation amendment must retain one evaluator over independently reverified requirement,
selected-graph, realization, binding/source, profile/subject, invocation, and policy-decision truth;
it must not reload or reconstruct command-scoped truth, contact a provider, or convert provider
availability into an assumption. Clearing those prerequisites does not activate step 5 by itself.

#### Step 5 Activation Amendment

Implementation-order step 5 is activated by the commit carrying this amendment after independent
review. The prerequisite sealed effect derivation and canonical policy integration are independently
reviewed and committed at Core `f1178fe3`. Hosted checks for that exact implementation revision are
useful background verification but are not represented as a Step 5 activation prerequisite or as
provider evidence.

This amendment authorizes only one crate-private secret-delivery evaluator and provider-free dry-run
plan. The evaluator derives the selected requirement set from the retained contract and exact
task/workflow graph, then consumes independently verifiable Step 1-4 domain records supplied by its
caller: the ordered selected invocation graph, the
resolved binding and source, profile and implementation subject, protected invocation binding,
derived effect/attachment/realization records, and the exact canonical effect-policy decision. It
must call the existing semantic verifiers rather than trust a self-consistent resolved record or
create a parallel secret-specific policy decision.

The evaluator must fail closed on missing, duplicate, reordered selected-invocation, stale,
substituted, cross-scope, unsupported, or unreconciled input. Selected invocation order is semantic
and must be retained exactly; unordered semantic sets are canonicalized so caller ordering cannot
change identity. Every realization must retain its exact selected invocation mapping. An effect-
policy deny remains a refusal. An allow or warn may establish only structural eligibility for a
later provider check; it is not provider authority, availability, delivery, execution admission,
or assurance. Policy cannot manufacture a missing requirement, binding, source posture, profile,
implementation subject, invocation binding, selected occurrence, or target capability.

The dry-run plan must bind the exact evaluation inputs and decision identities while reporting
`availability: not_checked`, `provider_contact: not_attempted`, `delivery: not_attempted`, and
`execution_started: false`. It must not contain a secret value, secret-derived hash, length, prefix,
transformation, private provider reference, credential, token, or provider response. It must not
describe a structurally eligible lane as runnable, available, delivered, supported, or assured.
No public JSON schema or command output is authorized at this step.

Only a Core-derived empty selected requirement set is `not_applicable` and must not inspect
unrelated protected bindings. A non-empty set requires complete Step 1-4 reconciliation and one
unambiguous decision per exact realization. The evaluator and plan identities must be domain-
separated and independently recomputable from retained non-secret identity inputs; caller ordering
of semantic sets must not change them, while selected invocation order remains semantic.

Step 5 does not authorize a provider-binding loader, policy-authoring surface, CLI route, `run`,
`up`, proof, Doctor, CI, sandbox, or harness consumer, command-scoped admission, OIDC token request,
Workload Identity Federation exchange, Google Secret Manager contact, network access,
materialization, process-environment injection, child execution, receipt, archive, public output,
assurance promotion, pressure evidence, or support claim. Implementation-order step 6 and every
later step remain unauthorized.

### Step 6 Activation Gate

Step 6 is not authorized by any earlier activation record. It may be activated only after the
step-5 evaluator and provider-free dry-run plan are independently reviewed and committed. The
activation amendment must preserve one retained command-scoped admission per command invocation,
with CI projection carrying only a bounded public expectation and provider-checkout CI independently
deriving its own admission. It must keep contracts without selected secret requirements compatible,
define an honest negative output boundary for governed requirements while no production binding
loader or provider transaction exists, and preserve zero provider contact and zero secret delivery.
Clearing those prerequisites does not activate step 6 by itself.

#### Step 6 Activation Amendment

Implementation-order step 6 is activated by the commit carrying this amendment after independent
review. The prerequisite sealed evaluator and provider-free dry-run plan are independently reviewed
and committed at Core `9fd4b4fb8f991b7dca8c2a2df2c2b6be5e5a0baa`.

This amendment authorizes one command-scoped secret-delivery admission object per command
invocation and the minimum orchestration needed for `run`, `up`, proof commands, Doctor context,
sandbox capability, and harness output to consume it. Each invocation must derive the selected
subject and ordered execution graph once, retain one requirement selection and one Step 5
evaluation/plan, and pass that exact admission to every downstream consumer within that invocation.
No downstream consumer within the invocation may reload the contract, re-resolve bindings, derive
another invocation origin, reconstruct policy truth, or create a parallel secret-specific admission
decision.

CI projection and provider-checkout CI re-evaluation are separate command invocations. Projection
may carry only a bounded public-safe expectation derived by the render invocation. The provider-
checkout invocation must independently derive current contract, selected graph, policy, protected
truth, and command-scoped admission from the exact checkout, then reconcile its public projection
with the projected expectation. It must never inherit or reuse render-host admission, policy,
binding, source, evaluation, or plan truth.

Only a Core-derived empty selected requirement set is `not_applicable`; it must preserve existing
behavior and must not load or inspect secret-delivery policy, bindings, profiles, or provider state.
A non-empty selected set may become `structurally_eligible` only from complete independently
reverified Step 1-5 records. Missing production binding/source, profile/subject, invocation,
effect/policy, or evaluator truth must produce one stable unavailable or refused admission before
setup, hydration, workflow environment rendering, durable execution-log creation, replay, crossing,
sandbox execution, services, hooks, proof artifacts, child creation, repository mutation, or any
provider interaction. An unavailable production source must not be represented as policy denial,
provider unavailability, or `not_applicable`.

Dry-run and every other public consumer must use a separately derived public projection and
domain-separated projection identity over only fields individually classified as public-safe. The
projection may retain bounded applicability, stable reason codes, approved non-secret contract or
effect classes, and `availability: not_checked`, `provider_contact: not_attempted`,
`delivery: not_attempted`, and `execution_started: false`. Private effect-policy decision,
realization, evaluation, plan, binding, source, implementation-subject, and invocation identities
remain internal unless a later plan explicitly classifies one exact projection as public-safe. A
public projection must not transitively hash, correlate, or reveal protected provider-reference or
authority truth.

Real execution must refuse after the same structural admission because no Step 7 provider
transaction or delivery adapter exists. Doctor, CI, sandbox, and harness projections may report
only the same bounded public applicability and refusal posture; they cannot promote a lane to
callable, runnable, available, delivered, supported, or assured. Human and JSON output must use
stable reason codes, preserve `--plain`, and reject locally contradictory states through published
schemas where a public schema exists.

The implementation must regression-lock task and workflow roots, dependencies, hooks, repeated
roles, proof-only roots, selected-mode exclusion, dry-run/real-execution parity, malformed or absent
protected truth, and denial before every material side-effect boundary. Existing contracts with no
selected secret requirements must retain their current output and execution behavior. Connected
Core references, command/output specifications, JSON schemas, Site reference/docs, Learn, FAQ,
Glossary, standalone Examples, and canonical Skills/mirrors must each be updated when affected or
recorded explicitly as unaffected.

Step 6 does not authorize a production provider-binding loader or new authority source, provider
adapter registration, OIDC token request, Workload Identity Federation exchange, Google Secret
Manager contact, network access, materialization, process-environment injection, secret-bearing
child execution, positive secret-delivery receipt, secret-delivery archive, positive assurance,
pressure evidence, or support claim. Real `ota up` retains its pre-existing generic blocked
execution receipt with `execution_attempted: false`; that negative failure record is not delivery
evidence and grants no authority.
Implementation-order step 7 and every later step remain unauthorized.

### Step 7 Activation Gate

Step 7 is not authorized by any earlier activation record. It may be activated only after the
Step 6 command admission and its connected Site and Skill propagation are independently reviewed,
committed, immutably reconciled, and pushed. The activation amendment must name one production
provider-binding loader whose authority cannot be created or redirected by `ota.yaml`, repository
content, workflow YAML, inherited environment, CLI arguments, or caller labels. It must also define
the exact provider transaction, recipient process-tree, interruption, and cleanup boundaries before
any OIDC request or provider contact is implemented. Clearing those prerequisites does not activate
Step 7 by itself.

#### Step 7 Activation Amendment (Active At Core `ad14ae8b`)

The prerequisite Step 6 implementation is independently reviewed and committed at Core
`67de2b4d9d6773e8b7ea0506669d4635f5feaf3f`; immutable Site and Skills reconciliation is committed
at Core `9d6696f02b939e9366cffe4bd6b7121ede89d822`, and the completion handoff is committed at Core
`0d5bc93b`. Independent review found no remaining material issue, and Core `ad14ae8b` committed and
activated this amendment. Only the exact boundaries below are authorized.

#### Canonical Backend-Selected Execution Graph Correctness Amendment (Active 2026-09-08)

Discord contract pressure proved that aggregate mode eligibility can currently consume
`all_depends_on` inventory truth while the runner executes a backend-selected dependency graph. A
native-only dependency from an unselected mode branch can therefore make a valid container
aggregate appear unavailable. V12.1 admission already depends on selected execution-graph truth,
so this is an active correctness dependency, not a new secret-delivery capability.

This amendment authorizes only the aggregate mode-eligibility repair defined in the
[execution-contract follow-on plan](../execution-contract-follow-ons/plan.md): reuse the existing
runner planner as the canonical derivation; preserve all-branch inventory for structural
validation; make execution-facing consumers use the selected graph; bind ordered selected graph
semantics into current evidence; retain unreconstructable historical evidence as
`legacy_unverified`; and prove the exact Discord native/container divergence without changing its
contract first. It does not authorize another resolver, mixed-backend aggregate execution, new
contract syntax, secret delivery, provider contact, or broader Step 7 claims.

The first executable target is narrower than generic GitHub Actions: `linux/x86_64`, native
execution, a transient selected recipient process tree, and an administrator-controlled self-hosted
runner with a protected launch boundary. GitHub-hosted runners, repository-provisioned runners,
local execution, macOS, Windows, containers, remote execution, persistent runtimes, raw shell, and
unowned descendants remain unsupported or unproved. This is a capability and target change, so
Step 7 must introduce a new versioned protected-runner profile and
`profile_semantic_identity`; it cannot reinterpret
`google_secret_manager_github_oidc_process_environment_v1`. The new profile binds the protected
launcher, retained process ownership, transport, and target semantics. Its concrete implementation
subject separately binds the exact launcher and Ota build artifacts and the `linux/x86_64`
protected-runner target. Registration and lifecycle identities remain uncreated.

Root-owned files alone do not establish authority. The selected job must enter Ota through one
administrator-installed root service exposed through a fixed root-owned Unix socket; the repository
runner account has permission to request an admitted launch but has no `sudo` authorization or
other privilege-escalation path. The service's executable, socket, request protocol, launch policy,
service account, authority descriptors, and cgroup-v2 ownership are outside repository and workflow
control. Its versioned request accepts only an untrusted checkout descriptor, one selected Ota
command/task-or-workflow identity, and the two OIDC capability values; Ota re-derives the checkout,
contract, selection, and claims instead of treating request fields as authority. The service accepts
no caller-selected executable, authority path, inherited environment, network setting, privilege,
or arbitrary launch option. It opens the authority store, creates one invocation cgroup, and passes
retained read-only descriptors plus a fresh launch capability to the fixed unprivileged Ota
implementation subject. Ota must refuse when its
real, effective, or saved identity is root; when any effective, permitted, inheritable, or ambient
Linux capability is present; when `no_new_privs` is absent; when its parent, launch policy, service
identity, retained descriptors, or cgroup do not reconcile; or when repository code can select or
replace the launcher inputs. A domain-separated `ProtectedLauncherCapabilityIdentity` binds the
launcher executable and policy identities, runner-administrator identity, bounded request identity,
service UID/GID,
invocation nonce, boot identity, cgroup identity, inherited socket and authority-descriptor metadata,
and exact Ota implementation-subject identity. Ota derives these facts from the retained descriptors,
Linux credentials, process state, and cgroup state rather than trusting caller fields. Admission and
the secret-delivery transaction both bind that identity. A normal root shell, `sudo`, repository-
owned wrapper, direct invocation of the Ota binary, or caller-constructed local descriptor cannot
satisfy it.

Before the production observation-service route may construct that capability, the root installer
must create one closed, versioned `ProtectedLauncherAuthorityContextV1` at a fixed protected path
and bind its exact file identity as a singular role in `ProtectedInstallationManifestV1`. Its
outer identity binds one separately domain-derived `RunnerAdministratorAuthorityIdentity` and one
distinct `ProtectedLauncherImplementationSubjectV1`; the capability receives the administrator
identity, never the mutable outer-context identity. It is not the Step-3 Google provider-adapter
subject. The Launcher implementation subject must separately bind the exact Launcher and Ota
source, build, installed-artifact, Protocol/Core compatibility, protected-launcher profile, and
`linux/x86_64` target truth, then rederive against the retained installation manifest and executable
identities before capability reconciliation. The root Launcher, after accepting the canonical
invocation but before child/scope creation, generates a fresh 256-bit invocation nonce and derives
`invocation_nonce_identity` under its own Protocol domain. It obtains `boot_identity` only from a
retained and immediately reobserved descriptor opened beneath a verified `PROC_SUPER_MAGIC` procfs
root at the exact regular-file path `sys/kernel/random/boot_id`, with no symlink, alias, or mount
substitution; it parses the canonical UUID and derives the identity under its dedicated domain. The
raw nonce and boot value remain protected. A request, workflow, environment variable, config label,
or fixture-only identity cannot supply any of these values. The context constructor remains
crate-private and takes retained authority, nonce, and boot observations rather than four free
identity strings; authority-record, implementation-subject, executable, nonce, and boot
substitution must refuse even when their outer identities are recomputed.

`ProtectedLauncherCapabilityIdentity` is protected transaction truth, not a public CI identity: it
transitively binds protected authority-store and descriptor observations. The launcher may retain it
only in the protected invocation and Core transaction carriers. A hosted compatibility workflow may
receive only a separately derived, domain-separated public capability-observation projection whose
inputs are individually approved as public-safe and which cannot hash, correlate, or reveal the
protected capability, store, descriptor, request, binding, source, or invocation identities.

The root-owned launcher is the sole derivation owner. For one probe it accepts the Core-generated
fresh public challenge, canonicalizes and reserves its identity before child preparation, derives
the protected capability from the same retained child, cgroup, session, and authority observations,
records the private capability-to-challenge reconciliation link in the protected transaction, and
consumes the challenge exactly once before returning the projection. The caller cannot select a
capability, descriptor, authority path, profile, signature, or completion state. Missing, malformed,
duplicate, expired, replayed, or substituted challenges and a missing, duplicate, stale, replayed,
or substituted projection must refuse. A recovered or copied projection cannot satisfy a new
workflow because Core generates and retains its expected fresh challenge before contacting the
launcher and requires exact equality after signature verification. The private reconciliation link
and raw capability identity are never serialized into workflow, log, artifact, receipt, archive, or
public output.

The fixed local Core-to-Launcher observation request also binds the expected protected Launcher
invocation-request identity. The Launcher recomputes that identity from its accepted invocation and
refuses a mismatch before reserving replay state, capability derivation, or Attestor signing. This
identity is protected local transport truth: it is not included in the public challenge, projection,
workflow output, log, artifact, receipt, or archive. The protected local response retains only its
domain-separated request-identity commitment for exact reconciliation; it never directly serializes
the raw invocation identity, and neither value enters the public projection or workflow output.

`ProtectedLauncherCapabilityObservationChallengeV1` is a closed, shared Protocol record. It contains
exactly `schema_version: 1`; message kind
`protected_launcher_capability_observation_challenge`; a domain-separated `identity`; canonical
positive-decimal GitHub workflow run and attempt identifiers; the canonical exact workflow reference;
a 256-bit Core-generated random nonce commitment; and canonical issued/expiry Unix seconds. The raw
nonce is passed only in the fixed local launcher request and is never logged, persisted in a public
artifact, or projected. Core generates it from the operating-system CSPRNG immediately before the
probe, permits a maximum five-minute lifetime, and retains the expected challenge only for that
workflow invocation. Launcher recomputes the commitment and identity, requires issuance not in the
future and expiry within the bounded lifetime, and atomically reserves the challenge identity in one
fixed root-owned, no-follow, durable replay store before child preparation. It records consumption
only after protected derivation and projection signing; an unexpired reservation, a consumed record,
or any filesystem/lock/durability ambiguity refuses. The Protocol owns canonicalization and identity
derivation; the root launcher owns durable reservation/consumption; Core owns its in-memory expected
challenge reconciliation. No workflow file, environment value, artifact, or client response may
replace those owners.

#### Protected Boot Observation Profile Correction (Proposed 2026-09-09)

The immutable `ota.authority-launcher.systemd/v3` profile and public class
`systemd_protected_launcher_v3` remain the historical owners of the retained governed-invocation
evidence. They must not be reinterpreted. Live Linux/X64 pressure localized authority-context
refusal, and a separate bounded systemd reproduction proved that V3's `ProtectProc=invisible` plus
`ProcSubset=pid` removes `/proc/sys/kernel/random/boot_id` from the Launcher service namespace. A
narrower bind-only attempt also refused because systemd applied the proc subset before resolving
the bind source. The activated protected-capability design requires the Launcher to retain that
exact kernel-owned value without widening the procfs namespace inherited by the selected child.
The resulting V3 refusal is correct.

The correction may introduce only `ota.authority-launcher.systemd/v4` and public class
`systemd_protected_launcher_v4`. V4 inherits the complete V3 profile, retains
`ProtectProc=invisible` and `ProcSubset=pid`, and adds exactly one systemd-manager-owned
`OpenFile=/proc/sys/kernel/random/boot_id:ota-boot-id:read-only` service setting. The target must run
systemd 253 or newer, where `OpenFile=` is supported. No other service-unit change is authorized:
the protected history and execution-disabled broker units retain their explicit `ProcSubset=pid`
settings, while the Attestor retains its existing profile without either proc setting. The one
connected socket-unit change is the exact listener descriptor name required below.

The Launcher socket unit must set `FileDescriptorName=ota-launcher-listener`. V4 accepts exactly one
complete inherited descriptor set: `LISTEN_PID` equals the current Launcher process, `LISTEN_FDS=2`,
and `LISTEN_FDNAMES` contains exactly one `ota-launcher-listener` and one `ota-boot-id`, with no
empty or additional role. Both descriptors are selected by their names; descriptor position is not
authority and reordering does not change meaning. Missing, duplicate, unnamed, additional, or
role-substituted descriptors refuse. The named listener must independently satisfy the existing
AF_UNIX listener, path, ownership, group, and mode checks. The named boot descriptor must be
read-only, regular, root-owned, non-writable, procfs-backed, and bound to the exact manager-opened
service setting; alias or substitution refuses.

The Launcher sets close-on-exec on both descriptors immediately, verifies the retained boot
descriptor's filesystem, type, owner, mode, and canonical UUID bytes, derives the boot identity
under the existing Protocol domain, and reobserves the same descriptor immediately before
capability reconciliation. It must not reopen the path from its restricted namespace. The
selected-child boundary must close the boot descriptor before the child stops or executes, and
Linux pressure must inspect the child descriptor table and prove the named boot descriptor is
absent. The existing close-all-except-explicit-retained-descriptors rule remains authoritative; the
boot descriptor may never enter that retained child allowlist.

The Launcher must verify the effective V4 systemd settings before protected-capability derivation,
and the protected capability, implementation subject, installation evidence, observation
projection, and Core compatibility verifier must bind V4 rather than accepting V3 as an alias. V3
and V4 identities and classes must be substitution-tested and must not compare equal.

This correction exposes only one PID-1-opened read-only boot-ID descriptor to the already root-owned
protected Launcher observer. It does not widen the Launcher or selected child's procfs namespace
and grants no new filesystem write, network, provider, credential, execution, or repository
authority. A successful fixture proves only the bounded boot-observation and capability-derivation
path under the V4 profile; it does not prove accepted-session provenance, OIDC exchange, provider
contact, materialization, delivery, positive provider evidence, Step 8, or V12.2.

This correction was independently reviewed and activated at Core
`7a86c9313921ba012fb966c527e9c901a2b18e66`. That activation authorizes only the bounded V4
Protocol, Launcher, Core compatibility, and hosted-proof work described above. It does not
authorize provider contact, delivery, Step 8, V12.2, or a broader public capability claim.

The first public `ProtectedLauncherCapabilityObservationProjectionV1` is closed and rejects unknown
fields. Its unsigned canonical payload contains exactly: `schema_version: 1`; evidence kind
`protected_launcher_capability_observation`; the canonical public challenge identity;
`derivation: verified`; target posture `environment: self_hosted`, `os: linux`,
`architecture: x64`; the approved class `systemd_protected_launcher_v4`; the observed canonical
runner version; and the launcher public signing-key identity. `projection_identity` is the SHA-256
identity of precisely that JCS payload under
`ota.protected-launcher-capability-observation-projection.v1\0`; it is not an input to its own
derivation. The outer projection adds only that `projection_identity` and one Ed25519 signature over
the canonical byte sequence `ota.protected-launcher-capability-observation-signature.v1\0` followed
by the projection identity. The signature is not an input to either payload or identity derivation.

The projection contains no raw capability or protected-transaction identity, descriptor or store
metadata/content identity, request or repository path, binding/source/provider reference, principal,
PID, cgroup, nonce, boot identity, implementation subject, timestamp, token, credential, response,
or provider result. `verified` is the sole initial derivation state. The signer is the root-owned
launcher/attestor boundary; Core's compatibility verifier owns schema, closed-vocabulary, payload,
identity, signature, expected-challenge, one-result, and target/profile/runner-version
reconciliation. It must reject any other state or public class. The projection is neither authority
nor evidence of provider contact, delivery, execution approval, or cleanup beyond the exact retained
launcher result.

Core obtains the projection verifier only from the administrator-installed fixed path
`/var/lib/ota/authority-launcher-public/capability-projection-verifier-v1.json`. This closed, versioned,
root-owned regular record has a protected ownership chain, no symlink or writable ancestor, and
contains exactly `schema_version: 1`; record kind `protected_launcher_capability_projection_verifier`;
its domain-separated record identity; one approved Ed25519 public key; that key's identity; key usage
`protected_launcher_capability_observation_projection`; and required signature domain
`ota.protected-launcher-capability-observation-signature.v1\0`. The root installer writes it
atomically and binds its record identity into the administrator-controlled public installation
evidence; the launcher signs only with the matching protected attestor key. Core reopens and
rederives the fixed record and installation evidence before signature verification, requires the
projection signer-key identity, key usage, and signature domain to equal the independently loaded
record, then verifies the signature over the specified domain and identity. It must never trust a
key, key identity, trust root, or verification result from the projection, caller, workflow,
repository, CLI, environment, or artifact; the projection signature is untrusted verification input
only. Missing, duplicate, substituted, malformed, unverifiable, or installation-mismatched verifier
records refuse.

The launcher reads `/etc/ota/secret-delivery/verifiers-v1.json` and
`/etc/ota/secret-delivery/bindings-v1.json` from a root-owned `0700` authority directory. Every path
component must be a real root-owned directory that is not group- or world-writable. Linux lookup
must use an `openat2`-equivalent beneath the retained authority-directory descriptor with
no-symlink, no-magic-link, no-mount-crossing resolution; final-component `O_NOFOLLOW` alone is
insufficient. Both files must be regular, root-owned, mode `0400`, versioned, and opened before the
unprivileged Ota process starts. Launcher retains and reconciles descriptor device, inode, owner,
mode, size, and exact bytes before starting that process, before producing any protected snapshot,
and again before binding a transaction candidate. Core receives no filesystem path or descriptor
authority from this observation; it independently validates the closed transferred records and
parses the opaque signed payload under a Core-owned schema. The binding
bundle must be domain-separated and Ed25519-signed by exactly one admitted verifier from the
protected verifier store. The verifier store names the authority, admitted key, signature domain,
validity bounds, signed generation, and exactly one active binding-bundle identity; a merely valid
signature over another bundle is not current within that retained snapshot. Missing files,
aliases, ownership or mode drift, unknown or duplicate verifiers, invalid signatures, expired
authority, generation mismatch, unpinned bundles, replacement, descriptor drift, or byte drift
refuse. No repository path, workspace path, environment variable, workflow input, CLI flag, or
fallback search may change the store paths or trust roots. The initial slice proves snapshot
consistency and intra-invocation drift refusal only; rollback by the independent runner
administrator to an older still-valid verifier-store and bundle pair remains explicitly unproved.

After ordinary execution, crossing, sandbox, contract, selected-graph, command, and secret-
requirement-applicability admission passes, the private authority-snapshot exchange may begin. Only
after Core uses that snapshot to independently re-resolve typed-effect, secret-policy, protected-
binding, profile, subject, and target truth may real execution open one in-memory, invocation-scoped
secret-delivery transaction and register interruption and cleanup authority before the first network
request. Dry-run, Doctor, CI rendering, harness output, and every refused invocation remain provider-
free. The transaction binds the exact contract, selected invocation graph, requirement, effect
decision, secret-delivery decision, protected binding/source bytes, profile, implementation subject,
target, GitHub run claims, Google resources, and numeric Secret Manager version. No prior receipt,
projection, provider response, token, handle, or availability result can satisfy a new transaction.

#### Same-Child Capability Prelude Correction (Proposed 2026-09-13)

The existing signed capability-observation probe remains compatibility evidence only. It creates a
separate refused child and cgroup, so its public projection cannot admit or authorize a later
provider-requesting child. After ordinary structural and secret-requirement-applicability admission,
the normal selected-child boundary must first perform exactly one protected same-child capability-
observation prelude over its inherited startup-bound session. Launcher derives the observation from
that execution-bearing child, cgroup, session, authority stores, and implementation subject; Core
verifies and retains it only as same-invocation correlation truth. It does not admit policy,
candidate derivation, provider contact, or execution. A sibling probe, child, cgroup, session,
request, capability, projection, verifier, or transaction candidate must refuse even when every
substituted outer identity is recomputed.

Only that retained same-child observation may seed the subsequent invocation context and V2
transaction. The V2 binding must reuse and revalidate its exact challenge, request, protected
capability, signed projection, verifier, and installation identities; it must not create a second
observation. Before provider contact, the normal selected-child boundary then derives one closed,
protected same-execution transaction binding. Authority Protocol owns that versioned record's
canonicalization and identity domain. Core independently derives and retains the private Step 1-6
transaction-candidate identity; Launcher receives that value only over the exact selected child's
inherited startup-bound session, binds it as opaque transport truth without treating it as authority,
and returns the private record only over that same session. The record is strictly local transport
and transaction truth: it never enters logs, workflow output, artifacts, receipts, archives, public
JSON, or the public projection.

Protocol must define one closed private `ProtectedSameChildCapabilityPreludeV1` carrier. It contains
exactly `schema_version: 1`, record kind `protected_same_child_capability_prelude`, its
domain-separated identity, same-child observation request identity, public projection identity,
protected capability identity, verifier identity, installation-evidence identity, accepted Launcher
request identity, startup-continuation identity, inherited-session identity, and expiry. Its identity
is SHA-256 over the exact JCS record excluding only `identity`, under
`ota.protected-same-child-capability-prelude.v1\0`; unknown, missing, duplicate, noncanonical, or
substituted fields refuse. Launcher derives it only after the signed observation response and sends
two ordered frames over that exact inherited session: first the unchanged
`ProtectedLauncherCapabilityObservationResponseV1`, then the private prelude carrier. Core accepts
only that exact order, validates the closed carrier shape and identity, and retains it as opaque
protected correlation truth. Core does not treat its capability or descriptor fields as filesystem
authority. The carrier, raw capability identity, and private reconciliation link are prohibited from
the observation response, logs, workflow output, artifacts, receipts, archives, public JSON, and
public projection. V2 request, binding, and response each bind the exact prelude identity; Core
compares their retained private fields with that carrier, rather than widening the public observation
response.

Core retains that verified same-execution binding, its expected fresh challenge and request, the
signed projection, and the reconciled verifier-record and installation-evidence identities in a
crate-private one-use guard. Immediately before constructing the first network request, the guard
must reconcile the returned candidate to its independently retained candidate, startup
continuation, and inherited session, then recheck transaction, snapshot, and V2-binding equality and
freshness and consume itself exactly once. Launcher owns the final retained-descriptor and exact-byte
revalidation immediately before returning the V2 binding; Core owns reconciliation of the resulting
snapshot and V2 identities and never treats copied bytes or descriptor claims as filesystem
authority. Duplicate use, delay past expiry, missing or substituted installation authority, or any
mismatch refuses before OIDC environment access, network-client construction, or provider contact.
Any observation obtained before ordinary structural and applicability admission, or outside the
selected child's inherited session, is invalid for this purpose. The same-child prelude is
correlation truth only until the subsequent snapshot reconstruction admits the canonical Step 1-6
candidate and policy decision.
Empty selections and pre-snapshot structural or applicability refusals open neither the Launcher
snapshot route nor its reservation state. A semantic or policy refusal found after snapshot transfer
must mark that reserved snapshot terminally refused, return no transaction binding, open no provider
transaction, and complete the existing child and cgroup cleanup; it cannot restore the snapshot to an
unused state or make it eligible for retry.

#### Protected Authority Snapshot Bridge Activation Amendment (Independently Reviewed 2026-09-11)

The same-execution transaction binding described above requires one same-child capability-observation
prelude followed by one private authority-snapshot exchange. This is necessary because Core cannot
derive the canonical Step 1-6 transaction candidate until it has independently reconstructed the
protected binding, source, profile, target, and policy truth retained by Launcher. The public
root-socket capability-observation projection is compatibility evidence only and must never satisfy
this exchange or become authority for candidate derivation.

Protocol's structural record layer was committed at `d1d1fd4` before this activation amendment.
That record-only prerequisite did not activate the bridge: Core and Launcher did not consume snapshot
records, no snapshot reservation or V2 exchange existed at runtime, and no provider behavior was
enabled. This independently reviewed activation amendment authorizes the provider-free bridge only
upon its commit; V1 cannot become a fallback for a snapshot-backed transaction.

Authority Protocol must add closed, versioned snapshot challenge, request, payload, and response
records without changing the committed V1 transaction-binding records. The private
`ProtectedAuthoritySnapshotChallengeV1` contains exactly `schema_version: 1`, message kind
`protected_authority_snapshot_challenge`, its identity, a commitment to one Core-generated 256-bit
CSPRNG nonce, and canonical issued and expiry Unix seconds no more than five minutes apart. Its
nonce commitment is SHA-256 over the exact 32 raw bytes under
`ota.protected-authority-snapshot-nonce.v1\0`; Core obtains issuance time from its own clock and
Launcher independently checks issuance, expiry, and current freshness against its own clock. Its
identity is SHA-256 over the exact JCS record excluding only `identity`, under
`ota.protected-authority-snapshot-challenge.v1\0`. It is distinct from the public capability-
observation challenge. Core retains the raw 32-byte nonce; the private request carries it only so
Launcher can independently decode it, rederive the commitment and challenge identity, and prove
freshness. The raw nonce must never enter the response, snapshot payload, V2 binding, logs, workflow
output, artifacts, receipts, archives, public JSON, or a public projection.

The snapshot request contains exactly `schema_version: 1`, message kind
`protected_authority_snapshot_request`, its identity, the complete challenge, canonical unpadded
base64url raw nonce, accepted Launcher invocation-request identity, startup-continuation identity,
inherited launcher-session binding identity, contract identity, and selected execution-graph
identity. Its identity is SHA-256 over the exact JCS record excluding only `identity`, under
`ota.protected-authority-snapshot-request.v1\0`.

`ProtectedAuthoritySnapshotPayloadV1` is the one closed unsigned snapshot payload. It contains
exactly `schema_version: 1`, record kind `protected_authority_snapshot`, the request identity; the
accepted Launcher request, startup continuation, inherited session, contract, and selected-graph
identities; the two fixed store roles; each retained descriptor's exact device, inode, owner, group,
mode, size, and content identity; verifier-store identity, authority identity, active binding-bundle
identity and generation, binding-bundle identity and generation, signed-payload identity, authority
and bundle validity bounds; and the exact bounded verifier-store and binding-bundle bytes required
for Core reconstruction. `ProtectedAuthoritySnapshotIdentityV1` is SHA-256 over precisely that JCS
payload under `ota.protected-authority-snapshot.v1\0`. The response contains exactly
`schema_version: 1`, message kind `protected_authority_snapshot_response`, request identity, that
payload, its independently rederived protected snapshot identity, and its own identity derived over
the complete response excluding only `identity` under
`ota.protected-authority-snapshot-response.v1\0`. Protocol reconciliation must reconstruct the
challenge, request, payload, snapshot, and response identities rather than accepting any supplied
identity as authority.

The response is protected local transport truth and may contain private provider-reference material.
It must never enter logs, workflow output, artifacts, receipts, archives, public JSON, or the public
capability projection.

Launcher opens the fixed stores before the selected child starts and retains the live descriptors,
exact bytes, and verified bundle state for that child. It accepts exactly one same-child capability-
observation request followed by exactly one snapshot request over that child's inherited startup-
bound session, after ordinary applicability is established and before candidate derivation. It
validates and atomically reserves the observation challenge before protected derivation, returns one
signed public response plus the private prelude carrier, then marks that public challenge consumed.
It retains the protected derivation only in one protected child/session prelude slot, available for
one V2 response; a second observation or V2 attempt refuses. It recomputes all subsequent request
identities, revalidates the same retained descriptors and bytes, verifies bundle signature, active
selection, generation, and freshness, reserves the snapshot for that child and session, and returns
exactly one response. Missing, duplicate, stale, replayed, out-of-order, cross-child, cross-session,
cross-startup, path-substituted, descriptor-replaced, metadata-drifted, byte-drifted, signature-
invalid, inactive, or second-observation requests refuse. No caller, repository, workflow,
environment, CLI, or public projection may supply or replace either record. Launcher atomically
reserves the snapshot challenge and request before returning the snapshot response. After constructing
one valid V2 binding response from that same retained observation, one atomic terminal transition
consumes both the prelude slot and the exact snapshot reservation. A persistence failure may leave
either record terminally consumed or refused, but neither may remain reusable or ambiguously
reserved. Every failure after either reservation is terminal and cannot restore the challenge, slot,
snapshot, or eligibility; ambiguity cannot permit a second response.

Core generates and retains the request, accepts one response only over the same inherited session,
reconciles every closed Protocol field, parses the opaque payload through a closed Core-owned schema,
and independently re-resolves the selected requirement, binding/source, provider profile and subject,
effect, policy decision, evaluation, and dry-run plan before deriving the canonical Step 1-6
transaction candidate. The transferred payload is untrusted semantic input until that complete
reconstruction succeeds. A structurally valid bundle, response, or recomputed identity cannot bypass
Core's contract, selected-graph, policy, target, or canonical provider-tuple reconciliation.

Core must first retain one closed, versioned `ProtectedSecretDeliveryInvocationContextV1` from that
same-child observation response and private prelude carrier for the command invocation. It contains the exact workflow run
identifier, run attempt, and workflow reference, each rederived from the reconciled observation
challenge, plus the accepted Launcher invocation-request, startup-continuation, inherited-session,
contract, and selected-graph identities that bind it to this child. Its identity is domain-separated
from the snapshot and candidate. This is a bounded same-invocation correlation record, not evidence
of any GitHub provider claim. The remaining GitHub claim values needed by the configured profile are
retained only as closed, unverified expectations derived during Core's successful reconstruction of
the protected binding/profile snapshot; they are bound to the snapshot and context but cannot assert
that a GitHub token carries them. The bundle, snapshot, projection, environment, workflow, and
caller cannot supply, select, or override the challenge-backed correlation values or any expectation
outside that reconstructed protected authority. Core may derive the provider-free candidate and V2
binding from these expectations, but must require the V2 prelude identity and its observation,
projection, capability, verifier, and installation identities to equal the retained prelude before
accepting it. A separately reviewed provider-contact slice must independently verify every expected value against the signed
GitHub OIDC token before Google exchange or delivery. A missing, additional, or disagreeing token
claim must refuse. A syntactically valid claim set is never sufficient.

The canonical effect-policy finalizer gains one explicit `verified_protected_snapshot` source
posture. It is produced only after the complete snapshot checks below and carries the snapshot and
verifier evidence identities; it is neither repository-controlled, workspace-controlled, nor
caller-selected. This extends the existing finalizer and its precedence/identity model. A
secret-specific parallel policy evaluator is prohibited.

Core first retains the opaque exact binding-bundle bytes and reconciles their raw-byte identity with
the snapshot response without semantically parsing them. It then selects the one active verifier
from the separately retained outer verifier-store record and verifies the binding-bundle Ed25519
signature over those exact bytes under that verifier's exact key usage and signature domain. Only
after that signature succeeds may Core parse the bytes through its closed, versioned,
duplicate-refusing payload schema, require an exact JCS round trip, and require the payload's
authority, trust-root, and verifier identities to derive from the admitted outer records rather
than payload fields. Core passes the complete unfiltered binding collection to
`resolve_secret_provider_bindings`, preserving its existing duplicate and conflict refusal. Any
byte, signature, outer-authority, verifier, trust, source, policy, or payload-shape ambiguity
refuses before candidate derivation.

Authority Protocol must also add an additive V2 transaction-binding request and response. Both bind
the exact protected snapshot identity and `ProtectedSameChildCapabilityPreludeV1` identity in addition
to the existing candidate, accepted Launcher request, startup continuation, inherited session, and
the exact retained prelude capability, challenge, projection, verifier, installation, and expiry
truth. Launcher must revalidate the same retained descriptors and exact bytes immediately before
constructing the V2 binding from that observation; Core's one-use guard must require equality with
its retained prelude, snapshot response, and candidate. Protocol cross-record reconciliation accepts
those retained records as inputs, independently reconstructs the snapshot and prelude identities,
and requires the V2 request, binding, and response to select those exact values. A later separately
reviewed provider-contact slice may consume both exactly once immediately before construction of the
first provider request. V1 remains immutable and may not be accepted in this selected-child lane as
a snapshot-backed fallback.

The selected-child private state machine therefore permits only ordinary completion or this exact
sequence: same-child capability-observation request, signed observation response, Core observation
verification and invocation-context retention, snapshot request, snapshot response, candidate
derivation, V2 binding request using that same observation, V2 binding response, and terminal
completion with provider contact still refused. Inserting a provider transaction between the V2
response and completion requires a separately reviewed and committed Step 7 provider-contact slice.
A sibling or second observation, a V1 binding request in this lane, a snapshot or binding request out
of order, multiple snapshots, a second candidate, use after refusal or expiry, or completion
interleaved with an unfinished exchange refuses and triggers the existing terminal child and cgroup
cleanup. Dry-run, Doctor, CI projection, harness, sandbox, empty selection, and every prelude or
pre-snapshot structural or applicability refusal retain the existing provider-free behavior and open
neither observation, snapshot, nor binding state. A semantic or policy refusal found after snapshot
transfer follows the terminally refused reservation behavior above and cannot advance to V2 binding.

This activation amendment and the 2026-09-13 same-child correction authorize only the private
same-child observation prelude and carrier, their protected replay lifecycle, the private snapshot
bridge, additive V2 binding reconciliation, and their provider-free regressions and protected
Linux/X64 pressure. They do not authorize OIDC access, network-client construction, provider contact,
materialization, injection, positive evidence, Step 8, or V12.2. Implementation may begin only
after the correction is independently reviewed and committed.

#### Step 7 Provider-Contact Slice Amendment (Activated 2026-09-15)

The provider-free prerequisite is now committed, independently reviewed, and exercised through the
exact protected Linux/X64 service path. Protocol
`2c46cb676ef6e0844312bd6a157adec7ccb54de1`, Launcher
`a00f0e1ebd0ba886015a1aa32216ad30b56dcd9a`, and Core
`c071fed09ffe42d4ac00979d16dff0967ad208aa` bind one selected child's signed capability
observation, protected authority snapshot, reconstructed Step 1-6 candidate, and one-use V2
transaction on the inherited startup session. Core run
[34933139633](https://github.com/ota-run/ota/actions/runs/34933139633), protected job
`104265387910`, passed that provider-free path and refused before provider contact or task start.
The amendment was independently reviewed and committed at
`bcc4090bb038211568319749fb6e08c6b60470f4`. It authorizes only the ordered implementation gates
below; it does not by itself authorize a network request or provider claim.

The first provider target remains the existing public-safe profile class: GitHub Actions OIDC to
Google Workload Identity Federation, one service-account access token, one exact numeric Secret
Manager version, and process-environment delivery to the selected transient native Linux/X64
recipient process tree. The administrator-controlled pressure installation owns one non-production
Google project, one pool/provider, one service account, and one synthetic canary secret version.
Their concrete names, resource paths, binding/source identities, and policy identities remain in
protected installation records and may not enter repository content, workflow inputs, CLI
arguments, logs, public projections, artifacts, receipts, or archives.

The GitHub workload provider must independently restrict the stable public claim boundary to the
exact numeric Ota repository and owner identities, the dedicated provider-pressure workflow,
`workflow_dispatch`, and a self-hosted runner. Its administrator-owned condition must additionally
require `workflow_sha` to equal the exact independently reviewed Core revision containing that
workflow. The branch and workflow path are additional routing constraints, not immutable
implementation identity. Every new pressure revision requires an out-of-band administrator update
to that exact condition before dispatch; repository code and the workflow cannot update or satisfy
it by selecting a branch, path, or caller value. Google claim mapping may expose exact run, attempt,
commit, workflow-commit, actor, and subject values as provider attributes, but Ota must not claim
those dynamic values are Google-enforced unless the installed provider condition actually requires
them. Ota's protected invocation tuple must independently bind and reconcile all of those exact
dynamic claims before any Google request. Repository YAML, caller input, inherited generic
environment, or a provider response cannot widen either authority source.

The provider transaction may begin only after Core has reconstructed the Step 1-6 candidate,
reconciled the same-child prelude and signed protected authority-snapshot V2, received the exact
additive transaction-binding V4 response with its transport-dependency record identity, and
atomically consumed its one-use guard. The earlier transaction-binding V2/V3 proof and wire
records remain immutable historical evidence; neither binding version satisfies this
provider-contact gate. Consumption occurs immediately before Ota reads the two
GitHub OIDC capability values or constructs the first network client. Failure, cancellation,
timeout, or interruption after consumption cannot retry, replay, or transfer the transaction to a
second child, session, snapshot, candidate, endpoint, or run.

The transaction order is closed:

1. Validate the retained GitHub token-request endpoint profile, request one JWT for the exact WIF
   audience, and accept no redirect, caller transport, proxy, custom CA, alternate origin, or
   credential forwarding.
2. Parse the JWT through one closed structural path and require its exact issuer, audience, subject,
   repository, owner, workflow, ref, event, runner environment, actor, run, attempt, commit, and
   workflow-commit claims before any Google contact. Missing, duplicate, malformed, stale, or
   substituted claims refuse. Google STS acceptance is the initial slice's signature and issuer
   acknowledgement; Ota must not represent its pre-exchange claim parsing as independent GitHub
   signing-key verification.
3. Exchange that JWT with the exact configured Google STS audience, grant type, subject-token type,
   and requested token type. Reconcile the response type, bounded expiry, and required fields; do
   not accept a refresh token or caller-selected scope.
4. Use only the resulting federated token to request one access token for the exact admitted service
   account and closed cloud-platform scope; separately installed IAM must limit that account to
   Secret Manager access on the admitted secret. No service-account key, ambient Google credential,
   metadata-server credential, local ADC file, or inherited bearer may satisfy this step.
5. Access only the admitted numeric Secret Manager version. `latest`, aliases, alternate projects,
   pools, providers, service accounts, secrets, versions, endpoints, or response resource names
   refuse even when the returned bytes have the expected shape.
6. Retain the secret bytes only in bounded process memory, reconcile them through the protected
   pressure-canary verifier defined below, construct one recipient-only environment entry from that
   same verified buffer, start the exact selected structured command in the already-owned process
   tree, and prove through a fixed pressure helper that the environment entry was present and usable
   without emitting its value, hash, length, prefix, transformation, environment entry, or provider
   reference. Parents,
   siblings, setup, hydration, services, proof helpers, durable logs, raw shell, and processes
   outside the selected recipient tree are not recipients. Arbitrary descendants inside that tree
   may inherit the value and remain explicitly unproved rather than represented as excluded.

Provider response acknowledgement is not delivery proof. Before child start, Core must reconcile
the exact resource version and provider response to the protected invocation tuple. Exact synthetic
canary truth is owned only by one pressure-only
`ProtectedSecretDeliveryPressureCanaryVerifierV1` at a fixed root-owned path and singular protected
installation-manifest role. Its closed record contains exactly its schema version and kind, its
domain-separated identity, the exact requirement, binding, provider-profile, secret-resource, and
numeric-version identities, one administrator-generated 256-bit HMAC-SHA256 verifier key, and one
32-byte expected tag. The key and tag use canonical unpadded base64url and remain protected
secret-derived correlation material. The record identity is SHA-256 over its exact JCS payload,
excluding only the identity itself, under
`ota.secret-delivery-pressure.canary-verifier.v1\0`.

Core loads that record only through a descriptor-retained, no-follow, immediately reobserved
protected installation path, reconciles every bound identity with the admitted transaction, and
compares the fetched bytes' HMAC tag in constant time after provider CRC32C validation but before
environment construction. Missing, duplicate, malformed, path-substituted, metadata-drifted,
byte-drifted, identity-mismatched, tuple-substituted, key/tag-substituted, or unequal canary truth
refuses. The same verified byte buffer must move into environment construction without
re-resolution; test-only mutation between verification and construction must refuse. This record
contains no secret value, grants no provider authority, exists only under the non-default pressure
feature, and cannot satisfy an ordinary production requirement.

The selected helper owns no expected secret truth. It is one fixed administrator-installed pressure
executable whose exact artifact identity and selected graph role are reconciled before provider
contact; repository or workflow content cannot replace it. Its static success marker proves only
that the already verified environment entry was present and usable in that recipient; the protected
HMAC check and same-buffer continuity establish the exact synthetic value. A missing marker, extra
marker, child failure, non-recipient observation, or any stdout/stderr/artifact leak refuses the
pressure result. The canary verifier key/tag, secret value, OIDC JWT, federated token, and
service-account token must never enter public output or be serialized by the transaction to disk or
diagnostic output. Buffers are released and overwritten where the language/runtime permits, but
complete memory erasure remains unproved.

Interruption and fault pressure must cover refusal before each network stage; malformed and
substituted responses; expiry; replay; duplicate response; provider denial; partial read;
materialization failure; pre-start cancellation; child-start failure; child failure; timeout; and
termination during execution. Every terminal path must reap the selected child, remove its scope and
active slot, leave the cgroup empty or absent, close retained descriptors, and prevent reuse of the
consumed V4 transaction. Provider-issued bearer expiry and external audit logs remain provider-owned;
Ota must not claim immediate token revocation or deletion of provider-side audit state.

The first pressure workflow is manual and dedicated, with `contents: read` and `id-token: write`, no
repository secret, no service-account key, and no release or merge authority. Its public output may
retain only closed non-secret phase postures and approved public identities. Step 8 receipts,
protected provider attachments, archives, public positive assurance, adapter registration, support,
other providers, GitHub-hosted runners, local execution, containers, macOS, Windows, raw shell,
persistent runtimes, arbitrary descendants, privileged escape prevention, exfiltration prevention,
and complete memory erasure remain unauthorized or unproved.

Implementation may begin only after this amendment is independently reviewed and committed. The
first implementation batch must stop at protected authority loading and provider-client request and
response models with network contact disabled. Network enablement, live pressure, materialization,
and injection advance as separately reviewed commits so no structural test can silently authorize a
real provider call.

The following transport-preparation checkpoint records its completed, network-disabled V2
boundary. The provider-free V3 compatibility gate was subsequently superseded for future
dispatch by the signed authority-snapshot V2 and transaction-binding V4 route; the V2
preparation checkpoint cannot itself enable a provider request.

The first network-disabled model was independently reviewed and committed at
`77329526e6ec07449c889bac6c3172294620ec4e`. Before any request can be sent, a second
authority-bound transport-preparation checkpoint must be independently reviewed and committed. It
may add only:

- one opaque Core-owned consumed-provider-transaction capability that can be constructed only by
  re-verifying and atomically consuming the exact same-child snapshot-bound V2 transaction in this
  historical network-disabled checkpoint;
- one private capability-input owner that reads the exact GitHub OIDC request URL and bearer only
  after that consumption and rejects missing, empty, or non-UTF-8 input. Both values remain
  untrusted runner-supplied capabilities rather than independent identity evidence. Core cannot
  locally authenticate the initially supplied bearer; GitHub's request service owns that
  acknowledgement. Opaque ownership must prevent post-acquisition replacement, cross-wiring, or
  transfer to another request, transaction, or origin, and must retain neither value in
  diagnostics, arguments, public output, or durable state;
- reconciliation of the retained endpoint observation and exact protected operation plan before
  the bearer can be attached to the in-memory OIDC request model; and
- one fresh synchronous Rust transport configuration using direct dependency `ureq = =3.4.2` with
  default features disabled and only its Rustls and WebPKI-root features enabled, plus the complete
  transitive graph pinned by `Cargo.lock`; effective proxy configuration is explicitly reset to
  `None`, HTTPS is required, redirects and redirect credential forwarding are disabled, cookies and
  content decompression are absent, caller trust roots and client certificates are unavailable,
  DNS/connect/send/read and overall timeouts are bounded, response headers and bodies are bounded,
  and no caller configuration is reusable.

`ureq`'s public configuration builder may inspect ambient proxy variables while constructing its
default value before Ota overrides the proxy to `None`. Ota therefore claims and tests that ambient
proxy values are neither retained nor used, not that the dependency never reads them. The final
configuration must expose `proxy == None`; no request may be built or sent before that state is
verified.

That checkpoint must expose no `send`, `call`, socket, connector, or provider-response path. Its
tests must poison proxy, custom-CA, netrc, client-certificate, and generic HTTP configuration
inputs; substitute candidate, V2 binding, endpoint
observation, URL, post-acquisition bearer owner,
audience, target, and realization/invocation identities; and prove refusal or unchanged fixed
transport posture. The opaque capability
must not be cloneable, serializable, printable, reusable after consumption, or constructible from a
plain binding record. Existing pure byte-model helpers remain non-authoritative implementation
details and cannot become the transport admission boundary.

The authority-bound transport-preparation checkpoint was independently reviewed and committed at
Core `8350b479`. It adds only the opaque consumed-V2 capability, post-consumption one-use OIDC input
owner, endpoint and operation reconciliation, and fixed network-disabled `ureq = 3.4.2` posture
authorized above. It exposes no request send/call, socket, connector, provider response,
materialization, delivery, execution, or public-output route.

The future dispatch gate must consume the signed authority-snapshot V2 and transaction-binding
V4 capability, including its transport-dependency record identity, before it can replace this
network-disabled V2 transport-preparation boundary. The historical V3 binding is not a fallback.

### Complete Transport-Dependency Snapshot Amendment

The complete administrator-expected transport-dependency graph cannot be retained through the
released V1 authority-snapshot response: V1 repeats the signed binding bundle as both typed fields
and base64url store bytes, while its one-frame 64 KiB boundary is immutable. This amendment was
independently reviewed and committed at Core `93a36b23`. It authorizes only an additive V2
authority-snapshot exchange and its additive V4 transaction binding; it does not authorize an OIDC
request, a socket, a provider operation, materialization, delivery, execution, evidence, Step 8,
or V12.2.

The V2 exchange must add closed
`ProtectedAuthoritySnapshotRequestV2`, `ProtectedAuthoritySnapshotPayloadV2`, and
`ProtectedAuthoritySnapshotResponseV2` records with distinct fixed message kinds and
domain-separated request, payload, and response identities. The V2 request retains the exact
`launcher_request_identity`, `startup_continuation_identity`, `session_identity`,
`contract_identity`, selected-execution-graph identity, and fresh challenge context from V1. The
V2 payload retains exactly its `request_identity` plus those same five context identities; each
must equal the retained V2 request and startup continuation. It must not accept a V1 request or
response as a V2 substitute, and V1's records, identity domains, store layout, and 64 KiB frame
limit remain immutable.

V2 snapshot truth must not be threaded through the released V3 transaction-binding records: their
reconciliation APIs and historical semantics are V1-snapshot-specific, while their wire payload has
no snapshot-version discriminator. The amendment must therefore add closed
`ProtectedLauncherSecretDeliveryTransactionBindingRequestV4`,
`ProtectedLauncherSecretDeliveryTransactionBindingV4`, and
`ProtectedLauncherSecretDeliveryTransactionBindingResponseV4` records with new message kinds and
domain-separated V4 request/binding identities. V4 retains V3's exact same-child, capability,
candidate, projection, verifier, installation, expiry, and transport-dependency fields. Its request,
binding, and response each carry exact `protected_snapshot_schema_version: 2` plus fixed
`protected_snapshot_record_kind: protected_authority_snapshot_v2`; all three values must equal
each other and the reconciled V2 snapshot payload. Each V4 carrier's snapshot identity must
reconcile only through the retained V2 request/response before the existing same-child and
capability checks. V3 remains immutable historical proof and must refuse a V2 snapshot; V4 must
refuse V1 fallback, a different schema or kind, and every cross-version request/binding/response
substitution.

The V2 payload has exactly `request_identity`, `launcher_request_identity`,
`startup_continuation_identity`, `session_identity`, `contract_identity`,
`selected_execution_graph_identity`, one verifier-store descriptor, one binding-store descriptor,
and canonical base64url bytes for each respective store. It deliberately does not serialize
duplicate parsed verifier-store or binding-bundle fields.
Protocol decodes each exact byte sequence once, rejects duplicate JSON keys, and requires the raw
store bytes to equal JCS serialization of its decoded closed record before reconciling descriptors,
the verifier-store record, the binding-bundle record, and their generation relationship. Core alone
verifies the signed bundle and parses its private payload after that response has reconciled.
Descriptor role, size, content identity, device/inode separation, canonical encoding, record
identity, request/session identity, expiry, unknown-field, duplicate-store, malformed-byte, and
typed/raw substitution all refuse.

The committed 40 KiB inner binding-bundle-payload maximum is not itself a V2 transportability
allowance: base64url occurs inside the bundle and again for the V2 store field. V2 therefore derives
the actual canonical response bytes before framing and refuses unless their length is at most
`65,532` bytes, leaving the immutable four-byte prefix within `MAX_FRAME_BYTES = 65,536`. The
complete generated graph fixture, full signed authority payload, verifier store, binding bundle,
and V2 response must be measured together; no fixed inner-payload maximum is treated as proof that
the outer exchange fits.

The signed protected binding-bundle payload must retain the complete expected
`SecretDeliveryTransportDependencyFeatureGraphV1`, complete expected
`SecretDeliveryTransportDependencyRecordV1`, and the same record identity. Before sending V2,
Core independently rederives only its embedded graph and record. After the V2 response has passed
structural, descriptor, and bundle-signature reconciliation, Core compares that retained local
truth with the administrator-signed expectation, implementation subject, invocation binding,
reconstructed candidate, and V4 transaction carrier. The future V4-consumed capability and
prepared transport must retain that exact complete expectation rather than a record identity alone.
Missing, stale, oversized, malformed, replayed, V1-fallback, graph, record, lock-byte, node, edge,
feature, source/checksum, target, descriptor, store, request, candidate, binding, or carrier
substitution refuses before an OIDC input is read or an HTTP client is constructed.

Tests must retain a real generated complete graph and prove its complete canonical V2 response
including its four-byte frame prefix fits the exact one-frame envelope, prove V1 continues to reject
that expanded bundle without reinterpretation, and refuse duplicate-key/non-JCS raw stores plus
self-consistent graph/record substitutions after all relevant nested identities are recomputed.
Launcher and Core must pin one immutable Protocol revision and prove V2 byte/descriptor plus V4
transaction reconciliation through the protected Linux/X64 service path before V4 consumption can
replace the historical V2 transport-preparation model.
The adversarial matrix must also reject every V1/V2/V3/V4 request, payload, binding, and response
substitution, including self-consistent recomputed V4 carriers with a mismatched snapshot
discriminator.

#### Protected Hosted-Evidence Custody Correction (Implemented And Proved 2026-09-22)

The provider-free protected Linux/X64 service-path gate may retain a bounded job-produced evidence
set only through a separately provisioned root capture service. The job-owned staging directory and
its self-generated checksum manifest are not administrator-owned or independently attested evidence.
They are an input to capture only; a completed hosted job remains service-path evidence, but no
local bundle may be described as durable administrator custody until this correction passes.

The root provisioner must derive one exact final source path from the administrator-installed
pressure request's canonical run and attempt. A systemd path unit may trigger only on that exact
non-staging `COMPLETE` path, never a broad job-writable directory or inferred run identifier. The
one-shot capture service must refuse malformed, missing, duplicate, stale, cancelled, aliased,
symlinked, mounted, hard-linked, owner/mode/type-mismatched, oversized, incomplete, unexpected, or
mutating capture attempts. It must distinguish the closed success set from the closed
failure-diagnostic set: a complete failed-job diagnostic may be captured only as `failure`, while
an incomplete or malformed failed input refuses and no failure capture can satisfy the
`success_set` capture class. The public class is custody of a closed job-produced set, not root
validation of the job's assertions. It must retain a bounded retry/failed-state policy so a stale or incomplete trigger cannot
later satisfy a new run.

The root service must reopen every source directory and file descriptor-relatively beneath the fixed
source root with no symlink, magic-link, alias, or mount traversal; verify pre/post metadata while
copying; compute its own digests from the bytes it captures; fsync the files and directories; and
publish with no replacement into a root-only store. It must then emit one root-owned, non-secret,
closed public capture record for that exact run and attempt, binding the administrator-installed
request and installation identities, source revisions, capture class (`success_set` or
`failure_diagnostic_set`), and root-computed bundle
digest. Core may accept only that record after verifying its root-owned path and mode, closed shape,
exact run/attempt and installation/request identities, and one-use/no-clobber posture. The record
is a job-inaccessible root-custodied copy of job-produced evidence. It proves neither independent
provenance of the job's assertions nor immutability against the trusted root administrator.

This correction may add only the pressure-only Launcher capture module, its fixed root systemd
service/path units and provisioned directories, the Core workflow's closed capture wait and public
record reconciliation, the Core protected-service-path runbook, the Launcher independently
administered-pressure runbook, and adversarial/provisioning/hosted tests. It creates no Protocol
wire type, generic archive, OIDC request, network client, provider contact, materialization,
injection, selected-work release, receipt, archive, positive evidence, Step 8, or V12.2 capability.
It may be implemented only after this amendment is independently reviewed and committed; the changed
custody path then requires a fresh exact protected Linux/X64 run. That implementation was independently
reviewed and committed at Core `b9c5d0b65eaca9d15d975e5913ba16135337e6b7` and Launcher
`eee7dc636b0cc3a89d33ce6b1b82d5dfa4c56032`. The fresh exact Core run
[`35748662341`](https://github.com/ota-run/ota/actions/runs/35748662341), attempt `1`, passed on the
protected Linux/X64 runner. Its root capture service retained the closed success set in root-only
state, verified the captured checksum manifest, and published the run-bound public record
`sha256:ed085656fe9c16254429bfaffb971ee13fe2df6f2c75608b9c09e93896f8b64f` with root-computed bundle
digest `sha256:f334e62c4b863696e2f857aea840e216d2f037a61f0ce64fef9c50f3d6d8bfd4`. This proves custody of
job-produced bytes and their binding after capture, not independent semantic validation of the job
or any provider, materialization, injection, selected-work, positive-evidence, Step 8, or V12.2
claim.

The later public-root path correction required its own paired-revision proof. Core
[`36271452585`](https://github.com/ota-run/ota/actions/runs/36271452585), attempt `1`, at
`f0668e9e6c3eae89176ba89695186823a22e2a9f` passed on the protected Linux/X64 runner with
Launcher `aa55319fa88f14e96b47e3fa9d08a0940fae5456` and Protocol
`e819f95890ea23ae2f336a59fb3ff62cfa858d8b`. The root-only retained success set passed its
five checksums, and the new root-owned public record at
`/var/lib/ota/authority-launcher-public/hosted-evidence-captures/36271452585-1.json` has identity
`sha256:d4a06e4a9583b090ec18ef52e9b87eb2a79290ada5b71a6c2c1db8a09b925f89` and
root-computed bundle digest `sha256:2cb4b4d2827a4ee1780b93ff4ffdc85867a8c695806de7fecd4e83462ef10a01`.
Independent review found no P1/P2/P3 issue. This closes only provider-free custody for that exact
pairing and attempt, not independent validation of job assertions or any provider or selected-work
claim.

### GitHub OIDC Network-Call Checkpoint

#### Activation Amendment (Active 2026-09-26)

The exact Core `f0668e9e6c3eae89176ba89695186823a22e2a9f`, Launcher
`aa55319fa88f14e96b47e3fa9d08a0940fae5456`, and Protocol
`e819f95890ea23ae2f336a59fb3ff62cfa858d8b` pairing passed the fresh
protected Linux/X64 hosted-evidence custody gate in Core run `36271452585`,
attempt `1`. Its root-owned record and checked bundle establish custody of
bounded job-produced provider-free evidence and a provider-free refusal, not
independent validation of the job assertions or permission to contact a provider.

This amendment was independently reviewed without a P1/P2/P3 finding. Its commit activates only
implementation of the one-shot GitHub Actions OIDC request-service call described below. The active
same-child route for that slice is the complete signed
authority-snapshot **V2** payload and transaction-binding **V4** response, not
the historical V1 snapshot/V3 binding or the network-disabled V2 preparation
checkpoint. Core must reconstruct and independently compare the complete
administrator-signed transport graph and record, consume the exact V2/V4
capability once, and refuse missing, mismatched, substituted, or replayed
carriers before constructing the request. Earlier versions cannot supply
fallback authority for this slice. The earlier V3 descriptions below remain
historical record-format and compatibility requirements; where they describe
the future dispatch route, V2 snapshot/V4 binding supersedes them.

The first live test requires a separately reviewed exact-revision manual
workflow, a fresh stopped-runner request and provisioned state, the protected
Linux/X64 service path, and `id-token: write` only on the bounded job. It may
make one direct request to GitHub's Actions OIDC service, retain the returned
JWT only as an opaque, structurally checked and **unadmitted** private value,
and record only the closed non-secret Core dispatch outcome. No JWT-claim or
signature admission, Google STS/WIF, IAM Credentials, Secret Manager, delivery,
injection, startup release, selected work, positive provider evidence, Step 8,
or V12.2 is authorized. The provider-contact implementation must receive its
own independent review before commit and exact hosted proof; this planning
amendment and the custody run alone do not prove a network call.

Before wiring that workflow or Core dispatch, the GitHub runner's per-job OIDC
request URL and bearer require a protected one-use transport into the selected
Core child. The current Launcher pressure installation constructs the child's
environment from an exact root-owned allowlist; its installation evidence also
publishes that allowlist. The job's `ACTIONS_ID_TOKEN_REQUEST_URL` and
`ACTIONS_ID_TOKEN_REQUEST_TOKEN` therefore do not reach the child, and adding
the bearer to the installed environment would disclose it. A Core-only read of
those variables is not a working hosted route. The dispatch owner and V4
preparation remain unreachable from the protected command until this boundary
is closed.

The implementation gate at activation was an additive, independently reviewed
Protocol/Launcher/Core one-use in-memory relay over the existing authenticated
job-to-Launcher and Launcher-to-exact-same-child channels. The Launcher must
obtain the inputs from its already authenticated job peer only after the
same-child V2/V4 reconciliation; released V4 records and the generic initial
JSON request must not be extended with credential bytes. A new private bounded
message turn needs exact session/binding/nonce correlation, one-use ownership,
and non-secret diagnostics. It must bind the
runner-supplied URL and bearer to the root-owned exact workflow/run/attempt,
verified V2 snapshot and V4 transaction before Core can consume them; a
workflow-controlled value is input, never authority. Missing, duplicate,
substituted, expired, replayed, cross-child, or provider-free-workflow delivery
must refuse; failures before HTTP invocation retain zero Core dispatches,
while failures after it begins retain one and can never authorize a second.
The bearer and full URL must
not enter the root-owned public installation evidence, child environment,
arguments, durable files, logs, diagnostics, public JSON, or receipts; retained
Ota-owned private buffers must be bounded and zeroized. The HTTP implementation necessarily owns
a transient URI only for the bounded dispatch lifetime; Ota must not log, persist, or expose it and
must zeroize every application-owned URL copy on refusal or completion. The exact additive message shape,
ownership and failure cleanup are specified below and require independent
implementation review before commit. The existing provider-free workflow and its refusal are
unchanged. A live workflow dispatch and narrow GitHub OIDC request-service claim required
independent relay and route review followed by exact hosted proof; neither this amendment nor
the first queued-but-cancelled dispatch supplied that evidence. The relay establishes bounded
job-principal supply, not independent GitHub provenance of the URL or bearer.

#### Private Runner-Capability Relay Contract (Implemented; Live Hosted Gate Passed 2026-09-28)

The protected job client is the sole source of the per-job GitHub URL and
bearer. It must read them only after a Launcher challenge on the already
authenticated invocation socket. The challenge may be issued only after the
selected child has independently reconciled the complete signed V2 snapshot,
rechecked its graph and record, and consumed its verified V4 transaction once.
Core retains that consumed authority in a non-cloneable private owner so a
failed relay cannot leave a verified-but-unconsumed retry path; V4 preparation
for the live route must accept that owner rather than consume V4 a second time.
Only that owner can issue a relay request on the existing same-child session.
The request carries the exact launcher request, startup continuation, session,
snapshot, V4 binding, observation request, signed workflow reference, run ID,
run attempt, transaction-candidate, and transport-dependency record identities.
Launcher compares the fields it owns to the retained selected-child session,
root-owned installation, V4 response, and opaque snapshot identity. Core owns
the signed payload, candidate, single selected operation, and transport
semantics comparisons; Launcher does not reinterpret those Core-owned records.
A V1/V2/V3 carrier, provider-free workflow, wrong task, or independently
supplied workflow flag cannot request this turn.

Protocol must add a distinct versioned private message turn; it must not add
credential fields to released snapshot or binding records, the generic
`LauncherInvocationRequestV1`, or the existing JSON completion/output frames.
Launcher mints a fresh nonce only after validating the child request and
irreversibly reserving the one relay turn. Its challenge exposes to the
lower-trust job client only the request identity, exact run/attempt, nonce,
and length limits, not snapshot or operation internals. The client `read_session`
loop accepts this new non-secret framed kind alongside output, finalization,
and terminal frames, and returns exactly one private response. The response
echoes the request identity and nonce; the Launcher-to-child delivery also
binds the retained session and V4 identifiers. Core checks that non-secret
header against its consumed owner before accepting credential bytes. The
challenge uses an exact non-prefetching framed read before Core switches to the
private binary decoder, so a coalesced private frame cannot enter the generic JSON buffer. The
non-secret acknowledgement follows successful private delivery. The job
response and Launcher-to-child delivery use bounded private
binary frames, parsed directly into non-cloneable, non-serializable,
non-printable zeroizing owners, not `serde_json::Value`, ordinary `String`, or
the generic framed-JSON reader. Cap the URL at 4096 bytes, bearer at 8192
bytes, and the complete private frame below 16 KiB; reject zero lengths,
non-UTF-8 URL, malformed bearer bytes, trailing data, overlong lengths, or a
second response observed while the turn remains pending. Do not wait for
stream EOF or claim to detect a duplicate that arrives after dispatch; the
state machine must ensure no later bytes can authorize a second delivery or
dispatch. The URL is independently validated
against the existing fixed GitHub endpoint profile after delivery. No token or
URL digest becomes a public identity or retained artifact.

The additive non-secret Core request is
`ProtectedGithubOidcCapabilityRelayRequestV1` with exactly `schema_version: 1`,
fixed `message_kind`, `identity`, `launcher_request_identity`,
`startup_continuation_identity`, `session_identity`,
`protected_snapshot_identity`, `v4_binding_identity`,
`observation_request_identity`, `transaction_candidate_identity`,
`transport_dependency_record_identity`, `workflow_reference`,
`workflow_run_id`, and `workflow_run_attempt`. Its identity is SHA-256 over
`ota.protected-github-oidc-capability-relay-request.v1\0` followed by JCS of
every field except `identity`. Launcher replies to the authenticated job client
with `ProtectedGithubOidcCapabilityRelayChallengeV1` containing exactly
`schema_version: 1`, fixed `message_kind`, `request_identity`, the exact run ID
and attempt, a fresh 32-byte OS-random nonce encoded as 64 lowercase hex
characters, and fixed maximum URL/bearer lengths. The challenge carries no
snapshot, candidate, binding, bearer, or full URL. Core's non-secret
`ProtectedGithubOidcCapabilityRelayAcknowledgementV1` contains exactly
`schema_version: 1`, fixed `message_kind`, `request_identity`, the same nonce,
`session_identity`, and `v4_binding_identity`; Launcher accepts it only on the
retained same-child session after one private delivery. No message in this
turn signs, grants, or replaces V2/V4 authority.

The private response and delivery each use an independent length-prefixed
binary frame with exactly: 4-byte big-endian payload length; 8 ASCII bytes
`OTAOIDC1`; 32 raw bytes of the request's SHA-256 identity; 32 raw nonce bytes;
2-byte big-endian URL length; 2-byte big-endian bearer length; then exactly
those URL and bearer bytes. The payload length must equal the remaining frame
length and be at most 16,384 bytes; the URL length must be 1–4,096 and bearer
length 1–8,192. There is no padding, extension field, trailing byte, alternate
encoding, or generic JSON fallback. A decoder owns its entire bounded input
buffer from the first read so every incomplete or rejected frame can be
best-effort zeroized; no Ota-owned `encode_frame` copy or generic
`Vec<u8>`/`String` intermediate may retain credential bytes after refusal.
The third-party HTTP request may retain its transient URI only through the one bounded invocation.
The header is checked
before the credential range is exposed to Core. Protocol owns this exact wire
shape and structural validation only; Launcher owns peer/session and replay
state; Core owns signed semantic comparison, endpoint/bearer validation, V4
consumption, and dispatch.

Launcher must challenge the same `SO_PEERCRED`-authenticated job connection
that initiated the selected invocation, never an arbitrary reconnect. Its
challenge write must serialize with concurrent output-frame writes on that
socket; the response read must have a short bounded deadline. The private
response must match the outstanding nonce, exact request identity, and one
selected child/session. Launcher marks the reservation spent before its first
challenge write, then forwards at most once over the already retained
same-child Unix stream. Partial write, timeout, missing acknowledgement, or
other failure cannot reopen or replace the spent turn. Core acknowledges only
after receiving the protected owner and reconciling the request/session/V4
identifiers. Every socket close, timeout, duplicate, wrong-peer, wrong-child,
wrong-version, mismatch, cancellation, or failure to acknowledge invalidates
the turn, zeroizes retained application bytes, and follows existing
child/cgroup/scope/active-slot cleanup. Failure before the Core HTTP invocation
records `core_invocations=0`; failure after it begins retains
`core_invocations=1` with the appropriate closed terminal outcome. The command's terminal
failure output is the non-secret hosted-job record for this internal gate and must carry both
fields on every route; no secret value or transport error detail may substitute. A late
duplicate, if observed, must be refused but cannot authorize another dispatch.
No fallback
to process environment, argv, a durable file, a new unauthenticated socket,
or the historical provider-free route is permitted.

After successful relay, Core must use the consumed V4 authority owner and one
protected input owner for endpoint observation, preparation, and at most one
dispatch; the current environment-reading
`take_github_oidc_bearer_v1` path is not a live-route substitute. Neither the
relay nor a positive acknowledgement admits JWT claims or proves GitHub
provenance, provider-side receipt, or lower-layer request cardinality. The
implementation must test exact same-peer/same-child transfer, output-frame
interleaving, absent/duplicate/oversized/truncated responses, stale nonce,
replay after reconnect, refusal on the old workflow, zeroization on every
error path, and absence of URL/bearer in installed evidence, logs, JSON,
receipts, and retained job artifacts. MUSE's independent read-only design
review found no P1/P2 wire or one-use ownership defect; implementation and
cross-repository pinning require their own review and proof.

The separate manual workflow is staged at
`.github/workflows/secret-delivery-github-oidc-live.yml`. It requires the operator to name the exact
Core revision and independently reconciles that value to the run SHA, installed Core, Launcher
`bafbf1717f102c9d9765c5af382ad06c4ea7eb66`, Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`, root-owned pressure request, workflow, run, attempt,
runner version, branch, and protected Linux/X64 posture before the private relay can begin. The job
has only `id-token: write`, invokes no third-party action, retains no URL, bearer, or JWT, and emits
only the closed non-secret Core invocation count and outcome with explicit `not_proved` provider and
lower-layer cardinality. The first dispatch, run `36350170156` at Core
`4a8549104b3abf24cec9e88deead9d7bad5bad57`, was cancelled while queued with no runner assigned.
Pre-provision review found that Core's pressure builder still admitted only the historical
provider-free workflow reference. The builder correction admits the exact live reference only on
`refs/heads/1.6.29-implementation`, retains the historical path, and refuses other workflow or
branch substitutions. A fresh Core build, run-bound root request, provisioned state, and hosted
run were required after that cancellation. No network or provider behavior was exercised by the
cancelled attempt.

The later bounded hosted slice may contact only GitHub's Actions OIDC request service.
Neither the hosted-evidence custody correction nor this planning amendment activates that route;
the private relay and live route require independent implementation review and exact hosted proof.
This amendment itself implements no network call. It does not authorize JWT
claim admission, Google STS, IAM Credentials, Secret Manager, materialization, injection, release
of the startup continuation, beginning the selected workload/recipient command, positive evidence,
Step 8, or V12.2.

The later exact [live run `36472855700`](https://github.com/ota-run/ota/actions/runs/36472855700),
attempt `1`, job `109099182428`, satisfied that narrow hosted gate on a fresh protected Linux/X64
runner. Root provisioning bound the queued run and attempt before runner start. The job reconciled
Core `8906804faf77b1f2873db9d1f680840bee16e4c5`, Launcher
`bafbf1717f102c9d9765c5af382ad06c4ea7eb66`, and Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`. It reported one Core GitHub OIDC request-service
invocation with `outcome=response_received`, followed by the expected refusal before JWT admission
or selected work. MUSE independently found no P1/P2 blocker for this bounded claim. The retained
public logs and installation records are indexed in [current state](../../ai/current-state.md).
This live job output is not a root-custodied semantic attestation. Provider-side and lower-layer
request cardinality remain `not_proved`; no JWT issuer, signature, or claim admission, Google STS/WIF,
Secret Manager, materialization, injection, selected work, Step 8, or V12.2 follows from this run.
The prospective conditions above remain the historical activation boundary, not a claim that
broader provider authority is now available.

The checkpoint may add only:

- one closed build-owned `SecretDeliveryTransportDependencyFeatureGraphV1`. The reviewed build
  owner derives it from
  `cargo metadata --locked --format-version 1 --filter-platform x86_64-unknown-linux-gnu --features secret-delivery-pressure`
  against the exact source tree and lock bytes, plus the sorted exact output of
  `rustc --print cfg --target x86_64-unknown-linux-gnu` for target-expression evaluation. Its exact
  fields are `schema_version: 1`, fixed
  `kind: secret_delivery_transport_dependency_feature_graph`, `target_triple`, sorted unique
  `target_cfg`, `root_package_name`, `root_package_version`, sorted unique
  `selected_core_features`, fixed `root_dependency_alias: ureq`,
  `root_dependency_node_identity`, canonical `nodes`, canonical directed activated `edges`, and
  `feature_graph_identity`. The root is `ota` at the package version in the reviewed source.
  Traversal begins at the root's one applicable normal dependency-kind entry whose dependency alias
  is `ureq`, then includes the complete applicable non-dev normal/build transitive closure.

  `SecretDeliveryTransportDependencyNodeV1` has exactly `schema_version: 1`, fixed
  `kind: secret_delivery_transport_dependency_node`, package `name`, `version`, `source`,
  `checksum`, sorted unique `enabled_features`, and `node_identity`. It never uses Cargo's opaque
  PackageId as identity input. `source` is the exact Cargo metadata source string or JSON `null`
  when absent; `checksum` is the exact lowercase registry checksum or JSON `null` when absent.
  JCS over every node field except `node_identity`, prefixed by
  `ota.secret-delivery-transport-dependency-node.v1\0`, derives the canonical package key used by
  the graph. Distinct Cargo PackageIds that project to one node tuple are ambiguous and refuse.

  `SecretDeliveryTransportDependencyEdgeV1` has exactly `schema_version: 1`, fixed
  `kind: secret_delivery_transport_dependency_edge`, `from_node_identity`, `to_node_identity`,
  `dependency_alias`, canonical `dependency_kind` equal to `normal` or `build`,
  `target_expression`, and `edge_identity`. Each applicable Cargo `dep_kinds[]` member becomes one
  separate edge; a missing Cargo kind canonicalizes to `normal`. `dev` entries are excluded by the
  named non-dev build graph before traversal and never create nodes or edges unless the same package
  is independently reachable through an applicable normal/build entry. A missing target is JSON
  `null`. A present target must parse with Cargo's platform-expression grammar, evaluate true
  against the exact target triple and retained `target_cfg`, and serialize through that grammar's
  canonical Display form; the exact displayed UTF-8 string becomes `target_expression`. Parse
  failure, non-round-trip display, or a false expression refuses. JCS over every edge field except `edge_identity`,
  prefixed by `ota.secret-delivery-transport-dependency-edge.v1\0`, derives the edge identity.

  Nodes sort by `node_identity`; edges sort by `edge_identity`; every identity must be unique; every
  edge endpoint and `root_dependency_node_identity` must name exactly one retained node. Missing,
  ambiguous, path-substituted, unresolved, duplicate, non-applicable, or dangling graph entries
  refuse rather than being normalized away. JCS over every graph field except
  `feature_graph_identity`, prefixed by
  `ota.secret-delivery-transport-dependency-feature-graph.v1\0`, derives that identity;
- one closed build-owned `SecretDeliveryTransportDependencyRecordV1` with exact fields
  `schema_version: 1`, fixed `kind: secret_delivery_transport_dependencies`,
  `cargo_lock_identity`, `feature_graph_identity`, target triple, root package name and version,
  sorted unique selected Core features, and `record_identity`. SHA-256 over the exact unmodified
  `Cargo.lock` bytes prefixed by `ota.secret-delivery-transport-cargo-lock.v1\0` derives the lock
  identity. JCS over every record field except `record_identity`, prefixed by
  `ota.secret-delivery-transport-dependencies.v1\0`, derives the record identity. The record's
  target, root, selected features, and feature-graph identity must equal the graph rather than
  merely being self-consistent;
- one explicit mandatory `transport_dependency_record_identity` field in the named adapter's
  `AdapterImplementationSubjectInput`, resolved implementation subject, and implementation-subject
  identity payload. The signed protected binding-bundle payload must retain the complete expected
  `SecretDeliveryTransportDependencyFeatureGraphV1`, complete expected
  `SecretDeliveryTransportDependencyRecordV1`, and the same record identity; an identity-only
  protected expectation is insufficient. The same exact identity must be retained through the
  invocation binding/realization, authority snapshot, reconstructed candidate, additive same-child
  V3 compatibility transaction, and V4 transaction. Future dispatch must retain it through the
  consumed V4 provider capability and prepared transport. Before installation, the
  administrator independently derives the graph and record from the exact reviewed source tree,
  lock bytes, target, and feature selection rather than copying the artifact's embedded claim,
  reconciles them with the artifact-embedded graph and record, and retains the expected closed
  records plus identities in administrator installation evidence. Dispatch rederives the embedded
  graph and record and reconciles them against that protected expected record, every retained
  carrier, the implementation subject, source/build/artifact identity, and the prepared
  configuration. Missing, duplicate, malformed, source/checksum, node, edge, package, version,
  feature, lock-byte, target, subject, carrier, or artifact substitution refuses. Repository,
  workflow, environment, caller, CLI, or response values cannot supply or override either side;
- one additive `ProtectedLauncherSecretDeliveryTransactionBindingRequestV3`,
  `ProtectedLauncherSecretDeliveryTransactionBindingV3`, and
  `ProtectedLauncherSecretDeliveryTransactionBindingResponseV3`. Released V2 records, message
  kinds, identity domains, and reconciliation remain immutable. The V3 request has exactly
  `schema_version: 3`, its fixed message kind, `identity`, `launcher_request_identity`,
  `observation`, `secret_transaction_candidate_identity`, `startup_continuation_identity`,
  `session_identity`, `same_child_capability_prelude_identity`, `protected_snapshot_identity`, and
  mandatory `transport_dependency_record_identity`. The V3 binding has exactly
  `schema_version: 3`, its fixed message kind, `identity`, `request_identity`,
  `launcher_request_identity`, `startup_continuation_identity`, `session_identity`,
  `same_child_capability_prelude_identity`, `protected_snapshot_identity`,
  `protected_capability_identity`, `secret_transaction_candidate_identity`,
  `observation_request_identity`, `projection_identity`, `verifier_identity`,
  `installation_evidence_identity`, `expires_at_unix_seconds`, and the same mandatory
  `transport_dependency_record_identity`. The V3 response has exactly `schema_version: 3`, its
  fixed message kind, `request_identity`, `same_child_capability_prelude_identity`,
  `protected_snapshot_identity`, the V3 `binding`, and `projection`. Like V2, the response has no
  independent identity field or identity domain: reconciliation binds its complete closed content
  to the retained request and recomputes the nested binding and projection identities. It does not
  duplicate an independently mutable transport-dependency identity. The new message kinds are
  `protected_launcher_secret_delivery_transaction_binding_request_v3`,
  `protected_launcher_secret_delivery_transaction_binding_v3`, and
  `protected_launcher_secret_delivery_transaction_binding_response_v3`; request and binding
  identities use the new domains
  `ota.protected-launcher-secret-delivery-transaction-binding-request.v3\0` and
  `ota.protected-launcher-secret-delivery-transaction-binding.v3\0`. Request and binding identities
  use exact JCS over every field except their own `identity`, including
  `transport_dependency_record_identity`. Core derives the request field only from its semantically
  reconstructed candidate. Launcher accepts no caller, workflow,
  environment, CLI, or repository override, validates canonical SHA-256 shape, and copies the exact
  request identity into the binding after the existing same-child, snapshot, capability,
  projection, verifier, installation, and expiry checks succeed. Core refuses missing, malformed,
  V2-fallback, request/binding mismatch, signed-bundle mismatch, embedded-record mismatch, or replay
  before transport construction. V3 creates no provider authority and cannot authorize dispatch by
  itself;
- one crate-private, non-default-feature dispatch owner that consumes
  `PreparedSecretDeliveryProviderTransportV4` by value. Historical V1 preparation remains
  network-disabled and cannot serve as fallback. The owner must have no constructor from a plain
  URL, bearer, request, configuration, candidate, or binding record and must expose no reusable
  transport, credential, or response accessor;
- one fresh `ureq = 3.4.2` Agent constructed from the retained verified configuration immediately
  before dispatch. No caller Agent, connector, resolver, proxy, TLS configuration, trust root,
  cookie store, redirect policy, retry policy, middleware, or response reader is accepted;
- one exact `GET` request derived inside that owner from the retained endpoint profile, endpoint
  input, endpoint observation, and protected operation. It preserves only the admitted existing
  query, appends exactly one percent-encoded `audience` equal to the protected WIF provider URL,
  sends exactly one `Authorization: bearer ...` header, and sends no body, cookie, client
  certificate, or additional credential-bearing header; and
- one opaque `RetainedUnadmittedGithubOidcJwtV1` containing only the bounded response JWT bytes.
  Its Debug output is redacted, it is not cloneable, serializable, printable, or durably writable,
  and its owned bytes are best-effort zeroized on drop. The raw bounded response body, parser-owned
  JSON value, and transfer into this owner must each use protected owned buffers that zeroize on
  success and every refusal path; parsing must leave no additional retained String or byte copy.
  Only a separately reviewed later claim-reconciliation checkpoint may consume the final owner;
  and
- one internal `GithubOidcDispatchAttemptStateV1` owned solely by the dispatch function. Its closed
  terminal state records Core dispatch invocations with cardinality `0` or `1` and outcome
  `not_attempted`, `response_received`, `claims_refused`, or `transport_refused`.
  `claims_refused` means a structurally valid response arrived but local matching refused it;
  it is not signature verification or provider admission. Any pressure projection is a
  separately derived non-secret Core-observed posture carrying only that cardinality and outcome;
  it is not provider-attested request evidence and provider-side or lower-layer request cardinality
  remains `not_proved`.

Immediately before constructing the Agent and again before dispatch, Core must re-verify the fixed
transport configuration and exact request against the consumed preparation truth. It must refuse
any changed method, scheme, host, port, path, existing query, duplicate or missing audience,
userinfo, fragment, bearer owner, operation identity, endpoint observation, runner version,
candidate, V4 binding, V4 transport-dependency record identity, transport feature graph, or
lockfile identity. A failure before dispatch
records zero Core dispatch invocations. There is no Core retry, redirect, origin fallback,
connection-pool reuse, or second dispatch. The fresh Agent and request are dropped after that one
terminal invocation. This does not claim independent observation of every lower-layer packet or
provider-side request count.

The response boundary accepts only HTTP `200` and a media type whose essence is
`application/json`, with at most an optional UTF-8 charset parameter. Core reads at most
`MAX_OIDC_RESPONSE_BYTES + 1`, refuses overflow before parsing, and uses the existing closed
`{"value":"..."}` parser and compact-JWT structural check. Non-200 status, redirect, missing or
unsupported content type, malformed or additional JSON fields, empty or oversized body, read
failure, timeout, and structurally invalid JWT all terminally refuse. Response headers, body, JWT,
bearer, URL, and transport errors must not enter arguments, logs, diagnostics, receipts, archives,
public JSON, or retained artifacts. This checkpoint calls the returned token structurally valid and
unadmitted only; it does not claim issuer, audience, claims, signature, freshness, Google
acceptance, or execution authority.

Tests must lock consumption and non-reuse of the prepared transport, exact request construction,
single-dispatch behavior, dependency-record and implementation-subject reconciliation, response
bounds, status and media-type refusal, protected-buffer cleanup, redaction, zero durable output,
and unchanged fixed posture under poisoned proxy, custom-CA, netrc, client-certificate, and generic
HTTP inputs. The exact protected Linux/X64 workflow must run under the signed authority-snapshot
V2 and transaction-binding V4 same-child service path with `id-token: write`. The selected Core
child therefore already exists in its blocked startup/session state; the gate must never release its
startup continuation or begin the selected workload/recipient command. Every pre-dispatch
substitution must retain zero Core
dispatch invocations, while success invokes the dispatch owner exactly once and receives one
successful GitHub response. The public pressure posture may retain only the closed non-secret Core
counter/outcome and must state that provider and lower-layer cardinality are not proved. Every
terminal path must reap the existing child and complete cgroup, scope, and active-slot cleanup. The
workflow must use synthetic authority, no repository secret or service-account key, and preserve
no JWT or runner bearer. A hosted pass proves only the bounded GitHub request-service call on that
exact revision and runner; it does not authorize or prove Google provider contact.

Only after that checkpoint is implemented, independently reviewed, committed, and passes the exact
protected Linux/X64 gate may a separate amendment authorize local JWT claim reconciliation. Google
STS and every later operation remain blocked until that subsequent checkpoint also closes.

The provider client uses a closed transport profile bound into the new profile and implementation-
subject identities. It performs direct TLS with build-pinned public Web PKI roots and exact DNS/TLS
hostname verification; ignores `HTTP_PROXY`, `HTTPS_PROXY`, `ALL_PROXY`, `NO_PROXY`, custom CA,
`SSL_CERT_*`, netrc, client-certificate, and generic HTTP-client configuration; accepts no caller
trust root; rejects IP literals, userinfo, fragments, redirects, alternate origins, and authority
changes; and never forwards credentials across an origin. GitHub's
`ACTIONS_ID_TOKEN_REQUEST_URL` and `ACTIONS_ID_TOKEN_REQUEST_TOKEN` are untrusted capability inputs,
not independent identity evidence. A separate versioned
`GitHubActionsOidcRequestEndpointProfile` binds the narrowly admitted HTTPS request-service host,
port, path grammar, existing query-key grammar, audience-append rule, and transport behavior derived
from retained real self-hosted-runner evidence. It is part of the new profile semantic identity and
must not equate the request-service origin with the JWT issuer. Ota uses the bearer only at the exact
validated request origin and locally reconciles the complete protected claim set of the returned JWT,
including exact issuer `https://token.actions.githubusercontent.com`, before exchange. Google STS
acceptance is the initial slice's provider acknowledgement of JWT signature and configured WIF
issuer acceptance; it is not independent Ota verification of GitHub's signing key. Substituted
ambient values must refuse and cannot change the endpoint grammar or transport configuration.

The only authorized byte-exact provider sequence is:

1. parse the GitHub Actions OIDC request URL and bearer capability supplied to the selected job as
   untrusted inputs, require the URL to match the exact versioned self-hosted request-endpoint
   profile, and serialize neither value into arguments, logs, public output, or durable state;
2. request one JWT with audience equal to the exact protected WIF provider URL;
3. require JWT issuer `https://token.actions.githubusercontent.com` and reconcile the audience and
   complete required claim set to the protected invocation binding before exchange, while treating
   Google acceptance as provider acknowledgement rather than independent proof of every claim;
4. `POST https://sts.googleapis.com/v1/token` as `application/x-www-form-urlencoded` with exactly
   `grant_type=urn:ietf:params:oauth:grant-type:token-exchange`, the protected WIF audience,
   `scope=https://www.googleapis.com/auth/cloud-platform`,
   `requested_token_type=urn:ietf:params:oauth:token-type:access_token`, the reconciled JWT as
   `subject_token`, and `subject_token_type=urn:ietf:params:oauth:token-type:jwt`; send no
   authorization header or `options`, and accept only `Bearer`, the requested issued-token type,
   and a positive `expires_in` no greater than 3600 seconds;
5. `POST` the exact
   `https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{ACCOUNT}:generateAccessToken`
   path as JSON containing only one cloud-platform `scope` and `lifetime: "600s"`, with `delegates`
   omitted; require one nonempty access token and an `expireTime` after the response time and no
   later than 600 seconds after it;
6. `GET` exactly
   `https://secretmanager.googleapis.com/v1/projects/{PROJECT}/secrets/{SECRET}/versions/{NUMERIC_VERSION}:access`
   with an empty body and the service-account bearer token; require the returned resource name to
   equal the requested numeric version, decode only canonical base64 payload data, and verify the
   returned CRC32C before use; and
7. inject the returned bytes only into the exact selected native recipient process-tree environment.

The GitHub request is one `GET` to the exact request-service origin and path admitted by
`GitHubActionsOidcRequestEndpointProfile`, preserving only its admitted existing query keys and
appending exactly one canonically encoded protected audience parameter, with one bearer
authorization header. The request endpoint is not accepted as JWT issuer or identity authority.
Its bounded JSON response must contain exactly one nonempty JWT value, which must reconcile before
Google exchange. The accepted STS federated-token lifetime is at most 3600 seconds; the requested
and accepted IAM service-account-token lifetime and the Ota transaction lifetime are each at most
600 seconds. Ota does not claim provider-side revocation of the longer-lived STS credential after
its owned buffer is dropped. Every request has bounded connect/read/overall timeouts and response
sizes and performs no implicit retry or provider fallback in the initial slice. The transaction
rejects changed method, media type,
field set, response shape, audience, claims, repository/ref/workflow/run identity, WIF pool/provider,
service account, project, secret resource, numeric version, token lifetime, provider origin,
transport posture, launcher capability, or implementation subject. Provider aliases such as
`latest` remain forbidden.

Ota must construct the complete recipient environment without copying a same-named ambient value,
start no recipient until materialization and injection succeed, and start no non-recipient helper,
hook, service, proof observer, negative control, or lifecycle child with the delivered variable.
The selected recipient process tree may inherit the value. The protected launcher must place Ota
and every selected descendant in one fresh cgroup-v2 subtree before provider contact, prevent the
unprivileged process from moving itself or descendants to another cgroup, and retain the cgroup
descriptor and identity in transaction truth. A target without cgroup-v2 kill and populated-state
observation refuses before provider contact. Failure before child start leaves every selected child
unstarted. Failure or interruption after child start invokes cgroup kill, waits for the retained
subtree's populated state to reach zero, and only then terminalizes; timeout or observation failure
is cleanup-uncertain and cannot be reported as cleaned. Processes outside the retained cgroup,
kernel or privileged escape, and exfiltration after receipt remain unproved. Ota must drop and
best-effort zeroize its owned secret and credential buffers and remove every adapter-owned transient
handle, while explicitly not claiming process-memory erasure, provider revocation, rotation,
absence of prior copying, or absence of exfiltration.

The implementation acceptance matrix must prove that the new profile identity differs from the
provider-free v1 profile and that direct Ota invocation, root execution, any Linux capability,
missing `no_new_privs`, substituted launcher/socket/cgroup metadata, and caller-created authority
descriptors refuse before provider contact. Store regressions must cover readable binding files,
writable or aliased parent components, final aliases, mount crossing, descriptor replacement,
byte drift, signature/generation/current-bundle mismatch, and an explicit record that a complete
older administrator-installed snapshot is not detected across invocations. Transport regressions
must poison proxy, CA, netrc, client-certificate, DNS result, OIDC URL, OIDC bearer, and generic HTTP
configuration independently and prove that none redirects credentials or changes trust. Protocol
regressions must substitute every method, origin, media type, field, token type, audience, scope,
lifetime, service account, secret resource, numeric version, response identity, and payload checksum.
Before any provider-contact implementation begins, one real `linux/x86_64` protected self-hosted
runner fixture using the admitted launcher profile must retain the protected launcher capability in
its protected transaction carrier and a separately derived, signed public capability-observation
projection, plus its runner version and non-secret request URL components. Core must generate one
fresh public challenge, verify one exact signed projection against that challenge, and refuse missing,
duplicate, stale, replayed, substituted, malformed, or signature-invalid records before accepting
the endpoint fixture. The matrix must independently substitute the verifier record/key identity,
challenge nonce/commitment, run/attempt/workflow context, expiry, projection signer, signature,
target/profile class, and derivation status; every mutation must refuse. It must prove the endpoint
profile accepts that exact shape, prove the Actions Toolkit calls the supplied request URL rather
than the JWT issuer, and reject changed scheme, host, port, path, query keys, duplicate audience,
userinfo, fragment, and alternate origin. The
provider-free ARM64 discovery run `33919324208` confirms the regional request-service and JWT-issuer
distinction but cannot satisfy this target-specific gate.
Process regressions must include a daemonizing descendant,
interruption at every provider and child
boundary, cgroup-kill failure, populated-state timeout, and terminal refusal unless the retained
cgroup is empty. Every pre-provider refusal must retain zero provider requests and zero selected
children; no test-only injector may exist in default builds.

The implementation must have a non-default pressure feature until Step 8 evidence and Step 10 real-
repository pressure are complete. It may return the ordinary selected-task outcome for local test
control, but it cannot emit a positive secret-delivery receipt, archive, assurance, support claim,
or release-enabled adapter posture. Step 8 and later work remain unauthorized. Connected public
documentation must continue to describe provider delivery as unavailable until a later reviewed
step authorizes and ships the operator surface.

Planned follow-on: [V12.2 Contract-Authored Crossing Requirements](../v12.2/plan.md) remains
inactive and may be activated only after V12.1 completes or is formally deferred. This sequencing
link does not authorize implementation.

### Local GitHub OIDC JWT Claim Reconciliation Checkpoint (Active 2026-09-28)

The exact protected Linux/X64 GitHub request-service checkpoint above is closed, but its JWT was
only structurally valid and unadmitted. This activated Step 7 slice initially authorized only the
local Core implementation and synthetic tests specified below; it did not itself authorize a
hosted gate. The exact hosted reconciliation gate below was separately authorized by the operator
on 2026-09-29. Neither authorization permits dispatch beyond the already proved request-service
call or token use at a provider. The existing V1 profile, binding,
authority records, released V2/V4 wire meanings, and provider-free routes remain immutable.

Core alone would parse the retained JWT payload under its existing bounded, one-use protected
value owner. The payload is untrusted input, even when received from the exact request-service
route: parsing and equality checks do not verify GitHub's signature, issuer, or control of the
request service. The expected values must come from the independently reconstructed, signed
protected authority snapshot and consumed V4 invocation, not the JWT, repository YAML, workflow
input, inherited environment, or a caller-supplied claim map. Core must reconcile the exact
protected audience and subject; issuer must be the literal
`https://token.actions.githubusercontent.com`. It must require exact equality for the existing
eleven bound claims (`repository_id`, `repository_owner_id`, `workflow_ref`, `workflow_sha`,
`ref`, `sha`, `actor_id`, `event_name`, `run_id`, `run_attempt`, and `sub`), plus `repository` and
`repository_owner` derived only from the signed invocation binding's `workflow_ref` claim and
`runner_environment` equal to `self-hosted`. Require that bound `workflow_ref` exactly ends in
`@` followed by the separately signed `ref` claim. Parse the remaining prefix as exactly
`OWNER/REPO/.github/workflows/FILE`, with single nonempty owner, repo, and filename components,
no extra path component or encoded separator, and exact byte equality with the independently
reconciled same-child workflow reference. Compare the token's `repository` to `OWNER/REPO` and
its `repository_owner` to `OWNER`, with no case folding or token-derived fallback; retain the
separate signed numeric repository and owner ID comparisons. The protected workflow-commit value
must still equal the reviewed Core implementation revision; the token cannot supply or revise it.
These three additional checks are additive admission constraints, not a silent rewrite of the
existing eleven-claim V1 profile or its signed identity. A versioned successor is required if
they become part of a signed profile or binding.

One strictly bounded base64url/UTF-8/JSON decode path must reject duplicate JSON members before
mapping, including duplicates of required names; missing, null, array, numeric, or otherwise
non-string identity claims; an audience array or alternate audience; invalid time types; and
malformed, oversized, noncanonical, or substituted token segments. Unused GitHub claims may be
present but confer no authority and must not be copied into retained evidence. `iat`, `nbf`, and
`exp` must be integral NumericDate values; reject arithmetic overflow and require
`nbf <= now < exp`, `iat <= now + 60 seconds`, `iat < exp`, `nbf < exp`,
`exp - iat <= 900 seconds`, and `exp - nbf <= 900 seconds`, all with checked arithmetic.
Do not require `iat <= nbf`: GitHub's documented example places `nbf` before `iat`.
Read a trusted local clock at reconciliation; recheck freshness at any later provider-contact
boundary rather than treating this result as a reusable admission. If the signed workflow
reference cannot supply an exact owner/repository spelling, or the installed GitHub subject
format differs from the bound subject, refuse and review a new versioned binding; do not infer
an expectation from the token or relax matching.

Successful local reconciliation may produce only a private, single-use, still-unadmitted result
for the same child/session/consumed V4 transaction. It must not release startup, start selected
work, call Google STS/WIF or Secret Manager, materialize or inject a secret, emit a positive
receipt, or expose JWT bytes or parsed claims in logs, public JSON, archives, artifacts, or
diagnostics. Every refusal remains terminal and reaps the existing child and cgroup. Neither
this local check nor the prior GitHub response proves signature validity, provider acceptance,
provider-side request cardinality, or execution authority.

GitHub JWT claims bind the expected workflow and run context; they do not encode Ota's local
child/session identity. Distinct-child/session replay protection therefore belongs to the private
relay frame and capability request: the frame must correlate to the signed consumed-V4
request/session identity and one-time nonce. Do not claim that otherwise-valid raw JWT bytes are
intrinsically child-specific or that local claim reconciliation alone refuses their reuse.

After activation and before hosted proof, add focused production-path tests with synthetic JWTs:
one exact protected match; independent substitution of issuer, audience, each bound identity claim,
repository, owner, and runner environment; missing/duplicate/wrong-type fields; malformed or
oversized encoding; stale, future, boundary, overflow, and inconsistently ordered times, including
the documented `nbf < iat < exp` shape; and protected snapshot or V4 mismatch. Separately prove
that a private relay frame/capability from one child/session cannot correlate to a second
independently derived request/session while holding the nonce constant. Lock no JWT disclosure, no
second Core dispatch, no Google request, no selected-work execution, and terminal cleanup. Connected
unit and production-path coverage may establish those local properties, but it is not an
end-to-end replay-to-terminal hosted demonstration. The hosted proof gate then needs its own exact
Core/Launcher/Protocol revisions, fresh administrator-owned request, stopped-runner provisioning,
and protected Linux/X64 run. Retain only a closed non-secret local outcome; a green hosted job must
still label the JWT unadmitted and Google/provider acceptance `not_proved`. Google contact remains
a separate later amendment and proof gate.

An exact protected Linux/X64 reconciliation workflow passed in Core
[run `36548056015`](https://github.com/ota-run/ota/actions/runs/36548056015), attempt `1`,
job `109339134798`, at Core `eeab51e4d473d09073094424750643abbfe2dc11`, Launcher
`bafbf1717f102c9d9765c5af382ad06c4ea7eb66`, Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`, and Runner.Listener `2.337.0`.
The administrator-owned mode-`0400` request bound the queued job before provisioning and runner
start. The job checked the exact public installation, made one Core request-service invocation,
locally matched the structurally valid JWT against its protected expectation, and refused before
admission or selected work. Its job-derived public posture is `outcome=response_received`,
`jwt_claim_reconciliation=matched_unadmitted`, `jwt_admission=not_established`,
`jwt_signature_verification=not_attempted`, `google_contact=not_attempted`, and
`selected_work_executed=false`. The retained logs and two public installation records are in
[`secret-delivery-github-oidc-live-36548056015.zip`](../../pressure/retained-artifacts/secret-delivery-github-oidc-live-36548056015.zip)
(SHA-256 `5bab516260b47ca107bbea7f3c9a8018b5e51a5bde373dfd939f01ce96fba312`).
This is not an independent root-custodied semantic attestation. Provider-side and lower-layer
cardinality remain `not_proved`; no JWT signature, issuer authority, provider acceptance, Google
STS/WIF, Secret Manager, materialization, injection, selected work, Step 8, or V12.2 follows.
Operator-side checks after the run confirmed the runner was stopped and deregistered, temporary
repository access was removed, and the short-lived VM, boot disk, firewall, subnet, and VPC were
deleted; these administrative actions are not established by the retained job archive. The
planned hosted proof gate remains open: the prerequisite private-frame distinct-child/session
replay and selected-failure terminal-cleanup coverage above was not present in this run's exact
revisions. That coverage was subsequently added and independently reviewed with no P1/P2 finding:
Core holds the nonce constant while refusing a frame under a second independently derived
request/session, and Launcher exercises the exact selected-failure cleanup helper against a real
root/systemd child, scope/cgroup, and active slot on Linux/arm64. The Launcher test does not send a
terminal client frame or inject a real relay mismatch, and `/bin/true` may exit immediately after
resume; it proves connected cleanup behavior, not a full end-to-end replay-to-terminal path. This
historical run remains a bounded single-attempt observation. At that point a fresh exact-revision
protected Linux/X64 run was still required before the planned gate could close.

The fresh gate passed in Core
[run `36580942380`](https://github.com/ota-run/ota/actions/runs/36580942380), attempt `1`, job
`109448647893`, at Core `c39e0bc5376fc9c77a45b8f0d4c31a29748d8625`, Launcher
`7d81d93d309fe360968dd48543671d13958adadc`, Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`, and Runner.Listener `2.337.0`. Operator-side
checks recorded that a fresh root-owned mode-`0400` request and provisioned state bound the queued
run before the protected runner started. The retained public records reconcile the request identity,
but do not independently prove the request file's mode or pre-start ordering. Both workflow steps
passed; the job-derived non-secret posture records one Core
request-service invocation, a structurally valid locally matched but unadmitted response, and
refusal before selected work. The public logs and installation records are retained in
[`secret-delivery-github-oidc-live-36580942380.zip`](../../pressure/retained-artifacts/secret-delivery-github-oidc-live-36580942380.zip)
(SHA-256 `0d1240d44872212ac21c7367c6d76684b55c1ce215c419c8329812cf9a729110`).
This closes the exact-revision hosted reconciliation gate, not end-to-end replay-to-terminal
proof, independent root-custodied semantic attestation, JWT signature or issuer authority,
provider-side cardinality, Google acceptance, or any later delivery or selected-work gate.
V12.1 Step 7 remains active; no next provider-contact or admission slice is activated here.

### Google STS Exchange Checkpoint (First Batch Active 2026-09-29)

The hosted GitHub OIDC reconciliation gate above demonstrated one structurally valid, locally
matched but unadmitted runtime JWT; it retained no token for reuse. The next possible Step 7 gate
under the provider-contact amendment is one Google STS token exchange, not Secret Manager delivery.
MUSE independently reviewed this checkpoint and its ownership/test-scope repairs with no remaining
P1/P2/P3 finding. The operator activated only the first Google-network-disabled implementation
batch below on 2026-09-29. Google network enablement and hosted dispatch require the later gates.

The first implementation batch remains Google-network-disabled; the existing GitHub OIDC
request-service call is not a Google connection. Successful GitHub dispatch must move its exact
consumed `PreparedSecretDeliveryProviderTransportV4` and locally reconciled JWT together into one
non-cloneable, one-shot private owner, rather than returning a standalone token that can be rebound
to caller-reconstructed context. Immediately before a prospective STS exchange, that owner must
re-derive and compare the exact operation from its retained candidate against the signed protected
V2/V4 truth, reject an expired consumed binding, and recheck JWT claims against protected
expectations and the `nbf`/`iat`/`exp` window against a fresh trusted clock. The STS audience and
request derive only from that exact rederived operation plan, which is bound to signed V2/V4
authority but is not itself signed. V1, V2, V3, caller-supplied, inherited, or repository values
cannot substitute for V4 authority. Tests must exercise the production one-shot STS request/response
owner through an injected fake Google transport, proving zero or one dispatch, fixed URL and form
fields, no ambient credential or proxy fallback, bounded response parsing, redaction, non-reuse,
and terminal child/cgroup cleanup on every refusal. This batch must not open a Google connection
or produce an admitted token.

The first batch's local ownership/cleanup checkpoint is complete: Core provider tests passed
12/12, snapshot tests passed 22/22, and the network-isolated Linux/arm64 root fixture passed
all 11 STS refusal cases through the actual scoped Core child, retained completion session,
Launcher production relay, durable acknowledgement, observed exit, and terminal cleanup.
MUSE's frozen five-file source review found no remaining P1/P2/P3 issue. This closes only
the synthetic local batch, not installed authority admission, hosted Linux/X64 proof, or
Google networking; the next gate below remains inactive.

Network enablement requires a separate frozen-diff review and explicit authorization. A later
manual protected Linux/X64 proof requires exact pinned Core/Launcher/Protocol revisions, a fresh
administrator-owned request, the stopped-runner sequence, and an out-of-band administrator check
that the installed WIF provider condition binds the reviewed workflow commit. Retain only closed
non-secret status and independently distinguish job observation from provider acknowledgement.
A successful STS response would show Google accepted the submitted JWT under that installed WIF
configuration and returned a federated token; it would not prove independent Ota signature
verification, every claim's provider-side enforcement, provider-side request cardinality, or
authority to run selected work. IAM Credentials, Secret Manager, materialization, injection,
positive delivery evidence, Step 8, and V12.2 remain outside this checkpoint.

#### STS Network-Enablement Preparation (Implementation Active 2026-09-30)

The operator explicitly activated this independently reviewed implementation-only checkpoint on
2026-09-30. Source and local network-disabled tests may implement the boundary below. Google
contact, provider configuration, installation changes, and hosted dispatch remain unauthorized
until a separate frozen-source review and explicit hosted authorization. This activation neither
closes Step 7 nor permits a later provider operation.

MUSE independently reviewed this proposal on 2026-09-30. Its initial P2 identified the missing
administrator-owned live WIF target source; the additive V2 pressure-request boundary below
resolved it. Re-review found no remaining P1/P2/P3 planning finding. This clears planning review
only: implementation is now active, but source/transport review and hosted authorization remain open.

The next implementation batch must reuse the existing retained V4/JWT owner, fixed direct-TLS
configuration, STS request/response parser, and terminal completion path. It must:

- Preserve `secret-delivery-github-oidc-live.yml` as the existing GitHub-only proof route, including
  its zero-Google-contact posture. Use the separately named
  `secret-delivery-google-sts-live.yml` only for the new manual STS proof; no generic provider
  selector or ambient enablement flag is added.
- Derive the STS route solely from the exact invocation context reconciled with the signed
  authority snapshot V2 and consumed transaction binding V4: repository, workflow/ref/SHA,
  run/attempt, selected `task`/`governed` lane, exact selected child, and retained session must
  match. The pressure authority builder admits that new workflow only on
  `refs/heads/1.6.29-implementation`. An ambient workflow string, CLI input, old snapshot,
  identity-only authority, V1-V3 binding, or caller-created JWT cannot enable STS. Admission of
  the new workflow must not widen the historical workflow's provider scope.
- Carry the live WIF target in an additive, closed `PressureAuthorityRequestV2`, used only for
  the new STS workflow. Preserve V1's exact schema, identity domain, fixed synthetic coordinates,
  and existing workflow semantics. V2 uses schema version 2, record kind
  `secret_delivery_sts_pressure_authority_request`, and identity domain
  `ota.secret-delivery-sts-pressure.authority-request.v2\0`; its JCS identity covers every existing
  invocation field plus `sts_target.workload_identity_provider`. Reject unknown fields, mixed
  versions/kinds, legacy workflows with V2, and the STS workflow with V1; there is no V1 fallback.
  This version is the administrator pressure request, not a Protocol authority-payload or
  transaction-binding version change.
- The only new target input is one canonical resource name
  `projects/{positive-decimal-project-number}/locations/global/workloadIdentityPools/{pool}/providers/{provider}`,
  validated by the existing Google tuple grammar with canonical round-trip equality. Derive the
  parent pool, JWT audience (`https://iam.googleapis.com/{provider}`), and STS audience
  (`//iam.googleapis.com/{provider}`) from that same value; accept no separately editable audience,
  endpoint, scope, service-account, or secret target. The existing provisioner's descriptor/owner/
  mode checks must cover V2 as they cover the fresh root-owned mode-0400 V1 request. No repository,
  job, environment, or CLI-supplied target can substitute for that retained administrator input.
  The builder signs the derived provider/pool/JWT audience in authority payload V2 and binds the
  complete request identity into the existing protected installation evidence; Core still derives
  its operation only from the signed snapshot and consumed V4 owner. Unchanged synthetic
  service-account/secret fixture references remain non-executable at this checkpoint and require
  neither creation nor access grants. A generic provider selector is out of scope.
- Consume the successful GitHub terminal's exact private V4/JWT owner directly. Before any STS
  network call, freshly recheck full signed transport graph/record, Core's embedded expectation,
  operation derivation, binding expiry, complete protected JWT claims, and JWT time window.
  A successful GitHub request with failed claim reconciliation must make zero STS calls.
- Build exactly one POST to `https://sts.googleapis.com/v1/token`, with the six form fields
  already specified above, no authorization header or `options`, and no alternate endpoint,
  implicit retry, proxy, ambient credential, custom CA, or provider fallback. Independently check
  the constructed HTTP method, URI, headers, and body against the retained request before sending.
  Reuse the build-pinned Rustls/Web PKI roots, exact hostname verification, and existing fixed
  timeout/header limits. Bound the response read to `MAX_TOKEN_RESPONSE_BYTES + 1` before parsing;
  refuse over-limit, non-200, ambiguous content type, redirect, malformed/duplicate fields,
  invalid token type/lifetime, or transport failure without retaining response details.
- Recheck the retained binding and JWT time window after response receipt and before recording
  acceptance. Drop the federated token in the same terminal owner; do not expose it to callers,
  logs, arguments, environment, files, a next provider operation, or a reusable admission object.
  On every outcome, return failure through the existing retained completion session, reap the
  actual child, and reconcile scope/cgroup/active-slot cleanup. Never release startup continuation
  or execute the selected workload. Buffer cleanup is not process-memory erasure or provider
  revocation proof.
- Return only closed non-secret attempt/outcome values: separate GitHub and STS Core dispatch
  counters, local claim reconciliation, STS accepted/refused/not-attempted outcome, and selected
  work false. A Core dispatch counter does not establish provider or lower-layer cardinality.
  A validated response is job-observed provider acknowledgement under the installed WIF
  configuration, not independent Ota JWT signature verification or proof of every condition's
  provider-side enforcement. The workflow must derive posture from validated Core terminal output,
  not replace a missing or failed response with a synthetic success record.

Required local tests must exercise the production transport request construction and bounded read
through an injected network-disabled transport, together with the snapshot-bound V4 owner and
real terminal completion/cleanup fixture. Cover exact successful-response token disposal, zero
STS dispatch for authority/claim/route substitution, one dispatch with no retry on transport or
response refusal, expiry during response receipt, redaction, unchanged legacy GitHub-only behavior,
and default/no-feature refusal. Test hooks must not become production endpoint or TLS overrides.
Frozen-diff MUSE review and local tests precede commit readiness; neither authorizes a live call.

Before creating hosted infrastructure or a live WIF target, verify that the new manual workflow
is present on the repository's default branch and registered in Actions, with the applicable
exact-source CI/merge gates complete and default-branch registration explicitly authorized.
Branch-only presence is not `workflow_dispatch` admission. Do not merge the whole implementation
batch, change the default branch, substitute a historical workflow, or add a different trigger
to bypass this prerequisite.

Runner-group compatibility is a separate prerequisite. This non-reusable `workflow_dispatch`
workflow must use the exact fully qualified branch selector
`ota-run/ota/.github/workflows/secret-delivery-google-sts-live.yml@refs/heads/1.6.29-implementation`;
GitHub documents branch pins for non-reusable workflows and permits SHA pins for reusable ones.
API persistence of a full-SHA selector does not prove scheduling compatibility. For this bounded
operator-controlled checkpoint, explicitly freeze branch updates and competing dispatches at the
reviewed Core SHA. Independently observe that branch head, exact queued run/attempt/job and absence
of any other eligible queued/in-progress invocation before authority issuance and again before
runner start. On drift or competing assignment, stop/cancel/teardown rather than retargeting the
packet or retrying. These external controls are not cryptographic scheduling proof: branch access
does not enforce immutable workflow code, and workflow guards run only after assignment. Preserve
the exact installed source, signed workflow/commit/run/attempt, full graph/record, offline comparison
and provider-condition checks without substitution. Signed Ota authority does not govern arbitrary
shell execution before those checks. If immutable job-code admission becomes required, separately
design and review a SHA-pinned reusable workflow with explicit caller/callee identity handling;
it is not a transparent replacement or part of this pressure-only scheduling correction.

Before any separately authorized hosted run, pin exact Core/Launcher/Protocol revisions and
installed artifact identities. Out-of-band administration must verify a dedicated WIF provider's
issuer, exact allowed audience, attribute mapping, and condition binding the numeric repository
and owner IDs, exact workflow/ref/commit, event, and selected run/attempt. Retain its non-secret
configuration and distinguish administrator observation from job evidence. After the job queues,
reconcile the administrator-observed provider resource, issuer, allowed audience, mapping, and
condition against the exact V2 request, freshly signed snapshot, and Core-rederived operation
target before runner start; an identity string alone or a check of another provider is insufficient.
Refuse provisioning/start on a target/configuration mismatch rather than patching the audience in
the job. The retained request identity must rederive from the same complete V2 request bytes.
Required local request tests must lock V1 output/identity compatibility, V2 JCS identity sensitivity
to target and invocation changes, wrong-workflow/version refusal, malformed target refusal,
signed target substitution refusal, and exact equality of derived audiences/pool/provider. No
local fixture proves installed remote WIF configuration. This proof requires
no service-account impersonation or Secret Manager grant and must not add either. Use a fresh
root-owned mode-0400 request and fresh provisioned state with the stopped-runner sequence; reruns
need a new request/state. Root request admission is not proof of remote WIF configuration.

Pre-start verification gap (2026-09-30): MUSE held the frozen readiness packet for run
`36725521542`, which was cancelled before activation and whose temporary infrastructure was
removed. Public installation/request identities and canonical builder regeneration do not replace
reading and verifying the actual installed signed V2 payload and using Core's production target
derivation. At that readiness boundary, the internal pressure-helper CLI had no mode that performed
the complete comparison. The smallest source repair is an explicit offline verification mode in that existing
feature-gated helper, preserving default rendering bytes. Reuse canonical protected-store and
outer envelope preconditions, the production V2 signature/JCS/schema/full graph-and-record
verification, and canonical target derivation. If the planner cannot be reached without genuine
candidate evidence, share its pure target derivation with the diagnostic; never fabricate
semantic, snapshot-bound, consumed-V4 or JWT-owner objects. Export only a closed non-secret,
offline/not-admitted/not-dispatched projection and identities. Do not read signing keys, construct
provider transports, make snapshot RPCs, consume replay/reservations, mutate installation/state,
or grant execution authority. Test positive installed-style fixtures and request/provider/
condition/audience, key/signature/schema/JCS, full graph/record, legacy fallback, alias and size
refusals, with unchanged inputs/state and no network on success or failure. Frozen review and
new source gates precede any fresh hosted attempt. This records the missing proof path, not a
waiver of the pre-start bar or a completed implementation.

Local repair checkpoint: the existing helper now has an explicit offline `--verify-installed`
mode with fixed root-controlled request/store/readback/public-installation inputs. It calls the
production Protocol envelope/JCS reconciler and Core signed V2/full graph-and-record verifier,
compares the actual payload with complete regenerated expectation, and shares pure STS target
derivation with runtime planning. Public installation comparison binds the complete request
identity, builder/source and selected environment, including runner version. Provider readback
must be disabled/ACTIVE and exactly match resource, issuer, audience, mapping and run condition.
The fixed recipient-owned contract is bounded subject input, not authority: exact directory/file
modes and common observed ownership do not prove canonical recipient identity. Its one retained
read supports point-in-time comparison only; runtime reconciliation is still required. Closed
output states these limits and no contact, snapshot exchange, consumption, writes or selected work.
MUSE cleared the frozen source review and the test-only recheck with no remaining P1/P2/P3; local
pressure, snapshot, provider-client, CLI, default-feature and isolated Linux/arm64 production-path
checks pass. This neither waives human publication approval or hosted source gates nor closes the
separately authorized fresh Linux/X64 provider attempt.

Fresh installed checkpoint (2026-09-30): exact Core `0476b9a3` passed source gates, including
Release Gate `36745309983`, and its production offline helper passed against a fresh Linux/X64
installation with Launcher `dd667c3d` and Protocol `e5fe1c83`. MUSE cleared the exact packet plus
addendum conditionally; this did not admit execution. Run `36764606243`, attempt `1`, was cancelled
queued with zero steps because final checks left insufficient signed lifetime for the job timeout,
propagation and cleanup. Provider/pool remained disabled and runner stopped; no OIDC/STS occurred.
Temporary resources were removed and their absence/disabled soft-deletion observed. The retained
closed outcome is `docs/pressure/retained-artifacts/secret-delivery-google-sts-readiness-36764606243.json`.
For the next attempt, stage builds and review reusable operational helpers before issuing the new
short-lived request; exact installed verification, independent packet clearance and fresh admission
checks remain mandatory afterward. Do not extend, renew or reuse cancelled authority. This closes
the installed offline comparison checkpoint only, not the positive STS acceptance bar below.

Scheduling checkpoint: fresh run `36775307269`, attempt `1`, passed production offline verification,
independent packet review and immediate admission with adequate lifetime. Operator process checks
were corrected to accept only `pgrep` status `1`, refusing matches and observation errors; the
corrected sources, stub checks and actual host observations were independently rechecked. The
provider was enabled and the stopped runner started, but the healthy online runner remained idle
and the job unassigned. The manual workflow's full-SHA runner-group pin conflicts with GitHub's
documented branch-only pinning for non-reusable workflows. This is a likely scheduling cause, not
a definitive scheduler diagnostic. The run was cancelled with zero steps, its fresh resources
removed, and no OIDC/STS proof obtained. Retention:
`docs/pressure/retained-artifacts/secret-delivery-google-sts-readiness-36775307269.json`.
MUSE's read-only design review supports the branch-specific preparation under the explicit
operator branch/dispatch freeze above, preserving exact signed source/run authority. That review
does not clear another live activation. Do not relax a live reviewed packet or claim branch
admission enforces immutable workflow code. No cancelled request/authority is reusable.

Review-window checkpoint: run `36784346916`, attempt `1`, used the reviewed branch selector and
passed the installed production offline comparison after one canonical authority issue. Preparation,
group binding and pre-issue packet were independently cleared, but final installed-packet activation
review did not return a verdict within the safe window. With 1232 seconds remaining, review, fresh
checks and propagation could not preserve the 1200-second final-admission floor. The run was
cancelled with zero steps before enabling the provider or starting the runner; temporary resources
were removed with administrator-observed absence/disabled soft-deletion. Retention:
`docs/pressure/retained-artifacts/secret-delivery-google-sts-readiness-36784346916.json`.
MUSE's retrospective review found no P1 or additional P2/P3 installed-packet defect and identified
a P2 operator/reviewer sequencing failure: broad post-issue review consumed the usable admission
budget. This is not activation clearance or a demonstrated Core runtime defect. Corrected branch
scheduling and live OIDC/STS remain unexercised.

Bounded admission checklist for independent review before another infrastructure attempt:

1. Complete broad source/helper/refusal/configuration and packet-shape review before issuing.
   Use existing network-disabled fixtures and the canonical verifier, not a parallel validator.
   Freeze exact source pins, helpers, expected graph/build/configuration and this acceptance list.
2. Stage and independently review the exact fresh invocation/provider/readbacks, stopped-runner
   posture and build evidence before signing. Obtain an explicit reviewer-ready handoff; no broad
   review or user-dependent question may remain outstanding at issuance.
3. Issue once. Run the production complete signed-store/JCS/schema/graph/record/request/provider
   comparison with independently observed producer expectations. Final independent review checks
   only the delta against the frozen pre-review: fresh signed identities/window, verifier exit 0
   and closed offline posture, installed digests/ownership/units, exact external tuple and strict
   absence/drift observations. Retain the full packet; compact comparison must not omit any check.
4. Target five minutes for that final review as an operational budget, not an approval guarantee.
   Interruption, drift or a need for wider investigation requires cancellation, not restarting
   review against the same ageing authority. Fresh admission still requires at least 1200 signed
   seconds after propagation and no competing eligible invocation. No automatic retry.
5. If the bounded handoff is still unreliable, hold infrastructure and separately review the
   preparation design. Do not lengthen TTL, override verification time, substitute unsigned
   rendering for installed proof, waive independent clearance or renew/reuse cancelled authority.

MUSE independently cleared this cancellation-retention checkpoint and bounded sequencing design
with no new P1/P2/P3 findings. That verdict is documentation/sequencing readiness only, not live
activation clearance. The checklist does not itself authorize a new VM or activation; it preserves
the existing proof bar and fixed signed window.

Fresh attempt-5 checkpoint (2026-10-01): run `36793610729`, attempt `1`, job `110151889834`,
was assigned to the exact runner `1383`/group `8` at Core
`0476b9a3b484c672fccb164517a1dc3097cd5e66`. Corrected branch scheduling was exercised under
the operator freeze, not immutable workflow-code enforcement. Pre-issue review and explicit
reviewer-ready handoff preceded one issue; MUSE conditionally cleared the signed/installed delta.
Production offline complete graph/record comparison and immediate checks passed; 2409 signed
seconds remained after the 300-second operator propagation hold, above the 1200-second floor.
The first workflow reconciliation failed before the protected client: `cmp` could not read the
execution-owned `/srv/ota-v3-pressure/ota.yaml` (directory 0750, contract 0640). The provider step
was skipped. Provider enablement and scheduling do not establish OIDC/STS execution or acceptance.
Provider/pool were disabled and exact temporary resources removed; teardown returned 0 and
administrator observations confirmed absence/disabled soft-deletion. Terminal observation found
no principal processes, authority scopes, active slots, selected marker or public capture record;
the runner unit was failed/failed with exit 2, not protected-client finalization proof. Retention:
`docs/pressure/retained-artifacts/secret-delivery-google-sts-readiness-36793610729.json`.
This is a workflow principal-boundary mismatch, not a demonstrated Core runtime defect. Review
the comparison and connected job-side marker guards at their actual owner boundary before another
attempt: `test ! -e` cannot distinguish absence from inaccessible execution-owned paths. Do not
relax ownership/modes, grant job access, invent a parallel verifier or silently remove the check.
Any source repair requires independent review and exact source gates; protected workflow edits
require task-specific authorization. No new VM, retry, issue or activation is authorized by this
record. Step 7 stays active/open; later delivery gates remain inactive. No public propagation is
required for this internal pre-client failure; owner-side readiness and cleanup observations are
not independently attested execution evidence.

The hosted acceptance bar is one locally reconciled GitHub response, one accepted STS response,
no selected-work marker, and exact child/cgroup/scope/active-slot cleanup. A refusal is retained
honestly but does not close the positive STS gate. Provider-side cardinality, independently
root-custodied semantic attestation, IAM Credentials, Secret Manager, materialization, injection,
positive delivery evidence, Step 8, and V12.2 remain outside this amendment. No public command,
schema, public JSON reference, Site, Skills, Examples, Learn, FAQ, Glossary, or command-reference
change is required for this internal implementation checkpoint; hosted activation remains separate.

Owner-boundary repair (2026-10-01, reviewed source checkpoint): exact original fixture-byte
equality is now a predicate of the existing administrator-only installed preflight, against the
fixture embedded in that exact Core build. Comment or formatting changes that preserve parsed
semantics must refuse; complete signed graph/record/request/provider comparison remains mandatory.
The job retains source cleanliness, public installation identity/pins, exact invocation mirror
and protected-client terminal/finalization checks, but does not inspect execution-owned files.
Its internal posture qualifies `selected_work_executed: false` with
`selected_work_evidence: client_terminal_only_owner_marker_observation_required`. This is not an
independent filesystem observation, root-custodied semantic attestation or provider cardinality.

Keep earlier private packets frozen. Fresh preparation must freeze the shared read-only observer
`scripts/observe-secret-delivery-pressure-marker.py` from the exact root-owned Core source tree
with the other helper hashes. It accepts no path/UID/environment overrides and performs only a
root, no-follow, descriptor-relative observation of the fixed repository and marker. Only final
marker ENOENT is absence; existing entries (including dangling symlinks) and every earlier
ownership, mode, directory-open or observation error refuse. Run with an isolated interpreter.

In the existing root prestart recipe, after stopped-runner/process/scope/slot checks and the
complete production installed verification, replace the shell marker absence probe with:

```sh
/usr/bin/python3 -I -B /opt/ota-build/service-path-core/scripts/observe-secret-delivery-pressure-marker.py
```

In the existing root post-run terminal recipe, after stopping the runner and confirming principal
process, scope, slot and runner-cgroup absence, perform and retain a separate observation:

```sh
/usr/bin/python3 -I -B /opt/ota-build/service-path-core/scripts/observe-secret-delivery-pressure-marker.py
```

Success is administrator-observed marker absence at that instant, not proof that selected work
never ran. Both call sites must retain exit status and the exact frozen observer hash; never use
the job checkout, change permissions or rewrite earlier packets. The real-principal Linux
regression uses actual job/exec UIDs and 0750/0640 permissions, actual extracted workflow scripts
and the production observer/public Listener verifier. Service topology and the protected-client
response are test stubs, not hosted authority or live provider proof. Root production preflight
coverage separately exercises exact bytes, comment-only mismatch and non-mutating refusal.
Local/source readiness does not authorize another VM, provider activation or attempt.
MUSE's frozen source review found no P1/P2/P3 findings. The operator authorized commit and push of
this source batch on 2026-10-01. The prior two ownership defects and principal-regression gap are
resolved within this scope. That verdict accepts the reported local validation, not an independent
test rerun, hosted proof or activation clearance.

Fresh attempt-6 checkpoint (2026-10-01, independently reviewed outcome): run `36874656786`,
attempt `1`, job `110410885453`, passed at exact Core
`41a979b0d16200cef1608c5f469279b1dc2d2440`, Launcher
`dd667c3d6b63d8d26d9e2a40f831e9d08a7a6864`, Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`, on runner `1384`/group `9`.
Separate human authorization, complete off-clock preparation review and explicit MUSE
reviewer-ready handoff preceded one canonical issue. The frozen signed/installed delta was
independently cleared with no P1/P2/P3 findings. Full production offline V2 graph/record and
independent producer/build comparison passed. After the 300-second operator hold, complete
fresh admission passed with 2482 signed seconds remaining, exceeding the 1200-second floor.
The job's closed projection reports one Core-level GitHub response and one accepted STS
response; JWT claims were matched-unadmitted, returned token discarded, and protected-client
exit 1/finalization validated before IAM Credentials, Secret Manager or selected work.
Separate root terminal observation returned 0 with no principal, scope, slot entry or marker,
and empty runner cgroup/MainPID 0. Marker absence is point-in-time administrator observation,
not proof that selected work never ran. The runner unit separately ended failed/failed with
exit 2 and read-only/denied diagnostic text; do not report a clean runner-unit exit. No public
capture record or independently root-custodied semantic attestation was produced.
Provider/pool were disabled; reviewed teardown returned 0 and final readbacks confirmed exact
temporary-resource absence/disabled soft-deletion. Retention:
`docs/pressure/retained-artifacts/secret-delivery-google-sts-live-36874656786.json`.
MUSE confirmed this meets the bounded job-observed positive STS bar, with no P1/P2 findings;
its sole P3 diagnostic-ordering wording correction is applied and retention is cleared.
It does not close all of Step 7 or authorize the next provider-contact/admission slice.
Provider/lower-layer cardinality, condition enforcement, independent JWT verification,
revocation and token-memory erasure remain explicitly `not_proved`. Cloud lifecycle, runner
retirement, branch/dispatch freeze and administrator cleanup are repo-owned external behavior.
Later delivery/selected-work continuation were not attempted. No new public command, schema,
authoring concept, first-party public propagation, merge or release is implied by this record.

Implementation record (2026-09-30): the source batch implements this boundary with no live
contact or installation. MUSE's initial frozen source review found only a P2 requiring
independent post-response JWT-expiry coverage while binding authority remains fresh. That
test-only refinement is implemented, cleared by MUSE's focused frozen recheck, and locally passes,
including the final snapshot suite and paired 14-case network-disabled actual-child cleanup
fixture. The implementation-only batch is locally complete; see the live handoff for exact
validation. The operator authorized commit and push of the reviewed source checkpoint on
2026-09-30; this does not authorize hosted proof, Google contact, configuration changes,
or any later provider/selected-work gate.

### IAM Credentials Checkpoint (First Network-Disabled Batch Active 2026-10-01)

The independently reviewed bounded STS outcome above meets that checkpoint's job-observed
acceptance bar. The next proposed Step 7 operation is one service-account access-token request,
followed by deliberate terminal refusal before Secret Manager, materialization, injection or
selected work. The independently reviewed proposal is activated only through the first
network-disabled batch record below. No production IAM network route, IAM grant, provider
configuration, issuance, runner or hosted attempt is activated. Step 7 remains active; Step 8
and V12.2 remain inactive.

Independent plan review: MUSE's initial review found one P2 requiring IAM timestamp
provider-format compatibility. The amended parser/fractional-expiry requirements below cleared
that planning blocker in the frozen focused recheck, with no remaining P1/P2/P3 findings and all
four plan/handoff/source hashes unchanged during review. The parser repair and first-batch
source/test gate remain open. Review clearance is not implementation or hosted authorization.

#### First Network-Disabled Batch Activation Record

The operator requested this independently reviewed implementation batch on 2026-10-01 and
explicitly authorized this activation/evidence documentation commit and focused checks.
The activation takes effect when this record is committed, before source implementation.
It permits the Core-owned IAM timestamp repair, retained one-shot STS/IAM ownership and
network-disabled production-path tests described below. It permits no IAM production transport,
Google connection, workflow creation/dispatch, credential grant, provider configuration, host,
issuance, materialization, injection, selected work, merge or release.

The candidate/producer/request allowlist boundary is frozen for this batch: production private
relay admission continues to admit only the existing GitHub-only and STS-only workflow references.
The proposed IAM reference may be admitted only inside `cfg(test)` fixtures to construct exact
signed V2/V4 truth for the fake transport and real-child cleanup tests. No existing serialized
request shape or V1/V2/V3 semantics change, and no IAM workflow file is added. Core continues to
pin the same Launcher and Protocol revisions. The future production IAM route, administrator
producer/request/preflight surfaces and network transport require the separate next source gate.
The connected Launcher extension is test-only and reuses the existing actual-child completion
and cleanup fixture; it does not change production behavior, schemas, versions or Core's pin.

Acceptance checklist:

- [x] Activate through this authorized documentation commit before implementation; keep unrelated changes intact.
- [x] Repair the production IAM parser and lock provider grammar plus full-precision expiry.
- [x] Implement the one-shot retained owner without promoting either existing route.
- [x] Pass network-disabled production-path mutation/refusal/disposal tests and actual-child cleanup.
- [x] Obtain independent frozen source review and reconcile the handoff and affected surfaces.

Local source-batch result (independent review cleared, source commit authorized): the production IAM parser
preserves full fractional precision and accepts documented trailing-zero forms. The private
non-cloneable owner retains the exact consumed V4 context, signed V2 graph/record and conservative
STS expiry through one fake IAM call, rechecks authority and both clocks before/after transport,
and drops the new token without releasing selected work. Production relay admission remains
OIDC/STS-only; IAM is admitted only by `cfg(test)` fixtures. The bounded response seam now also
rejects oversized decoded headers instead of relying solely on ureq's wire-header limit.
Initial frozen source review found one P2: sampling time before full signed-truth/JWT validation
could permit dispatch with a stale observation. The repair finishes that validation first, then
samples wall/monotonic time and applies lightweight canonical binding/JWT/credential deadlines
before dispatch and after parsing. Clock callbacks assert full context validation completed;
eight additional cases require zero corresponding calls across expiry or final clock failure.
MUSE's focused frozen repair recheck cleared the P2 with no remaining P1/P2/P3 findings and
all nine hashes/both HEADs unchanged. It inspected source and retained logs without rerunning
tests. Only the review-outcome handoff/plan records changed afterward. This clears source
commit readiness subject to human authorization, not hosted/provider readiness.
macOS provider-client tests pass 16/16, signed-owner matrices 2/2 (52 IAM and 14 STS cases),
legacy no-promotion 2/2, production admission refusal 1/1 and the existing STS workflow 5/5.
The network-isolated Linux/arm64 actual-child fixture passes 1/1 across all 52 IAM cases,
including successful fake acceptance followed by deliberate refusal, durable completion and
exact child/scope/cgroup/active-slot cleanup. Its Launcher change is test-only on local HEAD
`6fd8cde8782b772fb22eead5f8c691f036b425b5`; Core's production pin remains unchanged.
This does not establish hosted Linux/X64, IAM provider acceptance, independent semantic custody,
delivery, memory erasure or provider revocation. The existing STS child-fixture preservation
rerun also passes 1/1 across all 14 cases after the clock-ordering repair. Local logs are
`/tmp/ota-iam-offline-expiry-repair.log` and `/tmp/ota-sts-offline-expiry-repair.log`.
No Ota invocation scopes remain after both runs.
Formatting/sync/diff checks pass; strict
warnings-as-errors Clippy is not a passing repository gate (863 diagnostics, with the filtered
provider/binding diagnostic in unchanged STS code). No unrelated lint cleanup is included.

#### Production IAM Source-Enablement Batch Activation

The reviewed offline source batch is committed at Core `b789fad6`; its connected Launcher
test-only extension is committed at `b5c8f99`. Core's production Launcher and Protocol pins
remain `dd667c3d6b63d8d26d9e2a40f831e9d08a7a6864` and
`e5fe1c83e562e02f60e27026c7148918bd016155`. The operator requested moving to the next
gate on 2026-10-01. After MUSE's planning clearance, the operator explicitly authorized this
source-only batch, its activation-record commit, the dedicated protected IAM workflow edits
and focused Cargo/Actionlint checks. Activation is committed at
`a1b06cb1358f3454f078e572503eaeab925dee5c`, before implementation.
Source commit/push and any live attempt remain unauthorized.

Checklist and acceptance for the next source-only batch:

- [x] Obtain independent planning clearance and task-specific authorization for the new
  `.github/workflows/secret-delivery-google-iam-live.yml`; commit an activation record before
  changing production admission. No `ota.yaml`, `AGENTS.md`, Cargo lock/pin or release change
  is presumed. Focused direct Cargo/Actionlint checks also require task-specific authorization.
- [x] Add a closed administrator request V3, kind `secret_delivery_iam_pressure_authority_request`,
  with identity domain `ota.secret-delivery-iam-pressure.authority-request.v3\0`. Carry every
  existing invocation field plus one `iam_target` object containing exactly
  `workload_identity_provider`, `google_project` and `service_account`. Reject duplicate/unknown
  fields, missing targets, route/version/kind substitutions, invalid tuples and V1/V2 fallbacks.
  V3 admits only the exact IAM workflow/ref already named below. V1 and STS V2 parsing,
  identity domains, targets, workflow admission and rendered bytes retain their existing meaning.
- [x] Extend the existing Core pressure producer to bind the complete V3 request into the
  existing signed authority payload V2. Derive provider/pool/audience with the existing canonical
  tuple validator; set the named account/project together and retain the same non-executed
  secret fixture identifier/version under that project. This creates no Secret Manager access.
  Reuse the current Launcher signing/install path unchanged; public installation evidence binds
  the V3 request identity, while the signed private payload carries the account and full graph/record.
- [x] Extend the existing offline administrator preflight to inspect V3 without weakening V2.
  Rebuild and compare the entire signed payload and public installation from protected inputs,
  derive the IAM endpoint through the production operation plan, and compare the exact disabled
  WIF provider readback/claim condition for the request. Retain the existing fixed protected paths,
  descriptor/owner/mode checks, byte-exact contract check and no-mutation/no-contact posture.
  Its report must identify the IAM request and target without treating configuration readback
  as admission, independent provider attestation or service-account policy proof. The separate
  hosted packet must inspect service-account-scoped policy and inherited access; this source
  preflight does not claim to have proved either.
- [x] Admit IAM only through its exact retained signed candidate and consumed V4 binding. Reuse
  the private STS-to-IAM owner, exact request builders, fixed no-proxy/direct-TLS ureq posture,
  bounded response reader and fresh post-validation clock checks. Add one production IAM callback
  to that route only; no retries, redirects, renewal, ADC/metadata/credential-file fallbacks,
  alternate target or standalone bearer inputs. Revalidate the exact HTTP request and fixed
  transport inside the callback before `Agent::run`. For both STS and IAM on the new route,
  finish callback validation and HTTP/agent construction before a fresh lightweight wall/monotonic
  temporal check immediately before `Agent::run`. Apply the retained binding/JWT/transaction
  deadlines and, for IAM, STS-token expiry with the same canonical temporal checks; do not
  rerun expensive graph/context validation after this final sample. Failure or clock uncertainty
  prevents the send seam. Keep Core callback counts distinct from actual send/provider-contact
  claims. Preserve the old GitHub/STS terminal routes.
- [x] Keep CLI orchestration thin. Return a closed internal checkpoint posture for the selected
  route, count only Core-level calls, and always fail terminally after IAM acceptance/refusal.
  Existing OIDC/STS diagnostics keep their meaning; the IAM diagnostic explicitly refuses before
  Secret Manager, materialization, injection and selected work. No public JSON schema changes.
- [x] Prepare one manual-only IAM workflow using the existing protected installation/principal
  reconciliation and cleanup path. Expected Core revision, WIF provider and service account are
  comparison-only public mirrors of the installed request, never runtime target overrides.
  Reconstruct V3 identity locally and require it to match installation evidence before invoking
  the client once. Retain only a closed non-secret job-observed projection after exact terminal
  refusal and cleanup checks. Never read the recipient-owned contract from the unprivileged job,
  publish tokens/provider error bodies or imply certification/independent custody/cardinality.
- [x] Prove V1/V2 byte/identity preservation, V3 field/version/domain mutations, producer and
  complete installed preflight agreement, account/project/provider/invocation substitution,
  exact route/no-promotion dispatch and refusal counts using network-disabled tests. Exercise
  preparation-delayed expiry/clock failure for both new-route sends through the production
  preparation/temporal path and an injected send seam: zero sends must occur if HTTP/agent
  preparation crosses a deadline. A clock sample before entering the callback is insufficient.
  Exercise
  the rendered workflow as the real non-root principal in an isolated Linux fixture, including
  wrong mirrors/request/build/owner, pre-client failures, token redaction and cleanup failure.
  Preserve the 52 IAM/14 STS actual-child cases; no provider connection is needed for source proof.
- [x] Obtain frozen MUSE source review, then request commit authorization (approved 2026-10-02). Assess first-party
  propagation: these remain non-default internal pressure/helper surfaces, not a supported public
  command/JSON/authoring contract. Core changelog and handoff are required; Site/Skills/Examples/
  Learn/FAQ/Glossary/schema/command cards need changes only if implementation introduces a
  public term, command or support claim. Do not silently waive a changed public surface.

Risks: request-version substitution must not widen old routes; public mirrors must not acquire
authority; expensive reconciliation must not make dispatch clocks stale; and an internal source
route must not be represented as a proven provider path. Runtime source admission is distinct
from installed authority and operator permission. No hosted activation follows automatically.
Fresh cloud resources, API enablement, accounts/grants, authority issuance, runner changes,
workflow dispatch, IAM provider contact and teardown remain a separately authorized hosted gate.
On 2026-10-02 the operator authorized the temporary AWS host lifecycle and exact-resource
cleanup only. London on-demand `m6i.xlarge`, Ubuntu 24.04 X64 and encrypted auto-delete gp3
are selected; launch is deferred until the source/hosted packet is ready to avoid idle billing.
No host exists yet. This partial infrastructure authorization does not activate Google resources,
grants, authority issuance, the runner, workflow dispatch or provider contact.

Initial frozen planning review found one P2: the existing owner clock sample precedes callback
HTTP/agent preparation. The proposal now explicitly requires a fresh lightweight temporal check
after that preparation at both actual send boundaries, plus isolated zero-send regressions.
MUSE's focused frozen planning recheck cleared the P2 with no remaining P1/P2/P3 findings.
Both document hashes and repository HEADs matched during review. Only this review-outcome record
and the corresponding handoff changed afterward. Planning readiness is cleared; the operator
authorized source implementation as recorded above. No hosted activation follows from this.

Source implementation is reviewed and authorized for commit/push on 2026-10-02.
The first frozen batch's macOS checks pass: producer/preflight 20/20 (including
V1/V2 preservation and V3 mutation/refusal), provider client 16/16, IAM matrix 1/1 across 58
cases, STS matrix 1/1 across 14 cases, production router 1/1, IAM workflow 1/1 and existing
STS workflow 5/5. The six added IAM cases exercise positive send guards and zero-send refusal
when HTTP/agent preparation expires or the final clock fails, separately from callback counts.
Actionlint and Python syntax pass. That batch's isolated Linux/arm64 actual-child checks passed
58 IAM/14 STS cases, with no scopes left, and both rendered workflow/principal checks passed
in fresh network-none containers. The serial build passed after a combined build was memory-killed.
MUSE's first source review found one P2: V3 rewrote invocation project/resource after resolving
the snapshot for `ota-pressure`; regeneration-only preflight repeated the inconsistency and
runtime correctly refused it before V4. The producer now canonically validates/selects the
target before snapshot construction and resolves binding/source identities once. Offline
inspection verifies raw signed stores itself, derives its evidence internally and shares the
same runtime semantic candidate derivation and production operation planner. Only a seven-field
plan projection escapes; no snapshot identity, candidate, V4 binding, provider owner or dispatch
handle is returned. Protected runtime correlation checks and snapshot-bound wrapping remain
runtime-only. Repaired producer/preflight 21/21 passes, including the requested-project
locator/full operation plan and a validly signed locator mismatch refused by shared semantics.
Final repaired authority/snapshot tests pass 27/27, the serial offline Linux build and default
library check pass, and fresh network-disabled Linux/arm64 actual-child checks pass all 58 IAM
and 14 STS cases with no Ota invocation scopes left. Logs:
`/tmp/ota-iam-repaired-source-send-gate.log` and `/tmp/ota-sts-repaired-source-send-gate.log`.
Both rendered workflow/principal checks pass in fresh network-none containers; the existing
STS root installed-preflight production-entrypoint success/refusal regression passes 1/1 there.
These are offline source regressions, not installed or hosted IAM proof. MUSE's frozen repair
recheck has no remaining P1/P2/P3 findings; all reviewed hashes and both HEADs matched before
this outcome was recorded. Only review-outcome records changed afterward. The final passing
checks satisfy MUSE's conditional source readiness; the operator approved source commit/push
on 2026-10-02. The Launcher test-only extension is committed at
`77478ae9fad170d2484aa4409b0bb3032436a52c`; neither production pin changes.
The Launcher extension is test-only and
neither runtime pin changes. Site/Skills/Examples/Learn/FAQ/Glossary/schema/command cards need
no changes because these are internal feature-gated pressure surfaces with no new public CLI,
JSON, authoring or shipped support claim. Core changelog/handoff and the sync waiver record this.
This source batch does not close the hosted IAM gate or establish service-account policy proof.

#### Hosted Preparation Delta (2026-10-02; Not Activated)

- Reviewed source is pushed at Core `d67886f27f6ee76766401d85427b0e4ea63694e2`; the
  Launcher test-only extension is pushed at `77478ae9fad170d2484aa4409b0bb3032436a52c`.
  Production installation pins remain Launcher `dd667c3d6b63d8d26d9e2a40f831e9d08a7a6864`
  and Protocol `e5fe1c83e562e02f60e27026c7148918bd016155`.
- Read-only GitHub workflow lookup returned 404 for the new IAM workflow. The default branch
  is `main`; the documented `workflow_dispatch` registration prerequisite is not satisfied
  merely by publishing a new workflow on this implementation branch. This is a hosted
  scheduling prerequisite, not an Ota runtime gap. MUSE cleared an inert same-path registration
  stub with matching inputs, empty permissions, an ordinary hosted runner and unconditional
  refusal; do not copy the protected IAM implementation onto `main`. No automatic merge,
  default-branch change, alternate trigger,
  existing-route promotion or weakened admission is permitted. Any main change requires
  separate authorization and passing exact-head gates.
  The operator has now authorized only the inert stub promotion. MUSE cleared the exact
  one-file registration batch at `23ca11096211c8fc25c945ed0d684dc5859c8cbc`, based on main
  `8422f2ce915c2c0b853c0074ef1cdf8c69ab2d9f`. Source Release Gate `36941232116`, registration
  Release Gate `36942569361` and all six applicable registration-branch workflows passed.
  The frozen stub and unchanged main base were rechecked before the authorized one-file
  fast-forward to `23ca1109`. GitHub reports main at the exact reviewed registration commit
  and workflow `372936715` as `active`. This closes registration only: no dispatch, protected
  installation, provider authority or hosted IAM acceptance is established. The real
  implementation remains on `1.6.29-implementation`; the main stub always refuses.
- Read-only Google preparation confirmed project `ota-v121-step7-20260910`, number
  `783599651848`, is active; IAM, IAM Credentials and STS APIs are enabled. This neither
  proves service-account/inherited policy nor authorizes grants or provider contact.
- The AWS host lifecycle is separately authorized, but launch stays deferred until scheduling
  readiness and the off-clock packet are cleared. Freeze exact source/workflow/build identities,
  fresh resource names and teardown ownership before creation. Use one dedicated VPC/subnet,
  no NAT gateway or instance role, SSH from the current operator /32, encrypted auto-delete
  boot disk and exact resource-ID retirement. Import only a locally generated SSH public key;
  never return a private key through plugin output. Verify the SSH host key against EC2's
  authenticated console evidence before connection. Preserve IMDSv2 for cloud-init bootstrap;
  any later metadata restriction must be explicit and verified rather than breaking key setup.
- Rebuild the packet from committed sources and protected inputs; the predecessor private
  `/tmp` packet is unavailable. Never reuse its expired request, run, tokens or authority.
  Freeze the runner-group selector and workflow scheduling before issue; then retain the
  canonical stopped-runner sequence and the existing installed/pre-start review and signed
  lifetime floor. Google grants/account/pool/provider creation, authority issuance, runner
  start and workflow dispatch remain separately unauthorized.

#### Ownership And Route Preservation

- Reuse Core's existing provider operation plan, request builder, response parser, fixed direct-TLS
  posture and protected V2/V4 transaction path. Do not introduce an SDK, external credential file,
  shell exchange, proxy, ADC, metadata-server or caller-supplied bearer/endpoint fallback.
- Keep the existing GitHub-only and STS-only routes terminal at their current boundaries. In
  particular, the existing STS checkpoint still disposes of its returned token and cannot reach
  IAM Credentials. A future IAM route is selected only by the exact workflow reference
  `ota-run/ota/.github/workflows/secret-delivery-google-iam-live.yml@refs/heads/1.6.29-implementation`
  carried by the independently rederived candidate and signed V2/V4 authority. The source-only
  batch prepares that workflow locally; this plan alone neither publishes nor dispatches it.
  Ambient flags, a
  caller label, workflow inputs or a successful STS response cannot promote an older route.
- Before implementation, freeze the connected candidate/producer/request allowlist changes for
  that exact route. Preserve released V1/V2/V3 semantics and existing request versions; if a
  request shape needs new fields, use an additive version rather than reinterpreting old bytes.
  No Protocol/Launcher version bump is presumed: their existing signed full graph/record and V4
  identity relay remain the ownership boundary unless review demonstrates a concrete gap.
- For the IAM-selected route only, move the accepted STS token, its freshness bounds and the
  exact consumed prepared V4 transaction into one private, non-cloneable, one-shot owner. Retain
  the locally reconciled JWT context needed for existing claim/window checks. Do not return a
  standalone reusable token, accept a parsed token from a caller, or reconstruct authority from
  public identities. A consumed owner cannot dispatch twice, renew, retry, resume or change route.
- Immediately before request construction, immediately before the injected transport call, and
  after the bounded response, rederive the operation from the retained candidate, compare the
  complete signed V2 graph/record against Core's embedded expectation, and validate the exact
  consumed V4 binding, current expiry and unchanged fixed transport posture. Apply the existing
  protected JWT claim/window checks; neither STS acceptance nor IAM acceptance admits selected
  execution. Changed service account, project, realization, invocation, run/attempt/workflow,
  profile, implementation, dependency expectation or retained session must refuse.

#### Exact Request, Freshness And Disposal

The request is exactly one POST to the plan-derived
`https://iamcredentials.googleapis.com/v1/projects/-/serviceAccounts/{ACCOUNT}:generateAccessToken`,
with one bearer authorization value from the retained STS token, JSON media type, and the existing
builder's body containing only `scope: ["https://www.googleapis.com/auth/cloud-platform"]` and
`lifetime: "600s"`. Omit `delegates`; reject different method, account, origin, path, query,
headers, media type, scope, lifetime or body before the transport seam. Account selection is
signed private candidate truth, not an administrator mirror or workflow input. Mirrors may be
compared as an additional refusal guard but create no authority.

The new owner must enforce a transaction deadline no later than 600 seconds after the first
provider dispatch and no later than the existing binding expiry. Record the STS dispatch-start
clock and derive its conservative token expiry from that time plus the accepted positive
`expires_in` (at most 3600 seconds), not from a later response time. Use checked arithmetic,
trusted wall-clock expiry checks and monotonic elapsed-time bounds; clock failure, backward
observation, expired JWT/binding/STS credential or exhausted transaction time refuses without
renewal. Provider timeouts must not extend these bounds. The JWT remains locally matched but
unadmitted, and every response must pass post-response authority/deadline checks before success.

Use the existing 16-KiB token-response body bound, bounded response-head handling and redacted
errors. Require the expected successful JSON response head and exactly one nonempty bounded
`accessToken` and provider-canonical `expireTime`. Reuse the existing parser path, but repair its
format check before this batch can close: Google-generated RFC 3339 timestamps are Z-normalized
with 0, 3, 6 or 9 fractional digits, including trailing zeros. Comparing time's generic RFC 3339
round-trip text rejects valid `.100Z`/`.100000Z`/`.100000000Z` responses because it prints `.1Z`.
Validate the admitted provider grammar and actual calendar/time value instead; accept those
documented widths without accepting offsets, malformed dates or arbitrary timestamp syntax.
Duplicates, extra fields, malformed timestamps, expired tokens and expiry more than 600 seconds
beyond trusted response observation refuse. Compare the full timestamp precision against that
trusted observation and the 600-second bound before deriving any seconds-only summary; fractional
truncation must not admit a timestamp just beyond the bound or lose its future/expired distinction.
Do not use the new service-account token for a subsequent call: drop it at
this checkpoint, along with all retained JWT/STS/request/response buffers, on success or refusal.
Best-effort buffer clearing is not process-memory erasure, provider revocation or non-exfiltration.

Google's [generateAccessToken reference](https://docs.cloud.google.com/iam/docs/reference/credentials/rest/v1/projects.serviceAccounts/generateAccessToken)
defines the required wildcard account path, request and response. Its
[WIF access guidance](https://docs.cloud.google.com/iam/docs/workload-download-cred-and-grant-access)
describes service-account-scoped Workload Identity User grants. These sources describe provider
semantics, not Ota activation, evidence or authorization.

#### First Batch Acceptance And Later Gates

1. Network-disabled source batch: a private injected fake transport must exercise the production
   retained-owner path, existing builders/parsers and bounded reads. Prove zero IAM calls for any
   pre-dispatch refusal, exactly one Core-level call for success or post-dispatch refusal, no
   replay/rebinding, conservative STS expiry including response delay, binding/JWT/transaction
   expiry before and after each call, overflow/clock refusal, and token disposal. Independently
   mutate account/operation/graph/record/session and every material HTTP/request/response field;
   exercise oversized/truncated/read-failure bodies and invalid response heads. Preserve explicit
   IAM timestamp regressions for all documented fractional widths, trailing-zero cases, invalid
   widths/offsets/calendar values, exact expiry boundaries and fractions immediately beyond the
   600-second bound; exercise them through the production owner/parser, not a duplicate validator.
   The named Ota gap is IAM expiry provider-format compatibility, owned by
   `src/secret_delivery_provider_client.rs::parse_service_account_token_response_v1`.
   Preserve explicit
   no-IAM regression coverage for both existing routes. No new production Google connection or
   IAM-selected workflow dispatch is enabled in this batch.
2. Connected protected-child proof: reuse the actual Core child, retained completion session,
   Launcher relay, durable terminal acknowledgement and child/scope/cgroup/slot cleanup path in a
   network-disabled Linux root fixture. Exercise successful fake IAM acceptance followed by
   deliberate refusal and every newly introduced refusal category. Plain parser/unit tests alone
   do not close this gate; arm64 fixtures do not establish hosted Linux/X64 provider evidence.
3. Separately reviewed network-enablement batch: only after the above evidence and frozen source
   are cleared, authorize exact production request/response transport and the dedicated IAM
   workflow/request/preflight/closed-posture surfaces. Review first-party propagation at that
   implementation boundary. Network implementation permission is not hosted activation.
4. Separately authorized hosted attempt: use fresh pinned sources, fresh protected Linux/X64
   state, one new root-owned request, the stopped-runner sequence and independent installed
   graph/record/producer/build verification. Review the complete off-clock packet before issue
   and the bounded signed/installed delta before start; retain the final admission-time floor.
   Do not reuse the STS checkpoint's destroyed resources, tokens, authority or expectations.
   The provider scope must include a fresh dedicated service account with no resource-access
   roles or keys and one service-account-scoped custom-role grant only to the exact reviewed
   federated principal. That fresh project role must contain exactly
   `iam.serviceAccounts.getAccessToken`; the predefined `roles/iam.workloadIdentityUser` is not
   an acceptable fallback because it also grants ID-token minting and account metadata reads.
   Independently read back the exact role definition, account policy and inherited access
   as well as the exact-run/workflow-commit WIF condition; broad principal sets, project-level
   impersonation, delegation, Token Creator fallback or Secret Manager grants refuse. Grant
   changes, API enablement, custom-role/service-account creation and all infrastructure actions
   need the separate human authorization and reviewed teardown, including grant/account/custom-role
   retirement and provider/pool disablement. This paragraph authorizes none of those actions today.

#### Off-Clock IAM Preparation Review (2026-10-02; No Hosted Activation)

The fresh operator artifact is `/tmp/ota-iam-live-20261002-preparation`: `packet.json`,
`operator-packet.md` and `policy-readbacks.md`. It is preparation data, not a second live handoff
or an executable authority request. Launch/issue/dispatch readiness is false; final Core revision,
queued invocation, Listener version, installed artifacts and signed authority remain unresolved.
After retention, freeze the resulting clean local/remote implementation HEAD and applicable
exact-head gates; preserve the reviewed workflow/helper hashes and production Launcher/Protocol
pins, and independently review any changed source identity before host creation.

MUSE's initial review found two P2 planning gaps: the predefined Workload Identity User role
permits alternate credential minting, and the resource-access audit did not name a closed
executable readback set. The proposed correction uses the one-permission custom role above,
with no ID-token/signing/delegation/attachment permission or predefined fallback. The live
read-only permission probe is GA with omitted `customRolesSupportLevel`; Google's
[custom-role reference](https://docs.cloud.google.com/iam/docs/creating-custom-roles) defines
that omission as fully supported. Custom-role creation/definition readback are not yet performed.

The closed audit freezes project description/ancestry and each ancestor policy, direct fresh
account policy/user-managed keys, exact custom-role definition, full unfiltered paginated project
asset/resource-policy inventories and targeted account/principal/pool/public-member queries.
It accepts only the sole intended fresh-account grant, no extra account/federated or applicable
broad/public bindings within the enumerated set, and converged inventory reconciled with direct
live readbacks. Unsupported material resources require frozen native readbacks; missing material
coverage blocks launch/issue rather than becoming an absence claim. Cross-project trust, external
group membership and unsupported data authorization remain explicitly `not_proved`; any required
acceptance narrowing needs a reviewed plan amendment, not an operator waiver.

Read-only observations: the proposed pool/account `ota-iam-20261002-a1` and custom role
`otaIamAccess20261002A1` are absent; project ancestry is project-only. The existing Compute Editor
account and all historical pools/groups remain outside teardown ownership. A fresh runner group
would admit only the one exact IAM branch workflow in Core; no runners or IAM runs currently exist.
Cloud Asset API is not enabled, so the full baseline audit is unavailable and remains a launch
blocker; no API enablement or asset query was attempted. Two read-only AWS plugin calls returned
internal errors without API-call evidence, so current AWS identity/connectivity also remains a
launch blocker. Neither blocker authorizes fallback credentials, permissions changes or a VM.

Google API/custom-role/account/provider/grant changes, GitHub runner/group mutation, authority
issuance, runner activation and the hosted attempt still require separate human authorization.
No provider call, installed proof, hosted IAM acceptance, resource access, delivery or selected work
is established. This planning-only correction changes no public CLI/JSON/schema or authoring
surface; Site/Skills/Examples/Learn/FAQ/Glossary/command reference need no propagation.

Frozen MUSE re-review resolved both P2s with no remaining P1/P2/P3 findings. It clears only
documentation/preparation retention: the Cloud Asset baseline and current AWS readiness remain
hard launch blockers, and final source/PREISSUE/installed reviews remain open. Reviewed packet
hashes are `7801f34228a70b062121760ae61ee214c4c38590debaf90e8e0a6d3ed8710c77` (JSON),
`2724e8d273a5bdc670d350f77175bd8300b6c51a3855d6897fa67387cf53f6a8` (operator recipe) and
`52da5aca0c6bfcdc77cd859ef1c09cc9a42c892d5c1db28a589ae216b93ebf6a` (closed policy readbacks).

Subsequent authorized baseline observation: the operator approved only
`cloudasset.googleapis.com` enablement for the read-only policy audit; operation
`operations/acat.p2-783599651848-a5446aac-c887-4cde-8818-da0bf334b729` succeeded.
Initial raw outputs, completion timestamps and hashes are retained in the preparation directory's
`baseline-audit` artifacts. Retained outputs from two unfiltered invocations are byte-identical: 142 resource
rows across 16 types and three policies. Four fresh-account/principal/pool/public binding
queries are empty; exact proposed resources remain `NOT_FOUND`. Native policy reads confirm
the older reader's canary/federation grants, outside fresh-attempt ownership. Native Logging
reads found three views absent from CAI; their direct policies have no bindings. Stable index
output therefore does not close inventory completeness. Material coverage reconciliation and
independent outcome review remain open; no final baseline/PREISSUE/hosted acceptance is claimed.
The authorized API change remains enabled; restoration needs explicit cleanup permission and
no competing use. All other Google/GitHub mutations, host launch, authority issuance and dispatch
remain inactive. AWS connectivity still requires reconnection after the prior internal failures.

MUSE identified missing per-command success provenance and unbound artifact paths in that initial
retention. A fresh read-only collection, `baseline-audit-v2`, records all 30 exact invocations,
exit statuses, UTC start/end times and stdout/stderr hashes (10:30:38-10:31:29 UTC). All 23
expected-success inventory/project/native policy reads exited 0; six fresh-resource reads exited 1
with explicit `NOT_FOUND`. The older indexed STS2 pool policy read also exited 1 because the pool is
`DELETED`; native describe confirms `DELETED`/disabled. It is not treated as an empty policy.
The older active pool's direct policy is empty. Repeated new inventories/policies are byte-identical;
142 rows contain 141 unique names and one exact duplicate STS2 provider row. This still does not
close material/native coverage or eventual-consistency limits. The private operator packet was
updated to explicitly supersede its disabled-API observation; all readiness flags remain false.
Frozen evidence identities under `/tmp/ota-iam-live-20261002-preparation`:

- Initial raw manifest `baseline-audit-sha256.txt`: `b40d185b85d1381d81c769d97ffe3c33dec9ec71c478aa290c100b2197dcbeb7`.
- Initial timestamps `baseline-audit-timestamps.txt`: `20e479a20e4a70c3384ccc4e6fd99f96ad861aef6561c2f4556c083657272de2`.
- Updated result `baseline-audit-result.md`: `ce6c1ccefb9ef63bcbf5da7331ad9d5bdf185b34118be26a1646210754ba87d9`.
- Command ledger `baseline-audit-v2/command-ledger.json`: `c40a9f945132d127c51c56c0b057dfc6e2e743505662cbd6924fdeb55428b273`.
- New raw/command manifest `baseline-audit-v2/sha256.txt`: `e57a7a3a34fe8a6a7a749bd3d7bda41491af38a86c1d0a44f1b89cbb24e7d480`.
- Updated packet JSON/operator/readback specification: `9bf1535f9cfb8c55e49bc0d883f3e07acfb0cd7d331a3478e0d3666802bac20c`,
  `4507366b0527f855f5e9ea25258bb5adb59d28d380510234d4189ce8fd25a0e7`,
  `a1972ae5931b102e915fe7d1ba65bea1e4303469108de27cc52cd4392b5e6035` respectively.

MUSE's frozen delta re-review resolved both P2 evidence-retention gaps and the stale-packet P3.
The remaining P3 success/failure sentence was corrected to the exact 23/6/1 status split above.
Review clears docs-only observation retention, not material coverage acceptance, AWS readiness,
final source/packet freeze, host launch, PREISSUE, provider activation, issuance or dispatch.

The subsequent read-only coverage candidate classifies all 16 indexed types and each of the
141 unique indexed names, preserving the one identical duplicate row. It also adds the three
native-only LogViews and five native-only deleted pools: 149 classified names total. Material
direct policy surfaces are project, subnetworks, service accounts, active pools, secrets and
LogViews; other material rows name their parent/project policy boundary. The project billing
association is not treated as a project-scoped allow-policy surface; external billing trust is
not proved. Classification is grounded in Google's
[allow-policy resource reference](https://docs.cloud.google.com/iam/docs/resource-types-with-policies)
and connected service references, not absence of a CLI command alone.

The `coverage-audit` collector retains all 58 successful native read invocations/statuses/stdio
hashes from 10:41:32-10:43:08 UTC. Native subnet/account/secret/global-log-bucket lists match
the observed inventory; all 42 direct subnet policies are empty. Native pools identify one
active historical pool and six DELETED/disabled pools; the active pool policy is empty. Both
account policies, the synthetic-secret policy and three direct view policies are retained.
An unfiltered aggregated native instance list shows zero Google VMs. None of these observations
authorizes teardown of historical resources or a no-exfiltration/global no-access claim.

MUSE's frozen review found no P1/P2 classification gaps and two P3 evidence-retention repairs.
The repaired native parity artifact records source hashes, exact normalized keys, counts/deltas
and a retained deterministic generator. Its project ID/number alias is derived from the frozen
project readback, not assumed from matching basenames. Credential wording now states that no
target secret payload or provider-minted workload token was requested or retained; gcloud used
operator authentication. MUSE's final frozen recheck cleared the evidence-retention and GCP
credential/key/network guard repairs with no P1/P2/P3 findings. This clears narrow docs/private
preparation retention only, not host launch, pricing/operator IP, final source/packet, installed,
provider, issuance or dispatch readiness. The boundary is the enumerated project
resource/ancestor allow-policy set, not arbitrary cloud inventory, external billing/group/trust,
unsupported data authorization or provider resource-access proof. Later native/index changes
reopen coverage; fresh direct role/account/provider/target readbacks remain mandatory at issue
and immediately before start. The operator reports AWS suspension and selects GCP; AWS readiness
is no longer a prerequisite. GCP host-delta review, final source/packet review and all hosted gates
remain open. No cloud mutation occurred in this coverage collection.
Frozen preparation-directory identities:

- `coverage-ledger.json`: `69e42add33de9550757519afdc097aed980e10f63a3844e1dd223dbb84102484`.
- `coverage-audit/command-ledger.json`: `6b9a55e41f1693558430a7f3c3c2cabd0d36b8f2b1295ac9f410e3df19b2a2d0`.
- `coverage-audit/sha256.txt`: `0945b6e89377e7c911092e927be1557d2d5730ba4b1aa1d5c100ea9dcaac6a93`.
- `coverage-result.md`: `89616f9fea685026c68a1a8beed134d5817556598efcba3b40bb485ab7c371b7`.
- `coverage-native-parity.json`: `9c8d3814c837fd4b76a75c1c7095ca86437a3c29ab03fdf77c83427921a6e7fe`.
- `build-native-parity.sh`: `23ccf93c6dbf71567f044ee2fc53461ca2ae04536e052b53c97bbf3947e5026a`.
- Classification/generator/collector: `f461fde74e4a677cbbdc97dc03f3939a42e92509b43ae01f50261a86106a7716`,
  `a21aa95f1def91ce7a1716e6f9e865401e5612cccca851b60d4430946a6f52ee`,
  `4cd3d557e9762951b1aa15b1b2545383bcb025cdaaf84f8161fcdd9af32fbfdc` respectively.

The operator-selected GCP replacement proposes one on-demand London `e2-standard-4` X64 host
(4 vCPUs/16 GiB), a 60 GiB auto-delete balanced disk, dedicated custom network/subnet, ephemeral
IPv4 and SSH restricted to a fresh operator /32. Creation explicitly requires
`--no-service-account --no-scopes` and `block-project-ssh-keys=TRUE,enable-oslogin=FALSE` metadata.
Before any build/install or runner registration, retain the exact created instance's fresh
`instances describe`, network/subnet describes and unfiltered project firewall-rules list. Require
`verify-gcp-host-readbacks.sh` and its jq guard to accept ledger-bound project/zone/name/numeric
IDs, no `serviceAccounts`, the exact three metadata keys and a pre-frozen operator SSH-value
SHA-256. The closed key allowlist rejects additional keys/startup metadata. The same guard checks
the exact instance tag/network/subnet and custom unpeered network, and exactly one enabled
dedicated-network ingress rule: TCP/22 from the frozen operator /32 to the expected tag, without
alternate source/target selectors or additional ingress. Freeze expected key hash, names/CIDR/tag
before mutation; append only the returned IDs rather than deriving expectation from observed
configuration. A missing or mismatched readback terminally refuses and retires owned resources before
continuing; no credential repair/fallback is permitted. Repeat before issue/start. Record the
first host key through authenticated control-plane serial output, not independent host-key
provenance; enforce strict known_hosts thereafter. Attach no service account or OAuth scopes;
never inherit the existing Compute Editor identity. The exact publisher image is
`ubuntu-os-cloud/ubuntu-2404-noble-amd64-v20260918`, image ID `763874002631433611`.
Six successful read-only commands collected at 11:59:25-11:59:32 UTC confirm project/billing,
empty VM inventory, machine/image and regional quotas. These are not capacity guarantees,
a cost quote or installed readiness. Verify fresh full pricing and the operator /32 before
launch; avoid idle charges and delete/verify owned VM, disk, firewall, subnet and network.
Creating these resources reopens material inventory/policy reconciliation. Host lifecycle
authorization does not authorize the fresh Google identity/grant, runner/group, issuance,
provider activation or dispatch. All readiness flags remain false and final Core SHA is unset.
The earlier AWS host proposal and baseline packet hashes above are historical, superseded for
host selection by this operator delta. No AWS resources were created by this work.
Frozen GCP replacement identities under the same private preparation directory:

- `gcp-host-plan.md`: `7150c6701576f18d1ae7dfc961618f5cffeaf07909cae26cdcbec2a5f61f593c`.
- `verify-gcp-host-readbacks.sh`: `29c35c641b58f8135a3fdf4806587b19e592cf729fd4f1db59099a1f91431f83`.
- `verify-gcp-host-readback.jq`: `de718034879eff9b12531a174cc668872652b014bbf47e49acacb4d7b550ef9b`.
- `test-gcp-host-readback.sh`: `afc30ab5de198402632d11f70735477787b3a919fbe88603734964e86043e8e4`.
- `gcp-host-audit/command-ledger.json`: `bf33a3cf4f4a5b5715176987476c70a1f63b50beca51578e1a97f47f42d64019`.
- `gcp-host-audit/sha256.txt`: `111efc46e82449907ed99948952273fcff250489bfd58af3aefc0c8e3ebc6eb8`.
- Replacement packet JSON/operator specification: `3d2646ff0966fad439649686b0a13daf14e65484695ef9304bbd7c55ef9d1045`,
  `53538dafd5b753dc5f61abacff29fb9da8f9b08ace00fe9021917e7faa95b35d` respectively.
The wrapper/readback helper's local controls accept explicit/omitted empty account lists and refuse
attached/null accounts, unsafe/duplicate/missing metadata, wrong instance ID, additional SSH keys,
startup metadata, extra tags, network/subnet mismatch, broad CIDR, ports, wrong target and extra
ingress (2 accepted, 16 refused). These are local
fixture controls only, not actual host readback or installed proof.

This is internal operator-evidence retention, not public CLI/schema/JSON/authoring behavior;
Site/Skills/Examples/Learn/FAQ/Glossary/command-reference propagation is not required.

#### A2 PREISSUE Refusal And Retirement (2026-10-02)

After the operator's separate bounded IAM authorization, the fresh A2 host passed its exact-ID
credential/key/network guard and strict serial-linked SSH check. Exact Linux/X64 Core
`4aa1baa320e4bdafdddd2fc3682230730eff1b0f`, Launcher `dd667c3d` and Protocol `e5fe1c83`
builds/installations, restricted runner/group, actual queued V3 request and disabled exact-run
provider were frozen for PREISSUE review. The refreshed policy audit has 146 converged indexed
names/19 types and 156 classified union names, including native-only pool/view state. All 65
native coverage reads and six indexed/native list comparisons pass, including fresh VM/disk
policies, three account policies and all 43 subnet policies. This is enumerated policy coverage,
not global absence or credential non-exfiltration. The custom grant remains access-token-only.

MUSE confirmed one P2 operator-preparation defect: recursive root hardening made `.env` and
`.path` root:root 0600; the job account could not read them. Pinned Listener startup reads `.env`
before command handling, so the planned post-issuance job-principal version probe and runner
startup would fail. Testing `--version` only as root did not prove job-principal readiness.
No other P1/P2/P3 finding was identified in that reviewed scope. No exception to the frozen
terminal infrastructure-error rule was made: A2 was cancelled and exactly retired, not repaired
in place. Run `37052192848`/attempt 1/job `110988059901` ended cancelled with no assigned runner
and zero steps. No signing/provisioning or workload OIDC/STS/IAM credential checkpoint call
occurred; administrator/tool-observed final SSH assertions reported no issued stores, job/exec
process or workload marker. Their stdout retained only a summary, without a contemporaneous
command/status/UTC record. The additive `retirement-a2/host-final-retrospective.md` retains the
invocation and tool-reported exit 0 from this SCOOBY interaction, not an accessible durable
transcript or independent attestation. Adjacent recorded local commands bracket the observation
at 19:35:40Z-19:36:45Z, not exact SSH/remote UTC. The 47 separately timed cloud/GitHub records
remain distinct; original frozen manifests are unchanged and no new remote call was made.

The exact VM, auto-delete disk, firewall, subnet and network are natively absent; no reserved
former IP is listed and the local attempt SSH key is removed. Runner 1385/group 10 return 404.
Account disable/delete returned success and the exact unique ID/email is absent from the native
active-account list; its post-delete describe returns PERMISSION_DENIED, which is retained as
indeterminate, not relabelled NOT_FOUND. The sole grant was removed first. Custom role is
DISABLED/deleted, provider and pool are DELETED/disabled; the pool's initial ACTIVE/disabled
post-delete read is preserved separately from the converged result. No issued token revocation
is inferred. Historical project resources remain outside cleanup; API restoration still needs
separate permission and no competing use.

Frozen evidence under `/tmp/ota-iam-live-20261002-preparation`:

- PREISSUE packet/433-entry manifest: `a4f21b097ad3edd0f7fff403f80d41d111fc4f37ca8b8c0b88b94a6cf4ebd468`,
  `7d060857a18dd2bc20e6b8a7069cfe9c792a14e0a5cf88ada864a52b3de6f62d`.
- Metadata-only refusal delta: `5fd3585ed43f401bff483b819e5c8f9b9a441e9ba77f5a139838e3ccaafec5e8`.
- Retirement command ledger/raw manifest: `cb8bb249405f0b5b894006ca4dc735c8d5ff9565688c71580f60e82752fabd46`,
  `7df6b8dd911280348ec8baa8846d0329507fd1f264dca2e9ae496a41a74108dd`.
- Additive retrospective SSH provenance: `116532aaf79f3adff8f0ed11a4285df5d5f2373191783908efa0ebcf6e78d8be`;
  original summary stdout `850cb673b0842e40aceadf11b1be330e8d180ebdd6ac0922d78acf58f766521d`.
- Staged `register-runner-v2.sh`: `fbf9096b4dfeb3e79bef170b308519f6743ab58c7bc1e2262a1b3f70be68db71`.

The staged repair makes all five runtime/configuration files root-controlled, singular 0640 and
job-group readable but not writable, then requires the fixed Listener `--version` to succeed as
the job principal before issuance. Syntax checks are not fresh-host proof. Independent review,
separate fresh-attempt authorization, rebuilt frozen sources/policies/installed expectations and
the existing stopped/disabled signed-delta/propagation/lifetime gates remain required. Step 7
stays active/open; no automatic retry, renewal, delivery activation, merge or release follows.
This is private operator preparation, not public CLI/schema/JSON behavior; no connected public
Site/Skills/Examples/Learn/FAQ/Glossary/command-reference propagation is needed.
Canonical `ota run ci --agent` completed successfully after these initial docs-only outcome
edits (container mode, Rust 1.95 bookworm). The subsequent provenance wording delta passes
`git diff --check`; neither validation is fresh-host or hosted IAM evidence. Future SSH assertions
must retain contemporaneous command/status/UTC/hash metadata before retiring the host.

The proposed hosted bar is one job-observed GitHub response, one accepted STS response and one
accepted IAM response, each counted only at Core's call seam, followed by validated deliberate
client refusal and exact child/scope/cgroup/slot cleanup. Retain only a closed non-secret status
projection. Independently record administrator marker absence at the observation time, runner
unit status, resource retirement and whether a public custody record actually exists; do not
infer never-execution or independent semantic custody from those observations. A passing job
must not hide a failed runner-unit exit. Provider-side/lower-layer cardinality, enforcement of
every WIF claim, independent JWT verification, root-custodied semantic attestation, resource
access, selected-work authority, revocation and memory erasure remain `not_proved`.

Uncovered material behavior: the network-disabled owner/refusal/cleanup paths are locally
contract-owned and proved within the independently reviewed fixture limits above;
provider policy, cloud lifecycle and runner retirement
are administrator-owned external behavior; IAM provider acceptance remains an inactive hosted
proof gate. Secret Manager, materialization, injection, recipient execution and positive delivery
remain later inactive Step 7 work. This internal proposal changes no public command, schema,
JSON reference, authoring concept or support claim, so Site/Skills/Examples/Learn/FAQ/Glossary and
command-reference propagation are not required here. Reassess at implementation, not by assuming
that a future provider checkpoint has shipped.

## Product Boundary

Ota does not store, rotate, mint, encrypt, export, or centrally manage secret values. It governs
which declared execution may receive a provider-owned value. The repository declares the need, a
separate authority maps that need to a provider reference, and an execution adapter performs
bounded late delivery. Evidence records only non-secret boundary facts.

No provider becomes supported merely because a value can be read from an environment variable,
file, or shell command. Each adapter must prove its delivery and cleanup boundary or refuse before
selected work begins.

Ota does not claim:

- that a provider value is correct, fresh, rotated, or independently administered;
- that a secret was never observable outside the adapter's enforced boundary;
- that selected code, descendants, third-party tools, or remote hosts did not exfiltrate it;
- that a provider reference is a credential or authorization to use the value;
- that cleanup erased process memory or revoked provider-side material; or
- control over raw shell outside Ota.

## Compatibility With Existing Environment Behavior

V12.1 is additive. Existing `env.vars`, `secret: true`, inherited process environment, declared env
sources, task `env`, task `env_files`, container projection, and current remote-secret refusal
remain valid compatibility behavior.

Compatibility behavior does not become governed secret delivery automatically:

- `secret: true` marks a required value as sensitive and cannot have a contract default;
- a value written in `ota.yaml`, task `env`, or any default is not governed secret delivery;
- inherited process or CI environment is caller- or provider-supplied compatibility input, not
  evidence of Ota-owned late delivery;
- redaction does not prove exact recipient injection, inheritance prevention, cleanup, provider
  authority, freshness, or non-exfiltration; and
- remote execution continues to refuse secret forwarding through generated command text or copied
  environment state.

One logical requirement cannot be owned ambiguously by both `env.vars` and the future governed
secret-requirement surface. Validation must require an explicit compatibility link or reject the
duplicate. That link is migration or alias metadata only: inherited environment and env-file
material can never satisfy governed admission, materialization, injection, cleanup, or assurance.
Matching variable names never establish identity, fulfillment, or authority.

## Canonical Domains And Identities

All identities use versioned semantic domains and canonical serialization. Contract-local
requirement IDs and display labels are locators, not authority or semantic identity. A normalized
delivery destination is semantic truth: an environment key, descriptor role, runtime mount target,
provider-mediated slot, or target-relative path materially changes delivery and participates in
`SecretRequirementIdentity`.

### Repository-owned requirement

`ota.yaml` declares a provider-neutral `SecretRequirementIdentity` containing only repository-owned
truth:

- a typed purpose and secret class from a versioned Ota vocabulary;
- a typed delivery destination such as process environment, non-inheritable descriptor, mounted
  runtime handle, or provider-mediated runtime injection;
- exact task/workflow roots and recipient segment ownership;
- explicit propagation posture for dependencies, hooks, helpers, services, containers, remotes,
  proof observers, negative controls, and lifecycle children; and
- execution constraints such as actor mode, environment class, execution mode, target platform,
  runtime boundary, and required capability class.

The requirement cannot select a GitHub secret, Vault path, cloud-secret identifier, provider tenant,
or provider authority. It cannot claim that a provider is available, trusted, fresh, or
independently administered.

Actor, environment, target, event, and workload constraints in the requirement are requested
eligibility only. Actual actor, environment, workload, and event evidence comes from runner- or
provider-owned authority. Caller labels cannot satisfy a stronger posture; unavailable evidence
remains `unknown` and policy decides whether that posture must refuse.

### Authority-owned provider binding

A separately sourced `SecretProviderBindingIdentity` maps exactly one requirement identity to one
provider-specific reference. It binds:

- provider and adapter identity;
- provider tenant, organization, repository, workspace, environment, or equivalent authority scope;
- workload identity or runner-integration identity;
- a provider-reference identity or safe opaque binding identity;
- provider authority and binding-source posture;
- static or dynamic lifecycle posture, including bounded freshness or lease requirements; and
- target execution and capability constraints.

Provider reference labels and paths are normalized into a safe canonical identity or protected by
an opaque private binding identity. A material provider reference is never omitted from the private
canonical binding identity merely because public disclosure is redacted.

The binding source is explicit. Local OSS bindings may be repository-controlled,
workspace-controlled, or caller-selected and retain that weaker posture in decisions and evidence.
Enterprise bindings are administrator-owned control-plane or protected runner-integration truth. A
repository file, task, workflow, environment variable, or CLI argument cannot claim Enterprise
posture or redirect an administrator-owned binding.

`SecretProviderBindingEvidence` binds an opaque source identity, trust-root or source-verifier
identity where one exists, verification result, and derived authority posture. Authority posture is
never accepted as a caller-authored label. Source, trust root, verifier, adapter registration, or
binding substitution refuses. A local source without independent verification remains explicitly
repository-controlled, workspace-controlled, or caller-selected.

Provider-attested lease/version freshness proves only that the provider materialization is live and
bound to the transaction under that provider's protocol. It does not prove that the secret value is
correct, recently rotated, uncompromised, or suitable for the application.

Unknown or duplicate requirement selectors, duplicate bindings for one authority slot, conflicting
binding sources, cross-tenant or cross-repository substitution, and unresolved authority refuse.
IDs and labels cannot resolve ambiguity by precedence.

### Delivery plan

One canonical `SecretDeliveryPlan` exists for each ordered requirement/binding pair. The selected
invocation separately derives an ordered plan set that combines:

- every requirement identity;
- every selected provider-binding identity and its source/authority evidence;
- the complete selected executable closure and recipient segment graph;
- the runner capability profile and selected adapter;
- all applicable policy decisions;
- the exact target execution identity; and
- any V12 effect and realization identities that apply.

Secret-delivery policy is monotonic narrowing only. It may reject provider classes, authority postures, delivery
destinations, environments, execution modes, recipient segments, propagation edges, or lifecycle
posture. It cannot add a requirement, select or replace a provider binding, add recipients, widen
propagation, manufacture capabilities, or convert unsupported delivery into an allow.

Secret-delivery decisions remain `allow | review | deny`. V12 effect decisions remain
`allow | warn | deny`. Final admission preserves both records and their complete rule/source basis:
any `deny` or secret-delivery `review` refuses, while effect `warn` remains a warning and never maps
to `review` or `allow`. Neither domain can authorize, erase, or weaken the other. Invalid
requirements, unresolved or conflicting bindings, unsupported capabilities, untrusted execution
contexts, and hard provider-identity mismatches are unconditional secret-delivery denial.

## Exact Recipient And Capability Semantics

Before provider contact, Ota resolves the complete potentially executable closure: dependencies,
aggregates, lifecycle tasks, services, assertions, hooks, proof observers, negative controls, and
provider-created boundaries. Every node belongs to an explicit recipient segment.

Default propagation is deny. A non-recipient closure node does not force refusal when the adapter
can segment delivery and prove that node receives nothing. Ota refuses when segmentation cannot be
enforced, an unapproved edge would receive material, or recipient ownership is ambiguous.

For process-environment delivery, the honest enforcement unit is the selected process tree. Ota
does not claim it can remove an environment value from arbitrary descendants after selected code
receives it. Exact-process-only delivery requires an enforcing mechanism such as a non-inheritable
descriptor or provider/runtime-mediated injection.

### Destination collision and binding selection

Every requirement resolves one normalized destination identity. Multiple governed requirements
targeting the same destination refuse unless the schema defines one explicit ordered composition;
the initial surface defines no such composition. Contract-owned task env, mode env, `env_files`,
rendered profile env, service-derived bindings, literals, and defaults that target a governed
destination are validation or admission conflicts unless a named compatibility migration link
defines replacement semantics. That link never fulfills governed delivery.

Ambient inherited process or CI environment with the same key is also never fulfillment. For a
supported late-injection adapter, Ota excludes that destination from ambient and contract-owned
sources before constructing each selected recipient environment, injects it exactly once from the
admitted provider transaction, and scrubs it from every non-recipient segment. If complete
exclusion and scrubbing cannot be proved, execution refuses. No source may silently overwrite
another.

Provider-binding discovery selects one exact authority slot. Zero bindings or more than one
eligible binding refuses. Ota never falls back implicitly between caller, repository, workspace,
Enterprise, or provider sources and never retries a weaker source after a stronger source fails.

A runner capability profile states which delivery destinations, segment boundaries, provider
protocols, cleanup controls, and target platforms the runner can enforce. The profile and adapter
registration are runner-owned or provider-attested, identity-bound evidence; they are never
repository- or caller-authored. They are not policy, authority, or permission and cannot widen
requirements or bindings. Profile, adapter, or attestation substitution refuses.

Authoritative provider and adapter evidence must bind the workload identity, target execution,
transaction challenge, freshness posture, and replay posture supported by that provider. Transport
authentication alone is not workload or provider attestation.

CI stores commonly inject secrets before Ota starts. Those values remain inherited compatibility
inputs unless a concrete mechanism proves late, transaction-bound Ota delivery. An untrusted-fork
or external-contributor event is eligible only when both the repository requirement and the
authority-owned provider binding already name that exact event and workload posture. Policy may
deny or narrow that eligibility; it cannot create it. Otherwise governed secrets are unavailable
and execution refuses.

## Phase-Accurate Delivery

### Pre-provider admission

One canonical evaluator first derives the V12 secret-delivery effect and realization inputs from
the requirement, selected closure, and destination truth. It then derives final
`SecretDeliveryAdmission` from those V12 decisions plus provider binding, capability profile,
secret-delivery policy, target execution, and event posture. Admission runs after every other
ordinary execution, crossing, and sandbox preflight but before provider contact, provisioning,
hydration, services, child creation, repository mutation, or selected work.

Provider contact is an auditable external interaction. A lane not authorized to resolve a
requirement refuses before any provider request.

Dry-run never contacts a provider. It reports `availability: not_checked` and can prove only
deterministic plan and admission truth. It cannot claim materialization, injection, lease freshness,
cleanup, or provider availability. Real execution preserves the same pre-provider plan while adding
runtime evidence; dry-run and real execution do not share availability or injection verdicts.

Phase states are structural and mutually constrained:

| Phase state | Required truth | Forbidden overclaim |
| --- | --- | --- |
| `preflight_refused` | denied/reviewed admission; provider contact and execution not started | materialization, injection, or cleanup success |
| `admitted` | immutable plan set and cleanup authority registered | provider availability or delivered material |
| `materialization_failed` | provider interaction and typed failure; no recipient started | injection success |
| `materialized` | one or more ordered requirements materialized; no injection yet | recipient start or injection success |
| `injection_failed` | materialized requirement set plus typed injection failure | preflight refusal or denial that provider material existed |
| `injected` | exact recipient segments received all assigned requirements | terminal cleanup or execution completion |
| `renewal_failed` | recipient started; renewable material expired or renewal failed | continued governed execution or preflight refusal |
| `terminal` | selected outcome plus complete, partial, or failed cleanup posture | stronger cleanup than observed |

Schemas reject contradictory phase combinations. Partial cleanup is terminal evidence, never
`completed` cleanup and never authorization for another run. Every runtime failure records
`failure_stage`, whether any recipient started, and terminal cleanup/termination posture. A
materialization, injection, or renewal failure cannot be reported as `preflight_refused`.

Each requirement/binding pair carries its own ordered materialization and injection state inside the
invocation transaction. If one pair fails after others materialized or injected, Ota records that
partial state, starts no later dependent segment, terminates any governed recipient boundary already
started where the adapter claims that capability, and cleans up every owned partial resource.

### Runtime delivery transaction

After every preflight admission passes, real execution opens one invocation-scoped, runner-owned
delivery transaction over the ordered plan set and registers terminalization and cleanup authority
before the first provider request. Every materialization, injection, renewal, and cleanup binds to
that transaction. No recipient segment starts until all requirements assigned to it are delivered.
If any requirement fails, Ota blocks every dependent segment and cleans up all partially created
delivery resources before terminalization. Runtime evidence is split into:

- `SecretMaterializationEvidence`: provider resolution, binding reconciliation, static/dynamic
  posture, safe version/lease/handle identity, and freshness or expiry result;
- `SecretInjectionEvidence`: exact transaction, recipient segment, delivery mechanism, and target
  execution binding; and
- `SecretCleanupEvidence`: cancellation/interruption handling and terminal removal or termination
  of adapter-owned handles, files, descriptors, leases, and process/runtime boundaries.

Materialization and injection bind to the same transaction without hashing, fingerprinting, or
serializing the value. A provider version, lease, or handle identity may be carried only when
non-secret; otherwise the adapter uses an opaque provider-attested binding identity.

Injection must consume the exact materialized handle or provider transaction returned by that
resolution. Re-reading a file, environment variable, reference, alias, or provider “latest” value
between materialization and injection is a different observation and refuses or starts a new
transaction. Static delivery records whether the provider supplied a stable version identity;
absence of one remains explicit and policy may require refusal rather than infer freshness.

The transaction pins the admitted requirement set, binding-source evidence, provider bindings,
adapter registrations, capability profiles, recipient graph, target execution, provider handles,
leases, and renewal responses. Any substitution or semantic change refuses and triggers bounded
cleanup; runtime evidence cannot silently replace pre-provider admission truth.

Dynamic values require bounded TTL/expiry and renewal rules. Renewal remains bound to the same plan,
workload, recipient segments, and transaction. Stale, expired, replayed, substituted, reused, or
ambiguously consumed material refuses.

Expiry or renewal failure after a recipient starts must stop or terminate the exact governed
recipient boundary before cleanup when the adapter advertises renewable delivery. An adapter that
cannot enforce that termination cannot support renewable delivery.

Cancellation, interruption, timeout, partial setup, and selected-work failure still finalize the
transaction. Cleanup proves only removal or termination of adapter-owned handles, files,
descriptors, leases, and process/runtime boundaries. It does not prove memory erasure,
provider-side rotation or revocation, absence of prior copying, or absence of exfiltration.

Persistent containers and runtimes require an adapter that proves per-invocation replacement or
removal of every injected destination, handle, lease, and derived runtime binding. Otherwise
governed delivery into that persistent boundary is unsupported.

## Replay, Cache, And Persistence

A receipt, archive, crossing record, or previous delivery transaction is evidence only. It is never
provider authority, current admission, or reusable secret material.

Replay performs current admission, resolves the current authoritative provider binding, and obtains
static or fresh provider material under a new invocation transaction. It never reuses a value,
handle, lease, availability result, materialization result, or injection evidence from an earlier
run.

Secret material and delivery handles must not enter replay inputs, promoted baselines, generated
snapshots, caches, active-execution state, persistent-container metadata, CI projections, workspace
artifacts, or other durable Ota state. Archive verification proves only historical
binding/admission/delivery evidence. Later rotation or revocation does not falsify historical
evidence, but historical evidence cannot satisfy a current run.

## Delivery Adapters

Adapters inject only after every applicable admission succeeds. They must not use generated shell
text, CLI arguments, public files, CI projection YAML, logs, receipts, archives, process titles, or
unrestricted inherited environments.

The first adapter is named at activation from a real design-partner or pressure case and must
support late, transaction-bound delivery. V12.1 cannot activate with hypothetical provider classes.
GitHub or GitLab inherited environment variables remain compatibility evidence unless a concrete
provider mechanism proves stronger delivery.

A local adapter may use a protected source only when it binds the resolved value to a supported
recipient segment without writing it to the repository. A CI adapter must use a provider-native
protected mechanism rather than serializing values into workflow content. A remote adapter remains
unsupported until it proves provider-backed late injection without command text, copied environment
state, or an unbounded inheritance path.

Adapters enforce the admitted segment graph and fail closed when injection, segmentation, cleanup,
or descriptor/environment posture cannot be verified. Ota never falls back to `dotenv`,
repository files, shell expansion, caller environment capture, or unbounded inheritance to make a
governed requirement work.

## Illustrative Contract Boundary

The eventual repository schema contains requirements, not provider bindings:

```yaml
secret_requirements:
  billing_api_token:
    secret_class: authentication_credential
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: BILLING_API_TOKEN
    recipients:
      tasks: [publish_billing]
      dependencies: deny
      hooks: deny
      services: deny
      helpers: deny
      containers: deny
      remote_execution: deny
      proof_observers: deny
      negative_controls: deny
      lifecycle_children: deny
    constraints:
      actor_mode: agent
      environment: production
      execution_mode: native
      target_platform: linux
      runtime_boundary: process
      capability: segmented_process_environment
```

This shape became the accepted provider-neutral step-1 surface after activation and schema review.
It remains declaration-only until later implementation-order steps add separately sourced binding,
admission, and runtime support. The provider mapping is intentionally absent.

## Evidence, Privacy, Receipts, And Archives

Human and JSON output expose only:

- requirement, provider-binding, admission, plan, and transaction identities;
- provider and binding classes at the permitted disclosure level;
- availability, authorization, materialization, injection, cleanup, and authority posture;
- exact selected scope and recipient segments; and
- typed refusal or boundary-failure reasons.

They may expose approved non-secret semantic or projection identities. They never expose secret
values; secret- or provider-reference-derived hashes, lengths, prefixes, or transformed values;
raw provider paths; credentials; descriptors; or reusable handles. Provider paths, secret names,
reference labels, tenancy details, workload metadata, and provider metadata may themselves be
sensitive. Public output uses redacted or opaque identities plus a disclosure class.

Public opaque identities and access-controlled Enterprise detail are projections of the same
canonical private binding evidence. Core verifies their reconciliation without exposing enough
public material to reconstruct or redirect the provider reference. Public output is never a
provider selector, trust root, or binding authority.

Receipts bind the requirement, provider binding and source evidence, plan, admission, transaction,
authority posture, selected scope, effect/realization identities, adapter controls,
materialization, injection, and terminal cleanup.

Full archive re-derivation requires a protected, non-secret canonical binding snapshot plus its
source/verifier evidence. A local binding may embed the complete snapshot only when every field is
safe to disclose. Enterprise Evidence Service may retain the protected attachment under access
control and retention policy.

A public redacted projection can validate schema, signatures or digests, phase links, and
correspondence to the protected evidence identity. It cannot independently re-derive hidden
provider-reference semantics; that dimension is explicitly `redacted_not_independently_rederived`.
Removal, downgrade, or substitution of the protected attachment invalidates full verification.
Public evidence alone is never authority, a replay baseline, or sufficient current admission.

Archive verification preserves the binding's historical weak or strong posture and never
re-resolves current provider truth or upgrades local authority. Current rotation or revocation
affects future admission, not the validity of correctly bounded historical evidence. Historical
evidence never claims current availability and cannot authorize replay. Evidence proves a bounded
delivery boundary, not application correctness, secret correctness, memory erasure, provider
revocation, or absence of exfiltration.

## Shared Enforcement And Assurance

One canonical evaluator drives `run`, `up`, proof commands, dry-run, Doctor contextual findings,
CI projection, sandbox capability output, receipts, archives, V12 effect assurance, and future
refusal canaries. Every consumer uses the same identities, precedence, decision codes, and
pre-provider admission. Runtime consumers add materialization, injection, and cleanup evidence
without reconstructing pre-provider truth.

### V12 effect integration

V12.1 registers one canonical mechanically derived `secret_material_delivery` effect profile.
Before final admission, Core derives a consequence projection from
`SecretRequirementIdentity` rather than hashing the whole requirement. Repositories and policy
cannot author a parallel effect claim.

`EffectIdentity` binds bounded consequence truth without secret/provider value, selected
invocation, task/workflow recipient, or origin: secret class, purpose, normalized delivery
destination and consequence, plus resource/environment bounds that define the same real-world
effect. `EffectRealizationIdentity` and attachment evidence bind the exact requirement identity,
recipient segment, provider binding, adapter/profile, target execution, selected subject, and
invocation origin.

Two differently named lanes delivering the same bounded consequence may share `EffectIdentity`
while retaining distinct realizations. Different recipients never alias as realizations, but
recipient difference alone does not prevent effect equivalence. Effect policy may narrow by
realization, subject, or recipient in addition to effect identity, but cannot provide a requirement,
provider binding, capability, authority, or missing effect truth.

Effect refusal assurance and pressure use this derived profile. A refusal passes only when
attributable to the secret-delivery/effect evaluator for the exact effect and realization. A
task-name, missing-tool, generic agent-safety, or unrelated sandbox refusal is `not_evaluated`.
Equivalent paths not challenged remain `equivalent_execution_paths_not_proved`; opaque or
undeclared delivery remains `unknown` or `contradicted`, never protected.

## OSS And Enterprise Ownership

OSS Core owns:

- provider-neutral requirements and canonical identities;
- the evaluator, monotonic policy, typed refusals, and fail-closed execution;
- non-secret JSON/schema, receipts, archive re-derivation, and disclosure classes;
- adapter and runner-capability interfaces; and
- at least one pressure-proven late-delivery adapter.

Enterprise owns centrally administered provider bindings, tenant/org scope, provider integrations,
workload identity, policy distribution, fleet posture, controlled evidence retention, exceptions,
and management UX. Enterprise consumes Core's canonical model rather than defining a parallel
secret taxonomy.

## First Implementation Boundary

The first slice adds only the provider-neutral `secret_requirements` contract surface, canonical
destination and recipient semantics, domain-separated `SecretRequirementIdentity`, and strict
validation. It must reject secret defaults, ambiguous compatibility ownership, duplicate or
unknown recipients, unsupported destinations, noncanonical environment-variable names, and any
provider selector in `ota.yaml`.

Step 1 does not load a provider binding, evaluate secret-delivery policy, request GitHub OIDC,
contact Google, materialize or inject bytes, alter child environments, emit positive delivery
evidence, or make any task agent-safe. Existing `env` behavior remains compatibility input and
cannot satisfy the new requirement.

Implementation-order step 1 is independently reviewed and committed at Core `9835edfa`: the
additive model, Rust-owned JSON Schema, resolver, domain-separated identity, strict compatibility
checks, contract reference, and adversarial unit/schema tests are present. Step 2 is separately
activated under the amendment above after the connected v1.6.28 proof-assurance hardening completed
its hosted proof and immutable reconciliation. Step 1 completion did not authorize step 2.

Implementation-order step 2 is independently reviewed and committed at Core `9218151b`. The sealed
Core-only model derives separate source-evidence, private-binding, source-projection, and public-
projection identities; requires exact source/binding authority-scope equality; validates the whole
protected snapshot before selection; and returns only sources used by the selected requirement set.
Its resolver refuses missing, duplicate, conflicting, unknown, noncanonical, cross-scope, target-
substituted, and identity-substituted inputs. With no selected requirement it remains not applicable
and does not evaluate unrelated protected snapshots. Public projection excludes requirement linkage,
private binding/source identities, provider references, authority scope, workload identity, trust
roots, and verifier identities. No loader, CLI route, provider adapter, provider contact, secret
delivery, policy/effect admission, execution, receipt, archive, or support path consumes this model.

Implementation-order step 3 is independently reviewed and committed at Core `9ac9274f`. The sealed
Core-only model defines the exact
`google_secret_manager_github_oidc_process_environment_v1` profile, a separately identified
implementation subject, and a protected per-run invocation binding. It keeps stable profile
semantics, implementation/build/target truth, and dynamic invocation/provider truth in three
domain-separated identities. The profile is restricted to `linux/x86_64` GitHub Actions native
execution and a transient selected recipient process tree. Resolution requires the complete
canonical GitHub OIDC claim set, exact GitHub issuer, an audience equal to the selected WIF provider
URL, exact WIF pool and provider resources, project-matched service account and Secret Manager
resource, and one numeric secret version. Independent semantic verification reconstructs retained
inputs rather than trusting a self-consistent resolved record. No loader, registration or lifecycle
identity, CLI route, token request, provider contact, effect/policy admission, delivery, execution,
evidence, or support path consumes this model. At the step-3 completion boundary, step 4 and later
steps remained unauthorized.

Implementation-order step 4 was activated at Core `a7dd07ba` and is independently reviewed and
implemented. The sealed Core-only derivation binds stable secret class, purpose,
destination, environment, runtime-boundary, and capability consequence truth in one
`EffectIdentity`, while attachment and realization identities bind the exact requirement,
recipient, selected closure role, invocation origin, protected binding/source evidence, profile,
implementation subject, invocation binding, process-tree boundary, and target posture. The
canonical effect-policy evaluator independently re-derives every retained secret-delivery input,
requires each realization origin to match exactly one structured retained selected invocation,
binds the realization-to-invocation mapping into execution-graph identity, rejects scope mismatch,
duplicate selected invocations, and duplicate exact realizations, and uses the same fallback,
precedence, set-identity, decision-identity, and semantic-verification finalizer as existing typed
effects. Repeated uses of one attachment remain distinct realizations only when the retained graph
contains each exact invocation rather than collapsing or inventing occurrences by recipient.
No binding loader, policy-authoring surface, CLI route, dry-run plan, command admission, provider
contact, secret delivery, execution, public output, receipt, archive, assurance promotion,
pressure evidence, or support path consumed this model. At the Step 4 completion boundary, Step 5
and later steps remained unauthorized.

Implementation-order step 5 is activated at Core `f455a246`, independently reviewed, and committed
at Core `9fd4b4fb8f991b7dca8c2a2df2c2b6be5e5a0baa`. The sealed evaluator derives the selected-
requirement set from the retained contract and exact task/workflow graph, then consumes the retained
ordered invocation graph, Step 4 derivations, one retained policy snapshot, and the exact canonical
effect-policy decision. It reuses Step 1-4 semantic verifiers, refuses missing,
duplicate, stale, substituted, cross-scope, unsupported, decision-mismatched, or selection-
mismatched truth, preserves selected invocation order, and canonicalizes unordered identity sets.
An empty selection is `not_applicable` and accepts no effect or policy evidence; policy denial is
`refused`; allow or warn is only `structurally_eligible` for a future provider check. The
domain-separated evaluation and dry-run-plan identities retain only non-secret identities and the
four negative runtime states: `availability: not_checked`, `provider_contact: not_attempted`,
`delivery: not_attempted`, and `execution_started: false`. Plan creation independently re-verifies
the retained evaluation rather than trusting a resolved record. No loader, command consumer,
provider interaction, delivery, execution, public output, receipt, archive, assurance, pressure,
or support path consumes the model. At the Step 5 completion boundary, Step 6 and later steps
remained unauthorized.

Implementation-order step 6 was activated at Core `2dd20ab8`, independently reviewed, and committed
at Core `67de2b4d9d6773e8b7ea0506669d4635f5feaf3f`. It retains one command-scoped admission per
invocation across `run`, `up`, runtime and lifecycle proof, Doctor context, CI projection and
provider-checkout re-evaluation, sandbox capability, and task/workflow harness output. Empty
selections preserve existing behavior. Non-empty selections refuse with the public-safe
`secret_delivery_protected_truth_unavailable` projection before setup, hydration, environment
rendering, durable logs, services, proof artifacts, child creation, mutation, or provider contact
while production protected truth is unavailable. Real `ota up` may retain only its generic blocked
execution receipt with `execution_attempted: false`; no positive secret-delivery receipt, archive,
assurance, provider transaction, or delivery exists. Site `4169fe6cf0d790b7f23b5aedb656766af60a37fa`
and Skills `082ce38088e8208b6492334bb09ce19bbd8276af` carry the connected guidance, reconciled by Core
`9d6696f02b939e9366cffe4bd6b7121ede89d822`. At the Step 6 completion boundary, step 7 and every
later step remain unauthorized.

## Implementation Order

1. Add the additive provider-neutral requirement schema, parser model, canonical destinations,
   recipient graph, semantic identity, validation, and adversarial identity/schema tests.
2. Add a separately sourced protected provider-binding model, binding/source identities,
   disclosure classes, and zero/duplicate/substitution refusal without provider contact.
3. Define the exact Google Secret Manager/GitHub OIDC capability profile and implementation-subject
   descriptor under the adapter/profile conformance rules; require the complete protected binding
   tuple named in the activation record and reject every substitution; unsupported targets refuse.
   Do not finalize or register a concrete implementation subject before exact source, build, and
   artifact identities exist.
4. Mechanically derive the `secret_material_delivery` V12 effect, realization inputs, and refusal-
   assurance profile from the canonical requirement, selected closure, destination, recipient
   process-tree, and bounded consequence truth. Integrate the effect-policy decision without
   allowing either policy domain to manufacture authority for the other.
5. Add one shared secret-delivery evaluator and dry-run plan that consumes the retained V12 effect
   decision, reports `availability: not_checked`, and performs no provider interaction.
6. Route `run`, `up`, proof, Doctor context, CI re-evaluation, sandbox capability, and harness
   output through one retained command-scoped admission before provider contact or side effects.
7. Implement the transaction-bound adapter: OIDC request, WIF exchange, exact-version Secret
   Manager access, recipient-only injection, non-recipient exclusion, interruption, and terminal
   cleanup.
8. Add phase-accurate non-secret receipts, protected archive attachments, public redacted
   projections, re-derivation, replay refusal, and tamper/substitution tests.
9. Propagate the shipped operator surface to Core references, changelog, Example, Skill, Site,
   Learn, FAQ, and Glossary as each behavior becomes available; do not publish future semantics as
   current behavior.
10. Run internal adversarial matrices, then immutable real-repository pressure against the named
    adapter and independently reconcile every proved and `not_proved` boundary before closure.

Only one step may advance at a time. Completion of a step does not imply provider support or allow
later steps to consume unimplemented evidence.

## Initial Pressure Bar

The adapter must also satisfy the cross-cutting
[OSS Adapter and Profile Conformance](../adapter-profile-conformance/plan.md) plan before Ota may
call it supported. Registration or successful local execution alone is insufficient.

Activation names one concrete adapter from a real design-partner or pressure case. Pressure uses
synthetic canary material so leak scanning is possible without retaining real values. It proves:

- positive late delivery to one exact supported recipient segment;
- non-recipient segmentation when the adapter can enforce it;
- honest process-tree inheritance for process-environment delivery;
- explicit acknowledgement that arbitrary descendants of the selected recipient process tree may
  inherit process-environment material, while undeclared descendants and raw-shell behavior remain
  unproved rather than represented as excluded helpers;
- hook, service, helper, proof/lifecycle child, and unsupported-segment refusal;
- untrusted-fork/external-contributor refusal unless both requirement and authority-owned binding
  already admit the exact event/workload posture, with policy only narrowing;
- cross-repository and cross-tenant substitution refusal;
- zero/multiple authority-slot bindings and implicit provider fallback refusal;
- collision refusal across governed requirements, literals/defaults, task/mode/profile env,
  `env_files`, and service-derived bindings unless an explicit migration link replaces the
  contract-owned source;
- ambient/inherited environment exclusion plus non-recipient scrubbing, with refusal when complete
  scrubbing cannot be proved;
- unavailable, stale, expired, replayed, mismatched, and duplicate binding refusal;
- replay resolving the current authority binding under a new transaction without reusing historical
  material, handles, leases, availability, or delivery evidence;
- dynamic-lease expiry and renewal where the first adapter uses dynamic material;
- materialization success before injection, injection failure after materialization, and renewal
  failure after recipient start, each with exact failure stage and cleanup/termination evidence;
- multi-requirement partial materialization/injection proving no later dependent segment starts
  after failure;
- interruption, cancellation, partial-setup, and terminal cleanup behavior;
- persistent-runtime refusal unless the adapter proves per-invocation replacement and removal;
- scanning command output, logs, process arguments/titles where observable, repository files,
  generated CI projection, replay/baseline/snapshot/cache state, active-execution state,
  persistent-runtime metadata, workspace artifacts, receipts, archives, and public JSON for canary
  leakage;
- unsupported remote delivery refusal before provider contact or command construction;
- two differently named lanes resolving the same derived effect through the same policy;
- effect identities splitting when destination or bounded consequence differs, while recipient or
  subject differences retain distinct realizations without necessarily splitting effect identity;
- protected archive attachment removal/downgrade/substitution refusal plus honest
  `redacted_not_independently_rederived` public posture; and
- an omitted or undeclared path remaining `unknown` or `not_proved`, never protected.

Each result includes an uncovered-material-behavior inventory and keeps claims bounded to the
selected adapter, target, repository, recipient segments, and observed execution.

## Non-Goals

V12.1 does not:

- make Ota a vault, certificate authority, identity provider, or rotation service;
- infer provider bindings from matching names or environment variables;
- promise exact-process isolation from process-environment delivery;
- govern raw shell, arbitrary provider access, or exfiltration after code receives a value;
- treat cleanup as memory erasure or provider revocation;
- make inherited GitHub/GitLab environment into late-delivery proof;
- ship every provider, remote backend, or Enterprise control plane; or
- propagate Site, Examples, Skills, or schemas before behavior ships.

## Definition Of Done

V12.1 is complete only when all of the following are measurable and independently reviewed:

- **Contract and validation:** one additive versioned requirement schema exists; provider bindings
  remain separately sourced; ambiguous ownership, unknown/duplicate selectors, contradictory
  bindings, destination collisions, compatibility fulfillment, and secret defaults refuse with
  stable codes; normalized destinations participate in requirement identity.
- **Identity and policy:** requirement, binding, plan, admission, transaction, and evidence
  identities have adversarial mutation tests; policy cannot manufacture authority, recipients,
  capabilities, event/workload eligibility, or provider support; binding authority, trust roots,
  adapter registration, and capability profiles are identity-bound and substitution-safe; V12
  `allow | warn | deny` and secret-delivery `allow | review | deny` remain distinct; any effect
  `deny` or secret-delivery `deny`/`review` refuses, effect `warn` is retained, and no domain
  authorizes another.
- **Evaluator and commands:** Doctor, dry-run, `run`, `up`, proof commands, CI projection, and
  capability output consume one evaluator; failed admission refuses before provider contact, and
  dry-run reports `availability: not_checked`; `deny > review > allow` applies only within
  secret-delivery policy; cross-domain aggregation preserves V12 warnings and refuses for any
  effect deny or secret-delivery deny/review; phase schemas reject contradictory preflight,
  materialized, injection-failure, renewal-failure, partial-cleanup, and terminal evidence.
- **Runtime evidence:** the named adapter binds the ordered multi-requirement plan set,
  materialization, injection, and renewal to one invocation transaction; registers cleanup
  authority before provider contact; gates each recipient segment on its complete requirement set;
  records per-requirement partial states; starts no later dependent segment after failure; terminates
  an already-started governed recipient on renewal failure; rejects identity substitution and
  re-resolution TOCTOU; finalizes all partial resources; and never hashes or emits material.
- **JSON, receipts, and archives:** schemas require phase-accurate fields and disclosure classes;
  blocked and successful evidence validates; full re-derivation requires the protected canonical
  binding snapshot and source/verifier evidence; redacted public evidence reports the hidden
  dimension as not independently re-derived; attachment removal/downgrade/substitution refuses;
  public output remains non-redirectable and insufficient for current admission.
- **Compatibility:** existing `env` behavior retains documented semantics and cannot be mistaken for
  governed delivery assurance; contract-owned collisions require refusal or explicit migration
  replacement; recipient construction excludes ambient values before one admitted injection and
  scrubs non-recipient segments or refuses; unsupported remote forwarding remains fail-closed.
- **Replay and persistence:** every replay performs current admission and provider resolution under
  a new transaction; no value/handle/lease or delivery result enters baseline, snapshot, cache,
  projection, active-execution, workspace, or persistent-runtime state; unsupported persistent
  delivery refuses.
- **Effect integration:** one mechanically derived `secret_material_delivery` profile drives V12
  policy and assurance before final admission; equal consequences may share identity across lanes
  and recipients, while destination/consequence bounds prevent false aliasing; realization binds
  requirement, recipient, provider, adapter, execution, subject, and origin.
- **Tests:** focused unit, integration, schema, archive, leak-canary, TOCTOU, substitution,
  destination-collision, current-binding replay, persistent-runtime, effect-non-aliasing,
  materialized-before-injection, post-materialization injection failure, in-flight renewal failure,
  multi-requirement partial failure, interruption, and negative-path regressions pass with stable
  exit semantics.
- **Documentation and propagation:** Core command/contract/JSON/receipt references, changelog,
  canonical example, canonical Skill, and Site reference explain when and why to use the shipped
  surface. These surfaces are updated only when behavior ships.
- **Pressure evidence:** the named adapter and adversarial matrix pass on real repositories, with
  bounded claims and uncovered-material-behavior inventories.

Additional providers, remote delivery, and Enterprise management remain unsupported until each has
its own adapter, pressure evidence, and bounded claim.
