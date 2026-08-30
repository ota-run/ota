#!/usr/bin/env bash
#
#                █████
#               ░░███
#       ██████  ███████    ██████
#      ███░░███░░░███░    ░░░░░███
#     ░███ ░███  ░███      ███████
#     ░███ ░███  ░███ ███ ███░░███
#     ░░██████   ░░█████ ░░████████
#      ░░░░░░     ░░░░░   ░░░░░░░░
#
#   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
#
#   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
#
#   Licensed under the Apache License, Version 2.0 (the "License");
#   you may not use this file except in compliance with the License.
#   You may obtain a copy of the License at
#
#       http://www.apache.org/licenses/LICENSE-2.0
#
#   Unless required by applicable law or agreed to in writing, software
#   distributed under the License is distributed on an "AS IS" BASIS,
#   WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
#   See the License for the specific language governing permissions and
#   limitations under the License.

set -euo pipefail

if [ "$#" -ne 3 ]; then
  printf 'usage: %s <ota-binary> <evidence-root> <core-root>\n' "$0" >&2
  exit 2
fi

ota=$1
proof_root=$2
core_root=$3
fixture="$proof_root/fixture"
preview_fixture="$proof_root/preview-fixture"
ci_fixture="$proof_root/ci-fixture"
stage=initialization

record_stage() {
  stage=$1
  printf '%s\n' "$stage" >> "$proof_root/stages-completed.txt"
}

fail() {
  printf '%s\n' "$1" >&2
  return 1
}

on_error() {
  status=$?
  printf 'failed_stage=%s\nexit_status=%s\n' "$stage" "$status" > "$proof_root/failure.txt"
  exit "$status"
}

trap on_error ERR

test -x "$ota" || fail "Ota binary is not executable: $ota"
test -d "$core_root/.git" || fail "Core root is not a Git checkout: $core_root"
mkdir -p "$proof_root"
test -z "$(find "$proof_root" -mindepth 1 -maxdepth 1 -print -quit)" \
  || fail "Evidence root must be empty: $proof_root"
stage=fixture_creation
mkdir -p "$fixture/migrations" "$fixture/.ota" "$preview_fixture/migrations" \
  "$preview_fixture/.ota" "$ci_fixture/migrations" "$ci_fixture/.ota"
printf 'create table example ();\n' > "$fixture/migrations/001.sql"
cp "$fixture/migrations/001.sql" "$preview_fixture/migrations/001.sql"
cp "$fixture/migrations/001.sql" "$ci_fixture/migrations/001.sql"

python3 - "$fixture" "$preview_fixture" "$ci_fixture" <<'PY'
import hashlib
import json
import pathlib
import sys
import textwrap

