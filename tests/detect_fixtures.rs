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

use std::path::{Path, PathBuf};

use ota::detector::detect_repo;
use ota::parser::parse_contract_str;
use ota::validator::validate_contract;

fn fixture_path(name: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("tests")
        .join("fixtures")
        .join("detect")
        .join(name)
}

fn assert_detected_contract_valid(name: &str) -> ota::detector::DetectReport {
    let report = detect_repo(&fixture_path(name)).expect("fixture detection should succeed");
    let yaml = serde_yaml::to_string(&report.contract).expect("detected contract should serialize");
    let contract =
        parse_contract_str(Path::new("ota.yaml"), &yaml).expect("detected YAML should parse");
    validate_contract(&contract).expect("detected contract should validate");
    report
}

fn assert_high_confidence_projection_valid(report: &ota::detector::DetectReport) {
    let yaml = serde_yaml::to_string(&report.high_confidence_contract())
        .expect("projected contract should serialize");
    let contract =
        parse_contract_str(Path::new("ota.yaml"), &yaml).expect("projected YAML should parse");
    validate_contract(&contract).expect("projected contract should validate");
}

#[test]
fn detects_node_pnpm_fixture() {
    let report = assert_detected_contract_valid("node-pnpm");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-web")
    );
    assert_eq!(
        report.contract.runtimes.get("node"),
        Some(&"22".to_string())
    );
    assert_eq!(
        report.contract.tools.get("pnpm"),
        Some(&"10.1.0".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("dev")
            .map(|task| task.run.as_str()),
        Some("pnpm dev")
    );
}

#[test]
fn detects_python_fixture() {
    let report = assert_detected_contract_valid("python-project");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-py")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&">=3.12".to_string())
    );
}

#[test]
fn detects_go_fixture() {
    let report = assert_detected_contract_valid("go-service");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("go-service")
    );
    assert_eq!(
        report.contract.runtimes.get("go"),
        Some(&"1.24.0".to_string())
    );
}

#[test]
fn detects_tool_versions_fixture() {
    let report = assert_detected_contract_valid("tool-versions-node");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-tool-versions")
    );
    assert_eq!(
        report.contract.runtimes.get("node"),
        Some(&"24.1.0".to_string())
    );
    assert_eq!(
        report.contract.tools.get("pnpm"),
        Some(&"10.2.1".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("build")
            .map(|task| task.run.as_str()),
        Some("npm run build")
    );
}

#[test]
fn detects_poetry_fixture() {
    let report = assert_detected_contract_valid("python-poetry");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-poetry")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&"^3.11".to_string())
    );
}

#[test]
fn detects_node_version_file_fixture() {
    let report = assert_detected_contract_valid("node-version-file");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-node-version")
    );
    assert_eq!(
        report.contract.runtimes.get("node"),
        Some(&"24.0.1".to_string())
    );
}

#[test]
fn detects_python_version_file_fixture() {
    let report = assert_detected_contract_valid("python-version-file");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("python-version-file")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&"3.13.1".to_string())
    );
}

#[test]
fn detects_mixed_node_python_fixture() {
    let report = assert_detected_contract_valid("mixed-node-python");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-fullstack")
    );
    assert_eq!(
        report.contract.runtimes.get("node"),
        Some(&"22".to_string())
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&">=3.12".to_string())
    );
    assert_eq!(
        report.contract.tools.get("pnpm"),
        Some(&"10.4.0".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("dev")
            .map(|task| task.run.as_str()),
        Some("pnpm dev")
    );
}

#[test]
fn detects_fullstack_node_go_fixture() {
    let report = assert_detected_contract_valid("fullstack-node-go");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-gateway")
    );
    assert_eq!(
        report.contract.runtimes.get("node"),
        Some(&"22.4.0".to_string())
    );
    assert_eq!(
        report.contract.runtimes.get("go"),
        Some(&"1.24.0".to_string())
    );
    assert_eq!(
        report.contract.tools.get("npm"),
        Some(&"10.8.2".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("build")
            .map(|task| task.run.as_str()),
        Some("npm run build")
    );
}

