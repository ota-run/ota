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
        id: "resource_bindings",
        introduced_in: "1.6.27",
    },
    ContractCapabilitySpec {
        id: "effect_definitions",
        introduced_in: "1.6.27",
    },
    ContractCapabilitySpec {
        id: "secret_requirements",
        introduced_in: "1.6.28",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.declared",
        introduced_in: "1.6.27",
    },
    ContractCapabilitySpec {
        id: "tasks.action.database_schema_mutation",
        introduced_in: "1.6.27",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.writes",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.workspace_writes",
        introduced_in: "1.6.23",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.network",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.network_kind",
        introduced_in: "1.6.16",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.adapter_state",
        introduced_in: "1.6.22",
    },
    ContractCapabilitySpec {
        id: "tasks.effects.external_state",
        introduced_in: "1.6.15",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_env_file",
        introduced_in: "1.6.16",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_file",
        introduced_in: "1.6.16",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_directory",
        introduced_in: "1.6.16",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_git_checkout",
        introduced_in: "1.6.22",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_git_template",
        introduced_in: "1.6.23",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_git_checkout.remotes",
        introduced_in: "1.6.23",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_git_checkouts",
        introduced_in: "1.6.23",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_container_network",
        introduced_in: "1.6.21",
    },
    ContractCapabilitySpec {
        id: "tasks.action.build_container_image",
        introduced_in: "1.6.24",
    },
    ContractCapabilitySpec {
        id: "tasks.action.reset_compose_service_volume",
        introduced_in: "1.6.22",
    },
    ContractCapabilitySpec {
        id: "tasks.action.ensure_bundle",
        introduced_in: "1.6.17",
    },
    ContractCapabilitySpec {
        id: "checks.changed_files",
        introduced_in: "1.6.16",
    },
    ContractCapabilitySpec {
        id: "tasks.when.checks",
        introduced_in: "1.6.17",
    },
    ContractCapabilitySpec {
        id: "services.readiness.compose_health",
        introduced_in: "1.6.16",
    },
    ContractCapabilitySpec {
        id: "tasks.runtime.readiness.signal_probes",
        introduced_in: "1.6.16",
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
        "resource_bindings" => nonempty_mapping_child(document, "resource_bindings"),
        "effect_definitions" => nonempty_mapping_child(document, "effect_definitions"),
        "secret_requirements" => nonempty_mapping_child(document, "secret_requirements"),
        "tasks.effects.declared" => tasks_effects_declared_present(document),
        "tasks.effects.writes" => tasks_effects_writes_present(document),
        "tasks.effects.workspace_writes" => tasks_effects_workspace_writes_present(document),
        "tasks.effects.network" => tasks_effects_network_present(document),
        "tasks.effects.network_kind" => tasks_effects_network_kind_present(document),
        "tasks.effects.adapter_state" => tasks_effects_adapter_state_present(document),
        "tasks.effects.external_state" => tasks_effects_external_state_present(document),
        "tasks.action.ensure_env_file" => tasks_action_ensure_env_file_present(document),
        "tasks.action.ensure_file" => tasks_action_ensure_file_present(document),
        "tasks.action.ensure_directory" => tasks_action_ensure_directory_present(document),
        "tasks.action.ensure_git_checkout" => tasks_action_ensure_git_checkout_present(document),
        "tasks.action.ensure_git_template" => tasks_action_ensure_git_template_present(document),
        "tasks.action.ensure_git_checkout.remotes" => {
            tasks_action_ensure_git_checkout_remotes_present(document)
        }
        "tasks.action.ensure_git_checkouts" => tasks_action_ensure_git_checkouts_present(document),
        "tasks.action.ensure_container_network" => {
            tasks_action_ensure_container_network_present(document)
        }
        "tasks.action.build_container_image" => {
            tasks_action_build_container_image_present(document)
        }
        "tasks.action.reset_compose_service_volume" => {
            tasks_action_reset_compose_service_volume_present(document)
        }
        "tasks.action.ensure_bundle" => tasks_action_ensure_bundle_present(document),
        "checks.changed_files" => checks_changed_files_present(document),
        "tasks.when.checks" => tasks_when_checks_present(document),
        "services.readiness.compose_health" => services_readiness_compose_health_present(document),
        "tasks.runtime.readiness.signal_probes" => {
            tasks_runtime_readiness_signal_probes_present(document)
        }
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
        "resource_bindings" => !contract.resource_bindings.is_empty(),
        "effect_definitions" => !contract.effect_definitions.is_empty(),
        "secret_requirements" => !contract.secret_requirements.is_empty(),
        "tasks.effects.declared" => contract
            .tasks
            .values()
            .any(|task| !task.effects.declared.is_empty()),
        "tasks.effects.writes" => contract
            .tasks
            .values()
            .any(|task| !task.effects.writes.is_empty()),
        "tasks.effects.workspace_writes" => contract
            .tasks
            .values()
            .any(|task| !task.effects.workspace_writes.is_empty()),
        "tasks.effects.network" => contract.tasks.values().any(|task| task.effects.network),
        "tasks.effects.network_kind" => contract
            .tasks
            .values()
            .any(|task| task.effects.network_kind.is_some()),
        "tasks.effects.adapter_state" => contract
            .tasks
            .values()
            .any(|task| !task.effects.adapter_state.is_empty()),
        "tasks.effects.external_state" => contract
            .tasks
            .values()
            .any(|task| !task.effects.external_state.is_empty()),
        "tasks.action.ensure_env_file" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::EnsureEnvFile(_))
            )
        }),
        "tasks.action.ensure_file" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::EnsureFile(_))
            )
        }),
        "tasks.action.ensure_directory" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::EnsureDirectory(_))
            )
        }),
        "tasks.action.ensure_git_checkout" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::EnsureGitCheckout(_))
            )
        }),
        "tasks.action.ensure_git_template" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::EnsureGitTemplate(_))
            )
        }),
        "tasks.action.ensure_git_checkout.remotes" => {
            contract
                .tasks
                .values()
                .any(|task| match task.action.as_ref() {
                    Some(crate::schema::TaskActionSpec::EnsureGitCheckout(spec)) => {
                        !spec.remotes.is_empty()
                    }
                    Some(crate::schema::TaskActionSpec::EnsureGitCheckouts(spec)) => spec
                        .checkouts
                        .iter()
                        .any(|checkout| !checkout.remotes.is_empty()),
                    _ => false,
                })
        }
        "tasks.action.ensure_git_checkouts" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::EnsureGitCheckouts(_))
            )
        }),
        "tasks.action.ensure_container_network" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::EnsureContainerNetwork(_))
            )
        }),
        "tasks.action.build_container_image" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::BuildContainerImage(_))
            )
        }),
        "tasks.action.reset_compose_service_volume" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::ResetComposeServiceVolume(_))
            )
        }),
        "tasks.action.ensure_bundle" => contract.tasks.values().any(|task| {
            matches!(
                task.action.as_ref(),
                Some(crate::schema::TaskActionSpec::EnsureBundle(_))
            )
        }),
        "checks.changed_files" => contract
            .checks
            .iter()
            .any(|check| check.kind == crate::schema::CheckKind::ChangedFiles),
        "tasks.when.checks" => contract.tasks.values().any(|task| !task.when.is_empty()),
        "services.readiness.compose_health" => contract.services.values().any(|service| {
            service.readiness.as_ref().is_some_and(|readiness| {
                matches!(
                    readiness.kind,
                    Some(crate::schema::ServiceReadinessKind::ComposeHealth)
                )
            })
        }),
        "tasks.runtime.readiness.signal_probes" => contract.tasks.values().any(|task| {
            task.runtime
                .as_ref()
                .and_then(|runtime| runtime.readiness.as_ref())
                .is_some_and(|readiness| !readiness.signal_probes.is_empty())
        }),
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

