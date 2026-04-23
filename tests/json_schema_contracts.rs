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

use serde_json::{Value, json};

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
    let member_task_properties = &member_properties["tasks"]["items"]["properties"];

    assert!(success.get("agent").is_some());
    assert!(success.get("members").is_some());
    assert!(member_properties.get("member").is_some());
    assert!(member_properties.get("tasks").is_some());
    assert!(task_properties.get("selected_variant_os").is_some());
    assert!(task_properties.get("requires_services").is_some());
    assert!(task_properties.get("variants").is_some());
    assert!(task_properties.get("default_mode").is_some());
    assert!(task_properties.get("modes").is_some());
    assert!(member_task_properties.get("requires_services").is_some());
    assert!(member_task_properties.get("default_mode").is_some());
    assert!(member_task_properties.get("modes").is_some());
}

#[test]
fn doctor_schema_includes_agent_summary() {
    let schema = load_schema("docs/spec/json-schemas/doctor.json");
    let shared = load_schema("docs/spec/json-schemas/shared.json");
    let properties = &schema["properties"];
    let member_properties = &properties["members"]["items"]["properties"];
    let execution_properties = &properties["execution"]["properties"];
    let execution_env_properties = &execution_properties["env"]["items"]["properties"];
    let provisioning_action = &shared["$defs"]["provisioningAction"]["properties"];
    let provisioning_entry = &shared["$defs"]["provisioningPlanEntry"]["properties"];

    assert!(properties.get("agent").is_some());
    assert!(properties.get("findings").is_some());
    assert!(properties.get("members").is_some());
    assert!(properties.get("mode").is_some());
    assert!(properties.get("provisioning").is_some());
    assert!(properties.get("provisioning_request").is_some());
    assert!(properties.get("adapter_bootstrap").is_some());
    assert!(member_properties.get("member").is_some());
    assert!(member_properties.get("findings").is_some());
    assert!(execution_properties.get("env").is_some());
    assert!(execution_env_properties.get("policy").is_some());
    assert!(provisioning_action.get("normalized_requirement").is_some());
    assert!(provisioning_action.get("resolved_version").is_some());
    assert!(provisioning_action.get("policy_match").is_some());
    assert!(provisioning_entry.get("normalized_requirement").is_some());
    assert!(provisioning_entry.get("resolved_version").is_some());
    assert!(provisioning_entry.get("policy_match").is_some());
    assert!(properties["summary"]["properties"].get("verdict").is_some());
    assert!(
        properties["summary"]["properties"]
            .get("agent_verdict")
            .is_some()
    );
}

#[test]
fn execution_schema_includes_resolved_and_declared_execution_fields() {
    let schema = load_schema("docs/spec/json-schemas/execution.json");
    let success = &schema["oneOf"][0]["properties"];
    let resolved = &success["resolved"]["properties"];
    let overrides = &success["overrides"]["properties"];

    assert!(success.get("contract_identity").is_some());
    assert!(success.get("declared_execution").is_some());
    assert_eq!(
        success["declared_execution"]["$ref"],
        serde_json::json!("./doctor.json#/properties/execution")
    );
    assert!(resolved.get("backend").is_some());
    assert!(resolved.get("backend_source").is_some());
    assert!(resolved.get("engine_candidates").is_some());
    assert!(resolved.get("target_strategy").is_some());
    assert!(overrides.get("backend").is_some());
    assert!(overrides.get("lifecycle").is_some());
}

#[test]
fn workspace_execution_schema_reports_per_repo_resolved_and_declared_fields() {
    let schema = load_schema("docs/spec/json-schemas/workspace-execution.json");
    let properties = &schema["properties"];
    let summary = &properties["summary"]["properties"];
    let repo = &properties["repos"]["items"]["properties"];

    assert_eq!(
        properties["mode"],
        serde_json::json!({ "const": "execution-plan" })
    );
    assert!(summary.get("repo_count").is_some());
    assert!(summary.get("resolved_count").is_some());
    assert!(summary.get("required_unresolved_count").is_some());
    assert!(repo.get("contract_identity").is_some());
    assert_eq!(
        repo["declared_execution"]["$ref"],
        serde_json::json!("./workspace-doctor.json#/properties/repos/items/properties/execution")
    );
    assert_eq!(
        repo["resolved"]["$ref"],
        serde_json::json!("./execution.json#/oneOf/0/properties/resolved")
    );
    assert!(properties.get("overrides").is_some());
}

