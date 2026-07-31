//                █████
//               ░░███
//       ██████  ███████    ██████
//      ███░░███░░░███░    ░░░░░███
//     ░███ ░███  ░███      ███████
//     ░███ ░███  ░███ ███ ███░░███
//     ░░██████   ░░█████ ░░████████
//      ░░░░░░     ░░░░░   ░░░░░░░░
//
//   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.
//
//   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.
//
//   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
//   You may not use this file except in compliance with that License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs};

use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use tempfile::TempDir;

fn run_ota(args: &[&str], cwd: &Path) -> Value {
    run_ota_with_env(args, cwd, &[], true)
}

#[test]
fn crossing_grant_preview_refusal_matches_published_schema() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: crossing-grant-preview
governance:
  crossing_authority:
    authority_id: release-authority
tasks:
  publish:
    command:
      exe: sh
      args: ["-c", "printf publish"]
    safe_for_agent: false
"#,
    )
    .expect("contract");

    let preview = run_ota_json_output(&["run", "publish", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &preview);
    assert_eq!(preview["execution_started"], false);
    assert_eq!(
        preview["crossing_grant_admission"]["reason_family"],
        "crossing_grant_required"
    );
    assert_eq!(
        preview["crossing_grant_admission"]["authority_source"],
        "prebound_file"
    );
    assert_eq!(
        preview["crossing_grant_admission"]["authority_id"],
        "release-authority"
    );
    assert!(
        preview["crossing_grant_admission"]["scope_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("sha256:"))
    );
    assert!(
        preview["crossing_grant_admission"]["contract_identity"]
            .as_str()
            .is_some_and(|identity| identity.starts_with("sha256:"))
    );
    assert_eq!(
        preview["crossing_grant_admission"]["boundary_family"],
        "unsafe_task"
    );
    assert_eq!(
        preview["crossing_grant_admission"]["classification"],
        "escalated"
    );
    assert_eq!(
        preview["crossing_grant_admission"]["execution_started"],
        false
    );
}

#[test]
fn crossing_grant_up_refusal_receipt_carries_typed_authority_evidence() {
    let fixture = tempfile::tempdir().expect("tempdir");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: crossing-grant-up-refusal
governance:
  crossing_authority:
    authority_id: release-authority
tasks:
  publish:
    command:
      exe: sh
      args: ["-c", "printf publish"]
    safe_for_agent: false
workflows:
  default: release
  release:
    run:
      task: publish
"#,
    )
    .expect("contract");

    let refusal = run_ota_json_output(
        &["up", "--workflow", "release", "--json", "--receipt"],
        fixture.path(),
    );
    assert_matches_schema("up.json", &refusal);
    assert_eq!(refusal["receipt"]["crossing"], Value::Null);
    assert_eq!(
        refusal["receipt"]["refusal"]["boundary_family"],
        "crossing_grant_authority"
    );
    assert_eq!(
        refusal["receipt"]["refusal"]["authority_source"],
        "prebound_file"
    );
    assert_eq!(
        refusal["receipt"]["refusal"]["authority_id"],
        "release-authority"
    );
    assert_eq!(
        refusal["receipt"]["refusal"]["reason_family"],
        "crossing_grant_required"
    );
    assert_eq!(refusal["receipt"]["refusal"]["execution_started"], false);
}

fn run_ota_with_env(
    args: &[&str],
    cwd: &Path,
    envs: &[(&str, &str)],
    expect_success: bool,
) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .envs(envs.iter().copied())
        .output()
        .expect("ota command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert_eq!(
        output.status.success(),
        expect_success,
        "ota command status mismatch\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    let json_bytes = if expect_success {
        &output.stdout
    } else {
        &output.stderr
    };
    serde_json::from_slice(json_bytes).expect("command should emit valid JSON")
}

fn run_ota_failure_stdout_json(args: &[&str], cwd: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("ota command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        !output.status.success(),
        "ota command should fail\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    serde_json::from_slice(&output.stdout).expect("command should emit valid stdout JSON")
}

fn run_ota_json_output(args: &[&str], cwd: &Path) -> Value {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("ota command should run");

    serde_json::from_slice(&output.stdout).unwrap_or_else(|stdout_error| {
        serde_json::from_slice(&output.stderr).unwrap_or_else(|stderr_error| {
            panic!(
                "ota command should emit JSON\nstatus: {}\nstdout JSON error: {stdout_error}\nstderr JSON error: {stderr_error}\nstdout:\n{}\nstderr:\n{}",
                output.status,
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            )
        })
    })
}

fn run_ota_success_text(args: &[&str], cwd: &Path) -> String {
    let output = Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .current_dir(cwd)
        .output()
        .expect("ota command should run");
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        output.status.success(),
        "ota command should succeed\nstdout:\n{stdout}\nstderr:\n{stderr}"
    );
    stdout.into_owned()
}

fn schema_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/spec/json-schemas")
}

fn load_json(path: &Path) -> Value {
    let contents = fs::read_to_string(path).expect("JSON file should be readable");
    serde_json::from_str(&contents).expect("JSON file should parse")
}

fn assert_matches_schema(schema_name: &str, instance: &Value) {
    let schema_path = schema_dir().join(schema_name);
    let raw_schema = load_json(&schema_path);
    let mut options = JSONSchema::options();
    options.with_draft(Draft::Draft202012);
    for entry in fs::read_dir(schema_dir()).expect("schema dir should be readable") {
        let entry = entry.expect("schema dir entry should load");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let document = load_json(&path);
        if let Some(id) = document.get("$id").and_then(Value::as_str) {
            options.with_document(id.to_string(), document);
        }
    }
    let compiled = options.compile(&raw_schema).expect("schema should compile");
    if let Err(errors) = compiled.validate(instance) {
        let messages = errors.map(|error| error.to_string()).collect::<Vec<_>>();
        panic!(
            "instance did not match schema `{schema_name}`:\n{}",
            messages.join("\n")
        );
    }
}

fn assert_rejects_schema(schema_name: &str, instance: &Value) {
    let schema_path = schema_dir().join(schema_name);
    let raw_schema = load_json(&schema_path);
    let mut options = JSONSchema::options();
    options.with_draft(Draft::Draft202012);
    for entry in fs::read_dir(schema_dir()).expect("schema dir should be readable") {
        let entry = entry.expect("schema dir entry should load");
        let path = entry.path();
        if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
            continue;
        }
        let document = load_json(&path);
        if let Some(id) = document.get("$id").and_then(Value::as_str) {
            options.with_document(id.to_string(), document);
        }
    }
    let compiled = options.compile(&raw_schema).expect("schema should compile");
    assert!(
        compiled.validate(instance).is_err(),
        "instance unexpectedly matched schema `{schema_name}`"
    );
}

#[test]
fn lifecycle_proof_json_schema_accepts_runner_owned_transaction() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed_with_unproven_boundaries",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke",
            "intent": "manager_owned_service_lifecycle"
        },
        "transaction_id": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "services": [{
            "service": "database",
            "transaction_id": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            "preexisting_state": "inactive_observed",
            "cleanup_lease": "released",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_observed", "evidence_class": "derived" }
        }],
        "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }]
    });
    assert_matches_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_accepts_isolated_boundary_termination() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed_with_unproven_boundaries",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke",
            "intent": "runner_owned_isolated_lifecycle_boundary"
        },
        "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "services": [{
            "service": "caddy",
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "boundary_identity": "container:docker:ota-lifecycle-test",
            "preexisting_state": "boundary_absent_attested",
            "cleanup_lease": "released",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "boundary_terminated", "evidence_class": "attested" }
        }],
        "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "service_started_state_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "contract_lane"
        }, {
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }]
    });
    assert_matches_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_accepts_attested_isolated_cleanup_failure() {
    let payload = serde_json::json!({
        "ok": false,
        "proof_verdict": "failed",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke"
        },
        "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "services": [{
            "service": "caddy",
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "boundary_identity": "container:docker:ota-lifecycle-test",
            "preexisting_state": "boundary_absent_attested",
            "cleanup_lease": "cleanup_failed",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_not_observed", "evidence_class": "attested" }
        }],
        "finalization": { "state": "incomplete", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }],
        "error": "runner-owned lifecycle boundary could not be terminated"
    });
    assert_matches_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_rejects_attested_manager_state_failure() {
    let payload = serde_json::json!({
        "ok": false,
        "proof_verdict": "failed",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke"
        },
        "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "services": [{
            "service": "database",
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "preexisting_state": "inactive_observed",
            "cleanup_lease": "cleanup_failed",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "state_observed", "evidence_class": "derived" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_not_observed", "evidence_class": "attested" }
        }],
        "finalization": { "state": "incomplete", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }],
        "error": "manager state was not observed"
    });
    assert_rejects_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_rejects_manager_state_on_isolated_boundary() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed_with_unproven_boundaries",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": {
            "kind": "lifecycle_transition",
            "proof_class": "slice_proof",
            "workflow": "smoke"
        },
        "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
        "services": [{
            "service": "caddy",
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "boundary_identity": "container:docker:ota-lifecycle-test",
            "preexisting_state": "boundary_absent_attested",
            "cleanup_lease": "released",
            "ownership": "started_this_transaction",
            "start": { "state": "command_succeeded", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_observed", "evidence_class": "derived" }
        }],
        "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{
            "kind": "application_output_not_proved",
            "relative_to": "declared_lifecycle_service_transition",
            "source": "scope"
        }, {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "selected_lifecycle_workflow",
            "source": "scope"
        }]
    });
    assert_rejects_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_archive_schema_accepts_scope_bound_record() {
    let payload = serde_json::json!({
        "kind": "lifecycle_proof",
        "version": 2,
        "contract_identity": {
            "version": 1,
            "project": { "name": "archive" },
            "counts": { "runtimes": 0, "tools": 0, "env": 0, "services": 1, "checks": 0, "tasks": 1 }
        },
        "contract_snapshot_hash": "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
        "contract_snapshot_ref": ".ota/contracts/sha256-aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa.json",
        "scope": {
            "workflow": "smoke",
            "member": "api",
            "selected_services": ["database"],
            "service_closure": ["database"],
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "boundary_identity": "container:docker:ota-lifecycle-test",
            "backend": "container",
            "mode": "container",
            "provider": "docker",
            "lifecycle": "ephemeral",
            "target": "local",
            "target_os": "linux"
        },
        "proof": {
            "ok": true,
            "proof_verdict": "passed_with_unproven_boundaries",
            "path": "ota.yaml",
            "mode": "lifecycle-proof",
            "workflow": "smoke",
            "phase": "lifecycle",
            "stage_family": "proof",
            "proof_scope": { "kind": "lifecycle_transition", "proof_class": "slice_proof", "workflow": "smoke" },
            "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
            "services": [{
                "service": "database",
                "transaction_id": "sha256:bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb",
                "boundary_identity": "container:docker:ota-lifecycle-test",
                "preexisting_state": "boundary_absent_attested",
                "cleanup_lease": "released",
                "ownership": "started_this_transaction",
                "start": { "state": "command_succeeded", "evidence_class": "attested" },
                "readiness": { "state": "not_declared" },
                "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
                "teardown_assertion": { "state": "boundary_terminated", "evidence_class": "attested" }
            }],
            "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
            "not_proved": [{ "kind": "application_output_not_proved", "relative_to": "declared_lifecycle_service_transition", "source": "scope" }, { "kind": "broader_repo_completion_not_proved", "relative_to": "selected_lifecycle_workflow", "source": "scope" }]
        }
    });
    assert_matches_schema("proof-lifecycle-archive.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_rejects_unbounded_success() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": { "kind": "lifecycle_transition", "proof_class": "slice_proof" },
        "transaction_id": "sha256:cccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccccc",
        "services": [],
        "finalization": { "state": "not_run", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": []
    });
    assert_rejects_schema("proof-lifecycle.json", &payload);
}

