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
use std::collections::BTreeSet;
use std::ffi::OsStr;
use std::fs;
use std::io::{self, IsTerminal, Read, Write};
use std::net::TcpStream;
use std::path::Component;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use semver::{Op, VersionReq};
use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value as JsonValue;

use crate::execution::{
    container_backend_probe_failure, container_engine_candidates,
    container_engine_candidates_from_backend, matching_declared_execution_context_name,
    preferred_container_backend_probe_failure, selected_container_engine,
    selected_container_engine_from_backend,
};
use crate::policy_pack::{
    LoadPolicyPackError, LoadedOrgPolicyPack, ProvisioningAction, ProvisioningBackendRequest,
    ProvisioningPlan, ProvisioningTargetKind, load_org_policy_pack_auto_details,
};
use crate::provisioning::{
    ProvisioningBackendError, ProvisioningExecutionTarget, ProvisioningFailureDiagnosis,
    ProvisioningFailureKind, probe_provisioning_installability_with_target,
    render_provisioning_action_command,
};
use crate::runner::{
    DeclaredEnvSourceStatus, ExecutionOverrides, HttpReadinessRequest, HttpReadinessStatus,
    LoadedDeclaredEnvSource, ResolvedExecutionBackend, ResolvedNamedReadinessProbe,
    ResolvedNamedReadinessProbeContract, RunError, capture_declared_native_activation_env,
    combine_readiness_probe_paths, effective_execution, effective_task_execution,
    evaluate_declared_env_check, host_runtime_readiness_observed, http_readiness_endpoint_status,
    load_declared_env_sources, parse_http_probe_url, resolve_context_execution_backend,
    resolve_declared_env_source_value, resolve_named_readiness_probe,
    resolve_named_readiness_probe_contract, resolve_task_target_binding_url_with_contract_path,
    run_backend_argv_command_captured, run_backend_command_captured,
    task_runtime_host_readiness_probe_for_backend, task_surface_host_readiness_probe_for_backend,
};
use crate::schema::{
    Backend, CheckKind, CheckSeverity, ContainerBackend, Contract, ExtensionKind, Lifecycle,
    NativePrerequisiteActivationShell, ReadinessProbeSpec, RequirementSurface, RuntimeRequirement,
    ServiceProducerSpec, ServiceReadinessSpec, ServiceSpec, TaskNetworkEffectKind,
    ToolAcquisitionProvider, ToolAcquisitionSpec, ToolRequirement, ToolchainFulfillmentSource,
};
use crate::terminal::supports_dynamic_stderr_ui;
use crate::toolchains::{
    ToolchainManagedSurfaceKind, ToolchainOpportunityContext, declared_toolchain_contract,
    declared_toolchain_source_label,
    requirement_surface_with_toolchain_owned_capabilities_for_required_tools,
    requirement_surface_with_toolchain_owned_tools_for_required_tools,
    shipped_toolchain_contract_by_label, tool_versions_entry, toolchain_fulfillment_source_label,
    toolchain_repo_signals, unsupported_toolchain_opportunity_context,
    unsupported_toolchain_opportunity_ecosystems,
};
use crate::validator::{
    ContractAdvisory, TaskExecutionBoundary, collect_contract_advisories_with_contract_path,
};
use crate::workspace::load_contract_for_workspace_repo_ref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub identity: Option<FindingIdentity>,
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Deserialize)]
pub struct FindingIdentity {
    pub code: String,
    pub category: String,
    pub owner: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum DoctorMode {
    Native,
    Container,
    Remote,
}

impl DoctorMode {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            DoctorMode::Native => "native",
            DoctorMode::Container => "container",
            DoctorMode::Remote => "remote",
        }
    }
}

fn backend_for_mode(mode: DoctorMode) -> Backend {
    match mode {
        DoctorMode::Native => Backend::Native,
        DoctorMode::Container => Backend::Container,
        DoctorMode::Remote => Backend::Remote,
    }
}

fn doctor_selected_lifecycle(
    mode: DoctorMode,
    lifecycle_override: Option<Lifecycle>,
) -> Option<Lifecycle> {
    lifecycle_override.or(match mode {
        DoctorMode::Container => Some(Lifecycle::Ephemeral),
        DoctorMode::Native | DoctorMode::Remote => None,
    })
}

fn doctor_command_string(mode: DoctorMode, lifecycle: Option<Lifecycle>) -> String {
    let mut command = String::from("ota doctor");
    match mode {
        DoctorMode::Container => command.push_str(" --mode container"),
        DoctorMode::Remote => command.push_str(" --mode remote"),
        DoctorMode::Native => {}
    }
    if let Some(lifecycle) = lifecycle {
        command.push_str(" --lifecycle ");
        command.push_str(match lifecycle {
            Lifecycle::Persistent => "persistent",
            Lifecycle::Ephemeral => "ephemeral",
        });
    }
    command
}

fn dedupe_findings_preserve_order(findings: &mut Vec<Finding>) {
    let mut deduped = Vec::with_capacity(findings.len());
    for finding in findings.drain(..) {
        if !deduped.contains(&finding) {
            deduped.push(finding);
        }
    }
    *findings = deduped;
}

fn rerun_doctor_command(mode: DoctorMode, lifecycle_override: Option<Lifecycle>) -> String {
    doctor_command_string(mode, doctor_selected_lifecycle(mode, lifecycle_override))
}

fn rerun_doctor_command_for_workflow(
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
    workflow_name: Option<&str>,
) -> String {
    let mut command = doctor_command_string(mode, lifecycle);
    if let Some(workflow_name) = workflow_name {
        command.push_str(" --workflow ");
        command.push_str(workflow_name);
    }
    command
}

fn doctor_mode_for_service(contract: &Contract, service: &ServiceSpec) -> DoctorMode {
    let Some(readiness) = service.readiness.as_ref() else {
        return DoctorMode::Native;
    };
    let Some(from_context) = readiness.from_context() else {
        return DoctorMode::Native;
    };

    let backend = contract
        .execution
        .as_ref()
        .and_then(|execution| execution.contexts.get(from_context))
        .map(|context| context.backend)
        .unwrap_or(Backend::Native);

    match backend {
        Backend::Native => DoctorMode::Native,
        Backend::Container => DoctorMode::Container,
        Backend::Remote => DoctorMode::Remote,
    }
}

fn current_host_platform() -> &'static str {
    match std::env::consts::OS {
        "macos" => "macos",
        "windows" => "windows",
        other => other,
    }
}

fn backend_key(backend: Backend) -> &'static str {
    match backend {
        Backend::Native => "native",
        Backend::Container => "container",
        Backend::Remote => "remote",
    }
}

fn current_host_arch() -> &'static str {
    match std::env::consts::ARCH {
        "x86_64" | "amd64" => "x64",
        "aarch64" | "arm64" => "arm64",
        other => other,
    }
}

fn selected_context_names_for_mode(
    contract: &Contract,
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
    workflow_name: Option<&str>,
    overrides: ExecutionOverrides,
) -> BTreeSet<String> {
    let backend = backend_for_mode(mode);
    let task_names = contract.selected_workflow_task_closure_names(workflow_name);
    let mut context_names = BTreeSet::new();

    if task_names.is_empty() {
        if let Some(context_name) = matching_declared_execution_context_name(
            contract.execution.as_ref(),
            backend,
            lifecycle,
        ) {
            context_names.insert(context_name.to_string());
        }
        return context_names;
    }

    for task_name in task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        if effective_task_execution(contract, task_name.as_str(), overrides).backend != backend {
            continue;
        }
        if let Some(context_name) = task.context_for_backend(contract.execution.as_ref(), backend) {
            context_names.insert(context_name.to_string());
        }
    }

    context_names
}

fn unsupported_host_context_findings(
    contract: &Contract,
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
    workflow_name: Option<&str>,
    overrides: ExecutionOverrides,
) -> Vec<Finding> {
    let current_os = current_host_platform();
    let current_arch = current_host_arch();
    let mut findings = Vec::new();

    for context_name in
        selected_context_names_for_mode(contract, mode, lifecycle, workflow_name, overrides)
    {
        let Some((_, context)) = contract
            .execution
            .as_ref()
            .and_then(|execution| execution.contexts.get_key_value(context_name.as_str()))
        else {
            continue;
        };
        if context.active_for_host(current_os, current_arch) {
            continue;
        }

        let supported_os = context
            .only_on
            .as_ref()
            .map(|platforms| platforms.join(", "))
            .unwrap_or_else(|| String::from("all hosts"));
        let supported_arch = context
            .only_arch
            .as_ref()
            .map(|architectures| architectures.join(", "))
            .unwrap_or_else(|| String::from("all architectures"));
        let next = if current_os == "windows"
            && context
                .only_on
                .as_ref()
                .is_some_and(|platforms| platforms.iter().any(|platform| platform == "linux"))
        {
            format!(
                "run this path on a supported host ({supported_os}; {supported_arch}) or use WSL and rerun `{}`",
                rerun_doctor_command_for_workflow(mode, lifecycle, workflow_name)
            )
        } else {
            format!(
                "run this path on a supported host ({supported_os}; {supported_arch}) and rerun `{}`",
                rerun_doctor_command_for_workflow(mode, lifecycle, workflow_name)
            )
        };
        let mut constraints = Vec::new();
        if context.only_on.is_some() {
            constraints.push(format!("`only_on: [{supported_os}]`"));
        }
        if context.only_arch.is_some() {
            constraints.push(format!("`only_arch: [{supported_arch}]`"));
        }
        findings.push(Finding::identified(
            "OTA_CONTEXT_HOST_PLATFORM_UNSUPPORTED",
            "execution",
            "host",
            FindingSeverity::Error,
            format!("Unsupported host platform for context: {context_name}"),
            format!(
                "the selected workflow/task path resolves `execution.contexts.{context_name}`, but that context declares {} and the current host is `{current_os}/{current_arch}`",
                constraints.join(" and ")
            ),
            next,
        ));
    }

    findings
}

fn contract_has_remote_execution_context(contract: &Contract) -> bool {
    contract.execution.as_ref().is_some_and(|execution| {
        execution.preferred == Some(Backend::Remote)
            || execution
                .contexts
                .values()
                .any(|context| context.backend == Backend::Remote)
    })
}

fn first_execution_context_for_backend<'a>(
    contract: &'a Contract,
    backend: Backend,
) -> Option<(&'a str, &'a crate::schema::ExecutionContext)> {
    let execution = contract.execution.as_ref()?;
    if let Some((name, context)) = execution.default_context()
        && context.backend == backend
    {
        return Some((name, context));
    }

    execution
        .contexts
        .iter()
        .find(|(_, context)| context.backend == backend)
        .map(|(name, context)| (name.as_str(), context))
}

fn execution_context_for_backend<'a>(
    contract: &'a Contract,
    backend: Backend,
    lifecycle: Option<Lifecycle>,
) -> Option<(&'a str, &'a crate::schema::ExecutionContext)> {
    let execution = contract.execution.as_ref()?;
    matching_declared_execution_context_name(Some(execution), backend, lifecycle)
        .and_then(|name| execution.contexts.get_key_value(name))
        .map(|(name, context)| (name.as_str(), context))
        .or_else(|| first_execution_context_for_backend(contract, backend))
}

#[derive(Debug, Default, Clone)]
struct ScopedPreconditionSelection {
    requirement_surface: RequirementSurface,
    toolchain_names: BTreeSet<String>,
    native_names: BTreeSet<String>,
    env_names: BTreeSet<String>,
    env_scoped: bool,
    dependency_hydration_owned: bool,
}

fn merge_effective_launch_command_tool_requirement(
    surface: &mut RequirementSurface,
    task: &crate::schema::TaskSpec,
    backend: Backend,
) {
    if let Some(exe) = task.effective_command_launch_executable_for_backend(backend, current_os()) {
        surface.presence_only_tools.insert(exe);
    }
}

#[derive(Debug, Clone)]
struct BackendPreconditionSelection {
    backend: Backend,
    context_name: Option<String>,
    requirement_surface: RequirementSurface,
    toolchain_names: BTreeSet<String>,
    native_names: BTreeSet<String>,
    env_names: BTreeSet<String>,
    env_scoped: bool,
    dependency_hydration_owned: bool,
}

impl From<BackendPreconditionSelection> for ScopedPreconditionSelection {
    fn from(value: BackendPreconditionSelection) -> Self {
        Self {
            requirement_surface: value.requirement_surface,
            toolchain_names: value.toolchain_names,
            native_names: value.native_names,
            env_names: value.env_names,
            env_scoped: value.env_scoped,
            dependency_hydration_owned: value.dependency_hydration_owned,
        }
    }
}

fn prepare_spec_owns_dependency_hydration(prepare: &crate::schema::TaskPrepareSpec) -> bool {
    match prepare {
        crate::schema::TaskPrepareSpec::DependencyHydration(_) => true,
        crate::schema::TaskPrepareSpec::Sequence(sequence) => sequence
            .steps
            .iter()
            .any(prepare_sequence_step_owns_dependency_hydration),
        _ => false,
    }
}

fn prepare_sequence_step_owns_dependency_hydration(
    step: &crate::schema::TaskPrepareSequenceStepSpec,
) -> bool {
    match step {
        crate::schema::TaskPrepareSequenceStepSpec::DependencyHydration(_) => true,
        crate::schema::TaskPrepareSequenceStepSpec::ToolBootstrap(_) => false,
        crate::schema::TaskPrepareSequenceStepSpec::Sequence(sequence) => sequence
            .steps
            .iter()
            .any(prepare_sequence_step_owns_dependency_hydration),
        crate::schema::TaskPrepareSequenceStepSpec::CopyIfMissing(_)
        | crate::schema::TaskPrepareSequenceStepSpec::EnsureEnvFile(_)
        | crate::schema::TaskPrepareSequenceStepSpec::EnsureFile(_)
        | crate::schema::TaskPrepareSequenceStepSpec::EnsureDirectory(_)
        | crate::schema::TaskPrepareSequenceStepSpec::EnsureGitCheckout(_)
        | crate::schema::TaskPrepareSequenceStepSpec::EnsureContainerNetwork(_)
        | crate::schema::TaskPrepareSequenceStepSpec::ResetComposeServiceVolume(_) => false,
    }
}

fn task_execution_owns_dependency_hydration(
    task: &crate::schema::TaskSpec,
    backend: Backend,
) -> bool {
    task.resolved_execution_for_backend(backend, current_os())
        .and_then(|execution| execution.prepare())
        .is_some_and(prepare_spec_owns_dependency_hydration)
}

fn looks_like_repo_local_executable(name: &str) -> bool {
    name.starts_with("./") || name.starts_with("../") || name.contains('/') || name.contains('\\')
}

fn should_skip_presence_only_container_hydration_probe_failure(
    unresolved_executable: &str,
    dependency_hydration_owned: bool,
    error: Option<&str>,
) -> bool {
    dependency_hydration_owned
        && looks_like_repo_local_executable(unresolved_executable)
        && container_repo_dependency_hydration_probe_error(error).is_some()
}

fn selected_backend_precondition_selections(
    contract: &Contract,
    workflow_name: Option<&str>,
    overrides: ExecutionOverrides,
) -> Vec<BackendPreconditionSelection> {
    let task_names = contract.selected_workflow_task_closure_names(workflow_name);
    if task_names.is_empty() {
        return Vec::new();
    }

    let scoped_runtimes = task_names.iter().any(|task_name| {
        contract.tasks.get(task_name.as_str()).is_some_and(|task| {
            let backend = effective_task_execution(contract, task_name.as_str(), overrides).backend;
            let context_name = task.context_for_backend(contract.execution.as_ref(), backend);
            !task
                .scoped_requirement_surface_for_execution(backend, context_name)
                .runtimes
                .is_empty()
        })
    });
    let scoped_env = task_names.iter().any(|task_name| {
        contract.tasks.get(task_name.as_str()).is_some_and(|task| {
            let backend = effective_task_execution(contract, task_name.as_str(), overrides).backend;
            let context_name = task.context_for_backend(contract.execution.as_ref(), backend);
            !task
                .scoped_env_requirements_for_execution(backend, context_name)
                .is_empty()
        })
    });

    let mut selections = Vec::<BackendPreconditionSelection>::new();
    let mut selected_tool_names_by_backend =
        BTreeMap::<(String, Option<String>), BTreeSet<String>>::new();

    for task_name in task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        let backend = effective_task_execution(contract, task_name.as_str(), overrides).backend;
        let context_name = task.context_for_backend(contract.execution.as_ref(), backend);
        let selection_context_name = matches!(backend, Backend::Container)
            .then(|| context_name.map(str::to_string))
            .flatten();
        let selection = if let Some(existing) = selections
            .iter_mut()
            .find(|item| item.backend == backend && item.context_name == selection_context_name)
        {
            existing
        } else {
            selections.push(BackendPreconditionSelection {
                backend,
                context_name: selection_context_name.clone(),
                requirement_surface: RequirementSurface::default(),
                toolchain_names: BTreeSet::new(),
                native_names: BTreeSet::new(),
                env_names: BTreeSet::new(),
                env_scoped: scoped_env,
                dependency_hydration_owned: false,
            });
            selections.last_mut().expect("selection was just pushed")
        };
        let selected_tool_names = selected_tool_names_by_backend
            .entry((
                backend_key(backend).to_string(),
                selection_context_name.clone(),
            ))
            .or_default();

        selection.dependency_hydration_owned |=
            task_execution_owns_dependency_hydration(task, backend);
        let scoped_surface = task.scoped_requirement_surface_for_execution(backend, context_name);
        for (name, requirement) in &scoped_surface.runtimes {
            selection.requirement_surface.runtimes.insert(
                name.clone(),
                contract.resolve_scoped_runtime_requirement(name, requirement),
            );
        }
        for (name, requirement) in &scoped_surface.tools {
            selected_tool_names.insert(name.clone());
            selection.requirement_surface.tools.insert(
                name.clone(),
                contract.resolve_scoped_tool_requirement(name, requirement),
            );
        }
        selection
            .toolchain_names
            .extend(contract.task_toolchain_names_for_execution(task, backend, context_name));
        selection
            .env_names
            .extend(task.scoped_env_requirements_for_execution(backend, context_name));
        if matches!(backend, Backend::Native) {
            let scoped_native =
                task.scoped_native_requirements_for_execution(backend, context_name);
            let native_toolchains = contract.native_prerequisite_required_toolchain_names_for_os(
                scoped_native.clone(),
                current_os(),
            );
            let native_env = contract
                .native_prerequisite_required_env_names_for_os(scoped_native.clone(), current_os());
            selection.toolchain_names.extend(native_toolchains);
            if !native_env.is_empty() {
                selection.env_scoped = true;
                selection.env_names.extend(native_env);
            }
            selection.native_names.extend(scoped_native.iter().cloned());
            selection.requirement_surface.merge(
                &contract
                    .native_prerequisite_requirement_surface_for_os(scoped_native, current_os()),
            );
        }
        merge_effective_launch_command_tool_requirement(
            &mut selection.requirement_surface,
            task,
            backend,
        );
        if let Some(exe) =
            task.effective_command_launch_executable_for_backend(backend, current_os())
        {
            selected_tool_names.insert(exe);
        }

        if let Some(context_name) = task.context_for_backend(contract.execution.as_ref(), backend)
            && let Some(context) = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.contexts.get(context_name))
        {
            for tool_name in context.requirements.tools.keys() {
                selected_tool_names.insert(tool_name.clone());
            }
            selection
                .requirement_surface
                .merge(&contract.resolved_context_requirement_surface(context));
        }
    }

    for selection in &mut selections {
        if !scoped_runtimes {
            let mut runtimes = contract.runtimes.clone();
            runtimes.extend(selection.requirement_surface.runtimes.clone());
            selection.requirement_surface.runtimes = runtimes;
        }
        if let Some(selected_tool_names) = selected_tool_names_by_backend.get(&(
            backend_key(selection.backend).to_string(),
            selection.context_name.clone(),
        )) {
            for tool_name in selected_tool_names {
                if let Some(requirement) = selection.requirement_surface.tools.get_mut(tool_name) {
                    requirement.force_required();
                }
            }
        }
    }

    selections
}

fn selected_task_backend_precondition_selections(
    contract: &Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Vec<BackendPreconditionSelection> {
    let task_names = contract.task_dependency_closure_names([task_name.to_string()]);
    if task_names.is_empty() {
        return Vec::new();
    }

    let scoped_runtimes = task_names.iter().any(|task_name| {
        contract.tasks.get(task_name.as_str()).is_some_and(|task| {
            let effective = effective_task_execution(contract, task_name.as_str(), overrides);
            !task
                .scoped_requirement_surface_for_execution(effective.backend, effective.context_name)
                .runtimes
                .is_empty()
        })
    });
    let scoped_env = task_names.iter().any(|task_name| {
        contract.tasks.get(task_name.as_str()).is_some_and(|task| {
            let effective = effective_task_execution(contract, task_name.as_str(), overrides);
            !task
                .scoped_env_requirements_for_execution(effective.backend, effective.context_name)
                .is_empty()
        })
    });

    let mut selections = Vec::<BackendPreconditionSelection>::new();
    let mut selected_tool_names_by_backend =
        BTreeMap::<(String, Option<String>), BTreeSet<String>>::new();

    for task_name in task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        let effective = effective_task_execution(contract, task_name.as_str(), overrides);
        let selection_context_name = matches!(effective.backend, Backend::Container)
            .then(|| effective.context_name.map(str::to_string))
            .flatten();
        let selection = if let Some(existing) = selections.iter_mut().find(|item| {
            item.backend == effective.backend && item.context_name == selection_context_name
        }) {
            existing
        } else {
            selections.push(BackendPreconditionSelection {
                backend: effective.backend,
                context_name: selection_context_name.clone(),
                requirement_surface: RequirementSurface::default(),
                toolchain_names: BTreeSet::new(),
                native_names: BTreeSet::new(),
                env_names: BTreeSet::new(),
                env_scoped: scoped_env,
                dependency_hydration_owned: false,
            });
            selections.last_mut().expect("selection was just pushed")
        };
        let selected_tool_names = selected_tool_names_by_backend
            .entry((
                backend_key(effective.backend).to_string(),
                selection_context_name.clone(),
            ))
            .or_default();

        selection.dependency_hydration_owned |=
            task_execution_owns_dependency_hydration(task, effective.backend);
        let scoped_surface = task
            .scoped_requirement_surface_for_execution(effective.backend, effective.context_name);
        for (name, requirement) in &scoped_surface.runtimes {
            selection.requirement_surface.runtimes.insert(
                name.clone(),
                contract.resolve_scoped_runtime_requirement(name, requirement),
            );
        }
        for (name, requirement) in &scoped_surface.tools {
            selected_tool_names.insert(name.clone());
            selection.requirement_surface.tools.insert(
                name.clone(),
                contract.resolve_scoped_tool_requirement(name, requirement),
            );
        }
        selection
            .toolchain_names
            .extend(contract.task_toolchain_names_for_execution(
                task,
                effective.backend,
                effective.context_name,
            ));
        selection.env_names.extend(
            task.scoped_env_requirements_for_execution(effective.backend, effective.context_name),
        );
        if matches!(effective.backend, Backend::Native) {
            let scoped_native = task.scoped_native_requirements_for_execution(
                effective.backend,
                effective.context_name,
            );
            let native_toolchains = contract.native_prerequisite_required_toolchain_names_for_os(
                scoped_native.clone(),
                current_os(),
            );
            let native_env = contract
                .native_prerequisite_required_env_names_for_os(scoped_native.clone(), current_os());
            selection.toolchain_names.extend(native_toolchains);
            if !native_env.is_empty() {
                selection.env_scoped = true;
                selection.env_names.extend(native_env);
            }
            selection.native_names.extend(scoped_native.iter().cloned());
            selection.requirement_surface.merge(
                &contract
                    .native_prerequisite_requirement_surface_for_os(scoped_native, current_os()),
            );
        }
        merge_effective_launch_command_tool_requirement(
            &mut selection.requirement_surface,
            task,
            effective.backend,
        );
        if let Some(exe) =
            task.effective_command_launch_executable_for_backend(effective.backend, current_os())
        {
            selected_tool_names.insert(exe);
        }

        if let Some(context_name) = effective.context_name
            && let Some(context) = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.contexts.get(context_name))
        {
            for tool_name in context.requirements.tools.keys() {
                selected_tool_names.insert(tool_name.clone());
            }
            selection
                .requirement_surface
                .merge(&contract.resolved_context_requirement_surface(context));
        }
    }

    for selection in &mut selections {
        if !scoped_runtimes {
            let mut runtimes = contract.runtimes.clone();
            runtimes.extend(selection.requirement_surface.runtimes.clone());
            selection.requirement_surface.runtimes = runtimes;
        }
        if let Some(selected_tool_names) = selected_tool_names_by_backend.get(&(
            backend_key(selection.backend).to_string(),
            selection.context_name.clone(),
        )) {
            for tool_name in selected_tool_names {
                if let Some(requirement) = selection.requirement_surface.tools.get_mut(tool_name) {
                    requirement.force_required();
                }
            }
        }
    }

    selections
}

fn scoped_precondition_selection(
    contract: &Contract,
    mode: DoctorMode,
    workflow_name: Option<&str>,
) -> ScopedPreconditionSelection {
    let backend = backend_for_mode(mode);
    let task_names = contract.selected_workflow_task_closure_names(workflow_name);
    if task_names.is_empty() {
        return ScopedPreconditionSelection {
            requirement_surface: contract.requirement_surface_for_backend(backend),
            toolchain_names: contract.toolchains.keys().cloned().collect(),
            ..ScopedPreconditionSelection::default()
        };
    }

    let scoped_runtimes = task_names.iter().any(|task_name| {
        contract.tasks.get(task_name.as_str()).is_some_and(|task| {
            let context_name = task.context_for_backend(contract.execution.as_ref(), backend);
            !task
                .scoped_requirement_surface_for_execution(backend, context_name)
                .runtimes
                .is_empty()
        })
    });
    let scoped_tools = task_names.iter().any(|task_name| {
        contract.tasks.get(task_name.as_str()).is_some_and(|task| {
            let context_name = task.context_for_backend(contract.execution.as_ref(), backend);
            !task
                .scoped_requirement_surface_for_execution(backend, context_name)
                .tools
                .is_empty()
        })
    });

    let mut selection = ScopedPreconditionSelection {
        requirement_surface: RequirementSurface::default(),
        toolchain_names: contract.selected_workflow_required_toolchain_names(workflow_name),
        ..ScopedPreconditionSelection::default()
    };
    let mut selected_tool_names = BTreeSet::new();

    for task_name in task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        let context_name = task.context_for_backend(contract.execution.as_ref(), backend);
        selection.dependency_hydration_owned |=
            task_execution_owns_dependency_hydration(task, backend);
        selection
            .requirement_surface
            .merge(&task.scoped_requirement_surface_for_execution(backend, context_name));
        selected_tool_names.extend(
            task.scoped_requirement_surface_for_execution(backend, context_name)
                .tools
                .keys()
                .cloned(),
        );
        let scoped_env = task.scoped_env_requirements_for_execution(backend, context_name);
        if !scoped_env.is_empty() {
            selection.env_scoped = true;
            selection.env_names.extend(scoped_env);
        }
        let scoped_native = task.scoped_native_requirements_for_execution(backend, context_name);
        selection.native_names.extend(scoped_native.iter().cloned());
        selection
            .toolchain_names
            .extend(contract.task_toolchain_names_for_execution(task, backend, context_name));
        if matches!(backend, Backend::Native) {
            let native_toolchains = contract.native_prerequisite_required_toolchain_names_for_os(
                scoped_native.clone(),
                current_os(),
            );
            let native_env = contract
                .native_prerequisite_required_env_names_for_os(scoped_native.clone(), current_os());
            selection.toolchain_names.extend(native_toolchains);
            if !native_env.is_empty() {
                selection.env_scoped = true;
                selection.env_names.extend(native_env);
            }
            selection.requirement_surface.merge(
                &contract
                    .native_prerequisite_requirement_surface_for_os(scoped_native, current_os()),
            );
        }
        merge_effective_launch_command_tool_requirement(
            &mut selection.requirement_surface,
            task,
            backend,
        );
        if let Some(exe) =
            task.effective_command_launch_executable_for_backend(backend, current_os())
        {
            selected_tool_names.insert(exe);
        }
        if let Some(context_name) = task.context_for_backend(contract.execution.as_ref(), backend)
            && let Some(context) = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.contexts.get(context_name))
        {
            selected_tool_names.extend(context.requirements.tools.keys().cloned());
            selection
                .requirement_surface
                .merge(&contract.resolved_context_requirement_surface(context));
        }
    }

    if !scoped_runtimes {
        let mut runtimes = contract.runtimes.clone();
        runtimes.extend(selection.requirement_surface.runtimes.clone());
        selection.requirement_surface.runtimes = runtimes;
    }
    if !scoped_tools && matches!(backend, Backend::Native) {
        let mut tools = contract.tools.clone();
        tools.extend(selection.requirement_surface.tools.clone());
        selection.requirement_surface.tools = tools;
    }
    for tool_name in &selected_tool_names {
        if let Some(requirement) = selection.requirement_surface.tools.get_mut(tool_name) {
            requirement.force_required();
        }
    }

    selection
}

#[cfg(test)]
fn precondition_requirement_surface(
    contract: &Contract,
    mode: DoctorMode,
    workflow_name: Option<&str>,
) -> RequirementSurface {
    scoped_precondition_selection(contract, mode, workflow_name).requirement_surface
}

#[derive(Debug, Clone, Default)]
struct RemoteTaskRequirementSelection {
    by_context: BTreeMap<String, ScopedPreconditionSelection>,
    fallback: Option<ScopedPreconditionSelection>,
}

fn selected_remote_task_requirement_selection(
    contract: &Contract,
    workflow_name: Option<&str>,
) -> Option<RemoteTaskRequirementSelection> {
    let task_names = contract.selected_workflow_task_closure_names(workflow_name);
    if task_names.is_empty() {
        return None;
    }

    #[derive(Debug, Default, Clone)]
    struct Entry {
        surface: RequirementSurface,
        toolchain_names: BTreeSet<String>,
        scoped_runtimes: bool,
        scoped_tools: bool,
        scoped_toolchains: bool,
    }

    let mut by_context = BTreeMap::<String, Entry>::new();
    let mut fallback = Entry::default();
    let mut fallback_used = false;
    let mut saw_task = false;
    for task_name in task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        saw_task = true;
        let target = if let Some(context_name) =
            task.context_for_backend(contract.execution.as_ref(), Backend::Remote)
        {
            by_context.entry(context_name.to_string()).or_default()
        } else {
            fallback_used = true;
            &mut fallback
        };
        let context_name = task.context_for_backend(contract.execution.as_ref(), Backend::Remote);
        let scoped_surface =
            task.scoped_requirement_surface_for_execution(Backend::Remote, context_name);
        if !scoped_surface.runtimes.is_empty() {
            target.scoped_runtimes = true;
        }
        if !scoped_surface.tools.is_empty() {
            target.scoped_tools = true;
        }
        let scoped_toolchains =
            contract.task_toolchain_names_for_execution(task, Backend::Remote, context_name);
        if !scoped_toolchains.is_empty() {
            target.scoped_toolchains = true;
            target.toolchain_names.extend(scoped_toolchains);
        }
        for (name, requirement) in &scoped_surface.runtimes {
            target.surface.runtimes.insert(
                name.clone(),
                contract.resolve_scoped_runtime_requirement(name, requirement),
            );
        }
        for (name, requirement) in &scoped_surface.tools {
            target.surface.tools.insert(
                name.clone(),
                contract.resolve_scoped_tool_requirement(name, requirement),
            );
        }
        merge_effective_launch_command_tool_requirement(&mut target.surface, task, Backend::Remote);
    }

    if !saw_task {
        return None;
    }

    let finalize_entry = |entry: Entry| -> ScopedPreconditionSelection {
        let mut surface = entry.surface;
        if !entry.scoped_runtimes {
            surface.runtimes = contract.runtimes.clone();
        }
        ScopedPreconditionSelection {
            requirement_surface: surface,
            toolchain_names: entry.toolchain_names,
            native_names: BTreeSet::new(),
            env_names: BTreeSet::new(),
            env_scoped: false,
            dependency_hydration_owned: false,
        }
    };

    let by_context = by_context
        .into_iter()
        .map(|(name, entry)| (name, finalize_entry(entry)))
        .collect();
    let fallback = fallback_used.then(|| finalize_entry(fallback));

    Some(RemoteTaskRequirementSelection {
        by_context,
        fallback,
    })
}

fn policy_requirement_surface_for_toolchains(
    contract: &Contract,
    requirement_surface: &RequirementSurface,
    toolchain_names: &BTreeSet<String>,
    target_os: &str,
) -> RequirementSurface {
    let required_tools = requirement_surface
        .tools
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    requirement_surface_with_toolchain_owned_capabilities_for_required_tools(
        contract,
        requirement_surface,
        toolchain_names,
        target_os,
        Some(&required_tools),
    )
}

fn corepack_activation_command(acquisition: &ToolAcquisitionSpec) -> String {
    format!(
        "corepack enable && corepack prepare {}@{} --activate",
        acquisition
            .package
            .as_deref()
            .expect("validated corepack acquisition package"),
        acquisition
            .version
            .as_deref()
            .expect("validated corepack acquisition version")
    )
}

fn corepack_bootstrap_and_activation_command(acquisition: &ToolAcquisitionSpec) -> String {
    format!(
        "npm install -g corepack && {}",
        corepack_activation_command(acquisition)
    )
}

fn tool_acquisition_shell_label(shell: NativePrerequisiteActivationShell) -> &'static str {
    match shell {
        NativePrerequisiteActivationShell::Sh => "sh",
        NativePrerequisiteActivationShell::Bash => "bash",
        NativePrerequisiteActivationShell::Zsh => "zsh",
        NativePrerequisiteActivationShell::Pwsh => "pwsh",
        NativePrerequisiteActivationShell::Cmd => "cmd",
    }
}

fn tool_acquisition_command(acquisition: &ToolAcquisitionSpec) -> String {
    match acquisition.provider {
        ToolAcquisitionProvider::Corepack => {
            if !command_available("corepack") && command_available("npm") {
                corepack_bootstrap_and_activation_command(acquisition)
            } else {
                corepack_activation_command(acquisition)
            }
        }
        ToolAcquisitionProvider::Command => {
            let shell = acquisition
                .shell
                .expect("validated command acquisition shell");
            let run = acquisition
                .run
                .as_deref()
                .expect("validated command acquisition run");
            match shell {
                NativePrerequisiteActivationShell::Sh
                | NativePrerequisiteActivationShell::Bash
                | NativePrerequisiteActivationShell::Zsh => format!(
                    "{} -lc {}",
                    tool_acquisition_shell_label(shell),
                    shell_single_quote(run)
                ),
                NativePrerequisiteActivationShell::Pwsh => format!(
                    "pwsh -NoProfile -NonInteractive -Command {}",
                    shell_single_quote(run)
                ),
                NativePrerequisiteActivationShell::Cmd => {
                    format!("cmd /d /s /c {}", shell_single_quote(run))
                }
            }
        }
        ToolAcquisitionProvider::ReleaseAsset => String::from("ota up"),
        ToolAcquisitionProvider::Apt => format!(
            "apt-get install -y {}",
            acquisition.package.as_deref().unwrap_or("<package>")
        ),
        ToolAcquisitionProvider::Brew => format!(
            "brew install {}",
            acquisition.package.as_deref().unwrap_or("<package>")
        ),
        ToolAcquisitionProvider::Winget => format!(
            "winget install --id {} --exact",
            acquisition.package.as_deref().unwrap_or("<package>")
        ),
        ToolAcquisitionProvider::Choco => format!(
            "choco install {} -y",
            acquisition.package.as_deref().unwrap_or("<package>")
        ),
        ToolAcquisitionProvider::Scoop => format!(
            "scoop install {}",
            acquisition.package.as_deref().unwrap_or("<package>")
        ),
    }
}

fn corepack_provider_bootstrap_available() -> bool {
    !command_available("corepack") && command_available("npm")
}

fn tool_acquisition_provider_available(acquisition: &ToolAcquisitionSpec) -> bool {
    match acquisition.provider {
        ToolAcquisitionProvider::Corepack => {
            command_available("corepack") || corepack_provider_bootstrap_available()
        }
        ToolAcquisitionProvider::Command => {
            command_available(tool_acquisition_provider_requirement(acquisition))
        }
        ToolAcquisitionProvider::ReleaseAsset => {
            command_available("curl") || command_available("wget")
        }
        ToolAcquisitionProvider::Apt
        | ToolAcquisitionProvider::Brew
        | ToolAcquisitionProvider::Winget
        | ToolAcquisitionProvider::Choco
        | ToolAcquisitionProvider::Scoop => {
            command_available(tool_acquisition_provider_requirement(acquisition))
        }
    }
}

fn tool_acquisition_provider_requirement(acquisition: &ToolAcquisitionSpec) -> &'static str {
    match acquisition.provider {
        ToolAcquisitionProvider::Corepack => "corepack",
        ToolAcquisitionProvider::Command => tool_acquisition_shell_label(
            acquisition
                .shell
                .expect("validated command acquisition shell"),
        ),
        ToolAcquisitionProvider::ReleaseAsset => "curl or wget",
        ToolAcquisitionProvider::Apt => "apt-get",
        ToolAcquisitionProvider::Brew => "brew",
        ToolAcquisitionProvider::Winget => "winget",
        ToolAcquisitionProvider::Choco => "choco",
        ToolAcquisitionProvider::Scoop => "scoop",
    }
}

fn direct_tool_acquisition_provisioning_actions(
    contract: &Contract,
    requirement_surface: &RequirementSurface,
    target_os: &str,
) -> Vec<ProvisioningAction> {
    requirement_surface
        .tools
        .iter()
        .filter(|(_, requirement)| {
            requirement.active_for_os(target_os) && requirement.required_for_os(target_os)
        })
        .filter_map(|(name, requirement)| {
            let contract_requirement = contract.tools.get(name.as_str());
            let acquisition = contract_requirement
                .and_then(|tool| tool.acquisition_for_os(target_os))
                .or_else(|| requirement.acquisition_for_os(target_os))?;
            let source = acquisition.provider.provisioning_source()?;
            let requested_version = contract_requirement
                .map(|tool| tool.version_for_os(target_os).trim().to_string())
                .filter(|version| !version.is_empty() && version != "*")
                .unwrap_or_else(|| requirement.version_for_os(target_os).to_string());
            Some(ProvisioningAction {
                kind: if matches!(acquisition.provider, ToolAcquisitionProvider::ReleaseAsset) {
                    crate::policy_pack::ProvisioningActionKind::SelectSource
                } else {
                    crate::policy_pack::ProvisioningActionKind::Install
                },
                target_kind: ProvisioningTargetKind::Tool,
                name: name.clone(),
                requested_version,
                normalized_requirement: None,
                resolved_version: None,
                package: acquisition.package.clone().or_else(|| {
                    (!matches!(acquisition.provider, ToolAcquisitionProvider::ReleaseAsset))
                        .then(|| name.clone())
                }),
                source: source.to_string(),
                source_config: acquisition.source_config.clone(),
                approved_version: None,
                policy_match: None,
            })
        })
        .collect()
}

fn merge_direct_tool_acquisition_provisioning(
    contract: &Contract,
    provisioning: Option<ProvisioningDiagnostics>,
    requirement_surface: &RequirementSurface,
    target_os: &str,
) -> Option<ProvisioningDiagnostics> {
    let direct_actions =
        direct_tool_acquisition_provisioning_actions(contract, requirement_surface, target_os);
    if direct_actions.is_empty() {
        return provisioning;
    }

    let mut diagnostics = provisioning.unwrap_or(ProvisioningDiagnostics {
        plan: ProvisioningPlan::default(),
        request: ProvisioningBackendRequest {
            actions: Vec::new(),
        },
    });

    for action in direct_actions {
        let duplicate = diagnostics.request.actions.iter().any(|existing| {
            existing.target_kind == action.target_kind
                && existing.name == action.name
                && existing.source == action.source
        });
        if duplicate {
            continue;
        }
        diagnostics.plan.actions.push(action.clone());
        diagnostics.request.actions.push(action);
    }

    Some(diagnostics)
}

fn merged_provisioning_actions_for_requirement_surface(
    contract: &Contract,
    base_actions: Vec<ProvisioningAction>,
    requirement_surface: &RequirementSurface,
    target_os: &str,
) -> Vec<ProvisioningAction> {
    merge_direct_tool_acquisition_provisioning(
        contract,
        Some(ProvisioningDiagnostics {
            plan: ProvisioningPlan::default(),
            request: ProvisioningBackendRequest {
                actions: base_actions,
            },
        }),
        requirement_surface,
        target_os,
    )
    .map(|diagnostics| diagnostics.request.actions)
    .unwrap_or_default()
}

fn exact_tooling_remediation(
    target_kind: ProvisioningTargetKind,
    name: &str,
    requirement: &str,
    provider_hint: Option<&str>,
    acquisition: Option<&ToolAcquisitionSpec>,
    contract_path: &Path,
    provisioning_actions: &[ProvisioningAction],
) -> Option<String> {
    if let Some(acquisition) = acquisition {
        if acquisition.provider.provisioning_source().is_some() && acquisition.package.is_none() {
            let mut acquisition = acquisition.clone();
            acquisition.package = Some(name.to_string());
            return Some(tool_acquisition_command(&acquisition));
        }
        return Some(tool_acquisition_command(acquisition));
    }
    exact_tooling_remediation_fallback(
        target_kind,
        name,
        requirement,
        provider_hint,
        contract_path,
        provisioning_actions,
    )
}

fn remote_os_probe_command() -> &'static str {
    r#"uname -s 2>/dev/null || printf '%s\n' "${OS:-}""#
}

fn normalize_remote_target_os(value: &str) -> Option<&'static str> {
    let normalized = value.trim().to_ascii_lowercase();
    if normalized.contains("linux") {
        Some("linux")
    } else if normalized.contains("darwin") || normalized.contains("mac") {
        Some("macos")
    } else if normalized.contains("windows")
        || normalized.contains("mingw")
        || normalized.contains("msys")
        || normalized.contains("cygwin")
    {
        Some("windows")
    } else {
        None
    }
}

fn remote_target_os_probe_finding(context_name: Option<&str>, why: String) -> Finding {
    let suffix = context_name
        .map(|value| format!(": {value}"))
        .unwrap_or_default();
    let next = context_name
        .map(|value| {
            format!(
                "make sure ota can execute `{}` successfully in remote context `{value}`, then rerun `ota doctor --mode remote`",
                remote_os_probe_command()
            )
        })
        .unwrap_or_else(|| {
            format!(
                "make sure ota can execute `{}` successfully through the selected remote backend, then rerun `ota doctor --mode remote`",
                remote_os_probe_command()
            )
        });
    Finding::identified(
        "OTA_REMOTE_TARGET_OS_UNDETERMINED",
        "remote",
        "remote_backend",
        FindingSeverity::Error,
        format!("Remote target operating system could not be determined{suffix}"),
        why,
        next,
    )
}

fn probe_remote_target_os(
    backend: &ResolvedExecutionBackend,
    working_dir: &Path,
) -> Result<String, String> {
    let output = run_backend_command_captured(
        "doctor-remote-os",
        remote_os_probe_command(),
        working_dir,
        backend,
    )
    .map_err(|error| error.to_string())?;
    if output.exit_code != 0 {
        let details = if output.stderr.trim().is_empty() {
            if output.stdout.trim().is_empty() {
                format!(
                    "`{}` exited with code {} before ota could determine the remote OS",
                    remote_os_probe_command(),
                    output.exit_code
                )
            } else {
                format!(
                    "`{}` exited with code {}: {}",
                    remote_os_probe_command(),
                    output.exit_code,
                    output.stdout.trim()
                )
            }
        } else {
            format!(
                "`{}` exited with code {}: {}",
                remote_os_probe_command(),
                output.exit_code,
                output.stderr.trim()
            )
        };
        return Err(details);
    }

    let raw = output
        .stdout
        .lines()
        .chain(output.stderr.lines())
        .map(str::trim)
        .find(|line| !line.is_empty())
        .ok_or_else(|| {
            format!(
                "`{}` did not return a recognizable operating system name",
                remote_os_probe_command()
            )
        })?;

    normalize_remote_target_os(raw)
        .map(str::to_string)
        .ok_or_else(|| {
            format!(
                "ota could not map remote OS probe output `{raw}` to `linux`, `macos`, or `windows`"
            )
        })
}

fn remote_doctor_probe_contexts(
    contract: &Contract,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    workflow_name: Option<&str>,
    findings: &mut Vec<Finding>,
) -> Vec<RemoteProbeContext> {
    let mut probes = Vec::new();
    let Some(execution) = contract.execution.as_ref() else {
        return probes;
    };
    let working_dir = contract_working_dir(contract_path);
    let selected_task_requirements =
        selected_remote_task_requirement_selection(contract, workflow_name);

    for (name, context) in execution
        .contexts
        .iter()
        .filter(|(_, context)| context.backend == Backend::Remote)
    {
        let selected_task_surface = if let Some(selection) = selected_task_requirements.as_ref() {
            let Some(scoped) = selection.by_context.get(name.as_str()) else {
                continue;
            };
            Some(scoped)
        } else {
            None
        };
        let backend = match resolve_context_execution_backend(contract, name) {
            Ok(backend) => backend,
            Err(error) => {
                findings.push(remote_backend_finding(
                    "OTA_REMOTE_CONTEXT_UNEXECUTABLE",
                    FindingSeverity::Error,
                    format!("Remote execution context is not executable: {name}"),
                    error.to_string(),
                    format!(
                        "repair `execution.contexts.{name}` so ota can execute that remote context, then rerun `ota doctor --mode remote`"
                    ),
                ));
                continue;
            }
        };
        let target_os = match probe_remote_target_os(&backend, working_dir) {
            Ok(target_os) => target_os,
            Err(why) => {
                findings.push(remote_target_os_probe_finding(Some(name.as_str()), why));
                continue;
            }
        };
        let mut requirement_surface = selected_task_surface
            .map(|selection| selection.requirement_surface.clone())
            .unwrap_or_else(|| RequirementSurface {
                runtimes: contract.runtimes.clone(),
                tools: contract.tools.clone(),
                presence_only_tools: BTreeSet::new(),
            });
        requirement_surface.merge(&contract.resolved_context_requirement_surface(context));
        let selected_toolchain_names = selected_task_surface
            .map(|selection| selection.toolchain_names.clone())
            .unwrap_or_else(|| contract.toolchains.keys().cloned().collect());
        let policy_requirement_surface = policy_requirement_surface_for_toolchains(
            contract,
            &requirement_surface,
            &selected_toolchain_names,
            &target_os,
        );
        let required_tools = requirement_surface
            .tools
            .keys()
            .cloned()
            .collect::<BTreeSet<_>>();
        let provisioning_requirement_surface =
            requirement_surface_with_toolchain_owned_tools_for_required_tools(
                contract,
                &requirement_surface,
                &selected_toolchain_names,
                &target_os,
                Some(&required_tools),
            );
        let provisioning_actions = merged_provisioning_actions_for_requirement_surface(
            contract,
            loaded_policy
                .map(|loaded| {
                    loaded
                        .pack
                        .selected_provisioning_actions_for_requirement_surface_os(
                            &target_os,
                            &policy_requirement_surface,
                        )
                })
                .unwrap_or_default(),
            &provisioning_requirement_surface,
            &target_os,
        );
        probes.push(RemoteProbeContext {
            context_name: Some(name.clone()),
            backend,
            target_os,
            requirement_surface,
            policy_requirement_surface,
            provisioning_actions,
        });
    }

    if !probes.is_empty() {
        return probes;
    }

    if selected_task_requirements
        .as_ref()
        .is_some_and(|selection| !selection.by_context.is_empty() && selection.fallback.is_none())
    {
        return probes;
    }

    let Some(remote) = execution
        .backends
        .as_ref()
        .and_then(|backends| backends.remote.as_ref())
    else {
        return probes;
    };
    let Some(target) = remote
        .target
        .clone()
        .filter(|target| !target.trim().is_empty())
    else {
        return probes;
    };
    let backend = match remote.provider.as_str() {
        "daytona" | "ssh" | "tsh" | "kubectl" => ResolvedExecutionBackend::Remote {
            shared_local_backend: None,
            provider: remote.provider.clone(),
            target,
            cwd: remote.cwd.clone(),
            ssh: remote.ssh.clone(),
        },
        other => {
            let Some(extension) = contract.extensions.get(other) else {
                return probes;
            };
            if extension.kind != ExtensionKind::BackendProvider || extension.api_version != 1 {
                return probes;
            }
            ResolvedExecutionBackend::BackendProvider {
                shared_local_backend: None,
                provider: remote.provider.clone(),
                command: extension.command.clone(),
                target,
                cwd: remote.cwd.clone(),
            }
        }
    };
    let target_os = match probe_remote_target_os(&backend, working_dir) {
        Ok(target_os) => target_os,
        Err(why) => {
            findings.push(remote_target_os_probe_finding(None, why));
            return probes;
        }
    };
    let requirement_surface = selected_task_requirements
        .as_ref()
        .and_then(|selection| selection.fallback.as_ref())
        .map(|selection| selection.requirement_surface.clone())
        .unwrap_or_else(|| RequirementSurface {
            runtimes: contract.runtimes.clone(),
            tools: contract.tools.clone(),
            presence_only_tools: BTreeSet::new(),
        });
    let selected_toolchain_names = selected_task_requirements
        .as_ref()
        .and_then(|selection| selection.fallback.as_ref())
        .map(|selection| selection.toolchain_names.clone())
        .unwrap_or_else(|| contract.toolchains.keys().cloned().collect());
    let policy_requirement_surface = policy_requirement_surface_for_toolchains(
        contract,
        &requirement_surface,
        &selected_toolchain_names,
        policy_target_os_for_mode(DoctorMode::Remote),
    );
    let required_tools = requirement_surface
        .tools
        .keys()
        .cloned()
        .collect::<BTreeSet<_>>();
    let provisioning_requirement_surface =
        requirement_surface_with_toolchain_owned_tools_for_required_tools(
            contract,
            &requirement_surface,
            &selected_toolchain_names,
            policy_target_os_for_mode(DoctorMode::Remote),
            Some(&required_tools),
        );
    let provisioning_actions = merged_provisioning_actions_for_requirement_surface(
        contract,
        loaded_policy
            .map(|loaded| {
                loaded
                    .pack
                    .selected_provisioning_actions_for_requirement_surface_os(
                        &target_os,
                        &policy_requirement_surface,
                    )
            })
            .unwrap_or_default(),
        &provisioning_requirement_surface,
        &target_os,
    );

    probes.push(RemoteProbeContext {
        context_name: None,
        backend,
        target_os,
        requirement_surface,
        policy_requirement_surface,
        provisioning_actions,
    });

    probes
}

fn has_remote_backend_blocker(findings: &[Finding]) -> bool {
    findings.iter().any(|finding| {
        matches!(
            finding.code(),
            "OTA_BACKEND_CLI_MISSING"
                | "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED"
                | "OTA_CONTAINER_BACKEND_CLI_MISSING"
        )
    })
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize)]
pub struct FindingEvidence {
    pub observed: String,
    pub expected: String,
    pub source: String,
    pub checked_at: String,
    pub command: String,
    pub path: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PolicyFindingContext<'a> {
    outcome: &'a str,
    reason: &'a str,
    source: &'a str,
    install_scope: &'a str,
    mutation_allowed: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DriftFindingContext<'a> {
    owner_kind: &'a str,
    ownership: &'a str,
    provenance: &'a str,
    provenance_key: &'a str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FindingProvenanceContext<'a> {
    provenance: &'a str,
    provenance_key: &'a str,
}

#[derive(Debug, Clone)]
struct FindingResolvedMetadata<'a> {
    category: &'static str,
    owner: &'static str,
    correlation_surfaces: &'static [&'static str],
    correlation_owner_prefix: Option<String>,
    correlation_entity: Option<String>,
    evidence: Option<FindingEvidence>,
    provenance: Option<FindingProvenanceContext<'a>>,
    policy: Option<PolicyFindingContext<'a>>,
}

#[derive(Clone, Copy)]
struct FindingRegistryEntry {
    resolver: for<'a> fn(&'a Finding) -> FindingResolvedMetadata<'a>,
}

fn static_finding_evidence(observed: &str, expected: &str, source: &str) -> FindingEvidence {
    FindingEvidence {
        observed: observed.to_string(),
        expected: expected.to_string(),
        source: source.to_string(),
        checked_at: String::new(),
        command: String::new(),
        path: String::new(),
    }
}

fn repo_contract_provenance() -> Option<FindingProvenanceContext<'static>> {
    Some(FindingProvenanceContext {
        provenance: "repo contract",
        provenance_key: "repo_contract",
    })
}

fn org_policy_provenance() -> Option<FindingProvenanceContext<'static>> {
    Some(FindingProvenanceContext {
        provenance: "org policy",
        provenance_key: "org_policy",
    })
}

fn repo_signals_provenance() -> Option<FindingProvenanceContext<'static>> {
    Some(FindingProvenanceContext {
        provenance: "repo signals",
        provenance_key: "repo_signals",
    })
}

fn finding_probe_target_owner(finding: &Finding) -> &'static str {
    if finding_targets_container_image(&finding.why) {
        "container_target"
    } else if finding_targets_remote_backend(&finding.why) {
        "remote_target"
    } else {
        "host"
    }
}

fn probe_finding_evidence(finding: &Finding, observed: &str, expected: &str) -> FindingEvidence {
    FindingEvidence {
        observed: observed.to_string(),
        expected: expected.to_string(),
        source: finding_probe_target_owner(finding).to_string(),
        checked_at: String::new(),
        command: finding_probe_command(&finding.why).unwrap_or_default(),
        path: finding_probe_path(&finding.why).unwrap_or_default(),
    }
}

fn finding_suffix_after_prefix<'a>(summary: &'a str, prefixes: &[&str]) -> Option<&'a str> {
    prefixes
        .iter()
        .find_map(|prefix| summary.strip_prefix(prefix))
        .map(str::trim)
}

fn resolve_contract_core_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let evidence = match finding.code() {
        "OTA_TASKS_MISSING" => Some(static_finding_evidence(
            "no runnable task entry was declared",
            "at least one runnable task is declared",
            "contract",
        )),
        "OTA_CONTRACT_DRIFT" => Some(static_finding_evidence(
            "repo signals differ from the declared contract",
            "repo signals match the declared contract",
            "detect",
        )),
        "OTA_CONTRACT_ADVISORY_TASK_MUTATES_MANAGED_ISOLATED_PATH" => {
            Some(static_finding_evidence(
                "a task body appears to mutate an ota-managed isolated attachment path",
                "task bodies leave ota-managed isolated attachment paths to the underlying tool",
                "contract",
            ))
        }
        _ => None,
    };

    FindingResolvedMetadata {
        category: "contract",
        owner: "repo_contract",
        correlation_surfaces: match finding.code() {
            "OTA_TASKS_MISSING" | "OTA_SELECTED_TASK_PATH_EXTERNAL_STATE" => &["task"],
            _ => &[],
        },
        correlation_owner_prefix: match finding.code() {
            "OTA_TASKS_MISSING" => Some(String::from("tasks")),
            _ => None,
        },
        correlation_entity: None,
        evidence,
        provenance: repo_contract_provenance(),
        policy: None,
    }
}

fn resolve_contractless_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let code = finding.code();
    let evidence = match code {
        "OTA_CONTRACTLESS_REPO_CONTRACT_MISSING" => Some(static_finding_evidence(
            "no repo contract was found from the selected path upward",
            "a repo contract is present and readable",
            "repo filesystem",
        )),
        "OTA_CONTRACTLESS_SIGNAL_INSPECTION_FAILED" => Some(static_finding_evidence(
            "repo signal inspection failed before a starter contract could be inferred",
            "repo signal inspection succeeds and a starter contract can be inferred",
            "detect",
        )),
        _ => None,
    };

    FindingResolvedMetadata {
        category: "contract",
        owner: if code == "OTA_CONTRACTLESS_REPO_CONTRACT_MISSING" {
            "repo_contract"
        } else {
            "repo_signals"
        },
        correlation_surfaces: &[],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence,
        provenance: repo_signals_provenance(),
        policy: None,
    }
}

fn resolve_execution_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let evidence = match finding.code() {
        "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY" | "OTA_LIFECYCLE_EPHEMERAL_ADVISORY" => {
            Some(static_finding_evidence(
                "ephemeral lifecycle was requested",
                "isolated backend-backed execution is available",
                "execution",
            ))
        }
        "OTA_CHECK_FAILED" => Some(static_finding_evidence(
            "the configured check failed",
            "the configured check succeeds",
            "execution",
        )),
        "OTA_CHECK_TIMED_OUT" => Some(static_finding_evidence(
            "the configured check timed out",
            "the configured check completes within the timeout",
            "execution",
        )),
        "OTA_FILE_CHECK_FAILED" => Some(static_finding_evidence(
            "the configured file check failed",
            "the configured file state matches the contract",
            "repo filesystem",
        )),
        "OTA_FILE_CHECK_TIMED_OUT" => Some(static_finding_evidence(
            "the configured file check timed out",
            "the configured file check completes within the timeout",
            "repo filesystem",
        )),
        "OTA_WORKFLOW_PROBE_FAILED" | "OTA_WORKFLOW_SIGNAL_PROBE_FAILED" => {
            Some(static_finding_evidence(
                "the configured workflow probe did not succeed",
                "the configured workflow probe succeeds",
                "execution",
            ))
        }
        "OTA_WORKFLOW_PROBE_TIMED_OUT" | "OTA_WORKFLOW_SIGNAL_PROBE_TIMED_OUT" => {
            Some(static_finding_evidence(
                "the configured workflow probe timed out",
                "the configured workflow probe completes within its timeout",
                "execution",
            ))
        }
        "OTA_WORKFLOW_SURFACE_READINESS_FAILED"
        | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED" => Some(static_finding_evidence(
            "the selected workflow surface did not become ready",
            "the selected workflow surface becomes ready",
            "execution",
        )),
        "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT"
        | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT" => Some(static_finding_evidence(
            "the selected workflow surface did not become ready before timing out",
            "the selected workflow surface becomes ready within its timeout",
            "execution",
        )),
        "OTA_WORKFLOW_SURFACE_READINESS_UNEVALUABLE"
        | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_UNEVALUABLE" => Some(static_finding_evidence(
            "the selected workflow surface could not be resolved or checked",
            "the selected workflow surface can be resolved and checked",
            "execution",
        )),
        _ => None,
    };

    let owner = if finding.code() == "OTA_CONTEXT_HOST_PLATFORM_UNSUPPORTED" {
        "host"
    } else {
        "repo_contract"
    };

    FindingResolvedMetadata {
        category: "execution",
        owner,
        correlation_surfaces: match finding.code() {
            "OTA_CHECK_FAILED"
            | "OTA_CHECK_TIMED_OUT"
            | "OTA_FILE_CHECK_FAILED"
            | "OTA_FILE_CHECK_TIMED_OUT" => &["task"],
            "OTA_WORKFLOW_PROBE_FAILED"
            | "OTA_WORKFLOW_PROBE_TIMED_OUT"
            | "OTA_WORKFLOW_SIGNAL_PROBE_FAILED"
            | "OTA_WORKFLOW_SIGNAL_PROBE_TIMED_OUT"
            | "OTA_WORKFLOW_SURFACE_READINESS_FAILED"
            | "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT"
            | "OTA_WORKFLOW_SURFACE_READINESS_UNEVALUABLE"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_UNEVALUABLE" => &["workflow"],
            _ => &["execution"],
        },
        correlation_owner_prefix: match finding.code() {
            "OTA_CHECK_FAILED" | "OTA_CHECK_TIMED_OUT" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &["Check failed: ", "Check timed out: "],
            )
            .map(|check| format!("checks.{check}")),
            "OTA_WORKFLOW_PROBE_FAILED"
            | "OTA_WORKFLOW_PROBE_TIMED_OUT"
            | "OTA_WORKFLOW_SIGNAL_PROBE_FAILED"
            | "OTA_WORKFLOW_SIGNAL_PROBE_TIMED_OUT" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &[
                    "Probe failed: ",
                    "Probe timed out: ",
                    "Signal probe failed: ",
                    "Signal probe timed out: ",
                ],
            )
            .map(|probe| format!("readiness.probes.{probe}")),
            "OTA_WORKFLOW_SURFACE_READINESS_FAILED"
            | "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT"
            | "OTA_WORKFLOW_SURFACE_READINESS_UNEVALUABLE"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_UNEVALUABLE" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &[
                    "Surface readiness failed: ",
                    "Surface readiness timed out: ",
                    "Surface readiness could not be evaluated: ",
                    "Signal surface readiness failed: ",
                    "Signal surface readiness timed out: ",
                    "Signal surface readiness could not be evaluated: ",
                ],
            )
            .map(|surface| format!("surfaces.{surface}")),
            _ => None,
        },
        correlation_entity: match finding.code() {
            "OTA_CHECK_FAILED" | "OTA_CHECK_TIMED_OUT" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &["Check failed: ", "Check timed out: "],
            )
            .map(str::to_string),
            "OTA_WORKFLOW_PROBE_FAILED"
            | "OTA_WORKFLOW_PROBE_TIMED_OUT"
            | "OTA_WORKFLOW_SIGNAL_PROBE_FAILED"
            | "OTA_WORKFLOW_SIGNAL_PROBE_TIMED_OUT" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &[
                    "Probe failed: ",
                    "Probe timed out: ",
                    "Signal probe failed: ",
                    "Signal probe timed out: ",
                ],
            )
            .map(str::to_string),
            "OTA_WORKFLOW_SURFACE_READINESS_FAILED"
            | "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT"
            | "OTA_WORKFLOW_SURFACE_READINESS_UNEVALUABLE"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT"
            | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_UNEVALUABLE" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &[
                    "Surface readiness failed: ",
                    "Surface readiness timed out: ",
                    "Surface readiness could not be evaluated: ",
                    "Signal surface readiness failed: ",
                    "Signal surface readiness timed out: ",
                    "Signal surface readiness could not be evaluated: ",
                ],
            )
            .map(str::to_string),
            _ => None,
        },
        evidence,
        provenance: repo_contract_provenance(),
        policy: None,
    }
}

fn resolve_backend_cli_finding_metadata(_: &Finding) -> FindingResolvedMetadata<'_> {
    FindingResolvedMetadata {
        category: "execution",
        owner: "host",
        correlation_surfaces: &["execution"],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence: Some(static_finding_evidence(
            "required backend CLI was not found on PATH",
            "a supported backend CLI is available on PATH",
            "host",
        )),
        provenance: repo_contract_provenance(),
        policy: None,
    }
}

fn resolve_container_backend_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    FindingResolvedMetadata {
        category: "execution",
        owner: if finding.code() == "OTA_CONTAINER_IMAGE_UNAVAILABLE" {
            "container_target"
        } else {
            "host"
        },
        correlation_surfaces: &["execution"],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence: None,
        provenance: repo_contract_provenance(),
        policy: None,
    }
}

fn resolve_remote_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let code = finding.code();
    let evidence = match code {
        "OTA_REMOTE_MODE_NOT_CONFIGURED" => Some(static_finding_evidence(
            "the selected remote execution path is not declared",
            "the contract declares a remote execution path",
            "repo_contract",
        )),
        "OTA_REMOTE_DOCTOR_PARTIAL" | "OTA_REMOTE_DOCTOR_HOST_SCOPE_NOTE" => {
            Some(static_finding_evidence(
                "remote doctor mode is still constrained by host-scoped checks",
                "remote doctor mode can evaluate the selected remote path end-to-end",
                "repo_contract",
            ))
        }
        "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED" => Some(static_finding_evidence(
            "the declared remote backend provider is unsupported",
            "a supported remote backend provider is declared",
            "remote_backend",
        )),
        "OTA_REMOTE_TARGET_SUSPICIOUS" => Some(static_finding_evidence(
            "the remote target shape did not match provider expectations",
            "a provider-compatible remote target is declared",
            "remote_backend",
        )),
        "OTA_REMOTE_CONTEXT_UNEXECUTABLE" => Some(static_finding_evidence(
            "the named remote execution context could not be resolved",
            "the named remote execution context is executable",
            "remote_backend",
        )),
        "OTA_REMOTE_TARGET_OS_UNDETERMINED" => Some(FindingEvidence {
            observed: String::from("ota could not determine the remote target operating system"),
            expected: String::from("ota can determine the remote target operating system"),
            source: String::from("remote_backend"),
            checked_at: String::new(),
            command: remote_os_probe_command().to_string(),
            path: String::new(),
        }),
        _ => None,
    };

    FindingResolvedMetadata {
        category: "remote",
        owner: if code == "OTA_REMOTE_MODE_NOT_CONFIGURED" {
            "repo_contract"
        } else {
            "remote_backend"
        },
        correlation_surfaces: &["execution"],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence,
        provenance: repo_contract_provenance(),
        policy: None,
    }
}

fn resolve_service_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let evidence = match finding.code() {
        "OTA_SERVICE_READINESS_FAILED" => Some(static_finding_evidence(
            "the configured service readiness probe failed",
            "the service readiness probe passes from its declared context",
            "service",
        )),
        "OTA_SERVICE_CHECK_FAILED" => Some(static_finding_evidence(
            "the configured service healthcheck failed",
            "the service healthcheck passes",
            "service",
        )),
        "OTA_SERVICE_CHECK_TIMED_OUT" => Some(static_finding_evidence(
            "the configured service healthcheck timed out",
            "the service healthcheck completes within its timeout",
            "service",
        )),
        "OTA_SERVICE_UNVERIFIABLE" => Some(static_finding_evidence(
            "the service cannot be verified from the contract",
            "the service declares enough information to verify readiness",
            "service",
        )),
        _ => None,
    };

    FindingResolvedMetadata {
        category: "service",
        owner: "service",
        correlation_surfaces: &["service"],
        correlation_owner_prefix: finding_suffix_after_prefix(
            finding.summary.trim(),
            &[
                "Service readiness failed: ",
                "Service readiness context is not executable: ",
                "Required service cannot be verified: ",
                "Service producer is not ready: ",
                "Service healthcheck failed: ",
                "Service healthcheck timed out: ",
            ],
        )
        .map(|service| format!("services.{service}")),
        correlation_entity: finding_suffix_after_prefix(
            finding.summary.trim(),
            &[
                "Service readiness failed: ",
                "Service readiness context is not executable: ",
                "Required service cannot be verified: ",
                "Service producer is not ready: ",
                "Service healthcheck failed: ",
                "Service healthcheck timed out: ",
            ],
        )
        .map(str::to_string),
        evidence,
        provenance: repo_contract_provenance(),
        policy: None,
    }
}

fn resolve_environment_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let code = finding.code();
    let evidence = match code {
        "OTA_ENV_MISSING" => Some(static_finding_evidence(
            "a required environment variable was missing",
            "the environment variable is present",
            "contract",
        )),
        "OTA_ENV_INVALID" => Some(static_finding_evidence(
            "the resolved environment value is outside the allowed set",
            "the environment value satisfies the allowed set",
            "contract",
        )),
        "OTA_ENV_SOURCE_MISSING_REQUIRED" => Some(static_finding_evidence(
            "a required declared environment source was missing",
            "the declared environment source exists",
            "repo filesystem",
        )),
        "OTA_ENV_SOURCE_PARSE_FAILED" => Some(static_finding_evidence(
            "a declared environment source could not be parsed",
            "the declared environment source parses successfully",
            "repo filesystem",
        )),
        "OTA_ENV_SOURCE_INVALID_STRUCTURE" => Some(static_finding_evidence(
            "a declared environment source had unsupported structure",
            "the declared environment source has supported structure",
            "repo filesystem",
        )),
        "OTA_ENV_SOURCE_KEY_COLLISION" => Some(static_finding_evidence(
            "declared environment sources produced conflicting keys",
            "declared environment sources resolve without key collisions",
            "repo filesystem",
        )),
        "OTA_RUNTIME_VERSION_MISMATCH" | "OTA_TOOL_VERSION_MISMATCH" => {
            Some(probe_finding_evidence(
                finding,
                "the installed version did not match the contract requirement",
                "the installed version satisfies the contract requirement",
            ))
        }
        "OTA_RUNTIME_MISSING" | "OTA_TOOL_MISSING" => Some(FindingEvidence {
            observed: String::from("the required runtime or tool was not available"),
            expected: String::from("the required runtime or tool is available on PATH"),
            source: finding_probe_target_owner(finding).to_string(),
            checked_at: String::new(),
            command: String::new(),
            path: String::new(),
        }),
        "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED" => Some(static_finding_evidence(
            "the selected repo path is using fallback runtime/tool declarations for an ecosystem Ota does not yet ship as a managed toolchain",
            "a shipped toolchain provider exists for that ecosystem or the repo intentionally stays on the fallback runtime/tool model",
            "repo_signals",
        )),
        "OTA_RUNTIME_PROBE_FAILED" | "OTA_TOOL_PROBE_FAILED" => Some(probe_finding_evidence(
            finding,
            "the resolved executable could not report a version",
            "the resolved executable reports a version that satisfies the contract",
        )),
        "OTA_RUNTIME_VERSION_UNPARSEABLE" | "OTA_TOOL_VERSION_UNPARSEABLE" => {
            Some(probe_finding_evidence(
                finding,
                "the resolved executable did not emit a parseable version",
                "the resolved executable emits a parseable version that satisfies the contract",
            ))
        }
        "OTA_NATIVE_PREREQUISITE_MISSING" | "OTA_NATIVE_PREREQUISITE_TIMED_OUT" => {
            Some(static_finding_evidence(
                "the selected native prerequisite check did not pass",
                "the selected native prerequisite check passes",
                "host",
            ))
        }
        _ => None,
    };

    FindingResolvedMetadata {
        category: "environment",
        owner: match code {
            "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED" => finding_probe_target_owner(finding),
            "OTA_ENV_SOURCE_MISSING_REQUIRED"
            | "OTA_ENV_SOURCE_PARSE_FAILED"
            | "OTA_ENV_SOURCE_INVALID_STRUCTURE"
            | "OTA_ENV_SOURCE_KEY_COLLISION" => "repo_contract",
            _ => finding_probe_target_owner(finding),
        },
        correlation_surfaces: match code {
            "OTA_ENV_MISSING"
            | "OTA_ENV_INVALID"
            | "OTA_ENV_SOURCE_MISSING_REQUIRED"
            | "OTA_ENV_SOURCE_PARSE_FAILED"
            | "OTA_ENV_SOURCE_INVALID_STRUCTURE"
            | "OTA_ENV_SOURCE_KEY_COLLISION" => &["env"],
            "OTA_NATIVE_PREREQUISITE_MISSING" | "OTA_NATIVE_PREREQUISITE_TIMED_OUT" => {
                &["execution"]
            }
            _ => &["toolchain"],
        },
        correlation_owner_prefix: match code {
            "OTA_ENV_MISSING" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &["Missing environment variable: "],
            )
            .map(|variable| format!("env.vars.{variable}")),
            "OTA_NATIVE_PREREQUISITE_MISSING" | "OTA_NATIVE_PREREQUISITE_TIMED_OUT" => {
                finding_suffix_after_prefix(
                    finding.summary.trim(),
                    &[
                        "Native prerequisite missing: ",
                        "Native prerequisite timed out: ",
                    ],
                )
                .map(|name| format!("native_prerequisites.{name}"))
            }
            "OTA_RUNTIME_MISSING" | "OTA_RUNTIME_VERSION_MISMATCH" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &["Missing runtime: ", "Version mismatch for runtime: "],
            )
            .map(|value| value.split_whitespace().next().unwrap_or(value))
            .map(|runtime| format!("toolchains.{runtime}")),
            "OTA_TOOL_MISSING" => {
                finding_suffix_after_prefix(finding.summary.trim(), &["Missing tool: "])
                    .map(|tool| format!("tools.{tool}"))
            }
            _ => None,
        },
        correlation_entity: match code {
            "OTA_ENV_MISSING" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &["Missing environment variable: "],
            )
            .map(str::to_string),
            "OTA_NATIVE_PREREQUISITE_MISSING" | "OTA_NATIVE_PREREQUISITE_TIMED_OUT" => {
                finding_suffix_after_prefix(
                    finding.summary.trim(),
                    &[
                        "Native prerequisite missing: ",
                        "Native prerequisite timed out: ",
                    ],
                )
                .map(str::to_string)
            }
            "OTA_RUNTIME_MISSING" | "OTA_RUNTIME_VERSION_MISMATCH" => finding_suffix_after_prefix(
                finding.summary.trim(),
                &["Missing runtime: ", "Version mismatch for runtime: "],
            )
            .map(|value| value.split_whitespace().next().unwrap_or(value).to_string()),
            "OTA_TOOL_MISSING" => {
                finding_suffix_after_prefix(finding.summary.trim(), &["Missing tool: "])
                    .map(str::to_string)
            }
            _ => None,
        },
        evidence,
        provenance: if code == "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED" {
            repo_signals_provenance()
        } else {
            repo_contract_provenance()
        },
        policy: None,
    }
}

fn resolve_contractless_environment_finding_metadata(
    finding: &Finding,
) -> FindingResolvedMetadata<'_> {
    let evidence = match finding.code() {
        "OTA_CONTRACTLESS_HOST_TOOL_AVAILABLE" => Some(static_finding_evidence(
            "the inferred host tool is already available on PATH",
            "the inferred host tool remains available on PATH",
            "host",
        )),
        "OTA_CONTRACTLESS_HOST_TOOL_MISSING" => Some(static_finding_evidence(
            "the inferred host tool is not available on PATH",
            "the inferred host tool is available on PATH",
            "host",
        )),
        _ => None,
    };

    FindingResolvedMetadata {
        category: "environment",
        owner: "host",
        correlation_surfaces: &["toolchain"],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence,
        provenance: repo_signals_provenance(),
        policy: None,
    }
}

fn resolve_provisioning_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let code = finding.code();
    let (observed, expected, source) = match code {
        "OTA_CONTAINER_APT_VERSION_UNAVAILABLE" => (
            "the configured container apt sources do not provide the pinned package version",
            "the configured container apt sources provide the pinned package version",
            "container_apt",
        ),
        "OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE" => (
            "the configured container apt sources do not provide the requested package",
            "the configured container apt sources provide the requested package",
            "container_apt",
        ),
        "OTA_CONTAINER_APT_INDEX_UNAVAILABLE" => (
            "the configured container apt sources could not refresh indexes",
            "the configured container apt sources refresh successfully",
            "container_apt",
        ),
        "OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE" => (
            "the configured container provisioning backend could not provide the pinned version",
            "the configured container provisioning backend provides the pinned version",
            "container_provisioning",
        ),
        "OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE" => (
            "the configured container provisioning backend could not provide the requested package",
            "the configured container provisioning backend provides the requested package",
            "container_provisioning",
        ),
        "OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE" => (
            "the configured container provisioning backend could not refresh its sources",
            "the configured container provisioning backend refreshes its sources successfully",
            "container_provisioning",
        ),
        "OTA_CONTAINER_PROVISIONING_BACKEND_FAILED" => (
            "the configured container provisioning backend could not satisfy the requested prerequisite",
            "the configured container provisioning backend satisfies the requested prerequisite",
            "container_provisioning",
        ),
        "OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE" => (
            "the configured host provisioning backend could not provide the pinned version",
            "the configured host provisioning backend provides the pinned version",
            "host_provisioning",
        ),
        "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE" => (
            "the configured host provisioning backend could not provide the requested package",
            "the configured host provisioning backend provides the requested package",
            "host_provisioning",
        ),
        "OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE" => (
            "the configured host provisioning backend could not refresh its sources",
            "the configured host provisioning backend refreshes its sources successfully",
            "host_provisioning",
        ),
        "OTA_HOST_PROVISIONING_BACKEND_FAILED" => (
            "the configured host provisioning backend could not satisfy the requested prerequisite",
            "the configured host provisioning backend satisfies the requested prerequisite",
            "host_provisioning",
        ),
        "OTA_REMOTE_APT_VERSION_UNAVAILABLE" => (
            "the configured remote apt sources do not provide the pinned package version",
            "the configured remote apt sources provide the pinned package version",
            "remote_provisioning",
        ),
        "OTA_REMOTE_APT_PACKAGE_UNAVAILABLE" => (
            "the configured remote apt sources do not provide the requested package",
            "the configured remote apt sources provide the requested package",
            "remote_provisioning",
        ),
        "OTA_REMOTE_APT_INDEX_UNAVAILABLE" => (
            "the configured remote apt sources could not refresh indexes",
            "the configured remote apt sources refresh successfully",
            "remote_provisioning",
        ),
        "OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE" => (
            "the configured remote provisioning backend could not provide the pinned version",
            "the configured remote provisioning backend provides the pinned version",
            "remote_provisioning",
        ),
        "OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE" => (
            "the configured remote provisioning backend could not provide the requested package",
            "the configured remote provisioning backend provides the requested package",
            "remote_provisioning",
        ),
        "OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE" => (
            "the configured remote provisioning backend could not refresh its sources",
            "the configured remote provisioning backend refreshes its sources successfully",
            "remote_provisioning",
        ),
        _ => (
            "the configured remote provisioning backend could not satisfy the requested prerequisite",
            "the configured remote provisioning backend satisfies the requested prerequisite",
            "remote_provisioning",
        ),
    };
    let owner = match code {
        "OTA_CONTAINER_APT_VERSION_UNAVAILABLE"
        | "OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE"
        | "OTA_CONTAINER_APT_INDEX_UNAVAILABLE"
        | "OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE"
        | "OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE"
        | "OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE"
        | "OTA_CONTAINER_PROVISIONING_BACKEND_FAILED" => "container_target",
        "OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE"
        | "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE"
        | "OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE"
        | "OTA_HOST_PROVISIONING_BACKEND_FAILED" => "host",
        _ => "remote_target",
    };

    FindingResolvedMetadata {
        category: "provisioning",
        owner,
        correlation_surfaces: &["policy"],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence: Some(static_finding_evidence(observed, expected, source)),
        provenance: org_policy_provenance(),
        policy: None,
    }
}

fn resolve_policy_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let code = finding.code();
    let evidence = match code {
        "OTA_POLICY_PACK_VIOLATION" => Some(static_finding_evidence(
            "the repo failed org policy validation",
            "the repo satisfies the org policy pack",
            "org_policy",
        )),
        "OTA_POLICY_PACK_INVALID" => Some(static_finding_evidence(
            "the org policy pack failed to load or validate",
            "the org policy pack loads and validates",
            "org_policy",
        )),
        "OTA_POLICY_BACKED_VERSION_RULES_DECLARED" => Some(static_finding_evidence(
            "the org policy pack declares approved repo version rules",
            "the org policy pack has no approved repo version rules declared",
            "org_policy",
        )),
        "OTA_POLICY_BACKED_PROVISIONING_DECLARED" => Some(static_finding_evidence(
            "the org policy pack declares approved provisioning sources",
            "the org policy pack has no provisioning sources declared",
            "org_policy",
        )),
        "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING" => Some(static_finding_evidence(
            "policy-backed provisioning is missing required package identifiers",
            "policy-backed provisioning rules declare required package identifiers for OS package managers",
            "org_policy",
        )),
        "OTA_POLICY_NATIVE_PACKAGE_NOT_APPROVED" => Some(static_finding_evidence(
            "the repo requires a host package that is not approved by org policy",
            "the repo's required host packages are approved by org policy",
            "org_policy",
        )),
        "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED" => Some(static_finding_evidence(
            "the org policy pack declares approved adapter bootstrap sources",
            "the org policy pack has no adapter bootstrap sources declared",
            "org_policy",
        )),
        _ => Some(static_finding_evidence(
            "the installed version satisfies the repo contract but violates strict org policy",
            "the installed version also complies with strict org policy",
            "org_policy",
        )),
    };
    let policy = match code {
        "OTA_POLICY_PACK_VIOLATION" => {
            let has_sections = finding.why.contains("missing contract sections:");
            let has_files = finding.why.contains("missing files:");
            let reason = match (has_sections, has_files) {
                (true, true) => "missing_required_sections_and_files",
                (true, false) => "missing_required_sections",
                (false, true) => "missing_required_files",
                (false, false) => "org_policy_pack_violation",
            };
            Some(PolicyFindingContext {
                outcome: "blocked_by_policy",
                reason,
                source: "org",
                install_scope: "repo_local",
                mutation_allowed: false,
            })
        }
        "OTA_POLICY_BACKED_VERSION_RULES_DECLARED" => Some(PolicyFindingContext {
            outcome: "policy_surface_available",
            reason: "policy_backed_version_rules_declared",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: false,
        }),
        "OTA_POLICY_BACKED_PROVISIONING_DECLARED" => Some(PolicyFindingContext {
            outcome: "policy_surface_available",
            reason: "policy_backed_provisioning_declared",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: false,
        }),
        "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED" => Some(PolicyFindingContext {
            outcome: "policy_surface_available",
            reason: "policy_backed_adapter_bootstrap_declared",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: false,
        }),
        "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING" => Some(PolicyFindingContext {
            outcome: "blocked_by_policy",
            reason: "missing_package_identifiers",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: false,
        }),
        "OTA_POLICY_NATIVE_PACKAGE_NOT_APPROVED" => Some(PolicyFindingContext {
            outcome: "blocked_by_policy",
            reason: "native_package_not_approved",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: false,
        }),
        "OTA_POLICY_INSTALLED_VERSION_NONCOMPLIANT" => Some(PolicyFindingContext {
            outcome: "blocked_by_policy",
            reason: "strict_version_noncompliance",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: false,
        }),
        _ => Some(PolicyFindingContext {
            outcome: "blocked_by_integrity_policy",
            reason: "invalid_org_policy_pack",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: false,
        }),
    };

    FindingResolvedMetadata {
        category: "policy",
        owner: "org_policy",
        correlation_surfaces: &["policy"],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence,
        provenance: org_policy_provenance(),
        policy,
    }
}

fn resolve_policy_effect_finding_metadata(finding: &Finding) -> FindingResolvedMetadata<'_> {
    let policy = match finding.code() {
        "OTA_POLICY_EFFECT_ALLOWED" => PolicyFindingContext {
            outcome: "allowed_by_policy",
            reason: "effect_allowed",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: true,
        },
        "OTA_POLICY_EFFECT_WARNED" => PolicyFindingContext {
            outcome: "warned_by_policy",
            reason: "effect_warned",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: true,
        },
        _ => PolicyFindingContext {
            outcome: "blocked_by_policy",
            reason: "effect_denied",
            source: "org",
            install_scope: "repo_local",
            mutation_allowed: false,
        },
    };

    FindingResolvedMetadata {
        category: "policy",
        owner: "org_policy",
        correlation_surfaces: &["policy"],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence: Some(static_finding_evidence(
            "org policy evaluated a requested task effect",
            "org policy keeps the requested task effect decision explicit",
            "org_policy",
        )),
        provenance: org_policy_provenance(),
        policy: Some(policy),
    }
}

fn resolve_adapter_bootstrap_failure_metadata(_: &Finding) -> FindingResolvedMetadata<'_> {
    FindingResolvedMetadata {
        category: "provisioning",
        owner: "repo_contract",
        correlation_surfaces: &["execution"],
        correlation_owner_prefix: None,
        correlation_entity: None,
        evidence: Some(static_finding_evidence(
            "the declared adapter bootstrap path did not complete in the selected execution environment",
            "the declared adapter bootstrap path completes in the selected execution environment",
            "repo_contract",
        )),
        provenance: repo_contract_provenance(),
        policy: None,
    }
}

fn finding_registry_entry(code: &str) -> Option<FindingRegistryEntry> {
    let resolver = match code {
        "OTA_TASKS_MISSING"
        | "OTA_REPO_HYGIENE_OTA_STATE_GITIGNORE"
        | "OTA_REPO_HYGIENE_GITIGNORE_UNREADABLE"
        | "OTA_AGENT_BOUNDARY_UNREVIEWED"
        | "OTA_DEVCONTAINER_RUNTIME_DRIFT"
        | "OTA_DEVCONTAINER_PACKAGE_MANAGER_DRIFT"
        | "OTA_CONTRACT_DRIFT"
        | "OTA_CONTRACTLESS_REPO_CONTRACT_MISSING"
        | "OTA_CONTRACTLESS_SIGNAL_INSPECTION_FAILED"
        | "OTA_CONTRACTLESS_SIGNAL"
        | "OTA_SELECTED_TASK_PATH_NETWORK_REQUIRED"
        | "OTA_SELECTED_TASK_PATH_DEPENDENCY_HYDRATION"
        | "OTA_SELECTED_TASK_PATH_EXTERNAL_STATE"
        | "OTA_CONTRACT_ADVISORY_TASK_MUTATES_MANAGED_ISOLATED_PATH" => {
            if matches!(
                code,
                "OTA_CONTRACTLESS_REPO_CONTRACT_MISSING"
                    | "OTA_CONTRACTLESS_SIGNAL_INSPECTION_FAILED"
                    | "OTA_CONTRACTLESS_SIGNAL"
            ) {
                resolve_contractless_finding_metadata
                    as for<'a> fn(&'a Finding) -> FindingResolvedMetadata<'a>
            } else {
                resolve_contract_core_finding_metadata
                    as for<'a> fn(&'a Finding) -> FindingResolvedMetadata<'a>
            }
        }
        "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY"
        | "OTA_LIFECYCLE_EPHEMERAL_ADVISORY"
        | "OTA_CONTAINER_MODE_NOT_CONFIGURED"
        | "OTA_CONTAINER_DOCTOR_HOST_SCOPE_NOTE"
        | "OTA_WORKFLOW_PROBE_FAILED"
        | "OTA_WORKFLOW_PROBE_TIMED_OUT"
        | "OTA_WORKFLOW_SIGNAL_PROBE_FAILED"
        | "OTA_WORKFLOW_SIGNAL_PROBE_TIMED_OUT"
        | "OTA_WORKFLOW_SURFACE_READINESS_FAILED"
        | "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT"
        | "OTA_WORKFLOW_SURFACE_READINESS_UNEVALUABLE"
        | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED"
        | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT"
        | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_UNEVALUABLE"
        | "OTA_CONTEXT_HOST_PLATFORM_UNSUPPORTED"
        | "OTA_CHECK_FAILED"
        | "OTA_CHECK_TIMED_OUT"
        | "OTA_FILE_CHECK_FAILED"
        | "OTA_FILE_CHECK_TIMED_OUT" => resolve_execution_finding_metadata,
        "OTA_BACKEND_CLI_MISSING" | "OTA_CONTAINER_BACKEND_CLI_MISSING" => {
            resolve_backend_cli_finding_metadata
        }
        "OTA_CONTAINER_BACKEND_UNAVAILABLE" | "OTA_CONTAINER_IMAGE_UNAVAILABLE" => {
            resolve_container_backend_finding_metadata
        }
        "OTA_REMOTE_MODE_NOT_CONFIGURED"
        | "OTA_REMOTE_DOCTOR_PARTIAL"
        | "OTA_REMOTE_DOCTOR_HOST_SCOPE_NOTE"
        | "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED"
        | "OTA_REMOTE_TARGET_SUSPICIOUS"
        | "OTA_REMOTE_CONTEXT_UNEXECUTABLE"
        | "OTA_REMOTE_TARGET_OS_UNDETERMINED" => resolve_remote_finding_metadata,
        "OTA_SERVICE_READINESS_CONTEXT_UNEXECUTABLE"
        | "OTA_SERVICE_READINESS_FAILED"
        | "OTA_SERVICE_CHECK_FAILED"
        | "OTA_SERVICE_CHECK_TIMED_OUT"
        | "OTA_SERVICE_UNVERIFIABLE" => resolve_service_finding_metadata,
        "OTA_ENV_MISSING"
        | "OTA_ENV_INVALID"
        | "OTA_CONTRACTLESS_HOST_TOOL_AVAILABLE"
        | "OTA_CONTRACTLESS_HOST_TOOL_MISSING"
        | "OTA_ENV_SOURCE_MISSING_REQUIRED"
        | "OTA_ENV_SOURCE_PARSE_FAILED"
        | "OTA_ENV_SOURCE_INVALID_STRUCTURE"
        | "OTA_ENV_SOURCE_KEY_COLLISION"
        | "OTA_RUNTIME_VERSION_MISMATCH"
        | "OTA_RUNTIME_MISSING"
        | "OTA_RUNTIME_PROBE_FAILED"
        | "OTA_RUNTIME_VERSION_UNPARSEABLE"
        | "OTA_TOOL_VERSION_MISMATCH"
        | "OTA_TOOL_MISSING"
        | "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED"
        | "OTA_TOOLCHAIN_PROVIDER_MISSING"
        | "OTA_TOOLCHAIN_PROVIDER_PROBE_FAILED"
        | "OTA_TOOLCHAIN_COMPONENT_MISSING"
        | "OTA_TOOLCHAIN_TARGET_MISSING"
        | "OTA_TOOL_ACTIVATION_PROVIDER_MISSING"
        | "OTA_TOOL_PROBE_FAILED"
        | "OTA_TOOL_VERSION_UNPARSEABLE"
        | "OTA_NATIVE_PREREQUISITE_MISSING"
        | "OTA_NATIVE_PREREQUISITE_TIMED_OUT" => {
            if matches!(
                code,
                "OTA_CONTRACTLESS_HOST_TOOL_AVAILABLE" | "OTA_CONTRACTLESS_HOST_TOOL_MISSING"
            ) {
                resolve_contractless_environment_finding_metadata
            } else {
                resolve_environment_finding_metadata
            }
        }
        "OTA_CONTAINER_APT_VERSION_UNAVAILABLE"
        | "OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE"
        | "OTA_CONTAINER_APT_INDEX_UNAVAILABLE"
        | "OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE"
        | "OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE"
        | "OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE"
        | "OTA_CONTAINER_PROVISIONING_BACKEND_FAILED"
        | "OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE"
        | "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE"
        | "OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE"
        | "OTA_HOST_PROVISIONING_BACKEND_FAILED"
        | "OTA_REMOTE_APT_VERSION_UNAVAILABLE"
        | "OTA_REMOTE_APT_PACKAGE_UNAVAILABLE"
        | "OTA_REMOTE_APT_INDEX_UNAVAILABLE"
        | "OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE"
        | "OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE"
        | "OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE"
        | "OTA_REMOTE_PROVISIONING_BACKEND_FAILED" => resolve_provisioning_finding_metadata,
        "OTA_POLICY_PACK_VIOLATION"
        | "OTA_POLICY_PACK_INVALID"
        | "OTA_POLICY_EFFECT_ALLOWED"
        | "OTA_POLICY_EFFECT_WARNED"
        | "OTA_POLICY_EFFECT_DENIED"
        | "OTA_POLICY_BACKED_VERSION_RULES_DECLARED"
        | "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
        | "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING"
        | "OTA_POLICY_NATIVE_PACKAGE_NOT_APPROVED"
        | "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED"
        | "OTA_POLICY_INSTALLED_VERSION_NONCOMPLIANT" => {
            if matches!(
                code,
                "OTA_POLICY_EFFECT_ALLOWED"
                    | "OTA_POLICY_EFFECT_WARNED"
                    | "OTA_POLICY_EFFECT_DENIED"
            ) {
                resolve_policy_effect_finding_metadata
            } else {
                resolve_policy_finding_metadata
            }
        }
        "OTA_ADAPTER_BOOTSTRAP_FAILED" => resolve_adapter_bootstrap_failure_metadata,
        s if s.starts_with("OTA_CONTRACT_ADVISORY_") => resolve_contract_core_finding_metadata,
        _ => return None,
    };

    Some(FindingRegistryEntry { resolver })
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct ProvisioningDiagnostics {
    pub plan: ProvisioningPlan,
    pub request: ProvisioningBackendRequest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AdapterBootstrapDiagnostics {
    pub plan: crate::policy_pack::AdapterBootstrapPlan,
    pub request: crate::policy_pack::ProvisioningBackendRequest,
}

#[derive(Debug, Clone)]
struct ContainerProbeContext {
    image: String,
    engine: String,
}

#[derive(Debug, Clone)]
struct RemoteProbeContext {
    context_name: Option<String>,
    backend: ResolvedExecutionBackend,
    target_os: String,
    requirement_surface: RequirementSurface,
    policy_requirement_surface: RequirementSurface,
    provisioning_actions: Vec<ProvisioningAction>,
}

const CONTAINER_PROBE_PATH_MARKER: &str = "__OTA_RESOLVED_PATH__";
const CONTAINER_PROBE_STARTED_MARKER: &str = "__OTA_CONTAINER_PROBE_STARTED__";
const DOCTOR_DEFAULT_SERVICE_READINESS_RETRIES: u32 = 120;
const DOCTOR_WORKFLOW_SURFACE_READINESS_FAILED_RETRIES: u32 = 600;
const DOCTOR_WORKFLOW_SURFACE_READINESS_TIMEOUT_RETRIES: u32 = 30;
const DOCTOR_WORKFLOW_SURFACE_READINESS_INTERVAL_MS: u64 = 200;
const DOCTOR_READINESS_MAX_START_PERIOD_MS: u64 = 5_000;
const DOCTOR_WORKFLOW_SURFACE_MAX_PROBE_TIMEOUT_MS: u64 = 5_000;
const DOCTOR_WORKFLOW_SURFACE_FAILED_RETRY_WINDOW_MS: u64 = 30_000;
const DOCTOR_WORKFLOW_SURFACE_TIMEOUT_RETRY_WINDOW_MS: u64 = 30_000;

#[derive(Debug, Clone, PartialEq, Eq)]
struct CommandVersionProbe {
    command: String,
    resolved_path: Option<PathBuf>,
    probe_started: bool,
    outcome: CommandVersionProbeOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum CommandVersionProbeOutcome {
    Missing,
    ProbeFailed {
        exit_code: Option<i32>,
        error: Option<String>,
    },
    Unparseable,
    Version(String),
}

impl CommandVersionProbe {
    fn version(self) -> Option<String> {
        match self.outcome {
            CommandVersionProbeOutcome::Version(version) => Some(version),
            _ => None,
        }
    }
}

fn provisioning_action_audit_summary(action: &ProvisioningAction) -> String {
    format!(
        "{} {} {} via {}{}",
        action.target_kind,
        action.display_name(),
        action.version_display(),
        action.source,
        action.policy_display_suffix()
    )
}

fn provisioning_diagnosis_requirement_summary(diagnosis: &ProvisioningFailureDiagnosis) -> String {
    let request = format!("`{} {}`", diagnosis.name, diagnosis.requested_version);
    match (
        diagnosis.resolved_version.as_deref(),
        diagnosis.policy_match.as_deref(),
    ) {
        (Some(resolved), Some(rule)) if resolved != diagnosis.requested_version => {
            format!("{request}; policy resolves that to `{resolved}` using rule `{rule}`")
        }
        (Some(resolved), None) if resolved != diagnosis.requested_version => {
            format!("{request}; ota resolves that to `{resolved}`")
        }
        (_, Some(rule)) => format!("{request}; policy approves it using rule `{rule}`"),
        _ => request,
    }
}

fn remote_target_label(context_name: Option<&str>) -> String {
    context_name
        .map(|value| format!("remote context `{value}`"))
        .unwrap_or_else(|| String::from("the selected remote backend"))
}

fn remote_context_summary_suffix(context_name: Option<&str>) -> String {
    context_name
        .map(|value| format!(" (context {value})"))
        .unwrap_or_default()
}

pub(crate) fn provisioning_installability_finding(
    diagnosis: &ProvisioningFailureDiagnosis,
    target: &ProvisioningExecutionTarget,
    rerun_command: &str,
) -> Finding {
    let image = match target {
        ProvisioningExecutionTarget::Container { image, .. } => Some(image.as_str()),
        ProvisioningExecutionTarget::Native | ProvisioningExecutionTarget::Remote { .. } => None,
    };
    let image_hint = image
        .map(|value| format!(" (currently `{value}`)"))
        .unwrap_or_default();
    let remote_context = match target {
        ProvisioningExecutionTarget::Remote { context_name, .. } => context_name.as_deref(),
        _ => None,
    };
    let remote_suffix = remote_context_summary_suffix(remote_context);
    let remote_label = remote_target_label(remote_context);
    let owner = match target {
        ProvisioningExecutionTarget::Container { .. } => "container_target",
        ProvisioningExecutionTarget::Native => "host",
        ProvisioningExecutionTarget::Remote { .. } => "remote_target",
    };

    let (code, summary, why, next) = match (&target, diagnosis.backend.as_str(), diagnosis.kind) {
        (
            ProvisioningExecutionTarget::Container { .. },
            "apt",
            ProvisioningFailureKind::VersionUnavailable,
        ) => (
            "OTA_CONTAINER_APT_VERSION_UNAVAILABLE",
            format!(
                "Container apt cannot install pinned package version: {}",
                diagnosis.name
            ),
            format!(
                "the Linux/container target requests {}, but the configured apt sources do not provide that version",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!(
                "update the selected container image{image_hint} or its apt sources, or relax the Linux/container version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Container { .. },
            "apt",
            ProvisioningFailureKind::PackageUnavailable,
        ) => (
            "OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE",
            format!(
                "Container apt cannot locate required package: {}",
                diagnosis.name
            ),
            format!(
                "the Linux/container target requests `{}`, but the configured apt sources do not provide that package",
                diagnosis.name
            ),
            format!(
                "update the selected container image{image_hint} or its apt sources so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Container { .. },
            "apt",
            ProvisioningFailureKind::IndexUnavailable,
        ) => (
            "OTA_CONTAINER_APT_INDEX_UNAVAILABLE",
            format!(
                "Container apt cannot refresh configured sources: {}",
                diagnosis.name
            ),
            format!(
                "the Linux/container target could not refresh apt indexes, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!(
                "fix apt repository access in the selected container image{image_hint}, then rerun `{rerun_command}`"
            ),
        ),
        (
            ProvisioningExecutionTarget::Container { .. },
            backend,
            ProvisioningFailureKind::VersionUnavailable,
        ) => (
            "OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE",
            format!(
                "Container {backend} cannot install pinned version: {}",
                diagnosis.name
            ),
            format!(
                "the Linux/container target requests {}, but the configured `{backend}` provisioning path does not provide that version inside container image `{}`",
                provisioning_diagnosis_requirement_summary(diagnosis),
                image.unwrap_or("unknown")
            ),
            format!(
                "fix the selected container image{image_hint} or the configured `{backend}` provisioning path, or relax the version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Container { .. },
            backend,
            ProvisioningFailureKind::PackageUnavailable,
        ) => (
            "OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE",
            format!(
                "Container {backend} cannot locate required package: {}",
                diagnosis.name
            ),
            format!(
                "the Linux/container target requests `{}`, but the configured `{backend}` provisioning path does not provide that package inside container image `{}`",
                diagnosis.name,
                image.unwrap_or("unknown")
            ),
            format!(
                "fix the selected container image{image_hint} or the configured `{backend}` provisioning path so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Container { .. },
            backend,
            ProvisioningFailureKind::IndexUnavailable,
        ) => (
            "OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE",
            format!(
                "Container {backend} cannot refresh configured sources: {}",
                diagnosis.name
            ),
            format!(
                "the Linux/container target could not refresh the configured `{backend}` sources, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!(
                "fix `{backend}` repository access in the selected container image{image_hint}, then rerun `{rerun_command}`"
            ),
        ),
        (ProvisioningExecutionTarget::Container { .. }, backend, _) => (
            "OTA_CONTAINER_PROVISIONING_BACKEND_FAILED",
            format!(
                "Container {backend} cannot install requested prerequisite: {}",
                diagnosis.name
            ),
            format!(
                "the Linux/container target requests {}, but the configured `{backend}` provisioning path could not satisfy it inside container image `{}`",
                provisioning_diagnosis_requirement_summary(diagnosis),
                image.unwrap_or("unknown")
            ),
            format!(
                "fix the selected container image{image_hint} or the configured `{backend}` provisioning path for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Native,
            backend,
            ProvisioningFailureKind::VersionUnavailable,
        ) => (
            "OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE",
            format!(
                "Host {backend} cannot install pinned version: {}",
                diagnosis.name
            ),
            format!(
                "the host target requests {}, but the configured `{backend}` provisioning path could not provide that version",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!(
                "fix the host `{backend}` provisioning path or relax the version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Native,
            backend,
            ProvisioningFailureKind::PackageUnavailable,
        ) => (
            "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE",
            format!(
                "Host {backend} cannot locate required package: {}",
                diagnosis.name
            ),
            format!(
                "the host target requests `{}`, but the configured `{backend}` provisioning path does not provide that package",
                diagnosis.name
            ),
            format!(
                "fix the host `{backend}` provisioning path so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Native,
            backend,
            ProvisioningFailureKind::IndexUnavailable,
        ) => (
            "OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE",
            format!(
                "Host {backend} cannot refresh configured sources: {}",
                diagnosis.name
            ),
            format!(
                "the host target could not refresh the configured `{backend}` sources, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!("fix the host `{backend}` repository access, then rerun `{rerun_command}`"),
        ),
        (ProvisioningExecutionTarget::Native, backend, _) => (
            "OTA_HOST_PROVISIONING_BACKEND_FAILED",
            format!(
                "Host {backend} cannot install requested prerequisite: {}",
                diagnosis.name
            ),
            format!(
                "the host target requests {}, but the configured `{backend}` provisioning path could not satisfy it",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!(
                "fix the host `{backend}` provisioning path for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Remote { .. },
            "apt",
            ProvisioningFailureKind::VersionUnavailable,
        ) => (
            "OTA_REMOTE_APT_VERSION_UNAVAILABLE",
            format!(
                "Remote apt cannot install pinned package version: {}{}",
                diagnosis.name, remote_suffix
            ),
            format!(
                "{remote_label} requests {}, but the configured apt sources do not provide that version",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!(
                "fix the remote apt sources or relax the version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Remote { .. },
            "apt",
            ProvisioningFailureKind::PackageUnavailable,
        ) => (
            "OTA_REMOTE_APT_PACKAGE_UNAVAILABLE",
            format!(
                "Remote apt cannot locate required package: {}{}",
                diagnosis.name, remote_suffix
            ),
            format!(
                "{remote_label} requests `{}`, but the configured apt sources do not provide that package",
                diagnosis.name
            ),
            format!(
                "fix the remote apt sources so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Remote { .. },
            "apt",
            ProvisioningFailureKind::IndexUnavailable,
        ) => (
            "OTA_REMOTE_APT_INDEX_UNAVAILABLE",
            format!(
                "Remote apt cannot refresh configured sources: {}{}",
                diagnosis.name, remote_suffix
            ),
            format!(
                "{remote_label} could not refresh apt indexes, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!("fix remote apt repository access, then rerun `{rerun_command}`"),
        ),
        (
            ProvisioningExecutionTarget::Remote { .. },
            backend,
            ProvisioningFailureKind::VersionUnavailable,
        ) => (
            "OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE",
            format!(
                "Remote {backend} cannot install pinned version: {}{}",
                diagnosis.name, remote_suffix
            ),
            format!(
                "{remote_label} requests {}, but the configured `{backend}` provisioning path does not provide that version",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!(
                "fix the remote `{backend}` provisioning path or relax the version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Remote { .. },
            backend,
            ProvisioningFailureKind::PackageUnavailable,
        ) => (
            "OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE",
            format!(
                "Remote {backend} cannot locate required package: {}{}",
                diagnosis.name, remote_suffix
            ),
            format!(
                "{remote_label} requests `{}`, but the configured `{backend}` provisioning path does not provide that package",
                diagnosis.name
            ),
            format!(
                "fix the remote `{backend}` provisioning path so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
        (
            ProvisioningExecutionTarget::Remote { .. },
            backend,
            ProvisioningFailureKind::IndexUnavailable,
        ) => (
            "OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE",
            format!(
                "Remote {backend} cannot refresh configured sources: {}{}",
                diagnosis.name, remote_suffix
            ),
            format!(
                "{remote_label} could not refresh the configured `{backend}` sources, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!("fix the remote `{backend}` repository access, then rerun `{rerun_command}`"),
        ),
        (ProvisioningExecutionTarget::Remote { .. }, backend, _) => (
            "OTA_REMOTE_PROVISIONING_BACKEND_FAILED",
            format!(
                "Remote {backend} cannot install requested prerequisite: {}{}",
                diagnosis.name, remote_suffix
            ),
            format!(
                "{remote_label} requests {}, but the configured `{backend}` provisioning path could not satisfy it",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            format!(
                "fix the remote `{backend}` provisioning path for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        ),
    };

    Finding::identified(
        code,
        "provisioning",
        owner,
        FindingSeverity::Error,
        summary,
        why,
        next,
    )
}

impl FindingIdentity {
    pub(crate) fn new(code: &str, category: &str, owner: &str) -> Self {
        Self {
            code: code.to_string(),
            category: category.to_string(),
            owner: owner.to_string(),
        }
    }

    fn from_contract_advisory(advisory: &ContractAdvisory) -> Self {
        Self {
            code: advisory.code().to_string(),
            category: advisory.category().to_string(),
            owner: advisory.owner().to_string(),
        }
    }
}

impl Finding {
    pub(crate) fn identified(
        code: &str,
        category: &str,
        owner: &str,
        severity: FindingSeverity,
        summary: impl Into<String>,
        why: impl Into<String>,
        next: impl Into<String>,
    ) -> Self {
        Self {
            identity: Some(FindingIdentity::new(code, category, owner)),
            severity,
            summary: summary.into(),
            why: why.into(),
            next: next.into(),
        }
    }

    fn generic_evidence(&self) -> FindingEvidence {
        FindingEvidence {
            observed: self.summary.clone(),
            expected: self.why.clone(),
            source: String::from("doctor"),
            checked_at: String::new(),
            command: String::new(),
            path: String::new(),
        }
    }

    fn resolved_metadata(&self) -> Option<FindingResolvedMetadata<'_>> {
        finding_registry_entry(self.code()).map(|entry| (entry.resolver)(self))
    }

    fn policy_context(&self) -> Option<PolicyFindingContext<'_>> {
        self.resolved_metadata()
            .and_then(|metadata| metadata.policy)
    }

    pub(crate) fn code(&self) -> &str {
        if let Some(identity) = self.identity.as_ref() {
            return identity.code.as_str();
        }
        match self.summary.as_str() {
            "No tasks defined in contract" => "OTA_TASKS_MISSING",
            "Repo local Ota artifacts are not ignored by git" => {
                "OTA_REPO_HYGIENE_OTA_STATE_GITIGNORE"
            }
            "Ephemeral lifecycle is execution-only" => "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY",
            "Ephemeral lifecycle is advisory in native mode" => "OTA_LIFECYCLE_EPHEMERAL_ADVISORY",
            s if s.starts_with("Missing execution backend CLI: ") => "OTA_BACKEND_CLI_MISSING",
            s if s.starts_with("Missing container execution backend CLI: ") => {
                "OTA_CONTAINER_BACKEND_CLI_MISSING"
            }
            s if s.starts_with("Unsupported remote execution backend provider: ") => {
                "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED"
            }
            s if s.starts_with("Suspicious remote target for ") => "OTA_REMOTE_TARGET_SUSPICIOUS",
            "Remote execution contexts are only partially evaluated in native mode" => {
                "OTA_REMOTE_DOCTOR_PARTIAL"
            }
            s if s.starts_with("Remote execution context is not executable: ") => {
                "OTA_REMOTE_CONTEXT_UNEXECUTABLE"
            }
            s if s.starts_with("Remote target operating system could not be determined") => {
                "OTA_REMOTE_TARGET_OS_UNDETERMINED"
            }
            s if s.starts_with("Service readiness context is not executable: ") => {
                "OTA_SERVICE_READINESS_CONTEXT_UNEXECUTABLE"
            }
            s if s.starts_with("Service readiness failed: ") => "OTA_SERVICE_READINESS_FAILED",
            s if s.starts_with("Service producer is not ready: ") => "OTA_SERVICE_CHECK_FAILED",
            s if s.starts_with("Service healthcheck failed: ") => "OTA_SERVICE_CHECK_FAILED",
            s if s.starts_with("Service healthcheck timed out: ") => "OTA_SERVICE_CHECK_TIMED_OUT",
            s if s.starts_with("Required service cannot be verified: ") => {
                "OTA_SERVICE_UNVERIFIABLE"
            }
            s if s.starts_with("Missing environment variable: ") => "OTA_ENV_MISSING",
            s if s.starts_with("Invalid environment value: ") => "OTA_ENV_INVALID",
            s if s.starts_with("Version mismatch for runtime: ") => "OTA_RUNTIME_VERSION_MISMATCH",
            s if s.starts_with("Missing runtime: ") => "OTA_RUNTIME_MISSING",
            s if s.starts_with("Runtime probe failed: ") => "OTA_RUNTIME_PROBE_FAILED",
            s if s.starts_with("Unparseable version for runtime: ") => {
                "OTA_RUNTIME_VERSION_UNPARSEABLE"
            }
            s if s.starts_with("Version mismatch for tool: ") => "OTA_TOOL_VERSION_MISMATCH",
            s if s.starts_with("Missing tool: ") => "OTA_TOOL_MISSING",
            s if s.starts_with("Managed toolchain opportunity: ") => {
                "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED"
            }
            s if s.starts_with("Missing toolchain provider: ") => "OTA_TOOLCHAIN_PROVIDER_MISSING",
            s if s.starts_with("Toolchain provider probe failed: ") => {
                "OTA_TOOLCHAIN_PROVIDER_PROBE_FAILED"
            }
            s if s.starts_with("Missing toolchain component: ") => {
                "OTA_TOOLCHAIN_COMPONENT_MISSING"
            }
            s if s.starts_with("Missing toolchain target: ") => "OTA_TOOLCHAIN_TARGET_MISSING",
            s if s.starts_with("Missing tool activation provider: ") => {
                "OTA_TOOL_ACTIVATION_PROVIDER_MISSING"
            }
            s if s.starts_with("Tool probe failed: ") => "OTA_TOOL_PROBE_FAILED",
            s if s.starts_with("Unparseable version for tool: ") => "OTA_TOOL_VERSION_UNPARSEABLE",
            s if s.starts_with("Native prerequisite missing: ") => {
                "OTA_NATIVE_PREREQUISITE_MISSING"
            }
            s if s.starts_with("Native prerequisite timed out: ") => {
                "OTA_NATIVE_PREREQUISITE_TIMED_OUT"
            }
            s if s.starts_with("Container apt cannot install pinned package version: ") => {
                "OTA_CONTAINER_APT_VERSION_UNAVAILABLE"
            }
            s if s.starts_with("Container apt cannot locate required package: ") => {
                "OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE"
            }
            s if s.starts_with("Container apt cannot refresh configured sources: ") => {
                "OTA_CONTAINER_APT_INDEX_UNAVAILABLE"
            }
            s if s.starts_with("Container ") && s.contains(" cannot install pinned version: ") => {
                "OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE"
            }
            s if s.starts_with("Container ") && s.contains(" cannot locate required package: ") => {
                "OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE"
            }
            s if s.starts_with("Container ")
                && s.contains(" cannot refresh configured sources: ") =>
            {
                "OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE"
            }
            s if s.starts_with("Container ")
                && s.contains(" cannot install requested prerequisite: ") =>
            {
                "OTA_CONTAINER_PROVISIONING_BACKEND_FAILED"
            }
            s if s.starts_with("Host ") && s.contains(" cannot install pinned version: ") => {
                "OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE"
            }
            s if s.starts_with("Host ") && s.contains(" cannot locate required package: ") => {
                "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE"
            }
            s if s.starts_with("Host ") && s.contains(" cannot refresh configured sources: ") => {
                "OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE"
            }
            s if s.starts_with("Host ")
                && s.contains(" cannot install requested prerequisite: ") =>
            {
                "OTA_HOST_PROVISIONING_BACKEND_FAILED"
            }
            s if s.starts_with("Remote apt cannot install pinned package version: ") => {
                "OTA_REMOTE_APT_VERSION_UNAVAILABLE"
            }
            s if s.starts_with("Remote apt cannot locate required package: ") => {
                "OTA_REMOTE_APT_PACKAGE_UNAVAILABLE"
            }
            s if s.starts_with("Remote apt cannot refresh configured sources: ") => {
                "OTA_REMOTE_APT_INDEX_UNAVAILABLE"
            }
            s if s.starts_with("Remote ") && s.contains(" cannot install pinned version: ") => {
                "OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE"
            }
            s if s.starts_with("Remote ") && s.contains(" cannot locate required package: ") => {
                "OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE"
            }
            s if s.starts_with("Remote ") && s.contains(" cannot refresh configured sources: ") => {
                "OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE"
            }
            s if s.starts_with("Remote ")
                && s.contains(" cannot install requested prerequisite: ") =>
            {
                "OTA_REMOTE_PROVISIONING_BACKEND_FAILED"
            }
            "Repo does not satisfy org policy pack" => "OTA_POLICY_PACK_VIOLATION",
            "Invalid org policy pack" => "OTA_POLICY_PACK_INVALID",
            "Policy-backed provisioning sources are declared" => {
                "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
            }
            "Policy provisioning needs explicit package identifiers" => {
                "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING"
            }
            s if s.starts_with("Org policy does not approve native package: ") => {
                "OTA_POLICY_NATIVE_PACKAGE_NOT_APPROVED"
            }
            "Adapter bootstrap sources are declared" => {
                "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED"
            }
            s if s.starts_with("Check failed: ") => "OTA_CHECK_FAILED",
            s if s.starts_with("Check timed out: ") => "OTA_CHECK_TIMED_OUT",
            s if s.starts_with("File check failed: ") => "OTA_FILE_CHECK_FAILED",
            s if s.starts_with("File check timed out: ") => "OTA_FILE_CHECK_TIMED_OUT",
            s if s.starts_with("Contract drift:") => "OTA_CONTRACT_DRIFT",
            s if s.starts_with("Task `")
                && s.contains(" depends_on `")
                && s.contains(" across different execution boundaries") =>
            {
                "OTA_CONTRACT_ADVISORY_DEPENDS_ON_BOUNDARY"
            }
            s if s.starts_with("Attachment `") && s.contains(" may be unused in context `") => {
                "OTA_CONTRACT_ADVISORY_LIKELY_UNUSED_ATTACHMENT"
            }
            s if s.starts_with("Isolated path `")
                && s.contains(" may shadow required Yarn release artifacts in context `") =>
            {
                "OTA_CONTRACT_ADVISORY_ISOLATED_YARN_RELEASE_SHADOW"
            }
            s if s.starts_with("Node contract uses split ownership (`runtimes.node` + tools: ") => {
                "OTA_CONTRACT_ADVISORY_LEGACY_NODE_RUNTIME_TOOL_SPLIT"
            }
            s if s.starts_with("Poetry is modeled as a standalone tool (") => {
                "OTA_CONTRACT_ADVISORY_LEGACY_STANDALONE_POETRY"
            }
            s if s.starts_with("task `")
                && s.contains(" uses opaque shell `")
                && s.contains(" for long-running service path `") =>
            {
                "OTA_CONTRACT_ADVISORY_SERVICE_OPAQUE_SHELL_START"
            }
            s if s.starts_with("task `")
                && s.contains(" uses replaceable shell `")
                && s.contains(" instead of `command`") =>
            {
                "OTA_CONTRACT_ADVISORY_REPLACEABLE_FINITE_SHELL_COMMAND"
            }
            s if s.starts_with("task `")
                && s.contains(" hard-codes dependency hydration in its task body") =>
            {
                "OTA_CONTRACT_ADVISORY_REPLACEABLE_DEPENDENCY_HYDRATION"
            }
            s if s.starts_with("task `")
                && s.contains(" declares exceptional dependency hydration override `")
                && s.contains(" for `") =>
            {
                "OTA_CONTRACT_ADVISORY_EXCEPTIONAL_DEPENDENCY_HYDRATION_OVERRIDE"
            }
            s if s.starts_with("check `")
                && s.contains(" uses replaceable shell file glue for `") =>
            {
                "OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_FILE_CHECK"
            }
            s if s.starts_with("check `")
                && s.contains(" uses replaceable shell env-file glue for `") =>
            {
                "OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_ENV_CHECK"
            }
            s if s.starts_with("task `")
                && s.contains(" uses replaceable shell env-file mutation") =>
            {
                "OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_ENV_MUTATION"
            }
            s if s.starts_with("task `")
                && s.contains(" hard-codes compose adapter input ownership in its task body") =>
            {
                "OTA_CONTRACT_ADVISORY_REPLACEABLE_COMPOSE_ENV_FILE_OWNERSHIP"
            }
            s if s.starts_with("task `")
                && s.contains(" hard-codes Bake file selection in its task body") =>
            {
                "OTA_CONTRACT_ADVISORY_REPLACEABLE_BAKE_FILE_OWNERSHIP"
            }
            s if s.starts_with("native prerequisite `")
                && s.contains(" platform `")
                && s.contains(" declares likely wrong-OS package manager `") =>
            {
                "OTA_CONTRACT_ADVISORY_NATIVE_PACKAGE_MANAGER_LIKELY_WRONG_PLATFORM"
            }
            s if s.starts_with("native prerequisite `")
                && s.contains(" platform `")
                && s.contains(" mixes manual install glue with manager-owned package truth") =>
            {
                "OTA_CONTRACT_ADVISORY_MIXED_NATIVE_PACKAGE_OWNERSHIP"
            }
            s if s.starts_with("workflow `")
                && s.contains(" profile `")
                && s.contains(" duplicates rendered env artifact ownership in task `") =>
            {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_RENDERED_ENV_OWNERSHIP"
            }
            s if s.starts_with("workflow `")
                && s.contains(" duplicates compose `env_files` ownership in task `") =>
            {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_ENV_FILES_OWNERSHIP"
            }
            s if s.starts_with("workflow `")
                && s.contains(" duplicates compose `files` ownership in task `") =>
            {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_FILES_OWNERSHIP"
            }
            s if s.starts_with("workflow `")
                && s.contains(" duplicates compose `profiles` ownership in task `") =>
            {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROFILES_OWNERSHIP"
            }
            s if s.starts_with("workflow `")
                && s.contains(" duplicates compose `project_name` ownership in task `") =>
            {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROJECT_NAME_OWNERSHIP"
            }
            s if s.starts_with("workflow `")
                && s.contains(" duplicates bake `files` ownership in task `") =>
            {
                "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_BAKE_FILES_OWNERSHIP"
            }
            s if s.starts_with("`agent.writable_paths` includes sensitive ") => {
                "OTA_CONTRACT_ADVISORY_SENSITIVE_AGENT_WRITABLE_PATH"
            }
            s if s
                .starts_with("`agent.exceptions.sensitive_writes` includes unnecessary path `") =>
            {
                "OTA_CONTRACT_ADVISORY_SENSITIVE_WRITE_EXCEPTION"
            }
            s if s.starts_with("task `")
                && s.contains(" uses non-canonical external-state token `") =>
            {
                "OTA_CONTRACT_ADVISORY_EXTERNAL_STATE_TOKEN_CANONICAL"
            }
            s if s.starts_with("task `")
                && s.contains(" materializes git checkout `")
                && s.contains(" without explicit `source.ref`") =>
            {
                "OTA_CONTRACT_ADVISORY_ENSURE_GIT_CHECKOUT_MOVING_HEAD"
            }
            s if s.starts_with("`agent.bootstrap.ota.")
                && s.ends_with("` should pin the ota release version") =>
            {
                "OTA_CONTRACT_ADVISORY_AGENT_BOOTSTRAP_UNPINNED"
            }
            s if s.starts_with("Agent-safe task `")
                && s.contains(" performs network dependency hydration") =>
            {
                "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_DEPENDENCY_HYDRATION"
            }
            s if s.starts_with("Agent-safe task `")
                && s.contains(" performs network integration testing") =>
            {
                "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_INTEGRATION_TEST"
            }
            s if s.starts_with("Agent-safe task `") && s.contains(" requires network access") => {
                "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_NETWORK"
            }
            s if s.starts_with("Agent-safe task `") && s.contains(" mutates external state: ") => {
                "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_EXTERNAL_STATE"
            }
            s if s.starts_with("Task `") && s.contains(" mutates managed isolated path `") => {
                "OTA_CONTRACT_ADVISORY_TASK_MUTATES_MANAGED_ISOLATED_PATH"
            }
            _ => "OTA_DOCTOR_FINDING_UNKNOWN",
        }
    }

    pub(crate) fn category(&self) -> &str {
        if let Some(metadata) = self.resolved_metadata() {
            metadata.category
        } else if let Some(identity) = self.identity.as_ref() {
            identity.category.as_str()
        } else {
            "contract"
        }
    }

    pub(crate) fn owner(&self) -> &str {
        if let Some(metadata) = self.resolved_metadata() {
            metadata.owner
        } else if let Some(identity) = self.identity.as_ref() {
            identity.owner.as_str()
        } else {
            "repo_contract"
        }
    }

    pub(crate) fn correlation_surfaces(&self) -> &'static [&'static str] {
        if let Some(metadata) = self.resolved_metadata() {
            metadata.correlation_surfaces
        } else {
            &[]
        }
    }

    pub(crate) fn correlation_owner_prefix(&self) -> Option<String> {
        self.resolved_metadata()
            .and_then(|metadata| metadata.correlation_owner_prefix.clone())
    }

    pub(crate) fn correlation_entity(&self) -> Option<String> {
        self.resolved_metadata()
            .and_then(|metadata| metadata.correlation_entity.clone())
    }

    fn evidence(&self) -> FindingEvidence {
        self.resolved_metadata()
            .and_then(|metadata| metadata.evidence)
            .unwrap_or_else(|| self.generic_evidence())
    }

    fn drift_context(&self) -> Option<DriftFindingContext<'_>> {
        if self.code() == "OTA_CONTRACT_DRIFT" {
            Some(DriftFindingContext {
                owner_kind: "merged",
                ownership: "repo_contract",
                provenance: "repo signals were compared against ota-managed contract fields with `ota detect`",
                provenance_key: "repo_signals",
            })
        } else {
            None
        }
    }

    fn toolchain_opportunity_context(&self) -> Option<ToolchainOpportunityContext<'static>> {
        let ecosystem = self
            .summary
            .strip_prefix("Managed toolchain opportunity: ")?;
        unsupported_toolchain_opportunity_context(ecosystem.trim())
    }

    fn provenance_context(&self) -> Option<FindingProvenanceContext<'_>> {
        if let Some(drift) = self.drift_context() {
            return Some(FindingProvenanceContext {
                provenance: drift.provenance,
                provenance_key: drift.provenance_key,
            });
        }

        if let Some(metadata) = self.resolved_metadata()
            && let Some(provenance) = metadata.provenance
        {
            return Some(provenance);
        }

        if self.code() == "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED" {
            return Some(FindingProvenanceContext {
                provenance: "repo signals",
                provenance_key: "repo_signals",
            });
        }

        let summary = self.summary.as_str();
        if matches!(
            summary,
            "Contract missing"
                | "Could not inspect repo signals"
                | "Detected Rust repo"
                | "No strong repo signals were detected yet"
                | "Detected repo type: Node"
        ) || summary.starts_with("Detected Docker Compose services: ")
            || summary.starts_with("Detected package manager: ")
            || summary.starts_with("Detected likely runnable tasks: ")
            || summary.starts_with("Host tool available: ")
            || summary.starts_with("Missing host tool: ")
            || (summary.starts_with("Missing container execution backend CLI: ")
                && self.why.contains("Docker Compose signals were detected"))
        {
            return Some(FindingProvenanceContext {
                provenance: "repo signals",
                provenance_key: "repo_signals",
            });
        }

        None
    }

    pub(crate) fn provenance(&self) -> Option<String> {
        self.provenance_context()
            .map(|context| context.provenance.to_string())
    }

    pub(crate) fn provenance_key(&self) -> Option<String> {
        self.provenance_context()
            .map(|context| context.provenance_key.to_string())
    }
}

fn policy_finding(
    code: &str,
    severity: FindingSeverity,
    summary: impl Into<String>,
    why: impl Into<String>,
    next: impl Into<String>,
) -> Finding {
    Finding::identified(code, "policy", "org_policy", severity, summary, why, next)
}

fn remote_backend_finding(
    code: &str,
    severity: FindingSeverity,
    summary: impl Into<String>,
    why: impl Into<String>,
    next: impl Into<String>,
) -> Finding {
    Finding::identified(
        code,
        "remote",
        "remote_backend",
        severity,
        summary,
        why,
        next,
    )
}

impl Serialize for Finding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let policy = self.policy_context();
        let drift = self.drift_context();
        let toolchain_opportunity = self.toolchain_opportunity_context();
        let provenance = self.provenance_context();
        let mut state = serializer.serialize_struct(
            "Finding",
            8 + policy.map(|_| 5).unwrap_or_default()
                + drift.map(|_| 2).unwrap_or_default()
                + toolchain_opportunity.map(|_| 1).unwrap_or_default()
                + provenance.map(|_| 2).unwrap_or_default(),
        )?;

        state.serialize_field("code", self.code())?;
        state.serialize_field("category", self.category())?;
        state.serialize_field("owner", self.owner())?;
        state.serialize_field("severity", &self.severity)?;
        state.serialize_field("summary", &self.summary)?;
        state.serialize_field("why", &self.why)?;
        state.serialize_field("next", &self.next)?;
        state.serialize_field("evidence", &self.evidence())?;

        if let Some(policy) = policy {
            state.serialize_field("policy_outcome", policy.outcome)?;
            state.serialize_field("policy_reason", policy.reason)?;
            state.serialize_field("policy_source", policy.source)?;
            state.serialize_field("install_scope", policy.install_scope)?;
            state.serialize_field("mutation_allowed", &policy.mutation_allowed)?;
        }

        if let Some(drift) = drift {
            state.serialize_field("owner_kind", drift.owner_kind)?;
            state.serialize_field("ownership", drift.ownership)?;
        }

        if let Some(toolchain_opportunity) = toolchain_opportunity {
            #[derive(Serialize)]
            struct ToolchainOpportunityJson<'a> {
                ecosystem: &'a str,
                fallback_runtime: &'a str,
                fallback_tools: &'a [&'a str],
                candidate_providers: &'a [&'a str],
                shipped: bool,
                agent_note: &'a str,
            }

            state.serialize_field(
                "toolchain_opportunity",
                &ToolchainOpportunityJson {
                    ecosystem: toolchain_opportunity.ecosystem,
                    fallback_runtime: toolchain_opportunity.fallback_runtime,
                    fallback_tools: toolchain_opportunity.fallback_tools,
                    candidate_providers: toolchain_opportunity.candidate_providers,
                    shipped: false,
                    agent_note: toolchain_opportunity.agent_note,
                },
            )?;
        }

        if let Some(provenance) = provenance {
            state.serialize_field("provenance", provenance.provenance)?;
            state.serialize_field("provenance_key", provenance.provenance_key)?;
        }

        state.end()
    }
}

impl<'de> Deserialize<'de> for Finding {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        #[derive(Deserialize)]
        struct FindingJson {
            #[serde(default)]
            identity: Option<FindingIdentity>,
            #[serde(default)]
            code: Option<String>,
            #[serde(default)]
            category: Option<String>,
            #[serde(default)]
            owner: Option<String>,
            severity: FindingSeverity,
            summary: String,
            why: String,
            next: String,
        }

        let finding = FindingJson::deserialize(deserializer)?;
        let identity = finding.identity.or_else(|| {
            let code = finding.code?;
            let category = finding.category?;
            let owner = finding.owner?;
            if code.is_empty() || category.is_empty() || owner.is_empty() {
                None
            } else {
                Some(FindingIdentity {
                    code,
                    category,
                    owner,
                })
            }
        });

        Ok(Finding {
            identity,
            severity: finding.severity,
            summary: finding.summary,
            why: finding.why,
            next: finding.next,
        })
    }
}

#[derive(Debug, PartialEq, Eq, Serialize)]
pub struct DoctorReport {
    pub ok: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provisioning: Option<ProvisioningDiagnostics>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub adapter_bootstrap: Option<AdapterBootstrapDiagnostics>,
    #[serde(skip)]
    pub execution_target: Option<String>,
    pub findings: Vec<Finding>,
}

#[derive(Debug)]
pub struct PolicyReviewReport {
    pub policy: Option<LoadedOrgPolicyPack>,
    pub report: DoctorReport,
}

pub fn diagnose_contract(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::All,
        DoctorMode::Native,
        None,
        None,
        ExecutionOverrides::default(),
        None,
    )
}

pub fn diagnose_contract_in_mode(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::All,
        mode,
        None,
        None,
        ExecutionOverrides::default(),
        None,
    )
}

pub fn diagnose_contract_with_mode_and_lifecycle(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
    lifecycle_override: Option<Lifecycle>,
) -> DoctorReport {
    diagnose_contract_with_mode_and_lifecycle_for_workflow(
        contract,
        contract_path,
        mode,
        lifecycle_override,
        None,
    )
}

pub fn diagnose_contract_with_mode_and_lifecycle_for_workflow(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
    lifecycle_override: Option<Lifecycle>,
    workflow_name: Option<&str>,
) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::All,
        mode,
        lifecycle_override,
        workflow_name,
        ExecutionOverrides::default(),
        None,
    )
}

pub fn diagnose_contract_with_mode_and_lifecycle_for_workflow_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
    lifecycle_override: Option<Lifecycle>,
    workflow_name: Option<&str>,
    overrides: ExecutionOverrides,
) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::All,
        mode,
        lifecycle_override,
        workflow_name,
        overrides,
        None,
    )
}

pub fn diagnose_policy_review(contract: &Contract, contract_path: &Path) -> PolicyReviewReport {
    let mut findings = Vec::new();
    let loaded_policy = match load_org_policy_pack_auto_details(contract_path) {
        Ok(policy) => policy,
        Err(err) => {
            findings.push(policy_error_finding(err));
            None
        }
    };

    if let Some(loaded_policy_ref) = loaded_policy.as_ref() {
        let requirement_surface = contract.all_requirement_surface();
        let toolchain_names = contract.toolchains.keys().cloned().collect();
        diagnose_org_policy(
            contract,
            contract_path,
            Some(loaded_policy_ref),
            current_os(),
            &requirement_surface,
            &toolchain_names,
            &BTreeSet::new(),
            &mut findings,
        );
        diagnose_adapter_bootstrap(Some(loaded_policy_ref), &mut findings);
    }

    let report = DoctorReport {
        ok: !findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error),
        provisioning: None,
        adapter_bootstrap: None,
        execution_target: None,
        findings,
    };

    PolicyReviewReport {
        policy: loaded_policy,
        report,
    }
}

pub fn diagnose_preconditions(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_preconditions_with_mode(contract, contract_path, DoctorMode::Native)
}

pub fn diagnose_preconditions_with_mode(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
) -> DoctorReport {
    diagnose_preconditions_with_mode_for_workflow(contract, contract_path, mode, None)
}

pub fn diagnose_preconditions_with_mode_for_workflow(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
    workflow_name: Option<&str>,
) -> DoctorReport {
    diagnose_preconditions_with_mode_for_workflow_with_overrides(
        contract,
        contract_path,
        mode,
        workflow_name,
        ExecutionOverrides::default(),
    )
}

pub fn diagnose_preconditions_with_mode_for_workflow_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
    workflow_name: Option<&str>,
    overrides: ExecutionOverrides,
) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::Preconditions,
        mode,
        None,
        workflow_name,
        overrides,
        None,
    )
}

pub fn diagnose_preconditions_with_mode_for_task_with_overrides(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::Preconditions,
        mode,
        None,
        None,
        overrides,
        Some(task_name),
    )
}

pub fn diagnose_checks_only(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_checks_only_for_workflow(contract, contract_path, None)
}

pub fn diagnose_checks_only_for_workflow(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::ChecksOnly,
        DoctorMode::Native,
        None,
        workflow_name,
        ExecutionOverrides::default(),
        None,
    )
}

pub fn diagnose_services_only(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_services_only_for_workflow(contract, contract_path, None)
}

pub fn diagnose_services_only_for_workflow(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::ServicesOnly,
        DoctorMode::Native,
        None,
        workflow_name,
        ExecutionOverrides::default(),
        None,
    )
}

pub fn diagnose_service(contract: &Contract, contract_path: &Path, name: &str) -> DoctorReport {
    let mut findings = Vec::new();
    let working_dir = contract_working_dir(contract_path);

    if let Some(service) = contract.services.get(name)
        && let Some(finding) = service_finding(
            contract,
            contract_path,
            name,
            service,
            working_dir,
            doctor_mode_for_service(contract, service),
            None,
        )
    {
        findings.push(finding);
    }

    DoctorReport {
        ok: !findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error),
        provisioning: None,
        adapter_bootstrap: None,
        execution_target: None,
        findings,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DoctorScope {
    All,
    Preconditions,
    ChecksOnly,
    ServicesOnly,
}

fn diagnose_contract_with_scope(
    contract: &Contract,
    contract_path: &Path,
    scope: DoctorScope,
    mode: DoctorMode,
    lifecycle_override: Option<Lifecycle>,
    workflow_name: Option<&str>,
    overrides: ExecutionOverrides,
    task_name: Option<&str>,
) -> DoctorReport {
    let mut findings = Vec::new();
    let mut provisioning = None;
    let mut adapter_bootstrap = None;
    let mut execution_target = None;
    let selected_lifecycle = doctor_selected_lifecycle(mode, lifecycle_override);
    let empty_env_names = BTreeSet::<String>::new();
    let backend_precondition_selections = if let Some(task_name) = task_name {
        selected_task_backend_precondition_selections(contract, task_name, overrides)
    } else {
        selected_backend_precondition_selections(contract, workflow_name, overrides)
    };
    let precondition_selection = backend_precondition_selections
        .iter()
        .find(|selection| selection.backend == backend_for_mode(mode))
        .cloned()
        .map(ScopedPreconditionSelection::from)
        .unwrap_or_else(|| scoped_precondition_selection(contract, mode, workflow_name));
    let requirement_surface = precondition_selection.requirement_surface.clone();
    let loaded_policy = if matches!(scope, DoctorScope::All | DoctorScope::Preconditions) {
        match load_org_policy_pack_auto_details(contract_path) {
            Ok(policy) => policy,
            Err(err) => {
                findings.push(policy_error_finding(err));
                None
            }
        }
    } else {
        None
    };
    let provisioning_actions = if mode == DoctorMode::Remote {
        Vec::new()
    } else {
        loaded_policy
            .as_ref()
            .map(|loaded| {
                let policy_requirement_surface = policy_requirement_surface_for_toolchains(
                    contract,
                    &requirement_surface,
                    &precondition_selection.toolchain_names,
                    policy_target_os_for_mode(mode),
                );
                loaded
                    .pack
                    .selected_provisioning_actions_for_requirement_surface_os(
                        policy_target_os_for_mode(mode),
                        &policy_requirement_surface,
                    )
            })
            .unwrap_or_default()
    };
    if let Some(finding) = detect_missing_ota_state_gitignore(contract_path) {
        findings.push(finding);
    }
    if mode == DoctorMode::Native
        && let Some(finding) =
            detect_devcontainer_runtime_drift(contract, contract_path, &requirement_surface)
    {
        findings.push(finding);
    }
    if mode == DoctorMode::Native
        && let Some(finding) = detect_devcontainer_package_manager_drift(contract, contract_path)
    {
        findings.push(finding);
    }

    if matches!(scope, DoctorScope::All | DoctorScope::Preconditions) {
        let unsupported_host_findings = unsupported_host_context_findings(
            contract,
            mode,
            selected_lifecycle,
            workflow_name,
            overrides,
        );
        if !unsupported_host_findings.is_empty() {
            findings.extend(unsupported_host_findings);
            dedupe_findings_preserve_order(&mut findings);
            return DoctorReport {
                ok: false,
                provisioning: None,
                adapter_bootstrap: None,
                execution_target: None,
                findings,
            };
        }
        diagnose_lifecycle(contract, mode, selected_lifecycle, &mut findings);
        let container_probe = diagnose_execution_backend(
            contract,
            &mut findings,
            mode,
            selected_lifecycle,
            overrides,
            None,
        );
        let declared_env_sources = load_declared_env_sources(contract, contract_path);
        diagnose_env_sources(&declared_env_sources, &mut findings);
        if mode == DoctorMode::Native {
            diagnose_env(
                contract,
                loaded_policy
                    .as_ref()
                    .map(|loaded| loaded.pack.env_values()),
                &declared_env_sources,
                precondition_selection
                    .env_scoped
                    .then_some(&precondition_selection.env_names),
                &mut findings,
            );
        }
        if mode == DoctorMode::Container
            && contract_has_host_bound_readiness_surfaces(contract)
            && !precondition_selection.env_scoped
        {
            findings.push(container_mode_scope_note_finding(contract));
        }
        let mut container_execution_target_probe = container_probe.clone();
        let container_probe_started = if mode == DoctorMode::Remote {
            if let Some(note) = remote_mode_host_scope_note_finding(contract) {
                findings.push(note);
            }
            let mut remote_probe_contexts = Vec::new();
            if !has_remote_backend_blocker(&findings) {
                remote_probe_contexts = remote_doctor_probe_contexts(
                    contract,
                    contract_path,
                    loaded_policy.as_ref(),
                    workflow_name,
                    &mut findings,
                );
                for remote_probe in &remote_probe_contexts {
                    let required_tools = remote_probe
                        .requirement_surface
                        .tools
                        .keys()
                        .cloned()
                        .collect::<BTreeSet<_>>();
                    diagnose_runtimes(
                        &remote_probe.requirement_surface.runtimes,
                        &remote_probe.target_os,
                        contract_path,
                        loaded_policy.as_ref(),
                        mode,
                        selected_lifecycle,
                        None,
                        Some(&remote_probe.backend),
                        remote_probe.context_name.as_deref(),
                        &remote_probe.provisioning_actions,
                        &mut findings,
                    );
                    diagnose_tools(
                        contract,
                        &precondition_selection.toolchain_names,
                        &requirement_surface_with_toolchain_owned_tools_for_required_tools(
                            contract,
                            &remote_probe.requirement_surface,
                            &precondition_selection.toolchain_names,
                            &remote_probe.target_os,
                            Some(&required_tools),
                        ),
                        false,
                        &remote_probe.target_os,
                        contract_path,
                        loaded_policy.as_ref(),
                        mode,
                        selected_lifecycle,
                        None,
                        Some(&remote_probe.backend),
                        remote_probe.context_name.as_deref(),
                        &remote_probe.provisioning_actions,
                        &mut findings,
                    );
                    diagnose_toolchains(
                        contract,
                        &precondition_selection.toolchain_names,
                        &remote_probe.target_os,
                        contract_path,
                        mode,
                        None,
                        Some(&remote_probe.backend),
                        remote_probe.context_name.as_deref(),
                        &mut findings,
                    );
                }
            }
            diagnose_remote_org_policy(
                contract,
                contract_path,
                loaded_policy.as_ref(),
                &remote_probe_contexts,
                &mut findings,
            );
            false
        } else if mode == DoctorMode::Container {
            let active_container_selections: Vec<&BackendPreconditionSelection> =
                backend_precondition_selections
                    .iter()
                    .filter(|selection| selection.backend == Backend::Container)
                    .collect();
            let container_selections = if active_container_selections.is_empty() {
                vec![BackendPreconditionSelection {
                    backend: Backend::Container,
                    context_name: None,
                    requirement_surface: requirement_surface.clone(),
                    toolchain_names: precondition_selection.toolchain_names.clone(),
                    native_names: precondition_selection.native_names.clone(),
                    env_names: precondition_selection.env_names.clone(),
                    env_scoped: precondition_selection.env_scoped,
                    dependency_hydration_owned: precondition_selection.dependency_hydration_owned,
                }]
            } else {
                active_container_selections.into_iter().cloned().collect()
            };
            let mut any_probe_started = false;
            for selection in &container_selections {
                let selection_container_probe = diagnose_execution_backend(
                    contract,
                    &mut findings,
                    mode,
                    selected_lifecycle,
                    ExecutionOverrides {
                        backend: Some(Backend::Container),
                        ..overrides
                    },
                    selection.context_name.as_deref(),
                );
                let required_tools = selection
                    .requirement_surface
                    .tools
                    .keys()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                let provisioning_requirement_surface =
                    requirement_surface_with_toolchain_owned_tools_for_required_tools(
                        contract,
                        &selection.requirement_surface,
                        &selection.toolchain_names,
                        policy_target_os_for_mode(mode),
                        Some(&required_tools),
                    );
                let selection_provisioning_actions =
                    merged_provisioning_actions_for_requirement_surface(
                        contract,
                        loaded_policy
                            .as_ref()
                            .map(|loaded| {
                                let policy_requirement_surface =
                                    policy_requirement_surface_for_toolchains(
                                        contract,
                                        &selection.requirement_surface,
                                        &selection.toolchain_names,
                                        policy_target_os_for_mode(mode),
                                    );
                                loaded
                                    .pack
                                    .selected_provisioning_actions_for_requirement_surface_os(
                                        policy_target_os_for_mode(mode),
                                        &policy_requirement_surface,
                                    )
                            })
                            .unwrap_or_default(),
                        &provisioning_requirement_surface,
                        policy_target_os_for_mode(mode),
                    );
                diagnose_env(
                    contract,
                    loaded_policy
                        .as_ref()
                        .map(|loaded| loaded.pack.env_values()),
                    &declared_env_sources,
                    if !selection.env_scoped {
                        Some(&empty_env_names)
                    } else {
                        Some(&selection.env_names)
                    },
                    &mut findings,
                );
                let runtime_probe_started = diagnose_runtimes(
                    &selection.requirement_surface.runtimes,
                    policy_target_os_for_mode(mode),
                    contract_path,
                    loaded_policy.as_ref(),
                    mode,
                    selected_lifecycle,
                    selection_container_probe.as_ref(),
                    None,
                    None,
                    &selection_provisioning_actions,
                    &mut findings,
                );
                let tool_probe_started = diagnose_tools(
                    contract,
                    &selection.toolchain_names,
                    &provisioning_requirement_surface,
                    selection.dependency_hydration_owned,
                    policy_target_os_for_mode(mode),
                    contract_path,
                    loaded_policy.as_ref(),
                    mode,
                    selected_lifecycle,
                    selection_container_probe.as_ref(),
                    None,
                    None,
                    &selection_provisioning_actions,
                    &mut findings,
                );
                let toolchain_probe_started = diagnose_toolchains(
                    contract,
                    &selection.toolchain_names,
                    policy_target_os_for_mode(mode),
                    contract_path,
                    mode,
                    selection_container_probe.as_ref(),
                    None,
                    None,
                    &mut findings,
                );
                if container_execution_target_probe.is_none() && selection_container_probe.is_some()
                {
                    container_execution_target_probe = selection_container_probe.clone();
                }
                any_probe_started |=
                    runtime_probe_started || tool_probe_started || toolchain_probe_started;
            }
            any_probe_started
        } else {
            let runtime_probe_started = diagnose_runtimes(
                &requirement_surface.runtimes,
                policy_target_os_for_mode(mode),
                contract_path,
                loaded_policy.as_ref(),
                mode,
                selected_lifecycle,
                container_probe.as_ref(),
                None,
                None,
                &provisioning_actions,
                &mut findings,
            );
            let required_tools = requirement_surface
                .tools
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>();
            let tool_probe_started = diagnose_tools(
                contract,
                &precondition_selection.toolchain_names,
                &requirement_surface_with_toolchain_owned_tools_for_required_tools(
                    contract,
                    &requirement_surface,
                    &precondition_selection.toolchain_names,
                    policy_target_os_for_mode(mode),
                    Some(&required_tools),
                ),
                precondition_selection.dependency_hydration_owned,
                policy_target_os_for_mode(mode),
                contract_path,
                loaded_policy.as_ref(),
                mode,
                selected_lifecycle,
                container_probe.as_ref(),
                None,
                None,
                &provisioning_actions,
                &mut findings,
            );
            let toolchain_probe_started = diagnose_toolchains(
                contract,
                &precondition_selection.toolchain_names,
                policy_target_os_for_mode(mode),
                contract_path,
                mode,
                container_probe.as_ref(),
                None,
                None,
                &mut findings,
            );
            if mode == DoctorMode::Native {
                diagnose_native_prerequisites(
                    contract,
                    contract_path,
                    &precondition_selection.native_names,
                    policy_target_os_for_mode(mode),
                    &mut findings,
                );
            }
            for additional_selection in backend_precondition_selections
                .iter()
                .filter(|selection| selection.backend != backend_for_mode(mode))
            {
                let additional_mode = match additional_selection.backend {
                    Backend::Native => DoctorMode::Native,
                    Backend::Container => DoctorMode::Container,
                    Backend::Remote => DoctorMode::Remote,
                };
                if additional_mode == DoctorMode::Remote {
                    continue;
                }
                let additional_lifecycle =
                    doctor_selected_lifecycle(additional_mode, lifecycle_override);
                let additional_container_probe = if additional_mode == DoctorMode::Container {
                    diagnose_execution_backend(
                        contract,
                        &mut findings,
                        additional_mode,
                        additional_lifecycle,
                        ExecutionOverrides {
                            backend: Some(Backend::Container),
                            ..overrides
                        },
                        additional_selection.context_name.as_deref(),
                    )
                } else {
                    None
                };
                let required_tools = additional_selection
                    .requirement_surface
                    .tools
                    .keys()
                    .cloned()
                    .collect::<BTreeSet<_>>();
                let provisioning_requirement_surface =
                    requirement_surface_with_toolchain_owned_tools_for_required_tools(
                        contract,
                        &additional_selection.requirement_surface,
                        &additional_selection.toolchain_names,
                        policy_target_os_for_mode(additional_mode),
                        Some(&required_tools),
                    );
                let additional_provisioning_actions =
                    merged_provisioning_actions_for_requirement_surface(
                        contract,
                        loaded_policy
                            .as_ref()
                            .map(|loaded| {
                                let policy_requirement_surface =
                                    policy_requirement_surface_for_toolchains(
                                        contract,
                                        &additional_selection.requirement_surface,
                                        &additional_selection.toolchain_names,
                                        policy_target_os_for_mode(additional_mode),
                                    );
                                loaded
                                    .pack
                                    .selected_provisioning_actions_for_requirement_surface_os(
                                        policy_target_os_for_mode(additional_mode),
                                        &policy_requirement_surface,
                                    )
                            })
                            .unwrap_or_default(),
                        &provisioning_requirement_surface,
                        policy_target_os_for_mode(additional_mode),
                    );
                if matches!(additional_mode, DoctorMode::Native | DoctorMode::Container) {
                    diagnose_env(
                        contract,
                        loaded_policy
                            .as_ref()
                            .map(|loaded| loaded.pack.env_values()),
                        &declared_env_sources,
                        if additional_mode == DoctorMode::Container
                            && !additional_selection.env_scoped
                        {
                            Some(&empty_env_names)
                        } else {
                            additional_selection
                                .env_scoped
                                .then_some(&additional_selection.env_names)
                        },
                        &mut findings,
                    );
                }
                diagnose_runtimes(
                    &additional_selection.requirement_surface.runtimes,
                    policy_target_os_for_mode(additional_mode),
                    contract_path,
                    loaded_policy.as_ref(),
                    additional_mode,
                    additional_lifecycle,
                    additional_container_probe.as_ref(),
                    None,
                    None,
                    &additional_provisioning_actions,
                    &mut findings,
                );
                diagnose_tools(
                    contract,
                    &additional_selection.toolchain_names,
                    &provisioning_requirement_surface,
                    additional_selection.dependency_hydration_owned,
                    policy_target_os_for_mode(additional_mode),
                    contract_path,
                    loaded_policy.as_ref(),
                    additional_mode,
                    additional_lifecycle,
                    additional_container_probe.as_ref(),
                    None,
                    None,
                    &additional_provisioning_actions,
                    &mut findings,
                );
                diagnose_toolchains(
                    contract,
                    &additional_selection.toolchain_names,
                    policy_target_os_for_mode(additional_mode),
                    contract_path,
                    additional_mode,
                    additional_container_probe.as_ref(),
                    None,
                    None,
                    &mut findings,
                );
                if additional_mode == DoctorMode::Native {
                    diagnose_native_prerequisites(
                        contract,
                        contract_path,
                        &additional_selection.native_names,
                        policy_target_os_for_mode(additional_mode),
                        &mut findings,
                    );
                }
            }
            runtime_probe_started || tool_probe_started || toolchain_probe_started
        };
        if mode == DoctorMode::Native && contract_has_remote_execution_context(contract) {
            findings.push(remote_mode_scope_note_finding());
        }
        if mode == DoctorMode::Container
            && container_probe_started
            && let Some(container_probe) = container_execution_target_probe.as_ref()
        {
            execution_target = Some(match selected_lifecycle {
                Some(Lifecycle::Persistent) => crate::runner::persistent_container_name(
                    contract_working_dir(contract_path),
                    &container_probe.image,
                    &container_probe.engine,
                ),
                Some(Lifecycle::Ephemeral) | None => crate::runner::ephemeral_container_name(
                    contract_working_dir(contract_path),
                    &container_probe.image,
                    &container_probe.engine,
                ),
            });
        }
        diagnose_unsupported_toolchain_opportunities(
            contract,
            contract_path,
            &requirement_surface,
            &mut findings,
        );
        if mode != DoctorMode::Remote {
            provisioning = diagnose_org_policy(
                contract,
                contract_path,
                loaded_policy.as_ref(),
                policy_target_os_for_mode(mode),
                &requirement_surface,
                &precondition_selection.toolchain_names,
                &precondition_selection.native_names,
                &mut findings,
            );
            let required_tools = requirement_surface
                .tools
                .keys()
                .cloned()
                .collect::<BTreeSet<_>>();
            let provisioning_requirement_surface =
                requirement_surface_with_toolchain_owned_tools_for_required_tools(
                    contract,
                    &requirement_surface,
                    &precondition_selection.toolchain_names,
                    policy_target_os_for_mode(mode),
                    Some(&required_tools),
                );
            provisioning = merge_direct_tool_acquisition_provisioning(
                contract,
                provisioning,
                &provisioning_requirement_surface,
                policy_target_os_for_mode(mode),
            );
        }
        adapter_bootstrap = diagnose_adapter_bootstrap(loaded_policy.as_ref(), &mut findings);
    }
    if scope == DoctorScope::All {
        diagnose_tasks_surface(contract, &mut findings);
        diagnose_agent_boundary_review(contract, &mut findings);
        diagnose_contract_advisories(
            contract,
            contract_path,
            &mut findings,
            overrides,
            workflow_name,
        );
        diagnose_selected_task_effects(contract, workflow_name, &mut findings);
    }
    if scope == DoctorScope::All
        && findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error)
    {
        dedupe_findings_preserve_order(&mut findings);
        findings.sort_by_key(|finding| finding.severity);
        return DoctorReport {
            ok: false,
            provisioning,
            adapter_bootstrap,
            execution_target,
            findings,
        };
    }
    if matches!(scope, DoctorScope::All | DoctorScope::ServicesOnly) {
        diagnose_services(
            contract,
            contract_path,
            mode,
            selected_lifecycle,
            workflow_name,
            &mut findings,
        );
    }
    if scope != DoctorScope::ServicesOnly {
        if mode == DoctorMode::Native {
            diagnose_checks(
                contract,
                contract_path,
                scope,
                workflow_name,
                task_name,
                &mut findings,
                overrides,
            );
        }
    }

    dedupe_findings_preserve_order(&mut findings);
    findings.sort_by_key(|finding| finding.severity);

    DoctorReport {
        ok: !findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error),
        provisioning,
        adapter_bootstrap,
        execution_target,
        findings,
    }
}

fn diagnose_tasks_surface(contract: &Contract, findings: &mut Vec<Finding>) {
    if !contract.tasks.is_empty() {
        return;
    }

    let severity = if project_type_allows_no_tasks(contract) {
        FindingSeverity::Warn
    } else {
        FindingSeverity::Error
    };

    findings.push(Finding::identified(
        "OTA_TASKS_MISSING",
        "contract",
        "repo_contract",
        severity,
        "No tasks defined in contract",
        "without at least one task, `ota run <task>` cannot execute a repo entrypoint and the readiness contract is not operational for humans or agents",
        "run `ota detect --dry-run` to review inferred tasks before writing, or run `ota assist add-task --name dev --kind command` when you want one explicit runnable task",
    ));
}

fn diagnose_contract_advisories(
    contract: &Contract,
    contract_path: &Path,
    findings: &mut Vec<Finding>,
    overrides: ExecutionOverrides,
    workflow_name: Option<&str>,
) {
    let selected_task_names = contract
        .selected_workflow_task_closure_names(workflow_name)
        .into_iter()
        .collect::<BTreeSet<_>>();
    for advisory in collect_contract_advisories_with_contract_path(contract, Some(contract_path)) {
        let advisory = match advisory {
            ContractAdvisory::DependsOnBoundary(advisory) => {
                if let Some(advisory) =
                    normalize_depends_on_boundary_for_overrides(contract, advisory, overrides)
                {
                    ContractAdvisory::DependsOnBoundary(advisory)
                } else {
                    continue;
                }
            }
            ContractAdvisory::LikelyUnusedAttachment(advisory) => {
                ContractAdvisory::LikelyUnusedAttachment(advisory)
            }
            ContractAdvisory::IsolatedYarnReleaseShadow(advisory) => {
                ContractAdvisory::IsolatedYarnReleaseShadow(advisory)
            }
            ContractAdvisory::MutatesManagedIsolatedPath(advisory) => {
                ContractAdvisory::MutatesManagedIsolatedPath(advisory)
            }
            ContractAdvisory::LegacyNodeRuntimeToolSplit(advisory) => {
                ContractAdvisory::LegacyNodeRuntimeToolSplit(advisory)
            }
            ContractAdvisory::LegacyStandalonePoetry(advisory) => {
                ContractAdvisory::LegacyStandalonePoetry(advisory)
            }
            ContractAdvisory::LegacyToolchainProvider(advisory) => {
                ContractAdvisory::LegacyToolchainProvider(advisory)
            }
            ContractAdvisory::LegacyFlatToolchainFulfillment(advisory) => {
                ContractAdvisory::LegacyFlatToolchainFulfillment(advisory)
            }
            ContractAdvisory::LegacyHostServiceLifecycle(advisory) => {
                ContractAdvisory::LegacyHostServiceLifecycle(advisory)
            }
            ContractAdvisory::LegacyServiceReadinessRun(advisory) => {
                ContractAdvisory::LegacyServiceReadinessRun(advisory)
            }
            ContractAdvisory::ServiceUsesOpaqueShellStart(advisory) => {
                ContractAdvisory::ServiceUsesOpaqueShellStart(advisory)
            }
            ContractAdvisory::ReplaceableFiniteShellCommand(advisory) => {
                ContractAdvisory::ReplaceableFiniteShellCommand(advisory)
            }
            ContractAdvisory::ReplaceableDependencyHydrationOwnership(advisory) => {
                ContractAdvisory::ReplaceableDependencyHydrationOwnership(advisory)
            }
            ContractAdvisory::ExceptionalDependencyHydrationOverride(advisory) => {
                ContractAdvisory::ExceptionalDependencyHydrationOverride(advisory)
            }
            ContractAdvisory::ReplaceableShellCheck(advisory) => {
                ContractAdvisory::ReplaceableShellCheck(advisory)
            }
            ContractAdvisory::ReplaceableShellEnvMutation(advisory) => {
                ContractAdvisory::ReplaceableShellEnvMutation(advisory)
            }
            ContractAdvisory::ReplaceableToolBootstrapOwnership(advisory) => {
                ContractAdvisory::ReplaceableToolBootstrapOwnership(advisory)
            }
            ContractAdvisory::ReplaceableSystemdServiceOwnership(advisory) => {
                ContractAdvisory::ReplaceableSystemdServiceOwnership(advisory)
            }
            ContractAdvisory::ReplaceableContainerNetworkOwnership(advisory) => {
                ContractAdvisory::ReplaceableContainerNetworkOwnership(advisory)
            }
            ContractAdvisory::ReplaceableComposeVolumeResetOwnership(advisory) => {
                ContractAdvisory::ReplaceableComposeVolumeResetOwnership(advisory)
            }
            ContractAdvisory::ReplaceableAdapterInputOwnership(advisory) => {
                ContractAdvisory::ReplaceableAdapterInputOwnership(advisory)
            }
            ContractAdvisory::NativePackageManagerLikelyWrongPlatform(advisory) => {
                ContractAdvisory::NativePackageManagerLikelyWrongPlatform(advisory)
            }
            ContractAdvisory::MixedNativePackageOwnership(advisory) => {
                ContractAdvisory::MixedNativePackageOwnership(advisory)
            }
            ContractAdvisory::EmptyAdapterInputMarker(advisory) => {
                ContractAdvisory::EmptyAdapterInputMarker(advisory)
            }
            ContractAdvisory::DuplicateWorkflowRenderedEnvOwnership(advisory) => {
                ContractAdvisory::DuplicateWorkflowRenderedEnvOwnership(advisory)
            }
            ContractAdvisory::DuplicateWorkflowAdapterInputOwnership(advisory) => {
                ContractAdvisory::DuplicateWorkflowAdapterInputOwnership(advisory)
            }
            ContractAdvisory::SensitiveAgentWritablePath(advisory) => {
                ContractAdvisory::SensitiveAgentWritablePath(advisory)
            }
            ContractAdvisory::SensitiveWriteException(advisory) => {
                ContractAdvisory::SensitiveWriteException(advisory)
            }
            ContractAdvisory::NonCanonicalExternalStateToken(advisory) => {
                ContractAdvisory::NonCanonicalExternalStateToken(advisory)
            }
            ContractAdvisory::EnsureGitCheckoutMovingHead(advisory) => {
                ContractAdvisory::EnsureGitCheckoutMovingHead(advisory)
            }
            ContractAdvisory::AgentBootstrapUnpinned(advisory) => {
                ContractAdvisory::AgentBootstrapUnpinned(advisory)
            }
            ContractAdvisory::AgentBootstrapBranchTracking(advisory) => {
                ContractAdvisory::AgentBootstrapBranchTracking(advisory)
            }
            ContractAdvisory::AgentSafeTaskNetwork(advisory) => {
                if !selected_task_names.is_empty()
                    && !selected_task_names.contains(advisory.task_name.as_str())
                {
                    continue;
                }
                ContractAdvisory::AgentSafeTaskNetwork(advisory)
            }
            ContractAdvisory::AgentSafeTaskExternalState(advisory) => {
                if !selected_task_names.is_empty()
                    && !selected_task_names.contains(advisory.task_name.as_str())
                {
                    continue;
                }
                ContractAdvisory::AgentSafeTaskExternalState(advisory)
            }
            ContractAdvisory::MissingIntegrationTestNetworkKind(advisory) => {
                if !selected_task_names.is_empty()
                    && !selected_task_names.contains(advisory.task_name.as_str())
                {
                    continue;
                }
                ContractAdvisory::MissingIntegrationTestNetworkKind(advisory)
            }
        };

        findings.push(contract_advisory_finding(advisory));
    }
}

fn contract_advisory_finding(advisory: ContractAdvisory) -> Finding {
    let summary = match &advisory {
        ContractAdvisory::DependsOnBoundary(advisory) => format!(
            "Task `{}` depends_on `{}` across different execution boundaries",
            advisory.parent_task, advisory.dependency_task
        ),
        ContractAdvisory::LikelyUnusedAttachment(advisory) => format!(
            "Attachment `{}` may be unused in context `{}`",
            advisory.isolated_path, advisory.context_name
        ),
        ContractAdvisory::IsolatedYarnReleaseShadow(advisory) => format!(
            "Isolated path `{}` may shadow required Yarn release artifacts in context `{}`",
            advisory.isolated_path, advisory.context_name
        ),
        ContractAdvisory::MutatesManagedIsolatedPath(advisory) => format!(
            "Task `{}` mutates managed isolated path `{}`",
            advisory.task_name, advisory.isolated_path
        ),
        ContractAdvisory::LegacyNodeRuntimeToolSplit(advisory) => format!(
            "Node contract uses split ownership (`runtimes.node` + tools: {})",
            advisory.package_managers.join(", ")
        ),
        ContractAdvisory::LegacyStandalonePoetry(advisory) => format!(
            "Poetry is modeled as a standalone tool ({})",
            advisory.locations.join(", ")
        ),
        ContractAdvisory::LegacyToolchainProvider(advisory) => format!(
            "Toolchain `{}` keeps legacy provider ownership",
            advisory.toolchain_name
        ),
        ContractAdvisory::LegacyFlatToolchainFulfillment(advisory) => format!(
            "Toolchain `{}` keeps legacy flat fulfillment ownership",
            advisory.toolchain_name
        ),
        ContractAdvisory::LegacyHostServiceLifecycle(advisory) => format!(
            "Service `{}` keeps host lifecycle ownership on legacy top-level field(s)",
            advisory.service_name
        ),
        ContractAdvisory::LegacyServiceReadinessRun(advisory) => format!(
            "Service `{}` keeps readiness ownership on legacy `run`",
            advisory.service_name
        ),
        ContractAdvisory::ServiceUsesOpaqueShellStart(_)
        | ContractAdvisory::ReplaceableFiniteShellCommand(_)
        | ContractAdvisory::ReplaceableDependencyHydrationOwnership(_)
        | ContractAdvisory::ExceptionalDependencyHydrationOverride(_)
        | ContractAdvisory::ReplaceableShellCheck(_)
        | ContractAdvisory::ReplaceableShellEnvMutation(_)
        | ContractAdvisory::ReplaceableToolBootstrapOwnership(_)
        | ContractAdvisory::ReplaceableSystemdServiceOwnership(_)
        | ContractAdvisory::ReplaceableContainerNetworkOwnership(_)
        | ContractAdvisory::ReplaceableComposeVolumeResetOwnership(_)
        | ContractAdvisory::ReplaceableAdapterInputOwnership(_)
        | ContractAdvisory::NativePackageManagerLikelyWrongPlatform(_)
        | ContractAdvisory::MixedNativePackageOwnership(_)
        | ContractAdvisory::EmptyAdapterInputMarker(_)
        | ContractAdvisory::DuplicateWorkflowRenderedEnvOwnership(_)
        | ContractAdvisory::DuplicateWorkflowAdapterInputOwnership(_) => advisory.summary(),
        ContractAdvisory::SensitiveAgentWritablePath(advisory) => format!(
            "`agent.writable_paths` includes sensitive {} `{}`",
            advisory.category, advisory.path
        ),
        ContractAdvisory::SensitiveWriteException(advisory) => format!(
            "`agent.exceptions.sensitive_writes` includes unnecessary path `{}`",
            advisory.path
        ),
        ContractAdvisory::NonCanonicalExternalStateToken(advisory) => format!(
            "Task `{}` uses non-canonical external-state token `{}`",
            advisory.task_name, advisory.token
        ),
        ContractAdvisory::EnsureGitCheckoutMovingHead(advisory) => format!(
            "Task `{}` materializes git checkout `{}` without explicit `source.ref`",
            advisory.task_name, advisory.checkout_path
        ),
        ContractAdvisory::AgentBootstrapUnpinned(advisory) => {
            format!("`{}` should pin the ota release version", advisory.field)
        }
        ContractAdvisory::AgentBootstrapBranchTracking(advisory) => format!(
            "`{}` tracks ota branch `{}` for pressure testing",
            advisory.field, advisory.branch
        ),
        ContractAdvisory::AgentSafeTaskNetwork(advisory) => match advisory.network_kind {
            TaskNetworkEffectKind::DependencyHydration => format!(
                "Agent-safe task `{}` performs network dependency hydration",
                advisory.task_name
            ),
            TaskNetworkEffectKind::IntegrationTest => format!(
                "Agent-safe task `{}` performs network integration testing",
                advisory.task_name
            ),
            TaskNetworkEffectKind::ToolBootstrap => format!(
                "Agent-safe task `{}` performs network tool bootstrap",
                advisory.task_name
            ),
            TaskNetworkEffectKind::Broad => {
                format!(
                    "Agent-safe task `{}` requires network access",
                    advisory.task_name
                )
            }
        },
        ContractAdvisory::AgentSafeTaskExternalState(advisory) => format!(
            "Agent-safe task `{}` mutates external state: {}",
            advisory.task_name,
            advisory.systems.join(", ")
        ),
        ContractAdvisory::MissingIntegrationTestNetworkKind(advisory) => format!(
            "Test task `{}` uses real service verification without `effects.network_kind: integration_test`",
            advisory.task_name
        ),
    };

    Finding {
        identity: Some(FindingIdentity::from_contract_advisory(&advisory)),
        severity: FindingSeverity::Warn,
        summary,
        why: advisory.why(),
        next: advisory.next(),
    }
}

fn diagnose_selected_task_effects(
    contract: &Contract,
    workflow_name: Option<&str>,
    findings: &mut Vec<Finding>,
) {
    let selected_task_names = contract.selected_workflow_task_closure_names(workflow_name);
    if selected_task_names.is_empty() {
        return;
    }

    let mut broad_network_tasks = Vec::new();
    let mut hydration_network_tasks = Vec::new();
    let mut integration_test_network_tasks = Vec::new();
    let mut tool_bootstrap_tasks = Vec::new();
    let mut external_state_tasks = Vec::new();
    let mut external_state_systems = BTreeSet::new();

    for task_name in selected_task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        if let Some(kind) = task.effects.effective_network_kind() {
            match kind {
                TaskNetworkEffectKind::Broad => broad_network_tasks.push(task_name.clone()),
                TaskNetworkEffectKind::DependencyHydration => {
                    hydration_network_tasks.push(task_name.clone())
                }
                TaskNetworkEffectKind::IntegrationTest => {
                    integration_test_network_tasks.push(task_name.clone())
                }
                TaskNetworkEffectKind::ToolBootstrap => {
                    tool_bootstrap_tasks.push(task_name.clone())
                }
            }
        }
        if !task.effects.external_state.is_empty() {
            external_state_tasks.push(task_name.clone());
            for system in &task.effects.external_state {
                external_state_systems.insert(system.clone());
            }
        }
    }

    if !broad_network_tasks.is_empty() {
        findings.push(Finding::identified(
            "OTA_SELECTED_TASK_PATH_NETWORK_REQUIRED",
            "execution",
            "repo_contract",
            FindingSeverity::Info,
            format!(
                "Selected task path requires network access: {}",
                broad_network_tasks.join(", ")
            ),
            "the selected task path includes tasks with `effects.network: true`, so readiness may still depend on registry, API, or remote service reachability even when repo write boundaries are otherwise narrow",
            "treat the selected path as network-dependent in CI and agent execution, and keep `effects.network: true` explicit on those tasks",
        ));
    }

    if !hydration_network_tasks.is_empty() {
        findings.push(Finding::identified(
            "OTA_SELECTED_TASK_PATH_DEPENDENCY_HYDRATION",
            "execution",
            "repo_contract",
            FindingSeverity::Info,
            format!(
                "Selected task path performs network dependency hydration: {}",
                hydration_network_tasks.join(", ")
            ),
            "the selected task path includes tasks with `effects.network_kind: dependency_hydration`; this is a narrower network lane (for example lockfile-backed package-manager fetches), but still depends on registry reachability",
            "keep lockfiles and package-manager provenance strict for these tasks, and keep `effects.network_kind: dependency_hydration` explicit on that path",
        ));
    }

    if !integration_test_network_tasks.is_empty() {
        findings.push(Finding::identified(
            "OTA_SELECTED_TASK_PATH_INTEGRATION_TEST",
            "execution",
            "repo_contract",
            FindingSeverity::Info,
            format!(
                "Selected task path performs network integration testing: {}",
                integration_test_network_tasks.join(", ")
            ),
            "the selected task path includes tasks with `effects.network_kind: integration_test`; this is a narrower network lane for staging, live, or remote-backed verification, but still depends on real service reachability and non-local test credentials or fixtures",
            "keep the live or staging dependency surface explicit through `requirements.env`, `effects.external_state`, and `effects.network_kind: integration_test`, and avoid treating these lanes as routine safe-task execution",
        ));
    }

    if !tool_bootstrap_tasks.is_empty() {
        findings.push(Finding::identified(
            "OTA_SELECTED_TASK_PATH_TOOL_BOOTSTRAP",
            "execution",
            "repo_contract",
            FindingSeverity::Info,
            format!(
                "Selected task path performs network tool bootstrap: {}",
                tool_bootstrap_tasks.join(", ")
            ),
            "the selected task path includes tasks with `effects.network_kind: tool_bootstrap`; this is a narrower network lane for contract-owned tool installation (for example `pip install uv`), but still depends on package index reachability and mutable tool-install state",
            "keep the tool bootstrap source explicit, prefer first-class `prepare.kind: tool_bootstrap` over shell glue, and keep `effects.network_kind: tool_bootstrap` explicit on that path",
        ));
    }

    if !external_state_tasks.is_empty() {
        findings.push(Finding::identified(
            "OTA_SELECTED_TASK_PATH_EXTERNAL_STATE",
            "execution",
            "repo_contract",
            FindingSeverity::Warn,
            format!(
                "Selected task path mutates external state: {}",
                external_state_systems.into_iter().collect::<Vec<_>>().join(", ")
            ),
            format!(
                "the selected task path includes `{}`, which declares `effects.external_state`; repo write boundaries do not cover that out-of-repo mutation",
                external_state_tasks.join(", ")
            ),
            "run the selected path only when those external systems are meant to change, and keep `effects.external_state` explicit on the mutating tasks",
        ));
    }
}

fn normalize_depends_on_boundary_for_overrides(
    contract: &Contract,
    advisory: crate::validator::DependsOnBoundaryAdvisory,
    overrides: ExecutionOverrides,
) -> Option<crate::validator::DependsOnBoundaryAdvisory> {
    if overrides.backend.is_none() && overrides.lifecycle.is_none() {
        return Some(advisory);
    }

    let parent = match effective_boundary_for_task(contract, &advisory.parent_task, overrides) {
        Some(boundary) => boundary,
        None => advisory.parent,
    };
    let dependency =
        match effective_boundary_for_task(contract, &advisory.dependency_task, overrides) {
            Some(boundary) => boundary,
            None => advisory.dependency,
        };

    (parent != dependency).then_some(crate::validator::DependsOnBoundaryAdvisory {
        parent_task: advisory.parent_task,
        dependency_task: advisory.dependency_task,
        parent,
        dependency,
        parent_backend_selection_source: advisory.parent_backend_selection_source,
        dependency_backend_selection_source: advisory.dependency_backend_selection_source,
    })
}

fn effective_boundary_for_task(
    contract: &Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Option<TaskExecutionBoundary> {
    let task = contract.tasks.get(task_name)?;
    let effective = effective_task_execution(contract, task_name, overrides);

    Some(TaskExecutionBoundary {
        context_name: effective.context_name.map(str::to_string),
        backend: effective.backend,
        lifecycle: effective.lifecycle,
        backend_binding: task
            .backend_binding_for_backend(effective.backend)
            .map(str::to_string),
    })
}

fn diagnose_agent_boundary_review(contract: &Contract, findings: &mut Vec<Finding>) {
    let Some(inferred_boundary) = contract
        .agent
        .as_ref()
        .and_then(|agent| agent.inferred_boundary.as_ref())
    else {
        return;
    };

    if inferred_boundary.reviewed {
        return;
    }

    findings.push(Finding::identified(
        "OTA_AGENT_BOUNDARY_UNREVIEWED",
        "contract",
        "repo_contract",
        FindingSeverity::Warn,
        "Agent boundary is inferred and unreviewed",
        "`agent.inferred_boundary.reviewed: false` means Ota inferred the current writable and protected paths, but the repo owner has not confirmed that boundary yet",
        "review `agent.writable_paths` and `agent.protected_paths`, set `agent.inferred_boundary.reviewed: true`, then rerun `ota validate`",
    ));
}

fn project_type_allows_no_tasks(contract: &Contract) -> bool {
    let project_type = contract
        .project
        .project_type
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| value.to_ascii_lowercase());

    match project_type.as_deref() {
        Some("sdk" | "library" | "lib" | "package" | "module" | "plugin") => true,
        _ => false,
    }
}

fn diagnose_lifecycle(
    contract: &Contract,
    mode: DoctorMode,
    selected_lifecycle: Option<Lifecycle>,
    findings: &mut Vec<Finding>,
) {
    let Some(execution) = contract.execution.as_ref() else {
        return;
    };

    if execution.lifecycle != Some(Lifecycle::Ephemeral) {
        return;
    }

    if execution.preferred == Some(Backend::Container) {
        let next = match (mode, selected_lifecycle) {
            (DoctorMode::Container, Some(lifecycle)) => format!(
                "use `ota run <task>` for isolated execution; use `ota up --mode container --lifecycle {}` for readiness only",
                match lifecycle {
                    Lifecycle::Persistent => "persistent",
                    Lifecycle::Ephemeral => "ephemeral",
                }
            ),
            _ => String::from(
                "use `ota run <task>` for isolated execution; use `ota up` for readiness only",
            ),
        };
        findings.push(Finding::identified(
            "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY",
            "execution",
            "repo_contract",
            FindingSeverity::Warn,
            "Ephemeral lifecycle is execution-only",
            "`execution.lifecycle: ephemeral` only applies to task execution. Diagnosis, healthchecks, and teardown are not covered.",
            next,
        ));
    } else {
        findings.push(Finding::identified(
            "OTA_LIFECYCLE_EPHEMERAL_ADVISORY",
            "execution",
            "repo_contract",
            FindingSeverity::Warn,
            "Ephemeral lifecycle is advisory in native mode",
            "`execution.lifecycle: ephemeral` is advisory in native mode only. Native execution still runs in the host shell.",
            "use `ota run <task>` for isolated execution; use `ota up` for readiness only",
        ));
    }
}

fn diagnose_execution_backend(
    contract: &Contract,
    findings: &mut Vec<Finding>,
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
    overrides: ExecutionOverrides,
    context_name_override: Option<&str>,
) -> Option<ContainerProbeContext> {
    let Some(execution) = contract.execution.as_ref() else {
        if mode == DoctorMode::Container {
            findings.push(container_mode_not_configured_finding());
        } else if mode == DoctorMode::Remote {
            findings.push(remote_mode_not_configured_finding());
        }
        return None;
    };

    if mode == DoctorMode::Container {
        if let Some((_, context)) = context_name_override
            .and_then(|context_name| execution.contexts.get_key_value(context_name))
            .map(|(name, context)| (name.as_str(), context))
            .filter(|(_, context)| context.backend == Backend::Container)
            .or_else(|| execution_context_for_backend(contract, Backend::Container, lifecycle))
            && let Some(container) = context.container.as_ref()
        {
            let Some(engine) = selected_container_engine_from_backend(Some(container)) else {
                diagnose_container_backend_cli_for_container(container, findings);
                return None;
            };
            if let Some(failure) = preferred_container_backend_probe_failure(Some(container)) {
                findings.push(container_backend_unavailable_finding(
                    failure.engine.as_str(),
                    failure.details.as_str(),
                ));
                return None;
            }

            return Some(ContainerProbeContext {
                image: container.image.clone(),
                engine,
            });
        }

        if let Some(container) = execution
            .backends
            .as_ref()
            .and_then(|backends| backends.container.as_ref())
        {
            let Some(engine) = selected_container_engine(contract) else {
                diagnose_container_backend_cli(contract, findings);
                return None;
            };
            if let Some(failure) = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.backends.as_ref())
                .and_then(|backends| backends.container.as_ref())
                .and_then(|container| preferred_container_backend_probe_failure(Some(container)))
            {
                findings.push(container_backend_unavailable_finding(
                    failure.engine.as_str(),
                    failure.details.as_str(),
                ));
                return None;
            }

            return Some(ContainerProbeContext {
                image: container.image.clone(),
                engine,
            });
        }

        findings.push(container_mode_not_configured_finding());
        return None;
    }

    if mode == DoctorMode::Remote {
        let Some(remote) = execution_context_for_backend(contract, Backend::Remote, lifecycle)
            .and_then(|(_, context)| context.remote.as_ref())
            .or_else(|| {
                execution
                    .backends
                    .as_ref()
                    .and_then(|backends| backends.remote.as_ref())
            })
        else {
            findings.push(remote_mode_not_configured_finding());
            return None;
        };

        let provider = remote.provider.trim();
        let cli = match provider {
            "daytona" => Some("daytona"),
            "ssh" => Some("ssh"),
            "tsh" => Some("tsh"),
            "kubectl" => Some("kubectl"),
            other => {
                let Some(extension) = contract.extensions.get(other) else {
                    findings.push(remote_backend_finding(
                        "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED",
                        FindingSeverity::Error,
                        format!("Unsupported remote execution backend provider: {other}"),
                        format!(
                            "the contract requests remote diagnosis with provider `{other}`, but current Ota only supports built-in providers or a matching `backend_provider` extension"
                        ),
                        "change `execution.backends.remote.provider` to `daytona`, `ssh`, `tsh`, or `kubectl`, or declare a matching `backend_provider` extension",
                    ));
                    return None;
                };

                if extension.kind != ExtensionKind::BackendProvider {
                    findings.push(remote_backend_finding(
                        "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED",
                        FindingSeverity::Error,
                        format!("Unsupported remote execution backend provider: {other}"),
                        format!(
                            "the contract requests remote diagnosis with provider `{other}`, but the matching extension is not a `backend_provider`"
                        ),
                        "change the extension kind to `backend_provider` or change the remote provider name",
                    ));
                    return None;
                }

                if extension.api_version != 1 {
                    findings.push(remote_backend_finding(
                        "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED",
                        FindingSeverity::Error,
                        format!("Unsupported backend provider api_version: {other}"),
                        format!(
                            "the matching backend provider extension declares unsupported `api_version {}`",
                            extension.api_version
                        ),
                        "bump the backend provider extension to `api_version: 1`",
                    ));
                    return None;
                }

                None
            }
        };
        if let Some(cli) = cli {
            if let Some(target) = remote.target.as_deref() {
                diagnose_remote_target_shape(provider, target, findings);
            }
            diagnose_backend_cli(
                cli,
                &format!("remote execution backend provider `{provider}`"),
                findings,
            );
        }
        return None;
    }

    match effective_execution(contract, overrides).0 {
        Backend::Container => diagnose_container_backend_cli(contract, findings),
        Backend::Remote => {
            let Some(remote) = execution
                .backends
                .as_ref()
                .and_then(|backends| backends.remote.as_ref())
            else {
                return None;
            };

            let provider = remote.provider.trim();
            let cli = match provider {
                "daytona" => Some("daytona"),
                "ssh" => Some("ssh"),
                "tsh" => Some("tsh"),
                "kubectl" => Some("kubectl"),
                other => {
                    let Some(extension) = contract.extensions.get(other) else {
                        findings.push(remote_backend_finding(
                            "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED",
                            FindingSeverity::Error,
                            format!("Unsupported remote execution backend provider: {other}"),
                            format!(
                                "the contract requests `execution.preferred: remote` with provider `{other}`, but current Ota only supports built-in providers or a matching `backend_provider` extension"
                            ),
                            "change `execution.backends.remote.provider` to `daytona`, `ssh`, `tsh`, or `kubectl`, or declare a matching `backend_provider` extension",
                        ));
                        return None;
                    };

                    if extension.kind != ExtensionKind::BackendProvider {
                        findings.push(remote_backend_finding(
                            "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED",
                            FindingSeverity::Error,
                            format!("Unsupported remote execution backend provider: {other}"),
                            format!(
                                "the contract requests `execution.preferred: remote` with provider `{other}`, but the matching extension is not a `backend_provider`"
                            ),
                            "change the extension kind to `backend_provider` or change the remote provider name",
                        ));
                        return None;
                    }

                    if extension.api_version != 1 {
                        findings.push(remote_backend_finding(
                            "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED",
                            FindingSeverity::Error,
                            format!("Unsupported backend provider api_version: {other}"),
                            format!(
                                "the matching backend provider extension declares unsupported `api_version {}`",
                                extension.api_version
                            ),
                            "bump the backend provider extension to `api_version: 1`",
                        ));
                        return None;
                    }

                    None
                }
            };
            if let Some(cli) = cli {
                if let Some(target) = remote.target.as_deref() {
                    diagnose_remote_target_shape(provider, target, findings);
                }
                diagnose_backend_cli(
                    cli,
                    &format!("remote execution backend provider `{provider}`"),
                    findings,
                );
            }
        }
        Backend::Native => {
            if let Some(remote) = execution
                .backends
                .as_ref()
                .and_then(|backends| backends.remote.as_ref())
            {
                let provider = remote.provider.trim();
                if matches!(provider, "ssh" | "tsh" | "kubectl")
                    && let Some(target) = remote.target.as_deref()
                {
                    diagnose_remote_target_shape(provider, target, findings);
                }
            }
        }
    }

    None
}

fn container_mode_not_configured_finding() -> Finding {
    Finding::identified(
        "OTA_CONTAINER_MODE_NOT_CONFIGURED",
        "execution",
        "repo_contract",
        FindingSeverity::Error,
        "Container execution is not configured",
        "container diagnosis requires `execution.backends.container.image` so Ota can inspect the execution image that actually runs tasks",
        "add `execution.backends.container.image`, then rerun `ota doctor --mode container`",
    )
}

fn remote_mode_not_configured_finding() -> Finding {
    Finding::identified(
        "OTA_REMOTE_MODE_NOT_CONFIGURED",
        "remote",
        "repo_contract",
        FindingSeverity::Error,
        "Remote execution is not configured",
        "remote diagnosis requires `execution.backends.remote.provider` and a targetable remote execution context so Ota can inspect the remote backend that actually runs tasks",
        "add `execution.backends.remote.provider` plus `execution.backends.remote.target`, or rerun `ota doctor` without `--mode remote`",
    )
}

fn contract_has_host_bound_readiness_surfaces(contract: &Contract) -> bool {
    !contract.env.vars.is_empty()
        || !contract.checks.is_empty()
        || contract
            .services
            .values()
            .any(|service| service.readiness.is_none() && service.healthcheck.is_some())
}

fn container_mode_scope_note_finding(contract: &Contract) -> Finding {
    let mut skipped = Vec::new();
    if !contract.checks.is_empty() {
        skipped.push("checks");
    }
    if contract
        .services
        .values()
        .any(|service| service.readiness.is_none() && service.healthcheck.is_some())
    {
        skipped.push("legacy service healthchecks");
    }

    let verb = "remain";
    let skipped = skipped.join(", ");
    Finding::identified(
        "OTA_CONTAINER_DOCTOR_HOST_SCOPE_NOTE",
        "execution",
        "repo_contract",
        FindingSeverity::Info,
        "Container readiness does not include host-only checks",
        format!(
            "container mode validated the selected execution image and container execution path; {skipped} {verb} host-bound and would mix contexts"
        ),
        "use `ota doctor --mode native` for host readiness, or run declared tasks with `ota run <task> --mode container` through the validated container path",
    )
}

fn remote_mode_scope_note_finding() -> Finding {
    remote_backend_finding(
        "OTA_REMOTE_DOCTOR_PARTIAL",
        FindingSeverity::Info,
        "Remote execution contexts are only partially evaluated in native mode",
        "native doctor mode can validate remote backend declarations and run contextual readiness probes from executable remote contexts, but runtime and tool version checks still evaluate the local host rather than the declared remote environment",
        "use `ota doctor --mode remote` to probe remote contexts directly, and `ota execution plan --mode remote` to inspect the remote backend contract when debugging topology",
    )
}

fn remote_policy_subject(context_name: Option<&str>) -> String {
    context_name
        .map(|value| format!("remote context `{value}`"))
        .unwrap_or_else(|| String::from("the selected remote backend"))
}

fn remote_mode_host_scope_note_finding(contract: &Contract) -> Option<Finding> {
    let skipped = contract
        .services
        .iter()
        .filter(|(_, service)| service.healthcheck.is_some() && service.readiness.is_none())
        .map(|(name, _)| name.as_str())
        .collect::<Vec<_>>();

    if skipped.is_empty() {
        return None;
    }

    let verb = if skipped.len() == 1 {
        "remains"
    } else {
        "remain"
    };
    let skipped = skipped.join(", ");
    Some(remote_backend_finding(
        "OTA_REMOTE_DOCTOR_HOST_SCOPE_NOTE",
        FindingSeverity::Info,
        "Host-bound readiness checks are not evaluated in remote mode",
        format!(
            "remote mode checks the declared remote execution backend; {skipped} {verb} host-bound and would mix contexts"
        ),
        "use `ota doctor --mode native` for host readiness, or declare `services.<name>.readiness.from` on an executable context for topology-aware remote checks",
    ))
}

fn diagnose_remote_target_shape(provider: &str, target: &str, findings: &mut Vec<Finding>) {
    let target = target.trim();
    if target.is_empty() {
        return;
    }

    match provider {
        "ssh" | "tsh" => {
            if !target.contains('@') {
                findings.push(remote_backend_finding(
                    "OTA_REMOTE_TARGET_SUSPICIOUS",
                    FindingSeverity::Warn,
                    format!("Suspicious remote target for {provider}: {target}"),
                    format!(
                        "remote provider `{provider}` usually expects a `user@host` style target, but current target `{target}` has no `@` separator"
                    ),
                    format!(
                        "set `execution.backends.remote.target` to a host target such as `user@host` for provider `{provider}`"
                    ),
                ));
            }
        }
        "kubectl" => {
            if !target.starts_with("pod/") {
                findings.push(remote_backend_finding(
                    "OTA_REMOTE_TARGET_SUSPICIOUS",
                    FindingSeverity::Warn,
                    format!("Suspicious remote target for kubectl: {target}"),
                    format!(
                        "remote provider `kubectl` is currently validated for `pod/<name>` style targets, but current target `{target}` does not start with `pod/`"
                    ),
                    "set `execution.backends.remote.target` to a pod target such as `pod/ota-dev`",
                ));
            }
        }
        _ => {}
    }
}

fn diagnose_services(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
    workflow_name: Option<&str>,
    findings: &mut Vec<Finding>,
) {
    let working_dir = contract_working_dir(contract_path);
    let selected_services = selected_workflow_service_names(contract, workflow_name);

    for (name, service) in &contract.services {
        if let Some(selected) = selected_services.as_ref()
            && !selected.contains(name)
        {
            continue;
        }
        if let Some(finding) = service_finding(
            contract,
            contract_path,
            name,
            service,
            working_dir,
            mode,
            lifecycle,
        ) {
            findings.push(finding);
        }
    }
}

fn service_finding(
    contract: &Contract,
    contract_path: &Path,
    name: &str,
    service: &ServiceSpec,
    working_dir: &Path,
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
) -> Option<Finding> {
    let rerun_doctor = rerun_doctor_command(mode, lifecycle);
    let service_severity = if service.required {
        FindingSeverity::Error
    } else {
        FindingSeverity::Warn
    };
    if let Some(producer) = service.producer.as_ref() {
        return producer_owned_service_finding(
            name,
            service,
            producer,
            contract_path,
            mode,
            lifecycle,
        );
    }
    if let Some(readiness) = &service.readiness {
        let from_context = readiness.from_context().unwrap_or_default();
        return match run_service_readiness(contract, name, service, working_dir, readiness) {
            Ok(CheckStatus::Passed) => None,
            Ok(CheckStatus::Failed) => Some(Finding::identified(
                "OTA_SERVICE_READINESS_FAILED",
                "service",
                "service",
                service_severity,
                format!("Service readiness failed: {name}"),
                service_readiness_failure_why(
                    name,
                    resolve_service_readiness_endpoint(service, readiness),
                    from_context,
                ),
                match service.start_command(name) {
                    Some(start) => format!("run `{start}` and rerun `{rerun_doctor}`"),
                    None => format!(
                        "repair `{name}` from context `{}` and rerun `{rerun_doctor}`",
                        from_context,
                    ),
                },
            )),
            Ok(CheckStatus::TimedOut(_)) => None,
            Err(error) => Some(Finding::identified(
                "OTA_SERVICE_READINESS_CONTEXT_UNEXECUTABLE",
                "service",
                "service",
                service_severity,
                format!("Service readiness context is not executable: {name}"),
                service_readiness_execution_why(
                    name,
                    resolve_service_readiness_endpoint(service, readiness),
                    from_context,
                    &error,
                ),
                service_readiness_execution_next(name, from_context, mode, lifecycle),
            )),
        };
    }

    if let Some(healthcheck) = service.healthcheck.as_deref() {
        if mode != DoctorMode::Native {
            return None;
        }
        return match run_service_healthcheck(name, service, working_dir, healthcheck) {
            CheckStatus::Passed => None,
            CheckStatus::Failed => Some(Finding::identified(
                "OTA_SERVICE_CHECK_FAILED",
                "service",
                "service",
                service_severity,
                format!("Service healthcheck failed: {name}"),
                format!("service `{name}` did not pass its configured healthcheck"),
                match service.start_command(name) {
                    Some(start) => format!("run `{start}` and rerun `{rerun_doctor}`"),
                    None => format!(
                        "start or repair `{name}` and rerun its healthcheck: {healthcheck}, then rerun `{rerun_doctor}`"
                    ),
                },
            )),
            CheckStatus::TimedOut(timeout) => Some(Finding::identified(
                "OTA_SERVICE_CHECK_TIMED_OUT",
                "service",
                "service",
                service_severity,
                format!("Service healthcheck timed out: {name}"),
                format!("service `{name}` did not become ready within {}ms", timeout),
                format!(
                    "make `services.{name}.healthcheck` complete faster or raise `services.{name}.timeout`, then rerun `{rerun_doctor}`"
                ),
            )),
        };
    }

    if service.required {
        let compose_managed = service
            .manager
            .as_ref()
            .is_some_and(|manager| manager.kind == crate::schema::ServiceManagerKind::Compose);
        let systemd_managed = service
            .manager
            .as_ref()
            .and_then(|manager| {
                (manager.kind == crate::schema::ServiceManagerKind::Host)
                    .then_some(manager.host.as_ref())
                    .flatten()
            })
            .is_some_and(|host| host.kind == crate::schema::HostServiceManagerKind::Systemd);
        let can_anchor_structured_readiness = service
            .readiness
            .as_ref()
            .and_then(ServiceReadinessSpec::from_context)
            .is_some()
            || service.has_endpoint_for_context("host")
            || service.endpoints.len() == 1
            || compose_managed
            || systemd_managed;

        let why = if can_anchor_structured_readiness {
            format!(
                "service `{name}` is required but no `healthcheck` is configured, so Ota cannot verify readiness"
            )
        } else {
            format!(
                "service `{name}` is required but the current managed service shape does not yet expose one truthful readiness surface, so Ota cannot verify readiness"
            )
        };

        let next = if can_anchor_structured_readiness {
            if compose_managed {
                format!(
                    "declare readiness with `ota assist declare-readiness --service {name} --style compose-health` (or `--style tcp` / `--style http`), then rerun `ota doctor`"
                )
            } else if systemd_managed {
                format!(
                    "declare readiness with `ota assist declare-readiness --service {name} --style systemd-active` (or `--style tcp` / `--style http`), then rerun `ota doctor`"
                )
            } else {
                format!(
                    "declare readiness with `ota assist declare-readiness --service {name} --style tcp` or `--style http`, then rerun `ota doctor`"
                )
            }
        } else {
            format!(
                "refine the managed service with `ota assist declare-service --name {name} --style tcp` or `--style http`, then rerun `ota doctor`"
            )
        };

        return Some(Finding::identified(
            "OTA_SERVICE_UNVERIFIABLE",
            "service",
            "service",
            FindingSeverity::Warn,
            format!("Required service cannot be verified: {name}"),
            why,
            next,
        ));
    }

    None
}

fn producer_owned_service_finding(
    name: &str,
    service: &ServiceSpec,
    producer: &ServiceProducerSpec,
    contract_path: &Path,
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
) -> Option<Finding> {
    let rerun_doctor = rerun_doctor_command(mode, lifecycle);
    let service_severity = if service.required {
        FindingSeverity::Error
    } else {
        FindingSeverity::Warn
    };
    let (producer_contract, producer_contract_path) = match load_contract_for_workspace_repo_ref(
        contract_path,
        producer.repo.as_str(),
        "producer.repo",
    ) {
        Ok(value) => value,
        Err(error) => {
            return Some(Finding::identified(
                "OTA_SERVICE_UNVERIFIABLE",
                "service",
                "service",
                service_severity,
                format!("Required service cannot be verified: {name}"),
                format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but Ota could not load that producer contract: {}",
                    producer.repo, producer.task, error
                ),
                format!(
                    "repair workspace repo `{}` or run `ota workspace up`, then rerun `{rerun_doctor}`",
                    producer.repo
                ),
            ));
        }
    };
    let producer_task = match producer_contract.tasks.get(producer.task.as_str()) {
        Some(task) => task,
        None => {
            return Some(Finding::identified(
                "OTA_SERVICE_UNVERIFIABLE",
                "service",
                "service",
                service_severity,
                format!("Required service cannot be verified: {name}"),
                format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but that task is not declared",
                    producer.repo, producer.task
                ),
                format!(
                    "repair workspace repo `{}` task `{}` or run `ota workspace up`, then rerun `{rerun_doctor}`",
                    producer.repo, producer.task
                ),
            ));
        }
    };
    let listener_name = match resolve_producer_service_listener_name(
        producer_task,
        producer.listener.as_deref(),
    ) {
        Ok(name) => name,
        Err(error) => {
            return Some(Finding::identified(
                "OTA_SERVICE_UNVERIFIABLE",
                "service",
                "service",
                service_severity,
                format!("Required service cannot be verified: {name}"),
                format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but {}",
                    producer.repo, producer.task, error
                ),
                format!(
                    "repair workspace repo `{}` task `{}` or refine `services.{name}.producer.listener`, then rerun `{rerun_doctor}`",
                    producer.repo, producer.task
                ),
            ));
        }
    };
    let backend = match crate::runner::resolve_execution_backend_with_contract_path(
        &producer_contract,
        producer.task.as_str(),
        crate::runner::ExecutionOverrides::default(),
        Some(producer_contract_path.as_path()),
    ) {
        Ok(backend) => match backend {
            ResolvedExecutionBackend::Native { .. } => Backend::Native,
            ResolvedExecutionBackend::Container { .. } => Backend::Container,
            ResolvedExecutionBackend::Remote { .. } => Backend::Remote,
            ResolvedExecutionBackend::BackendProvider { .. } => Backend::Remote,
        },
        Err(error) => {
            return Some(Finding::identified(
                "OTA_SERVICE_UNVERIFIABLE",
                "service",
                "service",
                service_severity,
                format!("Required service cannot be verified: {name}"),
                format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but Ota could not resolve that producer runtime: {}",
                    producer.repo, producer.task, error
                ),
                format!(
                    "repair workspace repo `{}` task `{}` or run `ota workspace up`, then rerun `{rerun_doctor}`",
                    producer.repo, producer.task
                ),
            ));
        }
    };
    let probe = match task_runtime_host_readiness_probe_for_backend(
        &producer_contract,
        producer_task,
        backend,
        listener_name.as_str(),
    ) {
        Ok(probe) => probe,
        Err(error) => {
            return Some(Finding::identified(
                "OTA_SERVICE_UNVERIFIABLE",
                "service",
                "service",
                service_severity,
                format!("Required service cannot be verified: {name}"),
                format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but {}",
                    producer.repo, producer.task, error
                ),
                format!(
                    "repair workspace repo `{}` task `{}` listener `{}` or run `ota workspace up`, then rerun `{rerun_doctor}`",
                    producer.repo, producer.task, listener_name
                ),
            ));
        }
    };
    if host_runtime_readiness_observed(&probe, None) {
        return None;
    }

    Some(Finding::identified(
        "OTA_SERVICE_CHECK_FAILED",
        "service",
        "service",
        service_severity,
        format!("Service producer is not ready: {name}"),
        format!(
            "service `{name}` is owned by workspace repo `{}` task `{}` listener `{}` and its projected host endpoint `{}:{}` is not ready",
            producer.repo, producer.task, probe.listener, probe.address, probe.port
        ),
        format!(
            "run `ota workspace up` to prepare the workspace end to end, or run `ota run {} {}` for workspace repo `{}` before rerunning `{rerun_doctor}`",
            producer.task,
            doctor_shell_quote(
                &producer_contract_path
                    .parent()
                    .unwrap_or(producer_contract_path.as_path())
                    .components()
                    .filter(|component| !matches!(component, Component::CurDir))
                    .fold(PathBuf::new(), |mut path, component| {
                        path.push(component.as_os_str());
                        path
                    })
                    .display()
                    .to_string()
            ),
            producer.repo
        ),
    ))
}

fn resolve_producer_service_listener_name(
    task: &crate::schema::TaskSpec,
    explicit_listener: Option<&str>,
) -> Result<String, String> {
    if let Some(listener) = explicit_listener.map(str::trim) {
        if listener.is_empty() {
            return Err(String::from("producer field `listener` must not be empty"));
        }
        return Ok(listener.to_string());
    }
    let listeners = task_declared_service_listener_names(task);
    match listeners.len() {
        1 => Ok(listeners
            .into_iter()
            .next()
            .expect("one listener should exist")),
        0 => Err(String::from(
            "that task does not declare any service listeners",
        )),
        _ => Err(String::from(
            "that task declares multiple listeners, so `services.<name>.producer.listener` must be explicit",
        )),
    }
}

fn task_declared_service_listener_names(task: &crate::schema::TaskSpec) -> BTreeSet<String> {
    let mut listeners = BTreeSet::new();
    if let Some(runtime) = task
        .runtime
        .as_ref()
        .filter(|runtime| runtime.kind == crate::schema::TaskRuntimeKind::Service)
    {
        listeners.extend(runtime.listeners.keys().cloned());
    }
    if let Some(execution) = task.execution.as_ref() {
        for (_, branch) in execution.modes.iter() {
            if let Some(runtime) = branch
                .runtime
                .as_ref()
                .filter(|runtime| runtime.kind == crate::schema::TaskRuntimeKind::Service)
            {
                listeners.extend(runtime.listeners.keys().cloned());
            }
        }
    }
    listeners
}

fn service_readiness_failure_why(
    name: &str,
    endpoint: Option<&crate::schema::ServiceEndpointSpec>,
    context_name: &str,
) -> String {
    match endpoint {
        Some(endpoint) => format!(
            "service `{name}` did not pass its configured readiness probe from context `{context_name}`; projected endpoint is `{}:{}`",
            endpoint.address, endpoint.port
        ),
        None => format!(
            "service `{name}` did not pass its configured readiness probe from context `{context_name}`"
        ),
    }
}

fn service_readiness_execution_why(
    name: &str,
    endpoint: Option<&crate::schema::ServiceEndpointSpec>,
    context_name: &str,
    error: &RunError,
) -> String {
    match endpoint {
        Some(endpoint) => format!(
            "service `{name}` declares readiness from context `{context_name}` against projected endpoint `{}:{}`, but Ota could not execute that readiness probe: {}",
            endpoint.address, endpoint.port, error
        ),
        None => format!(
            "service `{name}` declares readiness from context `{context_name}`, but Ota could not execute that readiness probe: {}",
            error
        ),
    }
}

fn service_readiness_execution_next(
    name: &str,
    context_name: &str,
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
) -> String {
    format!(
        "repair execution context `{context_name}` or move `services.{name}.readiness.from` to a context Ota can execute, then rerun `{}`",
        rerun_doctor_command(mode, lifecycle)
    )
}

fn resolve_service_readiness_endpoint<'a>(
    service: &'a ServiceSpec,
    readiness: &crate::schema::ServiceReadinessSpec,
) -> Option<&'a crate::schema::ServiceEndpointSpec> {
    let from_context = readiness.from_context()?;
    if let Some(endpoint_name) = readiness
        .endpoint_name()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        service.endpoint_named(endpoint_name)
    } else {
        service.endpoint_for_context(from_context)
    }
}

fn run_service_readiness(
    contract: &Contract,
    name: &str,
    service: &ServiceSpec,
    working_dir: &Path,
    readiness: &crate::schema::ServiceReadinessSpec,
) -> Result<CheckStatus, RunError> {
    if matches!(
        readiness.structured_kind(),
        Some(crate::schema::ServiceReadinessKind::ComposeHealth)
    ) {
        return run_service_compose_health_readiness(name, service, working_dir, readiness);
    }
    if matches!(
        readiness.structured_kind(),
        Some(crate::schema::ServiceReadinessKind::SystemdActive)
    ) {
        return run_service_systemd_active_readiness(name, service, working_dir, readiness);
    }

    let Some(from_context) = readiness.from_context() else {
        return Ok(CheckStatus::Failed);
    };
    let backend = resolve_context_execution_backend(contract, from_context)?;

    if matches!(
        backend,
        ResolvedExecutionBackend::Native {
            shared_local_backend: _
        }
    ) && let Some(args) = service
        .manager
        .as_ref()
        .and_then(|manager| manager.compose_ps_command_argv(name))
        && let Some(engine) = service
            .manager
            .as_ref()
            .and_then(crate::schema::ServiceManagerSpec::compose_cli_exe)
    {
        match run_backend_argv_command_captured(
            &format!("service-manager:{name}"),
            engine,
            &args,
            working_dir,
            &backend,
        ) {
            Ok(output) if output.exit_code != 0 || output.stdout.trim().is_empty() => {
                return Ok(CheckStatus::Failed);
            }
            Ok(_) => {}
            Err(error) => return Err(error),
        }
    }

    if let Some(probe_name) = readiness
        .probe
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        let Ok(resolved) = resolve_named_readiness_probe_contract(contract, probe_name) else {
            return Ok(CheckStatus::Failed);
        };
        let Some(endpoint) = resolve_service_readiness_endpoint(service, readiness) else {
            return Ok(CheckStatus::Failed);
        };
        let is_native_backend = matches!(backend, ResolvedExecutionBackend::Native { .. });
        let timing = service_readiness_timing_policy(readiness);
        if !timing.start_period.is_zero() {
            thread::sleep(timing.start_period);
        }
        let mut failed_attempts = 0u32;
        loop {
            match &resolved {
                ResolvedNamedReadinessProbeContract::Http {
                    request, timeout, ..
                } => {
                    if is_native_backend {
                        match http_readiness_endpoint_status(
                            endpoint.address.as_str(),
                            endpoint.port,
                            request,
                            *timeout,
                        ) {
                            HttpReadinessStatus::Passed => return Ok(CheckStatus::Passed),
                            HttpReadinessStatus::Failed | HttpReadinessStatus::TimedOut => {
                                failed_attempts = failed_attempts.saturating_add(1);
                                if failed_attempts >= timing.retries {
                                    return Ok(CheckStatus::Failed);
                                }
                            }
                        }
                    } else {
                        let command = service_http_readiness_probe_command_from_request(
                            endpoint, request, *timeout,
                        );
                        match run_backend_command_captured(
                            &format!("readiness:{name}"),
                            command.as_str(),
                            working_dir,
                            &backend,
                        ) {
                            Ok(output) if output.exit_code == 0 => return Ok(CheckStatus::Passed),
                            Ok(_) => {
                                failed_attempts = failed_attempts.saturating_add(1);
                                if failed_attempts >= timing.retries {
                                    return Ok(CheckStatus::Failed);
                                }
                            }
                            Err(error) => return Err(error),
                        }
                    }
                }
                ResolvedNamedReadinessProbeContract::Tcp { timeout, .. } => {
                    if is_native_backend {
                        match tcp_readiness_endpoint_status(
                            endpoint.address.as_str(),
                            endpoint.port,
                            *timeout,
                        ) {
                            HttpReadinessStatus::Passed => return Ok(CheckStatus::Passed),
                            HttpReadinessStatus::Failed | HttpReadinessStatus::TimedOut => {
                                failed_attempts = failed_attempts.saturating_add(1);
                                if failed_attempts >= timing.retries {
                                    return Ok(CheckStatus::Failed);
                                }
                            }
                        }
                    } else {
                        let command =
                            service_tcp_readiness_probe_command_from_timeout(endpoint, *timeout);
                        match run_backend_command_captured(
                            &format!("readiness:{name}"),
                            command.as_str(),
                            working_dir,
                            &backend,
                        ) {
                            Ok(output) if output.exit_code == 0 => return Ok(CheckStatus::Passed),
                            Ok(_) => {
                                failed_attempts = failed_attempts.saturating_add(1);
                                if failed_attempts >= timing.retries {
                                    return Ok(CheckStatus::Failed);
                                }
                            }
                            Err(error) => return Err(error),
                        }
                    }
                }
            }
            thread::sleep(timing.interval);
        }
    }

    if let Some(command) = readiness.legacy_run_command() {
        return match run_backend_command_captured(
            &format!("readiness:{name}"),
            command.as_str(),
            working_dir,
            &backend,
        ) {
            Ok(output) if output.exit_code == 0 => Ok(CheckStatus::Passed),
            Ok(_) => Ok(CheckStatus::Failed),
            Err(error) => Err(error),
        };
    }

    let Some(kind) = readiness.structured_kind() else {
        return Ok(CheckStatus::Failed);
    };
    let Some(endpoint) = resolve_service_readiness_endpoint(service, readiness) else {
        return Ok(CheckStatus::Failed);
    };
    let timing = service_readiness_timing_policy(readiness);
    if !timing.start_period.is_zero() {
        thread::sleep(timing.start_period);
    }
    let mut failed_attempts = 0u32;
    let is_native_backend = matches!(backend, ResolvedExecutionBackend::Native { .. });
    loop {
        if is_native_backend {
            match kind {
                crate::schema::ServiceReadinessKind::Http => {
                    let request = HttpReadinessRequest {
                        method: readiness
                            .method
                            .unwrap_or(crate::schema::TaskRuntimeReadinessHttpMethod::Get),
                        path: normalized_runtime_path(readiness.path.as_deref()),
                        headers: readiness.headers.clone(),
                        success_statuses: readiness
                            .success
                            .as_ref()
                            .filter(|success| !success.status.is_empty())
                            .map(|success| success.status.clone())
                            .unwrap_or_else(|| (200u16..400u16).collect()),
                        body_contains: readiness.body.as_ref().map(|body| body.contains.clone()),
                    };
                    let timeout = readiness
                        .timeout
                        .as_deref()
                        .and_then(crate::schema::parse_readiness_duration_spec);
                    match http_readiness_endpoint_status(
                        endpoint.address.as_str(),
                        endpoint.port,
                        &request,
                        timeout,
                    ) {
                        HttpReadinessStatus::Passed => return Ok(CheckStatus::Passed),
                        HttpReadinessStatus::Failed | HttpReadinessStatus::TimedOut => {
                            failed_attempts = failed_attempts.saturating_add(1);
                            if failed_attempts >= timing.retries {
                                return Ok(CheckStatus::Failed);
                            }
                        }
                    }
                }
                crate::schema::ServiceReadinessKind::Tcp => {
                    let timeout = readiness
                        .timeout
                        .as_deref()
                        .and_then(crate::schema::parse_readiness_duration_spec);
                    match tcp_readiness_endpoint_status(
                        endpoint.address.as_str(),
                        endpoint.port,
                        timeout,
                    ) {
                        HttpReadinessStatus::Passed => return Ok(CheckStatus::Passed),
                        HttpReadinessStatus::Failed | HttpReadinessStatus::TimedOut => {
                            failed_attempts = failed_attempts.saturating_add(1);
                            if failed_attempts >= timing.retries {
                                return Ok(CheckStatus::Failed);
                            }
                        }
                    }
                }
                crate::schema::ServiceReadinessKind::ComposeHealth => {
                    unreachable!("compose health readiness is handled before endpoint projection")
                }
                crate::schema::ServiceReadinessKind::SystemdActive => {
                    unreachable!("systemd active readiness is handled before endpoint projection")
                }
            }
        } else {
            let command = structured_service_readiness_command(readiness, endpoint, kind);
            match run_backend_command_captured(
                &format!("readiness:{name}"),
                command.as_str(),
                working_dir,
                &backend,
            ) {
                Ok(output) if output.exit_code == 0 => return Ok(CheckStatus::Passed),
                Ok(_) => {
                    failed_attempts = failed_attempts.saturating_add(1);
                    if failed_attempts >= timing.retries {
                        return Ok(CheckStatus::Failed);
                    }
                }
                Err(error) => return Err(error),
            }
        }
        thread::sleep(timing.interval);
    }
}

#[derive(Debug, Clone, Copy)]
struct ServiceReadinessTimingPolicy {
    start_period: Duration,
    interval: Duration,
    retries: u32,
}

fn service_readiness_timing_policy(
    readiness: &crate::schema::ServiceReadinessSpec,
) -> ServiceReadinessTimingPolicy {
    ServiceReadinessTimingPolicy {
        start_period: readiness
            .start_period
            .as_deref()
            .and_then(crate::schema::parse_readiness_duration_spec)
            .map(cap_doctor_readiness_start_period)
            .unwrap_or(Duration::ZERO),
        interval: readiness
            .interval
            .as_deref()
            .and_then(crate::schema::parse_readiness_duration_spec)
            .unwrap_or(Duration::from_millis(200)),
        retries: readiness
            .retries
            .unwrap_or(DOCTOR_DEFAULT_SERVICE_READINESS_RETRIES),
    }
}

fn cap_doctor_readiness_start_period(duration: Duration) -> Duration {
    duration.min(Duration::from_millis(DOCTOR_READINESS_MAX_START_PERIOD_MS))
}

fn structured_service_readiness_command(
    readiness: &crate::schema::ServiceReadinessSpec,
    endpoint: &crate::schema::ServiceEndpointSpec,
    kind: crate::schema::ServiceReadinessKind,
) -> String {
    match kind {
        crate::schema::ServiceReadinessKind::Http => {
            service_http_readiness_probe_command(readiness, endpoint)
        }
        crate::schema::ServiceReadinessKind::Tcp => {
            service_tcp_readiness_probe_command(readiness, endpoint)
        }
        crate::schema::ServiceReadinessKind::ComposeHealth => {
            unreachable!("compose health readiness does not use endpoint probing commands")
        }
        crate::schema::ServiceReadinessKind::SystemdActive => {
            unreachable!("systemd active readiness does not use endpoint probing commands")
        }
    }
}

fn run_service_compose_health_readiness(
    name: &str,
    service: &ServiceSpec,
    working_dir: &Path,
    readiness: &crate::schema::ServiceReadinessSpec,
) -> Result<CheckStatus, RunError> {
    let command = service
        .manager
        .as_ref()
        .and_then(|manager| manager.compose_health_status_command(name));
    let Some(command) = command else {
        return Ok(CheckStatus::Failed);
    };

    let timing = service_readiness_timing_policy(readiness);
    if !timing.start_period.is_zero() {
        thread::sleep(timing.start_period);
    }

    let backend = ResolvedExecutionBackend::Native {
        shared_local_backend: None,
    };
    let mut failed_attempts = 0u32;
    loop {
        match run_backend_command_captured(
            &format!("readiness:{name}"),
            command.as_str(),
            working_dir,
            &backend,
        ) {
            Ok(output) if output.exit_code == 0 => return Ok(CheckStatus::Passed),
            Ok(_) => {
                failed_attempts = failed_attempts.saturating_add(1);
                if failed_attempts >= timing.retries {
                    return Ok(CheckStatus::Failed);
                }
            }
            Err(error) => return Err(error),
        }
        thread::sleep(timing.interval);
    }
}

fn run_service_systemd_active_readiness(
    name: &str,
    service: &ServiceSpec,
    working_dir: &Path,
    readiness: &crate::schema::ServiceReadinessSpec,
) -> Result<CheckStatus, RunError> {
    let command = service
        .manager
        .as_ref()
        .and_then(crate::schema::ServiceManagerSpec::systemd_active_command);
    let Some(command) = command else {
        return Ok(CheckStatus::Failed);
    };

    let timing = service_readiness_timing_policy(readiness);
    if !timing.start_period.is_zero() {
        thread::sleep(timing.start_period);
    }

    let backend = ResolvedExecutionBackend::Native {
        shared_local_backend: None,
    };
    let mut failed_attempts = 0u32;
    loop {
        match run_backend_command_captured(
            &format!("readiness:{name}"),
            command.as_str(),
            working_dir,
            &backend,
        ) {
            Ok(output) if output.exit_code == 0 => return Ok(CheckStatus::Passed),
            Ok(_) => {
                failed_attempts = failed_attempts.saturating_add(1);
                if failed_attempts >= timing.retries {
                    return Ok(CheckStatus::Failed);
                }
            }
            Err(error) => return Err(error),
        }
        thread::sleep(timing.interval);
    }
}

fn service_http_readiness_probe_command(
    readiness: &crate::schema::ServiceReadinessSpec,
    endpoint: &crate::schema::ServiceEndpointSpec,
) -> String {
    let request = HttpReadinessRequest {
        method: readiness
            .method
            .unwrap_or(crate::schema::TaskRuntimeReadinessHttpMethod::Get),
        path: normalized_runtime_path(readiness.path.as_deref()),
        headers: readiness.headers.clone(),
        success_statuses: readiness
            .success
            .as_ref()
            .filter(|success| !success.status.is_empty())
            .map(|success| success.status.clone())
            .unwrap_or_else(|| (200u16..400u16).collect()),
        body_contains: readiness.body.as_ref().map(|body| body.contains.clone()),
    };
    let timeout = readiness
        .timeout
        .as_deref()
        .and_then(crate::schema::parse_readiness_duration_spec);
    service_http_readiness_probe_command_from_request(endpoint, &request, timeout)
}

fn service_http_readiness_probe_command_from_request(
    endpoint: &crate::schema::ServiceEndpointSpec,
    request: &HttpReadinessRequest,
    timeout: Option<Duration>,
) -> String {
    let url = format!(
        "http://{}:{}{}",
        endpoint.address.trim(),
        endpoint.port,
        request.path
    );
    let status_csv = if request.success_statuses.is_empty() {
        String::from("200,201,202,203,204,205,206,207,208,226,300,301,302,303,304,305,306,307,308")
    } else {
        request
            .success_statuses
            .iter()
            .map(|status| status.to_string())
            .collect::<Vec<_>>()
            .join(",")
    };
    let headers_shell = request
        .headers
        .iter()
        .map(|(name, value)| format!("-H {}", doctor_shell_quote(&format!("{name}: {value}"))))
        .collect::<Vec<_>>()
        .join(" ");
    let headers_json =
        serde_json::to_string(&request.headers).unwrap_or_else(|_| String::from("{}"));
    let body_contains = request.body_contains.clone().unwrap_or_default();
    let timeout_seconds = timeout
        .map(|duration| duration.as_secs_f64().max(0.001))
        .unwrap_or(2.0);
    format!(
        "url={url}; method={method}; statuses={statuses}; contains={contains}; headers_json={headers_json}; timeout={timeout}; \
if command -v curl >/dev/null 2>&1; then \
  body_file=$(mktemp 2>/dev/null || printf '/tmp/ota-service-readiness-body-$$'); \
  code=$(curl -sS --connect-timeout \"$timeout\" --max-time \"$timeout\" -X \"$method\" {headers} -o \"$body_file\" -w '%{{http_code}}' \"$url\"); curl_status=$?; \
  if [ $curl_status -eq 28 ]; then rm -f \"$body_file\"; exit 124; fi; \
  [ $curl_status -eq 0 ] || {{ rm -f \"$body_file\"; exit 1; }}; \
  matched=1; OLDIFS=\"$IFS\"; IFS=,; for expected in $statuses; do if [ \"$code\" = \"$expected\" ]; then matched=0; break; fi; done; IFS=\"$OLDIFS\"; \
  [ $matched -eq 0 ] || {{ rm -f \"$body_file\"; exit 1; }}; \
  if [ -n \"$contains\" ]; then grep -Fq -- \"$contains\" \"$body_file\" || {{ rm -f \"$body_file\"; exit 1; }}; fi; \
  rm -f \"$body_file\"; exit 0; \
fi; \
if command -v python3 >/dev/null 2>&1; then \
  python3 - \"$url\" \"$method\" \"$statuses\" \"$contains\" \"$headers_json\" \"$timeout\" <<'PY'\n\
import json, socket, sys, urllib.error, urllib.request\n\
class NoRedirect(urllib.request.HTTPRedirectHandler):\n\
    def redirect_request(self, req, fp, code, msg, headers, newurl):\n\
        return None\n\
url, method, statuses_raw, contains, headers_raw, timeout_raw = sys.argv[1:7]\n\
statuses = {{int(value) for value in statuses_raw.split(',') if value}}\n\
headers = json.loads(headers_raw)\n\
timeout = float(timeout_raw)\n\
request = urllib.request.Request(url, method=method, headers=headers)\n\
opener = urllib.request.build_opener(NoRedirect)\n\
try:\n\
    with opener.open(request, timeout=timeout) as response:\n\
        status = response.status\n\
        body = response.read().decode(errors='ignore')\n\
except urllib.error.HTTPError as error:\n\
    status = error.code\n\
    body = error.read().decode(errors='ignore')\n\
except urllib.error.URLError as error:\n\
    reason = getattr(error, 'reason', None)\n\
    if isinstance(reason, (TimeoutError, socket.timeout)) or str(reason).lower() == 'timed out':\n\
        sys.exit(124)\n\
    sys.exit(1)\n\
except (TimeoutError, socket.timeout):\n\
    sys.exit(124)\n\
except Exception:\n\
    sys.exit(1)\n\
if status not in statuses:\n\
    sys.exit(1)\n\
if contains and contains not in body:\n\
    sys.exit(1)\n\
PY\n\
  probe_status=$?; [ $probe_status -eq 0 ] && exit 0; [ $probe_status -eq 124 ] && exit 124; exit 1\n\
fi; \
exit 1",
        url = doctor_shell_quote(&url),
        method = doctor_shell_quote(request.method.as_str()),
        statuses = doctor_shell_quote(&status_csv),
        contains = doctor_shell_quote(&body_contains),
        headers_json = doctor_shell_quote(&headers_json),
        timeout = doctor_shell_quote(&timeout_seconds.to_string()),
        headers = headers_shell,
    )
}

fn service_tcp_readiness_probe_command(
    readiness: &crate::schema::ServiceReadinessSpec,
    endpoint: &crate::schema::ServiceEndpointSpec,
) -> String {
    let timeout_seconds = readiness
        .timeout
        .as_deref()
        .and_then(crate::schema::parse_readiness_duration_spec)
        .map(|duration| duration.as_secs_f64().max(0.001))
        .unwrap_or(2.0);
    format!(
        "host={host}; port={port}; timeout={timeout}; \
if command -v python3 >/dev/null 2>&1; then \
  python3 - \"$host\" \"$port\" \"$timeout\" <<'PY'\n\
import socket, sys\n\
host, port_raw, timeout_raw = sys.argv[1:4]\n\
port = int(port_raw)\n\
timeout = float(timeout_raw)\n\
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)\n\
sock.settimeout(timeout)\n\
try:\n\
    sock.connect((host, port))\n\
except socket.timeout:\n\
    sys.exit(124)\n\
except Exception:\n\
    sys.exit(1)\n\
finally:\n\
    try:\n\
        sock.close()\n\
    except Exception:\n\
        pass\n\
PY\n\
  probe_status=$?; [ $probe_status -eq 0 ] && exit 0; [ $probe_status -eq 124 ] && exit 124; exit 1\n\
fi; \
if command -v nc >/dev/null 2>&1; then nc -z -w \"$timeout\" \"$host\" \"$port\" >/dev/null 2>&1 && exit 0 || exit 1; fi; \
if command -v bash >/dev/null 2>&1; then bash -lc \"exec 3<>/dev/tcp/$host/$port\" >/dev/null 2>&1 && exit 0 || exit 1; fi; \
exit 1",
        host = doctor_shell_quote(endpoint.address.trim()),
        port = doctor_shell_quote(&endpoint.port.to_string()),
        timeout = doctor_shell_quote(&timeout_seconds.to_string()),
    )
}

fn service_tcp_readiness_probe_command_from_timeout(
    endpoint: &crate::schema::ServiceEndpointSpec,
    timeout: Option<Duration>,
) -> String {
    let timeout_seconds = timeout
        .map(|duration| duration.as_secs_f64().max(0.001))
        .unwrap_or(2.0);
    format!(
        "host={host}; port={port}; timeout={timeout}; \
if command -v python3 >/dev/null 2>&1; then \
  python3 - \"$host\" \"$port\" \"$timeout\" <<'PY'\n\
import socket, sys\n\
host, port_raw, timeout_raw = sys.argv[1:4]\n\
port = int(port_raw)\n\
timeout = float(timeout_raw)\n\
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)\n\
sock.settimeout(timeout)\n\
try:\n\
    sock.connect((host, port))\n\
except socket.timeout:\n\
    sys.exit(124)\n\
except Exception:\n\
    sys.exit(1)\n\
finally:\n\
    try:\n\
        sock.close()\n\
    except Exception:\n\
        pass\n\
PY\n\
  probe_status=$?; [ $probe_status -eq 0 ] && exit 0; [ $probe_status -eq 124 ] && exit 124; exit 1\n\
fi; \
if command -v nc >/dev/null 2>&1; then nc -z -w \"$timeout\" \"$host\" \"$port\" >/dev/null 2>&1 && exit 0 || exit 1; fi; \
if command -v bash >/dev/null 2>&1; then bash -lc \"exec 3<>/dev/tcp/$host/$port\" >/dev/null 2>&1 && exit 0 || exit 1; fi; \
exit 1",
        host = doctor_shell_quote(endpoint.address.trim()),
        port = doctor_shell_quote(&endpoint.port.to_string()),
        timeout = doctor_shell_quote(&timeout_seconds.to_string()),
    )
}

fn normalized_runtime_path(path: Option<&str>) -> String {
    match path {
        Some(path) => {
            let trimmed = path.trim();
            if trimmed.is_empty() {
                String::from("/")
            } else if trimmed.starts_with('/') {
                trimmed.to_string()
            } else {
                format!("/{trimmed}")
            }
        }
        None => String::from("/"),
    }
}

fn doctor_shell_quote(value: &str) -> String {
    if value.is_empty() {
        return String::from("''");
    }
    let mut quoted = String::from("'");
    for ch in value.chars() {
        if ch == '\'' {
            quoted.push_str("'\"'\"'");
        } else {
            quoted.push(ch);
        }
    }
    quoted.push('\'');
    quoted
}

fn run_service_healthcheck(
    name: &str,
    service: &ServiceSpec,
    working_dir: &Path,
    healthcheck: &str,
) -> CheckStatus {
    let command = service.healthcheck_command(name, healthcheck);
    run_check(&command, working_dir, service.timeout)
}

#[cfg(test)]
fn compose_service_healthcheck_command(name: &str, healthcheck: &str) -> String {
    let service = ServiceSpec {
        provider: Some(String::from("docker-compose")),
        ..ServiceSpec::default()
    };
    service.healthcheck_command(name, healthcheck)
}

fn diagnose_env_sources(declared_sources: &[LoadedDeclaredEnvSource], findings: &mut Vec<Finding>) {
    for source in declared_sources {
        match source.status {
            DeclaredEnvSourceStatus::Loaded => {}
            DeclaredEnvSourceStatus::MissingOptional => {}
            DeclaredEnvSourceStatus::MissingRequired => findings.push(Finding::identified(
                "OTA_ENV_SOURCE_MISSING_REQUIRED",
                "environment",
                "repo_contract",
                FindingSeverity::Error,
                format!("Missing required environment source: {}", source.label()),
                format!(
                    "the repo declares `{}:{}` with `must_exist: true`, but that file is missing",
                    source.kind, source.path
                ),
                format!(
                    "create `{}` or update `env.sources`, then rerun `ota doctor`",
                    source.path
                ),
            )),
            DeclaredEnvSourceStatus::ParseFailed => findings.push(Finding::identified(
                "OTA_ENV_SOURCE_PARSE_FAILED",
                "environment",
                "repo_contract",
                FindingSeverity::Error,
                format!("Environment source parse failed: {}", source.label()),
                format!(
                    "ota could not read declared source `{}:{}`: {}",
                    source.kind,
                    source.path,
                    source.details.as_deref().unwrap_or("unknown parse error")
                ),
                format!(
                    "fix `{}` so ota can parse declared source `{}`, then rerun `ota doctor`",
                    source.path,
                    source.label()
                ),
            )),
            DeclaredEnvSourceStatus::InvalidStructure => findings.push(Finding::identified(
                "OTA_ENV_SOURCE_INVALID_STRUCTURE",
                "environment",
                "repo_contract",
                FindingSeverity::Error,
                format!("Environment source has invalid structure: {}", source.label()),
                format!(
                    "declared source `{}` loaded as text, but its structure is not supported: {}",
                    source.label(),
                    source.details.as_deref().unwrap_or("unknown structure error")
                ),
                format!(
                    "replace unsupported values in `{}` with scalar env-shaped values only, then rerun `ota doctor`",
                    source.path
                ),
            )),
            DeclaredEnvSourceStatus::Collision => findings.push(Finding::identified(
                "OTA_ENV_SOURCE_KEY_COLLISION",
                "environment",
                "repo_contract",
                FindingSeverity::Error,
                format!("Environment source key collision: {}", source.label()),
                format!(
                    "declared source `{}` contains multiple keys that normalize to the same env name: {}",
                    source.label(),
                    source.details.as_deref().unwrap_or("unknown collision")
                ),
                format!(
                    "rename the colliding keys in `{}` so each normalized env name is unique, then rerun `ota doctor`",
                    source.path
                ),
            )),
        }
    }
}

fn diagnose_env(
    contract: &Contract,
    policy_env: Option<&BTreeMap<String, String>>,
    declared_sources: &[LoadedDeclaredEnvSource],
    selected_env_names: Option<&BTreeSet<String>>,
    findings: &mut Vec<Finding>,
) {
    for (name, requirement) in &contract.env {
        if let Some(selected_env_names) = selected_env_names
            && !selected_env_names.contains(name)
        {
            continue;
        }
        let required_for_selected_path = selected_env_names
            .map(|names| names.contains(name))
            .unwrap_or(requirement.required);
        let value = policy_env
            .and_then(|values| values.get(name))
            .cloned()
            .or_else(|| std::env::var(name).ok())
            .or_else(|| {
                resolve_declared_env_source_value(name, &declared_sources).map(|(value, _)| value)
            })
            .or_else(|| requirement.default.clone());

        match value {
            Some(value) => {
                if !requirement.allowed.is_empty()
                    && !requirement.allowed.iter().any(|allowed| allowed == &value)
                {
                    findings.push(Finding::identified(
                        "OTA_ENV_INVALID",
                        "environment",
                        "host",
                        FindingSeverity::Error,
                        format!("Invalid environment value: {name}"),
                        format!(
                            "{name} resolved to `{value}`, which is outside the allowed values"
                        ),
                        format!(
                            "run `ota env` to inspect the resolved source for {name}, then set {name} to one of: {}",
                            requirement.allowed.join(", ")
                        ),
                    ));
                }
            }
            None if required_for_selected_path => findings.push(Finding::identified(
                "OTA_ENV_MISSING",
                "environment",
                "host",
                FindingSeverity::Error,
                format!("Missing environment variable: {name}"),
                if requirement.required {
                    format!("{name} is required by this repo contract")
                } else {
                    format!("{name} is required by the selected task or workflow path")
                },
                format!(
                    "run `ota env` to inspect the current precedence, then set {name} in policy env, the shell, or a declared env source before running tasks"
                ),
            )),
            None => {}
        }
    }
}

fn diagnose_runtimes(
    runtimes: &BTreeMap<String, RuntimeRequirement>,
    target_os: &str,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    mode: DoctorMode,
    selected_lifecycle: Option<Lifecycle>,
    container_probe: Option<&ContainerProbeContext>,
    remote_probe: Option<&ResolvedExecutionBackend>,
    remote_context_name: Option<&str>,
    provisioning_actions: &[ProvisioningAction],
    findings: &mut Vec<Finding>,
) -> bool {
    if mode == DoctorMode::Container && container_probe.is_none() {
        return false;
    }

    let mut container_probe_started = false;
    for (name, requirement) in runtimes {
        if !requirement.active_for_os(target_os) {
            continue;
        }
        let required = requirement.required_for_os(target_os);
        let executable_candidates =
            runtime_executable_candidates(name, requirement.version_for_os(target_os));

        container_probe_started |= diagnose_command_version(
            "runtime",
            name,
            &executable_candidates,
            requirement.version_for_os(target_os),
            required,
            false,
            runtime_provider_hint(requirement, target_os),
            None,
            mode,
            selected_lifecycle,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            loaded_policy,
            target_os,
            false,
            false,
            provisioning_actions,
            findings,
        );
    }
    container_probe_started
}

fn diagnose_tools(
    contract: &Contract,
    selected_toolchains: &BTreeSet<String>,
    requirement_surface: &RequirementSurface,
    dependency_hydration_owned: bool,
    target_os: &str,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    mode: DoctorMode,
    selected_lifecycle: Option<Lifecycle>,
    container_probe: Option<&ContainerProbeContext>,
    remote_probe: Option<&ResolvedExecutionBackend>,
    remote_context_name: Option<&str>,
    provisioning_actions: &[ProvisioningAction],
    findings: &mut Vec<Finding>,
) -> bool {
    if mode == DoctorMode::Container && container_probe.is_none() {
        return false;
    }

    let mut container_probe_started = false;
    for (name, requirement) in &requirement_surface.tools {
        if !requirement.active_for_os(target_os) {
            continue;
        }
        let required = requirement.required_for_os(target_os);
        let executable_candidates = vec![tool_executable_name(name).to_string()];
        let toolchain_owned_requirement = selected_toolchain_owned_requirement_for_tool(
            contract,
            selected_toolchains,
            target_os,
            name,
        );
        let run_path_fulfillment_source = selected_toolchain_run_fulfillment_source_for_tool(
            contract,
            selected_toolchains,
            target_os,
            name,
        );
        let tool_acquisition = requirement.acquisition_for_os(target_os).or_else(|| {
            toolchain_owned_requirement
                .as_ref()
                .and_then(|requirement| requirement.acquisition_for_os(target_os))
        });
        let run_path_fulfillment_allowed = run_path_fulfillment_source.is_some()
            || tool_acquisition.is_some_and(|acquisition| {
                acquisition.provider == ToolAcquisitionProvider::Corepack
            });

        container_probe_started |= diagnose_command_version(
            "tool",
            name,
            &executable_candidates,
            requirement.version_for_os(target_os),
            required,
            false,
            run_path_fulfillment_source.map(toolchain_fulfillment_source_label),
            tool_acquisition,
            mode,
            selected_lifecycle,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            loaded_policy,
            target_os,
            run_path_fulfillment_allowed,
            dependency_hydration_owned,
            provisioning_actions,
            findings,
        );
    }
    for name in &requirement_surface.presence_only_tools {
        if requirement_surface.tools.keys().any(|owned_tool_name| {
            owned_tool_name == name || tool_executable_name(owned_tool_name) == name
        }) {
            continue;
        }
        let executable_candidates = vec![name.to_string()];
        container_probe_started |= diagnose_command_version(
            "tool",
            name,
            &executable_candidates,
            "*",
            true,
            true,
            None,
            None,
            mode,
            selected_lifecycle,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            loaded_policy,
            target_os,
            false,
            dependency_hydration_owned,
            provisioning_actions,
            findings,
        );
    }
    container_probe_started
}

fn selected_toolchain_owned_requirement_for_tool(
    contract: &Contract,
    selected_toolchains: &BTreeSet<String>,
    target_os: &str,
    tool_name: &str,
) -> Option<ToolRequirement> {
    selected_toolchains.iter().find_map(|toolchain_name| {
        let toolchain = contract.toolchains.get(toolchain_name.as_str())?;
        if !toolchain.active_for_os(target_os) {
            return None;
        }
        let provider = declared_toolchain_contract(toolchain_name, toolchain)?;
        provider
            .owned_tool_requirements(toolchain, target_os)
            .into_iter()
            .find_map(|(owned_tool_name, requirement)| {
                (owned_tool_name == tool_name
                    || tool_executable_name(&owned_tool_name) == tool_name)
                    .then_some(requirement)
            })
    })
}

fn selected_toolchain_run_fulfillment_source_for_tool(
    contract: &Contract,
    selected_toolchains: &BTreeSet<String>,
    target_os: &str,
    tool_name: &str,
) -> Option<ToolchainFulfillmentSource> {
    selected_toolchains.iter().find_map(|toolchain_name| {
        let toolchain = contract.toolchains.get(toolchain_name.as_str())?;
        if !toolchain.active_for_os(target_os)
            || !matches!(
                toolchain.fulfillment_mode(),
                crate::schema::ToolchainFulfillmentMode::Run
            )
        {
            return None;
        }
        let provider = declared_toolchain_contract(toolchain_name, toolchain)?;
        let fulfillable_tools = provider
            .owned_tool_requirements(toolchain, target_os)
            .keys()
            .filter(|owned_tool_name| {
                provider.provider() != crate::schema::ToolchainProvider::Uv
                    || matches!(owned_tool_name.as_str(), "uv" | "poetry")
            })
            .any(|owned_tool_name| {
                owned_tool_name == tool_name || tool_executable_name(owned_tool_name) == tool_name
            });
        fulfillable_tools
            .then(|| toolchain.fulfillment_source())
            .flatten()
    })
}

fn diagnose_toolchains(
    contract: &Contract,
    selected_toolchains: &BTreeSet<String>,
    target_os: &str,
    contract_path: &Path,
    mode: DoctorMode,
    container_probe: Option<&ContainerProbeContext>,
    remote_probe: Option<&ResolvedExecutionBackend>,
    remote_context_name: Option<&str>,
    findings: &mut Vec<Finding>,
) -> bool {
    let mut probe_started = false;
    for toolchain_name in selected_toolchains {
        let Some(toolchain) = contract.toolchains.get(toolchain_name.as_str()) else {
            continue;
        };
        let Some(provider) = declared_toolchain_contract(toolchain_name, toolchain) else {
            continue;
        };
        if !toolchain.active_for_os(target_os) {
            continue;
        }
        let executable_candidates = runtime_executable_candidates(
            provider.owned_runtime(),
            toolchain.version_for_os(target_os),
        );

        let run_path_fulfillment_source = (toolchain.fulfillment_mode()
            == crate::schema::ToolchainFulfillmentMode::Run
            && toolchain.fulfillment_source() == Some(ToolchainFulfillmentSource::Mise))
        .then_some(ToolchainFulfillmentSource::Mise);

        probe_started |= diagnose_command_version(
            "runtime",
            toolchain_name,
            &executable_candidates,
            toolchain.version_for_os(target_os),
            toolchain.required_for_os(target_os),
            false,
            Some(declared_toolchain_source_label(toolchain_name, toolchain)),
            None,
            mode,
            None,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            None,
            target_os,
            run_path_fulfillment_source.is_some(),
            false,
            &[],
            findings,
        );

        let surface_probes = provider.managed_surface_probes(toolchain, target_os);
        if surface_probes.is_empty() {
            continue;
        }
        probe_started = true;
        let mut provider_missing_reported = BTreeSet::new();
        for surface_probe in surface_probes {
            let probe_name = format!(
                "doctor-probe:{}:{}",
                provider.label(),
                surface_probe.kind.label()
            );
            match provider_installed_entries(
                probe_name.as_str(),
                surface_probe.command.as_str(),
                mode,
                container_probe,
                remote_probe,
                contract_path,
            ) {
                Ok(Some(installed_entries)) => {
                    for entry in surface_probe.required_entries {
                        if installed_entries.iter().any(|installed| {
                            installed == &entry
                                || (matches!(
                                    surface_probe.kind,
                                    ToolchainManagedSurfaceKind::Component
                                ) && installed.starts_with(&format!("{entry}-")))
                        }) {
                            continue;
                        }
                        findings.push(missing_toolchain_managed_surface_finding(
                            toolchain_name,
                            provider,
                            surface_probe.kind,
                            &entry,
                            mode,
                        ));
                    }
                }
                Ok(None) => {
                    if provider_missing_reported.insert(surface_probe.kind.label()) {
                        findings.push(missing_toolchain_provider_finding(
                            toolchain_name,
                            provider,
                            mode,
                            surface_probe.kind.label(),
                        ));
                    }
                }
                Err(details) => {
                    findings.push(toolchain_provider_probe_failed_finding(
                        toolchain_name,
                        provider,
                        surface_probe.command.as_str(),
                        details,
                        mode,
                    ));
                }
            }
        }
    }

    probe_started
}
fn doctor_probe_backend(
    mode: DoctorMode,
    container_probe: Option<&ContainerProbeContext>,
    remote_probe: Option<&ResolvedExecutionBackend>,
) -> Option<ResolvedExecutionBackend> {
    match mode {
        DoctorMode::Native => Some(ResolvedExecutionBackend::Native {
            shared_local_backend: None,
        }),
        DoctorMode::Container => container_probe.map(|probe| ResolvedExecutionBackend::Container {
            context_name: None,
            shared_local_backend: None,
            image: probe.image.clone(),
            engine: probe.engine.clone(),
            lifecycle: Lifecycle::Ephemeral,
            memory_bytes: None,
            compose_networks: Vec::new(),
            publications: Vec::new(),
            dependency_isolation_paths: Vec::new(),
        }),
        DoctorMode::Remote => remote_probe.cloned(),
    }
}

fn provider_installed_entries(
    probe_name: &str,
    command: &str,
    mode: DoctorMode,
    container_probe: Option<&ContainerProbeContext>,
    remote_probe: Option<&ResolvedExecutionBackend>,
    contract_path: &Path,
) -> Result<Option<BTreeSet<String>>, String> {
    let Some(backend) = doctor_probe_backend(mode, container_probe, remote_probe) else {
        return Ok(None);
    };
    // Split the probe command into argv and use direct exec on native backends so that
    // login-shell PATH reordering (e.g. macOS path_helper via `sh -lc`) does not cause
    // the wrong binary to be found. Container and remote backends still use shell.
    let mut parts = command.split_whitespace();
    let exe = parts.next().unwrap_or_default();
    let args: Vec<String> = parts.map(str::to_string).collect();
    let output = run_backend_argv_command_captured(
        probe_name,
        exe,
        &args,
        contract_working_dir(contract_path),
        &backend,
    )
    .map_err(|error| error.to_string())?;
    if output.exit_code == 127 {
        return Ok(None);
    }
    if output.exit_code != 0 {
        let details = format!("`{command}` exited with code {}", output.exit_code);
        return Err(details);
    }
    let mut installed = BTreeSet::new();
    for line in output.stdout.lines() {
        let entry = line
            .split_whitespace()
            .next()
            .map(str::trim)
            .unwrap_or_default();
        if !entry.is_empty() {
            installed.insert(entry.to_string());
        }
    }
    Ok(Some(installed))
}

fn missing_toolchain_provider_finding(
    toolchain_name: &str,
    provider: crate::toolchains::ToolchainProviderContract,
    mode: DoctorMode,
    surface: &str,
) -> Finding {
    let narrative = provider.missing_provider_diagnostic(
        toolchain_name,
        surface,
        &rerun_doctor_command(mode, None),
    );
    Finding::identified(
        "OTA_TOOLCHAIN_PROVIDER_MISSING",
        "environment",
        command_version_finding_owner(mode),
        FindingSeverity::Error,
        narrative.summary,
        narrative.why,
        narrative.next,
    )
}

fn toolchain_provider_probe_failed_finding(
    toolchain_name: &str,
    provider: crate::toolchains::ToolchainProviderContract,
    command: &str,
    details: String,
    mode: DoctorMode,
) -> Finding {
    let narrative = provider.probe_failed_diagnostic(
        toolchain_name,
        command,
        &details,
        &rerun_doctor_command(mode, None),
    );
    Finding::identified(
        "OTA_TOOLCHAIN_PROVIDER_PROBE_FAILED",
        "environment",
        command_version_finding_owner(mode),
        FindingSeverity::Error,
        narrative.summary,
        narrative.why,
        narrative.next,
    )
}

fn missing_toolchain_managed_surface_finding(
    toolchain_name: &str,
    provider: crate::toolchains::ToolchainProviderContract,
    kind: ToolchainManagedSurfaceKind,
    entry: &str,
    mode: DoctorMode,
) -> Finding {
    let narrative = provider.missing_managed_surface_diagnostic(
        toolchain_name,
        kind,
        entry,
        &rerun_doctor_command(mode, None),
    );
    let code = match kind {
        ToolchainManagedSurfaceKind::Component => "OTA_TOOLCHAIN_COMPONENT_MISSING",
        ToolchainManagedSurfaceKind::Target => "OTA_TOOLCHAIN_TARGET_MISSING",
    };
    Finding::identified(
        code,
        "environment",
        command_version_finding_owner(mode),
        FindingSeverity::Error,
        narrative.summary,
        narrative.why,
        narrative.next,
    )
}

fn diagnose_native_prerequisites(
    contract: &Contract,
    contract_path: &Path,
    native_names: &BTreeSet<String>,
    target_os: &str,
    findings: &mut Vec<Finding>,
) {
    if native_names.is_empty() {
        return;
    }

    let working_dir = contract_working_dir(contract_path);
    for name in native_names {
        let Some(prerequisite) = contract.native_prerequisites.get(name) else {
            continue;
        };
        if !prerequisite.active_for_os(target_os) {
            continue;
        }
        let check_name = prerequisite.check_for_os(target_os);
        let status = if let Some(check_name) = check_name {
            let Some(check) = contract
                .checks
                .iter()
                .find(|check| check.name == check_name)
            else {
                continue;
            };
            run_native_prerequisite_check(prerequisite, name, target_os, check, working_dir)
        } else if let Some(platform) = prerequisite.platform_for_os(target_os)
            && native_prerequisite_has_visual_studio_probe(platform)
        {
            run_visual_studio_native_prerequisite_check(platform)
        } else {
            continue;
        };
        let check_name = check_name.unwrap_or("visual_studio");
        match status {
            NativePrerequisiteCheckStatus::Passed => {}
            NativePrerequisiteCheckStatus::Failed(details) => {
                findings.push(native_prerequisite_finding(
                    name,
                    prerequisite,
                    check_name,
                    target_os,
                    false,
                    None,
                    details.as_deref(),
                ))
            }
            NativePrerequisiteCheckStatus::TimedOut(timeout) => {
                findings.push(native_prerequisite_finding(
                    name,
                    prerequisite,
                    check_name,
                    target_os,
                    true,
                    Some(timeout),
                    None,
                ))
            }
        }
    }
}

fn native_prerequisite_has_visual_studio_probe(
    platform: &crate::schema::NativePrerequisitePlatformSpec,
) -> bool {
    platform.visual_studio_build_tools || platform.visual_studio.is_some()
}

fn run_visual_studio_native_prerequisite_check(
    platform: &crate::schema::NativePrerequisitePlatformSpec,
) -> NativePrerequisiteCheckStatus {
    let Some(vswhere_path) = visual_studio_vswhere_path() else {
        return NativePrerequisiteCheckStatus::Failed(Some(String::from(
            "`vswhere.exe` was not found under Program Files (x86)",
        )));
    };
    if !vswhere_path.is_file() {
        return NativePrerequisiteCheckStatus::Failed(Some(format!(
            "`{}` was not found",
            vswhere_path.display()
        )));
    }

    let mut command = Command::new(vswhere_path);
    command.arg("-latest").arg("-products").arg("*");
    if let Some(visual_studio) = platform.visual_studio.as_ref() {
        for component in &visual_studio.components {
            command.arg("-requires").arg(component);
        }
    }
    command.arg("-property").arg("installationPath");

    match command.output() {
        Ok(output) if output.status.success() => {
            let installation_path = String::from_utf8_lossy(&output.stdout).trim().to_string();
            if installation_path.is_empty() {
                NativePrerequisiteCheckStatus::Failed(Some(String::from(
                    "vswhere did not report a Visual Studio installation path",
                )))
            } else {
                NativePrerequisiteCheckStatus::Passed
            }
        }
        Ok(output) => {
            let details = String::from_utf8_lossy(&output.stderr).trim().to_string();
            NativePrerequisiteCheckStatus::Failed(
                (!details.is_empty()).then_some(format!("vswhere failed: {details}")),
            )
        }
        Err(error) => {
            NativePrerequisiteCheckStatus::Failed(Some(format!("failed to run vswhere: {error}")))
        }
    }
}

fn visual_studio_vswhere_path() -> Option<PathBuf> {
    std::env::var_os("ProgramFiles(x86)")
        .map(PathBuf::from)
        .map(|program_files_x86| {
            program_files_x86
                .join("Microsoft Visual Studio")
                .join("Installer")
                .join("vswhere.exe")
        })
}

fn run_native_prerequisite_check(
    prerequisite: &crate::schema::NativePrerequisiteSpec,
    prerequisite_name: &str,
    target_os: &str,
    check: &crate::schema::CheckSpec,
    working_dir: &Path,
) -> NativePrerequisiteCheckStatus {
    if check.kind == crate::schema::CheckKind::File {
        return match run_file_check(check, working_dir) {
            CheckStatus::Passed => NativePrerequisiteCheckStatus::Passed,
            CheckStatus::Failed => NativePrerequisiteCheckStatus::Failed(None),
            CheckStatus::TimedOut(timeout) => NativePrerequisiteCheckStatus::TimedOut(timeout),
        };
    }

    let Some(command) = check.run.as_deref() else {
        return NativePrerequisiteCheckStatus::Failed(None);
    };

    let env_overrides = prerequisite
        .platform_for_os(target_os)
        .and_then(|platform| platform.activation.as_ref())
        .map(|activation| {
            capture_declared_native_activation_env(prerequisite_name, activation, working_dir)
        });

    let env_overrides = match env_overrides {
        Some(Ok(env)) => Some(env),
        Some(Err(error)) => {
            return NativePrerequisiteCheckStatus::Failed(Some(format!(
                "declared native activation failed before `{}` could run: {error}",
                check.name
            )));
        }
        None => None,
    };

    match run_check_with_env(command, working_dir, check.timeout, env_overrides.as_ref()) {
        DetailedCheckStatus::Passed => NativePrerequisiteCheckStatus::Passed,
        DetailedCheckStatus::Failed(details) => {
            NativePrerequisiteCheckStatus::Failed(check_failure_details_summary(&details))
        }
        DetailedCheckStatus::TimedOut(timeout) => NativePrerequisiteCheckStatus::TimedOut(timeout),
    }
}

fn native_prerequisite_finding(
    name: &str,
    prerequisite: &crate::schema::NativePrerequisiteSpec,
    check_name: &str,
    target_os: &str,
    timed_out: bool,
    timeout: Option<u64>,
    failure_details: Option<&str>,
) -> Finding {
    let summary = if timed_out {
        format!("Native prerequisite timed out: {name}")
    } else {
        format!("Native prerequisite missing: {name}")
    };
    let check_context = if let Some(timeout) = timeout {
        format!("check `{check_name}` did not finish within {timeout}ms")
    } else {
        format!("check `{check_name}` did not pass")
    };
    let description = prerequisite
        .description
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .unwrap_or("a selected workflow/task requires native OS tooling");
    let failure_suffix = failure_details
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(|value| format!(" (details: {value})"))
        .unwrap_or_default();
    Finding::identified(
        if timed_out {
            "OTA_NATIVE_PREREQUISITE_TIMED_OUT"
        } else {
            "OTA_NATIVE_PREREQUISITE_MISSING"
        },
        "environment",
        "host",
        if prerequisite.required {
            FindingSeverity::Error
        } else {
            FindingSeverity::Warn
        },
        summary,
        format!(
            "{description}; {check_context} on {target_os}, so ota cannot prove the native prerequisite is available{failure_suffix}"
        ),
        native_prerequisite_next(name, prerequisite, target_os),
    )
}

fn native_prerequisite_next(
    name: &str,
    prerequisite: &crate::schema::NativePrerequisiteSpec,
    target_os: &str,
) -> String {
    let Some(platform) = prerequisite.platform_for_os(target_os) else {
        return format!(
            "install or repair native prerequisite `{name}` for {target_os}, then rerun `ota doctor`"
        );
    };

    if let Some(command) = platform
        .install
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        return format!("run `{command}` and rerun `ota doctor`");
    }

    let mut suggestions = Vec::new();
    if platform.xcode_clt {
        suggestions.push(String::from("run `xcode-select --install`"));
    }
    if let Some(visual_studio) = platform.visual_studio.as_ref() {
        if visual_studio.components.is_empty() {
            suggestions.push(String::from("install Visual Studio Build Tools"));
        } else {
            suggestions.push(format!(
                "install Visual Studio Build Tools with components: `{}`",
                visual_studio.components.join(" ")
            ));
        }
    } else if platform.visual_studio_build_tools {
        suggestions.push(String::from("install Visual Studio Build Tools"));
    }
    if let Some(activation) = platform.activation.as_ref() {
        match activation.kind {
            crate::schema::NativePrerequisiteActivationKind::VisualStudioDevShell => {
                let arch = activation.arch.as_deref().unwrap_or("x64");
                suggestions.push(format!(
                    "activate the Visual Studio Developer shell for `{arch}` before running native checks"
                ));
            }
            crate::schema::NativePrerequisiteActivationKind::Command => {
                if let Some(shell) = activation.shell {
                    let shell = match shell {
                        crate::schema::NativePrerequisiteActivationShell::Sh => "sh",
                        crate::schema::NativePrerequisiteActivationShell::Bash => "bash",
                        crate::schema::NativePrerequisiteActivationShell::Zsh => "zsh",
                        crate::schema::NativePrerequisiteActivationShell::Pwsh => "pwsh",
                        crate::schema::NativePrerequisiteActivationShell::Cmd => "cmd",
                    };
                    suggestions.push(format!(
                        "run the declared `{shell}` activation path before rerunning native checks"
                    ));
                } else {
                    suggestions.push(String::from(
                        "run the declared native activation path before rerunning native checks",
                    ));
                }
            }
        }
    }
    if !platform.apt.is_empty() {
        suggestions.push(format!(
            "install apt packages: `{}`",
            platform.apt.join(" ")
        ));
    }
    if !platform.brew.is_empty() {
        suggestions.push(format!(
            "install Homebrew packages: `{}`",
            platform.brew.join(" ")
        ));
    }
    if !platform.winget.is_empty() {
        suggestions.push(format!(
            "install winget packages: `{}`",
            platform.winget.join(" ")
        ));
    }
    if !platform.choco.is_empty() {
        suggestions.push(format!(
            "install Chocolatey packages: `{}`",
            platform.choco.join(" ")
        ));
    }
    if !platform.scoop.is_empty() {
        suggestions.push(format!(
            "install Scoop packages: `{}`",
            platform.scoop.join(" ")
        ));
    }
    if let Some(note) = platform
        .note
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        suggestions.push(note.to_string());
    }

    if suggestions.is_empty() {
        format!("install or repair native prerequisite `{name}` and rerun `ota doctor`")
    } else {
        format!("{}; then rerun `ota doctor`", suggestions.join("; "))
    }
}

fn runtime_provider_hint<'a>(requirement: &'a RuntimeRequirement, os: &str) -> Option<&'a str> {
    requirement.provider_for_os(os)
}

fn command_version_finding_owner(mode: DoctorMode) -> &'static str {
    match mode {
        DoctorMode::Container => "container_target",
        DoctorMode::Remote => "remote_target",
        DoctorMode::Native => "host",
    }
}

fn command_version_code(kind: &str, status: &str) -> &'static str {
    match (kind, status) {
        ("runtime", "missing") => "OTA_RUNTIME_MISSING",
        ("runtime", "probe_failed") => "OTA_RUNTIME_PROBE_FAILED",
        ("runtime", "unparseable") => "OTA_RUNTIME_VERSION_UNPARSEABLE",
        ("runtime", "mismatch") => "OTA_RUNTIME_VERSION_MISMATCH",
        ("tool", "missing") => "OTA_TOOL_MISSING",
        ("tool", "probe_failed") => "OTA_TOOL_PROBE_FAILED",
        ("tool", "unparseable") => "OTA_TOOL_VERSION_UNPARSEABLE",
        ("tool", "mismatch") => "OTA_TOOL_VERSION_MISMATCH",
        _ => "OTA_DOCTOR_FINDING_UNKNOWN",
    }
}

fn diagnose_org_policy(
    contract: &Contract,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    policy_os: &str,
    requirement_surface: &RequirementSurface,
    toolchain_names: &BTreeSet<String>,
    native_names: &BTreeSet<String>,
    findings: &mut Vec<Finding>,
) -> Option<ProvisioningDiagnostics> {
    let Some(loaded_policy) = loaded_policy else {
        return None;
    };
    let policy_pack = &loaded_policy.pack;
    let policy_path = &loaded_policy.path;
    let policy_requirement_surface = policy_requirement_surface_for_toolchains(
        contract,
        requirement_surface,
        toolchain_names,
        policy_os,
    );

    let contract_root = contract_working_dir(contract_path);

    let missing_sections = policy_pack.missing_required_sections(contract);
    let missing_files = policy_pack.missing_required_files(contract_root);
    let version_violations = policy_pack.version_policy_violations_for_requirement_surface_os(
        policy_os,
        &policy_requirement_surface,
    );
    let native_package_plan =
        policy_pack.native_package_provisioning_plan_for_os(policy_os, contract, native_names);
    let native_package_violations: Vec<String> = native_package_plan
        .blocked
        .iter()
        .filter_map(|entry| entry.blocked_reason.clone())
        .collect();
    if missing_sections.is_empty()
        && missing_files.is_empty()
        && version_violations.is_empty()
        && native_package_violations.is_empty()
    {
        if !policy_pack.policies.version_policy.runtimes.is_empty()
            || !policy_pack.policies.version_policy.tools.is_empty()
        {
            let mut rules = Vec::new();
            for (name, rule) in &policy_pack.policies.version_policy.runtimes {
                let effective_versions = policy_pack
                    .effective_version_policy_versions_for_os(
                        policy_os,
                        crate::policy_pack::ProvisioningTargetKind::Runtime,
                        name,
                    )
                    .unwrap_or_else(|| rule.approved_versions.clone());
                let approved_versions = if effective_versions.is_empty() {
                    String::from("any version")
                } else {
                    format!("versions {}", effective_versions.join(", "))
                };
                rules.push(format!("runtime {name} ({approved_versions})"));
            }
            for (name, rule) in &policy_pack.policies.version_policy.tools {
                let effective_versions = policy_pack
                    .effective_version_policy_versions_for_os(
                        policy_os,
                        crate::policy_pack::ProvisioningTargetKind::Tool,
                        name,
                    )
                    .unwrap_or_else(|| rule.approved_versions.clone());
                let approved_versions = if effective_versions.is_empty() {
                    String::from("any version")
                } else {
                    format!("versions {}", effective_versions.join(", "))
                };
                rules.push(format!("tool {name} ({approved_versions})"));
            }

            findings.push(policy_finding(
                "OTA_POLICY_BACKED_VERSION_RULES_DECLARED",
                FindingSeverity::Info,
                "Policy-backed version rules are declared",
                format!(
                    "`{}` declares approved repo version rules: {}",
                    compact_display_path(&policy_path),
                    rules.join(", ")
                ),
                "use `ota policy review` to inspect the active policy source, or keep these approved version rules in mind when repo runtimes or tools need a governed version",
            ));
        }

        let mut provisioning_plan = policy_pack
            .provisioning_plan_for_requirement_surface_os(policy_os, &policy_requirement_surface);
        provisioning_plan
            .allowed
            .extend(native_package_plan.allowed.clone());
        provisioning_plan
            .blocked
            .extend(native_package_plan.blocked.clone());
        provisioning_plan
            .actions
            .extend(native_package_plan.actions.clone());
        let provisioning_request = ProvisioningBackendRequest {
            actions: provisioning_plan.actions.clone(),
        };

        let missing_packages: Vec<String> = provisioning_plan
            .blocked
            .iter()
            .filter_map(|entry| {
                entry.blocked_reason.as_ref().and_then(|reason| {
                    if reason.contains("requires an explicit `package`") {
                        Some(reason.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();

        if !missing_packages.is_empty() {
            findings.push(policy_finding(
                "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING",
                FindingSeverity::Warn,
                "Policy provisioning needs explicit package identifiers",
                format!(
                    "policy-backed provisioning cannot proceed for {}",
                    missing_packages.join("; ")
                ),
                "add `package` to the matching `policies.provisioning.<name>` rule or platform override, then rerun `ota doctor`",
            ));
        }

        if !policy_pack.policies.provisioning.is_empty() {
            let mut sources = Vec::new();
            for (name, rule) in &policy_pack.policies.provisioning {
                let versions = if rule.approved_versions.is_empty() {
                    String::from("any approved version")
                } else {
                    format!("versions {}", rule.approved_versions.join(", "))
                };
                let source_config = format_source_config_summary(rule.source_config.as_ref());
                let package_hint = rule
                    .package
                    .as_deref()
                    .map(|package| format!("package: {package}; "))
                    .unwrap_or_default();
                if source_config.is_empty() {
                    sources.push(format!(
                        "{name} via {} ({}{})",
                        rule.source, package_hint, versions
                    ));
                } else {
                    sources.push(format!(
                        "{name} via {} ({}{}; source_config: {source_config})",
                        rule.source, package_hint, versions
                    ));
                }
            }

            let matched_targets: Vec<String> = policy_pack
                .selected_provisioning_actions_for_requirement_surface_os(
                    policy_os,
                    &policy_requirement_surface,
                )
                .into_iter()
                .map(|entry| provisioning_action_audit_summary(&entry))
                .collect();

            findings.push(policy_finding(
                "OTA_POLICY_BACKED_PROVISIONING_DECLARED",
                FindingSeverity::Info,
                "Policy-backed provisioning sources are declared",
                if matched_targets.is_empty() {
                    format!(
                        "`{}` declares approved provisioning sources: {}",
                        compact_display_path(&policy_path),
                        sources.join(", ")
                    )
                } else {
                    format!(
                        "`{}` declares approved provisioning sources: {}. This repo's declared prerequisites can be provisioned through: {}",
                        compact_display_path(&policy_path),
                        sources.join(", "),
                        matched_targets.join(", ")
                    )
                },
                "use `ota policy review` to inspect the active policy source, or keep these approved sources in mind when repo prerequisites need a governed install path",
            ));
        }

        return Some(ProvisioningDiagnostics {
            plan: provisioning_plan,
            request: provisioning_request,
        });
    }

    let mut why_parts = Vec::new();
    if !missing_sections.is_empty() {
        why_parts.push(format!(
            "missing contract sections: {}",
            missing_sections.join(", ")
        ));
    }
    if !missing_files.is_empty() {
        why_parts.push(format!("missing files: {}", missing_files.join(", ")));
    }
    if !version_violations.is_empty() {
        why_parts.push(format!(
            "version policy violations: {}",
            version_violations.join("; ")
        ));
    }
    if !native_package_violations.is_empty() {
        for entry in &native_package_plan.blocked {
            let (Some(source), Some(package)) = (entry.source.as_deref(), entry.package.as_deref())
            else {
                continue;
            };
            findings.push(policy_finding(
                "OTA_POLICY_NATIVE_PACKAGE_NOT_APPROVED",
                FindingSeverity::Error,
                format!("Org policy does not approve native package: {source}:{package}"),
                entry.blocked_reason.clone().unwrap_or_else(|| {
                    format!(
                        "repo-native prerequisite requires {source} package `{package}`, but org policy does not approve it"
                    )
                }),
                format!(
                    "update the repo contract to use approved {source} package truth, or widen `{}#policies.native_packages.{source}.approved`",
                    compact_display_path(&policy_path)
                ),
            ));
        }
        why_parts.push(format!(
            "native package policy violations: {}",
            native_package_violations.join("; ")
        ));
    }

    findings.push(policy_finding(
        "OTA_POLICY_PACK_VIOLATION",
        FindingSeverity::Error,
        "Repo does not satisfy org policy pack",
        format!(
            "`{}` requires {}",
            compact_display_path(&policy_path),
            why_parts.join(" and ")
        ),
        if version_violations.is_empty() && native_package_violations.is_empty() {
            format!(
                "add the missing items or update `{}`",
                compact_display_path(&policy_path)
            )
        } else if missing_sections.is_empty() && missing_files.is_empty() {
            format!(
                "update the repo contract to match policy, or widen `{}`",
                compact_display_path(&policy_path)
            )
        } else {
            format!(
                "add the missing items, update the repo contract versions, or update `{}`",
                compact_display_path(&policy_path)
            )
        },
    ));

    None
}

fn policy_version_rules_for_requirement_surface_os(
    policy_pack: &crate::policy_pack::OrgPolicyPack,
    policy_os: &str,
    requirement_surface: &RequirementSurface,
) -> Vec<String> {
    let mut rules = Vec::new();

    for (name, requirement) in &requirement_surface.runtimes {
        if !requirement.required_for_os(policy_os) {
            continue;
        }
        let Some(effective_versions) = policy_pack.effective_version_policy_versions_for_os(
            policy_os,
            ProvisioningTargetKind::Runtime,
            name,
        ) else {
            continue;
        };
        let approved_versions = if effective_versions.is_empty() {
            String::from("any version")
        } else {
            format!("versions {}", effective_versions.join(", "))
        };
        rules.push(format!("runtime {name} ({approved_versions})"));
    }

    for (name, requirement) in &requirement_surface.tools {
        if !requirement.required_for_os(policy_os) {
            continue;
        }
        let Some(effective_versions) = policy_pack.effective_version_policy_versions_for_os(
            policy_os,
            ProvisioningTargetKind::Tool,
            name,
        ) else {
            continue;
        };
        let approved_versions = if effective_versions.is_empty() {
            String::from("any version")
        } else {
            format!("versions {}", effective_versions.join(", "))
        };
        rules.push(format!("tool {name} ({approved_versions})"));
    }

    rules
}

fn diagnose_remote_org_policy(
    contract: &Contract,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    remote_probe_contexts: &[RemoteProbeContext],
    findings: &mut Vec<Finding>,
) {
    let Some(loaded_policy) = loaded_policy else {
        return;
    };
    let policy_pack = &loaded_policy.pack;
    let policy_path = &loaded_policy.path;
    let contract_root = contract_working_dir(contract_path);

    let missing_sections = policy_pack.missing_required_sections(contract);
    let missing_files = policy_pack.missing_required_files(contract_root);
    if !missing_sections.is_empty() || !missing_files.is_empty() {
        let mut why_parts = Vec::new();
        if !missing_sections.is_empty() {
            why_parts.push(format!(
                "missing contract sections: {}",
                missing_sections.join(", ")
            ));
        }
        if !missing_files.is_empty() {
            why_parts.push(format!("missing files: {}", missing_files.join(", ")));
        }

        findings.push(policy_finding(
            "OTA_POLICY_PACK_VIOLATION",
            FindingSeverity::Error,
            "Repo does not satisfy org policy pack",
            format!(
                "`{}` requires {}",
                compact_display_path(policy_path),
                why_parts.join(" and ")
            ),
            format!(
                "add the missing items or update `{}`",
                compact_display_path(policy_path)
            ),
        ));
        return;
    }

    for remote_probe in remote_probe_contexts {
        let context_label = remote_policy_subject(remote_probe.context_name.as_deref());
        let version_violations = policy_pack.version_policy_violations_for_requirement_surface_os(
            &remote_probe.target_os,
            &remote_probe.policy_requirement_surface,
        );
        if !version_violations.is_empty() {
            findings.push(policy_finding(
                "OTA_POLICY_PACK_VIOLATION",
                FindingSeverity::Error,
                "Repo does not satisfy org policy pack",
                format!(
                    "`{}` requires {context_label} to stay within approved versions, but version policy violations: {}",
                    compact_display_path(policy_path),
                    version_violations.join("; ")
                ),
                format!(
                    "update the requirements for {context_label}, or widen `{}`",
                    compact_display_path(policy_path)
                ),
            ));
            continue;
        }

        let version_rules = policy_version_rules_for_requirement_surface_os(
            policy_pack,
            &remote_probe.target_os,
            &remote_probe.policy_requirement_surface,
        );
        if !version_rules.is_empty() {
            findings.push(policy_finding(
                "OTA_POLICY_BACKED_VERSION_RULES_DECLARED",
                FindingSeverity::Info,
                "Policy-backed version rules are declared",
                format!(
                    "`{}` declares approved repo version rules for {context_label}: {}",
                    compact_display_path(policy_path),
                    version_rules.join(", ")
                ),
                "use `ota policy review` to inspect the active policy source, or keep these approved version rules in mind when remote context requirements need a governed version",
            ));
        }

        let provisioning_plan = policy_pack.provisioning_plan_for_requirement_surface_os(
            &remote_probe.target_os,
            &remote_probe.policy_requirement_surface,
        );
        let missing_packages: Vec<String> = provisioning_plan
            .blocked
            .iter()
            .filter_map(|entry| {
                entry.blocked_reason.as_ref().and_then(|reason| {
                    if reason.contains("requires an explicit `package`") {
                        Some(reason.clone())
                    } else {
                        None
                    }
                })
            })
            .collect();

        if !missing_packages.is_empty() {
            findings.push(policy_finding(
                "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING",
                FindingSeverity::Warn,
                "Policy provisioning needs explicit package identifiers",
                format!(
                    "policy-backed provisioning cannot proceed for {context_label}: {}",
                    missing_packages.join("; ")
                ),
                format!(
                    "add `package` to the matching `policies.provisioning.<name>` rule or platform override, then rerun `ota doctor --mode remote` for {context_label}",
                ),
            ));
        }

        if !remote_probe.provisioning_actions.is_empty() {
            let matched_targets: Vec<String> = remote_probe
                .provisioning_actions
                .iter()
                .map(provisioning_action_audit_summary)
                .collect();
            findings.push(policy_finding(
                "OTA_POLICY_BACKED_PROVISIONING_DECLARED",
                FindingSeverity::Info,
                "Policy-backed provisioning sources are declared",
                format!(
                    "`{}` declares approved provisioning sources for {context_label}: {}. This repo's declared prerequisites can be provisioned through: {}",
                    compact_display_path(policy_path),
                    matched_targets.join(", "),
                    matched_targets.join(", ")
                ),
                "use `ota policy review` to inspect the active policy source, or keep these approved sources in mind when remote context prerequisites need a governed install path",
            ));
        }
    }
}

fn policy_target_os_for_mode(mode: DoctorMode) -> &'static str {
    match mode {
        DoctorMode::Native => current_os(),
        DoctorMode::Container => "linux",
        DoctorMode::Remote => current_os(),
    }
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

fn format_source_config_summary(
    source_config: Option<&std::collections::BTreeMap<String, serde_yaml::Value>>,
) -> String {
    let Some(source_config) = source_config else {
        return String::new();
    };

    source_config
        .iter()
        .map(|(key, value)| {
            let rendered = match value {
                serde_yaml::Value::Bool(value) => value.to_string(),
                serde_yaml::Value::Number(value) => value.to_string(),
                serde_yaml::Value::String(value) => value.clone(),
                other => serde_yaml::to_string(other)
                    .map(|value| value.trim().to_string())
                    .unwrap_or_else(|_| String::from("<unrenderable>")),
            };
            format!("{key}={rendered}")
        })
        .collect::<Vec<_>>()
        .join(", ")
}

fn diagnose_adapter_bootstrap(
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    findings: &mut Vec<Finding>,
) -> Option<AdapterBootstrapDiagnostics> {
    let Some(loaded_policy) = loaded_policy else {
        return None;
    };
    let policy_pack = &loaded_policy.pack;
    let policy_path = &loaded_policy.path;

    let adapter_names = policy_pack
        .policies
        .adapter_bootstrap
        .keys()
        .map(|name| name.as_str())
        .collect::<Vec<_>>();
    let plan = policy_pack.adapter_bootstrap_plan(&adapter_names);
    let request = policy_pack.adapter_bootstrap_backend_request(&adapter_names);

    if !request.actions.is_empty() {
        let sources: Vec<String> = request
            .actions
            .iter()
            .map(|action| format!("{} via {}", action.name, action.source))
            .collect();

        findings.push(policy_finding(
            "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED",
            FindingSeverity::Info,
            "Adapter bootstrap sources are declared",
            format!(
                "`{}` can bootstrap missing adapter binaries through: {}",
                compact_display_path(&policy_path),
                sources.join(", ")
            ),
            "use `ota policy review` to inspect the active policy source, or keep these approved bootstrap surfaces in mind when adapter install needs approval or audit",
        ));
    }

    Some(AdapterBootstrapDiagnostics { plan, request })
}

fn policy_error_finding(err: LoadPolicyPackError) -> Finding {
    policy_finding(
        "OTA_POLICY_PACK_INVALID",
        FindingSeverity::Error,
        "Invalid org policy pack",
        err.to_string(),
        format!(
            "repair `{}` and rerun `ota doctor`",
            compact_display_path(Path::new(err.path()))
        ),
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolVersionsManager {
    Asdf,
    Mise,
}

fn exact_tooling_remediation_fallback(
    target_kind: ProvisioningTargetKind,
    name: &str,
    requirement: &str,
    provider_hint: Option<&str>,
    contract_path: &Path,
    provisioning_actions: &[ProvisioningAction],
) -> Option<String> {
    if let Some(action) = provisioning_actions
        .iter()
        .find(|action| action.target_kind == target_kind && action.name == name)
        && let Some(command) = render_provisioning_action_command(action)
    {
        return Some(command);
    }

    let contract_root = contract_working_dir(contract_path);

    if let Some(command) = provider_hint_remediation(target_kind, name, requirement, provider_hint)
    {
        return Some(command);
    }

    match (target_kind, name) {
        (ProvisioningTargetKind::Runtime, "node") => {
            if contract_root.join(".nvmrc").is_file() {
                return Some(format!(
                    "nvm install {requirement} && nvm use {requirement}"
                ));
            }
            if contract_root.join(".node-version").is_file() {
                return Some(format!("nodenv install {requirement}"));
            }
            tool_versions_remediation(&contract_root, &["nodejs", "node"], requirement)
        }
        (ProvisioningTargetKind::Runtime, "python") => {
            if contract_root.join("uv.lock").is_file() {
                return Some(format!("uv python install {requirement}"));
            }
            if contract_root.join(".python-version").is_file() {
                return Some(format!("pyenv install {requirement}"));
            }
            tool_versions_remediation(&contract_root, &["python"], requirement)
        }
        (ProvisioningTargetKind::Runtime, "java") => {
            if let Some(version) = sdkman_candidate_version(&contract_root, "java") {
                return Some(format!("sdk install java {version}"));
            }
            tool_versions_remediation(&contract_root, &["java"], requirement)
        }
        (ProvisioningTargetKind::Runtime, "go") => {
            if contract_root.join(".go-version").is_file() {
                return Some(format!("goenv install {requirement}"));
            }
            tool_versions_remediation(&contract_root, &["go", "golang"], requirement)
        }
        (ProvisioningTargetKind::Runtime, "rust") => {
            tool_versions_remediation(&contract_root, &["rust"], requirement)
        }
        (ProvisioningTargetKind::Runtime, "ruby") => {
            if contract_root.join(".ruby-version").is_file() {
                return Some(format!("rbenv install {requirement}"));
            }
            tool_versions_remediation(&contract_root, &["ruby"], requirement)
        }
        (ProvisioningTargetKind::Runtime, "php") => {
            tool_versions_remediation(&contract_root, &["php"], requirement)
        }
        (ProvisioningTargetKind::Runtime, "dotnet") => {
            if let Some(command) = dotnet_remediation_command(&contract_root, requirement) {
                return Some(command);
            }
            tool_versions_remediation(&contract_root, &["dotnet"], requirement)
        }
        (ProvisioningTargetKind::Runtime, "elixir") => {
            tool_versions_remediation(&contract_root, &["elixir"], requirement)
        }
        (ProvisioningTargetKind::Runtime, _) => None,
        (ProvisioningTargetKind::Tool, "node") => {
            tool_versions_remediation(&contract_root, &["nodejs", "node"], requirement)
        }
        (ProvisioningTargetKind::Tool, "dotnet") => {
            if let Some(command) = dotnet_remediation_command(&contract_root, requirement) {
                return Some(command);
            }
            tool_versions_remediation(&contract_root, &["dotnet"], requirement)
        }
        (ProvisioningTargetKind::Tool, "maven") => {
            if let Some(version) = sdkman_candidate_version(&contract_root, "maven") {
                return Some(format!("sdk install maven {version}"));
            }
            tool_versions_remediation(&contract_root, &[name], requirement)
        }
        (ProvisioningTargetKind::Tool, name) => {
            tool_versions_remediation(&contract_root, &[name], requirement)
        }
    }
}

fn provider_hint_remediation(
    target_kind: ProvisioningTargetKind,
    name: &str,
    requirement: &str,
    provider_hint: Option<&str>,
) -> Option<String> {
    let provider = provider_hint?.trim().to_ascii_lowercase();

    if target_kind == ProvisioningTargetKind::Runtime
        && let Some(contract) = shipped_toolchain_contract_by_label(provider.as_str())
        && contract.owned_runtime() == name
        && let Some(command) = contract.owned_runtime_remediation_command(requirement)
    {
        return Some(command);
    }

    provider_hint_remediation_without_toolchains(
        target_kind,
        name,
        requirement,
        Some(provider.as_str()),
    )
}

fn provider_hint_remediation_without_toolchains(
    target_kind: ProvisioningTargetKind,
    name: &str,
    requirement: &str,
    provider_hint: Option<&str>,
) -> Option<String> {
    let provider = provider_hint?.trim().to_ascii_lowercase();

    match (target_kind, name, provider.as_str()) {
        (ProvisioningTargetKind::Runtime, "node", "volta") => {
            Some(format!("volta install node@{requirement}"))
        }
        (ProvisioningTargetKind::Runtime, "node", "nvm") => Some(format!(
            "nvm install {requirement} && nvm use {requirement}"
        )),
        (ProvisioningTargetKind::Runtime, "node", "nodenv") => {
            Some(format!("nodenv install {requirement}"))
        }
        (ProvisioningTargetKind::Runtime, "python", "uv") => {
            Some(format!("uv python install {requirement}"))
        }
        (ProvisioningTargetKind::Runtime, "python", "pyenv") => {
            Some(format!("pyenv install {requirement}"))
        }
        (ProvisioningTargetKind::Runtime, "java", "sdkman") => {
            Some(format!("sdk install java {requirement}"))
        }
        (ProvisioningTargetKind::Runtime, "go", "goenv") => {
            Some(format!("goenv install {requirement}"))
        }
        (ProvisioningTargetKind::Runtime, "ruby", "rbenv") => {
            Some(format!("rbenv install {requirement}"))
        }
        (ProvisioningTargetKind::Runtime, _, "asdf")
        | (ProvisioningTargetKind::Tool, _, "asdf") => {
            provider_tool_versions_remediation(ToolVersionsManager::Asdf, name, requirement)
        }
        (ProvisioningTargetKind::Runtime, _, "mise")
        | (ProvisioningTargetKind::Tool, _, "mise") => {
            provider_tool_versions_remediation(ToolVersionsManager::Mise, name, requirement)
        }
        _ => None,
    }
}

fn provider_hint_from_probe_path(path: &Path) -> Option<&'static str> {
    for component in path.components() {
        let value = component.as_os_str().to_string_lossy().to_ascii_lowercase();
        let value = value.trim_start_matches('.');
        if value == "mise" || value == "mise.exe" {
            return Some("mise");
        }
        if value == "asdf" || value == "asdf.exe" {
            return Some("asdf");
        }
        if value == "volta" {
            return Some("volta");
        }
        if value == "nodenv" {
            return Some("nodenv");
        }
        if value == "pyenv" {
            return Some("pyenv");
        }
    }
    None
}

fn unsupported_toolchain_fallback_tools(
    requirement_surface: &RequirementSurface,
    names: &[&str],
) -> Vec<String> {
    names
        .iter()
        .filter(|name| requirement_surface.tools.contains_key(**name))
        .map(|name| (*name).to_string())
        .collect()
}

fn unsupported_toolchain_opportunity_finding(
    ecosystem: &str,
    contract_root: &Path,
    requirement_surface: &RequirementSurface,
) -> Option<Finding> {
    let context = unsupported_toolchain_opportunity_context(ecosystem)?;
    if !requirement_surface
        .runtimes
        .contains_key(context.fallback_runtime)
    {
        return None;
    }

    let repo_signals = toolchain_repo_signals(contract_root, ecosystem);
    if repo_signals.is_empty() {
        return None;
    }

    let fallback_tools =
        unsupported_toolchain_fallback_tools(requirement_surface, context.fallback_tools);
    let fallback_model = if fallback_tools.is_empty() {
        format!("`runtimes.{}`", context.fallback_runtime)
    } else {
        format!(
            "`runtimes.{}` and {}",
            context.fallback_runtime,
            fallback_tools
                .iter()
                .map(|tool| format!("`tools.{tool}`"))
                .collect::<Vec<_>>()
                .join(", ")
        )
    };
    let signal_summary = repo_signals
        .iter()
        .map(|signal| format!("`{signal}`"))
        .collect::<Vec<_>>()
        .join(", ");
    let ecosystem_label = match ecosystem {
        "java" => "a JVM build surface",
        "python" => "a Python project surface",
        _ => "a managed ecosystem surface",
    };

    Some(Finding::identified(
        "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED",
        "contract",
        "repo_contract",
        FindingSeverity::Info,
        format!("Managed toolchain opportunity: {ecosystem}"),
        format!(
            "this repo uses {ecosystem_label} and currently models it through {fallback_model}; repo signals: {signal_summary}; ota does not ship a {ecosystem} toolchain provider yet"
        ),
        format!(
            "keep {fallback_model} for now; ota can model this more cleanly once {ecosystem} toolchain support is shipped"
        ),
    ))
}

fn diagnose_unsupported_toolchain_opportunities(
    contract: &Contract,
    contract_path: &Path,
    requirement_surface: &RequirementSurface,
    findings: &mut Vec<Finding>,
) {
    let contract_root = contract_working_dir(contract_path);
    for ecosystem in unsupported_toolchain_opportunity_ecosystems() {
        if contract.toolchains.contains_key(*ecosystem) {
            continue;
        }
        if let Some(finding) =
            unsupported_toolchain_opportunity_finding(ecosystem, contract_root, requirement_surface)
        {
            findings.push(finding);
        }
    }
}

fn provider_tool_versions_remediation(
    manager: ToolVersionsManager,
    name: &str,
    requirement: &str,
) -> Option<String> {
    let tool_name = match name {
        "node" => "node",
        "python" => "python",
        "java" => "java",
        "go" => "go",
        "rust" => "rust",
        "ruby" => "ruby",
        "php" => "php",
        "dotnet" => "dotnet",
        "elixir" => "elixir",
        other if !other.is_empty() => other,
        _ => return None,
    };

    match manager {
        ToolVersionsManager::Asdf => Some(format!("asdf install {tool_name} {requirement}")),
        ToolVersionsManager::Mise => Some(format!("mise install {tool_name}@{requirement}")),
    }
}

fn tool_versions_remediation(
    contract_root: &Path,
    candidate_names: &[&str],
    requirement: &str,
) -> Option<String> {
    let tool_name = tool_versions_entry(contract_root, candidate_names)?;
    match tool_versions_manager(contract_root)? {
        ToolVersionsManager::Asdf => Some(format!("asdf install {tool_name} {requirement}")),
        ToolVersionsManager::Mise => Some(format!("mise install {tool_name}@{requirement}")),
    }
}

fn tool_versions_manager(contract_root: &Path) -> Option<ToolVersionsManager> {
    if contract_root.join("mise.toml").is_file() || contract_root.join("mise.local.toml").is_file()
    {
        return Some(ToolVersionsManager::Mise);
    }

    let asdf_available = command_available("asdf");
    let mise_available = command_available("mise");

    match (asdf_available, mise_available) {
        (true, false) => Some(ToolVersionsManager::Asdf),
        (false, true) => Some(ToolVersionsManager::Mise),
        _ => None,
    }
}

fn sdkman_candidate_version(contract_root: &Path, candidate: &str) -> Option<String> {
    let contents = std::fs::read_to_string(contract_root.join(".sdkmanrc")).ok()?;
    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let Some((key, value)) = trimmed.split_once('=') else {
            continue;
        };
        if key.trim() != candidate {
            continue;
        }
        let version = value.trim().trim_start_matches('v');
        if !version.is_empty() {
            return Some(version.to_string());
        }
    }
    None
}

fn dotnet_global_json_version(contract_root: &Path) -> Option<String> {
    let contents = std::fs::read_to_string(contract_root.join("global.json")).ok()?;
    let json: JsonValue = serde_json::from_str(&contents).ok()?;
    let version = json
        .get("sdk")
        .and_then(|sdk| sdk.get("version"))
        .and_then(JsonValue::as_str)?
        .trim();
    if version.is_empty() {
        None
    } else {
        Some(version.to_string())
    }
}

fn dotnet_remediation_command(contract_root: &Path, requirement: &str) -> Option<String> {
    let global_json_version = dotnet_global_json_version(contract_root);
    if let Some(version) = global_json_version.as_deref()
        && version_matches(requirement, version)
    {
        return Some(dotnet_install_version_command(version));
    }
    if let Some(command) = dotnet_requirement_install_command(requirement) {
        return Some(command);
    }
    global_json_version
        .as_deref()
        .map(dotnet_install_version_command)
}

fn dotnet_requirement_install_command(requirement: &str) -> Option<String> {
    let trimmed = requirement.trim();
    if trimmed.is_empty() || trimmed == "*" {
        return None;
    }

    if dotnet_plain_version_token(trimmed) {
        let parts_count = trimmed.split('.').count();
        return Some(if parts_count >= 3 {
            dotnet_install_version_command(trimmed)
        } else {
            let channel = if parts_count == 1 {
                format!("{trimmed}.0")
            } else {
                trimmed.to_string()
            };
            dotnet_install_channel_command(&channel)
        });
    }

    let requirement = parse_semver_requirement(trimmed)?;
    let channel = dotnet_channel_from_requirement(&requirement)?;
    Some(dotnet_install_channel_command(&channel))
}

fn dotnet_plain_version_token(value: &str) -> bool {
    value
        .split('.')
        .all(|segment| !segment.is_empty() && segment.chars().all(|ch| ch.is_ascii_digit()))
}

fn dotnet_channel_from_requirement(requirement: &VersionReq) -> Option<String> {
    let comparator = requirement
        .comparators
        .iter()
        .filter(|comparator| {
            matches!(
                comparator.op,
                Op::Exact | Op::Caret | Op::Tilde | Op::GreaterEq | Op::Greater | Op::Wildcard
            )
        })
        .max_by_key(|comparator| {
            (
                comparator.major,
                comparator.minor.unwrap_or(0),
                comparator.patch.unwrap_or(0),
            )
        })?;
    Some(format!(
        "{}.{}",
        comparator.major,
        comparator.minor.unwrap_or(0)
    ))
}

fn dotnet_install_version_command(version: &str) -> String {
    if cfg!(windows) {
        format!(
            "powershell -ExecutionPolicy Bypass -Command \"iwr https://dot.net/v1/dotnet-install.ps1 -OutFile dotnet-install.ps1; ./dotnet-install.ps1 -Version {version}\""
        )
    } else {
        format!(
            "curl -fsSL https://dot.net/v1/dotnet-install.sh -o dotnet-install.sh && bash dotnet-install.sh --version {version}"
        )
    }
}

fn dotnet_install_channel_command(channel: &str) -> String {
    if cfg!(windows) {
        format!(
            "powershell -ExecutionPolicy Bypass -Command \"iwr https://dot.net/v1/dotnet-install.ps1 -OutFile dotnet-install.ps1; ./dotnet-install.ps1 -Channel {channel}\""
        )
    } else {
        format!(
            "curl -fsSL https://dot.net/v1/dotnet-install.sh -o dotnet-install.sh && bash dotnet-install.sh --channel {channel}"
        )
    }
}

fn corepack_activation_provider_probe_candidates() -> [String; 1] {
    [String::from("corepack")]
}

fn diagnose_command_version(
    kind: &str,
    display_name: &str,
    executable_candidates: &[String],
    requirement: &str,
    required: bool,
    presence_only: bool,
    provider_hint: Option<&str>,
    tool_acquisition: Option<&ToolAcquisitionSpec>,
    mode: DoctorMode,
    selected_lifecycle: Option<Lifecycle>,
    container_probe: Option<&ContainerProbeContext>,
    remote_probe: Option<&ResolvedExecutionBackend>,
    remote_context_name: Option<&str>,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    target_os: &str,
    run_path_fulfillment_allowed: bool,
    dependency_hydration_owned: bool,
    provisioning_actions: &[ProvisioningAction],
    findings: &mut Vec<Finding>,
) -> bool {
    let rerun_doctor = rerun_doctor_command(mode, selected_lifecycle);
    let unresolved_executable = executable_candidates
        .first()
        .cloned()
        .unwrap_or_else(|| display_name.to_string());
    let target_kind = match kind {
        "runtime" => ProvisioningTargetKind::Runtime,
        _ => ProvisioningTargetKind::Tool,
    };
    let exact_remediation = if mode == DoctorMode::Native {
        exact_tooling_remediation(
            target_kind,
            display_name,
            requirement,
            provider_hint,
            tool_acquisition,
            contract_path,
            provisioning_actions,
        )
    } else {
        None
    };

    let version_probe = if mode == DoctorMode::Native {
        Some(command_version_probe_candidates(
            executable_candidates,
            requirement,
            |candidate| {
                command_version_probe_in_working_dir(candidate, contract_working_dir(contract_path))
            },
        ))
    } else if mode == DoctorMode::Container {
        let Some(container_probe) = container_probe else {
            return false;
        };
        let backend = doctor_probe_backend(DoctorMode::Container, Some(container_probe), None)
            .expect("container mode should produce a probe backend when container probe exists");
        Some(command_version_probe_candidates(
            executable_candidates,
            requirement,
            |candidate| {
                command_version_probe_in_container(
                    &backend,
                    candidate,
                    contract_working_dir(contract_path),
                )
            },
        ))
    } else if mode == DoctorMode::Remote {
        let Some(remote_probe) = remote_probe else {
            return false;
        };
        Some(command_version_probe_candidates(
            executable_candidates,
            requirement,
            |candidate| {
                command_version_probe_in_remote(
                    remote_probe,
                    candidate,
                    contract_working_dir(contract_path),
                )
            },
        ))
    } else {
        None
    };
    let mut probe_started = version_probe
        .as_ref()
        .map(|probe| probe.probe_started)
        .unwrap_or(false);
    let finding_display_name = remote_context_name
        .map(|context_name| format!("{display_name} (context {context_name})"))
        .unwrap_or_else(|| display_name.to_string());
    let acquisition_provider_missing = matches!(mode, DoctorMode::Native)
        && tool_acquisition
            .is_some_and(|acquisition| !tool_acquisition_provider_available(acquisition));
    let actual = if let Some(probe) = version_probe.as_ref() {
        match &probe.outcome {
            CommandVersionProbeOutcome::Version(actual) => Some(actual.clone()),
            _ => None,
        }
    } else {
        None
    };

    let Some(actual) = actual else {
        if presence_only && let Some(probe) = version_probe.as_ref() {
            match &probe.outcome {
                CommandVersionProbeOutcome::ProbeFailed { error, .. }
                    if mode == DoctorMode::Container
                        && should_skip_presence_only_container_hydration_probe_failure(
                            unresolved_executable.as_str(),
                            dependency_hydration_owned,
                            error.as_deref(),
                        ) =>
                {
                    return probe_started;
                }
                CommandVersionProbeOutcome::Unparseable => return probe_started,
                CommandVersionProbeOutcome::ProbeFailed {
                    exit_code: Some(_),
                    error: None,
                } => return probe_started,
                _ => {}
            }
        }
        if acquisition_provider_missing && let Some(acquisition) = tool_acquisition {
            if acquisition.provider == ToolAcquisitionProvider::Corepack
                && corepack_provider_bootstrap_available()
            {
                return probe_started;
            }
            let provider_requirement = tool_acquisition_provider_requirement(acquisition);
            findings.push(Finding::identified(
                "OTA_TOOL_ACTIVATION_PROVIDER_MISSING",
                "environment",
                "host",
                if required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                format!("Missing tool activation provider: {provider_requirement}"),
                format!(
                    "{display_name} is required by the selected workflow/task prerequisites, but the contract acquires it through `{}` and `{provider_requirement}` is not available on PATH",
                    if acquisition.provider == ToolAcquisitionProvider::Command {
                        "command activation"
                    } else {
                        acquisition.provider.as_str()
                    }
                ),
                format!(
                    "install `{provider_requirement}` or change the tool acquisition path, then run `{}` and rerun `{rerun_doctor}`",
                    tool_acquisition_command(acquisition)
                ),
            ));
            return probe_started;
        }

        if run_path_fulfillment_allowed
            && kind == "tool"
            && provider_hint.is_some_and(|provider| provider == "corepack")
        {
            if corepack_provider_bootstrap_available() {
                return probe_started;
            }
            probe_started |= diagnose_command_version(
                "tool",
                "corepack",
                &corepack_activation_provider_probe_candidates(),
                "*",
                required,
                false,
                None,
                None,
                mode,
                selected_lifecycle,
                container_probe,
                remote_probe,
                remote_context_name,
                contract_path,
                loaded_policy,
                target_os,
                false,
                dependency_hydration_owned,
                provisioning_actions,
                findings,
            );
            return probe_started;
        }

        if run_path_fulfillment_allowed && provider_hint.is_some_and(|provider| provider == "mise")
        {
            probe_started |= diagnose_command_version(
                "tool",
                "mise",
                &[String::from("mise")],
                "*",
                required,
                false,
                None,
                None,
                mode,
                selected_lifecycle,
                container_probe,
                remote_probe,
                remote_context_name,
                contract_path,
                loaded_policy,
                target_os,
                false,
                dependency_hydration_owned,
                provisioning_actions,
                findings,
            );
            return probe_started;
        }

        if run_path_fulfillment_allowed
            && tool_acquisition.is_some_and(|acquisition| {
                acquisition.provider == ToolAcquisitionProvider::Corepack
            })
        {
            probe_started |= diagnose_command_version(
                "tool",
                "corepack",
                &corepack_activation_provider_probe_candidates(),
                "*",
                required,
                false,
                None,
                None,
                mode,
                selected_lifecycle,
                container_probe,
                remote_probe,
                remote_context_name,
                contract_path,
                loaded_policy,
                target_os,
                false,
                dependency_hydration_owned,
                provisioning_actions,
                findings,
            );
            return probe_started;
        }

        if mode == DoctorMode::Container
            && let Some(failure) = container_installability_failure(
                target_kind,
                display_name,
                requirement,
                container_probe,
                contract_path,
                provisioning_actions,
            )
        {
            findings.push(provisioning_installability_finding(
                &failure,
                &ProvisioningExecutionTarget::Container {
                    image: container_probe
                        .expect("container probe is present in container mode")
                        .image
                        .clone(),
                    engine: container_probe
                        .expect("container probe is present in container mode")
                        .engine
                        .clone(),
                    lifecycle: selected_lifecycle.unwrap_or(Lifecycle::Ephemeral),
                    container_name: None,
                },
                &rerun_doctor,
            ));
            return probe_started;
        }

        if let Some(probe) = version_probe.as_ref() {
            match &probe.outcome {
                CommandVersionProbeOutcome::Missing => {}
                CommandVersionProbeOutcome::ProbeFailed { exit_code, error } => {
                    if mode == DoctorMode::Container
                        && !probe.probe_started
                        && push_container_image_acquisition_finding(
                            findings,
                            container_probe,
                            error.as_deref(),
                            rerun_doctor.as_str(),
                        )
                    {
                        return probe_started;
                    }
                    let resolved_path = probe
                        .resolved_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| unresolved_executable.clone());
                    let (why, next) = match mode {
                        DoctorMode::Container => {
                            let image = container_probe
                                .map(|probe| probe.image.as_str())
                                .unwrap_or("unknown");
                            let why = if let Some(message) =
                                container_repo_dependency_hydration_probe_error(error.as_deref())
                            {
                                format!(
                                    "ota probed `{resolved_path}` inside container image `{image}` with `{}`, but the command resolved through repo-local dependency state that is not hydrated in the selected container path yet: {message}",
                                    probe.command
                                )
                            } else {
                                match (error.as_deref(), exit_code) {
                                    (Some(message), _) => format!(
                                        "ota probed `{resolved_path}` inside container image `{image}` with `{}`, but the command could not be executed: {message}",
                                        probe.command
                                    ),
                                    (None, Some(code)) => format!(
                                        "ota probed `{resolved_path}` inside container image `{image}` with `{}`, but the command exited with code {code} before ota could read a version",
                                        probe.command
                                    ),
                                    (None, None) => format!(
                                        "ota probed `{resolved_path}` inside container image `{image}` with `{}`, but the command failed before ota could read a version",
                                        probe.command
                                    ),
                                }
                            };
                            let next = container_probe_failure_next_step(
                                probe.command.as_str(),
                                resolved_path.as_str(),
                                rerun_doctor.as_str(),
                                error.as_deref(),
                            );
                            (why, next)
                        }
                        DoctorMode::Remote => {
                            let why = match (error.as_deref(), exit_code) {
                                (Some(message), _) => format!(
                                    "ota probed `{resolved_path}` through the declared remote backend with `{}`, but the command could not be executed: {message}",
                                    probe.command
                                ),
                                (None, Some(code)) => format!(
                                    "ota probed `{resolved_path}` through the declared remote backend with `{}`, but the command exited with code {code} before ota could read a version",
                                    probe.command
                                ),
                                (None, None) => format!(
                                    "ota probed `{resolved_path}` through the declared remote backend with `{}`, but the command failed before ota could read a version",
                                    probe.command
                                ),
                            };
                            let next = format!(
                                "run `{}` through the selected remote backend, inspect `{resolved_path}`, and make sure the probe succeeds before rerunning `{}`",
                                probe.command, rerun_doctor
                            );
                            (why, next)
                        }
                        DoctorMode::Native => {
                            let why = match (error.as_deref(), exit_code) {
                                (Some(message), _) => format!(
                                    "ota probed `{resolved_path}` with `{}`, but the command could not be executed: {message}",
                                    probe.command
                                ),
                                (None, Some(code)) => format!(
                                    "ota probed `{resolved_path}` with `{}`, but the command exited with code {code} before ota could read a version",
                                    probe.command
                                ),
                                (None, None) => format!(
                                    "ota probed `{resolved_path}` with `{}`, but the command failed before ota could read a version",
                                    probe.command
                                ),
                            };
                            let next = format!(
                                "run `{}` directly, inspect `{resolved_path}`, and make sure the probe succeeds before rerunning `{}`",
                                probe.command, rerun_doctor
                            );
                            (why, next)
                        }
                    };
                    findings.push(Finding::identified(
                        command_version_code(kind, "probe_failed"),
                        "environment",
                        command_version_finding_owner(mode),
                        if required {
                            FindingSeverity::Error
                        } else {
                            FindingSeverity::Warn
                        },
                        format!("{} probe failed: {finding_display_name}", kind_label(kind)),
                        why,
                        next,
                    ));
                    return probe_started;
                }
                CommandVersionProbeOutcome::Unparseable => {
                    let resolved_path = probe
                        .resolved_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| unresolved_executable.clone());
                    let (why, next) = match mode {
                        DoctorMode::Container => {
                            let image = container_probe
                                .map(|probe| probe.image.as_str())
                                .unwrap_or("unknown");
                            let why = format!(
                                "ota probed `{resolved_path}` inside container image `{image}` with `{}`, but the output did not contain a parseable version",
                                probe.command
                            );
                            let next = format!(
                                "run `{}` inside the selected container image, inspect `{resolved_path}`, and make sure the output contains a parseable version before rerunning `{}`",
                                probe.command, rerun_doctor
                            );
                            (why, next)
                        }
                        DoctorMode::Remote => {
                            let why = format!(
                                "ota probed `{resolved_path}` through the declared remote backend with `{}`, but the output did not contain a parseable version",
                                probe.command
                            );
                            let next = format!(
                                "run `{}` through the selected remote backend, inspect `{resolved_path}`, and make sure the output contains a parseable version before rerunning `{}`",
                                probe.command, rerun_doctor
                            );
                            (why, next)
                        }
                        DoctorMode::Native => {
                            let why = format!(
                                "ota probed `{resolved_path}` with `{}`, but the output did not contain a parseable version",
                                probe.command
                            );
                            let next = format!(
                                "run `{}` directly, inspect `{resolved_path}`, and make sure the output contains a parseable version before rerunning `{}`",
                                probe.command, rerun_doctor
                            );
                            (why, next)
                        }
                    };
                    findings.push(Finding::identified(
                        command_version_code(kind, "unparseable"),
                        "environment",
                        command_version_finding_owner(mode),
                        if required {
                            FindingSeverity::Error
                        } else {
                            FindingSeverity::Warn
                        },
                        format!("Unparseable version for {kind}: {finding_display_name}"),
                        why,
                        next,
                    ));
                    return probe_started;
                }
                CommandVersionProbeOutcome::Version(_) => {}
            }
        }
        if mode == DoctorMode::Remote
            && let Some(failure) = remote_installability_failure(
                target_kind,
                display_name,
                requirement,
                remote_probe,
                remote_context_name,
                contract_path,
                provisioning_actions,
            )
            && let Some(target) = remote_provisioning_target(remote_probe, remote_context_name)
        {
            findings.push(provisioning_installability_finding(
                &failure,
                &target,
                &rerun_doctor,
            ));
            return probe_started;
        }
        if provisionable_missing_command_is_covered(
            target_kind,
            display_name,
            requirement,
            mode,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            provisioning_actions,
        ) {
            return probe_started;
        }
        let container_image = container_probe.map(|probe| probe.image.as_str());
        findings.push(Finding::identified(
            command_version_code(kind, "missing"),
            "environment",
            command_version_finding_owner(mode),
            if required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            format!("Missing {kind}: {finding_display_name}"),
            match (mode, container_image) {
                (DoctorMode::Container, Some(image)) => format!(
                    "{display_name} is declared in the contract but is not available inside container image `{image}`"
                ),
                (DoctorMode::Container, None) => format!(
                    "{display_name} is declared in the contract but is not available inside the configured container image"
                ),
                (DoctorMode::Remote, _) => remote_context_name
                    .map(|context_name| {
                        format!(
                            "{display_name} is declared for remote context `{context_name}` but is not available through the declared remote backend"
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "{display_name} is declared for remote execution but is not available through the declared remote backend"
                        )
                    }),
                _ => format!("{display_name} is declared in the contract but is not available on PATH"),
            },
            match (mode, container_image) {
                (DoctorMode::Container, Some(image)) => format!(
                    "update `execution.backends.container.image` (currently `{image}`) so `{display_name}` is available, then rerun `{rerun_doctor}`"
                ),
                (DoctorMode::Container, None) => format!(
                    "update `execution.backends.container.image` so `{display_name}` is available, then rerun `{rerun_doctor}`"
                ),
                (DoctorMode::Remote, _) => remote_context_name
                    .map(|context_name| {
                        format!(
                            "make `{display_name}` available in remote context `{context_name}` and rerun `{rerun_doctor}`"
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "make `{display_name}` available in the selected remote backend and rerun `{rerun_doctor}`"
                        )
                    }),
                _ => exact_remediation
                    .map(|command| format!("run `{command}` and rerun `{rerun_doctor}`"))
                    .unwrap_or_else(|| {
                        format!(
                            "install {display_name} and make it available on PATH, then rerun `{rerun_doctor}`"
                        )
                    }),
            },
        ));
        return probe_started;
    };

    if version_matches(requirement, &actual) {
        if let Some(loaded_policy) = loaded_policy
            && let Some(policy_violation) = loaded_policy
                .pack
                .strict_version_compliance_violation_for_actual_version_os(
                    target_os,
                    target_kind,
                    display_name,
                    &actual,
                )
        {
            let probe_suffix = version_probe
                .as_ref()
                .and_then(|probe| {
                    let path = probe.resolved_path.as_ref()?;
                    Some(match (mode, container_probe.map(|probe| probe.image.as_str()), remote_context_name) {
                        (DoctorMode::Container, Some(image), _) => format!(
                            "; ota probed `{}` inside container image `{image}` with `{}`",
                            path.display(),
                            probe.command
                        ),
                        (DoctorMode::Container, None, _) => format!(
                            "; ota probed `{}` inside the configured container image with `{}`",
                            path.display(),
                            probe.command
                        ),
                        (DoctorMode::Remote, _, Some(context_name)) => format!(
                            "; ota probed `{}` through remote context `{context_name}` with `{}`",
                            path.display(),
                            probe.command
                        ),
                        (DoctorMode::Remote, _, None) => format!(
                            "; ota probed `{}` through the selected remote backend with `{}`",
                            path.display(),
                            probe.command
                        ),
                        _ => format!("; ota probed `{}` with `{}`", path.display(), probe.command),
                    })
                })
                .unwrap_or_default();
            findings.push(policy_finding(
                "OTA_POLICY_INSTALLED_VERSION_NONCOMPLIANT",
                if required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                format!("Installed {kind} is not compliant with org policy: {finding_display_name}"),
                format!(
                    "{display_name} resolved to `{actual}` and satisfies the repo contract `{requirement}`, but `{}` enforces strict version compliance and {policy_violation}{probe_suffix}",
                    compact_display_path(&loaded_policy.path)
                ),
                match mode {
                    DoctorMode::Container => format!(
                        "update the selected execution environment so `{display_name}` uses an approved version, or widen `{}`",
                        compact_display_path(&loaded_policy.path)
                    ),
                    DoctorMode::Remote => format!(
                        "update the selected remote environment so `{display_name}` uses an approved version, or widen `{}`",
                        compact_display_path(&loaded_policy.path)
                    ),
                    DoctorMode::Native => format!(
                        "install an approved `{display_name}` version or widen `{}`",
                        compact_display_path(&loaded_policy.path)
                    ),
                },
            ));
        }
        return probe_started;
    }

    if run_path_fulfillment_allowed
        && tool_acquisition
            .is_some_and(|acquisition| acquisition.provider == ToolAcquisitionProvider::Corepack)
    {
        if corepack_provider_bootstrap_available() {
            return probe_started;
        }
        let finding_count = findings.len();
        probe_started |= diagnose_command_version(
            "tool",
            "corepack",
            &corepack_activation_provider_probe_candidates(),
            "*",
            required,
            false,
            None,
            None,
            mode,
            selected_lifecycle,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            loaded_policy,
            target_os,
            false,
            dependency_hydration_owned,
            provisioning_actions,
            findings,
        );
        if findings.len() == finding_count {
            return probe_started;
        }
        return probe_started;
    }

    if run_path_fulfillment_allowed && provider_hint.is_some_and(|provider| provider == "mise") {
        let finding_count = findings.len();
        probe_started |= diagnose_command_version(
            "tool",
            "mise",
            &[String::from("mise")],
            "*",
            required,
            false,
            None,
            None,
            mode,
            selected_lifecycle,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            loaded_policy,
            target_os,
            false,
            dependency_hydration_owned,
            provisioning_actions,
            findings,
        );
        if findings.len() == finding_count {
            return probe_started;
        }
        return probe_started;
    }

    if mode == DoctorMode::Container
        && let Some(failure) = container_installability_failure(
            target_kind,
            display_name,
            requirement,
            container_probe,
            contract_path,
            provisioning_actions,
        )
    {
        findings.push(provisioning_installability_finding(
            &failure,
            &ProvisioningExecutionTarget::Container {
                image: container_probe
                    .expect("container probe is present in container mode")
                    .image
                    .clone(),
                engine: container_probe
                    .expect("container probe is present in container mode")
                    .engine
                    .clone(),
                lifecycle: selected_lifecycle.unwrap_or(Lifecycle::Ephemeral),
                container_name: None,
            },
            &rerun_doctor,
        ));
        return probe_started;
    }
    if mode == DoctorMode::Remote
        && let Some(failure) = remote_installability_failure(
            target_kind,
            display_name,
            requirement,
            remote_probe,
            remote_context_name,
            contract_path,
            provisioning_actions,
        )
        && let Some(target) = remote_provisioning_target(remote_probe, remote_context_name)
    {
        findings.push(provisioning_installability_finding(
            &failure,
            &target,
            &rerun_doctor,
        ));
        return probe_started;
    }

    if acquisition_provider_missing && let Some(acquisition) = tool_acquisition {
        if acquisition.provider == ToolAcquisitionProvider::Corepack
            && corepack_provider_bootstrap_available()
        {
            return probe_started;
        }
        let provider_requirement = tool_acquisition_provider_requirement(acquisition);
        findings.push(Finding::identified(
            "OTA_TOOL_ACTIVATION_PROVIDER_MISSING",
            "environment",
            "host",
            if required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            format!("Missing tool activation provider: {provider_requirement}"),
            format!(
                "{display_name} is required by the selected workflow/task prerequisites, but the contract upgrades it through `{}` and `{provider_requirement}` is not available on PATH",
                if acquisition.provider == ToolAcquisitionProvider::Command {
                    "command activation"
                } else {
                    acquisition.provider.as_str()
                }
            ),
            format!(
                "install `{provider_requirement}` or change the tool acquisition path, then run `{}` and rerun `{rerun_doctor}`",
                tool_acquisition_command(acquisition)
            ),
        ));
        return probe_started;
    }

    let container_image = container_probe.map(|probe| probe.image.as_str());
    let probe_suffix = version_probe
        .as_ref()
        .and_then(|probe| {
            let path = probe.resolved_path.as_ref()?;
            Some(match (mode, container_image, remote_context_name) {
                (DoctorMode::Container, Some(image), _) => format!(
                    "; ota probed `{}` inside container image `{image}` with `{}`",
                    path.display(),
                    probe.command
                ),
                (DoctorMode::Container, None, _) => format!(
                    "; ota probed `{}` inside the configured container image with `{}`",
                    path.display(),
                    probe.command
                ),
                (DoctorMode::Remote, _, Some(context_name)) => format!(
                    "; ota probed `{}` through remote context `{context_name}` with `{}`",
                    path.display(),
                    probe.command
                ),
                (DoctorMode::Remote, _, None) => format!(
                    "; ota probed `{}` through the selected remote backend with `{}`",
                    path.display(),
                    probe.command
                ),
                _ => format!("; ota probed `{}` with `{}`", path.display(), probe.command),
            })
        })
        .unwrap_or_default();
    let mismatch_remediation = version_probe
        .as_ref()
        .and_then(|probe| probe.resolved_path.as_deref())
        .and_then(provider_hint_from_probe_path)
        .and_then(|hint| {
            provider_hint_remediation_without_toolchains(
                target_kind,
                display_name,
                requirement,
                Some(hint),
            )
        })
        .or(exact_remediation);
    findings.push(Finding::identified(
        command_version_code(kind, "mismatch"),
        "environment",
        command_version_finding_owner(mode),
        if required {
            FindingSeverity::Error
        } else {
            FindingSeverity::Warn
        },
        format!("Version mismatch for {kind}: {finding_display_name}"),
        match (mode, container_image) {
            (DoctorMode::Container, Some(image)) => format!(
                "{display_name} resolved to `{actual}` inside container image `{image}` but the contract requires `{requirement}`"
            ),
            (DoctorMode::Container, None) => format!(
                "{display_name} resolved to `{actual}` inside the configured container image but the contract requires `{requirement}`"
            ),
            (DoctorMode::Remote, _) => remote_context_name
                .map(|context_name| {
                    format!(
                        "{display_name} resolved to `{actual}` through remote context `{context_name}` but the contract requires `{requirement}`{probe_suffix}"
                    )
                })
                .unwrap_or_else(|| {
                    format!(
                        "{display_name} resolved to `{actual}` through the selected remote backend but the contract requires `{requirement}`{probe_suffix}"
                    )
                }),
            _ => format!(
                "{display_name} resolved to `{actual}` but the contract requires `{requirement}`{probe_suffix}"
            ),
        },
        match (mode, container_image) {
            (DoctorMode::Container, Some(image)) => format!(
                "update `execution.backends.container.image` (currently `{image}`) so `{display_name}` satisfies `{requirement}`, then rerun `{rerun_doctor}`"
            ),
            (DoctorMode::Container, None) => format!(
                "update `execution.backends.container.image` so `{display_name}` satisfies `{requirement}`, then rerun `{rerun_doctor}`"
            ),
            (DoctorMode::Remote, _) => remote_context_name
                .map(|context_name| {
                    format!(
                        "update `{display_name}` in remote context `{context_name}` so it satisfies `{requirement}`, then rerun `{rerun_doctor}`"
                    )
                })
                .unwrap_or_else(|| {
                    format!(
                        "update `{display_name}` in the selected remote backend so it satisfies `{requirement}`, then rerun `{rerun_doctor}`"
                    )
                }),
            _ => mismatch_remediation
                .map(|command| format!("run `{command}` and rerun `{rerun_doctor}`"))
                .unwrap_or_else(|| {
                    format!(
                        "install a compatible {display_name} version that satisfies `{requirement}`, then rerun `{rerun_doctor}`"
                    )
                }),
        },
    ));
    probe_started
}

fn container_installability_failure(
    target_kind: ProvisioningTargetKind,
    display_name: &str,
    requirement: &str,
    container_probe: Option<&ContainerProbeContext>,
    contract_path: &Path,
    provisioning_actions: &[ProvisioningAction],
) -> Option<ProvisioningFailureDiagnosis> {
    let container_probe = container_probe?;
    let action =
        selected_provisioning_action(target_kind, display_name, requirement, provisioning_actions)?;
    let target = ProvisioningExecutionTarget::Container {
        image: container_probe.image.clone(),
        engine: container_probe.engine.clone(),
        lifecycle: Lifecycle::Ephemeral,
        container_name: None,
    };
    match probe_provisioning_installability_with_target(
        action,
        contract_working_dir(contract_path),
        &target,
    ) {
        Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => Some(diagnosis),
        _ => None,
    }
}

fn selected_provisioning_action<'a>(
    target_kind: ProvisioningTargetKind,
    display_name: &str,
    requirement: &str,
    provisioning_actions: &'a [ProvisioningAction],
) -> Option<&'a ProvisioningAction> {
    provisioning_actions.iter().find(|action| {
        action.target_kind == target_kind
            && action.name.eq_ignore_ascii_case(display_name)
            && (action.requested_version == requirement
                || requirement == "*"
                || action.requested_version == "*")
    })
}

fn provisionable_missing_command_is_covered(
    target_kind: ProvisioningTargetKind,
    display_name: &str,
    requirement: &str,
    mode: DoctorMode,
    container_probe: Option<&ContainerProbeContext>,
    remote_probe: Option<&ResolvedExecutionBackend>,
    remote_context_name: Option<&str>,
    contract_path: &Path,
    provisioning_actions: &[ProvisioningAction],
) -> bool {
    let Some(action) =
        selected_provisioning_action(target_kind, display_name, requirement, provisioning_actions)
    else {
        return false;
    };

    if !matches!(mode, DoctorMode::Native) {
        return true;
    }

    let target = match mode {
        DoctorMode::Native => ProvisioningExecutionTarget::Native,
        DoctorMode::Container => {
            let Some(container_probe) = container_probe else {
                return false;
            };
            ProvisioningExecutionTarget::Container {
                image: container_probe.image.clone(),
                engine: container_probe.engine.clone(),
                lifecycle: Lifecycle::Ephemeral,
                container_name: None,
            }
        }
        DoctorMode::Remote => {
            let Some(target) = remote_provisioning_target(remote_probe, remote_context_name) else {
                return false;
            };
            target
        }
    };

    probe_provisioning_installability_with_target(
        action,
        contract_working_dir(contract_path),
        &target,
    )
    .is_ok()
}

fn remote_provisioning_target(
    remote_probe: Option<&ResolvedExecutionBackend>,
    remote_context_name: Option<&str>,
) -> Option<ProvisioningExecutionTarget> {
    match remote_probe? {
        ResolvedExecutionBackend::Remote {
            shared_local_backend: _,
            provider,
            target,
            cwd,
            ssh,
        } => Some(ProvisioningExecutionTarget::Remote {
            provider: provider.clone(),
            provider_command: None,
            target: target.clone(),
            cwd: cwd.clone(),
            ssh: ssh.clone(),
            context_name: remote_context_name.map(str::to_string),
        }),
        ResolvedExecutionBackend::BackendProvider {
            shared_local_backend: _,
            provider,
            command,
            target,
            cwd,
        } => Some(ProvisioningExecutionTarget::Remote {
            provider: provider.clone(),
            provider_command: Some(command.clone()),
            target: target.clone(),
            cwd: cwd.clone(),
            ssh: None,
            context_name: remote_context_name.map(str::to_string),
        }),
        ResolvedExecutionBackend::Native { .. } | ResolvedExecutionBackend::Container { .. } => {
            None
        }
    }
}

fn container_probe_failure_next_step(
    probe_command: &str,
    resolved_path: &str,
    rerun_doctor: &str,
    error_message: Option<&str>,
) -> String {
    if container_repo_dependency_hydration_probe_error(error_message).is_some() {
        return format!(
            "hydrate the selected repo dependency lane inside the selected container path first (for example the setup step that installs repo dependencies), then rerun `{rerun_doctor}`"
        );
    }
    if let Some(mismatch_hint) = container_manifest_platform_mismatch_hint(error_message) {
        return format!("{mismatch_hint}, then rerun `{rerun_doctor}`");
    }
    format!(
        "run `{probe_command}` inside the selected container image, inspect `{resolved_path}`, and make sure the probe succeeds before rerunning `{rerun_doctor}`"
    )
}

fn container_repo_dependency_hydration_probe_error(error_message: Option<&str>) -> Option<&str> {
    let message = error_message?;
    let lower = message.to_ascii_lowercase();
    if lower.contains("bundler::gemnotfound")
        || (lower.contains("could not find") && lower.contains("in locally installed gems"))
        || lower.contains("local package.json exists, but node_modules missing")
    {
        return Some(message);
    }
    None
}

fn container_manifest_platform_mismatch_hint(error_message: Option<&str>) -> Option<String> {
    let message = error_message?;
    let lower = message.to_ascii_lowercase();
    if !lower.contains("no matching manifest for") {
        return None;
    }
    if lower.contains("no matching manifest for windows") {
        return Some(String::from(
            "the selected container image does not publish a Windows manifest for this engine request; switch Docker Desktop to Linux container mode or use a Windows-compatible image tag",
        ));
    }
    if lower.contains("no matching manifest for linux") {
        return Some(String::from(
            "the selected container image does not publish a Linux manifest for this engine request; use a Linux-compatible image tag or switch the engine/container platform",
        ));
    }
    Some(String::from(
        "the selected container image does not publish a manifest compatible with the current engine platform request; align image platform and engine mode",
    ))
}

fn remote_installability_failure(
    target_kind: ProvisioningTargetKind,
    display_name: &str,
    requirement: &str,
    remote_probe: Option<&ResolvedExecutionBackend>,
    remote_context_name: Option<&str>,
    contract_path: &Path,
    provisioning_actions: &[ProvisioningAction],
) -> Option<ProvisioningFailureDiagnosis> {
    let target = remote_provisioning_target(remote_probe, remote_context_name)?;
    let action =
        selected_provisioning_action(target_kind, display_name, requirement, provisioning_actions)?;
    match probe_provisioning_installability_with_target(
        action,
        contract_working_dir(contract_path),
        &target,
    ) {
        Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => Some(diagnosis),
        _ => None,
    }
}

fn push_container_image_acquisition_finding(
    findings: &mut Vec<Finding>,
    container_probe: Option<&ContainerProbeContext>,
    error_message: Option<&str>,
    rerun_doctor: &str,
) -> bool {
    let Some(container_probe) = container_probe else {
        return false;
    };
    let Some(message) = error_message else {
        return false;
    };
    if !is_container_image_acquisition_error(message) {
        return false;
    }

    let summary = format!("Container image unavailable: {}", container_probe.image);
    if findings.iter().any(|finding| finding.summary == summary) {
        return true;
    }

    let next = container_manifest_platform_mismatch_hint(Some(message))
        .map(|hint| format!("{hint}, then rerun `{rerun_doctor}`"))
        .unwrap_or_else(|| {
            format!(
                "run `{} pull {}` or fix container registry and engine network access, then rerun `{rerun_doctor}`",
                container_probe.engine, container_probe.image
            )
        });
    findings.push(Finding::identified(
        "OTA_CONTAINER_IMAGE_UNAVAILABLE",
        "execution",
        "container_target",
        FindingSeverity::Error,
        summary,
        format!(
            "ota could not start configured container image `{}` before running runtime/tool probes: {}",
            container_probe.image,
            compact_probe_failure_message(message)
        ),
        next,
    ));
    true
}

fn is_container_image_acquisition_error(message: &str) -> bool {
    let lower = message.to_ascii_lowercase();
    lower.contains("unable to find image")
        || lower.contains("pull access denied")
        || lower.contains("repository does not exist")
        || lower.contains("manifest unknown")
        || lower.contains("no matching manifest")
        || lower.contains("registry-1.docker.io")
        || lower.contains("context deadline exceeded")
        || lower.contains("client.timeout exceeded")
        || lower.contains("bad gateway")
}

fn compact_probe_failure_message(message: &str) -> String {
    message
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join(" ")
}

fn version_command_string(name: &str) -> String {
    let args = crate::runner::tool_version_probe_arg_sets(name)
        .first()
        .copied()
        .unwrap_or(&["--version"]);
    if args.is_empty() {
        return name.to_string();
    }
    format!("{name} {}", args.join(" "))
}

fn version_probe_command_string(name: &str) -> String {
    let quoted_name = shell_single_quote(name);
    let exec = crate::runner::tool_runtime_version_probe_commands(name, "\"$resolved\"");

    format!(
        "printf '%s\\n' '{CONTAINER_PROBE_STARTED_MARKER}' >&2\nresolved=\"$(command -v {quoted_name} 2>/dev/null)\" || exit 127\nprintf '%s%s\\n' '{CONTAINER_PROBE_PATH_MARKER}' \"$resolved\" >&2\n({exec})"
    )
}

fn command_version_probe_in_container(
    backend: &ResolvedExecutionBackend,
    name: &str,
    working_dir: &Path,
) -> CommandVersionProbe {
    let command = version_command_string(name);
    let output = run_backend_command_captured(
        &format!("doctor-probe:{name}"),
        version_probe_command_string(name).as_str(),
        working_dir,
        backend,
    );
    let (probe_started, resolved_path, outcome) = match output {
        Ok(output) => {
            let (probe_started, resolved_path, combined) =
                extract_container_probe_output(output.stdout.as_bytes(), output.stderr.as_bytes());
            let outcome = if output.exit_code == 0 {
                extract_version_token(&combined)
                    .map(CommandVersionProbeOutcome::Version)
                    .unwrap_or(CommandVersionProbeOutcome::Unparseable)
            } else if probe_started && resolved_path.is_none() {
                CommandVersionProbeOutcome::Missing
            } else {
                CommandVersionProbeOutcome::ProbeFailed {
                    exit_code: Some(output.exit_code),
                    error: Some(combined),
                }
            };
            (probe_started, resolved_path.map(PathBuf::from), outcome)
        }
        Err(error) => (
            false,
            None,
            CommandVersionProbeOutcome::ProbeFailed {
                exit_code: None,
                error: Some(error.to_string()),
            },
        ),
    };

    CommandVersionProbe {
        command,
        resolved_path,
        probe_started,
        outcome,
    }
}

fn command_version_probe_in_remote(
    backend: &ResolvedExecutionBackend,
    name: &str,
    working_dir: &Path,
) -> CommandVersionProbe {
    let command = version_command_string(name);
    let output = run_backend_command_captured(
        &format!("doctor-probe:{name}"),
        version_probe_command_string(name).as_str(),
        working_dir,
        backend,
    );
    let (probe_started, resolved_path, outcome) = match output {
        Ok(output) => {
            let (probe_started, resolved_path, combined) =
                extract_container_probe_output(output.stdout.as_bytes(), output.stderr.as_bytes());
            let outcome = if output.exit_code == 0 {
                extract_version_token(&combined)
                    .map(CommandVersionProbeOutcome::Version)
                    .unwrap_or(CommandVersionProbeOutcome::Unparseable)
            } else if probe_started && resolved_path.is_none() {
                CommandVersionProbeOutcome::Missing
            } else {
                CommandVersionProbeOutcome::ProbeFailed {
                    exit_code: Some(output.exit_code),
                    error: Some(combined),
                }
            };
            (probe_started, resolved_path.map(PathBuf::from), outcome)
        }
        Err(error) => (
            false,
            None,
            CommandVersionProbeOutcome::ProbeFailed {
                exit_code: None,
                error: Some(error.to_string()),
            },
        ),
    };

    CommandVersionProbe {
        command,
        resolved_path,
        probe_started,
        outcome,
    }
}

fn extract_container_probe_output(stdout: &[u8], stderr: &[u8]) -> (bool, Option<String>, String) {
    let (stdout_started, stdout_path, cleaned_stdout) =
        strip_container_probe_markers(&String::from_utf8_lossy(stdout));
    let (stderr_started, stderr_path, cleaned_stderr) =
        strip_container_probe_markers(&String::from_utf8_lossy(stderr));
    let probe_started = stdout_started || stderr_started;
    let resolved_path = stdout_path.or(stderr_path);
    let combined = format!("{cleaned_stdout} {cleaned_stderr}")
        .trim()
        .to_string();
    (probe_started, resolved_path, combined)
}

fn strip_container_probe_markers(stream: &str) -> (bool, Option<String>, String) {
    let mut probe_started = false;
    let mut resolved_path = None;
    let mut kept = Vec::new();
    for line in stream.lines() {
        if line.trim() == CONTAINER_PROBE_STARTED_MARKER {
            probe_started = true;
        } else if let Some(value) = line.strip_prefix(CONTAINER_PROBE_PATH_MARKER) {
            resolved_path = Some(value.trim().to_string());
        } else {
            kept.push(line);
        }
    }
    (probe_started, resolved_path, kept.join("\n"))
}

fn compact_display_path(path: &Path) -> String {
    let Ok(current_dir) = std::env::current_dir() else {
        return path.display().to_string();
    };

    path.strip_prefix(&current_dir)
        .map(|relative| {
            if relative.as_os_str().is_empty() {
                String::from(".")
            } else {
                relative.display().to_string()
            }
        })
        .unwrap_or_else(|_| path.display().to_string())
}

fn diagnose_backend_cli(name: &str, backend: &str, findings: &mut Vec<Finding>) {
    if command_available(name) {
        return;
    }

    findings.push(Finding::identified(
        "OTA_BACKEND_CLI_MISSING",
        "execution",
        "host",
        FindingSeverity::Error,
        format!("Missing execution backend CLI: {name}"),
        format!("{backend} requires `{name}` to be available on PATH"),
        format!("install {name} and make it available on PATH, then rerun `ota doctor`"),
    ));
}

fn diagnose_container_backend_cli(contract: &Contract, findings: &mut Vec<Finding>) {
    diagnose_container_backend_cli_for_candidates(container_engine_candidates(contract), findings);
}

fn diagnose_container_backend_cli_for_container(
    container: &ContainerBackend,
    findings: &mut Vec<Finding>,
) {
    diagnose_container_backend_cli_for_candidates(
        container_engine_candidates_from_backend(Some(container)),
        findings,
    );
}

fn diagnose_container_backend_cli_for_candidates(
    engines: Vec<String>,
    findings: &mut Vec<Finding>,
) {
    let available_engines = engines
        .iter()
        .filter(|engine| command_available(engine))
        .cloned()
        .collect::<Vec<_>>();
    if available_engines.is_empty() {
        let supported = engines.join(", ");
        findings.push(Finding::identified(
            "OTA_CONTAINER_BACKEND_CLI_MISSING",
            "execution",
            "host",
            FindingSeverity::Error,
            format!("Missing container execution backend CLI: {supported}"),
            format!(
                "container execution requires one of these CLIs to be available on PATH: {supported}"
            ),
            "install one of the supported container engines or use `--mode native` if the contract allows it, then rerun `ota doctor`",
        ));
        return;
    }
    let mut first_failure = None;
    for engine in available_engines {
        match container_backend_probe_failure(engine.as_str()) {
            None => return,
            Some(failure) if first_failure.is_none() => first_failure = Some(failure),
            Some(_) => {}
        }
    }
    if let Some(failure) = first_failure {
        findings.push(container_backend_unavailable_finding(
            failure.engine.as_str(),
            failure.details.as_str(),
        ));
    }
}

fn container_backend_unavailable_finding(engine: &str, details: &str) -> Finding {
    Finding::identified(
        "OTA_CONTAINER_BACKEND_UNAVAILABLE",
        "execution",
        "host",
        FindingSeverity::Error,
        format!("Container execution backend unavailable: {engine}"),
        format!(
            "container execution resolved `{engine}`, but `{} info` could not reach a usable container backend: {details}",
            engine
        ),
        "start or repair the selected container engine, or use `--mode native` if the contract allows it, then rerun `ota doctor`",
    )
}

fn tool_executable_name(name: &str) -> &str {
    match name {
        "maven" => "mvn",
        "bundler" => "bundle",
        _ => name,
    }
}

fn diagnose_checks(
    contract: &Contract,
    contract_path: &Path,
    scope: DoctorScope,
    workflow_name: Option<&str>,
    task_name: Option<&str>,
    findings: &mut Vec<Finding>,
    overrides: ExecutionOverrides,
) {
    let working_dir = contract_working_dir(contract_path);
    let finding_start_index = findings.len();
    let selected_checks = selected_workflow_check_names(contract, workflow_name, scope);
    let selected_signal_checks =
        selected_workflow_signal_check_names(contract, workflow_name, scope);
    let selected_precondition_checks = task_name
        .and_then(|task_name| {
            selected_task_run_requirement_check_names(contract, task_name, overrides)
        })
        .or_else(|| selected_task_requirement_check_names(contract, workflow_name));
    let selected_probes = selected_workflow_probe_names(contract, workflow_name, scope);
    let selected_signal_probes =
        selected_workflow_signal_probe_names(contract, workflow_name, scope);
    let selected_surfaces = selected_workflow_surface_names(contract, workflow_name, scope);
    let selected_signal_surfaces =
        selected_workflow_signal_surface_names(contract, workflow_name, scope);
    let scoped_workflow_targets = selected_checks.is_some()
        || selected_signal_checks.is_some()
        || selected_precondition_checks.is_some()
        || selected_probes.is_some()
        || selected_signal_probes.is_some()
        || selected_surfaces.is_some()
        || selected_signal_surfaces.is_some();
    let mut probes_executed_via_checks = BTreeSet::new();

    for check in &contract.checks {
        let is_selected_precondition = selected_precondition_checks
            .as_ref()
            .is_some_and(|selected| selected.contains(check.name.as_str()));
        let is_selected_check = selected_checks
            .as_ref()
            .is_some_and(|selected| selected.contains(check.name.as_str()));
        let is_selected_signal_check = selected_signal_checks
            .as_ref()
            .is_some_and(|selected| selected.contains(check.name.as_str()));
        let is_selected_signal = is_selected_signal_check && !is_selected_check;
        let has_explicit_workflow_check_selection =
            selected_checks.is_some() || selected_signal_checks.is_some();
        let is_explicitly_selected_check =
            is_selected_precondition || is_selected_check || is_selected_signal_check;

        if scope == DoctorScope::Preconditions && !is_precondition_style_check(check.kind) {
            continue;
        }
        if scope == DoctorScope::Preconditions {
            if let Some(selected) = selected_precondition_checks.as_ref()
                && !selected.contains(check.name.as_str())
            {
                continue;
            }
        } else if scope == DoctorScope::ChecksOnly {
            if is_selected_precondition {
                // Explicit task-scoped prerequisite checks participate in `ota check` for the
                // selected workflow without changing legacy check selection when none are declared.
            } else {
                if is_precondition_style_check(check.kind) {
                    continue;
                }
                if scoped_workflow_targets && !has_explicit_workflow_check_selection {
                    continue;
                }
                if has_explicit_workflow_check_selection && !is_explicitly_selected_check {
                    continue;
                }
            }
        } else {
            if is_precondition_style_check(check.kind) {
                if let Some(selected) = selected_precondition_checks.as_ref()
                    && !selected.contains(check.name.as_str())
                {
                    continue;
                }
            } else if scoped_workflow_targets && !has_explicit_workflow_check_selection {
                continue;
            } else if has_explicit_workflow_check_selection && !is_explicitly_selected_check {
                continue;
            }
        }

        if let Some(probe_name) = check.probe.as_deref() {
            probes_executed_via_checks.insert(probe_name.to_string());
        }

        match run_declared_check(contract, contract_path, check, working_dir, None, overrides) {
            CheckStatus::Passed => continue,
            CheckStatus::Failed => findings.push(check_status_finding(
                contract,
                workflow_name,
                check,
                if is_selected_signal {
                    FindingSeverity::Info
                } else {
                    map_check_severity(check.severity)
                },
                None,
            )),
            CheckStatus::TimedOut(timeout) => findings.push(check_status_finding(
                contract,
                workflow_name,
                check,
                if is_selected_signal {
                    FindingSeverity::Info
                } else {
                    map_check_severity(check.severity)
                },
                Some(timeout),
            )),
        }
    }

    let mut executed_workflow_probes = BTreeSet::new();
    if let Some(selected_probe_names) = selected_probes {
        for probe_name in selected_probe_names {
            if probes_executed_via_checks.contains(probe_name) {
                continue;
            }
            executed_workflow_probes.insert(probe_name.to_string());
            if contract.probe(probe_name).is_none() {
                continue;
            }
            let probe = contract
                .probe(probe_name)
                .expect("checked probe existence above");
            match run_named_probe(contract, contract_path, probe_name, None, overrides) {
                CheckStatus::Passed => continue,
                CheckStatus::Failed => findings.push(Finding::identified(
                    "OTA_WORKFLOW_PROBE_FAILED",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Error,
                    format!("Probe failed: {probe_name}"),
                    format!(
                        "the configured workflow readiness probe `{probe_name}` ({}) did not succeed",
                        probe_source_description(contract, probe_name)
                    ),
                    failed_probe_next(probe_name, probe),
                )),
                CheckStatus::TimedOut(timeout) => findings.push(Finding::identified(
                    "OTA_WORKFLOW_PROBE_TIMED_OUT",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Error,
                    format!("Probe timed out: {probe_name}"),
                    format!(
                        "the configured workflow readiness probe `{probe_name}` ({}) did not finish within {}ms",
                        probe_source_description(contract, probe_name),
                        timeout
                    ),
                    timed_out_probe_next(probe_name, probe),
                )),
            }
        }
    }

    if let Some(selected_signal_probe_names) = selected_signal_probes {
        for probe_name in selected_signal_probe_names {
            if probes_executed_via_checks.contains(probe_name)
                || executed_workflow_probes.contains(probe_name)
            {
                continue;
            }
            if contract.probe(probe_name).is_none() {
                continue;
            }
            let probe = contract
                .probe(probe_name)
                .expect("checked probe existence above");
            match run_named_probe(contract, contract_path, probe_name, None, overrides) {
                CheckStatus::Passed => continue,
                CheckStatus::Failed => findings.push(Finding::identified(
                    "OTA_WORKFLOW_SIGNAL_PROBE_FAILED",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Info,
                    format!("Signal probe failed: {probe_name}"),
                    format!(
                        "the configured workflow signal probe `{probe_name}` ({}) did not succeed",
                        probe_source_description(contract, probe_name)
                    ),
                    failed_probe_next(probe_name, probe),
                )),
                CheckStatus::TimedOut(timeout) => findings.push(Finding::identified(
                    "OTA_WORKFLOW_SIGNAL_PROBE_TIMED_OUT",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Info,
                    format!("Signal probe timed out: {probe_name}"),
                    format!(
                        "the configured workflow signal probe `{probe_name}` ({}) did not finish within {}ms",
                        probe_source_description(contract, probe_name),
                        timeout
                    ),
                    timed_out_probe_next(probe_name, probe),
                )),
            }
        }
    }

    let has_blocking_workflow_gate = findings[finding_start_index..]
        .iter()
        .any(|finding| finding.severity == FindingSeverity::Error);

    if has_blocking_workflow_gate {
        return;
    }

    if let Some(selected_surface_names) = selected_surfaces {
        for surface_name in selected_surface_names {
            let rerun_command = rerun_selected_workflow_doctor_command(workflow_name);
            match run_workflow_surface_readiness(
                contract,
                contract_path,
                workflow_name,
                surface_name,
                overrides,
            ) {
                Ok(observation) if observation.status == CheckStatus::Passed => continue,
                Ok(observation) if observation.status == CheckStatus::Failed => findings.push(Finding::identified(
                    "OTA_WORKFLOW_SURFACE_READINESS_FAILED",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Error,
                    format!("Surface readiness failed: {surface_name}"),
                    format!(
                        "the selected workflow surface `{surface_name}` on run task `{}` (backend `{}`; endpoint `{}:{}`) did not become ready after {} checks",
                        observation.run_task_name,
                        observation.backend_label,
                        observation.address,
                        observation.port,
                        observation.attempts
                    ),
                    format!(
                        "if workflow run task `{}` is still booting, wait and rerun `{}`; otherwise start or repair `{}` and rerun `{}`",
                        observation.run_task_name,
                        rerun_command,
                        observation.run_task_name,
                        rerun_command
                    ),
                )),
                Ok(observation) => findings.push(Finding::identified(
                    "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Error,
                    format!("Surface readiness timed out: {surface_name}"),
                    format!(
                        "the selected workflow surface `{surface_name}` on run task `{}` (backend `{}`; endpoint `{}:{}`) did not become ready within {}ms across {} checks",
                        observation.run_task_name,
                        observation.backend_label,
                        observation.address,
                        observation.port,
                        observation.timeout_ms,
                        observation.attempts
                    ),
                    format!(
                        "if workflow run task `{}` is still booting, wait and rerun `{}`; otherwise start or repair `{}` and rerun `{}`",
                        observation.run_task_name,
                        rerun_command,
                        observation.run_task_name,
                        rerun_command
                    ),
                )),
                Err(error) => findings.push(Finding::identified(
                    "OTA_WORKFLOW_SURFACE_READINESS_UNEVALUABLE",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Error,
                    format!("Surface readiness could not be evaluated: {surface_name}"),
                    format!(
                        "the selected workflow surface `{surface_name}` on run task `{}` could not be resolved or checked: {error}",
                        contract
                            .selected_workflow(workflow_name)
                            .and_then(|(_, workflow)| workflow.run.as_ref())
                            .map(|run| run.task.as_str())
                            .unwrap_or("-")
                    ),
                    format!(
                        "repair workflow run task `{}` surface attachment/readiness and rerun `{}`",
                        contract
                            .selected_workflow(workflow_name)
                            .and_then(|(_, workflow)| workflow.run.as_ref())
                            .map(|run| run.task.as_str())
                            .unwrap_or("-"),
                        rerun_command
                    ),
                )),
            }
        }
    }

    if let Some(selected_signal_surface_names) = selected_signal_surfaces {
        for surface_name in selected_signal_surface_names {
            let rerun_command = rerun_selected_workflow_doctor_command(workflow_name);
            match run_workflow_surface_readiness(
                contract,
                contract_path,
                workflow_name,
                surface_name,
                overrides,
            ) {
                Ok(observation) if observation.status == CheckStatus::Passed => continue,
                Ok(observation) if observation.status == CheckStatus::Failed => findings.push(Finding::identified(
                    "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Info,
                    format!("Signal surface readiness failed: {surface_name}"),
                    format!(
                        "the configured workflow signal surface `{surface_name}` on run task `{}` (backend `{}`; endpoint `{}:{}`) did not become ready after {} checks",
                        observation.run_task_name,
                        observation.backend_label,
                        observation.address,
                        observation.port,
                        observation.attempts
                    ),
                    format!(
                        "if workflow run task `{}` is still booting, wait and rerun `{}`; otherwise start or repair `{}` and rerun `{}`",
                        observation.run_task_name,
                        rerun_command,
                        observation.run_task_name,
                        rerun_command
                    ),
                )),
                Ok(observation) => findings.push(Finding::identified(
                    "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Info,
                    format!("Signal surface readiness timed out: {surface_name}"),
                    format!(
                        "the configured workflow signal surface `{surface_name}` on run task `{}` (backend `{}`; endpoint `{}:{}`) did not become ready within {}ms across {} checks",
                        observation.run_task_name,
                        observation.backend_label,
                        observation.address,
                        observation.port,
                        observation.timeout_ms,
                        observation.attempts
                    ),
                    format!(
                        "if workflow run task `{}` is still booting, wait and rerun `{}`; otherwise start or repair `{}` and rerun `{}`",
                        observation.run_task_name,
                        rerun_command,
                        observation.run_task_name,
                        rerun_command
                    ),
                )),
                Err(error) => findings.push(Finding::identified(
                    "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_UNEVALUABLE",
                    "execution",
                    "repo_contract",
                    FindingSeverity::Info,
                    format!("Signal surface readiness could not be evaluated: {surface_name}"),
                    format!(
                        "the configured workflow signal surface `{surface_name}` on run task `{}` could not be resolved or checked: {error}",
                        contract
                            .selected_workflow(workflow_name)
                            .and_then(|(_, workflow)| workflow.run.as_ref())
                            .map(|run| run.task.as_str())
                            .unwrap_or("-")
                    ),
                    format!(
                        "repair workflow run task `{}` surface attachment/readiness and rerun `{}`",
                        contract
                            .selected_workflow(workflow_name)
                            .and_then(|(_, workflow)| workflow.run.as_ref())
                            .map(|run| run.task.as_str())
                            .unwrap_or("-"),
                        rerun_command
                    ),
                )),
            }
        }
    }
}

fn failed_check_next(
    contract: &Contract,
    workflow_name: Option<&str>,
    check: &crate::schema::CheckSpec,
) -> String {
    if let Some(probe_name) = check.probe.as_deref()
        && let Some(probe) = contract.probe(probe_name)
    {
        return failed_probe_next(probe_name, probe);
    }
    if check.kind == crate::schema::CheckKind::ChangedFiles {
        let matchers = check
            .changed_files
            .as_ref()
            .map(|changed_files| changed_files.paths.join(", "))
            .unwrap_or_else(|| String::from("<unset>"));
        return format!(
            "update `changed_files.paths` for check `{}` or rerun in a range where [{matchers}] changed, then rerun `ota doctor`",
            check.name
        );
    }
    if check.kind == crate::schema::CheckKind::File {
        let path = check.path.as_deref().unwrap_or("-");
        if let Some(setup_task) = contract.selected_setup_task_name_for(workflow_name) {
            return format!(
                "run `ota up` or `ota run {setup_task}` to satisfy `{path}`, then rerun `ota doctor`"
            );
        }
        return format!(
            "satisfy file check `{}` for `{path}`, then rerun `ota doctor`",
            check.name
        );
    }
    if check.kind == crate::schema::CheckKind::Env {
        let path = check
            .env
            .as_ref()
            .map(|env| env.path.as_str())
            .unwrap_or("-");
        if let Some(setup_task) = contract.selected_setup_task_name_for(workflow_name) {
            return format!(
                "run `ota up` or `ota run {setup_task}` to regenerate `{path}` with contract-compatible env values, then rerun `ota doctor`"
            );
        }
        return format!(
            "regenerate `{path}` with contract-compatible env values for check `{}`, then rerun `ota doctor`",
            check.name
        );
    }

    let Some(command) = check.run.as_deref() else {
        return format!(
            "inspect check `{}` in `ota.yaml`, then rerun `ota doctor`",
            check.name
        );
    };

    if let Some(path) = missing_file_check_path(command) {
        if let Some(setup_task) = contract.selected_setup_task_name_for(workflow_name) {
            return format!(
                "run `ota up` or `ota run {setup_task}` to create `{path}`, then rerun `ota doctor`"
            );
        }
        return format!(
            "create `{path}` now, or declare a setup path with `ota assist wire-setup --run '<command>'`, then rerun `ota doctor`"
        );
    }

    format!(
        "run `{}` and fix the reported issue, then rerun `ota doctor`",
        command
    )
}

fn rerun_selected_workflow_doctor_command(workflow_name: Option<&str>) -> String {
    workflow_name
        .map(|workflow| format!("ota doctor --workflow {workflow}"))
        .unwrap_or_else(|| String::from("ota doctor"))
}

fn missing_file_check_path(command: &str) -> Option<&str> {
    let trimmed = command.trim();

    if let Some(rest) = trimmed.strip_prefix("test -f ") {
        let path = rest.trim();
        return (!path.is_empty() && !path.contains(char::is_whitespace)).then_some(path);
    }

    if let Some(rest) = trimmed.strip_prefix("[ -f ") {
        let path = rest.strip_suffix(" ]")?.trim();
        return (!path.is_empty() && !path.contains(char::is_whitespace)).then_some(path);
    }

    None
}

fn selected_workflow_check_names<'a>(
    contract: &'a Contract,
    workflow_name: Option<&str>,
    scope: DoctorScope,
) -> Option<BTreeSet<&'a str>> {
    if scope == DoctorScope::Preconditions {
        return None;
    }

    let (_, workflow) = contract.selected_workflow(workflow_name)?;
    if workflow.readiness.checks.is_empty() && workflow.readiness.probes.is_empty() {
        return None;
    }

    Some(
        workflow
            .readiness
            .checks
            .iter()
            .map(|check| check.as_str())
            .collect(),
    )
}

fn selected_workflow_signal_check_names<'a>(
    contract: &'a Contract,
    workflow_name: Option<&str>,
    scope: DoctorScope,
) -> Option<BTreeSet<&'a str>> {
    if scope == DoctorScope::Preconditions {
        return None;
    }

    let (_, workflow) = contract.selected_workflow(workflow_name)?;
    if workflow.readiness.signal.checks.is_empty() {
        return None;
    }

    Some(
        workflow
            .readiness
            .signal
            .checks
            .iter()
            .map(|check| check.as_str())
            .collect(),
    )
}

fn selected_task_requirement_check_names(
    contract: &Contract,
    workflow_name: Option<&str>,
) -> Option<BTreeSet<String>> {
    let mut scoped = false;
    let mut selected = BTreeSet::new();
    for task_name in contract.selected_workflow_task_closure_names(workflow_name) {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        let backend =
            effective_task_execution(contract, task_name.as_str(), ExecutionOverrides::default())
                .backend;
        let context_name = task.context_for_backend(contract.execution.as_ref(), backend);
        let scoped_checks = task.scoped_check_requirements_for_execution(backend, context_name);
        let scoped_native = task.scoped_native_requirements_for_execution(backend, context_name);
        if !scoped_checks.is_empty() || !scoped_native.is_empty() {
            scoped = true;
        }
        selected.extend(scoped_checks);
        let native_checks =
            contract.native_prerequisite_required_check_names_for_os(scoped_native, current_os());
        if !native_checks.is_empty() {
            scoped = true;
            selected.extend(native_checks);
        }
    }

    if workflow_name.is_none() {
        (scoped || !selected.is_empty()).then_some(selected)
    } else {
        Some(selected)
    }
}

fn selected_task_run_requirement_check_names(
    contract: &Contract,
    task_name: &str,
    overrides: ExecutionOverrides,
) -> Option<BTreeSet<String>> {
    let task_names = contract.task_dependency_closure_names([task_name.to_string()]);
    if task_names.is_empty() {
        return None;
    }

    let mut selected = BTreeSet::new();
    for task_name in task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        let effective = effective_task_execution(contract, task_name.as_str(), overrides);
        selected.extend(
            task.scoped_check_requirements_for_execution(effective.backend, effective.context_name),
        );
        if matches!(effective.backend, Backend::Native) {
            let scoped_native = task.scoped_native_requirements_for_execution(
                effective.backend,
                effective.context_name,
            );
            selected.extend(
                contract
                    .native_prerequisite_required_check_names_for_os(scoped_native, current_os()),
            );
        }
    }

    Some(selected)
}

fn selected_workflow_probe_names<'a>(
    contract: &'a Contract,
    workflow_name: Option<&str>,
    scope: DoctorScope,
) -> Option<BTreeSet<&'a str>> {
    if scope == DoctorScope::Preconditions {
        return None;
    }

    let (_, workflow) = contract.selected_workflow(workflow_name)?;
    if workflow.readiness.probes.is_empty() {
        return None;
    }

    Some(
        workflow
            .readiness
            .probes
            .iter()
            .map(|probe| probe.as_str())
            .collect(),
    )
}

fn selected_workflow_signal_probe_names<'a>(
    contract: &'a Contract,
    workflow_name: Option<&str>,
    scope: DoctorScope,
) -> Option<BTreeSet<&'a str>> {
    if scope == DoctorScope::Preconditions {
        return None;
    }

    let (_, workflow) = contract.selected_workflow(workflow_name)?;
    if workflow.readiness.signal.probes.is_empty() {
        return None;
    }

    Some(
        workflow
            .readiness
            .signal
            .probes
            .iter()
            .map(|probe| probe.as_str())
            .collect(),
    )
}

fn selected_workflow_surface_names<'a>(
    contract: &'a Contract,
    workflow_name: Option<&str>,
    scope: DoctorScope,
) -> Option<BTreeSet<&'a str>> {
    if scope == DoctorScope::Preconditions {
        return None;
    }

    let (_, workflow) = contract.selected_workflow(workflow_name)?;
    if workflow.readiness.surfaces.is_empty() {
        return None;
    }

    Some(
        workflow
            .readiness
            .surfaces
            .iter()
            .map(|surface| surface.as_str())
            .collect(),
    )
}

fn selected_workflow_signal_surface_names<'a>(
    contract: &'a Contract,
    workflow_name: Option<&str>,
    scope: DoctorScope,
) -> Option<BTreeSet<&'a str>> {
    if scope == DoctorScope::Preconditions {
        return None;
    }

    let (_, workflow) = contract.selected_workflow(workflow_name)?;
    if workflow.readiness.signal.surfaces.is_empty() {
        return None;
    }

    Some(
        workflow
            .readiness
            .signal
            .surfaces
            .iter()
            .map(|surface| surface.as_str())
            .collect(),
    )
}

fn selected_workflow_service_names(
    contract: &Contract,
    workflow_name: Option<&str>,
) -> Option<BTreeSet<String>> {
    let _ = contract.selected_workflow(workflow_name)?;
    Some(
        contract
            .selected_workflow_required_service_names(workflow_name)
            .into_iter()
            .collect(),
    )
}

fn is_precondition_style_check(kind: CheckKind) -> bool {
    matches!(
        kind,
        CheckKind::Precondition | CheckKind::Env | CheckKind::ChangedFiles
    )
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CheckStatus {
    Passed,
    Failed,
    TimedOut(u64),
}

enum NativePrerequisiteCheckStatus {
    Passed,
    Failed(Option<String>),
    TimedOut(u64),
}

struct CheckFailureDetails {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
}

enum DetailedCheckStatus {
    Passed,
    Failed(CheckFailureDetails),
    TimedOut(u64),
}

fn run_declared_check(
    contract: &Contract,
    contract_path: &Path,
    check: &crate::schema::CheckSpec,
    working_dir: &Path,
    command_override: Option<&str>,
    overrides: ExecutionOverrides,
) -> CheckStatus {
    if check.kind == crate::schema::CheckKind::ChangedFiles {
        return run_changed_files_check(check, working_dir);
    }
    if check.kind == crate::schema::CheckKind::File {
        return run_file_check(check, working_dir);
    }
    if check.kind == crate::schema::CheckKind::Env {
        return run_env_check(check, working_dir);
    }
    if let Some(command) = command_override.or(check.run.as_deref()) {
        return run_check(command, working_dir, check.timeout);
    }
    if let Some(probe_name) = check.probe.as_deref()
        && contract.probe(probe_name).is_some()
    {
        return run_named_probe(
            contract,
            contract_path,
            probe_name,
            check.timeout,
            overrides,
        );
    }
    CheckStatus::Failed
}

fn run_changed_files_check(check: &crate::schema::CheckSpec, working_dir: &Path) -> CheckStatus {
    let Some(changed_files) = check.changed_files.as_ref() else {
        return CheckStatus::Failed;
    };
    if changed_files.paths.is_empty() {
        return CheckStatus::Failed;
    }

    let mut diff = Command::new("git");
    diff.arg("-C")
        .arg(working_dir)
        .arg("diff")
        .arg("--name-only");
    if let Some(base_ref) = changed_files
        .base_ref
        .as_deref()
        .map(str::trim)
        .filter(|value| !value.is_empty())
    {
        let head_ref = changed_files
            .head_ref
            .as_deref()
            .map(str::trim)
            .filter(|value| !value.is_empty())
            .unwrap_or("HEAD");
        diff.arg(format!("{base_ref}..{head_ref}"));
    } else {
        diff.arg("HEAD");
    }
    diff.arg("--");
    for matcher in &changed_files.paths {
        diff.arg(matcher);
    }

    let diff_output = match diff.output() {
        Ok(output) => output,
        Err(_) => return CheckStatus::Failed,
    };
    if !diff_output.status.success() {
        return CheckStatus::Failed;
    }
    if !String::from_utf8_lossy(&diff_output.stdout)
        .trim()
        .is_empty()
    {
        return CheckStatus::Passed;
    }

    if changed_files.include_untracked {
        let mut untracked = Command::new("git");
        untracked
            .arg("-C")
            .arg(working_dir)
            .arg("ls-files")
            .arg("--others")
            .arg("--exclude-standard")
            .arg("--");
        for matcher in &changed_files.paths {
            untracked.arg(matcher);
        }
        let output = match untracked.output() {
            Ok(output) => output,
            Err(_) => return CheckStatus::Failed,
        };
        if !output.status.success() {
            return CheckStatus::Failed;
        }
        if !String::from_utf8_lossy(&output.stdout).trim().is_empty() {
            return CheckStatus::Passed;
        }
    }

    CheckStatus::Failed
}

fn run_file_check(check: &crate::schema::CheckSpec, working_dir: &Path) -> CheckStatus {
    let Some(path) = check
        .path
        .as_deref()
        .map(str::trim)
        .filter(|path| !path.is_empty())
    else {
        return CheckStatus::Failed;
    };
    let target = match check.scope.unwrap_or(crate::schema::FileCheckScope::Repo) {
        crate::schema::FileCheckScope::Repo => working_dir.join(path),
        crate::schema::FileCheckScope::Workspace => working_dir.join(path),
    };
    match check
        .expect
        .unwrap_or(crate::schema::FileCheckExpectation::Exists)
    {
        crate::schema::FileCheckExpectation::Exists => {
            if target.exists() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            }
        }
        crate::schema::FileCheckExpectation::File => {
            if target.is_file() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            }
        }
        crate::schema::FileCheckExpectation::Directory => {
            if target.is_dir() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            }
        }
        crate::schema::FileCheckExpectation::Missing => {
            if !target.exists() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            }
        }
    }
}

fn run_env_check(check: &crate::schema::CheckSpec, working_dir: &Path) -> CheckStatus {
    if evaluate_declared_env_check(check, working_dir) {
        CheckStatus::Passed
    } else {
        CheckStatus::Failed
    }
}

fn run_named_probe(
    contract: &Contract,
    contract_path: &Path,
    probe_name: &str,
    timeout_override_ms: Option<u64>,
    overrides: ExecutionOverrides,
) -> CheckStatus {
    let Some(probe) = contract.probe(probe_name) else {
        return CheckStatus::Failed;
    };
    if probe_uses_task_observer(probe) {
        return run_observer_task_named_probe(
            contract,
            contract_path,
            probe_name,
            probe,
            timeout_override_ms,
            overrides,
        );
    }
    let Ok(resolved) = resolve_named_readiness_probe(contract, probe_name) else {
        return CheckStatus::Failed;
    };
    let timeout_ms = timeout_override_ms.or(match &resolved {
        ResolvedNamedReadinessProbe::Http { timeout, .. }
        | ResolvedNamedReadinessProbe::Tcp { timeout, .. } => {
            timeout.map(|duration| duration.as_millis() as u64)
        }
    });
    let spinner = CheckSpinner::start();
    let status = match resolved {
        ResolvedNamedReadinessProbe::Http {
            address,
            port,
            request,
            timeout,
            ..
        } => http_readiness_endpoint_status(
            address.as_str(),
            port,
            &request,
            timeout_override_ms.map(Duration::from_millis).or(timeout),
        ),
        ResolvedNamedReadinessProbe::Tcp {
            address,
            port,
            timeout,
            ..
        } => tcp_readiness_endpoint_status(
            address.as_str(),
            port,
            timeout_override_ms.map(Duration::from_millis).or(timeout),
        ),
    };
    let status = match status {
        HttpReadinessStatus::Passed => CheckStatus::Passed,
        HttpReadinessStatus::Failed => CheckStatus::Failed,
        HttpReadinessStatus::TimedOut => CheckStatus::TimedOut(
            timeout_ms.expect("validated readiness probes always declare a timeout"),
        ),
    };
    spinner.stop();
    status
}

fn run_workflow_surface_readiness(
    contract: &Contract,
    contract_path: &Path,
    workflow_name: Option<&str>,
    surface_name: &str,
    overrides: ExecutionOverrides,
) -> Result<WorkflowSurfaceReadinessObservation, String> {
    let (_, workflow) = contract
        .selected_workflow(workflow_name)
        .ok_or_else(|| String::from("selected workflow is not declared"))?;
    let run_task_name = workflow
        .run
        .as_ref()
        .map(|run| run.task.as_str())
        .ok_or_else(|| String::from("selected workflow does not declare `run.task`"))?;
    let task = contract
        .tasks
        .get(run_task_name)
        .ok_or_else(|| format!("workflow run task `{run_task_name}` is not declared"))?;
    let backend = match crate::runner::resolve_execution_backend_with_contract_path(
        contract,
        run_task_name,
        overrides,
        Some(contract_path),
    )
    .map_err(|error| error.to_string())?
    {
        ResolvedExecutionBackend::Native { .. } => Backend::Native,
        ResolvedExecutionBackend::Container { .. } => Backend::Container,
        ResolvedExecutionBackend::Remote { .. }
        | ResolvedExecutionBackend::BackendProvider { .. } => Backend::Remote,
    };
    let probe =
        task_surface_host_readiness_probe_for_backend(contract, task, backend, surface_name)?;
    let timing = workflow_surface_readiness_timing_policy(contract, surface_name);
    let resolved_timeout = probe.default_timeout.unwrap_or(Duration::from_millis(200));
    let effective_probe_timeout = resolved_timeout.min(Duration::from_millis(
        DOCTOR_WORKFLOW_SURFACE_MAX_PROBE_TIMEOUT_MS,
    ));
    let effective_probe_timeout_ms = effective_probe_timeout.as_millis() as u64;
    let failed_retry_budget = capped_failed_retry_budget(timing.failed_retries, timing.interval);
    let timed_out_retry_budget =
        capped_timed_out_retry_budget(timing.timed_out_retries, effective_probe_timeout_ms);
    let spinner = CheckSpinner::start();
    let mut status = CheckStatus::Failed;
    let mut failed_attempts = 0u32;
    let mut timed_out_attempts = 0u32;
    let mut attempts_performed = 0u32;
    let max_attempts = failed_retry_budget.max(timed_out_retry_budget);
    if !timing.start_period.is_zero() {
        thread::sleep(timing.start_period);
    }
    let observation_started = Instant::now();
    for _ in 0..max_attempts {
        attempts_performed += 1;
        let observed = match probe.request.as_ref() {
            Some(request) => http_readiness_endpoint_status(
                probe.address.as_str(),
                probe.port,
                request,
                Some(effective_probe_timeout),
            ),
            None => tcp_readiness_endpoint_status(
                probe.address.as_str(),
                probe.port,
                Some(effective_probe_timeout),
            ),
        };
        status = match observed {
            HttpReadinessStatus::Passed => CheckStatus::Passed,
            HttpReadinessStatus::Failed => CheckStatus::Failed,
            HttpReadinessStatus::TimedOut => CheckStatus::TimedOut(effective_probe_timeout_ms),
        };
        if status == CheckStatus::Passed {
            break;
        }
        let should_continue = match status {
            CheckStatus::Failed => {
                failed_attempts += 1;
                failed_attempts < failed_retry_budget
            }
            CheckStatus::TimedOut(_) => {
                timed_out_attempts += 1;
                timed_out_attempts < timed_out_retry_budget
            }
            CheckStatus::Passed => false,
        };
        let elapsed_ms = observation_started.elapsed().as_millis() as u64;
        let within_failed_window = elapsed_ms < DOCTOR_WORKFLOW_SURFACE_FAILED_RETRY_WINDOW_MS;
        let within_timed_out_window = elapsed_ms < DOCTOR_WORKFLOW_SURFACE_TIMEOUT_RETRY_WINDOW_MS;
        let within_window = match status {
            CheckStatus::Failed => within_failed_window,
            CheckStatus::TimedOut(_) => within_timed_out_window,
            CheckStatus::Passed => false,
        };
        if !should_continue || !within_window {
            break;
        }
        thread::sleep(timing.interval);
    }
    spinner.stop();
    Ok(WorkflowSurfaceReadinessObservation {
        status,
        attempts: attempts_performed,
        run_task_name: run_task_name.to_string(),
        backend_label: match backend {
            Backend::Native => "native",
            Backend::Container => "container",
            Backend::Remote => "remote",
        }
        .to_string(),
        address: probe.address.clone(),
        port: probe.port,
        timeout_ms: effective_probe_timeout_ms,
    })
}

fn capped_timed_out_retry_budget(configured_retries: u32, timeout_ms: u64) -> u32 {
    let timeout_ms = timeout_ms.max(1);
    let cap = DOCTOR_WORKFLOW_SURFACE_TIMEOUT_RETRY_WINDOW_MS
        .div_ceil(timeout_ms)
        .max(1);
    configured_retries.min(cap as u32).max(1)
}

fn capped_failed_retry_budget(configured_retries: u32, interval: Duration) -> u32 {
    let interval_ms = (interval.as_millis() as u64).max(1);
    let cap = DOCTOR_WORKFLOW_SURFACE_FAILED_RETRY_WINDOW_MS
        .div_ceil(interval_ms)
        .max(1);
    configured_retries.min(cap as u32).max(1)
}

#[derive(Debug, Clone, Copy)]
struct WorkflowSurfaceReadinessTimingPolicy {
    start_period: Duration,
    interval: Duration,
    failed_retries: u32,
    timed_out_retries: u32,
}

fn workflow_surface_readiness_timing_policy(
    contract: &Contract,
    surface_name: &str,
) -> WorkflowSurfaceReadinessTimingPolicy {
    let readiness = contract
        .surface(surface_name)
        .and_then(|surface| surface.readiness.as_ref());
    let retries = readiness.and_then(|readiness| readiness.retries);
    WorkflowSurfaceReadinessTimingPolicy {
        start_period: readiness
            .and_then(|readiness| readiness.start_period.as_deref())
            .and_then(crate::schema::parse_readiness_duration_spec)
            .map(cap_doctor_readiness_start_period)
            .unwrap_or(Duration::ZERO),
        interval: readiness
            .and_then(|readiness| readiness.interval.as_deref())
            .and_then(crate::schema::parse_readiness_duration_spec)
            .unwrap_or(Duration::from_millis(
                DOCTOR_WORKFLOW_SURFACE_READINESS_INTERVAL_MS,
            )),
        failed_retries: retries.unwrap_or(DOCTOR_WORKFLOW_SURFACE_READINESS_FAILED_RETRIES),
        timed_out_retries: retries.unwrap_or(DOCTOR_WORKFLOW_SURFACE_READINESS_TIMEOUT_RETRIES),
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct WorkflowSurfaceReadinessObservation {
    status: CheckStatus,
    attempts: u32,
    run_task_name: String,
    backend_label: String,
    address: String,
    port: u16,
    timeout_ms: u64,
}

fn probe_uses_task_observer(probe: &crate::schema::ReadinessProbeSpec) -> bool {
    probe
        .target
        .as_ref()
        .filter(|target| target.kind == crate::schema::ReadinessProbeTargetKind::Task)
        .and_then(|target| target.observer.as_ref())
        .is_some_and(|observer| observer.kind == crate::schema::ReadinessProbeObserverKind::Task)
}

fn run_observer_task_named_probe(
    contract: &Contract,
    contract_path: &Path,
    probe_name: &str,
    probe: &crate::schema::ReadinessProbeSpec,
    timeout_override_ms: Option<u64>,
    overrides: ExecutionOverrides,
) -> CheckStatus {
    let Some(target) = probe.target.as_ref() else {
        return CheckStatus::Failed;
    };
    let Some(observer) = target.observer.as_ref() else {
        return CheckStatus::Failed;
    };
    let Some(observer_task_name) = observer
        .task
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    else {
        return CheckStatus::Failed;
    };

    let observer_backend = match crate::runner::resolve_execution_backend_with_contract_path(
        contract,
        observer_task_name,
        overrides,
        Some(contract_path),
    ) {
        Ok(backend) => backend,
        Err(_) => return CheckStatus::Failed,
    };
    let caller_backend =
        crate::runner::effective_task_execution(contract, observer_task_name, overrides).backend;
    let resolved = match resolve_observer_task_probe_command(
        contract,
        contract_path,
        probe_name,
        probe,
        observer_task_name,
        caller_backend,
        timeout_override_ms,
        overrides,
    ) {
        Ok(resolved) => resolved,
        Err(_) => return CheckStatus::Failed,
    };
    let spinner = CheckSpinner::start();
    let output = run_backend_command_captured(
        &format!("probe:{probe_name}"),
        resolved.command.as_str(),
        contract_working_dir(contract_path),
        &observer_backend,
    );
    spinner.stop();
    match output {
        Ok(result) if result.exit_code == 0 => CheckStatus::Passed,
        Ok(result) if result.exit_code == 124 => resolved
            .timeout_ms
            .map(CheckStatus::TimedOut)
            .unwrap_or(CheckStatus::Failed),
        Ok(_) => CheckStatus::Failed,
        Err(_) => CheckStatus::Failed,
    }
}

struct ObserverTaskProbeCommand {
    command: String,
    timeout_ms: Option<u64>,
}

fn resolve_observer_task_probe_command(
    contract: &Contract,
    contract_path: &Path,
    probe_name: &str,
    probe: &crate::schema::ReadinessProbeSpec,
    observer_task_name: &str,
    caller_backend: Backend,
    timeout_override_ms: Option<u64>,
    overrides: ExecutionOverrides,
) -> Result<ObserverTaskProbeCommand, String> {
    let target = probe
        .target
        .as_ref()
        .ok_or_else(|| format!("readiness probe `{probe_name}` does not declare a target"))?;
    let listener_name = target
        .listener
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .ok_or_else(|| {
            format!("readiness probe `{probe_name}` task target must declare `target.listener`")
        })?;
    let synthetic_target = crate::schema::TaskTargetSpec {
        service: Some(crate::schema::TaskTargetServiceRefSpec {
            member: None,
            repo: None,
            task: target.name.clone(),
            listener: Some(listener_name.to_string()),
            address_view: target.address_view,
        }),
        url: None,
        override_input: None,
        activation: crate::schema::TaskTargetActivationSpec::default(),
    };
    let resolved_target = resolve_task_target_binding_url_with_contract_path(
        contract,
        contract_path,
        observer_task_name,
        probe_name,
        &synthetic_target,
        caller_backend,
        overrides,
    )
    .map_err(|error| error.to_string())?;
    let timeout_duration = timeout_override_ms
        .map(Duration::from_millis)
        .or(probe.timeout.map(Duration::from_millis));
    let timeout_ms = timeout_override_ms.or(probe.timeout);

    let command = match probe.kind {
        crate::schema::ReadinessProbeKind::Http => {
            let (address, port, base_path) = parse_http_probe_url(resolved_target.as_str())
                .ok_or_else(|| {
                    format!(
                        "readiness probe `{probe_name}` could not resolve one HTTP target URL from observer task `{observer_task_name}`"
                    )
                })?;
            let path =
                combine_readiness_probe_paths(Some(base_path.as_str()), probe.path.as_deref());
            let endpoint = crate::schema::ServiceEndpointSpec {
                context: None,
                address,
                port,
            };
            let request = HttpReadinessRequest {
                method: probe
                    .method
                    .unwrap_or(crate::schema::TaskRuntimeReadinessHttpMethod::Get),
                path,
                headers: probe.headers.clone(),
                success_statuses: probe
                    .success
                    .as_ref()
                    .map(|success| success.status.clone())
                    .or_else(|| probe.expect_status.map(|status| vec![status]))
                    .unwrap_or_else(|| vec![200]),
                body_contains: probe.body.as_ref().map(|body| body.contains.clone()),
            };
            service_http_readiness_probe_command_from_request(&endpoint, &request, timeout_duration)
        }
        crate::schema::ReadinessProbeKind::Tcp => {
            let (address, port) = parse_observer_probe_tcp_target(
                probe_name,
                observer_task_name,
                resolved_target.as_str(),
            )?;
            let endpoint = crate::schema::ServiceEndpointSpec {
                context: None,
                address,
                port,
            };
            service_tcp_readiness_probe_command_from_timeout(&endpoint, timeout_duration)
        }
    };

    Ok(ObserverTaskProbeCommand {
        command,
        timeout_ms,
    })
}

fn parse_observer_probe_tcp_target(
    probe_name: &str,
    observer_task_name: &str,
    resolved_target: &str,
) -> Result<(String, u16), String> {
    let (address, port_text) = resolved_target.rsplit_once(':').ok_or_else(|| {
        format!(
            "readiness probe `{probe_name}` could not resolve one TCP target from observer task `{observer_task_name}`"
        )
    })?;
    let port = port_text.parse::<u16>().map_err(|_| {
        format!(
            "readiness probe `{probe_name}` resolved one invalid TCP target from observer task `{observer_task_name}`"
        )
    })?;
    Ok((address.trim().to_string(), port))
}

fn tcp_readiness_endpoint_status(
    address: &str,
    port: u16,
    timeout: Option<Duration>,
) -> HttpReadinessStatus {
    let connect_timeout = timeout.unwrap_or(Duration::from_millis(200));
    let addrs = crate::runner::probe_socket_candidates(address, port);
    if addrs.is_empty() {
        return HttpReadinessStatus::Failed;
    }
    let mut saw_non_timeout_error = false;
    let mut timed_out = false;
    for socket in addrs {
        match TcpStream::connect_timeout(&socket, connect_timeout) {
            Ok(stream) => {
                let _ = stream.shutdown(std::net::Shutdown::Both);
                return HttpReadinessStatus::Passed;
            }
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => timed_out = true,
            Err(_) => saw_non_timeout_error = true,
        }
    }
    if timed_out && !saw_non_timeout_error {
        HttpReadinessStatus::TimedOut
    } else if saw_non_timeout_error {
        HttpReadinessStatus::Failed
    } else {
        HttpReadinessStatus::Failed
    }
}

fn failed_check_summary(check: &crate::schema::CheckSpec) -> String {
    if check.kind == crate::schema::CheckKind::File {
        format!("File check failed: {}", check.name)
    } else if check.kind == crate::schema::CheckKind::Env {
        format!("Env check failed: {}", check.name)
    } else if check.kind == crate::schema::CheckKind::ChangedFiles {
        format!("Changed-files check not satisfied: {}", check.name)
    } else if check.probe.is_some() {
        format!("Probe check failed: {}", check.name)
    } else {
        format!("Check failed: {}", check.name)
    }
}

fn check_status_finding(
    contract: &Contract,
    workflow_name: Option<&str>,
    check: &crate::schema::CheckSpec,
    severity: FindingSeverity,
    timed_out: Option<u64>,
) -> Finding {
    let (code, summary, why, next) = if let Some(timeout) = timed_out {
        (
            if check.kind == crate::schema::CheckKind::File {
                "OTA_FILE_CHECK_TIMED_OUT"
            } else {
                "OTA_CHECK_TIMED_OUT"
            },
            timed_out_check_summary(check),
            timed_out_check_why(contract, check, timeout),
            timed_out_check_next(contract, check),
        )
    } else {
        (
            if check.kind == crate::schema::CheckKind::File {
                "OTA_FILE_CHECK_FAILED"
            } else {
                "OTA_CHECK_FAILED"
            },
            failed_check_summary(check),
            failed_check_why(contract, check),
            failed_check_next(contract, workflow_name, check),
        )
    };

    Finding::identified(
        code,
        "execution",
        "repo_contract",
        severity,
        summary,
        why,
        next,
    )
}

fn failed_check_why(contract: &Contract, check: &crate::schema::CheckSpec) -> String {
    if check.kind == crate::schema::CheckKind::File {
        let path = check.path.as_deref().unwrap_or("-");
        let expected = file_check_expectation_label(check.expect);
        format!("expected `{path}` to be {expected}, but the file check did not pass")
    } else if check.kind == crate::schema::CheckKind::Env {
        if let Some(env) = check.env.as_ref() {
            let keys = env
                .assertions
                .iter()
                .map(|assertion| assertion.key.as_str())
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                "expected dotenv file `{}` to satisfy env assertions for [{keys}], but the env check did not pass",
                env.path
            )
        } else {
            format!("the configured `{}` env check did not succeed", check.name)
        }
    } else if check.kind == crate::schema::CheckKind::ChangedFiles {
        if let Some(changed_files) = check.changed_files.as_ref() {
            let matchers = changed_files.paths.join(", ");
            let compare = changed_files
                .base_ref
                .as_deref()
                .map(str::trim)
                .filter(|value| !value.is_empty())
                .map(|base| {
                    let head = changed_files
                        .head_ref
                        .as_deref()
                        .map(str::trim)
                        .filter(|value| !value.is_empty())
                        .unwrap_or("HEAD");
                    format!("{base}..{head}")
                })
                .unwrap_or_else(|| String::from("working tree vs HEAD"));
            format!("no changed files matched [{matchers}] for `{compare}`")
        } else {
            format!(
                "the configured `{}` changed-files check did not succeed",
                check.name
            )
        }
    } else if let Some(probe_name) = check.probe.as_deref() {
        format!(
            "the configured `{}` probe-backed check ({}) did not succeed",
            probe_name,
            probe_source_description(contract, probe_name)
        )
    } else {
        format!("the configured `{}` check did not succeed", check.name)
    }
}

fn timed_out_check_summary(check: &crate::schema::CheckSpec) -> String {
    if check.kind == crate::schema::CheckKind::File {
        format!("File check timed out: {}", check.name)
    } else if check.kind == crate::schema::CheckKind::Env {
        format!("Env check timed out: {}", check.name)
    } else if check.kind == crate::schema::CheckKind::ChangedFiles {
        format!("Changed-files check timed out: {}", check.name)
    } else if check.probe.is_some() {
        format!("Probe check timed out: {}", check.name)
    } else {
        format!("Check timed out: {}", check.name)
    }
}

fn timed_out_check_why(
    contract: &Contract,
    check: &crate::schema::CheckSpec,
    timeout: u64,
) -> String {
    if let Some(probe_name) = check.probe.as_deref() {
        format!(
            "the configured `{}` probe-backed check ({}) did not finish within {}ms",
            probe_name,
            probe_source_description(contract, probe_name),
            timeout
        )
    } else {
        format!(
            "the configured `{}` check did not finish within {}ms",
            check.name, timeout
        )
    }
}

fn probe_source_description(contract: &Contract, probe_name: &str) -> String {
    if let Some(probe) = contract.probe(probe_name)
        && let Some(target) = probe.target.as_ref()
        && target.kind == crate::schema::ReadinessProbeTargetKind::Task
        && let Some(observer) = target.observer.as_ref()
        && observer.kind == crate::schema::ReadinessProbeObserverKind::Task
        && let Some(observer_task) = observer.task.as_deref()
    {
        return format!(
            "task listener `{}.{}` (`address_view: {}`, observer: {})",
            target.name,
            target.listener.as_deref().unwrap_or("-"),
            match target.address_view {
                crate::schema::TaskTargetAddressView::Topology => "topology",
                crate::schema::TaskTargetAddressView::Host => "host",
                crate::schema::TaskTargetAddressView::Internal => "internal",
            },
            observer_task.trim()
        );
    }
    match resolve_named_readiness_probe(contract, probe_name) {
        Ok(ResolvedNamedReadinessProbe::Http { source, .. })
        | Ok(ResolvedNamedReadinessProbe::Tcp { source, .. }) => source.description(),
        Err(_) => format!("probe `{probe_name}`"),
    }
}

fn timed_out_check_next(contract: &Contract, check: &crate::schema::CheckSpec) -> String {
    if let Some(probe_name) = check.probe.as_deref()
        && let Some(probe) = contract.probe(probe_name)
    {
        return timed_out_probe_next(probe_name, probe);
    }
    if check.kind == crate::schema::CheckKind::ChangedFiles {
        return format!(
            "reduce changed-files matcher scope for `{}` or rerun when git metadata is available, then rerun `ota doctor`",
            check.name
        );
    }
    let command = check.run.as_deref().unwrap_or("<unknown>");
    format!(
        "make `{}` complete faster or raise `checks.timeout` for `{}`, then rerun `ota doctor`",
        command, check.name
    )
}

fn file_check_expectation_label(
    expect: Option<crate::schema::FileCheckExpectation>,
) -> &'static str {
    match expect.unwrap_or(crate::schema::FileCheckExpectation::Exists) {
        crate::schema::FileCheckExpectation::Exists => "present",
        crate::schema::FileCheckExpectation::File => "a file",
        crate::schema::FileCheckExpectation::Directory => "a directory",
        crate::schema::FileCheckExpectation::Missing => "missing",
    }
}

fn failed_probe_next(probe_name: &str, probe: &ReadinessProbeSpec) -> String {
    format!(
        "inspect probe `{probe_name}` ({}) and fix the reported issue, then rerun `ota doctor`",
        readiness_probe_summary(probe)
    )
}

fn timed_out_probe_next(probe_name: &str, probe: &ReadinessProbeSpec) -> String {
    format!(
        "make probe `{probe_name}` ({}) respond within the configured timeout, or raise that timeout, then rerun `ota doctor`",
        readiness_probe_summary(probe)
    )
}

fn readiness_probe_summary(probe: &ReadinessProbeSpec) -> String {
    if let Some(url) = probe.url.as_deref() {
        return format!("literal URL `{url}`");
    }
    let Some(target) = probe.target.as_ref() else {
        return String::from("undeclared target");
    };
    match target.kind {
        crate::schema::ReadinessProbeTargetKind::Task => format!(
            "task listener `{}.{}` (`address_view: {}`)",
            target.name,
            target.listener.as_deref().unwrap_or("<missing>"),
            match target.address_view {
                crate::schema::TaskTargetAddressView::Topology => "topology",
                crate::schema::TaskTargetAddressView::Host => "host",
                crate::schema::TaskTargetAddressView::Internal => "internal",
            }
        ),
        crate::schema::ReadinessProbeTargetKind::Service => format!(
            "service endpoint `{}.{}`",
            target.name,
            target.endpoint.as_deref().unwrap_or("<auto>")
        ),
    }
}

fn run_check(command: &str, working_dir: &Path, timeout_ms: Option<u64>) -> CheckStatus {
    match run_check_with_env(command, working_dir, timeout_ms, None) {
        DetailedCheckStatus::Passed => CheckStatus::Passed,
        DetailedCheckStatus::Failed(_) => CheckStatus::Failed,
        DetailedCheckStatus::TimedOut(timeout) => CheckStatus::TimedOut(timeout),
    }
}

fn run_check_with_env(
    command: &str,
    working_dir: &Path,
    timeout_ms: Option<u64>,
    env_overrides: Option<&BTreeMap<String, String>>,
) -> DetailedCheckStatus {
    let Ok(mut child) = shell_command(command)
        .current_dir(working_dir)
        .envs(env_overrides.into_iter().flat_map(|env| env.iter()))
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    else {
        return DetailedCheckStatus::Failed(CheckFailureDetails {
            exit_code: None,
            stdout: String::new(),
            stderr: String::from("failed to spawn declared check command"),
        });
    };

    let spinner = CheckSpinner::start();
    let status = match timeout_ms {
        Some(timeout_ms) => wait_for_child_with_timeout(&mut child, timeout_ms),
        None => wait_for_child(&mut child),
    };
    spinner.stop();

    status
}

fn wait_for_child(child: &mut std::process::Child) -> DetailedCheckStatus {
    match child.wait() {
        Ok(status) => {
            let (stdout, stderr) = collect_child_output(child);
            if status.success() {
                DetailedCheckStatus::Passed
            } else {
                DetailedCheckStatus::Failed(CheckFailureDetails {
                    exit_code: status.code(),
                    stdout,
                    stderr,
                })
            }
        }
        Err(error) => DetailedCheckStatus::Failed(CheckFailureDetails {
            exit_code: None,
            stdout: String::new(),
            stderr: error.to_string(),
        }),
    }
}

fn wait_for_child_with_timeout(
    child: &mut std::process::Child,
    timeout_ms: u64,
) -> DetailedCheckStatus {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                let (stdout, stderr) = collect_child_output(child);
                return if status.success() {
                    DetailedCheckStatus::Passed
                } else {
                    DetailedCheckStatus::Failed(CheckFailureDetails {
                        exit_code: status.code(),
                        stdout,
                        stderr,
                    })
                };
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return DetailedCheckStatus::TimedOut(timeout_ms);
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return DetailedCheckStatus::Failed(CheckFailureDetails {
                    exit_code: None,
                    stdout: String::new(),
                    stderr: String::from("failed while waiting for declared check command"),
                });
            }
        }
    }
}

fn collect_child_output(child: &mut std::process::Child) -> (String, String) {
    let mut stdout = String::new();
    if let Some(mut stream) = child.stdout.take() {
        let _ = stream.read_to_string(&mut stdout);
    }

    let mut stderr = String::new();
    if let Some(mut stream) = child.stderr.take() {
        let _ = stream.read_to_string(&mut stderr);
    }

    (stdout.trim().to_string(), stderr.trim().to_string())
}

fn check_failure_details_summary(details: &CheckFailureDetails) -> Option<String> {
    let mut parts = Vec::new();
    if let Some(exit_code) = details.exit_code {
        parts.push(format!("exit code {exit_code}"));
        if let Some(guidance) = windows_exit_code_guidance_line(exit_code) {
            parts.push(guidance);
        }
    }
    if !details.stdout.is_empty() {
        parts.push(format!("stdout: {}", details.stdout));
    }
    if !details.stderr.is_empty() {
        parts.push(format!("stderr: {}", details.stderr));
    }
    (!parts.is_empty()).then(|| parts.join("; "))
}

fn windows_exit_code_guidance(exit_code: i32) -> Option<&'static str> {
    match exit_code {
        -1073741819 => Some("Windows access violation crash"),
        -1073740791 => Some("Windows fast-fail / stack buffer overrun crash"),
        -1073741510 => Some("Windows interrupt/termination (Ctrl+C or console close)"),
        -1073741502 => Some("Windows DLL initialization failure"),
        _ => None,
    }
}

fn windows_exit_code_guidance_line(exit_code: i32) -> Option<String> {
    windows_exit_code_guidance(exit_code).map(|guidance| {
        format!(
            "on Windows this usually maps to `{guidance}` (`0x{:08X}`)",
            exit_code as u32
        )
    })
}

struct CheckSpinner {
    stop: Arc<AtomicBool>,
    handle: Option<thread::JoinHandle<()>>,
}

impl CheckSpinner {
    fn start() -> Self {
        if !should_show_spinner() {
            return Self {
                stop: Arc::new(AtomicBool::new(true)),
                handle: None,
            };
        }

        let stop = Arc::new(AtomicBool::new(false));
        let thread_stop = Arc::clone(&stop);
        let handle = thread::spawn(move || {
            let frames = ["⠋", "⠙", "⠹", "⠸", "⠼", "⠴", "⠦", "⠧", "⠇", "⠏"];
            let mut index = 0usize;
            let mut stderr = io::stderr();
            while !thread_stop.load(Ordering::Relaxed) {
                let frame = frames[index % frames.len()];
                let _ = write!(stderr, "\r🦦 {frame}");
                let _ = stderr.flush();
                index += 1;
                thread::sleep(Duration::from_millis(160));
            }
        });

        Self {
            stop,
            handle: Some(handle),
        }
    }

    fn stop(mut self) {
        self.stop.store(true, Ordering::Relaxed);
        if let Some(handle) = self.handle.take() {
            let _ = handle.join();
        }
        if io::stderr().is_terminal() {
            let mut stderr = io::stderr();
            let _ = write!(stderr, "\r\x1b[2K\r");
            let _ = stderr.flush();
        }
    }
}

fn should_show_spinner() -> bool {
    supports_dynamic_stderr_ui(io::stderr().is_terminal())
        && std::env::var_os("OTA_PLAIN_MODE").is_none()
        && std::env::var_os("OTA_JSON_MODE").is_none()
}

fn map_check_severity(severity: CheckSeverity) -> FindingSeverity {
    match severity {
        CheckSeverity::Error => FindingSeverity::Error,
        CheckSeverity::Warn => FindingSeverity::Warn,
        CheckSeverity::Info => FindingSeverity::Info,
    }
}

pub(crate) fn command_version(name: &str) -> Option<String> {
    command_version_probe(name).version()
}

pub(crate) fn command_available(name: &str) -> bool {
    resolve_command_path(name).is_some()
}

fn command_version_probe(name: &str) -> CommandVersionProbe {
    command_version_probe_in_working_dir(name, Path::new("."))
}

fn command_version_probe_in_working_dir(name: &str, working_dir: &Path) -> CommandVersionProbe {
    let command = version_command_string(name);
    let resolved_path = resolve_command_path(name);
    if resolved_path.is_none() && !looks_like_command_path(name) {
        return CommandVersionProbe {
            command,
            resolved_path: None,
            probe_started: false,
            outcome: CommandVersionProbeOutcome::Missing,
        };
    }

    let resolved_path = resolved_path
        .as_deref()
        .filter(|path| !should_probe_via_command_name(name, path))
        .map(Path::to_path_buf);
    let program = version_probe_program(name, resolved_path.as_deref());
    let mut parseable_attempt_observed = false;
    let mut last_probe_failed = None;
    let mut outcome = CommandVersionProbeOutcome::Missing;
    for args in crate::runner::tool_version_probe_arg_sets(name) {
        let attempt = version_command(program, working_dir, args).output();
        outcome = match attempt {
            Ok(output) if output.status.success() => {
                let combined = format!(
                    "{} {}",
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                );
                if let Some(version) = extract_version_token(&combined) {
                    CommandVersionProbeOutcome::Version(version)
                } else {
                    parseable_attempt_observed = true;
                    continue;
                }
            }
            Ok(output) => {
                last_probe_failed = Some(CommandVersionProbeOutcome::ProbeFailed {
                    exit_code: output.status.code(),
                    error: None,
                });
                continue;
            }
            Err(error) => {
                last_probe_failed = Some(CommandVersionProbeOutcome::ProbeFailed {
                    exit_code: None,
                    error: Some(error.to_string()),
                });
                continue;
            }
        };
        break;
    }
    let outcome = match outcome {
        CommandVersionProbeOutcome::Version(_) => outcome,
        _ if parseable_attempt_observed => CommandVersionProbeOutcome::Unparseable,
        _ => last_probe_failed.unwrap_or(CommandVersionProbeOutcome::Missing),
    };

    CommandVersionProbe {
        command,
        resolved_path,
        probe_started: true,
        outcome,
    }
}

fn command_version_probe_candidates<F>(
    candidates: &[String],
    requirement: &str,
    mut probe: F,
) -> CommandVersionProbe
where
    F: FnMut(&str) -> CommandVersionProbe,
{
    let mut first_missing = None;
    let mut first_probe_issue = None;
    let mut first_version = None;

    for candidate in candidates {
        let result = probe(candidate.as_str());
        match &result.outcome {
            CommandVersionProbeOutcome::Version(actual) if version_matches(requirement, actual) => {
                return result;
            }
            CommandVersionProbeOutcome::Version(_) => {
                if first_version.is_none() {
                    first_version = Some(result);
                }
            }
            CommandVersionProbeOutcome::ProbeFailed { .. }
            | CommandVersionProbeOutcome::Unparseable => {
                if first_probe_issue.is_none() {
                    first_probe_issue = Some(result);
                }
            }
            CommandVersionProbeOutcome::Missing => {
                if first_missing.is_none() {
                    first_missing = Some(result);
                }
            }
        }
    }

    first_version
        .or(first_probe_issue)
        .or(first_missing)
        .unwrap_or_else(|| CommandVersionProbe {
            command: version_command_string(
                candidates.first().map(String::as_str).unwrap_or_default(),
            ),
            resolved_path: None,
            probe_started: false,
            outcome: CommandVersionProbeOutcome::Missing,
        })
}

fn runtime_executable_candidates(name: &str, requirement: &str) -> Vec<String> {
    match name {
        "python" => python_runtime_executable_candidates(requirement),
        "rust" => vec![String::from("rustc")],
        _ => vec![name.to_string()],
    }
}

fn python_runtime_executable_candidates(requirement: &str) -> Vec<String> {
    let mut candidates = Vec::new();

    if let Some(req) = parse_semver_requirement(requirement) {
        extend_python_minor_range_candidates(&mut candidates, &req);
        for comparator in &req.comparators {
            push_python_version_candidate(&mut candidates, comparator.major, comparator.minor);
        }
    }

    candidates.push(String::from("python3"));
    candidates.push(String::from("python"));
    dedupe_preserve_order(candidates)
}

fn extend_python_minor_range_candidates(candidates: &mut Vec<String>, requirement: &VersionReq) {
    let lower = requirement
        .comparators
        .iter()
        .filter(|comparator| matches!(comparator.op, Op::Greater | Op::GreaterEq | Op::Exact))
        .max_by_key(|comparator| {
            (
                comparator.major,
                comparator.minor.unwrap_or(0),
                comparator.patch.unwrap_or(0),
            )
        });
    let upper = requirement
        .comparators
        .iter()
        .filter(|comparator| matches!(comparator.op, Op::Less | Op::LessEq))
        .min_by_key(|comparator| {
            (
                comparator.major,
                comparator.minor.unwrap_or(u64::MAX),
                comparator.patch.unwrap_or(u64::MAX),
            )
        });

    let (Some(lower), Some(upper)) = (lower, upper) else {
        return;
    };
    let (Some(lower_minor), Some(upper_minor)) = (lower.minor, upper.minor) else {
        return;
    };
    if lower.major != upper.major || upper_minor <= lower_minor {
        return;
    }

    let upper_exclusive = match upper.op {
        Op::LessEq => upper_minor.saturating_add(1),
        _ => upper_minor,
    };
    if upper_exclusive <= lower_minor || upper_exclusive - lower_minor > 8 {
        return;
    }

    for minor in (lower_minor..upper_exclusive).rev() {
        push_python_version_candidate(candidates, lower.major, Some(minor));
    }
}

fn push_python_version_candidate(candidates: &mut Vec<String>, major: u64, minor: Option<u64>) {
    let Some(minor) = minor else {
        return;
    };
    candidates.push(format!("python{major}.{minor}"));
}

fn parse_semver_requirement(value: &str) -> Option<VersionReq> {
    let trimmed = value.trim();
    VersionReq::parse(trimmed).ok().or_else(|| {
        normalize_short_version_requirement(trimmed)
            .and_then(|normalized| VersionReq::parse(&normalized).ok())
    })
}

fn normalize_short_version_requirement(value: &str) -> Option<String> {
    if value.is_empty() || value == "*" || value.contains("||") {
        return None;
    }
    if value
        .chars()
        .all(|character| character.is_ascii_digit() || character == '.')
    {
        let segments = value
            .split('.')
            .filter(|segment| !segment.is_empty())
            .collect::<Vec<_>>();
        return match segments.as_slice() {
            [major] => {
                let major = major.parse::<u64>().ok()?;
                Some(format!(">={major}.0.0,<{}.0.0", major.saturating_add(1)))
            }
            [major, minor] => {
                let major = major.parse::<u64>().ok()?;
                let minor = minor.parse::<u64>().ok()?;
                Some(format!(
                    ">={major}.{minor}.0,<{}.{minor_next}.0",
                    major,
                    minor_next = minor.saturating_add(1)
                ))
            }
            _ => None,
        };
    }

    let normalized = value.split_whitespace().collect::<Vec<_>>().join(", ");
    (normalized != value).then_some(normalized)
}

fn dedupe_preserve_order(values: Vec<String>) -> Vec<String> {
    let mut seen = BTreeSet::new();
    let mut deduped = Vec::new();
    for value in values {
        if seen.insert(value.clone()) {
            deduped.push(value);
        }
    }
    deduped
}

fn version_command(program: &OsStr, working_dir: &Path, args: &[&str]) -> Command {
    #[cfg(windows)]
    let mut command = {
        let is_wrapper = Path::new(program)
            .extension()
            .and_then(|value| value.to_str())
            .is_some_and(|value| {
                value.eq_ignore_ascii_case("cmd") || value.eq_ignore_ascii_case("bat")
            });
        if is_wrapper {
            let mut command = Command::new("cmd");
            command.arg("/C").arg(program);
            command
        } else {
            Command::new(program)
        }
    };
    #[cfg(not(windows))]
    let mut command = Command::new(program);
    command.current_dir(working_dir);
    for arg in args {
        command.arg(arg);
    }
    command
}

fn version_probe_program<'a>(name: &'a str, resolved_path: Option<&'a Path>) -> &'a OsStr {
    if looks_like_command_path(name) {
        return resolved_path.unwrap_or_else(|| Path::new(name)).as_os_str();
    }

    if let Some(path) = resolved_path
        && is_repo_owned_source_managed_probe_path(path)
    {
        return path.as_os_str();
    }

    #[cfg(windows)]
    if let Some(path) = resolved_path
        && is_cmd_wrapper_path(path)
    {
        return path.as_os_str();
    }

    OsStr::new(name)
}

fn is_repo_owned_source_managed_probe_path(path: &Path) -> bool {
    let Ok(current_dir) = std::env::current_dir() else {
        return false;
    };
    path.starts_with(current_dir.join(".ota/state/source-managed/bin"))
}

#[cfg(windows)]
fn is_cmd_wrapper_path(path: &Path) -> bool {
    path.extension()
        .and_then(|value| value.to_str())
        .is_some_and(|value| value.eq_ignore_ascii_case("cmd") || value.eq_ignore_ascii_case("bat"))
}

fn should_probe_via_command_name(name: &str, resolved_path: &Path) -> bool {
    if looks_like_command_path(name) {
        return false;
    }
    let Ok(target) = std::fs::read_link(resolved_path) else {
        return false;
    };
    provider_hint_from_probe_path(&target) == Some("mise")
}

pub(crate) fn resolve_command_path(name: &str) -> Option<PathBuf> {
    if looks_like_command_path(name) {
        return command_path_candidates(Path::new(name))
            .into_iter()
            .find(|candidate| is_probeable_file(candidate));
    }

    if let Some(candidate) = repo_owned_source_managed_command_path(name) {
        return Some(candidate);
    }

    #[cfg(windows)]
    if let Ok(current_dir) = std::env::current_dir()
        && let Some(candidate) = command_path_candidates(&current_dir.join(name))
            .into_iter()
            .find(|candidate| is_probeable_file(candidate))
    {
        return Some(candidate);
    }

    let path = std::env::var_os("PATH")?;
    for entry in std::env::split_paths(&path) {
        if let Some(candidate) = command_path_candidates(&entry.join(name))
            .into_iter()
            .find(|candidate| is_probeable_file(candidate))
        {
            return Some(candidate);
        }
    }
    None
}

fn repo_owned_source_managed_command_path(name: &str) -> Option<PathBuf> {
    let current_dir = std::env::current_dir().ok()?;
    command_path_candidates(&current_dir.join(".ota/state/source-managed/bin").join(name))
        .into_iter()
        .find(|candidate| is_probeable_file(candidate))
}

fn looks_like_command_path(name: &str) -> bool {
    Path::new(name).is_absolute() || name.contains('/') || name.contains('\\')
}

fn command_path_candidates(path: &Path) -> Vec<PathBuf> {
    #[cfg(windows)]
    {
        if path.extension().is_some() {
            return vec![path.to_path_buf()];
        }

        let mut candidates = Vec::new();
        let mut extensions = std::env::var_os("PATHEXT")
            .map(|value| {
                value
                    .to_string_lossy()
                    .split(';')
                    .map(str::trim)
                    .filter(|value| !value.is_empty())
                    .map(|value| value.to_uppercase())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_else(|| vec![".COM".to_string(), ".EXE".to_string(), ".BAT".to_string()]);
        let baseline_extensions = [".CMD", ".COM", ".EXE", ".BAT"];
        for extension in baseline_extensions {
            if !extensions
                .iter()
                .any(|value| value.eq_ignore_ascii_case(extension))
            {
                extensions.push(extension.to_string());
            }
        }

        for ext in extensions {
            let mut candidate = path.as_os_str().to_os_string();
            candidate.push(ext);
            candidates.push(PathBuf::from(candidate));
        }
        candidates
    }
    #[cfg(not(windows))]
    {
        vec![path.to_path_buf()]
    }
}

fn is_probeable_file(path: &Path) -> bool {
    let Ok(metadata) = std::fs::metadata(path) else {
        return false;
    };
    if !metadata.is_file() {
        return false;
    }

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        metadata.permissions().mode() & 0o111 != 0
    }
    #[cfg(not(unix))]
    {
        true
    }
}

fn kind_label(kind: &str) -> &'static str {
    match kind {
        "runtime" => "Runtime",
        _ => "Tool",
    }
}

fn contract_working_dir(contract_path: &Path) -> &Path {
    contract_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
}

pub(crate) const OTA_STATE_GITIGNORE_COMMENT: &str = "# Ota local runtime artifacts";
pub(crate) const OTA_STATE_GITIGNORE_ENTRY: &str = ".ota/state/*";
pub(crate) const OTA_RECEIPTS_GITIGNORE_ENTRY: &str = ".ota/receipts/*";
pub(crate) const OTA_PROOF_GITIGNORE_ENTRY: &str = ".ota/proof/*";

fn gitignore_has_ota_state_entry(contents: &str) -> bool {
    contents.lines().any(|line| {
        matches!(
            line.trim(),
            ".ota/"
                | ".ota"
                | ".ota/*"
                | "/.ota/"
                | "/.ota"
                | "/.ota/*"
                | ".ota/state/"
                | ".ota/state"
                | ".ota/state/*"
                | "/.ota/state/"
                | "/.ota/state"
                | "/.ota/state/*"
        )
    })
}

fn gitignore_has_ota_receipts_entry(contents: &str) -> bool {
    contents.lines().any(|line| {
        matches!(
            line.trim(),
            ".ota/"
                | ".ota"
                | ".ota/*"
                | "/.ota/"
                | "/.ota"
                | "/.ota/*"
                | ".ota/receipts/"
                | ".ota/receipts"
                | ".ota/receipts/*"
                | "/.ota/receipts/"
                | "/.ota/receipts"
                | "/.ota/receipts/*"
        )
    })
}

fn gitignore_has_ota_proof_entry(contents: &str) -> bool {
    contents.lines().any(|line| {
        matches!(
            line.trim(),
            ".ota/"
                | ".ota"
                | ".ota/*"
                | "/.ota/"
                | "/.ota"
                | "/.ota/*"
                | ".ota/proof/"
                | ".ota/proof"
                | ".ota/proof/*"
                | "/.ota/proof/"
                | "/.ota/proof"
                | "/.ota/proof/*"
        )
    })
}

pub(crate) fn repo_missing_ota_state_gitignore(root: &Path) -> Result<bool, String> {
    let git_dir = root.join(".git");
    let gitignore_path = root.join(".gitignore");
    if !git_dir.exists() && !gitignore_path.exists() {
        return Ok(false);
    }

    if !gitignore_path.exists() {
        return Ok(true);
    }

    let contents = fs::read_to_string(&gitignore_path)
        .map_err(|error| format!("failed to read `{}`: {}", gitignore_path.display(), error))?;
    Ok(!(gitignore_has_ota_state_entry(&contents)
        && gitignore_has_ota_receipts_entry(&contents)
        && gitignore_has_ota_proof_entry(&contents)))
}

pub(crate) fn detect_missing_ota_state_gitignore(contract_path: &Path) -> Option<Finding> {
    let root = contract_working_dir(contract_path);
    match repo_missing_ota_state_gitignore(root) {
        Ok(true) => Some(Finding::identified(
            "OTA_REPO_HYGIENE_OTA_STATE_GITIGNORE",
            "contract",
            "repo_contract",
            FindingSeverity::Warn,
            "Repo local Ota artifacts are not ignored by git",
            "`.ota/state/`, `.ota/receipts/`, and `.ota/proof/` store Ota-owned local runtime artifacts; if they are tracked by git, execution residue, archived receipts, and runtime proof artifacts can pollute repo diffs and diagnosis artifacts",
            "run `ota doctor --fix --dry-run` to preview adding `.ota/state/`, `.ota/receipts/`, and `.ota/proof/` to `.gitignore`, or add the ignore rules manually",
        )),
        Ok(false) => None,
        Err(error) => Some(Finding::identified(
            "OTA_REPO_HYGIENE_GITIGNORE_UNREADABLE",
            "contract",
            "repo_contract",
            FindingSeverity::Warn,
            "Repo `.gitignore` could not be inspected",
            format!(
                "ota could not inspect whether `.ota/state/`, `.ota/receipts/`, and `.ota/proof/` are ignored: {error}"
            ),
            "repair `.gitignore` readability and rerun `ota doctor`",
        )),
    }
}

fn detect_devcontainer_runtime_drift(
    contract: &Contract,
    contract_path: &Path,
    requirement_surface: &RequirementSurface,
) -> Option<Finding> {
    let current_os = current_host_platform();
    let required = requirement_surface
        .runtimes
        .get("node")
        .filter(|requirement| requirement.required_for_os(current_os))
        .map(|requirement| requirement.version_for_os(current_os).to_string())
        .or_else(|| {
            contract
                .runtimes
                .get("node")
                .filter(|requirement| requirement.required_for_os(current_os))
                .map(|requirement| requirement.version_for_os(current_os).to_string())
        })
        .or_else(|| {
            contract
                .toolchains
                .get("node")
                .filter(|toolchain| toolchain.required_for_os(current_os))
                .map(|toolchain| toolchain.version_for_os(current_os).to_string())
        })?;

    let root = contract_working_dir(contract_path);
    let devcontainer_path = root.join(".devcontainer").join("devcontainer.json");
    if !devcontainer_path.exists() {
        return None;
    }

    let contents = fs::read_to_string(&devcontainer_path).ok()?;
    let devcontainer: JsonValue = serde_json::from_str(&contents).ok()?;
    let image = devcontainer.get("image").and_then(JsonValue::as_str)?;
    let hinted_node = devcontainer_node_image_version(image)?;
    if version_matches(&required, &hinted_node) {
        return None;
    }

    Some(Finding::identified(
        "OTA_DEVCONTAINER_RUNTIME_DRIFT",
        "contract",
        "repo_contract",
        FindingSeverity::Warn,
        "Devcontainer drift: Node image differs from repo runtime",
        format!(
            "`{}` declares image `{image}`, which hints Node `{hinted_node}`, but the repo contract requires Node version `{required}`",
            compact_display_path(&devcontainer_path)
        ),
        format!(
            "update `{}` to a Node image satisfying `{required}`, or narrow the repo contract if the devcontainer is intentionally legacy",
            compact_display_path(&devcontainer_path)
        ),
    ))
}

fn detect_devcontainer_package_manager_drift(
    contract: &Contract,
    contract_path: &Path,
) -> Option<Finding> {
    let expected_manager = repo_node_package_manager_truth(contract)?;
    let root = contract_working_dir(contract_path);
    let devcontainer_path = root.join(".devcontainer").join("devcontainer.json");
    if !devcontainer_path.exists() {
        return None;
    }

    let contents = fs::read_to_string(&devcontainer_path).ok()?;
    let devcontainer: JsonValue = serde_json::from_str(&contents).ok()?;
    let post_create = devcontainer
        .get("postCreateCommand")
        .and_then(JsonValue::as_str)?;
    let actual_manager = command_package_manager_token(post_create)?;
    if actual_manager == expected_manager {
        return None;
    }

    Some(Finding::identified(
        "OTA_DEVCONTAINER_PACKAGE_MANAGER_DRIFT",
        "contract",
        "repo_contract",
        FindingSeverity::Warn,
        format!(
            "Devcontainer drift: bootstrap command uses `{actual_manager}` instead of repo package manager `{expected_manager}`"
        ),
        format!(
            "`{}` declares `postCreateCommand: {post_create}`, but the repo contract's Node package manager truth is `{expected_manager}`",
            compact_display_path(&devcontainer_path)
        ),
        format!(
            "update `{}` so `postCreateCommand` uses `{expected_manager}`, or narrow the repo contract if a different package manager is intentionally canonical",
            compact_display_path(&devcontainer_path)
        ),
    ))
}

fn repo_node_package_manager_truth(contract: &Contract) -> Option<&str> {
    const NODE_PACKAGE_MANAGERS: [&str; 4] = ["pnpm", "npm", "yarn", "bun"];

    if let Some(toolchain) = contract.toolchains.get("node") {
        let mut matches = toolchain
            .package_managers
            .keys()
            .filter_map(|name| {
                NODE_PACKAGE_MANAGERS
                    .contains(&name.as_str())
                    .then_some(name.as_str())
            })
            .collect::<Vec<_>>();
        matches.sort_unstable();
        matches.dedup();
        if matches.len() == 1 {
            return matches.into_iter().next();
        }
    }

    let mut matches = contract
        .tools
        .keys()
        .filter_map(|name| {
            NODE_PACKAGE_MANAGERS
                .contains(&name.as_str())
                .then_some(name.as_str())
        })
        .collect::<Vec<_>>();
    matches.sort_unstable();
    matches.dedup();
    (matches.len() == 1).then(|| matches[0])
}

fn devcontainer_node_image_version(image: &str) -> Option<String> {
    let image_without_digest = image.split('@').next()?.trim();
    let (repository, tag) = image_without_digest.rsplit_once(':')?;
    let image_name = repository.rsplit('/').next().unwrap_or(repository);
    if !image_name.contains("node") {
        return None;
    }

    let version = tag
        .chars()
        .skip_while(|ch| !ch.is_ascii_digit())
        .take_while(|ch| ch.is_ascii_digit() || *ch == '.')
        .collect::<String>();
    if version.is_empty() {
        None
    } else {
        Some(version)
    }
}

fn command_package_manager_token(command: &str) -> Option<&'static str> {
    let mut matches = command
        .split(|ch: char| !(ch.is_ascii_alphanumeric() || ch == '-' || ch == '_'))
        .filter_map(|token| match token {
            "pnpm" => Some("pnpm"),
            "npm" => Some("npm"),
            "yarn" => Some("yarn"),
            "bun" => Some("bun"),
            _ => None,
        })
        .collect::<Vec<_>>();
    matches.sort_unstable();
    matches.dedup();
    (matches.len() == 1).then(|| matches[0])
}

fn extract_version_token(output: &str) -> Option<String> {
    output
        .split_whitespace()
        .find(|token| token.chars().any(|ch| ch.is_ascii_digit()))
        .map(|token| {
            token
                .trim_matches(|ch: char| !ch.is_ascii_alphanumeric() && ch != '.' && ch != '-')
                .trim_start_matches('v')
                .to_string()
        })
        .filter(|token| !token.is_empty())
}

fn extract_backticked_after(value: &str, marker: &str) -> Option<String> {
    let rest = value.split_once(marker)?.1;
    let end = rest.find('`')?;
    Some(rest[..end].to_string())
}

fn finding_probe_path(why: &str) -> Option<String> {
    extract_backticked_after(why, "ota probed `")
}

fn finding_probe_command(why: &str) -> Option<String> {
    extract_backticked_after(why, " with `")
}

pub(crate) fn finding_targets_container_image(why: &str) -> bool {
    why.contains("inside container image `")
        || why.contains("inside the configured container image")
}

pub(crate) fn finding_targets_remote_backend(why: &str) -> bool {
    why.contains("through the declared remote backend")
        || why.contains("through remote context `")
        || why.contains("through the selected remote backend")
}

pub(crate) fn version_matches(requirement: &str, actual: &str) -> bool {
    let requirement = requirement.trim();
    if requirement == "*" {
        return true;
    }

    if requirement.contains("||") {
        return requirement
            .split("||")
            .map(str::trim)
            .filter(|branch| !branch.is_empty())
            .any(|branch| version_matches(branch, actual));
    }

    if requirement.contains(',') || requirement.starts_with(['^', '~', '=']) {
        if let (Some(req), Some(actual_version)) = (
            parse_semver_requirement(requirement),
            parse_semver_candidate(actual),
        ) {
            return req.matches(&actual_version);
        }
    }

    if let Some(maximum) = requirement.strip_prefix("<=") {
        return compare_version_tokens(actual, maximum.trim())
            .is_some_and(|ordering| ordering <= 0);
    }

    if let Some(maximum) = requirement.strip_prefix('<') {
        return compare_version_tokens(actual, maximum.trim()).is_some_and(|ordering| ordering < 0);
    }

    if let Some(minimum) = requirement.strip_prefix(">=") {
        return compare_version_tokens(actual, minimum.trim())
            .is_some_and(|ordering| ordering >= 0);
    }

    if let Some(minimum) = requirement.strip_prefix('>') {
        return compare_version_tokens(actual, minimum.trim()).is_some_and(|ordering| ordering > 0);
    }

    if let Some(compatible) = requirement.strip_prefix('^') {
        return version_matches_caret(actual, compatible.trim());
    }

    actual == requirement
        || actual.starts_with(&format!("{requirement}."))
        || compare_version_tokens(actual, requirement).is_some_and(|ordering| ordering == 0)
}

fn version_matches_caret(actual: &str, base: &str) -> bool {
    let actual_parts = match parse_version_parts(actual) {
        Some(parts) => parts,
        None => return false,
    };
    let base_parts = match parse_version_parts(base) {
        Some(parts) => parts,
        None => return false,
    };

    if compare_parts(&actual_parts, &base_parts) < 0 {
        return false;
    }

    let upper_bound = caret_upper_bound(&base_parts);
    compare_parts(&actual_parts, &upper_bound) < 0
}

fn caret_upper_bound(base: &[u64]) -> Vec<u64> {
    let mut upper = base.to_vec();
    let pivot = base.iter().position(|part| *part != 0).unwrap_or(0);

    if upper.len() <= pivot {
        upper.resize(pivot + 1, 0);
    }

    upper[pivot] += 1;
    for part in upper.iter_mut().skip(pivot + 1) {
        *part = 0;
    }

    upper
}

fn compare_version_tokens(actual: &str, minimum: &str) -> Option<i8> {
    let actual_parts = parse_version_parts(actual)?;
    let minimum_parts = parse_version_parts(minimum)?;
    Some(compare_parts(&actual_parts, &minimum_parts))
}

fn compare_parts(left: &[u64], right: &[u64]) -> i8 {
    let len = left.len().max(right.len());

    for index in 0..len {
        let left = *left.get(index).unwrap_or(&0);
        let right = *right.get(index).unwrap_or(&0);
        if left > right {
            return 1;
        }
        if left < right {
            return -1;
        }
    }

    0
}

fn parse_semver_candidate(value: &str) -> Option<semver::Version> {
    let mut parts = parse_version_parts(value)?;
    if parts.len() > 3 {
        parts.truncate(3);
    }
    while parts.len() < 3 {
        parts.push(0);
    }
    Some(semver::Version::new(parts[0], parts[1], parts[2]))
}

fn parse_version_parts(input: &str) -> Option<Vec<u64>> {
    let parts = input
        .trim()
        .split('.')
        .map(|part| {
            let digits = part
                .chars()
                .skip_while(|ch| !ch.is_ascii_digit())
                .take_while(|ch| ch.is_ascii_digit())
                .collect::<String>();
            if digits.is_empty() {
                None
            } else {
                digits.parse::<u64>().ok()
            }
        })
        .collect::<Option<Vec<_>>>()?;

    if parts.is_empty() { None } else { Some(parts) }
}

#[cfg(unix)]
fn shell_command(command: &str) -> Command {
    let mut shell = Command::new("sh");
    shell.arg("-lc").arg(command);
    shell
}

#[cfg(windows)]
fn shell_command(command: &str) -> Command {
    if looks_like_posix_script(command) && has_bash() {
        let mut shell = Command::new("bash");
        shell.arg("-lc").arg(command);
        shell
    } else {
        let mut shell = Command::new("cmd");
        shell.arg("/C").arg(command);
        shell
    }
}

#[cfg(windows)]
fn looks_like_posix_script(command: &str) -> bool {
    const POSIX_MARKERS: [&str; 8] = [
        "command -v ",
        "; then",
        " fi",
        "&& ",
        " || ",
        "<<'PY'",
        "sh -lc ",
        "bash -lc ",
    ];
    if command.starts_with("cmd ") || command.starts_with("cmd/") {
        return false;
    }
    if command.starts_with("powershell ") || command.starts_with("pwsh ") {
        return false;
    }
    command.contains("&&")
        || command.contains("||")
        || POSIX_MARKERS.iter().any(|marker| command.contains(marker))
        || command.contains(" ${")
}

#[cfg(windows)]
fn has_bash() -> bool {
    static HAS_BASH: std::sync::OnceLock<bool> = std::sync::OnceLock::new();
    *HAS_BASH.get_or_init(|| {
        Command::new("bash")
            .arg("--version")
            .status()
            .map(|status| status.success())
            .unwrap_or(false)
    })
}

fn shell_single_quote(command: &str) -> String {
    let escaped = command.replace('\'', r"'\''");
    format!("'{}'", escaped)
}

#[cfg(test)]
mod tests {
    use std::collections::BTreeMap;
    use std::collections::BTreeSet;
    use std::env;
    use std::fs;
    use std::io::{ErrorKind, Read, Write};
    use std::net::{TcpListener, TcpStream};
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use std::thread;
    use std::time::{Duration, Instant};

    use crate::parser::parse_contract_str;
    use crate::policy_pack::{ProvisioningActionKind, ProvisioningTargetKind};
    use crate::provisioning::{
        ProvisioningExecutionTarget, ProvisioningFailureDiagnosis, ProvisioningFailureKind,
    };
    use crate::runner::{ExecutionOverrides, HttpReadinessRequest};
    use crate::schema::{ServiceSpec, ToolchainFulfillmentSource};
    #[cfg(windows)]
    use crate::test_support::cwd_mutex_lock;
    use crate::test_support::{cwd_mutex_lock, env_mutex_lock};
    use tempfile::TempDir;

    use super::{
        Backend, CheckStatus, DoctorMode, Finding, FindingSeverity,
        compose_service_healthcheck_command, diagnose_checks_only, diagnose_contract,
        diagnose_contract_in_mode,
        diagnose_contract_with_mode_and_lifecycle_for_workflow_with_overrides,
        diagnose_preconditions, diagnose_preconditions_with_mode, provider_hint_remediation,
        tool_executable_name, version_matches,
    };

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

    fn synthetic_contract_path() -> &'static Path {
        static PATH: OnceLock<PathBuf> = OnceLock::new();
        PATH.get_or_init(|| {
            let dir = std::env::temp_dir().join("ota-unit-test-contract");
            fs::create_dir_all(&dir).unwrap();
            dir.join("ota.yaml")
        })
        .as_path()
    }

    fn finding_contract_projection(lane: &str, finding: &Finding) -> serde_json::Value {
        let json = serde_json::to_value(finding).expect("finding should serialize");
        serde_json::json!({
            "lane": lane,
            "code": json.get("code").and_then(|value| value.as_str()).expect("finding code"),
            "category": json.get("category").and_then(|value| value.as_str()).expect("finding category"),
            "owner": json.get("owner").and_then(|value| value.as_str()).expect("finding owner"),
            "severity": json.get("severity").and_then(|value| value.as_str()).expect("finding severity"),
            "summary": json.get("summary").and_then(|value| value.as_str()).expect("finding summary"),
            "evidence_source": json
                .get("evidence")
                .and_then(|value| value.get("source"))
                .and_then(|value| value.as_str()),
            "provenance_key": json.get("provenance_key").and_then(|value| value.as_str()),
            "policy_reason": json.get("policy_reason").and_then(|value| value.as_str()),
        })
    }

    #[test]
    fn finding_deserialize_restores_identity_from_flat_receipt_fields() {
        let json = serde_json::json!({
            "code": "OTA_SELECTED_TASK_PATH_EXTERNAL_STATE",
            "category": "contract",
            "owner": "repo_contract",
            "severity": "warn",
            "summary": "Selected task path mutates external state: clickhouse, postgres",
            "why": "the selected task path declares external state",
            "next": "keep effects.external_state explicit"
        });

        let finding: Finding =
            serde_json::from_value(json).expect("flat receipt finding should deserialize");

        assert_eq!(finding.code(), "OTA_SELECTED_TASK_PATH_EXTERNAL_STATE");
        assert_eq!(finding.category(), "contract");
        assert_eq!(finding.owner(), "repo_contract");
    }

    fn current_native_package_test_case() -> (&'static str, &'static str, &'static str, &'static str)
    {
        match super::current_os() {
            "macos" => (
                "macos",
                "brew",
                "pkg-config",
                "ruby-native-build-tools-macos",
            ),
            "windows" => (
                "windows",
                "winget",
                "Microsoft.VisualStudio.2022.BuildTools",
                "ruby-native-build-tools-windows",
            ),
            _ => (
                "linux",
                "apt",
                "build-essential",
                "ruby-native-build-tools-linux",
            ),
        }
    }

    fn write_native_package_policy_fixture(
        fixture: &TempDir,
        approved_package: &str,
    ) -> (&'static str, &'static str, &'static str, &'static str) {
        let (platform_name, package_source, package_name, check_name) =
            current_native_package_test_case();
        fs::write(
            fixture.path().join("ota.yaml"),
            format!(
                r#"
version: 1
project:
  name: ota
native_prerequisites:
  ruby-native-build-tools:
    platforms:
      {platform_name}:
        check: {check_name}
        {package_source}:
          - {package_name}
checks:
  - name: {check_name}
    kind: precondition
    severity: error
    run: sh -c "cc --version && pkg-config --version"
tasks:
  setup:
    run: bundle install
    requirements:
      native:
        - ruby-native-build-tools
workflows:
  default: app
  app:
    setup:
      task: setup
"#,
            ),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            format!(
                r#"
policies:
  native_packages:
    {package_source}:
      approved:
        - {approved_package}
"#,
            ),
        )
        .unwrap();
        (platform_name, package_source, package_name, check_name)
    }

    fn drain_probe_request_if_available(stream: &mut TcpStream) {
        // Let the client start the request before responding so the fake server does
        // not race the probe, but bound the wait to keep the tests deterministic.
        let _ = stream.set_nodelay(true);
        let deadline = Instant::now() + Duration::from_millis(500);
        let mut buffer = [0u8; 256];
        loop {
            match stream.peek(&mut buffer) {
                Ok(0) => {}
                Ok(_) => {
                    let _ = stream.set_read_timeout(Some(Duration::from_millis(25)));
                    let _ = stream.read(&mut buffer);
                    break;
                }
                Err(error)
                    if matches!(error.kind(), ErrorKind::WouldBlock | ErrorKind::TimedOut) => {}
                Err(_) => break,
            }

            if Instant::now() >= deadline {
                break;
            }

            thread::sleep(Duration::from_millis(10));
        }
    }

    #[cfg(windows)]
    fn normalize_windows_fake_container_probe_matchers(script: &str) -> String {
        // Replace `echo %* | findstr` probe matchers with cmd.exe positional-arg checks.
        //
        // The old approach captured %* into a variable via `set "__OTA_ARGS=%*"`, which
        // breaks when %* contains embedded `"` characters (as the version probe script does).
        //
        // Instead we use stable positional args:
        //   version probe:      docker run --rm --name <n> --entrypoint sh <img> -c <script>
        //                       → %2=--rm, %3=--name  (always stable)
        //   provisioning probe: docker run --rm -i --entrypoint sh -v ... <img> -lc <shell>
        //                       → %3=-i               (always stable)
        let Some(remainder) = script.strip_prefix("@echo off\r\n") else {
            return script.to_string();
        };
        let lines: Vec<&str> = remainder.split("\r\n").collect();
        let mut result = Vec::new();
        let mut needs_extra_close_index: Option<usize> = None;

        for (i, line) in lines.iter().enumerate() {
            // Check if we need to add extra closing paren from version probe
            if let Some(idx) = needs_extra_close_index {
                if idx == i && line.trim() == ")" {
                    result.push("))".to_string());
                    needs_extra_close_index = None;
                    continue;
                }
            }

            // Version probe line: detect by the "command -v '" pattern + block opener
            if line.contains("echo %* | findstr /C:\"command -v '") && line.ends_with(">nul && (") {
                let indent = line.split("echo %* |").next().unwrap_or_default();
                result.push(format!(
                    "{indent}if \"%2\"==\"--rm\" (if \"%3\"==\"--name\" ("
                ));
                // Find the matching closing paren for the next iteration
                let mut paren_depth = 1;
                for j in (i + 1)..lines.len() {
                    if lines[j].contains("(") {
                        paren_depth += lines[j].chars().filter(|&c| c == '(').count();
                    }
                    if lines[j].contains(")") {
                        paren_depth -= lines[j].chars().filter(|&c| c == ')').count();
                        if paren_depth == 0 {
                            needs_extra_close_index = Some(j);
                            break;
                        }
                    }
                }
            }
            // Any remaining probe-line that uses echo %* | … >nul && ( is a provisioning
            // probe. Detect it by the reliable %3==-i flag present on all Ephemeral runs.
            else if line.contains("echo %* |") && line.ends_with(">nul && (") {
                let indent = line.split("echo %* |").next().unwrap_or_default();
                result.push(format!("{indent}if \"%3\"==\"-i\" ("));
            } else {
                result.push(line.to_string());
            }
        }

        format!("@echo off\r\n{}", result.join("\r\n"))
    }

    fn write_fake_command(bin_dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
        let path = if cfg!(windows) {
            let name_path = Path::new(name);
            if name_path.extension().is_some() {
                bin_dir.join(name_path)
            } else {
                bin_dir.join(format!("{name}.cmd"))
            }
        } else {
            bin_dir.join(name)
        };

        #[cfg(windows)]
        let body = if (name == "docker" || name == "podman") && !body.contains("\"%1\"==\"info\"") {
            let remainder = body.strip_prefix("@echo off\r\n").unwrap_or(body);
            format!("@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\n{remainder}")
        } else {
            body.to_string()
        };

        #[cfg(windows)]
        let body = normalize_windows_fake_container_probe_matchers(&body);

        #[cfg(not(windows))]
        let body = body.to_string();

        fs::write(&path, body).unwrap();

        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
        }

        path
    }

    #[cfg(windows)]
    fn assert_windows_path_eq(actual: Option<&Path>, expected: &Path) {
        assert_eq!(
            actual.map(|path| path.display().to_string().to_ascii_lowercase()),
            Some(expected.display().to_string().to_ascii_lowercase())
        );
    }

    #[cfg(windows)]
    fn assert_windows_path_text_contains(text: &str, path: &Path) {
        assert!(
            text.to_ascii_lowercase()
                .contains(&path.display().to_string().to_ascii_lowercase()),
            "{text}"
        );
    }

    #[cfg(windows)]
    fn fake_remote_ssh_body(version: &str) -> String {
        format!(
            "@echo off\r\necho Linux\r\necho {started} 1>&2\r\necho {path}/usr/bin/jq 1>&2\r\necho {version}\r\nexit /b 0\r\n",
            started = super::CONTAINER_PROBE_STARTED_MARKER,
            path = super::CONTAINER_PROBE_PATH_MARKER,
        )
    }

    #[cfg(unix)]
    fn fake_remote_ssh_body(_: &str) -> String {
        r#"#!/bin/sh
target="$1"
shift
[ -n "$target" ] || exit 1
[ "$#" -ge 1 ] || exit 1
cmd="$*"
case "$cmd" in
  "sh -lc "*)
    eval "remote_script=${cmd#sh -lc }"
    exec /bin/sh -c "$remote_script"
    ;;
esac
exec /bin/sh -c "$cmd"
"#
        .to_string()
    }

    fn write_fake_remote_backend_provider(bin_dir: &Path) {
        write_fake_command(
            bin_dir,
            "fake-remote-provider",
            if cfg!(windows) {
                "@echo off\r\necho %OTA_BACKEND_PROVIDER_COMMAND% | findstr /C:\"uname -s\" >nul && (\r\n  echo {\"ok\":true,\"result\":{\"exit_code\":0,\"stdout\":\"Linux\\n\",\"stderr\":\"\",\"target\":\"sandbox-dev\"},\"errors\":[]}\r\n  exit /b 0\r\n)\r\necho {\"ok\":true,\"result\":{\"exit_code\":0,\"stdout\":\"jq-1.8.1\\n\",\"stderr\":\"__OTA_CONTAINER_PROBE_STARTED__\\n__OTA_RESOLVED_PATH__/usr/bin/jq\\n\",\"target\":\"sandbox-dev\"},\"errors\":[]}\r\n"
            } else {
                r#"#!/bin/sh
case "${OTA_BACKEND_PROVIDER_COMMAND:-}" in
  *"uname -s"*)
    printf '%s' '{"ok":true,"result":{"exit_code":0,"stdout":"Linux\n","stderr":"","target":"sandbox-dev"},"errors":[]}'
    ;;
  *)
    printf '%s' '{"ok":true,"result":{"exit_code":0,"stdout":"jq-1.8.1\n","stderr":"__OTA_CONTAINER_PROBE_STARTED__\n__OTA_RESOLVED_PATH__/usr/bin/jq\n","target":"sandbox-dev"},"errors":[]}'
    ;;
esac
"#
            },
        );
    }

    #[test]
    fn prioritizes_blocking_env_errors_before_warnings() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  definitely-not-installed-native-tool: "*"
env:
  vars:
    OTA_DOCTOR_REQUIRED_MISSING:
      required: true
services:
  cache:
    required: false
    healthcheck: exit 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        assert!(!report.ok);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(
            report.findings[0].summary,
            "Missing environment variable: OTA_DOCTOR_REQUIRED_MISSING"
        );
        assert!(!report.findings.is_empty());
    }

    #[test]
    fn warns_when_ephemeral_lifecycle_is_only_advisory() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  lifecycle: ephemeral
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Ephemeral lifecycle is advisory in native mode")
            .expect("expected ephemeral lifecycle advisory finding");
        assert_eq!(finding.severity, FindingSeverity::Warn);
    }

    #[test]
    fn doctor_warns_when_depends_on_crosses_execution_boundaries() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
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
      container:
        image: node:24-bookworm
    verify:
      backend: native
tasks:
  setup:
    context: app
    run: cargo --version
  build:
    context: verify
    run: rustc --version
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let mut findings = Vec::new();
        super::diagnose_contract_advisories(
            &contract,
            synthetic_contract_path(),
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
            None,
        );

        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary
                    == "Task `build` depends_on `setup` across different execution boundaries"
                && finding
                    .why
                    .contains("only durable external side effects survive")
        }));
    }

    #[test]
    fn doctor_does_not_warn_depends_on_crosses_execution_boundaries_when_overridden_native() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
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
      container:
        image: node:24-bookworm
    verify:
      backend: native
tasks:
  setup:
    context: app
    run: cargo --version
  build:
    context: verify
    run: rustc --version
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let report = diagnose_contract_with_mode_and_lifecycle_for_workflow_with_overrides(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            None,
            None,
            crate::runner::ExecutionOverrides {
                backend: Some(Backend::Native),
                lifecycle: None,
                host_port: None,
                memory: None,
                skip_deps: false,
            },
        );

        assert!(
            report.ok,
            "findings={:?}",
            report
                .findings
                .iter()
                .map(|finding| finding.summary.as_str())
                .collect::<Vec<_>>()
        );
        let boundary_crossings: Vec<_> = report
            .findings
            .iter()
            .filter(|finding| {
                finding.summary
                    == "Task `build` depends_on `setup` across different execution boundaries"
            })
            .collect();
        assert!(boundary_crossings.is_empty());
    }

    #[test]
    fn doctor_warns_when_attachment_path_is_likely_unused() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: maven:3.9.14-eclipse-temurin-21-noble
      attachments:
        isolated_paths:
          - .m2
tasks:
  build:
    context: app
    env:
      MAVEN_OPTS: -Dmaven.repo.local=/tmp/m2/repository
    run: mvn -q test
"#,
        )
        .unwrap();

        let mut findings = Vec::new();
        super::diagnose_contract_advisories(
            &contract,
            synthetic_contract_path(),
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
            None,
        );

        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary == "Attachment `.m2` may be unused in context `app`"
                && finding.why.contains("point Maven at `/workspace/.m2`")
        }));
    }

    #[test]
    fn doctor_warns_when_task_mutates_managed_isolated_path() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: verify:ctx
  contexts:
    verify:ctx:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:24-bookworm
      attachments:
        isolated_paths:
          - .next
tasks:
  build:
    run: rm -rf .next && next build
"#,
        )
        .unwrap();

        let mut findings = Vec::new();
        super::diagnose_contract_advisories(
            &contract,
            synthetic_contract_path(),
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
            None,
        );

        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary == "Task `build` mutates managed isolated path `.next`"
                && finding
                    .why
                    .contains("execution.contexts.verify:ctx.attachments.isolated_paths")
                && finding.provenance().as_deref() == Some("repo contract")
        }));
    }

    #[test]
    fn doctor_warns_when_isolated_yarn_path_can_shadow_release_artifacts() {
        let tempdir = TempDir::new().unwrap();
        let contract_path = tempdir.path().join("ota.yaml");
        fs::write(
            tempdir.path().join(".yarnrc.yml"),
            "yarnPath: .yarn/releases/yarn-4.12.0.cjs\n",
        )
        .unwrap();
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: node:22-bookworm
      attachments:
        isolated_paths:
          - .yarn
tasks:
  setup:
    context: app
    run: yarn install --immutable
"#,
        )
        .unwrap();

        let mut findings = Vec::new();
        super::diagnose_contract_advisories(
            &contract,
            &contract_path,
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
            None,
        );

        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary.contains("Isolated path `.yarn`")
                && finding.summary.contains("Yarn release artifacts")
                && finding.why.contains(".yarn/releases/yarn-*.cjs")
        }));
    }

    #[test]
    fn doctor_warns_for_legacy_node_runtime_tool_split_contracts() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
runtimes:
  node: "22"
tools:
  pnpm: "11"
tasks:
  test:
    run: pnpm test
"#,
        )
        .unwrap();

        let mut findings = Vec::new();
        super::diagnose_contract_advisories(
            &contract,
            synthetic_contract_path(),
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
            None,
        );

        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary
                    == "Node contract uses split ownership (`runtimes.node` + tools: pnpm)"
                && finding.why.contains("split Node ownership")
                && finding.next.contains("toolchains.node")
        }));
    }

    #[test]
    fn doctor_warns_when_agent_safe_task_declares_network_and_external_state() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
    safe_for_agent: true
    effects:
      network: true
      external_state:
        - docker
"#,
        )
        .unwrap();

        let mut findings = Vec::new();
        super::diagnose_contract_advisories(
            &contract,
            synthetic_contract_path(),
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
            None,
        );

        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary == "Agent-safe task `setup` requires network access"
        }));
        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary == "Agent-safe task `setup` mutates external state: docker"
        }));
    }

    #[test]
    fn doctor_labels_agent_safe_dependency_hydration_narrowly() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
    safe_for_agent: true
    effects:
      network: true
      network_kind: dependency_hydration
"#,
        )
        .unwrap();

        let mut findings = Vec::new();
        super::diagnose_contract_advisories(
            &contract,
            synthetic_contract_path(),
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
            None,
        );

        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary
                    == "Agent-safe task `setup` performs network dependency hydration"
        }));
    }

    #[test]
    fn doctor_scopes_agent_safe_task_effect_advisories_to_selected_workflow() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
    safe_for_agent: true
    effects:
      network: true
  publish:
    run: docker compose up -d
    safe_for_agent: true
    effects:
      external_state:
        - docker
workflows:
  default: app
  app:
    setup:
      task: setup
"#,
        )
        .unwrap();

        let report = diagnose_contract_with_mode_and_lifecycle_for_workflow_with_overrides(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            None,
            Some("app"),
            crate::runner::ExecutionOverrides::default(),
        );

        assert!(report.findings.iter().any(|finding| {
            finding.summary == "Agent-safe task `setup` requires network access"
        }));
        assert!(!report.findings.iter().any(|finding| {
            finding.summary == "Agent-safe task `publish` mutates external state: docker"
        }));
    }

    #[test]
    fn doctor_skips_agent_safe_dependency_hydration_advisory_for_first_class_hydration() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    version: "22"
tasks:
  setup:
    safe_for_agent: true
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: node_package_manager
        cwd: .
        manager: pnpm
        mode: install
        frozen_lockfile: true
    requirements:
      toolchains:
        - node
    effects:
      writes:
        - node_modules
      network: true
      network_kind: dependency_hydration
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(!report.findings.iter().any(|finding| {
            finding.summary == "Agent-safe task `setup` performs network dependency hydration"
        }));
        assert!(report.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Info
                && finding.summary
                    == "Selected task path performs network dependency hydration: setup"
        }));
    }

    #[test]
    fn doctor_surfaces_selected_task_path_effects() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install
    effects:
      network: true
  services:up:
    run: docker compose up -d postgres
    effects:
      external_state:
        - docker
workflows:
  default: app
  app:
    setup:
      task: setup
    run:
      task: services:up
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Info
                && finding.summary == "Selected task path requires network access: setup"
        }));
        assert!(report.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary == "Selected task path mutates external state: docker"
        }));
    }

    #[test]
    fn doctor_surfaces_selected_task_path_dependency_hydration_effects() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  setup:
    run: pnpm install --frozen-lockfile
    effects:
      network: true
      network_kind: dependency_hydration
workflows:
  default: app
  app:
    setup:
      task: setup
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Info
                && finding.summary
                    == "Selected task path performs network dependency hydration: setup"
        }));
    }

    #[test]
    fn doctor_surfaces_selected_task_path_integration_test_effects() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  test:live:
    run: npm run test:live
    effects:
      network: true
      network_kind: integration_test
workflows:
  default: verify
  verify:
    run:
      task: test:live
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Info
                && finding.summary
                    == "Selected task path performs network integration testing: test:live"
        }));
    }

    #[test]
    fn preconditions_in_container_mode_skip_host_bound_env_checks() {
        let _guard = env_mutex_lock();
        let tempdir = TempDir::new().unwrap();
        let bin_dir = tempdir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var("PATH").unwrap_or_default();
        let path_separator = if cfg!(windows) { ";" } else { ":" };
        let new_path = format!("{}{}{}", bin_dir.display(), path_separator, original_path);
        unsafe {
            env::set_var("PATH", new_path);
        }
        write_fake_command(
            &bin_dir,
            if cfg!(windows) {
                "docker.cmd"
            } else {
                "docker"
            },
            if cfg!(windows) {
                "@echo off\r\nif \"%*\"==\"version\" echo Docker version 29.3.1\r\n"
            } else {
                "#!/bin/sh\nif [ \"$1\" = \"version\" ]; then echo 'Docker version 29.3.1'; fi\n"
            },
        );

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: jdxcode/mise:latest
env:
  vars:
    OTA_CONTAINER_ONLY_REQUIRED:
      required: true
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_preconditions_with_mode(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
        );

        assert!(report.ok, "findings={:?}", report.findings);
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.summary.contains("Missing environment variable")),
            "unexpected host-bound env finding: {:?}",
            report.findings
        );

        unsafe {
            env::set_var("PATH", original_path);
        }
    }

    #[test]
    fn container_mode_uses_linux_policy_sources_for_provisioning() {
        let tempdir = TempDir::new().unwrap();
        let contract_path = tempdir.path().join("ota.yaml");
        let policy_dir = tempdir.path().join(".ota");
        fs::create_dir_all(&policy_dir).unwrap();

        fs::write(
            policy_dir.join("org-policy.yaml"),
            r#"
policies:
  provisioning:
    curl:
      source: brew
      approved_versions:
        - "8.13.0"
      platforms:
        macos:
          source: brew
          approved_versions:
            - "8.13.0"
        linux:
          source: apt
          package: curl
          approved_versions:
            - "8.13.0"
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: persistent
  backends:
    container:
      image: jdxcode/mise:latest
runtimes:
  curl: 8.13.0
"#,
        )
        .unwrap();

        let report =
            diagnose_preconditions_with_mode(&contract, &contract_path, DoctorMode::Container);

        let provisioning = report
            .provisioning
            .expect("expected provisioning diagnostics in container mode");
        assert_eq!(provisioning.request.actions.len(), 1);
        assert_eq!(provisioning.request.actions[0].source, "apt");
    }

    #[test]
    fn warns_when_container_ephemeral_only_applies_to_run() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: ghcr.io/ota/dev:latest
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        let warning = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Ephemeral lifecycle is execution-only")
            .expect("expected lifecycle warning for container+ephemeral configuration");
        assert_eq!(warning.severity, FindingSeverity::Warn);
    }

    #[test]
    fn reports_missing_container_backend_cli() {
        let _guard = env_mutex_lock();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/definitely-not-a-real-bin");
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  backends:
    container:
      image: ghcr.io/ota/dev:latest
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing container execution backend CLI: docker")
        );
    }

    #[test]
    fn reports_missing_remote_backend_cli() {
        let _guard = env_mutex_lock();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/definitely-not-a-real-bin");
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing execution backend CLI: ssh")
        );
    }

    #[test]
    fn reports_missing_tsh_remote_backend_cli() {
        let _guard = env_mutex_lock();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/definitely-not-a-real-bin");
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing execution backend CLI: tsh")
        );
    }

    #[test]
    fn reports_missing_kubectl_remote_backend_cli() {
        let _guard = env_mutex_lock();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/definitely-not-a-real-bin");
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: pod/ota-dev
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing execution backend CLI: kubectl")
        );
    }

    #[test]
    #[cfg(unix)]
    fn command_version_handles_go_subcommand() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();

        let go_path = bin_dir.join("go");
        fs::write(
            &go_path,
            "#!/bin/sh\nprintf 'go version go1.24.2 fake/amd64\\n'\n",
        )
        .unwrap();

        use std::os::unix::fs::PermissionsExt;
        let mut permissions = fs::metadata(&go_path).unwrap().permissions();
        permissions.set_mode(0o755);
        fs::set_permissions(&go_path, permissions).unwrap();

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        } else if cfg!(unix) {
            path_entries.push(PathBuf::from("/usr/bin"));
            path_entries.push(PathBuf::from("/bin"));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let version = super::command_version("go");

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(version.as_deref(), Some("go1.24.2"));
    }

    #[test]
    #[cfg(windows)]
    fn resolve_command_path_checks_current_directory_before_path() {
        let _guard = env_mutex_lock();
        let _cwd_guard = cwd_mutex_lock();
        let temp = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        let original_path = env::var_os("PATH");

        let local_npm = write_fake_command(temp.path(), "npm", "@echo off\r\necho 9.9.9\r\n");

        unsafe {
            env::set_var("PATH", temp.path().join("missing-bin"));
        }
        env::set_current_dir(temp.path()).unwrap();

        let resolved = super::resolve_command_path("npm");

        env::set_current_dir(original_dir).unwrap();
        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        #[cfg(windows)]
        assert_windows_path_eq(resolved.as_deref(), local_npm.as_path());
        #[cfg(not(windows))]
        assert_eq!(resolved.as_deref(), Some(local_npm.as_path()));
    }

    #[test]
    #[cfg(windows)]
    fn resolve_command_path_prefers_path_extensions_over_extensionless_file() {
        let _guard = env_mutex_lock();
        let _cwd_guard = cwd_mutex_lock();
        let temp = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        let original_path = env::var_os("PATH");

        let bare_path = temp.path().join("npm");
        fs::write(&bare_path, "not-an-exe").unwrap();
        let ext_path = write_fake_command(temp.path(), "npm", "@echo off\r\necho 9.9.9\r\n");

        unsafe {
            env::set_var("PATH", temp.path().join("missing-bin"));
        }
        env::set_current_dir(temp.path()).unwrap();

        let resolved = super::resolve_command_path("npm");

        env::set_current_dir(original_dir).unwrap();
        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        #[cfg(windows)]
        assert_windows_path_eq(resolved.as_deref(), ext_path.as_path());
        #[cfg(not(windows))]
        assert_eq!(resolved.as_deref(), Some(ext_path.as_path()));
    }

    #[test]
    #[cfg(windows)]
    fn resolve_command_path_includes_common_extensions_when_pathext_is_sparse() {
        let _guard = env_mutex_lock();
        let _cwd_guard = cwd_mutex_lock();
        let temp = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        let original_path = env::var_os("PATH");
        let original_pathext = env::var_os("PATHEXT");

        let npm_path = write_fake_command(temp.path(), "npm", "@echo off\r\necho 9.9.9\r\n");

        unsafe {
            env::set_var("PATH", temp.path());
            env::set_var("PATHEXT", ".EXE;.BAT");
        }
        env::set_current_dir(temp.path()).unwrap();

        let resolved = super::resolve_command_path("npm");

        env::set_current_dir(original_dir).unwrap();
        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        match original_pathext {
            Some(pathext) => unsafe {
                env::set_var("PATHEXT", pathext);
            },
            None => unsafe {
                env::remove_var("PATHEXT");
            },
        }

        #[cfg(windows)]
        assert_windows_path_eq(resolved.as_deref(), npm_path.as_path());
        #[cfg(not(windows))]
        assert_eq!(resolved.as_deref(), Some(npm_path.as_path()));
    }

    #[test]
    #[cfg(unix)]
    fn mise_shim_paths_use_command_name_probe_strategy() {
        use std::os::unix::fs::symlink;

        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let shim_bin = temp.path().join("shim-bin");
        let manager_bin = temp.path().join("manager-bin");
        fs::create_dir_all(&shim_bin).unwrap();
        fs::create_dir_all(&manager_bin).unwrap();

        let mise_path = write_fake_command(&manager_bin, "mise", "#!/bin/sh\necho v24.16.0\n");
        let shim_path = shim_bin.join("node");
        symlink(&mise_path, &shim_path).unwrap();
        assert!(super::should_probe_via_command_name("node", &shim_path));
        assert!(!super::should_probe_via_command_name(
            shim_path.to_str().unwrap(),
            &shim_path
        ));
    }

    #[test]
    #[cfg(unix)]
    fn command_version_probe_uses_contract_working_dir_for_mise_shims() {
        use std::os::unix::fs::symlink;

        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        let repo_dir = temp.path().join("repo-22");
        fs::create_dir_all(&bin_dir).unwrap();
        fs::create_dir_all(&repo_dir).unwrap();

        let mise_path = write_fake_command(
            &bin_dir,
            "mise",
            "#!/bin/sh\ncase \"$PWD\" in\n  *repo-22*) echo v22.22.3 ;;\n  *) echo v24.16.0 ;;\nesac\n",
        );
        let shim_path = bin_dir.join("node");
        symlink(&mise_path, &shim_path).unwrap();

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let probe = super::command_version_probe_in_working_dir("node", &repo_dir);

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        match probe.outcome {
            super::CommandVersionProbeOutcome::Version(actual) => {
                assert_eq!(actual, "22.22.3");
            }
            other => panic!("expected version probe result, got {other:?}"),
        }
    }

    #[test]
    fn reports_tool_probe_failures_with_resolved_probe_path() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let npm_path = write_fake_command(
            &bin_dir,
            "npm",
            if cfg!(windows) {
                "@echo off\r\nexit /b 1\r\n"
            } else {
                "#!/bin/sh\nexit 1\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        } else if cfg!(unix) {
            path_entries.push(PathBuf::from("/usr/bin"));
            path_entries.push(PathBuf::from("/bin"));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  npm: "*"
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Tool probe failed: npm")
            .expect("expected tool probe failure finding");
        #[cfg(windows)]
        assert_windows_path_text_contains(&finding.why, npm_path.as_path());
        #[cfg(not(windows))]
        assert!(finding.why.contains(&format!(
            "ota probed `{}` with `npm --version`",
            npm_path.display()
        )));
        assert_eq!(finding.evidence().command, "npm --version");
        #[cfg(windows)]
        assert_eq!(
            finding.evidence().path.to_ascii_lowercase(),
            npm_path.display().to_string().to_ascii_lowercase()
        );
        #[cfg(not(windows))]
        assert_eq!(finding.evidence().path, npm_path.display().to_string());
    }

    #[test]
    fn reports_unparseable_tool_versions_with_resolved_probe_path() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let npm_path = write_fake_command(
            &bin_dir,
            "npm",
            if cfg!(windows) {
                "@echo off\r\necho ready\r\n"
            } else {
                "#!/bin/sh\necho ready\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        } else if cfg!(unix) {
            path_entries.push(PathBuf::from("/usr/bin"));
            path_entries.push(PathBuf::from("/bin"));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  npm: "*"
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Unparseable version for tool: npm")
            .expect("expected unparseable tool version finding");
        #[cfg(windows)]
        assert_windows_path_text_contains(&finding.why, npm_path.as_path());
        #[cfg(not(windows))]
        assert!(finding.why.contains(&format!(
            "ota probed `{}` with `npm --version`",
            npm_path.display()
        )));
        assert_eq!(finding.evidence().command, "npm --version");
        #[cfg(windows)]
        assert_eq!(
            finding.evidence().path.to_ascii_lowercase(),
            npm_path.display().to_string().to_ascii_lowercase()
        );
        #[cfg(not(windows))]
        assert_eq!(finding.evidence().path, npm_path.display().to_string());
    }

    #[test]
    fn probes_kubectl_with_version_client() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "kubectl",
            if cfg!(windows) {
                "@echo off\r\nif \"%1\"==\"version\" if \"%2\"==\"--client\" (\r\necho Client Version: v1.33.9\r\necho Kustomize Version: v5.6.0\r\nexit /b 0\r\n)\r\nexit /b 1\r\n"
            } else {
                "#!/bin/sh\nif [ \"$1\" = \"version\" ] && [ \"$2\" = \"--client\" ]; then\n  echo 'Client Version: v1.33.9'\n  echo 'Kustomize Version: v5.6.0'\n  exit 0\nfi\nexit 1\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        } else if cfg!(unix) {
            path_entries.push(PathBuf::from("/usr/bin"));
            path_entries.push(PathBuf::from("/bin"));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  kubectl: "*"
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(report.ok, "{report:?}");
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Tool probe failed: kubectl"),
            "{report:?}"
        );
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Unparseable version for tool: kubectl"),
            "{report:?}"
        );
    }

    #[test]
    fn probes_helm_with_version_short() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "helm",
            if cfg!(windows) {
                "@echo off\r\nif \"%1\"==\"version\" if \"%2\"==\"--short\" (\r\necho v3.21.1+gc56dd00\r\nexit /b 0\r\n)\r\nif \"%1\"==\"dependency\" if \"%2\"==\"build\" exit /b 0\r\nexit /b 1\r\n"
            } else {
                "#!/bin/sh\nif [ \"$1\" = \"version\" ] && [ \"$2\" = \"--short\" ]; then\n  echo 'v3.21.1+gc56dd00'\n  exit 0\nfi\nif [ \"$1\" = \"dependency\" ] && [ \"$2\" = \"build\" ]; then\n  exit 0\nfi\nexit 1\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        } else if cfg!(unix) {
            path_entries.push(PathBuf::from("/usr/bin"));
            path_entries.push(PathBuf::from("/bin"));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  helm: "*"
tasks:
  test:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: helm
        cwd: deploy/helm
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(report.ok, "{report:?}");
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Tool probe failed: helm"),
            "{report:?}"
        );
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Unparseable version for tool: helm"),
            "{report:?}"
        );
    }

    #[test]
    fn presence_only_command_executable_does_not_fail_doctor_when_version_probe_exits_nonzero() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "svcctl",
            if cfg!(windows) {
                "@echo off\r\nif \"%1\"==\"check\" exit /b 0\r\nexit /b 2\r\n"
            } else {
                "#!/bin/sh\nif [ \"$1\" = \"check\" ]; then\n  exit 0\nfi\nexit 2\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        } else if cfg!(unix) {
            path_entries.push(PathBuf::from("/usr/bin"));
            path_entries.push(PathBuf::from("/bin"));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  verify:
    command:
      exe: svcctl
      args: [check]
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(report.ok, "{report:?}");
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Tool probe failed: svcctl"),
            "{report:?}"
        );
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Unparseable version for tool: svcctl"),
            "{report:?}"
        );
    }

    #[test]
    fn explicit_tool_requirement_still_fails_when_version_probe_exits_nonzero() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "svcctl",
            if cfg!(windows) {
                "@echo off\r\nif \"%1\"==\"check\" exit /b 0\r\nexit /b 2\r\n"
            } else {
                "#!/bin/sh\nif [ \"$1\" = \"check\" ]; then\n  exit 0\nfi\nexit 2\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        } else if cfg!(unix) {
            path_entries.push(PathBuf::from("/usr/bin"));
            path_entries.push(PathBuf::from("/bin"));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  svcctl: "*"
tasks:
  verify:
    command:
      exe: svcctl
      args: [check]
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok, "{report:?}");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Tool probe failed: svcctl"),
            "{report:?}"
        );
    }

    #[test]
    fn reports_container_tool_probe_failures_with_container_probe_context() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            format!(
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\nif \"%1\"==\"ps\" exit /b 0\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"npm\" >nul && (\r\n    echo {started} 1>&2\r\n    echo {path}/usr/local/bin/npm 1>&2\r\n    exit /b 1\r\n  )\r\n)\r\nif \"%1\"==\"exec\" (\r\n  echo %* | findstr /C:\"npm\" >nul && (\r\n    echo {started} 1>&2\r\n    echo {path}/usr/local/bin/npm 1>&2\r\n    exit /b 1\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"info\" ] || [ \"$1\" = \"ps\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"run\" ] || [ \"$1\" = \"exec\" ]; then\n  case \"$*\" in\n    *\"npm\"*) echo '{started}' >&2; echo '{path}/usr/local/bin/npm' >&2; exit 1 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER
            )
        };
        write_fake_command(&bin_dir, "docker", &docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tools:
  npm: "*"
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let report = super::diagnose_contract_in_mode(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Tool probe failed: npm")
            .expect("expected tool probe failure finding");
        assert!(
            finding.why.contains(
                "ota probed `npm` inside container image `premium/test:latest` with `npm --version`"
            ),
            "{report:?}"
        );
        assert_eq!(finding.evidence().command, "npm --version");
        assert_eq!(finding.evidence().path, "npm");
        assert_eq!(finding.evidence().source, "container_target");
        assert_eq!(finding.owner(), "container_target");
    }

    #[test]
    fn reports_container_unparseable_fixture_without_exec_support_falls_back_to_probe_failure() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            format!(
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\nif \"%1\"==\"ps\" exit /b 0\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"npm\" >nul && (\r\n    echo not-a-version\r\n    echo {started} 1>&2\r\n    echo {path}/usr/local/bin/npm 1>&2\r\n    exit /b 0\r\n  )\r\n)\r\nif \"%1\"==\"exec\" (\r\n  echo %* | findstr /C:\"npm\" >nul && (\r\n    echo not-a-version\r\n    echo {started} 1>&2\r\n    echo {path}/usr/local/bin/npm 1>&2\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"info\" ] || [ \"$1\" = \"ps\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"run\" ] || [ \"$1\" = \"exec\" ]; then\n  case \"$*\" in\n    *\"npm\"*) echo 'not-a-version'; echo '{started}' >&2; echo '{path}/usr/local/bin/npm' >&2; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER
            )
        };
        write_fake_command(&bin_dir, "docker", &docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tools:
  npm: "*"
tasks:
  test:
    run: echo test
"#,
        )
        .unwrap();

        let report = super::diagnose_contract_in_mode(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Tool probe failed: npm")
            .unwrap_or_else(|| panic!("expected tool probe failure finding: {report:?}"));
        assert!(
            finding.why.contains(
                "ota probed `npm` inside container image `premium/test:latest` with `npm --version`"
            ),
            "{report:?}"
        );
        assert_eq!(finding.evidence().command, "npm --version");
        assert_eq!(finding.evidence().path, "npm");
        assert_eq!(finding.evidence().source, "container_target");
        assert_eq!(finding.owner(), "container_target");
    }

    #[test]
    fn skips_repo_local_presence_only_container_probe_failures_when_hydration_is_owned() {
        assert!(
            super::should_skip_presence_only_container_hydration_probe_failure(
                "bin/brakeman",
                true,
                Some(
                    "/usr/local/lib/ruby/3.3.0/bundler/definition.rb:599:in `materialize': Could not find brakeman-8.0.4 in locally installed gems (Bundler::GemNotFound)",
                ),
            )
        );
        assert!(
            !super::should_skip_presence_only_container_hydration_probe_failure(
                "brakeman",
                true,
                Some(
                    "/usr/local/lib/ruby/3.3.0/bundler/definition.rb:599:in `materialize': Could not find brakeman-8.0.4 in locally installed gems (Bundler::GemNotFound)",
                ),
            )
        );
        assert!(
            !super::should_skip_presence_only_container_hydration_probe_failure(
                "bin/brakeman",
                false,
                Some(
                    "/usr/local/lib/ruby/3.3.0/bundler/definition.rb:599:in `materialize': Could not find brakeman-8.0.4 in locally installed gems (Bundler::GemNotFound)",
                ),
            )
        );
    }

    #[test]
    fn reports_container_tool_probe_failed_when_command_cannot_run() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\necho daemon unavailable\r\nexit 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\necho daemon unavailable >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  backends:
    container:
      image: premium/test:latest
      engines: [docker]
tools:
  node: ">=20"
tasks:
  test:
    run: node --version
"#,
        )
        .unwrap();

        let report = super::diagnose_contract_in_mode(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Tool probe failed: node")
            .expect("expected tool probe failure finding");
        assert_eq!(finding.evidence().path, "node");
        assert_eq!(finding.owner(), "container_target");
    }

    #[test]
    fn reports_container_image_acquisition_failure_once() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\necho Unable to find image 'ruby:3.3.11-bookworm' locally 1>&2\r\necho docker: Error response from daemon: Get \"https://registry-1.docker.io/v2/\": context deadline exceeded 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\necho \"Unable to find image 'ruby:3.3.11-bookworm' locally\" >&2\necho 'docker: Error response from daemon: Get \"https://registry-1.docker.io/v2/\": context deadline exceeded' >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  backends:
    container:
      image: ruby:3.3.11-bookworm
      engines: [docker]
runtimes:
  ruby: "3.3.11"
tools:
  bundler: ">=2.5.3"
tasks:
  test:
    run: bundle exec rspec
"#,
        )
        .unwrap();

        let report = super::diagnose_contract_in_mode(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok);
        let image_findings = report
            .findings
            .iter()
            .filter(|finding| {
                finding.summary == "Container image unavailable: ruby:3.3.11-bookworm"
            })
            .count();
        assert_eq!(image_findings, 1);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Container image unavailable: ruby:3.3.11-bookworm")
            .expect("expected container image blocker");
        assert!(finding.why.contains("registry-1.docker.io"));
        assert!(finding.next.contains("docker pull ruby:3.3.11-bookworm"));
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Runtime probe failed: ruby")
        );
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Tool probe failed: bundler")
        );
    }

    #[test]
    fn reports_container_tool_probe_platform_mismatch_guidance_for_manifest_errors() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\necho Unable to find image 'node:22-bookworm' locally\r\necho docker: no matching manifest for windows(10.0.26100)/amd64 in the manifest list entries 1>&2\r\nexit /b 1\r\n"
        } else {
            "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\necho \"Unable to find image 'node:22-bookworm' locally\" >&2\necho \"docker: no matching manifest for windows(10.0.26100)/amd64 in the manifest list entries\" >&2\nexit 1\n"
        };
        write_fake_command(&bin_dir, "docker", docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  backends:
    container:
      image: node:22-bookworm
      engines: [docker]
tools:
  node: ">=22"
tasks:
  test:
    run: node --version
"#,
        )
        .unwrap();

        let report = super::diagnose_contract_in_mode(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Container image unavailable: node:22-bookworm")
            .expect("expected container image finding");
        assert!(finding.next.contains(
            "switch Docker Desktop to Linux container mode or use a Windows-compatible image tag"
        ));
    }

    #[test]
    fn reports_unsupported_remote_backend_provider() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: unknown
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        assert!(!report.ok);
        assert_eq!(
            report.findings[0].summary,
            "Unsupported remote execution backend provider: unknown"
        );
    }

    #[test]
    fn accepts_declared_backend_provider_remote_backend() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: ota-ext-backend
    api_version: 1
execution:
  preferred: remote
  backends:
    remote:
      provider: backend-demo
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        assert!(report.ok);
        assert!(report.findings.iter().all(|finding| {
            !finding
                .summary
                .contains("Unsupported remote execution backend provider")
        }));
    }

    #[test]
    fn warns_that_native_doctor_only_partially_evaluates_remote_contexts() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
extensions:
  backend-demo:
    kind: backend_provider
    command: ota-ext-backend
    api_version: 1
execution:
  default_context: remote-app
  contexts:
    remote-app:
      backend: remote
      remote:
        provider: backend-demo
        target: sandbox-dev
      requirements:
        tools:
          jq: "*"
tasks:
  test:
    context: remote-app
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary
                    == "Remote execution contexts are only partially evaluated in native mode"
            })
            .expect("expected remote doctor scope note");
        assert_eq!(finding.severity, FindingSeverity::Info);
        assert!(
            finding
                .why
                .contains("runtime and tool version checks still evaluate the local host")
        );
        assert!(finding.next.contains("ota doctor --mode remote"));
        assert!(finding.next.contains("ota execution plan --mode remote"));
        assert!(!finding.next.contains("dedicated remote doctor mode ships"));
    }

    #[test]
    fn container_mode_requirement_surface_includes_inherited_context_requirements() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app-base:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/dev:latest
      requirements:
        runtimes:
          node: "24"
        tools:
          pnpm: "10"
    app:
      extends: app-base
tasks:
  dev:
    context: app
    run: pnpm dev
"#,
        )
        .unwrap();

        let surface =
            super::precondition_requirement_surface(&contract, DoctorMode::Container, None);
        assert_eq!(
            surface
                .runtimes
                .get("node")
                .map(|requirement| requirement.version().to_string()),
            Some(String::from("24"))
        );
        assert_eq!(
            surface
                .tools
                .get("pnpm")
                .map(|requirement| requirement.version().to_string()),
            Some(String::from("10"))
        );
    }

    #[test]
    fn container_mode_precondition_surface_infers_mode_selected_run_command_tool() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: python:3.12-bookworm
tasks:
  setup:
    run: "true"
    execution:
      default_mode: native
      modes:
        container:
          context: app
          run: uv sync
"#,
        )
        .unwrap();

        let surface =
            super::precondition_requirement_surface(&contract, DoctorMode::Container, None);
        assert!(surface.presence_only_tools.contains("uv"));
    }

    #[test]
    fn container_mode_precondition_surface_projects_top_level_release_asset_tool_from_context() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  yq:
    version: "4.52.5"
    acquisition:
      provider: release_asset
      source_config:
        asset_by_platform:
          linux_x86_64: https://example.com/releases/v{version}/yq_linux_amd64
          linux_aarch64: https://example.com/releases/v{version}/yq_linux_arm64
          macos_x86_64: https://example.com/releases/v{version}/yq_darwin_amd64
          macos_aarch64: https://example.com/releases/v{version}/yq_darwin_arm64
          windows_x86_64: https://example.com/releases/v{version}/yq_windows_amd64.exe
          windows_aarch64: https://example.com/releases/v{version}/yq_windows_arm64.exe
execution:
  default_context: app
  contexts:
    app:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/dev:latest
      requirements:
        tools:
          yq: "4.52.5"
tasks:
  render:
    context: app
    command:
      exe: yq
      args:
        - --version
"#,
        )
        .unwrap();

        let surface =
            super::precondition_requirement_surface(&contract, DoctorMode::Container, None);
        let acquisition = surface
            .tools
            .get("yq")
            .and_then(|requirement| requirement.acquisition())
            .expect("yq acquisition");
        assert_eq!(acquisition.provider.as_str(), "release-asset");
    }

    #[test]
    fn selected_workflow_preconditions_do_not_fall_back_to_all_toolchains() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "^22.12.0"
execution:
  default_context: docker-host
  contexts:
    docker-host:
      backend: native
      requirements:
        tools:
          docker: "*"
tasks:
  setup:docker-env:
    context: docker-host
    action:
      kind: copy_if_missing
      from: docker/.env.example
      to: docker/.env
  dev:studio-docker:
    context: docker-host
    run: cd docker && docker compose up
    depends_on:
      - setup:docker-env
workflows:
  default: studio:docker
  studio:docker:
    prepare:
      task: setup:docker-env
    run:
      task: dev:studio-docker
"#,
        )
        .unwrap();

        let selection = super::scoped_precondition_selection(
            &contract,
            DoctorMode::Native,
            Some("studio:docker"),
        );

        assert!(selection.toolchain_names.is_empty());
        assert!(!selection.requirement_surface.runtimes.contains_key("node"));
        assert_eq!(
            selection
                .requirement_surface
                .tools
                .get("docker")
                .map(|requirement| requirement.version().to_string()),
            Some(String::from("*"))
        );
    }

    #[test]
    fn selected_workflow_preconditions_force_explicit_optional_tools_required() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  helm:
    version: ">=3.8"
    required: false
tasks:
  render:
    command:
      exe: helm
      args:
        - template
    requirements:
      tools:
        helm: ">=3.8"
workflows:
  default: chart
  chart:
    run:
      task: render
"#,
        )
        .unwrap();

        let surface =
            super::precondition_requirement_surface(&contract, DoctorMode::Native, Some("chart"));

        assert!(
            surface
                .tools
                .get("helm")
                .expect("helm requirement")
                .required_for_os(super::current_os())
        );
    }

    #[test]
    fn doctor_blocks_selected_workflow_on_unsupported_host_context() {
        let unsupported = if super::current_host_platform() == "windows" {
            "linux"
        } else {
            "windows"
        };
        let contract = parse_contract_str(
            synthetic_contract_path(),
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
      only_on:
        - {unsupported}
tasks:
  dev:
    run: echo hi
workflows:
  default: app
  app:
    run:
      task: dev
"#
            ),
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("app"),
        );

        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Unsupported host platform for context: host")
            .expect("expected unsupported host finding");
        assert_eq!(finding.code(), "OTA_CONTEXT_HOST_PLATFORM_UNSUPPORTED");
        assert_eq!(finding.category(), "execution");
        assert_eq!(finding.owner(), "host");
        assert!(finding.why.contains("execution.contexts.host"));
        assert!(finding.why.contains("only_on"));
        assert!(finding.next.contains("ota doctor --workflow app"));
    }

    #[test]
    fn doctor_blocks_selected_workflow_on_unsupported_host_arch_context() {
        let unsupported = if super::current_host_arch() == "arm64" {
            "x64"
        } else {
            "arm64"
        };
        let contract = parse_contract_str(
            synthetic_contract_path(),
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
      only_arch:
        - {unsupported}
tasks:
  dev:
    run: echo hi
workflows:
  default: app
  app:
    run:
      task: dev
"#
            ),
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("app"),
        );

        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Unsupported host platform for context: host")
            .expect("expected unsupported host finding");
        assert!(finding.why.contains("execution.contexts.host"));
        assert!(finding.why.contains("only_arch"));
        assert!(finding.why.contains(super::current_host_arch()));
    }

    #[test]
    fn remote_task_requirement_selection_follows_selected_workflow_contexts() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  pnpm:
    version: "10"
    acquisition:
      provider: corepack
      package: pnpm
      version: "10.22.0"
execution:
  contexts:
    contributor-remote:
      backend: remote
      remote:
        provider: ssh
        target: contributor.example
    instant-remote:
      backend: remote
      remote:
        provider: ssh
        target: instant.example
tasks:
  setup:
    context: contributor-remote
    run: pnpm install
    requirements:
      tools:
        pnpm: "10"
  contributor:
    context: contributor-remote
    run: pnpm dev
    depends_on:
      - setup
  instant:
    context: instant-remote
    run: npx ota-demo
    requirements:
      tools:
        npx: "*"
workflows:
  default: contributor
  contributor:
    run:
      task: contributor
  instant:
    run:
      task: instant
"#,
        )
        .unwrap();

        let selection =
            super::selected_remote_task_requirement_selection(&contract, Some("instant"))
                .expect("selected workflow should produce remote task requirements");

        assert_eq!(
            selection.by_context.keys().cloned().collect::<Vec<_>>(),
            vec![String::from("instant-remote")]
        );
        let instant_surface = selection
            .by_context
            .get("instant-remote")
            .expect("instant context should be selected");
        assert!(
            instant_surface
                .requirement_surface
                .tools
                .contains_key("npx")
        );
        assert!(
            !instant_surface
                .requirement_surface
                .tools
                .contains_key("pnpm")
        );
        assert!(selection.fallback.is_none());
    }

    #[test]
    fn doctor_surfaces_corepack_activation_for_selected_workflow_tools() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "node.cmd" } else { "node" },
            if cfg!(windows) {
                "@echo off\r\necho v24.0.0\r\n"
            } else {
                "#!/bin/sh\necho 'v24.0.0'\n"
            },
        );
        write_fake_command(
            &bin_dir,
            if cfg!(windows) {
                "corepack.cmd"
            } else {
                "corepack"
            },
            if cfg!(windows) {
                "@echo off\r\necho corepack 0.31.0\r\n"
            } else {
                "#!/bin/sh\necho 'corepack 0.31.0'\n"
            },
        );
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "24"
    fulfillment: run
    package_managers:
      pnpm: "10.22.0"
tasks:
  setup:
    run: pnpm install
    requirements:
      toolchains:
        - node
      tools:
        pnpm: "10.22.0"
  docker:run:
    launch:
      kind: container
      image: ghcr.io/example/app:latest
    requirements:
      tools:
        docker: "*"
workflows:
  default: contributor
  contributor:
    run:
      task: setup
  docker:
    run:
      task: docker:run
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("contributor"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report.findings.iter().any(|finding| {
                finding.summary.contains("pnpm")
                    && finding
                        .next
                        .contains("install pnpm and make it available on PATH")
            }),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.summary.contains("docker")),
            "{report:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn command_version_probe_in_working_dir_runs_cmd_wrappers() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "corepack.cmd",
            "@echo off\r\necho corepack 0.31.0\r\n",
        );

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let probe = super::command_version_probe_in_working_dir("corepack", fixture.path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert_eq!(
            probe.clone().version().as_deref(),
            Some("0.31.0"),
            "{probe:?}"
        );
    }

    #[cfg(windows)]
    #[test]
    fn version_probe_program_uses_resolved_cmd_wrapper_path() {
        let wrapper = Path::new(r"C:\toolcache\corepack.CMD");
        assert_eq!(
            super::version_probe_program("corepack", Some(wrapper)),
            wrapper.as_os_str()
        );
    }

    #[test]
    fn doctor_blocks_on_missing_corepack_for_selected_workflow_tools() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "node.cmd" } else { "node" },
            if cfg!(windows) {
                "@echo off\r\necho v24.0.0\r\n"
            } else {
                "#!/bin/sh\necho 'v24.0.0'\n"
            },
        );
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "24"
    package_managers:
      pnpmx: "10.22.0"
tasks:
  setup:
    run: pnpm install
    requirements:
      toolchains:
        - node
      tools:
        pnpmx: "10.22.0"
workflows:
  default: contributor
  contributor:
    setup:
      task: setup
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("contributor"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Missing tool activation provider: corepack")
            .expect("corepack blocker should be surfaced");
        assert!(finding.why.contains("pnpmx"));
        assert!(
            finding
                .next
                .contains("corepack prepare pnpmx@10.22.0 --activate")
        );
    }

    #[test]
    fn doctor_does_not_block_on_missing_corepack_when_npm_can_bootstrap_it() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "node.cmd" } else { "node" },
            if cfg!(windows) {
                "@echo off\r\necho v24.0.0\r\n"
            } else {
                "#!/bin/sh\necho 'v24.0.0'\n"
            },
        );
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "npm.cmd" } else { "npm" },
            if cfg!(windows) {
                "@echo off\r\necho 10.9.0\r\n"
            } else {
                "#!/bin/sh\necho '10.9.0'\n"
            },
        );
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "24"
    package_managers:
      pnpmx: "10.22.0"
tasks:
  setup:
    run: pnpm install
    requirements:
      toolchains:
        - node
      tools:
        pnpmx: "10.22.0"
workflows:
  default: contributor
  contributor:
    setup:
      task: setup
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("contributor"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            !report
                .findings
                .iter()
                .any(|finding| { finding.summary == "Missing tool activation provider: corepack" }),
            "{report:?}"
        );
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing tool: pnpmx"),
            "{report:?}"
        );
    }

    #[cfg(target_os = "macos")]
    #[test]
    fn doctor_emits_direct_tool_acquisition_provisioning_request() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "node.cmd" } else { "node" },
            if cfg!(windows) {
                "@echo off\r\necho v24.0.0\r\n"
            } else {
                "#!/bin/sh\necho 'v24.0.0'\n"
            },
        );
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  helm:
    version: ">=3.8"
    platforms:
      macos:
        acquisition:
          provider: brew
          package: helm
          source_config:
            tap_name: vendor/tap
            tap_url: https://github.com/vendor/homebrew-tap
tasks:
  render:
    run: helm template app ./chart
    requirements:
      tools:
        helm: ">=3.8"
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let provisioning = report
            .provisioning
            .as_ref()
            .expect("direct tool acquisition provisioning should be present");
        assert!(provisioning.request.actions.iter().any(|action| {
            action.kind == ProvisioningActionKind::Install
                && action.target_kind == ProvisioningTargetKind::Tool
                && action.name == "helm"
                && action.source == "brew"
                && action.package.as_deref() == Some("helm")
                && action
                    .source_config
                    .as_ref()
                    .and_then(|config| config.get("tap_name"))
                    .and_then(|value| value.as_str())
                    == Some("vendor/tap")
        }));
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.why.contains("helm"))
        );
    }

    #[test]
    fn doctor_emits_direct_release_asset_tool_acquisition_provisioning_request() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  yq:
    version: "4.52.5"
    acquisition:
      provider: release_asset
      source_config:
        asset_by_platform:
          linux_x86_64: https://example.com/releases/v{version}/yq_linux_amd64
          linux_aarch64: https://example.com/releases/v{version}/yq_linux_arm64
          macos_x86_64: https://example.com/releases/v{version}/yq_darwin_amd64
          macos_aarch64: https://example.com/releases/v{version}/yq_darwin_arm64
          windows_x86_64: https://example.com/releases/v{version}/yq_windows_amd64.exe
        version_args:
          - --version
tasks:
  render:
    run: yq --version
    requirements:
      tools:
        yq: "4.52.5"
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let provisioning = report
            .provisioning
            .as_ref()
            .expect("direct release-asset tool acquisition provisioning should be present");
        assert!(provisioning.request.actions.iter().any(|action| {
            action.kind == ProvisioningActionKind::SelectSource
                && action.target_kind == ProvisioningTargetKind::Tool
                && action.name == "yq"
                && action.source == "release-asset"
                && action.requested_version == "4.52.5"
                && action.package.is_none()
                && action
                    .source_config
                    .as_ref()
                    .and_then(|config| config.get("asset_by_platform"))
                    .is_some()
        }));
    }

    #[test]
    fn doctor_does_not_false_flag_release_asset_activation_provider_when_curl_is_present() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "curl.cmd" } else { "curl" },
            if cfg!(windows) {
                "@echo off\r\nexit /b 0\r\n"
            } else {
                "#!/bin/sh\nexit 0\n"
            },
        );

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  yq:
    version: "4.52.5"
    acquisition:
      provider: release_asset
      source_config:
        asset_by_platform:
          linux_x86_64: https://example.com/releases/v{version}/yq_linux_amd64
          linux_aarch64: https://example.com/releases/v{version}/yq_linux_arm64
          macos_x86_64: https://example.com/releases/v{version}/yq_darwin_amd64
          macos_aarch64: https://example.com/releases/v{version}/yq_darwin_arm64
          windows_x86_64: https://example.com/releases/v{version}/yq_windows_amd64.exe
        version_args:
          - --version
tasks:
  render:
    run: yq --version
    requirements:
      tools:
        yq: "4.52.5"
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.code() == "OTA_TOOL_ACTIVATION_PROVIDER_MISSING"),
            "{report:?}"
        );
    }

    #[test]
    fn doctor_direct_release_asset_actions_use_declared_contract_version_when_task_requires_any() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  vale:
    version: "3.15.1"
    required: false
    acquisition:
      provider: release_asset
      source_config:
        asset_by_platform:
          linux_x86_64: https://example.com/releases/v{version}/vale_linux_amd64
tasks:
  verify:
    command:
      exe: vale
      args:
        - --version
    requirements:
      tools:
        vale: "*"
"#,
        )
        .unwrap();

        let task = contract.tasks.get("verify").expect("verify task");
        let requirement_surface =
            task.scoped_requirement_surface_for_execution(crate::schema::Backend::Native, None);
        let actions = super::merged_provisioning_actions_for_requirement_surface(
            &contract,
            Vec::new(),
            &requirement_surface,
            "linux",
        );
        assert!(actions.iter().any(|action| {
            action.kind == ProvisioningActionKind::SelectSource
                && action.target_kind == ProvisioningTargetKind::Tool
                && action.name == "vale"
                && action.source == "release-asset"
                && action.requested_version == "3.15.1"
        }));
    }

    #[test]
    fn container_mode_does_not_report_release_asset_tool_missing_when_selected_path_can_provision_it()
     {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            if cfg!(windows) {
                "docker.cmd"
            } else {
                "docker"
            },
            if cfg!(windows) {
                r#"@echo off
if "%1"=="info" exit /b 0
if "%1"=="run" (
  echo %* | findstr /C:"command -v 'task'" >nul && exit /b 1
  echo %* | findstr /C:"curl -fsSL -o" >nul && (
    mkdir .\.ota\state\source-managed\bin >nul 2>&1
    > .\.ota\state\source-managed\bin\task.cmd echo @echo off
    >> .\.ota\state\source-managed\bin\task.cmd echo if "%%1"=="--version" echo 3.51.1
    exit /b 0
  )
  echo %* | findstr /C:".ota/state/source-managed/bin/task" >nul && (
    echo 3.51.1
    exit /b 0
  )
)
exit /b 1
"#
            } else {
                r#"#!/bin/sh
if [ "$1" = "info" ]; then
  exit 0
fi
if [ "$1" = "run" ]; then
  args="$*"
  case "$args" in
    *"command -v 'task'"*)
      exit 1
      ;;
    *"curl -fsSL -o"*)
      /bin/mkdir -p ./.ota/state/source-managed/bin
      cat > ./.ota/state/source-managed/bin/task <<'EOF'
#!/bin/sh
if [ "$1" = "--version" ]; then
  echo '3.51.1'
  exit 0
fi
exit 1
EOF
      /bin/chmod +x ./.ota/state/source-managed/bin/task
      exit 0
      ;;
    *".ota/state/source-managed/bin/task"*)
      echo '3.51.1'
      exit 0
      ;;
  esac
fi
exit 1
"#
            },
        );

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract_path = fixture.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
tools:
  task:
    version: "3.51.1"
    acquisition:
      provider: release_asset
      source_config:
        asset_by_platform:
          linux_x86_64:
            url: https://example.com/releases/v{version}/task_linux_amd64.tar.gz
            archive:
              format: tar_gz
              executable_path: task
          linux_aarch64:
            url: https://example.com/releases/v{version}/task_linux_arm64.tar.gz
            archive:
              format: tar_gz
              executable_path: task
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: premium/test:latest
tasks:
  verify:
    command:
      exe: task
      args:
        - --version
    requirements:
      tools:
        task: "3.51.1"
"#,
        )
        .unwrap();

        let report =
            diagnose_preconditions_with_mode(&contract, &contract_path, DoctorMode::Container);

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let provisioning = report
            .provisioning
            .as_ref()
            .expect("container release-asset provisioning should be present");
        assert!(provisioning.request.actions.iter().any(|action| {
            action.kind == ProvisioningActionKind::SelectSource
                && action.target_kind == ProvisioningTargetKind::Tool
                && action.name == "task"
                && action.source == "release-asset"
        }));
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.code() != "OTA_TOOL_MISSING"),
            "unexpected missing-tool findings: {:?}",
            report.findings
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn native_mode_recognizes_repo_owned_source_managed_release_asset_tools() {
        let _env_guard = env_mutex_lock();
        let _cwd_guard = cwd_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let original_dir = env::current_dir().unwrap();
        let original_path = env::var_os("PATH");
        let managed_bin = fixture.path().join(".ota/state/source-managed/bin");
        fs::create_dir_all(&managed_bin).unwrap();
        write_fake_command(
            &managed_bin,
            "vale",
            "#!/bin/sh\nif [ \"$1\" = \"--version\" ]; then\n  echo 'vale version 3.15.1'\n  exit 0\nfi\nexit 1\n",
        );
        unsafe {
            env::set_current_dir(fixture.path()).unwrap();
            env::set_var("PATH", fixture.path().join("missing-bin"));
        }

        let contract_path = fixture.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: backstage
tools:
  vale:
    version: "3.15.1"
    required: false
    acquisition:
      provider: release_asset
      source_config:
        asset_by_platform:
          linux_x86_64: https://example.com/releases/v{version}/vale_linux_amd64
tasks:
  verify:
    command:
      exe: vale
      args:
        - --version
    requirements:
      tools:
        vale: "*"
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            &contract_path,
            DoctorMode::Native,
            Some("verify"),
        );

        unsafe {
            env::set_current_dir(original_dir).unwrap();
            match original_path {
                Some(path) => env::set_var("PATH", path),
                None => env::remove_var("PATH"),
            }
        }

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.code() != "OTA_TOOL_MISSING"),
            "unexpected missing-tool findings: {:?}",
            report.findings
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.code() != "OTA_TOOL_VERSION_MISMATCH"),
            "unexpected version findings: {:?}",
            report.findings
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.code() != "OTA_TOOL_PROBE_FAILED"),
            "unexpected probe-failed findings: {:?}",
            report.findings
        );
    }

    #[test]
    fn doctor_selected_workflow_does_not_surface_unrequired_corepack_package_manager_findings() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "node.cmd" } else { "node" },
            if cfg!(windows) {
                "@echo off\r\necho v24.0.0\r\n"
            } else {
                "#!/bin/sh\necho 'v24.0.0'\n"
            },
        );
        write_fake_command(
            &bin_dir,
            if cfg!(windows) {
                "corepack.cmd"
            } else {
                "corepack"
            },
            if cfg!(windows) {
                "@echo off\r\necho corepack 0.31.0\r\n"
            } else {
                "#!/bin/sh\necho 'corepack 0.31.0'\n"
            },
        );
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "npx.cmd" } else { "npx" },
            if cfg!(windows) {
                "@echo off\r\necho 10.22.0\r\n"
            } else {
                "#!/bin/sh\necho '10.22.0'\n"
            },
        );
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "24"
    package_managers:
      pnpmx: "10.22.0"
tools:
  npx: "*"
tasks:
  setup:
    run: pnpmx install
    requirements:
      toolchains:
        - node
      tools:
        pnpmx: "10.22.0"
  quickstart:
    run: npx --yes n8n
    requirements:
      toolchains:
        - node
      tools:
        npx: "*"
workflows:
  default: backend
  backend:
    setup:
      task: setup
  instant:
    run:
      task: quickstart
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("instant"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report.findings.iter().all(|finding| {
                !finding.summary.contains("pnpmx")
                    && !finding.why.contains("pnpmx")
                    && !finding.next.contains("pnpmx")
            }),
            "{report:?}"
        );
    }

    #[test]
    fn doctor_selected_task_does_not_surface_unrelated_optional_repo_tool_findings() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            if cfg!(windows) { "node.cmd" } else { "node" },
            if cfg!(windows) {
                "@echo off\r\necho v24.0.0\r\n"
            } else {
                "#!/bin/sh\necho 'v24.0.0'\n"
            },
        );
        write_fake_command(
            &bin_dir,
            if cfg!(windows) {
                "corepack.cmd"
            } else {
                "corepack"
            },
            if cfg!(windows) {
                "@echo off\r\necho corepack 0.31.0\r\n"
            } else {
                "#!/bin/sh\necho 'corepack 0.31.0'\n"
            },
        );

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: langfuse
tools:
  clickhouse:
    version: "*"
    required: false
toolchains:
  node:
    version: "24"
    package_managers:
      pnpm: "11.1.3"
tasks:
  playwright:browsers:
    prepare:
      kind: tool_bootstrap
      tool: playwright_browsers
      browsers:
        - chromium
      source:
        kind: node_package_manager
        cwd: .
        manager: pnpm
        filter: web
    requirements:
      toolchains:
        - node
    effects:
      network: true
      network_kind: tool_bootstrap
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_task_with_overrides(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            "playwright:browsers",
            ExecutionOverrides::default(),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report.findings.iter().all(|finding| {
                !(finding.code() == "OTA_TOOL_MISSING" && finding.summary.contains("clickhouse"))
            }),
            "unexpected unrelated optional tool finding: {:?}",
            report.findings
        );
    }

    #[test]
    fn doctor_container_corepack_owned_tool_probe_uses_repo_workdir() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let repo_dir = fixture.path().join("repo");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::write(
            repo_dir.join("package.json"),
            r#"{"name":"twenty","packageManager":"yarn@4.13.0"}"#,
        )
        .unwrap();

        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            format!(
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"command -v 'node'\" >nul && (\r\n    echo v24.15.0\r\n    echo {started} 1>&2\r\n    echo {path}/usr/local/bin/node 1>&2\r\n    exit /b 0\r\n  )\r\n  echo %* | findstr /C:\"command -v 'yarn'\" >nul && (\r\n    echo {started} 1>&2\r\n    echo {path}/usr/local/bin/yarn 1>&2\r\n    echo %* | findstr /C:\"/workspace\" >nul && (\r\n      echo 4.13.0\r\n      exit /b 0\r\n    )\r\n    echo 1.22.22\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER,
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"command -v 'node'\"*) echo 'v24.15.0'; echo '{started}' >&2; echo '{path}/usr/local/bin/node' >&2; exit 0 ;;\n    *\"command -v 'yarn'\"*) echo '{started}' >&2; echo '{path}/usr/local/bin/yarn' >&2; case \"$*\" in *\"/workspace\"*) echo '4.13.0'; exit 0 ;; *) echo '1.22.22'; exit 0 ;; esac ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER,
            )
        };
        write_fake_command(&bin_dir, "docker", &docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract_path = repo_dir.join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: twenty
execution:
  preferred: container
  backends:
    container:
      image: node:24-bookworm
      engines: [docker]
toolchains:
  node:
    provider: corepack
    version: "^24.5.0"
    package_managers:
      yarn: "4.13.0"
tasks:
  install:
    run: yarn --immutable
    execution:
      default_mode: container
    requirements:
      toolchains:
        - node
workflows:
  default: verify
  verify:
    setup:
      task: install
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            &contract_path,
            DoctorMode::Container,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Version mismatch for tool: yarn"),
            "{report:?}"
        );
    }

    #[test]
    fn doctor_selected_workflow_infers_launch_command_tool_requirement() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  quickstart:
    launch:
      kind: command
      exe: npx
      args: [--yes, n8n]
workflows:
  default: instant
  instant:
    run:
      task: quickstart
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("instant"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary.contains("Missing tool: npx")),
            "{report:?}"
        );
    }

    #[test]
    fn doctor_blocks_on_missing_command_acquisition_shell_for_selected_workflow_tools() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", env::join_paths([bin_dir.as_path()]).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  bunx:
    version: ">=1.2.0"
    acquisition:
      provider: command
      shell: sh
      run: curl -fsSL https://bun.sh/install | sh
tasks:
  setup:
    run: bun install
    requirements:
      tools:
        bunx: ">=1.2.0"
workflows:
  default: contributor
  contributor:
    setup:
      task: setup
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("contributor"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Missing tool activation provider: sh")
            .expect("command acquisition shell blocker should be surfaced");
        assert!(finding.why.contains("bunx"));
        assert!(
            finding
                .next
                .contains("sh -lc 'curl -fsSL https://bun.sh/install | sh'")
        );
    }

    #[test]
    fn container_mode_uses_inherited_named_context_backend_configuration() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
    app-base:
      backend: container
      lifecycle: ephemeral
      container:
        image: ghcr.io/ota/dev:latest
        engines:
          - missing-engine
    app:
      extends: app-base
tasks:
  dev:
    context: app
    run: pnpm dev
"#,
        )
        .unwrap();

        let report =
            diagnose_contract_in_mode(&contract, synthetic_contract_path(), DoctorMode::Container);

        assert!(report.findings.iter().any(|finding| {
            finding.summary == "Missing container execution backend CLI: missing-engine"
        }));
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| { finding.summary == "Container execution is not configured" })
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn remote_doctor_mode_emits_policy_surfaces_per_remote_context() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_remote_backend_provider(&bin_dir);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }
        let provider_command = if cfg!(windows) {
            bin_dir.join("fake-remote-provider.cmd")
        } else {
            bin_dir.join("fake-remote-provider")
        };
        let provider_command = provider_command.display().to_string();

        fs::write(
            fixture.path().join("ota.yaml"),
            format!(
                r#"
version: 1
project:
  name: ota
extensions:
  remote-fixture:
    kind: backend_provider
    command: "{provider_command}"
    api_version: 1
execution:
  default_context: remote-app
  contexts:
    remote-app:
      backend: remote
      remote:
        provider: remote-fixture
        target: sandbox-dev
      requirements:
        tools:
          jq: "jq-1.8.1"
tasks:
  test:
    context: remote-app
    run: cargo test
"#,
            ),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
  version_policy:
    tools:
      jq:
        approved_versions:
          - "jq-1.8.1"
  provisioning:
    jq:
      source: apt
      package: jq
      approved_versions:
        - "jq-1.8.1"
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            &fixture.path().join("ota.yaml"),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_contract_in_mode(
            &contract,
            &fixture.path().join("ota.yaml"),
            DoctorMode::Remote,
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let version_finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Policy-backed version rules are declared")
            .unwrap_or_else(|| panic!("expected remote policy version finding: {report:?}"));
        assert!(
            version_finding
                .why
                .contains("for remote context `remote-app`")
        );
        assert!(version_finding.why.contains("tool jq (versions jq-1.8.1)"));

        let provisioning_finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Policy-backed provisioning sources are declared")
            .expect("expected remote policy provisioning finding");
        assert!(
            provisioning_finding
                .why
                .contains("for remote context `remote-app`")
        );
        assert!(
            provisioning_finding
                .why
                .contains("tool jq jq-1.8.1 via apt")
        );
        assert!(!report.findings.iter().any(|finding| {
            finding.summary == "Remote doctor mode still has partial policy reporting"
        }));
        assert!(!report.findings.iter().any(|finding| {
            finding.summary
                == "Remote execution contexts are only partially evaluated in native mode"
        }));
    }

    #[test]
    fn remote_doctor_mode_requires_remote_execution_configuration() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report =
            diagnose_contract_in_mode(&contract, synthetic_contract_path(), DoctorMode::Remote);

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Remote execution is not configured")
            .expect("expected remote configuration blocker");
        assert_eq!(finding.code(), "OTA_REMOTE_MODE_NOT_CONFIGURED");
        assert_eq!(finding.category(), "remote");
        assert_eq!(finding.owner(), "repo_contract");
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert!(finding.next.contains("ota doctor"));
        assert!(finding.next.contains("--mode remote"));
    }

    #[cfg(unix)]
    #[test]
    fn remote_doctor_mode_probes_tool_versions_through_remote_contexts() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "jq", "#!/bin/sh\necho 'jq-1.7.0'\n");
        write_fake_command(&bin_dir, "ssh", &fake_remote_ssh_body("jq-1.8.1"));

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: remote-app
  contexts:
    remote-app:
      backend: remote
      remote:
        provider: ssh
        target: user@host
      requirements:
        tools:
          jq: "jq-1.8.1"
tasks:
  test:
    context: remote-app
    run: cargo test
"#,
        )
        .unwrap();

        let report =
            diagnose_contract_in_mode(&contract, synthetic_contract_path(), DoctorMode::Remote);

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Version mismatch for tool: jq (context remote-app)")
            .expect("expected remote version mismatch");
        assert!(finding.why.contains("through remote context `remote-app`"));
        assert_eq!(finding.owner(), "remote_target");
        assert_eq!(finding.evidence().source, "remote_target");
        assert_eq!(finding.evidence().command, "jq --version");
        assert!(!finding.evidence().path.is_empty());
        assert!(finding.evidence().path.ends_with("/jq"));
    }

    #[cfg(unix)]
    #[test]
    fn remote_doctor_mode_reports_policy_backed_provisioning_failures_from_remote_contexts() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "uname", "#!/bin/sh\necho 'Linux'\n");
        write_fake_command(
            &bin_dir,
            "apt-get",
            r#"#!/bin/sh
case " $* " in
  *" update "*) exit 0 ;;
  *) echo "E: Unable to locate package nodejs" >&2; exit 100 ;;
esac
"#,
        );
        write_fake_command(&bin_dir, "ssh", &fake_remote_ssh_body("jq-1.8.1"));

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: remote-app
  contexts:
    remote-app:
      backend: remote
      remote:
        provider: ssh
        target: user@host
      requirements:
        runtimes:
          node:
            version: "24.14.1"
tasks:
  test:
    context: remote-app
    run: cargo test
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
  provisioning:
    node:
      source: apt
      package: nodejs
      approved_versions:
        - "24.14.1"
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            &fixture.path().join("ota.yaml"),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_contract_in_mode(
            &contract,
            &fixture.path().join("ota.yaml"),
            DoctorMode::Remote,
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary
                    == "Remote apt cannot locate required package: node (context remote-app)"
            })
            .expect("expected remote provisioning blocker");
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert!(
            finding
                .why
                .contains("remote context `remote-app` requests `node`")
        );
        assert_eq!(finding.owner(), "remote_target");
        assert_eq!(finding.provenance().as_deref(), Some("org policy"));
        assert_eq!(finding.provenance_key().as_deref(), Some("org_policy"));
        assert_eq!(finding.evidence().source, "remote_provisioning");
    }

    #[cfg(not(windows))]
    #[test]
    fn remote_doctor_mode_reports_version_policy_violations_per_context() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_remote_backend_provider(&bin_dir);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }
        let provider_command = if cfg!(windows) {
            bin_dir.join("fake-remote-provider.cmd")
        } else {
            bin_dir.join("fake-remote-provider")
        };
        let provider_command = provider_command.display().to_string();

        fs::write(
            fixture.path().join("ota.yaml"),
            format!(
                r#"
version: 1
project:
  name: ota
extensions:
  remote-fixture:
    kind: backend_provider
    command: "{provider_command}"
    api_version: 1
execution:
  default_context: remote-app
  contexts:
    remote-app:
      backend: remote
      remote:
        provider: remote-fixture
        target: sandbox-dev
      requirements:
        tools:
          jq: "jq-1.8.1"
tasks:
  test:
    context: remote-app
    run: cargo test
"#,
            ),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
  version_policy:
    tools:
      jq:
        approved_versions:
          - "jq-1.7.0"
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            &fixture.path().join("ota.yaml"),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_contract_in_mode(
            &contract,
            &fixture.path().join("ota.yaml"),
            DoctorMode::Remote,
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Repo does not satisfy org policy pack")
            .unwrap_or_else(|| panic!("expected remote version policy blocker: {report:?}"));
        assert!(finding.why.contains("remote context `remote-app`"));
        assert!(finding.why.contains("version policy violations"));
        assert!(
            finding
                .next
                .contains("update the requirements for remote context `remote-app`")
        );
    }

    #[test]
    fn remote_doctor_mode_reports_unexecutable_non_default_remote_contexts() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
    remote-bad:
      backend: remote
      remote:
        provider: ssh
      requirements:
        tools:
          jq: "*"
tasks:
  test:
    context: remote-bad
    run: cargo test
workflows:
  default: remote
  remote:
    run:
      task: test
"#,
        )
        .unwrap();

        let report =
            diagnose_contract_in_mode(&contract, synthetic_contract_path(), DoctorMode::Remote);

        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Remote execution context is not executable: remote-bad"
                    || finding.summary == "Invalid contract"
                    || finding.summary == "Missing execution backend CLI: ssh"
            })
            .unwrap_or_else(|| panic!("expected remote context blocker: {report:?}"));
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert!(
            finding.next.contains("execution.contexts.remote-bad")
                || finding
                    .why
                    .contains("execution.contexts.remote-bad.remote.target")
                || finding
                    .why
                    .contains("remote execution backend provider `ssh`"),
            "{finding:?}"
        );
    }

    #[test]
    fn warns_for_suspicious_ssh_remote_target_shape() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: ssh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Suspicious remote target for ssh: sandbox-dev")
        );
    }

    #[test]
    fn warns_for_suspicious_tsh_remote_target_shape() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: tsh
      target: sandbox-dev
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Suspicious remote target for tsh: sandbox-dev")
        );
    }

    #[test]
    fn warns_for_suspicious_kubectl_remote_target_shape() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: remote
  backends:
    remote:
      provider: kubectl
      target: ota-dev
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Suspicious remote target for kubectl: ota-dev")
        );
    }

    #[test]
    fn reports_allowed_env_value_mismatches() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_DOCTOR_ALLOWED:
      required: false
      default: prod
      allowed:
        - development
        - test
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.code() == "OTA_ENV_INVALID"
                    && finding.severity == FindingSeverity::Error)
        );
    }

    #[test]
    fn reports_missing_required_env_sources() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        let contents = r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_REQUIRED:
      required: true
  sources:
    - kind: dotenv
      path: .env
      must_exist: true
tasks:
  test:
    run: cargo test
"#;
        let contract = parse_contract_str(&contract_path, contents.trim_start()).unwrap();

        let report = diagnose_contract(&contract, &contract_path);

        assert!(
            report.findings.iter().any(
                |finding| finding.summary == "Missing required environment source: dotenv:.env"
            )
        );
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Missing required environment source: dotenv:.env")
            .expect("missing required env source should be present");
        assert_eq!(finding.code(), "OTA_ENV_SOURCE_MISSING_REQUIRED");
        assert_eq!(finding.category(), "environment");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn container_mode_still_reports_missing_required_env_sources() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        let contents = r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: node:22-bookworm-slim
env:
  sources:
    - kind: dotenv
      path: .env
      must_exist: true
tasks:
  test:
    run: cargo test
"#;
        let contract = parse_contract_str(&contract_path, contents.trim_start()).unwrap();

        let report = diagnose_contract_in_mode(&contract, &contract_path, DoctorMode::Container);

        assert!(
            report.findings.iter().any(
                |finding| finding.summary == "Missing required environment source: dotenv:.env"
            )
        );
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Missing required environment source: dotenv:.env")
            .expect("missing required env source should be present");
        assert_eq!(finding.code(), "OTA_ENV_SOURCE_MISSING_REQUIRED");
        assert_eq!(finding.category(), "environment");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn reports_env_source_parse_failures() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        let contents = r#"
version: 1
project:
  name: ota
env:
  sources:
    - kind: properties
      path: app.properties
tasks:
  test:
    run: cargo test
"#;
        fs::write(fixture.path().join("app.properties"), "bad=\\u12G4\n").unwrap();
        let contract = parse_contract_str(&contract_path, contents.trim_start()).unwrap();

        let report = diagnose_contract(&contract, &contract_path);

        assert!(report.findings.iter().any(|finding| {
            finding.summary == "Environment source parse failed: properties:app.properties"
                && finding
                    .why
                    .contains("ota could not read declared source `properties:app.properties`")
        }));
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Environment source parse failed: properties:app.properties"
            })
            .expect("env source parse failure should be present");
        assert_eq!(finding.code(), "OTA_ENV_SOURCE_PARSE_FAILED");
        assert_eq!(finding.category(), "environment");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn reports_env_source_invalid_structure_failures() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        let contents = r#"
version: 1
project:
  name: ota
env:
  sources:
    - kind: json
      path: env.json
tasks:
  test:
    run: cargo test
"#;
        fs::write(
            fixture.path().join("env.json"),
            r#"{"app":{"ports":[8080]}}"#,
        )
        .unwrap();
        let contract = parse_contract_str(&contract_path, contents.trim_start()).unwrap();

        let report = diagnose_contract(&contract, &contract_path);

        assert!(report.findings.iter().any(|finding| {
            finding.summary == "Environment source has invalid structure: json:env.json"
                && finding.why.contains("arrays are not supported")
                && finding.next.contains("scalar env-shaped values only")
        }));
    }

    #[test]
    fn reports_yaml_env_source_invalid_structure_failures() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        let contents = r#"
version: 1
project:
  name: ota
env:
  sources:
    - kind: yaml
      path: env.yaml
tasks:
  test:
    run: cargo test
"#;
        fs::write(
            fixture.path().join("env.yaml"),
            "app:\n  ports:\n    - 8080\n",
        )
        .unwrap();
        let contract = parse_contract_str(&contract_path, contents.trim_start()).unwrap();

        let report = diagnose_contract(&contract, &contract_path);

        assert!(report.findings.iter().any(|finding| {
            finding.summary == "Environment source has invalid structure: yaml:env.yaml"
                && finding.why.contains("arrays are not supported")
                && finding.next.contains("scalar env-shaped values only")
        }));
    }

    #[test]
    fn reports_env_source_key_collisions() {
        let fixture = TempDir::new().unwrap();
        let contract_path = fixture.path().join("ota.yaml");
        let contents = r#"
version: 1
project:
  name: ota
env:
  sources:
    - kind: properties
      path: app.properties
tasks:
  test:
    run: cargo test
"#;
        fs::write(
            fixture.path().join("app.properties"),
            "app.port=8080\napp-port=8081\n",
        )
        .unwrap();
        let contract = parse_contract_str(&contract_path, contents.trim_start()).unwrap();

        let report = diagnose_contract(&contract, &contract_path);

        assert!(report.findings.iter().any(|finding| {
            finding.summary == "Environment source key collision: properties:app.properties"
                && finding.why.contains("normalize to the same env name")
        }));
    }

    #[test]
    fn precondition_mode_skips_health_checks() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: wait-for-db
    kind: health
    severity: error
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, synthetic_contract_path());
        assert!(report.ok);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn checks_only_mode_skips_env_runtime_and_tool_diagnosis() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_REQUIRED:
      required: true
tools:
  ota-tool-that-does-not-exist:
    version: "*"
checks:
  - name: health-check
    kind: health
    severity: warn
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].summary, "Check failed: health-check");
    }

    #[test]
    fn probe_backed_check_succeeds() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:{port}/healthz/readiness
      # Windows CI can take longer to establish local-loopback probe
      # handshakes under Git Bash/runner scheduling; keep above jitter.
      timeout: 30000
checks:
  - name: backend-ready
    kind: health
    severity: error
    probe: backend-ready
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn probe_backed_check_failure_is_reported_as_probe() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:{port}/healthz/readiness
      # Windows CI can take longer to establish local-loopback probe
      # handshakes under Git Bash/runner scheduling; keep above jitter.
      timeout: 30000
checks:
  - name: backend-ready
    kind: health
    severity: error
    probe: backend-ready
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        server.join().expect("probe server should finish");
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].summary,
            "Probe check failed: backend-ready"
        );
        assert_eq!(report.findings[0].code(), "OTA_CHECK_FAILED");
        assert_eq!(report.findings[0].category(), "execution");
        assert_eq!(report.findings[0].owner(), "repo_contract");
        assert!(report.findings[0].why.contains("probe-backed check"));
        assert!(report.findings[0].why.contains("did not succeed"));
    }

    #[test]
    fn diagnose_checks_points_missing_file_precondition_to_setup_when_available() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: env-local-present
    kind: precondition
    severity: error
    run: test -f .env.local
tasks:
  setup:
    run: test -f .env.local || cp .env.example .env.local
    requirements:
      checks:
        - env-local-present
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].summary,
            "Check failed: env-local-present"
        );
        assert_eq!(
            report.findings[0].next,
            "run `ota up` or `ota run setup` to create `.env.local`, then rerun `ota doctor`"
        );
    }

    #[test]
    fn file_checks_use_repo_filesystem_without_shelling_out() {
        let dir = TempDir::new().unwrap();
        fs::create_dir_all(dir.path().join("node_modules")).unwrap();
        let contract_path = dir.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
checks:
  - name: workspace-dependencies-installed
    kind: file
    severity: error
    path: node_modules
    expect: directory
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, &contract_path);
        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn workspace_scoped_file_checks_use_workspace_relative_paths_without_shelling_out() {
        let root = TempDir::new().unwrap();
        let repo_dir = root.path().join("java-sdk");
        let sibling_dir = root.path().join("task-sdk");
        fs::create_dir_all(&repo_dir).unwrap();
        fs::create_dir_all(&sibling_dir).unwrap();
        fs::write(sibling_dir.join("schema.json"), "{}\n").unwrap();
        let contract_path = repo_dir.join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
checks:
  - name: shared-schema-present
    kind: file
    severity: error
    scope: workspace
    path: ../task-sdk/schema.json
    expect: file
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, &contract_path);
        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn file_checks_report_missing_expected_files() {
        let dir = TempDir::new().unwrap();
        let contract_path = dir.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
checks:
  - name: workspace-dependencies-installed
    kind: file
    severity: error
    path: node_modules
    expect: directory
tasks:
  setup:
    run: pnpm install
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, &contract_path);
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].summary,
            "File check failed: workspace-dependencies-installed"
        );
        assert!(
            report.findings[0]
                .why
                .contains("expected `node_modules` to be a directory"),
            "{report:?}"
        );
        assert_eq!(
            report.findings[0].next,
            "run `ota up` or `ota run setup` to satisfy `node_modules`, then rerun `ota doctor`"
        );
    }

    #[test]
    fn workflow_scoped_checks_do_not_run_unselected_global_checks() {
        let dir = TempDir::new().unwrap();
        let contract_path = dir.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
checks:
  - name: workspace-dependencies-installed
    kind: file
    severity: error
    path: node_modules
    expect: directory
tasks:
  quickstart:
    run: npx --yes n8n
    requirements:
      tools:
        node: "20"
workflows:
  default: instant
  instant:
    run:
      task: quickstart
"#,
        )
        .unwrap();

        let report =
            super::diagnose_checks_only_for_workflow(&contract, &contract_path, Some("instant"));

        assert!(report.ok, "{report:?}");
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary
                    != "File check failed: workspace-dependencies-installed"),
            "{report:?}"
        );
    }

    #[test]
    fn workflow_signal_checks_are_non_gating() {
        let dir = TempDir::new().unwrap();
        let contract_path = dir.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
checks:
  - name: workspace-dependencies-installed
    kind: file
    severity: error
    path: node_modules
    expect: directory
  - name: runtime-signal
    kind: file
    severity: error
    path: runtime-ready.flag
    expect: file
tasks:
  quickstart:
    run: npx --yes n8n
workflows:
  default: instant
  instant:
    run:
      task: quickstart
    readiness:
      signal:
        checks:
          - runtime-signal
"#,
        )
        .unwrap();

        let report =
            super::diagnose_checks_only_for_workflow(&contract, &contract_path, Some("instant"));

        assert!(report.ok, "{report:?}");
        assert!(report.findings.iter().any(|finding| {
            finding.summary == "File check failed: runtime-signal"
                && finding.severity == FindingSeverity::Info
        }));
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary
                    != "File check failed: workspace-dependencies-installed"),
            "{report:?}"
        );
    }

    #[test]
    fn workflow_signal_probes_are_non_gating() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:{port}/healthz/readiness
      # Windows CI can take longer to establish local-loopback probe
      # handshakes under Git Bash/runner scheduling; keep above jitter.
      timeout: 30000
workflows:
  default: backend
  backend:
    readiness:
      signal:
        probes:
          - backend-ready
"#,
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Signal probe failed: backend-ready")
            .expect("signal probe failure should be present");
        assert_eq!(finding.severity, FindingSeverity::Info);
        assert_eq!(finding.code(), "OTA_WORKFLOW_SIGNAL_PROBE_FAILED");
        assert_eq!(finding.category(), "execution");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn workflow_readiness_probes_are_executed() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:{port}/healthz/readiness
      timeout: 1000
workflows:
  default: backend
  backend:
    readiness:
      probes:
        - backend-ready
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn workflow_readiness_surfaces_are_executed() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: {port}
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 1000
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn workflow_blocking_check_skips_surface_readiness_evaluation() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: workflow-surface-gate
execution:
  default_context: host
  contexts:
    host:
      backend: native
surfaces:
  web:
    kind: http
    port: 65530
checks:
  - name: env-ready
    kind: file
    severity: error
    path: .env
    expect: file
tasks:
  dev:
    run: pnpm dev
    requirements:
      checks:
        - env-ready
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - web
"#,
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("app"),
        );

        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "File check failed: env-ready"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.summary.starts_with("Surface readiness failed:")),
            "{report:?}"
        );
    }

    #[test]
    fn workflow_env_check_blocks_surface_readiness_evaluation() {
        let tempdir = TempDir::new().unwrap();
        fs::write(
            tempdir.path().join(".env.compose"),
            "REDIS_HOST=127.0.0.1\nDATABASE_URL=postgres://user:pass@127.0.0.1:5432/langfuse\n",
        )
        .unwrap();
        let contract_path = tempdir.path().join("ota.yaml");
        let contract = parse_contract_str(
            contract_path.as_path(),
            r#"
version: 1
project:
  name: env-check-gate
execution:
  default_context: host
  contexts:
    host:
      backend: native
surfaces:
  web:
    kind: http
    port: 65530
checks:
  - name: compose-env-ready
    kind: env
    severity: error
    env:
      path: .env.compose
      assertions:
        - key: REDIS_HOST
          host:
            policy: not_loopback
        - key: DATABASE_URL
          url_host:
            policy: not_loopback
tasks:
  dev:
    run: pnpm dev
    requirements:
      checks:
        - compose-env-ready
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - web
"#,
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            contract_path.as_path(),
            Some("app"),
        );

        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Env check failed: compose-env-ready"),
            "{report:?}"
        );
        let env_finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Env check failed: compose-env-ready")
            .expect("env check finding should be present");
        assert_eq!(env_finding.code(), "OTA_CHECK_FAILED");
        assert_eq!(env_finding.category(), "execution");
        assert_eq!(env_finding.owner(), "repo_contract");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.why.contains(".env.compose")),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| !finding.summary.starts_with("Surface readiness failed:")),
            "{report:?}"
        );
    }

    #[test]
    fn workflow_readiness_surfaces_retry_until_ready() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let server = thread::spawn(move || {
            thread::sleep(Duration::from_millis(250));
            let listener = TcpListener::bind(("127.0.0.1", port)).expect("listener should bind");
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: {port}
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 100
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn workflow_surface_readiness_allows_brief_boot_delay() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        drop(listener);
        let server = thread::spawn(move || {
            thread::sleep(Duration::from_millis(1500));
            let listener = TcpListener::bind(("127.0.0.1", port)).expect("listener should bind");
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: {port}
    readiness:
      kind: http
      path: /healthz/readiness
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn workflow_surface_readiness_accepts_ipv6_only_loopback_listener() {
        let listener = match TcpListener::bind("[::1]:0") {
            Ok(listener) => listener,
            Err(_) => return,
        };
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: {port}
    readiness:
      kind: http
      path: /healthz/readiness
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn workflow_surface_failure_preserves_selected_workflow_in_next_step() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 6551
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 50ms
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#,
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );

        assert!(!report.ok, "{report:?}");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.next.contains("ota doctor --workflow backend")),
            "{report:?}"
        );
        assert!(
            report.findings.iter().any(|finding| finding
                .why
                .contains("backend `native`; endpoint `127.0.0.1:6551`")),
            "{report:?}"
        );
    }

    #[test]
    fn workflow_surface_failure_uses_native_override_backend_label_over_task_context_backend() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: verify
  contexts:
    verify:
      backend: container
      container:
        image: node:24-bookworm
surfaces:
  backend:
    kind: http
    port: 6551
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 50ms
tasks:
  dev:be:
    context: verify
    run: cargo --version
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#,
        )
        .unwrap();

        let report = diagnose_contract_with_mode_and_lifecycle_for_workflow_with_overrides(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            None,
            Some("backend"),
            crate::runner::ExecutionOverrides {
                backend: Some(Backend::Native),
                ..crate::runner::ExecutionOverrides::default()
            },
        );

        assert!(!report.ok, "{report:?}");
        assert!(
            report.findings.iter().any(|finding| {
                (finding.summary == "Surface readiness failed: backend"
                    || finding.summary == "Surface readiness timed out: backend")
                    && finding
                        .why
                        .contains("backend `native`; endpoint `127.0.0.1:6551`")
            }),
            "{report:?}"
        );
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Surface readiness failed: backend"
                    || finding.summary == "Surface readiness timed out: backend"
            })
            .expect("surface readiness finding should be present");
        assert!(matches!(
            finding.code(),
            "OTA_WORKFLOW_SURFACE_READINESS_FAILED" | "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT"
        ));
        assert_eq!(finding.category(), "execution");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn workflow_surface_timeout_uses_effective_default_timeout() {
        let retries = 3u32;
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            for _ in 0..retries {
                let (_stream, _) = listener.accept().expect("probe should connect");
                thread::sleep(Duration::from_millis(350));
            }
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: {port}
    readiness:
      kind: http
      path: /healthz/readiness
      retries: {retries}
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        server.join().expect("probe server should finish");

        assert!(!report.ok, "{report:?}");
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Surface readiness failed: backend"
                    || finding.summary == "Surface readiness timed out: backend"
            })
            .expect("surface timeout finding should be present");
        assert!(matches!(
            finding.code(),
            "OTA_WORKFLOW_SURFACE_READINESS_FAILED" | "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT"
        ));
        assert_eq!(finding.category(), "execution");
        assert_eq!(finding.owner(), "repo_contract");
        if finding.summary == "Surface readiness timed out: backend" {
            assert!(finding.why.contains("within 200ms across"), "{report:?}");
            assert!(!finding.why.contains("within 0ms"), "{report:?}");
        } else {
            assert!(finding.why.contains("after 3 checks"), "{report:?}");
        }
    }

    #[test]
    fn workflow_surface_timeout_retry_budget_caps_large_timeouts() {
        let configured = 120;
        let capped = super::capped_timed_out_retry_budget(configured, 10_000);
        assert_eq!(capped, 3);
    }

    #[test]
    fn workflow_surface_timeout_retry_budget_preserves_small_timeouts() {
        let configured = 30;
        let capped = super::capped_timed_out_retry_budget(configured, 200);
        assert_eq!(capped, configured);
    }

    #[test]
    fn workflow_surface_failed_retry_budget_caps_large_intervals() {
        let configured = 120;
        let capped = super::capped_failed_retry_budget(configured, Duration::from_secs(5));
        assert_eq!(capped, 6);
    }

    #[test]
    fn workflow_surface_failed_retry_budget_preserves_small_intervals() {
        let configured = 30;
        let capped = super::capped_failed_retry_budget(configured, Duration::from_millis(200));
        assert_eq!(capped, configured);
    }

    #[test]
    fn workflow_signal_surface_failure_is_non_gating() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 6553
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 50ms
      interval: 10ms
      retries: 1
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      signal:
        surfaces:
          - backend
"#,
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );

        assert!(report.ok, "{report:?}");
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Signal surface readiness failed: backend"
                    || finding.summary == "Signal surface readiness timed out: backend"
            })
            .expect("signal surface readiness finding should be present");
        assert_eq!(finding.severity, FindingSeverity::Info);
        assert!(matches!(
            finding.code(),
            "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED"
                | "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT"
        ));
        assert_eq!(finding.category(), "execution");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn workflow_surface_readiness_timing_caps_start_period_for_doctor() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 6553
    readiness:
      kind: http
      path: /healthz/readiness
      timeout: 50ms
      interval: 10ms
      retries: 1
      start_period: 30s
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#,
        )
        .unwrap();

        let timing = super::workflow_surface_readiness_timing_policy(&contract, "backend");
        assert_eq!(
            timing.start_period,
            Duration::from_millis(super::DOCTOR_READINESS_MAX_START_PERIOD_MS)
        );
    }

    #[test]
    fn service_readiness_timing_caps_start_period_for_doctor() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    required: true
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
    readiness:
      from: host
      run: exit 1
      start_period: 30s
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let readiness = contract
            .services
            .get("postgres")
            .and_then(|service| service.readiness.as_ref())
            .expect("service readiness should be present");
        let timing = super::service_readiness_timing_policy(readiness);
        assert_eq!(
            timing.start_period,
            Duration::from_millis(super::DOCTOR_READINESS_MAX_START_PERIOD_MS)
        );
    }

    #[test]
    fn windows_exit_code_guidance_line_decodes_common_crash_codes() {
        let guidance = super::windows_exit_code_guidance_line(-1073741819)
            .expect("known windows code should decode");
        assert!(guidance.contains("0xC0000005"), "{guidance}");
        assert!(guidance.contains("access violation"), "{guidance}");
    }

    #[test]
    fn workflow_surface_resolution_failure_becomes_blocking_finding() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
surfaces:
  backend:
    kind: http
    port: 6551
tasks:
  dev:be:
    run: pnpm dev:be
    runtime:
      kind: service
      listeners:
        http:
          http: 6551
workflows:
  default: backend
  backend:
    run:
      task: dev:be
    readiness:
      surfaces:
        - backend
"#,
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );

        assert!(!report.ok, "{report:?}");
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Surface readiness could not be evaluated: backend")
            .expect("resolution error should surface as a finding");
        assert!(finding.why.contains("could not be resolved or checked"));
        assert_eq!(
            finding.next,
            "repair workflow run task `dev:be` surface attachment/readiness and rerun `ota doctor --workflow backend`"
        );
    }

    #[test]
    fn probe_backed_check_can_reuse_task_target_probe() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      target:
        kind: task
        name: dev
        listener: backend
        address_view: host
      path: /healthz/readiness
      timeout: 1000
checks:
  - name: backend-ready
    kind: health
    severity: error
    probe: backend-ready
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: {port}
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: {port}
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn probe_backed_check_can_reuse_observer_task_topology_probe() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  shared_backends:
    workbench:
      scope: local
      backend: native
      lifecycle: persistent
readiness:
  probes:
    backend-ready:
      kind: http
      target:
        kind: task
        name: dev
        listener: backend
        address_view: topology
        observer:
          kind: task
          task: sandbox
      path: /healthz/readiness
      timeout: 1000
checks:
  - name: backend-ready
    kind: health
    severity: error
    probe: backend-ready
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        backend:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: {port}
  sandbox:
    run: echo sandbox
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        web:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 3000
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn service_http_probe_command_keeps_python_timeout_classification_reachable() {
        let endpoint = crate::schema::ServiceEndpointSpec {
            context: None,
            address: String::from("127.0.0.1"),
            port: 8080,
        };
        let request = HttpReadinessRequest {
            method: crate::schema::TaskRuntimeReadinessHttpMethod::Get,
            path: String::from("/healthz"),
            headers: BTreeMap::new(),
            success_statuses: vec![200],
            body_contains: None,
        };

        let command = super::service_http_readiness_probe_command_from_request(
            &endpoint,
            &request,
            Some(Duration::from_secs(1)),
        );

        assert!(!command.contains("<<'PY' && exit 0 || exit 1"));
        assert!(command.contains(
            "probe_status=$?; [ $probe_status -eq 0 ] && exit 0; [ $probe_status -eq 124 ] && exit 124; exit 1"
        ));
    }

    #[test]
    fn service_tcp_probe_command_keeps_python_timeout_classification_reachable() {
        let endpoint = crate::schema::ServiceEndpointSpec {
            context: None,
            address: String::from("127.0.0.1"),
            port: 5432,
        };

        let command = super::service_tcp_readiness_probe_command_from_timeout(
            &endpoint,
            Some(Duration::from_secs(1)),
        );

        assert!(!command.contains("<<'PY' && exit 0 || exit 1"));
        assert!(command.contains(
            "probe_status=$?; [ $probe_status -eq 0 ] && exit 0; [ $probe_status -eq 124 ] && exit 124; exit 1"
        ));
    }

    #[test]
    fn observer_task_probe_timeout_is_reported_deterministically() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (_stream, _) = listener.accept().expect("probe should connect");
            thread::sleep(Duration::from_millis(250));
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
execution:
  preferred: native
  shared_backends:
    workbench:
      scope: local
      backend: native
      lifecycle: persistent
readiness:
  probes:
    backend-ready:
      kind: http
      target:
        kind: task
        name: dev
        listener: backend
        address_view: topology
        observer:
          kind: task
          task: sandbox
      path: /healthz/readiness
      timeout: 100
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: {port}
  sandbox:
    run: echo sandbox
    runtime:
      kind: service
      backend_binding: workbench
      listeners:
        web:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 3000
"#
            )
            .as_str(),
        )
        .unwrap();

        let status = super::run_named_probe(
            &contract,
            synthetic_contract_path(),
            "backend-ready",
            None,
            crate::runner::ExecutionOverrides::default(),
        );
        server.join().expect("probe server should finish");

        assert!(matches!(status, CheckStatus::TimedOut(100)));
    }

    #[test]
    fn workflow_readiness_can_reference_task_target_probe() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      target:
        kind: task
        name: dev
        listener: backend
        address_view: host
      path: /healthz/readiness
      timeout: 1000
workflows:
  default: backend
  backend:
    readiness:
      probes:
        - backend-ready
tasks:
  dev:
    run: echo dev
    runtime:
      kind: service
      listeners:
        backend:
          protocol: http
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: {port}
          project:
            host:
              address: 127.0.0.1
              primary: true
              port:
                mode: fixed
                value: {port}
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn workflow_scoped_diagnosis_ignores_unrelated_workflow_probes() {
        let backend_listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let backend_port = backend_listener.local_addr().unwrap().port();
        let backend_server = thread::spawn(move || {
            let (mut stream, _) = backend_listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 0\r\nConnection: close\r\n\r\n")
                .expect("probe response should write");
        });

        let failing_listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let failing_port = failing_listener.local_addr().unwrap().port();
        drop(failing_listener);

        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
readiness:
  probes:
    app-ready:
      kind: http
      url: http://127.0.0.1:{failing_port}/healthz/readiness
      timeout: 100
    backend-ready:
      kind: http
      url: http://127.0.0.1:{backend_port}/healthz/readiness
      timeout: 1000
workflows:
  default: app
  app:
    readiness:
      probes:
        - app-ready
  backend:
    readiness:
      probes:
        - backend-ready
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        backend_server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn workflow_preconditions_follow_selected_task_closure_requirements() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    CONTRIBUTOR_ONLY:
      required: true
    DOCKER_ONLY:
      required: true
checks:
  - name: contributor-precondition
    kind: precondition
    severity: error
    run: __ota_missing_contributor_check__
  - name: docker-precondition
    kind: precondition
    severity: error
    run: echo docker-ready
tasks:
  setup:
    run: echo setup
    requirements:
      tools:
        contributor-setup-tool: "*"
      checks:
        - contributor-precondition
  install:
    run: echo install
    requirements:
      tools:
        contributor-only-tool: "*"
      env:
        - CONTRIBUTOR_ONLY
      checks:
        - contributor-precondition
  contributor:
    run: echo contributor
    depends_on:
      - install
  docker:run:
    run: echo docker
    requirements:
      tools:
        docker-only-tool: "*"
      env:
        - DOCKER_ONLY
      checks:
        - docker-precondition
workflows:
  default: contributor
  contributor:
    run:
      task: contributor
  docker:
    run:
      task: docker:run
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("docker"),
        );

        assert!(!report.ok, "{report:?}");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing environment variable: DOCKER_ONLY"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing tool: docker-only-tool"),
            "{report:?}"
        );
        assert!(
            report.findings.iter().all(|finding| {
                !finding.summary.contains("CONTRIBUTOR_ONLY")
                    && !finding.summary.contains("contributor-setup-tool")
                    && !finding.summary.contains("contributor-only-tool")
                    && !finding.summary.contains("contributor-precondition")
            }),
            "{report:?}"
        );
    }

    #[test]
    fn workflow_checks_include_explicit_task_precondition_checks_only() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: contributor-precondition
    kind: precondition
    severity: error
    run: __ota_missing_contributor_check__
  - name: docker-precondition
    kind: precondition
    severity: error
    run: echo docker-ready
tasks:
  contributor:
    run: echo contributor
    requirements:
      checks:
        - contributor-precondition
  docker:run:
    run: echo docker
    requirements:
      checks:
        - docker-precondition
workflows:
  default: contributor
  contributor:
    run:
      task: contributor
  docker:
    run:
      task: docker:run
"#,
        )
        .unwrap();

        let report = super::diagnose_checks_only_for_workflow(
            &contract,
            synthetic_contract_path(),
            Some("docker"),
        );

        assert!(report.ok, "{report:?}");
        assert!(
            report.findings.iter().all(|finding| {
                !finding.summary.contains("contributor-precondition")
                    && !finding.summary.contains("docker-precondition")
            }),
            "{report:?}"
        );
    }

    #[test]
    fn container_workflow_doctor_blocks_on_missing_selected_env() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "docker",
            if cfg!(windows) {
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
            } else {
                "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\necho unsupported >&2\nexit 1\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  preferred: container
  backends:
    container:
      image: node:20-bookworm
      engines: [docker]
env:
  vars:
    REQUIRED_CONTAINER_ENV:
      required: false
tasks:
  verify:
    run: echo verify
    execution:
      default_mode: container
    requirements:
      env:
        - REQUIRED_CONTAINER_ENV
workflows:
  default: verify
  verify:
    run:
      task: verify
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(!report.ok, "{report:?}");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary
                    == "Missing environment variable: REQUIRED_CONTAINER_ENV"),
            "{report:?}"
        );
        assert!(
            report.findings.iter().all(|finding| finding.summary
                != "Container readiness does not include host-only checks"),
            "{report:?}"
        );
    }

    #[test]
    fn workflow_preconditions_with_scoped_requirements_do_not_run_unreferenced_global_checks() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: contributor-precondition
    kind: precondition
    severity: error
    run: __ota_missing_contributor_check__
tasks:
  setup:
    run: echo setup
    requirements:
      checks:
        - contributor-precondition
  docker:run:
    run: echo docker
    requirements:
      tools:
        docker: "*"
workflows:
  default: contributor
  contributor:
    setup:
      task: setup
  docker:
    run:
      task: docker:run
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("docker"),
        );

        assert!(
            report
                .findings
                .iter()
                .all(|finding| { !finding.summary.contains("contributor-precondition") }),
            "{report:?}"
        );

        let full_report = super::diagnose_contract_with_mode_and_lifecycle_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            None,
            Some("docker"),
        );
        assert!(
            full_report
                .findings
                .iter()
                .all(|finding| { !finding.summary.contains("contributor-precondition") }),
            "{full_report:?}"
        );
    }

    #[test]
    fn workflow_preconditions_surface_selected_native_prerequisites() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  node-native-build-tools:
    description: Native compiler toolchain for packages with native addons
    platforms:
      linux:
        check: node-native-build-tools-linux
        apt:
          - build-essential
          - python3
      macos:
        check: node-native-build-tools-macos
        xcode_clt: true
      windows:
        check: node-native-build-tools-windows
        visual_studio_build_tools: true
checks:
  - name: node-native-build-tools-linux
    kind: precondition
    severity: error
    run: __ota_missing_native_build_tools__
  - name: node-native-build-tools-macos
    kind: precondition
    severity: error
    run: __ota_missing_native_build_tools__
  - name: node-native-build-tools-windows
    kind: precondition
    severity: error
    run: __ota_missing_native_build_tools__
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
workflows:
  default: app
  app:
    setup:
      task: setup
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("app"),
        );

        assert!(!report.ok);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary
                    == "Native prerequisite missing: node-native-build-tools"),
            "{report:?}"
        );
        assert!(
            report.findings.iter().all(|finding| !finding
                .summary
                .starts_with("Check failed: node-native-build-tools-")),
            "{report:?}"
        );
    }

    #[cfg(unix)]
    #[test]
    fn structured_visual_studio_probe_uses_vswhere_components() {
        let _guard = env_mutex_lock();
        let tempdir = TempDir::new().unwrap();
        let installer_dir = tempdir
            .path()
            .join("Microsoft Visual Studio")
            .join("Installer");
        fs::create_dir_all(&installer_dir).unwrap();
        let log_path = tempdir.path().join("vswhere.args");
        write_fake_command(
            &installer_dir,
            "vswhere.exe",
            &format!(
                "#!/bin/sh\nprintf '%s\\n' \"$*\" > '{}'\nprintf '%s\\n' '/opt/VisualStudio'\n",
                log_path.display()
            ),
        );

        let original_program_files = env::var_os("ProgramFiles(x86)");
        unsafe {
            env::set_var("ProgramFiles(x86)", tempdir.path());
        }
        let platform = crate::schema::NativePrerequisitePlatformSpec {
            visual_studio: Some(crate::schema::NativePrerequisiteVisualStudioSpec {
                components: vec![String::from(
                    "Microsoft.VisualStudio.Component.VC.Tools.x86.x64",
                )],
            }),
            ..Default::default()
        };

        let status = super::run_visual_studio_native_prerequisite_check(&platform);

        unsafe {
            match original_program_files {
                Some(value) => env::set_var("ProgramFiles(x86)", value),
                None => env::remove_var("ProgramFiles(x86)"),
            }
        }
        assert!(
            matches!(status, super::NativePrerequisiteCheckStatus::Passed),
            "structured Visual Studio probe should pass"
        );
        let args = fs::read_to_string(log_path).unwrap();
        assert!(
            args.contains("-requires Microsoft.VisualStudio.Component.VC.Tools.x86.x64"),
            "{args}"
        );
    }

    #[test]
    fn legacy_preconditions_still_run_without_task_scoped_requirements() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: global-precondition
    kind: precondition
    severity: error
    run: __ota_missing_global_check__
tasks:
  dev:
    run: echo dev
    requirements:
      checks:
        - global-precondition
workflows:
  default: app
  app:
    run:
      task: dev
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("app"),
        );

        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary.contains("global-precondition")),
            "{report:?}"
        );
    }

    #[test]
    fn command_based_checks_still_work_after_probe_support() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: health-check
    kind: health
    severity: error
    run: exit 1
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].summary, "Check failed: health-check");
    }

    #[cfg(unix)]
    #[test]
    fn command_native_prerequisite_diagnosis_uses_declared_activation_env() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  nix-shell:
    description: Project shell activation
    platforms:
      linux:
        check: nix-shell-check
        activation:
          kind: command
          shell: sh
          run: export OTA_NATIVE_READY=yes
checks:
  - name: nix-shell-check
    kind: precondition
    severity: error
    run: test "$OTA_NATIVE_READY" = yes
"#,
        )
        .unwrap();

        let prerequisite = contract
            .native_prerequisites
            .get("nix-shell")
            .expect("native prerequisite");
        let check = contract
            .checks
            .iter()
            .find(|check| check.name == "nix-shell-check")
            .expect("declared check");

        let status = super::run_native_prerequisite_check(
            prerequisite,
            "nix-shell",
            "linux",
            check,
            Path::new("."),
        );

        assert!(matches!(
            status,
            super::NativePrerequisiteCheckStatus::Passed
        ));
    }

    #[cfg(unix)]
    #[test]
    fn command_native_prerequisite_diagnosis_reports_activation_failure_details() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  nix-shell:
    description: Project shell activation
    platforms:
      linux:
        check: nix-shell-check
        activation:
          kind: command
          shell: sh
          run: exit 7
checks:
  - name: nix-shell-check
    kind: precondition
    severity: error
    run: test "$OTA_NATIVE_READY" = yes
"#,
        )
        .unwrap();

        let prerequisite = contract
            .native_prerequisites
            .get("nix-shell")
            .expect("native prerequisite");
        let check = contract
            .checks
            .iter()
            .find(|check| check.name == "nix-shell-check")
            .expect("declared check");

        let status = super::run_native_prerequisite_check(
            prerequisite,
            "nix-shell",
            "linux",
            check,
            Path::new("."),
        );

        match status {
            super::NativePrerequisiteCheckStatus::Failed(Some(details)) => {
                assert!(
                    details.contains("declared native activation failed"),
                    "{details}"
                );
            }
            _ => panic!("expected activation failure details"),
        }
    }

    #[test]
    fn diagnose_checks_points_missing_file_precondition_to_wire_setup_when_setup_missing() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: env-local-present
    kind: precondition
    severity: error
    run: test -f .env.local
tasks:
  dev:
    run: cargo test
    requirements:
      checks:
        - env-local-present
workflows:
  default: dev
  dev:
    run:
      task: dev
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].summary,
            "Check failed: env-local-present"
        );
        assert_eq!(
            report.findings[0].next,
            "create `.env.local` now, or declare a setup path with `ota assist wire-setup --run '<command>'`, then rerun `ota doctor`"
        );
    }

    #[test]
    fn reports_optional_tool_version_mismatches_as_warnings() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let _rustc_path = write_fake_command(
            &bin_dir,
            "rustc",
            if cfg!(windows) {
                "@echo off\r\necho rustc 1.99.0 (fake)\r\n"
            } else {
                "#!/bin/sh\nprintf 'rustc 1.99.0 (fake)\\n'\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  rustc:
    version: "999.0.0"
    required: false
tasks:
  test:
    run: rustc --version
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.code() == "OTA_TOOL_VERSION_MISMATCH")
            .expect("expected tool version mismatch finding");
        assert_eq!(finding.severity, FindingSeverity::Warn);
        assert_eq!(finding.summary, "Version mismatch for tool: rustc");
        assert_eq!(finding.category(), "environment");
        assert_eq!(finding.owner(), "host");
    }

    #[test]
    fn reports_required_service_healthcheck_failures_as_errors() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    required: true
    start: docker compose up -d postgres
    healthcheck: exit 1
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.code() == "OTA_SERVICE_CHECK_FAILED")
            .expect("expected service healthcheck failure finding");
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert_eq!(finding.summary, "Service healthcheck failed: postgres");
        assert_eq!(finding.category(), "service");
        assert_eq!(finding.owner(), "service");
    }

    #[test]
    fn reports_contextual_service_readiness_failures_with_projected_endpoint() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    required: true
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
    readiness:
      from: host
      run: exit 1
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.code() == "OTA_SERVICE_READINESS_FAILED")
            .expect("expected service readiness failure finding");
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert_eq!(finding.summary, "Service readiness failed: postgres");
        assert_eq!(finding.category(), "service");
        assert_eq!(finding.owner(), "service");
        assert!(
            finding
                .why
                .contains("projected endpoint is `127.0.0.1:5432`")
        );
    }

    #[test]
    fn diagnose_service_infers_container_mode_for_contextual_readiness_guidance() {
        struct EnvPathGuard {
            original: Option<std::ffi::OsString>,
        }

        impl Drop for EnvPathGuard {
            fn drop(&mut self) {
                match &self.original {
                    Some(path) => unsafe {
                        env::set_var("PATH", path);
                    },
                    None => unsafe {
                        env::remove_var("PATH");
                    },
                }
            }
        }

        let _guard = env_mutex_lock();
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        let path_separator = if cfg!(windows) { ';' } else { ':' };
        let new_path = match &original_path {
            Some(path) => {
                format!(
                    "{}{}{}",
                    bin_dir.display(),
                    path_separator,
                    path.to_string_lossy()
                )
            }
            None => bin_dir.display().to_string(),
        };
        let _path_guard = EnvPathGuard {
            original: original_path,
        };
        unsafe {
            env::set_var("PATH", new_path);
        }

        let fake_docker = if cfg!(windows) {
            r#"@echo off
if "%1"=="info" exit /b 0
if "%1"=="inspect" exit /b 1
if "%1"=="run" exit /b 0
if "%1"=="exec" (
  echo %* | findstr /C:"exit 1" >nul
  if errorlevel 1 exit /b 0
  exit /b 1
)
if "%1"=="ps" exit /b 0
exit /b 0
"#
        } else {
            "#!/bin/sh\n\
case \"$1\" in\n\
  info)\n    exit 0\n    ;;\n\
  inspect)\n    exit 1\n    ;;\n\
  run)\n    exit 0\n    ;;\n\
  exec)\n    if echo \"$*\" | grep -q \"exit 1\"; then\n      exit 1\n    fi\n    exit 0\n    ;;\n\
  ps)\n    exit 0\n    ;;\n\
  *)\n    exit 0\n    ;;\n\
esac\n"
        };

        write_fake_command(&bin_dir, "docker", fake_docker);

        let contract = parse_contract_str(
            synthetic_contract_path(),
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
      container:
        image: ghcr.io/ota/dev:latest
services:
  postgres:
    required: true
    readiness:
      from: app
      run: exit 1
tasks:
  dev:
    run: echo dev
"#,
        )
        .unwrap();

        let report = super::diagnose_service(&contract, synthetic_contract_path(), "postgres");
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert!(
            report.findings[0]
                .next
                .contains("rerun `ota doctor --mode container --lifecycle ephemeral`"),
            "{}",
            report.findings[0].next
        );
    }

    #[test]
    fn diagnose_service_supports_structured_http_readiness() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener
            .local_addr()
            .expect("listener address should resolve")
            .port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(
                    b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"status\":\"UP\"}",
                )
                .expect("probe response should write");
        });
        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  api:
    required: true
    endpoints:
      host:
        address: 127.0.0.1
        port: {port}
    readiness:
      from: host
      kind: http
      method: GET
      path: /health
      headers:
        Accept: application/json
      success:
        status: [200]
      body:
        contains: '"status":"UP"'
      interval: 50ms
      timeout: 200ms
      retries: 2
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_service(&contract, synthetic_contract_path(), "api");
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    #[cfg(not(windows))]
    fn diagnose_service_structured_http_readiness_uses_default_retry_budget_when_retries_omitted() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener
            .local_addr()
            .expect("listener address should resolve")
            .port();
        let server = thread::spawn(move || {
            for attempt in 0..2 {
                let (mut stream, _) = listener.accept().expect("probe should connect");
                drain_probe_request_if_available(&mut stream);
                if attempt == 0 {
                    stream
                        .write_all(
                            b"HTTP/1.1 503 Service Unavailable\r\nContent-Type: application/json\r\nContent-Length: 19\r\nConnection: close\r\n\r\n{\"status\":\"BOOTING\"}",
                        )
                        .expect("initial probe response should write");
                } else {
                    stream
                        .write_all(
                            b"HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: 15\r\nConnection: close\r\n\r\n{\"status\":\"UP\"}",
                        )
                        .expect("ready probe response should write");
                }
            }
        });
        let contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  api:
    required: true
    endpoints:
      host:
        address: 127.0.0.1
        port: {port}
    readiness:
      from: host
      kind: http
      method: GET
      path: /health
      headers:
        Accept: application/json
      success:
        status: [200]
      body:
        contains: '"status":"UP"'
      interval: 20ms
      timeout: 200ms
"#
            )
            .as_str(),
        )
        .unwrap();

        let report = super::diagnose_service(&contract, synthetic_contract_path(), "api");
        server.join().expect("probe server should finish");

        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn container_mode_service_readiness_failure_uses_container_rerun_guidance() {
        struct EnvPathGuard {
            original: Option<std::ffi::OsString>,
        }

        impl Drop for EnvPathGuard {
            fn drop(&mut self) {
                match &self.original {
                    Some(path) => unsafe {
                        env::set_var("PATH", path);
                    },
                    None => unsafe {
                        env::remove_var("PATH");
                    },
                }
            }
        }

        let _guard = env_mutex_lock();
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(path) = &original_path {
            path_entries.extend(env::split_paths(path));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        let _path_guard = EnvPathGuard {
            original: original_path,
        };
        unsafe {
            env::set_var("PATH", joined_path);
        }

        let fake_docker = if cfg!(windows) {
            r#"@echo off
if "%1"=="info" exit /b 0
if "%1"=="inspect" exit /b 1
if "%1"=="run" exit /b 0
if "%1"=="exec" (
  echo %* | findstr /C:"exit 1" >nul
  if errorlevel 1 exit /b 0
  exit /b 1
)
if "%1"=="ps" exit /b 0
exit /b 0
"#
        } else {
            "#!/bin/sh\n\
case \"$1\" in\n\
  info)\n    exit 0\n    ;;\n\
  inspect)\n    exit 1\n    ;;\n\
  run)\n    exit 0\n    ;;\n\
  exec)\n    if echo \"$*\" | grep -q \"exit 1\"; then\n      exit 1\n    fi\n    exit 0\n    ;;\n\
  ps)\n    exit 0\n    ;;\n\
  *)\n    exit 0\n    ;;\n\
esac\n"
        };
        write_fake_command(&bin_dir, "docker", fake_docker);

        let contract = parse_contract_str(
            synthetic_contract_path(),
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
      container:
        image: ghcr.io/ota/dev:latest
services:
  postgres:
    required: true
    readiness:
      from: app
      run: exit 1
tasks:
  dev:
    run: echo dev
"#,
        )
        .unwrap();

        let report = super::diagnose_service(&contract, synthetic_contract_path(), "postgres");
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary.starts_with("Service readiness"))
            .expect("expected service readiness finding");
        assert!(
            finding
                .next
                .contains("rerun `ota doctor --mode container --lifecycle ephemeral`"),
            "{}",
            finding.next
        );
    }

    #[test]
    fn reports_unexecutable_service_readiness_contexts_explicitly() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
    remote-db:
      backend: remote
      remote:
        provider: ssh
services:
  postgres:
    required: true
    endpoints:
      remote-db:
        address: postgres.internal
        port: 5432
    readiness:
      from: remote-db
      run: pg_isready -h postgres.internal -p 5432
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Service readiness context is not executable: postgres"
            })
            .expect("expected readiness execution blocker");
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert_eq!(
            finding.summary,
            "Service readiness context is not executable: postgres"
        );
        assert_eq!(finding.code(), "OTA_SERVICE_READINESS_CONTEXT_UNEXECUTABLE");
        assert_eq!(finding.category(), "service");
        assert_eq!(finding.owner(), "service");
        assert!(finding.why.contains("context `remote-db`"));
        assert!(finding.why.contains("postgres.internal:5432"));
        assert!(
            finding
                .why
                .contains("requires `execution.backends.remote.target`")
        );
        assert!(
            finding
                .next
                .contains("repair execution context `remote-db`")
        );
    }

    #[test]
    fn preconditions_use_native_context_requirements() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
      requirements:
        tools:
          definitely-not-installed: "*"
tasks:
  setup:
    context: host
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, synthetic_contract_path());
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing tool: definitely-not-installed")
        );
    }

    #[test]
    fn preconditions_dedupe_identical_missing_tool_findings() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
      requirements:
        tools:
          definitely-not-installed: "*"
tasks:
  setup:
    context: host
    command:
      exe: definitely-not-installed
    requirements:
      tools:
        definitely-not-installed: "*"
"#,
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, synthetic_contract_path());
        let missing = report
            .findings
            .iter()
            .filter(|finding| finding.summary == "Missing tool: definitely-not-installed")
            .count();
        assert_eq!(missing, 1, "{report:?}");
    }

    #[test]
    fn preconditions_dedupe_missing_tool_alias_for_owned_requirement() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
      requirements:
        tools:
          maven: "*"
tasks:
  setup:
    context: host
    command:
      exe: mvn
      args:
        - test
workflows:
  default: app
  app:
    run:
      task: setup
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("app"),
        );
        let canonical_missing = report
            .findings
            .iter()
            .filter(|finding| finding.summary == "Missing tool: maven")
            .count();
        assert_eq!(canonical_missing, 1, "{report:?}");
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing tool: mvn"),
            "{report:?}"
        );
    }

    #[test]
    fn repo_hygiene_accepts_parent_ota_gitignore_entry() {
        let fixture = TempDir::new().unwrap();
        fs::write(fixture.path().join(".gitignore"), ".ota/\n").unwrap();

        let missing = super::repo_missing_ota_state_gitignore(fixture.path()).unwrap();
        assert!(!missing);
    }

    #[test]
    fn preconditions_include_selected_native_prerequisite_platform_requires() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    NODE_GYP_HOME:
      required: true
native_prerequisites:
  node-native-build-tools:
    platforms:
      linux:
        check: node-native-build-tools-linux
        apt:
          - build-essential
        requires:
          tools:
            definitely-not-installed-native-tool: "*"
          env:
            - NODE_GYP_HOME
          checks:
            - native-extra-check
      macos:
        check: node-native-build-tools-macos
        xcode_clt: true
        requires:
          tools:
            definitely-not-installed-native-tool: "*"
          env:
            - NODE_GYP_HOME
          checks:
            - native-extra-check
      windows:
        check: node-native-build-tools-windows
        requires:
          tools:
            definitely-not-installed-native-tool: "*"
          env:
            - NODE_GYP_HOME
          checks:
            - native-extra-check
checks:
  - name: node-native-build-tools-linux
    kind: precondition
    severity: error
    run: echo native-tools-present
  - name: node-native-build-tools-macos
    kind: precondition
    severity: error
    run: echo native-tools-present
  - name: node-native-build-tools-windows
    kind: precondition
    severity: error
    run: echo native-tools-present
  - name: native-extra-check
    kind: precondition
    severity: error
    run: __ota_missing_native_extra_check__
tasks:
  setup:
    run: pnpm install
    requirements:
      native:
        - node-native-build-tools
"#,
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, synthetic_contract_path());
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary
                    == "Missing tool: definitely-not-installed-native-tool"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing environment variable: NODE_GYP_HOME"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary.contains("native-extra-check")),
            "{report:?}"
        );
    }

    #[test]
    fn container_preconditions_do_not_pull_unrelated_host_context_requirements_for_compose_managers()
     {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: app
  contexts:
    host:
      backend: native
      requirements:
        tools:
          definitely-not-installed: "*"
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
tasks:
  setup:
    context: app
    run: printf ready
services:
  postgres:
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
"#,
        )
        .unwrap();

        let report = diagnose_preconditions_with_mode(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
        );
        assert!(
            !report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing tool: definitely-not-installed")
        );
    }

    #[test]
    fn container_precondition_surface_excludes_host_global_tool_fallback_for_selected_path() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tools:
  docker: "*"
execution:
  default_context: app
  contexts:
    host:
      backend: native
    app:
      backend: container
      lifecycle: persistent
      container:
        image: ghcr.io/ota/dev:latest
tasks:
  build:
    context: app
    run: pnpm build
workflows:
  default: app
  app:
    run:
      task: build
"#,
        )
        .unwrap();

        let selection = super::selected_backend_precondition_selections(
            &contract,
            None,
            ExecutionOverrides::default(),
        )
        .into_iter()
        .find(|selection| selection.backend == Backend::Container)
        .expect("container backend selection should resolve");
        let surface = selection.requirement_surface;
        assert!(
            !surface.tools.contains_key("docker"),
            "container selected path should not inherit global host tool fallback"
        );
    }

    #[test]
    fn reports_optional_service_healthcheck_failures_as_warnings() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  cache:
    required: false
    healthcheck: exit 1
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Service healthcheck failed: cache")
            .expect("expected optional service healthcheck warning");
        assert_eq!(finding.severity, FindingSeverity::Warn);
    }

    #[test]
    fn required_service_with_start_and_endpoint_routes_to_declare_readiness() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    required: true
    start: docker compose up -d postgres
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Required service cannot be verified: postgres")
            .expect("expected required service unverifiable finding");
        assert_eq!(finding.severity, FindingSeverity::Warn);
        assert_eq!(
            finding.next,
            "declare readiness with `ota assist declare-readiness --service postgres --style tcp` or `--style http`, then rerun `ota doctor`"
        );
    }

    #[test]
    fn warns_when_required_service_has_no_start_or_healthcheck() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    required: true
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Required service cannot be verified: postgres")
            .expect("expected required service unverifiable finding");
        assert_eq!(finding.severity, FindingSeverity::Warn);
        assert_eq!(
            finding.next,
            "refine the managed service with `ota assist declare-service --name postgres --style tcp` or `--style http`, then rerun `ota doctor`"
        );
    }

    #[test]
    fn service_readiness_probe_reuse_succeeds() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("bind probe listener");
        let port = listener
            .local_addr()
            .expect("probe listener address")
            .port();
        let server = std::thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            let mut buffer = [0u8; 512];
            let _ = stream.read(&mut buffer);
            stream
                .write_all(b"HTTP/1.1 200 OK\r\nContent-Length: 2\r\nConnection: close\r\n\r\nok")
                .expect("probe response should write");
        });

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &format!(
                r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
readiness:
  probes:
    postgres-ready:
      kind: http
      url: http://127.0.0.1:65535/ready
      timeout: 10000
services:
  postgres:
    required: true
    endpoints:
      host:
        address: 127.0.0.1
        port: {port}
    readiness:
      from: host
      probe: postgres-ready
tasks:
  setup:
    run: printf ready
"#
            ),
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        server.join().expect("probe server should finish");
        assert!(report.findings.iter().all(|finding| {
            !finding.summary.starts_with("Service readiness")
                && finding.summary != "Required service cannot be verified: postgres"
        }));
    }

    #[test]
    fn required_host_service_with_one_endpoint_routes_to_declare_readiness() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    required: true
    manager:
      kind: host
      name: local-postgres
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Required service cannot be verified: postgres")
            .expect("expected required service unverifiable finding");
        assert_eq!(
            finding.next,
            "declare readiness with `ota assist declare-readiness --service postgres --style tcp` or `--style http`, then rerun `ota doctor`"
        );
    }

    #[test]
    fn required_compose_service_without_endpoint_routes_to_compose_health_readiness() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Required service cannot be verified: postgres")
            .expect("expected required service unverifiable finding");
        assert_eq!(
            finding.next,
            "declare readiness with `ota assist declare-readiness --service postgres --style compose-health` (or `--style tcp` / `--style http`), then rerun `ota doctor`"
        );
    }

    #[test]
    #[cfg(not(windows))]
    fn diagnose_service_supports_structured_compose_health_readiness() {
        struct EnvPathGuard {
            original: Option<std::ffi::OsString>,
        }

        impl Drop for EnvPathGuard {
            fn drop(&mut self) {
                match &self.original {
                    Some(path) => unsafe {
                        env::set_var("PATH", path);
                    },
                    None => unsafe {
                        env::remove_var("PATH");
                    },
                }
            }
        }

        let _guard = env_mutex_lock();
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        let path_separator = ':';
        let new_path = match &original_path {
            Some(path) => {
                format!(
                    "{}{}{}",
                    bin_dir.display(),
                    path_separator,
                    path.to_string_lossy()
                )
            }
            None => bin_dir.display().to_string(),
        };
        let _path_guard = EnvPathGuard {
            original: original_path,
        };
        unsafe {
            env::set_var("PATH", new_path);
        }

        write_fake_command(
            &bin_dir,
            "docker",
            "#!/bin/sh\n\
if [ \"$1\" = \"compose\" ]; then\n\
  shift\n\
  while [ $# -gt 0 ]; do\n\
    if [ \"$1\" = \"ps\" ]; then\n\
      shift\n\
      if [ \"$1\" = \"-q\" ]; then\n\
        echo worker-container\n\
        exit 0\n\
      fi\n\
    fi\n\
    shift\n\
  done\n\
  exit 1\n\
fi\n\
if [ \"$1\" = \"inspect\" ]; then\n\
  echo healthy\n\
  exit 0\n\
fi\n\
exit 1\n",
        );

        let contract_path = temp_dir.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
services:
  worker:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: worker
    readiness:
      kind: compose_health
      interval: 10ms
      retries: 1
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = super::diagnose_service(&contract, synthetic_contract_path(), "worker");
        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    #[cfg(not(windows))]
    fn diagnose_service_supports_structured_podman_compose_health_readiness() {
        struct EnvPathGuard {
            original: Option<std::ffi::OsString>,
        }

        impl Drop for EnvPathGuard {
            fn drop(&mut self) {
                match &self.original {
                    Some(path) => unsafe {
                        env::set_var("PATH", path);
                    },
                    None => unsafe {
                        env::remove_var("PATH");
                    },
                }
            }
        }

        let _guard = env_mutex_lock();
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        let path_separator = ':';
        let new_path = match &original_path {
            Some(path) => {
                format!(
                    "{}{}{}",
                    bin_dir.display(),
                    path_separator,
                    path.to_string_lossy()
                )
            }
            None => bin_dir.display().to_string(),
        };
        let _path_guard = EnvPathGuard {
            original: original_path,
        };
        unsafe {
            env::set_var("PATH", new_path);
        }

        write_fake_command(
            &bin_dir,
            "podman",
            "#!/bin/sh\n\
if [ \"$1\" = \"compose\" ]; then\n\
  shift\n\
  while [ $# -gt 0 ]; do\n\
    if [ \"$1\" = \"ps\" ]; then\n\
      shift\n\
      if [ \"$1\" = \"-q\" ]; then\n\
        echo worker-container\n\
        exit 0\n\
      fi\n\
    fi\n\
    shift\n\
  done\n\
  exit 1\n\
fi\n\
if [ \"$1\" = \"inspect\" ]; then\n\
  echo healthy\n\
  exit 0\n\
fi\n\
exit 1\n",
        );

        let contract_path = temp_dir.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
services:
  worker:
    required: true
    manager:
      kind: compose
      engine: podman
      name: local
      file: compose.yaml
      service: worker
    readiness:
      kind: compose_health
      interval: 10ms
      retries: 1
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = super::diagnose_service(&contract, synthetic_contract_path(), "worker");
        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    #[cfg(not(windows))]
    fn diagnose_service_supports_structured_systemd_active_readiness() {
        struct EnvPathGuard {
            original: Option<std::ffi::OsString>,
        }

        impl Drop for EnvPathGuard {
            fn drop(&mut self) {
                match &self.original {
                    Some(path) => unsafe {
                        env::set_var("PATH", path);
                    },
                    None => unsafe {
                        env::remove_var("PATH");
                    },
                }
            }
        }

        let _guard = env_mutex_lock();
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        let new_path = match &original_path {
            Some(path) => format!("{}:{}", bin_dir.display(), path.to_string_lossy()),
            None => bin_dir.display().to_string(),
        };
        let _path_guard = EnvPathGuard {
            original: original_path,
        };
        unsafe {
            env::set_var("PATH", new_path);
        }

        write_fake_command(
            &bin_dir,
            "systemctl",
            "#!/bin/sh\n\
if [ \"$1\" = \"--user\" ]; then\n\
  shift\n\
fi\n\
if [ \"$1\" = \"is-active\" ] && [ \"$2\" = \"--quiet\" ] && [ \"$3\" = \"redis.service\" ]; then\n\
  exit 0\n\
fi\n\
exit 1\n",
        );

        let contract_path = temp_dir.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
services:
  redis:
    required: true
    manager:
      kind: host
      host:
        kind: systemd
        unit: redis.service
        scope: user
    readiness:
      kind: systemd_active
      interval: 10ms
      retries: 1
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = super::diagnose_service(&contract, synthetic_contract_path(), "redis");
        assert!(report.ok, "{report:?}");
        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    #[cfg(not(windows))]
    fn diagnose_service_compose_manager_fails_fast_when_service_is_not_running() {
        struct EnvPathGuard {
            original: Option<std::ffi::OsString>,
        }

        impl Drop for EnvPathGuard {
            fn drop(&mut self) {
                match &self.original {
                    Some(path) => unsafe {
                        env::set_var("PATH", path);
                    },
                    None => unsafe {
                        env::remove_var("PATH");
                    },
                }
            }
        }

        let _guard = env_mutex_lock();
        let temp_dir = TempDir::new().unwrap();
        let bin_dir = temp_dir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var_os("PATH");
        let new_path = match &original_path {
            Some(path) => format!("{}:{}", bin_dir.display(), path.to_string_lossy()),
            None => bin_dir.display().to_string(),
        };
        let _path_guard = EnvPathGuard {
            original: original_path,
        };
        unsafe {
            env::set_var("PATH", new_path);
        }

        write_fake_command(
            &bin_dir,
            "docker",
            "#!/bin/sh\n\
if [ \"$1\" = \"compose\" ]; then\n\
  exit 1\n\
fi\n\
exit 1\n",
        );

        let contract_path = temp_dir.path().join("ota.yaml");
        let contract = parse_contract_str(
            &contract_path,
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    required: true
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: postgres
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
    readiness:
      kind: tcp
      from: host
      interval: 2s
      retries: 50
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let started = std::time::Instant::now();
        let report = super::diagnose_service(&contract, &contract_path, "postgres");
        let elapsed = started.elapsed();

        assert!(!report.ok, "{report:?}");
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary.starts_with("Service readiness"))
            .expect("expected service readiness failure finding");
        assert!(elapsed < Duration::from_secs(1), "elapsed: {elapsed:?}");
        assert!(
            finding
                .next
                .contains("docker compose -f 'compose.yaml' -p 'local' up -d 'postgres'"),
            "{}",
            finding.next
        );
    }

    #[test]
    fn workflow_scoped_services_ignore_unselected_global_services() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: workflow-services
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    readiness:
      kind: tcp
      from: host
      interval: 1s
      retries: 1
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
  redis:
    readiness:
      kind: tcp
      from: host
      interval: 1s
      retries: 1
    endpoints:
      host:
        address: 127.0.0.1
        port: 6379
tasks:
  selfhost:
    run: docker compose up
  dev:
    run: pnpm dev
    requires_services:
      - postgres
      - redis
workflows:
  default: dev
  dev:
    run:
      task: dev
  selfhost:
    run:
      task: selfhost
"#,
        )
        .unwrap();

        let report = super::diagnose_services_only_for_workflow(
            &contract,
            Path::new("ota.yaml"),
            Some("selfhost"),
        );

        assert!(report.findings.is_empty(), "{report:?}");
    }

    #[test]
    fn producer_owned_service_surfaces_workspace_owner_when_unreachable() {
        let reserved_listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let reserved_port = reserved_listener.local_addr().unwrap().port();
        drop(reserved_listener);
        let fixture = tempfile::tempdir().unwrap();
        fs::write(
            fixture.path().join("ota.workspace.yaml"),
            r#"
version: 1
workspace:
  name: ota-workspace
repos:
  api:
    path: ./api
  web:
    path: ./web
"#
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("api")).unwrap();
        fs::write(
            fixture.path().join("api").join("ota.yaml"),
            r#"
version: 1
project:
  name: api
tasks:
  dev:
    run: echo api
    runtime:
      kind: service
      readiness:
        kind: tcp
        listener: http
      listeners:
        http:
          protocol: http
          bind:
            address: 0.0.0.0
            port:
              mode: fixed
              value: 8080
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: __PORT__
"#
            .replace("__PORT__", reserved_port.to_string().as_str())
            .trim_start(),
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join("web")).unwrap();
        let contract_path = fixture.path().join("web").join("ota.yaml");
        fs::write(
            &contract_path,
            r#"
version: 1
project:
  name: web
services:
  user-api:
    required: true
    producer:
      repo: api
      task: dev
      listener: http
tasks:
  setup:
    requires_services:
      - user-api
    run: echo setup
"#
            .trim_start(),
        )
        .unwrap();
        let contract = crate::parser::load_contract(&contract_path).unwrap();

        let report = diagnose_contract(&contract, &contract_path);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Service producer is not ready: user-api")
            .expect("expected producer-owned service finding");

        assert!(
            finding
                .why
                .contains("workspace repo `api` task `dev` listener `http`")
        );
        assert!(finding.next.contains("ota workspace up"));
        assert!(finding.next.contains("ota run dev"));
        assert!(finding.next.contains(&format!(
            "ota run dev '{}'",
            fixture.path().join("api").display()
        )));
    }

    #[test]
    fn reports_missing_tasks_as_not_ready() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(report.findings[0].summary, "No tasks defined in contract");
        assert_eq!(
            report.findings[0].next,
            "run `ota detect --dry-run` to review inferred tasks before writing, or run `ota assist add-task --name dev --kind command` when you want one explicit runnable task"
        );
    }

    #[test]
    fn warns_missing_tasks_for_sdk_type() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota-sdk
  type: sdk
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[0].summary, "No tasks defined in contract");
    }

    #[test]
    fn warns_missing_tasks_for_library_type_case_insensitive() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota-lib
  type: Library
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[0].summary, "No tasks defined in contract");
    }

    #[test]
    fn checks_only_scope_does_not_require_tasks() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: formatting
    kind: health
    severity: warn
    run: exit 0
"#,
        )
        .unwrap();

        let report = diagnose_checks_only(&contract, synthetic_contract_path());
        assert!(report.ok);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn reports_timed_out_service_healthchecks() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
services:
  postgres:
    required: true
    healthcheck: sleep 1
    timeout: 10
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Service healthcheck timed out: postgres")
            .expect("expected timed out service healthcheck finding");
        assert_eq!(finding.severity, FindingSeverity::Error);
    }

    #[test]
    fn sorts_errors_before_warnings_before_info() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
services:
  cache:
    required: false
    healthcheck: exit 1
checks:
  - name: blocking-check
    kind: health
    severity: error
    run: exit 1
  - name: informational-check
    kind: health
    severity: info
    run: exit 1
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(!report.ok);
        assert!(report.findings.len() >= 3);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.severity == FindingSeverity::Warn)
        );
        assert_eq!(
            report.findings.last().map(|finding| finding.severity),
            Some(FindingSeverity::Info)
        );
    }

    #[test]
    fn supports_caret_requirements_for_detected_versions() {
        assert!(version_matches("^3.11", "3.11.0"));
        assert!(version_matches("^3.11", "3.12.4"));
        assert!(!version_matches("^3.11", "4.0.0"));
        assert!(version_matches("^0.6.0", "0.6.4"));
        assert!(!version_matches("^0.6.0", "0.7.0"));
        assert!(version_matches("22 || 24", "24.16.0"));
        assert!(!version_matches("22 || 24", "23.4.0"));
        assert!(version_matches(
            ">=22.0.0, <23.0.0 || >=24.0.0, <25.0.0",
            "24.16.0"
        ));
        assert!(version_matches("<=21", "21"));
        assert!(version_matches("<21", "20.9"));
        assert!(version_matches(">21", "21.1"));
        assert!(!version_matches("<=21", "25.0.2"));
        assert!(version_matches(">=go1.2.1", "go1.24.2"));
        assert!(version_matches("1.26.3", "go1.26.3"));
    }

    #[test]
    fn maps_maven_tool_to_mvn_executable() {
        assert_eq!(tool_executable_name("maven"), "mvn");
        assert_eq!(tool_executable_name("bundler"), "bundle");
        assert_eq!(tool_executable_name("cargo"), "cargo");
    }

    #[test]
    fn selected_toolchain_run_fulfillment_source_matches_owned_tool_aliases() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: athena-api
toolchains:
  ruby:
    version: "3.3.11"
    package_managers:
      bundler: "2.5.3"
    fulfillment:
      source: ruby
      mode: run
tasks:
  install:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: bundler
        cwd: .
        path: vendor/bundle
    requirements:
      toolchains:
        - ruby
    effects:
      writes:
        - .bundle
        - vendor/bundle
      network: true
      network_kind: dependency_hydration
"#,
        )
        .unwrap();

        let source = super::selected_toolchain_run_fulfillment_source_for_tool(
            &contract,
            &BTreeSet::from([String::from("ruby")]),
            "macos",
            "bundle",
        );

        assert_eq!(source, Some(ToolchainFulfillmentSource::Ruby));
    }

    #[test]
    fn selected_toolchain_run_fulfillment_source_supports_poetry_under_python_toolchain() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: openhands
toolchains:
  python:
    provider: uv
    version: "3.12"
    package_managers:
      poetry: ">=1.8"
    fulfillment:
      source: uv
      mode: run
tasks:
  test:
    run: poetry run pytest
    requirements:
      toolchains:
        - python
"#,
        )
        .unwrap();

        let source = super::selected_toolchain_run_fulfillment_source_for_tool(
            &contract,
            &BTreeSet::from([String::from("python")]),
            "macos",
            "poetry",
        );

        assert_eq!(source, Some(ToolchainFulfillmentSource::Uv));
    }

    #[test]
    fn reports_missing_toolchain_component_for_rustup_toolchain() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "rustc",
            if cfg!(windows) {
                "@echo off\r\necho rustc 1.94.0 (abc123 2026-01-01)\r\n"
            } else {
                "#!/bin/sh\necho 'rustc 1.94.0 (abc123 2026-01-01)'\n"
            },
        );
        write_fake_command(
            &bin_dir,
            "rustup",
            if cfg!(windows) {
                "@echo off\r\nif \"%1\"==\"component\" if \"%2\"==\"list\" if \"%3\"==\"--installed\" (\r\n  echo clippy-x86_64-pc-windows-msvc (installed)\r\n  exit /b 0\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n"
            } else {
                "#!/bin/sh\nif [ \"$1\" = \"component\" ] && [ \"$2\" = \"list\" ] && [ \"$3\" = \"--installed\" ]; then\n  echo 'clippy-x86_64-unknown-linux-gnu (installed)'\n  exit 0\nfi\necho unsupported >&2\nexit 1\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        unsafe {
            env::set_var("PATH", env::join_paths(path_entries).unwrap());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
    components:
      - rustfmt
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Missing toolchain component: rust.rustfmt"
                    || finding.summary == "Missing toolchain provider: rustup"
            })
            .expect("expected rustfmt component or missing provider finding");
        assert!(matches!(
            finding.code(),
            "OTA_TOOLCHAIN_COMPONENT_MISSING" | "OTA_TOOLCHAIN_PROVIDER_MISSING"
        ));
        assert_eq!(finding.category(), "environment");
        assert_eq!(finding.owner(), "host");
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert!(
            finding.next.contains("rustup component add rustfmt")
                || finding.next.contains("install `rustup`"),
            "{finding:?}"
        );
    }

    #[test]
    fn composes_docker_compose_healthcheck_command_inside_container() {
        assert_eq!(
            compose_service_healthcheck_command("postgres", "pg_isready -U qredex -d qredex"),
            "docker compose exec -T postgres sh -lc 'pg_isready -U qredex -d qredex'"
        );
    }

    #[test]
    fn compose_service_manager_wraps_healthcheck_in_declared_project_and_file() {
        let service = ServiceSpec {
            manager: Some(crate::schema::ServiceManagerSpec {
                kind: crate::schema::ServiceManagerKind::Compose,
                engine: crate::schema::ComposeCliEngine::Docker,
                name: Some(String::from("local")),
                file: Some(String::from("compose.yaml")),
                files: Vec::new(),
                env_file: None,
                env_files: Vec::new(),
                profiles: Vec::new(),
                service: Some(String::from("postgres")),
                host: None,
                start: None,
                stop: None,
            }),
            ..ServiceSpec::default()
        };

        assert_eq!(
            service.healthcheck_command("postgres", "pg_isready -U qredex -d qredex"),
            "docker compose -f 'compose.yaml' -p 'local' exec -T 'postgres' sh -lc 'pg_isready -U qredex -d qredex'"
        );
        assert_eq!(
            service.start_command("postgres").as_deref(),
            Some("docker compose -f 'compose.yaml' -p 'local' up -d 'postgres'")
        );
    }

    #[test]
    fn host_service_manager_keeps_healthcheck_on_host_without_derived_start_command() {
        let service = ServiceSpec {
            manager: Some(crate::schema::ServiceManagerSpec {
                kind: crate::schema::ServiceManagerKind::Host,
                engine: crate::schema::ComposeCliEngine::Docker,
                name: Some(String::from("local-postgres")),
                file: None,
                files: Vec::new(),
                env_file: None,
                env_files: Vec::new(),
                profiles: Vec::new(),
                service: None,
                host: None,
                start: None,
                stop: None,
            }),
            ..ServiceSpec::default()
        };

        assert_eq!(
            service.healthcheck_command("postgres", "pg_isready -h 127.0.0.1 -p 5432"),
            "pg_isready -h 127.0.0.1 -p 5432"
        );
        assert_eq!(service.start_command("postgres"), None);
        assert_eq!(service.stop_command("postgres"), None);
    }

    #[test]
    fn reports_invalid_org_policy_pack() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
unexpected: true
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        assert!(!report.ok, "{report:?}");
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].summary, "Invalid org policy pack");
        assert_eq!(report.findings[0].code(), "OTA_POLICY_PACK_INVALID");
        assert_eq!(report.findings[0].category(), "policy");
        assert_eq!(report.findings[0].owner(), "org_policy");
        assert!(report.findings[0].why.contains("org-policy.yaml"));
    }

    #[test]
    fn reports_policy_backed_provisioning_sources_as_info() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  java: "22"
tools:
  maven: "3.9"
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
  provisioning:
    java:
      source: sdkman
      approved_versions:
        - "22"
    maven:
      source: sdkman
      approved_versions:
        - "3.9"
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.code() == "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
                    || finding.code() == "OTA_POLICY_BACKED_VERSION_RULES_DECLARED"
            })
            .expect("policy-backed provisioning finding should be present");
        assert_eq!(finding.category(), "policy");
        assert_eq!(finding.owner(), "org_policy");
        assert_eq!(finding.severity, FindingSeverity::Info);
        assert!(finding.why.contains("java via sdkman"));
        assert!(finding.why.contains("runtime java 22 via sdkman"));
        assert!(finding.why.contains("tool maven 3.9 via sdkman"));
    }

    #[test]
    fn does_not_report_python_managed_toolchain_opportunity_when_uv_provider_is_shipped() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  python:
    provider: uv
    version: "3.12"
tasks:
  test:
    run: uv run pytest
"#,
        )
        .unwrap();
        fs::write(
            fixture.path().join("pyproject.toml"),
            "[project]\nname = 'demo'\n",
        )
        .unwrap();
        fs::write(fixture.path().join("uv.lock"), "version = 1\n").unwrap();

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Managed toolchain opportunity: python")
        );
    }

    #[test]
    fn finding_code_classifies_contract_advisory_service_shell_start() {
        let finding = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "task `dev` uses opaque shell `script` for long-running service path `tasks.dev.script`",
            ),
            why: String::from("service launch is hidden in shell"),
            next: String::from("move to launch.kind: command"),
        };

        assert_eq!(
            finding.code(),
            "OTA_CONTRACT_ADVISORY_SERVICE_OPAQUE_SHELL_START"
        );
        assert_eq!(finding.category(), "contract");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn finding_code_classifies_contract_advisory_replaceable_finite_shell_command() {
        let finding = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from("task `lint` uses replaceable shell `run` instead of `command`"),
            why: String::from("finite argv command kept in shell"),
            next: String::from("replace `tasks.lint.run` with `tasks.lint.command`"),
        };

        assert_eq!(
            finding.code(),
            "OTA_CONTRACT_ADVISORY_REPLACEABLE_FINITE_SHELL_COMMAND"
        );
        assert_eq!(finding.category(), "contract");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn finding_code_classifies_contract_advisory_exceptional_dependency_hydration_override() {
        let finding = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "task `install` declares exceptional dependency hydration override `--force` for `npm`",
            ),
            why: String::from("resolver override weakens normal package-manager safety"),
            next: String::from("keep only when repo truth requires it"),
        };

        assert_eq!(
            finding.code(),
            "OTA_CONTRACT_ADVISORY_EXCEPTIONAL_DEPENDENCY_HYDRATION_OVERRIDE"
        );
        assert_eq!(finding.category(), "contract");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn finding_code_classifies_contract_advisory_agent_safe_network_and_external_state() {
        let network = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from("Agent-safe task `setup` requires network access"),
            why: String::from("networked task"),
            next: String::from("keep effects explicit"),
        };
        let external_state = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from("Agent-safe task `setup` mutates external state: postgres"),
            why: String::from("external state mutation"),
            next: String::from("remove from safe tasks"),
        };

        assert_eq!(
            network.code(),
            "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_NETWORK"
        );
        assert_eq!(
            external_state.code(),
            "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_EXTERNAL_STATE"
        );
    }

    #[test]
    fn finding_code_classifies_contract_advisory_managed_isolated_path() {
        let finding = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from("Task `build` mutates managed isolated path `.next`"),
            why: String::from("managed isolated path mutation"),
            next: String::from("let ota own the path"),
        };

        assert_eq!(
            finding.code(),
            "OTA_CONTRACT_ADVISORY_TASK_MUTATES_MANAGED_ISOLATED_PATH"
        );
        assert_eq!(finding.category(), "contract");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn finding_code_classifies_adapter_ownership_contract_advisories() {
        let bake = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from("task `build` hard-codes Bake file selection in its task body"),
            why: String::from("bake file selection is hidden in shell"),
            next: String::from("move ownership under adapter_inputs.bake.files"),
        };
        let wrong_platform_manager = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "native prerequisite `native-build-tools` platform `linux` declares likely wrong-OS package manager `winget`",
            ),
            why: String::from("winget is likely the wrong lane for linux"),
            next: String::from("move to the correct OS package-manager lane"),
        };
        let mixed_native_package_ownership = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "native prerequisite `native-build-tools` platform `linux` mixes manual install glue with manager-owned package truth",
            ),
            why: String::from("package truth is split across shell glue and apt"),
            next: String::from("separate opaque install glue from manager-owned package truth"),
        };
        let workflow_compose_env_files = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "workflow `compose` duplicates compose `env_files` ownership in task `build`",
            ),
            why: String::from("workflow and task both own the same adapter input"),
            next: String::from("keep adapter ownership in one declarative place"),
        };
        let workflow_compose_files = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "workflow `compose` duplicates compose `files` ownership in task `build`",
            ),
            why: String::from("workflow and task both own the same adapter input"),
            next: String::from("keep adapter ownership in one declarative place"),
        };
        let workflow_compose_profiles = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "workflow `compose` duplicates compose `profiles` ownership in task `build`",
            ),
            why: String::from("workflow and task both own the same adapter input"),
            next: String::from("keep adapter ownership in one declarative place"),
        };
        let workflow_compose_project_name = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "workflow `compose` duplicates compose `project_name` ownership in task `build`",
            ),
            why: String::from("workflow and task both own the same adapter input"),
            next: String::from("keep adapter ownership in one declarative place"),
        };
        let workflow_bake_files = Finding {
            identity: None,
            severity: FindingSeverity::Warn,
            summary: String::from(
                "workflow `image` duplicates bake `files` ownership in task `build`",
            ),
            why: String::from("workflow and task both own the same adapter input"),
            next: String::from("keep adapter ownership in one declarative place"),
        };

        assert_eq!(
            bake.code(),
            "OTA_CONTRACT_ADVISORY_REPLACEABLE_BAKE_FILE_OWNERSHIP"
        );
        assert_eq!(
            wrong_platform_manager.code(),
            "OTA_CONTRACT_ADVISORY_NATIVE_PACKAGE_MANAGER_LIKELY_WRONG_PLATFORM"
        );
        assert_eq!(
            mixed_native_package_ownership.code(),
            "OTA_CONTRACT_ADVISORY_MIXED_NATIVE_PACKAGE_OWNERSHIP"
        );
        assert_eq!(
            workflow_compose_env_files.code(),
            "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_ENV_FILES_OWNERSHIP"
        );
        assert_eq!(
            workflow_compose_files.code(),
            "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_FILES_OWNERSHIP"
        );
        assert_eq!(
            workflow_compose_profiles.code(),
            "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROFILES_OWNERSHIP"
        );
        assert_eq!(
            workflow_compose_project_name.code(),
            "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROJECT_NAME_OWNERSHIP"
        );
        assert_eq!(
            workflow_bake_files.code(),
            "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_BAKE_FILES_OWNERSHIP"
        );
        assert_eq!(bake.category(), "contract");
        assert_eq!(workflow_compose_project_name.owner(), "repo_contract");
    }

    #[test]
    fn finding_identity_overrides_summary_derived_code() {
        let finding = Finding {
            identity: Some(super::FindingIdentity {
                code: String::from("OTA_CONTRACT_ADVISORY_SERVICE_OPAQUE_SHELL_START"),
                category: String::from("contract"),
                owner: String::from("repo_contract"),
            }),
            severity: FindingSeverity::Warn,
            summary: String::from("custom summary"),
            why: String::from("custom why"),
            next: String::from("custom next"),
        };

        assert_eq!(
            finding.code(),
            "OTA_CONTRACT_ADVISORY_SERVICE_OPAQUE_SHELL_START"
        );
        assert_eq!(finding.category(), "contract");
        assert_eq!(finding.owner(), "repo_contract");
    }

    #[test]
    fn doctor_json_omits_toolchain_opportunity_agent_metadata_when_none_are_shipped() {
        let finding = Finding {
            identity: None,
            severity: FindingSeverity::Info,
            summary: String::from("Repository Ready"),
            why: String::from("all selected-path preconditions are satisfied"),
            next: String::from("run ota up when you want ota to prepare the repo path"),
        };

        let json = serde_json::to_value(&finding).expect("finding should serialize");
        assert!(json.get("toolchain_opportunity").is_none(), "{json}");
    }

    #[test]
    fn doctor_json_contract_pack_snapshots_representative_finding_identity_and_metadata() {
        let mut findings = Vec::new();

        let policy_fixture = TempDir::new().unwrap();
        fs::write(
            policy_fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
"#,
        )
        .unwrap();
        fs::create_dir_all(policy_fixture.path().join(".ota")).unwrap();
        fs::write(
            policy_fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
unexpected: true
"#,
        )
        .unwrap();
        let policy_contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(policy_fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();
        let policy_report =
            diagnose_preconditions(&policy_contract, &policy_fixture.path().join("ota.yaml"));
        let policy_finding = policy_report
            .findings
            .iter()
            .find(|finding| finding.summary == "Invalid org policy pack")
            .expect("policy finding should be present");
        findings.push(finding_contract_projection("policy", policy_finding));

        let workflow_listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let workflow_port = workflow_listener.local_addr().unwrap().port();
        let workflow_server = thread::spawn(move || {
            let (mut stream, _) = workflow_listener.accept().expect("probe should connect");
            drain_probe_request_if_available(&mut stream);
            stream
                .write_all(
                    b"HTTP/1.1 503 Service Unavailable\r\nContent-Length: 0\r\nConnection: close\r\n\r\n",
                )
                .expect("probe response should write");
        });

        let workflow_contract = parse_contract_str(
            synthetic_contract_path(),
            format!(
                r#"
version: 1
project:
  name: ota
readiness:
  probes:
    backend-ready:
      kind: http
      url: http://127.0.0.1:{workflow_port}/healthz/readiness
      # Windows CI can take longer to establish local-loopback probe
      # handshakes under Git Bash/runner scheduling; keep above jitter.
      timeout: 5000
workflows:
  default: backend
  backend:
    readiness:
      signal:
        probes:
          - backend-ready
"#,
            )
            .as_str(),
        )
        .unwrap();
        let workflow_report = super::diagnose_checks_only_for_workflow(
            &workflow_contract,
            synthetic_contract_path(),
            Some("backend"),
        );
        workflow_server.join().expect("probe server should finish");
        let workflow_finding = workflow_report
            .findings
            .iter()
            .find(|finding| finding.summary == "Signal probe failed: backend-ready")
            .expect("workflow signal finding should be present");
        findings.push(finding_contract_projection("workflow", workflow_finding));

        let service_contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
execution:
  default_context: host
  contexts:
    host:
      backend: native
services:
  postgres:
    required: true
    start: docker compose up -d postgres
    endpoints:
      host:
        address: 127.0.0.1
        port: 5432
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();
        let service_report = diagnose_contract(&service_contract, synthetic_contract_path());
        let service_finding = service_report
            .findings
            .iter()
            .find(|finding| finding.summary == "Required service cannot be verified: postgres")
            .expect("service finding should be present");
        findings.push(finding_contract_projection("service", service_finding));

        let env_fixture = TempDir::new().unwrap();
        let env_contract_path = env_fixture.path().join("ota.yaml");
        let env_contract = parse_contract_str(
            &env_contract_path,
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_REQUIRED:
      required: true
  sources:
    - kind: dotenv
      path: .env
      must_exist: true
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();
        let env_report = diagnose_contract(&env_contract, &env_contract_path);
        let env_finding = env_report
            .findings
            .iter()
            .find(|finding| finding.summary == "Missing required environment source: dotenv:.env")
            .expect("env source finding should be present");
        findings.push(finding_contract_projection("env", env_finding));

        let native_package_policy_fixture = TempDir::new().unwrap();
        let (_, native_package_source, native_package_name, _) =
            write_native_package_policy_fixture(&native_package_policy_fixture, "libpq-dev");
        let native_package_policy_contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(native_package_policy_fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();
        let native_package_policy_report = diagnose_preconditions(
            &native_package_policy_contract,
            &native_package_policy_fixture.path().join("ota.yaml"),
        );
        let native_package_policy_finding = native_package_policy_report
            .findings
            .iter()
            .find(|finding| {
                finding.summary
                    == format!(
                        "Org policy does not approve native package: {native_package_source}:{native_package_name}"
                    )
            })
            .expect("native package policy finding should be present");
        findings.push(finding_contract_projection(
            "native_package_policy",
            native_package_policy_finding,
        ));

        let contract_bake_advisory = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  image:build:
    run: docker buildx bake -f docker-bake.hcl app
"#,
        )
        .unwrap();
        let contract_bake_report =
            diagnose_contract(&contract_bake_advisory, synthetic_contract_path());
        let contract_bake_finding = contract_bake_report
            .findings
            .iter()
            .find(|finding| {
                finding.summary
                    == "task `image:build` hard-codes Bake file selection in its task body"
            })
            .expect("replaceable bake advisory should be present");
        findings.push(finding_contract_projection(
            "contract_bake",
            contract_bake_finding,
        ));

        let contract_workflow_advisory = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  dev:
    adapter_inputs:
      compose:
        files:
          - compose.local.yaml
    run: docker compose up
workflows:
  default: compose
  compose:
    env:
      adapter_inputs:
        compose:
          files:
            - compose.dev.yaml
    run:
      task: dev
"#,
        )
        .unwrap();
        let contract_workflow_report =
            diagnose_contract(&contract_workflow_advisory, synthetic_contract_path());
        let contract_workflow_finding = contract_workflow_report
            .findings
            .iter()
            .find(|finding| {
                finding.summary
                    == "workflow `compose` duplicates compose `files` ownership in task `dev`"
            })
            .expect("workflow adapter-input duplicate advisory should be present");
        findings.push(finding_contract_projection(
            "contract_workflow",
            contract_workflow_finding,
        ));

        let contract_native_manager_advisory = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  native-build-tools:
    platforms:
      linux:
        check: native-build-tools-linux
        winget:
          - Microsoft.VisualStudio.2022.BuildTools
checks:
  - name: native-build-tools-linux
    kind: precondition
    severity: error
    run: sh -c "cc --version"
"#,
        )
        .unwrap();
        let contract_native_manager_report =
            diagnose_contract(&contract_native_manager_advisory, synthetic_contract_path());
        let contract_native_manager_finding = contract_native_manager_report
            .findings
            .iter()
            .find(|finding| {
                finding.summary
                    == "native prerequisite `native-build-tools` platform `linux` declares likely wrong-OS package manager `winget`"
            })
            .expect("wrong-platform native package manager advisory should be present");
        findings.push(finding_contract_projection(
            "contract_native_manager",
            contract_native_manager_finding,
        ));

        let contract_mixed_native_ownership_advisory = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
native_prerequisites:
  native-build-tools:
    platforms:
      linux:
        check: native-build-tools-linux
        install: ./scripts/bootstrap-native.sh
        apt:
          - build-essential
checks:
  - name: native-build-tools-linux
    kind: precondition
    severity: error
    run: sh -c "cc --version"
"#,
        )
        .unwrap();
        let contract_mixed_native_ownership_report = diagnose_contract(
            &contract_mixed_native_ownership_advisory,
            synthetic_contract_path(),
        );
        let contract_mixed_native_ownership_finding =
            contract_mixed_native_ownership_report
                .findings
                .iter()
                .find(|finding| {
                    finding.summary
                        == "native prerequisite `native-build-tools` platform `linux` mixes manual install glue with manager-owned package truth"
                })
                .expect("mixed native package ownership advisory should be present");
        findings.push(finding_contract_projection(
            "contract_mixed_native_ownership",
            contract_mixed_native_ownership_finding,
        ));

        let provisioning_finding = super::provisioning_installability_finding(
            &ProvisioningFailureDiagnosis {
                backend: String::from("apt"),
                target_kind: ProvisioningTargetKind::Tool,
                name: String::from("jq"),
                requested_version: String::from("jq-1.8.1"),
                resolved_version: None,
                policy_match: None,
                kind: ProvisioningFailureKind::PackageUnavailable,
            },
            &ProvisioningExecutionTarget::Native,
            "ota doctor",
        );
        findings.push(finding_contract_projection(
            "provisioning",
            &provisioning_finding,
        ));

        let remote_contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();
        let remote_report = diagnose_contract_in_mode(
            &remote_contract,
            synthetic_contract_path(),
            DoctorMode::Remote,
        );
        let remote_finding = remote_report
            .findings
            .iter()
            .find(|finding| finding.summary == "Remote execution is not configured")
            .expect("remote finding should be present");
        findings.push(finding_contract_projection("remote", remote_finding));

        assert_eq!(
            findings,
            vec![
                serde_json::json!({
                    "lane": "policy",
                    "code": "OTA_POLICY_PACK_INVALID",
                    "category": "policy",
                    "owner": "org_policy",
                    "severity": "error",
                    "summary": "Invalid org policy pack",
                    "evidence_source": "org_policy",
                    "provenance_key": "org_policy",
                    "policy_reason": "invalid_org_policy_pack",
                }),
                serde_json::json!({
                    "lane": "workflow",
                    "code": "OTA_WORKFLOW_SIGNAL_PROBE_FAILED",
                    "category": "execution",
                    "owner": "repo_contract",
                    "severity": "info",
                    "summary": "Signal probe failed: backend-ready",
                    "evidence_source": "execution",
                    "provenance_key": "repo_contract",
                    "policy_reason": serde_json::Value::Null,
                }),
                serde_json::json!({
                    "lane": "service",
                    "code": "OTA_SERVICE_UNVERIFIABLE",
                    "category": "service",
                    "owner": "service",
                    "severity": "warn",
                    "summary": "Required service cannot be verified: postgres",
                    "evidence_source": "service",
                    "provenance_key": "repo_contract",
                    "policy_reason": serde_json::Value::Null,
                }),
                serde_json::json!({
                    "lane": "env",
                    "code": "OTA_ENV_SOURCE_MISSING_REQUIRED",
                    "category": "environment",
                    "owner": "repo_contract",
                    "severity": "error",
                    "summary": "Missing required environment source: dotenv:.env",
                    "evidence_source": "repo filesystem",
                    "provenance_key": "repo_contract",
                    "policy_reason": serde_json::Value::Null,
                }),
                serde_json::json!({
                    "lane": "native_package_policy",
                    "code": "OTA_POLICY_NATIVE_PACKAGE_NOT_APPROVED",
                    "category": "policy",
                    "owner": "org_policy",
                    "severity": "error",
                    "summary": format!(
                        "Org policy does not approve native package: {native_package_source}:{native_package_name}"
                    ),
                    "evidence_source": "org_policy",
                    "provenance_key": "org_policy",
                    "policy_reason": "native_package_not_approved",
                }),
                serde_json::json!({
                    "lane": "contract_bake",
                    "code": "OTA_CONTRACT_ADVISORY_REPLACEABLE_BAKE_FILE_OWNERSHIP",
                    "category": "contract",
                    "owner": "repo_contract",
                    "severity": "warn",
                    "summary": "task `image:build` hard-codes Bake file selection in its task body",
                    "evidence_source": "doctor",
                    "provenance_key": "repo_contract",
                    "policy_reason": serde_json::Value::Null,
                }),
                serde_json::json!({
                    "lane": "contract_workflow",
                    "code": "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_FILES_OWNERSHIP",
                    "category": "contract",
                    "owner": "repo_contract",
                    "severity": "warn",
                    "summary": "workflow `compose` duplicates compose `files` ownership in task `dev`",
                    "evidence_source": "doctor",
                    "provenance_key": "repo_contract",
                    "policy_reason": serde_json::Value::Null,
                }),
                serde_json::json!({
                    "lane": "contract_native_manager",
                    "code": "OTA_CONTRACT_ADVISORY_NATIVE_PACKAGE_MANAGER_LIKELY_WRONG_PLATFORM",
                    "category": "contract",
                    "owner": "repo_contract",
                    "severity": "warn",
                    "summary": "native prerequisite `native-build-tools` platform `linux` declares likely wrong-OS package manager `winget`",
                    "evidence_source": "doctor",
                    "provenance_key": "repo_contract",
                    "policy_reason": serde_json::Value::Null,
                }),
                serde_json::json!({
                    "lane": "contract_mixed_native_ownership",
                    "code": "OTA_CONTRACT_ADVISORY_MIXED_NATIVE_PACKAGE_OWNERSHIP",
                    "category": "contract",
                    "owner": "repo_contract",
                    "severity": "warn",
                    "summary": "native prerequisite `native-build-tools` platform `linux` mixes manual install glue with manager-owned package truth",
                    "evidence_source": "doctor",
                    "provenance_key": "repo_contract",
                    "policy_reason": serde_json::Value::Null,
                }),
                serde_json::json!({
                    "lane": "provisioning",
                    "code": "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE",
                    "category": "provisioning",
                    "owner": "host",
                    "severity": "error",
                    "summary": "Host apt cannot locate required package: jq",
                    "evidence_source": "host_provisioning",
                    "provenance_key": "org_policy",
                    "policy_reason": serde_json::Value::Null,
                }),
                serde_json::json!({
                    "lane": "remote",
                    "code": "OTA_REMOTE_MODE_NOT_CONFIGURED",
                    "category": "remote",
                    "owner": "repo_contract",
                    "severity": "error",
                    "summary": "Remote execution is not configured",
                    "evidence_source": "repo_contract",
                    "provenance_key": "repo_contract",
                    "policy_reason": serde_json::Value::Null,
                }),
            ]
        );
    }

    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    struct DoctorFindingReferenceEntry {
        code: &'static str,
        category: &'static str,
        owner_surface: &'static str,
        provenance_key_surface: &'static str,
    }

    fn shipped_doctor_finding_reference_entries() -> &'static [DoctorFindingReferenceEntry] {
        &[
            DoctorFindingReferenceEntry {
                code: "OTA_AGENT_BOUNDARY_UNREVIEWED",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_AGENT_BOOTSTRAP_UNPINNED",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_ENSURE_GIT_CHECKOUT_MOVING_HEAD",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_DEPENDENCY_HYDRATION",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_EXTERNAL_STATE",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_NETWORK",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_AGENT_SAFE_TASK_INTEGRATION_TEST",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_EXTERNAL_STATE_TOKEN_CANONICAL",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_DEPENDS_ON_BOUNDARY",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_BAKE_FILES_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_ENV_FILES_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_FILES_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROFILES_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_COMPOSE_PROJECT_NAME_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_DUPLICATE_WORKFLOW_RENDERED_ENV_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_EXCEPTIONAL_DEPENDENCY_HYDRATION_OVERRIDE",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_ISOLATED_YARN_RELEASE_SHADOW",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_LEGACY_NODE_RUNTIME_TOOL_SPLIT",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_LEGACY_STANDALONE_POETRY",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_LIKELY_UNUSED_ATTACHMENT",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_MIXED_NATIVE_PACKAGE_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_NATIVE_PACKAGE_MANAGER_LIKELY_WRONG_PLATFORM",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_REPLACEABLE_BAKE_FILE_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_REPLACEABLE_COMPOSE_ENV_FILE_OWNERSHIP",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_REPLACEABLE_DEPENDENCY_HYDRATION",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_REPLACEABLE_FINITE_SHELL_COMMAND",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_ENV_CHECK",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_ENV_MUTATION",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_REPLACEABLE_SHELL_FILE_CHECK",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_SENSITIVE_AGENT_WRITABLE_PATH",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_SENSITIVE_WRITE_EXCEPTION",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_SERVICE_OPAQUE_SHELL_START",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_ADVISORY_TASK_MUTATES_MANAGED_ISOLATED_PATH",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACT_DRIFT",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_signals",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACTLESS_REPO_CONTRACT_MISSING",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_signals",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACTLESS_SIGNAL",
                category: "contract",
                owner_surface: "repo_signals",
                provenance_key_surface: "repo_signals",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACTLESS_SIGNAL_INSPECTION_FAILED",
                category: "contract",
                owner_surface: "repo_signals",
                provenance_key_surface: "repo_signals",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_DEVCONTAINER_PACKAGE_MANAGER_DRIFT",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_DEVCONTAINER_RUNTIME_DRIFT",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REPO_HYGIENE_GITIGNORE_UNREADABLE",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REPO_HYGIENE_OTA_STATE_GITIGNORE",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_SELECTED_TASK_PATH_DEPENDENCY_HYDRATION",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_SELECTED_TASK_PATH_EXTERNAL_STATE",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_SELECTED_TASK_PATH_NETWORK_REQUIRED",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TASKS_MISSING",
                category: "contract",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_BACKEND_CLI_MISSING",
                category: "execution",
                owner_surface: "host",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CHECK_FAILED",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CHECK_TIMED_OUT",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_BACKEND_CLI_MISSING",
                category: "execution",
                owner_surface: "host",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_BACKEND_UNAVAILABLE",
                category: "execution",
                owner_surface: "host",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_DOCTOR_HOST_SCOPE_NOTE",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_IMAGE_UNAVAILABLE",
                category: "execution",
                owner_surface: "container_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_MODE_NOT_CONFIGURED",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTEXT_HOST_PLATFORM_UNSUPPORTED",
                category: "execution",
                owner_surface: "host",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_FILE_CHECK_FAILED",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_FILE_CHECK_TIMED_OUT",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_LIFECYCLE_EPHEMERAL_ADVISORY",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_PROBE_FAILED",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_PROBE_TIMED_OUT",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_SIGNAL_PROBE_FAILED",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_SIGNAL_PROBE_TIMED_OUT",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_FAILED",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_TIMED_OUT",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_SIGNAL_SURFACE_READINESS_UNEVALUABLE",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_SURFACE_READINESS_FAILED",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_SURFACE_READINESS_TIMED_OUT",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKFLOW_SURFACE_READINESS_UNEVALUABLE",
                category: "execution",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED",
                category: "remote",
                owner_surface: "remote_backend",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_CONTEXT_UNEXECUTABLE",
                category: "remote",
                owner_surface: "remote_backend",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_DOCTOR_HOST_SCOPE_NOTE",
                category: "remote",
                owner_surface: "remote_backend",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_DOCTOR_PARTIAL",
                category: "remote",
                owner_surface: "remote_backend",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_MODE_NOT_CONFIGURED",
                category: "remote",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_TARGET_OS_UNDETERMINED",
                category: "remote",
                owner_surface: "remote_backend",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_TARGET_SUSPICIOUS",
                category: "remote",
                owner_surface: "remote_backend",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_SERVICE_CHECK_FAILED",
                category: "service",
                owner_surface: "service",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_SERVICE_CHECK_TIMED_OUT",
                category: "service",
                owner_surface: "service",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_SERVICE_READINESS_CONTEXT_UNEXECUTABLE",
                category: "service",
                owner_surface: "service",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_SERVICE_READINESS_FAILED",
                category: "service",
                owner_surface: "service",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_SERVICE_UNVERIFIABLE",
                category: "service",
                owner_surface: "service",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_ENV_INVALID",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_ENV_MISSING",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_ENV_SOURCE_INVALID_STRUCTURE",
                category: "environment",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_ENV_SOURCE_KEY_COLLISION",
                category: "environment",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_ENV_SOURCE_MISSING_REQUIRED",
                category: "environment",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_ENV_SOURCE_PARSE_FAILED",
                category: "environment",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACTLESS_HOST_TOOL_AVAILABLE",
                category: "environment",
                owner_surface: "host",
                provenance_key_surface: "repo_signals",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTRACTLESS_HOST_TOOL_MISSING",
                category: "environment",
                owner_surface: "host",
                provenance_key_surface: "repo_signals",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_NATIVE_PREREQUISITE_MISSING",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_NATIVE_PREREQUISITE_TIMED_OUT",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_RUNTIME_MISSING",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_RUNTIME_PROBE_FAILED",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_RUNTIME_VERSION_MISMATCH",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_RUNTIME_VERSION_UNPARSEABLE",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOLCHAIN_COMPONENT_MISSING",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_signals",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOLCHAIN_PROVIDER_MISSING",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOLCHAIN_PROVIDER_PROBE_FAILED",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOLCHAIN_TARGET_MISSING",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOL_ACTIVATION_PROVIDER_MISSING",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOL_MISSING",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOL_PROBE_FAILED",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOL_VERSION_MISMATCH",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_TOOL_VERSION_UNPARSEABLE",
                category: "environment",
                owner_surface: "host|container_target|remote_target",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_APT_INDEX_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "container_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "container_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_APT_VERSION_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "container_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_PROVISIONING_BACKEND_FAILED",
                category: "provisioning",
                owner_surface: "container_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "container_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "container_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "container_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_HOST_PROVISIONING_BACKEND_FAILED",
                category: "provisioning",
                owner_surface: "host",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "host",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "host",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "host",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_ADAPTER_BOOTSTRAP_FAILED",
                category: "provisioning",
                owner_surface: "repo_contract",
                provenance_key_surface: "repo_contract",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_APT_INDEX_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "remote_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_APT_PACKAGE_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "remote_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_APT_VERSION_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "remote_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_PROVISIONING_BACKEND_FAILED",
                category: "provisioning",
                owner_surface: "remote_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "remote_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "remote_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE",
                category: "provisioning",
                owner_surface: "remote_target",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_EFFECT_ALLOWED",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_EFFECT_DENIED",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_EFFECT_WARNED",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_BACKED_PROVISIONING_DECLARED",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_BACKED_VERSION_RULES_DECLARED",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_INSTALLED_VERSION_NONCOMPLIANT",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_PACK_INVALID",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_PACK_VIOLATION",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_NATIVE_PACKAGE_NOT_APPROVED",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING",
                category: "policy",
                owner_surface: "org_policy",
                provenance_key_surface: "org_policy",
            },
        ]
    }

    fn shipped_workspace_finding_reference_entries() -> &'static [DoctorFindingReferenceEntry] {
        &[
            DoctorFindingReferenceEntry {
                code: "OTA_WORKSPACE_REPO_NOT_ACQUIRED",
                category: "workspace",
                owner_surface: "workspace_acquisition",
                provenance_key_surface: "omitted",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKSPACE_REPO_CONTRACT_INVALID",
                category: "workspace",
                owner_surface: "repo_contract",
                provenance_key_surface: "omitted",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKSPACE_REPO_CONTRACT_MISSING",
                category: "workspace",
                owner_surface: "repo_contract",
                provenance_key_surface: "omitted",
            },
            DoctorFindingReferenceEntry {
                code: "OTA_WORKSPACE_REPO_CONTRACT_UNREADABLE",
                category: "workspace",
                owner_surface: "repo_contract",
                provenance_key_surface: "omitted",
            },
        ]
    }

    fn normalize_line_endings(source: String) -> String {
        source.replace("\r\n", "\n")
    }

    fn doctor_production_source() -> String {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/doctor.rs");
        let source = normalize_line_endings(
            fs::read_to_string(&source_path).expect("doctor source should load"),
        );
        let (production_source, _) = source
            .split_once("#[cfg(test)]\nmod tests {")
            .expect("doctor tests module marker should exist");
        production_source.to_string()
    }

    fn commands_production_source() -> String {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/cli/commands.rs");
        let source = normalize_line_endings(
            fs::read_to_string(&source_path).expect("commands source should load"),
        );
        let (production_source, _) = source
            .split_once("#[cfg(test)]\nmod tests {")
            .expect("commands tests module marker should exist");
        production_source.to_string()
    }

    fn extract_ota_codes(source: &str) -> std::collections::BTreeSet<String> {
        let mut codes = std::collections::BTreeSet::new();
        let mut index = 0usize;
        while let Some(offset) = source[index..].find("OTA_") {
            let start = index + offset;
            let mut end = start + 4;
            while end < source.len() {
                let ch = source.as_bytes()[end] as char;
                if ch.is_ascii_uppercase() || ch.is_ascii_digit() || ch == '_' {
                    end += 1;
                } else {
                    break;
                }
            }
            codes.insert(source[start..end].to_string());
            index = end;
        }
        codes
    }

    fn shipped_doctor_finding_codes_from_production_source() -> std::collections::BTreeSet<String> {
        extract_ota_codes(&doctor_production_source())
            .into_iter()
            .filter(|code| {
                code != "OTA_DOCTOR_FINDING_UNKNOWN"
                    && code != "OTA_CONTRACT_ADVISORY_"
                    && super::finding_registry_entry(code).is_some()
            })
            .collect()
    }

    fn shipped_workspace_finding_codes_from_production_source() -> std::collections::BTreeSet<String>
    {
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/workspace.rs");
        let source = fs::read_to_string(&source_path).expect("workspace source should load");
        let diagnose_start = source
            .find("pub(crate) fn diagnose_workspace_repo(")
            .expect("workspace diagnosis entrypoint should exist");
        let repo_finding_start = source[diagnose_start..]
            .find("fn repo_finding(")
            .map(|offset| diagnose_start + offset)
            .expect("workspace repo_finding helper should exist");
        let diagnose_surface = &source[diagnose_start..repo_finding_start];

        let mut codes = std::collections::BTreeSet::new();
        let mut cursor = diagnose_surface;
        while let Some(call_start) = cursor.find("repo_finding(") {
            let after_call = &cursor[call_start + "repo_finding(".len()..];
            let Some(first_quote) = after_call.find('"') else {
                break;
            };
            let after_first_quote = &after_call[first_quote + 1..];
            let Some(second_quote) = after_first_quote.find('"') else {
                break;
            };
            let code = &after_first_quote[..second_quote];
            if code.starts_with("OTA_WORKSPACE_") {
                codes.insert(code.to_string());
            }
            cursor = &after_first_quote[second_quote + 1..];
        }
        codes
    }

    fn shipped_command_finding_codes_from_production_source() -> std::collections::BTreeSet<String>
    {
        extract_ota_codes(&commands_production_source())
            .into_iter()
            .filter(|code| {
                code != "OTA_DOCTOR_FINDING_UNKNOWN"
                    && code != "OTA_CONTRACT_ADVISORY_"
                    && super::finding_registry_entry(code).is_some()
            })
            .collect()
    }

    fn surface_allows(surface: &str, value: &str) -> bool {
        surface.split('|').any(|candidate| candidate == value)
    }

    fn markdown_code_cell(value: &str) -> String {
        format!("`{}`", value.replace('|', "\\|"))
    }

    fn category_heading(category: &str) -> &'static str {
        match category {
            "contract" => "Contract",
            "execution" => "Execution",
            "remote" => "Remote",
            "service" => "Service",
            "environment" => "Environment",
            "provisioning" => "Provisioning",
            "policy" => "Policy",
            "workspace" => "Workspace",
            _ => "Other",
        }
    }

    fn render_doctor_finding_reference_markdown() -> String {
        let mut output = String::from(
            r#"<!--
                █████
               ░░███
       ██████  ███████    ██████
      ███░░███░░░███░    ░░░░░███
     ░███ ░███  ░███      ███████
     ░███ ░███  ░███ ███ ███░░███
     ░░██████   ░░█████ ░░████████
      ░░░░░░     ░░░░░   ░░░░░░░░

   Copyright (C) 2026 — 2026, Ota. All Rights Reserved.

   DO NOT ALTER OR REMOVE COPYRIGHT NOTICES OR THIS FILE HEADER.

   Licensed under the Apache License, Version 2.0. See LICENSE for the full license text.
   You may not use this file except in compliance with that License.
   Unless required by applicable law or agreed to in writing, software distributed under the
   License is distributed on an "AS IS" BASIS, WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND,
   either express or implied. See the License for the specific language governing permissions
   and limitations under the License.

   If you need additional information or have any questions, please email: os@ota.run
-->

# Doctor Finding Reference

Status: generated reference.

This document is generated from the shipped doctor and workspace finding identity catalogs in
`src/doctor.rs` and `src/workspace.rs`.
Do not edit the rows by hand; update the catalog and rerun the sync tests.

`owner_surface` and `provenance_key_surface` use `|` when the emitted value depends on the
selected execution target instead of one fixed owner or provenance lane. `omitted` means the
field is not emitted for that finding family.
"#,
        );
        let categories = [
            "contract",
            "execution",
            "remote",
            "service",
            "environment",
            "provisioning",
            "policy",
            "workspace",
        ];

        for category in categories {
            output.push_str(&format!("\n## {}\n\n", category_heading(category)));
            output.push_str(
                "| Code | Category | Owner Surface | Provenance Key Surface |\n\
| --- | --- | --- | --- |\n",
            );
            for entry in shipped_doctor_finding_reference_entries()
                .iter()
                .chain(shipped_workspace_finding_reference_entries().iter())
                .filter(|entry| entry.category == category)
            {
                output.push_str(&format!(
                    "| {} | {} | {} | {} |\n",
                    markdown_code_cell(entry.code),
                    markdown_code_cell(entry.category),
                    markdown_code_cell(entry.owner_surface),
                    markdown_code_cell(entry.provenance_key_surface),
                ));
            }
        }

        output
    }

    #[test]
    fn shipped_doctor_findings_require_explicit_identity_metadata() {
        let production_source = doctor_production_source();
        let source_path = Path::new(env!("CARGO_MANIFEST_DIR")).join("src/doctor.rs");

        assert!(
            !production_source.contains("identity: None,")
                && !production_source.contains("identity: Default::default()"),
            "shipped doctor findings must carry explicit identity metadata: {}",
            source_path.display()
        );
    }

    #[test]
    fn doctor_finding_reference_catalog_stays_in_sync_with_registry_and_production_source() {
        let reference_codes: std::collections::BTreeSet<_> =
            shipped_doctor_finding_reference_entries()
                .iter()
                .chain(shipped_workspace_finding_reference_entries().iter())
                .map(|entry| entry.code.to_string())
                .collect();

        let production_codes: std::collections::BTreeSet<_> =
            shipped_doctor_finding_codes_from_production_source()
                .into_iter()
                .chain(shipped_command_finding_codes_from_production_source())
                .chain(shipped_workspace_finding_codes_from_production_source())
                .collect();

        assert_eq!(
            reference_codes, production_codes,
            "doctor finding reference catalog must cover the full shipped repo, command, and workspace production surface"
        );

        for entry in shipped_doctor_finding_reference_entries() {
            let finding = Finding {
                identity: Some(super::FindingIdentity::new(
                    entry.code,
                    entry.category,
                    entry
                        .owner_surface
                        .split('|')
                        .next()
                        .unwrap_or("repo_contract"),
                )),
                severity: FindingSeverity::Warn,
                summary: String::from("reference finding"),
                why: String::from("reference finding"),
                next: String::from("reference finding"),
            };
            finding.resolved_metadata().unwrap_or_else(|| {
                panic!("reference entry must resolve via registry: {}", entry.code)
            });

            assert_eq!(finding.category(), entry.category, "{}", entry.code);
            assert!(
                surface_allows(entry.owner_surface, finding.owner()),
                "{} owner mismatch: expected {}, got {}",
                entry.code,
                entry.owner_surface,
                finding.owner()
            );
            assert!(
                surface_allows(
                    entry.provenance_key_surface,
                    finding.provenance_key().as_deref().unwrap_or(""),
                ),
                "{} provenance mismatch: expected {}, got {:?}",
                entry.code,
                entry.provenance_key_surface,
                finding.provenance_key()
            );
        }

        for entry in shipped_workspace_finding_reference_entries() {
            let finding = Finding {
                identity: Some(super::FindingIdentity::new(
                    entry.code,
                    entry.category,
                    entry.owner_surface,
                )),
                severity: FindingSeverity::Warn,
                summary: String::from("reference finding"),
                why: String::from("reference finding"),
                next: String::from("reference finding"),
            };

            assert_eq!(finding.category(), entry.category, "{}", entry.code);
            assert_eq!(finding.owner(), entry.owner_surface, "{}", entry.code);
            assert_eq!(
                finding.provenance_key().as_deref().unwrap_or("omitted"),
                entry.provenance_key_surface,
                "{}",
                entry.code
            );
        }
    }

    #[test]
    fn doctor_finding_reference_markdown_matches_checked_in_doc() {
        let doc_path =
            Path::new(env!("CARGO_MANIFEST_DIR")).join("docs/spec/doctor-finding-reference.md");
        let expected = render_doctor_finding_reference_markdown();
        let actual = normalize_line_endings(
            fs::read_to_string(&doc_path).expect("doctor finding reference should load"),
        );

        assert_eq!(
            actual,
            expected,
            "doctor finding reference doc drifted: {}",
            doc_path.display()
        );
    }

    #[test]
    fn policy_surfaces_include_toolchain_owned_runtime_requirements() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "rustc",
            if cfg!(windows) {
                "@echo off\r\necho rustc 1.94.0\r\nexit /b 0\r\n"
            } else {
                "#!/bin/sh\necho rustc 1.94.0\nexit 0\n"
            },
        );
        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        } else if cfg!(unix) {
            path_entries.push(PathBuf::from("/usr/bin"));
            path_entries.push(PathBuf::from("/bin"));
        }
        unsafe {
            env::set_var("PATH", env::join_paths(path_entries).unwrap());
        }
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
tasks:
  setup:
    run: cargo fetch
    requirements:
      toolchains:
        - rust
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
  version_policy:
    runtimes:
      rust:
        approved_versions:
          - "1.94.0"
  provisioning:
    rust:
      source: mise
      approved_versions:
        - "1.94.0"
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        assert!(report.ok, "{report:?}");

        let version_finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Policy-backed version rules are declared")
            .expect("policy-backed version finding should be present");
        assert_eq!(
            version_finding.code(),
            "OTA_POLICY_BACKED_VERSION_RULES_DECLARED"
        );
        assert_eq!(version_finding.category(), "policy");
        assert_eq!(version_finding.owner(), "org_policy");
        assert_eq!(version_finding.provenance().as_deref(), Some("org policy"));
        assert_eq!(
            version_finding.provenance_key().as_deref(),
            Some("org_policy")
        );
        let version_json = serde_json::to_value(version_finding)
            .expect("policy-backed version finding should serialize");
        assert_eq!(
            version_json
                .get("policy_reason")
                .and_then(|value| value.as_str()),
            Some("policy_backed_version_rules_declared")
        );
        assert!(
            version_finding
                .why
                .contains("runtime rust (versions 1.94.0)"),
            "{version_finding:?}"
        );

        let provisioning_finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Policy-backed provisioning sources are declared")
            .expect("policy-backed provisioning finding should be present");
        assert_eq!(
            provisioning_finding.code(),
            "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
        );
        assert_eq!(provisioning_finding.category(), "policy");
        assert_eq!(provisioning_finding.owner(), "org_policy");
        assert!(
            provisioning_finding
                .why
                .contains("runtime rust 1.94.0 via mise"),
            "{provisioning_finding:?}"
        );
    }

    #[test]
    fn provider_hint_remediation_uses_toolchain_owned_runtime_contracts() {
        assert_eq!(
            provider_hint_remediation(
                ProvisioningTargetKind::Runtime,
                "rust",
                "1.94.0",
                Some("rustup")
            ),
            Some(String::from("rustup toolchain install 1.94.0"))
        );
        assert_eq!(
            provider_hint_remediation(
                ProvisioningTargetKind::Runtime,
                "node",
                "22",
                Some("corepack")
            ),
            None
        );
    }

    #[test]
    fn container_probe_failure_next_step_points_to_hydration_for_repo_dependency_errors() {
        let next = super::container_probe_failure_next_step(
            "bin/brakeman --version",
            "bin/brakeman",
            "ota doctor --mode container --workflow verify-static",
            Some(
                "/usr/local/lib/ruby/3.3.0/bundler/definition.rb:599:in `materialize': Could not find brakeman-8.0.4 in locally installed gems (Bundler::GemNotFound)",
            ),
        );
        assert!(next.contains(
            "hydrate the selected repo dependency lane inside the selected container path first"
        ));
        assert!(next.contains("ota doctor --mode container --workflow verify-static"));
    }

    #[test]
    fn container_probe_failure_next_step_keeps_generic_probe_guidance_without_hydration_signal() {
        let next = super::container_probe_failure_next_step(
            "npm --version",
            "/usr/local/bin/npm",
            "ota doctor --mode container",
            Some("permission denied"),
        );
        assert!(next.contains("run `npm --version` inside the selected container image"));
        assert!(!next.contains("hydrate the selected repo dependency lane"));
    }

    #[test]
    fn reports_policy_provisioning_missing_package_mapping() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  java: "22"
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
  provisioning:
    java:
      source: apt
      approved_versions:
        - "22"
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        assert!(!report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Policy provisioning needs explicit package identifiers"
            })
            .expect("missing package warning should be present");
        assert_eq!(
            finding.code(),
            "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING"
        );
        assert_eq!(finding.category(), "policy");
        assert_eq!(finding.owner(), "org_policy");
        assert_eq!(finding.severity, FindingSeverity::Warn);
        assert!(finding.why.contains("requires an explicit `package`"));
    }

    #[test]
    fn reports_missing_policy_required_sections() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_sections:
    - tasks
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].summary,
            "Repo does not satisfy org policy pack"
        );
        assert!(report.findings[0].why.contains("tasks"));
    }

    #[test]
    fn skips_platform_scoped_tool_when_not_required_on_current_os() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            &format!(
                r#"
version: 1
project:
  name: ota
tools:
  definitely-not-installed:
    version: "1.0.0"
    only_on:
      - {}
tasks:
  test:
    run: cargo test
"#,
                match super::current_os() {
                    "windows" => "linux",
                    _ => "windows",
                }
            ),
        )
        .unwrap();
        assert!(
            !contract
                .tools
                .get("definitely-not-installed")
                .expect("tool requirement should exist")
                .required_for_os(super::current_os())
        );

        let report = diagnose_preconditions(&contract, synthetic_contract_path());
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing tool: definitely-not-installed")
        );
    }

    #[test]
    fn skips_platform_scoped_runtime_when_not_required_on_current_os() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            &format!(
                r#"
version: 1
project:
  name: ota
runtimes:
  definitely-not-installed:
    version: "1.0.0"
    only_on:
      - {}
tasks:
  test:
    run: cargo test
"#,
                match super::current_os() {
                    "windows" => "linux",
                    _ => "windows",
                }
            ),
        )
        .unwrap();
        assert!(
            !contract
                .runtimes
                .get("definitely-not-installed")
                .expect("runtime requirement should exist")
                .required_for_os(super::current_os())
        );

        let report = diagnose_preconditions(&contract, synthetic_contract_path());
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing runtime: definitely-not-installed")
        );
    }

    #[test]
    fn python_runtime_probe_accepts_versioned_python_aliases() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "python3.13", "#!/bin/sh\necho Python 3.13.2\n");
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: openhands
execution:
  default_context: host
  contexts:
    host:
      backend: native
      requirements:
        runtimes:
          python: ">=3.12,<3.14"
tasks:
  build:
    context: host
    run: echo ready
workflows:
  default: app
  app:
    setup:
      task: build
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("app"),
        );
        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing runtime: python"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Version mismatch for runtime: python"),
            "{report:?}"
        );
    }

    #[test]
    fn python_runtime_probe_uses_requirement_range_candidates() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "python3.11", "#!/bin/sh\necho Python 3.11.9\n");
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: openhands
execution:
  default_context: host
  contexts:
    host:
      backend: native
      requirements:
        runtimes:
          python: ">=3.11,<3.13"
tasks:
  build:
    context: host
    run: echo ready
workflows:
  default: app
  app:
    setup:
      task: build
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("app"),
        );
        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing runtime: python"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Version mismatch for runtime: python"),
            "{report:?}"
        );
    }

    #[test]
    fn toolchain_owned_python_runtime_probe_uses_python_candidates_in_native_mode() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "uv", "#!/bin/sh\necho uv 0.4.16\n");
        write_fake_command(&bin_dir, "python3.12", "#!/bin/sh\necho Python 3.12.8\n");

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: airflow
toolchains:
  python:
    provider: uv
    version: "3.12"
tasks:
  test:
    run: echo ready
    requirements:
      toolchains:
        - python
workflows:
  default: verify
  verify:
    run:
      task: test
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing runtime: python"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Version mismatch for runtime: python"),
            "{report:?}"
        );
    }

    #[test]
    fn toolchain_owned_python_runtime_probe_uses_python_candidates_in_container_mode() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            format!(
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"command -v 'python3.12'\" >nul && (\r\n    echo {started} 1>&2\r\n    exit /b 127\r\n  )\r\n  echo %* | findstr /C:\"command -v 'python3'\" >nul && (\r\n    echo Python 3.12.8\r\n    echo {started} 1>&2\r\n    echo {path}/usr/local/bin/python3 1>&2\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER,
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"command -v 'python3.12'\"*) echo '{started}' >&2; exit 127 ;;\n    *\"command -v 'python3'\"*) echo 'Python 3.12.8'; echo '{started}' >&2; echo '{path}/usr/local/bin/python3' >&2; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER,
            )
        };
        write_fake_command(&bin_dir, "docker", &docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: airflow
execution:
  preferred: container
  lifecycle: ephemeral
  backends:
    container:
      image: python:3.12-bookworm
      engines: [docker]
toolchains:
  python:
    provider: uv
    version: "3.12"
tasks:
  test:
    run: echo ready
    requirements:
      toolchains:
        - python
workflows:
  default: verify
  verify:
    run:
      task: test
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing runtime: python"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Version mismatch for runtime: python"),
            "{report:?}"
        );
    }

    #[test]
    fn doctor_scopes_container_toolchain_probes_to_selected_context_images() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            format!(
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"maven:3.9.14-eclipse-temurin-21-noble\" >nul && (\r\n    echo %* | findstr /C:\"command -v 'java'\" >nul && (\r\n      echo openjdk 21.0.7\r\n      echo {started} 1>&2\r\n      echo {path}/usr/bin/java 1>&2\r\n      exit /b 0\r\n    )\r\n  )\r\n  echo %* | findstr /C:\"mcr.microsoft.com/devcontainers/javascript-node:24-bookworm\" >nul && (\r\n    echo %* | findstr /C:\"command -v 'node'\" >nul && (\r\n      echo v24.11.0\r\n      echo {started} 1>&2\r\n      echo {path}/usr/local/bin/node 1>&2\r\n      exit /b 0\r\n    )\r\n  )\r\n  echo {started} 1>&2\r\n  exit /b 127\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER,
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"maven:3.9.14-eclipse-temurin-21-noble\"*\"command -v 'java'\"*) echo 'openjdk 21.0.7'; echo '{started}' >&2; echo '{path}/usr/bin/java' >&2; exit 0 ;;\n    *\"mcr.microsoft.com/devcontainers/javascript-node:24-bookworm\"*\"command -v 'node'\"*) echo 'v24.11.0'; echo '{started}' >&2; echo '{path}/usr/local/bin/node' >&2; exit 0 ;;\n    *) echo '{started}' >&2; exit 127 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER,
            )
        };
        write_fake_command(&bin_dir, "docker", &docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: scoped-container-doctor
execution:
  default_context: tooling
  contexts:
    application:
      backend: container
      container:
        image: maven:3.9.14-eclipse-temurin-21-noble
        engines: [docker]
    tooling:
      backend: container
      container:
        image: mcr.microsoft.com/devcontainers/javascript-node:24-bookworm
        engines: [docker]
toolchains:
  java:
    provider: sdkman
    version: "21"
tasks:
  setup:
    context: application
    run: echo setup
    requirements:
      toolchains:
        - java
  contract:
    context: tooling
    run: node --version
    depends_on:
      - setup
    requirements:
      runtimes:
        node: "24"
workflows:
  default: verify
  verify:
    run:
      task: contract
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing runtime: java"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing runtime: node"),
            "{report:?}"
        );
    }

    #[test]
    fn toolchain_owned_uv_tool_is_required_in_container_mode() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            format!(
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"command -v 'python3.12'\" >nul && (\r\n    echo Python 3.12.8\r\n    echo {started} 1>&2\r\n    echo {path}/usr/local/bin/python3 1>&2\r\n    exit /b 0\r\n  )\r\n  echo %* | findstr /C:\"command -v 'uv'\" >nul && (\r\n    echo {started} 1>&2\r\n    exit /b 127\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER,
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"command -v 'python3.12'\"*) echo 'Python 3.12.8'; echo '{started}' >&2; echo '{path}/usr/local/bin/python3' >&2; exit 0 ;;\n    *\"command -v 'uv'\"*) echo '{started}' >&2; exit 127 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
                started = super::CONTAINER_PROBE_STARTED_MARKER,
                path = super::CONTAINER_PROBE_PATH_MARKER,
            )
        };
        write_fake_command(&bin_dir, "docker", &docker_body);

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: airflow
execution:
  preferred: container
  backends:
    container:
      image: python:3.12-bookworm
      engines: [docker]
toolchains:
  python:
    provider: uv
    version: "3.12"
tasks:
  setup:
    run: uv sync
    execution:
      default_mode: container
    requirements:
      toolchains:
        - python
workflows:
  default: verify
  verify:
    setup:
      task: setup
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Container,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary.contains("uv"))
            .expect("expected uv finding");
        assert!(finding.next.contains("uv --version"), "{finding:?}");
    }

    #[cfg(not(windows))]
    #[test]
    fn toolchain_owned_uv_tool_version_mismatch_blocks_in_native_mode() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "uv", "#!/bin/sh\necho uv 0.4.16\n");
        write_fake_command(&bin_dir, "python3.10", "#!/bin/sh\necho Python 3.10.18\n");

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: airflow
toolchains:
  python:
    provider: uv
    version: "3.10"
    fulfillment: run
    package_managers:
      uv: ">=0.11.8"
tasks:
  setup:
    run: uv sync
    requirements:
      toolchains:
        - python
      tools:
        uv: ">=0.11.8"
workflows:
  default: verify
  verify:
    setup:
      task: setup
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.code() == "OTA_TOOL_VERSION_MISMATCH" && finding.summary.contains("uv")
            })
            .expect("expected uv version mismatch finding");
        assert!(finding.why.contains(">=0.11.8"), "{finding:?}");
        assert!(finding.why.contains("0.4.16"), "{finding:?}");
    }

    #[cfg(not(windows))]
    #[test]
    fn toolchain_owned_bundler_tool_version_mismatch_blocks_via_bundle_alias() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(
            &bin_dir,
            "bundle",
            "#!/bin/sh\necho Bundler version 2.5.3\n",
        );
        write_fake_command(&bin_dir, "ruby", "#!/bin/sh\necho ruby 3.4.1p0\n");

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: discourse
toolchains:
  ruby:
    provider: ruby
    version: ">=3.4,<3.5"
    fulfillment: run
    package_managers:
      bundler: "2.6.4"
tasks:
  test:
    run: bundle --version
    requirements:
      toolchains:
        - ruby
      tools:
        bundler: "2.6.4"
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_task_with_overrides(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            "test",
            ExecutionOverrides::default(),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.code() == "OTA_TOOL_VERSION_MISMATCH"
                    && (finding.summary.contains("bundle") || finding.summary.contains("bundler"))
            })
            .expect("expected bundle version mismatch finding");
        assert!(finding.why.contains("2.6.4"), "{finding:?}");
        assert!(finding.why.contains("2.5.3"), "{finding:?}");
    }

    #[cfg(not(windows))]
    #[test]
    fn doctor_selected_workflow_allows_run_fulfilled_bundler_when_ruby_exists() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "ruby", "#!/bin/sh\necho ruby 3.3.11p0\n");

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: athena-api
toolchains:
  ruby:
    version: "3.3.11"
    package_managers:
      bundler: "2.5.3"
    fulfillment:
      source: ruby
      mode: run
tasks:
  install:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: bundler
        cwd: .
        path: vendor/bundle
    requirements:
      toolchains:
        - ruby
    effects:
      writes:
        - .bundle
        - vendor/bundle
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    setup:
      task: install
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing tool: bundle"
                    && finding.summary != "Tool probe failed: bundle"
                    && finding.summary != "Version mismatch for tool: bundle"),
            "{report:?}"
        );
    }

    #[cfg(not(windows))]
    #[test]
    fn doctor_selected_workflow_reports_run_fulfilled_ruby_probe_failure_once() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "ruby", "#!/bin/sh\nexit 1\n");

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: athena-api
toolchains:
  ruby:
    version: "3.3.11"
    package_managers:
      bundler: "2.5.3"
    fulfillment:
      source: ruby
      mode: run
tasks:
  install:
    prepare:
      kind: dependency_hydration
      medium: package_dependencies
      source:
        kind: bundler
        cwd: .
        path: vendor/bundle
    requirements:
      toolchains:
        - ruby
    effects:
      writes:
        - .bundle
        - vendor/bundle
      network: true
      network_kind: dependency_hydration
workflows:
  default: verify
  verify:
    setup:
      task: install
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let runtime_probe_failures = report
            .findings
            .iter()
            .filter(|finding| finding.summary == "Runtime probe failed: ruby")
            .count();
        assert_eq!(runtime_probe_failures, 1, "{report:?}");
    }

    #[cfg(not(windows))]
    #[test]
    fn toolchain_owned_rust_runtime_probe_uses_rustc_candidate() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "rustc", "#!/bin/sh\necho rustc 1.94.0\n");

        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", bin_dir.as_os_str().to_os_string());
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
tasks:
  test:
    run: echo ready
    requirements:
      toolchains:
        - rust
workflows:
  default: verify
  verify:
    run:
      task: test
"#,
        )
        .unwrap();

        let report = super::diagnose_preconditions_with_mode_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            Some("verify"),
        );

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Missing runtime: rust"),
            "{report:?}"
        );
        assert!(
            report
                .findings
                .iter()
                .all(|finding| finding.summary != "Version mismatch for runtime: rust"),
            "{report:?}"
        );
    }

    #[test]
    fn reports_policy_version_violations_without_provisioning() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  node: "24"
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  version_policy:
    runtimes:
      node:
        approved_versions:
          - "22"
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Repo does not satisfy org policy pack")
            .expect("policy violation should be present");
        assert!(
            finding
                .why
                .contains("runtime `node` version `24` is not approved by policy")
        );
        assert!(
            finding
                .next
                .contains("update the repo contract to match policy")
        );
        assert!(finding.next.ends_with("org-policy.yaml`"));
    }

    #[test]
    fn reports_policy_native_package_violations() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let (_, package_source, package_name, _) =
            write_native_package_policy_fixture(&fixture, "libpq-dev");

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        let native_package_finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary
                    == format!("Org policy does not approve native package: {package_source}:{package_name}")
            })
            .expect("native package policy finding should be present");
        assert_eq!(
            native_package_finding.code(),
            "OTA_POLICY_NATIVE_PACKAGE_NOT_APPROVED"
        );
        assert!(native_package_finding.why.contains(&format!(
            "native prerequisite `ruby-native-build-tools` requires {package_source} package `{package_name}`"
        )));
        assert!(native_package_finding.next.contains(&format!(
            "policies.native_packages.{package_source}.approved"
        )));
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Repo does not satisfy org policy pack")
            .expect("policy violation should be present");
        assert!(
            finding
                .why
                .contains(&format!(
                    "native prerequisite `ruby-native-build-tools` requires {package_source} package `{package_name}`"
                ))
        );
        assert!(
            finding
                .next
                .starts_with("update the repo contract to match policy, or widen `")
        );
    }

    #[test]
    fn policy_approved_native_packages_flow_into_precondition_provisioning_request() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let (_, package_source, package_name, _) =
            write_native_package_policy_fixture(&fixture, current_native_package_test_case().2);

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        let provisioning = report
            .provisioning
            .as_ref()
            .expect("policy-approved native package request should be present");
        assert!(provisioning.request.actions.iter().any(|action| {
            action.kind == ProvisioningActionKind::Install
                && action.source == package_source
                && action.install_name() == package_name
        }));
    }

    #[test]
    fn reports_strict_policy_compliance_violations_for_installed_tools() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  yq: "4"
tasks:
  test:
    run: yq --version
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  strict_versions: true
  version_policy:
    tools:
      yq:
        approved_versions:
          - "4.52.5"
"#,
        )
        .unwrap();

        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let _yq = write_fake_command(
            &bin_dir,
            "yq",
            if cfg!(windows) {
                "@echo off\r\necho yq 4.52.6\r\n"
            } else {
                "#!/bin/sh\nprintf \"yq 4.52.6\\n\"\n"
            },
        );

        let original_path = env::var_os("PATH");
        let mut path_entries = vec![bin_dir.clone()];
        if let Some(existing) = original_path.as_ref() {
            path_entries.extend(env::split_paths(existing));
        }
        let joined_path = env::join_paths(path_entries).unwrap();
        unsafe {
            env::set_var("PATH", &joined_path);
        }

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));

        match original_path {
            Some(path) => unsafe {
                env::set_var("PATH", path);
            },
            None => unsafe {
                env::remove_var("PATH");
            },
        }

        let finding = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary == "Installed tool is not compliant with org policy: yq"
            })
            .expect("strict policy compliance finding should be present");
        assert_eq!(finding.code(), "OTA_POLICY_INSTALLED_VERSION_NONCOMPLIANT");
        assert_eq!(finding.category(), "policy");
        assert_eq!(finding.owner(), "org_policy");
        assert_eq!(finding.provenance().as_deref(), Some("org policy"));
        assert_eq!(finding.provenance_key().as_deref(), Some("org_policy"));
        let json = serde_json::to_value(finding)
            .expect("strict policy compliance finding should serialize");
        assert_eq!(
            json.get("policy_reason").and_then(|value| value.as_str()),
            Some("strict_version_noncompliance")
        );
        assert!(
            finding
                .why
                .contains("resolved to `4.52.6` and satisfies the repo contract `4`")
        );
        assert!(
            finding
                .why
                .contains("resolved version `4.52.6` is not compliant with strict policy")
        );
    }

    #[test]
    fn reports_missing_policy_required_files() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();
        fs::create_dir_all(fixture.path().join(".ota")).unwrap();
        fs::write(
            fixture.path().join(".ota").join("org-policy.yaml"),
            r#"
policies:
  required_files:
    - AGENTS.md
"#,
        )
        .unwrap();

        let contract = parse_contract_str(
            synthetic_contract_path(),
            &fs::read_to_string(fixture.path().join("ota.yaml")).unwrap(),
        )
        .unwrap();

        let report = diagnose_preconditions(&contract, &fixture.path().join("ota.yaml"));
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].summary,
            "Repo does not satisfy org policy pack"
        );
        assert!(report.findings[0].why.contains("AGENTS.md"));
    }

    #[cfg(unix)]
    #[test]
    fn reports_timed_out_checks() {
        let _guard = env_mutex_lock();
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
checks:
  - name: slow-check
    kind: health
    severity: warn
    run: sleep 1
    timeout: 50
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Check timed out: slow-check")
            .expect("expected timed out check finding");
        assert_eq!(finding.severity, FindingSeverity::Warn);
        assert!(finding.why.contains("50ms"));
    }

    #[test]
    fn doctor_short_circuits_surface_diagnosis_when_preconditions_are_blocked() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
runtimes:
  definitely-not-installed:
    version: "*"
surfaces:
  backend:
    kind: http
    port: 6551
    readiness:
      kind: http
      path: /healthz
tasks:
  dev:
    run: npm run dev
    runtime:
      kind: service
      surfaces:
        - backend
workflows:
  default: app
  app:
    run:
      task: dev
    readiness:
      surfaces:
        - backend
"#,
        )
        .unwrap();

        let report = super::diagnose_contract_with_mode_and_lifecycle_for_workflow(
            &contract,
            synthetic_contract_path(),
            DoctorMode::Native,
            None,
            Some("app"),
        );

        assert!(!report.ok, "{report:?}");
        assert!(
            report
                .findings
                .iter()
                .any(|finding| finding.summary == "Missing runtime: definitely-not-installed")
        );
        assert!(report.findings.iter().all(|finding| {
            finding.summary != "Surface readiness failed: backend"
                && finding.summary != "Surface readiness timed out: backend"
        }));
    }
}
