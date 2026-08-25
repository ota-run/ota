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

# V11.22: Contract Creation and Quality UX

Status: active. This OSS authoring slice follows completed V11.21 and V11.7. Activation authorizes
only the reviewed source-bound candidate and fail-closed execution-closure foundation; public
candidate commands, contract writes, and external pressure claims remain gated by their own
implementation and proof bars.

## Sequencing

V11.7 is complete for its bounded OSS audited-crossing slice. Core implements signed semantic grant
scope, crossing-time liveness and revocation, authorizer binding, one-use broker authority, selected
execution, terminal cleanup, and receipt/archive re-verification. The production invocation client,
least-privilege protected-history source, independently administered launcher installation, and
administrator-driven reboot/fault recovery are pressure-proven. Provider attestation is optional
stronger hardening, while contract-authored crossing declarations are explicitly deferred
follow-on authoring work.

V11.22 neither authorizes execution nor consumes a crossing record as candidate authority, so its
candidate model does not depend on a crossing record. V11.7 completion satisfies this sequencing
prerequisite, and V11.22 is now active. It must not imply that a crossing record is approval or
grant authority.

## Problem

Ota can validate a contract, execute its declared closure, report V11.14 claim assurance, and
refuse unsupported runtime enforcement. It is still too expensive for a new repository to create
an honest contract from observable repository truth and to understand which material behavior is
governed, contradicted, or simply unknown.

Valid YAML is not verified operational truth. Conversely, a detector must not turn incomplete
inspection into a confident contract. The first OSS answer is conservative, reviewable contract
authoring and quality evidence, not an account, LLM, hosted service, or generic code-review score.

## Product Boundary

V11.22 answers:

- what contract candidate can Ota derive deterministically from inspected repository evidence?
- which candidate fields are supported by named immutable evidence, conflicted, or unknown?
- what existing contract field would an explicit upgrade change, and why?

It does not answer:

- whether opaque shell behavior is correct or safe;
- whether a maintainer's business intent is complete;
- whether a candidate is approved for merge or deployment;
- how an organization centrally authorizes policy, approvals, or fleet changes.

V11.14 remains the canonical assurance domain for its shipped claim families. V11.22 may display
that structured machine truth for those families, but it must not create a parallel safety or proof
taxonomy or extend assurance to arbitrary detected fields.

## Canonical Candidate Model

`ota detect` and `ota init` should produce one serializable, content-addressed
`ContractCandidate` before writing a contract. Each proposed change carries:

```text
candidate identity
  schema version
  selected evidence-manifest identity
  discovery-inventory identity
  existing contract snapshot identity
  detector version and deterministic input set
  candidate kind: detection | upgrade

candidate change
  subject: structured contract path and semantic field family
  operation: add (detection candidates) | add | replace | remove (upgrade candidates)
  proposed value: canonical semantic value
  evidence: source kind, normalized repository path, source identity, and extraction location
  execution closure (for runnable or agent-safety proposals): canonical closure identity, ordered
    nodes and edges, working directory, platform, requirements, effects, classifications,
    unresolved reasons, and attributable source evidence for every recorded element
  confidence: high | medium | low
  disposition: applicable | conflict | unknown | unsupported

application projection
  projection identity: canonical content address of the exact contract mutation plan
  base contract identity: absent | canonical existing-contract identity
  operations: ordered canonical contract-path additions, replacements, or removals after
    detector-to-contract normalization
  resulting contract identity: canonical identity of the fully validated post-apply contract
```

The candidate is not `ota.yaml`. It is a portable, versioned JSON review artifact. Its identity
binds the logical root, discovery inventory, selected evidence manifest, detector version, proposed
semantic changes, complete closure records where applicable, and existing contract snapshot so a
later write cannot silently apply a result discovered against different source truth. A closure
digest is an address, not a substitute for its reviewable nodes, edges, classifications, unresolved
reasons, and source evidence. Git provenance is carried beside that identity, not folded into it.

