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
use std::path::Path;

use crate::schema::{
    Contract, RequirementSurface, RuntimeDetail, RuntimePlatformDetail, RuntimeRequirement,
    ToolAcquisitionProvider, ToolAcquisitionSpec, ToolDetail, ToolPlatformDetail, ToolRequirement,
    ToolchainFulfillmentMode, ToolchainProvider, ToolchainSpec,
};

pub(crate) const SHARED_TOOLCHAIN_CORE_SUMMARY: &str =
    "`provider`, `version`, `fulfillment`, `required`, `only_on`, and `platforms.<os>.version`";
pub(crate) const UNKNOWN_TOOLCHAIN_PROVIDER_LABEL: &str = "toolchain provider";
pub(crate) const RUSTUP_TOOLCHAIN_NAME: &str = "rust";
pub(crate) const COREPACK_TOOLCHAIN_NAME: &str = "node";
pub(crate) const JAVA_TOOLCHAIN_NAME: &str = "java";
pub(crate) const PYTHON_TOOLCHAIN_NAME: &str = "python";
const RUSTUP_PROVIDER_SPECIFIC_FIELDS: &[ToolchainProviderSpecificField] = &[
    ToolchainProviderSpecificField::Profile,
    ToolchainProviderSpecificField::Components,
    ToolchainProviderSpecificField::Targets,
];
const COREPACK_PROVIDER_SPECIFIC_FIELDS: &[ToolchainProviderSpecificField] =
    &[ToolchainProviderSpecificField::PackageManagers];
const RUSTUP_PROVIDER_SPECIFIC_FIELD_SUMMARY: &str =
    "`profile`, `components`, `targets`, and their `platforms.<os>.*` overrides";
const COREPACK_PROVIDER_SPECIFIC_FIELD_SUMMARY: &str =
    "`package_managers` and `platforms.<os>.package_managers`";
pub(crate) const JAVA_TOOLCHAIN_OPPORTUNITY_CONTEXT: ToolchainOpportunityContext<'static> =
    ToolchainOpportunityContext {
        ecosystem: "java",
        fallback_runtime: "java",
        fallback_tools: &["maven", "gradle"],
        candidate_providers: &["sdkman", "mise"],
        agent_note: "This repo is a strong candidate for future `toolchains.java` support once Ota ships a Java provider boundary.",
    };
pub(crate) const PYTHON_TOOLCHAIN_OPPORTUNITY_CONTEXT: ToolchainOpportunityContext<'static> =
    ToolchainOpportunityContext {
        ecosystem: "python",
        fallback_runtime: "python",
        fallback_tools: &["uv"],
        candidate_providers: &["uv", "mise"],
        agent_note: "This repo is a strong candidate for future `toolchains.python` support once Ota ships a Python provider boundary.",
    };
pub(crate) const RUSTUP_TOOLCHAIN_CONTRACT: ToolchainProviderContract = ToolchainProviderContract {
    toolchain_name: RUSTUP_TOOLCHAIN_NAME,
    provider: ToolchainProvider::Rustup,
    label: "rustup",
    primary_executable: "rustc",
    owned_runtime: "rust",
    provider_specific_fields: RUSTUP_PROVIDER_SPECIFIC_FIELDS,
    provider_specific_field_summary: RUSTUP_PROVIDER_SPECIFIC_FIELD_SUMMARY,
    requirement_detail_parts_fn: rustup_requirement_detail_parts,
    owned_capabilities_fn: rustup_owned_capabilities,
    owned_tool_requirements_fn: rustup_owned_tool_requirements,
    fulfillment_commands_fn: rustup_fulfillment_commands,
    owned_runtime_remediation_command_fn: rustup_owned_runtime_remediation_command,
    run_fulfillment_validation_error_fn: rustup_run_fulfillment_validation_error,
    managed_surface_probes_fn: rustup_managed_surface_probes,
    managed_surface_entries_fn: rustup_managed_surface_entries,
    managed_surface_remediation_command_fn: rustup_managed_surface_remediation_command,
};
pub(crate) const COREPACK_TOOLCHAIN_CONTRACT: ToolchainProviderContract =
    ToolchainProviderContract {
        toolchain_name: COREPACK_TOOLCHAIN_NAME,
        provider: ToolchainProvider::Corepack,
        label: "corepack",
        primary_executable: "node",
        owned_runtime: "node",
        provider_specific_fields: COREPACK_PROVIDER_SPECIFIC_FIELDS,
        provider_specific_field_summary: COREPACK_PROVIDER_SPECIFIC_FIELD_SUMMARY,
        requirement_detail_parts_fn: corepack_requirement_detail_parts,
        owned_capabilities_fn: corepack_owned_capabilities,
        owned_tool_requirements_fn: corepack_owned_tool_requirements,
        fulfillment_commands_fn: corepack_fulfillment_commands,
        owned_runtime_remediation_command_fn: corepack_owned_runtime_remediation_command,
        run_fulfillment_validation_error_fn: corepack_run_fulfillment_validation_error,
        managed_surface_probes_fn: corepack_managed_surface_probes,
        managed_surface_entries_fn: corepack_managed_surface_entries,
        managed_surface_remediation_command_fn: corepack_managed_surface_remediation_command,
    };

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolchainOwnedCapabilityKind {
    Runtime,
    Tool,
}

