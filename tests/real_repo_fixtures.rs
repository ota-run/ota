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

use serde_json::Value;
use tempfile::TempDir;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("real")
        .join(name)
}

fn run_ota(args: &[&str]) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .output()
        .expect("ota command should execute")
}

fn copy_dir_recursive(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();

    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let from_path = entry.path();
        let to_path = to.join(entry.file_name());

        if from_path.is_dir() {
            copy_dir_recursive(&from_path, &to_path);
        } else {
            fs::create_dir_all(to_path.parent().unwrap()).unwrap();
            fs::copy(&from_path, &to_path).unwrap();
        }
    }
}

fn copied_fixture(name: &str) -> TempDir {
    let dir = TempDir::new().unwrap();
    copy_dir_recursive(&fixture_path(name), dir.path());
    dir
}

#[test]
fn init_reports_blank_mode_for_java_gradle_fixture() {
    let output = run_ota(&[
        "init",
        "--json",
        fixture_path("java-gradle").to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "blank");
    assert_eq!(json["config"]["project"]["name"], "java-gradle");
}

#[test]
fn init_reports_detected_mode_for_docker_heavy_fixture() {
    let output = run_ota(&[
        "init",
        "--json",
        fixture_path("docker-heavy-node").to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-containerized");
    assert_eq!(json["config"]["runtimes"]["node"], "20");
}

#[test]
fn detect_reports_precedence_on_ugly_polyglot_fixture() {
    let output = run_ota(&[
        "detect",
        "--json",
        "--dry-run",
        fixture_path("ugly-polyglot").to_str().unwrap(),
    ]);

    assert!(output.status.success());
    let json: Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["project"]["name"], "ota-platform");
    assert_eq!(json["config"]["runtimes"]["node"], "22");
    assert_eq!(json["config"]["runtimes"]["python"], "3.13.2");
    assert_eq!(json["config"]["tools"]["pnpm"], "10.5.0");
}

#[cfg(unix)]
#[test]
fn doctor_surfaces_real_repo_gap_on_java_fixture() {
    let fixture = copied_fixture("java-gradle");
    fs::write(
        fixture.path().join("ota.yaml"),
        r#"
version: 1
project:
  name: java-gradle
checks:
  - name: gradle-wrapper-present
    kind: health
    severity: warn
    run: test -f gradlew
tasks:
  test:
    run: ./gradlew test
"#
        .trim_start(),
    )
    .unwrap();

    let output = run_ota(&["doctor", fixture.path().to_str().unwrap()]);

    assert!(output.status.success());
    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(stdout.contains("DOCTOR"));
    assert!(stdout.contains("READY"));
    assert!(stdout.contains("WARN  Check failed: gradle-wrapper-present"));
}
