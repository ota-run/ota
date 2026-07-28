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

# V11.21: Enforced Sandbox Policy Application

Status: planned and inactive. V11.20 remains the active slice. V11.21 must not begin implementation
until V11.20 closes.

Source direction:

- [V11 plan](../v11/plan.md)
- [V11.3 plan](../v11.3/plan.md)
- [V11.4 plan](../v11.4/plan.md)
- [V11.6 plan](../v11.6/plan.md)
- [V11.8 plan](../v11.8/plan.md)
- [Contract reference](../../spec/contract-reference.md)
- [JSON output reference](../../spec/json-output-reference.md)

V11.21 theme:

- apply Ota's compiled runtime-boundary policy through cooperating execution providers
- fail closed when a provider cannot enforce an authoritative selected-lane control
- bind compiled and applied policy identity into runner-authored execution evidence

V11.8 remains the shipped foundation for:

- contract-owned `runtime_boundary` declarations
- selected-lane filesystem and network policy compilation
- destination-shaped outbound policy
- the first `codex_local` capability profile

V11.21 does not reopen or replace that model. It closes the narrower remaining boundary between a
compiled profile and a provider that actually applies it.

## Product principle

`ota.yaml` is the canonical execution-governance spec. Ota resolves one effective selected-lane
boundary. A provider may enforce that boundary, but it must not reinterpret or widen it.

The authority chain is:

```text
contract
  -> selected task/workflow closure
  -> canonical sandbox policy
policy pack / hosted policy
  -> explicit restriction overlays
canonical sandbox policy + restriction overlays
  -> effective sandbox policy
  -> provider capability negotiation
  -> provider-specific application
  -> runner-witnessed applied-policy evidence
  -> receipt/archive verification
```

Each arrow must preserve identity. A provider configuration is not a second authoring surface.

Ota cannot stop an unrestricted developer or agent from bypassing Ota with raw shell. This slice
provides enforcement only when the execution chokepoint delegates to Ota or consumes the exact
compiled policy. Mandatory CI gates, agent harnesses, and enterprise runners are separate adoption
layers that can make that delegation non-optional.

## Problem

Ota can currently publish:

- callable and refused task/workflow lanes
- effective safe-closure posture
- filesystem writable/protected boundaries
- network defaults and outbound targets
- destination-shaped constraints

That is stronger than prose, but a profile alone does not prove runtime enforcement.

The remaining OSS gaps are:

1. no canonical provider-neutral policy identity for the complete selected execution closure;
2. no capability negotiation that distinguishes `enforced`, `advisory`, and `unsupported`;
3. no fail-closed admission when an authoritative control cannot be applied;
4. no runner-authored binding between compiled policy and provider-applied policy;
5. no archive verification that can reject a receipt whose enforcement claim does not reconcile.

## Existing contract shape

V11.21 should not add a second sandbox-policy tree. It consumes the existing selected-lane
`runtime_boundary` model.

```yaml
agent:
  safe_tasks:
    - verify
  writable_paths:
    - coverage
  protected_paths:
    - ota.yaml
    - .github/workflows

tasks:
  verify:
    safe_for_agent: true
    command:
      exe: npm
      args:
        - test
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
        writable_paths:
          - coverage
        protected_paths:
          - ota.yaml
          - .github/workflows
      network:
        default: deny
```

The same rule applies to workflow-scoped runtime boundaries: workflow truth may restrict the
selected path, while task-owned concrete fields remain authoritative for the executable task.
Incompatible same-field values that cannot be reduced to monotonic narrowing must remain validation
errors, not provider guesses.

## Canonical OSS domain

V11.21 should introduce one internal, serializable `SandboxPolicy` envelope derived from the same
selected-lane evaluator used by task/workflow discovery and dry-run.

It must include:

