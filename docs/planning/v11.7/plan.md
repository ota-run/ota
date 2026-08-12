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

# V11.7 Plan

Status: active, partially implemented. Boundary crossing records and provenance are shipped. The
canonical semantic crossing evaluator and first `prebound_file` signed-authority carrier are
implemented in Core. GitHub-hosted pressure proves that absent fixed authority refuses before the
selected lane starts through create-chrome-extension run
[30714738522](https://github.com/bobaikato/create-chrome-extension/actions/runs/30714738522).
Pre-provisioned Linux/x64 VPS pressure now proves the bounded carrier under one exact workflow
revision: a live grant creates and retains a transaction-bound archive in
[30863257307](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863257307), and
expired, revoked, and out-of-scope grants refuse before task work with full checkout mutation
controls in [30862934335](https://github.com/bobaikato/create-chrome-extension/actions/runs/30862934335),
[30863024099](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863024099), and
[30863121110](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863121110).
This proves only the first carrier's `current_process_filesystem_guarded` posture, not
provider-attested separation. Core now also implements the Unix launcher-session
`authority_broker` carrier for governed `ota run` and `ota up`: it verifies challenge-bound
launcher attestation, obtains a signed exact-scope authorization, durably creates the crossing
transaction, and atomically consumes one lease before selected-lane work. Authority launcher run
[31033509379](https://github.com/ota-run/authority-launcher/actions/runs/31033509379) is green
against exact Core `bd80b29d971ccd5ac8609d9fc767a491ff382ef8`: it proves one live
broker-backed run, expired/revoked/wrong-scope/replayed refusals, same-scope missing-launcher
runtime-proof refusal, runtime archive reconciliation through Doctor, and completed runtime and
lifecycle proof transactions. The lifecycle case uses a root-owned deterministic pressure control
stub because the job principal deliberately lacks Docker socket access; it proves Ota-owned
selection, assertion ordering, finalization, and cleanup, not Docker-provider behavior. A fresh
proof execution identity is separate from semantic scope and is bound into terminal transaction
status, preventing another valid same-scope crossing from being substituted for the selected proof
run. Authority launcher run
[31250919192](https://github.com/ota-run/authority-launcher/actions/runs/31250919192) against exact
Core `257be61dd91799237357390b145be950f2fc6b3f` additionally proves broker-unavailable,
bounded approval-timeout, local-cancellation, and conflicting-pending-response refusal before
selected work, with byte-identical checkout manifests and no receipt state for every refusal.
Core now durably journals consume intent, re-queries an uncertain outcome only through fresh
launcher attestation, and terminalizes every verified status without resuming abandoned work.
Core regressions cover consumed, not-consumed, unknown, substituted-status, and crash-after-status
paths. Authority launcher run
[31257509444](https://github.com/ota-run/authority-launcher/actions/runs/31257509444) against exact
Core `9244eb2bc6a44151c4172c0634ac44bdb216a65a` and protocol
`242685d5b7c3904681f1c71d734fbe2d41679dda` proves the two-invocation hosted recovery case: the
first invocation loses the consume acknowledgement and performs no selected work; the second uses
fresh attestation to recover the consumed status, closes the abandoned transaction as incomplete,
obtains fresh authorization, executes exactly once, and leaves one valid receipt archive and zero
invalid archives. Independent dispatch
[31257511093](https://github.com/ota-run/authority-launcher/actions/runs/31257511093) reproduced the
same complete workflow at the same immutable launcher revision. Independent authority-launcher
dispatch [31260927337](https://github.com/ota-run/authority-launcher/actions/runs/31260927337)
against exact Core `9244eb2bc6a44151c4172c0634ac44bdb216a65a`, followed by green final
merge-gate run [31261639968](https://github.com/ota-run/authority-launcher/actions/runs/31261639968),
closes the remaining bounded adversarial broker cases. It proves that an allowed decision created
only after local cancellation reaches terminal launcher state cannot be delivered or start work;
an attestation with 31 seconds remaining refuses before authorization because the configured wait
plus post-approval margin requires 32 seconds; and two executions of the same broad three-task
closure retain one three-node/two-edge semantic breadth with network and repository-write effects
while consuming distinct work-unit authority and producing two valid archives. Refusal checkout
manifests remain byte-identical and no receipt state is created. Stronger
provider/launcher-attested separation remains open.
Runtime-boundary attestation v2 pressure is green in authority-launcher run
[31269597378](https://github.com/ota-run/authority-launcher/actions/runs/31269597378) against exact
Core `787ac35f7d0195d2adae85e1113e26ce4a30acc2`, protocol
`bff47c2c79b145831a3b411614301d7e09d6f377`, and launcher
`01efd331ca0d4dcf2f8899512b1e3705fc649c6d`. It proves challenge-bound observation of a dedicated
non-root Ota principal, disjoint broker/attestor keys, the exact ordered protected-launcher
profile, atomic one-use consumption, adversarial refusal and recovery, distinct catch-all work
units, and terminal runtime/lifecycle proof archives. The root-only pressure peer and fixed public
test keys make this bounded launcher conformance evidence; the workflow controller still
provisions the test authority, so this is not provider-attested production-host separation and
does not complete V11.7.

The public `ota-run/authority-protocol` crate now owns the exact v1 wire structs, fixed domains,
bounded framing, and canonical nonce/message/work-unit identities; Core pins its immutable revision
and retains all trust-root, admission, execution, receipt, and archive semantics.

The reviewed stopped-child foundation is immutable at authority-protocol
`6a2d0dc504a313a513ee41105f51449195c85797` and authority-launcher
`73a39c95ffab3125819ee655bdc7a740ec3204b9`. It proves durable pre-fork intent, exact stopped-child
identity, crash-safe temporary-state promotion, and PID-bound cleanup on Linux. It does not prove
transient-scope ownership, child continuation, broker admission, lease consumption, or selected
work execution.

The reviewed transient-scope slice is immutable at authority-protocol
`adaabfb8300925a09975c7244e27242b5cd41e60` and authority-launcher
`0f9d9eb33e37d6cd855aafdbc7c4d72b3c8957e2`. It adds a protocol-owned scope identity and native
systemd manager adapter, binding the exact stopped child to the request-derived unit, fixed
invocation slice, non-delegated controls, and kernel cgroup before durably advancing the active-slot
journal. Cleanup stops the exact scope when still present and confirms the scope absent plus its
recorded cgroup empty or absent before releasing the principal slot.
OrbStack's systemd currently refuses the pre-exec PID attachment with `ENOTTY`, which is retained as
fail-closed local evidence with no residual child or unit. Immutable Linux/x64 VPS run
[31373366733](https://github.com/bobaikato/create-chrome-extension/actions/runs/31373366733)
proves the exact reproducibly built launcher and client, positive scope ownership, terminal scope
removal, and unchanged repository state. Run
[31373928434](https://github.com/bobaikato/create-chrome-extension/actions/runs/31373928434)
uses a root-only one-shot fault after durable `scope_attached` persistence, then proves that the
next activation reconciles the abandoned slot before accepting a new request and leaves no slot,
scope, or recorded child. An execution-disabled child may exit early enough for systemd to collect
its empty scope before the post-crash observation; the durable slot, exact exit `86`, subsequent
successful reconciliation, and terminal absence are the bounded recovery evidence. No resume path,
broker contact, lease consumption, selected work, receipt, or archive exists in those immutable
scope-foundation revisions.

Activation prerequisite: closed by independent design review. Crossing records remain evidence,
never reusable authority. The reviewed first carrier uses a fixed system trust binding that cannot
be redirected by repository content, `OTA_POLICY`, environment variables, or caller flags.

Release target:

- `v1.6.26` implementation branch; signed offline authority and its bounded pre-provisioned
  hardened non-root pressure are in Core, and broker-backed one-use authority is implemented for
  `run`/`up` plus proof-wide transaction retention; the initial hosted live/refusal/proof-wide set
  plus broker-unavailable, approval-timeout, cancellation, late-approval, insufficient-freshness,
  ambiguous-response, recovery, repeated broad-closure work-unit, and strict v2 protected-launcher
  profile pressure are green, while stronger provider-attested separation remains open

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.3 plan](../v11.3/plan.md)
- [V11.4 plan](../v11.4/plan.md)
- [V11.5 plan](../v11.5/plan.md)
- [V11.6 plan](../v11.6/plan.md)
- [JSON output reference](../../spec/json-output-reference.md)
- [Execution receipt](../../spec/execution-receipt.md)

V11.7 theme:

- audited execution boundary crossings

This slice closes the remaining OSS governance gap after runner enforcement, portable governance
verdicts, merge-gate projection, and harness export already exist.

The product goal is not:

- enterprise approvals and waiver workflow

The product goal is:

- make intentional boundary crossing explicit, classifiable, and auditable in OSS
- create a first-class crossing record that anchors later reason, receipt, proof, and enterprise
  approval evidence

## Canonical product principle

Ota should not treat all non-default execution equally.

If an operator or agent intentionally crosses the routine safe/default lane, the crossing should be:

- explicit
- attributable to a requested path
- classifiable
- emitted as harness-authored evidence
- anchored by a boundary-authored crossing record that the crosser cannot author on their own

That means:

- `ota.yaml` remains the canonical execution-governance spec
- V11.3 defines what is refused versus allowed in agent-enforced execution
- V11.4 defines the canonical governance model and phase semantics
- V11.5 and V11.6 let CI and harnesses consume that truth
- V11.7 adds explicit audited crossing records for allowed-but-heavier execution paths
- the contract and its derived governance model, not caller prose alone, determine whether a
  crossing was required and how it should be classified
- reason and runtime evidence attach to the crossing record; they do not replace it

What this does not mean:

- inventing enterprise approval workflow in OSS
- turning every non-safe task into a denied path
- replacing receipts with a separate audit store
- collapsing refusal and allowed escalation into one ambiguous “warning” state
- letting the crosser author the audit truth about their own boundary crossing

## Problem statement

After V11.3 through V11.6, Ota can answer:

- what is callable
- what is refused
- what CI should enforce
- what a harness may expose

What is still weaker than it should be is the execution story for paths that are:

- allowed, but heavier than the routine safe/default lane
- intentionally chosen because the routine lane was insufficient
- relevant to later review, merge, or organizational evidence

Today the contract can distinguish safety and effect posture, but it still does not publish one
first-class OSS record for:

- which boundary was crossed
- which exact lane was crossed
- whether the selected lane required an explicit audited crossing
- whether the crossing was routine, escalated, or exceptional
- who or what principal triggered it
- what grant or approval binding allowed it, where applicable
- why the crossing happened
- which runtime evidence later attached to that crossing

It also does not yet distinguish cleanly between:

- a reusable grant that authorizes a class of crossings while live and in scope
- a fresh crossing record that must be emitted every time the boundary is actually crossed

V11.7 is the slice for making that crossing explicit instead of leaving it as implicit context in
task choice alone.

## Included capabilities

- contract-owned or contract-derived crossing-required truth
- first-class boundary-authored crossing records
- explicit audited crossing intent for allowed higher-risk execution paths
- optional or required machine-readable crossing reason capture
- stable crossing classification in receipts and governance output
- explicit actor/principal attribution in crossing evidence
- exact lane and grant or approval binding capture where applicable
- explicit separation between reusable grant authority and one-use crossing evidence
- grant liveness and scope re-check at crossing time
- explicit linkage from the crossing to the boundary that was crossed
- reason and runtime evidence attachments to the crossing record
- OSS evidence semantics that enterprise approvals and waivers can build on later

## Non-goals

- do not build human approval routing in OSS
- do not build centralized exception management in OSS
- do not force reasons on routine default-safe execution
- do not blur refused execution with allowed-but-audited execution
- do not create a second governance taxonomy outside V11.4
- do not treat caller-authored reason text as the primary audit record

## Core product gaps

### 1. Allowed heavier execution is still under-explained

There is a real difference between:

- default-safe routine execution
- allowed execution that crosses a heavier boundary
- refused execution

V11.3 covers refusal.
V11.7 should cover the middle category explicitly.

### 2. Intent is still reconstructive

Today a reviewer may be able to infer that a heavier lane was used from:

- the selected task
- the effect surface
- the resulting receipt

That is still weaker than an explicit crossing record saying:

- which exact lane was crossed
- which grant or approval binding allowed it, where applicable
- which boundary was crossed
- whether the crossing was routine, escalated, or exceptional
- what reason was supplied as context

The crossing record is the anchor. Reason and runtime evidence are useful only if they point back
to that boundary-authored record.

### 3. OSS evidence needs to be stronger before enterprise approval layers

Enterprise approvals and waivers should not be the first place this truth exists.

OSS should already be able to emit:

- a boundary-authored crossing record
- a separate grant reference model where reusable authority exists
- crossing-required truth
- crossing intent
- crossing classification
- crossing evidence

Then later enterprise layers can add:

- approvals
- waivers
- policy-based exception routing
- fleet-wide audit and reporting

## Proposed implementation slices

### 1. Audited crossing model

Define one additive governance concept for execution centered on a first-class crossing record.

The crossing record is:

- boundary-authored
- immutable after creation except for additive attachments
- single-use execution evidence, never a reusable approval object
- linked to one exact task or workflow lane
- linked to one boundary family and classification
- linked to the actor/principal mode that triggered it
- linked to the authorizer identity or authorizer-attribution state where a reusable grant or
  approval binding exists
- linked to a grant or approval binding where applicable
- linked to the runner attestation context that actually enforced and emitted the crossing
  evidence

This record is the durable anchor for later reason, receipt, proof, and enterprise approval
evidence.

The modeled crossing is:

- allowed
- non-routine relative to the default-safe lane
- worth publishing as explicit evidence

The important distinction is:

- grants may be reused while live and in scope
- crossing records may not be reused
- every crossing emits a fresh boundary-authored record, even when an existing live grant is what
  allowed the crossing

Direction:

- crossing classification values such as:
  - `routine`
  - `escalated`
  - `exceptional`
- stable boundary families such as:
  - `unsafe_task`
  - `heavier_workflow`
  - `external_effect_lane`
  - `runtime_proof_lane`
  - `blackbox_verification_lane`

The exact taxonomy can stay narrow first, but it must be stable and machine-readable.

### 2. Crossing-required truth

Ota should not leave “was an explicit crossing required here?” to runner folklore.

V11.7 should define one canonical source of truth for crossing-required posture:

- first, a contract-owned declaration where the repo explicitly marks a task or workflow lane as
  requiring audited crossing
- second, a narrow contract-derived fallback for existing lanes where the contract already makes
  the heavier boundary explicit and Ota can derive the requirement honestly

The fallback derivation should stay narrow first. Direction:

- agent-safe default lanes do not require crossing
- declared lanes outside the default-safe callable surface but still executable under a heavier
  posture may require crossing
- specifically declared heavier verification or external-effect lanes may require crossing only
  when the contract already exposes that distinction structurally

The important part is:

- no silent runner heuristics as the primary truth
- no caller deciding for itself whether crossing was required
- the repo contract and derived governance model remain canonical

The first implementation must make this evaluator explicit before grant admission. Until contract
declarations add other supported families, its only derived outputs are the existing
`unsafe_task` and `heavier_workflow` families. A grant cannot name `external_effect_lane`, runtime
proof, or another family that Ota cannot itself derive for the selected semantic closure.

#### Contract-owned requirements

The contract-owned form must be additive and monotonic. Direction:

```yaml
governance:
  crossing_requirements:
    tasks:
      publish: required
    workflows:
      release: required
```

- selectors resolve to declared task or workflow roots and are unique; unknown or duplicate
  selectors are validation errors;
- detection or caller input cannot write this declaration, and it has no `allow`, `false`,
  exemption, or caller-override form;
- `required` applies to the exact semantic selected root and complete execution closure, not a
  display name or provider-check label;
- canonical requirement is `declared || derived`; a declaration can add a requirement but can
  never suppress a derived one; and
- output records `declared` when present while retaining every derived family/classification reason
  in the decision basis. A declared-only requirement receives the fixed runner-derived
  `contract_declared_boundary` family and `escalated` classification; the declaration cannot
  self-assign authority source, family, classification, actor, or environment.

When more than one ground applies, the canonical grant-scope family/classification uses fixed
precedence: `unsafe_task` > `heavier_workflow` > `contract_declared_boundary`. The decision basis
retains every applicable declared and derived ground. This makes the selected scope deterministic
without allowing a lower-precedence declaration to weaken or hide a stronger derived boundary.

This is a planned contract surface. Current Core derives only `unsafe_task` and
`heavier_workflow`; it must not claim declaration support until schema, validation, JSON, and
archive re-derivation ship together.

### 3. Boundary-authored crossing record

When a crossing is required and grant admission succeeds, Ota should mint the crossing record
before the first execution side effect, then finalize it after success, failure, interruption, or
timeout. Missing, expired, revoked, or out-of-scope grants produce a typed grant-admission refusal
and never mint a crossing record.

Minimum record fields:

- crossing id
- exact task or workflow lane crossed
- boundary family
- crossing classification
- crossing requirement source
- actor mode
- principal attribution state
- authorizer attribution state, where applicable
- runner attestation state
- grant or approval binding reference, where applicable
- grant liveness state at crossing time
- grant scope-evaluation result at crossing time
- created timestamp
- reason state
- evidence attachment state

The important part is:

- the record is emitted by Ota or the harness boundary, not by the crosser
- the exact lane and grant binding are stamped synchronously
- grant liveness and scope are re-checked at crossing time before the crossing is finalized as
  allowed
- reason and runtime evidence attach to this record later or during execution
- routine crossings can create this record cheaply and automatically
- exceptional crossings can require louder reason or approval capture

### 3a. Grant model and liveness

V11.7 should keep grant authority and crossing evidence as separate objects.

Direction:

- a grant is reusable authority, not execution evidence
- a grant may authorize one lane or boundary family for some bounded scope
- a crossing record is emitted fresh for each actual crossing and may reference one grant
- grant reuse is valid only while the grant is:
  - live
  - not revoked
  - not expired
  - still in scope for the selected crossing

The mature rule is:

- cheap routine crossing comes from reusing a live in-scope grant
- loud crossing is triggered when the grant is missing, revoked, expired, or out of scope
- no previous crossing record can be replayed as authority for a new crossing
- grant age alone is not enough; stale standing authority should be cut off by bounded work-unit
  lifetime first, with calendar TTL only as fallback

### 3b. Grant scope dimensions

Grant scope should be machine-readable and narrow enough to avoid stale standing authority.

Direction:

- the core identity of a grant should be:
  - actor
  - action
  - resource
- repo and lane should usually be treated as resource-granularity choices inside that identity
  rather than as drifting parallel grant concepts
- environment should be treated as a hard wall around grant scope, not as an ordinary wildcardable
  dimension
- grant lifetime should default to the justified unit of work where Ota can model it honestly:
  - `this run`
  - `this task`
  - `this batch`
- calendar TTL should be the fallback only when no truthful work-unit boundary exists
- Ota should not hardcode one universal scope dimension if the governance model already exposes a
  narrower truthful boundary

The important part is:

- scope must be evaluated at crossing time, not only when the grant is first created
- crossing evidence should be able to say whether the selected crossing was in scope, not just
  whether some historical grant existed
- grants should be reviewable at a blast radius a human can understand in one breath
- the model should avoid both broad standing authority and approval-fatigue micro-grants
- grants carry the coarse reviewed scope; crossing records carry the fine-grained per-use detail

### 3c. Grant authority activation prerequisite

The repository contract may declare that a lane requires an audited crossing, but it must not issue
the grant that authorizes one. Existing `OTA_POLICY` overrides and discovered local/workspace
policy packs are caller- or repository-controlled inputs: they may narrow execution, but they must
never widen it by issuing a grant.

Before implementation, V11.7 needs one pre-bound authority source with all of these properties:

- an issuer identity configured independently of the command's policy override and bound to the
  selected contract snapshot;
- grant integrity verification against that issuer, not a self-consistent local YAML artifact;
- a fresh revocation source or a bounded issuer-signed validity window, with the clock source and
  revocation observation posture carried as evidence;
- exact semantic scope binding to contract identity, selected closure identity, derived crossing
  family/classification, actor attribution posture, and environment posture;
- archive verification that re-derives the same authority decision from archived contract,
  authority, and grant evidence.

#### Authority source model

The first authority source is `prebound_file`: a bounded-TTL signed grant bundle whose trust
binding is installed outside the repository and outside caller-controlled command inputs. The
contract references only an `authority_id`. It does not declare the bundle location, trusted key,
or public-key fingerprint.

```yaml
governance:
  crossing_authority:
    authority_id: platform-release-authority
```

The Ota installation or enforcing launcher binds that ID in a fixed system trust store to:

- one issuer identity;
- exact Ed25519 public-key material and derived key fingerprint;
- an absolute signed-bundle path outside the repository;
- optional repository and contract constraints;
- the maximum accepted bundle age;
- the store's authority and integrity posture.

The first implementation recognizes only fixed platform store locations. It has no CLI flag,
environment-variable override, repository fallback, or user-config fallback. On platforms where
Ota cannot establish that the store and bundle are outside the selected repository and not writable
by the executing principal, authority admission refuses. A writable or unverifiable store may
still be described as configuration, but it is not grant authority.

The signed bundle uses a versioned envelope with:

- `schema_version`;
- `bundle_id`;
- `issuer_id`;
- `key_id`;
- monotonic issuer `sequence`;
- `issued_at`, `not_before`, and `next_update`;
- a deterministically ordered grant set;
- a deterministically ordered revocation set;
- `algorithm: ed25519`;
- one detached signature.

Ota verifies Ed25519 over canonical RFC 8785 JSON bytes for the unsigned payload, prefixed by the
domain separator `ota.crossing-authority.bundle.v1\0`. Unknown fields, duplicate object keys,
duplicate key IDs, duplicate grant IDs, duplicate revocations, non-canonical identities, unknown
algorithms, or envelope-version downgrade refuse. Revocations are part of the signed payload.
The verified bundle identity is the SHA-256 identity of those same domain-separated canonical
payload bytes.

Each grant carries its own identity, semantic crossing-scope identity, contract identity, derived
crossing family/classification, supported actor-attribution posture, supported environment posture,
validity window, and expiry kind. Ota verifies the system binding, signature, bundle identity,
freshness, contract binding, selected grant, and signed revocation set before execution. A missing
store, stale bundle, future-issued bundle outside the declared skew bound, unknown key, invalid
signature, missing grant, revoked grant, or scope mismatch is a typed preflight refusal.

`OTA_POLICY`, discovered repo/workspace policy packs, environment variables, repository files, and
caller flags may continue to narrow execution, but they cannot provide, replace, or redirect the
authority store, bundle, key material, or issuer identity.

#### Semantic crossing scope

The canonical crossing evaluator must emit a content-addressed scope before grant admission. Its
identity includes:

- the semantic contract identity;
- the exact task or workflow lane, selected workflow instance, and ordered prerequisite-instance
  closure;
- the complete ordered conditional execution graph, including dependencies, aggregates, preparation,
  and `after_success`, `after_failure`, and `after_always` edges;
- service provisioning, readiness, lifecycle, teardown, and cleanup invocations reachable from the
  selected lane;
- proof observers, negative controls, lifecycle assertions, and every other provider mutation that
  can occur in the selected transaction;
- the derived crossing family and classification;
- selected mode, lifecycle, context, backend, and provider identity;
- target OS, architecture, and platform identity;
- normalized declared effect posture for every invocation;
- execution overrides and selected effect overrides;
- eventually, resolved non-secret task-input identity and stable source/version posture for
  secret-bearing inputs without embedding, hashing, or otherwise fingerprinting secret values.

The identity is over the ordered graph and edge conditions, not an unordered task set. A renamed or
expanded workflow, changed hook, platform-specific variant, or changed execution override therefore
has a different scope identity even when its display lane remains unchanged. The first
`prebound_file` carrier conservatively marks every free-form task-input invocation as incomplete and
refuses it. A later typed-input identity lane may admit non-secret values or stable secret-source
posture, but it must never substitute a raw or low-entropy secret hash.

The authority bundle binds grants to that scope identity. Ota evaluates scope equality at the
crossing boundary; a display-name match alone never authorizes execution.

#### Attribution, time, and revocation posture

The first source distinguishes evidence from claims:

- runner mode is directly observed as `agent` or `non_agent`; CI and human identity are not
  independently attested by Core and remain `unknown` without an authority adapter;
- environment posture is `unknown` unless the selected execution provider supplies a
  transaction-bound environment attestation; an absent `CI` variable is not local-environment
  proof;
- expiry and bundle freshness are evaluated against the runner clock and emitted as
  `runner_clock_observed`, not an external time attestation;
- revocation is bounded by the verified bundle's signed contents and `next_update` deadline. Once
  stale, the authority source fails closed rather than treating its last `revoked: false` as live.

The signed-file adapter is bounded offline authority, not strong online revocation or hardened
privilege separation. Root ownership and mode checks establish only that Ota's current unprivileged
process cannot rewrite authority files; they cannot prove the invoking job lacks `sudo`,
capabilities, or namespace control. The carrier publishes
`current_process_filesystem_guarded`, not a provider-attested separation claim. A hardened launcher
or provider attestation is required before Ota can claim stronger authority separation. It accepts
a bundle only inside its short signed validity window and only when the authority adapter can supply:

- protected issuer-sequence or last-observed-time high-water evidence; and
- a system-clock posture the executing principal cannot modify inside the enforced boundary.

The independently managed authority provisioning process advances the greatest issued sequence and
last observed time in protected adapter state alongside the signed bundle. Ota is intentionally
non-root and read-only against that state: it verifies exact bundle/high-water agreement and refuses
rollback. If either the protected high-water state or trustworthy clock posture is unavailable,
`prebound_file` refuses authority; a disclaimer is not enough to turn an untrustworthy time source
into expiry evidence. A runner clock before `not_before`, after `next_update`, behind protected
last-observed time beyond allowed skew, or otherwise untrustworthy for the requested authority
posture refuses.

The initial signed-bundle carrier must not authorize a grant whose scope requires stronger actor or
environment attribution than Core can evidence. Provider-issued authority adapters may add that
capability later without changing the canonical crossing scope.

#### Admission and evidence split

Grant admission occurs before setup, provisioning, child-process creation, service mutation, or
proof artifacts. A failed admission emits a typed refusal receipt with authority-source and
grant-evaluation evidence, but no crossing record.

For an allowed crossing, Ota durably creates a runner-owned crossing transaction and cleanup/finalizer
journal after admission and before the first execution side effect. Journal creation is atomic,
durably flushed, and protected by an exclusive transaction lock. It binds the crossing scope,
verified bundle, selected grant, observed clock posture, revocation posture, and pending terminal
state. State transitions are monotonic and compare the expected prior state before replacement.
Every success, failure, interruption, timeout, or startup failure finalizes that same transaction.
If durable journaling is unavailable, Ota refuses instead of executing.

The first local journal is runner-authored, exclusively locked, compare-before-replace, and
content-addressed. It is not independently authenticated against a same-user process that can
rewrite `.ota/state`. Receipts must publish that posture as
`authentication_posture: runner_local_content_addressed`; archive verification may claim internal
identity and admission/outcome reconciliation, but not tamper-proof execution attestation. The
broker-backed work-unit adapter below is the path to independently authenticated per-use
consumption evidence.

On startup, Ota recovers pending journals before admitting another crossing for the same repository
and scope. A journal left pending by `SIGKILL`, process crash, or power loss is finalized as
`abandoned` or `incomplete` after any recoverable cleanup/finalizer work. It can never be read as an
allowed completed crossing. Recovery must first verify the pending journal's recorded semantic
identity; malformed or identity-mismatched state refuses recovery rather than being legitimized as a
new terminal record. Recovery and finalization errors remain explicit terminal evidence.

Archive verification re-derives the crossing evaluator from the archived contract snapshot and an
authority-bearing archive-context v2 canonical selected-invocation scope (rather than a mutable
lane label),
re-verifies the archived signed envelope against the current fixed trust-store binding, and requires
that binding identity to match the one recorded at execution. If the current store has rotated, an
independently signed historical trust-root attestation must bridge the recorded binding to the
current root; a substituted archived binding is never sufficient. Verification then replays the
grant decision against recorded protected clock/high-water evidence and rejects a permitted crossing
with missing terminal finalization. Archive verification proves the historical decision made from
the bound authority snapshot; it does not claim that the grant remains live or unrevoked now.

The first carrier must not claim more attribution than Ota observes. Current Core distinguishes
`agent` from non-agent runner mode only; it does not independently attest CI identity or a generic
`local` environment. Those fields remain `unknown` unless the chosen authority adapter supplies
transaction-bound evidence. A grant never bypasses V11.3 agent-safe admission.

#### Broker-backed one-use work-unit leases (implemented across governed execution)

A runner-verifiable work-unit lifetime is also required by the V11.7 acceptance bar. The second
`authority_broker` carrier is now implemented for governed `ota run`, `ota up`, and proof
transactions; it is not an
Enterprise approval service and not an extension of caller-controlled policy. Its purpose is
narrowly to make one independently issued authority lease usable once for one exact crossing
transaction. Runtime and lifecycle proof bind their complete invocation and cleanup sets to one
proof-owned terminal transaction.

##### Canonical broker binding and attestation record

The implemented `authority_broker` carrier uses one versioned, administrator-owned binding
record at the fixed protected Linux path `/etc/ota/crossing-brokers.json`. It is root-owned, outside
the repository, and has no repository, environment, policy, workflow, or CLI override. Every
adapter consumes this record; adapters must not invent their own endpoint, credential, or
attestation model. Its canonical identity includes:

- binding schema version, `authority_id`, broker identity, and accepted authority/contract labels;
- normalized HTTPS origin, expected server name, redirect prohibition, and protocol version;
- authenticated transport profile (`mtls` or `provider_workload_identity`), never a repository or
  task-environment bearer token;
- root trust-bundle identity, verifier key set and key IDs, and an explicit bounded key-rotation
  overlap rule;
- protected credential-source descriptor and audience, resolved by the hardened launcher or
  provider through a non-delegable channel: a launcher-held one-shot capability, non-inheritable
  descriptor, or provider exchange cryptographically bound to this Ota invocation. The selected
  task must not inherit or be able to reacquire that credential, its metadata path, or its session;
- attestation trust-bundle identity, verifier key set and key IDs, issuer/audience, the fixed
  mandatory protocol claim set, maximum age/skew, and attestation key-rotation posture. The first
  carrier rejects administrator claim extensions until they have canonical validation semantics;
  and
- phase-separated message domains, maximum approval wait, minimum post-approval freshness, maximum
  lease duration, and replay-binding requirements.

Broker admission uses a two-phase handshake. Ota first freezes the selected semantic scope and
creates a fresh nonce commitment and work-unit identity. The launcher then obtains provider or
launcher attestation bound to that exact challenge, runner principal, audience, and issue/expiry
window. Ota verifies the returned attestation before it requests an authorization decision or
lease. Stale, wrong-audience, cached, substituted, or replayed attestations refuse before broker
or selected-lane work. A provider that cannot enforce this separation refuses the broker carrier;
transport authentication alone is not attestation.

##### Initial launcher-session adapter

The first `authority_broker` adapter is Unix-only and is a launcher-session delivery carrier. Other
platforms refuse this carrier rather than emulating descriptor isolation. The launcher owns the
binding's configured `mtls` or `provider_workload_identity` transport authentication; Ota receives
no mTLS private key, provider token, metadata endpoint, or broker credential in its environment,
repository, contract, receipt, archive, or task environment. The administrator-owned binding names
one fixed inherited descriptor number. The launcher delivers that already-connected local session
when Ota starts, but it is initially untrusted IPC rather than authority.

The descriptor is an Ota-only transport channel, not caller authority:

- the fixed binding, not an environment variable or CLI flag, selects the descriptor number,
  expected broker origin, protocol version, and verifier keys;
- Ota verifies that the inherited descriptor is a connected Unix-domain stream and successfully
  sets `FD_CLOEXEC` before it sends the challenge. All Ota child spawning must pass through one
  defensive descriptor-sanitization path; any missing, wrong-kind, malformed, or inheritable
  channel is a pre-side-effect broker refusal;
- the launcher keeps all broker transport credentials and provider-exchange material outside the
  job user and passes only the one invocation's channel to Ota;
- Ota writes a framed challenge request containing its fresh nonce commitment, work-unit identity,
  and exact semantic scope; the launcher obtains the bound attestation through its protected
  channel. Only after Ota verifies that response does it treat the channel as authority-capable,
  send the authorization or lease request, and verify each signed response against the binding
  before creating a crossing transaction; and
- Ota never serializes a channel handle, raw nonce, credential, or secret provider material into
  public output. Receipts and archives retain the signed public-safe protocol payloads required for
  re-verification. `invocation_id`, `runner_principal`, and `authority_mounts` are bounded non-secret
  labels, never filesystem paths, tokens, user-supplied text, or credential material. The first
  inherited-session adapter retains its existing runner label; the production systemd adapter below
  uses a protected content-addressed principal-mapping identity instead of choosing only its client
  or execution UID; and
- a launcher/provider attestation must state that the channel was delivered for this Ota invocation
  and cannot be reacquired by selected task code. If that assertion is absent, the broker carrier
  refuses rather than treating a same-user descriptor as authority separation.

This adapter leaves direct Ota-to-broker mTLS and provider-workload-identity transports for later
only if they can meet the same non-delegable delivery and task-isolation requirements. They must
reuse this binding and request/response model rather than defining new authority semantics.

##### Structured runtime-boundary attestation (implemented verifier and bounded pressure)

Protocol v1 proves a fresh challenge-bound launcher session and one-use broker consumption, but its
bounded labels do not establish the effective runtime separation required to complete V11.7. Its
wire types, domains, binding schema, identities, and archives remain immutable. Stronger separation
uses an additive attestation protocol, not a reinterpretation of broker protocol v1:

- binding schema v1 accepts only the existing public Rust type `LauncherAttestationPayload`
  (referred to here as the conceptual v1 payload),
  `ota-crossing-broker/attestation-response/v1`, and the existing response-domain identity
  computation;
- binding schema v2 still uses `ota-crossing-broker/v1` for authorization and consumption, but
  requires `attestation.protocol_version: ota-runtime-boundary-attestation/v2`,
  `LauncherAttestationPayloadV2`, response domain
  `ota-crossing-broker/attestation-response/v2`, identity domain
  `ota.crossing-broker.attestation.v2\0`, and binding identity domain
  `ota.crossing-broker.binding.v2\0`; and
- schema v2 cannot omit its profile binding or accept a v1 response. Schema v1 rejects every v2
  field. Archive readers preserve the original branch and never inject defaults before identity
  verification.

The v2 signed payload retains every v1 challenge, work-unit, scope, principal, origin, channel,
issuer, audience, and freshness binding. It adds
`attestation_protocol_version: ota-runtime-boundary-attestation/v2` and one canonical
`runtime_boundary` record containing schema version 1, stable `profile_id`, content-addressed
`profile_identity`, attestor kind `protected_launcher`, attestor-instance identity, signed
`launcher_session_binding_identity`, and an ordered observation array.
Observation states are the closed enum `verified | failed | unknown`; an unknown or failed required
observation refuses the stronger posture.

Profile `ota.runtime-boundary.protected-launcher/v1` requires exactly these observations in this
order:

1. `job_principal_non_root` via `launcher_principal_binding`;
2. `authority_binding_write_denied` via `target_principal_access_check`;
3. `attestor_state_write_denied` via `target_principal_access_check`;
4. `broker_credentials_absent_from_job` via `launcher_environment_exclusion`;
5. `broker_credentials_absent_from_task` via `child_environment_exclusion`;
6. `broker_session_non_inheritable` via `descriptor_cloexec_verification`;
7. `broker_session_not_reacquirable` via `protected_session_lifetime`;
8. `host_control_socket_unavailable` via `target_principal_access_check`;
9. `privilege_escalation_unavailable` via `launcher_privilege_policy`;
10. `launcher_binary_identity_bound` via `protected_binary_measurement`; and
11. `launcher_config_identity_bound` via `protected_config_measurement`.

Profile `ota.runtime-boundary.protected-launcher-image/v1` requires those eleven observations plus,
in order, `runner_image_identity_bound` through `protected_image_measurement` and
`hardening_profile_identity_bound` through `protected_profile_measurement`. Each published profile
has one stable `profile_id` and one exact SHA-256 `profile_identity` over its canonical ordered
definition under domain `ota.runtime-boundary.profile.v1\0`. The base profile identity is
`sha256:8a0c2b279b90840a038525f841f896016030a9f61a054fb759da4bb197faf4e8`; the image profile
identity is `sha256:8e59ecce1e92370ad682d9a73c4e710f86f302122f9bd1dc7c829f0b11aa5f7b`.
The binding and signed runtime record carry both and must match the protocol-published
definition. The profile defines requiredness; bindings and callers cannot make an observation
optional. Observation names and evidence methods are closed enums; duplicates, omissions,
reordering, unexpected observations, or inconsistent profile/kind combinations refuse. Every
separation-critical observation, including privilege escalation, must be `verified`.

Provider attestation is deliberately outside the first implementation. A later
`provider_attested_one_use` posture requires a provider-specific complete profile, authenticated
adapter, and provider job/run identity model; it must not inherit launcher evidence methods or
compose two authorities implicitly.

Each observation carries only its name, state, evidence method, stable reason code, and a bounded
semantic identity where the profile requires one. Raw paths, socket names, tokens, provider
documents, nested credential signatures, or mutable caller text do not enter public evidence.
Image, hardening-profile, launcher, runtime-profile, and attestor claims are content-addressed
bounded identities, never display strings trusted for authorization.

Binding schema v2 selects one exact `profile_id` plus `profile_identity`, attestor kind, attestor
issuer/audience, attestor key set and rotation posture. Attestor and broker authorization key sets
are disjoint and key IDs may not overlap. Repository content, environment variables, policy,
workflow input, and CLI flags
cannot select a weaker profile, change the attestor, or disable a required observation.

The first protected attestor adapter is `launcher_session_peer/v1`. It has no repository or
environment discovery. The fixed `/etc/ota/authority-launcher.json` session binding names the
inherited descriptor and expected Unix peer UID/GID; only the root-owned launcher supervisor may
supply that connected stream. Its canonical configuration identity and authority-specific session
binding identity cover `authority_id`, descriptor selection, expected peer, protected Ota binary
identity, target principal, and bounded environment. The v2 broker binding carries the expected
session-binding identity. Launcher startup requires exact `authority_id` and descriptor agreement
between both protected files before IPC, and the signed v2 payload carries the same identity without
exposing the descriptor. The attestor receives Ota's frozen challenge on that stream, obtains or
derives the exact profile observations outside the job principal, and returns the signed v2
payload before broker authorization. The same protected stream may relay broker phases, but the
attestor signing key and broker authorization key are separate authorities. Missing peer
credentials, a caller-created stream, direct job access to the attestor, or an attestor key
available to Ota or selected code refuses before authorization.

##### First production protected-launcher adapter (planned)

The first production authority-separation adapter is
`systemd_protected_launcher/v1`. It is a Linux-only implementation of the existing
`protected_launcher` attestor kind and profiles, not a provider attestor and not a new authority
carrier. It reuses the broker protocol, semantic scope, one-use lease, transaction, receipt, and
archive model above. `provider_attested_one_use` remains reserved.

The adapter is split into an unprivileged client and an independently administered system service:

- a root-owned `ota-authority-launcher.socket` is enabled at boot and socket-activates
  `ota-authority-launcher.service` from one protected absolute binary path. The service accepts only
  the inherited `AF_UNIX` stream listener whose path, inode, owner, mode, and socket-unit identity
  match protected configuration; it never binds a caller-selected listener;
- the fixed stream endpoint lives under `/run/ota`. An allowed job principal may connect to request
  an Ota invocation, but cannot replace or administer the socket, service, executable,
  configuration, producer signing key, broker proxy, credential, or state. The client verifies the
  connected server's root `SO_PEERCRED` identity, obtains the kernel-bound peer descriptor through
  `SO_PEERPIDFD`, and reconciles the protected socket, executable, process-start, unit, and cgroup
  identities before sending and again after receiving the first response. Any additional packet
  already queued at that point refuses; a later packet cannot retroactively invalidate the
  persisted signed response and is never processed. A kernel without
  `SO_PEERPIDFD` support refuses this adapter rather than falling back to a PID-number lookup;
- the client sends only a bounded invocation request: authority label, supported Ota command and
  arguments, and an absolute logical repository working directory.
  The service derives caller UID/GID only through `SO_PEERCRED`; it never accepts caller-authored
  identity or freshness. Protected configuration binds one exact job-peer UID/GID to one distinct
  dedicated execution UID/GID; mappings are one-to-one and neither identity may appear in another
  mapping. Environment inheritance, executable paths,
  descriptors, trust roots, broker origin, profile selection, target principal, and signing
  material are not request fields;
- the service accepts only one canonical request frame, rejects unknown fields and duplicate or
  concurrent request identities, and mints the authoritative per-request identity from OS
  randomness. Protected configuration lists absolute allowed repository roots. A short-lived helper
  first drops supplementary groups and adopts the configured target UID/GID, then opens the
  requested directory beneath exactly one allowed root with full-component containment and no
  magic-link or symlink escape. It returns only the directory descriptor through `SCM_RIGHTS`.
  The service binds the canonical logical path plus device/inode identity into the invocation and
  the child uses `fchdir` on that retained descriptor after dropping privilege, never resolving the
  caller path again;
- the service resolves the authority from fixed stores, connects only to the configured protected
  local broker proxy and verifies its root-owned `SO_PEERCRED` identity, creates the Ota-only socket
  pair, and launches the fixed Ota binary as the configured non-root principal with the configured
  environment and `no_new_privs`. Before Ota performs any authority-protocol I/O it sets itself
  non-dumpable, clears any ptracer allowance, and verifies that posture;
- the service keeps the producer-verification binding, broker-proxy descriptor, service
  control socket, and request connection outside the Ota child. The Ota child receives only its
  profile-fixed session descriptor. That descriptor must remain inheritable through the exact
  `fexecve`; Core sets and verifies `FD_CLOEXEC` immediately after startup and before protocol
  traffic. Every other descriptor is closed through one centralized pre-exec sanitization path;
  and
- the service observes the exact v2 profile only after receiving Ota's frozen challenge, submits
  those claims to the separately protected attestation producer, and verifies the returned signed
  response before relaying it. The launcher never holds the attestor signing credential. It then
  relays authorization, lease, consume, recovery, and terminal protocol phases without allowing
  the client or selected task to author or replace signed messages.

The service admits at most one active invocation for the exact peer-to-execution-principal mapping.
Before the
child can execute, it durably creates and fsyncs one intent-stage active-slot journal under
`/var/lib/ota/authority-launcher/active`, forks the child in a stopped state, then atomically
updates and fsyncs the journal with the child identity. It asks systemd over its root-only manager
channel to place that PID in
`ota-authority-invocation-<request-identity>.scope` under the fixed
`ota-authority-invocations.slice`. The scope uses `Delegate=no`, `KillMode=control-group`, and
`CollectMode=inactive-or-failed`; only after systemd confirms the exact scope/PID/slice identity may
the service atomically record and fsync the scope identity and continue the child. The staged
journal binds effective principal, request, scope when known, child when known, working-directory,
and launcher-session identities; every crash point is recoverable without inventing absent fields.
If a crash leaves a valid child-bearing temporary journal before rename, startup promotes and
reconciles that newer state rather than deleting it. An intent-only journal has no sufficient child
identity and therefore remains a hard refusal pending explicit operator recovery; absence is never
invented from a missing child record. Pre-scope recorded-child cleanup requires Linux `pidfd_open`
and `pidfd_send_signal` so PID reuse cannot redirect cleanup. Unsupported PID-bound cleanup,
identity mismatch, or uncertain exit retains the journal and refuses before accepting a client.

The service does not release either mapped principal's slot until Ota is terminal, systemd has
stopped the complete invocation scope, and the scope is observed empty. On service startup it
reconciles every active-slot journal with systemd before accepting a client: a live or uncertain
scope is stopped and observed empty, a missing or mismatched scope is retained as explicit failed
recovery evidence, and no affected principal can start new work until recovery is terminal. The
execution principal cannot migrate processes between cgroups, create delegated subgroups, or control
the systemd manager.
Selected code runs as the distinct execution UID/GID, has no permission to connect to the launcher
socket, and cannot obtain another launcher session. Multiple peers cannot map to one execution
principal in this adapter. A later operator
invocation, including identical command bytes after a service restart, is a legitimate new request
and creates a new service request identity, Ota work-unit identity, authorization, and lease; no
prior launcher session becomes reusable authority.

The local client is convenience, not authority. Possession of the public service socket permits a
caller to request only the same governed Ota command it could request directly; it does not permit
the caller to select a trust root, profile, principal, broker credential, grant, lease, or signed
observation. The fixed service configuration identity is the protocol's
`launcher_session_binding_identity`. The service-generated per-request identity, observed client
peer identity, opened-directory identity, systemd invocation identity, and Ota child identity form
the attestation's `attestor_instance_identity`. They never alter the fixed binding identity. No
caller nonce is authority or freshness evidence.

For this adapter, the existing protocol `runner_principal` value is exactly
`sha256:<ota.launcher.principal-mapping.v1 identity>`. Its canonical protected input binds the
job-peer real/effective/saved/filesystem UID/GID posture, the distinct execution UID/GID posture,
job-principal profile identity, launcher-session binding identity, and one-to-one mapping rule. Raw
account names or caller labels never enter broker policy. The same mapping identity is carried by
attestation, authorization request, lease, consumption, receipt, and archive; broker policy
therefore authorizes both sides of the launcher mapping or refuses.

The production adapter adds one launcher-local preface before the existing broker challenge. After
dropping privilege and before reading from or writing broker protocol frames, the fixed Ota binary
must successfully apply `PR_SET_DUMPABLE(0)`, apply `PR_SET_PTRACER(0)`, measure
`PR_GET_DUMPABLE == 0`, and emit one canonical length-bounded
`ota_process_posture/v1` frame over the private launcher session. The frame binds its message kind,
schema version, Ota PID/start-time identity, fixed Ota binary identity, `no_new_privs` posture,
dumpable measurement, ptracer-clear operation result, principal-mapping identity, and its own
content identity. It is adapter-local evidence consumed by the launcher and is never relayed as a
broker message or accepted from the client connection.

The launcher reconciles that frame with the child it created and the protected Ota binary, then
corroborates it through the external process-access probes. The Ota-authored frame cannot establish
separation alone; a missing/malformed/substituted frame, failed operation, nonzero dumpability,
wrong process identity, or successful external access probe refuses before the launcher submits the
complete claims to the protected producer. The frame identity enters `attestor_instance_identity`
and the signed session non-reacquisition observation basis. Existing inherited-session evidence is not
reinterpreted; the new preface is negotiated only by the fixed
`systemd_protected_launcher/v1` adapter and requires an additive binding/profile version before
implementation.

Client disconnect does not let the launcher infer broker or transaction state. The service sends
the same bounded interruption signal to the Ota process group that Core already handles; Ota alone
classifies cancellation before consumption, uncertain consumption, or post-consumption
finalization from its durable transaction state. The launcher then performs cgroup cleanup and
never resumes or silently re-executes the abandoned work.

The client protocol is non-interactive in the first adapter: no stdin or PTY is forwarded. One
canonical length-bounded frame schema separates `stdout`, `stderr`, and a service-authored terminal
frame; output bytes are opaque payload inside sequenced stream frames and can never be parsed as a
terminal or launcher-control message. The client connection is never reused for broker frames,
signed evidence, or authority state. Losing it cannot rewrite the Ota transaction outcome.

The protected launcher configuration gains a versioned service branch rather than reinterpreting
the existing inherited-session branch. Its semantic identity binds:

- service adapter and schema version;
- fixed client socket identity and exact one-to-one job-peer/execution UID/GID mapping;
- fixed allowed repository roots and target-principal directory-resolution posture;
- fixed Ota and launcher binary identities;
- selected protected-launcher profile identity;
- configured target UID/GID and complete child environment identity;
- broker-proxy endpoint identity and expected root-owned peer identity. The proxy, not this service,
  owns remote mTLS or workload credentials;
- producer verifier-key-set, producer binding, and producer socket identities. The separate
  `ota-authority-attestor` service owns one `LoadCredentialEncrypted=` signing credential whose
  decrypted file exists only in its own systemd credential directory. The launcher receives only
  the public verifier set and cannot read or inherit the signing credential;
- launcher service/socket unit and drop-in identities plus one published
  `ota.authority-launcher.systemd/v2` hardening-profile identity; and
- one protected `ota.authority-job-principal.systemd/v1` identity for the service that owns the
  connecting job principal; and
- maximum request size, session count, startup time, and terminal wait bounds.

`ota.authority-launcher.systemd/v2` is one closed profile, not a label for arbitrary hardened
units. Authority Protocol publishes its canonical ordered definition and content-addressed profile
identity, `sha256:c816a49e01120bf1f793aedcfec094ca0f23a8ee80f1c7e5bed4c2d9c797cb42`.
The legacy `ota.authority-launcher.systemd/v1` profile remains verifiable for historical evidence,
but it models a launcher-owned attestor credential and cannot authorize the separated producer
path.
The service unit requires these exact semantic settings:

- `User=root`, `Group=root`, empty supplementary groups, `AmbientCapabilities=CAP_SETUID`,
  `UMask=0077`, `NoNewPrivileges=yes`, `RestrictSUIDSGID=no`, `LockPersonality=yes`,
  `MemoryDenyWriteExecute=no`, `RestrictRealtime=yes`, and native system-call architecture. The
  profile deliberately does not claim executable-memory denial because selected language runtimes
  may require JIT compilation. The launcher receives only ambient `CAP_SETUID` because it must
  perform one verified `setresuid` transition for its target-principal helper and systemd otherwise
  removes that capability while applying the sandbox. The launcher retains race-resistant
  `openat2` resolution; on supported systemd hosts the `RestrictSUIDSGID` seccomp filter blocks that
  syscall, so this launcher-TCB setting is explicitly disabled while `NoNewPrivileges`, strict
  filesystem protection, and exact writable carve-outs remain mandatory. The transition to the
  non-root target clears the ambient capability before selected code can execute; selected job and
  execution evidence requires empty inheritable, permitted, effective, and ambient capability
  sets;
- capability bounding set exactly `CAP_SETUID CAP_SETGID CAP_KILL`; no other capability is
  permitted. The launcher does not write the cgroup filesystem directly; the systemd manager
  creates and stops child scopes through its root manager channel;
- `PrivateTmp=yes`, `PrivateDevices=yes`, `ProtectSystem=strict`, `ProtectHome=read-only`,
  `ProtectKernelTunables=yes`, `ProtectKernelModules=yes`, `ProtectKernelLogs=yes`,
  `ProtectClock=yes`, `ProtectControlGroups=yes`, `ProtectProc=invisible`, and `ProcSubset=pid`;
- address families restricted to `AF_UNIX`, `AF_INET`, and `AF_INET6`, preserving ordinary
  contract-selected network behavior while excluding other families; namespace creation denied;
- `ReadOnlyPaths=` restricted to the fixed `/etc/ota` stores, installation manifest, unit/drop-in files,
  launcher/Ota executables, producer public verifier set, producer socket metadata, and
  broker-proxy socket metadata;
- write access only to `/run/ota/authority-launcher`,
  `/var/lib/ota/authority-launcher`, and the protected configuration's exact allowed repository
  roots. No wildcard, relative path, caller path, home-directory expansion, or additional unit
  drop-in may widen that set; and
- service termination uses `KillMode=control-group`, while each child invocation remains in its
  separately named non-delegated scope and is recovered from the durable active-slot journal.

Each transient invocation scope fixes `Slice=ota-authority-invocations.slice`, the one stopped child
PID, `Delegate=no`, `KillMode=control-group`, and `CollectMode=inactive-or-failed`; no caller or
selected task can add or replace scope properties.

The socket unit requires `Accept=no`, the exact `/run/ota/authority-launcher.sock` path,
root ownership, the configured job-peer group, mode `0660`, removal on stop, and the exact service
unit. Unit and drop-in order is canonical; absent, duplicate, unknown, or administrator-added
directives refuse profile reconciliation rather than being treated as harmless hardening.

The root launcher service and systemd manager are explicit members of this adapter's trusted
computing base. Root D-Bus access is not described as scope-limited merely because the launcher
uses it only for the fixed child slice. Their protected binary/unit/configuration identities and
the job principal's inability to control them are verified; compromise of either remains outside
the signed protected-launcher claim. A later narrow mediator may reduce that trusted computing base
without reinterpreting this profile.

`ota.authority-job-principal.systemd/v1` is also closed and versioned. Protected configuration
binds the exact job-peer account, distinct execution account, and runner-service identity.
Its protocol-published profile identity is
`sha256:e69ef375070bbb4f5616ba46b6f29b9a987372909016d1a1dfa40a5d4daae93d`.
Admission requires all of these
non-mutating observations immediately before accepting a request:

Each ordered requirement hashes its closed allowed evidence methods. In particular, one-to-one
mapping and peer matching require both the protected mapping configuration and live process/account
observations; process state alone cannot establish configuration-wide uniqueness.

- peer real/effective/saved/filesystem UID and GID equal the fixed non-root job principal;
  supplementary
  groups and inheritable/permitted/effective/ambient capabilities are empty; and peer
  `NoNewPrivs` is `1` in `/proc/<peer-pid>/status`;
- the peer belongs to the expected protected runner service/cgroup whose active unit and drop-in
  identities match the fixed job-principal profile and whose effective `NoNewPrivileges` posture
  is enabled. Every currently live process with the job-peer UID must belong to that runner service;
  every live process with the execution UID must belong to the one active launcher invocation
  scope. Any unrelated process under either identity refuses admission;
- both accounts are locked against password authentication, have no configured supplementary
  administrative group, use the configured non-login shell, and have no allowed command in the
  canonical non-interactive `sudo -n -l -U <principal>` policy query. The query may list policy only; it must never execute an
  elevated command. Missing sudo is an allowed verified-absent posture; malformed or ambiguous
  output is `unknown` and refuses;
- systemd and Polkit authorization checks show that neither principal can start, stop, reload, replace,
  signal, or inspect the launcher, broker proxy, credential state, job-principal service, child
  slice, or invocation scopes; and
- job- and execution-principal access checks confirm neither can write any protected launcher,
  broker, attestor, systemd-unit, executable, credential, or state path or access configured host
  control sockets. The execution principal also cannot connect to the launcher client socket; and
- before authority protocol begins, the service verifies Ota's measured pre-authority posture and
  corroborates that it is non-dumpable with its ptracer allowance cleared. Access to
  `/proc/<ota-pid>/fd`, `/proc/<ota-pid>/mem`, `ptrace`,
  `process_vm_readv`, and `pidfd_getfd` from both mapped unprivileged principals must fail.

This is bounded host-principal evidence, not proof against kernel compromise, an already-compromised
root administrator, unknown privileged software outside the inspected policy sources, or a cloud
provider control plane. Any unobservable host-principal posture refuses
`protected_launcher_attested_one_use`; it is not converted to a warning.

Runtime reconciliation uses only defined evidence sources: `SO_PEERCRED` and socket metadata for
client/proxy/producer/listener identity; protected-file verification plus canonical content identities for
units, drop-ins, binaries, stores, producer public verifier set, producer binding, and configuration; systemd manager
properties for active unit, invocation, slice, scope, cgroup, effective directive, and Polkit
control posture; account/group and non-interactive sudo policy queries for bounded host-principal
posture; `/proc/<pid>/status`, `/proc/<pid>/fd`, and `/proc/net/unix` for peer/child identities,
groups, capabilities, `no_new_privs`, dumpability/ptracer posture, descriptor inheritance, and
session lifetime; the Ota process-posture preface for measured dumpability and successful ptracer
clear; explicit non-mutating process-access probes for `/proc`, ptrace,
`process_vm_readv`, and `pidfd_getfd`; and
target-principal access probes for protected paths and host-control sockets. Unsupported systemd
versions or unavailable observation sources refuse the profile.

Those sources map to the existing protected-launcher observations without adding new meaning:

- principal status supplies `job_principal_non_root`;
- target-principal access probes supply authority/attestor-state write denial and host-control
  socket unavailability;
- protected environment, credential-directory, proxy, and child-FD checks supply both broker
  credential-absence observations;
- descriptor, Ota process-posture identity, non-dumpable process-access probes, distinct
  peer/execution identities, launcher
  socket denial, and active-slot/scope reconciliation supply session non-inheritance and
  non-reacquisition;
- peer/child status, account lock, administrative-group and sudo-policy denial, runner-service and
  all-live-process containment, plus effective systemd/Polkit properties supply
  privilege-escalation unavailability;
  and
- protected content identities supply launcher binary and configuration binding, plus image/profile
  identities only when the image profile is selected.

The protected installation manifest contains the exact identities of the launcher and producer
service/socket units, all drop-ins, executables, launcher configuration, producer binding, producer
public verifier set, and hardening profile. It never contains the producer signing credential. At
startup the service reconciles those protected files, its inherited listener, systemd invocation
identity, and current cgroup with the manifest. The job principal must have no permission through
systemd or Polkit to start, stop, reload, replace, signal, inspect protected credentials, or manage
the child slice/scopes. Unit text alone is configuration evidence, not runtime proof; the service
must also observe each enforceable runtime property.

At startup and for every invocation the service reconciles the active boundary and child process
observations required by the selected profile. Missing `SO_PEERCRED`, an unexpected listener,
writable protected parents, wrong executable or active-unit identity, an unprotected key,
unexpected supplementary groups or capabilities, missing `no_new_privs`, task-visible launcher
credentials, a reacquirable broker session, accessible host-control sockets, job control over the
service, or an unverifiable required observation refuses before broker authorization.
The signed record remains bounded to its exact ordered observations and does not imply host-wide,
kernel, hypervisor, or cloud-provider integrity.

The execution-disabled candidate first passed local Linux/x64 VPS kernel pressure for the
fixed socket, root-stopped child, `openat2` containment, request-derived transient scope, terminal
scope removal, child reap, and active-slot cleanup. That run exposed and fixed three adapter
defects: listener reconciliation incorrectly rejected same-path connected socket rows; scope
properties were queried from the generic Unit interface instead of Scope; and a scope collected
between `KillUnit` and `StopUnit` was treated as cleanup failure without reconciling the recorded
cgroup. Immutable hosted runs
[31373366733](https://github.com/bobaikato/create-chrome-extension/actions/runs/31373366733)
and [31373928434](https://github.com/bobaikato/create-chrome-extension/actions/runs/31373928434)
now bind that normal and crash/recovery evidence to launcher
`0f9d9eb33e37d6cd855aafdbc7c4d72b3c8957e2`, reproducible installed binary identities, workflow
revisions `cbf5183e0b3c8edf000f9d0ea840e1b50bfa4802` and
`b7fe6daa96ba193134e9a4c75eca8f69eb1584d2`, and unchanged repository identities. This closes the
execution-disabled transient-scope foundation gate only; the production pressure bar below remains
open.

The completed execution-disabled posture slice at Core
`cc680cef790bf8334ee0dfe513c202a51c21954e`, Protocol
`b4f36fe450dc4047bd7bd623ea8ba60fd951e31d`, and Launcher
`d8aa1d0bf9783d29d53d0a5e912f09f1fa414624` advances that exact scoped child only through Core's
private `ota_process_posture/v1` preface. The launcher uses `pidfd` to resume the previously verified
PID only after scope identity is durable, reads one protocol-bounded frame, and reconciles its
semantic identity, PID/start identity, protected Ota binary identity, and principal-mapping identity
to the recorded child. Core emits that frame before CLI parsing or command dispatch and then blocks
for an explicit launcher continuation. This slice deliberately sends no continuation. Missing,
malformed, oversized, invalid-control, or substituted posture fails closed through exact scope,
child, and active-slot cleanup. This still creates no signed V3 launcher instance, forwards no
broker challenge, consumes no lease, executes no selected work, and does not satisfy the production
pressure bar. The first hosted attempt exposed one Core build-reproducibility defect: production
schema discovery embedded the absolute compile checkout through `CARGO_MANIFEST_DIR`. Core now
embeds the published schema set into the source-built executable instead, preserving installed
schema validation while allowing exact immutable binaries to reconcile across checkout paths.
Hosted normal run
[31389237232](https://github.com/bobaikato/create-chrome-extension/actions/runs/31389237232)
binds those exact source revisions and root-installed binary identities to an unchanged repository,
zero residual transient scopes, and the typed `posture_admitted_boundary_removed` terminal stage.
Root-armed crash/recovery run
[31389713244](https://github.com/bobaikato/create-chrome-extension/actions/runs/31389713244)
records launcher exit `86`, then proves fresh reconciliation and the same exact terminal cleanup.
This closes only the immutable hosted execution-disabled posture gate; V3 attestation, broker
authorization, lease consumption, selected execution, receipt/archive evidence, and provider
attestation remain open under the production pressure bar below.

The independently reviewed next execution-disabled bridge is additive to that proved posture gate
and pins Protocol `953e9e6407c9de030822b1f891046c2829b3c714` plus Launcher
`0ed578a46ce821d8dd1da671a2e53c75ded1ed0b`. After reconciling the exact Ota posture, the launcher
sends one content-addressed startup continuation bound to the invocation, recorded child, working
directory, posture, and principal mapping. Core consumes the launcher-only startup environment into
private memory and removes it before CLI dispatch. The continuation is not authority; it only
permits Core to parse the requested command, derive the exact semantic scope, and create its fresh
broker challenge on the same private descriptor. The launcher submits that challenge plus the
complete observed claims to the separately credentialed protected producer and relays only its
independently verified signed V3 response. Core remains the sole owner
of signature, trust-root, freshness, complete profile, process-posture, principal, and exact scope
verification, and reconciles the signed invocation, child, working-directory, posture, and
principal identities against the retained startup continuation. Reaching the next exact
`authorization_request` therefore establishes that Core admitted the signed V3 response, but the
launcher deliberately does not forward that request. It then removes the exact scope, confirms its
recorded cgroup empty or absent, reaps the child, finalizes the active slot, and emits
`attestation_admitted_before_authorization_boundary_removed`. This slice does not obtain an
authorization decision, issue or consume a lease, run selected work, or create crossing
receipt/archive evidence. Malformed, substituted, or internally contradictory bridge traffic emits
`pre_authorization_protocol_refused_boundary_removed`; it is never labeled as an authority
decision. The slice requires immutable Linux/x64 pressure before the historical status above can
be advanced.

The committed candidate at Authority Protocol
`574563d1f69a674960d0b3228c5a13b13bc42c19`, Authority Launcher
`13bf6db71610b86c81a251f440b80b9b8947a67d`, and Core
`31fa95b4d28a8a4971ee3fd65c841d40e54ac4d9` implements the complete
protected collector and producer bridge described below. Local ARM64 OrbStack PID 1 systemd pressure verified protected installation
identity, exact systemd runtime properties, process containment, account/sudo/Polkit posture,
protected-path and host-socket denial, Ota process-access denial, producer-owned signing, and Core's
independent complete-profile reconciliation. The positive path reached the exact authorization
request, withheld it, and finalized with zero selected work, receipts, broker decision/lease state,
active slots, or scopes. Installation drift, runtime-property drift, and unavailable producer
credentials refused before authorization. A pressure-only crash after durable scope recording left
one slot; the next activation reconciled it to zero before accepting another request. This
established local candidate evidence before the immutable hosted gate below.

Authority Launcher now carries the dedicated immutable Linux/x64 PID 1 systemd workflow for that
gate. It requires the contract-selected clean Core source build, an immutable Protocol dependency,
the complete signed admission path, exact terminal cleanup, installation and runtime drift
refusals, unavailable producer credential refusal, and crash-after-scope recovery. The workflow
passed in run
[31530832876](https://github.com/ota-run/authority-launcher/actions/runs/31530832876) against exact
Protocol `574563d1f69a674960d0b3228c5a13b13bc42c19`, Launcher
`c69ad3afc6afef0e260a7eeaa4f7340971db50af`, and clean source-built Core
`31fa95b4d28a8a4971ee3fd65c841d40e54ac4d9`. Its retained cursor-isolated journals bind the exact
positive signed stages and typed terminal refusal, prove each installation/runtime/credential
negative control remains pre-authorization, and retain the injected `scope_attached` crash slot
before successful recovery. Every terminal case has zero slots/scopes, repository manifests are
byte-identical, no selected-work or `.ota` state exists, and the artifact contains public verifier
identity rather than private credential material or path. This closes only the immutable hosted
Linux/x64 execution-disabled V3 admission gate. Broker authorization, one-use lease consumption,
selected execution, crossing receipt/archive evidence, independently administered
provider/launcher separation, and provider attestation remain open.

The execution-disabled authorization-decision slice advances exactly one protocol boundary beyond
that hosted gate. Protocol defines a Core-authored
`authorization_decision_admission` identity and a launcher-owned relay envelope. The Launcher binds
one protected broker-proxy executable and unit set in its installation manifest, rechecks the live
pidfd-bound executable around relay traffic, forwards only the exact Core request after complete V3
admission, relays only signed decisions, and durably records the decision plus Core's exact
acknowledgement before removing the child, scope, cgroup, and active slot. Core emits the
acknowledgement only after canonical signature, freshness, request,
attestation, contract, work-unit, and semantic-scope verification. Allowed decisions terminate at
`authorization_decision_verified_before_lease_boundary_removed`; denied decisions and malformed,
stale, wrong-scope, ambiguous, timed-out, or unavailable broker outcomes remain execution-disabled
refusals. This slice issues and consumes no lease, executes no selected work, and creates no crossing
receipt/archive. Immutable Linux/x64 PID 1 systemd run
[31561247605](https://github.com/ota-run/authority-launcher/actions/runs/31561247605) passed against
exact Protocol `6a92d8db9d089e44d1980f1871bf6e90eccb9960`, Launcher
`77ab20aa6ed5e3dd42cc6815ba2de7cd36d543bf`, and clean source-built Core
`b71b78ca33ea2edd7bb03ceb66c5e1e104217cd9`. The matrix does not accept the shared
protocol-refusal terminal by itself. Pressure-only broker
checkpoints bind scenario, response ordinal, decision posture, and signed decision identity. Stale
and wrong-scope cases require a relayed signed response with zero Core acknowledgements; pending
timeout requires exactly one acknowledged pending decision; ambiguity requires two distinct signed
pending responses and exactly one acknowledgement. The retained artifact includes the public broker
verifier binding, public signed decision responses, and complete bounded relay envelope so the
signed decision, Core acknowledgement, and their identities remain independently re-verifiable
after active-slot cleanup. The matrix also crashes after durable allowed-decision recording and proves
cleanup-only recovery of the exact slot, child, cgroup, and scope before a fresh request proceeds.
Each decision scenario compares the complete repository manifest before and after. Retained evidence
independently re-verifies all eight public signed decisions, all five relayed decision/admission
identity pairs, and exact decision-to-relay reconciliation. Every terminal case has zero active slots
and scopes; both crash injections retain one deliberate recovery slot before the next activation
removes it. Fourteen before/after repository-manifest pairs are byte-identical, and no selected-work,
`.ota`, lease, receipt, or archive state exists. The artifact contains only public verifier material;
private signing keys and credentials are never archived. This closes only execution-disabled signed
decision admission and cleanup pressure. One-use lease consumption, selected execution, crossing
receipt/archive evidence, independently administered provider/launcher separation, and provider
attestation remain open.

The execution-disabled one-use lease boundary is now pressure-proven in immutable Linux/x64 PID 1
systemd run [31631358796](https://github.com/ota-run/authority-launcher/actions/runs/31631358796),
binding Protocol `899718c93f205eea8ae403e041be9449daa89192`, Launcher
`2185682777c3603ae428dda68d47b1e39d709753`, and clean source-built Core
`874c5954798453f92a0141bfc964fe1a90db8d92`. The carrier persists the exact consume intent before
broker transport, verifies and records one signed consumed response, and then refuses before
selected execution while removing the exact scope, cgroup, child, and active slot. The pressure-only
broker atomically records each spent lease identity in root-owned durable state before returning
`consumed`; replaying the identical lease and consume request returns a signed `already_consumed`
response, while Core accepts only the first consumption. Pressure also covers typed decision
refusals, timeout and ambiguity, unavailable broker, installation/runtime/credential drift, and
intent/acknowledgement plus post-consumption crash recovery. Repository manifests remain
byte-identical and no selected-work, `.ota`, receipt, or archive residue is produced. This closes
only execution-disabled one-use consumption for this systemd carrier and pressure broker. Selected
execution, launcher-owned crossing receipt/archive evidence, independently administered
provider/launcher separation, and provider attestation remain required before V11.7 can complete.
The replay reopens the durable state but does not restart the pressure broker process, so restart
persistence remains a separate pressure requirement before any broader broker durability claim.

The next selected-execution candidate is implemented locally but is not yet immutable pressure
evidence. After exact V3 admission and atomic one-use consumption, Core retains the protected
launcher session while executing only the frozen work unit. Core finalizes the crossing transaction
and receipt first, sends one identity-bound completion, and requires the launcher to persist that
exact record before Core exits. The launcher then reconciles the observed child exit, reaps that
child, removes the exact systemd scope and cgroup, removes the active slot, and emits one terminal
finalization record. Completion alone never proves cleanup, and cleanup uncertainty remains a
retained boundary failure. The current portable Ota archive re-verifies the broker admission,
consumption, semantic scope, and terminal crossing transaction; launcher finalization is retained
only by the outer systemd client artifact in this candidate. Binding that post-process finalization
into portable archive evidence remains a separate acceptance gate. Immutable Linux/x64 PID 1
pressure must prove completed, failed, interrupted, pre-execution refusal, and crash-recovery paths
before this paragraph can become a pressure-completion claim.

###### Protected V3 attestation producer protocol

Immutable pressure for that bridge requires a real protected producer; a deterministic fixture
that marks observations verified without collecting them is forbidden. The root launcher is the
bounded evidence collector and remains part of the adapter's trusted computing base. The producer
is the separate `ota-authority-attestor` binary and systemd service in the
`ota-run/authority-launcher` repository. Runtime separation comes from distinct units, sockets,
credentials, process identities, and protected state rather than a separate source repository.
Only that producer owns the attestor signing key and producer clock. The launcher holds only the
producer-verification key set and protected producer-binding truth. The producer has no
repository-selected input and its attestation role remains distinct from broker authorization and
remote-broker transport.

The launcher must not send only `BrokerChallenge` to the producer. After freezing the challenge and
collecting every closed-profile observation, it sends one canonical
`launcher_attestation_signing_request/v1` containing:

- `schema_version: 1`, `message_kind: launcher_attestation_signing_request`, and
  `request_identity`. That identity is the canonical request excluding `request_identity` under the
  fixed `ota.authority-launcher.attestation-signing-request.v1\0` domain;
- the exact `BrokerChallenge` received from Core;
- `claims_identity` plus one canonical `LauncherAttestationClaimsV3` that contains every V3 payload
  field except `issued_at` and `expires_at`, including invocation, runner principal, authenticated
  origin, authority mounts, and `SystemdProtectedLauncherInstanceEvidenceV2`. The claims object has
  no self-identity field. `claims_identity` is `sha256:<lowercase hex>` over the UTF-8 bytes of the
  JCS-normalized claims object prefixed by the fixed
  `ota.authority-launcher.attestation-claims.v3\0` domain;
- the protected launcher service-binding, configuration, executable, profile, and producer-binding
  identities; and
- one producer audience and requested maximum validity bounded by protected configuration.

The request carries no signing key, credential path, caller time, validity timestamp, precomputed
signature, or repository-controlled evidence status. Its digest proves canonical content
integrity; it is not authentication. Another semantic request can always have another valid digest
and must still fail protected-peer, producer-binding, challenge, and claim reconciliation when it
does not describe the admitted launcher invocation. The launcher derives every observation from
the canonical sources listed above. Missing, unknown, contradictory, reordered, duplicated, or
placeholder observations refuse before producer contact.

The producer processes at most one bounded `SOCK_SEQPACKET` request and returns at most one bounded
response on a fixed root-owned Unix socket. Truncation, oversize payloads, ancillary descriptors,
unknown flags, an additional packet already queued before signing, or expiry of the read/write
deadline refuses. The producer closes the connection immediately after its one response or
refusal. A packet arriving after signing cannot retroactively invalidate the persisted response,
but it is never processed and can never produce another signature. The adapter does not use stream
framing and therefore does not rely on delayed trailing-input detection. Immediately after
`accept`, the producer reads `SO_PEERCRED`, obtains the connected peer's kernel-bound descriptor
through `SO_PEERPIDFD`, and derives the executable identity from the live peer rather than pathname
spelling. A kernel without `SO_PEERPIDFD` support refuses this adapter rather than falling back to a
PID-number lookup.
Before parsing and again immediately before signing, it requires the pidfd to remain live, rechecks
the same PID/start/executable identity, and reconciles the peer through the systemd manager to the
expected launcher unit, invocation, and cgroup. UID `0` alone is never sufficient. Any exit, PID
reuse, executable change, unit/cgroup change, or observation race refuses.

The producer then requires the fixed launcher/producer binding identities, canonical request
identity, challenge-to-claim equality, complete profile identity, and an active protected attestor
key. It sets `issued_at` and `expires_at` from its own bounded clock, constructs the complete
`LauncherAttestationPayloadV3`, signs under the existing V3 attestation domain, and returns one
canonical `launcher_attestation_signing_response/v1` envelope. The response binds
`schema_version`, `message_kind`, `request_identity`, `claims_identity`, the complete
`SignedLauncherAttestationV3`, and `response_identity`; the response identity excludes itself and
uses the fixed `ota.authority-launcher.attestation-signing-response.v1\0` domain. The producer
cannot alter the semantic scope, work unit, child, posture, principal, profile, or observation set.

Freshness derivation is deterministic. The producer samples its protected UTC clock once, rounded
to whole seconds, and uses that value as `issued_at`. The active key must already be valid at that
instant. `expires_at` is the earliest of: `issued_at + requested_maximum_validity_seconds`,
`issued_at + producer_maximum_attestation_age_seconds`, `issued_at` plus the selected verifier
binding's `maximum_age_seconds`, and the active signing key's protected `not_after` instant. The
producer maximum is an administrator-owned producer-configuration field bound into the producer
identity and must not exceed the selected verifier binding. Non-positive effective lifetime,
overflow, unavailable key validity, or a result outside the producer's configured issuer/audience
policy refuses before persistence or signing. The same sampled time and derived expiry are retained
by idempotent replay; they are never recomputed.

Issuance is durable and idempotent by request identity. Before returning a first response, the
producer atomically persists and fsyncs the canonical request identity, claims identity, exact
response-envelope bytes, response identity, key identity, and issuance/expiry values. A repeated
byte-identical request returns only those exact stored response bytes; it never receives new
timestamps or a second signature. Reuse of a request identity with different content, an
incomplete issuance record, an ambiguous persistence outcome, or a replay after the stored
response has expired refuses. Protected compaction may replace old entries only with a durable
spent-request checkpoint that preserves non-reissuance.

Authority Protocol defines one canonical projection from `SignedLauncherAttestationV3` to
`LauncherAttestationClaimsV3`: take the signed payload, remove producer-owned `issued_at` and
`expires_at`, and discard the signed wrapper's `key_id`, `algorithm`, and `signature`. The launcher
re-derives that projected claims identity and requires it to equal both the signing request and
response-envelope `claims_identity`. It separately verifies the response identity, request
identity, producer-owned timestamps, signed payload identity, algorithm, signature, key
authorization, freshness, audience, and producer binding before relaying only the inner signed
attestation to Core. Durable issuance is producer-owned state: the producer returns no response
before its issuance record is terminal, while pressure and producer recovery tests establish
idempotence rather than the launcher inferring persistence from an envelope.
Core independently repeats its existing trust-root, signature, freshness, challenge, scope,
profile, child, posture, and principal verification. The producer socket, unit, executable,
encrypted credential, public verifier set, issuance state, and effective runtime posture are
administrator-owned protected identities. Repository configuration, environment variables,
workflow inputs, CLI flags, and the job principal cannot redirect or invoke the producer. Selected
code receives neither the producer socket nor its credential.

This protocol is the prerequisite for immutable V3 bridge pressure. A dedicated immutable pressure
manifest and retained workflow artifact, not an Ota crossing receipt/archive, must bind the exact
Protocol, Core, Launcher, producer binary, unit, socket, public verifier-set identity, signing key
ID and public-key identity, and producer-binding identity. It must never contain the private key,
decrypted credential, or credential path. The pressure lane must prove that the job principal
cannot read, invoke, or connect to the producer; prove the complete observation set was collected
rather than injected; and retain the typed
execution-disabled terminal result. Positive pressure must reconcile the exact request, projected
claims, signed response, scope, cgroup, child, working directory, active slot, and terminal cleanup,
and must leave repository state unchanged. Negative controls must cover malformed and substituted
claims, stale response, duplicate and replayed requests, request-identity/content mismatch, wrong
or changed peer, unavailable producer, read/write timeout, truncation, an additional packet queued
before signing, a late packet that must not produce a second signature, substituted producer
binding, uncertain issuance persistence, and cleanup interruption. Every
refusal must preserve unchanged repository state and exact scope/cgroup/child/slot cleanup or retain
explicit failed recovery evidence. It still does not establish a broker authorization decision,
lease consumption, selected execution, receipt/archive crossing evidence, or provider attestation.

The initial production pressure bar requires an administrator-provisioned Linux/x64 host where the
repository job cannot administer the service or read its keys. It must prove:

- valid exact-scope authority succeeds once through the public client socket and archives the exact
  launcher profile, service-binding identity, consumed lease, and terminal transaction;
- direct execution without the launcher, a caller-created or substituted socket, wrong peer,
  altered request, directory outside an allowed root, target-principal-inaccessible directory,
  symlinked parent escape, changed path after descriptor binding, duplicate job-peer or execution
  configuration, selected-child reconnection, and a second concurrent use refuse before selected
  work. An identical request after terminal cleanup is a new invocation and must consume a new work
  unit rather than being mistaken for protocol replay;
- missing, unreadable, substituted, or writable launcher config, unit profile, executable, key,
  broker proxy, and state refuse before authorization;
- active service/socket unit or drop-in mutation, systemd or Polkit control by the job principal,
  wrong inherited listener, output-frame injection, forged terminal frames, and unexpected active
  cgroup state refuse or remain untrusted without changing authority outcome;
- peer capability, group, `no_new_privs`, runner-service, locked-account, sudo-policy, and protected
  path posture mismatch refuse before authorization; one ordinary network-requiring selected lane
  proves that the launcher does not silently replace contract network truth, while one
  Ota-enforced network-denied lane remains bounded to Ota's runtime provider rather than this
  authority profile;
- Ota dumpability or ptracer mismatch and successful job-peer or execution-principal access through
  `/proc/<pid>/fd`, `/proc/<pid>/mem`, ptrace, `process_vm_readv`, or `pidfd_getfd` refuse before
  authorization. Selected code cannot reconnect because its execution identity has no launcher
  socket permission;
- missing, malformed, replayed, or substituted Ota process-posture frames and any disagreement
  among principal-mapping identity, signed `runner_principal`, authorization, lease, receipt, and
  archive refuse before work or fail archive verification;
- selected code cannot inspect the producer signing key, broker credential, service descriptor,
  client request channel, or consumed Ota session, and cannot ask either the launcher or producer
  to sign an arbitrary challenge;
- service restart; client disconnect before consume, while consume is in flight, after broker
  response, and after durable consumption; process interruption after consumption; lost consume
  acknowledgement; service restart with live, empty, missing, and identity-mismatched child scopes;
  and host reboot preserve the established recovery rules; and
- archive history re-authorizes the historical attestor key/profile through current protected
  trust state and rejects stripped or substituted service, profile, attestation, lease, and
  terminal evidence.

This adapter may satisfy V11.7's hardened-launcher completion branch only after that independently
administered pressure is durable and reviewable. It does not make the provider-attested branch true,
and the pressure-only peer cannot be promoted or installed as this service.

Ota verifies the complete v2 payload before sending an authorization request, then binds its
identity into broker authorization, consumption, crossing receipt, and archive evidence. History
re-derives semantic scope and runtime-boundary identity from the archived contract and invocation,
then authorizes the historical attestor key/profile through the current protected trust root or an
independently signed historical bridge. The archive-authored binding snapshot is evidence only; it
cannot authorize itself. Only after trust authorization does Ota reconcile the archived binding
identity. Missing, stripped, or substituted runtime-boundary evidence refuses.

The production adapter's archive carrier must retain the complete immutable
`SystemdProtectedLauncherInstanceEvidenceV1`, not only its digest. That record contains the exact
`LauncherPrincipalMappingV1`, `OtaProcessPostureV1`, launcher/job profile identities, fixed launcher
session binding, and bounded systemd invocation, opened-directory, and child-process identities.
History validates each V1 record before identity derivation, recomputes every nested and outer
identity, requires the mapping identity to equal signed `runner_principal` throughout
attestation/authorization/lease/consumption, and requires the outer instance identity to equal the
signed `attestor_instance_identity`. Numeric UID/GID and PID are bounded machine evidence; account
names, working-directory paths, credentials, tokens, and raw process contents are not archived.
Missing, malformed, future-version, non-uniform, same-principal, or substituted instance evidence
refuses rather than falling back to the signed digest.

`launcher_attested_one_use` remains the honest v1 posture. A fully verified v2 profile derives
`protected_launcher_attested_one_use`; it does not imply host-wide security beyond its exact signed
observations. `provider_attested_one_use` is reserved and cannot be emitted by this slice. A
launcher self-check, image signature, root-owned file, or successful pressure peer cannot
independently upgrade the posture.

The first OSS implementation is verifier-side and carrier-neutral: protocol publishes the v2 wire
record, Core validates and archives it, and authority-launcher relays it without owning an
organization signing key or approval policy. Operators or providers supply the protected attestor.
The pressure-only peer may emit deterministic v2 fixtures, but it remains test code and cannot be
presented as a production issuer.

Hosted completion pressure requires a pre-provisioned non-root runner where the job cannot invoke
the attestor directly, alter its profile, or cause it to sign outside the one launcher-bound
challenge. It must prove a valid v2 profile succeeds once; v1 downgrade, wrong
profile/profile-identity/image/hardening-profile/principal/session-binding,
missing/duplicate/reordered observation, stale attestation, writable
authority state, exposed credential/session, overlapping attestor/broker key, and unavailable
required control all refuse before work. Selected code must be unable to read attestor credentials
or reacquire the consumed session, and receipt history must re-verify the exact v2 attestation and
terminal one-use transaction.

##### Core admission model

Core must not overload the first carrier's `GrantAdmissionEvidence` with broker fields. Its signed
bundle, protected sequence-state, and calendar-TTL claims are specific to `prebound_file`; filling
them with broker placeholders would make receipts lie. Core uses one carrier-neutral
`CrossingAuthorityAdmission` envelope with
strict variants:

- `prebound_file`: the existing signed-bundle admission evidence unchanged;
- `authority_broker`: binding identity, verified attestation identity/posture, work-unit identity,
  nonce commitment, authorization-decision identity, prepared lease identity, broker revision, and
  bounded approval state; and
- no implicit or fallback variant. A contract-selected broker binding that is missing, unsupported,
  or unavailable refuses rather than falling back to a standing file grant.

The common envelope carries only canonical facts shared by every carrier: authority identity,
admission identity, authorization identity, contract identity, semantic scope identity, boundary
family, classification, runner-observed actor mode, decision, and admission time. JSON output,
crossing transactions, receipt archives, and history verification consume that envelope and switch
on its explicit carrier variant. This is the only allowed path for later carriers; `prebound_file`
compatibility remains additive.

Admission evidence is immutable evidence available before selected-lane execution. It must not
contain a terminal crossing-transaction binding, consume response, task outcome, or cleanup result.
Those belong to the separately created pending transaction and its terminal finalization record.
The broker consume record links the prepared lease to that transaction only after Ota has durably
created the journal.

The first broker binding record is read only from `/etc/ota/crossing-brokers.json`, is addressed by
the existing contract `authority_id` only, and retains this canonical schema-v1 shape:

```json
{
  "schema_version": 1,
  "identity": "sha256:<canonical binding digest>",
  "authority_id": "platform-release-authority",
  "broker_id": "platform-crossing-broker",
  "origin": "https://broker.example.internal",
  "server_name": "broker.example.internal",
  "protocol_version": "ota-crossing-broker/v1",
  "transport_authentication": {
    "kind": "mtls",
    "trust_bundle_identity": "sha256:<broker trust bundle>",
    "credential_source_identity": "launcher:workload-session/v1"
  },
  "credential_delivery": {
    "kind": "launcher_session_fd",
    "descriptor": 3,
    "session_audience": "ota-crossing-broker"
  },
  "broker_verifiers": [
    { "key_id": "broker-2026-01", "algorithm": "ed25519", "public_key": "..." }
  ],
  "attestation": {
    "issuer": "runner-launcher",
    "audience": "ota-crossing-broker",
    "trust_bundle_identity": "sha256:<launcher attestation trust bundle>",
    "verifiers": [
      { "key_id": "launcher-2026-01", "algorithm": "ed25519", "public_key": "..." }
    ],
    "maximum_age_seconds": 180,
    "maximum_clock_skew_seconds": 5,
    "key_rotation_overlap_seconds": 300,
    "mandatory_protocol_claims": [
      "binding_identity",
      "challenge_nonce_commitment",
      "invocation_id",
      "work_unit_identity",
      "semantic_scope_identity",
      "runner_principal",
      "channel_delivery",
      "authenticated_origin",
      "authority_mounts"
    ],
    "required_administrator_claims": []
  },
  "message_domains": {
    "challenge_request": "ota-crossing-broker/challenge-request/v1",
    "attestation_response": "ota-crossing-broker/attestation-response/v1",
    "authorization_request": "ota-crossing-broker/authorization-request/v1",
    "authorization_decision": "ota-crossing-broker/authorization-decision/v1",
    "lease_issuance": "ota-crossing-broker/lease-issuance/v1",
    "lease_consume": "ota-crossing-broker/lease-consume/v1",
    "lease_consume_response": "ota-crossing-broker/lease-consume-response/v1",
    "lease_consumption_query": "ota-crossing-broker/lease-consumption-query/v1",
    "lease_consumption_status": "ota-crossing-broker/lease-consumption-status/v1"
  },
  "maximum_approval_wait_seconds": 120,
  "minimum_post_approval_freshness_seconds": 30,
  "maximum_lease_seconds": 300
}
```

Binding schema v2 retains every non-attestation v1 field and changes only these canonical fields:

```json
{
  "schema_version": 2,
  "identity": "sha256:<ota.crossing-broker.binding.v2 digest>",
  "protocol_version": "ota-crossing-broker/v1",
  "attestation": {
    "protocol_version": "ota-runtime-boundary-attestation/v2",
    "profile_id": "ota.runtime-boundary.protected-launcher/v1",
    "profile_identity": "sha256:<published profile definition>",
    "attestor_kind": "protected_launcher",
    "adapter": "launcher_session_peer/v1",
    "launcher_session_binding_identity": "sha256:<protected launcher session binding>",
    "issuer": "runner-launcher",
    "audience": "ota-crossing-broker",
    "trust_bundle_identity": "sha256:<launcher attestation trust bundle>",
    "verifiers": [
      { "key_id": "launcher-2026-01", "algorithm": "ed25519", "public_key": "..." }
    ],
    "maximum_age_seconds": 180,
    "maximum_clock_skew_seconds": 5,
    "key_rotation_overlap_seconds": 300
  },
  "message_domains": {
    "attestation_response": "ota-crossing-broker/attestation-response/v2"
  }
}
```

That fragment is a version delta, not a standalone binding: all unchanged required v1 broker,
transport, credential-delivery, authorization, lease, recovery-domain, and bounded-window fields
remain mandatory. Schema v2 removes configurable `mandatory_protocol_claims` and
`required_administrator_claims`; the selected profile fixes the complete claim and observation
set. Its attestor verifier keys must be disjoint from the unchanged `broker_verifiers`. A profile
identity is accepted only with its fixed matching `attestor_kind` and adapter.

`identity` is the SHA-256 digest of JCS-normalized content with its own value blank and the
schema-matched `ota.crossing-broker.binding.v1\0` or `ota.crossing-broker.binding.v2\0` domain
prefix. Broker and attestation verifier entries are sorted
by unique `key_id`; the binding rejects duplicates, unsupported algorithms, non-HTTPS origins,
redirects, non-absolute launcher-owned paths, unknown administrator claims, unsupported descriptor
values, duplicate or missing message domains, or windows outside the bounded maximum. Every signed
message carries the matching `message_kind` and is verified only under its corresponding domain.
The mandatory protocol claims are not configurable. Any key rotation uses a new binding identity
with an explicit, time-bounded overlap of verifier keys; a caller cannot supply a replacement key
or trust root.

Ota loads the fixed store through the existing protected-file verifier. Symlinks, non-regular files,
writable parents, malformed or unknown records, duplicate `authority_id` entries, and duplicate
binding identities all refuse before IPC, broker, or selected-lane work.

Carrier selection is external and exact. For a contract `authority_id`, Ota reads the existing
fixed `prebound_file` store and `/etc/ota/crossing-brokers.json` through the same protected-file
rules, then admits exactly one matching carrier binding. Zero matches refuse as unknown; more than
one match, including a file/broker collision, refuses as `crossing_authority_ambiguous`. Ota never
uses store ordering, fallback, or a repository declaration to choose a carrier. A missing broker
store is not an error for an existing unambiguous `prebound_file` authority; a present broker store
must be structurally valid before any authority decision.

`image_identity` and `hardening_profile_identity` are the mandatory bounded identities behind the
`runner_image_identity_bound` and `hardening_profile_identity_bound` observations when the binding
selects `ota.runtime-boundary.protected-launcher-image/v1`. They are absent from the base
protected-launcher profile rather than becoming optional fields inside one profile.

When a reference runner image is selected, the same protected attestation must additionally bind
its exact immutable OCI digest and hardening-profile identity. Ota binds those verified identities
into the lease, crossing receipt, and archive; a signed image, SBOM, or provenance statement alone
does not prove that the runner executed that image.

##### Authority source and request

- The broker endpoint, trust root, and runner credential source are pre-bound by the platform
  administrator or hardened launcher. Repository files, `OTA_POLICY`, environment variables,
  caller flags, and workflow YAML cannot select or replace them.
- Ota resolves the complete semantic crossing scope before contacting the broker, then creates a
  fresh cryptographic nonce commitment and runner-generated work-unit identity. This frozen
  challenge is given to the launcher, which obtains an attestation bound to that exact challenge.
  Ota verifies the returned attestation before it requests an authorization decision or lease. The
  caller never supplies any of these values.
- `--grant <id>` is an explicit diagnostic or disambiguation request for a configured authority
  label. It cannot name a lease, inject issuer data, or select an endpoint. Missing or
  inapplicable labels refuse before broker mutation. This does not change the first
  `prebound_file` carrier: it continues to require its explicit `--grant` admission surface.
- Ota sends the verified attestation identity, nonce commitment, work-unit identity, contract
  identity, exact scope identity, requested action/resource, runner-observed actor posture, and a
  bounded requested lifetime. Unavailable required runner identity or challenge-bound attestation
  refuses rather than being represented as a caller assertion.

##### Authority-selection UX (broker carrier)

The broker carrier makes the routine governed path operable without callers copying a raw grant or
lease identifier. This does not relax the current `prebound_file` rules above:

- Default-safe execution has no authority request and no crossing.
- For a crossing-required lane, Ota resolves the contract-selected `authority_id` and asks the
  independently administered broker for eligible authority for the exact verified invocation. It
  may proceed only when exactly one live, in-scope authorization matches the verified contract,
  semantic scope, actor posture, and attestation. Zero matches and multiple matches both refuse
  with distinct typed evidence; Ota never guesses a selection or applies evaluation order as a
  tie-breaker.
- The caller does not receive, copy, or name a raw grant or lease identity. Ota records the broker
  selection identity and its bounded selection basis in the fresh crossing transaction, receipt,
  and archive so later audit does not rely on implicit selection.
- An exceptional crossing uses the same fixed authority source but requires a broker-issued,
  one-use work-unit authorization. The authorization decision and any approval reference are
  broker-policy-owned and invocation-bound. Caller-provided justification is non-authoritative
  context only and cannot create or widen authority.
- `--grant` may remain during migration as an explicit label-level diagnostic/disambiguation
  surface. It must never silently select an actual broker lease, bypass exact-one matching, or
  widen contract authority.

##### Exceptional same-terminal approval wait

An exceptional crossing may wait in the invoking terminal for a broker-side authorization decision,
but only after Ota has frozen one complete semantic scope, work-unit identity, nonce commitment,
attestation identity, and requested lifetime. The wait is bounded by
`maximum_approval_wait_seconds`; its state machine is
`requested -> pending -> allowed | denied | timed_out | cancelled | ambiguous`. Ota starts no
selected-lane work while pending, and no lease is issued or consumed while approval remains pending.
The first carrier never refreshes a frozen attestation. Ota begins waiting only when its remaining
freshness covers `maximum_approval_wait_seconds` plus the binding's
`minimum_post_approval_freshness_seconds`; otherwise it refuses before waiting. Ota rechecks the
same attestation before lease issuance and again before consumption.

- timeout, denial, local interruption/cancellation, unavailable broker, stale attestation, changed
  scope, changed work unit, or an otherwise indeterminate response refuse before execution. A
  later approval for a cancelled local request is non-executable and must not issue a usable lease;
- a pending decision may be retransmitted only when its identity is byte-for-byte equivalent to the
  already verified pending result. A distinct pending response is `ambiguous` and refuses. A final
  `allowed` or `denied` response after pending must carry a strictly greater broker revision;
  rollback or equal-revision finalization refuses before acknowledgement. Lease issuance and
  consumption happen only after an allowed decision;
- the caller cannot continue waiting against a changed invocation, edit the selection while a
  request is pending, or carry a pending request into another process; and
- caller justification remains non-authoritative context. The broker owns the authorization
  decision and any approval reference.

The wait is an exceptional broker interaction, not a general approval UI or a repository-owned
policy mechanism.

The adoption bar is not that Ota is shorter than raw shell. The broker carrier must instead prove
that pre-authorized work needs no repository policy edit, no caller-authored authority, and one
governed command; exceptional work has one explicit broker-side authorization step; both emit
fresh evidence and actionable pre-side-effect refusal.

##### Lease and consume protocol

- The protocol order is fixed: authorization decision -> lease issuance -> durable local journal
  -> atomic broker consumption -> execution. A pending approval is not a lease and cannot be
  consumed.
- After an allowed decision, the broker authenticates the pre-bound runner principal and issues one
  signed lease for that exact work-unit, contract, scope, authority label, and expiry. The response
  carries a lease identity, issuer/key identity, broker sequence or revision, issue/expiry times,
  and the cryptographic binding needed for later verification.
- Ota verifies the broker response against the pre-bound trust root and durably records a pending
  local crossing transaction before requesting consumption.
- The broker atomically checks current lease liveness and revocation, then consumes the lease using
  the work-unit identity and nonce as an idempotency key in the same transition. A successful
  consume response must bind the exact lease and transaction identity. Ota starts selected-lane
  work only after it has verified and durably recorded that response.
- The broker, not runner-local state, rejects duplicate consume, replayed nonce, stale or revoked
  lease, wrong runner identity, wrong authority label, contract mismatch, scope substitution, and
  expiry. Ota maps each failure to typed grant-admission refusal with `execution_started: false`.

##### Scope-breadth evidence

Every broker crossing records a runner-derived scope-breadth summary inside, but never in place of,
the canonical semantic scope. Its content-addressed identity binds the selected closure's node and
edge counts, declared effect categories, and bounded resource identities/counts. Raw resource
values do not enter the summary. Receipt evidence carries that scope, and archive verification
re-derives the complete scope and breadth from the archived contract. Task inputs, secrets, raw
authority data, and unbounded provider resource strings never enter breadth evidence.

Breadth evidence explains the shape of an authorization to an operator and helps pressure tests
detect catch-all requests. It is not an approval metric: one declared workflow may truthfully be
one atomic work unit even when its closure contains many nodes. The authority invariant remains one
verified semantic scope and one work-unit identity per consumed lease.

##### Recovery, finalization, and evidence

- If Ota crashes after requesting consumption but before durable acknowledgement, it must not
  execute or silently retry. Ota persists the exact consume intent before sending it. A later
  invocation obtains a fresh challenge-bound launcher attestation, then re-queries that exact
  lease, consume-request identity, work unit, and pending transaction through the pre-bound broker.
  verified `consumed`, `not_consumed`, and `unknown` statuses all finalize the abandoned transaction as incomplete;
  none resumes its work. Any later execution requires a fresh authorization and lease.
- The signed recovery status is durably journaled before finalization. If Ota stops after recording
  that status, the next invocation re-verifies the recorded signed status and completes local
  finalization without issuing a second status query. Recovery messages have distinct
  `lease_consumption_query` and `lease_consumption_status` domains and cannot be interpreted as
  ordinary consume requests or responses. A consumed recovery retains its original consume intent
  until the atomic terminal write, so every crash point remains in this dedicated recovery path.
  Archive reads preserve the exact semantic identity of the earlier seven-domain binding profile;
  only live bindings require the complete nine-domain recovery profile.
- Ota finalizes the same crossing transaction on success, task failure, interruption, timeout,
  startup failure, or abandoned recovery. It may report that terminal outcome to the broker, but
  delivery acknowledgement is not proof that selected work completed.
- Receipts and archives bind the broker authority identity, lease identity, work-unit identity,
  nonce commitment, consume response identity, exact semantic scope, and terminal local
  transaction. Archive verification re-derives scope from the archived contract snapshot and
  refuses missing, substituted, replayed, or terminally incomplete broker evidence.
- No broker token, runner credential, raw nonce, or private endpoint diagnostics appear in public
  JSON, receipts, archives, repository state, or task environments. Debug diagnostics remain
  launcher/operator-local.

##### Carrier boundaries and pressure bar

- The broker carrier does not add hosted accounts, approval routing, waivers, centralized policy,
  or fleet reporting. It consumes authority already issued by an independently operated source.
- A hosted runner may claim stronger separation only when the launcher or provider supplies a
  verifiable attestation that binds the runner principal and protected authority material. A
  self-hosted image without that evidence remains `current_process_filesystem_guarded`.
- Pressure must prove one exact eligible authorization, zero eligible authorizations, multiple
  eligible authorizations, a valid one-use lease, expired lease, revoked lease, out-of-scope
  lease, duplicate/replayed consume, broker-unavailable preflight, and interruption/recovery.
  Every refusal must occur before task, service, provider, child-process, or repository mutation;
  the valid path must archive a broker-bound terminal crossing transaction.
- Pressure must also prove pending-approval scope mutation, timeout, local cancellation followed by
  late approval, insufficient attestation freshness refusing before a pending wait, the
  post-approval freshness recheck, conflicting approval response, and a deliberately broad
  catch-all workflow. One work unit is one exact selected invocation closure: a
  second invocation, including the same displayed lane, has a distinct work-unit identity and
  requires a distinct lease. A genuinely atomic declared workflow remains valid and retains its
  breadth summary as explanation rather than a refusal trigger; Ota introduces no atomicity
  heuristic.

##### Reference hardened runner image (deferred delivery)

Ota may later publish a **reference hardened runner image**, never a “prebound authority image.”
It can supply a pinned Ota binary and minimal runtime, non-root runner user, fixed authority-store
layout, hardening checks, SBOM, provenance, signature, and immutable digest. It must not contain
shared trust roots, signed grants, broker credentials, or reusable organization authority.

An operator supplies authority separately through an administrator-controlled image layer,
protected read-only mount, or non-delegable provider/launcher channel. In-image conformance checks
are diagnostic evidence only: they cannot independently establish absence of root escalation,
Docker socket access, mutable authority mounts, or credential access. Stronger separation requires
provider or launcher attestation that binds the effective runtime boundary and, where used, the
exact image digest and hardening-profile identity. The delivery order is a published hardening
profile and conformance check first; a signed reference image follows only after the broker and
launcher binding contract is stable, with its CVE, patching, provenance, and release obligations
explicitly owned.

##### Hardening profile and diagnostic conformance

Before publishing a reference image, Ota publishes one versioned `prebound_file` hardening profile
and a read-only conformance command:

```bash
ota authority inspect --json
```

The command has no contract mutation, grant-selection, transaction, or execution authority. It
must not mint a crossing record, create or update receipts/archives/high-water state, provision a
resource, execute a task, or make a grant admissible. It reports runner-observed diagnostic
evidence only:

- supported OS and architecture;
- effective user/root posture and non-elevating passwordless-sudo evidence when the platform can
  observe it safely; otherwise the sudo capability remains `unknown` or `unavailable` rather than
  executing a privileged command merely to test the boundary;
- `DOCKER_HOST` presence and the observed common Docker socket posture;
- fixed trust-store, bundle, and sequence-state existence, resolved regular-file posture, and
  root-owner/non-writable parent checks produced by the same fixed-path canonical verifier used by
  `prebound_file` admission; the command inspects every binding in the fixed store and accepts no
  authority id, path, environment, policy, or repository override; and
- profile identity/version plus explicit `unknown` for capabilities it cannot observe, including
  namespace control, alternative container endpoints, provider metadata credentials, and broader
  administrative escalation.

Each JSON observation has one of `passed | failed | unknown | unavailable`, whether it is required
or informational, its bounded observation method, and a stable reason code. The derived diagnostic
profile is `matched_with_unknowns` only when every required check passed and only informational
capabilities remain unknown, `incomplete` when a required check is `unknown` or `unavailable`,
`failed` when any required check failed, and `unsupported` when the platform cannot evaluate the
fixed carrier. Only `matched_with_unknowns` exits zero. Every other verdict remains schema-valid
JSON and exits nonzero; human output follows the same verdict. Unknown capabilities remain explicit
and cannot silently become passed checks.

A matched profile means only that the locally observable checks matched; it publishes
`authority_separation_posture: current_process_filesystem_guarded`, never
provider-attested/hardened separation. `DOCKER_HOST` and named common-socket observations do not
claim that every container endpoint is absent. Missing, unreadable, or malformed authority state
is an inspectable diagnostic failure and never falls back to an advisory allow. Public output must
not expose raw paths, public keys, fingerprints, signatures, bundle contents, grant identities,
or parser/filesystem error details. The CCE hardened-runner workflow should consume this canonical
JSON rather than duplicate its in-scope checks in shell; its remaining worktree-mutation and
receipt assertions stay pressure-specific.

The `prebound_file` adapter intentionally supports bounded calendar TTL only. It is the first grant
carrier, not evidence that the work-unit acceptance bar is complete. The broker-backed one-use
lifetime is implemented for `run`/`up`; V11.7 remains open until that carrier is pressure-proven,
proof commands bind complete transactions, and the stronger attested-separation bar is met.

#### `--grant` admission semantics

`ota run <task> --grant <id>` and `ota up --workflow <name> --grant <id>` use the same option
admission boundary as other Ota-owned execution flags. The flag remains Ota-owned when it appears
after the task/workflow selector and before declared task inputs.

The first carrier uses explicit selection only:

- a crossing-required lane without exactly one `--grant <id>` refuses;
- duplicate `--grant` flags refuse during parsing;
- an unknown, revoked, expired, stale, out-of-scope, or incorrectly attributed grant refuses;
- supplying `--grant` for a lane where the canonical evaluator says no crossing is required refuses
  as inapplicable;
- Ota never auto-selects a grant merely because one bundle entry appears to match.

Dry-run performs the same authority-source, signature, freshness, revocation, attribution, and scope
evaluation as real execution. It emits `admissible_not_consumed`, creates no crossing transaction,
does not emit a crossing record, does not consume a broker work unit, and performs no execution side
effect. A failed grant admission emits a typed refusal and likewise never emits a crossing record.
Real execution repeats
the time-, sequence-, and revocation-sensitive checks immediately before durable transaction
creation; any changed decision refuses.

### 4. Execution-intent capture

Add an explicit opt-in lane for allowed audited crossings.

The exact command spelling can stay narrow, but the product should support a shape like:

- `ota run <task> --reason "..."`
- `ota up --workflow <name> --reason "..."`
- or an equivalent additive crossing flag/value pair if reason and crossing intent need to stay
  separate

The important part is:

- the crossing is intentional, not inferred only from task choice
- the runner/harness records the supplied intent
- the output distinguishes no-reason, optional-reason, and required-reason cases honestly
- reason capture remains additive evidence, not the source of truth for whether crossing was
  required
- caller-supplied reason is preserved as narrative context, not the authoritative audit anchor

### 5. Actor and principal attribution

Crossing evidence should answer more than “a crossing happened.”

It should also publish who or what triggered it, at the OSS level that Ota can claim honestly.

Direction:

- actor mode such as:
  - `agent`
  - `non_agent`
- richer principal kind such as `human`, `ci`, or `harness` only when a selected provider or
  launcher supplies transaction-bound attestation; otherwise principal attribution is `unknown`
- triggering principal attribution where available from verified provider or launcher evidence
- explicit authorizer attribution where a reusable grant or approval binding exists
- explicit runner attestation posture for the execution context that enforced and emitted the
  crossing evidence
- explicit distinction between:
  - caller-supplied identity or label
  - runner-known execution mode / principal kind
  - authorizer identity or authorizer-attribution state
  - runner identity or runner-attestation state

The important part is:

- receipts and governance output can answer who or what crossed the boundary
- receipts and governance output can distinguish:
  - the acting principal
  - the human or policy authorizer who granted the reusable authority, where applicable
  - the runner context that actually enforced and attested the crossing
- OSS does not overclaim identity it cannot verify locally
- enterprise can later layer stronger organizational identity, retention, and approvals on top

### 6. Governance-model integration

Do not create a second machine output model.

V11.7 should extend the V11.4 governance model with additive fields for:

- crossing record id
- crossing required / not required
- crossing requirement source:
  - `declared`
  - `derived`
- crossing classification
- crossing classification source:
  - `runner_derived`
- crossing boundary family
- crossing lane id
- crossing actor mode
- crossing principal attribution state
- crossing authorizer attribution state
- crossing runner attestation state
- crossing grant binding state
- crossing grant liveness state
- crossing grant scope state
- crossing grant identity shape
- crossing grant environment boundary state
- crossing grant expiry kind
- crossing intent source:
  - `caller_supplied`
  - `runner_defaulted`
- crossing reason present / missing
- crossing evidence attachment state

Preflight and post-execution semantics must remain phase-accurate.

Classification should not be caller-authored truth.

The mature rule is:

- callers may request or justify a heavier lane
- the runner derives the crossing classification from contract/governance truth plus the selected
  execution path
- if caller intent is preserved, it is preserved as intent metadata, not canonical classification

### 7. Receipt and evidence linkage

Crossing evidence should be carried by the existing evidence story, not outside it.

That means:

- receipts remain harness-authored execution evidence
- governance output remains the portable evaluation layer
- crossing evidence is linked from both surfaces where relevant
- receipt-linked crossing evidence carries actor/principal attribution and reason state alongside
  boundary family and classification
- receipt-linked crossing evidence also carries authorizer attribution and runner-attestation
  posture so later review can separate actor, grant authority, and execution context honestly
- receipt-linked crossing evidence also carries grant binding, liveness, and scope-evaluation
  posture where applicable
- runtime proof and receipt evidence attach to the crossing record instead of floating as separate
  audit claims
- refusal and crossing stay distinct outcomes

### 8. OSS / enterprise boundary

Keep OSS focused on:

- first-class boundary-authored crossing records
- explicit crossing-required truth
- explicit crossing intent
- explicit crossing classification
- explicit actor/principal attribution at the level Ota can honestly verify
- machine-readable evidence

Reserve enterprise for:

- approvals
- waivers
- exception policy rollout
- fleet-wide visibility and reporting

## Acceptance bar

V11.7 is complete when:

- Ota creates a first-class crossing record as the immutable anchor for audited boundary crossings
- the crossing record is boundary-authored, not crosser-authored
- the crossing record stamps the exact lane crossed and grant or approval binding where applicable
- grant authority and crossing evidence remain separate objects
- a fresh crossing record is emitted for every actual crossing, even when a prior grant is reused
- Ota re-checks grant liveness and scope at crossing time instead of treating grants as stale
  standing authority
- grant identity is modeled around actor + action + resource rather than loose approval prose
- environment remains a hard grant boundary rather than a wildcardable scope field
- the first `prebound_file` carrier remains explicitly bounded as
  `current_process_filesystem_guarded`; V11.7 cannot complete until a hardened launcher or
  provider-attested carrier establishes that the job cannot self-supply authority
- completion requires broker-backed, nonce-bound, atomic one-use lease consumption for the exact
  work unit; calendar TTL alone is not work-unit lifetime authority
- the canonical broker binding defines transport authentication, protected credential isolation,
  trust/key rotation, attestation verification/freshness, and replay binding, and its identity is
  preserved and archive-rederived with every broker crossing
- grant-required runtime and lifecycle proof either bind one terminal crossing transaction across
  the complete proof invocation set and cleanup, or refuse before proof work starts
- pressure proves valid live authority, expired, revoked, out-of-scope, double-spend/replay,
  broker-unavailable, interruption, recovery, cleanup, and archive re-verification controls;
  every refusal is pre-side-effect and every allowed terminal transaction is independently
  reconciled
- crossing evidence distinguishes the acting principal, the authorizer, and the runner attestation
  context instead of smearing them into one identity field
- Ota can answer whether a crossing was required from contract-owned or contract-derived truth
- Ota can distinguish routine execution from allowed audited boundary crossings
- crossing classification is runner-derived governance truth, not merely caller prose
- crossing evidence can attribute the crossing to an actor/principal mode honestly
- a crossing can carry machine-readable reason state without collapsing into receipt prose
- reason and runtime evidence attach to the crossing record instead of replacing it
- governance output publishes crossing posture in a stable additive form
- receipts preserve crossing evidence as harness-authored truth
- refusal remains a distinct execution/governance outcome from allowed audited crossing
- the OSS surface is strong enough that enterprise approval layers can build on it instead of
  inventing the concept later

## Follow-on boundary

V11.7 is the OSS audited-crossing layer.

What can later build on top of it in enterprise is:

- approval routing
- waiver lifecycle
- policy-scoped exception approval requirements
- centralized audit, retention, and fleet-level reporting

What should tighten the OSS trust model after this is:

- [V11.9](../v11.9/plan.md): governance truth reconciliation and evidence classes, so
  boundary-authored crossing records and attached reason/evidence fields stay emitted from the
  same authoritative decision line and clearly distinguish caller assertions from runner-derived or
  runner-attested truth