Review evidence alone is not a writable patch. Before a candidate can expose `--write`, its
registered producer must also serialize an `application_projection`: the exact canonical contract
operations after all detector-owned representations have been normalized into published contract
fields, plus the expected post-apply contract identity. The projection is identity-bound to the
candidate, shown for review, re-derived with the candidate, and validated as a complete contract
before publication. A writer never invents this mapping from detector facts at apply time. This
prevents detector-internal representations from becoming a second, implicit contract authoring
language.

Evidence must be deterministic and path-specific. Initial supported source families are manifests,
lockfiles, toolchain files, Compose files, checked-in CI workflows, checked-in root or nested agent
guidance consulted by the detector, and existing Ota contracts. Every consulted guidance path is
included in both the discovery inventory and selected evidence manifest when it corroborates or
contradicts a candidate; changing it must stale the candidate it influenced. Opaque shell,
uninspected helper scripts, live services, ambient environment variables, and provider-owned
configuration remain `unknown` unless a shipped detector can identify a narrower fact without
inference.

### Detection Safety and Source Limits

Detection may propose a runnable task only from an exact, finite, repository-owned execution
closure that it serializes in the candidate change and classifies node-by-node. The closure
identity covers its ordered nodes and edges, working directory, platform, requirements, effects,
and unresolved reasons. V11.22 does not emit `agent.safe_tasks` or
`tasks.<name>.safe_for_agent: true` proposals: resolving command shape is not proof of effect
safety, and absence of a detected effect is never affirmative safety evidence. Existing
maintainer-authored agent-safe declarations remain authoritative contract truth. Positive inferred
safety is deferred to V12's typed effect and realization evaluator, where every material node,
requirement, and effect must receive an affirmative result with zero unknowns or conflicts before
any future registered detector rule may propose it. The candidate remains review input until a
maintainer explicitly applies it. A verifier-like task name is not evidence of safe execution. The
following remain `unknown` in V11.22:

- package-manager scripts, task-runner recipes, and other wrapper commands whose referenced body
  or dependency closure is unresolved;
- CI shell fragments containing shell-variable expansion, workflow expressions, matrix/context
  substitution, working-directory indirection, conditional control flow, or multi-command
  orchestration;
- commands requiring unmodelled browsers, native desktop prerequisites, containers, services,
  credentials, host integration, or external effects.

Checked-in CI workflows and agent guidance are source evidence, not execution or safety authority.
They may corroborate a resolved candidate, contradict a proposed candidate, or explain why Ota
must retain `unknown`; they must never manufacture agent authorization or override a contradictory
manifest, resolver, or existing contract. A repository's existing `agent.safe_tasks` remains
maintainer-owned contract truth and is not inferred from prose.

Safety precedence is fail-closed: an existing contract remains authoritative; resolved
manifest/task-graph evidence supplies execution facts; CI and agent guidance can only corroborate
or contradict those facts. Any material contradiction, unresolved requirement, unknown effect, or
unsupported platform boundary prevents agent-safe promotion. Positive CI or prose evidence never
offsets a conflict or unknown.

Project identity is computed per resolved project root. Multiple applicable manifests at one root
are composed only through a registered deterministic rule; otherwise their disagreement is a
candidate conflict. A package or language manifest identity outranks an incidental shell script
name; a script can supply a task candidate but cannot replace an equally or more authoritative
manifest project identity. Lane identity binds the execution closure, working directory, platform,
requirements, services, environment, and effects, so fast and integration lanes remain distinct
even when their visible command text overlaps.

When inspected evidence distinguishes finite fast verification from a service-backed, integration,
or platform-specific lane, detection preserves those as separate candidates. It must not collapse
them into a generic `test` task or imply that an unmodelled service dependency is admitted.

### Candidate Snapshot Identity

Candidate identity is a canonical semantic hash over a selected source manifest, not timestamps or
machine state:

- every evidence path is normalized root-relative with `/` separators and sorted by path, source
  kind, and content identity;
- every selected regular file carries a SHA-256 identity; absolute paths, timestamps, and host
  identifiers are excluded;
- the detector records a sorted discovery inventory for every supported source location in its
  declared root and scope. Its identity is distinct from the selected evidence manifest so added,
  removed, renamed, or newly higher-precedence source truth cannot be hidden by unchanged selected
  files;
