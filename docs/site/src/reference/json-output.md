# JSON Output

ota supports machine-readable JSON for core commands and workspace commands.

Use it when you need stable parsing in CI, editors, agents, or scripts.

## Source model

This page is the canonical public reference for JSON output. It adds examples,
use cases, and operator guidance so the page stands on its own while staying
aligned with shipped behavior.

## Why it matters

- JSON is the integration contract for automation
- human text output can change without breaking machines
- the same payload shape should work for repo and workspace flows
- consumers should not need to scrape terminal text to make decisions

## What to use

Use the smallest surface that matches the job:

- `agents` for repo-local `AGENTS.md` export preview or sync reports
- `tasks` for repo task inventory and agent-safe task hints
- `validate` for contract gating
- `init` for starter-contract scaffolding and inference
- `detect` for contract comparison, merge, and write decisions
- `doctor` for readiness diagnosis
- `check` for checks-only repo readiness
- `workspace explain` for ordered remediation
- `workspace init` for workspace scaffolding
- `workspace detect` for workspace contract detection
- `workspace tasks` for workspace inventory and task availability
- `workspace list` for lightweight workspace inventory and readiness
- `workspace check` for checks-only workspace readiness with a roll-up summary
- `up` and `workspace up` for preparation and readiness roll-up
- `workspace run` for coordinated multi-repo execution roll-up and receipts
- `diff` and `explain` for change impact and remediation planning

## What consumers should read

Editor and CI consumers should prefer the smallest stable fields for the job:

- `ota agents --json` for `ok`, `path`, `output`, `written`, `mode`, and `content`
- `ota tasks --json` for `ok`, `path`, `agent`, and `tasks`
- `ota validate --json` and `ota workspace validate --json` for `ok`, `summary.error_count`, `errors` or `error`, and `next`
- `ota init --json` and `ota workspace init --json` for scaffolding mode, written state, inferred fields, and comparison details
- `ota detect --json` and `ota workspace detect --json` for comparison, inference, and merge/write outcomes
- `ota doctor --json` and `ota workspace doctor --json` for the top-level `summary`, per-repo `findings`, `execution`, and primary-blocker details
- `ota check --json` and `ota workspace check --json` for checks-only findings and roll-ups
- `ota workspace explain --json` for the top-level `summary`, per-repo `findings`, and per-repo `steps`
- `ota workspace tasks --json` for the top-level `summary`, per-repo `tasks`, and dependency order
- `ota workspace list --json` for the top-level `summary`, per-repo readiness, and contract presence
- `ota workspace check --json` for the top-level `summary` and per-repo findings
- `ota up --json` and `ota workspace up --json` for the top-level `summary`, `receipt`, and per-repo results
- `ota workspace run --json` for the top-level `summary`, `receipt`, and per-repo results
- `ota diff --json` and `ota explain --json` for the change summary and remediation steps

## Canonical schema URLs

The current shipped JSON schemas are published under the ota spec distribution URL:

