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

use std::io::{self, IsTerminal, Write};
use std::path::Path;
use std::process::Command;
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use std::thread;
use std::time::{Duration, Instant};

use serde::Serialize;
use serde::ser::{SerializeStruct, Serializer};

use crate::execution::container_engine_candidates;
use crate::policy_pack::{
    LoadPolicyPackError, ProvisioningBackendRequest, ProvisioningPlan, load_org_policy_pack_auto,
};
use crate::schema::{
    Backend, CheckKind, CheckSeverity, Contract, ExtensionKind, Lifecycle, ServiceSpec,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum FindingSeverity {
    Error,
    Warn,
    Info,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Finding {
    pub severity: FindingSeverity,
    pub summary: String,
    pub why: String,
    pub next: String,
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
    ownership: &'a str,
    provenance: &'a str,
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
            "Ephemeral lifecycle is only enforced for backend-backed task execution" => {
                "OTA_LIFECYCLE_EPHEMERAL_BACKEND_ONLY"
            }
            "Ephemeral lifecycle is advisory only in V1" => "OTA_LIFECYCLE_EPHEMERAL_ADVISORY",
            s if s.starts_with("Missing execution backend CLI: ") => "OTA_BACKEND_CLI_MISSING",
            s if s.starts_with("Missing container execution backend CLI: ") => {
                "OTA_CONTAINER_BACKEND_CLI_MISSING"
            }
            s if s.starts_with("Unsupported remote execution backend provider: ") => {
                "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED"
            }
            s if s.starts_with("Suspicious remote target for ") => "OTA_REMOTE_TARGET_SUSPICIOUS",
            s if s.starts_with("Service healthcheck failed: ") => "OTA_SERVICE_CHECK_FAILED",
            s if s.starts_with("Service healthcheck timed out: ") => "OTA_SERVICE_CHECK_TIMED_OUT",
            s if s.starts_with("Required service cannot be verified: ") => {
                "OTA_SERVICE_UNVERIFIABLE"
            }
            s if s.starts_with("Missing environment variable: ") => "OTA_ENV_MISSING",
            s if s.starts_with("Invalid environment value: ") => "OTA_ENV_INVALID",
            s if s.starts_with("Version mismatch for runtime: ") => "OTA_RUNTIME_VERSION_MISMATCH",
            s if s.starts_with("Missing runtime: ") => "OTA_RUNTIME_MISSING",
            s if s.starts_with("Version mismatch for tool: ") => "OTA_TOOL_VERSION_MISMATCH",
            s if s.starts_with("Missing tool: ") => "OTA_TOOL_MISSING",
            "Repo does not satisfy org policy pack" => "OTA_POLICY_PACK_VIOLATION",
            "Invalid org policy pack" => "OTA_POLICY_PACK_INVALID",
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
            "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED" | "OTA_REMOTE_TARGET_SUSPICIOUS" => "remote",
            "OTA_SERVICE_CHECK_FAILED"
            | "OTA_SERVICE_CHECK_TIMED_OUT"
            | "OTA_SERVICE_UNVERIFIABLE" => "service",
            "OTA_ENV_MISSING"
            | "OTA_ENV_INVALID"
            | "OTA_RUNTIME_VERSION_MISMATCH"
            | "OTA_RUNTIME_MISSING"
            | "OTA_TOOL_VERSION_MISMATCH"
            | "OTA_TOOL_MISSING" => "environment",
            "OTA_POLICY_PACK_VIOLATION"
            | "OTA_POLICY_PACK_INVALID"
            | "OTA_POLICY_BACKED_PROVISIONING_DECLARED" => "policy",
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
            | "OTA_TOOL_VERSION_MISMATCH"
            | "OTA_TOOL_MISSING" => "host",
            "OTA_REMOTE_BACKEND_PROVIDER_UNSUPPORTED" | "OTA_REMOTE_TARGET_SUSPICIOUS" => {
                "remote_backend"
            }
            "OTA_SERVICE_CHECK_FAILED"
            | "OTA_SERVICE_CHECK_TIMED_OUT"
            | "OTA_SERVICE_UNVERIFIABLE" => "service",
            "OTA_POLICY_PACK_VIOLATION"
            | "OTA_POLICY_PACK_INVALID"
            | "OTA_POLICY_BACKED_PROVISIONING_DECLARED" => "org_policy",
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
                "host".to_string(),
                String::new(),
                String::new(),
            ),
            "OTA_RUNTIME_MISSING" | "OTA_TOOL_MISSING" => (
                "the required runtime or tool was not available".to_string(),
                "the required runtime or tool is available on PATH".to_string(),
                "host".to_string(),
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
                ownership: "repo_contract",
                provenance: "repo signals were compared against the declared contract with `ota detect`",
            })
        } else {
            None
        }
    }

    pub(crate) fn provenance(&self) -> Option<String> {
        if self.policy_context().is_some() {
            return Some(String::from("org policy"));
        }

        self.drift_context()
            .map(|context| context.provenance.to_string())
    }
}