- existing parent paths and symlink targets are resolved before collection; a path or target that
  escapes the contract root is rejected rather than represented;
- candidate creation refuses when `--candidate-out`, after canonical parent and symlink resolution,
  aliases the contract file, any selected evidence path, or any selected evidence parent. It never
  excludes a colliding path from evidence and then overwrites it;
- Git `HEAD` and dirty posture are non-authoritative provenance fields; non-Git repositories omit
  them rather than inventing an identity;
- a dirty worktree is inspectable, and application refuses if the reviewed discovery inventory,
  selected evidence manifest, or contract snapshot no longer matches.

This identity deliberately binds inspected truth, not all repository content. Unselected material
behavior remains outside the candidate's asserted coverage.

## Candidate Admission and Writing

Candidate creation and application need one explicit, reviewable protocol. The provisional public
shape is:

```text
ota init --candidate-out <path> --json
ota detect --candidate-out <path> --json
ota contract upgrade --candidate-out <path> --json
ota detect --candidate-out <path> --replace-candidate --json
ota contract apply-candidate <path> --json
ota contract apply-candidate <path> --write --json
ota contract apply-candidate <path> --write --require-complete --json
ota contract apply-candidate <path> --write --carrier git --json
```

Creation writes only the caller-selected candidate artifact, never `ota.yaml`; it uses create-new
atomic no-replace semantics on Linux and macOS and refuses elsewhere or for an existing path.
`--replace-candidate` may replace only a regular,
schema-valid Ota candidate after canonical path and hardlink-alias checks; it never overwrites an
arbitrary file. A successful inspection exits zero even when its artifact contains conflicts,
unknowns, or unsupported facts.

Application treats the candidate as untrusted review input, not producer authority. Before dry-run
or write it must verify the artifact self-identity, require the exact registered detector or
migration implementation identity, re-run that detector over the recorded logical root and
discovery scope, require the discovery inventory and selected evidence identities to match, and
re-derive the proposed semantic changes and application projection. A later implementation may be
accepted only when the registry declares a versioned byte-equivalent derivation contract for that
exact candidate schema and implementation identity. It refuses when the re-derived canonical
candidate differs.
Application is dry-run by default and writes atomically only with `--write`. Its typed non-zero
outcomes are `candidate_malformed`, `candidate_identity_invalid`, `candidate_implementation_incompatible`,
`candidate_not_reproducible`, `candidate_stale`, `candidate_contract_mismatch`,
`candidate_conflict`, `candidate_incomplete`, `candidate_unsupported`, `candidate_write_failed`,
and `candidate_write_durability_uncertain`; a successful reapplication is an explicit semantic
no-op.

- `ota init --dry-run` emits the versioned `init_starter_preview_v1` source-bound candidate for
  the exact starter contract it previews. It binds the immutable discovery inventory, selected
  evidence, candidate identity, and resulting contract identity; every detected starter transform
  reads that retained capture, while an explicit pack retains its own detector-independent
  semantics. It carries no application projection or write authority. `ota init` may create a
  complete starter only when no contract exists; migrating that legacy writer through candidate
  application remains a separate change.
- `kind: detection` candidates use `operation: add` only. `ota detect` may propose additions to an
  existing contract, but must not overwrite an existing semantic field implicitly.
- V11.2 field-family precedence ranks external evidence and selects the primary proposed value
  deterministically. It never overrides existing canonical contract truth; a material disagreement
  with that truth remains a conflict and requires a maintainer-authored resolution.
- Low-confidence, unsupported, and unknown candidates are review output only. They never become
  contract declarations through a write flag.
- Candidate application is change-scoped: it may atomically apply every `applicable` change while
  retaining unrelated `unknown` or `unsupported` entries as residual findings. It refuses the
  whole write only when an unresolved entry targets the same subject, would be required for a
  selected change's semantic validity, or the caller passes `--require-complete`.
