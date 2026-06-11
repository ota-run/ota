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

use semver::{Op, VersionReq};

use crate::schema::{
    Contract, RequirementSurface, RuntimeDetail, RuntimePlatformDetail, RuntimeRequirement,
    ToolAcquisitionProvider, ToolAcquisitionSpec, ToolDetail, ToolPlatformDetail, ToolRequirement,
    ToolchainFulfillmentMode, ToolchainFulfillmentSource, ToolchainProvider, ToolchainSpec,
};

pub(crate) const SHARED_TOOLCHAIN_CORE_SUMMARY: &str = "`version`, `fulfillment`, `required`, `only_on`, and `platforms.<os>.version` (legacy `provider` is still accepted for compatibility)";
pub(crate) const UNKNOWN_TOOLCHAIN_PROVIDER_LABEL: &str = "toolchain provider";
pub(crate) const RUSTUP_TOOLCHAIN_NAME: &str = "rust";
pub(crate) const COREPACK_TOOLCHAIN_NAME: &str = "node";
pub(crate) const JAVA_TOOLCHAIN_NAME: &str = "java";
pub(crate) const PYTHON_TOOLCHAIN_NAME: &str = "python";
pub(crate) const GO_TOOLCHAIN_NAME: &str = "go";
pub(crate) const RUBY_TOOLCHAIN_NAME: &str = "ruby";
pub(crate) const DOTNET_TOOLCHAIN_NAME: &str = "dotnet";
const RUSTUP_PROVIDER_SPECIFIC_FIELDS: &[ToolchainProviderSpecificField] = &[
    ToolchainProviderSpecificField::Profile,
    ToolchainProviderSpecificField::Components,
    ToolchainProviderSpecificField::Targets,
];
const COREPACK_PROVIDER_SPECIFIC_FIELDS: &[ToolchainProviderSpecificField] =
    &[ToolchainProviderSpecificField::PackageManagers];
const UV_PROVIDER_SPECIFIC_FIELDS: &[ToolchainProviderSpecificField] =
    &[ToolchainProviderSpecificField::PackageManagers];
const RUBY_PROVIDER_SPECIFIC_FIELDS: &[ToolchainProviderSpecificField] =
    &[ToolchainProviderSpecificField::PackageManagers];
const RUSTUP_PROVIDER_SPECIFIC_FIELD_SUMMARY: &str =
    "`profile`, `components`, `targets`, and their `platforms.<os>.*` overrides";
const COREPACK_PROVIDER_SPECIFIC_FIELD_SUMMARY: &str =
    "`package_managers` and `platforms.<os>.package_managers`";
const UV_PROVIDER_SPECIFIC_FIELD_SUMMARY: &str =
    "`package_managers` and `platforms.<os>.package_managers` (uv and Poetry only)";
const RUBY_PROVIDER_SPECIFIC_FIELD_SUMMARY: &str =
    "`package_managers` and `platforms.<os>.package_managers` (Bundler only)";
