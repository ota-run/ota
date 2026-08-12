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

Status: planned and inactive. This plan does not authorize contract, CLI, provider, or runtime
implementation.

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
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: BILLING_API_TOKEN
    recipients:
      tasks: [publish_billing]
      propagation: deny
      hooks: deny
      services: deny
      helpers: deny
      containers: deny
      remote_execution: deny
    constraints:
      actor_mode: agent
      environment: production
      capability: segmented_process_environment
```

This is illustrative only. It is not an accepted `ota.yaml` surface until activation and schema
review. The provider mapping is intentionally absent.

## Evidence, Privacy, Receipts, And Archives

Human and JSON output expose only:

- requirement, provider-binding, admission, plan, and transaction identities;
- provider and binding classes at the permitted disclosure level;
- availability, authorization, materialization, injection, cleanup, and authority posture;
- exact selected scope and recipient segments; and
- typed refusal or boundary-failure reasons.

They never expose values, transformed values, hashes, lengths, prefixes, raw provider paths,
credentials, descriptors, or reusable handles. Provider paths, secret names, reference labels,
tenancy details, workload metadata, and provider metadata may themselves be sensitive. Public
output uses redacted or opaque identities plus a disclosure class.

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

## Initial Pressure Bar

Activation names one concrete adapter from a real design-partner or pressure case. Pressure uses
synthetic canary material so leak scanning is possible without retaining real values. It proves:

- positive late delivery to one exact supported recipient segment;
- non-recipient segmentation when the adapter can enforce it;
- honest process-tree inheritance for process-environment delivery;
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