- [`agents.json`](https://dist.ota.run/spec/json-schemas/latest/agents.json)
- [`tasks.json`](https://dist.ota.run/spec/json-schemas/latest/tasks.json)
- [`validate.json`](https://dist.ota.run/spec/json-schemas/latest/validate.json)
- [`init.json`](https://dist.ota.run/spec/json-schemas/latest/init.json)
- [`detect.json`](https://dist.ota.run/spec/json-schemas/latest/detect.json)
- [`doctor.json`](https://dist.ota.run/spec/json-schemas/latest/doctor.json)
- [`check.json`](https://dist.ota.run/spec/json-schemas/latest/check.json)
- [`up.json`](https://dist.ota.run/spec/json-schemas/latest/up.json)
- [`workspace-init.json`](https://dist.ota.run/spec/json-schemas/latest/workspace-init.json)
- [`workspace-tasks.json`](https://dist.ota.run/spec/json-schemas/latest/workspace-tasks.json)
- [`workspace-run.json`](https://dist.ota.run/spec/json-schemas/latest/workspace-run.json)
- [`workspace-check.json`](https://dist.ota.run/spec/json-schemas/latest/workspace-check.json)
- [`workspace-doctor.json`](https://dist.ota.run/spec/json-schemas/latest/workspace-doctor.json)
- [`workspace-explain.json`](https://dist.ota.run/spec/json-schemas/latest/workspace-explain.json)
- [`workspace-up.json`](https://dist.ota.run/spec/json-schemas/latest/workspace-up.json)
- [`diff.json`](https://dist.ota.run/spec/json-schemas/latest/diff.json)
- [`explain.json`](https://dist.ota.run/spec/json-schemas/latest/explain.json)

## Design intent

- JSON shapes are treated as stable integration surfaces
- human text output and JSON output are intentionally separate
- exit code behavior and JSON payloads should be consumed together in automation
- validation JSON includes a compact `summary.error_count` so hosted gates can read one field instead of re-counting errors
- workspace doctor and explain JSON include top-level summary roll-ups for repo, finding, and step counts so hosted consumers do not have to derive them from nested reports
- doctor JSON can also surface contract-drift warning findings when repo signals no longer match the declared contract; those findings include `ownership` and `provenance` so hosted consumers can classify the mismatch as repo-contract drift and point users to `ota detect --merge --dry-run` for the comparison preview
- `path` usually reflects the resolved contract path as rendered by current CLI path compaction

## Common patterns

- success payloads include `ok: true`
- failure payloads include `ok: false` and structured error/findings context
- workspace commands include per-repo result objects when applicable
- execution metadata is descriptive and should be consumed directly rather than reconstructed from text output

## Practical integration pattern

For each command execution in automation:

1. run with `--json`
2. check the process exit code first
3. parse payload fields such as `ok`, `errors`, `findings`, `summary`, `receipt`, and per-repo reports

Example:

```bash
ota doctor --json | tee .ota-doctor.json
ota workspace doctor --json | tee .ota-workspace-doctor.json
ota workspace explain --json | tee .ota-workspace-explain.json
```

## Use cases

- a CI job runs `ota doctor --json`, fails on errors, and posts warnings as annotations
- an editor shows the primary blocker without parsing text output
- an agent reads the receipt to see what ota actually did
- a workspace gate surfaces per-repo readiness instead of flattening everything into one string
- a release pipeline compares contract changes through `ota diff --json` before writing updates
- a repo author previews a new `AGENTS.md` with `ota agents --json` before writing it

## Surface summaries

### `ota agents --json`

Use for repo-local `AGENTS.md` export preview or sync reports. The payload
includes `ok` for success, `path` for the repo contract path, `output` for the
rendered file path, `written` for whether a file was changed, `mode` for how
the request was handled, and `content` for the generated text when present.

### `ota tasks --json`

Use for repo task inventory and agent-safe task hints. The payload includes
`ok` for success, `path` for the repo contract path, `agent` for agent-specific
task hints, and `tasks` for the repo task list with descriptions, notes,
dependencies, and inputs when present.

### `ota init --json`

Use for starter-contract scaffolding and inference. The payload includes `ok`
for success, `path` for the contract path, `written` for whether ota wrote a
file, `mode` for the init mode, `config` for the generated or previewed
contract, and `inferred` for the fields ota inferred and their provenance.

### `ota detect --json`

Use for contract comparison, merge, and write decisions. The payload includes
`ok` for success, `path` for the contract path, `written` for whether ota wrote
changes, `config` for the detected or merged contract, `comparison` for the
contract comparison, and `inferred` for the inferred fields and provenance.

### `ota workspace init --json`

Use for workspace scaffolding. The payload includes `ok` for success, `path`
for the workspace contract path, `written` for whether ota wrote a workspace
contract, `mode` for the workspace init mode, `config` for the generated or
previewed workspace contract, `included` for the repos in the scaffold,
`missing_contract` for repo entries that still need a contract, and
`comparison` for the comparison against the existing workspace contract.

### `ota workspace detect --json`

Use for workspace contract detection and comparison. The payload includes `ok`
for success, `path` for the workspace contract path, `written` for whether ota
wrote changes, `mode` for the detect mode, `config` for the detected or merged
workspace contract, `included` for the repos detected, `missing_contract` for
repo entries that still need a contract, and `comparison` for the workspace
comparison.

## JSON surface guide

### `ota validate --json`

Use for contract gating. `ok`, `summary.error_count`, `errors` or `error`, and `next` are the stable fields.

Typical use:

- reject an invalid contract before merge
- show one concise error block in CI
- feed a simple editor status badge

### `ota doctor --json`

Use for readiness diagnosis. Read `summary`, `findings`, `execution`, and `primary_blocker` when present.

What the payload gives you:

- a severity-ordered blocker list
- the repo or workspace execution context
- env provenance when execution metadata is declared
- policy context when policy-aware diagnosis is surfaced

Finding objects always include stable identity fields: `code` is the stable
machine-readable finding identifier, `category` is the broad type of finding
such as contract or readiness, `owner` is which surface owns the finding such
as the repo contract, and `evidence` is the structured proof behind the
finding.

Finding objects may also include additive policy context keys when policy-aware diagnosis is
surfaced: `policy_outcome` is the decision made by the policy layer,
`policy_reason` is why that policy decision was made, `policy_source` is which
policy source supplied the value or decision, `install_scope` is where the
policy applies such as repo or workspace scope, and `mutation_allowed` says
whether the policy allows mutation for the affected surface.

Doctor JSON can also surface contract-drift warning findings when repo signals no longer match the
declared contract. Those findings include `ownership` and `provenance` so hosted consumers can
classify the mismatch as repo-contract drift and point users to `ota detect --merge --dry-run` for
the comparison preview.

Example shape:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "error_count": 1,
    "warn_count": 1,
    "info_count": 0
  },
  "findings": [
    {
      "code": "OTA_TASKS_MISSING",
      "category": "contract",
      "owner": "repo_contract",
      "severity": "warn",
      "summary": "No tasks defined in contract",
      "why": "...",
      "next": "...",
      "evidence": {
        "observed": "...",
        "expected": "...",
        "source": "...",
        "checked_at": "...",
        "command": "...",
        "path": "..."
      }
    }
  ]
}
```

### `ota workspace doctor --json`

Use for workspace readiness and roll-up diagnosis. Read the top-level summary and per-repo findings and execution metadata.

The workspace version adds:

- per-repo readiness
- dependency-ordered diagnosis
- workspace roll-up counts
- optional acquisition state for missing repos

The same finding shape is reused per repo, so editors and CI can render one consistent issue model
across repo and workspace diagnosis.

Example shape:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "repo_count": 1,
    "ready_count": 0,
    "not_ready_count": 1,
    "error_count": 1,
    "warn_count": 0,
    "info_count": 0
  },
  "repos": [
    {
      "name": "web",
      "path": "/abs/path/to/apps/web",
      "contract_path": "/abs/path/to/apps/web/ota.yaml",
      "required": true,
      "ok": false,
      "execution": {
        "preferred": "native",
        "supported": ["native"],
        "lifecycle": "persistent"
      },
      "findings": [
        {
          "severity": "error",
          "summary": "Repo not acquired: web",
          "why": "...",
          "next": "..."
        }
      ]
    }
  ]
}
```

### `ota up --json`

Use for repo preparation and receipts. Read `summary` and `receipt`.

`ota up --json` has two failure classes:

- execution reached the `up` pipeline: returns `UpStatus` (`status`, `phase`, `findings`, optional `service`/`task`/`exit_code`)
- contract load/validation failed before the `up` pipeline: returns `ValidateFailure` shape (`ok`, `path`, and either `errors` or `error`)

Optional fields:

- `service`: present when a required service start command fails
- `task`: present when a task failure is reported
- `exit_code`: present when a child command failure is reported

Example:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "status": "READY",
  "phase": "post-setup diagnosis",
  "findings": []
}
```

### `ota workspace up --json`

Use for workspace preparation and rollout receipts. Read `summary`, `receipt`, and per-repo results.

The workspace receipt mirrors the top-level execution roll-up and keeps backend-aware execution
metadata on the same surface as the repo-level execution commands.

Optional per-repo fields:

- `service`
- `task`
- `exit_code`
- `stdout`
- `stderr`
- `env_sources`

Example:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.workspace.yaml",
  "summary": {
    "error_count": 0,
    "warn_count": 0,
    "info_count": 0,
    "step_count": 4
  },
  "receipt": {
    "ok": true,
    "path": "/abs/path/to/ota.workspace.yaml",
    "scope": "workspace",
    "contract": "/abs/path/to/ota.workspace.yaml",
    "workspace": "ota-dev",
    "steps": []
  }
}
```

### `ota diff --json`

Use for semantic contract comparison. Read `summary.readiness_impact` and the ordered `changes`.

The summary includes readiness-impact data and field counts. The `changes` array preserves
deterministic ordering and may include optional `provenance` for policy-section changes.

Useful cases:

- review what a proposed contract change will do before writing it
- compare a branch against main in CI
- summarize the impact of a workspace bootstrap change

Example:

```json
{
  "ok": true,
  "path": "/abs/path/to/ota.yaml",
  "base": "/abs/path/to/before/ota.yaml",
  "target": "/abs/path/to/after/ota.yaml",
  "summary": {
    "readiness_impact": "improves",
    "added_count": 0,
    "removed_count": 0,
    "changed_count": 1,
    "weakened_count": 0,
    "strengthened_count": 1
  },
  "changes": [
    {
      "path": "policies.env.OTA_ENV.default",
      "status": "change",
      "base": "local",
      "target": "ci",
      "provenance": "policy"
    }
  ]
}
```

### `ota explain --json`

Use for remediation plans. Read the ordered `steps`, stable `code`, and optional `provenance`.

`ota explain --json` turns readiness findings into ordered remediation steps and keeps the plan
read-only and deterministic.

Each step includes `order` for fix order starting at 1, `code` for the stable
finding identifier, `severity` for the error/warn/info level, `summary` for
the short human-readable title, `why` for the reason the step exists, `next`
for the safest next action ota can recommend, and optional `provenance` when
the step came from policy or drift.

Example:

```json
{
  "ok": false,
  "path": "/abs/path/to/ota.yaml",
  "summary": {
    "error_count": 1,
    "warn_count": 1,
    "info_count": 0,
    "step_count": 2
  },
  "steps": [
    {
      "order": 1,
      "code": "OTA_TASKS_MISSING",
      "severity": "error",
      "summary": "No tasks defined in contract",
      "why": "...",
      "next": "...",
      "provenance": "org policy"
    }
  ]
}
```

### `ota workspace tasks --json`

Use for workspace inventory and task availability.
Read `summary`, per-repo `tasks`, and dependency order. `summary` is the workspace-level repo and
readiness count, `tasks` is the task inventory for each repo, and `dependency order` is the
deterministic order ota respects when enumerating workspace tasks.

This is the surface you want when you need to know which tasks are available across a workspace
without reading every repo’s contract manually.

### `ota workspace list --json`

Use for lightweight workspace inventory and readiness.
Read `summary`, per-repo readiness, and contract presence. `summary` is the workspace inventory
count, `per-repo readiness` tells you whether each repo is ready, ready-but-acquired, or not ready,
and `contract presence` tells you whether the repo has a contract file.

This is the surface for a quick inventory view or workspace status dashboard.

### `ota workspace check --json`

Use for checks-only workspace readiness with a roll-up summary.
Read `summary` and per-repo findings. `summary` is the workspace-level check count and `per-repo
findings` are the check findings for each repo.

This keeps checks separate from task execution and gives CI one stable roll-up.

## Notes

- `ok: true` does not always mean zero findings; warning-only diagnosis can still be successful
- `path` usually reflects the resolved contract path as rendered by current CLI path compaction
- JSON mode does not change exit-code behavior
- warning-only diagnosis is still success
