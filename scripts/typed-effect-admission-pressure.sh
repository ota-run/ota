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
mkdir -p "$fixture/migrations" "$fixture/.ota" "$preview_fixture/migrations" "$preview_fixture/.ota"
printf 'create table example ();\n' > "$fixture/migrations/001.sql"
cp "$fixture/migrations/001.sql" "$preview_fixture/migrations/001.sql"

python3 - "$fixture" "$preview_fixture" <<'PY'
import hashlib
import json
import pathlib
import sys
import textwrap

fixture = pathlib.Path(sys.argv[1])
preview_fixture = pathlib.Path(sys.argv[2])
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
PY
record_stage fixture_created

stage=contract_validation
git -C "$core_root" rev-parse HEAD > "$proof_root/core-revision.txt"
"$ota" validate "$fixture" --plain 2>&1 | tee "$proof_root/validate.txt"
"$ota" validate "$preview_fixture" --plain 2>&1 | tee "$proof_root/preview-validate.txt"
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
grep -Fq 'Phase: preconditions' "$proof_root/proof-refusal.txt" \
  || fail "ota proof runtime did not preserve the preconditions stage"
grep -Fq 'effect policy denied' "$fixture/.ota/proof/typed/up.log" \
  || fail "proof up.log did not retain the typed effect-policy refusal"
test ! -e "$fixture/setup-sentinel" || fail "workflow setup executed before refusal"
test ! -e "$fixture/.env.typed" || fail "workflow environment rendered before refusal"
test ! -e "$fixture/.ota/state/logs" || fail "durable execution logs were created before refusal"
record_stage execution_refusals_verified

stage=refusal_schema_validation
"$ota" json validate --schema up.json --allow-exit 1 --assert-eq ok=false \
  --assert-eq governance.post_execution.execution_attempted=false \
  --write-payload "$proof_root/up-refusal.json" -- "$ota" up --json "$fixture"
record_stage refusal_schema_validated

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
  'policy_decision_published' \
  'policy_denial_code_published' \
  'setup_sentinel_absent' \
  'workflow_env_artifact_absent' \
  'stale_input_refused' \
  'intermediate_symlink_escape_refused' > "$proof_root/checkpoints.txt"
rm -f "$proof_root/failure.txt"
