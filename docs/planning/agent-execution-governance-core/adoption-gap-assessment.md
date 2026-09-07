# Repository Execution Governance: Adoption Gap Assessment

Status: draft builder input; inactive. This document does not activate implementation, change an
existing acceptance bar, assign a version, or claim an integration is shipped. V12.1 continues
under its existing plan and independently reviewed step gates.

Assessment basis: committed Core `777c42abab3dd329c0a2f9102b3f5e3dd8b12ffd` on
`1.6.28-implementation`, reviewed 2026-09-07. Uncommitted Step 7 implementation and this draft are
not treated as established product state. This is a targeted planning reconciliation of local
specifications, plans, selected implementation surfaces, and the founder's supplied conversation.
It is not a complete source audit, hosted-evidence review, market study, or competitor assessment.
The conversation supplies hypotheses, not authoritative product facts or implementation permission.

## Intended Outcome

A maintainer adopts Ota for one repository workflow. Humans, CI, and different AI agents can use
that workflow successfully. Within the explicitly adopted protected boundary, omitting a flag,
changing tools, editing the contract, or presenting a different receipt cannot expand authority or
satisfy verification obligations that were not met. The maintainer can understand the outcome and
keep the integration working without permanent founder operation.

The ambition is to make Ota essential wherever software work needs a consistent execution contract,
an enforceable authority boundary, and evidence another system can check. Essentiality is a product
hypothesis to demonstrate through repeat use and consequential decisions, not a valuation claim.

The product has four connected layers:

| Layer | Immediate user value | Evidence of adoption |
| --- | --- | --- |
| Repo readiness | Diagnose prerequisites and run the correct verification path | Maintainer repeatedly uses and updates the contract |
| Repo execution governance | Apply requirements, admission, and supported runtime restrictions | A real operation depends on the governed path |
| Agent execution governance | Preserve those restrictions across commands and tools | Useful agent work succeeds and bypass controls fail |
| Organization governance | Administer the same semantics across repositories | Teams pay to operate and coordinate controls they already need |

These are not four consecutive completion projects. Future milestones should connect the layers
through one complete use case while preserving the single-active-version discipline.

## Evidence And Ownership Baseline

Use these existing owners rather than introducing parallel evaluators or evidence systems:

| Surface | What the reviewed material establishes | What it does not establish |
| --- | --- | --- |
| [Agent commands](../../spec/command-reference.md) and [contract](../../spec/contract-reference.md) | Selected `--agent` closure admission, discovery, and refusal controls | Repository-file presence intercepts all execution |
| `src/crossing.rs` and crossing specifications | Derived crossing requirements; configured protected authority for eligible non-agent work | Omitting `--agent` authenticates a human, or a grant overrides agent refusal |
| [V12](../v12/plan.md) | Ownership of typed effects, selected admission, negative controls and bounded refusal evidence | Positive database execution or arbitrary shell-effect enumeration |
| [V12.1](../v12.1/plan.md) and [current state](../../ai/current-state.md) | Active first secret-delivery transaction work and command-admission foundations | Completion of the provider transaction, support, or whole-session governance |
| [V12.2](../v12.2/plan.md) | Inactive monotonic contract-authored crossing requirements | Protected repo enrollment, policy-authored crossing, or agent privilege escalation |
| [V12.3](../v12.3/plan.md), [V12.4](../v12.4/plan.md), [V12.5](../v12.5/plan.md) | Demand-gated provider and native platform carriers | Mandatory features for every repo or prerequisites to all useful adoption |
| [V12.6](../v12.6/plan.md) | Demand-gated evidence interoperability, authority history, and repository reporting | Enterprise service, universal public verification, or an already shipped merge gate |
| [Adapter/profile conformance](../adapter-profile-conformance/plan.md) | Inactive standards for observation, session rebinding, resources and support | An activated runtime adapter or registration lifecycle |
| [Authority distribution](../authority-distribution-lifecycle/plan.md) | Inactive normative installation/update/recovery requirements | An assigned implementation owner or shipped installer |
| [Incident ratchet](../incident-ratchet-application/plan.md) | Inactive, narrowly scoped archive-candidate application | Permission to make current review-only candidates writable |

