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

use semver::Version;
use serde_json::{Value as JsonValue, json};
use serde_yaml::Value;

use crate::schema::{AgentPosture, Contract};

pub(crate) const CONTRACT_SCHEMA_VERSION: u64 = 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct ContractCapabilitySpec {
    pub id: &'static str,
    pub introduced_in: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) struct BuildIdentity {
    pub source_build: bool,
    pub commit: Option<&'static str>,
    pub dirty: bool,
}

const CONTRACT_CAPABILITY_SPECS: &[ContractCapabilitySpec] = &[
    ContractCapabilitySpec {
        id: "toolchains",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "execution.contexts.only_on",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "metadata.ota.minimum_version",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.writes",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.network",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.external_state",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "agent.posture",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "agent.exceptions.sensitive_writes",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "native_prerequisites.visual_studio",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "native_prerequisites.requires",
        introduced_in: "1.6.15",
    },
];

pub(crate) fn contract_capabilities_json() -> JsonValue {
    JsonValue::Array(
        CONTRACT_CAPABILITY_SPECS
            .iter()
            .map(|capability| {
                json!({
                    "id": capability.id,
                    "introduced_in": capability.introduced_in,
                })
            })
            .collect(),
    )
}

pub(crate) fn detect_declared_contract_capabilities(
    document: &Value,
) -> Vec<&'static ContractCapabilitySpec> {
    detect_contract_capabilities_with(|capability| {
        capability_present_in_document(capability, document)
    })
}

pub(crate) fn unsupported_declared_contract_capabilities(
    document: &Value,
    current: &Version,
) -> Vec<&'static ContractCapabilitySpec> {
    detect_declared_contract_capabilities(document)
        .into_iter()
        .filter(|capability| capability_is_unsupported(capability, current))
        .collect()
}

pub(crate) fn unsupported_declared_contract_capabilities_in_contract(
    contract: &Contract,
    current: &Version,
) -> Vec<&'static ContractCapabilitySpec> {
    detect_contract_capabilities_with(|capability| {
        capability_present_in_contract(capability, contract)
    })
    .into_iter()
    .filter(|capability| capability_is_unsupported(capability, current))
    .collect()
}

pub(crate) fn format_minimum_version_error(
    minimum_version: &str,
    current: &Version,
    capabilities: &[&ContractCapabilitySpec],
) -> String {
    format_minimum_version_error_with_identity(
        minimum_version,
        current,
        current_build_identity(),
        capabilities,
    )
}

pub(crate) fn format_minimum_version_upgrade_hint(
    minimum_version: &str,
    current: &Version,
    capabilities: &[&ContractCapabilitySpec],
) -> String {
    format_minimum_version_upgrade_hint_with_identity(
        minimum_version,
        current,
        current_build_identity(),
        capabilities,
    )
}

fn detect_contract_capabilities_with(
    mut predicate: impl FnMut(&ContractCapabilitySpec) -> bool,
) -> Vec<&'static ContractCapabilitySpec> {
    let mut detected = Vec::new();
    for capability in CONTRACT_CAPABILITY_SPECS {
        if predicate(capability) {
            detected.push(capability);
        }
    }
    detected
}

fn capability_is_unsupported(capability: &ContractCapabilitySpec, current: &Version) -> bool {
    Version::parse(capability.introduced_in)
        .map(|introduced| current < &introduced)
        .unwrap_or(false)
}

pub(crate) fn current_build_identity() -> BuildIdentity {
    BuildIdentity {
        source_build: option_env!("OTA_BUILD_SOURCE").is_some(),
        commit: option_env!("OTA_BUILD_COMMIT"),
        dirty: option_env!("OTA_BUILD_DIRTY").is_some(),
    }
}

fn format_minimum_version_error_with_identity(
    minimum_version: &str,
    current: &Version,
    identity: BuildIdentity,
    capabilities: &[&ContractCapabilitySpec],
) -> String {
    let mut lines = Vec::new();
    if let Some(feature_line) = format_contract_feature_mismatch_line(capabilities) {
        lines.push(feature_line);
    }
    lines.push(format!(
        "Contract minimum: Ota >= `{minimum_version}` via `metadata.ota.minimum_version`"
    ));
    lines.push(format!(
        "Current binary: `{}`",
        render_build_identity_summary(current, identity)
    ));
    lines.push(format_upgrade_guidance_line(minimum_version, identity));
    lines.join("\n")
}