impl ToolchainOwnedCapabilityKind {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::Runtime => "runtime",
            Self::Tool => "tool",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolchainOwnedCapability {
    pub(crate) kind: ToolchainOwnedCapabilityKind,
    pub(crate) name: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolchainCommandSpec {
    pub(crate) program: &'static str,
    pub(crate) args: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolchainManagedSurfaceKind {
    Component,
    Target,
}

impl ToolchainManagedSurfaceKind {
    pub(crate) const fn label(self) -> &'static str {
        match self {
            Self::Component => "components",
            Self::Target => "targets",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolchainManagedSurfaceProbe {
    pub(crate) kind: ToolchainManagedSurfaceKind,
    pub(crate) command: String,
    pub(crate) required_entries: Vec<String>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToolchainOpportunityContext<'a> {
    pub(crate) ecosystem: &'a str,
    pub(crate) fallback_runtime: &'a str,
    pub(crate) fallback_tools: &'a [&'a str],
    pub(crate) candidate_providers: &'a [&'a str],
    pub(crate) agent_note: &'a str,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolchainFulfillmentAttemptSummary {
    pub(crate) provider_label: String,
    pub(crate) requirement_clause: String,
    pub(crate) allowance_clause: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ToolchainDiagnosticNarrative {
    pub(crate) summary: String,
    pub(crate) why: String,
    pub(crate) next: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ToolchainProviderSpecificField {
    Profile,
    Components,
    PackageManagers,
    Targets,
}

impl ToolchainProviderSpecificField {
    const fn name(self) -> &'static str {
        match self {
            Self::Profile => "profile",
            Self::Components => "components",
            Self::PackageManagers => "package_managers",
            Self::Targets => "targets",
        }
    }

    fn declared_paths(self, toolchain: &ToolchainSpec) -> Vec<String> {
        let mut fields = Vec::new();
        match self {
            Self::Profile => {
                if toolchain.profile.is_some() {
                    fields.push(String::from("profile"));
                }
                for (platform, detail) in &toolchain.platforms {
                    if detail.profile.is_some() {
                        fields.push(format!("platforms.{platform}.profile"));
                    }
                }
            }
            Self::Components => {
                if !toolchain.components.is_empty() {
                    fields.push(String::from("components"));
                }
                for (platform, detail) in &toolchain.platforms {
                    if !detail.components.is_empty() {
                        fields.push(format!("platforms.{platform}.components"));
                    }
                }
            }
            Self::PackageManagers => {
                if !toolchain.package_managers.is_empty() {
                    fields.push(String::from("package_managers"));
                }
                for (platform, detail) in &toolchain.platforms {
                    if !detail.package_managers.is_empty() {
                        fields.push(format!("platforms.{platform}.package_managers"));
                    }
                }
            }
            Self::Targets => {
                if !toolchain.targets.is_empty() {
                    fields.push(String::from("targets"));
                }
                for (platform, detail) in &toolchain.platforms {
                    if !detail.targets.is_empty() {
                        fields.push(format!("platforms.{platform}.targets"));
                    }
                }
            }
        }
        fields
    }

    fn validation_errors(self, name: &str, toolchain: &ToolchainSpec) -> Vec<String> {
        let mut errors = Vec::new();
        match self {
            Self::Profile => {
                if toolchain
                    .profile
                    .as_deref()
                    .is_some_and(|profile| profile.trim().is_empty())
                {
                    errors.push(format!(
                        "toolchain `{name}` must not declare an empty `profile`"
                    ));
                }
                for (platform, detail) in &toolchain.platforms {
                    if detail
                        .profile
                        .as_deref()
                        .is_some_and(|profile| profile.trim().is_empty())
                    {
                        errors.push(format!(
                            "toolchain `{name}` platform `{platform}` must not declare an empty `profile`"
                        ));
                    }
                }
            }
            Self::Components => {
                if toolchain
                    .components
                    .iter()
                    .any(|component| component.trim().is_empty())
                {
                    errors.push(format!(
                        "toolchain `{name}` must not declare an empty `components` entry"
                    ));
                }
                for (platform, detail) in &toolchain.platforms {
                    if detail
                        .components
                        .iter()
                        .any(|component| component.trim().is_empty())
                    {
                        errors.push(format!(
                            "toolchain `{name}` platform `{platform}` must not declare an empty `components` entry"
                        ));
                    }
                }
            }
            Self::PackageManagers => {
                if toolchain
                    .package_managers
                    .iter()
                    .any(|(name, version)| name.trim().is_empty() || version.trim().is_empty())
                {
                    errors.push(format!(
                        "toolchain `{name}` must not declare empty `package_managers` names or versions"
                    ));
                }
                for (package_name, version) in &toolchain.package_managers {
                    if !package_name.trim().is_empty()
                        && !is_shell_safe_corepack_token(package_name)
                    {
                        errors.push(format!(
                            "toolchain `{name}` package manager `{package_name}` must be a shell-safe Corepack package token"
                        ));
                    }
                    if !version.trim().is_empty() && !is_shell_safe_corepack_token(version) {
                        errors.push(format!(
                            "toolchain `{name}` package manager `{package_name}` version must be a shell-safe Corepack version token"
                        ));
                    }
                }
                for (platform, detail) in &toolchain.platforms {
                    if detail
                        .package_managers
                        .iter()
                        .any(|(name, version)| name.trim().is_empty() || version.trim().is_empty())
                    {
                        errors.push(format!(
                            "toolchain `{name}` platform `{platform}` must not declare empty `package_managers` names or versions"
                        ));
                    }
                    for (package_name, version) in &detail.package_managers {
                        if !package_name.trim().is_empty()
                            && !is_shell_safe_corepack_token(package_name)
                        {
                            errors.push(format!(
                                "toolchain `{name}` platform `{platform}` package manager `{package_name}` must be a shell-safe Corepack package token"
                            ));
                        }
                        if !version.trim().is_empty() && !is_shell_safe_corepack_token(version) {
                            errors.push(format!(
                                "toolchain `{name}` platform `{platform}` package manager `{package_name}` version must be a shell-safe Corepack version token"
                            ));
                        }
                    }
                }
            }
            Self::Targets => {
                if toolchain
                    .targets
                    .iter()
                    .any(|target| target.trim().is_empty())
                {
                    errors.push(format!(
                        "toolchain `{name}` must not declare an empty `targets` entry"
                    ));
                }
                for (platform, detail) in &toolchain.platforms {
                    if detail.targets.iter().any(|target| target.trim().is_empty()) {
                        errors.push(format!(
                            "toolchain `{name}` platform `{platform}` must not declare an empty `targets` entry"
                        ));
                    }
                }
            }
        }
        errors
    }
}

#[derive(Debug, Clone, Copy)]
pub(crate) struct ToolchainProviderContract {
    toolchain_name: &'static str,
    provider: ToolchainProvider,
    label: &'static str,
    primary_executable: &'static str,
    owned_runtime: &'static str,
    provider_specific_fields: &'static [ToolchainProviderSpecificField],
    provider_specific_field_summary: &'static str,
    requirement_detail_parts_fn: fn(ToolchainProviderContract, &ToolchainSpec, &str) -> Vec<String>,
    owned_capabilities_fn:
        fn(ToolchainProviderContract, &ToolchainSpec) -> Vec<ToolchainOwnedCapability>,
    owned_tool_requirements_fn:
        fn(ToolchainProviderContract, &ToolchainSpec, &str) -> BTreeMap<String, ToolRequirement>,
    fulfillment_commands_fn:
        fn(ToolchainProviderContract, &ToolchainSpec, &str) -> Vec<ToolchainCommandSpec>,
    owned_runtime_remediation_command_fn: fn(ToolchainProviderContract, &str) -> Option<String>,
    run_fulfillment_validation_error_fn:
        fn(ToolchainProviderContract, &str, &ToolchainSpec) -> Option<String>,
    managed_surface_probes_fn:
        fn(ToolchainProviderContract, &ToolchainSpec, &str) -> Vec<ToolchainManagedSurfaceProbe>,
    managed_surface_entries_fn: fn(
        ToolchainProviderContract,
        ToolchainManagedSurfaceKind,
        &ToolchainSpec,
        &str,
    ) -> Vec<String>,
    managed_surface_remediation_command_fn:
        fn(ToolchainProviderContract, ToolchainManagedSurfaceKind, &str) -> Option<String>,
}

impl ToolchainProviderContract {
    pub(crate) const fn toolchain_name(self) -> &'static str {
        self.toolchain_name
    }

    pub(crate) const fn provider(self) -> ToolchainProvider {
        self.provider
    }

    pub(crate) const fn label(self) -> &'static str {
        self.label
    }

    pub(crate) const fn primary_executable(self) -> &'static str {
        self.primary_executable
    }

    pub(crate) const fn owned_runtime(self) -> &'static str {
        self.owned_runtime
    }

