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

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
#[cfg(unix)]
use std::{env, os::unix::fs::PermissionsExt};

use jsonschema::{Draft, JSONSchema};
use serde_json::Value;
use tempfile::TempDir;

fn run_ota(args: &[&str], cwd: &Path) -> Value {
    run_ota_with_env(args, cwd, &[], true)
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
fn execution_plan_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: execution-demo
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
    assert_eq!(json["phase"], "readiness");
    assert_eq!(json["proof_scope"]["kind"], "runtime_path");
    assert_eq!(json["proof_scope"]["proof_class"], "slice_proof");
    assert_eq!(json["proof_scope"]["workflow"], "app");
    assert_eq!(json["proof_scope"]["task"], "setup");
    assert_eq!(
        json["not_proved"][0]["kind"],
        "functional_runtime_not_proved"
    );
    assert_eq!(
        json["not_proved"][1]["kind"],
        "external_network_path_not_proved"
    );
    assert_eq!(
        json["not_proved"][1]["declared_by_workflows"],
        serde_json::json!(["live"])
    );
    assert_eq!(json["not_proved"][1]["source"], "contract_lane");
    assert_eq!(
        json["not_proved"][2]["kind"],
        "broader_repo_completion_not_proved"
    );
    assert_eq!(json["not_proved"][2]["source"], "proof_scope");
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
    assert_eq!(json["failure_class"], "precondition_blocked");
    assert_eq!(json["proof_scope"]["kind"], "runtime_path");
    assert_eq!(
        json["not_proved"][0]["kind"],
        "functional_runtime_not_proved"
    );
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
fn up_dry_run_json_output_matches_published_schema() {
    let fixture = TempDir::new().expect("fixture");
    write_contract(
        &fixture,
        r#"
version: 1
project:
  name: up-demo
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
            "up",
            "--json",
            "--dry-run",
            fixture.path().to_str().unwrap(),
        ],
        fixture.path(),
    );
    assert_matches_schema("up.json", &json);
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["declared_hydration_provenance"]["source_posture"],
        "explicit_sources"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["declared_hydration_provenance"]["config_file"],
        "NuGet.Config"
    );
    assert_eq!(
        json["plan"]["dependency_steps"][0]["prepare"]["resolved_hydration_provenance"]["sources"]
            [0],
        "https://api.nuget.org/v3/index.json"
    );
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
