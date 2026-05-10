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
use std::fmt;

use serde::de::{Deserializer, Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::ser::{SerializeMap, SerializeSeq, Serializer};
use serde::{Deserialize, Serialize};

const MEMORY_KIB: u64 = 1024;
const MEMORY_MIB: u64 = MEMORY_KIB * 1024;
const MEMORY_GIB: u64 = MEMORY_MIB * 1024;
const MEMORY_TIB: u64 = MEMORY_GIB * 1024;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MemorySizeParseError {
    Empty,
    MissingNumber,
    InvalidNumber,
    UnsupportedUnit { unit: String },
    Overflow,
}

impl std::fmt::Display for MemorySizeParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Empty => {
                f.write_str("memory size must not be empty (examples: `512MiB`, `2GiB`)")
            }
            Self::MissingNumber => {
                f.write_str("memory size must start with a positive integer amount")
            }
            Self::InvalidNumber => f.write_str("memory size amount must be a positive integer"),
            Self::UnsupportedUnit { unit } => write!(
                f,
                "unsupported memory size unit `{unit}` (supported units: `B`, `KiB`, `MiB`, `GiB`, `TiB`)"
            ),
            Self::Overflow => f.write_str("memory size is too large"),
        }
    }
}

pub fn parse_memory_size_bytes(value: &str) -> Result<u64, MemorySizeParseError> {
    let trimmed = value.trim();
    if trimmed.is_empty() {
        return Err(MemorySizeParseError::Empty);
    }

    let first_non_digit = trimmed
        .find(|character: char| !character.is_ascii_digit())
        .unwrap_or(trimmed.len());
    let (amount_text, unit_text) = trimmed.split_at(first_non_digit);
    if amount_text.is_empty() {
        return Err(MemorySizeParseError::MissingNumber);
    }
    let amount = amount_text
        .parse::<u64>()
        .map_err(|_| MemorySizeParseError::InvalidNumber)?;
    if amount == 0 {
        return Err(MemorySizeParseError::InvalidNumber);
    }

    let normalized_unit = unit_text.trim().to_ascii_lowercase();
    let multiplier = match normalized_unit.as_str() {
        "" | "b" => 1_u64,
        "kib" => MEMORY_KIB,
        "mib" => MEMORY_MIB,
        "gib" => MEMORY_GIB,
        "tib" => MEMORY_TIB,
        _ => {
            return Err(MemorySizeParseError::UnsupportedUnit {
                unit: unit_text.trim().to_string(),
            });
        }
    };

    amount
        .checked_mul(multiplier)
        .ok_or(MemorySizeParseError::Overflow)
}

pub fn format_memory_size_bytes(bytes: u64) -> String {
    if bytes % MEMORY_TIB == 0 {
        format!("{}TiB", bytes / MEMORY_TIB)
    } else if bytes % MEMORY_GIB == 0 {
        format!("{}GiB", bytes / MEMORY_GIB)
    } else if bytes % MEMORY_MIB == 0 {
        format!("{}MiB", bytes / MEMORY_MIB)
    } else if bytes % MEMORY_KIB == 0 {
        format!("{}KiB", bytes / MEMORY_KIB)
    } else {
        format!("{bytes}B")
    }
}

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
    pub env: EnvConfig,
    #[serde(default)]
    pub readiness: ContractReadinessConfig,
    #[serde(default)]
    pub surfaces: BTreeMap<String, SurfaceSpec>,
    #[serde(default)]
    pub services: BTreeMap<String, ServiceSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub workflows: Option<WorkflowCatalog>,
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

impl Contract {
    pub fn probe(&self, name: &str) -> Option<&ReadinessProbeSpec> {
        self.readiness.probes.get(name)
    }

    pub fn surface(&self, name: &str) -> Option<&SurfaceSpec> {
        self.surfaces.get(name)
    }

    pub fn workflow(&self, name: &str) -> Option<&WorkflowSpec> {
        self.workflows
            .as_ref()
            .and_then(|workflows| workflows.items.get(name))
    }

    pub fn default_workflow(&self) -> Option<(&str, &WorkflowSpec)> {
        let workflows = self.workflows.as_ref()?;
        workflows
            .items
            .get_key_value(workflows.default.as_str())
            .map(|(name, workflow)| (name.as_str(), workflow))
    }

    pub fn selected_workflow(&self, name: Option<&str>) -> Option<(&str, &WorkflowSpec)> {
        match name {
            Some(name) => self
                .workflows
                .as_ref()
                .and_then(|workflows| workflows.items.get_key_value(name))
                .map(|(name, workflow)| (name.as_str(), workflow)),
            None => self.default_workflow(),
        }
    }

    pub fn selected_setup_task_name(&self) -> Option<&str> {
        self.selected_setup_task_name_for(None)
    }

    pub fn selected_setup_task_name_for(&self, workflow_name: Option<&str>) -> Option<&str> {
        if let Some(name) = workflow_name {
            return self
                .workflow(name)
                .and_then(|workflow| workflow.setup.as_ref())
                .map(|phase| phase.task.as_str())
                .filter(|task| !task.trim().is_empty());
        }

        self.default_workflow()
            .and_then(|(_, workflow)| workflow.setup.as_ref())
            .map(|phase| phase.task.as_str())
            .filter(|task| !task.trim().is_empty())
            .or_else(|| self.tasks.contains_key("setup").then_some("setup"))
    }

    pub fn selected_run_task_name(&self) -> Option<&str> {
        self.selected_run_task_name_for(None)
    }

    pub fn selected_run_task_name_for(&self, workflow_name: Option<&str>) -> Option<&str> {
        let selected_workflow = self.selected_workflow(workflow_name);
        selected_workflow
            .and_then(|(_, workflow)| workflow.run.as_ref())
            .map(|phase| phase.task.as_str())
            .filter(|task| !task.trim().is_empty())
            .or_else(|| {
                selected_workflow.is_none().then(|| {
                    self.agent.as_ref().and_then(|agent| {
                        agent
                            .default_task
                            .as_deref()
                            .or(agent.entrypoint.as_deref())
                    })
                })?
            })
    }

    pub fn task_dependency_closure_names(
        &self,
        roots: impl IntoIterator<Item = String>,
    ) -> Vec<String> {
        let mut ordered = Vec::new();
        let mut visited = BTreeSet::new();
        for root in roots {
            self.collect_task_dependency_closure(root.as_str(), &mut visited, &mut ordered);
        }
        ordered
    }

    pub fn selected_workflow_task_closure_names(&self, workflow_name: Option<&str>) -> Vec<String> {
        let mut roots = Vec::new();
        if let Some(setup) = self.selected_setup_task_name_for(workflow_name) {
            roots.push(setup.to_string());
        }
        if let Some(run) = self.selected_run_task_name_for(workflow_name)
            && !roots.iter().any(|name| name == run)
        {
            roots.push(run.to_string());
        }
        self.task_dependency_closure_names(roots)
    }