    pub(crate) fn owned_runtime_remediation_command(self, requirement: &str) -> Option<String> {
        (self.owned_runtime_remediation_command_fn)(self, requirement)
    }

    pub(crate) const fn provider_hint(self) -> &'static str {
        self.label
    }

    pub(crate) const fn shared_core_summary(self) -> &'static str {
        SHARED_TOOLCHAIN_CORE_SUMMARY
    }

    pub(crate) const fn provider_specific_field_summary(self) -> &'static str {
        self.provider_specific_field_summary
    }

    pub(crate) fn declared_provider_specific_fields(
        self,
        toolchain: &ToolchainSpec,
    ) -> Vec<String> {
        self.provider_specific_fields
            .iter()
            .flat_map(|field| field.declared_paths(toolchain))
            .collect()
    }

    pub(crate) fn provider_specific_validation_errors(
        self,
        name: &str,
        toolchain: &ToolchainSpec,
    ) -> Vec<String> {
        let mut errors = self
            .provider_specific_fields
            .iter()
            .flat_map(|field| field.validation_errors(name, toolchain))
            .collect::<Vec<_>>();
        let disallowed_fields = declared_known_provider_specific_fields(toolchain)
            .into_iter()
            .filter(|field| !self.provider_specific_fields_allow(field.as_str()))
            .collect::<Vec<_>>();
        if !disallowed_fields.is_empty() {
            let toolchain_ref = format!("toolchains.{}", self.toolchain_name());
            let restriction = if self.provider_specific_fields.is_empty() {
                format!(
                    "current `{toolchain_ref}` only supports the shared provider-agnostic fields"
                )
            } else {
                format!(
                    "valid provider-specific fields for `{toolchain_ref}` are {}",
                    self.provider_specific_field_summary()
                )
            };
            errors.extend(disallowed_fields.into_iter().map(|field| {
                format!(
                    "toolchain `{name}` with `provider: {}` must not declare `{field}`; {restriction}",
                    self.label()
                )
            }));
        }
        errors
    }

    fn provider_specific_fields_allow(self, field_name: &str) -> bool {
        self.provider_specific_fields.iter().any(|field| {
            field.name() == field_name || field_name.ends_with(&format!(".{}", field.name()))
        })
    }

    pub(crate) fn requirement_detail_parts(
        self,
        toolchain: &ToolchainSpec,
        target_os: &str,
    ) -> Vec<String> {
        (self.requirement_detail_parts_fn)(self, toolchain, target_os)
    }

    pub(crate) fn owned_capabilities(
        self,
        toolchain: &ToolchainSpec,
    ) -> Vec<ToolchainOwnedCapability> {
        (self.owned_capabilities_fn)(self, toolchain)
    }

    pub(crate) fn fulfillment_commands(
        self,
        toolchain: &ToolchainSpec,
        target_os: &str,
    ) -> Vec<ToolchainCommandSpec> {
        (self.fulfillment_commands_fn)(self, toolchain, target_os)
    }

    pub(crate) fn owned_tool_requirements(
        self,
        toolchain: &ToolchainSpec,
        target_os: &str,
    ) -> BTreeMap<String, ToolRequirement> {
        (self.owned_tool_requirements_fn)(self, toolchain, target_os)
    }

    pub(crate) fn run_fulfillment_validation_error(
        self,
        name: &str,
        toolchain: &ToolchainSpec,
    ) -> Option<String> {
        (self.run_fulfillment_validation_error_fn)(self, name, toolchain)
    }

    pub(crate) fn managed_surface_probes(
        self,
        toolchain: &ToolchainSpec,
        target_os: &str,
    ) -> Vec<ToolchainManagedSurfaceProbe> {
        (self.managed_surface_probes_fn)(self, toolchain, target_os)
    }

    pub(crate) fn managed_surface_entries(
        self,
        kind: ToolchainManagedSurfaceKind,
        toolchain: &ToolchainSpec,
        target_os: &str,
    ) -> Vec<String> {
        (self.managed_surface_entries_fn)(self, kind, toolchain, target_os)
    }

    pub(crate) fn missing_provider_diagnostic(
        self,
        toolchain_name: &str,
        surface: &str,
        rerun_command: &str,
    ) -> ToolchainDiagnosticNarrative {
        ToolchainDiagnosticNarrative {
            summary: format!("Missing toolchain provider: {}", self.label()),
            why: format!(
                "ota needs `{}` to inspect or fulfill {surface} for toolchain `{toolchain_name}` on the selected execution path",
                self.label()
            ),
            next: format!(
                "install `{}` or remove provider-managed {surface} from toolchain `{toolchain_name}`, then rerun `{rerun_command}`",
                self.label()
            ),
        }
    }

    pub(crate) fn probe_failed_diagnostic(
        self,
        toolchain_name: &str,
        command: &str,
        details: &str,
        rerun_command: &str,
    ) -> ToolchainDiagnosticNarrative {
        ToolchainDiagnosticNarrative {
            summary: format!("Toolchain provider probe failed: {toolchain_name}"),
            why: format!(
                "ota could not inspect toolchain `{toolchain_name}` through `{}`; `{command}` failed: {details}",
                self.label()
            ),
            next: format!("run `{command}` directly and rerun `{rerun_command}`"),
        }
    }

    pub(crate) fn missing_managed_surface_diagnostic(
        self,
        toolchain_name: &str,
        kind: ToolchainManagedSurfaceKind,
        entry: &str,
        rerun_command: &str,
    ) -> ToolchainDiagnosticNarrative {
        ToolchainDiagnosticNarrative {
            summary: format!(
                "Missing toolchain {}: {toolchain_name}.{entry}",
                kind.label().trim_end_matches('s')
            ),
            why: format!(
                "ota inspected toolchain `{toolchain_name}` through `{}`, but {} `{entry}` is not installed",
                self.label(),
                kind.label().trim_end_matches('s')
            ),
            next: format!(
                "run `{}` and rerun `{rerun_command}`",
                self.managed_surface_remediation_command(kind, entry)
            ),
        }
    }

    pub(crate) fn managed_surface_remediation_command(
        self,
        kind: ToolchainManagedSurfaceKind,
        entry: &str,
    ) -> String {
        (self.managed_surface_remediation_command_fn)(self, kind, entry).unwrap_or_default()
    }
}