fn format_minimum_version_upgrade_hint_with_identity(
    minimum_version: &str,
    current: &Version,
    identity: BuildIdentity,
    capabilities: &[&ContractCapabilitySpec],
) -> String {
    let mut parts = Vec::new();
    if let Some(feature_line) = format_contract_feature_mismatch_line(capabilities) {
        parts.push(feature_line);
    }
    parts.push(format!(
        "contract minimum is Ota >= `{minimum_version}` via `metadata.ota.minimum_version`"
    ));
    parts.push(format!(
        "current binary is `{}`",
        render_build_identity_summary(current, identity)
    ));
    parts.push(format_upgrade_guidance_line(minimum_version, identity));
    parts.join("; ")
}

fn format_contract_feature_mismatch_line(
    capabilities: &[&ContractCapabilitySpec],
) -> Option<String> {
    if capabilities.is_empty() {
        return None;
    }

    let details = capabilities
        .iter()
        .map(|capability| {
            format!(
                "`{}` (introduced in Ota {})",
                capability.id, capability.introduced_in
            )
        })
        .collect::<Vec<_>>()
        .join(", ");
    let label = if capabilities.len() == 1 {
        "Unsupported contract feature"
    } else {
        "Unsupported contract features"
    };
    Some(format!("{label}: {details}"))
}

fn render_build_identity_summary(current: &Version, identity: BuildIdentity) -> String {
    let mut details = Vec::new();
    details.push(if identity.source_build {
        String::from("source build")
    } else {
        String::from("release build")
    });
    if let Some(commit) = identity.commit {
        details.push(format!("commit {commit}"));
    }
    if identity.dirty {
        details.push(String::from("dirty"));
    }
    format!("Ota {} ({})", current, details.join(", "))
}

fn format_upgrade_guidance_line(minimum_version: &str, identity: BuildIdentity) -> String {
    if identity.source_build {
        String::from(format!(
            "Next: rebuild this source checkout from Ota {minimum_version} or newer, then rerun `ota --version --json` to confirm capability support"
        ))
    } else {
        String::from(format!(
            "Next: install Ota >= `{minimum_version}` and rerun `ota --version --json` to confirm capability support"
        ))
    }
}

fn capability_present_in_document(capability: &ContractCapabilitySpec, document: &Value) -> bool {
    match capability.id {
        "toolchains" => nonempty_mapping_child(document, "toolchains"),
        "execution.contexts.only_on" => execution_context_only_on_present(document),
        "metadata.ota.minimum_version" => {
            document_has_path(document, &["metadata", "ota", "minimum_version"])
        }
        "tasks.effects.writes" => tasks_effects_writes_present(document),
        "tasks.effects.network" => tasks_effects_network_present(document),
        "tasks.effects.external_state" => tasks_effects_external_state_present(document),
        "agent.posture" => document_has_path(document, &["agent", "posture"]),
        "agent.exceptions.sensitive_writes" => {
            document_has_path(document, &["agent", "exceptions", "sensitive_writes"])
        }
        "native_prerequisites.visual_studio" => {
            native_prerequisites_visual_studio_present(document)
        }
        "native_prerequisites.requires" => native_prerequisites_requires_present(document),
        _ => false,
    }
}