#[test]
fn lifecycle_proof_json_schema_rejects_cross_phase_transition() {
    let payload = serde_json::json!({
        "ok": true,
        "proof_verdict": "passed_with_unproven_boundaries",
        "path": "ota.yaml",
        "mode": "lifecycle-proof",
        "workflow": "smoke",
        "phase": "lifecycle",
        "stage_family": "proof",
        "proof_scope": { "kind": "lifecycle_transition", "proof_class": "slice_proof" },
        "transaction_id": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
        "services": [{
            "service": "database",
            "transaction_id": "sha256:dddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddddd",
            "preexisting_state": "inactive_observed",
            "cleanup_lease": "released",
            "ownership": "started_this_transaction",
            "start": { "state": "boundary_terminated", "evidence_class": "attested" },
            "readiness": { "state": "not_declared" },
            "teardown": { "state": "command_succeeded", "evidence_class": "attested" },
            "teardown_assertion": { "state": "state_observed", "evidence_class": "derived" }
        }],
        "finalization": { "state": "completed", "after_interruption": false, "evidence_class": "attested" },
        "not_proved": [{ "kind": "application_output_not_proved", "relative_to": "declared_lifecycle_service_transition", "source": "scope" }, { "kind": "broader_repo_completion_not_proved", "relative_to": "selected_lifecycle_workflow", "source": "scope" }]
    });
    assert_rejects_schema("proof-lifecycle.json", &payload);
}

fn write_contract(dir: &TempDir, contents: &str) {
    fs::write(dir.path().join("ota.yaml"), contents).expect("contract should be written");
}

fn write_workspace_contract(
    dir: &TempDir,
    workspace_contents: &str,
    repo_rel_path: &str,
    repo_contract_contents: &str,
) {
    let repo_dir = dir.path().join(repo_rel_path);
    fs::create_dir_all(&repo_dir).expect("repo dir should be created");
    fs::write(dir.path().join("ota.workspace.yaml"), workspace_contents)
        .expect("workspace contract should be written");
    fs::write(repo_dir.join("ota.yaml"), repo_contract_contents)
        .expect("repo contract should be written");
}

#[test]
fn execution_topology_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: schema-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
surfaces:
  backend:
    kind: http
    port: 5678
    path: /
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 10s
tasks:
  dev:
    context: host
    run: npx --yes n8n
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - backend
    exposes:
      - surface: backend
"#,
    );

    let json = run_ota(
        &[
            "execution",
            "topology",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("execution-topology.json", &json);
}

#[test]
fn version_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    let json = run_ota(&["--version", "--json"], fixture.path());

    assert_matches_schema("version.json", &json);
    assert_eq!(json["ok"], true);
}

#[test]
fn refusal_canary_json_output_matches_published_schema_for_expected_and_missing_refusals() {
    let refused = TempDir::new().expect("refused fixture");
    write_contract(
        &refused,
        r#"
version: 1
project:
  name: refusal-canary-refused
tasks:
  publish:
    command:
      exe: sh
      args: ["-c", "exit 99"]
agent:
  refusal_canaries:
    - task: publish
"#,
    );
    let refused_json = run_ota(
        &["run", "--agent", "--expect-refusal", "--json", "publish"],
        refused.path(),
    );
    assert_matches_schema("refusal-canary.json", &refused_json);
    assert_eq!(refused_json["status"], "refused_as_expected");
    assert_eq!(refused_json["receipt"]["ok"], false);

    let rich_receipt = TempDir::new().expect("rich receipt fixture");
    write_contract(
        &rich_receipt,
        r#"
version: 1
project:
  name: refusal-canary-rich-receipt
env:
  vars:
    APP_MODE:
      default: test
  sources:
    - kind: dotenv
      path: .env
toolchains:
  ruby:
    version: "3.3"
tasks:
  install:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: bundler
        cwd: .
        path: vendor/bundle
    requirements:
      toolchains: [ruby]
    effects:
      writes: [vendor/bundle]
      network: true
      network_kind: dependency_hydration
    safe_for_agent: true
  hydrate:images:
    prepare:
      kind: dependency_hydration
      medium: container_images
      source:
        kind: docker_compose
        cwd: compose
        files: [docker-compose.base.yml, docker-compose.dev.yml]
        env_files: [.env.compose]
      targets: [web]
    requirements:
      tools:
        docker: "*"
    effects:
      network: true
      network_kind: container_image_hydration
    safe_for_agent: true
  publish:
    command:
      exe: sh
      args: ["-c", "exit 99"]
    depends_on: [install, hydrate:images]
agent:
  safe_tasks: [install, hydrate:images]
  refusal_canaries:
    - task: publish
"#,
    );
    fs::write(rich_receipt.path().join(".env"), "APP_MODE=test\n")
        .expect("dotenv fixture should be written");
    let rich_receipt_json = run_ota(
        &["run", "--agent", "--expect-refusal", "--json", "publish"],
        rich_receipt.path(),
    );
    assert_matches_schema("refusal-canary.json", &rich_receipt_json);
    assert!(
        rich_receipt_json["receipt"]["dependency_steps"]
            .as_array()
            .expect("dependency steps")
            .iter()
            .any(|step| step.get("prepare").is_some())
    );
    assert!(
        rich_receipt_json["receipt"]["dependency_steps"]
            .as_array()
            .expect("dependency steps")
            .iter()
            .any(|step| {
                step["prepare"]["source_kind"] == "docker_compose"
                    && step["prepare"]["files"].as_array().is_some()
                    && step["prepare"]["env_files"].as_array().is_some()
            }),
        "compose hydration summary should retain declared compose file and env-file truth"
    );
    assert!(
        rich_receipt_json["receipt"]["env_sources"]
            .as_array()
            .expect("environment sources")
            .iter()
            .any(|source| source.get("source_kind").is_some())
    );
    let mut invalid_source_status = rich_receipt_json.clone();
    invalid_source_status["receipt"]["env_sources"][0]["source_status"] =
        Value::String("not_a_runner_status".to_string());
    assert_rejects_schema("refusal-canary.json", &invalid_source_status);

    let admitted = TempDir::new().expect("admitted fixture");
    write_contract(
        &admitted,
        r#"
version: 1
project:
  name: refusal-canary-admitted
tasks:
  verify:
    safe_for_agent: true
    command:
      exe: sh
      args: ["-c", "exit 99"]
agent:
  safe_tasks: [verify]
  refusal_canaries:
    - task: verify
"#,
    );
    let admitted_json = run_ota_failure_stdout_json(
        &["run", "--agent", "--expect-refusal", "--json", "verify"],
        admitted.path(),
    );
    assert_matches_schema("refusal-canary.json", &admitted_json);
    assert_eq!(admitted_json["status"], "refusal_not_observed");
    assert_eq!(admitted_json["canary"]["execution_started"], false);

    let policy_refused = TempDir::new().expect("policy-refused fixture");
    write_contract(
        &policy_refused,
        r#"
version: 1
project:
  name: refusal-canary-policy-refused
tasks:
  verify:
    safe_for_agent: true
    command:
      exe: sh
      args: ["-c", "exit 99"]
agent:
  safe_tasks: [verify]
  refusal_canaries:
    - task: verify
"#,
    );
    fs::create_dir_all(policy_refused.path().join(".ota")).expect("policy directory");
    fs::write(
        policy_refused.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  agent:
    claim_assurance:
      agent_safety:
        minimum_status: supported
        on_insufficient: deny
"#,
    )
    .expect("policy should be written");
    let policy_refused_json = run_ota_failure_stdout_json(
        &["run", "--agent", "--expect-refusal", "--json", "verify"],
        policy_refused.path(),
    );
    assert_matches_schema("refusal-canary.json", &policy_refused_json);
    assert_eq!(policy_refused_json["status"], "wrong_refusal_boundary");
}

