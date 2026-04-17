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

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Contract {
    pub version: u32,
    pub project: Project,
    #[serde(default)]
    pub workspace: Option<RepoWorkspaceSpec>,
    #[serde(default)]
    pub execution: Option<Execution>,
    #[serde(default)]
    pub extensions: BTreeMap<String, ExtensionSpec>,
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
    pub exports: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub policies: BTreeMap<String, serde_yaml::Value>,
    #[serde(default)]
    pub metadata: BTreeMap<String, serde_yaml::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub agent: Option<AgentConfig>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: String,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(rename = "type", default)]
    pub project_type: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecutionBackends {
    #[serde(default)]
    pub container: Option<ContainerBackend>,
    #[serde(default)]
    pub remote: Option<RemoteBackend>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtensionSpec {
    pub kind: ExtensionKind,
    pub command: String,
    pub api_version: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub config: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExtensionKind {
    CheckProvider,
    ExportProvider,
    BackendProvider,
}

impl ExtensionKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::CheckProvider => "check_provider",
            Self::ExportProvider => "export_provider",
            Self::BackendProvider => "backend_provider",
        }
    }
}

impl std::fmt::Display for ExtensionKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str(self.as_str())
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ContainerBackend {
    pub image: String,
    #[serde(default)]
    pub engines: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum RuntimeRequirement {
    Simple(String),
    Detailed(RuntimeDetail),
}

impl RuntimeRequirement {
    pub fn active_for_os(&self, os: &str) -> bool {
        match self {
            Self::Simple(_) => true,
            Self::Detailed(detail) => detail
                .only_on
                .as_ref()
                .is_none_or(|platforms| platforms.iter().any(|platform| platform == os)),
        }
    }

    pub fn version(&self) -> &str {
        match self {
            Self::Simple(version) => version,
            Self::Detailed(detail) => &detail.version,
        }
    }

    pub fn version_for_os(&self, os: &str) -> &str {
        match self {
            Self::Simple(version) => version,
            Self::Detailed(detail) => detail
                .platforms
                .get(os)
                .and_then(|platform| platform.version.as_deref())
                .unwrap_or(&detail.version),
        }
    }

    pub fn required_for_os(&self, os: &str) -> bool {
        match self {
            Self::Simple(_) => true,
            Self::Detailed(detail) => self.active_for_os(os) && detail.required,
        }
    }

    pub fn provider_for_os(&self, os: &str) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed(detail) => detail
                .platforms
                .get(os)
                .and_then(|platform| platform.provider.as_deref())
                .or(detail.provider.as_deref()),
        }
    }

    pub fn distribution_for_os(&self, os: &str) -> Option<&str> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed(detail) => detail
                .platforms
                .get(os)
                .and_then(|platform| platform.distribution.as_deref())
                .or(detail.distribution.as_deref()),
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RuntimeDetail {
    pub version: String,
    #[serde(default = "default_required")]
    pub required: bool,
    #[serde(default)]
    pub only_on: Option<Vec<String>>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub distribution: Option<String>,
    #[serde(default)]
    pub platforms: BTreeMap<String, RuntimePlatformDetail>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RuntimePlatformDetail {
    #[serde(default)]
    pub version: Option<String>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub distribution: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(untagged)]
pub enum ToolRequirement {
    Simple(String),
    Detailed(ToolDetail),
}

impl ToolRequirement {
    pub fn active_for_os(&self, os: &str) -> bool {
        match self {
            Self::Simple(_) => true,
            Self::Detailed(detail) => detail
                .only_on
                .as_ref()
                .is_none_or(|platforms| platforms.iter().any(|platform| platform == os)),
        }
    }

    pub fn version(&self) -> &str {
        match self {
            Self::Simple(version) => version,
            Self::Detailed(detail) => &detail.version,
        }
    }

    pub fn version_for_os(&self, os: &str) -> &str {
        match self {
            Self::Simple(version) => version,
            Self::Detailed(detail) => detail
                .platforms
                .get(os)
                .and_then(|platform| platform.version.as_deref())
                .unwrap_or(&detail.version),
        }
    }

    pub fn required_for_os(&self, os: &str) -> bool {
        match self {
            Self::Simple(_) => true,
            Self::Detailed(detail) => self.active_for_os(os) && detail.required,
        }
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ToolDetail {
    pub version: String,
    #[serde(default = "default_required")]
    pub required: bool,
    #[serde(default)]
    pub only_on: Option<Vec<String>>,
    #[serde(default)]
    pub platforms: BTreeMap<String, ToolPlatformDetail>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ToolPlatformDetail {
    #[serde(default)]
    pub version: Option<String>,
}

fn default_required() -> bool {
    true
}

#[derive(Debug, Default, Deserialize, Clone)]
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prepend: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub append: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskSpec {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub notes: Option<String>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub inputs: BTreeMap<String, TaskInputSpec>,
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskInputSpec {
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub default: Option<String>,
    #[serde(default)]
    pub allowed: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
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

#[derive(Debug, Default, Deserialize, Clone)]
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

#[derive(Debug, Deserialize, Clone)]
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

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentBootstrapTargetConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub note: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub sh: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub powershell: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentBootstrapConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ota: Option<AgentBootstrapTargetConfig>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentConfig {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub entrypoint: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default_task: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub safe_tasks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub verify_after_changes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub writable_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protected_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<AgentBootstrapConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}
