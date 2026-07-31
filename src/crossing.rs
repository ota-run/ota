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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_target: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effect_overrides: Vec<String>,
}

impl CrossingExecutionSelection {
    fn from_overrides(
        overrides: ExecutionOverrides,
        effect_overrides: &[String],
        sandbox_target: Option<&str>,
        run_behavior: Option<&str>,
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
            sandbox_target: sandbox_target.map(str::to_string),
            effect_overrides,
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
    pub segment_identities: Vec<String>,
    pub edge_identities: Vec<String>,
    pub execution_selection: CrossingExecutionSelection,
    pub input_identity_posture: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unknown_dimensions: Vec<String>,
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
    boundary_family: &str,
    classification: &str,
) -> Result<CrossingSemanticScope, String> {
    let policy = sandbox_policy_for_workflow(contract, workflow_name, overrides)?;
    crossing_scope_from_policy(
        policy,
        overrides,
        &[],
        effect_overrides,
        sandbox_target,
        Some(run_behavior),
        boundary_family,
        classification,
    )
}

fn crossing_scope_from_policy(
    policy: SandboxPolicy,
    overrides: ExecutionOverrides,
    task_inputs: &[String],
    effect_overrides: &[String],
    sandbox_target: Option<&str>,
    run_behavior: Option<&str>,
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
    let mut scope = CrossingSemanticScope {
        schema_version: CROSSING_SCOPE_SCHEMA_VERSION,
        identity: String::new(),
        contract_identity: policy.contract_identity.clone(),
        lane: policy.lane,
        boundary_family: boundary_family.to_string(),
        classification: classification.to_string(),
        target_platform: policy.target_platform,
        execution_graph_identity: policy.identity,
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
        ),
        input_identity_posture,
        unknown_dimensions,
    };
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
            "heavier_workflow",
            "escalated",
        )
        .expect("detached scope");
        assert_ne!(baseline.identity, overridden.identity);
        assert_ne!(baseline.identity, detached.identity);
        assert!(baseline.complete());
    }
}
