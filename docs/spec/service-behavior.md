# ota Service Behavior

This page describes the canonical shipped service behavior across `ota doctor`, `ota up`, and `ota services`.

`ota services` is the list command for declared services. It reports the service contract surface without pretending services are direct task entrypoints.
Canonical service authoring uses `manager`, `endpoints`, and `readiness`, with `producer` for cross-repo ownership.

## Contract surface

Current service fields:

- `required`
- `producer`
- `manager`
- `endpoints`
- `depends_on`
- `readiness`

Authoring guidance:

- use `manager` for locally managed services
- use `producer` when another workspace repo owns the service
- use `endpoints` to declare how each execution context reaches the service
- use `readiness` to declare how Ota proves the service is actually ready

## `ota doctor`

Current behavior:

- validates declared service-manager control-plane requirements
- validates endpoint projection for context-scoped readiness
- runs declared service readiness for required services
- reports failed required service readiness as blocking errors
- reports failed optional service readiness as warnings
- reports timed out required service readiness as blocking errors
- reports timed out optional service readiness as warnings
- warns when a required service has no declared readiness, because readiness cannot be verified
- for `manager.kind: compose`, can verify container health directly with `readiness.kind: compose_health`

## `ota services`

Current behavior:

- lists declared services from the validated contract
- when the root contract declares `workspace.type: monorepo`, lists root services and grouped member summaries
- shows the service fields that matter for ownership, reachability, and readiness
- does not run services directly like `ota run` runs tasks

## `ota up`

Current behavior:

1. validate the contract
2. run blocking preconditions
3. if preconditions fail and `setup` exists, run `setup` early and re-check readiness
4. start required services, and required-service dependencies, through declared managers in dependency order
5. verify required service readiness as readiness gates
6. stop in the `services` phase if required services still are not ready
7. run `setup` if present
8. rerun readiness diagnosis

Important boundaries:

- ota preserves child exit codes for service start failures
- for workflow run tasks with `runtime.kind: service`, default `ota up` behavior now runs
  detached readiness proof and tears down proof-owned run execution after readiness confirms
- `ota up --detach` is the explicit keep-running path for that proved workflow run task
- ota still does not perform broad automatic teardown of declared repo services after startup
- ota does not provide deep service orchestration
- ota does not infer service dependency ordering

## Recommendation

Treat service behavior as explicit contract infrastructure.

Model local services with explicit managers, per-context endpoints, and structured readiness.
If a repo needs startup ordering or other orchestration semantics, add them to the contract/spec
first rather than relying on implicit command behavior.
