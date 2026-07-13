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

use serde::{Deserialize, Serialize, Serializer, ser::SerializeMap};
use serde_json::Value as JsonValue;
use std::collections::{BTreeMap, BTreeSet};
use std::path::Path;

use crate::detector::{Confidence, DetectContract, Inference};
use crate::doctor::{AdapterBootstrapDiagnostics, Finding, FindingSeverity};
use crate::policy_pack::{OrgPolicyPack, ProvisioningBackendRequest, ProvisioningPlan};
use crate::runner::{
    BackendFulfillmentEvidence, ExecutionOverrides, ResolvedExecutionBackend, ResolvedTaskRuntime,
    SharedLocalBackendEvidence, TaskTargetResolutionEvidence, blocking_declared_env_source_label,
    effective_task_execution, env_resolution_source_label, load_declared_env_sources,
    load_policy_env_overlay, orchestrator_execution_preview, resolve_declared_env_source_value,
    resolve_execution_backend_with_contract_path,
};
use crate::schema::{
    AgentConfig, Backend, Contract, ExecutionContext, ExtensionSpec, GeneratedArtifactSpec,
    Lifecycle, ServiceSpec, TaskInputSpec, TaskSpec, TaskVariantView,
};
use crate::workspace::{WorkspaceExecutionSummary, WorkspaceRepoDoctorReport};

fn slice_is_empty<T>(value: &[T]) -> bool {
    value.is_empty()
}

fn contract_identity_metadata_is_empty(value: &ContractIdentityMetadata) -> bool {
    value.owner.is_none() && value.team.is_none() && value.repo_class.is_none()
}

fn contract_identity_execution_is_empty(value: &ContractIdentityExecution) -> bool {
    value.preferred.is_none()
        && value.lifecycle.is_none()
        && value.supported.is_empty()
        && value.image.is_none()
}

#[cfg(target_os = "windows")]
fn current_os() -> &'static str {
    "windows"
}

#[cfg(target_os = "macos")]
fn current_os() -> &'static str {
    "macos"
}

#[cfg(all(not(target_os = "windows"), not(target_os = "macos")))]
fn current_os() -> &'static str {
    "linux"
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OutputFormat {
    Text,
    Json,
}

#[derive(Debug)]
pub struct CommandOutput {
    pub stdout: String,
    pub stderr: Option<String>,
    pub exit_code: i32,
}