#[test]
fn github_projection_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: github-projection-fixture
tasks:
  verify:
    run: echo verify
  publish:
    run: echo publish
workflows:
  default: verify
  verify:
    intent: ci_verification
    run:
      task: verify
  release:
    run:
      task: publish
agent:
  safe_tasks:
    - verify
  refusal_canaries:
    - task: publish
    - workflow: release
"#,
    );
    let output = ".github/workflows/ota-governance.yml";
    let caller = ".github/workflows/ci.yml";
    let canonical = run_ota(
        &[
            "ci",
            "projection",
            "--json",
            "--workflow",
            "verify",
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("ci-projection.json", &canonical);
    assert_eq!(canonical["projection"]["mode"], "native");
    assert_eq!(
        canonical["projection"]["refusal_canaries"],
        serde_json::json!([
            {
                "kind": "task",
                "target": "publish",
                "merge_check_id": "ota.refusal-canary.task.publish"
            },
            {
                "kind": "workflow",
                "target": "release",
                "merge_check_id": "ota.refusal-canary.workflow.release"
            }
        ])
    );
    let resolved_default = run_ota(
        &[
            "ci",
            "projection",
            "--json",
            "--workflow",
            "verify",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("ci-projection.json", &resolved_default);
    assert_eq!(resolved_default["projection"]["mode"], "native");
    assert_eq!(
        resolved_default["projection"]["identity"],
        canonical["projection"]["identity"]
    );
    let render = run_ota(
        &[
            "ci",
            "github",
            "render",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &render);
    assert_eq!(
        render["projection"]["projection"]["identity"],
        canonical["projection"]["identity"]
    );
    assert_eq!(
        render["projection"]["provider_checks"],
        serde_json::json!([
            {
                "merge_check_id": "ota.verify.verify",
                "provider_check_name": "ota.verify.verify (linux/native)"
            },
            {
                "merge_check_id": "ota.refusal-canary.task.publish",
                "provider_check_name": "ota.refusal-canary.task.publish (linux/native)"
            },
            {
                "merge_check_id": "ota.refusal-canary.workflow.release",
                "provider_check_name": "ota.refusal-canary.workflow.release (linux/native)"
            }
        ])
    );
    let identity = render["projection"]["projection"]["identity"]
        .as_str()
        .expect("projection identity");
    let caller_path = fixture.path().join(caller);
    fs::create_dir_all(caller_path.parent().expect("caller parent")).expect("caller directory");
    fs::write(
        &caller_path,
        format!(
            "jobs:\n  ota:\n    uses: ./.github/workflows/ota-governance.yml\n    with:\n      ota_projection_identity: {identity}\n      ota_target_os: linux\n"
        ),
    )
    .expect("caller workflow");

    let sync = run_ota(
        &[
            "ci",
            "github",
            "sync",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--caller",
            caller,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &sync);
    assert_eq!(sync["mutated"], true);
    let repeated_sync = run_ota(
        &[
            "ci",
            "github",
            "sync",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--caller",
            caller,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &repeated_sync);
    assert_eq!(repeated_sync["mutated"], false);
    let check = run_ota(
        &[
            "ci",
            "github",
            "check",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--caller",
            caller,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &check);
    assert_eq!(check["mutated"], false);
    assert_eq!(sync["binding_identity"], check["binding_identity"]);

    fs::write(fixture.path().join(output), "name: externally-owned\n").expect("tamper output");
    let rejected = run_ota_failure_stdout_json(
        &[
            "ci",
            "github",
            "sync",
            "--json",
            "--workflow",
            "verify",
            "--output",
            output,
            "--caller",
            caller,
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("github-projection.json", &rejected);
    assert_eq!(rejected["code"], "managed_output_unowned");
}

#[test]
fn execution_plan_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: execution-demo
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: container
      lifecycle: ephemeral
      container:
        image: rust:1.94-bookworm
tasks:
  setup:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: dotnet_restore
        cwd: .
        config_file: NuGet.Config
        sources:
          - https://api.nuget.org/v3/index.json
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
"#,
    );

    let json = run_ota(
        &[
            "execution",
            "plan",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("execution.json", &json);
}

#[test]
fn assist_wire_setup_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: assist-demo
"#,
    );

    let json = run_ota(
        &[
            "assist",
            "wire-setup",
            "--json",
            "--copy-from",
            ".env.example",
            "--copy-to",
            ".env",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("assist-wire-setup.json", &json);
}

#[test]
fn proof_runtime_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: proof-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo setup-ready
    effects:
      network: true
      external_state:
        - remote_api
  live:
    context: host
    run: echo live-ready
    effects:
      network: true
      network_kind: integration_test
      external_state:
        - remote_api
  unrelated:
    context: host
    run: echo unrelated-ready
    effects:
      network: true
      network_kind: integration_test
      external_state:
        - unrelated_api
workflows:
  default: app
  app:
    setup:
      task: setup
  live:
    run:
      task: live
  unrelated:
    run:
      task: unrelated
"#,
    );

    let json = run_ota(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "app",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["proof_verdict"], "passed_with_unproven_boundaries");
    assert_eq!(json["phase"], "readiness");
    assert_eq!(json["proof_scope"]["kind"], "runtime_path");
    assert_eq!(json["proof_scope"]["proof_class"], "slice_proof");
    assert_eq!(json["proof_scope"]["workflow"], "app");
    assert_eq!(json["proof_scope"]["task"], "setup");
    assert_eq!(json["execution_boundary"]["schema_version"], 1);
    assert_eq!(json["execution_boundary"]["target_freshness"], "unknown");
    assert_eq!(
        json["execution_boundary"]["asserted_target_closure"],
        serde_json::json!([])
    );
    assert_eq!(
        json["not_proved"][0]["kind"],
        "external_network_path_not_proved"
    );
    assert_eq!(
        json["not_proved"][1]["kind"],
        "functional_runtime_not_proved"
    );
    assert_eq!(
        json["not_proved"][0]["declared_by_workflows"],
        serde_json::json!(["live"])
    );
    assert_eq!(json["not_proved"][0]["source"], "contract_lane");
    assert_eq!(
        json["not_proved"][2]["kind"],
        "broader_repo_completion_not_proved"
    );
    assert_eq!(json["not_proved"][2]["source"], "proof_scope");

    // The proof carrier must validate the stronger V11.11 evidence shape as well as the
    // ordinary narrow-proof result above.
    let mut seam_proof = json.clone();
    seam_proof["dependency_evidence"] = serde_json::json!([
        {
            "dependency_id": "service:postgres",
            "proof_obligation_id": "proof:postgres-round-trip",
            "level": "fault_tested",
            "observation": {
                "origin": "round_trip_effect",
                "evidence_class": "attested"
            },
            "negative_control": {
                "evidence_class": "derived",
                "status": "validated",
                "same_obligation": true,
                "failure_mode": "expected_missing_effect",
                "failure_attestation_digest": "sha256:control"
            }
        }
    ]);
    seam_proof["seam_observations"] = serde_json::json!([
        {
            "id": "proof:postgres-round-trip",
            "dependency_id": "service:postgres",
            "producer_task": "app",
            "transaction_id": "transaction-1",
            "observer_task": "observe-postgres",
            "marker_env": "OTA_SEAM_MARKER",
            "outcome": "observed",
            "proof_scope_ref": "workflow:app",
            "evidence_class": "attested",
            "attestation_digest": "sha256:attestation"
        }
    ]);
    seam_proof["negative_control"] = serde_json::json!({
        "id": "postgres-down",
        "dependency_id": "service:postgres",
        "obligation_id": "proof:postgres-round-trip",
        "transaction_id": "transaction-1",
        "control_task": "verify-with-postgres-down",
        "intervention": { "kind": "service_unavailable", "id": "postgres" },
        "expected_failure": "round_trip_missing",
        "outcome": "expected_obligation_failed",
        "status": "validated",
        "failure_mode": "expected_missing_effect",
        "proof_scope_ref": "workflow:app",
        "evidence_class": "attested",
        "failure_attestation_digest": "sha256:control"
    });
    seam_proof["not_proved"] = serde_json::json!([
        {
            "kind": "dependency_output_shaping_not_proved",
            "relative_to": "runtime_path",
            "source": "contract_lane",
            "dependency_id": "service:postgres",
            "proof_obligation_id": "proof:postgres-round-trip",
            "reason": "seam_causality_does_not_prove_broader_output_shaping"
        },
        {
            "kind": "broader_repo_completion_not_proved",
            "relative_to": "runtime_path",
            "source": "proof_scope"
        }
    ]);
    assert_matches_schema("proof-runtime.json", &seam_proof);

    let up_log = fixture
        .path()
        .join(".ota")
        .join("proof")
        .join("app")
        .join("up.log");
    let up_log_contents = fs::read_to_string(&up_log).expect("proof up log should be written");
    assert!(
        up_log_contents.contains("setup-ready"),
        "expected captured phase output in up.log, got:\n{up_log_contents}"
    );
}

#[test]
fn proof_runtime_failed_json_output_includes_failure_class() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: proof-failure-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
checks:
  - name: required-tool
    kind: precondition
    severity: error
    run: missing-proof-runtime-check
    timeout: 50
tasks:
  setup:
    context: host
    command:
      exe: echo
      args:
        - setup-ready
    requirements:
      checks:
        - required-tool
workflows:
  default: app
  app:
    setup:
      task: setup
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "app",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["ok"], false);
    assert_eq!(json["proof_verdict"], "failed");
    assert_eq!(json["failure_class"], "precondition_blocked");
    assert_eq!(json["proof_scope"]["kind"], "runtime_path");
    assert_eq!(
        json["not_proved"][0]["kind"],
        "functional_runtime_not_proved"
    );
}

#[test]
fn proof_runtime_replay_policy_refusal_precedes_artifacts_and_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: proof-policy-refusal
services:
  database:
    manager:
      kind: compose
      name: proof-policy-refusal
      file: compose.yaml
      service: database
tasks:
  setup:
    requires_services: [database]
    command:
      exe: sh
      args: ["-c", "touch task-ran"]
  observe-database:
    requires_services: [database]
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
        expected_identity: sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff
    command:
      exe: sh
      args: ["-c", "touch observer-ran"]
workflows:
  default: app
  app:
    setup:
      task: setup
    proof:
      seam_observations:
        - id: database-marker
          dependency: database
          producer_task: setup
          task: observe-database
          marker_env: OTA_PROOF_DATABASE_MARKER
"#,
    );
    fs::write(fixture.path().join("fixture.txt"), "frozen").expect("fixture input");
    fs::write(
        fixture.path().join("compose.yaml"),
        "services:\n  database:\n    image: postgres:17\n",
    )
    .expect("compose file");
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      workflows:
        app:
          on_insufficient: review
"#,
    )
    .expect("policy");

    let json = run_ota_json_output(
        &[
            "proof",
            "runtime",
            "--json",
            "--workflow",
            "app",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("proof-runtime.json", &json);
    assert_eq!(json["code"], "replay_input_identity_mismatch");
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["preflight"]["kind"], "replay_input_identity_mismatch");
    assert_eq!(
        json["replay_input_policy"]["decision"], "deny",
        "a declared pin mismatch must remain an unconditional denial"
    );
    assert_eq!(
        json["replay_input_policy"]["applicable_rules"][0]["closure_tasks"],
        serde_json::json!(["observe-database", "setup"])
    );
    assert!(!fixture.path().join("task-ran").exists());
    assert!(!fixture.path().join("observer-ran").exists());
    assert!(
        !fixture.path().join(".ota/proof").exists(),
        "proof refusal must precede parent artifact creation"
    );
}

#[test]
fn proof_lifecycle_replay_policy_refusal_covers_assertion_closure() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: lifecycle-policy-refusal
services:
  database:
    manager:
      kind: compose
      name: lifecycle-policy-refusal
      file: compose.yaml
      service: database
    lifecycle:
      teardown_assertion: manager_inactive
tasks:
  build:
    command:
      exe: sh
      args: ["-c", "touch build-ran"]
  assert-database:
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
    command:
      exe: sh
      args: ["-c", "touch assertion-ran"]
workflows:
  default: smoke
  smoke:
    run:
      task: build
    proof:
      lifecycle:
        services: [database]
        assertion:
          task: assert-database
"#,
    );
    fs::write(fixture.path().join("fixture.txt"), "frozen").expect("fixture input");
    fs::write(
        fixture.path().join("compose.yaml"),
        "services:\n  database:\n    image: postgres:17\n",
    )
    .expect("compose file");
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      workflows:
        smoke:
          on_insufficient: deny
"#,
    )
    .expect("policy");

    let json = run_ota_json_output(
        &[
            "proof",
            "lifecycle",
            "--json",
            "--workflow",
            "smoke",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("proof-lifecycle.json", &json);
    assert_eq!(json["code"], "replay_input_policy_deny");
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["replay_input_policy"]["decision"], "deny");
    assert_eq!(
        json["replay_input_policy"]["applicable_rules"][0]["closure_tasks"],
        serde_json::json!(["assert-database", "build"])
    );
    assert!(!fixture.path().join("build-ran").exists());
    assert!(!fixture.path().join("assertion-ran").exists());
}

#[test]
fn tasks_json_output_with_copy_if_missing_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  setup:
    action:
      kind: copy_if_missing
      from: .env.example
      to: .env
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
}