fixture = pathlib.Path(sys.argv[1])
preview_fixture = pathlib.Path(sys.argv[2])
ci_fixture = pathlib.Path(sys.argv[3])
migration = fixture / "migrations" / "001.sql"
file_identity = "sha256:" + hashlib.sha256(migration.read_bytes()).hexdigest()
manifest = {
    "schema_version": 1,
    "root": "migrations",
    "files": [{"path": "001.sql", "identity": file_identity}],
}
manifest_json = json.dumps(manifest, sort_keys=True, separators=(",", ":"), ensure_ascii=False)
manifest_identity = "sha256:" + hashlib.sha256(
    b"ota.schema-migration-manifest.v1\0" + manifest_json.encode()
).hexdigest()
common = textwrap.dedent(f"""\
version: 1
metadata:
  ota:
    minimum_version: "1.6.27"
project:
  name: typed-effect-admission-pressure
resource_bindings:
  primary:
    kind: database
    provider: postgresql
    namespace: {{ authority: dns:example.org, environment: pressure, account: primary }}
effect_definitions:
  migration:
    kind: database_schema_mutation
    action: apply_migration_set
    resource: {{ engine: postgresql, target_ref: primary, schema: public }}
    bounds:
      migration_set: {{ root: migrations, content_identity: {manifest_identity} }}
      start_state: any_within_set
""")
(preview_fixture / "ota.yaml").write_text(common + textwrap.dedent("""\
tasks:
  migrate:
    action: { kind: database_schema_mutation, effect: migration }
    effects:
      declared: [migration]
workflows:
  default: typed
  typed:
    run:
      task: migrate
"""))
(ci_fixture / "ota.yaml").write_text(common + textwrap.dedent("""\
tasks:
  migrate:
    safe_for_agent: true
    action: { kind: database_schema_mutation, effect: migration }
    effects:
      declared: [migration]
workflows:
  default: typed
  typed:
    run:
      task: migrate
agent:
  safe_tasks: [migrate]
"""))
(fixture / "ota.yaml").write_text(common + textwrap.dedent("""\
env:
  profiles:
    typed:
      env:
        TYPED_SENTINEL: must-not-render
      render:
        dotenv:
          path: .env.typed
          include: [TYPED_SENTINEL]
tasks:
  setup:
    command:
      exe: sh
      args: [-c, "touch setup-sentinel"]
  migrate:
    action: { kind: database_schema_mutation, effect: migration }
    effects:
      declared: [migration]
  migrate-declared-only:
    command:
      exe: sh
      args: [-c, "touch declared-only-sentinel"]
    effects:
      declared: [migration]
  mixed:
    command:
      exe: sh
      args: [-c, "touch mixed-sentinel"]
    depends_on: [migrate, migrate-declared-only]
workflows:
  default: typed
  typed:
    env:
      profile: typed
    setup:
      task: setup
    run:
      task: migrate
    proof:
      claim: bounded
agent:
  effect_refusal_canaries:
    - id: pressure_schema_refusal
      effect: migration
      challenge_lanes:
        - task: migrate
          origin: { task: migrate, effect: migration }
        - workflow: typed
          origin: { task: migrate, effect: migration }
    - id: pressure_declared_only_refusal
      effect: migration
      challenge_lanes:
        - task: mixed
          origin: { task: migrate-declared-only, effect: migration }
"""))
policy = textwrap.dedent("""\
policies:
  effects:
    mode: compatibility
    typed:
      rules:
        - id: deny_pressure_schema_mutation
          selector:
            kind: database_schema_mutation
            actions: [apply_migration_set]
            resource:
              match: any
              engine: postgresql
          decision: deny
""")
(fixture / ".ota/org-policy.yaml").write_text(policy)
(preview_fixture / ".ota/org-policy.yaml").write_text(policy)
(ci_fixture / ".ota/org-policy.yaml").write_text(textwrap.dedent("""\
policies:
  effects:
    mode: compatibility
"""))
PY
record_stage fixture_created
printf '%s\n' \
  'policies:' \
  '  effects:' \
  '    mode: strict' > "$proof_root/strict-policy.yaml"

stage=contract_validation
git -C "$core_root" rev-parse HEAD > "$proof_root/core-revision.txt"
"$ota" validate "$fixture" --plain 2>&1 | tee "$proof_root/validate.txt"
"$ota" validate "$preview_fixture" --plain 2>&1 | tee "$proof_root/preview-validate.txt"
"$ota" validate "$ci_fixture" --plain 2>&1 | tee "$proof_root/ci-validate.txt"
record_stage contracts_validated

stage=preview_validation
"$ota" json validate --schema run-preview.json --assert-eq dry_run=true \
  --assert-eq task=migrate --assert-non-empty-string plan.effect_application_plans.0.identity \
  --assert-eq plan.effect_policy_decision.aggregate_decision=deny \
  --assert-eq plan.effect_policy_decision.explicit_typed_deny=true \
  --write-payload "$proof_root/run-dry-run.json" \
  -- "$ota" run migrate --dry-run --json "$preview_fixture"
"$ota" json validate --schema up.json --assert-eq dry_run=true \
  --assert-eq phase=preview --write-payload "$proof_root/up-dry-run.json" \
  -- "$ota" up --dry-run --json "$preview_fixture"
