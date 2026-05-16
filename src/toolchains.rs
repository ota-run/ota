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

use crate::schema::{ToolchainProvider, ToolchainSpec};

pub(crate) const SHARED_TOOLCHAIN_CORE_SUMMARY: &str =
    "`provider`, `version`, `fulfillment`, `required`, `only_on`, and `platforms.<os>.version`";

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
pub(crate) struct ToolchainProviderDefinition {
    provider: ToolchainProvider,
    label: &'static str,
    primary_executable: &'static str,
}

impl ToolchainProviderDefinition {
    pub(crate) const fn label(self) -> &'static str {
        self.label
    }

    pub(crate) const fn primary_executable(self) -> &'static str {
        self.primary_executable
    }

    pub(crate) const fn provider_hint(self) -> &'static str {
        self.label
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
                        name: String::from("rust"),
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
        }
    }
}

pub(crate) fn declared_toolchain_provider(
    name: &str,
    toolchain: &ToolchainSpec,
) -> Option<ToolchainProviderDefinition> {
    match (name, toolchain.provider) {
        ("rust", ToolchainProvider::Rustup) => Some(ToolchainProviderDefinition {
            provider: ToolchainProvider::Rustup,
            label: "rustup",
            primary_executable: "rustc",
        }),
        _ => None,
    }
}

pub(crate) fn declared_rustup_specific_fields(toolchain: &ToolchainSpec) -> Vec<String> {
    let mut fields = Vec::new();
    if toolchain
        .profile
        .as_deref()
        .is_some_and(|profile| !profile.trim().is_empty())
    {
        fields.push(String::from("profile"));
    }
    if !toolchain.components.is_empty() {
        fields.push(String::from("components"));
    }
    if !toolchain.targets.is_empty() {
        fields.push(String::from("targets"));
    }
    for (platform, detail) in &toolchain.platforms {
        if detail
            .profile
            .as_deref()
            .is_some_and(|profile| !profile.trim().is_empty())
        {
            fields.push(format!("platforms.{platform}.profile"));
        }
        if !detail.components.is_empty() {
            fields.push(format!("platforms.{platform}.components"));
        }
        if !detail.targets.is_empty() {
            fields.push(format!("platforms.{platform}.targets"));
        }
    }
    fields
}

fn rustup_component_tool_name(component: &str) -> Option<&'static str> {
    match component {
        "rustfmt" => Some("rustfmt"),
        "clippy" => Some("clippy"),
        _ => None,
    }
}