impl Serialize for Finding {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let policy = self.policy_context();
        let drift = self.drift_context();
        let mut state = serializer.serialize_struct(
            "Finding",
            8 + policy.map(|_| 5).unwrap_or_default() + drift.map(|_| 2).unwrap_or_default(),
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
            state.serialize_field("ownership", drift.ownership)?;
            state.serialize_field("provenance", drift.provenance)?;
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
    pub findings: Vec<Finding>,
}

pub fn diagnose_contract(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::All)
}

pub fn diagnose_preconditions(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::Preconditions)
}

pub fn diagnose_checks_only(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::ChecksOnly)
}

pub fn diagnose_services_only(contract: &Contract, contract_path: &Path) -> DoctorReport {
    diagnose_contract_with_scope(contract, contract_path, DoctorScope::ServicesOnly)
}

pub fn diagnose_service(contract: &Contract, contract_path: &Path, name: &str) -> DoctorReport {
    let mut findings = Vec::new();
    let working_dir = contract_working_dir(contract_path);

    if let Some(service) = contract.services.get(name)
        && let Some(finding) = service_finding(name, service, working_dir)
    {
        findings.push(finding);
    }

    DoctorReport {
        ok: !findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error),
        provisioning: None,
        adapter_bootstrap: None,
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
) -> DoctorReport {
    let mut findings = Vec::new();
    let mut provisioning = None;
    let mut adapter_bootstrap = None;

    if matches!(scope, DoctorScope::All | DoctorScope::Preconditions) {
        diagnose_lifecycle(contract, &mut findings);
        diagnose_execution_backend(contract, &mut findings);
        diagnose_env(contract, &mut findings);
        diagnose_runtimes(contract, &mut findings);
        diagnose_tools(contract, &mut findings);
        provisioning = diagnose_org_policy(contract, contract_path, &mut findings);
        adapter_bootstrap = diagnose_adapter_bootstrap(contract, contract_path, &mut findings);
    }
    if scope == DoctorScope::All {
        diagnose_tasks_surface(contract, &mut findings);
    }
    if matches!(scope, DoctorScope::All | DoctorScope::ServicesOnly) {
        diagnose_services(contract, contract_path, &mut findings);
    }
    if scope != DoctorScope::ServicesOnly {
        diagnose_checks(contract, contract_path, scope, &mut findings);
    }

    findings.sort_by_key(|finding| finding.severity);

    DoctorReport {
        ok: !findings
            .iter()
            .any(|finding| finding.severity == FindingSeverity::Error),
        provisioning,
        adapter_bootstrap,
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
            summary: String::from(
                "Ephemeral lifecycle is only enforced for backend-backed task execution",
            ),
            why: String::from(
                "the contract requests `execution.lifecycle: ephemeral`; 🦦 now uses fresh container execution for `ota run` and the `setup` task inside `ota up`, but service commands, healthchecks, diagnosis, and full repo teardown still do not run in isolated temporary environments",
            ),
            next: String::from(
                "use `ota run` or the `setup` phase of `ota up` for isolated task execution; do not rely on `ota up` for full ephemeral cleanup yet",
            ),
        });
    } else {
        findings.push(Finding {
            severity: FindingSeverity::Warn,
            summary: String::from("Ephemeral lifecycle is advisory only in V1"),
            why: String::from(
                "the contract requests `execution.lifecycle: ephemeral`, but current Ota execution remains shell-native and does not create isolated temporary environments",
            ),
            next: String::from(
                "treat `ephemeral` as a portability hint for now; do not rely on isolation or automatic cleanup in V1",
            ),
        });
    }
}

