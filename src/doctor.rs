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

use serde::ser::{SerializeStruct, Serializer};
use serde::{Deserialize, Serialize};
use serde_json::Value as JsonValue;

use crate::execution::{
    container_backend_probe_failure, container_engine_candidates,
    container_engine_candidates_from_backend, container_engine_command,
    matching_declared_execution_context_name, preferred_container_backend_probe_failure,
    selected_container_engine, selected_container_engine_from_backend,
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
    host_runtime_readiness_observed, http_readiness_endpoint_status, load_declared_env_sources,
    parse_http_probe_url, resolve_context_execution_backend, resolve_declared_env_source_value,
    resolve_named_readiness_probe, resolve_named_readiness_probe_contract,
    resolve_task_target_binding_url_with_contract_path, run_backend_command_captured,
    task_runtime_host_readiness_probe_for_backend, task_surface_host_readiness_probe_for_backend,
};
use crate::schema::{
    Backend, CheckKind, CheckSeverity, ContainerBackend, Contract, ExtensionKind, Lifecycle,
    NativePrerequisiteActivationShell, ReadinessProbeSpec, RequirementSurface, RuntimeRequirement,
    ServiceProducerSpec, ServiceReadinessSpec, ServiceSpec, ToolAcquisitionProvider,
    ToolAcquisitionSpec, ToolRequirement,
};
use crate::terminal::supports_dynamic_stderr_ui;
use crate::toolchains::{
    ToolchainManagedSurfaceKind, ToolchainOpportunityContext, declared_toolchain_contract,
    requirement_surface_with_toolchain_owned_capabilities,
    requirement_surface_with_toolchain_owned_tools, shipped_toolchain_contract_by_label,
    tool_versions_entry, toolchain_repo_signals, unsupported_toolchain_opportunity_context,
};
use crate::validator::{ContractAdvisory, TaskExecutionBoundary, collect_contract_advisories};
use crate::workspace::load_contract_for_workspace_repo_ref;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct Finding {
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
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

fn rerun_doctor_command(mode: DoctorMode, lifecycle_override: Option<Lifecycle>) -> String {
    doctor_command_string(mode, doctor_selected_lifecycle(mode, lifecycle_override))
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
}

#[derive(Debug, Clone)]
struct BackendPreconditionSelection {
    backend: Backend,
    requirement_surface: RequirementSurface,
    toolchain_names: BTreeSet<String>,
    native_names: BTreeSet<String>,
    env_names: BTreeSet<String>,
    env_scoped: bool,
}

impl From<BackendPreconditionSelection> for ScopedPreconditionSelection {
    fn from(value: BackendPreconditionSelection) -> Self {
        Self {
            requirement_surface: value.requirement_surface,
            toolchain_names: value.toolchain_names,
            native_names: value.native_names,
            env_names: value.env_names,
            env_scoped: value.env_scoped,
        }
    }
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
        contract
            .tasks
            .get(task_name.as_str())
            .is_some_and(|task| !task.requirements.runtimes.is_empty())
    });
    let scoped_tools = task_names.iter().any(|task_name| {
        contract
            .tasks
            .get(task_name.as_str())
            .is_some_and(|task| !task.requirements.tools.is_empty())
    });
    let scoped_toolchains = task_names.iter().any(|task_name| {
        contract
            .tasks
            .get(task_name.as_str())
            .is_some_and(|task| !task.requirements.toolchains.is_empty())
    });
    let scoped_env = task_names.iter().any(|task_name| {
        contract
            .tasks
            .get(task_name.as_str())
            .is_some_and(|task| !task.requirements.env.is_empty())
    });

    let mut selections = Vec::<BackendPreconditionSelection>::new();

    for task_name in task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        let backend = effective_task_execution(contract, task_name.as_str(), overrides).backend;
        let selection =
            if let Some(existing) = selections.iter_mut().find(|item| item.backend == backend) {
                existing
            } else {
                selections.push(BackendPreconditionSelection {
                    backend,
                    requirement_surface: RequirementSurface::default(),
                    toolchain_names: BTreeSet::new(),
                    native_names: BTreeSet::new(),
                    env_names: BTreeSet::new(),
                    env_scoped: scoped_env,
                });
                selections.last_mut().expect("selection was just pushed")
            };

        for (name, requirement) in &task.requirements.runtimes {
            selection.requirement_surface.runtimes.insert(
                name.clone(),
                contract.resolve_scoped_runtime_requirement(name, requirement),
            );
        }
        for (name, requirement) in &task.requirements.tools {
            selection.requirement_surface.tools.insert(
                name.clone(),
                contract.resolve_scoped_tool_requirement(name, requirement),
            );
        }
        selection
            .toolchain_names
            .extend(task.requirements.toolchains.iter().cloned());
        selection
            .env_names
            .extend(task.requirements.env.iter().cloned());
        if matches!(backend, Backend::Native) {
            selection
                .native_names
                .extend(task.requirements.native.iter().cloned());
        }

        if let Some(context_name) = task.context_for_backend(contract.execution.as_ref(), backend)
            && let Some(context) = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.contexts.get(context_name))
        {
            selection.requirement_surface.merge(&RequirementSurface {
                runtimes: context.requirements.runtimes.clone(),
                tools: context.requirements.tools.clone(),
            });
        }
    }

    for selection in &mut selections {
        if !scoped_runtimes {
            let mut runtimes = contract.runtimes.clone();
            runtimes.extend(selection.requirement_surface.runtimes.clone());
            selection.requirement_surface.runtimes = runtimes;
        }
        if !scoped_tools {
            let mut tools = contract.tools.clone();
            tools.extend(selection.requirement_surface.tools.clone());
            selection.requirement_surface.tools = tools;
        }
        if !scoped_toolchains {
            selection.toolchain_names = contract.toolchains.keys().cloned().collect();
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

    let mut selection = ScopedPreconditionSelection {
        requirement_surface: contract
            .selected_workflow_task_requirement_surface(workflow_name)
            .unwrap_or_default(),
        toolchain_names: contract.selected_workflow_required_toolchain_names(workflow_name),
        ..ScopedPreconditionSelection::default()
    };

    for task_name in task_names {
        let Some(task) = contract.tasks.get(task_name.as_str()) else {
            continue;
        };
        selection
            .requirement_surface
            .merge(&task.scoped_requirement_surface());
        if !task.requirements.env.is_empty() {
            selection.env_scoped = true;
            selection
                .env_names
                .extend(task.requirements.env.iter().cloned());
        }
        selection
            .native_names
            .extend(task.requirements.native.iter().cloned());
        if let Some(context_name) = task.context_for_backend(contract.execution.as_ref(), backend)
            && let Some(context) = contract
                .execution
                .as_ref()
                .and_then(|execution| execution.contexts.get(context_name))
        {
            selection.requirement_surface.merge(&RequirementSurface {
                runtimes: context.requirements.runtimes.clone(),
                tools: context.requirements.tools.clone(),
            });
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
        if !task.requirements.runtimes.is_empty() {
            target.scoped_runtimes = true;
        }
        if !task.requirements.tools.is_empty() {
            target.scoped_tools = true;
        }
        if !task.requirements.toolchains.is_empty() {
            target.scoped_toolchains = true;
            target
                .toolchain_names
                .extend(task.requirements.toolchains.iter().cloned());
        }
        for (name, requirement) in &task.requirements.runtimes {
            target.surface.runtimes.insert(
                name.clone(),
                contract.resolve_scoped_runtime_requirement(name, requirement),
            );
        }
        for (name, requirement) in &task.requirements.tools {
            target.surface.tools.insert(
                name.clone(),
                contract.resolve_scoped_tool_requirement(name, requirement),
            );
        }
    }

    if !saw_task {
        return None;
    }

    let finalize_entry = |entry: Entry| -> ScopedPreconditionSelection {
        let mut surface = entry.surface;
        if !entry.scoped_runtimes {
            surface.runtimes = contract.runtimes.clone();
        }
        if !entry.scoped_tools {
            surface.tools = contract.tools.clone();
        }
        let toolchain_names = if entry.scoped_toolchains {
            entry.toolchain_names
        } else {
            contract.toolchains.keys().cloned().collect()
        };
        ScopedPreconditionSelection {
            requirement_surface: surface,
            toolchain_names,
            native_names: BTreeSet::new(),
            env_names: BTreeSet::new(),
            env_scoped: false,
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
    requirement_surface_with_toolchain_owned_capabilities(
        contract,
        requirement_surface,
        toolchain_names,
        target_os,
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
        ToolAcquisitionProvider::Corepack => corepack_activation_command(acquisition),
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
    }
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
    Finding {
        severity: FindingSeverity::Error,
        summary: format!("Remote target operating system could not be determined{suffix}"),
        why,
        next,
    }
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
                findings.push(Finding {
                    severity: FindingSeverity::Error,
                    summary: format!("Remote execution context is not executable: {name}"),
                    why: error.to_string(),
                    next: format!(
                        "repair `execution.contexts.{name}` so ota can execute that remote context, then rerun `ota doctor --mode remote`"
                    ),
                });
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
            });
        requirement_surface.merge(&RequirementSurface {
            runtimes: context.requirements.runtimes.clone(),
            tools: context.requirements.tools.clone(),
        });
        let selected_toolchain_names = selected_task_surface
            .map(|selection| selection.toolchain_names.clone())
            .unwrap_or_else(|| contract.toolchains.keys().cloned().collect());
        let policy_requirement_surface = policy_requirement_surface_for_toolchains(
            contract,
            &requirement_surface,
            &selected_toolchain_names,
            &target_os,
        );
        let provisioning_actions = loaded_policy
            .map(|loaded| {
                loaded
                    .pack
                    .selected_provisioning_actions_for_requirement_surface_os(
                        &target_os,
                        &policy_requirement_surface,
                    )
            })
            .unwrap_or_default();
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
    let provisioning_actions = loaded_policy
        .map(|loaded| {
            loaded
                .pack
                .selected_provisioning_actions_for_requirement_surface_os(
                    &target_os,
                    &policy_requirement_surface,
                )
        })
        .unwrap_or_default();

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
const DOCTOR_WORKFLOW_SURFACE_READINESS_FAILED_RETRIES: u32 = 120;
const DOCTOR_WORKFLOW_SURFACE_READINESS_TIMEOUT_RETRIES: u32 = 30;
const DOCTOR_WORKFLOW_SURFACE_READINESS_INTERVAL_MS: u64 = 200;
const DOCTOR_WORKFLOW_SURFACE_MAX_PROBE_TIMEOUT_MS: u64 = 2_000;
const DOCTOR_WORKFLOW_SURFACE_FAILED_RETRY_WINDOW_MS: u64 = 90_000;
const DOCTOR_WORKFLOW_SURFACE_TIMEOUT_RETRY_WINDOW_MS: u64 = 90_000;

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

    match (&target, diagnosis.backend.as_str(), diagnosis.kind) {
        (
            ProvisioningExecutionTarget::Container { .. },
            "apt",
            ProvisioningFailureKind::VersionUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Container apt cannot install pinned package version: {}",
                diagnosis.name
            ),
            why: format!(
                "the Linux/container target requests {}, but the configured apt sources do not provide that version",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "update the selected container image{image_hint} or its apt sources, or relax the Linux/container version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Container { .. },
            "apt",
            ProvisioningFailureKind::PackageUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Container apt cannot locate required package: {}",
                diagnosis.name
            ),
            why: format!(
                "the Linux/container target requests `{}`, but the configured apt sources do not provide that package",
                diagnosis.name
            ),
            next: format!(
                "update the selected container image{image_hint} or its apt sources so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Container { .. },
            "apt",
            ProvisioningFailureKind::IndexUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Container apt cannot refresh configured sources: {}",
                diagnosis.name
            ),
            why: format!(
                "the Linux/container target could not refresh apt indexes, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix apt repository access in the selected container image{image_hint}, then rerun `{rerun_command}`"
            ),
        },
        (
            ProvisioningExecutionTarget::Container { .. },
            backend,
            ProvisioningFailureKind::VersionUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Container {backend} cannot install pinned version: {}",
                diagnosis.name
            ),
            why: format!(
                "the Linux/container target requests {}, but the configured `{backend}` provisioning path does not provide that version inside container image `{}`",
                provisioning_diagnosis_requirement_summary(diagnosis),
                image.unwrap_or("unknown")
            ),
            next: format!(
                "fix the selected container image{image_hint} or the configured `{backend}` provisioning path, or relax the version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Container { .. },
            backend,
            ProvisioningFailureKind::PackageUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Container {backend} cannot locate required package: {}",
                diagnosis.name
            ),
            why: format!(
                "the Linux/container target requests `{}`, but the configured `{backend}` provisioning path does not provide that package inside container image `{}`",
                diagnosis.name,
                image.unwrap_or("unknown")
            ),
            next: format!(
                "fix the selected container image{image_hint} or the configured `{backend}` provisioning path so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Container { .. },
            backend,
            ProvisioningFailureKind::IndexUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Container {backend} cannot refresh configured sources: {}",
                diagnosis.name
            ),
            why: format!(
                "the Linux/container target could not refresh the configured `{backend}` sources, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix `{backend}` repository access in the selected container image{image_hint}, then rerun `{rerun_command}`"
            ),
        },
        (ProvisioningExecutionTarget::Container { .. }, backend, _) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Container {backend} cannot install requested prerequisite: {}",
                diagnosis.name
            ),
            why: format!(
                "the Linux/container target requests {}, but the configured `{backend}` provisioning path could not satisfy it inside container image `{}`",
                provisioning_diagnosis_requirement_summary(diagnosis),
                image.unwrap_or("unknown")
            ),
            next: format!(
                "fix the selected container image{image_hint} or the configured `{backend}` provisioning path for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Native,
            backend,
            ProvisioningFailureKind::VersionUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Host {backend} cannot install pinned version: {}",
                diagnosis.name
            ),
            why: format!(
                "the host target requests {}, but the configured `{backend}` provisioning path could not provide that version",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix the host `{backend}` provisioning path or relax the version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Native,
            backend,
            ProvisioningFailureKind::PackageUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Host {backend} cannot locate required package: {}",
                diagnosis.name
            ),
            why: format!(
                "the host target requests `{}`, but the configured `{backend}` provisioning path does not provide that package",
                diagnosis.name
            ),
            next: format!(
                "fix the host `{backend}` provisioning path so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Native,
            backend,
            ProvisioningFailureKind::IndexUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Host {backend} cannot refresh configured sources: {}",
                diagnosis.name
            ),
            why: format!(
                "the host target could not refresh the configured `{backend}` sources, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix the host `{backend}` repository access, then rerun `{rerun_command}`"
            ),
        },
        (ProvisioningExecutionTarget::Native, backend, _) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Host {backend} cannot install requested prerequisite: {}",
                diagnosis.name
            ),
            why: format!(
                "the host target requests {}, but the configured `{backend}` provisioning path could not satisfy it",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix the host `{backend}` provisioning path for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Remote { .. },
            "apt",
            ProvisioningFailureKind::VersionUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Remote apt cannot install pinned package version: {}{}",
                diagnosis.name, remote_suffix
            ),
            why: format!(
                "{remote_label} requests {}, but the configured apt sources do not provide that version",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix the remote apt sources or relax the version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Remote { .. },
            "apt",
            ProvisioningFailureKind::PackageUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Remote apt cannot locate required package: {}{}",
                diagnosis.name, remote_suffix
            ),
            why: format!(
                "{remote_label} requests `{}`, but the configured apt sources do not provide that package",
                diagnosis.name
            ),
            next: format!(
                "fix the remote apt sources so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Remote { .. },
            "apt",
            ProvisioningFailureKind::IndexUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Remote apt cannot refresh configured sources: {}{}",
                diagnosis.name, remote_suffix
            ),
            why: format!(
                "{remote_label} could not refresh apt indexes, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!("fix remote apt repository access, then rerun `{rerun_command}`"),
        },
        (
            ProvisioningExecutionTarget::Remote { .. },
            backend,
            ProvisioningFailureKind::VersionUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Remote {backend} cannot install pinned version: {}{}",
                diagnosis.name, remote_suffix
            ),
            why: format!(
                "{remote_label} requests {}, but the configured `{backend}` provisioning path does not provide that version",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix the remote `{backend}` provisioning path or relax the version pin for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Remote { .. },
            backend,
            ProvisioningFailureKind::PackageUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Remote {backend} cannot locate required package: {}{}",
                diagnosis.name, remote_suffix
            ),
            why: format!(
                "{remote_label} requests `{}`, but the configured `{backend}` provisioning path does not provide that package",
                diagnosis.name
            ),
            next: format!(
                "fix the remote `{backend}` provisioning path so `{}` is available, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
        (
            ProvisioningExecutionTarget::Remote { .. },
            backend,
            ProvisioningFailureKind::IndexUnavailable,
        ) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Remote {backend} cannot refresh configured sources: {}{}",
                diagnosis.name, remote_suffix
            ),
            why: format!(
                "{remote_label} could not refresh the configured `{backend}` sources, so ota could not verify or install {}",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix the remote `{backend}` repository access, then rerun `{rerun_command}`"
            ),
        },
        (ProvisioningExecutionTarget::Remote { .. }, backend, _) => Finding {
            severity: FindingSeverity::Error,
            summary: format!(
                "Remote {backend} cannot install requested prerequisite: {}{}",
                diagnosis.name, remote_suffix
            ),
            why: format!(
                "{remote_label} requests {}, but the configured `{backend}` provisioning path could not satisfy it",
                provisioning_diagnosis_requirement_summary(diagnosis)
            ),
            next: format!(
                "fix the remote `{backend}` provisioning path for `{}`, then rerun `{rerun_command}`",
                diagnosis.name
            ),
        },
    }
}