#[test]
fn tasks_json_output_with_compose_volume_reset_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  postgres:reset:
    action:
      kind: reset_compose_service_volume
      service: postgres
      volume: app_postgres-data
      compose:
        files:
          - docker-compose.yml
        project_name: app
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(json["tasks"][0]["name"], "postgres:reset");
    assert_eq!(json["tasks"][0]["kind"], "reset_compose_service_volume");
    assert_eq!(
        json["tasks"][0]["action"]["kind"],
        "reset_compose_service_volume"
    );
    assert_eq!(json["tasks"][0]["action"]["from"], "postgres");
    assert_eq!(json["tasks"][0]["action"]["to"], "app_postgres-data");
}

#[test]
fn tasks_json_output_with_container_network_action_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  integration:network:
    action:
      kind: ensure_container_network
      name: task-demo-integration
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(
        json["tasks"][0]["action"]["kind"],
        "ensure_container_network"
    );
    assert_eq!(json["tasks"][0]["action"]["from"], "docker");
    assert_eq!(json["tasks"][0]["action"]["to"], "task-demo-integration");
}

#[test]
fn tasks_json_output_with_container_image_build_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-demo
tasks:
  image:build:
    action:
      kind: build_container_image
      file: Dockerfile.integration
      context: integration
      tag: task-demo:integration
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(json["tasks"][0]["name"], "image:build");
    assert_eq!(json["tasks"][0]["kind"], "build_container_image");
    assert_eq!(json["tasks"][0]["action"]["kind"], "build_container_image");
    assert_eq!(json["tasks"][0]["action"]["from"], "Dockerfile.integration");
    assert_eq!(json["tasks"][0]["action"]["to"], "task-demo:integration");
    assert_eq!(json["tasks"][0]["action"]["context"], "integration");
}

#[test]
fn tasks_json_output_reports_command_shape() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-command-demo
tasks:
  test:
    command:
      exe: uv
      args:
        - run
        - pytest
      cwd: backend
      interaction: required
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    assert_eq!(json["tasks"][0]["name"], "test");
    assert_eq!(json["tasks"][0]["kind"], "command");
    assert_eq!(json["tasks"][0]["command"]["exe"], "uv");
    assert_eq!(json["tasks"][0]["command"]["args"][0], "run");
    assert_eq!(json["tasks"][0]["command"]["args"][1], "pytest");
    assert_eq!(json["tasks"][0]["command"]["cwd"], "backend");
    assert_eq!(json["tasks"][0]["command"]["interaction"], "required");
}

#[test]
fn tasks_json_output_reports_resolved_default_auto_interaction() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-command-interaction-demo
tasks:
  auto:
    command:
      exe: wrangler
      args: [login]
      interaction: auto
  captured:
    command:
      exe: cargo
      args: [test]
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &json);
    let auto = json["tasks"]
        .as_array()
        .expect("task array")
        .iter()
        .find(|task| task["name"] == "auto")
        .expect("auto task");
    let captured = json["tasks"]
        .as_array()
        .expect("task array")
        .iter()
        .find(|task| task["name"] == "captured")
        .expect("captured task");
    assert_eq!(auto["command"]["interaction"], "auto");
    assert_eq!(captured["command"]["interaction"], "auto");
}

#[test]
fn run_dry_run_json_reports_invocation_interaction_resolution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-command-interaction-preview
tasks:
  login:
    command:
      exe: sh
      args: [-c, "echo login"]
"#,
    );

    let json = run_ota(
        &[
            "run",
            "login",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_eq!(json["interaction"]["posture"], "auto");
    assert_eq!(json["interaction"]["resolution"], "piped");
    assert_eq!(json["interaction"]["terminal_available"], false);
}

#[test]
fn tasks_json_output_reports_prepare_sequence_and_aggregate_shapes() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-prepare-demo
toolchains:
  node:
    version: "22"
    package_managers:
      pnpm: "10"
  python:
    version: "3.12"
tasks:
  setup:
    description: Prepare mixed dependencies
    env:
      OTA_ENV: local
    inputs:
      profile:
        required: true
        default: dev
        allowed:
          - dev
          - ci
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
            frozen_lockfile: true
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: api
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
  verify:
    aggregate:
      tasks:
        - setup
"#,
    );

    let json = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_eq!(json["tasks"][0]["name"], "setup");
    assert_eq!(json["tasks"][0]["kind"], "sequence");
    assert_eq!(json["tasks"][0]["prepare"]["kind"], "sequence");
    assert_eq!(
        json["tasks"][0]["prepare"]["steps"][0]["kind"],
        "dependency_hydration"
    );
    assert_eq!(json["tasks"][1]["name"], "verify");
    assert_eq!(json["tasks"][1]["kind"], "aggregate");
    assert_eq!(json["tasks"][1]["aggregate"]["tasks"][0], "setup");
}