fn tasks_effects_workspace_writes_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_effects_workspace_writes_present)
}

fn tasks_effects_network_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_effects_network_present)
}

fn tasks_effects_network_kind_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_effects_network_kind_present)
}

fn tasks_effects_external_state_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_effects_external_state_present)
}

fn tasks_effects_declared_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks
        .values()
        .any(|task| document_has_path(task, &["effects", "declared"]))
}

fn tasks_effects_adapter_state_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_effects_adapter_state_present)
}

fn tasks_action_ensure_env_file_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_action_ensure_env_file_present)
}

fn tasks_action_ensure_file_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(task_action_ensure_file_present)
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

fn task_effects_workspace_writes_present(task: &Value) -> bool {
    document_has_path(task, &["effects", "workspace_writes"])
}

fn task_effects_network_present(task: &Value) -> bool {
    document_has_path(task, &["effects", "network"])
}

fn task_effects_network_kind_present(task: &Value) -> bool {
    document_has_path(task, &["effects", "network_kind"])
}

fn task_effects_external_state_present(task: &Value) -> bool {
    document_has_path(task, &["effects", "external_state"])
}

fn task_effects_adapter_state_present(task: &Value) -> bool {
    document_has_path(task, &["effects", "adapter_state"])
}

fn task_action_ensure_env_file_present(task: &Value) -> bool {
    mapping_child(task, "action")
        .and_then(|action| mapping_child(action, "kind"))
        .and_then(Value::as_str)
        .is_some_and(|kind| kind == "ensure_env_file")
}

