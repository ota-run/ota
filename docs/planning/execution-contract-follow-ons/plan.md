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

# Cross-Cutting Plan: Execution Contract Follow-Ons

Status: mixed. Aggregate selected-graph correctness was completed and independently reviewed as a
bounded V12.1 correctness dependency. Section A has an activated, bounded native
source-authoring increment under its named sequencing exception. MUSE and VLAD accepted
its repaired source checkpoint; runtime validation remains on hold. The other registered
follow-ons remain inactive.
This register does not otherwise change released behavior, delay the Eris adoption PR, or widen
V12.1 secret-delivery scope.

## Purpose

Bounded Eris adoption pressure proved that released Ota v1.6.27 can govern one native agent lane
and a smaller digest-pinned container lane without claiming that the model, Qdrant, external
integrations, or the complete repository ran. The same pressure exposed six product opportunities:

1. lock enforcement for typed Cargo hydration;
2. deterministic mode eligibility for aggregate tasks;
3. clearer default and explicit selection for mixed-mode previews;
4. root-cause reconciliation for repeated Doctor advisories;
5. stronger declaration and evidence boundaries for source-owned shell orchestration; and
6. identity-bound terminal cleanup evidence for container-created resources.

Section G separately records source-confirmed bootstrap-integrity and drift-coverage gaps
exposed by ATR PR #618. It is a repair candidate, not an Eris proof result or an activated slice.

These are not one feature. This document records their separate ownership and activation bars so
that pressure findings do not become either forgotten backlog or an unjustified expansion of the
active version.

## Activation And Sequencing

Each sub-slice is independently activatable and closable. An active version plan must name the
sub-slice before implementation begins. Activating one does not activate the others.

V12.1 remains the only active product version. A follow-on does not enter V12.1 merely because
pressure exposed it. Aggregate mode eligibility was the sole exception recorded here because the
Discord failure proved that V12.1 admission consumed contradictory selected-graph truth;
the repair was completed at a between-batch boundary without widening secret-delivery scope.
Section A's explicit exception permits planning only while Step 7 remains open;
it does not satisfy or waive the implementation sequencing gate. Every other
follow-on may be selected only at a later between-batch boundary or by a later
version after the current V12.1 step is committed, reconciled, and pressure-ready.

Activation requires:

- one named implementation owner and exact affected product surface;
- regression fixtures that reproduce the pressure signal without depending on the Eris fork;
- an explicit compatibility posture for existing contracts and JSON consumers;
- a bounded pressure target able to exercise the new behavior;
- connected Core, Site, Skill, Example, Learn, FAQ, and Glossary assessment; and
- independent review before the implementation is called complete.

Eris supplies the discovery evidence for sections A-F; section G has its own ATR/source evidence.
Neither is sufficient support, conformance, or release evidence for an activated implementation.

## A. Lock-Strict Cargo Hydration

### 1.6.29 Scoped Planning Exception

Status: ONE bounded native source-implementation increment activated on
2026-10-10 by MUSE after scoped planning; runtime validation remains HOLD.
Bobai requested work after the recommendation to select this one independent
follow-on while Step 7 remains open. Owner: SCOOBY. MUSE reconciled and accepted
the planning exception; qualified VLAD planning review
`01a1276a-e166-7b23-81f9-3d0b7602f0aa` found no P1/P2/P3. This is not design
acceptance by itself. Qualified VLAD concrete review
`01a1276f-d64e-7fc2-8d94-5bf9eca161a6` found no P1/P2/P3. MUSE independently
accepted the requirements amendment and explicitly activated this source-only
increment under Bobai's selection and standing delegated roadmap authority.
This is a narrow exception to the current-step/pressure-ready sequencing rule
for this increment ONLY, not a finding that Step 7 is pressure-ready.
This does not activate the other follow-ons, Step 8,
V12.2, a release, infrastructure, provider operations, commits or pushes.
The closed kernel experiment and native-admission HOLD remain unchanged.

Planning checklist:

- [x] Reconcile this planning exception in the active V12.1 plan and canonical handoff.
- [x] Specify absent/false/true compatibility and the canonical Cargo source
  root, including Cargo workspace/member lockfile selection.
- [x] Reuse typed command selection, hydration provenance and replay identity;
  specify native, container and Compose-backed paths without shell-only glue.
- [x] Specify canonical path/object/byte admission and pre/post drift checks;
  distinguish observed replacement from unobserved concurrent or ABA mutation.
- [x] Specify regression and bounded real-repository pressure requirements; execution is not admitted.
- [x] Assess all connected first-party surfaces before source implementation.
- [x] Obtain MUSE and qualified non-author design acceptance, then separately
  activate only the first bounded source-implementation increment.

Authoring gate: Core source/manual fixtures/tests/spec/JSON-reference/Unreleased
only, with trusted local read/hash/text/diff inspection. No tests, build, target
evaluation, formatter/generator, Cargo invocation, containers, network, SDK/
provider/root/VM/spend or commit/push/merge/release. No dependencies, Cargo.toml,
Cargo.lock, version, protected ota.yaml/AGENTS/workflow, Protocol/Launcher/pins
changes. Generated schemas await a separately admitted generation route.
Bind both member and root manifests where selection depends on them; include
early unsupported-backend admission across nested/hook/aggregate selections.

Baseline discovery: before this increment `TaskCargoHydrationSourceSpec` had
cwd/Compose fields but no lock posture; runner argv/preview hardcoded unlocked
fetch, and summaries lacked lock-policy evidence. The current unvalidated
source now stages the shared posture, native admission and sampled receipt path.
Existing hydration provenance remains the owner, not a new execution or
authority subsystem. No runtime, generated-schema or pressure acceptance yet.

### Accepted Design: Source Authoring Only

**Compatibility.** Add `locked: bool` to the existing Cargo source, default
false and omit false from canonical serialization. Absent and explicit false
remain the same unlocked semantic posture and execute the existing fetch path;
neither starts new lock-root probes or requires Cargo.lock. True is authored
semantic input and selects structured `cargo fetch --locked`. Preview and
execution derive arguments from one shared Cargo plan, not independent strings.
Do not infer true from a discovered lockfile. Older runtimes reject this new
field under their existing deny-unknown-fields rule; public guidance must name
the supporting release rather than describe older binaries as compatible.

**Root selection.** Do not implement a second Cargo workspace resolver. A
locked native execution first uses the selected admitted Cargo executable and
environment for `cargo locate-project --workspace --manifest-path
<canonical-cwd>/Cargo.toml --message-format json`. This is an explicit bounded
metadata predecessor, not fetch/build or a repository-authored script. Local
installed Cargo documentation confirms the JSON `root` member and that
`--workspace` follows ancestor roots and package.workspace. Bind the returned
manifest path, require it to be a regular canonical non-aliased path inside the
contract repository, and select its sibling Cargo.lock. Standalone and root
cwd therefore select their own root; member cwd uses the actual workspace root.
Refuse failed/malformed/oversized metadata, missing lockfile, outside-repo root,
symlinked components, nonregular or multiply linked files and a root/manifest
identity that changes across admission. No fallback to cwd/Cargo.lock.
Metadata command output is capped at 64 KiB and deadline at 5 seconds through
existing bounded process controls; unsupported control paths must refuse.
No metadata command runs in validate, Doctor or dry-run: they show authored
posture and `root_resolution: not_proved`, including the planned predecessor.
Do not mark a guessed lock identity as resolved or imply fetch has run.

**Observed identity and outcome.** Existing hydration provenance owns a Cargo
record, and existing receipt evaluated-input/replay machinery owns its sampled
lockfile entry. Bind selected cwd/root, root-manifest content identity,
repository-relative lock path, observed object identity and raw-byte SHA-256
(16 MiB maximum lockfile input). The planner returns a resolved/unsupported/
unavailable result, not a bool-only verdict. Execute only after a fresh root,
path/object and byte recheck matches the admitted samples. Sample again after
the command, including nonzero exit. Preserve the actual command status;
observed drift makes the hydration result failed even if Cargo exited zero.
Required typed outcomes: missing_lockfile, unsupported_root,
unsupported_backend, identity_changed_before_execution, drift_after_execution,
observation_unavailable and unchanged_at_samples. Use existing typed error and
receipt categories; these names are proposed outcome values, not CLI commands.
Unresolved/missing observations never produce a positive lock identity. Replay
compares authored posture and sampled inputs; it cannot manufacture historical
observations by hashing the current file after a run.

Required machine-readable limit for true, even on success:
`acquisition: not_proved` and `concurrent_mutation: not_proved`. The samples
prove observations and selected command policy only. They do not prove the
exact bytes Cargo opened, absence of ABA replacement, registry/package integrity,
offline/hermetic resolution, or malicious-actor exclusion. No new broker,
immutable-copy workspace or authority/custody subsystem is proposed.

**Backend boundary.** First source increment supports direct native Linux/macOS
only. Orchestrator-wrapped prepares are also refused because their shell wrapper
would bypass the typed observer. Other native platforms and locked container,
remote and Compose-backed sources produce explicit
unsupported_backend refusal before starting fetch or a container/provisioner;
they must not fall through to existing shell commands or borrow host hashes.
Unlocked behavior remains unchanged on all existing backends. A later reviewed
increment within this same feature must admit the actual selected container
root/view and observations through existing execution-context paths. Until
that is designed and proved, native/container parity is an OPEN completion
gate: native-only source acceptance cannot close, ship or release Section A.
Metadata uses the same native executable resolver/environment as fetch, with
regular-file output sinks (32 KiB each), a 5-second leader deadline, and
unreaped-leader group termination before final status acquisition. The waitid
compatibility, normal/error cleanup and aggregate containment are NOT_RUN;
process-group signalling is not proof against escaped descendants or hostile
toolchains. These controls require independent source and runtime review before
validation admission. Toolchain-shim auto-install behavior must be assessed in
that admission; no fictitious Rustup offline environment setting is permitted.

**Requirements disposition.** The previous unconditional refusal requirement
for every concurrently replaced lockfile is not implementable by sampling
alone. This candidate explicitly replaces it below with observed-substitution
refusal plus mandatory acquisition/concurrency not_proved evidence. This is a
acceptance amendment explicitly accepted by MUSE after qualified non-author
design review, not a claim that the old unconditional requirement was satisfied
or any reduction of Step 7 custody guarantees.

### Implementation And Verification Candidate

Smallest first batch, after separate activation: shared Cargo plan/evaluator in
existing schema/hydration-provenance owners, native pre/post admission and
structured execution, typed refusals for other locked backends, and coherent
Doctor/preview/receipt/replay data plus focused fixtures. Update generated
schemas and Core spec/JSON reference/Unreleased with that batch; do not expose
an arguments-only implementation as complete. Runtime imports/tests/builds
remain unauthorized by this planning record.

Regression matrix (each row has a separate asserted command and evidence result):

| Case | Expected Result |
| --- | --- |
| absent / false, all existing backends | Existing fetch behavior and legacy output compatibility |
| true standalone / workspace root / member cwd | Cargo-selected in-repo root, locked argv, matched samples |
| malformed metadata / external root / missing lock | Typed refusal, no fetch |
| root/path alias / hard link / replacement before dispatch | Typed refusal, no fetch |
| bytes or object replaced during command, still different afterward | Retained command status plus failed drift result |
| unchanged samples / deliberately restored ABA fixture | Narrow observations only; acquisition and concurrent mutation not_proved |
| nonzero command / unreadable post sample | Original exit retained, no successful hydration proof |
| true container / Compose / remote in first batch | Unsupported refusal, no process/container/provisioning for fetch |
| dry-run / Doctor / receipt / replay | Same posture; no invented root resolution or historical sample |

Use existing fake-Cargo runner fixtures with fixed locate-project JSON and
deterministic mutation hooks, plus a no-dependency real Cargo workspace fixture
for later offline root-selection corroboration. Fake command logs are not proof
of Cargo semantics; the real fixture is a distinct required test.

Proposed first offline driver is only the existing declared Core check task,
`ota run check --native --agent`, not direct Cargo outside agent admission.
This still runs setup and may execute build scripts/proc macros; it is not a
source-only check. The full `ota run test --native --agent` is a separate later
gate, not included in this first admission. Setup/toolchain fulfillment may
require network, so clearance must first
bind its selected graph and prove installed toolchain plus dependency cache
suffice with Cargo offline mode and enforced network denial. Rustup fulfillment
must need no refresh/install; an attempted network or installation fallback
fails admission rather than relying on an invented Rustup offline setting.
Missing prerequisites STOP; no install, pull or network fallback. Require an
existing qualified isolation route with read-only source/lock, separate bounded
target/scratch (8 GiB), 4 GiB memory, swap0, 2 CPUs, 256 PIDs, 1024 FDs,
20-minute parent deadline and 1 MiB retained output. No claimed route is selected
or executed here; exact driver identity/settings need review before validation.
Do not use the unqualified Darwin R3 supervisor for this ordinary test gate.

#### Combined Accepted-Source Offline Validation: DESIGN R1 Accepted / Execution HOLD

MUSE selected ONE design-only packet after accepting local-child SOURCE R2.
SCOOBY owns this section and the live handoff; no parallel tracker or validation
driver is created. This binds the combined accepted Cargo, native Rust-restriction,
workspace-parent and inspectable local-child source, not merely the latest repair.
Qualified non-author VLAD review `01a12a38-8fab-7213-a99a-8de4dfd57cd2`, final
`msg_0074fa586a19efbe016acb528625f8819183378bd334e7376a`, found no P1/P2/P3
in this HOLD inventory. MUSE independently reconciled HEAD, the exact review
delta and 33 packet/input/action/local-binary digests, accepting DESIGN R1 as a
correct prerequisite inventory, not an execution-ready packet. Reviewed plan
`dd677661e374d410f272aee6f9de719aa6daeab31823a7909088bcafb3be01b1`;
state `bf33bb4eea01cba844fbddc88b49172ee1122bf7ccbcee5b6dfca42b9fc340bf`.
Execution HOLD is not approval to bootstrap or run a check.

**Exact candidate / inspection evidence.** Core branch `1.6.29-implementation`,
HEAD `8926ddb1fc0e4a225059c48375fc6e1185081b1e`, with uncommitted repairs.
The review-only combined Git delta identity is SHA-256
`9c026a18a8fd5c4eeef775bb363796ca82ee57809b880b0ceeea116b7275c681`,
derived with `git diff --binary --full-index HEAD -- .` excluding only this plan
and current-state status files. No untracked files were reported. This Git
identity is NOT a raw sealed execution-snapshot manifest or a clean/released
driver identity; any later sealing must bind every raw source/build input,
embedded reference/schema asset, mode/link and lock byte and preserve this delta.
The accepted local-child 17-file packet identities are retained in its section.
Additional combined dirty source identities (SHA-256):

| Input | Inspected digest |
| --- | --- |
| `src/cli/commands/effect_refusal.rs` | `a1e3c01cdfdac0723b3147452754b3a262fd4aa5aeccecad52f3b7d87d0a9cdd` |
| `src/cli/commands/init_starter.rs` | `83160484dccbd2e8ab4ebb75240c64a0029d5e1d4691ba5c4d3df88657d59a90` |
| `src/doctor.rs` | `caa3636989cc635d8a06df40c6798b420550c7c3c8608d58da74160221ae8f24` |
| `src/hydration_provenance.rs` | `7e289b52447fe3891fa880272a4378b2b55052b1e2cf226bda2e30944a7892a0` |
| `src/output.rs` | `38459d38b8a239bcea1efb8450be184785c9cca844cabd42708850fffb7c3f7b` |
| `src/published_contract_schemas.rs` | `eeb3ad4425141a47708c7a9f107fabeec0963f4d46ae8036c4de09c44dd52ac3` |
| `src/schema.rs` | `185ec0675e88e281e14c61731d52bfe124c3fa60896f04ce085ebff244d8a11b` |
| `src/validator.rs` | `2f1987b44dee0cfe4a9593813c22caa8e1bd0905d58e00e07f03fc2e0c079b23` |
| `ota.yaml` | `3fb331bf8f15efc85f0ea7066803523234cc35ccf72efa32c09fbfe9aa11fa6c` |
| `Cargo.toml` | `0a419f4f0ecd3c6d8d9fd309086291cbef00cd939fe1a037cd7afc1cf95c1c99` |
| `Cargo.lock` | `8f653b6fda4a7fdf6576ed59a02746b02b2bc9bc9558c6c05db97ecaa7b68367` |
| `build.rs` | `1bcad0a24fbd43a6a994a21e7abadf8720560446bd704418677e47bc934c5ab0` |

Cargo pins Authority Protocol `e5fe1c83e562e02f60e27026c7148918bd016155`.
Default features/profile, target, build-script/proc-macro inputs, compiler/Cargo/
components/sysroot and registry/Git cache manifests must all be bound before
execution; a lock hash or directory existence cannot substitute for those bytes.

**Driver / route inventory (metadata and digests only).**

| Prerequisite | Actual inspected state / admission |
| --- | --- |
| Installed driver `/Users/bobai/.cargo/bin/ota` | SHA `9afa5210fcab9da87ddc08d765150567ad760c576eb802dd6c8787e4a1cc4d9c`; source/build provenance to this dirty packet NOT_VERIFIED; not admitted |
| Existing `target/debug/ota` | SHA `27515598af42fc5240cb1274605b09792bc05f815b207f077d39862bab07b543`; provenance to this packet NOT_VERIFIED; not admitted |
| Existing source-build action | `.github/actions/install-ota-from-source/action.yml`, SHA `bb2a3a3d286264c210dcc4a66f1c3e46f6c0bb7321c396598672a5e676dbd214`; installs Rust (default stable), Cargo-installs and invokes version; no retained driver/build-input manifest |
| Toolchain selection | Contract requires Rust `1.95.0` + `rustfmt`, fulfillment `run`; local `1.95.0-aarch64-apple-darwin` directory exists, which proves neither installed readiness nor a Linux toolchain |
| Local shims/cache | Cargo/rustc symlink to rustup; rustup SHA `ec1b9233e7f72990ecd8e62063fa7f6c3dfc2bec8e97f88bff165f9100ac696a`; registry/Git cache directories exist, contents/sufficiency NOT_VERIFIED |
| Isolation and Linux image/cache | No existing route selected or qualified by this packet; digest, effective mounts/network/resource/cleanup settings NOT_VERIFIED; no daemon query/pull/start occurred |
| Operator-owned local policy | Required exact local `policies: {toolchain_fulfillment: {rust: deny}}`; file path/owner/raw+semantic identity NOT_VERIFIED; no policy file created or remote fallback allowed |

**Only proposed first command.** Existing `ota run check --native --agent`:
canonical selected dependency graph is `setup` (`cargo fetch`) then `check`
(`cargo check`), both Rust-requiring agent-safe tasks. Preserve `ota.yaml` and its
PATH prepends (`/usr/local/cargo/bin`, `/usr/local/rustup/bin`); do not replace the
task, strip setup, invoke Cargo directly or omit agent admission. Check executes
build scripts/proc macros; it does not run tests. Compiler success alone does not
prove the NOT_RUN regressions or runtime refusal behavior. No full test suite,
schema evaluator, formatter, generator or release gate is included.

**Execution admission prerequisites (all unmet items STOP).** Bind a repaired
driving binary to the full dirty snapshot/build manifest and exact artifact
digest; sealed read-only source/lock/embedded data and toolchain/cache mounts;
exact target/features/profile and launch environment; fixed local operator-owned
deny pack, no inherited alternate policy/shim/compiler/config source. Pin actual
Rustup/Cargo/rustc/rustfmt paths and bytes including source-managed and contract
PATH precedence. Set `CARGO_NET_OFFLINE=true` but independently enforce outer
network denial with no host daemon/credential sockets or privileged fallback.
No shim refresh/install/download, remote policy or missing-cache fallback is
allowed. Readiness/cache/probe qualification must occur only in a separately
admitted bounded preflight; no-Cargo metadata cannot establish cache sufficiency.

Reuse an existing independently qualified containment/cleanup route, not the
unqualified Darwin R3 supervisor or a new runtime wrapper. Source/lock/policy
stay read-only except an empty non-symlink 64 MiB `.ota/state` overlay. Other
target/HOME/cache/temp scratch gets the remaining 8 GiB minus 64 MiB; aggregate
memory is 4 GiB (including tmpfs), swap0, 2 CPUs, 256 PIDs, 1024 FDs per process.
Preflight/check/cleanup share one 20-minute parent deadline with cleanup reserve
and one 1 MiB retained output budget. The approved outer owner retains exact
resource creation identity, command/status/deadline/failure/output evidence and
owns cancellation, descendant/resource removal and independent absence checks.
Ambiguous creation, incomplete cleanup, budget exhaustion or unidentified mounts
STOP and retain failure; never retry, delete unrelated resources or imply success.
Actual route, resource owner, cleanup evidence path and independent absence
mechanism remain NOT_VERIFIED, so there is no execution-ready command packet.

**Decision / smallest next authority.** HOLD: no inspected repaired retained
driver, no qualified Linux containment/cache route, no sealed execution manifest
or approved local pack. Do not fill these gaps with a released/dirty binary,
direct Cargo or runtime probes. First request a separately scoped first-party
bootstrap-only build/retention prerequisite using the existing source-build
implementation with minimum provenance/artifact retention, then independent
binary/no-install/containment admission. This needs explicit source/build scope;
it is not granted here and does not authorize commit/push, hosted dispatch,
toolchain installation, download, VM/spend, merge or release. No external resource
was created. Step7 OPEN/R1 BLOCKED, Step8/V12.2 inactive; public propagation OPEN.

**Active next / MUSE sequencing.** ONE DESIGN-only first-party bootstrap/
provenance prerequisite, using the existing source-build seam while preserving
ref-build consumers. The action and all production/test/helper source remain
READ-ONLY. Design must bind sealed dirty raw inputs and contained build before
compiler/build-script/proc-macro execution, minimally retain binary digest/build
provenance, and separate immutable dependency/toolchain payloads from bounded
writable Cargo housekeeping/HOME/target/scratch. No global driver replacement,
new runtime wrapper or speculative pipeline refactor. Freeze the concrete design
for qualified VLAD review, then MUSE's exact source-write-set decision. Missing
Linux route/cache remains a named prerequisite, not something planning proves.
No implementation, compiler/probe/bootstrap/dispatch/install/download/network/
infra/VM/spend/commit/push/merge/release authority is granted.

#### Retained Bootstrap Driver: DESIGN R2 Inventory Accepted / Source HOLD

**Decision / scope.** Design only, SCOOBY owner. Preserve the existing source-build
action's ref-install behavior and conditionally add one sealed-artifact mode at that
seam, with one small first-party build/retention helper. RUBY's read-only seam
analysis supports this option over a separate driver pipeline. It is not
independent design acceptance. No action/helper/test implementation or execution
is authorized. This design does not qualify an isolation route or cache by itself.
Qualified non-author VLAD DESIGN R1 review
`01a12a56-6e56-7ed0-864d-c2557ecd01c3`, final
`msg_0074fa586a19efbe016acb5a5fba7481918c30750cdc1e95c6`, held P1: the
owner/helper trust interface was deferred until after source authoring. Reviewed
R1 plan `0793d6ae07c4dffb63024019823887be6fd6af457b1c83dd32536525558854fc`,
state `9703fbc10534c7e96d65050d36ecfaa81aedade5373e1e32a1ef2853a64c2a64`.
MUSE independently reconciled that finding and cleared only this DESIGN R2
record and bounded discovery. R2 makes the missing interface explicit;
it does NOT repair the P1 or clear source. Option A remains NOT_SOURCE_READY.
Qualified non-author R2 review `01a12a5f-871f-70c2-b60a-fd390e62ae6f`, final
`msg_0074fa586a19efbe016acb5ca9ebac819183016732e04b573e`, found no new
P1/P2/P3 in this truthful HOLD inventory. MUSE independently verified exact
persisted review-turn model/effort, frozen identities and bounded candidate
incompatibility, accepting the inventory ONLY. Reviewed R2 plan
`0d4160fc532280cd8102f475bb4679314d4ae8618e7aa4b4ed91be7c698594eb`,
state `516379516fb84d4cd88f1c0f369cf5759d9651379516110deb27e536323b3620`;
these identities precede this acceptance-only record update. Original P1 OPEN.

**Compatibility and proposed interface.** Keep existing `repository`, required
`ref`, `path` and `rust-toolchain` inputs/defaults unchanged. Add `source-mode`,
default `ref-install`, with exactly one alternative `sealed-artifact`; reject
unknown modes and mixed mode-specific inputs before any action step. In the
default mode, retain checkout, toolchain setup, Cargo installation and version
verification semantics. That legacy path is NOT the offline-validation bootstrap.
In sealed-artifact mode, `ref` identifies the approved base commit, not the dirty
build identity; require an exact commit matching the sealed manifest. Require
explicit absolute sealed-source, expected-manifest-SHA-256, build-packet and
exclusive retention-directory inputs. Do not interpret `path`, `repository` or
`rust-toolchain` as a request to refresh anything. This mode skips checkout,
toolchain setup, install and version execution; no PATH/GITHUB_PATH mutation,
global `~/.cargo/bin` replacement, artifact execution, upload or fallback.
Inputs cross into the helper as literal argv/environment values, never shell
source interpolation. Artifact path/digest and terminal record are outputs only.
No inspected local caller requires a default change; external callers are not
claimed absent. No existing release/systemd proof workflow is repurposed.

**Approved inputs, not ambient inference.** The independently authorized outer
owner supplies a sealed directory and a strict versioned internal manifest;
this increment does not add archive extraction, snapshot transport or another
runtime supervisor. The expected manifest digest must come from the operator's
approved packet outside the worker, not a digest recomputed from arbitrary input
and then trusted. Bind the approved HEAD, review-only delta and a complete sorted
raw inventory: normalized relative path, directory/regular-file type, permission
mode, byte length and SHA-256. Reject duplicate/unknown fields, absolute/escaping
paths, duplicate entries, missing/extra files, symlinks, hard-linked files,
special files and mutable source inputs. No silent dereference or exclusions
for build inputs. Bound manifest/file counts, inventory bytes and total snapshot
bytes in the frozen execution packet; missing concrete limits STOP. Git delta
is contextual review evidence, never a substitute for this inventory.

Include Cargo manifests/lock, build.rs, all source and embedded
`docs/spec/json-schemas` bytes. Include self-contained Git metadata needed for
the actual HEAD/index/refs/object closure because build.rs reads Git source/dirty
labels. No borrowed ancestor Git repository, external gitdir/worktree pointer,
object alternates, remote fetch, hook or ambient/global Git configuration. Use
the actual admitted metadata, not fabricated clean labels or a new build.rs
override. Verify the base and retained dirty state independently before sealing;
an unsupported Git layout/configuration or unresolved object STOPs this first
increment. Operator sealing precedes execution and remains read-only throughout;
post-hoc digest comparison alone does not prove absence of replacement/ABA races.

Bind the action/helper/Python and installed Cargo/rustc/toolchain/sysroot bytes,
native linker and required host tools/libraries, exact target/features/profile,
configuration and allowlisted environment. Invoke admitted real Cargo/rustc paths,
not ambient Rustup shims; exclude compiler wrappers, inherited flags/preload,
ancestor/user Cargo config and credential/daemon sockets. Retain the complete
registry and Git dependency payload closure, including the locked Protocol pin.
build.rs also invokes Cargo metadata for `secret-delivery-pressure` and rustc cfg
for Linux/X64: default-feature compilation alone does not establish cache
sufficiency. Those nested commands inherit pinned CARGO/RUSTC and
`CARGO_NET_OFFLINE=true`; independently enforced network denial remains mandatory.
No cache miss authorizes a download, lockfile change or feature reduction.

**Build ownership and command.** A concrete existing outer containment owner,
not a caller-provided JSON assertion, must establish and retain the effective
mount/network/resource/identity/deadline controls BEFORE any Cargo, compiler,
build-script or proc-macro execution. No host-side preflight Cargo/metadata or
uncontained helper build is allowed. The trusted helper is acquired separately
from the dirty target and is read-only; it must not import target code as Python.
The proposed build is one fixed native Linux/X64 default-feature dev-profile
`cargo build --locked --offline --bin ota --target x86_64-unknown-linux-gnu`, with
2 jobs, incremental disabled, an empty owned target and explicit environment.
Any change to features/profile/argv requires a new frozen packet. This is a
bootstrap-only prerequisite: direct Cargo needs a separately granted explicit
bootstrap exception. It never replaces the later canonical setup -> check via
`ota run check --native --agent` or implies that NOT_RUN tests passed.

Source, Git metadata, toolchain and dependency payload bytes stay immutable.
Only bounded private HOME, target, temp and Cargo housekeeping are writable.
Cache mount geometry must distinguish lock/index housekeeping from immutable
registry archives/extracted sources/Git objects/checkouts; source checkouts and
Cargo configuration are not writable scratch. A private overlay is admissible
only if payload writes are prevented, not merely discarded later. Missing lock
slots or an incompatible cache layout STOP rather than chmod/remount/install.
Actual mount geometry and offline sufficiency remain NOT_VERIFIED. Do not assert
that an ordinary read-only CARGO_HOME works, or invent a new cache backend.

The build worker cannot write approved manifests/helper bytes or trusted retained
evidence. It writes only its bounded candidate target/log sinks. The outer owner
holds the actual launch/exit/resource observations and owns cancellation,
descendant quiescence and cleanup; the helper is not a new process supervisor.
Only after successful exit and independently established descendant quiescence
may the trusted retention phase acquire the fixed candidate by no-follow handles,
require regular file, one link, approved ownership/length/mode, and copy it into
an exclusive operator-owned retention directory. Hash the retained bytes and
seal them read-only; reject preexisting output or changing candidate identity.
No worker-provided success JSON substitutes for these observations. The explicit
existing-owner/helper interface must be identified, inspected and independently
accepted BEFORE any of the proposed action/helper/test source is authored, not
invented during implementation or deferred to an execution packet. Its actual
runtime qualification is a separate later gate. No such source-compatible
interface is identified by this bounded inspection. Naming an owner is not
admission; do not add a wrapper or claim strings attest kernel containment.

**Missing prerequisite: MISSING_EXISTING_BOOTSTRAP_OWNER_INTERFACE.** The required
roles below specify constraints, NOT an implemented wire protocol or authorized
adapter. Their concrete invokers/channel/acquisition/consume mechanisms are
UNBOUND and unusable until an existing inspected interface is supplied:

| Phase / Invoker | Required Truth And Write Boundary |
| --- | --- |
| Owner preflight/launch, outside worker | Exclusively owns one invocation and approved packet/source/toolchain/dependency/config/budget identities, resource generation and fresh candidate destination; establishes containment before compile and retains actual launch identity. Worker cannot write this custody. |
| Contained worker build | Reads approved immutable inputs; writes only candidate target and bounded log sinks. It cannot invoke trusted finalization, write its observations, widen inputs or mint a successful driver record. Zero exit or worker output is not quiescence/retention authority. |
| Owner finalization/retention, outside worker | Uses its own same-invocation exit, terminal descendant/resource observations and owned cleanup result; acquires the fixed candidate only after quiescence, hashes retained bytes and consumes the one finalization opportunity. Only this owner can publish trusted retained evidence. |

The owner must bind packet/source/toolchain/config/budget, actual launch/resource
generation and candidate path/object/length/digest to the SAME invocation.
Refuse missing/stale/replayed/mismatched or incomplete observations, any worker/
caller-forged completion, late or changing candidates and a second finalization.
Cleanup uncertainty is terminal failure, not a reusable pending success token.
Serialized owner records may carry evidence, but require protected provenance,
acquisition and transaction linkage outside worker control; JSON shape or a
digest supplied by the caller cannot establish those properties. Actual channel
and replay-consumption mechanisms remain UNBOUND, not TODOs inside the helper.

**Bounded candidate discovery / no promotion.** At most two named existing owners
were inspected; no daemon, runtime, environment or cache query ran:

- Frozen experiment companion
  `/Users/bobai/.codex/artifacts/2026-10-10/kernel-custody-outer-r13-r3/cloud_control.py`,
  SHA `1a7354be26ca676c218dff33533cc3652d33f50606f5d5de324365b25af71c87`,
  manifest `089117de71004bfdf7c0f24dc083fb3cd1909bbae36da17559bce0c5a61d4a7d`;
  directly referenced R13-R2 `outer-launch.sh`, SHA
  `8b3aab22f5b553647baf2b04e66b6ce9a2970bc202c361906d8a789608eb66ef`.
  Controller owns fixed cloud/SSH command groups and retained root experiment
  archives; root launch owns a fixed kernel/guest service, cgroup stop and
  collector allowlist. It exposes no sealed Cargo-driver transaction/retention
  interface. Source-ready only, native admission HOLD, expired/false live grant,
  different budgets and unproved compatibility/cleanup. Neither these historical
  records nor source inspection permit repurposing the experiment.
- RUBY's disjoint read-only Launcher candidate at HEAD
  `60c22cf9b2e8aa5897b56430bf27a038fd7b919a`: `src/systemd_service.rs`, SHA
  `1a39a1fd28efdc4bdce1a8165b8ce39d3033a82c8e61a6037ed257d0b7c91a92`,
  and `src/systemd_scope.rs`, SHA
  `2ac03f1328df0c2b9ba65ee38ede4a3c28126d955d3fba51bd6ba937b73c3f8a`.
  Service owns peer/mapped-principal, fixed protected Ota child, active slot,
  scope lifecycle and signed authority/lease/completion/receipt retention.
  ScopeBoundary is crate-private and typed to that child/scope, not a public
  Cargo build callback. Existing contract excludes caller-selected executable/
  environment/network options. No inspected generic bootstrap sealed mounts or
  post-quiescence binary harvest exists. Reusing its lifecycle would change the
  authority contract; no Launcher/Protocol/provider source changes are proposed.

The existing install action is only the compatibility seam. Directly inspected
linux-systemd-native-proof, smoke and release-gate build steps are same-job
install/proof or release/package/upload paths, not an independent owner handoff;
they are not substitute candidates or execution authority. This is bounded
absence among inspected interfaces, not a claim no suitable owner exists anywhere
or a new defect in Launcher. Stop discovery here; do not repeat it or expand into
general host exploration. The smallest next prerequisite is an identified
pre-existing operator-controlled Linux owner with exact readable source/path/
revision/hash, launch/terminal/quiescence/cleanup interface and protected
transaction-bound retention acquisition. If none is available, return for an
explicit scope/plan decision to introduce one. Existing-only scope cannot silently
authorize a new wrapper/supervisor/channel/service, even if it would unblock work.

**Retention, failures and budgets.** Retain raw inventory/digest, review base/delta,
helper/action/toolchain/dependency/config/launch identities, actual bounded logs,
exit/deadline/resource/quiescence/cleanup observations, artifact path/size/digest,
and a terminal complete-or-failed record. A complete record is permitted only
when acquisition, artifact binding and required cleanup/absence checks succeed.
Failure preserves bounded failure evidence; a partial candidate is not exposed
as a successful driver. Cleanup removes only exact owned scratch/resources;
approved retained evidence/artifact is not treated as disposable scratch. No
install, release provenance, runtime attestation, reproducible-build guarantee,
provider authority or execution-validation claim follows from a build digest.