#[test]
fn up_schema_preview_execution_includes_optional_image() {
    let schema = load_schema("docs/spec/json-schemas/up.json");
    let preview_execution = &schema["oneOf"][0]["properties"]["execution"]["properties"];
    let preview_contract_identity =
        &schema["oneOf"][0]["properties"]["contract_identity"]["properties"];
    let preview_member_properties =
        &schema["oneOf"][0]["properties"]["members"]["items"]["properties"];
    let preview_member_execution = &preview_member_properties["execution"]["properties"];

    assert!(preview_execution.get("image").is_some());
    assert!(preview_contract_identity.get("project").is_some());
    assert!(preview_contract_identity.get("execution").is_some());
    assert!(preview_contract_identity.get("counts").is_some());
    assert!(preview_member_properties.get("contract_identity").is_some());
    assert!(preview_member_execution.get("image").is_some());
}

#[test]
fn up_schema_keeps_aggregate_member_output_separate_from_repo_receipts() {
    let schema = load_schema("docs/spec/json-schemas/up.json");
    let aggregate = schema["oneOf"]
        .as_array()
        .expect("up schema oneOf should be an array")
        .iter()
        .find(|branch| branch["properties"]["phase"]["const"] == serde_json::json!("aggregate"))
        .expect("up schema should include an aggregate branch");
    let aggregate_member_variants = &aggregate["properties"]["members"]["items"]["oneOf"];

    assert_eq!(
        aggregate["properties"]["phase"]["const"],
        serde_json::json!("aggregate")
    );
    assert_eq!(
        aggregate["required"],
        serde_json::json!([
            "ok", "path", "dry_run", "status", "phase", "findings", "members"
        ])
    );
    assert!(aggregate["properties"].get("receipt").is_none());
    assert!(
        aggregate_member_variants[0]["properties"]
            .get("stderr")
            .is_some()
    );
    assert!(
        aggregate_member_variants[1]["properties"]
            .get("contract_identity")
            .is_some()
    );
}

#[test]
fn policy_review_schema_includes_summary_and_policy_fields() {
    let schema = load_schema("docs/spec/json-schemas/policy-review.json");
    let properties = &schema["properties"];
    let summary = &properties["summary"]["properties"];
    let finding_groups = &properties["finding_groups"]["items"]["properties"];

    assert!(properties.get("policy_source").is_some());
    assert!(properties.get("policy_path").is_some());
    assert!(properties.get("policy").is_some());
    assert!(properties.get("findings").is_some());
    assert!(summary.get("ok").is_some());
    assert!(summary.get("error_count").is_some());
    assert!(summary.get("warn_count").is_some());
    assert!(summary.get("info_count").is_some());
    assert!(finding_groups.get("action_key").is_some());
    assert!(finding_groups.get("action_title").is_some());
    assert!(finding_groups.get("action_next").is_some());
    assert!(finding_groups.get("count").is_some());
}