The distinction between available primitives and a supported end-to-end integration must remain
visible. A gap below means the proposed outcome is not established by this assessment; the builder
must inventory reusable implementation before classifying any primitive as missing.

The strategy's Activation Discipline is reconciled with the current-state handoff to name Step 7 as
active. That planning correction does not activate implementation or change Step 7's existing gate.

## Design Decisions To Preserve

1. `ota.yaml` declares repository requirements. An external protected enrollment or execution
   boundary makes selected requirements mandatory. File presence alone cannot do that.
2. `--agent`, TTY state, parent names, and environment markers are context or ergonomic hints.
   They cannot grant authority or prove that a caller is human, agent, or independently trusted.
3. Inside a protected boundary, baseline restrictions apply irrespective of caller classification.
   Additional authority is exact, independently supplied, and reconciled by Core. Preserve existing
   agent refusal; any future agent-proposed privileged operation needs a separately reviewed model.
4. Keep ordinary local use compatible. Protected enrollment is an explicit operator choice whose
   requirements cannot subsequently be bypassed by the governed actor. Do not require remote signed
   permission for every routine local build.
5. Task admission, runtime containment, and outcome gates solve different problems. A required merge
   check cannot undo credential theft or another effect that occurred before the check.
6. Core derives one retained admission per invocation. A later provider transaction is separately
   identified. CI/provider checkout starts its own invocation and re-derives its own admission.
7. Protect both policy and the meaning of required verification. Correctly recording a passing
   weakened test does not establish that the original obligation was satisfied.
8. Trust follows protected sources and observed capabilities, not executable names, file names,
   repository authorship guesses, signatures alone, or a boolean `agent_admission` field.
9. Public and privileged verification are distinct. Public projections cannot expose protected
   correlation material or claim re-derivation of omitted private evidence.
10. Scope every guarantee to exact integrations, operations, resources, targets, and observations.
    Inventory bypass routes and unsupported behavior explicitly.

## Work Packages For Milestone Allocation

Each package requires a named version owner, review, and activation before implementation. Suggested
ownership below is a mapping proposal, not an amendment to an existing plan.

### G01. Readiness And Contract Maintenance

Weakness: a correct contract can still cost more to maintain than the ambiguity it removes.

Plan: inventory maintainer-used setup, build, test, service and runtime lanes; preserve independent
results for materially different lanes; derive prerequisites through existing typed surfaces;
retain detector provenance and explicit unresolved inputs. Generated contracts need honest version
floors and workflows must install a compatible released version. Detect drift without automatically
weakening requirements or rewriting operator-owned configuration. Reuse existing candidate review.

Acceptance: a maintainer can diagnose and run one lane from a fresh checkout, distinguish skipped
verification from success, update it after a dependency/workflow change, and explain its result.
Track onboarding time, recurring failures, contract maintenance, and founder interventions against
the prior workflow. Cache and environment shortcuts must not fabricate fresh verification.

Owner: current shipped-surface repair discipline and a future bounded adoption milestone where
needed. Connected defects can be fixed in the appropriate current release; new capability demand
must not silently widen V12.1.

### G02. Protected Repository Enrollment

Weakness: contract adoption and mandatory enforcement are currently different propositions.

Plan: define protected enrollment for one exact repository identity and governed operation set.
Retain owner, approved contract/policy basis, required executor/profile and evidence obligations in
a protected source. Repository names, paths, remotes, environment variables and caller labels cannot
manufacture enrollment. Define explicit initial adoption, update, removal, expiry and recovery.
Prevent a repo edit, alternate checkout, deleted `ota.yaml`, or older Ota binary from disabling an
enrolled operation's requirements at the controlling boundary.