impl Finding {
    fn policy_context(&self) -> Option<PolicyFindingContext<'_>> {
        match self.summary.as_str() {
            "Repo does not satisfy org policy pack" => {
                let has_sections = self.why.contains("missing contract sections:");
                let has_files = self.why.contains("missing files:");
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
            "Policy-backed provisioning sources are declared" => Some(PolicyFindingContext {
                outcome: "policy_surface_available",
                reason: "policy_backed_provisioning_declared",
                source: "org",
                install_scope: "repo_local",
                mutation_allowed: false,
            }),
            "Adapter bootstrap sources are declared" => Some(PolicyFindingContext {
                outcome: "policy_surface_available",
                reason: "policy_backed_adapter_bootstrap_declared",
                source: "org",
                install_scope: "repo_local",
                mutation_allowed: false,
            }),
            "Policy provisioning needs explicit package identifiers" => {
                Some(PolicyFindingContext {
                    outcome: "blocked_by_policy",
                    reason: "missing_package_identifiers",
                    source: "org",
                    install_scope: "repo_local",
                    mutation_allowed: false,
                })
            }
            "Invalid org policy pack" => Some(PolicyFindingContext {
                outcome: "blocked_by_integrity_policy",
                reason: "invalid_org_policy_pack",
                source: "org",
                install_scope: "repo_local",
                mutation_allowed: false,
            }),
            _ => None,
        }
    }