record_stage previews_validated

stage=mixed_realization_refusal
"$ota" json validate --schema run-preview.json --allow-exit 1 \
  --assert-eq ok=false --assert-eq preview_status=BLOCKED \
  --assert-eq summary.primary_blocker.code=typed_effect_admission_refused \
  --assert-eq plan.effect_policy_decision.aggregate_decision=deny \
  --write-payload "$proof_root/mixed-realization-preview.json" \
  -- "$ota" run mixed --dry-run --json "$fixture"
python3 - "$proof_root/mixed-realization-preview.json" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
effects = payload["plan"]["effect_policy_decision"]["effects"]
if len(effects) != 2:
    raise SystemExit(f"expected two mixed realizations, got {len(effects)}")
typed = next((effect for effect in effects if effect["derivation_posture"] == "declared_and_typed"), None)
declared = next((effect for effect in effects if effect["derivation_posture"] == "declared_only"), None)
if typed is None or declared is None:
    raise SystemExit(f"missing mixed realization posture: {effects}")
if typed["effect_identity"] != declared["effect_identity"]:
    raise SystemExit("mixed realizations did not bind the same effect")
if typed["attachment_identity"] == declared["attachment_identity"]:
    raise SystemExit("mixed realizations collapsed their attachment identities")
if typed["realization_identity"] == declared["realization_identity"]:
    raise SystemExit("mixed realizations collapsed their realization identities")
if typed["eligible"] is not True or declared["eligible"] is not False:
    raise SystemExit(f"mixed realization eligibility is incorrect: {effects}")
PY
if "$ota" run mixed --plain "$fixture" > "$proof_root/mixed-realization-refusal.txt" 2>&1; then
  mixed_status=0
else
  mixed_status=$?
fi
[ "$mixed_status" -eq 1 ] || fail "mixed realization run returned $mixed_status, expected 1"
grep -Fq 'declared-only effect realization' "$proof_root/mixed-realization-refusal.txt" \
  || fail "mixed realization run did not report structural declared-only refusal"
test ! -e "$fixture/declared-only-sentinel" \
  || fail "declared-only command executed before mixed realization refusal"
test ! -e "$fixture/mixed-sentinel" \
  || fail "mixed command executed before mixed realization refusal"
record_stage mixed_realization_refusal_verified

stage=ci_projection_policy_reconciliation
"$ota" json validate --schema ci-projection.json --assert-eq ok=true \
  --assert-eq projection.governance.effect_policy_decision.aggregate_decision=warn \
  --assert-eq projection.governance.effect_policy_decision.explicit_typed_deny=false \
  --assert-non-empty-string projection.identity \
  --assert-non-empty-string projection.governance.effect_policy_decision.identity \
  --write-payload "$proof_root/ci-projection-warn.json" \
  -- "$ota" ci projection --workflow typed --mode native --target-os "${OTA_PRESSURE_TARGET_OS:-linux}" \
    --json "$ci_fixture"
cp "$ci_fixture/.ota/org-policy.yaml" "$proof_root/ci-policy-warn.yaml"
cat > "$ci_fixture/.ota/org-policy.yaml" <<'YAML'
policies:
  effects:
    mode: compatibility
    typed:
      rules:
        - id: deny_pressure_ci_migration
          selector:
            kind: database_schema_mutation
            actions: [apply_migration_set]
            resource:
              match: exact
              engine: postgresql
              namespace: { authority: dns:example.org, environment: pressure, account: primary }
              schema: public
          decision: deny