#[derive(Debug, Serialize)]
pub struct DoctorSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub summary: DoctorSummary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub finding_groups: Vec<DoctorFindingGroupSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub governance: Option<DoctorGovernanceSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisioning: Option<&'a ProvisioningPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisioning_request: Option<&'a ProvisioningBackendRequest>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_bootstrap: Option<&'a AdapterBootstrapDiagnostics>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: &'a BTreeMap<String, ExtensionSpec>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix: Option<DoctorFixSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub toolchains: Vec<ToolchainSelectionSummary>,
    pub findings: &'a [Finding],
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorGovernanceSummary {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required_verification_lanes: Vec<DoctorRequiredVerificationLane>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub merge_gate: Option<DoctorMergeGateSummary>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorRequiredVerificationLane {
    pub merge_check_id: String,
    pub lane_task: String,
    pub lane_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contract_sources: Vec<String>,
    pub evidence_classes: DoctorRequiredVerificationLaneEvidenceClasses,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorMergeGateSummary {
    pub state: String,
    pub blocking: bool,
    pub required_lane_count: usize,
    pub drift_lane_count: usize,
    pub evidence_classes: DoctorMergeGateSummaryEvidenceClasses,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_basis: Vec<GovernanceDecisionBasisEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_inputs: Vec<GovernanceDecisionInputEntry>,
    pub replay: GovernanceReplayResult,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lanes: Vec<DoctorMergeGateLane>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorMergeGateLane {
    pub merge_check_id: String,
    pub lane_task: String,
    pub lane_kind: String,
    pub state: String,
    pub blocking: bool,
    pub evidence_classes: DoctorMergeGateLaneEvidenceClasses,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_basis: Vec<GovernanceDecisionBasisEntry>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decision_inputs: Vec<GovernanceDecisionInputEntry>,
    pub replay: GovernanceReplayResult,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub contract_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provider_sources: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorRequiredVerificationLaneEvidenceClasses {
    pub merge_check_id: String,
    pub lane_task: String,
    pub lane_kind: String,
    pub contract_sources: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorMergeGateSummaryEvidenceClasses {
    pub state: String,
    pub blocking: String,
    pub required_lane_count: String,
    pub drift_lane_count: String,
    pub decision_inputs: String,
    pub replay: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorMergeGateLaneEvidenceClasses {
    pub merge_check_id: String,
    pub lane_task: String,
    pub lane_kind: String,
    pub state: String,
    pub blocking: String,
    pub decision_inputs: String,
    pub replay: String,
    pub contract_sources: String,
    pub provider_sources: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorFixSummary {
    pub requested: bool,
    pub dry_run: bool,
    pub fixable_count: usize,
    pub planned_count: usize,
    pub applied_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<DoctorFixActionSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorFixActionSummary {
    pub key: String,
    pub kind: String,
    pub path: String,
    pub change: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preview: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct CheckSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: DoctorSummary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub finding_groups: Vec<DoctorFindingGroupSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentSummary<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub toolchains: Vec<ToolchainSelectionSummary>,
    pub findings: &'a [Finding],
}

#[derive(Debug, Serialize)]
pub struct WorkspaceDoctorSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: WorkspaceDoctorSummary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub finding_groups: Vec<DoctorFindingGroupSummary>,
    pub repos: &'a [WorkspaceRepoDoctorReport],
}

#[derive(Debug, Serialize, Default, Clone, PartialEq, Eq)]
pub struct DoctorSummary {
    pub verdict: DoctorVerdict,
    pub agent_verdict: DoctorVerdict,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_blocker: Option<DoctorPrimaryBlocker>,
}

#[derive(Debug, Serialize, Default, Clone, PartialEq, Eq)]
pub struct PolicyReviewSummary {
    pub ok: bool,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DoctorVerdict {
    #[default]
    Ready,
    Risky,
    NotReady,
    PolicyBlocked,
    AgentBlocked,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorPrimaryBlocker {
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct PolicyReviewSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub policy_source: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy_path: Option<String>,
    pub summary: PolicyReviewSummary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub finding_groups: Vec<DoctorFindingGroupSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<&'a OrgPolicyPack>,
    pub findings: &'a [Finding],
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct DoctorFindingGroupSummary {
    pub action_key: String,
    pub action_title: String,
    pub action_next: String,
    pub count: usize,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExplainSummary {
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub step_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExecutionReceiptSummary {
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub step_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ready_count: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_ready_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptStep {
    pub order: usize,
    pub label: String,
    pub stage_family: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_resolutions: Vec<TaskTargetResolutionEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_fulfillment: Option<BackendFulfillmentEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_local_backend: Option<SharedLocalBackendEvidence>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct DependencyPlaneProvenance {
    pub parent_task: String,
    pub dependency_task: String,
    pub parent_backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent_context: Option<String>,
    pub parent_backend_selection_source: String,
    pub dependency_backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_context: Option<String>,
    pub dependency_backend_selection_source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptEnvSource {
    pub name: String,
    pub value: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_status: Option<EnvSourceStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_label: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptNativeActivation {
    pub kind: String,
    pub applied: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub arch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shell: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptNativePrerequisite {
    pub name: String,
    pub required: bool,
    pub platform: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub check: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub activation: Option<ExecutionReceiptNativeActivation>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requires: Option<ExecutionReceiptNativeRequires>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub provisioning: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptNativeRequires {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub runtimes: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub tools: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub toolchains: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub checks: Vec<String>,
    pub source: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ToolchainSelectionSummary {
    pub name: String,
    pub provider: String,
    pub backend: String,
    pub target_os: String,
    pub version: String,
    pub fulfillment: String,
    pub required: bool,
    pub owns_runtime: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub fulfilled: Option<bool>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub commands: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub owns_tools: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub components: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ContractIdentityProject {
    pub name: String,
    #[serde(rename = "type", skip_serializing_if = "Option::is_none")]
    pub project_type: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ContractIdentityMetadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub team: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_class: Option<String>,
}

#[derive(Debug, Default, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ContractIdentityExecution {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supported: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ContractIdentityCounts {
    pub runtimes: usize,
    pub tools: usize,
    pub env: usize,
    pub services: usize,
    pub checks: usize,
    pub tasks: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repos: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policies: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ContractIdentity {
    pub version: u32,
    pub project: ContractIdentityProject,
    #[serde(default, skip_serializing_if = "contract_identity_metadata_is_empty")]
    pub metadata: ContractIdentityMetadata,
    #[serde(default, skip_serializing_if = "contract_identity_execution_is_empty")]
    pub execution: ContractIdentityExecution,
    pub counts: ContractIdentityCounts,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceipt {
    pub ok: bool,
    pub path: String,
    pub scope: String,
    pub contract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_identity: Option<ContractIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_snapshot_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_snapshot_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumption_set_hash: Option<String>,
    /// Immutable execution inputs captured while Ota issued this receipt.
    /// These are compared receipt-to-receipt; consumers must not substitute a later filesystem read.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evaluated_inputs: Vec<ExecutionReceiptEvaluatedInput>,
    /// Observed execution evidence. Unlike `evaluated_inputs`, these records are not
    /// current-run decision inputs and must not be treated as such during replay.
    #[serde(
        default,
        skip_serializing_if = "ExecutionReceiptWitnessedObservations::is_empty"
    )]
    pub witnessed_observations: ExecutionReceiptWitnessedObservations,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub crossing: Option<ExecutionBoundaryCrossing>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<GovernanceRefusalRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container_memory_bytes: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub acquired: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_sources: Vec<ExecutionReceiptEnvSource>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub workflow_env_artifacts: Vec<EnvRenderedArtifactEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub native_prerequisites: Vec<ExecutionReceiptNativePrerequisite>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub toolchains: Vec<ToolchainSelectionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<ResolvedTaskRuntime>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_termination: Option<crate::runner::ServiceTermination>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub host_service_cleanup: Vec<crate::runner::HostServiceCleanupEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_fulfillment: Option<BackendFulfillmentEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub logs: Option<ExecutionReceiptLogs>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub workloads: BTreeMap<String, ResolvedTaskRuntime>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub policy: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty", skip_deserializing)]
    pub dependency_steps: Vec<RunPreviewDependencyStep>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<ExecutionReceiptStep>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failed_dependency: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_origin: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocked: Vec<String>,
    pub summary: ExecutionReceiptSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

/// An execution input captured at receipt-authoring time.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptEvaluatedInput {
    pub id: String,
    pub kind: String,
    pub input_class: ReplayInputClass,
    pub identity: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifact_lineage: Option<ExecutionReceiptArtifactLineage>,
}

/// Receipt-owned observations from declared evidence artifacts.
#[derive(Debug, Serialize, Deserialize, Clone, Default, PartialEq, Eq)]
pub struct ExecutionReceiptWitnessedObservations {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub query_traces: Vec<ExecutionReceiptQueryTraceObservation>,
}

impl ExecutionReceiptWitnessedObservations {
    pub fn is_empty(&self) -> bool {
        self.query_traces.is_empty()
    }
}

/// A captured query-identity trace. The trace is attested historical evidence, not an input
/// evaluated by the current task execution.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptQueryTraceObservation {
    pub id: String,
    pub source_path: String,
    pub source_identity: String,
    pub evidence_class: ExecutionEvidenceClass,
    pub records: Vec<ExecutionReceiptQueryTraceRecord>,
    pub summary: ExecutionReceiptQueryTraceSummary,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptQueryTraceRecord {
    pub subject: String,
    pub run: u64,
    pub identity: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptQueryTraceSummary {
    pub subjects: usize,
    pub records: usize,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub divergent_subjects: Vec<ExecutionReceiptQueryTraceDivergence>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptQueryTraceDivergence {
    pub subject: String,
    pub distinct_identities: usize,
}

/// How Ota knows an observation. This is distinct from where a seam observation originated.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionEvidenceClass {
    Attested,
    Derived,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProofRuntimeDependencyObservation {
    pub origin: String,
    pub evidence_class: ExecutionEvidenceClass,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProofRuntimeDependencyEvidence {
    pub dependency_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub level: Option<String>,
    #[serde(skip_serializing_if = "std::ops::Not::not")]
    pub interaction_attempted: bool,
    pub observation: ProofRuntimeDependencyObservation,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub declared_by_tasks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub declared_by_workflows: Vec<String>,
}

/// Contract-declared lineage for a generated artifact consumed by the selected execution path.
#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptArtifactLineage {
    pub producer: String,
    pub paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub inputs: Vec<String>,
}

/// Canonical replay input families shared by receipt capture and comparison evidence.
#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReplayInputClass {
    ContractTruth,
    SourceIdentity,
    PolicyRulesetIdentity,
    DeclaredEnvSourceIdentity,
    DeclaredDependencyResolution,
    SelectedRuntimeVersion,
    SelectedRuntimeArtifact,
    ExecutionPresentationProfile,
    ComparatorSemantics,
    GeneratedArtifactLineage,
    DeclaredReplayInput,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ExecutionConflictReceipt {
    pub reasons: Vec<String>,
}

pub(crate) fn execution_receipt_conflict(blocked: &[String]) -> Option<ExecutionConflictReceipt> {
    let reasons = blocked
        .iter()
        .filter_map(|entry| entry.strip_prefix("execution_conflict:"))
        .filter(|reason| !reason.trim().is_empty())
        .map(str::to_string)
        .collect::<Vec<_>>();
    if reasons.is_empty() {
        None
    } else {
        Some(ExecutionConflictReceipt { reasons })
    }
}

fn execution_receipt_next_steps_json(next: Option<&str>) -> Vec<String> {
    next.map(|next| {
        next.split("; ")
            .map(str::trim)
            .filter(|part| !part.is_empty() && !part.starts_with("log capture failed:"))
            .map(str::to_string)
            .collect()
    })
    .unwrap_or_default()
}

impl Serialize for ExecutionReceipt {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(None)?;
        map.serialize_entry("ok", &self.ok)?;
        map.serialize_entry("path", &self.path)?;
        map.serialize_entry("scope", &self.scope)?;
        map.serialize_entry("contract", &self.contract)?;
        if let Some(contract_identity) = self.contract_identity.as_ref() {
            map.serialize_entry("contract_identity", contract_identity)?;
        }
        if let Some(contract_snapshot_hash) = self.contract_snapshot_hash.as_ref() {
            map.serialize_entry("contract_snapshot_hash", contract_snapshot_hash)?;
        }
        if let Some(contract_snapshot_ref) = self.contract_snapshot_ref.as_ref() {
            map.serialize_entry("contract_snapshot_ref", contract_snapshot_ref)?;
        }
        if let Some(assumption_set_hash) = self.assumption_set_hash.as_ref() {
            map.serialize_entry("assumption_set_hash", assumption_set_hash)?;
        }
        if !self.evaluated_inputs.is_empty() {
            map.serialize_entry("evaluated_inputs", &self.evaluated_inputs)?;
        }
        if !self.witnessed_observations.is_empty() {
            map.serialize_entry("witnessed_observations", &self.witnessed_observations)?;
        }
        if let Some(crossing) = self.crossing.as_ref() {
            map.serialize_entry("crossing", crossing)?;
        }
        if let Some(refusal) = self.refusal.as_ref() {
            map.serialize_entry("refusal", refusal)?;
        }
        if let Some(workspace) = self.workspace.as_ref() {
            map.serialize_entry("workspace", workspace)?;
        }
        if let Some(backend) = self.backend.as_ref() {
            map.serialize_entry("backend", backend)?;
        }
        if let Some(context) = self.context.as_ref() {
            map.serialize_entry("context", context)?;
        }
        if let Some(lifecycle) = self.lifecycle.as_ref() {
            map.serialize_entry("lifecycle", lifecycle)?;
        }
        if let Some(image) = self.image.as_ref() {
            map.serialize_entry("image", image)?;
        }
        if let Some(container_memory_bytes) = self.container_memory_bytes {
            map.serialize_entry("container_memory_bytes", &container_memory_bytes)?;
        }
        if let Some(target) = self.target.as_ref() {
            map.serialize_entry("target", target)?;
        }
        if let Some(provider) = self.provider.as_ref() {
            map.serialize_entry("provider", provider)?;
        }
        if let Some(cwd) = self.cwd.as_ref() {
            map.serialize_entry("cwd", cwd)?;
        }
        if !self.acquired.is_empty() {
            map.serialize_entry("acquired", &self.acquired)?;
        }
        if !self.env.is_empty() {
            map.serialize_entry("env", &self.env)?;
        }
        if !self.env_sources.is_empty() {
            map.serialize_entry("env_sources", &self.env_sources)?;
        }
        if !self.workflow_env_artifacts.is_empty() {
            map.serialize_entry("workflow_env_artifacts", &self.workflow_env_artifacts)?;
        }
        if !self.native_prerequisites.is_empty() {
            map.serialize_entry("native_prerequisites", &self.native_prerequisites)?;
        }
        if !self.toolchains.is_empty() {
            map.serialize_entry("toolchains", &self.toolchains)?;
        }
        if let Some(runtime) = self.runtime.as_ref() {
            map.serialize_entry("runtime", runtime)?;
        }
        if let Some(service_termination) = self.service_termination.as_ref() {
            map.serialize_entry("service_termination", service_termination)?;
        }
        if !self.host_service_cleanup.is_empty() {
            map.serialize_entry("host_service_cleanup", &self.host_service_cleanup)?;
        }
        if let Some(backend_fulfillment) = self.backend_fulfillment.as_ref() {
            map.serialize_entry("backend_fulfillment", backend_fulfillment)?;
        }
        if let Some(logs) = self.logs.as_ref() {
            map.serialize_entry("logs", logs)?;
        }
        if !self.workloads.is_empty() {
            map.serialize_entry("workloads", &self.workloads)?;
        }
        if !self.policy.is_empty() {
            map.serialize_entry("policy", &self.policy)?;
        }
        if !self.dependency_steps.is_empty() {
            map.serialize_entry("dependency_steps", &self.dependency_steps)?;
        }
        if !self.steps.is_empty() {
            map.serialize_entry("steps", &self.steps)?;
        }
        if let Some(status) = self.status.as_ref() {
            map.serialize_entry("status", status)?;
        }
        if let Some(failed_task) = self.failed_task.as_ref() {
            map.serialize_entry("failed_task", failed_task)?;
        }
        if let Some(failed_dependency) = self.failed_dependency.as_ref() {
            map.serialize_entry("failed_dependency", failed_dependency)?;
        }
        if let Some(failure_origin) = self.failure_origin.as_ref() {
            map.serialize_entry("failure_origin", failure_origin)?;
        }
        if !self.blocked.is_empty() {
            map.serialize_entry("blocked", &self.blocked)?;
        }
        if let Some(execution_conflict) = execution_receipt_conflict(&self.blocked) {
            map.serialize_entry("execution_conflict", &execution_conflict)?;
        }
        map.serialize_entry("summary", &self.summary)?;
        if let Some(next) = self.next.as_ref() {
            map.serialize_entry("next", next)?;
        }
        let next_steps = execution_receipt_next_steps_json(self.next.as_deref());
        if !next_steps.is_empty() {
            map.serialize_entry("next_steps", &next_steps)?;
        }
        map.end()
    }
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptLogs {
    pub dir: String,
    pub stdout: String,
    pub stderr: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ExecutionPlanResolved {
    pub backend: String,
    pub backend_source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub engine_candidates: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    pub target_strategy: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ExecutionPlanOverrides {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionPlanSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub contract: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<&'a str>,
    pub contract_identity: ContractIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_execution: Option<ExecutionSummary<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workflow_env_artifacts: Vec<EnvRenderedArtifactEntry>,
    pub resolved: ExecutionPlanResolved,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<ExecutionPlanOverrides>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionPlanFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone, PartialEq, Eq)]
pub struct RunPreviewPlan {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependency_chain: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependency_steps: Vec<RunPreviewDependencyStep>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requirement_lines: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub staged_actions: Vec<PreviewStageAction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct PreviewStageAction {
    pub stage_family: String,
    pub action: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct RunPreviewDependencyStep {
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub parent: Option<String>,
    pub backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub backend_selection_source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare: Option<WorkspaceTaskPrepareSummary>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct RunPreviewRunnableMode {
    pub mode: String,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub default: bool,
    pub command: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ArtifactRoute {
    pub role: String,
    pub kind: String,
    pub stage_family: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionBoundaryCrossing {
    pub id: String,
    pub created_at: String,
    pub lane_id: String,
    pub lane_kind: String,
    pub boundary_family: String,
    pub classification: String,
    pub requirement_source: String,
    pub actor_mode: String,
    pub principal_attribution_state: String,
    pub intent_source: String,
    pub reason_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub evidence_attachment_state: String,
    pub evidence_classes: ExecutionBoundaryCrossingEvidenceClasses,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct ExecutionBoundaryCrossingEvidenceClasses {
    pub id: String,
    pub created_at: String,
    pub lane_id: String,
    pub lane_kind: String,
    pub boundary_family: String,
    pub classification: String,
    pub requirement_source: String,
    pub actor_mode: String,
    pub principal_attribution_state: String,
    pub intent_source: String,
    pub reason_present: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub evidence_attachment_state: String,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct GovernanceRefusalRecord {
    pub reason_family: String,
    pub boundary_family: String,
    pub closure_status: String,
    pub requested_task: String,
    pub blocked_task: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub closure_path: Vec<String>,
    pub evidence_class: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GovernanceDecisionBasisEntry {
    pub id: String,
    pub family: String,
    pub evidence_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GovernanceDecisionInputEntry {
    pub id: String,
    pub family: String,
    pub evidence_class: String,
    pub replay_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GovernanceReplayResult {
    pub status: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub mismatches: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GovernancePreflightEvidenceClasses {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_required: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_safe_for_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_safe_for_agent: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub unsafe_closure_tasks: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_reason_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossing_required: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossing_classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossing_boundary_family: Option<String>,
    pub decision_inputs: String,
    pub replay: String,
    pub receipt_expected: String,
    pub proof_expected: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GovernancePreflightEvaluation {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub review_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_safe_for_agent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_safe_for_agent: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsafe_closure_tasks: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_reason_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<GovernanceRefusalRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossing_required: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossing_classification: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossing_boundary_family: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decision_basis: Vec<GovernanceDecisionBasisEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decision_inputs: Vec<GovernanceDecisionInputEntry>,
    pub replay: GovernanceReplayResult,
    pub evidence_classes: GovernancePreflightEvidenceClasses,
    pub receipt_expected: bool,
    pub proof_expected: bool,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GovernancePostExecutionEvidenceClasses {
    pub state: String,
    pub execution_attempted: String,
    pub refusal_occurred: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_reason_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_run_reason: Option<String>,
    pub crossing_record_state: String,
    pub decision_inputs: String,
    pub replay: String,
    pub receipt_present: String,
    pub proof_present: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_status: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GovernancePostExecutionEvidence {
    pub state: String,
    pub execution_attempted: bool,
    pub refusal_occurred: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal_reason_family: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub refusal: Option<GovernanceRefusalRecord>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub not_run_reason: Option<String>,
    pub crossing_record_state: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decision_basis: Vec<GovernanceDecisionBasisEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub decision_inputs: Vec<GovernanceDecisionInputEntry>,
    pub replay: GovernanceReplayResult,
    pub evidence_classes: GovernancePostExecutionEvidenceClasses,
    pub receipt_present: bool,
    pub proof_present: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receipt_status: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct GovernanceEvaluation {
    pub preflight: GovernancePreflightEvaluation,
    pub post_execution: GovernancePostExecutionEvidence,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_policy: Option<HarnessSandboxPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub crossing: Option<ExecutionBoundaryCrossing>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct RunPreviewGovernanceSummary {
    pub safety_posture: String,
    pub review_required: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub declared_safe_for_agent: bool,
    pub effective_safe_for_agent: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsafe_closure_tasks: Vec<String>,
    pub default_mode: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub runnable_modes: Vec<RunPreviewRunnableMode>,
    pub network: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_kind: Option<crate::schema::TaskNetworkEffectKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub writes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workspace_writes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adapter_state: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub external_state: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_policy: Option<HarnessSandboxPolicy>,
    pub receipt_follow_up_command: String,
    pub evaluation: GovernanceEvaluation,
}

#[derive(Debug, Serialize)]
pub struct RunPreviewSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub contract: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<&'a str>,
    pub task: &'a str,
    pub dry_run: bool,
    pub preview_status: &'a str,
    pub summary: DoctorSummary,
    pub contract_identity: ContractIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_execution: Option<ExecutionSummary<'a>>,
    pub resolved: ExecutionPlanResolved,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<ExecutionPlanOverrides>,
    pub requested_task: TaskSummary<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub requested_context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_context: Option<String>,
    pub env_summary: EnvSummary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<EnvSourceEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub toolchains: Vec<ToolchainSelectionSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub native_prerequisites: Vec<ExecutionReceiptNativePrerequisite>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisioning: Option<&'a ProvisioningPlan>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisioning_request: Option<&'a ProvisioningBackendRequest>,
    pub governance: RunPreviewGovernanceSummary,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_routing: Vec<ArtifactRoute>,
    pub plan: RunPreviewPlan,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologySuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub contract: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<&'a str>,
    pub contract_identity: ContractIdentity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_execution: Option<ExecutionSummary<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub shared_backends: Vec<ExecutionTopologySharedBackendSummary>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub readiness_probes: BTreeMap<String, ExecutionTopologyProbeSummary>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub surfaces: BTreeMap<String, ExecutionTopologySurfaceSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<ServiceSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub tasks: Vec<ExecutionTopologyTaskSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologySharedBackendSummary {
    pub name: String,
    pub scope: String,
    pub backend: String,
    pub lifecycle: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fulfillment: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment: Option<ExecutionTopologySharedBackendEnvironmentSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologySharedBackendEnvironmentSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image_alias: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyTaskSummary<'a> {
    #[serde(flatten)]
    pub task: TaskSummary<'a>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub runtime: Option<ExecutionTopologyRuntimeSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<ExecutionTopologyTargetSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyRuntimeSummary {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend_binding: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness: Option<ExecutionTopologyReadinessSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub attached_surfaces: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub surface_attachments: BTreeMap<String, ExecutionTopologySurfaceAttachmentSummary>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub listeners: BTreeMap<String, ExecutionTopologyListenerSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologySurfaceSummary {
    pub kind: String,
    pub port: u16,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub visibility: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness: Option<ExecutionTopologyReadinessSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologySurfaceAttachmentSummary {
    pub uses_defaults: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind: Option<ExecutionTopologySurfaceBindOverrideSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project: Option<ExecutionTopologySurfaceProjectionOverrideSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologySurfaceBindOverrideSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_value: Option<u16>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologySurfaceProjectionOverrideSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<ExecutionTopologySurfaceHostProjectionOverrideSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologySurfaceHostProjectionOverrideSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_value: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyProbeSummary {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<ExecutionTopologyProbeTargetSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<ExecutionTopologyReadinessSuccessSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<ExecutionTopologyReadinessBodySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyProbeTargetSummary {
    pub kind: String,
    pub name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listener: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub address_view: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observer: Option<ExecutionTopologyProbeObserverSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_plane: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyProbeObserverSummary {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyReadinessSummary {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listener: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<ExecutionTopologyReadinessSuccessSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<ExecutionTopologyReadinessBodySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyReadinessSuccessSummary {
    pub status: Vec<u16>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyReadinessBodySummary {
    pub contains: String,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyListenerSummary {
    pub protocol: String,
    pub bind_address: String,
    pub bind_port_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bind_port_value: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host_projection: Option<ExecutionTopologyHostProjectionSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyHostProjectionSummary {
    pub address: String,
    pub port_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port_value: Option<u16>,
    pub primary: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyTargetSummary {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub override_input: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    pub activation_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<ExecutionTopologyTargetServiceSummary>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionTopologyTargetServiceSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub member: Option<String>,
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listener: Option<String>,
    pub address_view: String,
}

#[derive(Debug, Serialize)]
pub struct ExplainStep {
    pub order: usize,
    pub code: String,
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExplainAction {
    pub order: usize,
    pub action_key: String,
    pub action_title: String,
    pub severity: FindingSeverity,
    pub count: usize,
    pub why: String,
    pub next: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExplainSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: ExplainSummary,
    pub actions: &'a [ExplainAction],
    pub steps: &'a [ExplainStep],
}

#[derive(Debug, Serialize)]
pub struct ExplainFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub error: &'a str,
}

#[derive(Debug, Serialize, Default, Clone, PartialEq, Eq)]
pub struct WorkspaceDoctorSummary {
    pub repo_count: usize,
    pub ready_count: usize,
    pub not_ready_count: usize,
    pub verdict: DoctorVerdict,
    pub agent_verdict: DoctorVerdict,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_blocker: Option<WorkspacePrimaryBlocker>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct WorkspacePrimaryBlocker {
    pub repo: String,
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub code: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_key: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceExplainSummary {
    pub repo_count: usize,
    pub ready_count: usize,
    pub not_ready_count: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub step_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoExplainReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    pub required: bool,
    pub ok: bool,
    pub summary: ExplainSummary,
    pub actions: Vec<ExplainAction>,
    pub steps: Vec<ExplainStep>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceExplainAction {
    pub repo: String,
    pub path: String,
    pub contract_path: String,
    pub required: bool,
    #[serde(flatten)]
    pub action: ExplainAction,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceExplainSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: WorkspaceExplainSummary,
    pub actions: &'a [WorkspaceExplainAction],
    pub repos: &'a [WorkspaceRepoExplainReport],
}

#[derive(Debug, Serialize)]
pub struct ExecutionContainerSummary<'a> {
    pub image: &'a str,
}

#[derive(Debug, Serialize)]
pub struct ExecutionRemoteSummary<'a> {
    pub provider: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionBackendsSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ExecutionContainerSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<ExecutionRemoteSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionEnvSummary<'a> {
    pub name: &'a str,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub policy: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionContextAttachmentsSummary<'a> {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub compose: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub isolated_paths: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub isolated_effective_paths: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionContextSummary<'a> {
    pub name: &'a str,
    pub backend: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub container: Option<ExecutionContainerSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remote: Option<ExecutionRemoteSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attachments: Option<ExecutionContextAttachmentsSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_context: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub supported: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backends: Option<ExecutionBackendsSummary<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contexts: Vec<ExecutionContextSummary<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<ExecutionEnvSummary<'a>>,
}

impl<'a> ExecutionSummary<'a> {
    pub fn from_contract(contract: &'a Contract, contract_path: &std::path::Path) -> Option<Self> {
        Self::from_contract_with_required_env_names(contract, contract_path, None)
    }

    pub fn from_contract_with_required_env_names(
        contract: &'a Contract,
        contract_path: &std::path::Path,
        selected_required_env_names: Option<&BTreeSet<String>>,
    ) -> Option<Self> {
        let execution = contract.execution.as_ref()?;
        let (policy_env, policy_label, policy_issue) = match load_policy_env_overlay(contract_path)
        {
            Ok(overlay) => (overlay.values, overlay.label, None),
            Err(_) => (
                BTreeMap::new(),
                String::new(),
                Some(String::from("invalid policy pack")),
            ),
        };
        let declared_sources = load_declared_env_sources(contract, contract_path);

        Some(Self {
            default_context: execution.default_context.as_deref(),
            preferred: execution.preferred.map(format_backend),
            supported: execution
                .supported
                .iter()
                .map(|backend| format_backend(*backend))
                .collect(),
            lifecycle: execution.lifecycle.map(format_lifecycle),
            backends: execution
                .backends
                .as_ref()
                .map(|backends| ExecutionBackendsSummary {
                    container: backends.container.as_ref().map(|container| {
                        ExecutionContainerSummary {
                            image: &container.image,
                        }
                    }),
                    remote: backends
                        .remote
                        .as_ref()
                        .map(|remote| ExecutionRemoteSummary {
                            provider: &remote.provider,
                            target: remote.target.as_deref(),
                            cwd: remote.cwd.as_deref(),
                        }),
                }),
            contexts: execution
                .contexts
                .iter()
                .map(|(name, context)| summarize_execution_context(name, context))
                .collect(),
            env: contract
                .env
                .iter()
                .map(|(name, requirement)| ExecutionEnvSummary {
                    name,
                    required: requirement.required
                        || selected_required_env_names.is_some_and(|names| names.contains(name)),
                    default: requirement.default.as_deref(),
                    policy: policy_env.get(name).cloned(),
                    source: blocking_declared_env_source_label(&declared_sources)
                        .or_else(|| policy_issue.clone())
                        .or_else(|| {
                            policy_env
                                .get(name)
                                .map(|_| policy_label.clone())
                                .or_else(|| {
                                    std::env::var(name).ok().map(|_| String::from("process"))
                                })
                                .or_else(|| {
                                    resolve_declared_env_source_value(name, &declared_sources)
                                        .map(|(_, source)| env_resolution_source_label(&source))
                                })
                                .or_else(|| {
                                    requirement
                                        .default
                                        .as_ref()
                                        .map(|_| String::from("default"))
                                })
                        })
                        .unwrap_or_else(|| String::from("missing")),
                    allowed: requirement.allowed.iter().map(String::as_str).collect(),
                })
                .collect(),
        })
    }
}

fn summarize_execution_context<'a>(
    name: &'a str,
    context: &'a ExecutionContext,
) -> ExecutionContextSummary<'a> {
    ExecutionContextSummary {
        name,
        backend: format_backend(context.backend),
        lifecycle: context.lifecycle.map(format_lifecycle),
        container: context
            .container
            .as_ref()
            .map(|container| ExecutionContainerSummary {
                image: &container.image,
            }),
        remote: context
            .remote
            .as_ref()
            .map(|remote| ExecutionRemoteSummary {
                provider: &remote.provider,
                target: remote.target.as_deref(),
                cwd: remote.cwd.as_deref(),
            }),
        attachments: (!context.attachments.compose.is_empty()
            || !context.attachments.isolated_paths.is_empty())
        .then(|| ExecutionContextAttachmentsSummary {
            compose: context
                .attachments
                .compose
                .iter()
                .map(String::as_str)
                .collect(),
            isolated_paths: context
                .attachments
                .isolated_paths
                .iter()
                .map(String::as_str)
                .collect(),
            isolated_effective_paths: crate::execution::context_dependency_isolation_paths(context)
                .into_iter()
                .map(|path| format!("/workspace/{path}"))
                .collect(),
        }),
    }
}

fn format_backend(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

fn format_lifecycle(lifecycle: Lifecycle) -> &'static str {
    match lifecycle {
        Lifecycle::Persistent => "persistent",
        Lifecycle::Ephemeral => "ephemeral",
    }
}

#[derive(Debug, Serialize)]
pub struct WorkspaceTaskSummary {
    pub name: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch: Option<WorkspaceTaskLaunchSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<WorkspaceTaskActionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare: Option<WorkspaceTaskPrepareSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<TaskAggregateSummary>,
    #[serde(default, skip_serializing_if = "TaskEffectsSummary::is_empty")]
    pub effects: TaskEffectsSummary,
    pub depends_on: Vec<String>,
    pub requires_services: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requires_artifacts: Vec<String>,
    pub after_success: Vec<String>,
    pub after_failure: Vec<String>,
    pub after_always: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoTasksReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    pub required: bool,
    pub acquired: bool,
    pub depends_on: Vec<String>,
    pub tasks: Vec<WorkspaceTaskSummary>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceTasksSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: WorkspaceTasksSummary,
    pub repos: &'a [WorkspaceRepoTasksReport],
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceTasksSummary {
    pub repo_count: usize,
    pub acquired_count: usize,
    pub task_count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct WorkspaceTaskLaunchSummary {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exe: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub detach: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub remove: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<WorkspaceTaskLaunchVolumeSummary>,
}

#[derive(Debug, Serialize, Clone)]
pub struct WorkspaceTaskLaunchVolumeSummary {
    pub kind: &'static str,
    pub source: String,
    pub target: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct WorkspaceTaskActionSummary {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoListReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    pub contract_present: bool,
    pub required: bool,
    pub acquired: bool,
    pub status: String,
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<WorkspaceExecutionSummary>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceListSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: WorkspaceListSummary,
    pub repos: &'a [WorkspaceRepoListReport],
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceListSummary {
    pub repo_count: usize,
    pub ready_count: usize,
    pub not_ready_count: usize,
    pub acquired_count: usize,
    pub missing_contract_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoExecutionPlanReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    pub required: bool,
    pub acquired: bool,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_identity: Option<ContractIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_execution: Option<WorkspaceExecutionSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved: Option<ExecutionPlanResolved>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceExecutionPlanSummary {
    pub repo_count: usize,
    pub resolved_count: usize,
    pub unresolved_count: usize,
    pub required_unresolved_count: usize,
    pub not_acquired_count: usize,
    pub missing_contract_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceExecutionPlanSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub summary: WorkspaceExecutionPlanSummary,
    pub repos: &'a [WorkspaceRepoExecutionPlanReport],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub overrides: Option<ExecutionPlanOverrides>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoUpReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    pub required: bool,
    pub ok: bool,
    pub status: String,
    pub phase: String,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_sources: Vec<ExecutionReceiptEnvSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceUpSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<&'a str>,
    pub summary: ExecutionReceiptSummary,
    pub receipt: ExecutionReceipt,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_routing: Vec<ArtifactRoute>,
    pub repos: &'a [WorkspaceRepoUpReport],
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoDiffReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    pub required: bool,
    pub acquired: bool,
    pub status: String,
    pub drift_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ahead: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behind: Option<usize>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dirty: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceDiffSummary {
    pub repo_count: usize,
    pub match_count: usize,
    pub different_count: usize,
    pub dirty_count: usize,
    pub missing_count: usize,
    pub missing_repo_count: usize,
    pub missing_contract_count: usize,
    pub unresolved_count: usize,
    pub target_unavailable_count: usize,
    pub comparison_unresolved_count: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceDiffSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub summary: WorkspaceDiffSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<&'a str>,
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub next_steps: &'a [String],
    pub repos: &'a [WorkspaceRepoDiffReport],
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoStatusReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    pub required: bool,
    pub acquired: bool,
    pub ready: bool,
    pub readiness_status: String,
    pub drift_status: String,
    pub drift_kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub head: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ahead: Option<usize>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub behind: Option<usize>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub dirty: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct WorkspaceStatusSummary {
    pub repo_count: usize,
    pub ready_count: usize,
    pub not_ready_count: usize,
    pub match_count: usize,
    pub different_count: usize,
    pub dirty_count: usize,
    pub missing_count: usize,
    pub missing_repo_count: usize,
    pub missing_contract_count: usize,
    pub unresolved_count: usize,
    pub target_unavailable_count: usize,
    pub comparison_unresolved_count: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceStatusSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub summary: WorkspaceStatusSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<&'a str>,
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub next_steps: &'a [String],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_routing: Vec<ArtifactRoute>,
    pub repos: &'a [WorkspaceRepoStatusReport],
}

#[derive(Debug, Serialize)]
pub struct WorkspaceReceiptSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub summary: ExecutionReceiptSummary,
    pub receipt: ExecutionReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_routing: Vec<ArtifactRoute>,
    pub repos: &'a [WorkspaceRepoStatusReport],
}

#[derive(Debug, Serialize)]
pub struct ReceiptSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<&'a str>,
    pub summary: ExecutionReceiptSummary,
    pub receipt: ExecutionReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_baseline: Option<ReceiptPromotedBaseline>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_routing: Vec<ArtifactRoute>,
    pub findings: &'a [Finding],
}

#[derive(Debug, Serialize, Clone)]
pub struct ReceiptPromotedBaseline {
    pub path: String,
    pub archive_path: String,
    pub promoted_at: String,
}

#[derive(Debug, Serialize)]
pub struct ReceiptHistorySummary {
    pub archive_count: usize,
    pub invalid_archive_count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReceiptHistoryEntry {
    pub archive_path: String,
    pub archived_at: String,
    pub ok: bool,
    pub contract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub summary: ExecutionReceiptSummary,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReceiptHistoryInvalidArchive {
    pub archive_path: String,
    pub error: String,
}

#[derive(Debug, Serialize)]
pub struct ReceiptHistorySuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub summary: ReceiptHistorySummary,
    pub archives: &'a [ReceiptHistoryEntry],
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub invalid_archives: &'a [ReceiptHistoryInvalidArchive],
}

#[derive(Debug, Serialize)]
pub struct ReceiptSnapshotSummary {
    pub input_count: usize,
    pub assumption_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ReceiptSnapshotContract<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_identity: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct ReceiptSnapshotSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub summary: ReceiptSnapshotSummary,
    pub source: &'a str,
    pub selection_kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_at: Option<&'a str>,
    pub snapshot_hash: &'a str,
    pub assumption_set_hash: &'a str,
    pub snapshot_path: &'a str,
    pub contract: ReceiptSnapshotContract<'a>,
    pub snapshot: JsonValue,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReceiptDiffCounts {
    pub count: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReceiptDiffComparison {
    pub baseline_identity_label: String,
    pub current_identity_label: String,
    pub identity_changed: bool,
    pub readiness_change: ReceiptDiffReadinessChange,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_snapshot_changed: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_trust: Vec<ReceiptDiffArtifactTrust>,
    /// A receipt diff compares witnesses; it does not itself rerun the selected lane.
    pub replay: ReceiptDiffReplayPosture,
    pub correlation: ReceiptDiffCorrelation,
}

/// The exact lane and trust boundary for replay-related receipt comparison output.
#[derive(Debug, Serialize, Clone)]
pub struct ReceiptDiffReplayPosture {
    pub scope: ReceiptDiffReplayScope,
    pub posture: ReceiptDiffReplayPostureKind,
    pub hermeticity: ReceiptDiffReplayHermeticity,
    pub reason: String,
}

#[derive(Debug, Serialize, Clone)]
pub struct ReceiptDiffReplayScope {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptDiffReplayPostureKind {
    WitnessOnly,
    ReplayVerified,
    ReplayFailed,
    ReplayUnavailable,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptDiffReplayHermeticity {
    Unassessed,
    Hermetic,
    PartlyAmbient,
    AmbientFreshDerivation,
}

/// Trust posture for a comparison artifact that was actually captured by both receipts.
/// `acquitting` applies only to the named input class, never to the entire execution outcome.
#[derive(Debug, Serialize, Clone)]
pub struct ReceiptDiffArtifactTrust {
    pub id: String,
    pub kind: String,
    pub input_classes: Vec<ReplayInputClass>,
    pub trust_role: ReceiptDiffArtifactTrustRole,
    pub baseline_identity: String,
    pub current_identity: String,
    pub comparison: ReceiptDiffArtifactComparison,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptDiffArtifactTrustRole {
    Acquitting,
    Narrowing,
    PointerOnly,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptDiffArtifactComparison {
    Matched,
    Changed,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptDiffReadinessChange {
    Unchanged,
    Improved,
    Regressed,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptDiffCorrelation {
    LikelyRelated,
    PossiblyRelated,
    NoClearCorrelation,
}

#[derive(Debug, Serialize)]
pub struct UpReplayExecution {
    pub baseline: UpReplayBaseline,
    pub scope: ReceiptDiffReplayScope,
    pub posture: ReceiptDiffReplayPostureKind,
    pub hermeticity: ReceiptDiffReplayHermeticity,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_kind: Option<UpReplayFailureKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub hidden_input_candidates: Vec<String>,
    pub reason: String,
    pub comparison: ReceiptDiffComparison,
    pub introduced: ReceiptDiffCounts,
    pub resolved: ReceiptDiffCounts,
    pub unchanged: ReceiptDiffCounts,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpReplayFailureKind {
    BaselineUnavailable,
    SemanticContractDrift,
    NamedInputDrift,
    HiddenInputSuspicion,
    WitnessMismatch,
}

#[derive(Debug, Serialize)]
pub struct UpReplayBaseline {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_at: Option<String>,
    pub ok: bool,
    pub last_known_good: UpReplayBaselineStatus,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum UpReplayBaselineStatus {
    ReplayVerified,
    StaleWitness,
    Unavailable,
}

#[derive(Debug, Serialize)]
pub struct ReceiptDiffSummary {
    pub baseline_ok: bool,
    pub current_ok: bool,
    pub comparison: ReceiptDiffComparison,
    pub introduced: ReceiptDiffCounts,
    pub resolved: ReceiptDiffCounts,
    pub unchanged: ReceiptDiffCounts,
}

#[derive(Debug, Serialize)]
pub struct ReceiptDiffGate {
    pub rule: String,
    pub passed: bool,
    pub new_blocker_count: usize,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_summary: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_next: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocking_provenance_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ReceiptDiffSide {
    pub ok: bool,
    pub contract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_identity_details: Option<ContractIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_snapshot_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_snapshot_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumption_set_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evaluated_inputs: Vec<ExecutionReceiptEvaluatedInput>,
    #[serde(
        default,
        skip_serializing_if = "ExecutionReceiptWitnessedObservations::is_empty"
    )]
    pub witnessed_observations: ExecutionReceiptWitnessedObservations,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub summary: ExecutionReceiptSummary,
}

#[derive(Debug, Serialize)]
pub struct ReceiptDiffBaseline {
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selection_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archived_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_at: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_identity: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_identity_details: Option<ContractIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_snapshot_hash: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub contract_snapshot_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assumption_set_hash: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub evaluated_inputs: Vec<ExecutionReceiptEvaluatedInput>,
    #[serde(
        default,
        skip_serializing_if = "ExecutionReceiptWitnessedObservations::is_empty"
    )]
    pub witnessed_observations: ExecutionReceiptWitnessedObservations,
    pub ok: bool,
    pub contract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub status: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    pub summary: ExecutionReceiptSummary,
}

#[derive(Debug, Serialize)]
pub struct ReceiptDiffSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub baseline: ReceiptDiffBaseline,
    pub current: ReceiptDiffSide,
    pub summary: ReceiptDiffSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub gate: Option<ReceiptDiffGate>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub contract_changes: Vec<DiffChange>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub likely_related_changes: Vec<DiffChange>,
    pub introduced: Vec<Finding>,
    pub resolved: Vec<Finding>,
    pub unchanged: Vec<Finding>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoRunReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    pub required: bool,
    pub ok: bool,
    pub status: String,
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_task: Option<String>,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_url: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_ref: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task_command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_sources: Vec<ExecutionReceiptEnvSource>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub next_steps: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRunSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub task: &'a str,
    pub summary: ExecutionReceiptSummary,
    pub receipt: ExecutionReceipt,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_routing: Vec<ArtifactRoute>,
    pub repos: &'a [WorkspaceRepoRunReport],
}

#[derive(Debug, Serialize)]
pub struct InitSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub written: bool,
    pub mode: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_options: Option<InitSelectedPackOptions>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub pack_advisory: Option<InitPackAdvisory>,
    pub config: &'a DetectContract,
    pub inferred: &'a [Inference],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub toolchain_opportunities: Vec<ToolchainOpportunityAdvisory>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub provenance: Vec<ContractFieldProvenance>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ToolchainOpportunityAdvisory {
    pub ecosystem: String,
    pub fallback_runtime: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub fallback_tools: Vec<String>,
    pub candidate_providers: Vec<String>,
    pub shipped: bool,
    pub agent_note: String,
}

#[derive(Debug, Serialize)]
pub struct InitPackAdvisorySignal {
    pub signal: String,
    pub weight: usize,
}

#[derive(Debug, Serialize)]
pub struct InitPackAdvisory {
    pub selected_pack: String,
    pub suggested_pack: String,
    pub selected_pack_score: usize,
    pub suggested_pack_score: usize,
    pub score_gap: usize,
    pub summary: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub signals: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub signal_details: Vec<InitPackAdvisorySignal>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub selected_signal_details: Vec<InitPackAdvisorySignal>,
    pub next: String,
}

#[derive(Debug, Serialize)]
pub struct InitSelectedPackOptions {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub package_manager: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub test_runner: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InitPackSeeds {
    pub toolchains: Vec<String>,
    pub runtimes: Vec<String>,
    pub tools: Vec<String>,
    pub checks: Vec<String>,
    pub tasks: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InitPackOption {
    pub flag: String,
    pub summary: String,
    pub default: String,
    pub values: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct InitPackInfo {
    pub name: String,
    pub summary: String,
    pub when: String,
    pub command: String,
    pub next: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub options: Vec<InitPackOption>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub does_not_infer: Vec<String>,
    pub seeds: InitPackSeeds,
}

#[derive(Debug, Serialize)]
pub struct InitPackCatalogSuccess {
    pub ok: bool,
    pub mode: &'static str,
    pub packs: Vec<InitPackInfo>,
}

#[derive(Debug, Serialize)]
pub struct InitFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub written: bool,
    pub error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct PolicyInitSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub written: bool,
    pub mode: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<&'a str>,
    pub config: JsonValue,
}

#[derive(Debug, Serialize)]
pub struct PolicyInitFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub written: bool,
    pub mode: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preset: Option<&'a str>,
    pub error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct AgentsSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub output: &'a str,
    pub written: bool,
    pub mode: &'a str,
    pub content: &'a str,
}

#[derive(Debug, Serialize)]
pub struct AgentsFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub written: bool,
    pub error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct DetectSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub written: bool,
    pub config: JsonValue,
    pub inferred: &'a [Inference],
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub toolchain_opportunities: Vec<ToolchainOpportunityAdvisory>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub comparison: Option<&'a DetectComparison>,
}

#[derive(Debug, Serialize)]
pub struct DetectFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub written: bool,
    pub error: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<&'a str>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ContractFieldProvenance {
    pub field: String,
    pub provenance: String,
    pub provenance_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectComparison {
    pub existing_contract: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub changes: Vec<DetectComparisonChange>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub removals: Vec<DetectComparisonRemoval>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectComparisonChange {
    pub field: String,
    pub status: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub existing: Option<String>,
    pub detected: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_key: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub confidence: Option<Confidence>,
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectComparisonRemoval {
    pub field: String,
    pub existing: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub owner_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ownership: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance_key: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct UpStatus<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub status: &'a str,
    pub phase: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cause: Option<&'a str>,
    pub governance: GovernanceEvaluation,
    pub findings: &'a [Finding],
    pub receipt: ExecutionReceipt,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_routing: Vec<ArtifactRoute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Serialize)]
pub struct ProofRuntimeArtifacts<'a> {
    pub topology: &'a str,
    pub doctor: &'a str,
    pub up_log: &'a str,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProofRuntimeLikelyCauseEvidence {
    pub kind: String,
    pub artifact: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub signal: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listener: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub variable: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub host: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub port: Option<u16>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub observed_target: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProofRuntimeScope {
    pub kind: String,
    pub proof_class: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<String>,
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct ProofRuntimeNotProved {
    pub kind: String,
    pub relative_to: String,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dependency_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub declared_by_tasks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub declared_by_workflows: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct ProofRuntimeStatus<'a> {
    pub ok: bool,
    /// Terminal evaluation of the selected proof carrier. This is deliberately separate from
    /// `ok` so consumers cannot collapse a qualified proof into an unbounded pass.
    pub proof_verdict: &'a str,
    pub path: &'a str,
    pub mode: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<&'a str>,
    pub phase: &'a str,
    pub stage_family: &'a str,
    pub proof_scope: ProofRuntimeScope,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependency_evidence: Vec<ProofRuntimeDependencyEvidence>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub not_proved: Vec<ProofRuntimeNotProved>,
    pub summary: DoctorSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<ProofRuntimeArtifacts<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workflow_env_artifacts: Vec<EnvRenderedArtifactEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub artifact_routing: Vec<ArtifactRoute>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub failure_class: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cleanup_failure: Option<JsonValue>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likely_cause: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub likely_cause_evidence: Option<ProofRuntimeLikelyCauseEvidence>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<&'a str>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpPreviewExecution {
    pub backend: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct UpPreviewPlan {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub staged_actions: Vec<PreviewStageAction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub skipped: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub staged_skipped: Vec<PreviewStageAction>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependency_chain: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub dependency_steps: Vec<RunPreviewDependencyStep>,
}

#[derive(Debug, Serialize)]
pub struct UpPreviewStatus<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub dry_run: bool,
    pub status: &'a str,
    pub preview_status: &'a str,
    pub phase: &'a str,
    pub summary: DoctorSummary,
    pub contract_identity: ContractIdentity,
    pub execution: UpPreviewExecution,
    pub plan: UpPreviewPlan,
    pub governance: GovernanceEvaluation,
    #[serde(skip_serializing_if = "<[Finding]>::is_empty")]
    pub blockers: &'a [Finding],
}

impl CommandOutput {
    pub fn success(stdout: String) -> Self {
        Self {
            stdout,
            stderr: None,
            exit_code: 0,
        }
    }

    pub fn status(exit_code: i32) -> Self {
        Self {
            stdout: String::new(),
            stderr: None,
            exit_code,
        }
    }

    pub fn failure(stderr: String) -> Self {
        Self {
            stdout: String::new(),
            stderr: Some(stderr),
            exit_code: 1,
        }
    }

    pub fn failure_with_code(stderr: String, exit_code: i32) -> Self {
        Self {
            stdout: String::new(),
            stderr: Some(stderr),
            exit_code,
        }
    }

    pub fn with_stderr(mut self, stderr: Option<String>) -> Self {
        self.stderr = match (self.stderr.take(), stderr) {
            (None, None) => None,
            (Some(existing), None) => Some(existing),
            (None, Some(extra)) => Some(extra),
            (Some(existing), Some(extra)) => Some(format!("{existing}\n{extra}")),
        };
        self
    }
}

#[derive(Debug, Serialize)]
pub struct ValidateSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ValidateSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warning_details: Vec<ValidateWarning>,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct ValidateSummary {
    pub error_count: usize,
    pub warn_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ValidateFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ValidateSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warnings: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub warning_details: Vec<ValidateWarning>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ValidateWarning {
    pub code: String,
    pub category: String,
    pub owner: String,
    pub severity: String,
    pub summary: String,
    pub why: String,
    pub next: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<DependencyPlaneProvenance>,
}

#[derive(Debug, Serialize)]
pub struct EnvSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<&'a str>,
    pub summary: EnvSummary,
    pub sources: Vec<EnvSourceEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub rendered_artifacts: Vec<EnvRenderedArtifactEntry>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<EnvEntry>,
}

#[derive(Debug, Serialize)]
pub struct EnvFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<&'a str>,
    pub error: &'a str,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct EnvSummary {
    pub contract_count: usize,
    pub source_count: usize,
    pub source_issue_count: usize,
    pub task_count: usize,
    pub resolved_count: usize,
    pub missing_count: usize,
    pub invalid_count: usize,
}

#[derive(Debug, Serialize, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvSourceStatus {
    Loaded,
    MissingOptional,
    MissingRequired,
    ParseFailed,
    InvalidStructure,
    Collision,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct EnvSourceEntry {
    pub kind: String,
    pub path: String,
    pub label: String,
    pub must_exist: bool,
    pub status: EnvSourceStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Deserialize, Clone, PartialEq, Eq)]
pub struct EnvRenderedArtifactEntry {
    pub path: String,
    pub kind: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub includes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    pub exists: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profile: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub consumers: Vec<String>,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvEntryKind {
    Contract,
    Task,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvEntryStatus {
    Resolved,
    Missing,
    Optional,
    Invalid,
    Task,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct EnvEntry {
    pub name: String,
    pub kind: EnvEntryKind,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub value: Option<String>,
    pub source: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_status: Option<EnvSourceStatus>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_label: Option<String>,
    pub status: EnvEntryStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct DiffSummary {
    pub added_count: usize,
    pub removed_count: usize,
    pub changed_count: usize,
    pub weakened_count: usize,
    pub strengthened_count: usize,
    pub readiness_impact: &'static str,
}

#[derive(Debug, Serialize, Clone)]
pub struct DiffChange {
    pub path: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub risk: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiffInputSide<'a> {
    pub path: &'a str,
    pub kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub snapshot_path: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct DiffSuccess<'a> {
    pub ok: bool,
    pub base: &'a str,
    pub target: &'a str,
    pub base_input: DiffInputSide<'a>,
    pub target_input: DiffInputSide<'a>,
    pub summary: DiffSummary,
    pub changes: &'a [DiffChange],
}

#[derive(Debug, Serialize)]
pub struct DiffFailure<'a> {
    pub ok: bool,
    pub base: &'a str,
    pub target: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base_input: Option<DiffInputSide<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target_input: Option<DiffInputSide<'a>>,
    pub error: &'a str,
}

#[derive(Debug, Serialize)]
pub struct TasksSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_profile: Option<HarnessCapabilityProfile>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub artifacts: BTreeMap<String, GeneratedArtifactSpec>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<MemberTasksSuccess<'a>>,
    pub tasks: Vec<TaskSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct MemberTasksSuccess<'a> {
    pub member: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<WorkflowSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_profile: Option<HarnessCapabilityProfile>,
    pub tasks: Vec<TaskSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct TasksFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowsSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_profile: Option<HarnessCapabilityProfile>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<MemberWorkflowsSuccess<'a>>,
    pub workflows: Vec<ListedWorkflowSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct MemberWorkflowsSuccess<'a> {
    pub member: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub capability_profile: Option<HarnessCapabilityProfile>,
    pub workflows: Vec<ListedWorkflowSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowsFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ServicesSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<MemberServicesSuccess>,
    pub services: Vec<ServiceSummary>,
}

#[derive(Debug, Serialize)]
pub struct MemberServicesSuccess {
    pub member: String,
    pub services: Vec<ServiceSummary>,
}

#[derive(Debug, Serialize)]
pub struct ServicesFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct AgentSummary<'a> {
    pub posture: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_task: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub safe_tasks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verify_after_changes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub writable_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub protected_paths: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub inferred_boundary_reviewed: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<AgentBootstrapSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessCapabilityProfile {
    pub actor_mode: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub posture: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_task: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub verify_after_changes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub writable_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub protected_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub callable_tasks: Vec<HarnessLaneCapability>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub refused_tasks: Vec<HarnessLaneCapability>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub callable_workflows: Vec<HarnessLaneCapability>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub refused_workflows: Vec<HarnessLaneCapability>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessSandboxPolicy {
    pub target: String,
    pub filesystem: HarnessSandboxFilesystemPolicy,
    pub network: HarnessSandboxNetworkPolicy,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessSandboxFilesystemPolicy {
    pub state: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub repo_root_mode: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub writable_paths: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub protected_paths: Vec<String>,
    pub source: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessSandboxNetworkPolicy {
    pub state: String,
    pub default: String,
    pub scope: String,
    pub enforcement: String,
    pub source: String,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub outbound_targets: Vec<HarnessSandboxOutboundTarget>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessSandboxOutboundTarget {
    pub kind: String,
    pub value: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_shape: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub destination_constraint: Option<HarnessSandboxDestinationConstraint>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessSandboxDestinationConstraint {
    pub kind: String,
    pub values: Vec<String>,
    pub source_posture: String,
    pub enforcement: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub shared_pin: Option<HarnessSandboxSharedPin>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessSandboxSharedPin {
    pub r#ref: String,
    pub freshness: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessLaneCapability {
    pub lane_id: String,
    pub lane_kind: String,
    pub name: String,
    pub command: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub environment_boundary: Option<HarnessEnvironmentBoundary>,
    pub preflight: GovernancePreflightEvaluation,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sandbox_policy: Option<HarnessSandboxPolicy>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effects: Option<TaskEffectsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blocked_task: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub closure_path: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct HarnessEnvironmentBoundary {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub primary_task: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkflowSummary<'a> {
    pub name: &'a str,
    #[serde(rename = "use")]
    pub usage: LaneUseSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub instance: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub intent: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare_task: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare_action: Option<TaskActionSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub setup_task: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_task: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub attach_task: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run_task_launch: Option<TaskLaunchSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_safe_for_agent: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub effective_safe_for_agent: Option<bool>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsafe_closure_tasks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub required_services: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub readiness_checks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub readiness_probes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub readiness_surfaces: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub signal_readiness_checks: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub signal_readiness_probes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub signal_readiness_surfaces: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub exposes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub expose_surfaces: Vec<String>,
    #[serde(skip)]
    pub expose_entries: Vec<WorkflowExposeEntry>,
}

#[derive(Debug, Clone)]
pub struct WorkflowExposeEntry {
    pub url: String,
    pub surface: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ListedWorkflowSummary<'a> {
    #[serde(flatten)]
    pub workflow: WorkflowSummary<'a>,
    pub default: bool,
}

impl<'a> WorkflowSummary<'a> {
    pub fn from_contract(contract: &'a Contract) -> Option<Self> {
        Self::from_contract_selected_inner(contract, None, None)
    }

    pub fn from_contract_with_path(contract: &'a Contract, contract_path: &Path) -> Option<Self> {
        Self::from_contract_selected_with_path(contract, contract_path, None)
    }

    pub fn from_contract_named(contract: &'a Contract, workflow_name: &'a str) -> Option<Self> {
        Self::from_contract_named_inner(contract, None, workflow_name)
    }

    pub fn from_contract_named_with_path(
        contract: &'a Contract,
        contract_path: &Path,
        workflow_name: &'a str,
    ) -> Option<Self> {
        Self::from_contract_named_inner(contract, Some(contract_path), workflow_name)
    }

    fn from_contract_named_inner(
        contract: &'a Contract,
        contract_path: Option<&Path>,
        workflow_name: &'a str,
    ) -> Option<Self> {
        let workflow = contract.workflow(workflow_name)?;
        let workflow_safety = crate::cli::workflow_effective_safety(contract, workflow_name);
        let mut exposes = Vec::new();
        let mut expose_surfaces = Vec::new();
        let mut expose_entries = Vec::new();
        for expose in &workflow.exposes {
            match expose {
                crate::schema::WorkflowExposeSpec::Url(url) => {
                    exposes.push(url.clone());
                    expose_entries.push(WorkflowExposeEntry {
                        url: url.clone(),
                        surface: None,
                    });
                }
                crate::schema::WorkflowExposeSpec::SurfaceRef { surface } => {
                    expose_surfaces.push(surface.clone());
                    if let Some(url) = workflow.run.as_ref().and_then(|run| {
                        workflow_surface_host_url(
                            contract,
                            contract_path,
                            run.task.as_str(),
                            surface,
                        )
                    }) {
                        exposes.push(url.clone());
                        expose_entries.push(WorkflowExposeEntry {
                            url,
                            surface: Some(surface.clone()),
                        });
                    }
                }
            }
        }
        Some(Self {
            name: workflow_name,
            usage: workflow_lane_use_summary(workflow_name, workflow_safety.effective_safe),
            instance: None,
            intent: workflow.intent.as_deref(),
            description: workflow.description.as_deref(),
            notes: workflow.notes.as_deref(),
            prepare_task: workflow
                .prepare
                .as_ref()
                .and_then(|phase| phase.task.as_deref()),
            prepare_action: workflow
                .prepare
                .as_ref()
                .and_then(|phase| summarize_task_action(phase.action.as_ref())),
            setup_task: workflow.setup.as_ref().map(|phase| phase.task.as_str()),
            run_task: workflow.run.as_ref().map(|phase| phase.task.as_str()),
            attach_task: workflow.attach.as_ref().map(|phase| phase.task.as_str()),
            run_task_launch: workflow
                .run
                .as_ref()
                .and_then(|phase| contract.tasks.get(phase.task.as_str()))
                .and_then(|task| {
                    let backend = task.workflow_backend(contract.execution.as_ref());
                    task.resolved_execution_for_backend(backend, current_os())
                })
                .and_then(|execution| summarize_task_launch(execution.launch())),
            declared_safe_for_agent: workflow_safety.declared_safe,
            effective_safe_for_agent: workflow_safety.effective_safe,
            unsafe_closure_tasks: workflow_safety.unsafe_closure_tasks,
            required_services: contract
                .selected_workflow_required_service_names(Some(workflow_name)),
            readiness_checks: workflow.readiness.checks.clone(),
            readiness_probes: workflow.readiness.probes.clone(),
            readiness_surfaces: workflow.readiness.surfaces.clone(),
            signal_readiness_checks: workflow.readiness.signal.checks.clone(),
            signal_readiness_probes: workflow.readiness.signal.probes.clone(),
            signal_readiness_surfaces: workflow.readiness.signal.surfaces.clone(),
            exposes,
            expose_surfaces,
            expose_entries,
        })
    }

    pub fn from_contract_selected(
        contract: &'a Contract,
        workflow_name: Option<&str>,
    ) -> Option<Self> {
        Self::from_contract_selected_inner(contract, None, workflow_name)
    }

    pub fn from_contract_selected_with_path(
        contract: &'a Contract,
        contract_path: &Path,
        workflow_name: Option<&str>,
    ) -> Option<Self> {
        Self::from_contract_selected_inner(contract, Some(contract_path), workflow_name)
    }

    fn from_contract_selected_inner(
        contract: &'a Contract,
        contract_path: Option<&Path>,
        workflow_name: Option<&str>,
    ) -> Option<Self> {
        let (name, _) = contract.selected_workflow(workflow_name)?;
        let mut summary = Self::from_contract_named_inner(contract, contract_path, name)?;
        summary.instance = contract.selected_workflow_instance_name(workflow_name);
        let workflow_selector = match summary.instance.as_deref() {
            Some(instance) => format!("{}@{}", summary.name, instance),
            None => summary.name.to_string(),
        };
        summary.usage =
            workflow_lane_use_summary(workflow_selector.as_str(), summary.effective_safe_for_agent);
        Some(summary)
    }

    pub fn list_from_contract(contract: &'a Contract) -> Vec<ListedWorkflowSummary<'a>> {
        Self::list_from_contract_inner(contract, None)
    }

    pub fn list_from_contract_with_path(
        contract: &'a Contract,
        contract_path: &Path,
    ) -> Vec<ListedWorkflowSummary<'a>> {
        Self::list_from_contract_inner(contract, Some(contract_path))
    }

    fn list_from_contract_inner(
        contract: &'a Contract,
        contract_path: Option<&Path>,
    ) -> Vec<ListedWorkflowSummary<'a>> {
        let default_name = contract
            .workflows
            .as_ref()
            .map(|workflows| workflows.default.as_str());
        contract
            .workflows
            .as_ref()
            .map(|workflows| {
                workflows
                    .items
                    .keys()
                    .filter_map(|name| {
                        Self::from_contract_named_inner(contract, contract_path, name.as_str()).map(
                            |workflow| ListedWorkflowSummary {
                                default: default_name == Some(name.as_str()),
                                workflow,
                            },
                        )
                    })
                    .collect()
            })
            .unwrap_or_default()
    }
}

fn workflow_surface_host_url(
    contract: &Contract,
    contract_path: Option<&Path>,
    task_name: &str,
    surface_name: &str,
) -> Option<String> {
    let task = contract.tasks.get(task_name)?;
    let backend = workflow_surface_backend(contract, contract_path, task_name, task)?;
    let runtime = task.service_runtime_for_backend(backend)?;
    if !runtime.surfaces.contains_name(surface_name) {
        return None;
    }
    let listener = runtime.listeners.get(surface_name)?;
    let host = listener.project.host.as_ref()?;
    let port = host.port.value?;
    let path = host.path.as_deref().unwrap_or("");
    match listener.protocol {
        crate::schema::TaskRuntimeProtocol::Http => {
            Some(format!("http://{}:{}{}", host.address, port, path))
        }
        crate::schema::TaskRuntimeProtocol::Https => {
            Some(format!("https://{}:{}{}", host.address, port, path))
        }
        crate::schema::TaskRuntimeProtocol::Tcp => Some(format!("tcp://{}:{}", host.address, port)),
    }
}

fn workflow_surface_backend(
    contract: &Contract,
    contract_path: Option<&Path>,
    task_name: &str,
    task: &TaskSpec,
) -> Option<Backend> {
    if let Some(contract_path) = contract_path {
        let backend = match resolve_execution_backend_with_contract_path(
            contract,
            task_name,
            ExecutionOverrides::default(),
            Some(contract_path),
        )
        .ok()?
        {
            ResolvedExecutionBackend::Native { .. } => Backend::Native,
            ResolvedExecutionBackend::Container { .. } => Backend::Container,
            ResolvedExecutionBackend::Remote { .. }
            | ResolvedExecutionBackend::BackendProvider { .. } => Backend::Remote,
        };
        return Some(backend);
    }

    Some(task.workflow_backend(contract.execution.as_ref()))
}

impl<'a> AgentSummary<'a> {
    pub fn from_config(agent: &'a AgentConfig) -> Option<Self> {
        let summary = Self {
            posture: agent.posture.as_str(),
            entrypoint: agent.entrypoint.as_deref(),
            default_task: agent.default_task.as_deref(),
            safe_tasks: agent.safe_tasks.clone(),
            verify_after_changes: agent.verify_after_changes.clone(),
            writable_paths: agent.writable_paths.clone(),
            protected_paths: agent.protected_paths.clone(),
            inferred_boundary_reviewed: agent
                .inferred_boundary
                .as_ref()
                .map(|boundary| boundary.reviewed),
            bootstrap: agent
                .bootstrap
                .as_ref()
                .and_then(AgentBootstrapSummary::from_config),
            notes: agent.notes.as_deref(),
        };

        (!summary.is_empty()).then_some(summary)
    }

    pub fn retain_visible_tasks(&mut self, visible_task_names: &BTreeSet<String>) {
        if self
            .entrypoint
            .is_some_and(|entrypoint| !visible_task_names.contains(entrypoint))
        {
            self.entrypoint = None;
        }
        if self
            .default_task
            .is_some_and(|default_task| !visible_task_names.contains(default_task))
        {
            self.default_task = None;
        }
        self.safe_tasks
            .retain(|task| visible_task_names.contains(task.as_str()));
        self.verify_after_changes
            .retain(|task| visible_task_names.contains(task.as_str()));
    }

    pub fn is_empty(&self) -> bool {
        self.posture == "readiness_strict"
            && self.entrypoint.is_none()
            && self.default_task.is_none()
            && self.safe_tasks.is_empty()
            && self.verify_after_changes.is_empty()
            && self.writable_paths.is_empty()
            && self.protected_paths.is_empty()
            && self.inferred_boundary_reviewed.is_none()
            && self.bootstrap.is_none()
            && self.notes.is_none()
    }
}

#[derive(Debug, Serialize)]
pub struct AgentBootstrapSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub ota: Option<AgentBootstrapTargetSummary<'a>>,
}

impl<'a> AgentBootstrapSummary<'a> {
    pub fn from_config(bootstrap: &'a crate::schema::AgentBootstrapConfig) -> Option<Self> {
        let summary = Self {
            ota: bootstrap
                .ota
                .as_ref()
                .map(AgentBootstrapTargetSummary::from_config),
        };

        summary.ota.is_some().then_some(summary)
    }
}

#[derive(Debug, Serialize)]
pub struct AgentBootstrapTargetSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub note: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source: Option<AgentBootstrapOtaSourceSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sh: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powershell: Option<String>,
}

impl<'a> AgentBootstrapTargetSummary<'a> {
    pub fn from_config(bootstrap: &'a crate::schema::AgentBootstrapTargetConfig) -> Self {
        Self {
            note: bootstrap.note.as_deref(),
            source: bootstrap
                .effective_source()
                .as_ref()
                .map(AgentBootstrapOtaSourceSummary::from_source),
            sh: bootstrap.rendered_sh().map(|value| value.into_owned()),
            powershell: bootstrap
                .rendered_powershell()
                .map(|value| value.into_owned()),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct AgentBootstrapOtaSourceSummary<'a> {
    pub kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub version: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub rev: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
    pub deterministic: bool,
    pub pressure_only: bool,
}

impl<'a> AgentBootstrapOtaSourceSummary<'a> {
    fn from_source(source: &crate::schema::AgentBootstrapOtaSource) -> Self {
        match source {
            crate::schema::AgentBootstrapOtaSource::Version { version } => Self {
                kind: "version",
                version: Some(version.clone()),
                rev: None,
                branch: None,
                deterministic: true,
                pressure_only: false,
            },
            crate::schema::AgentBootstrapOtaSource::GitRev { rev } => Self {
                kind: "git_rev",
                version: None,
                rev: Some(rev.clone()),
                branch: None,
                deterministic: true,
                pressure_only: false,
            },
            crate::schema::AgentBootstrapOtaSource::Branch { branch } => Self {
                kind: "branch",
                version: None,
                rev: None,
                branch: Some(branch.clone()),
                deterministic: false,
                pressure_only: true,
            },
        }
    }
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskSummary<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<&'static str>,
    #[serde(skip_serializing)]
    pub effective_default_mode: &'static str,
    #[serde(rename = "use")]
    pub usage: LaneUseSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<&'a str>,
    #[serde(skip_serializing)]
    pub preview: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch_preview: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<String>,
    #[serde(skip_serializing_if = "TaskAdapterInputsSummary::is_empty")]
    pub adapter_inputs: TaskAdapterInputsSummary,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub inputs: BTreeMap<String, TaskInputSpec>,
    pub kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<TaskCommandSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose: Option<TaskComposeExecutionSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch: Option<TaskLaunchSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<TaskActionSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare: Option<TaskPrepareSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub aggregate: Option<TaskAggregateSummary>,
    #[serde(default, skip_serializing_if = "TaskEffectsSummary::is_empty")]
    pub effects: TaskEffectsSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_variant_os: Option<&'a str>,
    pub depends_on: Vec<String>,
    pub requires_services: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub requires_artifacts: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub when_checks: Vec<String>,
    pub after_success: Vec<String>,
    pub after_failure: Vec<String>,
    pub after_always: Vec<String>,
    pub safe_for_agent: bool,
    pub effective_safe_for_agent: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub unsafe_closure_tasks: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub internal: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<TaskVariantView<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub modes: Vec<TaskModeView<'a>>,
    #[serde(skip_serializing)]
    pub supports_native_mode_override: bool,
}

#[derive(Debug, Serialize, Clone, Default, PartialEq, Eq)]
pub struct TaskEffectsSummary {
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub writes: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workspace_writes: Vec<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub network: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_kind: Option<crate::schema::TaskNetworkEffectKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub adapter_state: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub external_state: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TaskAggregateSummary {
    pub tasks: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct LaneUseSummary {
    /// Compatibility projection for the selected human execution mode.
    pub human: String,
    /// Compatibility projection for the selected agent execution mode.
    pub agent: LaneUseInvocationSummary,
    /// Canonical task execution-mode matrix. Workflows currently leave this empty because their
    /// mode selection is owned by the selected task path rather than the workflow declaration.
    pub modes: Vec<LaneUseModeSummary>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct LaneUseInvocationSummary {
    pub callable: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
}

/// Backward-compatible Rust name for the selected agent invocation projection.
pub type AgentLaneUseSummary = LaneUseInvocationSummary;

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct LaneUseModeSummary {
    pub mode: String,
    pub default: bool,
    pub availability: LaneUseModeAvailability,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub reason: Option<String>,
    pub human: LaneUseInvocationSummary,
    pub agent: LaneUseInvocationSummary,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum LaneUseModeAvailability {
    Supported,
    Unavailable,
}

#[derive(Debug, Serialize, Clone, Default, PartialEq, Eq)]
pub struct TaskAdapterInputsSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose: Option<TaskComposeAdapterInputsSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake: Option<TaskBakeAdapterInputsSummary>,
}

impl TaskAdapterInputsSummary {
    pub fn is_empty(&self) -> bool {
        self.compose
            .as_ref()
            .is_none_or(TaskComposeAdapterInputsSummary::is_empty)
            && self
                .bake
                .as_ref()
                .is_none_or(TaskBakeAdapterInputsSummary::is_empty)
    }
}

#[derive(Debug, Serialize, Clone, Default, PartialEq, Eq)]
pub struct TaskComposeAdapterInputsSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<String>,
}

impl TaskComposeAdapterInputsSummary {
    pub fn is_empty(&self) -> bool {
        self.cwd.is_none()
            && self.env_files.is_empty()
            && self.files.is_empty()
            && self.profiles.is_empty()
            && self.project_name.is_none()
    }
}

#[derive(Debug, Serialize, Clone, Default, PartialEq, Eq)]
pub struct TaskBakeAdapterInputsSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
}

impl TaskBakeAdapterInputsSummary {
    pub fn is_empty(&self) -> bool {
        self.cwd.is_none() && self.files.is_empty()
    }
}

impl TaskEffectsSummary {
    pub fn from_spec(spec: &crate::schema::TaskEffectsSpec) -> Self {
        Self {
            writes: spec.writes.clone(),
            workspace_writes: spec.workspace_writes.clone(),
            network: spec.network,
            network_kind: spec.network_kind,
            adapter_state: spec.adapter_state.clone(),
            external_state: spec.external_state.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
            && self.workspace_writes.is_empty()
            && !self.network
            && self.network_kind.is_none()
            && self.adapter_state.is_empty()
            && self.external_state.is_empty()
    }
}

impl<'a> TaskSummary<'a> {
    pub fn from_spec(
        name: &'a str,
        task: &'a TaskSpec,
        current_os: &str,
        contract: &'a Contract,
    ) -> Self {
        Self::from_spec_with_overrides(
            name,
            task,
            current_os,
            contract,
            ExecutionOverrides::default(),
        )
    }

    pub fn from_spec_with_overrides(
        name: &'a str,
        task: &'a TaskSpec,
        current_os: &str,
        contract: &'a Contract,
        overrides: ExecutionOverrides,
    ) -> Self {
        let task_safety =
            crate::cli::task_effective_safety_with_overrides(contract, name, overrides);
        let effective = effective_task_execution(contract, name, overrides);
        let selected_backend = effective.backend;
        let resolved_execution = task
            .resolved_execution_for_backend(selected_backend, current_os)
            .expect("validated task must resolve to a default or variant execution");
        let effective_env = task.env_for_backend_with_context_name_for_os(
            contract.execution.as_ref(),
            selected_backend,
            effective.context_name,
            current_os,
        );
        let effective_env_files = task.env_files_for_backend_for_os(selected_backend, current_os);
        let effective_adapter_inputs =
            effective_task_adapter_inputs_summary(task, selected_backend, current_os);
        let inputs = task.inputs_for_os(current_os);
        let preview =
            effective_task_execution_preview(contract, name, task, selected_backend, current_os);
        let launch_preview =
            effective_task_launch_preview(contract, name, task, selected_backend, current_os);
        let modes: Vec<TaskModeView<'a>> = task
            .execution
            .as_ref()
            .map(|execution| {
                execution
                    .modes
                    .iter()
                    .map(|(backend, branch)| {
                        let branch_effective = effective_task_execution(
                            contract,
                            name,
                            ExecutionOverrides {
                                backend: Some(backend),
                                lifecycle: None,
                                host_port: None,
                                memory: None,
                                skip_deps: false,
                            },
                        );
                        let branch_execution = branch.execution();
                        TaskModeView {
                            mode: task_mode_name(backend),
                            context: branch_effective.context_name,
                            depends_on: branch
                                .depends_on
                                .clone()
                                .unwrap_or_else(|| task.depends_on.clone()),
                            env_files: branch.env_files.iter().map(String::as_str).collect(),
                            adapter_inputs: summarize_task_adapter_inputs(&branch.adapter_inputs),
                            lifecycle: branch.lifecycle.map(format_lifecycle),
                            kind: branch_execution.map(|execution| execution.kind),
                            run: branch.run.as_deref(),
                            script: branch.script.as_deref(),
                            command: summarize_task_command(branch.command.as_ref()),
                            compose: summarize_task_compose(branch.compose.as_ref()),
                            launch: branch
                                .launch
                                .as_ref()
                                .and_then(|launch| summarize_task_launch(Some(launch))),
                            prepare: summarize_task_prepare(branch.prepare.as_ref()),
                            has_runtime: branch.runtime.is_some(),
                        }
                    })
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default();
        let supports_native_mode_override = task.workflow_backend(contract.execution.as_ref())
            == crate::schema::Backend::Container
            && task
                .mode_execution_branch(crate::schema::Backend::Native)
                .is_none()
            && task.resolved_execution(current_os).is_some();
        let mode_platform_availability = [
            (
                "container",
                contract.task_active_for_backend_on_os(
                    task,
                    crate::schema::Backend::Container,
                    current_os,
                ),
            ),
            (
                "native",
                contract.task_active_for_backend_on_os(
                    task,
                    crate::schema::Backend::Native,
                    current_os,
                ),
            ),
            (
                "remote",
                contract.task_active_for_backend_on_os(
                    task,
                    crate::schema::Backend::Remote,
                    current_os,
                ),
            ),
        ];
        Self {
            name,
            context: effective.context_name,
            default_mode: task.mode_default_backend().map(task_mode_name),
            effective_default_mode: task_mode_name(selected_backend),
            usage: task_lane_use_summary(
                name,
                &inputs,
                task_safety.effective_safe,
                task_mode_name(selected_backend),
                &modes,
                supports_native_mode_override,
                &mode_platform_availability,
            ),
            description: task.description.as_deref(),
            notes: task.notes.as_deref(),
            category: task.category.as_deref(),
            preview,
            launch_preview,
            env: effective_env,
            env_files: effective_env_files,
            adapter_inputs: effective_adapter_inputs,
            inputs,
            kind: resolved_execution.kind,
            run: (resolved_execution.kind == "run")
                .then(|| resolved_execution.shell_body())
                .flatten(),
            script: (resolved_execution.kind == "script")
                .then(|| resolved_execution.shell_body())
                .flatten(),
            command: summarize_task_command(resolved_execution.command()),
            compose: summarize_task_compose(resolved_execution.compose()),
            launch: summarize_task_launch(resolved_execution.launch()),
            action: summarize_task_action(resolved_execution.action()),
            prepare: summarize_task_prepare(resolved_execution.prepare()),
            aggregate: summarize_task_aggregate(resolved_execution.aggregate()),
            effects: TaskEffectsSummary::from_spec(&task.effects),
            selected_variant_os: task
                .selected_variant(current_os)
                .and_then(|variant| variant.when.os.as_deref())
                .or(resolved_execution.os),
            depends_on: task.depends_on_for_backend(selected_backend).to_vec(),
            requires_services: task.requires_services.clone(),
            requires_artifacts: task.requires_artifacts.clone(),
            when_checks: task.when.checks.clone(),
            after_success: task.after_success.clone(),
            after_failure: task.after_failure.clone(),
            after_always: task.after_always.clone(),
            safe_for_agent: task_safety.declared_safe,
            effective_safe_for_agent: task_safety.effective_safe,
            unsafe_closure_tasks: task_safety.unsafe_closure_tasks,
            internal: task.internal,
            variants: task
                .variants
                .iter()
                .map(|variant| TaskVariantView {
                    os: variant
                        .when
                        .os
                        .as_deref()
                        .expect("validated task variant must declare `when.os`"),
                    kind: variant
                        .execution_kind()
                        .or_else(|| task.any_execution_kind())
                        .expect("validated task variant must resolve to one execution form"),
                    env: &variant.env,
                    env_files: variant.env_files.iter().map(String::as_str).collect(),
                    inputs: &variant.inputs,
                    run: variant.run.as_deref(),
                    script: variant.script.as_deref(),
                    command: variant
                        .command
                        .as_ref()
                        .and_then(|command| summarize_task_command(Some(command))),
                    compose: summarize_task_compose(variant.compose.as_ref()),
                    adapter_inputs: summarize_task_adapter_inputs(&variant.adapter_inputs),
                })
                .collect(),
            modes,
            supports_native_mode_override,
        }
    }

    pub fn retain_visible_task_relationships(&mut self, visible_task_names: &BTreeSet<String>) {
        if let Some(aggregate) = self.aggregate.as_mut() {
            aggregate
                .tasks
                .retain(|task| visible_task_names.contains(task.as_str()));
        }
        self.depends_on
            .retain(|task| visible_task_names.contains(task.as_str()));
        self.after_success
            .retain(|task| visible_task_names.contains(task.as_str()));
        self.after_failure
            .retain(|task| visible_task_names.contains(task.as_str()));
        self.after_always
            .retain(|task| visible_task_names.contains(task.as_str()));
    }
}

fn append_task_input_placeholders(command: &mut String, inputs: &BTreeMap<String, TaskInputSpec>) {
    for (name, spec) in inputs {
        command.push(' ');
        command.push_str(&format!("--{}", name.replace('_', "-")));
        command.push(' ');
        command.push_str(&if spec.allowed.is_empty() {
            String::from("<value>")
        } else {
            format!("<{}>", spec.allowed.join("|"))
        });
    }
}

fn task_lane_use_summary(
    task_name: &str,
    inputs: &BTreeMap<String, TaskInputSpec>,
    effective_safe_for_agent: bool,
    default_mode: &str,
    modes: &[TaskModeView<'_>],
    supports_native_mode_override: bool,
    mode_platform_availability: &[(&str, bool)],
) -> LaneUseSummary {
    let mut human = format!("ota run {task_name}");
    append_task_input_placeholders(&mut human, inputs);

    let default_platform_available = mode_platform_availability
        .iter()
        .find(|(mode, _)| *mode == default_mode)
        .map(|(_, available)| *available)
        .unwrap_or(true);
    let agent = if !default_platform_available {
        AgentLaneUseSummary {
            callable: false,
            command: None,
            reason: Some(String::from("unsupported_host_platform")),
        }
    } else if effective_safe_for_agent {
        let mut command = format!("ota run {task_name} --agent");
        append_task_input_placeholders(&mut command, inputs);
        AgentLaneUseSummary {
            callable: true,
            command: Some(command),
            reason: None,
        }
    } else {
        AgentLaneUseSummary {
            callable: false,
            command: None,
            reason: Some(String::from("not_safe")),
        }
    };

    let mut displayed_modes = vec!["container", "native"];
    if default_mode == "remote" || modes.iter().any(|mode| mode.mode == "remote") {
        displayed_modes.push("remote");
    }

    let mode_summary = |mode: &str| {
        let platform_available = mode_platform_availability
            .iter()
            .find(|(candidate, _)| *candidate == mode)
            .map(|(_, available)| *available)
            .unwrap_or(true);
        let supported = platform_available
            && (mode == default_mode
                || modes.iter().any(|entry| entry.mode == mode)
                || (mode == "native" && supports_native_mode_override));
        let mut mode_human = format!("ota run {task_name}");
        let mut mode_agent = format!("ota run {task_name}");
        if mode != default_mode {
            let flag = match mode {
                "native" => "--native",
                "container" => "--container",
                "remote" => "--remote",
                _ => "--mode",
            };
            mode_human.push(' ');
            mode_human.push_str(flag);
            mode_agent.push(' ');
            mode_agent.push_str(flag);
        }
        mode_agent.push_str(" --agent");
        append_task_input_placeholders(&mut mode_human, inputs);
        append_task_input_placeholders(&mut mode_agent, inputs);

        let unavailable = || AgentLaneUseSummary {
            callable: false,
            command: None,
            reason: Some(String::from(if platform_available {
                "not_supported_by_task"
            } else {
                "unsupported_host_platform"
            })),
        };
        LaneUseModeSummary {
            mode: mode.to_string(),
            default: mode == default_mode,
            availability: if supported {
                LaneUseModeAvailability::Supported
            } else {
                LaneUseModeAvailability::Unavailable
            },
            reason: (!supported).then(|| {
                String::from(if platform_available {
                    "not_supported_by_task"
                } else {
                    "unsupported_host_platform"
                })
            }),
            human: if supported {
                AgentLaneUseSummary {
                    callable: true,
                    command: Some(mode_human),
                    reason: None,
                }
            } else {
                unavailable()
            },
            agent: if supported && effective_safe_for_agent {
                AgentLaneUseSummary {
                    callable: true,
                    command: Some(mode_agent),
                    reason: None,
                }
            } else if supported {
                AgentLaneUseSummary {
                    callable: false,
                    command: None,
                    reason: Some(String::from("not_safe")),
                }
            } else {
                unavailable()
            },
        }
    };

    LaneUseSummary {
        human,
        agent,
        modes: displayed_modes.into_iter().map(mode_summary).collect(),
    }
}

fn workflow_lane_use_summary(
    workflow_selector: &str,
    effective_safe_for_agent: Option<bool>,
) -> LaneUseSummary {
    LaneUseSummary {
        human: format!("ota up --workflow {workflow_selector}"),
        agent: match effective_safe_for_agent {
            Some(true) => AgentLaneUseSummary {
                callable: true,
                command: Some(format!("ota up --workflow {workflow_selector} --agent")),
                reason: None,
            },
            Some(false) => AgentLaneUseSummary {
                callable: false,
                command: None,
                reason: Some(String::from("not_safe")),
            },
            None => AgentLaneUseSummary {
                callable: false,
                command: None,
                reason: Some(String::from("unknown")),
            },
        },
        modes: Vec::new(),
    }
}

fn effective_task_execution_preview(
    contract: &Contract,
    task_name: &str,
    task: &TaskSpec,
    backend: Backend,
    current_os: &str,
) -> String {
    orchestrator_execution_preview(contract, task_name, task, backend, current_os)
        .or_else(|| {
            task.resolved_execution_for_backend(backend, current_os)
                .map(|execution| execution.preview())
        })
        .unwrap_or_else(|| String::from("-"))
}

fn effective_task_launch_preview(
    contract: &Contract,
    task_name: &str,
    task: &TaskSpec,
    backend: Backend,
    current_os: &str,
) -> Option<String> {
    let execution = task.resolved_execution_for_backend(backend, current_os)?;
    execution.launch()?;
    orchestrator_execution_preview(contract, task_name, task, backend, current_os).or_else(|| {
        execution
            .launch()
            .map(crate::schema::TaskLaunchSpec::preview)
    })
}

pub fn summarize_task_aggregate(
    aggregate: Option<&crate::schema::TaskAggregateSpec>,
) -> Option<TaskAggregateSummary> {
    aggregate.map(|aggregate| TaskAggregateSummary {
        tasks: aggregate.tasks.clone(),
    })
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskModeView<'a> {
    pub mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<&'a str>,
    #[serde(skip_serializing_if = "TaskAdapterInputsSummary::is_empty")]
    pub adapter_inputs: TaskAdapterInputsSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<TaskCommandSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose: Option<TaskComposeExecutionSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub launch: Option<TaskLaunchSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub prepare: Option<TaskPrepareSummary<'a>>,
    pub has_runtime: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskCommandSummary<'a> {
    pub exe: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<&'a str>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskComposeExecutionSummary<'a> {
    pub kind: &'static str,
    pub engine: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exe: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<&'a str>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rm: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub build: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub service_ports: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub detach: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub force_recreate: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub force: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub follow: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub remove_volumes: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tty: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskComposeInvocationSummary<'a> {
    pub kind: &'static str,
    pub engine: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<&'a str>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rm: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub build: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub service_ports: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub detach: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub force_recreate: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub force: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub follow: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub remove_volumes: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tty: bool,
}

pub fn summarize_task_adapter_inputs(
    adapter_inputs: &crate::schema::TaskAdapterInputsSpec,
) -> TaskAdapterInputsSummary {
    TaskAdapterInputsSummary {
        compose: adapter_inputs
            .effective_compose()
            .map(|compose| TaskComposeAdapterInputsSummary {
                cwd: compose
                    .cwd
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                env_files: compose.env_files,
                files: compose.files,
                profiles: compose.profiles,
                project_name: compose
                    .project_name
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
            })
            .filter(|compose| !compose.is_empty()),
        bake: adapter_inputs
            .effective_bake()
            .map(|bake| TaskBakeAdapterInputsSummary {
                cwd: bake
                    .cwd
                    .map(|value| value.trim().to_string())
                    .filter(|value| !value.is_empty()),
                files: bake.files,
            })
            .filter(|bake| !bake.is_empty()),
    }
}

fn effective_task_adapter_inputs_summary(
    task: &TaskSpec,
    backend: crate::schema::Backend,
    current_os: &str,
) -> TaskAdapterInputsSummary {
    let compose = {
        let cwd = task.compose_adapter_cwd_for_backend_for_os(backend, current_os);
        let env_files = task.compose_adapter_env_files_for_backend_for_os(backend, current_os);
        let files = task.compose_adapter_files_for_backend_for_os(backend, current_os);
        let profiles = task.compose_adapter_profiles_for_backend_for_os(backend, current_os);
        let project_name =
            task.compose_adapter_project_name_for_backend_for_os(backend, current_os);
        let summary = TaskComposeAdapterInputsSummary {
            cwd,
            env_files,
            files,
            profiles,
            project_name,
        };
        (!summary.is_empty()).then_some(summary)
    };

    let bake = {
        let summary = TaskBakeAdapterInputsSummary {
            cwd: task.bake_adapter_cwd_for_backend_for_os(backend, current_os),
            files: task.bake_adapter_files_for_backend_for_os(backend, current_os),
        };
        (!summary.is_empty()).then_some(summary)
    };

    TaskAdapterInputsSummary { compose, bake }
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskLaunchSummary<'a> {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exe: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub image: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub engine: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub action: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<&'a str>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub detach: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<&'a str>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub remove: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<TaskLaunchVolumeSummary<'a>>,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskPrepareSummary<'a> {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<TaskPrepareSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_mode: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<&'a str>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub frozen_lockfile: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub inline_builds: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub force: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub no_root: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub skip_tests: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub with_deps: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub browsers: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_hydration_provenance: Option<TaskHydrationProvenanceSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_hydration_provenance: Option<TaskHydrationProvenanceSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose: Option<TaskComposeInvocationSummary<'a>>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct WorkspaceTaskPrepareSummary {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<WorkspaceTaskPrepareSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub medium: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_kind: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub filter: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mode: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub group_mode: Option<&'static str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub groups: Vec<String>,
    #[serde(default, skip_serializing_if = "is_false")]
    pub frozen_lockfile: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub inline_builds: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub force: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub no_root: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub skip_tests: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub with_deps: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub browsers: Vec<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub declared_hydration_provenance: Option<WorkspaceTaskHydrationProvenanceSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_hydration_provenance: Option<WorkspaceTaskHydrationProvenanceSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose: Option<WorkspaceTaskComposeInvocationSummary>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TaskHydrationProvenanceSummary<'a> {
    pub source_posture: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<&'a str>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct WorkspaceTaskHydrationProvenanceSummary {
    pub source_posture: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub config_file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub source_identities: Vec<WorkspaceTaskHydrationSourceIdentity>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution_error: Option<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct WorkspaceTaskHydrationSourceIdentity {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    pub url: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct WorkspaceTaskComposeInvocationSummary {
    pub kind: &'static str,
    pub engine: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub services: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workdir: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub rm: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub build: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub service_ports: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub detach: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub force_recreate: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub force: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub follow: bool,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub remove_volumes: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout_seconds: Option<u32>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub tty: bool,
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskLaunchVolumeSummary<'a> {
    pub kind: &'static str,
    pub source: &'a str,
    pub target: &'a str,
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Serialize, Clone)]
pub struct TaskActionSummary<'a> {
    pub kind: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub to: Option<&'a str>,
}

fn summarize_task_command<'a>(
    command: Option<&'a crate::schema::TaskCommandSpec>,
) -> Option<TaskCommandSummary<'a>> {
    command.map(|command| TaskCommandSummary {
        exe: command.exe.as_str(),
        args: command.args.iter().map(String::as_str).collect(),
        cwd: command.cwd.as_deref(),
    })
}

fn summarize_task_compose<'a>(
    compose: Option<&'a crate::schema::TaskComposeExecutionSpec>,
) -> Option<TaskComposeExecutionSummary<'a>> {
    compose.map(|compose| TaskComposeExecutionSummary {
        kind: compose.invocation.kind.label(),
        engine: compose.invocation.engine.as_str(),
        service: (!compose.invocation.service.trim().is_empty())
            .then_some(compose.invocation.service.as_str()),
        services: compose
            .invocation
            .services
            .iter()
            .map(String::as_str)
            .collect(),
        exe: (!compose.exe.trim().is_empty()).then_some(compose.exe.as_str()),
        args: compose.args.iter().map(String::as_str).collect(),
        workdir: compose.invocation.workdir.as_deref(),
        rm: compose.invocation.rm,
        build: compose.invocation.build,
        service_ports: compose.invocation.service_ports,
        detach: compose.invocation.detach,
        force_recreate: compose.invocation.force_recreate,
        force: compose.invocation.force,
        follow: compose.invocation.follow,
        remove_volumes: compose.invocation.remove_volumes,
        timeout_seconds: compose.invocation.timeout_seconds,
        tty: compose.invocation.tty,
    })
}

fn summarize_task_compose_invocation<'a>(
    compose: Option<&'a crate::schema::TaskComposeInvocationSpec>,
) -> Option<TaskComposeInvocationSummary<'a>> {
    compose.map(|compose| TaskComposeInvocationSummary {
        kind: compose.kind.label(),
        engine: compose.engine.as_str(),
        service: (!compose.service.trim().is_empty()).then_some(compose.service.as_str()),
        services: compose.services.iter().map(String::as_str).collect(),
        workdir: compose.workdir.as_deref(),
        rm: compose.rm,
        build: compose.build,
        service_ports: compose.service_ports,
        detach: compose.detach,
        force_recreate: compose.force_recreate,
        force: compose.force,
        follow: compose.follow,
        remove_volumes: compose.remove_volumes,
        timeout_seconds: compose.timeout_seconds,
        tty: compose.tty,
    })
}

fn summarize_task_compose_invocation_owned(
    compose: Option<&crate::schema::TaskComposeInvocationSpec>,
) -> Option<WorkspaceTaskComposeInvocationSummary> {
    compose.map(|compose| WorkspaceTaskComposeInvocationSummary {
        kind: compose.kind.label(),
        engine: compose.engine.as_str(),
        service: (!compose.service.trim().is_empty()).then_some(compose.service.clone()),
        services: compose.services.clone(),
        workdir: compose.workdir.clone(),
        rm: compose.rm,
        build: compose.build,
        service_ports: compose.service_ports,
        detach: compose.detach,
        force_recreate: compose.force_recreate,
        force: compose.force,
        follow: compose.follow,
        remove_volumes: compose.remove_volumes,
        timeout_seconds: compose.timeout_seconds,
        tty: compose.tty,
    })
}

fn summarize_task_launch<'a>(
    launch: Option<&'a crate::schema::TaskLaunchSpec>,
) -> Option<TaskLaunchSummary<'a>> {
    match launch? {
        crate::schema::TaskLaunchSpec::Command(command) => Some(TaskLaunchSummary {
            kind: "command",
            exe: Some(command.exe.as_str()),
            args: command.args.iter().map(String::as_str).collect(),
            image: None,
            engine: None,
            action: None,
            services: Vec::new(),
            detach: false,
            name: None,
            remove: false,
            volumes: Vec::new(),
        }),
        crate::schema::TaskLaunchSpec::Compose(compose) => Some(TaskLaunchSummary {
            kind: "compose",
            exe: None,
            args: Vec::new(),
            image: None,
            engine: Some(compose.engine.as_str()),
            action: Some(compose.action.label()),
            services: compose.services.iter().map(String::as_str).collect(),
            detach: compose.detach,
            name: None,
            remove: false,
            volumes: Vec::new(),
        }),
        crate::schema::TaskLaunchSpec::Container(container) => Some(TaskLaunchSummary {
            kind: "container",
            exe: None,
            args: container.args.iter().map(String::as_str).collect(),
            image: Some(container.image.as_str()),
            engine: container.engine.as_deref(),
            action: None,
            services: Vec::new(),
            detach: false,
            name: container.name.as_deref(),
            remove: container.remove,
            volumes: container
                .volumes
                .iter()
                .map(|volume| TaskLaunchVolumeSummary {
                    kind: match volume.kind {
                        crate::schema::TaskContainerLaunchVolumeKind::Named => "named",
                    },
                    source: volume.source.as_str(),
                    target: volume.target.as_str(),
                })
                .collect(),
        }),
    }
}

fn summarize_task_action<'a>(
    action: Option<&'a crate::schema::TaskActionSpec>,
) -> Option<TaskActionSummary<'a>> {
    match action? {
        crate::schema::TaskActionSpec::CopyIfMissing(copy) => Some(TaskActionSummary {
            kind: "copy_if_missing",
            from: Some(copy.from.as_str()),
            to: Some(copy.to.as_str()),
        }),
        crate::schema::TaskActionSpec::EnsureEnvFile(spec) => Some(TaskActionSummary {
            kind: "ensure_env_file",
            from: spec.template.as_deref(),
            to: Some(spec.path.as_str()),
        }),
        crate::schema::TaskActionSpec::EnsureFile(spec) => Some(TaskActionSummary {
            kind: "ensure_file",
            from: spec.template.as_deref(),
            to: Some(spec.path.as_str()),
        }),
        crate::schema::TaskActionSpec::EnsureDirectory(spec) => Some(TaskActionSummary {
            kind: "ensure_directory",
            from: None,
            to: Some(spec.path.as_str()),
        }),
        crate::schema::TaskActionSpec::EnsureVirtualenv(spec) => Some(TaskActionSummary {
            kind: "ensure_virtualenv",
            from: Some(spec.provider.label()),
            to: Some(spec.path.as_str()),
        }),
        crate::schema::TaskActionSpec::EnsureGitCheckout(spec) => Some(TaskActionSummary {
            kind: "ensure_git_checkout",
            from: Some(spec.source.git.as_str()),
            to: Some(spec.path.as_str()),
        }),
        crate::schema::TaskActionSpec::EnsureGitTemplate(spec) => Some(TaskActionSummary {
            kind: "ensure_git_template",
            from: Some(spec.source.git.as_str()),
            to: Some(spec.path.as_str()),
        }),
        crate::schema::TaskActionSpec::EnsureGitCheckouts(_) => Some(TaskActionSummary {
            kind: "ensure_git_checkouts",
            from: None,
            to: None,
        }),
        crate::schema::TaskActionSpec::EnsureContainerNetwork(spec) => Some(TaskActionSummary {
            kind: "ensure_container_network",
            from: Some(spec.provider.label()),
            to: Some(spec.name.as_str()),
        }),
        crate::schema::TaskActionSpec::ResetComposeServiceVolume(spec) => Some(TaskActionSummary {
            kind: "reset_compose_service_volume",
            from: Some(spec.service.as_str()),
            to: Some(spec.volume.as_str()),
        }),
        crate::schema::TaskActionSpec::EnsureBundle(_) => Some(TaskActionSummary {
            kind: "ensure_bundle",
            from: None,
            to: None,
        }),
    }
}

pub fn summarize_task_launch_owned(
    launch: Option<&crate::schema::TaskLaunchSpec>,
) -> Option<WorkspaceTaskLaunchSummary> {
    match launch? {
        crate::schema::TaskLaunchSpec::Command(command) => Some(WorkspaceTaskLaunchSummary {
            kind: "command",
            exe: Some(command.exe.clone()),
            args: command.args.clone(),
            image: None,
            engine: None,
            action: None,
            services: Vec::new(),
            detach: false,
            name: None,
            remove: false,
            volumes: Vec::new(),
        }),
        crate::schema::TaskLaunchSpec::Compose(compose) => Some(WorkspaceTaskLaunchSummary {
            kind: "compose",
            exe: None,
            args: Vec::new(),
            image: None,
            engine: Some(compose.engine.as_str().to_string()),
            action: Some(compose.action.label()),
            services: compose.services.clone(),
            detach: compose.detach,
            name: None,
            remove: false,
            volumes: Vec::new(),
        }),
        crate::schema::TaskLaunchSpec::Container(container) => Some(WorkspaceTaskLaunchSummary {
            kind: "container",
            exe: None,
            args: container.args.clone(),
            image: Some(container.image.clone()),
            engine: container.engine.clone(),
            action: None,
            services: Vec::new(),
            detach: false,
            name: container.name.clone(),
            remove: container.remove,
            volumes: container
                .volumes
                .iter()
                .map(|volume| WorkspaceTaskLaunchVolumeSummary {
                    kind: match volume.kind {
                        crate::schema::TaskContainerLaunchVolumeKind::Named => "named",
                    },
                    source: volume.source.clone(),
                    target: volume.target.clone(),
                })
                .collect(),
        }),
    }
}

pub fn summarize_task_action_owned(
    action: Option<&crate::schema::TaskActionSpec>,
) -> Option<WorkspaceTaskActionSummary> {
    match action? {
        crate::schema::TaskActionSpec::CopyIfMissing(copy) => Some(WorkspaceTaskActionSummary {
            kind: "copy_if_missing",
            from: Some(copy.from.clone()),
            to: Some(copy.to.clone()),
        }),
        crate::schema::TaskActionSpec::EnsureEnvFile(spec) => Some(WorkspaceTaskActionSummary {
            kind: "ensure_env_file",
            from: spec.template.clone(),
            to: Some(spec.path.clone()),
        }),
        crate::schema::TaskActionSpec::EnsureFile(spec) => Some(WorkspaceTaskActionSummary {
            kind: "ensure_file",
            from: spec.template.clone(),
            to: Some(spec.path.clone()),
        }),
        crate::schema::TaskActionSpec::EnsureDirectory(spec) => Some(WorkspaceTaskActionSummary {
            kind: "ensure_directory",
            from: None,
            to: Some(spec.path.clone()),
        }),
        crate::schema::TaskActionSpec::EnsureVirtualenv(spec) => Some(WorkspaceTaskActionSummary {
            kind: "ensure_virtualenv",
            from: Some(spec.provider.label().to_string()),
            to: Some(spec.path.clone()),
        }),
        crate::schema::TaskActionSpec::EnsureGitCheckout(spec) => {
            Some(WorkspaceTaskActionSummary {
                kind: "ensure_git_checkout",
                from: Some(spec.source.git.clone()),
                to: Some(spec.path.clone()),
            })
        }
        crate::schema::TaskActionSpec::EnsureGitTemplate(spec) => {
            Some(WorkspaceTaskActionSummary {
                kind: "ensure_git_template",
                from: Some(spec.source.git.clone()),
                to: Some(spec.path.clone()),
            })
        }
        crate::schema::TaskActionSpec::EnsureGitCheckouts(_) => Some(WorkspaceTaskActionSummary {
            kind: "ensure_git_checkouts",
            from: None,
            to: None,
        }),
        crate::schema::TaskActionSpec::EnsureContainerNetwork(spec) => {
            Some(WorkspaceTaskActionSummary {
                kind: "ensure_container_network",
                from: Some(spec.provider.label().to_string()),
                to: Some(spec.name.clone()),
            })
        }
        crate::schema::TaskActionSpec::ResetComposeServiceVolume(spec) => {
            Some(WorkspaceTaskActionSummary {
                kind: "reset_compose_service_volume",
                from: Some(spec.service.clone()),
                to: Some(spec.volume.clone()),
            })
        }
        crate::schema::TaskActionSpec::EnsureBundle(_) => Some(WorkspaceTaskActionSummary {
            kind: "ensure_bundle",
            from: None,
            to: None,
        }),
    }
}

pub fn summarize_task_prepare(
    prepare: Option<&crate::schema::TaskPrepareSpec>,
) -> Option<TaskPrepareSummary<'_>> {
    match prepare? {
        crate::schema::TaskPrepareSpec::Sequence(spec) => Some(TaskPrepareSummary {
            kind: "sequence",
            steps: spec
                .steps
                .iter()
                .filter_map(summarize_task_prepare_sequence_step)
                .collect(),
            medium: None,
            source_kind: None,
            cwd: None,
            file: None,
            files: Vec::new(),
            env_files: Vec::new(),
            manager: None,
            filter: None,
            mode: None,
            group_mode: None,
            groups: Vec::new(),
            frozen_lockfile: false,
            inline_builds: false,
            force: false,
            no_root: false,
            skip_tests: false,
            with_deps: false,
            targets: Vec::new(),
            browsers: Vec::new(),
            declared_hydration_provenance: None,
            resolved_hydration_provenance: None,
            compose: None,
        }),
        crate::schema::TaskPrepareSpec::ToolBootstrap(spec) => {
            Some(summarize_tool_bootstrap_prepare_spec(spec))
        }
        crate::schema::TaskPrepareSpec::DependencyHydration(spec) => {
            Some(summarize_dependency_hydration_prepare_spec(spec))
        }
    }
}

fn summarize_task_prepare_sequence_step(
    step: &crate::schema::TaskPrepareSequenceStepSpec,
) -> Option<TaskPrepareSummary<'_>> {
    match step {
        crate::schema::TaskPrepareSequenceStepSpec::DependencyHydration(spec) => {
            Some(summarize_dependency_hydration_prepare_spec(spec))
        }
        crate::schema::TaskPrepareSequenceStepSpec::ToolBootstrap(spec) => {
            Some(summarize_tool_bootstrap_prepare_spec(spec))
        }
        crate::schema::TaskPrepareSequenceStepSpec::Sequence(spec) => Some(TaskPrepareSummary {
            kind: "sequence",
            steps: spec
                .steps
                .iter()
                .filter_map(summarize_task_prepare_sequence_step)
                .collect(),
            ..empty_task_prepare_summary("sequence")
        }),
        crate::schema::TaskPrepareSequenceStepSpec::CopyIfMissing(_) => {
            Some(empty_task_prepare_summary("copy_if_missing"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureEnvFile(_) => {
            Some(empty_task_prepare_summary("ensure_env_file"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureFile(_) => {
            Some(empty_task_prepare_summary("ensure_file"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureDirectory(_) => {
            Some(empty_task_prepare_summary("ensure_directory"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureVirtualenv(_) => {
            Some(empty_task_prepare_summary("ensure_virtualenv"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureGitCheckout(_) => {
            Some(empty_task_prepare_summary("ensure_git_checkout"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureGitTemplate(_) => {
            Some(empty_task_prepare_summary("ensure_git_template"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureContainerNetwork(_) => {
            Some(empty_task_prepare_summary("ensure_container_network"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::ResetComposeServiceVolume(_) => {
            Some(empty_task_prepare_summary("reset_compose_service_volume"))
        }
    }
}

fn summarize_tool_bootstrap_prepare_spec(
    spec: &crate::schema::TaskToolBootstrapPrepareSpec,
) -> TaskPrepareSummary<'_> {
    let (source_kind, mode, cwd, manager, filter) = match &spec.source {
        crate::schema::TaskToolBootstrapSourceSpec::Pip(_source) => {
            ("pip", Some(spec.tool.label()), None, None, None)
        }
        crate::schema::TaskToolBootstrapSourceSpec::Poetry(source) => (
            "poetry",
            Some(spec.tool.label()),
            Some(source.cwd.trim()),
            None,
            None,
        ),
        crate::schema::TaskToolBootstrapSourceSpec::NodePackageManager(source) => (
            "node_package_manager",
            Some(spec.tool.label()),
            Some(source.cwd.trim()),
            Some(source.manager.label()),
            source.filter.as_deref().map(str::trim),
        ),
    };
    TaskPrepareSummary {
        kind: "tool_bootstrap",
        steps: Vec::new(),
        medium: None,
        source_kind: Some(source_kind),
        cwd,
        file: None,
        files: Vec::new(),
        env_files: Vec::new(),
        manager,
        filter,
        mode,
        group_mode: None,
        groups: Vec::new(),
        frozen_lockfile: false,
        inline_builds: false,
        force: false,
        no_root: false,
        skip_tests: false,
        with_deps: spec.with_deps,
        targets: Vec::new(),
        browsers: spec
            .browsers
            .iter()
            .map(|browser| browser.label())
            .collect(),
        declared_hydration_provenance: None,
        resolved_hydration_provenance: None,
        compose: None,
    }
}

fn summarize_dependency_hydration_prepare_spec(
    spec: &crate::schema::TaskDependencyHydrationPrepareSpec,
) -> TaskPrepareSummary<'_> {
    let mut declared_hydration_provenance = None;
    let resolved_hydration_provenance = None;
    let (
        source_kind,
        cwd,
        file,
        files,
        env_files,
        manager,
        mode,
        group_mode,
        groups,
        frozen_lockfile,
        inline_builds,
        force,
        no_root,
        skip_tests,
        compose,
    ) = match &spec.source {
        crate::schema::TaskDependencyHydrationSourceSpec::DockerCompose(source) => (
            "docker_compose",
            Some(source.cwd.as_str()),
            source.file.as_deref(),
            source.files.iter().map(String::as_str).collect(),
            source.env_files.iter().map(String::as_str).collect(),
            None,
            None,
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            None,
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::NodePackageManager(source) => (
            "node_package_manager",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some(source.manager.label()),
            Some(match source.mode {
                crate::schema::TaskNodePackageManagerHydrationMode::Install => "install",
                crate::schema::TaskNodePackageManagerHydrationMode::Ci => "ci",
            }),
            None,
            Vec::new(),
            source.frozen_lockfile,
            source.inline_builds,
            source.force,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::Bundler(source) => (
            "bundler",
            Some(source.cwd.as_str()),
            source.path.as_deref(),
            Vec::new(),
            Vec::new(),
            Some("bundle"),
            Some("install"),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::Composer(source) => (
            "composer",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some("composer"),
            Some("install"),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::Uv(source) => (
            "uv",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some("uv"),
            Some("sync"),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::Poetry(source) => (
            "poetry",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some("poetry"),
            Some("install"),
            Some(match source.group_mode {
                crate::schema::TaskPoetryHydrationGroupMode::With => "with",
                crate::schema::TaskPoetryHydrationGroupMode::Only => "only",
            }),
            source.groups.iter().map(String::as_str).collect(),
            false,
            false,
            false,
            source.no_root,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::GoModules(source) => (
            "go_modules",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            None,
            Some("download"),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::Helm(source) => (
            "helm",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some("helm"),
            Some("dependency_build"),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::Maven(source) => (
            "maven",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some(if source.wrapper { "./mvnw" } else { "mvn" }),
            Some(source.mode.goal()),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            source.skip_tests,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::Gradle(source) => (
            "gradle",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some(if source.wrapper {
                "./gradlew"
            } else {
                "gradle"
            }),
            Some("dependencies"),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::Cargo(source) => (
            "cargo",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some("cargo"),
            Some("fetch"),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
        crate::schema::TaskDependencyHydrationSourceSpec::DotnetRestore(source) => (
            "dotnet_restore",
            Some(source.cwd.as_str()),
            None,
            Vec::new(),
            Vec::new(),
            Some("dotnet"),
            Some("restore"),
            None,
            Vec::new(),
            false,
            false,
            false,
            false,
            false,
            summarize_task_compose_invocation(source.compose.as_ref()),
        ),
    };
    if let crate::schema::TaskDependencyHydrationSourceSpec::DotnetRestore(source) = &spec.source {
        let provenance = TaskHydrationProvenanceSummary {
            source_posture: source.source_posture(),
            config_file: source.config_file.as_deref(),
            sources: source.sources.iter().map(String::as_str).collect(),
        };
        declared_hydration_provenance = Some(provenance);
    }
    TaskPrepareSummary {
        kind: "dependency_hydration",
        steps: Vec::new(),
        medium: Some(match spec.medium {
            crate::schema::TaskDependencyHydrationMedium::ContainerImages => "container_images",
            crate::schema::TaskDependencyHydrationMedium::PackageDependencies => {
                "package_dependencies"
            }
        }),
        source_kind: Some(source_kind),
        cwd,
        file,
        files,
        env_files,
        manager,
        filter: match &spec.source {
            crate::schema::TaskDependencyHydrationSourceSpec::NodePackageManager(source) => source
                .filter
                .as_deref()
                .map(str::trim)
                .filter(|filter| !filter.is_empty()),
            _ => None,
        },
        mode,
        group_mode,
        groups,
        frozen_lockfile,
        inline_builds,
        force,
        no_root,
        skip_tests,
        with_deps: false,
        targets: spec.targets.iter().map(String::as_str).collect(),
        browsers: Vec::new(),
        declared_hydration_provenance,
        resolved_hydration_provenance,
        compose,
    }
}

fn empty_task_prepare_summary(kind: &'static str) -> TaskPrepareSummary<'static> {
    TaskPrepareSummary {
        kind,
        steps: Vec::new(),
        medium: None,
        source_kind: None,
        cwd: None,
        file: None,
        files: Vec::new(),
        env_files: Vec::new(),
        manager: None,
        filter: None,
        mode: None,
        group_mode: None,
        groups: Vec::new(),
        frozen_lockfile: false,
        inline_builds: false,
        force: false,
        no_root: false,
        skip_tests: false,
        with_deps: false,
        targets: Vec::new(),
        browsers: Vec::new(),
        declared_hydration_provenance: None,
        resolved_hydration_provenance: None,
        compose: None,
    }
}

pub fn summarize_task_prepare_owned(
    prepare: Option<&crate::schema::TaskPrepareSpec>,
) -> Option<WorkspaceTaskPrepareSummary> {
    match prepare? {
        crate::schema::TaskPrepareSpec::Sequence(spec) => Some(WorkspaceTaskPrepareSummary {
            kind: "sequence",
            steps: spec
                .steps
                .iter()
                .filter_map(summarize_task_prepare_sequence_step_owned)
                .collect(),
            medium: None,
            source_kind: None,
            cwd: None,
            file: None,
            files: Vec::new(),
            env_files: Vec::new(),
            manager: None,
            filter: None,
            mode: None,
            group_mode: None,
            groups: Vec::new(),
            frozen_lockfile: false,
            inline_builds: false,
            force: false,
            no_root: false,
            skip_tests: false,
            with_deps: false,
            targets: Vec::new(),
            browsers: Vec::new(),
            declared_hydration_provenance: None,
            resolved_hydration_provenance: None,
            compose: None,
        }),
        crate::schema::TaskPrepareSpec::ToolBootstrap(spec) => {
            let (source_kind, mode, cwd, manager, filter) = match &spec.source {
                crate::schema::TaskToolBootstrapSourceSpec::Pip(_source) => {
                    ("pip", Some(spec.tool.label()), None, None, None)
                }
                crate::schema::TaskToolBootstrapSourceSpec::Poetry(source) => (
                    "poetry",
                    Some(spec.tool.label()),
                    Some(source.cwd.trim().to_string()),
                    None,
                    None,
                ),
                crate::schema::TaskToolBootstrapSourceSpec::NodePackageManager(source) => (
                    "node_package_manager",
                    Some(spec.tool.label()),
                    Some(source.cwd.trim().to_string()),
                    Some(source.manager.label()),
                    source.filter.as_deref().map(str::trim).map(str::to_string),
                ),
            };
            Some(WorkspaceTaskPrepareSummary {
                kind: "tool_bootstrap",
                steps: Vec::new(),
                medium: None,
                source_kind: Some(source_kind),
                cwd,
                file: None,
                files: Vec::new(),
                env_files: Vec::new(),
                manager,
                filter,
                mode,
                group_mode: None,
                groups: Vec::new(),
                frozen_lockfile: false,
                inline_builds: false,
                force: false,
                no_root: false,
                skip_tests: false,
                with_deps: spec.with_deps,
                targets: Vec::new(),
                browsers: spec
                    .browsers
                    .iter()
                    .map(|browser| browser.label())
                    .collect(),
                declared_hydration_provenance: None,
                resolved_hydration_provenance: None,
                compose: None,
            })
        }
        crate::schema::TaskPrepareSpec::DependencyHydration(spec) => {
            let (
                source_kind,
                cwd,
                file,
                files,
                env_files,
                manager,
                mode,
                group_mode,
                groups,
                frozen_lockfile,
                inline_builds,
                force,
                no_root,
                skip_tests,
                compose,
            ) = match &spec.source {
                crate::schema::TaskDependencyHydrationSourceSpec::DockerCompose(source) => (
                    "docker_compose",
                    Some(source.cwd.clone()),
                    source.file.clone(),
                    source.files.clone(),
                    source.env_files.clone(),
                    None,
                    None,
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    None,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::NodePackageManager(source) => (
                    "node_package_manager",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some(match source.manager {
                        crate::schema::TaskNodePackageManagerKind::Npm => "npm",
                        crate::schema::TaskNodePackageManagerKind::Pnpm => "pnpm",
                        crate::schema::TaskNodePackageManagerKind::Yarn => "yarn",
                        crate::schema::TaskNodePackageManagerKind::Bun => "bun",
                    }),
                    Some(match source.mode {
                        crate::schema::TaskNodePackageManagerHydrationMode::Install => "install",
                        crate::schema::TaskNodePackageManagerHydrationMode::Ci => "ci",
                    }),
                    None,
                    Vec::new(),
                    source.frozen_lockfile,
                    source.inline_builds,
                    source.force,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Bundler(source) => (
                    "bundler",
                    Some(source.cwd.clone()),
                    source.path.clone(),
                    Vec::new(),
                    Vec::new(),
                    Some("bundle"),
                    Some("install"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Composer(source) => (
                    "composer",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some("composer"),
                    Some("install"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Uv(source) => (
                    "uv",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some("uv"),
                    Some("sync"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Poetry(source) => (
                    "poetry",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some("poetry"),
                    Some("install"),
                    Some(match source.group_mode {
                        crate::schema::TaskPoetryHydrationGroupMode::With => "with",
                        crate::schema::TaskPoetryHydrationGroupMode::Only => "only",
                    }),
                    source.groups.clone(),
                    false,
                    false,
                    false,
                    source.no_root,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::GoModules(source) => (
                    "go_modules",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    None,
                    Some("download"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Helm(source) => (
                    "helm",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some("helm"),
                    Some("dependency_build"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Maven(source) => (
                    "maven",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some(if source.wrapper { "./mvnw" } else { "mvn" }),
                    Some(source.mode.goal()),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    source.skip_tests,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Gradle(source) => (
                    "gradle",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some(if source.wrapper {
                        "./gradlew"
                    } else {
                        "gradle"
                    }),
                    Some("dependencies"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Cargo(source) => (
                    "cargo",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some("cargo"),
                    Some("fetch"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::DotnetRestore(source) => (
                    "dotnet_restore",
                    Some(source.cwd.clone()),
                    None,
                    Vec::new(),
                    Vec::new(),
                    Some("dotnet"),
                    Some("restore"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                    summarize_task_compose_invocation_owned(source.compose.as_ref()),
                ),
            };
            let mut declared_hydration_provenance = None;
            let mut resolved_hydration_provenance = None;
            if let crate::schema::TaskDependencyHydrationSourceSpec::DotnetRestore(source) =
                &spec.source
            {
                let provenance = WorkspaceTaskHydrationProvenanceSummary {
                    source_posture: source.source_posture(),
                    config_file: source.config_file.clone(),
                    sources: source.sources.clone(),
                    source_identities: Vec::new(),
                    resolution: None,
                    resolution_error: None,
                };
                declared_hydration_provenance = Some(provenance.clone());
                resolved_hydration_provenance = None;
            }
            Some(WorkspaceTaskPrepareSummary {
                kind: "dependency_hydration",
                steps: Vec::new(),
                medium: Some(match spec.medium {
                    crate::schema::TaskDependencyHydrationMedium::ContainerImages => {
                        "container_images"
                    }
                    crate::schema::TaskDependencyHydrationMedium::PackageDependencies => {
                        "package_dependencies"
                    }
                }),
                source_kind: Some(source_kind),
                cwd,
                file,
                files,
                env_files,
                manager,
                filter: match &spec.source {
                    crate::schema::TaskDependencyHydrationSourceSpec::NodePackageManager(
                        source,
                    ) => source
                        .filter
                        .as_deref()
                        .map(str::trim)
                        .filter(|filter| !filter.is_empty())
                        .map(str::to_string),
                    _ => None,
                },
                mode,
                group_mode,
                groups,
                frozen_lockfile,
                inline_builds,
                force,
                no_root,
                skip_tests,
                with_deps: false,
                targets: spec.targets.clone(),
                browsers: Vec::new(),
                declared_hydration_provenance,
                resolved_hydration_provenance,
                compose,
            })
        }
    }
}

fn summarize_task_prepare_sequence_step_owned(
    step: &crate::schema::TaskPrepareSequenceStepSpec,
) -> Option<WorkspaceTaskPrepareSummary> {
    match step {
        crate::schema::TaskPrepareSequenceStepSpec::DependencyHydration(spec) => {
            summarize_task_prepare_owned(Some(
                &crate::schema::TaskPrepareSpec::DependencyHydration(spec.clone()),
            ))
        }
        crate::schema::TaskPrepareSequenceStepSpec::ToolBootstrap(spec) => {
            summarize_task_prepare_owned(Some(&crate::schema::TaskPrepareSpec::ToolBootstrap(
                spec.clone(),
            )))
        }
        crate::schema::TaskPrepareSequenceStepSpec::Sequence(spec) => summarize_task_prepare_owned(
            Some(&crate::schema::TaskPrepareSpec::Sequence(spec.clone())),
        ),
        crate::schema::TaskPrepareSequenceStepSpec::CopyIfMissing(_) => {
            Some(empty_workspace_task_prepare_summary("copy_if_missing"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureEnvFile(_) => {
            Some(empty_workspace_task_prepare_summary("ensure_env_file"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureFile(_) => {
            Some(empty_workspace_task_prepare_summary("ensure_file"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureDirectory(_) => {
            Some(empty_workspace_task_prepare_summary("ensure_directory"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureVirtualenv(_) => {
            Some(empty_workspace_task_prepare_summary("ensure_virtualenv"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureGitCheckout(_) => {
            Some(empty_workspace_task_prepare_summary("ensure_git_checkout"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureGitTemplate(_) => {
            Some(empty_workspace_task_prepare_summary("ensure_git_template"))
        }
        crate::schema::TaskPrepareSequenceStepSpec::EnsureContainerNetwork(_) => Some(
            empty_workspace_task_prepare_summary("ensure_container_network"),
        ),
        crate::schema::TaskPrepareSequenceStepSpec::ResetComposeServiceVolume(_) => Some(
            empty_workspace_task_prepare_summary("reset_compose_service_volume"),
        ),
    }
}

fn empty_workspace_task_prepare_summary(kind: &'static str) -> WorkspaceTaskPrepareSummary {
    WorkspaceTaskPrepareSummary {
        kind,
        steps: Vec::new(),
        medium: None,
        source_kind: None,
        cwd: None,
        file: None,
        files: Vec::new(),
        env_files: Vec::new(),
        manager: None,
        filter: None,
        mode: None,
        group_mode: None,
        groups: Vec::new(),
        frozen_lockfile: false,
        inline_builds: false,
        force: false,
        no_root: false,
        skip_tests: false,
        with_deps: false,
        targets: Vec::new(),
        browsers: Vec::new(),
        declared_hydration_provenance: None,
        resolved_hydration_provenance: None,
        compose: None,
    }
}

pub fn workspace_prepare_summary_from_task_prepare_summary(
    summary: TaskPrepareSummary<'_>,
) -> WorkspaceTaskPrepareSummary {
    WorkspaceTaskPrepareSummary {
        kind: summary.kind,
        steps: summary
            .steps
            .into_iter()
            .map(workspace_prepare_summary_from_task_prepare_summary)
            .collect(),
        medium: summary.medium,
        source_kind: summary.source_kind,
        cwd: summary.cwd.map(str::to_string),
        file: summary.file.map(str::to_string),
        files: summary.files.into_iter().map(str::to_string).collect(),
        env_files: summary.env_files.into_iter().map(str::to_string).collect(),
        manager: summary.manager,
        filter: summary.filter.map(str::to_string),
        mode: summary.mode,
        group_mode: summary.group_mode,
        groups: summary.groups.into_iter().map(str::to_string).collect(),
        frozen_lockfile: summary.frozen_lockfile,
        inline_builds: summary.inline_builds,
        force: summary.force,
        no_root: summary.no_root,
        skip_tests: summary.skip_tests,
        with_deps: summary.with_deps,
        targets: summary.targets.into_iter().map(str::to_string).collect(),
        browsers: summary.browsers,
        declared_hydration_provenance: summary.declared_hydration_provenance.map(|value| {
            WorkspaceTaskHydrationProvenanceSummary {
                source_posture: value.source_posture,
                config_file: value.config_file.map(str::to_string),
                sources: value.sources.into_iter().map(str::to_string).collect(),
                source_identities: Vec::new(),
                resolution: None,
                resolution_error: None,
            }
        }),
        resolved_hydration_provenance: summary.resolved_hydration_provenance.map(|value| {
            WorkspaceTaskHydrationProvenanceSummary {
                source_posture: value.source_posture,
                config_file: value.config_file.map(str::to_string),
                sources: value.sources.into_iter().map(str::to_string).collect(),
                source_identities: Vec::new(),
                resolution: None,
                resolution_error: None,
            }
        }),
        compose: summary
            .compose
            .map(|compose| WorkspaceTaskComposeInvocationSummary {
                kind: compose.kind,
                engine: compose.engine,
                service: compose.service.map(str::to_string),
                services: compose.services.into_iter().map(str::to_string).collect(),
                workdir: compose.workdir.map(str::to_string),
                rm: compose.rm,
                build: compose.build,
                service_ports: compose.service_ports,
                detach: compose.detach,
                force_recreate: compose.force_recreate,
                force: compose.force,
                follow: compose.follow,
                remove_volumes: compose.remove_volumes,
                timeout_seconds: compose.timeout_seconds,
                tty: compose.tty,
            }),
    }
}

fn task_mode_name(mode: Backend) -> &'static str {
    match mode {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceSummary {
    pub name: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub producer: Option<ServiceProducerSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<ServiceManagerSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub readiness: Option<ServiceReadinessSummary>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub endpoints: BTreeMap<String, ServiceEndpointSummary>,
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

impl ServiceSummary {
    pub fn from_spec(name: &str, service: &ServiceSpec) -> Self {
        Self {
            name: name.to_string(),
            required: service.required,
            producer: service
                .producer
                .as_ref()
                .map(ServiceProducerSummary::from_spec),
            manager: service
                .manager
                .as_ref()
                .map(ServiceManagerSummary::from_spec),
            provider: service.provider.clone(),
            start: service.start_command(name),
            stop: service.stop_command(name),
            healthcheck: service.healthcheck.clone(),
            readiness: service
                .readiness
                .as_ref()
                .map(ServiceReadinessSummary::from_spec),
            endpoints: service
                .endpoints
                .iter()
                .map(|(context, endpoint)| {
                    (context.clone(), ServiceEndpointSummary::from_spec(endpoint))
                })
                .collect(),
            depends_on: service.depends_on.clone(),
            timeout: service.timeout,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceProducerSummary {
    pub repo: String,
    pub task: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub listener: Option<String>,
    pub address_view: String,
}

impl ServiceProducerSummary {
    pub fn from_spec(producer: &crate::schema::ServiceProducerSpec) -> Self {
        Self {
            repo: producer.repo.clone(),
            task: producer.task.clone(),
            listener: producer.listener.clone(),
            address_view: match producer.address_view {
                crate::schema::TaskTargetAddressView::Topology => "topology",
                crate::schema::TaskTargetAddressView::Host => "host",
                crate::schema::TaskTargetAddressView::Internal => "internal",
            }
            .to_string(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceReadinessSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub probe: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub kind: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub success: Option<ExecutionTopologyReadinessSuccessSummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub body: Option<ExecutionTopologyReadinessBodySummary>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,
}

impl ServiceReadinessSummary {
    fn from_spec(readiness: &crate::schema::ServiceReadinessSpec) -> Self {
        Self {
            from: readiness.from.clone(),
            endpoint: readiness.endpoint.clone(),
            run: readiness.run.clone(),
            probe: readiness.probe.clone(),
            kind: readiness.kind.map(|kind| kind.as_str().to_string()),
            method: readiness.method.map(|method| method.as_str().to_string()),
            path: readiness.path.clone(),
            headers: readiness.headers.clone(),
            success: readiness.success.as_ref().map(|success| {
                ExecutionTopologyReadinessSuccessSummary {
                    status: success.status.clone(),
                }
            }),
            body: readiness
                .body
                .as_ref()
                .map(|body| ExecutionTopologyReadinessBodySummary {
                    contains: body.contains.clone(),
                }),
            interval: readiness.interval.clone(),
            timeout: readiness.timeout.clone(),
            retries: readiness.retries,
            start_period: readiness.start_period.clone(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceEndpointSummary {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<String>,
    pub address: String,
    pub port: u16,
}

impl ServiceEndpointSummary {
    fn from_spec(endpoint: &crate::schema::ServiceEndpointSpec) -> Self {
        Self {
            context: endpoint.context.clone(),
            address: endpoint.address.clone(),
            port: endpoint.port,
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceManagerSummary {
    pub kind: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
}

impl ServiceManagerSummary {
    fn from_spec(manager: &crate::schema::ServiceManagerSpec) -> Self {
        Self {
            kind: match manager.kind {
                crate::schema::ServiceManagerKind::Compose => String::from("compose"),
                crate::schema::ServiceManagerKind::Host => String::from("host"),
            },
            name: manager.name.clone(),
            file: manager.file.clone(),
            files: manager.files.clone(),
            env_file: manager.env_file.clone(),
            env_files: manager.env_files.clone(),
            profiles: manager.profiles.clone(),
            service: manager.service.clone(),
        }
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::parser::parse_contract_str;

    #[test]
    fn summarize_task_compose_uses_contract_kind_label() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  db:migrate:
    compose:
      kind: exec
      detach: true
      service: api
      exe: bundle
      args:
        - exec
        - rails
        - db:migrate
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "db:migrate",
            contract.tasks.get("db:migrate").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_exec");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "exec");
        assert_eq!(compose.service, Some("api"));
        assert!(compose.detach);
    }

    #[test]
    fn task_summary_exposes_native_override_for_container_context_task() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  contexts:
    verify:
      backend: container
      container:
        image: node:24-bookworm
tasks:
  build:
    context: verify
    safe_for_agent: true
    command:
      exe: npm
      args:
        - run
        - build
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "build",
            contract.tasks.get("build").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.effective_default_mode, "container");
        assert!(summary.supports_native_mode_override);
        assert_eq!(summary.usage.modes.len(), 2);

        let container = &summary.usage.modes[0];
        assert_eq!(container.mode, "container");
        assert!(container.default);
        assert_eq!(
            container.availability,
            super::LaneUseModeAvailability::Supported
        );
        assert_eq!(container.human.command.as_deref(), Some("ota run build"));
        assert_eq!(
            container.agent.command.as_deref(),
            Some("ota run build --agent")
        );

        let native = &summary.usage.modes[1];
        assert_eq!(native.mode, "native");
        assert!(!native.default);
        assert_eq!(
            native.availability,
            super::LaneUseModeAvailability::Supported
        );
        assert_eq!(
            native.human.command.as_deref(),
            Some("ota run build --native")
        );
        assert_eq!(
            native.agent.command.as_deref(),
            Some("ota run build --native --agent")
        );
    }

    #[test]
    fn summarize_task_compose_up_uses_service_groups() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  staged:up:
    compose:
      kind: up
      detach: true
      services:
        - web
        - worker
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "staged:up",
            contract.tasks.get("staged:up").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_up");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "up");
        assert_eq!(compose.service, None);
        assert_eq!(compose.services, vec!["web", "worker"]);
        assert_eq!(compose.exe, None);
        assert!(compose.detach);
    }

    #[test]
    fn task_summary_uses_orchestrator_launcher_subcommand_preview() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
orchestrators:
  devenv:
    kind: devenv
    launcher:
      exe: nix
      args:
        - run
        - github:cachix/devenv/main#devenv
        - --
tasks:
  test:
    command:
      exe: test
    execution:
      orchestrator:
        ref: devenv
        mode: subcommand
  dev:
    launch:
      kind: command
      exe: up
    execution:
      orchestrator:
        ref: devenv
        mode: subcommand
"#,
        )
        .expect("contract should parse");

        let test_summary = super::TaskSummary::from_spec(
            "test",
            contract.tasks.get("test").expect("task should exist"),
            "linux",
            &contract,
        );
        assert_eq!(
            test_summary.preview,
            "nix run github:cachix/devenv/main#devenv -- test"
        );
        assert_eq!(test_summary.launch_preview, None);

        let dev_summary = super::TaskSummary::from_spec(
            "dev",
            contract.tasks.get("dev").expect("task should exist"),
            "linux",
            &contract,
        );
        assert_eq!(
            dev_summary.preview,
            "nix run github:cachix/devenv/main#devenv -- up"
        );
        assert_eq!(
            dev_summary.launch_preview,
            Some(String::from(
                "nix run github:cachix/devenv/main#devenv -- up"
            ))
        );
    }

    #[test]
    fn summarize_task_compose_build_uses_service_groups() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  image:build:
    compose:
      kind: build
      services:
        - web
        - worker
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "image:build",
            contract
                .tasks
                .get("image:build")
                .expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_build");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "build");
        assert_eq!(compose.service, None);
        assert_eq!(compose.services, vec!["web", "worker"]);
        assert_eq!(compose.exe, None);
    }

    #[test]
    fn summarize_task_compose_down_can_remove_volumes() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  stack:clean:
    compose:
      kind: down
      remove_volumes: true
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "stack:clean",
            contract
                .tasks
                .get("stack:clean")
                .expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_down");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "down");
        assert!(compose.remove_volumes);
    }

    #[test]
    fn summarize_task_compose_down_can_set_timeout() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  stack:down:
    compose:
      kind: down
      timeout_seconds: 2
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "stack:down",
            contract.tasks.get("stack:down").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_down");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "down");
        assert_eq!(compose.timeout_seconds, Some(2));
    }

    #[test]
    fn summarize_task_compose_restart_uses_service_groups() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  stack:restart:
    compose:
      kind: restart
      services:
        - web
        - worker
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "stack:restart",
            contract
                .tasks
                .get("stack:restart")
                .expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_restart");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "restart");
        assert_eq!(compose.service, None);
        assert_eq!(compose.services, vec!["web", "worker"]);
    }

    #[test]
    fn summarize_task_compose_stop_uses_service_groups() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  stack:stop:
    compose:
      kind: stop
      services:
        - web
        - worker
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "stack:stop",
            contract.tasks.get("stack:stop").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_stop");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "stop");
        assert_eq!(compose.service, None);
        assert_eq!(compose.services, vec!["web", "worker"]);
    }

    #[test]
    fn summarize_task_compose_logs_can_follow() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  stack:logs:
    compose:
      kind: logs
      follow: true
      services:
        - web
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "stack:logs",
            contract.tasks.get("stack:logs").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_logs");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "logs");
        assert!(compose.follow);
        assert_eq!(compose.services, vec!["web"]);
    }

    #[test]
    fn summarize_task_compose_ps_uses_service_groups() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  stack:ps:
    compose:
      kind: ps
      services:
        - main
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "stack:ps",
            contract.tasks.get("stack:ps").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_ps");
        let compose = summary.compose.expect("compose summary should exist");
        assert_eq!(compose.kind, "ps");
        assert_eq!(compose.service, None);
        assert_eq!(compose.services, vec!["main"]);
    }

    #[test]
    fn summarize_task_launch_compose_up_uses_service_groups() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  selfhost:
    launch:
      kind: compose
      engine: podman
      action: up
      detach: true
      services:
        - api
        - worker
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "selfhost",
            contract.tasks.get("selfhost").expect("task should exist"),
            "linux",
            &contract,
        );

        let launch = summary.launch.expect("launch summary should exist");
        assert_eq!(launch.kind, "compose");
        assert_eq!(launch.engine, Some("podman"));
        assert_eq!(launch.action, Some("up"));
        assert_eq!(launch.services, vec!["api", "worker"]);
        assert!(launch.detach);
    }

    #[test]
    fn summarize_task_uses_selected_variant_adapter_inputs_with_base_compose() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    adapter_inputs:
      overlays:
        compose:
          files:
            - compose.yaml
          project_name: app
    compose:
      kind: up
      detach: true
      services:
        - web
    variants:
      - when:
          os: linux
        adapter_inputs:
          overlays:
            compose:
              env_files:
                - .env.linux
              files:
                - compose.linux.yaml
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "dev",
            contract.tasks.get("dev").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.kind, "compose_up");
        assert_eq!(summary.selected_variant_os, Some("linux"));
        let compose = summary
            .adapter_inputs
            .compose
            .expect("compose adapter inputs should exist");
        assert_eq!(compose.env_files, vec![String::from(".env.linux")]);
        assert_eq!(
            compose.files,
            vec![
                String::from("compose.yaml"),
                String::from("compose.linux.yaml")
            ]
        );
        assert_eq!(compose.project_name.as_deref(), Some("app"));
        assert_eq!(summary.variants.len(), 1);
        assert!(
            !summary.variants[0].adapter_inputs.is_empty(),
            "variant summary should surface adapter inputs"
        );
    }

    #[test]
    fn summarize_task_uses_selected_variant_env_with_base_command() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    env:
      APP_MODE: base
    command:
      exe: echo
      args:
        - hi
    variants:
      - when:
          os: linux
        env:
          APP_MODE: linux
        env_files:
          - .env.linux
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "dev",
            contract.tasks.get("dev").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(summary.selected_variant_os, Some("linux"));
        assert_eq!(
            summary.env.get("APP_MODE").map(String::as_str),
            Some("linux")
        );
        assert_eq!(summary.env_files, vec![String::from(".env.linux")]);
        assert_eq!(
            summary.variants[0].env.get("APP_MODE").map(String::as_str),
            Some("linux")
        );
        assert_eq!(summary.variants[0].env_files, vec![".env.linux"]);
    }

    #[test]
    fn summarize_task_uses_selected_variant_inputs_with_base_command() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    inputs:
      profile:
        default: dev
        allowed:
          - dev
          - prod
    command:
      exe: echo
      args:
        - hi
    variants:
      - when:
          os: linux
        inputs:
          profile:
            default: ci
            allowed:
              - ci
              - prod
"#,
        )
        .expect("contract should parse");

        let summary = super::TaskSummary::from_spec(
            "dev",
            contract.tasks.get("dev").expect("task should exist"),
            "linux",
            &contract,
        );

        assert_eq!(
            summary
                .inputs
                .get("profile")
                .and_then(|spec| spec.default.as_deref()),
            Some("ci")
        );
        assert_eq!(
            summary.variants[0]
                .inputs
                .get("profile")
                .and_then(|spec| spec.default.as_deref()),
            Some("ci")
        );
    }

    #[test]
    fn summarize_dependency_hydration_prepare_includes_compose_wrapper() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: node_package_manager
        cwd: app
        manager: npm
        mode: ci
        compose:
          kind: run
          service: app
          workdir: /workspace
          rm: true
"#,
        )
        .expect("contract should parse");

        let task = contract.tasks.get("setup").expect("task should exist");
        let prepare = super::summarize_task_prepare(task.prepare.as_ref())
            .expect("prepare summary should exist");
        let compose = prepare
            .compose
            .expect("compose wrapper should be summarized");

        assert_eq!(compose.kind, "run");
        assert_eq!(compose.engine, "docker");
        assert_eq!(compose.service, Some("app"));
        assert_eq!(compose.workdir, Some("/workspace"));
        assert!(compose.rm);
    }
}
