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
    load_policy_env_overlay, resolve_declared_env_source_value,
    resolve_execution_backend_with_contract_path,
};
use crate::schema::{
    AgentConfig, Backend, Contract, ExecutionContext, ExtensionSpec, Lifecycle, ServiceSpec,
    TaskInputSpec, TaskSpec, TaskVariantView,
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
    pub path: String,
    pub change: String,
    pub status: String,
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
    pub requirement_lines: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub actions: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub notes: Vec<String>,
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
    pub repos: &'a [WorkspaceRepoStatusReport],
}

#[derive(Debug, Serialize)]
pub struct ReceiptSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    pub summary: ExecutionReceiptSummary,
    pub receipt: ExecutionReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub archive_path: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub promoted_baseline: Option<ReceiptPromotedBaseline>,
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
pub struct ReceiptDiffCounts {
    pub count: usize,
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ReceiptDiffComparison {
    pub baseline_identity_label: String,
    pub current_identity_label: String,
    pub identity_changed: bool,
    pub readiness_change: ReceiptDiffReadinessChange,
}

#[derive(Debug, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReceiptDiffReadinessChange {
    Unchanged,
    Improved,
    Regressed,
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
    pub findings: &'a [Finding],
    pub receipt: ExecutionReceipt,
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

#[derive(Debug, Serialize)]
pub struct ProofRuntimeStatus<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub mode: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workflow: Option<&'a str>,
    pub phase: &'a str,
    pub summary: DoctorSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub artifacts: Option<ProofRuntimeArtifacts<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub workflow_env_artifacts: Vec<EnvRenderedArtifactEntry>,
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
    pub skipped: Vec<String>,
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

#[derive(Debug, Serialize)]
pub struct DiffChange {
    pub path: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provenance: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct DiffSuccess<'a> {
    pub ok: bool,
    pub base: &'a str,
    pub target: &'a str,
    pub summary: DiffSummary,
    pub changes: &'a [DiffChange],
}

#[derive(Debug, Serialize)]
pub struct DiffFailure<'a> {
    pub ok: bool,
    pub base: &'a str,
    pub target: &'a str,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<MemberWorkflowsSuccess<'a>>,
    pub workflows: Vec<ListedWorkflowSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct MemberWorkflowsSuccess<'a> {
    pub member: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default: Option<&'a str>,
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

#[derive(Debug, Serialize)]
pub struct WorkflowSummary<'a> {
    pub name: &'a str,
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
    pub run_task_launch: Option<TaskLaunchSummary<'a>>,
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
            run_task_launch: workflow
                .run
                .as_ref()
                .and_then(|phase| contract.tasks.get(phase.task.as_str()))
                .and_then(|task| {
                    let backend = task.workflow_backend(contract.execution.as_ref());
                    task.resolved_execution_for_backend(backend, current_os())
                })
                .and_then(|execution| summarize_task_launch(execution.launch())),
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
        Self::from_contract_named_inner(contract, contract_path, name)
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
    pub sh: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub powershell: Option<&'a str>,
}