#[test]
fn policy_init_schema_includes_minimal_policy_pack_shape() {
    let schema = load_schema("docs/spec/json-schemas/policy-init.json");
    let success = &schema["oneOf"][0]["properties"];
    let config = &success["config"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert_eq!(success["mode"]["const"], serde_json::json!("policy"));
    assert_eq!(
        success["preset"]["enum"],
        serde_json::json!(["required-sections", "provisioning", "agent"])
    );
    assert!(config.get("policies").is_some());
    assert_eq!(failure["mode"]["const"], serde_json::json!("policy"));
    assert_eq!(
        failure["preset"]["enum"],
        serde_json::json!(["required-sections", "provisioning", "agent"])
    );
    assert!(failure.get("next").is_some());
}

#[test]
fn detect_schema_includes_comparison_preview() {
    let schema = load_schema("docs/spec/json-schemas/detect.json");
    let success = &schema["oneOf"][0]["properties"];
    let failure = &schema["oneOf"][1]["properties"];
    let comparison = &success["comparison"]["properties"];
    let change = &comparison["changes"]["items"]["properties"];
    let removal = &comparison["removals"]["items"]["properties"];

    assert!(success.get("comparison").is_some());
    assert!(comparison.get("removals").is_some());
    assert!(change.get("owner_kind").is_some());
    assert!(change.get("ownership").is_some());
    assert!(change.get("provenance").is_some());
    assert!(change.get("provenance_key").is_some());
    assert!(change.get("source").is_some());
    assert!(change.get("confidence").is_some());
    assert!(removal.get("owner_kind").is_some());
    assert!(removal.get("ownership").is_some());
    assert!(removal.get("provenance").is_some());
    assert!(removal.get("provenance_key").is_some());
    assert!(success.get("config").is_some());
    assert!(success.get("inferred").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn receipt_schema_includes_receipt_and_findings() {
    let schema = load_schema("docs/spec/json-schemas/receipt.json");
    let success = &schema["oneOf"][0]["properties"];
    let success_summary = &success["summary"]["properties"];
    let success_receipt = &success["receipt"]["properties"];
    let diff = &schema["oneOf"][1]["properties"];
    let diff_baseline = &diff["baseline"]["properties"];
    let diff_summary = &diff["summary"]["properties"];
    let history = &schema["oneOf"][2]["properties"];
    let history_summary = &history["summary"]["properties"];
    let failure = &schema["oneOf"][3]["properties"];

    assert!(success.get("mode").is_some());
    assert!(success.get("receipt").is_some());
    assert!(success_receipt.get("contract_identity").is_some());
    assert!(success_receipt.get("service_termination").is_some());
    assert!(success.get("findings").is_some());
    assert!(success.get("promoted_baseline").is_some());
    assert!(success_summary.get("error_count").is_some());
    assert!(success_summary.get("warn_count").is_some());
    assert!(success_summary.get("info_count").is_some());
    assert!(success_summary.get("step_count").is_some());
    assert!(diff.get("baseline").is_some());
    assert!(diff.get("current").is_some());
    assert!(diff.get("introduced").is_some());
    assert!(diff.get("resolved").is_some());
    assert!(diff.get("unchanged").is_some());
    assert!(diff.get("gate").is_some());
    assert!(diff["gate"]["properties"].get("blocking_summary").is_some());
    assert!(diff["gate"]["properties"].get("blocking_next").is_some());
    assert!(
        diff["gate"]["properties"]
            .get("blocking_provenance")
            .is_some()
    );
    assert!(
        diff["gate"]["properties"]
            .get("blocking_provenance_key")
            .is_some()
    );
    assert!(diff_baseline.get("selection_path").is_some());
    assert!(diff_baseline.get("promoted_at").is_some());
    assert!(diff_baseline.get("contract_identity").is_some());
    assert!(diff_baseline.get("contract_identity_details").is_some());
    assert!(
        diff["current"]["properties"]
            .get("contract_identity")
            .is_some()
    );
    assert!(
        diff["current"]["properties"]
            .get("contract_identity_details")
            .is_some()
    );
    assert!(diff_summary.get("baseline_ok").is_some());
    assert!(diff_summary.get("current_ok").is_some());
    assert!(diff_summary.get("comparison").is_some());
    assert!(
        diff_summary["comparison"]["properties"]
            .get("baseline_identity_label")
            .is_some()
    );
    assert!(
        diff_summary["comparison"]["properties"]
            .get("current_identity_label")
            .is_some()
    );
    assert!(
        diff_summary["comparison"]["properties"]
            .get("identity_changed")
            .is_some()
    );
    assert!(
        diff_summary["comparison"]["properties"]
            .get("readiness_change")
            .is_some()
    );
    assert!(diff_summary.get("introduced").is_some());
    assert!(diff_summary.get("resolved").is_some());
    assert!(diff_summary.get("unchanged").is_some());
    assert!(history.get("archives").is_some());
    assert!(history_summary.get("archive_count").is_some());
    assert!(history_summary.get("invalid_archive_count").is_some());
    assert!(history.get("invalid_archives").is_some());
    assert!(failure.get("errors").is_some());
    assert!(failure.get("error").is_some());
}

#[test]
fn agents_schema_includes_generated_content() {
    let schema = load_schema("docs/spec/json-schemas/agents.json");
    let success = &schema["oneOf"][0]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert!(success.get("output").is_some());
    assert!(success.get("content").is_some());
    assert!(success.get("mode").is_some());
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
fn diff_schema_includes_policy_provenance_on_changes() {
    let schema = load_schema("docs/spec/json-schemas/diff.json");
    let change = &schema["properties"]["changes"]["items"]["properties"];

    assert!(change.get("provenance").is_some());
}

#[test]
fn shared_finding_schema_includes_optional_policy_context() {
    let schema = load_schema("docs/spec/json-schemas/shared.json");
    let finding = &schema["$defs"]["finding"]["properties"];

    assert!(finding.get("code").is_some());
    assert!(finding.get("category").is_some());
    assert!(finding.get("owner").is_some());
    assert!(finding.get("evidence").is_some());
    assert!(finding.get("ownership").is_some());
    assert!(finding.get("provenance").is_some());
    assert!(finding.get("provenance_key").is_some());
    assert!(finding.get("policy_outcome").is_some());
    assert!(finding.get("policy_reason").is_some());
    assert!(finding.get("policy_source").is_some());
    assert!(finding.get("install_scope").is_some());
    assert!(finding.get("mutation_allowed").is_some());

    let evidence = &finding["evidence"]["properties"];
    assert!(evidence.get("observed").is_some());
    assert!(evidence.get("expected").is_some());
    assert!(evidence.get("source").is_some());
    assert!(evidence.get("checked_at").is_some());
    assert!(evidence.get("command").is_some());
    assert!(evidence.get("path").is_some());
}

#[test]
fn init_schema_includes_optional_next_on_failures() {
    let schema = load_schema("docs/spec/json-schemas/init.json");
    let shared = load_schema("docs/spec/json-schemas/shared.json");
    let success_required = schema["oneOf"][0]["required"]
        .as_array()
        .expect("required array");
    let success = &schema["oneOf"][0]["properties"];
    let advisory = &success["pack_advisory"]["properties"];
    let pack_options = &success["pack_options"]["properties"];
    let catalog = &schema["oneOf"][1]["properties"];
    let catalog_pack = &catalog["packs"]["items"]["properties"];
    let catalog_option = &catalog_pack["options"]["items"]["properties"];
    let failure = &schema["oneOf"][2]["properties"];
    let provenance = &shared["$defs"]["contractFieldProvenance"]["properties"];

    assert!(success_required.iter().any(|entry| entry == "provenance"));
    assert!(
        success["mode"]
            .as_object()
            .and_then(|mode| mode.get("enum"))
            .and_then(|mode| mode.as_array())
            .is_some_and(|values| values.iter().any(|value| value == "pack"))
    );
    assert!(
        success["pack"]
            .as_object()
            .and_then(|pack| pack.get("enum"))
            .and_then(|pack| pack.as_array())
            .is_some_and(|values| values.iter().any(|value| value == "php-composer"))
    );
    assert!(success.get("pack").is_some());
    assert!(success.get("pack_options").is_some());
    assert_eq!(
        pack_options["package_manager"]["enum"],
        json!(["npm", "pnpm", "yarn", "bun"])
    );
    assert_eq!(
        pack_options["test_runner"]["enum"],
        json!(["pytest", "unittest"])
    );
    assert!(success.get("pack_advisory").is_some());
    assert!(success.get("config").is_some());
    assert!(success.get("inferred").is_some());
    assert!(success.get("provenance").is_some());
    assert!(advisory.get("selected_pack").is_some());
    assert!(advisory.get("suggested_pack").is_some());
    assert!(advisory.get("selected_pack_score").is_some());
    assert!(advisory.get("suggested_pack_score").is_some());
    assert!(advisory.get("score_gap").is_some());
    assert!(advisory.get("summary").is_some());
    assert!(advisory.get("signal_details").is_some());
    assert!(advisory.get("selected_signal_details").is_some());
    assert!(advisory.get("next").is_some());
    assert!(catalog.get("packs").is_some());
    assert!(catalog_pack.get("command").is_some());
    assert!(catalog_pack.get("next").is_some());
    assert!(catalog_pack.get("does_not_infer").is_some());
    assert!(catalog_pack.get("options").is_some());
    assert!(catalog_option.get("flag").is_some());
    assert!(catalog_option.get("summary").is_some());
    assert!(catalog_option.get("default").is_some());
    assert!(catalog_option.get("values").is_some());
    assert!(provenance.get("field").is_some());
    assert!(provenance.get("provenance").is_some());
    assert!(provenance.get("provenance_key").is_some());
    assert!(provenance.get("source").is_some());
    assert!(provenance.get("confidence").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn workspace_doctor_schema_exists_and_covers_repo_reports() {
    let schema = load_schema("docs/spec/json-schemas/workspace-doctor.json");
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let summary = &schema["properties"]["summary"]["properties"];
    let execution = &repo["execution"]["properties"];
    let execution_env = &execution["env"]["items"]["properties"];

    assert!(repo.get("contract_path").is_some());
    assert!(repo.get("required").is_some());
    assert!(repo.get("findings").is_some());
    assert!(repo.get("agent_verdict").is_some());
    assert!(repo.get("provisioning").is_some());
    assert!(repo.get("adapter_bootstrap").is_some());
    assert!(execution.get("env").is_some());
    assert!(execution_env.get("policy").is_some());
    assert!(execution_env.get("source").is_some());
    assert!(summary.get("verdict").is_some());
    assert!(summary.get("agent_verdict").is_some());
}

#[test]
fn workspace_init_schema_exists_and_covers_scaffold_fields() {
    let schema = load_schema("docs/spec/json-schemas/workspace-init.json");
    let shared = load_schema("docs/spec/json-schemas/shared.json");
    let success_required = schema["oneOf"][0]["required"]
        .as_array()
        .expect("required array");
    let success = &schema["oneOf"][0]["properties"];
    let config = &success["config"]["properties"];
    let repo_summary = &schema["$defs"]["repoSummary"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];
    let provenance = &shared["$defs"]["contractFieldProvenance"]["properties"];

    assert!(success_required.iter().any(|entry| entry == "provenance"));
    assert!(success.get("mode").is_some());
    assert!(success.get("config").is_some());
    assert!(success.get("provenance").is_some());
    assert!(success.get("included").is_some());
    assert!(success.get("missing_contract").is_some());
    assert!(success.get("comparison").is_some());
    assert!(config.get("workspace").is_some());
    assert!(config.get("repos").is_some());
    assert!(provenance.get("field").is_some());
    assert!(provenance.get("provenance").is_some());
    assert!(provenance.get("provenance_key").is_some());
    assert!(provenance.get("source").is_some());
    assert!(provenance.get("confidence").is_some());
    let provenance_enum =
        shared["$defs"]["contractFieldProvenance"]["properties"]["provenance"]["enum"]
            .as_array()
            .expect("provenance enum");
    let provenance_key_enum =
        shared["$defs"]["contractFieldProvenance"]["properties"]["provenance_key"]["enum"]
            .as_array()
            .expect("provenance key enum");
    assert!(
        provenance_enum
            .iter()
            .any(|entry| entry == "workspace-declared")
    );
    assert!(
        provenance_key_enum
            .iter()
            .any(|entry| entry == "workspace_contract")
    );
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
    assert!(task.get("description").is_some());
    assert!(task.get("depends_on").is_some());
    assert!(task.get("requires_services").is_some());
    assert!(task.get("after_success").is_some());
    assert!(task.get("after_failure").is_some());
    assert!(task.get("after_always").is_some());
}

#[test]
fn workspace_run_schema_exists_and_covers_repo_run_reports() {
    let schema = load_schema("docs/spec/json-schemas/workspace-run.json");
    let properties = &schema["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let receipt = &properties["receipt"]["properties"];

    assert!(properties.get("summary").is_some());
    assert!(properties.get("receipt").is_some());
    assert!(receipt.get("contract_identity").is_some());
    assert!(repo.get("status").is_some());
    assert!(repo.get("task").is_some());
    assert!(repo.get("findings").is_some());
    assert!(repo.get("exit_code").is_some());
    assert!(repo.get("stdout").is_some());
    assert!(repo.get("stderr").is_some());
    assert!(repo.get("env_sources").is_some());
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
fn explain_schema_includes_step_provenance() {
    let schema = load_schema("docs/spec/json-schemas/explain.json");
    let step = &schema["properties"]["steps"]["items"]["properties"];

    assert!(step.get("provenance").is_some());
    assert!(step.get("provenance_key").is_some());
}

#[test]
fn workspace_explain_schema_includes_step_provenance() {
    let schema = load_schema("docs/spec/json-schemas/workspace-explain.json");
    let step =
        &schema["properties"]["repos"]["items"]["properties"]["steps"]["items"]["properties"];

    assert!(step.get("provenance").is_some());
    assert!(step.get("provenance_key").is_some());
}

#[test]
fn workspace_up_schema_exists_and_covers_repo_status_fields() {
    let schema = load_schema("docs/spec/json-schemas/workspace-up.json");
    let properties = &schema["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let receipt = &properties["receipt"]["properties"];

    assert!(properties.get("summary").is_some());
    assert!(properties.get("receipt").is_some());
    assert!(receipt.get("contract_identity").is_some());
    assert!(repo.get("status").is_some());
    assert!(repo.get("phase").is_some());
    assert!(repo.get("exit_code").is_some());
    assert!(repo.get("stdout").is_some());
    assert!(repo.get("stderr").is_some());
    assert!(repo.get("env_sources").is_some());
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
    let preview_properties = &schema["oneOf"][0]["properties"];
    let runtime_properties = &schema["oneOf"][1]["properties"];
    let runtime_receipt_properties = &runtime_properties["receipt"]["properties"];
    let runtime_member_properties =
        &runtime_properties["members"]["items"]["oneOf"][0]["properties"];
    let preview_member_properties =
        &runtime_properties["members"]["items"]["oneOf"][1]["properties"];
    let validate_failure_ref = schema["oneOf"]
        .as_array()
        .and_then(|branches| {
            branches
                .iter()
                .filter_map(|branch| branch.get("$ref").and_then(Value::as_str))
                .find(|reference| *reference == "./validate.json#/oneOf/1")
        })
        .expect("up schema should include validate failure shape");

    assert!(preview_properties.get("dry_run").is_some());
    assert!(preview_properties.get("execution").is_some());
    assert!(preview_properties.get("plan").is_some());
    assert!(preview_properties.get("stderr").is_some());
    assert!(runtime_properties.get("stderr").is_some());
    assert!(runtime_properties.get("members").is_some());
    assert!(runtime_receipt_properties.get("runtime").is_some());
    assert!(
        runtime_receipt_properties
            .get("service_termination")
            .is_some()
    );
    assert!(runtime_receipt_properties.get("workloads").is_some());
    assert!(runtime_member_properties.get("member").is_some());
    assert!(runtime_member_properties.get("status").is_some());
    assert!(runtime_member_properties.get("phase").is_some());
    assert!(runtime_member_properties.get("stderr").is_some());
    assert!(preview_member_properties.get("dry_run").is_some());
    assert!(preview_member_properties.get("plan").is_some());
    assert_eq!(validate_failure_ref, "./validate.json#/oneOf/1");
}

#[test]
fn receipt_schema_includes_runtime_endpoint_metadata() {
    let schema = load_schema("docs/spec/json-schemas/receipt.json");
    let shared = load_schema("docs/spec/json-schemas/shared.json");
    let receipt_properties = &schema["oneOf"][0]["properties"]["receipt"]["properties"];
    let resolved_runtime = &shared["$defs"]["resolvedTaskRuntime"]["properties"];

    assert!(receipt_properties.get("runtime").is_some());
    assert!(receipt_properties.get("workloads").is_some());
    assert!(resolved_runtime.get("primary_listener").is_some());
    assert!(resolved_runtime.get("primary_endpoint").is_some());
    assert!(resolved_runtime.get("exposed_endpoints").is_some());
}

#[test]
fn diff_schema_includes_readiness_impact_summary() {
    let schema = load_schema("docs/spec/json-schemas/diff.json");
    let summary = &schema["properties"]["summary"]["properties"];

    assert!(summary.get("readiness_impact").is_some());
    assert!(summary.get("added_count").is_some());
    assert!(summary.get("removed_count").is_some());
    assert!(summary.get("changed_count").is_some());
    assert!(summary.get("weakened_count").is_some());
    assert!(summary.get("strengthened_count").is_some());
}

#[test]
fn explain_schema_includes_steps_and_summary_counts() {
    let schema = load_schema("docs/spec/json-schemas/explain.json");
    let summary = &schema["properties"]["summary"]["properties"];
    let step = &schema["properties"]["steps"]["items"]["properties"];

    assert!(summary.get("error_count").is_some());
    assert!(summary.get("warn_count").is_some());
    assert!(summary.get("info_count").is_some());
    assert!(summary.get("step_count").is_some());
    assert!(step.get("order").is_some());
    assert!(step.get("code").is_some());
    assert!(step.get("severity").is_some());
    assert!(step.get("summary").is_some());
    assert!(step.get("why").is_some());
    assert!(step.get("next").is_some());
}

#[test]
fn workspace_explain_schema_includes_repo_steps() {
    let schema = load_schema("docs/spec/json-schemas/workspace-explain.json");
    let summary = &schema["properties"]["summary"]["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let step = &repo["steps"]["items"]["properties"];

    assert!(summary.get("repo_count").is_some());
    assert!(summary.get("ready_count").is_some());
    assert!(summary.get("not_ready_count").is_some());
    assert!(summary.get("step_count").is_some());
    assert!(repo.get("contract_path").is_some());
    assert!(repo.get("summary").is_some());
    assert!(repo.get("steps").is_some());
    assert!(step.get("order").is_some());
    assert!(step.get("code").is_some());
    assert!(step.get("severity").is_some());
    assert!(step.get("summary").is_some());
}

#[test]
fn release_gate_workflow_publishes_all_schema_artifacts_to_latest_and_versioned_prefixes() {
    let workflow_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/release-gate.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("workflow should be readable");

    assert!(workflow.contains("Publish JSON Schemas to R2"));
    assert!(workflow.contains("find docs/spec/json-schemas -maxdepth 1 -type f"));
    assert!(workflow.contains("basename \"${file}\""));
    assert!(workflow.contains("spec/json-schemas/latest"));
    assert!(workflow.contains("spec/json-schemas/v${version}"));
    assert!(workflow.contains("--content-type application/json"));
    assert!(workflow.contains("--remote"));
    assert!(workflow.contains("Publish install scripts"));
    assert!(workflow.contains("scripts/install.sh"));
    assert!(workflow.contains("scripts/install.ps1"));
    assert!(workflow.contains("--content-type text/plain"));
}