- contract snapshot identity
- selected task or workflow identity
- complete dependency/phase closure identity
- resolved target OS, architecture, and applicable platform/ABI identity
- an ordered policy-segment graph for every executable step in the selected closure
- selected mode, context, backend, lifecycle, and interaction posture for each segment
- filesystem policy:
  - repo-root mode
  - writable paths
  - protected paths
- network policy:
  - default posture
  - outbound targets
  - destination constraints
  - source posture
  - enforcement locus: `runtime`, `application`, or `advisory`
- relevant effect and external-state posture
- authoritative, advisory, and unresolved requirement classifications
- deterministic semantic identity over canonical policy fields

Provider names, provider YAML, mount syntax, runner defaults, and formatting must not participate in
the canonical policy identity. Provider-relative states such as `unsupported` must not appear in
this domain.

Target platform means the platform whose declared task variant and runtime behavior Ota resolved.
For native execution it is the native target; for container execution it is the selected image
platform. The host runner platform remains separate provider evidence and must not silently replace
the target platform under emulation or cross-platform execution.

### Phase- and step-scoped policy

One closure-wide filesystem or network intersection is not a valid execution model. Hydration,
setup, service start, verification, and finalization may have different declared boundaries.

The canonical `SandboxPolicy` must therefore bind an ordered graph of `SandboxPolicySegment`
records. Each segment must carry:

- stable segment identity and selected step/task identity;
- phase and dependency ordering;
- resolved target OS, architecture, and applicable platform identity;
- mode, context, backend, lifecycle, and interaction posture;
- its own filesystem, network, effect, external-state, and application-obligation truth; and
- a deterministic segment-policy identity.

The closure policy identity binds the graph topology and every ordered segment identity. Ota must
not flatten differing segment policies into a union that under-restricts one step or an intersection
that prevents another from running.

Adjacent segments with identical effective policy may share one boundary. Segments with different
effective policy require either:

- an attested policy transition after the previous segment has terminated and before the next
  segment starts; or
- separate provider boundaries and segment leases.

Concurrent segments may share a boundary only when their effective policies are identical.
Otherwise the provider must isolate them or Ota must refuse. No process from an earlier segment may
survive into a later segment whose policy would have refused that process or capability.

### Required domain separation

V11.21 must keep five objects separate:

1. `SandboxPolicy`
   - canonical selected-lane requirements and ordered segment graph derived from contract truth;
2. `SandboxRestrictionOverlay`
   - an explicit, identified policy restriction from a policy pack or hosted enterprise policy;
3. `EffectiveSandboxPolicy`
   - the deterministic per-segment meet of canonical policy and explicit restrictions;
4. `ProviderCapabilityDescriptor`
   - what one provider can enforce, without changing policy;
5. `ProviderApplicationEvaluation`
   - what the provider actually applied and what Ota witnessed.

Each object needs its own semantic identity and source. Capability must never become policy, and
observed application must never be written back as declaration.

An explicit restriction overlay may narrow execution. It must come from a contract-referenced
policy pack or an authenticated hosted-policy authority that is authorized for the selected repo
and lane. A provider default is not policy authority and Ota must never convert it into an overlay.
If a provider would impose undeclared behavior, Ota must refuse with a typed
provider-policy mismatch.

### Restriction algebra

The effective policy is derived independently for every segment. Each supported control family must
have a deterministic, total `meet(canonical, overlays)` operation whose result is one of:

- one effective value that is no broader than the canonical value;
- an explicit denied value when the restrictions eliminate the capability; or
- a typed conflict/refusal when the inputs are incomparable or cannot be represented safely.

The initial algebra is:

- repo-root mode: `read_only` is narrower than `read_write`;
- writable paths: intersection over canonicalized path-reachability regions, not literal path
  strings; an overlay may remove writable reachability but never add it;
- protected paths: canonicalized set union; an overlay may add protection but never remove it;
- network default: `deny` is narrower than `allow`;
- outbound hosts, service aliases, and destinations: typed set intersection; an overlay may remove
  destinations or add constraints but never add reachability;