impl<'a> AgentBootstrapTargetSummary<'a> {
    pub fn from_config(bootstrap: &'a crate::schema::AgentBootstrapTargetConfig) -> Self {
        Self {
            note: bootstrap.note.as_deref(),
            sh: bootstrap.sh.as_deref(),
            powershell: bootstrap.powershell.as_deref(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct TaskSummary<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_mode: Option<&'static str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<&'a str>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env: &'a BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<&'a str>,
    #[serde(skip_serializing_if = "TaskAdapterInputsSummary::is_empty")]
    pub adapter_inputs: TaskAdapterInputsSummary<'a>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub inputs: &'a BTreeMap<String, TaskInputSpec>,
    pub kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub command: Option<TaskCommandSummary<'a>>,
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
    pub when_checks: Vec<String>,
    pub after_success: Vec<String>,
    pub after_failure: Vec<String>,
    pub after_always: Vec<String>,
    pub safe_for_agent: bool,
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
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub network: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub network_kind: Option<crate::schema::TaskNetworkEffectKind>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub external_state: Vec<String>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TaskAggregateSummary {
    pub tasks: Vec<String>,
}

#[derive(Debug, Serialize, Clone, Default, PartialEq, Eq)]
pub struct TaskAdapterInputsSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub compose: Option<TaskComposeAdapterInputsSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub bake: Option<TaskBakeAdapterInputsSummary<'a>>,
}

impl<'a> TaskAdapterInputsSummary<'a> {
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
pub struct TaskComposeAdapterInputsSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub profiles: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub project_name: Option<&'a str>,
}

impl<'a> TaskComposeAdapterInputsSummary<'a> {
    pub fn is_empty(&self) -> bool {
        self.cwd.is_none()
            && self.env_files.is_empty()
            && self.files.is_empty()
            && self.profiles.is_empty()
            && self.project_name.is_none()
    }
}

#[derive(Debug, Serialize, Clone, Default, PartialEq, Eq)]
pub struct TaskBakeAdapterInputsSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cwd: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub files: Vec<&'a str>,
}

impl<'a> TaskBakeAdapterInputsSummary<'a> {
    pub fn is_empty(&self) -> bool {
        self.cwd.is_none() && self.files.is_empty()
    }
}

impl TaskEffectsSummary {
    pub fn from_spec(spec: &crate::schema::TaskEffectsSpec) -> Self {
        Self {
            writes: spec.writes.clone(),
            network: spec.network,
            network_kind: spec.network_kind,
            external_state: spec.external_state.clone(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.writes.is_empty()
            && !self.network
            && self.network_kind.is_none()
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
        let effective = effective_task_execution(contract, name, overrides);
        let selected_backend = effective.backend;
        let resolved_execution = task
            .resolved_execution_for_backend(selected_backend, current_os)
            .expect("validated task must resolve to a default or variant execution");
        Self {
            name,
            context: effective.context_name,
            default_mode: task.mode_default_backend().map(task_mode_name),
            description: task.description.as_deref(),
            notes: task.notes.as_deref(),
            category: task.category.as_deref(),
            env: &task.env,
            env_files: task.env_files.iter().map(String::as_str).collect(),
            adapter_inputs: summarize_task_adapter_inputs(&task.adapter_inputs),
            inputs: &task.inputs,
            kind: resolved_execution.kind,
            run: (resolved_execution.kind == "run")
                .then(|| resolved_execution.shell_body())
                .flatten(),
            script: (resolved_execution.kind == "script")
                .then(|| resolved_execution.shell_body())
                .flatten(),
            command: summarize_task_command(resolved_execution.command()),
            launch: summarize_task_launch(resolved_execution.launch()),
            action: summarize_task_action(resolved_execution.action()),
            prepare: summarize_task_prepare(resolved_execution.prepare()),
            aggregate: summarize_task_aggregate(resolved_execution.aggregate()),
            effects: TaskEffectsSummary::from_spec(&task.effects),
            selected_variant_os: resolved_execution.os,
            depends_on: task.depends_on.clone(),
            requires_services: task.requires_services.clone(),
            when_checks: task.when.checks.clone(),
            after_success: task.after_success.clone(),
            after_failure: task.after_failure.clone(),
            after_always: task.after_always.clone(),
            safe_for_agent: task.safe_for_agent,
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
                        .expect("validated task variant must declare exactly one execution form"),
                    run: variant.run.as_deref(),
                    script: variant.script.as_deref(),
                    command: variant
                        .command
                        .as_ref()
                        .and_then(|command| summarize_task_command(Some(command))),
                })
                .collect(),
            modes: task
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
                                env_files: branch.env_files.iter().map(String::as_str).collect(),
                                adapter_inputs: summarize_task_adapter_inputs(
                                    &branch.adapter_inputs,
                                ),
                                lifecycle: branch.lifecycle.map(format_lifecycle),
                                kind: branch_execution.map(|execution| execution.kind),
                                run: branch.run.as_deref(),
                                script: branch.script.as_deref(),
                                command: summarize_task_command(branch.command.as_ref()),
                                launch: branch
                                    .launch
                                    .as_ref()
                                    .and_then(|launch| summarize_task_launch(Some(launch))),
                                prepare: summarize_task_prepare(branch.prepare.as_ref()),
                                has_runtime: branch.runtime.is_some(),
                            }
                        })
                        .collect()
                })
                .unwrap_or_default(),
            supports_native_mode_override: task.mode_default_backend()
                == Some(crate::schema::Backend::Container)
                && task
                    .mode_execution_branch(crate::schema::Backend::Native)
                    .is_none()
                && task.resolved_execution(current_os).is_some(),
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

pub fn summarize_task_aggregate(
    aggregate: Option<&crate::schema::TaskAggregateSpec>,
) -> Option<TaskAggregateSummary> {
    aggregate.map(|aggregate| TaskAggregateSummary {
        tasks: aggregate.tasks.clone(),
    })
}