Acceptance: an enrolled operation refuses missing or substituted enrollment, removed declarations,
renamed/cloned repository impersonation, and unsupported enforcement. Unenrolled ordinary workflows
retain documented compatibility. Show exactly which external system makes enforcement mandatory.

Owner: V13 owns protected enrollment for its selected provider profile. Reuse V12.2 requirements
only where applicable; V12.2's contract-only scope does not own external enrollment.

### G03. Caller-Independent Admission

Weakness: cooperative agent admission does not establish a mandatory caller boundary.

Plan: derive effective restrictions from the protected invocation context and approved contract;
retain actor attribution separately from authority. Apply requirements before setup, environment
rendering, services, hooks, proof work, logs and child execution as applicable. Inventory aliases,
workspace/member routes, modes, direct task paths, helper APIs, nested Ota, proof, CI and harnesses.
Detection heuristics may improve advice but cannot permit or weaken execution.

Acceptance: removing `--agent`, removing vendor environment variables, adding a TTY, requesting a
human lane or invoking another CLI entry point does not weaken an enrolled boundary. A standalone
binary outside that boundary cannot obtain its credentials or produce accepted evidence. Preserve
the existing non-agent crossing/agent-refusal distinction until explicitly reviewed otherwise.

Owner: V13 owns caller-independent admission for its selected enrolled operation, reusing canonical
Core evaluators and carriers.

### G04. Approved Contract And Verification Obligations

Weakness: editable source, editable tests and active authority can be confused.

Plan: distinguish the candidate working tree from the approved governance basis. Derive the exact
candidate closure while enforcing independently approved obligations and constraints. Define which
contract, test, assertion, workflow, dependency and target changes require review. Represent removal,
skip, timeout, reduced matrix, changed assertion and changed test discovery explicitly. A reviewed
change may alter obligations; the candidate cannot silently authorize its own weakening.

Acceptance: deleting a test, replacing verification with `true`, narrowing the matrix, changing a
safe task's body, altering a lockfile or invoking stale generated output either requires the named
review or fails the required obligation. Legitimate task changes remain possible. Do not claim
general test adequacy or application correctness from structural checks; trusted tests and semantic
review remain explicit responsibilities.

Owner: V13 owns verification-obligation reconciliation. Reuse semantic snapshots, proof assurance
and drift/candidate mechanisms; V12.2 alone does not own these approval semantics.

### G05. Exact Input Capture And Isolated Verification

Weakness: a commit SHA alone does not describe every executed byte or runtime dependency.

Plan: name the selected revision, dirty/untracked posture, submodules, LFS material, symlinks,
generated inputs, toolchain, lockfiles, container/build artifacts, workflow revision and effective
CWD where material. Extend existing capture mechanisms only where missing. Resolve which inputs
are captured, protected externally, or explicitly unproved. Keep editable development work separate
from the snapshot submitted for trusted verification. Reobserve before use and protect retained
inputs against subsequent mutation. Define cache keys and provenance, including untrusted cache
writers, restored outputs and shared writable state.

Acceptance: post-preview edits, branch switches, source substitution, changed submodule/LFS bytes,
poisoned cache, stale build output and CWD substitution cannot reuse prior admission or evidence.
Concurrent worktrees cannot exchange authority. Bound capture size/time and refuse incomplete input.

Owner: V13 owns exact input capture for its selected acceptance path. Inventory existing closure,
replay and semantic-identity machinery first; missing portions belong there, not in a replacement
build system.

### G06. One Governed Agent Integration

Weakness: instructions, skills and task discovery do not establish control over all agent tools.

Plan: choose one runtime and target based on an actual maintainer need and available enforcement
hooks. Prove feasibility before promising support. Define two distinct optional deployment profiles:
declared-task execution only, and exploratory execution within enforced capabilities. The latter
does not claim that every command is contract-declared. Inventory terminal, filesystem tools, MCP,
connectors, browser/download execution, remote tools, host sockets and delegated agents. Each route
must be mediated, bounded externally, disabled or explicitly outside the claim.