#[test]
fn json_validate_accepts_recursive_tasks_schema_payload() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: task-prepare-demo
toolchains:
  node:
    version: "22"
    package_managers:
      pnpm: "10"
  python:
    version: "3.12"
tasks:
  setup:
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: api
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
"#,
    );

    let payload = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    let payload_path = fixture.path().join("tasks.json");
    fs::write(
        &payload_path,
        serde_json::to_vec_pretty(&payload).expect("payload should serialize"),
    )
    .expect("payload should write");

    let stdout = run_ota_success_text(
        &[
            "json",
            "validate",
            "--schema",
            "tasks.json",
            "--input",
            payload_path.to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert!(stdout.contains("validated"), "{stdout}");
}

#[test]
fn services_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: services-demo
execution:
  default_context: app
  contexts:
    app:
      backend: native
services:
  postgres:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
    endpoints:
      app:
        address: 127.0.0.1
        port: 5432
    healthcheck: pg_isready -h 127.0.0.1 -p 5432
"#,
    );

    let json = run_ota(
        &["services", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("services.json", &json);
}

#[test]
fn validate_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: validate-demo
tasks:
  test:
    run: echo ok
"#,
    );

    let json = run_ota(
        &["validate", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("validate.json", &json);
}

#[test]
fn env_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: env-demo
env:
  vars:
    OTA_TEST_SHARED:
      required: true
      default: workspace-policy
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    run: echo ok
"#,
    );

    let json = run_ota(
        &["env", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("env.json", &json);
}

#[test]
fn doctor_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: doctor-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
env:
  vars:
    OTA_TEST_SHARED:
      required: true
      default: workspace-policy
tasks:
  setup:
    context: host
    run: echo ready
agent:
  default_task: setup
  safe_tasks:
    - setup
"#,
    );

    let json = run_ota(
        &["doctor", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("doctor.json", &json);
}

#[test]
fn replay_input_identity_policy_matches_doctor_preview_receipt_and_projection_schemas() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: replay-input-policy-schema-fixture
tasks:
  verify:
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
    command:
      exe: sh
      args: ["-c", "true"]
agent:
  default_task: verify
"#,
    );
    fs::write(fixture.path().join("fixture.txt"), "frozen").expect("fixture input");
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      tasks:
        verify:
          on_insufficient: deny
"#,
    )
    .expect("policy");

    let doctor = run_ota_json_output(&["doctor", "--json"], fixture.path());
    assert_matches_schema("doctor.json", &doctor);
    assert_eq!(doctor["replay_input_policy"]["decision"], "deny");
    let mut unavailable_doctor = doctor.clone();
    let unavailable_input = &mut unavailable_doctor["replay_input_policy"]["inputs"][0];
    unavailable_input["status"] = serde_json::json!("observation_unavailable");
    unavailable_input
        .as_object_mut()
        .expect("policy input should be an object")
        .remove("observed_identity");
    unavailable_input["error"] =
        serde_json::json!("replay input was not captured by the command preflight");
    assert_matches_schema("doctor.json", &unavailable_doctor);

    let preview = run_ota_json_output(&["run", "verify", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &preview);
    assert_eq!(preview["replay_input_policy"]["decision"], "deny");
    assert_eq!(preview["execution_started"], false);

    let refusal = run_ota_json_output(&["up", "--json", "--receipt"], fixture.path());
    assert_matches_schema("up.json", &refusal);
    assert_eq!(
        refusal["receipt"]["replay_input_policy"]["decision"],
        "deny"
    );
    assert_eq!(
        refusal["receipt"]["failure_origin"],
        "replay_input_policy_deny"
    );
    assert_eq!(refusal["receipt"]["status"], "blocked");

    fs::remove_file(fixture.path().join("fixture.txt")).expect("remove replay input");
    let missing_preview =
        run_ota_json_output(&["run", "verify", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &missing_preview);
    assert_eq!(missing_preview["execution_started"], false);
    assert_eq!(
        missing_preview["replay_input_policy"]["inputs"][0]["status"],
        "unpinned_unreadable"
    );
    fs::write(fixture.path().join("fixture.txt"), "frozen").expect("restore fixture input");

    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        "policies:\n  replay_inputs: [invalid\n",
    )
    .expect("invalid policy");
    let unavailable_policy_preview =
        run_ota_json_output(&["run", "verify", "--dry-run", "--json"], fixture.path());
    assert_matches_schema("run-preview.json", &unavailable_policy_preview);
    assert_eq!(
        unavailable_policy_preview["code"],
        "replay_input_policy_unavailable"
    );
    assert_eq!(unavailable_policy_preview["execution_started"], false);
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      tasks:
        verify:
          on_insufficient: deny
"#,
    )
    .expect("restore policy");

    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: replay-input-policy-schema-fixture
tasks:
  verify:
    safe_for_agent: true
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
    command:
      exe: sh
      args: ["-c", "true"]
workflows:
  default: verify
  verify:
    intent: ci_verification
    run:
      task: verify
agent:
  safe_tasks: [verify]
"#,
    );
    let projection = run_ota_json_output(
        &[
            "ci",
            "projection",
            "--json",
            "--workflow",
            "verify",
            "--mode",
            "native",
            "--target-os",
            "linux",
        ],
        fixture.path(),
    );
    assert_matches_schema("ci-projection.json", &projection);
    assert_eq!(projection["code"], "replay_input_policy_deny");
    assert_eq!(
        projection["projection"]["governance"]["replay_input_policy"]["selected_closure"],
        serde_json::json!(["verify"])
    );
}

#[test]
fn aggregate_monorepo_doctor_carries_member_replay_input_policy() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: replay-policy-monorepo
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  verify:
    command:
      exe: sh
      args: ["-c", "true"]
agent:
  default_task: verify
"#,
    );
    for member in ["api", "web"] {
        let member_dir = fixture.path().join(member);
        fs::create_dir_all(&member_dir).expect("member directory");
        fs::write(
            member_dir.join("ota.yaml"),
            format!(
                r#"
version: 1
project:
  name: {member}
tasks:
  verify:
    replay_inputs:
      - id: fixture
        kind: static_file
        path: fixture.txt
    command:
      exe: sh
      args: ["-c", "true"]
"#
            ),
        )
        .expect("member contract");
        fs::write(member_dir.join("fixture.txt"), "frozen").expect("member replay input");
    }
    fs::create_dir_all(fixture.path().join(".ota")).expect("policy directory");
    fs::write(
        fixture.path().join(".ota/org-policy.yaml"),
        r#"
policies:
  replay_inputs:
    identity:
      tasks:
        verify:
          on_insufficient: deny
"#,
    )
    .expect("member policy");

    let json = run_ota_failure_stdout_json(
        &["doctor", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );

    assert_matches_schema("doctor.json", &json);
    let members = json["members"].as_array().expect("aggregate members");
    let api = members
        .iter()
        .find(|member| member["member"] == "api")
        .expect("api member");
    assert_eq!(api["replay_input_policy"]["decision"], "deny");
    assert_eq!(
        api["replay_input_policy"]["applicable_rules"][0]["closure_tasks"],
        serde_json::json!(["verify"])
    );
    let web = members
        .iter()
        .find(|member| member["member"] == "web")
        .expect("web member");
    assert_eq!(web["replay_input_policy"]["decision"], "deny");
    assert_eq!(
        web["replay_input_policy"]["applicable_rules"][0]["closure_tasks"],
        serde_json::json!(["verify"])
    );
}

#[test]
fn doctor_remote_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: doctor-remote-demo
tasks:
  test:
    run: cargo test
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "doctor",
            "--mode",
            "remote",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("doctor.json", &json);
    assert_eq!(json["mode"], "remote");
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        "OTA_REMOTE_MODE_NOT_CONFIGURED"
    );
}

#[test]
fn workspace_tasks_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: echo ready
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-tasks.json", &json);
}

#[test]
fn workspace_tasks_json_output_with_container_network_action_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
tasks:
  integration:network:
    action:
      kind: ensure_container_network
      name: web-integration
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-tasks.json", &json);
    assert_eq!(
        json["repos"][0]["tasks"][0]["action"]["kind"],
        "ensure_container_network"
    );
    assert_eq!(json["repos"][0]["tasks"][0]["action"]["from"], "docker");
    assert_eq!(
        json["repos"][0]["tasks"][0]["action"]["to"],
        "web-integration"
    );
}

#[test]
fn workspace_tasks_json_output_reports_prepare_sequence_shape() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
toolchains:
  node:
    version: "22"
    package_managers:
      pnpm: "10"
  python:
    version: "3.12"
tasks:
  setup:
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
            frozen_lockfile: true
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: api
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_eq!(json["repos"][0]["tasks"][0]["kind"], "sequence");
    assert_eq!(json["repos"][0]["tasks"][0]["prepare"]["kind"], "sequence");
}

#[test]
fn json_validate_accepts_recursive_workspace_tasks_schema_payload() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
toolchains:
  node:
    version: "22"
    package_managers:
      pnpm: "10"
  python:
    version: "3.12"
tasks:
  setup:
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: node_package_manager
            cwd: .
            manager: pnpm
            mode: install
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: uv
            cwd: api
    requirements:
      toolchains:
        - node
        - python
    effects:
      writes:
        - node_modules
        - .venv
      network: true
      network_kind: dependency_hydration