fn task_action_ensure_file_present(task: &Value) -> bool {
    mapping_child(task, "action")
        .and_then(|action| mapping_child(action, "kind"))
        .and_then(Value::as_str)
        .is_some_and(|kind| kind == "ensure_file")
}

fn tasks_runtime_readiness_signal_probes_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks
        .values()
        .any(task_runtime_readiness_signal_probes_present)
}

fn task_runtime_readiness_signal_probes_present(task: &Value) -> bool {
    document_has_path(task, &["runtime", "readiness", "signal_probes"])
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

fn tasks_action_ensure_directory_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        mapping_child(task, "action")
            .and_then(Value::as_mapping)
            .and_then(|action| {
                action
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("ensure_directory")
    })
}

fn tasks_action_ensure_bundle_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        mapping_child(task, "action")
            .and_then(Value::as_mapping)
            .and_then(|action| {
                action
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("ensure_bundle")
    })
}

fn tasks_action_ensure_container_network_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        mapping_child(task, "action")
            .and_then(Value::as_mapping)
            .and_then(|action| {
                action
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("ensure_container_network")
    })
}

fn tasks_action_build_container_image_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        mapping_child(task, "action")
            .and_then(Value::as_mapping)
            .and_then(|action| {
                action
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("build_container_image")
    })
}

fn tasks_action_ensure_git_checkout_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        mapping_child(task, "action")
            .and_then(Value::as_mapping)
            .and_then(|action| {
                action
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("ensure_git_checkout")
    })
}

fn tasks_action_ensure_git_template_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        mapping_child(task, "action")
            .and_then(Value::as_mapping)
            .and_then(|action| {
                action
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("ensure_git_template")
    })
}

fn tasks_action_ensure_git_checkouts_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        mapping_child(task, "action")
            .and_then(Value::as_mapping)
            .and_then(|action| {
                action
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("ensure_git_checkouts")
    })
}

fn tasks_action_ensure_git_checkout_remotes_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        let Some(action) = mapping_child(task, "action").and_then(Value::as_mapping) else {
            return false;
        };
        match action
            .get(Value::String(String::from("kind")))
            .and_then(Value::as_str)
        {
            Some("ensure_git_checkout") => action
                .get(Value::String(String::from("remotes")))
                .and_then(Value::as_sequence)
                .is_some_and(|remotes| !remotes.is_empty()),
            Some("ensure_git_checkouts") => action
                .get(Value::String(String::from("checkouts")))
                .and_then(Value::as_sequence)
                .is_some_and(|checkouts| {
                    checkouts.iter().any(|checkout| {
                        mapping_child(checkout, "remotes")
                            .and_then(Value::as_sequence)
                            .is_some_and(|remotes| !remotes.is_empty())
                    })
                }),
            _ => false,
        }
    })
}

