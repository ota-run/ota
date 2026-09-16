# Agent Authority Pressure Signals

## Status

This is an internal product-pressure intake record, not retained pressure evidence. The cited issue
reports have not been independently reproduced, do not prove the reported behavior, and create no
public Ota claim, version activation, or customer commitment.

V12.1 remains the only active implementation version. This record does not widen its current
Step 7 protected-capability work, activate V12.2 or later, or authorize provider contact.

## Signals

### [OpenCode #47825](https://github.com/anomalyco/opencode/issues/47825): inherited approval must not widen a narrower denial

**Priority:** highest post-V12.1 pressure case.

**Reported failure mode:** a narrower agent denial can be weakened by inherited approval.

**Ota pressure question:** given one exact invocation and authority graph, does Ota derive the
effective decision from provenance, scope, and precedence such that a narrower explicit denial
cannot be widened by an inherited approval?

**Required future fixture:** independently reproduce the issue against an exact revision; retain
the complete policy/authority graph, selected invocation, expected denial, and substitutions for
scope, source, and precedence. The fixture must distinguish configured rules from the resolved
effective decision.

**Not proved:** OpenCode behavior, whole-session agent governance, external policy administration,
or enforcement outside an Ota-admitted invocation.

### [OpenCode #47819](https://github.com/anomalyco/opencode/issues/47819): effective authority merge semantics

**Priority:** supporting pressure case after denial precedence is established.

**Reported failure mode:** a partial agent permission configuration is added to permissive defaults
instead of replacing them, so an intended allowlist can leave undeclared powers open.

**Ota pressure question:** given one exact invocation and authority graph, does Ota derive the
effective decision from explicit provenance, scope, precedence, and merge semantics so a partial
declaration cannot inherit unbounded permission?

**Required future fixture:** independently reproduce against an exact revision and compare
the partial declaration, inherited/default authority, and resolved decision. Mutate scope,
source, precedence, and merge behavior while retaining the expected refusal or admission.

**Not proved:** Ota currently governs OpenCode, provides an equivalent agent permission model, or
can disclose complete policy truth safely.

### [Claude #92782](https://github.com/anthropics/claude-code/issues/92782): adversarial authority hypothesis

**Priority:** strategic hypothesis only.

The reported issue is not independently reproduced. It may inform adversarial test design around
scope, provenance, and precedence, but it is not evidence and must not appear in public Ota
positioning until independently verified.

## Backlog Signal Review: 2026-09-03 Reports

This is a backlog review recorded on 2026-09-09, not last-24-hours intelligence. Each report is
six days old at review time and remains an unverified pressure hypothesis until independently
reproduced at an exact upstream revision.

### [Codex #42398](https://github.com/openai/codex/issues/42398): declared read-only posture versus hidden writes

**Priority:** strongest direct repository-readiness pressure hypothesis.

The report indicates that declared read-only intent can conflict with runtime write requirements.
Its Ota pressure question is whether a selected execution plan can identify required material write
paths before admission and refuse when they are undeclared or protected. The fixture must retain the
exact declared writable posture, attempted paths, exit status, and effective filesystem outcome.

**Not proved:** Codex behavior, all hidden runtime writes, Ota support for this case, or
repository-wide immutability.

### [Hermes #101785](https://github.com/NousResearch/hermes-agent/issues/101785): blocked gate erased by successful transition

**Priority:** future state/effect-acceptance pressure hypothesis.

The report indicates that an apparently successful transition can erase a blocked human gate. This
is not part of today's repository-readiness wedge. It is future pressure for identity-bound state,
effect, and outcome acceptance: a success status must not override retained refusal or unfulfilled
approval obligations.

**Not proved:** Hermes behavior, human-approval governance, or an Ota state/effect acceptance
capability.

### [Unattended-agent practitioner discussion](https://www.reddit.com/r/ClaudeCode/comments/1vsyn8t/what_i_learned_running_25_claude_code_and_codex/)

**Priority:** practitioner-discovery lead only.