pub(crate) fn toolchain_provider_contract(
    name: &str,
    provider: ToolchainProvider,
) -> Option<ToolchainProviderContract> {
    shipped_toolchain_contracts()
        .iter()
        .copied()
        .find(|contract| contract.toolchain_name() == name && contract.provider() == provider)
}

pub(crate) fn shipped_toolchain_contracts() -> &'static [ToolchainProviderContract] {
    &[RUSTUP_TOOLCHAIN_CONTRACT, COREPACK_TOOLCHAIN_CONTRACT]
}

pub(crate) fn shipped_toolchain_contract_by_name(name: &str) -> Option<ToolchainProviderContract> {
    shipped_toolchain_contracts()
        .iter()
        .copied()
        .find(|contract| contract.toolchain_name() == name)
}

pub(crate) fn shipped_toolchain_contract_by_provider(
    provider: ToolchainProvider,
) -> Option<ToolchainProviderContract> {
    shipped_toolchain_contracts()
        .iter()
        .copied()
        .find(|contract| contract.provider() == provider)
}

pub(crate) fn shipped_toolchain_contract_by_label(
    label: &str,
) -> Option<ToolchainProviderContract> {
    shipped_toolchain_contracts()
        .iter()
        .copied()
        .find(|contract| contract.label() == label)
}

pub(crate) fn shipped_toolchain_contracts_summary() -> String {
    shipped_toolchain_contracts()
        .iter()
        .map(|contract| {
            format!(
                "`toolchains.{}` with `provider: {}`",
                contract.toolchain_name(),
                contract.label()
            )
        })
        .collect::<Vec<_>>()
        .join(" and ")
}

pub(crate) fn declared_toolchain_contract(
    name: &str,
    toolchain: &ToolchainSpec,
) -> Option<ToolchainProviderContract> {
    toolchain_provider_contract(name, toolchain.provider)
}

pub(crate) fn declared_toolchain_provider_label(
    name: &str,
    toolchain: &ToolchainSpec,
) -> &'static str {
    declared_toolchain_contract(name, toolchain)
        .map(|provider| provider.label())
        .unwrap_or(UNKNOWN_TOOLCHAIN_PROVIDER_LABEL)
}

