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

use serde::de::Deserializer;
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
        self.runtimes.extend(other.runtimes.clone());
        self.tools.extend(other.tools.clone());
    }
}

impl Contract {
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
}

impl std::fmt::Display for EnvSourceKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Dotenv => f.write_str("dotenv"),
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
            .map(|readiness| readiness.from.as_str())
    }

    pub fn readiness_command(&self, service_name: &str) -> Option<String> {
        self.readiness
            .as_ref()
            .map(|readiness| readiness.run.clone())
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

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceEndpointSpec {
    pub address: String,
    pub port: u16,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
#[serde(deny_unknown_fields)]
pub struct ServiceReadinessSpec {
    pub from: String,
    pub run: String,
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

    pub fn mode_default_backend(&self) -> Option<Backend> {
        self.execution
            .as_ref()
            .and_then(|execution| execution.default_mode)
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
    pub runtime: Option<TaskRuntimeSpec>,
}

impl TaskModeBranchSpec {
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
            os: None,
        })
    }
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
    pub listeners: BTreeMap<String, TaskRuntimeListenerSpec>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskRuntimeKind {
    Service,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeReadinessSpec {
    pub kind: TaskRuntimeReadinessKind,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub listener: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskRuntimeReadinessKind {
    Http,
    Tcp,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskRuntimeListenerSpec {
    pub protocol: TaskRuntimeProtocol,
    pub bind: TaskRuntimeBindSpec,
    #[serde(default)]
    pub project: TaskRuntimeProjectionSpec,
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
    pub service: TaskTargetServiceRefSpec,
    #[serde(default)]
    pub override_input: Option<String>,
    #[serde(default)]
    pub activation: TaskTargetActivationSpec,
}

#[derive(Debug, Deserialize, Serialize, Clone, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct TaskTargetServiceRefSpec {
    pub task: String,
    pub listener: String,
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
    EnsureReady,
}

#[derive(Debug, Default, Deserialize, Serialize, Clone, Copy, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum TaskTargetAddressView {
    #[default]
    Topology,
    Host,
    Internal,
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

#[cfg(test)]
mod tests {
    use std::path::Path;

    use crate::parser::parse_contract_str;

    use super::Backend;

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
}
