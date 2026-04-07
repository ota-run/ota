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
fn detects_node_pnpm_lock_fixture() {
    let report = assert_detected_contract_valid("node-pnpm-lock");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-pnpm-lock")
    );
    assert_eq!(report.contract.tools.get("pnpm"), Some(&"*".to_string()));
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
fn detects_node_yarn_lock_fixture() {
    let report = assert_detected_contract_valid("node-yarn-lock");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-yarn-lock")
    );
    assert_eq!(report.contract.tools.get("yarn"), Some(&"*".to_string()));
    assert_eq!(
        report
            .contract
            .tasks
            .get("test")
            .map(|task| task.run.as_str()),
        Some("yarn test")
    );
}

#[test]
fn detects_node_pnpm_workspace_fixture() {
    let report = assert_detected_contract_valid("node-pnpm-workspace");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-pnpm-workspace")
    );
    assert_eq!(report.contract.tools.get("pnpm"), Some(&"*".to_string()));
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
fn detects_node_bun_lock_fixture() {
    let report = assert_detected_contract_valid("node-bun-lock");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-bun-lock")
    );
    assert_eq!(report.contract.tools.get("bun"), Some(&"*".to_string()));
    assert_eq!(
        report
            .contract
            .tasks
            .get("dev")
            .map(|task| task.run.as_str()),
        Some("bun run dev")
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
fn detects_pipenv_fixture() {
    let report = assert_detected_contract_valid("python-pipenv");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("python-pipenv")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&"3.12".to_string())
    );
    assert_eq!(report.contract.tools.get("pipenv"), Some(&"*".to_string()));
}

#[test]
fn detects_uv_fixture() {
    let report = assert_detected_contract_valid("python-uv");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-uv")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&">=3.12".to_string())
    );
    assert_eq!(report.contract.tools.get("uv"), Some(&"*".to_string()));
}

#[test]
fn detects_requirements_fixture() {
    let report = assert_detected_contract_valid("python-requirements");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("python-requirements")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&"3.12.7".to_string())
    );
    assert_eq!(report.contract.tools.get("pip"), Some(&"*".to_string()));
}

#[test]
fn detects_setup_cfg_fixture() {
    let report = assert_detected_contract_valid("python-setup-cfg");
    assert_high_confidence_projection_valid(&report);

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-legacy-python")
    );
    assert_eq!(
        report.contract.runtimes.get("python"),
        Some(&"3.12.8".to_string())
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
fn detects_java_version_file_fixture() {
    let report = assert_detected_contract_valid("java-version-file");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("java-version-file")
    );
    assert_eq!(
        report.contract.runtimes.get("java"),
        Some(&"21".to_string())
    );
}

#[test]
fn detects_sdkman_java_fixture() {
    let report = assert_detected_contract_valid("sdkman-java");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("sdkman-java")
    );
    assert_eq!(
        report.contract.runtimes.get("java"),
        Some(&"21.0.2-tem".to_string())
    );
}

#[test]
fn detects_rust_cargo_fixture() {
    let report = assert_detected_contract_valid("rust-cargo");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("ota-rust")
    );
    assert_eq!(
        report.contract.runtimes.get("rust"),
        Some(&"1.85.0".to_string())
    );
    assert_eq!(report.contract.tools.get("cargo"), Some(&"*".to_string()));
    assert_eq!(
        report
            .contract
            .tasks
            .get("test")
            .map(|task| task.run.as_str()),
        Some("cargo test")
    );
}

#[test]
fn detects_rust_toolchain_file_fixture() {
    let report = assert_detected_contract_valid("rust-toolchain-file");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("rust-toolchain-file")
    );
    assert_eq!(
        report.contract.runtimes.get("rust"),
        Some(&"stable".to_string())
    );
    assert_eq!(report.contract.tools.get("cargo"), Some(&"*".to_string()));
}

#[test]
fn detects_php_composer_fixture() {
    let report = assert_detected_contract_valid("php-composer");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("qredex/php-app")
    );
    assert_eq!(
        report.contract.runtimes.get("php"),
        Some(&"^8.2".to_string())
    );
    assert_eq!(
        report.contract.tools.get("composer"),
        Some(&"*".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("test")
            .map(|task| task.run.as_str()),
        Some("composer run test")
    );
}

#[test]
fn detects_cmake_cpp_fixture() {
    let report = assert_detected_contract_valid("cmake-cpp");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("qredex-cpp")
    );
    assert_eq!(report.contract.tools.get("cmake"), Some(&"*".to_string()));
    assert_eq!(report.contract.runtimes.get("cpp"), Some(&"20".to_string()));
    assert_eq!(
        report
            .contract
            .tasks
            .get("build")
            .map(|task| task.run.as_str()),
        Some("cmake -S . -B build && cmake --build build")
    );
}

#[test]
fn detects_clojure_project_fixture() {
    let report = assert_detected_contract_valid("clojure-project");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("qredex-clj")
    );
    assert_eq!(
        report.contract.tools.get("leiningen"),
        Some(&"*".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("test")
            .map(|task| task.run.as_str()),
        Some("lein test")
    );
}

#[test]
fn detects_haskell_stack_cabal_fixture() {
    let report = assert_detected_contract_valid("haskell-stack-cabal");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("qredex-hs")
    );
    assert_eq!(report.contract.tools.get("stack"), Some(&"*".to_string()));
    assert_eq!(report.contract.tools.get("cabal"), Some(&"*".to_string()));
    assert_eq!(
        report
            .contract
            .tasks
            .get("build")
            .map(|task| task.run.as_str()),
        Some("stack build")
    );
}

#[test]
fn detects_lua_rockspec_fixture() {
    let report = assert_detected_contract_valid("lua-rockspec");

    assert_eq!(
        report
            .contract
            .project
            .as_ref()
            .map(|project| project.name.as_str()),
        Some("qredex-lua-1.0.0-1")
    );
    assert_eq!(
        report.contract.tools.get("luarocks"),
        Some(&"*".to_string())
    );
    assert_eq!(
        report
            .contract
            .tasks
            .get("build")
            .map(|task| task.run.as_str()),
        Some("luarocks make")
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
        Some("docker compose exec -T db sh -lc 'pg_isready -U postgres'")
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
        Some("docker compose exec -T postgres sh -lc 'pg_isready -U ota'")
    );
    assert_eq!(
        report.high_confidence_contract().runtimes.get("python"),
        None
    );
}
