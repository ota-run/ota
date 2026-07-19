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

# V11.18: Managed Lifecycle-Sequence Proof

Status: planned. This follows V11.16 and V11.17 under version discipline. It reuses V11.11 proof
boundaries and V11.16 boundary provenance; it does not reopen V11.15's generated build/test
projection or silently claim upstream CI equivalence.

## Problem

Repositories often keep a material lifecycle smoke sequence in CI shell:

```sh
service start
service stop
```

That is neither a finite task nor a conventional long-running runtime proof. A shell sequence can
leave a process behind when a later step fails, stop a service Ota did not start, or make a green
result look like functional application proof when it only established lifecycle command success.

Caddy is the first pressure signal. Its upstream contributor CI starts and stops the built Caddy
binary. Ota's managed build/test lane does not own that sequence today. Replacing it with `caddy
version` would be a false narrowing; treating it as generic shell would preserve duplicate lifecycle
truth and no cleanup guarantee.

## Product Principle

Lifecycle command authority belongs to the declared service manager. A lifecycle proof names the
service and the expected state transition; it never repeats `start`, `stop`, readiness, or status
commands in workflow shell or proof metadata.

The runner owns the lifecycle transaction. Its exact phase order is fixed:

1. execute the selected workflow's existing `prepare`, `setup`, and `run` closure as the
   prerequisite/build phase;
2. resolve the selected service dependency closure in topological order, observe its pre-state,
   and acquire one transaction-local cleanup lease for each service proven inactive;
3. start leased services in dependency order;
4. evaluate their declared readiness in dependency order;
5. execute the optional finite lifecycle assertion task after all required services are ready;
6. teardown every leased service in reverse dependency order from a runner-owned finalizer,
   including after start, readiness, assertion, or interruption failure;
7. verify stopped state only through a positive manager-state observation, then archive the ordered
   evidence and derive the qualified proof verdict.

Ota must never stop a pre-existing service simply because it shares a declared name or endpoint.

## Contract Shape

V11.18 adds an additive, service-reference-only lifecycle declaration under an existing workflow
proof block:

```yaml
workflows:
  contributor-smoke:
    run:
      task: build
    proof:
      lifecycle:
        services:
          - caddy
        assertion:
          task: caddy-lifecycle-assert
```

`workflows.<name>.proof.lifecycle.services[]` references services already declared under
`services.<name>`. `workflow.run.task` remains the existing prerequisite/build phase and executes
before Ota acquires any lifecycle lease. `proof.lifecycle.assertion.task` is optional, must name a
finite task, and executes exactly once after selected services are ready. It is the only
contract-owned post-readiness assertion slot; an omitted assertion is an honest command/state-only
proof, not an implied application check.

Each selected service expands its declared `services.<name>.depends_on` closure. Ota starts that
closure in stable topological order and tears down only transaction-leased services in reverse
order. A cycle, an unselected lifecycle-ineligible dependency, or an ambiguous dependency order is
a contract error.

It is valid only when every selected or transitively required service has canonical manager-owned
lifecycle truth and one explicit initial-state capability:

- `services.<name>.manager.kind: compose` with its derived lifecycle controls and typed service
  state observer;
- `services.<name>.manager.kind: host` with structured `manager.start` and `manager.stop` plus a
  typed state observer; or
- an Ota-owned isolated-boundary absence attestation that proves no prior service can occupy the
  selected execution boundary; or
- a later typed manager adapter that can expose the same start, stop, and state-observation
  semantics.

Structured `start` and `stop` alone are not a lifecycle-proof capability. They can execute only
after Ota has acquired a cleanup lease from one of the explicit initial-state capabilities above.
This deliberately leaves a native Caddy start/stop smoke unsupported until its execution boundary
or manager can establish absence without guessing. An isolated container boundary may qualify for
the narrower command/state proof when its identity and absence attestation are runner-authored.

Lifecycle state capability is runner-derived, never a maintainer-authored boolean. The first
capabilities are the existing Compose service-state control plane, existing systemd active/inactive
control plane, and an Ota-owned isolated execution boundary whose absence attestation is bound to
the current transaction. Generic host commands remain ineligible until a later typed adapter can
provide equivalent positive active and inactive observations.

The lifecycle declaration must not carry commands, paths, ports, process IDs, readiness URLs,
expected exit codes, or caller-authored result labels. Those belong respectively to existing service
manager, endpoint/readiness, and runner-owned output truth. Unknown, duplicate, or managerless
service references are contract errors.

Existing `services.<name>.readiness` remains the canonical start-state assertion. V11.18 adds a
typed teardown observation only where the manager can positively observe inactive state:

```yaml
services:
  caddy:
    lifecycle:
      teardown_assertion: manager_inactive