"#,
    );

    let payload = run_ota(
        &[
            "workspace",
            "tasks",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    let payload_path = fixture.path().join("workspace-tasks.json");
    fs::write(
        &payload_path,
        serde_json::to_vec_pretty(&payload).expect("payload should serialize"),
    )
    .expect("payload should write");

    let stdout = run_ota_success_text(
        &[
            "json",
            "validate",
            "--schema",
            "workspace-tasks.json",
            "--input",
            payload_path.to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert!(stdout.contains("validated"), "{stdout}");
}

#[test]
fn check_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: check-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo ready
"#,
    );

    let json = run_ota(
        &["check", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("check.json", &json);
}

#[test]
fn receipt_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: receipt-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo ready
"#,
    );

    let json = run_ota(
        &["receipt", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("receipt.json", &json);
}

#[test]
fn receipt_json_schema_accepts_execution_conflict_metadata() {
    let json = serde_json::json!({
        "ok": false,
        "path": "/abs/path/to/ota.yaml",
        "mode": "receipt",
        "summary": {
            "error_count": 1,
            "warn_count": 0,
            "info_count": 0,
            "step_count": 0
        },
        "receipt": {
            "ok": false,
            "path": "/abs/path/to/ota.yaml",
            "scope": "repo",
            "contract": "/abs/path/to/ota.yaml",
            "contract_identity": {
                "version": 1,
                "project": {
                    "name": "receipt-conflict"
                },
                "counts": {
                    "runtimes": 0,
                    "tools": 0,
                    "env": 0,
                    "services": 0,
                    "checks": 0,
                    "tasks": 1
                }
            },
            "status": "blocked",
            "blocked": [
                "execution_conflict:host_service",
                "execution_conflict:compose_project"
            ],
            "execution_conflict": {
                "reasons": [
                    "host_service",
                    "compose_project"
                ]
            },
            "steps": [],
            "summary": {
                "error_count": 1,
                "warn_count": 0,
                "info_count": 0,
                "step_count": 0
            }
        },
        "findings": []
    });

    assert_matches_schema("receipt.json", &json);
}

#[test]
fn receipt_json_schema_accepts_promoted_replay_baseline_authority() {
    let identity = "sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa";
    let json = serde_json::json!({
        "ok": true,
        "path": "/abs/path/to/ota.yaml",
        "mode": "receipt",
        "summary": {
            "error_count": 0,
            "warn_count": 0,
            "info_count": 0,
            "step_count": 1
        },
        "receipt": {
            "ok": true,
            "path": "/abs/path/to/ota.yaml",
            "scope": "repo",
            "contract": "/abs/path/to/ota.yaml",
            "contract_identity": {
                "version": 1,
                "project": { "name": "replay-baseline" },
                "counts": {
                    "runtimes": 0,
                    "tools": 0,
                    "env": 0,
                    "services": 0,
                    "checks": 0,
                    "tasks": 1
                }
            },
            "witnessed_observations": {
                "query_traces": [{
                    "id": "recorded_sql",
                    "source_path": "data/fixture.jsonl",
                    "source_identity": identity,
                    "evidence_class": "attested",
                    "records": [{
                        "subject": "total_revenue",
                        "run": 0,
                        "identity": identity
                    }],
                    "summary": {
                        "subjects": 1,
                        "records": 1,
                        "divergent_subjects": [{
                            "subject": "total_revenue",
                            "distinct_identities": 2
                        }]
                    }
                }],
                "replay_baseline_recordings": [{
                    "artifact": "recorded-baseline",
                    "producer": "record:live",
                    "execution_scope": "task:record:live",
                    "execution_mode": "container",
                    "execution_lifecycle": "ephemeral",
                    "attestation_identity": identity,
                    "attestation_path": ".ota/replay-baselines/recorded-baseline/attestation.json",
                    "evidence_class": "attested"
                }]
            },
            "evaluated_inputs": [{
                "id": "generated_artifact:recorded-baseline",
                "kind": "promoted_replay_baseline",
                "input_class": "promoted_replay_baseline",
                "identity": identity,
                "artifact_lineage": {
                    "producer": "record:live",
                    "paths": ["data/baseline.json"],
                    "replay_authority": {
                        "authority_manifest": "replay/recorded-baseline.ota.json",
                        "trust_root": "scm_review",
                        "selected_attestation_identity": identity,
                        "promotion_identity": identity,
                        "consumption": "verify_unchanged"
                    }
                }
            }],
            "steps": [],
            "summary": {
                "error_count": 0,
                "warn_count": 0,
                "info_count": 0,
                "step_count": 1
            }
        },
        "findings": []
    });

    assert_matches_schema("receipt.json", &json);
}

#[test]
fn up_dry_run_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: up-demo
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: dotnet_restore
        cwd: app
        config_file: NuGet.Config
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    intent: verification
    run:
      task: setup
"#,
    );
    fs::create_dir_all(fixture.path().join("app")).expect("create app directory");
    fs::write(
        fixture.path().join("app/NuGet.Config"),
        r#"<configuration>
  <packageSources>
    <clear />
    <add key="nuget.org" value="https://api.nuget.org/v3/index.json" />
  </packageSources>
</configuration>"#,
    )
    .expect("write NuGet config");

    let json = run_ota_json_output(
        &[
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    assert_eq!(json["execution_started"], false);
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["declared_hydration_provenance"]["source_posture"],
        "config_file"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["declared_hydration_provenance"]["config_file"],
        "NuGet.Config"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"]["source_identities"]
            [0]["name"],
        "nuget.org"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"]["source_identities"]
            [0]["url"],
        "https://api.nuget.org/v3/index.json"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"]["resolution"],
        "resolved"
    );
}

#[test]
fn up_dry_run_json_refuses_unenforceable_native_lifecycle_before_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: rejected-up-lifecycle-preview
tasks:
  setup:
    command:
      exe: true
    execution:
      default_mode: native
      modes:
        native: {}
workflows:
  default: verify
  verify:
    setup:
      task: setup
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "up",
            "--ephemeral",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("up.json", &json);
    assert_eq!(json["ok"], false);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["preview_status"], "BLOCKED");
    assert_eq!(
        json["blockers"][0]["code"],
        "OTA_EXECUTION_OPTION_UNSUPPORTED_LIFECYCLE"
    );
    assert_eq!(
        json["blockers"][0]["summary"],
        "Requested lifecycle is not supported by this execution mode"
    );
    assert_eq!(
        json["plan"]["actions"],
        serde_json::json!(["refuse unsupported execution option before task `setup` startup"])
    );
}

#[test]
fn up_dry_run_json_marks_missing_dotnet_config_provenance_unavailable() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: unavailable-dotnet-provenance
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: dotnet_restore
        cwd: .
        config_file: missing/NuGet.Config
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    intent: verification
    run:
      task: setup
"#,
    );
    let json = run_ota_json_output(
        &[
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    let provenance =
        &json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"];
    assert_eq!(provenance["resolution"], "unavailable");
    assert!(provenance["source_identities"].as_array().is_none());
    assert!(
        provenance["resolution_error"]
            .as_str()
            .is_some_and(|message| message.contains("missing/NuGet.Config"))
    );
}

#[test]
fn up_dry_run_json_marks_ambient_dotnet_source_provenance_unavailable() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: ambient-dotnet-provenance
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: dotnet_restore
        cwd: .
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    intent: verification
    run:
      task: setup
"#,
    );

    let json = run_ota_json_output(
        &[
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    let provenance =
        &json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"];
    assert_eq!(provenance["source_posture"], "ambient_default");
    assert_eq!(provenance["resolution"], "unavailable");
    assert!(provenance["source_identities"].as_array().is_none());
    assert!(
        provenance["resolution_error"]
            .as_str()
            .is_some_and(|message| message.contains("ambient"))
    );
}

#[test]
fn up_dry_run_json_resolves_nested_explicit_dotnet_sources_without_fabricating_names() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: nested-dotnet-provenance
toolchains:
  dotnet:
    version: "*"
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    prepare:
      kind: sequence
      steps:
        - kind: dependency_hydration
          medium: package_dependencies
          source:
            kind: dotnet_restore
            cwd: .
            sources:
              - https://packages.example.test/v3/index.json
    requirements:
      toolchains:
        - dotnet
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    intent: verification
    run:
      task: setup
"#,
    );

    let json = run_ota_json_output(
        &[
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    let provenance = &json["plan"]["dependency_steps"][0]["prepare"]["steps"][0]["resolved_hydration_provenance"];
    assert_eq!(provenance["resolution"], "resolved");
    assert_eq!(
        provenance["source_identities"][0]["url"],
        "https://packages.example.test/v3/index.json"
    );
    assert!(provenance["source_identities"][0]["name"].is_null());
}

#[test]
fn run_dry_run_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: run-preview-demo
tasks:
  ci:
    run: npm test
"#,
    );

    let json = run_ota(
        &[
            "run",
            "ci",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &json);
    assert!(
        json.get("interaction").is_none(),
        "non-command task bodies must not publish a fabricated interaction posture"
    );
}

#[test]
fn sandbox_run_preview_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    fs::create_dir(fixture.path().join("reports")).expect("sandbox writable path");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: sandbox-preview
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: debian:bookworm-slim
      platform: linux/amd64