pub(crate) fn declared_toolchain_requirement_clause(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> String {
    let parts = declared_toolchain_contract(name, toolchain)
        .map(|provider| provider.requirement_detail_parts(toolchain, target_os))
        .unwrap_or_else(|| vec![format!("version `{}`", toolchain.version_for_os(target_os))]);
    format!(
        "check toolchain `{name}` via `{}` ({})",
        declared_toolchain_provider_label(name, toolchain),
        parts.join(", ")
    )
}

pub(crate) fn declared_toolchain_preview_action(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> String {
    let base = declared_toolchain_requirement_clause(name, toolchain, target_os);
    match toolchain.fulfillment_mode() {
        ToolchainFulfillmentMode::None => {
            format!("{base}; fulfillment: none (diagnose only, no provisioning)")
        }
        ToolchainFulfillmentMode::Run => format!(
            "{base}; fulfillment: run (ota may provision the selected toolchain on the selected run path)"
        ),
    }
}

pub(crate) fn declared_toolchain_fulfillment_attempt_summary(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> ToolchainFulfillmentAttemptSummary {
    let provider_label = declared_toolchain_provider_label(name, toolchain).to_string();
    ToolchainFulfillmentAttemptSummary {
        requirement_clause: declared_toolchain_requirement_clause(name, toolchain, target_os),
        allowance_clause: format!(
            "`toolchains.{name}.fulfillment: run` allowed ota to attempt run-path provisioning via `{provider_label}`"
        ),
        provider_label,
    }
}

pub(crate) fn declared_known_provider_specific_fields(toolchain: &ToolchainSpec) -> Vec<String> {
    known_provider_specific_fields()
        .into_iter()
        .flat_map(|field| field.declared_paths(toolchain))
        .collect()
}

pub(crate) fn known_provider_specific_field_owner_groups(
    toolchain: &ToolchainSpec,
) -> Vec<(ToolchainProviderContract, Vec<String>)> {
    shipped_toolchain_contracts()
        .iter()
        .copied()
        .filter_map(|contract| {
            let fields = contract.declared_provider_specific_fields(toolchain);
            (!fields.is_empty()).then_some((contract, fields))
        })
        .collect()
}

pub(crate) fn toolchain_provider_label(provider: ToolchainProvider) -> &'static str {
    shipped_toolchain_contract_by_provider(provider)
        .map(|contract| contract.label())
        .unwrap_or(UNKNOWN_TOOLCHAIN_PROVIDER_LABEL)
}

pub(crate) fn fallback_toolchain_fulfillment_attempt_summary(
    toolchain_name: &str,
) -> ToolchainFulfillmentAttemptSummary {
    ToolchainFulfillmentAttemptSummary {
        provider_label: String::from(UNKNOWN_TOOLCHAIN_PROVIDER_LABEL),
        requirement_clause: format!(
            "check toolchain `{toolchain_name}` via `{UNKNOWN_TOOLCHAIN_PROVIDER_LABEL}`"
        ),
        allowance_clause: format!(
            "`toolchains.{toolchain_name}.fulfillment: run` allowed ota to attempt run-path provisioning via `{UNKNOWN_TOOLCHAIN_PROVIDER_LABEL}`"
        ),
    }
}

pub(crate) fn toolchain_fulfillment_status_detail(
    fulfillment: &str,
    fulfilled: bool,
) -> &'static str {
    if fulfilled {
        "applied on this execution path"
    } else if fulfillment == "run" {
        "selected on this execution path; no provider fulfillment command ran"
    } else {
        "check-only on this execution path"
    }
}

fn owned_runtime_requirement_from_toolchain(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
) -> RuntimeRequirement {
    RuntimeRequirement::Detailed(RuntimeDetail {
        version: toolchain.version.clone(),
        required: toolchain.required,
        only_on: toolchain.only_on.clone(),
        provider: Some(provider.label().to_string()),
        distribution: None,
        platforms: toolchain
            .platforms
            .iter()
            .map(|(platform, detail)| {
                (
                    platform.clone(),
                    RuntimePlatformDetail {
                        version: detail.version.clone(),
                        provider: Some(provider.label().to_string()),
                        distribution: None,
                    },
                )
            })
            .collect(),
    })
}

pub(crate) fn requirement_surface_with_toolchain_owned_runtimes(
    contract: &Contract,
    base: &RequirementSurface,
    toolchain_names: &BTreeSet<String>,
) -> RequirementSurface {
    let mut merged = base.clone();

    for toolchain_name in toolchain_names {
        let Some(toolchain) = contract.toolchains.get(toolchain_name.as_str()) else {
            continue;
        };
        let Some(provider) = declared_toolchain_contract(toolchain_name, toolchain) else {
            continue;
        };
        let runtime_name = provider.owned_runtime().to_string();
        let runtime_requirement = owned_runtime_requirement_from_toolchain(provider, toolchain);
        let merged_requirement = merged
            .runtimes
            .get(runtime_name.as_str())
            .map(|base_requirement| base_requirement.merged_with_overlay(&runtime_requirement))
            .unwrap_or(runtime_requirement);
        merged.runtimes.insert(runtime_name, merged_requirement);
    }

    merged
}

pub(crate) fn requirement_surface_with_toolchain_owned_tools(
    contract: &Contract,
    base: &RequirementSurface,
    toolchain_names: &BTreeSet<String>,
    target_os: &str,
) -> RequirementSurface {
    let mut merged = base.clone();

    for toolchain_name in toolchain_names {
        let Some(toolchain) = contract.toolchains.get(toolchain_name.as_str()) else {
            continue;
        };
        let Some(provider) = declared_toolchain_contract(toolchain_name, toolchain) else {
            continue;
        };
        for (tool_name, requirement) in provider.owned_tool_requirements(toolchain, target_os) {
            let merged_requirement = merged
                .tools
                .get(tool_name.as_str())
                .map(|base_requirement| base_requirement.merged_with_overlay(&requirement))
                .unwrap_or(requirement);
            merged.tools.insert(tool_name, merged_requirement);
        }
    }

    merged
}

pub(crate) fn requirement_surface_with_toolchain_owned_capabilities(
    contract: &Contract,
    base: &RequirementSurface,
    toolchain_names: &BTreeSet<String>,
    target_os: &str,
) -> RequirementSurface {
    let with_runtimes =
        requirement_surface_with_toolchain_owned_runtimes(contract, base, toolchain_names);
    requirement_surface_with_toolchain_owned_tools(
        contract,
        &with_runtimes,
        toolchain_names,
        target_os,
    )
}

fn base_requirement_detail_parts(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<String> {
    vec![
        format!("owns runtime `{}`", provider.owned_runtime()),
        format!("version `{}`", toolchain.version_for_os(target_os)),
    ]
}

fn rustup_requirement_detail_parts(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<String> {
    let mut parts = base_requirement_detail_parts(provider, toolchain, target_os);
    let components = toolchain.components_for_os(target_os);
    if !components.is_empty() {
        parts.push(format!("components `{}`", components.join("`, `")));
    }
    let targets = toolchain.targets_for_os(target_os);
    if !targets.is_empty() {
        parts.push(format!("targets `{}`", targets.join("`, `")));
    }
    parts
}

fn corepack_requirement_detail_parts(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<String> {
    let mut parts = base_requirement_detail_parts(provider, toolchain, target_os);
    let package_managers = toolchain.package_managers_for_os(target_os);
    if !package_managers.is_empty() {
        let rendered = package_managers
            .iter()
            .map(|(name, version)| format!("{name}@{version}"))
            .collect::<Vec<_>>();
        parts.push(format!("package managers `{}`", rendered.join("`, `")));
    }
    parts
}

fn rustup_owned_capabilities(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
) -> Vec<ToolchainOwnedCapability> {
    let mut owned = vec![
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Runtime,
            name: provider.owned_runtime().to_string(),
        },
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: String::from("cargo"),
        },
    ];
    for component in toolchain.components.iter().chain(
        toolchain
            .platforms
            .values()
            .flat_map(|platform| platform.components.iter()),
    ) {
        let Some(tool_name) = rustup_component_tool_name(component.as_str()) else {
            continue;
        };
        if owned.iter().any(|capability| {
            capability.kind == ToolchainOwnedCapabilityKind::Tool && capability.name == tool_name
        }) {
            continue;
        }
        owned.push(ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: tool_name.to_string(),
        });
    }
    owned
}

fn corepack_owned_capabilities(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
) -> Vec<ToolchainOwnedCapability> {
    let mut owned = vec![
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Runtime,
            name: provider.owned_runtime().to_string(),
        },
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: provider.owned_runtime().to_string(),
        },
    ];
    let mut package_managers = BTreeSet::new();
    package_managers.extend(toolchain.package_managers.keys().cloned());
    for detail in toolchain.platforms.values() {
        package_managers.extend(detail.package_managers.keys().cloned());
    }
    for package_manager in package_managers {
        owned.push(ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: package_manager,
        });
    }
    owned
}