fn diagnose_execution_backend(contract: &Contract, findings: &mut Vec<Finding>) {
    let Some(execution) = contract.execution.as_ref() else {
        return;
    };

    match execution.preferred {
        Some(Backend::Container) => diagnose_container_backend_cli(contract, findings),
        Some(Backend::Remote) => {
            let Some(remote) = execution
                .backends
                .as_ref()
                .and_then(|backends| backends.remote.as_ref())
            else {
                return;
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
                        return;
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
                        return;
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
                        return;
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

fn diagnose_services(contract: &Contract, contract_path: &Path, findings: &mut Vec<Finding>) {
    let working_dir = contract_working_dir(contract_path);

    for (name, service) in &contract.services {
        if let Some(finding) = service_finding(name, service, working_dir) {
            findings.push(finding);
        }
    }
}

fn service_finding(name: &str, service: &ServiceSpec, working_dir: &Path) -> Option<Finding> {
    if let Some(healthcheck) = service.healthcheck.as_deref() {
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
                next: match service.start.as_deref() {
                    Some(start) => format!("run `{start}` and re-run `ota doctor`"),
                    None => format!(
                        "start or repair `{name}` and re-run its healthcheck: {healthcheck}, then rerun `ota doctor`"
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
                    "make `services.{name}.healthcheck` complete faster or raise `services.{name}.timeout`, then rerun `ota doctor`"
                ),
            }),
        };
    }

    if service.required {
        let why = if service.start.is_some() {
            format!(
                "service `{name}` is required but no `healthcheck` is configured, so Ota cannot verify readiness"
            )
        } else {
            format!(
                "service `{name}` is required but no `healthcheck` or `start` command is configured, so Ota cannot verify or prepare it"
            )
        };

        let next = if service.start.is_some() {
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

fn run_service_healthcheck(
    name: &str,
    service: &ServiceSpec,
    working_dir: &Path,
    healthcheck: &str,
) -> CheckStatus {
    match service.provider.as_deref() {
        Some("docker-compose") => {
            let command = compose_service_healthcheck_command(name, healthcheck);
            run_check(&command, working_dir, service.timeout)
        }
        _ => run_check(healthcheck, working_dir, service.timeout),
    }
}

fn compose_service_healthcheck_command(name: &str, healthcheck: &str) -> String {
    format!(
        "docker compose exec -T {name} sh -lc {}",
        shell_single_quote(healthcheck)
    )
}

fn diagnose_env(contract: &Contract, findings: &mut Vec<Finding>) {
    for (name, requirement) in &contract.env {
        let value = std::env::var(name)
            .ok()
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
                next: format!("set {name} in your environment before running tasks"),
            }),
            None => {}
        }
    }
}

fn diagnose_runtimes(contract: &Contract, findings: &mut Vec<Finding>) {
    for (name, requirement) in &contract.runtimes {
        diagnose_command_version("runtime", name, name, requirement.version(), true, findings);
    }
}

fn diagnose_tools(contract: &Contract, findings: &mut Vec<Finding>) {
    for (name, requirement) in &contract.tools {
        let required = match requirement {
            crate::schema::ToolRequirement::Simple(_) => true,
            crate::schema::ToolRequirement::Detailed(detail) => detail.required,
        };

        diagnose_command_version(
            "tool",
            name,
            tool_executable_name(name),
            requirement.version(),
            required,
            findings,
        );
    }
}

fn diagnose_org_policy(
    contract: &Contract,
    contract_path: &Path,
    findings: &mut Vec<Finding>,
) -> Option<ProvisioningDiagnostics> {
    let (policy_pack, policy_path) = match load_org_policy_pack_auto(contract_path) {
        Ok(Some(policy_pack)) => policy_pack,
        Ok(None) => return None,
        Err(err) => {
            findings.push(policy_error_finding(err));
            return None;
        }
    };

    let contract_root = contract_working_dir(contract_path);

    let missing_sections = policy_pack.missing_required_sections(contract);
    let missing_files = policy_pack.missing_required_files(contract_root);
    if missing_sections.is_empty() && missing_files.is_empty() {
        let provisioning_plan = policy_pack.provisioning_plan(contract);
        let provisioning_request = policy_pack.provisioning_backend_request(contract);

        if !policy_pack.policies.provisioning.is_empty() {
            let mut sources = Vec::new();
            for (name, rule) in &policy_pack.policies.provisioning {
                let versions = if rule.approved_versions.is_empty() {
                    String::from("any approved version")
                } else {
                    format!("versions {}", rule.approved_versions.join(", "))
                };
                let source_config = format_source_config_summary(rule.source_config.as_ref());
                if source_config.is_empty() {
                    sources.push(format!("{name} via {} ({versions})", rule.source));
                } else {
                    sources.push(format!(
                        "{name} via {} ({versions}; source_config: {source_config})",
                        rule.source
                    ));
                }
            }

            let matched_targets: Vec<String> = policy_pack
                .selected_provisioning_actions(contract)
                .into_iter()
                .map(|entry| {
                    format!(
                        "{} {} {} via {}",
                        entry.target_kind, entry.name, entry.requested_version, entry.source
                    )
                })
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
                    "use this policy surface when repo prerequisites need an approved source",
                ),
            });
        }

        if !policy_pack.policies.adapter_bootstrap.is_empty() {
            let adapter_names = policy_pack
                .policies
                .adapter_bootstrap
                .keys()
                .map(|name| name.as_str())
                .collect::<Vec<_>>();
            let mut sources = Vec::new();
            for (name, rule) in &policy_pack.policies.adapter_bootstrap {
                let versions = if rule.approved_versions.is_empty() {
                    String::from("any approved version")
                } else {
                    format!("versions {}", rule.approved_versions.join(", "))
                };
                sources.push(format!("{name} via {} ({versions})", rule.source));
            }

            let matched_targets: Vec<String> = policy_pack
                .adapter_bootstrap_backend_request(&adapter_names)
                .actions
                .into_iter()
                .map(|action| format!("{} via {}", action.name, action.source))
                .collect();

            findings.push(Finding {
                severity: FindingSeverity::Info,
                summary: String::from("Adapter bootstrap sources are declared"),
                why: if matched_targets.is_empty() {
                    format!(
                        "`{}` declares approved adapter bootstrap sources: {}",
                        compact_display_path(&policy_path),
                        sources.join(", ")
                    )
                } else {
                    format!(
                        "`{}` declares approved adapter bootstrap sources: {}. This repo's declared prerequisites can be provisioned through: {}",
                        compact_display_path(&policy_path),
                        sources.join(", "),
                        matched_targets.join(", ")
                    )
                },
                next: String::from(
                    "use this policy surface when a missing adapter binary needs an approved source",
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

    findings.push(Finding {
        severity: FindingSeverity::Error,
        summary: String::from("Repo does not satisfy org policy pack"),
        why: format!(
            "`{}` requires {}",
            compact_display_path(&policy_path),
            why_parts.join(" and ")
        ),
        next: format!(
            "add the missing items or update `{}`",
            compact_display_path(&policy_path)
        ),
    });

    None
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
    _contract: &Contract,
    contract_path: &Path,
    findings: &mut Vec<Finding>,
) -> Option<AdapterBootstrapDiagnostics> {
    let (policy_pack, policy_path) = match load_org_policy_pack_auto(contract_path) {
        Ok(Some(policy_pack)) => policy_pack,
        Ok(None) => return None,
        Err(_) => return None,
    };

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
                "use this policy surface when adapter bootstrap needs to be approved or audited",
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
            "repair `{}` and re-run `ota doctor`",
            compact_display_path(Path::new(err.path()))
        ),
    }
}

fn diagnose_command_version(
    kind: &str,
    display_name: &str,
    executable_name: &str,
    requirement: &str,
    required: bool,
    findings: &mut Vec<Finding>,
) {
    let Some(actual) = command_version(executable_name) else {
        findings.push(Finding {
            severity: if required {
                FindingSeverity::Error
            } else {
                FindingSeverity::Warn
            },
            summary: format!("Missing {kind}: {display_name}"),
            why: format!("{display_name} is declared in the contract but is not available on PATH"),
            next: format!(
                "install {display_name} and make it available on PATH, then rerun `ota doctor`"
            ),
        });
        return;
    };

    if version_matches(requirement, &actual) {
        return;
    }

    findings.push(Finding {
        severity: if required {
            FindingSeverity::Error
        } else {
            FindingSeverity::Warn
        },
        summary: format!("Version mismatch for {kind}: {display_name}"),
        why: format!(
            "{display_name} resolved to `{actual}` but the contract requires `{requirement}`"
        ),
        next: format!(
            "install a compatible {display_name} version that satisfies `{requirement}`, then rerun `ota doctor`"
        ),
    });
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
    let engines = container_engine_candidates(contract);
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
            "install one of the supported container engines or run `ota run --backend native` if the contract allows it, then rerun `ota doctor`",
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
                next: format!(
                    "run `{}` and fix the reported issue, then rerun `ota doctor`",
                    check.run
                ),
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
    io::stderr().is_terminal()
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
    let output = version_command(name).output().ok()?;
    if !output.status.success() {
        return None;
    }

    let combined = format!(
        "{} {}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );

    extract_version_token(&combined)
}

pub(crate) fn command_available(name: &str) -> bool {
    Command::new(name).output().is_ok()
}

fn version_command(name: &str) -> Command {
    let mut command = Command::new(name);
    if name == "go" {
        command.arg("version");
    } else {
        command.arg("--version");
    }
    command
}

fn contract_working_dir(contract_path: &Path) -> &Path {
    contract_path
        .parent()
        .filter(|path| !path.as_os_str().is_empty())
        .unwrap_or_else(|| Path::new("."))
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
    use std::path::Path;

    use crate::parser::parse_contract_str;
    use crate::test_support::ENV_MUTEX;
    use tempfile::TempDir;

    use super::{
        FindingSeverity, compose_service_healthcheck_command, diagnose_checks_only,
        diagnose_contract, diagnose_preconditions, tool_executable_name, version_matches,
    };

    #[cfg(unix)]
    use std::os::unix::fs::PermissionsExt;

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
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: false
env:
  OTA_DOCTOR_REQUIRED_MISSING:
    required: true
tasks:
  test:
    run: cargo test
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

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
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(
            report.findings[0].summary,
            "Ephemeral lifecycle is advisory only in V1"
        );
    }

    #[test]
    fn warns_when_container_ephemeral_only_applies_to_run() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        let warning = report
            .findings
            .iter()
            .find(|finding| {
                finding.summary
                    == "Ephemeral lifecycle is only enforced for backend-backed task execution"
            })
            .expect("expected lifecycle warning for container+ephemeral configuration");
        assert_eq!(warning.severity, FindingSeverity::Warn);
    }

    #[test]
    fn reports_missing_container_backend_cli() {
        let _guard = ENV_MUTEX.lock().unwrap();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/definitely-not-a-real-bin");
        }

        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

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
        let _guard = ENV_MUTEX.lock().unwrap();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/definitely-not-a-real-bin");
        }

        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

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
        let _guard = ENV_MUTEX.lock().unwrap();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/definitely-not-a-real-bin");
        }

        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

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
        let _guard = ENV_MUTEX.lock().unwrap();
        let original_path = env::var_os("PATH");
        unsafe {
            env::set_var("PATH", "/definitely-not-a-real-bin");
        }

        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

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
        let _guard = ENV_MUTEX.lock().unwrap();
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
    fn reports_unsupported_remote_backend_provider() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

        assert!(!report.ok);
        assert_eq!(
            report.findings[0].summary,
            "Unsupported remote execution backend provider: unknown"
        );
    }

    #[test]
    fn accepts_declared_backend_provider_remote_backend() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

        assert!(report.ok);
        assert!(report.findings.iter().all(|finding| {
            !finding
                .summary
                .contains("Unsupported remote execution backend provider")
        }));
    }

    #[test]
    fn warns_for_suspicious_ssh_remote_target_shape() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
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
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
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
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
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
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
    }

    #[test]
    fn precondition_mode_skips_health_checks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_preconditions(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn checks_only_mode_skips_env_runtime_and_tool_diagnosis() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_REQUIRED:
    required: true
tools:
  ota-tool-that-does-not-exist:
    version: "*"
    required: true
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

        let report = diagnose_checks_only(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].summary, "Check failed: health-check");
    }

    #[test]
    fn reports_optional_tool_version_mismatches_as_warnings() {
        let _guard = ENV_MUTEX.lock().unwrap();
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
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));

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
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(
            report.findings[0].summary,
            "Service healthcheck failed: postgres"
        );
    }

    #[test]
    fn reports_optional_service_healthcheck_failures_as_warnings() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
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
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
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
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(!report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Error);
        assert_eq!(report.findings[0].summary, "No tasks defined in contract");
    }

    #[test]
    fn warns_missing_tasks_for_sdk_type() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota-sdk
  type: sdk
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[0].summary, "No tasks defined in contract");
    }

    #[test]
    fn warns_missing_tasks_for_library_type_case_insensitive() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota-lib
  type: Library
