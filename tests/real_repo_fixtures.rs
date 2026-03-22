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
fn init_json_reports_blank_mode_for_java_gradle_fixture() {
    let fixture = real_fixture_path("java-gradle");
    let output = run_ota(&["init", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "blank");
    assert_eq!(json["config"]["project"]["name"], "java-gradle");
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
