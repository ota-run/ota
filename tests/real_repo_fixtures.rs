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
use std::process::{Command, Output};

use serde_json::Value;
use tempfile::TempDir;

fn real_fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("real")
        .join(name)
}

fn run_ota(args: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .output()
        .expect("ota command should run")
}

fn stdout_json(output: &Output) -> Value {
    assert!(
        output.status.success(),
        "expected success, stderr was: {}",
        String::from_utf8_lossy(&output.stderr)
    );

    serde_json::from_slice(&output.stdout).expect("stdout should be valid JSON")
}

fn copy_fixture_to_temp(name: &str) -> TempDir {
    let temp = TempDir::new().expect("temp dir should be created");
    copy_dir_recursive(&real_fixture_path(name), temp.path());
    temp
}

fn copy_dir_recursive(src: &Path, dest: &Path) {
    fs::create_dir_all(dest).expect("destination directory should exist");

    for entry in fs::read_dir(src).expect("fixture directory should be readable") {
        let entry = entry.expect("fixture entry should be readable");
        let entry_path = entry.path();
        let target_path = dest.join(entry.file_name());
        let metadata = entry
            .metadata()
            .expect("fixture entry metadata should be readable");

        if metadata.is_dir() {
            copy_dir_recursive(&entry_path, &target_path);
        } else {
            fs::copy(&entry_path, &target_path).expect("fixture file should copy");
        }
    }
}

#[test]
fn init_json_reports_detected_mode_for_java_gradle_fixture() {
    let fixture = real_fixture_path("java-gradle");
    let output = run_ota(&["init", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-java-service");
    assert_eq!(json["config"]["runtimes"]["java"], "21");
    assert_eq!(json["config"]["tools"]["gradle"], "8.10.2");
    assert_eq!(json["config"]["tasks"]["build"]["run"], "./gradlew build");
}

#[test]
fn init_json_reports_detected_mode_for_java_maven_fixture() {
    let fixture = real_fixture_path("java-maven");
    let output = run_ota(&["init", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-maven-service");
    assert_eq!(json["config"]["runtimes"]["java"], "21");
    assert_eq!(json["config"]["tools"]["maven"], "*");
    assert_eq!(json["config"]["tasks"]["test"]["run"], "mvn test");
}

#[test]
fn init_json_reports_detected_mode_for_java_gradle_multimodule_fixture() {
    let fixture = real_fixture_path("java-gradle-multimodule");
    let output = run_ota(&["init", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-platform");
    assert_eq!(json["config"]["runtimes"]["java"], "21");
    assert_eq!(json["config"]["tools"]["gradle"], "8.11.1");
    assert_eq!(json["config"]["tasks"]["build"]["run"], "./gradlew build");
}

#[test]
fn init_json_reports_blank_mode_for_docker_legacy_fixture() {
    let fixture = real_fixture_path("docker-legacy");
    let output = run_ota(&["init", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "blank");
    assert_eq!(json["config"]["project"]["name"], "docker-legacy");
    assert_eq!(json["inferred"][0]["source"], "directory-name");
}

#[test]
fn detect_json_handles_docker_heavy_node_fixture() {
    let fixture = real_fixture_path("docker-heavy-node");
    let output = run_ota(&["detect", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["project"]["name"], "ota-containerized-web");
    assert_eq!(json["config"]["runtimes"]["node"], "22.3.0");
    assert_eq!(json["config"]["tools"]["pnpm"], "10.5.0");
    assert_eq!(json["config"]["tasks"]["dev"]["run"], "pnpm dev");
}

#[test]
fn detect_json_prefers_repo_specific_signals_in_node_conflict_fixture() {
    let fixture = real_fixture_path("node-conflict-monorepo");
    let output = run_ota(&["detect", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);
    let inferred = json["inferred"].as_array().unwrap();

    assert_eq!(json["ok"], true);
    assert_eq!(json["config"]["project"]["name"], "ota-monorepo");
    assert_eq!(json["config"]["runtimes"]["node"], "22.8.1");
    assert_eq!(json["config"]["tools"]["pnpm"], "10.7.0");
    assert_eq!(json["config"]["tasks"]["dev"]["run"], "pnpm dev");
    assert!(inferred.iter().any(|inference| {
        inference["field"] == "runtimes.node"
            && inference["source"] == ".nvmrc"
            && inference["value"] == "22.8.1"
    }));
    assert!(inferred.iter().any(|inference| {
        inference["field"] == "tools.pnpm"
            && inference["source"] == "package.json#packageManager"
            && inference["value"] == "10.7.0"
    }));
}

#[test]
fn detect_json_handles_ugly_polyglot_fixture() {
    let fixture = real_fixture_path("ugly-polyglot");
    let output = run_ota(&["detect", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);
    let inferred = json["inferred"].as_array().unwrap();

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["project"]["name"], "ota-polyglot-app");
    assert_eq!(json["config"]["runtimes"]["node"], "22");
    assert_eq!(json["config"]["runtimes"]["python"], "3.12.4");
    assert_eq!(json["config"]["runtimes"]["go"], "1.24.0");
    assert_eq!(json["config"]["tools"]["pnpm"], "10.6.0");
    assert_eq!(json["config"]["tasks"]["dev"]["run"], "pnpm dev");
    assert!(inferred.iter().any(|inference| {
        inference["field"] == "runtimes.node"
            && inference["source"] == ".nvmrc"
            && inference["value"] == "22"
    }));
}

#[cfg(unix)]
#[test]
fn doctor_json_reports_service_and_lifecycle_findings_in_polyglot_ops_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
execution:
  preferred: native
  lifecycle: ephemeral
services:
  postgres:
    required: true
    healthcheck: test -f .service-ready
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json =
        serde_json::from_slice::<Value>(&output.stdout).expect("stdout should be valid JSON");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["ok"], false);
    assert_eq!(json["findings"].as_array().unwrap().len(), 2);
    assert_eq!(json["findings"][0]["severity"], "error");
    assert_eq!(
        json["findings"][0]["summary"],
        "Service healthcheck failed: postgres"
    );
    assert_eq!(json["findings"][1]["severity"], "warn");
    assert_eq!(
        json["findings"][1]["summary"],
        "Ephemeral lifecycle is advisory only in V1"
    );
}

#[cfg(unix)]
#[test]
fn doctor_json_runs_warning_check_in_ugly_polyglot_fixture() {
    let fixture = copy_fixture_to_temp("ugly-polyglot");
    let contract = r#"
version: 1
project:
  name: ota-polyglot-app
checks:
  - name: docs-ops
    kind: health
    severity: warn
    run: test -f docs/ops.md
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["findings"].as_array().unwrap().len(), 1);
    assert_eq!(json["findings"][0]["severity"], "warn");
    assert_eq!(json["findings"][0]["summary"], "Check failed: docs-ops");
}