- destination constraints: logical conjunction using the control's canonical typed form; an
  unrepresentable or contradictory conjunction becomes denial or typed refusal, never broad access;
- secret bindings: an overlay may remove a binding or narrow its exposure but never introduce a
  new secret source;
- process, lifecycle, and isolation controls: use an explicit control-specific partial order;
  incomparable postures produce typed refusal;
- effects and external state: an overlay may require stronger review or denial but may not authorize
  an effect absent from canonical repo truth.

An omitted field in an overlay is neutral, not an empty set. Overlay order must not affect the
result, and the effective-policy identity must bind the canonical identity, every authorized
overlay identity, and the derived result for each segment.

## Provider adapter contract

Every provider adapter must publish a machine-readable capability descriptor before execution.

Capabilities must be granular enough to distinguish:

- read-only repository root
- writable path carve-outs
- protected-path write denial
- external IP-network connectivity denial
- host/domain/service-alias allowlisting
- destination-level constraints
- secret binding
- process and lifecycle isolation
- cleanup and interruption handling

Each runtime-owned control in the effective policy must resolve to exactly one provider application
state:

- `enforced`: the provider can apply and attest the control;
- `advisory`: the provider can carry or report it but cannot enforce it;
- `unsupported`: the provider cannot represent it.

Application-owned controls such as `authoritative_app_enforced` destination constraints are not
provider capability failures. They remain separate application obligations with their own evidence
state:

- `attested`
- `not_proved`
- `contradicted`

Sandbox enforcement must not promote an application obligation to `attested`. Policy may separately
require application evidence before accepting a higher-level proof or merge gate.

Admission rules:

- an authoritative runtime-owned control that resolves to `unsupported` or `advisory` refuses
  before hydration, setup, service start, or task execution;
- an authoritative application-owned control remains an explicit obligation and must not be
  mistaken for runtime enforcement;
- an explicitly advisory contract input may remain advisory, but output must preserve that weakness;
- explicit policy restrictions may narrow canonical policy and must produce a separate effective
  policy identity;
- provider defaults may not narrow or widen policy; any policy-relevant difference from effective
  policy is a typed provider-policy mismatch;
- no provider may silently discard an outbound destination constraint because it can enforce only
  the first-hop host.

## CLI surface

V11.21 should extend existing execution surfaces rather than invent a parallel task runner.

Direction:

```text
ota run <task> --agent --sandbox-target <target>
ota up --workflow <workflow> --agent --sandbox-target <target>
```

The exact flag name is subject to normal command review, but the semantics are fixed:

- target selection is an explicit execution-affecting request;
- an unavailable or incompatible target is refused, never treated as advisory;
- target selection must not change the resolved execution mode, context, backend, or task body;
- dry-run and real execution use the same admission decision;
- `ota tasks --json`, `ota workflows --json`, and dry-run JSON continue to expose the canonical
  compiled policy for discovery;
- real execution adds applied-policy and provider-attestation evidence;
- a human invocation without agent enforcement must not be relabeled agent-safe merely because it
  uses a sandbox target.

Omitting `--sandbox-target` must not bypass enforcement:

- when the selected agent lane has authoritative runtime-owned controls, Ota resolves the target
  from contract-owned mode/context/backend truth and any explicitly selected policy-owned target;
- when exactly one compatible enforcing target remains, Ota selects it deterministically and
  reports that resolution;
- when no compatible target exists, or more than one remains without a contract-owned default, Ota
  refuses before preparation;
- an explicit target may choose only among compatible enforcing targets and cannot change execution
  truth; and
- when no authoritative runtime-owned control applies, ordinary V11.3 agent admission remains
  unchanged.

JSON refusal must identify:

- requested or automatically resolved provider target
- canonical sandbox policy identity
- applicable restriction-overlay identities
- effective sandbox policy identity
- unsupported or conflicting control
- required enforcement posture
- provider capability state
- supported alternatives when Ota can establish them

## Applied-policy handshake

Before any provider mutation or child process starts:

1. Ota resolves the selected closure, target platform, and canonical segment graph.
2. Ota resolves every authorized restriction overlay and derives the effective segment graph.
3. Ota resolves the explicit or contract-owned provider target.
4. The provider returns its capability descriptor and adapter identity for the target platform.
5. Ota derives an ordered provider application plan for runtime-owned segment controls.
6. Ota refuses if any authoritative runtime-owned segment control cannot be enforced.
7. Ota registers an idempotent finalizer and acquires a pre-mutation cleanup lease bound to the
   runner transaction, provider, intended boundary namespace, and application plan.
8. The provider applies the first segment plan and returns initial application evidence.
9. Ota verifies that the applied policy, boundary lease, target platform, and capability set match
   the admitted segment plan.
10. Only then may hydration, setup, services, or task execution begin.

If provider application creates any partial state and then fails, Ota must invoke the registered
finalizer. The terminal record must distinguish confirmed cleanup, incomplete cleanup, and unknown
cleanup even when no user task started. A provider that cannot grant cleanup authority before its
first mutation cannot support authoritative enforcement.

The provider application identity must bind:

- canonical sandbox policy identity
- ordered canonical and effective segment-policy identities
- every restriction-overlay identity
- effective sandbox policy identity
- resolved target OS, architecture, and applicable platform identity
- provider target and adapter version
- provider capability-descriptor identity
- rendered provider-policy identity
- applied boundary/session identity
- enforcement states by control
- start time and runner-owned transaction identity

A self-declared `applied: true` flag is insufficient.

Application evidence must come from the strongest control-specific source the provider exposes:

- provider/kernel/runtime state inspection where the applied boundary is queryable;
- runner-owned enforcement canaries where a control can be tested safely before user work starts;
- `unsupported` when neither inspection nor a trustworthy canary can establish enforcement.

A canary must exercise the same applied boundary and exact control class as the user task. A
generic failed process or unrelated denied operation is not enforcement evidence.

### External attestation trust

Local OCI enforcement may use Ota-runner-owned runtime inspection and canaries because Ota owns the
boundary transaction directly. An external adapter claim requires an authenticated attestation
protocol.

Every external capability, initial-application, transition, terminal-application, and cleanup
attestation must bind:

- registered issuer/workload identity and trust-root identity;
- runner transaction, provider adapter, target platform, segment, policy, and boundary identities;
- an Ota-generated single-use challenge nonce;
- issued-at and expiry timestamps within a configured freshness window;
- signature or authenticated-transport evidence and verifier identity/version; and
- prior-attestation identity where the record advances an existing boundary state.

Ota must verify issuer authorization, signature or authenticated channel, nonce equality, freshness,
transaction binding, and single use before accepting the record. Replayed, expired, revoked,
mis-scoped, or unverifiable attestations are refused. Archives must retain the challenge,
attestation, verification result, and trust-root snapshot needed to re-verify the claim without
treating an internally consistent provider document as proof.

### Enforcement lifetime

Initial application is not enough for a completion receipt.

Ota must hold the pre-mutation cleanup lease before provider application begins. The provider must
finalize every created segment boundary after success, failure, timeout, cancellation, interruption,
or partial application. Enforcement evidence must establish one of:

- the applied boundary is immutable for the lease lifetime; or
- Ota re-inspected every authoritative runtime control at finalization.

Terminal states are:

- `enforced_through_completion`
- `enforcement_lost`
- `enforcement_unknown_after_interruption`
- `not_started`
- `refused`

Only `enforced_through_completion` may support a completed sandbox-enforcement claim. Loss of the
boundary, failed terminal inspection, missing finalization, or an unbound replacement boundary must
remain explicit and must not serialize as completed enforcement.

