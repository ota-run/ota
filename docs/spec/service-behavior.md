# Ota Service Behavior

This page describes the current shipped service behavior across `ota doctor`, `ota up`, and `ota detect`.

`ota services` is the list command for declared services. It reports the service contract surface without pretending services are direct task entrypoints.

## Contract surface

Current service fields:

- `required`
- `provider`
- `start`
- `stop`
- `healthcheck`
- `depends_on`
- `timeout`

At least one actionable field is required:

- `provider`
- `start`
- `stop`
- `healthcheck`

## `ota doctor`

Current behavior:

- runs declared service `healthcheck` commands
- for `provider: docker-compose`, runs the healthcheck inside the service container via `docker compose exec -T <service> sh -lc <healthcheck>`
- reports failed required service healthchecks as blocking errors
- reports failed optional service healthchecks as warnings
- reports timed out required service healthchecks as blocking errors
- reports timed out optional service healthchecks as warnings
- warns when a required service has no `healthcheck`, because readiness cannot be verified

## `ota services`

Current behavior:

- lists declared services from the validated contract
- when the root contract declares `workspace.type: monorepo`, lists root services and grouped member summaries
- shows the service fields that matter for readiness and startup management
- does not run services directly like `ota run` runs tasks

## `ota up`

Current behavior:

1. validate the contract
2. run blocking preconditions
3. start required services, and required-service dependencies, in declared dependency order
4. verify required service healthchecks as readiness gates
5. stop in the `services` phase if required services still are not ready
6. run `setup` if present
7. re-run readiness diagnosis

Important boundaries:

- Ota preserves child exit codes for service start failures
- Ota does not perform automatic teardown
- Ota does not provide deep service orchestration
- Ota does not infer service dependency ordering

## `ota detect`

Current Docker Compose inference:

- `provider` at high confidence
- `start` / `stop` at medium confidence
- declared `healthcheck.test` at medium confidence

Current supported Compose filenames:

- `docker-compose.yml`
- `docker-compose.yaml`
- `compose.yml`
- `compose.yaml`

Important boundaries:

- Ota does not invent healthchecks
- Ota does not infer service dependency ordering
- write mode still writes only high-confidence fields

## Recommendation

Treat service behavior as explicit contract infrastructure.

If a repo needs startup ordering or other orchestration semantics, add them to the contract/spec first rather than relying on implicit command behavior.