Acceptance: real coding/verification work completes; alternate shells, interpreters, nested Ota,
direct tool calls, parent/child processes and delegated agents cannot widen capabilities. Failure of
the adapter refuses required operations. An environment lacking a mandatory hook may offer discovery
or isolated verification, but cannot claim whole-session governance.

Owner: V13.1 owns one demanded integration, governed by adapter/profile conformance. No agent
vendor is selected by this draft, and no provider compatibility is assumed.

### G07. Enforced Runtime Capabilities

Weakness: admitting an approved task does not constrain malicious code inside it.

Plan: select an enforceable profile covering needed filesystem writes/reads, network, credentials,
processes, mounts, IPC, host control sockets, privilege escalation and resource bounds. Distinguish
protected controller/tooling from the untrusted workload. Define effective-observation methods and
freshness per dimension; configuration presence or acknowledgement cannot establish enforcement.
Decide how required downloads/setup work without quietly granting broad runtime network access.

Acceptance: a test subprocess attempting prohibited access is denied; symlinks, helper processes,
daemonization, inherited handles, container socket access and controller failure cannot provide a
fallback route. Test allowed work too. Unsupported DNS/egress/OS features refuse only where required
and remain explicitly unproved elsewhere. Do not claim protection against privileged/kernel escape.

Owner: V13.1 owns target-specific integration, while adapter/profile conformance owns the shared
standard. No new OS carrier is implied without its demand gate.

### G08. Protected Credentials And Consequential Effects

Weakness: withholding trusted receipts is insufficient if the agent already holds effect authority.

Plan: finish V12.1's exact secret-delivery boundary as specified. Separately inventory credentials
available through ambient environment, files, agents, connectors and provider metadata. For each
future consequential operation, define typed intent, exact resource, least-privilege capability,
independent authority, transaction, freshness, replay and cleanup before adding an adapter. Compare
selected-process secret delivery with operation-specific brokering; neither is universally stronger.

Acceptance: another command cannot acquire the protected capability. Wrong resource, substituted
recipient, replay, lost response, cancellation and recovery refuse or retain uncertainty. Explicitly
record that a process receiving a credential can use or copy it within the available runtime bounds;
environment delivery alone cannot prove action-specific use or absence of exfiltration.

Owner: V12.1 for its existing secret scope; future named demand-backed effect slices for publication,
deployment or other operations. No general privileged-agent override or database executor is added
through this package.

### G09. Trusted Evidence Production And Verification

Weakness: a correctly shaped receipt or signature does not prove producer authority or completeness.

Plan: retain exact invocation, input, policy, approved obligations, executor/profile, observations,
result, coverage and terminal state through existing evidence machinery. Protect the producer,
signing access where applicable, storage and verification configuration from the workload. Treat
logs and agent narratives as untrusted content. Consumers independently reconcile intended scope,
trusted producer and freshness; authentic evidence for the wrong obligation must refuse.

Acceptance: fabricated JSON, copied identities, re-signed inconsistent records, wrong revision,
omitted failing lane, stale receipt, altered obligation and alternate producer do not satisfy the
consumer. Failures to publish or durably retain evidence remain explicit. Historical validity must
not imply present execution authority, current revocation knowledge or fresh verification.

Owner: reuse current proof/archive semantics. V12.6 owns portable and cross-repository consumer
surfaces only after its independent-consumer and organizational-demand gates. V13 owns its first
bounded gate integration and cannot claim V12.6 interoperability.

### G10. Required Repository Outcome Gate

Weakness: generated CI configuration is not an independently enforced merge/release requirement.

Plan: integrate one provider-owned protected operation first. Establish the trusted workflow source,
required check producer, exact revision or merge-group input, fork permissions, required obligations,
and protected gate configuration. Bind evaluation to the revision actually merged or released;
recheck after head/base changes. Missing, cancelled, skipped and unavailable runs cannot silently
satisfy the requirement. Inventory bypass administrators and record the residual trust explicitly.

