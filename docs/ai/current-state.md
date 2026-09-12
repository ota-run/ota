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

# Current Ota Development State

Update this file at a completed implementation, pressure, release, or handoff boundary. Replace
stale state; do not turn it into an activity log. Durable decisions belong in `docs/adr/` and
durable agent workflow belongs in the canonical Ota skill.

## Active Work

- branch: `1.6.28-implementation`
- released baseline: `v1.6.27`
- development package identity: `v1.6.28`; this is an unreleased implementation build, not a
  release or support claim.
- active version: V12.1 secret-delivery governance. Activated on 2026-09-02 after the released V12
  closure and feasibility review of PythiaLabs' credentialed CAEP boundary. The named first adapter
  is `google_secret_manager_github_oidc_process_environment_v1`, initially limited to a
  `linux/x86_64` GitHub Actions native-process recipient using GitHub OIDC, Google Workload Identity
  Federation, and one exact Google Secret Manager version. Activation authorizes implementation-order
  step 1 and, under separate amendments, implementation-order steps 2, 3, 4, 5, and 6: the
  provider-neutral requirement schema, canonical recipient/destination model,
  `SecretRequirementIdentity`, and the Core-owned provider-binding domain model with pure
  fail-closed resolution, followed by the exact first-adapter profile and implementation-subject
  descriptor model, the crate-private `secret_material_delivery` effect derivation foundation, the
  sealed provider-free evaluator/dry-run-plan model, and one retained command-scoped admission per
  invocation. Step 6 was independently reviewed and activated at Core `2dd20ab8`; its implementation
  is independently reviewed and committed at Core `67de2b4d`, with immutable consumer reconciliation
  recorded at Core `9d6696f0`.
  No provider binding loader, concrete implementation registration, provider
  contact, materialization, injection, execution authority, positive evidence, or support claim is
  implemented yet. V12.2 and later versions remain inactive.
- V12.1 implementation-order step 1 is independently reviewed and committed. `ota.yaml` now has an
  additive provider-neutral `secret_requirements` catalog with one initial
  `authentication_credential` / `external_api_authentication` vocabulary, canonical
  `process_environment` destinations, explicit task/workflow recipients, deny-only propagation
  edges, requested execution constraints, and domain-separated `SecretRequirementIdentity`.
  Validation requires `metadata.ota.minimum_version` of at least `1.6.28` and rejects provider
  selectors, secret defaults, destination collisions, compatibility ownership conflicts, unknown
  or noncanonical recipients, and noncanonical environment-variable names. The Site contract
  reference, Glossary, FAQ, Learn lesson, canonical Skill, and both installed mirrors carry the
  current fail-closed admission boundary; Examples remain unchanged because no usable delivery flow
  exists. Step 6 command admission now consumes exact selected recipients and refuses before setup
  or execution while protected production binding truth is unavailable. No provider binding loader
  exists and no secret bytes are loaded or delivered. Step 2 was explicitly activated under the V12.1 plan amendment
  after the connected proof-assurance hardening and immutable Linux/macOS evidence were inspected.
  It authorizes only the Core-owned provider-binding domain model, private canonical identities,
  opaque public disclosure, and pure fail-closed resolution of authority-sourced protected
  snapshots. It does not authorize a binding loader, CLI route, provider adapter, provider contact,
  OIDC/WIF, materialization, injection, policy/effect admission, execution, receipts, archives, or
  support claims.
- V12.1 implementation-order step 2 is independently reviewed and committed at Core `9218151b`.
  The sealed Core-only model derives domain-separated protected source-evidence and private-binding
  identities from exact requirement, provider reference, authority scope, source evidence, adapter,
  lifecycle, and target truth. The pure resolver validates the complete protected snapshot before
  selection, requires exact source/binding scope and requirement-target agreement, refuses zero,
  duplicate, conflicting, unknown, noncanonical, or substituted inputs without fallback, and
  returns only selected bindings and their sources. Its fail-closed public projection omits
  requirement linkage and every private source/reference input; changing private references does
  not create a public correlation oracle. No loader, CLI, adapter, OIDC/WIF, provider contact,
  materialization, injection, policy/effect admission, execution, receipt, archive, or support path
  consumes the model. Site, Skills, Examples, Learn, FAQ, and Glossary remain unchanged because
  this sealed internal foundation creates no authoring or operator workflow, command, output, or
  public vocabulary.
- V12.1 implementation-order step 3 is explicitly activated under a separate plan amendment. It
  authorizes only the crate-private capability-profile and implementation-subject descriptor model
  for `google_secret_manager_github_oidc_process_environment_v1`, deterministic identities, the
  complete protected GitHub OIDC/WIF/Google Secret Manager binding tuple, exact `linux/x86_64`
  GitHub Actions native-process target posture, unsupported-target refusal, and adversarial
  substitution tests. Stable profile semantics, implementation/build/target truth, and exact
  per-run values remain separated through `profile_semantic_identity`,
  `implementation_subject_identity`, and protected `SecretDeliveryInvocationBindingIdentity`.
  A concrete implementation subject cannot be finalized or registered until exact source, build,
  and artifact identities exist; registration and lifecycle identities remain uncreated. No loader,
  registry installation, lifecycle promotion, CLI route,
  token request, provider contact, network access, effect/policy admission, materialization,
  injection, execution, receipt, archive, assurance, conformance result, pressure evidence, or
  support claim is authorized, and the generic cross-cutting adapter/profile conformance plan
  remains inactive. The activated model is independently reviewed and committed at Core
  `9ac9274f`. It uses domain-separated profile, implementation-subject, and protected invocation-
  binding identities; requires the complete canonical GitHub claim set and exact issuer; binds the
  audience to the selected WIF provider URL; reconciles exact WIF, service-account, project, Secret
  Manager, requirement, and provider-binding truth; and independently re-derives retained input
  before accepting a resolved record. Ten adversarial tests cover normalization, omission,
  duplication, target widening, tuple substitution, provider-specific resource grammars, cross-
  resource aliases, and self-consistent resolved forgeries. At that completion boundary, Step 4
  and later steps remained unauthorized. This internal model creates no Site, Skill, Example,
  Learn, FAQ, Glossary,
  contract-schema, or public-JSON change because it has no loader, command, output, or operator
  workflow. Step 4 is separately activated at Core `a7dd07ba`. It authorizes only crate-private
  derivation of the `secret_material_delivery` V12 effect,
  realization inputs, internal refusal-assurance eligibility, and the narrow canonical evaluator
  extension needed to consume the derived effect. It keeps stable bounded consequence truth in
  `EffectIdentity`; binds exact requirement, recipient, closure role, invocation origin, protected
  binding/source, profile, implementation subject, and target truth in the realization identity;
  and requires structural authority reconciliation before policy evaluation. The independently
  reviewed implementation derives separate effect,
  attachment, realization, and internal refusal-attribution identities; independently reconstructs
  every retained derivation before policy evaluation; requires each realization to match exactly
  one retained selected invocation; binds that realization-to-invocation mapping into the
  execution-graph identity; preserves repeated attachment roles as distinct realizations only when
  the graph retains each exact occurrence; and reuses the canonical effect-policy fallback,
  precedence, set, decision, and verification finalizer. At the Step 4 completion boundary, Step 5
  and later steps remained unauthorized, and no loader,
  policy-authoring surface, CLI route, provider contact, materialization, injection, execution,
  public output, receipt, archive, assurance promotion, pressure evidence, or support claim is
  introduced. Site, Skills, Examples, Learn, FAQ, Glossary, contract schema, and public JSON remain
  unchanged because this internal foundation has no operator surface. Step 5 is separately
  activated under the V12.1 plan amendment after Step 4 was independently reviewed and committed at
  Core `f1178fe3`. It authorizes only one crate-private evaluator and dry-run plan consuming retained
  Step 1-4 truth. The implementation was independently reviewed and committed at Core
  `9fd4b4fb8f991b7dca8c2a2df2c2b6be5e5a0baa`.
  It derives the selected-requirement set from the retained contract and exact task/workflow graph,
  reuses the Step 1-4 semantic verifiers, preserves selected invocation order, canonicalizes
  unordered identity sets, and independently reconstructs both evaluation and plan identities.
  A genuinely empty selection remains `not_applicable`, policy denial
  refuses, and allow/warn is only structural eligibility for a future provider check. Every plan
  retains
  `availability: not_checked`, `provider_contact: not_attempted`, `delivery: not_attempted`, and
  `execution_started: false`; it is never runnable, available, delivered, supported, or assured.
  No loader, command consumer, provider interaction, execution, public output, or evidence path was
  authorized at the Step 5 completion boundary. Step 6 was independently reviewed and activated at
  Core `2dd20ab8`; it authorizes one retained command-scoped admission per invocation across
  `run`, `up`, proof, Doctor context, sandbox capability, and harness output. CI projection carries
  only a bounded public expectation; provider-checkout CI independently re-derives admission from
  its exact checkout and reconciles the public projection rather than inheriting render-host truth.
  The amendment preserves existing behavior for empty selections; refuses non-empty selections
  before setup, hydration, environment rendering, durable logs, services, proof artifacts, child
  creation, mutation, or provider contact when complete Step 1-5 truth is unavailable; and permits
  only a separately derived public-safe projection that excludes private decision, realization,
  evaluation, plan, binding, source, subject, and invocation identities. Every production binding
  loader, provider transaction, delivery, execution, positive secret-delivery receipt,
  secret-delivery archive, assurance, pressure, and support path remains unauthorized. Real
  `ota up` retains only its existing generic blocked execution receipt with
  `execution_attempted: false`. The committed implementation wires this one retained admission
  through `run`, `up`, runtime and lifecycle proof, Doctor context, CI projection/re-evaluation,
  sandbox capability, and task/workflow harness output. Empty selections preserve existing behavior;
  selected recipients emit only a domain-separated public negative projection with protected
  identities excluded. Focused command and schema regressions prove refusal before fixture setup,
  environment rendering, durable logs, proof artifacts, and command execution. Site `4169fe6` and
  Skills `082ce38` carry the connected public and agent guidance; Core `9d6696f0` pins both immutable
  consumer revisions. Standalone Examples remain unchanged because Step 6 has no executable delivery
  example. At this completion boundary, Step 7 and later steps remained unauthorized.
- V12.1 Step 7 was independently reviewed and activated at Core `ad14ae8b`. It authorizes only the
  first real provider transaction on a `linux/x86_64` GitHub Actions self-hosted protected runner:
  a new protected-runner profile; an administrator-installed launcher that starts unprivileged Ota
  with retained authority descriptors, no privilege-escalation posture, and cgroup-v2 process
  ownership; a caller-independent TLS transport; byte-exact GitHub OIDC, Google WIF, service-account,
  and numeric Secret Manager version requests; selected native process-tree environment injection;
  interruption; and terminal cleanup observation. The proposal proves only intra-invocation store
  consistency and leaves independent-administrator rollback, privileged escape, exfiltration, and
  memory erasure unproved. It separately binds a protected-self-hosted-runner-derived OIDC request-
  endpoint profile and exact JWT issuer. ARM64 discovery run `33919324208` confirmed the regional
  request-service distinction without provider contact, but it does not satisfy the required
  `linux/x86_64` protected-launcher fixture; provider contact cannot be implemented until that exact
  compatibility fixture exists. Step 7 keeps GitHub-hosted and repository-provisioned runners,
  other targets, positive receipts/archives/assurance, adapter support, Step 8, and V12.2 inactive.