- An explicit write requires the candidate identity and current source/evidence identities to match
  the reviewed values, requires a re-derived candidate and application-projection match, and
  validates the exact projected post-apply contract identity. The initial carrier atomically
  creates only a previously absent `ota.yaml` through a no-follow repository handle. It does not
  replace an existing contract because a generic repository filesystem cannot safely
  compare-and-swap arbitrary external writers. Any source, contract, or semantic derivation drift
  refuses and asks for a new candidate; a post-publication directory-sync failure is reported as
  written-but-durability-uncertain rather than as an unwritten failure.
- Re-running detection over unchanged inputs must produce the same candidate identity and semantic
  diff. Applying an accepted candidate a second time is a no-op.

Detection candidates are `kind: detection` and never replace or remove contract fields. Source
absence is not authority to delete maintainer truth. Only `kind: upgrade` may use replacement or
removal operations, with a `migration_id`, before/after semantic proof, and the same explicit
reviewed write admission.

The first implementation checkpoint publishes and dry-run admits schema-v2 upgrade candidates.
It deliberately refuses upgrade `--write`: the create-new contract writer cannot provide atomic
compare-and-swap against a non-cooperating external writer of an existing `ota.yaml`. A later
owned update carrier must satisfy the write and reapplication acceptance bar before legacy
mutation commands can route through it.

### Git Existing-Contract Update Carrier

The first owned update carrier is explicit: `ota contract apply-candidate <path> --write --carrier
git`. It is not the default `--write` behavior and it does not make a generic working tree an
authority boundary. It admits only a non-detached Git checkout where `ota.yaml` is tracked at
`HEAD`, the index and working-tree entry both match that exact `HEAD` blob, and the candidate base
identity, re-derived source/evidence, projection, and resulting contract all remain exact. It
constructs one contract-only commit, updates the checked-out branch with an expected-old `HEAD`
compare-and-swap, and then materializes and re-reads `ota.yaml` to confirm it matches the committed
contract. A competing branch advance, staged contract change, untracked or symlinked contract,
materialization mismatch, or post-write worktree mismatch refuses with a typed outcome.

The carrier records the prior and resulting commit identities, current branch ref, candidate and
projection identities, and resulting contract identity in successful machine output. It scrubs
caller `GIT_*` routing state and disables configured Git helper execution before every carrier Git
command. A successful
ref update followed by failed materialization is not reported as an unwritten failure: it is a
committed-but-worktree-unsynced terminal state with the exact prior/resulting commit and branch
identities plus an operator recovery instruction. The carrier currently requires the same Linux or
macOS no-follow directory-descriptor support as candidate application. Ota does not claim to prevent a same-user process from changing the
worktree after that observation; subsequent application re-derives repository truth and refuses
when it differs. The carrier never pushes, rebases, amends, changes unrelated paths, accepts a
detached `HEAD`, or silently converts existing `--write` calls into commits.

The compatibility window closes with the first owned existing-contract carrier. Repo-level
`detect --merge`, `--apply`, `--apply-all`, `--rewrite`, and `--yes` are removed from the public
CLI rather than preserved as a second mutation authority. Parser tombstones refuse them before
repository detection, candidate construction, locking, or mutation, with stable text and JSON
migration output. Read-only discovery remains `ota detect`; reviewable proposed additions remain
`ota detect --candidate-out`; create-new application remains `ota contract apply-candidate --write`;
and an existing tracked contract update requires the explicit Git carrier. `detect --write` remains
a temporary first-contract-only convenience alias, but must construct and apply the same
source-bound candidate through the create-new evaluator. It refuses when `ota.yaml` already exists.
No command claims a replacement equivalent for rewrite: detection candidates cannot express
replacement or removal without a separately versioned, reviewable operation model. Runnable and
agent-safety promotion remain explicitly outside this compatibility change and must use the shared
fail-closed closure classifier.

Before routing `detect --write`, Ota must add a versioned conservative first-contract candidate
profile. The general detection-review candidate intentionally carries unresolved review findings
and only direct inference operations; it cannot silently become the historical starter writer.
The profile must carry every conservative detector promotion, starter-owned default, generated
metadata field, and attributed source transformation as canonical candidate operations, then derive
the same complete validated projection through the shared evaluator. A profile that cannot produce
that projection refuses rather than falling back to a direct YAML writer.