The discussion is a potential source of operator constraints and reproducible workflow examples.
It is not product evidence, a design-partner commitment, or an outreach target until the author's
reachability and interest are established.

### Working principle

> Exit `0` proves a process ended. It does not prove the promised state exists.

This principle is test-design guidance. It does not claim that Ota currently observes or verifies
all promised states.

## Daily Signal Review: 2026-09-10 Reports

These reports are independently sourced product-pressure inputs, not reproduced Ota evidence. They
do not activate implementation, widen V12.1, create a public claim, or establish that Ota currently
solves the reported behavior.

### Hermes #106456: stale installer-owned runtime lifecycle

**Source:** <https://github.com/NousResearch/Hermes-Agent/issues/106456>

**Observed behavior:** the report describes Hermes installing and owning a Node.js runtime that can
age independently of `hermes update`, with no supported upgrade path once newer dependencies require
a newer runtime.

**Affected user cost:** an installation that was initially usable can become stale without an
actionable ownership, update, or recovery route; users must reverse-engineer an internal bootstrap
implementation to restore readiness.

**Current Ota capability:** Ota contracts can declare required toolchain versions and Doctor can
compare an observed runtime against those requirements before a selected task executes. This records
whether the current runtime satisfies the declared execution requirement; it does not establish who
owns an externally installed runtime.

**Remaining named gap:** protected prerequisite lifecycle ownership: installation provenance,
approved update source, freshness, downgrade/rollback, recovery after partial update, and removal.

**Roadmap owner:** V13's first protected installation, upgrade, recovery, and removal path (G12),
under the shared requirements of
[authority distribution and lifecycle](../planning/authority-distribution-lifecycle/plan.md) and
[adapter/profile conformance](../planning/adapter-profile-conformance/plan.md).

**Not proved:** Hermes behavior, Ota ownership of the Hermes runtime, Ota support for arbitrary
tool-managed updates, safe runtime rollback, or provider/runtime lifecycle enforcement.

### Claude Code #92958: completed setup without effective workspace state

**Source:** <https://github.com/anthropics/claude-code/issues/92958>

**Observed behavior:** the report describes a Windows sandbox reporting Plan9-share attachment
completion while the promised shares are unavailable to the executor, so every shell operation
fails.

**Affected user cost:** a platform acknowledgement is presented as readiness, leaving an agent with
an unusable workspace and an operator without a distinct attachment, visibility, or access failure.

**Current Ota capability:** Ota can run declared readiness checks and retain that their selected
checks passed or failed. It does not independently observe arbitrary external sandbox mount state or
promote a provider's setup acknowledgement into effective-runtime truth.

**Remaining named gap:** postcondition-backed effective-runtime observation: bind the intended
resource, target executor, exercised access probe, observation authority, completeness, and
freshness so that acknowledgement is distinct from demonstrated usable state.

**Roadmap owner:**
[adapter/profile conformance](../planning/adapter-profile-conformance/plan.md), including its
`EffectiveRuntimeObservation` and target-specific pressure requirements. A future activated adapter
may consume that standard; this signal does not select a provider or activate one.

**Not proved:** Claude Code behavior, Plan9 semantics, Ota observation of Windows sandbox mounts,
workspace write authority, or provider/platform remediation.

### Hermes #106441: terminal bypass of high-level approval rules

**Source:** <https://github.com/NousResearch/hermes-agent/issues/106441>

**Observed behavior:** the report describes high-level approval guards permitting direct terminal
commands that read credential material and transmit it over the network because no guard matched.

**Affected user cost:** an operator can believe a tool-level approval posture governs sensitive work
while equivalent filesystem and network effects remain reachable through an unmediated terminal
path.

**Current Ota capability:** Ota derives bounded admission for selected contract tasks and records
declared effects and explicit limits. A repository contract or agent-safe-task declaration alone
cannot prevent an actor from invoking an unrelated raw-shell, inherited-credential, or alternate
execution route.

**Remaining named gap:** caller-independent enforcement and bypass resistance: an externally owned
capability boundary must make the admitted route mandatory for the specific consequential operation,
with raw-shell, alternate entry-point, inherited-credential, recovery, and administrator bypasses
inventoried and pressure-tested.