#[test]
fn detects_python_version_priority_fixture() {
    let report = assert_detected_contract_valid("python-version-priority");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-py-priority")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&"3.13.2".to_string())
    );
}

#[test]
fn detects_polyglot_tool_versions_fixture() {
    let report = assert_detected_contract_valid("polyglot-tool-versions");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-polyglot")
    );
    assert_eq!(
        report.contract.runtimes.get("node"),
        Some(&"22.6.0".to_string())
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&"3.12.4".to_string())
    );
    assert_eq!(
        report.contract.runtimes.get("go"),
        Some(&"1.24.1".to_string())
    );
    assert_eq!(
        report.contract.tools.get("pnpm"),
        Some(&"10.5.0".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("build")
            .map(|task| task.run.as_str()),
        Some("pnpm build")
    );
}

#[test]
fn detects_python_version_over_tool_versions_fixture() {
    let report = assert_detected_contract_valid("python-tool-versions-conflict");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-python-conflict")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&"3.13.2".to_string())
    );
}

#[test]
fn detects_go_mod_over_tool_versions_fixture() {
    let report = assert_detected_contract_valid("go-tool-versions-conflict");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("go-tool-versions-conflict")
    );
    assert_eq!(
        report.contract.runtimes.get("go"),
        Some(&"1.24.1".to_string())
    );
}

#[test]
fn detects_package_json_project_name_over_pyproject_fixture() {
    let report = assert_detected_contract_valid("project-name-conflict");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-web")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&">=3.12".to_string())
    );
}

#[test]
fn detects_package_json_package_manager_over_tool_versions_fixture() {
    let report = assert_detected_contract_valid("package-manager-conflict");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-web")
    );
    assert_eq!(
        report.contract.tools.get("pnpm"),
        Some(&"10.4.0".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("dev")
            .map(|task| task.run.as_str()),
        Some("pnpm dev")
    );
}

#[test]
fn detects_fullstack_node_compose_fixture() {
    let report = assert_detected_contract_valid("fullstack-node-compose");
    assert_high_confidence_projection_valid(&report);

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-compose-web")
    );
    assert_eq!(
        report.contract.runtimes.get("node"),
        Some(&"22.7.0".to_string())
    );
    assert_eq!(
        report.contract.tools.get("pnpm"),
        Some(&"10.6.0".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("dev")
            .map(|task| task.run.as_str()),
        Some("pnpm dev")
    );
    assert_eq!(
        report
            .contract
            .services
            .get("db")
            .and_then(|service| service.provider.as_deref()),
        Some("docker-compose")
    );
    assert_eq!(
        report
            .contract
            .services
            .get("db")
            .and_then(|service| service.healthcheck.as_deref()),
        Some("pg_isready -U postgres")
    );
    assert_eq!(
        report
            .contract
            .services
            .get("cache")
            .and_then(|service| service.start.as_deref()),
        Some("docker compose up -d cache")
    );
}

#[test]
fn detects_mixed_node_python_compose_fixture() {
    let report = assert_detected_contract_valid("mixed-node-python-compose");
    assert_high_confidence_projection_valid(&report);

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-hybrid-app")
    );
    assert_eq!(
        report.contract.runtimes.get("node"),
        Some(&"22.8.0".to_string())
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&">=3.12".to_string())
    );
    assert_eq!(
        report.contract.tools.get("npm"),
        Some(&"10.9.0".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("worker")
            .map(|task| task.run.as_str()),
        Some("npm run worker")
    );
    assert_eq!(
        report
            .contract
            .services
            .get("postgres")
            .and_then(|service| service.provider.as_deref()),
        Some("docker-compose")
    );
    assert_eq!(
        report
            .contract
            .services
            .get("postgres")
            .and_then(|service| service.healthcheck.as_deref()),
        Some("pg_isready -U ota")
    );
    assert_eq!(
        report.high_confidence_contract().runtimes.get("python"),
        None
    );
}