YAML
cp "$ci_fixture/.ota/org-policy.yaml" "$proof_root/ci-policy-deny.yaml"
"$ota" json validate --schema ci-projection.json --allow-exit 1 \
  --assert-eq ok=false --assert-eq code=effect_policy_denied \
  --assert-eq projection.governance.effect_policy_decision.aggregate_decision=deny \
  --assert-eq projection.governance.effect_policy_decision.explicit_typed_deny=true \
  --assert-non-empty-string projection.identity \
  --assert-non-empty-string projection.governance.effect_policy_decision.identity \
  --write-payload "$proof_root/ci-projection-deny.json" \
  -- "$ota" ci projection --workflow typed --mode native --target-os "${OTA_PRESSURE_TARGET_OS:-linux}" \
    --json "$ci_fixture"
python3 - "$proof_root/ci-projection-warn.json" "$proof_root/ci-projection-deny.json" <<'PY'
import json
import pathlib
import sys

warn = json.loads(pathlib.Path(sys.argv[1]).read_text())
deny = json.loads(pathlib.Path(sys.argv[2]).read_text())
if warn["projection"]["identity"] == deny["projection"]["identity"]:
    raise SystemExit("typed effect-policy drift did not change CI projection identity")
PY
test ! -e "$ci_fixture/setup-sentinel" || fail "CI projection executed workflow setup"
test ! -e "$ci_fixture/.ota/state/logs" || fail "CI projection created durable execution logs"
record_stage ci_projection_policy_reconciled

stage=execution_refusal
if "$ota" run migrate --plain "$fixture" > "$proof_root/run-refusal.txt" 2>&1; then
  run_status=0
else
  run_status=$?
fi
if "$ota" up --plain "$fixture" > "$proof_root/up-refusal.txt" 2>&1; then
  up_status=0
else
  up_status=$?
fi
if "$ota" proof runtime --plain "$fixture" > "$proof_root/proof-refusal.txt" 2>&1; then
  proof_status=0
else
  proof_status=$?
fi
printf 'run_status=%s\nup_status=%s\nproof_status=%s\n' \
  "$run_status" "$up_status" "$proof_status" > "$proof_root/terminal-status.txt"
[ "$run_status" -eq 1 ] || fail "ota run returned $run_status, expected 1"
[ "$up_status" -eq 1 ] || fail "ota up returned $up_status, expected 1"
[ "$proof_status" -eq 1 ] || fail "ota proof runtime returned $proof_status, expected 1"
grep -Fq 'OTA_EFFECT_POLICY_DENIED' "$proof_root/run-refusal.txt" \
  || fail "ota run did not report the typed effect-policy denial code"
grep -Fq 'Typed effect denied by policy' "$proof_root/up-refusal.txt" \
  || fail "ota up did not report typed effect-policy denial"
grep -Fq 'OTA_EFFECT_POLICY_DENIED' "$proof_root/proof-refusal.txt" \
  || fail "ota proof runtime did not report the typed effect-policy denial code"
test ! -e "$fixture/.ota/proof" \
  || fail "proof artifacts were created before typed effect-policy refusal"
test ! -e "$fixture/setup-sentinel" || fail "workflow setup executed before refusal"
test ! -e "$fixture/.env.typed" || fail "workflow environment rendered before refusal"
test ! -e "$fixture/.ota/state/logs" || fail "durable execution logs were created before refusal"
record_stage execution_refusals_verified

stage=effect_refusal_canary
"$ota" json validate --schema effect-refusal-canary.json --assert-eq ok=true \
  --assert-eq status=passed --assert-eq canary.lane_kind=task \
  --assert-eq canary.lane_target=migrate --assert-eq canary.actual_decision=deny \
  --assert-eq canary.execution_started=false \
  --assert-non-empty-string canary.canary_identity \
  --write-payload "$proof_root/effect-canary-task.json" \
  -- "$ota" run --agent --expect-effect-refusal pressure_schema_refusal --json migrate "$fixture"
"$ota" json validate --schema effect-refusal-canary.json --assert-eq ok=true \
  --assert-eq status=passed --assert-eq canary.lane_kind=workflow \
  --assert-eq canary.lane_target=typed --assert-eq canary.actual_decision=deny \
  --assert-eq canary.execution_started=false \
  --assert-non-empty-string canary.canary_identity \
  --write-payload "$proof_root/effect-canary-workflow.json" \
  -- "$ota" up --workflow typed --agent --expect-effect-refusal pressure_schema_refusal --json "$fixture"