fn rustup_owned_tool_requirements(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> BTreeMap<String, ToolRequirement> {
    BTreeMap::new()
}

fn corepack_owned_tool_requirements(
    _provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> BTreeMap<String, ToolRequirement> {
    toolchain
        .package_managers_for_os(target_os)
        .into_iter()
        .map(|(name, version)| {
            (
                name.clone(),
                ToolRequirement::Detailed(ToolDetail {
                    version: version.clone(),
                    required: toolchain.required_for_os(target_os),
                    only_on: toolchain.only_on.clone(),
                    platforms: BTreeMap::<String, ToolPlatformDetail>::new(),
                    acquisition: Some(ToolAcquisitionSpec {
                        provider: ToolAcquisitionProvider::Corepack,
                        package: Some(name.clone()),
                        version: Some(version),
                        shell: None,
                        run: None,
                    }),
                }),
            )
        })
        .collect()
}

fn rustup_fulfillment_commands(
    _provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    let mut args = vec![
        String::from("toolchain"),
        String::from("install"),
        toolchain.version_for_os(target_os).to_string(),
    ];
    if let Some(profile) = toolchain.profile_for_os(target_os) {
        args.push(String::from("--profile"));
        args.push(profile.to_string());
    }
    for component in toolchain.components_for_os(target_os) {
        args.push(String::from("--component"));
        args.push(component);
    }
    for target in toolchain.targets_for_os(target_os) {
        args.push(String::from("--target"));
        args.push(target);
    }
    vec![ToolchainCommandSpec {
        program: "rustup",
        args,
    }]
}

fn corepack_fulfillment_commands(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    Vec::new()
}

fn rustup_owned_runtime_remediation_command(
    _provider: ToolchainProviderContract,
    requirement: &str,
) -> Option<String> {
    Some(format!("rustup toolchain install {requirement}"))
}

fn corepack_owned_runtime_remediation_command(
    _provider: ToolchainProviderContract,
    _requirement: &str,
) -> Option<String> {
    None
}

fn rustup_run_fulfillment_validation_error(
    _provider: ToolchainProviderContract,
    name: &str,
    toolchain: &ToolchainSpec,
) -> Option<String> {
    let trimmed = toolchain.version.trim();
    let installable = !trimmed.is_empty()
        && !trimmed.contains(char::is_whitespace)
        && !trimmed.contains('>')
        && !trimmed.contains('<')
        && !trimmed.contains('=')
        && !trimmed.contains('^')
        && !trimmed.contains('~')
        && !trimmed.contains('*');
    (!installable).then(|| {
        format!(
            "toolchain `{name}` uses `provider: rustup` with `fulfillment: run`, so `toolchains.{name}.version` must be an installable rustup toolchain reference like `stable`, `beta`, `nightly`, or `1.94.0`"
        )
    })
}

fn corepack_run_fulfillment_validation_error(
    _provider: ToolchainProviderContract,
    name: &str,
    _toolchain: &ToolchainSpec,
) -> Option<String> {
    Some(format!(
        "toolchain `{name}` uses `provider: corepack` with `fulfillment: run`, but Corepack-backed Node toolchains are currently check-only; keep `toolchains.{name}.fulfillment: none` and declare package-manager activation under `toolchains.{name}.package_managers`"
    ))
}

fn rustup_managed_surface_probes(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<ToolchainManagedSurfaceProbe> {
    let mut probes = Vec::new();
    let components = toolchain.components_for_os(target_os);
    if !components.is_empty() {
        probes.push(ToolchainManagedSurfaceProbe {
            kind: ToolchainManagedSurfaceKind::Component,
            command: format!("{} component list --installed", provider.label()),
            required_entries: components,
        });
    }
    let targets = toolchain.targets_for_os(target_os);
    if !targets.is_empty() {
        probes.push(ToolchainManagedSurfaceProbe {
            kind: ToolchainManagedSurfaceKind::Target,
            command: format!("{} target list --installed", provider.label()),
            required_entries: targets,
        });
    }
    probes
}

fn rustup_managed_surface_entries(
    _provider: ToolchainProviderContract,
    kind: ToolchainManagedSurfaceKind,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<String> {
    match kind {
        ToolchainManagedSurfaceKind::Component => toolchain.components_for_os(target_os),
        ToolchainManagedSurfaceKind::Target => toolchain.targets_for_os(target_os),
    }
}

fn corepack_managed_surface_probes(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainManagedSurfaceProbe> {
    Vec::new()
}

fn corepack_managed_surface_entries(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<String> {
    Vec::new()
}

fn is_shell_safe_corepack_token(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '@' | '/' | '.' | '_' | '-' | '+' | '~')
        })
}

fn rustup_managed_surface_remediation_command(
    _provider: ToolchainProviderContract,
    kind: ToolchainManagedSurfaceKind,
    entry: &str,
) -> Option<String> {
    Some(match kind {
        ToolchainManagedSurfaceKind::Component => format!("rustup component add {entry}"),
        ToolchainManagedSurfaceKind::Target => format!("rustup target add {entry}"),
    })
}

fn corepack_managed_surface_remediation_command(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _entry: &str,
) -> Option<String> {
    None
}

fn rustup_component_tool_name(component: &str) -> Option<&'static str> {
    match component {
        "rustfmt" => Some("rustfmt"),
        "clippy" => Some("clippy"),
        _ => None,
    }
}

fn known_provider_specific_fields() -> Vec<ToolchainProviderSpecificField> {
    let mut fields = Vec::new();
    for contract in shipped_toolchain_contracts() {
        for field in contract.provider_specific_fields {
            if !fields.contains(field) {
                fields.push(*field);
            }
        }
    }
    fields
}

pub(crate) fn unsupported_toolchain_opportunity_context(
    ecosystem: &str,
) -> Option<ToolchainOpportunityContext<'static>> {
    match ecosystem {
        JAVA_TOOLCHAIN_NAME => Some(JAVA_TOOLCHAIN_OPPORTUNITY_CONTEXT),
        PYTHON_TOOLCHAIN_NAME => Some(PYTHON_TOOLCHAIN_OPPORTUNITY_CONTEXT),
        _ => None,
    }
}

pub(crate) fn unsupported_toolchain_repo_signals(
    contract_root: &Path,
    ecosystem: &str,
) -> Vec<&'static str> {
    match ecosystem {
        JAVA_TOOLCHAIN_NAME => {
            let mut signals = Vec::new();
            if contract_root.join("pom.xml").is_file() {
                signals.push("pom.xml");
            }
            if contract_root.join("build.gradle").is_file() {
                signals.push("build.gradle");
            }
            if contract_root.join("build.gradle.kts").is_file() {
                signals.push("build.gradle.kts");
            }
            if contract_root.join(".sdkmanrc").is_file() {
                signals.push(".sdkmanrc");
            }
            if tool_versions_entry(contract_root, &[JAVA_TOOLCHAIN_NAME]).is_some() {
                signals.push(".tool-versions");
            }
            signals
        }
        PYTHON_TOOLCHAIN_NAME => {
            let mut signals = Vec::new();
            if contract_root.join("uv.lock").is_file() {
                signals.push("uv.lock");
            }
            if contract_root.join("pyproject.toml").is_file() {
                signals.push("pyproject.toml");
            }
            if contract_root.join(".python-version").is_file() {
                signals.push(".python-version");
            }
            if tool_versions_entry(contract_root, &[PYTHON_TOOLCHAIN_NAME]).is_some() {
                signals.push(".tool-versions");
            }
            signals
        }
        _ => Vec::new(),
    }
}

