# v9.1 Pressure-Test Gap Ledger

This ledger tracks platform gaps surfaced by pressure-test repositories and the current maturity
state in Ota.

## Open Maturity Work

| Gap                                                                                              | Status | Acceptance Evidence |
| ------------------------------------------------------------------------------------------------ | ------ | ------------------- |
| Toolchain fulfillment depth for check-only providers (`corepack`, `go`, `ruby`, `sdkman`)      | Open   | `provider:*` toolchains support a governed `fulfillment: run` lane (or an explicit policy-backed fulfillment mode) with deterministic policy enforcement, capability/min-version gating, and command/docs parity |
| Effects governance parity for `network` and `external_state`                                    | Open   | `ota run` / `ota up` enforce policy decisions for selected-path `effects.network` and `effects.external_state` pre-execution (not advisory-only), with deterministic allow/deny semantics and machine-readable decision receipts |
| Plan/docs status hygiene for shipped planning slices                                             | Open   | Planning docs carry unambiguous status semantics (active/completed/archived) so shipped features are not presented as still-planned work; status guidance is documented and applied consistently for active slices |

## Closed In Current Branch


| Gap                                                                                                | Status | Acceptance Evidence                                                               |
| -------------------------------------------------------------------------------------------------- | ------ | --------------------------------------------------------------------------------- |
| Agent-safe transitive protected-path/write boundary enforcement                                    | Closed | Safe-task closure validation blocks protected/out-of-bound reachable writes       |
| `ota run --dry-run` context clarity (`task` vs selected execution context)                         | Closed | Preview text and JSON now include requested vs selected context surfaces          |
| Optional global tools leaking into unrelated selected task previews                                | Closed | Selected-path requirement rendering now scopes by selected backend/context        |
| Task-level tool requirements requiring duplicate top-level declarations                            | Closed | Task path`requirements.tools` now validate without duplicate global entries       |
| First-class disjunctive selected-path requirements                                                 | Closed | `requirements.any_of` supported and wired into selected requirement resolution    |
| First-class deterministic bootstrap actions (`ensure_file`, `ensure_env_file`, `ensure_directory`) | Closed | Contract schema + runner + docs + capability gating in place                      |
| First-class compose health readiness for internal services                                         | Closed | `services.<name>.readiness.kind: compose_health` supported and documented         |
| Agent exception semantics for sensitive writable boundaries                                        | Closed | `agent.exceptions.sensitive_writes` validation/advisories documented and enforced |
| Agent authority authoring ergonomics                                                             | Closed | contract/site guidance now pin posture/exception usage, safe-task defaults, and explicit `ota tasks --safe/--unsafe --via` authoring-discovery lanes |
| JSON schema validation pipeline maturity for command outputs                                       | Closed | `ota json validate` and CI schema guard lane shipped                              |
| Contract-native conditional execution beyond `checks.kind: changed_files` (for example `nx affected`) | Closed | `tasks.<name>.when.checks` gates execution with first-class precondition/file/changed_files checks |
| Higher-level bootstrap orchestration (multi-file compose/secret generation plans)                | Closed | `action.kind: ensure_bundle` composes ordered deterministic setup steps without shell glue |
| Task launch sources as first-class task model (`tasks.<name>.launch`)                            | Closed | Shipped command/container launch surfaces wired through schema, validator, runner, tasks/workflows/topology output |

## Remaining Product-Maturity Work

The open maturity items are listed in the `Open Maturity Work` section above.

## Rule For New Pressure Gaps

When a new gap is found:

1. Reproduce with one concrete repo contract and one command invocation.
2. Classify as one of:
   - trust bug
   - contract model gap
   - docs/authoring guidance gap
3. Add acceptance evidence before marking closed.