#[derive(Debug, Serialize)]
pub struct TaskModeView<'a> {
    pub mode: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub context: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_files: Vec<&'a str>,
    #[serde(skip_serializing_if = "TaskAdapterInputsSummary::is_empty")]
    pub adapter_inputs: TaskAdapterInputsSummary<'a>,
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
}

pub fn summarize_task_adapter_inputs<'a>(
    adapter_inputs: &'a crate::schema::TaskAdapterInputsSpec,
) -> TaskAdapterInputsSummary<'a> {
    TaskAdapterInputsSummary {
        compose: adapter_inputs
            .compose
            .as_ref()
            .map(|compose| TaskComposeAdapterInputsSummary {
                cwd: compose
                    .cwd
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
                env_files: compose.env_files.iter().map(String::as_str).collect(),
                files: compose.files.iter().map(String::as_str).collect(),
                profiles: compose.profiles.iter().map(String::as_str).collect(),
                project_name: compose
                    .project_name
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
            })
            .filter(|compose| !compose.is_empty()),
        bake: adapter_inputs
            .bake
            .as_ref()
            .map(|bake| TaskBakeAdapterInputsSummary {
                cwd: bake
                    .cwd
                    .as_deref()
                    .map(str::trim)
                    .filter(|value| !value.is_empty()),
                files: bake.files.iter().map(String::as_str).collect(),
            })
            .filter(|bake| !bake.is_empty()),
    }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<&'static str>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<&'a str>,
}