Preserve proposed 4 GiB aggregate memory including tmpfs, swap0, 2 CPUs, 256 PIDs,
1024 FDs/process, 8 GiB total writable storage and a 20-minute parent budget
including preflight/build/cleanup with explicit cleanup reserve. No automatic
retry. Manifest/source limits and concrete cleanup reserve must be frozen before
execution admission. The existing 1 MiB retained-output limit cannot silently
cover a larger binary: propose at most 512 MiB separately for the retained driver,
with candidate AND retained copies charged against the 8 GiB total. Keep logs
within 1 MiB and separately bound manifest/provenance bytes in the execution
packet. This is a budget clarification PROPOSED, NOT APPROVED; source/design
acceptance does not grant storage/spend/execution or override a human boundary.
If dev-profile output exceeds the eventual approved cap/deadline, STOP; do not
switch profile, strip bytes or expand resources without a new decision.

**Conditional proposed SOURCE write set / proof obligations.** NOT_CLEARED and
NOT_SOURCE_READY until the missing concrete interface is independently accepted;
these paths describe the future seam, not authorization to create placeholders:

- `.github/actions/install-ota-from-source/action.yml`: additive explicit mode,
  pre-step strict input guards and unchanged default branch.
- `.github/ota/build_retained_driver.py`: one stateless first-party input/build/
  artifact-retention implementation at this seam, using admitted outer ownership;
  no runtime wrapper, installer, downloader or self-asserted containment.
- `.github/ota/test_build_retained_driver.py`: focused malformed/missing/extra/
  link/mode/hash/config/cache inputs, failure/partial publication, output collision,
  changed candidate and bounded retention source regressions.
- `tests/source_build_action.rs`: locks default compatibility, branch separation,
  no checkout/setup/install/version/PATH/upload/fallback in sealed mode and exact
  source/config handoff. Tests remain NOT_RUN until separately authorized.
- This plan and current-state only for coherent acceptance/limits.

No ota.yaml, Cargo.lock, build.rs, installer, runtime/CLI, release workflow or
other production/test surface is proposed. Tests must cover the production
input/retention path; YAML string guards cannot prove isolation or cache
sufficiency. A later separately approved contained build must establish actual
toolchain/cache/owner/quiescence/artifact/cleanup evidence. No public command,
schema, JSON or contract change is designed; Site/Learn/FAQ/Glossary/Skills/Examples
need no new propagation for this internal DESIGN-only prerequisite. The broader
product propagation remains OPEN. Changelog impact is reassessed if implemented
action semantics become a user-visible integration surface.

**Review packet / next.** Existing action SHA
`bb2a3a3d286264c210dcc4a66f1c3e46f6c0bb7321c396598672a5e676dbd214`,
build.rs/Cargo/lock/ota.yaml identities and HEAD/review delta are retained above;
all source remains unchanged. Freeze R2 for qualified non-author VLAD review and
MUSE acceptance as a truthful HOLD inventory, not implementation-ready design.
P1 remains an implementation blocker. Execution
HOLD: no selected qualified Linux owner, sealed snapshot, admitted Linux toolchain/
offline cache or approved retention budget. Next is the named owner/interface
prerequisite above, or an explicit scope decision if it cannot be supplied, NOT
source clearance or another abstract design loop. MUSE recommends ONE explicit
human-approved DESIGN-only scope amendment for a minimal dedicated operator-
controlled Linux build/retention owner using standard lifecycle/containment
primitives. Bobai approved that DESIGN-only amendment in human message
`01a12a69-f758-7ff3-8686-9270f8a4dd70`; the concrete design follows below.
That approval grants no source implementation or execution. Do not widen
protected Launcher/provider authority or create a general attestation service.
An operator-supplied concrete existing owner remains an alternative, not a
request that Bobai discover an unknown technical source path. Stop the current
inventory slice here; no further discovery or inventory-design iteration.
Only after concrete source
compatibility is independently accepted may the action/helper/test write set be
considered; live/bootstrap qualification separately needs exact route/toolchain/
cache/budget and explicit authority. Step7 OPEN/R1 BLOCKED;
Step8/V12.2 inactive. No compiler/tests/probes/network/infra/spend/commit/push.

#### Dedicated Linux Bootstrap Owner: DESIGN R3-R3 / SOURCE R3 Accepted / Validation HOLD

**Authority / refinement.** ONE bounded design only, SCOOBY owner. Bobai's direct
answer to `call_GO2z7bQ1oybc9rpsDmQYGC9e` in MUSE turn
`01a12a69-f6f6-7ee2-a28a-d74b503b8deb`, human message
`01a12a69-f758-7ff3-8686-9270f8a4dd70`, permits designing a dedicated trusted
Linux build controller rather than requiring an unidentified existing owner.
No controller code, builds, installs, infrastructure or spending. Qualified
non-author R3 review `01a12a7e-c954-7862-a31d-dc93aa1b89b3`, final
`msg_0074fa586a19efbe016acb64660d08819180dfd09c849552d0`, found no new P1/P2
and judged the original undefined-interface P1 repaired at DESIGN level; MUSE
reconciliation remains required. One P3 stale live active/write-set summary is
corrected in R3-R2 without changing the architecture/source. Reviewed R3 plan
`bd5977d34e0f5c3224a8cfec521621d2acb27665239a839bfa764597c0b9c472`,
state `620f7fe07fe601f21b641dda074c298570195cf5bb2c8a2425e9158e45e9e208`.
At the R3-R2 checkpoint, original P1 closure awaited MUSE; final design-only
acceptance is recorded below. No implementation or execution approval.
Qualified R3-R2 delta review `01a12a82-5406-7683-b774-5f94841da8e4`, final
`msg_0074fa586a19efbe016acb6505b2d0819192029f96e1970687`, closed VLAD's P3
with no new findings. MUSE independently reconciled the concrete interface, then
found a distinct P2 design overclaim: writable build output does not exclude
arbitrary descendant compiler/config choices. R3-R3 narrows that guarantee below;
this is not a demonstrated runtime escape or a request for new enforcement.
Qualified R3-R3 delta review `01a12a87-383d-77d2-bb28-ad81d56720c0`, final
`msg_0074fa586a19efbe016acb66492a5c8191b544a2829c2a2b0e`, closed the P2
with no new findings. MUSE independently reconciled exact reviewer-turn metadata,
frozen docs/source identities and repaired boundaries and ACCEPTED DESIGN R3-R3
ONLY: original P1 repaired at design level, P3/P2 closed, no remaining P1/P2/P3
in this bounded design. Reviewed plan
`3acf2ddc3c3f45a6587fe761530930d857aa9641507c1c4149bc4eccd9c4d81a`,
state `a3bdfd1cfc880a68757a9105250795748ffd2998baff2cf98061a60c65aba183`;
these precede this factual acceptance-only update. Source proposal is INACTIVE /
NOT_AUTHORIZED pending separate human approval. MUSE's technical review or exact
write-set recommendation cannot extend Bobai's design-only grant to code.
This design supersedes R2's proposed Action-driven build interface: the operator
is the first consumer, so leave install-ota-from-source/action.yml byte-for-byte
unchanged. Optional artifact consumption by an Action is deferred until an actual
caller needs it, never a privileged launch path. No new pipeline, daemon/socket,
general command runner, attestation service, or Launcher/Protocol/provider reuse.

**Trust model and exact proposed invokers.** Trusted operator owns the Linux
machine configuration and installed controller/Python/bwrap/systemd/kernel
closure. Root administration and kernel compromise are outside this boundary;
UID 0 alone is not evidence of an approved build. Untrusted source, build scripts,
proc macros, worker logs, candidate bytes and CI runner cannot select authority.
Require the exact operator-approved request at
`/etc/ota-bootstrap/requests/<nonce>.json` (root-owned regular, single-link 0400,
protected no-link ancestors), controller at `/opt/ota-bootstrap/controller.py`
(root-owned regular single-link 0444) and admitted `/usr/bin/python3` closure.
Nonce is exactly 32 lowercase hex digits, nonzero, operator-created and single-use.
Controller roles are fixed internal operations, not a public Ota command:

1. Operator starts the reviewed `ota-bootstrap-owner@<nonce>.service` template.
   Its only ExecStart is Python `-I -B`, the fixed controller, `build-owner`, and
   the derived request path. No root evaluation of a composite Action/workflow.
   Future staging/template installation needs separate explicit approval.
2. Build-owner creates its protected invocation journal and owns one stopped
   child, worker cgroup, private scratch, actual wait status and tentative artifact.
   Its child runs the same admitted script's fixed `worker` role via posix_spawn,
   not Python preexec_fn or arbitrary post-fork Python in the privileged parent.
3. After the owner has returned, the operator invokes the same admitted controller's
   `finalize-owner` role using admitted systemd-run, fixed unit name
   `ota-bootstrap-postflight-<nonce>.service`, Type=exec, root, Restart=no,
   MemoryMax=1GiB, MemorySwapMax=0, CPUQuota=200%, TasksMax=256,
   LimitNOFILE=1024, RuntimeMaxSec no greater than the remaining 120-second
   postflight reserve, KillMode=control-group and bounded stop timeout.
   The only command is admitted Python `-I -B`, the fixed controller,
   `finalize-owner`, and the derived request path. The controller rederives its
   actual postflight unit/generation/limits before reading the pending artifact.
   It independently queries PID1 and the kernel, terminally
   removes the exact build unit, and only then finalizes protected retention.
   This role cannot launch/restart a build, change input or consume another nonce.

The full later execution packet must bind both operator commands and effective
postflight resource controls; no unbounded direct root invocation is admitted.
Operator process/manager lifecycle after postflight is operator-owned, outside
the build-worker result. It must be separately retained/closed before subsequent
driver execution; the build record does not claim machine-global cleanup.
The eventual result explicitly carries operator postflight-unit termination and
resource-envelope closure as not_proved until the external operator retains
manager exit/absence; finalized alone cannot admit subsequent driver execution.

**Protected handoff / state machine.** Root-only
`/var/lib/ota-bootstrap/invocations/<nonce>` is exclusively acquired, 0700,
no-follow, with durable O_EXCL `started` before any worker. An existing path/token
refuses replay; no resume/re-entry/retry. Separate retained root
`/var/lib/ota-bootstrap/retained/<nonce>` is outside worker scratch and namespace.
Root records are strictly versioned UTF-8 JSON, duplicate/unknown fields refused,
canonical bytes hashed and acquired by held dirfds. Root ownership/protected
ancestors plus the admitted producer, operator packet and independently observed
facts establish custody; JSON shape or a caller-provided hash does not.

State order: `armed -> child_stopped -> placed -> building -> worker_terminal ->
artifact_pending -> finalized`; any failure is terminal `failed` or `interrupted`.
Only root owner writes these records. Bind request digest, raw source/Git manifest,
all toolchain/cache/platform/config/controller identities, exact argv/environment,
budget, boot ID and absolute boottime deadline, systemd InvocationID/unit name,
owner birth identity, child PID/pidfd/start identity, retained cgroup mount/dev/ino,
target directory object and fixed candidate-relative path. Artifact_pending adds
actual exit/status, forced-cleanup disposition, recursive terminal facts, candidate/
retained path/length/digest and owned scratch cleanup. It is NOT a usable driver.

Finalize-owner accepts only the original root-produced artifact_pending plus the
same approved request/producer/boot/unit generation. It independently requires
the retained manager generation's successful main exit, no live MainPID, and
empty worker/owner hierarchy; StopUnit and cleanup syscall success alone are not
terminal evidence. It holds/rechecks exact observed cgroup and retention handles
through cleanup, requires build-unit terminality and empty-or-absent exact cgroup,
then exclusively publishes one finalized record and consumes finalization.
Missing manager generation/evidence, restart, stale/forged/replayed/mismatched
records, nonzero owner exit, incomplete cleanup, changing candidate/retained bytes,
or publication failure refuses. No absent-unit shortcut before generation binding.
Root temporary files stay 0600 through complete write/fsync, then immutable final
0444 via no-replace publication and parent fsync. Interrupted pending artifacts
remain private/quarantined, not exposed as success. No worker/caller success JSON
can advance state. Trusted postflight cannot repair a failed build into success.

**Linux lifecycle / containment before compilation.** Proposed owner unit is
Type=exec, root, Restart=no, RemainAfterExit=yes, KillMode=control-group,
bounded TimeoutStopSec, fixed RuntimeMaxSec, private mount/network contexts and
root-only delegation of cpu/memory/pids. RemainAfterExit preserves successful
manager generation for postflight; it is not quiescence evidence. Manager retains
emergency whole-unit kill on failure/timeout outside controller survival.
Controller death at any phase cannot emit finalized success; no automatic recovery
or build resumption. Parent-death signaling is not relied on for transitive cleanup.
Effective manager policy and kernel limits must be rederived, not accepted from
template text. Missing support/configuration refuses before compiler.

Require unified cgroup-v2 domain hierarchy with all requested controllers enabled,
root-only ancestors/controls and no preexisting foreign processes. Owner moves
itself into a controller leaf before enabling domain subtree controllers; worker
leaf is separately root-owned, never delegated to its UID. Aggregate owner+worker
limits remain enforced at the unit ancestor; retained kernel directory handles
bind the hierarchy, not a reusable pathname or label. Child starts as a fixed
trusted shim within this ancestor, arms an own-parent identity check and SIGSTOPs
BEFORE compiler/build scripts/proc macros. Parent must observe its own child's
actual SIGSTOP via waitid, retain pidfd and boot/start identity, attach that stopped
child to the worker leaf, then independently verify exact stopped identity and
membership again. It writes one protected placement grant and continues via
pidfd only. Unexpected exit/stop/continuation, missing pidfd, partial placement,
wrong cgroup generation or absent grant refuses; no raw-PID fallback.

Trusted shim verifies the owner-issued same-invocation placement grant before
any build, then drops supplementary groups/all real-effective-saved IDs to the
dedicated build UID/GID, clears all capability sets/ambient/keepcaps, sets and
checks no_new_privs, and closes EVERY descriptor except fixed stdin and bounded
stdout/stderr transport. No journal/retention/control dirfd reaches the build.
Use admitted non-setuid bwrap to establish user/mount/PID/IPC/UTS/network separation,
drop namespace capabilities and expose only the frozen filesystem closure. No
host bus, cgroup control, credential/daemon socket, device/namespace handle or
privileged fallback. Host and namespace UID/capability/descriptor facts are
separate; namespace root is not host authority. Actual bwrap/Python/kernel
semantics and the posix_spawn/stop/drop ordering remain NOT_PROVED.

Readonly mounts bind complete source/Git/embedded data, platform/native tools,
Python/controller, exact Rust 1.95.0 toolchain/sysroot and dependency payloads.
Allow only private HOME/temp, fresh target and explicitly enumerated writable
Cargo housekeeping; admitted configuration, registry archives/extracted sources/
index and Git payload/checkouts remain immutable. Bind the top-level Cargo
configuration/search roots and declared launch inputs of the inspected first-party
nested metadata invocation; actually consulted admitted config/input locations
must be immutable, with no ambient inherited compiler wrappers/flags. This is NOT
a universal command/config policy for untrusted descendants: build scripts/proc
macros may generate wrapper-like files in executable target output, alter their
own child environments or spawn nested commands/configs. That behavior stays
untrusted within the proposed OS boundary, not independently audited or globally
excluded. All-descendant compiler/config enforcement remains NOT_PROVED. No exec
broker, syscall/argv filter, instrumentation or new sandbox flow is introduced.
Exact bind geometry is a prerequisite; if the
pinned Cargo needs unavailable writes or payload mutation, STOP, not chmod,
overlay-discard proof or online hydration. Target must permit the declared build
script/proc-macro execution, with nosuid/nodev; do not incorrectly mark it noexec.
Other scratch is noexec where compatible and declared. Private mount visibility,
dedicated worker principal and no outside writable alias are required, not merely
stable post-hoc hashes. Root/operator concurrency over this invocation is excluded.

Fixed worker build remains default-feature dev-profile native Linux/X64
`cargo build --locked --offline --bin ota --target x86_64-unknown-linux-gnu`,
2 jobs, incremental disabled, fresh target, pinned CARGO/RUSTC and allowlisted
environment. No shell or caller-selected command at this admitted launch; this
does not prohibit build-code child commands. Nested build.rs metadata's
secret-delivery-pressure closure and rustc cfg are included. Set Cargo offline
and GIT_OPTIONAL_LOCKS=0; enforce network denial independently before all code.
No Ota execution, installation/version/PATH replacement, upload or remote policy.
Direct compiler bootstrap still needs separately approved exact execution scope.

**Observed terminality, retention and crash behavior.** Parent acquires real owned
child wait status with WNOWAIT; no Popen.poll/wait or premature reap discards the
child generation before cancellation. Zero exit is not descendant absence.
After normal exit or failure, freeze further launch; any surviving descendant
after normal worker exit makes the build FAILED, even if later forced cleanup
succeeds. Use root cgroup.kill, observe recursive cgroup.events populated=0 through
the retained exact hierarchy, then reap the original child and remove worker leaf.
No single-directory cgroup.procs-empty predicate or process-group-only proof.
Kill/read/identity/removal timeout or ambiguity is failed/cleanup-uncertain; retain
failure, never touch an unrelated group. No relaunch after terminal observation.

Only then acquire target/x86_64-unknown-linux-gnu/debug/ota through the retained
target dirfd, with no-follow component traversal; require regular/single-link,
expected ownership/mode, bounded exact size, stable object and streamed bytes.
No external writer may remain. Copy into a root-owned exclusive pending file on
retention storage, fsync, seal/verify its digest and metadata, remove only owned
scratch/mounts and durably publish artifact_pending. Finalize-owner revalidates
retained bytes/producer/manager cleanup before final publication. RuntimeMaxSec
and manager group kill cover owner loss even while the child is stopped or ignores
signals; unreadable/ambiguous emergency cleanup leaves interrupted, not success.
Actual manager emergency behavior must be tested, not presumed from PIDFD or UID.

**Bounds / authority limits.** All phases share one non-resettable same-boot
1200-second deadline; proposed build-owner operational ceiling 960 seconds,
120 seconds reserved for worker cleanup, 120 seconds for postflight/publication.
No new clock/attempt on finalization. Aggregate 4 GiB memory including tmpfs,
swap0, 2 CPUs, 256 PIDs, 1024 FDs/process and 8 GiB writable storage remain ceilings.
Propose build unit <=3 GiB and postflight <=1 GiB, both charged against aggregate
budget including retained caches; exact effective accounting/outer postflight
containment must be frozen/qualified, never inferred from separate labels.
Scratch tmpfs <=7 GiB; candidate within scratch and retained binary <=512 MiB;
logs <=1 MiB combined, metadata <=16 MiB, inventory <=100,000 entries/16 MiB,
raw readonly source inventory <=32 GiB. Stream copying/hashing and logs, no
unbounded buffering. These are proposed design bounds, NOT operational approval;
512 MiB/artifact and metadata allowance still need human clarification with the
concrete execution packet. Exhaustion or unsupported limits STOP without resizing.

**Minimal future source / proof obligations.** Proposed only, NOT_CLEARED:
`.github/ota/build_retained_driver.py` (one controller/worker/postflight
implementation); `.github/ota/retained-driver-owner@.service` (fixed operator unit
template); `.github/ota/test_build_retained_driver.py` (portable state/input/
retention tests plus explicitly gated Linux lifecycle cases); canonical plan/state.
No Action, ota.yaml, build.rs, lockfile, CLI/runtime, installer, provider, Protocol,
Launcher, release workflow or unrelated test changes. No unused placeholder
adapter. Tests must exercise production functions, not just YAML string matching.

Portable obligations: strict path/link/owner/mode/manifest/env/argv/capacity
validation; transaction/generation state transitions; nonce replay, caller/worker
forgery, stale/partial/late-result refusal; collision, changing candidate and
copy/fsync/seal/cleanup failure; no finalized record from pending/nonzero/interrupted.
Altered/ambient config at admitted consultation roots must refuse; generated or
unobserved descendant commands must not gain stronger provenance from zero exit.
Separate Linux obligations: pre-placement compiler sentinel stays untouched;
wrong/reused child/pidfd/boot or cgroup and unexpected SIGCONT; unavailable
controllers/delegation/limits/namespaces; FD/credential/bus/cgroup-write leakage;
nested/rapid/signal-resistant descendants and escaped/migration attempts;
recursive quiescence and retained-object acquisition; manager emergency cleanup
on owner loss at stop/placement/drop/compile/kill/copy/seal/cleanup/publication;
postflight generation drift/restart, service cleanup and budget exhaustion.
Retain bounded independent kernel/manager observations, not worker assertions.
No source inspection or synthetic success substitutes for actual Linux execution.

All regressions NOT_RUN; no installed/runtime compatibility or cache sufficiency
claimed. Final record binds observed retained bytes to the admitted build closure,
not reproducible compilation, compiler semantic correctness, runtime attestation,
provider authority or general agent containment. Malicious build code may produce
arbitrary candidate bytes; retention never authorizes executing them. Owner
records explicitly retain all-descendant compiler/config enforcement and
source-to-binary semantic correctness as not_proved, rather than promoting source/
toolchain/initial-argv/config identity into universal compiler-use evidence. No
controller imports/evaluates target code; payloads and admitted inputs remain
immutable with no outside mutable aliases. Subsequent
driver admission and canonical `ota run check --native --agent` remain separate.
No public command/schema/JSON/contract term changes in this internal design;
Site/Skills/Examples/Learn/FAQ/Glossary need no new propagation here, broader
product propagation still OPEN. Design review is complete. Recommended next,
At the design checkpoint, NOT_GRANTED: ONE separately human-approved SOURCE-only increment in the three
proposed files and canonical plan/state; author portable tests without execution,
freeze for independent review then MUSE reconciliation. Installed Linux/build/
check/driver admission needs a later exact execution packet and approvals. No
source authoring or implementation in the current design grant; stop this slice.
Step7 OPEN/R1 BLOCKED, Step8/V12.2 inactive. No executors, probes, network,
installations, infrastructure, spending, commits, pushes or release.

**SOURCE R1 authority and coverage (supersedes the design-only write restriction).**
Bobai directly approved the three source files plus canonical plan/state, then MUSE
review; no builds or infrastructure. Human message
`01a12a91-091c-7b73-b590-bcf792164b2d`, turn
`01a12a91-08e2-7e60-b314-e8ebba33104f`, independently verified by MUSE.
Candidate at Core `8926ddb1fc0e4a225059c48375fc6e1185081b1e`:
`.github/ota/build_retained_driver.py`, `.github/ota/retained-driver-owner@.service`,
`.github/ota/test_build_retained_driver.py`. No Action/product/Protocol/Launcher edits.

Controller source implements strict canonical private request/record acquisition,
root-protected identity and single-use journal, full source/platform/toolchain/cache
tree inventories and admitted host interpreter/native closure, independently queried
manager and cgroup limits, posix_spawn stopped child/pidfd placement, principal/capability/
descriptor drop, fixed offline bwrap Cargo launch, recursive cleanup before child reap,
retained exact candidate copying and separate nonce-consuming manager postflight.
Writable Cargo geometry is deliberately limited to two declared lock files; missing
payload/lock/native-toolchain geometry refuses instead of hydrating or weakening inputs.
The strict frozen host-tree/no-link admission is not claimed compatible with an ordinary
distribution or rustup installation. Exact Linux cache/host geometry remains to qualify.
Owner observations are not worker success JSON; finalized still says usable_driver=false
and carries descendant compiler/config, semantic compilation and outer-postflight closure
limits. Root/operator concurrency and compromise remain excluded, not detected guarantees.

Test source covers canonical/strict request and budgets, immutable launch inputs versus
ambient wrapper flags, exact metadata/link modes, no-follow descriptor flags, journal
transition/refusal, protected write/fsync/seal/no-replace ordering and faults, effective
manager ceilings, stopped-child/generation/membership refusal, recursive cgroup cleanup,
and production finalization with stale/nonzero/restart/changed-artifact refusals. One
separately gated Linux openat2 descriptor case is authored. Full installed manager-owner
loss, nested descendants, namespace/FD/credential leakage, mount and cache geometry,
termination under resource exhaustion and artifact-copy fault coverage remain OPEN.
Do not substitute mocks or source review for that matrix, or call this execution-ready.

Qualified non-author VLAD read-only review followed by MUSE substantive reconciliation
is REQUIRED; SOURCE R1 was NOT_ACCEPTED pending findings. All tests/import/syntax/build/
runtime checks NOT_RUN. Only textual review, file identities and diff hygiene are allowed
under this slice. Artifact/metadata budgets remain proposed NOT_APPROVED; no toolchain
installation, worker/compiler/Ota execution, service activation, infrastructure, spending,
commit/push or release. No new public CLI/schema/JSON/contract term: Site/Skills/Examples/
Learn/FAQ/Glossary need no additional propagation for this private source candidate;
broader product propagation remains OPEN. Step7 OPEN/R1 BLOCKED; Step8/V12.2 inactive.

**SOURCE R1 review HOLD / SOURCE R2 corrective pass.** Qualified non-author VLAD
turn `01a12aa4-630d-7551-bfac-249051cc957e`, final
`msg_0074fa586a19efbe016acb6e938018819185c31c29abb214ca`, configured
gpt-6.1-sol/high as independently verified by MUSE, found one P1 and four P2s.
MUSE independently read all source and accepted those findings, adding two P2s.
Reviewed R1 controller `8408574c77e99e2941517014ab76ce9d05ab17eb79bf6211ab12ae60643a90b6`,
unit `f32c840ec871dabbfda4380e6c34da9557d013092c78d4d3302e00791cae6069`,
tests `98a3a981308357085360c9e1bf489bb0eb088fffcc777aff38cbe053934a60cb`,
plan `595c83df39e30a3b0e38f5f4d61b9f78412aed24f8d5e26b98c1ce04374fcdef`,
state `09f09b0971630fbc8774aa456496ff097aa236da7f0ade5cef36b0ba6e13f802`.
No source acceptance. MUSE permitted one R2 repair within the existing human grant,
same five-file write set; no new execution/installation/authority architecture.

R2 changes and source-authored production regressions:
- P1 scratch ancestor: private tmpfs root is root:root 0711 (search only), independently
  checked before spawn; nonce/work remains private to the dedicated worker. No widening
  of requests, journals, retained objects or cgroup controls.
- P2 input paths/links: explicit absolute-path component admission rejects dots/parents/
  empty components before opening; bounded complete symlink graph resolution rejects
  transitive escapes, cycles, dangling/unadmitted referents and non-directory traversal.
- P2 reviewed provenance: retain the bounded canonical request, raw complete inventories,
  approved HEAD/base/raw delta plus separate untracked path/content identities, and an
  inspected self-contained Git-layout record. Reject external .git/commondir/alternates/
  config includes and unsupported submodule layouts without invoking Git or target code.
  Root/operator approval binds the reviewed packet to the expected full source manifest;
  independent Git-delta reapplication remains not_proved, not semantic attestation.
- P2 candidate: nonblocking/no-follow acquisition, type/metadata rejection BEFORE output
  creation, with authored special-object/changing-object/copy/fsync failure cases.
- P2 logs: retain bounded actual raw stdout/stderr bytes, drain through terminal EOF
  after recursive cleanup, and retain failed/incomplete/truncated disposition on overflow
  or collection failure. Never treat exit status as EOF or prefix hash as complete logs.
- P2 completion: only ONE protected finalization-candidate carrier, explicitly provisional,
  plus independently observed successful same-generation postflight manager exit forms
  the consumption predicate. Failed second publication is removed by construction;
  rename-success/parent-fsync-failure leaves at most a candidate, not completed success.
  Tests exercise the production predicate against late publication failure/nonzero/live/
  stale manager observations. Finalized bytes alone cannot authorize driver execution.
  External postflight cgroup/resource-envelope closure remains a separate open gate;
  no new command/role, service, broker, global attestation platform or Action flow.

SOURCE R2 is NOT_ACCEPTED pending qualified VLAD delta recheck and substantive MUSE
reconciliation. Existing Action/HEAD/other tracked source delta must remain unchanged;
new untracked source files are separately hashed. All tests/import/syntax/build/runtime
checks NOT_RUN. Installed Linux owner-loss/descendant/namespace/resource fault matrix,
cache/toolchain/host and output-grammar qualification remain OPEN (including tmpfs size
and postflight duration serialization). Artifact/metadata budgets remain NOT_APPROVED.
No infrastructure, spending, installation, execution, commit/push or activation.

**SOURCE R2 HOLD / SOURCE R3 bounded corrections.** Qualified non-author VLAD
turn `01a12aba-13d5-76b2-9b8e-504527f4e086`, final
`msg_0074fa586a19efbe016acb73ec342081918cd4e532c048116d`, and MUSE substantive
review closed the six connected R1 repairs at SOURCE level, with no runtime claim.
R2 controller `deed7ee9810d577e123aab6ccc5c08368155720c717754cd58303e7d85fbfa13`,
unit `0243fe85b9e990de47122b8f897a37b340d706c3fd207a5ddef895f2165f5fad`,
tests `6f1b4b45ef980cc552786b1a2578fb353d309872432239c1bdce83184b1e172c`,
plan `11915f20b4bb33bdf0be9828238e9d73cf04b3e9e4388f1b3dbee9adf450ec31`,
state `ec1edee043653d0a1e68869e231c0c05e271fad20454d674efe0b2109dccdbbb`.
Two P2s remain distinct from prior retention repair: matching HEAD strings/objects
dictionary keys did not establish actual Git object closure; the default mocked
lifecycle test directly used Linux-only os.P_PIDFD. SOURCE R2 was NOT_ACCEPTED.

MUSE cleared ONE R3 source-only correction in the same five-file set. R3 admits only
real protected objects/refs directories and a bounded loose-object HEAD commit/tree/
blob/parent closure. Streaming stdlib zlib decoding checks strict type/length/framing,
legacy Git identifiers, compressed EOF/trailing data, decompression totals, visited/
active cycle detection, work queue/depth and the same deadline. Per-object payload
<=8 MiB, aggregate decompression <=512 MiB, graph/queue/depth <=100,000 within existing
source/entry bounds. Unsupported packed payloads, tags/submodules/other encodings,
missing/corrupt/wrong-type/dangling/cyclic/exhausted inputs refuse; an empty pack
directory is not itself a packed payload. The retained bounded closure summary plus
complete raw inventory binds the validated HEAD closure, not every Git ref or full
Git semantic correctness. SHA1 is ONLY legacy format consistency; root/operator
approval and SHA256 full raw inventories remain the authority anchor. Independent
delta reapplication and source-to-binary semantics remain not_proved.

Authored production-validator cases cover minimal commit/tree/blob closure, actual
compression/framing/hash/length/EOF, corrupt/trailing/incomplete streams, directory
kinds, empty/missing/dangling/wrong-type/cyclic objects, decompression/work exhaustion,
and unsupported versus empty pack directories. Default lifecycle/finalization mock
interfaces explicitly provide required Linux constants/methods; Linux primitives
remain separately gated. All source/test/import/syntax/runtime checks NOT_RUN.
SOURCE R3 is ACCEPTED after qualified non-author VLAD read-only recheck and substantive
MUSE reconciliation; no execution, install, infrastructure, spend, commit/push or activation.
Installed loose-object packet preparation/sufficiency, host/field-grammar/budget/resource/
namespace/emergency-cleanup and outer-postflight proof gates remain OPEN; no expanded
16MiB metadata allowance or new operational approval. Existing Action/HEAD/unrelated
source delta unchanged; new source files separately hashed. Step7 OPEN/R1 BLOCKED;
Step8/V12.2 inactive; broader public propagation still OPEN.

**SOURCE R3 acceptance / closure-only record.** Qualified non-author VLAD turn
`01a12ac9-3006-79e3-9b6d-7b595adc7719`, final
`msg_0074fa586a19efbe016acb7768c5808191965630b04ce9370c`, and MUSE turn
`01a12ac9-33b5-78e1-a56c-b9a274f53d66`, final
`msg_0074fa586a19efbe016acb78f927188191bea57dd8a2cac125`, independently reconciled
the bounded Git closure and portable mocks. Both R2 P2s are closed at SOURCE level;
no remaining P1/P2/P3 in this bounded review. MUSE verified VLAD's persisted
gpt-6.1-sol/high configuration, not provider attestation. Reviewed frozen R3 hashes:
controller `0ec68b7dbd6a2960da57bf814ad08acd4d70f5a0a8ec44be729506524b338a13`,
unit `0243fe85b9e990de47122b8f897a37b340d706c3fd207a5ddef895f2165f5fad`,
tests `2088547d681cc47ecf737616874b632bfdc092eb5fba6eed5ef200a177a40834`,
pre-closure plan `dfd46805d287177ae8c3edb658b02614ab66eef4d384d371b151a8b727489bdf`,
pre-closure state `70bbb715db3bc8b1b3b7afc640fbc5fbc1421c7a6425b70ab5e4c57cb487992a`,
unchanged Action `bb2a3a3d286264c210dcc4a66f1c3e46f6c0bb7321c396598672a5e676dbd214`.
HEAD remains `8926ddb1fc0e4a225059c48375fc6e1185081b1e`; tracked binary/full-index
source delta excluding only plan/state remains
`9c026a18a8fd5c4eeef775bb363796ca82ee57809b880b0ceeea116b7275c681`.
The three new source files are separately hashed, not included in that tracked delta.
This factual acceptance update changes only plan/state after the frozen review;
source, unit, tests and Action stay unchanged. The approved source implementation
slice is complete. Validation HOLD: all imports/syntax/tests/builds/primitives remain
NOT_RUN; installed proof, loose-object packet preparation, platform/field grammar,
toolchain/cache sufficiency, budget approval, aggregate resource accounting and
outer-postflight closure remain OPEN. No driver-execution grant; stop after closure.

**Fresh Python-only validation preparation / execution HOLD.** Bobai requested
bounded validation after MUSE discussion in turn
`01a12b8f-b0a2-7bf0-a36a-ee5204dcc607`, human
`01a12b8f-b0e6-7ea1-bd66-88e5a37b3fe0`. MUSE independently verified that request
and selected ONE fresh cached-Linux packet preparation, not revival of the closed
kernel experiment or native Darwin admission. Exact packet/parent-controls and
qualified non-author VLAD/MUSE reconciliation must precede target execution.
No new guard/framework or weaker control is silently admitted.

The intended selection is exactly 41 authored methods in InputTests,
JournalTests, LifecycleTests, FinalizationTests, CandidateTests, LogTests,
GitLayoutTests and LooseGitTests. Both LinuxDescriptorTests are excluded entirely;
no discovery or descriptor opt-in. Compile syntax only in memory, import only
inside containment, and inspect the unit as trusted text, never load it. Mocked
production finalize_owner calls are permitted; operational controller main/roles,
unmocked effects, descriptors, kernel/manager/root actions and Cargo remain excluded.
The source-derived count is not an executed test result.

Required packet controls: installed pinned local runtime and exact cached image
only, no pull/build/install/daemon change/network/cloud/spend; nonroot, readonly,
no host home/credentials/sockets, explicit clean environment, no bytecode or
payload subprocess/exec. Total 30-second creation/test/cleanup deadline with the
last 5 seconds reserved for cleanup, fixed-payload CPU <=10 seconds (not an
unexplained per-process aggregate claim), aggregate memory <=512 MiB including
scratch, swap0, one CPU, <=64 FDs, <=64 KiB aggregate output and at most8 MiB
bounded scratch. PID cap alone is not subprocess denial. Freeze effective
configuration/identities and independent owned-container/staging/attachment
absence predicates; absent enforcement stops. Failure retains diagnostics and
stops without source correction, retry or continuation.

