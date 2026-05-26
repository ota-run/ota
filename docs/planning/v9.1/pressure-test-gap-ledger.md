span

# v9.1 Pressure-Test Gap Ledger

This ledger tracks platform gaps surfaced by pressure-test repositories and the current maturity
state in Ota.

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
| JSON schema validation pipeline maturity for command outputs                                       | Closed | `ota json validate` and CI schema guard lane shipped                              |

## Remaining Product-Maturity Work


| Gap                                                                                                  | Status  | Next Maturity Step                                                                                    |
| ---------------------------------------------------------------------------------------------------- | ------- | ----------------------------------------------------------------------------------------------------- |
| Contract-native conditional execution beyond`checks.kind: changed_files` (for example `nx affected`) | Planned | Add a first-class task/workflow condition surface instead of command-shaped shell checks              |
| Higher-level bootstrap orchestration (multi-file compose/secret generation plans)                    | Planned | Layer on top of existing first-class task`action.kind` primitives without reintroducing shell glue    |
| Agent authority authoring ergonomics                                                                 | Planned | Keep posture/exception behavior stable, continue tightening authoring UX/examples to reduce ambiguity |

## Rule For New Pressure Gaps

When a new gap is found:

1. Reproduce with one concrete repo contract and one command invocation.
2. Classify as one of:
   - trust bug
   - contract model gap
   - docs/authoring guidance gap
3. Add acceptance evidence before marking closed.
