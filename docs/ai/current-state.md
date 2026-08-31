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

- branch: `1.6.27-implementation`
- released baseline: `v1.6.26`
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
  export safety remain unproved or outside V12. V12.1 remains planned and inactive; V12 closure
  does not activate it, and its own gates require the released V12 surface plus one
  concrete late-delivery provider case.
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
  replace the exhaustive local identity-domain regression matrix. V12.1 onward and both cross-cutting plans remain
  planned and inactive. The reference Example correctly requires Ota `1.6.27`; the source-built
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
`docs/planning/v11.7/plan.md`. Then inspect the actual worktree state in Core, `ota-site`,
`ota-run/examples`, and `/Users/bobai/Workspace/Ota.run/skills` before editing.

V11.21 and the bounded V11.7 OSS audited-crossing slice are complete. V11.7 pressure includes the
green GitHub-hosted missing-authority refusal and pre-provisioned Linux/x64 VPS live, expired,
revoked, and out-of-scope carrier matrix. The Unix
launcher-session broker carrier is implemented for governed `run`/`up`. Its initial hosted
live/refusal/proof-wide pressure set is green in authority-launcher run
[31033509379](https://github.com/ota-run/authority-launcher/actions/runs/31033509379) against exact
Core `bd80b29d971ccd5ac8609d9fc767a491ff382ef8`. Run
[31250919192](https://github.com/ota-run/authority-launcher/actions/runs/31250919192) against exact
Core `257be61dd91799237357390b145be950f2fc6b3f` additionally proves broker-unavailable,
approval-timeout, cancellation, and ambiguous-response refusal without selected work, receipt
state, or checkout mutation. Run
[31257509444](https://github.com/ota-run/authority-launcher/actions/runs/31257509444) against exact
Core `9244eb2bc6a44151c4172c0634ac44bdb216a65a` and immutable protocol
`242685d5b7c3904681f1c71d734fbe2d41679dda` proves bounded lost-acknowledgement recovery without
resuming abandoned work, followed by one fresh authorized execution and a valid recovery archive.
Authority launcher dispatch
[31260927337](https://github.com/ota-run/authority-launcher/actions/runs/31260927337), with final
merge-gate confirmation in
[31261639968](https://github.com/ota-run/authority-launcher/actions/runs/31261639968), closes late
approval after terminal cancellation, insufficient pre-wait freshness, and repeated broad-closure
work-unit pressure against the same exact Core revision. Stronger provider-attested separation
remains open. The current branch now implements the additive runtime-boundary attestation v2
verifier and archive path against immutable authority-protocol
`bff47c2c79b145831a3b411614301d7e09d6f377`: strict binding/payload/domain branches, exact
content-addressed protected-launcher profiles, signed launcher-session configuration identity,
disjoint attestor and broker authorities, and no reinterpretation of v1 evidence. Complete v2
evidence derives only `protected_launcher_attested_one_use`; v1 retains
`launcher_attested_one_use`. Core adversarial regressions cover downgrade, profile mutation,
missing/reordered/failed observations, required-identity removal, full one-use consumption, and
archive re-verification. Authority-launcher run
[31269597378](https://github.com/ota-run/authority-launcher/actions/runs/31269597378) is green
against exact Core `787ac35f7d0195d2adae85e1113e26ce4a30acc2`, protocol
`bff47c2c79b145831a3b411614301d7e09d6f377`, and launcher
`01efd331ca0d4dcf2f8899512b1e3705fc649c6d`. It proves signed challenge-bound observation of the
constrained Ota child, disjoint broker/attestor keys, one-use live/refusal/recovery behavior,
distinct catch-all work units, and archive-valid runtime/lifecycle proof transactions. This is
bounded pressure-peer conformance: the GitHub workflow controller provisions the root-only fixed
test authority, so it does not prove independently administered provider/launcher separation.
The current committed V11.7 gate at Protocol
`574563d1f69a674960d0b3228c5a13b13bc42c19`, Launcher
`13bf6db71610b86c81a251f440b80b9b8947a67d`, and Core
`31fa95b4d28a8a4971ee3fd65c841d40e54ac4d9` completes the Linux-only
`systemd_protected_launcher/v1` execution-disabled adapter through the full closed collector and
separately credentialed producer. Local ARM64 OrbStack PID 1 pressure first proved exact signed V3
admission, authorization-request observation without forwarding, terminal boundary cleanup,
pre-authorization drift/refusal controls, and crash-after-scope recovery. Immutable Linux/x64 run
[31530832876](https://github.com/ota-run/authority-launcher/actions/runs/31530832876) repeats that
bounded gate against exact Launcher `c69ad3afc6afef0e260a7eeaa4f7340971db50af`, retains
cursor-isolated refusal and cleanup evidence, and binds the same exact Protocol and Core revisions.
Core re-derives the exact
V3 launcher/job profiles, rejects schema-2 or legacy-profile evidence inside a V3 binding or signed
payload, and published schemas enforce the same branch. Historical V1/V2 evidence remains readable
only through its original carrier/schema branch. That hosted run closes execution-disabled V3
attestation admission. Follow-on immutable Linux/x64 run
[31561247605](https://github.com/ota-run/authority-launcher/actions/runs/31561247605) binds Protocol
`6a92d8db9d089e44d1980f1871bf6e90eccb9960`, Launcher
`77ab20aa6ed5e3dd42cc6815ba2de7cd36d543bf`, and Core
`b71b78ca33ea2edd7bb03ceb66c5e1e104217cd9` while proving execution-disabled signed-decision
admission, typed negative outcomes, exact relay evidence, terminal cleanup, and crash recovery. No
one-use lease consumption, selected execution, crossing receipt/archive, independently administered
provider/launcher separation, or provider-attested carrier existed at those immutable revisions,
and that run does not make `provider_attested_one_use` true. The later immutable one-use
  consumption and selected-execution gates are green. The pressure-proven portable-finalization
  batch binds immutable Protocol `3e912f721ba9673090d14bcf5f88a2ee27a6b58a`, Core
  `cf3114f3d96d5c030c748a12b2e359586f0ded8c`, and Launcher
  `6954a39aefd35b8df648534a6028c0206c0372f9`; it binds launcher-owned transaction schema v3 into
the signed consume exchange. That transaction requires broker-archive schema v2 and portable
finalization verification without
  invalidating historical transaction v2 archives. It now
  fsyncs protected recovery state before active-slot deletion, survives every modeled intermediate
  stage until exact sidecar acknowledgement, retains the exact terminal until a separate
  identity-bound acknowledgement, and carries separate producer signatures for cleanup
  and the exact receipt-archive/crossing-transaction association. Core durably writes the exact
  receipt archive before publishing launcher completion. The root launcher reopens it through the
  execution-principal repository descriptor, verifies owner, content, and transaction identity,
  and atomically publishes the root-owned sidecar; the job principal only acknowledges the exact
  result. Core publishes the archive with atomic create-new semantics and file/directory sync, while
  the launcher requires execution-principal-owned `.ota` and `.ota/receipts` directories at mode
  `0700`. The signed launcher profile now binds `CAP_DAC_OVERRIDE` in its exact bounding set, but
  not ambiently, so the root launcher can traverse that hierarchy; protected signing-key paths,
  explicit writable roots, owner/mode checks, and signed archive identity remain mandatory. Core
  then re-verifies both signatures and all identities in local archive regressions.
  The pressure-only client remains a proof harness,
  not a production attachment surface. Crash recovery now distinguishes live schema-v1
  finalization, where the launcher directly observed and reaped the child, from schema-v2
  `recovered_absent_completion_bound`, where a restarted launcher proves absence against Core's
  durable completion without claiming an observed exit or child reaping. Immutable Linux/x64 PID 1
  [run 31758094819](https://github.com/ota-run/authority-launcher/actions/runs/31758094819)
  passed for the corrected batch against Protocol
  `3e912f721ba9673090d14bcf5f88a2ee27a6b58a`, Core
  `cf3114f3d96d5c030c748a12b2e359586f0ded8c`, and Launcher
  `6954a39aefd35b8df648534a6028c0206c0372f9`. The artifact reports zero terminal
  active slots, finalization journals, and scopes across 21 boundaries; one valid and zero invalid
  archives for positive execution and all three terminal crash-recovery points; consumed one-use
  authority; unchanged refusal/crash worktrees; and separate root-owned cleanup-finalization and
  archive-attachment issuance records.
  The production operator attachment and protected-history surfaces are implemented and immutable
  across Protocol, Launcher, and Core. The installed
  `ota-authority-systemd-client` reuses the existing untrusted invocation request without exposing
  pressure controls. Before terminal acknowledgement, Launcher freezes and durably publishes the
  exact receipt archive, referenced immutable contract snapshot, and signed finalization sidecar as
  three root-owned content-addressed blobs plus one catalog entry. Its separate history service
  admits only the exact installed non-root client under the bounded operator profile, retains and
  rechecks the pidfd-bound executable, process posture, working-directory instance, and peer
  identity, and derives repository/catalog selection only from the protected mapping.
  `ota receipt --history --source systemd_protected_launcher` is the explicit Linux Core consumer.
  It accepts no path, `--file`, or `OTA_FILE` redirect, optionally selects one exact archive content
  identity, requires a complete bounded manifest, and never falls back to local history. Protocol
  entries bind the ordered archive, contract-snapshot, and sidecar object identities without a
  circular hash dependency. Launcher owns protected storage and producer-signature verification;
  Core reconstructs the exact three objects and remains the sole semantic archive verifier. JSON
  output distinguishes `local_archive_directory_observed` from
  `complete_selected_catalog_snapshot` and carries the bounded operator, repository, catalog, and
  per-entry identities only for the protected source.
  Protocol `04a199a1eddd72b5b61958e0fe7f2d4e662e05cf`, clean source-built Core
  `d9d424168b1c1dad48351651c610789e54f74dcf`, and Launcher
  `c80828aa7b64a4bb8c1d9957d937d4fae4d70828` passed immutable Linux/x64 PID 1
  [run 31823037642](https://github.com/ota-run/authority-launcher/actions/runs/31823037642).
  Its production-client path completes selected execution and re-verifies one valid protected
  archive with zero invalid archives, one catalog entry, and three content-addressed objects. The
  paired matrix proves one-use authority, refusal, drift, failure, interruption, replay, and crash
  recovery with exact cleanup and unchanged refusal worktrees. Retained artifacts contain public
  verifier and semantic identities but no private signing material. This closes the production
  invocation and protected-history pressure gates. The workflow controller provisioned the root
  authority stack, so that run did not prove independently administered launcher separation or
  provider-attested authority. Examples remain unaffected because no contract-authoring shape
  changed.
  Launcher `ea7480e8d8b8aa214c5602628fb6dfa6382e2088` carries the reviewed consumer-only
  self-hosted workflow and administrator runbook used for the separate gate. Provisioning
  binds admission to the exact administrator-installed
  GitHub runner unit, derives Core plus the provisioning Launcher's own and linked Protocol
  revisions from installed build artifacts,
  recursively rejects repository truth writable by the job principal, and emits a non-secret
  installation-evidence copy beneath a root-protected parent chain. A static substring regression
  is only drift detection for the committed workflow; it is not enforcement against equivalent
  job code. Protected systemd identities and filesystem authority provide that boundary. No
  prepared-runner authority write occurs until Launcher has identity-bound an inactive/dead
  canonical runner, an empty job/execution process inventory, and fresh managed authority state;
  every existing managed-path ancestor must also be root-owned, non-writable, and alias-free;
  the exact runner unit is gated on the final protected installation-evidence publication, and
  public installation evidence carries that observation. Immutable Linux/x64 PID 1
  [run 31939777636](https://github.com/ota-run/authority-launcher/actions/runs/31939777636)
  is now green against Protocol `04a199a1eddd72b5b61958e0fe7f2d4e662e05cf`, clean source-built
  Core `634a2c169e083da4e02abd72a7bf29ae388ddf3d`, and Launcher
  `ea7480e8d8b8aa214c5602628fb6dfa6382e2088`. The retained public evidence binds the inactive,
  process-free prepared runner; distinct job/execution principals; fresh 32-path authority
  namespace; exact installed binaries and units; one consumed work unit; completed selected
  execution; child reap; scope, cgroup, and active-slot removal; and one valid protected archive
  with zero invalid archives. No private key or reusable credential material is retained. This
  closes the independently administered positive hardened-launcher branch.
  The separate administrator-owned reboot/fault-recovery matrix is green in immutable Linux/x64
  PID 1 [run 31953535665](https://github.com/ota-run/authority-launcher/actions/runs/31953535665)
  against Protocol `04a199a1eddd72b5b61958e0fe7f2d4e662e05cf`, clean source-built Core
  `e49f21ee77e522a614a776bcf17c9f9be16c8a90`, and Launcher
  `a348a13fd60b067266013cf8a0f047bbe274fd81`. Its consumer-only artifact independently reconciles
  the execution-completion, finalization-intent, and terminal-recorded reboot cases with exact boot
  transitions, three expected and valid protected archive identities, zero invalid or legacy
  archives, unchanged repository manifests, complete terminal cleanup, and no private authority
  material. These runs satisfy V11.7 through the independently administered hardened-launcher
  alternative. Provider attestation remains optional stronger evidence rather than a completion
  gate.
Provider-specific attestation remains a separate stronger profile rather than an implied property
of the systemd carrier.
Each terminal transaction is bound to a fresh runner-generated proof execution identity, and
ordinary post-admission failures finalize explicitly; local content addressing remains integrity
reconciliation rather than tamper-proof storage against the same host user.
V11.22 is complete for its source-bound candidate and fail-closed closure-classification foundation.
Its first implementation checkpoint demotes task-name, wrapper, opaque-shell, CI-fragment, and
agent-guidance signals from agent authorization: they may remain review evidence, but cannot emit
`safe_for_agent` or a starter executable agent boundary. `ota detect --candidate-out` now writes
the self-verifying source-bound review artifact without writing `ota.yaml`; `ota contract
apply-candidate` independently rebuilds its exact application projection and validated resulting
contract identity. Without `--write`, admission is dry-run only. With `--write`, Ota locks the
retained no-follow repository directory, re-derives current source/evidence, and atomically creates
only a missing `ota.yaml`; existing contracts require the explicit Linux/macOS Git carrier, which
scrubs caller Git routing state and disables configured helpers, commits only the reviewed contract
with branch-ref compare-and-swap, and verifies the materialized worktree. A matching resulting
contract is an explicit no-op. The non-default publication fault adapter remains unavailable to ordinary builds,
while `candidate-publication:faults` and the non-Windows Release Gate permanently exercise
concurrent target creation, pre-publication cleanup failure, and post-publication durability
uncertainty. A missing projection refuses admission;
unrelated `unknown` or `unsupported` findings remain visible review state unless
`--require-complete` is requested. Repo-level legacy mutation flags now refuse before repository
access with `detect_legacy_mutation_removed`; `detect --write` uses the versioned
`detect_conservative_first_contract_v1` profile and the shared locked create-new carrier, and its
successful JSON records the exact `write_candidate` identity, schema version, and profile. Rewrite
and removal remain unavailable until the candidate model can carry them honestly. Required
external pressure is complete. A clean local macOS pressure pass against
`block/buzz@2d280376ad36134cec1f23bead6d866d30bed147` with Core
`76dd13417131978b894e34194842f2ab55e5cb20` exposed and fixed one candidate-domain defect:
an unresolved new `setup` wrapper left `tasks.setup.internal` independently applicable, which
prevented projection of unrelated contract truth. New-task metadata now inherits the execution
change's fail-closed disposition unless the task already exists. The corrected candidate is
byte-stable across repeated capture, proposes no agent-safe task, retains unresolved Just and shell
closures as `unknown`, dry-runs and creates one valid missing contract, reapplies as a semantic
no-op, refuses selected-source drift as `candidate_stale`, and refuses residual findings under
`--require-complete`. Its candidate, projection, and resulting-contract identities are respectively
`sha256:9459433bcb349d227dcb78763a3d2bd035a4c47f120e4b1edb73241d60b29333`,
`sha256:f3adc3ffa7e07440835dd47962ce73d0ad060945b539e75b133fc711512b44d2`, and
`sha256:d0e5a4860150ea9321a2813c3e9fd7d6c570c03b69801bb0b94e13de25af04c4`.
The generated contract validates and exposes zero safe tasks; Doctor remains locally `not_ready`
because the host's resolved `pnpm` cannot report its version, plus bounded hygiene and CI-drift
warnings. This is retained local product evidence only: it does not prove native/container task
execution, hosted pressure, or complete Buzz governance. The artifact
binds a required content identity for every inventory entry, every normalized evidence tuple,
structured contract-path segments, canonical semantic proposal values, existing-contract
conflicts, and explicit closure facts. Equivalent existing truth is omitted before publication;
dotted task names remain exact map keys and typed commands reconcile against equivalent detected
invocations. Indexed environment-source fields traverse arrays, and explicit schema-default command
posture reconciles with omission. The artifact cannot authorize a contract write or agent-safe
task. Candidate JSON distinguishes artifact publication from contract mutation through
`candidate_published` and `candidate_publication`; durability uncertainty never reports the
artifact as absent. `ota contract upgrade --candidate-out` now publishes the registered
legacy-flat-toolchain migration as a schema-v2 review artifact and `apply-candidate` dry-runs it
against current source truth. Its approved existing-contract application requires `--write --carrier
git`; ordinary `--write` remains create-new-only. Core docs, Site, the canonical Skill and installed
mirrors, and Examples now carry the same operator workflow. The connected propagation is committed
and pinned: Site `fca59417df3c56c31719d8ebb14d133456fe1503`, Skills
`ac8e89694f92d4f1418c7d13d1f105707f689503`, and Examples
`89467f3ad0751b5613a57382553c1a1b38dab4c7`; Core records the Site and Skills revisions as
synced rather than waived. Init and detect candidate output changed, while the `ota.yaml`
contract-authoring shape did not.
Immutable Caddy candidate/application pressure is green across Linux, macOS, and Windows in
[run 32742495306](https://github.com/bobaikato/caddy/actions/runs/32742495306) against Core
`850ac767c8be582fc5a804b89f5ed2a781bbbbee` and Caddy pressure revision
`e9f45dbb0b9bdd2c3ff598b3667f49c3ba44c9dd`. Linux and macOS independently produced the same
candidate, application-projection, and resulting-contract identities; retained an inferred Caddy
test command as `unknown`; refused disputed project truth as `candidate_conflict`; refused the
residual finding under `--require-complete`; preserved the existing contract under ordinary
create-new `--write`; and used the explicit Git carrier to commit exactly `ota.yaml`. Windows
completed read-only detection, refused durable candidate publication without leaving an artifact,
and returned `candidate_write_unsupported_platform` before loading a deliberately missing
candidate or mutating the contract. This exposed and fixed one real Core portability defect: the
Unix-only Git materializer had remained reachable from a Windows compilation path. The result is
candidate/application evidence only. It does not prove Caddy build, test, server, runtime,
lifecycle, container, release, network, or repo-global governance, and it must not be combined with
earlier Caddy runtime/OCI pressure to imply one continuous proof.
Immutable GitButler candidate pressure is green across Linux, macOS, and Windows in
[run 32749052846](https://github.com/bobaikato/gitbutler/actions/runs/32749052846) against Core
`996090390d8544908bbb222614ece28f4dca8b4c`, pressure-controller revision
`f208f8d1d0e9da7fc7e1b32c6e58288e97afb80d`, and clean upstream fixture
`gitbutlerapp/gitbutler@2068a7811629950c05bd6f17429c5f2454f8ef4f`. Linux and macOS produced
byte-identical candidates with candidate, projection, and resulting-contract identities
`sha256:5d3211d1a3e8a57702d71dfec0abc6a549b755379024ccfabe6f56d033ef6fa7`,
`sha256:064b9aed754ed8c7d57529327cb0ce2f4fbf03fc05a965971fd5316aadb425be`, and
`sha256:7d409f2dcebd3900a3c625af6086077cabe743df095c95ee6f3d77e640474c39`.
The candidate retained 43 unresolved findings, kept 12 selected package/Turbo/Tauri/Playwright
tasks `unknown`, proposed zero agent-safe authority, admitted only the unrelated applicable
projection in ordinary dry-run, and refused under `--require-complete`. Windows completed
read-only detection and refused durable candidate publication without leaving an artifact. The
first hosted run exposed a real cross-platform identity defect: case-insensitive macOS lookup
reinterpreted GitButler's lowercase `claude.md` symlink and uppercase `Makefile` as separate
registered paths. Core now requires exact spelling for every registered source-path component;
the retained Linux/macOS candidates have identical bytes after that fix.
Immutable BAML candidate pressure is green across the same matrix in
[run 32749052420](https://github.com/bobaikato/baml/actions/runs/32749052420) against the same Core,
pressure-controller revision `72e1bcf5e12e6c755de747db2330ef4e1f9f2e41`, and clean revision-pinned
upstream fixture `BoundaryML/baml@a50430fba33012bea9a740ab0466c10697050678`. The checkout is
explicitly recorded as Git LFS pointer-only. Linux and macOS produced byte-identical candidates
with candidate, projection, and resulting-contract identities
`sha256:042392f506455ffc4e14d9c4c015c954cd148fe0e29268c76bc366d5c0cb2ab8`,
`sha256:fa9134aa371de6e15772da2aa0071035b02656a9a612ba1b25590e28608c7584`, and
`sha256:f72f4afe88046ec96207250ebe8200606fb9402228c146a99896618c50c12f03`.
The candidate retained 32 unresolved findings, projected zero tasks, proposed zero agent-safe
authority, and refused strict application; the two reusable-workflow commands did not become
runnable truth. Windows again remained read-only. These GitButler and BAML runs prove candidate
capture, cross-platform identity, conservative non-promotion, dry-run admission, and strict
refusal only. They do not prove either repository's build, tests, desktop/runtime behavior,
complete LFS checkout, contract approval or write, task execution, lifecycle, release,
deployment, or repo-global governance.
Immutable Atuin distinct-lane pressure is green across Linux, macOS, and Windows in
[run 32753440076](https://github.com/bobaikato/atuin/actions/runs/32753440076) against Core
`a4c38c7224fcc49efc5eb2c0b4e89881c9236d23`, pressure-controller revision
`0d641aea0598a397bd058cc2d33400fad1a65e14`, and clean upstream fixture
`atuinsh/atuin@824c8716d82bed774ee6a83c683087ae77715814`. Linux and macOS produced
byte-identical candidate artifacts with candidate identity
`sha256:68a5208d2cf8614842cca6fcbc353230ac474384f589cc4ed71d41ce5401ef12`
and byte digest `bfaf21e1179180729362641a4c2042e47fe10f90af6a0e9211831cafac69f556`.
The detector retained `check`, nightly `fmt`, `test:unit`, and PostgreSQL-backed
`test:integration` as four separate unresolved changes. Their closure evidence preserves the
repository-selected Rust `1.98.0` posture, command-selected nightly toolchain, matrix-unknown
platforms, Linux integration platform, PostgreSQL service image, and `ATUIN_DB_URI` requirement
name without treating CI as authority or exposing the environment value. The candidate contains
9 unknown and 5 applicable findings, proposes zero agent-safe authority, emits no application
projection, and refuses application as `candidate_incomplete`. Windows observes all four lanes
through read-only detection and refuses durable candidate publication without leaving an
artifact. The run does not prove Atuin compilation, formatting, tests, PostgreSQL state, sync
behavior, shell integration, runtime, release, deployment, or repo-global governance. The named
Atuin, GitButler, and BAML detector-pressure matrix is satisfied. Immutable hosted
[Buzz pressure](https://github.com/bobaikato/buzz/actions/runs/32795701879) now proves source-bound
candidate creation, explicit create-new application, and matching no-op reapplication against clean
upstream `block/buzz@2d280376ad36134cec1f23bead6d866d30bed147` on Linux and macOS. Its artifacts
are byte-identical across platforms, create only `ota.yaml`, retain residual `unknown` review
state, and propose no agent-safe authority. Immutable hosted
[Flowise pressure](https://github.com/bobaikato/Flowise/actions/runs/32794430600) now proves the
registered legacy-flat-toolchain upgrade and Git-carrier application against historical revision
`90121fac54e234ca83e2a85c435354af1df8ac8f`, with byte-identical artifacts, unchanged before/after
semantic contract identity, a commit limited to `ota.yaml`, and matching no-op reapplication on
Linux and macOS. Neither run executes repository tasks or approves candidate changes. Local Buzz
pressure exposed and Core now fixes one evidence-manifest defect: after the first write, a retained execution closure
could reference package-manager evidence whose unchanged top-level proposal had been omitted from
the rebuilt manifest, causing exact reapplication to refuse rather than report a semantic no-op.
Local Flowise pressure exposed and Core now fixes a second projection defect: the candidate
evaluator bypassed canonical parser normalization, so historical service-runtime `surfaces` that
normal Ota parsing accepts prevented the registered representation-only upgrade from projecting.
The independent closure audit accepted V11.22 with no remaining P1/P2 findings after the buildable
Site propagation was pinned. The bounded engineering notes remain drafts pending publication
review; their draft status is not an implementation or evidence gap, and the pressure evidence is
not a release claim.
The reviewed post-V11 sequence is now explicit. [V12 effect-bound refusal
assurance](../planning/v12/plan.md) is complete: its bounded implementation, real-repository
pressure, and independent closure reconciliation are complete. The following plans remain inactive:
[V12.1 secret-delivery governance](../planning/v12.1/plan.md),
[V12.2 contract-authored crossing requirements](../planning/v12.2/plan.md),
[V12.3 provider-attested authority carrier](../planning/v12.3/plan.md),
[V12.4 macOS protected authority carrier](../planning/v12.4/plan.md), and
[V12.5 Windows protected authority carrier](../planning/v12.5/plan.md), followed by the public
[V12.6 OSS Enterprise interoperability foundation](../planning/v12.6/plan.md) umbrella. V12.6
separates portable export, repository reporting, authority history, and authority references into
independently activated and closed sub-slices rather than one trust-sensitive batch. Completion of
V11.22 did not implicitly activate V12; the explicit V11 parent reconciliation and reviewed V12
activation record did. V12 activation does not activate any later slice. One successor may activate
only after its predecessor completes or is formally deferred and its own feasibility, pressure, and
independent-review gates are met.
The completed V12 `effect_assurance` candidate remains archive-bound, `unknown`, and review-only
even when its private archive and current typed graph reconcile exactly. Its writable ratchet is
owned separately by the planned inactive
[Incident Ratchet Application](../planning/incident-ratchet-application/plan.md) plan; V12's
review artifact cannot acquire write authority by reinterpretation.
The platform plans do not claim parity in advance: Linux/systemd remains the completed V11.7
hardened carrier, provider attestation remains unimplemented, and macOS/Windows containment and
recovery remain unproved. Enterprise approvals, organization policy distribution, provider
rotation operations, controlled retention, and fleet reporting remain a separate Enterprise
roadmap. V12.6 allocates public references, policy-authority evidence, verifier history, portable
export, repository reporting, and compatibility to separately reviewed OSS sub-slices that prevent
that Enterprise roadmap from inventing parallel truth. Core-owned export profiles permit original
bytes only when verified public-safe; other artifacts require a distinctly identified redacted
projection or refuse. Redacted projections expose source identities or digests only when each
linkage field is independently public-safe; otherwise linkage remains protected or omitted with
the resulting verification loss explicit. Repository fleet reports use a separate Core-owned
`RepositoryReportProfile` that classifies every identity and drill-down reference as public,
protected, or omitted; protected values cannot leak through stable hashes, pseudonyms, references,
or report identity.
Two additional cross-cutting plans remain planning-only and inactive rather than extending the
version sequence:
[OSS adapter and profile conformance](../planning/adapter-profile-conformance/plan.md)
defines registration, capability, pressure, support, deprecation, and revocation rules for V12+
implementations. Shared behavior derives `profile_semantic_identity`, while owner, source, build,
compatibility ranges, and exact target posture derive `implementation_subject_identity`.
Conformance, pressure, and release evidence bind that subject, and the final evidence-backed
registration derives `implementation_registration_identity` without a cycle. Lifecycle remains
separate registry-snapshot state; admissions and retained evidence bind all three identities plus
the observed lifecycle and snapshot;
[authority distribution lifecycle](../planning/authority-distribution-lifecycle/plan.md)
defines the normative acceptance standard for reproducible artifacts, compatibility, protected
installation, upgrade, rollback, state recovery, uninstall, and support policy. It cannot activate
or assign implementation by itself; a future version plan must explicitly own a bounded delivery
slice. Neither cross-cutting plan registers an adapter, publishes an artifact, activates V12, or
begins the deferred Enterprise V1 control-plane roadmap.
Implementation-registration revocation is bounded to the accepted installed registry snapshot
rather than claimed as globally immediate under an offline release model. Positive admission and
retained evidence must name all three exact identities plus the observed registry identity,
lifecycle, and freshness posture; an installation without a fresh update source reports
`installed_snapshot_only` and cannot claim awareness of later revocations.
Roadmap activation now distinguishes technical pressure from adoption demand. V12 through V12.2
may use signal-selected public repositories, controlled forks, and synthetic/disposable external
resources without waiting for a design partner; those runs prove bounded technical behavior, not
maintainer adoption or commercial demand. V12.3 provider-attested authority, V12.4 macOS protected
authority, and V12.5 Windows protected authority are optional demand-gated carriers. None may
activate from roadmap order, feasibility, public-repository signals, or technical curiosity alone:
each requires a documented current operator/design-partner need for the exact provider/platform
boundary and access to immutable native pressure. In the absence of that demand, formally defer
the carrier rather than implementing nominal parity, and allow V12.6 to proceed after the recorded
deferrals.
The local real-OCI
fixture plus create-chrome-extension run
[30544809360](https://github.com/bobaikato/create-chrome-extension/actions/runs/30544809360)
and Caddy run [30544809898](https://github.com/bobaikato/caddy/actions/runs/30544809898)
prove the bounded stock-OCI subset against exact Core
`d796f28e5556c0f1315052e8782ed774e9156922`. They do not prove application output, broader repo
completion, targeted egress, managed isolated paths, typed preparation, services, lifecycle proof,
or raw-shell governance.

The V11.7 public example, canonical skill, global skill mirrors, and site references are carried.
Core broker-session, crossing, archive, schema, and JSON conformance tests plus first-party
example/skill/site checks are green. Do not reopen V11.21 or widen its bounded claims. Provider
attestation remains optional stronger follow-on hardening, and contract-authored
`governance.crossing_requirements` remains explicit follow-on authoring work; neither is part of the
completed V11.7 claim. V11.22 is complete only for its reviewed candidate and source-closure work;
do not imply that a generated contract is approved, written, or pressure-proven.

## Working Rules

- Read the canonical Ota skill before Ota-specific work.
- Use `references/pressure-testing-protocol.md` for every pressure pass.
- Make and record the connected-surface decision for core docs, examples, skills, and site.
- Use released Ota versions for released proof; use the active branch only for explicit unreleased
  pressure testing.