Acceptance: a forged same-name check, workflow edit, force push, changed merge base, stale success,
concurrent completion and untrusted fork artifact fail to satisfy the gate. Run verification in a
restricted executor even when its controller is trusted. Do not run untrusted PR code with gate
administration or broad publication credentials. Apply required checks regardless of guessed
human/agent authorship.

Owner: V13 owns one bounded gate adapter using canonical CI/evidence semantics. Provider branch
protection remains provider-owned; Ota must observe/reconcile its required posture or name what it
cannot verify. Gate installation must be explicit and reviewable, not silently mandatory in a PR.

### G11. Session Continuity And Lifecycle

Weakness: process/session continuity can outlive the authority and source it originally described.

Plan: apply fresh reconciliation on resume, reconnect, changed checkout, policy update and runtime
replacement. Bind delegated work and descendants to their exact allowed scope; no delegation may
expand authority. Define interruption, timeout, resource limits, orphan detection, cleanup and crash
recovery for the owned resources. Retrying ambiguous operations requires reconciliation first.

Acceptance: expired/revoked context, different repository, replacement runtime and zombie session
cannot inherit old admission. Controller crash and daemonizing descendants leave explicit terminal
or cleanup-uncertain state. Ota-created resources use creation intents/handles; externally owned
resources cannot be relabeled as Ota-owned or claim Ota cleanup support.

Owner: adapter/profile conformance requirements plus each activated runtime/provider slice. Reuse
V12.1's bounded lifecycle where applicable; do not widen it into universal session management.

### G12. Installation, Upgrade And Recovery

Weakness: source-level security is not sufficient for an independently operated installation.

Plan: V13's first profile owns compatible Core/Protocol/Launcher/profile artifacts, verified
installation, protected enrollment provisioning, upgrades, rollback, recovery and removal. Include
key/registry freshness, revocation limits, recovery after partial update, least privilege and safe
uninstall. A recovery or break-glass path must be independently authorized, bounded and recorded.
It must never silently convert a protected operation to unrestricted execution.

Acceptance: altered package, incompatible protocol, stale registry, downgrade, partial installation,
failed update and removed service refuse protected use. One maintainer installs, upgrades and
recovers the supported boundary from published instructions without founder access.

Owner: V13 owns the first protected-repository installation, upgrade, recovery and removal path;
authority-distribution-lifecycle and adapter/profile conformance define its shared requirements.
V13.1 may consume that path for its selected runtime. The normative standards do not assign this
implementation to V12.3-V12.5 automatically.

### G13. Operator UX And Propagation

Weakness: rigorous internals can remain confusing or feel like redundant tooling.

Plan: expose one coherent flow for discovery, preview, adoption, diagnosis, execution and evidence
review using canonical commands where possible. Show declared versus currently enforced scope,
missing authority, unsupported controls, and the shortest legitimate remediation. Preserve plain
and JSON parity. Discovery must expose callable/refused/not-applicable truth without becoming an
authority token. A UI must consume Core semantics rather than invent a second evaluator.

Acceptance: an unfamiliar maintainer can explain what is mandatory, why a command refused, who can
change the requirement and what evidence is missing. Reconcile Core refs/schema/changelog, Site
command index/reference/Learn/FAQ/Glossary, Skills and mirrors, and copy-ready Examples for each
operator surface. Pin committed consumers before Core where required. Provider support claims need
their exact tested profile and release status.

Owner: every user-facing milestone. No broad website rewrite or new command spelling is approved
here. The draft itself requires no public schema, Skill, Site or Example change.

### G14. Performance, Reliability And Operational Cost

Weakness: adding control is unattractive if routine verification becomes slow, brittle or expensive.