"#,
        )
        .unwrap();

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[0].summary, "No tasks defined in contract");
    }

    #[test]
    fn checks_only_scope_does_not_require_tasks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_checks_only(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert!(report.findings.is_empty());
    }

    #[test]
    fn reports_timed_out_service_healthchecks() {
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
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
            Path::new("ota.yaml"),
            r#"
version: 1
project:
  name: ota
env:
  OTA_DOCTOR_SORT_REQUIRED:
    required: true
tools:
  cargo:
    version: "999.0.0"
    required: false
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
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
    fn reports_invalid_org_policy_pack() {
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
            Path::new("ota.yaml"),
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
            Path::new("ota.yaml"),
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
    fn reports_missing_policy_required_sections() {
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
            Path::new("ota.yaml"),
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
    fn reports_missing_policy_required_files() {
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
            Path::new("ota.yaml"),
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
        let contract = parse_contract_str(
            Path::new("ota.yaml"),
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

        let report = diagnose_contract(&contract, Path::new("ota.yaml"));
        assert!(report.ok);
        assert_eq!(report.findings.len(), 1);
        assert_eq!(report.findings[0].severity, FindingSeverity::Warn);
        assert_eq!(report.findings[0].summary, "Check timed out: slow-check");
        assert!(report.findings[0].why.contains("50ms"));
    }
}