## Contract Quality View

Candidate evidence disposition is distinct from V11.14 assurance. It is a source-inspection fact,
not a policy input or an assertion that an arbitrary contract field is supported:

| Candidate disposition | Meaning |
| --- | --- |
| `applicable` | Deterministic inspected evidence can propose this conservative change. |
| `conflict` | Inspected evidence conflicts with existing contract truth or another source. |
| `unknown` | Ota lacks enough inspected evidence to propose the field. |
| `unsupported` | The material behavior needs an ownership surface Ota does not ship. |

V11.14 remains the only canonical assurance carrier for its existing claim families, including
agent safety and proof breadth. Machine output preserves its canonical
`supported | contradicted | unknown` status. In human-only authoring guidance, `corroborated` is
display wording for `supported`; `declared` describes the maintainer's claim, never its assurance
status. V11.22 must not relabel arbitrary candidate fields or uninspected material behavior as
V11.14 assurance.

The quality view must classify each material inspected behavior as one of:

- contract-owned with applicable candidate evidence or existing V11.14 assurance within stated
  coverage;
- contract-owned but conflicting, contradicted, or unknown;
- explicitly bounded by `not_proved` or an unsupported-capability refusal;
- repository-owned outside the selected scope; or
- a named Ota platform gap with the missing ownership surface.

It must never score a repository, hide unknowns behind a percentage, or promote a green command
into repo-global governance.

## Deterministic Contract Upgrades

V11.22 introduces one explicit upgrade path, provisionally named `ota contract upgrade`. It emits
the same versioned `ContractCandidate` artifact that `apply-candidate` reviews and writes:

1. Read one contract snapshot and identify its schema/capability migration.
2. Produce a versioned semantic upgrade candidate and a human-readable diff by default.
3. Bind the candidate to the original semantic snapshot identity and upgrade implementation
   version.
4. Refuse write when the current contract changed, the migration is lossy, or an unresolved
   candidate conflict remains.
5. Write atomically only through an explicit apply flag; never update pins, baselines, or inferred
   operational values automatically.

The first concrete migration is `legacy_flat_toolchain_fulfillment_v1`: a recognized legacy flat
`toolchains.<name>.fulfillment: run|none` representation becomes the documented structured
`fulfillment.mode` shape while preserving the same toolchain owner, version, and fulfillment
source. It does not convert unmanaged `runtimes.<name>` checks into toolchains. The migration
registry is version-aware and reads a bounded raw YAML document before current-schema validation,
dispatching only a registered `from_version`/representation pair. A document with no registered
reader or lossless migration returns a typed `upgrade_unsupported` candidate; it is never coerced
through a current parser or rewritten speculatively.

The initial shipped carrier stops after candidate publication and reproducible dry-run admission.
It does not overwrite the existing contract. That bounded checkpoint proves migration discovery,
identity, semantics, and review UX without weakening the separate existing-file publication gate.

An upgrade may normalize a deprecated representation only when the new representation has the same
published semantics. Any narrower, broader, or uncertain interpretation is a review-required
candidate, not an automatic migration. The output must disclose formatting impact separately from
semantic change.

## Implementation Order

1. Extract a shared candidate/evidence and execution-closure domain below CLI commands; do not
   embed rules in output formatting.
2. Implement the shared closure resolver and fail-closed runnable/agent-safety classifier before
   candidate publication or legacy write migration. This is an intentional compatibility tightening:
   previously inferred safe tasks may become ordinary runnable or `unknown` candidates when their
   closure is unresolved, and the documented legacy detection-selection semantics do not override
   that safety rule.
3. Route existing `init` and `detect` dry-run paths through canonical candidate creation using that
   resolver without changing their public write behavior yet.
4. Publish the versioned candidate artifact, candidate schema, human review output, source
   identities, conflict detail, and snapshot binding.
5. Add explicit dry-run/write admission for candidate application and deterministic schema
   upgrades.
6. Display V11.14 assurance only for its existing claim families; do not reload policy or
   re-observe sources independently within one command.
