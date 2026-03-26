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

use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub version: u32,
    pub project: Project,
    #[serde(default)]
    pub workspace: Option<RepoWorkspaceSpec>,
    #[serde(default)]
    pub execution: Option<Execution>,
    #[serde(default)]
    pub extensions: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub runtimes: BTreeMap<String, RuntimeRequirement>,
    #[serde(default)]
    pub tools: BTreeMap<String, ToolRequirement>,
    #[serde(default)]
    pub env: BTreeMap<String, EnvRequirement>,
    #[serde(default)]
    pub services: BTreeMap<String, ServiceSpec>,
    #[serde(default)]
    pub tasks: BTreeMap<String, TaskSpec>,
    #[serde(default)]
    pub checks: Vec<CheckSpec>,
    #[serde(default)]
    pub agent: Option<AgentConfig>,
    #[serde(default)]
    pub exports: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub policies: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type", default)]
    pub project_type: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RepoWorkspaceSpec {
    #[serde(rename = "type")]
    pub workspace_type: RepoWorkspaceType,
    pub members: Vec<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum RepoWorkspaceType {
    Monorepo,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Execution {
    #[serde(default)]
    pub preferred: Option<Backend>,
    #[serde(default)]
    pub supported: Vec<Backend>,
    #[serde(default)]
    pub lifecycle: Option<Lifecycle>,
    #[serde(default)]
    pub backends: Option<ExecutionBackends>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ExecutionBackends {
    #[serde(default)]
    pub container: Option<ContainerBackend>,
    #[serde(default)]
    pub remote: Option<RemoteBackend>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ContainerBackend {
    pub image: String,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RemoteBackend {
    pub provider: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Backend {
    Native,
    Container,
    Remote,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Lifecycle {
    Persistent,
    Ephemeral,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum RuntimeRequirement {
    Simple(String),
    Detailed(RuntimeDetail),
}

impl RuntimeRequirement {
    pub fn version(&self) -> &str {
        match self {
            Self::Simple(version) => version,
            Self::Detailed(detail) => &detail.version,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDetail {
    pub version: String,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub distribution: Option<String>,
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
pub enum ToolRequirement {
    Simple(String),
    Detailed(ToolDetail),
}

impl ToolRequirement {
    pub fn version(&self) -> &str {
        match self {
            Self::Simple(version) => version,
            Self::Detailed(detail) => &detail.version,
        }
    }
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ToolDetail {
    pub version: String,
    #[serde(default = "default_required")]
    pub required: bool,
}

fn default_required() -> bool {
    true
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct EnvRequirement {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub secret: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub allowed: Vec<String>,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ServiceSpec {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub stop: Option<String>,
    #[serde(default)]
    pub healthcheck: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskSpec {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub run: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub safe_for_agent: bool,
    #[serde(default)]
    pub variants: Vec<TaskVariantSpec>,
}

impl TaskSpec {
    pub fn default_execution_kind(&self) -> Option<&'static str> {
        match (self.run.as_ref(), self.script.as_ref()) {
            (Some(_), None) => Some("run"),
            (None, Some(_)) => Some("script"),
            _ => None,
        }
    }

    pub fn default_execution_body(&self) -> Option<&str> {
        match (self.run.as_deref(), self.script.as_deref()) {
            (Some(run), None) => Some(run),
            (None, Some(script)) => Some(script),
            _ => None,
        }
    }

    pub fn resolved_execution(&self, os: &str) -> Option<TaskExecution<'_>> {
        self.variants
            .iter()
            .find(|variant| variant.when.matches(os))
            .and_then(TaskVariantSpec::execution)
            .or_else(|| {
                Some(TaskExecution {
                    kind: self.default_execution_kind()?,
                    body: self.default_execution_body()?,
                    os: None,
                })
            })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskVariantSpec {
    pub when: TaskWhen,
    #[serde(default)]
    pub run: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
}

impl TaskVariantSpec {
    pub fn execution_kind(&self) -> Option<&'static str> {
        match (self.run.as_ref(), self.script.as_ref()) {
            (Some(_), None) => Some("run"),
            (None, Some(_)) => Some("script"),
            _ => None,
        }
    }

    pub fn execution_body(&self) -> Option<&str> {
        match (self.run.as_deref(), self.script.as_deref()) {
            (Some(run), None) => Some(run),
            (None, Some(script)) => Some(script),
            _ => None,
        }
    }

    pub fn execution(&self) -> Option<TaskExecution<'_>> {
        Some(TaskExecution {
            kind: self.execution_kind()?,
            body: self.execution_body()?,
            os: self.when.os.as_deref(),
        })
    }
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TaskWhen {
    #[serde(default)]
    pub os: Option<String>,
}

impl TaskWhen {
    pub fn matches(&self, os: &str) -> bool {
        self.os.as_deref() == Some(os)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct TaskExecution<'a> {
    pub kind: &'static str,
    pub body: &'a str,
    pub os: Option<&'a str>,
}

#[derive(Debug, Serialize)]
pub struct TaskVariantView<'a> {
    pub os: &'a str,
    pub kind: &'a str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub run: Option<&'a str>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub script: Option<&'a str>,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckSpec {
    pub name: String,
    pub kind: CheckKind,
    pub severity: CheckSeverity,
    pub run: String,
    #[serde(default)]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum CheckKind {
    Precondition,
    Health,
}

#[derive(Debug, Clone, Copy, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Default, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    #[serde(default)]
    pub entrypoint: Option<String>,
    #[serde(default)]
    pub default_task: Option<String>,
    #[serde(default)]
    pub safe_tasks: Vec<String>,
    #[serde(default)]
    pub verify_after_changes: Vec<String>,
    #[serde(default)]
    pub writable_paths: Vec<String>,
    #[serde(default)]
    pub notes: Option<String>,
}