Plan: measure readiness/admission overhead, input-capture cost, peak memory, process count, evidence
size, storage growth, concurrency and offline behavior on small and large repos. Derive numerical
budgets from the selected maintainer workflow before activation. Bound resource use and retention;
do not bypass trust checks to meet budgets. Separate cached diagnosis from fresh execution evidence.

Acceptance: representative large closures remain bounded, simultaneous jobs remain isolated,
network/control-plane failure has documented behavior, and added latency stays within the agreed
budget. Operators can identify why time or storage increased.

Owner: acceptance obligation of each activated slice; use existing runner and capture mechanisms.

### G15. Adversarial Conformance And Durable Regressions

Weakness: individually passing unit tests do not establish an enforceable deployment.

Plan: give each profile an allowed control, independent negative mutations, exact failure reason,
named side-effect sentinels, hostile input and a retained execution record. Challenge self-consistent
forgeries after recomputing hashes. Verify actual test counts and immutable source/target identities;
cross-compilation, ignored tests and overall CI success cannot substitute for the required control.

Acceptance: a fresh supported installation reproduces the result, including failure paths and
unsupported dimensions. Test adapters and fault injection cannot grant default production authority.
Record each uncovered material behavior as contract-owned/proved, explicitly bounded/not_proved,
repo-owned/outside scope, or a named platform gap. Promote incidents to durable regressions; use
incident-ratchet writability only after its own gates, not by reinterpreting current review artifacts.

Owner: each implementation slice plus adapter conformance; incident-ratchet retains its separate
inactive applicability and demand requirements.

### G16. Adoption And Economic Validation

Weakness: technical pressure, contributed fixes, stars and merged PRs do not alone prove dependence.

Plan: recruit an existing willing maintainer with one recurring problem; record the prior workflow,
cost and alternatives. Start with a reviewable integration and then separately ask whether the
maintainer wants the exact boundary mandatory. Measure repeat use across meaningful repository
changes, maintainer-owned updates, founder interventions, legitimate work blocked, actionable
refusals and evidence actually used for a review or release decision.

Acceptance: the maintainer keeps the lane, can operate it and can name a recurring benefit exceeding
its upkeep. A recommended next cohort is three independently operated repositories; that is a
proposed research milestone, not current adoption or a requirement for enterprise customers.
If readiness alone supplies value, count it honestly. If nobody wants the proposed mandatory
boundary, revise that integration/use case rather than adding more controls to manufacture demand.

Owner: product discovery attached to engineering milestones; no outreach or external work is
authorized by this draft. Existing consent and contact preferences remain in force.

### G17. Enterprise Boundary For Later Planning

Plan later from repeated organizational demand: repository inventory, protected policy distribution,
delegated administration, SSO/RBAC, approvals/exceptions, retention/search, incident workflows,
deployment options, billing, reliability commitments and support. Include tenant/repository scoping,
authorization, isolation, deletion and audit of administrative changes in the eventual design.

Core must remain independently usable: contract/effect semantics, admission, enforcement interfaces,
verification, protected local evidence and a reference deployment cannot require a paid truth oracle.
Enterprise can operate these systems at scale; it cannot rewrite Core decisions or invent assurance.
Pricing and willingness to pay require customer evidence. Do not preassign all protected evidence or
approvals exclusively to a paid product: the OSS trust boundary already depends on those primitives.

Owner: future Enterprise discovery and an explicitly authorized commercial plan, outside active
Core implementation. V12.6 remains its separately demand-gated OSS interoperability foundation.

## Roadmap Allocation

This table is the allocation result of the assessment. An existing owner means the package must be
implemented only within that plan's activated scope. A planned owner names an inactive version or
cross-cutting plan; it does not authorize implementation.

