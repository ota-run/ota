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

use std::collections::BTreeSet;

use crate::schema::{
    Contract, RequirementSurface, RuntimeDetail, RuntimePlatformDetail, RuntimeRequirement,
    ToolchainFulfillmentMode, ToolchainProvider, ToolchainSpec,
};

pub(crate) const SHARED_TOOLCHAIN_CORE_SUMMARY: &str =
    "`provider`, `version`, `fulfillment`, `required`, `only_on`, and `platforms.<os>.version`";
pub(crate) const RUSTUP_TOOLCHAIN_NAME: &str = "rust";
pub(crate) const COREPACK_TOOLCHAIN_NAME: &str = "node";
const RUSTUP_PROVIDER_SPECIFIC_FIELDS: &[ToolchainProviderSpecificField] = &[
    ToolchainProviderSpecificField::Profile,
    ToolchainProviderSpecificField::Components,
    ToolchainProviderSpecificField::Targets,
];
const COREPACK_PROVIDER_SPECIFIC_FIELDS: &[ToolchainProviderSpecificField] = &[];
pub(crate) const RUSTUP_TOOLCHAIN_CONTRACT: ToolchainProviderContract = ToolchainProviderContract {
    toolchain_name: RUSTUP_TOOLCHAIN_NAME,
    provider: ToolchainProvider::Rustup,
    label: "rustup",
    primary_executable: "rustc",
    owned_runtime: "rust",
    provider_specific_fields: RUSTUP_PROVIDER_SPECIFIC_FIELDS,
};
pub(crate) const COREPACK_TOOLCHAIN_CONTRACT: ToolchainProviderContract =
    ToolchainProviderContract {
        toolchain_name: COREPACK_TOOLCHAIN_NAME,
        provider: ToolchainProvider::Corepack,
        label: "corepack",
        primary_executable: "node",
        owned_runtime: "node",
        provider_specific_fields: COREPACK_PROVIDER_SPECIFIC_FIELDS,
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
pub(crate) enum ToolchainProviderSpecificField {
    Profile,
    Components,
    Targets,
}

impl ToolchainProviderSpecificField {
    const fn name(self) -> &'static str {
        match self {
            Self::Profile => "profile",
            Self::Components => "components",
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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ToolchainProviderContract {
    toolchain_name: &'static str,
    provider: ToolchainProvider,
    label: &'static str,
    primary_executable: &'static str,
    owned_runtime: &'static str,
    provider_specific_fields: &'static [ToolchainProviderSpecificField],
}

impl ToolchainProviderContract {
    pub(crate) const fn toolchain_name(self) -> &'static str {
        self.toolchain_name
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

    pub(crate) const fn provider_hint(self) -> &'static str {
        self.label
    }

    pub(crate) const fn shared_core_summary(self) -> &'static str {
        SHARED_TOOLCHAIN_CORE_SUMMARY
    }

    pub(crate) const fn provider_specific_field_summary(self) -> &'static str {
        if self.provider_specific_fields.is_empty() {
            "no provider-specific fields; current Corepack-backed Node toolchains use only the shared provider-agnostic fields"
        } else {
            "`profile`, `components`, `targets`, and their `platforms.<os>.*` overrides"
        }
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
        let mut parts = vec![format!("owns runtime `{}`", self.owned_runtime())];
        parts.push(format!("version `{}`", toolchain.version_for_os(target_os)));
        match self.provider {
            ToolchainProvider::Rustup => {
                let components = toolchain.components_for_os(target_os);
                if !components.is_empty() {
                    parts.push(format!("components `{}`", components.join("`, `")));
                }
                let targets = toolchain.targets_for_os(target_os);
                if !targets.is_empty() {
                    parts.push(format!("targets `{}`", targets.join("`, `")));
                }
            }
            ToolchainProvider::Corepack => {}
        }
        parts
    }

    pub(crate) fn owned_capabilities(
        self,
        toolchain: &ToolchainSpec,
    ) -> Vec<ToolchainOwnedCapability> {
        match self.provider {
            ToolchainProvider::Rustup => {
                let mut owned = vec![
                    ToolchainOwnedCapability {
                        kind: ToolchainOwnedCapabilityKind::Runtime,
                        name: self.owned_runtime().to_string(),
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
                        capability.kind == ToolchainOwnedCapabilityKind::Tool
                            && capability.name == tool_name
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
            ToolchainProvider::Corepack => vec![
                ToolchainOwnedCapability {
                    kind: ToolchainOwnedCapabilityKind::Runtime,
                    name: self.owned_runtime().to_string(),
                },
                ToolchainOwnedCapability {
                    kind: ToolchainOwnedCapabilityKind::Tool,
                    name: self.owned_runtime().to_string(),
                },
            ],
        }
    }

    pub(crate) fn fulfillment_commands(
        self,
        toolchain: &ToolchainSpec,
        target_os: &str,
    ) -> Vec<ToolchainCommandSpec> {
        match self.provider {
            ToolchainProvider::Rustup => {
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
            ToolchainProvider::Corepack => Vec::new(),
        }
    }

    pub(crate) fn run_fulfillment_validation_error(
        self,
        name: &str,
        toolchain: &ToolchainSpec,
    ) -> Option<String> {
        match self.provider {
            ToolchainProvider::Rustup => {
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
            ToolchainProvider::Corepack => Some(format!(
                "toolchain `{name}` uses `provider: corepack` with `fulfillment: run`, but Corepack-backed Node toolchains are currently check-only; keep `toolchains.{name}.fulfillment: none` and use `tools.<package-manager>.acquisition.provider: corepack` for package-manager activation"
            )),
        }
    }

    pub(crate) fn managed_surface_probes(
        self,
        toolchain: &ToolchainSpec,
        target_os: &str,
    ) -> Vec<ToolchainManagedSurfaceProbe> {
        match self.provider {
            ToolchainProvider::Rustup => {
                let mut probes = Vec::new();
                let components = toolchain.components_for_os(target_os);
                if !components.is_empty() {
                    probes.push(ToolchainManagedSurfaceProbe {
                        kind: ToolchainManagedSurfaceKind::Component,
                        command: format!("{} component list --installed", self.label),
                        required_entries: components,
                    });
                }
                let targets = toolchain.targets_for_os(target_os);
                if !targets.is_empty() {
                    probes.push(ToolchainManagedSurfaceProbe {
                        kind: ToolchainManagedSurfaceKind::Target,
                        command: format!("{} target list --installed", self.label),
                        required_entries: targets,
                    });
                }
                probes
            }
            ToolchainProvider::Corepack => Vec::new(),
        }
    }

    pub(crate) fn managed_surface_remediation_command(
        self,
        kind: ToolchainManagedSurfaceKind,
        entry: &str,
    ) -> String {
        match self.provider {
            ToolchainProvider::Rustup => match kind {
                ToolchainManagedSurfaceKind::Component => {
                    format!("rustup component add {entry}")
                }
                ToolchainManagedSurfaceKind::Target => format!("rustup target add {entry}"),
            },
            ToolchainProvider::Corepack => String::new(),
        }
    }
}

pub(crate) fn toolchain_provider_contract(
    name: &str,
    provider: ToolchainProvider,
) -> Option<ToolchainProviderContract> {
    match (name, provider) {
        (RUSTUP_TOOLCHAIN_NAME, ToolchainProvider::Rustup) => Some(RUSTUP_TOOLCHAIN_CONTRACT),
        (COREPACK_TOOLCHAIN_NAME, ToolchainProvider::Corepack) => Some(COREPACK_TOOLCHAIN_CONTRACT),
        _ => None,
    }
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
        .find(|contract| contract.provider == provider)
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
        .unwrap_or("toolchain provider")
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

pub(crate) fn declared_known_provider_specific_fields(toolchain: &ToolchainSpec) -> Vec<String> {
    known_provider_specific_fields()
        .iter()
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

pub(crate) const fn toolchain_provider_label(provider: ToolchainProvider) -> &'static str {
    match provider {
        ToolchainProvider::Rustup => "rustup",
        ToolchainProvider::Corepack => "corepack",
    }
}

fn owned_runtime_requirement_from_toolchain(toolchain: &ToolchainSpec) -> RuntimeRequirement {
    RuntimeRequirement::Detailed(RuntimeDetail {
        version: toolchain.version.clone(),
        required: toolchain.required,
        only_on: toolchain.only_on.clone(),
        provider: None,
        distribution: None,
        platforms: toolchain
            .platforms
            .iter()
            .map(|(platform, detail)| {
                (
                    platform.clone(),
                    RuntimePlatformDetail {
                        version: detail.version.clone(),
                        provider: None,
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
        let runtime_requirement = owned_runtime_requirement_from_toolchain(toolchain);
        let merged_requirement = merged
            .runtimes
            .get(runtime_name.as_str())
            .map(|base_requirement| base_requirement.merged_with_overlay(&runtime_requirement))
            .unwrap_or(runtime_requirement);
        merged.runtimes.insert(runtime_name, merged_requirement);
    }

    merged
}

fn rustup_component_tool_name(component: &str) -> Option<&'static str> {
    match component {
        "rustfmt" => Some("rustfmt"),
        "clippy" => Some("clippy"),
        _ => None,
    }
}

const fn known_provider_specific_fields() -> &'static [ToolchainProviderSpecificField] {
    &[
        ToolchainProviderSpecificField::Profile,
        ToolchainProviderSpecificField::Components,
        ToolchainProviderSpecificField::Targets,
    ]
}