pub(crate) fn tool_versions_entry(
    contract_root: &Path,
    candidate_names: &[&str],
) -> Option<String> {
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::parser::parse_contract_str;
    use std::path::Path;

    fn contract(yaml: &str) -> Contract {
        parse_contract_str(Path::new("./ota.yaml"), yaml).unwrap()
    }

    #[test]
    fn rustup_contract_exposes_provider_specific_behavior_without_provider_match_fallback() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
    profile: minimal
    components:
      - rustfmt
    targets:
      - x86_64-unknown-linux-musl
"#,
        );
        let toolchain = contract.toolchains.get("rust").unwrap();
        let provider = declared_toolchain_contract("rust", toolchain).unwrap();

        assert_eq!(
            provider.provider_specific_field_summary(),
            RUSTUP_PROVIDER_SPECIFIC_FIELD_SUMMARY
        );
        assert!(
            provider
                .requirement_detail_parts(toolchain, "linux")
                .iter()
                .any(|part| part.contains("components `rustfmt`"))
        );
        assert!(
            provider
                .managed_surface_probes(toolchain, "linux")
                .iter()
                .any(|probe| probe.command == "rustup component list --installed")
        );
        assert_eq!(
            provider.managed_surface_entries(
                ToolchainManagedSurfaceKind::Component,
                toolchain,
                "linux"
            ),
            vec![String::from("rustfmt")]
        );
        assert_eq!(
            provider.managed_surface_remediation_command(
                ToolchainManagedSurfaceKind::Target,
                "x86_64-unknown-linux-musl"
            ),
            "rustup target add x86_64-unknown-linux-musl"
        );
        assert_eq!(
            provider.owned_runtime_remediation_command("1.94.0"),
            Some(String::from("rustup toolchain install 1.94.0"))
        );
    }

    #[test]
    fn corepack_contract_stays_check_only_through_provider_contract_hooks() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    provider: corepack
    version: "22"
    package_managers:
      pnpm: "10.22.0"
