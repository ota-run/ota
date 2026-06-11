# v9.1 Pressure-Test Gap Ledger

This ledger tracks platform gaps surfaced by pressure-test repositories and the current maturity
state in Ota.

## Open Maturity Work

The current open maturity program is tracked in
[trust-and-governance-hardening.md](trust-and-governance-hardening.md).

| Gap | Status | Planned Workstream |
| --- | --- | --- |
| Proof phase truth is still too loose | Open | Proof Trust |
| Env compatibility policy is still too shell-shaped | Open | Env Governance |
| Replaceable shell glue still lacks strong governance warnings | Open | Contract Governance |
| Env overlay transformation is not yet a first-class governed surface | Open | Env Governance |
| Adapter/runtime input ownership is still partly shell-carried | Open | Adapter Ownership Cleanup |
| Proof root-cause diagnostics are still too narrow | Open | Proof Trust |
| Governance advisories are not yet broadly productized | Open | Contract Governance |

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
| `.NET` toolchain parity across init/detect/validator                                             | Closed | Shipped `toolchains.dotnet` (`provider: dotnet`), dotnet pack now toolchain-owned, detect/write converges `.NET` to toolchain ownership, and duplicate split ownership is validated out |
| Effects governance parity for `network` and `external_state`                                     | Closed | `ota run` and `ota up` now enforce deny decisions pre-execution on selected task/workflow paths (including setup-present `ota up` lanes) and emit effect-governance policy decision lines in receipts via the `policy` surface |
| Policy-governed run-path fulfillment for Go/Ruby/.NET toolchains                                | Closed | `toolchains.go`, `toolchains.ruby`, and `toolchains.dotnet` now accept `fulfillment: run` and validate as policy-governed fulfillment lanes instead of hard check-only rejects |
| Policy-governed run-path fulfillment for Corepack/SDKMAN toolchains                              | Closed | `toolchains.node` (`provider: corepack`) and `toolchains.java` (`provider: sdkman`) now accept `fulfillment: run` and validate as policy-governed selected-path fulfillment lanes |

## Remaining Product-Maturity Work

The open maturity items are listed in the `Open Maturity Work` section above.

## Upstream Adoption Outcomes

| Repo                     | PR                                                           | Outcome                | Notes                                                                                       |
| ------------------------ | ------------------------------------------------------------ | ---------------------- | ------------------------------------------------------------------------------------------- |
| `hoppscotch/hoppscotch` | [#6382](https://github.com/hoppscotch/hoppscotch/pull/6382) | Closed as not planned | Maintainers confirmed current CI + `docker-compose` profiles already cover their needs now |

## Rule For New Pressure Gaps

When a new gap is found:

1. Reproduce with one concrete repo contract and one command invocation.
2. Classify as one of:
   - trust bug
   - contract model gap
   - docs/authoring guidance gap
3. Add acceptance evidence before marking closed.
