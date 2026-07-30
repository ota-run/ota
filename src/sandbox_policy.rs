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

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Component, Path, PathBuf};

use serde::{Deserialize, Serialize};

use crate::policy_pack::{
    LoadedOrgPolicyPack, PolicySandboxFilesystemRules, PolicySandboxNetworkRules,
    PolicySandboxRules,
};
use crate::runner::{
    ExecutionOverrides, TaskExecutionRelation, effective_task_execution,
    plan_task_execution_with_overrides,
};
use crate::schema::{
    Backend, CommandInteractionPosture, Contract, Lifecycle,
    RuntimeBoundaryDestinationConstraintEnforcement, RuntimeBoundaryFilesystemSpec,
    RuntimeBoundaryNetworkDefault, RuntimeBoundaryNetworkSpec, RuntimeBoundaryRepoRootMode,
    RuntimeBoundarySpec, TaskEffectsSpec,
};
use crate::semantic_identity::semantic_contract_identity;

pub(crate) const SANDBOX_POLICY_SCHEMA_VERSION: u32 = 1;
pub(crate) const OCI_LOCAL_TARGET: &str = "oci_local";
pub(crate) const OCI_LOCAL_ADAPTER_VERSION: &str = "ota-oci-local-v1";

fn is_sha256_identity(value: &str) -> bool {
    value.len() == 71
        && value.starts_with("sha256:")
        && value[7..]
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxPolicy {
    pub schema_version: u32,
    pub identity: String,
    pub contract_identity: String,
    pub lane: SandboxLaneIdentity,
    pub target_platform: SandboxTargetPlatform,
    pub segments: Vec<SandboxPolicySegment>,
    pub edges: Vec<SandboxPolicyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxLaneIdentity {
    pub kind: SandboxLaneKind,
    pub name: String,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxLaneKind {
    Task,
    Workflow,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxTargetPlatform {
    pub os: String,
    pub architecture: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxPolicySegment {
    pub id: String,
    pub identity: String,
    pub task: String,
    pub phase: SandboxPolicyPhase,
    pub order: usize,
    pub target_platform: SandboxTargetPlatform,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime_image: Option<String>,
    pub backend: Backend,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    pub interaction: CommandInteractionPosture,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inherited_service_networks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub isolated_paths: Vec<String>,
    /// User-controlled setup that the initial OCI adapter cannot place inside the task command
    /// boundary. Admission must refuse these lanes instead of claiming the later command boundary
    /// covered earlier materialization.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_boundary_actions: Vec<String>,
    pub execution_kind: String,
    pub runtime_boundary: RuntimeBoundarySpec,
    pub effects: TaskEffectsSpec,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxPolicyPhase {
    Prepare,
    Setup,
    Run,
    Attach,
    Dependency,
    Hook,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxPolicyEdge {
    pub identity: String,
    pub source: String,
    pub destination: String,
    pub condition: SandboxPolicyEdgeCondition,
    pub order: usize,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxPolicyEdgeCondition {
    Unconditional,
    OnSuccess,
    OnFailure,
    Always,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxRestrictionOverlay {
    pub identity: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filesystem: Option<PolicySandboxFilesystemRules>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network: Option<PolicySandboxNetworkRules>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxRestrictionAuthority {
    pub identity: String,
    pub source: String,
    pub source_identity: String,
    pub rules: PolicySandboxRules,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct EffectiveSandboxPolicy {
    pub identity: String,
    pub canonical_identity: String,
    pub restriction_overlays: Vec<SandboxRestrictionOverlay>,
    pub restriction_overlay_identities: Vec<String>,
    pub segments: Vec<SandboxPolicySegment>,
    pub edges: Vec<SandboxPolicyEdge>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProviderCapabilityDescriptor {
    pub identity: String,
    pub target: String,
    pub adapter_version: String,
    pub target_platform: SandboxTargetPlatform,
    pub controls: BTreeMap<String, ProviderControlCapability>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum ProviderControlCapability {
    Enforced,
    Advisory,
    Unsupported,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProviderApplicationEvaluation {
    pub admitted: bool,
    pub identity: String,
    pub provider_target: String,
    pub canonical_policy_identity: String,
    pub effective_policy_identity: String,
    pub capability_identity: String,
    pub segment_applications: Vec<ProviderSegmentApplication>,
    pub refusals: Vec<SandboxAdmissionRefusal>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct ProviderSegmentApplication {
    pub segment_id: String,
    pub segment_policy_identity: String,
    pub filesystem: ProviderControlCapability,
    pub network: ProviderControlCapability,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxAdmissionRefusal {
    pub segment_id: String,
    pub control: String,
    pub required: String,
    pub capability: ProviderControlCapability,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OciLocalSegmentPlan {
    pub segment_id: String,
    pub segment_policy_identity: String,
    pub task: String,
    pub declared_image: String,
    pub read_only_repo_root: bool,
    pub writable_paths: Vec<String>,
    pub protected_paths: Vec<String>,
    pub isolated_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pre_boundary_actions: Vec<String>,
    pub execution_kind: String,
    pub deny_external_network: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub platform: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct OciLocalApplicationPlan {
    pub identity: String,
    pub lane: SandboxLaneIdentity,
    pub execution_selection: SandboxExecutionSelection,
    pub canonical_policy_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restriction_authority: Option<SandboxRestrictionAuthority>,
    pub restriction_overlays: Vec<SandboxRestrictionOverlay>,
    pub restriction_overlay_identities: Vec<String>,
    pub effective_policy_identity: String,
    pub capability_identity: String,
    pub target_platform: SandboxTargetPlatform,
    pub segments: Vec<OciLocalSegmentPlan>,
    pub edges: Vec<SandboxPolicyEdge>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxExecutionSelection {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<Backend>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<Lifecycle>,
    pub skip_dependencies: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxApplicationEvidence {
    pub schema_version: u32,
    pub lane: SandboxLaneIdentity,
    pub execution_selection: SandboxExecutionSelection,
    pub canonical_policy_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub restriction_authority: Option<SandboxRestrictionAuthority>,
    pub restriction_overlays: Vec<SandboxRestrictionOverlay>,
    pub restriction_overlay_identities: Vec<String>,
    pub effective_policy_identity: String,
    pub target_platform: SandboxTargetPlatform,
    pub provider_target: String,
    pub provider_adapter_version: String,
    pub capability_identity: String,
    pub application_plan_identity: String,
    pub runner_transaction_identity: String,
    pub started_at: String,
    pub attestation: SandboxLocalAttestationEvidence,
    pub status: SandboxApplicationStatus,
    #[serde(default)]
    pub admitted_edge_identities: Vec<String>,
    #[serde(default)]
    pub admitted_segments: Vec<OciLocalSegmentPlan>,
    #[serde(default)]
    pub admitted_edges: Vec<SandboxPolicyEdge>,
    #[serde(default)]
    pub selected_edges: Vec<SandboxSelectedEdgeEvidence>,
    #[serde(default)]
    pub segments: Vec<SandboxSegmentApplicationEvidence>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxLocalAttestationEvidence {
    pub issuer: String,
    pub trust: String,
    pub challenge_identity: String,
    pub verifier: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxSelectedEdgeEvidence {
    pub identity: String,
    pub edge_identity: String,
    pub source: String,
    pub destination: String,
    /// The segment whose execution selected this edge. Dependency edges point child -> parent,
    /// while hooks and workflow phase transitions point parent -> child, so destination alone is
    /// not an execution identity.
    pub executed_segment: String,
    pub condition: SandboxPolicyEdgeCondition,
    pub edge_order: usize,
    pub generation: usize,
    pub state: SandboxSelectedEdgeState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_generation: Option<usize>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxSelectedEdgeState {
    Entered,
    Skipped,
    Reused,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxApplicationStatus {
    NotStarted,
    EnforcedThroughCompletion,
    EnforcementLost,
    EnforcementUnknownAfterInterruption,
}

#[derive(
    Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Hash, Default,
)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxSegmentApplicationPurpose {
    #[default]
    TaskExecution,
    PreconditionProbe,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxSegmentApplicationEvidence {
    pub segment_id: String,
    pub segment_policy_identity: String,
    #[serde(default)]
    pub purpose: SandboxSegmentApplicationPurpose,
    pub invocation_generation: usize,
    pub cleanup_lease_identity: String,
    pub boundary_identity: String,
    pub declared_image: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_image_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_platform: Option<String>,
    pub rendered_policy_identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub applied_policy_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_application_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_observation_identity: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub writable_mounts: Vec<SandboxWritableMountEvidence>,
    pub filesystem: SandboxControlApplicationEvidence,
    pub network: SandboxControlApplicationEvidence,
    pub cleanup: SandboxCleanupEvidence,
    pub status: SandboxSegmentApplicationStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxWritableMountEvidence {
    pub path: String,
    pub source_identity: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxControlApplicationEvidence {
    pub required: String,
    pub application: SandboxControlApplicationState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub evidence_identity: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxControlApplicationState {
    Pending,
    Enforced,
    NotRequired,
    NotProved,
    Contradicted,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub(crate) struct SandboxCleanupEvidence {
    pub authority: String,
    pub state: SandboxCleanupState,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub terminal_identity: Option<String>,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxCleanupState {
    Registered,
    Confirmed,
    Incomplete,
    Unknown,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub(crate) enum SandboxSegmentApplicationStatus {
    LeaseRegistered,
    Applied,
    EnforcementLost,
    EnforcementUnknownAfterInterruption,
    EnforcedThroughCompletion,
}

#[derive(Debug, Clone)]
struct PolicyGraphBuilder<'a> {
    contract: &'a Contract,
    overrides: ExecutionOverrides,
    segments: BTreeMap<String, SandboxPolicySegment>,
    edges: Vec<(String, String, SandboxPolicyEdgeCondition)>,
}

impl<'a> PolicyGraphBuilder<'a> {
    fn new(contract: &'a Contract, overrides: ExecutionOverrides) -> Self {
        Self {
            contract,
            overrides,
            segments: BTreeMap::new(),
            edges: Vec::new(),
        }
    }

    fn add_root(&mut self, task: &str, phase: SandboxPolicyPhase) -> Result<(), String> {
        let plan = plan_task_execution_with_overrides(self.contract, task, self.overrides)
            .map_err(|error| error.to_string())?;
        for step in &plan.steps {
            let mut candidate_phases = Vec::new();
            if step.task == task {
                candidate_phases.push(phase);
            }
            for edge in &plan.edges {
                let candidate = match edge.relation {
                    TaskExecutionRelation::AfterSuccess { .. }
                    | TaskExecutionRelation::AfterFailure { .. }
                    | TaskExecutionRelation::AfterAlways { .. }
                        if edge.destination == step.task =>
                    {
                        Some(SandboxPolicyPhase::Hook)
                    }
                    TaskExecutionRelation::DependsOn { .. }
                    | TaskExecutionRelation::AggregateMember { .. }
                        if edge.source == step.task =>
                    {
                        Some(SandboxPolicyPhase::Dependency)
                    }
                    _ => None,
                };
                if let Some(candidate) = candidate
                    && !candidate_phases.contains(&candidate)
                {
                    candidate_phases.push(candidate);
                }
            }
            if candidate_phases.is_empty() {
                candidate_phases.push(SandboxPolicyPhase::Dependency);
            }
            if candidate_phases.len() != 1 {
                return Err(format!(
                    "sandbox policy cannot represent task `{}` reused across multiple execution phases; split the repeated invocation into distinct task identities",
                    step.task
                ));
            }
            let step_phase = candidate_phases[0];
            let existing_phase = self
                .segments
                .get(step.task.as_str())
                .map(|segment| segment.phase);
            if let Some(existing_phase) = existing_phase {
                if existing_phase != step_phase {
                    return Err(format!(
                        "sandbox policy cannot represent task `{}` reused as both {:?} and {:?}; split the repeated invocation into distinct task identities",
                        step.task, existing_phase, step_phase
                    ));
                }
                continue;
            }
            let task_spec =
                self.contract.tasks.get(step.task.as_str()).ok_or_else(|| {
                    format!("sandbox policy references unknown task `{}`", step.task)
                })?;
            let step_overrides = ExecutionOverrides {
                backend: Some(step.backend),
                lifecycle: self.overrides.lifecycle,
                host_port: self.overrides.host_port,
                memory: self.overrides.memory,
                skip_deps: self.overrides.skip_deps,
            };
            let effective =
                effective_task_execution(self.contract, step.task.as_str(), step_overrides);
            let segment_target_platform = target_platform(step.backend, effective.container);
            let target_os = segment_target_platform.os.as_str();
            let interaction = task_spec
                .resolved_execution_for_backend(step.backend, target_os)
                .and_then(|execution| execution.command())
                .and_then(|command| command.interaction)
                .unwrap_or_default();
            let attachments = step
                .context
                .as_deref()
                .and_then(|context_name| {
                    self.contract
                        .execution
                        .as_ref()
                        .and_then(|execution| execution.contexts.get(context_name))
                })
                .map(|context| context.attachments.clone())
                .unwrap_or_default();
            let execution = task_spec
                .resolved_execution_for_backend(step.backend, target_os)
                .ok_or_else(|| {
                    format!(
                        "sandbox policy cannot resolve execution body for task `{}`",
                        step.task
                    )
                })?;
            let mut pre_boundary_actions = Vec::new();
            if task_spec.prepare.is_some() {
                pre_boundary_actions.push(String::from("task_prepare"));
            }
            if !task_spec.requires_services.is_empty() {
                pre_boundary_actions.push(String::from("required_services"));
            }
            let context_name = step.context.as_deref();
            let requirement_surface = task_spec.scoped_requirement_surface_for_execution_for_os(
                step.backend,
                context_name,
                target_os,
            );
            let has_selected_requirements = !requirement_surface.runtimes.is_empty()
                || !requirement_surface.tools.is_empty()
                || !task_spec
                    .scoped_toolchain_requirements_for_execution_for_os(
                        step.backend,
                        context_name,
                        target_os,
                    )
                    .is_empty()
                || !task_spec
                    .scoped_native_requirements_for_execution_for_os(
                        step.backend,
                        context_name,
                        target_os,
                    )
                    .is_empty()
                || !task_spec
                    .scoped_env_requirements_for_execution_for_os(
                        step.backend,
                        context_name,
                        target_os,
                    )
                    .is_empty()
                || !task_spec
                    .scoped_check_requirements_for_execution_for_os(
                        step.backend,
                        context_name,
                        target_os,
                    )
                    .is_empty();
            if has_selected_requirements {
                pre_boundary_actions.push(String::from("task_requirements"));
            }
            if !task_spec.when.checks.is_empty() {
                pre_boundary_actions.push(String::from("task_when_checks"));
            }
            if let Some(context) = step.context.as_deref().and_then(|context_name| {
                self.contract
                    .execution
                    .as_ref()
                    .and_then(|root| root.contexts.get(context_name))
            }) && (!context.requirements.runtimes.is_empty()
                || !context.requirements.tools.is_empty()
                || !context.requirements.toolchains.is_empty())
            {
                pre_boundary_actions.push(String::from("context_requirements"));
            }
            pre_boundary_actions.sort();
            pre_boundary_actions.dedup();
            let mut segment = SandboxPolicySegment {
                id: format!("task:{}", step.task),
                identity: String::new(),
                task: step.task.clone(),
                phase: step_phase,
                order: 0,
                target_platform: segment_target_platform,
                runtime_image: effective.container.map(|container| container.image.clone()),
                backend: step.backend,
                context: step.context.clone(),
                lifecycle: effective.lifecycle,
                interaction,
                inherited_service_networks: attachments
                    .compose
                    .into_iter()
                    .map(|project| project.trim().to_string())
                    .filter(|project| !project.is_empty())
                    .map(|project| format!("{project}_default"))
                    .collect(),
                isolated_paths: attachments.isolated_paths,
                pre_boundary_actions,
                execution_kind: execution.kind.to_string(),
                runtime_boundary: effective_runtime_boundary_for_task(
                    self.contract,
                    step.task.as_str(),
                ),
                effects: task_spec.effects.clone(),
            };
            segment.identity = semantic_contract_identity(&segment)?;
            self.segments.insert(step.task.clone(), segment);
        }
        for edge in plan.edges {
            let condition = match edge.relation {
                TaskExecutionRelation::AfterSuccess { .. } => SandboxPolicyEdgeCondition::OnSuccess,
                TaskExecutionRelation::AfterFailure { .. } => SandboxPolicyEdgeCondition::OnFailure,
                TaskExecutionRelation::AfterAlways { .. } => SandboxPolicyEdgeCondition::Always,
                TaskExecutionRelation::Requested
                | TaskExecutionRelation::DependsOn { .. }
                | TaskExecutionRelation::AggregateMember { .. } => {
                    SandboxPolicyEdgeCondition::Unconditional
                }
            };
            let candidate = (edge.source, edge.destination, condition);
            if !self.edges.contains(&candidate) {
                self.edges.push(candidate);
            }
        }
        Ok(())
    }

    fn finish(mut self) -> Result<(Vec<SandboxPolicySegment>, Vec<SandboxPolicyEdge>), String> {
        let ordered_names = stable_topological_order(&self.segments, &self.edges)?;
        let order_by_name = ordered_names
            .iter()
            .enumerate()
            .map(|(order, name)| (name.clone(), order))
            .collect::<BTreeMap<_, _>>();
        let mut segments = Vec::with_capacity(ordered_names.len());
        for name in ordered_names {
            let mut segment = self
                .segments
                .remove(name.as_str())
                .ok_or_else(|| format!("sandbox policy segment `{name}` disappeared"))?;
            segment.order = *order_by_name
                .get(name.as_str())
                .ok_or_else(|| format!("sandbox policy segment `{name}` has no order"))?;
            segment.identity = semantic_contract_identity(&segment)?;
            segments.push(segment);
        }
        self.edges.sort_by(|left, right| {
            (
                order_by_name.get(left.0.as_str()),
                order_by_name.get(left.1.as_str()),
                left.2,
            )
                .cmp(&(
                    order_by_name.get(right.0.as_str()),
                    order_by_name.get(right.1.as_str()),
                    right.2,
                ))
        });
        let mut edges = Vec::with_capacity(self.edges.len());
        for (order, (source, destination, condition)) in self.edges.into_iter().enumerate() {
            let mut edge = SandboxPolicyEdge {
                identity: String::new(),
                source: format!("task:{source}"),
                destination: format!("task:{destination}"),
                condition,
                order,
            };
            edge.identity = semantic_contract_identity(&edge)?;
            edges.push(edge);
        }
        Ok((segments, edges))
    }

    fn apply_restrictive_boundary(&mut self, boundary: &RuntimeBoundarySpec) -> Result<(), String> {
        for segment in self.segments.values_mut() {
            if let Some(filesystem) = boundary.filesystem.as_ref() {
                let filesystem = runtime_filesystem_restriction(filesystem);
                segment.runtime_boundary.filesystem = Some(meet_filesystem(
                    segment.runtime_boundary.filesystem.as_ref(),
                    &filesystem,
                )?);
            }
            if let Some(network) = boundary.network.as_ref() {
                let network = runtime_network_restriction(network);
                segment.runtime_boundary.network = Some(meet_network(
                    segment.runtime_boundary.network.as_ref(),
                    &network,
                )?);
            }
            segment.identity = semantic_contract_identity(segment)?;
        }
        Ok(())
    }
}

fn runtime_filesystem_restriction(
    boundary: &RuntimeBoundaryFilesystemSpec,
) -> PolicySandboxFilesystemRules {
    PolicySandboxFilesystemRules {
        repo_root_mode: boundary.repo_root_mode,
        writable_paths: if boundary.writable_paths.is_empty()
            && boundary.repo_root_mode != Some(RuntimeBoundaryRepoRootMode::ReadOnly)
        {
            None
        } else {
            Some(boundary.writable_paths.clone())
        },
        protected_paths: (!boundary.protected_paths.is_empty())
            .then(|| boundary.protected_paths.clone()),
    }
}

fn runtime_network_restriction(boundary: &RuntimeBoundaryNetworkSpec) -> PolicySandboxNetworkRules {
    PolicySandboxNetworkRules {
        default: boundary.default,
        outbound_targets: if boundary.outbound_targets.is_empty()
            && boundary.default != Some(RuntimeBoundaryNetworkDefault::Deny)
        {
            None
        } else {
            Some(boundary.outbound_targets.clone())
        },
    }
}

pub(crate) fn sandbox_policy_for_task(
    contract: &Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Result<SandboxPolicy, String> {
    let mut builder = PolicyGraphBuilder::new(contract, overrides);
    builder.add_root(task_name, SandboxPolicyPhase::Run)?;
    finish_policy(
        contract,
        SandboxLaneIdentity {
            kind: SandboxLaneKind::Task,
            name: task_name.to_string(),
        },
        builder,
    )
}

pub(crate) fn sandbox_policy_for_workflow(
    contract: &Contract,
    workflow_name: Option<&str>,
    overrides: ExecutionOverrides,
) -> Result<SandboxPolicy, String> {
    let selected_name = contract
        .selected_workflow(workflow_name)
        .map(|(name, _)| name.to_string())
        .ok_or_else(|| String::from("sandbox policy requires a selected workflow"))?;
    if contract
        .selected_prepare_action_for(workflow_name)
        .is_some()
    {
        return Err(format!(
            "workflow `{selected_name}` uses a direct prepare action; V11.21 cannot apply a task-scoped provider boundary to that action, so sandbox admission refuses before preparation"
        ));
    }
    let mut builder = PolicyGraphBuilder::new(contract, overrides);
    let roots = [
        (
            contract.selected_prepare_task_name_for(workflow_name),
            SandboxPolicyPhase::Prepare,
        ),
        (
            contract.selected_setup_task_name_for(workflow_name),
            SandboxPolicyPhase::Setup,
        ),
        (
            contract.selected_run_task_name_for(workflow_name),
            SandboxPolicyPhase::Run,
        ),
        (
            contract.selected_attach_task_name_for(workflow_name),
            SandboxPolicyPhase::Attach,
        ),
    ];
    let mut previous = None::<String>;
    let mut phase_roots = BTreeSet::new();
    for (root, phase) in roots {
        let Some(root) = root else {
            continue;
        };
        if !phase_roots.insert(root.to_string()) {
            return Err(format!(
                "workflow `{selected_name}` reuses task `{root}` across multiple execution phases; the initial enforcing provider requires one unambiguous task segment per workflow phase"
            ));
        }
        builder.add_root(root, phase)?;
        if let Some(previous) = previous.as_ref()
            && previous != root
        {
            builder.edges.push((
                previous.clone(),
                root.to_string(),
                SandboxPolicyEdgeCondition::Unconditional,
            ));
        }
        previous = Some(root.to_string());
    }
    if let Some(boundary) = contract
        .selected_workflow(workflow_name)
        .and_then(|(_, workflow)| workflow.runtime_boundary.as_ref())
    {
        builder.apply_restrictive_boundary(boundary)?;
    }
    finish_policy(
        contract,
        SandboxLaneIdentity {
            kind: SandboxLaneKind::Workflow,
            name: selected_name,
        },
        builder,
    )
}

fn finish_policy(
    contract: &Contract,
    lane: SandboxLaneIdentity,
    builder: PolicyGraphBuilder<'_>,
) -> Result<SandboxPolicy, String> {
    let (segments, edges) = builder.finish()?;
    if segments.is_empty() {
        return Err(format!(
            "sandbox policy lane `{}` has no executable task segments",
            lane.name
        ));
    }
    let contract_identity = semantic_contract_identity(contract)?;
    let target_platform = common_target_platform(&segments);
    let mut policy = SandboxPolicy {
        schema_version: SANDBOX_POLICY_SCHEMA_VERSION,
        identity: String::new(),
        contract_identity,
        lane,
        target_platform,
        segments,
        edges,
    };
    policy.identity = semantic_contract_identity(&policy)?;
    Ok(policy)
}

pub(crate) fn effective_sandbox_policy(
    canonical: &SandboxPolicy,
    overlays: &[SandboxRestrictionOverlay],
) -> Result<EffectiveSandboxPolicy, String> {
    let mut overlays = overlays.to_vec();
    overlays.sort_by(|left, right| left.identity.cmp(&right.identity));
    let mut segments = canonical.segments.clone();
    for overlay in &overlays {
        for segment in &mut segments {
            if let Some(filesystem) = overlay.filesystem.as_ref() {
                segment.runtime_boundary.filesystem = Some(meet_filesystem(
                    segment.runtime_boundary.filesystem.as_ref(),
                    filesystem,
                )?);
            }
            if let Some(network) = overlay.network.as_ref() {
                segment.runtime_boundary.network = Some(meet_network(
                    segment.runtime_boundary.network.as_ref(),
                    network,
                )?);
            }
            segment.identity = semantic_contract_identity(segment)?;
        }
    }
    let mut effective = EffectiveSandboxPolicy {
        identity: String::new(),
        canonical_identity: canonical.identity.clone(),
        restriction_overlays: overlays.clone(),
        restriction_overlay_identities: overlays
            .iter()
            .map(|overlay| overlay.identity.clone())
            .collect(),
        segments,
        edges: canonical.edges.clone(),
    };
    effective.identity = semantic_contract_identity(&effective)?;
    Ok(effective)
}

pub(crate) fn restriction_overlays_from_loaded_policy(
    loaded_policy: Option<&LoadedOrgPolicyPack>,
) -> Result<Vec<SandboxRestrictionOverlay>, String> {
    let authority = restriction_authority_from_loaded_policy(loaded_policy)?;
    restriction_overlays_from_authority(authority.as_ref())
}

pub(crate) fn restriction_authority_from_loaded_policy(
    loaded_policy: Option<&LoadedOrgPolicyPack>,
) -> Result<Option<SandboxRestrictionAuthority>, String> {
    let Some(loaded_policy) = loaded_policy else {
        return Ok(None);
    };
    let Some(sandbox) = loaded_policy.pack.policies.sandbox.as_ref() else {
        return Ok(None);
    };
    let source_identity = semantic_contract_identity(sandbox)?;
    let mut authority = SandboxRestrictionAuthority {
        identity: String::new(),
        source: loaded_policy.source.as_str().to_string(),
        source_identity,
        rules: sandbox.clone(),
    };
    authority.identity = semantic_contract_identity(&authority)?;
    Ok(Some(authority))
}

fn restriction_overlays_from_authority(
    authority: Option<&SandboxRestrictionAuthority>,
) -> Result<Vec<SandboxRestrictionOverlay>, String> {
    let Some(authority) = authority else {
        return Ok(Vec::new());
    };
    let mut unsigned = authority.clone();
    let identity = std::mem::take(&mut unsigned.identity);
    if authority.source_identity != semantic_contract_identity(&authority.rules)?
        || identity != semantic_contract_identity(&unsigned)?
    {
        return Err(String::from(
            "sandbox restriction authority snapshot has an invalid identity",
        ));
    }
    let mut overlay = SandboxRestrictionOverlay {
        identity: String::new(),
        source: format!(
            "{}:{}:{}",
            authority.source, authority.source_identity, authority.identity
        ),
        filesystem: authority.rules.filesystem.clone(),
        network: authority.rules.network.clone(),
    };
    overlay.identity = semantic_contract_identity(&overlay)?;
    Ok(vec![overlay])
}

pub(crate) fn oci_local_capabilities(
    target_platform: &SandboxTargetPlatform,
) -> Result<ProviderCapabilityDescriptor, String> {
    let mut controls = BTreeMap::new();
    for control in [
        "filesystem.repo_root.read_only",
        "filesystem.writable_paths",
        "filesystem.protected_paths",
        "network.external_connectivity.deny",
        "process.isolation",
        "lifecycle.cleanup",
    ] {
        controls.insert(control.to_string(), ProviderControlCapability::Enforced);
    }
    for control in [
        "filesystem.managed_isolated_paths",
        "network.target_allowlist",
        "network.destination_constraints",
        "secret.binding",
    ] {
        controls.insert(control.to_string(), ProviderControlCapability::Unsupported);
    }
    let mut descriptor = ProviderCapabilityDescriptor {
        identity: String::new(),
        target: OCI_LOCAL_TARGET.to_string(),
        adapter_version: OCI_LOCAL_ADAPTER_VERSION.to_string(),
        target_platform: target_platform.clone(),
        controls,
    };
    descriptor.identity = semantic_contract_identity(&descriptor)?;
    Ok(descriptor)
}

pub(crate) fn evaluate_oci_local_application(
    canonical: &SandboxPolicy,
    effective: &EffectiveSandboxPolicy,
    restriction_authority: Option<&SandboxRestrictionAuthority>,
    repo_root: &Path,
    overrides: ExecutionOverrides,
) -> Result<
    (
        ProviderApplicationEvaluation,
        Option<OciLocalApplicationPlan>,
    ),
    String,
> {
    let capabilities = oci_local_capabilities(&canonical.target_platform)?;
    let mut refusals = Vec::new();
    let mut segment_applications = Vec::new();
    for segment in &effective.segments {
        let mut filesystem_state = ProviderControlCapability::Enforced;
        let mut network_state = ProviderControlCapability::Enforced;
        if segment.backend != Backend::Container {
            refusals.push(refusal(
                segment,
                "execution.backend",
                "container",
                ProviderControlCapability::Unsupported,
                "oci_local cannot change a native or remote task into container execution",
            ));
        }
        if segment.lifecycle != Some(Lifecycle::Ephemeral) {
            refusals.push(refusal(
                segment,
                "execution.lifecycle",
                "ephemeral",
                ProviderControlCapability::Unsupported,
                "oci_local requires an ephemeral selected task lifecycle so Ota can own terminal cleanup",
            ));
        }
        if segment.target_platform.platform.is_none() {
            refusals.push(refusal(
                segment,
                "execution.target_platform",
                "explicit_os_and_architecture",
                ProviderControlCapability::Unsupported,
                "oci_local requires an explicit container `platform` so provider defaults cannot become sandbox policy",
            ));
        }

        let filesystem = segment.runtime_boundary.filesystem.as_ref();
        let read_only_repo_root = filesystem.and_then(|entry| entry.repo_root_mode)
            == Some(RuntimeBoundaryRepoRootMode::ReadOnly);
        if !read_only_repo_root {
            filesystem_state = ProviderControlCapability::Unsupported;
            refusals.push(refusal(
                segment,
                "filesystem.repo_root_mode",
                "read_only",
                filesystem_state,
                "oci_local requires an explicit `repo_root_mode: read_only`; it does not infer a provider default or claim future-write denial under a writable repository root",
            ));
        }
        let writable_paths = filesystem
            .map(|entry| entry.writable_paths.clone())
            .unwrap_or_default();
        let protected_paths = filesystem
            .map(|entry| entry.protected_paths.clone())
            .unwrap_or_default();
        let isolated_paths = segment.isolated_paths.clone();
        for isolated_path in &isolated_paths {
            filesystem_state = ProviderControlCapability::Unsupported;
            refusals.push(refusal(
                segment,
                "filesystem.isolated_path",
                "transactionally_evidenced_provider_resource",
                filesystem_state,
                &format!(
                    "oci_local cannot yet create or reuse managed isolated path `{isolated_path}` \
                     inside the pre-mutation cleanup transaction; remove the isolated attachment \
                     from this enforced lane or use an execution path that does not claim sandbox \
                     application evidence"
                ),
            ));
        }
        if let Err(reason) =
            validate_oci_filesystem_paths(repo_root, &writable_paths, &protected_paths)
        {
            filesystem_state = ProviderControlCapability::Unsupported;
            refusals.push(refusal(
                segment,
                "filesystem.path_boundaries",
                "canonical_non_overlapping_paths",
                filesystem_state,
                &reason,
            ));
        }

        let network = segment.runtime_boundary.network.as_ref();
        let network_default = network.and_then(|entry| entry.default);
        let outbound_targets = network
            .map(|entry| entry.outbound_targets.as_slice())
            .unwrap_or_default();
        if !segment.pre_boundary_actions.is_empty() {
            refusals.push(refusal(
                segment,
                "execution.pre_boundary_actions",
                segment.pre_boundary_actions.join(", ").as_str(),
                ProviderControlCapability::Unsupported,
                "the first oci_local adapter cannot place this task's setup, service, or requirement materialization inside the enforced command boundary",
            ));
        }
        if !matches!(
            segment.execution_kind.as_str(),
            "run" | "script" | "command" | "aggregate"
        ) {
            refusals.push(refusal(
                segment,
                "execution.kind",
                segment.execution_kind.as_str(),
                ProviderControlCapability::Unsupported,
                "the first oci_local adapter only enforces finite command bodies; typed action, compose, launch, and prepare bodies require a provider-specific execution adapter",
            ));
        }
        if network_default == Some(RuntimeBoundaryNetworkDefault::Deny)
            && !segment.inherited_service_networks.is_empty()
        {
            network_state = ProviderControlCapability::Unsupported;
            refusals.push(refusal(
                segment,
                "network.inherited_service_network",
                "no_inherited_service_networks",
                network_state,
                "oci_local network denial cannot be combined with inherited Compose networks",
            ));
        }
        if !outbound_targets.is_empty() {
            network_state = ProviderControlCapability::Unsupported;
            refusals.push(refusal(
                segment,
                "network.target_allowlist",
                "authoritative_targeted_egress",
                network_state,
                "stock OCI networking cannot enforce host, service, or destination allowlists without a cooperating network-policy adapter",
            ));
        }
        if outbound_targets.iter().any(|target| {
            target
                .destination_constraint
                .as_ref()
                .is_some_and(|constraint| {
                    constraint.enforcement
                    == RuntimeBoundaryDestinationConstraintEnforcement::AuthoritativeRuntimeEnforced
                })
        }) {
            network_state = ProviderControlCapability::Unsupported;
        }

        segment_applications.push(ProviderSegmentApplication {
            segment_id: segment.id.clone(),
            segment_policy_identity: segment.identity.clone(),
            filesystem: filesystem_state,
            network: network_state,
        });
    }
    let mut evaluation = ProviderApplicationEvaluation {
        admitted: refusals.is_empty(),
        identity: String::new(),
        provider_target: OCI_LOCAL_TARGET.to_string(),
        canonical_policy_identity: canonical.identity.clone(),
        effective_policy_identity: effective.identity.clone(),
        capability_identity: capabilities.identity.clone(),
        segment_applications,
        refusals,
    };
    evaluation.identity = semantic_contract_identity(&evaluation)?;
    let application_plan = if evaluation.admitted {
        Some(rederived_oci_local_application_plan(
            canonical,
            effective,
            restriction_authority,
            overrides,
        )?)
    } else {
        None
    };
    Ok((evaluation, application_plan))
}

fn oci_local_segment_plan(segment: &SandboxPolicySegment) -> OciLocalSegmentPlan {
    let filesystem = segment.runtime_boundary.filesystem.as_ref();
    let network = segment.runtime_boundary.network.as_ref();
    OciLocalSegmentPlan {
        segment_id: segment.id.clone(),
        segment_policy_identity: segment.identity.clone(),
        task: segment.task.clone(),
        declared_image: segment.runtime_image.clone().unwrap_or_default(),
        read_only_repo_root: filesystem.and_then(|entry| entry.repo_root_mode)
            == Some(RuntimeBoundaryRepoRootMode::ReadOnly),
        writable_paths: filesystem
            .map(|entry| entry.writable_paths.clone())
            .unwrap_or_default(),
        protected_paths: filesystem
            .map(|entry| entry.protected_paths.clone())
            .unwrap_or_default(),
        isolated_paths: segment.isolated_paths.clone(),
        pre_boundary_actions: segment.pre_boundary_actions.clone(),
        execution_kind: segment.execution_kind.clone(),
        deny_external_network: network.and_then(|entry| entry.default)
            == Some(RuntimeBoundaryNetworkDefault::Deny),
        platform: segment.target_platform.platform.clone(),
    }
}

fn rederived_oci_local_application_plan(
    canonical: &SandboxPolicy,
    effective: &EffectiveSandboxPolicy,
    restriction_authority: Option<&SandboxRestrictionAuthority>,
    overrides: ExecutionOverrides,
) -> Result<OciLocalApplicationPlan, String> {
    let capabilities = oci_local_capabilities(&canonical.target_platform)?;
    let mut plan = OciLocalApplicationPlan {
        identity: String::new(),
        lane: canonical.lane.clone(),
        execution_selection: SandboxExecutionSelection {
            backend: overrides.backend,
            lifecycle: overrides.lifecycle,
            skip_dependencies: overrides.skip_deps,
        },
        canonical_policy_identity: canonical.identity.clone(),
        restriction_authority: restriction_authority.cloned(),
        restriction_overlays: effective.restriction_overlays.clone(),
        restriction_overlay_identities: effective.restriction_overlay_identities.clone(),
        effective_policy_identity: effective.identity.clone(),
        capability_identity: capabilities.identity,
        target_platform: canonical.target_platform.clone(),
        segments: effective
            .segments
            .iter()
            .map(oci_local_segment_plan)
            .collect(),
        edges: effective.edges.clone(),
    };
    plan.identity = semantic_contract_identity(&plan)?;
    Ok(plan)
}

pub(crate) fn policy_has_authoritative_runtime_controls(policy: &SandboxPolicy) -> bool {
    segments_have_authoritative_runtime_controls(&policy.segments)
}

pub(crate) fn effective_policy_has_authoritative_runtime_controls(
    policy: &EffectiveSandboxPolicy,
) -> bool {
    segments_have_authoritative_runtime_controls(&policy.segments)
}

pub(crate) fn validate_application_evidence(
    evidence: &SandboxApplicationEvidence,
) -> Result<(), String> {
    if evidence.schema_version != SANDBOX_POLICY_SCHEMA_VERSION {
        return Err(String::from(
            "sandbox application evidence uses an unsupported schema version",
        ));
    }
    if evidence.provider_target != OCI_LOCAL_TARGET
        || evidence.provider_adapter_version != OCI_LOCAL_ADAPTER_VERSION
    {
        return Err(String::from(
            "sandbox application evidence names an unsupported provider target or adapter version",
        ));
    }
    let rederived_overlays =
        restriction_overlays_from_authority(evidence.restriction_authority.as_ref())?;
    if evidence.restriction_overlays != rederived_overlays {
        return Err(String::from(
            "sandbox application evidence restriction overlays do not derive from the archived policy authority snapshot",
        ));
    }
    for overlay in &evidence.restriction_overlays {
        let mut unsigned = overlay.clone();
        let identity = std::mem::take(&mut unsigned.identity);
        if identity != semantic_contract_identity(&unsigned)? {
            return Err(String::from(
                "sandbox application evidence contains an invalid restriction-overlay identity",
            ));
        }
    }
    let overlay_identities = evidence
        .restriction_overlays
        .iter()
        .map(|overlay| overlay.identity.as_str())
        .collect::<Vec<_>>();
    let declared_overlay_identities = evidence
        .restriction_overlay_identities
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if overlay_identities != declared_overlay_identities {
        return Err(String::from(
            "sandbox application evidence restriction-overlay identities do not match their snapshots",
        ));
    }
    let admitted_edge_snapshots = evidence
        .admitted_edges
        .iter()
        .map(|edge| edge.identity.as_str())
        .collect::<Vec<_>>();
    let admitted_edge_identities = evidence
        .admitted_edge_identities
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if admitted_edge_snapshots != admitted_edge_identities {
        return Err(String::from(
            "sandbox application evidence admitted edge identities do not match their graph snapshots",
        ));
    }
    for edge in &evidence.admitted_edges {
        let mut unsigned = edge.clone();
        let identity = std::mem::take(&mut unsigned.identity);
        if identity != semantic_contract_identity(&unsigned)? {
            return Err(String::from(
                "sandbox application evidence contains an invalid admitted edge identity",
            ));
        }
    }
    let capability = oci_local_capabilities(&evidence.target_platform)?;
    if evidence.capability_identity != capability.identity {
        return Err(String::from(
            "sandbox application evidence capability identity does not match its target platform",
        ));
    }
    if evidence.started_at.trim().is_empty()
        || evidence.attestation.issuer != "ota_runner"
        || evidence.attestation.trust != "runner_owned_runtime_inspection"
        || evidence.attestation.verifier != OCI_LOCAL_ADAPTER_VERSION
    {
        return Err(String::from(
            "sandbox application evidence is missing its runner-owned local attestation posture",
        ));
    }
    let expected_challenge_identity = semantic_contract_identity(&(
        evidence.runner_transaction_identity.as_str(),
        evidence.application_plan_identity.as_str(),
        evidence.started_at.as_str(),
        "oci_local_single_use_challenge",
    ))?;
    if evidence.attestation.challenge_identity != expected_challenge_identity {
        return Err(String::from(
            "sandbox application evidence challenge is not bound to its runner transaction",
        ));
    }
    let admitted_edge_identities = evidence
        .admitted_edge_identities
        .iter()
        .map(String::as_str)
        .collect::<BTreeSet<_>>();
    if admitted_edge_identities.len() != evidence.admitted_edge_identities.len() {
        return Err(String::from(
            "sandbox application evidence contains duplicate admitted edge identities",
        ));
    }
    let mut selected_edge_identities = BTreeSet::new();
    for edge in &evidence.selected_edges {
        if !selected_edge_identities.insert(edge.identity.as_str()) {
            return Err(String::from(
                "sandbox application evidence contains duplicate selected edge identities",
            ));
        }
        if !admitted_edge_identities.contains(edge.edge_identity.as_str()) {
            return Err(String::from(
                "sandbox application evidence selected an edge outside the admitted policy graph",
            ));
        }
        if !edge.source.starts_with("task:")
            || !edge.destination.starts_with("task:")
            || !edge.executed_segment.starts_with("task:")
        {
            return Err(String::from(
                "sandbox application evidence selected edge does not name canonical task segment identities",
            ));
        }
        if edge.executed_segment != edge.source && edge.executed_segment != edge.destination {
            return Err(String::from(
                "sandbox application evidence selected edge names an executed segment outside that edge",
            ));
        }
        let expected_edge_identity = semantic_contract_identity(&SandboxPolicyEdge {
            identity: String::new(),
            source: edge.source.clone(),
            destination: edge.destination.clone(),
            condition: edge.condition,
            order: edge.edge_order,
        })?;
        if edge.edge_identity != expected_edge_identity {
            return Err(String::from(
                "sandbox application evidence selected-edge policy identity is invalid",
            ));
        }
        let expected_identity = semantic_contract_identity(&(
            edge.edge_identity.as_str(),
            edge.source.as_str(),
            edge.destination.as_str(),
            edge.executed_segment.as_str(),
            edge.condition,
            edge.edge_order,
            edge.generation,
            edge.state,
            edge.source_exit_code,
            edge.source_generation,
        ))?;
        if edge.identity != expected_identity {
            return Err(String::from(
                "sandbox application evidence selected-edge identity is invalid",
            ));
        }
        match edge.condition {
            SandboxPolicyEdgeCondition::OnSuccess
                if edge.source_exit_code != Some(0) || edge.source_generation.is_none() =>
            {
                return Err(String::from(
                    "sandbox application evidence selected a success edge without a successful source outcome",
                ));
            }
            SandboxPolicyEdgeCondition::OnFailure
                if edge.source_exit_code.is_none_or(|exit_code| exit_code == 0)
                    || edge.source_generation.is_none() =>
            {
                return Err(String::from(
                    "sandbox application evidence selected a failure edge without a failed source outcome",
                ));
            }
            SandboxPolicyEdgeCondition::Always
                if edge.source_exit_code.is_none() || edge.source_generation.is_none() =>
            {
                return Err(String::from(
                    "sandbox application evidence selected an always edge without a witnessed source outcome",
                ));
            }
            SandboxPolicyEdgeCondition::Unconditional
                if edge.source_exit_code.is_some() || edge.source_generation.is_some() =>
            {
                return Err(String::from(
                    "sandbox application evidence attached a conditional source outcome to an unconditional edge",
                ));
            }
            _ => {}
        }
    }
    let mut segment_invocations = BTreeSet::new();
    let mut boundary_ids = BTreeSet::new();
    for segment in &evidence.segments {
        if !segment_invocations.insert((
            segment.segment_id.as_str(),
            segment.purpose,
            segment.invocation_generation,
        )) || !boundary_ids.insert(segment.boundary_identity.as_str())
        {
            return Err(String::from(
                "sandbox application evidence contains duplicate segment invocations or boundary identities",
            ));
        }
        let boundary_parts = segment.boundary_identity.splitn(3, ':').collect::<Vec<_>>();
        if boundary_parts.len() != 3
            || boundary_parts[0] != "container"
            || !matches!(boundary_parts[1], "docker" | "podman")
            || boundary_parts[2].is_empty()
        {
            return Err(String::from(
                "sandbox application evidence contains an invalid OCI boundary identity",
            ));
        }
        let expected_lease_identity = semantic_contract_identity(&(
            evidence.runner_transaction_identity.as_str(),
            evidence.application_plan_identity.as_str(),
            segment.segment_id.as_str(),
            segment.purpose,
            segment.invocation_generation,
            segment.boundary_identity.as_str(),
            "cleanup_authority_registered_before_mutation",
        ))?;
        if segment.cleanup_lease_identity != expected_lease_identity {
            return Err(String::from(
                "sandbox application evidence cleanup lease is not bound to its runner transaction",
            ));
        }
        let mut mount_paths = BTreeSet::new();
        for mount in &segment.writable_mounts {
            if !mount_paths.insert(mount.path.as_str())
                || !is_sha256_identity(mount.source_identity.as_str())
            {
                return Err(String::from(
                    "sandbox application evidence contains duplicate writable paths or invalid mount-source identities",
                ));
            }
        }
        if let Some(applied_identity) = segment.applied_policy_identity.as_deref() {
            let expected_applied_identity = semantic_contract_identity(&(
                segment.rendered_policy_identity.as_str(),
                segment.resolved_image_identity.as_deref(),
                segment.resolved_platform.as_deref(),
                &segment.filesystem,
                &segment.network,
                &segment.writable_mounts,
                segment.boundary_identity.as_str(),
            ))?;
            if applied_identity != expected_applied_identity {
                return Err(String::from(
                    "sandbox application evidence applied-policy identity does not match its witnessed controls",
                ));
            }
        }
        if let Some(terminal_identity) = segment.cleanup.terminal_identity.as_deref() {
            let expected_terminal_identity = semantic_contract_identity(&(
                segment.cleanup_lease_identity.as_str(),
                segment.boundary_identity.as_str(),
                segment.cleanup.state,
            ))?;
            if terminal_identity != expected_terminal_identity {
                return Err(String::from(
                    "sandbox application evidence cleanup terminal identity is invalid",
                ));
            }
        }
        if let Some(terminal_application_identity) =
            segment.terminal_application_identity.as_deref()
        {
            if segment.terminal_observation_identity.as_deref()
                != segment.applied_policy_identity.as_deref()
            {
                return Err(String::from(
                    "sandbox application evidence terminal observation does not match the applied policy identity",
                ));
            }
            let expected_terminal_application_identity = semantic_contract_identity(&(
                segment.cleanup_lease_identity.as_str(),
                segment.applied_policy_identity.as_deref(),
                segment.terminal_observation_identity.as_deref(),
                segment.boundary_identity.as_str(),
                "terminal_runtime_control_inspection",
            ))?;
            if terminal_application_identity != expected_terminal_application_identity {
                return Err(String::from(
                    "sandbox application evidence terminal application identity is invalid",
                ));
            }
        }
        let control_posture_valid = segment.filesystem.required
            == "repo_root_read_only_with_declared_writable_carveouts"
            && segment.filesystem.application == SandboxControlApplicationState::Enforced
            && match segment.network.required.as_str() {
                "external_ip_connectivity_denied" => {
                    segment.network.application == SandboxControlApplicationState::Enforced
                }
                "no_authoritative_network_control" => {
                    segment.network.application == SandboxControlApplicationState::NotRequired
                }
                _ => false,
            };
        if segment.status == SandboxSegmentApplicationStatus::EnforcedThroughCompletion
            && (segment.cleanup.state != SandboxCleanupState::Confirmed
                || segment.cleanup.terminal_identity.is_none()
                || segment.resolved_image_identity.is_none()
                || segment.resolved_platform.is_none()
                || segment.applied_policy_identity.is_none()
                || segment.terminal_application_identity.is_none()
                || segment.terminal_observation_identity.is_none()
                || !control_posture_valid)
        {
            return Err(String::from(
                "completed sandbox application evidence is missing enforced controls or confirmed cleanup",
            ));
        }
    }
    for edge in &evidence.selected_edges {
        if edge.state == SandboxSelectedEdgeState::Entered
            && !segment_invocations.contains(&(
                edge.executed_segment.as_str(),
                SandboxSegmentApplicationPurpose::TaskExecution,
                edge.generation,
            ))
        {
            return Err(String::from(
                "sandbox application evidence entered an edge without application evidence for its executed segment",
            ));
        }
    }
    let derived_status =
        if evidence.segments.is_empty() {
            SandboxApplicationStatus::NotStarted
        } else if evidence.segments.iter().all(|segment| {
            segment.status == SandboxSegmentApplicationStatus::EnforcedThroughCompletion
        }) {
            SandboxApplicationStatus::EnforcedThroughCompletion
        } else if evidence.segments.iter().any(|segment| {
            segment.status == SandboxSegmentApplicationStatus::EnforcementUnknownAfterInterruption
        }) {
            SandboxApplicationStatus::EnforcementUnknownAfterInterruption
        } else {
            SandboxApplicationStatus::EnforcementLost
        };
    if evidence.status != derived_status {
        return Err(String::from(
            "sandbox application evidence status does not match its segment terminal states",
        ));
    }
    Ok(())
}

pub(crate) fn validate_application_evidence_against_plan(
    evidence: &SandboxApplicationEvidence,
    plan: &OciLocalApplicationPlan,
) -> Result<(), String> {
    validate_application_evidence(evidence)?;
    if evidence.lane != plan.lane
        || evidence.execution_selection != plan.execution_selection
        || evidence.canonical_policy_identity != plan.canonical_policy_identity
        || evidence.restriction_authority != plan.restriction_authority
        || evidence.restriction_overlays != plan.restriction_overlays
        || evidence.restriction_overlay_identities != plan.restriction_overlay_identities
        || evidence.effective_policy_identity != plan.effective_policy_identity
        || evidence.target_platform != plan.target_platform
        || evidence.capability_identity != plan.capability_identity
        || evidence.application_plan_identity != plan.identity
    {
        return Err(String::from(
            "sandbox application evidence does not reconcile with its admitted application plan",
        ));
    }
    if evidence.admitted_segments != plan.segments || evidence.admitted_edges != plan.edges {
        return Err(String::from(
            "sandbox application evidence does not carry the exact admitted segment graph",
        ));
    }
    let admitted_edges = plan
        .edges
        .iter()
        .map(|edge| edge.identity.as_str())
        .collect::<Vec<_>>();
    let evidenced_edges = evidence
        .admitted_edge_identities
        .iter()
        .map(String::as_str)
        .collect::<Vec<_>>();
    if evidenced_edges != admitted_edges {
        return Err(String::from(
            "sandbox application evidence does not bind the exact admitted edge order",
        ));
    }
    for segment in &evidence.segments {
        let Some(admitted) = plan
            .segments
            .iter()
            .find(|candidate| candidate.segment_id == segment.segment_id)
        else {
            return Err(format!(
                "sandbox application evidence contains segment `{}` outside the admitted plan",
                segment.segment_id
            ));
        };
        if segment.segment_policy_identity != admitted.segment_policy_identity
            || segment.declared_image != admitted.declared_image
        {
            return Err(format!(
                "sandbox application evidence segment `{}` does not match its admitted policy",
                segment.segment_id
            ));
        }
        let boundary_parts = segment.boundary_identity.splitn(3, ':').collect::<Vec<_>>();
        let expected_rendered_policy_identity = semantic_contract_identity(&(
            admitted,
            segment.purpose,
            boundary_parts[1],
            boundary_parts[2],
            segment.declared_image.as_str(),
        ))?;
        if segment.rendered_policy_identity != expected_rendered_policy_identity {
            return Err(format!(
                "sandbox application evidence segment `{}` has an invalid rendered policy identity",
                segment.segment_id
            ));
        }
    }
    Ok(())
}

pub(crate) fn validate_application_evidence_against_contract(
    contract: &Contract,
    evidence: &SandboxApplicationEvidence,
) -> Result<(), String> {
    let overrides = ExecutionOverrides {
        backend: evidence.execution_selection.backend,
        lifecycle: evidence.execution_selection.lifecycle,
        host_port: None,
        memory: None,
        skip_deps: evidence.execution_selection.skip_dependencies,
    };
    let canonical = match evidence.lane.kind {
        SandboxLaneKind::Task => {
            sandbox_policy_for_task(contract, evidence.lane.name.as_str(), overrides)?
        }
        SandboxLaneKind::Workflow => {
            sandbox_policy_for_workflow(contract, Some(evidence.lane.name.as_str()), overrides)?
        }
    };
    let overlays = restriction_overlays_from_authority(evidence.restriction_authority.as_ref())?;
    let effective = effective_sandbox_policy(&canonical, &overlays)?;
    let plan = rederived_oci_local_application_plan(
        &canonical,
        &effective,
        evidence.restriction_authority.as_ref(),
        overrides,
    )?;
    validate_application_evidence_against_plan(evidence, &plan)
}

fn segments_have_authoritative_runtime_controls(segments: &[SandboxPolicySegment]) -> bool {
    segments.iter().any(|segment| {
        segment.runtime_boundary.filesystem.is_some()
            || segment
                .runtime_boundary
                .network
                .as_ref()
                .is_some_and(|network| {
                    network.default.is_some()
                        || network.outbound_targets.iter().any(|target| {
                            target
                                .destination_constraint
                                .as_ref()
                                .is_none_or(|constraint| {
                                    constraint.enforcement
                                != RuntimeBoundaryDestinationConstraintEnforcement::AdvisoryOnly
                                })
                        })
                })
    })
}

fn refusal(
    segment: &SandboxPolicySegment,
    control: &str,
    required: &str,
    capability: ProviderControlCapability,
    reason: &str,
) -> SandboxAdmissionRefusal {
    SandboxAdmissionRefusal {
        segment_id: segment.id.clone(),
        control: control.to_string(),
        required: required.to_string(),
        capability,
        reason: reason.to_string(),
    }
}

fn effective_runtime_boundary_for_task(
    contract: &Contract,
    task_name: &str,
) -> RuntimeBoundarySpec {
    let mut effective = contract
        .execution
        .as_ref()
        .and_then(|execution| execution.runtime_boundary.clone())
        .unwrap_or_default();
    if let Some(task_boundary) = contract
        .tasks
        .get(task_name)
        .and_then(|task| task.runtime_boundary.as_ref())
    {
        if task_boundary.filesystem.is_some() {
            effective.filesystem = task_boundary.filesystem.clone();
        }
        if task_boundary.network.is_some() {
            effective.network = task_boundary.network.clone();
        }
    }
    effective
}

fn meet_filesystem(
    canonical: Option<&RuntimeBoundaryFilesystemSpec>,
    overlay: &PolicySandboxFilesystemRules,
) -> Result<RuntimeBoundaryFilesystemSpec, String> {
    let canonical = canonical.cloned().unwrap_or_default();
    if canonical.repo_root_mode == Some(RuntimeBoundaryRepoRootMode::ReadOnly)
        && overlay.repo_root_mode == Some(RuntimeBoundaryRepoRootMode::Writable)
    {
        return Err(String::from(
            "sandbox filesystem overlay cannot widen a read-only repository root to writable",
        ));
    }
    let mut root_mode = match (canonical.repo_root_mode, overlay.repo_root_mode) {
        (Some(RuntimeBoundaryRepoRootMode::ReadOnly), _)
        | (_, Some(RuntimeBoundaryRepoRootMode::ReadOnly)) => {
            Some(RuntimeBoundaryRepoRootMode::ReadOnly)
        }
        (Some(RuntimeBoundaryRepoRootMode::Writable), _)
        | (_, Some(RuntimeBoundaryRepoRootMode::Writable)) => {
            Some(RuntimeBoundaryRepoRootMode::Writable)
        }
        (None, None) => None,
    };
    let canonical_root_mode = canonical.repo_root_mode;
    let canonical_writable_paths = canonical.writable_paths;
    let writable_paths = match overlay.writable_paths.as_ref() {
        None => canonical_writable_paths,
        Some(overlay_paths) => intersect_writable_regions(
            canonical_root_mode,
            canonical_writable_paths.as_slice(),
            overlay_paths,
        ),
    };
    if overlay.writable_paths.is_some() && root_mode == Some(RuntimeBoundaryRepoRootMode::Writable)
    {
        root_mode = Some(RuntimeBoundaryRepoRootMode::ReadOnly);
    }
    let mut protected_paths = canonical.protected_paths;
    protected_paths.extend(
        overlay
            .protected_paths
            .as_ref()
            .into_iter()
            .flatten()
            .cloned(),
    );
    protected_paths.sort();
    protected_paths.dedup();
    Ok(RuntimeBoundaryFilesystemSpec {
        repo_root_mode: root_mode,
        writable_paths,
        protected_paths,
    })
}

fn meet_network(
    canonical: Option<&RuntimeBoundaryNetworkSpec>,
    overlay: &PolicySandboxNetworkRules,
) -> Result<RuntimeBoundaryNetworkSpec, String> {
    let canonical = canonical.cloned().unwrap_or_default();
    if canonical.default == Some(RuntimeBoundaryNetworkDefault::Deny)
        && overlay.default == Some(RuntimeBoundaryNetworkDefault::Allow)
    {
        return Err(String::from(
            "sandbox network overlay cannot widen a deny-by-default boundary to allow",
        ));
    }
    let default = match (canonical.default, overlay.default) {
        (Some(RuntimeBoundaryNetworkDefault::Deny), _)
        | (_, Some(RuntimeBoundaryNetworkDefault::Deny)) => {
            Some(RuntimeBoundaryNetworkDefault::Deny)
        }
        (Some(RuntimeBoundaryNetworkDefault::Allow), _)
        | (_, Some(RuntimeBoundaryNetworkDefault::Allow)) => {
            Some(RuntimeBoundaryNetworkDefault::Allow)
        }
        (None, None) => None,
    };
    let outbound_targets = match overlay.outbound_targets.as_ref() {
        None => canonical.outbound_targets,
        Some(overlay_targets) if canonical.outbound_targets.is_empty() => {
            if canonical.default != Some(RuntimeBoundaryNetworkDefault::Deny) {
                overlay_targets.clone()
            } else {
                Vec::new()
            }
        }
        Some(overlay_targets) => canonical
            .outbound_targets
            .into_iter()
            .filter(|target| overlay_targets.contains(target))
            .collect(),
    };
    Ok(RuntimeBoundaryNetworkSpec {
        default,
        outbound_targets,
    })
}

fn intersect_writable_regions(
    canonical_root_mode: Option<RuntimeBoundaryRepoRootMode>,
    canonical_paths: &[String],
    overlay_paths: &[String],
) -> Vec<String> {
    let canonical_all = canonical_root_mode == Some(RuntimeBoundaryRepoRootMode::Writable)
        && canonical_paths.is_empty();
    let mut intersection = if canonical_all {
        overlay_paths.to_vec()
    } else {
        canonical_paths
            .iter()
            .flat_map(|canonical| {
                overlay_paths.iter().filter_map(move |overlay| {
                    if path_reachable_within(canonical, overlay) {
                        Some(canonical.clone())
                    } else if path_reachable_within(overlay, canonical) {
                        Some(overlay.clone())
                    } else {
                        None
                    }
                })
            })
            .collect()
    };
    intersection.sort();
    intersection.dedup();
    intersection
}

fn path_reachable_within(path: &str, region: &str) -> bool {
    let path = lexical_relative_path(path);
    let region = lexical_relative_path(region);
    path == region || path.starts_with(format!("{region}/").as_str())
}

fn validate_oci_filesystem_paths(
    repo_root: &Path,
    writable_paths: &[String],
    protected_paths: &[String],
) -> Result<(), String> {
    let canonical_root = repo_root
        .canonicalize()
        .map_err(|error| format!("could not canonicalize repository root: {error}"))?;
    let mut writable = Vec::new();
    let mut writable_host_paths = Vec::new();
    for path in writable_paths {
        let relative = validated_relative_path(path)?;
        let host_path = canonical_root.join(&relative);
        if !host_path.exists() {
            return Err(format!(
                "writable path `{path}` does not exist; the first oci_local adapter will not let the container runtime create an untracked host path"
            ));
        }
        if let Some(existing_ancestor) = nearest_existing_ancestor(&host_path)
            && !existing_ancestor
                .canonicalize()
                .map_err(|error| format!("could not canonicalize `{path}`: {error}"))?
                .starts_with(&canonical_root)
        {
            return Err(format!(
                "writable path `{path}` escapes the repository through a symlink"
            ));
        }
        writable.push(relative);
        writable_host_paths.push(host_path);
    }
    let mut protected_host_paths = Vec::new();
    for path in protected_paths {
        let protected = validated_relative_path(path)?;
        if writable.iter().any(|candidate| {
            protected == *candidate
                || protected.starts_with(candidate)
                || candidate.starts_with(&protected)
        }) {
            return Err(format!(
                "protected path `{path}` overlaps a writable carve-out and cannot be enforced by the stock OCI adapter"
            ));
        }
        let host_path = canonical_root.join(protected);
        if host_path.exists() {
            protected_host_paths.push(host_path);
        }
    }
    #[cfg(unix)]
    let protected_file_identities = protected_file_identities(&protected_host_paths)?;
    for writable_host_path in &writable_host_paths {
        reject_writable_aliases(
            writable_host_path,
            protected_host_paths.as_slice(),
            #[cfg(unix)]
            &protected_file_identities,
        )?;
    }
    Ok(())
}

fn reject_writable_aliases(
    writable_root: &Path,
    _protected_paths: &[PathBuf],
    #[cfg(unix)] protected_file_identities: &BTreeSet<(u64, u64)>,
) -> Result<(), String> {
    let mut pending = vec![writable_root.to_path_buf()];
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "could not inspect writable path `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "writable carve-out `{}` contains a symlink; the stock OCI adapter refuses writable aliasing rather than overclaiming protected-path isolation",
                path.display()
            ));
        }
        #[cfg(unix)]
        {
            use std::os::unix::fs::MetadataExt;
            if protected_file_identities.contains(&(metadata.dev(), metadata.ino())) {
                return Err(format!(
                    "writable path `{}` is a hardlink alias of protected content",
                    path.display()
                ));
            }
        }
        #[cfg(not(unix))]
        if !_protected_paths.is_empty() && metadata.is_file() {
            return Err(String::from(
                "the stock OCI adapter cannot prove hardlink separation for writable files on this host platform while protected paths are declared",
            ));
        }
        if metadata.is_dir() {
            let entries = fs::read_dir(&path).map_err(|error| {
                format!(
                    "could not inspect writable directory `{}`: {error}",
                    path.display()
                )
            })?;
            for entry in entries {
                let entry = entry.map_err(|error| {
                    format!(
                        "could not inspect writable directory `{}`: {error}",
                        path.display()
                    )
                })?;
                pending.push(entry.path());
            }
        }
    }
    Ok(())
}

#[cfg(unix)]
fn protected_file_identities(paths: &[PathBuf]) -> Result<BTreeSet<(u64, u64)>, String> {
    use std::os::unix::fs::MetadataExt;

    let mut identities = BTreeSet::new();
    let mut pending = paths.to_vec();
    while let Some(path) = pending.pop() {
        let metadata = fs::symlink_metadata(&path).map_err(|error| {
            format!(
                "could not inspect protected path `{}`: {error}",
                path.display()
            )
        })?;
        if metadata.file_type().is_symlink() {
            return Err(format!(
                "protected path `{}` contains a symlink; the stock OCI adapter cannot prove that it does not alias a writable region",
                path.display()
            ));
        }
        identities.insert((metadata.dev(), metadata.ino()));
        if metadata.is_dir() {
            let entries = fs::read_dir(&path).map_err(|error| {
                format!(
                    "could not inspect protected directory `{}`: {error}",
                    path.display()
                )
            })?;
            for entry in entries {
                let entry = entry.map_err(|error| {
                    format!(
                        "could not inspect protected directory `{}`: {error}",
                        path.display()
                    )
                })?;
                pending.push(entry.path());
            }
        }
    }
    Ok(identities)
}

fn validated_relative_path(path: &str) -> Result<PathBuf, String> {
    let trimmed = path.trim();
    if trimmed.is_empty() {
        return Err(String::from("sandbox filesystem paths must not be empty"));
    }
    let candidate = Path::new(trimmed);
    if candidate.is_absolute()
        || candidate.components().any(|component| {
            matches!(
                component,
                Component::ParentDir | Component::RootDir | Component::Prefix(_)
            )
        })
    {
        return Err(format!(
            "sandbox filesystem path `{path}` must remain repository-relative"
        ));
    }
    Ok(candidate.to_path_buf())
}

fn nearest_existing_ancestor(path: &Path) -> Option<&Path> {
    let mut candidate = path;
    loop {
        if candidate.exists() {
            return Some(candidate);
        }
        candidate = candidate.parent()?;
    }
}

fn lexical_relative_path(path: &str) -> String {
    Path::new(path)
        .components()
        .filter_map(|component| match component {
            Component::Normal(value) => Some(value.to_string_lossy().into_owned()),
            Component::CurDir => None,
            _ => Some(String::from("..")),
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn stable_topological_order(
    segments: &BTreeMap<String, SandboxPolicySegment>,
    edges: &[(String, String, SandboxPolicyEdgeCondition)],
) -> Result<Vec<String>, String> {
    let mut incoming = segments
        .keys()
        .map(|name| (name.clone(), 0_usize))
        .collect::<BTreeMap<_, _>>();
    let mut outgoing = BTreeMap::<String, Vec<String>>::new();
    for (source, destination, _) in edges {
        *incoming.entry(destination.clone()).or_default() += 1;
        outgoing
            .entry(source.clone())
            .or_default()
            .push(destination.clone());
    }
    let mut ready = incoming
        .iter()
        .filter_map(|(name, count)| (*count == 0).then_some(name.clone()))
        .collect::<BTreeSet<_>>();
    let mut ordered = Vec::with_capacity(segments.len());
    while let Some(name) = ready.pop_first() {
        ordered.push(name.clone());
        if let Some(destinations) = outgoing.get(name.as_str()) {
            for destination in destinations {
                let Some(count) = incoming.get_mut(destination) else {
                    continue;
                };
                *count = count.saturating_sub(1);
                if *count == 0 {
                    ready.insert(destination.clone());
                }
            }
        }
    }
    if ordered.len() != segments.len() {
        return Err(String::from(
            "sandbox policy graph contains a cycle or unresolved segment",
        ));
    }
    Ok(ordered)
}

fn common_target_platform(segments: &[SandboxPolicySegment]) -> SandboxTargetPlatform {
    let first = segments
        .first()
        .map(|segment| segment.target_platform.clone())
        .unwrap_or_else(|| target_platform(Backend::Native, None));
    if segments
        .iter()
        .all(|segment| segment.target_platform == first)
    {
        first
    } else {
        SandboxTargetPlatform {
            os: String::from("mixed"),
            architecture: String::from("mixed"),
            platform: Some(String::from("per_segment")),
        }
    }
}

fn target_platform(
    backend: Backend,
    container: Option<&crate::schema::ContainerBackend>,
) -> SandboxTargetPlatform {
    let declared_platform = container.and_then(|container| container.platform.clone());
    let (os, architecture) = if let Some(platform) = declared_platform.as_deref() {
        let mut parts = platform.split('/');
        (
            parts.next().unwrap_or("linux").to_string(),
            parts
                .next()
                .map(normalize_arch)
                .unwrap_or_else(|| normalize_arch(std::env::consts::ARCH)),
        )
    } else {
        (
            if backend == Backend::Container {
                String::from("linux")
            } else {
                current_target_os().to_string()
            },
            normalize_arch(std::env::consts::ARCH),
        )
    };
    SandboxTargetPlatform {
        os,
        architecture,
        platform: declared_platform,
    }
}

fn current_target_os() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        "linux" => "linux",
        other => other,
    }
}

fn normalize_arch(arch: &str) -> String {
    match arch {
        "x86_64" => String::from("amd64"),
        "aarch64" => String::from("arm64"),
        other => other.to_string(),
    }
}

#[cfg(test)]
mod tests {
    use std::fs;

    use tempfile::TempDir;

    use super::*;

    fn contract(source: &str) -> Contract {
        serde_yaml::from_str(source).expect("contract should parse")
    }

    #[test]
    fn task_policy_preserves_conditional_hook_edges() {
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
tasks:
  verify:
    command: { exe: sh, args: [-c, "true"] }
    depends_on: [setup]
    after_success: [success]
    after_failure: [failure]
    after_always: [always]
  setup:
    command: { exe: sh, args: [-c, "true"] }
  success:
    command: { exe: sh, args: [-c, "true"] }
  failure:
    command: { exe: sh, args: [-c, "true"] }
  always:
    command: { exe: sh, args: [-c, "true"] }
"#,
        );
        let policy =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        let conditions = policy
            .edges
            .iter()
            .map(|edge| edge.condition)
            .collect::<BTreeSet<_>>();
        assert_eq!(
            conditions,
            BTreeSet::from([
                SandboxPolicyEdgeCondition::Unconditional,
                SandboxPolicyEdgeCondition::OnSuccess,
                SandboxPolicyEdgeCondition::OnFailure,
                SandboxPolicyEdgeCondition::Always,
            ])
        );
    }

    #[test]
    fn task_policy_identity_is_deterministic() {
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
tasks:
  verify:
    command: { exe: sh, args: [-c, "true"] }
"#,
        );
        let left =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        let right =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        assert_eq!(left, right);
        assert!(left.identity.starts_with("sha256:"));
    }

    #[test]
    fn container_policy_resolves_linux_variant_from_declared_platform() {
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
execution:
  preferred: container
  backends:
    container:
      image: alpine:3.22
      platform: linux/amd64
tasks:
  verify:
    command:
      exe: sh
      args: [-c, "true"]
      interaction: forbidden
    variants:
      - when:
          os: linux
        command:
          exe: sh
          args: [-c, "true"]
          interaction: required
"#,
        );

        let policy =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();

        assert_eq!(policy.segments[0].target_platform.os, "linux");
        assert_eq!(
            policy.segments[0].interaction,
            CommandInteractionPosture::Required
        );
    }

    #[test]
    fn task_policy_refuses_one_task_reused_across_dependency_and_hook_phases() {
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
tasks:
  verify:
    command: { exe: sh, args: [-c, "true"] }
    depends_on: [shared]
    after_success: [shared]
  shared:
    command: { exe: sh, args: [-c, "true"] }
"#,
        );

        let error = sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default())
            .unwrap_err();

        assert!(error.contains("reused across multiple execution phases"));
        assert!(error.contains("distinct task identities"));
    }

    #[test]
    fn restriction_overlay_must_derive_from_identified_authority_snapshot() {
        let mut authority = SandboxRestrictionAuthority {
            identity: String::new(),
            source: String::from("repo policy"),
            rules: PolicySandboxRules {
                filesystem: Some(PolicySandboxFilesystemRules {
                    repo_root_mode: Some(RuntimeBoundaryRepoRootMode::ReadOnly),
                    ..PolicySandboxFilesystemRules::default()
                }),
                network: None,
            },
            source_identity: String::new(),
        };
        authority.source_identity = semantic_contract_identity(&authority.rules).unwrap();
        authority.identity = semantic_contract_identity(&authority).unwrap();
        let overlays = restriction_overlays_from_authority(Some(&authority)).unwrap();
        assert_eq!(overlays.len(), 1);

        let mut tampered = authority;
        tampered.rules.network = Some(PolicySandboxNetworkRules {
            default: Some(RuntimeBoundaryNetworkDefault::Deny),
            ..PolicySandboxNetworkRules::default()
        });
        let error = restriction_overlays_from_authority(Some(&tampered)).unwrap_err();
        assert!(error.contains("invalid identity"));
    }

    #[test]
    fn oci_refuses_native_and_targeted_network_policy() {
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
tasks:
  verify:
    command: { exe: sh, args: [-c, "true"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
      network:
        default: deny
        outbound_targets:
          - kind: host
            value: example.com
"#,
        );
        let policy =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        let effective = effective_sandbox_policy(&policy, &[]).unwrap();
        let root = TempDir::new().unwrap();
        let (evaluation, plan) = evaluate_oci_local_application(
            &policy,
            &effective,
            None,
            root.path(),
            ExecutionOverrides::default(),
        )
        .unwrap();
        assert!(!evaluation.admitted);
        assert!(plan.is_none());
        assert!(
            evaluation
                .refusals
                .iter()
                .any(|entry| entry.control == "execution.backend")
        );
        assert!(
            evaluation
                .refusals
                .iter()
                .any(|entry| entry.control == "network.target_allowlist")
        );
    }

    #[test]
    fn oci_admits_ephemeral_container_with_read_only_root_and_network_denial() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join("coverage")).unwrap();
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: alpine:3.22
      platform: linux/amd64
tasks:
  verify:
    command: { exe: sh, args: [-c, "true"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
        writable_paths: [coverage]
        protected_paths: [ota.yaml]
      network:
        default: deny
"#,
        );
        let policy =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        let effective = effective_sandbox_policy(&policy, &[]).unwrap();
        let (evaluation, plan) = evaluate_oci_local_application(
            &policy,
            &effective,
            None,
            root.path(),
            ExecutionOverrides::default(),
        )
        .unwrap();
        assert!(evaluation.admitted);
        let plan = plan.unwrap();
        assert!(plan.segments[0].read_only_repo_root);
        assert!(plan.segments[0].deny_external_network);
        assert_eq!(plan.segments[0].writable_paths, ["coverage"]);
    }

    #[test]
    fn oci_refuses_typed_prepare_before_the_enforced_command_boundary() {
        let root = TempDir::new().unwrap();
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: python:3.12-bookworm
      platform: linux/amd64
tasks:
  setup:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: uv
        cwd: .
        mode: pip_requirements
        requirements_file: requirements.txt
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
      network:
        default: deny
"#,
        );
        let policy =
            sandbox_policy_for_task(&contract, "setup", ExecutionOverrides::default()).unwrap();
        let effective = effective_sandbox_policy(&policy, &[]).unwrap();
        let (evaluation, plan) = evaluate_oci_local_application(
            &policy,
            &effective,
            None,
            root.path(),
            ExecutionOverrides::default(),
        )
        .unwrap();

        assert!(!evaluation.admitted);
        assert!(plan.is_none());
        assert!(evaluation.refusals.iter().any(|entry| {
            entry.control == "execution.pre_boundary_actions"
                && entry.required.contains("task_prepare")
        }));
        assert!(
            evaluation
                .refusals
                .iter()
                .any(|entry| entry.control == "execution.kind")
        );
    }

    #[test]
    fn oci_refuses_undeclared_target_platform() {
        let root = TempDir::new().unwrap();
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: alpine:3.22
tasks:
  verify:
    command: { exe: sh, args: [-c, "true"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
      network:
        default: deny
"#,
        );
        let policy =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        let effective = effective_sandbox_policy(&policy, &[]).unwrap();
        let (evaluation, plan) = evaluate_oci_local_application(
            &policy,
            &effective,
            None,
            root.path(),
            ExecutionOverrides::default(),
        )
        .unwrap();
        assert!(!evaluation.admitted);
        assert!(plan.is_none());
        assert!(evaluation.refusals.iter().any(|entry| {
            entry.control == "execution.target_platform"
                && entry.capability == ProviderControlCapability::Unsupported
        }));
    }

    #[test]
    fn oci_refuses_inherited_networks_and_undeclared_isolated_writes() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join("node_modules")).unwrap();
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
execution:
  preferred: container
  contexts:
    verify:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
        platform: linux/amd64
      attachments:
        compose: [app]
        isolated_paths: [node_modules]
tasks:
  verify:
    context: verify
    command: { exe: node, args: [-e, "process.exit(0)"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
      network:
        default: deny
"#,
        );
        let policy =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        let effective = effective_sandbox_policy(&policy, &[]).unwrap();
        let (evaluation, plan) = evaluate_oci_local_application(
            &policy,
            &effective,
            None,
            root.path(),
            ExecutionOverrides::default(),
        )
        .unwrap();
        assert!(!evaluation.admitted);
        assert!(plan.is_none());
        assert!(
            evaluation
                .refusals
                .iter()
                .any(|entry| { entry.control == "network.inherited_service_network" })
        );
        assert!(
            evaluation
                .refusals
                .iter()
                .any(|entry| { entry.control == "filesystem.isolated_path" })
        );
    }

    #[test]
    fn oci_refuses_declared_writable_isolated_paths_without_transaction_evidence() {
        let root = TempDir::new().unwrap();
        fs::create_dir(root.path().join("node_modules")).unwrap();
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
execution:
  preferred: container
  contexts:
    verify:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
        platform: linux/amd64
      attachments:
        isolated_paths: [node_modules]
tasks:
  verify:
    context: verify
    command: { exe: node, args: [-e, "process.exit(0)"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
        writable_paths: [node_modules]
      network:
        default: deny
"#,
        );
        let policy =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        let effective = effective_sandbox_policy(&policy, &[]).unwrap();
        let (evaluation, plan) = evaluate_oci_local_application(
            &policy,
            &effective,
            None,
            root.path(),
            ExecutionOverrides::default(),
        )
        .unwrap();

        assert!(!evaluation.admitted);
        assert!(plan.is_none());
        assert!(evaluation.refusals.iter().any(|entry| {
            entry.control == "filesystem.isolated_path"
                && entry.required == "transactionally_evidenced_provider_resource"
        }));
    }

    #[test]
    fn oci_rejects_protected_path_inside_writable_carveout() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("coverage/private")).unwrap();
        let error = validate_oci_filesystem_paths(
            root.path(),
            &[String::from("coverage")],
            &[String::from("coverage/private")],
        )
        .unwrap_err();
        assert!(error.contains("overlaps a writable carve-out"));
    }

    #[test]
    fn oci_rejects_writable_path_inside_protected_directory() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("protected/output")).unwrap();
        let error = validate_oci_filesystem_paths(
            root.path(),
            &[String::from("protected/output")],
            &[String::from("protected")],
        )
        .unwrap_err();
        assert!(error.contains("overlaps a writable carve-out"));
    }

    #[cfg(unix)]
    #[test]
    fn oci_rejects_nested_hardlink_alias_to_protected_content() {
        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("writable")).unwrap();
        fs::create_dir_all(root.path().join("protected/nested")).unwrap();
        let protected = root.path().join("protected/nested/secret");
        fs::write(&protected, "secret").unwrap();
        fs::hard_link(&protected, root.path().join("writable/alias")).unwrap();
        let error = validate_oci_filesystem_paths(
            root.path(),
            &[String::from("writable")],
            &[String::from("protected")],
        )
        .unwrap_err();
        assert!(error.contains("hardlink alias"));
    }

    #[cfg(unix)]
    #[test]
    fn oci_rejects_symlink_inside_protected_content() {
        use std::os::unix::fs::symlink;

        let root = TempDir::new().unwrap();
        fs::create_dir_all(root.path().join("writable")).unwrap();
        fs::create_dir_all(root.path().join("protected")).unwrap();
        symlink("../writable", root.path().join("protected/alias")).unwrap();
        let error = validate_oci_filesystem_paths(
            root.path(),
            &[String::from("writable")],
            &[String::from("protected")],
        )
        .unwrap_err();
        assert!(error.contains("contains a symlink"));
    }

    #[test]
    fn filesystem_overlay_omission_is_neutral_and_explicit_empty_denies_writes() {
        let canonical = RuntimeBoundaryFilesystemSpec {
            repo_root_mode: Some(RuntimeBoundaryRepoRootMode::ReadOnly),
            writable_paths: vec![String::from("coverage")],
            protected_paths: vec![String::from("ota.yaml")],
        };
        let neutral =
            meet_filesystem(Some(&canonical), &PolicySandboxFilesystemRules::default()).unwrap();
        assert_eq!(neutral, canonical);

        let denied = meet_filesystem(
            Some(&canonical),
            &PolicySandboxFilesystemRules {
                writable_paths: Some(Vec::new()),
                ..PolicySandboxFilesystemRules::default()
            },
        )
        .unwrap();
        assert_eq!(
            denied.repo_root_mode,
            Some(RuntimeBoundaryRepoRootMode::ReadOnly)
        );
        assert!(denied.writable_paths.is_empty());
    }

    #[test]
    fn filesystem_overlay_intersects_nested_writable_regions() {
        let effective = meet_filesystem(
            Some(&RuntimeBoundaryFilesystemSpec {
                repo_root_mode: Some(RuntimeBoundaryRepoRootMode::ReadOnly),
                writable_paths: vec![String::from("coverage")],
                protected_paths: Vec::new(),
            }),
            &PolicySandboxFilesystemRules {
                writable_paths: Some(vec![String::from("coverage/reports")]),
                ..PolicySandboxFilesystemRules::default()
            },
        )
        .unwrap();
        assert_eq!(effective.writable_paths, ["coverage/reports"]);
    }

    #[test]
    fn application_evidence_rejects_completed_status_without_segments() {
        let target_platform = SandboxTargetPlatform {
            os: String::from("linux"),
            architecture: String::from("amd64"),
            platform: Some(String::from("linux/amd64")),
        };
        let application_plan_identity = semantic_contract_identity(&"plan").unwrap();
        let runner_transaction_identity = semantic_contract_identity(&"transaction").unwrap();
        let started_at = String::from("2026-07-29T00:00:00Z");
        let evidence = SandboxApplicationEvidence {
            schema_version: SANDBOX_POLICY_SCHEMA_VERSION,
            lane: SandboxLaneIdentity {
                kind: SandboxLaneKind::Task,
                name: String::from("verify"),
            },
            execution_selection: SandboxExecutionSelection {
                backend: Some(Backend::Container),
                lifecycle: Some(Lifecycle::Ephemeral),
                skip_dependencies: false,
            },
            canonical_policy_identity: semantic_contract_identity(&"canonical").unwrap(),
            restriction_authority: None,
            restriction_overlays: Vec::new(),
            restriction_overlay_identities: Vec::new(),
            effective_policy_identity: semantic_contract_identity(&"effective").unwrap(),
            target_platform: target_platform.clone(),
            provider_target: OCI_LOCAL_TARGET.to_string(),
            provider_adapter_version: OCI_LOCAL_ADAPTER_VERSION.to_string(),
            capability_identity: oci_local_capabilities(&target_platform).unwrap().identity,
            application_plan_identity: application_plan_identity.clone(),
            runner_transaction_identity: runner_transaction_identity.clone(),
            started_at: started_at.clone(),
            attestation: SandboxLocalAttestationEvidence {
                issuer: String::from("ota_runner"),
                trust: String::from("runner_owned_runtime_inspection"),
                challenge_identity: semantic_contract_identity(&(
                    runner_transaction_identity.as_str(),
                    application_plan_identity.as_str(),
                    started_at.as_str(),
                    "oci_local_single_use_challenge",
                ))
                .unwrap(),
                verifier: OCI_LOCAL_ADAPTER_VERSION.to_string(),
            },
            status: SandboxApplicationStatus::EnforcedThroughCompletion,
            admitted_edge_identities: Vec::new(),
            admitted_segments: Vec::new(),
            admitted_edges: Vec::new(),
            selected_edges: Vec::new(),
            segments: Vec::new(),
        };
        let error = validate_application_evidence(&evidence).unwrap_err();
        assert!(error.contains("segment terminal states"));
    }

    #[test]
    fn application_evidence_rejects_selected_failure_edge_after_success() {
        let target_platform = SandboxTargetPlatform {
            os: String::from("linux"),
            architecture: String::from("amd64"),
            platform: Some(String::from("linux/amd64")),
        };
        let application_plan_identity = semantic_contract_identity(&"plan").unwrap();
        let runner_transaction_identity = semantic_contract_identity(&"transaction").unwrap();
        let started_at = String::from("2026-07-29T00:00:00Z");
        let mut edge = SandboxPolicyEdge {
            identity: String::new(),
            source: String::from("task:verify"),
            destination: String::from("task:cleanup"),
            condition: SandboxPolicyEdgeCondition::OnFailure,
            order: 0,
        };
        edge.identity = semantic_contract_identity(&edge).unwrap();
        let selected_identity = semantic_contract_identity(&(
            edge.identity.as_str(),
            edge.source.as_str(),
            edge.destination.as_str(),
            edge.destination.as_str(),
            edge.condition,
            edge.order,
            0usize,
            SandboxSelectedEdgeState::Entered,
            Some(0i32),
            Some(0usize),
        ))
        .unwrap();
        let evidence = SandboxApplicationEvidence {
            schema_version: SANDBOX_POLICY_SCHEMA_VERSION,
            lane: SandboxLaneIdentity {
                kind: SandboxLaneKind::Task,
                name: String::from("verify"),
            },
            execution_selection: SandboxExecutionSelection {
                backend: Some(Backend::Container),
                lifecycle: Some(Lifecycle::Ephemeral),
                skip_dependencies: false,
            },
            canonical_policy_identity: semantic_contract_identity(&"canonical").unwrap(),
            restriction_authority: None,
            restriction_overlays: Vec::new(),
            restriction_overlay_identities: Vec::new(),
            effective_policy_identity: semantic_contract_identity(&"effective").unwrap(),
            target_platform: target_platform.clone(),
            provider_target: OCI_LOCAL_TARGET.to_string(),
            provider_adapter_version: OCI_LOCAL_ADAPTER_VERSION.to_string(),
            capability_identity: oci_local_capabilities(&target_platform).unwrap().identity,
            application_plan_identity: application_plan_identity.clone(),
            runner_transaction_identity: runner_transaction_identity.clone(),
            started_at: started_at.clone(),
            attestation: SandboxLocalAttestationEvidence {
                issuer: String::from("ota_runner"),
                trust: String::from("runner_owned_runtime_inspection"),
                challenge_identity: semantic_contract_identity(&(
                    runner_transaction_identity.as_str(),
                    application_plan_identity.as_str(),
                    started_at.as_str(),
                    "oci_local_single_use_challenge",
                ))
                .unwrap(),
                verifier: OCI_LOCAL_ADAPTER_VERSION.to_string(),
            },
            status: SandboxApplicationStatus::NotStarted,
            admitted_edge_identities: vec![edge.identity.clone()],
            admitted_segments: Vec::new(),
            admitted_edges: vec![edge.clone()],
            selected_edges: vec![SandboxSelectedEdgeEvidence {
                identity: selected_identity,
                edge_identity: edge.identity,
                source: edge.source,
                destination: edge.destination,
                executed_segment: String::from("task:cleanup"),
                condition: edge.condition,
                edge_order: edge.order,
                generation: 0,
                state: SandboxSelectedEdgeState::Entered,
                source_exit_code: Some(0),
                source_generation: Some(0),
            }],
            segments: Vec::new(),
        };
        let error = validate_application_evidence(&evidence).unwrap_err();
        assert!(error.contains("failure edge"));
    }

    #[test]
    fn restriction_overlay_order_does_not_change_effective_identity() {
        let contract = contract(
            r#"
version: 1
project:
  name: sandbox
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: alpine:3.22
tasks:
  verify:
    command: { exe: sh, args: [-c, "true"] }
    runtime_boundary:
      filesystem:
        repo_root_mode: read_only
        writable_paths: [coverage]
      network:
        default: allow
"#,
        );
        let canonical =
            sandbox_policy_for_task(&contract, "verify", ExecutionOverrides::default()).unwrap();
        let overlays = [
            SandboxRestrictionOverlay {
                identity: String::from("sha256:bbbb"),
                source: String::from("policy:b"),
                filesystem: Some(PolicySandboxFilesystemRules {
                    protected_paths: Some(vec![String::from("ota.yaml")]),
                    ..PolicySandboxFilesystemRules::default()
                }),
                network: None,
            },
            SandboxRestrictionOverlay {
                identity: String::from("sha256:aaaa"),
                source: String::from("policy:a"),
                filesystem: None,
                network: Some(PolicySandboxNetworkRules {
                    default: Some(RuntimeBoundaryNetworkDefault::Deny),
                    outbound_targets: Some(Vec::new()),
                }),
            },
        ];
        let forward = effective_sandbox_policy(&canonical, &overlays).unwrap();
        let reverse =
            effective_sandbox_policy(&canonical, &[overlays[1].clone(), overlays[0].clone()])
                .unwrap();
        assert_eq!(forward, reverse);
        assert_eq!(
            forward.restriction_overlay_identities,
            ["sha256:aaaa", "sha256:bbbb"]
        );
    }

    #[test]
    fn overlays_cannot_widen_read_only_or_deny_boundaries() {
        let filesystem_error = meet_filesystem(
            Some(&RuntimeBoundaryFilesystemSpec {
                repo_root_mode: Some(RuntimeBoundaryRepoRootMode::ReadOnly),
                ..RuntimeBoundaryFilesystemSpec::default()
            }),
            &PolicySandboxFilesystemRules {
                repo_root_mode: Some(RuntimeBoundaryRepoRootMode::Writable),
                ..PolicySandboxFilesystemRules::default()
            },
        )
        .unwrap_err();
        assert!(filesystem_error.contains("cannot widen"));

        let network_error = meet_network(
            Some(&RuntimeBoundaryNetworkSpec {
                default: Some(RuntimeBoundaryNetworkDefault::Deny),
                outbound_targets: Vec::new(),
            }),
            &PolicySandboxNetworkRules {
                default: Some(RuntimeBoundaryNetworkDefault::Allow),
                outbound_targets: None,
            },
        )
        .unwrap_err();
        assert!(network_error.contains("cannot widen"));
    }
}