fn capability_present_in_contract(
    capability: &ContractCapabilitySpec,
    contract: &Contract,
) -> bool {
    match capability.id {
        "toolchains" => !contract.toolchains.is_empty(),
        "execution.contexts.only_on" => contract
            .execution
            .as_ref()
            .map(|execution| {
                execution.contexts.values().any(|context| {
                    context
                        .only_on
                        .as_ref()
                        .is_some_and(|items| !items.is_empty())
                })
            })
            .unwrap_or(false),
        "metadata.ota.minimum_version" => contract.minimum_ota_version().is_some(),
        "tasks.effects.writes" => contract
            .tasks
            .values()
            .any(|task| !task.effects.writes.is_empty()),
        "tasks.effects.network" => contract.tasks.values().any(|task| task.effects.network),
        "tasks.effects.external_state" => contract
            .tasks
            .values()
            .any(|task| !task.effects.external_state.is_empty()),
        "agent.posture" => contract
            .agent
            .as_ref()
            .map(|agent| agent.posture != AgentPosture::ReadinessStrict)
            .unwrap_or(false),
        "agent.exceptions.sensitive_writes" => contract
            .agent
            .as_ref()
            .map(|agent| !agent.sensitive_writable_paths().is_empty())
            .unwrap_or(false),
        "native_prerequisites.visual_studio" => {
            contract.native_prerequisites.values().any(|prerequisite| {
                prerequisite
                    .platforms
                    .values()
                    .any(|platform| platform.visual_studio.is_some())
            })
        }
        "native_prerequisites.requires" => {
            contract.native_prerequisites.values().any(|prerequisite| {
                prerequisite.platforms.values().any(|platform| {
                    !platform.requires.runtimes.is_empty() || !platform.requires.tools.is_empty()
                })
            })
        }
        _ => false,
    }
}

fn document_has_path(document: &Value, path: &[&str]) -> bool {
    let mut current = document;
    for segment in path {
        let Some(mapping) = current.as_mapping() else {
            return false;
        };
        let Some(next) = mapping.get(Value::String((*segment).to_owned())) else {
            return false;
        };
        current = next;
    }
    true
}

fn tasks_effects_writes_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_effects_writes_present)
}

fn tasks_effects_network_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_effects_network_present)
}

fn tasks_effects_external_state_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_effects_external_state_present)
}

fn execution_context_only_on_present(document: &Value) -> bool {
    let Some(contexts) = mapping_child(document, "execution")
        .and_then(|execution| mapping_child(execution, "contexts"))
        .and_then(Value::as_mapping)
    else {
        return false;
    };
    contexts
        .values()
        .any(|context| document_has_path(context, &["only_on"]))
}

fn task_effects_writes_present(task: &Value) -> bool {
    document_has_path(task, &["effects", "writes"])
}

fn task_effects_network_present(task: &Value) -> bool {
    document_has_path(task, &["effects", "network"])
}

fn task_effects_external_state_present(task: &Value) -> bool {
    document_has_path(task, &["effects", "external_state"])
}

fn native_prerequisites_visual_studio_present(document: &Value) -> bool {
    let Some(native_prerequisites) =
        mapping_child(document, "native_prerequisites").and_then(Value::as_mapping)
    else {
        return false;
    };
    native_prerequisites.values().any(|prerequisite| {
        mapping_child(prerequisite, "platforms")
            .and_then(Value::as_mapping)
            .map(|platforms| {
                platforms
                    .values()
                    .any(|platform| document_has_path(platform, &["visual_studio"]))
            })
            .unwrap_or(false)
    })
}

fn native_prerequisites_requires_present(document: &Value) -> bool {
    let Some(native_prerequisites) =
        mapping_child(document, "native_prerequisites").and_then(Value::as_mapping)
    else {
        return false;
    };
    native_prerequisites.values().any(|prerequisite| {
        mapping_child(prerequisite, "platforms")
            .and_then(Value::as_mapping)
            .map(|platforms| {
                platforms
                    .values()
                    .any(|platform| document_has_path(platform, &["requires"]))
            })
            .unwrap_or(false)
    })
}

fn mapping_child<'a>(value: &'a Value, key: &str) -> Option<&'a Value> {
    value.as_mapping()?.get(Value::String(key.to_owned()))
}

