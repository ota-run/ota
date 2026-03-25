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

use crate::detector::{DetectContract, Inference};
use crate::doctor::Finding;
use crate::schema::{AgentConfig, Backend, Contract, Lifecycle, ServiceSpec, TaskSpec, TaskVariantView};
use crate::workspace::WorkspaceRepoDoctorReport;

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
    #[serde(skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentSummary<'a>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub execution: Option<ExecutionSummary<'a>>,
    pub findings: &'a [Finding],
}

#[derive(Debug, Serialize)]
pub struct WorkspaceDoctorSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub repos: &'a [WorkspaceRepoDoctorReport],
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
pub struct ExecutionSummary<'a> {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preferred: Option<&'a str>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub supported: Vec<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub lifecycle: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub backends: Option<ExecutionBackendsSummary<'a>>,
}

impl<'a> ExecutionSummary<'a> {
    pub fn from_contract(contract: &'a Contract) -> Option<Self> {
        let execution = contract.execution.as_ref()?;

        Some(Self {
            preferred: execution.preferred.map(format_backend),
            supported: execution.supported.iter().map(|backend| format_backend(*backend)).collect(),
            lifecycle: execution.lifecycle.map(format_lifecycle),
            backends: execution.backends.as_ref().map(|backends| ExecutionBackendsSummary {
                container: backends.container.as_ref().map(|container| ExecutionContainerSummary {
                    image: &container.image,
                }),
                remote: backends.remote.as_ref().map(|remote| ExecutionRemoteSummary {
                    provider: &remote.provider,
                    target: remote.target.as_deref(),
                    cwd: remote.cwd.as_deref(),
                }),
            }),
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
    pub repos: &'a [WorkspaceRepoTasksReport],
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
}

#[derive(Debug, Serialize)]
pub struct WorkspaceListSuccess<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub repos: &'a [WorkspaceRepoListReport],
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

#[derive(Debug, Serialize)]
pub struct UpStatus<'a> {
    pub ok: bool,
    pub path: &'a str,
    pub status: &'a str,
    pub phase: &'a str,
    pub findings: &'a [Finding],
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
}

#[derive(Debug, Serialize)]
pub struct ValidateFailure<'a> {
    pub ok: bool,
    pub path: &'a str,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    pub errors: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
            notes: agent.notes.as_deref(),
        };

        (summary.entrypoint.is_some()
            || summary.default_task.is_some()
            || !summary.safe_tasks.is_empty()
            || !summary.verify_after_changes.is_empty()
            || !summary.writable_paths.is_empty()
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
    pub category: Option<&'a str>,
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
            category: task.category.as_deref(),
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