tasks:
  verify:
    safe_for_agent: true
    command: { exe: bash, args: ["-c", "true"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
        writable_paths: [reports]
      network:
        default: deny
agent:
  safe_tasks: [verify]
"#,
    );

    let json = run_ota(
        &[
            "run",
            "verify",
            "--agent",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["sandbox_admission"]["decision"], "admitted");
    assert_eq!(
        json["sandbox_admission"]["canonical_policy"]["segments"][0]["execution_kind"],
        "command"
    );
}

#[test]
fn sandbox_up_preview_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    fs::create_dir(fixture.path().join("reports")).expect("sandbox writable path");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: sandbox-up-preview
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: debian:bookworm-slim
      platform: linux/amd64
tasks:
  verify:
    safe_for_agent: true
    command: { exe: bash, args: ["-c", "true"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
        writable_paths: [reports]
      network:
        default: deny
workflows:
  default: verify
  verify:
    run:
      task: verify
agent:
  safe_tasks: [verify]
"#,
    );

    let json = run_ota(
        &[
            "up",
            "--workflow",
            "verify",
            "--agent",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    assert_eq!(json["sandbox_admission"]["decision"], "admitted");
    assert_eq!(
        json["governance"]["preflight"]["sandbox_admission"]["decision"],
        "admitted"
    );
}

#[test]
fn run_dry_run_json_keeps_governance_on_the_admitted_lane_when_mode_is_rejected() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: rejected-mode-preview
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/test:latest
tasks:
  integration:down:
    action:
      kind: ensure_container_network
      name: integration
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "run",
            "integration:down",
            "--container",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["overrides"]["backend"], "container");
    assert_eq!(json["governance"]["default_mode"], "native");
    assert_eq!(
        json["governance"]["runnable_modes"],
        serde_json::json!([
            {"mode": "native", "default": true, "command": "ota run integration:down"}
        ])
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["why"],
        "task `integration:down` was requested with `--mode container`, but it only supports modes: native"
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        "OTA_EXECUTION_OPTION_UNSUPPORTED_MODE"
    );
}

#[test]
fn run_dry_run_json_refuses_unenforceable_native_lifecycle_before_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: rejected-lifecycle-preview
tasks:
  deploy:
    command:
      exe: true
    execution:
      default_mode: native
      modes:
        native: {}
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "run",
            "deploy",
            "--ephemeral",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["overrides"]["lifecycle"], "ephemeral");
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        "OTA_EXECUTION_OPTION_UNSUPPORTED_LIFECYCLE"
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["summary"],
        "Requested lifecycle is not supported by this execution mode"
    );
    assert_eq!(
        json["summary"]["primary_blocker"]["why"],
        "task `deploy` was requested with `--lifecycle ephemeral`, but `native` execution does not provide a managed lifecycle boundary"
    );
    assert_eq!(
        json["plan"]["actions"],
        serde_json::json!(["refuse unsupported execution option before task `deploy` startup"])
    );
}

#[test]
fn run_dry_run_json_classifies_other_execution_option_refusals_before_execution() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: rejected-execution-options-preview
tasks:
  verify:
    command:
      exe: true
    execution:
      default_mode: native
      modes:
        native: {}
"#,
    );

    for (arguments, expected_code, override_field, expected_value) in [
        (
            vec!["--host-port", "4000"],
            "OTA_EXECUTION_OPTION_UNSUPPORTED_HOST_PORT",
            "host_port",
            serde_json::json!(4000),
        ),
        (
            vec!["--memory", "2GiB"],
            "OTA_EXECUTION_OPTION_UNSUPPORTED_MEMORY",
            "container_memory_bytes",
            serde_json::json!(2_147_483_648_u64),
        ),
        (
            vec!["--skip-deps"],
            "OTA_EXECUTION_OPTION_UNSUPPORTED_SKIP_DEPS",
            "skip_deps",
            serde_json::json!(true),
        ),
    ] {
        let mut command = vec!["run", "verify"];
        command.extend(arguments);
        command.extend(["--dry-run", "--json", fixture.path().to_str().unwrap()]);
        let json = run_ota_failure_stdout_json(&command, fixture.path());

        assert_matches_schema("run-preview.json", &json);
        assert_eq!(json["execution_started"], false);
        assert_eq!(json["summary"]["primary_blocker"]["code"], expected_code);
        assert_eq!(json["overrides"][override_field], expected_value);
        assert_eq!(
            json["plan"]["actions"],
            serde_json::json!(["refuse unsupported execution option before task `verify` startup"])
        );
    }
}

#[test]
fn run_dry_run_json_admits_native_docker_compose_host_port_projection() {
    let fixture = TempDir::new().expect("fixture");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("fake Docker bin directory");
    let docker_path = if cfg!(windows) {
        bin_dir.join("docker.cmd")
    } else {
        bin_dir.join("docker")
    };
    fs::write(
        &docker_path,
        if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"--version\" echo Docker version 26.1.0\r\nif \"%1\"==\"compose\" if \"%2\"==\"version\" echo Docker Compose version v2.27.0\r\nexit /b 0\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then echo 'Docker version 26.1.0'; fi\nif [ \"$1\" = \"compose\" ] && [ \"$2\" = \"version\" ]; then echo 'Docker Compose version v2.27.0'; fi\nexit 0\n"
        },
    )
    .expect("fake Docker executable");
    #[cfg(unix)]
    {
        let mut permissions = fs::metadata(&docker_path)
            .expect("fake Docker metadata")
            .permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&docker_path, permissions).expect("fake Docker permissions");
    }
    let mut path_entries = vec![bin_dir];
    if let Some(existing) = env::var_os("PATH") {
        path_entries.extend(env::split_paths(&existing));
    }
    let joined_path = env::join_paths(path_entries).expect("join fake Docker PATH");

    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: native-compose-host-port-preview
tasks:
  dev:
    adapter_inputs:
      compose:
        files:
          - docker-compose.yml
    compose:
      kind: up
      detach: true
      services:
        - web
    requirements:
      tools:
        docker: "*"
    runtime:
      kind: service
      listeners:
        web:http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
              primary: true
              path: /
            publication:
              compose:
                service: web
"#,
    );
    fs::write(
        fixture.path().join("docker-compose.yml"),
        "services:\n  web:\n    image: nginx:alpine\n",
    )
    .expect("compose fixture");

    let json = run_ota_with_env(
        &[
            "run",
            "dev",
            "--host-port",
            "4000",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
        &[("PATH", joined_path.to_str().expect("UTF-8 test PATH"))],
        true,
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["ok"], true);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["overrides"]["host_port"], 4000);
    assert_eq!(json["preview_status"], "RUNNABLE");
}

#[test]
fn run_dry_run_json_refuses_native_compose_host_port_without_file_stack() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: native-compose-host-port-missing-stack
tasks:
  dev:
    compose:
      kind: up
      detach: true
      services:
        - web
    requirements:
      tools:
        docker: "*"
    runtime:
      kind: service
      listeners:
        web:http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
              primary: true
              path: /
            publication:
              compose:
                service: web
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "run",
            "dev",
            "--host-port",
            "4000",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["execution_started"], false);
    assert_eq!(json["overrides"]["host_port"], 4000);
    assert_eq!(
        json["summary"]["primary_blocker"]["code"],
        "OTA_EXECUTION_OPTION_UNSUPPORTED_HOST_PORT"
    );
    assert_eq!(
        json["plan"]["actions"],
        serde_json::json!(["refuse unsupported execution option before task `dev` startup"])
    );
}

#[test]
fn run_dry_run_json_derives_aggregate_governance_from_the_selected_closure() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: aggregate-governance-preview
tasks:
  integration:test:
    run: echo integration
    effects:
      network: true
      network_kind: integration_test
      external_state:
        - postgres
  verify:integration:
    aggregate:
      tasks:
        - integration:test
"#,
    );

    let json = run_ota(
        &[
            "run",
            "verify:integration",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );

    assert_matches_schema("run-preview.json", &json);
    assert_eq!(json["governance"]["network"], true);
    assert_eq!(json["governance"]["network_kind"], "integration_test");
    assert_eq!(
        json["governance"]["external_state"],
        serde_json::json!(["postgres"])
    );
    assert_eq!(
        json["governance"]["sandbox_policy"]["network"]["default"],
        "allow"
    );
    assert_eq!(
        json["governance"]["sandbox_policy"]["network"]["source"],
        "lane_effect_network"
    );
}

#[test]
fn task_and_run_preview_json_preserve_service_readiness_network_kind() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: service-readiness-preview
services:
  api:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: api
tasks:
  api:health:
    category: test
    run: curl --fail http://127.0.0.1:3000/health
    requires_services: [api]
    effects:
      network: true
      network_kind: service_readiness
"#,
    );

    let tasks = run_ota(
        &["tasks", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("tasks.json", &tasks);
    assert_eq!(
        tasks["tasks"][0]["effects"]["network_kind"],
        "service_readiness"
    );

    let preview = run_ota(
        &[
            "run",
            "api:health",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &preview);
    assert_eq!(preview["governance"]["network_kind"], "service_readiness");
}

#[test]
fn run_dry_run_json_output_reports_compose_volume_reset_action() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: run-preview-demo
tasks:
  postgres:reset:
    action:
      kind: reset_compose_service_volume
      service: postgres
      volume: app_postgres-data
      compose:
        files:
          - docker-compose.yml
        project_name: app
"#,
    );

    let json = run_ota(
        &[
            "run",
            "postgres:reset",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &json);
    assert_eq!(
        json["requested_task"]["kind"],
        "reset_compose_service_volume"
    );
    assert_eq!(
        json["requested_task"]["action"]["kind"],
        "reset_compose_service_volume"
    );
    assert_eq!(json["requested_task"]["action"]["from"], "postgres");
    assert_eq!(json["requested_task"]["action"]["to"], "app_postgres-data");
    assert_eq!(
        json["plan"]["actions"][0],
        "would run task action `reset_compose_service_volume` on the host"
    );
}

#[test]
fn run_dry_run_blocked_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: blocked-run-preview
env:
  vars:
    SECRET_TOKEN:
      required: true
tasks:
  ci:
    run: echo ci
"#,
    );

    let json = run_ota_failure_stdout_json(
        &[
            "run",
            "ci",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &json);
}