"$ota" run --agent --expect-effect-refusal pressure_schema_refusal --plain migrate "$fixture" \
  > "$proof_root/effect-canary-plain.txt" 2>&1
grep -Fq 'EFFECT REFUSAL CANARY pressure_schema_refusal' "$proof_root/effect-canary-plain.txt" \
  || fail "plain effect-refusal canary did not render its heading"
if grep -Fq '🦦' "$proof_root/effect-canary-plain.txt"; then
  fail "plain effect-refusal canary rendered an icon"
fi
if env OTA_POLICY="$proof_root/strict-policy.yaml" "$ota" run --agent \
  --expect-effect-refusal pressure_schema_refusal --json migrate "$fixture" \
  > "$proof_root/effect-canary-strict-fallback-raw.json" 2>&1; then
  strict_canary_status=0
else
  strict_canary_status=$?
fi
[ "$strict_canary_status" -eq 1 ] \
  || fail "strict fallback canary returned $strict_canary_status, expected 1"
"$ota" json validate --schema effect-refusal-canary.json --allow-exit 1 \
  --assert-eq ok=false --assert-eq status=failed \
  --assert-eq canary.reason_code=effect_canary_explicit_typed_deny_not_observed \
  --write-payload "$proof_root/effect-canary-strict-fallback.json" \
  -- env OTA_POLICY="$proof_root/strict-policy.yaml" "$ota" run --agent \
    --expect-effect-refusal pressure_schema_refusal --json migrate "$fixture"
if "$ota" run --agent --expect-effect-refusal unknown_pressure_refusal --json migrate "$fixture" \
  > "$proof_root/effect-canary-unknown-raw.json" 2>&1; then
  unknown_canary_status=0
else
  unknown_canary_status=$?
fi
[ "$unknown_canary_status" -eq 1 ] \
  || fail "unknown canary returned $unknown_canary_status, expected 1"
"$ota" json validate --schema effect-refusal-canary.json --allow-exit 1 \
  --assert-eq ok=false --assert-eq status=not_evaluated \
  --assert-eq canary.reason_code=effect_canary_unknown \
  --write-payload "$proof_root/effect-canary-unknown.json" \
  -- "$ota" run --agent --expect-effect-refusal unknown_pressure_refusal --json migrate "$fixture"
"$ota" json validate --schema effect-refusal-canary.json --allow-exit 1 \
  --assert-eq ok=false --assert-eq status=assurance_gap \
  --assert-eq canary.reason_code=effect_canary_realization_ineligible \
  --assert-eq canary.execution_started=false \
  --assert-non-empty-string canary.effect_identity \
  --assert-non-empty-string canary.attachment_identity \
  --assert-non-empty-string canary.realization_identity \
  --write-payload "$proof_root/effect-canary-declared-only.json" \
  -- "$ota" run --agent --expect-effect-refusal pressure_declared_only_refusal --json mixed "$fixture"
python3 - "$proof_root/mixed-realization-preview.json" "$proof_root/effect-canary-declared-only.json" <<'PY'
import json
import pathlib
import sys

preview = json.loads(pathlib.Path(sys.argv[1]).read_text())
canary = json.loads(pathlib.Path(sys.argv[2]).read_text())["canary"]
declared = next(
    effect
    for effect in preview["plan"]["effect_policy_decision"]["effects"]
    if effect["derivation_posture"] == "declared_only"
)
if canary["effect_identity"] != declared["effect_identity"]:
    raise SystemExit("declared-only canary did not bind the selected effect identity")
if canary["attachment_identity"] != declared["attachment_identity"]:
    raise SystemExit("declared-only canary did not bind the selected attachment identity")
