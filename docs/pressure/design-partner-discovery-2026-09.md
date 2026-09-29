# September 2026 Design-Partner Pressure Discovery

The canonical evidence and exact fork revisions are in
[`evidence-manifest.json`](evidence-manifest.json). dbmask PR #37 is merged; the remaining
three design-partner tracks are draft PRs. A merge records acceptance of one bounded integration,
not repository-wide adoption or endorsement. Their retained matrices prove only the selected
lanes. The [dbmask engineering note](https://ota.run/blog/pressure-testing-ota-on-dbmask-postgresql-5m9q)
is published; the other engineering notes remain drafts pending upstream review.

## Historical Gap Recheck

- **Aggregate execution-mode propagation (sem):** The older v1.6.27 finding is
  retained in [`sem-discovery.md`](sem-discovery.md). The current fork still
  selects `test` and `local-index:freshness` separately in its hosted container
  lane, so the [retained run](https://github.com/bobaikato/sem/actions/runs/36239195385)
  does **not** exercise aggregate container execution. A local dry-run of
  `ota run verify --container --agent` with the current source-built v1.6.28
  binary resolved the complete container-selected closure. No post-tag changes
  to `src/runner.rs` or `src/doctor.rs` were found. Thus the historical finding
  must not be presented as a confirmed current product gap. A hosted aggregate
  run on the released binary is the return trigger if that coverage matters.

## Widening Questions, Not Confirmed Defects

- **Run-to-measurement evidence (Agent Threat Rules):** A repository-owned
  verifier checks that the PINT-format measurement is fresh for the selected
  execution and retains its inputs and outputs. Determine whether Ota's
  existing receipt and claim-assurance surfaces can bind the exact input,
  selected command, resulting measurement, and covered commit without that
  bespoke verifier. Do not call this a missing Core feature until the existing
  surfaces are tested against the same false-green fixture. The repository's
  850-sample corpus is not Lakera's official PINT benchmark.
- **Opaque script steps (sem):** Ota selects and bounds the shell-carried
  two-revision fixture but does not independently model every command inside
  it. Any Core design should preserve an explicit repository-owned opaque
  script boundary rather than pretend a green task proves each hidden step.
- **Tool-version probing (Anodizer):** The broader, superseded pressure work
  suggested an `nfpm --version` parsing mismatch. The current draft no longer
  exercises nfpm, so this is a reproduction candidate, not a finding from its
  green matrix. Capture the exact banner and a Core regression before treating
  it as a confirmed v1.6.28 defect.

## Uncovered Material Behavior

| Case | Contract-owned and proved | Bounded / `not_proved` | Repo-owned outside selected scope | Ota gap or candidate |
| --- | --- | --- | --- | --- |
| dbmask | Selected SQLite lanes and synthetic PostgreSQL sequence; merged upstream in PR #37 | Real data, MySQL, release, and repository-wide safety | SQL fixture assertions and date-classification fix | No confirmed Core gap in this lane |
| sem | Selected native and container local-index tasks | Cloud index, MCP consumers, hosted aggregate container selection, and release | Fixture shell commands and parser semantics | Historical aggregate finding needs a released hosted recheck; opaque-step modeling question |
| Agent Threat Rules | Selected Ota evaluation task and non-blocking lane | Official benchmark, certification, arbitrary runner integrity, and repository-wide rule quality | 850-sample corpus, regression gate, and measurement verifier | Test whether Core can bind run-to-measurement provenance |
| Anodizer | Selected disposable Linux preflight, matching builds, and drift refusal | Cross-platform release, signing, publication, and agent safety | Determinism fixture and release pipeline | Reproduce suspected nfpm version-probe issue separately |

No provider, external authority, production-data, or deployment conclusion is
drawn from these four selected cases. Keep upstream PRs narrow and keep these internal
product questions out of maintainer-facing evidence unless a concrete Ota fix
changes the selected lane.
