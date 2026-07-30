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

Status: planned and inactive. This is the next OSS planning slice after completed V11.21. It does
not authorize implementation until this plan receives review and is explicitly activated.

## Sequencing

V11.7 remains partially implemented: reusable grant authority, crossing-time liveness, scope
checks, and authorizer binding are deferred. V11.22 neither authorizes execution nor consumes a
crossing record as candidate authority, so it is not architecturally dependent on that work. Its
activation must nevertheless explicitly reconfirm the V11.7 deferral in the handoff; V11.22 must
not imply that a recorded crossing is an approval or grant.

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
  confidence: high | medium | low
  disposition: applicable | conflict | unknown | unsupported
```

The candidate is not `ota.yaml`. It is a portable, versioned JSON review artifact. Its identity
binds the logical root, discovery inventory, selected evidence manifest, detector version, proposed
semantic changes, and existing contract snapshot so a later write cannot silently apply a result
discovered against different source truth. Git provenance is carried beside that identity, not
folded into it.

Evidence must be deterministic and path-specific. Initial supported source families are manifests,
lockfiles, toolchain files, Compose files, checked-in CI workflows, and existing Ota contracts.
Opaque shell, uninspected helper scripts, live services, ambient environment variables, and
provider-owned configuration remain `unknown` unless a shipped detector can identify a narrower
fact without inference.

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
ota contract apply-candidate <path> --dry-run --json
ota contract apply-candidate <path> --write --json
ota contract apply-candidate <path> --write --require-complete --json
```

Creation writes only the caller-selected candidate artifact, never `ota.yaml`; it uses create-new
semantics by default and refuses an existing path. `--replace-candidate` may replace only a regular,
schema-valid Ota candidate after canonical path and hardlink-alias checks; it never overwrites an
arbitrary file. A successful inspection exits zero even when its artifact contains conflicts,
unknowns, or unsupported facts.

Application treats the candidate as untrusted review input, not producer authority. Before dry-run
or write it must verify the artifact self-identity, require the exact registered detector or
migration implementation identity, re-run that detector over the recorded logical root and
discovery scope, require the discovery inventory and selected evidence identities to match, and
re-derive the proposed semantic changes. A later implementation may be accepted only when the
registry declares a versioned byte-equivalent derivation contract for that exact candidate schema
and implementation identity. It refuses when the re-derived canonical candidate differs.
Application is dry-run by default and writes atomically only with `--write`. Its typed non-zero
outcomes are `candidate_malformed`, `candidate_identity_invalid`, `candidate_implementation_incompatible`,
`candidate_not_reproducible`, `candidate_stale`, `candidate_contract_mismatch`,
`candidate_conflict`, `candidate_incomplete`, and `candidate_unsupported`; a successful
reapplication is an explicit semantic no-op.

- `ota init` may create a complete starter only when no contract exists and its candidate is
  explicit about every inferred source.
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
- An explicit write requires the candidate identity and the current contract snapshot identity to
  match the reviewed values, and requires a re-derived candidate match. Any source, contract, or
  semantic derivation drift refuses and asks for a new candidate.
- Re-running detection over unchanged inputs must produce the same candidate identity and semantic
  diff. Applying an accepted candidate a second time is a no-op.

Detection candidates are `kind: detection` and never replace or remove contract fields. Source
absence is not authority to delete maintainer truth. Only `kind: upgrade` may use replacement or
removal operations, with a `migration_id`, before/after semantic proof, and the same explicit
reviewed write admission.

Shipped `detect --merge --apply`, `--apply-all`, and `--rewrite --yes` retain their existing
semantics during a compatibility window, but every contract-writing path must construct, validate,
and atomically apply through the one shared candidate evaluator. They are compatibility UX, not a
second write authority. They preserve documented selection and interaction semantics only; stale,
identity, reproducibility, collision, and semantic-conflict refusals remain mandatory. Any changed
exit behavior caused by those refusals is a documented compatibility tightening. Deprecation, if
later chosen, must document the equivalent `apply-candidate` invocation and preserve an explicit
write.

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

An upgrade may normalize a deprecated representation only when the new representation has the same
published semantics. Any narrower, broader, or uncertain interpretation is a review-required
candidate, not an automatic migration. The output must disclose formatting impact separately from
semantic change.

## Implementation Order

1. Extract a shared candidate/evidence domain below CLI commands; do not embed rules in output
   formatting.
2. Route existing `init` and `detect` dry-run paths through canonical candidate creation without
   changing their public write behavior yet.
3. Publish the versioned candidate artifact, candidate schema, human review output, source
   identities, conflict detail, and snapshot binding.
4. Add explicit dry-run/write admission for candidate application and deterministic schema
   upgrades.
5. Display V11.14 assurance only for its existing claim families; do not reload policy or
   re-observe sources independently within one command.
6. Only then migrate legacy high-confidence writes internally to the candidate path while retaining
   their documented compatibility semantics.

## Acceptance Bar

- unchanged repository evidence yields byte-stable normalized candidate JSON and the same
  candidate identity even when unrelated Git provenance changes;
- every proposed field has named source evidence or is explicitly `unknown`/`unsupported`;
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
- candidate artifact creation, dry-run, write, stale/conflict/incomplete refusal, and legacy
  mutation-command compatibility each have stable JSON and exit semantics;
- candidate output uses create-new semantics; replacement accepts only a schema-valid prior Ota
  candidate and refuses arbitrary-file, symlink, and hardlink collisions;
- application rejects malformed identity, incompatible detector/migration implementation, stale
  evidence, and any candidate whose current re-derivation does not match the reviewed artifact;
- re-derivation uses the exact registered implementation identity, unless a registry-owned,
  versioned byte-equivalent derivation contract explicitly permits the replacement;
- applicable changes can write while unrelated unknown or unsupported entries remain visible;
  same-subject, semantic-prerequisite, and `--require-complete` gaps refuse;
- every legacy mutation command exercises the same candidate evaluator and atomic apply path;
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
- a separately identified existing adopter that already carries a legacy flat toolchain fulfillment
  declaration, where the first upgrade is proven against committed historical truth. A synthetic
  Caddy history cannot close this migration pressure gate.

The bar is not a generated `ota.yaml` that validates. The no-contract repository must prove
candidate creation and explicit application; Caddy must prove conflict and non-overwrite behavior;
the identified legacy adopter must prove upgrade behavior. All pressure evidence must carry source
attribution, candidate identity, and truthful material-behavior classification before V11.22 is
called complete.

## Explicit Non-Goals

- LLM-generated contracts or hosted inference;
- account, billing, bot, fleet, or automatic pull-request features;
- bidirectional synchronization with CI providers;
- automatic baseline or identity-pin updates;
- proving opaque shell behavior, external service state, or maintainer intent from absence of
  evidence.
