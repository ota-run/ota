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
//   You may not use this file except in compliance with the License.
//   Unless required by applicable law or agreed to in writing, software distributed under the
//   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
//   either express or implied. See the License for the specific language governing permissions
//   and limitations under the License.
//
//   If you need additional information or have any questions, please email: os@ota.run

use std::collections::BTreeSet;

use serde::{Deserialize, Serialize};

use crate::runner::ExecutionOverrides;
use crate::sandbox_policy::{
    SandboxLaneIdentity, SandboxPolicy, SandboxTargetPlatform, sandbox_policy_for_task,
    sandbox_policy_for_workflow,
};
use crate::schema::{Backend, Contract, Lifecycle};
use crate::semantic_identity::semantic_contract_identity;

pub(crate) const CROSSING_SCOPE_SCHEMA_VERSION: u32 = 2;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CrossingClassification {
    Routine,
    Escalated,
}

impl CrossingClassification {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Routine => "routine",
            Self::Escalated => "escalated",
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum CrossingBoundaryFamily {
    UnsafeTask,
    HeavierWorkflow,
}

impl CrossingBoundaryFamily {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::UnsafeTask => "unsafe_task",
            Self::HeavierWorkflow => "heavier_workflow",
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CrossingRequirement {
    pub required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<CrossingClassification>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub boundary_family: Option<CrossingBoundaryFamily>,
}

pub(crate) fn evaluate_crossing_requirement(
    effective_safe_for_agent: Option<bool>,
    refused: bool,
    unsafe_closure_tasks: &[String],
    lane_kind: &str,
) -> CrossingRequirement {
    if refused {
        return CrossingRequirement {
            required: None,
            classification: None,
            boundary_family: None,
        };
    }

    match effective_safe_for_agent {
        Some(true) => CrossingRequirement {
            required: Some(false),
            classification: Some(CrossingClassification::Routine),
            boundary_family: None,
        },
        Some(false) => CrossingRequirement {
            required: Some(true),
            classification: Some(CrossingClassification::Escalated),
            boundary_family: Some(if lane_kind == "task" || !unsafe_closure_tasks.is_empty() {
                CrossingBoundaryFamily::UnsafeTask
            } else {
                CrossingBoundaryFamily::HeavierWorkflow
            }),
        },
        None => CrossingRequirement {
            required: None,
            classification: None,
            boundary_family: None,
        },
    }
}

pub(crate) fn crossing_preflight_posture(
    effective_safe_for_agent: Option<bool>,
    refused: bool,
    unsafe_closure_tasks: &[String],
    lane_kind: &str,
) -> (Option<bool>, Option<String>, Option<String>) {
    let posture = evaluate_crossing_requirement(
        effective_safe_for_agent,
        refused,
        unsafe_closure_tasks,
        lane_kind,
    );
    (
        posture.required,
        posture
            .classification
            .map(|classification| classification.label().to_string()),
        posture
            .boundary_family
            .map(|family| family.label().to_string()),
    )
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CrossingExecutionSelection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<Backend>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub memory: Option<u64>,
    pub skip_dependencies: bool,
    /// The requested workflow behavior is an execution selector. It is absent for direct tasks.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_behavior: Option<String>,
    /// Normalized readiness wait selected for ordinary workflow execution.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready_timeout_seconds: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_target: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effect_overrides: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow_instance: Option<CrossingWorkflowInstanceSelection>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CrossingWorkflowInstanceSelection {
    pub selector: String,
    pub instance_identity: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prerequisite_instances: Vec<CrossingWorkflowInstanceInvocation>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CrossingWorkflowInstanceInvocation {
    pub selector: String,
    pub instance_identity: String,
}

impl CrossingExecutionSelection {
    fn from_overrides(
        overrides: ExecutionOverrides,
        effect_overrides: &[String],
        sandbox_target: Option<&str>,
        run_behavior: Option<&str>,
        ready_timeout_seconds: Option<u64>,
    ) -> Self {
        let mut effect_overrides = effect_overrides.to_vec();
        effect_overrides.sort();
        effect_overrides.dedup();
        Self {
            backend: overrides.backend,
            lifecycle: overrides.lifecycle,
            host_port: overrides.host_port,
            memory: overrides.memory,
            skip_dependencies: overrides.skip_deps,
            run_behavior: run_behavior.map(str::to_string),
            ready_timeout_seconds,
            sandbox_target: sandbox_target.map(str::to_string),
            effect_overrides,
            workflow_instance: None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CrossingSemanticScope {
    pub schema_version: u32,
    pub identity: String,
    pub contract_identity: String,
    pub lane: SandboxLaneIdentity,
    pub boundary_family: String,
    pub classification: String,
    pub target_platform: SandboxTargetPlatform,
    pub execution_graph_identity: String,
    pub breadth: CrossingScopeBreadth,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_invocations: Vec<CrossingProofInvocation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub proof_transaction_selection: Option<CrossingProofTransactionSelection>,
    pub segment_identities: Vec<String>,
    pub edge_identities: Vec<String>,
    pub execution_selection: CrossingExecutionSelection,
    pub input_identity_posture: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_dimensions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CrossingScopeBreadth {
    pub schema_version: u32,
    pub identity: String,
    pub closure_node_count: usize,
    pub closure_edge_count: usize,
    pub effect_categories: Vec<String>,
    pub resource_count: usize,
    pub resource_identities: Vec<String>,
}

/// One declared proof-only invocation. The identity preserves the declared role and order even
/// when multiple proof obligations reference the same task.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CrossingProofInvocation {
    pub id: String,
    pub kind: String,
    pub task: String,
    pub order: usize,
}

/// Selection details that alter the proof transaction without changing its workflow display name.
/// They are runner-derived from normalized command inputs before grant admission.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct CrossingProofTransactionSelection {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub selected_services: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub service_closure: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready_timeout_seconds: Option<u64>,
}

impl CrossingSemanticScope {
    pub(crate) fn complete(&self) -> bool {
        self.unknown_dimensions.is_empty()
    }
}

pub(crate) fn crossing_scope_for_task(
    contract: &Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
    task_inputs: &[String],
    effect_overrides: &[String],
    sandbox_target: Option<&str>,
    boundary_family: &str,
    classification: &str,
) -> Result<CrossingSemanticScope, String> {
    let policy = sandbox_policy_for_task(contract, task_name, overrides)?;
    crossing_scope_from_policy(
        policy,
        overrides,
        task_inputs,
        effect_overrides,
        sandbox_target,
        None,
        None,
        boundary_family,
        classification,
    )
}

pub(crate) fn crossing_scope_for_workflow(
    contract: &Contract,
    workflow_name: Option<&str>,
    overrides: ExecutionOverrides,
    effect_overrides: &[String],
    sandbox_target: Option<&str>,
    run_behavior: &str,
    ready_timeout_seconds: Option<u64>,
    boundary_family: &str,
    classification: &str,
) -> Result<CrossingSemanticScope, String> {
    let policy = sandbox_policy_for_workflow(contract, workflow_name, overrides)?;
    let scope = crossing_scope_from_policy(
        policy,
        overrides,
        &[],
        effect_overrides,
        sandbox_target,
        Some(run_behavior),
        ready_timeout_seconds,
        boundary_family,
        classification,
    )?;
    crossing_scope_with_workflow_instance_selection(scope, contract, workflow_name)
}

pub(crate) fn crossing_scope_with_workflow_instance_selection(
    mut scope: CrossingSemanticScope,
    contract: &Contract,
    workflow_name: Option<&str>,
) -> Result<CrossingSemanticScope, String> {
    let Some((workflow_key, _)) = contract.selected_workflow(workflow_name) else {
        return Ok(scope);
    };
    let Some(selected) = contract.resolved_selected_workflow_instance(workflow_name) else {
        return Ok(scope);
    };
    let selector = format!("{workflow_key}@{}", selected.name);
    let instance_identity = semantic_contract_identity(&(
        "crossing_workflow_instance_v1",
        selector.as_str(),
        &selected.spec,
    ))?;
    let mut prerequisite_instances = Vec::new();
    for prerequisite_selector in
        contract.selected_workflow_instance_prerequisite_selectors(workflow_name)
    {
        let prerequisite = contract
            .resolved_selected_workflow_instance(Some(prerequisite_selector.as_str()))
            .ok_or_else(|| {
                format!(
                    "workflow instance prerequisite `{prerequisite_selector}` cannot be resolved"
                )
            })?;
        prerequisite_instances.push(CrossingWorkflowInstanceInvocation {
            instance_identity: semantic_contract_identity(&(
                "crossing_workflow_instance_v1",
                prerequisite_selector.as_str(),
                &prerequisite.spec,
            ))?,
            selector: prerequisite_selector,
        });
    }
    scope.lane.name = selector.clone();
    scope.execution_selection.workflow_instance = Some(CrossingWorkflowInstanceSelection {
        selector,
        instance_identity,
        prerequisite_instances,
    });
    scope.identity = semantic_contract_identity(&scope)?;
    Ok(scope)
}

pub(crate) fn crossing_scope_from_policy(
    policy: SandboxPolicy,
    overrides: ExecutionOverrides,
    task_inputs: &[String],
    effect_overrides: &[String],
    sandbox_target: Option<&str>,
    run_behavior: Option<&str>,
    ready_timeout_seconds: Option<u64>,
    boundary_family: &str,
    classification: &str,
) -> Result<CrossingSemanticScope, String> {
    let mut unknown_dimensions = Vec::new();
    let input_identity_posture = if task_inputs.is_empty() {
        String::from("not_applicable")
    } else {
        unknown_dimensions.push(String::from("task_input_value_identity"));
        String::from("unknown_secret_posture")
    };
    let breadth = crossing_scope_breadth(&policy)?;
    let mut scope = CrossingSemanticScope {
        schema_version: CROSSING_SCOPE_SCHEMA_VERSION,
        identity: String::new(),
        contract_identity: policy.contract_identity.clone(),
        lane: policy.lane,
        boundary_family: boundary_family.to_string(),
        classification: classification.to_string(),
        target_platform: policy.target_platform,
        execution_graph_identity: policy.identity,
        breadth,
        proof_invocations: Vec::new(),
        proof_transaction_selection: None,
        segment_identities: policy
            .segments
            .into_iter()
            .map(|segment| segment.identity)
            .collect(),
        edge_identities: policy.edges.into_iter().map(|edge| edge.identity).collect(),
        execution_selection: CrossingExecutionSelection::from_overrides(
            overrides,
            effect_overrides,
            sandbox_target,
            run_behavior,
            ready_timeout_seconds,
        ),
        input_identity_posture,
        unknown_dimensions,
    };
    scope.identity = semantic_contract_identity(&scope)?;
    Ok(scope)
}

fn crossing_scope_breadth(policy: &SandboxPolicy) -> Result<CrossingScopeBreadth, String> {
    let mut effect_categories = BTreeSet::new();
    let mut resource_identities = BTreeSet::new();
    let mut add_resource = |segment_id: &str, kind: &str, value: &str| -> Result<(), String> {
        effect_categories.insert(kind.to_string());
        resource_identities.insert(semantic_contract_identity(&(
            "crossing_scope_resource_v1",
            segment_id,
            kind,
            value,
        ))?);
        Ok(())
    };

    for segment in &policy.segments {
        if let Some(image) = segment.runtime_image.as_deref() {
            add_resource(segment.id.as_str(), "runtime_image", image)?;
        }
        for value in &segment.effects.writes {
            add_resource(segment.id.as_str(), "repo_write", value)?;
        }
        for value in &segment.effects.workspace_writes {
            add_resource(segment.id.as_str(), "workspace_write", value)?;
        }
        if segment.effects.network {
            let kind = serde_json::to_value(segment.effects.effective_network_kind())
                .map_err(|error| format!("failed to serialize network effect: {error}"))?;
            add_resource(segment.id.as_str(), "network", kind.to_string().as_str())?;
        }
        for value in &segment.effects.adapter_state {
            add_resource(segment.id.as_str(), "adapter_state", value)?;
        }
        for value in &segment.effects.external_state {
            add_resource(segment.id.as_str(), "external_state", value)?;
        }
        for value in &segment.inherited_service_networks {
            add_resource(segment.id.as_str(), "service_network", value)?;
        }
        for value in &segment.isolated_paths {
            add_resource(segment.id.as_str(), "isolated_path", value)?;
        }
        for value in &segment.pre_boundary_actions {
            add_resource(segment.id.as_str(), "pre_boundary_action", value)?;
        }
        if !segment.runtime_boundary.is_empty() {
            let boundary = serde_json::to_string(&segment.runtime_boundary)
                .map_err(|error| format!("failed to serialize runtime boundary: {error}"))?;
            add_resource(segment.id.as_str(), "runtime_boundary", boundary.as_str())?;
        }
    }

    let mut breadth = CrossingScopeBreadth {
        schema_version: 1,
        identity: String::new(),
        closure_node_count: policy.segments.len(),
        closure_edge_count: policy.edges.len(),
        effect_categories: effect_categories.into_iter().collect(),
        resource_count: resource_identities.len(),
        resource_identities: resource_identities.into_iter().collect(),
    };
    breadth.identity = semantic_contract_identity(&breadth)?;
    Ok(breadth)
}

pub(crate) fn crossing_scope_with_proof_invocations(
    mut scope: CrossingSemanticScope,
    proof_invocations: Vec<CrossingProofInvocation>,
) -> Result<CrossingSemanticScope, String> {
    scope.proof_invocations = proof_invocations;
    scope.identity = semantic_contract_identity(&scope)?;
    Ok(scope)
}

pub(crate) fn crossing_scope_with_proof_transaction_selection(
    mut scope: CrossingSemanticScope,
    selection: CrossingProofTransactionSelection,
) -> Result<CrossingSemanticScope, String> {
    scope.proof_transaction_selection = Some(selection);
    scope.identity = semantic_contract_identity(&scope)?;
    Ok(scope)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn contract(source: &str) -> Contract {
        serde_yaml::from_str(source).expect("contract should parse")
    }

    #[test]
    fn requirement_is_derived_from_effective_closure_safety() {
        let safe = evaluate_crossing_requirement(Some(true), false, &[], "task");
        assert_eq!(safe.required, Some(false));
        assert_eq!(safe.classification, Some(CrossingClassification::Routine));

        let unsafe_task = evaluate_crossing_requirement(
            Some(false),
            false,
            &[String::from("publish")],
            "workflow",
        );
        assert_eq!(unsafe_task.required, Some(true));
        assert_eq!(
            unsafe_task.boundary_family,
            Some(CrossingBoundaryFamily::UnsafeTask)
        );

        let refused = evaluate_crossing_requirement(Some(false), true, &[], "workflow");
        assert_eq!(refused.required, None);
    }

    #[test]
    fn task_scope_binds_hooks_platform_and_runtime_overrides() {
        let contract = contract(
            r#"
version: 1
project:
  name: crossing-scope
tasks:
  verify:
    command:
      exe: sh
      args: [-c, "printf verify"]
    after_success: [report]
    safe_for_agent: false
  report:
    command:
      exe: sh
      args: [-c, "printf report"]
    safe_for_agent: false
"#,
        );
        let baseline = crossing_scope_for_task(
            &contract,
            "verify",
            ExecutionOverrides::default(),
            &[],
            &[],
            None,
            "unsafe_task",
            "escalated",
        )
        .expect("baseline scope");
        let container = crossing_scope_for_task(
            &contract,
            "verify",
            ExecutionOverrides {
                backend: Some(Backend::Container),
                lifecycle: Some(Lifecycle::Ephemeral),
                ..ExecutionOverrides::default()
            },
            &[],
            &[],
            Some("oci_local"),
            "unsafe_task",
            "escalated",
        )
        .expect("container scope");
        let reclassified = crossing_scope_for_task(
            &contract,
            "verify",
            ExecutionOverrides::default(),
            &[],
            &[],
            None,
            "unsafe_task",
            "routine",
        )
        .expect("reclassified scope");
        assert_ne!(baseline.identity, container.identity);
        assert_ne!(baseline.identity, reclassified.identity);
        assert!(!baseline.edge_identities.is_empty());
        assert!(baseline.complete());
        let mut unsigned = baseline.clone();
        unsigned.identity.clear();
        assert_eq!(
            baseline.identity,
            semantic_contract_identity(&unsigned).expect("scope identity should re-derive")
        );
    }

    #[test]
    fn free_form_task_inputs_make_grant_scope_incomplete_without_hashing_values() {
        let contract = contract(
            r#"
version: 1
project:
  name: crossing-input-scope
tasks:
  publish:
    command:
      exe: sh
      args: [-c, "printf publish"]
    inputs:
      token:
        required: true
    safe_for_agent: false
"#,
        );
        let scope = crossing_scope_for_task(
            &contract,
            "publish",
            ExecutionOverrides::default(),
            &[String::from("--token"), String::from("secret")],
            &[],
            None,
            "unsafe_task",
            "escalated",
        )
        .expect("scope should be representable");
        assert!(!scope.complete());
        assert_eq!(
            scope.unknown_dimensions,
            vec![String::from("task_input_value_identity")]
        );
        let serialized = serde_json::to_string(&scope).expect("scope should serialize");
        assert!(!serialized.contains("\"secret\""));
    }

    #[test]
    fn workflow_scope_binds_selected_graph_and_effect_overrides() {
        let contract = contract(
            r#"
version: 1
project:
  name: crossing-workflow-scope
tasks:
  verify:
    command:
      exe: sh
      args: [-c, "printf verify"]
    safe_for_agent: false
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        );
        let baseline = crossing_scope_for_workflow(
            &contract,
            Some("verify"),
            ExecutionOverrides::default(),
            &[],
            None,
            "auto",
            None,
            "heavier_workflow",
            "escalated",
        )
        .expect("baseline scope");
        let overridden = crossing_scope_for_workflow(
            &contract,
            Some("verify"),
            ExecutionOverrides::default(),
            &[String::from("network:broad=allow")],
            None,
            "auto",
            None,
            "heavier_workflow",
            "escalated",
        )
        .expect("overridden scope");
        let detached = crossing_scope_for_workflow(
            &contract,
            Some("verify"),
            ExecutionOverrides::default(),
            &[],
            None,
            "detach",
            None,
            "heavier_workflow",
            "escalated",
        )
        .expect("detached scope");
        let bounded_timeout = crossing_scope_for_workflow(
            &contract,
            Some("verify"),
            ExecutionOverrides::default(),
            &[],
            None,
            "auto",
            Some(60),
            "heavier_workflow",
            "escalated",
        )
        .expect("timeout-bound scope");
        assert_ne!(baseline.identity, overridden.identity);
        assert_ne!(baseline.identity, detached.identity);
        assert_ne!(baseline.identity, bounded_timeout.identity);
        assert!(baseline.complete());
        assert_eq!(baseline.breadth.schema_version, 1);
        assert_eq!(baseline.breadth.closure_node_count, 1);
        assert_eq!(baseline.breadth.closure_edge_count, 0);
        assert_eq!(
            baseline.breadth.resource_count,
            baseline.breadth.resource_identities.len()
        );
        assert!(
            baseline
                .breadth
                .resource_identities
                .iter()
                .all(|identity| identity.starts_with("sha256:"))
        );
    }

    #[test]
    fn workflow_scope_binds_selected_instance_and_ordered_prerequisite_closure() {
        let contract = contract(
            r#"
version: 1
project:
  name: crossing-workflow-instance-scope
tasks:
  dev:
    run: npm run dev
    safe_for_agent: false
workflows:
  default: app
  app:
    run:
      task: dev
    instances:
      default: west
      base: {}
      west:
        topology:
          requires_instances: [base]
        env:
          REGION: west
      east:
        env:
          REGION: east
"#,
        );
        let west = crossing_scope_for_workflow(
            &contract,
            Some("app@west"),
            ExecutionOverrides::default(),
            &[],
            None,
            "auto",
            None,
            "heavier_workflow",
            "escalated",
        )
        .expect("west scope");
        let east = crossing_scope_for_workflow(
            &contract,
            Some("app@east"),
            ExecutionOverrides::default(),
            &[],
            None,
            "auto",
            None,
            "heavier_workflow",
            "escalated",
        )
        .expect("east scope");
        let default_west = crossing_scope_for_workflow(
            &contract,
            Some("app"),
            ExecutionOverrides::default(),
            &[],
            None,
            "auto",
            None,
            "heavier_workflow",
            "escalated",
        )
        .expect("default instance scope");

        assert_eq!(west, default_west);
        assert_ne!(west.identity, east.identity);
        assert_eq!(west.lane.name, "app@west");
        let selected = west
            .execution_selection
            .workflow_instance
            .expect("workflow instance selection");
        assert_eq!(selected.selector, "app@west");
        assert_eq!(selected.prerequisite_instances.len(), 1);
        assert_eq!(selected.prerequisite_instances[0].selector, "app@base");
    }

    #[test]
    fn proof_invocation_scope_binds_role_order_and_duplicate_task_uses() {
        let contract = contract(
            r#"
version: 1
project:
  name: crossing-proof-invocations
tasks:
  verify:
    command:
      exe: sh
      args: [-c, "printf verify"]
    safe_for_agent: false
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        );
        let baseline = crossing_scope_for_workflow(
            &contract,
            Some("verify"),
            ExecutionOverrides::default(),
            &[],
            None,
            "auto",
            None,
            "heavier_workflow",
            "escalated",
        )
        .expect("workflow scope");
        let observer_then_control = crossing_scope_with_proof_invocations(
            baseline.clone(),
            vec![
                CrossingProofInvocation {
                    id: String::from("seam_observation:database"),
                    kind: String::from("seam_observation"),
                    task: String::from("verify"),
                    order: 0,
                },
                CrossingProofInvocation {
                    id: String::from("negative_control:database-down"),
                    kind: String::from("negative_control"),
                    task: String::from("verify"),
                    order: 1,
                },
            ],
        )
        .expect("proof scope");
        let control_then_observer = crossing_scope_with_proof_invocations(
            baseline,
            vec![
                CrossingProofInvocation {
                    id: String::from("negative_control:database-down"),
                    kind: String::from("negative_control"),
                    task: String::from("verify"),
                    order: 0,
                },
                CrossingProofInvocation {
                    id: String::from("seam_observation:database"),
                    kind: String::from("seam_observation"),
                    task: String::from("verify"),
                    order: 1,
                },
            ],
        )
        .expect("proof scope");

        assert_ne!(
            observer_then_control.identity,
            control_then_observer.identity
        );
        assert_eq!(observer_then_control.proof_invocations.len(), 2);
        assert_eq!(
            observer_then_control.proof_invocations[0].kind,
            "seam_observation"
        );
    }

    #[test]
    fn proof_transaction_selection_binds_services_and_readiness_timeout() {
        let contract = contract(
            r#"
version: 1
project:
  name: crossing-proof-selection
tasks:
  verify:
    command:
      exe: sh
      args: [-c, "printf verify"]
    safe_for_agent: false
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        );
        let base = crossing_scope_for_workflow(
            &contract,
            Some("verify"),
            ExecutionOverrides::default(),
            &[],
            None,
            "lifecycle_proof",
            None,
            "unsafe_task",
            "escalated",
        )
        .expect("workflow scope");
        let database = crossing_scope_with_proof_transaction_selection(
            base.clone(),
            CrossingProofTransactionSelection {
                selected_services: vec![String::from("database")],
                service_closure: vec![String::from("database")],
                ready_timeout_seconds: None,
            },
        )
        .expect("database scope");
        let cache = crossing_scope_with_proof_transaction_selection(
            base.clone(),
            CrossingProofTransactionSelection {
                selected_services: vec![String::from("cache")],
                service_closure: vec![String::from("cache")],
                ready_timeout_seconds: None,
            },
        )
        .expect("cache scope");
        let short_timeout = crossing_scope_with_proof_transaction_selection(
            base.clone(),
            CrossingProofTransactionSelection {
                selected_services: Vec::new(),
                service_closure: Vec::new(),
                ready_timeout_seconds: Some(30),
            },
        )
        .expect("short timeout scope");
        let long_timeout = crossing_scope_with_proof_transaction_selection(
            base,
            CrossingProofTransactionSelection {
                selected_services: Vec::new(),
                service_closure: Vec::new(),
                ready_timeout_seconds: Some(90),
            },
        )
        .expect("long timeout scope");

        assert_ne!(database.identity, cache.identity);
        assert_ne!(short_timeout.identity, long_timeout.identity);
    }
}