Concrete existing-route incompatibility: retained
`/Users/bobai/.codex/artifacts/2026-10-10/kernel-custody-preflight-r11/python-preflight-r1.py`
SHA256 `f128f0226cb2cd2a0a2c3a1eff97c33ab536e8c02d810d18793767b2233dda4b`
matches its historical review, but hard-codes the old26-member packet/four staged
inputs, old test entrypoint and exact11-test report. Its 65,536-byte member limit
rejects this controller's 73,923 bytes (test source39,182 bytes). It uses32 MiB
scratch, no10-second CPU rlimit, independent30-second control-call timeouts and a
30-second capture deadline starting after creation, not the required total
deadline. It is not a supported parameterized interface. The narrowly checked
later outer-r13-r3 and admission-proposal-r13-r2 records offer cloud control or
planning only, not a compatible ordinary Python launcher. No daemon/image probe
was made because interface incompatibility already prevents an exact runnable
packet. Historical image/runtime observations are not current qualification.
Required next decision: explicitly clear a narrow source-only binding/limit
adaptation of the reviewed local route, or retain this concrete HOLD; do not
silently rewrite the guard or execute another route. No new controller/unit/test
edits. Source/Action/HEAD/tracked delta identities remain unchanged; only canonical
plan/state records change. Evidence ceiling is contained Linux Python/mock behavior,
not installed authority, manager lifecycle, artifact-budget qualification, build
or driver admission. Step7 OPEN/R1 BLOCKED; Step8/V12.2 inactive. This private
validation preparation adds no public CLI/schema/JSON/term; no Site/Skills/Examples/
Learn/FAQ/Glossary change is required for it, while broader product propagation
remains OPEN.

**Fixed-purpose validation packet R1 / SOURCE PREPARED.** MUSE explicitly amended
its unchanged-interface constraint and cleared one private source-only adaptation;
no additional founder question was required for that preparation. Historical R11
and all accepted candidate inputs remain unchanged. Packet directory:
`/Users/bobai/.codex/artifacts/2026-10-11/retained-controller-python-validation-r1`.
New parent.py SHA256
`f69a1e572b48932b07fc4a401ff99e30debe5ababa8e8907383005d73f98e9e1`,
bootstrap.py SHA256
`80430be7f07675a860d757168427de138ef55f06ff657eed17e4f5349fb25952`.
Trusted parent AST parsing only passed; neither new control nor target code ran.
Both new files are owner-matched singular regular0444 source, not installed code.

Exact proposed operator invocation is
`/usr/local/Cellar/python@3.13/3.13.3/bin/python3 -I -B -S <packet>/parent.py`;
trusted interpreter SHA256
`bab9c542210bd094e79ffd88399bf6c31c426e68487469e9172923b4b765432b`.
Docker CLI fixed absolute
`/Applications/OrbStack.app/Contents/MacOS/xbin/docker`, SHA256
`70d5a06adf388bd09b652f47ddc9ca31663a3e5353d24188bdb0054b3014ab3e`,
endpoint `unix:///Users/bobai/.orbstack/run/docker.sock`; clean control environment
PATH=/usr/bin:/bin, HOME=/Users/bobai, LC_ALL=C, DOCKER_HOST=that exact endpoint.
Read-only local image inspection currently confirms cached image
`sha256:934240a162082fd8b8a2f90cd5114446443f1eba1c5378f6687167ca405e6584`,
Linux/arm64; engineID `ef049779-8946-48a9-a0a4-3d7e83c24e52`, version29.4.0,
cgroup2, advertised memory/swap/PID/CFS support true. This is fact qualification,
not actual container/control proof. No daemon mutation, pull or external facts.

Reuse descriptor-stable SHA256-bound acquisition, exclusive UUID/name/label,
sealed0600-to0444 records, readonly staged copies and immutable container identity.
Input bound128KiB/member/256KiB total; source/test only plus trusted bootstrap
are mounted at /experiment; unit stays parent-only for exact text assertions.
Child entrypoint `/usr/bin/env`, argv `-i PATH=/usr/bin:/bin LC_ALL=C HOME=/tmp
/usr/bin/python3 -I -B -S /experiment/bootstrap.py`. No discovery: the bootstrap
freezes all41 exact IDs in the eight approved classes; LinuxDescriptorTests absent.
No bytecode, writable volume or tmpfs; readonly root/input, networknone, IPCnone,
private PID/cgroup, nonroot65534:65534, capabilities dropped, no-new-privileges,
no restart/healthcheck/TTY/stdin/devices/ports or host credential/home/socket mounts.
One PID/cgroup and NPROC1 prevent candidate-created processes/threads; CPU10s is
the fixed single payload's rlimit, separately from one-CPU cgroup quota. Payload
exec tripwires are Python-level accidental-effect checks, not kernel exec denial
or protection against arbitrary native code. Stubs exclude operational role/main/
command/syscall dispatch unless an authored test installs its own mock. The known
libc symbol resolution at import is not a syscall invocation. Containment remains
the OS boundary; installed/native-code authority is not claimed.

Parent inspects exact flags and bootstrap checks actual cgroup/rlimit/mount/drop
controls before target imports. Bounds:512MiB/swap0,64FDs,core0/file-size0,no
scratch, shared64KiB control+test capture (16KiB reserved for cleanup and512B
for the final parent summary). Whole-attempt30s with last5s reserved
for stop/removal/absence; no independent30s-per-control extension. Test failfast,
exact41-completion marker and terminal/noOOM status are necessary but insufficient:
owned container deletion/ID+UUIDlabel+anchoredname absence, attachment teardown,
identity-bound staging removal and retained raw log/result must also succeed.
ParentSIGKILL remains not_proved; deadline/cleanup ambiguity cannot be accepted.
On failure STOP with bounded diagnostics, no retry/repair. Qualified non-author
VLAD review of the whole changed control path, then substantive MUSE reconciliation
and fresh single-run clearance remain REQUIRED. This is NOT execution-ready.

**Validation SOURCE R1 HOLD / R2 successor.** Qualified non-author VLAD
`01a12bc5-80b7-7e53-9489-d2597489b4a3`, final
`msg_0074fa586a19efbe016acbb8b9ab988191b47f14d54091cfe7`, and substantive MUSE
independently found three P2s in R1: poll/wait could lose the leader anchor before
group teardown; a positive deadline verdict excluded result publication/terminal
completion; unguarded retention faults could lose diagnostics or leave a positive
looking carrier. VLAD's persisted gpt-6.1-sol/high configuration was independently
verified, not provider-attested. R1 source is immutable and remains NOT_ACCEPTED.
MUSE cleared ONE narrow source-only R2 correction, not target failure repair/retry.

Successor directory
`/Users/bobai/.codex/artifacts/2026-10-11/retained-controller-python-validation-r2`:
parent.py SHA256 `4fde43a18207626840469db4d095394fb4b3587fdc8c0e58d3c2a527463f974f`;
bootstrap unchanged `80430be7f07675a860d757168427de138ef55f06ff657eed17e4f5349fb25952`.
No changed selection/target/input/container/resource behavior. Trusted parent AST
only parsed; neither successor control nor target ran. R2 signals the owned group
before any poll/reap, while its unreaped session leader anchors the PGID; then
waits for actual child status and requires group ESRCH within the existing budget.
After reaping it probes only, never destructively signals a potentially reused
group. All control/attachment children must complete these checks. Escaping control
descendants and parentSIGKILL remain explicitly not_proved, not general custody.
Log sync/seal/close and result publication have sticky bounded failure handling;
failure exits nonzero without unbounded traceback or a second positive carrier.
R2 capture reserves1024B for parent/timer output (within64KiB combined), retaining
the16KiB cleanup reserve. Artifact/result fields are always provisional,
overall_passedfalse and usable_driverfalse. Visible candidate_passed cannot grant GO.

Fixed trusted operator invocation for later clearance:
`/usr/bin/env -i PATH=/usr/bin:/bin HOME=/Users/bobai LC_ALL=C /usr/bin/time -p
/usr/local/Cellar/python@3.13/3.13.3/bin/python3 -I -B -S <R2 packet>/parent.py`.
Timing binary SHA256
`6fe69a2da5af0a6e965785f8fbb03901c5bb8eef7f308f70620d4458ed28cce7`;
trusted Python/CLI/image/engine identities remain as recorded for R1. The operator
must independently acquire an actual terminal exit0 (not a running session), a
single anchored `real <decimal>` line from this trusted timing invocation and
whole elapsed <=30s including imports, acquisition, creation, tests, cleanup,
retention publication and parent completion. Require parsed real<=29.90s as a
conservative timing-format margin AND trusted tool whole wall<=30s, not the
candidate's prepublication timestamp. Require sealed regular single-link0444
owner-matched attempt/log/result with independently acquired hashes, valid
provisional candidate_passedtrue, overall_passedfalse/usablefalse, exact test
completion and cleanup facts, plus separate exact ID/UUIDlabel/anchoredname,
owned group and stage absence observations. Timing/report/cleanup/retention
ambiguity is HOLD; no retry or continuation. Observe combined log+outer output
<=64KiB, otherwise reject. No record alone is accepted. Qualified VLAD R2 whole
repair review then MUSE substantive reconciliation/single-run clearance remain
REQUIRED; all target validation NOT_RUN. No source/candidate/contract/public change
or infrastructure/build/install/spend/commit/push permission is expanded.

**R2 original findings closed / R3 wait-disposition repair.** Qualified non-author
VLAD `01a12bd3-534f-7922-80d3-94e350b26b6b`, final
`msg_0074fa586a19efbe016acbbb9dd2a48191bf9e1a1644cdb5b5`, and substantive MUSE
close all three original R1 P2s at SOURCE level. MUSE additionally identified
inherited SIGCHLD ignore/autoreap/SA_NOCLDWAIT as an admission P2: returncodeNone
alone cannot anchor an unreaped OS child if child waiting is disabled. The pinned
Python subprocess implementation can synthesize sts0 on ChildProcessError; no
host failure/escape was executed or observed. MUSE cleared ONE source-only R3 fix.
R1/R2 stay immutable; R3 directory
`/Users/bobai/.codex/artifacts/2026-10-11/retained-controller-python-validation-r3`,
parent SHA256 `a6f4b48591a02b8916c0d971ae35ffc9a7b0e39e7d4d9c83eb91139b23109394`;
bootstrap unchanged80430be7f07675a860d757168427de138ef55f06ff657eed17e4f5349fb25952.
The ordinary signal API actively establishes SIGCHLD=SIG_DFL and verifies it
before ANY Docker/control/attachment Popen; this single-threaded fixed parent has
one synchronous reaper, no competing SIGCHLD handler or target imports. No FFI,
signal/capability experiment, new framework or candidate change. Only packet root
and this directly connected admission fix differ from R2; identical suite, image,
budgets, provisional evidence and external acceptance. Exact timed operator command
uses the R3 packet path instead of R2. Trusted parent AST parsing only passed.
Qualified VLAD narrow independent assessment/recheck
`01a12bd9-b4da-75d2-8bf3-4dd665f5e792`, final
`msg_0074fa586a19efbe016acbbd1295a0819190b9d814d8deb36d`, and MUSE substantive
reconciliation (turn `01a12bd3-6621-7e50-8a0c-7797e3c4c0a9`) ACCEPT SOURCE R3;
no remaining P1/P2/P3 in this bounded packet. Exact persisted non-author
gpt-6.1-sol/high reviewer configuration independently checked, not attested.
MUSE gives SINGLE-RUN GO under fresh human request
`01a12b8f-b0e6-7ea1-bd66-88e5a37b3fe0`: only the exact timed R3 command above
with mandatory cleanup, independent sealed evidence/time/terminal/absence checks,
qualified VLAD runtime review and MUSE final reconciliation before gate acceptance.
All frozen controls/runtime/input/Action/HEAD/delta identities rehashed before
launch, unchanged. The single attempt subsequently failed and is consumed as
recorded below; no retry/repair/alternate route. This grant does not authorize
builds, actual controller/root/manager/descriptors/provider/VM/infra/spend/install,
driver execution, commit/push/release or prior experiment renewal.

**Validation R3 single attempt FAILED / STOP.** Exact cleared timed invocation
ran ONCE. Trusted exec chunk22bf0d ended exit1, wall1.426203083 seconds, no running
session; trusted timing output real1.52/user0.33/sys0.51. The parent retained
PermissionError/EPERM after the first read-only cached image-inspect query, during
owned process-group handling. Exact syscall/root cause is NOT_PROVED: no syscall
trace or exact child PID/birth identity was retained. The1456-byte log contains
only image-inspect JSON; no staging/create/bootstrap/control qualification,
candidate syntax/import or target test markers. All41 selected methods NOT_RUN,
not failed candidate tests. No validation gate is accepted.

Attempt nonce `5e837cb2fdaf4c5d8ebe9bba2786a14e`; name
`ota-retained-python-5e837cb2fdaf4c5d8ebe9bba2786a14e`; label
`ota.retained.python.attempt=5e837cb2fdaf4c5d8ebe9bba2786a14e`.
All retained files are bobai/staff singular regular0444 in the frozen R3 packet:
- attempt.json1232 bytes, SHA256
  `47d32a0cbb36d2b8c827f9f358a28444d76424ae0dcb0dcd923dec510643bbcb`;
- result.json1217 bytes, SHA256
  `26291c0fbedf7010a4e069eedccd0ca05a7b8caf17fb3680fc916cb39ba65d0e`;
- validation.log1456 bytes, SHA256
  `d0d5e591798c7b0a9f9edb8ebad620bd8dd6070c65a07964ab28f870f0379602`;
- outer-tool-result.json340 bytes, SHA256
  `566d1f179723432fee8763ff6def6a7d0040c5667e336d6f9d0966298a2db053`.
The last file is the exact copied exec response, not independent provider attestation.
Result remains provisional/overall_passedfalse/usable_driverfalse;
candidate_passedfalse/tests_passedfalse, container_idnull,
attachment_cleanup_okfalse and attachment_errorEPERM. Never rewrite this failed
carrier as authoritative or infer success from elapsed time alone.

Independent read-only current absence checks on the pinned local socket returned
exit0 with empty Docker `ps -a --no-trunc` results for the exact UUID label and
anchored name. Stage `staged-5e837cb2fdaf4c5d8ebe9bba2786a14e` has no entry or
symlink; an exact Docker CLI/image-inspect matching process query is empty.
These are current matching absence, NOT exact owned-group teardown: no saved
PID/birth/group closure exists, and the parent explicitly reported attachment
cleanup false. No unrelated containers were touched. No container or staging
was created; no VM/cloud resources/build/install/spend occurred.

Post-failure rehash confirms parent/bootstrap, three candidate inputs and Action
unchanged, HEAD `8926ddb1fc0e4a225059c48375fc6e1185081b1e`, tracked delta excluding
the two canonical records
`9c026a18a8fd5c4eeef775bb363796ca82ee57809b880b0ceeea116b7275c681`.
Qualified non-author VLAD runtime-failure review
`01a12be3-63fd-79b3-a0b6-f1fb13800dce`, final
`msg_0074fa586a19efbe016acbbfb97ae88191be7b73a8a65fdfe2`, identifies P2 at
parent.py305-314: control-child identity/status is retained only after all cleanup
succeeds, so faulting group cleanup loses the evidence needed to rederive the
exact failing phase and historical group terminality. This P2 remains OPEN;
no repair or root-cause conclusion is claimed. Frozen canonical closure recheck
`01a12be5-7b68-7bb1-9321-ca529627a2aa`, final
`msg_0074fa586a19efbe016acbc005517c8191889d879f2a263598`, finds no new P1/P2/P3
in the failure wording. Both exact persisted gpt-6.1-sol/high non-author reviewer
configurations were independently checked, not provider-attested.
MUSE substantively reconciled both reviews, immutable packet/evidence/input/HEAD/
delta identities and sealed record ownership/modes, and independently repeated
only current exact label/name container and stage absence checks. ACCEPT factual
FAILED/STOP closure only; source R3 acceptance unchanged, validation gate HOLD.
Recommended NEXT, not authorized: a separate SOURCE-only diagnostic successor
retaining child identity at launch and exact bounded signal/wait/cleanup phase
and observable status before faults, preserving unavailable birth evidence as
not_proved. No EPERM suppression, signal/cleanup semantic change, platform probe
or retry. Any source increment requires separate clearance outside this consumed
grant. Only the factual review stamp remains in scope. No source repair, retry, alternate route, build, new
experiment, infrastructure, commit/push or continuation. Step7 OPEN/R1 BLOCKED;
Step8/V12.2 inactive. Public propagation assessment unchanged: this private failed
validation adds no public CLI/schema/JSON/vocabulary or shipped behavior, so no
Site/Skills/Examples/Learn/FAQ/Glossary change is required; broader propagation OPEN.

**Validation diagnostic R4 SOURCE ACCEPTED / validation HOLD.** Bobai directly approved
MUSE question `call_ebe4618de2734a6d965170ba7345a909`, answer
`Approve source-only diagnostics (Recommended)`, human
`01a12c05-0f13-7a40-8bb1-31e4eed3618d`, turn
`01a12c05-0edf-7930-933a-c1d2d6232adb`; SCOOBY independently read the exact reply.
MUSE delegates ONE private diagnostic successor plus canonical records and
source-authored diagnostic cases only. This new SOURCE grant does not revive the
consumed R3 runtime grant. R3 and its failed evidence remain immutable.
Directory `/Users/bobai/.codex/artifacts/2026-10-11/retained-controller-python-validation-r4`:
- parent.py23849 bytes, SHA256
  `60cfe8e81900b2c89677ebf367b996aee22960bcccf0e5d25c6d3be3adae952a`;
- bootstrap.py7822 bytes, unchanged SHA256
  `80430be7f07675a860d757168427de138ef55f06ff657eed17e4f5349fb25952`;
- test_parent_diagnostics.py10208 bytes, SHA256
  `d2521a8e78889055fcc1a712b20b4198f9c6e7fd0346fedd69b33ee9fb700e76`.
All are singular regular owner-matched0444 source. R3->R4 parent unified diff
SHA256 `5dace6c65fa7ac76ff021c73b0dc7625aff62aaed9efc06393edc6fc25beafd2`
is content-bound to both parent identities above (diff includes file timestamp
headers, not semantic identity). No candidate controller/unit/test/Action changes.

R4 records each owned Popen role/ordinal/PID and provenance promptly after launch,
before collection/cleanup. No poll, native birth observer or platform discovery;
cached returncode is recorded, birth_identity remains explicitly not_proved.
Record phase/operation precedes existing killpg(SIGKILL), wait, killpg(0), group
deadline/sleep and stdout-close calls. Existing syscall ordering, wait timeouts,
ProcessLookupError interpretation and exception propagation remain unchanged.
Primary, collection-cleanup, close and final-cleanup error snapshots are separate
bounded fields; later cleanup cannot overwrite the primary cause. No error-message
payload or arbitrary argv is added; type name is bounded64 characters. A record's
outcome describes its last operation only, not whole-child success. Diagnostics
are linked into result immediately, not after all teardown succeeds. Missing birth
evidence and any failed group cleanup remain NOT_PROVED/fail-closed.

MUSE explicitly cleared minimally lifting the existing finish_group body to module
scope for direct production-function mock source cases, not a new wrapper/framework.
Fixed maximum launch graph is13: image/info/exclusive-name/create/pre-inspect (5),
attachment and post-inspect (2), cleanup label lookup/owned inspect/removal (3),
three exact absence queries (3). Any failure shortens this sequence; final group
cleanup creates no child. Capacity16 covers this graph; an exhausted capacity
refuses before Popen. Four fixed failure buckets per child bound metadata growth;
serialized child diagnostics must fit32768 bytes before result sealing, otherwise
retention fails sticky/nonzero. Existing capture65536-byte bound, cleanup reserve,
30-second whole-attempt budget, provisional/overallfalse/usablefalse, safe sealing,
container/bootstrap/test selection and candidate acceptance predicates unchanged.
No EPERM suppression or alternate signal/cleanup path.

Six SOURCE-authored mock cases cover primary signal failure, later collection and
final cleanup failure without replacing primary, partial diagnostic serialization
without success flags, unchanged signal/wait/probe ordering/status, distinct wait
failure, and production main's sealed provisional result under three injected group
failures. All six are NOT_RUN; authoring mocks is not an observed runtime result.
Trusted installed Python performed AST-only parsing of the three source files,
without importing/executing parent/bootstrap/tests/candidate. No signals, processes,
platform probes, container, build, infrastructure or spend. Qualified non-author
VLAD frozen source review `01a12c0d-d66f-7de2-8883-48b8749e99d1`, final
`msg_0074fa586a19efbe016acbcac2aa908191a38166083d981289`, and historical execution
wording P3 recheck `01a12c10-a661-72b3-8d3a-7d94691c3eae`, final
`msg_0074fa586a19efbe016acbcafa3ef88191ba45a18d255526d4`, are substantively
reconciled by MUSE. Exact persisted gpt-6.1-sol/high non-author configurations were
independently checked, not provider-attested. MUSE independently rederived13 launch
maximum/capacity16 and inspected the full connected launch/collection/group/final
cleanup/sealing paths and six source mock cases. Source/retained-diff/bootstrap,
historical R3/evidence/candidate/Action/HEAD/non-record delta identities matched.
Accepted pre-stamp canonical identities: state
`186d3858ab9bac5b7b4ec1ce806c7cfd901c58244b0ae0784f169afd21b64f3f`, plan
`1b2403751a11bfbff274b5f376eda95610a33bc139344b167b6029d7c6e9cbc0`.
R4 SOURCE ACCEPTED; no remaining P1/P2/P3 in this bounded diagnostic-only slice.
Prior missing-child-diagnostics P2 is CLOSED AT SOURCE ONLY, not runtime repair,
native birth attestation, EPERM root cause or historical group teardown proof.
All six diagnostic tests and41 candidate methods remain NOT_RUN. R4 parent and
all bootstrap/candidate imports/execution remain NOT_RUN. The current source grant
is complete; no next-gate preparation or new execution artifacts authorized.
Validation remains HOLD; no new runtime grant requested or assumed. Step7 OPEN/R1
BLOCKED; Step8/V12.2 inactive. This private diagnostic increment adds no public CLI/
schema/JSON/term or shipped behavior, so Site/Skills/Examples/Learn/FAQ/Glossary and
public docs/changelog need no change for this slice; broader product propagation OPEN.
No import/test execution, retry, build/install, manager/descriptor/provider/VM/infra,
commit/push/release or new runtime grant. Stop on material scope changes.

**R4 diagnostic / 41-mock PROPOSAL ACCEPTED ONLY / NOT_RUN / pending HUMAN RUN APPROVAL.**
Fresh human discussion request in MUSE chat, human
`01a12c2d-bbaa-7113-b68a-17ef6222f9ab`, turn
`01a12c2d-bb89-7f02-a2d4-91d8b4a36665`: discuss exact bounded R4 validation, no
builds or infrastructure. SCOOBY independently verified it. This entry authorizes
no launch, runtime probe, new executable artifact or revival of consumed R3.
Recommendation: ONE unchanged frozen R4 parent invocation. Its first failing
existing control operation can now yield bounded child/phase/error diagnostics;
if the route completes, it exercises only the unchanged41 candidate mock methods.
Do not call a diagnostic failure a passed candidate gate. A smaller standalone
signal probe would introduce a new path and does not validate retained production
failure wiring; static inspection cannot establish actual EPERM operation.

Exact proposed argv (no added shell pipeline, extra flags, discovery or fallback):
`/usr/bin/env -i PATH=/usr/bin:/bin HOME=/Users/bobai LC_ALL=C /usr/bin/time -p
/usr/local/Cellar/python@3.13/3.13.3/bin/python3 -I -B -S
/Users/bobai/.codex/artifacts/2026-10-11/retained-controller-python-validation-r4/parent.py`.
Trusted operator uses one30000ms exec wait; a running session is NOT a completed
attempt or deadline proof. Require actual terminal status and independent whole
tool wall<=30s plus exactly one anchored timing real<=29.90s, including publication
and parent exit. This is an acceptance ceiling, not a claim that blocked syscalls
or parentSIGKILL cleanup are independently guaranteed. Late/ambiguous/missing
terminal or artifacts means HOLD/STOP, never a longer test budget.

Frozen identities rechecked during proposal preparation, without execution:
- Core HEAD `8926ddb1fc0e4a225059c48375fc6e1185081b1e`;
- tracked delta excluding canonical plan/state
  `9c026a18a8fd5c4eeef775bb363796ca82ee57809b880b0ceeea116b7275c681`;
- R4 parent `60cfe8e81900b2c89677ebf367b996aee22960bcccf0e5d25c6d3be3adae952a`;
- unchanged bootstrap `80430be7f07675a860d757168427de138ef55f06ff657eed17e4f5349fb25952`;
- NOT-selected diagnostic tests
  `d2521a8e78889055fcc1a712b20b4198f9c6e7fd0346fedd69b33ee9fb700e76`;
- trusted `/usr/bin/time` `6fe69a2da5af0a6e965785f8fbb03901c5bb8eef7f308f70620d4458ed28cce7`;
- trusted installed Python `bab9c542210bd094e79ffd88399bf6c31c426e68487469e9172923b4b765432b`;
- fixed Docker CLI `70d5a06adf388bd09b652f47ddc9ca31663a3e5353d24188bdb0054b3014ab3e`;
- controller input `0ec68b7dbd6a2960da57bf814ad08acd4d70f5a0a8ec44be729506524b338a13`;
- candidate tests input `2088547d681cc47ecf737616874b632bfdc092eb5fba6eed5ef200a177a40834`;
- parent-only unit input `0243fe85b9e990de47122b8f897a37b340d706c3fd207a5ddef895f2165f5fad`;
- unchanged Action `bb2a3a3d286264c210dcc4a66f1c3e46f6c0bb7321c396598672a5e676dbd214`.
Cached Linux/arm64 image
`sha256:934240a162082fd8b8a2f90cd5114446443f1eba1c5378f6687167ca405e6584`,
CLI `/Applications/OrbStack.app/Contents/MacOS/xbin/docker`, endpoint
`unix:///Users/bobai/.orbstack/run/docker.sock`, expected engine
`ef049779-8946-48a9-a0a4-3d7e83c24e52`/29.4.0/cgroup2. No new image/daemon
qualification was probed: the frozen parent does its existing read-only checks
inside the one attempt. Drift refuses, no refresh/install/fallback. Rehash frozen
inputs/runtimes/HEAD/delta immediately before any separately cleared launch.

Controls unchanged: host nonroot trusted parent, SIGCHLD default/one controlled
reaper, isolated Python and clean environment;25s work/5s cleanup reserve within
30s total. At most one owned temporary local containment container, no engine
changes or unrelated container operations. Payload nonroot65534:65534, clean
PATH/LC_ALL/HOME, `/usr/bin/env -i ... /usr/bin/python3 -I -B -S
/experiment/bootstrap.py`, cached image only/no pull, networknone/IPCnone/private
PID+cgroup, readonly root and sole readonly staged-input mount, no host sockets/
credentials/home or writable tmpfs/volumes, capdropALL/no-new-privileges, pids1/
nproc1,512MiB memory/swap0,one CPU/10s CPU rlimit,64FDs/core0/fsize0, no restart/
init/healthcheck/TTY/stdin/devices/ports. Existing parent/bootstrap inspect actual
controls before target import; failures refuse. Acquisition128KiB/member/256KiB
total; unit is text-only and never loaded. Selection exactly41 IDs in the frozen
bootstrap's eight classes, both LinuxDescriptorTests omitted. No controller main/
operational roles, native descriptors/syscalls or build. Python tripwires/mocks
are accidental-effect checks, not an arbitrary-native-code security boundary.

Evidence bounds: combined captured log plus raw outer response<=65536 bytes,
including existing16KiB cleanup and1024-byte parent/timer reserves; child-diagnostic
array<=32768 bytes/16 child records, not a bound on all result metadata. Preserve
exclusive attempt/log/result and original R3 history. After terminal, acquire
exact outer response/exit/timing, owner-matched regular single-link0444 retained
attempt/log/result, lengths/hashes/stable identities and partial child diagnostics.
Retained diagnostics include owned Popen PID/role/ordinal, cached status and exact
recorded operation/failure buckets; native birth identity remains not_proved.
No trace/proc/birth observer or platform/signal experiment. A recorded failure
phase identifies an observed failing operation, not its OS root cause.

Outcome A: launcher failure. Retain nonzero/failed/partial evidence and perform
only unchanged owned cleanup. Require evidence before classifying any diagnostic
observation as complete; missing/oversized/unsealed/ambiguous retention is HOLD.
All41 candidate methods may remain NOT_RUN; completed prefixes, if any, are not
full-suite acceptance. Current exact owned ID/UUID-label/anchored-name/stage absence
is recorded only for acquired identities. No guessed-PID/PGID signals or inference
of historical group teardown from matching absence. Native birth or unresolved
group closure stays NOT_PROVED. Diagnostic observation ONLY, validation gate HOLD.

Outcome B: completed contained mock gate. Require terminal exit0, time predicates,
actual guards/syntax/exact selection markers and all41 completed with no skips or
failures, exited/noOOM container state, sealed retention, unchanged inputs, actual
parent-owned group cleanup and independent exact owned ID/label/name/stage absence.
Result MUST remain provisional/overall_passedfalse/usable_driverfalse; record alone
never grants acceptance. Qualified non-author VLAD runtime-evidence review then
MUSE substantive reconciliation are required before accepting this contained gate.
This is not root-installed manager/descriptor custody, actual build, driver admission,
provider evidence, Step7 completion or a generally safe controller. Step7 OPEN/R1
BLOCKED; Step8/V12.2 inactive; all stronger installed/authority proofs remain open.

The six parent-diagnostic mock cases are a distinct NOT_RUN surface. Frozen R4
does not stage them, and its bootstrap admits only the two candidate source modules
and exact41 suite. Testing them later requires a separately cleared/source-reviewed
fixed-purpose binding of parent+diagnostic source, hash/selection and contained
prelude to an admitted route; cannot simply run unittest/import on the host or
add six cases to this proposal. No such adaptation, launcher or executable artifact
is prepared now. Do not claim this attempt executes or certifies those six cases.

ONE attempt only; any failure/unsupported/late/ambiguous condition retains bounded
diagnostics/owned cleanup and stops, with no automatic repair/retry/alternate route.
No new execution artifacts, source changes, gratuitous preflights, pulls/install,
engine changes, builds, manager/provider/VM/cloud/infra/spend/commit/push/release.
Only this canonical proposal entry is authored. Qualified non-author VLAD proposal
review `01a12c33-da38-7922-9dcf-53bb51b9e49c`, final
`msg_0074fa586a19efbe016acbd453c93081918f232a299d06bdda`, and MUSE substantive
reconciliation ACCEPT THE PROPOSAL ONLY with no P1/P2/P3. Exact persisted configured
gpt-6.1-sol/high reviewer setting independently checked, not provider-attested.
MUSE rechecked exact argv, unchanged controls/selection, separate captured-output
and child-array bounds, terminal/time ceiling, both outcomes and excluded six
diagnostic cases. Accepted pre-stamp identities: state
`b267695a3b0668b0dab5e7f9a7efe30c24c2a24d3d46e2552fb78f475794d5c2`, plan
`e2f4da53e831ee7be65f5f0ca5899a17569bee6d6eba2cec9b9371e8bc930149`; frozen
R4/source inputs/runtime binaries/Action/HEAD/delta match. No execution clearance.
MUSE owns one direct human action question and will relay exact single-run
clearance only after approval. Dispatch consumes that grant even if attempt.json
cannot be written; missing exclusive carrier never permits a rerun. Unsupported,
failure, late or ambiguous condition retains bounded diagnostics and owned cleanup
then STOP, no repair/retry/alternate route. All execution/tests NOT_RUN; six
diagnostic cases NOT_SELECTED. Native birth/historical teardown/root cause and
Step7 remain unproved. No new controls/fixtures/selection/preparations permitted.
This proposal slice is complete; pending HUMAN RUN APPROVAL, no new execution
grant requested or assumed here. Broader public propagation assessment unchanged.

**R4 fresh SINGLE-RUN GRANT CONSUMED / FAILED-STOP.** Bobai directly approved
MUSE action question `call_81d85023fdf64e938404fe5b57447d4b`, answer
`Approve one R4 attempt (Recommended)`, human
`01a12c46-7990-79c3-a1c5-86e4c3983d5c`, turn
`01a12c46-7971-7623-9cda-318a586accd7`; SCOOBY independently verified the exact
reply. MUSE relayed the exact one-command clearance against the frozen accepted
proposal, without broader authority. Pre-dispatch accepted canonical identities
`79ca6f2a62bddbcea8a423976021338f4a91798baef76ba1f379435f08cf4c6a` (state),
`1b626e2e60411784eeef2cb685c08f13b93220717cb50c6de21d485d54a41db7` (plan)
and all R4/runtime/source/Action/HEAD/noncanonical-delta identities were rehashed
unchanged. This factual grant record is the only pre-dispatch edit. Dispatch
consumes the grant even without an attempt carrier; no retry or substitute path.
The exact frozen command was dispatched ONCE; no retry/substitute path or gate
acceptance. Terminal exec chunk19b55b exit1, tool wall0.398876667 seconds/no running
session; trusted time real0.50/user0.25/sys0.08. All timing values are below their
ceilings, but nonzero/failed cleanup prevents candidate-gate acceptance. R4 preserved
one owned Popen diagnostic: PID88588/ordinal1/role docker:image, cached statusnull,
birth_identitynot_proved; primary, collection_cleanup and final_cleanup snapshots
each show PermissionError/errno1 at os.killpg(SIGKILL). No wait status was observed;
group_absence_observedfalse and attachment_cleanup_okfalse. The failing operation
is observed, NOT its OS root cause or historical group teardown. No claim that the
CLI exited successfully from its complete image-query stdout alone.

Attempt `6517576685cd4500965b68871435c84a`; name
`ota-retained-python-6517576685cd4500965b68871435c84a`; label
`ota.retained.python.attempt=6517576685cd4500965b68871435c84a`.
Only1456-byte first-image-query JSON stdout was retained; no subsequent control,
staging/create/bootstrap/candidate syntax/import/test markers. Source flow and
partial diagnostic show the pre-target stop; no container or stage was created.
All41 candidate methods NOT_RUN, six diagnostic cases NOT_SELECTED/NOT_RUN.
Immutable R3 history and R4 source/bootstrap/candidate/unit/Action unchanged;
post-failure rehash and non-record tracked delta match the frozen identities.

Sealed ownerbobai/groupstaff singular regular0444 evidence in the R4 directory:
- attempt.json1256 bytes SHA256
  `bab6c32b5673598ba9f830e0a0947d61101e0aa47b6da7ca0037df0762f19479`;
- result.json1958 bytes SHA256
  `84057397ac6098e843f202e822132b37d335b2d7da57256a9ecba97d8b813804`;