**Roadmap owner:** V13.1's one demand-backed governed agent/runtime integration, with V13's protected
repository enrollment and caller-independent admission as its required repository-side dependency.

**Not proved:** Hermes behavior, universal terminal-effect analysis, prevention of all exfiltration,
whole-session agent governance, or current Ota enforcement of an arbitrary actor's shell.

## dbmask PostgreSQL Pressure Signals

These signals come from the bounded PostgreSQL pressure design for
[dbmask](https://github.com/sealandseacat/dbmask) at upstream revision
`516f34fa17776588f975c922e86196b55611d02f`. They are design pressure, not retained
evidence: the adoption workflow remains a separate non-blocking lane using disposable synthetic
data and does not activate a new implementation slice.

### GitHub Actions service-container identity and lifecycle

**Observed boundary:** Ota can validate the selected task graph, required secret inputs, and
declared database effects, while GitHub Actions starts the pinned PostgreSQL service container,
performs its health check, and tears it down. Those workflow declarations are reviewable, but Ota
does not currently bind the exact hosted service image, runtime instance, effective readiness, or
terminal cleanup into its own observation record.

**Affected user cost:** a successful task result can be correctly bounded to its selected database
workflow while leaving the identity and cleanup posture of the hosted service provider-owned.

**Current Ota capability:** Core models Compose and host services. Hosted workflow service
provisioning remains a provider-owned boundary; Ota must not promote it to effective-runtime or
cleanup evidence without an independently derived observation.

**Remaining named gap:** GitHub Actions service-container profile: bind the exact job/service/image
and runtime instance, then retain challenge-bound effective-readiness and terminal-cleanup posture.

**Roadmap owner:** [adapter/profile conformance](../planning/adapter-profile-conformance/plan.md).
V13 may consume a later conformance profile only when it is part of an independently enforced
repository acceptance gate.

**Not proved:** GitHub service-container identity, service startup, health, teardown, Ota-owned
cleanup evidence, PostgreSQL correctness, production safety, or broad backend support.

### Independently grounded synthetic-target classification

**Observed boundary:** dbmask's pressure fixture fail-closes to one exact local PostgreSQL
driver/host/port/account/password/database tuple before recreating its synthetic source and target
databases. That is a strong repository-owned guard for the lane, but it cannot prove who owns the
process at that socket, whether the resource is ephemeral, or whether an arbitrary similarly named
database is non-production.

**Affected user cost:** a contract can declare required secret inputs and external-state effects
without converting a target label or caller-supplied URL into independent authority that the target
is safe to mutate.

**Current Ota capability:** Ota validates selected inputs, derives the execution graph, and carries
declared effects. It deliberately does not treat an environment value or repository label as proof
of target ownership or non-production status.

**Remaining named gap:** independently administered target classification and challenge-bound
effective observation for consequential external resources.

**Roadmap owner:** V12.2's monotonic authority requirements, with demand-gated V12.3 carrier work
when provider attestation is needed, under
[adapter/profile conformance](../planning/adapter-profile-conformance/plan.md).

**Not proved:** database process ownership, ephemerality, production classification, arbitrary
target safety, provider authority, or secret delivery.

## Allocation

The signals reinforce the existing Authority, Evidence, and Enforcement planes. They do not
justify a new roadmap version. After V12.1 closes or is formally deferred, the first reproducible
case should be evaluated as demand input for the existing V13 repository-governance and V13.1
demand-backed agent/runtime planning boundaries.

## Intake Standard

Before a signal can become a pressure case, retain:

- the exact upstream revision and issue reference;
- an independently reproduced fixture and execution transcript;
- the selected invocation, authority inputs, effective decision, and expected refusal or admission;
- explicit provenance, scope, precedence, replay, and disclosure substitutions; and
- `proven_facts` and `not_proved` statements suitable for the pressure evidence registry.

Do not add an entry to `evidence-manifest.json` until a hosted or otherwise retained,
revision-bound matrix satisfies that standard.