const UNSUPPORTED_TOOLCHAIN_OPPORTUNITY_ECOSYSTEMS: &[&str] = &[];
pub(crate) const RUSTUP_TOOLCHAIN_CONTRACT: ToolchainProviderContract = ToolchainProviderContract {
    toolchain_name: RUSTUP_TOOLCHAIN_NAME,
    provider: ToolchainProvider::Rustup,
    label: "rustup",
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
pub(crate) const SDKMAN_TOOLCHAIN_CONTRACT: ToolchainProviderContract = ToolchainProviderContract {
    toolchain_name: JAVA_TOOLCHAIN_NAME,
    provider: ToolchainProvider::Sdkman,
    label: "sdkman",
    owned_runtime: JAVA_TOOLCHAIN_NAME,
    provider_specific_fields: &[],
    provider_specific_field_summary: "",
    requirement_detail_parts_fn: base_requirement_detail_parts,
    owned_capabilities_fn: sdkman_owned_capabilities,
    owned_tool_requirements_fn: sdkman_owned_tool_requirements,
    fulfillment_commands_fn: sdkman_fulfillment_commands,
    owned_runtime_remediation_command_fn: sdkman_owned_runtime_remediation_command,
    run_fulfillment_validation_error_fn: sdkman_run_fulfillment_validation_error,
    managed_surface_probes_fn: sdkman_managed_surface_probes,
    managed_surface_entries_fn: sdkman_managed_surface_entries,
    managed_surface_remediation_command_fn: sdkman_managed_surface_remediation_command,
};
pub(crate) const UV_TOOLCHAIN_CONTRACT: ToolchainProviderContract = ToolchainProviderContract {
    toolchain_name: PYTHON_TOOLCHAIN_NAME,
    provider: ToolchainProvider::Uv,
    label: "uv",
    owned_runtime: PYTHON_TOOLCHAIN_NAME,
    provider_specific_fields: UV_PROVIDER_SPECIFIC_FIELDS,
    provider_specific_field_summary: UV_PROVIDER_SPECIFIC_FIELD_SUMMARY,
    requirement_detail_parts_fn: uv_requirement_detail_parts,
    owned_capabilities_fn: uv_owned_capabilities,
    owned_tool_requirements_fn: uv_owned_tool_requirements,
    fulfillment_commands_fn: uv_fulfillment_commands,
    owned_runtime_remediation_command_fn: uv_owned_runtime_remediation_command,
    run_fulfillment_validation_error_fn: uv_run_fulfillment_validation_error,
    managed_surface_probes_fn: uv_managed_surface_probes,
    managed_surface_entries_fn: uv_managed_surface_entries,
    managed_surface_remediation_command_fn: uv_managed_surface_remediation_command,
};
pub(crate) const GO_TOOLCHAIN_CONTRACT: ToolchainProviderContract = ToolchainProviderContract {
    toolchain_name: GO_TOOLCHAIN_NAME,
    provider: ToolchainProvider::Go,
    label: "go",
    owned_runtime: GO_TOOLCHAIN_NAME,
    provider_specific_fields: &[],
    provider_specific_field_summary: "",
    requirement_detail_parts_fn: base_requirement_detail_parts,
    owned_capabilities_fn: go_owned_capabilities,
    owned_tool_requirements_fn: go_owned_tool_requirements,
    fulfillment_commands_fn: go_fulfillment_commands,
    owned_runtime_remediation_command_fn: go_owned_runtime_remediation_command,
    run_fulfillment_validation_error_fn: go_run_fulfillment_validation_error,
    managed_surface_probes_fn: go_managed_surface_probes,
    managed_surface_entries_fn: go_managed_surface_entries,
    managed_surface_remediation_command_fn: go_managed_surface_remediation_command,
};
pub(crate) const RUBY_TOOLCHAIN_CONTRACT: ToolchainProviderContract = ToolchainProviderContract {
    toolchain_name: RUBY_TOOLCHAIN_NAME,
    provider: ToolchainProvider::Ruby,
    label: "ruby",
    owned_runtime: RUBY_TOOLCHAIN_NAME,
    provider_specific_fields: RUBY_PROVIDER_SPECIFIC_FIELDS,
    provider_specific_field_summary: RUBY_PROVIDER_SPECIFIC_FIELD_SUMMARY,
    requirement_detail_parts_fn: ruby_requirement_detail_parts,
    owned_capabilities_fn: ruby_owned_capabilities,
    owned_tool_requirements_fn: ruby_owned_tool_requirements,
    fulfillment_commands_fn: ruby_fulfillment_commands,
    owned_runtime_remediation_command_fn: ruby_owned_runtime_remediation_command,
    run_fulfillment_validation_error_fn: ruby_run_fulfillment_validation_error,
    managed_surface_probes_fn: ruby_managed_surface_probes,
    managed_surface_entries_fn: ruby_managed_surface_entries,
    managed_surface_remediation_command_fn: ruby_managed_surface_remediation_command,
};
pub(crate) const DOTNET_TOOLCHAIN_CONTRACT: ToolchainProviderContract = ToolchainProviderContract {
    toolchain_name: DOTNET_TOOLCHAIN_NAME,
    provider: ToolchainProvider::Dotnet,
    label: "dotnet",
    owned_runtime: DOTNET_TOOLCHAIN_NAME,
    provider_specific_fields: &[],
    provider_specific_field_summary: "",
    requirement_detail_parts_fn: base_requirement_detail_parts,
    owned_capabilities_fn: dotnet_owned_capabilities,
    owned_tool_requirements_fn: dotnet_owned_tool_requirements,
    fulfillment_commands_fn: dotnet_fulfillment_commands,
    owned_runtime_remediation_command_fn: dotnet_owned_runtime_remediation_command,
    run_fulfillment_validation_error_fn: dotnet_run_fulfillment_validation_error,
    managed_surface_probes_fn: dotnet_managed_surface_probes,
    managed_surface_entries_fn: dotnet_managed_surface_entries,
    managed_surface_remediation_command_fn: dotnet_managed_surface_remediation_command,
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
                    if !package_name.trim().is_empty() && !is_shell_safe_package_token(package_name)
                    {
                        errors.push(format!(
                            "toolchain `{name}` package manager `{package_name}` must be a shell-safe package token"
                        ));
                    }
                    if !version.trim().is_empty()
                        && !is_shell_safe_package_version_constraint(version)
                    {
                        errors.push(format!(
                            "toolchain `{name}` package manager `{package_name}` version must be a shell-safe package version constraint"
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
                            && !is_shell_safe_package_token(package_name)
                        {
                            errors.push(format!(
                                "toolchain `{name}` platform `{platform}` package manager `{package_name}` must be a shell-safe package token"
                            ));
                        }
                        if !version.trim().is_empty()
                            && !is_shell_safe_package_version_constraint(version)
                        {
                            errors.push(format!(
                                "toolchain `{name}` platform `{platform}` package manager `{package_name}` version must be a shell-safe package version constraint"
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

    pub(crate) const fn owned_runtime(self) -> &'static str {
        self.owned_runtime
    }

    pub(crate) fn owned_runtime_remediation_command(self, requirement: &str) -> Option<String> {
        (self.owned_runtime_remediation_command_fn)(self, requirement)
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
        errors.extend(self.additional_provider_validation_errors(name, toolchain));
        errors
    }

    fn additional_provider_validation_errors(
        self,
        name: &str,
        toolchain: &ToolchainSpec,
    ) -> Vec<String> {
        match self.provider() {
            ToolchainProvider::Uv => uv_package_manager_validation_errors(name, toolchain),
            ToolchainProvider::Ruby => ruby_package_manager_validation_errors(name, toolchain),
            _ => Vec::new(),
        }
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
    &[
        RUSTUP_TOOLCHAIN_CONTRACT,
        COREPACK_TOOLCHAIN_CONTRACT,
        SDKMAN_TOOLCHAIN_CONTRACT,
        UV_TOOLCHAIN_CONTRACT,
        GO_TOOLCHAIN_CONTRACT,
        RUBY_TOOLCHAIN_CONTRACT,
        DOTNET_TOOLCHAIN_CONTRACT,
    ]
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
    _toolchain: &ToolchainSpec,
) -> Option<ToolchainProviderContract> {
    shipped_toolchain_contract_by_name(name)
}

fn declared_toolchain_fulfillment_contract(
    name: &str,
    toolchain: &ToolchainSpec,
) -> Option<ToolchainProviderContract> {
    if let Some(source) = toolchain.fulfillment_source()
        && let Some(provider) = fulfillment_source_legacy_provider(source)
        && let Some(contract) = toolchain_provider_contract(name, provider)
    {
        return Some(contract);
    }

    declared_toolchain_contract(name, toolchain)
}

pub(crate) fn toolchain_fulfillment_source_label(
    source: ToolchainFulfillmentSource,
) -> &'static str {
    match source {
        ToolchainFulfillmentSource::Rustup => "rustup",
        ToolchainFulfillmentSource::Corepack => "corepack",
        ToolchainFulfillmentSource::Sdkman => "sdkman",
        ToolchainFulfillmentSource::Uv => "uv",
        ToolchainFulfillmentSource::Go => "go",
        ToolchainFulfillmentSource::Ruby => "ruby",
        ToolchainFulfillmentSource::Dotnet => "dotnet",
        ToolchainFulfillmentSource::Mise => "mise",
    }
}

pub(crate) fn fulfillment_source_legacy_provider(
    source: ToolchainFulfillmentSource,
) -> Option<ToolchainProvider> {
    match source {
        ToolchainFulfillmentSource::Rustup => Some(ToolchainProvider::Rustup),
        ToolchainFulfillmentSource::Corepack => Some(ToolchainProvider::Corepack),
        ToolchainFulfillmentSource::Sdkman => Some(ToolchainProvider::Sdkman),
        ToolchainFulfillmentSource::Uv => Some(ToolchainProvider::Uv),
        ToolchainFulfillmentSource::Go => Some(ToolchainProvider::Go),
        ToolchainFulfillmentSource::Ruby => Some(ToolchainProvider::Ruby),
        ToolchainFulfillmentSource::Dotnet => Some(ToolchainProvider::Dotnet),
        ToolchainFulfillmentSource::Mise => None,
    }
}

pub(crate) fn declared_toolchain_source_label(
    name: &str,
    toolchain: &ToolchainSpec,
) -> &'static str {
    if let Some(source) = toolchain.fulfillment_source() {
        return toolchain_fulfillment_source_label(source);
    }
    declared_toolchain_contract(name, toolchain)
        .map(|provider| provider.label())
        .unwrap_or(UNKNOWN_TOOLCHAIN_PROVIDER_LABEL)
}

pub(crate) fn declared_toolchain_fulfillment_commands(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    match toolchain.fulfillment_source() {
        Some(ToolchainFulfillmentSource::Mise) => {
            mise_fulfillment_commands(name, toolchain, target_os)
        }
        _ => declared_toolchain_fulfillment_contract(name, toolchain)
            .map(|provider| provider.fulfillment_commands(toolchain, target_os))
            .unwrap_or_default(),
    }
}

pub(crate) fn declared_toolchain_requirement_clause(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> String {
    let parts = declared_toolchain_requirement_clause_parts(name, toolchain, target_os, None);
    format!(
        "check toolchain `{name}` via `{}` ({})",
        declared_toolchain_source_label(name, toolchain),
        parts.join(", ")
    )
}

pub(crate) fn declared_toolchain_requirement_clause_for_required_tools(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
    required_tools: &BTreeSet<String>,
) -> String {
    let parts = declared_toolchain_requirement_clause_parts(
        name,
        toolchain,
        target_os,
        Some(required_tools),
    );
    format!(
        "check toolchain `{name}` via `{}` ({})",
        declared_toolchain_source_label(name, toolchain),
        parts.join(", ")
    )
}

fn declared_toolchain_requirement_clause_parts(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
    required_tools: Option<&BTreeSet<String>>,
) -> Vec<String> {
    declared_toolchain_contract(name, toolchain)
        .map(|provider| {
            if provider.provider() == ToolchainProvider::Corepack {
                return corepack_requirement_detail_parts_scoped(
                    provider,
                    toolchain,
                    target_os,
                    required_tools,
                );
            }
            provider.requirement_detail_parts(toolchain, target_os)
        })
        .unwrap_or_else(|| vec![format!("version `{}`", toolchain.version_for_os(target_os))])
}

pub(crate) fn declared_toolchain_preview_action_for_required_tools(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
    required_tools: &BTreeSet<String>,
) -> String {
    let base = declared_toolchain_requirement_clause_for_required_tools(
        name,
        toolchain,
        target_os,
        required_tools,
    );
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
    let provider_label = declared_toolchain_source_label(name, toolchain).to_string();
    ToolchainFulfillmentAttemptSummary {
        requirement_clause: declared_toolchain_requirement_clause(name, toolchain, target_os),
        allowance_clause: format!(
            "`toolchains.{name}.fulfillment.mode: run` allowed ota to attempt run-path provisioning via `{provider_label}`"
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
            "`toolchains.{toolchain_name}.fulfillment.mode: run` allowed ota to attempt run-path provisioning via `{UNKNOWN_TOOLCHAIN_PROVIDER_LABEL}`"
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

pub(crate) fn requirement_surface_with_toolchain_owned_tools_for_required_tools(
    contract: &Contract,
    base: &RequirementSurface,
    toolchain_names: &BTreeSet<String>,
    target_os: &str,
    required_tools: Option<&BTreeSet<String>>,
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
            let destination_names =
                selected_requirement_surface_tool_names(&merged, required_tools, &tool_name);
            if destination_names.is_empty() {
                continue;
            }
            for destination_name in destination_names {
                let merged_requirement = merged
                    .tools
                    .get(destination_name.as_str())
                    .map(|base_requirement| base_requirement.merged_with_overlay(&requirement))
                    .unwrap_or_else(|| requirement.clone());
                merged.tools.insert(destination_name, merged_requirement);
            }
        }
        for capability in provider
            .owned_capabilities(toolchain)
            .into_iter()
            .filter(|capability| capability.kind == ToolchainOwnedCapabilityKind::Tool)
        {
            if capability.name == provider.owned_runtime() {
                continue;
            }
            let destination_names =
                selected_requirement_surface_tool_names(&merged, required_tools, &capability.name);
            if destination_names.is_empty() {
                continue;
            }
            for destination_name in destination_names {
                merged.tools.entry(destination_name).or_insert_with(|| {
                    ToolRequirement::Detailed(ToolDetail {
                        version: String::from("*"),
                        required: toolchain.required_for_os(target_os),
                        only_on: toolchain.only_on.clone(),
                        platforms: BTreeMap::new(),
                        acquisition: None,
                    })
                });
            }
        }
    }

    merged
}

fn selected_requirement_surface_tool_names(
    base: &RequirementSurface,
    required_tools: Option<&BTreeSet<String>>,
    tool_name: &str,
) -> Vec<String> {
    let mut names = Vec::new();
    if let Some(required_tools) = required_tools {
        if required_tools.contains(tool_name) {
            names.push(tool_name.to_string());
        }
        for alias in tool_requirement_executable_aliases(tool_name) {
            if required_tools.contains(*alias) {
                names.push((*alias).to_string());
            }
        }
        return names;
    }
    if base.tools.contains_key(tool_name) {
        names.push(tool_name.to_string());
    }
    for alias in tool_requirement_executable_aliases(tool_name) {
        if base.tools.contains_key(*alias) {
            names.push((*alias).to_string());
        }
    }
    if names.is_empty() {
        names.push(tool_name.to_string());
    }
    names
}

fn tool_requirement_executable_aliases(tool_name: &str) -> &'static [&'static str] {
    match tool_name {
        "bundler" => &["bundle"],
        "maven" => &["mvn"],
        _ => &[],
    }
}

pub(crate) fn requirement_surface_with_toolchain_owned_capabilities_for_required_tools(
    contract: &Contract,
    base: &RequirementSurface,
    toolchain_names: &BTreeSet<String>,
    target_os: &str,
    required_tools: Option<&BTreeSet<String>>,
) -> RequirementSurface {
    let with_runtimes =
        requirement_surface_with_toolchain_owned_runtimes(contract, base, toolchain_names);
    requirement_surface_with_toolchain_owned_tools_for_required_tools(
        contract,
        &with_runtimes,
        toolchain_names,
        target_os,
        required_tools,
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
    corepack_requirement_detail_parts_scoped(provider, toolchain, target_os, None)
}

fn corepack_requirement_detail_parts_scoped(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
    required_tools: Option<&BTreeSet<String>>,
) -> Vec<String> {
    let mut parts = base_requirement_detail_parts(provider, toolchain, target_os);
    let mut package_managers = toolchain.package_managers_for_os(target_os);
    if let Some(required_tools) = required_tools {
        package_managers.retain(|name, _| required_tools.contains(name));
    }
    if !package_managers.is_empty() {
        let rendered = package_managers
            .iter()
            .map(|(name, version)| format!("{name}@{version}"))
            .collect::<Vec<_>>();
        parts.push(format!("package managers `{}`", rendered.join("`, `")));
    }
    parts
}

fn ruby_requirement_detail_parts(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<String> {
    let mut parts = base_requirement_detail_parts(provider, toolchain, target_os);
    if let Some(version) = toolchain.package_managers_for_os(target_os).get("bundler") {
        parts.push(format!("bundler `{version}`"));
    }
    parts
}

fn uv_requirement_detail_parts(
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

fn sdkman_owned_capabilities(
    provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
) -> Vec<ToolchainOwnedCapability> {
    vec![
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Runtime,
            name: provider.owned_runtime().to_string(),
        },
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: String::from("java"),
        },
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: String::from("javac"),
        },
    ]
}

fn uv_owned_capabilities(
    provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
) -> Vec<ToolchainOwnedCapability> {
    let mut owned = vec![ToolchainOwnedCapability {
        kind: ToolchainOwnedCapabilityKind::Runtime,
        name: provider.owned_runtime().to_string(),
    }];
    let mut package_managers = BTreeSet::new();
    package_managers.extend(toolchain.package_managers.keys().cloned());
    for detail in toolchain.platforms.values() {
        package_managers.extend(detail.package_managers.keys().cloned());
    }
    for package_manager in package_managers {
        if owned.iter().any(|capability| {
            capability.kind == ToolchainOwnedCapabilityKind::Tool
                && capability.name == package_manager
        }) {
            continue;
        }
        owned.push(ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: package_manager,
        });
    }
    owned
}

fn go_owned_capabilities(
    provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
) -> Vec<ToolchainOwnedCapability> {
    vec![
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Runtime,
            name: provider.owned_runtime().to_string(),
        },
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: String::from("go"),
        },
    ]
}

fn ruby_owned_capabilities(
    provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
) -> Vec<ToolchainOwnedCapability> {
    vec![
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Runtime,
            name: provider.owned_runtime().to_string(),
        },
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: String::from("bundler"),
        },
    ]
}

fn dotnet_owned_capabilities(
    provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
) -> Vec<ToolchainOwnedCapability> {
    vec![
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Runtime,
            name: provider.owned_runtime().to_string(),
        },
        ToolchainOwnedCapability {
            kind: ToolchainOwnedCapabilityKind::Tool,
            name: String::from("dotnet"),
        },
    ]
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

fn sdkman_owned_tool_requirements(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> BTreeMap<String, ToolRequirement> {
    BTreeMap::new()
}

fn uv_owned_tool_requirements(
    _provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> BTreeMap<String, ToolRequirement> {
    let required = toolchain.required_for_os(target_os);
    let mut requirements = BTreeMap::new();
    let package_managers = toolchain.package_managers_for_os(target_os);
    if let Some(uv_version) = package_managers.get("uv").cloned() {
        requirements.insert(
            String::from("uv"),
            ToolRequirement::Detailed(ToolDetail {
                version: uv_version,
                required,
                only_on: toolchain.only_on.clone(),
                platforms: BTreeMap::<String, ToolPlatformDetail>::new(),
                acquisition: None,
            }),
        );
    }
    if let Some(poetry_version) = package_managers.get("poetry").cloned() {
        requirements.insert(
            String::from("poetry"),
            ToolRequirement::Detailed(ToolDetail {
                version: poetry_version,
                required,
                only_on: toolchain.only_on.clone(),
                platforms: BTreeMap::<String, ToolPlatformDetail>::new(),
                acquisition: None,
            }),
        );
    }
    requirements
}

fn go_owned_tool_requirements(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> BTreeMap<String, ToolRequirement> {
    BTreeMap::new()
}

fn ruby_owned_tool_requirements(
    _provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> BTreeMap<String, ToolRequirement> {
    let required = toolchain.required_for_os(target_os);
    let version = toolchain
        .package_managers_for_os(target_os)
        .get("bundler")
        .cloned()
        .unwrap_or_else(|| String::from("*"));
    BTreeMap::from([(
        String::from("bundler"),
        ToolRequirement::Detailed(ToolDetail {
            version,
            required,
            only_on: toolchain.only_on.clone(),
            platforms: BTreeMap::<String, ToolPlatformDetail>::new(),
            acquisition: None,
        }),
    )])
}

fn dotnet_owned_tool_requirements(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> BTreeMap<String, ToolRequirement> {
    BTreeMap::new()
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

fn sdkman_fulfillment_commands(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    Vec::new()
}

fn uv_fulfillment_commands(
    _provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    let python_version = toolchain.version_for_os(target_os).to_string();
    let mut commands = vec![ToolchainCommandSpec {
        program: "uv",
        args: vec![
            String::from("python"),
            String::from("install"),
            python_version.clone(),
        ],
    }];
    if let Some(poetry_version) = toolchain.package_managers_for_os(target_os).get("poetry") {
        commands.push(ToolchainCommandSpec {
            program: "uv",
            args: vec![
                String::from("tool"),
                String::from("install"),
                String::from("--python"),
                python_version,
                python_tool_install_package_spec("poetry", poetry_version),
            ],
        });
    }
    commands
}

fn go_fulfillment_commands(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    Vec::new()
}

fn ruby_fulfillment_commands(
    _provider: ToolchainProviderContract,
    toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    toolchain
        .package_managers_for_os(_target_os)
        .get("bundler")
        .cloned()
        .map(|version| {
            vec![ToolchainCommandSpec {
                program: "ruby",
                args: vec![
                    String::from("-S"),
                    String::from("gem"),
                    String::from("install"),
                    String::from("bundler"),
                    String::from("--no-document"),
                    String::from("--version"),
                    version,
                ],
            }]
        })
        .unwrap_or_default()
}

fn dotnet_fulfillment_commands(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    Vec::new()
}

fn python_tool_install_package_spec(name: &str, version: &str) -> String {
    let trimmed = version.trim();
    if trimmed.is_empty() || trimmed == "*" {
        return name.to_string();
    }
    if trimmed
        .chars()
        .next()
        .is_some_and(|character| character.is_ascii_digit())
    {
        return format!("{name}=={trimmed}");
    }
    format!("{name}{trimmed}")
}

fn mise_fulfillment_commands(
    name: &str,
    toolchain: &ToolchainSpec,
    target_os: &str,
) -> Vec<ToolchainCommandSpec> {
    let mut commands = vec![ToolchainCommandSpec {
        program: "mise",
        args: vec![
            String::from("install"),
            format!("{name}@{}", toolchain.version_for_os(target_os)),
        ],
    }];

    commands.extend(
        toolchain
            .package_managers_for_os(target_os)
            .into_iter()
            .map(|(package_name, version)| ToolchainCommandSpec {
                program: "mise",
                args: vec![String::from("install"), format!("{package_name}@{version}")],
            }),
    );

    commands
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

fn sdkman_owned_runtime_remediation_command(
    _provider: ToolchainProviderContract,
    requirement: &str,
) -> Option<String> {
    Some(format!("sdk install java {requirement}"))
}

fn uv_owned_runtime_remediation_command(
    _provider: ToolchainProviderContract,
    requirement: &str,
) -> Option<String> {
    Some(format!("uv python install {requirement}"))
}

fn go_owned_runtime_remediation_command(
    _provider: ToolchainProviderContract,
    _requirement: &str,
) -> Option<String> {
    None
}

fn ruby_owned_runtime_remediation_command(
    _provider: ToolchainProviderContract,
    _requirement: &str,
) -> Option<String> {
    None
}

fn dotnet_owned_runtime_remediation_command(
    _provider: ToolchainProviderContract,
    requirement: &str,
) -> Option<String> {
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

fn parse_semver_requirement(value: &str) -> Option<VersionReq> {
    let trimmed = value.trim();
    VersionReq::parse(trimmed).ok().or_else(|| {
        normalize_short_version_requirement(trimmed)
            .and_then(|normalized| VersionReq::parse(&normalized).ok())
    })
}

fn normalize_short_version_requirement(value: &str) -> Option<String> {
    if value.is_empty() || value == "*" {
        return None;
    }
    if value
        .chars()
        .all(|character| character.is_ascii_digit() || character == '.')
    {
        let segments = value
            .split('.')
            .filter(|segment| !segment.is_empty())
            .count();
        return match segments {
            1 => Some(format!(">={value}.0.0,<{value}.999.999")),
            2 => Some(format!(">={value}.0,<{value}.999")),
            _ => None,
        };
    }
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
    _name: &str,
    _toolchain: &ToolchainSpec,
) -> Option<String> {
    None
}

fn sdkman_run_fulfillment_validation_error(
    _provider: ToolchainProviderContract,
    _name: &str,
    _toolchain: &ToolchainSpec,
) -> Option<String> {
    None
}

fn uv_run_fulfillment_validation_error(
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
        && !trimmed.contains('*')
        && !trimmed.contains(',');
    (!installable).then(|| {
        format!(
            "toolchain `{name}` uses `provider: uv` with `fulfillment: run`, so `toolchains.{name}.version` must be an installable uv Python reference like `3.12`, `3.12.10`, or `3.13`"
        )
    })
}

fn go_run_fulfillment_validation_error(
    _provider: ToolchainProviderContract,
    _name: &str,
    _toolchain: &ToolchainSpec,
) -> Option<String> {
    None
}

fn ruby_run_fulfillment_validation_error(
    _provider: ToolchainProviderContract,
    _name: &str,
    _toolchain: &ToolchainSpec,
) -> Option<String> {
    None
}

fn dotnet_run_fulfillment_validation_error(
    _provider: ToolchainProviderContract,
    _name: &str,
    _toolchain: &ToolchainSpec,
) -> Option<String> {
    None
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

fn sdkman_managed_surface_probes(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainManagedSurfaceProbe> {
    Vec::new()
}

fn uv_managed_surface_probes(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainManagedSurfaceProbe> {
    Vec::new()
}

fn go_managed_surface_probes(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainManagedSurfaceProbe> {
    Vec::new()
}

fn ruby_managed_surface_probes(
    _provider: ToolchainProviderContract,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<ToolchainManagedSurfaceProbe> {
    Vec::new()
}

fn dotnet_managed_surface_probes(
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

fn sdkman_managed_surface_entries(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<String> {
    Vec::new()
}

fn uv_managed_surface_entries(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<String> {
    Vec::new()
}

fn go_managed_surface_entries(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<String> {
    Vec::new()
}

fn ruby_managed_surface_entries(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<String> {
    Vec::new()
}

fn dotnet_managed_surface_entries(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _toolchain: &ToolchainSpec,
    _target_os: &str,
) -> Vec<String> {
    Vec::new()
}

fn is_shell_safe_package_token(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.chars().all(|ch| {
            ch.is_ascii_alphanumeric() || matches!(ch, '@' | '/' | '.' | '_' | '-' | '+' | '~')
        })
}

fn is_shell_safe_package_version_constraint(value: &str) -> bool {
    let trimmed = value.trim();
    !trimmed.is_empty()
        && trimmed == value
        && trimmed.chars().all(|ch| {
            ch.is_ascii_alphanumeric()
                || matches!(
                    ch,
                    '.' | '_' | '-' | '+' | '~' | '^' | '<' | '>' | '=' | '!' | ',' | '*'
                )
        })
}

fn ruby_package_manager_validation_errors(name: &str, toolchain: &ToolchainSpec) -> Vec<String> {
    let mut errors = Vec::new();
    for package_name in toolchain.package_managers.keys() {
        if package_name != "bundler" {
            errors.push(format!(
                "toolchain `{name}` with `provider: ruby` must only declare `bundler` under `package_managers`; found `{package_name}`"
            ));
        }
    }
    for (platform, detail) in &toolchain.platforms {
        for package_name in detail.package_managers.keys() {
            if package_name != "bundler" {
                errors.push(format!(
                    "toolchain `{name}` platform `{platform}` with `provider: ruby` must only declare `bundler` under `package_managers`; found `{package_name}`"
                ));
            }
        }
    }
    errors
}

fn uv_package_manager_validation_errors(name: &str, toolchain: &ToolchainSpec) -> Vec<String> {
    let mut errors = Vec::new();
    for package_name in toolchain.package_managers.keys() {
        if package_name != "uv" && package_name != "poetry" {
            errors.push(format!(
                "toolchain `{name}` with `provider: uv` must only declare `uv` or `poetry` under `package_managers`; found `{package_name}`"
            ));
        }
    }
    for (platform, detail) in &toolchain.platforms {
        for package_name in detail.package_managers.keys() {
            if package_name != "uv" && package_name != "poetry" {
                errors.push(format!(
                    "toolchain `{name}` platform `{platform}` with `provider: uv` must only declare `uv` or `poetry` under `package_managers`; found `{package_name}`"
                ));
            }
        }
    }
    errors
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

fn sdkman_managed_surface_remediation_command(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _entry: &str,
) -> Option<String> {
    None
}

fn uv_managed_surface_remediation_command(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _entry: &str,
) -> Option<String> {
    None
}

fn go_managed_surface_remediation_command(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _entry: &str,
) -> Option<String> {
    None
}

fn ruby_managed_surface_remediation_command(
    _provider: ToolchainProviderContract,
    _kind: ToolchainManagedSurfaceKind,
    _entry: &str,
) -> Option<String> {
    None
}

fn dotnet_managed_surface_remediation_command(
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
    let _ = ecosystem;
    None
}

pub(crate) fn unsupported_toolchain_opportunity_ecosystems() -> &'static [&'static str] {
    UNSUPPORTED_TOOLCHAIN_OPPORTUNITY_ECOSYSTEMS
}

pub(crate) fn toolchain_repo_signals(contract_root: &Path, ecosystem: &str) -> Vec<&'static str> {
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
            if contract_root.join(".java-version").is_file() {
                signals.push(".java-version");
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
        GO_TOOLCHAIN_NAME => {
            let mut signals = Vec::new();
            if contract_root.join("go.mod").is_file() {
                signals.push("go.mod");
            }
            if contract_root.join("go.work").is_file() {
                signals.push("go.work");
            }
            if tool_versions_entry(contract_root, &[GO_TOOLCHAIN_NAME]).is_some() {
                signals.push(".tool-versions");
            }
            signals
        }
        RUBY_TOOLCHAIN_NAME => {
            let mut signals = Vec::new();
            if contract_root.join("Gemfile").is_file() {
                signals.push("Gemfile");
            }
            if contract_root.join("Gemfile.lock").is_file() {
                signals.push("Gemfile.lock");
            }
            if contract_root.join(".ruby-version").is_file() {
                signals.push(".ruby-version");
            }
            if contract_root.join("Rakefile").is_file() {
                signals.push("Rakefile");
            }
            if tool_versions_entry(contract_root, &[RUBY_TOOLCHAIN_NAME]).is_some() {
                signals.push(".tool-versions");
            }
            signals
        }
        DOTNET_TOOLCHAIN_NAME => {
            let mut signals = Vec::new();
            if contract_root.join("global.json").is_file() {
                signals.push("global.json");
            }
            if contract_root.join("Directory.Build.props").is_file() {
                signals.push("Directory.Build.props");
            }
            if contract_root.join("Directory.Build.targets").is_file() {
                signals.push("Directory.Build.targets");
            }
            if contract_root.join("dotnet-tools.json").is_file() {
                signals.push("dotnet-tools.json");
            }
            if contract_root.join("dotnet.json").is_file() {
                signals.push("dotnet.json");
            }
            if tool_versions_entry(contract_root, &[DOTNET_TOOLCHAIN_NAME]).is_some() {
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
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;

    fn contract(yaml: &str) -> Contract {
        parse_contract_str(Path::new("./ota.yaml"), yaml).unwrap()
    }

    #[test]
    fn unsupported_toolchain_opportunity_ecosystems_match_declared_contexts() {
        let ecosystems = unsupported_toolchain_opportunity_ecosystems();
        assert!(ecosystems.is_empty());
    }

    #[test]
    fn go_and_ruby_repo_signal_detection_is_explicit() {
        let fixture = TempDir::new().expect("tempdir");
        let root = fixture.path();
        fs::write(
            root.join("go.mod"),
            "module example.com/demo\n\ngo 1.24.0\n",
        )
        .unwrap();
        fs::write(
            root.join("Gemfile"),
            "source \"https://rubygems.org\"\nruby \"3.3.2\"\n",
        )
        .unwrap();
        fs::write(root.join(".ruby-version"), "3.3.2\n").unwrap();

        let go_signals = toolchain_repo_signals(root, GO_TOOLCHAIN_NAME);
        let ruby_signals = toolchain_repo_signals(root, RUBY_TOOLCHAIN_NAME);

        assert_eq!(go_signals, vec!["go.mod"]);
        assert_eq!(ruby_signals, vec!["Gemfile", ".ruby-version"]);
    }

    #[test]
    fn dotnet_repo_signal_detection_is_explicit() {
        let fixture = TempDir::new().expect("tempdir");
        let root = fixture.path();
        fs::write(
            root.join("global.json"),
            "{\n  \"sdk\": { \"version\": \"9.0.100\" }\n}\n",
        )
        .unwrap();

        let dotnet_signals = toolchain_repo_signals(root, DOTNET_TOOLCHAIN_NAME);
        assert_eq!(dotnet_signals, vec!["global.json"]);
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
    fn corepack_contract_supports_policy_governed_run_fulfillment_hooks() {
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
                .is_none()
        );
        assert_eq!(provider.owned_runtime_remediation_command("22"), None);
    }

    #[test]
    fn ruby_contract_projects_bundler_requirement_when_declared() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  ruby:
    provider: ruby
    version: "3.3.11"
    package_managers:
      bundler: "2.5"
"#,
        );
        let toolchain = contract.toolchains.get("ruby").unwrap();
        let provider = declared_toolchain_contract("ruby", toolchain).unwrap();

        assert_eq!(
            provider.provider_specific_field_summary(),
            RUBY_PROVIDER_SPECIFIC_FIELD_SUMMARY
        );
        assert!(
            provider
                .requirement_detail_parts(toolchain, "linux")
                .iter()
                .any(|part| part.contains("bundler `2.5`"))
        );
        let tool_requirements = provider.owned_tool_requirements(toolchain, "linux");
        assert_eq!(
            tool_requirements
                .get("bundler")
                .expect("projected bundler requirement")
                .version(),
            "2.5"
        );
        assert_eq!(
            provider.run_fulfillment_validation_error("ruby", toolchain),
            None
        );
        assert_eq!(provider.owned_runtime_remediation_command("3.3.11"), None);
    }

    #[test]
    fn uv_contract_projects_uv_requirement_when_declared() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  python:
    provider: uv
    version: "3.12"
    package_managers:
      uv: ">=0.11.8"
"#,
        );
        let toolchain = contract.toolchains.get("python").unwrap();
        let provider = declared_toolchain_contract("python", toolchain).unwrap();

        assert_eq!(
            provider.provider_specific_field_summary(),
            UV_PROVIDER_SPECIFIC_FIELD_SUMMARY
        );
        assert!(
            provider
                .requirement_detail_parts(toolchain, "linux")
                .iter()
                .any(|part| part.contains("package managers `uv@>=0.11.8`"))
        );
        let tool_requirements = provider.owned_tool_requirements(toolchain, "linux");
        assert_eq!(
            tool_requirements
                .get("uv")
                .expect("projected uv requirement")
                .version(),
            ">=0.11.8"
        );
    }

    #[test]
    fn uv_contract_projects_poetry_requirement_when_declared() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  python:
    provider: uv
    version: "3.12"
    package_managers:
      poetry: ">=1.8"
"#,
        );
        let toolchain = contract.toolchains.get("python").unwrap();
        let provider = declared_toolchain_contract("python", toolchain).unwrap();

        assert!(
            provider
                .requirement_detail_parts(toolchain, "linux")
                .iter()
                .any(|part| part.contains("poetry@>=1.8"))
        );
        let tool_requirements = provider.owned_tool_requirements(toolchain, "linux");
        assert!(
            !tool_requirements.contains_key("uv"),
            "poetry-only python toolchain should not project uv as a required host tool"
        );
        assert_eq!(
            tool_requirements
                .get("poetry")
                .expect("projected poetry requirement")
                .version(),
            ">=1.8"
        );
        assert!(
            provider
                .owned_capabilities(toolchain)
                .iter()
                .all(|capability| capability.name != "uv"),
            "poetry-only python toolchain should not own uv unless the contract declares it"
        );
    }

    #[test]
    fn poetry_only_python_toolchain_requirement_surface_does_not_inject_uv() {
        let contract = contract(
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
"#,
        );
        let toolchain_names = BTreeSet::from([String::from("python")]);
        let requirement_surface = requirement_surface_with_toolchain_owned_tools_for_required_tools(
            &contract,
            &RequirementSurface::default(),
            &toolchain_names,
            "linux",
            None,
        );

        assert!(
            !requirement_surface.tools.contains_key("uv"),
            "poetry-only python toolchain should not inject uv into the effective tool surface"
        );
        assert!(
            requirement_surface.tools.contains_key("poetry"),
            "poetry-only python toolchain should still project poetry into the effective tool surface"
        );
    }

    #[test]
    fn uv_contract_rejects_non_uv_package_manager_keys() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  python:
    provider: uv
    version: "3.12"
    package_managers:
      pip: "25"
"#,
        );
        let toolchain = contract.toolchains.get("python").unwrap();
        let provider = declared_toolchain_contract("python", toolchain).unwrap();
        assert_eq!(
            provider.provider_specific_validation_errors("python", toolchain),
            vec![String::from(
                "toolchain `python` with `provider: uv` must only declare `uv` or `poetry` under `package_managers`; found `pip`"
            )]
        );
    }

    #[test]
    fn sdkman_contract_owns_java_surface_and_allows_run_path_fulfillment() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  java:
    provider: sdkman
    version: "21.0.2-tem"
"#,
        );
        let toolchain = contract.toolchains.get("java").unwrap();
        let provider = declared_toolchain_contract("java", toolchain).unwrap();

        assert_eq!(provider.label(), "sdkman");
        assert!(
            provider
                .owned_capabilities(toolchain)
                .iter()
                .any(
                    |capability| capability.kind == ToolchainOwnedCapabilityKind::Tool
                        && capability.name == "javac"
                )
        );
        assert!(provider.fulfillment_commands(toolchain, "linux").is_empty());
        assert!(
            provider
                .run_fulfillment_validation_error("java", toolchain)
                .is_none()
        );
        assert_eq!(
            provider.owned_runtime_remediation_command("21.0.2-tem"),
            Some(String::from("sdk install java 21.0.2-tem"))
        );
    }

    #[test]
    fn dotnet_contract_owns_dotnet_surface_and_allows_run_path_fulfillment() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  dotnet:
    provider: dotnet
    version: "9.0"
"#,
        );
        let toolchain = contract.toolchains.get("dotnet").unwrap();
        let provider = declared_toolchain_contract("dotnet", toolchain).unwrap();

        assert_eq!(provider.label(), "dotnet");
        assert!(
            provider
                .owned_capabilities(toolchain)
                .iter()
                .any(
                    |capability| capability.kind == ToolchainOwnedCapabilityKind::Runtime
                        && capability.name == "dotnet"
                )
        );
        assert!(
            provider
                .owned_capabilities(toolchain)
                .iter()
                .any(
                    |capability| capability.kind == ToolchainOwnedCapabilityKind::Tool
                        && capability.name == "dotnet"
                )
        );
        assert!(provider.fulfillment_commands(toolchain, "linux").is_empty());
        assert_eq!(
            provider.run_fulfillment_validation_error("dotnet", toolchain),
            None
        );
        let expected = if cfg!(windows) {
            "powershell -ExecutionPolicy Bypass -Command \"iwr https://dot.net/v1/dotnet-install.ps1 -OutFile dotnet-install.ps1; ./dotnet-install.ps1 -Channel 9.0\""
        } else {
            "curl -fsSL https://dot.net/v1/dotnet-install.sh -o dotnet-install.sh && bash dotnet-install.sh --channel 9.0"
        };
        assert_eq!(
            provider.owned_runtime_remediation_command("9.0"),
            Some(String::from(expected))
        );
    }

    #[test]
    fn dotnet_runtime_remediation_uses_channel_for_semver_ranges() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  dotnet:
    provider: dotnet
    version: ">=9.0,<10.0"
"#,
        );
        let toolchain = contract.toolchains.get("dotnet").unwrap();
        let provider =
            declared_toolchain_contract("dotnet", toolchain).expect("dotnet provider contract");

        let expected = if cfg!(windows) {
            "powershell -ExecutionPolicy Bypass -Command \"iwr https://dot.net/v1/dotnet-install.ps1 -OutFile dotnet-install.ps1; ./dotnet-install.ps1 -Channel 9.0\""
        } else {
            "curl -fsSL https://dot.net/v1/dotnet-install.sh -o dotnet-install.sh && bash dotnet-install.sh --channel 9.0"
        };

        assert_eq!(
            provider.owned_runtime_remediation_command(">=9.0,<10.0"),
            Some(String::from(expected))
        );
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
            toolchain_provider_contract("java", ToolchainProvider::Sdkman)
                .unwrap()
                .label(),
            "sdkman"
        );
        assert_eq!(
            toolchain_provider_contract("python", ToolchainProvider::Uv)
                .unwrap()
                .label(),
            "uv"
        );
        assert_eq!(
            toolchain_provider_contract("go", ToolchainProvider::Go)
                .unwrap()
                .label(),
            "go"
        );
        assert_eq!(
            toolchain_provider_contract("ruby", ToolchainProvider::Ruby)
                .unwrap()
                .label(),
            "ruby"
        );
        assert_eq!(
            toolchain_provider_contract("dotnet", ToolchainProvider::Dotnet)
                .unwrap()
                .label(),
            "dotnet"
        );
        assert_eq!(
            shipped_toolchain_contract_by_label("rustup")
                .unwrap()
                .toolchain_name(),
            "rust"
        );
        assert_eq!(
            shipped_toolchain_contract_by_label("sdkman")
                .unwrap()
                .toolchain_name(),
            "java"
        );
        assert_eq!(
            toolchain_provider_label(ToolchainProvider::Rustup),
            "rustup"
        );
        assert_eq!(
            toolchain_provider_label(ToolchainProvider::Sdkman),
            "sdkman"
        );
        assert_eq!(toolchain_provider_label(ToolchainProvider::Go), "go");
        assert_eq!(toolchain_provider_label(ToolchainProvider::Ruby), "ruby");
        assert_eq!(
            toolchain_provider_label(ToolchainProvider::Dotnet),
            "dotnet"
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
                "`toolchains.rust.fulfillment.mode: run` allowed ota to attempt run-path provisioning via `rustup`"
            ),
            "{summary:?}"
        );
    }

    #[test]
    fn declared_toolchain_fulfillment_commands_support_mise_source() {
        let contract = contract(
            r#"
version: 1
project:
  name: ota
toolchains:
  node:
    version: "24.15.0"
    package_managers:
      pnpm: "10.33.4"
    fulfillment:
      source: mise
      mode: run
"#,
        );
        let toolchain = contract.toolchains.get("node").unwrap();
        let commands = declared_toolchain_fulfillment_commands("node", toolchain, "linux");

        assert_eq!(commands[0].program, "mise");
        assert_eq!(
            commands[0].args,
            vec![String::from("install"), String::from("node@24.15.0")]
        );
        assert_eq!(
            commands[1].args,
            vec![String::from("install"), String::from("pnpm@10.33.4")]
        );
    }

    #[test]
    fn declared_toolchain_fulfillment_commands_support_ruby_source() {
        let contract = contract(
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
"#,
        );
        let toolchain = contract.toolchains.get("ruby").unwrap();
        let commands = declared_toolchain_fulfillment_commands("ruby", toolchain, "linux");

        assert_eq!(commands.len(), 1);
        assert_eq!(commands[0].program, "ruby");
        assert_eq!(
            commands[0].args,
            vec![
                String::from("-S"),
                String::from("gem"),
                String::from("install"),
                String::from("bundler"),
                String::from("--no-document"),
                String::from("--version"),
                String::from("2.5.3"),
            ]
        );
    }

    #[test]
    fn declared_toolchain_fulfillment_commands_support_poetry_under_uv_source() {
        let contract = contract(
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
"#,
        );
        let toolchain = contract.toolchains.get("python").unwrap();
        let commands = declared_toolchain_fulfillment_commands("python", toolchain, "linux");

        assert_eq!(commands.len(), 2);
        assert_eq!(commands[0].program, "uv");
        assert_eq!(
            commands[0].args,
            vec![
                String::from("python"),
                String::from("install"),
                String::from("3.12"),
            ]
        );
        assert_eq!(commands[1].program, "uv");
        assert_eq!(
            commands[1].args,
            vec![
                String::from("tool"),
                String::from("install"),
                String::from("--python"),
                String::from("3.12"),
                String::from("poetry>=1.8"),
            ]
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
