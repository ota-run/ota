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

use ota::published_contract_schemas::{generated_contract_schema, published_contract_schemas};
use ota::published_docs_manifest::{generated_doc_manifest, published_doc_manifests};
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
    let sandbox_network = &schema["$defs"]["harnessSandboxNetworkPolicy"]["properties"];
    let sandbox_outbound_target = &schema["$defs"]["harnessSandboxOutboundTarget"]["properties"];
    let sandbox_destination_constraint =
        &schema["$defs"]["harnessSandboxDestinationConstraint"]["properties"];
    let lane_use = &schema["$defs"]["laneUseSummary"]["properties"];
    let lane_use_invocation = &schema["$defs"]["laneUseInvocationSummary"]["properties"];
    let lane_use_mode = &schema["$defs"]["laneUseModeSummary"]["properties"];
    let workflow_properties = &schema["$defs"]["workflowSummary"]["properties"];
    let task_command = &schema["$defs"]["taskCommand"]["properties"];
    let task_launch = &schema["$defs"]["taskLaunch"]["properties"];
    let task_prepare = &schema["$defs"]["taskPrepare"]["properties"];
    let task_aggregate = &schema["$defs"]["taskAggregate"]["properties"];
    let task_adapter_inputs = &schema["$defs"]["taskAdapterInputs"]["properties"];
    let task_compose_adapter_inputs = &schema["$defs"]["taskComposeAdapterInputs"]["properties"];
    let task_bake_adapter_inputs = &schema["$defs"]["taskBakeAdapterInputs"]["properties"];
    let task_input = &schema["$defs"]["taskInput"]["properties"];
    let task_action_variants = schema["$defs"]["taskAction"]["oneOf"]
        .as_array()
        .expect("task action variants");
    let agent_properties = &success["agent"]["properties"];
    let task_properties = &success["tasks"]["items"]["properties"];
    let member_properties = &success["members"]["items"]["properties"];
    let member_agent_properties = &member_properties["agent"]["properties"];
    let member_task_properties = &member_properties["tasks"]["items"]["properties"];
    let task_kind_enum = task_properties["kind"]
        .as_object()
        .and_then(|_| task_properties["kind"]["enum"].as_array())
        .expect("task kind enum");
    let task_mode_kind_enum = task_properties["modes"]["items"]["properties"]["kind"]["enum"]
        .as_array()
        .expect("task mode kind enum");

    assert!(success.get("workflow").is_some());
    assert!(success.get("artifacts").is_some());
    assert!(sandbox_network.get("enforcement").is_some());
    assert!(sandbox_network.get("outbound_targets").is_some());
    assert!(sandbox_outbound_target.get("destination_shape").is_some());
    assert!(
        sandbox_outbound_target
            .get("destination_constraint")
            .is_some()
    );
    assert!(
        sandbox_destination_constraint
            .get("source_posture")
            .is_some()
    );
    assert!(sandbox_destination_constraint.get("shared_pin").is_some());
    assert!(lane_use.get("human").is_some());
    assert!(lane_use.get("agent").is_some());
    assert!(lane_use.get("modes").is_some());
    assert!(lane_use_mode.get("mode").is_some());
    assert!(lane_use_mode.get("default").is_some());
    assert!(lane_use_mode.get("availability").is_some());
    assert!(lane_use_mode.get("human").is_some());
    assert!(lane_use_mode.get("agent").is_some());
    assert!(lane_use_invocation.get("callable").is_some());
    assert!(lane_use_invocation.get("command").is_some());
    assert!(lane_use_invocation.get("reason").is_some());
    assert!(workflow_properties.get("use").is_some());
    assert!(workflow_properties.get("run_task_launch").is_some());
    assert!(workflow_properties.get("notes").is_some());
    assert!(success.get("agent").is_some());
    assert!(success.get("members").is_some());
    assert!(member_properties.get("workflow").is_some());
    assert!(agent_properties.get("protected_paths").is_some());
    assert!(agent_properties.get("inferred_boundary_reviewed").is_some());
    assert!(member_properties.get("member").is_some());
    assert!(member_properties.get("tasks").is_some());
    assert!(member_agent_properties.get("protected_paths").is_some());
    assert!(
        member_agent_properties
            .get("inferred_boundary_reviewed")
            .is_some()
    );
    assert!(task_properties.get("use").is_some());
    assert!(task_properties.get("selected_variant_os").is_some());
    assert!(task_properties.get("requires_services").is_some());
    assert!(task_properties.get("requires_artifacts").is_some());
    assert!(task_properties.get("after_success").is_some());
    assert!(task_properties.get("after_failure").is_some());
    assert!(task_properties.get("after_always").is_some());
    assert!(task_properties.get("variants").is_some());
    assert!(task_properties.get("default_mode").is_some());
    assert!(task_properties.get("env").is_some());
    assert!(task_properties.get("inputs").is_some());
    assert!(task_properties.get("modes").is_some());
    assert!(task_properties.get("adapter_inputs").is_some());
    assert!(task_prepare.get("source").is_some());
    assert!(task_prepare.get("engine").is_some());
    assert!(task_properties.get("command").is_some());
    assert!(
        task_properties["variants"]["items"]["properties"]
            .get("command")
            .is_some()
    );
    assert!(task_properties.get("launch").is_some());
    assert!(task_properties.get("action").is_some());
    assert!(task_properties.get("prepare").is_some());
    assert!(task_properties.get("aggregate").is_some());
    assert!(task_command.get("exe").is_some());
    assert!(task_command.get("args").is_some());
    assert!(task_launch.get("exe").is_some());
    assert!(task_launch.get("image").is_some());
    assert!(task_launch.get("volumes").is_some());
    assert!(task_prepare.get("steps").is_some());
    assert!(task_prepare.get("source_kind").is_some());
    assert!(task_aggregate.get("tasks").is_some());
    assert!(task_adapter_inputs.get("compose").is_some());
    assert!(task_adapter_inputs.get("bake").is_some());
    assert!(task_compose_adapter_inputs.get("env_files").is_some());
    assert!(task_compose_adapter_inputs.get("files").is_some());
    assert!(task_compose_adapter_inputs.get("profiles").is_some());
    assert!(task_compose_adapter_inputs.get("project_name").is_some());
    assert!(task_bake_adapter_inputs.get("files").is_some());
    assert!(task_input.get("required").is_some());
    assert!(task_input.get("allowed").is_some());
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"] == json!({ "const": "copy_if_missing" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"] == json!({ "const": "ensure_env_file" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"] == json!({ "const": "ensure_file" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "ensure_git_checkout" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "ensure_git_template" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "ensure_git_checkouts" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"] == json!({ "const": "ensure_bundle" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "ensure_container_network" }))
    );
    assert!(task_action_variants.iter().any(
        |variant| variant["properties"]["kind"] == json!({ "const": "build_container_image" })
    ));
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "reset_compose_service_volume" }))
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "command"));
    assert!(task_kind_enum.iter().any(|entry| entry == "container"));
    assert!(task_kind_enum.iter().any(|entry| entry == "sequence"));
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "dependency_hydration")
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "tool_bootstrap"));
    assert!(task_kind_enum.iter().any(|entry| entry == "aggregate"));
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "copy_if_missing")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_env_file")
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "ensure_file"));
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_git_checkout")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_git_template")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_git_checkouts")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_container_network")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "build_container_image")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "reset_compose_service_volume")
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "ensure_bundle"));
    assert!(task_mode_kind_enum.iter().any(|entry| entry == "command"));
    assert!(task_mode_kind_enum.iter().any(|entry| entry == "container"));
    assert!(task_mode_kind_enum.iter().any(|entry| entry == "sequence"));
    assert!(
        task_mode_kind_enum
            .iter()
            .any(|entry| entry == "dependency_hydration")
    );
    assert!(
        task_mode_kind_enum
            .iter()
            .any(|entry| entry == "tool_bootstrap")
    );
    assert!(member_task_properties.get("use").is_some());
    assert!(member_task_properties.get("requires_services").is_some());
    assert!(member_task_properties.get("after_success").is_some());
    assert!(member_task_properties.get("after_failure").is_some());
    assert!(member_task_properties.get("after_always").is_some());
    assert!(member_task_properties.get("default_mode").is_some());
    assert!(member_task_properties.get("env").is_some());
    assert!(member_task_properties.get("inputs").is_some());
    assert!(member_task_properties.get("modes").is_some());
    assert!(member_task_properties.get("adapter_inputs").is_some());
    assert!(member_task_properties.get("launch").is_some());
    assert!(member_task_properties.get("action").is_some());
    assert!(member_task_properties.get("effects").is_some());
    assert!(member_task_properties.get("prepare").is_some());
    assert!(member_task_properties.get("aggregate").is_some());
}