fn nonempty_mapping_child(value: &Value, key: &str) -> bool {
    mapping_child(value, key)
        .and_then(Value::as_mapping)
        .map(|mapping| !mapping.is_empty())
        .unwrap_or(false)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detects_declared_contract_capabilities_from_document() {
        let document: Value = serde_yaml::from_str(
            r#"
toolchains:
  node:
    provider: corepack
    package_manager: pnpm
execution:
  contexts:
    host:
      only_on:
        - linux
metadata:
  ota:
    minimum_version: "1.6.15"
agent:
  posture: readiness_strict
  exceptions:
    sensitive_writes:
      - .github/workflows
tasks:
  setup:
    effects:
      writes:
        - node_modules
      network: true
      external_state:
        - docker
native_prerequisites:
  node-native-build-tools:
    platforms:
      windows:
        visual_studio:
          components:
            - Microsoft.VisualStudio.Component.VC.Tools.x86.x64
        requires:
          runtimes:
            python: ">=3.10"
"#,
        )
        .unwrap();

        let detected = detect_declared_contract_capabilities(&document)
            .into_iter()
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        assert_eq!(
            detected,
            vec![
                "toolchains",
                "execution.contexts.only_on",
                "metadata.ota.minimum_version",
                "tasks.effects.writes",
                "tasks.effects.network",
                "tasks.effects.external_state",
                "agent.posture",
                "agent.exceptions.sensitive_writes",
                "native_prerequisites.visual_studio",
                "native_prerequisites.requires",
            ]
        );
    }

    #[test]
    fn detects_unsupported_declared_contract_capabilities_against_current_version() {
        let document: Value = serde_yaml::from_str(
            r#"
toolchains:
  node:
    provider: corepack
    package_manager: pnpm
execution:
  contexts:
    host:
      only_on:
        - linux
agent:
  posture: readiness_strict
  exceptions:
    sensitive_writes:
      - .github/workflows
"#,
        )
        .unwrap();

        let current = Version::parse("1.6.14").unwrap();
        let detected = unsupported_declared_contract_capabilities(&document, &current)
            .into_iter()
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        assert_eq!(
            detected,
            vec![
                "toolchains",
                "execution.contexts.only_on",
                "agent.posture",
                "agent.exceptions.sensitive_writes",
            ]
        );
    }

    #[test]
    fn detects_non_default_agent_posture_from_contract() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
agent:
  posture: infra_authoring
"#,
        )
        .unwrap();

        let current = Version::parse("1.6.14").unwrap();
        let detected = unsupported_declared_contract_capabilities_in_contract(&contract, &current)
            .into_iter()
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        assert_eq!(detected, vec!["agent.posture"]);
    }

    #[test]
    fn formats_minimum_version_error_with_capability_hint() {
        let message = format_minimum_version_error(
            "1.6.15",
            &Version::parse("1.6.14").unwrap(),
            &[&CONTRACT_CAPABILITY_SPECS[0], &CONTRACT_CAPABILITY_SPECS[1]],
        );
        assert!(
            message.contains("Unsupported contract features:"),
            "{message}"
        );
        assert!(
            message.contains("Contract minimum: Ota >= `1.6.15`"),
            "{message}"
        );
        assert!(message.contains("Current binary: `Ota 1.6.14"), "{message}");
        assert!(message.contains("Next: "), "{message}");
        assert!(message.contains("`toolchains`"));
        assert!(message.contains("`execution.contexts.only_on`"));
    }

    #[test]
    fn minimum_version_upgrade_guidance_changes_for_source_builds() {
        let current = Version::parse("1.6.14").unwrap();
        let release = format_minimum_version_upgrade_hint_with_identity(
            "1.6.15",
            &current,
            BuildIdentity {
                source_build: false,
                commit: None,
                dirty: false,
            },
            &[&CONTRACT_CAPABILITY_SPECS[5]],
        );
        assert!(release.contains("install Ota >= `1.6.15`"), "{release}");

        let source = format_minimum_version_upgrade_hint_with_identity(
            "1.6.15",
            &current,
            BuildIdentity {
                source_build: true,
                commit: Some("abc1234"),
                dirty: true,
            },
            &[&CONTRACT_CAPABILITY_SPECS[5]],
        );
        assert!(
            source.contains("rebuild this source checkout from Ota 1.6.15 or newer"),
            "{source}"
        );
        assert!(
            source.contains("current binary is `Ota 1.6.14 (source build, commit abc1234, dirty)`"),
            "{source}"
        );
    }
}