- Step 7 endpoint-compatibility work is in progress. Core `777c42ab` commits a no-checkout
  Linux/X64 discovery workflow that binds the live OIDC request URL shape to the protected runner
  service posture and administrator installation identity. A separate no-`id-token` job uses
  pinned Actions Toolkit code with a loopback fake capability to prove it calls the supplied URL
  rather than the JWT issuer; the protected job reads that retained probe and observes the live
  endpoint shape.
  OIDC and provider contact are not instrumented or claimed.
  Successful Core run [34153231585](https://github.com/ota-run/ota/actions/runs/34153231585), at
  exact Core `f921209561b26f38cdb74c5f20f71e0b6734ae0d`, separately proves the retained endpoint
  shape on self-hosted `linux/x64` Runner `2.337.0` and that the pinned Toolkit used its supplied
  loopback URL rather than the JWT issuer. Its public evidence retains
  `capability_identity: null`, `capability_derivation: not_attempted`, and
  `provider_contact_observation: not_instrumented`; it retains neither a bearer nor a protected
  request URL value. It does not derive protected-launcher capability, make an OIDC request, or
  contact a provider.
  The earlier Core discovery run `34137753776` was cancelled from its queue and remains no
  compatibility evidence. The subsequent independently reviewed Launcher witness
  [34159892077](https://github.com/ota-run/authority-launcher/actions/runs/34159892077), at exact
  Launcher `79f2b5e81a7454c7c56394520b27a5f4f3fb9f48`, Core
  `f921209561b26f38cdb74c5f20f71e0b6734ae0d`, and Protocol
  `ae3c8e99164c2d1db7f387f061f875272015bb36`, proves one bounded protected-runner governed
  invocation: installed client identity/help reconciliation, completed selected execution, terminal
  child reap, scope removal, empty-or-absent cgroup, active-slot removal, and one valid protected
  receipt archive with zero invalid archives. The exact public hosted payload is retained by
  `authority-launcher` commit `a2cdd23b82472b4c4b7ae07ae28356f0d7338d71` at
  `docs/pressure/retained-artifacts/systemd-v3-independently-administered-34159892077.zip`
  (`sha256:0b15512fe736bee4b35a2dd205a5e9e3d68a9af496ee86f812c364b4f322258f`). Current runner-group
  configuration was independently checked to allow exactly the two documented repositories and
  four branch-pinned workflows, but that mutable configuration is not run-bound artifact evidence.
  A later independently reviewed Launcher witness,
  [34241049867](https://github.com/ota-run/authority-launcher/actions/runs/34241049867) job
  `102111003771`, at exact Launcher `8ca4763c1e5c6ef5ac06c2be5b778c49344c5030`, Core
  `f921209561b26f38cdb74c5f20f71e0b6734ae0d`, and Protocol
  `e0af492ba8a6fbe01e805c79762909c9cda28198`, proves the run reconciled the replay directory
  through the provisioning-owned exact fresh-state inventory, the consumer workflow's exact
  inventory check, and the protected Launcher's effective systemd `ReadWritePaths` verification.
  The bounded production client then completed one governed invocation with child reap, scope
  removal, empty-or-absent cgroup, active-slot removal, and one valid protected receipt archive
  with zero invalid archives. The exact eight-file public artifact is retained by `authority-launcher`
  commit `a272fbe863715f04996340591a9ae29dc80ccedc` at
  `docs/pressure/retained-artifacts/systemd-v3-independently-administered-34241049867.zip`
  (`sha256:090f3ac9fa2b516370b8d844520d4759bce877fae7b0f6a541bd5e23539d6496`).
  This later witness still does not prove production capability-observation routing, exact
  production context ownership, accepted-session provenance, Core capability-projection
  reconciliation, provider contact, real OIDC exchange, secret materialization or delivery,
  Step 8, V12.2, or general agent/repository governance.
  Ubuntu 26 subsequently exposed that its canonical `sudo-rs` executable refuses policy listing
  from the already-root protected Launcher when systemd sets `NoNewPrivileges=yes`. Launcher
  revision `22f17eff3e005bb4544d583d0743721835539d0a` keeps `NoNewPrivileges=no` only for that
  root-owned observer while independently requiring and revalidating `NoNewPrivileges=1`, empty
  capabilities, and exact principal boundaries for the runner and selected execution process.
  After a fresh immutable reprovision, repository-owned run
  [34356604479](https://github.com/ota-run/authority-launcher/actions/runs/34356604479), job
  `102482773723`, completed one governed invocation at exact Launcher `22f17eff3e005bb4544d583d0743721835539d0a`,
  Core `104c1345117a39f899fdc6a48bf8cf689ffbe9b6`, and Protocol
  `58526f3f29299873e345352963e30b8a1677044f`. Its terminal records exit `0`, child reap, scope
  removal, empty-or-absent cgroup, and active-slot removal; protected history records one valid
  receipt archive with zero invalid archives and the exact matching archive identity. The exact
  eight-file public artifact is retained by `authority-launcher` commit
  `ab2f7ca781f3e9cf617c513cf89ceb8ece98da56` at
  `docs/pressure/retained-artifacts/systemd-v3-independently-administered-34356604479.zip`
  (`sha256:33e749d46e1ff7d0904196373c8538deda66cc34eacefb3f86ffe3b0dbcfb725`). This witness proves
  neither protected capability-observation production routing nor OIDC exchange, provider contact,
  secret materialization or delivery, Step 8, V12.2, or general agent/repository governance.
  The earlier Launcher witnesses do not derive `ProtectedLauncherCapabilityIdentity`, execute the
  Core endpoint workflow, contact OIDC or a provider, materialize or deliver a secret, establish
  provider compatibility, or prove general agent/repository governance. A later Root boundary proof
  [34283728831](https://github.com/ota-run/authority-launcher/actions/runs/34283728831), at exact
  Launcher `fa842cdeb2ede58ff726c3a60dd33eaa47e1a7bd`, ran exactly one Linux privileged
  `retained_observation_derives_and_rejects_live_substitution` regression under the requested root
  systemd scope. It proves bounded retained protected-capability derivation plus refusal of a valid
  substituted expected Launcher-invocation identity before replay reservation, capability derivation,
  or Attestor signing. It does not prove production Core-to-Launcher observation transport, public
  projection reconciliation, production observation-service or accepted-session provenance, OIDC,
  provider contact, secret materialization or delivery, positive provider evidence, or general
  governance. The endpoint profile and semantic observation verifier remain crate-private. Provider
  contact stays blocked on the exact protected Linux/X64 compatibility gate: the raw capability
  identity must remain in the protected launcher transaction, while the workflow must verify one
  exact signed, closed public observation projection against a fresh canonical one-use public
  challenge and refuse replay, substitution, duplication, staleness, or signature/schema mismatch.
  Core must load the matching projection verification key only from the fixed administrator-owned
  verifier record reconciled with protected installation evidence; it cannot trust a workflow- or
  projection-supplied key. Root launcher state owns atomic challenge reservation and consumption;
  Core owns its expected challenge for the exact workflow invocation. The fixed local request also
  binds the expected protected Launcher invocation identity; the Launcher must recompute it from its
  accepted invocation and refuse mismatch before replay reservation, capability derivation, or
  Attestor signing. That identity remains protected and is excluded from the public projection and
  workflow-visible output. The Step 7 batch implements that feature-gated service route:
  it loads the independently installed `ProtectedLauncherAuthorityContextV1`, prepares the exact
  stopped Ota child and transient cgroup, retains the accepted Unix session and authority stores,
  delegates only public-projection signing to the protected Attestor, confirms cleanup, and returns
  only the signed public projection. Core issues the fresh challenge, sends the closed
  invocation-bound probe over the fixed root socket, reloads the verifier and its public
  installation binding, and reconciles the exact response. The endpoint observation now binds the
  verified public projection identity rather than the raw capability identity. Launcher first
  committed the route at `2d7d8a1e82753943059e4999bb6ec3a7a569b896`; Core committed reconciliation
  at `e5f5080bb0081f47077d0e2e73d90a8f3d7fa392` and extended only the hosted compilation timeout at
  `5db2564c538e1c66d5f2bb8a854e788557c5e4ac`. Hosted runs `34379005208` and `34379290838` exposed,
  respectively, a missing runner toolchain and the original five-minute timeout. Run `34381913282`
  then compiled and executed the exact transaction but refused with `LocalBoundaryUnavailable`
  because the canonical provisioner had not installed `/etc/ota/secret-delivery`. Launcher repairs
  that ownership boundary at `ec04db2dac7ad73e13dc38ec34cf78edf76cc8ee`: it installs fixed root-owned
  empty structural verifier and binding snapshots, includes the directory in fresh-state
  reconciliation, and grants no provider authority. The Core gate now binds that exact Launcher
  revision, Protocol `58526f3f29299873e345352963e30b8a1677044f`, and the run's exact Core
  `github.sha`. Linux runtime proof remains open; provider contact stays blocked until the exact
  committed Linux/X64 workflow succeeds and its substitution, replay, staleness, verifier,
  signature, target, revision, and cleanup controls are retained. No OIDC request, provider
  contact, materialization, delivery, Step 8, or V12.2 capability is activated.
  Runs `34386901632` and `34390181756` subsequently executed the exact transaction after canonical
  authority-store provisioning and localized the remaining refusal to retained authority-context
  acquisition. Separate systemd reproduction on the protected host established that the immutable
  V3 combination `ProtectProc=invisible` plus `ProcSubset=pid` removes the required
  `/proc/sys/kernel/random/boot_id`. A bind-only exception also refused because systemd applied the
  proc subset before resolving the bind source. Widening to `ProcSubset=all` is rejected because the
  forked selected child would inherit the Launcher's mount namespace.
  Step 7's V4 correction was independently reviewed and activated at Core
  `7a86c9313921ba012fb966c527e9c901a2b18e66`. Protocol
  `d16947b87d84e66a4164a275c703b47fce6ded20` preserves V3 historically and adds the immutable V4
  profile, exact inherited-listener and boot-descriptor roles, profile substitution refusal, and
  the V4 public projection class. Launcher `d690b8beb0c775a9d4ad7c944a150de895b1af67` retains both
  proc restrictions while systemd PID 1 passes the exact boot-ID file as one named read-only
  `OpenFile=` descriptor; the protected history and broker units retain their explicit
  `ProcSubset=pid`, and the Attestor remains unchanged. Root Boundary run
  [34417605017](https://github.com/ota-run/authority-launcher/actions/runs/34417605017) passed at that
  exact Launcher revision, including the privileged retained capability derivation. The first
  independently administered VPS run
  [34418027937](https://github.com/ota-run/authority-launcher/actions/runs/34418027937) then refused
  before authorization because Core still required V3; it is failure localization, not V4
  compatibility evidence. Core migration `5e4b2438422ae7e0f54c063ee6fbcfb585c2965e` pins the exact
  Protocol revision, requires V4 for live broker and capability-observation admission, preserves
  exact historical V3 archive re-verification without making V3 loadable for live admission, and
  binds the endpoint gate to the V4 public class. After an exact VPS reprovision, Launcher run
  [34422820241](https://github.com/ota-run/authority-launcher/actions/runs/34422820241)
  completed the governed invocation at exact Launcher
  `d690b8beb0c775a9d4ad7c944a150de895b1af67` and Core `5e4b2438`; Core endpoint run
  [34422827427](https://github.com/ota-run/ota/actions/runs/34422827427) then refused before
  capability derivation because the replay opener prohibited systemd's administrator-owned mount
  transition at `/var/lib/ota/authority-launcher`. Launcher
  `dac375dc323d8aa30be2e23db033b89647d5e331` admitted that one observed transition, but immutable
  Root Boundary run
  [34425445450](https://github.com/ota-run/authority-launcher/actions/runs/34425445450) executed the
  exact new test once and localized another distribution-specific mount transition earlier in the
  same fixed path. It is failed portability evidence, not compatibility proof. Launcher
  `017d866bf9aab8b193ef0ce0515acc95545f3399` now walks only the fixed absolute replay path with
  raw lexical alias refusal, no-symlink and no-magic-link resolution, root ownership and
  non-writability checks on every component, and exact `root:root 0700` final-directory validation;
  all replay record operations restore no-mount-crossing descriptor-relative resolution beneath
  the retained final descriptor. Both focused tests passed directly on the protected Linux/X64
  host. Root Boundary run
  [34431466395](https://github.com/ota-run/authority-launcher/actions/runs/34431466395), job
  `102727695360`, then passed at that exact Launcher revision, including the privileged retained
  capability derivation and production replay-store checks. After fresh provisioning at exact Core
  `1dee11e4bfaae4a335228d1fb76573dde9f65353`, Launcher `017d866b`, and Protocol `d16947b8`, Core
  endpoint run [34433055859](https://github.com/ota-run/ota/actions/runs/34433055859) completed the
  protected observation transaction and emitted one valid signed projection, but the workflow
  rejected it because Rust's test harness prefixed the marker line with the test name. The Core
  workflow repair at `f2db350fee630ce2ad1701bd9b5272933d6c7c1e` extracts exactly one marker
  from either harness placement and self-tests zero and duplicate refusal. Exact run
  [34439988695](https://github.com/ota-run/ota/actions/runs/34439988695) passed when GitHub supplied
  the retained shard-`1` profile, while prior exact run
  [34434534789](https://github.com/ota-run/ota/actions/runs/34434534789) completed capability
  observation and then rejected the live endpoint as outside that profile. Root-only mutable Runner
  diagnostics attributed the mismatch to service shard `3`; that operator observation is not
  immutable hosted artifact evidence. The two outcomes nevertheless expose that freezing one
  service shard is not a stable compatibility boundary. Core now admits only the East US service
  family with a canonical positive shard, still binds the exact observed host into each observation,
  and refuses zero, padded, overflowing, alternate-region, suffix, scheme, port, and authority
  substitutions. Exact Core run
  [34443349694](https://github.com/ota-run/ota/actions/runs/34443349694), protected job
  `102762828509`, then passed at `a93d65650e6d74167515fedc43a62ea9cc657489` against service shard
  `3`, with signed V4 capability derivation verified and the bounded endpoint artifact published.
  Its exact endpoint and no-token Toolkit ZIPs are retained under
  `docs/pressure/retained-artifacts/` with SHA-256 values
  `0abd5bfe6565bba8ac768205f05c9675db8c62167fa588cd44fad48300e69571` and
  `d525e5284463e96567219a4c5a6e05fd7ca056adf4cc9ff2506b1ae5c21acd26`.
  Launcher run
  [34443805276](https://github.com/ota-run/authority-launcher/actions/runs/34443805276), job
  `102764136034`, separately completed one governed invocation against the same installation with
  exit `0`, all four terminal cleanup conditions, and one valid protected receipt archive with zero
  invalid archives. Launcher `bf4e6d994d964aef0f5bcadec71d1b5cd53f63e6` durably retains that
  exact eight-file ZIP. These runs close the bounded protected-capability and endpoint-compatibility
  gate only. No real GitHub OIDC request, provider contact, materialization, delivery, positive
  provider evidence, Step 8, or V12.2 capability is activated.
  The signed observation service remains compatibility evidence because it creates a separate
  refused probe child and cgroup; it cannot authorize a later provider-requesting child. The
  independently reviewed private authority-snapshot bridge now closes the Core-owned provider-free
  reconstruction boundary. Protocol `d1d1fd4` supplies the record-only prerequisite. Core
  `15f59d87` adds request/response reconciliation, `8c22a18a` verifies the signed protected
  snapshot and retained bytes, and `f6d4633e` reconstructs one exact Step 1-6 transaction candidate
  from the challenge-backed invocation context and canonical selected `RunPlan`. Core independently
  re-derives the selected roots, binding/source, profile, implementation subject, provider tuple,
  effects, protected policy authority, evaluation, and dry-run plan. It rejects substituted
  authority, context, workflow identity, graph semantics, or provider truth and selected roots
  outside the exact native Linux transient structured-command boundary. The batch passed focused
  release compilation and regression suites, first-party synchronization, and independent review.
  No Site, Skills, Examples, Learn, FAQ, Glossary, contract-schema, or public-JSON propagation is
  required because the bridge remains crate-private and provider-free.
  The next Step 7 proof gate is Protocol's additive snapshot-bound V2 transaction binding, followed
  by Launcher's same-child/session relay, Core's one-use V2 reconciliation, and an exact protected
  Linux/X64 service-path run. Until that complete sequence passes, no real GitHub OIDC request,
  Google provider contact, materialization, injection, positive evidence, Step 8, or V12.2
  capability is activated.
- Vinicius' independent v1.6.27 source review confirmed that negative-control projection
  reconciliation is live on runtime proof, emitted-archive verification, and Doctor archive
  loading. It also exposed a narrower adversarial-test gap: the existing digest mutation was
  malformed rather than a valid sibling attestation. Core now retains the malformed case as
  structural coverage and adds a non-default test-only live-transaction fault boundary. Two
  declared controls execute through normal Core derivation in one proof run; substituting the
  sibling's valid digest after projection creation refuses at the production reconciler before
  terminal proof output, receipt, or archive publication. The shared Core and standalone Examples
  runtime-proof fixture now uses bounded dependency-readiness retries so the same transaction is
  reliable on hosted Linux and macOS without weakening failure semantics; the standalone copy is
  pinned at `ota-run/examples@6ab6635237aba8172812406499186a7fbc8a16cd`. Release Gate run
  `33737308315` then proved the reference-script retry was not the complete hosted fix: Windows
  passed, Ubuntu's fault step was cancelled, and macOS reached reconciliation without a selected
  projection after its detached fixture services failed to retain the green obligation. The
  feature-gated transaction regression now owns both fixture servers as retained test children,
  verifies their readiness before invoking Ota, keeps Ota's selected runtime child and production
  reconciliation path intact, and performs terminal cleanup. Release Gate
  [run 33744200940](https://github.com/ota-run/ota/actions/runs/33744200940) passed at exact Core
  `876680f074777aada0b3910a62aab9b245b34af7`: Ubuntu job `100612831644` and macOS job
  `100612831722` both passed the dedicated live substitution boundary, and the complete Windows,
  Ubuntu, and macOS gates succeeded. The hosted control proves that a real canonical sibling
  attestation digest substituted after projection creation is refused by production reconciliation
  before terminal proof output, receipt, or archive publication. It does not prove provider
  behavior, external mutation prevention, or positive proof assurance. Site, Skills, Learn, FAQ,
  Glossary, and standalone Examples remain unchanged by this test-harness correction because it
  changes no operator behavior, public vocabulary, or shared reference content.
- Discord contract pressure activated the aggregate mode-eligibility follow-on as a bounded V12.1
  correctness repair. Core now derives aggregate mode admission, task-discovery availability,
  agent-safety closure, task/workflow replay-input admission, managed CI projection, and
  task/workflow Doctor prerequisite selection from the existing backend-selected runner plan
  instead of contaminating the selected invocation with dependencies from unselected mode
  branches. Possible outcome hooks retain their own runtime backend and provenance rather than
  inheriting a caller override that execution would not apply. Projection failures use typed
  internal classifications rather than parsing display text. Core dry-run plans, run receipts,
  and managed CI projections bind the ordered selection through one SHA-256 selected graph
  identity. Each selected occurrence binds a digest of its resolved execution semantics; selected
  workflow-required services and their transitive definitions, plus requested lifecycle, host-port,
  and memory overrides, participate in the graph identity.
  Distinct workflow-phase roots and their edge endpoints retain separate invocation
  identities, including when phases reuse the same task. The Discord contract now makes
  `DISCORD_TOKEN` task-scoped and moves
  `setup:env:local` under the native `setup` mode branch while preserving explicit workflow host
  preparation. Selected task execution and dry-run reporting no longer read or expose optional
  host dotenv truth unless a selected occurrence declares the corresponding env obligation. A
  disposable clean fixture passed real container CI without `.env.local` and real native CI with
  `setup:env:local`; the live contract's dry-runs select those same distinct chains and omit
  `DISCORD_TOKEN` from container admission evidence.
  Persisted receipt archives now retain that canonical graph, reconcile its typed receipt input,
  and re-derive it from the immutable contract snapshot; older archives without reconstructable
  graph truth remain inspectable only as `legacy_unverified` and cannot become baseline, proof, or
  authority inputs. The first independent review found four remaining inventory reopenings;
  successful receipt env evidence, Doctor env reporting, occurrence-aware replay-rule scope, and
  workflow service summaries now consume the selected graph, with focused adversarial regressions.
  Lifecycle and runtime proof add assertions, seam observers, and negative controls as explicit
  canonical graph roots rather than rebuilding a task-name closure. Adversarial regressions also
  keep unselected mode branches out of strict replay snapshot creation, service cleanup, sandbox
  diagnosis, uv provenance findings, and workflow env artifact consumer evidence. Final
  independent selected-graph re-review passed. Native pressure additionally found that a declared
  Corepack package manager could resolve through an ambient native shim; the reviewed Core repair
  routes structured task commands and typed Node hydration through Corepack before any declared
  exec-mode orchestrator wraps them. Immutable propagation is Site
  `16b75bf531bfc788eb41a9e095a071b45cdca351`, Skills
  `f981911f2c21eb35e97fff6e077971e90aa4df1b`, and Examples
  `a565b22ae86e1840c9ecfe7fb97dd0c0bff9cc4b`; this Core reconciliation records the Site and
  Skills revisions against reviewed Core `a6b66b59ef255acb66726f022b572c41b6df5dd8`. The
  standalone Node service example materializes one host-only prerequisite for native setup while
  container verification omits it. Learn, FAQ, and Glossary remain unchanged because this repair
  adds no contract syntax, public term, or operator workflow.
- Eris adoption pressure exposed and repaired two bounded Core defects without changing the
  released `v1.6.27` runtime used by that partner contract: detector output no longer promotes
  named GitHub Actions bodies containing unresolved matrix or shell expressions into runnable task
  truth, and diagnose-only Rust toolchains must use comparable semantic requirements while
  rustup-owned run fulfillment retains channel names. Site and Skills carry the same distinction
  at immutable revisions pinned by Core. The same pressure review exposed a task-discovery UX gap:
  `ota tasks --use`, including `ota tasks --safe --use`, now renders task `notes` alongside
  descriptions and runnable commands so agents see declared proof limits and external-boundary
  guidance before selecting a lane. JSON already carried the notes and remains unchanged. Five
  remaining non-blocking follow-ons are recorded in the
  [execution-contract follow-on plan](../planning/execution-contract-follow-ons/plan.md):
  lock-strict Cargo hydration, mixed-mode preview selection, Doctor
  cause reconciliation, opaque-shell evidence boundaries, and identity-bound container cleanup.
  The remaining inactive items do not delay the Eris draft or interrupt V12.1; each requires a
  future version owner and independent activation.
- Anodizer onboarding pressure exposed two detector-fidelity defects. The repaired detector omits
  Taskfile helpers whose names begin with `_` or declare `internal: true` from executable contract
  truth and collapses repeated callers of the same reusable GitHub Actions verification step before
  collision disambiguation.
  A source-built reproduction at Anodizer `112a61a22557fa7f407bded897a8ec6671e35ae3`
  reduced the inferred task catalog from 50 to 46 entries, retained distinct verification lanes,
  and emitted neither private helpers nor caller-specific hash-qualified duplicates. Site and the
  canonical Skill carry this authoring boundary. Examples, Learn, FAQ, and Glossary remain
  unchanged because the repair adds no contract shape, command, public term, or operator workflow.
- V12 closure: implementation-order steps 1-10, the bounded real-repository pressure bar, and
  independent closure reconciliation are complete. Plausible and Outline retain exact
  selected-closure denial with
  `execution_started: false` plus independent setup, provider/database precursor,
  worktree/child-command, and outcome-hook absence witnesses; captured closure evidence records no
  selected service. The final corrected typed
  `ota up --dry-run` control is immutable-hosted in
  [run 33382559640](https://github.com/ota-run/ota/actions/runs/33382559640) against Core
  `a5aae10f5ce33e0d0927dbb913a685505933145b`. The archive-derived
  `effect_assurance` candidate remains schema-v5, `unknown`, reconciliation-bound, review-only,
  and platform-stably non-writable. Provider execution or mutation, callback behavior after Core's
  ordered delivery, independently administered policy authority, positive assurance, arbitrary
  child-process absence, repository-wide immutability, database correctness, and public archive
  export safety remain unproved or outside V12. V12 closure did not implicitly activate V12.1;
  the separate activation record in the V12.1 plan owns the new work.
- Core now owns `docs/pressure/evidence-manifest.json`, a machine-readable registry that binds
  retained pressure cases to exact revisions, matrices, exercised surfaces, proven facts, and
  explicit limits. The Site commits a generated discovery projection and validates it against Core
  through `ota run pressure:evidence:site:check`; it is not a certification, endorsement, or
  green-badge surface. The public index, Glossary, FAQ, Skills protocol, and engineering notes
  follow the same source-of-truth rule. Learn and Examples are unaffected because this is evidence
  accounting, not an operator contract workflow.
- Immutable Linux/x64 and macOS [run 33382559640](https://github.com/ota-run/ota/actions/runs/33382559640)
  against exact Core `a5aae10f5ce33e0d0927dbb913a685505933145b` closes the repaired typed
  `ota up --dry-run` admission control. Both retained artifacts show one admitted application plan,
  an explicit typed deny, `BLOCKED`, `execution_started: false`, and only the refusal action. The
  same jobs pass Core-owned plan-to-executor and sandbox-admission continuity controls. The exact
  selected fixture declares and immediately verifies absence of its setup sentinel, rendered
  environment artifact, proof artifact, durable-log path, and dependency command sentinel. This
  remains internal, provider-disabled evidence only; it does not prove provider contact or
  mutation, arbitrary child-process absence, repository-wide immutability, database correctness,
  positive assurance, or archive export safety.
- Final V12 internal Linux/x64 and macOS pressure is green in
  [run 33335973677](https://github.com/ota-run/ota/actions/runs/33335973677), bound to Core
  `25afb2b510a13ce149a2e9aa8ed5418a7af69482`. Both retained artifacts complete the selected typed
  refusal, policy/CI/sandbox, canary, archive, Doctor assurance, review-only candidate, stale-input,
  and contract-alias stages. They observe absent fixture setup, environment-rendering, proof-artifact,
  and durable-log paths after refusal. This does not prove provider contact or mutation, arbitrary
  child-process absence, repository-wide immutability, database correctness, positive assurance, or
  export safety.
- PythiaLabs pre-release fork pressure has reviewable declaration, native execution, and an
  Ota-owned Linux Node-container matrix. The declaration run passed; native execution passed ten
  of eleven lanes and retained Pythia's existing site-format failure as a repository finding; the
  container run passed the bounded MCP and site hydration/build closures using a digest-pinned Node
  image rather than Pythia's mutable Dockerfile. That exercise repaired Elixir version parsing when
  `elixir --version` also prints Erlang/OTP. Credentialed CAEP, merge, communications, and the
  Liminal lifecycle remain unmodeled. See `docs/pressure/pythialabs-discovery.md` for exact
  revisions, hosted evidence, and limits.
- V12 has bounded immutable real-repository refusal evidence: Linux/macOS fork matrices against
  Plausible Analytics and Outline each validate a committed PostgreSQL migration root, then pass
  direct task and workflow effect-refusal canaries with `execution_started: false` and an explicit
  typed deny. `ota doctor --json` now emits contract-graph coverage records for each declared
  typed-refusal canary, including unchallenged exact-equivalent attachments and opaque-path
  boundaries; it remains `unknown` until a later verified archive carries realization and execution
  evidence. A later Linux/macOS matrix against Core `84a988433cb3c0226a3569cdc2ee5202d3d5d375`
  confirms that those exact-equivalent and opaque paths remain unproved and that a generic caller
  refusal cannot pass a canary. Mixed-realization admission retains a declared-only
  attachment as an ineligible realization, binds its distinct identity into the decision, refuses
  ordinary execution and dry-run preview before its command body, reports the preview as `BLOCKED`
  without fabricating policy evidence when no pack exists, and reports an exact-origin canary as an assurance
  gap even when another attachment reaches the same effect. The internal mixed-realization carrier
  is immutable-hosted on Linux/x64 and macOS in
  [run 33300446201](https://github.com/ota-run/ota/actions/runs/33300446201) against Core
  `974caf686a45093587058ea140b82f1a81c0fa70`: both retained artifacts bind one shared effect
  identity, distinct eligible and ineligible attachment/realization identities, a blocked preview,
  and a declared-only `effect_canary_realization_ineligible` assurance gap with
  `execution_started: false`; neither command sentinel exists. This remains an internal,
  provider-disabled control. A later immutable Linux/macOS run
  [33301627289](https://github.com/ota-run/ota/actions/runs/33301627289) against Core
  `e7682a62287b173edaa8e2a18f57fc1593359dec` adds two equal local migration sets under distinct
  canonical resource namespaces: their effect and attachment identities remain distinct, an empty
  namespace authority refuses validation, and an exact primary-only policy deny does not select the
  secondary namespace. The same policy bytes retain one snapshot identity but produce distinct
  repository-controlled and caller-selected source evidence and decision identities. Rewriting an
  archived caller-selected decision as repository-controlled changes history from `1` valid / `0`
  invalid to `0` / `1`; restoring its original bytes returns it to `1` / `0`. This remains an
  internal, provider-disabled control. A subsequent immutable Linux/macOS
  [run 33302123045](https://github.com/ota-run/ota/actions/runs/33302123045) against Core
  `d0178b2013efd3d12f6baa0a94bb572f162c70a7` exercises the exact
  `ota ci projection --expect-identity` re-evaluation used after provider checkout. A changed
  policy makes the current projection identity differ from the rendered compatibility projection
  and returns `effect_policy_denied` with an explicit typed deny before workflow setup or durable
  logs. Both retained artifacts also pass the Core plan-to-executor substitution regression. This
  is still an internal, provider-disabled control. A subsequent immutable Linux/macOS
  [run 33303689321](https://github.com/ota-run/ota/actions/runs/33303689321) against Core
  `49a1a486a4431749ff33ec50ea4265afbc2a64f2` retains task and workflow typed-deny capability
  lanes with refused preflight and `provider_execution: disabled`, plus a typed-warn lane that
  remains refused as provider-disabled. Both artifacts pass the Core task/workflow retained
  command-admission sandbox control. This proves neither provider contact or mutation nor
  authoritative sandbox enforcement. The existing private workflow refusal archive now upgrades
  Doctor's V11.14 `effect_refusal_assurance` only for an exact current-contract workflow challenge
  with matching eligible attachment, realization, explicit typed deny, and pre-execution posture;
  task-only, stale, invalid, ambiguous, or mismatched evidence remains `unknown`. Immutable
  Linux/macOS [run 33309358828](https://github.com/ota-run/ota/actions/runs/33309358828) against
  Core `81c25e09c833559312e9cd43ce04a1c63f27d6fa` now proves that exact workflow-only promotion
  from a verified private archive and its fallback to `unknown` after archive tampering. It remains
  internal, provider-disabled negative evidence, not provider contact, mutation, positive
  assurance, or export safety. The final fork-only Linux/macOS matrices against Plausible and
  Outline are immutable-hosted in [run 33391482073](https://github.com/bobaikato/analytics/actions/runs/33391482073)
  and [run 33391486538](https://github.com/bobaikato/outline/actions/runs/33391486538) at fork
  revisions `fa24db238dae39a277e5fbfc08519488a32c1020` and
  `58b6a7731aff1a1237da1d9ade6021114b0a1c6e`. Every retained artifact binds clean source-built
  Core `e96cad13db9e4289c0985fca2ce6d8353a896da4`. Both create and verify one private workflow
  archive, promote only its exact workflow-only claim to `supported`, return it to `unknown` after
  context stripping, publish a projection-free `unknown` candidate, refuse its write attempt,
  emit a reconciliation-bound existing-declaration no-op, and refuse migration drift and a
  symlinked contract without publication. Their task and workflow canaries additionally retain
  absence of selected provider/database precursor, worktree/child-command, and outcome-hook
  sentinels; workflow canaries retain setup-sentinel absence, and closure evidence records no
  selected service. This is still selected-lane, provider-disabled evidence: it does not prove
  repository-wide readiness, actual provider/database behavior, database correctness, arbitrary
  child-process absence, complete repository immutability, positive assurance, or export safety.
  See `docs/pressure/v12-real-repository-effect-refusal.md`.
- completed version: V12 effect-bound refusal assurance. V11 is reconciled complete and V12 was
  the sole active version from 2026-08-25 through its closure. The first local implementation
  batch added strict PostgreSQL
  resource bindings, discriminated database schema-mutation definitions, exact task attachment
  origins, and separate JCS/SHA-256 domains for resource, consequence, attachment, evidence, and
  realization identity. Contract validation rejects ambiguous namespace authority, unresolved or
  duplicate references, action/bounds substitution, noncanonical or non-printable migration paths,
  Unicode or otherwise non-profile namespace components, and malformed identities. Authored migration
  content identities remain expected declaration truth rather than observed byte evidence. This
  batch includes a local, execution-disabled PostgreSQL schema-mutation action carrier: it
  captures the declared migration set with explicit entry-count, per-file, and total-byte limits.
  Unix capture retains no-follow directory/file handles; non-Unix execution refuses because an
  equivalent race-safe traversal is not implemented. The adapter requires its manifest identity to equal
  `migration_set.content_identity`, and derives a domain-separated application plan bound to the
  exact task attachment, contract invocation origin, repository-relative effective working directory,
  and effect realization. Dry-run publishes that non-secret plan. For repo-level `ota run` and
  `ota up`, including `ota up --dry-run`, one command-level typed preflight admits and verifies every
  typed action in the selected closure before
  command-scoped replay-input policy loading, agent/crossing/sandbox admission, workflow-environment
  artifact rendering, or durable-log preparation. It re-observes source truth through retained
  no-follow descriptors and returns the provider-disabled refusal before task conditions, required
  services, dependencies, shell dispatch, provider contact, or repository mutation. The runner
  repeats the same check as defense in depth for direct/internal callers. `ota proof runtime` and
  `ota proof lifecycle` now directly admit the complete selected proof closure, retaining phase
  and proof-helper invocation role/order without unselected-mode dependencies, before replay,
  crossing or sandbox admission, proof artifacts, service work, or child startup; a typed deny
  returns `OTA_EFFECT_POLICY_DENIED` with `execution_started: false`. `ota up --dry-run` carries the
  admitted non-secret plans and active decision in its blocked preview without starting work; other
  read-only command diagnosis and policy discovery are not claimed to occur after this boundary.
  Validation refuses mode or OS-variant execution-body overrides so runtime selection
  cannot replace the previewed typed action. Non-dry-run `ota up` emits its ordinary blocked
  readiness receipt with `execution_attempted: false`; the typed adapter emits no positive effect
  or execution receipt, archive, success claim, agent-safe authority, canary, or positive assurance.
  A non-dry-run `ota up --json` typed deny additionally retains the exact command-scoped decision
  as `receipt.typed_effect_policy_refusal` with `execution_started: false`. Ordinary refusal remains
  non-durable. Explicit `ota up --workflow <name> --archive-effect-refusal --json` creates one
  create-new receipt archive plus immutable contract and private policy snapshots only for an
  explicit typed deny. History independently re-derives the selected invocation closure,
  application plans, policy snapshot, and decision; missing, aliased, contradictory, or changed
  evidence invalidates the archive. This remains negative evidence, not provider execution,
  mutation proof, positive assurance, or a public export profile. Post-publication directory-sync
  failure returns `effect_refusal_archive_durability_uncertain` with `published: true`, the exact
  archive path, and recovery guidance; it never claims that no artifact was written. Failures
  after a receipt is durably published, including archive verification or retention pruning, return
  `effect_refusal_archive_post_publication_failed` with `published: true`, `durability: "confirmed"`,
  and the retained receipt path.
  Policy or contract snapshot sync uncertainty is separately identified with
  `effect_refusal_snapshot_durability_uncertain`, its exact `artifact_kind`, and
  `receipt_published: false`.
  The explicit archive carrier is immutable-hosted on Linux/x64 and macOS in
  [run 33243471896](https://github.com/ota-run/ota/actions/runs/33243471896) against Core
  `f8dc16c0eb3d8e5c9d9fda5c2f50674e2ba1b150`. Both retained artifacts record one valid refusal
  archive with `archive_count: 1` and `invalid_archive_count: 0`, then reject an archive with its
  replay context deliberately stripped (`0` valid, `1` invalid), and accept the restored bytes.
  The same fixture confirms workflow setup, environment rendering, and durable logs remain absent.
  It does not establish provider contact or mutation prevention outside Ota, positive assurance,
  or export safety.
  Archive-backed Doctor assurance is separately immutable-hosted on Linux/x64 and macOS in
  [run 33309358828](https://github.com/ota-run/ota/actions/runs/33309358828) against Core
  `81c25e09c833559312e9cd43ce04a1c63f27d6fa`. Both retained artifacts promote one exact
  workflow-only challenge to `supported` through private archive reconciliation, then return it
  to `unknown` after archive tampering. This remains internal, provider-disabled negative evidence;
  it does not prove provider contact, mutation prevention outside Ota, positive assurance, or
  export safety.
  Task and workflow harness capability JSON now carries mandatory typed effect-policy posture:
  untyped lanes are `not_applicable`, while typed lanes carry the evaluated decision, policy
  snapshot, selected execution graph, and effect-set identities or an explicit unavailable state.
  Only untyped `not_applicable` lanes appear under `callable_tasks` or `callable_workflows`; typed
  lanes remain under refused capabilities with `provider_execution: disabled`. The published
  schema requires evaluated deny, allow/warn, and unavailable posture to carry their matching
  refused preflight reason. Live
  sandbox admission consumes the exact closure, retained policy snapshot, application plans, and
  verified decision produced by command admission rather than re-planning under a second origin.
  Missing policy truth or aggregate denial refuses before canonical sandbox-policy construction and
  provider-capability evaluation, while allow/warn does not become provider authority. Malformed
  effect policy does not affect an untyped capability lane. Sandbox application evidence and archive
  semantics are unchanged because typed provider execution remains disabled.
  Provider execution, independently administered policy admission, provider-side mutation
  semantics, and independent real-repository effect pressure remain unproved. The bounded internal
  carrier is now immutable-hosted on Linux/x64 and macOS in
  [Smoke run 32994303400](https://github.com/ota-run/ota/actions/runs/32994303400) against exact Core
  `73f7fea9fb76af514e6a97e42562d30b683768ad`. Both retained artifacts bind that revision and prove
  contract validation, typed run/up previews, direct run/up/proof refusal before setup and environment
  rendering, blocked-receipt schema conformance with `execution_attempted: false`, stale migration-byte
  refusal, and intermediate-symlink escape refusal. Every execution/refusal status is exactly `1`,
  and neither setup sentinel, rendered environment, nor durable execution logs exist. This internal
  fixture does not contact a provider, execute a repository task, prove provider mutation semantics,
  or substitute for later independent real-repository pressure. The
  foundation is committed at Core `f3d4b8e1`, Site `5926f69`, Skills `d05b1d1`, and Examples
  `2dad574`; canonical-identity hardening is committed at Core `1b9a03d6`, Site `e78b963`, and
  Skills `30c8dbd`. The hosted carrier above exercises the selected plan/admission path but does not
  replace the exhaustive local identity-domain regression matrix. V12.2 onward and the authority
  distribution plan remain planned and inactive; adapter/profile conformance constrains the named
  V12.1 implementation without establishing support. The reference Example correctly requires Ota `1.6.27`; the source-built
  `v1.6.27` development binary validated that exact contract locally on 2026-08-25. This closes
  only the local minimum-version gate. Application plans now carry the canonical discriminated
  action bounds needed by an executor, rather than requiring contract reconstruction. A Core-owned,
  test-only continuity control re-verifies the admitted plan and retained migration bytes, then
  delivers every ordered file to the selected callback before Core records a delivery
  acknowledgement. Substituted source or plan and disconnected or failing callbacks refuse. The
  callback remains trusted for its behavior after delivery, so this does not enable provider contact,
  mutation, execution evidence, or positive assurance. The next local V12 batch extends the existing
  command-scoped policy-pack loader with canonical typed effect rules and one shared evaluator below
  CLI orchestration. Its content-addressed decision binds the policy snapshot, redacted source
  location, source kind and authority posture, selected invocation and execution graph, effect and
  realization sets, every matching typed rule, current coarse-effect components, and aggregate
  `deny > warn > allow` precedence. Repo-level `ota run` and `ota up`, including read-only
  `--dry-run`, consume the selected-closure decision before replay, authority, sandbox, setup,
  environment rendering, services, dependencies, provider contact, or repository mutation. A typed
  rule, strict fallback, or coarse component whose aggregate is `deny` causes
  `OTA_EFFECT_POLICY_DENIED`; dry-run publishes the non-secret plans and decision without starting work.
  Caller overrides remain limited to shipped coarse selectors and cannot target typed rules. This is
  operational refusal only: provider execution, positive effect/execution receipts and archives,
  and positive assurance remain disabled. Plan-to-executor continuity is now
  immutable-hosted on Linux/x64 and macOS in
  [run 33032683375](https://github.com/ota-run/ota/actions/runs/33032683375) against exact Core
  `32e3395f92e1114ce209dc620d14ecc82330856f`. Both retained artifacts bind that revision, record all
  seven admission/refusal stages and all seven side-effect-absence checkpoints, retain status `1`
  for run, up, inherited proof, stale-input, and intermediate-symlink refusals, publish canonical
  action bounds and ordered migration manifests, and pass the Core-owned continuity control. This
  remains an internal execution-disabled fixture; no provider behavior or independent
  real-repository effect pressure is proved. Contract validation now
  also rejects a released `agent.bootstrap.ota.source.version`
  below `metadata.ota.minimum_version`; `source: contract` is the canonical CI consumer so a
  workflow cannot maintain a divergent released bootstrap version. Git revisions and pressure
  branches remain intentionally incomparable to a release floor. The connected typed-adapter
  propagation is committed at Site `00a2f60729e264dc4806b1698b475576a3a58a93`, Skills
  `8ef5d9e2fe3a31010c9bb0af534114d390c52a3d`, and Examples
  `6634e1509ebcbbfef652e084b8f3982fc5fa0dda`; the unrelated generated Site pressure note remains
  outside this batch. The plan-to-executor reference propagation is committed at Site
  `cb1f21463abcc5b1e866ea87aafc1c31bdfc7729` and the reviewed implementation foundation at Core
  `32e3395f92e1114ce209dc620d14ecc82330856f`. Typed effect-policy refusal is committed at Core
  `d72c6c85`, with immutable admission-pressure harness `212446c000b55d68bad5906a4b532ce5055c1477`.
  [Run 33067741989](https://github.com/ota-run/ota/actions/runs/33067741989) is green on Linux/x64
  and macOS and retains artifacts bound to that exact Core revision. Both carriers publish an
  explicit `deny` decision for the eligible schema-mutation effect, return
  `OTA_EFFECT_POLICY_DENIED` from `run`, and block `up` and inherited runtime proof at
  `preconditions` with `execution_attempted: false`. Setup, rendered workflow environment, durable
  execution logs, stale migration input, and an intermediate-symlink escape are respectively
  absent or refused. This is bounded internal-fixture evidence of pre-side-effect policy refusal;
  it does not prove provider contact or mutation, canary assurance, positive receipts or archives,
  independently administered policy authority, or the required independent real-repository
  effect-pressure bar. The composed command-admission, capability, and sandbox carrier is now
  immutable-hosted on Linux/x64 and macOS in
  [run 33199213628](https://github.com/ota-run/ota/actions/runs/33199213628) against exact Core
  `1339476f1806a14278028de95020afd7e9ef5098`. Both retained artifacts record all nine staged
  checks: typed `run`, `up`, and inherited-proof denial with `execution_attempted: false`; absent
  workflow setup, environment-rendering, proof-artifact, and durable-log paths; stale-input and
  intermediate-symlink refusal; task/workflow canary results; and the Core-owned
  delivery-continuity control. The fixture does not independently establish arbitrary
  child-process absence, provider contact or mutation, or complete repository immutability. It
  remains internal execution-disabled evidence and does not close independent real-repository
  pressure, positive receipts/archives, assurance, or independently administered policy
  authority. Between V12 feature batches,
  typed-effect ownership was extracted without
  behavior changes: `effect_admission` owns runner-independent domain verification,
  `effect_orchestration` adapts selected runner closures for CLI admission, and `runner` translates
  domain failures without a reverse dependency. The current local branch adds contract-owned
  `agent.effect_refusal_canaries` and execution-free task/workflow invocations. A pass requires one
  exact predeclared origin and eligible realization denied by an explicit matching typed rule;
  strict fallback, generic refusal, unknown IDs, caller overrides, absent origins, and non-denial
  cannot false-green it. The semantic canary identity excludes the local locator while binding the
  effect, attachment, realization, selected invocation, invocation origin, and expected typed-deny
  posture. It emits
  `passed | not_evaluated | assurance_gap | failed` with `execution_started: false`. The bounded
  carrier is immutable-hosted on Linux/x64 and macOS in
  [run 33098093213](https://github.com/ota-run/ota/actions/runs/33098093213) against exact Core
  `dc368fbb2fc298490bfce6de86ea4ed79b493beb`. Both retained artifacts prove task and workflow
  canary passes only for the exact eligible explicit typed denial, strict-fallback and unknown-ID
  non-passing outcomes, icon-free plain output, and no setup, workflow-environment, or durable-log
  side effect. This internal fixture does not enable or prove provider execution or mutation,
  positive receipts, archives, assurance, independently administered policy authority, or the
  required independent real-repository effect-pressure bar. Connected Site, Skills, Examples,
  Learn, schema, and command propagation is committed at Site `f1ef9c9`, Skills `7298340`, and
  Examples `f209c31`; Core implementation and pressure harness are committed at `dc368fbb`.
  Complete selected runtime-proof closure admission is now immutable-hosted on Linux/x64 and macOS
  in [run 33166914327](https://github.com/ota-run/ota/actions/runs/33166914327) against Core
  `0f4db8e2a19367f4cfb6d6a4522ad3b007690bba`. Both retained artifacts prove a typed policy deny
  before proof artifacts, setup, workflow environment rendering, durable execution logs, or child
  startup, while the Core-owned ordered-delivery control remains green. This is still internal,
  execution-disabled evidence; it does not establish provider mutation, positive evidence, or
  independent real-repository pressure.
- completed V11.21 enforced sandbox policy application. Core now derives one
  provider-neutral, target-platform-bound segment graph from the selected task/workflow closure,
  applies only identified monotonic policy restrictions, and fails closed before preparation when
  the first `oci_local` provider cannot enforce an authoritative selected-lane control. Compatible
  explicit-platform ephemeral container lanes receive read-only repository mounts, existing
  writable carve-outs, bounded external-network denial, pre-mutation cleanup leases, distinct
  per-invocation boundaries, initial and terminal engine inspection, engine-confirmed removal, and
  runner-authored receipt evidence. Engine inspection rejects every mount outside the exact
  repository-root and declared carve-out set. Those receipts are archived automatically with the
  normalized contract snapshot and any identified policy-authority snapshot; archive reads
  re-derive canonical policy and authority-owned overlays, reconcile completed segments with
  archived task outcomes, and reject unbound overlay, segment, edge, capability, or application
  identities. Dry-run performs no provider-backed runtime/tool probes, while real OCI precondition
  probes execute as separately identified, cleanup-confirmed `precondition_probe` invocations
  bound to the exact admitted requirement-owning segment inside the registered sandbox application
  transaction; blocking probes retain that evidence in the refusal receipt and cannot substitute
  for task-execution evidence. Declared Linux OCI `container.platform` is canonical for
  target-specific variants, inputs, environment, requirements, and ordinary/provider-enforced
  container creation, including persistent-container reconciliation. Reusing one task identity
  across multiple phases refuses rather than collapsing separate invocations. Managed isolated
  paths refuse until their durable provider resources have transaction-bound creation, retention,
  and failure-cleanup evidence. Initial independent pressure exposed Docker Desktop's
  multi-platform image metadata as the wrong platform witness: image inspection can report the
  host-native variant even after `create --platform` selected another target. Ota now binds the
  exact provider-applied create request to the created container's platform evidence, requires an
  exact match when the provider reports OS and architecture, and accepts Docker's OS-only
  container report only after successful creation with the full declared platform.
  The first adapter admits finite command bodies only; typed task bodies, requirements, services,
  conditional checks, and authoritative lifecycle-proof closures refuse rather than execute
  outside the evidenced boundary. Ota-owned `run` flags, including `--sandbox-target`, remain
  command flags when written after the task and before task inputs. `codex_local` remains compiled
  guidance, targeted egress remains unsupported by stock OCI, and raw shell outside Ota remains
  outside this enforcement boundary. Core fixture proof is green. Independent hosted pressure is
  also green against exact Core `d796f28e5556c0f1315052e8782ed774e9156922`:
  create-chrome-extension run
  [30544809360](https://github.com/bobaikato/create-chrome-extension/actions/runs/30544809360)
  proves generated output inside one writable carve-out, and Caddy run
  [30544809898](https://github.com/bobaikato/caddy/actions/runs/30544809898) independently proves
  a source-manifest artifact under external-network denial. Both runs include protected
  `ota.yaml` write refusal, discovery JSON, dry-run admission, real task and workflow execution,
  terminal cleanup, archive reconciliation, and an explicit uncovered-material-behavior
  inventory. They prove only their selected Linux/amd64 `oci_local` lanes. The full Core and
  first-party release gate is green, so V11.21 is complete.
- supplementary V11.21 pressure: Buzz run
  [30559160264](https://github.com/bobaikato/buzz/actions/runs/30559160264) confirms that the
  stock adapter refuses a real Compose, migration, persistent-volume, hydration, and external-state
  integration closure before Buzz provider or worktree mutation. This does not widen V11.21. It
  records two explicit boundaries: the clean host remains Doctor-blocked on the declared missing
  `just` tool, and pre-boundary sandbox refusal is inline receipt evidence rather than a durable
  refusal archive.
- The active-execution registry now derives runtime listener ownership across the complete selected
  closure and compares actual execution namespace, network protocol, bind/publication address, and
  host port. This closes both sides of the earlier task-name heuristic: disjoint native/container
  service endpoints and isolated write namespaces can coexist, while different task names sharing
  one listener refuse. Shared write and env-materialization ownership also detects nested path
  overlap. Existing registry entries without runtime or write-namespace identity remain fail-closed
  until restarted. One-run `--host-port` selection now participates in that same resource identity
  for direct native service tasks as well as container and native Compose publication. Direct
  native execution applies the selected port to both bind and public runtime truth, including typed
  launch arguments and canonical runtime env, because there is no separate host-publication layer.
  When one fixed host listener collides, text output now names the host port, requested and active
  execution modes, and exact owner before suggesting `--host-port <free port>` when the selected
  lane's own execution-option preflight admits that override; mixed listener/write conflicts remain
  broader active-execution failures, identify the free-port choice as resolving only
  `runtime_listener`, and name every remaining typed reason that must be resolved before retrying.
  Suggested reruns preserve agent mode. The compact summary keeps its established ordering and adds
  only `Reason`/`Reasons` and `Host port` when that conflict evidence exists.
- V11.7 audited-crossing authority is complete for its bounded OSS slice after independent review
  and immutable hardened-launcher pressure. Core derives one canonical content-addressed crossing
  scope from the selected
  task/workflow execution graph, refuses unresolved task-input identity, and supports an opt-in
  `governance.crossing_authority.authority_id` plus `ota run|up|proof runtime|proof lifecycle
  --grant <id>` admission path. Proof commands refuse before artifacts, child execution, lifecycle
  ownership, service start, or assertion execution. Proof invocation role and order, lifecycle
  selected-service closure, target platform, host-port, memory, dependency selection, and
  normalized runtime `--ready-timeout` are semantic grant scope. One proof-owned transaction now
  spans the complete runtime or lifecycle invocation set and cleanup. A bounded runner-private
  Unix descriptor carries authority only between immediate Ota processes and is removed before
  selected code executes. Runtime archive v6 and lifecycle archive v3 embed and re-derive the
  exact terminal authority rather than inheriting a workflow-only grant. Runtime archive
  and lifecycle archive reconciliation preserve requested backend and lifecycle overrides
  separately from effective values, preventing an implicit default from changing authority
  identity during verification.
  The first `prebound_file` carrier reads only fixed system trust state protected from Ota's current
  unprivileged process; it does not claim hardened provider-attested privilege separation. It verifies an
  Ed25519/RFC-8785 signed bundle, exact contract/scope/family/classification/actor binding,
  bounded freshness, signed revocations, and protected sequence/clock high-water evidence; and
  refuses before sandbox admission or execution side effects. `ota run` repeats the
  time-, sequence-, and revocation-sensitive admission immediately before transaction creation.
  `ota authority inspect --json` now exposes a separate diagnostic-only hardening profile over the
  same fixed-path protected-file verifier. It checks every fixed-store binding, emits typed
  required/informational observations, and remains bounded to
  `current_process_filesystem_guarded`; it selects no grant, writes no authority state or receipt,
  and cannot make a crossing admissible. Passwordless sudo, namespace control, alternative
  container endpoints, provider metadata credentials, and broader escalation stay explicit
  unknowns when Ota cannot observe them safely.
  Real execution durably creates a
  runner-owned per-scope crossing transaction before any selected-lane side effect, terminalizes it
  on success, precondition/startup failure, interruption, or abandoned recovery, and binds that
  transaction identity to the fresh crossing receipt. Archived receipts preserve the signed
  bundle, binding, scope,
  admission time, and terminal transaction for re-derivation against the current fixed trust root.
  Receipt history derives crossing necessity from each authority-bearing archive's canonical
  selected-invocation scope and archived contract snapshot, never from global authority
  configuration or an editable lane label;
  older snapshot-less receipts remain visible only as `legacy_unverified`.
  The first local transaction carrier is explicitly `runner_local_content_addressed`: it is
  runner-authored, locked, and internally reconciled, but not independently authenticated against
  same-user state tampering. Refusal and dry-run emit no crossing record; successful dry-run
  publishes only `admissible_not_consumed`, while task dry-run and workflow-refusal evidence
  additionally carry derived scope/contract identities, boundary family, and classification alongside typed
  `prebound_file` authority-source, authority/grant selection, reason, and
  `execution_started: false` evidence. That issuance surface never exposes task inputs or trust
  material and avoids workflow-side reconstruction of Ota semantics. Existing contracts
  remain compatible until they opt into crossing authority, and grants never bypass V11.3 agent
  refusal. Core regression proof covers exact admission, mutation, revocation, sequence rollback,
  selected-graph expansion, missing-grant refusal, dry-run parity, pending-journal recovery, and
  terminal outcome reconciliation. Create-chrome-extension hosted refusal pressure
  [30714738522](https://github.com/bobaikato/create-chrome-extension/actions/runs/30714738522)
  confirms that a GitHub-hosted runner without the fixed authority source refuses before scaffold
  execution; its normal matrix [30718303916](https://github.com/bobaikato/create-chrome-extension/actions/runs/30718303916)
  is green across native and bounded container paths. This does not prove live grant authority
  separation because a hosted job could self-provision its own filesystem state. The first
  carrier's live, expired, revoked, and out-of-scope VPS pressure is now green:
  [30863257307](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863257307),
  [30862934335](https://github.com/bobaikato/create-chrome-extension/actions/runs/30862934335),
  [30863024099](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863024099), and
  [30863121110](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863121110).
  The live artifact retains the exact transaction-bound receipt archive; every refusal preserves
  typed admission evidence and an unchanged checkout before and after dry-run and real refusal.
  The immutable native Linux/systemd proof [31986767770](https://github.com/ota-run/ota/actions/runs/31986767770)
  now passes on the OrbStack-backed `self-hosted/linux/orbstack/systemd` runner at this Core
  revision. Its retained artifact proves native validation, `ota doctor`, `ota up`, and runtime
  proof execution; the proof remains explicitly bounded to its selected runtime path and records
  dependency exercise and broader-repository behavior as unproved.
  The carrier-neutral transaction/archive foundation uses transaction schema v2 to bind authority
  carrier, admission identity, authorization identity, and terminal state. V1 archives remain
  legacy `prebound_file` evidence without a carrier envelope; receipt history rejects
  carrier-envelope injection into v1 and missing or substituted envelopes in v2. The Unix
  launcher-session `authority_broker` carrier is now executable for governed `ota run` and
  `ota up`. Its v1 wire structs, fixed message domains, bounded framing, and canonical nonce,
  message, and work-unit identities now come from the immutable public
  `ota-run/authority-protocol` revision `242685d5b7c3904681f1c71d734fbe2d41679dda`; Core retains
  trust-root, verification, admission, transaction, receipt, and archive ownership. It selects
  exactly one protected binding from `/etc/ota/crossing-brokers.json`, freezes
  the semantic work unit, verifies challenge-bound launcher attestation, obtains signed
  authorization, creates the durable pending transaction, and atomically consumes the exact lease
  after deterministic admission succeeds and before provisioning or selected work. Ota persists
  the exact consume intent before transport. If acknowledgement is uncertain, a later
  invocation obtains fresh launcher attestation and re-queries the exact intent; consumed,
  not-consumed, and unknown results all close the abandoned transaction as incomplete and never
  resume work. A durably recorded signed recovery status is re-verified locally after restart
  without a second query, and consumed recovery retains its intent until the atomic terminal write.
  Pre-recovery seven-domain broker archives retain their original identity through archive-only
  compatibility; live bindings require all nine current domains.
  The reserved v3 Linux `systemd_protected_launcher/v1` branch now requires Core to send a
  private process-posture preface and to match that exact posture against the signed complete
  systemd launcher/job-principal instance before admitting broker traffic. Core receipt and archive
  re-verification are implemented and locally tested. Core now pins the immutable Protocol
  child-identity foundation at `6a2d0dc504a313a513ee41105f51449195c85797`; the reviewed
  execution-disabled Launcher implementation is `73a39c95ffab3125819ee655bdc7a740ec3204b9`.
  The execution-disabled authority-launcher
  foundation now consumes only the fixed systemd listener, reconciles its unique
  `/proc/net/unix` inode/path with protected socket metadata, derives the job peer through
  `SO_PEERCRED`, validates the governed Ota command before helper work, retains the exact verified
  Ota executable descriptor, and maps one protected job principal to one distinct execution
  principal. Its short-lived helper clears inherited descriptors and supplementary groups, adopts
  the complete target UID/GID posture with `no_new_privs`, and requires Linux `openat2` containment
  before returning only the repository directory descriptor through `SCM_RIGHTS`. The service
  now creates and fsyncs a protected active-slot intent, forks the exact fixed Ota binary as a
  root-stopped child, binds its invocation/request, PID/start, binary, principal, directory, and
  exact descriptor-object posture. Startup promotes valid child- or scope-bearing temporary state
  and uses PID-bound pre-scope cleanup; an intent-only, mismatched, unsupported, or uncertain
  recovery remains a hard refusal. The current immutable scope slice at Protocol
  `adaabfb8300925a09975c7244e27242b5cd41e60` and Launcher
  `0f9d9eb33e37d6cd855aafdbc7c4d72b3c8957e2` requests one request-derived transient scope from the
  root systemd manager, independently reconciles its fixed slice, non-delegated controls, kernel
  cgroup, and sole stopped PID, and records that identity before terminal cleanup. Scope-bearing
  recovery stops the exact scope when still present and confirms the scope absent plus its recorded
  cgroup empty or absent before releasing the principal slot.
  OrbStack's systemd refuses the real pre-exec PID attachment with `ENOTTY`, so that environment
  proves fail-closed behavior but not positive scope ownership. Immutable Linux/x64 VPS run
  [31373366733](https://github.com/bobaikato/create-chrome-extension/actions/runs/31373366733)
  now proves the exact reproducibly built launcher/client identities, fixed socket, root-stopped
  child, positive `openat2` containment, request-derived transient scope, terminal scope removal,
  child reap, active-slot cleanup, and unchanged repository state. Crash/recovery run
  [31373928434](https://github.com/bobaikato/create-chrome-extension/actions/runs/31373928434)
  proves a root-only post-scope crash at exit `86`, durable abandoned-slot reconciliation before the
  next request, terminal refusal, and zero residual slots, scopes, or recorded children. The empty
  execution-disabled child was collected before the post-crash scope observation, so the evidence
  does not claim a still-loaded scope at that instant. These runs exposed and fixed listener-table,
  systemd Scope-interface, collected-unit cleanup, and build-path reproducibility defects. Those
  immutable revisions never resume the child and do not contact the broker, consume authority,
  execute selected work, or emit receipts/archives. OrbStack Linux/x64 root tests
  against the immutable stopped-child revisions prove socket
  replacement refusal, descriptor transport, and fail-closed behavior when that environment
  reports `openat2` as `ENOSYS`. The separate VPS kernel-pressure run
  [31319741342](https://github.com/bobaikato/create-chrome-extension/actions/runs/31319741342)
  checks out exact authority-launcher `99affd90f712512fa1fd7c039868d114904736cf` and proves the
  positive `openat2` containment flags plus symlink-escape refusal on Linux/x86_64 kernel
  `6.8.0-134`. This does not exercise the root UID-switching helper or the systemd service.
  The execution-disabled immutable transient-scope foundation is now proved. Core
  `cc680cef790bf8334ee0dfe513c202a51c21954e`, Protocol
  `b4f36fe450dc4047bd7bd623ea8ba60fd951e31d`, and Launcher
  `d8aa1d0bf9783d29d53d0a5e912f09f1fa414624` resume that exact scoped child only far enough to
  receive Core's bounded private
  process-posture preface. It re-derives the posture identity and binds PID/start time, Ota binary,
  and principal mapping before exact scope/child/slot cleanup. Core emits the preface before CLI
  parsing or command dispatch and blocks for launcher continuation; this slice deliberately sends
  none. Malformed or substituted posture fails closed. Hosted normal run
  [31389237232](https://github.com/bobaikato/create-chrome-extension/actions/runs/31389237232)
  and root-armed crash/recovery run
  [31389713244](https://github.com/bobaikato/create-chrome-extension/actions/runs/31389713244)
  bind exact reproducible binary identities, unchanged repository state, zero terminal scopes, and
  the typed `posture_admitted_boundary_removed` terminal stage; the crash path records launcher
  exit `86` before fresh reconciliation. The first hosted posture attempt also exposed that Core's
  schema-validation fallback embedded its absolute compile checkout. Published JSON schemas are now
  embedded into the source-built binary, making installed schema validation and immutable Core
  binary reconciliation independent of both a source checkout and the checkout directory that
  compiled Ota. This closes only the immutable hosted execution-disabled posture gate. V3
  attestation, broker authorization, one-use lease consumption, selected execution, receipt/archive
  evidence, the production systemd execution path, and provider attestation remain unproved.
  The independently reviewed execution-disabled V3 bridge pins Protocol
  `953e9e6407c9de030822b1f891046c2829b3c714` and Launcher
  `0ed578a46ce821d8dd1da671a2e53c75ded1ed0b`. An identity-bound launcher continuation binds the
  exact invocation, child, working directory, posture, and principal mapping while unlocking CLI
  parsing after exact posture admission. Core consumes and removes the launcher-only startup
  environment before CLI dispatch, then freezes the real semantic scope and verifies one signed V3
  response through its canonical broker verifier, including exact reconciliation back to the
  retained startup binding. The launcher observes but does not forward the resulting exact
  authorization request. Exact scope, cgroup, child, and active-slot cleanup precede the typed
  `attestation_admitted_before_authorization_boundary_removed` refusal. This bridge has local
  protocol and Linux regression evidence only and still requires immutable Linux/x64 pressure. It
  does not prove authorization, lease consumption, selected execution, receipts/archives, the
  complete production path, or provider attestation. No example, Skill, or
  Site propagation is required for this internal execution-disabled carrier step because it adds no
  contract, CLI, operator, receipt, or archive surface; those surfaces must move with the first
  usable production adapter.
  The committed systemd V3 candidate at Authority Protocol
  `574563d1f69a674960d0b3228c5a13b13bc42c19`, Authority Launcher
  `13bf6db71610b86c81a251f440b80b9b8947a67d`, and Core
  `31fa95b4d28a8a4971ee3fd65c841d40e54ac4d9` completes the protected collector and producer
  bridge. Authority Protocol defines the canonical domain-separated claims/request/response,
  protected producer binding, `ota.authority-launcher.systemd/v3` profile identity
  `sha256:b5853a12e72c4ca32b0f93a38bc8f1097c7809039b58449f67fcf9019d0ea480`, and paired
  `ota.authority-job-principal.systemd/v2` identity
  `sha256:ee6ea951aff4a80f8a4f93c576a93e3b29245b87d162726c2401c124a7a78659`. The Launcher verifies
  protected installation identities, exact systemd unit/socket/scope properties, process
  containment, account/sudo/Polkit posture, protected-path and host-socket denial, and Ota
  process-access denial before invoking the separately credentialed `ota-authority-attestor` over
  fixed `SOCK_SEQPACKET`. The producer owns signing key, clock, and durable idempotent issuance;
  the launcher owns only public verification and exact request/response reconciliation. Core now
  independently re-derives the complete ordered profile, nested identities, signed claims, and
  retained startup binding before emitting an authorization request.
  Local ARM64 OrbStack PID 1 systemd pressure reached `authorization_received`, then the launcher
  deliberately withheld the request and removed the exact scope, cgroup, child, and active slot.
  Selected-work sentinel, receipt store, and broker decision/lease state remained empty.
  Protected-installation drift, systemd runtime drift, and missing producer credentials refused
  before authorization with zero terminal slots/scopes. A pressure-only exit after durable scope
  recording retained one recovery slot; the next activation reconciled it to zero before accepting
  another request. That was local candidate evidence. Immutable Linux/x64 PID 1 systemd run
  [31530832876](https://github.com/ota-run/authority-launcher/actions/runs/31530832876) now binds exact
  Protocol `574563d1f69a674960d0b3228c5a13b13bc42c19`, Launcher
  `c69ad3afc6afef0e260a7eeaa4f7340971db50af`, and clean source-built Core
  `31fa95b4d28a8a4971ee3fd65c841d40e54ac4d9`. Its retained cursor-isolated artifact proves the
  complete signed positive/recovery stage sequence and typed terminal refusal; installation drift,
  runtime-property drift, unavailable producer credentials, and the injected pre-session crash do
  not reach authorization. It records one durable `scope_attached` crash slot, zero terminal
  slots/scopes, byte-identical repository manifests, no selected-work or `.ota` state, and only the
  public verifier identity. This closes the hosted execution-disabled V3 admission gate only. The
  GitHub workflow controller still provisions the root services, so independently administered
  provider/launcher separation is not proved. At those immutable revisions, no authorization
  decision, one-use lease, selected execution, crossing receipt/archive, or provider-attested
  separation existed.
  The signed authorization-decision slice advances only through decision admission.
  Protocol adds a Core-authored, identity-bound decision acknowledgement and launcher relay
  envelope. The Launcher binds a protected pressure broker executable and service/socket identity,
  rechecks the live pidfd-bound executable around relay traffic, forwards Core's exact request only
  after complete V3 admission, relays only signed decisions, requires Core's exact acknowledgement,
  and durably journals that relay before exact boundary cleanup. Core acknowledges only a decision
  that passes canonical signature, freshness, request,
  attestation, contract, work-unit, and semantic-scope verification. Allowed decisions end at
  `authorization_decision_verified_before_lease_boundary_removed`; denied or invalid decisions
  remain bounded refusals. Immutable Linux/x64 PID 1 systemd run
  [31561247605](https://github.com/ota-run/authority-launcher/actions/runs/31561247605) covers allowed,
  denied, stale, wrong-scope, pending-timeout, ambiguous, and unavailable-proxy cases with zero
  worktree, receipt, active-slot,
  or scope residue. Negative cases require exact pressure-peer response checkpoints and Core
  acknowledgement counts rather than the generic protocol-refusal terminal alone. The artifact
  retains public signed decisions, the public broker verifier binding, and bounded relay envelopes
  for independent identity and signature re-verification after cleanup, never private signing
  material. Core also requires a final response after pending authority to advance the broker
  revision, preventing an older still-valid final response from replacing newer pending state. The
  matrix injects crashes after durable scope and allowed-decision recording and requires
  cleanup-only recovery before a fresh request, with complete repository-manifest equality for each
  decision scenario. It binds exact Protocol `6a92d8db9d089e44d1980f1871bf6e90eccb9960`, Launcher
  `77ab20aa6ed5e3dd42cc6815ba2de7cd36d543bf`, and clean source-built Core
  `b71b78ca33ea2edd7bb03ceb66c5e1e104217cd9`. Independent artifact inspection re-verified all eight
  signed decisions, all five relayed admission/decision identity pairs, zero terminal slots/scopes,
  two cleanup-only crash recoveries, fourteen byte-identical repository-manifest pairs, and no
  selected-work, `.ota`, lease, receipt, archive, private-key, or credential residue. No
  lease issuance/consumption, selected execution,
  receipt/archive, independently administered separation, or provider attestation is claimed.
  The execution-disabled one-use lease boundary is now immutable-hosted pressure evidence. Protocol
  `899718c93f205eea8ae403e041be9449daa89192`, Launcher
  `2185682777c3603ae428dda68d47b1e39d709753`, and clean source-built Core
  `874c5954798453f92a0141bfc964fe1a90db8d92` passed Linux/x64 PID 1 systemd run
  [31631358796](https://github.com/ota-run/authority-launcher/actions/runs/31631358796). Core freezes a
  launcher-owned pending transaction without repository state, binds its authentication posture to
  the private active-slot persistence owner, and emits one exact consume request only after signed
  V3 attestation, authorization, and prepared-lease verification. Launcher fsyncs the consume intent
  before broker relay and the signed consumed response before terminal cleanup. The pressure-only
  broker atomically persists spent lease identities in root-owned `0700`/`0600` state before its
  first response; replaying the identical lease and consume request produces one signed
  `already_consumed` response while Core records exactly one accepted consumption. The matrix also
  covers denial, stale and wrong-scope responses, pending timeout, ambiguity, unavailable broker,
  protected installation/runtime/credential drift, and both intent/acknowledgement and
  post-consumption crash recovery. Its retained artifact has byte-identical repository manifests,
  one deliberate pending recovery slot only at each injected crash boundary, and zero terminal
  slots/scopes or selected-work, `.ota`, receipt, and archive residue. This proves one-use lease
  consumption only for the execution-disabled systemd carrier and pressure broker. At that
  revision, selected execution, crossing receipts and archive re-verification for launcher-owned
  evidence, and independently administered provider/launcher separation remained open; the later
  immutable gates recorded below close those V11.7 requirements. Site, Skills, and Examples remain
  unaffected because this slice adds no public command,
  contract-authoring, receipt, archive, or usable operator surface.
  The exact replay reopens root-owned durable state but does not restart the pressure broker
  process; restart persistence is not separately pressure-proven.
  Immutable Linux/x64 PID 1 pressure run
  [31664495937](https://github.com/ota-run/authority-launcher/actions/runs/31664495937)
  binds Protocol `9fb00a4ab0f1b4c635dbab67c2e6b140b8eade9c`, Core
  `06976f3eb4919a0bddaa318ed0824a6b9448aaaf`, and Launcher
  `e8b6ae5108559508cfb75141cb9b317d46c182f3`. Core retains the launcher session after atomic
  consumption, executes only the frozen work unit, finalizes the crossing transaction and receipt,
  and requires exact launcher persistence acknowledgement before exiting. The launcher then
  reconciles the child exit and
  emits terminal finalization only after the exact child, scope, cgroup, and active slot are absent.
  The run proves completed, failed, interrupted, replay-refused, pre-execution refusal, and five
  crash-recovery boundaries with exact child, scope, cgroup, and active-slot removal. Receipt
  history reports one valid archive and zero invalid archives for the successful lane. Portable Ota
  archives do not yet embed the launcher-authored post-process finalization; the
  outer pressure artifact is the only current carrier for that cleanup record. The canonical Skill
  and Site broker reference carry that distinction. Examples and the public command index are
  unaffected because this candidate adds no contract shape, command, or flag.
  The committed additive `ota.authority-launcher.systemd/v2` foundation at Protocol
  `cb5f539a4c3d9d75e2dd36692da8e69be5ba6e14`, Launcher
  `fddb10393aa0e79258ff048e32774a685d5fac04`, and Core
  `e3febf3d8d4226dc26ef20ddebaf1e1b23ef5fd3` publishes profile identity
  `sha256:c816a49e01120bf1f793aedcfec094ca0f23a8ee80f1c7e5bed4c2d9c797cb42`. It preserves V1 archive
  verification while replacing launcher-owned credential settings with producer socket metadata
  and the public verifier set. Core accepts only the exact registered V1 or V2 profile-ID/identity
  pair, and the Launcher collector assembles observations in canonical order while refusing any
  unavailable source. The committed live Linux job-principal preflight at Launcher
  `60a07055477ed27d6c82a2885fa9a87da94c6a70` and Core
  `591289f441cf9f0832d9605001854e3aa89f5df5` runs before repository opening or child creation:
  socket-bound pidfd, protected UID/GID mapping, exact `/proc` UID/GID slots, empty supplementary
  groups and inheritable/permitted/effective/ambient capabilities, and `NoNewPrivs=1`. Launcher
  `d437aed99daf4ae55e5d8299a99ce5df535fb07f` additionally retains and revalidates the protected
  broker-proxy pidfd before and after bridge traffic, with an orchestration regression proving peer
  exit during that window refuses. Those committed slices remain historical foundations for the
  complete committed collector and producer path described above.
  `ota up` evaluates
  unrelated blockers and the complete ordered prerequisite-instance preflight
  before broker contact; those prerequisite instances execute once inside the parent work unit.
  Authority launcher run
  [31257509444](https://github.com/ota-run/authority-launcher/actions/runs/31257509444) against exact
  Core `9244eb2bc6a44151c4172c0634ac44bdb216a65a` and immutable protocol
  `242685d5b7c3904681f1c71d734fbe2d41679dda` proves lost consume acknowledgement, fresh-session
  consumed-status recovery, incomplete old-transaction finalization, fresh authorization,
  exactly one selected-task execution, one valid recovery archive, and zero invalid archives. Run
  [31257511093](https://github.com/ota-run/authority-launcher/actions/runs/31257511093) reproduced
  the same complete workflow at the same launcher revision.
  Ordinary workflow readiness timeout, selected workflow instance, ordered prerequisite-instance closure,
  and runner-derived scope breadth are identity-bound; breadth retains only counts, categories,
  and hashed resource identities. The archive retains a public verification binding, not the live
  launcher descriptor. Signed protocol payloads retained for
  archive re-verification accept only bounded public-safe labels, never raw paths, descriptors,
  credentials, or secret provider material. Dry-run performs no launcher interaction and reports
  only `requires_live_authorization`; task processes do not inherit the protected descriptor. Receipts
  and archives bind the broker admission, attestation, prepared lease, consume exchange, semantic
  scope, and terminal transaction, and history re-verifies them against the protected binding.
  Replay, missing consumption, and carrier substitution refuse. Grant-required runtime and
  lifecycle proof now retain one transaction across their complete invocation and cleanup sets;
  terminal runtime-proof transactions bind the final proof verdict rather than the intermediate
  readiness state. Proof archives now retain repo-relative contract-snapshot references while
  preserving same-root compatibility for earlier absolute references, and archive emission now
  requires immediate reconciliation through Doctor's semantic loader. Authority launcher run
  [31033509379](https://github.com/ota-run/authority-launcher/actions/runs/31033509379) is green
  against exact Core `bd80b29d971ccd5ac8609d9fc767a491ff382ef8`. It proves one live broker run,
  expired/revoked/wrong-scope/replayed refusal, same-scope missing-launcher proof refusal, runtime
  archive consumption by Doctor, and completed runtime and lifecycle proof transactions. The
  lifecycle fixture uses a root-owned deterministic pressure control because Docker remains
  inaccessible to the job principal; it proves Ota lifecycle authority/finalization, not Docker
  provider behavior.
  The public operator guide now documents the fixed trust-store, separately protected bundle and
  sequence-state layout, and the provisioner/runner boundary without publishing usable authority
  material. Hardened-runner pressure now proves the carrier's bounded
  `current_process_filesystem_guarded` posture; the guide remains a preview because it does not
  claim provider-attested separation, reusable broker credentials, or one-use work-unit authority.
  Authority launcher run
  [31250919192](https://github.com/ota-run/authority-launcher/actions/runs/31250919192) against exact
  Core `257be61dd91799237357390b145be950f2fc6b3f` additionally proves broker-unavailable,
  bounded approval-timeout, local-cancellation, and conflicting-pending-response refusal before
  selected work. Each refusal retains byte-identical checkout manifests and no receipt state.
  Authority launcher dispatch
  [31260927337](https://github.com/ota-run/authority-launcher/actions/runs/31260927337) against exact
  Core `9244eb2bc6a44151c4172c0634ac44bdb216a65a`, with final merge-gate confirmation in
  [31261639968](https://github.com/ota-run/authority-launcher/actions/runs/31261639968), proves
  terminal cancellation before an undeliverable late approval, insufficient pre-wait attestation
  freshness refusing before authorization, and two executions of one broad three-task semantic
  scope consuming distinct work units with two valid archives. At that revision, V11.7 still
  required the hardened-launcher separation later proved by runs `31939777636` and `31953535665`.
  Provider attestation is optional stronger follow-on hardening. V11.22 is complete for its
  source-bound candidate and fail-closed closure-classification foundation. The internal
  candidate now binds the registered detector source inventory: fixed root markers, supported
  environment files, package-manager locks, direct workflow files, bounded .NET project paths,
  and detector-owned root extension markers. Inventory sources cannot escape the repository
  through symlinks. Its shared closure resolver records direct finite manifest commands with their
  executable requirement. It also resolves a finite declared package script only after binding the
  exact manager invocation, script name, script body, and executable graph. It follows only
  same-manager, same-manifest script references and rejects cycles, missing hops, composed shell
  bodies, indirect package scripts, and CI-only commands as unresolved. It retains unknown effects
  for every resolved graph. Resolved closures bind only runtime, tool, or toolchain requirements
  detected from the same manifest and mark platform `unknown` until a task-scoped platform source
  exists. `ota detect --candidate-out <root-relative path>` now publishes that same
  self-verifying review artifact from one command-owned immutable source snapshot, with
  descriptor-safe create-new collision refusal and no `ota.yaml` mutation.
  `ota contract apply-candidate` now supplies candidate admission and explicit `--write`: it
  self-verifies the reviewed artifact, requires the exact detector implementation, re-derives
  current source and existing-contract truth, and reconciles an identity-bound application
  projection over the reviewed base contract, exact normalized operations, and fully validated
  resulting contract identity. Candidates without a complete valid projection refuse admission;
  unrelated `unknown` or `unsupported` findings remain review state unless `--require-complete`
  is requested. `--write` locks the retained no-follow repository descriptor, rechecks current source and evidence,
  and atomically creates only a previously absent `ota.yaml` from the shared evaluator's returned
  validated contract; default `--write` never overwrites an existing contract, and semantic
  reapplication is a no-op. The explicit `--write --carrier git` path now admits a non-detached
  tracked `ota.yaml` that matches `HEAD` in both index and worktree, commits only the reviewed contract with expected-HEAD branch
  compare-and-swap, verifies the resulting worktree/index, and reports branch plus prior/resulting
  commit identities. It never pushes, rebases, amends, or changes unrelated paths. The registered
  `legacy_flat_toolchain_fulfillment_v1` upgrade emits a schema-v2 source-bound candidate and can
  use the explicit Git carrier after exact re-derivation. Repo-level legacy mutation flags now refuse
  before repository access; `detect --write` remains the temporary conservative create-new alias.
  `init --dry-run --json` now emits the read-only schema-v4 `init_starter_preview_v1` candidate,
  binding the exact starter preview to its immutable source capture and resulting contract identity
  without an application projection or write authority. The pre-commit Buzz observation is local
  only and has no retained artifact or hosted run, so it is not completion evidence. V11.22
  intentionally emits no inferred agent-safe proposal: maintainer-authored contract safety remains
  authoritative, and positive effect-backed promotion is deferred to V12's typed effect and
  realization evaluator rather than treated as a V11.22 closure gate;
  V11.22 does not consume crossing records as approval authority. V12 effect-bound
  refusal assurance had completed its bounded implementation while the formal real-repository
  pressure bar was still open. The final Plausible and Outline witness matrices now complete that
  bounded pressure bar; independent closure reconciliation does not widen the completed
  crossing implementation. See [V11.7](../planning/v11.7/plan.md),
  [V11.22](../planning/v11.22/plan.md), [V12](../planning/v12/plan.md), and the planned,
  inactive [V12.1 secret-delivery governance follow-on](../planning/v12.1/plan.md).
- completed V11.17 trusted replay-baseline regeneration: Core now has an additive
  `artifacts.<name>.replay` authority chain: explicit producer record, immutable
  recorded attestation, exact promotion, then replay consumption. A portable authority manifest
  binds the canonical recursive output set, producer receipt, source/contract identities, and the
  producer's V11.16 execution-boundary graph plus asserted-target and derivation-input closures.
  `read_only` refuses outside a runner-owned ephemeral container boundary and mounts a run-scoped
  snapshot outside the writable workspace for the full selected closure; `verify_unchanged`
  detects mutation after native or container replay and never claims prevention. Portable authority
  declares SCM review as its external selection trust root, not signer-backed provenance or
  Ota-verified reviewer approval. Bedrock now has an additive unsafe `record:baseline` producer and separate
  promoted offline consumers. Its promoted lane correctly fails closed with
  `OTA_REPLAY_BASELINE_UNAVAILABLE` until an intentional live recording is reviewed and explicitly
  promoted. Upstream [run 30268181240](https://github.com/vinimabreu/bedrock/actions/runs/30268181240)
  recorded an approved live-model candidate with attestation
  `sha256:0bfb61977a38310c7ee515a4de31cec5ca4198b7bb63fb1a1bd980f920a39b93`
  and source `git:bb0ace385cfe17c1a5c195f3cd20de60a446cea6`. Its aggregate score improved, but
  `top_product_by_quantity` became `stable_wrong`; it is intentionally unpromoted. The uploaded
  review artifact omitted the required producer receipt archive, so it cannot later support
  `ota baseline promote`; Bedrock must retain `.ota/receipts/` and make a fresh reviewed recording
  after that workflow fix. This is a Bedrock workflow-retention gap, not an Ota Core defect. Core focused
  record/promotion, JSON conformance, and published-schema tests pass. EventCatalog closes the
  independent non-model generated-baseline gate: [run 30198942717](https://github.com/bobaikato/eventcatalog/actions/runs/30198942717)
  preserves the ordinary native generator matrix on Ubuntu, macOS, and Windows and separately
  proves a fresh runner can consume the committed portable authority without local `.ota` history:
  Doctor admission, workflow dry-run, setup hydration, detached compiler consumption, and receipt
  archival all pass. CI never creates or commits that authority; its `scm_review` trust root remains
  an external delivery/review assumption, not Ota-verified reviewer approval. Source identity
  correctly excludes declared Langium outputs and refuses workflow-generated transient JSON until
  that evidence moves to `$RUNNER_TEMP`; the promoted consumer compiles the approved authority
  rather than comparing a newly approved baseline against Git `HEAD`. EventCatalog run
  [30226480354](https://github.com/bobaikato/eventcatalog/actions/runs/30226480354) completes the
  independent strict-replay pressure gate: ordinary generated-source lineage remains green on
  Ubuntu, macOS, and Windows, while explicit record/promotion and committed-authority consumption
  both run through the declared ephemeral container boundary with `read_only` enforcement. A
  credentialed Bedrock recording is now an upstream, intentionally reviewed adoption lane. Its first
  candidate remains unpromoted for a real `stable_wrong` regression and missing receipt retention;
  it is not a release gate for the general V11.17 model. Dagger exposed
  and now exercises the needed hybrid posture:
  `generated_source` retains an ordinary producer-dependent SDK check while a second task can
  consume the same output only through explicit promoted replay authority. The Core schema, Doctor,
  receipt, mutation guard, and strict runner boundary all derive that consumer distinction from the
  producer dependency. Its hosted pressure run [30179771523](https://github.com/bobaikato/dagger/actions/runs/30179771523)
  bootstrapped Core `b5b55e0e` and passed contract/discovery/dry-run admission, but the ordinary
  producer failed before record/promotion because Dagger's module-owned runtime resolves
  `protobuf-dev~32` against a mutable Alpine package index that no longer satisfies it. This is not
  an Ota replay defect or a reason to weaken the generator; it is an explicitly bounded external
  Dagger engine/module-runtime provenance gap. Dagger remains unsuitable for green generated-
  baseline pressure until its upstream runtime is made reproducible or Ota gains a Dagger adapter
  that can attest that tool-managed container state.
- completed implementation slice: V11.19 typed uv local-project hydration. The replay classifier
  now requires resolved source posture, declared lockfile identity, and clean local-project source
  identity before editable hydration can be acquitting; missing lockfile or source identity remains
  narrowing evidence. Dograh [run 30165303012](https://github.com/bobaikato/dograh/actions/runs/30165303012)
  proves nested editable Pipecat hydration with its full declared extras, ordered `dev` group,
  manifest, lockfile, and Git source identities, plus bounded PostgreSQL/Redis lifecycle execution.
  Marimo [run 30165304206](https://github.com/bobaikato/marimo/actions/runs/30165304206) proves the
  different editable root-project plus `test`-group shape on Linux and macOS; it has no `uv.lock`,
  so Ota records manifest and Git source identities while correctly retaining narrowing replay
  evidence. Both matrices bootstrap exact Core `19509754` and retain platform-specific evidence
  artifacts. Dograh's Dev Container, unpinned Node validator install, and GitHub-service versus
  local-Compose divergence remain bounded. Marimo's frontend/pnpm, Playwright, Docker, release,
  and Windows lanes remain repo-owned outside its selected proof.
- completed finite-command interaction pressure: omitted
  `tasks.<name>.command.interaction` resolves to `auto`, allowing terminal passthrough only for
  human native terminal execution. Explicit `forbidden` keeps every prepared closure step
  noninteractive, while `required` refuses before dependencies or workflow prepare/setup phases
  when the selected boundary cannot provide a terminal. Effective posture survives mode selection
  and orchestrator wrapping; dry-run JSON publishes the invocation-specific
  `terminal_passthrough`, `piped`, or `refused` resolution. Agent, captured, container, remote,
  and ordinary non-TTY CI execution do not acquire terminal capability. Workers SDK
  [run 30265519625](https://github.com/bobaikato/workers-sdk/actions/runs/30265519625) bootstraps
  Core `c5256d64f3060b30d40f07fa389b2bb16fc61b1f` and proves validation, Doctor, task discovery,
  dry-run JSON, non-TTY refusal before hydration, and agent refusal across Ubuntu, macOS, and
  Windows. Windows also proves a failed optional WSL shell probe cannot corrupt machine JSON.
  The real OAuth/account success remains intentionally external and not proved; the contract does
  not advertise container or remote execution for this terminal-auth lane.
- completed replay-input identity hardening: optional task `replay_inputs[].expected_identity`
  pins validate canonical SHA-256 values, surface missing or mismatched artifacts through Doctor,
  block dry-run/run/up before task startup, and preserve expected plus observed identity in the
  receipt evaluated-input carrier. Bedrock pressure proves matching frozen inputs through Doctor,
  dry-run, and real native plus container agent-safe execution.
- completed V11.20 policy-governed replay-input identity implementation: the shared
  `replay_input_policy` evaluator applies cumulative task/workflow rules over their exact selected
  closures, observes each task-qualified declared input once, and derives `deny > review > allow`.
  Doctor findings and JSON, dry-run, run, up, proof runtime, proof lifecycle, and
  admission-produced execution/refusal receipts reuse one command-scoped loaded policy snapshot
  and observed identity set across agent safety, claim assurance, replay policy,
  provisioning/effect findings, proof, CI projection, and receipt policy evidence. Runtime proof
  passes that same admitted authority to its detached child through a private temporary snapshot.
  Active policy load failures remain typed fail-closed admission evidence, and selected closures
  include recursive outcome hooks so a governed or mismatched hook cannot execute behind an
  admitted parent.
  Aggregate monorepo Doctor JSON retains the policy result for each selected member. Admission
  refuses before native provisioning, proof artifact creation, dependency hydration, service
  ownership, assertion execution, or task startup; unavailable observations and unreadable or
  mismatched declared pins fail closed, and hard-pin refusals retain the active policy evidence.
  Generic readiness receipts do not reconstruct policy after execution. Runtime proof evaluates
  its full selected proof closure, including seam observers and its selected negative control,
  before passing one preflight through every readiness diagnosis and the embedded Doctor artifact;
  lifecycle proof evaluates its exact prerequisite-plus-assertion closure before beginning a
  transaction. Missing pin coverage follows the rule's `on_insufficient`. Unknown selectors are
  contextual policy findings, not contract-validation errors. CI projection carries the exact
  active policy identity, applicable rule identities, canonical execution closure including
  recursive outcome hooks, and unresolved selector identities but
  no render-host observation; the provider checkout recomputes observed replay-input identities.
  Typed-effect CI projection also carries its complete non-secret policy decision in the projection
  identity and re-evaluates it from checkout bytes before provider setup or selected execution. Core
  unit, real-repo no-execution, JSON-schema, JSON-conformance, and projection
  checkout-re-evaluation regressions pass; copy-ready Core/external examples, canonical skill, and
  site reference are aligned. Connected public guidance is pinned at Site
  `7fa71e4dd4a4f1348e9b45af9060acc954ed7034` and Skills
  `610e801c9b32030d888b0c6d0118a5e70af0165a`; Examples require no change because this slice adds
  no contract-authoring shape. The focused CI boundary is immutable-hosted on Linux/x64 and macOS
  in [run 33173733814](https://github.com/ota-run/ota/actions/runs/33173733814) against exact Core
  `39d2f3964aec84a6e5ff5b0fdb19fa94ce27c8eb`. Both retained artifacts carry schema-valid
  compatibility-warn and explicit-deny projections, bind distinct projection and policy-decision
  identities after checkout policy drift, and retain the deny as `effect_policy_denied` before
  setup, execution, or durable logs. This remains an internal fixture: provider execution,
  mutation, positive receipts, archives, assurance, and independent real-repository behavior are
  not proved.
  Bedrock [run 30413944121](https://github.com/bobaikato/bedrock/actions/runs/30413944121)
  proves strict matching admission for four declared frozen inputs through native and container
  execution. Kylrix [run 30413944203](https://github.com/bobaikato/kylrix/actions/runs/30413944203)
  preserves ordinary unpinned compatibility while its dedicated strict-policy lane refuses
  Doctor, dry-run, real `ota up`, receipt, and `doctor --fix` before setup outputs are created.
  Both bootstrap exact implementation Core `f97b96cc`; later Core `d0e77a95`, `ff35a910`,
  `2d0b20fc`, `c85af3d2`, `6aaa063e`, and `4729e042` reconcile release-gate fixtures, generated
  reference truth, and hermetic test tooling without changing runtime behavior. Core
  [Release Gate run 30452821989](https://github.com/ota-run/ota/actions/runs/30452821989) and
  [Ota Readiness run 30452822253](https://github.com/ota-run/ota/actions/runs/30452822253) are
  green at pushed candidate `6538cc07`; docs-quality, Smoke, CodeQL, and cargo-deny are green on
  the same source. Bedrock live
  recording/promotion, Kylrix long-running runtime surfaces, and undeclared ambient inputs remain
  outside this bounded policy proof. No new Ota platform gap was exposed.
- completed V11.14 contract-claim assurance: the shared `claim_assurance` domain supplies the
  first additive `ota doctor --json` carrier for declared agent-safe tasks and workflow proof
  claims. It keeps maintainer
  declaration, derived V11.3 closure, policy-independent assurance, and policy decision separate;
  declaration plus closure remains `unknown` without non-self-origin evidence. Its first
  deterministic contradiction is a typed `reset_compose_service_volume` action that omits the
  exact `effects.adapter_state: compose_volume:<volume>` it mutates; opaque shell remains
  `unknown`. `ota proof runtime --json --archive` now creates a content-addressed proof-owned
  record bound to the terminal proof output, archived contract snapshot, clean source identity
  when available, resolved execution, target-platform, host-port, and normalized readiness-timeout
  scope, and explicit witness-only replay posture. The shared
  proof-breadth evaluator consumes only a matching immutable archive: matching failed proof is
  cited as `contradicted`; missing, stale, source-mismatched, or scope-mismatched evidence remains
  `unknown`. Ota-owned `.ota` runtime state is excluded from the source-identity cleanliness check,
  so a fresh archive cannot invalidate its own proof claim.
  Opt-in
  `policies.agent.claim_assurance` requirements now drive the same canonical `deny` or `review`
  decision through Doctor, `ota run --agent`, previews, and `ota up --agent`; default agent
  admission remains unchanged without that policy requirement. Generic deterministic workflows
  without a declared dependency seam can opt into this same assurance through
  `workflows.<name>.proof.claim: bounded`; Doctor reports `bounded_proof` as `unknown` until a
  matching immutable archive exists, then `supported`. Bedrock proves that transition on its
  offline replay lane without inventing seam or negative-control evidence; Lead Quorum proves the
  independent `unknown` path without an archive.
- committed finite command interaction capability (`6b2fa0ca`): structured
  `tasks.<name>.command.interaction` defaults to `auto`, so a native human TTY passes through only
  when available. Explicit `forbidden` preserves deterministic captured execution; `required`
  refuses before any selected task, workflow setup, or dependency work begins when no real
  terminal can be provided. Agent, container, remote, and ordinary non-TTY CI boundaries never
  acquire terminal interaction. The task JSON and dry-run JSON expose the resolved posture and
  invocation resolution; the copy-ready Wrangler OAuth example, canonical skill, public site
  contract reference, schemas, changelog, and regressions are aligned.
- completed V11.18 managed lifecycle-sequence proof: committed lifecycle admission (`10a14971`) and
  the first bounded executor (`d70ca67e`, qualified by `dd3b02cd`). `ota proof lifecycle` selects
  only workflow-declared manager services, leases manager-observed inactive state before start,
  starts in dependency order, reuses transaction-owned services for an optional post-readiness
  assertion, and tears down in reverse order. Typed JSON/schema output binds each record to the
  transaction; a command-only start carries `service_started_state_not_proved`, alongside the
  mandatory application-output and broader-repo boundaries. Focused regressions cover pre-existing
  service preservation and assertion-failure teardown. Runner-owned finalization and a local
  content-addressed lifecycle archive landed in `af74ca4a`: the archive binds
  the semantic contract snapshot, selected workflow/service scope, transaction records, and
  terminal verdict. The command now shares selected-workflow agent admission, mode resolution for
  prerequisite/assertion tasks, and monorepo member loading; service-manager controls remain on
  their declared boundary. Archive scope and reader verification landed in `3607b6a7`; the current
  correction binds the resolved service closure/mode and contract/source identity, verifies archive
  filename plus snapshot staleness, and records typed interruption finalization with exactly-once
  teardown regressions. Multi-service dependency rollback now proves reverse, exactly-once
  finalization after a later start failure or interrupted later start; the matching local archive
  verifies the full closure, service records, transaction, and finalization binding. A
  runner-observed readiness interruption now emits a typed `interrupted` transition before the
  same finalizer runs. A stop-command failure retains typed transition evidence and does not
  prevent other leased services from finalizing; an interrupted teardown with unproved manager
  cleanup is explicitly `incomplete_after_interruption`. Focused local runner/archive/schema
  validation is complete; the lifecycle-lock correction is committed in `b3a018a7`. V11.18 does
  not reopen V11.15 provider-neutral or GitHub projection: lifecycle proof stays a local command
  with dedicated provider-owned pressure workflows until a later slice defines lifecycle-specific
  adapter semantics. **Pressure provenance correction:** the remote Flagr branch
  `bobai/flagr-v11.15-deployment-pressure` at `8881fbe` does not yet declare
  `workflows.integration.proof.lifecycle`; it cannot substantiate the recorded lifecycle archives.
  Treat those local records as implementation evidence only, not Flagr pressure proof. Open WebUI
  now provides the Compose pressure side: its pinned `5ac1388784` matrix and lifecycle-control run
  prove declared Docker health probes as `service_readiness`, successful lifecycle finalization,
  and a controlled assertion failure without copied shell cleanup. Caddy closes the isolated
  lifecycle boundary locally: its current-Core archived container proof
  `sha256:8cb602aa552d653c3a5d3e465e934b7ed773b8305235aa7bf1775c274c65a27e` runs the upstream
  structured `caddy start` / `caddy stop` commands inside one transaction-bound ephemeral session
  and attests only engine-confirmed session removal as `boundary_terminated`. It explicitly retains
  `service_started_state_not_proved`, `application_output_not_proved`, and
  `broader_repo_completion_not_proved`; it never claims `manager_inactive` or host process absence.
  Caddy matrix [30102633474](https://github.com/bobaikato/caddy/actions/runs/30102633474) is green
  for regenerated native and container governance lanes against committed Core `6025187b`, but it
  does not invoke `ota proof lifecycle`; it is not isolated-boundary pressure proof. The dedicated
  hosted lifecycle run [30111427705](https://github.com/bobaikato/caddy/actions/runs/30111427705)
  is green against Core `3ffaf362` and binds the exact runner-owned boundary identity. The final
  hardened rerun [30124528078](https://github.com/bobaikato/caddy/actions/runs/30124528078) is
  green against Core `53ff07eb`, emits qualified proof, and archives
  `sha256:8985f57cd191e4d1db370122a6adb33a5f3a3fc649a289b2855dd2b48894de39` with exact session
  `container:docker:ota-ephemeral-43c71044194b0e05`. The final setup-failure correction passed
  [30125996749](https://github.com/bobaikato/caddy/actions/runs/30125996749) against Core
  `fc88d215`, emitting archive
  `sha256:c2dab2e7535589819f416159ca06b5384d599093e87546cc1aab1b242d8e3235` with exact session
  `container:docker:ota-ephemeral-3890811a19b2944d`. Open WebUI supplies the independent Compose
  readiness/teardown family. V11.18 is complete: implementation, pressure evidence, and final
  independent review passed with no release blockers.
- completed V11.16 fresh-boundary setup proof: `ota proof runtime --json` and archived runtime
  proofs carry a content-addressed `execution_boundary` graph. Native `ensure_virtualenv` plus a
  runner-recorded `.venv/bin/*` consumer, and frozen native pnpm hydration plus a declared local
  consumer, carry runner-attested precondition, producer, and `asserted_at` identities before
  deriving `cold_start_verified` or `persistent_state_reused`. The pnpm carrier binds its
  generated `node_modules/.modules.yaml` layout marker to the declared lockfile rather than
  claiming whole-tree hashing. The evaluator canonicalizes graph identity and rejects ambiguous,
  forged, stale, cross-scope, or causally mismatched edges before proof JSON or archives emit.
  Lead Quorum run `29742813235` proves fresh and reused virtualenv evidence; OrchardCore run
  `29697072972` proves an ephemeral typed .NET container closure; Athena run `29786128386` proves
  container Bundler fulfillment while retaining PostgreSQL lifecycle/output boundaries; and Kylrix
  run `29828933200` closes the native pnpm carrier with absent `node_modules`, a `setup` producer,
  a matching local `dev` assertion, and `cold_start_verified`. Provider state, databases, services,
  volumes, general container filesystem state, Windows virtualenvs, and uninstrumented package
  layouts remain `unknown`; V11.16 does not claim repo-global cold-start proof.
- completed V11.15 managed CI projection: `ota ci projection --workflow <name> --mode <mode> --target-os <linux|macos|windows> --json`
  now emits the provider-neutral governance lane with a semantic identity, merge-check identities,
  proof requirement, and provider-neutral ownership categories. The GitHub adapter consumes that
  object through one renderer powering `ota ci github render`, `check`, and atomic `sync`.
  It emits separate projection, render, and parsed-caller binding identities; generated content
  runs validation, doctor, safe discovery, agent dry-run, execution, receipt archival, and a
  declared runtime proof when the selected workflow owns one. Agent-safe lanes retain `--agent`;
  proof claims do not bypass agent admission, and proof-required lanes use one authoritative execution.
  Each unique contract refusal canary is now emitted as its own provider check with a stable
  `merge_check_id`; the GitHub adapter publishes the scope-qualified provider-check mapping so
  native/container and OS lanes stay independently requireable. The generated check invokes Ota's
  `--expect-refusal` runner boundary directly. Projection also carries selected-closure,
  provider-neutral `toolchains[]`; the GitHub adapter renders Go setup from the contract through
  an immutable Action revision and refuses unsupported required sources rather than relying on an
  ambient hosted-runner toolchain. Aggregate execution-mode admission now uses the same concrete
  member-closure rule as projection and task discovery, preventing a valid container projection
  from later failing before aggregate members run.
  Kylrix renders valid distinct native
  and container reusable lanes without collapsing its separately selected `sqlite-dev` runtime
  proof into `verify`; its committed caller/matrix preserves the existing native/container
  evidence. NopCommerce independently proves generated .NET verification on both native and
  container lanes: the native lane projects the declared .NET 10 toolchain through an immutable
  `actions/setup-dotnet` revision, while the container lane uses only the declared SDK image.
  Both lanes bootstrap Ota from the pressure contract, verify their projection identity, run the
  agent-admitted workflow closure, and archive a receipt in GitHub run `29686807594`. Strict
  V11.14 agent and proof assurance admission is evaluated before projection render/check; denied
  or review-required lanes return their canonical refusal rather than generating a green wrapper.
  Outline then exposed a projection/runner mismatch: a safe run task could be admitted while an
  unsafe setup phase was later refused by `ota up --agent`. Projection now shares the runner's
  ordered prepare/setup/run/attach admission roots, and Outline's unchanged `checks` workflow
  returns the same inspectable `requested_task_not_safe` refusal from projection and render.
  A clean Flagr deployment pressure clone then exposed a second renderer defect: finite workflows
  used readiness-only `ota up` as their only generated execution step. Projection now binds
  `run_execution` as `finite_task` or `service_runtime`; finite lanes retain a dry-run `ota up`
  admission preview, then execute their selected closure directly through `ota run --agent`.
  Compatible ephemeral container closure steps now share one runner-owned session, so typed
  hydration state survives into its finite consumer without leaking across CLI invocations.
  OrchardCore proves that .NET restore/build/test path locally in both native and container modes;
  its tag-triggered release CI remains provider-owned and untouched. Its pushed matrix and Caddy's
  independent green native/container governance matrix now satisfy the two-repository pressure
  target. Caddy also hardened the GitHub adapter's Go lower-bound projection: a valid one-sided
  contract range such as `>=1.25.1` now renders its explicit lower release through immutable
  `actions/setup-go`. Caddy's upstream start/stop shell smoke remains separately modeled and
  explicitly outside the generated build/test lane; Ota must not call that narrower lane full
  upstream CI equivalence until it can recover the lifecycle assertion without reducing it to a
  command-shaped approximation. The final V11.15 review passed focused neutral projection,
  GitHub renderer, JSON-conformance, and formatting checks; Kylrix plus OrchardCore/Caddy satisfy
  the two-repository pressure bar. V11.15 is complete.
  Projection identity now reuses the canonical normalized semantic snapshot identity used by
  receipts; omitted mode resolves from the selected task's effective contract default, while an
  unavailable explicit mode is refused. Denied provider-neutral JSON preserves the evaluated
  projection and typed refusal. Managed workflow paths reject symlink escape, and the neutral
  projection carries bootstrap posture, proof claim, and target-OS identity for the first GitHub
  adapter.
- V11.3 refusal-canary implementation is pressure-proven on Athena and Kylrix: `agent.refusal_canaries` names
  one task or workflow negative control, and `ota run --agent --expect-refusal <task>` or
  `ota up --agent --expect-refusal --workflow <workflow>` passes only when the agent-safety
  closure refuses before selected work begins. A policy-only denial is the failing
  `wrong_refusal_boundary` outcome. The contract never supplies an expected reason; Ota emits the
  runner-derived refusal and blocked receipt for later comparison. First-party docs/site/skill are
  aligned, and the released safe-agent-execution example carries the canonical refusal-canary
  pattern.
- completed `V11.10` replay trust refinement: `ota up
  --replay-baseline ... --json` now carries replay-authored baseline posture directly through
  `replay.baseline.last_known_good`, while declared static replay inputs remain receipt
  `evaluated_inputs[]` and Bedrock-style historical query traces stay separate as attested
  `witnessed_observations.query_traces[]`; plain-text replay output now mirrors the same trust
  split by rendering matched acquitting, narrowing, and pointer-only evidence separately from
  changed inputs, and hermeticity now requires at least one matched material runtime,
  dependency-resolution, or presentation anchor rather than over-reading same-contract reruns as
  hermetic; hidden-input replay failure now emits ordered `hidden_input_candidates` so operators
  can promote the next likely ambient class instead of reading one generic suspicion bucket
- completed V11.12 typed hydration provenance across two ecosystems: successful `ota up --json`
  records selected structured hydration lanes through typed `receipt.evaluated_inputs[]` hydration
  records. The record captures contract-declared source posture and runner-resolved feed identity
  before execution, preserving explicit `resolution: unavailable` when source choice remains
  ambient. Azure SDK for .NET proves config-backed NuGet identity; Lead Quorum proves explicit uv
  PyPI index posture across native and container lanes on Ubuntu, macOS, and Windows. Ota projects
  uv index truth through flags supported by both older and current uv releases. Replay treats
  unavailable hydration resolution as narrowing evidence, never as a hermetic dependency-resolution
  anchor.
- completed V11.9 governance reconciliation: Athena exposed a preview that named
  `not_run_reason: preflight_refusal` while reporting `refusal_occurred: false`. The canonical
  preview now carries the same refusal record, reason, and basis in both phases while preserving
  post-execution `state: not_run` because no execution began.
- completed V11.11 proof-boundary and seam-control carrier: `ota proof runtime --json` publishes
  qualified proof verdicts, scoped `not_proved` boundaries, provenance-aware seam evidence, and
  canonical negative-control records. Athena's Rails/PostgreSQL lane proves transaction-bound
  marker recovery and same-obligation fault control across its green matrix. Every marker-bound
  seam retains `dependency_output_shaping_not_proved`; invariant coverage proves the pairing and
  derived control projection. Generic `ota up` and ordinary receipts intentionally do not inherit
  this evidence because they did not execute the proof lane. The canonical runtime-proof example
  now demonstrates the same carrier end to end. Compose dependencies started by proof are cleaned
  on success and readiness failure, while services already running before proof are preserved.
  Validated dependency projections now name their canonical negative-control record and bind its
  exact failure-attestation digest; schema rules prevent `fault_tested` evidence from omitting or
  downgrading that validated projection, while Core now reconciles the canonical ID, dependency,
  obligation, and digest relationship before emission and archive loading. Archive reads derive a
  selected control from archived contract/scope truth and require exactly one canonical record and
  matching projection; other consumers must apply that same rule because JSON Schema cannot
  compare sibling values.
- completed V11.13 generated-artifact lineage: Dagger proves the generator path and EventCatalog
  proves an independent sibling-consumer closure. Contract-owned producer, output-path, and input
  lineage is validated, surfaced in task discovery, checked before consumer execution, and carried
  into receipts as pointer-only evidence without claiming freshness.
- completed the 1.6.24 release-readiness sweep: the active pressure set has no unresolved Ota
  platform gap, V11.11 proof evidence is propagated through the canonical and public examples,
  skill guidance, site reference, generated contract schema, and changelog, and the complete native
  `release-gate` passes. Lead Quorum's newest local-only pressure commit remains unpushed and is not
  part of the matrix-backed release claim.

- V11.10 Bedrock replay proves native baseline replay as `replay_verified` and `partly_ambient`.
  A container replay against that native archive correctly returns `replay_unavailable` with
  `baseline_scope_mismatch`: workflow, backend, provider, remote target, and lifecycle identity are
  required for `last_known_good`.
  A freshly archived container witness then replays as `replay_verified` and `partly_ambient` on
  the same container/ephemeral scope. Backend-scoped informational doctor notes remain visible but
  do not stale an otherwise same-scope witness.

## Recent Completed Slice

- Kylrix pressure exposed two connected Ota execution gaps and proved their fixes on the
  deterministic SQLite contributor lane. `launch.runtime_projection.adapter: nextjs` now
  projects `--hostname` / `--port` from the runtime listener into direct `next dev` launches,
  while validation rejects package-script wrappers that would make projection ambiguous. Dry-run
  input resolution now recognizes `ensure_env_file` output from the selected dependency closure as
  planned setup state on a clean checkout, while real execution waits for dependencies and then
  validates the rendered dotenv input. Published contract-schema coverage was synchronized for
  command runtime projection and already-shipped generated workflow-instance fields; the full
  examples gate now passes. Kylrix itself proves idempotent SQLite env materialization, agent-safe
  Vitest/lint/build verification, workflow preparation, archived receipt, and isolated native
  runtime proof. Its interactive Appwrite topology and credential/schema-provisioning paths remain
  explicitly outside this narrow proof.

- Kylrix also exposed a native long-running task UX gap: applications such as Next.js can exit
  non-zero after a user `Ctrl+C`. Explicit runner interruption evidence, or a raw signal before a
  clean completion, returns canonical exit `130` with an `interrupted` receipt and summary. A late
  raw signal cannot overwrite an already-established non-interrupt task or service failure.

- Dagger generated-SDK pressure exposed and fixed native source-managed tool activation: a
  release-asset tool was fulfilled and version-probed correctly, but the native shell task path
  discarded the managed PATH before executing its command. Ota now applies the resolved PATH to
  native shell execution and has a focused regression test. The narrowed Dagger contract proves
  release-asset fulfillment, workflow preparation, selected generator execution, generated-source
  lineage, a clean consumer diff, scoped doctor, and archived receipt locally. The selected
  closure requires Dagger v0.21.7 despite root `dagger.json` still naming v0.21.0; that is
  recorded as repo truth rather than hidden by the pressure contract.

- Task platform availability is now contract-owned. `tasks.<name>.only_on` uses the same
  `linux` / `macos` / `windows` vocabulary as prerequisite and context scope; runner planning,
  `ota run`, and dry-run preview refuse an unsupported dependency closure before side effects.
  `ota tasks --use` marks context- or task-unavailable modes non-callable, and doctor filters
  unavailable closures before probing their requirements. Athena pressure uses this truth through
  its Linux/macOS Ruby context and proves the expected Windows refusal rather than hiding it with
  a workflow skip.

- The current V11.10 refinement adds contract-owned
  `tasks.<name>.witnessed_observations.query_traces[]` for existing JSONL query traces. Ota
  validates immutable repo-relative trace paths, captures the selected closure before execution,
  and emits source identity, full run records, and divergent-subject summary under receipt
  `witnessed_observations`. It deliberately keeps the trace outside `evaluated_inputs[]` so
  historical observed behavior cannot be over-read as a current-run decision input. Bedrock's
  recorded SQL trace proves the narrow admission: three subjects diverge while stable repeated
  queries retain one identity across runs.
- The completed `V11.13` core cut makes generated source a named repo-scoped contract artifact:
  `artifacts.<name>` declares `kind: generated_source`, one producer task, output paths, and
  optional source inputs; consumers declare `requires_artifacts` and directly depend on the
  producer. Validation rejects dangling, overlapping, and dependency-disconnected lineage. The
  runner checks declared outputs after the producer closure and before consumers execute. Task
  JSON carries the producer map plus consumer references, and receipt `evaluated_inputs[]` captures
  producer/path/input lineage at issue time as pointer-only evidence, never as a freshness claim.
- EventCatalog pressure proved the first healthy generator and sibling-package consumer closure:
  typed `pnpm --filter @eventcatalog/language-server install`, Langium generation, and the
  downstream VS Code extension build. It also widened `prepare.source.filter` from its old
  browser-bootstrap-only boundary into a pnpm-owned dependency-hydration selector. The first
  sibling build failure identified real missing SDK and visualiser build dependencies; modeling
  those finite tasks made the final extension build pass without shell orchestration.
- Bedrock pressure proved the V11.10 replay-artifact shape on a deterministic offline NL-to-SQL
  stability harness across Ubuntu, macOS, and Windows: explicit script-test aggregation, committed
  SQL fixture replay, and the defended baseline gate all run in agent mode with no model key. Its
  live recording lane remains intentionally outside that claim because it reaches Claude, rewrites
  the fixture, and depends on an unpinned generic-pip requirements path that Ota does not yet own
  through typed dependency hydration.
- The completed V11.10 replay carrier distinguishes whether the selected baseline is still the
  last known good witness. `ota up --replay-baseline ... --json` adds
  `replay.baseline.last_known_good` with `replay_verified`, `stale_witness`, or `unavailable`
  derived from the replay result itself, so promoted archives no longer all read as equally
  current after drift or unavailable-baseline failures.
- V11.10 also names active execution governance as a replay-grade input.
  Receipts now capture a loaded org policy pack as `policy_ruleset_identity`, and replay treats a
  changed ruleset as named input drift rather than generic hidden-input suspicion.
- V11.10 also names declared env-source files when the selected lane
  actually resolved from them. Receipts capture `env_source_identity` without recording values, so
  replay can distinguish declared env-source drift from still-ambient process or policy env.

- The completed task-discovery UX batch renders closure-aware `Human Run`, `Agent Run`, and
  `Agent Policy` sections in `ota tasks` and `ota tasks --use`. It keeps the `ota-site` internal
  verification setup task agent-callable so its declared-safe public verification closures remain
  truthful without exposing setup in the default task inventory. Task mode rows now use stable
  `Container`, `Native`, then `Remote` presentation, show unsupported local planes explicitly,
  and recover native override support for container-context tasks without requiring a redundant
  task mode branch. `ota tasks --json` now carries the same canonical per-mode truth under
  `tasks[].use.modes[]`, while the existing `use.human` and `use.agent` remain selected-mode
  compatibility projections.
- Flagr pressure confirmed native and container Go module hydration, binary build, aggregate
  verification, and runtime proof. It also closed two task-discovery regressions: aggregate mode
  rows now inherit executable closure support, and Doctor no longer version-probes repo-owned
  command paths before their producer task exists. The previously open locally tagged Dockerfile
  image-build gap is now closed by first-class `action.kind: build_container_image`: contracts own
  the provider, Dockerfile, repo-relative context, and local tag without raw `docker build` glue.
  Lead Quorum proves the direct image build; Flagr carries the equivalent integration-image task,
  with its live build awaiting a healthy Docker daemon on the pressure host.
- `V11.11` contract-derived proof boundaries is implemented in Ota commit `e3bbdf02`.
  `ota proof runtime --json` now emits terminal `proof_verdict`, and Lead Quorum pressure proved
  `passed_with_unproven_boundaries` on the app lane across Ubuntu and macOS.
- V11.11 keeps that qualified proof boundary visible in human output too:
  `ota proof runtime` now renders concrete `Proof Boundaries` entries whenever `not_proved[]`
  exists, so external-network and broader-scope exclusions travel with the green proof instead of
  living only in JSON.
- V11.11 makes those proof boundaries machine-actionable too: each
  `not_proved` entry now carries an explicit `reason`, and the human proof render includes the
  same reason label for seam, adjacent-lane, and broader-scope exclusions.
- The completed V11.11 seam-evidence carrier on
  `ota proof runtime --json`: `dependency_evidence[]` now publishes runner-derived
  `level: reachable` only for declared service seams that are also on the selected
  workflow-owned required-service path and have structured readiness Ota actually owns. This
  keeps selected service reachability distinct from still-unproved exercised interaction.
- V11.11 keeps caller-side seam attempts separate from proved reachability:
  proof-derived DNS, auth, and loopback service failure signals can now publish additive
  `interaction_attempted: true` with `observation.origin: caller_side`, while the paired
  `dependency_exercise_not_proved` boundary tightens to `caller_side_only_evidence` instead of
  generic missing evidence.
- The same commit fixes detached native proof lifecycle ownership: nested `ota up --detach` leaves
  the service running for the outer proof to observe and clean up, preventing recursive teardown.
- V11.10 emits a runner-derived receipt-comparison artifact-trust record for matching
  semantic contract snapshots. It is `acquitting` for `contract_truth` only; lockfile/runtime
  artifact capture remains the next implementation cut.
- V11.10 captures declared lockfile-strict Node identity in
  `receipt.evaluated_inputs[]` at receipt authoring time: `pnpm-lock.yaml` for frozen pnpm and
  `package-lock.json` or authoritative `npm-shrinkwrap.json` for `npm ci`. It carries this through
  archived baseline and current receipt diff and labels only matching
  `declared_dependency_resolution` identity as `acquitting`. Directus and ota-site proved matching
  archived/current paths with the source-built binary; unrelated runtime findings remain separate.
- Lead Quorum is not yet the first hermetic replay target: its typed `uv pip_requirements` lane
  and Python range are real current repo truth, but not the pinned dependency/runtime pair V11.10
  needs. Treat that as a repo contract/replay-readiness gap, not a reason to weaken Ota evidence.
- V11.10 captures `runtime:node` through contract-local `node --version` on the
  same typed lockfile-strict Node hydration path. It is deliberately `narrowing` for
  `selected_runtime_version`, not an executable/image-digest acquittal.
- V11.10 adds the first immutable runtime-artifact carrier. Receipts
  recover literal digest-pinned Compose `image` values only for explicitly selected services in
  explicitly declared files and their declared Compose `depends_on` closure as
  `selected_runtime_artifact`; receipt diff treats a matching digest as `acquitting` only for that
  named artifact. Mutable tags, interpolation, inferred files, and unrelated stack services remain
  outside the claim. Immich pressure also exposed and fixed an Ota runner gap: Compose adapter-file
  preflight now resolves files relative to the same adapter `cwd` used by execution. The narrow
  Redis/PostgreSQL launch, status, and stop path passed locally with the source-built binary.
- Immich and Grafana confirmed the follow-on taxonomy need. `effects.network_kind:
  container_image_hydration` now owns registry-backed Compose image acquisition independently from
  package dependency hydration; `prepare.medium: container_images` requires this label, doctor and
  policy packs expose the same lane, and immutable image receipt evidence remains separate from
  the effect declaration.
- The same branch upgraded the direct `quick-xml` dependency to `0.41.0` after `cargo deny`
  surfaced the two XML denial-of-service advisories in `0.38.4`; the NuGet feed-provenance parser
  uses the current XML 1.0 attribute-normalization API and its focused tests pass.
- Grafana confirmed the receipt carrier on a mixed Compose stack with locally built, mutable, and
  digest-pinned services. The selected observability lane records four explicit digest-pinned
  services plus `tempo-init` through Tempo's declared `depends_on` closure, while excluding
  unrelated built and mutable stack services. This exposed and fixed the closure-recovery gap in
  Ota rather than leaving the init image absent from a selected runtime receipt.
- The same Grafana pass exposed and fixed a doctor semver gap: whitespace-separated compound
  ranges such as `>=1.26.3 <1.27` now use the canonical normalized semver path while preserving
  Ota's established shorthand comparator behavior.

## Handoff To The Next Chat

Start by reading `AGENTS.md`, this file, the canonical Ota skill, and
`docs/planning/v12.1/plan.md`. Then inspect the actual Core, Protocol, Launcher, Site, Skills,
and Examples worktrees before editing.

The sole active implementation version is V12.1, Step 7. The protected Linux/X64 capability and
endpoint compatibility gate is closed by the exact retained runs and artifacts recorded above.
Those runs prove bounded protected capability derivation, signed public projection reconciliation,
selected execution, and terminal cleanup. They do not prove a real GitHub OIDC request, Google
provider contact, secret materialization, process-environment injection, positive provider
evidence, Step 8, V12.2, or general agent/repository governance.

The private authority-snapshot bridge is now committed in Core. Protocol `d1d1fd4` supplies the
record-only prerequisite. Core `15f59d87` adds private request/response reconciliation,
`8c22a18a` verifies the signed protected snapshot and retained bytes, and `f6d4633e`
independently reconstructs one provider-free Step 1-6 transaction candidate from the exact
challenge-backed invocation context and canonical selected `RunPlan`. Candidate reconstruction
re-derives binding/source, profile, implementation subject, provider tuple, effects, protected
policy authority, evaluation, and dry-run truth. It refuses substituted authority or context and
selected roots outside the exact native Linux transient structured-command boundary. The final
batch passed focused release compilation and regression suites, first-party synchronization, and
independent review. Site, Skills, Examples, Learn, FAQ, Glossary, contract schema, and public JSON
remain unaffected because no operator-facing route or vocabulary was introduced.

The bridge is not yet a production transaction. Launcher does not yet relay the protected snapshot
over the selected child's retained session, Protocol does not yet bind that snapshot into an
additive V2 transaction-binding exchange, and Core does not yet retain and consume the resulting V2
binding exactly once. Provider contact remains blocked.

Next action: implement and independently review Protocol's closed snapshot-bound V2
transaction-binding request, binding, response, identities, and cross-record reconciliation. After
that immutable Protocol commit, pin it in Launcher and Core, implement the same-child/session relay
and one-use Core guard, and run the exact protected Linux/X64 service path before considering the
separate provider-contact slice.

Do not touch the unrelated untracked `docs/pressure/agent-authority-signals.md` unless the user
explicitly asks.

## Working Rules

- Update this handoff immediately when the active step, blocker, next action, or proof status
  changes.
- Compact completed narration only after the coherent batch is committed, independently reviewed,
  and all affected repositories are reconciled. Remove detail only when a commit, plan, changelog,
  specification, pressure record, or retained evidence artifact durably owns it.
- Read the canonical Ota skill before Ota-specific work.
- Use `references/pressure-testing-protocol.md` for every pressure pass.
- Make and record the connected-surface decision for core docs, examples, skills, and site.
- Use released Ota versions for released proof; use the active branch only for explicit unreleased
  pressure testing.