- validation.log1456 bytes SHA256
  `d0d5e591798c7b0a9f9edb8ebad620bd8dd6070c65a07964ab28f870f0379602`;
- outer-tool-result.json333 bytes SHA256
  `88672c238ccc1f3a6979c46cc69f2da3f34c07a5a2cfdf2a160b28f51002af7c`.
- absence-tool-results.json1539 bytes SHA256
  `1636ef36ff4af5b3272ab38926b1ca09510e29739212cbdd236f5f429125dd03`.
Outer file is the exact copied tool response, not provider attestation. Raw outer
output209 ASCII bytes + log1456 =1665 bytes below64KiB. Whole result1958 bytes bounds
its child-diagnostic array below32KiB. Result remains provisional/overallfalse/
usablefalse/candidatefalse/testsfalse; cleanup_absenttrue means no container creation
attempt, not successful teardown of a created container. Stage absence true does
not convert failed group cleanup into completion.

Independent read-only exact label/name Docker ps checks returned exit0/empty; exact
stage noentry/no-symlink check exit0; ps for acquired PID88588 returned exit1/empty,
and exact Docker/image-query matching-process check exit0/empty. These are CURRENT
matching absence only, no historical group terminality/birth proof and no destructive
signal. The saved matching-process check reads the full process table before
filtering; no unrelated containers/processes were mutated. Copied exact check responses are
retained in absence-tool-results.json with the frozen identity above.
No build, infrastructure, engine/image change, source/control correction,
new signal/capability/failure-reproduction probe, provider/manager/descriptor/VM/
cloud/spend/commit/push occurred. Qualified non-author VLAD runtime-evidence review
`01a12c4c-485b-7473-b6a4-ab1cbd702dda`, final
`msg_0074fa586a19efbe016acbdacd406c8191bec2d527b3dc6bba`, and actual wording-only
recheck `01a12c4f-fa3a-7e42-8582-bed84a1f1bc9`, final
`msg_0074fa586a19efbe016acbdb2ae0b48191a0eb2e4e44cbed98`, are substantively
reconciled by MUSE. Exact persisted configured gpt-6.1-sol/high non-author settings
were independently checked, not provider-attested. Both P3s (acceptance-flag naming
and passive full-process-table read versus mutation/probing) are closed; no remaining
P1/P2/P3 within the diagnostic scope. MUSE independently verified original native
exec item `3653605a-e45d-4aaa-95eb-e5a0509236ff`, exact command/terminal exit1/time,
against copied outer evidence, sealed identities, three distinct failure snapshots,
frozen production phase wiring, absent target markers, bounds, false acceptance
flags, current matching absence limits and unchanged source/inputs/HEAD/delta.
Accepted pre-stamp canonical identities: state
`062a7a82b346f358c5e055c3d3070c05d431fe6755da38b1918dd00d65105454`, plan
`a0fc427b8e7f9eadf956e10827b403c766d6cea310faf8546216fb278aefaab2`.
ACCEPT R4 DIAGNOSTIC OBSERVATION ONLY / factual FAILED-STOP closure. This is a
production failure-location/retention observation, not a passing mock suite,
controller acceptance, OS root cause, child wait/birth or historical group teardown
proof. All41 candidate methods NOT_RUN; six diagnostic cases NOT_SELECTED/NOT_RUN.
Validation HOLD; Step7 OPEN/R1 BLOCKED; Step8/V12.2 inactive. Grant consumed; only
this factual acceptance stamp remains, then STOP. No source repair/retry/alternate
route/live probe/new execution/next preparation/build/infra/spend/commit/push or
promotion. A future source/signal investigation needs a separately agreed bounded
scope, not authorized by this closed attempt. Public propagation assessment unchanged.

#### R4 Signal Failure: Accepted Read-Only Source Assessment

MUSE cleared ONE read-only investigation in turn
`01a12c8a-9f54-72c2-9948-9a563e7bab70`, independently verifying Bobai message
`01a12c89-efc1-7852-8065-40f86f39bdc2` in turn
`01a12c89-ef9c-7480-bd7d-c86d301f4bce`. This is not another runtime grant.
Starting closure identities: state069999fd8c04976ef7e536a7bb3654b5f571fce11652494d226f69677b1a55a8 /
plan0096564c78c4091d266516ebb1340ae536dc1797184e9812abbbd3edf50a18a3.

**Observed / local source.** Frozen R4 parent lines115-145 signals the owned,
unreaped leader's group before waiting. Lines182-233 reach that operation after
pipe EOF, without poll/wait; EPERM prevents the first wait and repeats in cleanup.
Retained result records child88588/statusnull, three EPERMs and no group absence.
Parent60cfe8e81900b2c89677ebf367b996aee22960bcccf0e5d25c6d3be3adae952a and
result84057397ac6098e843f202e822132b37d335b2d7da57256a9ecba97d8b813804
were rechecked unchanged. Already installed
`/usr/local/Cellar/python@3.13/3.13.3/Frameworks/Python.framework/Versions/3.13/lib/python3.13/subprocess.py`
SHA25632b30e8638fadb0a463a20b8fec2ea2e14cccd2d9192061db9c10fef3583b7ef,
line876 initializes cached returncode; lines2018-2072 use waitpid to acquire it.
No installed C source was located in the narrowly inspected package root; public
CPython source below is a reference, not binary provenance or equivalence proof.

**Source-derived findings.** These are static possibilities, not observed host
behaviour:

- [CPython v3.13.3 posixmodule.c](https://github.com/python/cpython/blob/v3.13.3/Modules/posixmodule.c#L8940-L8953)
  audits os.killpg, invokes libc killpg and translates its error. An audit-hook
  exception is also possible in this path; R4 did not identify the error producer.
  [v3.13.3 subprocess.py](https://github.com/python/cpython/blob/v3.13.3/Lib/subprocess.py)
  `_try_wait` / POSIX `_wait`, plus the installed source above, contradict treating
  cached null as a fresh liveness observation. EOF alone is not a wait result.
- [Apple XNU xnu-11417.101.15 kern_sig.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-11417.101.15/bsd/kern/kern_sig.c#L1544-L1620)
  `killpg1` excludes SZOMB members; if a group exists but no eligible member is
  counted, its POSIX branch returns EPERM. The zombie-handling callback comment
  does not override that explicit group filter. Credential and MAC checks can
  also deny signalling. [kern_proc.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-11417.101.15/bsd/kern/kern_proc.c#L3821-L3899)
  applies the filter before callbacks. Thus zombie-only EPERM is a supported
  hypothesis, not proof of a host permission restriction or of R4's root cause.
- [The same XNU kern_exit.c](https://github.com/apple-oss-distributions/xnu/blob/xnu-11417.101.15/bsd/kern/kern_exit.c#L2897-L3109)
  supports WEXITED/WNOHANG/WNOWAIT and avoids reaping for WNOWAIT; zeroed si_pid is
  needed to distinguish no event. This supports non-reaping status acquisition as
  an experiment design primitive, not its availability in this installed Python.

**NOT_PROVED.** This public XNU tag is not mapped to the installed host kernel.
Installed libc/POSIX entrypoint, audit/MAC/sandbox policy, child/group membership
at the failed syscall, child terminal status and historical teardown are unknown.
These hypotheses are not exhaustive. The
[archived Apple kill manual](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/kill.2.html)
describes EPERM/ESRCH but does not resolve the source edge case or identify this host.
No live platform/process/engine inspection was performed in this investigation.

**Recommendation / smallest next proof.** Do not swallow EPERM, infer group
absence from leader exit, reap before a destructive group signal, or retry R4
unchanged. Static evidence cannot select a safe production repair yet. Separately
review a small, owned-child diagnostic fixture DESIGN: compare a live leader with
an exited-but-unreaped leader under the same execution boundary, acquire state
without reaping, retain the exact signal errno and wait status, then close only
owned resources within a fixed total ceiling. A later mixed group with a live
owned member is required before any success interpretation/repair; the first
comparison alone cannot prove arbitrary group closure or retroactively diagnose
R4. Binding the native observation mechanism, executable identities, cleanup and
host permission scope must precede any execution request. This recommendation
does not prepare source, authorize signals/builds or choose an alternate route.

Only this assessment/canonical record changed. Core HEAD8926ddb1fc0e4a225059c48375fc6e1185081b1e
and tracked non-record delta9c026a18a8fd5c4eeef775bb363796ca82ee57809b880b0ceeea116b7275c681
remain unchanged. No imports/tests/probes/signals/source repair/new executable/
retry/container/VM/build/infra/spend/commit/push occurred. All41+6 NOT_RUN;
validation HOLD; Step7 OPEN/R1 BLOCKED; Step8/V12.2 inactive. No user-visible
CLI/schema/reference/vocabulary change: Site/Skills/Examples/Learn/FAQ/Glossary
need no propagation for this private investigation. Broader product propagation
remains open. MUSE independently reconciled the frozen assessment in turn
`01a12c91-515e-7ab2-aebf-a9b378f29722`, final
`msg_0074fa586a19efbe016acbecb980248191a8011c8105d9bc2e`: ACCEPT SOURCE
ASSESSMENT ONLY, no P1/P2/P3. Accepted pre-stamp state
`d30e65a1b898d8b80fe214abc2649f1a0c87d4afa0162fb13f4002092a25028d`, plan
`808de15f9fea1d01fe3cf70023cb73e9bd822a6d1e71cd217c7f35f9cadbef15`.
No safe production repair/R4 root cause established. MUSE separately cleared
the following ONE canonical docs-only proposal, not source or execution.

#### Owned-Group Signal Diagnostic: Accepted Design Only / Runtime NO-GO

**Purpose / status.** Distinguish observed signal results for controlled live,
exited/unreaped and mixed groups on the current caller boundary. This is private
diagnostic planning, not another build controller or validation route. Runtime
NO-GO: separately authored source candidate is staged below, NOT_PARSED and not
accepted; no controls have passed runtime validation or fresh human signal/
execution approval. All41+6 remain NOT_RUN.

**Caller / primitive.** Proposed caller is the already pinned nonroot
`/usr/local/Cellar/python@3.13/3.13.3/bin/python3`, SHA256
`bab9c542210bd094e79ffd88399bf6c31c426e68487469e9172923b4b765432b`, isolated
`-I -B -S`, under the same native exec tool boundary. No root, native helper,
ctypes/FFI, install, Docker or build. Proposed diagnostic source path/hash and
exact invocation must be frozen only after separately cleared source work.
Installed include/python3.13/pyconfig.h SHA256
`4dcf9bd3d621fecda2368747199a8869c6f6a49f4e62909b4bc7b30e183940c3`
declares HAVE_FORK:387, HAVE_GETPGID:535, HAVE_GETSID:586, HAVE_KILLPG:647,
HAVE_SETPGID:1133, HAVE_SETSID:1154, HAVE_WAITID:1571, HAVE_WAITPID:1574.
This is static build-configuration evidence, not runtime
export/permission/enforcement proof. Proposed observation is native
`os.waitid(P_PID, acquired_pid, WEXITED | WNOHANG | WNOWAIT)`.
[CPython v3.13.3 wrapper](https://github.com/python/cpython/blob/v3.13.3/Modules/posixmodule.c#L9278-L9339)
uses native siginfo_t, zeroes si_pid, and returns None for no event; its own compiled
ABI handles the structure rather than new handwritten layout. Missing method or
constants/error is STOP, never an alternate primitive. Native module equivalence
to public source remains NOT_PROVED until appropriately bound validation.

**Ownership / topology.** One single-threaded observer F, SIGCHLD explicitly
SIG_DFL, sole reaper via exact-PID waitpid(WNOHANG). No Popen/poll/background reaper.
Before ANY fork, F acquires its own PID, calls os.setsid ONCE, and requires native
getsid(0) and getpgid(0) equal that acquired self PID. A failed setsid, existing
group-leader condition, missing primitive, mismatch or ambiguous observation is
STOP before child creation; no helper, added process, inherited-session fallback
or racy membership enumeration. [Apple setsid reference](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setsid.2.html)
describes the initial private session; [setpgid reference](https://developer.apple.com/library/archive/documentation/System/Conceptual/ManPages_iPhoneOS/man2/setpgid.2.html)
requires the target group to be in the caller's session. Together with reviewed
no-fork/no-exec child roles, this excludes unrelated same-session joiners within
the diagnostic model; it is not a host-compromise/adversarial-code guarantee.
At most TWO simultaneously owned children, FOUR total creations across three
sequential cases, no descendants or child fork/exec. F acquires every PID from
its own fork return; no child-reported PID authorizes action. Each leader L uses
setpgid(0,0) in F's verified private session; mixed-case member M is another DIRECT child of F and
joins L's group while L is alive and unreaped. This avoids orphan-grandchild
reaping. It deliberately differs from R4 start_new_session: it tests group
semantics, not session-topology equivalence or historical R4 cause. A native
getpgid on each acquired child plus fixed readiness IPC records group placement;
neither that read nor child IPC is syscall-atomic membership attestation.
Caller executable and native tool boundary stay unchanged; F's new private
session and child groups are deliberately different from R4's session-leader
child topology. Session establishment/permission remains NEEDS_VALIDATION.
L remains unreaped until the case's one destructive group-signal operation is
over. After any reap, no destructive signal to that group is allowed. A group
may contain only these source-controlled children; unrelated groups are never
enumerated or targeted. Lifetime/birth equivalence is not inferred from PID alone.

**Descriptor ownership.** F establishes an exact inherited descriptor allowlist
before fork; any unaccounted descriptor or inability to establish/close the
non-allowlist set blocks the route. Do not assume CLOEXEC helps a no-exec fork.
Each child receives only its gate read end and readiness write end, closes every
other inherited descriptor BEFORE readiness, then closes readiness writer after
one <=64-byte message. The fork role must terminate via its own os._exit(status),
including on error; it never returns/falls through F's cleanup, retention or
reaper path. Future source review must lock all normal/error exits.

| Role | Retained descriptors | Required immediate closure |
| --- | --- | --- |
| F | bounded stdout/stderr; evidence directory/files; gate WRITE and readiness READ for each currently owned child | every child-only gate READ/readiness WRITE after fork; unused stdin/unknown inherited handles |
| L | own gate READ; own readiness WRITE only until sent | own gate WRITE/readiness READ; every peer gate end/readiness end; all evidence directory/file handles; stdin/stdout/stderr and all non-allowlist descriptors |
| M | own gate READ; own readiness WRITE only until sent | own gate WRITE/readiness READ; BOTH ends of all L/other peer gates and readiness channels; all evidence handles; stdin/stdout/stderr and all non-allowlist descriptors |

Only F holds any gate writer. Closing all F gate writers, including on parent
loss, can therefore deliver EOF without entering F cleanup in a child. Source
mitigation is not a demonstrated parent-loss result. IPC is bounded sequencing,
never ownership/identity authority.

**State machine / races.** Child roles report bounded readiness, then wait on
their own bounded gate or exit on explicit release/EOF/deadline. F first records
exact native waitid result; expected
exit requires matching acquired si_pid, CLD_EXITED and known status, never EOF or
cached state. For the mixed case, M must be ready while L is still alive, then
F releases only L and observes L exited with WNOWAIT. M's pre-signal waitid must
report no event. F records one os.killpg(L.pid,SIGKILL) return/error, timestamps,
then native post-signal status and exact-PID waitpid(WNOHANG) result for every
owned child. No blocking wait is allowed: poll waitid(WEXITED|WNOHANG|WNOWAIT)
only under the remaining original budget; after a matching terminal event,
waitpid(WNOHANG) must return that same acquired PID and consistent status.
A zero waitpid return permits only bounded observation within that budget,
never a blocking wait, deadline renewal or second destructive signal. ECHILD,
unexpected PID/status or lost wait ownership is FAILED-STOP, never synthetic zero.
A no-event result is not proof of continued liveness at the later syscall;
deadline exits, unexpected statuses, missing readiness or intervening uncertainty
make the case INCONCLUSIVE. No result proves atomic group membership or arbitrary
descendant/escaped-group closure.

| Case | Required precondition | Signal/result interpretation | Acceptance limit |
| --- | --- | --- | --- |
| Live L | readiness/group observed; waitid no event | zero plus acquired L killed by SIGKILL supports live-group signal capability; EPERM/ESRCH is refusal or timing uncertainty | cannot identify policy cause from errno alone |
| Exited sole L | matching WNOWAIT exit event; no member ever created | EPERM is compatible with published zombie-filter hypothesis; zero shows another observed behaviour; ESRCH is inconclusive | not cleanup success, production repair or R4 diagnosis |
| Exited L + live M | L exit event retained unreaped; M readiness/group and no exit event | zero plus M's SIGKILL terminal supports mixed-group capability; EPERM/ESRCH/missing M terminal is inconclusive/failure | must not treat L's exit/EPERM as group absence |
| Any case | malformed event, ECHILD, unexpected reaping, changed identity, cap/deadline failure or unavailable primitive | STOP with retained failure; no retry/fallback/next case | no positive experiment result |

**Bounds / containment proposal.** Clean allowlist env PATH=/usr/bin:/bin,
HOME=/nonexistent, LC_ALL=C; no credential/provider variables. Fixed reviewed
stdlib-only roles, no subprocess exec, network calls, external resource acquisition,
imports from repo/user/site, child file writes, container/engine access or platform
enumeration. Allow only acquired IPC, exact owned-child state/signal/wait and
private retained evidence. Proposed F ceiling30s including acquisition/retention,
work20s/cleanup10s; each benign gate child exits on EOF or its own <=4s deadline.
Acquire/set/read-back CPU/FD/core/file limits before fork, fail closed if unavailable:
F CPU3s, each child CPU1s, F/child FD32, core0; child file-size0, F file-size32KiB.
No per-UID nproc change, which could affect unrelated work. Fixed creation count
is source-enforced, NOT a kernel process limit. Source-local memory/log bounds
are NOT kernel-enforced RAM or network confinement. No hard wall-clock or
host-kernel interruption guarantee follows from Python time checks. These limits,
fork/IPC support and safe cleanup are NEEDS_VALIDATION, not established runtime
controls; if required host confinement cannot be bound without infrastructure or
new tooling, keep runtime blocked rather than weaken controls/invent a fallback.

**Cleanup / failure.** First close parent-owned gate write ends so benign roles
see EOF; wait non-reapingly for terminal evidence up to remaining ceiling. While
an owned L is still unreaped AND native group placement was observed as L.pid,
a cleanup signal may target only that acquired group if its one destructive
operation has not already been attempted. At most ONE destructive signal attempt
per group, including cleanup, under separately granted exact scope; every denial
stays a failure. Without that placement/ownership, use gate closure/wait only.
Reap only acquired direct children under the same bounded WNOHANG rule; require
matching terminal evidence for positive cleanup and retain actual wait status.
After denied signalling, use gate closure plus bounded non-reaping observation,
then WNOHANG acquisition if terminal; NEVER block on a live/unknown child.
No retry after a destructive-signal error and no guessed/reused group signal.
Any remaining live/unknown child at ceiling means FAILED-STOP with external
exact-owned reconciliation, not success or permission to keep probing/signalling.
Lost parent: closed gates and benign child deadlines are source-design mitigations,
NOT demonstrated parent-SIGKILL cleanup. The outer terminal/evidence consumer
must reject missing terminal/carrier and independently reconcile acquired children
before any subsequent work. No automatic success or general kill guarantee.

**Retention / outcome.** Proposed private exclusive attempt directory, no shared
mutable evidence path; attempt/result/log each bounded, combined retained bytes
<=32KiB and combined outer output/log <=16KiB. Reserve failure publication space;
regular/no-link/singular/current-owner files, O_EXCL/NOFOLLOW creation0600, bounded
write/fsync/fchmod0444/fsync/close, directory fsync if supported. Retention failure
is sticky FAILED-STOP; no repaired positive carrier. The design must retain native
si_pid/si_code/si_status, exact waitpid status, acquired PID/observed PGID, ordering,
errno/exception class, cleanup evidence and observed outer terminal/time. Optional
host identity collection is not silently permitted; separately bind any future
read-only kernel/library provenance request. Current absence/killpg(0) alone cannot
substitute for retained child terminal status or historical group teardown.
Even all three expected outcomes establish ONLY this controlled diagnostic; R4
root cause and candidate/41-method/Step7 acceptance remain NOT_PROVED.

**Review / next gate.** Qualified non-author VLAD review
`01a12c98-8a46-7af3-a184-54998f944cb8`, final
`msg_0074fa586a19efbe016acbeee3378c81919bbe2e01d9f9549d`, configured
gpt-6.1-sol/high independently verified (not provider attestation), found P1
inherited-session group exclusivity and P2 gate descriptor ownership / blocking
wait ambiguity. MUSE reconciled DESIGN HOLD and permitted ONLY the narrow
docs-only fixes above. Original frozen state9365c64689d78e6d0d19a662e1cdc296590227b695d7496ee9532f34c0eac258 /
pland4c24b3a058d6b7d8986ba70969dedf647e76453814d799c036803c51781a464.
These risks were source/design findings, not observed host escapes. Repaired
design received qualified non-author VLAD recheck
`01a12c9f-e63f-74f0-bd33-5ff5c8d2326c`, final
`msg_0074fa586a19efbe016acbeffb5e8c819195dbf90cbc96b561`. Exact persisted
configured gpt-6.1-sol/high was independently checked, not provider attestation.
All three findings closed; no remaining P1/P2/P3 WITHIN CORRECTED DESIGN SCOPE.
MUSE substantively reconciled and ACCEPTS CORRECTED DESIGN ONLY in turn
`01a12c9f-e6a1-7391-9f25-74e9b3997691`, independently checking the corrected
ownership/descriptor/reaping rules, preserved caps/races/STOP limits and official
primitive references. Accepted pre-stamp state
`8cfba1a43061d9278f6f659bbe84933c0059cac5ad977d7176bcd4df7290ad61`, plan
`d6fd14ababe62b4f8d07cc488a587dcf64c072e3db29e812653d584d264642ce`.
ONLY factual canonical acceptance stamp cleared, then STOP. No source preparation,
additional investigation or implementation/action grant follows from acceptance.
Resolve source-only design gaps before considering any implementation. Actual
diagnostic source/control review and fresh exact human action approval must precede
native execution/signals. Canonical docs-only; no code/import/test/probe/signal/
retry/VM/build/infra/spend/commit/push. Step7 OPEN/R1 BLOCKED; Step8/V12.2 inactive.
No public CLI/schema/claim/vocabulary change or first-party propagation for this
private proposal. That design-only grant stopped at its acceptance stamp. The
subsequent separately cleared source-authoring slice is recorded below.

#### Owned-Group Diagnostic R1: Source HOLD / Reconciliation Pending / Runtime NO-GO

**Authorization / ownership.** SCOOBY sole writer. Bobai directly said 'lets work'
message `01a12f2e-7134-7653-b69b-0c174b527383`, turn
`01a12f2e-710d-7ad1-8869-2c66827828dc`, after the source-only/no-execution next
step explanation. MUSE independently verified the message and accepted-design
state849856cac5998c3e41ca143a1a5f7bba8ca3e131059cf4bed191af3e02ae1d49 /
plan68be32d2fe28e5844d733ed4cf876252aca94a05076e44afb1e01102d9c19bbf,
then cleared ONE private diagnostic source + ONE authored mocked-test source in
turn `01a12f2f-06eb-7e20-835a-2e001f213ba3`. Canonical factual staging only;
source review does not imply execution approval.

**Frozen packet.** `/Users/bobai/.codex/artifacts/2026-10-12/owned-group-signal-diagnostic-r1`:

- `diagnostic.py`: 469 lines / 20356 bytes, SHA256
  `e43252524177fc53169db8151884891ba3d92818dbbdf995674dffc70bdadde5`.
- `test_diagnostic.py`: 284 lines / 14575 bytes, SHA256
  `d49fee97c0ee731127c220ed7bf59c467a4ecaa764fc49984553e0701683ec2b`.

Both are regular files, single link, bobai:staff, exact0444/non-executable; frozen
by read-only mode and tool-side identity, not root-owned or tamper-proof custody.
No third executable/manifest/handoff was created. Exactly 21 mock test methods
are authored. Both sources NOT_PARSED, NOT_IMPORTED and NOT_EXECUTED; all21
new mocks NOT_RUN. No AST/py_compile/linter/formatter/test was used. Read-only
source/hash/stat/count checks and canonical git diff --check do not certify syntax
or behaviour. No passing source acceptance is claimed.

**Source shape / boundaries.** Explicit entrypoints, definitions and fixed stdlib
imports only; no import-time qualification, descriptor mutation or native probe.
Observer verifies fixed caller/clean env, initial exact FD admission, limit
readback, SIGCHLD default and private self-SID/PGID BEFORE fork. Native runtime
FD inventory is confined to the caller's `/dev/fd`, not OS process enumeration;
it is code for a future separately approved run, NOT a probe performed here.
Unknown/missing FD, primitive/session mismatch or limit failure refuses before
fork. Children are direct fork returns; exact inherited FD closure drops own/
peer gate writers, readiness parent ends, evidence and stdio, with child-only
_exit. Two simultaneous/four total; no child fork/exec/FFI/helper. WNOWAIT state,
signal errors, native group observations and exact WNOHANG statuses remain
separate. One destructive attempt/group, marked before the call; no post-reap
signal, ECHILD-as-success, deadline renewal or signal retry. Cleanup first closes
gates and acquires bounded terminal/wait status; it does not force a second signal.

The three controlled cases retain live/exited/mixed outcomes and explicit
inconclusive/error states. Sole-zombie EPERM is only a source-compatible
observation. Result always provisional/runtime_acceptedfalse/step7_provedfalse;
outer terminal/time/retention must be separately reconciled even if all cases
collect. Partial record writes stay0600; completed bytes/data sync precede0444
hardening and metadata/directory sync. Publication/close/cap/deadline errors exit
nonzero, no repaired positive carrier. Source count/no-network discipline,
cooperative clocks, proposed resource readback and gate EOF/deadlines are not
demonstrated kernel containment or parent-SIGKILL cleanup.

**Authored mock inventory.** Session refusal/mismatch before fork; both L/M own,
peer and evidence descriptor closure; child-only error exit; denied signal consumes
attempt; post-reap refusal; mismatch/no-event native observation; ECHILD loss;
exact WNOHANG and zero-result budget; count/deadline/unknown FD refusal;
oversize/partial/sticky retention and complete-data hardening order; mixed normal
exit refusal; sole-zombie observation bounds; cleanup unknown child without retry;
limit readback mismatch. Every native/FD/resource/signal/publication effect in
these test paths is mockable; tests are not selected/executed by source authoring.
They are distinct from the41 controller and6 R4 diagnostic methods.

**Repository / propagation.** Core HEAD
`8926ddb1fc0e4a225059c48375fc6e1185081b1e`; tracked non-record delta
`9c026a18a8fd5c4eeef775bb363796ca82ee57809b880b0ceeea116b7275c681` unchanged.
Controller/R3/R4/product source untouched. Private source files and these two
canonical records are the only slice changes. No public CLI/schema/reference/
vocabulary/claim change: Site/Skills/Examples/Learn/FAQ/Glossary need no propagation
for this private slice; broader product propagation remains open.

**Review / gate / STOP.** Qualified non-author VLAD READ-ONLY SOURCE review
`01a12f4f-34ee-7af0-8b9c-80c95d831ae6`, final
`msg_0074fa586a19efbe016acca1bd6f54819198050554cdebe91e`, independently verified
configured gpt-6.1-sol/high (not provider-attested), found TWO P2s / SOURCE HOLD:

- diagnostic.py:224: deadline/cancellation check precedes descriptor/pipe
  preparation, not the fork itself. Require a fresh check immediately before
  fork and mocks advancing time/cancellation during preparation.
- diagnostic.py:319: CLD_KILLED and CLD_DUMPED both compare only signal. Require
  native core-dump indication to match the event class, refuse if unavailable,
  and author contradictory-status regressions.

Other repaired ownership/FD/child-exit/bounded-wait/consumed-attempt boundaries
were present; the21 mock methods do not close these two findings. Review made
no edits or parsing/import/testing/probes/signals. Exact frozen source/docs/HEAD/
delta remained stable during review. Original reviewed canonical identities:
state562b5ff28fc196c119a1aa2c3c258f9eda14a125b5730c3e876fdacc53b161bf /
plane5e7ccd33d514c34becc43f0df6270031c43bee2a3aa6d3e5aab01c3aff9f315.
MUSE received the qualified findings and explicitly requires R1 unchanged, no
R2 edits/preparation yet, pending separate reconciliation of the two P2s. Author
also flagged the pre-packet outer carrier's missing original failure detail as
a diagnostic limitation for reconciliation, not an independently confirmed
security finding or permission to broaden the repair. Only these factual review
records were updated; R1 bytes/protection unchanged. MUSE separately reconciled
in turn `01a12f6c-94fc-7c03-ae6c-6f1c802596a7`, final
`msg_0074fa586a19efbe016acca7b849e881919d0e7dff3a2f5cbd`, confirming both
P2s and the pre-packet outer-error omission as MUSE P3 (not VLAD's finding).
GO for ONE R2 SOURCE-ONLY correction under the verified source-authoring grant:
copy exactly diagnostic.py/test_diagnostic.py into
`/Users/bobai/.codex/artifacts/2026-10-12/owned-group-signal-diagnostic-r2`, preserve
R1 unchanged, update ROOT, add the immediate pre-fork deadline/cancellation check,
require callable native WCOREDUMP before child creation and exact core-bit/event
matching, and retain closed coarse phase / bounded primary type+errno separately
from secondary retention failure in the EXISTING <=512-byte outer carrier.
No exception messages, traces, new carrier/path/probe/logging framework.
Author targeted expiry/interruption-during-preparation mocks with accounted FD
cleanup; positive/contradictory core indicators and unavailable/noncallable guard;
pre-packet Refusal/OSError versus secondary retention refusal, sticky retention-only
failure, and worst-case carrier cap. All mocks remain NOT_RUN; no parsing/imports.
R2 is authored and frozen source-only at
`/Users/bobai/.codex/artifacts/2026-10-12/owned-group-signal-diagnostic-r2`.
`diagnostic.py`: 21,482 bytes, 496 lines, SHA-256
`fadd4a02cc1edbcd77de1d08ae750e6db8308294b14da26fbc4edb82442f9d42`.
`test_diagnostic.py`: 20,665 bytes, 393 lines, 27 authored mock methods,
SHA-256 `ef5a5429e60486be688d1b718a0680a243d9cea6b8e074924f3e13b378e82c06`.
Both files are regular, current-owner/current-group, singular, 0444 and
non-executable. R1 remains unchanged at its recorded hashes. R1-to-R2 unified-diff
stream hashes are
`dc799974af5c6e76cb76a9831896f52fef6992ac65d15df1b88df85705a3cf26`
for `diagnostic.py` and
`13593ab5d0bf191ff19c39da1810cf7ac3b86099d7076ce8503b312eecdd79ef`
for `test_diagnostic.py`. The bounded source delta implements only the fixed R2
root, immediate cooperative pre-fork admission, callable native `WCOREDUMP`
admission and exact event/core reconciliation, plus coarse primary
phase/type/errno separated from retention failure in the existing <=512-byte
outer carrier. New authored mocks cover expiry/interruption during FD preparation,
positive/contradictory core indications, unavailable/noncallable primitive,
pre-packet Refusal/OSError separation, sticky retention-only failure and the
existing carrier/capture/retention caps. ALL 27 mocks remain NOT_RUN; neither R2
file was parsed, imported or executed. Qualified VLAD read-only review
`01a12f7d-8402-77b1-8f3b-c38d8d52f47f`, final
`msg_0074fa586a19efbe016accadd15a8081918088a594755f56dd`, found one P2:
two mocked execute paths left the process-global `os.umask` mutation live, plus
one P3 stale R1 mock count. The narrow correction mocks `os.umask` in both paths
and records the current R2 count; production diagnostic bytes are unchanged.
R2 is refrozen at the identities above. Qualified VLAD delta recheck
`01a12f89-4642-7f10-9514-2746af6ab732`, final
`msg_0074fa586a19efbe016accae8e4d6481919a46b48b60f51e4e`, closed both
findings with no remaining P1/P2/P3 and returned SOURCE GO for MUSE
reconciliation only. MUSE substantive reconciliation
`01a12f8a-bf85-7403-ac3e-9504b3ee6ee6` independently rehashed the packet,
read both qualified VLAD finals, found no remaining P1/P2/P3 and marked the
exact frozen R2 SOURCE ACCEPTED ONLY. Accepted pre-stamp canonical identities:
state `4241264e0e8e5bd2a274703193123b55cc208140b29807b7d7ef5689a40a0e84`,
plan `20fd0f4fe63a252f3c6c2da36dbdda3b885f8ab1d7ae23c627ba4388618ea9bd`.
This closes source review only. Next: read-only identify/freeze an existing
contained path for syntax plus exactly 27 R2 mocks. That inspection found no
unchanged suitable path. Historical R11 remains incompatible as recorded above.
R4 `parent.py` lines 29-33/170 and `bootstrap.py` lines 14-64/141-145 bind the
three Core inputs, eight named classes and exact 41-method report rather than the
two R2 files/27 methods. The R4 child-group path at parent lines 115-145 is also
the retained route that failed on the first cached-image query with
`os.killpg(SIGKILL)` EPERM before container creation. Exact reuse therefore cannot
validate R2. Any binding/selection or launcher correction is new source work and
requires MUSE selection/review before authoring. Parsing/import/test execution
requires separately reviewed exact invocation/controls and fresh human action
approval. No duplicate human question for this source-authoring
scope; a native action still needs fresh approval.
No source acceptance from authorship, no runtime preparation/action grant.
Existing 41+6 and current R2 27 mocks NOT_RUN; effective controls/R4 root cause NOT_PROVED.
Validation HOLD; Step7 OPEN/R1 BLOCKED; Step8/V12.2 inactive. No native/platform/
process/engine probe, signal, retry, build, container/VM/VPS/cloud/infra/spend/
provider/commit/push/release. Stop at frozen source review boundary; a later run
requires reviewed exact source/controls and fresh exact human action approval.

**Owned-group diagnostic validation successor: DESIGN R1 / SOURCE HOLD.** MUSE
read-only reconciliation selected ONE design-only successor derived from the
existing private R4 parent/prelude. This is not permission to edit that packet,
author the successor, parse/import/test R2, probe the engine, or execute anything.
The design corrects two fixed blockers together: exact R2 selection and the
host collector's cached-returncode lifecycle. It does not introduce a generic
validator, different launcher, new process owner, kernel primitive, alternate
host or fallback path.

**Prospective fixed files and selection.** A later separately cleared source-only
packet would contain exactly `parent.py`, `bootstrap.py` and
`test_parent_lifecycle.py` under the fixed private directory
`/Users/bobai/.codex/artifacts/2026-10-12/owned-group-signal-validation-r1`.
`parent.py` binds only immutable R2 `diagnostic.py` SHA-256
`fadd4a02cc1edbcd77de1d08ae750e6db8308294b14da26fbc4edb82442f9d42`
and `test_diagnostic.py` SHA-256
`ef5a5429e60486be688d1b718a0680a243d9cea6b8e074924f3e13b378e82c06`.
The prelude compiles those two bytes in memory, imports only the test module
inside containment, requires one `DiagnosticTests` class and builds one suite
from this closed 27-name set, with no discovery or fallback:

```text
test_session_failure_blocks_fork_and_cases
test_session_mismatch_blocks_fork
test_child_closes_own_peer_writers_and_evidence_then_only_exits
test_child_closure_failure_exits_without_parent_cleanup
test_denied_signal_consumes_only_attempt
test_post_reap_group_signal_is_refused
test_nonmatching_event_is_not_terminal_evidence
test_no_event_is_bounded_and_never_reaped
test_echild_is_lost_ownership_not_synthetic_success
test_reaping_uses_only_exact_pid_wnohang_after_terminal
test_zero_waitpid_cannot_renew_budget
test_count_and_deadline_refusal_precede_fork
test_unaccounted_fd_blocks_admission
test_oversize_retention_refused_before_open
test_sealing_hardens_only_complete_data_before_directory_sync
test_partial_record_is_not_hardened
test_partial_retention_failure_cannot_pass_execute
test_mixed_normal_exit_is_not_signal_capability
test_sole_zombie_eperm_is_observation_not_runtime_acceptance
test_cleanup_unknown_child_retains_failure_without_signal_retry
test_resource_readback_mismatch_refuses
test_expiry_and_interruption_during_pipe_preparation_forbid_fork
test_reap_requires_matching_native_core_indication
test_reap_refuses_contradictory_native_core_indication
test_missing_or_noncallable_core_primitive_refuses
test_prepacket_primary_failure_is_separate_from_retention_failure
test_worst_case_outer_carrier_and_retention_caps_remain_bounded
```

The target gate requires exactly 27 started/completed, zero skipped and one
fixed terminal marker. It never calls diagnostic `main`, selects the historical
41 or six R4 collector mocks, or admits an unlisted method. Prospective collector
source regressions remain a separate exact nine-method suite in
`test_parent_lifecycle.py`: EOF before status waits without signalling; terminal
before EOF is retained; live timeout consumes one bound group signal; group
identity ambiguity refuses without signalling; cancellation follows the same
single-attempt state machine; contradictory waitid/waitpid status refuses;
ECHILD loses ownership and forbids signalling; denied signalling is retained and
not retried; group absence is decided while the exact leader remains unreaped.
Those nine are review inputs for the host collector, never silently included in
the 27 target gate.

**Exact-owned collector state machine.** Preserve `Popen(...,
start_new_session=True)` and the exact returned child PID, but never use cached
`Popen.returncode`, pipe EOF or a zero-length read as terminal evidence. Native
function admission is split from typed constants: `waitid`, `waitpid`, `getsid`,
`getpgid`, `WIFEXITED`, `WEXITSTATUS`, `WIFSIGNALED`, `WTERMSIG` and `WCOREDUMP`
must be callable; `P_PID`, `WEXITED`, `WNOHANG`, `WNOWAIT` and expected `CLD_*`
values must be `int` and not `bool`. This admission is source design, not proof
that the installed host provides the primitives. Full reconciliation retains the
waitid event, raw waitpid status, exit/signal identity and core-dump distinction.
Acquire and retain the exact child PID/session/group binding
(`sid == pgid == pid`) while it is observable. A failed or changed binding is
ambiguous ownership, not authority to signal.

Use non-reaping `waitid(P_PID, pid, WEXITED|WNOHANG|WNOWAIT)` as the sole
terminal observation. EOF before status continues bounded observation. A retained
terminal event forbids destructive signalling. Timeout or recorded cancellation
with no terminal event may consume exactly one `killpg(pid, SIGKILL)` attempt only
while exact child wait ownership and the acquired session/group binding remain
intact; denial or uncertainty is retained and never retried. After any allowed
group decision, require a matching terminal event, make the final same-attempt
group-absence decision while the exact leader is still unreaped, then perform
bounded `waitpid(pid, WNOHANG)` and require the exact PID and status to reconcile
with the retained event. Reaping, ECHILD, contradictory status, changed binding
or deadline exhaustion consumes the authority to signal. There is no post-reap
signal/probe, guessed PID/PGID or suppression of EPERM. If absence cannot be
established before the cleanup deadline, exact-PID reaping may prevent a zombie
leak only after all authorized group decisions are closed; the validation remains
failed/HOLD and group cleanup is not proved.

R2's private-session/direct-fork topology does not prove this Docker CLI
`Popen` topology. Parent loss, escaping control descendants and historical EPERM
root cause remain `not_proved`.

| Observation | Required disposition |
| --- | --- |
| EOF, no terminal event | keep bounded exact-PID observation; never infer success or signal from EOF |
| matching terminal event | no destructive signal; decide group absence while unreaped, exact-PID reap and reconcile |
| live timeout/cancellation with stable binding | consume one SIGKILL group attempt; retain failure; require terminal/absence/exact reap |
| denied signal | retain errno and STOP; no retry or alternate signal |
| missing/changed group binding | STOP without signal; retain ownership/control gap |
| ECHILD, unexpected PID or contradictory status | lost/ambiguous ownership, STOP, no signal |
| group absence unresolved at cleanup deadline | fail/HOLD; close group decisions before exact-PID hygiene reap; no acceptance |
| all target/collector/retention/absence predicates pass | provisional contained mock result only; still no native diagnostic or Step 7 proof |

**Preserved containment and acceptance ceiling.** Reuse the same pinned existing
local engine/CLI/image and retained fixed-purpose parent model: no pull/install/
daemon change/new infrastructure/network; read-only staged inputs and rootfs; no
socket, host home or credentials exposed to target code; UID/GID 65534; all
capabilities dropped plus no-new-privileges; one target process; 512 MiB memory
with no additional swap; one CPU and 10-second CPU rlimit; nofile64/nproc1/core0/
fsize0; no writable mount or tmpfs; isolated clean Python environment; whole
attempt <=30 seconds with the last five reserved for cleanup; existing <=64 KiB
shared capture. Keep exact configuration readback, immutable same-attempt names/
labels/IDs, sealed bounded retention, trusted terminal/timing evidence and owned
container/staging/attachment absence. Results remain provisional,
`runtime_accepted:false`, `step7_proved:false` and unusable. Advertised controls
are not effective-containment proof.

**Implementation gate.** Missing/noncallable non-reaping wait primitives, inability
to acquire stable exact session/group ownership, unsafe lifecycle on the current
host, inability to preserve the controls above, or any need for a different owner,
engine, host or larger budget is HOLD with the exact gap. Future source requires
separate MUSE clearance, frozen identities, qualified VLAD source review and MUSE
reconciliation. A later exact invocation/control review and fresh human action
approval are required before any parsing, import, test, process or engine action.
All 27+41+6 remain NOT_RUN; Step 7 remains OPEN/R1 BLOCKED.

**DESIGN R1 review HOLD.** Qualified independent VLAD review
`01a12f99-abd1-7010-abea-e905db328e4f`, final
`msg_0074fa586a19efbe016accb59faa3c81918750ed11c01d6d95`, confirmed the
two-file binding, closed 27-name target selection and separate collector-test
scope, then found one P1 and one P2. P1: the proposed pre-reap group-absence
decision has no sound success predicate. An unreaped zombie leader can keep the
group extant, while the inherited `killpg(pgid, 0)` observation may return EPERM
under the documented Apple hypothesis rather than generation-bound absence.
Leader terminal state, EPERM or a later post-reap/reused-number probe cannot prove
owned-group termination. The design needs an admissible generation-bound
quiescence observation or must retain this named capability gap. P2: primitive
admission must separate callable functions (`waitid`, `waitpid`, `getsid`,
`getpgid`) from typed integer selector/flag/code constants and must explicitly
retain full terminal reconciliation including the core-dump distinction. No
implementation is cleared. Do not weaken cleanup, reinterpret EPERM, probe after
reap, change owner/engine/host or increase the budget to route around P1. Exact
collector test IDs/disposition must be frozen during any later source review.
All source/runtime boundaries above remain unchanged.

**MUSE reconciliation / named capability gap.** MUSE turn
`01a12fa6-6065-73a3-be10-29cf75b03d2a` confirms DESIGN HOLD. The current
admitted Darwin plus Docker-CLI contract has no operative pre-reap absence
success predicate for **generation-bound host-control-group quiescence**.
Exact-leader wait evidence binds only the leader; `getsid`/`getpgid` bind numeric
identity while held; `killpg(..., 0)`, EPERM and post-reap number probes do not
prove quiescence of all members/descendants. This is a named capability gap, not
a claim that no OS mechanism can exist and not the historical R4 root cause.
The DESIGN R1 outcome table remains historical/conditional only, not current
acceptance authority. No collector/validator source or execution proceeds.

Unblocking a host-helper route requires an admitted stable owner/generation handle
or equivalent kernel/manager primitive binding every relevant member/descendant
to the exact launched generation and independently retaining terminal/quiescence
evidence without weakening cleanup, time or retention predicates. Exact-leader
wait, endpoint/container absence and green mocks are insufficient. The protected
Authority Launcher is not repurposed as a generic validator/build owner.

A fixed-purpose single-process control transport to the same pinned local Engine
API is a future design hypothesis only: removing host CLI helper groups could
replace the blocked group-quiescence obligation with a zero-helper transport
obligation. It is not approved design, installed compatibility, socket authority,
effective-control proof or runtime permission. It requires separate direct human
approval before any design amendment, source preparation, socket connection,
probe or execution.

**Engine-direct validation successor: DESIGN R5 / DESIGN ACCEPTED ONLY.** Bobai directly
approved one design-only alternative through MUSE question
`call_b41280b371f1487b8cfb56fdda83b2cb`. The approval permits this text only. It
does not authorize source, socket access, an Engine query, a capability probe,
parsing/importing/testing R2, a process, container, build, infrastructure, spend,
commit, push or release. The Docker-CLI collector above remains DESIGN HOLD; this
design does not reinterpret its EPERM result or retrospectively prove group
quiescence.

Frozen DESIGN R2 identities were plan
`9539115136ba0a5b93e298fa10f5627920309f9f061f595aee7c1e6f74194c59`
and current-state
`c15d5d01d2262798b20f6720d2583eed69d5a317907345c3c471ab4f865274e1`.
The original VLAD turn `01a12fbd-3c9e-77e2-918c-59580da4567b` was closed as
superseded/incomplete and owns no acceptance. Replacement independent reviewer
Aquinas `01a12fde-afc2-75c3-9f69-be97af998ad6`, persisted turn
`01a12fde-b021-70f2-95e0-a0556b7e1a8e`, final
`msg_0a529e04e536a801016accc63dab388197ad5611ff3ac14947`, returned DESIGN HOLD
with four P2s and one P3: unresolved late mutation outcomes, undefined attach
completion/order, missing two-socket cleanup handoff, incomplete raw-framing
bounds and contradictory readback counts. DESIGN R3 corrects only those five
connected items under the existing design-only grant; no implementation or
action permission follows.

Frozen DESIGN R3 identities were plan
`0a28d43b7db2c0619624a710baffc4bedc99d9df569351d71b3a757b0907dee1`
and current-state
`aae4c8653caf8689f1e476c914546da5696849be497e66d3d63f65360c50b0c1`.
Aquinas bounded recheck turn `01a12ff0-9ead-7390-98f2-03cecb5d8d64` closed
the framing/EOF, socket handoff/deadline, parser-bound and named-readback findings
at design level, then retained one P2 and one P3: the later-attempt prohibition
was not global across every unresolved mutation/cleanup state, and the refusal
count did not distinguish pre-attach from post-attach/pre-start cleanup. DESIGN
R4 corrects only those two items; the reviewer disposition remains DESIGN HOLD
until bounded recheck and MUSE reconciliation.

Frozen DESIGN R4 identities were plan
`5db5f44777f4f5acf8d022d1f1050b7142e1bbba11609b98b2147c60ed47d646`
and current-state
`6625a5cadef1a84bf754c615e0b56d69110194ba1836c3490b4b4082f6a6f1d8`.
Aquinas recheck turn `01a12ffb-6c96-73e0-b931-cabd9a0b3aa2` closed the count
finding, then retained one P2 and one P3 wording conflict: independently observed
absence was still presented as an alternative to reviewed recovery, and stage
retention was stated as unconditional even after completed normal cleanup. DESIGN
R5 clarifies only those two statements; all transport, clock, resource and
connection limits remain unchanged.

**Fixed transport and zero-helper invariant.** A future separately authorized
source candidate would be one trusted nonroot Python parent using only standard-
library AF_UNIX sockets and bounded HTTP/1.1 framing against the same retained
endpoint `/Users/bobai/.orbstack/run/docker.sock`. It must not import or invoke a
Docker SDK, `subprocess`, shell, `docker`, `curl`, `wget`, SSH, fork/spawn/exec,
native helper, worker thread, TCP socket, proxy, listener, alternate endpoint or
daemon configuration. It may not read `DOCKER_HOST`, proxy variables, registries,
credentials or secrets. Every request uses an absolute `/v1.51/...` target; API
negotiation, unversioned paths, redirects, retry and TCP fallback are refused.
Docker's public matrix records Engine 29.4 as supporting API 1.40 through 1.54,
but the retained 29.4.0 observation does not prove current installed compatibility.
The exact future run must independently re-establish the retained Engine ID
`ef049779-8946-48a9-a0a4-3d7e83c24e52`, version `29.4.0`, API range containing
1.51, cgroup v2 and the required limit capabilities or STOP before creation.
Primary references are Docker's versioned Engine API v1.51 and API-version matrix:
`https://docs.docker.com/reference/api/engine/version/v1.51/` and
`https://docs.docker.com/reference/api/engine/`.

The architectural invariant is **one trusted parent process, zero spawned host
control helpers**. At most two Unix connections may be live concurrently and at
most sixteen may be opened for the entire attempt. Each connection is opened to
the fixed socket only after a no-symlink socket-path identity check; the path's
device, inode, owner, group and mode must remain unchanged through the final
absence check. Future source review must reject process/helper imports and calls,
and a separate exact negative-test suite must make fork/spawn/exec/subprocess and
AF_INET/AF_INET6 construction terminal failures. Those checks would test the
zero-helper design; they would not establish the old CLI process group's absence.

**Immutable target and closed selection.** The parent binds only R2
`diagnostic.py` SHA-256
`fadd4a02cc1edbcd77de1d08ae750e6db8308294b14da26fbc4edb82442f9d42`
and `test_diagnostic.py` SHA-256
`ef5a5429e60486be688d1b718a0680a243d9cea6b8e074924f3e13b378e82c06`.
The contained prelude compiles those two bytes in memory and selects the same
closed 27 `DiagnosticTests` names above. It never calls diagnostic `main`, selects
the historical 41 or six R4 cases, discovers tests dynamically or falls back to
another suite. Transport regressions are a separately named and exact-counted
future suite; they cannot silently join the 27-test gate.

**Protocol allowlist and bounds.** The future parent may issue only the following
same-attempt transitions, each once at most. The list transition has separate
one-shot pre-create, optional ambiguous-create reconciliation and final-absence
instances. Inspect has separate one-shot pre-start and terminal instances; an
inspect used to adopt an ambiguously created candidate consumes the sole pre-start
inspect slot and moves the attempt directly to cleanup-only failure, never target
start. Query encoding, JSON keys and ordering are fixed by reviewed source. Any
other method, path, query, status or response shape is terminal failure:

| Operation | Required response and purpose |
| --- | --- |
| `GET /v1.51/version` | `200`; bind version and supported API range |
| `GET /v1.51/info` | `200`; bind Engine ID, cgroup v2 and limit capabilities |
| `GET /v1.51/images/{pinned-digest}/json` | `200`; bind the retained Linux/arm64 image digest |
| `GET /v1.51/containers/json?all=1&filters={exact-name,label}` | `200`; one pre-create absence, optional one ambiguous-create reconciliation, and one final absence, each a distinct state transition |
| `POST /v1.51/containers/create?name={exact-name}` | `201`; exactly one creation attempt, never retried |
| `GET /v1.51/containers/{id}/json` | `200`; exact pre-start readback and exact terminal readback |
| `POST /v1.51/containers/{id}/attach?logs=0&stream=1&stdin=0&stdout=1&stderr=1` | `101 Upgrade`; attach stdout/stderr before start, never use logs API |
| `POST /v1.51/containers/{id}/start` | `204`; `304` or uncertain delivery refuses acceptance |
| `POST /v1.51/containers/{id}/wait?condition=not-running` | `200`; retain daemon terminal status |
| `DELETE /v1.51/containers/{id}?v=0&force={0|1}` | `204`; `force=0` after proved terminal state, otherwise one failure-cleanup transition with `force=1` |

The normal acknowledged path opens exactly twelve sockets: version, info, image,
pre-create list, create, pre-start inspect, attach, start, wait, terminal inspect,
delete and final list. An acknowledged pre-attach refusal opens at most eight;
an attach-established refusal before start opens at most nine because attach is
closed before delete and final list. An ambiguous-create cleanup path opens at
most nine; an empty reconciliation stops at six and retains the stage because no
owned container ID exists. A start-outcome-unknown cleanup path opens at most ten.
A started timeout/failure path opens at most twelve. No path may exceed sixteen
total openings or two live sockets, and no transition is retried to consume the
unused ceiling.

The parent sends no registry-auth header and never uses image pull, build, exec,
archive, logs, events, prune or broad list endpoints. Request headers are fixed to
`Host`, `Content-Type`/`Content-Length` when applicable, and `Connection`; attach
alone adds `Upgrade: tcp`. A request header block is at most 8 KiB and create JSON
at most 32 KiB. A response header block is at most 16 KiB, at most 64 fields and
each line at most 4 KiB, with no folding or duplicate critical fields. The read
buffer is at most 8 KiB. Content-Length must be one canonical nonnegative decimal
within the operation's body cap. Chunked responses permit at most 4,096 chunks,
each chunk-size line at most 128 bytes; chunk extensions, `Trailer`, trailers,
mixed length/chunk framing and unsupported transfer codings are refused before
allocation. The sole zero-size HTTP chunk ends that body and cannot be followed
by trailers or payload.

Decoded error bodies are at most 8 KiB; create/wait bodies 16 KiB; version 32 KiB;
info, image and list bodies 256 KiB; inspect bodies 512 KiB; all decoded control
bodies together remain at most 1 MiB. Each ordinary response's raw-wire count is
bounded by its decoded-body cap plus 32 KiB; aggregate non-attach raw wire is at
most 1,310,720 bytes. JSON body storage never exceeds the operation's decoded cap
and is additionally bounded to depth 32, 4,096 nodes, 1,024 list members, 256
object keys and 64 KiB per string. Unknown fields may be ignored only where
Docker's open response model permits them, never in the parent's owned identity/
config projections. Raw-wire counters are retained as bounded integers only;
they are transport accounting, not another retained-output channel.

`log-driver=none` is preserved. Output is acquired only through the pre-start
hijacked attach stream with TTY disabled. Every multiplex frame must have stream
type stdout or stderr, three zero reserved bytes, an unsigned big-endian length
within the remaining capture budget and a complete payload. Unknown stream,
truncated header/body or output overflow is failure. Ordinary frames repeat until
frame-aligned attach EOF; there is no multiplex terminal frame. At most 4,096
frames and 114,688 attach raw-wire bytes are accepted. A zero-length multiplex
frame counts toward both ceilings and is never completion. Attach EOF is retained
independently and may precede the wait response; acceptance requires frame-aligned
EOF, a complete acknowledged start, complete wait response and matching terminal
inspect, regardless of EOF/wait ordering. Target stdout/stderr plus retained
control diagnostics share the existing 64 KiB ceiling: at most 48 KiB target bytes
and 16 KiB control bytes. Combined attempt raw wire is at most 1,425,408 bytes;
only the bounded decoded output prefix and counters enter retention. Client
timeout, EOF or connection close is observation only, never cancellation, target
terminality or cleanup proof.

**Creation, execution and cleanup binding.** Before creation the parent seals a
128-bit lowercase-hex attempt nonce, exact name, exact labels, R2 identities,
image digest and monotonic deadlines. Creation returns one canonical 64-hex
container ID. Every mutation begins `not_sent`, becomes `acknowledged` only after
the complete expected response, or becomes permanently `outcome_unknown` for the
attempt if request delivery/commit may have occurred without that response.
Closing a client connection is not cancellation. Creation is never repeated. Its
one reconciliation list may return zero or one exact name+nonce-label match, but
zero is only an empty point observation: it cannot prove no late create, absence
or cleanup and leaves the attempt/stage blocked for separately reviewed recovery.
One match may be adopted only after the sole pre-start inspect fully binds it; the
attempt remains unusable and proceeds to exact-ID cleanup only. Multiple,
mismatched or uninspectable candidates are ambiguous ownership and STOP. A start
or delete with an incomplete response similarly remains `outcome_unknown`; later
readback/absence may support cleanup evidence but cannot restore acceptance or
prove cancellation of the mutation. Container ID, name, nonce label, packet
labels and image digest remain immutable for every later request.

One global `recovery_required` latch is set by any unresolved mutation outcome or
unproved resource cleanup, including create/start/delete uncertainty, parent or
daemon loss, ambiguous ownership, failed/uncertain deletion, nonempty or
untrustworthy absence observation, stage-cleanup failure and publication failure
after a possible effect. While set, it retains the nonce and any known container
ID, preserves every surviving stage object and identity, and forbids every
subsequent attempt. If stage removal already completed, it retains the recorded
removed-stage identity and removal evidence rather than claiming an extant stage;
partial or unknown removal remains explicitly unproved. An unresolved create with
possible late effects must keep its still-existing read-only stage, and an empty
observation never justifies stage removal. Only a separately reviewed exact-
attempt recovery may establish closure of all possible late effects and clear the
latch. Independently obtained absence observations may be inputs to that reviewed
recovery, but never independently clear the latch or authorize another attempt.
An empty snapshot, client close, elapsed time or unusable-result status never
clears it. This design authorizes no automatic recovery, cleanup action or
additional observation.

The exact create/readback contract preserves: no network; read-only rootfs and
one read-only staged-input bind only; no host socket, home or credentials in the
target; UID/GID 65534; `CapDrop=[ALL]`; no added capabilities; no-new-privileges;
private cgroup namespace; IPC none; restart none; no healthcheck; no ports,
devices, writable mount or tmpfs; memory 536,870,912 bytes with memory-swap equal
to memory; one CPU; pids limit 1; and rlimits nofile64, nproc1, core0, fsize0 and
CPU10s. The clean target environment is exactly `PATH=/usr/bin:/bin`, `LC_ALL=C`
and `HOME=/tmp`; no secret or ambient environment is admitted. Image remains
`sha256:934240a162082fd8b8a2f90cd5114446443f1eba1c5378f6687167ca405e6584`.

One absolute monotonic attempt starts at `t0`, fixes work cutoff `t0+25s` and
final cutoff `t0+30s`, and never renews either deadline. Attach socket A is opened
before start. Start uses socket B; its full `204` is retained, then B is
unregistered and closed before wait opens a new socket B. One nonblocking selector
owns A and B and retains attach frames/EOF plus the wait response. Once both are
complete, wait B and attach A are unregistered and closed before terminal inspect
or cleanup opens a control socket. On timeout/interruption or unresolved start/
wait evidence, both are unregistered/closed after retaining the bounded output
prefix, thereby freeing a slot for the one cleanup delete. No third live socket,
delayed cleanup reserve or timeout-as-cancellation is allowed. Start, wait and
attach must all reconcile before normal removal. Terminal success additionally requires
wait status zero, inspect state exited/not-running/not-dead/not-OOM, exit code
zero, empty Engine error, exact 27-test markers/counts, complete bounded output,
normal `force=0` deletion, exact name+label absence, then removal of the unchanged
read-only stage by retained device/inode identity. Timeout or controlled
interruption closes acceptance immediately and permits at most one `force=1`
delete under the exact ID/name/nonce binding. An uncertain delete remains sticky
`outcome_unknown`; its final list is evidence of the observed state only, never
proof that a late mutation cannot occur. A failed or uncertain delete, nonempty
final list or stage-removal failure is retained failure, never success.

The attempt/result/log carriers remain exclusive, bounded, fsynced and sealed
before visibility. Success is still provisional and unusable until a separately
reviewed outer action independently observes the exact trusted-parent terminal
status and elapsed bound, then verifies sealed retention plus container, stage and
attachment absence. Parent SIGKILL, host loss and daemon loss remain `not_proved`:
they cannot publish success, and a subsequent attempt is forbidden until a
separately reviewed exact-attempt recovery closes every possible late effect.
Independently obtained absence observations may inform that recovery but cannot
clear `recovery_required` by themselves. Engine/container generation evidence
does not prove host-parent survival or the historical CLI group's quiescence.

| Observation | Required disposition |
| --- | --- |
| socket/API/version/schema/config mismatch | STOP before target start; retain exact phase, never negotiate/fallback/retry |
| creation response lost | sticky `create_outcome_unknown` plus `recovery_required`; no second create; one exact lookup may adopt one match for cleanup, but an empty lookup cannot prove no late effect or permit another attempt |
| start/delete response lost | sticky mutation-outcome unknown plus `recovery_required`; later state may guide exact cleanup but never restore acceptance, imply cancellation or permit another attempt |
| attach/start/wait framing or status mismatch | unusable failure; retain frame-aligned EOF/wait/start state and enter exact-ID cleanup reserve only |
| target nonzero, malformed markers, OOM, dead/running or terminal mismatch | unusable target failure; retain output/status and clean exact ID |
| deadline or controlled interruption | close acceptance; one exact-ID `force=1` delete, absence check and bounded retention |
| parent SIGKILL/loss or daemon loss | no success; set `recovery_required`; cleanup not proved and no new attempt pending reviewed exact-attempt recovery |
| deletion, absence, stage cleanup or publication failure after possible effect | sticky failure plus `recovery_required`; no usable result, retry, alternate owner or subsequent attempt |
| all protocol, target, retention, parent-terminal and absence predicates pass | provisional contained 27-mock result only; `runtime_accepted:false`, `step7_proved:false` |

**DESIGN R5 acceptance and implementation gate.** Aquinas exact verification turn
`01a13006-f8ac-75a0-988a-f8a87aaa8d89` returned GO: DESIGN ONLY with no remaining
P1/P2/P3 in the exact R5 clarification delta. MUSE final substantive reconciliation
in turn `01a13005-ae5c-7fa2-9634-677e212593bd` accepted the complete R5 design
only. Accepted pre-stamp identities were plan
`caca888078f05b18538747349c750078eb1466f99814392747efd69a4a2ab3dd`
and current-state
`bf7b761115d22f58da4bb6d5eeb7a7535f87f23bc2525772f553a1a3ba47eec3`;
MUSE independently rehashed both after review. This closes the documented R2-R5
design findings only, not a full security audit or any source/runtime gate.

If fixed direct transport cannot preserve
the operation/schema/framing/output/time/cleanup constraints above, the route is
HOLD with the exact gap; no alternate owner, host, engine or larger budget is
substituted. Later source authoring needs its own exact clearance and frozen
identities. Parsing/import/testing or Engine access needs separately reviewed
source, invocation and controls plus fresh human action approval. All 27+41+6
remain NOT_RUN; effective containment, parent-loss cleanup and current installed
API compatibility remain NOT_PROVED; Step 7 remains OPEN/R1 BLOCKED and Step 8/
V12.2 remain inactive. This private design changes no public product, CLI, schema
or claim, so Site, Skills, Examples, Learn, FAQ and Glossary do not propagate.

#### Native Rust Fulfillment Prerequisite: Source Candidate / Validation HOLD

**Status / ownership.** ONE SOURCE implementation increment, owner SCOOBY.
MUSE cleared sequencing in `01a127d7-d01c-70e2-a505-703af25a56e4` after the
qualified VLAD call-chain review `01a127d2-e1cf-70c0-8904-9ac75f75cfe8`.
The two required amendments (bounded probes and complete ordinary-run outcome
table/cache separation) were incorporated before implementation. The accepted
Cargo checkpoint remains the historical source review record; runner bytes now
include this new prerequisite and require combined independent review. No
validation-driver execution or publication is authorized.

**Named gap: satisfied-toolchain provisioning avoidance.** The declared Core
check closure selects setup, and both select Rust fulfillment `mode: run`.
Before this candidate, `src/runner.rs` called `maybe_fulfill_toolchains_on_run_path` without a readiness
guard; that function executes the provider's fulfillment commands once per
runner state. `rustup_fulfillment_commands` in `src/toolchains.rs` produces
`rustup toolchain install 1.95.0 --component rustfmt`. A run-state cache prevents
duplicate invocation, not the first invocation for an installed toolchain.
This provisioning-capable invocation conflicts with the proposed no-install
validation gate. SOURCE inspection does not demonstrate actual reinstallation,
network contact, or failure of the installed Rustup executable. Neither the
prior Cargo checkpoint nor the retained driver's source revision
`63297ad95ef156b335a1dfb54090146406077a3f` has a satisfaction guard here.

**Smallest source-staged product correction.** Restrict the first increment to direct
native Linux/macOS Rust with canonical Rustup and an exact version reference.
Reuse selected-task/backend/environment resolution, the provider-owned runtime,
tool and managed-surface requirements, and existing version/entry parsers where
sound. Do not call all-repository Doctor as a run admission substitute or refactor
other providers. Introduce an internal satisfaction decision immediately before
the existing fulfillment dispatch, not a new contract field or parallel runner.
Only a positive, invocation-local decision may skip provisioning:

- Resolve the actual selected execution, OS overrides, required Rust version,
  components and targets. Check explicit profile requirements if supported;
  otherwise profile-bearing, channel or non-exact references are ineligible for
  this skip rather than being guessed satisfied.
- Bind the effective preparation environment and resolved Rustup, rustc, Cargo,
  and required component executables. Positive version text from another PATH,
  default toolchain, backend, working directory or invocation is insufficient.
- Check the exact requested installed toolchain and its components/targets with
  explicitly selected, read-only provider probes. Doctor's current unqualified
  `rustup component list --installed` / target probe strings cannot be copied
  unchanged: default-toolchain success must not satisfy a different selection.
  The actual pinned provider's no-install probe behavior remains NOT_VERIFIED.
- Return distinct satisfied, unsatisfied and unknown/ineligible outcomes. In the
  ordinary authorized `mode: run` path, satisfied alone skips; unsatisfied and
  unknown/ineligible retain existing provisioning without claiming readiness.
  Cancellation or failed owned-child cleanup is a terminal error, not fallback.
  The separate no-install route remains unqualified and must refuse every
  non-satisfied outcome before provisioning. No new permission is conferred.
- Keep selected bindings and result/failure attribution scoped to this invocation;
  do not fabricate a fulfillment action when none ran or mark failed probes as
  completed fulfillment. Existing non-native/other-provider behavior stays intact.
  A skip must not populate `fulfilled_toolchain_keys` or `fulfilled_toolchains`;
  reobserve later readiness even in the same TaskRunState. Retain deduplication
  only for actual provisioning, independently of satisfaction.

Native readiness probes use exact argv, the resolved executable and effective
preparation environment, never unchanged ambient-path/piped `Command.output()`
transport. Require successful status before parsing. Limits: 5 seconds per probe,
20 seconds aggregate for one satisfaction decision including cleanup (reserve
the last 2 seconds), 8 KiB per regular-file stdout/stderr sink and 64 KiB combined
across probes. Bound regular sinks before exec; terminate the owned process group
while its leader remains unreaped, then acquire status without an unbounded wait.
No later dispatch follows cancellation or cleanup failure. These are ordinary
native observation controls, not escaped-descendant/custody/attestation proof.
The staged source conservatively supports only no component or `rustfmt`, and
refuses to acquit nonempty `RUSTC`, `RUSTC_WRAPPER` or `RUSTC_WORKSPACE_WRAPPER`.
Rustup-selected and active installed executable paths must agree with the launch
selection; rustc release/host, Cargo version and host-qualified rustfmt/declared
target entries must match. Pre/post file metadata and environment observations
do not establish acquisition or absence of concurrent/ABA substitution. Provider
probe behavior with `RUSTUP_AUTO_INSTALL=0` remains runtime-NOT_VERIFIED.
Qualified VLAD source review `01a127f0-60ce-7b01-8e5d-0a33ffa50d35` found implicit
Rustup exclusion (P1), pre-final launch-context mismatch (P2), and fixture ambient
compiler inheritance (P3). Repairs reuse canonical provider fallback, retain
explicit alternative-source ineligibility and restore ambient compiler variables
after isolated fixtures. The context gate is deliberately conservative: only
structured Rust commands in the repository directory with no runtime projection,
explicit +toolchain selector, late contract environment/source, task env files/
service bindings, inputs/targets or native activation may borrow preparation
readiness. Unsupported contexts stay unknown and follow ordinary provisioning;
no early secret/environment activation or execution reordering is introduced.
Core's declared PATH overlay is therefore still ineligible. Qualifying that
exact driver requires separately binding its final context, not claiming this
source repair already closes the offline route. Qualified VLAD R2 recheck
`01a127f6-bd87-7e30-a7f3-562db8c5be33` found no remaining P1/P2/P3;
MUSE accepted this bounded source checkpoint in
`01a127f6-bdf6-7c00-bd6e-86cf4f946324`. All eleven regressions remain NOT_RUN;
source acceptance does not complete the Core prerequisite.

**Next cleared SOURCE IMPLEMENTATION: Core literal PATH binding.** Owner SCOOBY;
RUBY's bootstrap/admission inventory is complete. MUSE accepted the design with
the final-invalidation amendment below in `01a127fe-0a14-7c83-a1b2-b4af2b2e21ad`
and cleared ONE source increment, not execution. Core `ota.yaml:44-49` declares only
literal absolute PATH prepends; its setup/check are structured Cargo commands in
the repository directory. Keep all nested/dynamic/input/target/activation guards.

- Reuse `compose_path_value` in `src/runner.rs` for literal PATH composition,
  not a second PATH parser. Accept only the single non-secret PATH requirement
  without defaults/allowed values/source lookup or other declared variables;
  retain an exact UTF-8 base PATH and fixed literal absolute entries. Unknown
  base, template/dynamic entries or unsupported requirements stay ineligible.
- Bind both preparation and prospective final task contexts. Use the canonical
  selected task environment and the same pure composition function; never call
  the full environment/policy/source resolver early just to obtain readiness.
  Existing later resolution prioritizes task, workspace/repository policy,
  ambient and declared-source/default values. A policy-selected base must not
  silently be replaced by ambient PATH in a prospective probe.
- `resolve_task_env_details_with_policy_and_required_env_names` is not a pure
  pre-probe shortcut: it loads policy/environment sources, and
  `load_org_policy_pack_auto_details` can fetch remote policy. The existing
  internal absent-policy snapshot marker is scoped to a runtime-proof child;
  it is not demonstrated as the normal Core check's acquired policy context.
  Do not fabricate that marker or acquire new policy/network state for readiness.
  Carry an already acquired canonical policy/absence decision if the existing
  run admission provides one; otherwise classify this missing non-acquiring
  context input precisely rather than guessing policy absence.
- Execute/readiness-bind the actual launch resolver path and independently
  compare resolution under the composed PATH. Native structured launch resolves
  the executable before applying environment overrides; changing PATH text alone
  does not change that selected executable. A prepended shadow binary or
  source-managed path must not borrow the original Rustup observations. Retain
  requested/active toolchain, exact version/component/target and file/env checks.
- Before task dispatch, reconcile the observed selection against the actual
  final environment and working directory produced by existing canonical
  resolution. Positive pre-probe success is not final dispatch admission. If a
  provision-skipping witness is invalidated, fail before task dispatch with
  original attribution and partial evidence; no late provisioning, retry, failure
  hook or continuation. Ordinary unknown/unsatisfied fallback remains exclusively
  at its existing preparation point, without cached/fabricated fulfillment.
  Cancellation/cleanup refusal remains terminal. This consistency check is not
  acquisition/ABA or authority proof and is not the separate no-install gate.
- Required future source cases: Core-shaped literal PATH positive using canonical
  default Rustup, prepended shadow/mismatched toolchain, different or unresolved
  policy/ambient base, source-managed replacement, nested cwd, compiler/late-env
  override and same-state invalidation. Compilation/execution remains NOT_RUN.

**Accepted caller-to-runner handoff.** MUSE identified, and SCOOBY inspected,
the existing normal run seam: `src/cli/commands.rs:113106-113125` retains policy
through `CommandTypedEffectAdmission` and
`task_replay_input_preflight_with_retained_policy`; `TaskReplayInputPreflight`
retains `loaded_policy` and `policy_load_error`. The helper at `22693-22718`
either uses a genuinely observed result or performs the existing normal load,
retaining its error. The execution caller at `115404` invokes
`run_task_with_args_with_overrides_and_stream_capture`; private runner request/
state initialization can carry this already acquired result through the existing
run path to native readiness. Preserve public/legacy callers and distinguish
known-present, known-absent, load-error and unavailable context: an optional value
alone is not observed absence. Do not invoke Doctor, globally snapshot policy,
change final resolver load order/freshness, or acquire policy again for readiness.
Author a full Core-shaped setup/check source fixture that uses this retained
known-absence handoff rather than a forced helper boolean; include policy PATH
precedence/error/unavailability and acquisition-count/order assertions.

**Source candidate authored, pending independent review.** Normal streaming and
captured single-repository CLI paths project their existing preflight acquisition
into private invocation state; present env values, acquired absence, load error
and unavailable legacy callers are distinct. The projection performs no load.
Pure PATH preparation reuses `compose_path_value` and final precedence; normal
final env/policy resolution is unchanged. Qualified VLAD source review
`01a1281a-2a88-7d62-b981-9b4b7dac446a` found P1: preparation composed optional
unselected PATH while final resolution left it inherited. R2 reuses the pure
canonical selected-variable helper in both paths; no optional variable is made
required. Positive full-run coverage now explicitly separates selected and
unselected PATH; precedence/shadow/fresh-policy negatives select PATH, avoiding
refusal for the wrong reason. Qualified VLAD R2 recheck
`01a1281c-f107-7f03-8b5e-28ecd8146b9d` found no P1/P2/P3; MUSE independently
accepted this SOURCE checkpoint in `01a1281d-854c-7a12-8ecf-58b13e64cd67`.
Historical reviewed R2 packet (not hashes of subsequent record updates):

| File | SHA-256 |
| --- | --- |
| src/runner.rs | 32fba28c5e6b3b601e0696befb7154ae9d069f929deb086e69c5f2d223433b94 |
| src/cli/commands.rs | 96ddab720a45c80db470d04f618a27cf62430c1cd23272d0589e271c8f09d111 |
| docs/planning/execution-contract-follow-ons/plan.md | 195cc79509663fd752977d872dc2a157d197f192dd3a4a3ac9481da9ee5c0682 |
| docs/ai/current-state.md | ce3e00d227ac70e7fc2b8b87e94ccb3de3651dbcc1efeef0abc429f6e95dee1e |
| docs/spec/command-reference.md | 0b376bc0e7bdb1aeac0eb06e6babcfa69aadc011626772e33445ae5218ad8b76 |
| docs/spec/contract-reference.md | c6c3c00810f565d02df80d2ee1be39bcc8b37f2a8595b5558fc5954c9654417f |
| CHANGELOG.md | 2997085440a1a7b0e49b736d9aa8578cae6a3cfb4329eb206a7a1cf94e18691a |

A provision-skipping witness retains
launch/installed objects, cwd and the original environment/probe budget. Before
body dispatch it compares actual final env/executables and re-observes active
Rustup selection within the original deadline/output budget, not a fresh budget.
Invalidation or terminal probe cleanup fails without provider/retry/failure hooks;
the existing error wrapper retains preceding Cargo hydration observations. This
does not add general prerequisite evidence or prove acquisition/ABA safety.
Six additional regression sources cover full Core-shaped setup/check through
genuinely acquired absence, retained present/workspace PATH precedence without
reacquisition, unavailable/load-error distinction, shadow/managed precedence,
final active-selection change and fresh final-policy change. All seventeen
Rust readiness regressions plus the CLI retained-policy assertion are NOT_RUN.
Core command/contract docs and Unreleased are updated; affected Site/Skills
propagation remains OPEN before shipping. No public schema/JSON/flag or
Protocol/Launcher/pin change. The deny-rule implementation below is a new separate
source increment, not silently included in the accepted PATH packet.

**Implementation acceptance.** Source authoring only; Core docs/Unreleased and
propagation assessment must accompany the extension. Preserve accepted Cargo
semantics, protected files, other providers/backends and ordinary fallback. Freeze
the combined candidate for qualified VLAD review and MUSE acceptance; all compiler,
runtime and regression execution remains NOT_RUN. No public shape/flag, general
environment redesign, early side effect or new permission flow is cleared.
RUBY may separately propose the provider-action restriction surface at design level
only, using his existing inventory rather than duplicating this implementation.

**Independent bootstrap/admission inventory.** RUBY's read-only investigation
identifies two concrete gaps; SCOOBY reconciled the relevant runner/policy/source
build paths and exact file identities. These are validation-route blockers, not
new findings that ordinary authorized Run is unsafe by definition.

- No inspected existing gate admits positive invocation-local Rust satisfaction
  while refusing every later unknown/invalidated selection immediately before
  fulfillment dispatch. Agent-safe task/claim admission is not that predicate;
  provisioning policy selects runtime/tool sources, not this conditional refusal.
  Whole-task denial, network denial or read-only toolchain storage cannot prove
  refusal before an install invocation. A narrowly owned dispatch-refusal design
  needs separate review; no new permission/schema/flag is inferred here.
- `.github/actions/install-ota-from-source/action.yml` is a real source-build
  implementation, but installs Rust (default `stable`), runs Cargo install and
  retains no exact compiler/build-input manifest or binary digest artifact.
  Branch smoke/release-gate builds install in-job rather than exposing a qualified
  bootstrap-only retained driver. The release publication graph is main/tag gated
  and includes additional publication side effects; it is not a branch-bootstrap
  shortcut. Linux systemd proof combines building with proof execution and retains
  proof data, not a independently qualified driver binary. None is an admitted
  no-install route, and HEAD alone omits the uncommitted repair.

Recommended sequencing: settle pure PATH/final-selection and missing conditional
dispatch-refusal ownership at source first; then separately review a bootstrap-only
first-party build using existing implementation plus the minimum necessary artifact
retention. Required build inputs include the exact reviewed source packet/commit,
Cargo manifest/lock/pins, compiler/Cargo/components/target/features/profile,
hydration/network/cache provenance, compiler/build-script isolation, binary/archive
digest and retained logs. Publication/commit/push, CI dispatch, download/install
and offline invocation each retain their applicable authority gates. Do not route
this through release/main mutation, an old driver, direct local Cargo bypass or a
fake no-install assertion. Installed/cache/build/runtime behavior remains NOT_RUN.

**No-install validation is a separate admission bar.** Missing, mismatched or
unknown readiness must STOP before any provisioning invocation in this route.
Ordinary authorized `mode: run` fallback is not permission to use it here. A
preflight result alone is not that enforcement: the exact executing driver must
honor the verified selection at dispatch, and a qualified existing refusal gate
must stop an invalidated/unknown selection before fallback. That gate and its
effective-environment binding have not been demonstrated. Do not invent a policy
flag, fake Rustup success, wrapper/no-op shim, readiness cache, changed protected
fulfillment mode, or Rustup offline environment setting to bridge this gap.
If existing admission cannot express this boundary, return the exact missing
mechanism for separate design review rather than silently extending this slice.

#### Rust Fulfillment Restriction: Accepted Design / Source Implementation Active

MUSE selected `policies.toolchain_fulfillment: {rust: deny}` in
`01a12806-da1b-7b21-bba5-39393db3bae9`. This ONE source-design refinement is
separate from the cleared Core PATH implementation. SCOOBY owns the canonical
plan; RUBY independently checked entry-point ordering. The initial grant was
design-only; the separate source implementation activation below supersedes
that grant without admitting execution or publication.
Qualified VLAD R2 review `01a1280f-d34f-7163-b9c6-5fb584b3ba0f` found no remaining
P1/P2/P3 after adding upfront proof-parent refusal; MUSE independently accepted
R2 design and sequenced the already-cleared Core PATH source increment first.
Reviewed design hashes are historical: plan
`eac68d8d510126e2a1001a767b89841c196cf2f626d9c22ff1ba385919149674`, handoff
`57efa63d66eb89d4c291942b1365e35c396ad243cf1a3162fcd403ba574bebae`.
After accepting Core PATH source, MUSE explicitly activated ONE deny-only SOURCE
implementation increment in `01a1281d-854c-7a12-8ecf-58b13e64cd67`. Extend the
existing acquired projection with the typed rule and canonical policy identity/
source, not another acquisition flow. Required scope includes before-cache/provider
denial, original root/dependency attribution and partial refusal evidence, all
unsupported parent guards, typed policy validation/review, Doctor/preview,
affected receipts/references/Unreleased and authored NOT_RUN regressions. SCOOBY
owns integration; RUBY may own only an agreed disjoint policy_pack slice. Freeze
the combined source for qualified VLAD review then MUSE acceptance. Any genuine
ownership/order conflict returns to MUSE before dependent implementation; no
workaround, extra loader/marker or implicit no-rule bypass is permitted. RUBY
authored the policy_pack-only types/accessor and five NOT_RUN test sources;
SCOOBY authored retained restriction/identity/source/path projection and two
NOT_RUN test sources. The in-progress source now adds before-cache denial,
partial receipt attribution/context, explicit acquisition state, ordinary Run
retention, non-activating protected-carrier selection and restricted-route
refusal before full grant evaluation, lifecycle/runtime parent restriction
guards and static Doctor/preview notes. Additional source regressions and manual
policy-review/receipt schema coverage are authored, NOT_RUN. Up/workspace/member/
canary parent coverage, complete regression coverage, propagation and concrete
independent acceptance remain open; the increment is incomplete. The concrete
checkpoint was held by qualified VLAD `01a128af-848f-7b40-b28a-5e2ebbe80cb5`
for two P2s: structured early-refusal evidence loss and missing integrated
automatic V4 policy-deny/load-error source coverage. MUSE cleared their narrow
SOURCE repair only. A distinct route/root/task/workflow/policy attribution
carrier is now staged in requested Run receipts and proof admission JSON;
canonical receipt observation is explicitly disabled, so refusal rendering
cannot probe/read execution inputs or acquire authority. New actual-path V4
regressions use descriptor/PATH/store sentinels, challenge absence and zero
capability/activation counters. Serialization and connected manual schemas
retain negative evidence without fulfillment/readiness/authority promotion.
ALL NOT_RUN. Qualified VLAD R2 `01a128ba-ce0d-7333-bda9-00ca408e6886`, final
`msg_0074fa586a19efbe016acaf0b5226081919dc307913bd14a7c`, found no remaining
P1/P2/P3; both P2s are closed at SOURCE level only. MUSE independently reconciled
HEAD/all 13 hashes and accepted this bounded checkpoint in
`01a128ba-d1c4-7812-a06b-443f046e85b5`.

Historical R2 acceptance identities (before the record-only acceptance update):

| File | SHA-256 |
| --- | --- |
| src/effect_orchestration.rs | 9e980ea21f17d8613c8e37d86da44321e254c93a2591824f48321fb4a79dac02 |
| src/cli/commands.rs | 43d87f960ec7a561fd49fdfe7ea5aa62779c7c334e1cdb53bd84e941e021e218 |
| src/runner.rs | 3b4774ac583d9635835db26f2b4153b97e7031f31bf0bf86257a33be8ba29ac4 |
| src/policy_pack.rs | 91e3b09e60d82cfe67696d984ab20015a950b75b474d85eb5b0442c97a311a67 |
| src/output.rs | 38459d38b8a239bcea1efb8450be184785c9cca844cabd42708850fffb7c3f7b |
| src/doctor.rs | caa3636989cc635d8a06df40c6798b420550c7c3c8608d58da74160221ae8f24 |
| docs/spec/json-schemas/policy-review.json | aeeeeefa99bc3a357670ee39d3d3400c383fa87686de449803487b323489dd86 |
| docs/spec/json-schemas/receipt.json | 47de62878ccc99232a519a72df7182670286749315ca644529013a4fb964d4de |
| docs/spec/json-schemas/proof-lifecycle.json | caccc040961555306b674885cf12a31e7888777658ba0ef47f04fae2b634af72 |
| docs/spec/json-schemas/proof-runtime.json | 6d0fa20f4b133fade91764b648c871787b4ab9037ac8a982650c7249d5326439 |
| tests/json_schema_contracts.rs | e6c54b03bcbf26df015931302b7f8b4980148a0c5daef86df3feb403a0323151 |
| docs/ai/current-state.md | f0969d37d568c9411ca05fa652e10b7a50b3d8389a3af0df47475738a4808bb2 |
| docs/planning/execution-contract-follow-ons/plan.md | 4831d6a898547781bcbecb08fe7b8eacf23368f2192c182dade06837b332de22 |

**Accepted checkpoint: Up admission-handoff/selected-local-parent guards, SOURCE only.**
SCOOBY retains one owned successful admission through the canonical Up grant
orchestrator, lower replay preflight and same-context instance prerequisites.
Use existing from-admission refusal rendering; do not discard/reload or infer
acquisition from typed applicability. Move frontend Mise behind admission;
collect/validate selected local root/member targets and their actual separately
owned policy admissions before the first effect. A later deny/error cannot
arrive after earlier target execution. Captured Rust restriction refuses
unsupported Up before readiness, replay, full grant/capability handshake,
authority, task/hook/provider/backend/artifact work. Preserve pure protected
carrier precedence and distinct structured/non-observing refusal; new route
denial creates no typed-effect archive eligibility. Include directly reached
Up dry-run/canary/archive paths. Author NOT_RUN regressions for acquisition
reuse/removal, absence/error, root/later-member refusal before effects,
same-context prerequisites and no-rule/protected behavior. Workspace acquisition
edits remain deferred; no all-parent parity or whole-slice acceptance. Freeze
candidate/phase map for qualified VLAD then MUSE before widening further.

The candidate now owns successful command admission instead of dropping it.
Frontend monorepo and explicit multi-member paths retain separately acquired
target/admission pairs before Mise, replay baseline, root/member execution or
crossing admission. Lower Up replay preflight receives the retained pack,
including actual absence, and same-context instance recursion clones only that
matching owned pack. Captured/streamed prepare/setup/run/attach and their runner
dependencies/hooks receive the canonical retained runner projection while
preserving the separate policy environment overlay and existing terminal flags.
MUSE cleared these directly reached in-process seams in
`01a128c7-c39e-7d32-ba20-f11decb8a6c5`, not cross-process transport.
Admission/refusal construction is non-observing; refusal previews are declaration
only. Up canaries and eligible existing typed archives use from-loaded evaluation,
not a fresh loader. Rust route denial remains ineligible for that archive.
This is phase-scoped retention, not global full-pack no-reload. Qualified VLAD
Up R1 `01a128ce-2a08-7723-bce5-7e863982fa11`, final
`msg_0074fa586a19efbe016acaf63b21ec81919a628363f94487db`, held source for a
stray `..` static type error and incorrect source-change/overlay test fixtures.
MUSE confirmed the accepted fresh-final-environment boundary and authorized only
R2 source repair: exact token removal, declared/isolated overlay fixtures, stable
policy success cases and separate changed/missing/malformed-policy failure cases.
Final resolver load order/freshness and canonical loader semantics are unchanged.

| Up path | Policy owner and handoff | First effect / boundary |
| --- | --- | --- |
| Local single target | Actual command acquisition; full pack to replay, projection to runner | After guard; no acquisition inferred from typed applicability |
| Monorepo root / selected local members | Each actual target owns its admission; all retained before first target | No earlier root/member execution before a later deny/load failure |
| Captured/streamed prepare/setup/run/attach, dependencies/hooks | Same matching admission/readiness/restriction projection; policy_env kept independently | Final environment/backend evaluations remain fresh; not all-policy no-reload |
| Same-context instance prerequisites | Clone matching replay-owned pack for admission/replay/readiness | Final loaders remain fresh; no parent absence substituted for a different root |
| Final task environment resolution | Canonical `resolve_task_env_details_with_policy_and_required_env_names` / `load_policy_env_overlay` load fresh policy | Later than parent admission; failure before body/hooks does not prove zero earlier Up effects |
| Selected backend fulfillment | Canonical `maybe_fulfill_backend_requirements_on_run_path` loads fresh policy when its plan is selected | Backend-preparation phase; not replaced by the readiness projection |
| Shared-backend environment selection | Canonical shared-backend environment selection loads fresh policy for actual contract path | Runner startup/preflight; failed load refuses before task body, not before all Up effects |
| Dry-run / admission refusal | Captured restriction or load error retained; static declaration-only output | No readiness/replay sampling, capability adoption or preparation |
| Safety/effect canary / eligible typed archive | Reuse acquired policy; existing challenge/archive rules still apply | Restriction refusal creates no canary success or archive eligibility |
| Detached child | Parent guard before artifact/log/spawn; ordinary child acquires fresh policy | Separate invocation; parent-snapshot continuity NOT_PROVED |
| Workspace parent/acquisition | Deferred, not covered by local member guard | OPEN; no all-parent parity claim |

NOT_RUN regression sources cover root/later-member denial/error before effects,
retained descriptor/PATH/no-challenge under all listed guard routes including
detach, acquired no-rule pack after removal, actual absence after policy change,
both in-process modes with declared/isolated overlays/dependencies/hooks under
valid stable policy, and matching instance prerequisites. Acquired-present/absent
source-change assertions remain at admission/replay seams only. Separate NOT_RUN
cases preserve fresh final-policy failure under missing, parse-invalid and
validation-invalid sources before task bodies and hooks, without asserting zero
earlier Up effects. The readiness restriction projection is not fresh final
environment evidence and is not a substitute for backend policy evaluation.
These sources are not compile/test results. Detached child policy continuity is
an explicitly bounded owned follow-on: fresh child acquisition exists, but no
qualified parent-snapshot transfer. Any future carrier needs a separately reviewed
trust/custody/precedence design; no snapshot file, environment absence marker or
authority IPC is added here. Site/Skills/Examples/Learn/FAQ/Glossary propagation
remains deferred before shipping; existing receipt/schema carrier suffices for
this Up use, and no new public flag, ota.yaml shape, Protocol/Launcher or pin is
introduced. Qualified VLAD Up R2 `01a128d6-6c99-7472-bd1c-2b57197ce5cf`, final
`msg_0074fa586a19efbe016acaf7a398548191a9bab58c1469bb54`, found no remaining
P1/P2/P3 in the bounded repair and closed all three R1 findings by source
inspection. MUSE independently reconciled and accepted this source checkpoint.
Whole-slice acceptance, compilation/tests/runtime, public propagation and
bootstrap remain OPEN/HOLD. Reviewed pre-handoff HEAD:
`8926ddb1fc0e4a225059c48375fc6e1185081b1e`; historical SHA-256 identities below
precede these acceptance-record updates and do not assert runtime qualification.

| Up R2 reviewed file | Historical SHA-256 |
| --- | --- |
| src/cli/commands.rs | b4a3a7aa835fb26bf6af0c435c69d96b77996341c399d1c03dd36cf3820bf549 |
| src/cli/commands/effect_refusal.rs | a1e3c01cdfdac0723b3147452754b3a262fd4aa5aeccecad52f3b7d87d0a9cdd |
| src/runner.rs | f6d295e2e3316c095727e080b399f3ead4c48c1325f9a94122fea23c64c28bc9 |
| docs/spec/policy-packs.md | 008ece0740a3955df90f028cbc54cb5aee4005621fa78139314a2d2e699310cd |
| docs/spec/command-reference.md | 8ef7211dc91f71f42affa11912aa604e481877b7124a36c92be2f4607cf33aa2 |
| CHANGELOG.md | 603e015e06844536cfb0bc71a7a81f13d11fdccf3aa99b5d21261e2bd0d0c002 |
| docs/planning/execution-contract-follow-ons/plan.md | 8a86b46784a17866e759431e09848ec9a1059f985582e82222c41819e6044684 |
| docs/ai/current-state.md | 580d3812665fcfff66fb9c3e7a206331e7824714033b84eae2b17f7d6d305220 |

**ACTIVE: workspace-parent admission/acquisition-guard SOURCE checkpoint.**
MUSE accepted DESIGN R2 after qualified VLAD recheck; the design and original
source map below are historical reviewed evidence, not current source acceptance.
ONE integrated resolver/parent-guard/negative-output/regression/schema/Core-doc
increment is permitted. SCOOBY owns production and freeze; RUBY's two reserved
Core-doc files have returned to SCOOBY. All execution/action gates remain HOLD.
Historical design scope:
MUSE selected ONE captured/streamed workspace up/run design increment; SCOOBY
owns architecture and is sole shared writer. RUBY may supply a disjoint read-only
precedence/coverage inventory. Existing source is frozen except these records;
implementation remains HOLD until qualified VLAD then MUSE clear the design.
The design must map actual parent/child policy owners/outcomes and first
acquisition/progress/worker/mkdir/clone/effect, propose the smallest canonical
resolver/handoff and negative-carrier changes, and guard captured nonempty Rust
restrictions/applicable parent load errors before worker or acquisition effects.
Acquisition-only children cannot establish upfront policy merely by later cloning.
Explicitly resolve selected custom `--file` policy precedence: OTA_POLICY,
repo/ancestor policy, selected workspace.policy relative to its actual file,
and any inherited workspace fallback, including absent/empty/invalid sources.
Generic default-file fallback is not proof of selected-file absence. Do not
duplicate YAML/policy loading in commands or silently consult an unrelated
default workspace file. Preserve ordinary no-rule behavior, distinct
present/absent/error/unavailable outcomes, independent child roots and fresh
final environment/backend semantics. Name existing public remote acquisition
as effectful, never local/offline/zero-network.
Return a frozen ownership/effect map, exact canonical seams, negative output/
proof limits and NOT_RUN regression plan for custom file selection, deny/errors,
present/absent/empty compatibility, both modes, untouched FD/PATH/no challenge
and distinct local/acquisition-only children. Detached cross-process continuity,
refresh/new routes, all-parent parity and all execution gates remain outside
this design increment. No compiler/tests/formatter/generator/probes/runtime/
container/network/provider/VM/spend/commit/push/release or bootstrap is authorized.

**Accepted workspace-parent DESIGN R2 (historical).**
This packet proposes only a parent acquisition/admission guard for workspace
up/run, not transactional admission of every child. HEAD is
`8926ddb1fc0e4a225059c48375fc6e1185081b1e`. RUBY supplied a read-only inventory;
SCOOBY verified the selected-file, resolver, output and first-effect seams.
All source references below are to this unchanged candidate source.
Historical R1 received qualified VLAD DESIGN GO in
`01a128e1-d626-7a13-8ed0-a25aedb65fc9`, final
`msg_0074fa586a19efbe016acafaaba3fc8191929b191c5d4d7b4f`; MUSE then identified
the inherited absence-marker precedence flaw and held acceptance. MUSE authorized
R2 record correction only: remove the proposed parent bypass and add its source
regression requirement. R1 record hashes were plan
`4929488eb9c50878ae9afd445e2f8d52eedee41b5148aee50d63a991d616cfb2` and
current-state `db918f2a4e02c747504242639233c8621686f6f5d9c981c9236a4dd26cd3f131`.

| Owner / seam | Existing behavior and proposed boundary |
| --- | --- |
| CLI selection | `commands.rs:134463-134488` selects `--file`, then `OTA_FILE`, explicit path, or discovery. Up `41869` and Run `42529` pass that actual file to their parent loaders. Selection is not policy acquisition. |
| Parent contract | Up `145052-145056` / Run `145573-145577` load the selected workspace, compute identity and ordered refs. `workspace.rs:77` owns its external `workspace.policy`; the inline `policies.env.values` copied into `WorkspaceRepoRef.policy_env` is a distinct overlay, not that policy pack. |
| Captured parent effects | Up can emit BLOCKED at `145120`, create a channel at `145138`, emit ACQUIRE at `145145`, and spawn at `145156`. Run equivalents are `145645`, `145663`, ACQUIRE/RUN `145672/145683`, and spawn `145696`. Insert the shared parent guard before either stream branch, channel, progress or worker. |
| Streamed parent effects | Run BLOCKED/ACQUIRE/RUN at `145857/145872/145883`, then child `145892`; Up at `145952/145967/145977`, then child `145986`. Both use the same already-acquired parent outcome, not another mode-specific loader. |
| Child acquisition | Up `146296` / Run `147817` reach `acquire_workspace_repo` before child admission. Existing acquisition eligibility is `137224`; mkdir `137253`, clone `137265`, checkout `137288` are effects, including possible network/credential use. Parent denial/error must precede them. |
| Child ownership | `WorkspaceRepoRef` (`workspace.rs:116-127`) retains each actual contract root. Up loads it at `146402`, Run at `147916`; both pass only the separate inline overlay at `146426/147994/148020`. Parent policy present/absent must never be supplied as a child's own acquired policy. |

**Resolver decision and compatibility.** Add one canonical `policy_pack` entrypoint
for an already-loaded, shape-validated `WorkspaceContract` and the actual selected
workspace file. Share existing acquisition/location/identity helpers with
`load_org_policy_pack_auto_details` (`policy_pack.rs:3134-3181`); do not duplicate
YAML parsing, URL fetching, validation or semantic hashing in commands.
The selected-parent entrypoint has this exact source precedence:

1. `OTA_POLICY`, using its existing path/URL semantics and EnvOverride attribution.
2. Nearest `.ota/org-policy.yaml` found from the selected workspace file's directory
   upward, using existing RepoPolicy attribution and ancestor order.
3. The selected loaded workspace's `workspace.policy`, relative to that exact
   file's directory for local paths, unchanged for remote sources, with
   WorkspacePolicy attribution. Its successful canonical pack identity is retained.
4. If that selected pointer is absent, acquired-absent. There is **no inherited
   default-workspace fallback for this parent entrypoint**: neither an unrelated
   default file in the same directory nor one above it supplies parent policy.

The selected-parent resolver must ignore inherited
`OTA_INTERNAL_POLICY_SNAPSHOT_ABSENT`: proof-child absence is not absence for this
workspace owner and cannot outrank its explicit deny/error source. Do not clear
or change process environment (shared with workers), add a marker, or propagate
parent absence into children. Factor canonical source acquisition below the
existing generic marker check; the parent entrypoint calls that source acquisition
without the proof-child shortcut. Generic repository/child auto-discovery stays
unchanged, including the existing proof-child marker and default-file ancestor
fallback (`policy_pack.rs:3137-3139/3254-3298`). This is an explicit
parent-only ownership rule, not deletion of child inheritance. The alternative
of searching unrelated default files after selected-file absence was rejected:
it would make explicit workspace selection govern a different workspace's policy.
This new parent guard intentionally changes failure precedence: an applicable
parent deny/load error now fails before progress/acquisition, including when all
children are optional. It is not a repo-global or all-child guarantee.
The selected contract's empty pointer is a shape error even with an override
(`workspace.rs:648-656`); absence is not empty. A selected applicable source that
is missing, malformed or semantically invalid fails without lower-priority
fallback. A successfully loaded `policies: {}` pack is acquired-present with a
semantic identity, not absent. A source shadowed by a higher-priority source is
not fetched. Existing remote acquisition is effectful and possibly networked;
this packet does not label policy acquisition local/offline/zero-network.

**Guard, handoff and refusal semantics.** After cheap selected-file/shape/argument
validation and pure selected-graph inspection, but before policy acquisition,
preserve canonical terminal protected-delivery refusal for inspectable local
children. Reuse `secret_delivery_applies_to_selected_subject` / canonical
secret admission and the local carrier qualification seam
(`commands.rs:114308-114333`), not the full grant evaluator. Unsupported/invalid/
missing carriers refuse before policy loading, FD adoption, challenge or any
worker; policy outcome is unavailable, never acquired-absent or Rust denial.
No `--grant` does not itself mean no carrier. A qualifying V4 carrier only passes
this applicability check; it grants no workspace activation/promotion or provider
permission. Do not clone a missing child to discover a protected requirement.
Child execution still performs its canonical checks; this inspection neither
retains all child contracts nor proves they cannot change before execution.

Acquire parent policy once through the selected-parent resolver. A successful
owned `Option<LoadedOrgPolicyPack>` means acquired-present or acquired-absent;
load/validation failure means error, and a pre-acquisition terminal protected
refusal means unavailable. Do not turn error/unavailable into `None` success.
Feed a present pack to existing
`refuse_unsupported_toolchain_fulfillment_policy_route`
(`effect_orchestration.rs:307-328`) with route `workspace.up` or `workspace.run`.
Captured nonempty Rust denial refuses the parent; it is unsupported-route
refusal, not a selected Rust provisioning attempt. Keep the successful parent
owner alive through dispatch; do not reload it in workers or pass it as child
policy/environment. Ordinary no-rule dispatch retains existing ordering/results.
Independent child acquisition and final environment/backend freshness remain
at their established phases; this design does not extend full-pack lifetime.

Extend the private `WorkspaceProblem` (`commands.rs:149988`) with an admission
failure arm rather than mislabeling policy errors as workspace-contract reads.
Render it through `workspace_output.rs` for both public commands. Use the existing
`ToolchainFulfillmentPolicyRouteRefusal` negative carrier, deriving root from the
selected workspace file, retaining policy identity/source/path and exact route;
for Run, selected_task is the workspace verb, not a fictitious child task, and
selected_workflow stays unset. Carry selected file explicitly in the parent
failure envelope. Minimal additive parent failure JSON must distinguish
`policy_acquisition` status (acquired_present/error/unavailable), expose no pack
bytes, and use the existing refusal carrier only for actual Rust route refusal.
Error/unavailable cannot carry a successful policy identity. Acquisition-absent
is a success outcome, not this refusal envelope. Render one final failure and
nonzero exit without repo progress, child reports, receipt persistence, inferred
readiness, hydration observation, archival eligibility or authority evidence.
Existing carrier attempted/execution_started=false describes the refused route,
not a claim that remote policy acquisition made no network request. Preserve
structured negative evidence in JSON as well as honest text/--plain output.

**Uncovered material behavior.** Parent acquisition/guard is proposed contract-owned
behavior, NOT_PROVED until executed. Local child's own policy is independently
owned; a later local-child denial may follow earlier child work in this parent-only
slice. That is the named remaining workspace all-local-child admission gap, not
closed by this guard. Acquisition-only child policy is unavailable until clone;
later refusal does not prove zero clone/network/earlier-child effects. These limits
must be carried in affected public guidance, not disguised as all-parent parity.
Fresh final env/backend failure remains separately bounded; detached continuity,
refresh, new routes, Step 7 custody and provider/installed proof are outside scope.

**NOT_RUN regression/propagation checklist for implementation review.**
- Actual public workspace up/run, both capture/stream, jobs >1 and mixed local/
  acquisition-only children: selected custom file deny/read/parse/validate errors
  yield no progress, worker, mkdir, Git, hooks/tasks, retained-FD mutation, PATH
  activation or challenge. Isolate environment/CWD with existing test guards.
- Parent precedence conflicts: env > ancestor org > selected pointer; relative
  pointer anchored to actual selected file even with a different CWD; unrelated
  default and ancestor workspace files do not fill selected-parent absence.
  Preserve generic child fallback in a separate assertion; optional children do
  not make a parent denial warning-only.
- Inherited internal absence marker plus each applicable env/ancestor/selected
  parent deny or load-error source: parent still refuses before effects, leaves
  the marker untouched, and does not fabricate acquired absence. Parent absence
  requires actual source selection returning none. Preserve existing generic
  proof-child behavior separately; do not mutate global env to switch resolvers.
- Selected missing file/invalid shape/empty pointer, absent pointer, empty valid
  pack, missing/malformed/invalid applicable pack and shadowed pointer; present
  identity/source attribution and unchanged ordinary no-rule results/overlays.
- Canonical pure protected refusal wins over malformed/remote parent policy with
  no remote fetch, capability acquisition, activation or challenge; qualifying V4
  does not imply activation. Uninspectable children stay unavailable, not absent.
- Parent own absence cannot suppress local child's distinct deny/error; acquired
  child's own check remains after acquisition. Preserve final env/backend fresh
  checks and make no zero-earlier-effects assertion for their later failure.
- Parent failure JSON/text serialization and schema/conformance: exact selected
  file/root/route/selection/identity attribution, distinct error/unavailable,
  negative-only carrier, no synthetic receipt or positive execution fields.

Only this plan and current-state change in DESIGN R1. If accepted for source
implementation, the public semantic/output change requires Core command/policy/
workspace and JSON references, workspace up/run failure schemas/conformance tests,
and changelog assessment. Site command cards/sync, policy/workspace guidance,
Learn, FAQ and Glossary must reflect selected-file ownership and refusal limits;
Skills must teach the same boundary. Examples need a custom-file policy example
if their copy-ready shapes otherwise imply default-only selection. These are
OPEN propagation gates, not authorized edits in this design increment. No contract
schema key, flag, Protocol/Launcher pin or authority expansion is proposed.

Historical design freeze source identities (not implementation/runtime acceptance):

| Path | SHA-256 |
| --- | --- |
| `src/cli/commands.rs` | `b4a3a7aa835fb26bf6af0c435c69d96b77996341c399d1c03dd36cf3820bf549` |
| `src/cli/commands/workspace_output.rs` | `15b63db5bdb36c9404d4ce4097867cfbaa0454a915a9628466bf0ad9b8e6bff9` |
| `src/policy_pack.rs` | `91e3b09e60d82cfe67696d984ab20015a950b75b474d85eb5b0442c97a311a67` |
| `src/workspace.rs` | `97921b1c113606c2d65dc7d29b16d16ac461c8bd568c61293a07278fb885955d` |
| `src/effect_orchestration.rs` | `9e980ea21f17d8613c8e37d86da44321e254c93a2591824f48321fb4a79dac02` |
| `src/runner.rs` | `f6d295e2e3316c095727e080b399f3ead4c48c1325f9a94122fea23c64c28bc9` |

Qualified VLAD R2 `01a12912-a75a-7912-839a-75af835a9e00`, final
`msg_0074fa586a19efbe016acb0700182c8191a1969abc34a2bbdc`, closed the marker P2
at DESIGN level. MUSE independently reconciled and accepted the design. Reviewed
pre-acceptance plan hash was
`ac355c6f55eab4b123ed3824ed6d26a632d4e5f5dec497069033e2b26634b3db`; current-state
was `526bd50016bc95128ee6f75f0a741cbcfb003d2e1e76ba5b2a172ae1acba20d9`.

**Workspace-parent SOURCE R1 (historical HOLD) / SOURCE R2 (accepted SOURCE only).**
Qualified VLAD R1 `01a12923-b3aa-7871-8d86-8471a83d6e55`, final
`msg_0074fa586a19efbe016acb0c4815148191877b385ea28f3681`, found four blockers:
P1 portable shape/argument test called a Unix-only fixture; P2 child overlay used
invalid `env` shape; P2 protected Up fixture omitted the workflow recipient;
P2 bare-relative parent refusal carried an empty root. MUSE independently
reconciled the frozen R1 and granted ONE narrow SOURCE R2 repair, not acceptance.
R1 reviewed commands hash was
`156458c555c33b9c4128d29b30f75b3a7757d2e97783075deb6b2713be6f6be6`;
plan `2d2ddb91c949559da9e3e1ab3262cd015267c22eb9d59983aca36a807fa8f0ba`;
current-state `0f64dd31437134dcc50338a98c845a9ca0c433a39f4f1dcde709b5e5bd7010a4`.
R2 makes the filesystem/YAML helper portable, gates shell/FD cases as Unix-only,
corrects `env.vars` with explicit valid-child and policy-failure assertions,
and validates both task/workflow secret applicability before public commands.
The workspace caller alone changes an empty negative root to `.` while retaining
the original envelope path and policy attribution; other roots and the shared
runner are unchanged. This denotes the selected current directory, not absolute
path or custody proof, and adds no lower-source filesystem read after env selection.
A NOT_RUN bare-relative public Up/Run regression covers captured/streamed modes,
CWD locking/restoration, root/path attribution and untouched acquisition/FD/PATH.
Qualified VLAD R2 `01a1292d-93b4-7492-9c7e-5af0823d5888`, final
`msg_0074fa586a19efbe016acb0e5095748191afa215c741413e6f`, closed all four R1
findings by source inspection. MUSE independently reconciled and accepted only
bounded SOURCE R2. Reviewed commands hash was
`f244dd7be15eaebd3a944bf8fbe4642bc6772dd819ce6952f7d84f78ea0a11f6`;
pre-acceptance plan `ff72cab240bae56675b2f2088d47d406a0e7aeeb207164f66003eb62191492ee`;
current-state `ded5056ed9ebc107492afa06ee20ade8a8a8027c192ffef358d4555dc424fdee`.
All fourteen packet/three neighbor hashes and HEAD
`8926ddb1fc0e4a225059c48375fc6e1185081b1e` remained unchanged in review.
Compilation/tests/schema/runtime and whole-slice acceptance remain NOT_RUN/HOLD.

Source now shares canonical pack loading beneath the generic proof-child marker
shortcut. The selected-parent entrypoint ignores that inherited marker and
consumes the actual loaded workspace. MUSE separately cleared normalizing only
the selected DIRECTORY, after shadowing `OTA_POLICY` selection, then joining the
original basename. Bare-relative / `configs/../file` ancestry is canonical;
a file symlink does not relocate its pointer to the target file's directory.
Required directory failure is an attributed error; caller-selected spelling stays
in output. Generic child/proof acquisition and source attribution remain unchanged.
No process environment mutation, new marker, fetcher or custody/ABA claim is added.

Both parent loaders call `admit_workspace_parent` after local shape/refs and before
the mode branch. Existing inspectable local selected secret declarations reuse
canonical pure admission/local carrier qualification before parent acquisition;
invalid or uninspectable children retain their existing per-child report, never
an inferred policy absence. Successful owned parent Option is retained in the
parent scope, not sent to children. Captured Rust denial calls the existing route
refusal; load/identity errors and protected unavailability stay distinct.
`WorkspaceProblem::Admission` adds only shared final failure rendering/exhaustive
matches. JSON distinguishes `parent_admission.policy_acquisition`, reuses the
negative carrier, and forbids roll-up/receipt/positive authority artifacts.
Selected task is the workspace verb; no child workflow is fabricated.
Remote policy acquisition remains an effect before route refusal.

NOT_RUN sources now cover the actual public Up/Run guard with captured JSON jobs=2
and streamed Text jobs=1 (existing JSON-stream/parallel-stream argument restrictions
unchanged), optional local/acquisition-only children, env/org/selected deny/errors
under an inherited marker, original file/source/root selection attribution,
zero progress/worker/acquisition counters, retained socket/FD/PATH/no challenge,
pure protected malformed/remote no-load precedence, actual parent absence versus
distinct child denial/error, and stable inline-overlay success. Canonical resolver
sources cover present/absent/empty/invalid/shadowed/no-fallback, relative ancestry
and Unix file-symlink ownership; schema projection sources reject synthetic receipts,
positive execution, fabricated absence/identity and mismatched status/code.
The schema test isolates the real failure branch from unrelated success-receipt
references; it does not claim whole-document or produced-output conformance proof.
No tests, compiler, syntax/schema executor, formatter or runtime were run.

RUBY's disjoint Core-doc changes were inspected: workspace-reference and org-policy
carry source-stage parent-only ownership/limits. Core command/policy/JSON references,
failure schemas and changelog are staged. Site command cards/sync/reference,
Learn/FAQ/Glossary, Skills and Examples propagation remain OPEN before shipping;
no public term/command semantic change is exempted. No public flag/contract key,
Protocol/Launcher pin or authority activation is added. All-local-child upfront
admission remains a named gap, acquisition-only policy stays unknown before clone,
fresh final env/backend and detached continuity stay separate. Step7 OPEN/R1 BLOCKED,
Step8/V12.2 inactive. Local-child DESIGN review is next; no commit/push or broader gate clearance.

**Workspace local-child DESIGN R1 (historical HOLD) / DESIGN R2 (accepted; SOURCE checkpoint authorized).**
MUSE selected ONE design-only checkpoint after parent SOURCE R2 acceptance.
SCOOBY owns architecture and sole canonical-document writes; production/tests/
schemas remained frozen during DESIGN review. The integrated SOURCE grant below
is now the sole implementation write set. This does not close the whole all-local-child gap. Accepted
Up frontend root/member/canary work is not repeated. MUSE reconciled Up versus
Run restriction semantics and optional-child scope before this design freeze.
Qualified VLAD R1 `01a12937-c4d6-73c2-8742-3626530b92d3`, final
`msg_0074fa586a19efbe016acb10a6794c8191b77643e85bf459eb`, held one P2: child
typed/Up failures cannot inherit the parent-only Rust carrier/code/subject rules.
MUSE independently reconciled and granted ONE docs-only DESIGN R2 repair.
Historical R1 plan hash
`01eae993ad0f6d557443f936a8dd2186e856147abfe878c02a10b4c22bea2e39`;
current-state `89a8d6a586020cf992b28e24c37dad4c43aa305b33b7c730591950c6e8a77973`.
The independent-owner marker bypass is accepted as a bounded design invariant.
Qualified VLAD R2 `01a1293c-ae8f-7593-a4b1-e9533a7e1f08`, final
`msg_0074fa586a19efbe016acb11b8ccdc8191b2be66f84d7ca52b`, closed the R1 P2 at
DESIGN level. MUSE independently reconciled and accepted DESIGN R2, then cleared
ONE integrated SOURCE checkpoint. Historical reviewed plan hash
`dfc3cec2313b9d2f11da43feca33e628ccfc9c464e3f710f08b2dd36656ab7c3`;
current-state `4640ab4822cc748905402ecb4b91a9f4b49f1ed23712f72e4aadb35dee7df9a4`.
Local-child SOURCE R1 is historical HOLD; SOURCE R2 is accepted at source level
at Core `8926ddb1fc0e4a225059c48375fc6e1185081b1e`; validation remains HOLD.
Qualified R1 review `01a12a25-bdb9-7923-88e5-47301d66952c`, final
`msg_0074fa586a19efbe016acb4e24ce5c8191837e2847d98e1fd7`, held one P1: full
instance selectors were passed to literal base-workflow policy/secret consumers.
MUSE independently reconciled and granted only commands/in-file-test/plan/state
SOURCE R2 repair. Historical R1 commands hash
`e6a2c283fed6531575c16266a2af452f52d8e51b5d9d2ae3637c4afde5227506`;
plan `d6181b88bf64b1eb9bc637ae0826307ef1ce9f3671f47058ec3f302871cb2984`;
state `6110eb644843ec981caa3bbf39ff1665fa889670daa72cea71d5f8574aa7e4f8`.
Qualified R2 recheck `01a12a2e-74c9-7252-973b-ceb860170658`, final
`msg_0074fa586a19efbe016acb4fa6456c8191a490d35e0ac6df73`, found no remaining
P1/P2/P3 in the bounded repair. MUSE independently reconciled all 17 unchanged
packet hashes and accepted SOURCE R2, not compiled/runtime behavior. Reviewed
commands hash `0939cb05b4ae3953467c135e2b7a06560d06930afadca2fe2211dab9919b1bd9`;
plan `29bce6a6415ab4843f156754457f0e4a31685430c940f45cbc46df85ca780f90`;
state `286b4c34ae5b547290b786d66f40d5f694b9a4dd1e714afa798c36eed4045233`.

Next SOURCE write set: policy_pack.rs, effect_orchestration.rs, commands.rs,
workspace_output.rs and their in-file tests; connected workspace-up/run schemas,
json_schema_contracts.rs, narrow existing Core command/workspace/policy/JSON docs,
CHANGELOG and canonical plan/current-state. workspace.rs stays READ-ONLY; reuse
existing retained-policy Run APIs and from-admission Up path. MUSE cleared one
narrow runner.rs exception: a pure exhaustive `RunError::variant_name` diagnostic
accessor and focused portable source-only assertions. It returns the actual outer
variant, including wrappers, with no wildcard, message/Debug parsing, new policy
code or runtime/enum/Display change. Prior reviewed runner identity:
`f6d295e2e3316c095727e080b399f3ead4c48c1325f9a94122fea23c64c28bc9`.
All other runner surfaces remain READ-ONLY.
An isolated unused loader is not an accepted checkpoint: acquisition, matching
owned target/admission handoff, both-mode upfront guard, disjoint negative output
and NOT_RUN regression sources must integrate together. After-clone/final-phase/
body failures retain ordinary partial-effect reporting, never the pre-dispatch
no-execution envelope. Freeze SOURCE for qualified VLAD then MUSE before acceptance.
No compiler/test/schema execution/formatter/generator/probe/runtime/network/
provider/container/VM/spend/bootstrap/commit/push/release clearance is granted.

SOURCE R2 accepted seams (validation HOLD): independent-child acquisition shares
the canonical loader and command-admission core, bypassing only an unrelated proof-
child absence marker. Stage-owned acquisition failures distinguish structural,
protected, load, identity and post-acquisition evaluation errors. Present local
targets/admissions are retained with their actual refs before both scheduler paths;
required initial failure returns the disjoint negative child branch and optional
terminal children never dispatch. Workers consume retained target/admission through
existing from-admission Up and retained-context Run APIs; unknown-before-clone
children acquire fresh admission after acquisition and use normal child reports.
The output copies only actual decision/evidence and keeps typed realization,
typed denial and Up unsupported Rust refusal distinct, including selected instance
attribution. Run Rust deny remains conditional; no readiness probe was added.
NOT_RUN sources cover late required failures before progress/workers/acquisition,
marker conflicts, typed causes, mapped task/Up subject/carrier identity, optional
WARN/no worker/body, absence versus invalid selection/contract, independent owned
pack retention, missing-child unknowns, a late optional protected parent refusal,
normal final-policy failure reports and no-Rust Run compatibility. Matching schemas,
schema regression sources and two references were returned by RUBY and integrated.
Schema projections constrain disjoint roots and evidence shapes but explicitly
do not establish relational child/pack/task attribution. Up constructor source
assertions bind actual child root/pack and explicit/default `workflow@instance`;
the full selector reaches roots/profile adjustment and attribution, while the
resolved base workflow identity reaches typed-policy and secret admission.
Explicit/default/omitted selector sources require the actual workflow-scoped deny
rule, matching canonical adjusted decision, and workflow-only secret recipients'
protected refusal before policy loading with unavailable/no invented evidence.
No compilation, tests, schema evaluation or runtime proof exists for this slice.
The integrated SOURCE R2 passed qualified VLAD review and substantive MUSE
acceptance. Scoped static `git diff --check` passes; no execution gate or
propagation completion is inferred. MUSE selected only design preparation of the
combined offline check packet, not another product-source slice.

Checklist: map selected present local children; retain separate valid target/
admission/subject ownership; stop a required initial terminal admission failure
before any scheduling effect; keep optional terminal reports as WARN/no body;
specify attribution, compatibility and NOT_RUN regressions; obtain qualified
VLAD design review and substantive MUSE acceptance before implementation.

| Current seam | Proposed owner/handoff | Boundary |
| --- | --- | --- |
| commands.rs:145525 / :146050, parent Up/Run loaders | After existing parent admission, prepare every ordered present local child before either mode branch | Before progress/channels/workers/acquisition mkdir/clone/body; parent pack remains separate |
| workspace.rs:116 / :131, `WorkspaceRepoRef` / `resolve_workspace_task` | Bind each owned result to actual name/path/contract_path and mapped Run task or selected Up workflow | Preserve required/optional, dependency order, task args and `repo.policy_env`; no unrelated member expansion |
| commands.rs:146898 / :148412, existing child loading | Retain the valid loaded target and selected subject instead of reparsing it in the worker | Invalid/missing contract is child-owned error/uninspectable, never acquired policy absence; no disk immutability assertion |
| commands.rs:142400 / :142370, `prepare_up_command_admission` / `acquire_up_command_admission` | Reuse canonical workflow/profile/phase roots, owned policy and terminal typed/protected result; successful admission enters `execute_repo_up_with_admission_and_grant` (:142181) | Preserve actual false-agent/no-grant/default child invocation, accepted same-context prerequisites and final fresh phases |
| effect_orchestration.rs:293 / :438, `admit_command_execution_closure` | Run uses actual mapped roots, owned child pack and canonical typed decision; terminal typed denial/load error stops upfront | Do not turn every child `rust: deny` pack into an unconditional Run denial |
| runner.rs:19294 / :19315 / :7842 / :8023, retained policy and runner entrypoints | Matching child acquisition projects through `RetainedRunPolicyContext::from_acquired`; both existing `*_with_policy_context` runners receive it alongside separate inline overlay | Final env/backend remain fresh; conditional native fulfillment remains the existing runner's responsibility |
| commands.rs:145637 / :146183, captured spawn; :146323 / :146422, streamed loops | Move each per-ref prepared target/admission once with that ref; retain optional terminal report without dispatch | No worker/channel/progress before required upfront failure; no name-only policy substitution |
| commands.rs:137598, `acquire_workspace_repo` | Missing/acquisition-only child remains explicitly unprepared; after existing acquisition it needs fresh canonical child admission before body | Policy unknown before clone; cannot promise every later acquired child admits before earlier effects |

The smallest proposed state is a private owned prepared target/admission or
terminal report coupled to each validated ref. It is not a persisted snapshot,
global env mutation, new marker/parser/fetcher, authority IPC or receipt. Initial
required load error, explicit typed denial or unsupported-Up Rust denial aborts
the workspace before any worker/acquisition, even when a preceding child is
healthy. Optional child-owned terminal failure retains warning and real failure
attribution, excludes that child from dispatch and preserves existing optional
outcomes; it is not allowed, absent or executed success. Existing PARENT-level
protected refusal still inspects optional children and remains terminal before
policy loading/FD challenge; it cannot be downgraded into optional WARN.

**Restriction scope.** Up already refuses captured Rust deny via its unsupported
route helper. Run Rust denial is conditional on actual selected fulfillment/
readiness (runner.rs:19854-19906); a non-Rust or eligible satisfied-readiness task
must not become a blanket refusal. This design catches initially observable
canonical command-admission failures, retains successful Run restriction context,
and leaves conditional readiness-denial-before-ALL-workers NOT_PROVED/OPEN with
its separately reviewed mechanism. No probes, fulfillment, PATH mutation, task-
name heuristic or root-only closure approximation are hidden in preflight.

**Marker ownership prerequisite.** The generic acquisition used at
effect_orchestration.rs:438 still honors an inherited proof-child absence marker;
it cannot establish a different workspace child's policy absence. Before child
implementation, review one explicit independent-owner acquisition seam sharing
`policy_pack.rs:3152`'s canonical loader with no selected-workspace fallback input.
It must ignore only the unrelated inherited marker, preserve env/org/default
workspace discovery for that actual child, and retain the acquired result into
canonical command admission rather than fetch again. Ordinary proof-child callers
keep existing marker semantics. Sharing the existing command-admission core is
preferred to duplicating typed/secret closure logic. This connected loader/admission
entrypoint amendment is a MUSE-accepted bounded design invariant, not a separate
standalone implementation grant; it belongs only to the integrated SOURCE slice.
Pure protected applicability/carrier refusal
still precedes every policy acquisition, including a later protected child.

**Child failure model (DESIGN R2).** Keep the existing success and parent-only
failure branches unchanged. Add a disjoint negative `child_admission` branch to
the same outer workspace envelope: `ok: false`, original workspace `path`, workspace
`task` for Run only, `code: workspace_child_admission_refused`, canonical error
message in `error`, and `child_admission`. Forbid simultaneous `parent_admission`,
`receipt`, `repos`, `summary`, `artifact_routing` or success/attempt/archive fields.
`child_admission` requires outer route (`workspace.up` or `workspace.run`),
`execution_started: false`, `attempted: false`, `selected_child`, acquisition state
and discriminated `failure`. Those booleans describe refused dispatch, NOT zero
policy-network or proof that no earlier policy acquisition occurred.

`selected_child` requires actual validated ref name/path/contract_path plus actual
selected task and workflow as nullable fields: Run uses the mapped repo task,
workflow null; Up uses selected workflow/instance when present, task null. Invalid
selection retains the requested selector explicitly, not a claimed resolved one.
Acquisition state is independently `acquired_present`, `acquired_absent`, `error`
or `unavailable`. Present requires copied canonical policy identity/source/path;
all other states forbid successful-policy identity metadata. Absence requires a
successful independent-child acquisition returning None, never a marker shortcut.
`failure` requires kind and actual canonical error variant/message; preserve its
canonical code when one exists, do not invent a code for `FileActionFailed`.
Stage/kind derives from the actual validation/acquisition/admission outcome, never
message-text guesses. A failed identity derivation is acquisition error, not
fabricated acquired-present success.

| Failure kind | Allowed acquisition state | Required actual evidence / forbidden substitution |
| --- | --- | --- |
| `contract_invalid` / `selection_invalid` | unavailable | Original load/validation/selection error(s), requested selector if unresolved; no policy identity, decision or Rust/typed carrier |
| `policy_load_failed` / `policy_identity_failed` | error | Actual canonical acquisition/identity error and available source path; no successful identity or denial carrier |
| `protected_unavailable` | unavailable | Canonical pure protected reason/error; no pack or typed/Rust carrier; parent protected precedence remains unchanged |
| `typed_policy_denied` | acquired_present or genuinely observed acquired_absent | Actual canonical `EffectPolicyDenied`, actual `policy_decision`; existing typed-refusal evidence ONLY when `typed_effect_policy_refusal_evidence` returns Some; never Rust carrier |
| `typed_realization_unavailable` | acquired_present or genuinely observed acquired_absent | Actual canonical `FileActionFailed` from `typed_effect_admission_refusal`, e.g. declared-only/disabled-provider realization; actual decision if present, never manufacture a denial decision or carrier |
| `unsupported_route` | acquired_present | Existing `ToolchainFulfillmentPolicyRouteUnsupported` and exact CHILD Rust-route carrier (Up route `up`, actual child root/workflow, task null), identity/source/path matching this child's owned pack; never route `workspace.up` or borrowed parent evidence |
| `admission_error` | unavailable before acquisition; acquired_present/acquired_absent only after successful independent acquisition | Exact other canonical structural/admission error and observed stage; decision only if actually produced, no invented typed/Rust carrier or absent result |

The evidence helper at effect_orchestration.rs:838-851 emits typed-refusal evidence
only for explicit typed aggregate denial. Coarse policy denial does not have that
carrier; preserve the existing canonical decision/error without pretending one
exists. An absent-policy typed denial is allowed only if canonical evaluation
actually supplies it after successful observed absence; this design does not
create that outcome. `typed_realization_unavailable` is not a policy denial.
Preserve `typed_effect_admission_refusal` ordering: declared-only/ineligible
realization checks precede aggregate policy denial. Classification follows the
actual returned cause at that stage, not aggregate_decision alone; an unrelated
`FileActionFailed` never becomes typed realization unavailability.
No policy snapshot/refusal archive path is added by this preflight.

Future schema changes must add one child-only root union branch, corresponding
kind/state/required/forbidden rules and actual child selector; they must NOT relax
the existing parent `acquired_present` Rust-carrier rule, three codes or outer-route/
null-subject constraints (workspace-up.json:31 / workspace-run.json:37). Child Up
carrier route is `up`, not the outer `workspace.up`; Run typed evidence has no Rust
route carrier. Both schemas must reject mixed kinds/carriers and fabricated
policy identity. Cross-field child-root/subject/identity equality is the Rust
constructor's responsibility with explicit NOT_RUN regression assertions; standard
JSON Schema shape checking alone is not relational ownership proof. Streamed text
must display workspace and actual failing-child selectors, kind, acquisition state,
canonical reason and only the real evidence, with the same negative semantics.
Optional precomputed terminal WARN joins ordinary final child reporting, preserving
real failure and exclusion; it does not enter the required negative root branch.
No output/schema source change, new CLI flag or contract key is selected at DESIGN.
Existing captured/streamed jobs/format restrictions remain.

Acceptance/rejection examples: accept required child Up unsupported route with
present CHILD pack and `up`/selected workflow carrier; accept child Run typed denial
with actual present or observed-absent decision and conditional existing typed
evidence; accept malformed child pack as error with no identity; accept unavailable
invalid selection with original requested name. Reject typed denial wearing a
Rust carrier, absent/error/unavailable with borrowed identity, Up carrier naming
the workspace root/verb or nulling a real selected workflow, mixed parent/child
branches, mismatched mapped task/owner/source and every synthetic receipt/success/
attempt/archive. Required canonical evidence unavailable in source must remain
unavailable or the narrower non-denial variant, never be fabricated to fit schema.

Compatibility and risks: required invalid/missing subject or policy failure can
move earlier intentionally; optional warnings preserve existing required-versus-
optional status. Invalid/uninspectable children retain truthful error, not fabricated
policy status. Remote policy acquisition is effectful and can precede another
child's failure: attempted=false never proves zero policy-network. Successful
initial absence/present pack is owned only for its matching admission/readiness
projection; later final env/backend checks remain fresh, and source/contract drift
can fail after earlier effects. No ABA/custody/global full-pack no-reload claim.
Workers do not select unrelated frontend monorepo members or canary commands;
dependencies/hooks/workflow instances use canonical selected closure/Up ownership.

NOT_RUN regression plan: captured jobs>1 and streamed jobs=1; early valid child
then later required Up denial/Run typed denial/load error, zero prior progress/
workers/acquisition/body/challenge; mapped task/workflow and actual failing-child
identity/source/root attribution; optional terminal exclusion with no-rule children
unchanged; child-only Rust deny with non-Rust and satisfied/unsatisfied readiness
limits; valid fixtures checked before invocation; true absence versus invalid/
uninspectable; inherited marker conflict; retained-source change/removal at initial
seams versus separate fresh final-phase errors; distinct inline overlays and args;
missing clone-only policy explicitly unknown; later protected refusal before any
policy effect; no canary/archive promotion. Produced-output/complete-schema executed
conformance remains a separate gate, not a source projection test result.

Uncovered material behavior: the proposed initial valid present-local admission
scope is contract-owned but unproved; acquisition-only/uninspectable child policy,
post-preflight policy/contract drift and final conditional readiness remain bounded
NOT_PROVED or named ownership gaps. Earlier remote/optional effects must not turn
into all-child governance. Detached continuity, public Site/Learn/FAQ/Glossary/
Skills/Examples, full schema conformance, bootstrap and all execution gates remain
OPEN. Step7 stays OPEN/R1 BLOCKED; Step8/V12.2 inactive.

The concrete
Mise-before-acquisition and replay-before-policy order
was escalated to MUSE before dependent entrypoint edits. Existing
no compiler/test/formatter/generator/probe/runtime/network/provider/VM/spend/
commit/push/release restrictions remain. Bootstrap/offline admission stays HOLD.

**Admission ordering amendment.** MUSE cleared source relocation in
`01a12825-4720-7f93-be5b-50d6d05c85e9`: after cheap argument/path/contract
validation and pure selection, move the existing applicable policy acquisition
ahead of Mise activation, replay observation and effectful admission. Downstream
consumers share that owned result through from-loaded APIs. Successful no-rule
selection, identities, fulfillment and terminal outcomes stay compatible;
earlier policy acquisition/load-error precedence is an intentional change.
Workspace parent admission may use the existing canonical resolver before
workers/acquisition; it does not replace differently scoped child policy.
Canonical remote acquisition remains effectful and outside the independently
admitted LOCAL-only offline route; no new fetcher or capability is authorized.
MUSE corrected the ordering to preserve the existing protected Step 6 pure
applicability/refusal gate BEFORE policy acquisition/classification, including
malformed or remote-configured policy. This exception applies only to a terminal
pure refusal, never successful secret execution or authority activation. A caller
carrying that refusal must not interpose policy/replay/probe/activation effects;
workspace must not acquire missing children to discover it. Unacquired protected
refusal stays unavailable, never acquired-absent or a Rust policy denial.
MUSE subsequently corrected the carrier interpretation in
`01a1282e-8cfc-74a1-9871-1c29ce5c22d7`: the public protected refusal projection
can also be carried by the existing V4 bridge and is not itself terminal proof.
Missing/invalid/unsupported carriers must retain canonical refusal before
policy loading; absence of `--grant` does not establish carrier absence because
the protected broker may be selected automatically. After the existing
non-activating carrier validation, acquire applicable invocation policy once,
before authority activation. Captured Rust deny on that carrier-backed route
is an unsupported-restricted-route refusal, without readiness probes. No-rule
continues the established bridge; load failure is not absence. Keep this
invocation restriction separate from the protected admission's unacquired
effect-policy provenance; it supplies no provider truth or authority approval.
Early local custody reads, pure scope selection/hashing and carrier validation
must be mapped explicitly; broker prepare/consume and transaction creation
remain after the restriction gate, with existing freshness revalidation intact.
Qualified VLAD design review `01a12884-f776-70d0-81f5-9f4d85da3be8`, final
`msg_0074fa586a19efbe016acae2cae8bc8191954c281616aad26a`, found no P1/P2/P3;
MUSE accepted the bounded SOURCE ordering implementation. This is not concrete
implementation acceptance. The canonical local carrier selector
`select_crossing_authority_binding` reads/validates the protected prebound and
broker stores and refuses ambiguous bindings; it performs no handshake or
transaction. Its selected V4 binding can qualify the early carrier gate, not
the later authority gate. The full `evaluate_task_crossing_grant` must remain
later: it calls `proof_parent_authorizes_invocation`, whose capability loader
can remove the descriptor environment variable, adopt/change an FD, generate
and write a challenge, and read a capability response. None of that is pure
carrier validation. Do not move full scope/grant/capability evaluation before
the acquired restriction gate. Existing final validation remains authoritative.
Freeze the ownership/order map with the integrated source for VLAD and MUSE
review.

**Typed surface and scope.** Add a default-empty map to `PolicyRules`, with
typed selectors/values serialized exactly as `rust` and `deny`; reject other
keys, case variants, aliases, nulls, scalar/list shapes and values (including
`allow`/`warn`) through canonical pack parsing/validation. Do not encode the rule
in provider `source_config`, a repo fulfillment mode, or a new CLI flag. The
selector denotes the canonical contract toolchain `toolchains.rust`, not one
provider name: implicit/default, explicit and legacy Rustup plus accepted Mise
fulfillment sources are all covered, across selected OS/backend requirements.
Use canonical toolchain ownership, not executable labels, to match dispatch.
Unknown contract toolchains/provider combinations retain canonical validation
failure; no alias may create an unguarded Rust fulfillment lane. Standalone
tool entries, shell-installed Rust, other toolchains, backend bootstrap,
Corepack, repository acquisition and Cargo hydration are NOT covered by this
Rust fulfillment restriction and require separate packet bounds. It is not
global no-install or permission to execute arbitrary installed code.

**Compatibility and identity.** Absent and explicit empty maps are unrestricted
ordinary behavior, not a restricted-validation policy. Omit the empty field from
serialization (`skip_serializing_if`), and prove unchanged old-pack normalized
snapshots/semantic identities and absent-versus-empty equivalence. A nonempty
deny map changes the canonical policy identity. Existing strict serde parsing
alone does not qualify an older driver: workspace acquisition/member execution
may precede the eventual unknown-field error. The restricted packet MUST admit
the exact reviewed driver digest/source/build provenance and supported rule and
entry-point behavior externally BEFORE launch; a version string is insufficient.
Assign a minimum supported release only once shipped and behavior-qualified;
no current/released driver is inferred capable. Old drivers may reject even an
explicit empty new field; omit it for old-driver compatibility. No bootstrap,
installer or old-binary pre-effect guarantee is inferred from pack parsing.

**Policy producer and lifetime.** Carry the normal preflight's retained policy
load result into private invocation-owned runner state: acquired present with
canonical identity/source, acquired absent, load error, and unavailable legacy
caller are distinct. Use the existing acquisition, not Doctor, a fabricated
absence marker or an added early load. A captured deny remains narrowing for the
entire invocation and prerequisites; later policy/env resolution may add
restrictions but cannot remove it. For the restricted offline route require an
exact approved operator-owned LOCAL pack with this deny, its identity and
effective launch environment independently admitted before invocation. Missing,
empty, different, disabled, load-error or unavailable policy never qualifies
that route. Ordinary no-rule compatibility does not imply restricted admission.
Public remote-policy loading remains outside this offline packet. Without
protected external pack/environment custody this is process-local governance,
not tamperproof control of arbitrary agent activity.

**Dispatch decision.** In the shared selected-toolchain preparation path,
preserve terminal interruption/probe cleanup errors. An eligible invocation-local
positive satisfaction may skip fulfillment without populating cache/actions.
Otherwise evaluate captured deny BEFORE fulfillment dedup/cache insertion,
provider command construction/dispatch and each applicable provider call. Deny
means required fulfillment intent is refused even if a prior cached install
would suppress another command: Unknown/unsatisfied/ineligible cannot borrow
cache as positive readiness. `mode: none` has no fulfillment intent and retains
its readiness semantics; default no-rule Run retains ordinary fallback. Failed
or invalidated skipped witnesses at final selection remain terminal before task
body, with original attribution/partial evidence: no late install, retry or
failure hook. This restriction does not enlarge readiness eligibility or convert
a probe failure into a false fulfillment result.

**Entry-point contract, first increment.** Qualify canonical SINGLE-repository
`run` and its prerequisite closure, agent and non-agent alike. Explicitly refuse
captured nonempty restrictions on unsupported `up`, workspace up/run,
multi-member, `proof lifecycle` and `proof runtime` routes at parent admission
BEFORE readiness probes, activation/authority work, streaming workers,
artifact creation, acquisition or backend/provider/hook/task dispatch. Reuse canonical
retained-policy admission, not a duplicate parser. `run_contract_targets`
currently handles members sequentially (`commands.rs:113056-113082`); up executes
root before members (`35340-35365`). Workspace acquisition precedes child policy
(`145309-145431`, `146828-147021`), and can mkdir/clone (`136442-136454`). A
child-only guard or later-member parse failure is therefore insufficient. The
restricted validation launch admits only the supported single-repository route
and explicit local parent policy; policy discoverable only after acquisition
cannot qualify upfront refusal. Do not advertise parent acquisition protection
from a child-only pack. Lifecycle proof directly invokes prerequisite runners
(`commands.rs:3721`); runtime proof creates artifacts before its child workflow
(`4449`). Refuse both at parent admission, including lifecycle assertions/
prerequisites and runtime observers/negative controls; child-only denial is
insufficient. Proof support is a separate widening, not implied by single-repo
Run support. Any broader shared parity requires a separately reviewed
all-target preflight; unsupported routes must not silently ignore the new rule.

Allowed unsupported-route pre-refusal work is argument parsing, approved local
contract/policy reads and validation, pure selected-graph construction, bounded
identity hashing and result/stdio construction. No readiness child probe, repo
acquisition, fulfillment/cache mutation, secret/authority activation, provisioning
or task/hook command is permitted. Existing process-local Mise PATH activation
(`commands.rs:21369`, `34995`) and replay observations before policy load
(`22643-22717`) must be moved behind this guard or explicitly disqualify that
entry point; do not call them effect-free. Existing authorized refusal-receipt
archiving may persist only through its canonical path, not a new early writer.
For supported run, bounded readiness probes are allowed by its separate packet;
Rust deny does not grant other preparation effects or external access.

**Refusal and public output.** Add typed denial, proposed stable reason
`toolchain_fulfillment_policy_denied`; distinguish unsupported-route refusal from
actual denied intent. Reuse existing structured run failure exit **1**
(`commands.rs:115601-115701`), not a new code. Retain root task, refused dependency/
invocation path, canonical toolchain, backend/OS/provider, requested selection,
captured policy identity/source and observed readiness basis; attempted=false,
fulfillment NOT_RUN, no completed cache/action. Preserve already completed
prerequisite evidence, original cancellation/cleanup error and attribution.
Human/JSON/plain receipts must not say 'provisioning attempt failed' for a denial
(`119543-119570` is currently attempt-specific). Preview/Doctor may disclose the
restriction/unsupported route but cannot assert runtime satisfaction or readiness
from the policy alone; positive readiness and permission remain separate.

**Source regression design and propagation.** Cover implicit/explicit/legacy
Rustup and Mise; provider/OS/backend selection and conservative ineligibility;
prerequisite and same-state cache followed by missing/changed selection; final
witness invalidation; present/absent/empty/error/unavailable/different/disabled
policy, narrowing under later resolution and acquisition count/order; unknown
key/value/shape refusal; unchanged old-pack identities; agent/non-agent parity;
all unsupported parents/members, lifecycle prerequisites/assertions and runtime
observers/negative controls refuse before their first observable effects;
denial receipts/exit/partial evidence and no false action. These remain acceptance
requirements; authored tests include the parsing/identity/projection cases,
before-cache source cases, protected/proof-parent ordering and policy reuse,
static diagnostics/output and schema cases described above. Coverage is still
incomplete and none executed. Map the typed pack schema,
validation/policy review, CLI admission, Doctor/preview, receipt JSON, Core
contract/command/JSON references and Unreleased. Shipping requires affected Site
policy/reference and command cards, Learn, FAQ, Glossary, Skills and Examples,
including a copy-ready deny pack and supported-route/old-driver boundaries.
Protocol/Launcher/pins are unaffected by this process-local rule. This planning
amendment changes only the canonical plan/handoff; no public updates are required
yet, but propagation is mandatory before claiming the feature shipped.

**Bootstrap / exact driver.** Repairing candidate source does not change an older
Ota executable used to build it. No compliant bootstrap is currently qualified.
Preferred later route: obtain the repaired Linux driver through an existing
first-party build path under separately reviewed publication/build authority,
retaining the exact source packet/commit, toolchain and artifact digest, then
admit that artifact independently for the canonical check. This is a proposal,
not authorization for commit/push, workflow dispatch, downloads or installation.
Do not build directly with Cargo outside agent admission or substitute a
hand-edited executable. An already retained repaired binary would be an
alternative only after exact provenance and behavior qualification; none has
been identified. Until bootstrap and no-install admission are qualified, the
offline check control-source route remains HOLD, not a compile-ready driver.

**Acceptance regressions, authored but NOT_RUN.** Source cases cover exact
installed selection with all components/targets causes zero provisioning calls;
installed wrong version, missing component/target and unknown/failed/malformed
probe never acquit readiness; a satisfied default or unselected mode does not
acquit the selected mode; PATH/source-managed precedence and a changed binding
cannot borrow prior success; `none` and ordinary unsatisfied `run` preserve their
declared behavior; failed probes/provisioning retain attribution without false
success; repeated invocation/backend changes cannot reuse stale satisfaction.
Include ready-first then missing/changed binding in the same TaskRunState;
timeout, excessive stdout/stderr, plausible version on nonzero exit, malformed
output, spawn failure, descendant-retained output, interruption and cleanup
failure. Unknown/ineligible ordinary fallback must remain explicit.
Additional source cases cover the shared output/deadline budget, source-managed
precedence, compiler overrides and non-native/wrong-OS ineligibility. The lost-child
ownership fixture runs in an isolated test subprocess when separately authorized;
no fixture has executed under this source-only gate.
Separate default/explicit Rustup positives, alternative-source negative, nested
toolchain selection and late environment invalidation cases are authored, NOT_RUN.
The proposed no-install route must independently assert refusal before provider
dispatch for missing/unknown selection, including after preflight, and exercise
the repaired driver rather than merely compiling its source. No source fixture
or green compilation alone proves these properties.

**Containment proposal, still NOT_VERIFIED.** Use a sealed read-only full source
snapshot, not the live checkout. Predeclare an empty, non-symlink `.ota/state`
mountpoint for 64 MiB bounded writable state; keep all remaining `.ota` policy
immutable. Other scratch gets 8 GiB minus 64 MiB, including HOME/cache/target/temp.
Account for source-managed/bin precedence inside state, contract PATH prepends,
and image-provided writable mounts; PATH text alone does not pin execution.
Tmpfs and compiler allocations share the 4 GiB memory ceiling. Require actual
no-swap settings, 2 CPUs, 256 PIDs and 1024 FDs per process, not aggregate FD
custody. Preflight/check/cleanup share one 20-minute parent deadline with reserved
cleanup time and one 1 MiB output budget. Exact-owned ambiguous-creation recovery
and independent absence checks remain required. No-Cargo preflight cannot prove
dependency-cache sufficiency; canonical setup/check must establish that later.
No daemon query, container startup, toolchain probe or cache qualification ran.

**Propagation.** Focused runner regressions, Unreleased and the Core toolchain/run
documentation are source-staged. No schema or public JSON shape changes: existing
receipt fulfillment actions remain limited to commands actually dispatched;
Doctor/preview do not acquire execution readiness through this source helper.
Rendered Site command reference, Learn, FAQ/Glossary and Skills require the
installed-versus-provisioned boundary assessment before public completion;
propagation remains OPEN/deferred until qualification, not silently waived.
Examples need no new contract shape; existing exact-version Rustup examples remain
valid, with runtime pressure still required. No new public flag is proposed.
Protocol/Launcher/pins remain unaffected. Cargo
native/container parity, immutable pressure, Step 7 and public propagation stay
open; this prerequisite does not complete Section A or change custody guarantees.

Frozen prospective real pressure target: Ota Core source revision
`8926ddb1fc0e4a225059c48375fc6e1185081b1e`, using isolated copies and a separate
typed-hydration contract rather than changing protected Core ota.yaml. Later
native and container lanes must run the same locked source against this exact
revision with full command/status/lock evidence and negative controls. Availability
of locked dependency caches and a digest-pinned container remains NOT_VERIFIED;
no partner contact, target change or pressure execution is authorized now.
Section A cannot close without the container increment and immutable pressure.

Propagation owner SCOOBY: Core schema/validator/shared plan/runner/CLI summaries,
hydration provenance/evaluated inputs/replay tests, generated schemas, spec/JSON
reference and Unreleased. Connected Examples, Skills and Site reference/Learn/
FAQ/Glossary must explain default-unlocked compatibility, workspace-root
selection, unsupported backends and acquisition limits before public completion.
Command reference must assess the changed run/preview/receipt semantics even
without a new CLI flag. Protocol/Launcher and their pins remain unchanged.
Source repairs bind selected-mode and nested-source admission, keep typed failures
and retained samples through run/up, and forward the canonical child repo receipt
in workspace output. Cargo samples remain narrowing replay evidence, including
mixed-input uncertainty. Historical workspace child execution archival is
explicitly bounded `not_proved` via an informational finding: current up output
retains inline children, but later workspace receipt archives only current status.
Future durable child retention belongs to workspace/receipt ownership; no new
retention flow is activated by this source increment.

Proposed verification uses deterministic isolated Cargo fixtures for command,
path, identity and drift controls, followed by one Rust repository's exact-head
native/container pressure. No VM is required. Test execution, container use,
dependency network access and partner changes are not authorized by this
source-authoring exception. Runtime source and manual regressions are staged,
but none has been compiled or executed. Source acceptance and validation-route
admission remain separate gates.

Connected-surface assessment: eventual product work affects Core schema,
validator/planning, execution, Doctor, summaries/receipts/replay, generated JSON
schema/reference, tests, contract reference and Unreleased changelog. Examples,
Skills, Site reference, Learn, FAQ and Glossary must carry the compatible posture
and proof limits where relevant. No public CLI addition is proposed; command
reference semantics must still be assessed. Protocol/Launcher and authority
pins are outside this slice. These planning edits do not propagate unshipped
behavior to public surfaces.

### Problem

The typed Cargo dependency-hydration source currently executes `cargo fetch`. A repository that
requires lock-enforced resolution must fall back to a structured command such as
`cargo fetch --locked`. That preserves correctness, but it prevents the typed hydration source
from owning the lock policy it is meant to describe.

### Canonical owner

The existing `prepare.kind: dependency_hydration` / `prepare.source.kind: cargo` source owns this
truth. Do not add a task-global lock flag or infer strictness from the presence of `Cargo.lock`.

The first compatible extension should be an optional Cargo-owned `locked: true` posture. Omission
retains current `cargo fetch` behavior. When true, Core must:

- require a canonical repository-relative `Cargo.lock` at the selected Cargo source root;
- bind the authored lock posture and observed lockfile identity into hydration planning and
  evidence;
- execute the semantic equivalent of `cargo fetch --locked` through the existing typed source;
- refuse missing, aliased, escaping and observably replaced lockfiles; the
  proposed sampling-based acceptance amendment above requires explicit
  acquisition/concurrent-mutation not_proved evidence even when samples match;
- verify the retained lockfile bytes again before execution and record post-run drift; and
- never relabel registry reachability or downloaded package integrity as hermetic resolution.

### Acceptance bar

- absent, false, and true postures have explicit compatibility tests;
- native and container execution use the same semantic lock policy;
- dry-run, Doctor, execution, receipt, and replay surfaces agree on the selected posture;
- unlocked mutation succeeds only under the explicitly unlocked compatibility branch;
- lockfile substitution and update attempts refuse or produce bounded drift evidence; and
- at least one real Rust repository replaces a command fallback with the typed source and passes
  immutable native and container pressure.

## B. Aggregate Mode Eligibility

Status: completed and independently reviewed. Discord contract pressure proved that the former
all-branch closure could reject a valid container aggregate because a native-only mode branch
declared an unselected dependency. The selected-graph repair and its bounded pressure evidence are
recorded in `docs/ai/current-state.md`; this is not an active future batch.

### Problem

An aggregate has no executable body of its own. Its usable modes depend on every selected member,
dependency, hook, and context in the exact closure. Treating the aggregate as universally portable
or selecting different modes for hidden members would make the invocation identity misleading.

### Canonical owner

The existing execution planner owns aggregate mode eligibility. Core must not add a parallel
closure resolver. It is derived from one canonical backend-selected execution graph, not declared
as a second list on the aggregate and not inferred from unrelated repository contexts.

Keep two concepts separate:

- the selected graph carries the requested root, requested and effective backend, target OS,
  requested lifecycle, host-port and memory overrides, ordered executable edges, possible outcome
  hooks, occurrences, roles, invocation origins, selected contexts, a per-occurrence digest of
  the resolved executable body, requirements, environment, runtime, effects, and task resources,
  plus the selected workflow-required service closure and each service definition identity;
- the inventory closure carries every authored branch and remains available only to structural
  validation, authoring diagnostics, and contract inspection.

Execution-facing discovery, Doctor, dry-run, agent admission, CI projection, effects, replay-input
policy, sandbox policy, proof, execution, receipts, and archive verification must consume or
re-derive the selected graph. Structural validation continues to use `all_depends_on` so invalid
unselected branches do not escape contract validation.

Core should publish, for each requested mode:

- the exact closure members considered;
- each member's supported or unavailable posture;
- the intersection that makes the complete aggregate callable; and
- a stable blocker identifying every member that prevents the requested mode.

One aggregate invocation uses one mode. Existing child inheritance and override rules determine
the selected child backend, but aggregate admission refuses when any concrete selected node
resolves to a different backend. Per-member mixed execution remains unsupported unless a future
workflow model explicitly binds every transition, runtime, artifact handoff, and cleanup
obligation. Ota must not silently split one aggregate across host and container execution.

Ordered dependency and aggregate-member edges are semantic. Reordering them changes graph
identity. Only genuinely unordered sets may normalize without changing identity. Possible hooks
must be admitted before execution because their outcome is not yet known; receipts record only the
hooks that actually execute.

Historical artifacts that do not retain enough selected-graph truth remain structurally
inspectable as `legacy_unverified`. Core must not reinterpret an old all-branch identity as the new
selected graph or silently declare it a match.

### Acceptance bar

- an all-native, all-container, and dual-mode aggregate resolve deterministically;
- one unavailable member blocks the complete aggregate before setup or child execution;
- dependencies, hooks, repeated roles, variants, overlays, and generated instances participate;
- reordering or omitting a member changes or invalidates the execution graph identity;
- changing a selected mode-specific edge changes the graph identity while an unselected branch
  does not contaminate that invocation;
- changing selected executable semantics or a requested lifecycle, host-port, or memory override
  changes the graph identity, while an unselected mode branch remains identity-neutral;
- task, workflow, dry-run, proof, CI projection, and agent-safe listings agree; and
- selected execution and dry-run environment resolution do not read or report optional env sources
  owned only by an unselected mode branch;
- successful receipts and Doctor reports do not reopen unselected env sources;
- per-task replay policy follows selected occurrence edges when another workflow or proof root
  independently selects the same task name;
- workflow summaries include only services required by the selected mode closure;
- prior artifacts without reconstructable selected-graph truth remain `legacy_unverified`; and
- pressure includes the Discord bot fixture proving container CI without `.env.local` or Discord
  credentials, native CI with its selected env preparation, and unchanged contributor workflow
  preparation.

## C. Mixed-Mode Preview Selection

### Problem

A repository may declare both native and container contexts while one selected workflow is
intended to run natively. A preview that inspects an unselected container or silently chooses a
mode based on host availability can report the wrong blocker.

### Canonical owner

CLI selection and execution planning jointly own one versioned `mode_selection` posture. Selection
must be derived before runtime/tool admission and must bind:

- the selected task or workflow invocation;
- the authored default, explicit CLI override, or absence of either;
- the exact mode selected;
- available and unavailable modes for the selected closure only; and
- a stable reason when explicit selection is required.

An explicit `--native` must not require Docker for an unselected container context. An explicit
`--mode container` must not fall back to native. If the selected closure has multiple viable modes
and no canonical default, preview must refuse with a stable `mode_selection_required` posture
rather than choosing from ambient host state.

### Acceptance bar

- mixed-mode contracts behave identically across run, up, dry-run, proof, CI, and agent output;
- missing Docker cannot block an explicitly native selected closure;
- absent native tooling cannot block an explicitly container-selected closure;
- default, explicit, unavailable, and ambiguous mode states are schema-constrained;
- unselected contexts never enter readiness or execution identities; and
- Linux, macOS, and container pressure retain the exact selection source and refusal reason.

## D. Doctor Root-Cause Reconciliation

### Problem

One transitive hydration cause can produce the same advisory for several parent tasks. The warning
is truthful, but repeated prose makes the repository look noisier and can hide the one remediation
that resolves every affected path.

### Canonical owner

Doctor owns diagnosis. Human rendering may group findings, but Core must not deduplicate distinct
semantic paths into one weaker fact.

One domain-separated finding-group identity should bind:

- the exact underlying cause identity and code;
- every affected task, workflow, role, mode, platform, and context path;
- the strongest severity and agent posture;
- shared and path-specific remediation; and
- the original finding identities retained for machine reconciliation.

JSON keeps the complete findings and adds grouping rather than removing evidence. Plain and rich
output may render one cause with an affected-path count and expandable path details.

### Acceptance bar

- genuinely identical causes group deterministically regardless of map iteration order;
- different sources, modes, authorities, severities, or remediation never collapse;
- primary blocker selection remains stable;
- machine consumers can recover every original finding and affected path;
- agent admission consumes the original semantics, not presentation grouping; and
- a large aggregate pressure case demonstrates materially lower noise without information loss.

## E. Opaque Shell Execution Boundaries

### Problem

Eris's 47-batch runner is repository-owned shell orchestration. Ota can select it, bind its script
and invocation truth where declared, retain its output, and observe terminal success. It cannot
infer that every internal command ran correctly merely because the wrapper exited zero.

### Canonical owner

The contract remains authoritative. Detector, Doctor, and evidence consumers must label opaque
internals as opaque. Ota must not parse shell text into trusted substeps or treat output markers as
independent execution authority.

A future structured script-inventory profile may activate only when a repository authors and
pressure-tests it. Such a profile must bind the script bytes, interpreter, working directory,
ordered declared substeps, expected terminal markers, and drift posture. It remains observation of
the declared wrapper unless a registered execution adapter independently launches and witnesses
each substep.

### Acceptance bar

- opaque wrappers remain callable without gaining internal assurance;
- output and receipt schemas distinguish wrapper completion from independently witnessed steps;
- missing, reordered, duplicated, or forged markers cannot produce positive substep assurance;
- script-byte or interpreter drift invalidates the authored inventory;
- no generic shell parser becomes a trust source; and
- activation requires at least two materially different source-owned orchestration scripts.

## F. Identity-Bound Container Cleanup Evidence

### Problem

An ephemeral container log can name the selected image and container identity without proving that
the resource reached a terminal removed state after command completion.

### Canonical owner

The inactive [OSS Adapter and Profile Conformance](../adapter-profile-conformance/plan.md) plan
already owns `OwnedResourceCreationIntent`, `OwnedResourceHandle`, lifecycle observations, and
terminal cleanup evidence. This follow-on does not define another receipt or lifecycle taxonomy.

When a future version activates registered container execution under that conformance plan, the
container adapter must retain the stable handle through cleanup and emit exact terminal evidence:

- `cleaned`, with manager-observed absence for the same handle;
- `retained_by_contract`, with the explicit owner and cleanup obligation;
- `transferred`, with recipient acceptance and successor obligation; or
- `cleanup_uncertain`, which cannot support positive lifecycle assurance.

Process exit, an `ephemeral` label, client-side deletion intent, or absence from a later unrelated
listing is insufficient.

### Acceptance bar

- success, task failure, cancellation, timeout, runner crash, lost acknowledgement, and duplicate
  cleanup requests reconcile to the exact handle;
- no handle can disappear from the terminal inventory;
- protected provider/container locators remain protected evidence;
- public projections disclose only profile-approved fields and bounded verification loss;
- Linux evidence cannot establish macOS or Windows cleanup support; and
- immutable pressure includes leaked-resource negative controls and recovery after uncertainty.

## G. Fail-Closed Ota Bootstrap And Drift Coverage

Status: OPEN, source findings recorded 2026-10-16; G1 R2 scoped SOURCE GO, validation NOT RUN.
R1 remains historical SOURCE HOLD. The remaining
bootstrap trust-chain and drift-coverage slices are NOT ACTIVATED.
Accountable engineering owner: SCOOBY. Delivery owner: RUBY for the bounded installer repair,
working directly with MUSE without interrupting SCOOBY's frozen Step 7 checkpoint. Independent
review remains required; VLAD retains security ownership. MUSE owns sequencing and acceptance.

### Evidence And Current Boundary

[ATR PR #618](https://github.com/Agent-Threat-Rule/agent-threat-rules/pull/618) merged as
`44bfb18a5b3292b6ba27a8d09076f5e6f813255d`, from reviewed head
`01e2725cf64a2d9f68efc2063d8a69c3b3578e6a`. Its
[maintainer hardening](https://github.com/Agent-Threat-Rule/agent-threat-rules/pull/618#issuecomment-6202874145)
uses an immutable v1.6.28 release archive, a committed expected SHA-256, an exact binary-version
check and `ota-run/action` with `install: never`. MUSE and RUBY accept this bounded containment;
do not ask ATR to restore the weaker installation path merely to satisfy current drift detection.
Merge does not establish a fresh upstream manual run, sustained adoption or payment.

MUSE's 2026-10-16 read of `https://dist.ota.run/install.sh` returned 22,152 bytes, SHA-256
`1c034b21f12a83c3896dea4008b59bcdc836c6de78e050204707bb45cf5f7c6d`, identical to
Core `scripts/install.sh` at review. The removed setup pin
[`493cba84bf7f7c11c9e8e996d832d93a89c62184`](https://github.com/ota-run/setup/tree/493cba84bf7f7c11c9e8e996d832d93a89c62184)
and its actual `dist/index.js` were inspected, not executed. Source-confirmed gaps:

- The action SHA does not freeze the mutable installer script it downloads and executes.
- Missing checksum files, missing matching asset entries or missing SHA tools can skip verification.
  Release download/checksum failure can instead execute a Cargo Git-source install when release
  mode is not explicitly forced. Forcing release mode alone does not fix skipped verification.
- That older setup pin reports the resolved version without comparing it with the requested one.
  A guard in newer source does not retroactively protect the older pin or undo earlier build effects.
- Released v1.6.28 Doctor does not recognise direct release-download shell as Ota bootstrap truth;
  an empty bootstrap-drift finding set is not proof of that shell's contract/version conformity.
  ATR's current workflow and contract agree at the reviewed head, with future pairing manual.

These are static control-flow and served-byte observations, not an executed exploit or a new
installation test. The workflow's task evidence and its unsupported bootstrap-drift coverage
remain distinct. The canonical commercial merge record stays in the design-partner pipeline.

### Repair And Acceptance Bar

1. Make declared release installation fail closed before extraction/execution on missing,
   malformed, ambiguous or mismatched integrity evidence, or unavailable/failed verification.
   A release request must not silently become a source build or an ambient mode/ref selection.
   Explicit Git/source installation remains a separate, declared compatibility path.
2. Pin the executable bootstrap path as well as the selected asset, using a trusted expected
   integrity binding. Derive version/source selection from the canonical contract, verify the
   resolved binary against it and avoid permanent per-repository copies of version truth.
   Downloading a checksum beside an artifact is not independent authentication by itself.
3. Repair Core's bootstrap-drift coverage through canonical structured install truth. Recognise
   a supported binding or explicitly bound what was not checked; do not present an unrecognised
   shell install as proven consistent. Do not add a generic trusted shell parser as a shortcut.
4. Add isolated regressions for missing/invalid integrity, mismatch, download failure, absent SHA
   tools, wrong version, forbidden source fallback/ambient overrides and paired contract/workflow
   drift. Assert zero unverified payload execution and zero unintended Cargo invocation. Assess
   Linux/macOS/Windows and both installer families explicitly; one platform does not prove parity.
5. Independently review the exact source and bundled action, then separately validate the
   published installer/release path and a bounded consumer. Source, mock, hosted and publication
   evidence must remain separate. No upgrade, release, dispatch or upstream contact follows here.

Each implementation slice requires an exact workset, compatibility posture, permitted validation
and release/propagation plan. Assess Core specs/JSON/tests, setup/action docs and bundles, installer
publication, Site install/reference/Learn/FAQ/Glossary, Skills and Examples. No published behavior,
completed propagation or closed gap follows from a source repair. Current V12.1/Step 7, its
source-measurement authority and every operative limit remain unchanged.

### G1 - Strict Release Installation Source Repair

On 2026-10-16 Bobai requested that RUBY and MUSE repair the ATR-exposed gap in parallel without
interrupting SCOOBY. MUSE activated this security-correctness slice within the current V12.1 work;
it does not activate a new version or expand Step 7 authority.

RUBY is the sole source writer for exactly `scripts/install.sh`, `scripts/bootstrap.ps1`,
`scripts/install.ps1` and new `tests/installer_bootstrap.rs`. MUSE owns this plan, the current-state
pointer and any eventual changelog entry. Existing frozen artifacts, `.github`, `ota.yaml`,
`Cargo.lock`, `AGENTS.md` and SCOOBY's active work are excluded.

MUSE verified and approved the narrow fourth-file extension on 2026-10-16: the Windows wrapper's
`if`/`elseif` forwarding masks mutually exclusive mode switches from the bootstrap. Its G1 edit
only rejects conflicting `FromSource`/`FromGit`/`FromRelease` switches before bootstrap download,
temporary state or local-bootstrap selection, with regression source. No wrapper trust-chain
pinning or other G2 work is activated by this extension.

G1 removes implicit checkout-based source selection and release-to-Cargo fallback. Separately
explicit source/Git installs remain available. Release installation must refuse missing, unreadable,
ambiguous or malformed integrity evidence, unavailable/failed SHA verification, mismatches and
download failures before extraction, payload execution or installation, including custom-origin
and missing-checksum bypass paths. Both installer families require source coverage. Existing
`latest` semantics must not be silently redefined. Removing implicit source selection and permissive
verification is a compatibility change that requires migration guidance before publication.

Author isolated regression sources with dummy inputs and extraction/payload/Cargo counters, then
freeze the diff for MUSE and independent review. This activation permits source authoring and
ordinary text/hash/diff inspection only: no parsing/running target code, tests, installers, builds,
dependency installation, probes, Git mutation, publication, dispatch or infrastructure. Validation
is a separately bounded next step, not an automatic consequence of authoring.

RUBY froze the four-file R1 candidate on 2026-10-16 against Core
`8a0032c198a007840ecd7c97c4018b7ee93fb1b4` and stopped source writing. MUSE independently matched
the following SHA-256 identities before VLAD's focused independent source review:

- `scripts/install.sh`: 23,224 bytes, `e59425152f4f2c39ced03256173e889a5899a2ab3a9ad6c75146359ba9cebf1d`.
- `scripts/bootstrap.ps1`: 25,766 bytes, `6d3de35f8166721f9513833e9b2266cb0e01dba56b433075aea548d6d606c31d`.
- `scripts/install.ps1`: 3,934 bytes, `2fb0f42b440bc7f20a4990946c3de50456f59ff3b03b5af488ca3f569836f22f`.
- `tests/installer_bootstrap.rs`: 29,566 bytes, `3ddc6d2cfef460805b7b476750c6e5621d9f52805e55c97b4a801cf0477d66e7`.

The 13 ignored fixtures (six POSIX, seven Windows) are authored, NOT_PARSED and NOT_RUN.
POSIX fixtures select a synthetic Linux target; Windows positives deliberately stop at the copy
boundary, not a successful executable/replacement result. No actual OS/architecture matrix,
bootstrap publication or installed behavior is established. Existing installation guidance still
describes automatic Cargo fallback and a custom-mirror checksum bypass; reconcile it and the
connected first-party guidance before publication, not by claiming served installers changed.

VLAD's independent source review (`01a141d5-08d7-7ff0-97c3-57bf596cb8af`, final
`msg_0074fa586a19efbe016ad15df94f348191936d4d277a109a40`) found two status-handling blockers:
POSIX directory/staging/replacement errors can fall through to a successful installation result
because the caller's `if !` suppresses reliance on `set -e`; PowerShell native `tar` failure is
unchecked before binary discovery/copy. MUSE verified both source paths and accepted SOURCE HOLD.
MUSE authorized one correction pass in `scripts/install.sh`, `scripts/bootstrap.ps1` and the existing
test source only; the Windows wrapper retains its R1 hash. RUBY froze R2 and stopped; MUSE matched
its observed SHA-256 identities before VLAD's independent delta review:

- `scripts/install.sh`: 24,636 bytes, `1b200b9d7cb02f795789a67e8dd99d6c73326e4c020e3941a84bd4bfcc0435d2`.
- `scripts/bootstrap.ps1`: 25,959 bytes, `69ae0a95f68acd2b57397a75744146dfb69c95e49e56c747858da1003db69923`.
- `tests/installer_bootstrap.rs`: 40,181 bytes, `3d2068f32e22ed3e876f6f745936ff4c0ce812d668c56057bde95ce43f523d8e`.
- `scripts/install.ps1`: unchanged 3,934 bytes and the R1 hash above; total R2 source 94,710 bytes.

R2 explicitly propagates directory/staging/replacement failures and native TAR status. Its
Windows shell replacement path classifies only the fixed PowerShell helper's sharing/lock result
for pending replacement; simulations do not establish actual Windows exception classification,
availability or detached completion. The 16 ignored tests retain the original 13 and add status,
pending and native-TAR controls. The Unix PowerShell fixture requires a separately approved,
matching absolute `OTA_TEST_PWSH` tool and OS-enforced containment, neither supplied nor verified
by source authoring. No target parsing, tests, builds, installation or wider G2 work is authorized.
R1 remains a historical reviewed candidate, not retroactively cleared by R2.

VLAD's focused independent R2 review (`01a141e3-44c7-76d0-99c9-40a0b597d6f9`, final
`msg_0074fa586a19efbe016ad1615b65c08191a14e5ccebf169e93`) found no corrective P1/P2/P3 in the
scoped delta and closed both R1 status-handling findings by textual inspection. MUSE matched the
stable identities and reviewed the connected failure paths/oracles, accepting scoped SOURCE GO
only. All four files stay frozen; this is not compilation, actual OS/locking/replacement proof,
publication evidence, a complete pre-release audit or completed section G acceptance.

The permitted next step is one read-only, chat-only validation-readiness proposal using an
existing declared Ota path and existing OS-enforced containment. It must separate parsing/build
from selected ignored fixtures, native POSIX/PowerShell/Windows prerequisites, resource and
whole-process cleanup controls. Do not invoke Cargo around agent admission, invent task flags,
change the protected contract or build a new validator as an implicit follow-on. Missing admission
or containment is an explicit blocker; source review does not authorize a run.

RUBY's chat-only readiness check returned HOLD. MUSE verified `ota.yaml:125-139` is plain
`cargo test` with no filter input and a `setup` dependency that fetches dependencies, while
`src/runner.rs:23109-23194` treats trailing arguments as declared task inputs, not Cargo flags.
There is no currently permitted narrow runtime command. Do not drop `--agent`, call Cargo
directly, inject a filter through the environment or silently change the protected contract.

Bobai approved focused task definitions only on 2026-10-16 (question
`call_8f62100f0e444d1c92857d6ca9545d46`, answer "Approve focused task definitions only
(Recommended)"). MUSE relayed the explicit protected-file exception. RUBY is sole writer for
`ota.yaml` and only the corresponding generated task-list portion of `AGENTS.md`, adding
`test:installer:compile`, `test:installer:posix`, `test:installer:pwsh-tar` and
`test:installer:windows` with matching agent admission. Existing four G1 R2 source/test files
remain frozen. No general argument passthrough, setup-fetch dependency, toolchain/fulfillment
change or unrelated contract/guidance edit is permitted. Freeze the two-file definition for
independent source review; do not run a generator or parser to obtain it.

The two-file definition was frozen at SHA-256
`c3c4fc593bfb4936709a55092e7d7dc97b4edd90c1eb8a03bb88a2d24c470726` (`ota.yaml`) and
`d80e19dacc7bc91a7be2a9b3eb1b2ea923dee1383f6498fa78cced722d914e1e` (`AGENTS.md`).
VLAD review `01a141f5-3bc6-7c41-a6d0-d4ad40e53687`, final
`msg_0074fa586a19efbe016ad16740e0f48191b2df4a8bb493cd9f`, found P2: the four tasks
select global Rust fulfillment `run`, permitting provisioning before Cargo's offline arguments.
MUSE verified and accepts SOURCE HOLD for that mismatch; no run occurred. Existing task/context
requirements select toolchain names, not task-local fulfillment overrides. Dropping the canonical
Rust requirement or inventing an alias/override is not a correction.

Under Bobai's standing technical delegation, MUSE cleared one connected three-file source
correction: RUBY removes only those four definitions/admissions/generated list entries from
`ota.yaml` and `AGENTS.md`, and relocates them to new `ota.installer-validation.yaml`, selected
by the existing global `--file` mechanism. This is one standalone finite validation contract,
not an overlay or second owner for the same tasks. Its canonical Rust requirement uses
fulfillment `none`; ordinary tasks and global Rust fulfillment remain unchanged. Preserve fixed
arguments, platform limits, prerequisite/protection metadata and narrow agent authority. The new
file has a 16 KiB source-authoring allowance only; no operative limit changes. Existing four G1
R2 code/test files remain frozen. Freeze the three-file correction for independent VLAD source
review and MUSE disposition; no parsing, generator, execution or automatic successor is granted.
This is internal validation selection, not a new CLI feature or public workflow. Remaining
publication/consumer propagation stays open; no new Site/Examples/Skills/Learn/FAQ/Glossary term
or command semantics is introduced by this source-only correction.

The standalone definition candidate is now frozen for independent source review. MUSE matched
all seven definition/implementation/test hashes, read the complete finite contract and verified
that ordinary `ota.yaml` and `AGENTS.md` have no diff from their pre-definition baseline.
The new file is 173 lines / 5,956 bytes, within the 16 KiB authoring allowance. Exact current
definition identities are:

| File | SHA-256 |
| --- | --- |
| `ota.yaml` | `3fb331bf8f15efc85f0ea7066803523234cc35ccf72efa32c09fbfe9aa11fa6c` |
| `AGENTS.md` | `1c8fa657b2adf70bc276d6b8d6c66e8d6a36f5333a3a5e8cf93a0210ae2fc2cd` |
| `ota.installer-validation.yaml` | `b9f8db144f9fba33b9c2b4f2f24380152b3dbec34b8c719cb1f45c63f345057c` |

The four G1 R2 implementation/test identities above remain unchanged. The standalone contract
admits only the four fixed tasks, permits no agent source writes and protects both contracts,
agent guidance and the frozen implementation/test files. No implicit entrypoint/default/bootstrap
task, new Rust alias, dynamic arguments or setup-fetch dependency was added. This records the
source candidate, not closure of the prior P2 before review or permission to execute it. Tracked
whitespace checks passed; the untracked contract's no-index check returned 1 with empty output
because it is a new-file diff. No parser, generator, compiler or test was run.

VLAD's focused independent correction review `01a14221-ed29-7151-9743-7396dfcf7ee9`, final
`msg_0074fa586a19efbe016ad172835f088191989ba7d9a8d6556f`, has no corrective P1/P2/P3;
the prior provisioning-authority P2 closes by source inspection. MUSE rechecked all seven
hashes unchanged and accepts scoped SOURCE GO for the standalone definition correction.
Explicit `--file` selection, diagnose-only Rust, fixed task/OS/admission boundaries and restored
ordinary contracts are source-confirmed, not parser or runtime results. Agent path declarations
are not OS confinement. Rustup shims, caches, native tools and outer containment remain
unqualified; runtime readiness remains HOLD. No parsing, build, test, installation or automatic
validation successor follows from this disposition.

This grant permits source edits and text/hash/diff inspection only, not parsing, builds, tests,
installation, dependency provisioning, containers, infrastructure, Git or publication. The task
definitions are not execution or provisioning authority. Locked/offline Cargo arguments alone
do not establish a whole-path no-install/no-network guarantee. Rust 1.95.0/cache satisfaction,
native PowerShell/Windows qualification and an existing
OS-enforced network-denied/read-only/resource-bounded whole-descendant containment route remain
unqualified. Tool-path presence is metadata only. No new validator, runtime venue, numeric runtime
allowance or infrastructure is adopted by the readiness proposal.

G1 does not independently authenticate a publisher by fetching an adjacent checksum manifest. The
trusted expected digest, immutable setup/action bootstrap path, PowerShell second-stage chain and
connected self-update hop remain open. Doctor's structured checked/not-checked coverage remains a
separate inactive slice. Do not call section G closed after G1 or infer any change to served bytes.

An independent read-only design challenge recommends pinning the existing published
`ota-checksums.txt` hash per exact release, alongside immutable installer identities, rather than
duplicating per-platform release catalogues in both actions. MUSE verified that the existing
release workflow already generates and publishes this manifest. A follow-on must authenticate
the manifest as data before selecting an asset digest, authenticate both Windows script hops,
and bind managed installation to the installed binary rather than an unrelated PATH candidate.
This is a design candidate, not G2 source authority or reviewed production pin metadata. An exact
default bound to an action SHA would require an explicit compatibility/release decision; do not
silently redefine `latest`. No new publication workflow, signing service or installer engine is
justified merely to introduce another metadata format.

## Implementation Order

This register does not impose one bundled release train. When future demand selects work, prefer:

1. lock-strict Cargo hydration as the smallest contract-truth improvement;
2. mixed-mode preview selection as a separate reviewed execution-selection batch, without
   reopening completed aggregate selected-graph correctness;
3. Doctor root-cause reconciliation as a presentation-plus-machine-grouping batch;
4. opaque shell inventory only after a second materially different pressure case; and
5. cleanup evidence only through an activated registered container adapter under conformance.

Do not start item 2 merely because item 1 completes. Each item needs its own activation record,
tests, propagation assessment, pressure evidence, and closure review.

Section G is a security-relevant correctness repair candidate requiring an explicit sequencing
decision, not an automatic addition behind that feature order or permission to interrupt Step 7.

## Current Product Boundary

Aggregate mode admission now follows the completed selected-graph repair. Until another sub-slice
activates:

- Cargo lock enforcement remains command-authored where required;
- mixed-mode callers should select the intended mode explicitly;
- repeated Doctor advisories remain truthful even when verbose;
- source-owned shell internals remain opaque; and
- an ephemeral container result does not establish terminal cleanup evidence.

These boundaries are acceptable for the bounded Eris draft because its contract and evidence state
them explicitly. They must not be omitted from future pressure summaries or converted into support
claims by the Site, Skills, Examples, Enterprise, or an agent.

## Propagation Posture

The completed aggregate repair has its own reviewed Core, Site, Skills, and Examples propagation
record in `docs/ai/current-state.md`; this register adds no new shipped command, schema,
vocabulary, or operator flow. Each remaining activation must reassess Core, Site, Skills, Examples,
Learn, FAQ, and Glossary rather than using this plan as public product documentation.