    fn collect_task_dependency_closure(
        &self,
        name: &str,
        visited: &mut BTreeSet<String>,
        ordered: &mut Vec<String>,
    ) {
        if !visited.insert(name.to_string()) {
            return;
        }

        let Some(task) = self.tasks.get(name) else {
            return;
        };

        for dependency in &task.depends_on {
            self.collect_task_dependency_closure(dependency, visited, ordered);
        }

        ordered.push(name.to_string());
    }
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

#[derive(Debug, Default, Deserialize, Clone)]
pub struct WorkflowCatalog {
    pub default: String,
    #[serde(flatten)]
    pub items: BTreeMap<String, WorkflowSpec>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct WorkflowSpec {
    #[serde(default)]
    pub intent: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default)]
    pub setup: Option<WorkflowTaskRefSpec>,
    #[serde(default)]
    pub run: Option<WorkflowTaskRefSpec>,
    #[serde(default)]
    pub services: WorkflowServicesSpec,
    #[serde(default)]
    pub readiness: WorkflowReadinessSpec,
    #[serde(default)]
    pub exposes: Vec<WorkflowExposeSpec>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowTaskRefSpec {
    pub task: String,
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowServicesSpec {
    #[serde(default)]
    pub required: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct WorkflowReadinessSpec {
    #[serde(default)]
    pub checks: Vec<String>,
    #[serde(default)]
    pub probes: Vec<String>,
    #[serde(default)]
    pub surfaces: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(untagged)]
pub enum WorkflowExposeSpec {
    Url(String),
    SurfaceRef { surface: String },
}

impl WorkflowExposeSpec {
    pub fn surface_name(&self) -> Option<&str> {
        match self {
            Self::Url(_) => None,
            Self::SurfaceRef { surface } => Some(surface.as_str()),
        }
    }

    pub fn display_text(&self) -> String {
        match self {
            Self::Url(url) => url.clone(),
            Self::SurfaceRef { surface } => format!("surface:{surface}"),
        }
    }
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

#[derive(Debug, Clone)]
pub struct Execution {
    pub preferred: Option<Backend>,
    pub supported: Vec<Backend>,
    pub lifecycle: Option<Lifecycle>,
    pub backends: Option<ExecutionBackends>,
    pub default_context: Option<String>,
    pub contexts: BTreeMap<String, ExecutionContext>,
    pub shared_backends: BTreeMap<String, ExecutionSharedBackend>,
    context_resolution_errors: Vec<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecutionBackends {
    #[serde(default)]
    pub container: Option<ContainerBackend>,
    #[serde(default)]
    pub remote: Option<RemoteBackend>,
}

impl Execution {
    pub fn default_context(&self) -> Option<(&str, &ExecutionContext)> {
        let name = self.default_context.as_deref()?;
        self.contexts
            .get_key_value(name)
            .map(|(name, context)| (name.as_str(), context))
    }

    pub fn context_resolution_errors(&self) -> &[String] {
        &self.context_resolution_errors
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ExecutionWire {
    #[serde(default)]
    preferred: Option<Backend>,
    #[serde(default)]
    supported: Vec<Backend>,
    #[serde(default)]
    lifecycle: Option<Lifecycle>,
    #[serde(default)]
    backends: Option<ExecutionBackends>,
    #[serde(default)]
    default_context: Option<String>,
    #[serde(default)]
    contexts: BTreeMap<String, ExecutionContextWire>,
    #[serde(default)]
    shared_backends: BTreeMap<String, ExecutionSharedBackend>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ExecutionContextWire {
    #[serde(default)]
    extends: Option<String>,
    #[serde(default)]
    backend: Option<Backend>,
    #[serde(default)]
    lifecycle: Option<Lifecycle>,
    #[serde(default)]
    fulfillment: Option<ExecutionSharedBackendFulfillment>,
    #[serde(default)]
    env: Option<BTreeMap<String, String>>,
    #[serde(default)]
    container: Option<ContainerBackendWire>,
    #[serde(default)]
    remote: Option<RemoteBackendWire>,
    #[serde(default)]
    requirements: Option<ExecutionContextRequirementsWire>,
    #[serde(default)]
    attachments: Option<ExecutionContextAttachmentsWire>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ExecutionContextRequirementsWire {
    #[serde(default)]
    runtimes: BTreeMap<String, RuntimeRequirement>,
    #[serde(default)]
    tools: BTreeMap<String, ToolRequirement>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ExecutionContextAttachmentsWire {
    #[serde(default)]
    compose: Option<Vec<String>>,
    #[serde(default)]
    isolated_paths: Option<Vec<String>>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ContainerBackendWire {
    #[serde(default)]
    image: Option<String>,
    #[serde(default)]
    engines: Option<Vec<String>>,
    #[serde(default)]
    resources: Option<ContainerResourceSpecWire>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ContainerResourceSpecWire {
    #[serde(default)]
    memory: Option<ContainerMemoryResourceSpecWire>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct ContainerMemoryResourceSpecWire {
    #[serde(default)]
    minimum: Option<String>,
    #[serde(default)]
    default: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct RemoteBackendWire {
    #[serde(default)]
    provider: Option<String>,
    #[serde(default)]
    target: Option<String>,
    #[serde(default)]
    cwd: Option<String>,
    #[serde(default)]
    ssh: Option<RemoteSshOptionsWire>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
struct RemoteSshOptionsWire {
    #[serde(default)]
    config_file: Option<String>,
    #[serde(default)]
    identity_file: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct ExecutionContextMerged {
    backend: Option<Backend>,
    lifecycle: Option<Lifecycle>,
    fulfillment: Option<ExecutionSharedBackendFulfillment>,
    env: BTreeMap<String, String>,
    container: Option<ContainerBackendMerged>,
    remote: Option<RemoteBackendMerged>,
    requirements: ExecutionContextRequirements,
    attachments: ExecutionContextAttachments,
}

#[derive(Debug, Default, Clone)]
struct ContainerBackendMerged {
    image: Option<String>,
    engines: Option<Vec<String>>,
    resources: Option<ContainerResourceSpecMerged>,
}

#[derive(Debug, Default, Clone)]
struct ContainerResourceSpecMerged {
    memory: Option<ContainerMemoryResourceSpecMerged>,
}

#[derive(Debug, Default, Clone)]
struct ContainerMemoryResourceSpecMerged {
    minimum: Option<String>,
    default: Option<String>,
}

#[derive(Debug, Default, Clone)]
struct RemoteBackendMerged {
    provider: Option<String>,
    target: Option<String>,
    cwd: Option<String>,
    ssh: Option<RemoteSshOptionsMerged>,
}

#[derive(Debug, Default, Clone)]
struct RemoteSshOptionsMerged {
    config_file: Option<String>,
    identity_file: Option<String>,
}

impl<'de> Deserialize<'de> for Execution {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let wire = ExecutionWire::deserialize(deserializer)?;
        let (contexts, context_resolution_errors) = resolve_execution_contexts(&wire.contexts);
        Ok(Self {
            preferred: wire.preferred,
            supported: wire.supported,
            lifecycle: wire.lifecycle,
            backends: wire.backends,
            default_context: wire.default_context,
            contexts,
            shared_backends: wire.shared_backends,
            context_resolution_errors,
        })
    }
}

fn resolve_execution_contexts(
    contexts: &BTreeMap<String, ExecutionContextWire>,
) -> (BTreeMap<String, ExecutionContext>, Vec<String>) {
    let mut cache = BTreeMap::<String, ExecutionContextMerged>::new();
    let mut failed = BTreeSet::<String>::new();
    let mut stack = Vec::<String>::new();
    let mut errors = Vec::<String>::new();
    let mut resolved = BTreeMap::new();

    for name in contexts.keys() {
        let Some(merged) = resolve_execution_context(
            name,
            contexts,
            &mut cache,
            &mut failed,
            &mut stack,
            &mut errors,
        ) else {
            continue;
        };
        match finalize_execution_context(name, merged) {
            Ok(context) => {
                resolved.insert(name.clone(), context);
            }
            Err(error) => {
                failed.insert(name.clone());
                errors.push(error);
            }
        }
    }

    (resolved, errors)
}

fn resolve_execution_context(
    name: &str,
    contexts: &BTreeMap<String, ExecutionContextWire>,
    cache: &mut BTreeMap<String, ExecutionContextMerged>,
    failed: &mut BTreeSet<String>,
    stack: &mut Vec<String>,
    errors: &mut Vec<String>,
) -> Option<ExecutionContextMerged> {
    if let Some(cached) = cache.get(name) {
        return Some(cached.clone());
    }

    if failed.contains(name) {
        return None;
    }

    if stack.iter().any(|entry| entry == name) {
        let cycle_start = stack
            .iter()
            .position(|entry| entry == name)
            .unwrap_or_default();
        let mut cycle = stack[cycle_start..].to_vec();
        cycle.push(name.to_string());
        failed.insert(name.to_string());
        errors.push(format!(
            "`execution.contexts.{name}.extends` introduces an inheritance cycle: {}",
            cycle.join(" -> ")
        ));
        return None;
    }

    let Some(context) = contexts.get(name) else {
        failed.insert(name.to_string());
        errors.push(format!(
            "`execution.contexts.{name}` could not be resolved from declaration map"
        ));
        return None;
    };

    stack.push(name.to_string());
    let mut merged = if let Some(parent_name) = context.extends.as_deref() {
        let parent_name = parent_name.trim();
        if parent_name.is_empty() {
            let _ = stack.pop();
            failed.insert(name.to_string());
            errors.push(format!(
                "`execution.contexts.{name}.extends` must not be empty"
            ));
            return None;
        }
        if !contexts.contains_key(parent_name) {
            let _ = stack.pop();
            failed.insert(name.to_string());
            errors.push(format!(
                "`execution.contexts.{name}.extends` references unknown context `{parent_name}`"
            ));
            return None;
        }
        let Some(parent_merged) =
            resolve_execution_context(parent_name, contexts, cache, failed, stack, errors)
        else {
            let _ = stack.pop();
            failed.insert(name.to_string());
            return None;
        };
        if let (Some(parent_backend), Some(child_backend)) =
            (parent_merged.backend, context.backend)
            && parent_backend != child_backend
        {
            let _ = stack.pop();
            failed.insert(name.to_string());
            errors.push(format!(
                "`execution.contexts.{name}.backend` `{}` conflicts with inherited backend `{}` from `execution.contexts.{name}.extends`; backend-family overrides across `extends` are not supported",
                backend_label(child_backend),
                backend_label(parent_backend),
            ));
            return None;
        }
        parent_merged
    } else {
        ExecutionContextMerged::default()
    };
    merge_execution_context(&mut merged, context);
    let _ = stack.pop();

    cache.insert(name.to_string(), merged.clone());
    Some(merged)
}

fn merge_execution_context(target: &mut ExecutionContextMerged, source: &ExecutionContextWire) {
    if let Some(backend) = source.backend {
        target.backend = Some(backend);
    }
    if let Some(lifecycle) = source.lifecycle {
        target.lifecycle = Some(lifecycle);
    }
    if let Some(fulfillment) = source.fulfillment {
        target.fulfillment = Some(fulfillment);
    }
    if let Some(env) = source.env.as_ref() {
        target.env.extend(env.clone());
    }
    if let Some(container) = source.container.as_ref() {
        let merged = target
            .container
            .get_or_insert_with(ContainerBackendMerged::default);
        merge_container_backend(merged, container);
    }
    if let Some(remote) = source.remote.as_ref() {
        let merged = target
            .remote
            .get_or_insert_with(RemoteBackendMerged::default);
        merge_remote_backend(merged, remote);
    }
    if let Some(requirements) = source.requirements.as_ref() {
        target
            .requirements
            .runtimes
            .extend(requirements.runtimes.clone());
        target.requirements.tools.extend(requirements.tools.clone());
    }
    if let Some(attachments) = source.attachments.as_ref() {
        if let Some(compose) = attachments.compose.as_ref() {
            target.attachments.compose = compose.clone();
        }
        if let Some(isolated_paths) = attachments.isolated_paths.as_ref() {
            target.attachments.isolated_paths = isolated_paths.clone();
        }
    }
}

fn merge_container_backend(target: &mut ContainerBackendMerged, source: &ContainerBackendWire) {
    if let Some(image) = source.image.as_ref() {
        target.image = Some(image.clone());
    }
    if let Some(engines) = source.engines.as_ref() {
        target.engines = Some(engines.clone());
    }
    if let Some(resources) = source.resources.as_ref() {
        let merged = target
            .resources
            .get_or_insert_with(ContainerResourceSpecMerged::default);
        merge_container_resources(merged, resources);
    }
}

fn merge_container_resources(
    target: &mut ContainerResourceSpecMerged,
    source: &ContainerResourceSpecWire,
) {
    if let Some(memory) = source.memory.as_ref() {
        let merged = target
            .memory
            .get_or_insert_with(ContainerMemoryResourceSpecMerged::default);
        if let Some(minimum) = memory.minimum.as_ref() {
            merged.minimum = Some(minimum.clone());
        }
        if let Some(default) = memory.default.as_ref() {
            merged.default = Some(default.clone());
        }
    }
}

fn merge_remote_backend(target: &mut RemoteBackendMerged, source: &RemoteBackendWire) {
    if let Some(provider) = source.provider.as_ref() {
        target.provider = Some(provider.clone());
    }
    if let Some(target_name) = source.target.as_ref() {
        target.target = Some(target_name.clone());
    }
    if let Some(cwd) = source.cwd.as_ref() {
        target.cwd = Some(cwd.clone());
    }
    if let Some(ssh) = source.ssh.as_ref() {
        let merged = target
            .ssh
            .get_or_insert_with(RemoteSshOptionsMerged::default);
        if let Some(config_file) = ssh.config_file.as_ref() {
            merged.config_file = Some(config_file.clone());
        }
        if let Some(identity_file) = ssh.identity_file.as_ref() {
            merged.identity_file = Some(identity_file.clone());
        }
    }
}

fn finalize_execution_context(
    name: &str,
    merged: ExecutionContextMerged,
) -> Result<ExecutionContext, String> {
    let backend = merged.backend.ok_or_else(|| {
        format!(
            "`execution.contexts.{name}` does not resolve a backend; set `backend` directly or inherit it via `extends`"
        )
    })?;

    let container = merged.container.map(|container| ContainerBackend {
        image: container.image.unwrap_or_default(),
        engines: container.engines.unwrap_or_default(),
        resources: container.resources.map(|resources| ContainerResourceSpec {
            memory: resources.memory.map(|memory| ContainerMemoryResourceSpec {
                minimum: memory.minimum,
                default: memory.default,
            }),
        }),
    });
    let remote = merged.remote.map(|remote| RemoteBackend {
        provider: remote.provider.unwrap_or_default(),
        target: remote.target,
        cwd: remote.cwd,
        ssh: remote.ssh.map(|ssh| RemoteSshOptions {
            config_file: ssh.config_file,
            identity_file: ssh.identity_file,
        }),
    });

    Ok(ExecutionContext {
        backend,
        lifecycle: merged.lifecycle,
        fulfillment: merged.fulfillment,
        env: merged.env,
        container,
        remote,
        requirements: merged.requirements,
        attachments: merged.attachments,
    })
}

fn backend_label(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

#[derive(Debug, Default, Clone)]
pub struct RequirementSurface {
    pub runtimes: BTreeMap<String, RuntimeRequirement>,
    pub tools: BTreeMap<String, ToolRequirement>,
}

impl RequirementSurface {
    pub fn is_empty(&self) -> bool {
        self.runtimes.is_empty() && self.tools.is_empty()
    }

    pub fn merge(&mut self, other: &RequirementSurface) {
        for (name, requirement) in &other.runtimes {
            let merged = self
                .runtimes
                .get(name)
                .map(|base| base.merged_with_overlay(requirement))
                .unwrap_or_else(|| requirement.clone());
            self.runtimes.insert(name.clone(), merged);
        }
        for (name, requirement) in &other.tools {
            let merged = self
                .tools
                .get(name)
                .map(|base| base.merged_with_overlay(requirement))
                .unwrap_or_else(|| requirement.clone());
            self.tools.insert(name.clone(), merged);
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskRequirementsSpec {
    #[serde(default)]
    pub runtimes: BTreeMap<String, RuntimeRequirement>,
    #[serde(default)]
    pub tools: BTreeMap<String, ToolRequirement>,
    #[serde(default)]
    pub env: Vec<String>,
    #[serde(default)]
    pub checks: Vec<String>,
}

impl TaskRequirementsSpec {
    pub fn is_empty(&self) -> bool {
        self.runtimes.is_empty()
            && self.tools.is_empty()
            && self.env.is_empty()
            && self.checks.is_empty()
    }
}

impl Contract {
    pub fn selected_workflow_task_requirement_surface(
        &self,
        workflow_name: Option<&str>,
    ) -> Option<RequirementSurface> {
        self.task_requirement_surface(self.selected_workflow_task_closure_names(workflow_name))
    }

    pub fn task_requirement_surface(
        &self,
        task_names: impl IntoIterator<Item = String>,
    ) -> Option<RequirementSurface> {
        let mut surface = RequirementSurface::default();
        let mut saw_task = false;
        let mut scoped_runtimes = false;
        let mut scoped_tools = false;

        for task_name in task_names {
            let Some(task) = self.tasks.get(task_name.as_str()) else {
                continue;
            };
            saw_task = true;
            if !task.requirements.runtimes.is_empty() {
                scoped_runtimes = true;
            }
            if !task.requirements.tools.is_empty() {
                scoped_tools = true;
            }
            for (name, requirement) in &task.requirements.runtimes {
                surface.runtimes.insert(
                    name.clone(),
                    self.resolve_scoped_runtime_requirement(name, requirement),
                );
            }
            for (name, requirement) in &task.requirements.tools {
                surface.tools.insert(
                    name.clone(),
                    self.resolve_scoped_tool_requirement(name, requirement),
                );
            }
        }

        if !saw_task {
            return None;
        }

        if !scoped_runtimes {
            surface.runtimes = self.runtimes.clone();
        }
        if !scoped_tools {
            surface.tools = self.tools.clone();
        }

        Some(surface)
    }

    pub(crate) fn resolve_scoped_runtime_requirement(
        &self,
        name: &str,
        requirement: &RuntimeRequirement,
    ) -> RuntimeRequirement {
        self.runtimes
            .get(name)
            .map(|base| base.merged_with_overlay(requirement))
            .unwrap_or_else(|| requirement.clone())
    }

    pub(crate) fn resolve_scoped_tool_requirement(
        &self,
        name: &str,
        requirement: &ToolRequirement,
    ) -> ToolRequirement {
        self.tools
            .get(name)
            .map(|base| base.merged_with_overlay(requirement))
            .unwrap_or_else(|| requirement.clone())
    }

    pub fn all_requirement_surface(&self) -> RequirementSurface {
        let mut surface = RequirementSurface {
            runtimes: self.runtimes.clone(),
            tools: self.tools.clone(),
        };

        if let Some(execution) = self.execution.as_ref() {
            for context in execution.contexts.values() {
                surface
                    .runtimes
                    .extend(context.requirements.runtimes.clone());
                surface.tools.extend(context.requirements.tools.clone());
            }
        }

        surface
    }

    pub fn requirement_surface_for_backend(&self, backend: Backend) -> RequirementSurface {
        let mut surface = RequirementSurface {
            runtimes: self.runtimes.clone(),
            tools: self.tools.clone(),
        };
        surface.merge(&self.context_requirement_surface_for_backend(backend));
        surface
    }

    pub fn context_requirement_surface_for_backend(&self, backend: Backend) -> RequirementSurface {
        let mut surface = RequirementSurface::default();

        if let Some(execution) = self.execution.as_ref() {
            for context in execution.contexts.values() {
                if context.backend != backend {
                    continue;
                }

                surface
                    .runtimes
                    .extend(context.requirements.runtimes.clone());
                surface.tools.extend(context.requirements.tools.clone());
            }
        }

        surface
    }
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContext {
    pub backend: Backend,
    #[serde(default)]
    pub lifecycle: Option<Lifecycle>,
    #[serde(default)]
    pub fulfillment: Option<ExecutionSharedBackendFulfillment>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub container: Option<ContainerBackend>,
    #[serde(default)]
    pub remote: Option<RemoteBackend>,
    #[serde(default)]
    pub requirements: ExecutionContextRequirements,
    #[serde(default)]
    pub attachments: ExecutionContextAttachments,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSharedBackend {
    pub scope: ExecutionSharedBackendScope,
    pub backend: Backend,
    pub lifecycle: Lifecycle,
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub fulfillment: Option<ExecutionSharedBackendFulfillment>,
    #[serde(default)]
    pub environment: Option<ExecutionSharedBackendEnvironment>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSharedBackendScope {
    Local,
    Remote,
}

#[derive(Debug, Deserialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ExecutionSharedBackendFulfillment {
    None,
    Run,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExecutionSharedBackendEnvironment {
    #[serde(default)]
    pub profile: Option<String>,
    #[serde(default)]
    pub image_alias: Option<String>,
    #[serde(default)]
    pub image: Option<String>,
    #[serde(default)]
    pub source: Option<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContextRequirements {
    #[serde(default)]
    pub runtimes: BTreeMap<String, RuntimeRequirement>,
    #[serde(default)]
    pub tools: BTreeMap<String, ToolRequirement>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExecutionContextAttachments {
    #[serde(default)]
    pub compose: Vec<String>,
    #[serde(default)]
    pub isolated_paths: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ExtensionSpec {
    pub kind: ExtensionKind,
    pub command: String,
    pub api_version: u32,
    #[serde(default)]
    pub description: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub activation: Option<ExtensionActivationSpec>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub config: BTreeMap<String, serde_yaml::Value>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ExtensionActivationSpec {
    #[serde(default)]
    pub provider_managed_cleanup: bool,
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
    #[serde(default)]
    pub resources: Option<ContainerResourceSpec>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ContainerResourceSpec {
    #[serde(default)]
    pub memory: Option<ContainerMemoryResourceSpec>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ContainerMemoryResourceSpec {
    #[serde(default)]
    pub minimum: Option<String>,
    #[serde(default)]
    pub default: Option<String>,
}

#[derive(Debug, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct RemoteBackend {
    pub provider: String,
    #[serde(default)]
    pub target: Option<String>,
    #[serde(default)]
    pub cwd: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub ssh: Option<RemoteSshOptions>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct RemoteSshOptions {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub config_file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub identity_file: Option<String>,
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

    pub fn merged_with_overlay(&self, overlay: &RuntimeRequirement) -> RuntimeRequirement {
        match (self, overlay) {
            (Self::Detailed(base), Self::Simple(version)) => Self::Detailed(RuntimeDetail {
                version: version.clone(),
                required: base.required,
                only_on: base.only_on.clone(),
                provider: base.provider.clone(),
                distribution: base.distribution.clone(),
                platforms: base.platforms.clone(),
            }),
            (Self::Detailed(base), Self::Detailed(overlay)) => {
                let mut platforms = base.platforms.clone();
                platforms.extend(overlay.platforms.clone());
                Self::Detailed(RuntimeDetail {
                    version: overlay.version.clone(),
                    required: overlay.required,
                    only_on: overlay.only_on.clone().or_else(|| base.only_on.clone()),
                    provider: overlay.provider.clone().or_else(|| base.provider.clone()),
                    distribution: overlay
                        .distribution
                        .clone()
                        .or_else(|| base.distribution.clone()),
                    platforms,
                })
            }
            _ => overlay.clone(),
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

    pub fn acquisition(&self) -> Option<&ToolAcquisitionSpec> {
        match self {
            Self::Simple(_) => None,
            Self::Detailed(detail) => detail.acquisition.as_ref(),
        }
    }

    pub fn merged_with_overlay(&self, overlay: &ToolRequirement) -> ToolRequirement {
        match (self, overlay) {
            (Self::Detailed(base), Self::Simple(version)) => Self::Detailed(ToolDetail {
                version: version.clone(),
                required: base.required,
                only_on: base.only_on.clone(),
                platforms: base.platforms.clone(),
                acquisition: base.acquisition.clone(),
            }),
            (Self::Detailed(base), Self::Detailed(overlay)) => {
                let mut platforms = base.platforms.clone();
                platforms.extend(overlay.platforms.clone());
                Self::Detailed(ToolDetail {
                    version: overlay.version.clone(),
                    required: overlay.required,
                    only_on: overlay.only_on.clone().or_else(|| base.only_on.clone()),
                    platforms,
                    acquisition: overlay
                        .acquisition
                        .clone()
                        .or_else(|| base.acquisition.clone()),
                })
            }
            _ => overlay.clone(),
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
    #[serde(default)]
    pub acquisition: Option<ToolAcquisitionSpec>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ToolPlatformDetail {
    #[serde(default)]
    pub version: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ToolAcquisitionSpec {
    pub provider: ToolAcquisitionProvider,
    pub package: String,
    pub version: String,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ToolAcquisitionProvider {
    Corepack,
}

fn default_required() -> bool {
    true
}

fn is_false(value: &bool) -> bool {
    !*value
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EnvConfig {
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub vars: BTreeMap<String, EnvRequirement>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sources: Vec<EnvSource>,
}

impl EnvConfig {
    pub fn is_empty(&self) -> bool {
        self.vars.is_empty() && self.sources.is_empty()
    }

    pub fn len(&self) -> usize {
        self.vars.len() + self.sources.len()
    }

    pub fn contains_key(&self, name: &str) -> bool {
        self.vars.contains_key(name)
    }

    pub fn iter(&self) -> impl Iterator<Item = (&String, &EnvRequirement)> {
        self.vars.iter()
    }
}

impl<'a> IntoIterator for &'a EnvConfig {
    type Item = (&'a String, &'a EnvRequirement);
    type IntoIter = std::collections::btree_map::Iter<'a, String, EnvRequirement>;

    fn into_iter(self) -> Self::IntoIter {
        self.vars.iter()
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum EnvSourceKind {
    Dotenv,
    Properties,
    Json,
    Yaml,
    Toml,
}

impl std::fmt::Display for EnvSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dotenv => f.write_str("dotenv"),
            Self::Properties => f.write_str("properties"),
            Self::Json => f.write_str("json"),
            Self::Yaml => f.write_str("yaml"),
            Self::Toml => f.write_str("toml"),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EnvSource {
    pub kind: EnvSourceKind,
    pub path: String,
    #[serde(default, skip_serializing_if = "is_false")]
    pub must_exist: bool,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct EnvRequirement {
    #[serde(default, skip_serializing_if = "is_false")]
    pub required: bool,
    #[serde(default, skip_serializing_if = "is_false")]
    pub secret: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub default: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub prepend: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub append: Vec<String>,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ContractReadinessConfig {
    #[serde(default)]
    pub probes: BTreeMap<String, ReadinessProbeSpec>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SurfaceSpec {
    pub kind: SurfaceKind,
    pub port: u16,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub purpose: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub visibility: Option<SurfaceVisibility>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub readiness: Option<SurfaceReadinessSpec>,
}

impl SurfaceSpec {
    pub fn effective_path(&self) -> Option<String> {
        match self.kind {
            SurfaceKind::Http | SurfaceKind::Https => {
                Some(self.path.clone().unwrap_or_else(|| String::from("/")))
            }
            SurfaceKind::Tcp => None,
        }
    }

    pub fn normalized_listener(&self) -> TaskRuntimeListenerSpec {
        self.normalized_listener_with_attachment(&TaskRuntimeSurfaceAttachmentSpec::default())
    }

    pub fn normalized_listener_with_attachment(
        &self,
        attachment: &TaskRuntimeSurfaceAttachmentSpec,
    ) -> TaskRuntimeListenerSpec {
        let mut listener = TaskRuntimeListenerSpec {
            protocol: self.kind.as_runtime_protocol(),
            bind: TaskRuntimeBindSpec {
                address: String::from("127.0.0.1"),
                port: TaskRuntimePortSpec {
                    mode: TaskRuntimePortMode::Fixed,
                    value: Some(self.port),
                },
            },
            project: TaskRuntimeProjectionSpec {
                host: Some(TaskRuntimeHostProjectionSpec {
                    address: String::from("127.0.0.1"),
                    port: TaskRuntimeHostPortSpec {
                        mode: TaskRuntimeHostPortMode::Fixed,
                        value: Some(self.port),
                    },
                    primary: false,
                    path: self.effective_path(),
                }),
            },
        };

        if let Some(bind) = attachment.bind.as_ref() {
            if let Some(address) = bind.address.as_ref() {
                listener.bind.address = address.clone();
            }
            if let Some(port) = bind.port.as_ref() {
                listener.bind.port = port.clone();
            }
        }

        if let Some(project) = attachment.project.as_ref()
            && let Some(host_override) = project.host.as_ref()
            && let Some(host) = listener.project.host.as_mut()
        {
            if let Some(address) = host_override.address.as_ref() {
                host.address = address.clone();
            }
            if let Some(port) = host_override.port.as_ref() {
                host.port = port.clone();
            }
            if let Some(primary) = host_override.primary {
                host.primary = primary;
            }
            if let Some(path) = host_override.path.as_ref() {
                host.path = Some(path.clone());
            }
        }

        listener
    }

    pub fn derived_runtime_readiness(
        &self,
        listener_name: &str,
    ) -> Option<TaskRuntimeReadinessSpec> {
        let readiness = self.readiness.as_ref()?;
        Some(TaskRuntimeReadinessSpec {
            probe: None,
            kind: Some(readiness.kind),
            listener: Some(listener_name.to_string()),
            method: readiness.method,
            path: readiness.path.clone().or_else(|| match readiness.kind {
                TaskRuntimeReadinessKind::Http => self.effective_path(),
                TaskRuntimeReadinessKind::Tcp => None,
            }),
            headers: readiness.headers.clone(),
            success: readiness.success.clone(),
            body: readiness.body.clone(),
            interval: readiness.interval.clone(),
            timeout: readiness.timeout.clone(),
            retries: readiness.retries,
            start_period: readiness.start_period.clone(),
        })
    }

    pub fn host_url(&self) -> String {
        match self.kind {
            SurfaceKind::Http => format!(
                "http://127.0.0.1:{}{}",
                self.port,
                self.effective_path().unwrap_or_else(|| String::from("/"))
            ),
            SurfaceKind::Https => format!(
                "https://127.0.0.1:{}{}",
                self.port,
                self.effective_path().unwrap_or_else(|| String::from("/"))
            ),
            SurfaceKind::Tcp => format!("tcp://127.0.0.1:{}", self.port),
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeSurfaceAttachmentSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bind: Option<TaskRuntimeSurfaceBindOverrideSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub project: Option<TaskRuntimeSurfaceProjectionOverrideSpec>,
}

impl TaskRuntimeSurfaceAttachmentSpec {
    pub const fn uses_defaults(&self) -> bool {
        self.bind.is_none() && self.project.is_none()
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeSurfaceBindOverrideSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<TaskRuntimePortSpec>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeSurfaceProjectionOverrideSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub host: Option<TaskRuntimeSurfaceHostProjectionOverrideSpec>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeSurfaceHostProjectionOverrideSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub address: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub port: Option<TaskRuntimeHostPortSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub primary: Option<bool>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct TaskRuntimeSurfaceAttachments {
    entries: BTreeMap<String, TaskRuntimeSurfaceAttachmentSpec>,
    duplicate_names: Vec<String>,
}

impl TaskRuntimeSurfaceAttachments {
    pub fn iter(&self) -> impl Iterator<Item = (&String, &TaskRuntimeSurfaceAttachmentSpec)> {
        self.entries.iter()
    }

    pub fn names(&self) -> impl Iterator<Item = &String> {
        self.entries.keys()
    }

    pub fn names_cloned(&self) -> Vec<String> {
        self.entries.keys().cloned().collect()
    }

    pub fn get(&self, name: &str) -> Option<&TaskRuntimeSurfaceAttachmentSpec> {
        self.entries.get(name)
    }

    pub fn contains_name(&self, name: &str) -> bool {
        self.entries.contains_key(name)
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn duplicate_names(&self) -> &[String] {
        &self.duplicate_names
    }
}

impl Serialize for TaskRuntimeSurfaceAttachments {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if self
            .entries
            .values()
            .all(TaskRuntimeSurfaceAttachmentSpec::uses_defaults)
        {
            let mut seq = serializer.serialize_seq(Some(self.entries.len()))?;
            for name in self.entries.keys() {
                seq.serialize_element(name)?;
            }
            return seq.end();
        }

        let mut map = serializer.serialize_map(Some(self.entries.len()))?;
        for (name, attachment) in &self.entries {
            map.serialize_entry(name, attachment)?;
        }
        map.end()
    }
}

impl<'de> Deserialize<'de> for TaskRuntimeSurfaceAttachments {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        struct SurfaceAttachmentsVisitor;

        impl<'de> Visitor<'de> for SurfaceAttachmentsVisitor {
            type Value = TaskRuntimeSurfaceAttachments;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a surface attachment list or mapping")
            }

            fn visit_seq<A>(self, mut seq: A) -> Result<Self::Value, A::Error>
            where
                A: SeqAccess<'de>,
            {
                let mut entries = BTreeMap::new();
                let mut duplicate_names = Vec::new();
                while let Some(name) = seq.next_element::<String>()? {
                    if entries.contains_key(name.as_str()) {
                        duplicate_names.push(name);
                        continue;
                    }
                    entries.insert(name, TaskRuntimeSurfaceAttachmentSpec::default());
                }

                Ok(TaskRuntimeSurfaceAttachments {
                    entries,
                    duplicate_names,
                })
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut entries = BTreeMap::new();
                let mut duplicate_names = Vec::new();
                while let Some(name) = map.next_key::<String>()? {
                    let attachment = map.next_value::<TaskRuntimeSurfaceAttachmentSpec>()?;
                    if entries.insert(name.clone(), attachment).is_some() {
                        duplicate_names.push(name);
                    }
                }

                Ok(TaskRuntimeSurfaceAttachments {
                    entries,
                    duplicate_names,
                })
            }
        }

        deserializer.deserialize_any(SurfaceAttachmentsVisitor)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SurfaceKind {
    Http,
    Https,
    Tcp,
}

impl SurfaceKind {
    pub const fn as_runtime_protocol(self) -> TaskRuntimeProtocol {
        match self {
            Self::Http => TaskRuntimeProtocol::Http,
            Self::Https => TaskRuntimeProtocol::Https,
            Self::Tcp => TaskRuntimeProtocol::Tcp,
        }
    }

    pub const fn as_readiness_kind(self) -> TaskRuntimeReadinessKind {
        match self {
            Self::Http | Self::Https => TaskRuntimeReadinessKind::Http,
            Self::Tcp => TaskRuntimeReadinessKind::Tcp,
        }
    }

    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Https => "https",
            Self::Tcp => "tcp",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum SurfaceVisibility {
    Public,
    Internal,
}

impl SurfaceVisibility {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Public => "public",
            Self::Internal => "internal",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct SurfaceReadinessSpec {
    pub kind: TaskRuntimeReadinessKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<TaskRuntimeReadinessHttpMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<TaskRuntimeReadinessHttpSuccessSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<TaskRuntimeReadinessHttpBodySpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReadinessProbeSpec {
    pub kind: ReadinessProbeKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target: Option<ReadinessProbeTargetSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<TaskRuntimeReadinessHttpMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<TaskRuntimeReadinessHttpSuccessSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<TaskRuntimeReadinessHttpBodySpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expect_status: Option<u16>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<u64>,
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReadinessProbeKind {
    Http,
    Tcp,
}

#[derive(Debug, Deserialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReadinessProbeTargetSpec {
    pub kind: ReadinessProbeTargetKind,
    pub name: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listener: Option<String>,
    #[serde(default = "default_readiness_probe_target_address_view")]
    pub address_view: TaskTargetAddressView,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub observer: Option<ReadinessProbeObserverSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub endpoint: Option<String>,
}

const fn default_readiness_probe_target_address_view() -> TaskTargetAddressView {
    TaskTargetAddressView::Host
}

#[derive(Debug, Clone, Copy, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum ReadinessProbeTargetKind {
    Task,
    Service,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ReadinessProbeObserverSpec {
    #[serde(default)]
    pub kind: ReadinessProbeObserverKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub task: Option<String>,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ReadinessProbeObserverKind {
    #[default]
    CommandHost,
    Task,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceSpec {
    #[serde(default)]
    pub required: bool,
    #[serde(default)]
    pub manager: Option<ServiceManagerSpec>,
    #[serde(default)]
    pub provider: Option<String>,
    #[serde(default)]
    pub start: Option<String>,
    #[serde(default)]
    pub stop: Option<String>,
    #[serde(default)]
    pub producer: Option<ServiceProducerSpec>,
    #[serde(default)]
    pub endpoints: BTreeMap<String, ServiceEndpointSpec>,
    #[serde(default)]
    pub healthcheck: Option<String>,
    #[serde(default)]
    pub readiness: Option<ServiceReadinessSpec>,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub timeout: Option<u64>,
}

impl ServiceSpec {
    pub fn manager_label(&self) -> Option<String> {
        self.manager
            .as_ref()
            .map(ServiceManagerSpec::display_label)
            .or_else(|| self.provider.clone())
    }

    pub fn start_command(&self, service_name: &str) -> Option<String> {
        self.start.clone().or_else(|| {
            self.manager
                .as_ref()
                .and_then(|manager| manager.start_command(service_name))
        })
    }

    pub fn stop_command(&self, service_name: &str) -> Option<String> {
        self.stop.clone().or_else(|| {
            self.manager
                .as_ref()
                .and_then(|manager| manager.stop_command(service_name))
        })
    }

    pub fn healthcheck_command(&self, service_name: &str, healthcheck: &str) -> String {
        if let Some(manager) = &self.manager {
            manager.healthcheck_command(service_name, healthcheck)
        } else if self.provider.as_deref() == Some("docker-compose") {
            format!(
                "docker compose exec -T {service_name} sh -lc {}",
                shell_single_quote(healthcheck)
            )
        } else {
            healthcheck.to_string()
        }
    }

    pub fn readiness_context(&self) -> Option<&str> {
        self.readiness
            .as_ref()
            .and_then(ServiceReadinessSpec::from_context)
    }

    pub fn readiness_command(&self, service_name: &str) -> Option<String> {
        self.readiness
            .as_ref()
            .and_then(|readiness| readiness.legacy_run_command())
            .or_else(|| {
                self.healthcheck
                    .as_deref()
                    .map(|healthcheck| self.healthcheck_command(service_name, healthcheck))
            })
    }

    pub fn endpoint_for_context(&self, context_name: &str) -> Option<&ServiceEndpointSpec> {
        self.endpoints.get(context_name)
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct ServiceProducerSpec {
    pub repo: String,
    pub task: String,
    #[serde(default)]
    pub listener: Option<String>,
    #[serde(default = "default_service_producer_address_view")]
    pub address_view: TaskTargetAddressView,
}

impl Default for ServiceProducerSpec {
    fn default() -> Self {
        Self {
            repo: String::new(),
            task: String::new(),
            listener: None,
            address_view: default_service_producer_address_view(),
        }
    }
}

const fn default_service_producer_address_view() -> TaskTargetAddressView {
    TaskTargetAddressView::Host
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceEndpointSpec {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceReadinessSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub run: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<TaskRuntimeReadinessKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<TaskRuntimeReadinessHttpMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<TaskRuntimeReadinessHttpSuccessSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<TaskRuntimeReadinessHttpBodySpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,
}

impl ServiceReadinessSpec {
    pub fn from_context(&self) -> Option<&str> {
        self.from.as_deref()
    }

    pub fn legacy_run_command(&self) -> Option<String> {
        self.run.clone()
    }

    pub fn structured_kind(&self) -> Option<TaskRuntimeReadinessKind> {
        self.kind
    }
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceManagerSpec {
    pub kind: ServiceManagerKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub file: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub service: Option<String>,
}

impl ServiceManagerSpec {
    pub fn display_label(&self) -> String {
        match self.kind {
            ServiceManagerKind::Compose => match self.name.as_deref() {
                Some(name) if !name.trim().is_empty() => format!("compose ({name})"),
                _ => String::from("compose"),
            },
            ServiceManagerKind::Host => match self.name.as_deref() {
                Some(name) if !name.trim().is_empty() => format!("host ({name})"),
                _ => String::from("host"),
            },
        }
    }

    pub fn start_command(&self, service_name: &str) -> Option<String> {
        match self.kind {
            ServiceManagerKind::Compose => Some(format!(
                "{} up -d {}",
                self.compose_command_prefix(),
                shell_single_quote(self.compose_service(service_name))
            )),
            ServiceManagerKind::Host => None,
        }
    }

    pub fn stop_command(&self, service_name: &str) -> Option<String> {
        match self.kind {
            ServiceManagerKind::Compose => Some(format!(
                "{} stop {}",
                self.compose_command_prefix(),
                shell_single_quote(self.compose_service(service_name))
            )),
            ServiceManagerKind::Host => None,
        }
    }

    pub fn healthcheck_command(&self, service_name: &str, healthcheck: &str) -> String {
        match self.kind {
            ServiceManagerKind::Compose => format!(
                "{} exec -T {} sh -lc {}",
                self.compose_command_prefix(),
                shell_single_quote(self.compose_service(service_name)),
                shell_single_quote(healthcheck)
            ),
            ServiceManagerKind::Host => healthcheck.to_string(),
        }
    }

    fn compose_service<'a>(&'a self, service_name: &'a str) -> &'a str {
        self.service
            .as_deref()
            .filter(|service| !service.trim().is_empty())
            .unwrap_or(service_name)
    }

    fn compose_command_prefix(&self) -> String {
        let mut command = String::from("docker compose");
        if let Some(file) = self.file.as_deref().filter(|file| !file.trim().is_empty()) {
            command.push_str(" -f ");
            command.push_str(&shell_single_quote(file));
        }
        if let Some(name) = self.name.as_deref().filter(|name| !name.trim().is_empty()) {
            command.push_str(" -p ");
            command.push_str(&shell_single_quote(name));
        }
        command
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ServiceManagerKind {
    Compose,
    Host,
}

impl ServiceManagerKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Compose => "compose",
            Self::Host => "host",
        }
    }
}

fn shell_single_quote(input: &str) -> String {
    let escaped = input.replace('\'', r#"'\''"#);
    format!("'{escaped}'")
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
    pub context: Option<String>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub inputs: BTreeMap<String, TaskInputSpec>,
    #[serde(default)]
    pub targets: BTreeMap<String, TaskTargetSpec>,
    #[serde(default)]
    pub run: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub launch: Option<TaskLaunchSpec>,
    #[serde(default)]
    pub requirements: TaskRequirementsSpec,
    #[serde(default)]
    pub depends_on: Vec<String>,
    #[serde(default)]
    pub requires_services: Vec<String>,
    #[serde(default)]
    pub runtime: Option<TaskRuntimeSpec>,
    #[serde(default)]
    pub after_success: Vec<String>,
    #[serde(default)]
    pub after_failure: Vec<String>,
    #[serde(default)]
    pub after_always: Vec<String>,
    #[serde(default)]
    pub safe_for_agent: bool,
    #[serde(default)]
    pub internal: bool,
    #[serde(default)]
    pub variants: Vec<TaskVariantSpec>,
    #[serde(default)]
    pub execution: Option<TaskModeExecutionSpec>,
}

impl TaskSpec {
    pub fn default_execution_kind(&self) -> Option<&'static str> {
        match (
            self.run.as_ref(),
            self.script.as_ref(),
            self.launch.as_ref(),
        ) {
            (Some(_), None, None) => Some("run"),
            (None, Some(_), None) => Some("script"),
            (None, None, Some(launch)) => Some(launch.kind_str()),
            _ => None,
        }
    }

    pub fn default_execution_body(&self) -> Option<&str> {
        match (
            self.run.as_deref(),
            self.script.as_deref(),
            self.launch.as_ref(),
        ) {
            (Some(run), None, None) => Some(run),
            (None, Some(script), None) => Some(script),
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
                    body: self.default_execution_body(),
                    launch: self.launch.as_ref(),
                    os: None,
                })
            })
    }

    pub fn mode_default_backend(&self) -> Option<Backend> {
        self.execution
            .as_ref()
            .and_then(|execution| execution.default_mode)
    }

    pub fn workflow_backend(&self, execution: Option<&Execution>) -> Backend {
        if let Some(default_mode) = self.mode_default_backend() {
            return default_mode;
        }

        if let Some(context_name) = self.context.as_deref()
            && let Some(context) =
                execution.and_then(|execution| execution.contexts.get(context_name))
        {
            return context.backend;
        }

        if let Some((_, context)) = execution.and_then(Execution::default_context) {
            return context.backend;
        }

        if let Some(preferred) = execution.and_then(|execution| execution.preferred) {
            return preferred;
        }

        for backend in [Backend::Native, Backend::Container, Backend::Remote] {
            if self.mode_execution_branch(backend).is_some() {
                return backend;
            }
        }

        Backend::Native
    }

    pub fn mode_execution_branch(&self, backend: Backend) -> Option<&TaskModeBranchSpec> {
        self.execution
            .as_ref()
            .and_then(|execution| execution.modes.branch_for_backend(backend))
    }

    pub fn resolved_execution_for_backend(
        &self,
        backend: Backend,
        os: &str,
    ) -> Option<TaskExecution<'_>> {
        self.mode_execution_branch(backend)
            .and_then(TaskModeBranchSpec::execution)
            .or_else(|| self.resolved_execution(os))
    }

    pub fn runtime_for_backend(&self, backend: Backend) -> Option<&TaskRuntimeSpec> {
        self.mode_execution_branch(backend)
            .and_then(|branch| branch.runtime.as_ref())
            .or(self.runtime.as_ref())
    }

    pub fn backend_binding_for_backend(&self, backend: Backend) -> Option<&str> {
        self.runtime_for_backend(backend)
            .and_then(|runtime| runtime.backend_binding.as_deref())
            .map(str::trim)
            .filter(|binding| !binding.is_empty())
    }

    pub fn env_for_backend(
        &self,
        execution: Option<&Execution>,
        backend: Backend,
    ) -> BTreeMap<String, String> {
        self.env_for_backend_with_context_name(execution, backend, None)
    }

    pub fn env_for_backend_with_context_name(
        &self,
        execution: Option<&Execution>,
        backend: Backend,
        context_name_override: Option<&str>,
    ) -> BTreeMap<String, String> {
        let mut merged = context_name_override
            .and_then(|context_name| {
                execution
                    .and_then(|spec| spec.contexts.get(context_name))
                    .filter(|context| context.backend == backend)
            })
            .or_else(|| {
                self.context_for_backend(execution, backend)
                    .and_then(|context_name| {
                        execution.and_then(|spec| spec.contexts.get(context_name))
                    })
            })
            .map(|context| context.env.clone())
            .unwrap_or_default();
        merged.extend(self.env.clone());
        if let Some(branch) = self.mode_execution_branch(backend) {
            merged.extend(branch.env.clone());
        }
        merged
    }

    pub fn context_for_backend<'a>(
        &'a self,
        execution: Option<&'a Execution>,
        backend: Backend,
    ) -> Option<&'a str> {
        let execution = execution?;
        let branch_context = self
            .mode_execution_branch(backend)
            .and_then(|branch| branch.context.as_deref())
            .filter(|context_name| {
                execution
                    .contexts
                    .get(*context_name)
                    .is_some_and(|context| context.backend == backend)
            });

        if let Some(context_name) = branch_context {
            return Some(context_name);
        }

        if let Some(context_name) = self.context.as_deref() {
            if execution
                .contexts
                .get(context_name)
                .is_some_and(|context| context.backend == backend)
            {
                return Some(context_name);
            }
            if let Some((name, context)) = execution.default_context()
                && context.backend == backend
            {
                return Some(name);
            }
            return execution
                .contexts
                .iter()
                .find(|(_, context)| context.backend == backend)
                .map(|(name, _)| name.as_str());
        }

        if let Some((name, context)) = execution.default_context() {
            if context.backend == backend {
                return Some(name);
            }
            return execution
                .contexts
                .iter()
                .find(|(_, context)| context.backend == backend)
                .map(|(name, _)| name.as_str());
        }

        execution
            .contexts
            .iter()
            .find(|(_, context)| context.backend == backend)
            .map(|(name, _)| name.as_str())
    }

    pub fn service_runtime(&self) -> Option<&TaskRuntimeSpec> {
        self.runtime
            .as_ref()
            .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
    }

    pub fn service_runtime_for_backend(&self, backend: Backend) -> Option<&TaskRuntimeSpec> {
        self.runtime_for_backend(backend)
            .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
    }

    pub fn has_any_service_runtime(&self) -> bool {
        self.runtime
            .as_ref()
            .is_some_and(|runtime| runtime.kind == TaskRuntimeKind::Service)
            || self.execution.as_ref().is_some_and(|execution| {
                execution.modes.iter().any(|(_, branch)| {
                    branch
                        .runtime
                        .as_ref()
                        .is_some_and(|runtime| runtime.kind == TaskRuntimeKind::Service)
                })
            })
    }

    pub fn declared_surface_names(&self) -> BTreeSet<String> {
        let mut surfaces = BTreeSet::new();
        if let Some(runtime) = self
            .runtime
            .as_ref()
            .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
        {
            surfaces.extend(runtime.surfaces.names().cloned());
        }
        if let Some(execution) = self.execution.as_ref() {
            for (_, branch) in execution.modes.iter() {
                if let Some(runtime) = branch
                    .runtime
                    .as_ref()
                    .filter(|runtime| runtime.kind == TaskRuntimeKind::Service)
                {
                    surfaces.extend(runtime.surfaces.names().cloned());
                }
            }
        }
        surfaces
    }

    pub fn declares_surface(&self, name: &str) -> bool {
        self.declared_surface_names().contains(name)
    }

    pub fn scoped_requirement_surface(&self) -> RequirementSurface {
        RequirementSurface {
            runtimes: self.requirements.runtimes.clone(),
            tools: self.requirements.tools.clone(),
        }
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskModeExecutionSpec {
    #[serde(default)]
    pub default_mode: Option<Backend>,
    #[serde(default)]
    pub modes: TaskModeBranchesSpec,
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskModeBranchesSpec {
    #[serde(default)]
    pub native: Option<TaskModeBranchSpec>,
    #[serde(default)]
    pub container: Option<TaskModeBranchSpec>,
    #[serde(default)]
    pub remote: Option<TaskModeBranchSpec>,
}

impl TaskModeBranchesSpec {
    pub fn branch_for_backend(&self, backend: Backend) -> Option<&TaskModeBranchSpec> {
        match backend {
            Backend::Native => self.native.as_ref(),
            Backend::Container => self.container.as_ref(),
            Backend::Remote => self.remote.as_ref(),
        }
    }

    pub fn any(&self) -> bool {
        self.native.is_some() || self.container.is_some() || self.remote.is_some()
    }

    pub fn iter(&self) -> impl Iterator<Item = (Backend, &TaskModeBranchSpec)> {
        [
            (Backend::Native, self.native.as_ref()),
            (Backend::Container, self.container.as_ref()),
            (Backend::Remote, self.remote.as_ref()),
        ]
        .into_iter()
        .filter_map(|(backend, branch)| branch.map(|branch| (backend, branch)))
    }
}

#[derive(Debug, Default, Deserialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct TaskModeBranchSpec {
    #[serde(default)]
    pub context: Option<String>,
    #[serde(default)]
    pub lifecycle: Option<Lifecycle>,
    #[serde(default)]
    pub env: BTreeMap<String, String>,
    #[serde(default)]
    pub run: Option<String>,
    #[serde(default)]
    pub script: Option<String>,
    #[serde(default)]
    pub launch: Option<TaskLaunchSpec>,
    #[serde(default)]
    pub runtime: Option<TaskRuntimeSpec>,
}

impl TaskModeBranchSpec {
    pub fn execution_kind(&self) -> Option<&'static str> {
        match (
            self.run.as_ref(),
            self.script.as_ref(),
            self.launch.as_ref(),
        ) {
            (Some(_), None, None) => Some("run"),
            (None, Some(_), None) => Some("script"),
            (None, None, Some(launch)) => Some(launch.kind_str()),
            _ => None,
        }
    }

    pub fn execution_body(&self) -> Option<&str> {
        match (
            self.run.as_deref(),
            self.script.as_deref(),
            self.launch.as_ref(),
        ) {
            (Some(run), None, None) => Some(run),
            (None, Some(script), None) => Some(script),
            _ => None,
        }
    }

    pub fn execution(&self) -> Option<TaskExecution<'_>> {
        Some(TaskExecution {
            kind: self.execution_kind()?,
            body: self.execution_body(),
            launch: self.launch.as_ref(),
            os: None,
        })
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum TaskLaunchSpec {
    Command(TaskCommandLaunchSpec),
    Container(TaskContainerLaunchSpec),
}

impl TaskLaunchSpec {
    pub const fn kind_str(&self) -> &'static str {
        match self {
            Self::Command(_) => "command",
            Self::Container(_) => "container",
        }
    }

    pub fn preview(&self) -> String {
        match self {
            Self::Command(command) => {
                let mut preview = command.exe.clone();
                for arg in &command.args {
                    preview.push(' ');
                    preview.push_str(arg);
                }
                preview
            }
            Self::Container(container) => container.image.clone(),
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskCommandLaunchSpec {
    pub exe: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskContainerLaunchSpec {
    pub image: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub engine: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub args: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub remove: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub volumes: Vec<TaskContainerLaunchVolumeSpec>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TaskContainerLaunchVolumeKind {
    #[default]
    Named,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskContainerLaunchVolumeSpec {
    #[serde(
        default,
        skip_serializing_if = "is_default_task_container_launch_volume_kind"
    )]
    pub kind: TaskContainerLaunchVolumeKind,
    #[serde(alias = "name")]
    pub source: String,
    pub target: String,
}

fn is_default_task_container_launch_volume_kind(kind: &TaskContainerLaunchVolumeKind) -> bool {
    *kind == TaskContainerLaunchVolumeKind::Named
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeSpec {
    pub kind: TaskRuntimeKind,
    #[serde(default)]
    pub backend_binding: Option<String>,
    #[serde(default)]
    pub readiness: Option<TaskRuntimeReadinessSpec>,
    #[serde(default)]
    pub surfaces: TaskRuntimeSurfaceAttachments,
    #[serde(default)]
    pub listeners: BTreeMap<String, TaskRuntimeListenerSpec>,
    #[serde(skip, default)]
    pub normalized_surface_listeners: BTreeSet<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskRuntimeKind {
    Service,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeReadinessSpec {
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub probe: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub kind: Option<TaskRuntimeReadinessKind>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listener: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<TaskRuntimeReadinessHttpMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub headers: BTreeMap<String, String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub success: Option<TaskRuntimeReadinessHttpSuccessSpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub body: Option<TaskRuntimeReadinessHttpBodySpec>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub interval: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub timeout: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub retries: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub start_period: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskRuntimeReadinessKind {
    Http,
    Tcp,
}

impl TaskRuntimeReadinessKind {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Http => "http",
            Self::Tcp => "tcp",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "UPPERCASE")]
pub enum TaskRuntimeReadinessHttpMethod {
    Get,
    Head,
}

impl TaskRuntimeReadinessHttpMethod {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Get => "GET",
            Self::Head => "HEAD",
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeReadinessHttpSuccessSpec {
    pub status: Vec<u16>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeReadinessHttpBodySpec {
    pub contains: String,
}

pub(crate) fn parse_readiness_duration_spec(value: &str) -> Option<std::time::Duration> {
    let value = value.trim();
    if value.is_empty() {
        return None;
    }
    if let Some(number) = value.strip_suffix("ms") {
        return number
            .trim()
            .parse::<u64>()
            .ok()
            .map(std::time::Duration::from_millis);
    }
    if let Some(number) = value.strip_suffix('s') {
        return number
            .trim()
            .parse::<u64>()
            .ok()
            .map(std::time::Duration::from_secs);
    }
    if let Some(number) = value.strip_suffix('m') {
        return number
            .trim()
            .parse::<u64>()
            .ok()
            .and_then(|minutes| minutes.checked_mul(60))
            .map(std::time::Duration::from_secs);
    }
    if let Some(number) = value.strip_suffix('h') {
        return number
            .trim()
            .parse::<u64>()
            .ok()
            .and_then(|hours| hours.checked_mul(60 * 60))
            .map(std::time::Duration::from_secs);
    }
    None
}

#[derive(Debug, Serialize, Clone, PartialEq, Eq)]
pub struct TaskRuntimeListenerSpec {
    pub protocol: TaskRuntimeProtocol,
    pub bind: TaskRuntimeBindSpec,
    #[serde(default)]
    pub project: TaskRuntimeProjectionSpec,
}

impl<'de> Deserialize<'de> for TaskRuntimeListenerSpec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        enum Field {
            Protocol,
            Bind,
            Project,
            Http,
            Tcp,
        }

        impl<'de> Deserialize<'de> for Field {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: Deserializer<'de>,
            {
                struct FieldVisitor;

                impl<'de> Visitor<'de> for FieldVisitor {
                    type Value = Field;

                    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                        formatter.write_str("a listener field")
                    }

                    fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
                    where
                        E: DeError,
                    {
                        match value {
                            "protocol" => Ok(Field::Protocol),
                            "bind" => Ok(Field::Bind),
                            "project" => Ok(Field::Project),
                            "http" => Ok(Field::Http),
                            "tcp" => Ok(Field::Tcp),
                            _ => Err(E::unknown_field(
                                value,
                                &["protocol", "bind", "project", "http", "tcp"],
                            )),
                        }
                    }
                }

                deserializer.deserialize_identifier(FieldVisitor)
            }
        }

        struct ListenerVisitor;

        impl<'de> Visitor<'de> for ListenerVisitor {
            type Value = TaskRuntimeListenerSpec;

            fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
                formatter.write_str("a task runtime listener spec")
            }

            fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
            where
                A: MapAccess<'de>,
            {
                let mut protocol_present = false;
                let mut bind_present = false;
                let mut project_present = false;
                let mut protocol = None;
                let mut bind = None;
                let mut project = None;
                let mut http = None;
                let mut tcp = None;

                while let Some(field) = map.next_key()? {
                    match field {
                        Field::Protocol => {
                            if protocol_present {
                                return Err(A::Error::duplicate_field("protocol"));
                            }
                            protocol_present = true;
                            protocol = map.next_value::<Option<TaskRuntimeProtocol>>()?;
                        }
                        Field::Bind => {
                            if bind_present {
                                return Err(A::Error::duplicate_field("bind"));
                            }
                            bind_present = true;
                            bind = map.next_value::<Option<TaskRuntimeBindSpec>>()?;
                        }
                        Field::Project => {
                            if project_present {
                                return Err(A::Error::duplicate_field("project"));
                            }
                            project_present = true;
                            project = Some(map.next_value::<TaskRuntimeProjectionSpec>()?);
                        }
                        Field::Http => {
                            if http.is_some() {
                                return Err(A::Error::duplicate_field("http"));
                            }
                            http = Some(map.next_value::<u16>()?);
                        }
                        Field::Tcp => {
                            if tcp.is_some() {
                                return Err(A::Error::duplicate_field("tcp"));
                            }
                            tcp = Some(map.next_value::<u16>()?);
                        }
                    }
                }

                let shorthand_count = usize::from(http.is_some()) + usize::from(tcp.is_some());
                if shorthand_count > 1 {
                    return Err(A::Error::custom(
                        "listener shorthand must declare only one of `http` or `tcp`",
                    ));
                }
                if shorthand_count == 1 {
                    if protocol_present || bind_present || project_present {
                        return Err(A::Error::custom(
                            "listener shorthand cannot be combined with `protocol`, `bind`, or `project`",
                        ));
                    }
                    if let Some(port) = http {
                        return normalize_listener_shorthand(port, TaskRuntimeProtocol::Http)
                            .map_err(A::Error::custom);
                    }
                    if let Some(port) = tcp {
                        return normalize_listener_shorthand(port, TaskRuntimeProtocol::Tcp)
                            .map_err(A::Error::custom);
                    }
                }

                Ok(TaskRuntimeListenerSpec {
                    protocol: protocol.ok_or_else(|| A::Error::missing_field("protocol"))?,
                    bind: bind.ok_or_else(|| A::Error::missing_field("bind"))?,
                    project: project.unwrap_or_default(),
                })
            }
        }

        deserializer.deserialize_map(ListenerVisitor)
    }
}

fn normalize_listener_shorthand(
    port: u16,
    protocol: TaskRuntimeProtocol,
) -> Result<TaskRuntimeListenerSpec, String> {
    if port == 0 {
        return Err(String::from(
            "listener shorthand port must be between 1 and 65535",
        ));
    }
    Ok(TaskRuntimeListenerSpec {
        protocol,
        bind: TaskRuntimeBindSpec {
            address: String::from("127.0.0.1"),
            port: TaskRuntimePortSpec {
                mode: TaskRuntimePortMode::Fixed,
                value: Some(port),
            },
        },
        project: TaskRuntimeProjectionSpec {
            host: Some(TaskRuntimeHostProjectionSpec {
                address: String::from("127.0.0.1"),
                port: TaskRuntimeHostPortSpec {
                    mode: TaskRuntimeHostPortMode::Fixed,
                    value: Some(port),
                },
                primary: false,
                path: match protocol {
                    TaskRuntimeProtocol::Http => Some(String::from("/")),
                    TaskRuntimeProtocol::Https => Some(String::from("/")),
                    TaskRuntimeProtocol::Tcp => None,
                },
            }),
        },
    })
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskRuntimeProtocol {
    Http,
    Https,
    Tcp,
}

impl TaskRuntimeProtocol {
    pub const fn network_protocol(self) -> &'static str {
        "tcp"
    }

    pub const fn url_scheme(self) -> Option<&'static str> {
        match self {
            Self::Http => Some("http"),
            Self::Https => Some("https"),
            Self::Tcp => None,
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeBindSpec {
    pub address: String,
    pub port: TaskRuntimePortSpec,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimePortSpec {
    pub mode: TaskRuntimePortMode,
    #[serde(default)]
    pub value: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskRuntimePortMode {
    Fixed,
    Discover,
    Auto,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeProjectionSpec {
    #[serde(default)]
    pub host: Option<TaskRuntimeHostProjectionSpec>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeHostProjectionSpec {
    pub address: String,
    pub port: TaskRuntimeHostPortSpec,
    #[serde(default)]
    pub primary: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeHostPortSpec {
    pub mode: TaskRuntimeHostPortMode,
    #[serde(default)]
    pub value: Option<u16>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskRuntimeHostPortMode {
    Fixed,
    Auto,
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

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskTargetSpec {
    #[serde(default)]
    pub service: Option<TaskTargetServiceRefSpec>,
    #[serde(default)]
    pub url: Option<String>,
    #[serde(default)]
    pub override_input: Option<String>,
    #[serde(default)]
    pub activation: TaskTargetActivationSpec,
}

impl TaskTargetSpec {
    pub const fn kind(&self) -> TaskTargetKind {
        if self.service.is_some() {
            TaskTargetKind::Service
        } else {
            TaskTargetKind::Url
        }
    }
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskTargetServiceRefSpec {
    #[serde(default)]
    pub member: Option<String>,
    #[serde(default)]
    pub repo: Option<String>,
    pub task: String,
    #[serde(default)]
    pub listener: Option<String>,
    #[serde(default)]
    pub address_view: TaskTargetAddressView,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskTargetActivationSpec {
    #[serde(default)]
    pub mode: TaskTargetActivationMode,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskTargetActivationMode {
    #[default]
    Manual,
    EnsureStarted,
    RestartReady,
    EnsureReady,
    EnsureRunning,
}

impl TaskTargetActivationMode {
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Manual => "manual",
            Self::EnsureStarted => "ensure_started",
            Self::RestartReady => "restart_ready",
            Self::EnsureReady => "ensure_ready",
            Self::EnsureRunning => "ensure_running",
        }
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskTargetAddressView {
    #[default]
    Topology,
    Host,
    Internal,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskTargetKind {
    Service,
    Url,
}

pub(crate) fn task_target_env_name(name: &str) -> String {
    let mut env = String::from("OTA_TARGET_");
    for ch in name.chars() {
        if ch.is_ascii_alphanumeric() {
            env.push(ch.to_ascii_uppercase());
        } else {
            env.push('_');
        }
    }
    env
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
            body: self.execution_body(),
            launch: None,
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
    pub body: Option<&'a str>,
    pub launch: Option<&'a TaskLaunchSpec>,
    pub os: Option<&'a str>,
}

impl<'a> TaskExecution<'a> {
    pub fn shell_body(&self) -> Option<&'a str> {
        self.body
    }

    pub fn launch(&self) -> Option<&'a TaskLaunchSpec> {
        self.launch
    }

    pub fn preview(&self) -> String {
        self.body
            .map(ToOwned::to_owned)
            .or_else(|| self.launch.map(TaskLaunchSpec::preview))
            .unwrap_or_else(|| String::from("-"))
    }
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
    #[serde(default)]
    pub run: Option<String>,
    #[serde(default)]
    pub probe: Option<String>,
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
pub struct AgentBoundaryProvenanceConfig {
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub writable_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub protected_paths: Vec<String>,
}

impl AgentBoundaryProvenanceConfig {
    pub fn is_empty(&self) -> bool {
        self.writable_paths.is_empty() && self.protected_paths.is_empty()
    }
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct AgentInferredBoundaryConfig {
    #[serde(default)]
    pub reviewed: bool,
    #[serde(
        default,
        skip_serializing_if = "AgentBoundaryProvenanceConfig::is_empty"
    )]
    pub provenance: AgentBoundaryProvenanceConfig,
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
    pub inferred_boundary: Option<AgentInferredBoundaryConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub bootstrap: Option<AgentBootstrapConfig>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::parser::parse_contract_str;
    use crate::validator::validate_contract;

    use super::{Backend, TaskRuntimeHostPortMode, TaskRuntimePortMode, TaskRuntimeProtocol};

    #[test]
    fn task_env_for_backend_merges_context_task_and_mode_env_in_order() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: persistent
      env:
        FOO: context
        BAR: context
      container:
        image: node:24-bookworm
tasks:
  build:
    context: app
    env:
      BAR: task
      BAZ: task
    execution:
      default_mode: container
      modes:
        container:
          env:
            BAZ: mode
            QUX: mode
    run: npm run build
"#,
        )
        .unwrap();

        let env = contract.tasks["build"]
            .env_for_backend(contract.execution.as_ref(), Backend::Container);

        assert_eq!(env.get("FOO").map(String::as_str), Some("context"));
        assert_eq!(env.get("BAR").map(String::as_str), Some("task"));
        assert_eq!(env.get("BAZ").map(String::as_str), Some("mode"));
        assert_eq!(env.get("QUX").map(String::as_str), Some("mode"));
    }

    #[test]
    fn task_env_for_backend_uses_backend_matching_context_env_when_task_context_differs() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
      env:
        FOO: host
    app:
      backend: container
      lifecycle: persistent
      env:
        FOO: container
      container:
        image: node:24-bookworm
tasks:
  build:
    context: host
    execution:
      default_mode: container
      modes:
        container:
          run: npm run build
    run: npm run build
"#,
        )
        .unwrap();

        let env = contract.tasks["build"]
            .env_for_backend(contract.execution.as_ref(), Backend::Container);

        assert_eq!(env.get("FOO").map(String::as_str), Some("container"));
    }

    #[test]
    fn selected_run_task_name_for_does_not_fall_back_to_agent_for_selected_workflow() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  agent-dev:
    run: echo dev
workflows:
  default: app
  app:
    intent: local_development
agent:
  default_task: agent-dev
"#,
        )
        .unwrap();

        assert_eq!(contract.selected_run_task_name_for(Some("app")), None);
        assert_eq!(contract.selected_run_task_name_for(None), None);
        assert_eq!(contract.selected_run_task_name(), None);
    }

    #[test]
    fn selected_run_task_name_without_workflows_can_fall_back_to_agent() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  agent-dev:
    run: echo dev
agent:
  default_task: agent-dev
"#,
        )
        .unwrap();

        assert_eq!(contract.selected_run_task_name_for(None), Some("agent-dev"));
    }

    #[test]
    fn selected_setup_task_respects_explicit_workflow_without_setup() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
  quickstart:
    run: npx ota
workflows:
  default: app
  app:
    setup:
      task: setup
    run:
      task: quickstart
  instant:
    run:
      task: quickstart
"#,
        )
        .unwrap();

        assert_eq!(contract.selected_setup_task_name_for(None), Some("setup"));
        assert_eq!(
            contract.selected_setup_task_name_for(Some("app")),
            Some("setup")
        );
        assert_eq!(contract.selected_setup_task_name_for(Some("instant")), None);
        assert_eq!(
            contract.selected_workflow_task_closure_names(Some("instant")),
            vec![String::from("quickstart")]
        );
    }

    #[test]
    fn selected_setup_task_uses_legacy_setup_fallback_without_workflows() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
"#,
        )
        .unwrap();

        assert_eq!(contract.selected_setup_task_name_for(None), Some("setup"));
        assert_eq!(
            contract.selected_workflow_task_closure_names(None),
            vec![String::from("setup")]
        );
    }

    #[test]
    fn listener_http_shorthand_normalizes_to_verbose_shape() {
        let shorthand = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          http: 5678
"#,
        )
        .unwrap();
        let verbose = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 5678
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 5678
              path: /
"#,
        )
        .unwrap();

        let shorthand_listener = &shorthand.tasks["dev"]
            .runtime
            .as_ref()
            .expect("runtime should exist")
            .listeners["backend"];
        let verbose_listener = &verbose.tasks["dev"]
            .runtime
            .as_ref()
            .expect("runtime should exist")
            .listeners["backend"];

        assert_eq!(shorthand_listener, verbose_listener);
        assert_eq!(shorthand_listener.protocol, TaskRuntimeProtocol::Http);
        assert_eq!(
            shorthand_listener
                .project
                .host
                .as_ref()
                .and_then(|host| host.path.as_deref()),
            Some("/")
        );
    }

    #[test]
    fn listener_tcp_shorthand_normalizes_to_verbose_shape() {
        let shorthand = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        redis:
          tcp: 6379
"#,
        )
        .unwrap();
        let listener = &shorthand.tasks["dev"]
            .runtime
            .as_ref()
            .expect("runtime should exist")
            .listeners["redis"];

        assert_eq!(listener.protocol, TaskRuntimeProtocol::Tcp);
        assert_eq!(listener.bind.address, "127.0.0.1");
        assert_eq!(listener.bind.port.value, Some(6379));
        assert_eq!(
            listener
                .project
                .host
                .as_ref()
                .and_then(|host| host.port.value),
            Some(6379)
        );
        assert_eq!(
            listener
                .project
                .host
                .as_ref()
                .and_then(|host| host.path.as_deref()),
            None
        );
    }

    #[test]
    fn listener_shorthand_rejects_mixed_verbose_fields() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          http: 5678
          protocol: http
"#,
        )
        .expect_err("mixed shorthand and verbose fields should fail");

        assert!(error.to_string().contains(
            "listener shorthand cannot be combined with `protocol`, `bind`, or `project`"
        ));
    }

    #[test]
    fn listener_shorthand_rejects_empty_project_field_presence() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          http: 5678
          project: {}
"#,
        )
        .expect_err("empty project field should still count as mixed shorthand");

        assert!(error.to_string().contains(
            "listener shorthand cannot be combined with `protocol`, `bind`, or `project`"
        ));
    }

    #[test]
    fn listener_shorthand_rejects_null_protocol_field_presence() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          http: 5678
          protocol: null
"#,
        )
        .expect_err("null protocol field should still count as mixed shorthand");

        assert!(error.to_string().contains(
            "listener shorthand cannot be combined with `protocol`, `bind`, or `project`"
        ));
    }

    #[test]
    fn listener_shorthand_rejects_multiple_protocol_keys() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          http: 5678
          tcp: 5678
"#,
        )
        .expect_err("multiple shorthand protocol keys should fail");

        assert!(
            error
                .to_string()
                .contains("listener shorthand must declare only one of `http` or `tcp`")
        );
    }

    #[test]
    fn listener_shorthand_rejects_port_zero() {
        let error = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      listeners:
        backend:
          http: 0
"#,
        )
        .expect_err("port zero should fail");

        assert!(
            error
                .to_string()
                .contains("listener shorthand port must be between 1 and 65535")
        );
    }

    #[test]
    fn attached_surface_normalizes_to_runtime_listener_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
    path: /
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - backend
"#,
        )
        .unwrap();
        let listener = &contract.tasks["dev"]
            .runtime
            .as_ref()
            .expect("runtime should exist")
            .listeners["backend"];

        assert_eq!(listener.protocol, TaskRuntimeProtocol::Http);
        assert_eq!(listener.bind.address, "127.0.0.1");
        assert_eq!(listener.bind.port.value, Some(5678));
        assert_eq!(
            listener
                .project
                .host
                .as_ref()
                .expect("host projection should exist")
                .path
                .as_deref(),
            Some("/")
        );
    }

    #[test]
    fn attached_surface_unknown_reference_fails_validation() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - backend
"#,
        )
        .expect("unknown surface attachment should still parse structurally");
        let error = validate_contract(&contract)
            .expect_err("unknown surface attachment should fail validation");

        assert!(
            error
                .to_string()
                .contains("`tasks.dev.runtime.surfaces` references unknown surface `backend`")
        );
    }

    #[test]
    fn attached_surface_duplicate_name_fails_validation() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - backend
        - backend
"#,
        )
        .expect("duplicate surface attachment should still parse structurally");
        let error = validate_contract(&contract)
            .expect_err("duplicate surface attachment should fail validation");

        assert!(
            error.to_string().contains(
                "`tasks.dev.runtime.surfaces` must not declare duplicate surface `backend`"
            )
        );
    }

    #[test]
    fn attached_surface_listener_name_collision_fails_validation() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        - backend
      listeners:
        backend:
          http: 5678
"#,
        )
        .expect("surface/listener collision should still parse structurally");
        let error = validate_contract(&contract)
            .expect_err("surface/listener collision should fail validation");

        assert!(
            error.to_string().contains(
                "`tasks.dev.runtime.surfaces` attaches surface `backend`, but `tasks.dev.runtime.listeners.backend` is already declared"
            )
        );
    }

    #[test]
    fn attached_surface_object_form_normalizes_publication_override() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  site:
    kind: http
    port: 3000
tasks:
  dev:
    run: npm run dev
    runtime:
      kind: service
      surfaces:
        site:
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: auto
              path: /
              primary: true
"#,
        )
        .expect("surface attachment override should parse");

        let listener = &contract.tasks["dev"]
            .runtime
            .as_ref()
            .expect("runtime should exist")
            .listeners["site"];

        assert_eq!(listener.protocol, TaskRuntimeProtocol::Http);
        assert_eq!(listener.bind.address, "0.0.0.0");
        assert_eq!(listener.bind.port.mode, TaskRuntimePortMode::Fixed);
        assert_eq!(listener.bind.port.value, Some(3000));
        assert_eq!(
            listener
                .project
                .host
                .as_ref()
                .map(|host| host.address.as_str()),
            Some("127.0.0.1")
        );
        assert_eq!(
            listener.project.host.as_ref().map(|host| host.port.mode),
            Some(TaskRuntimeHostPortMode::Auto)
        );
        assert_eq!(
            listener
                .project
                .host
                .as_ref()
                .and_then(|host| host.path.as_deref()),
            Some("/")
        );
        assert_eq!(
            listener.project.host.as_ref().map(|host| host.primary),
            Some(true)
        );
    }

    #[test]
    fn attached_surface_bind_port_must_preserve_declared_surface_port() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        backend:
          bind:
            port:
              mode: fixed
              value: 4000
"#,
        )
        .expect("surface attachment override should still parse structurally");
        let error = validate_contract(&contract)
            .expect_err("surface bind port override should fail validation");

        assert!(error.to_string().contains(
            "`tasks.dev.runtime.surfaces.backend.bind.port` must preserve declared surface port 5678 with `mode: fixed`"
        ));
    }

    #[test]
    fn attached_primary_surface_derives_runtime_readiness_for_multi_surface_runtime() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 5678
    readiness:
      kind: http
      path: /healthz/readiness
  editor:
    kind: http
    port: 8080
    readiness:
      kind: http
      path: /
tasks:
  dev:
    run: pnpm dev
    runtime:
      kind: service
      surfaces:
        backend:
          project:
            host:
              primary: true
        editor: {}
"#,
        )
        .expect("surface attachments should parse");

        let readiness = contract.tasks["dev"]
            .runtime
            .as_ref()
            .and_then(|runtime| runtime.readiness.as_ref())
            .expect("primary attached surface should derive runtime readiness");

        assert_eq!(readiness.listener.as_deref(), Some("backend"));
        assert_eq!(readiness.path.as_deref(), Some("/healthz/readiness"));
    }
}