## First enforcement target

The first real OSS enforcement target should be a reference OCI/container adapter because it gives
Ota a concrete, reproducible boundary under its existing container execution architecture.

The target must not turn a native-only lane into container execution. The reference adapter applies
only when the selected contract path already declares a compatible container execution mode and
context. Incompatible native, remote, or provider-owned paths must refuse with supported
alternatives rather than changing execution truth.

The first target must prove only what its runtime can enforce:

- read-only repository root
- explicit writable mount carve-outs
- protected-path write refusal
- external IP-network connectivity denial
- isolated process and cleanup lifecycle

For the reference OCI target, external IP-network denial means:

- no provider bridge, host network, inherited service network, external route, or DNS path;
- loopback remains local to the isolated container namespace and is not claimed absent;
- Unix sockets and mounted runtime sockets are filesystem/capability channels, not covered by the
  network-denial claim; and
- Docker, Podman, containerd, SSH-agent, host-control, and similar runtime sockets must not be
  mounted into the boundary unless a later typed control explicitly governs them.

The filesystem application plan must canonicalize host and container paths before admission. It
must reject:

- symlink traversal or aliasing that makes protected content writable;
- writable hardlink aliases to protected files;
- nested or overlapping mounts that expose protected content through a writable mount; and
- provider-added writable paths absent from effective segment policy.

Stock OCI/container networking does not by itself prove host/domain or destination allowlists. The
reference adapter must therefore:

- enforce the bounded external-connectivity denial above where requested;
- refuse authoritative targeted-egress policy when no cooperating proxy or network-policy adapter
  is configured;
- never downgrade a targeted allowlist into broad network access.

`codex_local` remains a compiled capability profile until a stable integration can return an
applied-policy attestation. Profile publication alone must not be renamed enforcement.

## Receipts and archives

Runner-authored receipt evidence should add a bounded sandbox application record:

```json
{
  "sandbox_policy": {
    "canonical_identity": "sha256:...",
    "restriction_overlay_identities": [
      "sha256:..."
    ],
    "effective_identity": "sha256:...",
    "target_platform": {
      "os": "linux",
      "architecture": "amd64"
    },
    "provider_target": "oci_local",
    "provider_adapter_identity": "sha256:...",
    "capability_identity": "sha256:...",
    "rendered_policy_identity": "sha256:...",
    "applied_policy_identity": "sha256:...",
    "boundary_identity": "container:docker:...",
    "boundary_lease_identity": "sha256:...",
    "initial_application_identity": "sha256:...",
    "terminal_application_identity": "sha256:...",
    "attestation": {
      "issuer_identity": "runner:local",
      "challenge_identity": "sha256:...",
      "verification": "runner_owned"
    },
    "status": "enforced_through_completion",
    "segments": [
      {
        "id": "verify",
        "canonical_policy_identity": "sha256:...",
        "effective_policy_identity": "sha256:...",
        "boundary_lease_identity": "sha256:...",
        "status": "enforced_through_completion"
      }
    ],
    "transition_identities": [
      "sha256:..."
    ],
    "runtime_controls": [
      {
        "id": "filesystem.repo_root",
        "required": "read_only",
        "application": "enforced"
      }
    ],
    "application_obligations": [
      {
        "id": "network.destination.webhook",
        "required": "authoritative_app_enforced",
        "evidence": "not_proved"
      }
    ]
  }
}
```

The exact schema may follow existing receipt conventions, but these invariants are required:

- `status: enforced_through_completion` is derived from every authoritative runtime-owned control
  plus terminal boundary evidence, not caller supplied;
- a receipt cannot claim runtime enforcement when any authoritative runtime-owned control is
  advisory, unsupported, missing initial application evidence, or missing terminal finalization;
