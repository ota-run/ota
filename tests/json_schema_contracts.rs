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
use std::path::Path;

use serde_json::Value;

fn load_schema(path: &str) -> Value {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR")).join(path);
    let contents = fs::read_to_string(&schema_path).expect("schema file should be readable");
    serde_json::from_str(&contents).expect("schema file should be valid JSON")
}

#[test]
fn tasks_schema_includes_agent_and_variant_fields() {
    let schema = load_schema("docs/spec/json-schemas/tasks.json");
    let success = &schema["oneOf"][0]["properties"];
    let task_properties = &success["tasks"]["items"]["properties"];
    let member_properties = &success["members"]["items"]["properties"];

    assert!(success.get("agent").is_some());
    assert!(success.get("members").is_some());
    assert!(member_properties.get("member").is_some());
    assert!(member_properties.get("tasks").is_some());
    assert!(task_properties.get("selected_variant_os").is_some());
    assert!(task_properties.get("variants").is_some());
}

#[test]
fn doctor_schema_includes_agent_summary() {
    let schema = load_schema("docs/spec/json-schemas/doctor.json");
    let properties = &schema["properties"];
    let member_properties = &properties["members"]["items"]["properties"];

    assert!(properties.get("agent").is_some());
    assert!(properties.get("findings").is_some());
    assert!(properties.get("members").is_some());
    assert!(member_properties.get("member").is_some());
    assert!(member_properties.get("findings").is_some());
}

#[test]
fn detect_schema_includes_comparison_preview() {
    let schema = load_schema("docs/spec/json-schemas/detect.json");
    let success = &schema["oneOf"][0]["properties"];
    let failure = &schema["oneOf"][1]["properties"];
    let comparison = &success["comparison"]["properties"];

    assert!(success.get("comparison").is_some());
    assert!(comparison.get("removals").is_some());
    assert!(success.get("config").is_some());
    assert!(success.get("inferred").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn validate_schema_includes_summary_counts() {
    let schema = load_schema("docs/spec/json-schemas/validate.json");
    let success = &schema["oneOf"][0]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert!(success.get("summary").is_some());
    assert!(failure.get("summary").is_some());
    assert_eq!(
        schema["oneOf"][0]["required"],
        serde_json::json!(["ok", "path"])
    );
    assert_eq!(
        schema["oneOf"][1]["required"],
        serde_json::json!(["ok", "path"])
    );
}

#[test]
fn shared_finding_schema_includes_optional_policy_context() {
    let schema = load_schema("docs/spec/json-schemas/shared.json");
    let finding = &schema["$defs"]["finding"]["properties"];

    assert!(finding.get("policy_outcome").is_some());
    assert!(finding.get("policy_reason").is_some());
    assert!(finding.get("policy_source").is_some());
    assert!(finding.get("install_scope").is_some());
    assert!(finding.get("mutation_allowed").is_some());
}

#[test]
fn init_schema_includes_optional_next_on_failures() {
    let schema = load_schema("docs/spec/json-schemas/init.json");
    let failure = &schema["oneOf"][1]["properties"];

    assert!(failure.get("next").is_some());
}

#[test]
fn workspace_doctor_schema_exists_and_covers_repo_reports() {
    let schema = load_schema("docs/spec/json-schemas/workspace-doctor.json");
    let repo = &schema["properties"]["repos"]["items"]["properties"];

    assert!(repo.get("contract_path").is_some());
    assert!(repo.get("required").is_some());
    assert!(repo.get("findings").is_some());
}

#[test]
fn workspace_init_schema_exists_and_covers_scaffold_fields() {
    let schema = load_schema("docs/spec/json-schemas/workspace-init.json");
    let success = &schema["oneOf"][0]["properties"];
    let config = &success["config"]["properties"];
    let repo_summary = &schema["$defs"]["repoSummary"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert!(success.get("mode").is_some());
    assert!(success.get("config").is_some());
    assert!(success.get("included").is_some());
    assert!(success.get("missing_contract").is_some());
    assert!(success.get("comparison").is_some());
    assert!(config.get("workspace").is_some());
    assert!(config.get("repos").is_some());
    assert!(repo_summary.get("name").is_some());
    assert!(repo_summary.get("path").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn workspace_tasks_schema_exists_and_covers_repo_task_reports() {
    let schema = load_schema("docs/spec/json-schemas/workspace-tasks.json");
    let properties = &schema["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let task = &repo["tasks"]["items"]["properties"];

    assert!(properties.get("summary").is_some());
    assert!(repo.get("acquired").is_some());
    assert!(repo.get("depends_on").is_some());
    assert!(repo.get("tasks").is_some());
    assert!(task.get("name").is_some());
    assert!(task.get("kind").is_some());
    assert!(task.get("depends_on").is_some());
}

#[test]
fn workspace_run_schema_exists_and_covers_repo_run_reports() {
    let schema = load_schema("docs/spec/json-schemas/workspace-run.json");
    let repo = &schema["properties"]["repos"]["items"]["properties"];

    assert!(repo.get("status").is_some());
    assert!(repo.get("task").is_some());
    assert!(repo.get("findings").is_some());
    assert!(repo.get("exit_code").is_some());
    assert!(repo.get("stdout").is_some());
    assert!(repo.get("stderr").is_some());
}

#[test]
fn workspace_check_schema_exists_and_covers_repo_check_reports() {
    let schema = load_schema("docs/spec/json-schemas/workspace-check.json");
    let properties = &schema["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];

    assert!(properties.get("summary").is_some());
    assert!(repo.get("contract_path").is_some());
    assert!(repo.get("required").is_some());
    assert!(repo.get("findings").is_some());
}

#[test]
fn workspace_up_schema_exists_and_covers_repo_status_fields() {
    let schema = load_schema("docs/spec/json-schemas/workspace-up.json");
    let properties = &schema["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];

    assert!(properties.get("summary").is_some());
    assert!(repo.get("status").is_some());
    assert!(repo.get("phase").is_some());
    assert!(repo.get("exit_code").is_some());
    assert!(repo.get("stdout").is_some());
    assert!(repo.get("stderr").is_some());
}

#[test]
fn check_schema_includes_member_grouping() {
    let schema = load_schema("docs/spec/json-schemas/check.json");
    let properties = &schema["properties"];
    let member_properties = &properties["members"]["items"]["properties"];

    assert!(properties.get("members").is_some());
    assert!(member_properties.get("member").is_some());
    assert!(member_properties.get("findings").is_some());
}

#[test]
fn up_schema_includes_member_grouping() {
    let schema = load_schema("docs/spec/json-schemas/up.json");
    let runtime_properties = &schema["oneOf"][0]["properties"];
    let member_properties = &runtime_properties["members"]["items"]["properties"];
    let validate_failure_ref = schema["oneOf"][1]["$ref"]
        .as_str()
        .expect("up schema should include validate failure shape");

    assert!(runtime_properties.get("members").is_some());
    assert!(member_properties.get("member").is_some());
    assert!(member_properties.get("status").is_some());
    assert!(member_properties.get("phase").is_some());
    assert_eq!(validate_failure_ref, "./validate.json#/oneOf/1");
}