#[test]
fn published_contract_schema_includes_integration_test_network_kind() {
    let schema = load_schema("docs/spec/json-schemas/contract.json");
    let network_kind_enum = schema["$defs"]["taskEffects"]["properties"]["network_kind"]["enum"]
        .as_array()
        .expect("task effects network kind enum");

    assert!(
        network_kind_enum
            .iter()
            .any(|entry| entry == "integration_test")
    );
}

#[test]
fn published_contract_schema_includes_service_readiness_network_kind() {
    let schema = load_schema("docs/spec/json-schemas/contract.json");
    let network_kind_enum = schema["$defs"]["taskEffects"]["properties"]["network_kind"]["enum"]
        .as_array()
        .expect("task effects network kind enum");

    assert!(
        network_kind_enum
            .iter()
            .any(|entry| entry == "service_readiness")
    );
}

#[test]
fn published_contract_schema_includes_container_image_hydration_network_kind() {
    let schema = load_schema("docs/spec/json-schemas/contract.json");
    let network_kind_enum = schema["$defs"]["taskEffects"]["properties"]["network_kind"]["enum"]
        .as_array()
        .expect("task effects network kind enum");

    assert!(
        network_kind_enum
            .iter()
            .any(|entry| entry == "container_image_hydration")
    );
}

#[test]
fn published_contract_schema_includes_crossing_authority_reference() {
    let schema = load_schema("docs/spec/json-schemas/contract.json");
    let authority = &schema["$defs"]["governance"]["properties"]["crossing_authority"];

    assert_eq!(authority["$ref"], json!("#/$defs/crossingAuthority"));
    assert_eq!(
        schema["$defs"]["crossingAuthority"]["required"],
        json!(["authority_id"])
    );
}

#[test]
fn published_contract_schema_allows_replay_authority_on_generated_source() {
    let schema = load_schema("docs/spec/json-schemas/contract.json");
    let generated_source = schema["$defs"]["generatedArtifact"]["oneOf"]
        .as_array()
        .expect("generated artifact variants")
        .iter()
        .find(|variant| variant["properties"]["kind"] == json!({ "const": "generated_source" }))
        .expect("generated_source artifact variant");

    assert_eq!(
        generated_source["properties"]["replay"],
        json!({ "$ref": "#/$defs/replayBaselineArtifact" }),
        "generated source artifacts must expose the canonical replay authority definition"
    );
}

#[test]
fn services_schema_covers_published_service_summary_fields() {
    let schema = load_schema("docs/spec/json-schemas/services.json");
    let success = &schema["oneOf"][0]["properties"];
    let service = &schema["$defs"]["serviceSummary"]["properties"];
    let readiness = &schema["$defs"]["serviceReadiness"]["properties"];
    let producer = &schema["$defs"]["serviceProducer"]["properties"];
    let manager = &schema["$defs"]["serviceManager"]["properties"];
    let endpoint = &schema["$defs"]["serviceEndpoint"]["properties"];

    assert!(success.get("services").is_some());
    assert!(success.get("members").is_some());
    assert!(service.get("producer").is_some());
    assert!(service.get("manager").is_some());
    assert!(manager.get("engine").is_some());
    assert!(service.get("provider").is_some());
    assert!(service.get("start").is_some());
    assert!(service.get("stop").is_some());
    assert!(service.get("healthcheck").is_some());
    assert!(service.get("readiness").is_some());
    assert!(service.get("endpoints").is_some());
    assert!(service.get("depends_on").is_some());
    assert!(service.get("timeout").is_some());
    assert!(producer.get("repo").is_some());
    assert!(producer.get("task").is_some());
    assert!(producer.get("address_view").is_some());
    assert!(manager.get("kind").is_some());
    assert!(manager.get("env_file").is_some());
    assert!(manager.get("profiles").is_some());
    assert!(endpoint.get("address").is_some());
    assert!(endpoint.get("port").is_some());
    assert!(readiness.get("probe").is_some());
    assert!(readiness.get("success").is_some());
    assert!(readiness.get("body").is_some());
}

#[test]
fn proof_runtime_schema_covers_summary_and_artifact_fields() {
    let schema = load_schema("docs/spec/json-schemas/proof-runtime.json");
    let success = &schema["oneOf"][0]["properties"];
    let artifacts = &success["artifacts"]["properties"];

    assert!(success.get("mode").is_some());
    assert!(success.get("workflow").is_some());
    assert!(success.get("phase").is_some());
    assert!(success.get("execution_boundary").is_some());
    assert!(success.get("summary").is_some());
    assert!(success.get("dependency_evidence").is_some());
    assert!(success.get("seam_observations").is_some());
    assert!(success.get("negative_control").is_some());
    assert!(success.get("workflow_env_artifacts").is_some());
    assert!(success.get("artifacts").is_some());
    assert!(success.get("failure_class").is_some());
    assert!(success.get("cleanup_failure").is_some());
    assert!(success.get("likely_cause_evidence").is_some());
    assert!(success.get("next").is_some());
    assert!(artifacts.get("topology").is_some());
    assert!(artifacts.get("doctor").is_some());
    assert!(artifacts.get("up_log").is_some());
    let not_proved = &schema["$defs"]["proofNotProved"]["properties"];
    assert!(not_proved.get("proof_obligation_id").is_some());
    assert!(not_proved.get("reason").is_some());
    let boundary = &schema["$defs"]["executionBoundary"];
    assert_eq!(boundary["properties"]["schema_version"]["const"], 1);
    assert_eq!(
        boundary["properties"]["target_freshness"]["enum"],
        serde_json::json!(["cold_start_verified", "persistent_state_reused", "unknown"])
    );
    let kinds = not_proved["kind"]["enum"]
        .as_array()
        .expect("proof boundary kinds");
    assert!(kinds.contains(&serde_json::json!("dependency_causality_not_proved")));
    assert!(kinds.contains(&serde_json::json!("dependency_output_shaping_not_proved")));
    assert_eq!(
        success["ok"]["description"].as_str(),
        Some(
            "Execution/readiness success only. Consumers must use proof_verdict together with not_proved to interpret proof breadth."
        )
    );
    let dependency_negative_control = &schema["$defs"]["dependencyNegativeControl"];
    assert_eq!(
        dependency_negative_control["properties"]["evidence_class"]["const"],
        serde_json::json!("derived")
    );
    assert!(
        dependency_negative_control["required"]
            .as_array()
            .expect("negative-control required fields")
            .contains(&serde_json::json!("evidence_class"))
    );
}

#[test]
fn clean_schema_covers_repo_workspace_stale_and_nullable_stale_failure_resource() {
    let schema = load_schema("docs/spec/json-schemas/clean.json");
    let classified_failure = &schema["$defs"]["classifiedFailure"]["properties"];
    let classified_failure_reason_enum = classified_failure["reason"]["enum"]
        .as_array()
        .expect("clean classified failure reason enum");
    let generic_failure = &schema["$defs"]["genericFailure"]["properties"];
    let workspace = &schema["oneOf"][4]["properties"]["workspace"]["properties"];
    let stale_success = &schema["oneOf"][5]["properties"];

    assert!(classified_failure.get("reason").is_some());
    assert!(
        classified_failure_reason_enum.contains(&serde_json::json!("active_execution_conflict"))
    );
    assert!(classified_failure.get("engine").is_some());
    assert!(classified_failure.get("resource_kind").is_some());
    assert!(classified_failure.get("resource_name").is_some());
    assert!(classified_failure.get("registry_path").is_some());
    assert!(classified_failure.get("reasons").is_some());
    assert!(classified_failure.get("active_execution_count").is_some());
    assert!(classified_failure.get("owners").is_some());
    assert_eq!(
        classified_failure["resource_name"]["type"],
        serde_json::json!(["string", "null"])
    );
    assert!(generic_failure.get("summary").is_some());
    assert!(generic_failure.get("error").is_some());
    assert!(generic_failure.get("reason").is_none());
    assert!(workspace.get("root").is_some());
    assert!(workspace.get("members").is_some());
    assert_eq!(stale_success["scope"]["const"], "stale");
    assert!(stale_success.get("containers").is_some());
}