    pub(crate) fn code(&self) -> &'static str {
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
            "Adapter bootstrap sources are declared" => {
                "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED"
            }
            s if s.starts_with("Check failed: ") => "OTA_CHECK_FAILED",
            s if s.starts_with("Check timed out: ") => "OTA_CHECK_TIMED_OUT",
            s if s.starts_with("File check failed: ") => "OTA_FILE_CHECK_FAILED",
            s if s.starts_with("File check timed out: ") => "OTA_FILE_CHECK_TIMED_OUT",
            s if s.starts_with("Contract drift:") => "OTA_CONTRACT_DRIFT",
            s if s.starts_with("Task `") && s.contains(" mutates managed isolated path `") => {
                "OTA_TASK_MUTATES_MANAGED_ISOLATED_PATH"
            }
            _ => "OTA_DOCTOR_FINDING_UNKNOWN",
        }
    }

    fn category(&self) -> &'static str {
        match self.code() {
            "OTA_TASKS_MISSING" => "contract",
            "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY" | "OTA_LIFECYCLE_EPHEMERAL_ADVISORY" => {
                "execution"
            }
            "OTA_BACKEND_CLI_MISSING" | "OTA_CONTAINER_BACKEND_CLI_MISSING" => "execution",
            "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED"
            | "OTA_REMOTE_TARGET_SUSPICIOUS"
            | "OTA_REMOTE_CONTEXT_UNEXECUTABLE"
            | "OTA_REMOTE_TARGET_OS_UNDETERMINED" => "remote",
            "OTA_SERVICE_READINESS_FAILED"
            | "OTA_SERVICE_CHECK_FAILED"
            | "OTA_SERVICE_CHECK_TIMED_OUT"
            | "OTA_SERVICE_UNVERIFIABLE" => "service",
            "OTA_ENV_MISSING"
            | "OTA_ENV_INVALID"
            | "OTA_RUNTIME_VERSION_MISMATCH"
            | "OTA_RUNTIME_MISSING"
            | "OTA_RUNTIME_PROBE_FAILED"
            | "OTA_RUNTIME_VERSION_UNPARSEABLE"
            | "OTA_TOOL_VERSION_MISMATCH"
            | "OTA_TOOL_MISSING"
            | "OTA_TOOL_ACTIVATION_PROVIDER_MISSING"
            | "OTA_TOOL_PROBE_FAILED"
            | "OTA_TOOL_VERSION_UNPARSEABLE"
            | "OTA_NATIVE_PREREQUISITE_MISSING"
            | "OTA_NATIVE_PREREQUISITE_TIMED_OUT" => "environment",
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
            | "OTA_REMOTE_PROVISIONING_BACKEND_FAILED" => "provisioning",
            "OTA_POLICY_PACK_VIOLATION"
            | "OTA_POLICY_PACK_INVALID"
            | "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING"
            | "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
            | "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED" => "policy",
            "OTA_CHECK_FAILED"
            | "OTA_CHECK_TIMED_OUT"
            | "OTA_FILE_CHECK_FAILED"
            | "OTA_FILE_CHECK_TIMED_OUT" => "execution",
            "OTA_CONTRACT_DRIFT" | "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED" => "contract",
            "OTA_TASK_MUTATES_MANAGED_ISOLATED_PATH" => "contract",
            _ => "contract",
        }
    }

    fn owner(&self) -> &'static str {
        match self.code() {
            "OTA_TASKS_MISSING"
            | "OTA_CONTRACT_DRIFT"
            | "OTA_CHECK_FAILED"
            | "OTA_CHECK_TIMED_OUT"
            | "OTA_FILE_CHECK_FAILED"
            | "OTA_FILE_CHECK_TIMED_OUT"
            | "OTA_TASK_MUTATES_MANAGED_ISOLATED_PATH" => "repo_contract",
            "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY" | "OTA_LIFECYCLE_EPHEMERAL_ADVISORY" => {
                "repo_contract"
            }
            "OTA_BACKEND_CLI_MISSING"
            | "OTA_CONTAINER_BACKEND_CLI_MISSING"
            | "OTA_ENV_MISSING"
            | "OTA_ENV_INVALID"
            | "OTA_RUNTIME_VERSION_MISMATCH"
            | "OTA_RUNTIME_MISSING"
            | "OTA_RUNTIME_PROBE_FAILED"
            | "OTA_RUNTIME_VERSION_UNPARSEABLE"
            | "OTA_TOOL_VERSION_MISMATCH"
            | "OTA_TOOL_MISSING"
            | "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED"
            | "OTA_TOOL_PROBE_FAILED"
            | "OTA_TOOL_VERSION_UNPARSEABLE"
            | "OTA_NATIVE_PREREQUISITE_MISSING"
            | "OTA_NATIVE_PREREQUISITE_TIMED_OUT" => {
                if finding_targets_container_image(&self.why) {
                    "container_target"
                } else if finding_targets_remote_backend(&self.why) {
                    "remote_target"
                } else {
                    "host"
                }
            }
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
            "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED"
            | "OTA_REMOTE_TARGET_SUSPICIOUS"
            | "OTA_REMOTE_CONTEXT_UNEXECUTABLE"
            | "OTA_REMOTE_TARGET_OS_UNDETERMINED" => "remote_backend",
            "OTA_REMOTE_APT_VERSION_UNAVAILABLE"
            | "OTA_REMOTE_APT_PACKAGE_UNAVAILABLE"
            | "OTA_REMOTE_APT_INDEX_UNAVAILABLE"
            | "OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE"
            | "OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE"
            | "OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE"
            | "OTA_REMOTE_PROVISIONING_BACKEND_FAILED" => "remote_target",
            "OTA_SERVICE_READINESS_FAILED"
            | "OTA_SERVICE_CHECK_FAILED"
            | "OTA_SERVICE_CHECK_TIMED_OUT"
            | "OTA_SERVICE_UNVERIFIABLE" => "service",
            "OTA_POLICY_PACK_VIOLATION"
            | "OTA_POLICY_PACK_INVALID"
            | "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING"
            | "OTA_POLICY_BACKED_PROVISIONING_DECLARED"
            | "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED" => "org_policy",
            _ => "repo_contract",
        }
    }

    fn evidence(&self) -> FindingEvidence {
        let (observed, expected, source, command, path) = match self.code() {
            "OTA_TASKS_MISSING" => (
                "no runnable task entry was declared".to_string(),
                "at least one runnable task is declared".to_string(),
                "contract".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY" | "OTA_LIFECYCLE_EPHEMERAL_ADVISORY" => (
                "ephemeral lifecycle was requested".to_string(),
                "isolated backend-backed execution is available".to_string(),
                "execution".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_BACKEND_CLI_MISSING" | "OTA_CONTAINER_BACKEND_CLI_MISSING" => (
                "required backend CLI was not found on PATH".to_string(),
                "a supported backend CLI is available on PATH".to_string(),
                "host".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED" => (
                "the declared remote backend provider is unsupported".to_string(),
                "a supported remote backend provider is declared".to_string(),
                "remote_backend".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_TARGET_SUSPICIOUS" => (
                "the remote target shape did not match provider expectations".to_string(),
                "a provider-compatible remote target is declared".to_string(),
                "remote_backend".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_CONTEXT_UNEXECUTABLE" => (
                "the named remote execution context could not be resolved".to_string(),
                "the named remote execution context is executable".to_string(),
                "remote_backend".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_TARGET_OS_UNDETERMINED" => (
                "ota could not determine the remote target operating system".to_string(),
                "ota can determine the remote target operating system".to_string(),
                "remote_backend".to_string(),
                remote_os_probe_command().to_string(),
                String::new(),
            ),
            "OTA_SERVICE_READINESS_FAILED" => (
                "the configured service readiness probe failed".to_string(),
                "the service readiness probe passes from its declared context".to_string(),
                "service".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_SERVICE_CHECK_FAILED" => (
                "the configured service healthcheck failed".to_string(),
                "the service healthcheck passes".to_string(),
                "service".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_SERVICE_CHECK_TIMED_OUT" => (
                "the configured service healthcheck timed out".to_string(),
                "the service healthcheck completes within its timeout".to_string(),
                "service".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_SERVICE_UNVERIFIABLE" => (
                "the service cannot be verified from the contract".to_string(),
                "the service declares enough information to verify readiness".to_string(),
                "service".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_ENV_MISSING" => (
                "a required environment variable was missing".to_string(),
                "the environment variable is present".to_string(),
                "contract".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_ENV_INVALID" => (
                "the resolved environment value is outside the allowed set".to_string(),
                "the environment value satisfies the allowed set".to_string(),
                "contract".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_RUNTIME_VERSION_MISMATCH" | "OTA_TOOL_VERSION_MISMATCH" => (
                "the installed version did not match the contract requirement".to_string(),
                "the installed version satisfies the contract requirement".to_string(),
                if finding_targets_container_image(&self.why) {
                    "container_target".to_string()
                } else if finding_targets_remote_backend(&self.why) {
                    "remote_target".to_string()
                } else {
                    "host".to_string()
                },
                finding_probe_command(&self.why).unwrap_or_default(),
                finding_probe_path(&self.why).unwrap_or_default(),
            ),
            "OTA_RUNTIME_MISSING" | "OTA_TOOL_MISSING" => (
                "the required runtime or tool was not available".to_string(),
                "the required runtime or tool is available on PATH".to_string(),
                if finding_targets_container_image(&self.why) {
                    "container_target".to_string()
                } else if finding_targets_remote_backend(&self.why) {
                    "remote_target".to_string()
                } else {
                    "host".to_string()
                },
                String::new(),
                String::new(),
            ),
            "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED" => (
                "the selected repo path is using fallback runtime/tool declarations for an ecosystem Ota does not yet ship as a managed toolchain".to_string(),
                "a shipped toolchain provider exists for that ecosystem or the repo intentionally stays on the fallback runtime/tool model".to_string(),
                "repo_signals".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_RUNTIME_PROBE_FAILED" | "OTA_TOOL_PROBE_FAILED" => (
                "the resolved executable could not report a version".to_string(),
                "the resolved executable reports a version that satisfies the contract"
                    .to_string(),
                if finding_targets_container_image(&self.why) {
                    "container_target".to_string()
                } else if finding_targets_remote_backend(&self.why) {
                    "remote_target".to_string()
                } else {
                    "host".to_string()
                },
                finding_probe_command(&self.why).unwrap_or_default(),
                finding_probe_path(&self.why).unwrap_or_default(),
            ),
            "OTA_RUNTIME_VERSION_UNPARSEABLE" | "OTA_TOOL_VERSION_UNPARSEABLE" => (
                "the resolved executable did not emit a parseable version".to_string(),
                "the resolved executable emits a parseable version that satisfies the contract"
                    .to_string(),
                if finding_targets_container_image(&self.why) {
                    "container_target".to_string()
                } else if finding_targets_remote_backend(&self.why) {
                    "remote_target".to_string()
                } else {
                    "host".to_string()
                },
                finding_probe_command(&self.why).unwrap_or_default(),
                finding_probe_path(&self.why).unwrap_or_default(),
            ),
            "OTA_NATIVE_PREREQUISITE_MISSING" | "OTA_NATIVE_PREREQUISITE_TIMED_OUT" => (
                "the selected native prerequisite check did not pass".to_string(),
                "the selected native prerequisite check passes".to_string(),
                "host".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CONTAINER_APT_VERSION_UNAVAILABLE" => (
                "the configured container apt sources do not provide the pinned package version"
                    .to_string(),
                "the configured container apt sources provide the pinned package version"
                    .to_string(),
                "container_apt".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE" => (
                "the configured container apt sources do not provide the requested package"
                    .to_string(),
                "the configured container apt sources provide the requested package".to_string(),
                "container_apt".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CONTAINER_APT_INDEX_UNAVAILABLE" => (
                "the configured container apt sources could not refresh indexes".to_string(),
                "the configured container apt sources refresh successfully".to_string(),
                "container_apt".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE" => (
                "the configured container provisioning backend could not provide the pinned version".to_string(),
                "the configured container provisioning backend provides the pinned version".to_string(),
                "container_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE" => (
                "the configured container provisioning backend could not provide the requested package".to_string(),
                "the configured container provisioning backend provides the requested package".to_string(),
                "container_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE" => (
                "the configured container provisioning backend could not refresh its sources".to_string(),
                "the configured container provisioning backend refreshes its sources successfully".to_string(),
                "container_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CONTAINER_PROVISIONING_BACKEND_FAILED" => (
                "the configured container provisioning backend could not satisfy the requested prerequisite".to_string(),
                "the configured container provisioning backend satisfies the requested prerequisite".to_string(),
                "container_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE" => (
                "the configured host provisioning backend could not provide the pinned version".to_string(),
                "the configured host provisioning backend provides the pinned version".to_string(),
                "host_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE" => (
                "the configured host provisioning backend could not provide the requested package".to_string(),
                "the configured host provisioning backend provides the requested package".to_string(),
                "host_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE" => (
                "the configured host provisioning backend could not refresh its sources".to_string(),
                "the configured host provisioning backend refreshes its sources successfully".to_string(),
                "host_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_HOST_PROVISIONING_BACKEND_FAILED" => (
                "the configured host provisioning backend could not satisfy the requested prerequisite".to_string(),
                "the configured host provisioning backend satisfies the requested prerequisite".to_string(),
                "host_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_APT_VERSION_UNAVAILABLE" => (
                "the configured remote apt sources do not provide the pinned package version"
                    .to_string(),
                "the configured remote apt sources provide the pinned package version"
                    .to_string(),
                "remote_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_APT_PACKAGE_UNAVAILABLE" => (
                "the configured remote apt sources do not provide the requested package"
                    .to_string(),
                "the configured remote apt sources provide the requested package".to_string(),
                "remote_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_APT_INDEX_UNAVAILABLE" => (
                "the configured remote apt sources could not refresh indexes".to_string(),
                "the configured remote apt sources refresh successfully".to_string(),
                "remote_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE" => (
                "the configured remote provisioning backend could not provide the pinned version"
                    .to_string(),
                "the configured remote provisioning backend provides the pinned version"
                    .to_string(),
                "remote_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE" => (
                "the configured remote provisioning backend could not provide the requested package"
                    .to_string(),
                "the configured remote provisioning backend provides the requested package"
                    .to_string(),
                "remote_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE" => (
                "the configured remote provisioning backend could not refresh its sources"
                    .to_string(),
                "the configured remote provisioning backend refreshes its sources successfully"
                    .to_string(),
                "remote_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_REMOTE_PROVISIONING_BACKEND_FAILED" => (
                "the configured remote provisioning backend could not satisfy the requested prerequisite".to_string(),
                "the configured remote provisioning backend satisfies the requested prerequisite".to_string(),
                "remote_provisioning".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_POLICY_PACK_VIOLATION" => (
                "the repo failed org policy validation".to_string(),
                "the repo satisfies the org policy pack".to_string(),
                "org_policy".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_POLICY_PACK_INVALID" => (
                "the org policy pack failed to load or validate".to_string(),
                "the org policy pack loads and validates".to_string(),
                "org_policy".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_POLICY_BACKED_PROVISIONING_DECLARED" => (
                "the org policy pack declares approved provisioning sources".to_string(),
                "the org policy pack has no provisioning sources declared".to_string(),
                "org_policy".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_POLICY_PROVISIONING_PACKAGE_MAPPING_MISSING" => (
                "policy-backed provisioning is missing required package identifiers".to_string(),
                "policy-backed provisioning rules declare required package identifiers for OS package managers".to_string(),
                "org_policy".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_POLICY_BACKED_ADAPTER_BOOTSTRAP_DECLARED" => (
                "the org policy pack declares approved adapter bootstrap sources".to_string(),
                "the org policy pack has no adapter bootstrap sources declared".to_string(),
                "org_policy".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CHECK_FAILED" => (
                "the configured check failed".to_string(),
                "the configured check succeeds".to_string(),
                "execution".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CHECK_TIMED_OUT" => (
                "the configured check timed out".to_string(),
                "the configured check completes within the timeout".to_string(),
                "execution".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_FILE_CHECK_FAILED" => (
                "the configured file check failed".to_string(),
                "the configured file state matches the contract".to_string(),
                "repo filesystem".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_FILE_CHECK_TIMED_OUT" => (
                "the configured file check timed out".to_string(),
                "the configured file check completes within the timeout".to_string(),
                "repo filesystem".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_CONTRACT_DRIFT" => (
                "repo signals differ from the declared contract".to_string(),
                "repo signals match the declared contract".to_string(),
                "detect".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_TASK_MUTATES_MANAGED_ISOLATED_PATH" => (
                "a task body appears to mutate an ota-managed isolated attachment path"
                    .to_string(),
                "task bodies leave ota-managed isolated attachment paths to the underlying tool"
                    .to_string(),
                "contract".to_string(),
                String::new(),
                String::new(),
            ),
            _ => (
                self.summary.clone(),
                self.why.clone(),
                "doctor".to_string(),
                String::new(),
                String::new(),
            ),
        };

        FindingEvidence {
            observed,
            expected,
            source,
            checked_at: String::new(),
            command,
            path,
        }
    }

    fn drift_context(&self) -> Option<DriftFindingContext<'_>> {
        if self.summary.starts_with("Contract drift:") {
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
        if self.policy_context().is_some() {
            return Some(FindingProvenanceContext {
                provenance: "org policy",
                provenance_key: "org_policy",
            });
        }

        if self.code() == "OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED" {
            return Some(FindingProvenanceContext {
                provenance: "repo signals",
                provenance_key: "repo_signals",
            });
        }

        if let Some(drift) = self.drift_context() {
            return Some(FindingProvenanceContext {
                provenance: drift.provenance,
                provenance_key: drift.provenance_key,
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

        match self.code() {
            "OTA_TASKS_MISSING"
            | "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY"
            | "OTA_LIFECYCLE_EPHEMERAL_ADVISORY"
            | "OTA_BACKEND_CLI_MISSING"
            | "OTA_CONTAINER_BACKEND_CLI_MISSING"
            | "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED"
            | "OTA_REMOTE_TARGET_SUSPICIOUS"
            | "OTA_SERVICE_CHECK_FAILED"
            | "OTA_SERVICE_CHECK_TIMED_OUT"
            | "OTA_SERVICE_UNVERIFIABLE"
            | "OTA_ENV_MISSING"
            | "OTA_ENV_INVALID"
            | "OTA_RUNTIME_VERSION_MISMATCH"
            | "OTA_RUNTIME_MISSING"
            | "OTA_RUNTIME_PROBE_FAILED"
            | "OTA_RUNTIME_VERSION_UNPARSEABLE"
            | "OTA_TOOL_VERSION_MISMATCH"
            | "OTA_TOOL_MISSING"
            | "OTA_TOOL_PROBE_FAILED"
            | "OTA_TOOL_VERSION_UNPARSEABLE"
            | "OTA_NATIVE_PREREQUISITE_MISSING"
            | "OTA_NATIVE_PREREQUISITE_TIMED_OUT"
            | "OTA_CHECK_FAILED"
            | "OTA_CHECK_TIMED_OUT"
            | "OTA_FILE_CHECK_FAILED"
            | "OTA_FILE_CHECK_TIMED_OUT"
            | "OTA_TASK_MUTATES_MANAGED_ISOLATED_PATH" => Some(FindingProvenanceContext {
                provenance: "repo contract",
                provenance_key: "repo_contract",
            }),
            "OTA_CONTAINER_APT_VERSION_UNAVAILABLE"
            | "OTA_CONTAINER_APT_PACKAGE_UNAVAILABLE"
            | "OTA_CONTAINER_APT_INDEX_UNAVAILABLE"
            | "OTA_CONTAINER_PROVISIONING_VERSION_UNAVAILABLE"
            | "OTA_CONTAINER_PROVISIONING_PACKAGE_UNAVAILABLE"
            | "OTA_CONTAINER_PROVISIONING_INDEX_UNAVAILABLE"
            | "OTA_CONTAINER_PROVISIONING_BACKEND_FAILED"
            | "OTA_REMOTE_APT_VERSION_UNAVAILABLE"
            | "OTA_REMOTE_APT_PACKAGE_UNAVAILABLE"
            | "OTA_REMOTE_APT_INDEX_UNAVAILABLE"
            | "OTA_REMOTE_PROVISIONING_VERSION_UNAVAILABLE"
            | "OTA_REMOTE_PROVISIONING_PACKAGE_UNAVAILABLE"
            | "OTA_REMOTE_PROVISIONING_INDEX_UNAVAILABLE"
            | "OTA_REMOTE_PROVISIONING_BACKEND_FAILED"
            | "OTA_HOST_PROVISIONING_VERSION_UNAVAILABLE"
            | "OTA_HOST_PROVISIONING_PACKAGE_UNAVAILABLE"
            | "OTA_HOST_PROVISIONING_INDEX_UNAVAILABLE"
            | "OTA_HOST_PROVISIONING_BACKEND_FAILED" => Some(FindingProvenanceContext {
                provenance: "org policy",
                provenance_key: "org_policy",
            }),
            _ => None,
        }
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
) -> DoctorReport {
    let mut findings = Vec::new();
    let mut provisioning = None;
    let mut adapter_bootstrap = None;
    let mut execution_target = None;
    let selected_lifecycle = doctor_selected_lifecycle(mode, lifecycle_override);
    let backend_precondition_selections =
        selected_backend_precondition_selections(contract, workflow_name, overrides);
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

    if matches!(scope, DoctorScope::All | DoctorScope::Preconditions) {
        diagnose_lifecycle(contract, mode, selected_lifecycle, &mut findings);
        let container_probe = diagnose_execution_backend(
            contract,
            &mut findings,
            mode,
            selected_lifecycle,
            overrides,
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
        } else if mode == DoctorMode::Container
            && contract_has_host_bound_readiness_surfaces(contract)
        {
            findings.push(container_mode_scope_note_finding(contract));
        }
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
                        &requirement_surface_with_toolchain_owned_tools(
                            contract,
                            &remote_probe.requirement_surface,
                            &precondition_selection.toolchain_names,
                            &remote_probe.target_os,
                        )
                        .tools,
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
            let tool_probe_started = diagnose_tools(
                &requirement_surface_with_toolchain_owned_tools(
                    contract,
                    &requirement_surface,
                    &precondition_selection.toolchain_names,
                    policy_target_os_for_mode(mode),
                )
                .tools,
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
                    )
                } else {
                    None
                };
                let additional_provisioning_actions = loaded_policy
                    .as_ref()
                    .map(|loaded| {
                        let policy_requirement_surface = policy_requirement_surface_for_toolchains(
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
                    .unwrap_or_default();
                if additional_mode == DoctorMode::Native {
                    diagnose_env(
                        contract,
                        loaded_policy
                            .as_ref()
                            .map(|loaded| loaded.pack.env_values()),
                        &declared_env_sources,
                        additional_selection
                            .env_scoped
                            .then_some(&additional_selection.env_names),
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
                    &requirement_surface_with_toolchain_owned_tools(
                        contract,
                        &additional_selection.requirement_surface,
                        &additional_selection.toolchain_names,
                        policy_target_os_for_mode(additional_mode),
                    )
                    .tools,
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
            && let Some(container_probe) = container_probe.as_ref()
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
                &mut findings,
            );
        }
        adapter_bootstrap = diagnose_adapter_bootstrap(loaded_policy.as_ref(), &mut findings);
    }
    if scope == DoctorScope::All {
        diagnose_tasks_surface(contract, &mut findings);
        diagnose_agent_boundary_review(contract, &mut findings);
        diagnose_contract_advisories(contract, &mut findings, overrides);
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
            diagnose_checks(contract, contract_path, scope, workflow_name, &mut findings);
        }
    }

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

    findings.push(Finding {
        severity,
        summary: String::from("No tasks defined in contract"),
        why: String::from(
            "without at least one task, `ota run <task>` cannot execute a repo entrypoint and the readiness contract is not operational for humans or agents",
        ),
        next: String::from(
            "run `ota detect --dry-run` to review inferred tasks before writing, or run `ota assist add-task --name dev --kind command` when you want one explicit runnable task",
        ),
    });
}

fn diagnose_contract_advisories(
    contract: &Contract,
    findings: &mut Vec<Finding>,
    overrides: ExecutionOverrides,
) {
    for advisory in collect_contract_advisories(contract) {
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
            ContractAdvisory::MutatesManagedIsolatedPath(advisory) => {
                ContractAdvisory::MutatesManagedIsolatedPath(advisory)
            }
        };

        findings.push(match advisory {
            ContractAdvisory::DependsOnBoundary(advisory) => Finding {
                severity: FindingSeverity::Warn,
                summary: format!(
                    "Task `{}` depends_on `{}` across different execution boundaries",
                    advisory.parent_task, advisory.dependency_task
                ),
                why: ContractAdvisory::DependsOnBoundary(advisory.clone()).why(),
                next: ContractAdvisory::DependsOnBoundary(advisory).next(),
            },
            ContractAdvisory::LikelyUnusedAttachment(advisory) => Finding {
                severity: FindingSeverity::Warn,
                summary: format!(
                    "Attachment `{}` may be unused in context `{}`",
                    advisory.isolated_path, advisory.context_name
                ),
                why: ContractAdvisory::LikelyUnusedAttachment(advisory.clone()).why(),
                next: ContractAdvisory::LikelyUnusedAttachment(advisory).next(),
            },
            ContractAdvisory::MutatesManagedIsolatedPath(advisory) => Finding {
                severity: FindingSeverity::Warn,
                summary: format!(
                    "Task `{}` mutates managed isolated path `{}`",
                    advisory.task_name, advisory.isolated_path
                ),
                why: ContractAdvisory::MutatesManagedIsolatedPath(advisory.clone()).why(),
                next: ContractAdvisory::MutatesManagedIsolatedPath(advisory).next(),
            },
        });
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

    findings.push(Finding {
        severity: FindingSeverity::Warn,
        summary: String::from("Agent boundary is inferred and unreviewed"),
        why: String::from(
            "`agent.inferred_boundary.reviewed: false` means Ota inferred the current writable and protected paths, but the repo owner has not confirmed that boundary yet",
        ),
        next: String::from(
            "review `agent.writable_paths` and `agent.protected_paths`, set `agent.inferred_boundary.reviewed: true`, then rerun `ota validate`",
        ),
    });
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
        findings.push(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Ephemeral lifecycle is execution-only"),
            why: String::from(
                "`execution.lifecycle: ephemeral` only applies to task execution. Diagnosis, healthchecks, and teardown are not covered.",
            ),
            next,
        });
    } else {
        findings.push(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Ephemeral lifecycle is advisory in native mode"),
            why: String::from(
                "`execution.lifecycle: ephemeral` is advisory in native mode only. Native execution still runs in the host shell.",
            ),
            next: String::from(
                "use `ota run <task>` for isolated execution; use `ota up` for readiness only",
            ),
        });
    }
}

fn diagnose_execution_backend(
    contract: &Contract,
    findings: &mut Vec<Finding>,
    mode: DoctorMode,
    lifecycle: Option<Lifecycle>,
    overrides: ExecutionOverrides,
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
        if let Some((_, context)) =
            execution_context_for_backend(contract, Backend::Container, lifecycle)
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
                    findings.push(Finding {
                        severity: FindingSeverity::Error,
                        summary: format!("Unsupported remote execution backend provider: {other}"),
                        why: format!(
                            "the contract requests remote diagnosis with provider `{other}`, but current Ota only supports built-in providers or a matching `backend_provider` extension"
                        ),
                        next: String::from(
                            "change `execution.backends.remote.provider` to `daytona`, `ssh`, `tsh`, or `kubectl`, or declare a matching `backend_provider` extension",
                        ),
                    });
                    return None;
                };

                if extension.kind != ExtensionKind::BackendProvider {
                    findings.push(Finding {
                        severity: FindingSeverity::Error,
                        summary: format!("Unsupported remote execution backend provider: {other}"),
                        why: format!(
                            "the contract requests remote diagnosis with provider `{other}`, but the matching extension is not a `backend_provider`"
                        ),
                        next: String::from(
                            "change the extension kind to `backend_provider` or change the remote provider name",
                        ),
                    });
                    return None;
                }

                if extension.api_version != 1 {
                    findings.push(Finding {
                        severity: FindingSeverity::Error,
                        summary: format!("Unsupported backend provider api_version: {other}"),
                        why: format!(
                            "the matching backend provider extension declares unsupported `api_version {}`",
                            extension.api_version
                        ),
                        next: String::from(
                            "bump the backend provider extension to `api_version: 1`",
                        ),
                    });
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
                        findings.push(Finding {
                            severity: FindingSeverity::Error,
                            summary: format!("Unsupported remote execution backend provider: {other}"),
                            why: format!(
                                "the contract requests `execution.preferred: remote` with provider `{other}`, but current Ota only supports built-in providers or a matching `backend_provider` extension"
                            ),
                            next: String::from(
                                "change `execution.backends.remote.provider` to `daytona`, `ssh`, `tsh`, or `kubectl`, or declare a matching `backend_provider` extension",
                            ),
                        });
                        return None;
                    };

                    if extension.kind != ExtensionKind::BackendProvider {
                        findings.push(Finding {
                            severity: FindingSeverity::Error,
                            summary: format!("Unsupported remote execution backend provider: {other}"),
                            why: format!(
                                "the contract requests `execution.preferred: remote` with provider `{other}`, but the matching extension is not a `backend_provider`"
                            ),
                            next: String::from(
                                "change the extension kind to `backend_provider` or change the remote provider name",
                            ),
                        });
                        return None;
                    }

                    if extension.api_version != 1 {
                        findings.push(Finding {
                            severity: FindingSeverity::Error,
                            summary: format!("Unsupported backend provider api_version: {other}"),
                            why: format!(
                                "the matching backend provider extension declares unsupported `api_version {}`",
                                extension.api_version
                            ),
                            next: String::from(
                                "bump the backend provider extension to `api_version: 1`",
                            ),
                        });
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
    Finding {
        severity: FindingSeverity::Error,
        summary: String::from("Container execution is not configured"),
        why: String::from(
            "container diagnosis requires `execution.backends.container.image` so Ota can inspect the execution image that actually runs tasks",
        ),
        next: String::from(
            "add `execution.backends.container.image`, then rerun `ota doctor --mode container`",
        ),
    }
}

fn remote_mode_not_configured_finding() -> Finding {
    Finding {
        severity: FindingSeverity::Error,
        summary: String::from("Remote execution is not configured"),
        why: String::from(
            "remote diagnosis requires `execution.backends.remote.provider` and a targetable remote execution context so Ota can inspect the remote backend that actually runs tasks",
        ),
        next: String::from(
            "add `execution.backends.remote.provider` plus `execution.backends.remote.target`, or rerun `ota doctor` without `--mode remote`",
        ),
    }
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
    if !contract.env.vars.is_empty() {
        skipped.push("env requirements");
    }
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

    let verb = if skipped.len() == 1 {
        "remains"
    } else {
        "remain"
    };
    let skipped = skipped.join(", ");
    Finding {
        severity: FindingSeverity::Info,
        summary: String::from("Container readiness does not include host-only checks"),
        why: format!(
            "container mode validated the selected execution image and container execution path; {skipped} {verb} host-bound and would mix contexts"
        ),
        next: String::from(
            "use `ota doctor --mode native` for host readiness, or run declared tasks with `ota run <task> --mode container` through the validated container path",
        ),
    }
}

fn remote_mode_scope_note_finding() -> Finding {
    Finding {
        severity: FindingSeverity::Info,
        summary: String::from(
            "Remote execution contexts are only partially evaluated in native mode",
        ),
        why: String::from(
            "native doctor mode can validate remote backend declarations and run contextual readiness probes from executable remote contexts, but runtime and tool version checks still evaluate the local host rather than the declared remote environment",
        ),
        next: String::from(
            "use `ota doctor --mode remote` to probe remote contexts directly, and `ota execution plan --mode remote` to inspect the remote backend contract when debugging topology",
        ),
    }
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
    Some(Finding {
        severity: FindingSeverity::Info,
        summary: String::from("Host-bound readiness checks are not evaluated in remote mode"),
        why: format!(
            "remote mode checks the declared remote execution backend; {skipped} {verb} host-bound and would mix contexts"
        ),
        next: String::from(
            "use `ota doctor --mode native` for host readiness, or declare `services.<name>.readiness.from` on an executable context for topology-aware remote checks",
        ),
    })
}

fn diagnose_remote_target_shape(provider: &str, target: &str, findings: &mut Vec<Finding>) {
    let target = target.trim();
    if target.is_empty() {
        return;
    }

    match provider {
        "ssh" | "tsh" => {
            if !target.contains('@') {
                findings.push(Finding {
                    severity: FindingSeverity::Warn,
                    summary: format!("Suspicious remote target for {provider}: {target}"),
                    why: format!(
                        "remote provider `{provider}` usually expects a `user@host` style target, but current target `{target}` has no `@` separator"
                    ),
                    next: format!(
                        "set `execution.backends.remote.target` to a host target such as `user@host` for provider `{provider}`"
                    ),
                });
            }
        }
        "kubectl" => {
            if !target.starts_with("pod/") {
                findings.push(Finding {
                    severity: FindingSeverity::Warn,
                    summary: format!("Suspicious remote target for kubectl: {target}"),
                    why: format!(
                        "remote provider `kubectl` is currently validated for `pod/<name>` style targets, but current target `{target}` does not start with `pod/`"
                    ),
                    next: String::from(
                        "set `execution.backends.remote.target` to a pod target such as `pod/ota-dev`",
                    ),
                });
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
            && !selected.contains(name.as_str())
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
            Ok(CheckStatus::Failed) => Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Service readiness failed: {name}"),
                why: service_readiness_failure_why(
                    name,
                    service.endpoint_for_context(from_context),
                    from_context,
                ),
                next: match service.start_command(name) {
                    Some(start) => format!("run `{start}` and rerun `{rerun_doctor}`"),
                    None => format!(
                        "repair `{name}` from context `{}` and rerun `{rerun_doctor}`",
                        from_context,
                    ),
                },
            }),
            Ok(CheckStatus::TimedOut(_)) => None,
            Err(error) => Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Service readiness context is not executable: {name}"),
                why: service_readiness_execution_why(
                    name,
                    service.endpoint_for_context(from_context),
                    from_context,
                    &error,
                ),
                next: service_readiness_execution_next(name, from_context, mode, lifecycle),
            }),
        };
    }

    if let Some(healthcheck) = service.healthcheck.as_deref() {
        if mode != DoctorMode::Native {
            return None;
        }
        return match run_service_healthcheck(name, service, working_dir, healthcheck) {
            CheckStatus::Passed => None,
            CheckStatus::Failed => Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Service healthcheck failed: {name}"),
                why: format!("service `{name}` did not pass its configured healthcheck"),
                next: match service.start_command(name) {
                    Some(start) => format!("run `{start}` and rerun `{rerun_doctor}`"),
                    None => format!(
                        "start or repair `{name}` and rerun its healthcheck: {healthcheck}, then rerun `{rerun_doctor}`"
                    ),
                },
            }),
            CheckStatus::TimedOut(timeout) => Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Service healthcheck timed out: {name}"),
                why: format!("service `{name}` did not become ready within {}ms", timeout),
                next: format!(
                    "make `services.{name}.healthcheck` complete faster or raise `services.{name}.timeout`, then rerun `{rerun_doctor}`"
                ),
            }),
        };
    }

    if service.required {
        let can_anchor_structured_readiness = service
            .readiness
            .as_ref()
            .and_then(ServiceReadinessSpec::from_context)
            .is_some()
            || service.endpoints.contains_key("host")
            || service.endpoints.len() == 1;

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
            format!(
                "declare readiness with `ota assist declare-readiness --service {name} --style tcp` or `--style http`, then rerun `ota doctor`"
            )
        } else {
            format!(
                "refine the managed service with `ota assist declare-service --name {name} --style tcp` or `--style http`, then rerun `ota doctor`"
            )
        };

        return Some(Finding {
            severity: FindingSeverity::Warn,
            summary: format!("Required service cannot be verified: {name}"),
            why,
            next,
        });
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
    let (producer_contract, producer_contract_path) = match load_contract_for_workspace_repo_ref(
        contract_path,
        producer.repo.as_str(),
        "producer.repo",
    ) {
        Ok(value) => value,
        Err(error) => {
            return Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Required service cannot be verified: {name}"),
                why: format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but Ota could not load that producer contract: {}",
                    producer.repo, producer.task, error
                ),
                next: format!(
                    "repair workspace repo `{}` or run `ota workspace up`, then rerun `{rerun_doctor}`",
                    producer.repo
                ),
            });
        }
    };
    let producer_task = match producer_contract.tasks.get(producer.task.as_str()) {
        Some(task) => task,
        None => {
            return Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Required service cannot be verified: {name}"),
                why: format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but that task is not declared",
                    producer.repo, producer.task
                ),
                next: format!(
                    "repair workspace repo `{}` task `{}` or run `ota workspace up`, then rerun `{rerun_doctor}`",
                    producer.repo, producer.task
                ),
            });
        }
    };
    let listener_name = match resolve_producer_service_listener_name(
        producer_task,
        producer.listener.as_deref(),
    ) {
        Ok(name) => name,
        Err(error) => {
            return Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Required service cannot be verified: {name}"),
                why: format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but {}",
                    producer.repo, producer.task, error
                ),
                next: format!(
                    "repair workspace repo `{}` task `{}` or refine `services.{name}.producer.listener`, then rerun `{rerun_doctor}`",
                    producer.repo, producer.task
                ),
            });
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
            return Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Required service cannot be verified: {name}"),
                why: format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but Ota could not resolve that producer runtime: {}",
                    producer.repo, producer.task, error
                ),
                next: format!(
                    "repair workspace repo `{}` task `{}` or run `ota workspace up`, then rerun `{rerun_doctor}`",
                    producer.repo, producer.task
                ),
            });
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
            return Some(Finding {
                severity: if service.required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!("Required service cannot be verified: {name}"),
                why: format!(
                    "service `{name}` is owned by workspace repo `{}` task `{}`, but {}",
                    producer.repo, producer.task, error
                ),
                next: format!(
                    "repair workspace repo `{}` task `{}` listener `{}` or run `ota workspace up`, then rerun `{rerun_doctor}`",
                    producer.repo, producer.task, listener_name
                ),
            });
        }
    };
    if host_runtime_readiness_observed(&probe, None) {
        return None;
    }

    Some(Finding {
        severity: if service.required {
            FindingSeverity::Error
        } else {
            FindingSeverity::Warn
        },
        summary: format!("Service producer is not ready: {name}"),
        why: format!(
            "service `{name}` is owned by workspace repo `{}` task `{}` listener `{}` and its projected host endpoint `{}:{}` is not ready",
            producer.repo, producer.task, probe.listener, probe.address, probe.port
        ),
        next: format!(
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
    })
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

fn run_service_readiness(
    contract: &Contract,
    name: &str,
    service: &ServiceSpec,
    working_dir: &Path,
    readiness: &crate::schema::ServiceReadinessSpec,
) -> Result<CheckStatus, RunError> {
    let Some(from_context) = readiness.from_context() else {
        return Ok(CheckStatus::Failed);
    };
    let backend = resolve_context_execution_backend(contract, from_context)?;

    if let Some(probe_name) = readiness
        .probe
        .as_deref()
        .map(str::trim)
        .filter(|name| !name.is_empty())
    {
        let Ok(resolved) = resolve_named_readiness_probe_contract(contract, probe_name) else {
            return Ok(CheckStatus::Failed);
        };
        let Some(endpoint) = service.endpoint_for_context(from_context) else {
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
    let Some(endpoint) = service.endpoint_for_context(from_context) else {
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
                crate::schema::TaskRuntimeReadinessKind::Http => {
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
                crate::schema::TaskRuntimeReadinessKind::Tcp => {
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

fn structured_service_readiness_command(
    readiness: &crate::schema::ServiceReadinessSpec,
    endpoint: &crate::schema::ServiceEndpointSpec,
    kind: crate::schema::TaskRuntimeReadinessKind,
) -> String {
    match kind {
        crate::schema::TaskRuntimeReadinessKind::Http => {
            service_http_readiness_probe_command(readiness, endpoint)
        }
        crate::schema::TaskRuntimeReadinessKind::Tcp => {
            service_tcp_readiness_probe_command(readiness, endpoint)
        }
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
            DeclaredEnvSourceStatus::MissingRequired => findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: format!("Missing required environment source: {}", source.label()),
                why: format!(
                    "the repo declares `{}:{}` with `must_exist: true`, but that file is missing",
                    source.kind, source.path
                ),
                next: format!(
                    "create `{}` or update `env.sources`, then rerun `ota doctor`",
                    source.path
                ),
            }),
            DeclaredEnvSourceStatus::ParseFailed => findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: format!("Environment source parse failed: {}", source.label()),
                why: format!(
                    "ota could not read declared source `{}:{}`: {}",
                    source.kind,
                    source.path,
                    source.details.as_deref().unwrap_or("unknown parse error")
                ),
                next: format!(
                    "fix `{}` so ota can parse declared source `{}`, then rerun `ota doctor`",
                    source.path
                    ,
                    source.label()
                ),
            }),
            DeclaredEnvSourceStatus::InvalidStructure => findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: format!("Environment source has invalid structure: {}", source.label()),
                why: format!(
                    "declared source `{}` loaded as text, but its structure is not supported: {}",
                    source.label(),
                    source.details.as_deref().unwrap_or("unknown structure error")
                ),
                next: format!(
                    "replace unsupported values in `{}` with scalar env-shaped values only, then rerun `ota doctor`",
                    source.path
                ),
            }),
            DeclaredEnvSourceStatus::Collision => findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: format!("Environment source key collision: {}", source.label()),
                why: format!(
                    "declared source `{}` contains multiple keys that normalize to the same env name: {}",
                    source.label(),
                    source.details.as_deref().unwrap_or("unknown collision")
                ),
                next: format!(
                    "rename the colliding keys in `{}` so each normalized env name is unique, then rerun `ota doctor`",
                    source.path
                ),
            }),
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
                    findings.push(Finding {
                        severity: FindingSeverity::Error,
                        summary: format!("Invalid environment value: {name}"),
                        why: format!(
                            "{name} resolved to `{value}`, which is outside the allowed values"
                        ),
                        next: format!(
                            "run `ota env` to inspect the resolved source for {name}, then set {name} to one of: {}",
                            requirement.allowed.join(", ")
                        ),
                    });
                }
            }
            None if required_for_selected_path => findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: format!("Missing environment variable: {name}"),
                why: if requirement.required {
                    format!("{name} is required by this repo contract")
                } else {
                    format!("{name} is required by the selected task or workflow path")
                },
                next: format!(
                    "run `ota env` to inspect the current precedence, then set {name} in policy env, the shell, or a declared env source before running tasks"
                ),
            }),
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

        container_probe_started |= diagnose_command_version(
            "runtime",
            name,
            name,
            requirement.version_for_os(target_os),
            required,
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
            provisioning_actions,
            findings,
        );
    }
    container_probe_started
}