PY
test ! -e "$fixture/setup-sentinel" || fail "effect-refusal canary executed workflow setup"
test ! -e "$fixture/.env.typed" || fail "effect-refusal canary rendered workflow environment"
test ! -e "$fixture/.ota/state/logs" || fail "effect-refusal canary created durable execution logs"
test ! -e "$fixture/declared-only-sentinel" \
  || fail "declared-only canary executed the declared-only command"
test ! -e "$fixture/mixed-sentinel" || fail "declared-only canary executed the mixed command"
record_stage effect_refusal_canary_verified

stage=refusal_schema_validation
"$ota" json validate --schema up.json --allow-exit 1 --assert-eq ok=false \
  --assert-eq governance.post_execution.execution_attempted=false \
  --write-payload "$proof_root/up-refusal.json" -- "$ota" up --json "$fixture"
record_stage refusal_schema_validated

stage=durable_refusal_archive
"$ota" json validate --schema up.json --allow-exit 1 --assert-eq status=BLOCKED \
  --assert-eq receipt.typed_effect_policy_refusal.execution_started=false \
  --assert-non-empty-string receipt.typed_effect_policy_refusal.refusal_archive_path \
  --assert-non-empty-string receipt.typed_effect_policy_refusal.policy_snapshot_archive.path \
  --write-payload "$proof_root/refusal-archive-up.json" \
  -- "$ota" up --workflow typed --archive-effect-refusal --json "$fixture"
python3 - "$proof_root/refusal-archive-up.json" "$fixture" "$proof_root" <<'PY'
import json
import pathlib
import sys

payload = json.loads(pathlib.Path(sys.argv[1]).read_text())
fixture = pathlib.Path(sys.argv[2])
proof_root = pathlib.Path(sys.argv[3])
refusal = payload["receipt"]["typed_effect_policy_refusal"]
archive_path = fixture / refusal["refusal_archive_path"]
policy_snapshot_path = fixture / refusal["policy_snapshot_archive"]["path"]
contract_snapshot_path = fixture / payload["receipt"]["contract_snapshot_ref"]
for path in (archive_path, policy_snapshot_path, contract_snapshot_path):
    if not path.is_file():
        raise SystemExit(f"expected durable refusal artifact: {path}")
(proof_root / "refusal-archive-path.txt").write_text(str(archive_path) + "\n")
(proof_root / "refusal-archive-original.json").write_bytes(archive_path.read_bytes())
PY
"$ota" receipt --history --json "$fixture" > "$proof_root/refusal-archive-history-valid.json"
python3 - "$proof_root/refusal-archive-history-valid.json" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text())["summary"]
if summary["archive_count"] != 1 or summary["invalid_archive_count"] != 0:
    raise SystemExit(f"unexpected valid archive history: {summary}")
PY
python3 - "$proof_root/refusal-archive-path.txt" <<'PY'
import json
import pathlib
import sys

archive_path = pathlib.Path(pathlib.Path(sys.argv[1]).read_text().strip())
archive = json.loads(archive_path.read_text())
archive.pop("archive_context")
archive_path.write_text(json.dumps(archive, indent=2) + "\n")
PY
"$ota" receipt --history --json "$fixture" > "$proof_root/refusal-archive-history-tampered.json"
python3 - "$proof_root/refusal-archive-history-tampered.json" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text())["summary"]
if summary["archive_count"] != 0 or summary["invalid_archive_count"] != 1:
    raise SystemExit(f"tampered archive was accepted: {summary}")
PY
python3 - "$proof_root/refusal-archive-path.txt" "$proof_root/refusal-archive-original.json" <<'PY'
import pathlib
import sys

archive_path = pathlib.Path(pathlib.Path(sys.argv[1]).read_text().strip())
archive_path.write_bytes(pathlib.Path(sys.argv[2]).read_bytes())
PY
"$ota" receipt --history --json "$fixture" > "$proof_root/refusal-archive-history-restored.json"
python3 - "$proof_root/refusal-archive-history-restored.json" <<'PY'
import json
import pathlib
import sys