- application-owned obligations remain separate and cannot be promoted by sandbox status;
- the archive embeds or references the contract snapshot needed to re-derive the canonical policy;
- archive verification re-derives the selected closure, canonical policy, restriction overlays,
  ordered segment graph, target platform, and effective policy identity;
- archive verification checks applied-policy linkage without trusting human-readable summaries;
- archive verification rejects missing, reordered, or unbound segment-transition evidence;
- external-provider archives verify issuer authority, nonce, freshness-at-acceptance, signature or
  authenticated-transport evidence, replay posture, and trust-root snapshot;
- shared-pin identity and freshness used by selected destination constraints are retained through
  the canonical/effective policy and receipt linkage;
- successful sandbox enforcement proves only the selected execution boundary, not application
  output, repository completion, merge eligibility, or host-wide isolation.

## Provider-neutral integration surface

External harnesses and enterprise runners should consume the same canonical policy and return the
same applied-policy attestation shape.

The OSS interface must remain provider-neutral:

- provider adapters consume canonical policy;
- provider adapters declare capabilities;
- provider adapters return applied-policy evidence;
- Ota owns admission and reconciliation;
- no provider adapter owns repo business destinations or safe-task meaning.

This is the reusable boundary for:

- local container execution
- CI runners
- agent harnesses
- managed enterprise runners
- future VM or sandbox providers

## Validation and Doctor

Validation should reject:

- malformed runtime-boundary declarations;
- incompatible workflow/task values for the same concrete field;
- path escapes and invalid protected/writable overlaps;
- destination constraints whose shape is internally contradictory.

Doctor should report:

- lanes with authoritative runtime boundaries but no available enforcing target;
- targets that can enforce filesystem policy but not declared network policy;
- broad network effects where no explicit runtime boundary exists;
- provider capability drift when a previously supported control is no longer available.

Doctor must not claim a local provider is ready merely because the canonical policy compiled.

## Test bar

Core regressions must prove:

- canonical policy identity is deterministic and provider-neutral;
- target OS, architecture, and applicable platform identity participate in canonical policy
  identity;
- task and workflow dependency closures produce ordered segment policies without flattening
  differing phase boundaries;
- differing concurrent segment policies require separate boundaries or refuse;
- a policy transition cannot admit the next segment until the previous segment has terminated and
  the next effective policy has been applied and witnessed;
- policy-pack restrictions can narrow but not widen repo truth;
- the control-family restriction algebra is order-independent and cannot add writable paths,
  destinations, secrets, effects, or weaker lifecycle posture;
- provider defaults never modify policy; only an explicit identified restriction overlay may
  narrow canonical policy;
- omitted sandbox-target selection resolves one unambiguous contract-compatible enforcing target or
  refuses before preparation;
- unsupported authoritative runtime-owned controls refuse before any preparation or child process;
- application-owned controls remain separate obligations and are never reported as provider
  enforcement;
- dry-run and real execution return the same admission decision;
- a read-only repo rejects writes outside declared writable paths;
- declared writable paths remain writable;
- protected paths remain unwritable even when a broader parent path is writable;
- the reference target denies external IP-network connectivity without claiming loopback or
  filesystem-socket absence;
- symlink, hardlink, and nested-mount aliases cannot make protected content writable;
- inherited service networks and mounted runtime-control sockets cannot bypass the bounded network
  claim;
- targeted egress refuses when the reference target lacks a cooperating enforcement adapter;
- provider output cannot widen or omit canonical controls;
- provider application is verified through control-specific inspection or a runner-owned canary,
  never only a provider success flag;
- an OCI target cannot admit a native-only lane by silently changing its mode;
- altered capability, rendered-policy, or applied-policy identities invalidate the receipt;
- altered restriction-overlay or effective-policy identities invalidate the receipt;
- archive verification re-derives policy from the archived contract snapshot;
- successful, failed, canceled, timed-out, and interrupted runs all finalize the same boundary
  lease;
- partial provider application finalizes through cleanup authority registered before the first
  mutation, even when no user task starts;