fn diagnose_tools(
    tools: &BTreeMap<String, ToolRequirement>,
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
    for (name, requirement) in tools {
        if !requirement.active_for_os(target_os) {
            continue;
        }
        let required = requirement.required_for_os(target_os);

        container_probe_started |= diagnose_command_version(
            "tool",
            name,
            tool_executable_name(name),
            requirement.version_for_os(target_os),
            required,
            None,
            requirement.acquisition(),
            mode,
            selected_lifecycle,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            loaded_policy,
            target_os,
            provisioning_actions,
            findings,
        );
    }
    container_probe_started
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

        probe_started |= diagnose_command_version(
            "runtime",
            toolchain_name,
            provider.primary_executable(),
            toolchain.version_for_os(target_os),
            toolchain.required_for_os(target_os),
            Some(provider.provider_hint()),
            None,
            mode,
            None,
            container_probe,
            remote_probe,
            remote_context_name,
            contract_path,
            None,
            target_os,
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
    let output = run_backend_command_captured(
        probe_name,
        command,
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
    Finding {
        severity: FindingSeverity::Error,
        summary: narrative.summary,
        why: narrative.why,
        next: narrative.next,
    }
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
    Finding {
        severity: FindingSeverity::Error,
        summary: narrative.summary,
        why: narrative.why,
        next: narrative.next,
    }
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
    Finding {
        severity: FindingSeverity::Error,
        summary: narrative.summary,
        why: narrative.why,
        next: narrative.next,
    }
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
        let Some(check_name) = prerequisite.check_for_os(target_os) else {
            continue;
        };
        let Some(check) = contract
            .checks
            .iter()
            .find(|check| check.name == check_name)
        else {
            continue;
        };
        match run_native_prerequisite_check(prerequisite, name, target_os, check, working_dir) {
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
    Finding {
        severity: if prerequisite.required {
            FindingSeverity::Error
        } else {
            FindingSeverity::Warn
        },
        summary,
        why: format!(
            "{description}; {check_context} on {target_os}, so ota cannot prove the native prerequisite is available{failure_suffix}"
        ),
        next: native_prerequisite_next(name, prerequisite, target_os),
    }
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
    if platform.visual_studio_build_tools {
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
    if !platform.packages.is_empty() {
        suggestions.push(format!(
            "install packages: `{}`",
            platform.packages.join(" ")
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

fn diagnose_org_policy(
    contract: &Contract,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    policy_os: &str,
    requirement_surface: &RequirementSurface,
    toolchain_names: &BTreeSet<String>,
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
    if missing_sections.is_empty() && missing_files.is_empty() && version_violations.is_empty() {
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

            findings.push(Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Policy-backed version rules are declared"),
                why: format!(
                    "`{}` declares approved repo version rules: {}",
                    compact_display_path(&policy_path),
                    rules.join(", ")
                ),
                next: String::from(
                    "use `ota policy review` to inspect the active policy source, or keep these approved version rules in mind when repo runtimes or tools need a governed version",
                ),
            });
        }

        let provisioning_plan = policy_pack
            .provisioning_plan_for_requirement_surface_os(policy_os, &policy_requirement_surface);
        let provisioning_request = ProvisioningBackendRequest {
            actions: policy_pack.selected_provisioning_actions_for_requirement_surface_os(
                policy_os,
                &policy_requirement_surface,
            ),
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
            findings.push(Finding {
                severity: FindingSeverity::Warn,
                summary: String::from("Policy provisioning needs explicit package identifiers"),
                why: format!(
                    "policy-backed provisioning cannot proceed for {}",
                    missing_packages.join("; ")
                ),
                next: String::from(
                    "add `package` to the matching `policies.provisioning.<name>` rule or platform override, then rerun `ota doctor`",
                ),
            });
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

            findings.push(Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Policy-backed provisioning sources are declared"),
                why: if matched_targets.is_empty() {
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
                next: String::from(
                    "use `ota policy review` to inspect the active policy source, or keep these approved sources in mind when repo prerequisites need a governed install path",
                ),
            });
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

    findings.push(Finding {
        severity: FindingSeverity::Error,
        summary: String::from("Repo does not satisfy org policy pack"),
        why: format!(
            "`{}` requires {}",
            compact_display_path(&policy_path),
            why_parts.join(" and ")
        ),
        next: if version_violations.is_empty() {
            format!(
                "add the missing items or update `{}`",
                compact_display_path(&policy_path)
            )
        } else if missing_sections.is_empty() && missing_files.is_empty() {
            format!(
                "update the repo contract versions or widen `{}`",
                compact_display_path(&policy_path)
            )
        } else {
            format!(
                "add the missing items, update the repo contract versions, or update `{}`",
                compact_display_path(&policy_path)
            )
        },
    });

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

        findings.push(Finding {
            severity: FindingSeverity::Error,
            summary: String::from("Repo does not satisfy org policy pack"),
            why: format!(
                "`{}` requires {}",
                compact_display_path(policy_path),
                why_parts.join(" and ")
            ),
            next: format!(
                "add the missing items or update `{}`",
                compact_display_path(policy_path)
            ),
        });
        return;
    }

    for remote_probe in remote_probe_contexts {
        let context_label = remote_policy_subject(remote_probe.context_name.as_deref());
        let version_violations = policy_pack.version_policy_violations_for_requirement_surface_os(
            &remote_probe.target_os,
            &remote_probe.policy_requirement_surface,
        );
        if !version_violations.is_empty() {
            findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: String::from("Repo does not satisfy org policy pack"),
                why: format!(
                    "`{}` requires {context_label} to stay within approved versions, but version policy violations: {}",
                    compact_display_path(policy_path),
                    version_violations.join("; ")
                ),
                next: format!(
                    "update the requirements for {context_label}, or widen `{}`",
                    compact_display_path(policy_path)
                ),
            });
            continue;
        }

        let version_rules = policy_version_rules_for_requirement_surface_os(
            policy_pack,
            &remote_probe.target_os,
            &remote_probe.policy_requirement_surface,
        );
        if !version_rules.is_empty() {
            findings.push(Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Policy-backed version rules are declared"),
                why: format!(
                    "`{}` declares approved repo version rules for {context_label}: {}",
                    compact_display_path(policy_path),
                    version_rules.join(", ")
                ),
                next: String::from(
                    "use `ota policy review` to inspect the active policy source, or keep these approved version rules in mind when remote context requirements need a governed version",
                ),
            });
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
            findings.push(Finding {
                severity: FindingSeverity::Warn,
                summary: String::from("Policy provisioning needs explicit package identifiers"),
                why: format!(
                    "policy-backed provisioning cannot proceed for {context_label}: {}",
                    missing_packages.join("; ")
                ),
                next: format!(
                    "add `package` to the matching `policies.provisioning.<name>` rule or platform override, then rerun `ota doctor --mode remote` for {context_label}",
                ),
            });
        }

        if !remote_probe.provisioning_actions.is_empty() {
            let matched_targets: Vec<String> = remote_probe
                .provisioning_actions
                .iter()
                .map(provisioning_action_audit_summary)
                .collect();
            findings.push(Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Policy-backed provisioning sources are declared"),
                why: format!(
                    "`{}` declares approved provisioning sources for {context_label}: {}. This repo's declared prerequisites can be provisioned through: {}",
                    compact_display_path(policy_path),
                    matched_targets.join(", "),
                    matched_targets.join(", ")
                ),
                next: String::from(
                    "use `ota policy review` to inspect the active policy source, or keep these approved sources in mind when remote context prerequisites need a governed install path",
                ),
            });
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

        findings.push(Finding {
            severity: FindingSeverity::Info,
            summary: String::from("Adapter bootstrap sources are declared"),
            why: format!(
                "`{}` can bootstrap missing adapter binaries through: {}",
                compact_display_path(&policy_path),
                sources.join(", ")
            ),
            next: String::from(
                "use `ota policy review` to inspect the active policy source, or keep these approved bootstrap surfaces in mind when adapter install needs approval or audit",
            ),
        });
    }

    Some(AdapterBootstrapDiagnostics { plan, request })
}

fn policy_error_finding(err: LoadPolicyPackError) -> Finding {
    Finding {
        severity: FindingSeverity::Error,
        summary: String::from("Invalid org policy pack"),
        why: err.to_string(),
        next: format!(
            "repair `{}` and rerun `ota doctor`",
            compact_display_path(Path::new(err.path()))
        ),
    }
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
            if let Some(version) = dotnet_global_json_version(&contract_root) {
                return Some(dotnet_install_command(&version));
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
            if let Some(version) = dotnet_global_json_version(&contract_root) {
                return Some(dotnet_install_command(&version));
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

    Some(Finding {
        severity: FindingSeverity::Warn,
        summary: format!("Managed toolchain opportunity: {ecosystem}"),
        why: format!(
            "this repo uses {ecosystem_label} and currently models it through {fallback_model}; repo signals: {signal_summary}; ota does not ship a {ecosystem} toolchain provider yet"
        ),
        next: format!(
            "keep {fallback_model} for now; ota can model this more cleanly once {ecosystem} toolchain support is shipped"
        ),
    })
}

fn diagnose_unsupported_toolchain_opportunities(
    contract: &Contract,
    contract_path: &Path,
    requirement_surface: &RequirementSurface,
    findings: &mut Vec<Finding>,
) {
    let contract_root = contract_working_dir(contract_path);
    for ecosystem in ["python"] {
        if contract.toolchains.contains_key(ecosystem) {
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

fn dotnet_install_command(version: &str) -> String {
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

fn diagnose_command_version(
    kind: &str,
    display_name: &str,
    executable_name: &str,
    requirement: &str,
    required: bool,
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
    provisioning_actions: &[ProvisioningAction],
    findings: &mut Vec<Finding>,
) -> bool {
    let rerun_doctor = rerun_doctor_command(mode, selected_lifecycle);
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
        Some(command_version_probe(executable_name))
    } else if mode == DoctorMode::Container {
        let Some(container_probe) = container_probe else {
            return false;
        };
        Some(command_version_probe_in_container(
            &container_probe.engine,
            &container_probe.image,
            executable_name,
            contract_working_dir(contract_path),
        ))
    } else if mode == DoctorMode::Remote {
        let Some(remote_probe) = remote_probe else {
            return false;
        };
        Some(command_version_probe_in_remote(
            remote_probe,
            executable_name,
            contract_working_dir(contract_path),
        ))
    } else {
        None
    };
    let probe_started = version_probe
        .as_ref()
        .map(|probe| probe.probe_started)
        .unwrap_or(false);
    let finding_display_name = remote_context_name
        .map(|context_name| format!("{display_name} (context {context_name})"))
        .unwrap_or_else(|| display_name.to_string());
    let acquisition_provider_missing = matches!(mode, DoctorMode::Native)
        && tool_acquisition.is_some_and(|acquisition| {
            !command_available(tool_acquisition_provider_requirement(acquisition))
        });
    let actual = if let Some(probe) = version_probe.as_ref() {
        match &probe.outcome {
            CommandVersionProbeOutcome::Version(actual) => Some(actual.clone()),
            _ => None,
        }
    } else {
        None
    };

    let Some(actual) = actual else {
        if acquisition_provider_missing && let Some(acquisition) = tool_acquisition {
            let provider_requirement = tool_acquisition_provider_requirement(acquisition);
            findings.push(Finding {
                severity: if required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!(
                    "Missing tool activation provider: {provider_requirement}"
                ),
                why: format!(
                    "{display_name} is required by the selected workflow/task prerequisites, but the contract acquires it through `{}` and `{provider_requirement}` is not available on PATH",
                    match acquisition.provider {
                        ToolAcquisitionProvider::Corepack => "corepack",
                        ToolAcquisitionProvider::Command => "command activation",
                    }
                ),
                next: format!(
                    "install `{provider_requirement}` or change the tool acquisition path, then run `{}` and rerun `{rerun_doctor}`",
                    tool_acquisition_command(acquisition)
                ),
            });
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
                    let resolved_path = probe
                        .resolved_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| executable_name.to_string());
                    let (why, next) = match mode {
                        DoctorMode::Container => {
                            let image = container_probe
                                .map(|probe| probe.image.as_str())
                                .unwrap_or("unknown");
                            let why = match (error.as_deref(), exit_code) {
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
                            };
                            let next = format!(
                                "run `{}` inside the selected container image, inspect `{resolved_path}`, and make sure the probe succeeds before rerunning `{}`",
                                probe.command, rerun_doctor
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
                    findings.push(Finding {
                        severity: if required {
                            FindingSeverity::Error
                        } else {
                            FindingSeverity::Warn
                        },
                        summary: format!(
                            "{} probe failed: {finding_display_name}",
                            kind_label(kind)
                        ),
                        why,
                        next,
                    });
                    return probe_started;
                }
                CommandVersionProbeOutcome::Unparseable => {
                    let resolved_path = probe
                        .resolved_path
                        .as_ref()
                        .map(|path| path.display().to_string())
                        .unwrap_or_else(|| executable_name.to_string());
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
                    findings.push(Finding {
                        severity: if required {
                            FindingSeverity::Error
                        } else {
                            FindingSeverity::Warn
                        },
                        summary: format!("Unparseable version for {kind}: {finding_display_name}"),
                        why,
                        next,
                    });
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
        let container_image = container_probe.map(|probe| probe.image.as_str());
        findings.push(Finding {
            severity: if required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            summary: format!("Missing {kind}: {finding_display_name}"),
            why: match (mode, container_image) {
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
            next: match (mode, container_image) {
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
        });
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
            findings.push(Finding {
                severity: if required {
                    FindingSeverity::Error
                } else {
                    FindingSeverity::Warn
                },
                summary: format!(
                    "Installed {kind} is not compliant with org policy: {finding_display_name}"
                ),
                why: format!(
                    "{display_name} resolved to `{actual}` and satisfies the repo contract `{requirement}`, but `{}` enforces strict version compliance and {policy_violation}{probe_suffix}",
                    compact_display_path(&loaded_policy.path)
                ),
                next: match mode {
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
            });
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
        let provider_requirement = tool_acquisition_provider_requirement(acquisition);
        findings.push(Finding {
            severity: if required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            summary: format!("Missing tool activation provider: {provider_requirement}"),
            why: format!(
                "{display_name} is required by the selected workflow/task prerequisites, but the contract upgrades it through `{}` and `{provider_requirement}` is not available on PATH",
                match acquisition.provider {
                    ToolAcquisitionProvider::Corepack => "corepack",
                    ToolAcquisitionProvider::Command => "command activation",
                }
            ),
            next: format!(
                "install `{provider_requirement}` or change the tool acquisition path, then run `{}` and rerun `{rerun_doctor}`",
                tool_acquisition_command(acquisition)
            ),
        });
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
    findings.push(Finding {
        severity: if required {
            FindingSeverity::Error
        } else {
            FindingSeverity::Warn
        },
        summary: format!("Version mismatch for {kind}: {finding_display_name}"),
        why: match (mode, container_image) {
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
        next: match (mode, container_image) {
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
            _ => exact_remediation
                .map(|command| format!("run `{command}` and rerun `{rerun_doctor}`"))
                .unwrap_or_else(|| {
                    format!(
                        "install a compatible {display_name} version that satisfies `{requirement}`, then rerun `{rerun_doctor}`"
                    )
                }),
        },
    });
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
    let action = provisioning_actions.iter().find(|action| {
        action.target_kind == target_kind
            && action.name == display_name
            && action.requested_version == requirement
    })?;
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
    let action = provisioning_actions.iter().find(|action| {
        action.target_kind == target_kind
            && action.name == display_name
            && action.requested_version == requirement
    })?;
    match probe_provisioning_installability_with_target(
        action,
        contract_working_dir(contract_path),
        &target,
    ) {
        Err(ProvisioningBackendError::DiagnosedCommandFailed { diagnosis, .. }) => Some(diagnosis),
        _ => None,
    }
}

fn version_command_in_container(
    engine: &str,
    image: &str,
    name: &str,
    working_dir: &Path,
) -> Command {
    let container_name = crate::runner::ephemeral_container_name(working_dir, image, engine);
    let mut command = container_engine_command(engine);
    command
        .arg("run")
        .arg("--rm")
        .arg("--name")
        .arg(container_name)
        .arg("--entrypoint")
        .arg("sh")
        .arg(image)
        .arg("-c")
        .arg(version_probe_command_string(name));
    command
}

fn version_command_string(name: &str) -> String {
    if name == "go" {
        String::from("go version")
    } else {
        format!("{name} --version")
    }
}

fn version_probe_command_string(name: &str) -> String {
    let quoted_name = shell_single_quote(name);
    let exec = if name == "go" {
        String::from("\"$resolved\" version")
    } else {
        String::from("\"$resolved\" --version")
    };

    format!(
        "printf '%s\\n' '{CONTAINER_PROBE_STARTED_MARKER}' >&2\nresolved=\"$(command -v {quoted_name} 2>/dev/null)\" || exit 127\nprintf '%s%s\\n' '{CONTAINER_PROBE_PATH_MARKER}' \"$resolved\" >&2\nexec {exec}"
    )
}

fn command_version_probe_in_container(
    engine: &str,
    image: &str,
    name: &str,
    working_dir: &Path,
) -> CommandVersionProbe {
    let command = version_command_string(name);
    let output = version_command_in_container(engine, image, name, working_dir).output();
    let (probe_started, resolved_path, outcome) = match output {
        Ok(output) => {
            let (probe_started, resolved_path, combined) =
                extract_container_probe_output(&output.stdout, &output.stderr);
            let outcome = if output.status.success() {
                extract_version_token(&combined)
                    .map(CommandVersionProbeOutcome::Version)
                    .unwrap_or(CommandVersionProbeOutcome::Unparseable)
            } else if probe_started && resolved_path.is_none() {
                CommandVersionProbeOutcome::Missing
            } else {
                CommandVersionProbeOutcome::ProbeFailed {
                    exit_code: output.status.code(),
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

    findings.push(Finding {
        severity: FindingSeverity::Error,
        summary: format!("Missing execution backend CLI: {name}"),
        why: format!("{backend} requires `{name}` to be available on PATH"),
        next: format!("install {name} and make it available on PATH, then rerun `ota doctor`"),
    });
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
        findings.push(Finding {
            severity: FindingSeverity::Error,
            summary: format!("Missing container execution backend CLI: {supported}"),
            why: format!(
                "container execution requires one of these CLIs to be available on PATH: {supported}"
            ),
            next: String::from(
                "install one of the supported container engines or use `--mode native` if the contract allows it, then rerun `ota doctor`",
            ),
        });
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
    Finding {
        severity: FindingSeverity::Error,
        summary: format!("Container execution backend unavailable: {engine}"),
        why: format!(
            "container execution resolved `{engine}`, but `{} info` could not reach a usable container backend: {details}",
            engine
        ),
        next: String::from(
            "start or repair the selected container engine, or use `--mode native` if the contract allows it, then rerun `ota doctor`",
        ),
    }
}

fn tool_executable_name(name: &str) -> &str {
    match name {
        "maven" => "mvn",
        _ => name,
    }
}

fn diagnose_checks(
    contract: &Contract,
    contract_path: &Path,
    scope: DoctorScope,
    workflow_name: Option<&str>,
    findings: &mut Vec<Finding>,
) {
    let working_dir = contract_working_dir(contract_path);
    let selected_checks = selected_workflow_check_names(contract, workflow_name, scope);
    let selected_precondition_checks =
        selected_task_requirement_check_names(contract, workflow_name);
    let selected_probes = selected_workflow_probe_names(contract, workflow_name, scope);
    let selected_surfaces = selected_workflow_surface_names(contract, workflow_name, scope);
    let scoped_workflow_targets = selected_checks.is_some()
        || selected_precondition_checks.is_some()
        || selected_probes.is_some()
        || selected_surfaces.is_some();
    let mut probes_executed_via_checks = BTreeSet::new();

    for check in &contract.checks {
        let is_selected_precondition = selected_precondition_checks
            .as_ref()
            .is_some_and(|selected| selected.contains(check.name.as_str()));

        if scope == DoctorScope::Preconditions && check.kind != CheckKind::Precondition {
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
                if check.kind == CheckKind::Precondition {
                    continue;
                }
                if scoped_workflow_targets && selected_checks.is_none() {
                    continue;
                }
                if let Some(selected) = selected_checks.as_ref()
                    && !selected.contains(check.name.as_str())
                {
                    continue;
                }
            }
        } else {
            if check.kind == CheckKind::Precondition {
                if let Some(selected) = selected_precondition_checks.as_ref()
                    && !selected.contains(check.name.as_str())
                {
                    continue;
                }
            } else if scoped_workflow_targets && selected_checks.is_none() {
                continue;
            } else if let Some(selected) = selected_checks.as_ref()
                && !selected.contains(check.name.as_str())
            {
                continue;
            }
        }

        if let Some(probe_name) = check.probe.as_deref() {
            probes_executed_via_checks.insert(probe_name.to_string());
        }

        match run_declared_check(contract, contract_path, check, working_dir, None) {
            CheckStatus::Passed => continue,
            CheckStatus::Failed => findings.push(Finding {
                severity: map_check_severity(check.severity),
                summary: failed_check_summary(check),
                why: failed_check_why(contract, check),
                next: failed_check_next(contract, workflow_name, check),
            }),
            CheckStatus::TimedOut(timeout) => findings.push(Finding {
                severity: map_check_severity(check.severity),
                summary: timed_out_check_summary(check),
                why: timed_out_check_why(contract, check, timeout),
                next: timed_out_check_next(contract, check),
            }),
        }
    }

    if let Some(selected_probe_names) = selected_probes {
        for probe_name in selected_probe_names {
            if probes_executed_via_checks.contains(probe_name) {
                continue;
            }
            if contract.probe(probe_name).is_none() {
                continue;
            }
            let probe = contract
                .probe(probe_name)
                .expect("checked probe existence above");
            match run_named_probe(contract, contract_path, probe_name, None) {
                CheckStatus::Passed => continue,
                CheckStatus::Failed => findings.push(Finding {
                    severity: FindingSeverity::Error,
                    summary: format!("Probe failed: {probe_name}"),
                    why: format!(
                        "the configured workflow readiness probe `{probe_name}` ({}) did not succeed",
                        probe_source_description(contract, probe_name)
                    ),
                    next: failed_probe_next(probe_name, probe),
                }),
                CheckStatus::TimedOut(timeout) => findings.push(Finding {
                    severity: FindingSeverity::Error,
                    summary: format!("Probe timed out: {probe_name}"),
                    why: format!(
                        "the configured workflow readiness probe `{probe_name}` ({}) did not finish within {}ms",
                        probe_source_description(contract, probe_name),
                        timeout
                    ),
                    next: timed_out_probe_next(probe_name, probe),
                }),
            }
        }
    }

    if let Some(selected_surface_names) = selected_surfaces {
        for surface_name in selected_surface_names {
            let rerun_command = rerun_selected_workflow_doctor_command(workflow_name);
            match run_workflow_surface_readiness(
                contract,
                contract_path,
                workflow_name,
                surface_name,
            ) {
                Ok(observation) if observation.status == CheckStatus::Passed => continue,
                Ok(observation) if observation.status == CheckStatus::Failed => findings.push(Finding {
                    severity: FindingSeverity::Error,
                    summary: format!("Surface readiness failed: {surface_name}"),
                    why: format!(
                        "the selected workflow surface `{surface_name}` on run task `{}` (backend `{}`; endpoint `{}:{}`) did not become ready after {} checks",
                        observation.run_task_name,
                        observation.backend_label,
                        observation.address,
                        observation.port,
                        observation.attempts
                    ),
                    next: format!(
                        "if workflow run task `{}` is still booting, wait and rerun `{}`; otherwise start or repair `{}` and rerun `{}`",
                        observation.run_task_name,
                        rerun_command,
                        observation.run_task_name,
                        rerun_command
                    ),
                }),
                Ok(observation) => findings.push(Finding {
                    severity: FindingSeverity::Error,
                    summary: format!("Surface readiness timed out: {surface_name}"),
                    why: format!(
                        "the selected workflow surface `{surface_name}` on run task `{}` (backend `{}`; endpoint `{}:{}`) did not become ready within {}ms across {} checks",
                        observation.run_task_name,
                        observation.backend_label,
                        observation.address,
                        observation.port,
                        observation.timeout_ms,
                        observation.attempts
                    ),
                    next: format!(
                        "if workflow run task `{}` is still booting, wait and rerun `{}`; otherwise start or repair `{}` and rerun `{}`",
                        observation.run_task_name,
                        rerun_command,
                        observation.run_task_name,
                        rerun_command
                    ),
                }),
                Err(error) => findings.push(Finding {
                    severity: FindingSeverity::Error,
                    summary: format!("Surface readiness could not be evaluated: {surface_name}"),
                    why: format!(
                        "the selected workflow surface `{surface_name}` on run task `{}` could not be resolved or checked: {error}",
                        contract
                            .selected_workflow(workflow_name)
                            .and_then(|(_, workflow)| workflow.run.as_ref())
                            .map(|run| run.task.as_str())
                            .unwrap_or("-")
                    ),
                    next: format!(
                        "repair workflow run task `{}` surface attachment/readiness and rerun `{}`",
                        contract
                            .selected_workflow(workflow_name)
                            .and_then(|(_, workflow)| workflow.run.as_ref())
                            .map(|run| run.task.as_str())
                            .unwrap_or("-"),
                        rerun_command
                    ),
                }),
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
        if !task.requirements.is_empty() {
            scoped = true;
        }
        selected.extend(task.requirements.checks.iter().cloned());
    }

    (scoped || !selected.is_empty()).then_some(selected)
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

fn selected_workflow_service_names<'a>(
    contract: &'a Contract,
    workflow_name: Option<&str>,
) -> Option<BTreeSet<&'a str>> {
    let (_, workflow) = contract.selected_workflow(workflow_name)?;
    if workflow.services.required.is_empty() {
        return None;
    }

    Some(
        workflow
            .services
            .required
            .iter()
            .map(|service| service.as_str())
            .collect(),
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
) -> CheckStatus {
    if check.kind == crate::schema::CheckKind::File {
        return run_file_check(check, working_dir);
    }
    if let Some(command) = command_override.or(check.run.as_deref()) {
        return run_check(command, working_dir, check.timeout);
    }
    if let Some(probe_name) = check.probe.as_deref()
        && contract.probe(probe_name).is_some()
    {
        return run_named_probe(contract, contract_path, probe_name, check.timeout);
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
    let target = working_dir.join(path);
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

fn run_named_probe(
    contract: &Contract,
    contract_path: &Path,
    probe_name: &str,
    timeout_override_ms: Option<u64>,
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
        crate::runner::ExecutionOverrides::default(),
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
        crate::runner::ExecutionOverrides::default(),
        Some(contract_path),
    ) {
        Ok(backend) => backend,
        Err(_) => return CheckStatus::Failed,
    };
    let caller_backend = crate::runner::effective_task_execution(
        contract,
        observer_task_name,
        crate::runner::ExecutionOverrides::default(),
    )
    .backend;
    let resolved = match resolve_observer_task_probe_command(
        contract,
        contract_path,
        probe_name,
        probe,
        observer_task_name,
        caller_backend,
        timeout_override_ms,
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
        crate::runner::ExecutionOverrides::default(),
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
            let endpoint = crate::schema::ServiceEndpointSpec { address, port };
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
            let endpoint = crate::schema::ServiceEndpointSpec { address, port };
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
    let mut timed_out = false;
    for socket in addrs {
        match TcpStream::connect_timeout(&socket, connect_timeout) {
            Ok(stream) => {
                let _ = stream.shutdown(std::net::Shutdown::Both);
                return HttpReadinessStatus::Passed;
            }
            Err(error) if error.kind() == std::io::ErrorKind::TimedOut => timed_out = true,
            Err(_) => {}
        }
    }
    if timed_out {
        HttpReadinessStatus::TimedOut
    } else {
        HttpReadinessStatus::Failed
    }
}

fn failed_check_summary(check: &crate::schema::CheckSpec) -> String {
    if check.kind == crate::schema::CheckKind::File {
        format!("File check failed: {}", check.name)
    } else if check.probe.is_some() {
        format!("Probe check failed: {}", check.name)
    } else {
        format!("Check failed: {}", check.name)
    }
}

fn failed_check_why(contract: &Contract, check: &crate::schema::CheckSpec) -> String {
    if check.kind == crate::schema::CheckKind::File {
        let path = check.path.as_deref().unwrap_or("-");
        let expected = file_check_expectation_label(check.expect);
        format!("expected `{path}` to be {expected}, but the file check did not pass")
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
    }
    if !details.stdout.is_empty() {
        parts.push(format!("stdout: {}", details.stdout));
    }
    if !details.stderr.is_empty() {
        parts.push(format!("stderr: {}", details.stderr));
    }
    (!parts.is_empty()).then(|| parts.join("; "))
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
    let command = version_command_string(name);
    let Some(resolved_path) = resolve_command_path(name) else {
        return CommandVersionProbe {
            command,
            resolved_path: None,
            probe_started: false,
            outcome: CommandVersionProbeOutcome::Missing,
        };
    };

    let outcome = match version_command_at_path(&resolved_path, name).output() {
        Ok(output) if output.status.success() => {
            let combined = format!(
                "{} {}",
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr)
            );
            extract_version_token(&combined)
                .map(CommandVersionProbeOutcome::Version)
                .unwrap_or(CommandVersionProbeOutcome::Unparseable)
        }
        Ok(output) => CommandVersionProbeOutcome::ProbeFailed {
            exit_code: output.status.code(),
            error: None,
        },
        Err(error) => CommandVersionProbeOutcome::ProbeFailed {
            exit_code: None,
            error: Some(error.to_string()),
        },
    };

    CommandVersionProbe {
        command,
        resolved_path: Some(resolved_path),
        probe_started: true,
        outcome,
    }
}

fn version_command_at_path(path: &Path, name: &str) -> Command {
    let mut command = Command::new(path);
    if name == "go" {
        command.arg("version");
    } else {
        command.arg("--version");
    }
    command
}

pub(crate) fn resolve_command_path(name: &str) -> Option<PathBuf> {
    if looks_like_command_path(name) {
        return command_path_candidates(Path::new(name))
            .into_iter()
            .find(|candidate| is_probeable_file(candidate));
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
    contents
        .lines()
        .any(|line| matches!(line.trim(), ".ota/state/" | ".ota/state" | ".ota/state/*"))
}

fn gitignore_has_ota_receipts_entry(contents: &str) -> bool {
    contents.lines().any(|line| {
        matches!(
            line.trim(),
            ".ota/receipts/" | ".ota/receipts" | ".ota/receipts/*"
        )
    })
}

fn gitignore_has_ota_proof_entry(contents: &str) -> bool {
    contents
        .lines()
        .any(|line| matches!(line.trim(), ".ota/proof/" | ".ota/proof" | ".ota/proof/*"))
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
        Ok(true) => Some(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Repo local Ota artifacts are not ignored by git"),
            why: String::from(
                "`.ota/state/`, `.ota/receipts/`, and `.ota/proof/` store Ota-owned local runtime artifacts; if they are tracked by git, execution residue, archived receipts, and runtime proof artifacts can pollute repo diffs and diagnosis artifacts",
            ),
            next: String::from(
                "run `ota doctor --fix --dry-run` to preview adding `.ota/state/`, `.ota/receipts/`, and `.ota/proof/` to `.gitignore`, or add the ignore rules manually",
            ),
        }),
        Ok(false) => None,
        Err(error) => Some(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Repo `.gitignore` could not be inspected"),
            why: format!(
                "ota could not inspect whether `.ota/state/`, `.ota/receipts/`, and `.ota/proof/` are ignored: {error}"
            ),
            next: String::from("repair `.gitignore` readability and rerun `ota doctor`"),
        }),
    }
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

    actual == requirement || actual.starts_with(&format!("{requirement}."))
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
    use std::env;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use std::thread;
    use std::time::Duration;

    use crate::parser::parse_contract_str;
    use crate::policy_pack::ProvisioningTargetKind;
    use crate::runner::HttpReadinessRequest;
    use crate::schema::ServiceSpec;
    #[cfg(windows)]
    use crate::test_support::cwd_mutex_lock;
    use crate::test_support::env_mutex_lock;
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
        assert_eq!(report.findings[1].severity, FindingSeverity::Warn);
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Ephemeral lifecycle is advisory in native mode"
        );
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
    run: npm install
  build:
    context: verify
    run: npm run build
    depends_on:
      - setup
"#,
        )
        .unwrap();

        let mut findings = Vec::new();
        super::diagnose_contract_advisories(
            &contract,
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
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
    run: npm install
  build:
    context: verify
    run: npm run build
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

        assert!(report.ok);
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
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
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
            &mut findings,
            crate::runner::ExecutionOverrides::default(),
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
    fn preconditions_in_container_mode_skip_host_bound_env_checks() {
        let _guard = env_mutex_lock();
        let tempdir = TempDir::new().unwrap();
        let bin_dir = tempdir.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let original_path = env::var("PATH").unwrap_or_default();
        let new_path = format!("{}:{}", bin_dir.display(), original_path);
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
    fn reports_container_tool_probe_failures_with_resolved_probe_path() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            format!(
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"command -v 'npm'\" >nul && (\r\n    echo {}/usr/local/bin/npm 1>&2\r\n    exit /b 1\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                super::CONTAINER_PROBE_PATH_MARKER
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"command -v 'npm'\"*) echo '{}/usr/local/bin/npm' >&2; exit 1 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
                super::CONTAINER_PROBE_PATH_MARKER
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
        assert!(finding
            .why
            .contains("ota probed `/usr/local/bin/npm` inside container image `premium/test:latest` with `npm --version`"));
        assert_eq!(finding.evidence().command, "npm --version");
        assert_eq!(finding.evidence().path, "/usr/local/bin/npm");
        assert_eq!(finding.evidence().source, "container_target");
        assert_eq!(finding.owner(), "container_target");
    }

    #[test]
    fn reports_container_unparseable_tool_versions_with_resolved_probe_path() {
        let _guard = env_mutex_lock();
        let temp = TempDir::new().unwrap();
        let bin_dir = temp.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        let docker_body = if cfg!(windows) {
            format!(
                "@echo off\r\nif \"%1\"==\"info\" exit /b 0\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"command -v 'npm'\" >nul && (\r\n    echo ready\r\n    echo {}/usr/local/bin/npm 1>&2\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                super::CONTAINER_PROBE_PATH_MARKER
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"info\" ]; then\n  exit 0\nfi\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"command -v 'npm'\"*) echo 'ready'; echo '{}/usr/local/bin/npm' >&2; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
                super::CONTAINER_PROBE_PATH_MARKER
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
            .find(|finding| finding.summary == "Unparseable version for tool: npm")
            .expect("expected unparseable tool version finding");
        assert!(finding
            .why
            .contains("ota probed `/usr/local/bin/npm` inside container image `premium/test:latest` with `npm --version`"));
        assert_eq!(finding.evidence().command, "npm --version");
        assert_eq!(finding.evidence().path, "/usr/local/bin/npm");
        assert_eq!(finding.evidence().source, "container_target");
        assert_eq!(finding.owner(), "container_target");
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
    package_managers:
      pnpmx: "10.22.0"
tasks:
  setup:
    run: pnpm install
    requirements:
      toolchains:
        - node
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
                finding.summary.contains("pnpmx")
                    && finding
                        .next
                        .contains("corepack enable && corepack prepare pnpmx@10.22.0 --activate")
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

        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
extensions:
  remote-fixture:
    kind: backend_provider
    command: fake-remote-provider
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

        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
extensions:
  remote-fixture:
    kind: backend_provider
    command: fake-remote-provider
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
    context: host
    run: cargo test
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
            })
            .expect("expected remote context blocker");
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert!(finding.next.contains("execution.contexts.remote-bad"));
        assert_eq!(finding.owner(), "remote_backend");
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
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
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
        drop(listener);

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
      timeout: 100
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
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].summary,
            "Probe check failed: backend-ready"
        );
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
    fn workflow_readiness_probes_are_executed() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
    fn workflow_readiness_surfaces_retry_until_ready() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        drop(listener);

        let server = thread::spawn(move || {
            thread::sleep(Duration::from_millis(250));
            let listener = TcpListener::bind(("127.0.0.1", port)).expect("listener should bind");
            let (mut stream, _) = listener.accept().expect("probe should connect");
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
            .find(|finding| finding.summary == "Surface readiness timed out: backend")
            .expect("surface timeout finding should be present");
        assert!(finding.why.contains("within 200ms across"), "{report:?}");
        assert!(!finding.why.contains("within 0ms"), "{report:?}");
    }

    #[test]
    fn workflow_surface_timeout_retry_budget_caps_large_timeouts() {
        let configured = 120;
        let capped = super::capped_timed_out_retry_budget(configured, 10_000);
        assert_eq!(capped, 9);
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
        assert_eq!(capped, 18);
    }

    #[test]
    fn workflow_surface_failed_retry_budget_preserves_small_intervals() {
        let configured = 30;
        let capped = super::capped_failed_retry_budget(configured, Duration::from_millis(200));
        assert_eq!(capped, configured);
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
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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

        let status =
            super::run_named_probe(&contract, synthetic_contract_path(), "backend-ready", None);
        server.join().expect("probe server should finish");

        assert!(matches!(status, CheckStatus::TimedOut(100)));
    }

    #[test]
    fn workflow_readiness_can_reference_task_target_probe() {
        let listener = TcpListener::bind("127.0.0.1:0").expect("listener should bind");
        let port = listener.local_addr().unwrap().port();
        let server = thread::spawn(move || {
            let (mut stream, _) = listener.accept().expect("probe should connect");
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Version mismatch for tool: rustc"
        );
    }

    #[test]
    fn reports_required_service_healthcheck_failures_as_errors() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(
            report.findings[0].summary,
            "Service healthcheck failed: postgres"
        );
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(
            report.findings[0].summary,
            "Service readiness failed: postgres"
        );
        assert!(
            report.findings[0]
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
    manager:
      kind: compose
      name: ota
      file: docker-compose.yml
      service: postgres
    endpoints:
      app:
        address: postgres
        port: 5432
    readiness:
      from: app
      run: exit 1
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
            let mut buffer = [0u8; 256];
            let _ = stream.read(&mut buffer);
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
                let mut buffer = [0u8; 256];
                let _ = stream.read(&mut buffer);
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
    manager:
      kind: compose
      name: ota
      file: docker-compose.yml
      service: postgres
    endpoints:
      app:
        address: postgres
        port: 5432
    readiness:
      from: app
      run: exit 1
"#,
        )
        .unwrap();

        let report =
            diagnose_contract_in_mode(&contract, synthetic_contract_path(), DoctorMode::Container);
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
    fn reports_optional_service_healthcheck_failures_as_warnings() {
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
tasks:
  setup:
    run: printf ready
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, synthetic_contract_path());
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Service healthcheck failed: cache"
        );
    }

    #[test]
    fn required_service_with_start_and_endpoint_routes_to_declare_readiness() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Required service cannot be verified: postgres"
        );
        assert_eq!(
            report.findings[0].next,
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Required service cannot be verified: postgres"
        );
        assert_eq!(
            report.findings[0].next,
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].next,
            "declare readiness with `ota assist declare-readiness --service postgres --style tcp` or `--style http`, then rerun `ota doctor`"
        );
    }

    #[test]
    fn required_compose_service_without_endpoint_routes_to_declare_service() {
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(
            report.findings[0].next,
            "refine the managed service with `ota assist declare-service --name postgres --style tcp` or `--style http`, then rerun `ota doctor`"
        );
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(
            report.findings[0].summary,
            "Service healthcheck timed out: postgres"
        );
    }

    #[test]
    fn sorts_errors_before_warnings_before_info() {
        let contract = parse_contract_str(
            synthetic_contract_path(),
            r#"
version: 1
project:
  name: ota
env:
  vars:
    OTA_DOCTOR_SORT_REQUIRED:
      required: true
services:
  cache:
    required: false
    healthcheck: exit 1
checks:
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
        assert_eq!(report.findings.len(), 3);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(report.findings[1].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[2].severity, FindingSeverity::Info);
    }

    #[test]
    fn supports_caret_requirements_for_detected_versions() {
        assert!(version_matches("^3.11", "3.11.0"));
        assert!(version_matches("^3.11", "3.12.4"));
        assert!(!version_matches("^3.11", "4.0.0"));
        assert!(version_matches("^0.6.0", "0.6.4"));
        assert!(!version_matches("^0.6.0", "0.7.0"));
        assert!(version_matches("<=21", "21"));
        assert!(version_matches("<21", "20.9"));
        assert!(version_matches(">21", "21.1"));
        assert!(!version_matches("<=21", "25.0.2"));
        assert!(version_matches(">=go1.2.1", "go1.24.2"));
    }

    #[test]
    fn maps_maven_tool_to_mvn_executable() {
        assert_eq!(tool_executable_name("maven"), "mvn");
        assert_eq!(tool_executable_name("cargo"), "cargo");
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
            .find(|finding| finding.summary == "Missing toolchain component: rust.rustfmt")
            .expect("expected rustfmt component finding");
        assert_eq!(finding.severity, FindingSeverity::Error);
        assert!(finding.next.contains("rustup component add rustfmt"));
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
                name: Some(String::from("local")),
                file: Some(String::from("compose.yaml")),
                service: Some(String::from("postgres")),
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
                name: Some(String::from("local-postgres")),
                file: None,
                service: None,
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
      source: org-mirror
      approved_versions:
        - "22"
    maven:
      source: approved-manager
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
            .find(|finding| finding.summary == "Policy-backed provisioning sources are declared")
            .expect("policy-backed provisioning finding should be present");
        assert_eq!(finding.severity, FindingSeverity::Info);
        assert!(finding.why.contains("java via org-mirror"));
        assert!(finding.why.contains("runtime java 22 via org-mirror"));
        assert!(finding.why.contains("tool maven 3.9 via approved-manager"));
    }

    #[test]
    fn reports_python_managed_toolchain_opportunity_without_provider_advice_in_text() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        fs::write(
            fixture.path().join("ota.yaml"),
            r#"
version: 1
project:
  name: ota
runtimes:
  python: "3.12"
tools:
  uv: "*"
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
        let finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Managed toolchain opportunity: python")
            .expect("toolchain opportunity finding should be present");
        assert_eq!(finding.severity, FindingSeverity::Warn);
        assert!(finding.why.contains("`runtimes.python` and `tools.uv`"));
        assert!(
            finding
                .why
                .contains("repo signals: `uv.lock`, `pyproject.toml`")
        );
        assert!(!finding.next.contains("mise"));
        assert_eq!(finding.provenance().as_deref(), Some("repo signals"));
        assert_eq!(finding.provenance_key().as_deref(), Some("repo_signals"));
    }

    #[test]
    fn doctor_json_includes_toolchain_opportunity_agent_metadata() {
        let finding = Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Managed toolchain opportunity: python"),
            why: String::from("fallback model"),
            next: String::from("keep runtimes.python and tools.uv for now"),
        };

        let json = serde_json::to_value(&finding).expect("finding should serialize");
        assert_eq!(
            json["code"],
            serde_json::Value::String(String::from("OTA_TOOLCHAIN_OPPORTUNITY_UNSUPPORTED"))
        );
        assert_eq!(json["provenance_key"], "repo_signals");
        assert_eq!(json["toolchain_opportunity"]["ecosystem"], "python");
        assert_eq!(json["toolchain_opportunity"]["fallback_runtime"], "python");
        assert_eq!(
            json["toolchain_opportunity"]["candidate_providers"][0],
            "uv"
        );
        assert_eq!(
            json["toolchain_opportunity"]["candidate_providers"][1],
            "mise"
        );
        assert_eq!(json["toolchain_opportunity"]["shipped"], false);
        assert!(
            json["toolchain_opportunity"]["agent_note"]
                .as_str()
                .expect("agent note")
                .contains("toolchains.python")
        );
    }

    #[test]
    fn policy_surfaces_include_toolchain_owned_runtime_requirements() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
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
        assert!(report.ok, "{report:?}");

        let version_finding = report
            .findings
            .iter()
            .find(|finding| finding.summary == "Policy-backed version rules are declared")
            .expect("policy-backed version finding should be present");
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
                .starts_with("update the repo contract versions or widen `")
        );
        assert!(finding.next.ends_with("org-policy.yaml`"));
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
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[0].summary, "Check timed out: slow-check");
        assert!(report.findings[0].why.contains("50ms"));
    }
}