7. Add the versioned conservative first-contract candidate profile, then route `detect --write`
   through candidate creation and the create-new evaluator. Remove repo-level legacy mutation flags
   with pre-admission migration refusals; preserve no rewrite/replacement alias until a reviewed
   operation model exists.

## Acceptance Bar

- unchanged repository evidence yields byte-stable normalized candidate JSON and the same
  candidate identity even when unrelated Git provenance changes;
- every proposed field has named source evidence or is explicitly `unknown`/`unsupported`;
- every runnable proposal serializes its canonical closure and attributable source evidence for
  every material node, edge, requirement, effect, classification, and unresolved reason;
- V11.22 emits no inferred agent-safe proposal; unknown effects remain explicit, existing
  maintainer-authored safety remains authoritative, and positive effect-backed promotion is
  deferred to V12;
- verifier-like names alone never yield agent-safe candidates; unresolved wrapper, CI-context, and
  shell-variable fixtures remain `unknown` with attributed source evidence;
- CI and agent-guidance fixtures prove corroboration/contradiction without creating
  `agent.safe_tasks` authority;
- manifest project identity wins over incidental shell-script discovery, and separately evidenced
  fast, service-backed, and platform-specific verification lanes remain distinct candidates;
- fixtures with multiple applicable manifests at one project root prove registered deterministic
  composition or an explicit project-identity conflict;
- conflict fixtures prove Ota does not overwrite existing fields or choose a competing source;
- stale source and stale contract snapshots refuse before a write;
- added, removed, renamed, or newly higher-precedence supported sources invalidate the recorded
  discovery-inventory identity and refuse application before a write;
- applying the same candidate twice is a semantic no-op;
- detection never proposes removal from absent evidence; versioned removal migrations require
  explicit semantic proof and reviewed write admission;
- field-family precedence ranks external evidence deterministically while an existing contract
  disagreement remains an explicit non-overwriting conflict;
- the first registered legacy flat-toolchain-fulfillment migration proves raw version-aware loading,
  lossless mapping, unsupported-old-document refusal, and reviewed application;
- candidate artifact creation, dry-run, write, stale/conflict/incomplete refusal, and removed
  legacy mutation-flag migration refusals each have stable JSON and exit semantics;
- candidate output uses create-new semantics; replacement accepts only a schema-valid prior Ota
  candidate and refuses arbitrary-file, symlink, and hardlink collisions;
- application rejects malformed identity, incompatible detector/migration implementation, stale
  evidence, and any candidate whose current re-derivation does not match the reviewed artifact;
- re-derivation uses the exact registered implementation identity, unless a registry-owned,
  versioned byte-equivalent derivation contract explicitly permits the replacement;
- applicable changes can write while unrelated unknown or unsupported entries remain visible;
  same-subject, semantic-prerequisite, and `--require-complete` gaps refuse;
- removed legacy mutation flags refuse before repository inspection or mutation; `detect --write`
  uses the same candidate evaluator and create-new apply path;
- upgrades are versioned, deterministic, reviewable, and refuse lossy or ambiguous migration;
- `doctor`, `init`, `detect`, and candidate output share one command-scoped source observation set;
  V11.14 records are reused only when that command evaluates an existing supported claim family;
- JSON schemas, command/reference docs, examples, canonical skills, and site guidance are updated
  only when a public candidate or upgrade command ships;
- pressure separately includes one no-contract `init` repository, one existing-contract conflict
  repository, and one existing legacy-adopter upgrade repository;
- every pressure result includes the full uncovered-material-behavior inventory.

## Pressure Targets

The first pressure set should be deliberately different:

- the upstream Buzz checkout without a pressure contract, where `ota init` must produce a
  source-bound candidate from visible manifests and Compose topology while leaving `just` and shell
  helper sequencing explicitly unknown or unsupported;
- an existing-contract Caddy checkout, where candidate non-overwrite and a deterministic
  contract/source conflict can be proved without inventing deployment truth;
