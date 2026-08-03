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
Live grant pressure on a pre-provisioned hardened non-root runner remains open. The separate
broker-backed, runner-verifiable work-unit lifetime remains open.

Activation prerequisite: closed by independent design review. Crossing records remain evidence,
never reusable authority. The reviewed first carrier uses a fixed system trust binding that cannot
be redirected by repository content, `OTA_POLICY`, environment variables, or caller flags.

Release target:

- `v1.6.26` implementation branch; signed offline authority is in Core, while pre-provisioned
  hardened non-root runner live-grant pressure and broker-backed work-unit authority remain open

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
- the exact task or workflow lane and selected workflow instance;
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

#### Broker-backed one-use work-unit leases (planned second carrier)

A runner-verifiable work-unit lifetime is also required by the V11.7 acceptance bar. This is a
planned second `authority_broker` carrier, not an Enterprise approval service and not an extension
of caller-controlled policy. Its purpose is narrowly to make one independently issued authority
lease usable once for one exact crossing transaction.

##### Canonical broker binding and attestation record

Before implementing `authority_broker`, Ota must define one versioned, administrator-owned binding
record outside the repository. Every adapter consumes this record; adapters must not invent their
own endpoint, credential, or attestation model. Its canonical identity includes:

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
- attestation verifier identity, issuer/audience, required runner claims, maximum age/skew, and
  attestation key-rotation posture; and
- canonical request/response domain, maximum lease duration, and replay-binding requirements.

The broker request carries the binding identity, a fresh runner-generated nonce commitment,
work-unit identity, and exact semantic crossing scope. Accepted provider or launcher attestation
must bind that challenge, the runner principal, audience, issue/expiry window, and work-unit. Ota
verifies it before requesting a lease and binds its verified identity into the lease, consume
record, receipt, and archive. Stale, wrong-audience, cached, substituted, or replayed attestations
refuse before broker or selected-lane work. A provider that cannot enforce this separation refuses
the broker carrier; transport authentication alone is not attestation.

When a reference runner image is selected, the same protected attestation must additionally bind
its exact immutable OCI digest and hardening-profile identity. Ota binds those verified identities
into the lease, crossing receipt, and archive; a signed image, SBOM, or provenance statement alone
does not prove that the runner executed that image.

##### Authority source and request

- The broker endpoint, trust root, and runner credential source are pre-bound by the platform
  administrator or hardened launcher. Repository files, `OTA_POLICY`, environment variables,
  caller flags, and workflow YAML cannot select or replace them.
- Ota resolves the complete semantic crossing scope before contacting the broker, then creates a
  fresh cryptographic nonce and runner-generated work-unit identity. The caller never supplies
  either value.
- `--grant <id>` is an explicit diagnostic or disambiguation request for a configured authority
  label. It cannot name a lease, inject issuer data, or select an endpoint. Missing or
  inapplicable labels refuse before broker mutation. This does not change the first
  `prebound_file` carrier: it continues to require its explicit `--grant` admission surface until
  the broker carrier ships.
- Ota sends the nonce, work-unit identity, contract identity, exact scope identity, requested
  action/resource, runner-observed actor posture, available provider/launcher attestation, and a
  bounded requested lifetime. Unavailable required runner identity or attestation refuses rather
  than being represented as a caller assertion.

##### Authority-selection UX (planned broker carrier)

The broker carrier should make the routine governed path operable without callers copying a raw
grant or lease identifier. This is a future broker admission semantic, not a relaxation of the
current `prebound_file` rules above:

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

The adoption bar is not that Ota is shorter than raw shell. The broker carrier must instead prove
that pre-authorized work needs no repository policy edit, no caller-authored authority, and one
governed command; exceptional work has one explicit broker-side authorization step; both emit
fresh evidence and actionable pre-side-effect refusal.

##### Lease and consume protocol

- The broker authenticates the pre-bound runner principal and atomically issues one signed lease
  for that exact work-unit, contract, scope, authority label, and expiry. The response carries a
  lease identity, issuer/key identity, broker sequence or revision, issue/expiry times, and the
  cryptographic binding needed for later verification.
- Ota verifies the broker response against the pre-bound trust root and durably records a pending
  local crossing transaction before requesting consumption.
- The broker atomically checks current lease liveness and revocation, then consumes the lease using
  the work-unit identity and nonce as an idempotency key in the same transition. A successful
  consume response must bind the exact lease and transaction identity. Ota starts selected-lane
  work only after it has verified and durably recorded that response.
- The broker, not runner-local state, rejects duplicate consume, replayed nonce, stale or revoked
  lease, wrong runner identity, wrong authority label, contract mismatch, scope substitution, and
  expiry. Ota maps each failure to typed grant-admission refusal with `execution_started: false`.

##### Recovery, finalization, and evidence

- If Ota crashes after requesting consumption but before durable acknowledgement, it must not
  execute or silently retry. Recovery re-queries the exact work-unit through the pre-bound broker;
  an unknown or indeterminate state finalizes locally as incomplete and requires a new lease.
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
carrier, not evidence that the work-unit acceptance bar is complete. V11.7 remains open until the
broker-backed one-use lifetime is implemented and pressure-proven.

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
