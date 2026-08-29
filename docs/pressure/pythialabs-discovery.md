# PythiaLabs Design-Partner Discovery

Status: bounded pre-release fork pressure. This is not V12 completion evidence, an upstream
endorsement, or an upstream pull request. The fork remains a review artifact until Ota v1.6.27 is
released and Aleksei reviews a draft PR in writing.

## Subject

-   Repository: `safal207/pythiaLabs`
-   Default branch: `main`
-   Reviewed revision: `17df87775c0d5407c07e86f278455d912ed51305`
-   Review date: 2026-08-29
-   Fork: `bobaikato/pythiaLabs`, branch `bobai/ota-pre-release-pressure`
-   Native-pressure revision: `bf51a0b4c93128522226cf977895f54c6603d33d`
-   Container-pressure revision: `916f9f127c9d76580e8f5efa8f4f0af497019fff`, hosted in [run 33258442106](https://github.com/bobaikato/pythiaLabs/actions/runs/33258442106)
-   Ota probe: source-built Ota v1.6.27 at Core `cd99c9abd2c0225b454371e897eca2486319db26`
-   Upstream posture: Aleksei requested asynchronous written review through a draft PR from a fork, with a proposed `ota.yaml`, a separate non-blocking workflow, exercised assumptions, and explicit unproved boundaries.

This revision combines an Elixir core, a Rust NIF with an Elixir fallback, a Rust port worker, a Node MCP bridge, a generated Node site, Python conformance suites, credentialed provider pilots, and pinned multi-repository evidence workflows. The repository is a high-signal design-partner candidate because its important boundary is not language count. It is the difference between declared decision logic, selected execution posture, effective runtime use, and external authority.

## Observed Verification

The review used an isolated temporary clone. It did not modify or execute upstream GitHub state.

Locally observed:

-   `node integrations/mcp/smoke.mjs` passed its framing and routing assertions;
-   `cargo test` in `workers/solver_port` passed six tests;
-   Action Envelope conformance passed 64 tests in an isolated Python environment;
-   VCE conformance passed 17 tests in an isolated Python environment;
-   authorization fixtures and adversarial checks passed 12 cases in total;
-   CAEP tests passed 52 cases with their optional cryptographic dependency installed;
-   `npm ci` and `npm run build` under `site/` passed;
-   `npm run format:check` under `site/` failed on 13 existing files;
-   the Elixir lane was not run locally because this host did not have `mix`; and
-   current `main` hosted CI, security, ACI, and VCE runs are green at the reviewed revision.

The local Rust run generated an untracked `workers/solver_port/Cargo.lock`, confirming that the reviewed revision does not retain that resolution identity. Generated build and Python cache files were confined to the temporary clone.

## Pre-release Fork Pressure

The fork carries a reviewable `ota.yaml` and two separate visible workflows. They are not required
upstream CI checks and do not execute credentialed CAEP, GitHub merge, communications, or the
Liminal multi-repository lifecycle.

-   [Declaration and discovery pressure run 33256380938](https://github.com/bobaikato/pythiaLabs/actions/runs/33256380938) passed at the fork revision above. It validates the contract, exports declared task posture, runs Doctor, and emits a review-only detection candidate.
-   [Non-credentialed execution matrix 33256380935](https://github.com/bobaikato/pythiaLabs/actions/runs/33256380935) ran the reviewed native lanes with the exact Core revision above. Ten of eleven lanes passed: Elixir format/test, MCP smoke, Rust worker build/test, site build, and Action Envelope, VCE, ACI, and local CAEP conformance.
-   `verify:site-format` failed because Pythia's existing `prettier --check .` reports 13 unformatted site files. This is a repository-quality finding, not an Ota failure. The fork does not rewrite upstream source or downgrade the verification lane without maintainer review.
-   An earlier fork execution attempt exposed an Ota parser defect: the generic version parser read `Erlang/OTP 26` as the Elixir version. Core `cd99c9ab` now extracts the explicit `Elixir 1.16.3` line, with a regression. The contract also replaced unsupported `~> 1.15` syntax with the supported equivalent `>=1.15,<2`.

### Container Decision

Pythia's Dockerfile remains outside Ota authority: it installs Rust through a mutable network
bootstrap and is not a reviewed image, platform, mount, or hydration carrier. That does not remove
the value of Ota-owned container execution. The fork now declares a bounded container pressure slice
using an explicit, digest-pinned Linux Node context for the portable MCP and site closures, with
each task selecting it through `--mode container` and retaining separate artifacts. The
[hosted container matrix](https://github.com/bobaikato/pythiaLabs/actions/runs/33258442106) passed
that bounded Linux Node slice and retains Ota's selected-context summary plus child output.

This proves only Ota's declared context selection, closure execution, working-directory handling,
and the selected site hydration/build path. It does not validate the Dockerfile, establish a
multi-language container image, prove image provenance beyond the pinned digest, establish
macOS/Windows parity, or upgrade any lane to agent-safe authority.

## Ota Probe

Current source-built Ota detected Elixir and `mix`, but did not recover the repository's material lane structure:

-   it inferred the site's `npm run build` at repository root, losing the workflow-owned `working-directory: site`; the inferred command fails from that root;
-   it reduced a multiline MCP verification step to its first command;
-   it omitted the Rust NIF posture, Rust port-worker lane, Python conformance families, credentialed provider pilot, and pinned multi-repository lifecycle;
-   it produced a partial agent boundary that did not represent several material roots; and
-   its starter preview did not author a minimum Ota version floor.

The durable candidate remained honest about unresolved truth. The mis-rooted build and truncated check remained `unknown`, and `apply-candidate --require-complete` refused. The defect is therefore in discovery fidelity and lane recovery, not in candidate admission silently promoting unknown work.

## Classified Findings

### Repository maturity

1.  Native acceleration can silently disappear. The custom Rustler compiler converts NIF build errors and exceptions into successful fallback compilation, and the fallback message is hidden unless `PYTHIA_VERBOSE_FALLBACK` is enabled. Main CI does not independently prove that the NIF compiled and loaded. A fallback-compatible lane and a NIF-required lane should be distinct.
2.  Port-worker tests skip when Cargo is unavailable. Contributor `mix test` can therefore be green without exercising the advertised worker. Hosted CI currently installs Rust and separately builds/tests the worker, but contributor truth is weaker than CI truth.
3.  Rust resolution and toolchain inputs are mutable. Neither Rust project retains a committed `Cargo.lock`; CI uses `rust-toolchain@stable`; the Dockerfile installs current Rust through a network script. These are reproducibility gaps, not Ota execution failures.
4.  Site formatting is an existing red lane not present in hosted site CI or contributor guidance. The README also recommends `npm install` while hosted CI uses `npm ci`.
5.  Trust-sensitive workflow dependencies are inconsistently immutable. Some workflows pin actions to commits, while others use floating action tags or install provider dependencies dynamically.
6.  The default branch is not protected. That is repository administration posture and should not be represented as an execution-contract guarantee.

### Ota maturity gaps

1.  CI detection must bind job and step working directories into command closure identity.
2.  Multiline workflow steps must remain complete ordered closures or become explicitly unresolved; selecting only the first line is not an acceptable runnable inference.
3.  Multi-project lane recovery must preserve separate Elixir, Rust NIF, Rust worker, MCP, Python, site, credentialed, and multi-repository postures instead of collapsing them into generic `build`, `check`, and `test` names.
4.  Agent-boundary inference needs complete, evidence-bound path classification or an explicit incomplete posture. Partial writable/protected output must not look repository-global.
5.  Ota detects Elixir and Mix commands, but does not own a typed Mix/Hex hydration source. A first-class BEAM hydration and lockfile posture is a concrete widening opportunity if repeated pressure shows demand.
6.  Newly authored starter contracts should carry the Skill's minimum-version quality bar. Core may remain backward-compatible with older contracts, but omission from current authoring output is an onboarding maturity gap.

### V12 and later-version signals

PythiaLabs is not a current V12 database-schema-mutation pressure repository. It must not be counted toward V12's independent real-repository acceptance bar.

It does expose adjacent demand:

-   guarded GitHub merge is an external mutation requiring exact target, authority, replay, and provider evidence;
-   the credentialed CAEP benchmark performs multi-provider model calls with cost and concurrency dimensions and is a useful V12.1 secret-delivery pressure input;
-   provider calls, GitHub merge, communications, and external-data actions are possible future effect families, but each needs its own selectors, policy, schemas, evidence, and independent pressure before activation; and
-   Pythia decision artifacts could become inputs to a future provider-neutral adapter profile, but they must never become Ota execution grants merely because the verdict is `ALLOW`.

V12.3 through V12.5 remain demand-gated. This discovery does not activate a provider-attested or platform authority carrier.

### Governance gaps

1.  Pythia's decision authority and Ota's execution authority must stay separate. Any future bridge must bind action, resource, scope, subject, freshness, policy source, decision identity, and one-use consumption before the result can influence admission.
2.  Repository-controlled policy, an in-process replay store, and injected test executors do not establish independently administered authority or production enforcement.
3.  MCP configuration and smoke output do not prove that Cursor or another harness loaded the server, used the real Elixir evaluator, or applied the returned decision. A future adapter must distinguish projected, runtime-acknowledged, effective, and unknown posture.
4.  The Gmail-to-GitHub synchronization protocol describes consequential communications and code changes in prose. It is not an identity-bound execution or receipt surface today.

### Widening opportunities

1.  Use Pythia as a regression for working-directory-aware, multiline, multi-project detection.
2.  Pressure a first-class BEAM/Mix hydration source only after determining whether this need repeats beyond one repository.
3.  Treat the pinned three-repository Liminal workflow as a workspace-contract pressure case for exact revisions, mapped tasks, lifecycle cleanup, and evidence lineage. Do not flatten it into one repo-local shell task.
4.  Use the MCP boundary as future adapter-conformance pressure for required posture versus witnessed effective runtime posture. Do not substitute product-specific config scanning for attestation.
5.  Consider a versioned Pythia decision-evidence adapter only after a real runtime integration can produce immutable, independently reconcilable evidence.

## Demand-Gated Capability Ledger

These entries record product signals only. They do not activate a plan, broaden V12, or promise implementation before the relevant acceptance gates are satisfied.

Capability signal

Pythia evidence

Existing owner

Current decision

Activation threshold

Effective-runtime observation

MCP configuration and smoke prove projection/routing but not that a real harness loaded or enforced the evaluator

Inactive adapter/profile conformance plan

Needed as a provider-neutral capability; no implementation yet

One real runtime integration exposing immutable, pressure-testable effective-state evidence

Governed secret delivery

Credentialed CAEP pilot supplies model-provider credentials to consequential external calls

Inactive V12.1 plan

Already planned; Pythia is a pressure candidate, not activation evidence

V12 closed, V12.1 activated, a disposable provider/resource fixture, and retained non-secret evidence

GitHub merge effect family

Guarded merge seam binds authorization and replay checks before an injected executor

Future effect-family widening under the V12 effect model

Likely useful, but not yet approved as a shipped family

Repeated repository demand, canonical target/resource selectors, provider re-evaluation, and independent mutation/refusal pressure

Provider-call effect family

CAEP benchmark performs model-provider calls with model, cost, and concurrency dimensions

Future effect-family widening under the V12 effect model

Relevant signal only

Repeated demand plus canonical provider/action/resource/cost bounds and an independently pressure-testable adapter

Communication effect family

Gmail/GitHub protocol can produce replies, reactions, and code changes

Future effect-family widening under the V12 effect model

Relevant but prose-governed and currently unproved

Executable carrier, canonical recipient/channel/action scope, authority semantics, receipts, and more than one pressure case

External-data mutation effect family

Repository protocols describe consequential changes to external systems

Future effect-family widening under the V12 effect model

Relevant signal only

Stable resource/action schema, provider evidence, policy selectors, and independent pressure

Pythia decision-evidence adapter

Pythia emits deterministic decision and causal evidence that could inform Ota admission

Adapter/profile conformance if a provider-neutral profile emerges

Do not plan a Pythia-specific adapter yet; `ALLOW` remains evidence only

Real integration with immutable identity, freshness, exact action/resource/scope binding, authority separation, and one-use consumption

Pinned multi-repository lifecycle

Liminal workflow checks out exact revisions, builds multiple repositories, executes a lifecycle, and retains evidence

Existing workspace-contract surface first

Pressure current `ota.workspace.yaml` before proposing expansion

A concrete current-surface failure that cannot be represented without losing revision, lifecycle, cleanup, or evidence lineage truth

First-class Mix/Hex hydration

Elixir setup requires dependency hydration while Ota currently owns no typed Mix/Hex source

Current hydration/detection maturity backlog

Test current explicit setup modeling first; widen only if it remains materially weaker

Honest Pythia contract cannot model deterministic setup, or the same gap repeats in another BEAM repository

Current-surface detector defects are not demand-gated by this table. Working-directory loss, multiline truncation, lane collapse, incomplete agent-boundary posture, and starter minimum-version omission were reproduced with source-built v1.6.27 and require regression-backed repair before the release.

## Current-Surface Repair Status

The pending v1.6.27 detector repair addresses the reproduced Ota-owned defects without adding a Pythia-specific adapter:

-   canonical repository-relative GitHub Actions job or step `working-directory` becomes structured task `command.cwd`; dynamic, noncanonical, and escaping values do not become root-task truth;
-   named multiline verification steps retain the complete ordered `run` body and stay unresolved, rather than promoting one selected line;
-   CI job identity continues to separate distinct verifier lanes, while CI evidence remains non-authoritative and never becomes inferred agent-safe authority;
-   detected boundaries with heuristic safe tasks report `Partially inferred` unless the path boundary is explicit; and
-   newly authored starter contracts declare `metadata.ota.minimum_version` for the running Ota release.

These repairs need independent review and release validation. They do not prove BEAM hydration, real MCP runtime loading, provider calls, external effects, or Pythia repository behavior.

### Local Branch Replay

On the source-built `1.6.27-implementation` branch, a fresh clone pinned to
`17df87775c0d5407c07e86f278455d912ed51305` confirmed the repaired detector/candidate boundary:

-   the site build remains `npm run build` with `cwd: site` in both detected output and the
    source-bound candidate's identity-bound execution closure;
-   the MCP syntax step retains its complete ordered multiline body and remains `unknown` rather
    than becoming a selected one-line task; and
-   the candidate contains 11 changes, with 5 `applicable` and 6 `unknown`; `apply-candidate
    --require-complete` refuses with `candidate_incomplete` and writes no contract.

The remaining unknowns include inline shell `cd` worker lanes, heredoc/credentialed workflows,
and other non-simple command bodies. They are conservative review inputs, not lost or agent-safe
task truth. This is local unreleased-branch evidence only: it does not replace release validation,
hosted pressure, task execution, or the eventual draft PR.

### Local Branch Evidence Matrix

This matrix records only the branch-local detector and candidate observations above. It is not an
immutable hosted-pressure matrix and must not be used as release, upstream, runtime, or execution
evidence.

| Surface | Exercise and retained observation | Result | Boundary |
| --- | --- | --- | --- |
| Source identity | Fresh clone at `17df87775c0d5407c07e86f278455d912ed51305` with source-built Ota `1.6.27-implementation` | Exact local inputs identified | Neither a released binary nor a retained hosted artifact |
| GitHub Actions working directory | `ota detect --json` recovered the Lighthouse build as `npm run build` with `cwd: site` | Detected command matches the workflow-owned directory | Detects declaration; does not run the build |
| Candidate closure binding | `ota detect --candidate-out ... --json` retained `cwd: site` in the build change and identity-bound execution closure | Source-bound candidate preserves detected CWD truth | Candidate remains review-only and unexecuted |
| Multiline MCP verification | The MCP syntax step retained its complete ordered body and was classified `unknown` | No selected one-line task was fabricated | Does not establish a runnable MCP lane or runtime loading |
| Completeness admission | Candidate contained 11 changes: 5 `applicable`, 6 `unknown`; `apply-candidate --require-complete` returned `candidate_incomplete` | Incomplete discovery cannot write `ota.yaml` | No contract application or repository task execution occurred |
| Complex and credentialed workflow bodies | Inline `cd`, heredoc, credentialed, and other non-simple bodies remained `unknown` | Conservative refusal of unsupported inference | These lanes need maintainer review or future Ota capability work |
| Runtime and external authority | No Mix/NIF lane, MCP host session, provider call, credentialed pilot, merge, communication, or Liminal lifecycle was invoked | Not exercised | No claim of runtime enforcement, provider authority, external mutation, or repository-wide governance |

The eventual post-release fork PR needs a separate immutable matrix with the exact released Ota
version, fork revision, workflow URL, retained artifacts, exercised commands, and the same
unproved-boundary inventory. It must not overwrite this discovery record.

## Eventual Draft PR Scope

Do not publish the draft PR until Ota v1.6.27 is released. Before authoring it, review the lane map with Aleksei asynchronously so the contract does not encode false equivalence.

The bounded first PR should contain:

-   one reviewable `ota.yaml` with explicit contributor lanes rather than a global `test` claim;
-   a separate non-blocking Ota workflow with retained machine-readable artifacts;
-   explicit Elixir format/test, Rust NIF-required, Rust port-worker, MCP smoke, selected Python conformance, and site build/format lanes where their prerequisites are deterministic;
-   non-agent-safe setup/hydration tasks separated from verification tasks;
-   no credentialed provider calls, live merge execution, Pages deployment, or external repository mutation; and
-   an uncovered-material-behavior inventory in the PR description and pressure artifact.

Do not route the credentialed CAEP pilot, guarded merge executor, Gmail/GitHub protocol, or Liminal multi-repository workflow through ordinary agent-safe tasks. They require separate future contract, workspace, effect, secret-delivery, or provider-authority treatment.

## Uncovered Material Behavior

Behavior

Classification

Elixir dependency hydration, compile, format, and tests

Contract opportunity; not locally proved in this review

Rust NIF compilation and effective native loading

Repository maturity gap and contract opportunity; not proved

Elixir fallback behavior

Repo-owned intentional behavior; must be separated from NIF-required proof

Rust port worker

Locally exercised; not yet contract-owned

MCP framing and routing

Locally exercised with the smoke harness; real evaluator and host loading not proved

Python conformance families

Selected suites locally exercised; not yet contract-owned

Site build

Locally exercised; formatting currently red

Guarded GitHub merge

Future effect/provider boundary; live mutation not proved

Credentialed CAEP provider pilot

Future secret-delivery/effect boundary; not exercised

Pages deployment

Repo-owned external behavior outside the proposed first PR

Liminal three-repository E2E

Workspace widening opportunity; not exercised

Gmail/GitHub synchronization

Prose-governed future communication/external-data effect

Production identity, replay, key management, and policy administration

Explicitly not proved

macOS and Windows contributor behavior

Not proved

## Next Decision

Proceed with asynchronous lane review now. Publish a forked draft PR only after v1.6.27 is released and the initial lane taxonomy is agreed. Treat every gap discovered during that work as one of: repository contract work, repository implementation drift, an Ota maturity defect, a bounded V12 or later-version input, a governance gap, or a deliberate widening opportunity. Do not absorb one class into another to make the pressure result green.