| Package | Primary roadmap owner | Supporting owner or boundary |
| --- | --- | --- |
| G01 | [Execution-contract follow-ons](../execution-contract-follow-ons/plan.md) and ordinary shipped-surface repair | Only activated, bounded follow-on groups; no V12.1 widening |
| G02 | [V13 Protected Repository Adoption](../v13/plan.md) | Reuse V11.7 protected repository mapping and [authority distribution](../authority-distribution-lifecycle/plan.md); V12.2 does not own external enrollment |
| G03 | [V13 Protected Repository Adoption](../v13/plan.md) | Consume V12.2 contract-authored requirements if activated; preserve canonical Core admission and V11.7 carrier authority |
| G04 | [V13 Protected Repository Adoption](../v13/plan.md) | Reuse semantic snapshots, drift, proof assurance and review-only candidates; do not make incident-ratchet candidates writable by implication |
| G05 | [V13 Protected Repository Adoption](../v13/plan.md) | Reuse closure, replay-input, retained-byte and isolation foundations rather than creating a parallel build system |
| G06 | [V13.1 Governed Agent Runtime Integration](../v13.1/plan.md) | One demanded agent/runtime and enforcement hook; no vendor or universal-session claim is selected here |
| G07 | [V13.1 Governed Agent Runtime Integration](../v13.1/plan.md) under [adapter/profile conformance](../adapter-profile-conformance/plan.md) | V12.3-V12.5 supply only demanded provider/native carriers; none is mandatory for every integration |
| G08 | V12.1 for exact Secret Manager delivery; later named effect plans for other operations | No generic privileged-agent override or speculative adapter catalog |
| G09 | V12/V12.1 for owned local evidence; V12.6 for demanded portability and independent consumption | Earlier gate integration may consume existing evidence but cannot claim V12.6 interoperability |
| G10 | [V13 Protected Repository Adoption](../v13/plan.md) | Provider branch protection remains external authority and must be independently observed or explicitly bounded |
| G11 | Each activated provider/runtime/carrier slice | [Adapter/profile conformance](../adapter-profile-conformance/plan.md) owns shared lifecycle requirements, not a universal session manager |
| G12 | [V13 Protected Repository Adoption](../v13/plan.md) | [Authority distribution](../authority-distribution-lifecycle/plan.md) and adapter/profile conformance define shared requirements; V13 owns the first installation path |
| G13 | Every activated user-visible slice | Core, Site, Learn, FAQ, Glossary, Skills/mirrors and Examples propagate only when directly affected |
| G14 | Every activated implementation slice | Numerical budgets come from the exact selected workflow and target |
| G15 | Every activated implementation slice and adapter/profile conformance | Immutable pressure is profile-specific; cross-compilation and overall CI success are insufficient |
| G16 | Activation and closure gate for every adoption milestone | Product discovery may stop or narrow a technically complete integration |
| G17 | Future explicitly authorized Enterprise plan | V12.6 remains the demand-gated OSS interoperability foundation, not the hosted product |

V13 and V13.1 are distinct: protected repository adoption governs whether changes can be accepted,
while a governed agent runtime constrains what happens during an authoring session. Either may
deliver value without the other, and each keeps its own demand, threat model, target, activation and
proof boundary.

## Dependency Map

These are integration dependencies, not permission to implement and not a requirement to build the
packages in numerical order. Reusable primitives may exist earlier, but a milestone cannot claim a
dependent outcome until the named prerequisites are established for its exact target.

| Outcome | Required packages | Boundary |
| --- | --- | --- |
| Repeatable readiness adoption | G01, G13-G16 | Does not establish mandatory execution or agent containment |
| Protected repository enrollment | G02, G12-G15 | Requires one external enforcement owner; repository configuration alone is insufficient |
| Caller-independent protected admission | G02-G05, G12-G15 | Applies only to the exact enrolled operations and captured inputs |
| Required verification gate | G02, G04-G05, G09-G10, G12-G15 | Governs acceptance of agent- or human-produced changes; it does not govern every action in the authoring session |
| Governed agent execution integration | G02-G08, G11-G15 | Requires one enforceable runtime profile and a complete bypass inventory |
| Consequential operation | G02-G05, G07-G09, G11-G15 | Requires an independently owned capability unavailable outside admitted execution |
| Organization operation | Proven lower-layer use plus G16-G17 | Enterprise coordination cannot substitute for incomplete OSS truth or enforcement |