#[test]
fn run_dry_run_member_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: mono
workspace:
  type: monorepo
  members:
    - api
    - web
tasks:
  ci:
    run: echo root
"#,
    );
    fs::create_dir_all(fixture.path().join("api")).expect("api dir");
    fs::create_dir_all(fixture.path().join("web")).expect("web dir");
    fs::write(
        fixture.path().join("api").join("ota.yaml"),
        r#"
version: 1
project:
  name: api
tasks:
  ci:
    run: echo api
"#,
    )
    .expect("api contract");
    fs::write(
        fixture.path().join("web").join("ota.yaml"),
        r#"
version: 1
project:
  name: web
tasks:
  ci:
    run: echo web
"#,
    )
    .expect("web contract");

    let json = run_ota(
        &[
            "run",
            "ci",
            "--dry-run",
            "--json",
            "--member",
            "api",
            "--member",
            "web",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("run-preview.json", &json);
}

#[test]
fn run_dry_run_unknown_task_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: unknown-task-preview
tasks:
  ci:
    run: echo ci
"#,
    );

    let json = run_ota_with_env(
        &[
            "run",
            "missing",
            "--dry-run",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("run-preview.json", &json);
}

#[test]
fn workspace_check_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  setup:
    context: host
    run: echo ready
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "check",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-check.json", &json);
}

#[test]
fn workspace_doctor_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
execution:
  default_context: host
  contexts:
    host:
      backend: native
env:
  vars:
    OTA_TEST_SHARED:
      required: true
      default: workspace-policy
tasks:
  setup:
    context: host
    run: echo ready
agent:
  default_task: setup
  safe_tasks:
    - setup
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "doctor",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-doctor.json", &json);
}

#[test]
fn workspace_up_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_workspace_contract(
        &fixture,
        r#"
version: 1
workspace:
  name: ota-dev
repos:
  web:
    path: apps/web
    required: true
"#,
        "apps/web",
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    run: echo ready
"#,
    );

    let json = run_ota(
        &[
            "workspace",
            "up",
            "--json",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("workspace-up.json", &json);
}

#[test]
fn clean_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-demo
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  check:
    run: echo ok
"#,
    );

    let json = run_ota(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_workspace_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-workspace
workspace:
  type: monorepo
  members:
    - api
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  check:
    run: echo root
"#,
    );
    let api_dir = fixture.path().join("api");
    fs::create_dir_all(&api_dir).expect("member dir");
    fs::write(
        api_dir.join("ota.yaml"),
        r#"
version: 1
project:
  name: api
execution:
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  check:
    run: echo api
"#,
    )
    .expect("member contract");

    let json = run_ota(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_stale_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    let empty_bin = fixture.path().join("bin");
    fs::create_dir_all(&empty_bin).expect("bin dir");

    let json = run_ota_with_env(
        &["clean", "--stale", "--json"],
        fixture.path(),
        &[("PATH", empty_bin.to_str().unwrap())],
        true,
    );
    assert_matches_schema("clean.json", &json);
}

#[cfg(unix)]
#[test]
fn clean_failure_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-failure
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/test:latest
        engines:
          - podman
      attachments:
        isolated_paths:
          - node_modules
tasks:
  check:
    context: app
    run: echo ok
"#,
    );
    fs::create_dir_all(fixture.path().join(".ota").join("state")).expect("state dir");
    fs::write(
        fixture
            .path()
            .join(".ota")
            .join("state")
            .join("ownership-id"),
        "repo-1",
    )
    .expect("ownership token");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir");
    let podman_path = bin_dir.join("podman");
    fs::write(
        &podman_path,
        r#"#!/bin/sh
if [ "$1" = "volume" ] && [ "$2" = "ls" ]; then
  echo "Cannot connect to Podman" >&2
  echo "Error: unable to connect to Podman socket: dial tcp 127.0.0.1:57990: connect: connection refused" >&2
  exit 125
fi
exit 0
"#,
    )
    .expect("fake podman");
    let mut permissions = fs::metadata(&podman_path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&podman_path, permissions).expect("permissions");

    let mut path_entries = vec![bin_dir.clone()];
    if let Some(existing) = env::var_os("PATH") {
        path_entries.extend(env::split_paths(&existing));
    }
    let joined_path = env::join_paths(path_entries).expect("join path");

    let json = run_ota_with_env(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
        &[("PATH", joined_path.to_str().unwrap())],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_generic_failure_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-generic-failure
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
tasks:
  dev:
    context: app
    runtime:
      kind: service
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port: {}
"#,
    );

    let json = run_ota_with_env(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_invalid_contract_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: clean-invalid-contract
execution:
  preferred: host
  default_context: host
  contexts:
    host:
      backend: native
tasks:
  check:
    run: echo ok
"#,
    );

    let json = run_ota_with_env(
        &["clean", "--json", fixture.path().to_str().unwrap()],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn clean_unresolved_target_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");

    let json = run_ota_with_env(
        &["clean", "--json", "missing-repo-target"],
        fixture.path(),
        &[],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[cfg(unix)]
#[test]
fn clean_stale_failure_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    let bin_dir = fixture.path().join("bin");
    fs::create_dir_all(&bin_dir).expect("bin dir");
    let podman_path = bin_dir.join("podman");
    fs::write(
        &podman_path,
        r#"#!/bin/sh
echo "Cannot connect to Podman" >&2
echo "Error: unable to connect to Podman socket: dial tcp 127.0.0.1:57990: connect: connection refused" >&2
exit 125
"#,
    )
    .expect("fake podman");
    let mut permissions = fs::metadata(&podman_path).expect("metadata").permissions();
    permissions.set_mode(0o755);
    fs::set_permissions(&podman_path, permissions).expect("permissions");

    let json = run_ota_with_env(
        &["clean", "--stale", "--json"],
        fixture.path(),
        &[("PATH", bin_dir.to_str().unwrap())],
        false,
    );
    assert_matches_schema("clean.json", &json);
}

#[test]
fn replay_baseline_record_and_promote_json_match_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: replay-baseline-json
artifacts:
  recorded:
    kind: replay_baseline
    producer: record
    paths: [data/baseline.txt]
    replay:
      authority_manifest: replay/recorded.ota.json
      consumption: read_only
tasks:
  record:
    action:
      kind: ensure_file
      path: data/baseline.txt
      value: recorded
  replay:
    action:
      kind: ensure_directory
      path: scratch
    requires_artifacts: [recorded]
agent:
  safe_tasks: [replay]
"#,
    )
    .expect("contract");
    for args in [
        vec!["init"],
        vec!["config", "user.email", "ota@example.com"],
        vec!["config", "user.name", "Ota Tests"],
        vec!["add", "ota.yaml"],
        vec!["commit", "-m", "baseline contract"],
    ] {
        Command::new("git")
            .args(args)
            .current_dir(fixture.path())
            .status()
            .expect("git command")
            .success()
            .then_some(())
            .expect("git command succeeds");
    }

    let recorded = run_ota(
        &[
            "baseline",
            "record",
            "--artifact",
            "recorded",
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("replay-baseline.json", &recorded);
    let attestation = recorded["attestation"].as_str().expect("attestation");
    let attestation_json: Value = serde_json::from_slice(
        &fs::read(fixture.path().join(attestation)).expect("recorded attestation"),
    )
    .expect("attestation json");
    assert_matches_schema("replay-baseline-authority.json", &attestation_json);
    let mut missing_boundary_graph = attestation_json.clone();
    missing_boundary_graph
        .as_object_mut()
        .expect("attestation object")
        .remove("execution_boundary_graph_identity");
    assert_rejects_schema("replay-baseline-authority.json", &missing_boundary_graph);
    let promoted = run_ota(
        &[
            "baseline",
            "promote",
            "--artifact",
            "recorded",
            "--attestation",
            attestation,
            "--json",
            ".",
        ],
        fixture.path(),
    );
    assert_matches_schema("replay-baseline.json", &promoted);
    let authority_manifest = promoted["authority_manifest"]
        .as_str()
        .expect("authority manifest");
    let authority_json: Value = serde_json::from_slice(
        &fs::read(fixture.path().join(authority_manifest)).expect("authority manifest file"),
    )
    .expect("authority manifest json");
    assert_matches_schema("replay-baseline-authority.json", &authority_json);
    let mut missing_attestation = authority_json.clone();
    missing_attestation
        .as_object_mut()
        .expect("authority manifest object")
        .remove("attestation");
    assert_rejects_schema("replay-baseline-authority.json", &missing_attestation);

    let failure = serde_json::json!({
        "ok": false,
        "code": "replay_baseline_operation_failed",
        "error": "recording refused before execution"
    });
    assert_matches_schema("replay-baseline.json", &failure);
}
