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
use std::fs;
use std::io::{self, IsTerminal, Write};
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
    container_engine_candidates, container_engine_candidates_from_backend,
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
    DeclaredEnvSourceStatus, LoadedDeclaredEnvSource, ResolvedExecutionBackend, RunError,
    load_declared_env_sources, resolve_context_execution_backend,
    resolve_declared_env_source_value, run_backend_command_captured,
};
use crate::schema::{
    Backend, CheckKind, CheckSeverity, ContainerBackend, Contract, ExtensionKind, Lifecycle,
    RequirementSurface, RuntimeRequirement, ServiceSpec, ToolRequirement,
};
use crate::terminal::supports_dynamic_stderr_ui;
use crate::validator::{ContractAdvisory, collect_contract_advisories};

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

fn rerun_doctor_command(mode: DoctorMode) -> &'static str {
    match mode {
        DoctorMode::Native => "ota doctor",
        DoctorMode::Container => "ota doctor --mode container",
        DoctorMode::Remote => "ota doctor --mode remote",
    }
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

fn precondition_requirement_surface(contract: &Contract, mode: DoctorMode) -> RequirementSurface {
    contract.requirement_surface_for_backend(backend_for_mode(mode))
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
    findings: &mut Vec<Finding>,
) -> Vec<RemoteProbeContext> {
    let mut probes = Vec::new();
    let Some(execution) = contract.execution.as_ref() else {
        return probes;
    };
    let working_dir = contract_working_dir(contract_path);

    for (name, context) in execution
        .contexts
        .iter()
        .filter(|(_, context)| context.backend == Backend::Remote)
    {
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
        let mut requirement_surface = RequirementSurface {
            runtimes: contract.runtimes.clone(),
            tools: contract.tools.clone(),
        };
        requirement_surface
            .runtimes
            .extend(context.requirements.runtimes.clone());
        requirement_surface
            .tools
            .extend(context.requirements.tools.clone());
        let provisioning_actions = loaded_policy
            .map(|loaded| {
                loaded
                    .pack
                    .selected_provisioning_actions_for_requirement_surface_os(
                        &target_os,
                        &requirement_surface,
                    )
            })
            .unwrap_or_default();
        probes.push(RemoteProbeContext {
            context_name: Some(name.clone()),
            backend,
            target_os,
            requirement_surface,
            provisioning_actions,
        });
    }

    if !probes.is_empty() {
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
    let requirement_surface = RequirementSurface {
        runtimes: contract.runtimes.clone(),
        tools: contract.tools.clone(),
    };
    let provisioning_actions = loaded_policy
        .map(|loaded| {
            loaded
                .pack
                .selected_provisioning_actions_for_requirement_surface_os(
                    &target_os,
                    &requirement_surface,
                )
        })
        .unwrap_or_default();

    probes.push(RemoteProbeContext {
        context_name: None,
        backend,
        target_os,
        requirement_surface,
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
    provisioning_actions: Vec<ProvisioningAction>,
}

const CONTAINER_PROBE_PATH_MARKER: &str = "__OTA_RESOLVED_PATH__";
const CONTAINER_PROBE_STARTED_MARKER: &str = "__OTA_CONTAINER_PROBE_STARTED__";

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
            "Repo local runtime state is not ignored by git" => {
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
            s if s.starts_with("Tool probe failed: ") => "OTA_TOOL_PROBE_FAILED",
            s if s.starts_with("Unparseable version for tool: ") => "OTA_TOOL_VERSION_UNPARSEABLE",
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
            s if s.starts_with("Contract drift:") => "OTA_CONTRACT_DRIFT",
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
            | "OTA_TOOL_PROBE_FAILED"
            | "OTA_TOOL_VERSION_UNPARSEABLE" => "environment",
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
            "OTA_CHECK_FAILED" | "OTA_CHECK_TIMED_OUT" => "execution",
            "OTA_CONTRACT_DRIFT" => "contract",
            _ => "contract",
        }
    }

    fn owner(&self) -> &'static str {
        match self.code() {
            "OTA_TASKS_MISSING"
            | "OTA_CONTRACT_DRIFT"
            | "OTA_CHECK_FAILED"
            | "OTA_CHECK_TIMED_OUT" => "repo_contract",
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
            | "OTA_TOOL_PROBE_FAILED"
            | "OTA_TOOL_VERSION_UNPARSEABLE" => {
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
            "OTA_CONTRACT_DRIFT" => (
                "repo signals differ from the declared contract".to_string(),
                "repo signals match the declared contract".to_string(),
                "detect".to_string(),
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

    fn provenance_context(&self) -> Option<FindingProvenanceContext<'_>> {
        if self.policy_context().is_some() {
            return Some(FindingProvenanceContext {
                provenance: "org policy",
                provenance_key: "org_policy",
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
            "No `ota.yaml` found"
                | "Could not inspect repo signals"
                | "Detected Rust repo"
                | "No repo signals detected"
        ) || summary.starts_with("Detected Docker Compose services: ")
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
            | "OTA_CHECK_FAILED"
            | "OTA_CHECK_TIMED_OUT" => Some(FindingProvenanceContext {
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
        let provenance = self.provenance_context();
        let mut state = serializer.serialize_struct(
            "Finding",
            8 + policy.map(|_| 5).unwrap_or_default()
                + drift.map(|_| 2).unwrap_or_default()
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
    )
}

pub fn diagnose_contract_in_mode(
    contract: &Contract,
    contract_path: &Path,
    mode: DoctorMode,
) -> DoctorReport {
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::All, mode)
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
        diagnose_org_policy(
            contract,
            contract_path,
            Some(loaded_policy_ref),
            current_os(),
            &requirement_surface,
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
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::Preconditions, mode)
}

pub fn diagnose_checks_only(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::ChecksOnly,
        DoctorMode::Native,
    )
}

pub fn diagnose_services_only(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(
        contract,
        contract_path,
        DoctorScope::ServicesOnly,
        DoctorMode::Native,
    )
}

pub fn diagnose_service(contract: &Contract, contract_path: &Path, name: &str) -> DoctorReport {
    let mut findings = Vec::new();
    let working_dir = contract_working_dir(contract_path);

    if let Some(service) = contract.services.get(name)
        && let Some(finding) = service_finding(
            contract,
            name,
            service,
            working_dir,
            doctor_mode_for_service(contract, service),
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
) -> DoctorReport {
    let mut findings = Vec::new();
    let mut provisioning = None;
    let mut adapter_bootstrap = None;
    let mut execution_target = None;
    let requirement_surface = precondition_requirement_surface(contract, mode);
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
                loaded
                    .pack
                    .selected_provisioning_actions_for_requirement_surface_os(
                        policy_target_os_for_mode(mode),
                        &requirement_surface,
                    )
            })
            .unwrap_or_default()
    };
    if let Some(finding) = detect_missing_ota_state_gitignore(contract_path) {
        findings.push(finding);
    }

    if matches!(scope, DoctorScope::All | DoctorScope::Preconditions) {
        diagnose_lifecycle(contract, &mut findings);
        let container_probe = diagnose_execution_backend(contract, &mut findings, mode);
        let declared_env_sources = load_declared_env_sources(contract, contract_path);
        diagnose_env_sources(&declared_env_sources, &mut findings);
        if mode == DoctorMode::Native {
            diagnose_env(
                contract,
                loaded_policy
                    .as_ref()
                    .map(|loaded| loaded.pack.env_values()),
                &declared_env_sources,
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
                    &mut findings,
                );
                for remote_probe in &remote_probe_contexts {
                    diagnose_runtimes(
                        &remote_probe.requirement_surface.runtimes,
                        &remote_probe.target_os,
                        contract_path,
                        loaded_policy.as_ref(),
                        mode,
                        None,
                        Some(&remote_probe.backend),
                        remote_probe.context_name.as_deref(),
                        &remote_probe.provisioning_actions,
                        &mut findings,
                    );
                    diagnose_tools(
                        &remote_probe.requirement_surface.tools,
                        &remote_probe.target_os,
                        contract_path,
                        loaded_policy.as_ref(),
                        mode,
                        None,
                        Some(&remote_probe.backend),
                        remote_probe.context_name.as_deref(),
                        &remote_probe.provisioning_actions,
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
            diagnose_runtimes(
                &contract
                    .requirement_surface_for_backend(backend_for_mode(mode))
                    .runtimes,
                policy_target_os_for_mode(mode),
                contract_path,
                loaded_policy.as_ref(),
                mode,
                container_probe.as_ref(),
                None,
                None,
                &provisioning_actions,
                &mut findings,
            ) || diagnose_tools(
                &contract
                    .requirement_surface_for_backend(backend_for_mode(mode))
                    .tools,
                policy_target_os_for_mode(mode),
                contract_path,
                loaded_policy.as_ref(),
                mode,
                container_probe.as_ref(),
                None,
                None,
                &provisioning_actions,
                &mut findings,
            )
        };
        if mode == DoctorMode::Native && contract_has_remote_execution_context(contract) {
            findings.push(remote_mode_scope_note_finding());
        }
        if mode == DoctorMode::Container
            && container_probe_started
            && let Some(container_probe) = container_probe.as_ref()
        {
            execution_target = Some(crate::runner::ephemeral_container_name(
                contract_working_dir(contract_path),
                &container_probe.image,
                &container_probe.engine,
            ));
        }
        if mode != DoctorMode::Remote {
            provisioning = diagnose_org_policy(
                contract,
                contract_path,
                loaded_policy.as_ref(),
                policy_target_os_for_mode(mode),
                &requirement_surface,
                &mut findings,
            );
        }
        adapter_bootstrap = diagnose_adapter_bootstrap(loaded_policy.as_ref(), &mut findings);
    }
    if scope == DoctorScope::All {
        diagnose_tasks_surface(contract, &mut findings);
        diagnose_contract_advisories(contract, &mut findings);
    }
    if matches!(scope, DoctorScope::All | DoctorScope::ServicesOnly) {
        diagnose_services(contract, contract_path, mode, &mut findings);
    }
    if scope != DoctorScope::ServicesOnly {
        if mode == DoctorMode::Native {
            diagnose_checks(contract, contract_path, scope, &mut findings);
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
            "add at least one `tasks.<name>.run` or `tasks.<name>.script` entry, or run `ota detect --dry-run` and `ota detect --write` to regenerate",
        ),
    });
}

fn diagnose_contract_advisories(contract: &Contract, findings: &mut Vec<Finding>) {
    for advisory in collect_contract_advisories(contract) {
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
        });
    }
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

fn diagnose_lifecycle(contract: &Contract, findings: &mut Vec<Finding>) {
    let Some(execution) = contract.execution.as_ref() else {
        return;
    };

    if execution.lifecycle != Some(Lifecycle::Ephemeral) {
        return;
    }

    if execution.preferred == Some(Backend::Container) {
        findings.push(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Ephemeral lifecycle is execution-only"),
            why: String::from(
                "`execution.lifecycle: ephemeral` only applies to task execution. Diagnosis, healthchecks, and teardown are not covered.",
            ),
            next: String::from(
                "use `ota run <task>` for isolated execution; use `ota up` for readiness only",
            ),
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
            first_execution_context_for_backend(contract, Backend::Container)
            && let Some(container) = context.container.as_ref()
        {
            let Some(engine) = selected_container_engine_from_backend(Some(container)) else {
                diagnose_container_backend_cli_for_container(container, findings);
                return None;
            };

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

            return Some(ContainerProbeContext {
                image: container.image.clone(),
                engine,
            });
        }

        findings.push(container_mode_not_configured_finding());
        return None;
    }

    if mode == DoctorMode::Remote {
        let Some(remote) = first_execution_context_for_backend(contract, Backend::Remote)
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

    match execution.preferred {
        Some(Backend::Container) => diagnose_container_backend_cli(contract, findings),
        Some(Backend::Remote) => {
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
        _ => {}
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
        summary: String::from("Host-bound readiness checks are not evaluated in container mode"),
        why: format!(
            "container mode checks the execution image; {skipped} {verb} host-bound and would mix contexts"
        ),
        next: String::from(
            "use `ota doctor --mode native` for host readiness, or `ota up --mode container` for container execution readiness",
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
    findings: &mut Vec<Finding>,
) {
    let working_dir = contract_working_dir(contract_path);

    for (name, service) in &contract.services {
        if let Some(finding) = service_finding(contract, name, service, working_dir, mode) {
            findings.push(finding);
        }
    }
}

fn service_finding(
    contract: &Contract,
    name: &str,
    service: &ServiceSpec,
    working_dir: &Path,
    mode: DoctorMode,
) -> Option<Finding> {
    let rerun_doctor = rerun_doctor_command(mode);
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
                next: service_readiness_execution_next(name, from_context, mode),
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
        let why = if service.start_command(name).is_some() {
            format!(
                "service `{name}` is required but no `healthcheck` is configured, so Ota cannot verify readiness"
            )
        } else {
            format!(
                "service `{name}` is required but no `healthcheck` or `start` command is configured, so Ota cannot verify or prepare it"
            )
        };

        let next = if service.start_command(name).is_some() {
            format!("add `services.{name}.healthcheck` so `ota doctor` can verify readiness")
        } else {
            format!("add `services.{name}.healthcheck` and optionally `services.{name}.start`")
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

fn service_readiness_execution_next(name: &str, context_name: &str, mode: DoctorMode) -> String {
    format!(
        "repair execution context `{context_name}` or move `services.{name}.readiness.from` to a context Ota can execute, then rerun `{}`",
        rerun_doctor_command(mode)
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
    let command = structured_service_readiness_command(readiness, endpoint, kind);
    let timing = service_readiness_timing_policy(readiness);
    if !timing.start_period.is_zero() {
        thread::sleep(timing.start_period);
    }

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
                if timing
                    .retries
                    .is_some_and(|failure_budget| failed_attempts >= failure_budget)
                {
                    return Ok(CheckStatus::Failed);
                }
            }
            Err(error) => return Err(error),
        }
        thread::sleep(timing.interval);
    }
}

#[derive(Debug, Clone, Copy)]
struct ServiceReadinessTimingPolicy {
    start_period: Duration,
    interval: Duration,
    retries: Option<u32>,
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
        retries: readiness.retries,
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
    let url = format!(
        "http://{}:{}{}",
        endpoint.address.trim(),
        endpoint.port,
        normalized_runtime_path(readiness.path.as_deref())
    );
    let method = readiness
        .method
        .unwrap_or(crate::schema::TaskRuntimeReadinessHttpMethod::Get);
    let status_csv = readiness
        .success
        .as_ref()
        .filter(|success| !success.status.is_empty())
        .map(|success| {
            success
                .status
                .iter()
                .map(|status| status.to_string())
                .collect::<Vec<_>>()
                .join(",")
        })
        .unwrap_or_else(|| {
            String::from(
                "200,201,202,203,204,205,206,207,208,226,300,301,302,303,304,305,306,307,308",
            )
        });
    let headers_shell = readiness
        .headers
        .iter()
        .map(|(name, value)| format!("-H {}", doctor_shell_quote(&format!("{name}: {value}"))))
        .collect::<Vec<_>>()
        .join(" ");
    let headers_json =
        serde_json::to_string(&readiness.headers).unwrap_or_else(|_| String::from("{}"));
    let body_contains = readiness
        .body
        .as_ref()
        .map(|body| body.contains.clone())
        .unwrap_or_default();
    let timeout_seconds = readiness
        .timeout
        .as_deref()
        .and_then(crate::schema::parse_readiness_duration_spec)
        .map(|duration| duration.as_secs_f64().max(0.001))
        .unwrap_or(2.0);
    format!(
        "url={url}; method={method}; statuses={statuses}; contains={contains}; headers_json={headers_json}; timeout={timeout}; \
if command -v curl >/dev/null 2>&1; then \
  body_file=$(mktemp 2>/dev/null || printf '/tmp/ota-service-readiness-body-$$'); \
  code=$(curl -sS --connect-timeout \"$timeout\" --max-time \"$timeout\" -X \"$method\" {headers} -o \"$body_file\" -w '%{{http_code}}' \"$url\") || {{ rm -f \"$body_file\"; exit 1; }}; \
  matched=1; OLDIFS=\"$IFS\"; IFS=,; for expected in $statuses; do if [ \"$code\" = \"$expected\" ]; then matched=0; break; fi; done; IFS=\"$OLDIFS\"; \
  [ $matched -eq 0 ] || {{ rm -f \"$body_file\"; exit 1; }}; \
  if [ -n \"$contains\" ]; then grep -Fq -- \"$contains\" \"$body_file\" || {{ rm -f \"$body_file\"; exit 1; }}; fi; \
  rm -f \"$body_file\"; exit 0; \
fi; \
if command -v python3 >/dev/null 2>&1; then \
  python3 - \"$url\" \"$method\" \"$statuses\" \"$contains\" \"$headers_json\" \"$timeout\" <<'PY' && exit 0 || exit 1\n\
import json, sys, urllib.error, urllib.request\n\
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
except Exception:\n\
    sys.exit(1)\n\
if status not in statuses:\n\
    sys.exit(1)\n\
if contains and contains not in body:\n\
    sys.exit(1)\n\
PY\n\
fi; \
exit 1",
        url = doctor_shell_quote(&url),
        method = doctor_shell_quote(method.as_str()),
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
  python3 - \"$host\" \"$port\" \"$timeout\" <<'PY' && exit 0 || exit 1\n\
import socket, sys\n\
host, port_raw, timeout_raw = sys.argv[1:4]\n\
port = int(port_raw)\n\
timeout = float(timeout_raw)\n\
sock = socket.socket(socket.AF_INET, socket.SOCK_STREAM)\n\
sock.settimeout(timeout)\n\
try:\n\
    sock.connect((host, port))\n\
except Exception:\n\
    sys.exit(1)\n\
finally:\n\
    try:\n\
        sock.close()\n\
    except Exception:\n\
        pass\n\
PY\n\
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
    findings: &mut Vec<Finding>,
) {
    for (name, requirement) in &contract.env {
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
                        next: format!("set {name} to one of: {}", requirement.allowed.join(", ")),
                    });
                }
            }
            None if requirement.required => findings.push(Finding {
                severity: FindingSeverity::Error,
                summary: format!("Missing environment variable: {name}"),
                why: format!("{name} is required by this repo contract"),
                next: format!(
                    "set {name} in policy env, the shell, or a declared env source before running tasks"
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
            mode,
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
            mode,
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

fn runtime_provider_hint<'a>(requirement: &'a RuntimeRequirement, os: &str) -> Option<&'a str> {
    requirement.provider_for_os(os)
}

fn diagnose_org_policy(
    contract: &Contract,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    policy_os: &str,
    requirement_surface: &RequirementSurface,
    findings: &mut Vec<Finding>,
) -> Option<ProvisioningDiagnostics> {
    let Some(loaded_policy) = loaded_policy else {
        return None;
    };
    let policy_pack = &loaded_policy.pack;
    let policy_path = &loaded_policy.path;

    let contract_root = contract_working_dir(contract_path);

    let missing_sections = policy_pack.missing_required_sections(contract);
    let missing_files = policy_pack.missing_required_files(contract_root);
    let version_violations = policy_pack
        .version_policy_violations_for_requirement_surface_os(policy_os, requirement_surface);
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
            .provisioning_plan_for_requirement_surface_os(policy_os, requirement_surface);
        let provisioning_request = ProvisioningBackendRequest {
            actions: policy_pack.selected_provisioning_actions_for_requirement_surface_os(
                policy_os,
                requirement_surface,
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
                    requirement_surface,
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
            &remote_probe.requirement_surface,
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
            &remote_probe.requirement_surface,
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
            &remote_probe.requirement_surface,
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

fn exact_tooling_remediation(
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
        (ProvisioningTargetKind::Runtime, "rust", "rustup") => {
            Some(format!("rustup toolchain install {requirement}"))
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

fn tool_versions_entry(contract_root: &Path, candidate_names: &[&str]) -> Option<String> {
    let path = contract_root.join(".tool-versions");
    let contents = std::fs::read_to_string(path).ok()?;

    for line in contents.lines() {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }
        let mut parts = trimmed.split_whitespace();
        let Some(tool_name) = parts.next() else {
            continue;
        };
        if candidate_names
            .iter()
            .any(|candidate| candidate == &tool_name)
        {
            return Some(tool_name.to_string());
        }
    }

    None
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
    mode: DoctorMode,
    container_probe: Option<&ContainerProbeContext>,
    remote_probe: Option<&ResolvedExecutionBackend>,
    remote_context_name: Option<&str>,
    contract_path: &Path,
    loaded_policy: Option<&LoadedOrgPolicyPack>,
    target_os: &str,
    provisioning_actions: &[ProvisioningAction],
    findings: &mut Vec<Finding>,
) -> bool {
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
    let actual = if let Some(probe) = version_probe.as_ref() {
        match &probe.outcome {
            CommandVersionProbeOutcome::Version(actual) => Some(actual.clone()),
            _ => None,
        }
    } else {
        None
    };

    let Some(actual) = actual else {
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
                                "run `{}` inside the selected container image, inspect `{resolved_path}`, and make sure the probe succeeds before rerunning `ota doctor --mode container`",
                                probe.command
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
                                "run `{}` through the selected remote backend, inspect `{resolved_path}`, and make sure the probe succeeds before rerunning `ota doctor --mode remote`",
                                probe.command
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
                                "run `{}` directly, inspect `{resolved_path}`, and make sure the probe succeeds before rerunning `ota doctor`",
                                probe.command
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
                                "run `{}` inside the selected container image, inspect `{resolved_path}`, and make sure the output contains a parseable version before rerunning `ota doctor --mode container`",
                                probe.command
                            );
                            (why, next)
                        }
                        DoctorMode::Remote => {
                            let why = format!(
                                "ota probed `{resolved_path}` through the declared remote backend with `{}`, but the output did not contain a parseable version",
                                probe.command
                            );
                            let next = format!(
                                "run `{}` through the selected remote backend, inspect `{resolved_path}`, and make sure the output contains a parseable version before rerunning `ota doctor --mode remote`",
                                probe.command
                            );
                            (why, next)
                        }
                        DoctorMode::Native => {
                            let why = format!(
                                "ota probed `{resolved_path}` with `{}`, but the output did not contain a parseable version",
                                probe.command
                            );
                            let next = format!(
                                "run `{}` directly, inspect `{resolved_path}`, and make sure the output contains a parseable version before rerunning `ota doctor`",
                                probe.command
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
                    lifecycle: Lifecycle::Ephemeral,
                    container_name: None,
                },
                "ota doctor --mode container",
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
                "ota doctor --mode remote",
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
                    "update `execution.backends.container.image` (currently `{image}`) so `{display_name}` is available, then rerun `ota doctor --mode container`"
                ),
                (DoctorMode::Container, None) => format!(
                    "update `execution.backends.container.image` so `{display_name}` is available, then rerun `ota doctor --mode container`"
                ),
                (DoctorMode::Remote, _) => remote_context_name
                    .map(|context_name| {
                        format!(
                            "make `{display_name}` available in remote context `{context_name}` and rerun `ota doctor --mode remote`"
                        )
                    })
                    .unwrap_or_else(|| {
                        format!(
                            "make `{display_name}` available in the selected remote backend and rerun `ota doctor --mode remote`"
                        )
                    }),
                _ => exact_remediation
                    .map(|command| format!("run `{command}` and rerun `ota doctor`"))
                    .unwrap_or_else(|| {
                        format!(
                            "install {display_name} and make it available on PATH, then rerun `ota doctor`"
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
                lifecycle: Lifecycle::Ephemeral,
                container_name: None,
            },
            "ota doctor --mode container",
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
            "ota doctor --mode remote",
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
                "update `execution.backends.container.image` (currently `{image}`) so `{display_name}` satisfies `{requirement}`, then rerun `ota doctor --mode container`"
            ),
            (DoctorMode::Container, None) => format!(
                "update `execution.backends.container.image` so `{display_name}` satisfies `{requirement}`, then rerun `ota doctor --mode container`"
            ),
            (DoctorMode::Remote, _) => remote_context_name
                .map(|context_name| {
                    format!(
                        "update `{display_name}` in remote context `{context_name}` so it satisfies `{requirement}`, then rerun `ota doctor --mode remote`"
                    )
                })
                .unwrap_or_else(|| {
                    format!(
                        "update `{display_name}` in the selected remote backend so it satisfies `{requirement}`, then rerun `ota doctor --mode remote`"
                    )
                }),
            _ => exact_remediation
                .map(|command| format!("run `{command}` and rerun `ota doctor`"))
                .unwrap_or_else(|| {
                    format!(
                        "install a compatible {display_name} version that satisfies `{requirement}`, then rerun `ota doctor`"
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
    let mut command = Command::new(engine);
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
            } else if resolved_path.is_none() {
                CommandVersionProbeOutcome::Missing
            } else {
                CommandVersionProbeOutcome::ProbeFailed {
                    exit_code: output.status.code(),
                    error: None,
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
            } else if resolved_path.is_none() {
                CommandVersionProbeOutcome::Missing
            } else {
                CommandVersionProbeOutcome::ProbeFailed {
                    exit_code: Some(output.exit_code),
                    error: None,
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
    if engines.iter().any(|engine| command_available(engine)) {
        return;
    }

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
    findings: &mut Vec<Finding>,
) {
    let working_dir = contract_working_dir(contract_path);

    for check in &contract.checks {
        if scope == DoctorScope::Preconditions && check.kind != CheckKind::Precondition {
            continue;
        }

        match run_check(&check.run, working_dir, check.timeout) {
            CheckStatus::Passed => continue,
            CheckStatus::Failed => findings.push(Finding {
                severity: map_check_severity(check.severity),
                summary: format!("Check failed: {}", check.name),
                why: format!("the configured `{}` check did not succeed", check.name),
                next: failed_check_next(contract, check),
            }),
            CheckStatus::TimedOut(timeout) => findings.push(Finding {
                severity: map_check_severity(check.severity),
                summary: format!("Check timed out: {}", check.name),
                why: format!(
                    "the configured `{}` check did not finish within {}ms",
                    check.name, timeout
                ),
                next: format!(
                    "make `{}` complete faster or raise `checks.timeout` for `{}`, then rerun `ota doctor`",
                    check.run,
                    check.name
                ),
            }),
        }
    }
}

fn failed_check_next(contract: &Contract, check: &crate::schema::CheckSpec) -> String {
    if let Some(path) = missing_file_check_path(&check.run) {
        if contract.tasks.contains_key("setup") {
            return format!(
                "run `ota up` or `ota run setup` to create `{path}`, then rerun `ota doctor`"
            );
        }
        return format!("create `{path}`, then rerun `ota doctor`");
    }

    format!(
        "run `{}` and fix the reported issue, then rerun `ota doctor`",
        check.run
    )
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

enum CheckStatus {
    Passed,
    Failed,
    TimedOut(u64),
}

fn run_check(command: &str, working_dir: &Path, timeout_ms: Option<u64>) -> CheckStatus {
    let Ok(mut child) = shell_command(command)
        .current_dir(working_dir)
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .spawn()
    else {
        return CheckStatus::Failed;
    };

    let spinner = CheckSpinner::start();
    let status = match timeout_ms {
        Some(timeout_ms) => wait_for_child_with_timeout(&mut child, timeout_ms),
        None => wait_for_child(&mut child),
    };
    spinner.stop();

    match status {
        CheckStatus::Passed | CheckStatus::Failed | CheckStatus::TimedOut(_) => status,
    }
}

fn wait_for_child(child: &mut std::process::Child) -> CheckStatus {
    match child.wait() {
        Ok(status) => {
            if status.success() {
                CheckStatus::Passed
            } else {
                CheckStatus::Failed
            }
        }
        Err(_) => CheckStatus::Failed,
    }
}

fn wait_for_child_with_timeout(child: &mut std::process::Child, timeout_ms: u64) -> CheckStatus {
    let deadline = Instant::now() + Duration::from_millis(timeout_ms);
    loop {
        match child.try_wait() {
            Ok(Some(status)) => {
                return if status.success() {
                    CheckStatus::Passed
                } else {
                    CheckStatus::Failed
                };
            }
            Ok(None) if Instant::now() >= deadline => {
                let _ = child.kill();
                let _ = child.wait();
                return CheckStatus::TimedOut(timeout_ms);
            }
            Ok(None) => thread::sleep(Duration::from_millis(10)),
            Err(_) => {
                let _ = child.kill();
                let _ = child.wait();
                return CheckStatus::Failed;
            }
        }
    }
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

fn resolve_command_path(name: &str) -> Option<PathBuf> {
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

        let mut candidates = vec![path.to_path_buf()];
        let pathext = std::env::var_os("PATHEXT")
            .map(|value| value.to_string_lossy().into_owned())
            .unwrap_or_else(|| String::from(".COM;.EXE;.BAT;.CMD"));
        for ext in pathext
            .split(';')
            .map(str::trim)
            .filter(|value| !value.is_empty())
        {
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

pub(crate) const OTA_STATE_GITIGNORE_COMMENT: &str = "# Ota local runtime state";
pub(crate) const OTA_STATE_GITIGNORE_ENTRY: &str = ".ota/state/";

fn gitignore_has_ota_state_entry(contents: &str) -> bool {
    contents
        .lines()
        .any(|line| matches!(line.trim(), ".ota/state/" | ".ota/state" | ".ota/state/*"))
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
    Ok(!gitignore_has_ota_state_entry(&contents))
}

pub(crate) fn detect_missing_ota_state_gitignore(contract_path: &Path) -> Option<Finding> {
    let root = contract_working_dir(contract_path);
    match repo_missing_ota_state_gitignore(root) {
        Ok(true) => Some(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Repo local runtime state is not ignored by git"),
            why: String::from(
                "`.ota/state/` stores Ota-owned local runtime state; if it is tracked by git, local execution residue can pollute repo diffs and diagnosis artifacts",
            ),
            next: String::from(
                "run `ota doctor --fix --dry-run` to preview adding `.ota/state/` to `.gitignore`, or add the ignore rule manually",
            ),
        }),
        Ok(false) => None,
        Err(error) => Some(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Repo `.gitignore` could not be inspected"),
            why: format!("ota could not inspect whether `.ota/state/` is ignored: {error}"),
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
    let mut shell = Command::new("cmd");
    shell.arg("/C").arg(command);
    shell
}

fn shell_single_quote(command: &str) -> String {
    let escaped = command.replace('\'', r"'\''");
    format!("'{}'", escaped)
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::fs;
    use std::io::{Read, Write};
    use std::net::TcpListener;
    use std::path::{Path, PathBuf};
    use std::sync::OnceLock;
    use std::thread;

    use crate::parser::parse_contract_str;
    use crate::schema::ServiceSpec;
    #[cfg(windows)]
    use crate::test_support::cwd_mutex_lock;
    use crate::test_support::env_mutex_lock;
    use tempfile::TempDir;

    use super::{
        DoctorMode, FindingSeverity, compose_service_healthcheck_command, diagnose_checks_only,
        diagnose_contract, diagnose_contract_in_mode, diagnose_preconditions,
        diagnose_preconditions_with_mode, tool_executable_name, version_matches,
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

    fn write_fake_command(bin_dir: &Path, name: &str, body: &str) -> std::path::PathBuf {
        let path = if cfg!(windows) {
            bin_dir.join(format!("{name}.cmd"))
        } else {
            bin_dir.join(name)
        };

        fs::write(&path, body).unwrap();

        #[cfg(unix)]
        {
            let mut permissions = fs::metadata(&path).unwrap().permissions();
            permissions.set_mode(0o755);
            fs::set_permissions(&path, permissions).unwrap();
        }

        path
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
        super::diagnose_contract_advisories(&contract, &mut findings);

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
        super::diagnose_contract_advisories(&contract, &mut findings);

        assert!(findings.iter().any(|finding| {
            finding.severity == FindingSeverity::Warn
                && finding.summary == "Attachment `.m2` may be unused in context `app`"
                && finding.why.contains("point Maven at `/workspace/.m2`")
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

        assert_eq!(resolved.as_deref(), Some(local_npm.as_path()));
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
        assert!(finding.why.contains(&format!(
            "ota probed `{}` with `npm --version`",
            npm_path.display()
        )));
        assert_eq!(finding.evidence().command, "npm --version");
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
        assert!(finding.why.contains(&format!(
            "ota probed `{}` with `npm --version`",
            npm_path.display()
        )));
        assert_eq!(finding.evidence().command, "npm --version");
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
                "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"command -v 'npm'\" >nul && (\r\n    echo {}/usr/local/bin/npm 1>&2\r\n    exit /b 1\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                super::CONTAINER_PROBE_PATH_MARKER
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"command -v 'npm'\"*) echo '{}/usr/local/bin/npm' >&2; exit 1 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
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
                "@echo off\r\nif \"%1\"==\"run\" (\r\n  echo %* | findstr /C:\"command -v 'npm'\" >nul && (\r\n    echo ready\r\n    echo {}/usr/local/bin/npm 1>&2\r\n    exit /b 0\r\n  )\r\n)\r\necho unsupported 1>&2\r\nexit /b 1\r\n",
                super::CONTAINER_PROBE_PATH_MARKER
            )
        } else {
            format!(
                "#!/bin/sh\nif [ \"$1\" = \"run\" ]; then\n  case \"$*\" in\n    *\"command -v 'npm'\"*) echo 'ready'; echo '{}/usr/local/bin/npm' >&2; exit 0 ;;\n  esac\nfi\necho unsupported >&2\nexit 1\n",
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

        let surface = super::precondition_requirement_surface(&contract, DoctorMode::Container);
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

    #[test]
    fn remote_doctor_mode_emits_policy_surfaces_per_remote_context() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "uname", "#!/bin/sh\necho 'Linux'\n");
        write_fake_command(&bin_dir, "jq", "#!/bin/sh\necho 'jq-1.8.1'\n");
        write_fake_command(
            &bin_dir,
            "ssh",
            r#"#!/bin/sh
target="$1"
shift
[ -n "$target" ] || exit 1
[ "$#" -ge 1 ] || exit 1
exec /bin/sh -lc "$*"
"#,
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
            .expect("expected remote policy version finding");
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
        write_fake_command(
            &bin_dir,
            "ssh",
            r#"#!/bin/sh
target="$1"
shift
[ -n "$target" ] || exit 1
[ "$#" -ge 1 ] || exit 1
exec /bin/sh -lc "$*"
"#,
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
        write_fake_command(
            &bin_dir,
            "ssh",
            r#"#!/bin/sh
target="$1"
shift
[ -n "$target" ] || exit 1
[ "$#" -ge 1 ] || exit 1
exec /bin/sh -lc "$*"
"#,
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

    #[test]
    fn remote_doctor_mode_reports_version_policy_violations_per_context() {
        let _guard = env_mutex_lock();
        let fixture = TempDir::new().unwrap();
        let bin_dir = fixture.path().join("bin");
        fs::create_dir_all(&bin_dir).unwrap();
        write_fake_command(&bin_dir, "uname", "#!/bin/sh\necho 'Linux'\n");
        write_fake_command(&bin_dir, "jq", "#!/bin/sh\necho 'jq-1.8.1'\n");
        write_fake_command(
            &bin_dir,
            "ssh",
            r#"#!/bin/sh
target="$1"
shift
[ -n "$target" ] || exit 1
[ "$#" -ge 1 ] || exit 1
exec /bin/sh -lc "$*"
"#,
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
            .expect("expected remote version policy blocker");
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
                .contains("rerun `ota doctor --mode container`"),
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
    fn diagnose_service_structured_http_readiness_keeps_waiting_when_retries_omitted() {
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
            finding.next.contains("rerun `ota doctor --mode container`"),
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
    fn warns_when_required_service_has_no_healthcheck() {
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
        assert!(!report.ok);
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
        assert!(!report.ok);
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