#[derive(Debug, Serialize, Clone)]
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub manager: Option<&'static str>,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub targets: Vec<String>,
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
        crate::schema::TaskActionSpec::EnsureContainerNetwork(spec) => Some(TaskActionSummary {
            kind: "ensure_container_network",
            from: Some(spec.provider.label()),
            to: Some(spec.name.as_str()),
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
        crate::schema::TaskActionSpec::EnsureContainerNetwork(spec) => {
            Some(WorkspaceTaskActionSummary {
                kind: "ensure_container_network",
                from: Some(spec.provider.label().to_string()),
                to: Some(spec.name.clone()),
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
                .filter_map(|step| summarize_task_prepare(Some(step)))
                .collect(),
            medium: None,
            source_kind: None,
            cwd: None,
            file: None,
            manager: None,
            mode: None,
            group_mode: None,
            groups: Vec::new(),
            frozen_lockfile: false,
            inline_builds: false,
            force: false,
            no_root: false,
            skip_tests: false,
            targets: Vec::new(),
        }),
        crate::schema::TaskPrepareSpec::ToolBootstrap(spec) => {
            let (source_kind, mode) = match &spec.source {
                crate::schema::TaskToolBootstrapSourceSpec::Pip(_source) => {
                    ("pip", Some(spec.tool.label()))
                }
            };
            Some(TaskPrepareSummary {
                kind: "tool_bootstrap",
                steps: Vec::new(),
                medium: None,
                source_kind: Some(source_kind),
                cwd: None,
                file: None,
                manager: None,
                mode,
                group_mode: None,
                groups: Vec::new(),
                frozen_lockfile: false,
                inline_builds: false,
                force: false,
                no_root: false,
                skip_tests: false,
                targets: Vec::new(),
            })
        }
        crate::schema::TaskPrepareSpec::DependencyHydration(spec) => {
            let (
                source_kind,
                cwd,
                file,
                manager,
                mode,
                group_mode,
                groups,
                frozen_lockfile,
                inline_builds,
                force,
                no_root,
                skip_tests,
            ) = match &spec.source {
                crate::schema::TaskDependencyHydrationSourceSpec::DockerCompose(source) => (
                    "docker_compose",
                    Some(source.cwd.as_str()),
                    Some(source.file.as_str()),
                    None,
                    None,
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::NodePackageManager(source) => (
                    "node_package_manager",
                    Some(source.cwd.as_str()),
                    None,
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
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Bundler(source) => (
                    "bundler",
                    Some(source.cwd.as_str()),
                    Some(source.path.as_str()),
                    Some("bundle"),
                    Some("install"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Uv(source) => (
                    "uv",
                    Some(source.cwd.as_str()),
                    None,
                    Some("uv"),
                    Some("sync"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Poetry(source) => (
                    "poetry",
                    Some(source.cwd.as_str()),
                    None,
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
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::GoModules(source) => (
                    "go_modules",
                    Some(source.cwd.as_str()),
                    None,
                    None,
                    Some("download"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Maven(source) => (
                    "maven",
                    Some(source.cwd.as_str()),
                    None,
                    Some(if source.wrapper { "./mvnw" } else { "mvn" }),
                    Some(source.mode.goal()),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    source.skip_tests,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Gradle(source) => (
                    "gradle",
                    Some(source.cwd.as_str()),
                    None,
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
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Cargo(source) => (
                    "cargo",
                    Some(source.cwd.as_str()),
                    None,
                    Some("cargo"),
                    Some("fetch"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::DotnetRestore(source) => (
                    "dotnet_restore",
                    Some(source.cwd.as_str()),
                    None,
                    Some("dotnet"),
                    Some("restore"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
            };
            Some(TaskPrepareSummary {
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
                manager,
                mode,
                group_mode,
                groups,
                frozen_lockfile,
                inline_builds,
                force,
                no_root,
                skip_tests,
                targets: spec.targets.iter().map(String::as_str).collect(),
            })
        }
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
                .filter_map(|step| summarize_task_prepare_owned(Some(step)))
                .collect(),
            medium: None,
            source_kind: None,
            cwd: None,
            file: None,
            manager: None,
            mode: None,
            group_mode: None,
            groups: Vec::new(),
            frozen_lockfile: false,
            inline_builds: false,
            force: false,
            no_root: false,
            skip_tests: false,
            targets: Vec::new(),
        }),
        crate::schema::TaskPrepareSpec::ToolBootstrap(spec) => {
            let (source_kind, mode) = match &spec.source {
                crate::schema::TaskToolBootstrapSourceSpec::Pip(_source) => {
                    ("pip", Some(spec.tool.label()))
                }
            };
            Some(WorkspaceTaskPrepareSummary {
                kind: "tool_bootstrap",
                steps: Vec::new(),
                medium: None,
                source_kind: Some(source_kind),
                cwd: None,
                file: None,
                manager: None,
                mode,
                group_mode: None,
                groups: Vec::new(),
                frozen_lockfile: false,
                inline_builds: false,
                force: false,
                no_root: false,
                skip_tests: false,
                targets: Vec::new(),
            })
        }
        crate::schema::TaskPrepareSpec::DependencyHydration(spec) => {
            let (
                source_kind,
                cwd,
                file,
                manager,
                mode,
                group_mode,
                groups,
                frozen_lockfile,
                inline_builds,
                force,
                no_root,
                skip_tests,
            ) = match &spec.source {
                crate::schema::TaskDependencyHydrationSourceSpec::DockerCompose(source) => (
                    "docker_compose",
                    Some(source.cwd.clone()),
                    Some(source.file.clone()),
                    None,
                    None,
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::NodePackageManager(source) => (
                    "node_package_manager",
                    Some(source.cwd.clone()),
                    None,
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
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Bundler(source) => (
                    "bundler",
                    Some(source.cwd.clone()),
                    Some(source.path.clone()),
                    Some("bundle"),
                    Some("install"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Uv(source) => (
                    "uv",
                    Some(source.cwd.clone()),
                    None,
                    Some("uv"),
                    Some("sync"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Poetry(source) => (
                    "poetry",
                    Some(source.cwd.clone()),
                    None,
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
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::GoModules(source) => (
                    "go_modules",
                    Some(source.cwd.clone()),
                    None,
                    None,
                    Some("download"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Maven(source) => (
                    "maven",
                    Some(source.cwd.clone()),
                    None,
                    Some(if source.wrapper { "./mvnw" } else { "mvn" }),
                    Some(source.mode.goal()),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    source.skip_tests,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Gradle(source) => (
                    "gradle",
                    Some(source.cwd.clone()),
                    None,
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
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::Cargo(source) => (
                    "cargo",
                    Some(source.cwd.clone()),
                    None,
                    Some("cargo"),
                    Some("fetch"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
                crate::schema::TaskDependencyHydrationSourceSpec::DotnetRestore(source) => (
                    "dotnet_restore",
                    Some(source.cwd.clone()),
                    None,
                    Some("dotnet"),
                    Some("restore"),
                    None,
                    Vec::new(),
                    false,
                    false,
                    false,
                    false,
                    false,
                ),
            };
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
                manager,
                mode,
                group_mode,
                groups,
                frozen_lockfile,
                inline_builds,
                force,
                no_root,
                skip_tests,
                targets: spec.targets.clone(),
            })
        }
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub env_file: Option<String>,
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
            env_file: manager.env_file.clone(),
            profiles: manager.profiles.clone(),
            service: manager.service.clone(),
        }
    }
}