#[test]
fn doctor_schema_includes_agent_summary() {
    let schema = load_schema("docs/spec/json-schemas/doctor.json");
    let shared = load_schema("docs/spec/json-schemas/shared.json");
    let properties = &schema["properties"];
    let workflow_properties = &schema["$defs"]["workflowSummary"]["properties"];
    let agent_properties = &properties["agent"]["properties"];
    let member_properties = &properties["members"]["items"]["properties"];
    let member_agent_properties = &member_properties["agent"]["properties"];
    let execution_properties = &properties["execution"]["properties"];
    let execution_context_properties = &execution_properties["contexts"]["items"]["properties"];
    let execution_env_properties = &execution_properties["env"]["items"]["properties"];
    let provisioning_action = &shared["$defs"]["provisioningAction"]["properties"];
    let provisioning_entry = &shared["$defs"]["provisioningPlanEntry"]["properties"];

    assert!(properties.get("workflow").is_some());
    assert!(workflow_properties.get("run_task_launch").is_some());
    assert!(workflow_properties.get("notes").is_some());
    assert!(properties.get("agent").is_some());
    assert!(properties.get("findings").is_some());
    assert!(properties.get("members").is_some());
    assert!(properties.get("mode").is_some());
    assert_eq!(
        properties["mode"]["enum"],
        serde_json::json!(["native", "container", "remote"])
    );
    assert!(agent_properties.get("protected_paths").is_some());
    assert!(agent_properties.get("inferred_boundary_reviewed").is_some());
    assert!(properties.get("provisioning").is_some());
    assert!(properties.get("provisioning_request").is_some());
    assert!(properties.get("adapter_bootstrap").is_some());
    assert!(member_properties.get("member").is_some());
    assert!(member_properties.get("findings").is_some());
    assert!(member_agent_properties.get("protected_paths").is_some());
    assert!(
        member_agent_properties
            .get("inferred_boundary_reviewed")
            .is_some()
    );
    assert!(execution_properties.get("default_context").is_some());
    assert!(execution_properties.get("contexts").is_some());
    assert!(execution_context_properties.get("name").is_some());
    assert!(execution_context_properties.get("backend").is_some());
    assert!(execution_properties.get("env").is_some());
    assert!(execution_env_properties.get("policy").is_some());
    assert!(execution_env_properties.get("source").is_some());
    assert!(provisioning_action.get("normalized_requirement").is_some());
    assert!(provisioning_action.get("resolved_version").is_some());
    assert!(provisioning_action.get("policy_match").is_some());
    assert!(provisioning_entry.get("normalized_requirement").is_some());
    assert!(provisioning_entry.get("resolved_version").is_some());
    assert!(provisioning_entry.get("policy_match").is_some());
    assert!(properties["summary"]["properties"].get("verdict").is_some());
    assert!(
        properties["summary"]["properties"]
            .get("primary_blocker")
            .is_some()
    );
    assert!(
        properties["summary"]["properties"]
            .get("agent_verdict")
            .is_some()
    );
    assert!(
        properties["summary"]["properties"]["primary_blocker"]["properties"]
            .get("code")
            .is_some()
    );
}

#[test]
fn validate_schema_includes_warn_count_in_success_and_failure_summaries() {
    let schema = load_schema("docs/spec/json-schemas/validate.json");
    let success_summary = &schema["oneOf"][0]["properties"]["summary"]["properties"];
    let failure_summary = &schema["oneOf"][1]["properties"]["summary"]["properties"];

    assert!(success_summary.get("warn_count").is_some());
    assert!(failure_summary.get("warn_count").is_some());
}

#[test]
fn execution_schema_includes_resolved_and_declared_execution_fields() {
    let schema = load_schema("docs/spec/json-schemas/execution.json");
    let success = &schema["oneOf"][0]["properties"];
    let declared_execution = &schema["$defs"]["declaredExecution"]["properties"];
    let declared_execution_context = &schema["$defs"]["executionContext"]["properties"];
    let declared_execution_env = &schema["$defs"]["executionEnv"]["properties"];
    let resolved = &success["resolved"]["properties"];
    let overrides = &success["overrides"]["properties"];
    let workflow = &schema["$defs"]["workflowSummary"]["properties"];

    assert!(success.get("contract_identity").is_some());
    assert!(success.get("workflow").is_some());
    assert!(success.get("task").is_some());
    assert!(workflow.get("run_task_launch").is_some());
    assert!(workflow.get("notes").is_some());
    assert_eq!(
        workflow["run_task_launch"]["$ref"],
        serde_json::json!("#/$defs/taskLaunch")
    );
    assert!(success.get("declared_execution").is_some());
    assert_eq!(
        success["declared_execution"]["$ref"],
        serde_json::json!("#/$defs/declaredExecution")
    );
    assert!(declared_execution.get("default_context").is_some());
    assert!(declared_execution.get("contexts").is_some());
    assert!(declared_execution_context.get("attachments").is_some());
    assert!(declared_execution_env.get("source").is_some());
    assert!(resolved.get("backend").is_some());
    assert!(resolved.get("backend_source").is_some());
    assert!(resolved.get("engine_candidates").is_some());
    assert!(resolved.get("target_strategy").is_some());
    assert!(overrides.get("backend").is_some());
    assert!(overrides.get("lifecycle").is_some());
}

