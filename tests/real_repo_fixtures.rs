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
    run_ota_with_env(args, [])
}

fn run_ota_in_dir(args: &[&str], cwd: &Path) -> Output {
    run_ota_with_env_in_dir(args, [], cwd)
}

fn run_ota_with_env<const N: usize>(args: &[&str], envs: [(&str, &str); N]) -> Output {
    run_ota_with_env_in_dir(args, envs, Path::new("."))
}

fn run_ota_with_env_in_dir<const N: usize>(
    args: &[&str],
    envs: [(&str, &str); N],
    cwd: &Path,
) -> Output {
    Command::new(env!("CARGO_BIN_EXE_ota"))
        .args(args)
        .envs(envs)
        .current_dir(cwd)
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

#[cfg(unix)]
#[test]
fn workspace_up_stream_includes_live_child_output() {
    let temp = TempDir::new().expect("temp dir should be created");
    let repo_dir = temp.path().join("apps").join("web");
    fs::create_dir_all(&repo_dir).unwrap();
    fs::write(
        repo_dir.join("ota.yaml"),
        r#"
version: 1
project:
  name: web
tasks:
  setup:
    script: |
      printf stream-out
      printf stream-err >&2
"#,
    )
    .unwrap();
    fs::write(
        temp.path().join("ota.workspace.yaml"),
        r#"
version: 1
workspace:
  name: ota-stream
repos:
  web:
    path: apps/web
    required: true
"#,
    )
    .unwrap();

    let output = run_ota(&["workspace", "up", "--stream", temp.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");
    assert!(stdout.contains("stream-out"));
    assert!(stdout.contains("WORKSPACE UP"));
    assert!(stderr.contains("stream-err"));
    assert!(stderr.contains("WORKSPACE RUN web"));
    assert!(stderr.contains("WORKSPACE READY web"));
}

#[cfg(unix)]
#[test]
fn validate_discovers_contract_from_current_directory_real_fixture() {
    let fixture = real_fixture_path("task-variant-app");
    let nested = fixture.join("apps").join("web");

    let output = run_ota_in_dir(&["validate"], &nested);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains(&format!("VALID {}", fixture.join("ota.yaml").display())));
}

#[test]
fn validate_uses_ota_file_override_real_fixture() {
    let fixture = real_fixture_path("task-variant-app");
    let temp = TempDir::new().expect("temp dir should be created");

    let output = run_ota_with_env_in_dir(
        &["validate"],
        [("OTA_FILE", fixture.join("ota.yaml").to_str().unwrap())],
        temp.path(),
    );
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert!(output.status.success());
    assert!(stdout.contains(&format!("VALID {}", fixture.join("ota.yaml").display())));
}

#[test]
fn tasks_json_reports_resolved_task_variant_on_real_fixture() {
    let fixture = real_fixture_path("task-variant-app");
    let output = run_ota(&["tasks", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);
    let tasks = json["tasks"].as_array().unwrap();
    let setup = tasks
        .iter()
        .find(|task| task["name"] == "setup")
        .expect("setup task should be listed");

    match std::env::consts::OS {
        "macos" => {
            assert_eq!(setup["run"], "sh ./scripts/setup-macos.sh");
            assert_eq!(setup["selected_variant_os"], "macos");
        }
        _ => {
            assert_eq!(setup["run"], "sh ./scripts/setup.sh");
            assert!(setup.get("selected_variant_os").is_none());
        }
    }

    assert_eq!(setup["variants"].as_array().unwrap().len(), 2);
}

#[cfg(unix)]
#[test]
fn run_executes_task_variant_from_nested_directory_real_fixture() {
    let fixture = copy_fixture_to_temp("task-variant-app");
    let nested = fixture.path().join("apps").join("web");

    let output = run_ota_in_dir(&["run", "setup"], &nested);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");
    assert!(stderr.contains("RUN setup"));

    let expected = match std::env::consts::OS {
        "macos" => "macos",
        _ => "default",
    };
    assert_eq!(
        fs::read_to_string(fixture.path().join("setup-output.txt"))
            .expect("setup output should exist"),
        expected
    );
}

fn rename_if_exists(root: &Path, from: &str, to: &str) {
    let from_path = root.join(from);
    if from_path.exists() {
        fs::rename(from_path, root.join(to)).expect("fixture file should rename");
    }
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
fn init_write_writes_high_confidence_contract_for_java_gradle_fixture() {
    let fixture = copy_fixture_to_temp("java-gradle");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for java gradle fixture");

    assert!(written.contains("name: ota-java-service"));
    assert!(written.contains("java: '21'"));
    assert!(written.contains("gradle: 8.10.2"));
    assert!(written.contains("run: ./gradlew build"));

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
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

#[cfg(unix)]
#[test]
fn init_write_writes_high_confidence_contract_for_java_maven_fixture() {
    let fixture = copy_fixture_to_temp("java-maven");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for java maven fixture");

    assert!(written.contains("name: ota-maven-service"));
    assert!(written.contains("java: '21'"));
    assert!(!written.contains("tools:"));
    assert!(!written.contains("tasks:"));

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
}

#[cfg(unix)]
#[test]
fn init_json_prefers_maven_wrapper_on_real_fixture() {
    let fixture = copy_fixture_to_temp("java-maven");
    fs::write(fixture.path().join("mvnw"), "#!/bin/sh\n")
        .expect("wrapper script should be written");
    fs::create_dir_all(fixture.path().join(".mvn").join("wrapper"))
        .expect("wrapper directory should be created");
    fs::write(
        fixture.path().join(".mvn").join("wrapper").join("maven-wrapper.properties"),
        "distributionUrl=https://repo.maven.apache.org/maven2/org/apache/maven/apache-maven/3.9.9/apache-maven-3.9.9-bin.zip\n",
    )
    .expect("wrapper properties should be written");

    let output = run_ota(&["init", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["config"]["tools"]["maven"], "3.9.9");
    assert_eq!(json["config"]["tasks"]["build"]["run"], "./mvnw package");
    assert_eq!(json["config"]["tasks"]["test"]["run"], "./mvnw test");
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
fn init_json_reports_detected_mode_for_docker_legacy_fixture() {
    let fixture = real_fixture_path("docker-legacy");
    let output = run_ota(&["init", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "docker-legacy");
    assert_eq!(
        json["config"]["services"]["web"]["provider"],
        "docker-compose"
    );
    assert_eq!(
        json["config"]["services"]["db"]["start"],
        "docker compose up -d db"
    );
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "services.web.provider"
                    && inference["source"] == "docker-compose.yml#services.web"
            })
    );
}

#[test]
fn init_json_reports_detected_mode_for_rust_cargo_fixture() {
    let fixture = real_fixture_path("rust-cargo");
    let output = run_ota(&["init", "--json", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["mode"], "detected");
    assert_eq!(json["config"]["project"]["name"], "ota-rust-real");
    assert_eq!(json["config"]["runtimes"]["rust"], "1.85.0");
    assert_eq!(json["config"]["tools"]["cargo"], "*");
    assert_eq!(json["config"]["tasks"]["build"]["run"], "cargo build");
    assert_eq!(json["config"]["tasks"]["test"]["run"], "cargo test");
}

#[test]
fn init_write_writes_high_confidence_contract_for_rust_cargo_fixture() {
    let fixture = copy_fixture_to_temp("rust-cargo");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for rust cargo fixture");

    assert!(written.contains("name: ota-rust-real"));
    assert!(written.contains("rust: 1.85.0"));
    assert!(written.contains("cargo: '*'"));
    assert!(written.contains("run: cargo build"));
    assert!(written.contains("run: cargo test"));

    let validate_output = run_ota(&["validate", fixture.path().to_str().unwrap()]);
    assert!(
        validate_output.status.success(),
        "validate stderr was: {}",
        String::from_utf8_lossy(&validate_output.stderr)
    );
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
    assert_eq!(
        json["config"]["services"]["web"]["provider"],
        "docker-compose"
    );
    assert_eq!(
        json["config"]["services"]["web"]["stop"],
        "docker compose stop web"
    );
    assert_eq!(json["config"]["tasks"]["dev"]["run"], "pnpm dev");
}

#[test]
fn init_write_writes_high_confidence_contract_for_docker_heavy_node_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for docker-heavy fixture");

    assert!(written.contains("name: ota-containerized-web"));
    assert!(written.contains("node: 22.3.0"));
    assert!(written.contains("pnpm: 10.5.0"));
    assert!(written.contains("provider: docker-compose"));
    assert!(written.contains("run: pnpm build"));
    assert!(written.contains("run: pnpm dev"));
}

#[test]
fn detect_json_handles_rust_cargo_fixture() {
    let fixture = real_fixture_path("rust-cargo");
    let output = run_ota(&["detect", "--json", "--dry-run", fixture.to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["written"], false);
    assert_eq!(json["config"]["project"]["name"], "ota-rust-real");
    assert_eq!(json["config"]["runtimes"]["rust"], "1.85.0");
    assert_eq!(json["config"]["tools"]["cargo"], "*");
    assert_eq!(json["config"]["tasks"]["test"]["run"], "cargo test");
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "runtimes.rust"
                    && inference["source"] == "rust-toolchain.toml#toolchain.channel"
            })
    );
}

#[test]
fn detect_writes_high_confidence_contract_for_docker_heavy_node_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");

    let output = run_ota(&["detect", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for docker-heavy fixture");

    assert!(written.contains("name: ota-containerized-web"));
    assert!(written.contains("node: 22.3.0"));
    assert!(written.contains("pnpm: 10.5.0"));
    assert!(written.contains("provider: docker-compose"));
    assert!(written.contains("run: pnpm build"));
    assert!(written.contains("run: pnpm dev"));
}

#[cfg(unix)]
#[test]
fn detect_writes_high_confidence_contract_for_java_gradle_fixture() {
    let fixture = copy_fixture_to_temp("java-gradle");

    let output = run_ota(&["detect", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for java gradle fixture");

    assert!(written.contains("name: ota-java-service"));
    assert!(written.contains("java: '21'"));
    assert!(written.contains("gradle: 8.10.2"));
    assert!(written.contains("run: ./gradlew build"));
}

#[cfg(unix)]
#[test]
fn detect_writes_high_confidence_contract_for_java_maven_fixture() {
    let fixture = copy_fixture_to_temp("java-maven");

    let output = run_ota(&["detect", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for java maven fixture");

    assert!(written.contains("name: ota-maven-service"));
    assert!(written.contains("java: '21'"));
    assert!(!written.contains("tools:"));
    assert!(!written.contains("tasks:"));
}

#[cfg(unix)]
#[test]
fn detect_writes_high_confidence_contract_for_rust_cargo_fixture() {
    let fixture = copy_fixture_to_temp("rust-cargo");

    let output = run_ota(&["detect", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for rust cargo fixture");

    assert!(written.contains("name: ota-rust-real"));
    assert!(written.contains("rust: 1.85.0"));
    assert!(written.contains("cargo: '*'"));
    assert!(written.contains("run: cargo build"));
    assert!(written.contains("run: cargo test"));
}

#[cfg(unix)]
#[test]
fn detect_json_handles_compose_yaml_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");
    rename_if_exists(fixture.path(), "docker-compose.yml", "compose.yaml");

    let output = run_ota(&[
        "detect",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(
        json["config"]["services"]["web"]["provider"],
        "docker-compose"
    );
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "services.web.provider"
                    && inference["source"] == "compose.yaml#services.web"
            })
    );
}

#[cfg(unix)]
#[test]
fn detect_json_handles_compose_yml_fixture() {
    let fixture = copy_fixture_to_temp("docker-heavy-node");
    rename_if_exists(fixture.path(), "docker-compose.yml", "compose.yml");

    let output = run_ota(&[
        "detect",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(
        json["config"]["services"]["web"]["provider"],
        "docker-compose"
    );
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "services.web.provider"
                    && inference["source"] == "compose.yml#services.web"
            })
    );
}

#[cfg(unix)]
#[test]
fn detect_json_surfaces_declared_compose_healthcheck_on_real_fixture() {
    let fixture = copy_fixture_to_temp("docker-legacy");
    fs::write(
        fixture.path().join("docker-compose.yml"),
        r#"services:
  web:
    build: .
  db:
    image: postgres:16
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -h localhost -p 5432"]
"#,
    )
    .expect("compose file should be written");

    let output = run_ota(&[
        "detect",
        "--json",
        "--dry-run",
        fixture.path().to_str().unwrap(),
    ]);
    let json = stdout_json(&output);

    assert_eq!(
        json["config"]["services"]["db"]["healthcheck"],
        "pg_isready -h localhost -p 5432"
    );
    assert!(
        json["inferred"]
            .as_array()
            .unwrap()
            .iter()
            .any(|inference| {
                inference["field"] == "services.db.healthcheck"
                    && inference["source"] == "docker-compose.yml#services.db.healthcheck.test"
                    && inference["confidence"] == "medium"
            })
    );
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
    assert!(json.get("path").is_some());
    assert_eq!(json["config"]["version"], 1);
    assert_eq!(inferred[0]["confidence"], "high");
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

#[test]
fn init_write_writes_high_confidence_contract_for_polyglot_ops_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");

    let output = run_ota(&["init", "--write", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for polyglot fixture");

    assert!(written.contains("name: polyglot-ops"));
    assert!(written.contains("go: 1.24.2"));
    assert!(written.contains("python: 3.12.6"));
    assert!(written.contains("app:"));
    assert!(written.contains("postgres:"));
    assert!(written.contains("provider: docker-compose"));
    assert!(!written.contains("tools:"));
    assert!(!written.contains("tasks:"));
}

#[test]
fn detect_writes_high_confidence_contract_for_polyglot_ops_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");

    let output = run_ota(&["detect", fixture.path().to_str().unwrap()]);
    let stderr = String::from_utf8_lossy(&output.stderr);

    assert!(output.status.success(), "stderr was: {stderr}");

    let written = fs::read_to_string(fixture.path().join("ota.yaml"))
        .expect("ota.yaml should be written for polyglot fixture");

    assert!(written.contains("name: polyglot-ops"));
    assert!(written.contains("go: 1.24.2"));
    assert!(written.contains("python: 3.12.6"));
    assert!(written.contains("app:"));
    assert!(written.contains("postgres:"));
    assert!(written.contains("provider: docker-compose"));
    assert!(!written.contains("tools:"));
    assert!(!written.contains("tasks:"));
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
    assert!(json.get("path").is_some());
    assert_eq!(json["findings"].as_array().unwrap().len(), 2);
    assert_eq!(json["findings"][0]["severity"], "error");
    assert_eq!(
        json["findings"][0]["summary"],
        "Service healthcheck failed: postgres"
    );
    assert!(json["findings"][0]["why"].is_string());
    assert!(json["findings"][0]["next"].is_string());
    assert_eq!(json["findings"][1]["severity"], "warn");
    assert_eq!(
        json["findings"][1]["summary"],
        "Ephemeral lifecycle is advisory only in V1"
    );
}

#[cfg(unix)]
#[test]
fn doctor_json_uses_default_env_value_on_real_fixture() {
    let fixture = copy_fixture_to_temp("docker-legacy");
    let contract = r#"
version: 1
project:
  name: docker-legacy
env:
  OTA_ENV:
    required: false
    default: local
    allowed:
      - local
      - ci
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["findings"].as_array().unwrap().len(), 0);
}

#[cfg(unix)]
#[test]
fn doctor_json_reports_invalid_allowed_env_value_on_real_fixture() {
    let fixture = copy_fixture_to_temp("docker-legacy");
    let contract = r#"
version: 1
project:
  name: docker-legacy
env:
  OTA_ENV:
    required: false
    allowed:
      - local
      - ci
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota_with_env(
        &["doctor", "--json", fixture.path().to_str().unwrap()],
        [("OTA_ENV", "prod")],
    );
    let json =
        serde_json::from_slice::<Value>(&output.stdout).expect("stdout should be valid JSON");

    assert_eq!(output.status.code(), Some(1));
    assert_eq!(json["ok"], false);
    assert_eq!(json["findings"].as_array().unwrap().len(), 1);
    assert_eq!(json["findings"][0]["severity"], "error");
    assert_eq!(
        json["findings"][0]["summary"],
        "Invalid environment value: OTA_ENV"
    );
}

#[cfg(unix)]
#[test]
fn up_runs_service_start_and_stops_in_post_setup_diagnosis_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  postgres:
    required: true
    start: touch .service-ready
    healthcheck: test -f .service-ready
tasks:
  setup:
    run: printf ready > prepared.txt
checks:
  - name: docs-ops
    kind: health
    severity: error
    run: test -f docs/ops.md
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["up", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("NOT READY"));
    assert!(stdout.contains("Phase: post-setup diagnosis"));
    assert!(stdout.contains("ERROR  Check failed: docs-ops"));
    assert!(fixture.path().join(".service-ready").exists());
    assert!(fixture.path().join("prepared.txt").exists());
}

#[cfg(unix)]
#[test]
fn up_stops_in_services_phase_when_required_service_healthcheck_still_fails_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  postgres:
    required: true
    start: touch .service-started
    healthcheck: test -f .service-ready
tasks:
  setup:
    run: printf ready > prepared.txt
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["up", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(1));
    assert!(stdout.contains("NOT READY"));
    assert!(stdout.contains("Phase: services"));
    assert!(stdout.contains("ERROR  Service healthcheck failed: postgres"));
    assert!(fixture.path().join(".service-started").exists());
    assert!(!fixture.path().join("prepared.txt").exists());
}

#[cfg(unix)]
#[test]
fn up_json_reports_contract_shape_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  postgres:
    required: true
    start: touch .service-ready
    healthcheck: test -f .service-ready
  redis:
    required: false
    healthcheck: test -f .redis-ready
tasks:
  setup:
    run: printf ready > prepared.txt
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["up", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["status"], "READY");
    assert_eq!(json["phase"], "post-setup diagnosis");
    assert!(json.get("path").is_some());
    assert!(json["findings"].as_array().unwrap().len() >= 1);
    assert_eq!(json["findings"][0]["severity"], "warn");
    assert!(json.get("service").is_none());
    assert!(json.get("task").is_none());
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

#[cfg(unix)]
#[test]
fn doctor_json_reports_optional_service_failure_as_warning_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  redis:
    required: false
    healthcheck: test -f .redis-ready
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["doctor", "--json", fixture.path().to_str().unwrap()]);
    let json = stdout_json(&output);

    assert_eq!(json["ok"], true);
    assert_eq!(json["findings"].as_array().unwrap().len(), 1);
    assert_eq!(json["findings"][0]["severity"], "warn");
    assert_eq!(
        json["findings"][0]["summary"],
        "Service healthcheck failed: redis"
    );
}

#[cfg(unix)]
#[test]
fn up_returns_ready_when_only_warning_findings_remain_on_real_fixture() {
    let fixture = copy_fixture_to_temp("polyglot-ops");
    let contract = r#"
version: 1
project:
  name: polyglot-ops
services:
  postgres:
    required: true
    start: touch .service-ready
    healthcheck: test -f .service-ready
  redis:
    required: false
    healthcheck: test -f .redis-ready
tasks:
  setup:
    run: printf ready > prepared.txt
"#;

    fs::write(fixture.path().join("ota.yaml"), contract).expect("contract should be written");

    let output = run_ota(&["up", fixture.path().to_str().unwrap()]);
    let stdout = String::from_utf8_lossy(&output.stdout);

    assert_eq!(output.status.code(), Some(0));
    assert!(stdout.contains("READY"));
    assert!(stdout.contains("Phase: post-setup diagnosis"));
    assert!(stdout.contains("WARN  Service healthcheck failed: redis"));
    assert!(fixture.path().join(".service-ready").exists());
    assert!(fixture.path().join("prepared.txt").exists());
}