fn tasks_action_reset_compose_service_volume_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks.values().any(|task| {
        mapping_child(task, "action")
            .and_then(Value::as_mapping)
            .and_then(|action| {
                action
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("reset_compose_service_volume")
    })
}

fn checks_changed_files_present(document: &Value) -> bool {
    let Some(checks) = mapping_child(document, "checks").and_then(Value::as_sequence) else {
        return false;
    };
    checks
        .iter()
        .any(|check| mapping_child(check, "kind").and_then(Value::as_str) == Some("changed_files"))
}

fn tasks_when_checks_present(document: &Value) -> bool {
    let Some(tasks) = mapping_child(document, "tasks").and_then(Value::as_mapping) else {
        return false;
    };
    tasks
        .values()
        .any(|task| document_has_path(task, &["when", "checks"]))
}

fn services_readiness_compose_health_present(document: &Value) -> bool {
    let Some(services) = mapping_child(document, "services").and_then(Value::as_mapping) else {
        return false;
    };
    services.values().any(|service| {
        mapping_child(service, "readiness")
            .and_then(Value::as_mapping)
            .and_then(|readiness| {
                readiness
                    .get(Value::String(String::from("kind")))
                    .and_then(Value::as_str)
            })
            == Some("compose_health")
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
    fn publishes_and_detects_v12_typed_effect_capabilities() {
        let document: Value = serde_yaml::from_str(
            r#"
resource_bindings:
  production_primary:
    kind: database
effect_definitions:
  production_schema_migration:
    kind: database_schema_mutation
tasks:
  db-migrate:
    effects:
      declared: [production_schema_migration]
"#,
        )
        .unwrap();
        let detected = detect_declared_contract_capabilities(&document)
            .into_iter()
            .map(|capability| (capability.id, capability.introduced_in))
            .collect::<Vec<_>>();
        assert_eq!(
            detected,
            vec![
                ("resource_bindings", "1.6.27"),
                ("effect_definitions", "1.6.27"),
                ("tasks.effects.declared", "1.6.27"),
            ]
        );
    }

    #[test]
    fn publishes_and_detects_v12_1_secret_requirement_capability() {
        let document: Value = serde_yaml::from_str(
            r#"
secret_requirements:
  provider_api_token:
    secret_class: authentication_credential
"#,
        )
        .unwrap();
        let detected = detect_declared_contract_capabilities(&document)
            .into_iter()
            .map(|capability| (capability.id, capability.introduced_in))
            .collect::<Vec<_>>();
        assert_eq!(detected, vec![("secret_requirements", "1.6.28")]);
    }

    #[test]
    fn detects_v12_1_secret_requirement_capability_from_parsed_contract() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: secret-capability
tasks:
  provider-conformance:
    command:
      exe: "true"
secret_requirements:
  provider_api_token:
    secret_class: authentication_credential
    purpose: external_api_authentication
    delivery:
      kind: process_environment
      variable: GOOGLE_API_KEY
    recipients:
      tasks: [provider-conformance]
      dependencies: deny
      hooks: deny
      services: deny
      helpers: deny
      containers: deny
      remote_execution: deny
      proof_observers: deny
      negative_controls: deny
      lifecycle_children: deny
    constraints:
      actor_mode: ci
      environment: test
      execution_mode: native
      target_platform: linux
      runtime_boundary: process
      capability: segmented_process_environment
"#,
        )
        .unwrap();
        let detected = unsupported_declared_contract_capabilities_in_contract(
            &contract,
            &Version::parse("1.6.27").unwrap(),
        )
        .into_iter()
        .map(|capability| (capability.id, capability.introduced_in))
        .collect::<Vec<_>>();

        assert_eq!(detected, vec![("secret_requirements", "1.6.28")]);
    }

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
    action:
      kind: ensure_env_file
      path: .env
      vars:
        DATABASE_URL:
          value: postgres://localhost/dev
    effects:
      network: true
      network_kind: dependency_hydration
  bootstrap:token:
    action:
      kind: ensure_file
      path: secrets/token.txt
      random:
        bytes: 32
        encoding: hex
  bootstrap:cache:
    action:
      kind: ensure_directory
      path: .cache/dev
  bootstrap:network:
    action:
      kind: ensure_container_network
      name: penpot_shared
    when:
      checks:
        - web-changed
    runtime:
      kind: service
      readiness:
        kind: tcp
        listener: backend
        signal_probes:
          - backend-worker
      listeners:
        backend:
          protocol: tcp
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
    effects:
      writes:
        - node_modules
      network: true
      network_kind: dependency_hydration
      adapter_state:
        - compose_volume:node_modules
      external_state:
        - docker
  bootstrap:bundle:
    action:
      kind: ensure_bundle
      steps:
        - kind: ensure_directory
          path: .cache/setup
        - kind: ensure_file
          path: config/token.txt
          value: dev-token
readiness:
  probes:
    backend-worker:
      kind: tcp
      timeout: 1000
      target:
        kind: task
        name: setup
        listener: backend
checks:
  - name: web-changed
    kind: changed_files
    severity: info
    changed_files:
      paths:
        - apps/web/**
services:
  worker:
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: worker
    readiness:
      kind: compose_health
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
                "tasks.effects.network_kind",
                "tasks.effects.adapter_state",
                "tasks.effects.external_state",
                "tasks.action.ensure_env_file",
                "tasks.action.ensure_file",
                "tasks.action.ensure_directory",
                "tasks.action.ensure_container_network",
                "tasks.action.ensure_bundle",
                "checks.changed_files",
                "tasks.when.checks",
                "services.readiness.compose_health",
                "tasks.runtime.readiness.signal_probes",
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
tasks:
  setup:
    action:
      kind: ensure_env_file
      path: .env
      vars:
        DATABASE_URL:
          value: postgres://localhost/dev
    effects:
      network: true
      network_kind: dependency_hydration
  bootstrap:token:
    action:
      kind: ensure_file
      path: secrets/token.txt
      random:
        bytes: 32
        encoding: hex
  bootstrap:cache:
    action:
      kind: ensure_directory
      path: .cache/dev
  bootstrap:network:
    action:
      kind: ensure_container_network
      name: penpot_shared
    when:
      checks:
        - web-changed
    runtime:
      kind: service
      readiness:
        kind: tcp
        listener: backend
        signal_probes:
          - backend-worker
      listeners:
        backend:
          protocol: tcp
          bind:
            address: 127.0.0.1
            port:
              mode: fixed
              value: 3000
          project:
            host:
              address: 127.0.0.1
              port:
                mode: fixed
                value: 3000
  bootstrap:bundle:
    action:
      kind: ensure_bundle
      steps:
        - kind: ensure_directory
          path: .cache/setup
        - kind: ensure_file
          path: config/token.txt
          value: dev-token
readiness:
  probes:
    backend-worker:
      kind: tcp
      timeout: 1000
      target:
        kind: task
        name: setup
        listener: backend
checks:
  - name: web-changed
    kind: changed_files
    severity: info
    changed_files:
      paths:
        - apps/web/**
services:
  worker:
    manager:
      kind: compose
      name: local
      file: compose.yaml
      service: worker
    readiness:
      kind: compose_health
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
                "tasks.effects.network",
                "tasks.effects.network_kind",
                "tasks.action.ensure_env_file",
                "tasks.action.ensure_file",
                "tasks.action.ensure_directory",
                "tasks.action.ensure_container_network",
                "tasks.action.ensure_bundle",
                "checks.changed_files",
                "tasks.when.checks",
                "services.readiness.compose_health",
                "tasks.runtime.readiness.signal_probes",
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
    fn detects_ensure_git_checkout_capability_from_contract() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:deps:
    action:
      kind: ensure_git_checkout
      path: vendor/wagtail
      source:
        git: https://github.com/wagtail/wagtail.git
        ref: main
"#,
        )
        .unwrap();

        let current = Version::parse("1.6.21").unwrap();
        let detected = unsupported_declared_contract_capabilities_in_contract(&contract, &current)
            .into_iter()
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        assert_eq!(detected, vec!["tasks.action.ensure_git_checkout"]);
    }

    #[test]
    fn detects_ensure_git_template_capability_from_contract() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
tasks:
  scaffold:
    action:
      kind: ensure_git_template
      path: my-extension
      source:
        git: https://github.com/codyhxyz/create-chrome-extension.git
        ref: main
"#,
        )
        .unwrap();

        let current = Version::parse("1.6.22").unwrap();
        let detected = unsupported_declared_contract_capabilities_in_contract(&contract, &current)
            .into_iter()
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        assert_eq!(detected, vec!["tasks.action.ensure_git_template"]);
    }

    #[test]
    fn detects_workspace_writes_capability_from_contract() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
tasks:
  bootstrap:ui:
    run: echo ok
    effects:
      workspace_writes:
        - ../ui
"#,
        )
        .unwrap();

        let current = Version::parse("1.6.22").unwrap();
        let detected = unsupported_declared_contract_capabilities_in_contract(&contract, &current)
            .into_iter()
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        assert_eq!(detected, vec!["tasks.effects.workspace_writes"]);
    }

    #[test]
    fn detects_ensure_git_checkouts_capability_from_contract() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:deps:
    action:
      kind: ensure_git_checkouts
      checkouts:
        - path: vendor/wagtail
          source:
            git: https://github.com/wagtail/wagtail.git
            ref: main
"#,
        )
        .unwrap();

        let current = Version::parse("1.6.22").unwrap();
        let detected = unsupported_declared_contract_capabilities_in_contract(&contract, &current)
            .into_iter()
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        assert_eq!(detected, vec!["tasks.action.ensure_git_checkouts"]);
    }

    #[test]
    fn detects_ensure_git_checkout_remotes_capability_from_contract() {
        let contract: Contract = serde_yaml::from_str(
            r#"
version: 1
project:
  name: ota
tasks:
  setup:deps:
    action:
      kind: ensure_git_checkout
      path: vendor/wagtail
      source:
        git: https://github.com/wagtail/wagtail.git
        ref: main
      remotes:
        - name: upstream
          git: git@github.com:wagtail/wagtail.git
"#,
        )
        .unwrap();

        let current = Version::parse("1.6.22").unwrap();
        let detected = unsupported_declared_contract_capabilities_in_contract(&contract, &current)
            .into_iter()
            .map(|capability| capability.id)
            .collect::<Vec<_>>();
        assert_eq!(detected, vec!["tasks.action.ensure_git_checkout.remotes"]);
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