summary = json.loads(pathlib.Path(sys.argv[1]).read_text())["summary"]
if summary["archive_count"] != 1 or summary["invalid_archive_count"] != 0:
    raise SystemExit(f"restored archive did not reconcile: {summary}")
PY
test ! -e "$fixture/setup-sentinel" || fail "refusal archive executed workflow setup"
test ! -e "$fixture/.env.typed" || fail "refusal archive rendered workflow environment"
test ! -e "$fixture/.ota/state/logs" || fail "refusal archive created durable execution logs"
record_stage durable_refusal_archive_verified

stage=stale_input_refusal
cp -R "$preview_fixture" "$proof_root/stale-fixture"
printf 'alter table example add column value integer;\n' \
  >> "$proof_root/stale-fixture/migrations/001.sql"
if "$ota" run migrate --dry-run --json "$proof_root/stale-fixture" \
  > "$proof_root/stale-refusal.json" 2>&1; then
  stale_status=0
else
  stale_status=$?
fi
printf 'stale_status=%s\n' "$stale_status" > "$proof_root/stale-status.txt"
[ "$stale_status" -eq 1 ] || fail "stale migration returned $stale_status, expected 1"
grep -Fq 'effect_application_migration_set_drift' "$proof_root/stale-refusal.json" \
  || fail "stale migration did not report the bounded drift refusal"
test ! -e "$proof_root/stale-fixture/.ota/state/logs" \
  || fail "stale dry-run created durable execution logs"
record_stage stale_input_refused

stage=symlink_escape_refusal
mkdir -p "$proof_root/outside/migrations"
cp "$preview_fixture/migrations/001.sql" "$proof_root/outside/migrations/001.sql"
cp -R "$preview_fixture" "$proof_root/symlink-fixture"
ln -s "$proof_root/outside" "$proof_root/symlink-fixture/alias"
python3 - "$proof_root/symlink-fixture/ota.yaml" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
value = path.read_text()
path.write_text(value.replace("root: migrations", "root: alias/migrations", 1))
PY
if "$ota" run migrate --dry-run --json "$proof_root/symlink-fixture" \
  > "$proof_root/symlink-refusal.json" 2>&1; then
  symlink_status=0
else
  symlink_status=$?
fi
printf 'symlink_status=%s\n' "$symlink_status" > "$proof_root/symlink-status.txt"
[ "$symlink_status" -eq 1 ] || fail "symlink escape returned $symlink_status, expected 1"
grep -Fq 'effect_application_migration_set_invalid' "$proof_root/symlink-refusal.json" \
  || fail "symlink escape did not report an application-plan refusal"
record_stage symlink_escape_refused

printf '%s\n' \
  'run_refused_before_side_effects' \
  'up_refused_before_side_effects' \
  'proof_inherited_up_precondition_refusal' \
  'mixed_realization_refused_before_side_effects' \
  'mixed_realization_identity_bound' \
  'policy_decision_published' \
  'policy_denial_code_published' \
  'ci_projection_warn_identity_bound' \
  'ci_projection_checkout_deny_refused' \
  'ci_projection_identity_changed_with_policy' \
  'effect_refusal_canary_task_passed' \
  'effect_refusal_canary_workflow_passed' \
  'effect_refusal_canary_strict_fallback_failed' \
  'effect_refusal_canary_unknown_failed' \
  'effect_refusal_canary_declared_only_assurance_gap' \
  'effect_refusal_canary_plain_output_icon_free' \
  'durable_refusal_archive_created' \
  'durable_refusal_archive_history_rederived' \
  'durable_refusal_archive_tamper_rejected' \
  'durable_refusal_archive_restored' \
  'setup_sentinel_absent' \
  'workflow_env_artifact_absent' \
  'stale_input_refused' \
  'intermediate_symlink_escape_refused' > "$proof_root/checkpoints.txt"
rm -f "$proof_root/failure.txt"