- Flowise at committed historical revision `90121fac54e234ca83e2a85c435354af1df8ac8f`, which carries
  the legacy flat Node toolchain fulfillment declaration, where the first upgrade must be proven
  against that exact historical truth. A synthetic Caddy history cannot close this migration
  pressure gate.
- Atuin, after a manually reviewed local candidate models separate stable Rust verification,
  nightly formatting, non-service Nextest, and PostgreSQL-backed integration lanes. Its detected
  starter must not select `install.sh` as project identity or collapse service-backed integration
  into generic `cargo test`.
- GitButler, after package-script/Turbo/Tauri/Playwright closure resolution is available or the
  affected tasks remain `unknown`. Its agent and CI files may explain the boundary but cannot
  authorize desktop, browser, Docker, or E2E work.
- BAML, only from a clean checkout with captured acquisition evidence. A reusable-workflow command
  containing unresolved shell variables is a regression fixture for non-promotion, not a valid
  generic task candidate.

The next reviewed public-repository queue is intentionally staged rather than cloned or contacted
all at once:

- Pixi is the first V11.22 detection-safety pressure target after activation: its declared
  environments, platforms, task dependencies, generated-schema work, and secret-backed slow tests
  must remain separate candidates with explicit evidence boundaries.
- Biome is the confirming task-closure target: Rust, pnpm, Just, WASM, code generation, and
  Linux/macOS/Windows verification must not be flattened into familiar task-name heuristics.
- Harper and Dioxus are later multi-runtime pressure targets. Browser-extension, Tauri, desktop,
  mobile, and external-runtime behavior must be bounded rather than promoted through a broad
  green check.
- Lemma is a later wrapper-command and multi-toolchain target; its Rust task aliases and CI
  toolchain setup must be resolved or retained as unknown.
- Jujutsu is a later cross-platform Rust contract/CI-parity target, useful after the more novel
  Pixi and Biome source-closure work is proven.
- Berty is deferred until Ota is ready to model a deliberately narrow mobile/P2P slice. Its
  device, network, protobuf, and multi-language behavior would otherwise dominate the pressure
  result without proving the current V11.22 boundary.

These are evidence-driven selection notes, not outreach commitments or claims that any repository
is fully governed. Clone only the target currently needed to prove the next product boundary.

The bar is not a generated `ota.yaml` that validates. The no-contract repository must prove
candidate creation and explicit application; Caddy must prove conflict and non-overwrite behavior;
the identified legacy adopter must prove upgrade behavior. Atuin must exercise detector-produced
output for its distinct lanes; GitButler must prove unresolved package/Turbo/Tauri/Playwright work
does not become agent-safe; and BAML must use a clean revision-pinned checkout to prove an
unresolved reusable-workflow variable is not promoted. All pressure evidence must carry source
attribution, candidate identity, closure identity where applicable, and truthful material-behavior
classification before V11.22 is called complete.

The required pressure set is now evidenced. Immutable Linux/macOS Buzz pressure
[32795701879](https://github.com/bobaikato/buzz/actions/runs/32795701879) proves deterministic
no-contract candidate creation, create-new application, validation, and no-op reapplication at
`block/buzz@2d280376ad36134cec1f23bead6d866d30bed147`; it creates only `ota.yaml` and emits no
agent-safe proposal. Immutable Linux/macOS Flowise pressure
[32794430600](https://github.com/bobaikato/Flowise/actions/runs/32794430600) proves the registered
legacy upgrade and Git-carrier application at
`FlowiseAI/Flowise@90121fac54e234ca83e2a85c435354af1df8ac8f`; its semantic contract identity is
unchanged, its commit contains only `ota.yaml`, and reapplication is a no-op. These runs do not
execute repository tasks or approve candidate changes. V11.22 remains active until an independent
closure audit accepts the implementation, evidence, and propagated public claims.

## Explicit Non-Goals

- LLM-generated contracts or hosted inference;
- account, billing, bot, fleet, or automatic pull-request features;
- bidirectional synchronization with CI providers;
- automatic baseline or identity-pin updates;
- inferred agent-safe promotion before V12 supplies typed effect and realization eligibility;
- proving opaque shell behavior, external service state, or maintainer intent from absence of
  evidence.