```

`manager_inactive` requires an authoritative typed manager state observation after Ota-owned
teardown. A failed HTTP or TCP readiness probe may support diagnosis, but it can never independently
prove stopped state: it might be DNS, routing, credentials, or probe failure. Managers without a
positive inactive-state observer may emit command outcomes only and must carry an explicit
`service_stopped_state_not_proved` boundary.

## Execution And Evidence

The first command is:

```text
ota proof lifecycle --workflow <name> [--service <name>] [--json] [--archive]
```

Omitting `--service` selects the workflow's declared lifecycle-proof services. An explicit service
must be in that workflow declaration. It uses the same contract snapshot, scope identity, agent
admission, execution mode, target OS, receipt archive, and interruption rules as the selected
workflow; it does not introduce a second service runner.

The runner emits one canonical `lifecycle_proof` transaction with stable service records:

```json
{
  "service": "caddy",
  "transaction_id": "...",
  "preexisting_state": "inactive_observed",
  "cleanup_lease": "acquired",
  "ownership": "started_this_transaction",
  "start": { "state": "command_succeeded", "evidence_class": "attested" },
  "readiness": { "state": "not_declared" },
  "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
  "teardown_assertion": { "state": "not_declared" }
}
```

Allowed runner-derived state categories are deliberately narrow:

- `preexisting_state`: `inactive_observed`, `active_observed`, or `unknown`;
- `cleanup_lease`: `not_acquired`, `acquired`, `released`, or `cleanup_failed`;
- `ownership`: `started_this_transaction`, `reused_preexisting`, or `unknown`;
- transition states: `not_run`, `command_succeeded`, `command_failed`, `state_observed`,
  `state_not_observed`, or `interrupted`.

The emitted lifecycle record binds every observation and command outcome to the lifecycle
transaction ID, selected service identity, manager identity, contract snapshot, execution scope,
and ordered runner sequence. A stale status observation, previous PID, or matching service name
from an earlier run cannot satisfy the current transaction.

Ota acquires the cleanup lease before it invokes `start`, after a current-transaction inactive-state
observation succeeds. The runner finalizer attempts teardown after every start attempt while that
lease remains acquired, even if start returns an error or the process is interrupted. It may release
the lease without teardown only when the same manager proves that no transition occurred. If
teardown cannot be attempted, fails, or cannot release the lease through a positive inactive-state
observation, the proof fails with a typed cleanup failure and the archive remains available. It
must not report lifecycle success merely because start or a later assertion succeeded.

## Honest Proof Breadth

Lifecycle proof is not application-output proof.

- A successful structured start and stop command proves only the declared commands completed in
  order for this transaction.
- Declared readiness can prove the selected service reached the declared reachable state after
  start.
- Only a verified positive manager-inactive observation can prove the selected service was stopped
  after Ota-owned teardown. Inverse readiness is supporting evidence only.
- A finite assertion task can prove only its declared task obligation; it does not prove broader
  application output unless a future output-proof carrier says so.

V11.11 `not_proved[]` is mandatory for every unsupported depth claim. Typical entries are:

- `service_started_state_not_proved` when no manager/readiness state observation exists;
- `service_stopped_state_not_proved` when teardown completion cannot be independently observed;
- `application_output_not_proved` when lifecycle transitions did not exercise an application
  output obligation;
- `broader_repo_completion_not_proved` for every bounded lifecycle proof.

`proof_verdict` is derived at one runner-owned decision site. It cannot emit `passed` with any
`not_proved[]` entry. Caddy can become a qualified lifecycle proof only through an eligible
isolated boundary or typed state capability; its native generic-host start/stop smoke remains
outside lifecycle-proof admission until then. Neither path may be described as full runtime or
functional proof merely because both commands returned zero.

## Safety And Existing State

- Lifecycle proof is never implicitly agent-safe. It uses the selected workflow's existing agent
  closure and effect/policy admission.
- A manager-reported active preexisting service is preserved. Ota records `reused_preexisting` and
  refuses a start/stop lifecycle proof unless a later explicit policy-owned shared-service mode is
  introduced.
- An unknown initial state is not treated as inactive. Ota refuses destructive lifecycle ownership
  and records the unresolved boundary. Generic host `start`/`stop` commands without a typed state
  observer or isolated-boundary absence attestation are therefore ineligible, not weakly admitted.
- Cleanup ownership is transaction-local. A later CLI invocation, another workflow instance, or an
  unrelated process cannot inherit a right to stop the service.
- Native host lifecycle proof defaults to qualified evidence. V11.16 boundary evidence may
  strengthen initial-state provenance, but no caller environment signal can promote it.

## CI Projection

V11.15's provider-neutral projection gains lifecycle-proof requirements only after the local
command and archive are pressure-proven. The generated GitHub lane invokes `ota proof lifecycle`
as the one authoritative execution path for a lifecycle-proof workflow; it must not render a
separate `ota up`, copied `start`/`stop` shell block, or post-hoc cleanup step.

Provider triggers, credentials, deployment policy, and non-Ota release behavior remain
provider-owned. A generated lifecycle proof is a governed contributor verification lane, not a
claim that Ota owns the upstream release workflow.

## Implementation Order

1. Define the service-reference-only contract shape, explicit post-readiness assertion task,
   selected service dependency closure, semantic validator, public schema, and canonical
   lifecycle-capability check.
2. Implement pre-state observation and cleanup-lease acquisition before start, deterministic
   dependency-order startup, reverse-order rollback/teardown, and exact-once finalization over
   existing Compose and structured host manager controls.
3. Reuse existing service readiness as start-state evidence; add positive typed manager-inactive
   observation for stopped-state proof. Keep inverse readiness diagnostic-only.
4. Emit lifecycle-proof JSON, receipt/archive evidence, human output, V11.11-qualified verdict,
   and typed cleanup/interruption outcomes from one decision owner.
5. Add regression fixtures for start success plus readiness failure, assertion failure plus forced
   teardown, start error after lease acquisition, teardown failure, pre-existing service
   preservation, stale observation rejection, unknown state refusal, command-only qualified proof,
   and multi-service dependency rollback.
6. Extend the provider-neutral projection and GitHub renderer only after local lifecycle proof has
   passed its acceptance bar.
7. Pressure-test Caddy's start/stop intent in an eligible Ota-owned isolated boundary, then a
   Compose-managed service with both ready and stopped observations. Keep any native generic-host
   Caddy path explicitly ungoverned until it gains a truthful state capability. Publish the exact
   bounded claim each run proves.

## Non-Goals

- No generic workflow scripting language or arbitrary shell-step parser.
- No claim that a daemon's start command proves functional traffic handling.
- No stopping, killing, or cleaning a service Ota did not start in the current transaction.
- No reuse of status observations, PIDs, or readiness results from a prior run.
- No platform-specific Caddy adapter as the first implementation; the service-manager contract is
  the reusable boundary.

## Acceptance Bar

V11.18 is complete when:

- lifecycle-proof contracts reference only canonical service-manager truth and reject command
  duplication, managerless services, missing state capability, unsupported state assertions,
  duplicate service entries, and ambiguous dependency closure;
- `workflow.run.task` is executed before lifecycle ownership and an optional finite assertion task
  is executed only after selected services are ready;
- every leased service has a transaction-bound ordered record and a finalizer outcome, including a
  failed or interrupted start attempt;
- a failed readiness or assertion still runs Ota-owned reverse-order teardown exactly once;
- Ota never starts or stops a manager-observed pre-existing or unknown-state service;
- command-only start/stop proof remains visibly qualified with the correct `not_proved[]` entries;
- ready observations and stopped observations can be claimed only through declared,
  manager-supported state evidence bound to the current lifecycle transaction; inverse readiness
  alone never proves stopped state;
- JSON schema, receipt/archive schema, human output, and regression fixtures enforce the same
  lifecycle and proof-boundary invariants;
- Caddy proves its start/stop intent through an eligible typed surface without copied shell glue,
  while a second manager family proves readiness and teardown state without copied shell glue;
- first-party examples, skill guidance, command/spec references, changelog, and site documentation
  explain lifecycle proof as a bounded state-transition claim rather than full application proof.