"#,
        );
        let toolchain = contract.toolchains.get("node").unwrap();
        let provider = declared_toolchain_contract("node", toolchain).unwrap();

        assert_eq!(
            provider.provider_specific_field_summary(),
            COREPACK_PROVIDER_SPECIFIC_FIELD_SUMMARY
        );
        assert!(provider.fulfillment_commands(toolchain, "linux").is_empty());
        assert!(
            provider
                .managed_surface_probes(toolchain, "linux")
                .is_empty()
        );
        assert!(
            provider
                .managed_surface_entries(ToolchainManagedSurfaceKind::Component, toolchain, "linux")
                .is_empty()
        );
        assert!(
            provider
                .requirement_detail_parts(toolchain, "linux")
                .iter()
                .any(|part| part.contains("package managers `pnpm@10.22.0`"))
        );
        let tool_requirements = provider.owned_tool_requirements(toolchain, "linux");
        assert_eq!(
            tool_requirements
                .get("pnpm")
                .expect("projected pnpm requirement")
                .version(),
            "10.22.0"
        );
        assert!(
            provider
                .run_fulfillment_validation_error("node", toolchain)
                .is_some()
        );
        assert_eq!(provider.owned_runtime_remediation_command("22"), None);
    }

    #[test]
    fn shipped_registry_is_the_source_of_provider_lookup_and_field_discovery() {
        assert_eq!(
            toolchain_provider_contract("rust", ToolchainProvider::Rustup)
                .unwrap()
                .label(),
            "rustup"
        );
        assert_eq!(
            toolchain_provider_contract("node", ToolchainProvider::Corepack)
                .unwrap()
                .label(),
            "corepack"
        );
        assert_eq!(
            shipped_toolchain_contract_by_label("rustup")
                .unwrap()
                .toolchain_name(),
            "rust"
        );
        assert_eq!(
            toolchain_provider_label(ToolchainProvider::Rustup),
            "rustup"
        );
        assert_eq!(
            known_provider_specific_fields(),
            vec![
                ToolchainProviderSpecificField::Profile,
                ToolchainProviderSpecificField::Components,
                ToolchainProviderSpecificField::Targets,
                ToolchainProviderSpecificField::PackageManagers,
            ]
        );
    }

    #[test]
    fn projected_toolchain_owned_runtime_carries_provider_hint() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
"#,
        );
        let names = BTreeSet::from([String::from("rust")]);
        let surface = requirement_surface_with_toolchain_owned_runtimes(
            &contract,
            &RequirementSurface::default(),
            &names,
        );

        let rust = surface
            .runtimes
            .get("rust")
            .expect("projected rust runtime");
        assert_eq!(rust.provider_for_os("linux"), Some("rustup"));
    }

    #[test]
    fn declared_toolchain_fulfillment_attempt_summary_uses_provider_contract_wording() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  rust:
    provider: rustup
    version: "1.94.0"
    fulfillment: run
    components:
      - rustfmt
"#,
        );
        let toolchain = contract.toolchains.get("rust").unwrap();
        let summary = declared_toolchain_fulfillment_attempt_summary("rust", toolchain, "linux");

        assert_eq!(summary.provider_label, "rustup");
        assert!(
            summary
                .requirement_clause
                .contains("check toolchain `rust` via `rustup`"),
            "{summary:?}"
        );
        assert!(
            summary.allowance_clause.contains(
                "`toolchains.rust.fulfillment: run` allowed ota to attempt run-path provisioning via `rustup`"
            ),
            "{summary:?}"
        );
    }

    #[test]
    fn rustup_contract_exposes_doctor_diagnostic_narratives() {
        let diagnostic = RUSTUP_TOOLCHAIN_CONTRACT.missing_provider_diagnostic(
            "rust",
            "components",
            "ota doctor",
        );
        assert_eq!(diagnostic.summary, "Missing toolchain provider: rustup");
        assert!(diagnostic.why.contains("inspect or fulfill components"));
        assert!(diagnostic.next.contains("rerun `ota doctor`"));
    }

    #[test]
    fn fallback_toolchain_summary_helpers_are_stable() {
        let summary = fallback_toolchain_fulfillment_attempt_summary("rust");
        assert_eq!(summary.provider_label, UNKNOWN_TOOLCHAIN_PROVIDER_LABEL);
        assert_eq!(
            toolchain_fulfillment_status_detail("run", false),
            "selected on this execution path; no provider fulfillment command ran"
        );
        assert_eq!(
            toolchain_fulfillment_status_detail("none", false),
            "check-only on this execution path"
        );
    }
}