#[test]
fn execution_topology_schema_covers_declared_graph_fields() {
    let schema = load_schema("docs/spec/json-schemas/execution-topology.json");
    let success = &schema["oneOf"][0]["properties"];
    let task = &schema["$defs"]["task"]["properties"];
    let runtime = &schema["$defs"]["runtime"]["properties"];
    let readiness = &schema["$defs"]["readiness"]["properties"];
    let probe = &schema["$defs"]["probe"]["properties"];
    let surface = &schema["$defs"]["surface"]["properties"];
    let task_kind_enum = task["kind"]["enum"].as_array().expect("task kind enum");

    assert!(success.get("contract_identity").is_some());
    assert!(success.get("declared_execution").is_some());
    assert!(success.get("shared_backends").is_some());
    assert!(success.get("readiness_probes").is_some());
    assert!(success.get("surfaces").is_some());
    assert!(success.get("services").is_some());
    assert!(success.get("tasks").is_some());
    assert_eq!(
        success["declared_execution"]["$ref"],
        serde_json::json!("./execution.json#/$defs/declaredExecution")
    );
    assert_eq!(
        success["services"]["items"]["$ref"],
        serde_json::json!("./services.json#/oneOf/0/properties/services/items")
    );
    assert!(task.get("runtime").is_some());
    assert!(task.get("targets").is_some());
    assert!(task.get("launch").is_some());
    assert!(task.get("action").is_some());
    assert!(task.get("variants").is_some());
    assert!(
        task["variants"]["items"]["properties"]
            .get("command")
            .is_some()
    );
    assert!(task.get("modes").is_some());
    assert_eq!(
        task["launch"]["$ref"],
        serde_json::json!("./tasks.json#/$defs/taskLaunch")
    );
    assert_eq!(
        task["action"]["$ref"],
        serde_json::json!("./tasks.json#/$defs/taskAction")
    );
    assert!(runtime.get("attached_surfaces").is_some());
    assert!(runtime.get("surface_attachments").is_some());
    assert!(runtime.get("listeners").is_some());
    assert!(readiness.get("signal_probes").is_some());
    assert!(probe.get("target").is_some());
    assert!(surface.get("readiness").is_some());
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "copy_if_missing")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_env_file")
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "ensure_file"));
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
    assert!(repo.get("workflow").is_some());
    assert!(repo.get("task").is_some());
    assert!(repo.get("contract_identity").is_some());
    assert_eq!(
        repo["declared_execution"]["$ref"],
        serde_json::json!("./execution.json#/$defs/declaredExecution")
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
    let preview_required = schema["oneOf"][0]["required"]
        .as_array()
        .expect("up preview required fields should be an array");
    let preview_properties = &schema["oneOf"][0]["properties"];
    let preview_execution = &schema["oneOf"][0]["properties"]["execution"]["properties"];
    let preview_governance = &schema["oneOf"][0]["properties"]["governance"]["properties"];
    let preview_contract_identity =
        &schema["oneOf"][0]["properties"]["contract_identity"]["properties"];
    let preview_member_properties =
        &schema["oneOf"][0]["properties"]["members"]["items"]["properties"];
    let preview_member_execution = &preview_member_properties["execution"]["properties"];
    let preview_member_governance = &preview_member_properties["governance"]["properties"];

    assert!(preview_properties.get("summary").is_some());
    assert!(
        preview_required
            .iter()
            .any(|field| field == "execution_started")
    );
    assert_eq!(
        preview_properties["execution_started"]["const"],
        serde_json::json!(false)
    );
    assert_eq!(
        preview_properties["summary"]["$ref"],
        serde_json::json!("./doctor.json#/properties/summary")
    );
    assert!(preview_execution.get("image").is_some());
    assert_eq!(
        preview_governance["sandbox_policy"]["$ref"],
        serde_json::json!("./tasks.json#/$defs/harnessSandboxPolicy")
    );
    assert!(preview_contract_identity.get("project").is_some());
    assert!(preview_contract_identity.get("execution").is_some());
    assert!(preview_contract_identity.get("counts").is_some());
    assert!(preview_member_properties.get("contract_identity").is_some());
    assert_eq!(
        preview_member_properties["summary"]["$ref"],
        serde_json::json!("./doctor.json#/properties/summary")
    );
    assert!(preview_member_execution.get("image").is_some());
    assert_eq!(
        preview_member_governance["sandbox_policy"]["$ref"],
        serde_json::json!("./tasks.json#/$defs/harnessSandboxPolicy")
    );
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
fn run_preview_schema_includes_selected_task_env_and_plan_fields() {
    let schema = load_schema("docs/spec/json-schemas/run-preview.json");
    let single_target_required = schema["$defs"]["singleTarget"]["required"]
        .as_array()
        .expect("single-target required fields should be an array");
    let single_target = &schema["$defs"]["singleTarget"]["properties"];
    let env_summary = &schema["$defs"]["envSummary"]["properties"];
    let plan = &schema["$defs"]["plan"]["properties"];
    let governance = &schema["$defs"]["governance"]["properties"];
    let simple_failure = &schema["$defs"]["simpleFailure"]["properties"];

    assert_eq!(
        single_target["summary"]["$ref"],
        serde_json::json!("./doctor.json#/properties/summary")
    );
    assert!(
        single_target_required
            .iter()
            .any(|field| field == "execution_started")
    );
    assert_eq!(
        single_target["execution_started"]["const"],
        serde_json::json!(false)
    );
    assert_eq!(
        single_target["contract_identity"]["$ref"],
        serde_json::json!(
            "./receipt.json#/oneOf/0/properties/receipt/properties/contract_identity"
        )
    );
    assert_eq!(
        single_target["declared_execution"]["$ref"],
        serde_json::json!("./execution.json#/$defs/declaredExecution")
    );
    assert_eq!(
        single_target["resolved"]["$ref"],
        serde_json::json!("./execution.json#/oneOf/0/properties/resolved")
    );
    assert_eq!(
        single_target["requested_task"]["$ref"],
        serde_json::json!("./tasks.json#/oneOf/0/properties/tasks/items")
    );
    assert_eq!(
        single_target["requested_context"]["type"],
        serde_json::json!("string")
    );
    assert_eq!(
        single_target["selected_context"]["type"],
        serde_json::json!("string")
    );
    assert!(single_target.get("env_summary").is_some());
    assert!(single_target.get("sources").is_some());
    assert!(single_target.get("env").is_some());
    assert!(single_target.get("toolchains").is_some());
    assert!(single_target.get("native_prerequisites").is_some());
    assert_eq!(
        governance["sandbox_policy"]["$ref"],
        serde_json::json!("./tasks.json#/$defs/harnessSandboxPolicy")
    );
    assert_eq!(
        governance["evaluation"]["properties"]["preflight"]["properties"]["replay"]["$ref"],
        serde_json::json!("./tasks.json#/$defs/governanceReplayResult")
    );
    assert_eq!(
        governance["evaluation"]["properties"]["post_execution"]["properties"]["replay"]["$ref"],
        serde_json::json!("./tasks.json#/$defs/governanceReplayResult")
    );
    assert!(env_summary.get("source_issue_count").is_some());
    assert!(plan.get("dependency_chain").is_some());
    assert!(plan.get("requirement_lines").is_some());
    assert!(plan.get("actions").is_some());
    assert!(plan.get("notes").is_some());
    assert_eq!(
        simple_failure["dry_run"],
        serde_json::json!({ "const": true })
    );
}

#[test]
fn refusal_canary_schema_distinguishes_expected_refusal_from_admission_drift() {
    let schema = load_schema("docs/spec/json-schemas/refusal-canary.json");
    let properties = &schema["properties"];

    assert_eq!(
        properties["status"]["enum"],
        serde_json::json!([
            "refused_as_expected",
            "wrong_refusal_boundary",
            "refusal_not_observed"
        ])
    );
    assert_eq!(
        properties["canary"]["properties"]["execution_started"]["const"],
        serde_json::json!(false)
    );
    assert_eq!(
        properties["canary"]["properties"]["refusal"]["oneOf"][0]["$ref"],
        serde_json::json!("./tasks.json#/$defs/governanceRefusalRecord")
    );
}

#[test]
fn version_schema_covers_build_identity_and_capability_fields() {
    let schema = load_schema("docs/spec/json-schemas/version.json");
    let properties = &schema["properties"];
    let capability_properties = &properties["contract_capabilities"]["items"]["properties"];

    assert_eq!(properties["ok"]["const"], json!(true));
    assert!(properties.get("semver").is_some());
    assert!(properties.get("version").is_some());
    assert!(properties.get("source_build").is_some());
    assert!(properties.get("commit").is_some());
    assert!(properties.get("dirty").is_some());
    assert!(properties.get("schema_version").is_some());
    assert!(properties.get("contract_capabilities").is_some());
    assert!(capability_properties.get("id").is_some());
    assert!(capability_properties.get("introduced_in").is_some());
}

#[test]
fn run_preview_schema_keeps_member_aggregate_separate_from_single_target_preview() {
    let schema = load_schema("docs/spec/json-schemas/run-preview.json");
    let aggregate = &schema["$defs"]["aggregate"]["properties"];

    assert_eq!(aggregate["dry_run"], serde_json::json!({ "const": true }));
    assert_eq!(
        aggregate["members"]["items"]["$ref"],
        serde_json::json!("#/$defs/singleTarget")
    );
    assert!(aggregate.get("summary").is_none());
    assert!(aggregate.get("contract_identity").is_none());
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
fn assist_declare_service_schema_covers_preview_and_failure_contract() {
    let schema = load_schema("docs/spec/json-schemas/assist-declare-service.json");
    let success = &schema["oneOf"][0]["properties"];
    let failure = &schema["oneOf"][1]["properties"];
    let inputs = &success["inputs"]["properties"];

    assert_eq!(
        success["operation"]["const"],
        serde_json::json!("declare-service")
    );
    assert_eq!(
        success["subject"]["required"],
        serde_json::json!(["service"])
    );
    assert!(inputs.get("manager").is_some());
    assert!(inputs.get("endpoint").is_some());
    assert!(inputs.get("address").is_some());
    assert!(inputs.get("port").is_some());
    assert!(inputs.get("required").is_some());
    assert!(inputs.get("style").is_some());
    assert!(inputs.get("compose_file").is_some());
    assert!(inputs.get("compose_service").is_some());
    assert!(inputs.get("endpoint_context").is_some());
    assert!(inputs.get("producer").is_some());
    assert!(inputs.get("producer_repo").is_some());
    assert_eq!(
        success["inputs"]["required"],
        serde_json::json!(["required"])
    );
    assert_eq!(
        inputs["style"]["enum"],
        serde_json::json!(["spring-http", "http", "tcp", "compose-health"])
    );
    assert_eq!(
        failure["operation"]["const"],
        serde_json::json!("declare-service")
    );
    assert_eq!(
        failure["subject"]["required"],
        serde_json::json!(["service"])
    );
    assert!(failure.get("why").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn assist_wire_setup_schema_covers_preview_and_failure_contract() {
    let schema = load_schema("docs/spec/json-schemas/assist-wire-setup.json");
    let success = &schema["oneOf"][0]["properties"];
    let subject = &success["subject"]["properties"];
    let inputs = &success["inputs"]["properties"];
    let change = &success["changes"]["items"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert_eq!(success["mode"]["enum"], json!(["preview", "write"]));
    assert_eq!(success["operation"]["const"], json!("wire-setup"));
    assert_eq!(subject["task"]["const"], json!("setup"));
    assert!(inputs.get("run").is_some());
    assert!(inputs.get("script").is_some());
    assert!(inputs.get("copy_from").is_some());
    assert!(inputs.get("copy_to").is_some());
    assert!(inputs.get("services").is_some());
    assert!(inputs.get("clear_services").is_some());
    assert!(inputs.get("internal").is_some());
    assert_eq!(change["path"]["const"], json!("tasks.setup"));
    assert_eq!(change["action"]["const"], json!("set"));
    assert!(failure.get("why").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn assist_bind_task_schema_covers_preview_and_failure_contract() {
    let schema = load_schema("docs/spec/json-schemas/assist-bind-task.json");
    let success = &schema["oneOf"][0]["properties"];
    let subject = &success["subject"]["properties"];
    let inputs = &success["inputs"]["properties"];
    let change = &success["changes"]["items"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert_eq!(success["mode"]["enum"], json!(["preview", "write"]));
    assert_eq!(success["operation"]["const"], json!("bind-task"));
    assert!(subject.get("task").is_some());
    assert!(subject.get("target").is_some());
    assert!(inputs.get("to").is_some());
    assert!(inputs.get("producer_member").is_some());
    assert!(inputs.get("address_view").is_some());
    assert!(inputs.get("activation").is_some());
    assert!(inputs.get("override_input").is_some());
    assert_eq!(change["action"]["const"], json!("set"));
    assert!(failure.get("why").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn assist_declare_env_schema_covers_preview_and_failure_contract() {
    let schema = load_schema("docs/spec/json-schemas/assist-declare-env.json");
    let success = &schema["oneOf"][0]["properties"];
    let subject = &success["subject"]["properties"];
    let inputs = &success["inputs"]["properties"];
    let change = &success["changes"]["items"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert_eq!(success["mode"]["enum"], json!(["preview", "write"]));
    assert_eq!(success["operation"]["const"], json!("declare-env"));
    assert!(subject.get("kind").is_some());
    assert!(subject.get("name").is_some());
    assert!(subject.get("task").is_some());
    assert!(subject.get("source_kind").is_some());
    assert!(subject.get("source_path").is_some());
    assert!(inputs.get("required").is_some());
    assert!(inputs.get("secret").is_some());
    assert!(inputs.get("default").is_some());
    assert!(inputs.get("allowed").is_some());
    assert!(inputs.get("prepend").is_some());
    assert!(inputs.get("append").is_some());
    assert!(inputs.get("source_kind").is_some());
    assert!(inputs.get("source_path").is_some());
    assert!(inputs.get("must_exist").is_some());
    assert!(inputs.get("value").is_some());
    assert_eq!(change["action"]["const"], json!("set"));
    assert!(failure.get("why").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn assist_add_task_schema_covers_preview_and_failure_contract() {
    let schema = load_schema("docs/spec/json-schemas/assist-add-task.json");
    let success = &schema["oneOf"][0]["properties"];
    let subject = &success["subject"]["properties"];
    let inputs = &success["inputs"]["properties"];
    let change = &success["changes"]["items"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert_eq!(success["mode"]["enum"], json!(["preview", "write"]));
    assert_eq!(success["operation"]["const"], json!("add-task"));
    assert!(subject.get("task").is_some());
    assert!(inputs.get("kind").is_some());
    assert!(inputs.get("run").is_some());
    assert!(inputs.get("script").is_some());
    assert!(inputs.get("description").is_some());
    assert!(inputs.get("internal").is_some());
    assert!(inputs.get("listener").is_some());
    assert!(inputs.get("protocol").is_some());
    assert!(inputs.get("address").is_some());
    assert!(inputs.get("port").is_some());
    assert_eq!(change["action"]["const"], json!("set"));
    assert!(failure.get("why").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn assist_normalize_schema_covers_preview_and_failure_contract() {
    let schema = load_schema("docs/spec/json-schemas/assist-normalize.json");
    let success = &schema["oneOf"][0]["properties"];
    let subject = &success["subject"]["properties"];
    let inputs = &success["inputs"]["properties"];
    let change = &success["changes"]["items"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert_eq!(success["mode"]["enum"], json!(["preview", "write"]));
    assert_eq!(success["operation"]["const"], json!("normalize"));
    assert!(subject.get("task").is_some());
    assert!(subject.get("into").is_some());
    assert!(inputs.get("into").is_some());
    assert_eq!(success["changes"]["minItems"], json!(2));
    assert_eq!(change["action"]["enum"], json!(["delete", "set"]));
    assert!(failure.get("why").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn detect_schema_includes_comparison_preview() {
    let schema = load_schema("docs/spec/json-schemas/detect.json");
    let shared = load_schema("docs/spec/json-schemas/shared.json");
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
    assert!(success.get("toolchain_opportunities").is_some());
    assert!(
        shared["$defs"]["toolchainOpportunity"]["properties"]
            .get("candidate_providers")
            .is_some()
    );
    assert!(failure.get("next").is_some());
}

#[test]
fn receipt_schema_includes_receipt_and_findings() {
    let schema = load_schema("docs/spec/json-schemas/receipt.json");
    let variants = schema["oneOf"].as_array().expect("receipt oneOf");
    let success = &variants[0]["properties"];
    let success_summary = &success["summary"]["properties"];
    let success_receipt = &success["receipt"]["properties"];
    let diff = &variants
        .iter()
        .find(|variant| variant["properties"]["mode"]["const"] == "diff")
        .expect("diff variant")["properties"];
    let diff_baseline = &diff["baseline"]["properties"];
    let diff_summary = &diff["summary"]["properties"];
    let history = &variants
        .iter()
        .find(|variant| variant["properties"]["mode"]["const"] == "history")
        .expect("history variant")["properties"];
    let history_summary = &history["summary"]["properties"];
    let failure = &variants
        .iter()
        .find(|variant| variant["properties"]["ok"]["const"] == false)
        .expect("failure variant")["properties"];

    assert!(success.get("mode").is_some());
    assert!(success.get("receipt").is_some());
    assert!(success_receipt.get("contract_identity").is_some());
    assert!(success_receipt.get("service_termination").is_some());
    assert!(success_receipt.get("status").is_some());
    assert!(success_receipt.get("failed_task").is_some());
    assert!(success_receipt.get("failed_dependency").is_some());
    assert!(success_receipt.get("failure_origin").is_some());
    assert!(success_receipt.get("dependency_steps").is_some());
    assert!(success_receipt.get("next_steps").is_some());
    let evaluated_input = &success_receipt["evaluated_inputs"]["items"]["properties"];
    assert!(evaluated_input.get("expected_identity").is_some());
    assert!(evaluated_input.get("execution_started").is_some());
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
    assert_eq!(
        success["archive_context"]["oneOf"][0]["properties"]["kind"]["const"],
        "readiness"
    );
    assert_eq!(
        success["archive_context"]["oneOf"][1]["properties"]["kind"]["const"],
        "execution"
    );
    assert_eq!(
        success["archive_context"]["oneOf"][2]["properties"]["schema_version"]["const"],
        2
    );
    assert_eq!(
        success["archive_context"]["oneOf"][2]["properties"]["semantic_scope"]["$ref"],
        "#/$defs/crossingSemanticScope"
    );
    assert_eq!(
        history["invalid_archives"]["items"]["properties"]["posture"]["enum"],
        serde_json::json!(["legacy_unverified", "invalid"])
    );
    assert!(failure.get("errors").is_some());
    assert!(failure.get("error").is_some());
}

#[test]
fn task_schema_includes_governance_replay_surface() {
    let schema = load_schema("docs/spec/json-schemas/tasks.json");
    let replay = &schema["$defs"]["governanceReplayResult"]["properties"];
    let preflight = &schema["$defs"]["governancePreflightEvaluation"]["properties"];
    let preflight_classes = &schema["$defs"]["governancePreflightEvidenceClasses"]["properties"];
    let post_classes = &schema["$defs"]["governancePostExecutionEvidenceClasses"]["properties"];

    assert!(replay.get("status").is_some());
    assert!(replay.get("mismatches").is_some());
    assert_eq!(
        preflight["replay"]["$ref"],
        serde_json::json!("#/$defs/governanceReplayResult")
    );
    assert_eq!(
        preflight_classes["replay"]["type"],
        serde_json::json!("string")
    );
    assert_eq!(post_classes["replay"]["type"], serde_json::json!("string"));
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
    let success_warning = &success["warning_details"]["items"]["properties"];

    assert!(success.get("summary").is_some());
    assert!(failure.get("summary").is_some());
    assert!(success_warning.get("provenance").is_some());
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
    assert!(finding.get("toolchain_opportunity").is_some());

    let evidence = &finding["evidence"]["properties"];
    assert!(evidence.get("observed").is_some());
    assert!(evidence.get("expected").is_some());
    assert!(evidence.get("source").is_some());
    assert!(evidence.get("checked_at").is_some());
    assert!(evidence.get("command").is_some());
    assert!(evidence.get("path").is_some());

    let opportunity = &schema["$defs"]["toolchainOpportunity"]["properties"];
    assert!(opportunity.get("ecosystem").is_some());
    assert!(opportunity.get("fallback_runtime").is_some());
    assert!(opportunity.get("fallback_tools").is_some());
    assert!(opportunity.get("candidate_providers").is_some());
    assert!(opportunity.get("shipped").is_some());
    assert!(opportunity.get("agent_note").is_some());
}

#[test]
fn shared_inference_schema_includes_annotation_metadata() {
    let schema = load_schema("docs/spec/json-schemas/shared.json");
    let inference = &schema["$defs"]["inference"]["properties"];

    assert!(inference.get("field").is_some());
    assert_eq!(
        inference["type"]["enum"],
        json!([
            "project", "runtime", "tool", "env", "service", "check", "task", "agent", "field"
        ])
    );
    assert!(inference.get("value").is_some());
    assert!(inference.get("source").is_some());
    assert_eq!(
        inference["signal"]["enum"],
        json!([
            "config",
            "script",
            "lockfile",
            "file",
            "template",
            "convention"
        ])
    );
    assert!(inference.get("confidence").is_some());
    assert_eq!(
        inference["agent_safe"]["enum"],
        json!(["yes", "no", "unknown"])
    );
    assert_eq!(
        inference["agent_signal"]["enum"],
        json!(["verification_candidate", "bootstrap_candidate"])
    );
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
    assert!(success.get("toolchain_opportunities").is_some());
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
fn assist_declare_readiness_schema_covers_preview_and_failure_contract() {
    let schema = load_schema("docs/spec/json-schemas/assist-declare-readiness.json");
    let success = &schema["oneOf"][0]["properties"];
    let subject = &success["subject"]["properties"];
    let inputs = &success["inputs"]["properties"];
    let change = &success["changes"]["items"]["properties"];
    let failure = &schema["oneOf"][1]["properties"];

    assert_eq!(success["mode"]["enum"], json!(["preview", "write"]));
    assert_eq!(success["operation"]["const"], json!("declare-readiness"));
    assert!(subject.get("task").is_some());
    assert!(subject.get("service").is_some());
    assert!(inputs.get("endpoint").is_some());
    assert_eq!(
        inputs["style"]["enum"],
        json!(["spring-http", "http", "tcp", "compose-health"])
    );
    assert_eq!(change["action"]["const"], json!("set"));
    assert!(failure.get("why").is_some());
    assert!(failure.get("next").is_some());
}

#[test]
fn workspace_doctor_schema_exists_and_covers_repo_reports() {
    let schema = load_schema("docs/spec/json-schemas/workspace-doctor.json");
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let summary = &schema["properties"]["summary"]["properties"];
    let summary_primary_blocker = &summary["primary_blocker"]["properties"];
    let repo_primary_blocker = &repo["primary_blocker"]["properties"];
    let execution = &repo["execution"]["properties"];
    let execution_env = &execution["env"]["items"]["properties"];

    assert!(repo.get("contract_path").is_some());
    assert!(repo.get("required").is_some());
    assert!(repo.get("findings").is_some());
    assert!(repo.get("agent_verdict").is_some());
    assert!(repo.get("primary_blocker").is_some());
    assert!(repo.get("provisioning").is_some());
    assert!(repo.get("adapter_bootstrap").is_some());
    assert!(execution.get("env").is_some());
    assert!(execution_env.get("policy").is_some());
    assert!(execution_env.get("source").is_some());
    assert!(summary.get("verdict").is_some());
    assert!(summary.get("agent_verdict").is_some());
    assert!(summary.get("primary_blocker").is_some());
    assert!(summary_primary_blocker.get("repo").is_some());
    assert!(summary_primary_blocker.get("code").is_some());
    assert!(repo_primary_blocker.get("code").is_some());
}

#[test]
fn workspace_check_schema_exists_and_covers_primary_blockers() {
    let schema = load_schema("docs/spec/json-schemas/workspace-check.json");
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let summary = &schema["properties"]["summary"]["properties"];
    let summary_primary_blocker = &summary["primary_blocker"]["properties"];
    let repo_primary_blocker = &repo["primary_blocker"]["properties"];

    assert!(summary.get("primary_blocker").is_some());
    assert!(summary_primary_blocker.get("repo").is_some());
    assert!(summary_primary_blocker.get("code").is_some());
    assert!(repo.get("primary_blocker").is_some());
    assert!(repo_primary_blocker.get("code").is_some());
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
    let task_kind_enum = task["kind"]["enum"]
        .as_array()
        .expect("workspace task kind enum");
    let task_command = &schema["$defs"]["taskCommand"]["properties"];
    let task_launch = &schema["$defs"]["taskLaunch"]["properties"];
    let task_prepare = &schema["$defs"]["taskPrepare"]["properties"];
    let task_aggregate = &schema["$defs"]["taskAggregate"]["properties"];
    let task_action_variants = schema["$defs"]["taskAction"]["oneOf"]
        .as_array()
        .expect("task action variants");

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
    assert!(task.get("command").is_some());
    assert!(task.get("launch").is_some());
    assert!(task.get("action").is_some());
    assert!(task.get("prepare").is_some());
    assert!(task.get("aggregate").is_some());
    assert!(task_command.get("exe").is_some());
    assert!(task_command.get("args").is_some());
    assert!(task_launch.get("exe").is_some());
    assert!(task_launch.get("image").is_some());
    assert!(task_prepare.get("steps").is_some());
    assert!(task_aggregate.get("tasks").is_some());
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"] == json!({ "const": "copy_if_missing" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"] == json!({ "const": "ensure_env_file" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"] == json!({ "const": "ensure_file" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "ensure_git_checkout" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "ensure_git_template" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "ensure_git_checkouts" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"] == json!({ "const": "ensure_bundle" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "ensure_container_network" }))
    );
    assert!(
        task_action_variants
            .iter()
            .any(|variant| variant["properties"]["kind"]
                == json!({ "const": "reset_compose_service_volume" }))
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "command"));
    assert!(task_kind_enum.iter().any(|entry| entry == "container"));
    assert!(task_kind_enum.iter().any(|entry| entry == "sequence"));
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "dependency_hydration")
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "aggregate"));
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "copy_if_missing")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_env_file")
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "ensure_file"));
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_git_checkout")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_git_template")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_git_checkouts")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "ensure_container_network")
    );
    assert!(
        task_kind_enum
            .iter()
            .any(|entry| entry == "reset_compose_service_volume")
    );
    assert!(task_kind_enum.iter().any(|entry| entry == "ensure_bundle"));
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
    assert!(repo.get("next").is_some());
    assert!(repo.get("next_steps").is_some());
    assert!(repo.get("exit_code").is_some());
    assert!(repo.get("stdout").is_some());
    assert!(repo.get("stderr").is_some());
    assert!(repo.get("env_sources").is_some());
}

#[test]
fn workspace_refresh_schema_exists_and_covers_preview_and_apply_modes() {
    let schema = load_schema("docs/spec/json-schemas/workspace-refresh.json");
    let properties = &schema["properties"];

    assert_eq!(
        schema["required"],
        serde_json::json!(["ok", "path", "mode", "summary", "receipt", "repos"])
    );
    assert!(properties.get("summary").is_some());
    assert!(properties.get("receipt").is_some());
    assert!(properties.get("repos").is_some());
    assert_eq!(
        properties["mode"]["enum"],
        serde_json::json!(["preview", "refresh"])
    );
    assert_eq!(
        properties["receipt"]["$ref"],
        serde_json::json!("./workspace-up.json#/properties/receipt")
    );
    assert_eq!(
        properties["repos"]["$ref"],
        serde_json::json!("./workspace-up.json#/properties/repos")
    );
}

#[test]
fn workspace_check_schema_exists_and_covers_repo_check_reports() {
    let schema = load_schema("docs/spec/json-schemas/workspace-check.json");
    let properties = &schema["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let execution = &repo["execution"]["properties"];
    let execution_env = &execution["env"]["items"]["properties"];

    assert!(properties.get("summary").is_some());
    assert!(repo.get("contract_path").is_some());
    assert!(repo.get("required").is_some());
    assert!(repo.get("execution").is_some());
    assert!(execution.get("env").is_some());
    assert!(execution_env.get("source").is_some());
    assert!(repo.get("primary_blocker").is_some());
    assert!(repo.get("findings").is_some());
}

#[test]
fn workspace_diff_schema_includes_drift_semantics_and_followup_lanes() {
    let schema = load_schema("docs/spec/json-schemas/workspace-diff.json");
    let summary = &schema["properties"]["summary"]["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];

    assert_eq!(
        schema["properties"]["mode"],
        serde_json::json!({ "const": "diff" })
    );
    assert!(summary.get("missing_repo_count").is_some());
    assert!(summary.get("missing_contract_count").is_some());
    assert!(summary.get("target_unavailable_count").is_some());
    assert!(summary.get("comparison_unresolved_count").is_some());
    assert!(repo.get("drift_kind").is_some());
    assert!(repo.get("target_source").is_some());
    assert!(repo.get("next").is_some());
    assert!(repo.get("next_steps").is_some());
}

#[test]
fn workspace_status_schema_includes_drift_semantics_and_followup_lanes() {
    let schema = load_schema("docs/spec/json-schemas/workspace-status.json");
    let summary = &schema["properties"]["summary"]["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];

    assert_eq!(
        schema["properties"]["mode"],
        serde_json::json!({ "const": "status" })
    );
    assert!(summary.get("missing_repo_count").is_some());
    assert!(summary.get("missing_contract_count").is_some());
    assert!(summary.get("target_unavailable_count").is_some());
    assert!(summary.get("comparison_unresolved_count").is_some());
    assert!(repo.get("drift_kind").is_some());
    assert!(repo.get("target_source").is_some());
    assert!(repo.get("next").is_some());
    assert!(repo.get("next_steps").is_some());
}

#[test]
fn explain_schema_includes_step_provenance() {
    let schema = load_schema("docs/spec/json-schemas/explain.json");
    let action = &schema["properties"]["actions"]["items"]["properties"];
    let step = &schema["properties"]["steps"]["items"]["properties"];

    assert!(action.get("action_key").is_some());
    assert!(action.get("action_title").is_some());
    assert!(action.get("count").is_some());
    assert!(step.get("provenance").is_some());
    assert!(step.get("provenance_key").is_some());
}

#[test]
fn workspace_explain_schema_includes_step_provenance() {
    let schema = load_schema("docs/spec/json-schemas/workspace-explain.json");
    let top_level_action = &schema["properties"]["actions"]["items"]["properties"];
    let action =
        &schema["properties"]["repos"]["items"]["properties"]["actions"]["items"]["properties"];
    let step =
        &schema["properties"]["repos"]["items"]["properties"]["steps"]["items"]["properties"];

    assert!(action.get("action_key").is_some());
    assert!(action.get("action_title").is_some());
    assert!(action.get("count").is_some());
    assert!(top_level_action.get("repo").is_some());
    assert!(top_level_action.get("contract_path").is_some());
    assert!(step.get("provenance").is_some());
    assert!(step.get("provenance_key").is_some());
}

#[test]
fn workspace_up_schema_exists_and_covers_repo_status_fields() {
    let schema = load_schema("docs/spec/json-schemas/workspace-up.json");
    let properties = &schema["properties"];
    let summary = &properties["summary"]["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let receipt = &properties["receipt"]["properties"];

    assert!(properties.get("summary").is_some());
    assert!(summary.get("repo_count").is_some());
    assert!(summary.get("ready_count").is_some());
    assert!(summary.get("not_ready_count").is_some());
    assert!(properties.get("receipt").is_some());
    assert!(receipt.get("contract_identity").is_some());
    assert!(receipt.get("status").is_some());
    assert!(repo.get("status").is_some());
    assert!(repo.get("phase").is_some());
    assert!(repo.get("next").is_some());
    assert!(repo.get("next_steps").is_some());
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
    let preview_execution_properties = &preview_properties["execution"]["properties"];
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
    assert!(preview_execution_properties.get("context").is_some());
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
    assert!(runtime_receipt_properties.get("next_steps").is_some());
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
    assert!(receipt_properties.get("next_steps").is_some());
    assert!(resolved_runtime.get("primary_listener").is_some());
    assert!(resolved_runtime.get("primary_endpoint").is_some());
    assert!(resolved_runtime.get("exposed_endpoints").is_some());
}

#[test]
fn receipt_schema_includes_execution_conflict_metadata() {
    let schema = load_schema("docs/spec/json-schemas/receipt.json");
    let receipt_properties = &schema["oneOf"][0]["properties"]["receipt"]["properties"];
    let execution_conflict = &receipt_properties["execution_conflict"]["properties"];

    assert!(receipt_properties.get("execution_conflict").is_some());
    assert!(execution_conflict.get("reasons").is_some());
}

#[test]
fn receipt_and_preview_schemas_publish_crossing_grant_admission() {
    let receipt = load_schema("docs/spec/json-schemas/receipt.json");
    let authority = &receipt["oneOf"][0]["properties"]["receipt"]["properties"]["crossing"]["properties"]
        ["authority"];
    let preview = load_schema("docs/spec/json-schemas/run-preview.json");

    assert!(
        authority["required"]
            .as_array()
            .is_some_and(|required| { required.iter().any(|field| field == "archive_evidence") })
    );
    assert_eq!(
        authority["properties"]["actor_mode"]["enum"],
        json!(["agent", "non_agent"])
    );
    assert_eq!(
        authority["properties"]["decision"]["const"],
        json!("allowed")
    );
    assert_eq!(
        authority["properties"]["transaction"]["$ref"],
        json!("#/$defs/crossingTransaction")
    );
    assert_eq!(
        authority["properties"]["authority_separation_posture"]["const"],
        json!("current_process_filesystem_guarded")
    );
    assert_eq!(
        authority["properties"]["archive_evidence"]["$ref"],
        json!("#/$defs/crossingGrantArchiveEvidence")
    );
    assert_eq!(
        receipt["$defs"]["crossingGrantArchiveEvidence"]["additionalProperties"],
        json!(false)
    );
    assert_eq!(
        receipt["$defs"]["crossingTransaction"]["properties"]["state"]["enum"],
        json!([
            "pending",
            "completed",
            "failed",
            "interrupted",
            "incomplete"
        ])
    );
    assert_eq!(
        receipt["$defs"]["crossingTransaction"]["properties"]["authentication_posture"]["const"],
        json!("runner_local_content_addressed")
    );
    assert_eq!(
        preview["$defs"]["singleTarget"]["properties"]["crossing_grant_admission"]["$ref"],
        json!("#/$defs/crossingGrantAdmission")
    );
    assert_eq!(
        preview["$defs"]["crossingGrantAdmission"]["properties"]["decision"]["const"],
        json!("admissible_not_consumed")
    );
    assert_eq!(
        preview["$defs"]["crossingGrantAdmissionFailure"]["properties"]["crossing_grant_admission"]
            ["properties"]["decision"]["const"],
        json!("refused")
    );

    let up = load_schema("docs/spec/json-schemas/up.json");
    assert_eq!(
        up["oneOf"][0]["properties"]["crossing_grant_admission"]["$ref"],
        json!("./run-preview.json#/$defs/crossingGrantAdmission")
    );
    assert_eq!(
        up["oneOf"][0]["properties"]["members"]["items"]["properties"]["crossing_grant_admission"]
            ["$ref"],
        json!("./run-preview.json#/$defs/crossingGrantAdmission")
    );
}

#[test]
fn receipt_schema_includes_native_prerequisite_activation_metadata() {
    let schema = load_schema("docs/spec/json-schemas/receipt.json");
    let receipt_properties = &schema["oneOf"][0]["properties"]["receipt"]["properties"];
    let native_prerequisites = &receipt_properties["native_prerequisites"]["items"]["properties"];
    let activation = &native_prerequisites["activation"]["properties"];
    let requires = &native_prerequisites["requires"]["properties"];

    assert!(receipt_properties.get("native_prerequisites").is_some());
    assert!(native_prerequisites.get("provisioning").is_some());
    assert!(native_prerequisites.get("note").is_some());
    assert!(requires.get("runtimes").is_some());
    assert!(requires.get("tools").is_some());
    assert!(requires.get("toolchains").is_some());
    assert!(requires.get("env").is_some());
    assert!(requires.get("checks").is_some());
    assert!(requires.get("source").is_some());
    assert!(activation.get("kind").is_some());
    assert!(activation.get("applied").is_some());
    assert!(activation.get("shell").is_some());
    assert!(activation.get("run").is_some());
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
    let action = &schema["properties"]["actions"]["items"]["properties"];
    let step = &schema["properties"]["steps"]["items"]["properties"];

    assert!(summary.get("error_count").is_some());
    assert!(summary.get("warn_count").is_some());
    assert!(summary.get("info_count").is_some());
    assert!(summary.get("step_count").is_some());
    assert!(action.get("order").is_some());
    assert!(action.get("action_key").is_some());
    assert!(action.get("action_title").is_some());
    assert!(action.get("severity").is_some());
    assert!(action.get("count").is_some());
    assert!(action.get("why").is_some());
    assert!(action.get("next").is_some());
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
    let top_level_action = &schema["properties"]["actions"]["items"]["properties"];
    let repo = &schema["properties"]["repos"]["items"]["properties"];
    let action = &repo["actions"]["items"]["properties"];
    let step = &repo["steps"]["items"]["properties"];

    assert!(summary.get("repo_count").is_some());
    assert!(summary.get("ready_count").is_some());
    assert!(summary.get("not_ready_count").is_some());
    assert!(summary.get("step_count").is_some());
    assert!(schema["properties"].get("actions").is_some());
    assert!(top_level_action.get("repo").is_some());
    assert!(top_level_action.get("action_key").is_some());
    assert!(repo.get("contract_path").is_some());
    assert!(repo.get("summary").is_some());
    assert!(repo.get("actions").is_some());
    assert!(repo.get("steps").is_some());
    assert!(action.get("action_key").is_some());
    assert!(action.get("action_title").is_some());
    assert!(action.get("count").is_some());
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

    assert!(workflow.contains("Publish JSON Schemas and Docs Manifests to R2"));
    assert!(workflow.contains("find docs/spec/json-schemas -maxdepth 1 -type f"));
    assert!(workflow.contains("basename \"${file}\""));
    assert!(workflow.contains("spec/json-schemas/latest"));
    assert!(workflow.contains("spec/json-schemas/v${version}"));
    assert!(workflow.contains("Publish canonical docs manifest"));
    assert!(workflow.contains("find docs/spec/published-docs -maxdepth 1 -type f"));
    assert!(workflow.contains("spec/published-docs/latest"));
    assert!(workflow.contains("spec/published-docs/v${version}"));
    assert!(workflow.contains("--content-type application/json"));
    assert!(workflow.contains("--remote"));
    assert!(workflow.contains("Publish install scripts"));
    assert!(workflow.contains("scripts/install.sh"));
    assert!(workflow.contains("scripts/install.ps1"));
    assert!(workflow.contains("--content-type text/plain"));
}

#[test]
fn published_doc_manifest_files_are_generated_and_in_sync() {
    let manifest_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("spec")
        .join("published-docs");

    for generated in published_doc_manifests() {
        let path = manifest_dir.join(generated.filename);
        let on_disk =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(
            on_disk,
            generated.body,
            "{} is out of sync with the Rust-owned published docs generator; run `cargo run --bin sync_published_doc_manifests`",
            path.display()
        );

        let parsed = generated_doc_manifest(generated.filename)
            .unwrap_or_else(|| panic!("generated manifest {} should exist", generated.filename));
        let on_disk_parsed: Value =
            serde_json::from_str(&on_disk).expect("published doc manifest on disk should parse");
        assert_eq!(
            parsed,
            on_disk_parsed,
            "{} should serialize to the same JSON value as the Rust-owned published docs generator",
            path.display()
        );
    }
}

#[test]
fn canonical_docs_manifest_publishes_contract_reference_source_boundary() {
    let manifest = load_schema("docs/spec/published-docs/canonical-docs.json");
    let contract = manifest["docs"]
        .as_array()
        .expect("canonical docs manifest docs array")
        .iter()
        .find(|entry| entry["id"] == json!("contract-reference"))
        .expect("contract-reference manifest entry");

    assert_eq!(
        contract["source_path"],
        json!("docs/spec/contract-reference.md")
    );
    assert_eq!(
        contract["source_url"],
        json!("https://github.com/ota-run/ota/blob/main/docs/spec/contract-reference.md")
    );
    assert_eq!(
        contract["public_url"],
        json!("https://ota.run/docs/reference/contract")
    );
}

#[test]
fn canonical_docs_manifest_publishes_execution_governance_capability_map() {
    let manifest = load_schema("docs/spec/published-docs/canonical-docs.json");
    let capabilities = manifest["docs"]
        .as_array()
        .expect("canonical docs manifest docs array")
        .iter()
        .find(|entry| entry["id"] == json!("execution-governance-capabilities"))
        .expect("execution-governance-capabilities manifest entry");

    assert_eq!(
        capabilities["source_path"],
        json!("docs/spec/execution-governance-capabilities.md")
    );
    assert_eq!(
        capabilities["public_url"],
        json!("https://ota.run/docs#execution-governance-capabilities")
    );
}

#[test]
fn full_contract_schema_is_published_and_covered_by_schema_publication() {
    let schema_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/spec/json-schemas/contract.json");
    assert!(
        schema_path.exists(),
        "full contract schema must be published at docs/spec/json-schemas/contract.json"
    );

    let workflow_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/release-gate.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("workflow should be readable");

    assert!(
        workflow.contains("find docs/spec/json-schemas -maxdepth 1 -type f"),
        "full contract schema publication should stay on the same generated schema publication path"
    );
    let schema = load_schema("docs/spec/json-schemas/contract.json");
    assert_eq!(
        schema.get("$schema").and_then(|value| value.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    let task_spec = &schema["$defs"]["taskSpec"]["properties"];
    let task_mode_branch = &schema["$defs"]["taskModeBranch"]["properties"];
    let task_launch_variants = schema["$defs"]["taskLaunch"]["oneOf"]
        .as_array()
        .expect("taskLaunch oneOf");
    let env_profile_render = &schema["$defs"]["envProfileRender"]["properties"];
    let workflow = &schema["$defs"]["workflowSpec"]["properties"];
    let workflow_proof = &schema["$defs"]["workflowProof"]["properties"];
    let service_spec = &schema["$defs"]["serviceSpec"]["properties"];
    let workflow_env = &schema["$defs"]["workflowEnv"]["properties"];
    let workflow_instance_task_overlay =
        &schema["$defs"]["workflowInstanceTaskOverlay"]["properties"];
    let workflow_instance_runtime_overlay =
        &schema["$defs"]["workflowInstanceTaskRuntimeOverlay"]["properties"];
    assert!(task_spec.get("compose").is_some());
    assert!(task_mode_branch.get("compose").is_some());
    assert!(
        task_launch_variants
            .iter()
            .any(|variant| { variant["properties"]["kind"] == json!({ "const": "compose" }) })
    );
    assert!(env_profile_render.get("files").is_some());
    assert!(workflow.get("adapter_inputs").is_some());
    assert!(workflow.get("instances").is_some());
    assert!(workflow.get("attach").is_some());
    assert!(workflow.get("proof").is_some());
    assert!(workflow_proof.get("lifecycle").is_some());
    assert!(schema["$defs"].get("workflowLifecycleProof").is_some());
    assert_eq!(
        schema["$defs"]["workflowLifecycleProof"]["required"],
        json!(["services"])
    );
    assert_eq!(
        schema["$defs"]["workflowLifecycleProof"]["properties"]["services"]["minItems"],
        json!(1)
    );
    assert_eq!(
        schema["$defs"]["workflowLifecycleProof"]["properties"]["services"]["uniqueItems"],
        json!(true)
    );
    assert!(
        schema["$defs"]
            .get("workflowLifecycleProofAssertion")
            .is_some()
    );
    assert!(service_spec.get("lifecycle").is_some());
    assert!(schema["$defs"].get("serviceLifecycle").is_some());
    assert!(schema["$defs"].get("workflowSeamObservation").is_some());
    assert!(schema["$defs"].get("workflowNegativeControl").is_some());
    assert!(workflow_instance_task_overlay.get("runtime").is_some());
    assert!(workflow_instance_runtime_overlay.get("listeners").is_some());
    assert!(workflow_instance_runtime_overlay.get("readiness").is_some());
    assert!(workflow_env.get("compose_env_file_services").is_some());
    assert!(workflow_env.get("adapter_inputs").is_some());
    assert!(workflow_env.get("compose_files").is_some());
    assert!(workflow_env.get("compose_project_name").is_some());
}

#[test]
fn workspace_contract_schema_is_published_and_covered_by_schema_publication() {
    let schema_path = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/spec/json-schemas/workspace-contract.json");
    assert!(
        schema_path.exists(),
        "workspace contract schema must be published at docs/spec/json-schemas/workspace-contract.json"
    );

    let workflow_path =
        Path::new(env!("CARGO_MANIFEST_DIR")).join(".github/workflows/release-gate.yml");
    let workflow = fs::read_to_string(&workflow_path).expect("workflow should be readable");

    assert!(
        workflow.contains("find docs/spec/json-schemas -maxdepth 1 -type f"),
        "workspace contract schema publication should stay on the same generated schema publication path"
    );
    let schema = load_schema("docs/spec/json-schemas/workspace-contract.json");
    assert_eq!(
        schema.get("$schema").and_then(|value| value.as_str()),
        Some("https://json-schema.org/draft/2020-12/schema")
    );
    assert!(schema["properties"].get("workspace").is_some());
    assert!(schema["properties"].get("repos").is_some());
    assert!(schema["properties"].get("policies").is_some());
    assert!(
        schema["$defs"]["workspaceRepoSpec"]["properties"]
            .get("workflow")
            .is_some()
    );
    assert!(
        schema["$defs"]["workspaceRepoSource"]["oneOf"]
            .as_array()
            .expect("workspace repo source variants")
            .iter()
            .any(|variant| variant["properties"].get("git").is_some())
    );
    assert!(
        schema["$defs"]["workspaceRepoSource"]["oneOf"]
            .as_array()
            .expect("workspace repo source variants")
            .iter()
            .any(|variant| variant["properties"].get("repo").is_some())
    );
}

#[test]
fn published_contract_schema_files_are_generated_and_in_sync() {
    let schema_dir = Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs")
        .join("spec")
        .join("json-schemas");

    for generated in published_contract_schemas() {
        let path = schema_dir.join(generated.filename);
        let on_disk =
            fs::read_to_string(&path).unwrap_or_else(|error| panic!("{}: {error}", path.display()));
        assert_eq!(
            on_disk,
            generated.body,
            "{} is out of sync with the Rust-owned published schema generator; run `cargo run --bin sync_published_contract_schemas`",
            path.display()
        );

        let parsed = generated_contract_schema(generated.filename)
            .unwrap_or_else(|| panic!("generated schema {} should exist", generated.filename));
        let on_disk_parsed: Value =
            serde_json::from_str(&on_disk).expect("published schema on disk should parse");
        assert_eq!(
            parsed,
            on_disk_parsed,
            "{} should serialize to the same JSON value as the Rust-owned published schema generator",
            path.display()
        );
    }
}
