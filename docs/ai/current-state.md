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

   Licensed under the Apache License, Version 2.0 (the "License");
   you may not use this file except in compliance with the License.
   You may obtain a copy of the License at

   http://www.apache.org/licenses/LICENSE-2.0

   Unless required by applicable law or agreed to in writing, software
   distributed under the License is distributed on an "AS IS" BASIS,
   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
   See the License for the specific language governing permissions and
   limitations under the License.
-->

# Current Ota Development State

Update this file at a completed implementation, pressure, release, or handoff boundary. Replace
stale state; do not turn it into an activity log. Durable decisions belong in `docs/adr/` and
durable agent workflow belongs in the canonical Ota skill.

## Active Work

### Lock-Strict Cargo Hydration: Source Checkpoint Accepted / Validation HOLD

Bobai requested work on 2026-10-10 after the independent Cargo follow-on
recommendation. ONE native source-authoring increment is now selected inside
V12.1/1.6.29 after MUSE's explicit sequencing exception; owner
SCOOBY. See `docs/planning/execution-contract-follow-ons/plan.md`, section A,
and the matching V12.1 planning exception. MUSE accepted the planning exception;
qualified VLAD planning review `01a1276a-e166-7b23-81f9-3d0b7602f0aa` found no
P1/P2/P3. Qualified concrete review and MUSE accepted the design/requirements
amendment and source-only activation. MUSE now accepts the repaired native
source checkpoint; runtime validation remains HOLD.
The uncommitted source candidate stages default-false compatibility, bounded
native Cargo workspace-root metadata selection only at execution, shared plan/
admission, pre/post samples and retained command status, mandatory acquisition/
concurrent-mutation not_proved, and explicit locked-backend refusal outside
direct native Linux/macOS (also excluding Compose/orchestrator wrappers).
The unconditional concurrent-substitution bar was explicitly amended by MUSE,
not claimed satisfied. Native/container parity and immutable
pressure remain OPEN; a native increment alone cannot ship or close Section A.
First batch includes identity/evidence semantics, not an arguments-only change.
Prospective pressure target is isolated Core `8926ddb1fc0e4a225059c48375fc6e1185081b1e`;
offline cache/container digest and exact validation route are NOT_VERIFIED.
Qualified VLAD concrete design review `01a1276f-d64e-7fc2-8d94-5bf9eca161a6`
found no P1/P2/P3; MUSE independently reconciled and explicitly activated the
one end-to-end native source increment, not an argv-only/schema-only result.
No current-step pressure readiness is inferred. The offline driver must prove
installed toolchain/cache suffice and enforce network denial; no fictitious
Rustup offline environment setting is accepted. MUSE disclosed two public
documentation reads outside the requested local-only review, not runtime or
validation admission evidence; subsequent review is local-source-qualified.
Pre/post hashes do not prove absence of ABA substitution during Cargo.
Source changes include schema/shared argv, generator source, native observations,
early selected-graph/up refusal, nested preparation evidence, Doctor/preview,
receipts and sampled replay inputs, manual regressions, Core spec/JSON reference/
command reference and Unreleased. Fake-Cargo source cases cover unchanged,
post-drift, nonzero, unavailable-post, missing, malformed, pre-drift and spawn
failure; NONE executed. Actual Cargo root semantics, metadata waitid/cleanup,
toolchain-shim auto-install posture and real immutable pressure remain NOT_RUN.
Qualified first candidate review found a selected-mode admission P1 and partial
receipt/typed-error retention P2s; MUSE independently found incorrect nested
refusal attribution. The source repair now uses the resolved execution body and
one path-aware traversal, retains the exact refused source, and carries original
typed errors plus completed/in-flight samples through cleanup and existing run/up
error routing. One shared attachment binds witnessed records and evaluated inputs;
the narrow private up error wrapper does not create new continuation or authority.
Connected source review also found that matching Cargo samples could incorrectly
acquit acquisition. Sampled Cargo kinds now remain narrowing; dependency-resolution
uncertainty remains explicit even with mixed non-Cargo inputs. Workspace up forwards
the canonical child repo receipt only when Cargo evidence exists, without flattening
repo ownership into its outer aggregate. Manual regressions cover selected/unselected
modes, later nested attribution, partial typed failures/no later prepare or failure
hook, run/up evidence parity, workspace scope and replay limits; ALL NOT_RUN.
Uncovered material behavior: historical workspace child execution archival is
explicitly bounded not_proved by informational machine-readable finding
OTA_WORKSPACE_CHILD_EXECUTION_ARCHIVE_NOT_PROVED. The current up output retains
inline repo receipts; a later workspace receipt --archive is only a current-status
snapshot. Future durable child retention belongs to workspace/receipt ownership;
no new persistence flow is activated. Qualified repaired-source review
`01a127b3-cad8-74c1-8a3b-0da7241d6112` confirmed the original three blockers repaired
at source, then found repeated up invocation identity collisions (P2) and stale
Section A activation wording (P3). MUSE cleared only their source repair. The new
candidate derives hierarchical Cargo invocation paths from selected prerequisite
indices and fixed phases before child receipt construction/archival, preserves
original typed errors, and refuses ambiguous duplicate Cargo replay pairing.
Repeated identical/changed occurrence and partial-failure regressions are authored,
NOT_RUN; the planning introduction now reflects source-only activation. Fresh
qualified exact-source review `01a127bf-1ea4-7413-8440-d1f07f9d4471` closed those
two findings at source-review level, with all 17 hashes stable and regressions
NOT_RUN. MUSE then found a connected P2: skipped Cargo comparison artifacts could
let surviving non-Cargo inputs overstate final replay assurance. The source
repair now checks required pairing in the full replay builder, caps hermeticity
from either side's Cargo observations/inputs and preserves dependency-resolution
uncertainty. Full mixed-evidence verdict regressions are authored, NOT_RUN;
qualified VLAD final recheck `01a127c4-b3c7-7af0-8ef9-fb9eaebfdc7f`, final
`msg_0074fa586a19efbe016acab18315488191bff6e425987ee1a1`, found no remaining
P1/P2/P3 and all 17 frozen hashes stable. MUSE independently reconciled the
exact source and accepted this checkpoint in turn
`01a127c4-b457-7431-a5db-7e1bc9a6b11d`. This closes the source findings only:
all authored regressions, compilation and execution validation remain NOT_RUN.
Reviewed HEAD remains `8926ddb1fc0e4a225059c48375fc6e1185081b1e`; changes
remain uncommitted. The reviewed checkpoint identities below precede this
record-only handoff update; its current-state hash is not the resulting live hash.

| Reviewed File | SHA-256 |
| --- | --- |
| src/schema.rs | 185ec0675e88e281e14c61731d52bfe124c3fa60896f04ce085ebff244d8a11b |
| src/validator.rs | 2f1987b44dee0cfe4a9593813c22caa8e1bd0905d58e00e07f03fc2e0c079b23 |
| src/cli/commands/init_starter.rs | 83160484dccbd2e8ab4ebb75240c64a0029d5e1d4691ba5c4d3df88657d59a90 |
| src/hydration_provenance.rs | 7e289b52447fe3891fa880272a4378b2b55052b1e2cf226bda2e30944a7892a0 |
| src/runner.rs | 61c80830969d7e87d3700f23a72efb1c9d478b07bd86c99ffd05fa21787a9791 |
| src/output.rs | cee35277ef05d8b1ce0c2c15f758861a1431f2b1ed973628ea3a58fe01831261 |
| src/doctor.rs | d7509e856382c5a7af2162975c363a28ab76721f7812e2bb15cda802c344800c |
| src/cli/commands.rs | f6e4c36de11a261b3b070116f4847fd72dc63aae4f0d48eab9138f09f97c3ca8 |
| src/cli/commands/workspace_output.rs | 15b63db5bdb36c9404d4ce4097867cfbaa0454a915a9628466bf0ad9b8e6bff9 |
| src/published_contract_schemas.rs | eeb3ad4425141a47708c7a9f107fabeec0963f4d46ae8036c4de09c44dd52ac3 |
| docs/ai/current-state.md | d875bc65f6fb043b8ff27245e2bd4208e19c416dd7e6a08b5c7173e965388f95 |
| docs/planning/execution-contract-follow-ons/plan.md | d79c6d54360609c48d68f59329ad2471e0be1569a50abd2e1f2608b4250cc53d |
| docs/planning/v12.1/plan.md | ebd1da1c9330289622e57ac52e40c2f27e728a154720e692408033c9b2da7051 |
| docs/spec/command-reference.md | 6a529442acd17c92a3ef4ef69b7137ffbed46f106be124cc882d0f33d3f50285 |
| docs/spec/contract-reference.md | 186f22d40b4af520674a4e7b1adeb6ed05307822653f38d4cb46fb408afa25c9 |
| docs/spec/json-output-reference.md | 960fe913578edef31d9468e80f0abadc1531e5d5c1b686d0c8c9d66b3b7a3c68 |
| CHANGELOG.md | 7b34c86f5a53f0e2bbe51353ec58d1cc6f99e52aff24555d8e153a6113220d25 |

Trusted textual diff inspection only; no tests, compiler, formatter/generator,
target evaluation, container, network, SDK/provider operations, VM, spending,
commit or push. Generated JSON schema files are intentionally not regenerated
under this authoring gate. Public Site/reference/Learn/FAQ/Glossary, Skills and
Examples propagation remains OPEN/deferred until the feature is qualified;
no published claims. Protocol/Launcher/pins are unaffected. Unrelated ROADMAP
and agent-execution-governance plan edits are preserved outside this slice.
The prospective offline check route now has a named prerequisite gap:
satisfied-toolchain provisioning avoidance. Trusted source inspection and VLAD
review `01a127d2-e1cf-70c0-8904-9ac75f75cfe8` confirm that selected Rust
`fulfillment: run` invokes the provider install command without first checking
installed satisfaction. This conflicts with the proposed no-install gate; it
does not demonstrate that Rustup actually reinstalls or reaches the network.
The retained older driver source has the same conflict, so repairing candidate
source alone cannot qualify the executing driver. No new control-source packet
has been created or invoked; the offline validation route remains HOLD.
MUSE cleared ONE native Rust readiness SOURCE implementation increment in
`01a127d7-d01c-70e2-a505-703af25a56e4`, after requiring the bounded-probe and
ordinary-run outcome/cache amendments now incorporated in Section A of
`docs/planning/execution-contract-follow-ons/plan.md`. The staged runner checks
direct native Linux/macOS canonical exact-version Rustup selection, effective
environment/executable bindings, rustc release/native host, Cargo version and
required rustfmt/targets before skipping provisioning. Probe source uses exact
argv, regular bounded sinks, success-before-parse, a 5s per-probe/20s aggregate
budget with cleanup reserve, and terminal cancellation/child-retirement errors.
Only positive readiness skips; unknown/ineligible/mismatched ordinary `run`
preserves authorized provisioning. Skips populate neither provisioning cache
nor fulfillment actions. Profiles/channels/other components/compiler wrappers
remain conservatively ineligible. Eleven regression sources cover readiness,
fallback, same-state invalidation, selection/environment, aggregate budgets,
timeouts/output/spawn/nonzero/malformed, descendants and isolated lost-child
ownership; ALL NOT_RUN. Core toolchain/command docs and Unreleased are staged.
VLAD review `01a127f0-60ce-7b01-8e5d-0a33ffa50d35` found implicit Rustup exclusion
(P1), pre-final launch-context mismatch (P2) and ambient compiler fixture inheritance
(P3). Source repairs use canonical default/explicit provider resolution, isolate
fixture compiler overrides and conservatively gate preparation readiness to
structured Rust commands in the repo directory without late env/source, input/
target or native-activation resolution. Nested/unsupported contexts retain
ordinary provisioning; no early secret/environment activation is introduced.
Core's declared PATH overlay remains ineligible, so this repair does not by itself
qualify its offline check. Qualified VLAD R2 recheck
`01a127f6-bd87-7e30-a7f3-562db8c5be33`, final
`msg_0074fa586a19efbe016acabe5b6f8081919f0fe601d59fcc34`, closed all three
findings with no new P1/P2/P3. MUSE independently reconciled and accepted R2
as a bounded SOURCE checkpoint in `01a127f6-bdf6-7c00-bd6e-86cf4f946324`.
The six reviewed SHA-256 identities precede this record update (not live hashes):
runner `059276d7f3bf4d8d97c4157597ff129576d6cb3e2910d4c889fed86109459a37`;
plan `57dba80a7403cd920073da1d02ffeb69a7dc71cc1625e3b78a851df1489f18de`;
current-state `02dbf4eebfe627f5f128221a4caa4a6935fb1eca7dbc7998dfc0bd61fbdc006f`;
command reference `08820dcd3c76d775f53f6ebdc61d771a2fa81284fff4f5e7356cb084ebf1d4b5`;
contract reference `7788130f8f1958d15cf3b35916c6bebe58fd15d05fd99b23837159bd04b5fb88`;
Unreleased `c4fd520bcfa1ea16ad091d59d585d57a322f7221aa029b944e7614d5473d3dc6`.
File metadata/env sampling is ordinary observation, not acquisition or concurrent
substitution proof; actual Rustup no-auto-install probe behavior is NOT_VERIFIED.
The 17 accepted Cargo checkpoint identities above remain historical, not hashes
of the expanded live candidate. NEXT CLEARED by MUSE in
`01a127fe-0a14-7c83-a1b2-b4af2b2e21ad`: ONE source implementation for Core's
literal PATH composition, retained acquired-policy handoff and actual final
same-selection check. The normal CLI preflight already retains loaded/absent/error
policy; carry that result through private run request/state, with unavailable
legacy callers kept ineligible. No added policy/source acquisition or global
snapshot; final resolver order/freshness remains unchanged. Post-final invalidation
of a provision-skipping witness must fail before task dispatch with original
attribution/partial evidence, never late provisioning/retry/failure hooks.
Ordinary fallback remains at preparation. Independent VLAD review then MUSE
acceptance is required for each new source increment. MUSE accepted the Core PATH
R2 SOURCE checkpoint after qualified VLAD recheck. Streaming and captured CLI paths project their actual
preflight result into invocation-owned readiness state without a policy load.
Literal PATH uses canonical composition/precedence; later normal resolution
remains fresh. VLAD `01a1281a-2a88-7d62-b981-9b4b7dac446a` found P1 optional
unselected PATH composition mismatch; R2 now reuses the pure selected-variable
helper in preparation and final resolution, leaving unselected PATH inherited.
Full positive coverage separates selected/unselected; drift/precedence/shadow
negative fixtures explicitly select PATH. Qualified VLAD recheck
`01a1281c-f107-7f03-8b5e-28ecd8146b9d` found no P1/P2/P3; MUSE independently
accepted R2 in `01a1281d-854c-7a12-8ecf-58b13e64cd67`. Reviewed R2 hashes are
historical in Section A's packet table; all runtime/driver gates remain HOLD.
Skipped witnesses bind cwd/env/launch and installed objects, then
compare actual final selection and recheck active Rustup within the original
probe deadline/output budget. Invalidated selection is terminal before body,
without late provisioning/retry/failure hooks; existing preceding Cargo hydration
context is retained, not a new generic step-evidence claim. Six additional source
regressions and a CLI preflight projection assertion are authored; all seventeen
Rust readiness regressions, compilation and runtime remain NOT_RUN.
MUSE selected a separate deny-only public policy direction in
`01a12806-da1b-7b21-bba5-39393db3bae9`:
`policies.toolchain_fulfillment: {rust: deny}`. ONE source-design refinement is
recorded in Section A's Rust Fulfillment Restriction subsection; its later
source activation is recorded below. Canonical rust ownership covers all accepted fulfillment
sources; absent/empty preserves ordinary behavior/old semantic identities.
Captured invocation policy narrows before cache/provider dispatch, while exact
driver/local-pack/launch qualification remains an external validation gate.
First proposed support is single-repository run plus prerequisites; unsupported
up/workspace/multi-member and proof lifecycle/runtime routes must refuse upfront,
not after acquisition, earlier-member effects, activation/authority work,
artifacts, proof prerequisites/assertions or observers/negative controls.
A child-only pack cannot prove parent acquisition
protection. Refusal/output/exit, old-driver and propagation requirements are
design only. Qualified VLAD R2 review `01a1280f-d34f-7163-b9c6-5fb584b3ba0f`
found no P1/P2/P3 after correcting proof-parent coverage; MUSE independently
accepted this design and sequenced the existing Core PATH implementation first.
Reviewed design hashes remain historical: plan
`eac68d8d510126e2a1001a767b89841c196cf2f626d9c22ff1ba385919149674`, handoff
`57efa63d66eb89d4c291942b1365e35c396ad243cf1a3162fcd403ba574bebae`.
NEXT ACTIVE: MUSE separately activated ONE deny-only SOURCE implementation in
`01a1281d-854c-7a12-8ecf-58b13e64cd67`, after PATH acceptance. Extend the existing
acquired projection with rule plus canonical identity/source; no second loader.
Scope includes before-cache/provider denial, typed attempted=false refusal and
partial evidence, upfront unsupported parent guards, connected Core outputs/docs
and NOT_RUN regressions. RUBY authored the policy_pack-only typed Rust/Deny map,
accessor and five parsing/identity test sources; SCOOBY extended the existing
retained context with owned restrictions and rederived identity/source/path,
plus two projection test sources. All are NOT_RUN; complete parent admission
and public propagation remain incomplete. SCOOBY raised the concrete
Mise-before-acquisition and replay-before-policy ordering conflict to MUSE;
MUSE cleared relocation of the existing producer ahead of effects in
`01a12825-4720-7f93-be5b-50d6d05c85e9`, explicitly accepting earlier policy-load
error timing and canonical parent workspace acquisition. The connected protected
secret-delivery exception was explicitly cleared: preserve the existing pure
Step 6 terminal refusal before policy acquisition/classification, with no
intervening activation/replay/probes. Its unacquired context stays unavailable,
never acquired-absent or a Rust policy denial. Task replay preflight now acquires
before observation and preserves load errors without observing inputs; broader
entrypoint integration remains incomplete. MUSE's carrier correction in
`01a1282e-8cfc-74a1-9871-1c29ce5c22d7` recognizes that the public protected
refusal projection can continue through the existing V4 bridge. Preserve
canonical invalid/unsupported-carrier refusal before policy loading; valid
non-activating carrier validation precedes one invocation-policy acquisition.
A captured Rust deny refuses that restricted carrier route before authority
activation, with no readiness probe; no-rule continues the existing bridge.
This restriction does not promote protected effect-policy/provider provenance.
VLAD design review `01a12884-f776-70d0-81f5-9f4d85da3be8` found no P1/P2/P3;
MUSE accepted bounded SOURCE implementation of the ordering. Canonical local
carrier-store selection/validation may precede the gate; full grant evaluation
must not, because its proof-parent capability path mutates/adopts the descriptor
and writes a challenge. Final authority/freshness validation remains later.
Runner before-cache denial, partial-receipt attachment, static
Doctor declaration, policy-review schema and additional regression sources
are now authored, but not accepted or executed. Normal Run now retains one
acquired policy before Mise/replay, with non-activating carrier discovery before
protected policy acquisition and typed restricted-carrier/proof-parent refusal
before the full grant evaluator. The protected admission's effect policy remains
unacquired; invocation restriction acquisition does not promote its provenance.
Lifecycle/runtime proof parents acquire through the shared producer, refuse a
captured restriction before replay/artifacts/observers and reuse actual absence
or presence; runtime Mise follows the guard. Refusal receipts carry prior
prerequisite/backend/target evidence, typed NOT_RUN fields and denied-task
attribution; public nested evidence does not duplicate the step graph.
Qualified concrete VLAD review `01a128af-848f-7b40-b28a-5e2ebbe80cb5` held
the checkpoint for two P2s: early unsupported-route refusal dropped structured
policy attribution, and qualifying automatic V4 had no integrated deny/load-error
Run regression. MUSE cleared their source repair. The candidate now adds distinct route-refusal evidence to requested
Run receipts and proof admission JSON, using canonical receipt construction with
execution input observation disabled. This does not acquire protected provenance,
assert readiness/fulfillment or create typed-effect archival eligibility.
New actual-path automatic V4 deny/load-error regression sources use a retained
socket descriptor, unchanged PATH/store bytes, zero capability-acquisition and
authority-activation counters, no challenge bytes and absent task/state artifacts.
Serialization/schema regressions retain root/selection/policy attribution and
reject positive execution or authority fields. All are NOT_RUN. Qualified VLAD
R2 `01a128ba-ce0d-7333-bda9-00ca408e6886`, final
`msg_0074fa586a19efbe016acaf0b5226081919dc307913bd14a7c`, found no remaining
P1/P2/P3 in this narrow checkpoint and closed both P2s at source-review level.
MUSE independently reconciled all 13 frozen hashes and HEAD and accepted this
bounded Run/proof admission/refusal source checkpoint in
`01a128ba-d1c4-7812-a06b-443f046e85b5`. Historical acceptance identities are
retained in the follow-on plan; they precede this handoff update.
ACCEPTED CHECKPOINT: ONE Up admission-handoff/selected-local-parent-guard SOURCE
increment, SCOOBY sole writer. Source now retains successful command admission
through canonical Up, replay preflight and same-context instance prerequisites.
All selected local root/member target/admission pairs precede Mise and the first
target effect; distinct child policy roots remain distinct. MUSE separately
cleared directly reached in-process runner seams in
`01a128c7-c39e-7d32-ba20-f11decb8a6c5`: captured/streamed prepare/setup/run/attach,
dependencies/hooks retain the matching policy projection and separate policy_env.
This is admission/replay/readiness/restriction retention only: canonical final
task-environment, selected backend-fulfillment and shared-backend environment
loaders remain fresh at their established phases. No global full-pack no-reload
or zero-earlier-effects promise is made for their later failures.
Restricted dry-run/canary/archive/detach Up refuses before effects, with
non-observing structured evidence and no typed-effect archive eligibility.
Refusal previews are declaration only. Canaries/eligible typed archives reuse the
acquired pack. Ordinary detached children remain separate invocations with fresh
policy acquisition; parent-snapshot continuity across that process boundary is
explicitly NOT_PROVED and requires separately reviewed design, not new IPC or
snapshot files here. NOT_RUN regression sources cover retention after source
removal/change, actual absence/error, later-member guards, untouched descriptor/
PATH/no challenge, both in-process modes/declared isolated overlays/dependencies/
hooks and instance prerequisites under stable valid policy. Separate NOT_RUN
cases assert fresh-policy load failures before body/hooks, not before all Up effects.
Qualified VLAD Up R2 `01a128d6-6c99-7472-bd1c-2b57197ce5cf`, final
`msg_0074fa586a19efbe016acaf7a398548191a9bab58c1469bb54`, found no remaining
P1/P2/P3 in the bounded repair and closed the three R1 findings by source
inspection. MUSE independently reconciled the frozen candidate and accepted
only this bounded source checkpoint. Pre-handoff reviewed identities and the
ownership/effect map are retained in the follow-on plan. Compilation/tests/runtime,
public propagation, whole-slice acceptance and bootstrap remain HOLD/OPEN.
ACCEPTED SOURCE only: narrow workspace-parent SOURCE R2 repair. Qualified
VLAD R1 `01a12923-b3aa-7871-8d86-8471a83d6e55`, final
`msg_0074fa586a19efbe016acb0c4815148191877b385ea28f3681`, found P1 portable
test-helper gating and P2 invalid child env fixture, missing protected workflow
recipient and empty bare-relative refusal root. MUSE reconciled the frozen R1
HOLD and cleared only these repairs. Source now uses a portable filesystem/YAML
helper, Unix gates for shell/FD cases, validated `env.vars` and intended child
policy-failure assertions, and validated task/workflow applicability. The workspace
caller alone maps an empty negative root to `.` without changing the envelope
path, policy identity/source/path, other roots or shared runner. NOT_RUN public
bare-relative Up/Run coverage retains CWD/FD/PATH and no-acquisition assertions.
Qualified VLAD R2 `01a1292d-93b4-7492-9c7e-5af0823d5888`, final
`msg_0074fa586a19efbe016acb0e5095748191afa215c741413e6f`, found no new blockers;
MUSE independently accepted only SOURCE R2. R1 HOLD/reviewed hashes remain
historical in the follow-on plan; compilation/tests/schema/runtime are NOT_RUN. MUSE
accepted DESIGN R2 after qualified VLAD `01a12912-a75a-7912-839a-75af835a9e00`,
final `msg_0074fa586a19efbe016acb0700182c8191a1969abc34a2bbdc`; the marker P2
is closed only at DESIGN level. Historical reviewed hashes/map remain in the
follow-on plan. Source now stages canonical selected-parent acquisition with
env > ancestor org > actual loaded workspace.policy > observed absence; no
unrelated default fallback or inherited proof-child absence shortcut. MUSE also
cleared selected DIRECTORY normalization after env selection, preserving the
selected basename/file-symlink directory and original output spelling. Generic
child/proof discovery is unchanged; no process env mutation is introduced.
Both parent modes guard before progress/channels/workers/mkdir/clone, preserving
pure local protected-carrier refusal before policy loading. Owned parent policy
is never substituted for child policy or its inline overlay. Negative parent
failure output has distinct acquisition/error/unavailable attribution, no synthetic
child receipt or authority/fulfillment evidence. Connected Core docs/schema/tests
are staged; RUBY's disjoint two-doc assignment is integrated. NOT_RUN regressions
cover actual captured JSON jobs=2 / streamed Text jobs=1, marker conflicts, selected
file/relative/symlink precedence, optional/acquisition-only children, retained
FD/PATH/no challenge, protected no-load precedence and ordinary child overlay.
Schema source negatives are failure-branch projection checks, not executed
whole-document or produced-output conformance. All-local-child upfront admission
and unknown pre-clone policy remain explicit gaps/limits; final env/backend
freshness and detached continuity are unchanged. ACTIVE NEXT: ONE integrated
local-child command-admission preflight/owned-handoff SOURCE checkpoint authorized;
SOURCE R1 historical HOLD / SOURCE R2 accepted at source level; validation HOLD.
DESIGN R2 accepted after qualified VLAD
`01a1293c-ae8f-7593-a4b1-e9533a7e1f08`, final
`msg_0074fa586a19efbe016acb11b8ccdc8191b2be66f84d7ca52b`, and independent
MUSE reconciliation. Reviewed historical identities remain in the follow-on plan.
Canonical plan maps actual present local targets, Run mappings/Up workflow roots,
separate owned admissions into both modes, required terminal failure before any
worker/acquisition, optional child-owned WARN/no body, and truthful failing-child
negative output. DESIGN R1 is historical HOLD: qualified VLAD
`01a12937-c4d6-73c2-8742-3626530b92d3`, final
`msg_0074fa586a19efbe016acb10a6794c8191b77643e85bf459eb`, found child failures
could not borrow parent-only Rust carrier/code/subject schema rules. MUSE granted
only docs repair. R2 specifies a distinct child_admission branch with actual
failing-child selection, independent acquisition state, concrete failure kind and
required/forbidden evidence rules; existing parent failure branches are unchanged.
Typed policy denial, typed realization unavailability and unsupported-Up Rust
refusal are distinct. Typed evidence is copied only when its canonical helper
actually emits it; no invented carrier, identity or archive. Existing parent-level
protected refusal cannot become optional
WARN. Run Rust restrictions stay conditional; blanket denial breaks non-Rust/
satisfied-readiness compatibility. Up/typed terminal failures and load errors are
the proposed initial bar; conditional readiness denial before all workers remains
OPEN. An independent-owner acquisition seam must ignore an unrelated inherited
proof-child marker without changing generic child/proof behavior or duplicating
the canonical loader/admission; MUSE accepted this bounded design invariant only.
Acquisition-only/uninspectable policy, later drift,
final fresh phases and detached continuity remain bounded. Production/tests/schema
enter only the integrated authorized source write set: policy_pack/effect_orchestration/
commands/workspace_output, in-file regressions and connected schemas/Core references.
Workspace stays read-only. MUSE granted a narrow runner.rs exception only for
pure exhaustive `RunError::variant_name` and portable source assertions, preserving
the outer wrapper variant; all retained-policy/readiness/dispatch/runtime behavior
stays read-only. Prior reviewed runner hash is retained in the follow-on plan.
Independent-child canonical acquisition, both-mode upfront guards, owned target/
admission worker handoff and disjoint negative output are authored; required initial
failures stop scheduling, optional child-owned terminal WARN never dispatches.
Unknown-before-clone and final-policy/body failures use ordinary reports. Run Rust
deny remains conditional, with no new readiness probe. Regression sources are
NOT_RUN; RUBY returned and SCOOBY integrated the five-file schema/test/reference
reservation. The schemas separate parent/child roots and lock failure/acquisition/
carrier shapes without claiming relational ownership. Constructor source tests
retain mapped-task/child-root/pack linkage; Up preserves canonical explicit/default
`workflow@instance` selection through roots, profile adjustment and refusal evidence;
typed policy and secret admission receive the resolved base workflow identity.
Projection tests explicitly expose cross-field identity/selection mismatches that
schema shape alone cannot reject. These are source assertions, not executed proof.
Required preflight failures may use the negative
branch only before dispatch; after-clone/final-phase/body failures retain normal
partial-effect reporting. Qualified SOURCE R1 review
`01a12a25-bdb9-7923-88e5-47301d66952c`, final
`msg_0074fa586a19efbe016acb4e24ce5c8191837e2847d98e1fd7`, held one P1:
passing the instance selector to base-name typed/secret consumers lost exact
workflow applicability. MUSE independently reconciled and cleared only a narrow
commands/in-file-test/plan/state SOURCE R2 repair. Explicit/default/omitted selector
regressions now require the intended workflow-scoped deny rule and workflow-only
secret refusal before policy acquisition, with no invented decision/carrier.
All assertions remain NOT_RUN. Qualified SOURCE R2 recheck
`01a12a2e-74c9-7252-973b-ceb860170658`, final
`msg_0074fa586a19efbe016acb4fa6456c8191a490d35e0ac6df73`, closed the P1 by
source inspection with no remaining P1/P2/P3 in the bounded repair. MUSE independently
reconciled all 17 unchanged packet hashes and accepted SOURCE R2 at Core
`8926ddb1fc0e4a225059c48375fc6e1185081b1e`. Reviewed source/plan/state identities
are retained in the follow-on plan. No isolated unused loader checkpoint was accepted.
The combined offline DESIGN R1 packet is accepted as a HOLD inventory after
qualified VLAD `01a12a38-8fab-7213-a99a-8de4dfd57cd2`, final
`msg_0074fa586a19efbe016acb528625f8819183378bd334e7376a`, and MUSE's independent
HEAD/delta/33-digest reconciliation. No execution-ready packet is claimed.
DESIGN ACCEPTED: dedicated Linux bootstrap-owner DESIGN R3-R3. Qualified delta
review `01a12a87-383d-77d2-bb28-ad81d56720c0`, final
`msg_0074fa586a19efbe016acb66492a5c8191b544a2829c2a2b0e`, and MUSE's independent
reviewer-turn/HEAD/delta/hash/boundary reconciliation close original interface P1
at DESIGN level, VLAD's P3 and MUSE's distinct P2. No remaining P1/P2/P3 in this
bounded design; no source or execution acceptance. Reviewed frozen identities
are retained in the plan and precede this factual acceptance-only record update.
SOURCE R3 ACCEPTED / VALIDATION FAILED-STOP: Bobai directly approved implementing the controller,
unit template and test source, followed by MUSE review, with no builds or infrastructure
(human message `01a12a91-091c-7b73-b590-bcf792164b2d`, turn
`01a12a91-08e2-7e60-b314-e8ebba33104f`, independently verified by MUSE).
Write set is ONLY `.github/ota/build_retained_driver.py`,
`.github/ota/retained-driver-owner@.service`, `.github/ota/test_build_retained_driver.py`
and canonical plan/state. R1 was HOLD after qualified VLAD and MUSE source inspection.
R2 repairs the scratch-search P1 and six connected P2s: transitive link admission,
success-looking partial publication, terminal raw log draining/retention, reviewed
HEAD/delta/untracked/complete inventory and self-contained Git provenance, nonblocking
candidate special-object refusal, and dot/parent path refusal. A single protected
finalization-candidate carrier PLUS independently observed successful same-generation
postflight exit is the sole completion predicate; no record alone claims finalized
success. Outer postflight cleanup/resource closure and driver admission remain separate.
R2 review confirmed those source repairs but retained two P2s: only Git HEAD labels
were checked, not object closure, and a mock accessed Linux-only os.P_PIDFD unguarded.
MUSE cleared one bounded R3 pass: private streaming stdlib-zlib validation of the
supported loose-object HEAD commit/tree/blob/parent closure, protected real objects/refs
directories, strict framing/hash/type/compressed EOF and bounded graph/decompression;
packed/unsupported/exhausted inputs refuse. SHA256 inventory/operator approval remains
the trust anchor; SHA1 is only legacy Git object consistency. Other Git refs/full
semantic correctness and installed compatibility are not claimed. Default lifecycle
and finalization mocks supply their Linux interface explicitly; real descriptors remain gated.
Qualified non-author VLAD R3 recheck `01a12ac9-3006-79e3-9b6d-7b595adc7719`, final
`msg_0074fa586a19efbe016acb7768c5808191965630b04ce9370c`, and MUSE substantive
reconciliation `01a12ac9-33b5-78e1-a56c-b9a274f53d66`, final
`msg_0074fa586a19efbe016acb78f927188191bea57dd8a2cac125`, close both remaining
R2 P2s at SOURCE level. No remaining P1/P2/P3 in this bounded review. Frozen
pre-closure identities are retained in the plan; this update records acceptance only.
The approved source implementation slice is complete; the one cleared validation attempt
failed before target checks. No validation gate is accepted.
Bobai's fresh bounded-validation request (human
`01a12b8f-b0e6-7ea1-bd66-88e5a37b3fe0`, turn
`01a12b8f-b0a2-7bf0-a36a-ee5204dcc607`) was independently verified by MUSE.
MUSE selected preparation of one fresh cached-Linux Python-only packet, not
the unqualified Darwin route or a renewal of the closed kernel experiment.
Target: 41 authored methods in eight explicit mock/pure classes, no discovery,
with both LinuxDescriptorTests omitted, in-memory syntax checks and trusted unit
text assertions. Mocked production finalization is allowed; operational role
dispatch, real descriptors/services/kernel actions and builds remain excluded.
The retained R11 launcher cannot support this packet unchanged: fixed old
four-input/11-test selection, 65,536-byte member cap versus 73,923-byte controller,
32 MiB scratch versus proposed 8 MiB, no 10-second CPU limit, and a capture-only
30-second deadline rather than total creation/test/cleanup. Its exact digest and
required controls are recorded in the follow-on plan. No compatible reusable
ordinary launcher was identified in the narrowly inspected retained routes.
No target import/test, daemon probe, container creation or source adaptation ran.
MUSE subsequently cleared ONE source-only fixed-purpose R11 adaptation in the
private `retained-controller-python-validation-r1` packet under
`/Users/bobai/.codex/artifacts/2026-10-11`; historical R11 remains unchanged.
R1 parent/bootstrap were source-prepared and frozen for qualified VLAD then
MUSE review. Both reviewers found three parent P2s: reaping before owned group
teardown, deadline verdict before final publication, and unguarded retention faults.
MUSE cleared ONE source-only R2 successor; immutable R1 and candidate remain unchanged.
Current packet is `retained-controller-python-validation-r3`, parent
`a6f4b48591a02b8916c0d971ae35ffc9a7b0e39e7d4d9c83eb91139b23109394`,
unchanged bootstrap `80430be7f07675a860d757168427de138ef55f06ff657eed17e4f5349fb25952`.
R2 retains the unreaped CLI session leader through group kill, then waits and
requires bounded group ESRCH without destructive signaling after reaping.
Qualified VLAD and MUSE closed the three original P2s at SOURCE level. MUSE then
identified inherited SIGCHLD ignore/autoreap as an additional P2 admission gap;
R3 actively establishes/verifies SIG_DFL before any Popen, with one synchronous
reaper and no competing handler. R1/R2 remain immutable. Qualified VLAD R3
`01a12bd9-b4da-75d2-8bf3-4dd665f5e792`, final
`msg_0074fa586a19efbe016acbbd1295a0819190b9d814d8deb36d`, and MUSE substantive
reconciliation accepted this packet at SOURCE level with no remaining P1/P2/P3.
MUSE cleared ONE exact timed R3 contained Python attempt under the fresh human
request, mandatory cleanup and independent runtime-evidence review. All frozen
control/runtime/input/Action/HEAD/delta identities were rechecked before launch.
The single timed attempt is consumed: parent exit1, trusted tool wall1.426203083s,
timing real1.52s. After the first read-only cached-image query, process-group
handling returned PermissionError/EPERM. The exact failed operation/root cause is
not_proved. No staging/container/bootstrap or candidate import/test ran; all41
selected methods and target syntax/import checks remain NOT_RUN. The raw image
query log and sealed attempt/result/copied outer-tool response are retained in
the R3 packet; hashes, lengths and observations are in the follow-on plan.
Independent current exact label/name container checks, staging noentry/no-symlink
and matching query-process checks are absent. However attachment_cleanup_okfalse
and missing exact child PID/birth evidence leave owned-group teardown NOT_PROVED;
these absence checks do not mean all cleanup passed. Candidate/Action/HEAD/non-record
tracked delta are unchanged. Qualified non-author VLAD failure review
`01a12be3-63fd-79b3-a0b6-f1fb13800dce`, final
`msg_0074fa586a19efbe016acbbfb97ae88191be7b73a8a65fdfe2`, frozen closure recheck
`01a12be5-7b68-7bb1-9321-ca529627a2aa`, final
`msg_0074fa586a19efbe016acbc005517c8191889d879f2a263598`, and MUSE substantive
reconciliation ACCEPT factual FAILED/STOP closure only. No new closure findings.
The immutable R3 diagnostic P2 was not repaired by failure closure: its parent
omits retained child identity/status/phase when group cleanup raises. Source R3
acceptance is unchanged; validation gate HOLD. R4 source-only closure is below.
Bobai subsequently approved ONE SOURCE-only diagnostic successor directly in MUSE's
question `call_ebe4618de2734a6d965170ba7345a909`, answer
`Approve source-only diagnostics (Recommended)`, human
`01a12c05-0f13-7a40-8bb1-31e4eed3618d`, turn
`01a12c05-0edf-7930-933a-c1d2d6232adb`; SCOOBY independently verified the reply.
This is a new source grant, not revival of R3's consumed execution grant.
Private validation R4 SOURCE ACCEPTED retains each owned Popen PID/role/ordinal
before collection, cached return status, explicit birth_identitynot_proved,
operation phase and separate primary/collection-cleanup/close/final-cleanup errors.
MUSE cleared the narrow module-level placement of existing finish_group for direct
mocked regression source, preserving signal/wait/ESRCH/rethrow/deadline behavior.
Fixed call graph has at most13 launches, under16 recorded-child capacity; diagnostic
records are bounded32KiB independently from unchanged64KiB captured output.
Six diagnostic mock cases are source-authored, including sealed-result wiring with
all group cleanup attempts failing. They and all41 candidate methods remain NOT_RUN.
Trusted AST-only parsing passed for R4 parent/bootstrap/test source. Frozen hashes
and lengths are in the plan. Qualified non-author VLAD source review
`01a12c0d-d66f-7de2-8883-48b8749e99d1`, final
`msg_0074fa586a19efbe016acbcac2aa908191a38166083d981289`, historical execution
wording P3 recheck `01a12c10-a661-72b3-8d3a-7d94691c3eae`, final
`msg_0074fa586a19efbe016acbcafa3ef88191ba45a18d255526d4`, and MUSE substantive
reconciliation ACCEPT R4 SOURCE ONLY with no remaining P1/P2/P3 in this slice.
The missing-child-diagnostics P2 is closed AT SOURCE ONLY, not a demonstrated
runtime repair, native birth attestation, EPERM root cause or historical teardown
proof. Accepted pre-stamp identities are retained in the plan. This source grant
is complete; no execution artifacts or runtime work were authorized by that grant.
R3 failed evidence and bootstrap/candidate/unit/Action remain unchanged. No EPERM
suppression, native birth discovery, new syscall, probe, test/import execution or retry.
Bobai's fresh MUSE-chat request to discuss exact bounded R4 validation (human
`01a12c2d-bbaa-7113-b68a-17ef6222f9ab`, turn
`01a12c2d-bb89-7f02-a2d4-91d8b4a36665`) authorizes discussion/proposal only;
SCOOBY independently verified it. ONE unchanged-parent attempt is PROPOSAL
ACCEPTED / R4 SINGLE-RUN GRANT CONSUMED / FAILED-STOP. Qualified non-author VLAD
proposal review `01a12c33-da38-7922-9dcf-53bb51b9e49c`, final
`msg_0074fa586a19efbe016acbd453c93081918f232a299d06bdda`, and MUSE substantive
reconciliation found no P1/P2/P3. Bobai directly approved MUSE action question
`call_81d85023fdf64e938404fe5b57447d4b`, answer
`Approve one R4 attempt (Recommended)`, human
`01a12c46-7990-79c3-a1c5-86e4c3983d5c`, turn
`01a12c46-7971-7623-9cda-318a586accd7`; SCOOBY independently verified the reply.
MUSE relayed exact single-run clearance, no broader work. All frozen proposal/
runtime/input/HEAD/delta identities were rechecked unchanged before dispatch.
The exact command ran ONCE: terminal exit1, tool wall0.398876667s, trusted timing
real0.50/user0.25/sys0.08. Dispatch consumed the grant; absence of
attempt.json would not authorize retry. R4 retained child88588, ordinal1,
role docker:image, and three separate EPERM/errno1 failures at os.killpg(SIGKILL)
in primary, collection_cleanup and final_cleanup phases. Cached returncode is
null, birth_identitynot_proved and group_absence_observedfalse. The operation is
observed; OS root cause, actual child terminal status and historical group teardown
remain NOT_PROVED. Only first image-query stdout was collected; no container,
staging or bootstrap/candidate import/test ran. All41 candidate methods NOT_RUN;
six diagnostic cases NOT_SELECTED/NOT_RUN. Sealed R4 attempt/log/result and exact
copied outer response/independent absence observations are retained with hashes in
the plan. Combined raw log+outer1665 bytes; result1958 bytes bounds child metadata
below32KiB. Result remains provisional, with tests_passed/candidate_passed/
overall_passed/usable_driver false and attachment_cleanup_okfalse.
cleanup_absent/staged_inputs_absent are true but do not imply group teardown. Independent exact label/name and stage
checks are currently empty/absent; acquired PID88588 and exact matching query
process are not observed. Current absence does not repair failed group teardown.
Qualified non-author VLAD runtime-evidence review
`01a12c4c-485b-7473-b6a4-ab1cbd702dda`, final
`msg_0074fa586a19efbe016acbdacd406c8191bec2d527b3dc6bba`, wording-only recheck
`01a12c4f-fa3a-7e42-8582-bed84a1f1bc9`, final
`msg_0074fa586a19efbe016acbdb2ae0b48191a0eb2e4e44cbed98`, and MUSE substantive
reconciliation ACCEPT R4 DIAGNOSTIC OBSERVATION ONLY / factual FAILED-STOP closure.
Both P3 wording corrections are closed; no P1/P2/P3 within this diagnostic scope.
MUSE independently checked original native exec evidence against retained copies;
accepted pre-stamp identities are in the plan. Validation HOLD, no controller/
41-method acceptance. Child wait/birth/historical group closure and OS root cause
remain NOT_PROVED. Subsequent ONE read-only source investigation was cleared by
MUSE in turn `01a12c8a-9f54-72c2-9948-9a563e7bab70` under independently verified
Bobai 'Lets work' message `01a12c89-efc1-7852-8065-40f86f39bdc2`.
MUSE accepted SOURCE ASSESSMENT ONLY in turn
`01a12c91-515e-7ab2-aebf-a9b378f29722`, final
`msg_0074fa586a19efbe016acbecb980248191a8011c8105d9bc2e`, no P1/P2/P3:
pinned Apple XNU
`xnu-11417.101.15` group signalling excludes SZOMB members and its POSIX branch
can return EPERM with no eligible target. That makes an exited/unreaped group
a supported hypothesis, NOT the demonstrated R4 root cause. Installed Python
source confirms returncode is cached; null and stdout EOF do not establish child
liveness or terminal status. Host kernel/libc mapping, policy denial, native child
state and historical group closure remain NOT_PROVED. Exact official citations,
installed source identity and proof recommendation are in the follow-on plan.
MUSE then cleared ONE canonical docs-only diagnostic proposal covering live,
exited/unreaped and mixed groups before any implementation. Proposal recorded
in the follow-on plan; independent VLAD recheck and MUSE reconciliation now
ACCEPT CORRECTED DESIGN ONLY, no remaining P1/P2/P3 within that scope.
Installed pyconfig.h declares HAVE_WAITID; this is configuration evidence, not
runtime availability or enforced containment. Proposed native os.waitid wrapper
avoids a new helper/FFI ABI. Fixed direct children preserve single-reaper ownership;
topology differs from R4's new session and cannot diagnose R4 by itself.
VLAD design HOLD identified inherited-session membership (P1), gate-writer
ownership (P2) and blocking reaping (P2). MUSE agreed narrow docs-only repairs:
verified private F session before fork, exact per-role FD closure/_exit, and
terminal evidence plus budgeted WNOHANG reaping. Qualified recheck
`01a12c9f-e63f-74f0-bd33-5ff5c8d2326c`, final
`msg_0074fa586a19efbe016acbeffb5e8c819195dbf90cbc96b561`, and MUSE
reconciliation `01a12c9f-e6a1-7391-9f25-74e9b3997691` accept the corrected
DESIGN ONLY. All three findings closed; configured non-author gpt-6.1-sol/high
independently verified, not provider-attested. Accepted pre-stamp hashes in plan.
After that closed design slice, Bobai explicitly said 'lets work'
`01a12f2e-7134-7653-b69b-0c174b527383`; MUSE independently verified it and
cleared ONE SOURCE-ONLY diagnostic plus mocked-test module in turn
`01a12f2f-06eb-7e20-835a-2e001f213ba3`. Exactly two private source files are now
authored/frozen under `/Users/bobai/.codex/artifacts/2026-10-12/owned-group-signal-diagnostic-r1`;
hashes/lengths/protection and source boundaries are in the follow-on plan.
Diagnostic e4325252... (20356 bytes) and test source d49fee97... (14575 bytes,
21 mock methods) are regular/singular bobai:staff0444/non-executable.
SOURCE CANDIDATE ONLY: NOT_PARSED, not imported/executed; 21 new mocks NOT_RUN.
VLAD source review `01a12f4f-34ee-7af0-8b9c-80c95d831ae6`, final
`msg_0074fa586a19efbe016acca1bd6f54819198050554cdebe91e`, is SOURCE HOLD:
P2 fresh deadline/cancellation admission immediately before fork; P2 native
core-dump bit must agree with CLD_KILLED/CLD_DUMPED. Exact configured non-author
gpt-6.1-sol/high independently checked, not provider-attested. MUSE acknowledged
the HOLD and requires R1 unchanged. MUSE reconciled in turn
`01a12f6c-94fc-7c03-ae6c-6f1c802596a7`, final
`msg_0074fa586a19efbe016acca7b849e881919d0e7dff3a2f5cbd`: GO for ONE
source-only R2 packet under the verified source-authoring grant. Scope: fixed R2
ROOT, fresh pre-fork admission, callable native WCOREDUMP/exact event-class
reconciliation, and bounded coarse phase + primary type/errno + separate retention
failure in the existing <=512-byte outer carrier. The latter is MUSE P3, not a
VLAD finding. Directly connected mock regressions only; preserve R1 bytes.
R2 is now authored and frozen source-only at
`/Users/bobai/.codex/artifacts/2026-10-12/owned-group-signal-diagnostic-r2`.
`diagnostic.py` is 21,482 bytes / 496 lines / SHA-256
`fadd4a02cc1edbcd77de1d08ae750e6db8308294b14da26fbc4edb82442f9d42`;
`test_diagnostic.py` is 20,665 bytes / 393 lines / SHA-256
`ef5a5429e60486be688d1b718a0680a243d9cea6b8e074924f3e13b378e82c06`
with 27 authored mock methods. Both are regular, singular, bobai:staff, 0444 and
non-executable. R1 hashes remain unchanged. R1-to-R2 unified-diff stream hashes are
`dc799974af5c6e76cb76a9831896f52fef6992ac65d15df1b88df85705a3cf26`
for the diagnostic and
`13593ab5d0bf191ff19c39da1810cf7ac3b86099d7076ce8503b312eecdd79ef`
for the mocks. The source-only repair adds the fixed R2 root, fresh cooperative
admission immediately before fork, callable native `WCOREDUMP` admission plus
exact `CLD_KILLED`/`CLD_DUMPED` reconciliation, and bounded coarse primary
phase/type/errno separated from retention failure in the existing outer carrier.
Connected mocks cover expiry/interruption during pipe preparation, matching and
contradictory core indications, unavailable/noncallable primitive, pre-packet
Refusal/OSError separation, sticky retention-only failure and carrier/cap bounds.
ALL 27 mocks remain NOT_RUN; the source was not parsed, imported or executed.
Qualified VLAD read-only R2 review
`01a12f7d-8402-77b1-8f3b-c38d8d52f47f`, final
`msg_0074fa586a19efbe016accadd15a8081918088a594755f56dd`, found one P2:
two execute-path mocks left process-global `os.umask` live, and one P3 stale R1
mock count. The source-only correction now mocks `os.umask` in both paths and
labels the current R2 count; no production diagnostic bytes changed. R2 was
refrozen at the identities above. Qualified VLAD delta recheck
`01a12f89-4642-7f10-9514-2746af6ab732`, final
`msg_0074fa586a19efbe016accae8e4d6481919a46b48b60f51e4e`, closed both
findings with no remaining P1/P2/P3 and returned SOURCE GO for MUSE
reconciliation only. MUSE substantive reconciliation
`01a12f8a-bf85-7403-ac3e-9504b3ee6ee6` independently rehashed the frozen
packet, read both qualified VLAD finals, found no remaining P1/P2/P3 and marked
the exact R2 packet SOURCE ACCEPTED ONLY. The accepted pre-stamp canonical
identities were state
`4241264e0e8e5bd2a274703193123b55cc208140b29807b7d7ef5689a40a0e84`
and plan `20fd0f4fe63a252f3c6c2da36dbdda3b885f8ab1d7ae23c627ba4388618ea9bd`.
This closes source review only. Next is read-only identification and freezing of
an existing contained path for syntax plus exactly 27 R2 mocks. That read-only
inspection is complete and no unchanged path is suitable. Historical R11 is
already recorded incompatible. R4 `parent.py` lines 29-33/170 and
`bootstrap.py` lines 14-64/141-145 hard-bind three Core inputs, eight test
classes and exactly 41 methods, not the two R2 files/27 methods. Its child-group
collector at `parent.py` lines 115-145 is also the route that failed on the first
cached-image query with retained `os.killpg(SIGKILL)` EPERM before container
creation. Reusing it unchanged cannot establish R2 syntax/mock validation; changing
bindings/selection or the launcher is new source work. Validation remains HOLD
pending MUSE selection of the smallest fixed-purpose adaptation. Actual parsing,
import or test execution needs separately reviewed exact controls and fresh human
action approval. Native FD/session/wait/limit availability, effective containment,
parent-loss cleanup and historical R4 root cause remain NOT_PROVED.
Runtime NO-GO; existing41 controller+6 R4 diagnostic cases remain NOT_RUN;
validation HOLD; Step7 OPEN/R1 BLOCKED. No controller/R3/R4/product source change,
AST/parser/test/probe/signal/build/container/VM/infra/spend/commit/push. STOP at
frozen source review boundary. Any later native signal/runtime action needs
reviewed exact controls and fresh human action approval.
Exact command, identities, limits and two outcome
ceilings are in the follow-on plan. No executable build or preflight probe;
private diagnostic source staging is distinct from R4 repair. The six parent-diagnostic cases are distinct from the41
candidate methods and are NOT selected; future contained testing needs a separately
reviewed fixed binding, not an unsandboxed import or silent suite expansion.
MUSE then selected one compact DESIGN-ONLY successor in the existing follow-on
plan. It binds only the two immutable R2 files and a closed 27-name
`DiagnosticTests` suite; nine separately named/count-bound host collector mocks
cannot join that target gate. The proposed collector replaces cached
`Popen.returncode` inference with exact-owned non-reaping `waitid`, one bounded
signal attempt only for a still-bound live timeout/cancellation, group decisions
while the leader remains unreaped, and bounded exact-PID/status reconciliation.
EOF, ECHILD, changed ownership, contradictory status, denied signal and unresolved
group absence all fail closed under the plan's outcome table. The accepted R4
containment ceilings are preserved without a different owner, launcher, engine,
host or larger budget. This is design text only: no successor source exists and
no parsing/import/test/process/engine action is authorized. Next is qualified
VLAD read-only DESIGN R1 review, then MUSE reconciliation and a decision on one
source-only adaptation.
Qualified VLAD DESIGN R1 review
`01a12f99-abd1-7010-abea-e905db328e4f`, final
`msg_0074fa586a19efbe016accb59faa3c81918750ed11c01d6d95`, returned DESIGN
HOLD with P1/P2. P1: no sound generation-bound group-absence success predicate
exists while the leader remains unreaped; zombie-group `killpg(..., 0)`/EPERM,
leader terminal state and post-reap probes cannot prove exact-owned quiescence.
P2: callable primitive admission must be separated from typed constants and retain
core-aware terminal reconciliation. The exact R2 binding/27-name selection remains
sound, but no validation successor implementation is cleared. Next is MUSE
reconciliation of the named quiescence capability gap; no cleanup weakening,
EPERM reinterpretation, alternate owner/engine/host or larger budget is allowed.
MUSE turn `01a12fa6-6065-73a3-be10-29cf75b03d2a` confirms DESIGN HOLD and
names the gap **generation-bound host-control-group quiescence**. The current
Darwin plus Docker-CLI contract has no admitted pre-reap absence predicate for all
members/descendants; exact-leader wait, numeric session/group identity,
`killpg(..., 0)`, EPERM and post-reap probes do not close it. Callable native
wait/status/core functions are now explicitly separated in the plan from
type-int/not-bool selectors, flags and codes; this wording does not prove host
availability or close P1. A host-helper route would require an admitted stable
generation owner/handle with independently retained quiescence evidence. A
fixed-purpose single-process transport to the same pinned local Engine API is
only a next design hypothesis and needs direct human approval; no socket access,
design amendment, source, probe or execution is authorized.
Bobai has now directly approved one design-only Engine-API alternative through
MUSE question `call_b41280b371f1487b8cfb56fdda83b2cb`. Frozen DESIGN R2 plan/
state hashes were `9539115136ba0a5b93e298fa10f5627920309f9f061f595aee7c1e6f74194c59` /
`c15d5d01d2262798b20f6720d2583eed69d5a317907345c3c471ab4f865274e1`.
The original VLAD review remained incomplete and was superseded. Replacement
reviewer Aquinas `01a12fde-afc2-75c3-9f69-be97af998ad6`, turn
`01a12fde-b021-70f2-95e0-a0556b7e1a8e`, final
`msg_0a529e04e536a801016accc63dab388197ad5611ff3ac14947`, returned DESIGN HOLD
with four P2s and one P3: late mutation outcomes, attach completion/order, the
two-socket cleanup handoff, raw framing bounds and operation-count consistency.
Frozen DESIGN R3 plan/state identities were
`0a28d43b7db2c0619624a710baffc4bedc99d9df569351d71b3a757b0907dee1` /
`aae4c8653caf8689f1e476c914546da5696849be497e66d3d63f65360c50b0c1`.
Aquinas recheck turn `01a12ff0-9ead-7390-98f2-03cecb5d8d64` closed the original
framing/EOF, two-socket/deadline, parser-bound and readback-slot findings at design
level, then retained one P2 and one P3: later-attempt blocking was not global to
every unresolved mutation/cleanup state, and pre-start refusal counts conflated
pre-attach with attach-established cleanup. DESIGN R4 adds one global retained
`recovery_required` latch across every possible late effect and corrects the path
counts to pre-attach <=8 and attach-before-start <=9 without increasing the <=16
total/two-live-socket ceiling. Frozen R4 plan/state identities were
`5db5f44777f4f5acf8d022d1f1050b7142e1bbba11609b98b2147c60ed47d646` /
`6625a5cadef1a84bf754c615e0b56d69110194ba1836c3490b4b4082f6a6f1d8`.
Aquinas recheck turn `01a12ffb-6c96-73e0-b931-cabd9a0b3aa2` closed the count
finding and retained two wording conflicts: absence evidence was still presented
as an alternative to reviewed recovery, and stage retention was unconditional
after possible completed cleanup. DESIGN R5 makes reviewed exact-attempt recovery
the only latch-clearing path; absence observations are inputs only. It preserves
surviving stage objects/identities, retains completed-removal identity/evidence
instead of claiming an extant stage, and keeps partial/unknown removal unproved.
Aquinas exact verification turn `01a13006-f8ac-75a0-988a-f8a87aaa8d89`
returned GO: DESIGN ONLY with no remaining P1/P2/P3 in the exact R5
clarification delta. MUSE final substantive reconciliation turn
`01a13005-ae5c-7fa2-9634-677e212593bd` accepted DESIGN R5 ONLY at pre-stamp
plan/state identities
`caca888078f05b18538747349c750078eb1466f99814392747efd69a4a2ab3dd` /
`bf7b761115d22f58da4bb6d5eeb7a7535f87f23bc2525772f553a1a3ba47eec3`.
This closes the design review chain, not source, validation or runtime. DESIGN R5
pins
API v1.51 on the same retained Unix endpoint/Engine/image, replaces Docker CLI
helpers with one trusted-parent AF_UNIX HTTP transport and makes zero spawned
host control helpers an architectural invariant. It preserves the immutable R2
two-file hashes and closed 27-name target, separates transport regressions,
retains `log-driver=none` with bounded pre-start attach streaming, fixes request/
response/connection/output limits and exact socket handoff, binds one nonce/name/
label/container ID, and makes incomplete create/start/delete responses sticky
unknown outcomes that cannot be repaired by connection close or one empty lookup.
It reconciles frame-aligned attach EOF separately from wait/terminal inspect and
fails closed on ambiguity, framing/version mismatch, timeout, parent loss,
deletion/absence or publication failure. This is design text only: no source,
socket query, probe, process, Engine action or runtime clearance exists. The old
CLI design remains HOLD; all 27+41+6 remain NOT_RUN, installed API compatibility
and parent-loss cleanup remain NOT_PROVED, runtime remains NO-GO, Step 7 remains
OPEN/R1 BLOCKED, and Step 8/V12.2 remain inactive. No public propagation is
required because no shipped product, CLI, schema or claim changed.
Validation HOLD after failed attempt; only bounded retained evidence, read-only
exact-owned absence checks, review and factual closure remain. Builds, infrastructure,
commit/push, source fixes or scope expansion are not cleared.
Result is always provisional/overall_passedfalse/usablefalse; acceptance requires
independently observed successful timed parent terminal including publication,
sealed retention and resource absence. Sticky retention errors exit nonzero with
bounded diagnostics, never an independently accepted visible carrier. Exact timed
invocation/parse predicates and reviewer history are in the plan. Parent AST
parsing passed; R3 and R4 parents each executed once and failed pre-target. All
bootstrap/candidate/test execution remains NOT_RUN. Read-only local facts confirm cached Linux/arm64 image934240a1, pinned
local OrbStack engine29.4.0/cgroup2 with advertised required limit support;
actual effective single-process controls are NOT_VERIFIED until an authorized run.
The packet selects pids1/nproc1, CPU10s, no writable volume/tmpfs, readonly input,
30-second total budget with5-second cleanup reserve and64KiB shared capture.
Exact identities/argv/selection/retention limits are in the plan. A Python audit
tripwire and refusal stubs detect accidental unmocked effects; they are not an
OS sandbox or a guarantee against arbitrary native code. No builds/infra/spend,
candidate correction, retry, commit/push or old experiment renewal. Execution STOP.
All target tests/import/syntax/build/runtime validation NOT_RUN. No build, installation, infrastructure/spend,
commit/push or activation permission. Existing Action and unrelated dirty source unchanged.
Controller covers fixed request/producer/manager binding, stopped-child placement,
UID/capability/FD drop, offline bwrap Cargo launch, recursive worker terminality,
sealed pending copy and separate manager-bound finalization. Tests are source-authored
production-function mocks plus explicitly gated Linux descriptor cases, not a
demonstrated Linux lifecycle matrix. Full installed namespace/manager emergency cleanup,
cache/toolchain sufficiency, descendant/owner-loss fault matrix and effective budget
qualification remain OPEN; separate 512 MiB/artifact and metadata budgets NOT_APPROVED.
The prior R2 HOLD inventory and original undefined-interface P1 are retained in
the follow-on plan as history, not an active existing-owner-only requirement.
Bounded discovery closed the fixed kernel experiment and protected Launcher routes
as incompatible with generic bootstrap under that scope; neither is repurposed.
Bobai directly approved ONE bounded dedicated Linux controller DESIGN-only scope
amendment in human message `01a12a69-f758-7ff3-8686-9270f8a4dd70` (MUSE turn
`01a12a69-f6f6-7ee2-a28a-d74b503b8deb`, question
`call_GO2z7bQ1oybc9rpsDmQYGC9e`). Controller DESIGN R3 is authored in the
existing follow-on plan. Qualified VLAD `01a12a7e-c954-7862-a31d-dc93aa1b89b3`,
final `msg_0074fa586a19efbe016acb64660d08819180dfd09c849552d0`, found no new
P1/P2 and judged the original interface P1 repaired at DESIGN level, subject to
MUSE reconciliation. One P3 stale active/write-set summary is repaired in R3-R2
and closed by qualified recheck `01a12a82-5406-7683-b774-5f94841da8e4`, final
`msg_0074fa586a19efbe016acb6505b2d0819192029f96e1970687`. MUSE then found a
distinct P2 design overclaim: read-only admitted config/initial compiler bindings
do not exclude all descendant commands/config/wrapper-like files. R3-R3 narrows
guarantees to admitted top-level consultation roots and inspected first-party
nested metadata launch inputs; all-descendant compiler/config enforcement and
semantic source-to-binary correctness remain not_proved. No new broker/filter/
instrumentation or sandbox flow. Source unchanged; runtime NOT_RUN. Fixed operator unit
starts root build-owner, a separately placed stopped child runs only unprivileged
contained Cargo, and root postflight independently reconciles PID1/kernel cleanup
before finalizing protected retention. Request/nonce/producer/unit/child/cgroup/
candidate identities and protected journal are concrete proposed interfaces, not
worker JSON authority. Manager emergency cleanup and recursive terminality remain
required; no installed/runtime support proved. RUBY provided disjoint mechanism/
negative-control design advice, not independent acceptance. Original P1 is repaired
at DESIGN level by MUSE final reconciliation; later source authoring approval is recorded above. MUSE explicitly
refined the seam: operator is the first consumer; Action remains byte-for-byte
unchanged, optional non-privileged artifact consumption deferred until needed.
No protected Launcher/provider authority expansion or general attestation service.
Design/inputs review is complete; no new owner discovery or inventory loop.
R3 supersedes R2's Action branch and four-file proposal: future source is ONLY
build_retained_driver.py, retained-driver-owner@.service and
test_build_retained_driver.py under .github/ota, plus canonical plan/state.
Source authoring and SOURCE R3 acceptance are recorded above; execution remains
NOT_CLEARED. Existing Action/default ref-install stays unchanged.
Complete raw/Git/build/toolchain/cache/config binding,
pre-compiler containment, nested offline metadata and post-quiescence trusted
retention remain mandatory. Separate 512 MiB driver cap is PROPOSED NOT_APPROVED;
human clarification waits for a concrete execution packet, not this design.
Manifest limits/cache geometry/Linux qualification remain unmet execution gates.
Prior DESIGN-only first-party bootstrap/provenance checkpoint (historical):
existing build action and all production/test/helper source remain READ-ONLY
until concrete design review and MUSE's exact source-write-set decision. Preserve
ref-build consumers; bind sealed dirty inputs and contain the build itself,
retaining the exact driver artifact/provenance without global driver replacement
or release/runtime-attestation claims. Distinguish immutable dependency/toolchain
payloads from bounded writable Cargo housekeeping, HOME/target/scratch. Freeze
design for qualified VLAD then substantive MUSE review; missing Linux route/cache
requires a precise prerequisite request, not a new wrapper or inferred readiness.
No new product-source slice or execution approval. The first check stays `ota run check --native --agent`
including setup. Exact repaired driver provenance, sealed inputs, Linux isolation,
installed toolchain/cache, operator-owned local Rust-deny pack and cleanup remain
prerequisites, not assumptions. Metadata inspection found only unqualified local
drivers and Darwin toolchain directories; the existing source-build action installs
Rust/Cargo without retaining a qualified driver manifest. DESIGN R1 records the
exact bootstrap/isolation blockers and smallest prerequisite authority in the
existing follow-on plan. No daemon or toolchain/cache probe ran.
No compiler/test/formatter/generator/runtime/
network/provider/VM/spend/commit/push/release/bootstrap clearance. Site command
cards/sync/reference, Learn/FAQ/Glossary, Skills/Examples propagation remain OPEN;
Step7 OPEN/R1 BLOCKED and Step8/V12.2 inactive.
New NOT_RUN source regressions cover untouched proof-parent descriptor/PATH
under deny/load-error, absent-carrier malformed/remote no-load precedence,
automatic V4 local selection/stale identity, retained policy after source
removal, static Doctor/preview limits, output serialization and strict schema
shapes. Accepted source checkpoints cover Up local root/member/canary ownership,
selected workspace-parent acquisition and inspectable local-child owned handoff;
remaining acceptance gaps include conditional readiness before all workers,
unknown pre-clone policy, full backend/OS/legacy dispatch and final-selection
failure coverage, broader integration and all execution validation. The broader
increment is incomplete; no runtime enforcement or commit readiness is claimed.
No second parser/loader or fabricated snapshot is introduced. SCOOBY owns integration and the canonical plan;
RUBY returned file ownership. Combined source needs VLAD review then MUSE acceptance;
material ordering/ownership conflicts must return to MUSE, not be bypassed.
No compiler/tests/formatter/generator/probes/network/provider/VM/spend/commit/push
or release is cleared. RUBY returned exact-source findings:
there is no inspected invocation-bound readiness-conditioned refusal before
Rust fulfillment dispatch, and no qualified non-release bootstrap-only retained
driver artifact path. SCOOBY reconciled the runner fallback, agent/provisioning
policy boundaries and existing source-build action; no source execution occurred.
The staged source candidate owns static PATH/final-selection binding; conditional provider
action restriction remains a separate design decision, followed by separately
authorized first-party driver build/retention and admission. Existing release
publication is not a shortcut.
Rendered Site policy/command reference/Learn/FAQ/Glossary, Skills and Examples
propagation remains OPEN/deferred. The deny map and optional denial JSON are
source-staged, with manual policy-review/receipt schema edits; no ota.yaml
contract shape, Protocol/Launcher or pin change is needed for this restriction.
Bootstrap and the separate no-install refusal mechanism remain NOT_VERIFIED;
no shim, contract-mode change or direct Cargo bypass is accepted. No source fix
qualifies the older executing driver. The offline control route remains HOLD.
Future first execution remains only `ota run check --native --agent`, including
setup; it executes build scripts/proc macros. Full `ota run test --native --agent`
is a separate later gate. The proposed 64 MiB `.ota/state` overlay and remaining
8 GiB-minus-64 MiB scratch preserve immutable source/lock/other policy, but their
effective PATH, mutable mounts, no-swap, deadline/output and cleanup boundaries
are NOT_VERIFIED. No-Cargo preflight cannot qualify Cargo cache sufficiency.
No daemon, container, toolchain/version, Cargo/compiler/tests, network,
install/refresh, VM/spend, commit/push/release is admitted or executed here.
Linux validation will not establish Darwin compatibility. Step7 OPEN/R1 BLOCKED,
native-admission HOLD, Step8/V12.2 inactive; closed experiment unchanged.

### Source-Only Kernel Acquisition Reopening Packet

Bobai explicitly approved ONE source-only preparation on 2026-10-10 after the
closed spike; MUSE independently verified human message
`01a126eb-11e5-76a2-a56f-4b84275a1505`. This is a separate preparation slice,
not revival of consumed execution approvals. Original two-engineering-day
window unchanged and still available at preparation start. Authoring plus
trusted read/hash/text/AST inspection only; no target import/evaluation/tests/
compiler/container/guest/network/VM/provider/spend/commit/push/release.
Frozen corrected candidate: `/Users/bobai/.codex/artifacts/2026-10-10/kernel-custody-reopen-r13-r2`.
Nineteen unchanged R11 members, the per-boot hash gate and R12 fixture/shared header;
22-member runtime manifest SHA
`1cb6700966e594018d1537d898aaa45f24eedf1f320c691a2b86342d0d84bb34`.
27-member source manifest SHA
`f0bdb0608258a58fe752ed72e3993d50bdceafdbe3217af374b36b8366865e9b`;
all28 files0444, regular/single-link/euid, exact hashes/origins verified.
Trusted AST parsing of three Python controls and JSON inspection passed; no
target code evaluated. All12 actual lifecycle cases remain not_proved.
Qualified VLAD R13 review `01a126f4-4502-7bd1-835f-73e766cd017d` found one P2:
hashes checked only before the loop, not each boot. R13-R2 adds only the repeated
build.sha256 gate immediately before QEMU; R13 preserved. Qualified VLAD recheck
`01a126fb-b47f-72b3-8571-ab3e8364d67c` closes the P2, no P1/P2/P3 in the exact
delta; source-ready ONLY. MUSE independently reconciled and cleared the missing
outer-control authoring within the SAME human source-only preparation.
Frozen corrected companion: `/Users/bobai/.codex/artifacts/2026-10-10/kernel-custody-outer-r13-r3`;
four-member source manifest SHA
`089117de71004bfdf7c0f24dc083fb3cd1909bbae36da17559bce0c5a61d4a7d`.
All five files0444, regular/single-link/euid; trusted AST/JSON/hash inspection
ONLY. Live grant template is false; no prepared code or provider CLI executed.
Qualified VLAD/MUSE reconciled five P2s in the original companion (no accepted
P1/P3): first-contact SSH trust, unguarded setup, leader-only group termination,
missing boot/birth generations and overstated cleanup after unresolved creation.
R2 adds authenticated stock guest-attribute host-key bootstrap before SSH,
guarded setup/unreaped-leader group KILL, controller/child boot-birth identities,
and terminal-operation admission/unresolved-create HOLD. Original packets preserved;
qualified R2 reviews accepted four corrections and the failure-group correction,
but retained one normal-group/readiness P2. R3 observes child exit without reaping,
finalizes both paths with the unreaped leader as anchor, makes group ambiguity
sticky HOLD and refuses unsafe readiness polling. Default SIGCHLD/single reaper;
new waitid/FFI runtime compatibility NOT_PROVED, no calls/tests performed.
Qualified VLAD R3 review `01a1271f-876c-79e1-9637-ee437e1b4ee2` closes the residual
P2 at source level, no P1/P2/P3 in the exact delta. MUSE independently reconciled
R3 and cleared SOURCE readiness only; clarification
`01a12728-8d6d-7292-8e1d-66b090f5cb1a` retains native Darwin admission HOLD.
FFI/wait, CLI compatibility and cleanup execution still
NOT_PROVED; no live grant or candidate/control execution. Environment/time/cost/cleanup
remain proposals, NOT authority; stale R7 wrappers must not run.
Frozen no-VM admission planning proposal:
`/Users/bobai/.codex/artifacts/2026-10-10/kernel-custody-admission-proposal-r13-r2/PLAN.md`,
SHA `edfa8e44eee0e0b5d8be16c4fafa96ce57162c1fbcca48afc888e4643decfc5d`.
Qualified VLAD planning review `01a1272e-53f4-76e3-af06-187974910ebd`
reports no P1/P2/P3; MUSE planning review independently accepted the proposal and
reconciled qualified VLAD, planning ONLY. Exact dummy harness/
launcher and trusted fact-query controls are not frozen, so NOT execution-ready.
First identify/review an existing qualifying native isolation route by bounded
source-only inspection; retain HOLD if none qualifies within unchanged constraints.
Only then freeze the exact admission harness, launcher and fact-query controls.
Next is source preparation for a separately authorized
no-VM admission gate, NOT a conditional VM request. Cached Linux sandbox may test
Linux control only; it cannot clear Mac FFI/control. Trusted read-only account/
image/package/pricing facts require separate human authorization through the
existing trusted operator path, NOT R3 supervision or target credentials.
No qualifying existing Darwin isolation route established; no claim none exists.
Bounded native source inspection found installed sandbox-exec, whose local man
page (`/usr/share/man/man1/sandbox-exec.1`) describes profile execution and marks
it deprecated, not an accepted profile/guard enforcing the required aggregate
resource, isolation and cleanup controls. Apple SDK sys/resource.h rlimit
constants do not establish those controls. Existing inspected experiment guards
are Linux cgroup2/bwrap paths, not native Mac admission. No runtime/capability
probe or sandbox invocation performed. Disposition: SOURCE-PREPARED /
NATIVE-ADMISSION-HOLD; MUSE independently reconciled the bounded source evidence
and handoff, no new P1/P2/P3. This source-only preparation is CLOSED with the
native admission gate retained; it does not pause other authorized workstreams.
Missing reviewed guard and independently established equivalent isolation,
aggregate resource and cleanup controls are the stop reason, not deprecation
alone or a claim of universal Darwin/kernel infeasibility. Do not prepare
speculative Linux harnesses or collect expiring facts as a native workaround.
If matching native controls cannot be admitted within the fixed window, HOLD;
no unsandboxed exception, new platform/install, VM discovery or deadline reset.
Step7 OPEN, original R1 BLOCKED, Step8/V12.2
inactive. Old closeout/evidence immutable. Private experiment only; no public
spec/schema/JSON/CLI/changelog/Site/Skills/Examples/Learn/FAQ/Glossary change.

### Bounded Kernel Outcome Acquisition Experiment

**Disposition: CLOSED / INCONCLUSIVE; kernel acquisition NOT_PROVED.**
MUSE accepted the narrow R12 compile result and reconciled qualified non-author
VLAD in `01a12664-b41c-7bf1-ae6c-ff46c87009d5`. No P1/P2/P3 in the scoped
retained compile/cleanup evidence. Further experimental execution under the
current grant is STOPPED. This is not evidence of universal infeasibility and
does not close Step7. Original R1 BLOCKED, Step7 OPEN, Step8/V12.2 inactive.
MUSE final closeout review `01a1266b-26ec-7172-b81a-5262e91ef2d9` accepted
the coherent handoff and existing consumer RESULT.md, no P1/P2/P3. Exact reviewed
RESULT5504B SHA `decbcb0b73be036e1ce471f5e803d4a6488deefb9d40342f3f95b179e4d8b9e4`
frozen0444, regular/single-link/euid and unchanged digest independently verified.
This receipt ends the experimental grant; no further execution or proposal.

Bobai approved the private feasibility spike on 2026-10-10 with the original
two-engineering-day stop; no automatic reset/extension. All four GCP and all
subsequent single-attempt local approvals are consumed. No retry, fifth VM or
new kernel-capable attempt is authorized. Routine bounded local decisions were
delegated to MUSE, not spending/VM/provider/production/release/stop renewal.
The Discord host `100.77.146.124` is excluded from builds/mutations/heavy tests/
reboots. No new experimental execution is active; the separate source-only
preparation above does not revive this closed grant.

**Accepted bounded facts and canonical private records** (under
`/Users/bobai/.codex/artifacts/2026-10-10`; RESULT.md and raw evidence retained):
- `kernel-custody-preflight-r11`: Python3.11.2,11 methods/12 synthetic shapes,
  including current-frame parent-detach refusal. VLAD
  `01a125fc-c0a8-76d2-b594-6c69863c6a39` and MUSE
  `01a125fc-c724-7393-8594-71791b16302e` accepted ONLY synthetic/parser evidence.
- `kernel-custody-c-helper-r11`: seven constructed child-status/CRC groups on
  native ARM64, not realwaitid/SIGKILL acquisition. Qualified runtime VLAD
  `01a1261f-559d-7751-94b6-ebc483114d30` and MUSE
  `01a1261f-5979-7050-ab30-a9d83b9d3731` accepted that narrow scope.
- `kernel-custody-fixture-compile-r11`: full R11 fixture ARM64 compile-only,
  NEVER run/exported. Qualified scoped VLAD
  `01a12636-8503-7723-bab8-4f2e5495f749` and MUSE
  `01a12632-f25a-7cd1-9921-5c7c72282b40` accepted compile/output-shape only.
  An unrelated article final was rejected as clearance; R1 marker-byte HOLD
  remained unexecuted and only corrected R2 ran.
- `kernel-custody-interop-r12`: actual shared2903B C serializer and pinned
  Python verifier passed19 synthetic groups:3 constructed positive shapes,
  2 valid-CRC semantic refusals,71252B max-width wire,4 producer guards and
  9 wire mutations. Qualified VLAD `01a12659-87ca-7a92-a3ce-657c694b13aa`
  and MUSE `01a12659-8d45-7a93-94b9-cbeafa40b6dd` accepted ONLY selected
  synthetic transport/refusals. NO real fixture lifecycle result is proved.
- `kernel-custody-fixture-consumer-compile-r12`: R12 successor fixture includes
  the shared header, with unchanged ABI/admission inputs. One exact approved
  ARM64 compile-only attempt passed. Qualified VLAD
  `01a12664-a972-7692-b752-9ca55f93e8d6` and MUSE above accepted compile/
  bounded output-shape/runner-observed identity ONLY. No /helper executable
  exception; output0400 on private noexec /tmp, NEVER run/exported.

**Latest exact evidence:** consumer attempt `15b75f6845234288ac044c073a1478c7`;
container `f328decba16a417f4d8c0c82943c3aa0843777fd1841b8ef68cb93bedabe652e`.
Root/attach/container0, terminal exited/nonrunning/nondead/noOOM/noError; all
cleanup predicates true. Independently confirmed exactID/UUIDlabel/anchoredname
absence, stageENOENT and launcher/attachment-by-ID absence by SCOOBY/VLAD/MUSE.
Regular single-link0444 attempt434B/result2766B/log5624B hashes respectively:
`d199268a8db9c013c6b1ba9e6a9de4d510f5232a7da8ee097e40e0f4ae7a0c1f`,
`09d441ddbf6d7648e240e46ff02e2fad88292e28232bc70743b7eac3c17e5cd5`,
`eb0cd19ed2e4101cdc5301daa88382ae74530cb34365c6cfb8ef57de79241f1d`.
Frozen eight-member preparation
`c2c4d5ecdc58eff0746fb5c0745bacd9411ef1e700effdf96687ea4dea0a1a89`;
four-input manifest
`d7b5d52f8534c5412b416679cf2f7f63801b5e6722b27b2b5e96cbe5b9e1cea3`.
Observed fixture707008B SHA
`8674dfa8c0f7aa5e7a1e410d5417e90e598d3bd9baf9948724083253d35dcfb2`
coincides prior R11 metadata, NOT independent binary rehash/reproducible-build
proof. Deleted binaries/wires have only runner-observed retained identities.

**Immutable failed/partial evidence:** `kernel-custody-cloud-r7/RESULT.md` and
its raw retention remain STOP/partial. X64 fixture/kernel/initramfs built;
normal/exit emitted PASS markers, not accepted acquired-state cases (exit ROW
console-corrupted). Reuse child failed EACCES13, then parent incorrectly emitted
PASS because status was unchecked. Outer rejection stopped CLI1/overall1;
remaining9 cases never ran, no full results manifest/GO. R10/R11 source repaired
directory traversal/status/framing and current-frame lineage, but no later
synthetic/compile result validates those repairs in a live kernel.
`kernel-custody-preflight-r10/RESULT.md` retains10/11 passed and the genuine
parent-detach failure; R10 HOLD. R8/R9/R10 frozen packets, prior build failures
and historical evidence remain unchanged. The earlier escaping P1 was withdrawn,
not a repair claim; the actual compile R1 marker-byte HOLD is separate.

**Resources:** previous four GCP attempts' owned VM/disks/address/networking
were deleted; independent six-inventory absence checks retained, private
operator key removed. Both R12 local capsules/staging/attachments independently
absent; no experiment resource remains running at these checkpoints. No new
cloud resources created. Settled billing is unverified, not zero-cost.

**Discriminating next live gate, NOT activated:** an independently reviewed,
fresh frozen kernel-capable acquisition attempt must run all12 cases fresh
(normal,exit,reuse,last,dup-refusal,paused,kill-close,late,survivor,
collector-before,collector-after,overflow) in networkless isolated guests,
retain bounded intact actual-state frames and outcomes, and independently
reconcile exact generations/visible lineage/actors/chronology/refusal states.
SPIKE_FAIL, missing/interleaved/truncated/bad frames, wrong child outcomes,
late/ambiguous/incomplete evidence or cleanup failure must refuse even with
PASS. No repair-by-filtering console text or reuse of R7 PASS markers.

Reopening requires a HUMAN decision on one exact kernel-capable environment/
attempt scope, fresh source/build/packaging/collector identities and qualified
VLAD/MUSE source clearance, available time within the original stop (or explicit
human renewal if outside it), and any VM/spend cap/provider deletion deadline.
Cached/local environment availability is not runtime authority. No current
approval supplies live execution; the separate source-only candidate above is
not a VM or budget renewal. Reassess after live evidence, not more synthetic coverage to
manufacture movement. Even a restricted12-case GO would not prove production
manager/SCM/eight-input closure or Step7; integration requires its own authority.

**Uncovered-material inventory:** synthetic/parser/transport and named native
compile/helper scopes above are proved ONLY within their declared experiment.
Explicitly not_proved: ALL actual fixture lifecycle executions, realwaitid/
SIGKILL admission, acquired-state authenticity/semantic attestation, full x64/
packing/guest/kernel matrix, manager/SCM/eight-input/mutator/driver closure,
unexported collector-before comparisons, original timing parity and installed
custody/Step7. +8s selected-work ceiling is serialized, not enforced by this
close-only experiment; proposed late-close control concerns +10s retirement.
CRC is corruption detection, not origin proof. ParentSIGKILL cleanup, raw wire/
binary custody and billing settlement are unproved. Production/public runtime,
pins/CI/provider/credential operations are repo-owned outside this checkpoint,
unchanged/inactive; no discovered gap is silently promoted to shipped capability.
No public spec/schema/JSON/CLI/changelog or Site/Skills/Examples/Learn/FAQ/
Glossary propagation: private inconclusive feasibility closeout. No commit/push.

### Accepted Step 7 Original Table Transition R1 Record

MUSE ACCEPTS R1 ONLY as a BLOCKED source-transition record after qualified
non-author VLAD `01a12088-7784-7310-8ce9-e39426fc4354`, no P1/P2/P3 content or
integrity findings. Frozen manifest
`6fda79462a50ac35c289811958f101ea2594cdca0128ff8e03a9f46de20329b3` (44 rows),
design `bb63293462cf5241a871a9f31c722bdec4f59a1f27888b691be0ce672a0edd37`;
pending words below historical. Exclusive-native-async-closer premise REJECTED;
STOP stock-manager custody route: no further topology/owner/guardian study,
one-closer result hook or Ota-only report workaround. No demonstrated race or
universal impossibility. Bobai approved ONE offline/source-only manager-native
custody/result capability experiment on 2026-10-09, not dependency adoption.
MUSE rejected the proposed raw-anchor/fixture validator before implementation:
it would repeat the known missing acquisitions, not supply a concrete producer.
At R1 acceptance the alternative kernel-dependent primitive was UNASSESSED and
no implementation was cleared. The bounded 2026-10-10 experimental grant and
preparation above supersede that historical grant status only; no production
kernel/manager capability or acquired result is accepted.
Original table/copy epoch, permitted actors through required completion and an
independently F-bindable retained outcome for the finite OWNED graph remain
required. R1 itself supplies no patch/fork/kernel/runtime/root/provider/CI/spend/
commit clearance; only the explicitly bounded experimental grant above applies.
Status only; proposal/predecessors and all limits unchanged. Return guards then STOP.
Step7 OPEN; Step8/V12.2 inactive.

ONE MUSE-cleared original-table/closer epoch source check completed; BLOCKED,
pending fresh qualified non-author review/MUSE disposition. RAW v257 + Linux
v6.12 trace distinguishes returning failure (before fatal unshare) from successful
exec. RUBY `01a1207d-0739-7ce2-aa51-ea9c8c6ffae1` completed the independent
failed-exec/direct-sweep edge; integrated after exact raw-anchor verification.
No duplicate study pending. Direct failed PID1 cleanup can freeze/sweep the
original shared table without CLOSE_RANGE_UNSHARE or original-slot exclusion;
the result is discarded. Native async closer is NOT the exclusive actor. A
successful exec can split a shared table: the worker retains OLD table while
CLOEXEC closes NEW copies. Count-one/no-sharer exec can instead CLOEXEC-close
the ORIGINAL abandoned slot; last-old-table cleanup is another possible closer.
Neither source branch nor death/sweep bookkeeping acquires an exact local result.
No same-slot rebind/wrong-object race or installed/timing proof demonstrated.

Exact missing dependency: independently acquired original table/copy epoch,
permitted close actor custody through lookup/detachment and required completion,
retained outcome independently bound by F; all finite OWNED copy/transfer/operation
obligations remain. New table references are charged; whole-OFD destruction is
NOT the predicate. Full preexec mutator closure and result/F acquisition absent.
Recommendation: keep route unavailable and return this dependency; MUSE decides
the concrete capability/approval needed. No automatic further topology/guardian/
kernel study, feasible producer, manager patch/fork or runtime selected. Complete
current freeze/review then STOP; no implementation while custody remains unknown.
Two docs/evidence and 15 guards/HEADs only; unrelated planning/predecessors stay
unchanged. No source/test/build/config/pin/Protocol/public/live/CI/spend/commit.
All original8s/10s/Observer5s1s, costs, four death stages/seven faults, firstfailure/
UNKNOWN I/W preserved. Spec/schema/JSON/changelog, Site/Skills/Examples/Learn/FAQ/
Glossary unaffected: private source record, no public behavior/claim. Adapter
inert; Observer uninstalled/unconnected. Step7 OPEN; Step8/V12.2 inactive.

### Accepted Step 7 Native Manager Copy Custody R1 Record

MUSE accepts native-custody R1 ONLY as a BLOCKED source-only record after
qualified VLAD `01a12075-3eec-73c1-b3e5-8568a902ea37`, no P1/P2/P3 content or
integrity findings. `01a12070-709e-7312-bca9-e286d2a7c499` unrelated final is NOT
review evidence; `01a12074-2781-7f93-b2fc-79f4ca658f15` content-only/incomplete
is NOT qualified closure. Frozen manifest
`aeebefbcda19951f33b2a1d5d7761f8cfdf6c3c53f9fc19558d9e8ed5b63235b` (49 rows),
design `1604d00b005698147166253e963ff8a5d7deb9c346a7de761cac69cc4156fd3c`;
pending words historical. No custody/result/F/installed/timing proof. MUSE clears
ONE targeted original-table/closer epoch source check, no implementation. Same
two docs/evidence and limits; Step7 OPEN; Step8/V12.2 inactive.

ONE MUSE-cleared bounded v257 source-native ownership/exclusion check completed;
result BLOCKED, pending fresh non-author review/MUSE disposition. Native adoption,
borrow/shallow cleanup, distinct serialization duplicates and one async handoff
are distinguished, not token-based proof. Reload/reexec release the original
through old-unit destruction, not continuity of the restored duplicate. Cancellation
is not close completion; errors can retain earlier adopted copies. Exact uncovered
class: pending CLONE_FILES closer/original-table epoch and competing close/rebind
actors through manager teardown/reexec. No actual wrong-object close demonstrated;
actor closure, bound result/F acquisition and installed/timing proof remain absent.
RUBY technical completion `01a12065-6fad-7710-a7d8-6f7084a1b2eb` integrated after
checking raw anchors; unrelated prior outreach final is not technical evidence.
No feasible manager change/runtime/private-table/guardian/lock selected. Return
frozen result; MUSE decides one targeted original-table/closer transition check
or holds route blocked. No implementation while custody unknown. Same two docs/
evidence, 15 guards and HEADs; unrelated planning/predecessors preserved. No source/
test/build/config/pin/Protocol/public/live/CI/spending/commit; all original clocks,
costs/faults/firstfailure/unknownI/W unchanged. Public propagation unaffected:
private source-feasibility record only. Step7 OPEN; Step8/V12.2 inactive.

### Accepted Step 7 Exact Manager-Owned Local Close R2 Record

MUSE accepts local-close R2 ONLY as a BLOCKED dependency record after qualified
non-author VLAD `01a12059-a73c-7c80-af64-8e1fda2b37d1`, no NEW P1/P2/P3.
R1P1 unsupported feasibility is withdrawn, NOT a solved custody mechanism;
concrete exclusion/producer/result/F acquisition and installed/timing proof remain
missing. R1P2 raw anchor corrected; R1 NOT ACCEPTED preserved. Frozen R2 manifest
`99c4049174856af1ced58b9750a44eb72d7c972465bdb733732ee30add6eed8c` (31 rows),
design `d5b04fa28ed742cb4f0290db9a887966d0710336ae9dafba81c8d8a0296191bd`;
pending words historical, not a candidate or upstream commitment. Status-only;
proposal/limits/predecessors unchanged. Return guard then stop; no successor
clearance. Step7 OPEN; Step8/V12.2 inactive.

MUSE clears ONE R2 documentation/evidence repair: one already manager-owned
donation copy for one admitted S invocation, from native release/async closer
through actual LOCAL close completion to independently admitted F reconciliation.
Local-close R1 NOT ACCEPTED: qualified VLAD
`01a1204f-35f9-7a01-8fb0-8b94c63b6ddf`, confirmed
`01a12051-a7e8-7893-b2c2-0e85ced699a1`, found P1 (exclusion stopped before
FD lookup/detachment), P2 (shifted raw adoption anchor), noP3. R1 stays immutable.
No implementation clearance. Earlier accepted dependency R2 remains BLOCKED
record-only; whole-OFD destruction is not required. Local R2 decision BLOCKED:
concrete shared-table custody/exclusion through lookup/detachment, conservatively
through return, is NOT acquired by a token or future admission. No feasible
upstream result is claimed. Producer/result/F acquisition stays absent. Correct
v257 raw adoption750-791/dup784; exact raw source retained. No patch/runtime/hook.
RUBY mapping `01a12047-20b0-74d3-ad2c-58f306d45c06` confirms consumer owners,
not producer/F admission. Fresh frozen non-author review then MUSE disposition
required. Remaining copies/transfers/continuations/resources remain not_proved;
one local outcome cannot close Step7. Preserve four death stages/seven faults,
first failure/unknown I/W, original8s/10s/Observer5s/1s and all charged costs.
Decision pending review: one separately bounded internal concrete-custody
feasibility slice or hold this route blocked; no sponsorship/external commitment
inferred. No Ota-local ACK workaround/private-fork/kernel-escalation default.
Two docs/evidence only: no source/test/build/config/pin/Protocol/public/live/CI/
spending/commit/push/merge/release. Source guards/HEADs/predecessors/unrelated
planning edits retained. Public surfaces unaffected: private design only.
Step7 OPEN; Step8/V12.2 inactive. No successor work before MUSE disposition.

### Accepted Step 7 Manager Completion Dependency R2 Record

Active R2: ONE documentation-only ownership-scope repair. R1 is NOT ACCEPTED:
qualified VLAD `01a1201a-b142-7901-bae8-3a6fd39f5d2f` found one P2, no P1/P3,
for substituting whole-OFD destruction for the inherited finite owned-retirement
bar. Preserve physical completion; distinguish outside same-OFD aliases from
owned copies/transfers/kernel operations and escaped custody. No duplicate RUBY
study, kernel research or successor. Focused R2 review/MUSE disposition completed
as BLOCKED-only acceptance below; no successor clearance.
Historical R1 manifest `bb6fe5b3e53916771a58e1cb43294041be977ff2b4f095e5f5d727d0082c5bdf`,
design `d042776f55c98d7c92b3b77dfecfb4d06a5bff4733ca67e3556f24933d80c788`;
pending words are historical, not acceptance. Actual R2 BEFORE private
`e2d1bd40e5c24b083993f13d65f99cbe259bb8f6b3dac698cf50ab14b7b0dc4f`, Core
`3c529dfd1158048116d731a04d8177855648de05196f746a0a95f241a1f8f339`.

MUSE accepted the predecessor status guard and cleared ONE missing-dependency
CONTRACT/FEASIBILITY design slice under standing bounded-roadmap delegation.
Evaluate one exact manager-level finite donation-retirement completion mechanism,
prefer an existing supported versioned capability; if absent, specify the minimum
proposed upstream capability and its actual kernel completion dependency. No
extra owner/journal, kernel instrumentation or custom manager fork is selected.
Identity/receipt fields do not establish physical completion.

SCOOBY owns this handoff and Launcher `docs/private-principal-adapter.md` only.
RUBY may do ONE non-overlapping primary/source producer-completion check. Checklist:
record scope; bind finite birth/copy/transfer/close lifecycle to admitted producers
and kernel completion; distinguish shipped APIs from proposal; retain independent
F binding, four death stages/seven faults, original clocks and all incremental
costs; recommend one decision with deployment/maintenance/portability limits;
freeze actual BEFORE/diffs/AFTER and 15 source guards; fresh qualified non-author
VLAD review then MUSE disposition. End EXISTING-SUPPORTED-PRIMITIVE,
UPSTREAM-MANAGER-CAPABILITY-REQUIRED or BLOCKED; stop at unsupported acquisition or
unapproved trust/requirement/resource/daemon/fork/instrumentation commitment.

Read-only primary/source research and two docs/evidence only: no source/enums/
serialization/fixtures/tests/builds/drivers/configuration/pins/Protocol/public
changes, runtime/artifact selection, patch/instrumentation, unit/VM/container/
root/installed/hosted/provider/CI action, spending, external/upstream messages,
commit/push/merge/release or transport. Predecessor packets and unrelated planning
edits stay untouched. Actual BEFORE private
`9342207d8b1e506c75b8541cbd028af3ee66e87bf4e848be0f3637bccb966348`, Core
`dd89f0c60220acd3d1af8ea49c8d1e9e6a8e37706b78ac4f68dfbf19b939724e`.
Step 7 OPEN, completion BLOCKED; Step 8/V12.2 inactive.

R2 decision proposal: **BLOCKED**, accepted by MUSE ONLY as a finite custody/
owned-operation-acquisition record after qualified non-author VLAD
`01a12030-4866-7ac2-9e4e-d3104151af07`, with no P1/P2/P3 for this exact R2 and
explicit closure of R1's P2. R1 remains NOT ACCEPTED historical. Frozen R2 packet
`/Users/bobai/.codex/artifacts/2026-10-09/principal-manager-completion-dependency-design-r2`,
manifest `1cb645400859fbf8a2c6491b4261a8ef8473e9a547109aec925a0a7f725110e4`
(39 rows), design `1d8c5772070b8024d9b0b5d217c39ea111e4bdbdf2ad7eefb0c2429a086420d9`:
pending words remain historical. Exact continuity/diffs/AFTERs, retained anchors,
15 unchanged guards and HEADs verified; qualified review closed the scope finding,
not the acquisition gap. No working mechanism/candidate acceptance. Actual finite
custody/physical owned-operation completion, installed/account/MAC admission,
resource retirement and original timing/accounting fit remain unproved. No
source/test/build/config/pin/public/Protocol/CI/spending/commit, runtime/kernel/
trust/requirement/resource change or successor clearance. Only acceptance/
provenance changed; predecessors/unrelated edits preserved. Return the status
guard to MUSE then stop. Step 7 OPEN; Step 8/V12.2 inactive.
ONE proposed upstream manager finite donation-retirement transaction is evaluated,
not a shipped API/selected patch. Native generation-scoped birth/copy/transfer/
reexec/close custody, independently authenticated F binding and partial-failure
ceilings are specified as required acquisition, not supplied by receipt fields.
The private repair cites the exact inherited owned-handle/channel/resource bar
and classifies originals, recipients, manager/observer copies, transfers/queues/
in-flight/reexec/async close and relevant owned kernel operations by custody and
actual required completion. These acquisitions remain not_proved. Independently
outside aliases may hold the same OFD; they grant no foreign closure authority
and do not by themselves prevent completed owned close/drop operations. Required
exclusive admission, if unavailable, yields refusal/not_proved; escaped owned
copies cannot be relabeled outside. Whole-OFD final cleanup is not a universal
completion gate. A local bound acknowledgement may establish one actual operation,
not the full finite graph; payload/intent/absence/EOF/UnitRemoved still cannot
substitute for required physical completion and independently bound evidence.

The proposal keeps four C-death stages/seven faults, all incremental tracking/
worker/kernel/F costs, original clocks, first failure and unknown I/W. It separates
shipped API/source limits from proposed upstream responsibility and names exact
kernel completion points without selecting instrumentation/patches. No claim
about other versions/installed support/impossibility. Recommend no implementation
around missing scoped acquisition; after accepted R2 MUSE decides the smallest
supported dependency. No kernel work or requirement reconsideration follows from
R1's unjustified whole-OFD condition, and no successor is authorized here.
Closure/account/MAC, resource/profile and actual fit proof remain open. Private
planning only: public Site/Skills/Examples/Learn/FAQ/Glossary/spec/schema/JSON/
CHANGELOG/Protocol unaffected. No external/upstream messages or downstream work.
RUBY's R1 producer-edge check `01a1200f-3b91-7162-b110-37a2a48aea27` retains its
accurate kernel facts, not acceptance of the overbroad predicate. No repeat study.

### Accepted Administrative Recovery Extension Design

MUSE verified the predecessor acceptance-status guard and cleared ONE successor
DESIGN/FEASIBILITY-only proposal under standing bounded-roadmap delegation.
Use the existing protected Launcher installation/recovery boundary as the reuse
anchor, not an already sufficient or selected final owner. Establish finite
independent admission and pre-creation custody, exact intent/manager epoch/job/
invocation/reference/handle identities, distinct enforcement/observation/durable
evidence after C death, and physical donated-handle close-completion acquisition.
UnitRemoved, NoSuchUnit, cleared bookkeeping, status and EOF alone are insufficient.
Retain four C-death stages, seven fault rows, first failure and unknown I/W facts.
All new control/watch/recording actors, handles and costs count against unchanged
aggregate limits and original start+8s/+10s clocks, even outside the attempt's
lifetime/failure domain. No off-clock recovery or resource exemption is cleared.

SCOOBY owns only this handoff and Launcher `docs/private-principal-adapter.md`.
RUBY may make ONE non-overlapping read-only primary/source primitive check.
Checklist: record scope; examine finite acquisition/physical-close mechanism;
write one concrete bounded proposal or exact blocker; freeze actual BEFORE/diffs/
AFTER and 15 unchanged source guards; obtain fresh qualified non-author VLAD
findings-first review, then MUSE disposition. DESIGN-CANDIDATE requires a defensible
mechanism for every edge, with installed behavior/timing/resource fit unproved.
Otherwise BLOCKED with the missing primitive and smallest decision; stop rather
than add topology. A new daemon, manager patch, requirement/trust expansion or
resource exemption requires separate disposition.

No code/enums/serialization/fixtures/tests/builds/drivers/configuration/pins/
Protocol/public edits, runtime/artifact selection, unit/VM/container/root/
installed/hosted/provider/CI operations, spending, commit/push/merge/release or
downstream transport. Preserve MUSE's unrelated dirty planning files and all older
immutable packets. Fresh parent evidence only. Step 7 OPEN and completion BLOCKED;
Step 8/V12.2 inactive. Actual accepted-status BEFORE hashes: private
`fe3b054b1c18b91aa6ecfc1edf38710912deb9cd8853d02eb136d8472ed2fdf5`, Core
`aa93ad915a5fc71563d5168c702862de1a7bcd0697c1c498389150172426fb84`.

Proposal result: **BLOCKED**, accepted by MUSE only as a coherent dependency-design
record after qualified non-author VLAD review
`01a12001-5618-7581-969e-825b006d7877`, with no P1/P2/P3 for this exact objective/packet.
One concrete installation-owned F responsibility extension reuses protected
admission/intent/recovery, not a selected new actor: independent pre-creation
arming, manager epoch/job/invocation custody, original-clock enforcement,
independent final observations and protected evidence. C/R death does not require
a watcher chain; F/manager loss remains failure/UNKNOWN and no replay. Every new
F cost/handle remains accounted even outside the retired aggregate. All four
C-death stages and seven faults retain first cause/missing I/W outcomes.

Exact missing dependency: independently acquired finite manager-donation/copy
identity and physical-close completion, including reexec/transfer/deferred-close
and observer comparison references. kcmp compares supplied slots, pidfd_getfd
duplicates an OFD, and proc fd entries are current snapshots, not that receipt.
UnitRemoved/EOF/bookkeeping do not close the edge. The private proposal gives
mechanisms/limits for each duty and stops at this dependency, not an impossibility
claim or proof of installed support. No implementation around an unobservable
edge. Recommend a separately bounded plan-owner decision on that manager/kernel
acquisition dependency; patch/instrumentation/trust expansion or operator-recovery
requirement relaxation is not selected/cleared. Artifact/closure/account/MAC,
resource/profile enforcement and real 8s/10s/aggregate fit remain open. No public
surface changes; Site/Skills/Examples/Learn/FAQ/Glossary/schema/JSON/Protocol and
CHANGELOG unaffected by this private design-only proposal.
RUBY's completed same primitive check `01a11ffb-3c59-7760-bd59-e2e9d1000e84`
agrees on NOT_PROVED: Linux v6.12 slot removal precedes close processing;
alias removal/closer completion/final OFD retirement stay distinct. Ordinary close
uses `__fput_sync`, not universally deferred cleanup. Its earlier unrelated
outreach final `01a11ff6-5422-7180-ba43-cf1b29440157` was rejected as evidence.

Frozen packet
`/Users/bobai/.codex/artifacts/2026-10-09/principal-administrative-recovery-extension-design-r1`,
manifest `39db89fac885007b4c57859b070c413217849b7e4d209e4a90854f7b2c3ef886`
(45 rows), design `73de41bc91e149cc110854113b0de4f7c42589be5e0f77393c69c383cf40a4a4`:
pending words remain historical. MUSE verified actual BEFOREs/live AFTERs, exact
diff bodies, 15 equal live source guards and retained primary evidence; VLAD
independently reviewed the exact objective/packet and proof boundaries. Acceptance
does not select an implementation candidate/owner/artifact/runtime, assert
impossibility/source vulnerability/installed proof, or clear patch/instrumentation/
trust/resource exemption, requirement relaxation, implementation/live/test/build/
configuration/pins/public/Protocol/CI/spending/commit/transport. Closure/account/
MAC, profile/resource enforcement and original 8s/10s fit remain open. Only
acceptance/provenance changed; predecessors and unrelated planning edits preserved.
Step 7 OPEN; Step 8/V12.2 inactive. Stop for the separately bounded missing-
dependency decision after MUSE verifies the returned acceptance-status guard.

### Accepted Final-Retirement Owner Assessment

MUSE cleared ONE successor DESIGN-ONLY/read-only assessment under Bobai's standing
bounded-roadmap delegation: determine whether an already independently admitted
administrative manager/recovery owner outside the attempt can enforce and
independently observe R's exact terminal/service-reference retirement and final
aggregate/resource/profile closure after C dies. Assess whether that owner
actually exists; PID1/root/admin names or a proposed report are not admission.
No per-attempt watcher chain or completion-requirement relaxation is authorized.

SCOOBY owns the assessment and only this handoff plus Launcher
`docs/private-principal-adapter.md`. RUBY may do ONE non-overlapping read-only
current-source/primary versioned API primitive check. No code/fixtures/enums/
serialization/constructors/tests/builds/drivers, configuration/pins/public edits,
unit creation/VM/container/root/installed/hosted/provider/CI operations, spending,
commit/push/merge/release or runtime/artifact selection. Preserve MUSE's unrelated
dirty planning files. Parent-created evidence packets only are allowed.

Acceptance: unchanged completion/fault assumptions, distinct enforcement/
observation/durable-evidence owners and finite acquired trust/handle graph;
exact invocation/job/reference/donated-FD/cgroup lifetimes and original clocks;
C death before creation/during arming/after release/during retirement; compact
failure table for C, R, both, manager/owner loss/reexec, lost replies, foreign
resources and expiry; unchanged aggregate resource/8s/10s limits and no invented
fit. Missing I/W facts stay unknown and first cause stays failure. End with
FEASIBLE-CANDIDATE, BLOCKED or NO FEASIBLE EXISTING OWNER, one recommendation and
smallest decision. Operator recovery/refusal is a separate plan-owner requirement
option, not equivalent proof or approved weakening. Stop if no owner meets the bar.

Freeze actual accepted-status BEFOREs, narrow diffs/full AFTER, necessary exact
source/upstream anchors, 11 source guards and immutable manifest. R1/R2 packets
remain untouched. Fresh qualified non-author VLAD review precedes MUSE disposition
or dependent work. This clearance permits assessment, not owner selection or
implementation; Step 7 OPEN and Step 8/V12.2 inactive.

Assessment accepted by MUSE as a bounded NEGATIVE current-architecture/acquisition-
gap assessment only after qualified non-author VLAD review
`01a11fe0-3c40-7662-8da4-b1bf75fc54c6`, with no P1/P2/P3 findings:
**NO FEASIBLE EXISTING OWNER** demonstrated for the unchanged complete C-death
retirement bar in the examined current architecture. Protected Launcher recovery
handles durable recorded child scopes, not private R service/jobs/references/
aggregate/profile/handle retirement or original-clock crash triggering. Root
custody capture copies a COMPLETE-triggered bundle, not those live facts. PID1
and RefUnit/GC APIs do not independently close the ownership/evidence edge.
RUBY's one read-only primitive check identifies a further concrete v257 limit:
manager donated-FD cleanup uses asynchronous close; UnitRemoved/NoSuchUnit or
cleared bookkeeping is not physical close-completion evidence. This is a wider
private completion gap, not a defect asserted against existing scoped behavior
or an installed-version claim. No live installation was examined.

The private assessment gives actual versus missing trust graphs, C-death stages,
finite failure inventory and unchanged-resource/8s/10s boundaries. Missing I/W
statuses remain unknown; no watcher chain, self-attestation, renewed clock or
foreign cleanup. Recommended smallest next plan-owner decision: whether to
authorize a concrete installation-owned administrative recovery responsibility
extension, reusing the existing Launcher recovery boundary only if independently
admitted, with exact manager-close/terminal/aggregate evidence and original-clock
fault policy. No such topology or source work is selected/cleared. Operator
recovery with failure/UNKNOWN is an explicit alternative requirements decision,
not equivalent proof or an approved weakening. Stop downstream transport until
this ownership gap has disposition. Same two private/canonical docs only; no
public surfaces affected. Frozen packet
`/Users/bobai/.codex/artifacts/2026-10-09/principal-final-retirement-owner-assessment-r1`,
manifest `aad3e1ef0ec667cad14990c76c30637d7271a23291f2b99dfd7149eed1dc01db`
(30 rows), assessment `3f97049788cbe187bf1a8ef3417f717777f1bac132d2728845d60adb5161372b`:
pending words remain historical. Exact BEFORE/diffs/AFTER, 15 equal source guards,
eight local anchors (839 lines), five upstream anchors (321 lines) and five raw
official v257 files reconciled. Earlier unrelated VLAD final
`01a11fd6-f075-7800-9955-62384928ae26` was rejected, not disposition/article authority.
Acceptance is not impossibility or an existing-scope defect. Completion remains
BLOCKED; no owner/topology/artifact selection, changed failure requirement,
physical-close/timing/resource proof, implementation/source-only/live/commit
clearance. Stop for the separately bounded recovery-extension DESIGN disposition;
no dependent work begins here. Predecessors/unrelated planning edits preserved.

### Accepted Blocked Manager-Owner Feasibility Proposal

Bobai asked to keep going. MUSE cleared ONE concrete DESIGN/FEASIBILITY proposal,
not source implementation, for Launcher's native manager-composition owner.
Only Launcher `docs/private-principal-adapter.md` and this handoff may change.
No source/fixtures/workflow/configuration/pins/Protocol, build/test/driver,
container/VM/root/manager/provider/live action, commit/push/merge/release or
positive constructor is authorized. MUSE-owned dirty planning files stay excluded.

SCOOBY proposes direct `StartTransientUnit` `ExtraFileDescriptors` donation to a
new per-attempt native S service, preserving all eight checked input objects;
S retains the separate Observer and hands only interpreter/modules/archive to
its independently selected W child. Production listener/boot activation and W/I
FD 3/4/5..10 remain unchanged. The proposal specifies two finite private channels,
original clocks, independent S/job/invocation completion and a new finite native
R guardian outside the killed work subtree but charged to the aggregate attempt.
R is proposed owned work-subtree cleanup after C death, not an existing actor
or proof of complete C-crash retirement. R cannot independently observe its own
terminal service/reference retirement or final aggregate emptiness; those remain
unknown/operator recovery after C death even if R survives.

RUBY's read-only official API/version check establishes upstream v257 support
for direct donation, not v255 support or installed capability. Donated stdin is
supported in both, but subsequent SCM_RIGHTS bypasses PID1 for the eight inputs
and needs an explicit requirements decision. Names/PIDs/FD numbers and worker
result packets are not authority. Manager-held duplicate lifetimes and shared
socket shutdown/actual EOF require their own later proof.

The full path is **feasibility BLOCKED**, not implementation/activation ready:
the v257-capable manager/client/bus support and additional R owner need disposition;
artifacts/full acyclic closure/private profile/numeric accounts/effective MAC,
same-object acquisition and manager-reference/readback lifecycle are unselected.
Real archive/tree/Listener work, journal/manager/IO/cleanup costs and aggregate
R/S resource use are not measured against unchanged start+8s/+10s and Observer
5s/1s bounds. A timer is not hard-real-time retirement proof. C death, manager
loss or simultaneous C/R death needs independent surviving enforcement or explicit
plan-owner failure-requirement disposition, not a success claim.

R1 was rejected: fresh VLAD review `01a11e95-d383-7190-baa9-e2a45323391b`
and MUSE found two P1 contradictions. R2 distinguishes actual pre-release W READY,
release-input frame/EOF, post-release launch-status EOF and S-result shutdown/EOF;
it marks R's own terminal/reference and final aggregate retirement unknown after
C death. Live prerequisite shorthand is corrected; frozen predecessors untouched.
MUSE accepted R2 as a repaired, explicitly BLOCKED feasibility proposal/gap
disclosure only after fresh non-author VLAD review
`01a11e9f-dcca-7411-b08a-0d6e282c698f`, with no P1/P2/P3 in the focused delta.
Packet `/Users/bobai/.codex/artifacts/2026-10-09/principal-manager-owner-feasibility-design-r2`,
manifest `31946e74bd7aeac8909065019c9e390cf07de87ff4067e0db4f5603d57fa1235`
(17 rows), design `39bac9a000cc0465ff74d33b8104217f5d0620554b7a3be274aa44387225e710`:
actual BEFORE=R1 AFTER, both narrow diffs/AFTER, 11 equal source guards and seven
source anchors (2,082 lines) reconciled. Frozen pending words and rejected R1
remain unchanged. Acceptance selects no v257/R topology or relaxed requirement.
Complete independent retirement after C death is still unsatisfied and is the
architecture blocker before downstream serialization. A later source-only seam
is NOT cleared: closed roles/enums/bitmaps/sequences remain unspecified. Stop for
next-slice disposition; no systemd calls or positive constructor is authorized.
Step 7 stays OPEN; Step 8/V12.2 inactive. Site/Skills/Examples/Learn/FAQ/Glossary,
public commands/schema/JSON and consumer pins are unaffected by private planning.

### Accepted Predecessor: Manager-Transfer Gap Analysis

The accepted existing-route analysis found neither listener/boot activation nor
stopped-child PID scopes currently demonstrate complete same-object C/S/W handoff,
independent S service completion and outer retirement. MUSE and non-author VLAD
review `01a11dc8-0ee4-7521-a2a0-8e1841feb467` accepted gap analysis/prerequisite
requirements ONLY with no P1/P2/P3, not feasibility or route selection. The new
proposal above has a separate scope clearance; no implementation is implied.
The earlier unrelated reviewer reply was rejected. The accepted review packet is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-manager-transfer-design-r1`;
manifest `e96fa42fdfc8a4c1ac32b0c7f213ffa4bfac3f638d4b851d71c0e96f1c915b47`
(17 rows), design `9c2e0c2aec7b39d79f1c576d46a53d391a4c230f223056c726014b271733f5ef`.
It retains actual pre-edit documents, complete narrow diffs and seven exact source
anchor files (2,150 lines); all 11 scoped pre/post source hashes match. Frozen
pending wording remains historical. Hosted CI readback was verified by SCOOBY
and MUSE, not VLAD. The accepted source/acquisition/clock requirements and proof
limits remain unchanged and are retained below and in the private document.

### Paired Step 7 Source Publication Checkpoint: 8 October 2026

Bobai authorized committing and pushing the accepted private source checkpoint
before any next slice. This publication does not activate another slice, merge
to main, release or authorize a live attempt. The source/configuration/test
inputs still match the accepted r10 inventory; only private design and factual
validation-status documentation has subsequently changed. The expectation-source
r1 binding recommendation is accepted after independent VLAD review and MUSE
disposition, not as implementation readiness or installed authority. MUSE-owned `ROADMAP.md` and
`docs/planning/agent-execution-governance-core/plan.md` changes are excluded from
this checkpoint. Core's production workflow Launcher pin remains
`aa55319fa88f14e96b47e3fa9d08a0940fae5456`; it is not replaced by the private
staging revision. Core's six exact-head checks completed successfully at
`8926ddb1fc0e4a225059c48375fc6e1185081b1e`, including Release Gate
`37843348131`. Launcher has no exact-head run at `60c22cf9b2e8aa5897b56430bf27a038fd7b919a`:
its CI triggers only on pull requests or pushes to main, not this implementation
branch. Launcher exact-head CI is unproved/open before merge/release, not a failed
or pending run; no PR, workflow change or dispatch is authorized here.
Installed/root/hosted/provider proof and Step 7 completion remain open.
The private adapter's normal entry remains unconditional no-IO refusal. The
observer has guarded observation/IO paths, not an inert stub; it remains
uninstalled/unconnected with privileged invocation separately gated.

The Launcher Stage A account/process extraction, Stage B private observer and
private adapter checkpoint are committed on `1.6.29-implementation` at
`60c22cf9b2e8aa5897b56430bf27a038fd7b919a`, based on
`77478ae9fad170d2484aa4409b0bb3032436a52c`. Stage A changes
`src/main.rs`, `src/closed_profile_observations.rs`, `src/pressure_provision.rs`, and
the new private Linux-only `src/principal_observations.rs`. This Core handoff's
pre-publication base on `1.6.29-implementation` is
`1b868e1036493e0b7042dc88cbd375ce930316a6`, not the handoff commit's own identity.
That independently reviewed ancestry-only synchronization includes main
`23ca11096211c8fc25c945ed0d684dc5859c8cbc` while preserving the exact pre-sync
development tree. Main's inert registration stubs and stub-only test remain
main-owned; a future development-to-main merge must separately reconcile them.
The carried handoff and Launcher source were not included in that ancestry merge;
they belong to this separate publication checkpoint. Core source checks at the
published handoff revision passed; Launcher exact-head CI remains open as above.
No prior exact-head CI or live admission transfers to either new build identity,
and these source checks do not establish installed execution or Step 7 closure.

MUSE independently accepted the exact source and sandboxed canonical Linux/arm64
verification: 231 top-level tests passed, none failed, 26 privileged tests remained
ignored, and all 15 new regressions passed. The frozen source packet is
`/tmp/ota-step7-principal-extraction-20261006-r2` (manifest
`4ecd5f9f00b925784b3c96b96faf3e5d77209b0040a75a29e36af1fc2f7d298d`);
the validation packet is `/tmp/ota-launcher-validation-20261006-r5-R65mZA`
(manifest `8b839dcc0f7dd87ed9a7456ccefadf99bb8f9804da38fcbe7dd251cb1bdb71ee`).
These local temporary packets are not published or durable hosted proof.

MUSE also accepted the frozen macOS source-non-impact projection at
`/tmp/ota-launcher-macos-impact-20261006-cPSFyY` (manifest
`c191da8b68e0e7878dd41fef9abe9b25d3c37a98efe85407871229621f2822ac`):
the exact delta leaves macOS-target-selected functional Rust paths unchanged
across existing features and targets. Build identity metadata can differ; this
does not establish artifact equality or a successful macOS build, formatter or
test run. On 6 October, Bobai explicitly accepted this independently reviewed
non-impact evidence in place of the local Stage A macOS execution prerequisite.
Local macOS canonical verification remains `not_run`; exact-head macOS CI remains
required before merge or release. No CI, contract or portable-refusal requirement
is removed by this disposition.

Stage B has source-implementation approval only. MUSE independently accepted
the revised design at
`/tmp/ota-step7-principal-stage-b-design-20261006-r2-J1atp6/design.md` (manifest
`ea1788a6356aeba2916070f1fc0ba00e71f22c3ea3b0df661234d27eb751a689`).
Implementation adds a private optional Linux observer target under the existing
`systemd-v3-pressure-provision` feature, closed framing and expectation
reconciliation, bounded native collection, and supervised child lifecycle.
It does not wire the target into installation, registration or composition.

The immutable formatted source packet is
`/tmp/ota-step7-stage-b-source-review-20261006-r4-UwwZWK/launcher` (manifest
`bf8a757145a083960a6fa7526990c1ab22cb1d91638c22188aa7efe0cb29f0a3`).
Canonical nonroot Linux/arm64 `ota run verify --agent --plain --stream` passed
fmt, check, Clippy and tests: 294 top-level tests passed, none failed, 26
privileged tests remained ignored, and all 43 unique new Stage B tests passed.
The separately invoked publication subprocess test is not double-counted.
Validation packet `/tmp/ota-launcher-stage-b-validation-20261006-r4-kaAlu4`
has manifest `8c2b69ef92f2711e8074a4b9af47a8d2a28c2979cbf726b2d31afe73ac441a3b`.
Its ten validated source/doc/Cargo hashes match the working Launcher. The r1
Clippy failure remains a failed attempt; no tests ran in that attempt.
These temporary packets are local source evidence, not durable hosted proof.

MUSE's r2 source review found a P2 combined-production-path regression gap.
The r3 repair adds eight tests driving production acquisition, request release,
exchange, owned cleanup, unprivileged preexec refusal and terminal delivery,
without weakening root checks or adding a production fixture selector.
MUSE closed that P2 in r3, then identified a P3 test executable-FD collision.
The r4 fixture uses the existing high-descriptor helper and asserts FD >=5 before
the unprivileged preexec refusal. MUSE independently accepted exact r4 source
and its bounded Linux/arm64 verification with no remaining P1/P2/P3 findings.
This is not final commit/PR/release readiness. New-target macOS verification is open:
the Stage A non-impact waiver does not transfer. Unprivileged kernel fixtures
exercise lifecycle, signal, descriptor and deadline primitives; injected
collection fixtures do not prove real protected account/shadow observations.
The complete root-only descriptor-exec/collector success, post-exec allowlist,
installed integrity, outer descendant/resource containment and host visibility
remain `not_proved`.
The existing private operator composition remains unconnected/refusal-only;
caller-side Python NSS and outer supervision remain integration gaps.

MUSE independently accepted the new-target portability and paired source-boundary
assessment at
`/tmp/ota-stage-b-portability-reconciliation-20261006-r1-L6W4Yt/assessment.md`
(manifest `eba80f77ed9cef6b441fa29c4ea17ec939df0e43bc5a363b075c24cc8da4402c`,
10 inputs verified). The existing feature selects the optional target under
all-feature verification; its macOS source excludes all three observer modules
and selects fixed exit 2. This is source reachability, not a macOS build, runtime
refusal or no-IO observation. Canonical Cargo tests use the binary test harness,
not its normal main; even a future green macOS verify does not close that runtime
gap without a separately bounded normal-binary check.
Core has only this handoff delta, Launcher matches the accepted r4 source, and
Protocol remains clean at the unchanged normal/dev/lock pin
`e5fe1c83e562e02f60e27026c7148918bd016155`. This source-boundary reconciliation
does not establish successor commit pins, clean artifact or installed admission.
No native macOS build/probe ran: the recorded trusted probes did not establish
the required address-space/data enforcement, and no adequate bounded native
lane has been established. Do not use an unbounded fallback. On 6 October,
Bobai explicitly accepted this independently reviewed source evidence in place
of the local Stage B macOS execution prerequisite, for exact r4 source manifest
`bf8a757145a083960a6fa7526990c1ab22cb1d91638c22188aa7efe0cb29f0a3`
and the assessment manifest above only. This is a fresh Stage B disposition,
not transfer of Stage A's waiver. Local macOS verification remains `not_run`;
no actual macOS build, runtime refusal, no-IO or artifact equality is proved.
The disposition does not transfer to changed source or another target and
authorizes no commit, push, installation or live action.

MUSE independently accepted the private integration design at
`/tmp/ota-step7-observer-integration-design-20261006-r2-8mMOOi/design.md`
(manifest `665f0c566c2aaafb1a15416d86966ecb0ca482ca53160b672f6ba5cd0af25a1a`)
for the narrow source-only native adapter and nonroot release/reconciliation/
composition-consumption fixtures, with no blocking P1/P2/P3 findings. The two
manifest entries, nine input guards and accepted Stage B r4 source were verified
at the review boundary, before this handoff update. These temporary packets
remain local design/source evidence, not durable hosted proof.
R1's three P2 design holds are addressed: capability-only root isolation is
replaced by mandatory independently enforcing per-attempt MAC policy; a retained
active parent slice carries recursive emptiness through worker-cgroup pruning;
an inert trusted worker and actual containment readback precede request release.
The selected systemd 255 lifecycle is source-semantics evidence, not an installed
manager/version or kernel enforcement observation. Exact materialized AppArmor
policy and r4 compatibility, installed closure/pipe/manager carrier, bounded
external controller/watchdog and kernel negative controls remain open. The
permission-transition/Listener phase needs its own outer lifecycle admission.

Bobai's subsequent direct approval authorized the narrow source-only implementation,
not root execution, infrastructure, installation, activation, commit or push.
The first private adapter consumption increment is implemented and independently
accepted by MUSE at exact r3 source manifest
`b2f2bded157c6a7811f28ed4093d1ec3df07d9a7dd9b5bc43a627a83679e7092`.
Frozen source and bounded validation are retained at
`/tmp/ota-principal-adapter-consumption-validation-20261006-r3-GsDyAs`
(validation manifest `81b4b9f5e62f87f0222211ba4e2e5c9de68875756316ac038a99df547d09cb8c`).
All thirteen validated file hashes match the Launcher working tree. Canonical
offline Linux/arm64 `ota tasks --safe --use` then `ota run verify --agent`
passed fmt/check/Clippy/tests as UID 65534 under fixed resource/deadline bounds,
read-only image, no network/capabilities and no-new-privileges. The run has 342
top-level passing tests, zero failed and 26 privileged tests ignored; all fifteen
unique new adapter tests passed. Thirty-three canonical tests linked in the new
target are repeats, not new coverage; the publication subprocess passed separately
and is not double-counted. Validation containers were removed.

The staged kernel reuses canonical expectation/terminal framing and reconciliation,
requires readback-before-release and EOF, caps bytes before retention, refuses
nonempty stderr/nonzero observer exit, and separates observer/wrapper/worker
outcomes, first failure and outer retirement. The consumption kernel takes the report by
value with a final deadline/owned-identity check; this is in-process one-use, not
durable attempt admission/replay protection. Readback/completion inputs are synthetic
controller facts in these fixtures, not acquired OS proof. RUBY supplied a read-only
test matrix. MUSE's r2 P2 uncovered a stderr test rejecting through the wrong phase;
r3 adds a bounded diagnostic before EOF and asserts exact `StderrPresent`, byte
count, raw-output discard and refused continuation. Final r3 review has no remaining
P1/P2/P3 findings for this inert increment only.

Bobai's subsequent direct source-only approval permits the minimum shared lifecycle
extraction. That extraction is implemented at frozen shared-lifecycle r4 source manifest
`15f9bc3c1c9be9eee5a8877aa2c7588e312b31f53866d7ed8b578030d1cb452e`, retained at
`/tmp/ota-principal-shared-process-validation-20261006-r4-9K4GBu`
(validation manifest `484ff997af5f3686fed7dc39fe529c83e000838324b41ff101e845c4f3b31e50`).
At that extraction checkpoint, all fourteen validated hashes matched the working tree. The private
process module retains the fixed collector arguments, environment, limits,
pre-fork preparation, pidfd/session ownership and cleanup sequence. Collector
artifact admission, challenge construction, interpretation and terminal delivery
remain in their existing typed owner; reconciliation runs once at the original
response boundary before cleanup, using the original deadline. No configurable
process API, duplicate supervisor or Observer invocation variant was added.
The old collector fixture module is byte-identical after removing the one new
seam test. That test checks held/unreaped normal-zero exit at reconciliation,
one-use callback, original deadline, refusal cleanup budget and exact reap.

The retained r3 preparation packet at
`/tmp/ota-principal-shared-process-validation-20261006-r3-atJJv2`
used the checked official v1.6.28 ARM64 release bootstrap to build clean pinned Core
`63297ad95ef156b335a1dfb54090146406077a3f` through its declared
`ota run build --native --agent` task, not raw Cargo or an installer script.
This bootstrap preparation required network for rustup's declared refresh; Cargo
remained offline and scratch rustup self-update was disabled. The container was
then disconnected and zero network attachments asserted before its Launcher
verification. The final doc-adjusted r4 run reused the retained exact Core binary
SHA-256 `59c91ef4e74f846dde832927f4d7c25509158474a8c47d3a8152fc5213a8ea9b`,
rechecking its clean source identity. Its container used `--network=none` from
creation with Docker mode readback, before `ota tasks --safe --use` and
`ota run verify --agent`. Offline Linux/arm64
fmt/check/strict Clippy/all-target/all-feature tests passed as UID 65534 under
fixed resource/deadline bounds, read-only image, no capabilities and
no-new-privileges: 343 top-level passed, zero failed, 26 privileged ignored.
The publication subprocess passed separately and is not double-counted. Observer
target coverage is 22 supervision tests plus 33 canonical relinks; only one test
is new in this extraction. Adapter coverage and source are unchanged. Validation
and toolchain-export containers were removed. Earlier bootstrap/fixture-import
failures remain retained; none is positive proof. MUSE found no code/regression
findings in r3 and one P3 stale-hash documentation claim; the doc-only r4 correction
is implemented. MUSE's final independent readback accepted the exact shared-lifecycle
r4 source, recorded validation and this handoff checkpoint with no remaining
P1/P2/P3 findings. This is extraction-only acceptance, not completed observer
integration or commit/release/live readiness.

The subsequent direct source-only instruction implements the staged held-observer
increment at `/tmp/ota-principal-held-observer-validation-20261006-r2-7Cfzgl`,
source manifest `6f8ecf73f9c6e2a2b4ca21a7ab77a2270b64a2ed320f1535d474375ecf56ca28`,
validation manifest `de56576f5a4043b90790668190038bfac9922d32706bd82f1d249ec71fa9339e`.
The shared loop has only closed Collector/Observer calls, fixed arguments,
environment and resource hardening. Adapter `observe_held` consumes one released,
readback-bound exchange under the original absolute eight-second budget; its held
descriptor is not installed executable/closure admission. The observer's own
five-second operation and one-second failure budget remain unchanged.
Actual read counts, EOF and exit are sampled before cleanup, never manufactured
from reap or closed descriptors. Read counts include consumed bytes before a
post-read refusal and the capped overflow probe, not all bytes emitted. The typed
terminal is reconciled before cleanup; a valid refused/exit-zero fixture selects
failure cleanup, while actual observer refusal retains exit 2 and cannot reconcile
as accepted. Closed transport cause, inner cleanup, observer, wrapper and externally
observed worker exits remain distinct. Incomplete cleanup, unknown exit or missing
EOF block continuation even with synthetic successful outer retirement.

Bounded offline-from-creation Linux/arm64 canonical `ota run verify --agent`
passed fmt/check/strict Clippy/all-target/all-feature tests as UID 65534 using the
same exact clean pinned Core binary: 357 top-level passed, zero failed, 26 privileged
ignored. The publication subprocess passed separately and is not double-counted.
Adapter coverage is 21 adapter tests, four shared-loop tests and 33 canonical
relinks; observer coverage is 22 unchanged supervision tests, four shared-loop
tests and 33 canonical relinks. The collector fixture module is byte-identical to
accepted extraction r4. New syscall fixtures exercise the production transport
kernel, including refused/nonzero exit, stderr/overflow/trailing frame and missing
EOF. Deterministic post-read timeout/cancellation seams lock honest counts.
The actual held-descriptor launch fixture proves only unprivileged pre-exec refusal,
not admitted/root execveat success, NSS, descriptor exclusion, systemd/MAC/cgroup
enforcement or watchdog. Outer readbacks remain synthetic controller inputs.
At that held-observer checkpoint, all fourteen validated hashes matched live source; source/diff were unchanged by
verification and both validation containers were removed. r1 strict-Clippy's
unused cross-target shared-field/helper failure is retained, not positive proof;
r2 narrowly documents the cross-bin consumers. MUSE independently accepted this
exact source, retained validation and handoff checkpoint
`5e876ebff339cae2bfae4276b2f3d80f1d39063d651925a11b610f23f6f0b4c5`
with no remaining P1/P2/P3 findings. Ten unique new tests are linked as fourteen
additional top-level passes, not fourteen distinct tests. This is bounded staged
source acceptance only, not installed/root/connected readiness or commit/live clearance.

The next human source-only instruction stages before/work/after composition, not
the runtime bridge. Launcher source is frozen at
`/tmp/ota-principal-composition-validation-20261006-r2-MHgYlA`, source manifest
`762194ef0d56ad6467070f6d0f9b5a035ecc9456c57d53c595d65738f1f8ffb0`, validation
manifest `88af23c07e7c62f4d20b083068d834c491da0c3207d84448d77ad5b0822b5924`.
One-use native states preserve the before report's absolute clock, bind a distinct
work-phase owner to the same installation/expectation and withhold numeric IDs
until independent phase readback. Work transport and exact-owned Completion must
reconcile separately from observer completion. MUSE's r1 P2 found that work
transport had no ownership check despite identity-bound Completion. R2 binds
transport to the exact attempt, worker/parent invocation, retained cgroup and
installation, refuses component substitution and preserves an existing failure
even with identity/retirement errors. The r1 source/365-test evidence remains
historical and nonaccepted; a green suite did not close that missing predicate.
After that retirement cut, the gate
constructs fresh observer waiting state using the same original budget; final
consumption matches its exact acquisition start. Precomputed distinct-ID reports,
substitution, transport/worker/parent/retirement failures and phase deadline expiry
cannot yield a continuation token. First failure is not cleared by later cleanup.

The historical Python composition packet remains immutable. Its derivative at
`/tmp/ota-principal-composition-staging-20261006-r2-p3ixD4` has source manifest
`cf6058f92e9f068e4fcd2bbf34a35d9fa61070b0970d7bbaf45a2637c6d15985`, validation
manifest `12fc5a1bdaef4e626fc09ec00717ea1b0d6bd0f8c2ca80badc27642953e97eb5`.
It preserves the exact pure service-property predicate/constants (AST equality)
and splits strict numeric validation from the unchanged root guard. The numeric
kernel reuses protected-root/archive/tree/state/permission/fixed-Listener predicates
without caller NSS or named pgrep. Numeric parameters are data, not admission;
Python parses no native Report and returns only kernel-complete with native
composition/live flags false. Legacy Listener timers/process-group cleanup remain
unproved and cannot supply the separately required owned outer lifecycle.

Offline bounded Linux/arm64 UID65534 canonical verification passed
fmt/check/strict Clippy/all-target/all-feature: 366 top-level passed, zero failed,
26 privileged ignored; the publication subprocess is separate. Nine new Rust
tests use explicit synthetic controller/transport facts; no permission/Listener
body is executed. Nine Python tests passed in a separate network-none, read-only,
no-capability/NNP UID65534 container with scratch-only writes and explicit
time/process/memory/IO limits. Those execute pure predicates and projected calls
only, not root acquisition, permission mutation or Listener execution. R1 Python's
unclosed test-source warning is retained; r2 closes it and adds AST equality proof.
All fourteen validated Launcher hashes match live source and canonical verification
left source/diff unchanged; all validation containers were removed. MUSE accepted
the combined Rust r2 and unchanged Python r2 source staging, retained validation
and exact Core handoff SHA
`362fca9deebdba8fbca1d2d3725a99f5225eaa98e066d1920a2fa7430816d03c`
with no remaining P1/P2/P3 findings. MUSE independently inspected source, retained
logs/hashes and scoped container metadata; it did not rerun tests or kernels.
This is bounded source acceptance, not commit, installed/root or live clearance.
Native controller/readback acquisition, service-property
acquisition, exact installed closure, contained kernel transport and the actual
before/after runtime connection remain open. This closes none of those live holds.

Dependencies and lock remain unchanged. The optional adapter entry is source-level unconditional exit 2 without
IO, not an observed normal-binary execution. Existing composition stays
unconnected/refusal-only. This is not completed observer integration: installed
admission and root execution of the held observer remain unproved; connection to
the existing composition is unfinished SOURCE work alongside controller holds.
The 7 October source-only successor stages finite verified input handles before
the contained carrier, not installed-closure admission. Launcher snapshot
`/tmp/ota-principal-inputs-validation-20261007-r1-AvM2pA` has fifteen-entry source
manifest `293b3e61a05c128ceeb647636fc48a5ac8ff2c1f86b9b19e1c67aca9f450e7fe`
and validation manifest
`b41369e10dc97b332f24ef4e15898fa628f92bf39f8852618eba1cc023810336`.
The source delta from accepted composition r2 is the private input module,
composition retention/recheck predecessor, private adapter documentation and
Unreleased entry. MUSE supported that narrow scope before candidate review;
bounded source acceptance is recorded below. Exact eight-role count/order binds observer,
interpreter, five existing numeric-kernel modules and runner archive to the
native installation expectation. Read-only/CLOEXEC root:root singular regular
handles retain exact digest/length/mode and stable dev/ino/type/owner/link/size/
mtime/ctime samples. Bounded offset-zero pread and exact EOF use the original
clock and existing signal checks; recheck consumes its state. No handle extraction,
serialization, path reopening, interpreter call or discovery flow is added.
Path labels do not prove descriptor path/ancestry/namespace. Mutable files are
not made immutable and blocking filesystem IO is not made interruptible. The
finite set is not a complete interpreter/stdlib/loader/library/NSS/profile closure;
its acquisition and independently enforced namespace/MAC remain open. The future
carrier must consume the retained handles rather than reopen labelled paths.

Canonical offline bounded Linux/arm64 UID65534 fmt/check/strict Clippy/all-target/
all-feature verification passed 375 top-level tests, zero failed, 26 privileged
ignored; the filtered publication subprocess remains separate. Nine new tests
distinguish actual descriptor/hash/offset/EOF/read-error/cancellation mechanics
and nonroot-owner refusal from projected protected metadata/sample predicates.
There is no positive root-owned complete-input or execution proof. Successful
protected-set duplicate-inode refusal is not exercised; the actual nonroot retain
fixture refuses on ownership. All fifteen
frozen/validated/live hashes match; source/diff stayed unchanged; the named
validation container was removed. MUSE independently accepted frozen inputs r1,
its retained validation and exact Core handoff SHA
`66204f146e0e6ff88ef93b6982a557332bd119bcbc5dbb2c52999e6d02e80643`
with no blocking P1/P2/P3 findings. MUSE inspected source, hashes, retained logs
and scoped container metadata; it did not rerun target builds/tests/kernels.
Acceptance is bounded source staging only, not installed/root/runtime or
commit/push/live clearance.
That input-handle checkpoint did not change Python kernel source or its accepted
predicate evidence.

The 7 October private descriptor-kernel successor removes the archive-path reopen
from the staged numeric route without adding interpreter execution. Frozen packet
`/tmp/ota-principal-fd-kernel-staging-20261007-r3-2585JI` has eleven-entry source
manifest `8dba1c3135c71bbe730b9229cd10d8274ebd6e655596680d596a383adefe9780`
and validation manifest
`194cad008cb6f1b6ef2d9f12f5af9c5f1c3c587a401a6c17a0b3f055b0be4f03`.
The borrowed checked archive descriptor is privately duplicated with verified
CLOEXEC, read-only/non-O_PATH access flags and same complete metadata; bounded
pread preserves the caller's offset. The borrowed-slot precheck is retained,
but owned-duplicate flags are independently checked before constructing the reader.
Shared package-policy parsing is bracketed by exact digest/EOF and complete
metadata checks, including after the final digest. The numeric fd route fixes the
canonical archive expectation; the legacy pathname route retains substitution
checks and error order. This is data verification, not native input admission.
The isolated offline Linux/arm64 Python 3.13.5 UID/GID65534 run passed 21/21;
source, driver, runtime identity and container configuration are retained. Tiny
archive descriptor/alias/digest/metadata tests are actual nonroot operations;
protected tree, permission-transition and Listener tests remain projected. The
initial r1 run failed only on a literal-escape gzip-header assertion; its source
and failed log remain retained unchanged. MUSE held r2 for a P2: pre-dup caller
slot reuse could yield same-inode O_RDWR without the owned copy's flags being
checked. Its related P3 identified missing read-time EOF/growth coverage. r3 adds
the duplicate guard and deterministic pre-dup different-inode/O_RDWR/O_PATH
refusals, private-copy closure and caller survival/offset checks. Actual fixture
truncation/growth after metadata admission reaches the reader's EOF guards.
The negative packet `/tmp/ota-principal-fd-kernel-negative-20261007-pocI4k`
uses r2 production source with the new substitution test: both same-inode cases
fail, while different-inode refusal passes. Its retained validation manifest is
bound in r3's validation manifest. Neither prior packet is rewritten.
No Rust source changed, so accepted
375-test Rust evidence retains only its original scope, not Python execution.
MUSE independently accepted exact r3 source and retained validation plus Core
handoff SHA `5db4a14880c0e11ea35b6ba49667f5013aea0fa0e00b9845fde89442d08f11fb`
with no remaining P1/P2/P3 findings; both r2 findings are closed. It checked all
source/validation/doc entries, the negative discriminator and unchanged Launcher
source, and inspected retained logs/configuration without rerunning target code
or tests. This acceptance record is a record-only handoff successor; the reviewed
source packets and frozen review docs remain unchanged. Acceptance is private
source staging only, not commit/push, installed/runtime or live clearance.
Normal entries remain refusal-only.
No native descriptor transfer, isolated interpreter/module loading, installed
dynamic closure, root admission or actual before/work/after execution is proved.

The next 7 October source-only predecessor is compile-only module preparation,
not the initially considered peer loader. MUSE's superseding scope removed all
resolver/initialization/execution scaffolding before validation; no real kernel
body was executed. Frozen packet
`/tmp/ota-principal-module-sources-20261007-r1-A5UixV` has four-entry source manifest
`6cb5d02aebc0879125a7639547a95e70d5b0497f11a85b21b455d500119b69d6`
and seven-entry validation manifest
`a2ae4d277b803afcec1b5adcbba8c609a212e41762774bce8a09f2bfd8c1b1c5`.
Five exact native module roles retain their order and 1 MiB each/5 MiB total caps.
Borrowed and owned-duplicate access flags, CLOEXEC, complete sampled metadata,
bounded pread/EOF/raw-byte hash and distinct objects reconcile before all five
immutable snapshots are decoded as strict UTF-8 and compiled with fixed flags
and synthetic role/digest origins. A complete code tuple is data, not loaded
modules or admission. No source path reopening, kernel import or body execution,
bytecode serialization, timer, resolver or retry API is added.
Eleven fixture tests passed in offline bounded Linux/arm64 Python 3.13.5 UID/GID
65534. Actual descriptor substitutions, read-time EOF/growth, final metadata,
cleanup/offsets and immutable snapshots are distinguished from the projected
aggregate-cap and CLOEXEC-readback fault controls. Two retained negative packets
remove the duplicate access guard or decode before whole-batch verification;
their selected tests fail as expected. Their exact validation manifests are bound
in the successor's validation manifest. No real permission/Listener module body
or root operation was tested. MUSE independently accepted the exact frozen source,
retained validation and Core handoff SHA
`82834180ba128d48b0d4299e557e3d3ebcea754a53d3114000f46ca125d1fea9`
with no P1/P2/P3 findings. It checked all source/validation/doc entries and both
isolated mutant diffs/logs, without rerunning target code or tests. This is bounded
private source staging only, not loading, installed/runtime, commit/push or live
clearance. The acceptance record does not rewrite any frozen packet. Rust source remains unchanged,
so prior Rust evidence is not new Python/interpreter/runtime evidence. Native
build-owned embedding, handle transfer, closed delayed-peer resolution, installed
dynamic closure and independently contained execution remain unimplemented.

The finite peer-resolution design was independently reviewed before implementation.
Frozen `/tmp/ota-principal-peer-design-20261007-r1-Mih10I/design.md` has SHA
`289858ea9ca9d4326eeb4793bac81e7b7ab2caa77971ecbc6cf51c708a286a3a`;
its one-entry source manifest has SHA
`f59ec763c198db5b20320a6774894cb63f2443cbd65cf70dcc1a5c445b0f44b0`.
MUSE verified the design and both predecessor source packets, checked the actual
direct/delayed import graph, and returned no P1/P2/P3 findings within the stated
trusted-verified-source scope. Fixed private module objects, per-role local import
dispatch, exact plain/from forms, retained delayed routing and sampled reserved
cache collision refusals avoid peer fallback or global importer/cache mutation.
Initialization is one-shot, with no unready/cyclic or partial-bundle exposure and
no rollback or renewed-clock claim. This is design acceptance only: no router,
module body, target compilation/execution or tests were run in this increment.
Native Launcher source remains unchanged. The real ctypes body effect and all
installed closure/containment gates remain explicit; no arbitrary-Python sandbox
or concurrent global-mutation guarantee follows.

Bobai's subsequent source-only instruction authorized the synthetic router
increment. Historical r1 at
`/tmp/ota-principal-peer-sources-20261007-r1-QIZ2m9` has six-entry source manifest
`d2de08593de212e89dc1a6c10c5214d7d91599aa6366dc0ec21470c3016815c9`
and nine-entry validation manifest
`3922b51b5b582a3991bf70220f9b2b3643e4bb8a0f55b141e551c093dd3974c9`.
Its 24 passing tests do not clear MUSE's cache-key review finding: the scan skipped
string subclasses despite dictionary equality with reserved names. R1 is not
accepted source and all its packets remain immutable. Frozen r2 at
`/tmp/ota-principal-peer-sources-20261007-r2-PnoILU` is independently accepted,
with six-entry source manifest
`acbde0a8725ed0d322c8acabd5207f1dd1679824907ce7f356c75db123fe3111`
and nine-entry validation manifest
`67fe7a3ff8442f5ba6d63f56da894e3d13546db049c59e8bd47a193c3c6b3a72`.
It refuses every non-exact-str cache key before user comparison or prefix calls;
canonical strings retain the exact/dotted reserved-name rule. New tests exercise
startup, post-external, gateway and delayed string-subclass collisions plus an
opaque equality alias; refusal runs no custom key comparison. R1 router with r2's
two unchanged new tests produces seven expected subcase failures. R2 also proves
failure of an earlier bundle's retained delayed function leaves a later bundle
intact. The validation manifest binds r1 evidence and the new discriminator.
The unchanged descriptor-preparation predecessor feeds a private one-shot closure,
fixed private module objects and role-local exact plain/from routing retained for
delayed imports. Unready peers, unsupported forms and sampled object/None/dotted
cache collisions refuse without adopting/deleting ambient peers or generic fallback.
Caught import refusal still terminally fails the attempt; a gateway exposes only
the fixed composition function after whole-batch preparation and all bodies.
Twenty-six r2 tests passed in bounded offline Linux/arm64 Python 3.13.5 UID/GID
65534: fifteen router tests plus the unchanged eleven preparation tests. Only
tiny synthetic bodies ran, not real kernels, NSS/CDLL, Listener or permissions.
R1's two isolated mutants permit delayed ambient fallback or failed-attempt retry;
their selected tests fail as expected and exact manifests bind through r1 validation.
All five test containers across r1/r2 are independently absent after cleanup.
The closure is
an unconnected private fixture seam, not native admission, worker ownership,
deadline or production retry authority. External effects are not rolled back and
external transitive imports/installed dynamic closure are not proved. Native
Launcher source remains unchanged; no new Rust or protected-runtime proof.

MUSE completed r1 review and accepted the exact frozen r2 source, retained
evidence and connected documentation with no remaining P1/P2/P3 findings. The
three-doc review manifest is
`088d2f1999999abcc0603953d987bf432a441ef4e24af81d2055152df1f8d9a5`.
It verified every entry and nested r1/three negative packets, inspected drivers/
configuration/logs and independently observed all five containers absent, without
rerunning/importing target code or revalidating the native-source manifest.
This record-only successor does not rewrite any frozen packet. Acceptance is
unconnected private synthetic source staging, not native/runtime or live clearance.

The separately instructed native bootstrap/descriptor-transfer design is now
independently accepted as design only. Frozen r2 is
`/tmp/ota-principal-native-transfer-design-20261007-r2-UPwRCL/design.md`,
source-manifest SHA-256
`c71ee421e3e5d0b7cd232a43a5eaf8eeaf2851c0b684b56f640d85eb8da534c1`,
design SHA-256
`6dc630ea55da6ff2e6074cdc731930be4a56ba5627c6031e0b200ab73e419349`.
MUSE verified the exact packet and r1-to-r2 delta with no remaining P1/P2/P3.
Historical r1 remains immutable/unaccepted: its P3 closing-order ambiguity is
resolved by keeping preparation exactly once inside the one-shot initializer,
then closing original source handles before peer initialization/body execution.
The design selects build-owned standalone bootstrap packaging and finite inherited
descriptor slots without changing the observer launch. It preserves native
attempt/phase/input ownership and original clocks; actual independently acquired
outer containment/readback must precede interpreter release. This is not an
implemented transfer, interpreter launch, installed admission or real kernel proof.
No native source/tests, Cargo/dependency/pin, Protocol, public spec/command/schema/
JSON, Site, Skills, Examples, Learn, FAQ or Glossary change is needed for this
design-only checkpoint; only this handoff and Launcher private documentation change.
No new changelog behavior claim or runtime execution accompanies it.

The separately instructed packaging/consuming-transfer source candidate is frozen
at `/tmp/ota-principal-transfer-sources-20261007-r1-3xJkO0`. Source-manifest SHA:
`b2b6bea29b4496426668ea009528ea372675d6db6adf498d80e2cbc122943960`;
validation-manifest SHA:
`69ab423779d77f0f2b4d9ee2c8d8c743f753f269119dba3db9aa6b8bf73fd40f`.
It consolidates the accepted preparation/router into build-embedded source, keeping
preparation exactly once and closing originals before creating peer objects/bodies.
Native PreparedInputs consumes the existing phase Readback predicate/JobIds route
and exact retained-input recheck into a private description. That predicate is not
actual manager/loaded-policy acquisition. The description retains checked handles,
phase ownership and original clock, with fixed finite slot metadata and bounded
arguments/environment. Its code stub defines the bootstrap only, never initializes
peers or invokes work. There is no raw-handle export, descriptor remap/inheritance
change, fork/exec, release, result wire or admission path added.

Bounded offline nonroot Linux/arm64 Rust 1.95 canonical pinned
`ota run verify --agent --plain --stream` passed with 379 tests and 26 privileged
tests ignored, plus source consistency checks. The nested subprocess test summary
is not counted twice. Python 3.13.5 passed 30 synthetic preparation/parity/router/
closure tests. The isolated missing-close mutant
`/tmp/ota-principal-transfer-close-negative-20261007-UR3pW8` fails both selected
ordering/close-failure controls as intended; its source/validation manifests are
bound by the candidate validation manifest. All three containers were observed
absent. Packing/ownership positives use test-only nonroot fixtures, not a positive
production/root-owned input set; no real kernel, permission, Listener, CDLL or NSS
operation is exercised. The source snapshot retains prior docs; the connected
current handoff/private documentation/Unreleased projection has a separate review
packet. MUSE accepted the exact source, retained validation and three connected
documents as unconnected private source staging, with no P1/P2/P3. The reviewed
three-doc manifest is
`12a6e373a45523eccf112390a46184ed65a93471e71f3fd80691473ca71b028d`.
It verified all entries/nested negative manifests, retained logs and 21 live
source/Cargo/test hashes, observed all three containers absent and checked the live
docs/diffs. It did not import/build/rerun target code. This record-only successor
preserves the frozen source/evidence/docs, including their pending-review wording;
acceptance clears only the source predecessor, not any runtime/installed gate.

The separately instructed finite metadata/kernel-report source slice is frozen at
`/Users/bobai/.codex/artifacts/2026-10-07/kernel-report-r4` against unchanged
Launcher `77478ae9fad170d2484aa4409b0bb3032436a52c` and Core
`1b868e1036493e0b7042dc88cbd375ce930316a6`.
Source-manifest SHA:
`090fd5afd06052f0ad3ccec411b644fc0698319dc207cc02f4cb516bf6fec25e`;
validation-manifest SHA:
`6e7e896298a7ccfc74f65b66dc02505e12f74dbb76f64ded9ef0b8d2292f1c82`.
MUSE cleared the exact inert candidate with no P1/P2/P3, checking all 242 source
and 37 validation entries, all 76 live Launcher hashes, each compiled-source list,
the exact mutant deltas and retained assertion failures. MUSE also accepted the
exact connected private-doc/Unreleased/single-handoff projection with no P1/P2/P3;
its six-entry manifest is
`f17537c4cd584685ca6b69f620d8ad92534be290f9f58ba7ed9d367c217a61ba`.
It checked all three actual deltas/live document hashes and the other 74 unchanged
Launcher files. This approval-status-only successor preserves the frozen
pending-review packet and source/validation identities. Acceptance clears this
source batch only, not runtime/installed gates or commit/push permission.

The finite native JSON metadata preserves exact runner_* roles/module slots 5..9,
archive slot 10 and canonical numeric IDs/caps. The pure Python projector validates
the closed synthetic result/limitations and closes the archive before returning
the 1,024-byte total four-byte/JSON frame; the definition-only stub never calls it.
Private native KernelReport requires object-only framing, literal string-only
status/version/ordered leaves, owned IDs and native_composition_complete=false /
live_authorized=false. MUSE's prior P2 was that derived unit enums also accepted
object-shaped variants; String decoding followed by finite TryFrom now refuses
them directly and through coherently length-matched owned-exchange tests for all
seven leaves. Reported kernel facts are not ObserverTerminal, admission or authority.

The private consuming exchange retains PreparedTransfer/captures and reconciles
exact phase/installation, pre-cleanup transport, independently supplied completion,
exact retirement and fresh after-observation under the original 8s/10s ceilings.
Actual completion/readback/capture acquisition remains unconnected; the test-only
bridge cannot promote synthetic facts into runtime admission. Normal native/Python
entries remain inert, with no composition remap/release/fork/exec or real kernels.

Fresh canonical pinned offline nonroot Linux/arm64 Rust 1.95 verification passes
383 top-level tests, zero failures, 26 privileged ignored (nested 1-pass summary
excluded); Python 3.13.5 passes 35 synthetic tests. Removing only the three serde
String attributes reaches the intended schema assertion; removing only the report
decode reaches both consuming assertions. Both canonical negative runs exit 101,
not from setup/build/lint, and all source consistency checks pass. SCOOBY observed
all temporary R4 containers absent. MUSE inspected retained evidence/configuration
only, with no Docker query or target imports/builds/test execution. The bounded
nonroot clean pinned-CLI bootstrap alone used network for declared rustup metadata;
Launcher/tests/mutants were offline. No authority or protected host was installed.

Evidence availability: previous `/tmp/ota-principal-*` execution evidence remains
unavailable; its identities/review outcomes are historical, not retained proof.
The private numeric-kernel source predecessor is now recovered in the durable
`/Users/bobai/.codex/artifacts/2026-10-07/kernel-source-recovery-r1` packet.
Its eleven-file source manifest exactly reproduces accepted descriptor-kernel R3
identity `8dba1c3135c71bbe730b9229cd10d8274ebd6e655596680d596a383adefe9780`;
recovery manifest is
`ab674e9e656433ac392fbfe0dabbf0159b0b20a2a9786cf237bc559ec8d6d8cd`.
MUSE independently verified both manifests, all eleven source hashes and the exact
historical archive anchor, accepting source recovery only with no P1/P2. The P3
private-document propagation omission is corrected alongside this handoff in a
successor sidecar; frozen R1 is unchanged. Recorded file changes/static copies
were interpreted only as data. During recovery, no recovered module was imported or executed;
old 21/21 logs/container evidence remain unavailable and no fresh derivative/parity
proof follows from recovery or the golden fixture. The recovered driver's obsolete
temporary paths must not be executed.

Fresh bounded synthetic revalidation is independently accepted at
`/Users/bobai/.codex/artifacts/2026-10-07/kernel-synthetic-validation-r7`.
Driver manifest is
`a1d632a3498626e296e295a3d8def8e08fccd5ed45dbb1644bf9ea783c3b6188`;
validation manifest is
`d28211e62ed0b6b5f7f2798ee3accdd354b796a6b95bdae5fb82f43d80ba153e`.
The recovered eleven-file positive source still matches historical `8dba1c3...`.
Canonical pinned Ota `run verify --agent --native --plain --stream` in the bounded
offline nonroot Linux/arm64 Python 3.13.5 fixture passed 21/21 tests without errors
or skips. Removing only the three owned-duplicate access checks produced the two
intended same-inode writable/O_PATH assertion failures in one negative-control
test; the unmutated different-inode refusal remains. MUSE independently verified
all 33 driver and 45 validation entries, raw results, containment and exact owned
container retirement/absence. Both worker records retain
`native_composition_complete=false`, `live_authorized=false` and explicit
`not_proved` groups. Acceptance covers synthetic descriptor mechanics and projected
derivative/kernel/predicate parity only, not actual permission/Listener behavior.
R1 remains unexecuted; R2-R6 remain frozen unaccepted attempts. R5's task exit0
without retained child results was not accepted; R6's intended failures were
rejected by a decoder that also counted Ota's prefixed diagnostic excerpt. R7
requires exactly two complete raw assertion lines. This is new retained evidence,
not recovery of the historical missing execution logs.

The composition-specific native carrier/capture design is independently accepted
as design only at
`/Users/bobai/.codex/artifacts/2026-10-08/principal-composition-carrier-design-r1`.
Seven-entry source-manifest identity:
`2c62fe70e63f6eb849b3cfe64e4d5e639fb042f636e054b398cdc83c5c83ce09`;
design identity:
`b1d217c25b10923e9d100b66a8c905eefce883b44faa4cd01c02546b6de6258a`.
MUSE contributed actor/status-producer advice; non-author RUBY independently
verified the packet and six live anchors with no P1/P2/P3 design findings.
The design requires a distinct native pre-exec latch/readback gate, collision-safe
source slots above10, same-handle transfer, actual interpreter capture/exit and
separate wrapper/manager outcomes, preserving original start+8s/start+10s limits.
The reference controller/supervisor/wrapper/interpreter topology is a producer
requirement, not an installed implementation. Manager-bound descriptor transfer,
protected native relay acquisition, installed/dynamic closure and actual outer
completion remain open. No source changes, target imports, tests, processes or
runtime activation accompany this design acceptance; normal entries remain inert.
Only this handoff and Launcher private documentation need the design record;
no Changelog, dependency/pin/Protocol or public-consumer propagation is required.

Uncovered-material-behavior inventory: finite metadata/framing/consuming source
controls are contract-owned and proved by local declared verification and synthetic
fixtures only. Actual permission/Listener changes, outer acquisition/readback,
release/remap/contained interpreter execution, installed/dynamic closure,
registration/private contents/descendants/providers/selected work are explicitly
bounded or not_proved; the private report's two false fields retain that boundary.
No actual kernel/root/installed/hosted/provider gate is closed. Launcher private
source/tests/docs/Unreleased and this handoff are affected. Dependencies/Cargo/pins/
Protocol and public commands/spec/schema/JSON/Site/Skills/Examples/Learn/FAQ/Glossary
are unaffected for this private inactive surface. No commit/push/merge/release,
root/VM/SSH/protected-install/sign/provider/composition activation occurred.
Step 7 active/open; Step 8/V12.2 inactive.

Bobai's direct "Yes. Also lets work." on 8 October, following the named source-only
question, authorizes this narrow implementation; MUSE approved roadmap readiness
within that grant, not source acceptance or executable verification. Launcher stages
a fixed native byte-plus-EOF waiting seam under the original monotonic deadline and
an unconnected owned inner capture outcome, retaining actual interpreter exit,
pre-cleanup counts/EOF/first cause and inner cleanup separately from wrapper/manager
facts. The synthetic fixtures are verified only within the bounded offline
checkpoint below; normal entry remains inert. Production
hardening/remap, manager-bound transfer and native relay acquisition remain open.

Non-author RUBY accepted the narrow static source at
`/Users/bobai/.codex/artifacts/2026-10-08/principal-carrier-source-r6`, manifest
`cc8366033dc65742fddbd39af677ba8634e80a5e4ba078ff8ef0b1f7823da63f`.
The consuming refusal retains its exchange/raw inner outcome/captures and independent
outer failure. Late-release and parent/input fixtures now isolate their refusal
guards; static review does not establish executed mutation discrimination.

One approved offline attempt, validation-r3, failed canonical fmt and stopped before
compile/lint/tests/mutations. Failed-results manifest:
`16f7fe5f58afa0725f472b1b728306ca61f6711f1f23d31032c64dac9e9e5939`;
exact owned container removal and absence are retained. r6 contains all four reported
manual formatting corrections after r4, independently rechecked by RUBY. validation-r4
remained unexecuted after MUSE found the omitted fourth hunk. MUSE approved one fresh
validation-r5 attempt, driver manifest
`20c4d9f26fbe163671cf584a83d992ab311442be2ab57e47acb05000437614e3`.
Non-author RUBY accepted its retained evidence with no P1/P2/P3 findings. The
validation packet is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-carrier-validation-r5`, manifest
`5f93f1502c2ba789c8ddab20afd2ba22e8166d8975a9a5aefe9fff1fa332b873`;
payload manifest is
`a8a1d35c51a820ba78b1b2171cb60749580781750fddbbfe46daa350653fa810`.
Canonical nonroot Linux/arm64 `ota run verify --agent --native --plain --stream`
passed fmt/check/strict Clippy/all-target/all-feature tests: 391 top-level passes,
zero failures, 26 tests ignored. One nested subprocess pass is not
double-counted. Latch/exit/clock mutations each exited 101 at exactly their intended
assertions; all four source copies retained before/after consistency. The records
named `*-compiled` contain source-input hash readbacks, not executable hashes;
raw canonical traces separately support compilation/test execution. Actual retained
Docker exec exited 0 with no timeout/overflow and matching complete byte counts.
Exact owned container removal returned 0, followed by literal exact-object absence
on the same daemon. This is local offline source/synthetic evidence only, not
production descriptor closure, hardening/readback, installed or hosted proof.
MUSE accepted the focused checkpoint documentation and the technical direction of
`/Users/bobai/.codex/artifacts/2026-10-08/principal-remap-proposal-r1.md`, SHA-256
`82be8bbbde374ead6244643ae6c8854eb02a507903ca46ec917f6937c4b9d59a`.
The subsequently staged and verified increment adds collision-safe composition
descriptor preparation/remap and a distinct native READY-before-release seam,
without changing Collector/Observer launch behavior or connecting release/exec/
readback admission. On 8 October Bobai directly delegated
bounded active-roadmap slice approval and routine offline verification to MUSE:
"This is the kind of approval I was request you take over. We have a road map,
so I believe you can approve the slices too." MUSE explicitly approved this exact
source-only proposal and one bounded offline nonroot attempt after independent
source and exact-driver readiness review. This supersedes the pending founder
question; future source/results still need qualified independent acceptance.
No root/provider/installed/hosted/controller activation, interpreter execution,
commit/push/release or Step 8/V12.2 action follows from that delegation.
The exact source increment is now staged only in Launcher's private process module:
owned duplicates above FD 10, fixed 0..10 map, identity/access/CLOEXEC checks,
unrelated descriptor closure and root hardening before READY/latch wait. Existing
Collector/Observer paths and normal entry remain unchanged. The seam has no
CheckedInputs/manager-transfer connection or interpreter exec/release writer;
mechanical descriptor checks are not installed role/digest/readback admission.
RUBY accepted the exact formatting-only successor source r5, manifest
`9ea7839c737b7f181c20a409c2f292f08a44ccda1d2c79ed9eb2125f40f642d6`.
The first remap offline attempt r3 failed at candidate formatting (exit 1), before
compilation, tests or negative controls; source consistency was retained. Its
failed-result manifest is
`af6f952087ff1ca9e26683640c56081b7ad496117d70832fd68f3367c6c2935f`.
Raw removal returned 0 and exact-container absence returned 1 on the same daemon;
MUSE reconciled the scoped failure/cleanup evidence, not a successful checkpoint.
All seven prescribed formatting corrections are applied without semantic changes.
MUSE held the unexecuted successor driver r4 because its inner manifest guard
still pinned r3. The driver-only successor r5 corrects that guard and uses a fresh
container identity, retaining the same source and negative controls; manifest
`4694a1ea51b06b5145f2c1ac3586b6a69eb9ae8753887982e73d5035347ff81b`.
MUSE cleared one r5 attempt after guard review, but it stopped before candidate
verification: read-only preparation had accidentally removed the toolchain's
executable bits, causing `rustc` permission refusal. Its failed-result manifest is
`509c6040aee39377cb8fa3310548a25ac1b5248a091f4a59c211c0a7da72cb80`;
raw exact-container removal returned 0 followed by literal absence on the same
daemon. This is a preparation failure, not a source-test result. The unexecuted
r6 driver restores reference read-only permissions while preserving execute bits,
with all 745 metadata readbacks bound in driver manifest
`3d5c7eeadf14baa232fe3f9e57fb36df916cf450fc031e47e5dc46c523419a8b`.
MUSE accepted that permission repair and cleared one r6 attempt. Candidate
canonical verification passed, and closure-negative reached its intended test
failures. Flags-negative instead failed strict Clippy on its injected OR pattern;
the decoder refused to count this as mutation proof. Access/clock controls were
not reached. At r6, full checkpoint acceptance remained open; its failed-result manifest
is `5734ab84a10ea6efb187610845e4677aa508c0dd0d5396962ca77c5fab581cab`.
The prepared r7 successor changed only that mutant's equivalent `3..=5` pattern,
re-pins both manifest guards and uses a fresh container identity. Driver manifest
`c846481a3430213f5e4c37eeaa043160c960856b4047e4ba25298ecd20205a37`
binds payload `6fef9ef8d1d8b681192f7638544389f50c98a69917465a119833ce76c45c19ab`
and read-only permissions preserving execute bits. Accepted source r5 is unchanged.
RUBY accepted the narrow mutation delta and MUSE independently cleared exactly
one r7 offline nonroot attempt. Fresh launch readback verified all 745 bound
read-only modes and retained all 15 executable files, exact hashes/cwd/closed env;
that attempt completed with actual host exit 0. The raw candidate canonical lane
has 401 top-level passes, one nested subprocess observation and 26 ignored tests.
All four mutants exited 101 at exactly their intended test IDs; all five source
copies retained consistency. Command stream lengths/hashes and no timeout/overflow
reconcile, with exact owned retirement retained. Frozen result manifest
`8ecf116656f2acb725e9558e3859be4b06681256f9d6409325b3848e68738056`
binds 53 retained files. RUBY independently reviewed the frozen results and MUSE
accepted the exact bounded offline source/synthetic checkpoint with no findings.
All ten required carrier/remap candidate test IDs passed; the raw count includes
one nested subprocess observation that is not an additional top-level test.
All 16 command records/32 retained streams reconcile, and the five source-input
inventories match 380 frozen source hashes; these are not executable identity
proof. Exact container `460fa4dadc14126bd32a163ba0e5809d2820213e4e5f931a2576a94c77950f92`
matched image/name/attempt ownership, with same-daemon removal 0 and literal
exact-object absence 1. Earlier remap drivers r1/r2/r4 remain held/unexecuted;
r3/r5/r6 remain failed, not superseded into successful evidence. No automatic
retry is authorized. Root posture, actual interpreter exec/CLOEXEC across exec,
semantic-input admission, privileged readback, manager/native relay, installed/
hosted/provider authority and integrated production budgets remain not_proved.
RUBY and MUSE accepted the directly connected checkpoint documentation after a
narrow historical-tense correction. The next ownership proposal r1 was held for
a vacuous accepting-baseline risk: unchecked nonroot files cannot pass unchanged
protected-input recheck. Revised proposal r2 separates common helper mechanics
from unproved positive production admission/wiring, SHA-256
`bff4463fc96c9aef3b871b851ecd0950f832d2b2e1e440fddbb83105b739b6cc`.
RUBY independently challenged that baseline and MUSE approved its bounded private
source implementation under the standing delegation, not execution.
That source is now staged: the complete eight-role owner retains optional fixed-map
duplicates; the real wrapper uses unchanged consuming pre/helper/post recheck.
Composition-derived UID/GID, installation and original start are checked by the
same mechanics helper used by explicitly unadmitted dummy fixtures. Canonical
role/spec validation is reused, with independent test role/object/byte oracles and
per-invocation cfg(test) post-duplication observation/signal injection. Production
root predicates and accepted mapper semantics remain unchanged. New ownership,
identity/role/access/clock and cleanup tests are not_run. RUBY held source r1 for a
missing cfg(test) `AsFd` import, then accepted the exact two-line repair in source
r2 with no remaining static findings. Source manifest
`243f502e4ca3fd96d3860e9c9cbffcdf572735a3cc758d97d6daf97f52662940`
and review manifest `162eeb24d7400364331c04d1cc5b33850540f5347e96b6c36cb2fc632b622ec8`
freeze the three sources, baseline diffs and test/mutation plan. MUSE independently
accepted exact source r2 for the static gate only. The canonical offline candidate
plus five exact controls are frozen as `principal-input-handoff-validation-r1`:
driver manifest `5133bfa9b8bdf83f8dd08ebfabc4ad1ec452ab5d5dcbd46f8204580e27396a1e`,
payload manifest `10a3989ec058dfa7fce5c4357a652349c40a0a78088e92b47a2981ea735e3d55`.
Static readback reconciles 719 source files, 856 metadata paths, fifteen executable
files, the three accepted candidate sources and unchanged control tests. RUBY and
MUSE accepted the static driver and cleared one contained nonroot offline attempt.
Fresh launch readback matched the exact tuple. That attempt failed at canonical
candidate formatting: four test-only inputs-source hunks, before compilation or
tests. All five controls remain not_run. Frozen failed-result manifest
`db16698368e8b98bdb8a940118d042dbaddf4d86e2b622ceae8cb7d973d8e7ed`
retains outer session 86308/exit 1, eleven command/twenty-two stream records and
exact same-daemon container removal 0/literal absence 1. No timeout or output loss
was observed; the candidate source remained unchanged. RUBY and MUSE independently
reconciled the failed result and exact cleanup. MUSE approved only the four emitted
formatting hunks, applied manually after source-identity readback. Corrected source
r3 is frozen with source manifest
`0af5451dc68132ad05200d8e72a28ac95ecef01add1e64c020d5b69917e97458`
and review manifest `52591cd305ad953e4742df7d2c2d2637e2a832f175d4c6b670d6443d77753e4f`.
RUBY and MUSE confirmed the four exact semantic-neutral hunks and unchanged other
sources and test plan; static acceptance is not executed verification. Fresh validation r2
is frozen with driver manifest
`d0202135e64592a4e27168eabe0afaadef11cc5e8fa1dc6c3bad3651f34d182f`
and payload `59844f3d3f706aa96fd7bbb02569b0dafc802ce52f7050578e15d82a6d6bcace`.
All five faults, tests, decoder, containment and caps are retained. The temporary
review hold closed after RUBY explicitly accepted the exact driver-r2 static delta.
MUSE reconciled that verdict and cleared one new contained nonroot offline attempt.
Fresh launch readback matched the manifests, source, metadata, Node and local
CLI/socket/runtime provenance. That attempt failed: outer session 1105/exit 1,
candidate canonical verification passed formatting/check/Clippy, then returned 101
with exactly `transfer_preparation_mechanics_refuse_roles_and_access` failed at
the channel-retirement POLLERR assertion (inputs line 806). All five controls
remain not_run. Failed-result manifest
`9e7ee42a7b041bc6ca4bda5091d4b6fa9bbb1fac84d2984364e0624771a8c77f`
retains 41 entries, eleven command/twenty-two stream records, source consistency
and exact same-daemon removal 0/literal absence 1 for the owned container.
No timeout, output loss or signal was observed. RUBY and MUSE independently
reconciled the failed phase, raw evidence and exact cleanup, not validation success.
Both flagged a P2 in the test oracle: immediate pipe EOF/POLLERR assumes process-global
exclusivity, while sibling fork fixtures can inherit those descriptors before
their own cleanup. The source establishes an interference path, not the cause of
this occurrence or absence of a helper leak; the raw failure lacks a fault index.
The proposed fix preserves identity-qualified owner-local observations and pipe
retirement assertions in isolated exact single-test subprocesses with bounded
supervision/retirement and explicit fault diagnostics. MUSE approved preparation
only of a concrete proposal. `principal-input-isolation-proposal-r1.md` is frozen
at `20a40dc78d68d6350f7a56453fd082913788cde46954dda90daa782d9b1dc88c`:
exact single-test routing before fixtures, existing pidfd/WNOWAIT child cleanup,
owned-group supervision, bounded regular-file output rather than capture-pipe EOF,
and distinct leaf assertion versus supervisor failure evidence. RUBY held proposal
r1 for another P2: std::Command pre_exec can block on an internal fork/exec
handshake before returning the child handle, outside the proposed supervisor.
Revised proposal r2 is frozen at
`27a5086449604694247d9948bc4cec37b5ac4a5587b8975c5341ee39406a73ed`:
prebuilt literal raw-fork/exec leaf launch, immediate exact-PID ownership, existing
pidfd/WNOWAIT observer and independent session/group verification before group
retirement. No hidden exec-error handshake or detached launch thread is proposed.
RUBY and MUSE accepted the corrected launch design at proposal level, but hold r2
for a second P2: its closed child environment omitted assigned HOME/TMPDIR, leaving
tempfile creation outside the explicit writable scratch assignment. Proposal r3
is frozen at `18e6d34b09e87683b7df3c972e290c65d00f4fa140d8bbd6a49a4a776694caf1`:
private per-leaf 0700 root/home/tmp under the canonical driver's assigned scratch,
explicit closed HOME/TMPDIR/routing fields and no-link/path/type/owner validation
before fixtures, with owned tree retention through cleanup and output reconciliation.
Native CI/other-host portability is not inferred from this offline scratch gate.
All current assertions, five fault controls, output/cleanup bounds and truthful
failure classifications remain required. RUBY accepted r3 at proposal level, but
MUSE holds source staging for a P2: requiring explicit parent TMPDIR changes the
existing canonical native test prerequisites. Preparation-only proposal r4 is
frozen at `fe904d697df9ca5a9fab6776c10f682f40bcb4b84aaba640b48272d3383ece61`:
ordinary parent tempfile selection is preserved;
its owned root is canonicalized, verified and recorded before fork, with exact
private HOME/TMPDIR passed to the leaf and no leaf fallback. The offline driver
still supplies /scratch/tmp and independently reconciles each actual root beneath
that assigned write plane. No new test/task flag or environment prerequisite,
launch/clock/cleanup/control change or native-CI success is implied. RUBY accepted
the exact r4 delta with no findings; its hash and verdict were returned to MUSE.
The source packet must preserve descriptor/path-object linkage, component-wise
canonical ancestry rather than string prefixes, and all post-fork error cleanup.
MUSE independently accepted r4 and granted narrow source staging only for the
cfg(test) six-leaf isolation supervisor/wrappers, diagnostics, connected record
decoder/refusal fixtures and private docs/handoff. Source r1 is now frozen:
all six wrappers precede fixture/channel creation, the fixed raw-fork supervisor
reuses pidfd/WNOWAIT cleanup, and routing/output/retirement failures are distinct
from intended semantic assertions. Original production helpers and clock semantics
remain unchanged. Source manifest
`7f19776a35fcf360a160bc505cc2a46c61ade047509f69bfd25fce8947594874`
and review manifest `06adb8c4c8257085eb0926ee88d1dc7eec1ba3ae577fb5198f42656167529d40`
retain the exact sources, deltas and concrete refusal/decoder plan. RUBY's nonauthor
static review holds r1 for three P2s: default tempfile root mode does not guarantee
the required 0700; generic refusal assertions can accept the wrong failure cause;
and semantic stage labels remain active over unrelated byte/access/channel/foreign
I/O assertions. MUSE independently challenged the refusal and stage cases and
granted only narrow source corrections. The staged revision uses atomic Builder
permissions 0700 with unchanged native location selection; each reserved refusal
requires its actual cause, exit, counts and route-specific marker/harness evidence,
including the original timeout observation. Stage transitions precede unrelated
byte/access/channel/foreign-I/O assertions. Synthetic decoder cross-cases reject
unrelated refusal and stage substitutions. Corrected source r2 is frozen with
source manifest `7f1cae539e039eb907694016a26f1021a6fb2f524d6cf8bbcff6e67828931a13`
and review manifest `88c1bba2adf3d08e055191c391708df1a26900c4e95e3ef8c5c69920d32bcc5e`.
RUBY independently verified all four source/fourteen review entries and the exact
corrective deltas, with no remaining P1/P2/P3 in that scope. All three source-r1
blockers are resolved statically; production helpers, original assertions and
controls remain unchanged. Fresh live/frozen source readback agrees. MUSE reconciled
the exact identities, corrective deltas and RUBY verdict, then accepted source r2
for the private static-source checkpoint only. Formatting/compiler/lint/tests stay
not_run; the frozen r1 stays immutable and held. MUSE granted preparation only of
a separately frozen nonroot offline driver/payload: canonical candidate, unchanged
five semantic mutants and all new causal-refusal/decoder controls. Its independent
decoder must bind raw parent/leaf names, counts, exits, stages, bytes, hashes,
actual refusal evidence and cleanup, with raw Unix path decoding and component-wise
scratch ancestry. Existing containment/time/output/one-attempt bounds are unchanged.
Driver preparation r1 remains frozen and not_run after an author check caught an
ambiguous completed-leaf count for failed-control lanes. Successor isolation
validation r2 is frozen: driver manifest
`4d0707668e8d85de84defd5b5330eacbf2452126b27137f2af3dda4466a97880`,
payload manifest `3d57b775b1cef90cb37c4376e252b7cf4eb0f6dd3ad4539d02f5d14416e0bd93`.
Static readback reconciles 726 source files, 863 metadata paths and fifteen
executables. Candidate overlays match accepted source r2; all five mutation
production prefixes match the earlier reviewed packet, and all tests remain
identical across variants. Independent decoding and prepared synthetic refusal/
substitution controls are not_run. Result counts distinguish six isolated records
from six passing candidate leaves or five passing/one assertion-failing mutant
leaves. Existing runtime file/socket tuple agrees by filesystem readback only;
no daemon/image query occurred. MUSE's initial static review found no issues but
explicitly withheld execution pending RUBY's final verdict. RUBY's completed
independent review holds r2 for a P2: its independent decoder requires empty stderr
even for intentionally failing --nocapture leaves, where Rust panic diagnostics
are expected. Empty-stderr synthetic failure samples hide that mismatch. No other
scoped finding was reported. The exact verdict was returned to MUSE; narrow
successor preparation was requested to keep bounded complete stderr evidence
while restricting the empty-stderr rule to successful leaves and adding realistic
failure-output fixtures. R2 stays frozen and not_run; no execution clearance exists.
MUSE granted preparation-only r3 correction: successful leaves retain the empty
stderr rule, while intended assertion leaves require bounded complete panic stderr
bound to the exact test name, source path and assertion content in addition to their
already required stage, normal 101, parent failure and cleanup. Stderr alone cannot
credit a control. R3 is now frozen: driver manifest
`720c66882892bef7a17dbae3bb2e034ab6c1ef25fa0ff64c06636ff7ef681461`, payload
manifest `4c4202db0ca2886b99bd1a73083c774e1c99e31097718de8bc611a311866648b`.
Static rebinding reconciles 726 source files, 863 metadata paths and fifteen
executables. It has representative nonempty panic fixtures for all five controls
and negative empty/wrong identity, source, length, hash, cap and refusal cases.
R2 remains frozen and not_run. RUBY's final r3 review holds the packet for a P2:
pinned Rust 1.95's default panic hook adds a parenthesized OS thread ID between the
exact test name and `panicked at`, but r3's representative samples and decoder omit
it. A P3 also leaves wrong-stage samples rejected by their missing stderr rather
than the stage mutation. No other scoped finding was reported. The exact repair
request is with MUSE: parse the pinned header without equating its test-thread TID
to the child process PID, preserve valid panic stderr while mutating stage and
recompute the associated raw evidence. R3 stays frozen and not_run. MUSE granted
narrow r4 preparation: parse the pinned Rust 1.95 header's escaped exact test name,
positive parenthesized test-thread TID and same-line source location, without
equating the TID to the child PID. All five fixtures and wrong-name/source cases use
that form; missing, zero and nonnumeric TIDs are negative cases. Wrong-stage
fixtures retain valid panic stderr while only their altered stdout evidence is
recomputed. No source/harness/mutation/runtime, containment, clock or budget change
is permitted. R4 is frozen: driver manifest
`65b706db1dbc6353582bf4084648c25495e4491dd3efb7973abafc6b750bf083`, payload
manifest `b42a8609d256fd255d9e50519384479392aef3ecdf42d4b72d438eae1bc5669c`.
Static rebinding retains 726 source files, 863 metadata paths and fifteen
executables. RUBY independently verified all seventeen driver and 726 payload
entries, accepted the r4 static driver with no P1/P2/P3, and confirmed the payload
differs from r3 only in the decoder hash; metadata/runtime provenance are identical.
MUSE subsequently found one coverage gap: r4 did not reject an otherwise-valid
exact-name/source/assertion panic header with only its parenthesized thread ID
omitted. R4 remains frozen and not_run. MUSE granted r5 preparation only to add
that single string to the existing per-control stderr-rejection loop, preserving
the existing valid stage, exit, challenge, output, cap and cleanup fixture path.
R5 is frozen with driver manifest
`c194a75943a1eef467c92ff2d65ceca227ded2fe688461b69d8314e759c19c9e` and
payload manifest `b934443ada14333ed9dadb5be2671ac9b6d5efc44ef7e744622dda59411aa054`.
RUBY independently verified all eighteen driver and 726 payload entries, found no
P1/P2/P3, and confirmed the added sample isolates the missing-TID field across all
five controls while the valid fixture evidence is retained. The payload differs
from r4 only in the decoder hash; metadata/runtime provenance are byte-identical.
MUSE then cleared one exact r5 attempt. Its fresh static identity preflight passed,
but strict Clippy stopped before any Rust isolation leaf or negative-control lane:
`principal_adapter_input_test_isolation.rs` discarded `TempDir::keep()`'s must-use
return under `-D warnings`. The driver consequently recorded candidate exit 101;
the decoder's canonical-exit refusal is consequential, not the first cause. The
owned container was removed and its literal absence recorded, but that cleanup does
not validate the candidate. Frozen results manifest
`4af6695ff43c0f541e50d444a508fa5af919c5fdcecd91915428f1f4e39e37a0`
covers 36 raw files; r5 is immutable failed evidence and its attempt is consumed.
RUBY also found a latent decoder mismatch: actual unchanged `#[path]` wiring emits
`src/bin/../principal_adapter_inputs.rs`, not the normalized spelling r5 expected.
MUSE granted preparation only for the one-line `let _ = temporary.keep();` source
repair and an r6 exact compiled-path matcher/samples that reject the normalized
spelling. No retry or execution clearance exists.
Isolation source r3 is frozen with source manifest
`1337e32e332b82742e82625f6fef8b592a0134bc36cbb354ab003802b30efff8`
and review manifest `cbf22057fcb400b09a8a11baa120daf5a925bde95ce60c2b570d418b5390f050`.
It changes only that cfg(test) must-use binding. R6 is separately frozen against
the r3 source identity: payload manifest
`63f9e13bae6147740b5c55075473e301a13ca0e5e0daccdf624049779b0bceba`,
driver manifest `8aa5da2dd34f5519aee603b5e51b5b4d3fc762304627be9f0c8caeccff39a8ab`.
All six lane copies bind the r3 isolation-source hash. R6 changes only the exact
compiled panic source spelling and adds the normalized-spelling rejection sample;
before its separate disposition it had no results directory and was not_run. RUBY found no P1/P2/P3 and MUSE
accepted source r3 and r6 statically. R6's driver manifest has 21 rows for 20
unique files because one identical source-delta entry is repeated; the verified
closure is unchanged and the frozen packet will not be rewritten. MUSE cleared one
fresh exact r6 offline nonroot attempt after a full parent-side readback. That
attempt is separately consumptive; no retry, commit, push, release, root, VM,
installed, hosted or provider action is authorized.
The r6 attempt is frozen failed evidence with result manifest
`982095378a17e73581ba09a41d80b0a722f8a04788033564412fea8e7e88205d` covering
36 raw files. The candidate lane passed: 411 parent passes, six isolated normal-zero
leaves with expected stages and empty stderr, and nine causal refusals. This is
bounded source-fixture evidence only. Roles-negative reached its intended slot-five
object assertion but also had two unrelated failures, so the decoder correctly
rejected its parent-failure set; UID/GID/clock/retirement lanes did not run. The
owned r6 container was removed and literal absence recorded, which proves cleanup
for this attempt but not full validation. Both attempts are consumed. RUBY found
two source-level P2s requiring a separately authorized decision: an isolation
supervisor group/session observation order race for fast exits, and an existing
launch-EOF versus terminal-exit precedence race in the nonroot preexec path.
MUSE separately approved source preparation only for the cfg(test) isolation
repair: after the exact pidfd-bound WNOWAIT terminal observation, freshly read
that still-owned child's process group and session before accepting terminal
ownership. Previously observed ownership remains available for error retirement;
missing fresh proof still refuses. The staged helper and two regressions use an
explicit pipe release to order a pre-setsid observation before real fast success,
launch refusal, missing-marker exit, and a child that never creates a session.
No sleeps, retry-to-hit-race, deadline extension or assertion relaxation is added.
The source-only r4 packet is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-source-r4`,
source manifest `f3bc577b7c01354c0e69ce2ecfe882b8cf165e9847f528131cb61b8e387a2dd1`
(four entries) and review manifest
`3b31b9dbb2257dff1c016c420a40b8de6fdfd5bc24acfe1d4cb4fca5cf6fb1a7`
(nine entries). The shared collector and both adjacent adapter source files are
byte-identical to r3. RUBY held r4 for one P2: its fast-success fixture compared
setsid's return with fork's child-local zero rather than the actual child PID.
The live source now compares with getpid; expected exit zero and the fixture's
ordering/assertions remain unchanged. The corrected frozen source-only successor
is `/Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-source-r5`,
source manifest `f24301567f16a453bf95d4d965d063b5f675b1d413aa8c79bcfc5739148bdf1b`
(four entries), review manifest
`07b79c051755a525fe20c7cf7ec64f355ed673a2352bc2bb92a027718c869423`
(ten entries). RUBY independently verified every row and the exact one-line
r4-to-r5 source delta, with no remaining scoped P1/P2/P3. MUSE independently
verified all r5 manifest rows and accepted that isolated source statically.
No execution evidence exists for either revision.
MUSE separately approved shared-collector source preparation with callback/action
count and private-snapshot qualifications. The staged shared loop requires exact
retained READY, observed length one and actual EOF before request handoff or
reconciliation. A terminal child gets a fresh bounded launch read if launch is
pending; held writers remain pending under the original clock. Bad terminal exit
classifies only after complete launch. Private launch observed/retained lengths,
retained byte and EOF are snapshotted at both capture owners before cleanup;
projected adapter fixtures retain unknown launch facts. Callback pre/post clock
checks preserve earlier callback refusals and honestly retained action counts.
Deterministic real-channel/owned-child regressions use a cfg(test) scheduling seam
on the same loop; no fabricated EOF/exit, sleeps, retries or serialization is added.
The frozen combined packet is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-launch-precedence-source-r1`,
source manifest `a9ef3c826cb33bf33832c77680b66ec1d17421af10fa3d27e2ff23c8371833ad`
(four entries), review manifest
`9d7b72fcf9f4383e214ea8f04bf3d0dbb615dd1eebcb77a5a9d313c4f2c63755`
(17 entries). Inputs and accepted isolation source r5 are byte-identical; adapter
delta only adds unknown launch facts to a projected test literal. Seven new shared
tests and the strengthened existing nonroot control are staged, not executed.
VLAD's independent review and MUSE held r1 for one P2 in the EOF-first fixture:
constructor return followed pidfd acquisition without observing child descriptor
closure, so a legal parent-first schedule could read EAGAIN and select Input.
No scoped production P1 or other P2/P3 was found; this is static review, not a
runtime result. The narrowly approved correction adds a dedicated child-prepared
acknowledgement after inherited writer closure. The parent verifies its exact byte
under the unchanged original fixture deadline before returning, while the child
still waits for the separate release. Acknowledgement failure retains owned-child
cleanup. No production precedence or expected Launch assertion is changed.
r1 remains immutable HOLD. The frozen successor is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-launch-precedence-source-r2`,
source manifest `cbc4c9a77ddcfdff7d2d095b2aef5a097b2a3df14bb690e639f481306fb22dcc`
(four entries), review manifest
`621f11205ab3b2483d6bfd51fa74efaf4548361f869447829a1ca31ac88f670f`
(18 entries). Only the test constructor changes; the other three source files,
private lifecycle docs, Unreleased, Cargo and main match r1 exactly. MUSE has r2
for focused independent recheck; neither revision has execution evidence.
VLAD and MUSE closed r2's own-child ordering P2 statically, but held r2 for the
residual parallel-fork P2 also raised by SCOOBY: ACK does not establish absence
of a writer inherited by a sibling test fork. No such interleaving is claimed
as observed in r6. The separately approved narrow correction now waits for
actual empty STATUS EOF using a distinct preparation Capture/read/poll under
the same original deadline, only in the EOF-before-exit case, while its child
remains release-blocked. Production launch Capture stays unpopulated and its
unchanged hook must independently read real EOF before release/normal7/Launch.
Assertions, production policy and exact owned-child cleanup are unchanged.
r1/r2 remain frozen HOLD. The successor is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-launch-precedence-source-r3`,
source manifest `d766d7d6abf073feb9a49bbcb44df428dc91ebc3601a014bff66af36b9e46355`
(four entries), review manifest
`589fe9093256a436e18b11a7c8a5213fd77ae8023d1289345a5301124aba515e`
(18 entries). Only order0 test preparation changes from r2; other source/docs,
Unreleased/Cargo/main match r2 exactly. VLAD's focused independent review found
no remaining P1/P2/P3 and closed the sibling-writer P2 statically. MUSE verified
all four source/18 review rows and the exact delta, then accepted frozen r3
statically. This is not formatter/compiler/test or runtime success.
At source acceptance, no formatter/compiler/test/driver or other target execution
was granted; new tests were not_run. MUSE separately cleared parent-only preparation of a frozen offline
driver/payload successor derived narrowly from reviewed validation r6, bound to
accepted shared-source-r3 and isolation-source-r5. Strict decoder/mutant failure
sets, all nine causal refusals, clocks/caps, offline containment/runtime/resources
and raw capture/cleanup reconciliation must remain unchanged. Preparation did not
clear invocation or self-test. The frozen successor is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-validation-r7`,
driver manifest `dec399c97fa216065f2d722ace1af2b63cbca8b87ff6258815b94f26163b7bf8`
(25 unique entries), payload manifest
`2c9033ae51de9c1516acee050265beaa1eabb03c5f4dc3611577bc7a7d362c74`
(726 entries). Trusted parent hash readback verifies all rows: exactly 36 accepted
source/private-doc/Unreleased overlays across six lanes; the other 690 payload
rows, all five input mutants and strict decoder remain unchanged from r6.
Driver delta is only fresh name/label and derived payload identity; inner driver
delta is only payload identity. At preparation freeze, no results directory
existed and r7 was prepared, not_run; its consumed result is recorded below.
This private, optional, uninstalled/unconnected amendment requires no Protocol,
pin, public CLI/spec/schema/JSON, Site/Skills/Examples/Learn/FAQ/Glossary propagation.
SCOOBY sent the bounded one-attempt proposal to MUSE. VLAD's independent read-only
driver review (turn `01a11c86-6137-7613-922d-354b19c2c68b`) found no P1/P2/P3;
MUSE independently verified the frozen identities, deltas, permissions and absent
results. MUSE then separately cleared exactly ONE offline nonroot Linux/arm64
attempt of this unchanged r7 driver under Bobai's bounded validation delegation.
The grant includes only its fixed preflights, decoder self-test, six canonical
agent-admitted verify lanes, raw capture and bounded exact-owned cleanup/recovery.
Run from r7 with `env -i PATH=/usr/bin:/bin LANG=C LC_ALL=C
/opt/homebrew/opt/node@24/bin/node /Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-validation-r7/run.mjs`.
Preserve all reviewed image/CLI/socket/daemon/runtime/resource/clock/output pins;
stop on drift rather than refreshing pins or substituting runtime. Any preflight,
decoder, build or test failure consumes the attempt. No independent exploratory
target query, driver/payload edit, limit enlargement, admission bypass or second
attempt is granted. Next: reconcile frozen raw results and request independent
RESULT review; `summary.accepted` alone is insufficient. Step7 remains OPEN;
Step8/V12.2 inactive. This does not clear root/VM/installed/hosted/provider/network,
commit/push/release or broader authority activation.
The one r7 attempt is now consumed FAILED. Retained raw results are frozen under
the packet's `results/`; `results.manifest.sha256` is
`c329b7984c19a0ac580c8aa774f2f3032fdfe585d0ded4aa91a8feffc36d6950`
(36 files, 11 command records). Candidate `ota run verify --agent --native`
failed at `cargo fmt --check`: six unique formatting hunks printed twice (12 raw
projections), all in the accepted shared
`principal_observer_process.rs` source. Canonical candidate exit 1 was correctly
refused by the unchanged decoder (`ValueError: canonical exit`). Decoder synthetic
self-test passed and `SOURCE_CONSISTENT candidate` records completion of the
reviewed script's before/after checks. Failed-run scratch manifests were not
exported, so independent retained scratch lineage is not claimed. Compilation/check,
Clippy, Rust tests, isolated leaves, causal refusals and all five mutant lanes
were not_run. Verify retained stdout 16,144 bytes and stderr 607 bytes; all raw
command stream hashes/counts reconcile without timeout/overflow/signal/failure.
Attempt `e84693c2-2cde-483f-abe1-32b012c46d7a` owned container
`b5146a72a664e0c14e0fedf898795b661a651183d07668c53b064c7f38970e2a`:
creation/configuration/cleanup identity, image and labels reconcile; exact-ID
removal succeeded and same-daemon inspect returned literal
`error: no such object: <exact-ID>` with exit 1 and `[]` stdout. Cleanup is proved,
not candidate acceptance or Step7 closure. No automatic retry is authorized.
VLAD independently reconciled failed r7 (turn `01a11c94-d17e-7c93-a8fe-8df0ae04bc82`)
and MUSE independently verified all result/driver/payload rows and exact cleanup.
r7 is accepted only as reconciled FAILED evidence, not candidate acceptance.
MUSE separately cleared manual formatting-only repair of the six retained hunks
in working `principal_observer_process.rs`, after exact accepted-r3 baseline
equality. Only prescribed reflow/indentation/trailing commas are permitted; all
literals, conditions, assertions, callback/release/EOF/exit/cleanup/deadline behavior
and mutant/decoder predicates stay unchanged. Trusted reads/hashes/local metadata
and diff check only: no formatter/compiler/test/decoder/driver/Docker invocation,
validation-driver preparation or new attempt is cleared. Next: freeze the source
successor and obtain independent static review. Do not edit or rerun consumed
r5/r6/r7 or accepted r3; do not relax fmt, canonical tasks or decoder predicates.
No Protocol/pin/spec/JSON/Site/Skills/Examples/Learn/FAQ/Glossary propagation is
required for this private formatting-only repair. The frozen source successor is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-launch-precedence-source-r4`,
source manifest `ae4cf2092b8068c2bfbbd1805c243c2d69a6cf77c05d594b01ce5d1862a671db`
(four rows), review manifest
`de6ff5a89025c838ba399fc65c76b793cffdd4c39b939c4723e605d8d3096ee0`
(15 rows). Trusted parent readback reconstructs exactly the six retained raw
before/after hunks and verifies whole-file equality to the working result. All
other source and connected files match accepted r3. No formatter or target ran;
diff checks pass. SCOOBY returns frozen r4 to MUSE for independent static review.
VLAD's independent r4 review (turn `01a11c9b-c691-70c1-bc58-542a67ea33f6`) found
no P1/P2 or source-semantic issue; MUSE verified the six exact substitutions and
all source/review identities, then accepted r4 STATICALLY ONLY. One P3 wording
correction: r7 did run `rustc --version`; compilation/check, Clippy and tests were
not reached. Frozen r4/r7 retain their original wording; carry the precise
correction in the next packet, without rewriting accepted or consumed evidence.
MUSE separately cleared parent-only preparation of a fresh r8 validation successor
from frozen r7, staging only accepted r4 process formatting across all six lanes,
connected handoff wording, fresh names/labels and derived identities/manifests.
Candidate inputs, five mutants, decoder/self-test bodies, nine causal refusals,
canonical task paths, cleanup and all runtime/resource/time/output pins stay
unchanged. No formatter/compiler/version probe/test/decoder/driver/Docker/runtime
query or invocation is cleared. Do not inherit results or any execution grant.
The frozen r8 packet is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-validation-r8`,
driver manifest `879521cac4dc60efe10133dbbd0ce261668d35272d8e0fe1f2e076d6d3326cd5`
(26 unique rows), payload manifest
`a9d4280a7b8d0a2e59419eafdfbf25c54e3e4aa364ca50b2ab0fc914281b7229`
(726 rows). Trusted parent verifies every row and exactly six r4-formatted process
overlays; the other 720 payload rows, all input mutants and decoder are r7-identical.
All 863 permission readback entries match r7; driver changes only names/labels and
payload hash, inner only payload hash. No results were inherited or produced.
The new review request carries the version-probe/compilation and retained-scratch
wording corrections without modifying frozen r4/r7. Preparation only, not_run.
VLAD's independent r8 DRIVER review (turn `01a11cbd-36da-75a0-87bf-ed3b0b73b16a`)
found no P1/P2/P3. MUSE independently verified the frozen source/payload/driver
identities, exact deltas, permissions and absent results, then separately cleared
exactly ONE unchanged r8 offline nonroot Linux/arm64 attempt under Bobai's bounded
roadmap delegation. Recheck frozen hashes, permissions and fresh result location
before invocation; stop on drift. The grant permits only this driver's fixed
preflights, container operations, decoder self-test, six canonical agent-admitted
verify lanes, source exports and exact-owned cleanup/literal same-daemon absence.
All pinned identities, containment and clocks/resources/output limits stay fixed.
Any preflight/self-test/fmt/build/test/decoder/cleanup failure consumes this grant;
no extra target query, automatic retry, substitution, edit or admission bypass.
Entry from r8: `env -i PATH=/usr/bin:/bin LANG=C LC_ALL=C
/opt/homebrew/opt/node@24/bin/node /Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-validation-r8/run.mjs`.
Next: freeze raw results and reconcile all controls/source/containment/cleanup for
independent RESULT review; `summary.accepted` alone is insufficient. Step7 OPEN,
Step8/V12.2 inactive; no root/VM/installed/hosted/provider/network, positive authority
or wiring, commit/push/release or broader activation is granted.
The one r8 attempt is consumed FAILED. Frozen raw results at packet `results/`
have manifest `fe09b6ef9d866e525c0d39bec764da1d84a8c01c87791abf0ace0cb96221825e`
(36 files, 11 command records). Candidate formatting passed; all-target/all-feature
compilation/check reached a test-target error: the supervisor `Exchange` literal
at `src/principal_observer_supervisor.rs:427` cannot construct the type with its
private cfg(test) `after_launch_read` field missing. Candidate canonical exit101
was correctly refused by the strict decoder; outer verify exit1, stdout12,264
bytes/stderr607. Clippy, tests/leaves/causal-refusal cases and five mutants were
not_run. Decoder synthetic self-test passed; `SOURCE_CONSISTENT candidate` records
script check completion, not independently retained scratch lineage (not exported).
All 11 command stream hashes/counts reconcile without timeout/overflow/signal/
spawn failure. Attempt `7dd999b4-299c-43a7-bffa-895c4fc1b570` owned container
`51d7db9a0f89ae9b4f3aa84f6184bad34db010f85a2d9c716e8c264ec7a495b7`:
retained configuration/cleanup exact-ID/image/name/labels and daemon agree;
exact-ID removal and literal same-daemon absence are supported by raw evidence.
This is failed validation with cleanup, not Step7 closure. No retry is cleared.
VLAD independently reviewed failed r8 (turn `01a11cc4-10a0-76a3-893d-6c4c11b0279e`);
MUSE reconciled all frozen raw/driver/payload records and exact cleanup. No added
P1/P3; r8 is accepted only as reconciled CONSUMED FAILED evidence. No retained
failed scratch lineage is claimed. MUSE separately cleared only the two-site
cfg(test) repair after exact baseline equality: make `after_launch_read` field
`pub(super)` while retaining its cfg(test), and initialize it to None in the
existing supervisor test-module fixture. Alias visibility, production initializers,
hook/call order/defaults/assertions/release/EOF/exit/deadline/cleanup/control/mutant/
decoder behavior stay unchanged. No new constructor/helper/public surface.
Next: freeze the prior four shared files plus connected supervisor and exact
two-site hashes/diffs/constructor inventory for independent STATIC review.
Trusted parent reads/hashes/local metadata, specified manual edits and diff check
only: no formatter/compiler/versionprobe/Clippy/test/decoder/driver/Docker/runtime
query, validation-driver preparation or retry granted. Compilation correctness
remains unproved. Preserve consumed r8 and accepted r3/r4. First-party public
surfaces are unaffected by this private cfg(test)-only repair with no production
layout/behavior change. No root/VM/provider/installed/hosted/network/commit/push/
release; Step7 OPEN, Step8/V12.2 inactive.
The frozen repair successor is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-launch-precedence-source-r5`,
source manifest `a49fb0a4858961dd9676aa2b4e62d2ac02742eef5534adf8701d74e23f0c0275`
(five files including connected supervisor), review manifest
`921175a7a85eba6741b701a01b0749a88931b6844d9334759f6271bf575e2cb1`
(18 rows). Whole-file reconstruction verifies exactly the two granted substitutions;
five connected Exchange constructors are retained in the inventory. Other source,
Cargo/main/private docs/Unreleased remain unchanged. Frozen manifest rows verify
and both repo diff checks pass. No formatter/compiler/versionprobe/test/driver or
runtime query was invoked for this source repair. SCOOBY returns r5 to MUSE for
independent static review. VLAD's r5 review (turn
`01a11cca-0553-7001-99ae-4602a855dcf5`) found no P1/P2/P3; MUSE verified the
exact two-site reconstruction, five connected constructors, cfg boundaries and
source/review identities, then accepted r5 STATICALLY ONLY. Formatting/compilation/
tests of this repaired source remain unproved. MUSE separately cleared parent-only
preparation of one fresh r9 validation proposal from frozen consumed r8: overlay
only accepted r5 process/supervisor in all six lane trees (12 changed rows, 714
unchanged); do not overwrite the mutant inputs. Keep all decoder/control/task/
dependency/toolchain/runtime/clock/resource/capture/source-check/cleanup semantics
unchanged. Only fresh names/labels and derived identities/readbacks/manifests/review
request may change. No results or attempt clearance is inherited. No formatter/
compiler/versionprobe/test/decoder/driver/Docker/runtime query or invocation is
granted. Next: freeze r9 for independent STATIC DRIVER review and separate MUSE
execution disposition. Stop on drift/extra changes; no automatic retry.
The fresh frozen proposal is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-validation-r9`,
driver manifest `ac544cad8e70cabf35ac645fe62ec3abd4aba08d5f18460909ae203e39de6bf6`
(26 unique rows), payload manifest
`3680c652971c28e44a06d3e18e21e352765f58fbe8a4d3ea2e016e5a9a7a72b9`
(726 rows). Trusted parent verifies every row, exactly 12 accepted process/
supervisor overlays and 714 unchanged payload rows. All five input mutants,
decoder and 863 permission readback entries remain r8-identical. Driver delta is
only fresh names/labels and derived payload hash; inner only derived hash. No
results exist or were inherited: PREPARED, not_run. SCOOBY sends r9 for independent
STATIC DRIVER review. VLAD's correlated r9 review (turn
`01a11cd4-18a2-7b81-a261-4c3fb73397bc`) found no P1/P2/P3; MUSE independently
verified the identities, 12 overlays/714 unchanged rows, all permission/path
entries and unchanged control/runtime boundaries. Packet root0700/payload root0755
still permit trusted-host owner writes; no host-root tamper-resistance is claimed.
MUSE then separately cleared ONE unchanged r9 offline nonroot Linux/arm64 attempt
under Bobai's bounded active-roadmap delegation. First recheck all frozen driver/
payload hashes, permissions and absent results; stop on drift. Only the frozen
driver's fixed preflights/container operations/decoder self-test/six canonical
agent-admitted verify lanes/source exports/exact-owned cleanup are permitted.
All pins/containment/CPU/disk/output/operation/retirement limits stay fixed. Any
failure consumes this clearance; no exploratory query, automatic retry, fallback,
edit, limit enlargement, pin/runtime substitution or admission bypass.
Entry from r9: `env -i PATH=/usr/bin:/bin LANG=C LC_ALL=C
/opt/homebrew/opt/node@24/bin/node /Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-validation-r9/run.mjs`.
Next: freeze all raw results and return exact controls/source/containment/cleanup
reconciliation for independent RESULT review. This clears no root/VM/provider/
installed/hosted/network, positive authority/wiring, commit/push/release or Step7
closure. Step7 OPEN; Step8/V12.2 inactive.
The r9 one-attempt clearance is consumed FAILED. Frozen results manifest
`de7ee01877c5f725821919cdf2383349d491fee771c13b70e1cc06f38c26188b`
(36 files/11 command records). Candidate formatting, all-target/all-feature check
and strict Clippy passed; Rust tests reached one failure in
`principal_observer_process::tests::completed_launch_preserves_bad_terminal_after_real_pending_read`
at process line1964: actual `Err(Input)`, expected `Err(ChildExit)`. The observer
test target reports 73 passed/1 failed; other completed target summaries are green.
Six isolated result records and nine causal-refusal records were emitted, but
full strict candidate/mutant acceptance is not established. Candidate exit101 was
correctly refused by the unchanged decoder; no mutant lanes ran. Decoder synthetic
self-test passed. SOURCE_CONSISTENT is reviewed-script completion only; failed
scratch lineage was not exported. Verify outerexit1/stdout1,193,558/stderr607;
all 11 raw stream hashes/counts reconcile without timeout/overflow/signal/spawn
failure. Attempt `820c9b34-c333-4817-9825-c77e70f81624` exact container
`a03da4746c1920f9843af69425f8147bdb49fdd57aefdfd8442758561225c3ab`
has matching retained config/cleanup ownership labels/image/name and same daemon;
exact-ID removal/literal same-daemon absence reconcile. Cleanup is separately
supported, not validation acceptance. No retry/repair is automatically authorized.
VLAD's correlated independent result review (turn
`01a11cda-ce9c-7860-a6b5-6e27c62852bd`) and MUSE reconcile r9 as FAILED/clearance
consumed, not validation acceptance. The static P2 is the recurring cfg(test)
after-launch-read arrangement: it may return Input on a later legitimate pending
iteration. The raw failure does not retain exact loop/writer-origin tracing; no
production defect is established by this alone. Hook exit observation uses
WNOWAIT, not reap; cleanup owns reap. Full driver manifest is
`ac544cad8e70cabf35ac645fe62ec3abd4aba08d5f18460909ae203e39de6bf6`;
compact message transcription must not omit its `09`.
MUSE separately cleared source-only repair after exact five-file r5 baseline
equality: only cfg(test) hook invocation changes to Option::take/local mut hook;
add a consumed-hook assertion in the existing completed-launch regression.
Preserve alias FnMut, hook body/order/first-read guards, ChildExit, actual READY/
EOF/exit, zero-callback, transport/original-deadline/cleanup assertions, production
source/constructors/task/decoder/causal-controls/mutants/dependencies/runtime pins.
Next: freeze source-r6 with exact baseline/diff/manifests and all-three-hook/
constructor/cfg readbacks for independent STATIC review. Trusted reads/hashes/
diff/metadata and granted manual edits only; no formatter/compiler/versionprobe/
import/test/Docker/runtime query, driver preparation/invocation or retry cleared.
Accepted source and consumed r9 stay untouched. No positive authority/wiring,
root/installed/hosted/provider/network/commit/push/release or Step7 closure;
Step7 OPEN, Step8/V12.2 inactive. Public first-party surfaces remain unaffected
by this private cfg(test)-only seam repair.
The frozen source successor is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-launch-precedence-source-r6`,
source manifest `8af18664cfd7594f8f6123cbd5ab8a2aaad909d2a809f1a0f98be106eb4b2819`
(five rows), review manifest
`cf38622fa93afa8e97ec5722697173e450f4613645ba13853e13fbecea4c08f9`
(16 rows). Exact baseline and whole-file reconstruction verify only the two
granted substitutions. All three hook bodies/first-read guards and five constructor
defaults match r5; adjacent source/Cargo/main/private docs/Unreleased unchanged.
Frozen rows readback-verify and diff checks pass; no formatter/compiler/versionprobe/
test/runtime target ran for source repair. SCOOBY returns r6 for independent
STATIC source review. VLAD's independent review (turn
`01a11ce2-83b4-74a3-9875-ffdd4aba3232`) found no P1/P2/P3; MUSE independently
verified the exact two substitutions, all hook/constructor/cfg boundaries and
source/review identities, then accepted r6 STATICALLY ONLY. No dynamic timing or
writer-origin trace is established. MUSE separately cleared parent-only preparation
of fresh validation-r10 from consumed frozen r9, overlaying only accepted r6
process in six lane trees (six changed rows/720 unchanged). Supervisor and all
candidate/mutant input/control/decoder/task/dependency/toolchain/runtime/capture/
clock/resource/cleanup semantics stay unchanged. Fresh names/labels and derived
identities/manifests/readbacks/review request only; no results/attempt inheritance.
Retain trusted-host owner-writable root and readonly payload/mount boundary;
no host-root tamper-resistance claim. Trusted parent reads/copies/hashes/diffs/
metadata/freeze only; no formatter/compiler/versionprobe/import/test/decoder/
selftest/driver/Docker/runtime query or invocation. Stop on drift/extra delta;
no pin refresh, substitution, limit enlargement or retry. Next: freeze r10 for
non-author independent STATIC DRIVER review and separate execution disposition.
The fresh frozen proposal is
`/Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-validation-r10`,
driver manifest `ba2d2f466ff002f3860ec4f3eb0dd8b05122a29da01b6b9cc61adb3decd68ae7`
(26 unique rows), payload manifest
`87db77b9f90166fcaba5e1903758663e881fae9c60e61bce450645a9dfded357`
(726 rows). Trusted parent readback verifies all rows, exactly six accepted process
overlays/720 unchanged rows, candidate five-file equality and untouched mutant
inputs. Decoder and all 863 permission entries match r9; driver differs only by
names/labels and payload identity, inner only by identity. No results inherited
or produced: prepared, not_run. Root owner-write boundary is explicit in the new
review request. SCOOBY sends r10 for non-author independent STATIC DRIVER review;
no target invocation, retry or execution clearance follows from preparation.
VLAD's independent r10 DRIVER review (turn
`01a11ce7-bf5f-7443-b793-924de186982c`) found no P1/P2/P3. MUSE independently
verified all hashes/paths/permissions, six accepted overlays/720 unchanged rows,
control/task/capture/source-export/cleanup equality and pinned local runtime/socket
metadata. Static readiness is not runtime success or host-root tamper resistance.
MUSE then separately cleared ONE unchanged r10 nonroot offline Linux/arm64 attempt
under Bobai's bounded active-roadmap/offline-validation delegation. Recheck exact
packet hashes/permissions, pinned runtime-file/socket metadata and absent results
immediately before invocation; stop on drift. Only the frozen driver's fixed
preflights/container operations/decoder self-test/six canonical agent-admitted
lanes/source exports/raw capture/exact-owned cleanup are cleared. All original
identity/containment/resource/clock/output limits remain fixed; any failure consumes
the clearance, with no extra query, retry, fallback, edit, pin/runtime refresh,
substitution, limit enlargement or task/admission bypass.
Entry from r10: `env -i PATH=/usr/bin:/bin LANG=C LC_ALL=C
/opt/homebrew/opt/node@24/bin/node /Users/bobai/.codex/artifacts/2026-10-08/principal-input-isolation-validation-r10/run.mjs`.
Complete raw results/source lineage and cleanup were frozen for independent
RESULT review; `summary.accepted` alone cannot close the gate. Step7 OPEN;
Step8/V12.2 inactive. No positive authority/wiring, root/VM/installed/hosted/provider/
production/network, commit/push/release or phase activation is granted.
The one r10 attempt completed successfully. MUSE accepted its bounded contained
Linux/arm64 source/synthetic evidence after independent raw reconciliation and
VLAD's non-author RESULT review (turn `01a11cf5-ee81-7ad1-8381-a3c385e814a9`),
with no P1/P2 result blockers. This acceptance is not Step 7 closure.
Frozen raw results manifest
`502310bd577b6513dc4f89161429f82fd76d54ffa12838a084c03ef99e3b6568`
(54 files/17 command records/34 streams). Candidate canonical exit0, six passed
isolated leaves, zero parent failures and 427 parent-pass records (not a claim
of 427 distinct tests). Each roles/UID/GID/clock/retirement mutant exits101 at
its sole intended assertion: five passed isolated leaves/one fixture assertion,
325 parent-pass records/one intended parent failure. Every lane emits six isolated
records and nine causal refusals; unchanged strict decoder/self-test accepted.
Candidate formatting/all-target-all-feature check/strict Clippy/test suite passed.
Verify exit0/stdout7,031,704 bytes/stderr114; all raw command and nested record
capture hashes/counts reconcile without timeout/overflow/signal/spawn failure.
Each retained BEFORE source manifest contains 77 files, all matching its exact
frozen lane payload including mutants. AFTER hashes/diffs were compared by the
reviewed script and SOURCE_CONSISTENT markers retained; AFTER manifests/diffs are
not independently exported. Twelve ignored `.idea/` metadata files per lane and
Git internals are outside these source exports; no repo-global mutation claim.
Attempt `1b62bd7e-cac8-48ed-ac30-03442e33f96e`, exact owned container
`607bd3441ebef6466ce50f43c24893d9c02899c46654c94c9d29e41ce0123f17`:
retained identity/image/name/labels, no-network nonroot containment/limits/readonly
mounts and reviewed daemon reconcile. Exact-ID removal0 and same-daemon literal
absence1/[] are supported by raw evidence, separately from candidate acceptance.
Next: propose the next canonical Step 7 source slice, with named problem,
dependencies, narrow scope, acceptance criteria, portability and independent
review plan, for MUSE's separate disposition. No successor implementation,
preparation, driver, attempt, validation, commit or release is cleared by this
result; no installed/root/hosted/provider/authority/wiring/production proof or
Step7 closure. Step7 OPEN; Step8/V12.2 inactive. Consumed failed r5-r9 remain intact.
Contained candidate compilation, strict Clippy and tests succeeded; full positive
admission/wiring, root/installed/hosted/provider/production behavior and the other
named limits remain not_proved. No further tests/build/driver/image/root inspection,
release/exec connection or installed/hosted/provider activation is authorized by
this result acceptance.
MUSE accepted the record-only r10 correction and separately cleared one DESIGN-ONLY
installed-closure/manager-transfer prerequisite slice. The existing Launcher
`docs/private-principal-adapter.md` now specifies the input/dependency frontier,
independent expectation producers, C/S/W/I ownership, same-object transfer,
refusal/lifetime/retirement and unchanged original clock/resource requirements.
Frozen review packet
`/Users/bobai/.codex/artifacts/2026-10-08/principal-installed-input-transfer-design-r1`,
manifest `08cf572032c334ece9fe4cb1ba8051c276caec59aeab1ac1cbd2dba4bb5eb9ad`
(eight files), design `e5fbc208fe4f89a3dbbafe952e9cbf2825cbeb8bca4f1de541fb578017a35576`.
Eight exact read-only source anchors and 15 author-captured scoped pre/post source/
Cargo/lock/changelog/observer-doc hash pairs agree; MUSE independently checked
current/AFTER agreement, not contemporaneous BEFORE state or repo-global non-change. The narrow
document diff's before side is derived by removing the new section, not a separately
captured pre-edit document. MUSE and non-author VLAD (turn
`01a11d08-f0f6-75c1-8692-29031e5fab8a`) accepted this as a prerequisite REQUIREMENTS
CONTRACT only, with no P1/P2/P3 findings, not executable/acquisition design or
implementation readiness. The frozen packet retains historical pending wording.
Exact installed transitive closure/private expectation binding, concrete manager
transfer/native relay acquisition and real archive/tree/closure/Listener budget
fit remain explicit refusal/open gates, not self-certified admission. No source,
fixture, Cargo/lock/pin/Protocol/CHANGELOG or public-surface changes; no build/test/
import, driver/payload preparation, container/root/VM/installed/provider/live action,
commit/push/release or phase activation. Normal adapter entries remain inert;
observer observation/IO paths remain uninstalled/unconnected and privileged
invocation separately gated. Step7 OPEN; Step8/V12.2 inactive.
Separate MUSE disposition precedes any successor work.
MUSE separately cleared one expectation-source DESIGN-ONLY increment before any
manager-route design or carrier wiring. The existing private document now compares
build-owned finite profile versus independently anchored protected sidecar, recommending
the former under existing administratively selected installed Launcher authority.
Independent artifact selection, private/native/runtime roles, installation/phase/
numeric expectation binding and mutable effective readback are specified as requirements;
actual artifact values/transitive closure, acquisition and real budget fit remain
refusal/open. Frozen candidate
`/Users/bobai/.codex/artifacts/2026-10-08/principal-expectation-source-design-r1`
has review manifest `49779cb93c1b52cc162ec4b419a9a66ed79887baef617bf910da8fae0bd7cfb2`
(nine rows) and design `106d991b9e645b31984fc722022d49543051da7f252e315812cc29d068780238`.
It retains the actual author-captured pre-edit private document, exact two-hunk diff,
six source-anchor files and 16 author-captured pre/post scoped hashes agreeing.
Independent VLAD review `01a11d41-0353-7db1-a431-c12bc158b3e9` and MUSE
disposition accept this binding recommendation only. Exact artifact selection,
acyclic complete loaded closure and C/S/W mapping, independent mutable readback
and same-object lifetime, effective namespace/MAC, manager transfer/retirement
and original-budget fit remain open. The frozen packet's pending-review wording
is historical; acceptance is not acquisition implementation or readiness.
No source/call-site/fixture/Cargo/lock/pin/Protocol/public changes or execution;
design clearance is not implementation or installed authority. Step7 OPEN;
Step8/V12.2 inactive.
The carrier design and bounded R7 synthetic
checkpoint alone did not authorize implementation or validation. Installed/manager/
native-relay acquisition remains separately gated.
Keep entry inert and do not remap/release/exec, invoke actual
kernels or connect admission. External controller/policy/readback, release gating,
installed/dynamic closure and production composition binding remain open.
The native contained composition carrier and independently acquired/
restricted installed input closure still require their own gates.
Those remain separate source gates; no caller NSS
fallback, root invocation or installed admission follows from this checkpoint.
Fresh exact-head macOS canonical CI remains required before merge/release;
observed normal-binary refusal, later paired commit/pin and clean-build gates
remain open. A new adapter target requires its own portability assessment; no
Stage B r4 macOS disposition transfers. Any production root invocation requires
independently reviewed owned outer resource,
descendant and cleanup containment first. Launcher source, optional Cargo target,
private documentation, Unreleased changelog and this handoff are affected.
Core consumers/pins, Protocol, dependencies/lock, public commands/spec/schema/JSON,
Site, Skills, Examples, Learn, FAQ and Glossary are unaffected: this is not an
installed or public feature. Paired reconciliation remains required before a
later commit/PR. No root invocation, VM, SSH mutation, host installation, signing,
registration/composition activation, provider action, commit or push was
authorized or performed. V12.1 Step 7 remains active/open; hosted Linux/X64 proof
and provider/registration gates remain open, Step 8 and V12.2 inactive. Older
historical evidence below retains only its original scope.

- branch: `1.6.29-implementation`
- released baseline: `v1.6.28`
- development package identity: `v1.6.28`; this branch contains unreleased post-release source
  changes and is not a new release or support claim.
- current proof gate (2026-10-06): obtain six successful exact-head source workflows for
  synchronized Core `1b868e1036493e0b7042dc88cbd375ce930316a6`; no previous-head gate transfers.
  A8 build/install and cleanup reconciliation was published at Core `c6e29c72` and remains
  historical bounded evidence, not successor admission. The bounded temporary
  build/staging and exact owned retirement bars are accepted; they are not installed-authority
  or provider proof. The resulting clean published source needs its own re-pin/review and six
  successful exact-head workflows before successor admission. No old-head CI or clean-worktree
  waiver transfers. After that boundary, prepare the remaining Step 7 hosted checkpoint
  off-clock under the existing plan, not by reusing the retired host or its observations.
  A8 used clean local/remote admission Core `864c2c8fd4cf3892a0a0a424515ee9295556b55f` and
  six exact-head successful runs: `37226340828`, `37226340827`, `37226340859`, `37226340829`,
  `37226341047`, `37226340881`. Runtime remained Core `d67886f`, Launcher `dd667c3d`,
  Protocol `e5fe1c83`; Launcher `77478ae9` was test-only, not an install target.
  MUSE accepted the fresh preparation and additive administrative-API wording correction,
  then the human separately authorized one bounded host/build/install/retention/retirement
  attempt. Fresh policy/operator/key/source checks, creation, two native guards, serial-key
  binding and the sole root-build call passed. Rust 1.95/X64 installation, exact locked builds,
  eight artifact build/private-stage/install digest rows and ELF64/X64 checks, and all 22
  allowlisted exports passed within the retained script/record boundaries. These are not
  independent guest/root/cgroup attestation or an installed authority service.
  Original outcome `/tmp/ota-step7-build-install-a8-preparation-20261004`,
  `live-outcome.manifest.sha256`
  `d8d9310d9b6efa8e9294178f6de515e024314436ab085679cf9ba353f9564c11` (1,180 entries),
  retains build/controller 0, cleanup 1, wrapper 1 and incomplete terminal status. The VM
  was deleted, but the strict absence parser refused an exact disk `HTTPError 404` form,
  safely stopping before firewall/subnet/network deletion. MUSE accepted this honest partial
  outcome; the original failed retirement remains unchanged, not retrospectively green.
  A separately frozen/reviewed cleanup-only sidecar tightened exact argv/operator/status/
  stream linkage and recognized only that exact scoped disk-404 form. Eight offline methods,
  shell syntax and the actual retained-response check passed. The human then separately
  authorized one continuation, not a host/build retry. Additive cleanup outcome
  `/tmp/ota-a8-cleanup-continuation-20261005`, `cleanup-outcome.manifest.sha256`
  `5182cbc1a36fe20ecb1c00070955cea2bd503b83b5b16a434a5580b0582f6ee9` (109 entries),
  is independently accepted by MUSE with no P1/P2/P3. Fresh exact-ID/selfLink guards preceded
  the three ordered deletions; 20 records retain 13 zero calls and seven exact scoped absence
  results, final empty project instance/address lists, then the held attempt-key unlink and
  successful finalization. No A8 VM, disk, firewall, subnet, network or held key remains.
  This is bounded recorded retirement, not secure erasure or global cloud-resource absence.
  The temporary build host and staged binaries are gone; no reusable installation, host
  liveness or policy freshness transfers. Read-only Secret Manager administrative API contact
  occurred during admission, not secret-value retrieval or provider-workload execution.
  Signed authority, runner registration/activation, dispatch, workload OIDC/STS/IAM acceptance,
  secret delivery, materialization, injection and selected work were not exercised by A8.
  Non-atomic identity-before-delete, independent root attestation, hostile-root escape,
  post-retirement apt-source content and broader credential-leak prevention remain unproved.
  Same-user/pathname/during-derivation races and atomic checked-input-to-use are not proved.
  A4/A5/A6 and accepted A7 are historical/retired; their evidence is retained in the plan.
  The original unaccepted A7 candidate/key remains untouched and was not adopted by A8.
  This is private operator sequencing, not a public runtime/spec/schema/command/JSON change;
  Site/Skills/Examples/Learn/FAQ/Glossary propagation is not required. V12.1 Step 7 remains
  active/open; further live build/install, signing, runner/provider activation, later delivery,
  Step 8 and V12.2 remain inactive. No commit/push/merge/release, retry, key generation,
  successor allocation or activation follows from this reconciliation alone.
  Earlier preparation records below are historical, not current admission or permission.
  A5 used clean published Core `d36ac2dbc30163d81a4614ae1ccac6020a6c9410`, unchanged runtime
  Core `d67886f27f6ee76766401d85427b0e4ea63694e2` and Launcher/Protocol pins, and six successful
  exact-head source checks (Release Gate `37117001127`). MUSE cleared the frozen V2 host packet
  after closing three private operator-kit P2s: canonical six-workflow admission, exact Google
  operator propagation and executable numeric-ID partial retirement. The human then authorized
  only one host/native guard/strict key/repaired root baseline plus cleanup.
  Fresh admission, exact creation and all six native host-guard reads passed. The key scan and
  authenticated serial read both returned 0, but the retained serial output contained no raw
  ed25519 public key for the reviewed exact comparison. The strict binding gate returned 1;
  no strict known_hosts file or authenticated root-baseline command followed. An SSH key scan
  is not authenticated root execution or repaired baseline proof. No retry, ingress widening,
  build/install, runner registration, IAM mutation, dispatch, signing or workload provider call
  occurred. Exact retirement completed with 13 successful commands and seven explicit NOT_FOUND
  reads, native instance/address checks and removal of the local private key. No global absence
  or provider authority is inferred. Frozen outcome and cleanup identities are in the active plan
  and `/tmp/ota-iam-live-20261003-a5-preparation/host-packet-v2`. The original packet and A4
  evidence are unchanged. MUSE found no P1/P2/P3 in the terminal-outcome review and diagnosed
  the incompatible raw-key expectation; a retrospective matching fingerprint is not A5 acceptance.
  Any fingerprint-binding candidate needs network-denied fixtures and independent source review.
  Source candidate under `/tmp/ota-host-key-preparation-20261003` now derives canonical SHA-256
  from exactly one scanned ED25519 wire key, compares it to one unique fingerprint in complete
  typed serial fingerprint prefixes through the ED25519/256 randomart header, and refuses
  malformed/missing/ambiguous/cross-algorithm or mismatched prefixes. Randomart body/footer
  completion is not verified; a stream ending at the header may be accepted. Its existing
  wrapper publishes only the validated row at mode 0600 and
  refuses existing files/symlinks. Syntax and five network-denied unittest methods pass, including
  27 parser refusal cases and production publication controls. This replays A5 evidence offline;
  it does not accept A5, prove a live baseline or authorize another host. MUSE's frozen source
  review found no P1/P2 and one P3 overclaim about complete generation stanzas. The additive
  `/tmp/ota-host-key-preparation-20261003-p3` candidate narrows wording/test labels only; parser
  and wrapper hashes are unchanged, and fresh bounded syntax/five-test validation passes.
  The original freeze is preserved. MUSE cleared the frozen delta with no remaining P1/P2/P3;
  source/validation identities are recorded in the active plan. Clearance permits only integration
  into a separately frozen successor preparation packet, not that packet or a live gate.
  A6 preparation kit `/tmp/ota-iam-live-20261003-a6-preparation` now incorporates that repair,
  fresh proposed names and unchanged strict source/operator/ownership gates. Frozen manifest
  `4afaf55a377576230c8b4968e1024966bb342d4264b0d15ca086540516f3533e`; MUSE found no P1/P2/P3
  and cleared preparation-kit coherence/source integration only, not launch readiness.
  Four bounded network-denied validation records pass: syntax, five host-key methods,
  2 accepted/31 refused host-guard controls and 29 admission/retirement controls. Failed initial
  scratch-path validation remains retained, not acceptance. No old ledger/key/admission success
  is promoted to A6. At kit review, Core was at `d36ac2d` with these two docs dirty; that
  clean-state blocker was not waived. No key, /32, fresh native/policy/GitHub readbacks or final candidate
  exists yet. MUSE inspected source/retained evidence without an independent test rerun.
  The human authorized the reviewed two-file documentation commit only. Next: re-pin/re-review
  that clean committed source and resolve key/readback/final-packet gates before separately authorized fresh host
  action. No VM or provider action has occurred.
  This handoff records the reviewed preparation boundary; no push, release or live successor authorization.
  This is a private operator key-binding preparation gap, not a demonstrated Ota runtime gap;
  no Site/Skills/Examples/Learn/FAQ/Glossary/schema/command/JSON propagation is required.
  A4 used clean Core `98d9b59c1fee8732ab7a8474c103a36e066d2101`, unchanged runtime pins and
  six exact-head source successes (Release Gate `37089370771`). MUSE accepted the read-only
  audit and frozen host-creation-only packet. Creation, all six native v2 host-guard reads and
  serial-linked host key passed. The one permitted root baseline SSH returned 0 at
  09:30:43Z-09:30:45Z, with matching bracketed HTTPS egress observations. This establishes
  bounded authenticated execution, not TCP/22 source, independent host/clock or provider proof.
  SCOOBY then found the baseline's two top-level `! getent` calls do not fail closed under
  Bash errexit. MUSE independently confirmed P2 and superseded the earlier account-assertion
  clearance. No individual lookup statuses were retained; aggregate exit 0 and empty account
  output do not close that gate or imply either account actually existed. No second SSH,
  repair-and-continue, build/install, runner registration, IAM mutation, dispatch, signing or
  workload provider call occurred. Exact numeric-ID retirement completed with 11 successful
  commands and five explicit NOT_FOUND readbacks; native lists exclude the instance and any
  reserved former IP, and the local private key is removed. Original evidence remains unchanged
  under `/tmp/ota-iam-live-20261003-a4`; the active plan binds packet/outcome/retirement identities.
  The off-clock repair is staged under `/tmp/ota-root-baseline-preparation-20261003`: labelled
  lookup stdout/stderr/status boundaries, only keyed-not-found status 2 accepted, found=0 and
  every other status refused. Syntax and 11 full-baseline stubbed controls pass with network
  denied, including both accounts and suppression of the final success marker. This is source
  readiness only, not a new host or installed baseline. MUSE cleared the frozen source/retention
  review with no remaining findings. This is a private operator-fixture defect, not a new Ota product gap; no public
  Site/Skills/Examples/Learn/FAQ/Glossary/schema/command/JSON propagation is required.
  Cloud Asset API restoration remains separately unauthorized; global resource/account or
  credential absence is not proved. Prior A3 connectivity failure and exact retirement remain
  unchanged in the active plan and `/tmp/ota-iam-live-20261002-a3`; its cause remains undetermined.
  Prior A2 disposition: the operator authorized the bounded IAM checkpoint and exact cleanup. A2 used
  clean Core `4aa1baa320e4bdafdddd2fc3682230730eff1b0f`, runtime Launcher `dd667c3d`, Protocol
  `e5fe1c83`, and successful exact-head source checks, including Release Gate `37006393687`.
  Exact Linux/X64 builds, guarded isolated GCP host, restricted group 10/runner 1385, disabled
  exact-run provider, sole account-scoped access-token-only custom grant and complete enumerated
  policy coverage reached frozen PREISSUE review. MUSE found one operator-preparation P2:
  root ownership hardening left `.env`/`.path` root:root 0600, unreadable by the job account;
  the root-only Listener version probe masked the required job-principal startup failure.
  The unchanged stop rule required cancellation/retirement, not chmod-and-continue.
  Run `37052192848`, attempt 1, job `110988059901` is cancelled, unassigned with zero steps.
  No authority was issued/provisioned and no workload OIDC/STS/IAM credential checkpoint call
  occurred. Administrator/tool-observed final SSH assertions reported no installed authority,
  job/exec process or workload marker. Their command/returned exit 0 is retained retrospectively,
  with local UTC bounds only, not a contemporaneous native clock/status record or attestation;
  the plan links this distinct provenance and the separate 47 timed cloud/GitHub records.
  The exact A2 VM/auto-delete disk/firewall/subnet/network are absent and its local operator key
  removed. Runner/group are 404; the sole grant is removed; account disable/delete succeeded,
  with exact ID/email absent from the native active-account list. The subsequent account describe
  is PERMISSION_DENIED/indeterminate, not NOT_FOUND. Role is DISABLED/deleted; provider and pool
  converge to DELETED/disabled (the initial pool read remained ACTIVE while deletion was pending).
  Historical resources are untouched; Cloud Asset API restoration remains separately unauthorized.
  Private evidence is under `/tmp/ota-iam-live-20261002-preparation`; the active plan records exact
  packet, metadata and retirement identities. `register-runner-v2.sh` is a staged, syntax-checked
  preparation repair: root-controlled/job-readable/non-writable runtime configuration plus an
  actual job-principal Listener version probe before issuance. It has not run on a new host and
  requires independent review and fresh installed proof; source repair is not hosted acceptance.
  This operator defect changes no public product surface or release claim; Site/Skills/Examples/
  Learn/FAQ/Glossary/schema/public-JSON/command-reference propagation is not required.
  Historical preparation/source checkpoints before A1/A2 follow; their no-host/no-dispatch
  observations and limited permissions are superseded only by the exact A2/A3/A4 dispositions.
  Preparation is staged at `/tmp/ota-iam-live-20261002-preparation`. MUSE's initial review
  found two P2 planning gaps: predefined federation grants include ID-token minting, and the policy
  audit lacked a closed readback set. The proposed correction uses an account-scoped custom role
  containing only `iam.serviceAccounts.getAccessToken` and a closed project/ancestor/asset-policy
  audit with explicit coverage limits. Frozen MUSE re-review found no P1/P2/P3 issues and cleared
  documentation/preparation retention only, not host, PREISSUE or installed readiness.
  The operator authorized only Cloud Asset API enablement for the read-only baseline; enablement
  succeeded. Initial and repeated unfiltered inventory/policy reads agree (142 resource rows,
  three indexed policies); the four fresh-account/principal/pool/public queries are empty.
  Native reads confirm the older reader's federation/canary grants, not grants to the fresh pool.
  Native Logging reads also found three views absent from CAI, with empty direct policies.
  Evidence is bound to the exact artifact identities in the active plan's baseline retention
  paragraph. A fresh 30-command collection records invocations, statuses, UTC times and output
  hashes; the older indexed STS2 pool is natively DELETED/disabled, not an empty policy result.
  MUSE cleared docs-only observation retention after evidence/status wording repairs. A frozen
  coverage candidate now classifies all 16 observed types, 141 unique indexed names, three native
  LogViews and five additional native deleted pools. All 58 targeted native reads exited 0,
  including 42 empty direct subnetwork policies; native list parity checks pass. Coverage outcome
  review found no P1/P2 gaps within the enumerated baseline; two P3 retention repairs add
  reproducible native parity and narrow the credential-read wording. MUSE cleared their final
  delta and the GCP credential/key/network guard with no P1/P2/P3 findings for docs/private
  preparation retention only, not launch or hosted readiness. The operator reports AWS account
  suspension and selected GCP instead; AWS readiness is
  no longer a launch prerequisite. Six successful read-only GCP checks confirm project/billing,
  zero instances, a London e2-standard-4, exact Ubuntu X64 image and regional quotas. Host delta,
  current price/operator /32 and final source/packet review remain open; no host was created.
  Host creation reopens policy coverage for its new resources. This does not permit wider grants
  or provider activation. No Google mutation beyond the authorized API
  enablement occurred; API restoration needs explicit cleanup permission/no competing use.
  Predecessor evidence: the fresh bounded Google STS checkpoint passed at Core
  `41a979b0d16200cef1608c5f469279b1dc2d2440`; MUSE cleared outcome retention after the
  narrow P3 diagnostic-ordering wording correction.
  V12.1 Step 7 remains active. MUSE cleared the next IAM Credentials checkpoint plan after the
  timestamp-requirement correction. The reviewed network-disabled source batch is now committed
  at Core `b789fad6`, with the Launcher test-only extension at `b5c8f99`; production pins stay
  unchanged. MUSE cleared the production IAM source-enablement plan, and the operator authorized
  its activation-record commit, dedicated protected IAM workflow edits and focused checks.
  Source activation is committed at `a1b06cb1358f3454f078e572503eaeab925dee5c`.
  The reviewed source batch is committed and pushed at Core
  `d67886f27f6ee76766401d85427b0e4ea63694e2`; MUSE's first source review found one P2
  producer/snapshot project mismatch. The repair selects the target before resolving bindings
  and makes offline inspection share actual runtime semantic reconstruction and production
  operation planning, without returning runtime authority. Frozen MUSE repair review has no
  remaining P1/P2/P3 findings; all reviewed hashes and HEADs matched, and the final repaired
  runtime and network-disabled Linux checks passed. On 2026-10-02 the operator authorized
  commit and push of this reviewed Core batch and the Launcher test-only extension, committed
  and pushed at Launcher `77478ae9fad170d2484aa4409b0bb3032436a52c`. Runtime pins remain unchanged.
  Additive request V3 binds the IAM account/project and
  full invocation into unchanged signed V2/V4 carriers; only the exact IAM route gains the
  fixed production callback. Both new-route sends check fresh temporal authority after HTTP/agent
  preparation. The dedicated manual workflow is published on the implementation branch but
  remains undispatched. Exact-head source Release Gate `36941232116` passed at `d67886f2`;
  docs-quality `36941232215` and cargo-deny `36941232100` passed. No implementation merge/release is authorized.
  Resolved scheduling prerequisite: the initial read-only GitHub lookup of
  `.github/workflows/secret-delivery-google-iam-live.yml` returned 404; the default branch is
  `main`, and GitHub's documented manual-dispatch path requires default-branch registration.
  Do not infer that a branch-only new workflow is dispatchable. MUSE cleared an inert same-path
  registration stub with matching inputs, empty permissions, an ordinary hosted runner and
  unconditional refusal; do not copy the protected IAM implementation onto `main`.
  On 2026-10-02 the operator authorized only this stub's promotion after independent review
  and passing exact-head gates. MUSE cleared the frozen 49-line addition with no P1/P2/P3
  findings; it is pushed at `23ca11096211c8fc25c945ed0d684dc5859c8cbc` on
  `bobai/register-google-iam-live-workflow`, based on main `8422f2ce915c2c0b853c0074ef1cdf8c69ab2d9f`.
  Registration Release Gate `36942569361` passed on Linux/macOS/Windows, and all six
  registration-branch workflows completed successfully at the exact registration commit.
  Source Release Gate `36941232116` passed. The frozen stub hash and unchanged main base were
  rechecked before the authorized one-file fast-forward from `8422f2ce` to `23ca1109`.
  GitHub independently reports main at `23ca11096211c8fc25c945ed0d684dc5859c8cbc` and workflow
  `372936715` (`secret-delivery-google-iam-live`) as `active`. This proves registration only,
  not manual execution, protected installation, provider authority or hosted IAM acceptance.
  The real implementation remains on `1.6.29-implementation`; the main stub always refuses.
  No default-branch setting, alternate trigger, protected runner or provider change is authorized.
  Historical host proposal, superseded by the GCP selection above: on 2026-10-02 the operator
  authorized one temporary AWS host and exact-resource cleanup,
  retaining the root-connected plugin for this disposable account. Read-only AWS preparation
  selected London `eu-west-2`, on-demand `m6i.xlarge` (4 vCPUs/16 GiB), Canonical Ubuntu 24.04
  X64 AMI `ami-05a81b93a249716f9`, and a 60 GiB encrypted baseline gp3 auto-delete boot disk.
  Verified base prices: compute USD 0.222/hour, public IPv4 USD 0.005/hour and disk USD
  0.0928/GB-month; estimated base cost USD 0.235/hour, excluding transfer/taxes/Google charges.
  No EC2 resources have been created. Defer launch until source commit/push and the reviewed
  hosted packet are ready; avoid an idle billed host. Use no instance role or NAT gateway,
  restrict SSH to the operator /32, retain exact created resource IDs, and verify VM/disk/public
  IP/key/network cleanup rather than merely stopping the VM. The IAM `ota-operator` identity
  remains read-only and unused by the plugin; it is not a protection boundary for the root session.
  This host-lifecycle authorization does not authorize source commit/push, Google configuration
  or grants, signed authority issuance, runner activation, workflow dispatch or provider contact.
  Merge and release remain inactive pending their separate gates.
  Historical predecessor: Core `0476b9a3b484c672fccb164517a1dc3097cd5e66`
  passed exact-head Release Gate `36745309983`
  and connected source gates; no merge or release occurred. Launcher is pinned to
  `dd667c3d6b63d8d26d9e2a40f831e9d08a7a6864`, Protocol to
  `e5fe1c83e562e02f60e27026c7148918bd016155`.
  Fresh run `36793610729`, attempt `1`, job `110151889834`, was assigned to runner
  `1383` in group `8` (`ota-sts5-20261001`) at that exact Core revision. Branch-ref
  scheduling was exercised successfully under the operator branch/dispatch freeze; it does
  not enforce immutable workflow code.
  Broad source/helper/configuration and fresh pre-issue review completed off the signed clock.
  MUSE explicitly acknowledged reviewer readiness before one canonical issue, then conditionally
  cleared the frozen signed/installed delta with no findings. Production offline verification
  compared the complete signed V2 graph/record with independent producer/build/artifact
  expectations. Immediate source/group/run/provider/installed/strict-absence checks passed;
  after provider/pool enablement and the 300-second operator propagation hold, 2409 signed
  seconds remained, exceeding the 1200-second final-admission floor. The hold is not evidence
  of provider acceptance. Signed expiry was `2026-10-01T01:11:08Z`; no authority is reusable.
  The first reconciliation step failed at `cmp` with
  `/srv/ota-v3-pressure/ota.yaml: Permission denied` (exit 2). The job principal cannot
  traverse the intentionally execution-owned 0750 repository or read its 0640 contract.
  The protected-client/provider step was skipped: no job OIDC/STS request or protected
  runtime reconciliation occurred. This is a workflow principal-boundary mismatch, not a
  demonstrated Core runtime defect. Job-side `test ! -e` marker guards also cannot prove
  absence inside this inaccessible repository; preserve owner-side absence observations
  separately and review the connected guard before another attempt.
  Provider/pool were disabled, runner stopped, and terminal administrator observation showed
  MainPID 0, empty cgroup, no principal processes, active slots, authority scopes, selected-work
  marker or public capture record. The runner unit ended failed/failed with exit 2; do not
  report protected-client finalization or root-custodied semantic proof.
  Reviewed teardown returned 0. VM `ota-sts5-20261001`, boot disk, network, subnet, firewall,
  runner and group are absent; the pool is disabled and soft-deleted, and provider deletion
  was acknowledged followed by NOT_FOUND under the deleted pool. No temporary host remains.
  Retention: `docs/pressure/retained-artifacts/secret-delivery-google-sts-readiness-36793610729.json`.
  Private frozen preparation/installation packet and separate result/cleanup observations:
  `/tmp/ota-sts-live-20261001-attempt5`. Earlier cancelled attempts
  `36764606243`/`36775307269`/`36784346916` remain in their retained readiness records
  and the Step 7 plan; they did not execute job OIDC/STS.
  MUSE confirmed two P2 workflow ownership defects and a P3 real-principal testing gap; the
  failure/cleanup retention is independently cleared. The operator authorized the connected
  owner-correct repair on 2026-10-01. The reviewed implementation moves exact original fixture-byte
  comparison into the existing administrator preflight, removes all job workload-file probes,
  qualifies the job's selected-work projection as client-terminal-only, and shares a root-only
  descriptor-relative marker observer in the existing prestart/terminal recipes. No workload
  permissions, installation schema or runtime authority have changed. New tests exercise exact
  bytes versus semantic-equivalent changes and the real job/exec permission split; topology and
  client response in the local workflow fixture are explicit stubs, not hosted proof.
  Local validation passes: macOS workflow tests 5/5 and pressure preflight tests 6/6; fresh
  network-disabled Linux/arm64 actual-workflow principal regression 1/1, including reidentified
  wrong source/request and writable-public-evidence refusals; fresh Linux root production
  preflight regression 1/1, including exact fixture bytes and non-mutating comment-only refusal.
  Actionlint, Python syntax, declared formatting and first-party sync checks pass. These are
  local fixture results, not protected Linux/X64 hosted or provider proof.
  MUSE completed the independent frozen source review with no P1/P2/P3 findings; both P2
  ownership defects and the P3 real-principal testing gap are resolved within this scope.
  All ten reviewed file hashes matched before and after review (manifest SHA-256
  `891240afbfc583ef79cc475f82542bb9ea37b1c673fd03d0df412e2899e11894`). MUSE source-reviewed
  the regressions and accepted the reported validation; it did not independently rerun them.
  Source/commit readiness is cleared. The operator authorized commit and push on 2026-10-01;
  this does not authorize hosted activation, merge or release.
  The reviewed repair is committed and pushed at Core
  `41a979b0d16200cef1608c5f469279b1dc2d2440`. Release Gate `36848230857` passed on Linux,
  macOS and Windows; all seven other connected source checks passed. Ordinary Linux/systemd
  proof `36848230872` also passed at that exact revision on attempt 2, job `110363849585`,
  using local OrbStack runner `27`. Attempt 1 was cancelled queued with no steps after restoring
  the existing stopped runner failed to assign that old job; the one exact-source retry executed.
  Artifact `11161343431` binds the same Core SHA. Its runtime proof reports `ok: true`, workflow
  `app` and verdict `ready`, with `agent_verdict: not_ready`; this is not agent-session containment
  or protected Step 7/provider evidence. Registration, labels, clock and user bus were checked
  before starting the ordinary runner; no re-registration or permission changes were needed.
  Fresh operator-helper source preparation is at
  `/tmp/ota-sts-live-20261001-attempt6-source-preparation`; the historical attempt-5 packet is unchanged.
  MUSE identified terminal scope/slot refusal gaps and a build-helper legacy marker probe. New
  copies use the canonical root observer, explicit all-entry slot absence, and systemd's own
  `ota-*.scope` filter with empty-output/error refusal. Prestart consumes five independent producer
  expectations through the existing installed verifier before observing the marker. Local guard
  regressions cover failed-unit status symbols, active/inactive scopes, observation errors, hidden
  slot entries, subdirectories and aliases. MUSE's final frozen scope-fix recheck found no
  P1/P2/P3 findings and cleared these six helper-source copies only; hashes matched before and
  after review. Reported tests were source-reviewed, not independently rerun. Fresh-packet,
  issuance and activation clearance remain separate.
  The operator separately authorized one fresh bounded OIDC-to-STS attempt on 2026-10-01.
  Fresh VM/network/subnet/pool/group `ota-sts6-20261001` were prepared; the VM had no attached
  service account/scopes, an auto-deleting boot disk and a 7200-second DELETE lifetime. Exact
  clean Core/Launcher builds completed with the runner stopped and provider/pool disabled.
  Fresh run `36874656786`, attempt `1`, job `110410885453`, passed at Core
  `41a979b0d16200cef1608c5f469279b1dc2d2440` on exact runner `1384`/group `9`.
  Fresh packet: `/tmp/ota-sts-live-20261001-attempt6`. MUSE cleared source and complete
  PREISSUE readiness with no P1/P2/P3 findings after one off-clock model-capacity retry;
  it explicitly acknowledged reviewer readiness before one canonical issue.
  MUSE found a P2 cancellation-cleanup gap: ephemeral registration can survive cancellation
  before execution. The fresh cleanup copy now verifies/removes exact runner `1384` or confirms
  organization-wide absence before group deletion, with twelve production-function guard
  scenarios passing locally. MUSE cleared the correction. One canonical provisioning operation
  completed with independently observed producer/build/artifact expectations; complete production
  offline signed V2 graph/record/request/provider comparison and strict prestart observation
  returned 0. MUSE's bounded signed/installed delta review found no P1/P2/P3 findings and
  conditionally cleared one attempt; all 61 preissue and 24 installed packet hashes matched.
  Generation 1 expiry was `2026-10-01T15:39:52Z`; no authority is reusable. After the 300-second
  operator propagation hold, final complete verification and exact external/strict-absence checks
  passed with 2482 signed seconds remaining, above the 1200-second floor. The assigned job passed
  installation reconciliation and the bounded STS checkpoint. Its closed projection reports one
  Core-level GitHub response and one accepted STS response, matched-unadmitted JWT claims and
  discarded returned token. The protected client deliberately exited 1 before IAM Credentials,
  Secret Manager, materialization, injection or selected work; the job validated its terminal
  envelope and child/scope/cgroup/active-slot cleanup. This is job-observed acceptance and
  finalization, not independently root-custodied semantic attestation or provider cardinality.
  Separate administrator terminal observation returned 0: stopped MainPID 0, empty runner cgroup,
  no principal process, authority scope, active-slot entry or selected-work marker. The marker
  observation is point-in-time absence, not proof of never executing. No public capture record
  was produced. The runner unit itself ended failed/failed with exit 2; read-only/denied text
  and listener-completion text were both observed without establishing their ordering. Retain
  that separately from the passing workflow and protected-client finalization. A clean runner-unit
  exit is not proved.
  Provider/pool were disabled and reviewed teardown returned 0. Readbacks confirm VM, boot disk,
  network, subnet, firewall, runner and group absent, disabled soft-deleted pool, and provider
  deletion acknowledged then NOT_FOUND under the deleted pool. No temporary host remains.
  Retention: `docs/pressure/retained-artifacts/secret-delivery-google-sts-live-36874656786.json`.
  MUSE confirmed the bounded positive STS bar is met with no P1/P2 findings. Its sole P3,
  an unsupported diagnostic-ordering claim, is corrected; outcome/cleanup retention is cleared
  within these proof limits. No later delivery, merge or release is authorized. The operator's
  activation/evidence documentation is committed at `def8d789bd4e205c241f0201fbb961e6d444469d`.
  The first network-disabled IAM batch is implemented and independently reviewed; the operator
  authorized its source commit on 2026-10-01. The Launcher test extension is committed at
  `b5c8f99`; Core's production Launcher pin remains unchanged.
  It retains one consumed V4 owner through STS/IAM, signed full graph/record comparisons,
  conservative credential/transaction deadlines and terminal disposal. That committed offline
  batch did not attach production IAM transport or admit IAM outside tests. The current source-only
  successor is recorded below; existing OIDC/STS routes stay terminal. Hosted activation remains
  a separate inactive review/authorization gate.
  MUSE's initial planning review found one P2: the current IAM parser's generic RFC 3339
  round-trip check rejects valid Google fractional timestamp formats with trailing zeros.
  The proposal now requires a narrow provider-format repair and full-precision expiry-boundary
  regressions in the first batch. This is a named Ota parser gap, not a provider or repo issue;
  the production parser now validates Google's grammar/calendar and full-precision expiry instead
  of generic round-trip formatting. MUSE's frozen focused recheck cleared the planning P2
  with no remaining P1/P2/P3 findings and all four reviewed hashes matching. Plan readiness is
  cleared. Initial frozen source review found one P2: a timestamp sampled before expensive
  signed-truth/JWT validation could be stale at dispatch. The repair completes that validation
  before sampling fresh wall/monotonic time, then applies lightweight binding/JWT/credential
  deadlines before dispatch and after parsing. Eight additional deterministic cases require
  zero corresponding calls when validation crosses expiry or the final clock fails; clock
  callbacks also assert full context validation completed first. MUSE's frozen repair recheck
  cleared the P2 with no remaining P1/P2/P3 findings; all nine hashes and both HEADs matched.
  It checked source and retained logs without rerunning tests. Only this review-outcome record
  changed after that frozen review. Source commit readiness is cleared, not hosted/provider
  readiness. Local macOS checks pass: provider
  client 16/16; signed-owner IAM/STS matrices 2/2 (52 IAM cases and 14 STS cases); legacy
  no-promotion 2/2; production IAM admission refusal 1/1; existing STS workflow 5/5.
  The network-isolated Linux/arm64 root fixture passes 1/1 across 52 IAM actual-child cases,
  including real-clock seams, fake acceptance followed by deliberate refusal, every new refusal
  category, durable acknowledgement and exact child/scope/cgroup/active-slot cleanup. This is not
  hosted Linux/X64 or provider proof. The fixture uses Launcher HEAD
  `6fd8cde8782b772fb22eead5f8c691f036b425b5` plus its uncommitted test-only extension; its build
  identity is a local fixture label, not installed authority. Core's runtime pin remains unchanged.
  Logs: `/tmp/ota-iam-offline-expiry-repair.log`; the existing 14-case STS child fixture also
  passes 1/1 after the clock-ordering repair (`/tmp/ota-sts-offline-expiry-repair.log`). No Ota
  invocation scopes remain after both runs. Formatting,
  first-party sync and diff checks pass.
  Strict `clippy --lib -- -D warnings` is not a passing gate: it reports 863 diagnostics, including
  unrelated pre-existing code. The filtered provider/binding diagnostics show only the unchanged
  STS HTTP URI comparison; no broad lint cleanup is included. The activation freezes
  production relay admission at the existing OIDC/STS routes; IAM admission is test-fixture-only
  in this batch. No IAM workflow, producer/request shape, Protocol or Launcher production change
  is authorized. The connected Launcher change is test-only: extend the existing actual-child
  cleanup fixture without changing its production runtime or Core's pinned revision.
  The offline source batch is committed at Core `b789fad6` and Launcher `b5c8f99` without push.
  The independently reviewed production IAM source-enablement batch is committed and pushed at
  Core `d67886f2` on activation `a1b06cb1`. It adds administrator request V3 (full invocation plus
  exact WIF provider, project and service account), unchanged signed V2/V4 carriers, complete
  offline rederivation, fixed IAM transport and one dedicated protected workflow with
  comparison-only mirrors. Production route tests lock legacy isolation and closed refusal counts.
  The IAM matrix expands from 52 to 58 cases: HTTP/agent preparation expiry and clock failure
  for both sends, plus positive controls through the same preparation/guard seam. Expired or
  uncertain preparation must reach zero injected sends, not merely report an accepted callback.
  The first frozen batch passed macOS producer/preflight 20/20, provider client 16/16, IAM 58-case
  and STS 14-case matrices, router, manual workflow and legacy-route checks. Its isolated
  Linux/arm64 actual-child checks passed all 58 IAM and 14 STS cases with no scopes remaining;
  both workflow/principal tests passed in fresh network-none containers. Logs are
  `/tmp/ota-iam-source-send-gate.log` and `/tmp/ota-sts-source-send-gate.log`.
  Those results did not detect the P2: the producer changed the invocation project/resource after
  resolving a snapshot for `ota-pressure`, and the old preflight regenerated that same inconsistent
  payload. Runtime correctly refused it before V4. The repaired producer validates/selects the
  target first, derives the snapshot locator/scope and invocation from it, and resolves identities
  once. Offline inspection verifies raw signed stores itself, internally derives policy evidence,
  shares runtime semantic reconstruction and returns only a seven-field production-plan projection.
  It cannot manufacture a snapshot, V4 authority, provider owner or dispatch handle. Repaired
  producer/preflight 21/21 passes, including a validly signed locator mismatch rejected by shared
  reconstruction and the requested-project full plan/locator positive control. Repaired runtime
  authority/snapshot tests pass 27/27; the serial offline Linux build and default library check
  pass. Fresh network-disabled Linux/arm64 actual-child checks pass all 58 IAM and 14 STS cases,
  with no Ota invocation scopes left. Logs: `/tmp/ota-iam-repaired-source-send-gate.log` and
  `/tmp/ota-sts-repaired-source-send-gate.log`. Both rendered workflow/principal checks pass in
  fresh network-none containers; the existing STS root installed-preflight production-entrypoint
  success/refusal regression passes 1/1 there. This is not an installed or hosted IAM proof.
  MUSE's frozen repair recheck has no remaining P1/P2/P3 findings. All reviewed file hashes and
  both repository HEADs matched before recording this outcome; only review-outcome records
  changed afterward. Its conditional source readiness is satisfied by the final passing checks.
  The connected Launcher change extends only its actual-child test list; production pins remain
  unchanged. No Protocol/Launcher production, public CLI, JSON, schema, authoring or support claim
  changes: Site/Skills/Examples/Learn/FAQ/Glossary/command cards need no propagation for this
  internal pressure batch. Core changelog and the Site sync waiver track this decision.
  Next: close the reviewed GCP host delta and policy evidence-retention repairs;
  keep host and all other Google/GitHub mutations inactive. Finish
  final source/hosted packet freeze and independent review after those blockers close. Read-only Google
  preparation confirmed project `ota-v121-step7-20260910` / `783599651848` is active and
  `iam.googleapis.com`, `iamcredentials.googleapis.com` and `sts.googleapis.com` are already
  enabled. This is not grant, inherited-access or provider-condition proof. The predecessor
  private packet at `/tmp/ota-sts-live-20261001-attempt5`
  is no longer available locally and must not be reused. No installed IAM packet, IAM policy
  proof or provider acceptance is established.
  The operator authorized this source-only batch, its protected workflow/focused checks and
  activation-record commit, followed by source commit/push on 2026-10-02. Hosted activation
  remains separately unauthorized.
  MUSE's initial frozen planning review found one P2: HTTP/agent preparation in the callback
  follows the owner clock sample. The corrected proposal requires fresh lightweight temporal
  checks after preparation immediately before both new-route sends, plus zero-send delayed-expiry
  regressions. MUSE's frozen focused recheck cleared the P2 with no remaining P1/P2/P3 findings;
  both document hashes and HEADs matched. Only review-outcome records changed afterward.
  Planning readiness is cleared; source activation takes effect through this documentation
  commit before implementation. Provider/cloud/workflow-dispatch authorization remains absent.
  Hosted activation remains separately inactive. Temporary GCP host lifecycle was selected by
  the operator after reporting AWS suspension, but is deferred until the packet is ready.
  Explicit no-service-account/no-scopes creation and a terminal exact-ID metadata/account
  readback must pass before any build/install or runner registration. The closed metadata/key-hash
  and exact network/subnet/tag/sole TCP22 /32 ingress guard is MUSE-reviewed; repeat all
  readbacks before issue/start. Any drift terminally refuses and retires owned resources.
  Do not change Google provider configuration/grants,
  issue authority, start the runner or dispatch an IAM workflow.
  Uncovered material behavior: bounded OIDC/STS and protected-client finalization are proved only
  by the job-observed projection plus separate administrator absence/cleanup observations.
  Provider/lower-layer cardinality, condition enforcement, independent JWT verification,
  root-custodied semantic attestation, revocation and token-memory erasure remain `not_proved`.
  Later delivery and selected-work continuation were not attempted. Cloud lifecycle, runner
  retirement and branch/dispatch freeze are repo-owned outside the declared Ota task scope;
  cleanup is administrator observation only.
  Complete installed comparison is proved by the offline verifier, not execution authority.
  Site/Skills/Examples/Learn/FAQ/Glossary/schema/public JSON and command reference are unaffected
  by this internal pressure checkpoint: no public command/schema/authoring concept changed. Provider delivery,
  Step 8 and V12.2 remain inactive. Numbered milestones retain their scope at recorded completion;
  this bullet and the latest Step 7 checkpoint govern the current next action.
- active version: V12.1 secret-delivery governance. Activated on 2026-09-02 after the released V12
  closure and feasibility review of PythiaLabs' credentialed CAEP boundary. The named first adapter
  is `google_secret_manager_github_oidc_process_environment_v1`, initially limited to a
  `linux/x86_64` GitHub Actions native-process recipient using GitHub OIDC, Google Workload Identity
  Federation, and one exact Google Secret Manager version. Activation authorizes implementation-order
  step 1 and, under separate amendments, implementation-order steps 2, 3, 4, 5, and 6: the
  provider-neutral requirement schema, canonical recipient/destination model,
  `SecretRequirementIdentity`, and the Core-owned provider-binding domain model with pure
  fail-closed resolution, followed by the exact first-adapter profile and implementation-subject
  descriptor model, the crate-private `secret_material_delivery` effect derivation foundation, the
  sealed provider-free evaluator/dry-run-plan model, and one retained command-scoped admission per
  invocation. Step 6 was independently reviewed and activated at Core `2dd20ab8`; its implementation
  is independently reviewed and committed at Core `67de2b4d`, with immutable consumer reconciliation
  recorded at Core `9d6696f0`.
  At that Step 6 completion boundary, no provider binding loader, concrete implementation registration, provider
  contact, materialization, injection, execution authority, positive evidence, or support claim was
  implemented. Subsequent Step 7 checkpoints below supersede that historical foundation;
  delivery, Step 8, V12.2 and later versions remain inactive.
- V12.1 implementation-order step 1 is independently reviewed and committed. `ota.yaml` now has an
  additive provider-neutral `secret_requirements` catalog with one initial
  `authentication_credential` / `external_api_authentication` vocabulary, canonical
  `process_environment` destinations, explicit task/workflow recipients, deny-only propagation
  edges, requested execution constraints, and domain-separated `SecretRequirementIdentity`.
  Validation requires `metadata.ota.minimum_version` of at least `1.6.28` and rejects provider
  selectors, secret defaults, destination collisions, compatibility ownership conflicts, unknown
  or noncanonical recipients, and noncanonical environment-variable names. The Site contract
  reference, Glossary, FAQ, Learn lesson, canonical Skill, and both installed mirrors carry the
  current fail-closed admission boundary; Examples remain unchanged because no usable delivery flow
  exists. Step 6 command admission now consumes exact selected recipients and refuses before setup
  or execution while protected production binding truth is unavailable. No provider binding loader
  exists and no secret bytes are loaded or delivered. Step 2 was explicitly activated under the V12.1 plan amendment
  after the connected proof-assurance hardening and immutable Linux/macOS evidence were inspected.
  It authorizes only the Core-owned provider-binding domain model, private canonical identities,
  opaque public disclosure, and pure fail-closed resolution of authority-sourced protected
  snapshots. It does not authorize a binding loader, CLI route, provider adapter, provider contact,
  OIDC/WIF, materialization, injection, policy/effect admission, execution, receipts, archives, or
  support claims.
- V12.1 implementation-order step 2 is independently reviewed and committed at Core `9218151b`.
  The sealed Core-only model derives domain-separated protected source-evidence and private-binding
  identities from exact requirement, provider reference, authority scope, source evidence, adapter,
  lifecycle, and target truth. The pure resolver validates the complete protected snapshot before
  selection, requires exact source/binding scope and requirement-target agreement, refuses zero,
  duplicate, conflicting, unknown, noncanonical, or substituted inputs without fallback, and
  returns only selected bindings and their sources. Its fail-closed public projection omits
  requirement linkage and every private source/reference input; changing private references does
  not create a public correlation oracle. No loader, CLI, adapter, OIDC/WIF, provider contact,
  materialization, injection, policy/effect admission, execution, receipt, archive, or support path
  consumes the model. Site, Skills, Examples, Learn, FAQ, and Glossary remain unchanged because
  this sealed internal foundation creates no authoring or operator workflow, command, output, or
  public vocabulary.
- V12.1 implementation-order step 3 is explicitly activated under a separate plan amendment. It
  authorizes only the crate-private capability-profile and implementation-subject descriptor model
  for `google_secret_manager_github_oidc_process_environment_v1`, deterministic identities, the
  complete protected GitHub OIDC/WIF/Google Secret Manager binding tuple, exact `linux/x86_64`
  GitHub Actions native-process target posture, unsupported-target refusal, and adversarial
  substitution tests. Stable profile semantics, implementation/build/target truth, and exact
  per-run values remain separated through `profile_semantic_identity`,
  `implementation_subject_identity`, and protected `SecretDeliveryInvocationBindingIdentity`.
  A concrete implementation subject cannot be finalized or registered until exact source, build,
  and artifact identities exist; registration and lifecycle identities remain uncreated. No loader,
  registry installation, lifecycle promotion, CLI route,
  token request, provider contact, network access, effect/policy admission, materialization,
  injection, execution, receipt, archive, assurance, conformance result, pressure evidence, or
  support claim is authorized, and the generic cross-cutting adapter/profile conformance plan
  remains inactive. The activated model is independently reviewed and committed at Core
  `9ac9274f`. It uses domain-separated profile, implementation-subject, and protected invocation-
  binding identities; requires the complete canonical GitHub claim set and exact issuer; binds the
  audience to the selected WIF provider URL; reconciles exact WIF, service-account, project, Secret
  Manager, requirement, and provider-binding truth; and independently re-derives retained input
  before accepting a resolved record. Ten adversarial tests cover normalization, omission,
  duplication, target widening, tuple substitution, provider-specific resource grammars, cross-
  resource aliases, and self-consistent resolved forgeries. At that completion boundary, Step 4
  and later steps remained unauthorized. This internal model creates no Site, Skill, Example,
  Learn, FAQ, Glossary,
  contract-schema, or public-JSON change because it has no loader, command, output, or operator
  workflow. Step 4 is separately activated at Core `a7dd07ba`. It authorizes only crate-private
  derivation of the `secret_material_delivery` V12 effect,
  realization inputs, internal refusal-assurance eligibility, and the narrow canonical evaluator
  extension needed to consume the derived effect. It keeps stable bounded consequence truth in
  `EffectIdentity`; binds exact requirement, recipient, closure role, invocation origin, protected
  binding/source, profile, implementation subject, and target truth in the realization identity;
  and requires structural authority reconciliation before policy evaluation. The independently
  reviewed implementation derives separate effect,
  attachment, realization, and internal refusal-attribution identities; independently reconstructs
  every retained derivation before policy evaluation; requires each realization to match exactly
  one retained selected invocation; binds that realization-to-invocation mapping into the
  execution-graph identity; preserves repeated attachment roles as distinct realizations only when
  the graph retains each exact occurrence; and reuses the canonical effect-policy fallback,
  precedence, set, decision, and verification finalizer. At the Step 4 completion boundary, Step 5
  and later steps remained unauthorized, and no loader,
  policy-authoring surface, CLI route, provider contact, materialization, injection, execution,
  public output, receipt, archive, assurance promotion, pressure evidence, or support claim is
  introduced. Site, Skills, Examples, Learn, FAQ, Glossary, contract schema, and public JSON remain
  unchanged because this internal foundation has no operator surface. Step 5 is separately
  activated under the V12.1 plan amendment after Step 4 was independently reviewed and committed at
  Core `f1178fe3`. It authorizes only one crate-private evaluator and dry-run plan consuming retained
  Step 1-4 truth. The implementation was independently reviewed and committed at Core
  `9fd4b4fb8f991b7dca8c2a2df2c2b6be5e5a0baa`.
  It derives the selected-requirement set from the retained contract and exact task/workflow graph,
  reuses the Step 1-4 semantic verifiers, preserves selected invocation order, canonicalizes
  unordered identity sets, and independently reconstructs both evaluation and plan identities.
  A genuinely empty selection remains `not_applicable`, policy denial
  refuses, and allow/warn is only structural eligibility for a future provider check. Every plan
  retains
  `availability: not_checked`, `provider_contact: not_attempted`, `delivery: not_attempted`, and
  `execution_started: false`; it is never runnable, available, delivered, supported, or assured.
  No loader, command consumer, provider interaction, execution, public output, or evidence path was
  authorized at the Step 5 completion boundary. Step 6 was independently reviewed and activated at
  Core `2dd20ab8`; it authorizes one retained command-scoped admission per invocation across
  `run`, `up`, proof, Doctor context, sandbox capability, and harness output. CI projection carries
  only a bounded public expectation; provider-checkout CI independently re-derives admission from
  its exact checkout and reconciles the public projection rather than inheriting render-host truth.
  The amendment preserves existing behavior for empty selections; refuses non-empty selections
  before setup, hydration, environment rendering, durable logs, services, proof artifacts, child
  creation, mutation, or provider contact when complete Step 1-5 truth is unavailable; and permits
  only a separately derived public-safe projection that excludes private decision, realization,
  evaluation, plan, binding, source, subject, and invocation identities. Every production binding
  loader, provider transaction, delivery, execution, positive secret-delivery receipt,
  secret-delivery archive, assurance, pressure, and support path remains unauthorized. Real
  `ota up` retains only its existing generic blocked execution receipt with
  `execution_attempted: false`. The committed implementation wires this one retained admission
  through `run`, `up`, runtime and lifecycle proof, Doctor context, CI projection/re-evaluation,
  sandbox capability, and task/workflow harness output. Empty selections preserve existing behavior;
  selected recipients emit only a domain-separated public negative projection with protected
  identities excluded. Focused command and schema regressions prove refusal before fixture setup,
  environment rendering, durable logs, proof artifacts, and command execution. Site `4169fe6` and
  Skills `082ce38` carry the connected public and agent guidance; Core `9d6696f0` pins both immutable
  consumer revisions. Standalone Examples remain unchanged because Step 6 has no executable delivery
  example. At this completion boundary, Step 7 and later steps remained unauthorized.
- V12.1 Step 7 was independently reviewed and activated at Core `ad14ae8b`. It authorizes only the
  first real provider transaction on a `linux/x86_64` GitHub Actions self-hosted protected runner:
  a new protected-runner profile; an administrator-installed launcher that starts unprivileged Ota
  with retained authority descriptors, no privilege-escalation posture, and cgroup-v2 process
  ownership; a caller-independent TLS transport; byte-exact GitHub OIDC, Google WIF, service-account,
  and numeric Secret Manager version requests; selected native process-tree environment injection;
  interruption; and terminal cleanup observation. The proposal proves only intra-invocation store
  consistency and leaves independent-administrator rollback, privileged escape, exfiltration, and
  memory erasure unproved. It separately binds a protected-self-hosted-runner-derived OIDC request-
  endpoint profile and exact JWT issuer. ARM64 discovery run `33919324208` confirmed the regional
  request-service distinction without provider contact, but at that stage it did not satisfy the
  required `linux/x86_64` protected-launcher fixture. The exact compatibility gate was subsequently
  closed by the later protected runs recorded below. Step 7 keeps GitHub-hosted and
  repository-provisioned runners,
  other targets, positive receipts/archives/assurance, adapter support, Step 8, and V12.2 inactive.
- Step 7 endpoint-compatibility work began at Core `777c42ab` with a no-checkout
  Linux/X64 discovery workflow that binds the live OIDC request URL shape to the protected runner
  service posture and administrator installation identity. A separate no-`id-token` job uses
  pinned Actions Toolkit code with a loopback fake capability to prove it calls the supplied URL
  rather than the JWT issuer; the protected job reads that retained probe and observes the live
  endpoint shape.
  OIDC and provider contact are not instrumented or claimed.
  Successful Core run [34153231585](https://github.com/ota-run/ota/actions/runs/34153231585), at
  exact Core `f921209561b26f38cdb74c5f20f71e0b6734ae0d`, separately proves the retained endpoint
  shape on self-hosted `linux/x64` Runner `2.337.0` and that the pinned Toolkit used its supplied
  loopback URL rather than the JWT issuer. Its public evidence retains
  `capability_identity: null`, `capability_derivation: not_attempted`, and
  `provider_contact_observation: not_instrumented`; it retains neither a bearer nor a protected
  request URL value. It does not derive protected-launcher capability, make an OIDC request, or
  contact a provider.
  The earlier Core discovery run `34137753776` was cancelled from its queue and remains no
  compatibility evidence. The subsequent independently reviewed Launcher witness
  [34159892077](https://github.com/ota-run/authority-launcher/actions/runs/34159892077), at exact
  Launcher `79f2b5e81a7454c7c56394520b27a5f4f3fb9f48`, Core
  `f921209561b26f38cdb74c5f20f71e0b6734ae0d`, and Protocol
  `ae3c8e99164c2d1db7f387f061f875272015bb36`, proves one bounded protected-runner governed
  invocation: installed client identity/help reconciliation, completed selected execution, terminal
  child reap, scope removal, empty-or-absent cgroup, active-slot removal, and one valid protected
  receipt archive with zero invalid archives. The exact public hosted payload is retained by
  `authority-launcher` commit `a2cdd23b82472b4c4b7ae07ae28356f0d7338d71` at
  `docs/pressure/retained-artifacts/systemd-v3-independently-administered-34159892077.zip`
  (`sha256:0b15512fe736bee4b35a2dd205a5e9e3d68a9af496ee86f812c364b4f322258f`). Current runner-group
  configuration was independently checked to allow exactly the two documented repositories and
  four branch-pinned workflows, but that mutable configuration is not run-bound artifact evidence.
  A later independently reviewed Launcher witness,
  [34241049867](https://github.com/ota-run/authority-launcher/actions/runs/34241049867) job
  `102111003771`, at exact Launcher `8ca4763c1e5c6ef5ac06c2be5b778c49344c5030`, Core
  `f921209561b26f38cdb74c5f20f71e0b6734ae0d`, and Protocol
  `e0af492ba8a6fbe01e805c79762909c9cda28198`, proves the run reconciled the replay directory
  through the provisioning-owned exact fresh-state inventory, the consumer workflow's exact
  inventory check, and the protected Launcher's effective systemd `ReadWritePaths` verification.
  The bounded production client then completed one governed invocation with child reap, scope
  removal, empty-or-absent cgroup, active-slot removal, and one valid protected receipt archive
  with zero invalid archives. The exact eight-file public artifact is retained by `authority-launcher`
  commit `a272fbe863715f04996340591a9ae29dc80ccedc` at
  `docs/pressure/retained-artifacts/systemd-v3-independently-administered-34241049867.zip`
  (`sha256:090f3ac9fa2b516370b8d844520d4759bce877fae7b0f6a541bd5e23539d6496`).
  This later witness still does not prove production capability-observation routing, exact
  production context ownership, accepted-session provenance, Core capability-projection
  reconciliation, provider contact, real OIDC exchange, secret materialization or delivery,
  Step 8, V12.2, or general agent/repository governance.
  Ubuntu 26 subsequently exposed that its canonical `sudo-rs` executable refuses policy listing
  from the already-root protected Launcher when systemd sets `NoNewPrivileges=yes`. Launcher
  revision `22f17eff3e005bb4544d583d0743721835539d0a` keeps `NoNewPrivileges=no` only for that
  root-owned observer while independently requiring and revalidating `NoNewPrivileges=1`, empty
  capabilities, and exact principal boundaries for the runner and selected execution process.
  After a fresh immutable reprovision, repository-owned run
  [34356604479](https://github.com/ota-run/authority-launcher/actions/runs/34356604479), job
  `102482773723`, completed one governed invocation at exact Launcher `22f17eff3e005bb4544d583d0743721835539d0a`,
  Core `104c1345117a39f899fdc6a48bf8cf689ffbe9b6`, and Protocol
  `58526f3f29299873e345352963e30b8a1677044f`. Its terminal records exit `0`, child reap, scope
  removal, empty-or-absent cgroup, and active-slot removal; protected history records one valid
  receipt archive with zero invalid archives and the exact matching archive identity. The exact
  eight-file public artifact is retained by `authority-launcher` commit
  `ab2f7ca781f3e9cf617c513cf89ceb8ece98da56` at
  `docs/pressure/retained-artifacts/systemd-v3-independently-administered-34356604479.zip`
  (`sha256:33e749d46e1ff7d0904196373c8538deda66cc34eacefb3f86ffe3b0dbcfb725`). This witness proves
  neither protected capability-observation production routing nor OIDC exchange, provider contact,
  secret materialization or delivery, Step 8, V12.2, or general agent/repository governance.
  The earlier Launcher witnesses do not derive `ProtectedLauncherCapabilityIdentity`, execute the
  Core endpoint workflow, contact OIDC or a provider, materialize or deliver a secret, establish
  provider compatibility, or prove general agent/repository governance. A later Root boundary proof
  [34283728831](https://github.com/ota-run/authority-launcher/actions/runs/34283728831), at exact
  Launcher `fa842cdeb2ede58ff726c3a60dd33eaa47e1a7bd`, ran exactly one Linux privileged
  `retained_observation_derives_and_rejects_live_substitution` regression under the requested root
  systemd scope. It proves bounded retained protected-capability derivation plus refusal of a valid
  substituted expected Launcher-invocation identity before replay reservation, capability derivation,
  or Attestor signing. It does not prove production Core-to-Launcher observation transport, public
  projection reconciliation, production observation-service or accepted-session provenance, OIDC,
  provider contact, secret materialization or delivery, positive provider evidence, or general
  governance. The endpoint profile and semantic observation verifier remain crate-private. The
  subsequently closed protected Linux/X64 compatibility gate required the raw capability identity
  to remain in the protected launcher transaction while the workflow verified one exact signed,
  closed public observation projection against a fresh canonical one-use public challenge and
  required refusal on replay, substitution, duplication, staleness, or signature/schema mismatch.
  Core must load the matching projection verification key only from the fixed administrator-owned
  verifier record reconciled with protected installation evidence; it cannot trust a workflow- or
  projection-supplied key. Root launcher state owns atomic challenge reservation and consumption;
  Core owns its expected challenge for the exact workflow invocation. The fixed local request also
  binds the expected protected Launcher invocation identity; the Launcher must recompute it from its
  accepted invocation and refuse mismatch before replay reservation, capability derivation, or
  Attestor signing. That identity remains protected and is excluded from the public projection and
  workflow-visible output. The Step 7 batch implements that feature-gated service route:
  it loads the independently installed `ProtectedLauncherAuthorityContextV1`, prepares the exact
  stopped Ota child and transient cgroup, retains the accepted Unix session and authority stores,
  delegates only public-projection signing to the protected Attestor, confirms cleanup, and returns
  only the signed public projection. Core issues the fresh challenge, sends the closed
  invocation-bound probe over the fixed root socket, reloads the verifier and its public
  installation binding, and reconciles the exact response. The endpoint observation now binds the
  verified public projection identity rather than the raw capability identity. Launcher first
  committed the route at `2d7d8a1e82753943059e4999bb6ec3a7a569b896`; Core committed reconciliation
  at `e5f5080bb0081f47077d0e2e73d90a8f3d7fa392` and extended only the hosted compilation timeout at
  `5db2564c538e1c66d5f2bb8a854e788557c5e4ac`. Hosted runs `34379005208` and `34379290838` exposed,
  respectively, a missing runner toolchain and the original five-minute timeout. Run `34381913282`
  then compiled and executed the exact transaction but refused with `LocalBoundaryUnavailable`
  because the canonical provisioner had not installed `/etc/ota/secret-delivery`. Launcher repairs
  that ownership boundary at `ec04db2dac7ad73e13dc38ec34cf78edf76cc8ee`: it installs fixed root-owned
  empty structural verifier and binding snapshots, includes the directory in fresh-state
  reconciliation, and grants no provider authority. The Core gate now binds that exact Launcher
  revision, Protocol `58526f3f29299873e345352963e30b8a1677044f`, and the run's exact Core
  `github.sha`. At that point Linux runtime proof remained open and provider contact stayed blocked
  until the exact committed Linux/X64 workflow succeeded with its substitution, replay, staleness,
  verifier, signature, target, revision, and cleanup controls retained. No OIDC request, provider
  contact, materialization, delivery, Step 8, or V12.2 capability is activated.
  Runs `34386901632` and `34390181756` subsequently executed the exact transaction after canonical
  authority-store provisioning and localized the remaining refusal to retained authority-context
  acquisition. Separate systemd reproduction on the protected host established that the immutable
  V3 combination `ProtectProc=invisible` plus `ProcSubset=pid` removes the required
  `/proc/sys/kernel/random/boot_id`. A bind-only exception also refused because systemd applied the
  proc subset before resolving the bind source. Widening to `ProcSubset=all` is rejected because the
  forked selected child would inherit the Launcher's mount namespace.
  Step 7's V4 correction was independently reviewed and activated at Core
  `7a86c9313921ba012fb966c527e9c901a2b18e66`. Protocol
  `d16947b87d84e66a4164a275c703b47fce6ded20` preserves V3 historically and adds the immutable V4
  profile, exact inherited-listener and boot-descriptor roles, profile substitution refusal, and
  the V4 public projection class. Launcher `d690b8beb0c775a9d4ad7c944a150de895b1af67` retains both
  proc restrictions while systemd PID 1 passes the exact boot-ID file as one named read-only
  `OpenFile=` descriptor; the protected history and broker units retain their explicit
  `ProcSubset=pid`, and the Attestor remains unchanged. Root Boundary run
  [34417605017](https://github.com/ota-run/authority-launcher/actions/runs/34417605017) passed at that
  exact Launcher revision, including the privileged retained capability derivation. The first
  independently administered VPS run
  [34418027937](https://github.com/ota-run/authority-launcher/actions/runs/34418027937) then refused
  before authorization because Core still required V3; it is failure localization, not V4
  compatibility evidence. Core migration `5e4b2438422ae7e0f54c063ee6fbcfb585c2965e` pins the exact
  Protocol revision, requires V4 for live broker and capability-observation admission, preserves
  exact historical V3 archive re-verification without making V3 loadable for live admission, and
  binds the endpoint gate to the V4 public class. After an exact VPS reprovision, Launcher run
  [34422820241](https://github.com/ota-run/authority-launcher/actions/runs/34422820241)
  completed the governed invocation at exact Launcher
  `d690b8beb0c775a9d4ad7c944a150de895b1af67` and Core `5e4b2438`; Core endpoint run
  [34422827427](https://github.com/ota-run/ota/actions/runs/34422827427) then refused before
  capability derivation because the replay opener prohibited systemd's administrator-owned mount
  transition at `/var/lib/ota/authority-launcher`. Launcher
  `dac375dc323d8aa30be2e23db033b89647d5e331` admitted that one observed transition, but immutable
  Root Boundary run
  [34425445450](https://github.com/ota-run/authority-launcher/actions/runs/34425445450) executed the
  exact new test once and localized another distribution-specific mount transition earlier in the
  same fixed path. It is failed portability evidence, not compatibility proof. Launcher
  `017d866bf9aab8b193ef0ce0515acc95545f3399` now walks only the fixed absolute replay path with
  raw lexical alias refusal, no-symlink and no-magic-link resolution, root ownership and
  non-writability checks on every component, and exact `root:root 0700` final-directory validation;
  all replay record operations restore no-mount-crossing descriptor-relative resolution beneath
  the retained final descriptor. Both focused tests passed directly on the protected Linux/X64
  host. Root Boundary run
  [34431466395](https://github.com/ota-run/authority-launcher/actions/runs/34431466395), job
  `102727695360`, then passed at that exact Launcher revision, including the privileged retained
  capability derivation and production replay-store checks. After fresh provisioning at exact Core
  `1dee11e4bfaae4a335228d1fb76573dde9f65353`, Launcher `017d866b`, and Protocol `d16947b8`, Core
  endpoint run [34433055859](https://github.com/ota-run/ota/actions/runs/34433055859) completed the
  protected observation transaction and emitted one valid signed projection, but the workflow
  rejected it because Rust's test harness prefixed the marker line with the test name. The Core
  workflow repair at `f2db350fee630ce2ad1701bd9b5272933d6c7c1e` extracts exactly one marker
  from either harness placement and self-tests zero and duplicate refusal. Exact run
  [34439988695](https://github.com/ota-run/ota/actions/runs/34439988695) passed when GitHub supplied
  the retained shard-`1` profile, while prior exact run
  [34434534789](https://github.com/ota-run/ota/actions/runs/34434534789) completed capability
  observation and then rejected the live endpoint as outside that profile. Root-only mutable Runner
  diagnostics attributed the mismatch to service shard `3`; that operator observation is not
  immutable hosted artifact evidence. The two outcomes nevertheless expose that freezing one
  service shard is not a stable compatibility boundary. Core now admits only the East US service
  family with a canonical positive shard, still binds the exact observed host into each observation,
  and refuses zero, padded, overflowing, alternate-region, suffix, scheme, port, and authority
  substitutions. Exact Core run
  [34443349694](https://github.com/ota-run/ota/actions/runs/34443349694), protected job
  `102762828509`, then passed at `a93d65650e6d74167515fedc43a62ea9cc657489` against service shard
  `3`, with signed V4 capability derivation verified and the bounded endpoint artifact published.
  Its exact endpoint and no-token Toolkit ZIPs are retained under
  `docs/pressure/retained-artifacts/` with SHA-256 values
  `0abd5bfe6565bba8ac768205f05c9675db8c62167fa588cd44fad48300e69571` and
  `d525e5284463e96567219a4c5a6e05fd7ca056adf4cc9ff2506b1ae5c21acd26`.
  Launcher run
  [34443805276](https://github.com/ota-run/authority-launcher/actions/runs/34443805276), job
  `102764136034`, separately completed one governed invocation against the same installation with
  exit `0`, all four terminal cleanup conditions, and one valid protected receipt archive with zero
  invalid archives. Launcher `bf4e6d994d964aef0f5bcadec71d1b5cd53f63e6` durably retains that
  exact eight-file ZIP. These runs close the bounded protected-capability and endpoint-compatibility
  gate only. No real GitHub OIDC request, provider contact, materialization, delivery, positive
  provider evidence, Step 8, or V12.2 capability is activated.
  The signed observation service remains compatibility evidence because it creates a separate
  refused probe child and cgroup; it cannot authorize a later provider-requesting child. The
  independently reviewed private authority-snapshot bridge now closes the Core-owned provider-free
  reconstruction boundary. Protocol `d1d1fd4` supplies the record-only prerequisite. Core
  `15f59d87` adds request/response reconciliation, `8c22a18a` verifies the signed protected
  snapshot and retained bytes, and `f6d4633e` reconstructs one exact Step 1-6 transaction candidate
  from the challenge-backed invocation context and canonical selected `RunPlan`. Core independently
  re-derives the selected roots, binding/source, profile, implementation subject, provider tuple,
  effects, protected policy authority, evaluation, and dry-run plan. It rejects substituted
  authority, context, workflow identity, graph semantics, or provider truth and selected roots
  outside the exact native Linux transient structured-command boundary. The batch passed focused
  release compilation and regression suites, first-party synchronization, and independent review.
  No Site, Skills, Examples, Learn, FAQ, Glossary, contract-schema, or public-JSON propagation is
  required because the bridge remains crate-private and provider-free.
  The independently reviewed provider-free same-child bridge is now proved through the exact
  protected Linux/X64 service path. Protocol
  `2c46cb676ef6e0844312bd6a157adec7ccb54de1`, Launcher
  `a00f0e1ebd0ba886015a1aa32216ad30b56dcd9a`, and Core
  `c071fed09ffe42d4ac00979d16dff0967ad208aa` bind one selected child's signed observation,
  protected snapshot, reconstructed Step 1-6 candidate, and one-use V2 transaction on the same
  inherited startup session. Core run
  [34933139633](https://github.com/ota-run/ota/actions/runs/34933139633), protected job
  `104265387910`, passed that full provider-free route. It executed exactly one ignored Linux
  capability regression; reconciled one observation, prelude, snapshot, and V2 response; recorded
  zero protocol mismatch/refusal counters; refused exactly once at the expected provider-free
  boundary; did not start the selected task; retained no protected capability identity in public
  output; and completed child reap, scope removal, cgroup cleanup, and active-slot removal. The
  separate Toolkit job `104265370625` proved only that pinned Toolkit code used its supplied
  loopback request URL with no OIDC capability at action start. The administrator-retained VPS
  bundle passed its checksum manifest but is not a publicly durable GitHub artifact. No real GitHub
  OIDC token request, provider contact, Google STS/WIF or Secret Manager operation,
  materialization, injection, positive provider evidence, Step 8, or V12.2 capability is activated.
- Vinicius' independent v1.6.27 source review confirmed that negative-control projection
  reconciliation is live on runtime proof, emitted-archive verification, and Doctor archive
  loading. It also exposed a narrower adversarial-test gap: the existing digest mutation was
  malformed rather than a valid sibling attestation. Core now retains the malformed case as
  structural coverage and adds a non-default test-only live-transaction fault boundary. Two
  declared controls execute through normal Core derivation in one proof run; substituting the
  sibling's valid digest after projection creation refuses at the production reconciler before
  terminal proof output, receipt, or archive publication. The shared Core and standalone Examples
  runtime-proof fixture now uses bounded dependency-readiness retries so the same transaction is
  reliable on hosted Linux and macOS without weakening failure semantics; the standalone copy is
  pinned at `ota-run/examples@6ab6635237aba8172812406499186a7fbc8a16cd`. Release Gate run
  `33737308315` then proved the reference-script retry was not the complete hosted fix: Windows
  passed, Ubuntu's fault step was cancelled, and macOS reached reconciliation without a selected
  projection after its detached fixture services failed to retain the green obligation. The
  feature-gated transaction regression now owns both fixture servers as retained test children,
  verifies their readiness before invoking Ota, keeps Ota's selected runtime child and production
  reconciliation path intact, and performs terminal cleanup. Release Gate
  [run 33744200940](https://github.com/ota-run/ota/actions/runs/33744200940) passed at exact Core
  `876680f074777aada0b3910a62aab9b245b34af7`: Ubuntu job `100612831644` and macOS job
  `100612831722` both passed the dedicated live substitution boundary, and the complete Windows,
  Ubuntu, and macOS gates succeeded. The hosted control proves that a real canonical sibling
  attestation digest substituted after projection creation is refused by production reconciliation
  before terminal proof output, receipt, or archive publication. It does not prove provider
  behavior, external mutation prevention, or positive proof assurance. Site, Skills, Learn, FAQ,
  Glossary, and standalone Examples remain unchanged by this test-harness correction because it
  changes no operator behavior, public vocabulary, or shared reference content.
- Discord contract pressure activated the aggregate mode-eligibility follow-on as a bounded V12.1
  correctness repair. Core now derives aggregate mode admission, task-discovery availability,
  agent-safety closure, task/workflow replay-input admission, managed CI projection, and
  task/workflow Doctor prerequisite selection from the existing backend-selected runner plan
  instead of contaminating the selected invocation with dependencies from unselected mode
  branches. Possible outcome hooks retain their own runtime backend and provenance rather than
  inheriting a caller override that execution would not apply. Projection failures use typed
  internal classifications rather than parsing display text. Core dry-run plans, run receipts,
  and managed CI projections bind the ordered selection through one SHA-256 selected graph
  identity. Each selected occurrence binds a digest of its resolved execution semantics; selected
  workflow-required services and their transitive definitions, plus requested lifecycle, host-port,
  and memory overrides, participate in the graph identity.
  Distinct workflow-phase roots and their edge endpoints retain separate invocation
  identities, including when phases reuse the same task. The Discord contract now makes
  `DISCORD_TOKEN` task-scoped and moves
  `setup:env:local` under the native `setup` mode branch while preserving explicit workflow host
  preparation. Selected task execution and dry-run reporting no longer read or expose optional
  host dotenv truth unless a selected occurrence declares the corresponding env obligation. A
  disposable clean fixture passed real container CI without `.env.local` and real native CI with
  `setup:env:local`; the live contract's dry-runs select those same distinct chains and omit
  `DISCORD_TOKEN` from container admission evidence.
  Persisted receipt archives now retain that canonical graph, reconcile its typed receipt input,
  and re-derive it from the immutable contract snapshot; older archives without reconstructable
  graph truth remain inspectable only as `legacy_unverified` and cannot become baseline, proof, or
  authority inputs. The first independent review found four remaining inventory reopenings;
  successful receipt env evidence, Doctor env reporting, occurrence-aware replay-rule scope, and
  workflow service summaries now consume the selected graph, with focused adversarial regressions.
  Lifecycle and runtime proof add assertions, seam observers, and negative controls as explicit
  canonical graph roots rather than rebuilding a task-name closure. Adversarial regressions also
  keep unselected mode branches out of strict replay snapshot creation, service cleanup, sandbox
  diagnosis, uv provenance findings, and workflow env artifact consumer evidence. Final
  independent selected-graph re-review passed. Native pressure additionally found that a declared
  Corepack package manager could resolve through an ambient native shim; the reviewed Core repair
  routes structured task commands and typed Node hydration through Corepack before any declared
  exec-mode orchestrator wraps them. Immutable propagation is Site
  `16b75bf531bfc788eb41a9e095a071b45cdca351`, Skills
  `f981911f2c21eb35e97fff6e077971e90aa4df1b`, and Examples
  `a565b22ae86e1840c9ecfe7fb97dd0c0bff9cc4b`; this Core reconciliation records the Site and
  Skills revisions against reviewed Core `a6b66b59ef255acb66726f022b572c41b6df5dd8`. The
  standalone Node service example materializes one host-only prerequisite for native setup while
  container verification omits it. Learn, FAQ, and Glossary remain unchanged because this repair
  adds no contract syntax, public term, or operator workflow.
- Eris adoption pressure exposed and repaired two bounded Core defects without changing the
  released `v1.6.27` runtime used by that partner contract: detector output no longer promotes
  named GitHub Actions bodies containing unresolved matrix or shell expressions into runnable task
  truth, and diagnose-only Rust toolchains must use comparable semantic requirements while
  rustup-owned run fulfillment retains channel names. Site and Skills carry the same distinction
  at immutable revisions pinned by Core. The same pressure review exposed a task-discovery UX gap:
  `ota tasks --use`, including `ota tasks --safe --use`, now renders task `notes` alongside
  descriptions and runnable commands so agents see declared proof limits and external-boundary
  guidance before selecting a lane. JSON already carried the notes and remains unchanged. Five
  remaining non-blocking follow-ons are recorded in the
  [execution-contract follow-on plan](../planning/execution-contract-follow-ons/plan.md):
  lock-strict Cargo hydration, mixed-mode preview selection, Doctor
  cause reconciliation, opaque-shell evidence boundaries, and identity-bound container cleanup.
  The remaining inactive items do not delay the Eris draft or interrupt V12.1; each requires a
  future version owner and independent activation.
- Anodizer onboarding pressure exposed two detector-fidelity defects. The repaired detector omits
  Taskfile helpers whose names begin with `_` or declare `internal: true` from executable contract
  truth and collapses repeated callers of the same reusable GitHub Actions verification step before
  collision disambiguation.
  A source-built reproduction at Anodizer `112a61a22557fa7f407bded897a8ec6671e35ae3`
  reduced the inferred task catalog from 50 to 46 entries, retained distinct verification lanes,
  and emitted neither private helpers nor caller-specific hash-qualified duplicates. Site and the
  canonical Skill carry this authoring boundary. Examples, Learn, FAQ, and Glossary remain
  unchanged because the repair adds no contract shape, command, public term, or operator workflow.
- V12 closure: implementation-order steps 1-10, the bounded real-repository pressure bar, and
  independent closure reconciliation are complete. Plausible and Outline retain exact
  selected-closure denial with
  `execution_started: false` plus independent setup, provider/database precursor,
  worktree/child-command, and outcome-hook absence witnesses; captured closure evidence records no
  selected service. The final corrected typed
  `ota up --dry-run` control is immutable-hosted in
  [run 33382559640](https://github.com/ota-run/ota/actions/runs/33382559640) against Core
  `a5aae10f5ce33e0d0927dbb913a685505933145b`. The archive-derived
  `effect_assurance` candidate remains schema-v5, `unknown`, reconciliation-bound, review-only,
  and platform-stably non-writable. Provider execution or mutation, callback behavior after Core's
  ordered delivery, independently administered policy authority, positive assurance, arbitrary
  child-process absence, repository-wide immutability, database correctness, and public archive
  export safety remain unproved or outside V12. V12 closure did not implicitly activate V12.1;
  the separate activation record in the V12.1 plan owns the new work.
- Core now owns `docs/pressure/evidence-manifest.json`, a machine-readable registry that binds
  retained pressure cases to exact revisions, matrices, exercised surfaces, proven facts, and
  explicit limits. The Site commits a generated discovery projection and validates it against Core
  through `ota run pressure:evidence:site:check`; it is not a certification, endorsement, or
  green-badge surface. The public index, Glossary, FAQ, Skills protocol, and engineering notes
  follow the same source-of-truth rule. Learn and Examples are unaffected because this is evidence
  accounting, not an operator contract workflow.
- Immutable Linux/x64 and macOS [run 33382559640](https://github.com/ota-run/ota/actions/runs/33382559640)
  against exact Core `a5aae10f5ce33e0d0927dbb913a685505933145b` closes the repaired typed
  `ota up --dry-run` admission control. Both retained artifacts show one admitted application plan,
  an explicit typed deny, `BLOCKED`, `execution_started: false`, and only the refusal action. The
  same jobs pass Core-owned plan-to-executor and sandbox-admission continuity controls. The exact
  selected fixture declares and immediately verifies absence of its setup sentinel, rendered
  environment artifact, proof artifact, durable-log path, and dependency command sentinel. This
  remains internal, provider-disabled evidence only; it does not prove provider contact or
  mutation, arbitrary child-process absence, repository-wide immutability, database correctness,
  positive assurance, or archive export safety.
- Final V12 internal Linux/x64 and macOS pressure is green in
  [run 33335973677](https://github.com/ota-run/ota/actions/runs/33335973677), bound to Core
  `25afb2b510a13ce149a2e9aa8ed5418a7af69482`. Both retained artifacts complete the selected typed
  refusal, policy/CI/sandbox, canary, archive, Doctor assurance, review-only candidate, stale-input,
  and contract-alias stages. They observe absent fixture setup, environment-rendering, proof-artifact,
  and durable-log paths after refusal. This does not prove provider contact or mutation, arbitrary
  child-process absence, repository-wide immutability, database correctness, positive assurance, or
  export safety.
- PythiaLabs pre-release fork pressure has reviewable declaration, native execution, and an
  Ota-owned Linux Node-container matrix. The declaration run passed; native execution passed ten
  of eleven lanes and retained Pythia's existing site-format failure as a repository finding; the
  container run passed the bounded MCP and site hydration/build closures using a digest-pinned Node
  image rather than Pythia's mutable Dockerfile. That exercise repaired Elixir version parsing when
  `elixir --version` also prints Erlang/OTP. Credentialed CAEP, merge, communications, and the
  Liminal lifecycle remain unmodeled. See `docs/pressure/pythialabs-discovery.md` for exact
  revisions, hosted evidence, and limits.
- V12 has bounded immutable real-repository refusal evidence: Linux/macOS fork matrices against
  Plausible Analytics and Outline each validate a committed PostgreSQL migration root, then pass
  direct task and workflow effect-refusal canaries with `execution_started: false` and an explicit
  typed deny. `ota doctor --json` now emits contract-graph coverage records for each declared
  typed-refusal canary, including unchallenged exact-equivalent attachments and opaque-path
  boundaries; it remains `unknown` until a later verified archive carries realization and execution
  evidence. A later Linux/macOS matrix against Core `84a988433cb3c0226a3569cdc2ee5202d3d5d375`
  confirms that those exact-equivalent and opaque paths remain unproved and that a generic caller
  refusal cannot pass a canary. Mixed-realization admission retains a declared-only
  attachment as an ineligible realization, binds its distinct identity into the decision, refuses
  ordinary execution and dry-run preview before its command body, reports the preview as `BLOCKED`
  without fabricating policy evidence when no pack exists, and reports an exact-origin canary as an assurance
  gap even when another attachment reaches the same effect. The internal mixed-realization carrier
  is immutable-hosted on Linux/x64 and macOS in
  [run 33300446201](https://github.com/ota-run/ota/actions/runs/33300446201) against Core
  `974caf686a45093587058ea140b82f1a81c0fa70`: both retained artifacts bind one shared effect
  identity, distinct eligible and ineligible attachment/realization identities, a blocked preview,
  and a declared-only `effect_canary_realization_ineligible` assurance gap with
  `execution_started: false`; neither command sentinel exists. This remains an internal,
  provider-disabled control. A later immutable Linux/macOS run
  [33301627289](https://github.com/ota-run/ota/actions/runs/33301627289) against Core
  `e7682a62287b173edaa8e2a18f57fc1593359dec` adds two equal local migration sets under distinct
  canonical resource namespaces: their effect and attachment identities remain distinct, an empty
  namespace authority refuses validation, and an exact primary-only policy deny does not select the
  secondary namespace. The same policy bytes retain one snapshot identity but produce distinct
  repository-controlled and caller-selected source evidence and decision identities. Rewriting an
  archived caller-selected decision as repository-controlled changes history from `1` valid / `0`
  invalid to `0` / `1`; restoring its original bytes returns it to `1` / `0`. This remains an
  internal, provider-disabled control. A subsequent immutable Linux/macOS
  [run 33302123045](https://github.com/ota-run/ota/actions/runs/33302123045) against Core
  `d0178b2013efd3d12f6baa0a94bb572f162c70a7` exercises the exact
  `ota ci projection --expect-identity` re-evaluation used after provider checkout. A changed
  policy makes the current projection identity differ from the rendered compatibility projection
  and returns `effect_policy_denied` with an explicit typed deny before workflow setup or durable
  logs. Both retained artifacts also pass the Core plan-to-executor substitution regression. This
  is still an internal, provider-disabled control. A subsequent immutable Linux/macOS
  [run 33303689321](https://github.com/ota-run/ota/actions/runs/33303689321) against Core
  `49a1a486a4431749ff33ec50ea4265afbc2a64f2` retains task and workflow typed-deny capability
  lanes with refused preflight and `provider_execution: disabled`, plus a typed-warn lane that
  remains refused as provider-disabled. Both artifacts pass the Core task/workflow retained
  command-admission sandbox control. This proves neither provider contact or mutation nor
  authoritative sandbox enforcement. The existing private workflow refusal archive now upgrades
  Doctor's V11.14 `effect_refusal_assurance` only for an exact current-contract workflow challenge
  with matching eligible attachment, realization, explicit typed deny, and pre-execution posture;
  task-only, stale, invalid, ambiguous, or mismatched evidence remains `unknown`. Immutable
  Linux/macOS [run 33309358828](https://github.com/ota-run/ota/actions/runs/33309358828) against
  Core `81c25e09c833559312e9cd43ce04a1c63f27d6fa` now proves that exact workflow-only promotion
  from a verified private archive and its fallback to `unknown` after archive tampering. It remains
  internal, provider-disabled negative evidence, not provider contact, mutation, positive
  assurance, or export safety. The final fork-only Linux/macOS matrices against Plausible and
  Outline are immutable-hosted in [run 33391482073](https://github.com/bobaikato/analytics/actions/runs/33391482073)
  and [run 33391486538](https://github.com/bobaikato/outline/actions/runs/33391486538) at fork
  revisions `fa24db238dae39a277e5fbfc08519488a32c1020` and
  `58b6a7731aff1a1237da1d9ade6021114b0a1c6e`. Every retained artifact binds clean source-built
  Core `e96cad13db9e4289c0985fca2ce6d8353a896da4`. Both create and verify one private workflow
  archive, promote only its exact workflow-only claim to `supported`, return it to `unknown` after
  context stripping, publish a projection-free `unknown` candidate, refuse its write attempt,
  emit a reconciliation-bound existing-declaration no-op, and refuse migration drift and a
  symlinked contract without publication. Their task and workflow canaries additionally retain
  absence of selected provider/database precursor, worktree/child-command, and outcome-hook
  sentinels; workflow canaries retain setup-sentinel absence, and closure evidence records no
  selected service. This is still selected-lane, provider-disabled evidence: it does not prove
  repository-wide readiness, actual provider/database behavior, database correctness, arbitrary
  child-process absence, complete repository immutability, positive assurance, or export safety.
  See `docs/pressure/v12-real-repository-effect-refusal.md`.
- completed version: V12 effect-bound refusal assurance. V11 is reconciled complete and V12 was
  the sole active version from 2026-08-25 through its closure. The first local implementation
  batch added strict PostgreSQL
  resource bindings, discriminated database schema-mutation definitions, exact task attachment
  origins, and separate JCS/SHA-256 domains for resource, consequence, attachment, evidence, and
  realization identity. Contract validation rejects ambiguous namespace authority, unresolved or
  duplicate references, action/bounds substitution, noncanonical or non-printable migration paths,
  Unicode or otherwise non-profile namespace components, and malformed identities. Authored migration
  content identities remain expected declaration truth rather than observed byte evidence. This
  batch includes a local, execution-disabled PostgreSQL schema-mutation action carrier: it
  captures the declared migration set with explicit entry-count, per-file, and total-byte limits.
  Unix capture retains no-follow directory/file handles; non-Unix execution refuses because an
  equivalent race-safe traversal is not implemented. The adapter requires its manifest identity to equal
  `migration_set.content_identity`, and derives a domain-separated application plan bound to the
  exact task attachment, contract invocation origin, repository-relative effective working directory,
  and effect realization. Dry-run publishes that non-secret plan. For repo-level `ota run` and
  `ota up`, including `ota up --dry-run`, one command-level typed preflight admits and verifies every
  typed action in the selected closure before
  command-scoped replay-input policy loading, agent/crossing/sandbox admission, workflow-environment
  artifact rendering, or durable-log preparation. It re-observes source truth through retained
  no-follow descriptors and returns the provider-disabled refusal before task conditions, required
  services, dependencies, shell dispatch, provider contact, or repository mutation. The runner
  repeats the same check as defense in depth for direct/internal callers. `ota proof runtime` and
  `ota proof lifecycle` now directly admit the complete selected proof closure, retaining phase
  and proof-helper invocation role/order without unselected-mode dependencies, before replay,
  crossing or sandbox admission, proof artifacts, service work, or child startup; a typed deny
  returns `OTA_EFFECT_POLICY_DENIED` with `execution_started: false`. `ota up --dry-run` carries the
  admitted non-secret plans and active decision in its blocked preview without starting work; other
  read-only command diagnosis and policy discovery are not claimed to occur after this boundary.
  Validation refuses mode or OS-variant execution-body overrides so runtime selection
  cannot replace the previewed typed action. Non-dry-run `ota up` emits its ordinary blocked
  readiness receipt with `execution_attempted: false`; the typed adapter emits no positive effect
  or execution receipt, archive, success claim, agent-safe authority, canary, or positive assurance.
  A non-dry-run `ota up --json` typed deny additionally retains the exact command-scoped decision
  as `receipt.typed_effect_policy_refusal` with `execution_started: false`. Ordinary refusal remains
  non-durable. Explicit `ota up --workflow <name> --archive-effect-refusal --json` creates one
  create-new receipt archive plus immutable contract and private policy snapshots only for an
  explicit typed deny. History independently re-derives the selected invocation closure,
  application plans, policy snapshot, and decision; missing, aliased, contradictory, or changed
  evidence invalidates the archive. This remains negative evidence, not provider execution,
  mutation proof, positive assurance, or a public export profile. Post-publication directory-sync
  failure returns `effect_refusal_archive_durability_uncertain` with `published: true`, the exact
  archive path, and recovery guidance; it never claims that no artifact was written. Failures
  after a receipt is durably published, including archive verification or retention pruning, return
  `effect_refusal_archive_post_publication_failed` with `published: true`, `durability: "confirmed"`,
  and the retained receipt path.
  Policy or contract snapshot sync uncertainty is separately identified with
  `effect_refusal_snapshot_durability_uncertain`, its exact `artifact_kind`, and
  `receipt_published: false`.
  The explicit archive carrier is immutable-hosted on Linux/x64 and macOS in
  [run 33243471896](https://github.com/ota-run/ota/actions/runs/33243471896) against Core
  `f8dc16c0eb3d8e5c9d9fda5c2f50674e2ba1b150`. Both retained artifacts record one valid refusal
  archive with `archive_count: 1` and `invalid_archive_count: 0`, then reject an archive with its
  replay context deliberately stripped (`0` valid, `1` invalid), and accept the restored bytes.
  The same fixture confirms workflow setup, environment rendering, and durable logs remain absent.
  It does not establish provider contact or mutation prevention outside Ota, positive assurance,
  or export safety.
  Archive-backed Doctor assurance is separately immutable-hosted on Linux/x64 and macOS in
  [run 33309358828](https://github.com/ota-run/ota/actions/runs/33309358828) against Core
  `81c25e09c833559312e9cd43ce04a1c63f27d6fa`. Both retained artifacts promote one exact
  workflow-only challenge to `supported` through private archive reconciliation, then return it
  to `unknown` after archive tampering. This remains internal, provider-disabled negative evidence;
  it does not prove provider contact, mutation prevention outside Ota, positive assurance, or
  export safety.
  Task and workflow harness capability JSON now carries mandatory typed effect-policy posture:
  untyped lanes are `not_applicable`, while typed lanes carry the evaluated decision, policy
  snapshot, selected execution graph, and effect-set identities or an explicit unavailable state.
  Only untyped `not_applicable` lanes appear under `callable_tasks` or `callable_workflows`; typed
  lanes remain under refused capabilities with `provider_execution: disabled`. The published
  schema requires evaluated deny, allow/warn, and unavailable posture to carry their matching
  refused preflight reason. Live
  sandbox admission consumes the exact closure, retained policy snapshot, application plans, and
  verified decision produced by command admission rather than re-planning under a second origin.
  Missing policy truth or aggregate denial refuses before canonical sandbox-policy construction and
  provider-capability evaluation, while allow/warn does not become provider authority. Malformed
  effect policy does not affect an untyped capability lane. Sandbox application evidence and archive
  semantics are unchanged because typed provider execution remains disabled.
  Provider execution, independently administered policy admission, provider-side mutation
  semantics, and independent real-repository effect pressure remain unproved. The bounded internal
  carrier is now immutable-hosted on Linux/x64 and macOS in
  [Smoke run 32994303400](https://github.com/ota-run/ota/actions/runs/32994303400) against exact Core
  `73f7fea9fb76af514e6a97e42562d30b683768ad`. Both retained artifacts bind that revision and prove
  contract validation, typed run/up previews, direct run/up/proof refusal before setup and environment
  rendering, blocked-receipt schema conformance with `execution_attempted: false`, stale migration-byte
  refusal, and intermediate-symlink escape refusal. Every execution/refusal status is exactly `1`,
  and neither setup sentinel, rendered environment, nor durable execution logs exist. This internal
  fixture does not contact a provider, execute a repository task, prove provider mutation semantics,
  or substitute for later independent real-repository pressure. The
  foundation is committed at Core `f3d4b8e1`, Site `5926f69`, Skills `d05b1d1`, and Examples
  `2dad574`; canonical-identity hardening is committed at Core `1b9a03d6`, Site `e78b963`, and
  Skills `30c8dbd`. The hosted carrier above exercises the selected plan/admission path but does not
  replace the exhaustive local identity-domain regression matrix. V12.2 onward and the authority
  distribution plan remain planned and inactive; adapter/profile conformance constrains the named
  V12.1 implementation without establishing support. The reference Example correctly requires Ota `1.6.27`; the source-built
  `v1.6.27` development binary validated that exact contract locally on 2026-08-25. This closes
  only the local minimum-version gate. Application plans now carry the canonical discriminated
  action bounds needed by an executor, rather than requiring contract reconstruction. A Core-owned,
  test-only continuity control re-verifies the admitted plan and retained migration bytes, then
  delivers every ordered file to the selected callback before Core records a delivery
  acknowledgement. Substituted source or plan and disconnected or failing callbacks refuse. The
  callback remains trusted for its behavior after delivery, so this does not enable provider contact,
  mutation, execution evidence, or positive assurance. The next local V12 batch extends the existing
  command-scoped policy-pack loader with canonical typed effect rules and one shared evaluator below
  CLI orchestration. Its content-addressed decision binds the policy snapshot, redacted source
  location, source kind and authority posture, selected invocation and execution graph, effect and
  realization sets, every matching typed rule, current coarse-effect components, and aggregate
  `deny > warn > allow` precedence. Repo-level `ota run` and `ota up`, including read-only
  `--dry-run`, consume the selected-closure decision before replay, authority, sandbox, setup,
  environment rendering, services, dependencies, provider contact, or repository mutation. A typed
  rule, strict fallback, or coarse component whose aggregate is `deny` causes
  `OTA_EFFECT_POLICY_DENIED`; dry-run publishes the non-secret plans and decision without starting work.
  Caller overrides remain limited to shipped coarse selectors and cannot target typed rules. This is
  operational refusal only: provider execution, positive effect/execution receipts and archives,
  and positive assurance remain disabled. Plan-to-executor continuity is now
  immutable-hosted on Linux/x64 and macOS in
  [run 33032683375](https://github.com/ota-run/ota/actions/runs/33032683375) against exact Core
  `32e3395f92e1114ce209dc620d14ecc82330856f`. Both retained artifacts bind that revision, record all
  seven admission/refusal stages and all seven side-effect-absence checkpoints, retain status `1`
  for run, up, inherited proof, stale-input, and intermediate-symlink refusals, publish canonical
  action bounds and ordered migration manifests, and pass the Core-owned continuity control. This
  remains an internal execution-disabled fixture; no provider behavior or independent
  real-repository effect pressure is proved. Contract validation now
  also rejects a released `agent.bootstrap.ota.source.version`
  below `metadata.ota.minimum_version`; `source: contract` is the canonical CI consumer so a
  workflow cannot maintain a divergent released bootstrap version. Git revisions and pressure
  branches remain intentionally incomparable to a release floor. The connected typed-adapter
  propagation is committed at Site `00a2f60729e264dc4806b1698b475576a3a58a93`, Skills
  `8ef5d9e2fe3a31010c9bb0af534114d390c52a3d`, and Examples
  `6634e1509ebcbbfef652e084b8f3982fc5fa0dda`; the unrelated generated Site pressure note remains
  outside this batch. The plan-to-executor reference propagation is committed at Site
  `cb1f21463abcc5b1e866ea87aafc1c31bdfc7729` and the reviewed implementation foundation at Core
  `32e3395f92e1114ce209dc620d14ecc82330856f`. Typed effect-policy refusal is committed at Core
  `d72c6c85`, with immutable admission-pressure harness `212446c000b55d68bad5906a4b532ce5055c1477`.
  [Run 33067741989](https://github.com/ota-run/ota/actions/runs/33067741989) is green on Linux/x64
  and macOS and retains artifacts bound to that exact Core revision. Both carriers publish an
  explicit `deny` decision for the eligible schema-mutation effect, return
  `OTA_EFFECT_POLICY_DENIED` from `run`, and block `up` and inherited runtime proof at
  `preconditions` with `execution_attempted: false`. Setup, rendered workflow environment, durable
  execution logs, stale migration input, and an intermediate-symlink escape are respectively
  absent or refused. This is bounded internal-fixture evidence of pre-side-effect policy refusal;
  it does not prove provider contact or mutation, canary assurance, positive receipts or archives,
  independently administered policy authority, or the required independent real-repository
  effect-pressure bar. The composed command-admission, capability, and sandbox carrier is now
  immutable-hosted on Linux/x64 and macOS in
  [run 33199213628](https://github.com/ota-run/ota/actions/runs/33199213628) against exact Core
  `1339476f1806a14278028de95020afd7e9ef5098`. Both retained artifacts record all nine staged
  checks: typed `run`, `up`, and inherited-proof denial with `execution_attempted: false`; absent
  workflow setup, environment-rendering, proof-artifact, and durable-log paths; stale-input and
  intermediate-symlink refusal; task/workflow canary results; and the Core-owned
  delivery-continuity control. The fixture does not independently establish arbitrary
  child-process absence, provider contact or mutation, or complete repository immutability. It
  remains internal execution-disabled evidence and does not close independent real-repository
  pressure, positive receipts/archives, assurance, or independently administered policy
  authority. Between V12 feature batches,
  typed-effect ownership was extracted without
  behavior changes: `effect_admission` owns runner-independent domain verification,
  `effect_orchestration` adapts selected runner closures for CLI admission, and `runner` translates
  domain failures without a reverse dependency. The current local branch adds contract-owned
  `agent.effect_refusal_canaries` and execution-free task/workflow invocations. A pass requires one
  exact predeclared origin and eligible realization denied by an explicit matching typed rule;
  strict fallback, generic refusal, unknown IDs, caller overrides, absent origins, and non-denial
  cannot false-green it. The semantic canary identity excludes the local locator while binding the
  effect, attachment, realization, selected invocation, invocation origin, and expected typed-deny
  posture. It emits
  `passed | not_evaluated | assurance_gap | failed` with `execution_started: false`. The bounded
  carrier is immutable-hosted on Linux/x64 and macOS in
  [run 33098093213](https://github.com/ota-run/ota/actions/runs/33098093213) against exact Core
  `dc368fbb2fc298490bfce6de86ea4ed79b493beb`. Both retained artifacts prove task and workflow
  canary passes only for the exact eligible explicit typed denial, strict-fallback and unknown-ID
  non-passing outcomes, icon-free plain output, and no setup, workflow-environment, or durable-log
  side effect. This internal fixture does not enable or prove provider execution or mutation,
  positive receipts, archives, assurance, independently administered policy authority, or the
  required independent real-repository effect-pressure bar. Connected Site, Skills, Examples,
  Learn, schema, and command propagation is committed at Site `f1ef9c9`, Skills `7298340`, and
  Examples `f209c31`; Core implementation and pressure harness are committed at `dc368fbb`.
  Complete selected runtime-proof closure admission is now immutable-hosted on Linux/x64 and macOS
  in [run 33166914327](https://github.com/ota-run/ota/actions/runs/33166914327) against Core
  `0f4db8e2a19367f4cfb6d6a4522ad3b007690bba`. Both retained artifacts prove a typed policy deny
  before proof artifacts, setup, workflow environment rendering, durable execution logs, or child
  startup, while the Core-owned ordered-delivery control remains green. This is still internal,
  execution-disabled evidence; it does not establish provider mutation, positive evidence, or
  independent real-repository pressure.
- completed V11.21 enforced sandbox policy application. Core now derives one
  provider-neutral, target-platform-bound segment graph from the selected task/workflow closure,
  applies only identified monotonic policy restrictions, and fails closed before preparation when
  the first `oci_local` provider cannot enforce an authoritative selected-lane control. Compatible
  explicit-platform ephemeral container lanes receive read-only repository mounts, existing
  writable carve-outs, bounded external-network denial, pre-mutation cleanup leases, distinct
  per-invocation boundaries, initial and terminal engine inspection, engine-confirmed removal, and
  runner-authored receipt evidence. Engine inspection rejects every mount outside the exact
  repository-root and declared carve-out set. Those receipts are archived automatically with the
  normalized contract snapshot and any identified policy-authority snapshot; archive reads
  re-derive canonical policy and authority-owned overlays, reconcile completed segments with
  archived task outcomes, and reject unbound overlay, segment, edge, capability, or application
  identities. Dry-run performs no provider-backed runtime/tool probes, while real OCI precondition
  probes execute as separately identified, cleanup-confirmed `precondition_probe` invocations
  bound to the exact admitted requirement-owning segment inside the registered sandbox application
  transaction; blocking probes retain that evidence in the refusal receipt and cannot substitute
  for task-execution evidence. Declared Linux OCI `container.platform` is canonical for
  target-specific variants, inputs, environment, requirements, and ordinary/provider-enforced
  container creation, including persistent-container reconciliation. Reusing one task identity
  across multiple phases refuses rather than collapsing separate invocations. Managed isolated
  paths refuse until their durable provider resources have transaction-bound creation, retention,
  and failure-cleanup evidence. Initial independent pressure exposed Docker Desktop's
  multi-platform image metadata as the wrong platform witness: image inspection can report the
  host-native variant even after `create --platform` selected another target. Ota now binds the
  exact provider-applied create request to the created container's platform evidence, requires an
  exact match when the provider reports OS and architecture, and accepts Docker's OS-only
  container report only after successful creation with the full declared platform.
  The first adapter admits finite command bodies only; typed task bodies, requirements, services,
  conditional checks, and authoritative lifecycle-proof closures refuse rather than execute
  outside the evidenced boundary. Ota-owned `run` flags, including `--sandbox-target`, remain
  command flags when written after the task and before task inputs. `codex_local` remains compiled
  guidance, targeted egress remains unsupported by stock OCI, and raw shell outside Ota remains
  outside this enforcement boundary. Core fixture proof is green. Independent hosted pressure is
  also green against exact Core `d796f28e5556c0f1315052e8782ed774e9156922`:
  create-chrome-extension run
  [30544809360](https://github.com/bobaikato/create-chrome-extension/actions/runs/30544809360)
  proves generated output inside one writable carve-out, and Caddy run
  [30544809898](https://github.com/bobaikato/caddy/actions/runs/30544809898) independently proves
  a source-manifest artifact under external-network denial. Both runs include protected
  `ota.yaml` write refusal, discovery JSON, dry-run admission, real task and workflow execution,
  terminal cleanup, archive reconciliation, and an explicit uncovered-material-behavior
  inventory. They prove only their selected Linux/amd64 `oci_local` lanes. The full Core and
  first-party release gate is green, so V11.21 is complete.
- supplementary V11.21 pressure: Buzz run
  [30559160264](https://github.com/bobaikato/buzz/actions/runs/30559160264) confirms that the
  stock adapter refuses a real Compose, migration, persistent-volume, hydration, and external-state
  integration closure before Buzz provider or worktree mutation. This does not widen V11.21. It
  records two explicit boundaries: the clean host remains Doctor-blocked on the declared missing
  `just` tool, and pre-boundary sandbox refusal is inline receipt evidence rather than a durable
  refusal archive.
- The active-execution registry now derives runtime listener ownership across the complete selected
  closure and compares actual execution namespace, network protocol, bind/publication address, and
  host port. This closes both sides of the earlier task-name heuristic: disjoint native/container
  service endpoints and isolated write namespaces can coexist, while different task names sharing
  one listener refuse. Shared write and env-materialization ownership also detects nested path
  overlap. Existing registry entries without runtime or write-namespace identity remain fail-closed
  until restarted. One-run `--host-port` selection now participates in that same resource identity
  for direct native service tasks as well as container and native Compose publication. Direct
  native execution applies the selected port to both bind and public runtime truth, including typed
  launch arguments and canonical runtime env, because there is no separate host-publication layer.
  When one fixed host listener collides, text output now names the host port, requested and active
  execution modes, and exact owner before suggesting `--host-port <free port>` when the selected
  lane's own execution-option preflight admits that override; mixed listener/write conflicts remain
  broader active-execution failures, identify the free-port choice as resolving only
  `runtime_listener`, and name every remaining typed reason that must be resolved before retrying.
  Suggested reruns preserve agent mode. The compact summary keeps its established ordering and adds
  only `Reason`/`Reasons` and `Host port` when that conflict evidence exists.
- V11.7 audited-crossing authority is complete for its bounded OSS slice after independent review
  and immutable hardened-launcher pressure. Core derives one canonical content-addressed crossing
  scope from the selected
  task/workflow execution graph, refuses unresolved task-input identity, and supports an opt-in
  `governance.crossing_authority.authority_id` plus `ota run|up|proof runtime|proof lifecycle
  --grant <id>` admission path. Proof commands refuse before artifacts, child execution, lifecycle
  ownership, service start, or assertion execution. Proof invocation role and order, lifecycle
  selected-service closure, target platform, host-port, memory, dependency selection, and
  normalized runtime `--ready-timeout` are semantic grant scope. One proof-owned transaction now
  spans the complete runtime or lifecycle invocation set and cleanup. A bounded runner-private
  Unix descriptor carries authority only between immediate Ota processes and is removed before
  selected code executes. Runtime archive v6 and lifecycle archive v3 embed and re-derive the
  exact terminal authority rather than inheriting a workflow-only grant. Runtime archive
  and lifecycle archive reconciliation preserve requested backend and lifecycle overrides
  separately from effective values, preventing an implicit default from changing authority
  identity during verification.
  The first `prebound_file` carrier reads only fixed system trust state protected from Ota's current
  unprivileged process; it does not claim hardened provider-attested privilege separation. It verifies an
  Ed25519/RFC-8785 signed bundle, exact contract/scope/family/classification/actor binding,
  bounded freshness, signed revocations, and protected sequence/clock high-water evidence; and
  refuses before sandbox admission or execution side effects. `ota run` repeats the
  time-, sequence-, and revocation-sensitive admission immediately before transaction creation.
  `ota authority inspect --json` now exposes a separate diagnostic-only hardening profile over the
  same fixed-path protected-file verifier. It checks every fixed-store binding, emits typed
  required/informational observations, and remains bounded to
  `current_process_filesystem_guarded`; it selects no grant, writes no authority state or receipt,
  and cannot make a crossing admissible. Passwordless sudo, namespace control, alternative
  container endpoints, provider metadata credentials, and broader escalation stay explicit
  unknowns when Ota cannot observe them safely.
  Real execution durably creates a
  runner-owned per-scope crossing transaction before any selected-lane side effect, terminalizes it
  on success, precondition/startup failure, interruption, or abandoned recovery, and binds that
  transaction identity to the fresh crossing receipt. Archived receipts preserve the signed
  bundle, binding, scope,
  admission time, and terminal transaction for re-derivation against the current fixed trust root.
  Receipt history derives crossing necessity from each authority-bearing archive's canonical
  selected-invocation scope and archived contract snapshot, never from global authority
  configuration or an editable lane label;
  older snapshot-less receipts remain visible only as `legacy_unverified`.
  The first local transaction carrier is explicitly `runner_local_content_addressed`: it is
  runner-authored, locked, and internally reconciled, but not independently authenticated against
  same-user state tampering. Refusal and dry-run emit no crossing record; successful dry-run
  publishes only `admissible_not_consumed`, while task dry-run and workflow-refusal evidence
  additionally carry derived scope/contract identities, boundary family, and classification alongside typed
  `prebound_file` authority-source, authority/grant selection, reason, and
  `execution_started: false` evidence. That issuance surface never exposes task inputs or trust
  material and avoids workflow-side reconstruction of Ota semantics. Existing contracts
  remain compatible until they opt into crossing authority, and grants never bypass V11.3 agent
  refusal. Core regression proof covers exact admission, mutation, revocation, sequence rollback,
  selected-graph expansion, missing-grant refusal, dry-run parity, pending-journal recovery, and
  terminal outcome reconciliation. Create-chrome-extension hosted refusal pressure
  [30714738522](https://github.com/bobaikato/create-chrome-extension/actions/runs/30714738522)
  confirms that a GitHub-hosted runner without the fixed authority source refuses before scaffold
  execution; its normal matrix [30718303916](https://github.com/bobaikato/create-chrome-extension/actions/runs/30718303916)
  is green across native and bounded container paths. This does not prove live grant authority
  separation because a hosted job could self-provision its own filesystem state. The first
  carrier's live, expired, revoked, and out-of-scope VPS pressure is now green:
  [30863257307](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863257307),
  [30862934335](https://github.com/bobaikato/create-chrome-extension/actions/runs/30862934335),
  [30863024099](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863024099), and
  [30863121110](https://github.com/bobaikato/create-chrome-extension/actions/runs/30863121110).
  The live artifact retains the exact transaction-bound receipt archive; every refusal preserves
  typed admission evidence and an unchanged checkout before and after dry-run and real refusal.
  The immutable native Linux/systemd proof [31986767770](https://github.com/ota-run/ota/actions/runs/31986767770)
  now passes on the OrbStack-backed `self-hosted/linux/orbstack/systemd` runner at this Core
  revision. Its retained artifact proves native validation, `ota doctor`, `ota up`, and runtime
  proof execution; the proof remains explicitly bounded to its selected runtime path and records
  dependency exercise and broader-repository behavior as unproved.
  The carrier-neutral transaction/archive foundation uses transaction schema v2 to bind authority
  carrier, admission identity, authorization identity, and terminal state. V1 archives remain
  legacy `prebound_file` evidence without a carrier envelope; receipt history rejects
  carrier-envelope injection into v1 and missing or substituted envelopes in v2. The Unix
  launcher-session `authority_broker` carrier is now executable for governed `ota run` and
  `ota up`. Its v1 wire structs, fixed message domains, bounded framing, and canonical nonce,
  message, and work-unit identities now come from the immutable public
  `ota-run/authority-protocol` revision `242685d5b7c3904681f1c71d734fbe2d41679dda`; Core retains
  trust-root, verification, admission, transaction, receipt, and archive ownership. It selects
  exactly one protected binding from `/etc/ota/crossing-brokers.json`, freezes
  the semantic work unit, verifies challenge-bound launcher attestation, obtains signed
  authorization, creates the durable pending transaction, and atomically consumes the exact lease
  after deterministic admission succeeds and before provisioning or selected work. Ota persists
  the exact consume intent before transport. If acknowledgement is uncertain, a later
  invocation obtains fresh launcher attestation and re-queries the exact intent; consumed,
  not-consumed, and unknown results all close the abandoned transaction as incomplete and never
  resume work. A durably recorded signed recovery status is re-verified locally after restart
  without a second query, and consumed recovery retains its intent until the atomic terminal write.
  Pre-recovery seven-domain broker archives retain their original identity through archive-only
  compatibility; live bindings require all nine current domains.
  The reserved v3 Linux `systemd_protected_launcher/v1` branch now requires Core to send a
  private process-posture preface and to match that exact posture against the signed complete
  systemd launcher/job-principal instance before admitting broker traffic. Core receipt and archive
  re-verification are implemented and locally tested. Core now pins the immutable Protocol
  child-identity foundation at `6a2d0dc504a313a513ee41105f51449195c85797`; the reviewed
  execution-disabled Launcher implementation is `73a39c95ffab3125819ee655bdc7a740ec3204b9`.
  The execution-disabled authority-launcher
  foundation now consumes only the fixed systemd listener, reconciles its unique
  `/proc/net/unix` inode/path with protected socket metadata, derives the job peer through
  `SO_PEERCRED`, validates the governed Ota command before helper work, retains the exact verified
  Ota executable descriptor, and maps one protected job principal to one distinct execution
  principal. Its short-lived helper clears inherited descriptors and supplementary groups, adopts
  the complete target UID/GID posture with `no_new_privs`, and requires Linux `openat2` containment
  before returning only the repository directory descriptor through `SCM_RIGHTS`. The service
  now creates and fsyncs a protected active-slot intent, forks the exact fixed Ota binary as a
  root-stopped child, binds its invocation/request, PID/start, binary, principal, directory, and
  exact descriptor-object posture. Startup promotes valid child- or scope-bearing temporary state
  and uses PID-bound pre-scope cleanup; an intent-only, mismatched, unsupported, or uncertain
  recovery remains a hard refusal. The current immutable scope slice at Protocol
  `adaabfb8300925a09975c7244e27242b5cd41e60` and Launcher
  `0f9d9eb33e37d6cd855aafdbc7c4d72b3c8957e2` requests one request-derived transient scope from the
  root systemd manager, independently reconciles its fixed slice, non-delegated controls, kernel
  cgroup, and sole stopped PID, and records that identity before terminal cleanup. Scope-bearing
  recovery stops the exact scope when still present and confirms the scope absent plus its recorded
  cgroup empty or absent before releasing the principal slot.
  OrbStack's systemd refuses the real pre-exec PID attachment with `ENOTTY`, so that environment
  proves fail-closed behavior but not positive scope ownership. Immutable Linux/x64 VPS run
  [31373366733](https://github.com/bobaikato/create-chrome-extension/actions/runs/31373366733)
  now proves the exact reproducibly built launcher/client identities, fixed socket, root-stopped
  child, positive `openat2` containment, request-derived transient scope, terminal scope removal,
  child reap, active-slot cleanup, and unchanged repository state. Crash/recovery run
  [31373928434](https://github.com/bobaikato/create-chrome-extension/actions/runs/31373928434)
  proves a root-only post-scope crash at exit `86`, durable abandoned-slot reconciliation before the
  next request, terminal refusal, and zero residual slots, scopes, or recorded children. The empty
  execution-disabled child was collected before the post-crash scope observation, so the evidence
  does not claim a still-loaded scope at that instant. These runs exposed and fixed listener-table,
  systemd Scope-interface, collected-unit cleanup, and build-path reproducibility defects. Those
  immutable revisions never resume the child and do not contact the broker, consume authority,
  execute selected work, or emit receipts/archives. OrbStack Linux/x64 root tests
  against the immutable stopped-child revisions prove socket
  replacement refusal, descriptor transport, and fail-closed behavior when that environment
  reports `openat2` as `ENOSYS`. The separate VPS kernel-pressure run
  [31319741342](https://github.com/bobaikato/create-chrome-extension/actions/runs/31319741342)
  checks out exact authority-launcher `99affd90f712512fa1fd7c039868d114904736cf` and proves the
  positive `openat2` containment flags plus symlink-escape refusal on Linux/x86_64 kernel
  `6.8.0-134`. This does not exercise the root UID-switching helper or the systemd service.
  The execution-disabled immutable transient-scope foundation is now proved. Core
  `cc680cef790bf8334ee0dfe513c202a51c21954e`, Protocol
  `b4f36fe450dc4047bd7bd623ea8ba60fd951e31d`, and Launcher
  `d8aa1d0bf9783d29d53d0a5e912f09f1fa414624` resume that exact scoped child only far enough to
  receive Core's bounded private
  process-posture preface. It re-derives the posture identity and binds PID/start time, Ota binary,
  and principal mapping before exact scope/child/slot cleanup. Core emits the preface before CLI
  parsing or command dispatch and blocks for launcher continuation; this slice deliberately sends
  none. Malformed or substituted posture fails closed. Hosted normal run
  [31389237232](https://github.com/bobaikato/create-chrome-extension/actions/runs/31389237232)
  and root-armed crash/recovery run
  [31389713244](https://github.com/bobaikato/create-chrome-extension/actions/runs/31389713244)
  bind exact reproducible binary identities, unchanged repository state, zero terminal scopes, and
  the typed `posture_admitted_boundary_removed` terminal stage; the crash path records launcher
  exit `86` before fresh reconciliation. The first hosted posture attempt also exposed that Core's
  schema-validation fallback embedded its absolute compile checkout. Published JSON schemas are now
  embedded into the source-built binary, making installed schema validation and immutable Core
  binary reconciliation independent of both a source checkout and the checkout directory that
  compiled Ota. This closes only the immutable hosted execution-disabled posture gate. V3
  attestation, broker authorization, one-use lease consumption, selected execution, receipt/archive
  evidence, the production systemd execution path, and provider attestation remain unproved.
  The independently reviewed execution-disabled V3 bridge pins Protocol
  `953e9e6407c9de030822b1f891046c2829b3c714` and Launcher
  `0ed578a46ce821d8dd1da671a2e53c75ded1ed0b`. An identity-bound launcher continuation binds the
  exact invocation, child, working directory, posture, and principal mapping while unlocking CLI
  parsing after exact posture admission. Core consumes and removes the launcher-only startup
  environment before CLI dispatch, then freezes the real semantic scope and verifies one signed V3
  response through its canonical broker verifier, including exact reconciliation back to the
  retained startup binding. The launcher observes but does not forward the resulting exact
  authorization request. Exact scope, cgroup, child, and active-slot cleanup precede the typed
  `attestation_admitted_before_authorization_boundary_removed` refusal. This bridge has local
  protocol and Linux regression evidence only and still requires immutable Linux/x64 pressure. It
  does not prove authorization, lease consumption, selected execution, receipts/archives, the
  complete production path, or provider attestation. No example, Skill, or
  Site propagation is required for this internal execution-disabled carrier step because it adds no
  contract, CLI, operator, receipt, or archive surface; those surfaces must move with the first
  usable production adapter.
  The committed systemd V3 candidate at Authority Protocol
  `574563d1f69a674960d0b3228c5a13b13bc42c19`, Authority Launcher
  `13bf6db71610b86c81a251f440b80b9b8947a67d`, and Core
  `31fa95b4d28a8a4971ee3fd65c841d40e54ac4d9` completes the protected collector and producer
  bridge. Authority Protocol defines the canonical domain-separated claims/request/response,
  protected producer binding, `ota.authority-launcher.systemd/v3` profile identity
  `sha256:b5853a12e72c4ca32b0f93a38bc8f1097c7809039b58449f67fcf9019d0ea480`, and paired
  `ota.authority-job-principal.systemd/v2` identity
  `sha256:ee6ea951aff4a80f8a4f93c576a93e3b29245b87d162726c2401c124a7a78659`. The Launcher verifies
  protected installation identities, exact systemd unit/socket/scope properties, process
  containment, account/sudo/Polkit posture, protected-path and host-socket denial, and Ota
  process-access denial before invoking the separately credentialed `ota-authority-attestor` over
  fixed `SOCK_SEQPACKET`. The producer owns signing key, clock, and durable idempotent issuance;
  the launcher owns only public verification and exact request/response reconciliation. Core now
  independently re-derives the complete ordered profile, nested identities, signed claims, and
  retained startup binding before emitting an authorization request.
  Local ARM64 OrbStack PID 1 systemd pressure reached `authorization_received`, then the launcher
  deliberately withheld the request and removed the exact scope, cgroup, child, and active slot.
  Selected-work sentinel, receipt store, and broker decision/lease state remained empty.
  Protected-installation drift, systemd runtime drift, and missing producer credentials refused
  before authorization with zero terminal slots/scopes. A pressure-only exit after durable scope
  recording retained one recovery slot; the next activation reconciled it to zero before accepting
  another request. That was local candidate evidence. Immutable Linux/x64 PID 1 systemd run
  [31530832876](https://github.com/ota-run/authority-launcher/actions/runs/31530832876) now binds exact
  Protocol `574563d1f69a674960d0b3228c5a13b13bc42c19`, Launcher
  `c69ad3afc6afef0e260a7eeaa4f7340971db50af`, and clean source-built Core
  `31fa95b4d28a8a4971ee3fd65c841d40e54ac4d9`. Its retained cursor-isolated artifact proves the
  complete signed positive/recovery stage sequence and typed terminal refusal; installation drift,
  runtime-property drift, unavailable producer credentials, and the injected pre-session crash do
  not reach authorization. It records one durable `scope_attached` crash slot, zero terminal
  slots/scopes, byte-identical repository manifests, no selected-work or `.ota` state, and only the
  public verifier identity. This closes the hosted execution-disabled V3 admission gate only. The
  GitHub workflow controller still provisions the root services, so independently administered
  provider/launcher separation is not proved. At those immutable revisions, no authorization
  decision, one-use lease, selected execution, crossing receipt/archive, or provider-attested
  separation existed.
  The signed authorization-decision slice advances only through decision admission.
  Protocol adds a Core-authored, identity-bound decision acknowledgement and launcher relay
  envelope. The Launcher binds a protected pressure broker executable and service/socket identity,
  rechecks the live pidfd-bound executable around relay traffic, forwards Core's exact request only
  after complete V3 admission, relays only signed decisions, requires Core's exact acknowledgement,
  and durably journals that relay before exact boundary cleanup. Core acknowledges only a decision
  that passes canonical signature, freshness, request,
  attestation, contract, work-unit, and semantic-scope verification. Allowed decisions end at
  `authorization_decision_verified_before_lease_boundary_removed`; denied or invalid decisions
  remain bounded refusals. Immutable Linux/x64 PID 1 systemd run
  [31561247605](https://github.com/ota-run/authority-launcher/actions/runs/31561247605) covers allowed,
  denied, stale, wrong-scope, pending-timeout, ambiguous, and unavailable-proxy cases with zero
  worktree, receipt, active-slot,
  or scope residue. Negative cases require exact pressure-peer response checkpoints and Core
  acknowledgement counts rather than the generic protocol-refusal terminal alone. The artifact
  retains public signed decisions, the public broker verifier binding, and bounded relay envelopes
  for independent identity and signature re-verification after cleanup, never private signing
  material. Core also requires a final response after pending authority to advance the broker
  revision, preventing an older still-valid final response from replacing newer pending state. The
  matrix injects crashes after durable scope and allowed-decision recording and requires
  cleanup-only recovery before a fresh request, with complete repository-manifest equality for each
  decision scenario. It binds exact Protocol `6a92d8db9d089e44d1980f1871bf6e90eccb9960`, Launcher
  `77ab20aa6ed5e3dd42cc6815ba2de7cd36d543bf`, and clean source-built Core
  `b71b78ca33ea2edd7bb03ceb66c5e1e104217cd9`. Independent artifact inspection re-verified all eight
  signed decisions, all five relayed admission/decision identity pairs, zero terminal slots/scopes,
  two cleanup-only crash recoveries, fourteen byte-identical repository-manifest pairs, and no
  selected-work, `.ota`, lease, receipt, archive, private-key, or credential residue. No
  lease issuance/consumption, selected execution,
  receipt/archive, independently administered separation, or provider attestation is claimed.
  The execution-disabled one-use lease boundary is now immutable-hosted pressure evidence. Protocol
  `899718c93f205eea8ae403e041be9449daa89192`, Launcher
  `2185682777c3603ae428dda68d47b1e39d709753`, and clean source-built Core
  `874c5954798453f92a0141bfc964fe1a90db8d92` passed Linux/x64 PID 1 systemd run
  [31631358796](https://github.com/ota-run/authority-launcher/actions/runs/31631358796). Core freezes a
  launcher-owned pending transaction without repository state, binds its authentication posture to
  the private active-slot persistence owner, and emits one exact consume request only after signed
  V3 attestation, authorization, and prepared-lease verification. Launcher fsyncs the consume intent
  before broker relay and the signed consumed response before terminal cleanup. The pressure-only
  broker atomically persists spent lease identities in root-owned `0700`/`0600` state before its
  first response; replaying the identical lease and consume request produces one signed
  `already_consumed` response while Core records exactly one accepted consumption. The matrix also
  covers denial, stale and wrong-scope responses, pending timeout, ambiguity, unavailable broker,
  protected installation/runtime/credential drift, and both intent/acknowledgement and
  post-consumption crash recovery. Its retained artifact has byte-identical repository manifests,
  one deliberate pending recovery slot only at each injected crash boundary, and zero terminal
  slots/scopes or selected-work, `.ota`, receipt, and archive residue. This proves one-use lease
  consumption only for the execution-disabled systemd carrier and pressure broker. At that
  revision, selected execution, crossing receipts and archive re-verification for launcher-owned
  evidence, and independently administered provider/launcher separation remained open; the later
  immutable gates recorded below close those V11.7 requirements. Site, Skills, and Examples remain
  unaffected because this slice adds no public command,
  contract-authoring, receipt, archive, or usable operator surface.
  The exact replay reopens root-owned durable state but does not restart the pressure broker
  process; restart persistence is not separately pressure-proven.
  Immutable Linux/x64 PID 1 pressure run
  [31664495937](https://github.com/ota-run/authority-launcher/actions/runs/31664495937)
  binds Protocol `9fb00a4ab0f1b4c635dbab67c2e6b140b8eade9c`, Core
  `06976f3eb4919a0bddaa318ed0824a6b9448aaaf`, and Launcher
  `e8b6ae5108559508cfb75141cb9b317d46c182f3`. Core retains the launcher session after atomic
  consumption, executes only the frozen work unit, finalizes the crossing transaction and receipt,
  and requires exact launcher persistence acknowledgement before exiting. The launcher then
  reconciles the child exit and
  emits terminal finalization only after the exact child, scope, cgroup, and active slot are absent.
  The run proves completed, failed, interrupted, replay-refused, pre-execution refusal, and five
  crash-recovery boundaries with exact child, scope, cgroup, and active-slot removal. Receipt
  history reports one valid archive and zero invalid archives for the successful lane. Portable Ota
  archives do not yet embed the launcher-authored post-process finalization; the
  outer pressure artifact is the only current carrier for that cleanup record. The canonical Skill
  and Site broker reference carry that distinction. Examples and the public command index are
  unaffected because this candidate adds no contract shape, command, or flag.
  The committed additive `ota.authority-launcher.systemd/v2` foundation at Protocol
  `cb5f539a4c3d9d75e2dd36692da8e69be5ba6e14`, Launcher
  `fddb10393aa0e79258ff048e32774a685d5fac04`, and Core
  `e3febf3d8d4226dc26ef20ddebaf1e1b23ef5fd3` publishes profile identity
  `sha256:c816a49e01120bf1f793aedcfec094ca0f23a8ee80f1c7e5bed4c2d9c797cb42`. It preserves V1 archive
  verification while replacing launcher-owned credential settings with producer socket metadata
  and the public verifier set. Core accepts only the exact registered V1 or V2 profile-ID/identity
  pair, and the Launcher collector assembles observations in canonical order while refusing any
  unavailable source. The committed live Linux job-principal preflight at Launcher
  `60a07055477ed27d6c82a2885fa9a87da94c6a70` and Core
  `591289f441cf9f0832d9605001854e3aa89f5df5` runs before repository opening or child creation:
  socket-bound pidfd, protected UID/GID mapping, exact `/proc` UID/GID slots, empty supplementary
  groups and inheritable/permitted/effective/ambient capabilities, and `NoNewPrivs=1`. Launcher
  `d437aed99daf4ae55e5d8299a99ce5df535fb07f` additionally retains and revalidates the protected
  broker-proxy pidfd before and after bridge traffic, with an orchestration regression proving peer
  exit during that window refuses. Those committed slices remain historical foundations for the
  complete committed collector and producer path described above.
  `ota up` evaluates
  unrelated blockers and the complete ordered prerequisite-instance preflight
  before broker contact; those prerequisite instances execute once inside the parent work unit.
  Authority launcher run
  [31257509444](https://github.com/ota-run/authority-launcher/actions/runs/31257509444) against exact
  Core `9244eb2bc6a44151c4172c0634ac44bdb216a65a` and immutable protocol
  `242685d5b7c3904681f1c71d734fbe2d41679dda` proves lost consume acknowledgement, fresh-session
  consumed-status recovery, incomplete old-transaction finalization, fresh authorization,
  exactly one selected-task execution, one valid recovery archive, and zero invalid archives. Run
  [31257511093](https://github.com/ota-run/authority-launcher/actions/runs/31257511093) reproduced
  the same complete workflow at the same launcher revision.
  Ordinary workflow readiness timeout, selected workflow instance, ordered prerequisite-instance closure,
  and runner-derived scope breadth are identity-bound; breadth retains only counts, categories,
  and hashed resource identities. The archive retains a public verification binding, not the live
  launcher descriptor. Signed protocol payloads retained for
  archive re-verification accept only bounded public-safe labels, never raw paths, descriptors,
  credentials, or secret provider material. Dry-run performs no launcher interaction and reports
  only `requires_live_authorization`; task processes do not inherit the protected descriptor. Receipts
  and archives bind the broker admission, attestation, prepared lease, consume exchange, semantic
  scope, and terminal transaction, and history re-verifies them against the protected binding.
  Replay, missing consumption, and carrier substitution refuse. Grant-required runtime and
  lifecycle proof now retain one transaction across their complete invocation and cleanup sets;
  terminal runtime-proof transactions bind the final proof verdict rather than the intermediate
  readiness state. Proof archives now retain repo-relative contract-snapshot references while
  preserving same-root compatibility for earlier absolute references, and archive emission now
  requires immediate reconciliation through Doctor's semantic loader. Authority launcher run
  [31033509379](https://github.com/ota-run/authority-launcher/actions/runs/31033509379) is green
  against exact Core `bd80b29d971ccd5ac8609d9fc767a491ff382ef8`. It proves one live broker run,
  expired/revoked/wrong-scope/replayed refusal, same-scope missing-launcher proof refusal, runtime
  archive consumption by Doctor, and completed runtime and lifecycle proof transactions. The
  lifecycle fixture uses a root-owned deterministic pressure control because Docker remains
  inaccessible to the job principal; it proves Ota lifecycle authority/finalization, not Docker
  provider behavior.
  The public operator guide now documents the fixed trust-store, separately protected bundle and
  sequence-state layout, and the provisioner/runner boundary without publishing usable authority
  material. Hardened-runner pressure now proves the carrier's bounded
  `current_process_filesystem_guarded` posture; the guide remains a preview because it does not
  claim provider-attested separation, reusable broker credentials, or one-use work-unit authority.
  Authority launcher run
  [31250919192](https://github.com/ota-run/authority-launcher/actions/runs/31250919192) against exact
  Core `257be61dd91799237357390b145be950f2fc6b3f` additionally proves broker-unavailable,
  bounded approval-timeout, local-cancellation, and conflicting-pending-response refusal before
  selected work. Each refusal retains byte-identical checkout manifests and no receipt state.
  Authority launcher dispatch
  [31260927337](https://github.com/ota-run/authority-launcher/actions/runs/31260927337) against exact
  Core `9244eb2bc6a44151c4172c0634ac44bdb216a65a`, with final merge-gate confirmation in
  [31261639968](https://github.com/ota-run/authority-launcher/actions/runs/31261639968), proves
  terminal cancellation before an undeliverable late approval, insufficient pre-wait attestation
  freshness refusing before authorization, and two executions of one broad three-task semantic
  scope consuming distinct work units with two valid archives. At that revision, V11.7 still
  required the hardened-launcher separation later proved by runs `31939777636` and `31953535665`.
  Provider attestation is optional stronger follow-on hardening. V11.22 is complete for its
  source-bound candidate and fail-closed closure-classification foundation. The internal
  candidate now binds the registered detector source inventory: fixed root markers, supported
  environment files, package-manager locks, direct workflow files, bounded .NET project paths,
  and detector-owned root extension markers. Inventory sources cannot escape the repository
  through symlinks. Its shared closure resolver records direct finite manifest commands with their
  executable requirement. It also resolves a finite declared package script only after binding the
  exact manager invocation, script name, script body, and executable graph. It follows only
  same-manager, same-manifest script references and rejects cycles, missing hops, composed shell
  bodies, indirect package scripts, and CI-only commands as unresolved. It retains unknown effects
  for every resolved graph. Resolved closures bind only runtime, tool, or toolchain requirements
  detected from the same manifest and mark platform `unknown` until a task-scoped platform source
  exists. `ota detect --candidate-out <root-relative path>` now publishes that same
  self-verifying review artifact from one command-owned immutable source snapshot, with
  descriptor-safe create-new collision refusal and no `ota.yaml` mutation.
  `ota contract apply-candidate` now supplies candidate admission and explicit `--write`: it
  self-verifies the reviewed artifact, requires the exact detector implementation, re-derives
  current source and existing-contract truth, and reconciles an identity-bound application
  projection over the reviewed base contract, exact normalized operations, and fully validated
  resulting contract identity. Candidates without a complete valid projection refuse admission;
  unrelated `unknown` or `unsupported` findings remain review state unless `--require-complete`
  is requested. `--write` locks the retained no-follow repository descriptor, rechecks current source and evidence,
  and atomically creates only a previously absent `ota.yaml` from the shared evaluator's returned
  validated contract; default `--write` never overwrites an existing contract, and semantic
  reapplication is a no-op. The explicit `--write --carrier git` path now admits a non-detached
  tracked `ota.yaml` that matches `HEAD` in both index and worktree, commits only the reviewed contract with expected-HEAD branch
  compare-and-swap, verifies the resulting worktree/index, and reports branch plus prior/resulting
  commit identities. It never pushes, rebases, amends, or changes unrelated paths. The registered
  `legacy_flat_toolchain_fulfillment_v1` upgrade emits a schema-v2 source-bound candidate and can
  use the explicit Git carrier after exact re-derivation. Repo-level legacy mutation flags now refuse
  before repository access; `detect --write` remains the temporary conservative create-new alias.
  `init --dry-run --json` now emits the read-only schema-v4 `init_starter_preview_v1` candidate,
  binding the exact starter preview to its immutable source capture and resulting contract identity
  without an application projection or write authority. The pre-commit Buzz observation is local
  only and has no retained artifact or hosted run, so it is not completion evidence. V11.22
  intentionally emits no inferred agent-safe proposal: maintainer-authored contract safety remains
  authoritative, and positive effect-backed promotion is deferred to V12's typed effect and
  realization evaluator rather than treated as a V11.22 closure gate;
  V11.22 does not consume crossing records as approval authority. V12 effect-bound
  refusal assurance had completed its bounded implementation while the formal real-repository
  pressure bar was still open. The final Plausible and Outline witness matrices now complete that
  bounded pressure bar; independent closure reconciliation does not widen the completed
  crossing implementation. See [V11.7](../planning/v11.7/plan.md),
  [V11.22](../planning/v11.22/plan.md), [V12](../planning/v12/plan.md), and the planned,
  inactive [V12.1 secret-delivery governance follow-on](../planning/v12.1/plan.md).
- completed V11.17 trusted replay-baseline regeneration: Core now has an additive
  `artifacts.<name>.replay` authority chain: explicit producer record, immutable
  recorded attestation, exact promotion, then replay consumption. A portable authority manifest
  binds the canonical recursive output set, producer receipt, source/contract identities, and the
  producer's V11.16 execution-boundary graph plus asserted-target and derivation-input closures.
  `read_only` refuses outside a runner-owned ephemeral container boundary and mounts a run-scoped
  snapshot outside the writable workspace for the full selected closure; `verify_unchanged`
  detects mutation after native or container replay and never claims prevention. Portable authority
  declares SCM review as its external selection trust root, not signer-backed provenance or
  Ota-verified reviewer approval. Bedrock now has an additive unsafe `record:baseline` producer and separate
  promoted offline consumers. Its promoted lane correctly fails closed with
  `OTA_REPLAY_BASELINE_UNAVAILABLE` until an intentional live recording is reviewed and explicitly
  promoted. Upstream [run 30268181240](https://github.com/vinimabreu/bedrock/actions/runs/30268181240)
  recorded an approved live-model candidate with attestation
  `sha256:0bfb61977a38310c7ee515a4de31cec5ca4198b7bb63fb1a1bd980f920a39b93`
  and source `git:bb0ace385cfe17c1a5c195f3cd20de60a446cea6`. Its aggregate score improved, but
  `top_product_by_quantity` became `stable_wrong`; it is intentionally unpromoted. The uploaded
  review artifact omitted the required producer receipt archive, so it cannot later support
  `ota baseline promote`; Bedrock must retain `.ota/receipts/` and make a fresh reviewed recording
  after that workflow fix. This is a Bedrock workflow-retention gap, not an Ota Core defect. Core focused
  record/promotion, JSON conformance, and published-schema tests pass. EventCatalog closes the
  independent non-model generated-baseline gate: [run 30198942717](https://github.com/bobaikato/eventcatalog/actions/runs/30198942717)
  preserves the ordinary native generator matrix on Ubuntu, macOS, and Windows and separately
  proves a fresh runner can consume the committed portable authority without local `.ota` history:
  Doctor admission, workflow dry-run, setup hydration, detached compiler consumption, and receipt
  archival all pass. CI never creates or commits that authority; its `scm_review` trust root remains
  an external delivery/review assumption, not Ota-verified reviewer approval. Source identity
  correctly excludes declared Langium outputs and refuses workflow-generated transient JSON until
  that evidence moves to `$RUNNER_TEMP`; the promoted consumer compiles the approved authority
  rather than comparing a newly approved baseline against Git `HEAD`. EventCatalog run
  [30226480354](https://github.com/bobaikato/eventcatalog/actions/runs/30226480354) completes the
  independent strict-replay pressure gate: ordinary generated-source lineage remains green on
  Ubuntu, macOS, and Windows, while explicit record/promotion and committed-authority consumption
  both run through the declared ephemeral container boundary with `read_only` enforcement. A
  credentialed Bedrock recording is now an upstream, intentionally reviewed adoption lane. Its first
  candidate remains unpromoted for a real `stable_wrong` regression and missing receipt retention;
  it is not a release gate for the general V11.17 model. Dagger exposed
  and now exercises the needed hybrid posture:
  `generated_source` retains an ordinary producer-dependent SDK check while a second task can
  consume the same output only through explicit promoted replay authority. The Core schema, Doctor,
  receipt, mutation guard, and strict runner boundary all derive that consumer distinction from the
  producer dependency. Its hosted pressure run [30179771523](https://github.com/bobaikato/dagger/actions/runs/30179771523)
  bootstrapped Core `b5b55e0e` and passed contract/discovery/dry-run admission, but the ordinary
  producer failed before record/promotion because Dagger's module-owned runtime resolves
  `protobuf-dev~32` against a mutable Alpine package index that no longer satisfies it. This is not
  an Ota replay defect or a reason to weaken the generator; it is an explicitly bounded external
  Dagger engine/module-runtime provenance gap. Dagger remains unsuitable for green generated-
  baseline pressure until its upstream runtime is made reproducible or Ota gains a Dagger adapter
  that can attest that tool-managed container state.
- completed implementation slice: V11.19 typed uv local-project hydration. The replay classifier
  now requires resolved source posture, declared lockfile identity, and clean local-project source
  identity before editable hydration can be acquitting; missing lockfile or source identity remains
  narrowing evidence. Dograh [run 30165303012](https://github.com/bobaikato/dograh/actions/runs/30165303012)
  proves nested editable Pipecat hydration with its full declared extras, ordered `dev` group,
  manifest, lockfile, and Git source identities, plus bounded PostgreSQL/Redis lifecycle execution.
  Marimo [run 30165304206](https://github.com/bobaikato/marimo/actions/runs/30165304206) proves the
  different editable root-project plus `test`-group shape on Linux and macOS; it has no `uv.lock`,
  so Ota records manifest and Git source identities while correctly retaining narrowing replay
  evidence. Both matrices bootstrap exact Core `19509754` and retain platform-specific evidence
  artifacts. Dograh's Dev Container, unpinned Node validator install, and GitHub-service versus
  local-Compose divergence remain bounded. Marimo's frontend/pnpm, Playwright, Docker, release,
  and Windows lanes remain repo-owned outside its selected proof.
- completed finite-command interaction pressure: omitted
  `tasks.<name>.command.interaction` resolves to `auto`, allowing terminal passthrough only for
  human native terminal execution. Explicit `forbidden` keeps every prepared closure step
  noninteractive, while `required` refuses before dependencies or workflow prepare/setup phases
  when the selected boundary cannot provide a terminal. Effective posture survives mode selection
  and orchestrator wrapping; dry-run JSON publishes the invocation-specific
  `terminal_passthrough`, `piped`, or `refused` resolution. Agent, captured, container, remote,
  and ordinary non-TTY CI execution do not acquire terminal capability. Workers SDK
  [run 30265519625](https://github.com/bobaikato/workers-sdk/actions/runs/30265519625) bootstraps
  Core `c5256d64f3060b30d40f07fa389b2bb16fc61b1f` and proves validation, Doctor, task discovery,
  dry-run JSON, non-TTY refusal before hydration, and agent refusal across Ubuntu, macOS, and
  Windows. Windows also proves a failed optional WSL shell probe cannot corrupt machine JSON.
  The real OAuth/account success remains intentionally external and not proved; the contract does
  not advertise container or remote execution for this terminal-auth lane.
- completed replay-input identity hardening: optional task `replay_inputs[].expected_identity`
  pins validate canonical SHA-256 values, surface missing or mismatched artifacts through Doctor,
  block dry-run/run/up before task startup, and preserve expected plus observed identity in the
  receipt evaluated-input carrier. Bedrock pressure proves matching frozen inputs through Doctor,
  dry-run, and real native plus container agent-safe execution.
- completed V11.20 policy-governed replay-input identity implementation: the shared
  `replay_input_policy` evaluator applies cumulative task/workflow rules over their exact selected
  closures, observes each task-qualified declared input once, and derives `deny > review > allow`.
  Doctor findings and JSON, dry-run, run, up, proof runtime, proof lifecycle, and
  admission-produced execution/refusal receipts reuse one command-scoped loaded policy snapshot
  and observed identity set across agent safety, claim assurance, replay policy,
  provisioning/effect findings, proof, CI projection, and receipt policy evidence. Runtime proof
  passes that same admitted authority to its detached child through a private temporary snapshot.
  Active policy load failures remain typed fail-closed admission evidence, and selected closures
  include recursive outcome hooks so a governed or mismatched hook cannot execute behind an
  admitted parent.
  Aggregate monorepo Doctor JSON retains the policy result for each selected member. Admission
  refuses before native provisioning, proof artifact creation, dependency hydration, service
  ownership, assertion execution, or task startup; unavailable observations and unreadable or
  mismatched declared pins fail closed, and hard-pin refusals retain the active policy evidence.
  Generic readiness receipts do not reconstruct policy after execution. Runtime proof evaluates
  its full selected proof closure, including seam observers and its selected negative control,
  before passing one preflight through every readiness diagnosis and the embedded Doctor artifact;
  lifecycle proof evaluates its exact prerequisite-plus-assertion closure before beginning a
  transaction. Missing pin coverage follows the rule's `on_insufficient`. Unknown selectors are
  contextual policy findings, not contract-validation errors. CI projection carries the exact
  active policy identity, applicable rule identities, canonical execution closure including
  recursive outcome hooks, and unresolved selector identities but
  no render-host observation; the provider checkout recomputes observed replay-input identities.
  Typed-effect CI projection also carries its complete non-secret policy decision in the projection
  identity and re-evaluates it from checkout bytes before provider setup or selected execution. Core
  unit, real-repo no-execution, JSON-schema, JSON-conformance, and projection
  checkout-re-evaluation regressions pass; copy-ready Core/external examples, canonical skill, and
  site reference are aligned. Connected public guidance is pinned at Site
  `7fa71e4dd4a4f1348e9b45af9060acc954ed7034` and Skills
  `610e801c9b32030d888b0c6d0118a5e70af0165a`; Examples require no change because this slice adds
  no contract-authoring shape. The focused CI boundary is immutable-hosted on Linux/x64 and macOS
  in [run 33173733814](https://github.com/ota-run/ota/actions/runs/33173733814) against exact Core
  `39d2f3964aec84a6e5ff5b0fdb19fa94ce27c8eb`. Both retained artifacts carry schema-valid
  compatibility-warn and explicit-deny projections, bind distinct projection and policy-decision
  identities after checkout policy drift, and retain the deny as `effect_policy_denied` before
  setup, execution, or durable logs. This remains an internal fixture: provider execution,
  mutation, positive receipts, archives, assurance, and independent real-repository behavior are
  not proved.
  Bedrock [run 30413944121](https://github.com/bobaikato/bedrock/actions/runs/30413944121)
  proves strict matching admission for four declared frozen inputs through native and container
  execution. Kylrix [run 30413944203](https://github.com/bobaikato/kylrix/actions/runs/30413944203)
  preserves ordinary unpinned compatibility while its dedicated strict-policy lane refuses
  Doctor, dry-run, real `ota up`, receipt, and `doctor --fix` before setup outputs are created.
  Both bootstrap exact implementation Core `f97b96cc`; later Core `d0e77a95`, `ff35a910`,
  `2d0b20fc`, `c85af3d2`, `6aaa063e`, and `4729e042` reconcile release-gate fixtures, generated
  reference truth, and hermetic test tooling without changing runtime behavior. Core
  [Release Gate run 30452821989](https://github.com/ota-run/ota/actions/runs/30452821989) and
  [Ota Readiness run 30452822253](https://github.com/ota-run/ota/actions/runs/30452822253) are
  green at pushed candidate `6538cc07`; docs-quality, Smoke, CodeQL, and cargo-deny are green on
  the same source. Bedrock live
  recording/promotion, Kylrix long-running runtime surfaces, and undeclared ambient inputs remain
  outside this bounded policy proof. No new Ota platform gap was exposed.
- completed V11.14 contract-claim assurance: the shared `claim_assurance` domain supplies the
  first additive `ota doctor --json` carrier for declared agent-safe tasks and workflow proof
  claims. It keeps maintainer
  declaration, derived V11.3 closure, policy-independent assurance, and policy decision separate;
  declaration plus closure remains `unknown` without non-self-origin evidence. Its first
  deterministic contradiction is a typed `reset_compose_service_volume` action that omits the
  exact `effects.adapter_state: compose_volume:<volume>` it mutates; opaque shell remains
  `unknown`. `ota proof runtime --json --archive` now creates a content-addressed proof-owned
  record bound to the terminal proof output, archived contract snapshot, clean source identity
  when available, resolved execution, target-platform, host-port, and normalized readiness-timeout
  scope, and explicit witness-only replay posture. The shared
  proof-breadth evaluator consumes only a matching immutable archive: matching failed proof is
  cited as `contradicted`; missing, stale, source-mismatched, or scope-mismatched evidence remains
  `unknown`. Ota-owned `.ota` runtime state is excluded from the source-identity cleanliness check,
  so a fresh archive cannot invalidate its own proof claim.
  Opt-in
  `policies.agent.claim_assurance` requirements now drive the same canonical `deny` or `review`
  decision through Doctor, `ota run --agent`, previews, and `ota up --agent`; default agent
  admission remains unchanged without that policy requirement. Generic deterministic workflows
  without a declared dependency seam can opt into this same assurance through
  `workflows.<name>.proof.claim: bounded`; Doctor reports `bounded_proof` as `unknown` until a
  matching immutable archive exists, then `supported`. Bedrock proves that transition on its
  offline replay lane without inventing seam or negative-control evidence; Lead Quorum proves the
  independent `unknown` path without an archive.
- committed finite command interaction capability (`6b2fa0ca`): structured
  `tasks.<name>.command.interaction` defaults to `auto`, so a native human TTY passes through only
  when available. Explicit `forbidden` preserves deterministic captured execution; `required`
  refuses before any selected task, workflow setup, or dependency work begins when no real
  terminal can be provided. Agent, container, remote, and ordinary non-TTY CI boundaries never
  acquire terminal interaction. The task JSON and dry-run JSON expose the resolved posture and
  invocation resolution; the copy-ready Wrangler OAuth example, canonical skill, public site
  contract reference, schemas, changelog, and regressions are aligned.
- completed V11.18 managed lifecycle-sequence proof: committed lifecycle admission (`10a14971`) and
  the first bounded executor (`d70ca67e`, qualified by `dd3b02cd`). `ota proof lifecycle` selects
  only workflow-declared manager services, leases manager-observed inactive state before start,
  starts in dependency order, reuses transaction-owned services for an optional post-readiness
  assertion, and tears down in reverse order. Typed JSON/schema output binds each record to the
  transaction; a command-only start carries `service_started_state_not_proved`, alongside the
  mandatory application-output and broader-repo boundaries. Focused regressions cover pre-existing
  service preservation and assertion-failure teardown. Runner-owned finalization and a local
  content-addressed lifecycle archive landed in `af74ca4a`: the archive binds
  the semantic contract snapshot, selected workflow/service scope, transaction records, and
  terminal verdict. The command now shares selected-workflow agent admission, mode resolution for
  prerequisite/assertion tasks, and monorepo member loading; service-manager controls remain on
  their declared boundary. Archive scope and reader verification landed in `3607b6a7`; the current
  correction binds the resolved service closure/mode and contract/source identity, verifies archive
  filename plus snapshot staleness, and records typed interruption finalization with exactly-once
  teardown regressions. Multi-service dependency rollback now proves reverse, exactly-once
  finalization after a later start failure or interrupted later start; the matching local archive
  verifies the full closure, service records, transaction, and finalization binding. A
  runner-observed readiness interruption now emits a typed `interrupted` transition before the
  same finalizer runs. A stop-command failure retains typed transition evidence and does not
  prevent other leased services from finalizing; an interrupted teardown with unproved manager
  cleanup is explicitly `incomplete_after_interruption`. Focused local runner/archive/schema
  validation is complete; the lifecycle-lock correction is committed in `b3a018a7`. V11.18 does
  not reopen V11.15 provider-neutral or GitHub projection: lifecycle proof stays a local command
  with dedicated provider-owned pressure workflows until a later slice defines lifecycle-specific
  adapter semantics. **Pressure provenance correction:** the remote Flagr branch
  `bobai/flagr-v11.15-deployment-pressure` at `8881fbe` does not yet declare
  `workflows.integration.proof.lifecycle`; it cannot substantiate the recorded lifecycle archives.
  Treat those local records as implementation evidence only, not Flagr pressure proof. Open WebUI
  now provides the Compose pressure side: its pinned `5ac1388784` matrix and lifecycle-control run
  prove declared Docker health probes as `service_readiness`, successful lifecycle finalization,
  and a controlled assertion failure without copied shell cleanup. Caddy closes the isolated
  lifecycle boundary locally: its current-Core archived container proof
  `sha256:8cb602aa552d653c3a5d3e465e934b7ed773b8305235aa7bf1775c274c65a27e` runs the upstream
  structured `caddy start` / `caddy stop` commands inside one transaction-bound ephemeral session
  and attests only engine-confirmed session removal as `boundary_terminated`. It explicitly retains
  `service_started_state_not_proved`, `application_output_not_proved`, and
  `broader_repo_completion_not_proved`; it never claims `manager_inactive` or host process absence.
  Caddy matrix [30102633474](https://github.com/bobaikato/caddy/actions/runs/30102633474) is green
  for regenerated native and container governance lanes against committed Core `6025187b`, but it
  does not invoke `ota proof lifecycle`; it is not isolated-boundary pressure proof. The dedicated
  hosted lifecycle run [30111427705](https://github.com/bobaikato/caddy/actions/runs/30111427705)
  is green against Core `3ffaf362` and binds the exact runner-owned boundary identity. The final
  hardened rerun [30124528078](https://github.com/bobaikato/caddy/actions/runs/30124528078) is
  green against Core `53ff07eb`, emits qualified proof, and archives
  `sha256:8985f57cd191e4d1db370122a6adb33a5f3a3fc649a289b2855dd2b48894de39` with exact session
  `container:docker:ota-ephemeral-43c71044194b0e05`. The final setup-failure correction passed
  [30125996749](https://github.com/bobaikato/caddy/actions/runs/30125996749) against Core
  `fc88d215`, emitting archive
  `sha256:c2dab2e7535589819f416159ca06b5384d599093e87546cc1aab1b242d8e3235` with exact session
  `container:docker:ota-ephemeral-3890811a19b2944d`. Open WebUI supplies the independent Compose
  readiness/teardown family. V11.18 is complete: implementation, pressure evidence, and final
  independent review passed with no release blockers.
- completed V11.16 fresh-boundary setup proof: `ota proof runtime --json` and archived runtime
  proofs carry a content-addressed `execution_boundary` graph. Native `ensure_virtualenv` plus a
  runner-recorded `.venv/bin/*` consumer, and frozen native pnpm hydration plus a declared local
  consumer, carry runner-attested precondition, producer, and `asserted_at` identities before
  deriving `cold_start_verified` or `persistent_state_reused`. The pnpm carrier binds its
  generated `node_modules/.modules.yaml` layout marker to the declared lockfile rather than
  claiming whole-tree hashing. The evaluator canonicalizes graph identity and rejects ambiguous,
  forged, stale, cross-scope, or causally mismatched edges before proof JSON or archives emit.
  Lead Quorum run `29742813235` proves fresh and reused virtualenv evidence; OrchardCore run
  `29697072972` proves an ephemeral typed .NET container closure; Athena run `29786128386` proves
  container Bundler fulfillment while retaining PostgreSQL lifecycle/output boundaries; and Kylrix
  run `29828933200` closes the native pnpm carrier with absent `node_modules`, a `setup` producer,
  a matching local `dev` assertion, and `cold_start_verified`. Provider state, databases, services,
  volumes, general container filesystem state, Windows virtualenvs, and uninstrumented package
  layouts remain `unknown`; V11.16 does not claim repo-global cold-start proof.
- completed V11.15 managed CI projection: `ota ci projection --workflow <name> --mode <mode> --target-os <linux|macos|windows> --json`
  now emits the provider-neutral governance lane with a semantic identity, merge-check identities,
  proof requirement, and provider-neutral ownership categories. The GitHub adapter consumes that
  object through one renderer powering `ota ci github render`, `check`, and atomic `sync`.
  It emits separate projection, render, and parsed-caller binding identities; generated content
  runs validation, doctor, safe discovery, agent dry-run, execution, receipt archival, and a
  declared runtime proof when the selected workflow owns one. Agent-safe lanes retain `--agent`;
  proof claims do not bypass agent admission, and proof-required lanes use one authoritative execution.
  Each unique contract refusal canary is now emitted as its own provider check with a stable
  `merge_check_id`; the GitHub adapter publishes the scope-qualified provider-check mapping so
  native/container and OS lanes stay independently requireable. The generated check invokes Ota's
  `--expect-refusal` runner boundary directly. Projection also carries selected-closure,
  provider-neutral `toolchains[]`; the GitHub adapter renders Go setup from the contract through
  an immutable Action revision and refuses unsupported required sources rather than relying on an
  ambient hosted-runner toolchain. Aggregate execution-mode admission now uses the same concrete
  member-closure rule as projection and task discovery, preventing a valid container projection
  from later failing before aggregate members run.
  Kylrix renders valid distinct native
  and container reusable lanes without collapsing its separately selected `sqlite-dev` runtime
  proof into `verify`; its committed caller/matrix preserves the existing native/container
  evidence. NopCommerce independently proves generated .NET verification on both native and
  container lanes: the native lane projects the declared .NET 10 toolchain through an immutable
  `actions/setup-dotnet` revision, while the container lane uses only the declared SDK image.
  Both lanes bootstrap Ota from the pressure contract, verify their projection identity, run the
  agent-admitted workflow closure, and archive a receipt in GitHub run `29686807594`. Strict
  V11.14 agent and proof assurance admission is evaluated before projection render/check; denied
  or review-required lanes return their canonical refusal rather than generating a green wrapper.
  Outline then exposed a projection/runner mismatch: a safe run task could be admitted while an
  unsafe setup phase was later refused by `ota up --agent`. Projection now shares the runner's
  ordered prepare/setup/run/attach admission roots, and Outline's unchanged `checks` workflow
  returns the same inspectable `requested_task_not_safe` refusal from projection and render.
  A clean Flagr deployment pressure clone then exposed a second renderer defect: finite workflows
  used readiness-only `ota up` as their only generated execution step. Projection now binds
  `run_execution` as `finite_task` or `service_runtime`; finite lanes retain a dry-run `ota up`
  admission preview, then execute their selected closure directly through `ota run --agent`.
  Compatible ephemeral container closure steps now share one runner-owned session, so typed
  hydration state survives into its finite consumer without leaking across CLI invocations.
  OrchardCore proves that .NET restore/build/test path locally in both native and container modes;
  its tag-triggered release CI remains provider-owned and untouched. Its pushed matrix and Caddy's
  independent green native/container governance matrix now satisfy the two-repository pressure
  target. Caddy also hardened the GitHub adapter's Go lower-bound projection: a valid one-sided
  contract range such as `>=1.25.1` now renders its explicit lower release through immutable
  `actions/setup-go`. Caddy's upstream start/stop shell smoke remains separately modeled and
  explicitly outside the generated build/test lane; Ota must not call that narrower lane full
  upstream CI equivalence until it can recover the lifecycle assertion without reducing it to a
  command-shaped approximation. The final V11.15 review passed focused neutral projection,
  GitHub renderer, JSON-conformance, and formatting checks; Kylrix plus OrchardCore/Caddy satisfy
  the two-repository pressure bar. V11.15 is complete.
  Projection identity now reuses the canonical normalized semantic snapshot identity used by
  receipts; omitted mode resolves from the selected task's effective contract default, while an
  unavailable explicit mode is refused. Denied provider-neutral JSON preserves the evaluated
  projection and typed refusal. Managed workflow paths reject symlink escape, and the neutral
  projection carries bootstrap posture, proof claim, and target-OS identity for the first GitHub
  adapter.
- V11.3 refusal-canary implementation is pressure-proven on Athena and Kylrix: `agent.refusal_canaries` names
  one task or workflow negative control, and `ota run --agent --expect-refusal <task>` or
  `ota up --agent --expect-refusal --workflow <workflow>` passes only when the agent-safety
  closure refuses before selected work begins. A policy-only denial is the failing
  `wrong_refusal_boundary` outcome. The contract never supplies an expected reason; Ota emits the
  runner-derived refusal and blocked receipt for later comparison. First-party docs/site/skill are
  aligned, and the released safe-agent-execution example carries the canonical refusal-canary
  pattern.
- completed `V11.10` replay trust refinement: `ota up
  --replay-baseline ... --json` now carries replay-authored baseline posture directly through
  `replay.baseline.last_known_good`, while declared static replay inputs remain receipt
  `evaluated_inputs[]` and Bedrock-style historical query traces stay separate as attested
  `witnessed_observations.query_traces[]`; plain-text replay output now mirrors the same trust
  split by rendering matched acquitting, narrowing, and pointer-only evidence separately from
  changed inputs, and hermeticity now requires at least one matched material runtime,
  dependency-resolution, or presentation anchor rather than over-reading same-contract reruns as
  hermetic; hidden-input replay failure now emits ordered `hidden_input_candidates` so operators
  can promote the next likely ambient class instead of reading one generic suspicion bucket
- completed V11.12 typed hydration provenance across two ecosystems: successful `ota up --json`
  records selected structured hydration lanes through typed `receipt.evaluated_inputs[]` hydration
  records. The record captures contract-declared source posture and runner-resolved feed identity
  before execution, preserving explicit `resolution: unavailable` when source choice remains
  ambient. Azure SDK for .NET proves config-backed NuGet identity; Lead Quorum proves explicit uv
  PyPI index posture across native and container lanes on Ubuntu, macOS, and Windows. Ota projects
  uv index truth through flags supported by both older and current uv releases. Replay treats
  unavailable hydration resolution as narrowing evidence, never as a hermetic dependency-resolution
  anchor.
- completed V11.9 governance reconciliation: Athena exposed a preview that named
  `not_run_reason: preflight_refusal` while reporting `refusal_occurred: false`. The canonical
  preview now carries the same refusal record, reason, and basis in both phases while preserving
  post-execution `state: not_run` because no execution began.
- completed V11.11 proof-boundary and seam-control carrier: `ota proof runtime --json` publishes
  qualified proof verdicts, scoped `not_proved` boundaries, provenance-aware seam evidence, and
  canonical negative-control records. Athena's Rails/PostgreSQL lane proves transaction-bound
  marker recovery and same-obligation fault control across its green matrix. Every marker-bound
  seam retains `dependency_output_shaping_not_proved`; invariant coverage proves the pairing and
  derived control projection. Generic `ota up` and ordinary receipts intentionally do not inherit
  this evidence because they did not execute the proof lane. The canonical runtime-proof example
  now demonstrates the same carrier end to end. Compose dependencies started by proof are cleaned
  on success and readiness failure, while services already running before proof are preserved.
  Validated dependency projections now name their canonical negative-control record and bind its
  exact failure-attestation digest; schema rules prevent `fault_tested` evidence from omitting or
  downgrading that validated projection, while Core now reconciles the canonical ID, dependency,
  obligation, and digest relationship before emission and archive loading. Archive reads derive a
  selected control from archived contract/scope truth and require exactly one canonical record and
  matching projection; other consumers must apply that same rule because JSON Schema cannot
  compare sibling values.
- completed V11.13 generated-artifact lineage: Dagger proves the generator path and EventCatalog
  proves an independent sibling-consumer closure. Contract-owned producer, output-path, and input
  lineage is validated, surfaced in task discovery, checked before consumer execution, and carried
  into receipts as pointer-only evidence without claiming freshness.
- completed the 1.6.24 release-readiness sweep: the active pressure set has no unresolved Ota
  platform gap, V11.11 proof evidence is propagated through the canonical and public examples,
  skill guidance, site reference, generated contract schema, and changelog, and the complete native
  `release-gate` passes. Lead Quorum's newest local-only pressure commit remains unpushed and is not
  part of the matrix-backed release claim.

- V11.10 Bedrock replay proves native baseline replay as `replay_verified` and `partly_ambient`.
  A container replay against that native archive correctly returns `replay_unavailable` with
  `baseline_scope_mismatch`: workflow, backend, provider, remote target, and lifecycle identity are
  required for `last_known_good`.
  A freshly archived container witness then replays as `replay_verified` and `partly_ambient` on
  the same container/ephemeral scope. Backend-scoped informational doctor notes remain visible but
  do not stale an otherwise same-scope witness.

## Recent Completed Slice

- Kylrix pressure exposed two connected Ota execution gaps and proved their fixes on the
  deterministic SQLite contributor lane. `launch.runtime_projection.adapter: nextjs` now
  projects `--hostname` / `--port` from the runtime listener into direct `next dev` launches,
  while validation rejects package-script wrappers that would make projection ambiguous. Dry-run
  input resolution now recognizes `ensure_env_file` output from the selected dependency closure as
  planned setup state on a clean checkout, while real execution waits for dependencies and then
  validates the rendered dotenv input. Published contract-schema coverage was synchronized for
  command runtime projection and already-shipped generated workflow-instance fields; the full
  examples gate now passes. Kylrix itself proves idempotent SQLite env materialization, agent-safe
  Vitest/lint/build verification, workflow preparation, archived receipt, and isolated native
  runtime proof. Its interactive Appwrite topology and credential/schema-provisioning paths remain
  explicitly outside this narrow proof.

- Kylrix also exposed a native long-running task UX gap: applications such as Next.js can exit
  non-zero after a user `Ctrl+C`. Explicit runner interruption evidence, or a raw signal before a
  clean completion, returns canonical exit `130` with an `interrupted` receipt and summary. A late
  raw signal cannot overwrite an already-established non-interrupt task or service failure.

- Dagger generated-SDK pressure exposed and fixed native source-managed tool activation: a
  release-asset tool was fulfilled and version-probed correctly, but the native shell task path
  discarded the managed PATH before executing its command. Ota now applies the resolved PATH to
  native shell execution and has a focused regression test. The narrowed Dagger contract proves
  release-asset fulfillment, workflow preparation, selected generator execution, generated-source
  lineage, a clean consumer diff, scoped doctor, and archived receipt locally. The selected
  closure requires Dagger v0.21.7 despite root `dagger.json` still naming v0.21.0; that is
  recorded as repo truth rather than hidden by the pressure contract.

- Task platform availability is now contract-owned. `tasks.<name>.only_on` uses the same
  `linux` / `macos` / `windows` vocabulary as prerequisite and context scope; runner planning,
  `ota run`, and dry-run preview refuse an unsupported dependency closure before side effects.
  `ota tasks --use` marks context- or task-unavailable modes non-callable, and doctor filters
  unavailable closures before probing their requirements. Athena pressure uses this truth through
  its Linux/macOS Ruby context and proves the expected Windows refusal rather than hiding it with
  a workflow skip.

- The current V11.10 refinement adds contract-owned
  `tasks.<name>.witnessed_observations.query_traces[]` for existing JSONL query traces. Ota
  validates immutable repo-relative trace paths, captures the selected closure before execution,
  and emits source identity, full run records, and divergent-subject summary under receipt
  `witnessed_observations`. It deliberately keeps the trace outside `evaluated_inputs[]` so
  historical observed behavior cannot be over-read as a current-run decision input. Bedrock's
  recorded SQL trace proves the narrow admission: three subjects diverge while stable repeated
  queries retain one identity across runs.
- The completed `V11.13` core cut makes generated source a named repo-scoped contract artifact:
  `artifacts.<name>` declares `kind: generated_source`, one producer task, output paths, and
  optional source inputs; consumers declare `requires_artifacts` and directly depend on the
  producer. Validation rejects dangling, overlapping, and dependency-disconnected lineage. The
  runner checks declared outputs after the producer closure and before consumers execute. Task
  JSON carries the producer map plus consumer references, and receipt `evaluated_inputs[]` captures
  producer/path/input lineage at issue time as pointer-only evidence, never as a freshness claim.
- EventCatalog pressure proved the first healthy generator and sibling-package consumer closure:
  typed `pnpm --filter @eventcatalog/language-server install`, Langium generation, and the
  downstream VS Code extension build. It also widened `prepare.source.filter` from its old
  browser-bootstrap-only boundary into a pnpm-owned dependency-hydration selector. The first
  sibling build failure identified real missing SDK and visualiser build dependencies; modeling
  those finite tasks made the final extension build pass without shell orchestration.
- Bedrock pressure proved the V11.10 replay-artifact shape on a deterministic offline NL-to-SQL
  stability harness across Ubuntu, macOS, and Windows: explicit script-test aggregation, committed
  SQL fixture replay, and the defended baseline gate all run in agent mode with no model key. Its
  live recording lane remains intentionally outside that claim because it reaches Claude, rewrites
  the fixture, and depends on an unpinned generic-pip requirements path that Ota does not yet own
  through typed dependency hydration.
- The completed V11.10 replay carrier distinguishes whether the selected baseline is still the
  last known good witness. `ota up --replay-baseline ... --json` adds
  `replay.baseline.last_known_good` with `replay_verified`, `stale_witness`, or `unavailable`
  derived from the replay result itself, so promoted archives no longer all read as equally
  current after drift or unavailable-baseline failures.
- V11.10 also names active execution governance as a replay-grade input.
  Receipts now capture a loaded org policy pack as `policy_ruleset_identity`, and replay treats a
  changed ruleset as named input drift rather than generic hidden-input suspicion.
- V11.10 also names declared env-source files when the selected lane
  actually resolved from them. Receipts capture `env_source_identity` without recording values, so
  replay can distinguish declared env-source drift from still-ambient process or policy env.

- The completed task-discovery UX batch renders closure-aware `Human Run`, `Agent Run`, and
  `Agent Policy` sections in `ota tasks` and `ota tasks --use`. It keeps the `ota-site` internal
  verification setup task agent-callable so its declared-safe public verification closures remain
  truthful without exposing setup in the default task inventory. Task mode rows now use stable
  `Container`, `Native`, then `Remote` presentation, show unsupported local planes explicitly,
  and recover native override support for container-context tasks without requiring a redundant
  task mode branch. `ota tasks --json` now carries the same canonical per-mode truth under
  `tasks[].use.modes[]`, while the existing `use.human` and `use.agent` remain selected-mode
  compatibility projections.
- Flagr pressure confirmed native and container Go module hydration, binary build, aggregate
  verification, and runtime proof. It also closed two task-discovery regressions: aggregate mode
  rows now inherit executable closure support, and Doctor no longer version-probes repo-owned
  command paths before their producer task exists. The previously open locally tagged Dockerfile
  image-build gap is now closed by first-class `action.kind: build_container_image`: contracts own
  the provider, Dockerfile, repo-relative context, and local tag without raw `docker build` glue.
  Lead Quorum proves the direct image build; Flagr carries the equivalent integration-image task,
  with its live build awaiting a healthy Docker daemon on the pressure host.
- `V11.11` contract-derived proof boundaries is implemented in Ota commit `e3bbdf02`.
  `ota proof runtime --json` now emits terminal `proof_verdict`, and Lead Quorum pressure proved
  `passed_with_unproven_boundaries` on the app lane across Ubuntu and macOS.
- V11.11 keeps that qualified proof boundary visible in human output too:
  `ota proof runtime` now renders concrete `Proof Boundaries` entries whenever `not_proved[]`
  exists, so external-network and broader-scope exclusions travel with the green proof instead of
  living only in JSON.
- V11.11 makes those proof boundaries machine-actionable too: each
  `not_proved` entry now carries an explicit `reason`, and the human proof render includes the
  same reason label for seam, adjacent-lane, and broader-scope exclusions.
- The completed V11.11 seam-evidence carrier on
  `ota proof runtime --json`: `dependency_evidence[]` now publishes runner-derived
  `level: reachable` only for declared service seams that are also on the selected
  workflow-owned required-service path and have structured readiness Ota actually owns. This
  keeps selected service reachability distinct from still-unproved exercised interaction.
- V11.11 keeps caller-side seam attempts separate from proved reachability:
  proof-derived DNS, auth, and loopback service failure signals can now publish additive
  `interaction_attempted: true` with `observation.origin: caller_side`, while the paired
  `dependency_exercise_not_proved` boundary tightens to `caller_side_only_evidence` instead of
  generic missing evidence.
- The same commit fixes detached native proof lifecycle ownership: nested `ota up --detach` leaves
  the service running for the outer proof to observe and clean up, preventing recursive teardown.
- V11.10 emits a runner-derived receipt-comparison artifact-trust record for matching
  semantic contract snapshots. It is `acquitting` for `contract_truth` only; lockfile/runtime
  artifact capture remains the next implementation cut.
- V11.10 captures declared lockfile-strict Node identity in
  `receipt.evaluated_inputs[]` at receipt authoring time: `pnpm-lock.yaml` for frozen pnpm and
  `package-lock.json` or authoritative `npm-shrinkwrap.json` for `npm ci`. It carries this through
  archived baseline and current receipt diff and labels only matching
  `declared_dependency_resolution` identity as `acquitting`. Directus and ota-site proved matching
  archived/current paths with the source-built binary; unrelated runtime findings remain separate.
- Lead Quorum is not yet the first hermetic replay target: its typed `uv pip_requirements` lane
  and Python range are real current repo truth, but not the pinned dependency/runtime pair V11.10
  needs. Treat that as a repo contract/replay-readiness gap, not a reason to weaken Ota evidence.
- V11.10 captures `runtime:node` through contract-local `node --version` on the
  same typed lockfile-strict Node hydration path. It is deliberately `narrowing` for
  `selected_runtime_version`, not an executable/image-digest acquittal.
- V11.10 adds the first immutable runtime-artifact carrier. Receipts
  recover literal digest-pinned Compose `image` values only for explicitly selected services in
  explicitly declared files and their declared Compose `depends_on` closure as
  `selected_runtime_artifact`; receipt diff treats a matching digest as `acquitting` only for that
  named artifact. Mutable tags, interpolation, inferred files, and unrelated stack services remain
  outside the claim. Immich pressure also exposed and fixed an Ota runner gap: Compose adapter-file
  preflight now resolves files relative to the same adapter `cwd` used by execution. The narrow
  Redis/PostgreSQL launch, status, and stop path passed locally with the source-built binary.
- Immich and Grafana confirmed the follow-on taxonomy need. `effects.network_kind:
  container_image_hydration` now owns registry-backed Compose image acquisition independently from
  package dependency hydration; `prepare.medium: container_images` requires this label, doctor and
  policy packs expose the same lane, and immutable image receipt evidence remains separate from
  the effect declaration.
- The same branch upgraded the direct `quick-xml` dependency to `0.41.0` after `cargo deny`
  surfaced the two XML denial-of-service advisories in `0.38.4`; the NuGet feed-provenance parser
  uses the current XML 1.0 attribute-normalization API and its focused tests pass.
- Grafana confirmed the receipt carrier on a mixed Compose stack with locally built, mutable, and
  digest-pinned services. The selected observability lane records four explicit digest-pinned
  services plus `tempo-init` through Tempo's declared `depends_on` closure, while excluding
  unrelated built and mutable stack services. This exposed and fixed the closure-recovery gap in
  Ota rather than leaving the init image absent from a selected runtime receipt.
- The same Grafana pass exposed and fixed a doctor semver gap: whitespace-separated compound
  ranges such as `>=1.26.3 <1.27` now use the canonical normalized semver path while preserving
  Ota's established shorthand comparator behavior.

## Handoff To The Next Chat

Start by reading `AGENTS.md`, this file, the canonical Ota skill, and
`docs/planning/v12.1/plan.md`. Then inspect the actual Core, Protocol, Launcher, Site, Skills,
and Examples worktrees before editing.

The sole active implementation version is V12.1, Step 7. The protected Linux/X64 capability and
endpoint compatibility gate is closed by the exact retained runs and artifacts recorded above.
Those runs prove bounded protected capability derivation, signed public projection reconciliation,
selected execution-path admission, and terminal cleanup. They do not prove a real GitHub OIDC
request, Google provider contact, secret materialization, process-environment injection, positive
provider evidence, Step 8, V12.2, or general agent/repository governance.

The provider-free same-child transaction bridge is committed, independently reviewed, and now
proved through the exact protected Linux/X64 service path. Protocol
`2c46cb676ef6e0844312bd6a157adec7ccb54de1`, Launcher
`a00f0e1ebd0ba886015a1aa32216ad30b56dcd9a`, and Core
`c071fed09ffe42d4ac00979d16dff0967ad208aa` bind one selected child's signed observation,
protected authority snapshot, reconstructed Step 1-6 candidate, and one-use snapshot-bound V2
transaction on the same inherited startup session.

Core run [34933139633](https://github.com/ota-run/ota/actions/runs/34933139633), protected job
`104265387910`, passed the complete provider-free route. It executed exactly one ignored Linux
capability regression; reconciled one observation, same-child prelude, authority snapshot, and V2
response; recorded zero protocol mismatch/refusal counters; reached exactly one expected
provider-free refusal; did not start the selected task; retained no raw protected capability
identity or private correlation material in public output; and completed child reap, scope removal,
cgroup cleanup, and active-slot removal. Toolkit job
`104265370625` separately proved only that pinned Toolkit code used its supplied loopback URL while
the action started without an OIDC request capability. MUSE independently reviewed the exact run
and found no P1/P2/P3 issue.

The administrator-retained VPS bundle at
`/opt/ota-actions-runner/_work/_ota-pressure-evidence/34933139633-1` passed its checksum manifest,
but it is not a publicly durable GitHub artifact; GitHub retains only the bounded Toolkit probe.
No real GitHub OIDC token request was made. Provider contact remains `not_instrumented`; Google
STS/WIF, Secret Manager access, materialization, process-environment injection, positive provider
evidence, Step 8, and V12.2 remain unproved and inactive.

The separate Step 7 provider-contact slice amendment is committed at
`bcc4090bb038211568319749fb6e08c6b60470f4`. It
binds the first transaction to one administrator-controlled non-production Google installation,
the dedicated manual provider-pressure workflow, its administrator-bound exact workflow revision,
the exact same-child transaction, closed OIDC, STS, service-account-token, numeric Secret Manager
version, recipient-only injection, interruption, cleanup, privacy, and no-replay boundaries. A
pressure-only protected HMAC verifier owns expected synthetic-canary truth without storing the
canary value or creating public secret-derived correlation. Concrete provider references and private
identities remain protected and are not copied into public planning or workflow input.

The first implementation batch was independently reviewed and committed at
`77329526e6ec07449c889bac6c3172294620ec4e`. Its crate-private, network-disabled provider client
model derives exact OIDC, STS, IAM Credentials, and numeric Secret Manager operation
targets from semantically verified candidate truth; constructs only protected in-memory request
bytes; and parses bounded closed response shapes with exact token types, lifetimes, resource names,
canonical encodings, and CRC32C reconciliation. It has no environment, filesystem, socket,
HTTP-client, V2-consumption, materialization, injection, execution, or public-output consumer.

The authority-bound transport-preparation checkpoint was independently reviewed and committed at
Core `8350b479`. It consumes and semantically re-verifies one exact same-child snapshot-bound V2
transaction before acquiring the fixed GitHub OIDC request URL and bearer through one private,
process-wide one-use owner. It reconciles the candidate, protected operation plan, capability
projection, endpoint observation, and runner version before retaining a fresh `ureq = 3.4.2`
configuration with default features disabled, Rustls/WebPKI roots, no proxy, no redirects, no
caller trust or client certificate, and bounded DNS/connect/send/read/global, header, and body
limits. The checkpoint exposes no request send/call, socket, connector, provider response,
materialization, delivery, execution, or public-output path. Focused provider-client tests, the
fresh-second-V2 ownership regression, locked all-target/all-feature compilation, formatting, and
diff checks passed; MUSE reported no P1/P2/P3 finding. Repository-wide strict Clippy remains red on
the pre-existing broad baseline outside this slice.

The build-owned transport-dependency foundation is committed at Core `cd8e2433`. It derives one
closed record from locked Cargo metadata for `x86_64-unknown-linux-gnu` with
`secret-delivery-pressure`, binding the exact raw `Cargo.lock` bytes, normalized `ureq` closure,
features, source/checksum nodes, normal/build edges, and canonical target expressions. Core
reconciles that identity through the implementation subject, invocation binding, protected
authority payload, transaction candidate, provider operation plan, consumed V2 capability, and
prepared transport. It rejects a non-normal root edge, noncanonical target expression, graph/lock
substitution, and inconsistent local carriers. It does not yet bind the record into the separately
administered Protocol/Launcher installation or prove cross-process agreement.

The additive V3 transaction exchange is committed in Protocol
`63a352f7a926a2fed0866db0de61749fa701df75`, Launcher
`19b4b15d79193c5e74fcf0771011ea0ad868755d`, and Core
`d30cdb278d1db619c91e7f63bec03b6ab0687b7`. Core
`fc2357ba35bbef979a4af0155f81962fa2608f41` routes the protected same-child session through
V3 after snapshot and candidate reconstruction; its broker session suite passed 33 tests and MUSE
found no blocking finding. The exact Core-owned transport-dependency record identity is retained
in the V3 request and binding. V2 remains immutable historical proof and the completed
network-disabled V2 transport-preparation checkpoint; that checkpoint is not yet migrated to V3.

Run [35095607162](https://github.com/ota-run/ota/actions/runs/35095607162) reached the protected
Linux/X64 host at exact Core `f59c35e9`, Launcher `19b4b15d`, and Protocol `63a352f7`, but failed
before capability observation: Cargo could not fetch the new locked Protocol revision into the
protected runner's read-only persistent Cargo home. The independently reviewed Core repair
`d8fd435514a8dedfe83a80b432d71c9ba2d40e12` moves only that test's Cargo home and target directory
under `RUNNER_TEMP`; it does not alter authority, provider, or execution behavior.

The exact protected Linux/X64 V3 compatibility gate is now closed by Core run
[35101698250](https://github.com/ota-run/ota/actions/runs/35101698250): Toolkit job `104812464123`
and protected job `104812509395` both passed at Core `d8fd435514a8dedfe83a80b432d71c9ba2d40e12`,
Launcher `19b4b15d79193c5e74fcf0771011ea0ad868755d`, and Protocol
`63a352f7a926a2fed0866db0de61749fa701df75`. The protected job rebuilt the isolated cache, derived
and reconciled one signed public capability observation, verified the protected runner's non-secret
endpoint posture, reconstructed the same-child V3 route, emitted exactly one
`binding_v3_response_reconciled` marker and zero V2 `response_reconciled` markers, then refused
exactly once at the expected provider-free boundary. Public evidence retains no raw capability
identity, bearer, or protected request URL. This is V3 cross-process compatibility proof, not a real
GitHub OIDC request, provider contact, Google STS/WIF or Secret Manager operation, materialization,
injection, positive evidence, Step 8, or V12.2 proof.

The prior V3 gate measured its complete graph at 32,309 canonical bytes; the generated graph size
must be rechecked for each build. Protocol `731737048e3bf8912145e404200bdf4264d3e7c8`
raises only the protected binding-bundle payload limit from 32 KiB to 40 KiB, while preserving the
64 KiB protected-store bound. MUSE independently reviewed that adjustment with no P1/P2/P3 finding.
Core cannot yet carry the complete graph through the released V1 authority snapshot because V1
duplicates the signed bundle as typed fields and base64url bytes inside its frozen 64 KiB frame.

The additive V2 authority-snapshot and V4 transaction-binding amendment was independently reviewed
and committed at Core `93a36b23`. Protocol V2 snapshot records were independently reviewed,
committed, and pushed at `c6bee9b49dc599ccd94bf6b0e60a1894614cb5a4`; additive V4 request,
binding, response, and reconciliation records were independently reviewed, committed, and pushed at
`e819f95890ea23ae2f336a59fb3ff62cfa858d8b`. Launcher `35da0ef` and Core
`083d9d62` bind the focused V2 snapshot/V4 reconciliation and service relay. Core reconstructs one
provider-free candidate from verified V2 payload, requests V2/V4 on its refusal lane, and rechecks
V4 before one-use consumption. Launcher retains live capability derivation, reserves and consumes
the V2 exchange in its descriptor-retained replay store, and relays V2/V4 only through the selected
child session. The fixture, exact-revision workflow, and bounded runbook were independently reviewed
without P1/P2/P3 findings.

The exact protected Linux/X64 V2/V4 service-path gate passed in Core
[run 35210008681](https://github.com/ota-run/ota/actions/runs/35210008681): Toolkit job
`105165017052` and protected job `105165215459` ran against Core
`083d9d62620fe6f092f51085237c6089ab97777c`, Launcher
`35da0efc2c77995321be39cb3a131acf8623f3e3`, and Protocol
`e819f95890ea23ae2f336a59fb3ff62cfa858d8b`. The protected job derived and reconciled one signed
public capability observation, one authority-snapshot V2 response, and one binding V4 response;
recorded no listed protocol mismatch/refusal counters; reached exactly one expected provider-free
refusal before task execution; left `selected-work-executed` absent; and completed child reap,
scope removal, cgroup cleanup, and active-slot removal. The separate Toolkit job started without an
OIDC request capability and proved only that pinned Toolkit code used its supplied loopback URL.

The protected host retains a checksummed bundle at
`/opt/ota-actions-runner/_work/_ota-pressure-evidence/35210008681-1`. It is job-owned local
evidence available to an administrator through root access, not administrator-owned, independently
durable, or public GitHub artifact evidence; its checksum manifest is created by the job. GitHub
retains only the bounded Toolkit probe artifact. A stronger long-term custody claim requires a
separate administrator-owned capture path and a new run. No real GitHub OIDC request was made;
provider contact is `not_instrumented`. Google STS/WIF, Secret Manager access, materialization,
process-environment injection, positive provider evidence, Step 8, and V12.2 remain unproved and
inactive. Site, Skills, Examples, Learn, FAQ, Glossary, contract schema, and public JSON remain
unaffected because this remains an internal provider-free trust-boundary foundation.

The hosted-evidence custody implementation is in Core `7ef8af1a552ffcc97cbff9550aee47016f4faf38`
and Launcher `19b4f7af4527073ef15520d9a6b43b7bcc654229`; Core `65cefc753dd762f34b5939d244316a1a9e829e79`
pins the reviewed Launcher `84eefe0a8ffc2a3d8ff745d54850d3c5312319d1` documentation revision, while
Core `61efe2544b0d8cd0d8aeb7297a3e3dd19cf97358` records the corresponding first-party waiver
reconciliation. No successful custody run exists yet. On 22 September, Core workflow
[`35718397185`](https://github.com/ota-run/ota/actions/runs/35718397185) at
`f4381cafe5bfc9fff5e67a160aad5c216d94f077` reached
the self-hosted queue while the runner remained stopped, but was cancelled before execution. Its
fresh root-owned request was removed after the cancellation. On 22 September, an independently
reviewed one-time administrator-owned operational reset of this borrowed host completed; it is not
a shipped Ota retirement capability. The root-only helper (SHA-256
`8c10f83dcd2fd958d7ebe7785fbcd64b53d2cf28a28c7814f9db83edfc2ebffa`) captured a closed 49-entry
managed inventory and 26 protected ancestor records, retained only three allowlisted public
installation records in a root-owned archive, destroyed the prior private authority state and
signing keys, and removed all 48 reset targets. Independent post-reset review verified that every
authority unit is absent/inactive, managed sockets and runtime are absent, the protected runner
service, hardening drop-in, binary, checkout, accounts, and GitHub runner registration remain
intact and inactive, and no job/execution principal remains. The helper journal is terminal and
must not be rerun or resumed. At that reset boundary, no new authority installation, request,
dispatch, runner start, or hosted custody proof existed. The first fresh attempt,
[`35730801737`](https://github.com/ota-run/ota/actions/runs/35730801737), at Core
`f4381cafe5bfc9fff5e67a160aad5c216d94f077`, Launcher
`ca8d4ab342fdc775a534767374ab15ead583a468`, and Protocol
`e819f95890ea23ae2f336a59fb3ff62cfa858d8b`, reached the protected provider-free command and
retained its job-owned closed evidence set, but its final root-custody reconciliation failed. The
root capture service correctly failed closed: its store directory was held through an `O_PATH`
descriptor that cannot `fsync`, and its `UMask=0077` reduced the intended public capture record
from `0644` to `0600`. That run establishes neither custody nor the broader hosted gate; the
runner was stopped immediately after it became terminal. Launcher `eee7dc636b0cc3a89d33ce6b1b82d5dfa4c56032`
reopens only the verified directory through constrained descriptor-relative `openat2`, then writes
and data-syncs the complete temporary record before explicitly finalizing it to `0644`, verifying
its exact protection, and publishing it. Independent review found no P1/P2/P3 issues; focused
Linux/X64 capture tests passed `6/6`, the capture-unit assertion passed `1/1`, and local
`ota run verify --agent` passed. Core `b9c5d0b65eaca9d15d975e5913ba16135337e6b7` pins that immutable
Launcher repair. The failed state was archived and not reused.

The required fresh special sequence then completed in Core
[`35748662341`](https://github.com/ota-run/ota/actions/runs/35748662341), attempt `1`, dispatched
at `1.6.28-implementation` / exact Core `b9c5d0b65eaca9d15d975e5913ba16135337e6b7`. Its protected
job `106816778715` ran on `ota-authority-aws-16-171-42-182` under Runner.Listener `2.337.0` only
after a root-owned mode-`0400` request and fresh provisioned state bound that exact run and attempt.
Both GitHub jobs passed. The root capture service copied the closed five-file success set into
root-only `0700` state, recomputed and verified its checksum manifest, and published exactly one
root-owned `0644`, no-clobber public record at
`/usr/share/ota/authority-launcher/hosted-evidence-captures/35748662341-1.json`. Its canonical
record identity is `sha256:ed085656fe9c16254429bfaffb971ee13fe2df6f2c75608b9c09e93896f8b64f`, its
root-computed bundle digest is `sha256:f334e62c4b863696e2f857aea840e216d2f037a61f0ce64fef9c50f3d6d8bfd4`,
and it binds exact Launcher `eee7dc636b0cc3a89d33ce6b1b82d5dfa4c56032` and Protocol
`e819f95890ea23ae2f336a59fb3ff62cfa858d8b`. The protected job recorded exactly one snapshot-V2
and one binding-V4 reconciliation, exactly one expected provider-free refusal, no selected-work
marker, and terminal child/scope/cgroup/active-slot cleanup. MUSE independently reviewed the final
evidence with no P1/P2/P3 findings. The runner was stopped immediately after the completed attempt;
its disabled unit has `Restart=no` and no Listener or Worker remains. Its intentional SIGTERM exits
`143` and leaves systemd in `failed`, so a future reviewed run must preserve this record and only
`reset-failed` as its own fresh-run preparation step.

This closes only the hosted-evidence custody gate: a root-custodied copy of bounded job-produced,
provider-free evidence and the corresponding provider-free refusal. The root service does not
independently validate the job assertions and the trusted root administrator remains in the trust
base. It does not prove a real GitHub OIDC request, provider contact, Google STS/WIF or Secret
Manager activity, materialization, injection, selected-work execution, positive provider evidence,
Step 8, or V12.2. This custody completion alone authorizes no additional provider-contact work.

The later Launcher V3 release-companion run `36194756777` stopped at public-root preflight:
the GitHub Linux host's `/usr/share` was root-owned but mode `0777`, so the protected
path check correctly refused it before authority or provider activity. Launcher
`aa55319fa88f14e96b47e3fa9d08a0940fae5456` and Core branch
`bobai/v12.1-public-evidence-root` move future public verifier, installation, capture,
and recovery evidence to the root-created
`/var/lib/ota/authority-launcher-public` sibling. The historical `/usr/share` record
above remains evidence for its completed attempt; it is not a fallback or migration
input. A fresh provision, regenerated identities, and new hosted proof are required
for this paired revision. No provider, OIDC, materialization, injection, or selected-work
authority is added. Core docs and tests are affected; Examples, Skills, Site, Learn,
FAQ, and Glossary are unchanged because this internal path correction introduces no
public contract, command, or operator vocabulary.

Launcher `aa55319fa88f14e96b47e3fa9d08a0940fae5456` passed ordinary CI, root-boundary proof,
and both GitHub-hosted V3 pressure jobs in run `36203430810`. Those jobs do not replace the
fresh protected Linux/X64 hosted custody proof for the paired Core and Launcher revisions.

The first 1.6.29 protected attempt, Core run
[`36268385879`](https://github.com/ota-run/ota/actions/runs/36268385879) at
`43844d718eb248e2b70e420ae0fb8e867b3afa96`, was cancelled while the protected job was
queued and the runner remained stopped. Its pressure fixture builder admitted only the older
`1.6.28-implementation` ref, so the root-owned request for `1.6.29-implementation` could not
produce a valid authority installation. The Toolkit probe passed, but this run proves no
protected service path or fresh custody. A narrow fixture/runbook repair admits canonical
`1.6.<patch>-implementation` refs from patch 28 onward and retains exact request/ref/SHA
matching. That cancelled attempt's partial authority state was not reused.

The fresh gate ran as Core
[`36271452585`](https://github.com/ota-run/ota/actions/runs/36271452585), attempt `1`, at
`f0668e9e6c3eae89176ba89695186823a22e2a9f`; the protected job `108486101249` and Toolkit
probe both passed. Before dispatch, the cancelled attempt's partial managed authority state was
moved into root-only `/root/ota-cancelled-36268385879`, with the earlier evidence backup,
runner registration, runner service/drop-in, and source checkouts preserved. The fresh root-owned
mode-`0400` request bound run/attempt, repository and actor IDs, workflow ref/SHA, and Runner.Listener
`2.337.0`. Provisioning installed exact Core `f0668e9e`, Launcher `aa55319f`, and Protocol
`e819f958` while the runner was stopped. The org runner group's stale Core 1.6.28 workflow ref
initially prevented job assignment; it was replaced by the exact 1.6.29 ref for this attempt and
removed after completion, leaving its three existing Launcher entries.

The one-off reset script is retained root-only in that archive (SHA-256
`7c364e8710f3e225c5bdf3951884a152add0e9ff7d742858ca6c567df7a74ea5`). Its final
`reset-failed` call reported the inactive runner unit as not loaded after `daemon-reload`; the
script was not rerun. Subsequent inspection confirmed the runner's exact unit and hardening
drop-in loaded, disabled, inactive/dead, and MainPID `0` before provisioning.

The root capture service published one root-owned mode-`0644`, single-link `success_set` record at
`/var/lib/ota/authority-launcher-public/hosted-evidence-captures/36271452585-1.json`, identity
`sha256:d4a06e4a9583b090ec18ef52e9b87eb2a79290ada5b71a6c2c1db8a09b925f89`, binding
request identity `sha256:84a30401cf2d8910052cc4a7d9077111f1d4c91d11fab9afef96e7a5199e4a5b`
and root-computed bundle digest `sha256:2cb4b4d2827a4ee1780b93ff4ffdc85867a8c695806de7fecd4e83462ef10a01`.
The five-file root-only evidence set passed its retained `SHA256SUMS`. The job verified exactly one
snapshot-V2 and binding-V4 reconciliation, the expected provider-free refusal, and terminal
child/scope/cgroup/active-slot cleanup. No selected-work marker exists. The runner was stopped
after the job and remains disabled with no Listener or Worker; its intentional SIGTERM leaves the
unit `failed`, to be addressed only in a separately reviewed fresh-run preparation. This is
bounded job-produced evidence under root custody, not independent semantic validation by root.
Provider contact remains `not_instrumented`; no real OIDC request, Google STS/WIF, Secret Manager,
materialization, injection, selected-work execution, positive provider evidence, Step 8, or V12.2
is proved or activated. MUSE independently rederived the public record identity and root bundle
digest from the retained bytes, matched the Toolkit artifact, checked refusal, cleanup, and runner
shutdown, and found no P1/P2/P3 issue with this bounded gate.

The independently reviewed GitHub Actions OIDC network-call activation amendment is committed with
this handoff in the V12.1 plan. It corrects the prospective dispatch route to the current signed
authority-snapshot V2/transaction-binding V4 pair; the historical V3 compatibility proof and
network-disabled V2 preparation cannot substitute. It authorizes only implementation of one bounded
GitHub OIDC request-service call, not Google provider contact. A later implementation review and
fresh exact protected Linux/X64 run are separate gates. Until then provider contact remains
`not_instrumented`, and the protected runner stays stopped.

The next Core checkpoint is committed with this handoff: a distinct, network-disabled
V4 provider-transport preparation consumes only the signed authority-snapshot V2 / transaction-binding
V4 path, retains the complete signed transport graph and record inside the opaque consumed capability
owned by the prepared transport, and rechecks them against the embedded build expectation, binding,
operation plan, and candidate. Historical V2 preparation remains distinct. Focused V4 positive,
record/projection substitution, replay, and V2-regression tests passed locally; MUSE found no
P1/P2/P3 blocker after repair. No HTTP Agent, request dispatch, CLI route, workflow, hosted proof,
GitHub OIDC request, Google contact, materialization, injection, or selected-work execution is
implemented by this checkpoint. This internal non-public change needs no Site, Skills, Examples,
Learn, FAQ, Glossary, schema, public JSON, or command-reference propagation.

The active follow-on is a feature-gated, crate-private GitHub OIDC one-shot dispatch owner.
Its implementation-only checkpoint is committed with this handoff on
`1.6.29-implementation`. MUSE's independent review found no remaining P1/P2 blocker after the
allocation-free JWT validation and bearer-substitution regression repairs. The initial synthetic
fixture accidentally reached the dispatch path before the test was corrected to invalidate its
V4 binding; the corrected fixture proves zero Core invocations on that pre-dispatch refusal only.
No successful provider contact is established, and no CLI or hosted workflow route is enabled.
Historical V1 preparation remains network-disabled. After the private relay and Core route are
implemented and independently reviewed, the later gates are a distinct manual workflow pinned
to its exact revision, fresh stopped-runner provisioning, and protected Linux/X64 proof.
Post-invocation terminal outcome and one real GitHub response remain unproved.
No Google STS/WIF, Secret Manager, materialization, injection, selected work, Step 8, or V12.2
is activated. This internal change needs no Site, Skills, Examples, Learn, FAQ, Glossary, schema,
public JSON, or command-reference propagation.

The private runner-capability relay design is now implemented across the three internal owners.
Protocol `e5fe1c83e562e02f60e27026c7148918bd016155` owns the closed
non-secret request/challenge/acknowledgement records and bounded zeroizing private binary frame.
Launcher `bafbf1717f102c9d9765c5af382ad06c4ea7eb66` spends one exact V4 reservation before challenging
the already authenticated job peer, forwards at most one correlated frame to the retained same-child
Core session, and requires Core's exact acknowledgement. Both revisions are committed and pushed.
Core `5eb78412c9ac142542e5255c5bfdc36d1fce30bf` is committed and pushed, pins the reviewed
Protocol revision, and implements the remaining ownership boundary: it accepts only the exact live
workflow/task route, consumes V4 before requesting the relay, validates the challenge and private
frame on the same session, constructs the fixed endpoint observation without process-environment
credentials, and dispatches at most one GitHub OIDC request.
Focused tests prove exact-route preparation and zero-dispatch refusal for the historical release
workflow, provider-free workflow, wrong task, stale nonce, truncated frame, and queued duplicate
bytes. They also prove timeout restoration and coalesced challenge/private-frame delivery without
prefetching credentials into the generic JSON buffer. Every Ota-owned URL copy is zeroized and
protected input/debug output does not expose the URL or bearer; the third-party HTTP request owns a
transient URI only for the bounded invocation. Linux/arm64 all-target/all-feature compilation
passes. Focused Core tests pass for all five private-frame relay cases, exact live-route V4
consumption and refusal substitution, endpoint redaction, and the closed dispatch-attempt state.
MUSE's first review identified two P2 blockers: Ota-owned full-URL copies that were not zeroized and
generic challenge framing that could prefetch private bytes. Both are repaired and regression-tested;
SCOOBY's stabilized-diff review found no remaining P1/P2. A requested final MUSE recheck did not
return a verdict because that thread was occupied by unrelated user work. The small VM could not
link the full all-feature test binary and killed `rustc` for memory. Canonical host `ota run ci`
also reached the unrelated pre-existing
`runner::tests::ensure_ready_activation_starts_and_cleans_up_shared_remote_internal_target` test,
whose background HTTP server was reaped while its readiness loop has no retry bound; the run was
stopped after the test continued probing indefinitely. No changed relay file owns that runner test.

The separately reviewed manual live workflow is committed and pushed at Core
`2a430b926aab231ae61490bea8ceac76a586ca1d`. It accepts only an explicit matching Core SHA, pins
Launcher `bafbf1717f102c9d9765c5af382ad06c4ea7eb66` and Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`, grants only `id-token: write`, runs no third-party
action, and emits only the closed non-secret Core dispatch count and outcome. At that checkpoint,
no hosted job had exercised this route. The first manual dispatch,
run `36350170156` at Core `4a8549104b3abf24cec9e88deead9d7bad5bad57`, was cancelled while
queued with no runner assigned. Pre-provision review found that Core's pressure builder admitted
only the historical provider-free workflow reference. Core correction
`758b877d14ab07008497d271dacc7beb00c5be65` is committed and pushed: it admits the exact live
workflow only on `refs/heads/1.6.29-implementation`, reuses the live route's canonical reference,
and tests V2 authority payload rendering plus branch/workflow substitution refusal. All five
pressure fixture tests pass; MUSE found no P1/P2 trust-boundary blocker. After all required
registration-branch release checks passed, only the inert workflow registration stub was
fast-forwarded to `main` at `a7fe38be7bbbdd31e499c35c5d0775814c781944`; GitHub registered
the workflow path. No implementation or version bump moved to `main`.

The old VPS became unreachable before its replacement Core build was installed; no fresh run used
that host. The exact live gate subsequently ran on a fresh Ubuntu 24.04 Linux/X64 Google Cloud VM
as Core [run `36472855700`](https://github.com/ota-run/ota/actions/runs/36472855700), attempt `1`,
job `109099182428`, at Core `8906804faf77b1f2873db9d1f680840bee16e4c5`, Launcher
`bafbf1717f102c9d9765c5af382ad06c4ea7eb66`, Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`, and Runner.Listener `2.337.0`. The new
`ota-authority-independent` runner `1377` remained offline while the exact job queued. A root-owned
mode-`0400` request bound run/attempt, repository and actor IDs, branch/ref/SHA, and runner version;
fresh provisioning installed the source-built binaries and published root-owned public installation
records before the runner started. The exact job passed on that runner. Its public posture recorded
`core_invocations=1`, `outcome=response_received`, and `selected_work_executed=false`, after the
client deliberately refused before claim admission. The workflow checked the exact installation,
request identity, source revisions, refusal text, terminal cleanup, absence of selected work, and
absence of OIDC input or JWT bytes in command output. The host also had no selected-work marker.
The runner was stopped immediately; its repository access and registration were removed. The VM,
auto-deleting boot disk, dedicated subnet, firewall, and VPC were deleted, with no VM or disk left
in the project. The retained public run logs and two public installation records are in
[`secret-delivery-github-oidc-live-36472855700.zip`](../pressure/retained-artifacts/secret-delivery-github-oidc-live-36472855700.zip)
(SHA-256 `0475120b487cfeb13e2f895aa78e97c393c081e6baba7a30665f270b7dedb82d`). MUSE's
independent read-only review found no P1/P2 blocker for this narrow hosted gate; its stale
source-comment P3 is corrected in this batch. MUSE's closure review found no remaining P1/P2/P3.

This proves one bounded Core GitHub OIDC request-service invocation received a structurally valid
unadmitted response under the exact protected route. The public live posture is job-derived, not
independent root-custodied semantic attestation; the separate earlier provider-free custody proof
does not substitute for it. Provider-side and lower-layer request cardinality remain `not_proved`.
No JWT issuer, signature, or claim admission, Google STS/WIF, Secret Manager, materialization,
injection, selected work, Step 8, or V12.2 is proved or authorized. Any broader provider-contact
or admission slice requires its own separately reviewed plan and proof gate. This internal
feature-gated route adds no public command, schema, JSON, or operator-facing concept, so no
Examples, Skills, Site, Learn, FAQ, Glossary, or command-reference propagation is required.

The local [GitHub OIDC JWT claim reconciliation checkpoint](../planning/v12.1/plan.md#local-github-oidc-jwt-claim-reconciliation-checkpoint-active-2026-09-28)
is committed and pushed as Core `f0ae68f89bbc84923096868cc9df2d9264931144`. It adds protected,
duplicate-safe local matching of the retained unadmitted JWT against the signed V4 candidate,
distinguishes `claims_refused` from transport refusal, and proves exact and substituted synthetic
responses through the snapshot-bound V4 fixture. MUSE's frozen-diff review found no P1/P2 and
identified distinct-child/session replay and child/cgroup cleanup as open proof prerequisites.
Default no-feature compilation and focused pressure tests passed.

An exact Step 7 protected Linux/X64 reconciliation workflow passed in Core
[run `36548056015`](https://github.com/ota-run/ota/actions/runs/36548056015), attempt `1`,
job `109339134798`, on a fresh Ubuntu 24.04 Google Cloud VM. It bound Core
`eeab51e4d473d09073094424750643abbfe2dc11`, Launcher
`bafbf1717f102c9d9765c5af382ad06c4ea7eb66`, Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`, and Runner.Listener `2.337.0`.
The runner remained stopped while the job queued; an administrator-owned mode-`0400` request
bound the exact run, attempt, ref, SHA, repository and actor IDs, and runner version before
provisioning. Operator-side GitHub checks found that the dedicated runner group initially had no
selected repositories, so it could not assign the job; temporary access was added only for
`ota-run/ota`, with the exact workflow restriction retained. After assignment, the job passed on
the provisioned runner. Its job-derived
public posture recorded `core_invocations=1`, `outcome=response_received`,
`jwt_claim_reconciliation=matched_unadmitted`, `jwt_admission=not_established`,
`jwt_signature_verification=not_attempted`, `google_contact=not_attempted`, and
`selected_work_executed=false`. The retained public logs and two public installation records are
in [`secret-delivery-github-oidc-live-36548056015.zip`](../pressure/retained-artifacts/secret-delivery-github-oidc-live-36548056015.zip)
(SHA-256 `5bab516260b47ca107bbea7f3c9a8018b5e51a5bde373dfd939f01ce96fba312`).
Operator-side checks after the run confirmed that the runner was stopped and deregistered,
temporary group repository access was removed, and the VM, auto-deleting boot disk, dedicated
firewall, subnet, and VPC were deleted; the project has no remaining VM or disk. These
administrative cleanup facts are not proved by the retained job archive. This is a bounded
job-observed local reconciliation result, not an independent
root-custodied semantic attestation. Provider-side and lower-layer request cardinality remain
`not_proved`; it does not establish GitHub signature validity, issuer authority, Google/provider
acceptance, Google STS/WIF, Secret Manager, materialization, injection, selected work, Step 8,
or V12.2.

The missing local prerequisites are now covered in the Core implementation on this branch and
Launcher `7d81d93d309fe360968dd48543671d13958adadc`. Core derives distinct consumed-V4
request/session identities and refuses the first private frame against the second at the protocol
correlator. A separate production inherited-session receiver test refuses a replayed frame against
a distinct synthetic request identity with the nonce held constant. GitHub JWT claims remain bound
to workflow/run context, not local child identity; this proves private-frame/capability replay
refusal, not that otherwise-valid raw JWT bytes are intrinsically session-specific. Launcher now
routes
`fail_selected_boundary` through one cleanup helper and exercises that exact helper against a real
root/systemd child, transient scope/cgroup, and active slot on Linux/arm64. The test confirms child
absence, scope/cgroup terminality through a non-mutating observer, and active-slot removal. It does
not send the terminal client frame or inject a real relay mismatch, and `/bin/true` may exit
immediately after resume, so it is connected production-path coverage rather than an end-to-end
replay-to-terminal demonstration. Focused Core tests passed 1/1 each, the Launcher root/systemd
test passed 1/1, and formatting and diff checks passed. MUSE independently reviewed the revised
frozen code diffs (Core
`217742a4bba24ac8851c6c46c18c31ca9e444325f227daab2c5829fd7cc9e526`, Launcher
`1e656797fb0ab3f4c137cc056141c62d7133296cb6fbe46f5b75dbd512a34707`) and found no
P1/P2; its remaining P3 is this retained proof limit.

Run `36548056015` predates these replay and cleanup prerequisites, so it could not close the
planned hosted reconciliation proof gate. These internal feature-gated tests and
cleanup refactor change no public command, schema, JSON, or operator-facing concept, so no Examples,
Skills, Site, Learn, FAQ, Glossary, command-reference, or public JSON propagation is required.

The paired revisions are published: Core `c39e0bc5376fc9c77a45b8f0d4c31a29748d8625` pins
Launcher `7d81d93d309fe360968dd48543671d13958adadc`, with Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155`. The first fresh hosted attempt,
[run `36576329741`](https://github.com/ota-run/ota/actions/runs/36576329741), job
`109432798394`, failed before authorization. Its host accounts had `/bin/bash` instead of the
configured `/usr/sbin/nologin`; the ephemeral runner also could not remove its credentials under
the read-only service profile after the failed job. No selected-work marker appeared. The runner
was stopped, its registration disappeared, and that VM was deleted. This failed attempt is not
hosted proof and its request and provisioned state were not reused.

The fresh protected Linux/X64 gate then passed in Core
[run `36580942380`](https://github.com/ota-run/ota/actions/runs/36580942380), attempt `1`, job
`109448647893`, on a new Ubuntu 24.04 Google Cloud VM at the exact Core, Launcher, and Protocol
revisions above and Runner.Listener `2.337.0`. Operator-side checks recorded that the
non-ephemeral runner stayed offline while the job queued and that a new root-owned mode-`0400`
request bound the run and attempt, repository and actor IDs, workflow/ref/SHA, and runner version
before fresh provisioning and runner start. The retained public installation records reconcile
the exact source revisions and request identity, but do not independently prove the request file's
mode or the pre-start ordering. Both workflow steps passed.
The job-derived public posture reports `core_invocations=1`, `outcome=response_received`,
`jwt_claim_reconciliation=matched_unadmitted`, `jwt_admission=not_established`,
`jwt_signature_verification=not_attempted`, `google_contact=not_attempted`, and
`selected_work_executed=false`. The host had no selected-work marker. Public run logs and the two
public installation records are retained in
[`secret-delivery-github-oidc-live-36580942380.zip`](../pressure/retained-artifacts/secret-delivery-github-oidc-live-36580942380.zip)
(SHA-256 `0d1240d44872212ac21c7367c6d76684b55c1ce215c419c8329812cf9a729110`).
Operator-side checks confirmed the runner was stopped and deregistered and both short-lived VMs,
their auto-deleting disks, the dedicated firewall, subnet, and VPC were removed; these cleanup
facts are not proved by the retained job archive.

This closes the planned exact-revision hosted reconciliation gate together with the separately
reviewed local replay-refusal and selected-failure cleanup tests. It does not turn those tests
into an end-to-end hosted replay-to-terminal demonstration or the job-derived posture into an
independent root-custodied semantic attestation. Provider-side and lower-layer cardinality remain
`not_proved`; JWT signature/issuer authority, admission, Google STS/WIF, Secret Manager,
materialization, injection, selected work, Step 8, and V12.2 remain unproved or inactive.
V12.1 Step 7 is still active. The next provider-contact or admission slice requires its own
explicit scope, independent review, and proof gate; none is activated by this run.

The operator subsequently activated only the first implementation batch of the independently
reviewed [Google STS exchange checkpoint](../planning/v12.1/plan.md#google-sts-exchange-checkpoint-first-batch-active-2026-09-29)
on 2026-09-29. It retains the exact consumed V4 transport and reconciled JWT in one private,
non-cloneable owner and exercises the production STS ownership path through a fake Google
transport. Google network enablement, hosted STS proof, IAM Credentials, Secret Manager,
materialization, injection, selected work, Step 8, and V12.2 remain inactive. This internal
checkpoint changes no public command, schema, JSON, or authoring/operator concept, so Site,
Skills, Examples, Learn, FAQ, Glossary, and command-reference propagation are not required.

The working first-batch implementation now couples the consumed V4 transport and locally
reconciled JWT in one non-cloneable private owner. Its fake STS path rechecks signed full
graph/record expectations, the rederived operation, binding expiry, and JWT claims/time before
one dispatch; it refuses substitution, redirects, ambiguous headers, oversized/malformed
responses, and invalid token types without exposing transport details or admitting a token.
The six STS form fields are independently decoded and asserted against protocol literals.

Local validation passed Core provider tests (12/12), snapshot tests (22/22), no-feature and
`secret-delivery-pressure` compilation, formatting/diff checks, and first-party sync for this
batch's explicit changed-file scope. The staged pressure-document changes were excluded from
that implementation batch; their subsequent reconciliation is recorded below.
The paired Launcher root
fixture `root_systemd_core_sts_refusals_reap_exact_child_cgroup_and_active_slot` passed 1/1
on Ubuntu 24.04 Linux/arm64 with Rust 1.98.1, exercising all 11 refusal cases. It launches the
Core test executable as the actual stopped/scoped child, binds its synthetic signed V2/consumed
V4 authority to that child's startup, and passes the real STS refusal result through
`SystemdExecutionCompletion::persist_terminal_completion` on the retained child session.
Launcher's production relay reconciles and durably records the completion and actual exit;
`execute_selected_boundary` takes its real missing-archive failure path. Each case confirms
child absence after reaping, exact scope removal/cgroup emptiness, active-slot removal, and
absence of the declared `selected-work-executed` marker. The existing root cleanup and V2
reservation-refusal regressions also passed 1/1 each.

Run this connected fixture only with the paired Core library test executable (compiled with
`secret-delivery-pressure`) and Launcher binary test executable (with `protected-attestor`):
`unshare --net env OTA_CORE_STS_TEST_BINARY=<core-test-executable> <launcher-test-executable> --exact systemd_service::tests::root_systemd_core_sts_refusals_reap_exact_child_cgroup_and_active_slot --ignored --nocapture --test-threads=1`.
The test requires a network namespace distinct from PID 1; this run used a fresh namespace
with only a down loopback interface. Build artifacts were preserved on the local VM's disk
after RAM-backed `/tmp` caused compilation-only memory failures. No cloud VM was created.

MUSE's final frozen five-file source review found no remaining P1/P2/P3 issue and confirmed
that the earlier connected-cleanup and independent-form-assertion findings are resolved.
The Google-network-disabled ownership/cleanup batch is locally complete. These are
synthetic local ownership/cleanup fixtures, not installed authority admission, hosted Linux/X64
proof, real Google transport, WIF acceptance, provider-side cardinality, or token-memory
erasure proof. Launcher fixture commit `dd667c3d6b63d8d26d9e2a40f831e9d08a7a6864` is test-only;
no runtime compatibility pin or Protocol change is required. Google network enablement remains
inactive and requires a separately reviewed
and explicitly authorized checkpoint. V12.1 Step 7 remains active.

The user-requested pressure-document review retained the bounded research signals without
activating implementation or declaring a reproduced Ota defect. It reconciled dbmask's merged
PR #37 with tested head `9f75a339e09fa3d762094ffee2ef612f93065e41`, exact-head fork/upstream
matrices `36409528942`/`36409530097`, and the published engineering note. The merge commit is
recorded separately from the tested head; maintainer acceptance does not imply ongoing use,
endorsement, or repository-wide governance. Site projection and reading guidance are synced at
`7d37cc9fe628243174e81f9855194ca121d79296`. Projection parity, generated pressure-page data,
content generation, whole-worktree first-party sync, and diff checks passed. This documentation
reconciliation changes no product command, schema, public JSON shape, or authoring concept;
Skills, Examples, Learn, FAQ, Glossary, and command-reference propagation are not required.
V12.1 Step 7 remains active; Google network enablement remains inactive.

The operator requested preparation of the next STS checkpoint on 2026-09-30. The
[network-enablement preparation checkpoint](../planning/v12.1/plan.md#sts-network-enablement-preparation-implementation-active-2026-09-30)
defines a separate signed-context-admitted manual STS proof route, exact bounded transport and
terminal token disposal, local negative controls, and administrator/hosted prerequisites. MUSE
identified a missing live-target source: the existing V1 builder signs only synthetic WIF
coordinates. The proposal now specifies an STS-only administrator pressure request V2 with one
canonical provider resource and a distinct complete-request identity; derived target coordinates
enter the signed authority, while V1 and the legacy workflow stay unchanged. MUSE's independent
re-review cleared that P2 with no remaining P1/P2/P3 planning findings. The proposal is reviewed
and explicitly activated for implementation only by the operator on 2026-09-30. Source review,
local production transport tests, and separately authorized hosted proof remain open. No Google
contact, provider configuration, installation change, or live dispatch is authorized in this batch.
The existing GitHub-only proof route must remain unchanged. All later provider operations and
selected work remain inactive.

The activated 2026-09-30 source batch now implements the closed administrator request V2,
signed-context-only STS workflow route, fixed HTTPS transport, bounded production response reader,
post-response authority/JWT checks, and terminal token disposal. The legacy GitHub-only workflow
is byte-for-byte unchanged, V1 request identity/output remains compatible, and failed GitHub
claim reconciliation or a legacy owner makes zero STS calls. The new workflow input is only a
public mirror used to reconcile the installed request identity; it is not passed to Core as a
provider target. Every command outcome remains failure with no positive receipt or selected work.

Local validation passed provider tests (14/14), snapshot tests (24/24), request tests (12/12),
new workflow tests (5/5), the existing provider-free workflow regression (1/1), default/no-feature
and feature-enabled no-session refusal (1/1 each), no-feature compilation, Actionlint,
formatting/diff checks, and first-party sync. MUSE's frozen source review found no other P1/P2/P3
blocker, but identified one P2 coverage gap: the response-time jump expired the binding before
the JWT recheck. The refinement retains that binding-expiry control and separately tests
`ResponseJwtExpired`: JWT valid at dispatch, expired at response time, with the production
binding/authority preflight explicitly still valid at that time. It requires one dispatch,
`response_refused`, and parsed-token disposal. The final snapshot suite passed 24/24.

The paired actual-child systemd cleanup test passed 1/1 with all 14 cases, including accepted
synthetic response disposal and both independent expiry controls, on the existing local
Ubuntu 24.04 Linux/arm64 VM with Rust 1.98.1. It ran inside an isolated network namespace and
confirmed child reaping, scope/cgroup/active-slot cleanup, and no selected-work marker.
The guest lacks Git, so the Launcher test compilation supplied its known HEAD and dirty marker
as build environment values; these test binaries are not clean installed-source/authority proof.
Launcher changes are test-only; its runtime compatibility and Protocol schema remain unchanged.

MUSE's focused frozen recheck cleared the P2 refinement with no remaining P1/P2/P3 finding.
The final 24/24 snapshot suite and rebuilt 14-case root cleanup run passed against those exact
refined sources. This implementation-only batch is locally complete. The operator authorized
commit and push on 2026-09-30; the Core checkpoint commit owns these exact sources and the paired
Launcher test-only checkpoint is `6fd8cde8782b772fb22eead5f8c691f036b425b5`.
The live workflow retains the reviewed Launcher runtime pin
`dd667c3d6b63d8d26d9e2a40f831e9d08a7a6864`; the paired change adds only cleanup test cases.
Ordinary CI is the next source gate, followed by read-only preparation of the separately
authorized hosted STS gate. No Google
call, cloud/provider configuration, installation change, or hosted dispatch occurred or is
authorized. Hosted Linux/X64 STS acceptance and exact remote WIF configuration reconciliation
remain open. Provider/lower-layer cardinality, independent root-custodied semantic attestation,
and token-memory erasure remain `not_proved`; IAM Credentials, Secret Manager, materialization,
injection, selected work, Step 8, and V12.2 remain inactive. V12.1 Step 7 is still active.
This internal pressure checkpoint changes no public command, schema, public JSON, or authoring
concept; Site, Skills, Examples, Learn, FAQ, Glossary, command-reference, and public changelog
propagation are not required. The planning and handoff records own this unshipped internal scope.

Publication and read-only hosted preparation (2026-09-30): Core
`0478311680276d87f5eb4930611d1f913828fff2` and Launcher
`6fd8cde8782b772fb22eead5f8c691f036b425b5` were pushed to
`1.6.29-implementation`; both worktrees were clean. Core push CI started. Its
[`docs-quality` run `36688113237`](https://github.com/ota-run/ota/actions/runs/36688113237)
failed only because Reddit returned HTTP 403 for one practitioner-report URL. An exact-URL
automated-client exclusion follows the existing `.lycheeignore` policy; unrelated links remain
checked. Other Core checks were still queued or running, so the checkpoint is not CI-green.
Launcher CI accepts only `main` pushes or pull requests; this implementation-branch push started
no Launcher hosted CI. The completed local cleanup proof remains distinct from hosted CI.

Read-only Google Cloud inspection found project `ota-v121-step7-20260910` has no VM and an active
pool `projects/783599651848/locations/global/workloadIdentityPools/ota-v121-pool`.
Its existing provider `github-ota-step7` admits the historical
`secret-delivery-provider-pressure.yml@refs/heads/1.6.28-implementation`, has no explicit
allowed-audience list, and does not bind exact workflow/source SHA or run/attempt. It is not
the new STS gate's reviewed configuration and must not be treated as ready. GitHub has no
registered protected Linux/X64 runner; the only listed runner is offline Linux/arm64.
The smallest next live preparation is a separately authorized dedicated WIF provider with the
plan's exact audience, mapping, and invocation conditions, plus a short-lived Linux/X64 host.
Preserve the historical provider. Freeze the final Core source and reviewed Launcher runtime
pin before queueing, then bind the queued attempt in both the provider condition and fresh
root-owned V2 request before provisioning/start. No provider/configuration mutation, VM creation,
runner start, hosted dispatch, merge, release, or Google STS call occurred in this preparation.

Hosted STS authorization and admission blocker (2026-09-30): the operator explicitly authorized
the dedicated WIF target, short-lived Linux/X64 VM, exact bounded STS run, evidence retention,
and deletion of the newly created infrastructure. This supersedes the earlier implementation-only
authorization limit for that narrow gate; it does not authorize IAM Credentials, Secret Manager,
selected work, release, or merging before the required source gate passes. Prefer a fresh
temporary WIF pool as well as provider, with no IAM grants, to avoid inheriting historical
pool-wide bindings. Preserve all existing pool/provider resources.

Before any mutation, read-only Actions checks found
`secret-delivery-google-sts-live.yml` absent from `main` and the registered workflow inventory;
both the default-branch contents lookup and workflow lookup returned HTTP 404. GitHub's
[manual-workflow documentation](https://docs.github.com/en/actions/how-tos/manage-workflow-runs/manually-run-a-workflow)
requires default-branch presence for `workflow_dispatch`. The source workflow exists at the
frozen implementation branch, but that alone is not dispatch readiness. Do not substitute the
historical GitHub-only route or change the repository default branch. The smallest next action
is to clear source CI and independently review/authorize narrow default-branch registration of
this manual workflow; do not merge the full implementation batch merely to register it.
Core `f9ddc0f67b52031cdf16d017b405c3783de56ced` source gates were still pending/running at
inspection; docs-quality passed at that exact head. No new pool/provider, VM, firewall, network,
runner registration, hosted run, or Google STS call was created while admission remained blocked.
MUSE independently confirmed this registration gap as a P2 and found no additional production
code repair; the plan now explicitly gates infrastructure creation on authorized default-branch
registration and completed applicable source/merge gates. Its read-only source review did not
execute tests or independently observe Cloud configuration. Exact-head Ota Readiness also passed
in run `36688481991`; Release Gate `36688482134` remained pending. Before eventual runner start,
separately verify runner-group/repository/workflow admission and one-shot job posture; four
matching runner labels do not exclude unrelated eligible queued work. Teardown must run on
success, refusal, cancellation, or timeout and include stopping/deregistering the runner,
disabling the new provider, retaining non-secret evidence, deleting only newly created resources,
and observing their deletion. No new production code or public propagation is required.

Registration preparation (2026-09-30): Release Gate
[`36688482134`](https://github.com/ota-run/ota/actions/runs/36688482134) passed at
`f9ddc0f67b52031cdf16d017b405c3783de56ced`. The operator then authorized the narrow
default-branch registration change. Reused the clean isolated registration worktree, based on
`main` at `a7fe38be7bbbdd31e499c35c5d0775814c781944`, and committed/pushed only the new
45-line STS registration stub as `8422f2ce915c2c0b853c0074ef1cdf8c69ab2d9f` on
`bobai/register-google-sts-live-workflow`. Like the existing GitHub-only registration stub,
it is manual-only, has empty global/job permissions, uses an ordinary hosted runner, and always
exits 1; it has no OIDC grant or protected-runner/provider command. Both required inputs match
the reviewed implementation workflow. Actionlint, structural checks, and diff check passed.
The real STS workflow and production implementation at `f9ddc0f6` remain unchanged. The separate
local agent-guidance commit and documentation changes were preserved, not promoted or discarded.

The registration branch's own Release Gate
[`36708801430`](https://github.com/ota-run/ota/actions/runs/36708801430) is running; docs-quality
and cargo-deny passed. MUSE's original pre-commit snapshot moved before review, so no independent
readiness verdict is claimed for it. MUSE completed the explicit frozen committed review of
`a7fe38be..8422f2ce` with no P1/P2/P3 finding, independently confirmed the file hash and input
parity against `f9ddc0f6`, and passed Actionlint and exact-range diff check. Its readiness verdict
is conditional on terminal green checks at exactly `8422f2ce`; it does not cover live authority
or dispatch. Main promotion remains held until applicable exact-source checks pass.
No PR, main update, cloud resource,
runner registration, or live dispatch had been created at that preparation checkpoint.

Registration publication and cancelled STS readiness (2026-09-30): all six applicable source
checks, including Release Gate `36708801430`, passed at exactly `8422f2ce`. After confirming
`main` still matched `a7fe38be`, the independently reviewed registration stub was non-force
fast-forwarded to `8422f2ce915c2c0b853c0074ef1cdf8c69ab2d9f`. Actions workflow `371186494`
is registered. This is registration only, not the full implementation merge or a release.

The authorized temporary Linux/X64 host built Core `f9ddc0f6`, Launcher runtime `dd667c3d`
and Protocol `e5fe1c83`, with pinned Listener `2.337.0`. Dedicated group `4` admitted only
Ota and the full-SHA STS workflow; runner `1379` registered ephemeral/disable-update and stayed
offline. Protected local configuration confirmed both flags; public runner GET does not
independently expose them. Canonical service argv, `Restart=no`, strict sandbox and the three
writable paths remained unchanged. Root-owned read-only runner configuration must not be made
writable to hide an anticipated ephemeral post-job deletion failure. No restart/reuse is allowed.
Canonical provisioning established fresh authority state, inactive units, no socket owners and
no principal processes. It correctly refused archive-preserved non-root runner ownership and a
premature empty capture directory before installation; preparation corrected only the new host.

Run [36725521542](https://github.com/ota-run/ota/actions/runs/36725521542), attempt `1`, job
`109921288686`, queued at exact `f9ddc0f6`. Request/public-installation identities independently
rederived, and the administrator provider readback matched the exact issuer, mapping, distinct
audience and invocation condition. MUSE nevertheless held activation with one P2: actual installed
signed V2 snapshot and Core-derived STS operation target were not canonically compared with the
complete request/provider readback. Public identities and builder linkage do not close that bar.
A P3 whole-journal exporter was changed before use to root-private scratch plus closed boolean
facts/systemd metadata; raw journal, credential/environment contents and diagnostic lines were
not exported. Its source-only correction is not a hosted/provider proof.

The run is terminally cancelled before runner assignment/start; no Core OIDC/STS call occurred.
Before deletion, runner status was inactive/dead, PID zero, empty cgroup, no restarts or principal
processes, and no authority scopes, active-slot files, capture record or selected-work marker.
Runner/group, VM, firewall/subnet/network were deleted; matching after-state
lists are empty. Fresh provider deletion succeeded and lookup returned `NOT_FOUND` after
parent-pool deletion. Pool `ota-sts-20260930` is disabled/`DELETED` under Google's soft-delete
lifecycle, not permanently erased. Pool/provider were never enabled; historical resources were
not teardown targets. Pool IAM `{}` is only that policy observation, not project-wide IAM proof.

Retained non-secret readiness/teardown archive:
`docs/pressure/retained-artifacts/secret-delivery-google-sts-readiness-36725521542.zip`, SHA-256
`0215fc51d7c277fbcf7148f987c0e04171e79aa2f2a6663bd2288c7f66c5f686`. Archive integrity and
JWT/private-key marker checks pass. Its operator `outcome.json` explicitly records cancellation,
administrator observation versus semantic attestation, and all unexercised provider boundaries.
The independently observed follow-up exact-name disk list is empty; the archive now retains
that observation and qualifies the unobserved original autoDelete setting/deletion mechanism.

Offline source repair is now implemented locally and independently source-reviewed. The feature-gated
helper adds `--verify-installed`, requiring `--provider-readback` and
`--expected-builder-artifact-identity` alongside its unchanged existing administrator expectations.
It reads the fixed mode-0400 root request, signed stores, and provider readback; root-owned mode-0644
public installation; and recipient-owned mode-0640 repository contract through bounded no-follow
descriptors. Root ownership, singular files, exact modes and protected ancestors remain mandatory
for authority inputs. The subject directory/file have exact 0750/0640 modes and common observed
ownership only: this is not independently verified recipient identity or an immutable subject.
One retained contract read feeds derivation without executing its tasks/hooks. Production Protocol
envelope/JCS reconciliation and Core V2 signature/schema/full graph/record verification precede
exact comparison with the complete expected payload. The public installation additionally binds
the complete request identity, builder/source and selected environment, including runner version.
The pure STS target derivation is shared with runtime planning; no semantic/snapshot/V4/JWT owner
is fabricated. Exact provider resource, issuer, audience, mapping and invocation condition must
match, with the administrator readback still disabled/ACTIVE. Output is closed, identity-only,
offline/not-admitted/not-dispatched, with point-in-time subject-owner consistency, un-attested
administrator provider observation, and runtime reconciliation still required. Default render
output remains unchanged. Local checks pass: 17 pressure-fixture tests, 1 CLI mode test, 14 provider-
client tests, 24 authority-snapshot tests, default-feature compilation, native Ota formatting and
first-party sync. Three production-path regressions also pass in a fresh network-disabled root
Linux/arm64 container with the workspace mounted read-only, including the complete installed
entrypoint's success/refusal and unchanged input bytes/metadata. This is not hosted Linux/X64 proof.
The preceding agent-guidance commit's existing local Skills reconciliation is recorded at
`4fe996eb`; its publication remains separate. MUSE's frozen source review found no P1/P2 and one
test-coverage P3; canonical, coherently reidentified environment and invocation mismatch tests
resolved it in the narrow recheck. Source commit readiness is clear. MUSE's separate fixed-commit
documentation review at `24543336` found no P1/P2 and one P3: the provider-free acceptance criteria
were structurally nested under the STS inspection heading. A sibling `Provider-Free Hosted
Acceptance` heading now separates the two scopes without changing either acceptance bar or source.
The user has explicitly approved commit and push of this reviewed
repair on `1.6.29-implementation`; no merge or release is authorized. Exact published-source gates
must pass before a fresh host/group/provider/request/attempt; do not reuse cancelled state.
Installed-host offline verification and a cleared pre-start packet remain mandatory before
activation. The following fresh-host checkpoint supersedes this source-only preparation state.

Fresh Linux/X64 offline checkpoint (2026-09-30): Core `0476b9a3b484c672fccb164517a1dc3097cd5e66`,
Launcher `dd667c3d6b63d8d26d9e2a40f831e9d08a7a6864`, and Protocol
`e5fe1c83e562e02f60e27026c7148918bd016155` were installed on a fresh Ubuntu 24.04 X64 VM.
The production root-only `--verify-installed` helper passed against actual installed signed
stores, complete graph/record, request/public installation, and disabled provider readback.
Verifier expectations came from the exact successful producer invocation and independently
checked source/build/artifact identities, not from the stores being verified. Its closed report
remained offline/not-admitted/not-dispatched with runtime reconciliation still required.
MUSE's packet review required fail-closed teardown argument guards and explicit service-account/
firewall observations; the separately retained addendum resolved both holds. No service account
or scopes were attached, and SSH was restricted to the operator's observed /32 on the fresh network.
Final installed observations matched the reviewed packet, but the remaining signed lifetime was
too short for safe activation. Run `36764606243`, attempt `1`, job `110055512134`, completed
cancelled with zero steps. No provider was enabled and no runner was started. Before deletion,
root observation confirmed runner MainPID `0`, no job/exec principal processes, no active Ota
scope/slot, no public capture record, and no selected-work marker. The reviewed guarded teardown
completed successfully; subsequent exact-name lists show no VM, boot disk, network, subnet, or
firewall, and GitHub inventories show neither runner `1380` nor group `5`. The pool is disabled
and `DELETED`; provider deletion was acknowledged before pool deletion, and its later description
returned `NOT_FOUND` under the deleted pool. This is not independently retained provider semantic
attestation. Closed public facts are retained in
`docs/pressure/retained-artifacts/secret-delivery-google-sts-readiness-36764606243.json`;
protected stores, producer identities, private traces and operator records are not published.
The subsequent fresh run `36775307269`, attempt `1`, job `110091638636`, used the same frozen
source pins on a fresh VM with fresh group `6`, runner `1381`, provider, request and generation-1
authority. The first observer call failed before strace/provisioner/builder because the fixed
request was not staged yet; that failure was preserved, the exact request staged at root `0400`,
and only one canonical authority issue followed. MUSE identified a P2 in the operator wrappers:
`pgrep` errors were treated like no matching process. Separately retained corrected read-only
wrappers accept only status `1`; local stub checks refuse `0`, `2` and `3`, and corrected host
observations/offline verification passed with installed bytes unchanged. MUSE cleared the frozen
packet plus correction addendum. No signed input, binary or unit changed and no authority was
renewed. Immediate pre-enable and post-propagation checks passed; remaining signed lifetime was
`1867` seconds before runner start, exceeding the new `1200`-second floor.
The fresh pool/provider were temporarily enabled, and the runner became online/idle with the
exact labels. The job remained queued/unassigned with zero steps. Current GitHub documentation
requires non-reusable workflows to be branch-pinned in runner groups; reusable workflows may be
SHA-pinned. The full-SHA group restriction is a likely incompatibility, not a conclusively
observed scheduler diagnostic. No in-place admission relaxation or rerun was attempted. The run
was cancelled, the provider/pool disabled and runner stopped. Corrected terminal observation
confirmed no principal processes, Ota scope/slot or selected marker; the unit retained `failed`
and exit status `143` after operator stop, not a successful job completion. No capture record
existed. Reviewed teardown completed; VM/disk/network/subnet/firewall/group/runner absence was
observed, the pool is disabled/`DELETED`, and provider deletion was acknowledged then its
description returned `NOT_FOUND` under the deleted pool. Closed facts are retained in
`docs/pressure/retained-artifacts/secret-delivery-google-sts-readiness-36775307269.json`.
MUSE's read-only design review supports the exact fully qualified branch selector under explicit
administrator-controlled branch/dispatch freeze and fresh exact-head/non-competing-invocation
observations before issuance and start. This fixes the documented compatibility mismatch, not
immutable workflow-code enforcement: branch code can change the guards, and signed Ota authority
does not constrain arbitrary pre-guard shell execution. A SHA-pinned reusable job would require
separate caller/callee identity design and is not a transparent substitution. No live activation
is cleared by this design review. Review reusable helpers and stage builds before issuing entirely fresh authority,
then independently review the new installed packet and freshly require at least `1200` seconds
for job timeout, propagation and cleanup. Cancelled/near-expiry authority is not renewed or reused.

Uncovered material behavior: exact-head source gates, installed Linux/X64 offline comparison and
administrator-observed temporary-resource cleanup are proved within their stated scope, not live
provider execution. Provider/delivery/cardinality claims remain explicitly `not_proved` in the
retained machine-readable outcome. GitHub scheduling and Google enforcement are external and
unexercised; neither cancelled job exercised provider execution or executed-job cleanup. The
process-observation defect is fixed in internal operator wrappers, not Core's runtime. GitHub
runner-group scheduling is repo-owned external CI behavior outside the declared Core authority
scope; the named pressure-admission correction is owned by the hosted operator packet/workflow,
not a demonstrated Core engine defect. Provider/delivery behavior remains explicitly bounded
and not proved in machine-readable retention. An inactive Launcher
diagnostic follow-on is the managed-ancestor refusal's missing offending path; owner
`pressure_provision::verify_existing_ancestor_chain_no_follow_from`, return during prepared-
provisioning UX triage without weakening checks. No public command/schema/JSON or authoring
concept changes, so Site, Skills, Examples, Learn, FAQ, Glossary and command-reference need no
propagation. Step 7 remains active; IAM Credentials, Secret Manager, materialization, injection,
selected work, Step 8, V12.2 and release remain outside this gate.

## Agent Guidance Command Parity

Generated `AGENTS.md` task commands now include `--agent` for default-workflow phases,
entrypoints, default tasks, safe tasks, and verification tasks. Core's `agent.notes` and
handwritten guidance use agent admission for execution and identify flagless inspection commands
by their actual CLI syntax. Release-version changes remain outside the agent-safe surface.
Authoring postures also forbid directly rerunning refused work; defect isolation applies only to
explicitly authorized unmodelled work. Older unmarked generated-only files refuse without mutation
and remain `update_needed` pending review; exact current generated-only files still migrate.
Core command docs, CHANGELOG, the Site command card, canonical Skill, and both installed Skill
mirrors carry this correction. Examples, schemas, Learn, FAQ, and Glossary need no new shape or
term: this fixes existing command guidance without changing admission semantics.

## Working Rules

- Update this handoff immediately when the active step, blocker, next action, or proof status
  changes.
- Compact completed narration only after the coherent batch is committed, independently reviewed,
  and all affected repositories are reconciled. Remove detail only when a commit, plan, changelog,
  specification, pressure record, or retained evidence artifact durably owns it.
- Read the canonical Ota skill before Ota-specific work.
- Use `references/pressure-testing-protocol.md` for every pressure pass.
- Make and record the connected-surface decision for core docs, examples, skills, and site.
- Use released Ota versions for released proof; use the active branch only for explicit unreleased
  pressure testing.