- external attestations reject untrusted issuers, invalid signatures or channels, stale or replayed
  nonces, revoked trust, and transaction mismatches;
- missing terminal inspection, replacement boundary identity, or enforcement loss cannot become
  `enforced_through_completion`.

## Pressure bar

At least two real repositories are required:

1. a repo whose selected lane writes generated output while the rest of the worktree remains
   read-only;
2. a repo whose selected lane requires either bounded external-network denial or explicit targeted
   egress.

Each repo must prove:

- `ota validate`
- `ota doctor`
- task/workflow discovery JSON with canonical sandbox policy
- dry-run admission
- real provider-backed execution
- attempted protected-path write refusal
- intended writable-path success where applicable
- bounded external-network denial or explicit unsupported-target refusal
- receipt and archive verification
- exact Ota source/version truth
- matrix coverage for every advertised provider, target OS, and architecture lane
- an uncovered-material-behavior inventory

A green provider-backed task is not repo-global sandbox proof.

## Explicit non-goals and boundaries

- Ota does not intercept raw shell outside an adopted execution chokepoint.
- Ota does not become a universal VM, container, kernel, or network-policy runtime.
- Ota does not claim host-wide process absence or host security posture.
- Ota does not manage secrets in this slice.
- Ota does not infer business destinations from credentials or provider configuration.
- Ota does not compute shared-pin age or freshness thresholds from a declared `shared_pin.ref`;
  V11.21 preserves the contract-declared freshness posture and policy identity only.
- Payload-level webhook, recipient, relay, or tenant restrictions remain advisory unless the
  selected provider or application boundary can enforce and attest them.
- Enterprise runner registration, central policy distribution, approvals, waivers, fleet
  reporting, billing, and retention remain enterprise concerns.

## OSS and Enterprise boundary

OSS owns:

- `runtime_boundary` contract truth
- selected-lane resolution and precedence
- canonical sandbox policy and identity
- provider capability and application interfaces
- fail-closed admission
- reference provider adapters
- dry-run, JSON, receipts, archives, validation, Doctor, tests, docs, examples, and skill guidance

Enterprise owns:

- connected GitHub, GitLab, self-hosted, on-premises, and Ota-managed runner registration
- runner binding, health, scheduling, quotas, and billing
- hosted policy distribution and restrictive overlays
- approvals, grants, waivers, and exception workflows
- fleet-wide enforcement posture, drift, evidence retention, and compliance reporting
- managed provider credentials and control-plane integration

Enterprise policy may restrict or require stronger controls. It must not re-derive the repo's
selected sandbox policy, silently widen it, or invent repo-owned business destinations.

## Definition of done

V11.21 is complete only when:

- V11.20 is complete before V11.21 implementation begins;
- one canonical provider-neutral policy envelope binds the target platform and ordered per-step
  policy graph derived from existing contract/governance truth;
- at least one real provider applies a supported authoritative runtime subset and attests it
  through completion;
- omitted provider selection resolves one unambiguous enforcing target or refuses;
- unsupported authoritative runtime controls and invalid policy transitions refuse before execution;
- explicit restriction overlays use the canonical control-family algebra and provider defaults
  never become policy;
- partial provider application is finalized through cleanup authority acquired before mutation;
- external provider attestations are issuer-authenticated, challenge-bound, fresh, and replay-safe;
- dry-run, run, up, receipts, archives, schemas, docs, examples, skills, and public site references
  agree on the same semantics;
- canonical-policy, target-platform, segment, restriction-overlay, effective-policy, provider,
  capability, rendered-policy, applied-policy, boundary-lease, transition, and terminal identities
  reconcile;
- two independent real repositories satisfy the pressure bar;
- every unsupported control and unproved behavior remains explicit; and
- no output implies that Ota can constrain execution outside a cooperating runtime or mandatory
  chokepoint.