G16 is a continuing falsification gate, not a final marketing step. If maintainers will not retain
or make the selected boundary consequential, the owning milestone must narrow or stop rather than
advancing because its technical dependencies exist.

## Recommended Milestone Shape

These are allocation envelopes, not new versions or activation dates:

1. Finish V12.1 and its existing release/pressure obligations. Record resulting reusable interfaces
   and remaining limits; this brief adds nothing to V12.1's closure bar.
2. Reconcile G01-G17 against actual code and plans. Confirm or amend the allocation table, resolve
   compatibility and trust-source decisions, and select a real maintainer workflow. V12.2, V13 and
   V13.1 keep their own activation bars; new work cannot be slipped into one because it follows
   numerically.
3. Build one complete trusted verification/adoption path through G02-G05 and G09-G10, with only the
   packaging, UX and runtime containment it requires. Agent-specific session control need not be a
   dependency for independently verifying agent-produced changes. This is a recommended first
   adoption target, contingent on the operator's actual need and the required plan approvals. Its
   claim is agent-agnostic acceptance governance, not whole-session agent governance.
4. Extend to one governed agent execution integration through G06-G07 and G11 when a supported
   runtime can enforce it. If the operator's primary pain is in-session harm, prioritize this path
   instead through an explicit roadmap decision; a post-work gate cannot solve that harm.
5. Add one demanded consequential operation using G08 and the completed foundations. Do not start a
   catalog of speculative provider adapters. If secret delivery meets the need, reuse V12.1 rather
   than introducing a second mechanism.
6. Expand integrations and organization administration only after repeat operation establishes the
   value. Preserve V12.3-V12.5 demand gates and V12.6's independent-consumer/organization gates;
   formal deferral is preferable to implementing unused carriers for nominal roadmap completeness.

Packaging and conformance owners may be prerequisites to the first independent deployment; they
must be named and activated explicitly. Do not invent a V12.7 merely to collect this list. A new
version is justified when an unowned bounded capability has a clear dependency graph and acceptance
bar, not because the strategy needs another number.

## Builder Handoff Checklist

- Classify each G-number as existing/reused, current-plan obligation, future owner needed, conditional
  demand, or intentional non-goal. Point to source/tests as well as planning prose.
- Name one implementation owner per missing capability; identify shared interfaces and prerequisites.
- Resolve protected enrollment source, contract-update approval, trusted evidence producer, gate
  consumer, first target and exact maintainer workflow before fixing a schema or CLI spelling.
- Keep policy source, admission owner, executor owner and evidence verifier distinct; do not let a
  repository-authored requirement manufacture its own authority.
- Specify independent positive/negative controls, exact source/OS coverage, measurable operator value,
  performance budget and rollback/recovery behavior for each milestone.
- Record public/private disclosure and historical compatibility before emitting new identities.
- Reconcile all affected first-party surfaces and immutable consumer pins at the implementation
  boundary; this internal draft itself introduces no public product behavior.
- Submit the resulting bounded plan amendments for independent review. Update current-state only at
  the actual reviewed planning/activation boundary; retain it as the single live handoff.

## What Success Must Mean

The first adoption milestone establishes an agent-agnostic outcome. One maintainer can say: "I can
use different agents, the same required verification still applies before accepted changes, and the
evidence helps me decide what to accept. I can keep this working myself." This does not claim that
Ota governed every tool or action in the agents' authoring sessions.

A later whole-session milestone is separate. It succeeds only when one exact agent/runtime
integration completes useful work while its declared capability and consequential-operation
boundaries resist the named bypasses. A protected operation is unavailable outside the admitted
path, and interruption and cleanup retain explicit evidence.

Fleet breadth, additional carriers and commercial coordination should compound demonstrated value
rather than substitute for either result.
