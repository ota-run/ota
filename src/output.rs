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

use serde::Serialize;
use std::collections::BTreeMap;

use crate::detector::{DetectContract, Inference};
use crate::doctor::{Finding, FindingSeverity};
use crate::schema::{
    AgentConfig, Backend, Contract, ExtensionSpec, Lifecycle, ServiceSpec, TaskInputSpec, TaskSpec,
    TaskVariantView,
};
use crate::workspace::{WorkspaceExecutionSummary, WorkspaceRepoDoctorReport};

fn slice_is_empty<T>(value: &[T]) -> bool {
    value.is_empty()
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
    pub summary: DoctorSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionSummary<'a>>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub extensions: &'a BTreeMap<String, ExtensionSpec>,
    pub findings: &'a [Finding],
}

#[derive(Debug, Serialize)]
pub struct WorkspaceDoctorSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: WorkspaceDoctorSummary,
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

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
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
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct ExplainSummary {
    pub error_count: usize,
    pub warn_count: usize,
    pub info_count: usize,
    pub step_count: usize,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
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

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptStep {
    pub order: usize,
    pub label: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub detail: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceiptEnvSource {
    pub name: String,
    pub value: String,
    pub source: String,
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct ExecutionReceipt {
    pub ok: bool,
    pub path: String,
    pub scope: String,
    pub contract: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub workspace: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backend: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub acquired: Vec<String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env: BTreeMap<String, String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env_sources: Vec<ExecutionReceiptEnvSource>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub policy: Vec<String>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub steps: Vec<ExecutionReceiptStep>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub blocked: Vec<String>,
    pub summary: ExecutionReceiptSummary,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub next: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ExplainStep {
    pub order: usize,
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
}

#[derive(Debug, Serialize)]
pub struct ExplainSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: ExplainSummary,
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
    pub steps: Vec<ExplainStep>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceExplainSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: WorkspaceExplainSummary,
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
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub allowed: Vec<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct ExecutionSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub supported: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backends: Option<ExecutionBackendsSummary<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub env: Vec<ExecutionEnvSummary<'a>>,
}

impl<'a> ExecutionSummary<'a> {
    pub fn from_contract(contract: &'a Contract) -> Option<Self> {
        let execution = contract.execution.as_ref()?;

        Some(Self {
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
            env: contract
                .env
                .iter()
                .map(|(name, requirement)| ExecutionEnvSummary {
                    name,
                    required: requirement.required,
                    default: requirement.default.as_deref(),
                    allowed: requirement.allowed.iter().map(String::as_str).collect(),
                })
                .collect(),
        })
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
    pub run: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<String>,
    pub depends_on: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoTasksReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
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

#[derive(Debug, Serialize)]
pub struct WorkspaceRepoListReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
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
pub struct WorkspaceRepoUpReport {
    pub name: String,
    pub path: String,
    pub contract_path: String,
    pub required: bool,
    pub ok: bool,
    pub status: String,
    pub phase: String,
    pub findings: Vec<Finding>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct WorkspaceUpSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub summary: ExecutionReceiptSummary,
    pub receipt: ExecutionReceipt,
    pub repos: &'a [WorkspaceRepoUpReport],
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
    pub exit_code: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stdout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stderr: Option<String>,
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
    pub config: &'a DetectContract,
    pub inferred: &'a [Inference],
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
pub struct DetectSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub written: bool,
    pub config: &'a DetectContract,
    pub inferred: &'a [Inference],
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
}

#[derive(Debug, Clone, Serialize)]
pub struct DetectComparisonRemoval {
    pub field: String,
    pub existing: String,
}

#[derive(Debug, Serialize)]
pub struct UpStatus<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub status: &'a str,
    pub phase: &'a str,
    pub findings: &'a [Finding],
    pub receipt: ExecutionReceipt,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub service: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub task: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub exit_code: Option<i32>,
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
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct ValidateSummary {
    pub error_count: usize,
}

#[derive(Debug, Serialize)]
pub struct ValidateFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub summary: Option<ValidateSummary>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

#[derive(Debug, Serialize, Default, Clone, Copy, PartialEq, Eq)]
pub struct DiffSummary {
    pub added_count: usize,
    pub removed_count: usize,
    pub changed_count: usize,
    pub weakened_count: usize,
    pub strengthened_count: usize,
}

#[derive(Debug, Serialize)]
pub struct DiffChange {
    pub path: String,
    pub status: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub base: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub target: Option<String>,
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
    pub agent: Option<AgentSummary<'a>>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub members: Vec<MemberTasksSuccess<'a>>,
    pub tasks: Vec<TaskSummary<'a>>,
}

#[derive(Debug, Serialize)]
pub struct MemberTasksSuccess<'a> {
    pub member: &'a str,
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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_task: Option<&'a str>,
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub safe_tasks: &'a [String],
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub verify_after_changes: &'a [String],
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub writable_paths: &'a [String],
    #[serde(skip_serializing_if = "slice_is_empty")]
    pub protected_paths: &'a [String],
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
}

impl<'a> AgentSummary<'a> {
    pub fn from_config(agent: &'a AgentConfig) -> Option<Self> {
        let summary = Self {
            entrypoint: agent.entrypoint.as_deref(),
            default_task: agent.default_task.as_deref(),
            safe_tasks: &agent.safe_tasks,
            verify_after_changes: &agent.verify_after_changes,
            writable_paths: &agent.writable_paths,
            protected_paths: &agent.protected_paths,
            notes: agent.notes.as_deref(),
        };

        (summary.entrypoint.is_some()
            || summary.default_task.is_some()
            || !summary.safe_tasks.is_empty()
            || !summary.verify_after_changes.is_empty()
            || !summary.writable_paths.is_empty()
            || !summary.protected_paths.is_empty()
            || summary.notes.is_some())
        .then_some(summary)
    }
}

#[derive(Debug, Serialize)]
pub struct TaskSummary<'a> {
    pub name: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub description: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub category: Option<&'a str>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub env: &'a BTreeMap<String, String>,
    #[serde(skip_serializing_if = "BTreeMap::is_empty")]
    pub inputs: &'a BTreeMap<String, TaskInputSpec>,
    pub kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub selected_variant_os: Option<&'a str>,
    pub depends_on: &'a [String],
    pub safe_for_agent: bool,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub variants: Vec<TaskVariantView<'a>>,
}

impl<'a> TaskSummary<'a> {
    pub fn from_spec(name: &'a str, task: &'a TaskSpec, current_os: &str) -> Self {
        let execution = task
            .resolved_execution(current_os)
            .expect("validated task must resolve to a default or variant execution");
        Self {
            name,
            description: task.description.as_deref(),
            notes: task.notes.as_deref(),
            category: task.category.as_deref(),
            env: &task.env,
            inputs: &task.inputs,
            kind: execution.kind,
            run: (execution.kind == "run").then_some(execution.body),
            script: (execution.kind == "script").then_some(execution.body),
            selected_variant_os: execution.os,
            depends_on: &task.depends_on,
            safe_for_agent: task.safe_for_agent,
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
                })
                .collect(),
        }
    }
}

#[derive(Debug, Serialize)]
pub struct ServiceSummary {
    pub name: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub start: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub healthcheck: Option<String>,
    pub depends_on: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

impl ServiceSummary {
    pub fn from_spec(name: &str, service: &ServiceSpec) -> Self {
        Self {
            name: name.to_string(),
            required: service.required,
            provider: service.provider.clone(),
            start: service.start.clone(),
            stop: service.stop.clone(),
            healthcheck: service.healthcheck.clone(),
            depends_on: service.depends_on.clone(),
            timeout: service.timeout,
        }
    }
}
